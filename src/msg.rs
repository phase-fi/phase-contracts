use cosmwasm_std::{Timestamp, Coin, Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{CoinWeight, StrategyType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub cron: String, 
    // can DCA into multiple coins
    pub destinations: Vec<CoinWeight>,
    pub platform_wallet: Addr,
    pub platform_fee: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CancelDca,
    PerformDca,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetCountResponse {
    pub count: i32,
}
