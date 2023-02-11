use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};
use phase_finance::types::{DcaConfig, State};

pub const CONFIG: Item<DcaConfig> = Item::new("config");
pub const STATE: Item<State> = Item::new("state");
pub const BONDED_BALANCES: Map<String, Uint128> = Map::new("bonded_balances");
