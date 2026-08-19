use soroban_sdk::{contract, contractimpl, Env, Address};

use crate::errors::NotifyError;
use crate::types::ProtocolConfig;
use crate::storage;
use crate::admin;

#[contract]
pub struct StellarNotifyContract;

#[contractimpl]
impl StellarNotifyContract {
    /// One-time registry setup. Must be called once after deployment.
    pub fn initialise(
        env: Env,
        admin: Address,
        max_per_owner: u32,
        max_ttl: u32,
    ) -> Result<(), NotifyError> {
        admin::initialise(env, admin, max_per_owner, max_ttl)
    }

    /// Current protocol configuration.
    pub fn get_config(env: Env) -> Result<ProtocolConfig, NotifyError> {
        storage::get_config(&env)
    }
}
