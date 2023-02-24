use cosmwasm_std::{
    ensure, ensure_eq, ensure_ne, to_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response,
    SubMsg, Uint128, WasmMsg,
};

use phase_finance::constants::DCA_SWAP_ID;
use phase_finance::error::ContractError;

use crate::helpers::get_expiration_time;
use crate::state::{CONFIG, STATE};

pub fn try_cancel_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(config.owner, info.sender, ContractError::Unauthorized {});

    let balances = deps.querier.query_all_balances(env.contract.address)?;
    if balances.is_empty() {
        return Err(ContractError::NoBalance {});
    }

    let msg = BankMsg::Send {
        to_address: config.owner,
        amount: balances,
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "try_cancel_dca"))
}

pub fn pause_dca(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    ensure!(!state.paused, ContractError::DcaPaused);
    ensure_eq!(config.owner, info.sender, ContractError::Unauthorized {});

    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.paused = true;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "pause_dca"))
}

pub fn resume_dca(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;
    ensure!(state.paused, ContractError::DcaNotPaused);
    ensure_eq!(config.owner, info.sender, ContractError::Unauthorized {});

    let state = STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if state.next_swap.is_expired(&env.block) {
            state.next_swap = config.swap_interval.after(&env.block);
        }
        state.paused = false;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "resume_dca")
        .add_attribute(
            "next_swap_time",
            get_expiration_time(state.next_swap).to_string(),
        ))
}

pub fn try_perform_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    ensure_eq!(
        config.executor_address,
        info.sender,
        ContractError::Unauthorized {}
    );

    ensure!(!state.paused, ContractError::DcaPaused);

    ensure_ne!(
        config.num_trades,
        state.num_trades_executed,
        ContractError::MaxTradeLimit {}
    );

    ensure!(
        state.next_swap.is_expired(&env.block),
        ContractError::DcaSwapNotAllowedYet {
            next_swap_event_time: get_expiration_time(state.next_swap)
        }
    );

    let total_weight = config
        .destinations
        .iter()
        .fold(Uint128::zero(), |acc, d| acc + d.weight);

    let msgs: Vec<SubMsg> = config
        .destinations
        .iter()
        .map(|d| {
            let in_funds = Coin {
                denom: config.source_denom.clone(),
                amount: d
                    .weight
                    .checked_mul(config.amount_per_trade)
                    .unwrap_or_default()
                    .checked_div(total_weight)
                    .unwrap_or_default(),
            };

            let msg = WasmMsg::Execute {
                contract_addr: config.router_contract.clone(),
                msg: to_binary(&swaprouter::msg::ExecuteMsg::Swap {
                    input_coin: in_funds.clone(),
                    output_denom: d.denom.clone(),
                    slippage: swaprouter::msg::Slippage::MaxSlippagePercentage(config.max_slippage),
                })
                .unwrap(),
                funds: vec![in_funds],
            };

            SubMsg::reply_always(msg, DCA_SWAP_ID)
        })
        .collect();

    // add the messages to swap & send funds to user
    Ok(Response::new()
        .add_submessages(msgs)
        .add_attribute("method", "try_perform_dca"))
}
