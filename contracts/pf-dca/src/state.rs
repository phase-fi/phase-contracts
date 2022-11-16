use cw_storage_plus::Item;
use phase_finance::types::DcaConfig;

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

pub const CONFIG: Item<DcaConfig> = Item::new("config");
pub const TASK_HASH: Item<String> = Item::new("task_hash");
