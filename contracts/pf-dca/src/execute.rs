use cosmwasm_std::{
    to_binary, BankMsg, Coin, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg, WasmMsg,
};
use cw_asset::Asset;
use phase_finance::constants::{APOLLO_ROUTER_ADDRESS, DCA_SWAP_ID};
use phase_finance::error::ContractError;

use crate::state::CONFIG;

pub fn try_cancel_dca(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.strategy_creator {
        return Err(ContractError::Unauthorized {});
    }

    let balances = deps.querier.query_all_balances(env.contract.address)?;

    if balances.len() == 0 {
        return Err(ContractError::NoBalance {});
    }

    let msg = BankMsg::Send {
        to_address: config.strategy_creator.to_string(),
        amount: balances,
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("method", "try_cancel_dca"))
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
