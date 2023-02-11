use cosmwasm_std::{coin, Coin};
use cw_utils::Expiration;

pub fn get_expiration_time(exp: Expiration) -> u64 {
    match exp {
        Expiration::AtTime(time) => time.seconds(),
        _ => u64::MAX,
    }
}

pub fn token_string_to_coin(token_string: &str) -> Option<Coin> {
    if token_string.len() == 0 {
        return None;
    }

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

    let amount = number_part.trim().parse::<u128>();

    if amount.is_err() {
        return None;
    }

    return Some(coin(amount.unwrap(), denom_part.trim()));
}
