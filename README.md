# stellarnotify-contract

> On-chain event subscription registry for the Stellar/Soroban ecosystem.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Soroban SDK](https://img.shields.io/badge/soroban--sdk-22.0.7-purple)](https://docs.rs/soroban-sdk)

## What it does

StellarNotify is a Soroban smart contract that acts as a **public, permissionless
subscription registry**. Any wallet can subscribe to events emitted by any
Soroban contract and choose how to receive notifications:

| Channel | Delivery |
|---------|----------|
| `Webhook` | HTTP POST to a registered endpoint (URL stored privately off-chain) |
| `InApp` | Server-Sent Events stream via the StellarNotify backend |
| `OnChain` | Re-emitted as a Soroban event — consumable by other contracts |

Subscriptions are stored in Soroban **persistent storage** with TTL bumping
on every access so they never silently expire. The contract holds no tokens
and has no custody over user funds.

## Why it exists

Stellar's RPC `getEvents` endpoint retains data for **at most 7 days**.
Every dapp that needs to react to contract events must build and maintain
its own polling infrastructure. StellarNotify eliminates this duplicated
work by providing a shared, on-chain registry that the
[StellarNotify backend](https://github.com/Stellarnotify/stellarnotify-backend)
subscribes to on behalf of all registered wallets.

## Contract functions

### Initialisation
| Function | Description |
|---|---|
| `initialise(admin, max_per_owner, max_ttl)` | One-time setup — must be called once after deployment |

### Subscription management (owner-only)
| Function | Description |
|---|---|
| `subscribe(owner, watched_contract, topics, channel, endpoint_ref, ttl_ledgers)` | Create a subscription, returns `u64` ID |
| `cancel(owner, id)` | Permanently delete a subscription |
| `pause_sub(owner, id)` | Pause deliveries — keep data |
| `resume_sub(owner, id)` | Resume a paused subscription |
| `update_endpoint_ref(owner, id, new_endpoint)` | Rotate the webhook URL hash |
| `renew_sub(owner, id, add_ttl_ledgers)` | Extend TTL without cancelling |

### Queries (read-only)
| Function | Description |
|---|---|
| `get_sub(id)` | Full subscription data |
| `list_by_owner(owner)` | All subscription IDs for a wallet |
| `list_by_contract(watched)` | All subscription IDs watching a contract |
| `list_summaries_by_owner(owner)` | Lightweight summaries for dashboard display |
| `get_config()` | Current protocol configuration |
| `get_version()` | Contract version string |

### Admin-only
| Function | Description |
|---|---|
| `update_config(admin, max_per_owner, max_ttl)` | Update protocol limits |
| `set_paused(admin, paused)` | Emergency pause / unpause |
| `transfer_admin(admin, new_admin)` | Hand off the admin role |

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/cli) v22+

```bash
rustup target add wasm32-unknown-unknown
```

## Build

```bash
stellar contract build
```

Output: `target/wasm32-unknown-unknown/release/stellarnotify_contract.wasm`

## Test

```bash
cargo test --features testutils
```

All tests must pass before deployment.

## Deploy to testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/stellarnotify_contract.wasm \
  --source YOUR_ADDRESS \
  --network testnet
```

## Initialise

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source YOUR_ADDRESS \
  --network testnet \
  -- initialise \
  --admin YOUR_ADDRESS \
  --max_per_owner 20 \
  --max_ttl 0
```

`max_ttl = 0` means no TTL cap — subscriptions may be permanent.

## Create your first subscription

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source YOUR_ADDRESS \
  --network testnet \
  -- subscribe \
  --owner YOUR_ADDRESS \
  --watched_contract <CONTRACT_TO_WATCH> \
  --topics '[]' \
  --channel '{"Webhook": null}' \
  --endpoint_ref '<SHA256_HEX_OF_YOUR_URL>' \
  --ttl_ledgers 0
```

## Project structure

```
src/
├── lib.rs          # Module declarations and re-exports
├── contract.rs     # Public interface — thin delegation layer
├── admin.rs        # Admin functions
├── subscribe.rs    # Owner functions
├── storage.rs      # Persistent/instance storage helpers with TTL management
├── validation.rs   # Input validation for subscribe()
├── events.rs       # Soroban event emission helpers
├── types.rs        # Subscription, Channel, ProtocolConfig, SubscriptionSummary
├── datakey.rs      # DataKey enum for storage keys
├── errors.rs       # NotifyError enum (11 variants)
└── test.rs         # Unit and integration tests
```

## Security model

- **No fund custody** — the contract holds no tokens.
- **Owner-only mutations** — every subscription function verifies the caller is the owner.
- **Admin isolation** — the admin can update limits and pause the protocol, but cannot touch individual user subscriptions.
- **Endpoint privacy** — webhook URLs are never stored on-chain. Only the SHA-256 hash is stored.
- **TTL management** — every storage read and write bumps TTL to prevent accidental archival.

## Related repos

- [stellarnotify-backend](https://github.com/Stellarnotify/stellarnotify-backend) — event ingester, webhook dispatcher, SSE broadcaster
- [stellarnotify-frontend](https://github.com/Stellarnotify/stellarnotify-frontend) — Next.js dashboard
- [stellarnotify-docs](https://github.com/Stellarnotify/stellarnotify-docs) — full documentation site

## License

[MIT](LICENSE) © 2026 StellarNotify Contributors
