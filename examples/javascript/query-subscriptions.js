/**
 * Query subscriptions example for StellarNotify contract
 * 
 * This example demonstrates:
 * - Retrieving a single subscription by ID
 * - Listing all subscriptions for an owner
 * - Listing all subscriptions watching a contract
 * - Getting lightweight subscription summaries
 * - Querying contract configuration and version
 */

import * as StellarSDK from '@stellar/stellar-sdk';

// ========== CONFIGURATION ==========
const CONTRACT_ID = 'CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX';
const SOURCE_SECRET = 'SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX';
const WATCHED_CONTRACT = 'CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX';

const NETWORK_PASSPHRASE = StellarSDK.Networks.TESTNET;
const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const RPC_URL = 'https://soroban-testnet.stellar.org';

// ========== QUERY FUNCTIONS ==========

/**
 * Get full subscription details by ID
 */
async function getSubscription(contract, subscriptionId) {
  console.log(`\nQuerying subscription ID: ${subscriptionId}...`);

  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  
  // Build simulation-only transaction (read-only)
  const account = await loadAccount(StellarSDK.Keypair.fromSecret(SOURCE_SECRET).publicKey());
  
  const tx = new StellarSDK.TransactionBuilder(account, {
    fee: StellarSDK.BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        'get_sub',
        StellarSDK.nativeToScVal(subscriptionId, { type: 'u64' })
      )
    )
    .setTimeout(30)
    .build();

  const simulated = await rpcServer.simulateTransaction(tx);
  
  if (StellarSDK.SorobanRpc.Api.isSimulationSuccess(simulated)) {
    const subscription = StellarSDK.scValToNative(simulated.result.retval);
    console.log('✓ Subscription found:');
    console.log('  Owner:', subscription.owner);
    console.log('  Watched contract:', subscription.watched_contract);
    console.log('  Channel:', Object.keys(subscription.channel)[0]);
    console.log('  Active:', subscription.active);
    console.log('  Created at ledger:', subscription.created_at);
    console.log('  Expires at ledger:', subscription.expires_at || 'Never');
    console.log('  Topics count:', subscription.topics.length);
    return subscription;
  } else {
    throw new Error(`Simulation failed: ${simulated.error}`);
  }
}

/**
 * List all subscription IDs for an owner
 */
async function listByOwner(contract, ownerAddress) {
  console.log(`\nListing subscriptions for owner: ${ownerAddress}...`);

  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  const account = await loadAccount(StellarSDK.Keypair.fromSecret(SOURCE_SECRET).publicKey());
  
  const tx = new StellarSDK.TransactionBuilder(account, {
    fee: StellarSDK.BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        'list_by_owner',
        StellarSDK.nativeToScVal(ownerAddress, { type: 'address' })
      )
    )
    .setTimeout(30)
    .build();

  const simulated = await rpcServer.simulateTransaction(tx);
  
  if (StellarSDK.SorobanRpc.Api.isSimulationSuccess(simulated)) {
    const subscriptionIds = StellarSDK.scValToNative(simulated.result.retval);
    console.log(`✓ Found ${subscriptionIds.length} subscription(s):`, subscriptionIds);
    return subscriptionIds;
  } else {
    throw new Error(`Simulation failed: ${simulated.error}`);
  }
}

/**
 * List all subscription IDs watching a specific contract
 */
async function listByContract(contract, watchedAddress) {
  console.log(`\nListing subscriptions watching: ${watchedAddress}...`);

  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  const account = await loadAccount(StellarSDK.Keypair.fromSecret(SOURCE_SECRET).publicKey());
  
  const tx = new StellarSDK.TransactionBuilder(account, {
    fee: StellarSDK.BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        'list_by_contract',
        StellarSDK.nativeToScVal(watchedAddress, { type: 'address' })
      )
    )
    .setTimeout(30)
    .build();

  const simulated = await rpcServer.simulateTransaction(tx);
  
  if (StellarSDK.SorobanRpc.Api.isSimulationSuccess(simulated)) {
    const subscriptionIds = StellarSDK.scValToNative(simulated.result.retval);
    console.log(`✓ Found ${subscriptionIds.length} subscription(s) watching this contract:`, subscriptionIds);
    return subscriptionIds;
  } else {
    throw new Error(`Simulation failed: ${simulated.error}`);
  }
}

/**
 * Get lightweight summaries for all owner subscriptions
 */
async function listSummariesByOwner(contract, ownerAddress) {
  console.log(`\nFetching subscription summaries for: ${ownerAddress}...`);

  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  const account = await loadAccount(StellarSDK.Keypair.fromSecret(SOURCE_SECRET).publicKey());
  
  const tx = new StellarSDK.TransactionBuilder(account, {
    fee: StellarSDK.BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(
      contract.call(
        'list_summaries_by_owner',
        StellarSDK.nativeToScVal(ownerAddress, { type: 'address' })
      )
    )
    .setTimeout(30)
    .build();

  const simulated = await rpcServer.simulateTransaction(tx);
  
  if (StellarSDK.SorobanRpc.Api.isSimulationSuccess(simulated)) {
    const summaries = StellarSDK.scValToNative(simulated.result.retval);
    console.log(`✓ Found ${summaries.length} subscription(s):`);
    summaries.forEach((summary, index) => {
      console.log(`\n  [${index + 1}] ID: ${summary.id}`);
      console.log(`      Watched: ${summary.watched_contract}`);
      console.log(`      Channel: ${Object.keys(summary.channel)[0]}`);
      console.log(`      Active: ${summary.active}`);
      console.log(`      Expires: ${summary.expires_at || 'Never'}`);
    });
    return summaries;
  } else {
    throw new Error(`Simulation failed: ${simulated.error}`);
  }
}

/**
 * Get contract configuration
 */
async function getConfig(contract) {
  console.log('\nQuerying contract configuration...');

  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  const account = await loadAccount(StellarSDK.Keypair.fromSecret(SOURCE_SECRET).publicKey());
  
  const tx = new StellarSDK.TransactionBuilder(account, {
    fee: StellarSDK.BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call('get_config'))
    .setTimeout(30)
    .build();

  const simulated = await rpcServer.simulateTransaction(tx);
  
  if (StellarSDK.SorobanRpc.Api.isSimulationSuccess(simulated)) {
    const config = StellarSDK.scValToNative(simulated.result.retval);
    console.log('✓ Configuration:');
    console.log('  Admin:', config.admin);
    console.log('  Max per owner:', config.max_per_owner);
    console.log('  Max TTL:', config.max_ttl || 'Unlimited');
    console.log('  Paused:', config.paused);
    return config;
  } else {
    throw new Error(`Simulation failed: ${simulated.error}`);
  }
}

/**
 * Get contract version
 */
async function getVersion(contract) {
  console.log('\nQuerying contract version...');

  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  const account = await loadAccount(StellarSDK.Keypair.fromSecret(SOURCE_SECRET).publicKey());
  
  const tx = new StellarSDK.TransactionBuilder(account, {
    fee: StellarSDK.BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call('get_version'))
    .setTimeout(30)
    .build();

  const simulated = await rpcServer.simulateTransaction(tx);
  
  if (StellarSDK.SorobanRpc.Api.isSimulationSuccess(simulated)) {
    const version = StellarSDK.scValToNative(simulated.result.retval);
    console.log(`✓ Contract version: ${version}`);
    return version;
  } else {
    throw new Error(`Simulation failed: ${simulated.error}`);
  }
}

async function loadAccount(publicKey) {
  const server = new StellarSDK.Horizon.Server(HORIZON_URL);
  return await server.loadAccount(publicKey);
}

// ========== MAIN EXECUTION ==========

async function main() {
  try {
    console.log('=== StellarNotify Query Examples ===\n');

    const contract = new StellarSDK.Contract(CONTRACT_ID);
    const sourceKeypair = StellarSDK.Keypair.fromSecret(SOURCE_SECRET);
    const ownerAddress = sourceKeypair.publicKey();

    // Get contract version and configuration
    await getVersion(contract);
    await getConfig(contract);

    // List all subscriptions for the current owner
    const subscriptionIds = await listByOwner(contract, ownerAddress);

    // If there are subscriptions, query details for the first one
    if (subscriptionIds.length > 0) {
      await getSubscription(contract, subscriptionIds[0]);
    }

    // Get lightweight summaries (more efficient for dashboards)
    await listSummariesByOwner(contract, ownerAddress);

    // List subscriptions watching a specific contract
    await listByContract(contract, WATCHED_CONTRACT);

    console.log('\n=== Query examples completed ===');

  } catch (error) {
    console.error('Error:', error.message);
    if (error.response) {
      console.error('Response:', error.response);
    }
    process.exit(1);
  }
}

main();
