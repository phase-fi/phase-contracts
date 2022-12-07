use cosmwasm_std::{Deps, DepsMut, Env};
use phase_finance::types::{DcaConfig, State};
use std::str::FromStr;


pub fn can_execute(env: &Env, config: &DcaConfig, state: &State) -> bool {
    state
        .pending_swap
        .unwrap_or(u64::MAX)
        .le(&env.block.time.nanos())
        && !state.paused
        && state.num_trades_executed < config.num_trades
}

pub fn get_next_swap_time(config: &DcaConfig, state: &State) -> Option<u64> {
    if state.num_trades_executed >= config.num_trades {
        return Option::None;
    }
    
    let cron_schedule = cron_schedule::Schedule::from_str(&config.cron).unwrap();
    cron_schedule.upcoming().next()
}
