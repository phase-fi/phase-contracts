use std::ops::{Add, Mul};

use cosmwasm_std::{Coin, Deps, Env, StdResult};

use cw_utils::Duration;
use phase_finance::types::{DcaConfig, State, UpcomingSwapResponse};

use crate::{
    helpers::get_expiration_time,
    state::{CONFIG, STATE},
};

pub fn query_upcoming_swap(deps: Deps, env: Env) -> StdResult<UpcomingSwapResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(UpcomingSwapResponse {
        next_swap: get_expiration_time(state.next_swap),
        can_execute: state.next_swap.is_expired(&env.block),
    })
}

pub fn query_all_upcoming_swaps(deps: Deps, env: Env) -> StdResult<Vec<UpcomingSwapResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    let pending_swaps = config.num_trades - state.num_trades_executed;

    let upcoming_swaps: Vec<UpcomingSwapResponse> =
        Vec::<UpcomingSwapResponse>::with_capacity(pending_swaps.u128() as usize)
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i > 0 {
                    let index = i + 1;
                    let interval_in_seconds = match config.swap_interval {
                        Duration::Time(time) => time,
                        _ => 0,
                    };
                    return UpcomingSwapResponse {
                        next_swap: get_expiration_time(state.next_swap)
                            .add(interval_in_seconds.mul(index as u64)),
                        can_execute: false,
                    };
                }

                UpcomingSwapResponse {
                    next_swap: get_expiration_time(state.next_swap),
                    can_execute: state.next_swap.is_expired(&env.block),
                }
            })
            .collect();

    Ok(upcoming_swaps)
}

pub fn query_bonded_funds(deps: Deps, env: Env) -> StdResult<Coin> {
    deps.querier.query_balance(
        env.contract.address,
        CONFIG.load(deps.storage)?.source_denom,
    )
}

pub fn query_funds(deps: Deps, env: Env) -> StdResult<Vec<Coin>> {
    let config = CONFIG.load(deps.storage)?;

    let destination_denoms: Vec<String> = CONFIG
        .load(deps.storage)?
        .destinations
        .iter()
        .map(|d| d.denom.clone())
        .collect();

    let balances: Vec<Coin> = deps
        .querier
        .query_all_balances(env.contract.address)?
        .into_iter()
        .filter(|coin| {
            destination_denoms.contains(&coin.denom) || coin.denom == config.source_denom
        })
        .collect();

    Ok(balances)
}

pub fn query_config(deps: Deps) -> StdResult<DcaConfig> {
    CONFIG.load(deps.storage)
}

pub fn query_state(deps: Deps) -> StdResult<State> {
    STATE.load(deps.storage)
}
