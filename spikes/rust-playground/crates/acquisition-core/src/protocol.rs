//! JSON-lines protocol spoken over the Unix socket.
//!
//! One JSON object per line in each direction. A connection that has sent
//! `subscribe` also receives unsolicited `event` lines interleaved with
//! responses; clients must be prepared to skip or dispatch them.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::job::{JobId, JobInfo, Outcome, Priority};
use crate::rails::RailsStatus;
use crate::ratelimit::{DegradedEndpoint, PolicyStatus, RuleStatus, SendRecord};

/// One unit of work to quote, in the daemon's own job vocabulary — exactly
/// the `(kind, params)` a `Submit` would carry (a plan action renders it).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuoteJob {
    pub kind: String,
    #[serde(default)]
    pub params: Value,
}

/// A read-only, non-reserving projection over current daemon knowledge
/// (CONTEXT.md, decided 2026-08-31): what the quoted work would meet at the
/// choke point as of `observed_at`. Nothing is sent, reserved, or
/// remembered — applying later may receive a different schedule (`eta_for`
/// is an estimate, not a promise) — and the quote names what it does not
/// cover (`not_covered`, per-scope `notes`) rather than claiming
/// completeness. A `RefreshPlan` may embed one verbatim as optional
/// enrichment, so this shape is part of the plan schema too: changing it
/// is a plan-schema event, not a silent edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quote {
    /// Unix seconds when the daemon took this projection.
    pub observed_at: i64,
    pub provider: String,
    /// The canonical account the work was keyed under, when any of it
    /// resolved to one (the same selector rules as `Submit`).
    pub account: Option<String>,
    /// The live-test rails halt in force, if any: nothing sends until
    /// `reset-tripwire`, so every estimate below is a floor.
    pub halted: Option<String>,
    /// One entry per scheduling scope. Estimates stay per policy/window
    /// and scope, never one scalar.
    pub scopes: Vec<QuoteScope>,
    /// Sends and schedules deliberately outside every estimate, named.
    pub not_covered: Vec<String>,
}

/// The quoted work on one scheduling scope — the dispatcher's key: a
/// learned policy's state key (`stash-request-limit@Alice#1234`), or the
/// bare endpoint key while the route's policy is unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuoteScope {
    pub key: String,
    /// The endpoint keys of the quoted work under this scope.
    pub endpoints: Vec<String>,
    /// Quoted requests on this scope.
    pub requests: u64,
    /// Non-terminal jobs already in this daemon on the same scope; the
    /// estimate puts them ahead of the quoted work.
    pub queued_ahead: u64,
    /// The governing policy's state key, once learned from headers.
    pub policy: Option<String>,
    /// The policy's rules as last reported — headroom is read per window
    /// (`hits` against `max_hits`), never one scalar.
    pub rules: Vec<RuleStatus>,
    /// Seconds (at `observed_at`) since the policy's headers were last
    /// seen — the basis of `rules`, which is that much older than the
    /// quote itself. Rules, this age, and the ETA are read under one
    /// limiter lock, so they describe the same instant.
    pub observed_seconds_ago: Option<u64>,
    /// Seconds until the last quoted request on this scope could dispatch,
    /// simulating the pacing rule forward over current limiter state and
    /// the queue. An estimate, never a promise or a reservation. `None`:
    /// unquotable until the policy is learned (see `notes`).
    pub eta_seconds: Option<u64>,
    /// What this scope's numbers cannot see, named: an unlearned policy,
    /// a degraded probe cooldown, a declared-policyless route.
    pub notes: Vec<String>,
}

/// A recent daemon-side error (job failures, auth/keyring trouble), for the
/// dashboard. Everything in here is also in the daemon log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub seconds_ago: f64,
    pub message: String,
}

/// One live session, for `auth status` and the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatus {
    pub username: String,
    pub access_expires_in_seconds: Option<u64>,
    /// "ok" or why this session is memory-only.
    pub keyring: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "req", rename_all = "snake_case")]
pub enum Request {
    /// Always sent first. The daemon answers with `hello`; the client is
    /// responsible for killing + respawning a version-mismatched daemon.
    Hello {
        client_version: String,
    },
    Submit {
        kind: String,
        params: Value,
        priority: Priority,
        submitted_by: String,
        /// Which account to run as: a username (with or without its
        /// `#discriminator`) or uuid. Omitted means the live session's
        /// account. Refused if it does not name the live session.
        #[serde(default)]
        account: Option<String>,
    },
    Status {
        id: JobId,
    },
    Result {
        id: JobId,
    },
    Cancel {
        id: JobId,
    },
    SetPriority {
        id: JobId,
        priority: Priority,
    },
    List,
    Subscribe,
    /// Begin an OAuth login. The daemon sets up PKCE + a loopback redirect
    /// listener and returns the URL for the user's browser; clients then poll
    /// `auth_status` until `pending` clears.
    AuthStart,
    AuthStatus,
    /// Active verification: prove the session works by obtaining a valid
    /// access token (refreshing through the provider if needed), unlike
    /// `auth_status` which only reports local belief.
    AuthCheck {
        /// Which session to prove; required when several are live.
        #[serde(default)]
        account: Option<String>,
    },
    /// Drop a session and its keyring entry. Omitted or the live session's
    /// account: the live session. Another known account: only its keyring
    /// entry (the index marks it not persisted); the live session stays.
    AuthLogout {
        #[serde(default)]
        account: Option<String>,
    },
    /// A read-only, non-reserving projection: what would sending this
    /// work cost and wait, as of now (see [`Quote`]). Deliberately its
    /// own request, never a flag on `Submit` — `Submit`'s contract is
    /// loaded with id/persistence/rollback semantics a projection must
    /// not inherit.
    Quote {
        jobs: Vec<QuoteJob>,
        /// The same selector rules as `Submit`: resolved before anything
        /// is projected, refused when ambiguous — a quote must key the
        /// same limiter state a submit would.
        #[serde(default)]
        account: Option<String>,
    },
    DaemonStatus,
    DaemonStop,
    /// Clear the live-test rails' tripwire and ceiling halt (`LIVE-TESTING.md`).
    ResetTripwire,
    /// Everything the live dashboard renders, in one round-trip: daemon
    /// vitals, auth state, limiter policies, jobs, HTTP sends, recent errors.
    Dashboard,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "resp", rename_all = "snake_case")]
pub enum Response {
    Hello {
        daemon_version: String,
        pid: u32,
        /// "mock" or "ggg"; clients respawn a daemon running the wrong mode.
        provider: String,
    },
    Submitted {
        id: JobId,
    },
    Status {
        job: JobInfo,
    },
    Result {
        id: JobId,
        outcome: Outcome,
    },
    Ack,
    Jobs {
        jobs: Vec<JobInfo>,
    },
    Subscribed,
    AuthUrl {
        authorize_url: String,
    },
    Auth {
        /// At least one session is live.
        logged_in: bool,
        /// A login flow is in progress (waiting on the browser, or on the
        /// login's own profile fetch).
        pending: bool,
        /// The account the most recent login flow registered. At most one
        /// of `login_ok`/`login_error` is set; both absent means no flow
        /// has finished since the daemon started (or one just began).
        /// Aggregate state cannot answer "did *my* login work" — another
        /// account's live session must not read as this flow's success.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        login_ok: Option<String>,
        /// Why the most recent login flow failed.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        login_error: Option<String>,
        /// The most recently logged-in account — informational; the daemon
        /// never selects by it.
        username: Option<String>,
        /// Of `username`'s session.
        access_expires_in_seconds: Option<u64>,
        /// "ok" or an error description; sessions still work in-memory when
        /// the keyring is unavailable.
        keyring: String,
        provider: String,
        /// Every live session.
        #[serde(default)]
        accounts: Vec<SessionStatus>,
    },
    DaemonStatus {
        pid: u32,
        version: String,
        provider: String,
        uptime_seconds: u64,
        connections: usize,
        jobs_waiting: usize,
        jobs_running: usize,
        /// Rate-limit policies learned from responses so far.
        policies_known: usize,
        in_flight: usize,
        max_in_flight: usize,
        /// Live-test rails state (tripwire, ceiling, journal).
        rails: RailsStatus,
        /// "ok" or the keyring failure; a failed save after refresh-token
        /// rotation leaves the session memory-only (LIVE-TESTING.md R7).
        keyring: String,
    },
    Stopping,
    Quote {
        quote: Quote,
    },
    Dashboard {
        pid: u32,
        version: String,
        provider: String,
        uptime_seconds: u64,
        connections: usize,
        logged_in: bool,
        username: Option<String>,
        access_expires_in_seconds: Option<u64>,
        keyring: String,
        /// Requests currently holding a slot, and the burst bound (P-B).
        in_flight: usize,
        max_in_flight: usize,
        /// Sorted by policy name.
        policies: Vec<PolicyStatus>,
        /// Endpoints that answered without any X-Rate-Limit headers.
        policyless_endpoints: Vec<String>,
        /// Endpoints closed by a failed/degraded probe (N20), with cooldown.
        degraded_endpoints: Vec<DegradedEndpoint>,
        jobs: Vec<JobInfo>,
        /// Newest first.
        sends: Vec<SendRecord>,
        rails: RailsStatus,
        /// Newest first.
        errors: Vec<ErrorRecord>,
    },
    Error {
        message: String,
    },
    /// Unsolicited; only on subscribed connections.
    Event {
        job: JobInfo,
    },
}
