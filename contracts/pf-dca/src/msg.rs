use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128};

use phase_finance::types::{CoinWeight, StrategyType};

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
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub cron: String,
    // can DCA into multiple coins
    pub destinations: Vec<CoinWeight>,
    pub platform_wallet: Addr,
    pub platform_fee: Uint128,
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
    ClaimFunds {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // get the next swap that will be performed
    #[returns(())]
    GetUpcomingSwap {},
    // get all upcoming swaps
    #[returns(())]
    GetAllUpcomingSwaps {},
    // get the amount of funds that are bonded
    #[returns(Coin)]
    GetBondedFunds {},
    // get the amount of funds that are claimable
    #[returns(Vec<Coin>)]
    GetClaimableFunds {},
    // get the strategy config
    #[returns(())]
    GetStrategyConfig {},
}
