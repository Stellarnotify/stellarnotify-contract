/**
 * Manage subscriptions example for StellarNotify contract
 * 
 * This example demonstrates:
 * - Pausing a subscription
 * - Resuming a paused subscription
 * - Updating the endpoint reference
 * - Renewing a subscription's TTL
 * - Cancelling a subscription
 */

import * as StellarSDK from '@stellar/stellar-sdk';
import { createHash } from 'crypto';

// ========== CONFIGURATION ==========
const CONTRACT_ID = 'CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX';
const SOURCE_SECRET = 'SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX';

const NETWORK_PASSPHRASE = StellarSDK.Networks.TESTNET;
const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const RPC_URL = 'https://soroban-testnet.stellar.org';

// ========== HELPER FUNCTIONS ==========

function computeEndpointRef(url) {
  return createHash('sha256').update(url).digest();
}

async function loadAccount(publicKey) {
  const server = new StellarSDK.Horizon.Server(HORIZON_URL);
  return await server.loadAccount(publicKey);
}

/**
 * Execute a contract function and wait for result
 */
async function executeContractFunction(contract, functionName, params, sourceKeypair) {
  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  const account = await loadAccount(sourceKeypair.publicKey());

  const tx = new StellarSDK.TransactionBuilder(account, {
    fee: StellarSDK.BASE_FEE,
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(contract.call(functionName, ...params))
    .setTimeout(30)
    .build();

  tx.sign(sourceKeypair);

  const preparedTx = await rpcServer.prepareTransaction(tx);
  preparedTx.sign(sourceKeypair);

  const response = await rpcServer.sendTransaction(preparedTx);
  console.log(`  Transaction hash: ${response.hash}`);

  // Poll for result
  let status = response.status;
  let attempts = 0;
  while (status === 'PENDING' && attempts < 10) {
    await new Promise(resolve => setTimeout(resolve, 1000));
    const txResponse = await rpcServer.getTransaction(response.hash);
    status = txResponse.status;
    attempts++;
  }

  if (status === 'SUCCESS') {
    const result = await rpcServer.getTransaction(response.hash);
    return result.returnValue ? StellarSDK.scValToNative(result.returnValue) : null;
  } else {
    throw new Error(`Transaction failed with status: ${status}`);
  }
}

// ========== SUBSCRIPTION MANAGEMENT FUNCTIONS ==========

/**
 * Pause a subscription
 */
async function pauseSubscription(contract, sourceKeypair, subscriptionId) {
  console.log(`\nPausing subscription ${subscriptionId}...`);

  const params = [
    StellarSDK.nativeToScVal(sourceKeypair.publicKey(), { type: 'address' }),
    StellarSDK.nativeToScVal(subscriptionId, { type: 'u64' })
  ];

  await executeContractFunction(contract, 'pause_sub', params, sourceKeypair);
  console.log('✓ Subscription paused');
}

/**
 * Resume a paused subscription
 */
async function resumeSubscription(contract, sourceKeypair, subscriptionId) {
  console.log(`\nResuming subscription ${subscriptionId}...`);

  const params = [
    StellarSDK.nativeToScVal(sourceKeypair.publicKey(), { type: 'address' }),
    StellarSDK.nativeToScVal(subscriptionId, { type: 'u64' })
  ];

  await executeContractFunction(contract, 'resume_sub', params, sourceKeypair);
  console.log('✓ Subscription resumed');
}

/**
 * Update the endpoint reference (webhook URL hash)
 */
async function updateEndpoint(contract, sourceKeypair, subscriptionId, newWebhookUrl) {
  console.log(`\nUpdating endpoint for subscription ${subscriptionId}...`);
  console.log(`  New webhook URL: ${newWebhookUrl}`);

  const endpointRef = computeEndpointRef(newWebhookUrl);

  const params = [
    StellarSDK.nativeToScVal(sourceKeypair.publicKey(), { type: 'address' }),
    StellarSDK.nativeToScVal(subscriptionId, { type: 'u64' }),
    StellarSDK.nativeToScVal(endpointRef, { type: 'bytes' })
  ];

  await executeContractFunction(contract, 'update_endpoint_ref', params, sourceKeypair);
  console.log('✓ Endpoint reference updated');
}

/**
 * Renew a subscription by extending its TTL
 */
async function renewSubscription(contract, sourceKeypair, subscriptionId, additionalLedgers) {
  console.log(`\nRenewing subscription ${subscriptionId}...`);
  console.log(`  Adding ${additionalLedgers} ledgers to TTL`);

  const params = [
    StellarSDK.nativeToScVal(sourceKeypair.publicKey(), { type: 'address' }),
    StellarSDK.nativeToScVal(subscriptionId, { type: 'u64' }),
    StellarSDK.nativeToScVal(additionalLedgers, { type: 'u32' })
  ];

  await executeContractFunction(contract, 'renew_sub', params, sourceKeypair);
  console.log('✓ Subscription renewed');
}

/**
 * Cancel a subscription permanently
 */
async function cancelSubscription(contract, sourceKeypair, subscriptionId) {
  console.log(`\nCancelling subscription ${subscriptionId}...`);
  console.log('  ⚠️  This action is permanent and cannot be undone');

  const params = [
    StellarSDK.nativeToScVal(sourceKeypair.publicKey(), { type: 'address' }),
    StellarSDK.nativeToScVal(subscriptionId, { type: 'u64' })
  ];

  await executeContractFunction(contract, 'cancel', params, sourceKeypair);
  console.log('✓ Subscription cancelled and removed from storage');
}

// ========== MAIN EXECUTION ==========

async function main() {
  try {
    console.log('=== StellarNotify Subscription Management Examples ===\n');

    const contract = new StellarSDK.Contract(CONTRACT_ID);
    const sourceKeypair = StellarSDK.Keypair.fromSecret(SOURCE_SECRET);

    console.log('Owner address:', sourceKeypair.publicKey());
    console.log('Contract ID:', CONTRACT_ID);

    // Replace with an actual subscription ID from your account
    const SUBSCRIPTION_ID = 1;

    console.log('\n--- Example 1: Pause a subscription ---');
    console.log('Pausing stops notification delivery but keeps the subscription data');
    await pauseSubscription(contract, sourceKeypair, SUBSCRIPTION_ID);

    console.log('\n--- Example 2: Resume a paused subscription ---');
    console.log('Resuming re-enables notification delivery');
    await resumeSubscription(contract, sourceKeypair, SUBSCRIPTION_ID);

    console.log('\n--- Example 3: Update endpoint reference ---');
    console.log('Rotate the webhook URL without cancelling the subscription');
    const newWebhookUrl = 'https://your-domain.com/webhooks/stellar-notify-v2';
    await updateEndpoint(contract, sourceKeypair, SUBSCRIPTION_ID, newWebhookUrl);

    console.log('\n--- Example 4: Renew subscription TTL ---');
    console.log('Extend the subscription lifetime by adding ledgers');
    const additionalLedgers = 1000000; // ~58 days at 5s/ledger
    await renewSubscription(contract, sourceKeypair, SUBSCRIPTION_ID, additionalLedgers);

    console.log('\n--- Example 5: Cancel subscription ---');
    console.log('⚠️  Commented out to prevent accidental deletion');
    console.log('Uncomment the line below to test cancellation:');
    console.log('// await cancelSubscription(contract, sourceKeypair, SUBSCRIPTION_ID);');
    
    // Uncomment to test:
    // await cancelSubscription(contract, sourceKeypair, SUBSCRIPTION_ID);

    console.log('\n=== Management examples completed ===');
    console.log('\nNext steps:');
    console.log('- Use query-subscriptions.js to verify the changes');
    console.log('- Check the subscription status in the StellarNotify dashboard');

  } catch (error) {
    console.error('Error:', error.message);
    if (error.response) {
      console.error('Response:', error.response);
    }
    process.exit(1);
  }
}

main();
