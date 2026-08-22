//! JSON-lines protocol spoken over the Unix socket.
//!
//! One JSON object per line in each direction. A connection that has sent
//! `subscribe` also receives unsolicited `event` lines interleaved with
//! responses; clients must be prepared to skip or dispatch them.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::job::{JobId, JobInfo, Outcome, Priority};
use crate::rails::RailsStatus;
use crate::ratelimit::{DegradedEndpoint, PolicyStatus, SendRecord};

/// A recent daemon-side error (job failures, auth/keyring trouble), for the
/// dashboard. Everything in here is also in the daemon log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorRecord {
    pub seconds_ago: f64,
    pub message: String,
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
    AuthCheck,
    AuthLogout,
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
        /// Defaulted so a pre-provider daemon parses as a mismatch, not an error.
        #[serde(default)]
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
        logged_in: bool,
        /// A login flow is in progress (waiting on the browser).
        pending: bool,
        username: Option<String>,
        access_expires_in_seconds: Option<u64>,
        /// "ok" or an error description; sessions still work in-memory when
        /// the keyring is unavailable.
        keyring: String,
        #[serde(default)]
        provider: String,
    },
    DaemonStatus {
        pid: u32,
        version: String,
        #[serde(default)]
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
        #[serde(default)]
        rails: RailsStatus,
    },
    Stopping,
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
        #[serde(default)]
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
