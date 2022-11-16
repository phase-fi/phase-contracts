use cosmwasm_std::{
    coin, to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response,
    SubMsg, WasmMsg,
};
use cw_asset::Asset;
use cw_croncat_core::msg::ExecuteMsg::RemoveTask;
use phase_finance::constants::{APOLLO_ROUTER_ADDRESS, CRONCAT_CONTRACT_ADDR, DCA_SWAP_ID};

use crate::{
    state::{CONFIG, TASK_HASH},
    ContractError,
};

pub fn try_cancel_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.strategy_creator {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.source.denom.clone())?;

    if balance.amount.is_zero() {
        return Err(ContractError::NoBalance {});
    }

    let task_msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: CRONCAT_CONTRACT_ADDR.to_string(),
        funds: vec![],
        msg: to_binary(&RemoveTask {
            task_hash: TASK_HASH.load(deps.storage)?,
        })?,
    });

    let msg = CosmosMsg::Bank(BankMsg::Send {
        amount: vec![coin(balance.amount.u128(), balance.denom)],
        to_address: config.strategy_creator.to_string(),
    });

    Ok(Response::new()
        .add_messages(vec![task_msg, msg])
        .add_attribute("action", "cancel_dca_task"))
}

// pub fn try_withdraw(deps: DepsMut, env: Env, info: MessageInfo)-> Result<Response, ContractError> {}

pub fn try_perform_dca(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // let balance = deps
    //     .querier
    //     .query_balance(env.contract.address, config.source.denom.clone())?;

    let in_funds = vec![Coin {
        denom: config.source.denom.clone(),
        amount: config.source.amount / config.num_trades,
    }];

    let to_assets = config
        .destinations
        .iter()
        .map(|d| Asset {
            amount: d.weight,
            info: cw_asset::AssetInfoBase::Native(d.denom.clone()),
        })
        .collect();

    // todo: perform swap via router
    let msg = WasmMsg::Execute {
        contract_addr: APOLLO_ROUTER_ADDRESS.to_string(),
        msg: to_binary(&apollo_router::msg::ExecuteMsg::Swap {
            to: apollo_router::msg::SwapToAssetsInput::Multi(to_assets),
            max_spread: Option::Some(Decimal::from_ratio(5u128, 1000u128)), // todo: max spread
            recipient: Option::Some(config.strategy_creator.to_string()),
            hook_msg: Option::None,
        })?,
        funds: in_funds,
    };

    // add the message to swap & send funds to user
    Ok(Response::new()
        .add_submessage(SubMsg::reply_on_error(msg, DCA_SWAP_ID)) // todo:: handle reply
        .add_attribute("method", "try_perform_dca"))
}

pub fn try_claim_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.strategy_creator {
        return Err(ContractError::Unauthorized {});
    }

    let balance = deps
        .querier
        .query_balance(env.contract.address, config.source.denom.clone())?;

    let msg = CosmosMsg::Bank(BankMsg::Send {
        amount: vec![coin(balance.amount.u128(), balance.denom)],
        to_address: config.strategy_creator.to_string(),
    });

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "claim_funds"))
}
