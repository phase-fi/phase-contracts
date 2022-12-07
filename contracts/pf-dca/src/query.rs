use std::str::FromStr;

use cosmwasm_std::{Coin, Deps, Env, StdResult};
use phase_finance::types::{UpcomingSwapResponse, DcaConfig, State};

use crate::{
    helpers::can_execute,
    state::{CONFIG, STATE},
};

pub fn query_upcoming_swap(deps: Deps, env: Env) -> StdResult<UpcomingSwapResponse> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    Ok(UpcomingSwapResponse {
        pending_swap: state.pending_swap,
        can_execute: can_execute(&env, &config, &state),
    })
}

pub fn query_all_upcoming_swaps(deps: Deps, env: Env) -> StdResult<Vec<UpcomingSwapResponse>> {
    let config = CONFIG.load(deps.storage)?;
    let state = STATE.load(deps.storage)?;

    let cron_schedule = cron_schedule::Schedule::from_str(&config.cron).unwrap();
    let upcoming_swaps = cron_schedule
        .upcoming()
        // todo is this safe?
        .skip(state.num_trades_executed.u128() as usize)
        .take(config.num_trades.u128() as usize)
        .collect::<Vec<u64>>();

    upcoming_swaps
        .iter()
        .map(|swap| {
            Ok(UpcomingSwapResponse {
                pending_swap: Some(*swap),
                can_execute: can_execute(&env, &config, &state),
            })
        })
        .collect()
}

pub fn query_bonded_funds(deps: Deps, env: Env) -> StdResult<Coin> {
    deps.querier.query_balance(
        env.contract.address,
        CONFIG.load(deps.storage)?.source.denom,
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
        .filter(|coin| destination_denoms.contains(&coin.denom) || coin.denom == config.source.denom)
        .collect();

    Ok(balances)
}


pub fn query_config(deps: Deps) -> StdResult<DcaConfig> {
    Ok(CONFIG.load(deps.storage))
}

pub fn query_state(deps: Deps) -> StdResult<State> {
    Ok(STATE.load(deps.storage))
}