#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, AllBalanceResponse, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsg, SubMsgResponse, Uint128,
};
use cw2::set_contract_version;

use cw_croncat_core::traits::Intervals;
use cw_croncat_core::types::BoundaryValidated;
use phase_finance::constants::DCA_SWAP_ID;

use crate::execute::{try_cancel_dca, try_perform_dca};
use crate::state::CONFIG;

use phase_finance::croncat_helpers::{
    construct_croncat_task_init, extract_croncat_task_hash, get_croncat_task,
};
use phase_finance::error::ContractError;
use phase_finance::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use phase_finance::types::{DcaConfig, UpcomingSwapResponse};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:phase-finance";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const CRONCAT_INIT_REPLY_ID: u64 = 1337;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // check that amount deposited is correct for dca params
    // if (msg.amount_per_trade.mul(msg.num_trades)).ne(&info.funds[0].amount) {
    //     return Err(ContractError::CustomError {
    //         val: "amount deposited does not match amount per trade and num trades".to_string(),
    //     });
    // }

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
        croncat_task_hash: Option::None,
        use_croncat: msg.use_croncat,
    };

    // ask croncat to start executing these tasks
    let croncat_msg = construct_croncat_task_init(deps.as_ref(), &info, &env, &config)?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        // .add_message(croncat_msg)
        .add_submessage(SubMsg::reply_always(croncat_msg, CRONCAT_INIT_REPLY_ID))
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
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        CRONCAT_INIT_REPLY_ID => match msg.result {
            cosmwasm_std::SubMsgResult::Ok(reply_msg) => {
                process_croncat_create_task_response(deps, reply_msg)
            }
            cosmwasm_std::SubMsgResult::Err(_) => Err(StdError::GenericErr {
                msg: "croncat job failed with error: ".to_string() + &msg.result.unwrap_err(),
            }),
        },
        DCA_SWAP_ID => match msg.result {
            cosmwasm_std::SubMsgResult::Ok(reply_msg) => {
                // in the function below (will be named process_dca_swap_response), we will need to get the swapEvent timestamp to avoid the edge case where a swap is executed just before the next swap begins, and we receive the swap response after, setting the swapEvent.executed value to true on the next swap event rather than the one we want. But I am tired and I forgot how to pass the swapEvent key correctly.
                // also in the function below, if everything checks out we need to set the swapEvent.executed value to true
                todo!()
            }
            cosmwasm_std::SubMsgResult::Err(_) => Err(StdError::GenericErr {
                msg: "dca swap failed with error: ".to_string() + &msg.result.unwrap_err(),
            }),
        },
        _ => Err(StdError::GenericErr {
            msg: "unknown reply id".to_string(),
        }),
    }
}

pub fn process_croncat_create_task_response(
    deps: DepsMut,
    reply_msg: SubMsgResponse,
) -> StdResult<Response> {
    // load config
    let mut config = CONFIG.load(deps.storage)?;

    let croncat_task_hash = extract_croncat_task_hash(reply_msg)?;

    config.croncat_task_hash = Option::Some(croncat_task_hash.clone());
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("croncat_task_hash", croncat_task_hash))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUpcomingSwap {} => to_binary(&query_upcoming_swap(deps, env)?),
        QueryMsg::GetAllUpcomingSwaps {} => todo!(),
        QueryMsg::GetBondedFunds {} => to_binary(&query_bonded_funds(deps, env)?),
        QueryMsg::GetClaimableFunds {} => to_binary(&query_claimable_funds(deps, env)?),
        QueryMsg::GetStrategyConfig {} => todo!(),
    }
}

fn query_upcoming_swap(deps: Deps, env: Env) -> StdResult<UpcomingSwapResponse> {
    let config = CONFIG.load(deps.storage)?;
    let croncat_task_hash = config.croncat_task_hash;

    match croncat_task_hash {
        Option::Some(task_hash) => {
            // let task: Option<TaskResponse> = from_binary(&response)?;
            let task = get_croncat_task(deps, task_hash)?;

            let interval = task.interval;
            let boundary = task.boundary;
            let next = interval.next(
                &env,
                BoundaryValidated::validate_boundary(boundary, &interval).unwrap(),
            );

            Ok(UpcomingSwapResponse {
                next: Uint128::from(next.0),
                slot_type: next.1,
            })
        }
        Option::None => Err(StdError::GenericErr {
            // another error we should never get
            msg: "No croncat task defined in config".to_string(),
        }),
    }
}

fn query_bonded_funds(deps: Deps, env: Env) -> StdResult<Coin> {
    Ok(deps.querier.query_balance(
        env.contract.address,
        CONFIG.load(deps.storage)?.source.denom,
    )?)
}

fn query_claimable_funds(deps: Deps, env: Env) -> StdResult<AllBalanceResponse> {
    let destinations: Vec<String> = CONFIG
        .load(deps.storage)?
        .destinations
        .iter()
        .map(|d| d.denom.clone())
        .collect();

    let balances: Vec<Coin> = deps
        .querier
        .query_all_balances(env.contract.address)?
        .into_iter()
        .filter(|coin| destinations.contains(&coin.denom))
        .collect();

    Ok(AllBalanceResponse { amount: balances })
}
