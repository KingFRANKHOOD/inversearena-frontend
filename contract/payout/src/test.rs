#[cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, PayoutContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PayoutContract, ());
    let client = PayoutContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let env_static: &'static Env = unsafe { &*(&env as *const Env) };
    let client = PayoutContractClient::new(env_static, &contract_id);
    (env, admin, client)
}

// ── initialize ────────────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let (_env, admin, client) = setup();
    assert_eq!(client.admin(), admin);
}

#[test]
fn test_double_initialize_returns_already_initialized() {
    let (_env, admin, client) = setup();
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(PayoutError::AlreadyInitialized)));
}

// ── distribute_winnings ───────────────────────────────────────────────────────

#[test]
fn test_admin_can_distribute_winnings() {
    let (env, admin, client) = setup();
    let winner = Address::generate(&env);
    let idempotency_key = 1u32;
    let amount = 1000i128;
    let currency = symbol_short!("XLM");

    assert!(!client.is_payout_processed(&idempotency_key, &winner));
    client.distribute_winnings(&admin, &idempotency_key, &winner, &amount, &currency);
    assert!(client.is_payout_processed(&idempotency_key, &winner));

    let payout = client.get_payout(&idempotency_key, &winner).unwrap();
    assert_eq!(payout.winner, winner);
    assert_eq!(payout.amount, amount);
    assert!(payout.paid);
}

#[test]
fn test_unauthorized_caller_returns_unauthorized() {
    let (env, _admin, client) = setup();
    let unauthorized = Address::generate(&env);
    let winner = Address::generate(&env);
    let idempotency_key = 1u32;
    let amount = 1000i128;
    let currency = symbol_short!("XLM");

    let result = client.try_distribute_winnings(
        &unauthorized,
        &idempotency_key,
        &winner,
        &amount,
        &currency,
    );
    assert_eq!(result, Err(Ok(PayoutError::Unauthorized)));
}

#[test]
fn test_zero_amount_returns_invalid_amount() {
    let (env, admin, client) = setup();
    let winner = Address::generate(&env);
    let idempotency_key = 1u32;
    let currency = symbol_short!("XLM");

    let result =
        client.try_distribute_winnings(&admin, &idempotency_key, &winner, &0, &currency);
    assert_eq!(result, Err(Ok(PayoutError::InvalidAmount)));
}

#[test]
fn test_negative_amount_returns_invalid_amount() {
    let (env, admin, client) = setup();
    let winner = Address::generate(&env);
    let idempotency_key = 1u32;
    let currency = symbol_short!("XLM");

    let result =
        client.try_distribute_winnings(&admin, &idempotency_key, &winner, &-100, &currency);
    assert_eq!(result, Err(Ok(PayoutError::InvalidAmount)));
}

#[test]
fn test_idempotency_prevents_double_pay_returns_already_processed() {
    let (env, admin, client) = setup();
    let winner = Address::generate(&env);
    let idempotency_key = 1u32;
    let amount = 1000i128;
    let currency = symbol_short!("XLM");

    client.distribute_winnings(&admin, &idempotency_key, &winner, &amount, &currency);
    let result =
        client.try_distribute_winnings(&admin, &idempotency_key, &winner, &amount, &currency);
    assert_eq!(result, Err(Ok(PayoutError::AlreadyProcessed)));
}

#[test]
fn test_distribute_winnings_when_not_initialized_returns_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(PayoutContract, ());
    let client = PayoutContractClient::new(&env, &contract_id);
    let caller = Address::generate(&env);
    let winner = Address::generate(&env);
    let currency = symbol_short!("XLM");

    let result = client.try_distribute_winnings(&caller, &1u32, &winner, &100, &currency);
    assert_eq!(result, Err(Ok(PayoutError::NotInitialized)));
}

#[test]
fn test_different_idempotency_keys_allow_multiple_payouts() {
    let (env, admin, client) = setup();
    let winner = Address::generate(&env);
    let amount = 1000i128;
    let currency = symbol_short!("XLM");

    client.distribute_winnings(&admin, &1u32, &winner, &amount, &currency);
    client.distribute_winnings(&admin, &2u32, &winner, &amount, &currency);

    assert!(client.is_payout_processed(&1u32, &winner));
    assert!(client.is_payout_processed(&2u32, &winner));
}

// ── get_payout ────────────────────────────────────────────────────────────────

#[test]
fn test_get_payout_returns_none_for_unprocessed() {
    let env = Env::default();
    let contract_id = env.register(PayoutContract, ());
    let client = PayoutContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let winner = Address::generate(&env);
    assert!(client.get_payout(&1u32, &winner).is_none());
}

#[test]
fn test_get_payout_returns_data_for_processed() {
    let (env, admin, client) = setup();
    let winner = Address::generate(&env);
    let idempotency_key = 1u32;
    let amount = 5000i128;
    let currency = symbol_short!("USDC");

    client.distribute_winnings(&admin, &idempotency_key, &winner, &amount, &currency);

    let payout = client.get_payout(&idempotency_key, &winner).unwrap();
    assert_eq!(payout.winner, winner);
    assert_eq!(payout.amount, amount);
    assert_eq!(payout.currency, currency);
    assert!(payout.paid);
}

// ── admin view ────────────────────────────────────────────────────────────────

#[test]
fn test_admin_returns_not_initialized_when_unset() {
    let env = Env::default();
    let contract_id = env.register(PayoutContract, ());
    let client = PayoutContractClient::new(&env, &contract_id);
    let result = client.try_admin();
    assert_eq!(result, Err(Ok(PayoutError::NotInitialized)));
}

// ── Schema versioning tests ──────────────────────────────────────────────────

#[test]
fn test_schema_version_set_on_init() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(PayoutContract, ());
    let client = PayoutContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    assert_eq!(client.schema_version(), 1);
}

#[test]
fn test_migrate_noop_when_current() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(PayoutContract, ());
    let client = PayoutContractClient::new(&env, &contract_id);
    client.initialize(&admin);
    client.migrate();
    assert_eq!(client.schema_version(), 1);
}

#[test]
fn test_migrate_from_v0() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(PayoutContract, ());
    let client = PayoutContractClient::new(&env, &contract_id);
    client.initialize(&admin);

    // Simulate a pre-versioning contract by clearing the version key.
    env.as_contract(&contract_id, || {
        env.storage().instance().remove(&symbol_short!("S_VER"));
    });
    assert_eq!(client.schema_version(), 0);

    client.migrate();
    assert_eq!(client.schema_version(), 1);
}
