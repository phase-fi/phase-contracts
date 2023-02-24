use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal, Uint128};
use cw_utils::Duration;

use crate::types::{CoinWeight, DcaConfig, State, StrategyType, UpcomingSwapResponse};

#[cw_serde]
pub struct InstantiateMsg {
    pub recipient_address: String,
    pub executor_address: Option<String>,
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub swap_interval: Duration,
    /// can DCA into multiple coins
    pub destinations: Vec<CoinWeight>,
    // slippage is the same for all swaps, can be changed later
    pub max_slippage: Decimal,

    pub router_contract: String,
    pub source_denom: String,

    // platform fee configurable by sender
    // platform fee paid in source_denom
    pub platform_fee: Uint128,
    pub platform_fee_recipient: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// perform swaps required for the dca
    PerformDca {},
    PauseDca {},
    ResumeDca {},
    /// cancel the dca
    CancelDca {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// get the next swap that will be performed
    #[returns(UpcomingSwapResponse)]
    GetUpcomingSwap {},
    /// get all upcoming swaps
    #[returns(Vec<UpcomingSwapResponse>)]
    GetAllUpcomingSwaps {},
    /// get the amount of funds that are bonded
    #[returns(Coin)]
    GetSourceFunds,
    /// get the amount of funds that are claimable
    #[returns(Vec<Coin>)]
    GetAllFunds {},
    /// get the strategy config
    #[returns(DcaConfig)]
    Config {},
    /// get the strategy state
    #[returns(State)]
    State {},
}
