use soroban_sdk::contracterror;

/// All error codes returned by the StellarNotify contract.
///
/// Every variant maps to a unique `u32` code so that clients
/// (SDKs, frontends, the backend) can match on the numeric value
/// without parsing strings.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum NotifyError {
    /// `initialise()` was called on a contract that is already set up.
    AlreadyInitialised = 1,

    /// A function was called before `initialise()` has been run.
    NotInitialised = 2,

    /// The caller is not the admin address stored in `ProtocolConfig`.
    Unauthorised = 3,

    /// The requested subscription ID does not exist in storage.
    SubNotFound = 4,

    /// The caller is not the owner of the targeted subscription.
    NotOwner = 5,

    /// The owner has reached the `max_per_owner` subscription cap.
    LimitExceeded = 6,

    /// The requested `ttl_ledgers` exceeds `ProtocolConfig.max_ttl`.
    TtlExceeded = 7,

    /// The protocol is paused; no new subscriptions can be created.
    Paused = 8,

    /// The subscription's TTL has passed; it cannot be resumed.
    Expired = 9,

    /// The `topics` vector exceeds the maximum allowed length of 10.
    TooManyTopics = 10,

    /// The `endpoint_ref` byte slice is empty.
    EmptyEndpoint = 11,
}
