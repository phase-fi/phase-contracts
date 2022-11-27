use cosmwasm_std::{
    to_binary, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg,
};
use cw_asset::Asset;
use phase_finance::constants::{APOLLO_ROUTER_ADDRESS, DCA_SWAP_ID};
use phase_finance::error::ContractError;
use phase_finance::types::SwapEvent;

use crate::state::{CONFIG, DCA_RECORD};

pub fn try_cancel_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.strategy_creator {
        return Err(ContractError::Unauthorized {});
    }

    let balances = deps.querier.query_all_balances(env.contract.address)?;

    if balances.len() == 0 {
        return Err(ContractError::NoBalance {});
    }

    let msg = BankMsg::Send {
        to_address: config.strategy_creator.to_string(),
        amount: balances,
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "try_cancel_dca"))
}

// pub fn try_withdraw(deps: DepsMut, env: Env, info: MessageInfo)-> Result<Response, ContractError> {}

pub fn try_perform_dca(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let dca_record = DCA_RECORD.load(deps.storage)?;

    let last_swap_event = dca_record.get_last_swap_event(&env);
    let next_swap_event = dca_record.get_next_swap_event(&env);
    let next_swap_event_time = match next_swap_event {
        Some(event) => event.timestamp_nanos.to_string(),
        None => "never".to_string(),
    };

    // todo: Throw this all into a validate_dca_execution function
    match last_swap_event {
        Some(last_swap_event) => {
            // if last swap event was already executed, do not allow another swap for this period
            if last_swap_event.executed {
                return Err(ContractError::SwapAlreadyExecuted { next_swap_event_time });
            } 
        }
        None => return Err(ContractError::CustomError {
            val: "DCA swap not allowed yet, next execution is ".to_owned() + &next_swap_event_time,
        }),
    }
    // let balance = deps
    //     .querier
    //     .query_balance(env.contract.address, config.source.denom.clone())?;

    let in_funds = vec![Coin {
        denom: config.source.denom.clone(),
        amount: config.amount_per_trade,
    }];

    let to_assets = config
        .destinations
        .iter()
        .map(|d| Asset {
            amount: d.weight,
            info: cw_asset::AssetInfoBase::Native(d.denom.clone()),
        })
        .collect();

    // todo: perform swap via router
    let msg = WasmMsg::Execute {
        contract_addr: APOLLO_ROUTER_ADDRESS.to_string(),
        msg: to_binary(&apollo_router::msg::ExecuteMsg::Swap {
            to: apollo_router::msg::SwapToAssetsInput::Multi(to_assets),
            max_spread: Option::Some(Decimal::from_ratio(5u128, 1000u128)), // todo: max spread
            recipient: Option::Some(config.strategy_creator.to_string()),
            hook_msg: Option::None,
        })?,
        funds: in_funds,
    };

    // add the message to swap & send funds to user
    Ok(Response::new()
        .add_submessage(SubMsg::reply_always(msg, DCA_SWAP_ID)) // todo:: handle reply
        .add_attribute("method", "try_perform_dca"))
}
