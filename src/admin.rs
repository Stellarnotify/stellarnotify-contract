//! Admin-only functions.
//!
//! All functions require the admin signature stored in `ProtocolConfig`.
//! Every function applies a double-auth check:
//! 1. `admin.require_auth()` — Soroban verifies the signature is present.
//! 2. `config.admin != admin` — verifies the signing address matches stored admin.

use soroban_sdk::{Address, Env};

use crate::errors::NotifyError;
use crate::events;
use crate::storage;
use crate::types::ProtocolConfig;

/// Initialise the StellarNotify registry.
///
/// Must be called exactly once immediately after deployment.
///
/// # Errors
/// - [`NotifyError::AlreadyInitialised`] — contract has already been set up.
pub fn initialise(
    env: Env,
    admin: Address,
    max_per_owner: u32,
    max_ttl: u32,
) -> Result<(), NotifyError> {
    if storage::get_config(&env).is_ok() {
        return Err(NotifyError::AlreadyInitialised);
    }
    admin.require_auth();
    let config = ProtocolConfig {
        max_per_owner,
        max_ttl,
        admin: admin.clone(),
        paused: false,
    };
    storage::save_config(&env, &config);
    events::config_updated(&env, &admin);
    Ok(())
}

/// Update the protocol's per-owner subscription cap and maximum TTL.
///
/// # Errors
/// - [`NotifyError::NotInitialised`] — contract has not been initialised.
/// - [`NotifyError::Unauthorised`]   — caller is not the current admin.
pub fn update_config(
    env: Env,
    admin: Address,
    max_per_owner: u32,
    max_ttl: u32,
) -> Result<(), NotifyError> {
    admin.require_auth();
    let mut config = storage::get_config(&env)?;
    if config.admin != admin {
        return Err(NotifyError::Unauthorised);
    }
    config.max_per_owner = max_per_owner;
    config.max_ttl = max_ttl;
    storage::save_config(&env, &config);
    events::config_updated(&env, &admin);
    Ok(())
}

/// Pause or unpause the entire protocol.
///
/// # Errors
/// - [`NotifyError::NotInitialised`] — contract has not been initialised.
/// - [`NotifyError::Unauthorised`]   — caller is not the current admin.
pub fn set_paused(env: Env, admin: Address, paused: bool) -> Result<(), NotifyError> {
    admin.require_auth();
    let mut config = storage::get_config(&env)?;
    if config.admin != admin {
        return Err(NotifyError::Unauthorised);
    }
    if config.paused != paused {
        config.paused = paused;
        storage::save_config(&env, &config);
        events::protocol_paused(&env, paused);
    }
    Ok(())
}

/// Transfer the admin role to a new address.
///
/// # Errors
/// - [`NotifyError::NotInitialised`] — contract has not been initialised.
/// - [`NotifyError::Unauthorised`]   — caller does not match stored admin.
pub fn transfer_admin(env: Env, admin: Address, new_admin: Address) -> Result<(), NotifyError> {
    admin.require_auth();
    let mut config = storage::get_config(&env)?;
    if config.admin != admin {
        return Err(NotifyError::Unauthorised);
    }
    let old_admin = config.admin.clone();
    config.admin = new_admin.clone();
    storage::save_config(&env, &config);
    events::admin_transferred(&env, &old_admin, &new_admin);
    Ok(())
}
