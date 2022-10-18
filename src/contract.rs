use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use cw_croncat_core::types::Action;

use crate::constants::CRONCAT_CONTRACT_ADDR;
use crate::error::ContractError;
use crate::execute::{try_cancel_dca, try_perform_dca};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use crate::utils::estimate_croncat_funding;

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
    let config = Config {
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
                boundary: Option::None, // todo set boundary for when job expires i guess (can also customize start time)
                stop_on_fail: false,
                actions: vec![Action {
                    msg: WasmMsg::Execute {
                        contract_addr: env.contract.address.to_string(),
                        msg: to_binary(&ExecuteMsg::PerformDca {})?,
                        funds: vec![],
                    }
                    .into(),
                    // Is there any concern with not passing in a gas limit?
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
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetUpcomingSwap {} => todo!(),
        QueryMsg::GetAllUpcomingSwaps {} => todo!(),
        QueryMsg::GetBondedFunds {} => todo!(),
        QueryMsg::GetClaimableFunds {} => todo!(),
        QueryMsg::GetStrategyConfig {} => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{CoinWeight, StrategyType};

    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, from_binary, Addr, BankMsg, CosmosMsg, Decimal, Uint128};
    

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_type: StrategyType::Linear,
            destinations: vec![CoinWeight {
                denom: "uion".to_string(),
                weight: Uint128::from(100u128),
            }],
            amount_per_trade: Uint128::from(10u128),
            num_trades: Uint128::from(10u128),
            cron: "* * 1 * *".to_string(),
            platform_wallet: Addr::unchecked("osmo123".to_string()),
            platform_fee: Uint128::zero(),
        };

        let info = mock_info("creator", &coins(100, "uion"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proper_execution() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            strategy_type: StrategyType::Linear,
            destinations: vec![CoinWeight {
                denom: "uion".to_string(),
                weight: Uint128::from(100u128),
            }],
            amount_per_trade: Uint128::from(10u128),
            num_trades: Uint128::from(10u128),
            cron: "* * 1 * *".to_string(),
            platform_wallet: Addr::unchecked("osmo123".to_string()),
            platform_fee: Uint128::zero(),
        };

        let info = mock_info("osmo123", &coins(100, "uion"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(100, "uion")),
            ExecuteMsg::PerformDca {},
        )
        .unwrap();
        assert_eq!(1, res.messages.len());

        // cast the first response message to a WasmMsg
        let wasm_msg = match res.messages[0].clone().msg {
            CosmosMsg::Wasm(wasm_msg) => match wasm_msg {
                WasmMsg::Execute { msg, .. } => from_binary(&msg).unwrap(),
                _ => panic!("unexpected message"),
            },
            _ => panic!("Unexpected message type"),
        };

        match wasm_msg {
            apollo_router::msg::ExecuteMsg::Swap {
                to: _,
                max_spread,
                recipient,
                hook_msg,
            } => {
                // assert_eq!(to, "uion".to_string());
                assert_eq!(
                    max_spread,
                    Option::Some(Decimal::from_ratio(5u128, 1000u128))
                );
                assert_eq!(recipient, Option::Some("osmo123".to_string()));
                assert_eq!(hook_msg, None);
            }
            _ => panic!("unexpected message"),
        };
    }

    #[test]
    fn proper_cancel() {
        let mut deps = mock_dependencies_with_balance(&coins(100, "uion"));
        let env = mock_env();

        let msg = InstantiateMsg {
            strategy_type: StrategyType::Linear,
            destinations: vec![CoinWeight {
                denom: "uion".to_string(),
                weight: Uint128::from(100u128),
            }],
            amount_per_trade: Uint128::from(10u128),
            num_trades: Uint128::from(10u128),
            cron: "* * 1 * *".to_string(),
            platform_wallet: Addr::unchecked("osmoabc".to_string()),
            platform_fee: Uint128::zero(),
        };

        let info = mock_info("osmo123", &coins(100, "uion"));

        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = execute(deps.as_mut(), env.clone(), info, ExecuteMsg::CancelDca {}).unwrap();
        assert_eq!(1, res.messages.len());

        // cast the first response message to a WasmMsg
        match res.messages[0].clone().msg {
            CosmosMsg::Bank(bank_msg) => match bank_msg {
                BankMsg::Send { to_address, amount } => {
                    assert_eq!(to_address, "osmo123".to_string());
                    assert_eq!(amount, coins(100, "uion"));
                }
                _ => panic!("unexpected message"),
            },
            _ => panic!("Unexpected message type"),
        };
    }

    #[test]
    fn proper_cancel_after_dca() {}

    #[test]
    fn dont_cancel_if_unauthorized() {
        let mut deps = mock_dependencies_with_balance(&coins(100, "uion"));

        let msg = InstantiateMsg {
            strategy_type: StrategyType::Linear,
            destinations: vec![CoinWeight {
                denom: "uion".to_string(),
                weight: Uint128::from(100u128),
            }],
            amount_per_trade: Uint128::from(10u128),
            num_trades: Uint128::from(10u128),
            cron: "* * 1 * *".to_string(),
            platform_wallet: Addr::unchecked("osmo123".to_string()),
            platform_fee: Uint128::zero(),
        };

        let info = mock_info("osmo123", &coins(100, "uion"));

        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(100, "uion")),
            ExecuteMsg::CancelDca {},
        )
        .unwrap_err();

        assert_eq!(res.to_string(), "Unauthorized");
    }
}
