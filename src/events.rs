use soroban_sdk::{symbol_short, Address, Env};

/// Emitted when a new subscription is successfully created.
///
/// Topics : `(symbol "sub_new", owner)`
/// Data   : `(id, watched_contract)`
pub fn sub_created(env: &Env, id: u64, owner: &Address, watched: &Address) {
    env.events()
        .publish((symbol_short!("sub_new"), owner.clone()), (id, watched.clone()));
}

/// Emitted when a subscription is permanently cancelled.
///
/// Topics : `(symbol "sub_cncl", owner)`
/// Data   : `id`
pub fn sub_cancelled(env: &Env, id: u64, owner: &Address) {
    env.events()
        .publish((symbol_short!("sub_cncl"), owner.clone()), id);
}

/// Emitted when a subscription is paused by its owner.
///
/// Topics : `(symbol "sub_paus", owner)`
/// Data   : `id`
pub fn sub_paused(env: &Env, id: u64, owner: &Address) {
    env.events()
        .publish((symbol_short!("sub_paus"), owner.clone()), id);
}

/// Emitted when a paused subscription is resumed by its owner.
///
/// Topics : `(symbol "sub_rsm", owner)`
/// Data   : `id`
pub fn sub_resumed(env: &Env, id: u64, owner: &Address) {
    env.events()
        .publish((symbol_short!("sub_rsm"), owner.clone()), id);
}

/// Emitted when a subscription's endpoint reference is updated.
///
/// Topics : `(symbol "sub_ep", owner)`
/// Data   : `id`
pub fn sub_endpoint_updated(env: &Env, id: u64, owner: &Address) {
    env.events()
        .publish((symbol_short!("sub_ep"), owner.clone()), id);
}

/// Emitted when a subscription's TTL is extended via `renew_sub()`.
///
/// Topics : `(symbol "sub_renew", owner)`
/// Data   : `(id, new_expires_at)`
pub fn sub_renewed(env: &Env, id: u64, owner: &Address, new_expires_at: u32) {
    env.events()
        .publish((symbol_short!("sub_renew"), owner.clone()), (id, new_expires_at));
}

/// Emitted when the protocol configuration is updated by the admin.
///
/// Topics : `(symbol "cfg_upd", admin)`
/// Data   : `()`
pub fn config_updated(env: &Env, admin: &Address) {
    env.events()
        .publish((symbol_short!("cfg_upd"), admin.clone()), ());
}

/// Emitted when the protocol is paused or unpaused by the admin.
///
/// Topics : `(symbol "proto_ps",)`
/// Data   : `paused (bool)`
pub fn protocol_paused(env: &Env, paused: bool) {
    env.events().publish((symbol_short!("proto_ps"),), paused);
}

/// Emitted when the admin role is transferred to a new address.
///
/// Topics : `(symbol "adm_xfr", old_admin)`
/// Data   : `new_admin`
pub fn admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events()
        .publish((symbol_short!("adm_xfr"), old_admin.clone()), new_admin.clone());
}

/// Emitted once when an OnChain subscription is first activated.
///
/// Topics : `(symbol "oc_live", owner)`
/// Data   : `(id, watched_contract)`
pub fn onchain_sub_activated(env: &Env, id: u64, owner: &Address, watched: &Address) {
    env.events()
        .publish((symbol_short!("oc_live"), owner.clone()), (id, watched.clone()));
}
