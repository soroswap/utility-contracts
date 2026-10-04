use soroban_sdk::{Address, Env, Vec};
use utility_contracts::timelock::TimelockClient;
use utility_contracts::pair::PairClient;
use utility_contracts::token::TokenClient;

fn setup() -> (Env, TimelockClient, PairClient, TokenClient, TokenClient) {
    let env = Env::default();
    
    // Deploy tokens
    let token1 = TokenClient::new(&env, &Address::random(&env));
    let token2 = TokenClient::new(&env, &Address::random(&env));
    
    // Deploy pair
    let pair = PairClient::new(&env, &Address::random(&env));
    pair.initialize(&token1.address(), &token2.address());
    
    // Deploy timelock
    let timelock = TimelockClient::new(&env, &Address::random(&env));
    
    // Mint tokens to timelock
    token1.mint(&timelock.address(), &10000);
    token2.mint(&timelock.address(), &10000);
    
    (env, timelock, pair, token1, token2)
}

#[test]
fn test_add_liquidity() {
    let (env, timelock, pair, token1, token2) = setup();
    
    // Set lock time
    timelock.set_lock_time(&100);
    
    // Add liquidity
    let amount1 = 1000;
    let amount2 = 1000;
    timelock.add_liquidity(&token1.address(), &token2.address(), &amount1, &amount2);
    
    // Verify pair balances
    assert_eq!(token1.balance(&pair.address()), amount1);
    assert_eq!(token2.balance(&pair.address()), amount2);
    
    // Verify timelock balances decreased
    assert_eq!(token1.balance(&timelock.address()), 9000);
    assert_eq!(token2.balance(&timelock.address()), 9000);
}

#[test]
fn test_withdraw_before_time_fails() {
    let (env, timelock, pair, token1, token2) = setup();
    
    timelock.set_lock_time(&100);
    timelock.add_liquidity(&token1.address(), &token2.address(), &1000, &1000);
    
    // Attempt withdrawal before time
    let result = timelock.try_withdraw();
    assert!(result.is_err());
}

#[test]
fn test_withdraw_after_time() {
    let (env, timelock, pair, token1, token2) = setup();
    
    timelock.set_lock_time(&100);
    timelock.add_liquidity(&token1.address(), &token2.address(), &1000, &1000);
    
    // Move time forward
    env.ledger().set_timestamp(101);
    
    // Withdraw should succeed
    timelock.withdraw();
    
    // Verify LP tokens transferred to timelock (assuming pair mints LP to timelock)
    let lp_token = pair.lp_token();
    assert!(lp_token.balance(&timelock.address()) > 0);
}
