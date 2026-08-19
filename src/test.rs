#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, Env, Vec,
};

use crate::contract::{StellarNotifyContract, StellarNotifyContractClient};
use crate::errors::NotifyError;
use crate::types::Channel;

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Deploy and initialise a fresh contract instance.
/// Returns (env, admin_address, client).
///
/// Settings:
///   max_per_owner = 20
///   max_ttl       = 100_000 ledgers
fn setup() -> (Env, Address, StellarNotifyContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &20u32, &100_000u32);
    (env, admin, client)
}

/// Create a minimal valid subscription and return its ID.
fn make_sub(
    env: &Env,
    client: &StellarNotifyContractClient,
    owner: &Address,
    watched: &Address,
) -> u64 {
    let topics: Vec<Bytes> = Vec::new(env);
    let endpoint = Bytes::from_slice(
        env,
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    client.subscribe(owner, watched, &topics, &Channel::Webhook, &endpoint, &0u32)
}

// ─────────────────────────────────────────────────────────────────────────────
// C23 — test_initialise_once
// ─────────────────────────────────────────────────────────────────────────────

/// Calling initialise() a second time must return AlreadyInitialised.
#[test]
fn test_initialise_once() {
    let (_, admin, client) = setup();
    let result = client.try_initialise(&admin, &20u32, &100_000u32);
    assert_eq!(
        result,
        Err(Ok(NotifyError::AlreadyInitialised)),
        "second initialise() must return AlreadyInitialised"
    );
}

/// Calling initialise() once must produce a readable config.
#[test]
fn test_initialise_config_stored() {
    let (_, admin, client) = setup();
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.max_per_owner, 20u32);
    assert_eq!(config.max_ttl, 100_000u32);
    assert!(!config.paused);
}

/// get_version() must return the expected version string.
#[test]
fn test_get_version() {
    let (_, _, client) = setup();
    assert_eq!(client.get_version(), "0.1.0");
}
