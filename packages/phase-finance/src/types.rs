use cosmwasm_schema::{cw_serde, schemars::Map};
use cosmwasm_std::{Addr, Coin, Env, Uint128};
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

#[cw_serde]
pub struct SwapEvent {
    pub executed: bool,
    pub token_in: Vec<Coin>,
    pub effective_tokens_out: Option<Vec<Coin>>,
    pub timestamp_nanos: u64, // here we add other necessary info whenever swaps happen.
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
