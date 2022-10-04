use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Serialize, Deserialize, Clone, Debug, EnumString, Hash, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StrategyType {
    Linear,
    // In theory we can add other DCA Curves here @lrosa
    // Exponential
    // https://seekingalpha.com/article/4151950-hell-highwater-method-vs-dollar-cost-averaging-introduction
    // https://medium.com/fortune-for-future/a-smarter-way-to-dollar-cost-average-the-2-75-50-rule-578895ca49d3
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CoinWeight {
    pub denom: String,
    pub weight: Uint128,
}