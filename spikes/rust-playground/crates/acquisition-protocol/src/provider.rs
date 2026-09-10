//! Which provider a process is for — part of the handshake (C10): a
//! daemon's `hello` names the provider it serves, and a client uses only
//! a daemon on the provider it wants itself. The names and the one knob
//! that selects between them live here; the providers themselves (URLs,
//! client id, user-agent, keyring service) are the daemon's
//! (`acquisition-core/src/provider.rs`), because only the daemon sends.

/// The in-process mock provider: the default, and the only provider
/// anything reaches without a human setting `ACQ_GGG=1`.
pub const MOCK: &str = "mock";

/// The real GGG API, under the existing "acquisition" registration.
pub const GGG: &str = "ggg";

/// True when this process (daemon or CLI) should be in real-GGG mode.
/// Deliberately strict: only the exact value "1" counts.
pub fn ggg_mode() -> bool {
    std::env::var("ACQ_GGG").is_ok_and(|v| v == "1")
}

/// The provider this process wants a daemon to serve: [`GGG`] under
/// `ACQ_GGG=1`, [`MOCK`] otherwise. What the client compares the daemon's
/// `hello` against.
pub fn wanted() -> &'static str {
    if ggg_mode() { GGG } else { MOCK }
}
