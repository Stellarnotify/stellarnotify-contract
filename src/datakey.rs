use soroban_sdk::{contracttype, Address};

/// All storage keys used by the StellarNotify contract.
///
/// Soroban has three storage tiers:
///
/// - `instance`    — lives as long as the contract instance. Used for small,
///                   frequently-read values like the global counter and config.
/// - `persistent`  — survives ledger archival if TTL is extended. Used for
///                   subscription data and indexes that must outlive a single session.
/// - `temporary`   — automatically removed when TTL expires. Not used here.
///
/// Key layout:
///
///   Instance storage
///   ├── SubCounter          — global u64 subscription ID counter
///   └── Config              — ProtocolConfig struct
///
///   Persistent storage
///   ├── Sub(u64)            — individual Subscription by ID
///   ├── OwnerSubs(Address)  — Vec<u64> of subscription IDs owned by an address
///   └── WatcherSubs(Address)— Vec<u64> of subscription IDs watching a contract
#[contracttype]
pub enum DataKey {
    /// Global subscription ID counter. Stored in instance storage.
    SubCounter,

    /// Individual subscription data, keyed by its unique u64 ID.
    /// Stored in persistent storage.
    Sub(u64),

    /// Index: all subscription IDs owned by a given wallet address.
    /// Stored as a `Vec<u64>` in persistent storage.
    OwnerSubs(Address),

    /// Index: all subscription IDs watching a given contract address.
    /// Stored as a `Vec<u64>` in persistent storage.
    WatcherSubs(Address),

    /// Protocol-level configuration (admin, limits, paused flag).
    /// Stored in instance storage.
    Config,
}
