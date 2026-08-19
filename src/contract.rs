use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

use crate::admin;
use crate::errors::NotifyError;
use crate::storage;
use crate::subscribe as sub_mod;
use crate::types::{Channel, ProtocolConfig, Subscription, SubscriptionSummary};

#[contract]
pub struct StellarNotifyContract;

#[contractimpl]
impl StellarNotifyContract {
    // ─────────────────────────────────────────────────────────────────────
    // Initialisation
    // ─────────────────────────────────────────────────────────────────────

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

    pub fn subscribe(
        env: Env,
        owner: Address,
        watched_contract: Address,
        topics: Vec<Bytes>,
        channel: Channel,
        endpoint_ref: Bytes,
        ttl_ledgers: u32,
    ) -> Result<u64, NotifyError> {
        sub_mod::subscribe(env, owner, watched_contract, topics, channel, endpoint_ref, ttl_ledgers)
    }

    pub fn cancel(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
        sub_mod::cancel(env, owner, id)
    }

    pub fn pause_sub(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
        sub_mod::pause_sub(env, owner, id)
    }

    pub fn resume_sub(env: Env, owner: Address, id: u64) -> Result<(), NotifyError> {
        sub_mod::resume_sub(env, owner, id)
    }

    pub fn update_endpoint_ref(
        env: Env,
        owner: Address,
        id: u64,
        new_endpoint: Bytes,
    ) -> Result<(), NotifyError> {
        sub_mod::update_endpoint_ref(env, owner, id, new_endpoint)
    }

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

    /// Full subscription data by ID.
    pub fn get_sub(env: Env, id: u64) -> Result<Subscription, NotifyError> {
        storage::get_sub(&env, id)
    }

    /// All subscription IDs owned by a wallet.
    pub fn list_by_owner(env: Env, owner: Address) -> Vec<u64> {
        storage::get_owner_subs(&env, &owner)
    }

    /// All subscription IDs watching a given contract.
    pub fn list_by_contract(env: Env, watched: Address) -> Vec<u64> {
        storage::get_watcher_subs(&env, &watched)
    }

    /// Lightweight summaries for a wallet — omits heavy topics/endpoint fields.
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

    /// Current protocol configuration.
    pub fn get_config(env: Env) -> Result<ProtocolConfig, NotifyError> {
        storage::get_config(&env)
    }

    /// Contract version string.
    pub fn get_version(_env: Env) -> &'static str {
        "0.1.0"
    }

    // ─────────────────────────────────────────────────────────────────────
    // Admin
    // ─────────────────────────────────────────────────────────────────────

    pub fn update_config(
        env: Env,
        admin: Address,
        max_per_owner: u32,
        max_ttl: u32,
    ) -> Result<(), NotifyError> {
        admin::update_config(env, admin, max_per_owner, max_ttl)
    }

    pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), NotifyError> {
        admin::set_paused(env, admin, paused)
    }

    pub fn transfer_admin(
        env: Env,
        admin: Address,
        new_admin: Address,
    ) -> Result<(), NotifyError> {
        admin::transfer_admin(env, admin, new_admin)
    }
}
