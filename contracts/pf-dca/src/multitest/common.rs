pub use anyhow::Result;
pub use derivative::Derivative;

pub use crate::contract::{
    execute as executeDCA, instantiate as instantiateDCA, query as queryDCA, reply as replyDCA,
};
pub use cosmwasm_std::{coin, BlockInfo, Coin, Decimal, Empty, StdResult, Uint128};
pub use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

pub use cw_utils::Duration;

pub use swaprouter::{
    contract::{execute as executeRouter, instantiate as instantiateRouter, query as queryRouter},
    msg::{
        ExecuteMsg as RouterExecuteMsg, InstantiateMsg as RouterInstantiateMsg,
        QueryMsg as RouterQueryMsg,
    },
};

pub use phase_finance::{
    error::ContractError as DCAContractError,
    msg::{
        ExecuteMsg as DCAExecuteMsg, InstantiateMsg as DCAInstantiateMsg, QueryMsg as DCAQueryMsg,
    },
    types::{CoinWeight, DcaConfig, State as DCAState, StrategyType, UpcomingSwapResponse},
};

pub const USER: &str = "user";
pub const DEPLOYER: &str = "deployer";
pub const EXECUTOR: &str = "executor";
pub const DENOM: &str = "uosmo";

pub fn contract_dca() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(executeDCA, instantiateDCA, queryDCA).with_reply(replyDCA);
    Box::new(contract)
}

pub fn contract_router() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(executeRouter, instantiateRouter, queryRouter);
    Box::new(contract)
}
