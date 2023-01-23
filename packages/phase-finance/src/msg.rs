use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Decimal, Uint128};
use cw_utils::Duration;

use crate::types::{CoinWeight, DcaConfig, State, StrategyType, UpcomingSwapResponse};

// Execute:
// create/instantiate: Both collab
// pause/resume: Nikita
// stop/cancel: J0nl1
// unbond/claim: J0nl1
// perform_swaps(something that gets called to perform the strategy every day/week/month): Nikita

// Queries:
// get_upcoming_swap: Both collab
// get_all_upcoming_swaps: Both collab
// bonded: J0nl1
// claimable: J0nl1
// strategy_config: Nikita

#[cw_serde]
pub struct InstantiateMsg {
    pub destination_wallet: String,
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub swap_interval: Duration,
    // can DCA into multiple coins
    pub destinations: Vec<CoinWeight>,
    // slippage is the same for all swaps, can be changed later
    pub max_slippage: Decimal,

    pub router_contract: String,
    pub source_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // perform swaps required for the dca
    PerformDca {},
    PauseDca {},
    ResumeDca {},
    // cancel the dca
    CancelDca {},
    // unbond (not required yet)
    // UnbondFunds {},
    // claim deposited funds (this will also claim unbonded funds when yield strategies are added)
    // no need to claim funds on the destinations since those should be sent to the users
    // wallet after every DCA step
    // ClaimFunds {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // get the next swap that will be performed
    #[returns(UpcomingSwapResponse)]
    GetUpcomingSwap {},
    // get all upcoming swaps
    #[returns(Vec<UpcomingSwapResponse>)]
    GetAllUpcomingSwaps {},
    // get the amount of funds that are bonded
    #[returns(Coin)]
    GetSourceFunds,
    // get the amount of funds that are claimable
    #[returns(Vec<Coin>)]
    GetAllFunds {},
    // get the strategy config
    #[returns(DcaConfig)]
    Config {},
    // get the strategy state
    #[returns(State)]
    State {},
}
