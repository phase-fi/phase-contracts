use cosmwasm_std::{
    to_binary, DepsMut, Env, MessageInfo, Response, StdError, StdResult, SubMsgResponse, WasmMsg, Deps,
};
use cosmwasm_std::{Coin, Uint128};
use cw_croncat_core::msg::TaskResponse;
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

pub fn construct_croncat_task_init(
    info: &MessageInfo,
    env: &Env,
    config: &DcaConfig,
) -> Result<WasmMsg, StdError> {
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

pub fn extract_croncat_task_hash(reply_msg: SubMsgResponse) -> StdResult<String> {
    // find the event of type wasm
    let event = reply_msg
        .events
        .into_iter()
        .find(|e| e.ty == "wasm")
        .ok_or(StdError::GenericErr {
            msg: "CRITICAL: no wasm event found in croncat create task response".to_string(),
        });

    match event {
        Ok(event) => {
            // find the attribute with key === task_hash
            let task_hash = event
                .attributes
                .into_iter()
                .find(|a| a.key == "task_hash")
                .ok_or(StdError::GenericErr {
                    msg: "CRITICAL: no task_hash attribute found in croncat create task response"
                        .to_string(),
                })?;

            Ok(task_hash.value)
        }
        Err(e) => Err(e),
    }
}

pub fn get_croncat_task(deps: Deps, croncat_task_hash: String) -> StdResult<TaskResponse>{
    let task_query = cw_croncat_core::msg::QueryMsg::GetTask {
        task_hash: croncat_task_hash,
    };
    let task: Option<TaskResponse> = deps.querier.query(
        &cosmwasm_std::WasmQuery::Smart {
            contract_addr: CRONCAT_CONTRACT_ADDR.to_string(),
            msg: to_binary(&task_query)?,
        }
        .into(),
    )?;

    Ok(task.unwrap())
}