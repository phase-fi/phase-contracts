pub use crate::contract::{
    execute as executeDCA, instantiate as instantiateDCA, query as queryDCA, reply as replyDCA,
};
use cosmwasm_std::Empty;
pub use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};

pub use swaprouter::contract::{
    execute as executeRouter, instantiate as instantiateRouter, query as queryRouter,
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
