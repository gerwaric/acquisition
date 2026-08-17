//! JSON-lines protocol spoken over the Unix socket.
//!
//! One JSON object per line in each direction. A connection that has sent
//! `subscribe` also receives unsolicited `event` lines interleaved with
//! responses; clients must be prepared to skip or dispatch them.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::job::{JobId, JobInfo, Outcome, Priority};

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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "resp", rename_all = "snake_case")]
pub enum Response {
    Hello {
        daemon_version: String,
        pid: u32,
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
    },
    DaemonStatus {
        pid: u32,
        version: String,
        uptime_seconds: u64,
        connections: usize,
        jobs_waiting: usize,
        jobs_running: usize,
        tokens_available: u32,
        oauth_tokens_available: u32,
    },
    Stopping,
    Error {
        message: String,
    },
    /// Unsolicited; only on subscribed connections.
    Event {
        job: JobInfo,
    },
}
