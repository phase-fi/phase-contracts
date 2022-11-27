use cosmwasm_std::{coins, from_binary, Addr, BankMsg, CosmosMsg, Decimal, Uint128, WasmMsg, AllBalanceResponse, Coin};
use cosmwasm_std::testing::{
    mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
};

use phase_finance::types::{CoinWeight, StrategyType};

use crate::contract::{instantiate, execute, query};
use crate::state::BONDED_BALANCES;
use phase_finance::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};


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

#[test]
fn query_handler_bonded_funds() {
    let mut deps = mock_dependencies();

    let res: AllBalanceResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetBondedFunds).unwrap()).unwrap();

    assert_eq!(res, AllBalanceResponse { amount: vec![] });

    BONDED_BALANCES.save(deps.as_mut().storage, "osmos".to_string(), &Uint128::MAX).unwrap();

    let res: AllBalanceResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetBondedFunds).unwrap()).unwrap();

    assert_eq!(res, AllBalanceResponse { amount: vec![Coin { amount: Uint128::MAX, denom: "osmos".to_string() }] });
}