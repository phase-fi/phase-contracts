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
    // Dao address
    pub user: Addr,
    // dca address
    pub dca: Addr,
    // router address
    pub router: Addr,
}

impl PhaseFinanceSuite {
    pub fn init() -> Result<PhaseFinanceSuite> {
        let genesis_funds = vec![coin(100000, "ucosm")];
        let deployer = Addr::unchecked("deployer");
        let user = Addr::unchecked(USER);
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &deployer, genesis_funds)
                .unwrap();
        });
        app.send_tokens(deployer.clone(), user.clone(), &[coin(50000, "ucosm")])?;

        let dca_id = app.store_code(contract_dca());
        let router_id = app.store_code(contract_router());

        let dca = app
            .instantiate_contract(
                dca_id,
                deployer.clone(),
                &{},
                &[],
                "dca_contract",
                Some(deployer.to_string()), // admin: Option<String>, will need this for upgrading
            )
            .unwrap();

        let router = app
            .instantiate_contract(
                router_id,
                deployer.clone(),
                &{},
                &[],
                "router_contract",
                Some(user.to_string()),
            )
            .unwrap();

        Ok(PhaseFinanceSuite {
            app,
            user,
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
}
