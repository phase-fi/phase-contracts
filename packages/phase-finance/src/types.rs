use cosmwasm_schema::{cw_serde, schemars::Map};
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_croncat_core::types::SlotType;

#[cw_serde]
pub struct DcaConfig {
    pub strategy_creator: Addr,
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub cron: String,
    pub source: Coin,
    // can DCA into multiple coins
    pub destinations: Vec<CoinWeight>,

    // platform fee (configurable by caller)
    pub platform_fee: Uint128,
    // platform fee recipient (configurable by caller)
    pub platform_wallet: Addr,

    // croncat config
    pub use_croncat: bool,
    pub croncat_task_hash: Option<String>,
}

#[cw_serde]
pub enum StrategyType {
    Linear,
    // In theory we can add other DCA Curves here @lrosa
    // Exponential
    // https://seekingalpha.com/article/4151950-hell-highwater-method-vs-dollar-cost-averaging-introduction
    // https://medium.com/fortune-for-future/a-smarter-way-to-dollar-cost-average-the-2-75-50-rule-578895ca49d3
}

#[cw_serde]
pub struct CoinWeight {
    pub denom: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct UpcomingSwapResponse {
    pub next: Uint128,
    pub slot_type: SlotType,
}


