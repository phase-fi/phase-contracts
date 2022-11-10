use std::vec;

use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, OwnedDeps, StdError,
    Uint128, WasmMsg,
};

use cw_croncat_core::msg::ExecuteMsg::RemoveTask;

use phase_finance::constants::CRONCAT_CONTRACT_ADDR;
use phase_finance::types::{CoinWeight, StrategyType};

use crate::contract::{execute, instantiate, query};
use crate::execute::{try_cancel_dca, try_claim_funds};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::TASK_HASH;
use crate::ContractError;

pub const ADMIN_ADDR: &str = "admin_addr";

fn do_instantiate() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let info = mock_info(ADMIN_ADDR, &coins(100, "uion"));
    let env = mock_env();

    let instantiate_msg = InstantiateMsg {
        strategy_type: StrategyType::Linear,
        destinations: vec![
            CoinWeight {
                denom: "juno".to_string(),
                weight: Uint128::from(100u128),
            },
            CoinWeight {
                denom: "osmo".to_string(),
                weight: Uint128::from(100u128),
            },
        ],
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        cron: "* * 1 * *".to_string(),
        platform_wallet: Addr::unchecked("osmo123".to_string()),
        platform_fee: Uint128::zero(),
    };

    instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    deps
}

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
fn proper_cancel_after_dca() {}

#[test]
fn query_handler_bonded_funds() {
    let mut deps = do_instantiate();
    let env = mock_env();

    deps.querier
        .update_balance(env.contract.address.clone(), coins(100, "uion"));

    let res: Coin =
        from_binary(&query(deps.as_ref(), env, QueryMsg::GetBondedFunds {}).unwrap()).unwrap();

    assert_eq!(res, Coin::new(100, "uion"));
}

#[test]
fn query_handler_claim_funds() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let balance = vec![Coin::new(100, "osmo"), Coin::new(100, "juno")];

    deps.querier
        .update_balance(env.contract.address.clone(), balance.clone());

    let res: Vec<Coin> =
        from_binary(&query(deps.as_ref(), env, QueryMsg::GetClaimableFunds {}).unwrap()).unwrap();

    assert_eq!(res, balance);
}

#[test]
fn execute_handler_try_claim_funds() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let balance = vec![Coin::new(100, "uion")];

    deps.querier
        .update_balance(env.contract.address.clone(), balance.clone());

    // Should fail because is not authorized
    let res =
        try_claim_funds(deps.as_mut(), env.clone(), mock_info("RANDOM_ADDR", &[])).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});

    let res = try_claim_funds(deps.as_mut(), env, mock_info(ADMIN_ADDR, &[])).unwrap();

    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Bank(BankMsg::Send {
            to_address: ADMIN_ADDR.to_string(),
            amount: balance
        })
    );

    assert_eq!(res.attributes, [("action", "claim_funds")]);
}

#[test]
fn execute_handler_try_cancel_dca() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let balance = vec![Coin::new(100, "uion")];

    // Should fail because is not authorized
    let res =
        try_cancel_dca(deps.as_mut(), env.clone(), mock_info("RANDOM_ADDR", &[])).unwrap_err();
    assert_eq!(res, ContractError::Unauthorized {});

    // Should fail in case there is not balance
    let res = try_cancel_dca(deps.as_mut(), env.clone(), mock_info(ADMIN_ADDR, &[])).unwrap_err();
    assert_eq!(res, ContractError::NoBalance {});

    deps.querier
        .update_balance(env.contract.address.clone(), balance.clone());

    // Should fail if there is not defined a task.
    let res = try_cancel_dca(deps.as_mut(), env.clone(), mock_info(ADMIN_ADDR, &[])).unwrap_err();

    assert_eq!(
        res,
        ContractError::Std(StdError::NotFound {
            kind: "alloc::string::String".to_string()
        })
    );

    TASK_HASH
        .save(deps.as_mut().storage, &"HASH".to_string())
        .unwrap();

    let res = try_cancel_dca(deps.as_mut(), env, mock_info(ADMIN_ADDR, &[])).unwrap();

    assert_eq!(
        res.messages[0].msg,
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: CRONCAT_CONTRACT_ADDR.to_string(),
            funds: vec![],
            msg: to_binary(&RemoveTask {
                task_hash: "HASH".to_string(),
            })
            .unwrap()
        })
    );

    assert_eq!(res.attributes, [("action", "cancel_dca_task")]);
}
