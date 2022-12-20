use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{Coin, Uint128};

#[cw_serde]
pub struct DcaConfig {
    pub owner: String,
    pub destination_wallet: String,
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub swap_interval_nanos: u64,
    pub source: Coin,
    // can DCA into multiple coins
    pub destinations: Vec<CoinWeight>,

    // platform fee (configurable by initializer)
    pub platform_fee: Uint128,
    // platform fee recipient (configurable by initializer)
    pub platform_wallet: Option<String>,

    pub router_contract: String,
    // croncat to be added once their contracts are on mainnet
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
    pub pending_swap_time_nanos: Option<u64>,
    pub can_execute: bool,
}

#[cw_serde]
pub struct State {
    // epoch time in nanons of the earliest allowed time of the swap that is yet to be executed
    pub pending_swap_time_nanos: Option<u64>,
    // if the strategy is paused
    pub paused: bool,
    // number of trades already executed (should never be more than config.num_trades)
    pub num_trades_executed: Uint128,

    // for collecting all swaps in the reply handler and incrementing DCA pending swap
    pub swap_status: Vec<SwapEvent>,
}

#[cw_serde]
pub struct SwapEvent {
    // whether or not the swap was executed yet
    pub executed: bool,
    // the source token denom and the amount_per_trade amount
    pub token_in: Option<Coin>,
    // will be empty if the swap failed or didnt happen yet
    pub effective_token_out: Option<Coin>,
    // the  timestamp for which this swap is scheduled
    pub timestamp_nanos: u64, // here we add other necessary info whenever swaps happen.
}
