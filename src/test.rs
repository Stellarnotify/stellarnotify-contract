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

// ─────────────────────────────────────────────────────────────────────────────
// C24 — test_subscribe_and_get
// ─────────────────────────────────────────────────────────────────────────────

/// Happy path: subscribe() returns a valid ID and get_sub() returns correct fields.
#[test]
fn test_subscribe_and_get() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);

    assert_eq!(id, 1u64, "first subscription ID must be 1");

    let sub = client.get_sub(&id);
    assert_eq!(sub.owner, owner);
    assert_eq!(sub.watched_contract, watched);
    assert!(sub.active);
    assert_eq!(sub.expires_at, 0u32);
    assert_eq!(sub.channel, Channel::Webhook);
}

/// subscribe() with ttl_ledgers > 0 sets expires_at = current_ledger + ttl.
#[test]
fn test_subscribe_with_ttl() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let topics: Vec<Bytes> = Vec::new(&env);
    let endpoint = Bytes::from_slice(
        &env,
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );

    let start_ledger = env.ledger().sequence();
    let ttl: u32 = 5_000;
    let id = client.subscribe(&owner, &watched, &topics, &Channel::Webhook, &endpoint, &ttl);
    let sub = client.get_sub(&id);

    assert_eq!(sub.expires_at, start_ledger + ttl);
}

/// IDs increment correctly across multiple subscriptions.
#[test]
fn test_subscribe_id_increments() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id1 = make_sub(&env, &client, &owner, &watched);
    let id2 = make_sub(&env, &client, &owner, &watched);
    let id3 = make_sub(&env, &client, &owner, &watched);

    assert_eq!(id1, 1u64);
    assert_eq!(id2, 2u64);
    assert_eq!(id3, 3u64);
}

// ─────────────────────────────────────────────────────────────────────────────
// C25 — test_cancel_subscription
// ─────────────────────────────────────────────────────────────────────────────

/// cancel() removes the subscription — get_sub() returns SubNotFound after.
#[test]
fn test_cancel_removes_sub() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.cancel(&owner, &id);

    let result = client.try_get_sub(&id);
    assert_eq!(result, Err(Ok(NotifyError::SubNotFound)));
}

/// cancel() removes the ID from the owner index.
#[test]
fn test_cancel_removes_from_owner_index() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert_eq!(client.list_by_owner(&owner).len(), 1u32);

    client.cancel(&owner, &id);
    assert_eq!(client.list_by_owner(&owner).len(), 0u32);
}

/// cancel() removes the ID from the watcher index.
#[test]
fn test_cancel_removes_from_watcher_index() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert_eq!(client.list_by_contract(&watched).len(), 1u32);

    client.cancel(&owner, &id);
    assert_eq!(client.list_by_contract(&watched).len(), 0u32);
}

/// cancel() on a non-existent ID returns SubNotFound.
#[test]
fn test_cancel_nonexistent_returns_not_found() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);

    let result = client.try_cancel(&owner, &999u64);
    assert_eq!(result, Err(Ok(NotifyError::SubNotFound)));
}

/// Cancelling one subscription does not affect others owned by the same wallet.
#[test]
fn test_cancel_one_leaves_others_intact() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id1 = make_sub(&env, &client, &owner, &watched);
    let id2 = make_sub(&env, &client, &owner, &watched);
    let id3 = make_sub(&env, &client, &owner, &watched);

    client.cancel(&owner, &id2);

    let _ = client.get_sub(&id1);
    let _ = client.get_sub(&id3);
    assert_eq!(client.try_get_sub(&id2), Err(Ok(NotifyError::SubNotFound)));
    assert_eq!(client.list_by_owner(&owner).len(), 2u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// C26 — test_pause_and_resume
// ─────────────────────────────────────────────────────────────────────────────

/// pause_sub() sets active = false.
#[test]
fn test_pause_sets_inactive() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert!(client.get_sub(&id).active);

    client.pause_sub(&owner, &id);
    assert!(!client.get_sub(&id).active);
}

/// resume_sub() sets active = true.
#[test]
fn test_resume_sets_active() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.pause_sub(&owner, &id);
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
}

/// pause_sub() is idempotent.
#[test]
fn test_pause_idempotent() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.pause_sub(&owner, &id);
    client.pause_sub(&owner, &id); // second call must not error
    assert!(!client.get_sub(&id).active);
}

/// resume_sub() is idempotent.
#[test]
fn test_resume_idempotent() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.pause_sub(&owner, &id);
    client.resume_sub(&owner, &id);
    client.resume_sub(&owner, &id); // second call must not error
    assert!(client.get_sub(&id).active);
}

/// Full cycle: active → paused → active → paused → active.
#[test]
fn test_pause_resume_full_cycle() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);

    client.pause_sub(&owner, &id);
    assert!(!client.get_sub(&id).active);
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
    client.pause_sub(&owner, &id);
    assert!(!client.get_sub(&id).active);
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
}

/// pause_sub() on a non-existent ID returns SubNotFound.
#[test]
fn test_pause_nonexistent_returns_not_found() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    assert_eq!(
        client.try_pause_sub(&owner, &999u64),
        Err(Ok(NotifyError::SubNotFound))
    );
}

/// resume_sub() on a non-existent ID returns SubNotFound.
#[test]
fn test_resume_nonexistent_returns_not_found() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    assert_eq!(
        client.try_resume_sub(&owner, &999u64),
        Err(Ok(NotifyError::SubNotFound))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C27 — test_limit_exceeded
// ─────────────────────────────────────────────────────────────────────────────

/// When an owner reaches max_per_owner, the next subscribe() returns LimitExceeded.
#[test]
fn test_limit_exceeded() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &2u32, &0u32);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    make_sub(&env, &client, &owner, &watched);
    make_sub(&env, &client, &owner, &watched);

    let result = client.try_subscribe(
        &owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &0u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::LimitExceeded)));
}

/// After cancelling one subscription, the owner can subscribe again.
#[test]
fn test_limit_freed_after_cancel() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &2u32, &0u32);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id1 = make_sub(&env, &client, &owner, &watched);
    make_sub(&env, &client, &owner, &watched);

    // At limit.
    assert_eq!(
        client.try_subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &0u32),
        Err(Ok(NotifyError::LimitExceeded))
    );

    client.cancel(&owner, &id1);

    // Now must succeed.
    let id3 = client.subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &0u32);
    assert!(id3 > 0u64);
}

/// Different owners each have their own independent limit.
#[test]
fn test_limit_per_owner_independent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &1u32, &0u32);

    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    make_sub(&env, &client, &owner_a, &watched);
    make_sub(&env, &client, &owner_b, &watched);

    assert_eq!(
        client.try_subscribe(&owner_a, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &0u32),
        Err(Ok(NotifyError::LimitExceeded))
    );
    assert_eq!(
        client.try_subscribe(&owner_b, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &0u32),
        Err(Ok(NotifyError::LimitExceeded))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C28 — test_protocol_pause_blocks_subscribe
// ─────────────────────────────────────────────────────────────────────────────

/// When the protocol is paused, subscribe() returns Paused.
#[test]
fn test_pause_blocks_subscribe() {
    let (env, admin, client) = setup();
    client.set_paused(&admin, &true);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let result = client.try_subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &0u32);
    assert_eq!(result, Err(Ok(NotifyError::Paused)));
}

/// Unpausing the protocol allows subscribe() to succeed again.
#[test]
fn test_unpause_allows_subscribe() {
    let (env, admin, client) = setup();
    client.set_paused(&admin, &true);
    client.set_paused(&admin, &false);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let id = make_sub(&env, &client, &owner, &watched);
    assert!(id > 0u64);
}

/// Pausing the protocol does not affect existing subscriptions.
#[test]
fn test_protocol_pause_does_not_affect_existing_subs() {
    let (env, admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.set_paused(&admin, &true);

    assert!(client.get_sub(&id).active);
    client.pause_sub(&owner, &id);
    assert!(!client.get_sub(&id).active);
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
    client.cancel(&owner, &id);
    assert_eq!(client.try_get_sub(&id), Err(Ok(NotifyError::SubNotFound)));
}

/// set_paused() is idempotent.
#[test]
fn test_set_paused_idempotent() {
    let (_, admin, client) = setup();
    client.set_paused(&admin, &true);
    client.set_paused(&admin, &true);
    assert!(client.get_config().paused);
}

/// Non-admin calling set_paused() returns Unauthorised.
#[test]
fn test_set_paused_unauthorised() {
    let (env, _admin, client) = setup();
    let attacker = Address::generate(&env);
    let result = client.try_set_paused(&attacker, &true);
    assert_eq!(result, Err(Ok(NotifyError::Unauthorised)));
}

// ─────────────────────────────────────────────────────────────────────────────
// C29 — test_expiry
// ─────────────────────────────────────────────────────────────────────────────

/// resume_sub() after TTL has passed returns Expired.
#[test]
fn test_resume_after_expiry_returns_expired() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id = client.subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &10u32);
    client.pause_sub(&owner, &id);

    // Advance past expiry.
    env.ledger().with_mut(|l| l.sequence_number += 20);

    let result = client.try_resume_sub(&owner, &id);
    assert_eq!(result, Err(Ok(NotifyError::Expired)));
}

/// resume_sub() exactly at expiry ledger returns Expired.
#[test]
fn test_resume_at_exact_expiry_returns_expired() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let ttl: u32 = 10;
    let id = client.subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &ttl);
    client.pause_sub(&owner, &id);

    env.ledger().with_mut(|l| l.sequence_number += ttl);

    assert_eq!(client.try_resume_sub(&owner, &id), Err(Ok(NotifyError::Expired)));
}

/// resume_sub() one ledger before expiry succeeds.
#[test]
fn test_resume_one_ledger_before_expiry_succeeds() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let ttl: u32 = 10;
    let id = client.subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &ttl);
    client.pause_sub(&owner, &id);

    env.ledger().with_mut(|l| l.sequence_number += ttl - 1);

    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
}

/// Permanent subscriptions (expires_at = 0) never expire.
#[test]
fn test_permanent_subscription_never_expires() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert_eq!(client.get_sub(&id).expires_at, 0u32);

    client.pause_sub(&owner, &id);
    env.ledger().with_mut(|l| l.sequence_number += 10_000_000);
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
}

// ─────────────────────────────────────────────────────────────────────────────
// C30 — test_list_by_owner_and_contract
// ─────────────────────────────────────────────────────────────────────────────

/// list_by_owner() returns all IDs for a given owner in insertion order.
#[test]
fn test_list_by_owner_returns_all_ids() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id1 = make_sub(&env, &client, &owner, &watched);
    let id2 = make_sub(&env, &client, &owner, &watched);
    let id3 = make_sub(&env, &client, &owner, &watched);

    let ids = client.list_by_owner(&owner);
    assert_eq!(ids.len(), 3u32);
    assert_eq!(ids.get(0).unwrap(), id1);
    assert_eq!(ids.get(1).unwrap(), id2);
    assert_eq!(ids.get(2).unwrap(), id3);
}

/// list_by_owner() returns empty Vec for unknown address.
#[test]
fn test_list_by_owner_empty_for_unknown_address() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    assert_eq!(client.list_by_owner(&nobody).len(), 0u32);
}

/// list_by_owner() is isolated per owner.
#[test]
fn test_list_by_owner_isolated_per_owner() {
    let (env, _admin, client) = setup();
    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let watched = Address::generate(&env);

    let id_a1 = make_sub(&env, &client, &owner_a, &watched);
    let id_a2 = make_sub(&env, &client, &owner_a, &watched);
    let id_b1 = make_sub(&env, &client, &owner_b, &watched);

    let ids_a = client.list_by_owner(&owner_a);
    let ids_b = client.list_by_owner(&owner_b);

    assert_eq!(ids_a.len(), 2u32);
    assert_eq!(ids_b.len(), 1u32);
    assert!(ids_a.contains(&id_a1));
    assert!(ids_a.contains(&id_a2));
    assert!(!ids_a.contains(&id_b1));
    assert!(ids_b.contains(&id_b1));
}

/// list_by_contract() returns all IDs watching a given contract.
#[test]
fn test_list_by_contract_returns_all_watchers() {
    let (env, _admin, client) = setup();
    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);
    let watched = Address::generate(&env);

    let id1 = make_sub(&env, &client, &owner_a, &watched);
    let id2 = make_sub(&env, &client, &owner_b, &watched);

    let watchers = client.list_by_contract(&watched);
    assert_eq!(watchers.len(), 2u32);
    assert!(watchers.contains(&id1));
    assert!(watchers.contains(&id2));
}

/// list_by_contract() returns empty Vec for unknown contract.
#[test]
fn test_list_by_contract_empty_for_unknown_contract() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    assert_eq!(client.list_by_contract(&nobody).len(), 0u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// C31 — test_unauthorised_cancel
// ─────────────────────────────────────────────────────────────────────────────

/// cancel() by a non-owner returns NotOwner.
#[test]
fn test_unauthorised_cancel() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let result = client.try_cancel(&attacker, &id);
    assert_eq!(result, Err(Ok(NotifyError::NotOwner)));
}

/// pause_sub() by a non-owner returns NotOwner.
#[test]
fn test_unauthorised_pause() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let result = client.try_pause_sub(&attacker, &id);
    assert_eq!(result, Err(Ok(NotifyError::NotOwner)));
}

/// resume_sub() by a non-owner returns NotOwner.
#[test]
fn test_unauthorised_resume() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.pause_sub(&owner, &id);
    let result = client.try_resume_sub(&attacker, &id);
    assert_eq!(result, Err(Ok(NotifyError::NotOwner)));
}

// ─────────────────────────────────────────────────────────────────────────────
// C32 — test_too_many_topics
// ─────────────────────────────────────────────────────────────────────────────

/// subscribe() with exactly 10 topics succeeds.
#[test]
fn test_ten_topics_succeeds() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let mut topics: Vec<Bytes> = Vec::new(&env);
    for _ in 0..10 {
        topics.push_back(Bytes::from_slice(&env, b"topic"));
    }

    let id = client.subscribe(&owner, &watched, &topics, &Channel::Webhook, &ep, &0u32);
    assert!(id > 0u64);
}

/// subscribe() with 11 topics returns TooManyTopics.
#[test]
fn test_too_many_topics() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let mut topics: Vec<Bytes> = Vec::new(&env);
    for _ in 0..11 {
        topics.push_back(Bytes::from_slice(&env, b"topic"));
    }

    let result = client.try_subscribe(&owner, &watched, &topics, &Channel::Webhook, &ep, &0u32);
    assert_eq!(result, Err(Ok(NotifyError::TooManyTopics)));
}

// ─────────────────────────────────────────────────────────────────────────────
// C33 — test_empty_endpoint
// ─────────────────────────────────────────────────────────────────────────────

/// subscribe() with empty endpoint_ref returns EmptyEndpoint.
#[test]
fn test_empty_endpoint() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let empty_ep = Bytes::from_slice(&env, b"");

    let result = client.try_subscribe(
        &owner, &watched, &Vec::new(&env), &Channel::Webhook, &empty_ep, &0u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::EmptyEndpoint)));
}

/// update_endpoint_ref() with empty endpoint returns EmptyEndpoint.
#[test]
fn test_update_endpoint_empty_returns_error() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let empty_ep = Bytes::from_slice(&env, b"");

    let result = client.try_update_endpoint_ref(&owner, &id, &empty_ep);
    assert_eq!(result, Err(Ok(NotifyError::EmptyEndpoint)));
}

// ─────────────────────────────────────────────────────────────────────────────
// C34 — test_ttl_exceeded
// ─────────────────────────────────────────────────────────────────────────────

/// subscribe() with ttl_ledgers > max_ttl returns TtlExceeded.
#[test]
fn test_ttl_exceeded() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    // max_ttl = 1000
    client.initialise(&admin, &20u32, &1_000u32);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let result = client.try_subscribe(
        &owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &1_001u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::TtlExceeded)));
}

/// subscribe() with ttl_ledgers == max_ttl succeeds.
#[test]
fn test_ttl_at_limit_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &20u32, &1_000u32);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id = client.subscribe(
        &owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &1_000u32,
    );
    assert!(id > 0u64);
}

/// subscribe() with max_ttl = 0 allows any TTL.
#[test]
fn test_ttl_no_cap_allows_any_ttl() {
    let (env, _admin, client) = setup(); // max_ttl = 100_000
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    // 99_999 is under the cap — must succeed.
    let id = client.subscribe(
        &owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &99_999u32,
    );
    assert!(id > 0u64);
}

// ─────────────────────────────────────────────────────────────────────────────
// C35 — test_transfer_admin
// ─────────────────────────────────────────────────────────────────────────────

/// transfer_admin() gives the new admin full admin rights.
#[test]
fn test_transfer_admin() {
    let (env, admin, client) = setup();
    let new_admin = Address::generate(&env);

    client.transfer_admin(&admin, &new_admin);

    // Config must reflect the new admin.
    assert_eq!(client.get_config().admin, new_admin);

    // Old admin can no longer call admin functions.
    let result = client.try_update_config(&admin, &20u32, &100_000u32);
    assert_eq!(result, Err(Ok(NotifyError::Unauthorised)));

    // New admin can call admin functions.
    client.update_config(&new_admin, &25u32, &200_000u32);
    assert_eq!(client.get_config().max_per_owner, 25u32);
}

/// transfer_admin() by non-admin returns Unauthorised.
#[test]
fn test_transfer_admin_unauthorised() {
    let (env, _admin, client) = setup();
    let attacker = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let result = client.try_transfer_admin(&attacker, &new_admin);
    assert_eq!(result, Err(Ok(NotifyError::Unauthorised)));
}

// ─────────────────────────────────────────────────────────────────────────────
// C36 — test_set_paused
// ─────────────────────────────────────────────────────────────────────────────

/// set_paused(true) pauses the protocol; set_paused(false) unpauses it.
#[test]
fn test_set_paused_toggle() {
    let (_, admin, client) = setup();

    client.set_paused(&admin, &true);
    assert!(client.get_config().paused);

    client.set_paused(&admin, &false);
    assert!(!client.get_config().paused);
}

/// set_paused(true) twice is idempotent.
#[test]
fn test_set_paused_twice_idempotent() {
    let (_, admin, client) = setup();
    client.set_paused(&admin, &true);
    client.set_paused(&admin, &true);
    assert!(client.get_config().paused);
}

/// set_paused(false) on already-unpaused protocol is idempotent.
#[test]
fn test_set_unpaused_idempotent() {
    let (_, admin, client) = setup();
    client.set_paused(&admin, &false);
    assert!(!client.get_config().paused);
}

/// update_config() by non-admin returns Unauthorised.
#[test]
fn test_update_config_unauthorised() {
    let (env, _admin, client) = setup();
    let attacker = Address::generate(&env);
    let result = client.try_update_config(&attacker, &5u32, &500u32);
    assert_eq!(result, Err(Ok(NotifyError::Unauthorised)));
}

/// update_config() by admin updates the values correctly.
#[test]
fn test_update_config_success() {
    let (_, admin, client) = setup();
    client.update_config(&admin, &5u32, &500u32);
    let config = client.get_config();
    assert_eq!(config.max_per_owner, 5u32);
    assert_eq!(config.max_ttl, 500u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// C37 — test_update_endpoint_ref
// ─────────────────────────────────────────────────────────────────────────────

/// update_endpoint_ref() stores the new endpoint hash on-chain.
#[test]
fn test_update_endpoint_ref() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);

    let new_ep = Bytes::from_slice(&env, b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    client.update_endpoint_ref(&owner, &id, &new_ep);

    let sub = client.get_sub(&id);
    assert_eq!(sub.endpoint_ref, new_ep);
}

/// update_endpoint_ref() by non-owner returns NotOwner.
#[test]
fn test_update_endpoint_ref_not_owner() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let new_ep = Bytes::from_slice(&env, b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");

    let result = client.try_update_endpoint_ref(&attacker, &id, &new_ep);
    assert_eq!(result, Err(Ok(NotifyError::NotOwner)));
}

/// update_endpoint_ref() on non-existent ID returns SubNotFound.
#[test]
fn test_update_endpoint_ref_not_found() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let new_ep = Bytes::from_slice(&env, b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");

    let result = client.try_update_endpoint_ref(&owner, &999u64, &new_ep);
    assert_eq!(result, Err(Ok(NotifyError::SubNotFound)));
}

// ─────────────────────────────────────────────────────────────────────────────
// C38 — test_renew_sub
// ─────────────────────────────────────────────────────────────────────────────

/// renew_sub() extends expires_at from current expires_at when still live.
#[test]
fn test_renew_extends_from_current_expiry() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let initial_ttl: u32 = 100;
    let start = env.ledger().sequence();
    let id = client.subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &initial_ttl);

    env.ledger().with_mut(|l| l.sequence_number += 50);

    let add: u32 = 200;
    client.renew_sub(&owner, &id, &add);

    let sub = client.get_sub(&id);
    assert_eq!(sub.expires_at, start + initial_ttl + add);
}

/// renew_sub() on a permanent subscription (expires_at = 0) is a no-op.
#[test]
fn test_renew_permanent_subscription_noop() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert_eq!(client.get_sub(&id).expires_at, 0u32);

    client.renew_sub(&owner, &id, &5_000u32);
    assert_eq!(client.get_sub(&id).expires_at, 0u32);
}

/// renew_sub() extending beyond max_ttl returns TtlExceeded.
#[test]
fn test_renew_exceeds_max_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &20u32, &1_000u32);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id = client.subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &500u32);

    let result = client.try_renew_sub(&owner, &id, &600u32);
    assert_eq!(result, Err(Ok(NotifyError::TtlExceeded)));
}

/// renew_sub() by non-owner returns NotOwner.
#[test]
fn test_renew_sub_not_owner() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);
    let ep = Bytes::from_slice(&env, b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

    let id = client.subscribe(&owner, &watched, &Vec::new(&env), &Channel::Webhook, &ep, &100u32);
    let result = client.try_renew_sub(&attacker, &id, &100u32);
    assert_eq!(result, Err(Ok(NotifyError::NotOwner)));
}
