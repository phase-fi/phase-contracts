use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{
    coins, from_binary, BlockInfo, Coin, Decimal, Env, OwnedDeps, StdResult, Timestamp, Uint128,
};

use cw_denom::DenomError;
use cw_utils::Duration;
use phase_finance::types::{CoinWeight, StrategyType};

use crate::contract::{execute, instantiate, query};
use crate::helpers::token_string_to_coin;
use crate::state::STATE;

use phase_finance::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub const ADMIN_ADDR: &str = "admin_addr";
pub const EXECUTOR_ADDR: &str = "executor";

fn do_instantiate() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let info = mock_info(ADMIN_ADDR, &coins(100, "uosmo"));
    let env = mock_env();

    let instantiate_msg = InstantiateMsg {
        recipient_address: "osmo123".to_string(),
        executor_address: Some(EXECUTOR_ADDR.to_string()),
        strategy_type: StrategyType::Linear,
        destinations: vec![
            CoinWeight {
                denom: "uion".to_string(),
                weight: Uint128::from(100u128),
            },
            CoinWeight {
                denom: "ujuno".to_string(),
                weight: Uint128::from(100u128),
            },
        ],
        max_slippage: Decimal::from_ratio(1u128, 100u128),
        twap_window_seconds: 1,
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Time(1),
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
        platform_fee: Uint128::zero(),
        platform_fee_recipient: "osmo123".to_string(),
    };

    instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    deps
}

fn fast_forward_time(mut env: Env, time: u64) -> Env {
    env.block = BlockInfo {
        time: env.block.time.plus_seconds(time),
        height: env.block.height + 1,
        chain_id: env.block.chain_id,
    };

    return env;
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        recipient_address: "osmo123".to_string(),
        executor_address: Some(EXECUTOR_ADDR.to_string()),
        strategy_type: StrategyType::Linear,
        destinations: vec![CoinWeight {
            denom: "uion".to_string(),
            weight: Uint128::from(100u128),
        }],
        max_slippage: Decimal::from_ratio(1u128, 100u128),
        twap_window_seconds: 1,
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Height(100_000_000_000),
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
        platform_fee: Uint128::zero(),
        platform_fee_recipient: "osmo123".to_string(),
    };

    let info = mock_info("creator", &coins(100, "uosmo"));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn should_fail_because_wrong_denom() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        recipient_address: "osmo123".to_string(),
        executor_address: Some(EXECUTOR_ADDR.to_string()),
        strategy_type: StrategyType::Linear,
        platform_fee: Uint128::zero(),
        platform_fee_recipient: "osmo123".to_string(),
        destinations: vec![CoinWeight {
            denom: "43Denom".to_string(),
            weight: Uint128::from(100u128),
        }],
        max_slippage: Decimal::from_ratio(1u128, 100u128),
        twap_window_seconds: 1,
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Height(100_000_000_000),
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
    };

    let info = mock_info("creator", &coins(100, "uosmo"));

    let err = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(err.to_string(), DenomError::NonAlphabeticAscii.to_string());
}

#[test]
fn dont_init_with_too_many_destinations() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        recipient_address: "osmo123".to_string(),
        executor_address: Some(EXECUTOR_ADDR.to_string()),
        strategy_type: StrategyType::Linear,
        destinations: vec![
            CoinWeight {
                denom: "uion".to_string(),
                weight: Uint128::from(100u128),
            };
            26
        ],
        max_slippage: Decimal::from_ratio(1u128, 100u128),
        twap_window_seconds: 1,
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Height(100_000_000_000),
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
        platform_fee: Uint128::zero(),
        platform_fee_recipient: "osmo123".to_string(),
    };

    let info = mock_info("creator", &coins(100, "uosmo"));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match res {
        phase_finance::error::ContractError::CustomError { val } => {
            assert_eq!(val, "Number of destination tokens must be between 1 and 25")
        }
        _ => panic!("Unexpected error: {:?}", res),
    }
}

#[test]
fn dont_init_with_bad_funds_with_platform_fee() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        recipient_address: "osmo123".to_string(),
        executor_address: Some(EXECUTOR_ADDR.to_string()),
        strategy_type: StrategyType::Linear,
        destinations: vec![
            CoinWeight {
                denom: "uion".to_string(),
                weight: Uint128::from(100u128),
            };
            21
        ],
        max_slippage: Decimal::from_ratio(1u128, 100u128),
        twap_window_seconds: 1,
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Height(100_000_000_000),
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
        platform_fee: Uint128::one(),
        platform_fee_recipient: "osmo123".to_string(),
    };

    let info = mock_info("creator", &coins(100, "uosmo"));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    match res {
        phase_finance::error::ContractError::CustomError { val } => {
            assert_eq!(
                val,
                "Amount deposited does not match exactly expected: <101> != actual: <100>"
            )
        }
        _ => panic!("Unexpected error: {:?}", res),
    }
}

#[test]
fn proper_initialization_with_platform_fee() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        recipient_address: "osmo123".to_string(),
        executor_address: Option::Some(EXECUTOR_ADDR.to_string()),
        strategy_type: StrategyType::Linear,
        destinations: vec![CoinWeight {
            denom: "uion".to_string(),
            weight: Uint128::from(100u128),
        }],
        max_slippage: Decimal::from_ratio(1u128, 100u128),
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Height(100_000_000_000),
        twap_window_seconds: 1,
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
        platform_fee: Uint128::one(),
        platform_fee_recipient: "osmo1234".to_string(),
    };

    let info = mock_info("creator", &coins(101, "uosmo"));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(1, res.messages.len());
    match &res.messages.get(0).unwrap().msg {
        cosmwasm_std::CosmosMsg::Bank(bank_msg) => match bank_msg {
            cosmwasm_std::BankMsg::Send { to_address, amount } => {
                assert_eq!("osmo1234", to_address);
                assert_eq!(1, amount.len());
            }
            _ => assert!(false, "expected BankMsg::Send in response"),
        },
        _ => assert!(false, "expected BankMsg in response"),
    }
}

#[test]
fn proper_execution() {
    let mut deps = do_instantiate();

    let mut env = mock_env();
    env.block = BlockInfo {
        height: 10000000,
        chain_id: "osmos".to_string(),
        time: Timestamp::from_seconds(9_000_000_000),
    };

    let res = execute(
        deps.as_mut(),
        env,
        mock_info(EXECUTOR_ADDR, &coins(100, "uion")),
        ExecuteMsg::PerformDca {},
    )
    .unwrap();

    assert_eq!(2, res.messages.len());
}

#[test]
fn proper_cancel() {
    let mut deps = do_instantiate();

    let env = mock_env();

    let balance = vec![Coin::new(100, "uion"), Coin::new(100, "ujuno")];

    deps.querier
        .update_balance(env.contract.address.clone(), balance.clone());

    let res = execute(
        deps.as_mut(),
        env,
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::CancelDca {},
    )
    .unwrap();

    assert_eq!(1, res.messages.len());
}

#[test]
fn dont_cancel_if_unauthorized() {
    let mut deps = do_instantiate();

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
fn should_be_able_to_pause() {
    let mut deps = do_instantiate();

    let env = mock_env();

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::PauseDca {},
    )
    .unwrap();

    let state = STATE.load(deps.as_ref().storage).unwrap();

    assert!(state.paused);
}

#[test]
fn should_be_able_to_resume_if_paused() {
    let mut deps = do_instantiate();

    let env = mock_env();

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::PauseDca {},
    )
    .unwrap();

    let state = STATE.load(deps.as_ref().storage).unwrap();

    assert!(state.paused);

    execute(
        deps.as_mut(),
        env,
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::ResumeDca {},
    )
    .unwrap();

    let state = STATE.load(deps.as_ref().storage).unwrap();

    assert!(!state.paused);
}

#[test]
fn cannot_be_paused_if_is_already_paused() {
    let mut deps = do_instantiate();

    let env = mock_env();

    execute(
        deps.as_mut(),
        env.clone(),
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::PauseDca {},
    )
    .unwrap();

    let state = STATE.load(deps.as_ref().storage).unwrap();

    assert!(state.paused);

    let err = execute(
        deps.as_mut(),
        env,
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::PauseDca {},
    )
    .unwrap_err();

    assert_eq!(err.to_string(), "DCA strategy is paused");
}

#[test]
fn cannot_be_resume_if_is_already_resumed() {
    let mut deps = do_instantiate();

    let env = mock_env();

    let state = STATE.load(deps.as_ref().storage).unwrap();

    assert!(!state.paused);

    let err = execute(
        deps.as_mut(),
        env,
        mock_info(ADMIN_ADDR, &[]),
        ExecuteMsg::ResumeDca {},
    )
    .unwrap_err();

    assert_eq!(err.to_string(), "DCA strategy is not paused");
}

#[test]
fn proper_token_string_to_coin() {
    let token_string = "100uosmo";

    let coin = token_string_to_coin(token_string).unwrap();

    assert_eq!(coin.amount, Uint128::from(100u128));
    assert_eq!(coin.denom, "uosmo".to_string());
}

#[test]
fn query_handler_claim_funds() {
    let mut deps = do_instantiate();
    let env = mock_env();
    let balance = vec![Coin::new(100, "uion"), Coin::new(100, "ujuno")];

    deps.querier
        .update_balance(env.contract.address.clone(), balance.clone());

    let res: Vec<Coin> =
        from_binary(&query(deps.as_ref(), env, QueryMsg::GetAllFunds {}).unwrap()).unwrap();

    assert_eq!(res, balance);
}

#[test]
fn cannot_perform_dca_if_trade_limit_reached() {
    let mut deps = do_instantiate();

    let env = fast_forward_time(mock_env(), 100);

    // Can execute because not limit reached
    execute(
        deps.as_mut(),
        env.clone(),
        mock_info("executor", &coins(100, "uion")),
        ExecuteMsg::PerformDca {},
    )
    .unwrap();

    let env = fast_forward_time(env, 100);

    STATE
        .update(deps.as_mut().storage, |mut s| -> StdResult<_> {
            s.num_trades_executed = Uint128::from(10u128);
            Ok(s)
        })
        .expect("error: updating state");

    // Cannot execute because limit reached
    let res = execute(
        deps.as_mut(),
        env,
        mock_info("executor", &coins(100, "uion")),
        ExecuteMsg::PerformDca {},
    )
    .unwrap_err();

    assert_eq!(res.to_string(), "Reached max trade limit");
}
