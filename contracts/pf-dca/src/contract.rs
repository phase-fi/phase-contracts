use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, AllBalanceResponse, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order, Response,
    StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_croncat_core::types::Action;

use crate::error::ContractError;
use crate::execute::{try_cancel_dca, try_perform_dca};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BONDED_BALANCES, CONFIG};
use phase_finance::types::DcaConfig;
use phase_finance::constants::CRONCAT_CONTRACT_ADDR;
use phase_finance::utils::estimate_croncat_funding;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:phase-finance";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // check that amount deposited is correct for dca params
    if (msg.amount_per_trade.mul(msg.num_trades)).ne(&info.funds[0].amount) {
        return Err(ContractError::CustomError {
            val: "amount deposited does not match amount per trade and num trades".to_string(),
        });
    }

    // check that amount deposited is not zero
    if info.funds[0].amount.is_zero() {
        return Err(ContractError::CustomError {
            val: "amount deposited is zero".to_string(),
        });
    }

    // check that amount of source coins is equal to 1
    if info.funds.len() != 1 {
        return Err(ContractError::CustomError {
            val: "amount of source coins is not equal to 1".to_string(),
        });
    }

    // store config for this DCA
    let config = DcaConfig {
        strategy_creator: info.sender.clone(),
        strategy_type: msg.strategy_type,
        source: info.funds[0].clone(),
        destinations: msg.destinations,
        amount_per_trade: msg.amount_per_trade,
        num_trades: msg.num_trades,
        cron: msg.cron.clone(),
        platform_wallet: msg.platform_wallet,
        platform_fee: msg.platform_fee,
    };

    let croncat_funding = estimate_croncat_funding(info.funds, &config);

    // ask croncat to start executing these tasks
    let _croncat_msg = WasmMsg::Execute {
        contract_addr: CRONCAT_CONTRACT_ADDR.to_string(),
        msg: to_binary(&cw_croncat_core::msg::ExecuteMsg::CreateTask {
            task: cw_croncat_core::msg::TaskRequest {
                interval: cw_croncat_core::types::Interval::Cron(msg.cron),
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
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("creator", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::PerformDca {} => try_perform_dca(deps, env, info),
        ExecuteMsg::PauseDca {} => todo!(),
        ExecuteMsg::ResumeDca {} => todo!(),
        ExecuteMsg::CancelDca {} => try_cancel_dca(deps, env, info),
        ExecuteMsg::ClaimFunds {} => todo!(),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUpcomingSwap {} => todo!(),
        QueryMsg::GetAllUpcomingSwaps {} => todo!(),
        QueryMsg::GetBondedFunds {} => to_binary(&query_bonded_funds(deps)?),
        QueryMsg::GetClaimableFunds {} => todo!(),
        QueryMsg::GetStrategyConfig {} => todo!(),
    }
}

// fn query_upcoming_swap(deps: Deps, env: Env) -> StdResult<UpcomingSwapResponse> {
//     let config = CONFIG.load(deps.storage)?;

//     // let our_swaps = cw_croncat_core::msg::QueryMsg::GetTasksByOwner { owner_id: () }

//     // Ok(UpcomingSwapResponse {
//     //     next_swap_time: next_swap_time,
//     // })
// }

fn query_bonded_funds(deps: Deps, env: Env) -> StdResult<Coin> {
    Ok(deps.querier.query_balance(
        env.contract.address,
        CONFIG.load(deps.storage)?.source.denom,
    )?)
}

fn query_claimable_funds(deps: Deps, env: Env) -> StdResult<Vec<Coin>> {
    let destinations: Vec<String> = CONFIG
        .load(deps.storage)?
        .destinations
        .iter()
        .map(|d| d.denom.clone())
        .collect();

    let balances = deps
        .querier
        .query_all_balances(env.contract.address)?
        .into_iter()
        .filter(|coin| destinations.contains(&coin.denom))
        .collect();

    Ok(AllBalanceResponse { amount: amount? })
}
