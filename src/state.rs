use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::Item;

use crate::types::{CoinWeight, StrategyType};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub strategy_creator: Addr,
    pub strategy_type: StrategyType,
    pub amount_per_trade: Uint128,
    pub num_trades: Uint128,
    pub cron: String, 
    pub source: Coin,
    // can DCA into multiple coins
    pub destinations: Vec<CoinWeight>,
    pub platform_wallet: Addr,
    pub platform_fee: Uint128,
}

// struct SubmitOrder {
//     address inToken;
//     address outToken;
//     uint256 amountPerTrade;
//     uint256 numTrades;
//     uint256 minSlippage;
//     uint256 maxSlippage;
//     uint256 delay;
//     address platformWallet;
//     uint256 platformFeeBps;
// }

// struct ExecOrder {
//     address user;
//     address inToken;
//     address outToken;
//     uint256 amountPerTrade;
//     uint256 nTradesLeft;
//     uint256 minSlippage;
//     uint256 maxSlippage;
//     uint256 delay;
//     uint256 lastExecutionTime;
//     address platformWallet;
//     uint256 platformFeeBps;
// }

pub const CONFIG: Item<Config> = Item::new("config");
