use cosmwasm_std::{Coin, Uint128};

use crate::state::Config;

pub fn estimate_croncat_funding(coin: Vec<Coin>, config: &Config) -> Vec<Coin> {
    vec![Coin {
        amount: config.num_trades * Uint128::from(200u128),
        denom: config.source.denom.clone(),
    }]
}
