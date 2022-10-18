use cosmwasm_std::{Coin, Uint128};

use crate::types::DcaConfig;

pub fn estimate_croncat_funding(_coin: Vec<Coin>, config: &DcaConfig) -> Vec<Coin> {
    vec![Coin {
        amount: config.num_trades * Uint128::from(200u128),
        denom: config.source.denom.clone(),
    }]
}
