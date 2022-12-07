use cosmwasm_std::{
    to_binary, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128,
    WasmMsg,
};

use cw_asset::AssetInfoBase;
use cw_dex_router::operations::{SwapOperationsList, SwapOperationsListUnchecked};
use phase_finance::constants::DCA_SWAP_ID;
use phase_finance::error::ContractError;

use crate::helpers::{can_execute, get_next_swap_time};
use crate::state::{CONFIG, STATE};

pub fn try_cancel_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    let balances = deps.querier.query_all_balances(env.contract.address)?;

    if balances.len() == 0 {
        return Err(ContractError::NoBalance {});
    }

    let msg = BankMsg::Send {
        to_address: config.owner.to_string(),
        amount: balances,
    };

    // todo: Do we have to handle failed sends?
    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "try_cancel_dca"))
}

pub fn pause_dca(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if (info.sender != config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    // update state with paused = true
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.paused = true;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "pause_dca"))
}

pub fn resume_dca(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if (info.sender != config.owner) {
        return Err(ContractError::Unauthorized {});
    }

    let next_swap_time = get_next_swap_time(&config, &state);

    // update state with paused = false & get the next swap time so we dont double exec on unpause
    // TODO: Do we want to execute the swap on unpause if after the next swap time?
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.paused = false;
        state.pending_swap = next_swap_time;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "resume_dca")
        .add_attribute(
            "next_swap_time",
            next_swap_time.unwrap_or(u64::MAX).to_string(),
        ))
}

pub fn try_perform_dca(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    if (state.paused) {
        return Err(ContractError::DcaPaused {});
    }

    let can_execute = can_execute(&env, &config, &state);
    if !can_execute {
        return Err(ContractError::DcaSwapNotAllowedYet {
            next_swap_event_time: state.pending_swap.unwrap_or(u64::MAX),
        });
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.source.denom.clone())?;

    // let in_funds = vec![Coin {
    //     denom: config.source.denom.clone(),
    //     amount: config.amount_per_trade,
    // }];

    // let to_assets = config
    //     .destinations
    //     .iter()
    //     .map(|d| Asset {
    //         amount: d.weight,
    //         info: cw_asset::AssetInfoBase::Native(d.denom.clone()),
    //     })
    //     .collect();

    // amount: config.amount_per_trade,
    //
    // amount: d
    //                 .weight
    //                 .checked_mul(config.amount_per_trade)
    //                 .unwrap()
    //                 .checked_div(total_weight)
    //                 .unwrap(),

    let total_weight = config
        .destinations
        .iter()
        .fold(Uint128::zero(), |acc, d| acc + d.weight);

    let source_asset = AssetInfoBase::Native(config.source.denom.clone());

    let msgs: Vec<SubMsg> = config
        .destinations
        .iter()
        .map(|d| {
            let path_query = cw_dex_router::msg::QueryMsg::PathForPair {
                offer_asset: source_asset.clone(),
                ask_asset: AssetInfoBase::Native(d.denom.clone()),
            };

            let swap_operations_list: SwapOperationsList = deps
                .querier
                .query(
                    &cosmwasm_std::WasmQuery::Smart {
                        contract_addr: config.router_contract.clone(),
                        msg: to_binary(&path_query).unwrap(),
                    }
                    .into(),
                )
                .unwrap();

            let in_funds = vec![Coin {
                denom: config.source.denom.clone(),
                amount: d
                    .weight
                    .checked_mul(config.amount_per_trade)
                    .unwrap()
                    .checked_div(total_weight)
                    .unwrap(),
            }];

            let msg = WasmMsg::Execute {
                contract_addr: config.router_contract.to_string(),
                funds: in_funds,
                msg: to_binary(&cw_dex_router::msg::ExecuteMsg::ExecuteSwapOperations {
                    operations: swap_operations_list.into(),
                    offer_amount: Option::None,    // needed for cw20s only
                    minimum_receive: Option::None, // todo: add min receive, can be done by adding support for osmo twap
                    to: Option::Some(config.owner.to_string()),
                })
                .unwrap(),
            };

            SubMsg::reply_always(msg, DCA_SWAP_ID)
        })
        .collect();

    // deps.querier.query_wasm_smart(config.router_contract, cw_dex_router::msg::QueryMsg::PathForPair { offer_asset: (), ask_asset: () })

    // let swapOperations = SwapOperationsListUnchecked::new(vec![]);

    // add the message to swap & send funds to user
    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("method", "try_perform_dca"))
}
