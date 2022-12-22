use cosmwasm_std::{
    to_binary, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128,
    WasmMsg,
};

use phase_finance::constants::DCA_SWAP_ID;
use phase_finance::error::ContractError;

use crate::helpers::{can_execute, get_next_swap_time, verify_sender};
use crate::state::{CONFIG, STATE};

pub fn try_cancel_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    verify_sender(&config, &info)?;

    let balances = deps.querier.query_all_balances(env.contract.address)?;
    if balances.is_empty() {
        return Err(ContractError::NoBalance {});
    }

    let msg = BankMsg::Send {
        to_address: config.owner,
        amount: balances,
    };

    // todo: Do we have to handle failed sends?
    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "try_cancel_dca"))
}

pub fn pause_dca(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    verify_sender(&config, &info)?;

    // update state with paused = true
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.paused = true;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "pause_dca"))
}

pub fn resume_dca(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    verify_sender(&config, &info)?;

    // resume DCA will wait an extra interval, otherwise pausing and resuming means you can run through the entire DCA strat by just pausing and resuming
    // TODO: is that somethng we want? (see todo below as well - should users be able to execute the next swap RIGHT NOW if they want to?)
    let next_swap_time = get_next_swap_time(env.block.time.nanos(), &config, &state);

    // update state with paused = false & get the next swap time so we dont double exec on unpause
    // TODO: Do we want to execute the swap on unpause if after the next swap time?
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.paused = false;
        state.pending_swap_time_nanos = next_swap_time;
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

    if state.paused {
        return Err(ContractError::DcaPaused {});
    }

    let can_execute = can_execute(&env, &config, &state);
    if !can_execute {
        return Err(ContractError::DcaSwapNotAllowedYet {
            next_swap_event_time: state.pending_swap_time_nanos.unwrap_or(u64::MAX),
        });
    }

    // todo: balance checks here?
    // let _balance = deps
    //     .querier
    //     .query_balance(env.contract.address, config.source.denom.clone())?;

    let total_weight = config
        .destinations
        .iter()
        .fold(Uint128::zero(), |acc, d| acc + d.weight);

    let msgs: Vec<SubMsg> = config
        .destinations
        .iter()
        .map(|d| {
            let in_funds = Coin {
                denom: config.source.denom.clone(),
                amount: d
                    .weight
                    .checked_mul(config.amount_per_trade)
                    .unwrap()
                    .checked_div(total_weight)
                    .unwrap(),
            };

            let msg = WasmMsg::Execute {
                contract_addr: config.router_contract.clone(),
                msg: to_binary(&swaprouter::msg::ExecuteMsg::Swap {
                    input_coin: in_funds.clone(),
                    output_denom: d.denom.clone(),
                    slippage: swaprouter::msg::Slippage::MaxSlippagePercentage(Decimal::percent(
                        1u64,
                    )), // 1% slippage, todo: configure from config
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
