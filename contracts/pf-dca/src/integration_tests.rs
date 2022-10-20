#[cfg(test)]
mod tests {
    use phase_finance::CoinWeight;
    use crate::{helpers::CwTemplateContract, types::StrategyType};
    use crate::msg::InstantiateMsg;
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    pub fn contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        );
        Box::new(contract)
    }

    const USER: &str = "USER";
    const ADMIN: &str = "ADMIN";
    const NATIVE_DENOM: &str = "denom";

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER),
                    vec![Coin {
                        denom: NATIVE_DENOM.to_string(),
                        amount: Uint128::new(1),
                    }],
                )
                .unwrap();
        })
    }

    fn proper_instantiate() -> (App, CwTemplateContract) {
        let mut app = mock_app();
        let cw_template_id = app.store_code(contract_template());

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
        let cw_template_contract_addr = app
            .instantiate_contract(
                cw_template_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "test",
                None,
            )
            .unwrap();

        let cw_template_contract = CwTemplateContract(cw_template_contract_addr);

        (app, cw_template_contract)
    }

    mod count {
        
        

        // #[test]
        // fn count() {
        //     let (mut app, cw_template_contract) = proper_instantiate();

        //     let msg = ExecuteMsg::Increment {};
        //     let cosmos_msg = cw_template_contract.call(msg).unwrap();
        //     app.execute(Addr::unchecked(USER), cosmos_msg).unwrap();
        // }
    }
}
