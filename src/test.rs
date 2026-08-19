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

fn setup() -> (Env, Address, StellarNotifyContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &20u32, &100_000u32);
    (env, admin, client)
}

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

fn ep(env: &Env) -> Bytes {
    Bytes::from_slice(
        env,
        b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
}

// ─────────────────────────────────────────────────────────────────────────────
// C23 — test_initialise_once
// ─────────────────────────────────────────────────────────────────────────────

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

#[test]
fn test_initialise_config_stored() {
    let (_, admin, client) = setup();
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert_eq!(config.max_per_owner, 20u32);
    assert_eq!(config.max_ttl, 100_000u32);
    assert!(!config.paused);
}

#[test]
fn test_get_version() {
    let (env, _, client) = setup();
    assert_eq!(client.get_version(), soroban_sdk::String::from_str(&env, "0.1.0"));
}

// ─────────────────────────────────────────────────────────────────────────────
// C24 — test_subscribe_and_get
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_subscribe_and_get() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert_eq!(id, 1u64);

    let sub = client.get_sub(&id);
    assert_eq!(sub.owner, owner);
    assert_eq!(sub.watched_contract, watched);
    assert!(sub.active);
    assert_eq!(sub.expires_at, 0u32);
    assert_eq!(sub.channel, Channel::Webhook);
}

#[test]
fn test_subscribe_with_ttl() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let topics: Vec<Bytes> = Vec::new(&env);

    let start_ledger = env.ledger().sequence();
    let ttl: u32 = 5_000;
    let id = client.subscribe(
        &owner,
        &watched,
        &topics,
        &Channel::Webhook,
        &ep(&env),
        &ttl,
    );
    let sub = client.get_sub(&id);
    assert_eq!(sub.expires_at, start_ledger + ttl);
}

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

#[test]
fn test_cancel_removes_sub() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.cancel(&owner, &id);
    assert_eq!(client.try_get_sub(&id), Err(Ok(NotifyError::SubNotFound)));
}

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

#[test]
fn test_cancel_nonexistent_returns_not_found() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    assert_eq!(
        client.try_cancel(&owner, &999u64),
        Err(Ok(NotifyError::SubNotFound))
    );
}

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

#[test]
fn test_pause_idempotent() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.pause_sub(&owner, &id);
    client.pause_sub(&owner, &id);
    assert!(!client.get_sub(&id).active);
}

#[test]
fn test_resume_idempotent() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.pause_sub(&owner, &id);
    client.resume_sub(&owner, &id);
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
}

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

#[test]
fn test_pause_nonexistent_returns_not_found() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    assert_eq!(
        client.try_pause_sub(&owner, &999u64),
        Err(Ok(NotifyError::SubNotFound))
    );
}

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

    make_sub(&env, &client, &owner, &watched);
    make_sub(&env, &client, &owner, &watched);

    let result = client.try_subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &0u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::LimitExceeded)));
}

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

    let id1 = make_sub(&env, &client, &owner, &watched);
    make_sub(&env, &client, &owner, &watched);

    assert_eq!(
        client.try_subscribe(
            &owner,
            &watched,
            &Vec::new(&env),
            &Channel::Webhook,
            &ep(&env),
            &0u32,
        ),
        Err(Ok(NotifyError::LimitExceeded))
    );

    client.cancel(&owner, &id1);

    let id3 = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &0u32,
    );
    assert!(id3 > 0u64);
}

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

    make_sub(&env, &client, &owner_a, &watched);
    make_sub(&env, &client, &owner_b, &watched);

    assert_eq!(
        client.try_subscribe(
            &owner_a,
            &watched,
            &Vec::new(&env),
            &Channel::Webhook,
            &ep(&env),
            &0u32,
        ),
        Err(Ok(NotifyError::LimitExceeded))
    );
    assert_eq!(
        client.try_subscribe(
            &owner_b,
            &watched,
            &Vec::new(&env),
            &Channel::Webhook,
            &ep(&env),
            &0u32,
        ),
        Err(Ok(NotifyError::LimitExceeded))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C28 — test_protocol_pause_blocks_subscribe
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_pause_blocks_subscribe() {
    let (env, admin, client) = setup();
    client.set_paused(&admin, &true);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let result = client.try_subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &0u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::Paused)));
}

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

#[test]
fn test_set_paused_idempotent() {
    let (_, admin, client) = setup();
    client.set_paused(&admin, &true);
    client.set_paused(&admin, &true);
    assert!(client.get_config().paused);
}

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

#[test]
fn test_resume_after_expiry_returns_expired() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &10u32,
    );
    client.pause_sub(&owner, &id);
    env.ledger().with_mut(|l| l.sequence_number += 20);
    assert_eq!(
        client.try_resume_sub(&owner, &id),
        Err(Ok(NotifyError::Expired))
    );
}

#[test]
fn test_resume_at_exact_expiry_returns_expired() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let ttl: u32 = 10;
    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &ttl,
    );
    client.pause_sub(&owner, &id);
    env.ledger().with_mut(|l| l.sequence_number += ttl);
    assert_eq!(
        client.try_resume_sub(&owner, &id),
        Err(Ok(NotifyError::Expired))
    );
}

#[test]
fn test_resume_one_ledger_before_expiry_succeeds() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let ttl: u32 = 10;
    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &ttl,
    );
    client.pause_sub(&owner, &id);
    env.ledger().with_mut(|l| l.sequence_number += ttl - 1);
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);
}

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

#[test]
fn test_list_by_owner_empty_for_unknown_address() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    assert_eq!(client.list_by_owner(&nobody).len(), 0u32);
}

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

#[test]
fn test_list_by_contract_empty_for_unknown_contract() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    assert_eq!(client.list_by_contract(&nobody).len(), 0u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// C31 — test_unauthorised_cancel
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unauthorised_cancel() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert_eq!(
        client.try_cancel(&attacker, &id),
        Err(Ok(NotifyError::NotOwner))
    );
}

#[test]
fn test_unauthorised_pause() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    assert_eq!(
        client.try_pause_sub(&attacker, &id),
        Err(Ok(NotifyError::NotOwner))
    );
}

#[test]
fn test_unauthorised_resume() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    client.pause_sub(&owner, &id);
    assert_eq!(
        client.try_resume_sub(&attacker, &id),
        Err(Ok(NotifyError::NotOwner))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C32 — test_too_many_topics
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_ten_topics_succeeds() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let mut topics: Vec<Bytes> = Vec::new(&env);
    for _ in 0..10 {
        topics.push_back(Bytes::from_slice(&env, b"topic"));
    }

    let id = client.subscribe(
        &owner,
        &watched,
        &topics,
        &Channel::Webhook,
        &ep(&env),
        &0u32,
    );
    assert!(id > 0u64);
}

#[test]
fn test_too_many_topics() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let mut topics: Vec<Bytes> = Vec::new(&env);
    for _ in 0..11 {
        topics.push_back(Bytes::from_slice(&env, b"topic"));
    }

    let result = client.try_subscribe(
        &owner,
        &watched,
        &topics,
        &Channel::Webhook,
        &ep(&env),
        &0u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::TooManyTopics)));
}

// ─────────────────────────────────────────────────────────────────────────────
// C33 — test_empty_endpoint
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_empty_endpoint() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);
    let empty_ep = Bytes::from_slice(&env, b"");

    let result = client.try_subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &empty_ep,
        &0u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::EmptyEndpoint)));
}

#[test]
fn test_update_endpoint_empty_returns_error() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let empty_ep = Bytes::from_slice(&env, b"");
    assert_eq!(
        client.try_update_endpoint_ref(&owner, &id, &empty_ep),
        Err(Ok(NotifyError::EmptyEndpoint))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C34 — test_ttl_exceeded
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_ttl_exceeded() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, StellarNotifyContract);
    let client = StellarNotifyContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialise(&admin, &20u32, &1_000u32);

    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let result = client.try_subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &1_001u32,
    );
    assert_eq!(result, Err(Ok(NotifyError::TtlExceeded)));
}

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

    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &1_000u32,
    );
    assert!(id > 0u64);
}

#[test]
fn test_ttl_no_cap_allows_any_ttl() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &99_999u32,
    );
    assert!(id > 0u64);
}

// ─────────────────────────────────────────────────────────────────────────────
// C35 — test_transfer_admin
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_transfer_admin() {
    let (env, admin, client) = setup();
    let new_admin = Address::generate(&env);

    client.transfer_admin(&admin, &new_admin);
    assert_eq!(client.get_config().admin, new_admin);

    assert_eq!(
        client.try_update_config(&admin, &20u32, &100_000u32),
        Err(Ok(NotifyError::Unauthorised))
    );

    client.update_config(&new_admin, &25u32, &200_000u32);
    assert_eq!(client.get_config().max_per_owner, 25u32);
}

#[test]
fn test_transfer_admin_unauthorised() {
    let (env, _admin, client) = setup();
    let attacker = Address::generate(&env);
    let new_admin = Address::generate(&env);
    assert_eq!(
        client.try_transfer_admin(&attacker, &new_admin),
        Err(Ok(NotifyError::Unauthorised))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C36 — test_set_paused
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_set_paused_toggle() {
    let (_, admin, client) = setup();
    client.set_paused(&admin, &true);
    assert!(client.get_config().paused);
    client.set_paused(&admin, &false);
    assert!(!client.get_config().paused);
}

#[test]
fn test_set_paused_twice_idempotent() {
    let (_, admin, client) = setup();
    client.set_paused(&admin, &true);
    client.set_paused(&admin, &true);
    assert!(client.get_config().paused);
}

#[test]
fn test_set_unpaused_idempotent() {
    let (_, admin, client) = setup();
    client.set_paused(&admin, &false);
    assert!(!client.get_config().paused);
}

#[test]
fn test_update_config_unauthorised() {
    let (env, _admin, client) = setup();
    let attacker = Address::generate(&env);
    assert_eq!(
        client.try_update_config(&attacker, &5u32, &500u32),
        Err(Ok(NotifyError::Unauthorised))
    );
}

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

#[test]
fn test_update_endpoint_ref() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let new_ep = Bytes::from_slice(
        &env,
        b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    client.update_endpoint_ref(&owner, &id, &new_ep);
    assert_eq!(client.get_sub(&id).endpoint_ref, new_ep);
}

#[test]
fn test_update_endpoint_ref_not_owner() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let new_ep = Bytes::from_slice(
        &env,
        b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert_eq!(
        client.try_update_endpoint_ref(&attacker, &id, &new_ep),
        Err(Ok(NotifyError::NotOwner))
    );
}

#[test]
fn test_update_endpoint_ref_not_found() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let new_ep = Bytes::from_slice(
        &env,
        b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    assert_eq!(
        client.try_update_endpoint_ref(&owner, &999u64, &new_ep),
        Err(Ok(NotifyError::SubNotFound))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C38 — test_renew_sub
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_renew_extends_from_current_expiry() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let initial_ttl: u32 = 100;
    let start = env.ledger().sequence();
    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &initial_ttl,
    );

    env.ledger().with_mut(|l| l.sequence_number += 50);
    client.renew_sub(&owner, &id, &200u32);
    assert_eq!(client.get_sub(&id).expires_at, start + initial_ttl + 200);
}

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

    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &500u32,
    );
    assert_eq!(
        client.try_renew_sub(&owner, &id, &600u32),
        Err(Ok(NotifyError::TtlExceeded))
    );
}

#[test]
fn test_renew_sub_not_owner() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let attacker = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &100u32,
    );
    assert_eq!(
        client.try_renew_sub(&attacker, &id, &100u32),
        Err(Ok(NotifyError::NotOwner))
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// C39 — test_list_summaries_by_owner
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_list_summaries_by_owner() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id = make_sub(&env, &client, &owner, &watched);
    let sub = client.get_sub(&id);
    let summaries = client.list_summaries_by_owner(&owner);

    assert_eq!(summaries.len(), 1u32);
    let s = summaries.get(0).unwrap();
    assert_eq!(s.id, id);
    assert_eq!(s.owner, sub.owner);
    assert_eq!(s.watched_contract, sub.watched_contract);
    assert_eq!(s.active, sub.active);
    assert_eq!(s.channel, sub.channel);
    assert_eq!(s.expires_at, sub.expires_at);
}

#[test]
fn test_list_summaries_multiple() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id1 = make_sub(&env, &client, &owner, &watched);
    let id2 = make_sub(&env, &client, &owner, &watched);
    let id3 = make_sub(&env, &client, &owner, &watched);

    let summaries = client.list_summaries_by_owner(&owner);
    assert_eq!(summaries.len(), 3u32);
    assert_eq!(summaries.get(0).unwrap().id, id1);
    assert_eq!(summaries.get(1).unwrap().id, id2);
    assert_eq!(summaries.get(2).unwrap().id, id3);
}

#[test]
fn test_list_summaries_empty_for_unknown_address() {
    let (env, _admin, client) = setup();
    let nobody = Address::generate(&env);
    assert_eq!(client.list_summaries_by_owner(&nobody).len(), 0u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// C40 — integration test — full lifecycle
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_full_lifecycle() {
    let (env, admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    // 1. Subscribe
    let id = client.subscribe(
        &owner,
        &watched,
        &Vec::new(&env),
        &Channel::Webhook,
        &ep(&env),
        &1_000u32,
    );
    assert_eq!(id, 1u64);
    assert!(client.get_sub(&id).active);

    // 2. Verify indexes
    assert_eq!(client.list_by_owner(&owner).len(), 1u32);
    assert_eq!(client.list_by_contract(&watched).len(), 1u32);

    // 3. Pause
    client.pause_sub(&owner, &id);
    assert!(!client.get_sub(&id).active);

    // 4. Resume
    client.resume_sub(&owner, &id);
    assert!(client.get_sub(&id).active);

    // 5. Update endpoint
    let new_ep = Bytes::from_slice(
        &env,
        b"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    client.update_endpoint_ref(&owner, &id, &new_ep);
    assert_eq!(client.get_sub(&id).endpoint_ref, new_ep);

    // 6. Renew TTL
    client.renew_sub(&owner, &id, &500u32);
    assert!(client.get_sub(&id).expires_at > 0u32);

    // 7. Admin pauses protocol — existing sub unaffected
    client.set_paused(&admin, &true);
    assert!(client.get_sub(&id).active);

    // 8. Cancel
    client.cancel(&owner, &id);
    assert_eq!(client.try_get_sub(&id), Err(Ok(NotifyError::SubNotFound)));
    assert_eq!(client.list_by_owner(&owner).len(), 0u32);
    assert_eq!(client.list_by_contract(&watched).len(), 0u32);
}

// ─────────────────────────────────────────────────────────────────────────────
// C41 — fix: remove_from_index handles duplicates safely
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_cancel_does_not_corrupt_index() {
    let (env, _admin, client) = setup();
    let owner = Address::generate(&env);
    let watched = Address::generate(&env);

    let id1 = make_sub(&env, &client, &owner, &watched);
    let id2 = make_sub(&env, &client, &owner, &watched);
    let id3 = make_sub(&env, &client, &owner, &watched);

    client.cancel(&owner, &id2);

    let owner_ids = client.list_by_owner(&owner);
    assert_eq!(owner_ids.len(), 2u32);
    assert!(owner_ids.contains(&id1));
    assert!(!owner_ids.contains(&id2));
    assert!(owner_ids.contains(&id3));

    let watcher_ids = client.list_by_contract(&watched);
    assert_eq!(watcher_ids.len(), 2u32);
    assert!(watcher_ids.contains(&id1));
    assert!(!watcher_ids.contains(&id2));
    assert!(watcher_ids.contains(&id3));
}
