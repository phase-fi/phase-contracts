use cosmwasm_std::{Coin, Deps, Env, StdResult};

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
    let _config = CONFIG.load(deps.storage)?;
    let _state = STATE.load(deps.storage)?;

    // calculate (config.num_trades - state.num_trades_executed) upcoming swaps and add config.swap_interval_nanos to each subsequent swap
    let upcoming_swaps: Vec<UpcomingSwapResponse> = Vec::new();
    let _next_swap_time_nanos = env.block.time.nanos();

    Ok(upcoming_swaps)
}

pub fn query_bonded_funds(deps: Deps, env: Env) -> StdResult<Coin> {
    deps.querier
        .query_balance(env.contract.address, CONFIG.load(deps.storage)?.source)
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
        .filter(|coin| destination_denoms.contains(&coin.denom) || coin.denom == config.source)
        .collect();

    Ok(balances)
}

pub fn query_config(deps: Deps) -> StdResult<DcaConfig> {
    CONFIG.load(deps.storage)
}

pub fn query_state(deps: Deps) -> StdResult<State> {
    STATE.load(deps.storage)
}
