//! Query subscriptions example for StellarNotify contract
//!
//! This example demonstrates:
//! - Retrieving a single subscription by ID
//! - Listing all subscriptions for an owner
//! - Listing all subscriptions watching a contract
//! - Getting lightweight subscription summaries
//! - Querying contract configuration and version

use soroban_sdk::{Address, Bytes, Env, Vec};

/// Subscription data structure matching contract's Subscription type
#[derive(Clone, Debug)]
pub struct Subscription {
    pub owner: Address,
    pub watched_contract: Address,
    pub topics: Vec<Bytes>,
    pub channel: Channel,
    pub endpoint_ref: Bytes,
    pub active: bool,
    pub created_at: u32,
    pub expires_at: u32,
}

/// Channel enum matching contract's Channel type
#[derive(Clone, Debug, PartialEq)]
pub enum Channel {
    Webhook,
    InApp,
    OnChain,
}

/// Lightweight subscription summary
#[derive(Clone, Debug)]
pub struct SubscriptionSummary {
    pub id: u64,
    pub owner: Address,
    pub watched_contract: Address,
    pub active: bool,
    pub channel: Channel,
    pub expires_at: u32,
}

/// Protocol configuration
#[derive(Clone, Debug)]
pub struct ProtocolConfig {
    pub max_per_owner: u32,
    pub max_ttl: u32,
    pub admin: Address,
    pub paused: bool,
}

fn main() {
    println!("=== StellarNotify Query Examples (Rust) ===\n");

    println!("This example demonstrates read-only query patterns for the StellarNotify contract.\n");

    // Create test environment
    let env = Env::default();
    let owner = Address::generate(&env);
    let watched_contract = Address::generate(&env);
    let subscription_id: u64 = 1;

    println!("--- Query 1: Get Subscription by ID ---");
    println!("Function: get_sub(id: u64) -> Result<Subscription, NotifyError>\n");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --network testnet \\");
    println!("  -- get_sub \\");
    println!("  --id {}", subscription_id);
    println!("\nReturns full subscription details including:");
    println!("  - owner: Address");
    println!("  - watched_contract: Address");
    println!("  - topics: Vec<Bytes>");
    println!("  - channel: Channel (Webhook/InApp/OnChain)");
    println!("  - endpoint_ref: Bytes (SHA-256 hash)");
    println!("  - active: bool");
    println!("  - created_at: u32 (ledger number)");
    println!("  - expires_at: u32 (0 = never)");

    println!("\n--- Query 2: List Subscriptions by Owner ---");
    println!("Function: list_by_owner(owner: Address) -> Vec<u64>\n");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --network testnet \\");
    println!("  -- list_by_owner \\");
    println!("  --owner {}", owner);
    println!("\nReturns a vector of subscription IDs owned by the address.");
    println!("Use this to discover all subscriptions for a wallet.");

    println!("\n--- Query 3: List Subscriptions by Contract ---");
    println!("Function: list_by_contract(watched: Address) -> Vec<u64>\n");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --network testnet \\");
    println!("  -- list_by_contract \\");
    println!("  --watched {}", watched_contract);
    println!("\nReturns all subscription IDs watching the specified contract.");
    println!("Useful for analytics and discovering who's watching your contract.");

    println!("\n--- Query 4: List Subscription Summaries by Owner ---");
    println!("Function: list_summaries_by_owner(owner: Address) -> Vec<SubscriptionSummary>\n");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --network testnet \\");
    println!("  -- list_summaries_by_owner \\");
    println!("  --owner {}", owner);
    println!("\nReturns lightweight summaries without topics and endpoint_ref.");
    println!("More efficient for dashboard displays. Each summary includes:");
    println!("  - id: u64");
    println!("  - owner: Address");
    println!("  - watched_contract: Address");
    println!("  - active: bool");
    println!("  - channel: Channel");
    println!("  - expires_at: u32");

    println!("\n--- Query 5: Get Contract Configuration ---");
    println!("Function: get_config() -> Result<ProtocolConfig, NotifyError>\n");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --network testnet \\");
    println!("  -- get_config");
    println!("\nReturns current protocol settings:");
    println!("  - max_per_owner: u32 (subscription limit per wallet)");
    println!("  - max_ttl: u32 (maximum TTL in ledgers, 0 = unlimited)");
    println!("  - admin: Address (admin wallet)");
    println!("  - paused: bool (whether new subscriptions are blocked)");

    println!("\n--- Query 6: Get Contract Version ---");
    println!("Function: get_version() -> String\n");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --network testnet \\");
    println!("  -- get_version");
    println!("\nReturns the contract version string (e.g., '0.1.0').");

    println!("\n--- Rust Client Implementation Example ---\n");
    println!("```rust");
    println!("use soroban_sdk::{{Address, Env}};");
    println!("");
    println!("// In a real client, you'd use the generated contract bindings");
    println!("// from stellar-cli: stellar contract bindings rust --wasm contract.wasm");
    println!("");
    println!("fn query_my_subscriptions(");
    println!("    env: &Env,");
    println!("    contract_id: &Address,");
    println!("    owner: &Address,");
    println!(") -> Vec<u64> {{");
    println!("    // Create contract client");
    println!("    let client = StellarNotifyContractClient::new(env, contract_id);");
    println!("    ");
    println!("    // Query subscriptions - no authentication needed for reads");
    println!("    let subscription_ids = client.list_by_owner(owner);");
    println!("    ");
    println!("    println!(\"Found {{}} subscriptions\", subscription_ids.len());");
    println!("    ");
    println!("    // Get details for each subscription");
    println!("    for id in subscription_ids.iter() {{");
    println!("        match client.try_get_sub(&id) {{");
    println!("            Ok(sub) => {{");
    println!("                println!(\"Subscription {{}}: watching {{}}\", id, sub.watched_contract);");
    println!("            }}");
    println!("            Err(e) => {{");
    println!("                eprintln!(\"Failed to get subscription {{}}: {{:?}}\", id, e);");
    println!("            }}");
    println!("        }}");
    println!("    }}");
    println!("    ");
    println!("    subscription_ids");
    println!("}}");
    println!("```");

    println!("\n--- Error Handling ---\n");
    println!("Query functions return Result types. Handle errors appropriately:");
    println!("");
    println!("Error codes (from NotifyError enum):");
    println!("  1 - AlreadyInitialised");
    println!("  2 - NotInitialised (contract not set up yet)");
    println!("  3 - Unauthorised");
    println!("  4 - SubNotFound (subscription ID doesn't exist)");
    println!("  5 - NotOwner");
    println!("  6 - LimitExceeded");
    println!("  7 - TtlExceeded");
    println!("  8 - Paused");
    println!("  9 - Expired");
    println!("  10 - TooManyTopics");
    println!("  11 - EmptyEndpoint");

    println!("\n--- Performance Considerations ---\n");
    println!("1. Use list_summaries_by_owner() instead of get_sub() for dashboards");
    println!("   - Summaries omit topics and endpoint_ref to reduce response size");
    println!("   - Significantly faster for users with many subscriptions");
    println!("");
    println!("2. Cache subscription IDs on the client side");
    println!("   - Subscription IDs never change once created");
    println!("   - Only re-query when you create/delete subscriptions");
    println!("");
    println!("3. All read queries are simulation-only");
    println!("   - No transaction fees");
    println!("   - No authentication required");
    println!("   - Fast and lightweight");

    println!("\n=== Query examples completed ===");
    println!("\nNext steps:");
    println!("- Generate Rust bindings: stellar contract bindings rust --wasm contract.wasm");
    println!("- Import the generated client in your project");
    println!("- See manage_subscriptions.rs for write operations");
}
