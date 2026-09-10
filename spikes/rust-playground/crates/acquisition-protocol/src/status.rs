//! The status documents the daemon reports on the wire — inside
//! `Response::DaemonStatus`, `Response::Dashboard` and a `Quote` — as a
//! frontend reads them. Documents, not state: the daemon builds each one
//! from its limiter and rails (`ratelimit.rs`'s `rule_statuses`,
//! `ChokePoint::policy_statuses`, `Rails::status`), and nothing here knows
//! how. Moved here whole from `rails.rs` and `ratelimit.rs` with the
//! protocol crate (DAEMON-SPLIT-SLICE.md, step 1); the fixtures under
//! `tests/fixtures/wire/` pin their shape through the responses that carry
//! them.

use serde::{Deserialize, Serialize};

/// The live-test rails, for `daemon status` and the dashboard
/// (`LIVE-TESTING.md`, "Rails").
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RailsStatus {
    pub tripwire_enabled: bool,
    /// Why sends are refused, if they are.
    pub halted: Option<String>,
    pub refresh_failed: Option<String>,
    pub sends: u64,
    pub max_sends: Option<u64>,
    pub journal: Option<String>,
}

/// A degraded endpoint (N20), for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradedEndpoint {
    pub endpoint: String,
    pub seconds_left: f64,
    pub reason: String,
}

// `WindowStatus`/`RuleStatus` also travel inside a `Quote`, which a
// `RefreshPlan` may embed — so they are `Eq` and parse strictly, like
// everything else a serialized plan carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowStatus {
    pub hits: u32,
    pub max_hits: u32,
    pub period_secs: u64,
    pub restriction_secs: u64,
    pub restricted_secs: u64,
    pub bucket_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleStatus {
    pub name: String,
    pub windows: Vec<WindowStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatus {
    pub policy: String,
    pub endpoints: Vec<String>,
    pub rules: Vec<RuleStatus>,
    pub next_safe_in_seconds: f64,
    pub last_observed_seconds_ago: f64,
    pub history_len: usize,
    pub retry_after_secs: Option<u64>,
    /// The raw `x-rate-limit-*` / `retry-after` headers last seen.
    pub headers: serde_json::Value,
}

/// One HTTP request the choke point actually sent (there is no other way to
/// send one), for the dashboard's sent-requests table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRecord {
    pub seconds_ago: f64,
    pub endpoint: String,
    pub method: String,
    pub url: String,
    /// HTTP status ("200 OK") or the transport error.
    pub outcome: String,
    pub ok: bool,
}
