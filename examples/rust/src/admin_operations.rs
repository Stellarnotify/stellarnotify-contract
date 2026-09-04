//! Admin operations example for StellarNotify contract
//!
//! This example demonstrates admin-only functions:
//! - Initializing the contract after deployment
//! - Updating protocol configuration (limits)
//! - Pausing/unpausing the protocol
//! - Transferring admin role

use soroban_sdk::{Address, Env};

fn main() {
    println!("=== StellarNotify Admin Operations (Rust) ===\n");

    println!("⚠️  ADMIN ACCESS REQUIRED - These functions can only be called by the admin address.\n");

    let env = Env::default();
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    println!("Admin address: {}", admin);
    println!();

    // ========== INITIALIZE ==========
    println!("--- Operation 1: Initialize Contract ---");
    println!("Function: initialise(admin: Address, max_per_owner: u32, max_ttl: u32) -> Result<(), NotifyError>\n");
    println!("MUST be called exactly once immediately after contract deployment.");
    println!("This sets up the protocol configuration and establishes the admin role.\n");
    
    let max_per_owner = 20u32; // Max subscriptions per wallet
    let max_ttl = 0u32; // 0 = unlimited TTL
    
    println!("Configuration:");
    println!("  max_per_owner: {} (subscription limit per wallet)", max_per_owner);
    println!("  max_ttl: {} (0 = no cap, subscriptions can be permanent)", max_ttl);
    println!();
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <ADMIN_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- initialise \\");
    println!("  --admin {} \\", admin);
    println!("  --max_per_owner {} \\", max_per_owner);
    println!("  --max_ttl {}", max_ttl);
    
    println!("\nErrors:");
    println!("  - AlreadyInitialised (code 1): Contract has already been initialized");
    println!("\nSecurity:");
    println!("  - Can only be called once");
    println!("  - The caller establishes themselves as the admin");
    println!("  - Choose admin carefully - only they can modify protocol settings");

    // ========== UPDATE CONFIG ==========
    println!("\n--- Operation 2: Update Configuration ---");
    println!("Function: update_config(admin: Address, max_per_owner: u32, max_ttl: u32) -> Result<(), NotifyError>\n");
    println!("Updates protocol-level limits without affecting existing subscriptions.\n");
    
    let new_max_per_owner = 50u32;
    let new_max_ttl = 5_256_000u32; // ~1 year at 6s/ledger
    
    println!("New configuration:");
    println!("  max_per_owner: {} (increased limit)", new_max_per_owner);
    println!("  max_ttl: {} ledgers (~{} days)", new_max_ttl, new_max_ttl * 6 / 86400);
    println!();
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <ADMIN_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- update_config \\");
    println!("  --admin {} \\", admin);
    println!("  --max_per_owner {} \\", new_max_per_owner);
    println!("  --max_ttl {}", new_max_ttl);
    
    println!("\nUse cases:");
    println!("  - Increase limits as the protocol matures");
    println!("  - Decrease limits to manage storage costs");
    println!("  - Introduce TTL caps to prevent permanent subscriptions");
    println!("\nImportant:");
    println!("  - Existing subscriptions are NOT affected");
    println!("  - Lowering max_per_owner doesn't force users to cancel");
    println!("  - Only new subscriptions are subject to the new limits");

    // ========== SET PAUSED ==========
    println!("\n--- Operation 3: Pause/Unpause Protocol ---");
    println!("Function: set_paused(admin: Address, paused: bool) -> Result<(), NotifyError>\n");
    println!("Emergency pause/unpause mechanism to halt new subscription creation.");
    println!("Existing subscriptions continue to function normally.\n");
    
    println!("Pause the protocol:");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <ADMIN_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- set_paused \\");
    println!("  --admin {} \\", admin);
    println!("  --paused true");
    
    println!("\nUnpause the protocol:");
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <ADMIN_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- set_paused \\");
    println!("  --admin {} \\", admin);
    println!("  --paused false");
    
    println!("\nWhen paused:");
    println!("  ✓ Existing subscriptions continue working");
    println!("  ✓ Users can pause, resume, renew, cancel subscriptions");
    println!("  ✓ Query functions remain available");
    println!("  ✗ New subscriptions cannot be created (Paused error, code 8)");
    println!("\nUse cases:");
    println!("  - Emergency response to security issues");
    println!("  - Controlled migration to new contract version");
    println!("  - Maintenance periods requiring no new state changes");

    // ========== TRANSFER ADMIN ==========
    println!("\n--- Operation 4: Transfer Admin Role ---");
    println!("Function: transfer_admin(admin: Address, new_admin: Address) -> Result<(), NotifyError>\n");
    println!("⚠️  IRREVERSIBLE - Transfers admin privileges to a new address.");
    println!("The current admin immediately loses all admin access.\n");
    
    println!("New admin: {}", new_admin);
    println!();
    
    println!("stellar contract invoke \\");
    println!("  --id <CONTRACT_ID> \\");
    println!("  --source <CURRENT_ADMIN_SECRET_KEY> \\");
    println!("  --network testnet \\");
    println!("  -- transfer_admin \\");
    println!("  --admin {} \\", admin);
    println!("  --new_admin {}", new_admin);
    
    println!("\n⚠️  CRITICAL WARNINGS:");
    println!("  - This action is PERMANENT");
    println!("  - Verify new_admin address multiple times");
    println!("  - Ensure new admin has secure key management");
    println!("  - Current admin loses all privileges immediately");
    println!("  - Cannot be undone without new admin's cooperation");
    println!("\nBest practices:");
    println!("  - Test with a temporary admin on testnet first");
    println!("  - Use multisig wallets for production admin keys");
    println!("  - Document the transfer procedure");
    println!("  - Verify new_admin can sign transactions before transfer");

    // ========== RUST CLIENT EXAMPLE ==========
    println!("\n--- Rust Client Implementation Example ---\n");
    println!("```rust");
    println!("use soroban_sdk::{{Address, Env}};");
    println!("");
    println!("/// Initialize a newly deployed contract");
    println!("fn initialize_contract(");
    println!("    env: &Env,");
    println!("    contract_id: &Address,");
    println!("    admin: &Address,");
    println!(") -> Result<(), NotifyError> {{");
    println!("    let client = StellarNotifyContractClient::new(env, contract_id);");
    println!("    ");
    println!("    // Set initial limits");
    println!("    let max_per_owner = 20u32;  // Conservative starting limit");
    println!("    let max_ttl = 0u32;          // Allow permanent subscriptions");
    println!("    ");
    println!("    client.initialise(admin, &max_per_owner, &max_ttl)?;");
    println!("    ");
    println!("    println!(\"Contract initialized successfully\");");
    println!("    println!(\"Admin: {{}}\", admin);");
    println!("    println!(\"Max per owner: {{}}\", max_per_owner);");
    println!("    println!(\"Max TTL: unlimited\");");
    println!("    ");
    println!("    Ok(())");
    println!("}}");
    println!("");
    println!("/// Emergency pause with automatic unpause");
    println!("fn emergency_pause_and_recover(");
    println!("    env: &Env,");
    println!("    contract_id: &Address,");
    println!("    admin: &Address,");
    println!(") -> Result<(), NotifyError> {{");
    println!("    let client = StellarNotifyContractClient::new(env, contract_id);");
    println!("    ");
    println!("    // Pause immediately");
    println!("    client.set_paused(admin, &true)?;");
    println!("    println!(\"⚠️  Protocol paused - investigating issue...\");");
    println!("    ");
    println!("    // Investigate and resolve the issue...");
    println!("    perform_emergency_maintenance()?;");
    println!("    ");
    println!("    // Unpause once safe");
    println!("    client.set_paused(admin, &false)?;");
    println!("    println!(\"✓ Protocol resumed - all systems normal\");");
    println!("    ");
    println!("    Ok(())");
    println!("}}");
    println!("");
    println!("/// Gradual limit increase");
    println!("fn scale_limits_gradually(");
    println!("    env: &Env,");
    println!("    contract_id: &Address,");
    println!("    admin: &Address,");
    println!(") -> Result<(), NotifyError> {{");
    println!("    let client = StellarNotifyContractClient::new(env, contract_id);");
    println!("    ");
    println!("    // Start: 20 per owner");
    println!("    client.update_config(admin, &20, &0)?;");
    println!("    ");
    println!("    // After 1 month: increase to 50");
    println!("    client.update_config(admin, &50, &0)?;");
    println!("    ");
    println!("    // After 3 months: increase to 100");
    println!("    client.update_config(admin, &100, &0)?;");
    println!("    ");
    println!("    println!(\"Limits scaled to production capacity\");");
    println!("    Ok(())");
    println!("}}");
    println!("```");

    println!("\n--- Admin Best Practices ---\n");
    println!("1. Initialization");
    println!("   - Deploy contract");
    println!("   - Immediately call initialise() in the same session");
    println!("   - Verify config with get_config()");
    println!("   - Document admin address securely");
    println!("");
    println!("2. Configuration updates");
    println!("   - Start with conservative limits");
    println!("   - Monitor usage before increasing");
    println!("   - Consider storage costs when setting max_per_owner");
    println!("   - Announce limit changes to users in advance");
    println!("");
    println!("3. Emergency procedures");
    println!("   - Have pause criteria documented");
    println!("   - Test pause/unpause on testnet");
    println!("   - Communicate with users during downtime");
    println!("   - Keep pause duration minimal");
    println!("");
    println!("4. Admin key security");
    println!("   - Use hardware wallets for mainnet admin keys");
    println!("   - Consider multisig solutions");
    println!("   - Never expose admin secret in logs or code");
    println!("   - Rotate keys periodically via transfer_admin");
    println!("");
    println!("5. Monitoring");
    println!("   - Track subscription growth vs limits");
    println!("   - Monitor storage usage and costs");
    println!("   - Set up alerts for unusual activity");
    println!("   - Regular config audits");

    println!("\n--- Admin Responsibilities ---\n");
    println!("The admin role controls protocol-level settings but has LIMITED scope:");
    println!("");
    println!("✓ What admin CAN do:");
    println!("  - Initialize the contract");
    println!("  - Update max_per_owner and max_ttl limits");
    println!("  - Pause/unpause new subscription creation");
    println!("  - Transfer admin role to another address");
    println!("");
    println!("✗ What admin CANNOT do:");
    println!("  - View, modify, or cancel user subscriptions");
    println!("  - Access webhook URLs or endpoint data");
    println!("  - Override owner permissions");
    println!("  - Prevent users from managing their own subscriptions");
    println!("  - Extract or custody any user funds (contract holds no tokens)");
    println!("");
    println!("This separation ensures user sovereignty over their subscriptions.");

    println!("\n=== Admin operations examples completed ===");
    println!("\nFor production deployment:");
    println!("1. Deploy contract: stellar contract deploy --wasm contract.wasm");
    println!("2. Initialize: Use Operation 1 above");
    println!("3. Verify: Check get_config() returns expected values");
    println!("4. Test: Create a test subscription to verify functionality");
    println!("5. Monitor: Watch for errors and usage patterns");
}
