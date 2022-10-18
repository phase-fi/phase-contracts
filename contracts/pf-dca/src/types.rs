use cosmwasm_schema::{cw_serde};
use cosmwasm_std::Uint128;
use strum_macros::EnumString;

#[cw_serde]
#[derive(EnumString)]
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