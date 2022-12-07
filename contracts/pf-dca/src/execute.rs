use cosmwasm_std::{
    to_binary, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg,
};
use cw_asset::Asset;
use cw_dex_router::operations::{SwapOperationsList, SwapOperationsListUnchecked};
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


    // add the message to swap & send funds to user
    Ok(Response::new()
        // .add_submessage(SubMsg::reply_always(msg, DCA_SWAP_ID)) // todo:: handle reply
        .add_attribute("method", "try_perform_dca"))
}
