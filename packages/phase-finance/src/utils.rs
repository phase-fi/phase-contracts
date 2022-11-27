use cosmwasm_std::{to_binary, Env, MessageInfo, WasmMsg, StdError};
use cosmwasm_std::{Coin, Uint128};
use cw_croncat_core::types::Action;

use crate::constants::CRONCAT_CONTRACT_ADDR;
use crate::msg::ExecuteMsg;
use crate::types::DcaConfig;

pub fn estimate_croncat_funding(_coin: Vec<Coin>, config: &DcaConfig) -> Vec<Coin> {
    vec![Coin {
        amount: config.num_trades * Uint128::from(10866u128),
        denom: config.source.denom.clone(),
    }]
}

pub fn construct_croncat_task_init(info: &MessageInfo, env: &Env, config: &DcaConfig) -> Result<WasmMsg, StdError> {
    let croncat_funding = estimate_croncat_funding(info.funds.clone(), config);

    Ok(WasmMsg::Execute {
        contract_addr: CRONCAT_CONTRACT_ADDR.to_string(),
        msg: to_binary(&cw_croncat_core::msg::ExecuteMsg::CreateTask {
            task: cw_croncat_core::msg::TaskRequest {
                interval: cw_croncat_core::types::Interval::Cron(config.cron.clone()),
                boundary: Option::None, // todo: set boundary for when job expires i guess (can also customize start time)
                stop_on_fail: false,
                actions: vec![Action {
                    msg: WasmMsg::Execute {
                        contract_addr: env.contract.address.to_string(),
                        msg: to_binary(&ExecuteMsg::PerformDca {})?,
                        funds: vec![],
                    }
                    .into(),
                    // todo: Is there any concern with not passing in a gas limit?
                    gas_limit: Option::None,
                }],
                rules: Option::None,
                cw20_coins: vec![],
            },
        })?,
        funds: croncat_funding,
    })
}
