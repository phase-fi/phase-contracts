use cosmwasm_std::{Coin, MessageInfo, Uint128};
use cw_utils::Expiration;
use phase_finance::{error::ContractError, types::DcaConfig};

use std::str::FromStr;

pub fn verify_sender(config: &DcaConfig, info: &MessageInfo) -> Result<(), ContractError> {
    if info.sender != config.owner && info.sender != config.recipient_address {
        return Err(ContractError::Unauthorized {});
    }
    Ok(())
}

pub fn get_expiration_time(exp: Expiration) -> u64 {
    match exp {
        Expiration::AtTime(time) => time.seconds(),
        _ => u64::MAX,
    }
}

pub fn token_string_to_coin(token_string: &str) -> Option<Coin> {
    // lets scan token string until we find a character that isnt a number
    let number_part = token_string
        .chars()
        .take_while(|c| c.is_numeric())
        .collect::<String>();

    // now lets grab the letters
    let denom_part = token_string
        .chars()
        .skip(number_part.len())
        .collect::<String>();

    Option::Some(Coin {
        amount: Uint128::from_str(&number_part).unwrap(),
        denom: denom_part,
    })
}
