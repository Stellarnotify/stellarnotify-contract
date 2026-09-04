//! Basic subscription example for StellarNotify contract
//!
//! This example demonstrates:
//! - Setting up a contract client in test mode
//! - Creating a subscription to watch contract events
//! - Computing the endpoint reference (SHA-256 hash)
//!
//! Note: This uses the test environment. For production usage, you'll need to
//! use stellar-cli or build a full RPC client implementation.

use sha2::{Digest, Sha256};
use soroban_sdk::{
    testutils::Address as _, token, Address, Bytes, BytesN, Env, String as SorobanString, Vec,
};

// Re-export the contract types we need
// In a real client, you'd generate these from the contract WASM using soroban-cli
#[derive(Clone, Debug, PartialEq)]
pub enum Channel {
    Webhook,
    InApp,
    OnChain,
}

/// Compute SHA-256 hash of webhook URL for on-chain storage
///
/// The contract stores only the hash to preserve endpoint privacy.
/// The actual URL is stored off-chain in the StellarNotify backend.
fn compute_endpoint_ref(env: &Env, url: &str) -> Bytes {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();

    // Convert hash bytes to Soroban Bytes
    Bytes::from_slice(env, &hash)
}

/// Convert Channel enum to the format expected by the contract
fn channel_to_symbol(env: &Env, channel: Channel) -> soroban_sdk::Val {
    match channel {
        Channel::Webhook => {
            soroban_sdk::symbol_short!("Webhook").to_val()
        }
        Channel::InApp => {
            soroban_sdk::symbol_short!("InApp").to_val()
        }
        Channel::OnChain => {
            soroban_sdk::symbol_short!("OnChain").to_val()
        }
    }
}

fn main() {
    println!("=== StellarNotify Basic Subscription Example (Rust) ===\n");

    // Create a test environment
    let env = Env::default();
    env.mock_all_auths(); // Skip signature verification in test mode

    println!("Setting up test environment...");

    // In a real scenario, you would:
    // 1. Load the contract address from configuration
    // 2. Set up proper authentication with your keypair
    // 3. Use the Stellar RPC to interact with the contract
    //
    // For this example, we'll demonstrate the data structures and logic:

    let owner = Address::generate(&env);
    let watched_contract = Address::generate(&env);
    let webhook_url = "https://your-domain.com/webhooks/stellar-notify";

    println!("Owner address: {}", owner);
    println!("Watched contract: {}", watched_contract);
    println!("Webhook URL (will be hashed): {}\n", webhook_url);

    // Compute the endpoint reference (SHA-256 hash)
    let endpoint_ref = compute_endpoint_ref(&env, webhook_url);
    println!("Endpoint reference (SHA-256): {}", hex::encode(endpoint_ref.to_array()));

    // Prepare subscription parameters
    let topics: Vec<Bytes> = Vec::new(&env); // Empty = watch all events
    let channel = Channel::Webhook;
    let ttl_ledgers: u32 = 0; // 0 = permanent subscription

    println!("\n--- Subscription Configuration ---");
    println!("Topics: [] (all events)");
    println!("Channel: {:?}", channel);
    println!("TTL: {} (permanent)", ttl_ledgers);

    println!("\n--- What happens next in production ---");
    println!("1. The contract.subscribe() function would be called with these parameters");
    println!("2. The contract validates inputs and checks limits (max_per_owner, max_ttl)");
    println!("3. A unique subscription ID is assigned (starts at 1)");
    println!("4. The subscription is stored in persistent storage with TTL bumping");
    println!("5. A SubscriptionCreated event is emitted");
    println!("6. The subscription ID is returned to the caller");

    println!("\n--- Example contract call (pseudocode) ---");
    println!("let subscription_id = contract.subscribe(");
    println!("    &owner,                 // Must match the transaction signer");
    println!("    &watched_contract,      // Contract to watch for events");
    println!("    &topics,                // Event topic filters (empty = all)");
    println!("    &channel,               // Webhook, InApp, or OnChain");
    println!("    &endpoint_ref,          // SHA-256 hash of webhook URL");
    println!("    &ttl_ledgers,           // Time-to-live (0 = permanent)");
    println!(");");

    println!("\n--- Integration with stellar-cli ---");
    println!("To invoke this function using stellar-cli:\n");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <YOUR_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- subscribe \\");
    println!("  --owner <YOUR_ADDRESS> \\");
    println!("  --watched_contract {} \\", watched_contract);
    println!("  --topics '[]' \\");
    println!("  --channel '{{\"Webhook\": null}}' \\");
    println!("  --endpoint_ref '{}' \\", hex::encode(endpoint_ref.to_array()));
    println!("  --ttl_ledgers 0");

    println!("\n=== Example completed ===");
    println!("\nNext steps:");
    println!("- Deploy the contract to testnet using `stellar contract deploy`");
    println!("- Initialize it with `stellar contract invoke ... initialise`");
    println!("- Use the stellar-cli commands above to create your subscription");
    println!("- See query_subscriptions.rs for how to retrieve subscription data");
}
