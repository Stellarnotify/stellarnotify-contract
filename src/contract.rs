use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, Vec};

use crate::admin;
use crate::errors::NotifyError;
use crate::storage;
use crate::subscribe as sub_mod;
use crate::types::{Channel, ProtocolConfig, Subscription};

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

    pub fn get_sub(env: Env, id: u64) -> Result<Subscription, NotifyError> {
        storage::get_sub(&env, id)
    }

    pub fn get_config(env: Env) -> Result<ProtocolConfig, NotifyError> {
        storage::get_config(&env)
    }
}
