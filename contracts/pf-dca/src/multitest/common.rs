pub use anyhow::Result;
pub use derivative::Derivative;

pub use crate::contract::{
    execute as executeDCA, instantiate as instantiateDCA, query as queryDCA, reply as replyDCA,
};
pub use cosmwasm_std::{coin, Coin, Empty, StdResult};
pub use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

pub use swaprouter::contract::{
    execute as executeRouter, instantiate as instantiateRouter, query as queryRouter,
};

pub use phase_finance::{
    error::ContractError as DCAContractError,
    msg::{
        ExecuteMsg as DCAExecuteMsg, InstantiateMsg as DCAInstantiateMsg, QueryMsg as DCAQueryMsg,
    },
    types::{DcaConfig, State as DCAState, UpcomingSwapResponse},
};

pub const USER: &str = "user";

pub fn contract_dca() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(executeDCA, instantiateDCA, queryDCA).with_reply(replyDCA);
    Box::new(contract)
}

pub fn contract_router() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(executeRouter, instantiateRouter, queryRouter);
    Box::new(contract)
}
