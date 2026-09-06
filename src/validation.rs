use soroban_sdk::{Address, Bytes, Env, Vec};

use crate::errors::NotifyError;
use crate::storage;
use crate::types::{BatchSubscribeParams, ProtocolConfig};

/// Maximum number of topic filters allowed per subscription.
pub const MAX_TOPICS: u32 = 10;

/// Maximum number of subscriptions allowed in a single batch operation.
pub const MAX_BATCH_SIZE: u32 = 10;

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

/// Validate batch subscribe parameters.
///
/// Performs all-or-nothing validation:
/// 1. Batch size does not exceed `MAX_BATCH_SIZE`.
/// 2. Protocol is not paused.
/// 3. All subscriptions in the batch pass individual validation.
/// 4. Total subscriptions (current + batch) does not exceed `max_per_owner`.
///
/// # Errors
/// `Paused` | `TooManyTopics` | `EmptyEndpoint` | `TtlExceeded` | `LimitExceeded`
pub fn validate_batch_subscribe(
    env: &Env,
    config: &ProtocolConfig,
    owner: &Address,
    params_list: &Vec<BatchSubscribeParams>,
) -> Result<(), NotifyError> {
    if config.paused {
        return Err(NotifyError::Paused);
    }

    let batch_size = params_list.len();
    if batch_size == 0 || batch_size > MAX_BATCH_SIZE {
        return Err(NotifyError::LimitExceeded);
    }

    let current_count = storage::owner_sub_count(env, owner);
    if current_count + batch_size > config.max_per_owner {
        return Err(NotifyError::LimitExceeded);
    }

    // Validate each subscription in the batch
    for params in params_list.iter() {
        if params.topics.len() > MAX_TOPICS {
            return Err(NotifyError::TooManyTopics);
        }
        if params.endpoint_ref.is_empty() {
            return Err(NotifyError::EmptyEndpoint);
        }
        if config.max_ttl > 0 && params.ttl_ledgers > config.max_ttl {
            return Err(NotifyError::TtlExceeded);
        }
    }

    Ok(())
}
