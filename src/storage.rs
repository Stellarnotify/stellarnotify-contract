//! Storage helpers for the StellarNotify contract.
//!
//! # TTL Strategy
//!
//! Every storage read AND write calls `extend_ttl` to prevent subscriptions
//! from being silently archived (C21).
//!
//! - `LEDGERS_TO_LIVE` = 3,110,400 ≈ 180 days at ~5 seconds per ledger.
//!
//! Business logic is split across `subscribe.rs` (owner functions) and
//! `admin.rs` (admin functions). `contract.rs` is a thin delegation layer (C22).

use soroban_sdk::{Address, Env, Vec};

use crate::datakey::DataKey;
use crate::errors::NotifyError;
use crate::types::{ProtocolConfig, Subscription};

const LEDGERS_TO_LIVE: u32 = 3_110_400;
const INSTANCE_LEDGERS_TO_LIVE: u32 = 3_110_400;

// ─────────────────────────────────────────────────────────────────────────────
// Subscription ID counter
// ─────────────────────────────────────────────────────────────────────────────

/// Read the current counter, increment it, persist it, and return the new ID.
/// IDs start at 1.
pub fn next_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::SubCounter)
        .unwrap_or(0u64);
    let next = current + 1;
    env.storage().instance().set(&DataKey::SubCounter, &next);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LEDGERS_TO_LIVE, INSTANCE_LEDGERS_TO_LIVE);
    next
}

// ─────────────────────────────────────────────────────────────────────────────
// Individual subscription CRUD
// ─────────────────────────────────────────────────────────────────────────────

/// Persist a subscription and extend its persistent TTL.
pub fn save_sub(env: &Env, id: u64, sub: &Subscription) {
    env.storage().persistent().set(&DataKey::Sub(id), sub);
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Sub(id), LEDGERS_TO_LIVE, LEDGERS_TO_LIVE);
}

/// Load a subscription by ID. Returns `SubNotFound` if missing or archived.
/// Bumps persistent TTL on every successful read.
pub fn get_sub(env: &Env, id: u64) -> Result<Subscription, NotifyError> {
    let sub = env
        .storage()
        .persistent()
        .get(&DataKey::Sub(id))
        .ok_or(NotifyError::SubNotFound)?;
    env.storage()
        .persistent()
        .extend_ttl(&DataKey::Sub(id), LEDGERS_TO_LIVE, LEDGERS_TO_LIVE);
    Ok(sub)
}

/// Remove a subscription entry from persistent storage.
pub fn remove_sub(env: &Env, id: u64) {
    env.storage().persistent().remove(&DataKey::Sub(id));
}

// ─────────────────────────────────────────────────────────────────────────────
// Owner index  (Address → Vec<u64>)
// ─────────────────────────────────────────────────────────────────────────────

/// Return the number of subscription IDs owned by `owner`.
pub fn owner_sub_count(env: &Env, owner: &Address) -> u32 {
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerSubs(owner.clone()))
        .unwrap_or(Vec::new(env));
    if !ids.is_empty() {
        env.storage().persistent().extend_ttl(
            &DataKey::OwnerSubs(owner.clone()),
            LEDGERS_TO_LIVE,
            LEDGERS_TO_LIVE,
        );
    }
    ids.len()
}

/// Return all subscription IDs owned by `owner`.
pub fn get_owner_subs(env: &Env, owner: &Address) -> Vec<u64> {
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerSubs(owner.clone()))
        .unwrap_or(Vec::new(env));
    if !ids.is_empty() {
        env.storage().persistent().extend_ttl(
            &DataKey::OwnerSubs(owner.clone()),
            LEDGERS_TO_LIVE,
            LEDGERS_TO_LIVE,
        );
    }
    ids
}

/// Append `id` to the owner's subscription index.
pub fn add_to_owner_index(env: &Env, owner: &Address, id: u64) {
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerSubs(owner.clone()))
        .unwrap_or(Vec::new(env));
    ids.push_back(id);
    env.storage()
        .persistent()
        .set(&DataKey::OwnerSubs(owner.clone()), &ids);
    env.storage().persistent().extend_ttl(
        &DataKey::OwnerSubs(owner.clone()),
        LEDGERS_TO_LIVE,
        LEDGERS_TO_LIVE,
    );
}

/// Remove `id` from the owner's subscription index.
/// Rebuilds the Vec excluding ALL occurrences of `id` (defensive deduplication).
pub fn remove_from_owner_index(env: &Env, owner: &Address, id: u64) {
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::OwnerSubs(owner.clone()))
        .unwrap_or(Vec::new(env));
    let mut new_ids: Vec<u64> = Vec::new(env);
    for existing in ids.iter() {
        if existing != id {
            new_ids.push_back(existing);
        }
    }
    env.storage()
        .persistent()
        .set(&DataKey::OwnerSubs(owner.clone()), &new_ids);
    env.storage().persistent().extend_ttl(
        &DataKey::OwnerSubs(owner.clone()),
        LEDGERS_TO_LIVE,
        LEDGERS_TO_LIVE,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Watcher index  (Address → Vec<u64>)
// ─────────────────────────────────────────────────────────────────────────────

/// Return all subscription IDs watching `watched`.
pub fn get_watcher_subs(env: &Env, watched: &Address) -> Vec<u64> {
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::WatcherSubs(watched.clone()))
        .unwrap_or(Vec::new(env));
    if !ids.is_empty() {
        env.storage().persistent().extend_ttl(
            &DataKey::WatcherSubs(watched.clone()),
            LEDGERS_TO_LIVE,
            LEDGERS_TO_LIVE,
        );
    }
    ids
}

/// Append `id` to the watcher index for `watched`.
pub fn add_to_watcher_index(env: &Env, watched: &Address, id: u64) {
    let mut ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::WatcherSubs(watched.clone()))
        .unwrap_or(Vec::new(env));
    ids.push_back(id);
    env.storage()
        .persistent()
        .set(&DataKey::WatcherSubs(watched.clone()), &ids);
    env.storage().persistent().extend_ttl(
        &DataKey::WatcherSubs(watched.clone()),
        LEDGERS_TO_LIVE,
        LEDGERS_TO_LIVE,
    );
}

/// Remove `id` from the watcher index for `watched`.
/// Rebuilds the Vec excluding ALL occurrences of `id` (defensive deduplication).
pub fn remove_from_watcher_index(env: &Env, watched: &Address, id: u64) {
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::WatcherSubs(watched.clone()))
        .unwrap_or(Vec::new(env));
    let mut new_ids: Vec<u64> = Vec::new(env);
    for existing in ids.iter() {
        if existing != id {
            new_ids.push_back(existing);
        }
    }
    env.storage()
        .persistent()
        .set(&DataKey::WatcherSubs(watched.clone()), &new_ids);
    env.storage().persistent().extend_ttl(
        &DataKey::WatcherSubs(watched.clone()),
        LEDGERS_TO_LIVE,
        LEDGERS_TO_LIVE,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Subscription summary helper
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a full `Subscription` into a lightweight `SubscriptionSummary`.
pub fn to_summary(id: u64, sub: &crate::types::Subscription) -> crate::types::SubscriptionSummary {
    crate::types::SubscriptionSummary {
        id,
        owner: sub.owner.clone(),
        watched_contract: sub.watched_contract.clone(),
        active: sub.active,
        channel: sub.channel.clone(),
        expires_at: sub.expires_at,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Protocol config
// ─────────────────────────────────────────────────────────────────────────────

/// Persist the protocol config and bump instance TTL.
pub fn save_config(env: &Env, config: &ProtocolConfig) {
    env.storage().instance().set(&DataKey::Config, config);
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LEDGERS_TO_LIVE, INSTANCE_LEDGERS_TO_LIVE);
}

/// Load the protocol config. Returns `NotInitialised` if not found.
pub fn get_config(env: &Env) -> Result<ProtocolConfig, NotifyError> {
    let config = env
        .storage()
        .instance()
        .get(&DataKey::Config)
        .ok_or(NotifyError::NotInitialised)?;
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LEDGERS_TO_LIVE, INSTANCE_LEDGERS_TO_LIVE);
    Ok(config)
}
