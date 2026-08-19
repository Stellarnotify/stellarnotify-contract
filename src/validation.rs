use soroban_sdk::{Address, Bytes, Env, Vec};

use crate::errors::NotifyError;
use crate::storage;
use crate::types::ProtocolConfig;

/// Maximum number of topic filters allowed per subscription.
pub const MAX_TOPICS: u32 = 10;

/// Validate all `subscribe()` inputs before any storage is touched.
///
/// Checks performed in order:
/// 1. Protocol is not paused.
/// 2. Topics list does not exceed `MAX_TOPICS`.
/// 3. `endpoint_ref` is not empty.
/// 4. Requested TTL does not exceed `config.max_ttl` (when cap is set).
/// 5. Owner has not reached their `config.max_per_owner` limit.
///
/// # Errors
/// `Paused` | `TooManyTopics` | `EmptyEndpoint` | `TtlExceeded` | `LimitExceeded`
pub fn validate_subscribe(
    env: &Env,
    config: &ProtocolConfig,
    owner: &Address,
    topics: &Vec<Bytes>,
    endpoint_ref: &Bytes,
    ttl_ledgers: u32,
) -> Result<(), NotifyError> {
    if config.paused {
        return Err(NotifyError::Paused);
    }
    if topics.len() > MAX_TOPICS {
        return Err(NotifyError::TooManyTopics);
    }
    if endpoint_ref.is_empty() {
        return Err(NotifyError::EmptyEndpoint);
    }
    if config.max_ttl > 0 && ttl_ledgers > config.max_ttl {
        return Err(NotifyError::TtlExceeded);
    }
    if storage::owner_sub_count(env, owner) >= config.max_per_owner {
        return Err(NotifyError::LimitExceeded);
    }
    Ok(())
}
