//! JSON-lines protocol spoken over the Unix socket (C9): one JSON object
//! per line in each direction, every line under [`MAX_FRAME_BYTES`].
//!
//! Two planes share every connection:
//!
//! - **The bootstrap plane** — [`Bootstrap`] and [`BootstrapReply`]:
//!   `hello` and `daemon_stop`, read on both sides outside the versioned
//!   enums, keyed on the `req`/`resp` name alone with every other field
//!   optional and unknown fields ignored. Its frames never change shape:
//!   a client and a daemon of any two runtime revisions can always
//!   identify each other (C10 compares what `hello` carries) and a human
//!   can always stop a daemon across a mismatch.
//! - **The versioned plane** — [`Request`] and [`Response`]: everything
//!   else, single-version on purpose (C10). A change here moves the
//!   runtime revision, and `tests/fixtures/wire/` (one document per
//!   variant, `tests/wire.rs`) makes it a diff a reviewer sees.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/daemon.md` for this
//! area (`C<n>`); what follows is the entry's full text as recorded there,
//! kept beside the code that implements it.
//!
//! ## C85 — Connection semantics
//!
//! **C85 — Connection semantics: 1 request in flight per connection; a
//! subscription uses a dedicated connection — hello/hello,
//! subscribe/subscribed, then events only, no more requests; events are
//! invalidation hints, never a complete stream — a subscriber snapshots
//! after subscribing, treats events as invalidations and re-reads before
//! relying on its view, is told `resync_required` when it lagged, and
//! after that or any disconnect subscribes and snapshots again; a frame
//! has a bound, and an oversize or malformed one is answered with an
//! error, not a closed socket; `hello` and `daemon_stop` are a stable
//! plane every version parses.** *Why:* a client must see losses to
//! recover from them. *Pinned:* `acquisition-core/tests/wire.rs`,
//! `acquisition-core/tests/contract.rs`. Ruled 2026-09-09.
//!
//! ## C85 — as built
//!
//! Every connection opens with the hello exchange. A request connection
//! (`client::Client`) writes one request and reads exactly one response
//! before the next; the daemon answers a connection's frames in order,
//! one at a time. A subscription connection (`client::Subscription`)
//! carries hello, `subscribe`/`subscribed`, then only `event` and
//! `resync_required` frames; a versioned request after `subscribed` is
//! answered `bad_request` and not performed (the bootstrap plane still
//! works there: a human can stop a daemon over any connection). The
//! daemon sends `resync_required { missed }` when its event channel
//! (capacity 256) overran this subscriber — the count is what the
//! subscriber did not see — and the subscriber then re-reads its
//! snapshot over a request connection. Each side reads a frame up to
//! [`MAX_FRAME_BYTES`]; a longer one is discarded through its newline
//! and, on the daemon, answered `bad_request` — as is a frame that is
//! not JSON, not UTF-8, or not a request of this version — and the
//! connection stays. The frame reader is `crate::frame`. The sequence a
//! subscriber follows is `acq jobs --watch` (`acquisition-cli/src/main.rs`):
//! subscribe, then `list` over a request connection, then every event is a
//! reason to re-read, and after `resync_required` or a disconnect,
//! subscribe and `list` again.
//!
//! ## Errors on the wire
//!
//! `Response::Error` carries a closed [`ErrorKind`] beside its message
//! (C47: stable kinds, structured; C53: the addition is additive). The
//! message stays useful on its own — a kind is what a frontend branches
//! on, never what it prints instead. Every daemon error site is
//! classified in `daemon.rs` at the point the error is made
//! (`Refusal`), so a site cannot reach the wire unclassified; the audit
//! that fixed the vocabulary is the commit that introduced it. A new
//! kind or field is a protocol change: the `error_kinds` fixture and the
//! exhaustive matches in `tests/wire.rs` refuse it silently landing.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::job::{JobId, JobInfo, Outcome, Priority};
use crate::rails::RailsStatus;
use crate::ratelimit::{DegradedEndpoint, PolicyStatus, RuleStatus, SendRecord};

/// The bound on one frame, in bytes, both directions (C85). Nothing
/// legitimate approaches it — an `apply` of a whole league's actions is
/// hundreds of kilobytes, a `result` carrying one tab's body a few
/// megabytes — so a frame that reaches it is a runaway or a rogue peer,
/// and each side stops reading it there, discards through its newline,
/// and keeps the connection.
pub const MAX_FRAME_BYTES: usize = 64 << 20;

// ---- the bootstrap plane ------------------------------------------------

/// The two requests every daemon of any revision reads (C85), sent as
/// `{"req":"hello","client_version":…}` and `{"req":"daemon_stop"}`.
/// [`Bootstrap::read`] is the lenient reader the daemon uses: it keys on
/// `req` alone and ignores every other field, so a client of a future
/// revision — extra fields, renamed fields, a shape this build has never
/// seen — still gets a `hello` back that names this daemon, and can still
/// stop it. Serialization is the exact shape the fixtures pin.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "req", rename_all = "snake_case")]
pub enum Bootstrap {
    /// Always sent first. Answered with [`BootstrapReply::Hello`]; the
    /// client decides from that what the daemon is (C10).
    Hello {
        /// The client's `VERSION_WITH_RUNTIME`; the daemon logs a mismatch.
        client_version: String,
    },
    /// Stop the daemon, whichever revision it is. Answered with
    /// [`BootstrapReply::Stopping`] before the process exits.
    DaemonStop,
}

impl Bootstrap {
    /// The bootstrap request a frame carries, if it is one: keyed on the
    /// `req` name, every other field optional, unknown fields ignored.
    /// `None` is any other frame — the versioned parser's business.
    pub fn read(frame: &[u8]) -> Option<Bootstrap> {
        let value: Value = serde_json::from_slice(frame).ok()?;
        match value.get("req")?.as_str()? {
            "hello" => Some(Bootstrap::Hello {
                client_version: value
                    .get("client_version")
                    .and_then(Value::as_str)
                    .unwrap_or(UNKNOWN)
                    .to_string(),
            }),
            "daemon_stop" => Some(Bootstrap::DaemonStop),
            _ => None,
        }
    }
}

/// What a bootstrap-plane field reads as when the peer's frame did not
/// carry it: a daemon of an unknown revision is reported, never mistaken
/// for this build's.
pub const UNKNOWN: &str = "unknown";

/// The daemon's answers on the bootstrap plane (C85):
/// `{"resp":"hello","daemon_version":…,"pid":…,"provider":…}` and
/// `{"resp":"stopping"}`. [`BootstrapReply::read`] is the client's
/// lenient reader, the mirror of [`Bootstrap::read`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "resp", rename_all = "snake_case")]
pub enum BootstrapReply {
    Hello {
        /// The daemon's `VERSION_WITH_RUNTIME` (C10).
        daemon_version: String,
        pid: u32,
        /// "mock" or "ggg"; a client uses only a daemon on its own provider.
        provider: String,
    },
    Stopping,
}

impl BootstrapReply {
    /// The bootstrap reply a frame carries, if it is one: keyed on the
    /// `resp` name; a missing `daemon_version` or `provider` reads as
    /// [`UNKNOWN`] and a missing `pid` as 0, so a daemon of another
    /// revision is identified as foreign rather than left unparsed.
    /// `None` is any other frame (a versioned `error`, say).
    pub fn read(frame: &[u8]) -> Option<BootstrapReply> {
        let value: Value = serde_json::from_slice(frame).ok()?;
        let text = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or(UNKNOWN)
                .to_string()
        };
        match value.get("resp")?.as_str()? {
            "hello" => Some(BootstrapReply::Hello {
                daemon_version: text("daemon_version"),
                pid: value
                    .get("pid")
                    .and_then(Value::as_u64)
                    .and_then(|p| u32::try_from(p).ok())
                    .unwrap_or(0),
                provider: text("provider"),
            }),
            "stopping" => Some(BootstrapReply::Stopping),
            _ => None,
        }
    }
}

/// The `message` of an error frame, read leniently (a versioned `error`
/// answering a bootstrap request may come from any revision): `None`
/// unless the frame is `{"resp":"error",…}`.
pub fn error_message(frame: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(frame).ok()?;
    if value.get("resp")?.as_str()? != "error" {
        return None;
    }
    Some(
        value
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("(no message)")
            .to_string(),
    )
}

// ---- the versioned plane ------------------------------------------------

/// Why a request failed, as a closed set a frontend can branch on (C47);
/// the `message` beside it says what happened and stays useful alone.
/// Coarse on purpose: subcategories and structured context wait for the
/// first typed consumer (the GUI slice). Every variant is produced by at
/// least one daemon site; the mapping from sites to kinds is the doc of
/// `daemon.rs`'s `Refusal`. Adding a variant is a protocol change, pinned
/// by the `error_kinds` fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// The frame was not a request the daemon could perform: not JSON,
    /// not UTF-8, over the frame bound, not a request of this version, a
    /// request missing fields — or a versioned request on a connection
    /// that has subscribed. The connection stays; nothing was done.
    BadRequest,
    /// No live session can serve the request: none at all, none for the
    /// named account, or the account's grant is dead (no refresh token,
    /// or one the provider rejected). The remedy is `acq auth`.
    NotLoggedIn,
    /// Several sessions are live and the request named none, or its
    /// selector matched more than one. The remedy is `--account`.
    AmbiguousAccount,
    /// The id names no job this daemon holds or remembers.
    UnknownJob,
    /// The job exists but its state does not admit the request: a result
    /// asked for before the job is terminal, a cancel after it is, a
    /// priority change off `waiting`. Re-read the job.
    WrongState,
    /// Well-formed work the daemon will not admit, judged before a job id
    /// exists: a kind outside a plan's vocabulary, a realm the kind's
    /// family does not take, a plan over its budget, a malformed tuple.
    /// Nothing was submitted.
    Refused,
    /// The persisted queue (`daemon.db`) failed a read or a write. A write
    /// failure is sticky (C6): the daemon refuses new work until restart.
    QueueFailed,
    /// The provider did not answer as the request needed — a token refresh
    /// that failed on transport, a 5xx, or exhausted 429 retries — with
    /// the session itself still standing. Try again later.
    Upstream,
    /// The daemon's own failure to do its part — a local resource it
    /// needed (a loopback listener, the accounts index) refused. Never an
    /// expected domain failure: those have a kind above.
    Internal,
}

impl ErrorKind {
    /// Every kind, in declaration order — the closed set the fixture pins.
    pub const ALL: [ErrorKind; 9] = [
        ErrorKind::BadRequest,
        ErrorKind::NotLoggedIn,
        ErrorKind::AmbiguousAccount,
        ErrorKind::UnknownJob,
        ErrorKind::WrongState,
        ErrorKind::Refused,
        ErrorKind::QueueFailed,
        ErrorKind::Upstream,
        ErrorKind::Internal,
    ];

    /// The wire name (`bad_request`), for logs and messages.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorKind::BadRequest => "bad_request",
            ErrorKind::NotLoggedIn => "not_logged_in",
            ErrorKind::AmbiguousAccount => "ambiguous_account",
            ErrorKind::UnknownJob => "unknown_job",
            ErrorKind::WrongState => "wrong_state",
            ErrorKind::Refused => "refused",
            ErrorKind::QueueFailed => "queue_failed",
            ErrorKind::Upstream => "upstream",
            ErrorKind::Internal => "internal",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One unit of work to quote, in the daemon's own job vocabulary — exactly
/// the `(kind, params)` a `Submit` would carry (a plan action renders it).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuoteJob {
    pub kind: String,
    #[serde(default)]
    pub params: Value,
}

/// A read-only, non-reserving projection over current daemon knowledge
/// (C40, decided 2026-08-31): what the quoted work would meet at the
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
    /// The work this quote projects: the request's job tuples echoed
    /// verbatim, in order. This is the quote's verifiable basis — a
    /// carrier can check it names exactly the work it claims to price
    /// (a plan accepts a quote only when this is its own action list),
    /// where matching totals alone would let a quote for other work of
    /// the same size stand in.
    pub work: Vec<QuoteJob>,
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
    /// Turn this connection into a subscription (C85): answered
    /// `subscribed`, after which the daemon writes only `event` and
    /// `resync_required` frames on it and answers any further versioned
    /// request `bad_request`. A subscriber snapshots over a request
    /// connection after subscribing.
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
    /// Clear the live-test rails' tripwire and ceiling halt (`LIVE-TESTING.md`).
    ResetTripwire,
    /// Everything the live dashboard renders, in one round-trip: daemon
    /// vitals, auth state, limiter policies, jobs, HTTP sends, recent errors.
    Dashboard,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "resp", rename_all = "snake_case")]
pub enum Response {
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
        /// The closed classification a frontend branches on.
        kind: ErrorKind,
        /// What happened, useful on its own.
        message: String,
    },
    /// Unsolicited; only on subscribed connections. An invalidation hint,
    /// never a complete stream (C85).
    Event {
        job: JobInfo,
    },
    /// Unsolicited; only on subscribed connections. The daemon's event
    /// channel overran this subscriber and `missed` events were dropped:
    /// snapshot again before relying on the view (C85).
    ResyncRequired {
        missed: u64,
    },
}
