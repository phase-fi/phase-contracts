use cosmwasm_std::{Env, MessageInfo, Coin, Uint128};
use phase_finance::{types::{DcaConfig, State}, error::ContractError};
use regex::Regex;
use std::str::FromStr;

pub fn verify_sender(config: &DcaConfig, info: &MessageInfo) -> Result<(), ContractError> {
    if (info.sender != config.owner && info.sender != config.destination_wallet) {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn can_execute(env: &Env, config: &DcaConfig, state: &State) -> bool {
    state
        .pending_swap_time_nanos
        .unwrap_or(u64::MAX)
        .le(&env.block.time.nanos())
        && !state.paused
        && state.num_trades_executed < config.num_trades
}

/// Gets the next swap time
///
/// If we have already executed the number of trades specified by the strategy, we will return Option::None
///
/// If we have a pending swap time, we will return that time if it is greater than the current time, otherwise we will return the current time + swap_interval_nanos
pub fn get_next_swap_time(
    current_time_nanos: u64,
    config: &DcaConfig,
    state: &State,
) -> Option<u64> {
    if state.num_trades_executed >= config.num_trades {
        return Option::None;
    }
    match state.pending_swap_time_nanos {
        Some(pending_swap_time_nanos) => {
            if current_time_nanos > pending_swap_time_nanos {
                // if current time is greater than the pending swap time, then we can swap during the next dca time (now + swap_interval_nanos)
                Option::Some(
                    current_time_nanos
                        .checked_add(config.swap_interval_nanos)
                        .unwrap_or(current_time_nanos), // TODO: in case of overflow, just return now. Is this safe?
                )
            } else {
                Option::Some(pending_swap_time_nanos)
            }
        }
        None => {
            return Option::Some(
                current_time_nanos
                    .checked_add(config.swap_interval_nanos)
                    .unwrap_or(current_time_nanos), // TODO: in case of overflow, just return now. Is this safe?
            );
        }
    }
}


pub fn token_string_to_coin(token_string: &str) -> Coin {

// lets scan token string until we find a character that isnt a number
    let number_part = token_string.chars().take_while(|c| c.is_numeric()).collect::<String>();
    
    // now lets grab the letters
    let denom_part = token_string.chars().skip(number_part.len()).collect::<String>();

    Coin {
        amount: Uint128::from_str(&number_part).unwrap(),
        denom: denom_part,
    }
}