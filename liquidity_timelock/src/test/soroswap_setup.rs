#![cfg(test)]
extern crate std;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env,
};
// Token Contract
pub mod token {
    soroban_sdk::contractimport!(
        file = "./soroswap_contracts/soroban_token_contract.optimized.wasm"
    );
    pub type TokenClient<'a> = Client<'a>;
}
pub use token::TokenClient;

pub fn create_token_contract<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    TokenClient::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

fn pair_contract_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(file = "./soroswap_contracts/soroswap_pair.optimized.wasm");
    e.deployer().upload_contract_wasm(WASM)
}

pub mod pair {
    soroban_sdk::contractimport!(file = "./soroswap_contracts/soroswap_pair.optimized.wasm");
    pub type SoroswapPairClient<'a> = Client<'a>;
}
pub use pair::SoroswapPairClient;

// SoroswapFactory Contract
pub mod factory {
    soroban_sdk::contractimport!(file = "./soroswap_contracts/soroswap_factory.optimized.wasm");
    pub type SoroswapFactoryClient<'a> = Client<'a>;
}
use factory::SoroswapFactoryClient;

fn create_soroswap_factory<'a>(e: &Env, setter: &Address) -> SoroswapFactoryClient<'a> {
    let pair_hash = pair_contract_wasm(&e);
    let factory_address = &e.register_contract_wasm(None, factory::WASM);
    let factory = SoroswapFactoryClient::new(e, factory_address);
    factory.initialize(&setter, &pair_hash);
    factory
}

// SoroswapRouter Contract
pub mod router {
    soroban_sdk::contractimport!(file = "./soroswap_contracts/soroswap_router.optimized.wasm");
    pub type SoroswapRouterClient<'a> = Client<'a>;
}
use router::SoroswapRouterClient;

// SoroswapRouter Contract
fn create_soroswap_router<'a>(e: &Env) -> SoroswapRouterClient<'a> {
    let router_address = &e.register_contract_wasm(None, router::WASM);
    let router = SoroswapRouterClient::new(e, router_address);
    router
}

pub struct SoroswapTest<'a> {
    pub env: Env,
    pub router_contract: SoroswapRouterClient<'a>,
    pub factory_contract: SoroswapFactoryClient<'a>,
    pub pair_address: Address,
    pub token_0: TokenClient<'a>,
    pub token_1: TokenClient<'a>,
    // pub token_2: TokenClient<'a>,
    pub user: Address,
    pub admin: Address,
}

impl<'a> SoroswapTest<'a> {
    pub fn soroswap_setup() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let router_contract = create_soroswap_router(&env);

        let initial_user_balance = 10_000_000_000_000_000_000;

        let admin = Address::generate(&env);
        let user = Address::generate(&env);
        assert_ne!(admin, user);

        let mut token_0 = create_token_contract(&env, &admin);
        let mut token_1 = create_token_contract(&env, &admin);
        let token_2 = create_token_contract(&env, &admin);
        if &token_1.address < &token_0.address {
            std::mem::swap(&mut token_0, &mut token_1);
        }
        token_0.mint(&user, &initial_user_balance);
        token_1.mint(&user, &initial_user_balance);
        token_2.mint(&user, &initial_user_balance);

        token_0.mint(&admin, &initial_user_balance);
        token_1.mint(&admin, &initial_user_balance);
        token_2.mint(&admin, &initial_user_balance);

        let factory_contract = create_soroswap_factory(&env, &admin);
        env.budget().reset_unlimited();

        let ledger_timestamp = 100;
        let desired_deadline = 1000;

        assert!(desired_deadline > ledger_timestamp);

        env.ledger().with_mut(|li| {
            li.timestamp = ledger_timestamp;
        });

        let amount_0: i128 = 1_000_000_000_000_000_000;
        let amount_1: i128 = 4_000_000_000_000_000_000;
        let expected_liquidity: i128 = 2_000_000_000_000_000_000;

        // Check initial user value of every token:
        assert_eq!(token_0.balance(&user), initial_user_balance);
        assert_eq!(token_1.balance(&user), initial_user_balance);
        assert_eq!(token_2.balance(&user), initial_user_balance);

        router_contract.initialize(&factory_contract.address);

        assert_eq!(
            factory_contract.pair_exists(&token_0.address, &token_1.address),
            false
        );
        let (added_token_0, added_token_1, added_liquidity) = router_contract.add_liquidity(
            &token_0.address,  //     token_a: Address,
            &token_1.address,  //     token_b: Address,
            &amount_0,         //     amount_a_desired: i128,
            &amount_1,         //     amount_b_desired: i128,
            &0,                //     amount_a_min: i128,
            &0,                //     amount_b_min: i128,
            &user,             //     to: Address,
            &desired_deadline, //     deadline: u64,
        );

        let (_added_token_2, _added_token_3, _added_liquidity_2) = router_contract.add_liquidity(
            &token_1.address,  //     token_a: Address,
            &token_2.address,  //     token_b: Address,
            &amount_1,         //     amount_a_desired: i128,
            &amount_0,         //     amount_b_desired: i128,
            &0,                //     amount_a_min: i128,
            &0,                //     amount_b_min: i128,
            &user,             //     to: Address,
            &desired_deadline, //     deadline: u64,
        );

        let (added_token_2, added_token_3, added_liquidity_2) = router_contract.add_liquidity(
            &token_0.address,  //     token_a: Address,
            &token_2.address,  //     token_b: Address,
            &amount_0,         //     amount_a_desired: i128,
            &amount_1,         //     amount_b_desired: i128,
            &0,                //     amount_a_min: i128,
            &0,                //     amount_b_min: i128,
            &user,             //     to: Address,
            &desired_deadline, //     deadline: u64,
        );

        static MINIMUM_LIQUIDITY: i128 = 1000;

        assert_eq!(added_token_0, amount_0);
        assert_eq!(added_token_1, amount_1);
        assert_eq!(added_token_2, amount_0);
        assert_eq!(added_token_3, amount_1);
        assert_eq!(
            added_liquidity,
            expected_liquidity.checked_sub(MINIMUM_LIQUIDITY).unwrap()
        );
        assert_eq!(
            added_liquidity_2,
            expected_liquidity.checked_sub(MINIMUM_LIQUIDITY).unwrap()
        );

        assert_eq!(token_0.balance(&user), 8_000_000_000_000_000_000);
        assert_eq!(token_1.balance(&user), 2_000_000_000_000_000_000);
        assert_eq!(token_2.balance(&user), 5_000_000_000_000_000_000);

        let pair_address = factory_contract.get_pair(&token_0.address, &token_1.address);
        SoroswapTest {
            env,
            router_contract,
            pair_address,
            factory_contract,
            token_0,
            token_1,
            // token_2,
            user,
            admin,
        }
    }
}
