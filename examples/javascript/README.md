# JavaScript/TypeScript Examples

Client integration examples for the StellarNotify contract using the Stellar SDK.

## Setup

1. Install dependencies:

```bash
npm install
```

2. Configure your environment:

Edit the configuration constants at the top of each file:

- `CONTRACT_ID` - Your deployed StellarNotify contract address
- `SOURCE_SECRET` - Your Stellar testnet secret key
- `WATCHED_CONTRACT` - The contract address you want to watch
- `WEBHOOK_URL` - Your webhook endpoint URL

**Security Note**: Never commit real secret keys to version control. Use environment variables in production:

```javascript
const SOURCE_SECRET = process.env.STELLAR_SECRET_KEY;
```

## Available Examples

### 1. Basic Subscribe (`basic-subscribe.js`)

Creates a new subscription to watch contract events.

```bash
npm run basic-subscribe
```

**What it does:**
- Connects to Stellar testnet
- Creates a webhook subscription to watch a contract
- Computes the SHA-256 hash of your webhook URL for privacy
- Returns the new subscription ID

**Key concepts:**
- Endpoint hashing for privacy
- Channel types (Webhook, InApp, OnChain)
- TTL management (0 = permanent)

### 2. Query Subscriptions (`query-subscriptions.js`)

Demonstrates read-only queries to retrieve subscription data.

```bash
npm run query
```

**What it does:**
- Gets contract version and configuration
- Lists all subscriptions for an owner
- Retrieves full subscription details by ID
- Gets lightweight summaries (efficient for dashboards)
- Lists subscriptions watching a specific contract

**Key concepts:**
- Simulation-only transactions (no fees)
- Different query patterns for different use cases
- Summaries vs full subscription data

### 3. Manage Subscriptions (`manage-subscriptions.js`)

Shows how to manage existing subscriptions.

```bash
npm run manage
```

**What it does:**
- Pause a subscription (stop delivery, keep data)
- Resume a paused subscription
- Update the endpoint reference (rotate webhook URL)
- Renew subscription TTL (extend lifetime)
- Cancel a subscription (permanent deletion)

**Key concepts:**
- Owner-only operations
- Idempotent operations (pause/resume)
- TTL extension vs recreation

## Common Patterns

### Error Handling

All contract errors return a numeric error code:

```javascript
try {
  await contract.call('subscribe', ...);
} catch (error) {
  // Check for specific NotifyError codes
  if (error.message.includes('LimitExceeded')) {
    console.error('You have reached the maximum subscriptions per owner');
  }
}
```

Error codes from `errors.rs`:
- `1` - AlreadyInitialised
- `2` - NotInitialised
- `3` - Unauthorised
- `4` - SubNotFound
- `5` - NotOwner
- `6` - LimitExceeded
- `7` - TtlExceeded
- `8` - Paused
- `9` - Expired
- `10` - TooManyTopics
- `11` - EmptyEndpoint

### Computing Endpoint Hash

The contract stores a SHA-256 hash of your webhook URL for privacy:

```javascript
import { createHash } from 'crypto';

function computeEndpointRef(url) {
  return createHash('sha256').update(url).digest();
}

const hash = computeEndpointRef('https://your-domain.com/webhook');
```

### Channel Types

The contract supports three delivery channels:

```javascript
// 1. Webhook - HTTP POST to your endpoint
const channelWebhook = StellarSDK.xdr.ScVal.scvVec([
  StellarSDK.xdr.ScVal.scvSymbol('Webhook')
]);

// 2. InApp - Server-Sent Events via StellarNotify backend
const channelInApp = StellarSDK.xdr.ScVal.scvVec([
  StellarSDK.xdr.ScVal.scvSymbol('InApp')
]);

// 3. OnChain - Re-emitted as Soroban event
const channelOnChain = StellarSDK.xdr.ScVal.scvVec([
  StellarSDK.xdr.ScVal.scvSymbol('OnChain')
]);
```

### Topic Filtering

Filter events by topic prefixes:

```javascript
// Watch all events (empty array)
const allEvents = [];

// Watch specific topics
const topics = [
  Buffer.from('transfer'),
  Buffer.from('mint')
];

const subscriptionId = await subscribe(
  contract,
  sourceKeypair,
  watchedContract,
  topics,  // Only events with these topic prefixes
  'Webhook',
  webhookUrl,
  0
);
```

### TTL Management

Subscriptions can be permanent or expire after a certain number of ledgers:

```javascript
// Permanent subscription
const ttl = 0;

// Expire after ~30 days (assuming 5 seconds per ledger)
const thirtyDays = 30 * 24 * 60 * 60 / 5; // ~518,400 ledgers
const ttl = thirtyDays;

// Renew before expiry
await renewSubscription(contract, keypair, subscriptionId, 518400);
```

## TypeScript Usage

These examples are written in JavaScript but work with TypeScript. To use with TypeScript:

1. Rename files from `.js` to `.ts`
2. Install type definitions (already in `package.json`):
   ```bash
   npm install --save-dev @types/node
   ```
3. Add type annotations:

```typescript
import * as StellarSDK from '@stellar/stellar-sdk';

interface SubscriptionConfig {
  watchedContract: string;
  topics: Buffer[];
  channel: 'Webhook' | 'InApp' | 'OnChain';
  webhookUrl: string;
  ttl: number;
}

async function createSubscription(
  contract: StellarSDK.Contract,
  keypair: StellarSDK.Keypair,
  config: SubscriptionConfig
): Promise<number> {
  // Implementation
}
```

## Testing

Before running these examples:

1. Ensure you have testnet XLM in your account
2. Deploy the StellarNotify contract to testnet
3. Initialize the contract with `initialise()`
4. Update the configuration constants in each example file

## Resources

- [Stellar SDK Documentation](https://stellar.github.io/js-stellar-sdk/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [StellarNotify Contract](https://github.com/Stellarnotify/stellarnotify-contract)
- [StellarNotify Backend](https://github.com/Stellarnotify/stellarnotify-backend)
