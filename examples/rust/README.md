# Rust Client Examples

Client integration examples for the StellarNotify contract using Rust and the Soroban SDK.

## Setup

1. Ensure you have Rust installed with the stable toolchain:

```bash
rustup install stable
rustup default stable
```

2. Build the examples:

```bash
cd examples/rust
cargo build --examples
```

## Available Examples

### 1. Basic Subscribe (`basic_subscribe.rs`)

Demonstrates the fundamentals of creating a subscription.

```bash
cargo run --example basic_subscribe
```

**What it covers:**
- Computing SHA-256 endpoint reference hashes
- Understanding subscription parameters
- Channel types (Webhook, InApp, OnChain)
- Integration with `stellar-cli`
- Topic filtering basics

**Note**: This example runs in test mode to demonstrate data structures. For actual blockchain interaction, use `stellar-cli` commands shown in the output.

### 2. Query Subscriptions (`query_subscriptions.rs`)

Shows all read-only query patterns for retrieving subscription data.

```bash
cargo run --example query_subscriptions
```

**What it covers:**
- Getting subscription by ID
- Listing subscriptions by owner
- Listing subscriptions by watched contract
- Fetching lightweight summaries for dashboards
- Querying protocol configuration
- Getting contract version
- Error handling for queries

**Key insight**: All query operations are simulation-only (no fees, no auth required).

### 3. Manage Subscriptions (`manage_subscriptions.rs`)

Demonstrates subscription lifecycle management.

```bash
cargo run --example manage_subscriptions
```

**What it covers:**
- Pausing subscriptions (temporary disable)
- Resuming paused subscriptions
- Updating endpoint references (webhook rotation)
- Renewing subscription TTL
- Cancelling subscriptions (permanent deletion)
- Best practices for each operation

**Owner authentication**: All management operations require the caller to be the subscription owner.

### 4. Admin Operations (`admin_operations.rs`)

Shows admin-only protocol management functions.

```bash
cargo run --example admin_operations
```

**What it covers:**
- Contract initialization after deployment
- Updating protocol configuration (limits)
- Emergency pause/unpause mechanism
- Admin role transfer
- Admin responsibilities and limitations
- Security best practices

**Admin access required**: These functions can only be called by the designated admin address.

## Generating Contract Bindings

For production usage, generate type-safe Rust bindings from the contract WASM:

```bash
# Build the contract first
cd ../..
stellar contract build

# Generate bindings
stellar contract bindings rust \
  --wasm target/wasm32-unknown-unknown/release/stellarnotify_contract.wasm \
  --output-dir examples/rust/bindings
```

This creates a `StellarNotifyContractClient` that you can use in your Rust applications:

```rust
use soroban_sdk::{Address, Env};

// Import generated client
mod bindings;
use bindings::StellarNotifyContractClient;

fn create_subscription(
    env: &Env,
    contract_id: &Address,
    owner: &Address,
    watched: &Address,
) -> Result<u64, NotifyError> {
    // Create client
    let client = StellarNotifyContractClient::new(env, contract_id);
    
    // Call contract method
    let subscription_id = client.subscribe(
        owner,
        watched,
        &Vec::new(env),           // topics
        &Channel::Webhook,         // channel
        &compute_endpoint_ref(env, "https://example.com/webhook"),
        &0u32,                     // ttl_ledgers
    )?;
    
    Ok(subscription_id)
}
```

## Using with stellar-cli

The examples output `stellar-cli` commands that you can run directly. Here's a complete workflow:

### 1. Deploy and Initialize

```bash
# Deploy the contract
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellarnotify_contract.wasm \
  --source YOUR_SECRET_KEY \
  --network testnet)

echo "Contract deployed: $CONTRACT_ID"

# Initialize the contract
stellar contract invoke \
  --id $CONTRACT_ID \
  --source YOUR_SECRET_KEY \
  --network testnet \
  -- initialise \
  --admin YOUR_ADDRESS \
  --max_per_owner 20 \
  --max_ttl 0
```

### 2. Create a Subscription

```bash
# Compute endpoint hash (using openssl or sha256sum)
ENDPOINT_HASH=$(echo -n "https://your-domain.com/webhook" | sha256sum | cut -d' ' -f1)

# Create subscription
stellar contract invoke \
  --id $CONTRACT_ID \
  --source YOUR_SECRET_KEY \
  --network testnet \
  -- subscribe \
  --owner YOUR_ADDRESS \
  --watched_contract CONTRACT_TO_WATCH \
  --topics '[]' \
  --channel '{"Webhook": null}' \
  --endpoint_ref $ENDPOINT_HASH \
  --ttl_ledgers 0
```

### 3. Query Subscriptions

```bash
# List your subscriptions
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- list_by_owner \
  --owner YOUR_ADDRESS

# Get subscription details
stellar contract invoke \
  --id $CONTRACT_ID \
  --network testnet \
  -- get_sub \
  --id 1
```

### 4. Manage Subscription

```bash
# Pause a subscription
stellar contract invoke \
  --id $CONTRACT_ID \
  --source YOUR_SECRET_KEY \
  --network testnet \
  -- pause_sub \
  --owner YOUR_ADDRESS \
  --id 1

# Resume it
stellar contract invoke \
  --id $CONTRACT_ID \
  --source YOUR_SECRET_KEY \
  --network testnet \
  -- resume_sub \
  --owner YOUR_ADDRESS \
  --id 1
```

## Data Types

### Channel

The contract supports three delivery channels:

```rust
pub enum Channel {
    Webhook,  // HTTP POST to registered endpoint
    InApp,    // Server-Sent Events via backend
    OnChain,  // Re-emitted as Soroban event
}
```

### Subscription

Full subscription data structure:

```rust
pub struct Subscription {
    pub owner: Address,              // Subscription owner
    pub watched_contract: Address,   // Contract being watched
    pub topics: Vec<Bytes>,          // Event topic filters
    pub channel: Channel,            // Delivery method
    pub endpoint_ref: Bytes,         // SHA-256 hash of URL
    pub active: bool,                // Whether notifications are enabled
    pub created_at: u32,             // Creation ledger
    pub expires_at: u32,             // Expiry ledger (0 = never)
}
```

### SubscriptionSummary

Lightweight version for efficient queries:

```rust
pub struct SubscriptionSummary {
    pub id: u64,
    pub owner: Address,
    pub watched_contract: Address,
    pub active: bool,
    pub channel: Channel,
    pub expires_at: u32,
}
```

### ProtocolConfig

Protocol-level configuration:

```rust
pub struct ProtocolConfig {
    pub max_per_owner: u32,  // Max subscriptions per wallet
    pub max_ttl: u32,        // Max TTL in ledgers (0 = unlimited)
    pub admin: Address,      // Admin address
    pub paused: bool,        // Whether new subscriptions are blocked
}
```

## Error Handling

All contract functions return `Result<T, NotifyError>`. Handle errors appropriately:

```rust
use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
pub enum NotifyError {
    AlreadyInitialised = 1,
    NotInitialised = 2,
    Unauthorised = 3,
    SubNotFound = 4,
    NotOwner = 5,
    LimitExceeded = 6,
    TtlExceeded = 7,
    Paused = 8,
    Expired = 9,
    TooManyTopics = 10,
    EmptyEndpoint = 11,
}
```

Example error handling:

```rust
match client.subscribe(...) {
    Ok(id) => println!("Created subscription {}", id),
    Err(NotifyError::LimitExceeded) => {
        eprintln!("You've reached the maximum subscriptions per owner");
    }
    Err(NotifyError::Paused) => {
        eprintln!("Protocol is paused, try again later");
    }
    Err(e) => eprintln!("Unexpected error: {:?}", e),
}
```

## Common Patterns

### Computing Endpoint Hash

```rust
use sha2::{Digest, Sha256};
use soroban_sdk::{Bytes, Env};

fn compute_endpoint_ref(env: &Env, url: &str) -> Bytes {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    Bytes::from_slice(env, &hash)
}
```

### Topic Filtering

```rust
use soroban_sdk::{Bytes, Env, Vec};

// Watch all events
let all_events: Vec<Bytes> = Vec::new(&env);

// Watch specific topics
let mut topics = Vec::new(&env);
topics.push_back(Bytes::from_slice(&env, b"transfer"));
topics.push_back(Bytes::from_slice(&env, b"mint"));
```

### TTL Calculations

```rust
// Assuming 5 seconds per ledger on average

// 30 days
let thirty_days = 30 * 24 * 60 * 60 / 5;  // 518,400 ledgers

// 1 year
let one_year = 365 * 24 * 60 * 60 / 5;    // 6,307,200 ledgers

// Permanent
let permanent = 0u32;
```

### Subscription Lifecycle

```rust
// Create
let id = client.subscribe(...)?;

// Use for some time...

// Pause temporarily
client.pause_sub(&owner, &id)?;

// Resume when ready
client.resume_sub(&owner, &id)?;

// Extend before expiry
client.renew_sub(&owner, &id, &1_000_000)?;

// Cancel when done
client.cancel(&owner, &id)?;
```

## Integration with Backend

The Rust examples focus on contract interaction. For complete notification delivery:

1. **Contract** (this repo): Stores subscription registry on-chain
2. **Backend** ([stellarnotify-backend](https://github.com/Stellarnotify/stellarnotify-backend)): Monitors events and dispatches notifications
3. **Your app**: Receives webhooks or connects to SSE streams

The endpoint hash stored on-chain must match the URL registered in the backend's `endpoint_registry` table.

## Testing

Write tests using `soroban-sdk`'s test utilities:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_subscribe_and_query() {
        let env = Env::default();
        let contract_id = env.register_contract(None, StellarNotifyContract);
        let client = StellarNotifyContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let watched = Address::generate(&env);
        
        // Initialize
        client.initialise(&admin, &20, &0);
        
        // Subscribe
        let id = client.subscribe(
            &owner,
            &watched,
            &Vec::new(&env),
            &Channel::Webhook,
            &Bytes::from_slice(&env, &[0u8; 32]),
            &0u32,
        );
        
        // Query
        let sub = client.get_sub(&id).unwrap();
        assert_eq!(sub.owner, owner);
        assert_eq!(sub.watched_contract, watched);
    }
}
```

## Performance Considerations

1. **Use summaries for lists**: `list_summaries_by_owner` is more efficient than calling `get_sub` repeatedly
2. **Cache subscription IDs**: IDs don't change once created
3. **Batch operations**: If creating multiple subscriptions, submit transactions in parallel
4. **Monitor TTLs**: Set up background jobs to renew subscriptions before expiry

## Resources

- [Soroban Documentation](https://soroban.stellar.org/docs)
- [soroban-sdk Docs](https://docs.rs/soroban-sdk)
- [Stellar CLI Guide](https://developers.stellar.org/docs/tools/cli)
- [StellarNotify Contract Source](https://github.com/Stellarnotify/stellarnotify-contract)
- [StellarNotify Backend](https://github.com/Stellarnotify/stellarnotify-backend)

## Support

For issues or questions:
- Contract issues: [stellarnotify-contract/issues](https://github.com/Stellarnotify/stellarnotify-contract/issues)
- Backend integration: [stellarnotify-backend](https://github.com/Stellarnotify/stellarnotify-backend)
- General documentation: [stellarnotify-docs](https://github.com/Stellarnotify/stellarnotify-docs)
