use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{coins, from_binary, BlockInfo, Coin, OwnedDeps, Timestamp, Uint128};

use cw_utils::Duration;
use phase_finance::types::{CoinWeight, StrategyType};

use crate::contract::{execute, instantiate, query};
use crate::helpers::token_string_to_coin;

use phase_finance::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

pub const ADMIN_ADDR: &str = "admin_addr";

fn do_instantiate() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let info = mock_info(ADMIN_ADDR, &coins(1000000000, "uosmo"));
    let env = mock_env();

    let instantiate_msg = InstantiateMsg {
        destination_wallet: "osmo123".to_string(),
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
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Time(1),
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
    };

    instantiate(deps.as_mut(), env, info, instantiate_msg).unwrap();

    deps
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let msg = InstantiateMsg {
        destination_wallet: "osmo123".to_string(),
        strategy_type: StrategyType::Linear,
        destinations: vec![CoinWeight {
            denom: "uion".to_string(),
            weight: Uint128::from(100u128),
        }],
        amount_per_trade: Uint128::from(10u128),
        num_trades: Uint128::from(10u128),
        swap_interval: Duration::Height(100_000_000_000),
        source_denom: "uosmo".to_string(),
        router_contract: "osmoabc".to_string(),
    };

    let info = mock_info("creator", &coins(100, "uosmo"));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
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
        mock_info("creator", &coins(100, "uion")),
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
fn proper_token_string_to_coin() {
    let token_string = "100uosmo";
    let _text = "100usomo";
    // println!("ya moms a hoe");
    // for mat in Regex::new(r"(?<=\d)(?=\D)|(?<=\D)(?=\d)").unwrap().find_iter(text) {
    //     println!("mat {:?}", mat);
    // }

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
