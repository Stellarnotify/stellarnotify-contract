//! Subscription management functions.
//!
//! All functions are owner-facing — they require the subscription
//! owner's signature on every call.

use soroban_sdk::{Address, Bytes, Env, Vec};

use crate::errors::NotifyError;
use crate::events;
use crate::storage;
use crate::types::{Channel, Subscription};
use crate::validation;

/// Create a new subscription.
///
/// # Parameters
/// - `owner`            — wallet that owns and controls this subscription.
/// - `watched_contract` — Soroban contract whose emitted events will trigger notifications.
/// - `topics`           — optional topic filter list. Empty = all events. Max 10 entries.
/// - `channel`          — delivery channel: Webhook, InApp, or OnChain.
/// - `endpoint_ref`     — SHA-256 hex hash of the webhook URL (non-empty).
/// - `ttl_ledgers`      — ledgers until expiry. 0 = no expiry.
///
/// # Returns
/// The new subscription's unique `u64` ID starting at 1.
///
/// # Errors
/// `NotInitialised` | `Paused` | `TooManyTopics` | `EmptyEndpoint` |
/// `TtlExceeded` | `LimitExceeded`
pub fn subscribe(
    env: Env,
    owner: Address,
    watched_contract: Address,
    topics: Vec<Bytes>,
    channel: Channel,
    endpoint_ref: Bytes,
    ttl_ledgers: u32,
) -> Result<u64, NotifyError> {
    owner.require_auth();
    let config = storage::get_config(&env)?;

    validation::validate_subscribe(&env, &config, &owner, &topics, &endpoint_ref, ttl_ledgers)?;

    let id = storage::next_id(&env);
    let expires_at: u32 = if ttl_ledgers == 0 {
        0
    } else {
        env.ledger().sequence() + ttl_ledgers
    };

    let sub = Subscription {
        owner: owner.clone(),
        watched_contract: watched_contract.clone(),
        topics,
        channel: channel.clone(),
        endpoint_ref,
        active: true,
        created_at: env.ledger().sequence(),
        expires_at,
    };

    storage::save_sub(&env, id, &sub);
    storage::add_to_owner_index(&env, &owner, id);
    storage::add_to_watcher_index(&env, &watched_contract, id);
    events::sub_created(&env, id, &owner, &watched_contract);

    if channel == Channel::OnChain {
        events::onchain_sub_activated(&env, id, &owner, &watched_contract);
    }

    Ok(id)
}

/// Permanently cancel a subscription and remove it from all storage.
///
/// # Errors
/// - [`NotifyError::SubNotFound`] — no subscription exists with this ID.
/// - [`NotifyError::NotOwner`]    — caller is not the subscription owner.
pub fn cancel(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
    owner.require_auth();
    let sub = storage::get_sub(&env, id)?;
    if sub.owner != owner {
        return Err(NotifyError::NotOwner);
    }
    storage::remove_from_owner_index(&env, &owner, id);
    storage::remove_from_watcher_index(&env, &sub.watched_contract, id);
    storage::remove_sub(&env, id);
    events::sub_cancelled(&env, id, &owner);
    Ok(())
}

/// Pause a subscription — keep all data but stop notification delivery.
///
/// Idempotent — pausing an already-paused subscription returns `Ok(())`.
///
/// # Errors
/// - [`NotifyError::SubNotFound`] — no subscription exists with this ID.
/// - [`NotifyError::NotOwner`]    — caller is not the subscription owner.
pub fn pause_sub(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
    owner.require_auth();
    let mut sub = storage::get_sub(&env, id)?;
    if sub.owner != owner {
        return Err(NotifyError::NotOwner);
    }
    if sub.active {
        sub.active = false;
        storage::save_sub(&env, id, &sub);
        events::sub_paused(&env, id, &owner);
    }
    Ok(())
}

/// Resume a paused subscription — re-enable notification delivery.
///
/// Idempotent — resuming an already-active subscription returns `Ok(())`.
///
/// # Errors
/// - [`NotifyError::SubNotFound`] — no subscription exists with this ID.
/// - [`NotifyError::NotOwner`]    — caller is not the subscription owner.
/// - [`NotifyError::Expired`]     — subscription TTL has passed.
pub fn resume_sub(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
    owner.require_auth();
    let mut sub = storage::get_sub(&env, id)?;
    if sub.owner != owner {
        return Err(NotifyError::NotOwner);
    }
    if sub.expires_at > 0 && env.ledger().sequence() >= sub.expires_at {
        return Err(NotifyError::Expired);
    }
    if !sub.active {
        sub.active = true;
        storage::save_sub(&env, id, &sub);
        events::sub_resumed(&env, id, &owner);
    }
    Ok(())
}

/// Rotate the endpoint reference hash on an existing subscription.
///
/// # Errors
/// - [`NotifyError::SubNotFound`]   — no subscription exists with this ID.
/// - [`NotifyError::NotOwner`]      — caller is not the subscription owner.
/// - [`NotifyError::EmptyEndpoint`] — `new_endpoint` is zero-length.
pub fn update_endpoint_ref(
    env: Env,
    owner: Address,
    id: u64,
    new_endpoint: Bytes,
) -> Result<(), NotifyError> {
    owner.require_auth();
    let mut sub = storage::get_sub(&env, id)?;
    if sub.owner != owner {
        return Err(NotifyError::NotOwner);
    }
    if new_endpoint.is_empty() {
        return Err(NotifyError::EmptyEndpoint);
    }
    sub.endpoint_ref = new_endpoint;
    storage::save_sub(&env, id, &sub);
    events::sub_endpoint_updated(&env, id, &owner);
    Ok(())
}

/// Extend the TTL of an existing subscription without cancelling it.
///
/// - Still live: new expiry = `expires_at + add_ttl_ledgers`.
/// - Already expired: new expiry = `current_ledger + add_ttl_ledgers`.
/// - Permanent (`expires_at == 0`): no-op, returns `Ok(())`.
///
/// # Errors
/// - [`NotifyError::SubNotFound`] — no subscription exists with this ID.
/// - [`NotifyError::NotOwner`]    — caller is not the subscription owner.
/// - [`NotifyError::TtlExceeded`] — renewal would exceed `max_ttl`.
pub fn renew_sub(
    env: Env,
    owner: Address,
    id: u64,
    add_ttl_ledgers: u32,
) -> Result<(), NotifyError> {
    owner.require_auth();
    let mut sub = storage::get_sub(&env, id)?;
    if sub.owner != owner {
        return Err(NotifyError::NotOwner);
    }
    if sub.expires_at == 0 {
        return Ok(());
    }
    let config = storage::get_config(&env)?;
    let current = env.ledger().sequence();
    let base: u32 = if sub.expires_at > current {
        sub.expires_at
    } else {
        current
    };
    let new_expires_at = base + add_ttl_ledgers;
    if config.max_ttl > 0 && new_expires_at > current + config.max_ttl {
        return Err(NotifyError::TtlExceeded);
    }
    sub.expires_at = new_expires_at;
    storage::save_sub(&env, id, &sub);
    events::sub_renewed(&env, id, &owner, new_expires_at);
    Ok(())
}
