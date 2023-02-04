use crate::multitest::common::*;
use cosmwasm_std::Addr;
use cw_multi_test::App;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct PhaseFinanceSuite {
    #[derivative(Debug = "ignore")]
    pub app: App,
    // The account that deploys everything
    pub deployer: Addr,
    // executor address
    pub executor: Addr,
    // user address
    pub user: Addr,
    // dca address
    pub dca: Addr,
    // router address
    pub router: Addr,
}

impl PhaseFinanceSuite {
    pub fn init(
        init_msg: Option<DCAInstantiateMsg>,
        funds: Option<Vec<Coin>>,
    ) -> Result<PhaseFinanceSuite> {
        let genesis_funds = vec![coin(150000, DENOM)];
        let deployer = Addr::unchecked(DEPLOYER);
        let executor = Addr::unchecked(EXECUTOR);
        let user = Addr::unchecked(USER);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer, genesis_funds)
                .unwrap();
        });
        app.send_tokens(deployer.clone(), user.clone(), &[coin(50000, DENOM)])?;
        app.send_tokens(deployer.clone(), executor.clone(), &[coin(50000, DENOM)])?;

        let dca_id = app.store_code(contract_dca());
        let router_id = app.store_code(contract_router());

        let router = app
            .instantiate_contract(
                router_id,
                deployer.clone(),
                &RouterInstantiateMsg {
                    owner: deployer.to_string(),
                },
                &[],
                "router_contract",
                Some(deployer.to_string()),
            )
            .unwrap();

        let dca = app
            .instantiate_contract(
                dca_id,
                deployer.clone(),
                &init_msg.unwrap_or(DCAInstantiateMsg {
                    recipient_address: user.to_string(),
                    executor_address: executor.to_string(),
                    router_contract: router.to_string(),
                    strategy_type: StrategyType::Linear,
                    max_slippage: Decimal::from_ratio(1u128, 100u128),
                    num_trades: Uint128::from(10u128),
                    swap_interval: Duration::Time(1),
                    source_denom: DENOM.to_string(),
                    amount_per_trade: Uint128::from(10u128),
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
                }),
                &funds.unwrap_or(vec![coin(100, DENOM)]),
                "dca_contract",
                Some(deployer.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        Ok(PhaseFinanceSuite {
            app,
            user,
            executor,
            deployer,
            router,
            dca,
        })
    }

    pub fn query_balance(&self, addr: &Addr) -> StdResult<Coin> {
        Ok(self.app.wrap().query_balance(addr.as_str(), "uosmo")?)
    }

    pub fn cancel_dca(&mut self, sender: &Addr) -> Result<(), DCAContractError> {
        let msg = DCAExecuteMsg::CancelDca {};
        self.app
            .execute_contract(sender.clone(), self.dca.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn resume_dca(&mut self, sender: &Addr) -> Result<(), DCAContractError> {
        let msg = DCAExecuteMsg::ResumeDca {};
        self.app
            .execute_contract(sender.clone(), self.dca.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn pause_dca(&mut self, sender: &Addr) -> Result<(), DCAContractError> {
        let msg = DCAExecuteMsg::PauseDca {};
        self.app
            .execute_contract(sender.clone(), self.dca.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn try_perform_dca(&mut self, sender: &Addr) -> Result<(), DCAContractError> {
        let msg = DCAExecuteMsg::PerformDca {};
        self.app
            .execute_contract(sender.clone(), self.dca.clone(), &msg, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    pub fn query_upcoming_swap(&self) -> StdResult<UpcomingSwapResponse> {
        let msg = DCAQueryMsg::GetUpcomingSwap {};
        self.app.wrap().query_wasm_smart(self.dca.clone(), &msg)
    }

    pub fn query_all_upcoming_swaps(&self) -> StdResult<Vec<UpcomingSwapResponse>> {
        let msg = DCAQueryMsg::GetAllUpcomingSwaps {};
        self.app.wrap().query_wasm_smart(self.dca.clone(), &msg)
    }

    pub fn query_bonded_funds(&self) -> StdResult<Coin> {
        let msg = DCAQueryMsg::GetSourceFunds {};
        self.app.wrap().query_wasm_smart(self.dca.clone(), &msg)
    }

    pub fn query_funds(&self) -> StdResult<Vec<Coin>> {
        let msg = DCAQueryMsg::GetAllFunds {};
        self.app.wrap().query_wasm_smart(self.dca.clone(), &msg)
    }

    pub fn query_state(&self) -> StdResult<DCAState> {
        let msg = DCAQueryMsg::State {};
        self.app.wrap().query_wasm_smart(self.dca.clone(), &msg)
    }

    pub fn query_config(&self) -> StdResult<DcaConfig> {
        let msg = DCAQueryMsg::Config {};
        self.app.wrap().query_wasm_smart(self.dca.clone(), &msg)
    }

    pub fn fast_forward_block_time(&mut self, forward_time_sec: u64) {
        let block = self.app.block_info();

        let mock_block = BlockInfo {
            height: block.height + 10,
            chain_id: block.chain_id,
            time: block.time.plus_seconds(forward_time_sec),
        };

        self.app.set_block(mock_block);
    }
}
