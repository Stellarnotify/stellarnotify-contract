//! Public contract interface — thin delegation layer.
//!
//! This file contains only the `#[contract]` struct and the `#[contractimpl]`
//! block. All business logic lives in dedicated modules:
//!
//! | Module       | Responsibility                                        |
//! |--------------|-------------------------------------------------------|
//! | `admin`      | initialise, update_config, set_paused, transfer_admin |
//! | `subscribe`  | subscribe, cancel, pause_sub, resume_sub,             |
//! |              | update_endpoint_ref, renew_sub                        |
//! | `storage`    | raw persistent/instance read-write helpers            |
//! | `validation` | subscribe() input validation                          |
//! | `events`     | Soroban event emission helpers                        |

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

use crate::admin;
use crate::errors::NotifyError;
use crate::storage;
use crate::subscribe as sub_mod;
use crate::types::{Channel, ProtocolConfig, Subscription, SubscriptionSummary};

/// Semantic version of this contract. Updated on each release commit.
const CONTRACT_VERSION: &str = "0.1.0";

#[contract]
pub struct StellarNotifyContract;

#[contractimpl]
impl StellarNotifyContract {
    // ─────────────────────────────────────────────────────────────────────
    // Initialisation
    // ─────────────────────────────────────────────────────────────────────

    /// Initialise the StellarNotify registry.
    ///
    /// Must be called exactly once immediately after deployment. The deployer
    /// must sign this transaction to claim the admin role.
    ///
    /// # Parameters
    /// - `admin`         — address that receives admin privileges.
    /// - `max_per_owner` — maximum active subscriptions per wallet.
    /// - `max_ttl`       — maximum TTL in ledgers. `0` = no cap.
    ///
    /// # Errors
    /// - [`NotifyError::AlreadyInitialised`] — contract already set up.
    pub fn initialise(
        env: Env,
        admin: Address,
        max_per_owner: u32,
        max_ttl: u32,
    ) -> Result<(), NotifyError> {
        admin::initialise(env, admin, max_per_owner, max_ttl)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Subscription management
    // ─────────────────────────────────────────────────────────────────────

    /// Create a new subscription.
    ///
    /// # Parameters
    /// - `owner`            — wallet that owns this subscription (must sign).
    /// - `watched_contract` — Soroban contract whose events trigger notifications.
    /// - `topics`           — topic filter list. Empty = all events. Max 10.
    /// - `channel`          — `Webhook`, `InApp`, or `OnChain`.
    /// - `endpoint_ref`     — SHA-256 hash of the webhook URL (non-empty).
    /// - `ttl_ledgers`      — ledgers until expiry. `0` = no expiry.
    ///
    /// # Returns
    /// Unique `u64` subscription ID starting at `1`.
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
        sub_mod::subscribe(
            env,
            owner,
            watched_contract,
            topics,
            channel,
            endpoint_ref,
            ttl_ledgers,
        )
    }

    /// Permanently cancel a subscription and remove it from all storage.
    ///
    /// # Errors
    /// - [`NotifyError::SubNotFound`] — no subscription with this ID.
    /// - [`NotifyError::NotOwner`]    — caller is not the owner.
    pub fn cancel(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
        sub_mod::cancel(env, owner, id)
    }

    /// Pause a subscription — keep data but stop notification delivery.
    ///
    /// Idempotent. Does not affect the owner's subscription count.
    ///
    /// # Errors
    /// - [`NotifyError::SubNotFound`] — no subscription with this ID.
    /// - [`NotifyError::NotOwner`]    — caller is not the owner.
    pub fn pause_sub(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
        sub_mod::pause_sub(env, owner, id)
    }

    /// Resume a paused subscription — re-enable notification delivery.
    ///
    /// Idempotent. Checks expiry before resuming.
    ///
    /// # Errors
    /// - [`NotifyError::SubNotFound`] — no subscription with this ID.
    /// - [`NotifyError::NotOwner`]    — caller is not the owner.
    /// - [`NotifyError::Expired`]     — subscription TTL has passed.
    pub fn resume_sub(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
        sub_mod::resume_sub(env, owner, id)
    }

    /// Rotate the endpoint reference hash without cancelling the subscription.
    ///
    /// # Errors
    /// - [`NotifyError::SubNotFound`]   — no subscription with this ID.
    /// - [`NotifyError::NotOwner`]      — caller is not the owner.
    /// - [`NotifyError::EmptyEndpoint`] — new endpoint is zero-length.
    pub fn update_endpoint_ref(
        env: Env,
        owner: Address,
        id: u64,
        new_endpoint: Bytes,
    ) -> Result<(), NotifyError> {
        sub_mod::update_endpoint_ref(env, owner, id, new_endpoint)
    }

    /// Extend the TTL of an existing subscription.
    ///
    /// Preserves the subscription ID. No-op for permanent subscriptions.
    ///
    /// # Errors
    /// - [`NotifyError::SubNotFound`] — no subscription with this ID.
    /// - [`NotifyError::NotOwner`]    — caller is not the owner.
    /// - [`NotifyError::TtlExceeded`] — renewal would exceed `max_ttl`.
    pub fn renew_sub(
        env: Env,
        owner: Address,
        id: u64,
        add_ttl_ledgers: u32,
    ) -> Result<(), NotifyError> {
        sub_mod::renew_sub(env, owner, id, add_ttl_ledgers)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Queries
    // ─────────────────────────────────────────────────────────────────────

    /// Return full subscription data by ID.
    ///
    /// # Errors
    /// - [`NotifyError::SubNotFound`] — no subscription with this ID.
    pub fn get_sub(env: Env, id: u64) -> Result<Subscription, NotifyError> {
        storage::get_sub(&env, id)
    }

    /// Return all subscription IDs owned by a wallet.
    pub fn list_by_owner(env: Env, owner: Address) -> Vec<u64> {
        storage::get_owner_subs(&env, &owner)
    }

    /// Return all subscription IDs watching a given contract.
    pub fn list_by_contract(env: Env, watched: Address) -> Vec<u64> {
        storage::get_watcher_subs(&env, &watched)
    }

    /// Return lightweight summaries for all subscriptions owned by a wallet.
    ///
    /// Omits `topics` and `endpoint_ref` to reduce response size.
    pub fn list_summaries_by_owner(env: Env, owner: Address) -> Vec<SubscriptionSummary> {
        let ids = storage::get_owner_subs(&env, &owner);
        let mut summaries: Vec<SubscriptionSummary> = Vec::new(&env);
        for id in ids.iter() {
            if let Ok(sub) = storage::get_sub(&env, id) {
                summaries.push_back(storage::to_summary(id, &sub));
            }
        }
        summaries
    }

    /// Return the current protocol configuration.
    ///
    /// # Errors
    /// - [`NotifyError::NotInitialised`] — contract not yet initialised.
    pub fn get_config(env: Env) -> Result<ProtocolConfig, NotifyError> {
        storage::get_config(&env)
    }

    /// Return the contract version string (e.g. `"0.1.0"`).
    pub fn get_version(_env: Env) -> &'static str {
        CONTRACT_VERSION
    }

    // ─────────────────────────────────────────────────────────────────────
    // Admin
    // ─────────────────────────────────────────────────────────────────────

    /// Update the per-owner subscription cap and maximum TTL.
    ///
    /// # Errors
    /// - [`NotifyError::NotInitialised`] — contract not yet initialised.
    /// - [`NotifyError::Unauthorised`]   — caller is not the admin.
    pub fn update_config(
        env: Env,
        admin: Address,
        max_per_owner: u32,
        max_ttl: u32,
    ) -> Result<(), NotifyError> {
        admin::update_config(env, admin, max_per_owner, max_ttl)
    }

    /// Pause or unpause the entire protocol.
    ///
    /// When paused, `subscribe()` is blocked. Existing subscriptions are unaffected.
    /// Idempotent.
    ///
    /// # Errors
    /// - [`NotifyError::NotInitialised`] — contract not yet initialised.
    /// - [`NotifyError::Unauthorised`]   — caller is not the admin.
    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), NotifyError> {
        admin::set_paused(env, admin, paused)
    }

    /// Transfer the admin role to a new address.
    ///
    /// The current admin must sign. Irreversible without new admin cooperation.
    ///
    /// # Errors
    /// - [`NotifyError::NotInitialised`] — contract not yet initialised.
    /// - [`NotifyError::Unauthorised`]   — caller is not the current admin.
    pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), NotifyError> {
        admin::transfer_admin(env, admin, new_admin)
    }
}
