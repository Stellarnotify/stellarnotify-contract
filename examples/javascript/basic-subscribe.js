/**
 * Basic subscription example for StellarNotify contract
 * 
 * This example demonstrates:
 * - Connecting to the Stellar testnet
 * - Creating a subscription to watch contract events
 * - Computing the endpoint reference (SHA-256 hash)
 */

import * as StellarSDK from '@stellar/stellar-sdk';
import { createHash } from 'crypto';

// ========== CONFIGURATION ==========
// Replace these with your actual values
const CONTRACT_ID = 'CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX';
const SOURCE_SECRET = 'SXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX';
const WATCHED_CONTRACT = 'CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX'; // Contract to watch
const WEBHOOK_URL = 'https://your-domain.com/webhooks/stellar-notify';

// Network configuration
const NETWORK_PASSPHRASE = StellarSDK.Networks.TESTNET;
const HORIZON_URL = 'https://horizon-testnet.stellar.org';
const RPC_URL = 'https://soroban-testnet.stellar.org';

// ========== HELPER FUNCTIONS ==========

/**
 * Compute SHA-256 hash of webhook URL for on-chain storage
 * @param {string} url - The webhook URL to hash
 * @returns {Buffer} - SHA-256 hash as buffer
 */
function computeEndpointRef(url) {
  return createHash('sha256').update(url).digest();
}

/**
 * Create a new subscription
 * @param {StellarSDK.Contract} contract - The contract instance
 * @param {StellarSDK.Keypair} sourceKeypair - Signer keypair
 * @param {string} watchedContractId - Contract address to watch
 * @param {Array<Buffer>} topics - Event topics to filter (empty = all events)
 * @param {string} channel - Channel type: 'Webhook', 'InApp', or 'OnChain'
 * @param {string} webhookUrl - Webhook URL (will be hashed)
 * @param {number} ttlLedgers - Time-to-live in ledgers (0 = permanent)
 * @returns {Promise<number>} - Subscription ID
 */
async function subscribe(
  contract,
  sourceKeypair,
  watchedContractId,
  topics = [],
  channel = 'Webhook',
  webhookUrl = WEBHOOK_URL,
  ttlLedgers = 0
) {
  console.log('Creating subscription...');

  // Convert parameters to Stellar SDK types
  const owner = sourceKeypair.publicKey();
  const watchedContract = new StellarSDK.Address(watchedContractId);
  const endpointRef = computeEndpointRef(webhookUrl);

  // Build the channel enum - must match contract's Channel type
  let channelValue;
  switch (channel) {
    case 'Webhook':
      channelValue = StellarSDK.xdr.ScVal.scvVec([
        StellarSDK.xdr.ScVal.scvSymbol('Webhook')
      ]);
      break;
    case 'InApp':
      channelValue = StellarSDK.xdr.ScVal.scvVec([
        StellarSDK.xdr.ScVal.scvSymbol('InApp')
      ]);
      break;
    case 'OnChain':
      channelValue = StellarSDK.xdr.ScVal.scvVec([
        StellarSDK.xdr.ScVal.scvSymbol('OnChain')
      ]);
      break;
    default:
      throw new Error(`Unknown channel type: ${channel}`);
  }

  // Build the transaction
  const tx = new StellarSDK.TransactionBuilder(
    await loadAccount(owner),
    {
      fee: StellarSDK.BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    }
  )
    .addOperation(
      contract.call(
        'subscribe',
        StellarSDK.nativeToScVal(owner, { type: 'address' }),
        StellarSDK.nativeToScVal(watchedContract, { type: 'address' }),
        StellarSDK.nativeToScVal(topics, { type: 'vec' }),
        channelValue,
        StellarSDK.nativeToScVal(endpointRef, { type: 'bytes' }),
        StellarSDK.nativeToScVal(ttlLedgers, { type: 'u32' })
      )
    )
    .setTimeout(30)
    .build();

  // Sign and submit
  tx.sign(sourceKeypair);

  const rpcServer = new StellarSDK.SorobanRpc.Server(RPC_URL);
  const preparedTx = await rpcServer.prepareTransaction(tx);
  preparedTx.sign(sourceKeypair);

  const response = await rpcServer.sendTransaction(preparedTx);
  console.log('Transaction sent:', response.hash);

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
    const subscriptionId = StellarSDK.scValToNative(result.returnValue);
    console.log(`✓ Subscription created with ID: ${subscriptionId}`);
    return subscriptionId;
  } else {
    throw new Error(`Transaction failed with status: ${status}`);
  }
}

/**
 * Load account from Horizon
 */
async function loadAccount(publicKey) {
  const server = new StellarSDK.Horizon.Server(HORIZON_URL);
  return await server.loadAccount(publicKey);
}

// ========== MAIN EXECUTION ==========

async function main() {
  try {
    console.log('=== StellarNotify Basic Subscription Example ===\n');

    // Initialize the contract
    const contract = new StellarSDK.Contract(CONTRACT_ID);
    const sourceKeypair = StellarSDK.Keypair.fromSecret(SOURCE_SECRET);

    console.log('Owner address:', sourceKeypair.publicKey());
    console.log('Contract ID:', CONTRACT_ID);
    console.log('Watched contract:', WATCHED_CONTRACT);
    console.log('Webhook URL (hashed):', WEBHOOK_URL);
    console.log('');

    // Create a subscription with default settings:
    // - Empty topics array = watch all events
    // - Webhook channel
    // - TTL = 0 (permanent subscription)
    const subscriptionId = await subscribe(
      contract,
      sourceKeypair,
      WATCHED_CONTRACT,
      [], // Empty topics = all events
      'Webhook',
      WEBHOOK_URL,
      0 // Permanent subscription
    );

    console.log('\n=== Subscription Details ===');
    console.log(`ID: ${subscriptionId}`);
    console.log('Status: Active');
    console.log('Channel: Webhook');
    console.log('TTL: Permanent (0 ledgers)');
    console.log('\nYou can now use this subscription ID to manage or query the subscription.');

  } catch (error) {
    console.error('Error:', error.message);
    if (error.response) {
      console.error('Response:', error.response);
    }
    process.exit(1);
  }
}

main();
