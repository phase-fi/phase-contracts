use cosmwasm_schema::{cw_serde, schemars::Map};
use cosmwasm_std::{Addr, Coin, Env, Uint128};

#[cw_serde]
pub struct DcaConfig {
    pub owner: String,
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub cron: String,
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
    pub pending_swap: Option<u64>,
    pub can_execute: bool,
}

#[cw_serde]
pub struct State {
    pub pending_swap: Option<u64>,
    pub paused: bool,
    pub num_trades_executed: Uint128,
}

#[cw_serde]
pub struct SwapEvent {
    // whether or not the swap was executed yet
    pub executed: bool,
    // the source token denom and the amount_per_trade amount
    pub token_in: Coin,
    // will be empty if the swap failed or didnt happen yet
    pub effective_tokens_out: Vec<Coin>,
    // the  timestamp for which this swap is scheduled
    pub timestamp_nanos: u64, // here we add other necessary info whenever swaps happen.
    // the actual timestamp for when this swap was executed, will be 0 if not executed yet
    pub effective_timestamp_nanos: u64,
}

#[cw_serde]
pub struct DcaRecord {
    pub swap_events: Map<u64, SwapEvent>,
}

impl DcaRecord {
    pub fn get_last_swap_event(&self, env: &Env) -> Option<&SwapEvent> {
        let now = env.block.time;

        let mut last_swap_event = Option::None;
        for (swap_event_timestamp, value) in self.swap_events.iter() {
            // update last swap event if it happened in the past
            if swap_event_timestamp < &now.nanos() {
                last_swap_event = Some(value);
            }
        }

        last_swap_event
    }

    pub fn get_next_swap_event(&self, env: &Env) -> Option<&SwapEvent> {
        let now = env.block.time;

        for (swap_event_timestamp, value) in self.swap_events.iter() {
            // return as soon as we find a swap event in the future
            if swap_event_timestamp > &now.nanos() {
                return Some(value);
            }
        }

        return Option::None;
    }
}
