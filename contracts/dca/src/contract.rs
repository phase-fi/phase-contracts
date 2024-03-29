#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, Decimal, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsgResponse, Uint128,
};

use cw2::set_contract_version;
use cw_denom::validate_native_denom;
use cw_utils::must_pay;

use crate::execute::{pause_dca, resume_dca, try_cancel_dca, try_perform_dca};
use crate::helpers::token_string_to_coin;
use crate::query::{
    query_all_upcoming_swaps, query_bonded_funds, query_config, query_funds, query_state,
    query_upcoming_swap,
};
use crate::state::{CONFIG, STATE};

use phase_finance::constants::DCA_SWAP_ID;
use phase_finance::error::ContractError;
use phase_finance::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use phase_finance::types::{DcaConfig, State, SwapEvent};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:phase-finance";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// the maximum number of destination tokens to dca into
const MAX_DESTINATIONS: u8 = 25;
// the maximum value for max_slippage on swaps
const MAX_SLIPPAGE_PERCENTAGE: u128 = 15;
// the maximum twap window in seconds for swaps
const MAX_TWAP_WINDOW_SECONDS: u64 = 120;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let funds = must_pay(&info, &msg.source_denom)?;

    let expected_funds = msg
        .amount_per_trade
        .checked_mul(msg.num_trades)
        .expect("overflow")
        .checked_add(msg.platform_fee)
        .expect("overflow");

    // check that amount deposited is correct for dca params
    if expected_funds.ne(&funds) {
        return Err(ContractError::CustomError {
            val: format!(
                "Amount deposited does not match exactly expected: <{expected_funds}> != actual: <{funds}>"
            ),
        });
    }

    // check that number of destination tokens is no more than MAX_DESTINATIONS
    if msg.destinations.len() > MAX_DESTINATIONS.into() || msg.destinations.is_empty() {
        return Err(ContractError::CustomError {
            val: format!("Number of destination tokens must be between 1 and {MAX_DESTINATIONS}"),
        });
    }

    // validate max_slippage is less than MAX_SLIPPAGE_PERCENTAGE%
    if msg
        .max_slippage
        .gt(&Decimal::from_ratio(MAX_SLIPPAGE_PERCENTAGE, 100u128))
    {
        return Err(ContractError::CustomError {
            val: format!("Max slippage must be between 0% and {MAX_SLIPPAGE_PERCENTAGE}%"),
        });
    }

    // validate that twap_window_seconds is between 1 and MAX_TWAP_WINDOW_SECONDS seconds
    if msg.twap_window_seconds.gt(&MAX_TWAP_WINDOW_SECONDS) || msg.twap_window_seconds.lt(&1u64) {
        return Err(ContractError::CustomError {
            val: format!("Twap window must be between 1 and {MAX_TWAP_WINDOW_SECONDS} seconds"),
        });
    }

    // check that swap_interval is greater than 0
    let swap_interval_value = match msg.swap_interval {
        cw_utils::Duration::Height(height) => height,
        cw_utils::Duration::Time(time) => time,
    };
    if swap_interval_value == 0 {
        return Err(ContractError::CustomError {
            val: "Swap interval must be greater than 0".to_string(),
        });
    }

    let executor_address = match msg.executor_address {
        Some(executor_address) => deps.api.addr_validate(&executor_address)?,
        None => info.sender.clone(),
    };

    for destination in msg.destinations.iter() {
        validate_native_denom(destination.denom.clone())?;
    }

    // store config for this DCA
    let config = DcaConfig {
        owner: info.sender.to_string(),
        executor_address,
        recipient_address: deps.api.addr_validate(&msg.recipient_address)?.to_string(),
        strategy_type: msg.strategy_type,
        source_denom: msg.source_denom.clone(),
        destinations: msg.destinations,
        max_slippage: msg.max_slippage,
        twap_window_seconds: msg.twap_window_seconds,
        amount_per_trade: msg.amount_per_trade,
        num_trades: msg.num_trades,
        swap_interval: msg.swap_interval,
        router_contract: deps.api.addr_validate(&msg.router_contract)?,
    };

    let state = State {
        next_swap: msg.swap_interval.after(&env.block),
        paused: false,
        num_trades_executed: Uint128::zero(),
        swap_status: vec![],
    };

    CONFIG.save(deps.storage, &config)?;
    STATE.save(deps.storage, &state)?;

    let mut msgs = vec![];
    if !msg.platform_fee.is_zero() {
        msgs.push(BankMsg::Send {
            to_address: deps
                .api
                .addr_validate(&msg.platform_fee_recipient)?
                .to_string(),
            amount: vec![Coin {
                amount: msg.platform_fee,
                denom: msg.source_denom,
            }],
        });
    }

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PerformDca {} => try_perform_dca(deps, env, info),
        ExecuteMsg::PauseDca {} => pause_dca(deps, info),
        ExecuteMsg::ResumeDca {} => resume_dca(deps, env, info),
        ExecuteMsg::CancelDca {} => try_cancel_dca(deps, env, info),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        DCA_SWAP_ID => match msg.result {
            cosmwasm_std::SubMsgResult::Ok(reply_msg) => {
                // in the function below, we will need to get the swapEvent timestamp to avoid the edge case
                // where a swap is executed just before the next swap begins, and we receive the swap response after,
                // setting the swapEvent.executed value to true on the next swap event rather than the one we want.
                // But I am tired and I forgot how to pass the swapEvent key correctly.
                process_dca_swap_response(deps, env, reply_msg)
            }
            cosmwasm_std::SubMsgResult::Err(_) => {
                try_store_and_finish_dca_swap(deps, env, Option::None)
            }
        },
        _ => Err(StdError::GenericErr {
            msg: "unknown reply id".to_string(),
        }),
    }
}

pub fn try_store_and_finish_dca_swap(
    deps: DepsMut,
    env: Env,
    swap_event: Option<SwapEvent>,
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;
    let mut swap_status = state.swap_status;

    swap_status.push(swap_event.unwrap_or(SwapEvent {
        executed: false,
        token_in: Option::None,
        effective_token_out: Option::None,
        timestamp_nanos: env.block.time.nanos(),
    }));
    state.swap_status = swap_status;

    // if we have gotten back all the responses we were expecting, then we can finish the swap
    if state.swap_status.len() == config.destinations.len() {
        // now that we have attempted all swaps, we can send the destination coins to the destination wallet
        let msg = BankMsg::Send {
            to_address: config.recipient_address.to_string(),
            amount: state
                .swap_status
                .iter()
                .filter(|swap_event| swap_event.executed)
                .map(|swap_event| swap_event.effective_token_out.clone().unwrap())
                .collect(),
        };

        // reset the state
        // prepare for the next swap
        state = State {
            next_swap: config.swap_interval.after(&env.block),
            paused: state.paused,
            num_trades_executed: state
                .num_trades_executed
                .checked_add(Uint128::from(1u128))
                .unwrap(),
            swap_status: vec![],
        };

        // at this point, we have attempted all swaps, so save the state for the next swap
        STATE.save(deps.storage, &state)?;

        // respond with compiled swap events
        // todo: add all swap events to the response
        Ok(Response::new()
            .add_message(msg)
            .add_attribute("method", "try_store_and_finish_dca_swap"))
    } else {
        // we are still waiting for more responses
        STATE.save(deps.storage, &state)?;
        Ok(Response::new())
    }
}

pub fn process_dca_swap_response(
    deps: DepsMut,
    env: Env,
    reply_msg: SubMsgResponse,
) -> StdResult<Response> {
    // find the event with ty = token_swapped

    let event = reply_msg
        .events
        .iter()
        .find(|event| event.ty == "token_swapped");
    if event.is_none() {
        return try_store_and_finish_dca_swap(deps, env, Option::None);
    }

    // find the attribute with key = tokens_in
    let tokens_in_attr = event
        .unwrap()
        .attributes
        .iter()
        .find(|attr| attr.key == "tokens_in");
    if tokens_in_attr.is_none() {
        return try_store_and_finish_dca_swap(deps, env, Option::None);
    }
    let token_in = tokens_in_attr.unwrap().value.to_string();

    // find the attribute with key = tokens_out
    let tokens_out_attr = event
        .unwrap()
        .attributes
        .iter()
        .find(|attr| attr.key == "tokens_out");
    if tokens_out_attr.is_none() {
        return try_store_and_finish_dca_swap(deps, env, None);
    }
    let token_out = tokens_out_attr.unwrap().value.to_string();

    try_store_and_finish_dca_swap(
        deps,
        env.clone(),
        Some(SwapEvent {
            executed: true,
            token_in: token_string_to_coin(&token_in),
            effective_token_out: token_string_to_coin(&token_out),
            timestamp_nanos: env.block.time.nanos(),
        }),
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUpcomingSwap {} => to_binary(&query_upcoming_swap(deps, env)?),
        QueryMsg::GetAllUpcomingSwaps {} => to_binary(&query_all_upcoming_swaps(deps, env)?),
        QueryMsg::GetSourceFunds {} => to_binary(&query_bonded_funds(deps, env)?),
        QueryMsg::GetAllFunds {} => to_binary(&query_funds(deps, env)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::State {} => to_binary(&query_state(deps)?),
    }
}
