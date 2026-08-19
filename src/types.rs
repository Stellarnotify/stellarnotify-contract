use soroban_sdk::{contracttype, Address, Bytes, Vec};

/// Delivery channels supported by StellarNotify.
///
/// - `Webhook`  — backend dispatches an HTTP POST to the registered endpoint URL.
/// - `InApp`    — backend publishes to a Redis channel; consumed by the SSE endpoint.
/// - `OnChain`  — contract re-emits a Soroban event; any listener on the network can react.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Channel {
    Webhook,
    InApp,
    OnChain,
}

/// A single subscription entry stored in persistent contract storage.
///
/// Every field is stored on-chain. The `endpoint_ref` is a SHA-256 hash of
/// the real webhook URL — the URL itself is stored privately off-chain in the
/// backend's `endpoint_registry` table.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Subscription {
    /// Wallet address that owns this subscription.
    pub owner: Address,
    /// The Soroban contract address whose events are being watched.
    pub watched_contract: Address,
    /// Optional list of event topic prefixes to filter on.
    /// Empty = deliver all events from the watched contract.
    pub topics: Vec<Bytes>,
    /// How the notification should be delivered.
    pub channel: Channel,
    /// SHA-256 hex hash of the webhook URL (or arbitrary ref for other channels).
    pub endpoint_ref: Bytes,
    /// Whether the subscription is currently active.
    pub active: bool,
    /// Ledger sequence number at which this subscription was created.
    pub created_at: u32,
    /// Ledger sequence number at which this subscription expires.
    /// 0 means no expiry.
    pub expires_at: u32,
}

/// Lightweight summary returned by list queries.
/// Avoids deserialising the full `topics` and `endpoint_ref` vectors
/// when only metadata is needed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct SubscriptionSummary {
    /// Subscription ID.
    pub id: u64,
    /// Owner wallet address.
    pub owner: Address,
    /// Watched contract address.
    pub watched_contract: Address,
    /// Whether the subscription is currently active.
    pub active: bool,
    /// Delivery channel.
    pub channel: Channel,
    /// Expiry ledger (0 = no expiry).
    pub expires_at: u32,
}

/// Protocol-level configuration stored in instance storage.
///
/// Only the admin can mutate this. It is read on every `subscribe()` call
/// to enforce rate limits and TTL caps.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProtocolConfig {
    /// Maximum number of active subscriptions allowed per owner address.
    pub max_per_owner: u32,
    /// Maximum TTL in ledgers allowed when creating a subscription.
    /// 0 means no cap (subscriptions may be permanent).
    pub max_ttl: u32,
    /// The address with admin privileges (update config, pause, transfer).
    pub admin: Address,
    /// When true, no new subscriptions can be created.
    pub paused: bool,
}
