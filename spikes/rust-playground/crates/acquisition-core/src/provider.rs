//! OAuth provider + API host configuration: the mock, or the real GGG.
//!
//! Real mode is opt-in via `ACQ_GGG=1` and uses the existing "acquisition"
//! registration (same client id, callback path, scopes, and user-agent as the
//! shipped C++ app — CONTEXT invariant 4). Everything else in the daemon is
//! provider-agnostic; this struct is the whole difference. The provider
//! *names* and `ggg_mode()` are the protocol crate's (`provider.rs` there):
//! which provider a process wants is part of the handshake.

/// Same shape as the shipped app (APP_NAME "/" APP_VERSION " (contact: " EMAIL ")")
/// and the same registration, but a distinct version so anything GGG sees from
/// live testing is attributable to this spike, not to a shipped release.
pub const USER_AGENT: &str = "acquisition/1.0.0-alpha.1 (contact: gerwaric@gmail.com)";

/// The C++ app's scope set plus `account:profile`, which the registration
/// offers but the shipped app never requested (2026-08-29). A user of the
/// spike consents to the four together at `acq auth`.
pub const SCOPES: &[&str] = &[
    "account:profile",
    "account:leagues",
    "account:stashes",
    "account:characters",
];

/// The callback path registered for the "acquisition" client. The mock
/// provider redirects wherever it's told, so both modes share it.
pub const CALLBACK_PATH: &str = "/auth/path-of-exile";

// Until step 2 of the daemon split switches the frontends to the
// protocol crate; step 2 deletes this line.
pub use acquisition_protocol::provider::ggg_mode;
use acquisition_protocol::provider::{GGG, MOCK};

pub struct Provider {
    /// [`MOCK`] or [`GGG`] — shown in the handshake so clients can detect a
    /// daemon running in the wrong mode.
    pub name: &'static str,
    pub authorize_url: String,
    pub token_url: String,
    pub api_base: String,
    pub client_id: &'static str,
    /// Separate keyring entries per provider so a mock refresh token can
    /// never be sent to GGG (or vice versa).
    pub keyring_service: &'static str,
}

impl Provider {
    pub fn mock(base_url: &str) -> Provider {
        Provider {
            name: MOCK,
            authorize_url: format!("{base_url}/authorize"),
            token_url: format!("{base_url}/token"),
            api_base: base_url.to_string(),
            client_id: "acquisition-playground",
            keyring_service: "acquisition-playground",
        }
    }

    pub fn ggg() -> Provider {
        Provider {
            name: GGG,
            authorize_url: "https://www.pathofexile.com/oauth/authorize".into(),
            token_url: "https://www.pathofexile.com/oauth/token".into(),
            api_base: "https://api.pathofexile.com".into(),
            client_id: "acquisition",
            keyring_service: "acquisition-rust-spike",
        }
    }

    pub fn is_real(&self) -> bool {
        self.name == GGG
    }
}
