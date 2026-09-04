//! Manage subscriptions example for StellarNotify contract
//!
//! This example demonstrates:
//! - Pausing a subscription
//! - Resuming a paused subscription
//! - Updating the endpoint reference
//! - Renewing a subscription's TTL
//! - Cancelling a subscription

use sha2::{Digest, Sha256};
use soroban_sdk::{Address, Bytes, Env};

/// Compute SHA-256 hash for endpoint reference
fn compute_endpoint_ref(url: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hasher.finalize().into()
}

fn main() {
    println!("=== StellarNotify Subscription Management Examples (Rust) ===\n");

    println!("This example demonstrates how to manage existing subscriptions.\n");
    println!("All operations require the caller to be the subscription owner.\n");

    // Test data
    let env = Env::default();
    let owner = Address::generate(&env);
    let subscription_id: u64 = 1;

    println!("Owner address: {}", owner);
    println!("Subscription ID: {}\n", subscription_id);

    // ========== PAUSE SUBSCRIPTION ==========
    println!("--- Operation 1: Pause Subscription ---");
    println!("Function: pause_sub(owner: Address, id: u64) -> Result<(), NotifyError>\n");
    println!("Pausing stops notification delivery but keeps all subscription data.");
    println!("The subscription remains in storage and counts toward owner limits.");
    println!("This operation is idempotent (safe to call multiple times).\n");
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <YOUR_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- pause_sub \\");
    println!("  --owner {} \\", owner);
    println!("  --id {}", subscription_id);
    
    println!("\nUse cases:");
    println!("  - Temporarily disable notifications during maintenance");
    println!("  - Pause while updating your webhook endpoint");
    println!("  - Disable during high-load periods");

    // ========== RESUME SUBSCRIPTION ==========
    println!("\n--- Operation 2: Resume Subscription ---");
    println!("Function: resume_sub(owner: Address, id: u64) -> Result<(), NotifyError>\n");
    println!("Resuming re-enables notification delivery for a paused subscription.");
    println!("The contract checks that the subscription hasn't expired before resuming.");
    println!("This operation is idempotent.\n");
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <YOUR_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- resume_sub \\");
    println!("  --owner {} \\", owner);
    println!("  --id {}", subscription_id);
    
    println!("\nErrors:");
    println!("  - Expired (code 9): Subscription TTL has passed, cannot resume");
    println!("  - SubNotFound (code 4): Invalid subscription ID");
    println!("  - NotOwner (code 5): Caller is not the owner");

    // ========== UPDATE ENDPOINT ==========
    println!("\n--- Operation 3: Update Endpoint Reference ---");
    println!("Function: update_endpoint_ref(owner: Address, id: u64, new_endpoint: Bytes) -> Result<(), NotifyError>\n");
    println!("Updates the endpoint reference hash without cancelling the subscription.");
    println!("Use this to rotate webhook URLs or change notification destinations.\n");
    
    let new_webhook_url = "https://your-domain.com/webhooks/stellar-notify-v2";
    let new_endpoint_hash = compute_endpoint_ref(new_webhook_url);
    
    println!("New webhook URL: {}", new_webhook_url);
    println!("New endpoint hash: {}\n", hex::encode(new_endpoint_hash));
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <YOUR_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- update_endpoint_ref \\");
    println!("  --owner {} \\", owner);
    println!("  --id {} \\", subscription_id);
    println!("  --new_endpoint '{}'", hex::encode(new_endpoint_hash));
    
    println!("\nImportant:");
    println!("  - You must also update the URL in the StellarNotify backend");
    println!("  - The hash on-chain must match the backend's stored URL");
    println!("  - Empty endpoint_ref will fail with EmptyEndpoint (code 11)");

    // ========== RENEW SUBSCRIPTION ==========
    println!("\n--- Operation 4: Renew Subscription TTL ---");
    println!("Function: renew_sub(owner: Address, id: u64, add_ttl_ledgers: u32) -> Result<(), NotifyError>\n");
    println!("Extends the subscription's time-to-live by adding more ledgers.");
    println!("The subscription ID remains the same - no need to update integrations.\n");
    
    let additional_ledgers: u32 = 1_000_000; // ~58 days at 5s/ledger
    
    println!("Adding {} ledgers (~{} days)", additional_ledgers, additional_ledgers * 5 / 86400);
    println!();
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <YOUR_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- renew_sub \\");
    println!("  --owner {} \\", owner);
    println!("  --id {} \\", subscription_id);
    println!("  --add_ttl_ledgers {}", additional_ledgers);
    
    println!("\nNotes:");
    println!("  - For permanent subscriptions (TTL=0), this is a no-op");
    println!("  - The new TTL must not exceed max_ttl from ProtocolConfig");
    println!("  - TtlExceeded error (code 7) if the limit is breached");
    println!("  - Best practice: Renew before expiry, not after");

    // ========== CANCEL SUBSCRIPTION ==========
    println!("\n--- Operation 5: Cancel Subscription ---");
    println!("Function: cancel(owner: Address, id: u64) -> Result<(), NotifyError>\n");
    println!("⚠️  PERMANENT DELETION - This action cannot be undone!");
    println!("Removes the subscription from all storage indexes and frees the slot.\n");
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <YOUR_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- cancel \\");
    println!("  --owner {} \\", owner);
    println!("  --id {}", subscription_id);
    
    println!("\nWhat gets deleted:");
    println!("  - Subscription data (owner, contract, topics, etc.)");
    println!("  - Entry in owner's subscription list");
    println!("  - Entry in watched contract's subscriber list");
    println!("  - The subscription ID is freed and may be reused");
    
    println!("\nAlternatives:");
    println!("  - Use pause_sub() instead if you might want to resume later");
    println!("  - Set a short TTL when creating subscriptions for auto-expiry");

    // ========== RUST CLIENT EXAMPLE ==========
    println!("\n--- Rust Client Implementation Example ---\n");
    println!("```rust");
    println!("use soroban_sdk::{{Address, Bytes, Env}};");
    println!("use sha2::{{Digest, Sha256}};");
    println!("");
    println!("fn rotate_webhook_url(");
    println!("    env: &Env,");
    println!("    contract_id: &Address,");
    println!("    owner: &Address,");
    println!("    subscription_id: u64,");
    println!("    new_url: &str,");
    println!(") -> Result<(), NotifyError> {{");
    println!("    // Generate new endpoint hash");
    println!("    let mut hasher = Sha256::new();");
    println!("    hasher.update(new_url.as_bytes());");
    println!("    let hash = hasher.finalize();");
    println!("    let endpoint_ref = Bytes::from_slice(env, &hash);");
    println!("    ");
    println!("    // Create contract client");
    println!("    let client = StellarNotifyContractClient::new(env, contract_id);");
    println!("    ");
    println!("    // Update the endpoint reference");
    println!("    client.update_endpoint_ref(owner, &subscription_id, &endpoint_ref)?;");
    println!("    ");
    println!("    println!(\"Endpoint updated successfully\");");
    println!("    Ok(())");
    println!("}}");
    println!("");
    println!("fn manage_subscription_lifecycle(");
    println!("    env: &Env,");
    println!("    contract_id: &Address,");
    println!("    owner: &Address,");
    println!("    subscription_id: u64,");
    println!(") -> Result<(), NotifyError> {{");
    println!("    let client = StellarNotifyContractClient::new(env, contract_id);");
    println!("    ");
    println!("    // Pause during maintenance");
    println!("    client.pause_sub(owner, &subscription_id)?;");
    println!("    println!(\"Paused subscription\");");
    println!("    ");
    println!("    // Perform maintenance...");
    println!("    ");
    println!("    // Resume when ready");
    println!("    client.resume_sub(owner, &subscription_id)?;");
    println!("    println!(\"Resumed subscription\");");
    println!("    ");
    println!("    // Extend TTL before expiry");
    println!("    let add_ledgers = 1_000_000u32;");
    println!("    client.renew_sub(owner, &subscription_id, &add_ledgers)?;");
    println!("    println!(\"Extended TTL by {{}} ledgers\", add_ledgers);");
    println!("    ");
    println!("    Ok(())");
    println!("}}");
    println!("```");

    println!("\n--- Best Practices ---\n");
    println!("1. Monitor subscription TTLs");
    println!("   - Query expires_at regularly");
    println!("   - Set up alerts before expiry");
    println!("   - Auto-renew critical subscriptions");
    println!("");
    println!("2. Handle errors gracefully");
    println!("   - Check for Expired before resume");
    println!("   - Verify owner before management operations");
    println!("   - Catch TtlExceeded and adjust renewal amounts");
    println!("");
    println!("3. Pause vs Cancel");
    println!("   - Pause: Temporary, reversible, keeps data");
    println!("   - Cancel: Permanent, irreversible, frees storage");
    println!("   - Prefer pause for short-term disabling");
    println!("");
    println!("4. Endpoint rotation");
    println!("   - Update backend URL first");
    println!("   - Then update on-chain hash");
    println!("   - Verify delivery after rotation");

    println!("\n=== Management examples completed ===");
    println!("\nNext steps:");
    println!("- See admin_operations.rs for admin-level functions");
    println!("- Check the StellarNotify backend for webhook management");
    println!("- Review the contract's errors.rs for complete error codes");
}
