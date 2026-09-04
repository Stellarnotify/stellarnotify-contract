# StellarNotify Client Integration Examples

This directory contains example code demonstrating how to interact with the StellarNotify contract from various programming languages and environments.

## Directory Structure

```
examples/
├── javascript/        # JavaScript/TypeScript examples using Stellar SDK
│   ├── basic-subscribe.js
│   ├── manage-subscriptions.js
│   └── query-subscriptions.js
└── rust/             # Rust client examples
    ├── Cargo.toml
    └── src/
        ├── basic_subscribe.rs
        ├── manage_subscriptions.rs
        └── query_subscriptions.rs
```

## Prerequisites

### JavaScript Examples
- Node.js 18+ or Bun
- `@stellar/stellar-sdk` package
- A funded Stellar testnet account

### Rust Examples
- Rust toolchain (stable)
- `stellar-strkey` for address encoding
- A funded Stellar testnet account

## Quick Start

### JavaScript

```bash
cd examples/javascript
npm install
# Edit the configuration constants at the top of each file
node basic-subscribe.js
```

### Rust

```bash
cd examples/rust
cargo build
cargo run --example basic_subscribe
```

## Common Operations

All examples demonstrate the following operations:

1. **Subscribing** - Create a new subscription to watch contract events
2. **Querying** - Retrieve subscription details and lists
3. **Managing** - Pause, resume, renew, and cancel subscriptions
4. **Admin** - Initialize and configure the contract (admin only)

## Contract Addresses

- **Testnet**: Update `CONTRACT_ID` constant in each example file
- **Mainnet**: Coming soon

## Security Notes

- Never commit private keys to version control
- Use environment variables for sensitive data in production
- The `endpoint_ref` field stores a SHA-256 hash, not the raw URL
- Always verify transaction results before considering operations complete

## Support

For issues or questions:
- [Documentation](https://github.com/Stellarnotify/stellarnotify-docs)
- [Backend](https://github.com/Stellarnotify/stellarnotify-backend)
- [Frontend](https://github.com/Stellarnotify/stellarnotify-frontend)
