//! The daemon: single job queue, single worker, JSON-lines Unix socket server.
//!
//! Lifecycle follows the gpg-agent model: clients spawn it on demand, it exits
//! on its own after a stretch with no connections and no live jobs.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/daemon.md` for this
//! area, `CONTEXT.md` for the cross-cutting ones (`C<n>`); what follows is each
//! entry's full text as recorded there, moved here on 2026-09-02 because
//! the mechanism it describes is this module's. The registry is current;
//! this is the mechanism as decided, kept beside the code that implements it.
//!
//! ## C6 — The job queue persists: a `jobs` table in a per-daemon `daemon.db` (SQLite, in the prov…
//!
//! **The job queue persists: a `jobs` table in a per-daemon `daemon.db` (SQLite, in the provider's store directory beside the account files), written through at every state change and read back at start.** Memory stays the runtime source of truth — the table is a mirror, "the HashMap, but it survives" — and the one thing read from it while the daemon runs is `result` for an id this lifetime never held (history, not state). On restore only open jobs are loaded (terminal rows carry bodies; a week of them does not belong in memory): `waiting` jobs resume; a `running` job is **re-queued** where the replay premise holds — every network kind is an idempotent GET and the restart probe reads GGG's current counters before it sends, so the duplicate costs one seen hit. Two exceptions (2026-08-30 review): on a declared **no-probe route** the premise fails — a replay would go out against an empty limiter — so the job fails as interrupted instead; and a running **parent whose children exist is mid-fan-out** (its held result was not yet written) — re-running it would submit a duplicate child set, so it holds for the children it has and then finishes as **interrupted, never success**: how many children were never submitted is unknowable, so a partial fan-out must not claim completeness (the children that did run recorded their responses; resubmitting completes the set). A parent whose held result was written resumes holding it; `probe` rows are dropped (one HEAD per lifetime, N16); ids continue from where they were (`AUTOINCREMENT`, never reused, so a stale `acq result <id>` can never name a different job) — a daemon that cannot open or read `daemon.db` refuses to start rather than risk reissuing them, and a queue **write** failure at runtime is sticky: a submit whose insert fails is refused with its id rolled back (a job exists only once its row does), later submits are refused outright, and the dispatcher stops picking while running jobs finish — **ids** never run ahead of disk. Completions are the accepted residual, stated plainly: a job already running when the flag trips finishes in memory but its outcome write fails, so disk still says running and the next daemon replays it (probed route: one seen duplicate hit; no-probe: fails as interrupted) — the send already happened, so refusing to finish it would record nothing at all. The same teeth apply per transition: a `waiting→running` write that fails reverts the job instead of running it (a send the queue cannot see must not happen), `cancel`/`set-priority` report a failed write instead of claiming success (the cancel still wins the job's terminal surface in this lifetime, though an already-running job may still complete its in-flight send — sends are committed once dispatched), and a `result` read failure is an error, never "no job". `submit_child` is refused once its parent is terminal or asked to cancel, under the lock `cancel` takes, so cancellation cannot race an active fan-out into submitting unseen children; a stopped fan-out finishes cancelled or failed, never as success over a partial set; a cancellation that lands after the last child is honored when the held result is installed, cancelled children never count toward a parent's success, and `finish` arbitrates a pending `cancel_requested` under the final lock — a cancel can land at any instant before terminalization and still win. Terminal rows stay so `acq result` has a memory across restarts (`acq jobs` lists live jobs only), pruned at start by age — `ACQ_JOB_RETENTION_DAYS` (default 7) for done/cancelled and `ACQ_FAILED_JOB_RETENTION_DAYS` (default 30) for failed, misread values logged as `CONFIG` errors like the rails knobs. Outcomes are stored verbatim, bodies included (a full refresh is ~50 MB, bounded by retention); compression was considered and deferred — it costs a crate and makes the column unreadable in `sqlite3`, and compressing one column later is a local change. **One daemon per store directory is an invariant, not a lock**: parallel daemons are for the mock and already require `ACQ_STORE_DIR=<scratch>` next to `ACQ_SOCKET` (`AGENTS.md`); two daemons on one `daemon.db` would each restore and run the same queue. Rationale: the queue was the one thing a restart lost once results moved to the store (2026-08-29); a mirror written under the same lock as the memory change keeps disk equal to memory at the `process::exit` the daemon leaves by (up to the declared write-failure residual above); SQLite because it is the crate's one persistence idiom, debuggable with `sqlite3`, and readable by a frontend without a daemon. Decided 2026-08-30.
//!
//! ## C23 — Work that needs many requests is a parent job that submits child jobs; a parent finishe…
//!
//! **Work that needs many requests is a parent job that submits child jobs; a parent finishes when its last descendant does, gives up its dispatcher task and scheduling key while waiting, and cancels its descendants when cancelled.** Rationale: the queue, dispatcher, priorities, ETAs, and events already work per job, so children get all of it for free; a job-internal loop would need its own scheduler and hide the requests from every tool. Observed API shapes (2026-08-20): folder children are in the stash list (a folder holds tabs only — never items, never another folder; confirmed against GGG patch notes 2026-08-24); map/unique substashes only appear on fetching the tab (one map tab listed 234); substash stubs carry `metadata.items` counts. Following substashes is opt-in per tab.
//!
//! ## C31 — Multi-account is one daemon holding many sessions, never one daemon per account.
//!
//! **Multi-account is one daemon holding many sessions, never one daemon per account.** The Cloudflare bound (`SendGate`, 2 live sends) is a per-IP property (P-B, ground truth §1) held as per-process state; two daemons on one machine make it a 4-wide burst that neither sees, with separate tripwires. Rung 11 (2026-08-30) showed the other half: `Account` rules count per account on GGG's side, so two accounts never contend on layer 2 — the only thing they share is layer 1 and the `Ip`-scoped token endpoint, which is exactly what the single gate exists for. Built in two halves with different blast radii (option C): **account as first-class identity now** (store path, job field, keyring key — leaves), **many live sessions later** (a refactor confined to the session layer). Limiter and probe scope keying — `(account, policy)` for `Account` rules, policy alone for `Ip` rules, scope learned from `X-Rate-Limit-Rules` — is a **precondition of the session map, not an optimization**: with two live sessions on one policy each response would overwrite shared state with a different account's counters, and the next send from the other account floods (a 429 path; the "over-waits, never floods" reading only held for rung 11's sequential switch). Decided 2026-08-29, amended 2026-08-30 after review across sessions; design below in "Multi-account design"; built 2026-08-30 through step (6) — step (7)'s live samples are in `LIVE-TESTING.md`'s run ledger.
//!
//! ## C32 — Per-route knowledge about GGG that headers cannot teach lives in one place (`Daemon::de…
//!
//! **Per-route knowledge about GGG that headers cannot teach lives in one place (`Daemon::declare_route_knowledge`), and strict observation is the default everywhere else.** `GET /profile` (first contact 2026-08-30) answers 200 with no `X-Rate-Limit-*` headers at all and 403 to HEAD, which strict observation ("every endpoint has a policy", post-N33) classed as a protocol failure and discarded. Now: a route *declared* policyless accepts a 2xx with **no** rate-limit header (a partial set is still a failure; a policy that later appears is learned strictly), becomes `EndpointState::Policyless`, and is paced by nothing but the send gate; a declared no-probe route goes straight to its GET. Only `/profile` is declared, and it is called at most once per login. Not generalised on purpose: "any headerless 2xx is fine" reopens the blind spot strict observation closed. Owner decision 2026-08-30; GGG confirmed the same day (Q12/N38): `/profile` is not rate limited at present, so the declaration is confirmed and stays until headers ever appear — strict observation covers that arm.
//!
//! ## C43 — Apply is its own pure fan-out parent job kind (`apply`), never the `refresh` parent.
//!
//! **Apply is its own pure fan-out parent job kind (`apply`), never the `refresh` parent.** The refresh parent re-lists by construction — it fans out from the listing it just fetched — which contradicts "executes exactly the listed actions": a plan's listing is an optional action and its fetches derive from reviewed facts. So `apply`'s params carry the plan's actions as explicit `(kind, params)` child tuples; the parent performs no send of its own, submits exactly one child per tuple, and holds for them. "Never expands" is structural, not disciplinary: the daemon stays plan-blind (it cannot link the store or plan crates), so what it admits is **vocabulary, not meaning** — only single-request kinds (`stashes`, or `stash` with `deep` false), each of which submits no children. Admission is at submit, before a job id exists: a malformed tuple list, an empty one, or a logical bound over the caller's `max_requests` refuses the submit whole — the mid-fan-out terminalization path is never the budget's normal mechanism (D8). Plan validation, the staleness check, and rendering actions to tuples are the frontend's (`acq refresh --apply`, through the planner's validating parse). The ad-hoc `refresh` kind (`--all`/`--tabs`) stays untouched as the explicit client-stated-selection surface; whether it retires rides on step 9's friction notes. Decided 2026-09-01.
//!
//! ## C49–C51 — Multi-account design as recorded (2026-08-30)
//!
//! **Complexity rule:** the only code that interprets accounts is the
//! session layer. Everywhere else account is data — a field on the job, a
//! path segment for the store, an opaque key component for the limiter
//! (which never reads it; scope comes from `X-Rate-Limit-Rules`). An
//! `if account == …` outside the session layer is the smell.
//!
//! - **Identity: the stable account key is the profile `uuid`, fetched at
//!   login and required** (amended 2026-08-31; was username-only with
//!   opportunistic uuid). After token exchange the daemon submits a profile
//!   job — causal service of the client's `acq auth` — and the session is
//!   registered, the keyring written, and `accounts.json` updated only when
//!   the uuid lands; a login whose profile fetch fails **fails whole**: no
//!   provisional identity, no minted keys, no rename-repair machinery — if
//!   `/profile` is broken, something is broken and login says so. A retry
//!   repeats the token exchange, paced by the `Ip`-scoped token policy, so
//!   a retry loop is already bounded. `accounts.json` maps
//!   username/discriminator/provider → uuid; a rename is a mapping update
//!   with intent untouched. The token response's `username`
//!   (`name#discriminator`) stays the display name and selector; fact files
//!   stay username-named (refetchable; rename-orphaning tolerable);
//!   annotation files are uuid-named. Entries without a uuid: one re-auth,
//!   no migration. The mock serves deterministic per-username uuids.
//! - **No daemon-side default account; stateless selection.** Every submit
//!   carries `account`. Omitted, it resolves only when exactly one session
//!   exists; otherwise the daemon refuses with the list. While the daemon
//!   holds one session, a submitted `account` is validated against it and
//!   refused on mismatch (so the selector is testable before the session
//!   map exists). The CLI resolves `--account` / `ACQ_ACCOUNT` client-side
//!   against a non-secret index file, `store/<provider>/accounts.json`
//!   (username, uuid when known, last login), so reads never spawn a daemon.
//!   Matching is exact — name with or without discriminator, or uuid —
//!   never by prefix. GUI/MCP hold their own selection and pass it.
//! - One-off (non-persisted) sessions are accounts: listed, selectable,
//!   marked "not persisted".
//! - A job has exactly one account; no cross-account `refresh --all`.
//!   Cross-account work is a frontend loop.
//! - `account` is a protocol field on `Submit`/`JobInfo`, not a params
//!   entry; fixed at submit (resolved against the live session, refused
//!   before a job exists otherwise), checked again at the moment a token is
//!   taken (a mismatch fails the job with no send), and it selects the store
//!   file — never the session at landing time. Shown in `jobs`, `dash`, the
//!   daemon log, and the journal (the `route` field is the endpoint key,
//!   `stash@Alice#1234`).
//! - **Limiter keying as built (step 5):** the endpoint key is
//!   `route@account`; a policy's state is keyed `name@account` only when
//!   *every* rule of the policy is `Account`-scoped and the send had an
//!   account — `Ip` rules, mixed scopes, and accountless sends share the
//!   bare name (over-waits at worst). One notch more conservative than
//!   "Account rules per account" if GGG ever mixes scopes in one policy.
//!   The token route (`oauth-token`) is deliberately accountless: it is
//!   `Ip`-scoped and has no probe, so an accounted key would be unpaced on
//!   an account's first login.
//! - **The free HEAD probe (N24) is per endpoint, not an API property.**
//!   First contact 2026-08-30: `HEAD /account/leagues` is answered 200 and
//!   counted as a hit (the free HEADs answer 204); `HEAD /profile` is 403.
//!   Both routes are declared no-probe in `Daemon::declare_route_knowledge`
//!   and taught by their first GET; pacing was never wrong (headers are
//!   post-increment and trusted), a probe there is just a wasted hit.
//! - Store: `store/<provider>/<account>.db`, opened lazily on first record;
//!   `tabs`/`items`/`store` take the selector and never span accounts.
//!   Keyring: one entry per account; the index file is how the daemon knows
//!   which entries to restore (the keyring crate cannot enumerate). Restore
//!   continues past a dead grant — the terminal-grant mark is per session.
//!   The existing single keyring entry is orphaned, not migrated: one
//!   re-auth.
//! - The `jobs` table lives in a per-daemon `daemon.db`, not inside an
//!   account file, and carries the account column (persistence decision above).
//! - Mock: the login page accepts any username and policies count per
//!   username (the access token carries it, `at-<user>-<rand>`), so
//!   two-account tests can distinguish per-account from shared counting —
//!   the property rung 11 established for GGG.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use acquisition_store::jobs::{JobDb, JobRow, Retention};
use acquisition_store::{Endpoint, Index, Store, account_matches, account_path, store_dir};
use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener, UnixStream};
use tokio::sync::{Notify, broadcast, watch};

use std::collections::VecDeque;

use crate::VERSION;
use crate::job::{JobId, JobInfo, JobState, Outcome, Priority, target_of};
use crate::protocol::{ErrorRecord, Quote, QuoteJob, QuoteScope, Request, Response, SessionStatus};
use crate::provider::{CALLBACK_PATH, Provider, SCOPES, ggg_mode};
use crate::rails::{BlockShape, Rails, RailsConfig};
use crate::ratelimit::{
    ChokePoint, Clock, EndpointState, RetryAfter, SendError, SystemClock, url_path,
};
use crate::ratelimit::{endpoint_key, split_endpoint_key};
use crate::realm::{Family, Realm};
use crate::{auth, mockggg};

const IDLE_SHUTDOWN: Duration = Duration::from_secs(60);

/// `ACQ_IDLE_SHUTDOWN=<secs>` overrides the idle exit (L0 rail 8) so a
/// live-test rung or soak runs on one daemon instead of a respawn cycle.
fn idle_shutdown_from_env() -> Duration {
    std::env::var("ACQ_IDLE_SHUTDOWN")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map_or(IDLE_SHUTDOWN, Duration::from_secs)
}
const IDLE_POLL: Duration = Duration::from_secs(5);
const ERROR_HISTORY: usize = 50;
/// Probes outrank everything: every job on that route is waiting on one.
const PROBE_PRIORITY: Priority = u8::MAX;

/// The login's own profile job: interactive, and nothing else on the
/// account can run before the login completes anyway.
const LOGIN_PRIORITY: Priority = u8::MAX - 1;

/// How long a login waits for its profile job before failing whole. Room
/// for a full token-endpoint hold; a rails-halted daemon fails the login
/// rather than pinning the flow forever.
const LOGIN_PROFILE_TIMEOUT: Duration = Duration::from_secs(120);
/// How many times a job is re-queued after a 429 before it fails for good
/// (ground truth P-A: violations are structural, so recovery is required;
/// N10: frequent violations get the application revoked, so it is bounded).
pub const MAX_429_RETRIES: u32 = 2;

// Must stay short: Unix socket paths cap out around 104 bytes (SUN_LEN),
// which deep per-user runtime dirs can exceed.
pub fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("ACQ_SOCKET") {
        return PathBuf::from(p);
    }
    std::env::temp_dir().join("acquisition-playground.sock")
}

pub fn log_path() -> PathBuf {
    socket_path().with_extension("log")
}

/// Default send-journal location (`ACQ_JOURNAL` overrides; `ACQ_JOURNAL=0`
/// disables).
pub fn journal_path(provider_name: &str) -> PathBuf {
    socket_path().with_extension(format!("{provider_name}.sends.jsonl"))
}

/// What a network call can fail with; 429 is its own arm so the job can
/// be re-queued rather than failed.
#[derive(Debug)]
enum ApiError {
    RateLimited(String),
    Protocol(String),
    Other(String),
}

/// What `execute` hands back to `process`.
enum Exec {
    Done(Outcome),
    RateLimited(String),
}

fn may_requeue_429(retries: u32, cancel_requested: bool) -> bool {
    retries < MAX_429_RETRIES && !cancel_requested
}

struct Entry {
    info: JobInfo,
    params: Value,
    outcome: Option<Outcome>,
    cancel_requested: bool,
    /// A parent's own result, held back until its descendants finish. Set
    /// means "running, waiting on children, with no active dispatcher task".
    deferred: Option<Outcome>,
    /// Unix seconds; the persisted row's `submitted_at`.
    submitted_at: i64,
}

impl Entry {
    /// The row `daemon.db` holds for this job (`CONTEXT.md`, "The job
    /// queue persists"): `JobInfo` column for column plus the fields a
    /// restart needs. `eta_seconds` is a prediction, never stored.
    fn row(&self, now: i64) -> JobRow {
        let json = |o: &Outcome| serde_json::to_value(o).unwrap_or(Value::Null);
        JobRow {
            id: self.info.id,
            kind: self.info.kind.clone(),
            state: self.info.state.to_string(),
            priority: self.info.priority,
            submitted_by: self.info.submitted_by.clone(),
            parent: self.info.parent,
            retries: self.info.retries,
            account: self.info.account.clone(),
            params: self.params.clone(),
            outcome: self.outcome.as_ref().map(json),
            deferred: self.deferred.as_ref().map(json),
            cancel_requested: self.cancel_requested,
            submitted_at: self.submitted_at,
            updated_at: now,
        }
    }

    /// A restored row. What a restart does with `running` (re-queue) or
    /// `probe` (drop) is decided by the caller; this is the plain mapping.
    fn from_row(row: JobRow) -> Option<Entry> {
        let state = match row.state.as_str() {
            "waiting" => JobState::Waiting,
            "running" => JobState::Running,
            "done" => JobState::Done,
            "failed" => JobState::Failed,
            "cancelled" => JobState::Cancelled,
            _ => return None,
        };
        let outcome = |v: Option<Value>| v.and_then(|v| serde_json::from_value(v).ok());
        Some(Entry {
            info: JobInfo {
                id: row.id,
                kind: row.kind,
                state,
                priority: row.priority,
                submitted_by: row.submitted_by,
                eta_seconds: None,
                parent: row.parent,
                retries: row.retries,
                account: row.account,
                params: row.params.clone(),
            },
            params: row.params,
            outcome: outcome(row.outcome),
            cancel_requested: row.cancel_requested,
            deferred: outcome(row.deferred),
            submitted_at: row.submitted_at,
        })
    }
}

/// Seconds since the Unix epoch, for persisted rows.
fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `ACQ_JOB_RETENTION_DAYS` / `ACQ_FAILED_JOB_RETENTION_DAYS`, each read
/// like a rails knob: absent means the default, a misread value is
/// reported (the second return) and the default stays.
fn retention_from_env() -> (Retention, Vec<String>) {
    let mut r = Retention::default();
    let mut problems = Vec::new();
    for (var, slot) in [
        ("ACQ_JOB_RETENTION_DAYS", &mut r.done_days),
        ("ACQ_FAILED_JOB_RETENTION_DAYS", &mut r.failed_days),
    ] {
        if let Ok(v) = std::env::var(var) {
            match v.trim().parse::<u32>() {
                Ok(days) => *slot = days,
                Err(_) => {
                    problems.push(format!("{var}={v:?} is not a number of days; using {slot}"))
                }
            }
        }
    }
    (r, problems)
}

type AccessTokenResult = Result<(String, String), String>;

const SESSION_CHANGED_DURING_REFRESH: &str =
    "authentication session changed while token refresh was in progress";
const REFRESH_OWNER_ABANDONED: &str = "token refresh owner was abandoned before producing a result";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct AuthGenerations {
    session: u64,
    access_token: u64,
    refresh_token: u64,
}

struct RefreshFlight {
    id: u64,
    generations: AuthGenerations,
    result: watch::Sender<Option<AccessTokenResult>>,
}

/// A login whose token exchange succeeded but whose profile fetch has not
/// landed the uuid yet (CONTEXT.md, identity decision). Held *outside* the
/// session map: the previously live session for the same account keeps
/// serving its jobs untouched, and no client lookup can reach these tokens
/// — only the profile route hands them out (`staged_profile_token`). A
/// login whose profile fetch fails drops this slot: it fails whole.
struct Staging {
    /// The flow that staged this; stale flows can neither complete nor
    /// abort it.
    flow: u64,
    username: String,
    access_token: String,
    access_expires_at: SystemTime,
    refresh_token: String,
}

#[derive(Default)]
struct AuthSession {
    access_token: Option<String>,
    /// Wall-clock, not `Instant`: on macOS a monotonic clock does not advance
    /// while the machine sleeps, so an `Instant` deadline would wake believing
    /// an expired token still has hours left and send 401s until it caught up.
    access_expires_at: Option<SystemTime>,
    refresh_token: Option<String>,
    username: Option<String>,
    /// The account uuid the login's profile fetch delivered (restored
    /// sessions read it from the index). A refresh that renames the account
    /// uses it to update the index mapping in place rather than minting a
    /// uuid-less twin entry.
    uuid: Option<String>,
    /// "ok" or an error description shown in `auth status`.
    keyring: String,
    generations: AuthGenerations,
    refresh_flight: Option<RefreshFlight>,
    next_refresh_flight: u64,
}

/// Every live session, by account. One daemon, many sessions (CONTEXT.md,
/// "Multi-account design"): the daemon holds no default — a caller names
/// the account, or there is exactly one.
#[derive(Default)]
struct Sessions {
    by_account: HashMap<String, AuthSession>,
    /// A login flow waiting on the browser (its flow generation).
    pending: Option<u64>,
    next_flow: u64,
    /// A login past token exchange, waiting on its profile job's uuid.
    staging: Option<Staging>,
    /// How the most recent login flow ended: the username it registered, or
    /// the error it failed with. Cleared when a new flow starts; this is
    /// what `acq auth` reports, so a failed login is never mistaken for a
    /// different account's live session.
    flow_result: Option<Result<String, String>>,
    /// The account of the most recent login — reported, never used to pick.
    last_login: Option<String>,
    /// Keyring health from restore, shown when there is no session to ask.
    keyring: String,
}

impl Sessions {
    /// The session an operation is for: the named account's, or the sole
    /// one. No selector with several live is refused — the daemon does not
    /// guess whose stash to spend sends on.
    /// Invariant: a session's map key is its `username`. Every insert goes
    /// through `replace`, and a refresh that renames goes through
    /// `rename`, so lookups can trust the key.
    fn find(&self, username: &str) -> Option<&AuthSession> {
        self.by_account.get(username)
    }

    fn find_mut(&mut self, username: &str) -> Option<&mut AuthSession> {
        self.by_account.get_mut(username)
    }

    /// Move a session to a new name (a refresh reported a different
    /// username). The session keeps its state; only the key changes. A
    /// session already under the new name is replaced.
    fn rename(&mut self, from: &str, to: &str) {
        if from == to {
            return;
        }
        if let Some(mut session) = self.by_account.remove(from) {
            session.username = Some(to.to_string());
            self.by_account.insert(to.to_string(), session);
            if self.last_login.as_deref() == Some(from) {
                self.last_login = Some(to.to_string());
            }
        }
    }

    fn get(&self, account: Option<&str>) -> Result<&AuthSession, String> {
        match account {
            Some(account) => self
                .find(account)
                .ok_or_else(|| format!("no session for {account} — run `acq auth`")),
            None => match self.by_account.len() {
                0 => Err("not logged in — run `acq auth`".into()),
                1 => Ok(self.by_account.values().next().expect("one")),
                _ => Err(format!(
                    "several accounts are logged in ({}); pick one with --account",
                    self.usernames().join(", ")
                )),
            },
        }
    }

    fn get_mut(&mut self, account: Option<&str>) -> Result<&mut AuthSession, String> {
        match account {
            Some(account) => self
                .find_mut(account)
                .ok_or_else(|| format!("no session for {account} — run `acq auth`")),
            None => {
                self.get(None)?;
                Ok(self.by_account.values_mut().next().expect("one"))
            }
        }
    }

    /// Live usernames, sorted.
    fn usernames(&self) -> Vec<String> {
        let mut v: Vec<String> = self.by_account.keys().cloned().collect();
        v.sort();
        v
    }

    /// The live session a selector names (username, name without
    /// discriminator, or uuid), if any. A staged login is not a session and
    /// cannot match: it lives outside this map.
    fn matching(&self, selector: &str) -> Option<&AuthSession> {
        self.by_account.values().find(|s| {
            s.username
                .as_deref()
                .is_some_and(|u| account_matches(selector, u, s.uuid.as_deref()))
        })
    }

    /// Insert or replace an account's session. Replacing advances the old
    /// session's generations so an in-flight refresh for it lands stale.
    fn replace(&mut self, mut session: AuthSession) -> &mut AuthSession {
        let key = session.username.clone().unwrap_or_default();
        if let Some(old) = self.by_account.remove(&key) {
            session.generations = AuthGenerations {
                session: old.generations.session.wrapping_add(1),
                access_token: old.generations.access_token.wrapping_add(1),
                refresh_token: old.generations.refresh_token.wrapping_add(1),
            };
            session.next_refresh_flight = old.next_refresh_flight;
        }
        self.last_login = Some(key.clone());
        self.by_account.entry(key).or_insert(session)
    }

    #[cfg(test)]
    fn with(session: AuthSession) -> Sessions {
        let mut s = Sessions {
            keyring: "ok".into(),
            ..Sessions::default()
        };
        s.replace(session);
        s
    }

    /// Test helper: the sole session.
    #[cfg(test)]
    fn one(&self) -> &AuthSession {
        assert_eq!(self.by_account.len(), 1, "exactly one session expected");
        self.by_account.values().next().expect("one")
    }

    #[cfg(test)]
    fn one_mut(&mut self) -> &mut AuthSession {
        assert_eq!(self.by_account.len(), 1, "exactly one session expected");
        self.by_account.values_mut().next().expect("one")
    }
}

impl AuthSession {
    fn advance_access_token(&mut self) {
        self.generations.access_token = self.generations.access_token.wrapping_add(1);
    }

    fn advance_refresh_token(&mut self) {
        self.generations.refresh_token = self.generations.refresh_token.wrapping_add(1);
    }
}

struct Shared {
    jobs: HashMap<JobId, Entry>,
    next_id: JobId,
    auth: Sessions,
    connections: usize,
    last_activity: Instant,
    /// Recent errors for the dashboard, newest last (bounded ring). Every
    /// entry is also in the log; this is the structured, queryable subset.
    errors: VecDeque<(Instant, String)>,
    /// Jobs with an active dispatcher task → their scheduling key.
    ///
    /// This is ordering ownership, not HTTP capacity: N4's send gate owns the
    /// actual-send bound. One active task per key keeps same-policy priority
    /// and FIFO stable without letting auth or pacing waits block other keys.
    active_jobs: HashMap<JobId, String>,
}

impl Shared {
    /// Waiting jobs in dispatch order: highest priority first, then FIFO.
    fn queue_order(&self) -> Vec<JobId> {
        let mut waiting: Vec<&Entry> = self
            .jobs
            .values()
            .filter(|e| e.info.state == JobState::Waiting)
            .collect();
        waiting.sort_by_key(|e| (std::cmp::Reverse(e.info.priority), e.info.id));
        waiting.iter().map(|e| e.info.id).collect()
    }

    /// Snapshot with eta filled in for waiting jobs. The daemon can predict
    /// because it sees the whole queue and the limiter.
    fn snapshot(&self, daemon: &Daemon, id: JobId) -> Option<JobInfo> {
        let queue = self.queue_order();
        let entry = self.jobs.get(&id)?;
        let mut info = entry.info.clone();
        info.eta_seconds = if info.state == JobState::Waiting {
            match daemon.keyed_route_for(&info.kind, &entry.params, info.account.as_deref()) {
                Some((route, _)) => {
                    // Only same-route jobs ahead of us compete for the same
                    // policy; counting them is what the estimate needs.
                    let ahead = queue
                        .iter()
                        .take_while(|&&q| q != id)
                        .filter(|q| {
                            self.jobs
                                .get(q)
                                .and_then(|e| {
                                    daemon.keyed_route_for(
                                        &e.info.kind,
                                        &e.params,
                                        e.info.account.as_deref(),
                                    )
                                })
                                .is_some_and(|(r, _)| r == route)
                        })
                        .count();
                    Some(daemon.choke.eta_for(&route, ahead as u32).as_secs())
                }
                None => Some(0),
            }
        } else {
            None
        };
        Some(info)
    }

    fn list(&self, daemon: &Daemon) -> Vec<JobInfo> {
        let mut ids: Vec<JobId> = self.jobs.keys().copied().collect();
        ids.sort();
        ids.into_iter()
            .filter_map(|id| self.snapshot(daemon, id))
            .collect()
    }
}

pub struct Daemon {
    shared: Mutex<Shared>,
    /// On `Daemon`, not `Shared`, so `log`'s uptime prefix needs no lock:
    /// `persist` logs failures while holding `shared`.
    started: Instant,
    events: broadcast::Sender<JobInfo>,
    work: Notify,
    log: Mutex<std::fs::File>,
    /// The single rate-limit choke point (CONTEXT invariant 1). It owns the
    /// HTTP client, so all outbound requests — OAuth included — consult the
    /// header-driven limiter and feed their responses back to it.
    choke: ChokePoint,
    /// The in-process mock by default; real GGG only when the daemon was
    /// started with ACQ_GGG=1.
    provider: Provider,
    credential_store: Arc<dyn CredentialStore>,
    /// The provider's store directory (`acquisition-store`): one file per
    /// account plus the account index. `None` in tests: nothing recorded.
    store_dir: Option<PathBuf>,
    /// The open store for the current session's account, opened on first
    /// record and reopened when the session's username changes. The daemon
    /// only ever writes it.
    store: Mutex<Option<(String, Store)>>,
    /// The persisted queue (`daemon.db`), mirrored from memory at every
    /// state change and read at start. Always present: `run` refuses to
    /// start without it, and test daemons get a throwaway in-memory one —
    /// so every test exercises the mirror.
    jobs_db: Mutex<JobDb>,
    /// Set (sticky) by the first failed queue write. A daemon whose
    /// mirror diverges from memory must not take or dispatch new work:
    /// finished jobs would replay and ids could repeat after a restart.
    /// Running jobs finish (their sends are committed); everything else
    /// refuses until a restart finds a working `daemon.db`.
    queue_failure: Mutex<Option<String>>,
}

struct RefreshOwnerGuard<'a> {
    daemon: &'a Daemon,
    account: String,
    id: u64,
    generations: AuthGenerations,
    result: Option<watch::Sender<Option<AccessTokenResult>>>,
}

impl RefreshOwnerGuard<'_> {
    fn finish(mut self, refresh: Result<auth::TokenResponse, String>) -> AccessTokenResult {
        let result = self.result.as_ref().expect("refresh owner result exists");
        let outcome =
            self.daemon
                .finish_refresh(&self.account, self.id, self.generations, refresh, result);
        self.result = None;
        outcome
    }
}

impl Drop for RefreshOwnerGuard<'_> {
    fn drop(&mut self) {
        let Some(result) = self.result.take() else {
            return;
        };
        if result.borrow().is_some() {
            return;
        }
        {
            let mut s = self.daemon.shared.lock().unwrap();
            if let Some(session) = s.auth.find_mut(&self.account) {
                let owns_current_flight = session.generations == self.generations
                    && session.refresh_flight.as_ref().is_some_and(|flight| {
                        flight.id == self.id && flight.generations == self.generations
                    });
                if owns_current_flight {
                    session.refresh_flight = None;
                }
            }
        }
        result.send_replace(Some(Err(REFRESH_OWNER_ABANDONED.into())));
    }
}

trait CredentialStore: Send + Sync {
    fn save(&self, service: &str, refresh_token: &str, username: &str) -> Result<(), String>;
    fn clear(&self, service: &str, username: &str) -> Result<(), String>;
}

struct OsCredentialStore;

impl CredentialStore for OsCredentialStore {
    fn save(&self, service: &str, refresh_token: &str, username: &str) -> Result<(), String> {
        auth::keyring_save(service, refresh_token, username)
    }

    fn clear(&self, service: &str, username: &str) -> Result<(), String> {
        auth::keyring_clear(service, username)
    }
}

impl Daemon {
    // Takes the shared lock for the uptime stamp — never call while holding it.
    fn log(&self, msg: &str) {
        let uptime = self.started.elapsed().as_secs();
        let mut f = self.log.lock().unwrap();
        let _ = writeln!(f, "[{uptime:>5}s] {msg}");
    }

    fn emit(&self, info: JobInfo) {
        self.log(&format!("job {} -> {}", info.id, info.state));
        let _ = self.events.send(info);
    }

    /// Log an error and keep it in the dashboard's recent-errors ring.
    fn note_error(&self, msg: &str) {
        {
            let mut s = self.shared.lock().unwrap();
            if s.errors.len() >= ERROR_HISTORY {
                s.errors.pop_front();
            }
            s.errors.push_back((Instant::now(), msg.to_string()));
        }
        self.log(msg);
    }

    /// Mirror one job to `daemon.db`. Called with the `Shared` lock held,
    /// so the table sees changes in memory's order. A failed write trips
    /// the sticky queue failure (logged here — safe, `log` takes no lock
    /// but its own — and refused loudly at the next submit): disk has
    /// diverged from memory, so no new work may be taken.
    fn persist(&self, entry: &Entry) -> bool {
        match self.jobs_db.lock().unwrap().upsert(&entry.row(unix_now())) {
            Ok(()) => true,
            Err(e) => {
                let mut failure = self.queue_failure.lock().unwrap();
                if failure.is_none() {
                    *failure = Some(format!("{e:#}"));
                    self.log(&format!(
                        "JOBS QUEUE FAILED persisting job {}: {e:#}; refusing new jobs until restart",
                        entry.info.id
                    ));
                }
                false
            }
        }
    }

    /// The sticky queue-write failure, if one has happened.
    fn queue_failed(&self) -> Option<String> {
        self.queue_failure.lock().unwrap().clone()
    }

    /// Take the previous lifetime's open jobs from `daemon.db`
    /// (`CONTEXT.md`, "The job queue persists"): waiting jobs resume; a
    /// running one is re-queued only where the replay premise holds — a
    /// probe will read GGG's current counters before the send. Two
    /// exceptions: a running job on a declared no-probe route would send
    /// blind, so it fails as interrupted instead; and a running parent
    /// whose children exist is mid-fan-out (its held result was not yet
    /// written) — re-running it would submit a duplicate child set, so it
    /// is held for its existing children with a synthetic result. A parent
    /// whose held result was written keeps holding it; probes are per
    /// lifetime and dropped. An open parent's finished children come back
    /// with it (its summary counts them). Ids continue from the table's
    /// sequence. Other terminal rows stay in the table for `result`,
    /// pruned by age first.
    /// A read failure is fatal to startup, like an unopenable file: a
    /// daemon that cannot see the previous lifetime's rows would reissue
    /// ids from 1 against them.
    fn restore_jobs(&self, retention: Retention) -> Result<()> {
        let db = &self.jobs_db;
        let (rows, next_id, pruned) = {
            let db = db.lock().unwrap();
            let pruned = db.prune(retention, unix_now()).unwrap_or_else(|e| {
                self.log(&format!("JOBS: prune failed: {e:#}"));
                0
            });
            match (db.load_open(), db.next_id()) {
                (Ok(rows), Ok(next)) => (rows, next, pruned),
                (Err(e), _) | (_, Err(e)) => {
                    anyhow::bail!("could not read {}: {e:#}", db.path().display());
                }
            }
        };
        let entries: Vec<Entry> = rows.into_iter().filter_map(Entry::from_row).collect();
        let has_children: HashSet<JobId> = entries.iter().filter_map(|e| e.info.parent).collect();
        let (mut requeued, mut held, mut not_replayed, mut dropped) = (0, 0, 0, 0);
        let mut finish_parents = Vec::new();
        let mut restored = Vec::new();
        {
            let mut s = self.shared.lock().unwrap();
            s.next_id = next_id;
            for mut entry in entries {
                if entry.info.kind == "probe" {
                    dropped += 1;
                    let _ = db.lock().unwrap().delete(entry.info.id);
                    continue;
                }
                if entry.info.state == JobState::Running {
                    let no_probe_route = self
                        .keyed_route_for(
                            &entry.info.kind,
                            &entry.params,
                            entry.info.account.as_deref(),
                        )
                        .is_some_and(|(route, _)| !Self::route_probes(&route));
                    if entry.cancel_requested {
                        entry.info.state = JobState::Cancelled;
                        entry.outcome = Some(Outcome::Cancelled);
                    } else if entry.deferred.is_some() {
                        // Holding its result for children: no task to
                        // give it; it finishes when they do.
                        finish_parents.push(entry.info.id);
                    } else if has_children.contains(&entry.info.id) {
                        // Mid-fan-out: children were submitted but the
                        // held result was not yet written. Re-running the
                        // parent would submit them all again, so it holds
                        // for the set it has — but how many children never
                        // got submitted is unknowable, so once they land
                        // it finishes as interrupted, never as success.
                        entry.deferred = Some(Outcome::Failure {
                            error: "interrupted by a daemon restart mid fan-out: the children \
                                    submitted before the restart ran (their responses are in \
                                    the store), but the full child set is unknown — resubmit \
                                    to complete it"
                                .into(),
                        });
                        held += 1;
                        finish_parents.push(entry.info.id);
                    } else if no_probe_route {
                        // The replay premise — a probe reads the previous
                        // lifetime's hits before anything sends — does not
                        // hold on a no-probe route; a replay would go out
                        // against an empty limiter.
                        not_replayed += 1;
                        entry.info.state = JobState::Failed;
                        entry.outcome = Some(Outcome::Failure {
                            error: format!(
                                "interrupted by a daemon restart and not replayed: \
{} sends without a probe, so a replay could not learn the previous lifetime's hits first; \
resubmit if still wanted",
                                entry.info.kind
                            ),
                        });
                    } else {
                        entry.info.state = JobState::Waiting;
                        requeued += 1;
                    }
                    self.persist(&entry);
                }
                restored.push(entry.info.clone());
                s.jobs.insert(entry.info.id, entry);
            }
        }
        if !restored.is_empty() || dropped > 0 {
            self.log(&format!(
                "JOBS: restored {} from {} ({requeued} re-queued from running, {held} parents held mid-fan-out, {not_replayed} not replayed on no-probe routes, {dropped} probes dropped, {pruned} old rows pruned); next id {next_id}",
                restored.len(),
                db.lock().unwrap().path().display()
            ));
        }
        for info in restored {
            self.emit(info);
        }
        // A parent whose last child finished in the instant before the
        // previous daemon died is finished now.
        for pid in finish_parents {
            self.maybe_finish_parent(pid);
        }
        self.work.notify_one();
        Ok(())
    }

    /// The previous lifetime's result for a job this one never held.
    /// `Ok(None)` is genuinely no such job; a queue that cannot be read
    /// is an error, never mistaken for "no job".
    fn stored_outcome(&self, id: JobId) -> Result<Option<Outcome>, String> {
        match self.jobs_db.lock().unwrap().get(id) {
            Ok(row) => Ok(row.and_then(Entry::from_row).and_then(|e| e.outcome)),
            Err(e) => Err(format!("could not read the persisted queue: {e:#}")),
        }
    }

    /// Apply one change to the account index, reporting failure to the
    /// caller. Safe under the shared lock (it never calls `note_error`).
    /// No-op in tests (no store directory).
    fn index_apply(&self, f: impl FnOnce(&mut Index) -> anyhow::Result<()>) -> Result<(), String> {
        let Some(dir) = &self.store_dir else {
            return Ok(());
        };
        Index::load(dir)
            .and_then(|mut index| f(&mut index))
            .map_err(|e| format!("accounts index: {e:#}"))
    }

    /// Apply one change to the account index, logging (never failing) on
    /// error. Not for callers holding the shared lock (`note_error` takes
    /// it) — those use `index_apply`.
    fn with_index(&self, f: impl FnOnce(&mut Index) -> anyhow::Result<()>) {
        if let Err(e) = self.index_apply(f) {
            self.note_error(&e);
        }
    }

    /// Hand a successful API body to the shared store. The daemon's whole
    /// involvement: endpoint + params + body; what is inside is the store's
    /// business. The result is classified: a body the store refuses as
    /// malformed (`acquisition_store::MalformedBody` — the response itself
    /// is bad, and ingesting it would have poisoned facts) comes back as
    /// the job's `Outcome::Failure`, so the caller returns it instead of
    /// reporting success; the store keeps that body verbatim in its
    /// `refused` table and the failure names the row, so the finding is
    /// readable without a re-fetch. Genuine persistence trouble (an unopenable file,
    /// a write error) stays logged-and-absorbed — the send happened and
    /// the payload still reaches the client that asked. The list-shaped
    /// jobs also pre-check their top-level array (a nicer early failure,
    /// and `refresh` needs the list anyway); entry-level judgment lives in
    /// the store alone and propagates from here.
    fn record(
        &self,
        account: Option<&str>,
        kind: &str,
        params: &Value,
        body: &Value,
    ) -> Result<(), Outcome> {
        let Some(dir) = &self.store_dir else {
            return Ok(());
        };
        let Some(endpoint) = Endpoint::from_job(kind, params) else {
            return Ok(());
        };
        // The job's account — fixed at submit — selects the file, never the
        // session at landing time: a login as B while A's refresh is still
        // landing must not file A's tabs under B.
        let Some(account) = account else {
            self.note_error(&format!(
                "store: {kind} landed with no account; not recorded"
            ));
            return Ok(());
        };
        let mut guard = self.store.lock().unwrap();
        if guard.as_ref().is_none_or(|(u, _)| u != account) {
            let path = account_path(dir, account);
            match Store::open(&path) {
                Ok(store) => {
                    self.log(&format!("store: {} opened for {account}", path.display()));
                    *guard = Some((account.to_string(), store));
                }
                Err(e) => {
                    self.note_error(&format!("store: could not open {}: {e:#}", path.display()));
                    return Ok(());
                }
            }
        }
        let (_, store) = guard.as_mut().expect("store opened above");
        let result = store.record(&endpoint, params, 200, body, acquisition_store::now());
        match result {
            Ok(ingest) => {
                self.log(&format!(
                    "store: {kind} {} -> response {} | {} items (+{} ~{} >{} -{}){}",
                    target_of(kind, params),
                    ingest.response_id,
                    ingest.items,
                    ingest.added,
                    ingest.changed,
                    ingest.moved,
                    ingest.removed,
                    match ingest.withheld {
                        Some(n) => format!(
                            " | WITHHELD: the location was retired by a listing; {n} item fact(s) kept on the response row, nothing landed"
                        ),
                        None => String::new(),
                    }
                ));
                Ok(())
            }
            Err(e) => match e.downcast_ref::<acquisition_store::MalformedBody>() {
                Some(malformed) => Err(Outcome::Failure {
                    error: malformed.to_string(),
                }),
                None => {
                    self.note_error(&format!("store: recording {kind} failed: {e:#}"));
                    Ok(())
                }
            },
        }
    }

    fn rails(&self) -> &Arc<Rails> {
        self.choke.rails()
    }

    /// Log a rails trip once, the first time any send path notices it.
    fn announce_trip(&self) {
        if let Some(cause) = self.rails().take_unannounced_trip() {
            self.note_error(&format!(
                "LIVE-TEST RAILS TRIPPED: {cause} — sends refused until `acq daemon reset-tripwire`"
            ));
        }
    }

    /// The route label and URL a job sends on, if it sends at all. Routes,
    /// not URLs, key the limiter: every league shares `stash-list`. A
    /// realm other than pc is a segment on the URL *and* on the label
    /// (`stash-list/xbox`): a realm's URL shape gets its own free HEAD
    /// probe before its first counted send, and whether it shares the pc
    /// policy is learned from its headers (same-name policies already
    /// share state, N6). pc adds nothing to either, so every pc URL and
    /// journal route is byte-identical to the pre-realm ones. `fetch` is a
    /// fake data endpoint that exists only on the mock.
    fn route_for(&self, kind: &str, params: &Value) -> Option<(String, String)> {
        let base = &self.provider.api_base;
        // Admission (`admit_realm`) refused anything a family does not
        // take before a job existed; a row that still fails here (a
        // persisted job from a build with a different table) sends nothing.
        let realm = |family: Family| family.realm_of(params).ok().map(Realm::infix);
        match kind {
            "characters" => {
                let r = realm(Family::Characters)?;
                Some((format!("character-list{r}"), format!("{base}/character{r}")))
            }
            // One character with its inventory/equipment: its own policy.
            "character" => {
                let r = realm(Family::Characters)?;
                let name = params.get("name").and_then(Value::as_str)?;
                Some((
                    format!("character{r}"),
                    format!("{base}/character{r}/{name}"),
                ))
            }
            // The account's leagues: `GET /account/leagues` (account:leagues).
            // `/league` is the public league list and needs `service:leagues`,
            // which the registration does not have (first contact
            // 2026-08-30: 403 `insufficient_scope`). Not realm-aware: no
            // consumer asks, and the plan never lists leagues.
            "leagues" => Some(("league".into(), format!("{base}/account/leagues"))),
            // The account profile (account:profile).
            "profile" => Some(("profile".into(), format!("{base}/profile"))),
            "stashes" | "refresh" => {
                let r = realm(Family::Stashes)?;
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard");
                Some((
                    format!("stash-list{r}"),
                    format!("{base}/stash{r}/{league}"),
                ))
            }
            // One tab, or one substash of a map/unique tab: same route, same
            // policy (stash-request-limit), one probe for all of them.
            "stash" => {
                let r = realm(Family::Stashes)?;
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard");
                let id = params.get("id").and_then(Value::as_str)?;
                let url = match params.get("sub").and_then(Value::as_str) {
                    Some(sub) => format!("{base}/stash{r}/{league}/{id}/{sub}"),
                    None => format!("{base}/stash{r}/{league}/{id}"),
                };
                Some((format!("stash{r}"), url))
            }
            "fetch" if !self.provider.is_real() => Some(("fetch".into(), format!("{base}/fetch"))),
            _ => None,
        }
    }

    /// `route_for`, keyed by the job's account: the limiter paces
    /// `Account`-scoped policies per account (rung 11) and serializes per
    /// `(account, policy)`; probes are per `(account, route)`.
    fn keyed_route_for(
        &self,
        kind: &str,
        params: &Value,
        account: Option<&str>,
    ) -> Option<(String, String)> {
        let (route, url) = self.route_for(kind, params)?;
        Some((endpoint_key(&route, account), url))
    }

    /// Whether a route needs the bearer token. The mock's `fetch` is open;
    /// everything on the real API is not.
    fn needs_auth(&self, route: &str) -> bool {
        self.provider.is_real() || route != "fetch"
    }

    /// What a job must not overlap with: probes with each other (N18),
    /// network jobs with anything under the same policy, everything else
    /// only with itself.
    fn serial_key(&self, e: &Entry) -> String {
        if e.info.kind == "probe" {
            return "probe".into();
        }
        match self.keyed_route_for(&e.info.kind, &e.params, e.info.account.as_deref()) {
            Some((route, _)) => self.choke.serial_key(&route),
            None => format!("solo:{}", e.info.id),
        }
    }

    fn probe_pending(s: &Shared, route: &str) -> bool {
        s.jobs.values().any(|e| {
            e.info.kind == "probe"
                && !e.info.state.is_terminal()
                && e.params.get("route").and_then(Value::as_str) == Some(route)
        })
    }

    /// Per-route knowledge about GGG that headers cannot teach, kept in
    /// one place (`LIVE-TESTING.md` run ledger, 2026-08-30):
    /// - `/profile`: HEAD is answered 403, and GET answers 200 with no
    ///   `X-Rate-Limit-*` headers at all — not probed, may be policyless.
    /// - `/account/leagues`: HEAD is answered 200 and **counted** as a hit
    ///   (state `1:10:0` after the probe; N24's free HEAD is per endpoint),
    ///   so a probe costs what the GET costs — not probed; the GET teaches.
    ///
    /// Every other route is probed (HEAD uncounted) and observed strictly.
    /// Owner decisions 2026-08-30, pending GGG's word on both endpoints.
    fn declare_route_knowledge(choke: &ChokePoint) {
        choke.declare_policyless("profile");
    }

    /// Routes whose first use is not preceded by a HEAD probe (see
    /// `declare_route_knowledge`).
    const NO_PROBE_ROUTES: &'static [&'static str] = &["profile", "league"];

    /// Whether a route's first use is preceded by a HEAD probe. `route` is
    /// the endpoint key (`profile@user`).
    fn route_probes(route: &str) -> bool {
        !Self::NO_PROBE_ROUTES.contains(&crate::ratelimit::split_endpoint_key(route).0)
    }

    /// Make sure a probe for `route` is queued or running; submit one if not.
    /// One probe per route per daemon lifetime in the normal case — the
    /// sanctioned "one HEAD at startup" (N16), sent lazily on first use.
    fn ensure_probe(&self, route: &str, url: &str, account: Option<String>) {
        let pending = Self::probe_pending(&self.shared.lock().unwrap(), route);
        if !pending {
            self.log(&format!(
                "route {route} unknown; probing {} first",
                url_path(url)
            ));
            if let Err(e) = self.submit(
                "probe".into(),
                json!({ "route": route, "url": url }),
                PROBE_PRIORITY,
                "daemon".into(),
                account,
            ) {
                // Queue failed: the dispatcher has stopped picking, so the
                // jobs behind this probe wait rather than spin.
                self.log(&format!("JOBS: could not queue a probe for {route}: {e}"));
            }
        }
    }

    /// Turn a client's account selector into the account a job runs as.
    /// One live session for now: the selector must name it (or be absent);
    /// anything else is refused at submit, before a job exists.
    fn resolve_account(
        &self,
        kind: &str,
        requested: Option<&str>,
    ) -> Result<Option<String>, String> {
        let s = self.shared.lock().unwrap();
        match requested {
            Some(req) => match s.auth.matching(req) {
                Some(session) => Ok(session.username.clone()),
                None if s.auth.by_account.is_empty() => {
                    Err(format!("account {req:?}: not logged in — run `acq auth`"))
                }
                None => Err(format!(
                    "account {req:?} is not logged in (live: {}); log in as it first",
                    s.auth.usernames().join(", ")
                )),
            },
            // A job that will need a token must not exist without an
            // account: it would slip past the token-time check. With
            // several sessions live, nothing is guessed.
            None if !self.kind_needs_account(kind) => Ok(None),
            None => s.auth.get(None).map(|session| session.username.clone()),
        }
    }

    /// A client's selector (username, name without discriminator, or uuid)
    /// as the live session's canonical username; `None` stays `None` (the
    /// sole session, or a refusal downstream when several are live).
    fn canonical_account(&self, selector: Option<&str>) -> Result<Option<String>, String> {
        let Some(sel) = selector else { return Ok(None) };
        let s = self.shared.lock().unwrap();
        s.auth
            .matching(sel)
            .and_then(|x| x.username.clone())
            .map(Some)
            .ok_or_else(|| format!("no session for {sel} — run `acq auth`"))
    }

    /// Every kind that sends with a token. `sleep` never sends; the mock's
    /// `fetch` is open.
    fn kind_needs_account(&self, kind: &str) -> bool {
        match kind {
            "sleep" => false,
            "fetch" => self.provider.is_real(),
            _ => true,
        }
    }

    fn submit(
        &self,
        kind: String,
        params: Value,
        priority: Priority,
        submitted_by: String,
        account: Option<String>,
    ) -> Result<JobId, String> {
        // An `apply` is admitted or refused whole, before a job id exists
        // (CONTEXT.md, decided 2026-09-01): vocabulary and budget checked
        // here, so a refusal admits nothing. A realm a kind's family does
        // not take is refused the same way, before an id exists.
        if kind == "apply" {
            validate_apply(&params)?;
        }
        admit_realm(&kind, &params)?;
        self.submit_with_parent(kind, params, priority, submitted_by, account, None)
    }

    /// A child inherits its parent's priority, submitter, and account.
    /// The cancellation guard lives in `submit_with_parent`, inside the
    /// same critical section as the insert.
    fn submit_child(&self, parent: JobId, kind: &str, params: Value) -> Result<JobId, String> {
        let (priority, by, account) = {
            let s = self.shared.lock().unwrap();
            let p = s
                .jobs
                .get(&parent)
                .ok_or_else(|| format!("parent job {parent} is gone"))?;
            (
                p.info.priority,
                p.info.submitted_by.clone(),
                p.info.account.clone(),
            )
        };
        self.submit_with_parent(kind.into(), params, priority, by, account, Some(parent))
    }

    /// A job exists only once its row does: an insert that fails rolls the
    /// id back and refuses the submit, so memory can never run ahead of
    /// the ids disk has seen.
    fn submit_with_parent(
        &self,
        kind: String,
        params: Value,
        priority: Priority,
        submitted_by: String,
        account: Option<String>,
        parent: Option<JobId>,
    ) -> Result<JobId, String> {
        if let Some(e) = self.queue_failed() {
            return Err(format!(
                "the persisted queue failed ({e}); the daemon refuses new jobs — restart it once daemon.db is writable"
            ));
        }
        let info = {
            let mut s = self.shared.lock().unwrap();
            // The parent guard sits here, inside the same critical section
            // as the insert: `cancel` takes this lock to enumerate
            // children, so a child can never be inserted after a
            // cancellation has swept and missed it.
            if let Some(pid) = parent {
                let p = s
                    .jobs
                    .get(&pid)
                    .ok_or_else(|| format!("parent job {pid} is gone"))?;
                if p.info.state.is_terminal() || p.cancel_requested {
                    return Err(format!("parent job {pid} was cancelled"));
                }
            }
            s.last_activity = Instant::now();
            let id = s.next_id;
            s.next_id += 1;
            let info = JobInfo {
                id,
                kind,
                state: JobState::Waiting,
                priority,
                submitted_by,
                eta_seconds: None,
                parent,
                retries: 0,
                account,
                params: params.clone(),
            };
            let entry = Entry {
                info: info.clone(),
                params,
                outcome: None,
                cancel_requested: false,
                deferred: None,
                submitted_at: unix_now(),
            };
            if !self.persist(&entry) {
                s.next_id -= 1;
                let e = self.queue_failed().unwrap_or_default();
                return Err(format!(
                    "the persisted queue failed ({e}); the daemon refuses new jobs — restart it once daemon.db is writable"
                ));
            }
            s.jobs.insert(id, entry);
            info
        };
        let id = info.id;
        self.emit(info);
        self.work.notify_one();
        Ok(id)
    }

    fn cancel(&self, id: JobId) -> Result<(), String> {
        // Cancelling a parent cancels everything under it: waiting
        // descendants immediately, running ones at their next slice.
        let mut emits = Vec::new();
        let mut persist_failed = false;
        {
            let mut s = self.shared.lock().unwrap();
            let entry = s.jobs.get(&id).ok_or_else(|| format!("no job {id}"))?;
            if entry.info.state.is_terminal() {
                return Err(format!("job {id} already {}", entry.info.state));
            }
            let mut targets = vec![id];
            let mut i = 0;
            while i < targets.len() {
                let p = targets[i];
                targets.extend(
                    s.jobs
                        .values()
                        .filter(|e| e.info.parent == Some(p))
                        .map(|e| e.info.id),
                );
                i += 1;
            }
            for t in targets {
                let entry = s.jobs.get_mut(&t).unwrap();
                match entry.info.state {
                    JobState::Waiting => {
                        entry.info.state = JobState::Cancelled;
                        entry.outcome = Some(Outcome::Cancelled);
                        emits.push(entry.info.clone());
                    }
                    // A parent waiting on children isn't on any worker;
                    // cancel it outright.
                    JobState::Running if entry.deferred.is_some() => {
                        entry.deferred = None;
                        entry.info.state = JobState::Cancelled;
                        entry.outcome = Some(Outcome::Cancelled);
                        emits.push(entry.info.clone());
                    }
                    JobState::Running => entry.cancel_requested = true,
                    _ => continue,
                }
                persist_failed |= !self.persist(entry);
            }
        }
        // A cancelled child may have been the last thing its parent was
        // holding for; without this the parent would wait forever.
        let parents: HashSet<JobId> = emits
            .iter()
            .filter(|i| i.state.is_terminal())
            .filter_map(|i| i.parent)
            .collect();
        for info in emits {
            self.emit(info);
        }
        for pid in parents {
            self.maybe_finish_parent(pid);
        }
        if persist_failed {
            // The cancellation holds for this lifetime — nothing here will
            // send — but disk may still say waiting/running, so a restart
            // can revive the job. Say so instead of claiming success.
            return Err(
                "cancelled for this daemon's lifetime, but the queue write failed — the \
                 cancellation may not survive a restart; the daemon refuses new jobs until then"
                    .to_string(),
            );
        }
        Ok(())
    }

    fn set_priority(&self, id: JobId, priority: Priority) -> Result<(), String> {
        let mut s = self.shared.lock().unwrap();
        let entry = s.jobs.get_mut(&id).ok_or_else(|| format!("no job {id}"))?;
        if entry.info.state != JobState::Waiting {
            return Err(format!("job {id} is {}, not waiting", entry.info.state));
        }
        let before = entry.info.priority;
        entry.info.priority = priority;
        if !self.persist(entry) {
            entry.info.priority = before;
            return Err(
                "priority unchanged: the queue write failed; the daemon refuses new work until restart"
                    .to_string(),
            );
        }
        Ok(())
    }

    fn cancelled(&self, id: JobId) -> bool {
        let s = self.shared.lock().unwrap();
        s.jobs.get(&id).map(|e| e.cancel_requested).unwrap_or(true)
    }

    // ---- dispatcher -----------------------------------------------------

    /// Starts at most one task per scheduling key. N4's gate, not this
    /// dispatcher, owns actual-send capacity. Woken by submits, completions,
    /// and reprioritization.
    async fn dispatcher(self: Arc<Self>) {
        loop {
            let picks = self.pick_runnable();
            for id in picks {
                tokio::spawn(self.clone().run_active(id));
            }
            self.work.notified().await;
        }
    }

    /// Waiting jobs, in dispatch order, that can start a task right now.
    fn pick_runnable(&self) -> Vec<JobId> {
        let mut s = self.shared.lock().unwrap();
        // A failed queue means no new dispatch at all: every transition
        // a job makes from here would widen the memory/disk divergence.
        if self.queue_failed().is_some() {
            return Vec::new();
        }
        let mut busy: HashSet<String> = s.active_jobs.values().cloned().collect();
        let halted = self.rails().halted().is_some();
        let mut picks = Vec::new();
        for id in s.queue_order() {
            let entry = &s.jobs[&id];
            if let Some((route, _)) = self.keyed_route_for(
                &entry.info.kind,
                &entry.params,
                entry.info.account.as_deref(),
            ) {
                // A halted daemon (LIVE-TESTING.md rails) sends nothing:
                // network jobs wait, on disk, for `reset-tripwire`.
                if halted {
                    continue;
                }
                // A job whose route is still being probed has nothing to do yet.
                if self.choke.endpoint_state(&route) == EndpointState::Unknown
                    && Self::probe_pending(&s, &route)
                {
                    continue;
                }
            }
            let key = self.serial_key(entry);
            if busy.contains(&key) {
                continue;
            }
            busy.insert(key.clone());
            picks.push((id, key));
        }
        for (id, key) in &picks {
            s.active_jobs.insert(*id, key.clone());
        }
        picks.into_iter().map(|(id, _)| id).collect()
    }

    async fn run_active(self: Arc<Self>, id: JobId) {
        self.process(id).await;
        self.shared.lock().unwrap().active_jobs.remove(&id);
        self.work.notify_one();
    }

    async fn process(&self, id: JobId) {
        let ready = self.choke.now();
        let (route, account) = {
            let s = self.shared.lock().unwrap();
            match s.jobs.get(&id) {
                Some(e) => (
                    self.keyed_route_for(&e.info.kind, &e.params, e.info.account.as_deref()),
                    e.info.account.clone(),
                ),
                None => return,
            }
        };

        // L0 rail 1: a halted daemon sends nothing — no probe, no pacing
        // wait, no permit. The job stays waiting (the dispatcher will not
        // pick it again until the reset); its key is given back.
        if route.is_some() && self.rails().halted().is_some() {
            return;
        }

        // A route we've never heard from gets a probe first (N16, N24) —
        // unless HEAD is known not to be accepted there; a degraded one
        // (N20) fails its jobs cleanly until the cooldown ends.
        if let Some((route, url)) = &route {
            match self.choke.endpoint_state(route) {
                EndpointState::Unknown if Self::route_probes(route) => {
                    self.ensure_probe(route, url, account);
                    return; // scheduling key released; the probe outranks us
                }
                EndpointState::Degraded { until, reason } => {
                    let left = until.saturating_duration_since(self.choke.now()).as_secs();
                    let error = format!("route {route} is degraded for another {left}s: {reason}");
                    self.note_error(&format!("job {id}: {error}"));
                    self.start_and_finish(id, Outcome::Failure { error });
                    return;
                }
                // A no-probe route's first GET is what teaches the limiter.
                EndpointState::Unknown | EndpointState::Policy(_) | EndpointState::Policyless => {}
            }
        }

        // Dispatcher pacing remains while the job is still `waiting`, in
        // short slices so cancellation and reprioritization stay responsive.
        // This is only a scheduling hint: after authentication, ChokePoint
        // repeats the final limiter check under the actual-send permit.
        if let Some((route, _)) = &route {
            loop {
                let step = {
                    let s = self.shared.lock().unwrap();
                    let Some(me) = s.jobs.get(&id) else { return };
                    if me.info.state != JobState::Waiting {
                        return; // cancelled out from under us
                    }
                    // A higher-priority job on the same key may have arrived;
                    // give the key back so the dispatcher picks it instead.
                    let my_key = self.serial_key(me);
                    let outranked = s
                        .queue_order()
                        .into_iter()
                        .take_while(|&q| q != id)
                        .any(|q| s.jobs.get(&q).is_some_and(|e| self.serial_key(e) == my_key));
                    if outranked {
                        return;
                    }
                    self.choke.check(route)
                };
                match step {
                    Ok(()) => break,
                    Err(d) => self.choke.sleep(d.min(Duration::from_secs(1))).await,
                }
            }
        }

        let job = {
            let mut s = self.shared.lock().unwrap();
            let Some(entry) = s.jobs.get_mut(&id) else {
                return;
            };
            if entry.info.state != JobState::Waiting {
                return;
            }
            entry.info.state = JobState::Running;
            if !self.persist(entry) {
                // Disk still says waiting: after a restart this run would
                // be invisible (a no-probe job would replay blind). A send
                // the queue cannot see must not happen — revert, don't run.
                entry.info.state = JobState::Waiting;
                return;
            }
            (entry.info.clone(), entry.params.clone())
        };
        let (info, params) = job;
        let kind = info.kind.clone();
        self.emit(info);

        let route = route.map(|(route, _)| route);
        let outcome = match self
            .execute(id, &kind, params, route, account.as_deref(), ready)
            .await
        {
            Exec::Done(outcome) => outcome,
            Exec::RateLimited(evidence) => {
                // P-A: a 429 is recovered from, not surfaced — unless it keeps
                // happening. The limiter already holds the policy for
                // Retry-After + bucket (N19); putting the job back to waiting
                // (it keeps its place: order is priority, then id) makes it
                // go out after that hold, with the ETA visible meanwhile.
                let requeued = {
                    let mut s = self.shared.lock().unwrap();
                    let Some(entry) = s.jobs.get_mut(&id) else {
                        return;
                    };
                    if may_requeue_429(entry.info.retries, entry.cancel_requested) {
                        entry.info.retries += 1;
                        entry.info.state = JobState::Waiting;
                        self.persist(entry);
                        Some(entry.info.clone())
                    } else {
                        None
                    }
                };
                match requeued {
                    Some(info) => {
                        self.note_error(&format!(
                            "job {id} ({kind}): rate limited (429), re-queued behind the limiter's hold (retry {}/{MAX_429_RETRIES}): {evidence}",
                            info.retries
                        ));
                        self.emit(info);
                        return; // scheduling key released; re-pick after the hold
                    }
                    None => Outcome::Failure {
                        error: format!(
                            "rate limited (429) {MAX_429_RETRIES} times; giving up (N10): {evidence}"
                        ),
                    },
                }
            }
        };
        if let Outcome::Failure { error } = &outcome {
            self.note_error(&format!("job {id} ({kind}): {error}"));
        }
        self.conclude(id, outcome);
    }

    /// A finished job's landing. A job that spawned children holds its
    /// own result until they're all done (its scheduling key is already
    /// given back so children can run); anything else finishes. A
    /// cancellation that landed during the fan-out is honored here, under
    /// the lock: the held result becomes `Cancelled`, so a parent
    /// cancelled after its last child was submitted can never report
    /// success.
    fn conclude(&self, id: JobId, outcome: Outcome) {
        let has_children = {
            let mut s = self.shared.lock().unwrap();
            let spawned = s.jobs.values().any(|e| e.info.parent == Some(id));
            if spawned
                && let Some(entry) = s.jobs.get_mut(&id)
                && entry.info.state == JobState::Running
            {
                entry.deferred = Some(if entry.cancel_requested {
                    Outcome::Cancelled
                } else {
                    outcome.clone()
                });
                self.persist(entry);
            }
            spawned
        };
        if has_children {
            self.maybe_finish_parent(id);
        } else {
            self.finish(id, outcome);
        }
    }

    /// If `pid` is waiting on children and none are left running, finish it
    /// with its held-back result plus a summary of what the children did.
    fn maybe_finish_parent(&self, pid: JobId) {
        let final_outcome = {
            let mut s = self.shared.lock().unwrap();
            let Some(parent) = s.jobs.get(&pid) else {
                return;
            };
            if parent.deferred.is_none() {
                return;
            }
            let children: Vec<&Entry> = s
                .jobs
                .values()
                .filter(|e| e.info.parent == Some(pid))
                .collect();
            if children.iter().any(|e| !e.info.state.is_terminal()) {
                return;
            }
            let (mut done, mut failed, mut cancelled) = (0, 0, 0);
            let mut failed_ids = Vec::new();
            for c in &children {
                match c.info.state {
                    JobState::Done => done += 1,
                    JobState::Failed => {
                        failed += 1;
                        failed_ids.push(c.info.id);
                    }
                    _ => cancelled += 1,
                }
            }
            let total = children.len();
            // Sorted for every reader: the ids are the remedy's handles
            // (`acq result <id>`), and a HashMap's order is noise.
            failed_ids.sort_unstable();
            let summary = json!({ "done": done, "failed": failed, "cancelled": cancelled, "failed_ids": failed_ids });
            let deferred = s.jobs.get_mut(&pid).unwrap().deferred.take().unwrap();
            match deferred {
                Outcome::Success { mut payload } if failed == 0 && cancelled == 0 => {
                    payload["children"] = summary;
                    Outcome::Success { payload }
                }
                // Cancelled children mean the work was not completed; a
                // parent must not call that success.
                Outcome::Success { .. } if failed == 0 => Outcome::Failure {
                    error: format!("{cancelled} of {total} child jobs were cancelled"),
                },
                Outcome::Success { .. } => Outcome::Failure {
                    error: format!(
                        "{failed} of {total} child jobs failed: {failed_ids:?} (acq result <id> for each)"
                    ),
                },
                other => other,
            }
        };
        self.finish(pid, final_outcome);
    }

    /// The one place a job becomes terminal (`start_and_finish` and
    /// `maybe_finish_parent` both land here). A pending cancellation is
    /// arbitrated under this final lock: if `cancel` set the flag at any
    /// point before terminalization — including between a caller
    /// computing this outcome and the lock below — the job finishes
    /// `Cancelled`, never with the stale outcome. Cancellation wins the
    /// job's terminal surface; work already done may have sent regardless
    /// (sends are committed once dispatched), and storable API responses
    /// were recorded as they landed — though not every outcome is stored
    /// separately (probes, mock fetches, and failures are not).
    fn finish(&self, id: JobId, outcome: Outcome) {
        let info = {
            let mut s = self.shared.lock().unwrap();
            let Some(entry) = s.jobs.get_mut(&id) else {
                return;
            };
            let outcome = if entry.cancel_requested {
                Outcome::Cancelled
            } else {
                outcome
            };
            entry.info.state = match &outcome {
                Outcome::Success { .. } => JobState::Done,
                Outcome::Failure { .. } => JobState::Failed,
                Outcome::Cancelled => JobState::Cancelled,
            };
            entry.outcome = Some(outcome);
            self.persist(entry);
            entry.info.clone()
        };
        let parent = info.parent;
        self.emit(info);
        if let Some(pid) = parent {
            self.maybe_finish_parent(pid);
        }
    }

    /// For jobs that fail before they ever run: pass through `running` so
    /// subscribers see the same state sequence as every other job.
    fn start_and_finish(&self, id: JobId, outcome: Outcome) {
        let info = {
            let mut s = self.shared.lock().unwrap();
            let Some(entry) = s.jobs.get_mut(&id) else {
                return;
            };
            if entry.info.state != JobState::Waiting {
                return;
            }
            entry.info.state = JobState::Running;
            self.persist(entry);
            entry.info.clone()
        };
        self.emit(info);
        self.finish(id, outcome);
    }

    /// `ready` is when the dispatcher picked the job: every send it makes
    /// journals its wait from that instant.
    async fn execute(
        &self,
        id: JobId,
        kind: &str,
        params: Value,
        route: Option<String>,
        account: Option<&str>,
        ready: Instant,
    ) -> Exec {
        // Network kinds bubble a 429 up as `Exec::RateLimited`; everything
        // else is an ordinary outcome.
        match self
            .execute_inner(id, kind, params, route, account, ready)
            .await
        {
            Ok(outcome) => Exec::Done(outcome),
            Err(ApiError::RateLimited(evidence)) => Exec::RateLimited(evidence),
            Err(ApiError::Protocol(error)) => Exec::Done(Outcome::Failure { error }),
            Err(ApiError::Other(error)) => Exec::Done(Outcome::Failure { error }),
        }
    }

    async fn execute_inner(
        &self,
        id: JobId,
        kind: &str,
        params: Value,
        route: Option<String>,
        account: Option<&str>,
        ready: Instant,
    ) -> Result<Outcome, ApiError> {
        Ok(match kind {
            // The one real API call: GET {api_base}/character. Same code in
            // both modes; only the provider's base URL differs. The choke
            // point classifies and observes every response; the dispatcher
            // requeues only acceptable 429s, while Cloudflare-shaped blocks
            // are never retried.
            "characters" => {
                let (token, username) = match self.valid_access_token(account, false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let Some((_, url)) = self.route_for(kind, &params) else {
                    return Ok(Outcome::Failure {
                        error: "characters: unrenderable realm".into(),
                    });
                };
                let route = route.as_deref().expect("characters is a network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                // A 2xx without the `characters` array is a malformed
                // response, not an empty account: the job fails and
                // nothing is recorded (the store would refuse it too —
                // ingesting it as empty would retire every character).
                let Some(characters) = v.get("characters").and_then(Value::as_array) else {
                    return Ok(Outcome::Failure {
                        error: "characters response without a `characters` array".into(),
                    });
                };
                let characters = json!(characters);
                if let Err(failure) = self.record(account, kind, &params, &v) {
                    return Ok(failure);
                }
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "username": username,
                        "characters": characters,
                        "rate_limit": rate,
                    }),
                }
            }
            "character" | "leagues" | "profile" => {
                // A profile job during a login runs on the staged tokens
                // (`staged_profile_token`): the account may have no
                // registered session yet, or a dead one being replaced —
                // the new tokens are the ones being proven.
                let staged = (kind == "profile")
                    .then(|| self.staged_profile_token(account))
                    .flatten();
                let (token, username) = match staged {
                    Some(pair) => pair,
                    None => match self.valid_access_token(account, false).await {
                        Ok(pair) => pair,
                        Err(error) => return Ok(Outcome::Failure { error }),
                    },
                };
                let Some((_, url)) = self.route_for(kind, &params) else {
                    return Ok(Outcome::Failure {
                        error: format!("{kind} needs a name"),
                    });
                };
                let route = route.as_deref().expect("network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                if let Err(failure) = self.record(account, kind, &params, &v) {
                    return Ok(failure);
                }
                // The profile's uuid is the stable account identity,
                // required at login (`complete_login` writes the index
                // entry). Recording it whenever a later profile lands too
                // keeps the mapping fresh across renames and backfills
                // pre-uuid entries.
                if kind == "profile"
                    && let (Some(account), Some(uuid)) =
                        (account, v.get("uuid").and_then(Value::as_str))
                {
                    {
                        let mut s = self.shared.lock().unwrap();
                        if let Some(session) = s.auth.find_mut(account) {
                            session.uuid = Some(uuid.to_string());
                        }
                    }
                    self.with_index(|index| index.set_uuid(account, uuid));
                }
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "username": username,
                        "params": params,
                        "body": v,
                        "rate_limit": rate,
                    }),
                }
            }
            // The stash list: one request under stash-list-request-limit, the
            // second real policy. Tab contents are a later step.
            "stashes" => {
                let (token, username) = match self.valid_access_token(account, false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard")
                    .to_string();
                let Some((_, url)) = self.route_for(kind, &params) else {
                    return Ok(Outcome::Failure {
                        error: "stashes: unrenderable realm".into(),
                    });
                };
                let route = route.as_deref().expect("stashes is a network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                // A 2xx without the `stashes` array is a malformed
                // response, not an empty account: the job fails and
                // nothing is recorded — ingested as empty it would retire
                // every tab and mint a false listing basis.
                let Some(stashes) = v.get("stashes").and_then(Value::as_array) else {
                    return Ok(Outcome::Failure {
                        error: "stashes response without a `stashes` array".into(),
                    });
                };
                let stashes = json!(stashes);
                if let Err(failure) = self.record(account, kind, &params, &v) {
                    return Ok(failure);
                }
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "username": username,
                        "league": league,
                        "stashes": stashes,
                        "rate_limit": rate,
                    }),
                }
            }
            "stash" => {
                let (token, _) = match self.valid_access_token(account, false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let Some((_, url)) = self.route_for("stash", &params) else {
                    return Ok(Outcome::Failure {
                        error: "stash needs an id".into(),
                    });
                };
                let route = route.as_deref().expect("stash is a network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                if let Err(failure) = self.record(account, kind, &params, &v) {
                    return Ok(failure);
                }
                let stash = v.get("stash").cloned().unwrap_or(v);
                // Map/unique tabs carry their substashes as stubs; following
                // them is opt-in per tab (--deep) because one map tab can
                // hold hundreds. Each substash becomes a child job.
                let deep = params.get("deep").and_then(Value::as_bool).unwrap_or(false);
                let children = stash
                    .get("children")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let mut submitted = Vec::new();
                if deep {
                    let realm = Realm::from_params(&params).unwrap_or(Realm::DEFAULT);
                    let league = params.get("league").cloned().unwrap_or(json!("Standard"));
                    let tab = params.get("id").cloned().unwrap_or(Value::Null);
                    for child in &children {
                        let Some(sub) = child.get("id").and_then(Value::as_str) else {
                            continue;
                        };
                        match self.submit_child(
                            id,
                            "stash",
                            json!({ "realm": realm, "league": league, "id": tab, "sub": sub, "deep": false }),
                        ) {
                            Ok(cid) => submitted.push(cid),
                            Err(e) => {
                                return Ok(fan_out_stopped(
                                    self.cancelled(id),
                                    submitted.len(),
                                    &e,
                                ));
                            }
                        }
                    }
                }
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "stash": stash,
                        "substashes_listed": children.len(),
                        "substash_jobs": submitted,
                        "rate_limit": rate,
                    }),
                }
            }
            // A refresh: one stash-list request, then one `stash` child per
            // selected tab. Folder children come straight from the list
            // (folders themselves are never fetched); map/unique substashes
            // only if `deep`. Selection is explicit — there is no default.
            "refresh" => {
                let (token, _) = match self.valid_access_token(account, false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard")
                    .to_string();
                let deep = params.get("deep").and_then(Value::as_bool).unwrap_or(false);
                let all = params.get("all").and_then(Value::as_bool).unwrap_or(false);
                let wanted: Vec<String> = params
                    .get("tabs")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                if !all && wanted.is_empty() {
                    return Ok(Outcome::Failure {
                        error: "refresh needs --all or --tabs <id,...>".into(),
                    });
                }
                let realm = Realm::from_params(&params).unwrap_or(Realm::DEFAULT);
                let Some((_, url)) = self.route_for(kind, &params) else {
                    return Ok(Outcome::Failure {
                        error: "refresh: unrenderable realm".into(),
                    });
                };
                let route = route.as_deref().expect("refresh is a network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                // A malformed listing fails the refresh whole, before the
                // store sees it: converted to an empty list it would
                // "succeed" with zero children over a retired tab set.
                let Some(listed) = v.get("stashes").and_then(Value::as_array).cloned() else {
                    return Ok(Outcome::Failure {
                        error: "stashes response without a `stashes` array".into(),
                    });
                };
                // The list a refresh fetches is the same response `stashes`
                // records; the store wants it under that endpoint.
                if let Err(failure) = self.record(
                    account,
                    "stashes",
                    &json!({ "realm": realm, "league": params.get("league").cloned().unwrap_or(json!("Standard")) }),
                    &v,
                ) {
                    return Ok(failure);
                }
                // Flatten: top-level tabs plus folder children; skip folders.
                let mut tabs: Vec<(String, String, String)> = Vec::new();
                for t in &listed {
                    let ty = t.get("type").and_then(Value::as_str).unwrap_or("");
                    if ty == "Folder" {
                        for c in t
                            .get("children")
                            .and_then(Value::as_array)
                            .into_iter()
                            .flatten()
                        {
                            if let Some(cid) = c.get("id").and_then(Value::as_str) {
                                tabs.push((
                                    cid.into(),
                                    c.get("name").and_then(Value::as_str).unwrap_or("").into(),
                                    c.get("type").and_then(Value::as_str).unwrap_or("").into(),
                                ));
                            }
                        }
                    } else if let Some(tid) = t.get("id").and_then(Value::as_str) {
                        tabs.push((
                            tid.into(),
                            t.get("name").and_then(Value::as_str).unwrap_or("").into(),
                            ty.into(),
                        ));
                    }
                }
                let selected: Vec<&(String, String, String)> = tabs
                    .iter()
                    .filter(|(tid, _, _)| all || wanted.contains(tid))
                    .collect();
                let unknown: Vec<&String> = wanted
                    .iter()
                    .filter(|w| !tabs.iter().any(|(tid, _, _)| tid == *w))
                    .collect();
                let mut submitted = Vec::new();
                for (tid, _, ty) in &selected {
                    let follow = deep && matches!(ty.as_str(), "MapStash" | "UniqueStash");
                    match self.submit_child(
                        id,
                        "stash",
                        json!({ "realm": realm, "league": league, "id": tid, "deep": follow }),
                    ) {
                        Ok(cid) => submitted.push(cid),
                        Err(e) => {
                            return Ok(fan_out_stopped(self.cancelled(id), submitted.len(), &e));
                        }
                    }
                }
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "league": league,
                        "tabs_listed": tabs.len(),
                        "tabs_selected": selected.iter().map(|(i, n, t)| json!({ "id": i, "name": n, "type": t })).collect::<Vec<_>>(),
                        "unknown_tab_ids": unknown,
                        "deep": deep,
                        "tab_jobs": submitted,
                        "rate_limit": rate,
                    }),
                }
            }
            // A plan's action set, executed exactly (tracer step 7): a pure
            // fan-out parent. Each admitted tuple becomes one child job; the
            // parent sends nothing itself, so it has no route and no token —
            // children authenticate and pace themselves. Admission validated
            // the vocabulary at submit; it is checked again here because a
            // restored `daemon.db` row was admitted by an earlier lifetime,
            // possibly an earlier build.
            "apply" => {
                if let Err(error) = validate_apply(&params) {
                    return Ok(Outcome::Failure { error });
                }
                let jobs = params
                    .get("jobs")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let mut submitted = Vec::new();
                for job in &jobs {
                    let kind = job.get("kind").and_then(Value::as_str).unwrap_or_default();
                    let child_params = job.get("params").cloned().unwrap_or_else(|| json!({}));
                    match self.submit_child(id, kind, child_params) {
                        Ok(cid) => submitted.push(cid),
                        Err(e) => {
                            return Ok(fan_out_stopped(self.cancelled(id), submitted.len(), &e));
                        }
                    }
                }
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "requests": submitted.len(),
                        "child_jobs": submitted,
                    }),
                }
            }
            "sleep" => {
                let seconds = params.get("seconds").and_then(Value::as_f64).unwrap_or(3.0);
                let deadline = Instant::now() + Duration::from_secs_f64(seconds);
                while Instant::now() < deadline {
                    if self.cancelled(id) {
                        return Ok(Outcome::Cancelled);
                    }
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                Outcome::Success {
                    payload: json!({ "slept_seconds": seconds }),
                }
            }
            "fetch" => {
                // A real request to the mock's fake data endpoint, so the
                // limiter learns a second policy from real headers. Never
                // sent to GGG: real mode has no such endpoint.
                if self.provider.is_real() {
                    return Ok(Outcome::Failure {
                        error: "fetch is a mock-only kind; real mode has no fake data endpoint"
                            .into(),
                    });
                }
                let url = format!("{}/fetch", self.provider.api_base);
                let route = route.as_deref().expect("mock fetch is a network kind");
                let (v, rate) = self.api_get(route, &url, None, ready).await?;
                Outcome::Success {
                    payload: json!({
                        "note": "fake data from the in-process mock",
                        "params": params,
                        "items": v.get("items").cloned().unwrap_or(v),
                        "rate_limit": rate,
                    }),
                }
            }
            "whoami" => {
                // The mock-only auth-exercising kind: access-token expiry and
                // silent refresh through the daemon-owned session. Fake data,
                // so in real mode it must not cost a token POST (L0 rail 6).
                if self.provider.is_real() {
                    return Ok(Outcome::Failure {
                        error: "whoami is a mock-only job kind; not run against real GGG".into(),
                    });
                }
                let (token, username) = match self.valid_access_token(account, false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                tokio::time::sleep(Duration::from_millis(300)).await;
                if self.cancelled(id) {
                    return Ok(Outcome::Cancelled);
                }
                Outcome::Success {
                    payload: json!({
                        "note": "fake data — playground never talks to GGG",
                        "username": username,
                        "league": "Standard",
                        "characters": [
                            { "name": "StashHoarder", "class": "Scion", "level": 97 },
                            { "name": "MuleQuadTab", "class": "Witch", "level": 12 },
                        ],
                        "authorized_with": format!("{}…", token.chars().take(11).collect::<String>()),
                    }),
                }
            }
            // The HEAD probe (N16): discovers an endpoint's policy and the
            // account's current counters before the first real send, without
            // counting against the policy (N24). Submitted by the daemon
            // itself; visible like any other job so it can be inspected.
            "probe" => {
                let (Some(route), Some(url)) = (
                    params
                        .get("route")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    params
                        .get("url")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                ) else {
                    return Ok(Outcome::Failure {
                        error: "probe needs a route and a url".into(),
                    });
                };
                let bearer = if self.needs_auth(&route) {
                    match self.valid_access_token(account, false).await {
                        Ok((token, _)) => Some(token),
                        Err(error) => {
                            // Close the endpoint too, or the waiting job would
                            // just ask for another probe; login reopens it.
                            self.choke.degrade(&route, &error);
                            return Ok(Outcome::Failure { error });
                        }
                    }
                } else {
                    None
                };
                let probed = self
                    .choke
                    .head(&route, &url, bearer.as_deref(), ready)
                    .await;
                self.announce_trip();
                match probed {
                    Ok((status, policy, headers)) => {
                        let name = policy.name;
                        self.log(&format!(
                            "HEAD {} -> {status} | policy {name} | {headers}",
                            url_path(&url)
                        ));
                        Outcome::Success {
                            payload: json!({
                                "route": route,
                                "endpoint": url_path(&url),
                                "status": status.as_u16(),
                                "policy": name,
                                "rate_limit": headers,
                            }),
                        }
                    }
                    Err(error) => Outcome::Failure {
                        error: format!("HEAD {}: {error}", url_path(&url)),
                    },
                }
            }
            other => Outcome::Failure {
                error: format!(
                    "unknown job kind '{other}' (kinds: sleep, fetch, whoami, profile, characters, character, leagues, stashes, stash, refresh, apply, probe)"
                ),
            },
        })
    }

    /// One rate-limited GET: sends with the receipt, logs the rate headers,
    /// and turns non-2xx into a typed error. A 429 is distinguishable so the
    /// caller can re-queue the job behind the limiter's hold (P-A); a
    /// Cloudflare-shaped 403/503 is never retried (invariant 3).
    async fn api_get(
        &self,
        route: &str,
        url: &str,
        bearer: Option<&str>,
        ready: Instant,
    ) -> Result<(Value, Value), ApiError> {
        let response = match bearer {
            Some(token) => self.choke.get_bearer(route, url, token, ready).await,
            None => self.choke.get(route, url, ready).await,
        }
        .map_err(|error| match error {
            SendError::Protocol(error) => ApiError::Protocol(format!(
                "GET {}: rate-limit protocol failure: {error}",
                url_path(url)
            )),
            SendError::Transport(error) => ApiError::Other(format!("GET {url} failed: {error}")),
            SendError::Halted(cause) => {
                ApiError::Other(format!("GET {} refused: {cause}", url_path(url)))
            }
        })?;
        self.announce_trip();
        let status = response.status;
        let rate = response.rate;
        let retry_after = response.retry_after;
        let path = url_path(url);
        self.log(&format!("GET {path} -> {status} | rate headers: {rate}"));
        classify_api_body(status, &retry_after, &path, rate, response.body)
    }

    // ---- auth -----------------------------------------------------------

    /// Kick off a login: bind a loopback redirect listener, build the
    /// authorize URL, and spawn a task that waits for the browser callback
    /// and exchanges the code. Returns the URL for the user to open.
    async fn auth_start(self: &Arc<Self>) -> Result<String, String> {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| format!("could not bind loopback listener: {e}"))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        // The registered callback path; GGG accepts any loopback port.
        let redirect_uri = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
        let (verifier, challenge) = auth::pkce_pair();
        let state = auth::random_token("st");
        let mut authorize_url =
            url::Url::parse(&self.provider.authorize_url).map_err(|e| e.to_string())?;
        authorize_url
            .query_pairs_mut()
            .append_pair("response_type", "code")
            .append_pair("client_id", self.provider.client_id)
            .append_pair("redirect_uri", &redirect_uri)
            .append_pair("state", &state)
            .append_pair("code_challenge", &challenge)
            .append_pair("code_challenge_method", "S256")
            .append_pair("scope", &SCOPES.join(" "));
        let authorize_url = authorize_url.to_string();
        let flow_generation = self.begin_auth_flow();
        self.log(&format!("auth flow started (callback on port {port})"));

        let daemon = self.clone();
        tokio::spawn(async move {
            let callback =
                tokio::time::timeout(Duration::from_secs(300), wait_callback(listener, &state))
                    .await;
            match callback {
                Ok(Ok(code)) => {
                    match auth::exchange_code(
                        &daemon.choke,
                        &daemon.provider,
                        &code,
                        &verifier,
                        &redirect_uri,
                    )
                    .await
                    {
                        Ok(tokens) => match daemon.stage_auth_flow(flow_generation, tokens) {
                            Some(staged) => {
                                daemon.login_with_profile(flow_generation, staged).await;
                            }
                            None => daemon.log("stale auth flow completion ignored"),
                        },
                        Err(e) => daemon.finish_auth_flow_error(
                            flow_generation,
                            &format!("token exchange failed: {e}"),
                        ),
                    }
                }
                Ok(Err(e)) => daemon
                    .finish_auth_flow_error(flow_generation, &format!("auth callback failed: {e}")),
                Err(_) => {
                    daemon.finish_auth_flow_error(flow_generation, "auth flow timed out after 300s")
                }
            }
        });
        Ok(authorize_url)
    }

    /// A login flow is daemon-wide (one browser at a time), not a session:
    /// it disturbs no live session until its tokens arrive and name one.
    fn begin_auth_flow(&self) -> u64 {
        let mut s = self.shared.lock().unwrap();
        s.auth.next_flow = s.auth.next_flow.wrapping_add(1);
        let generation = s.auth.next_flow;
        s.auth.pending = Some(generation);
        // A fresh flow owns the login state: an older flow's staged tokens
        // and result must not leak into it.
        s.auth.staging = None;
        s.auth.flow_result = None;
        generation
    }

    /// Token exchange succeeded: hold the tokens in the staging slot —
    /// outside the session map, so clients cannot reach them and a live
    /// session for the same account keeps serving its jobs untouched. The
    /// flow stays pending; nothing is registered until
    /// [`Self::complete_login`] has the uuid. `None`: the flow is stale (a
    /// newer login superseded it).
    fn stage_auth_flow(&self, generation: u64, tokens: auth::TokenResponse) -> Option<String> {
        let mut s = self.shared.lock().unwrap();
        if s.auth.pending != Some(generation) {
            return None;
        }
        let username = tokens.username.clone();
        s.auth.staging = Some(Staging {
            flow: generation,
            username: username.clone(),
            access_token: tokens.access_token,
            access_expires_at: self.choke.wall() + Duration::from_secs(tokens.expires_in),
            refresh_token: tokens.refresh_token,
        });
        Some(username)
    }

    /// The staged login's access token, handed out only for the profile
    /// route of the account being logged in: the login's own profile job
    /// must use the *new* tokens (the account may have no registered
    /// session yet, or a dead one being replaced), and no other work can
    /// reach them.
    fn staged_profile_token(&self, account: Option<&str>) -> Option<(String, String)> {
        let s = self.shared.lock().unwrap();
        s.auth
            .staging
            .as_ref()
            .filter(|st| Some(st.username.as_str()) == account)
            .map(|st| (st.access_token.clone(), st.username.clone()))
    }

    /// The second half of a login (CONTEXT.md, identity decision): submit a
    /// profile job as the staged account — causal service of the client's
    /// `acq auth` — and register the session only when the uuid lands.
    /// Any failure fails the login whole: the staged tokens are dropped and
    /// nothing was written anywhere.
    async fn login_with_profile(&self, generation: u64, username: String) {
        let uuid = match self.submit(
            "profile".into(),
            json!({}),
            LOGIN_PRIORITY,
            "daemon".into(),
            Some(username.clone()),
        ) {
            Err(e) => Err(format!("could not queue the profile fetch: {e}")),
            Ok(id) => match self.wait_job_terminal(id, LOGIN_PROFILE_TIMEOUT).await {
                None => {
                    let _ = self.cancel(id);
                    Err(format!(
                        "profile fetch (job {id}) did not finish within {}s",
                        LOGIN_PROFILE_TIMEOUT.as_secs()
                    ))
                }
                Some(Outcome::Success { payload }) => payload
                    .get("body")
                    .and_then(|b| b.get("uuid"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .ok_or_else(|| "profile response carried no uuid".to_string()),
                Some(Outcome::Failure { error }) => Err(format!("profile fetch failed: {error}")),
                Some(Outcome::Cancelled) => Err("profile fetch was cancelled".into()),
            },
        };
        match uuid {
            Ok(uuid) => {
                self.complete_login(generation, &username, &uuid);
            }
            Err(error) => self.abort_login(generation, &username, &error),
        }
    }

    /// The uuid landed: register the login in dependency order — keyring,
    /// then the index mapping (a login the index cannot record **fails
    /// whole**, keyring rolled back: a session without its uuid mapping
    /// could neither be restored nor find its annotation file), and only
    /// then the visible session and the flow's terminal result. A stale
    /// flow registers nothing.
    fn complete_login(&self, generation: u64, username: &str, uuid: &str) -> bool {
        enum Done {
            Stale,
            Failed(String),
            Registered { warning: Option<String> },
        }
        let done = {
            let mut s = self.shared.lock().unwrap();
            if s.auth.pending != Some(generation) {
                Done::Stale
            } else if let Some(staging) = s
                .auth
                .staging
                .take_if(|st| st.flow == generation && st.username == username)
            {
                // Keyring and index writes happen under the auth lock so
                // they are ordered with logout and re-login.
                let saved = self.credential_store.save(
                    self.provider.keyring_service,
                    &staging.refresh_token,
                    username,
                );
                let (keyring, warning) = match &saved {
                    Ok(()) => ("ok".to_string(), None),
                    Err(e) => (
                        format!("unavailable: {e}"),
                        Some(format!(
                            "keyring save failed: {e} (session is in-memory only)"
                        )),
                    ),
                };
                let persisted = saved.is_ok();
                let indexed = self.index_apply(|index| {
                    index.record_login(username, uuid, persisted, acquisition_store::now())
                });
                match indexed {
                    Err(e) => {
                        let mut error =
                            format!("could not record the login in the account index: {e}");
                        if persisted
                            && let Err(clear) = self
                                .credential_store
                                .clear(self.provider.keyring_service, username)
                        {
                            error = format!(
                                "{error}; the keyring entry could not be rolled back either: {clear}"
                            );
                        }
                        s.auth.pending = None;
                        s.auth.flow_result = Some(Err(error.clone()));
                        Done::Failed(error)
                    }
                    Ok(()) => {
                        // A re-login replaces the account's old session here
                        // — not before — so generations advance and an
                        // in-flight refresh for it lands stale.
                        let session = s.auth.replace(AuthSession {
                            username: Some(username.to_string()),
                            uuid: Some(uuid.to_string()),
                            keyring,
                            ..AuthSession::default()
                        });
                        session.access_token = Some(staging.access_token);
                        session.access_expires_at = Some(staging.access_expires_at);
                        session.refresh_token = Some(staging.refresh_token);
                        session.advance_access_token();
                        session.advance_refresh_token();
                        s.auth.pending = None;
                        s.auth.flow_result = Some(Ok(username.to_string()));
                        Done::Registered { warning }
                    }
                }
            } else {
                let error = "the staged tokens disappeared before the profile landed".to_string();
                s.auth.pending = None;
                s.auth.flow_result = Some(Err(error.clone()));
                Done::Failed(error)
            }
        };
        match done {
            Done::Stale => {
                self.log("stale login completion ignored");
                false
            }
            Done::Failed(error) => {
                self.note_error(&format!("login failed for {username}: {error}"));
                false
            }
            Done::Registered { warning } => {
                // Only a completed login clears the dead-grant mark: until
                // here the keyring still held the rejected token.
                self.rails().clear_refresh_failed(username);
                // A probe that failed for lack of a session would succeed
                // now; don't make the user sit out the cooldown.
                self.choke.forget_degraded();
                self.log(&format!("logged in as {username} ({uuid})"));
                if let Some(warning) = warning {
                    self.note_error(&warning);
                }
                true
            }
        }
    }

    /// A login failed after token exchange: drop the staged tokens (it
    /// fails whole — no provisional identity) and, if the flow is still
    /// current, close it with the error as its terminal result.
    fn abort_login(&self, generation: u64, username: &str, error: &str) {
        let current = {
            let mut s = self.shared.lock().unwrap();
            if s.auth
                .staging
                .as_ref()
                .is_some_and(|st| st.flow == generation)
            {
                s.auth.staging = None;
            }
            if s.auth.pending == Some(generation) {
                s.auth.pending = None;
                s.auth.flow_result = Some(Err(error.to_string()));
                true
            } else {
                false
            }
        };
        if current {
            self.note_error(&format!("login failed for {username}: {error}"));
        }
    }

    /// Wait until job `id` is terminal, returning its outcome (`None` on
    /// timeout). Daemon-internal; clients wait through the protocol.
    async fn wait_job_terminal(&self, id: JobId, timeout: Duration) -> Option<Outcome> {
        let mut rx = self.events.subscribe();
        tokio::time::timeout(timeout, async {
            loop {
                let outcome = {
                    let s = self.shared.lock().unwrap();
                    s.jobs
                        .get(&id)
                        .filter(|e| e.info.state.is_terminal())
                        .and_then(|e| e.outcome.clone())
                };
                if let Some(outcome) = outcome {
                    return outcome;
                }
                match rx.recv().await {
                    Ok(_) | Err(broadcast::error::RecvError::Lagged(_)) => {}
                    Err(broadcast::error::RecvError::Closed) => std::future::pending().await,
                }
            }
        })
        .await
        .ok()
    }

    fn finish_auth_flow_error(&self, generation: u64, error: &str) {
        let current = {
            let mut s = self.shared.lock().unwrap();
            if s.auth.pending == Some(generation) {
                s.auth.pending = None;
                s.auth.flow_result = Some(Err(error.to_string()));
                true
            } else {
                false
            }
        };
        if current {
            self.note_error(error);
        }
    }

    /// Store fresh tokens in memory and mirror the refresh token (which the
    /// provider rotates on every grant) into the keyring. The caller holds
    /// the auth-state lock so keyring mutation is ordered with logout and
    /// re-authentication — and updates the account index *after* releasing
    /// it (`with_index` reports errors through `note_error`, which takes
    /// the same lock).
    fn install_tokens_locked(
        &self,
        session: &mut AuthSession,
        tokens: auth::TokenResponse,
    ) -> Option<String> {
        let (keyring, warning) = match self.credential_store.save(
            self.provider.keyring_service,
            &tokens.refresh_token,
            &tokens.username,
        ) {
            Ok(()) => ("ok".to_string(), None),
            Err(e) => {
                let warning = format!("keyring save failed: {e} (session is in-memory only)");
                (format!("unavailable: {e}"), Some(warning))
            }
        };
        session.access_token = Some(tokens.access_token);
        session.access_expires_at =
            Some(self.choke.wall() + Duration::from_secs(tokens.expires_in));
        session.refresh_token = Some(tokens.refresh_token);
        session.username = Some(tokens.username);
        session.keyring = keyring;
        session.advance_access_token();
        session.advance_refresh_token();
        session.refresh_flight = None;
        warning
    }

    /// Current access token, refreshing through the provider if it is
    /// expired (or about to be). Jobs call this; clients never see tokens.
    /// `force_refresh` skips the cached token so the provider round-trip is
    /// guaranteed — that's what makes `auth check` an actual proof.
    /// `account` is the job's account (fixed at submit): the token handed
    /// back is that account's session's, looked up at the moment the token
    /// is taken — a logout of A after A's job was submitted fails A's job
    /// here, before any send ("no session for A"). `None` is the sole
    /// session (auth verbs), refused when several are live.
    async fn valid_access_token(
        &self,
        account: Option<&str>,
        force_refresh: bool,
    ) -> Result<(String, String), String> {
        enum Decision {
            Owner {
                id: u64,
                generations: AuthGenerations,
                refresh_token: String,
                result: watch::Sender<Option<AccessTokenResult>>,
            },
            Waiter(watch::Receiver<Option<AccessTokenResult>>),
        }

        let (username, decision) = {
            let mut s = self.shared.lock().unwrap();
            let wall = self.choke.wall();
            let session = s.auth.get_mut(account)?;
            let username = session.username.clone().unwrap_or_default();
            if !force_refresh
                && let (Some(token), Some(expires)) =
                    (&session.access_token, session.access_expires_at)
                && expires
                    .duration_since(wall)
                    .is_ok_and(|left| left > Duration::from_secs(5))
            {
                return Ok((token.clone(), username));
            }
            let refresh_token = match &session.refresh_token {
                Some(rt) => rt.clone(),
                None => return Err(format!("no refresh token for {username} — run `acq auth`")),
            };
            // A grant the provider already rejected is terminal (CONTEXT.md):
            // it is not sent again until login or logout replaces it.
            if let Some(cause) = self.rails().refresh_failed(&username) {
                return Err(format!(
                    "token refresh disabled for {username}: {cause}; run `acq auth`"
                ));
            }
            let generations = session.generations;
            let decision = if let Some(flight) = &session.refresh_flight
                && flight.generations == generations
            {
                Decision::Waiter(flight.result.subscribe())
            } else {
                let id = session.next_refresh_flight;
                session.next_refresh_flight = session.next_refresh_flight.wrapping_add(1);
                let (result, _) = watch::channel(None);
                session.refresh_flight = Some(RefreshFlight {
                    id,
                    generations,
                    result: result.clone(),
                });
                Decision::Owner {
                    id,
                    generations,
                    refresh_token,
                    result,
                }
            };
            (username, decision)
        };
        match decision {
            Decision::Waiter(result) => wait_for_refresh(result).await,
            Decision::Owner {
                id,
                generations,
                refresh_token,
                result,
            } => {
                let owner = RefreshOwnerGuard {
                    daemon: self,
                    account: username.clone(),
                    id,
                    generations,
                    result: Some(result),
                };
                // May wait on the token endpoint's limiter; the shared lock is
                // not held here, so every concurrent caller can join this owner.
                let refresh = auth::refresh(&self.choke, &self.provider, &refresh_token).await;
                self.announce_trip();
                // L0 rail 2: mark a rejected grant before the flight is
                // closed, so no caller can pass the fast path and re-send the
                // dead token in between; and only while this flight is still
                // the current one, so a stale flight cannot disable a session
                // that re-authenticated meanwhile. The persisted cause is the
                // status plus a fixed reason, never the response body.
                if let Some(error) = refresh.as_ref().err().filter(|e| e.is_rejected_grant()) {
                    let current = {
                        let s = self.shared.lock().unwrap();
                        s.auth.find(&username).is_some_and(|session| {
                            session.generations == generations
                                && session.refresh_flight.as_ref().is_some_and(|f| f.id == id)
                        })
                    };
                    let cause = format!(
                        "refresh token rejected with HTTP {}",
                        error.status.unwrap_or_default()
                    );
                    if current && self.rails().mark_refresh_failed(&username, &cause) {
                        self.note_error(&format!(
                            "AUTH: {username}: {cause}; further refreshes disabled until `acq auth` ({error})"
                        ));
                    }
                }
                let refresh = refresh.map_err(|e| format!("token refresh failed: {e}"));
                owner.finish(refresh)
            }
        }
    }

    fn finish_refresh(
        &self,
        account: &str,
        id: u64,
        generations: AuthGenerations,
        refresh: Result<auth::TokenResponse, String>,
        sender: &watch::Sender<Option<AccessTokenResult>>,
    ) -> AccessTokenResult {
        let mut warning = None;
        let mut renamed = None;
        let mut index_update = None;
        let outcome = {
            let mut s = self.shared.lock().unwrap();
            let Some(session) = s.auth.find_mut(account) else {
                return finish_stale(sender);
            };
            let owns_current_flight = session
                .refresh_flight
                .as_ref()
                .is_some_and(|flight| flight.id == id && flight.generations == generations);
            if !owns_current_flight || session.generations != generations {
                Err(SESSION_CHANGED_DURING_REFRESH.into())
            } else {
                session.refresh_flight = None;
                match refresh {
                    Ok(tokens) => {
                        // GGG returns the same account on a refresh; if the
                        // name ever differs (a rename), the session follows
                        // it — lookups go by the session's username.
                        if tokens.username != account {
                            renamed = Some(tokens.username.clone());
                        }
                        let result = (tokens.access_token.clone(), tokens.username.clone());
                        let username = tokens.username.clone();
                        warning = self.install_tokens_locked(session, tokens);
                        index_update = Some((username, session.uuid.clone(), warning.is_none()));
                        Ok(result)
                    }
                    Err(error) => Err(error),
                }
            }
        };
        // With the session's uuid, a refresh that renamed the account
        // updates the existing index mapping in place (`record_login`
        // follows the uuid) instead of minting a uuid-less twin entry; a
        // pre-uuid session falls back to plain bookkeeping.
        if let Some((username, uuid, persisted)) = index_update {
            self.with_index(|index| match &uuid {
                Some(uuid) => index.record_login(&username, uuid, persisted, unix_now()),
                None => index.upsert(&username, persisted, unix_now()),
            });
        }
        if let Some(new_name) = &renamed {
            self.shared.lock().unwrap().auth.rename(account, new_name);
        }
        sender.send_replace(Some(outcome.clone()));
        if outcome.is_ok() {
            self.log(&format!("access token refreshed for {account}"));
        }
        if let Some(new_name) = renamed {
            self.note_error(&format!(
                "AUTH: refresh for {account} returned username {new_name}; session follows the new name"
            ));
        }
        if let Some(warning) = warning {
            self.note_error(&warning);
        }
        outcome
    }

    /// Drop a session (the named account's, or the sole one), its keyring
    /// entry, and its dead-grant mark. Other sessions are untouched.
    fn logout(&self, account: Option<&str>) -> Result<(), String> {
        let (username, clear) = {
            let mut s = self.shared.lock().unwrap();
            let username = s.auth.get(account)?.username.clone().unwrap_or_default();
            s.auth
                .by_account
                .retain(|_, x| x.username.as_deref() != Some(username.as_str()));
            if s.auth.last_login.as_deref() == Some(username.as_str()) {
                s.auth.last_login = None;
            }
            let clear = self
                .credential_store
                .clear(self.provider.keyring_service, &username);
            (username, clear)
        };
        self.rails().clear_refresh_failed(&username);
        self.with_index(|index| index.set_persisted(&username, false));
        self.log(&format!("logged out {username}"));
        clear
    }

    /// Drop a *non-live* account's keyring entry and mark it not persisted.
    /// Nothing about the live session changes.
    fn forget_account(&self, selector: &str) -> Result<(), String> {
        let Some(dir) = &self.store_dir else {
            return Err("no account index in this daemon".into());
        };
        let index = Index::load(dir).map_err(|e| format!("accounts index: {e:#}"))?;
        let entry = index
            .resolve(Some(selector))
            .map_err(|e| e.to_string())?
            .clone();
        // Nothing to clear for a session that was never in the keyring.
        let cleared = if entry.persisted {
            self.credential_store
                .clear(self.provider.keyring_service, &entry.username)
        } else {
            Ok(())
        };
        self.with_index(|index| index.set_persisted(&entry.username, false));
        self.log(&format!(
            "forgot account {} (keyring entry cleared)",
            entry.username
        ));
        cleared
    }

    fn session_statuses(&self, s: &Shared) -> Vec<SessionStatus> {
        let wall = self.choke.wall();
        let mut v: Vec<SessionStatus> = s
            .auth
            .by_account
            .values()
            .map(|session| SessionStatus {
                username: session.username.clone().unwrap_or_default(),
                access_expires_in_seconds: session
                    .access_expires_at
                    .map(|t| t.duration_since(wall).unwrap_or_default().as_secs()),
                keyring: session.keyring.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.username.cmp(&b.username));
        v
    }

    /// The account reported as "the" username: the most recent login while
    /// it is live, else the sole session. Informational only.
    fn headline_session<'a>(&self, s: &'a Shared) -> Option<&'a AuthSession> {
        s.auth
            .last_login
            .as_ref()
            .and_then(|u| s.auth.find(u))
            .or_else(|| s.auth.get(None).ok())
    }

    fn keyring_summary(&self, s: &Shared) -> String {
        self.headline_session(s)
            .map(|session| session.keyring.clone())
            .unwrap_or_else(|| s.auth.keyring.clone())
    }

    fn auth_status(&self) -> Response {
        let s = self.shared.lock().unwrap();
        let head = self.headline_session(&s);
        Response::Auth {
            logged_in: !s.auth.by_account.is_empty(),
            pending: s.auth.pending.is_some(),
            login_ok: s
                .auth
                .flow_result
                .as_ref()
                .and_then(|r| r.as_ref().ok().cloned()),
            login_error: s
                .auth
                .flow_result
                .as_ref()
                .and_then(|r| r.as_ref().err().cloned()),
            username: head.and_then(|h| h.username.clone()),
            access_expires_in_seconds: head.and_then(|h| h.access_expires_at).map(|t| {
                t.duration_since(self.choke.wall())
                    .unwrap_or_default()
                    .as_secs()
            }),
            keyring: self.keyring_summary(&s),
            provider: self.provider.name.to_string(),
            accounts: self.session_statuses(&s),
        }
    }

    // ---- connection handling -------------------------------------------

    async fn handle_conn(self: Arc<Self>, stream: UnixStream) {
        {
            let mut s = self.shared.lock().unwrap();
            s.connections += 1;
            s.last_activity = Instant::now();
        }

        let (read, mut write) = stream.into_split();
        let mut lines = BufReader::new(read).lines();
        let mut events: Option<broadcast::Receiver<JobInfo>> = None;

        loop {
            tokio::select! {
                line = lines.next_line() => {
                    let Ok(Some(line)) = line else { break };
                    if line.trim().is_empty() {
                        continue;
                    }
                    let response = match serde_json::from_str::<Request>(&line) {
                        Ok(req) => self.handle_request(req, &mut events).await,
                        Err(e) => Response::Error { message: format!("bad request: {e}") },
                    };
                    let stopping = matches!(response, Response::Stopping);
                    if write_line(&mut write, &response).await.is_err() {
                        break;
                    }
                    if stopping {
                        self.log("stop requested; exiting");
                        let _ = std::fs::remove_file(socket_path());
                        std::process::exit(0);
                    }
                }
                event = recv_event(&mut events) => {
                    if write_line(&mut write, &Response::Event { job: event }).await.is_err() {
                        break;
                    }
                }
            }
        }

        let mut s = self.shared.lock().unwrap();
        s.connections -= 1;
        s.last_activity = Instant::now();
    }

    /// The `quote` request (CONTEXT.md, decided 2026-08-31): a read-only,
    /// non-reserving projection of what the quoted work would meet at the
    /// choke point right now. Reads the limiter and the queue, sends
    /// nothing, reserves nothing, remembers nothing — two quotes about the
    /// same work may disagree because the world moved. Account selection
    /// follows `Submit`'s rules exactly (resolved here, refused when
    /// ambiguous), so the quote keys the same limiter state a submit would.
    fn quote(&self, jobs: &[QuoteJob], account: Option<&str>) -> Result<Quote, String> {
        // The selector is judged before anything is projected — an unknown
        // or ambiguous account refuses the quote whole even when the job
        // list is empty, exactly as a submit would refuse it. Omitted with
        // several sessions live is ambiguous too: a quote is one projection
        // under one headline account, and the daemon never guesses whose.
        // The judged selector is what each job resolves against, so a
        // quoted job keys exactly what the same submit would key (an
        // omitted selector still runs accountless kinds accountless).
        let account = match account {
            Some(_) => self.canonical_account(account)?,
            None => {
                let s = self.shared.lock().unwrap();
                if !s.auth.by_account.is_empty() {
                    s.auth.get(None)?;
                }
                None
            }
        };
        // Group the work by scheduling scope: the policy state key once the
        // route is learned (same-name policies share counters, N6; `Account`
        // rules count per account, rung 11), else the endpoint key itself.
        let mut scopes: BTreeMap<String, (BTreeSet<String>, u64)> = BTreeMap::new();
        let mut sends_nothing: BTreeMap<String, u64> = BTreeMap::new();
        let mut resolved_account: Option<String> = account.clone();
        let mut needs_token = false;
        for job in jobs {
            let acct = self.resolve_account(&job.kind, account.as_deref())?;
            if let Some(a) = &acct {
                resolved_account.get_or_insert_with(|| a.clone());
            }
            match self.keyed_route_for(&job.kind, &job.params, acct.as_deref()) {
                None => *sends_nothing.entry(job.kind.clone()).or_insert(0) += 1,
                Some((endpoint, _)) => {
                    needs_token |= self.needs_auth(split_endpoint_key(&endpoint).0);
                    let slot = scopes.entry(self.choke.serial_key(&endpoint)).or_default();
                    slot.0.insert(endpoint);
                    slot.1 += 1;
                }
            }
        }
        // Jobs already here compete for the same scopes; the estimate puts
        // them ahead of the quoted work. Probes stay out (their own key, and
        // named under `not_covered` instead), and so does a parent holding a
        // deferred result: its own request already happened — it is only
        // waiting for its children, who count for themselves.
        let mut queued: HashMap<String, u64> = HashMap::new();
        {
            let s = self.shared.lock().unwrap();
            for e in s.jobs.values() {
                if e.info.state.is_terminal() || e.info.kind == "probe" || e.deferred.is_some() {
                    continue;
                }
                if let Some((endpoint, _)) =
                    self.keyed_route_for(&e.info.kind, &e.params, e.info.account.as_deref())
                {
                    *queued.entry(self.choke.serial_key(&endpoint)).or_insert(0) += 1;
                }
            }
        }
        let mut unprobed: BTreeSet<String> = BTreeSet::new();
        let mut out = Vec::new();
        for (key, (endpoints, requests)) in scopes {
            // Endpoints share a scope only through a learned common policy,
            // so any one of them names the scope's state. The projection is
            // taken under one limiter lock, so its rules, observation age,
            // and ETA describe the same instant.
            let sample = endpoints.iter().next().cloned().unwrap_or_default();
            let queued_ahead = queued.get(&key).copied().unwrap_or(0);
            let ahead = (queued_ahead + requests - 1) as u32;
            let projection = self.choke.project(&sample, ahead);
            let (policy, eta_seconds, notes) = match projection.state {
                EndpointState::Policy(name) => {
                    let eta = projection.eta.unwrap_or_default().as_secs();
                    (Some(name), Some(eta), Vec::new())
                }
                EndpointState::Policyless => (
                    None,
                    Some(0),
                    vec![
                        "declared policyless: paced by the send gate alone; headers, if they \
                         ever appear, are learned strictly"
                            .into(),
                    ],
                ),
                EndpointState::Unknown => {
                    let route = split_endpoint_key(&sample).0.to_string();
                    let note = if Self::route_probes(&sample) {
                        unprobed.insert(route.clone());
                        format!(
                            "policy not yet learned: a HEAD probe (its own send) precedes the \
                             first request on {route}"
                        )
                    } else {
                        format!(
                            "policy not yet learned: {route} is a declared no-probe route — \
                             its first GET teaches the limiter"
                        )
                    };
                    (None, None, vec![note])
                }
                EndpointState::Degraded { until, reason } => {
                    // The cooldown expires into `Unknown`, so the eventual
                    // replacement probe is a future send too.
                    let route = split_endpoint_key(&sample).0.to_string();
                    if Self::route_probes(&sample) {
                        unprobed.insert(route);
                    }
                    (
                        None,
                        None,
                        vec![format!(
                            "endpoint closed by a failed probe for another {}s: {reason}",
                            until.saturating_duration_since(self.choke.now()).as_secs()
                        )],
                    )
                }
            };
            out.push(QuoteScope {
                key,
                endpoints: endpoints.into_iter().collect(),
                requests,
                queued_ahead,
                policy,
                rules: projection.rules,
                observed_seconds_ago: projection.observed_seconds_ago,
                eta_seconds,
                notes,
            });
        }
        let total: u64 = out.iter().map(|s| s.requests).sum();
        let mut not_covered = Vec::new();
        if !unprobed.is_empty() {
            not_covered.push(format!(
                "a HEAD probe before the first request on {} (N16) — its own send, outside \
                 every estimate",
                unprobed.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
        if needs_token {
            not_covered.push(
                "an OAuth token refresh if the access token expires first (N33) — paced by \
                 the token policy, not quoted"
                    .into(),
            );
        }
        if total > 0 {
            not_covered.push(format!(
                "429 re-sends (up to {MAX_429_RETRIES} per request) — possible, never predicted"
            ));
        }
        for (kind, n) in sends_nothing {
            not_covered.push(format!(
                "{n} `{kind}` job(s) send nothing and are outside every scope"
            ));
        }
        Ok(Quote {
            observed_at: self
                .choke
                .wall()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            provider: self.provider.name.to_string(),
            account: resolved_account,
            halted: self.rails().halted(),
            // Echoed verbatim: the quote's verifiable basis — a carrier
            // checks these tuples name exactly the work it cares about.
            work: jobs.to_vec(),
            scopes: out,
            not_covered,
        })
    }

    async fn handle_request(
        self: &Arc<Self>,
        req: Request,
        events: &mut Option<broadcast::Receiver<JobInfo>>,
    ) -> Response {
        match req {
            Request::Hello { client_version } => {
                // The build stamp, not the package version: the client
                // decides staleness from this and replaces (or refuses) a
                // daemon from another commit.
                if client_version != crate::VERSION_WITH_BUILD {
                    self.log(&format!(
                        "version mismatch: client {client_version}, daemon {}",
                        crate::VERSION_WITH_BUILD
                    ));
                }
                Response::Hello {
                    daemon_version: crate::VERSION_WITH_BUILD.to_string(),
                    pid: std::process::id(),
                    provider: self.provider.name.to_string(),
                }
            }
            Request::Submit {
                kind,
                params,
                priority,
                submitted_by,
                account,
            } => match self
                .resolve_account(&kind, account.as_deref())
                .and_then(|account| self.submit(kind, params, priority, submitted_by, account))
            {
                Ok(id) => Response::Submitted { id },
                Err(message) => {
                    self.note_error(&format!("submit refused: {message}"));
                    Response::Error { message }
                }
            },
            Request::Status { id } => match self.shared.lock().unwrap().snapshot(self, id) {
                Some(job) => Response::Status { job },
                None => Response::Error {
                    message: format!("no job {id}"),
                },
            },
            Request::Result { id } => {
                let held = {
                    let s = self.shared.lock().unwrap();
                    s.jobs.get(&id).map(|e| (e.info.state, e.outcome.clone()))
                };
                match held {
                    Some((_, Some(outcome))) => Response::Result { id, outcome },
                    Some((state, None)) => Response::Error {
                        message: format!("job {id} is still {state}"),
                    },
                    // Not this lifetime's: a previous daemon's result, if
                    // retention still has it.
                    None => match self.stored_outcome(id) {
                        Ok(Some(outcome)) => Response::Result { id, outcome },
                        Ok(None) => Response::Error {
                            message: format!("no job {id}"),
                        },
                        Err(message) => {
                            self.note_error(&format!("result {id}: {message}"));
                            Response::Error { message }
                        }
                    },
                }
            }
            Request::Cancel { id } => match self.cancel(id) {
                Ok(()) => Response::Ack,
                Err(message) => Response::Error { message },
            },
            Request::SetPriority { id, priority } => match self.set_priority(id, priority) {
                Ok(()) => {
                    self.work.notify_one();
                    Response::Ack
                }
                Err(message) => Response::Error { message },
            },
            Request::List => Response::Jobs {
                jobs: self.shared.lock().unwrap().list(self),
            },
            Request::Subscribe => {
                *events = Some(self.events.subscribe());
                Response::Subscribed
            }
            Request::AuthStart => match self.auth_start().await {
                Ok(authorize_url) => Response::AuthUrl { authorize_url },
                Err(message) => Response::Error { message },
            },
            Request::AuthStatus => self.auth_status(),
            Request::AuthCheck { account } => match self.canonical_account(account.as_deref()) {
                Err(message) => Response::Error { message },
                Ok(account) => match self.valid_access_token(account.as_deref(), true).await {
                    Ok(_) => self.auth_status(),
                    Err(message) => {
                        self.note_error(&format!("auth check failed: {message}"));
                        Response::Error { message }
                    }
                },
            },
            Request::AuthLogout { account } => {
                // A live session (named, or the sole one) logs out; a known
                // but not live account only loses its keyring entry.
                let live = {
                    let s = self.shared.lock().unwrap();
                    match &account {
                        None => s
                            .auth
                            .get(None)
                            .map(|x| x.username.clone().unwrap_or_default()),
                        Some(req) => s
                            .auth
                            .matching(req)
                            .map(|x| x.username.clone().unwrap_or_default())
                            .ok_or_else(|| "not live".to_string()),
                    }
                };
                match (live, &account) {
                    (Ok(username), _) => {
                        if let Err(e) = self.logout(Some(&username)) {
                            self.log(&format!("keyring clear failed: {e}"));
                        }
                        Response::Ack
                    }
                    (Err(_), Some(req)) => match self.forget_account(req) {
                        Ok(()) => Response::Ack,
                        Err(message) => Response::Error { message },
                    },
                    (Err(message), None) => Response::Error { message },
                }
            }
            Request::Quote { jobs, account } => match self.quote(&jobs, account.as_deref()) {
                Ok(quote) => Response::Quote { quote },
                Err(message) => Response::Error { message },
            },
            Request::DaemonStatus => {
                let s = self.shared.lock().unwrap();
                let (in_flight, max_in_flight) = self.choke.actual_send_occupancy();
                let (waiting, running) =
                    s.jobs
                        .values()
                        .fold((0, 0), |(w, r), e| match e.info.state {
                            JobState::Waiting => (w + 1, r),
                            JobState::Running => (w, r + 1),
                            _ => (w, r),
                        });
                Response::DaemonStatus {
                    pid: std::process::id(),
                    version: crate::VERSION_WITH_BUILD.to_string(),
                    provider: self.provider.name.to_string(),
                    uptime_seconds: self.started.elapsed().as_secs(),
                    connections: s.connections,
                    jobs_waiting: waiting,
                    jobs_running: running,
                    policies_known: self.choke.policy_statuses().len(),
                    in_flight,
                    max_in_flight,
                    rails: self.rails().status(),
                    keyring: self.keyring_summary(&s),
                }
            }
            Request::DaemonStop => Response::Stopping,
            Request::ResetTripwire => {
                self.reset_rails();
                Response::Ack
            }
            Request::Dashboard => {
                let s = self.shared.lock().unwrap();
                let (in_flight, max_in_flight) = self.choke.actual_send_occupancy();
                Response::Dashboard {
                    pid: std::process::id(),
                    version: crate::VERSION_WITH_BUILD.to_string(),
                    provider: self.provider.name.to_string(),
                    uptime_seconds: self.started.elapsed().as_secs(),
                    connections: s.connections,
                    logged_in: !s.auth.by_account.is_empty(),
                    username: self.headline_session(&s).and_then(|h| h.username.clone()),
                    access_expires_in_seconds: self
                        .headline_session(&s)
                        .and_then(|h| h.access_expires_at)
                        .map(|t| {
                            t.duration_since(self.choke.wall())
                                .unwrap_or_default()
                                .as_secs()
                        }),
                    keyring: self.keyring_summary(&s),
                    in_flight,
                    max_in_flight,
                    policies: self.choke.policy_statuses(),
                    policyless_endpoints: self.choke.policyless_endpoints(),
                    degraded_endpoints: self.choke.degraded_endpoints(),
                    jobs: s.list(self),
                    sends: self.choke.recent_sends(),
                    rails: self.rails().status(),
                    errors: s
                        .errors
                        .iter()
                        .rev()
                        .map(|(at, message)| ErrorRecord {
                            seconds_ago: at.elapsed().as_secs_f64(),
                            message: message.clone(),
                        })
                        .collect(),
                }
            }
        }
    }

    /// Clear the rails halt and wake the jobs that were waiting on it.
    fn reset_rails(&self) {
        self.rails().reset_tripwire();
        self.log("live-test rails reset by request");
        self.work.notify_one();
    }

    async fn idle_watchdog(self: Arc<Self>) {
        let idle_shutdown = idle_shutdown_from_env();
        loop {
            tokio::time::sleep(IDLE_POLL).await;
            let idle = {
                let s = self.shared.lock().unwrap();
                let parked = self.rails().halted().is_some() || self.queue_failed().is_some();
                let live_jobs = Self::has_live_jobs(&s, parked);
                s.connections == 0 && !live_jobs && s.last_activity.elapsed() >= idle_shutdown
            };
            // Limiter history inside a policy window is worth more than a
            // clean exit: a daemon respawned a minute later would otherwise
            // have to assume the worst about every hit it can't see.
            let idle = idle && !self.choke.is_live();
            if idle {
                self.log("idle timeout; exiting");
                let _ = std::fs::remove_file(socket_path());
                std::process::exit(0);
            }
        }
    }
}

/// The outcome of a fan-out whose child submission was refused: the
/// parent was cancelled, or the queue failed. Already-submitted children
/// run either way (their sends are theirs); the parent never claims
/// success over a partial set.
/// Realm admission: a kind in a realm family (`crate::realm`) must name a
/// realm that family takes — or none, meaning pc. Runs at submit for
/// every kind and per tuple inside `validate_apply`, so a job that would
/// render a stash URL under `poe2` never gets an id (CONTEXT.md,
/// 2026-09-02: no code path renders an unobserved URL shape). Kinds
/// outside the families ignore the param.
fn admit_realm(kind: &str, params: &Value) -> Result<(), String> {
    let family = match kind {
        "characters" | "character" => Family::Characters,
        "stashes" | "stash" | "refresh" => Family::Stashes,
        _ => return Ok(()),
    };
    family.realm_of(params).map(|_| ())
}

/// The `apply` admission check (CONTEXT.md, decided 2026-09-01): the
/// vocabulary a plan-blind daemon can enforce. `params.jobs` must be a
/// non-empty array of `(kind, params)` tuples in which every kind is a
/// single-request one that submits no children — `stashes`, `stash` with
/// `deep` absent/false, `characters`, or `character` with a name
/// (characters joined 2026-09-02) — so no child can expand the reviewed set;
/// and when the caller declares `max_requests`, the tuple count must not
/// exceed it. Runs at submit, before a job id exists, so a refusal
/// admits nothing (mid-fan-out terminalization is never the budget's
/// mechanism); a misread budget refuses too — a limit half-honored by
/// ignoring it would spend exactly what the caller tried to cap.
fn validate_apply(params: &Value) -> Result<(), String> {
    let Some(jobs) = params.get("jobs").and_then(Value::as_array) else {
        return Err(
            "apply needs a `jobs` array of {kind, params} tuples (a plan's actions)".into(),
        );
    };
    if jobs.is_empty() {
        return Err(
            "apply with an empty `jobs` array: a plan with no actions has nothing to execute"
                .into(),
        );
    }
    for (i, job) in jobs.iter().enumerate() {
        let kind = job.get("kind").and_then(Value::as_str).unwrap_or("");
        let params = job.get("params").cloned().unwrap_or(Value::Null);
        admit_realm(kind, &params).map_err(|e| format!("apply job {i}: {e}"))?;
        match kind {
            "stashes" => {}
            "stash" => {
                if params.get("deep").and_then(Value::as_bool).unwrap_or(false) {
                    return Err(format!(
                        "apply job {i}: a plan's stash fetch never fans out (`deep` must be false)"
                    ));
                }
                if params.get("id").and_then(Value::as_str).is_none() {
                    return Err(format!("apply job {i}: stash needs an id"));
                }
            }
            "characters" => {}
            "character" => {
                if params.get("name").and_then(Value::as_str).is_none() {
                    return Err(format!("apply job {i}: character needs a name"));
                }
            }
            other => {
                return Err(format!(
                    "apply job {i}: kind {other:?} is not in the plan vocabulary \
                     (stashes, stash, characters, character)"
                ));
            }
        }
    }
    if let Some(max) = params.get("max_requests") {
        let Some(max) = max.as_u64() else {
            return Err(format!(
                "max_requests must be a non-negative integer, not {max}"
            ));
        };
        if jobs.len() as u64 > max {
            return Err(format!(
                "plan exceeds the budget: {} logical request(s) against max_requests {max} — nothing was submitted",
                jobs.len()
            ));
        }
    }
    Ok(())
}

fn fan_out_stopped(cancelled: bool, submitted: usize, why: &str) -> Outcome {
    if cancelled {
        Outcome::Cancelled
    } else {
        Outcome::Failure {
            error: format!("fan-out stopped after {submitted} children: {why}"),
        }
    }
}

impl Daemon {
    /// Whether any job keeps the daemon up. A halted daemon's waiting jobs
    /// do not: they are on disk, and its successor holds them until the
    /// reset (`CONTEXT.md`, "A rails halt leaves queued network jobs
    /// waiting"). A parent holding its result runs no task of its own.
    fn has_live_jobs(s: &Shared, halted: bool) -> bool {
        s.jobs.values().any(|e| match e.info.state {
            JobState::Running => e.deferred.is_none() || !halted,
            JobState::Waiting => !halted,
            _ => false,
        })
    }
}

fn classify_api_body(
    status: reqwest::StatusCode,
    retry_after: &RetryAfter,
    path: &str,
    rate: Value,
    body: Result<String, String>,
) -> Result<(Value, Value), ApiError> {
    // Status owns non-2xx outcomes even if reading the error body also failed
    // (D8). The body error remains in the evidence rather than becoming fake
    // empty content.
    if !status.is_success() {
        let body = match body {
            Ok(body) => body.chars().take(300).collect(),
            Err(error) => format!("<body read transport failure: {error}>"),
        };
        let evidence = format!("GET {path} returned {status}; rate headers {rate}; body: {body}");
        return Err(match status.as_u16() {
            429 if retry_after.is_acceptable() => ApiError::RateLimited(evidence),
            429 => ApiError::Other(format!(
                "terminal rate limit: {evidence}; {retry_after}; NOT retrying"
            )),
            403 | 503 => ApiError::Other(format!(
                "{evidence} — {}; NOT retrying (invariant 3)",
                BlockShape::of(&body).describe()
            )),
            _ => ApiError::Other(evidence),
        });
    }

    let body = body.map_err(|error| {
        ApiError::Other(format!(
            "GET {path} body read transport failure after {status}: {error}"
        ))
    })?;
    serde_json::from_str::<Value>(&body)
        .map(|value| (value, rate))
        .map_err(|error| ApiError::Other(format!("bad JSON from {path}: {error}")))
}

/// Wait for the browser to hit the loopback redirect with an auth code.
/// Ignores stray requests (favicons etc.) until `/callback` arrives.
async fn wait_callback(listener: TcpListener, expected_state: &str) -> Result<String, String> {
    loop {
        let (mut stream, _) = listener.accept().await.map_err(|e| e.to_string())?;
        let Some(req) = mockggg::read_request(&mut stream).await else {
            continue;
        };
        if req.method != "GET" || req.path != CALLBACK_PATH {
            mockggg::respond(&mut stream, "404 Not Found", "text/plain", "not found").await;
            continue;
        }
        match (req.query.get("code"), req.query.get("state")) {
            (Some(code), Some(state)) if state == expected_state => {
                mockggg::respond(
                    &mut stream,
                    "200 OK",
                    "text/html",
                    "<h1>Logged in</h1><p>You can close this tab and return to the terminal.</p>",
                )
                .await;
                return Ok(code.clone());
            }
            _ => {
                mockggg::respond(
                    &mut stream,
                    "400 Bad Request",
                    "text/plain",
                    "state mismatch",
                )
                .await;
                return Err("callback state mismatch".into());
            }
        }
    }
}

async fn recv_event(rx: &mut Option<broadcast::Receiver<JobInfo>>) -> JobInfo {
    match rx {
        Some(rx) => loop {
            match rx.recv().await {
                Ok(info) => return info,
                // Lagged: skip missed events, keep going. Closed can't happen
                // while the daemon holds the sender.
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => std::future::pending().await,
            }
        },
        None => std::future::pending().await,
    }
}

/// A refresh whose session vanished (logged out) while it was in flight.
fn finish_stale(sender: &watch::Sender<Option<AccessTokenResult>>) -> AccessTokenResult {
    let outcome: AccessTokenResult = Err(SESSION_CHANGED_DURING_REFRESH.into());
    sender.send_replace(Some(outcome.clone()));
    outcome
}

async fn wait_for_refresh(
    mut result: watch::Receiver<Option<AccessTokenResult>>,
) -> AccessTokenResult {
    loop {
        if let Some(outcome) = result.borrow().clone() {
            return outcome;
        }
        if result.changed().await.is_err() {
            return Err(REFRESH_OWNER_ABANDONED.into());
        }
    }
}

async fn write_line(
    write: &mut tokio::net::unix::OwnedWriteHalf,
    response: &Response,
) -> std::io::Result<()> {
    let mut line = serde_json::to_string(response).expect("response serializes");
    line.push('\n');
    write.write_all(line.as_bytes()).await
}

/// Run the daemon until stopped or idle-timed-out. Never returns Ok while the
/// socket is healthy; returns Err early if another daemon already owns it.
pub async fn run() -> Result<()> {
    // The log opens before anything that can refuse startup: a lazy-spawned
    // daemon's stderr goes to null, so the log is the only place a refusal
    // (broken daemon.db, failed bind) can reach the user — the CLI reads it
    // when the spawn fails.
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())?;
    let result = run_with_log(log.try_clone()?).await;
    if let Err(e) = &result {
        writeln!(&log, "STARTUP: {e:#}").ok();
    }
    result
}

async fn run_with_log(log: std::fs::File) -> Result<()> {
    let path = socket_path();
    if path.exists() {
        // Live daemon or stale socket from a crash?
        if UnixStream::connect(&path).await.is_ok() {
            anyhow::bail!("daemon already running on {}", path.display());
        }
        std::fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)
        .map_err(|e| anyhow::anyhow!("could not bind {}: {e}", path.display()))?;

    // Real GGG only on explicit opt-in; the default remains the in-process
    // mock, and in real mode the mock is never even started.
    let provider = if ggg_mode() {
        Provider::ggg()
    } else {
        Provider::mock(&mockggg::start().await?)
    };
    // Same limiter in both modes: empty until responses teach it policies.
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let rails = Arc::new(Rails::with_config_and_clock(
        RailsConfig::from_env(provider.name, &path, &journal_path(provider.name)),
        clock.clone(),
    ));
    let choke = ChokePoint::with_clock_and_rails(clock, rails);
    Daemon::declare_route_knowledge(&choke);

    // Sessions survive daemon restarts through the keyring, one entry per
    // account; the account index says which entries to look for. One live
    // session for now (the most recent login); the others stay in the
    // keyring untouched. One unreadable entry never blocks the rest.
    let dir = store_dir(provider.name);
    let mut sessions = Sessions {
        keyring: "ok".into(),
        ..Sessions::default()
    };
    match Index::load(&dir) {
        Ok(index) => {
            for entry in index.persisted() {
                match auth::keyring_load(provider.keyring_service, &entry.username) {
                    Ok(Some(refresh_token)) => {
                        sessions.replace(AuthSession {
                            refresh_token: Some(refresh_token),
                            username: Some(entry.username.clone()),
                            uuid: entry.uuid.clone(),
                            keyring: "ok".into(),
                            ..AuthSession::default()
                        });
                    }
                    Ok(None) => {
                        writeln!(
                            &log,
                            "ACCOUNTS: index lists {} as persisted but the keyring has no entry",
                            entry.username
                        )
                        .ok();
                    }
                    Err(e) => {
                        sessions.keyring = format!("unavailable: {e}");
                        writeln!(
                            &log,
                            "ACCOUNTS: keyring read for {} failed: {e}",
                            entry.username
                        )
                        .ok();
                    }
                }
            }
            // `persisted()` is newest first; `replace` marked the last one
            // inserted as the most recent login, which is the oldest.
            sessions.last_login = index.persisted().first().map(|e| e.username.clone());
        }
        Err(e) => {
            writeln!(&log, "ACCOUNTS: could not read {}: {e:#}", dir.display()).ok();
        }
    }

    // A daemon that cannot read its queue must not run: it would restart
    // ids at 1 and reuse them against the table's history the moment the
    // file comes back. Refusing to start is the safe failure; the log
    // names the file to repair or remove.
    let jobs_db = match JobDb::open(&acquisition_store::jobs::daemon_db_path(&dir)) {
        Ok(db) => Mutex::new(db),
        Err(e) => {
            anyhow::bail!(
                "could not open {}: {e:#} (repair or remove it; the daemon will not run without its queue)",
                acquisition_store::jobs::daemon_db_path(&dir).display()
            );
        }
    };
    let daemon = Arc::new(Daemon {
        shared: Mutex::new(Shared {
            jobs: HashMap::new(),
            next_id: 1,
            auth: sessions,
            connections: 0,
            last_activity: Instant::now(),
            errors: VecDeque::new(),
            active_jobs: HashMap::new(),
        }),
        started: Instant::now(),
        events: broadcast::channel(256).0,
        work: Notify::new(),
        log: Mutex::new(log),
        choke,
        provider,
        credential_store: Arc::new(OsCredentialStore),
        store_dir: Some(dir.clone()),
        store: Mutex::new(None),
        jobs_db,
        queue_failure: Mutex::new(None),
    });

    daemon.log(&format!(
        "daemon {} build {} listening on {} (pid {})",
        VERSION,
        crate::BUILD,
        path.display(),
        std::process::id()
    ));
    let (keyring, username) = {
        let s = daemon.shared.lock().unwrap();
        let names = s.auth.usernames();
        (
            daemon.keyring_summary(&s),
            (!names.is_empty()).then(|| names.join(", ")),
        )
    };
    let provider_desc = if daemon.provider.is_real() {
        "ggg (REAL GGG — requests leave this machine)".to_string()
    } else {
        format!("mock at {}", daemon.provider.api_base)
    };
    daemon.log(&format!(
        "provider: {provider_desc} | keyring: {keyring} | session: {}",
        username.as_deref().unwrap_or("none"),
    ));
    let rails = daemon.rails().status();
    daemon.log(&format!(
        "rails: tripwire {} | halted: {} | refresh-failed: {} | ceiling: {} | journal: {} | idle exit after {}s",
        if rails.tripwire_enabled { "ON" } else { "off" },
        rails.halted.as_deref().unwrap_or("no"),
        rails.refresh_failed.as_deref().unwrap_or("no"),
        rails.max_sends.map_or("none".to_string(), |n| n.to_string()),
        rails.journal.as_deref().unwrap_or("off"),
        idle_shutdown_from_env().as_secs(),
    ));
    for warning in daemon.rails().startup_warnings() {
        daemon.note_error(&format!("RAILS CONFIG: {warning}"));
    }
    let (retention, problems) = retention_from_env();
    for problem in problems {
        daemon.note_error(&format!("JOBS CONFIG: {problem}"));
    }
    if let Err(e) = daemon.restore_jobs(retention) {
        anyhow::bail!(
            "restore of the persisted queue failed: {e:#} (repair or remove it; the daemon will not run without its queue)"
        );
    }

    tokio::spawn(daemon.clone().dispatcher());
    tokio::spawn(daemon.clone().idle_watchdog());

    loop {
        let (stream, _addr) = listener.accept().await?;
        tokio::spawn(daemon.clone().handle_conn(stream));
    }
}

#[cfg(test)]
mod auth_session_tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use tokio::sync::oneshot;

    #[derive(Default)]
    struct MemoryCredentialStore {
        saves: Mutex<Vec<(String, String, String)>>,
        cleared: AtomicBool,
    }

    impl CredentialStore for MemoryCredentialStore {
        fn save(&self, service: &str, refresh_token: &str, username: &str) -> Result<(), String> {
            self.saves.lock().unwrap().push((
                service.to_string(),
                refresh_token.to_string(),
                username.to_string(),
            ));
            Ok(())
        }

        fn clear(&self, _service: &str, _username: &str) -> Result<(), String> {
            self.cleared.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    struct DelayedTokenResponse {
        base: String,
        arrived: oneshot::Receiver<()>,
        release: oneshot::Sender<()>,
        server: tokio::task::JoinHandle<()>,
    }

    async fn delayed_token_response(status: &'static str, body: String) -> DelayedTokenResponse {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (arrived_tx, arrived) = oneshot::channel();
        let (release, release_rx) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = mockggg::read_request(&mut stream).await.unwrap();
            assert_eq!(request.method, "POST");
            assert_eq!(request.path, "/token");
            assert!(request.body.contains("grant_type=refresh_token"));
            assert!(request.body.contains("refresh_token=rt-old"));
            arrived_tx.send(()).unwrap();
            release_rx.await.unwrap();
            let headers = concat!(
                "X-Rate-Limit-Policy: token-request-limit\r\n",
                "X-Rate-Limit-Rules: Ip\r\n",
                "X-Rate-Limit-Ip: 60:30:30\r\n",
                "X-Rate-Limit-Ip-State: 1:30:0\r\n",
            );
            mockggg::respond_with(&mut stream, status, "application/json", headers, &body).await;
        });
        DelayedTokenResponse {
            base,
            arrived,
            release,
            server,
        }
    }

    struct AbandonThenSuccessTokenResponses {
        base: String,
        first_arrived: oneshot::Receiver<()>,
        release_first: oneshot::Sender<()>,
        requests: Arc<Mutex<Vec<String>>>,
        server: tokio::task::JoinHandle<()>,
    }

    async fn abandon_then_success_token_responses(
        success_body: String,
    ) -> AbandonThenSuccessTokenResponses {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (first_arrived_tx, first_arrived) = oneshot::channel();
        let (release_first, release_first_rx) = oneshot::channel();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let server = tokio::spawn(async move {
            let headers = concat!(
                "X-Rate-Limit-Policy: token-request-limit\r\n",
                "X-Rate-Limit-Rules: Ip\r\n",
                "X-Rate-Limit-Ip: 60:30:30\r\n",
                "X-Rate-Limit-Ip-State: 1:30:0\r\n",
            );

            let (mut first, _) = listener.accept().await.unwrap();
            let request = mockggg::read_request(&mut first).await.unwrap();
            assert_eq!(request.method, "POST");
            assert_eq!(request.path, "/token");
            captured.lock().unwrap().push(request.body);
            first_arrived_tx.send(()).unwrap();
            release_first_rx.await.unwrap();
            mockggg::respond_with(
                &mut first,
                "200 OK",
                "application/json",
                headers,
                &success_body,
            )
            .await;

            let (mut second, _) = listener.accept().await.unwrap();
            let request = mockggg::read_request(&mut second).await.unwrap();
            assert_eq!(request.method, "POST");
            assert_eq!(request.path, "/token");
            captured.lock().unwrap().push(request.body);
            mockggg::respond_with(
                &mut second,
                "200 OK",
                "application/json",
                headers,
                &success_body,
            )
            .await;
        });
        AbandonThenSuccessTokenResponses {
            base,
            first_arrived,
            release_first,
            requests,
            server,
        }
    }

    fn tokens(access_token: &str, refresh_token: &str, username: &str) -> auth::TokenResponse {
        auth::TokenResponse {
            access_token: access_token.into(),
            refresh_token: refresh_token.into(),
            expires_in: 3600,
            username: username.into(),
        }
    }

    fn token_body(access_token: &str, refresh_token: &str, username: &str) -> String {
        json!({
            "access_token": access_token,
            "refresh_token": refresh_token,
            "expires_in": 3600,
            "username": username,
        })
        .to_string()
    }

    static AUTH_TEST_LOG_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

    fn test_daemon(base: &str) -> (Arc<Daemon>, Arc<MemoryCredentialStore>, PathBuf) {
        let log_path = std::env::temp_dir().join(format!(
            "acquisition-n2-auth-{}-{}.log",
            std::process::id(),
            AUTH_TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let log = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&log_path)
            .unwrap();
        let credential_store = Arc::new(MemoryCredentialStore::default());
        let daemon = Arc::new(Daemon {
            shared: Mutex::new(Shared {
                jobs: HashMap::new(),
                next_id: 1,
                auth: Sessions::with(AuthSession {
                    access_token: Some("at-expired".into()),
                    access_expires_at: Some(SystemTime::now()),
                    refresh_token: Some("rt-old".into()),
                    username: Some("old-user".into()),
                    keyring: "ok".into(),
                    generations: AuthGenerations {
                        session: 7,
                        access_token: 11,
                        refresh_token: 13,
                    },
                    next_refresh_flight: 1,
                    ..AuthSession::default()
                }),
                connections: 0,
                last_activity: Instant::now(),
                errors: VecDeque::new(),
                active_jobs: HashMap::new(),
            }),
            started: Instant::now(),
            events: broadcast::channel(16).0,
            work: Notify::new(),
            log: Mutex::new(log),
            choke: ChokePoint::new(),
            provider: Provider::mock(base),
            credential_store: credential_store.clone(),
            store_dir: None,
            store: Mutex::new(None),
            jobs_db: Mutex::new(JobDb::open_memory().unwrap()),
            queue_failure: Mutex::new(None),
        });
        (daemon, credential_store, log_path)
    }

    /// Realm on the wire (CONTEXT.md, 2026-09-02): pc is omitted, so a pc
    /// URL and route label are byte-identical whether the param is absent
    /// or explicit — every live send so far stays the same; any other
    /// realm is a segment before the league or name on the URL and a
    /// suffix on the label, so it gets its own probe; a realm a family
    /// does not take renders nothing and is refused at admission.
    #[test]
    fn realm_is_a_segment_before_league_or_name_and_pc_is_silent() {
        let (daemon, _, _) = test_daemon("http://mock");
        let route = |kind: &str, params: Value| daemon.route_for(kind, &params);
        for (kind, absent, explicit) in [
            ("characters", json!({}), json!({ "realm": "pc" })),
            (
                "character",
                json!({ "name": "Exile" }),
                json!({ "realm": "pc", "name": "Exile" }),
            ),
            (
                "stashes",
                json!({ "league": "Standard" }),
                json!({ "realm": "pc", "league": "Standard" }),
            ),
            (
                "stash",
                json!({ "league": "Standard", "id": "t1", "sub": "s1" }),
                json!({ "realm": "pc", "league": "Standard", "id": "t1", "sub": "s1" }),
            ),
            (
                "refresh",
                json!({ "league": "Standard", "all": true }),
                json!({ "realm": "pc", "league": "Standard", "all": true }),
            ),
        ] {
            let a = route(kind, absent).unwrap();
            assert_eq!(
                a,
                route(kind, explicit).unwrap(),
                "{kind}: pc must add nothing"
            );
            assert!(
                !a.0.contains('/'),
                "{kind}: pc label {:?} carries no realm",
                a.0
            );
        }
        assert_eq!(
            route("characters", json!({ "realm": "poe2" })).unwrap(),
            (
                "character-list/poe2".into(),
                "http://mock/character/poe2".into()
            )
        );
        assert_eq!(
            route("character", json!({ "realm": "poe2", "name": "Exile" })).unwrap(),
            (
                "character/poe2".into(),
                "http://mock/character/poe2/Exile".into()
            )
        );
        assert_eq!(
            route("stashes", json!({ "realm": "xbox", "league": "Standard" })).unwrap(),
            (
                "stash-list/xbox".into(),
                "http://mock/stash/xbox/Standard".into()
            )
        );
        assert_eq!(
            route(
                "stash",
                json!({ "realm": "sony", "league": "Standard", "id": "t1", "sub": "s1" })
            )
            .unwrap(),
            (
                "stash/sony".into(),
                "http://mock/stash/sony/Standard/t1/s1".into()
            )
        );
        // PoE1-only families never render a poe2 URL.
        assert_eq!(
            route("stashes", json!({ "realm": "poe2", "league": "Standard" })),
            None
        );
        assert_eq!(
            route(
                "stash",
                json!({ "realm": "poe2", "league": "Standard", "id": "t1" })
            ),
            None
        );
        // Admission refuses before a job id exists — and inside an apply's
        // tuple list, where the vocabulary check already lives.
        let refused = daemon
            .submit(
                "stashes".into(),
                json!({ "realm": "poe2", "league": "Standard" }),
                0,
                "test".into(),
                None,
            )
            .unwrap_err();
        assert!(
            refused.contains("stashes endpoints do not take realm poe2"),
            "{refused}"
        );
        let refused = daemon
            .submit(
                "characters".into(),
                json!({ "realm": "ps5" }),
                0,
                "test".into(),
                None,
            )
            .unwrap_err();
        assert!(refused.contains("unknown realm \"ps5\""), "{refused}");
        let refused = validate_apply(&json!({ "jobs": [
            { "kind": "stashes", "params": { "realm": "xbox", "league": "Standard" } },
            { "kind": "stash", "params": { "realm": "poe2", "league": "Standard", "id": "t1" } },
        ] }))
        .unwrap_err();
        assert!(
            refused.starts_with("apply job 1: the stashes endpoints"),
            "{refused}"
        );
        assert_eq!(
            daemon.shared.lock().unwrap().jobs.len(),
            0,
            "refusals admit nothing"
        );
    }

    fn remove_test_log(path: &PathBuf) {
        let _ = std::fs::remove_file(path);
    }

    async fn wait_for_refresh_waiters(daemon: &Daemon, expected: usize) {
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let receivers = daemon
                    .shared
                    .lock()
                    .unwrap()
                    .auth
                    .one()
                    .refresh_flight
                    .as_ref()
                    .map_or(0, |flight| flight.result.receiver_count());
                if receivers == expected {
                    return;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("refresh waiters joined the flight");
    }

    #[tokio::test]
    async fn concurrent_expiry_has_one_refresh_owner_and_shared_waiter_result() {
        let delayed = delayed_token_response(
            "200 OK",
            token_body("at-rotated", "rt-rotated", "shared-user"),
        )
        .await;
        let (daemon, store, log_path) = test_daemon(&delayed.base);

        let owner_daemon = daemon.clone();
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(None, false).await });
        delayed.arrived.await.unwrap();
        let waiter_daemon = daemon.clone();
        let waiter =
            tokio::spawn(async move { waiter_daemon.valid_access_token(None, false).await });
        tokio::task::yield_now().await;
        delayed.release.send(()).unwrap();

        let owner_result = owner.await.unwrap().unwrap();
        let waiter_result = waiter.await.unwrap().unwrap();
        assert_eq!(owner_result, ("at-rotated".into(), "shared-user".into()));
        assert_eq!(waiter_result, owner_result);
        assert_eq!(store.saves.lock().unwrap().len(), 1);
        delayed.server.await.unwrap();
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn authentication_completes_before_the_api_send_takes_a_gate_slot() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (hold_arrived_tx, hold_arrived) = oneshot::channel();
        let (release_hold, release_hold_rx) = oneshot::channel();
        let (token_arrived_tx, token_arrived) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (mut hold, _) = listener.accept().await.unwrap();
            let request = mockggg::read_request(&mut hold).await.unwrap();
            assert_eq!(request.path, "/hold");
            let headers = concat!(
                "HTTP/1.1 200 OK\r\n",
                "X-Rate-Limit-Policy: hold-policy\r\n",
                "X-Rate-Limit-Rules: Ip\r\n",
                "X-Rate-Limit-Ip: 60:30:30\r\n",
                "X-Rate-Limit-Ip-State: 1:30:0\r\n",
                "Content-Type: application/json\r\n",
                "Content-Length: 2\r\n",
                "Connection: close\r\n\r\n",
            );
            hold.write_all(headers.as_bytes()).await.unwrap();
            hold_arrived_tx.send(()).unwrap();

            let (mut token, _) = listener.accept().await.unwrap();
            let request = mockggg::read_request(&mut token).await.unwrap();
            assert_eq!(request.method, "POST");
            assert_eq!(request.path, "/token");
            token_arrived_tx.send(()).unwrap();
            let token_headers = concat!(
                "X-Rate-Limit-Policy: token-request-limit\r\n",
                "X-Rate-Limit-Rules: Ip\r\n",
                "X-Rate-Limit-Ip: 60:30:30\r\n",
                "X-Rate-Limit-Ip-State: 1:30:0\r\n",
            );
            mockggg::respond_with(
                &mut token,
                "200 OK",
                "application/json",
                token_headers,
                &token_body("at-live", "rt-rotated", "test-user"),
            )
            .await;

            let (mut api, _) = listener.accept().await.unwrap();
            let request = mockggg::read_request(&mut api).await.unwrap();
            assert_eq!(request.method, "GET");
            assert_eq!(request.path, "/character");
            let api_headers = concat!(
                "X-Rate-Limit-Policy: character-list-request-limit\r\n",
                "X-Rate-Limit-Rules: Account\r\n",
                "X-Rate-Limit-Account: 2:10:60,5:300:300\r\n",
                "X-Rate-Limit-Account-State: 1:10:0,1:300:0\r\n",
            );
            mockggg::respond_with(
                &mut api,
                "200 OK",
                "application/json",
                api_headers,
                r#"{"characters":[]}"#,
            )
            .await;

            release_hold_rx.await.unwrap();
            hold.write_all(b"{}").await.unwrap();
        });

        let (daemon, _, log_path) = test_daemon(&base);
        let hold = {
            let daemon = daemon.clone();
            let url = format!("{base}/hold");
            tokio::spawn(async move {
                daemon
                    .choke
                    .get("hold-route", &url, daemon.choke.now())
                    .await
            })
        };
        hold_arrived.await.unwrap();
        let api = {
            let daemon = daemon.clone();
            tokio::spawn(async move {
                daemon
                    .execute_inner(
                        1,
                        "characters",
                        serde_json::Value::Null,
                        Some("character-list".into()),
                        None,
                        daemon.choke.now(),
                    )
                    .await
            })
        };

        tokio::time::timeout(Duration::from_secs(1), token_arrived)
            .await
            .expect("refresh can use the remaining gate slot before the API send")
            .unwrap();
        assert!(matches!(
            api.await.unwrap().unwrap(),
            Outcome::Success { .. }
        ));
        release_hold.send(()).unwrap();
        hold.await.unwrap().unwrap();
        server.await.unwrap();
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn abandoned_refresh_owner_completes_all_waiters_and_allows_retry() {
        let responses = abandon_then_success_token_responses(token_body(
            "at-retried",
            "rt-retried",
            "retry-user",
        ))
        .await;
        let (daemon, store, log_path) = test_daemon(&responses.base);
        let before = daemon.shared.lock().unwrap().auth.one().generations;

        let owner_daemon = daemon.clone();
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(None, false).await });
        responses.first_arrived.await.unwrap();

        let mut waiters = Vec::new();
        for _ in 0..3 {
            let waiter_daemon = daemon.clone();
            waiters.push(tokio::spawn(async move {
                waiter_daemon.valid_access_token(None, false).await
            }));
        }
        wait_for_refresh_waiters(&daemon, waiters.len()).await;

        owner.abort();
        assert!(owner.await.unwrap_err().is_cancelled());
        let mut waiter_errors = Vec::new();
        for waiter in waiters {
            waiter_errors.push(
                tokio::time::timeout(Duration::from_secs(1), waiter)
                    .await
                    .expect("abandoned refresh waiter completed")
                    .unwrap()
                    .unwrap_err(),
            );
        }
        assert_eq!(waiter_errors, vec![REFRESH_OWNER_ABANDONED; 3]);
        {
            let s = daemon.shared.lock().unwrap();
            assert!(s.auth.one().refresh_flight.is_none());
            assert_eq!(s.auth.one().generations, before);
            assert_eq!(s.auth.one().refresh_token.as_deref(), Some("rt-old"));
            assert_eq!(s.auth.one().access_token.as_deref(), Some("at-expired"));
        }
        assert!(store.saves.lock().unwrap().is_empty());

        responses.release_first.send(()).unwrap();
        let retry = tokio::time::timeout(
            Duration::from_secs(2),
            daemon.valid_access_token(None, false),
        )
        .await
        .expect("retry after owner abandonment completed")
        .unwrap();
        assert_eq!(retry, ("at-retried".into(), "retry-user".into()));
        responses.server.await.unwrap();
        let requests = responses.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert!(
            requests
                .iter()
                .all(|body| body.contains("refresh_token=rt-old"))
        );
        assert_eq!(store.saves.lock().unwrap().len(), 1);
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn abandoned_refresh_owner_without_waiters_allows_retry() {
        let responses = abandon_then_success_token_responses(token_body(
            "at-retried",
            "rt-retried",
            "retry-user",
        ))
        .await;
        let (daemon, store, log_path) = test_daemon(&responses.base);
        let before = daemon.shared.lock().unwrap().auth.one().generations;

        let owner_daemon = daemon.clone();
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(None, false).await });
        responses.first_arrived.await.unwrap();
        owner.abort();
        assert!(owner.await.unwrap_err().is_cancelled());
        {
            let s = daemon.shared.lock().unwrap();
            assert!(s.auth.one().refresh_flight.is_none());
            assert_eq!(s.auth.one().generations, before);
            assert_eq!(s.auth.one().refresh_token.as_deref(), Some("rt-old"));
            assert_eq!(s.auth.one().access_token.as_deref(), Some("at-expired"));
        }
        assert!(store.saves.lock().unwrap().is_empty());

        responses.release_first.send(()).unwrap();
        let retry = tokio::time::timeout(
            Duration::from_secs(2),
            daemon.valid_access_token(None, false),
        )
        .await
        .expect("retry after owner abandonment completed")
        .unwrap();
        assert_eq!(retry, ("at-retried".into(), "retry-user".into()));
        responses.server.await.unwrap();
        let requests = responses.requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert!(
            requests
                .iter()
                .all(|body| body.contains("refresh_token=rt-old"))
        );
        assert_eq!(store.saves.lock().unwrap().len(), 1);
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn successful_refresh_rotation_advances_token_generations_and_persists_new_token() {
        let delayed = delayed_token_response(
            "200 OK",
            token_body("at-rotated", "rt-rotated", "rotated-user"),
        )
        .await;
        let (daemon, store, log_path) = test_daemon(&delayed.base);
        let before = daemon.shared.lock().unwrap().auth.one().generations;

        let refresh_daemon = daemon.clone();
        let refresh =
            tokio::spawn(async move { refresh_daemon.valid_access_token(None, false).await });
        delayed.arrived.await.unwrap();
        delayed.release.send(()).unwrap();
        assert_eq!(
            refresh.await.unwrap().unwrap(),
            ("at-rotated".into(), "rotated-user".into())
        );

        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.one().refresh_token.as_deref(), Some("rt-rotated"));
            assert_eq!(s.auth.one().access_token.as_deref(), Some("at-rotated"));
            assert_eq!(s.auth.one().generations.session, before.session);
            assert_ne!(s.auth.one().generations.access_token, before.access_token);
            assert_ne!(s.auth.one().generations.refresh_token, before.refresh_token);
        }
        assert_eq!(
            store.saves.lock().unwrap().as_slice(),
            [(
                "acquisition-playground".into(),
                "rt-rotated".into(),
                "rotated-user".into()
            )]
        );
        delayed.server.await.unwrap();
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn refresh_failure_is_shared_and_leaves_token_state_unchanged() {
        let delayed =
            delayed_token_response("400 Bad Request", r#"{"error":"invalid_grant"}"#.into()).await;
        let (daemon, store, log_path) = test_daemon(&delayed.base);
        let before = daemon.shared.lock().unwrap().auth.one().generations;

        let owner_daemon = daemon.clone();
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(None, false).await });
        delayed.arrived.await.unwrap();
        let waiter_daemon = daemon.clone();
        let waiter =
            tokio::spawn(async move { waiter_daemon.valid_access_token(None, false).await });
        tokio::task::yield_now().await;
        delayed.release.send(()).unwrap();

        let owner_error = owner.await.unwrap().unwrap_err();
        let waiter_error = waiter.await.unwrap().unwrap_err();
        assert_eq!(waiter_error, owner_error);
        assert!(owner_error.contains("400 Bad Request"));
        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.one().generations, before);
            assert_eq!(s.auth.one().refresh_token.as_deref(), Some("rt-old"));
            assert_eq!(s.auth.one().access_token.as_deref(), Some("at-expired"));
            assert!(s.auth.one().refresh_flight.is_none());
        }
        assert!(store.saves.lock().unwrap().is_empty());
        delayed.server.await.unwrap();
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn logout_during_refresh_rejects_stale_completion_in_memory_and_keyring() {
        let delayed =
            delayed_token_response("200 OK", token_body("at-stale", "rt-stale", "stale-user"))
                .await;
        let (daemon, store, log_path) = test_daemon(&delayed.base);

        let refresh_daemon = daemon.clone();
        let refresh =
            tokio::spawn(async move { refresh_daemon.valid_access_token(None, false).await });
        delayed.arrived.await.unwrap();
        daemon.logout(None).unwrap();
        delayed.release.send(()).unwrap();

        assert_eq!(
            refresh.await.unwrap().unwrap_err(),
            SESSION_CHANGED_DURING_REFRESH
        );
        assert!(
            daemon.shared.lock().unwrap().auth.by_account.is_empty(),
            "the session is gone"
        );
        assert!(store.cleared.load(Ordering::SeqCst));
        assert!(store.saves.lock().unwrap().is_empty());
        delayed.server.await.unwrap();
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn reauthentication_during_refresh_keeps_new_session_and_rejects_old_completion() {
        // A re-login as the *same* account while its refresh is in flight:
        // the new session replaces the old one (generations advance), and
        // the stale completion is rejected. Another account's login would
        // not touch this session at all (see the two-session tests).
        let delayed =
            delayed_token_response("200 OK", token_body("at-stale", "rt-stale", "old-user")).await;
        let (daemon, store, log_path) = test_daemon(&delayed.base);
        let before = daemon.shared.lock().unwrap().auth.one().generations;

        let refresh_daemon = daemon.clone();
        let refresh =
            tokio::spawn(async move { refresh_daemon.valid_access_token(None, false).await });
        delayed.arrived.await.unwrap();
        let flow_generation = daemon.begin_auth_flow();
        let staged = daemon
            .stage_auth_flow(
                flow_generation,
                tokens("at-reauth", "rt-reauth", "old-user"),
            )
            .expect("current flow stages");
        assert!(daemon.complete_login(flow_generation, &staged, "u-old"));
        delayed.release.send(()).unwrap();

        assert_eq!(
            refresh.await.unwrap().unwrap_err(),
            SESSION_CHANGED_DURING_REFRESH
        );
        assert_eq!(
            daemon.shared.lock().unwrap().auth.one().uuid.as_deref(),
            Some("u-old"),
            "the registered session carries its uuid"
        );
        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.one().access_token.as_deref(), Some("at-reauth"));
            assert_eq!(s.auth.one().refresh_token.as_deref(), Some("rt-reauth"));
            assert_eq!(s.auth.one().username.as_deref(), Some("old-user"));
            assert_ne!(s.auth.one().generations.session, before.session);
        }
        assert_eq!(
            store.saves.lock().unwrap().as_slice(),
            [(
                "acquisition-playground".into(),
                "rt-reauth".into(),
                "old-user".into()
            )]
        );
        delayed.server.await.unwrap();
        remove_test_log(&log_path);
    }

    /// A refresh that renames the account updates the existing index
    /// mapping in place (the session carries its uuid, so `record_login`
    /// follows it) — never a second, uuid-less entry that would make both
    /// the username and uuid selectors ambiguous.
    #[tokio::test]
    async fn a_refresh_rename_updates_the_index_mapping_in_place() {
        let delayed =
            delayed_token_response("200 OK", token_body("at-new", "rt-new", "NewName#2")).await;
        let (mut daemon, _, log_path) = test_daemon(&delayed.base);
        let dir = std::env::temp_dir().join(format!(
            "acq-rename-{}-{}",
            std::process::id(),
            AUTH_TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        Arc::get_mut(&mut daemon).unwrap().store_dir = Some(dir.clone());
        // As a completed login left things: indexed with the uuid, and the
        // session carrying it.
        {
            let mut index = Index::load(&dir).unwrap();
            index.record_login("old-user", "u-1", true, 1).unwrap();
        }
        daemon.shared.lock().unwrap().auth.one_mut().uuid = Some("u-1".into());

        let refresh_daemon = daemon.clone();
        let refresh =
            tokio::spawn(async move { refresh_daemon.valid_access_token(None, false).await });
        delayed.arrived.await.unwrap();
        delayed.release.send(()).unwrap();
        assert_eq!(
            refresh.await.unwrap().unwrap(),
            ("at-new".into(), "NewName#2".into())
        );

        let index = Index::load(&dir).unwrap();
        assert_eq!(index.entries().len(), 1, "one entry, moved — no twin");
        let entry = index.get("NewName#2").expect("the entry follows the name");
        assert_eq!(entry.uuid.as_deref(), Some("u-1"));
        assert!(
            daemon
                .shared
                .lock()
                .unwrap()
                .auth
                .find("NewName#2")
                .is_some_and(|x| x.uuid.as_deref() == Some("u-1")),
            "the session moved with its uuid"
        );
        delayed.server.await.unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        remove_test_log(&log_path);
    }

    // ---- multi-account step (2): the job's account is what runs ---------

    #[test]
    fn submit_resolves_the_account_against_the_live_session() {
        let (daemon, _, log_path) = test_daemon("http://127.0.0.1:1");
        // Session is "old-user" (test_daemon).
        assert_eq!(
            daemon.resolve_account("characters", None),
            Ok(Some("old-user".into()))
        );
        assert_eq!(
            daemon.resolve_account("characters", Some("OLD-USER")),
            Ok(Some("old-user".into()))
        );
        let err = daemon
            .resolve_account("characters", Some("Other#1"))
            .unwrap_err();
        assert!(err.contains("Other#1") && err.contains("old-user"), "{err}");
        // Two sessions live: no selector is ambiguous for a token kind.
        daemon.shared.lock().unwrap().auth.replace(AuthSession {
            username: Some("Other#1".into()),
            ..AuthSession::default()
        });
        let err = daemon.resolve_account("characters", None).unwrap_err();
        assert!(err.contains("several accounts"), "{err}");
        assert_eq!(
            daemon.resolve_account("characters", Some("other")),
            Ok(Some("Other#1".into()))
        );
        daemon.shared.lock().unwrap().auth.by_account.clear();
        // No session: an auth-required kind is refused at submit; a kind
        // that never sends with a token simply has no account.
        let err = daemon.resolve_account("characters", None).unwrap_err();
        assert!(err.contains("not logged in"), "{err}");
        assert_eq!(daemon.resolve_account("sleep", None), Ok(None));
        let err = daemon
            .resolve_account("characters", Some("Other#1"))
            .unwrap_err();
        assert!(err.contains("not logged in"), "{err}");
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn a_token_is_refused_once_the_session_no_longer_matches_the_job() {
        let (daemon, _, log_path) = test_daemon("http://127.0.0.1:1");
        {
            let mut s = daemon.shared.lock().unwrap();
            s.auth.one_mut().access_token = Some("at-live".into());
            s.auth.one_mut().access_expires_at =
                Some(SystemTime::now() + Duration::from_secs(3600));
        }
        // The job's account has a session: its live token, no refresh.
        let (token, user) = daemon
            .valid_access_token(Some("old-user"), false)
            .await
            .unwrap();
        assert_eq!((token.as_str(), user.as_str()), ("at-live", "old-user"));
        // Another account logging in changes nothing for old-user's jobs.
        daemon.shared.lock().unwrap().auth.replace(AuthSession {
            username: Some("new-user".into()),
            access_token: Some("at-new".into()),
            access_expires_at: Some(SystemTime::now() + Duration::from_secs(3600)),
            ..AuthSession::default()
        });
        let (token, _) = daemon
            .valid_access_token(Some("old-user"), false)
            .await
            .unwrap();
        assert_eq!(token, "at-live");
        // No selector with two sessions live: refused, never guessed.
        let err = daemon.valid_access_token(None, false).await.unwrap_err();
        assert!(
            err.contains("several accounts") && err.contains("--account"),
            "{err}"
        );
        // old-user logs out: its jobs fail at token time, before any send.
        daemon.logout(Some("old-user")).unwrap();
        let err = daemon
            .valid_access_token(Some("old-user"), false)
            .await
            .unwrap_err();
        assert!(err.contains("no session for old-user"), "{err}");
        // The other session is untouched, and is now the sole one.
        assert!(daemon.valid_access_token(None, false).await.is_ok());
        remove_test_log(&log_path);
    }

    #[test]
    fn session_keys_follow_renames_so_replace_finds_the_right_session() {
        let mut sessions = Sessions::with(AuthSession {
            username: Some("A#1".into()),
            refresh_token: Some("rt-a".into()),
            ..AuthSession::default()
        });
        let before = sessions.find("A#1").unwrap().generations;
        // A refresh reported a new name: the session moves under it.
        sessions.rename("A#1", "B#2");
        assert!(sessions.find("A#1").is_none());
        assert_eq!(
            sessions.find("B#2").unwrap().refresh_token.as_deref(),
            Some("rt-a")
        );
        assert_eq!(sessions.usernames(), vec!["B#2".to_string()]);
        // A login as B now replaces that one session (generations advance)
        // instead of leaving a stale twin behind.
        sessions.replace(AuthSession {
            username: Some("B#2".into()),
            refresh_token: Some("rt-b".into()),
            ..AuthSession::default()
        });
        assert_eq!(sessions.by_account.len(), 1);
        let after = sessions.find("B#2").unwrap();
        assert_eq!(after.refresh_token.as_deref(), Some("rt-b"));
        assert_ne!(after.generations.session, before.session);
    }

    #[test]
    fn a_response_is_filed_under_the_job_account_not_the_session() {
        let (mut daemon, _, log_path) = test_daemon("http://127.0.0.1:1");
        let dir =
            std::env::temp_dir().join(format!("acq-store-{}-{}", std::process::id(), line!()));
        let _ = std::fs::remove_dir_all(&dir);
        Arc::get_mut(&mut daemon).unwrap().store_dir = Some(dir.clone());
        // Session is "old-user"; the job was submitted as "A#1" (a login
        // happened in between). The body lands in A#1's file.
        daemon
            .record(
                Some("A#1"),
                "stashes",
                &json!({ "league": "Standard" }),
                &json!({ "stashes": [ { "id": "t1", "name": "T", "type": "PremiumStash" } ] }),
            )
            .unwrap();
        assert!(acquisition_store::account_path(&dir, "A#1").exists());
        assert!(!acquisition_store::account_path(&dir, "old-user").exists());
        // No account at all: nothing recorded (absorbed, not a job
        // failure), an error noted.
        daemon
            .record(None, "stashes", &json!({}), &json!({ "stashes": [] }))
            .unwrap();
        assert!(
            daemon
                .shared
                .lock()
                .unwrap()
                .errors
                .iter()
                .any(|(_, m)| m.contains("no account"))
        );
        std::fs::remove_dir_all(&dir).unwrap();
        remove_test_log(&log_path);
    }
}

#[cfg(test)]
mod response_tests {
    use super::*;

    fn rate() -> Value {
        serde_json::json!({ "retry-after": "1" })
    }

    #[test]
    fn bounded_429_requeue_keeps_the_existing_two_retry_contract() {
        assert!(may_requeue_429(0, false));
        assert!(may_requeue_429(1, false));
        assert!(!may_requeue_429(2, false));
        assert!(!may_requeue_429(0, true));
    }

    #[test]
    fn acceptable_429_is_the_only_body_outcome_that_requeues() {
        let error = classify_api_body(
            reqwest::StatusCode::TOO_MANY_REQUESTS,
            &RetryAfter::Acceptable { seconds: 1 },
            "/character",
            rate(),
            Ok("slow down".into()),
        )
        .unwrap_err();
        assert!(matches!(error, ApiError::RateLimited(_)));

        let terminal = [
            RetryAfter::Missing,
            RetryAfter::Malformed { raw: "soon".into() },
            RetryAfter::Negative { raw: "-1".into() },
            RetryAfter::OverCap { raw: "901".into() },
        ];
        for retry_after in terminal {
            let error = classify_api_body(
                reqwest::StatusCode::TOO_MANY_REQUESTS,
                &retry_after,
                "/character",
                rate(),
                Ok("slow down".into()),
            )
            .unwrap_err();
            let ApiError::Other(message) = error else {
                panic!("{retry_after} entered the retryable arm")
            };
            assert!(message.contains("terminal rate limit"));
            assert!(message.contains("NOT retrying"));
        }
    }

    #[test]
    fn body_read_transport_failure_is_not_recast_as_bad_json() {
        let error = classify_api_body(
            reqwest::StatusCode::OK,
            &RetryAfter::Missing,
            "/character",
            Value::Null,
            Err("connection closed before message completed".into()),
        )
        .unwrap_err();
        let ApiError::Other(message) = error else {
            panic!("body transport failure used the wrong error kind")
        };
        assert!(message.contains("body read transport failure"));
        assert!(message.contains("connection closed before message completed"));
        assert!(!message.contains("bad JSON"));
    }

    #[test]
    fn http_status_keeps_precedence_over_error_body_transport_failure() {
        let error = classify_api_body(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            &RetryAfter::Missing,
            "/character",
            Value::Null,
            Err("body reset".into()),
        )
        .unwrap_err();
        let ApiError::Other(message) = error else {
            panic!("HTTP failure used the wrong error kind")
        };
        assert!(message.contains("returned 500 Internal Server Error"));
        assert!(message.contains("body read transport failure: body reset"));
    }

    #[test]
    fn malformed_success_body_remains_a_json_failure() {
        let error = classify_api_body(
            reqwest::StatusCode::OK,
            &RetryAfter::Missing,
            "/character",
            Value::Null,
            Ok("not json".into()),
        )
        .unwrap_err();
        let ApiError::Other(message) = error else {
            panic!("JSON failure used the wrong error kind")
        };
        assert!(message.contains("bad JSON"));
    }
}

#[cfg(test)]
mod dispatcher_tests {
    use super::*;
    use crate::ratelimit::Clock;
    use std::collections::HashSet;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use tokio::sync::{Semaphore, oneshot};

    /// Wall-clock origin for scenarios: 2000-01-01T00:00:00Z. Deliberately
    /// synthetic so a journal written under this clock is recognizable as a
    /// scenario even if someone strips the header line.
    const MANUAL_WALL_ORIGIN: Duration = Duration::from_secs(946_684_800);

    struct ManualClock {
        now: Mutex<Instant>,
        wall: Mutex<SystemTime>,
    }

    impl ManualClock {
        fn new() -> Self {
            ManualClock {
                now: Mutex::new(Instant::now()),
                wall: Mutex::new(SystemTime::UNIX_EPOCH + MANUAL_WALL_ORIGIN),
            }
        }

        /// Time passes normally: both faces move.
        fn advance(&self, duration: Duration) {
            let mut now = self.now.lock().unwrap();
            *now = now.checked_add(duration).expect("bounded test deadline");
            *self.wall.lock().unwrap() += duration;
        }

        /// The lid closes: the wall clock moves, the monotonic clock does
        /// not. This is the R8 shape.
        fn laptop_sleep(&self, duration: Duration) {
            *self.wall.lock().unwrap() += duration;
        }
    }

    impl Clock for ManualClock {
        fn now(&self) -> Instant {
            *self.now.lock().unwrap()
        }

        fn wall(&self) -> SystemTime {
            *self.wall.lock().unwrap()
        }

        fn kind(&self) -> &'static str {
            "manual"
        }

        fn sleep(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
            Box::pin(async move {
                self.advance(duration);
                tokio::task::yield_now().await;
            })
        }
    }

    struct BlockingClock {
        now: Mutex<Instant>,
        origin: Mutex<Instant>,
        sleepers: AtomicUsize,
        releases: Semaphore,
    }

    impl BlockingClock {
        fn new() -> Self {
            let start = Instant::now();
            BlockingClock {
                now: Mutex::new(start),
                origin: Mutex::new(start),
                sleepers: AtomicUsize::new(0),
                releases: Semaphore::new(0),
            }
        }

        async fn wait_for_sleepers(&self, expected: usize) {
            tokio::time::timeout(Duration::from_secs(1), async {
                while self.sleepers.load(Ordering::SeqCst) < expected {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .expect("dispatcher jobs reached their limiter waits");
        }

        fn release(&self, count: usize) {
            self.releases.add_permits(count);
        }
    }

    impl Clock for BlockingClock {
        fn now(&self) -> Instant {
            *self.now.lock().unwrap()
        }

        fn wall(&self) -> SystemTime {
            SystemTime::UNIX_EPOCH
                + MANUAL_WALL_ORIGIN
                + self.now().duration_since(*self.origin.lock().unwrap())
        }

        fn kind(&self) -> &'static str {
            "manual"
        }

        fn sleep(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
            Box::pin(async move {
                self.sleepers.fetch_add(1, Ordering::SeqCst);
                self.releases
                    .acquire()
                    .await
                    .expect("test clock remains open")
                    .forget();
                let mut now = self.now.lock().unwrap();
                *now = now.checked_add(duration).expect("bounded test deadline");
            })
        }
    }

    struct NoopCredentialStore;

    impl CredentialStore for NoopCredentialStore {
        fn save(
            &self,
            _service: &str,
            _refresh_token: &str,
            _username: &str,
        ) -> Result<(), String> {
            Ok(())
        }

        fn clear(&self, _service: &str, _username: &str) -> Result<(), String> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct RecordingCredentialStore {
        saves: Mutex<Vec<(String, String, String)>>,
        cleared: Mutex<Vec<String>>,
    }

    impl CredentialStore for RecordingCredentialStore {
        fn save(&self, service: &str, refresh_token: &str, username: &str) -> Result<(), String> {
            self.saves.lock().unwrap().push((
                service.to_string(),
                refresh_token.to_string(),
                username.to_string(),
            ));
            Ok(())
        }

        fn clear(&self, _service: &str, username: &str) -> Result<(), String> {
            self.cleared.lock().unwrap().push(username.to_string());
            Ok(())
        }
    }

    struct ScriptedResponse {
        method: &'static str,
        /// The request path this response answers; the server asserts it.
        path: &'static str,
        status: &'static str,
        headers: String,
        body: String,
    }

    impl ScriptedResponse {
        fn full(
            method: &'static str,
            status: &'static str,
            retry_after: Option<u64>,
            body: &str,
        ) -> Self {
            let retry_after = retry_after
                .map(|seconds| format!("Retry-After: {seconds}\r\n"))
                .unwrap_or_default();
            ScriptedResponse {
                method,
                path: "/fetch",
                status,
                headers: format!(
                    "X-Rate-Limit-Policy: dispatcher-test-policy\r\nX-Rate-Limit-Rules: Account\r\nX-Rate-Limit-Account: 100:1:60\r\nX-Rate-Limit-Account-State: 0:1:0\r\n{retry_after}"
                ),
                body: body.into(),
            }
        }

        fn malformed_head_429() -> Self {
            ScriptedResponse {
                method: "HEAD",
                path: "/fetch",
                status: "429 Too Many Requests",
                headers: "X-Rate-Limit-Policy: dispatcher-test-policy\r\nRetry-After: 0\r\n".into(),
                body: String::new(),
            }
        }
    }

    async fn scripted_server(
        responses: Vec<ScriptedResponse>,
    ) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let task = tokio::spawn(async move {
            for response in responses {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = mockggg::read_request(&mut stream).await.unwrap();
                assert_eq!(request.method, response.method);
                assert_eq!(request.path, response.path);
                captured.lock().unwrap().push(request.method);
                mockggg::respond_with(
                    &mut stream,
                    response.status,
                    "application/json",
                    &response.headers,
                    &response.body,
                )
                .await;
            }
        });
        (base, requests, task)
    }

    async fn saturated_probe_server() -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            for (path, policy) in [
                ("/character", "character-test-policy"),
                ("/stash/Standard", "stash-list-test-policy"),
            ] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = mockggg::read_request(&mut stream).await.unwrap();
                assert_eq!(request.method, "HEAD");
                assert_eq!(request.path, path);
                let headers = format!(
                    "X-Rate-Limit-Policy: {policy}\r\nX-Rate-Limit-Rules: Account\r\nX-Rate-Limit-Account: 1:1:60\r\nX-Rate-Limit-Account-State: 1:1:0\r\n"
                );
                mockggg::respond_with(
                    &mut stream,
                    "204 No Content",
                    "application/json",
                    &headers,
                    "",
                )
                .await;
            }
        });
        (base, task)
    }

    async fn delayed_token_server() -> (
        String,
        oneshot::Receiver<()>,
        oneshot::Sender<()>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (arrived_tx, arrived_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let request = mockggg::read_request(&mut stream).await.unwrap();
            assert_eq!(request.method, "POST");
            assert_eq!(request.path, "/token");
            assert!(request.body.contains("grant_type=refresh_token"));
            assert!(request.body.contains("refresh_token=rt-old"));
            arrived_tx.send(()).unwrap();
            release_rx.await.unwrap();
            let body = json!({
                "access_token": "at-new",
                "refresh_token": "rt-rotated",
                "expires_in": 3600,
                "username": "test-user",
            })
            .to_string();
            let headers = concat!(
                "X-Rate-Limit-Policy: token-request-limit\r\n",
                "X-Rate-Limit-Rules: Ip\r\n",
                "X-Rate-Limit-Ip: 60:30:30\r\n",
                "X-Rate-Limit-Ip-State: 1:30:0\r\n",
            );
            mockggg::respond_with(&mut stream, "200 OK", "application/json", headers, &body).await;
        });
        (base, arrived_rx, release_tx, task)
    }

    async fn wait_for_refresh_waiters(daemon: &Daemon, expected_at_least: usize) {
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let receivers = daemon
                    .shared
                    .lock()
                    .unwrap()
                    .auth
                    .one()
                    .refresh_flight
                    .as_ref()
                    .map_or(0, |flight| flight.result.receiver_count());
                if receivers >= expected_at_least {
                    return;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("concurrent API callers joined the refresh flight");
    }

    static TEST_LOG_ID: AtomicU64 = AtomicU64::new(1);

    fn test_daemon(base: &str, clock: Arc<dyn Clock>) -> (Arc<Daemon>, PathBuf) {
        test_daemon_with(Provider::mock(base), clock, RailsConfig::default())
    }

    fn test_log_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "acquisition-n1-dispatcher-{}-{}.log",
            std::process::id(),
            TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    /// The send journal every harness daemon writes, beside its log.
    fn journal_of(log_path: &std::path::Path) -> PathBuf {
        log_path.with_extension("jsonl")
    }

    /// Every harness daemon journals, on the test's clock, so that
    /// `assert_journal_matches_wire` can hold the daemon's own account of
    /// what it sent against what the server received. The rails handle
    /// comes back through `daemon.choke.rails()`.
    fn test_daemon_with(
        provider: Provider,
        clock: Arc<dyn Clock>,
        config: RailsConfig,
    ) -> (Arc<Daemon>, PathBuf) {
        let log_path = test_log_path();
        let _ = std::fs::remove_file(journal_of(&log_path));
        let rails = Arc::new(Rails::with_config_and_clock(
            RailsConfig {
                journal_path: Some(journal_of(&log_path)),
                ..config
            },
            clock.clone(),
        ));
        test_daemon_scenario_at(
            provider,
            clock,
            rails,
            Arc::new(OsCredentialStore),
            log_path,
        )
    }

    fn test_daemon_scenario(
        provider: Provider,
        clock: Arc<dyn Clock>,
        rails: Arc<Rails>,
        credential_store: Arc<dyn CredentialStore>,
    ) -> (Arc<Daemon>, PathBuf) {
        test_daemon_scenario_at(provider, clock, rails, credential_store, test_log_path())
    }

    fn test_daemon_scenario_at(
        provider: Provider,
        clock: Arc<dyn Clock>,
        rails: Arc<Rails>,
        credential_store: Arc<dyn CredentialStore>,
        log_path: PathBuf,
    ) -> (Arc<Daemon>, PathBuf) {
        let log = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&log_path)
            .unwrap();
        let daemon = Arc::new(Daemon {
            shared: Mutex::new(Shared {
                jobs: HashMap::new(),
                next_id: 1,
                auth: Sessions::with(AuthSession::default()),
                connections: 0,
                last_activity: Instant::now(),
                errors: VecDeque::new(),
                active_jobs: HashMap::new(),
            }),
            started: Instant::now(),
            events: broadcast::channel(256).0,
            work: Notify::new(),
            log: Mutex::new(log),
            choke: ChokePoint::with_clock_and_rails(clock, rails),
            provider,
            credential_store,
            store_dir: None,
            store: Mutex::new(None),
            jobs_db: Mutex::new(JobDb::open_memory().unwrap()),
            queue_failure: Mutex::new(None),
        });
        // Test daemons know what the real one knows about routes.
        Daemon::declare_route_knowledge(&daemon.choke);
        (daemon, log_path)
    }

    /// A server that answers by *what was sent*, not by position in a
    /// script: a stale bearer gets 401 on any route, the token endpoint
    /// rotates to a fresh one, everything else succeeds. Runs until aborted.
    /// Scenario tests assert invariants over the journal, so the server must
    /// not encode the expected sequence.
    async fn bearer_aware_server(
        stale_bearer: &'static str,
        fresh_bearer: &'static str,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            loop {
                let (mut stream, _) = listener.accept().await.unwrap();
                let Some(request) = mockggg::read_request(&mut stream).await else {
                    continue;
                };
                let bearer = request
                    .headers
                    .get("authorization")
                    .and_then(|v| v.strip_prefix("Bearer "))
                    .unwrap_or("");
                let api_headers = "X-Rate-Limit-Policy: scenario-policy\r\nX-Rate-Limit-Rules: Account\r\nX-Rate-Limit-Account: 100:1:60\r\nX-Rate-Limit-Account-State: 0:1:0\r\n";
                let (status, headers, body) = if request.path == "/token" {
                    let body = json!({
                        "access_token": fresh_bearer,
                        "refresh_token": "rt-rotated",
                        "expires_in": 3600,
                        "username": "scenario-user",
                    })
                    .to_string();
                    (
                        "200 OK",
                        "X-Rate-Limit-Policy: token-request-limit\r\nX-Rate-Limit-Rules: Ip\r\nX-Rate-Limit-Ip: 60:30:30\r\nX-Rate-Limit-Ip-State: 1:30:0\r\n",
                        body,
                    )
                } else if bearer == stale_bearer {
                    (
                        "401 Unauthorized",
                        api_headers,
                        r#"{"error":"expired"}"#.to_string(),
                    )
                } else if request.method == "HEAD" {
                    ("204 No Content", api_headers, String::new())
                } else {
                    ("200 OK", api_headers, r#"{"characters":[]}"#.to_string())
                };
                mockggg::respond_with(&mut stream, status, "application/json", headers, &body)
                    .await;
            }
        });
        (base, task)
    }

    fn read_journal(path: &std::path::Path) -> Vec<Value> {
        std::fs::read_to_string(path)
            .expect("journal was written")
            .lines()
            .map(|l| serde_json::from_str(l).expect("journal line is JSON"))
            .collect()
    }

    /// Seconds since midnight from a journal `ts`; enough to reason about a
    /// scenario that starts the day at the manual wall origin.
    fn ts_seconds(line: &Value) -> u64 {
        let ts = line["ts"].as_str().expect("ts is a string");
        assert!(ts.starts_with("2000-01-01T"), "manual-clock stamp: {ts}");
        let hms: Vec<u64> = ts[11..19].split(':').map(|p| p.parse().unwrap()).collect();
        hms[0] * 3600 + hms[1] * 60 + hms[2]
    }

    /// R8 as a scenario (TESTING-NOTES, "the experiment"). The token was
    /// issued for 3600 s; the lid closed for 1800 s and then 2000 s passed
    /// normally. On the wall it is 3800 s later and the token is dead; on a
    /// stopwatch only 2000 s have passed and it looks fine. The invariant is
    /// over the journal, not the sequence: no send is answered 401, the
    /// refresh happened on the wire, and the whole job finished within a
    /// minute of virtual time.
    ///
    /// Breaker (verified 2026-08-24): measure expiry from `self.choke.now()`
    /// instead of `wall()` in `install_tokens_locked` / `valid_access_token`
    /// and this fails on the 401 assertion.
    #[tokio::test]
    async fn expired_token_after_laptop_sleep_is_refreshed_before_any_send() {
        let (base, server) = bearer_aware_server("at-old", "at-new").await;
        let clock = Arc::new(ManualClock::new());
        let journal = std::env::temp_dir().join(format!(
            "acquisition-r8-{}-{}.jsonl",
            std::process::id(),
            TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_file(&journal);
        let rails = Arc::new(Rails::with_config_and_clock(
            RailsConfig {
                journal_path: Some(journal.clone()),
                ..RailsConfig::default()
            },
            clock.clone(),
        ));
        let (daemon, log_path) = test_daemon_scenario(
            Provider::mock(&base),
            clock.clone(),
            rails,
            Arc::new(NoopCredentialStore),
        );
        {
            let mut s = daemon.shared.lock().unwrap();
            // Key == username is the map's invariant; the scenario daemon
            // starts with one anonymous session.
            s.auth.rename("", "scenario-user");
            daemon.install_tokens_locked(
                s.auth.one_mut(),
                auth::TokenResponse {
                    access_token: "at-old".into(),
                    refresh_token: "rt-old".into(),
                    expires_in: 3600,
                    username: "scenario-user".into(),
                },
            );
        }
        clock.laptop_sleep(Duration::from_secs(1800));
        clock.advance(Duration::from_secs(2000));
        let scenario_start = 3800;

        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let id = daemon
            .submit("characters".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let (info, _) = wait_terminal(&daemon, id).await;
        server.abort();
        finish_harness(dispatcher, &log_path);

        let lines = read_journal(&journal);
        let _ = std::fs::remove_file(&journal);
        let header = &lines[0];
        assert_eq!(header["event"], "open");
        assert_eq!(header["clock"], "manual");
        assert_eq!(header["build"], crate::BUILD);
        assert_eq!(header["ts"], "2000-01-01T00:00:00.000Z");
        let sends = &lines[1..];
        assert_wire_contract(sends);
        assert_pacing_follows_responses(sends);

        assert!(
            sends.iter().all(|l| l["status"] != 401),
            "no send may be answered 401: {sends:?}"
        );
        assert_eq!(info.state, JobState::Done, "{sends:?}");
        assert!(
            sends
                .iter()
                .any(|l| l["method"] == "POST" && l["path"] == "/token"),
            "the refresh must reach the wire: {sends:?}"
        );
        for line in sends {
            let at = ts_seconds(line);
            assert!(
                (scenario_start..scenario_start + 60).contains(&at),
                "send outside the scenario's minute: {line}"
            );
        }
    }

    async fn wait_terminal(daemon: &Daemon, id: JobId) -> (JobInfo, Outcome) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if let Some(done) = {
                    let shared = daemon.shared.lock().unwrap();
                    shared.jobs.get(&id).and_then(|entry| {
                        entry
                            .info
                            .state
                            .is_terminal()
                            .then(|| (entry.info.clone(), entry.outcome.clone().unwrap()))
                    })
                } {
                    return done;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("dispatcher completed the scripted job")
    }

    fn fetch_payload_marker(outcome: &Outcome) -> &str {
        let Outcome::Success { payload } = outcome else {
            panic!("expected successful fetch, got {outcome:?}")
        };
        payload["items"][0].as_str().unwrap()
    }

    fn terminal_event_count(receiver: &mut broadcast::Receiver<JobInfo>, id: JobId) -> usize {
        let mut count = 0;
        while let Ok(info) = receiver.try_recv() {
            if info.id == id && info.state.is_terminal() {
                count += 1;
            }
        }
        count
    }

    fn finish_harness(dispatcher: tokio::task::JoinHandle<()>, log_path: &std::path::Path) {
        dispatcher.abort();
        remove_harness_files(log_path);
    }

    fn remove_harness_files(log_path: &std::path::Path) {
        let _ = std::fs::remove_file(log_path);
        let _ = std::fs::remove_file(journal_of(log_path));
    }

    /// The token servers record bodies, not methods; every one was a POST.
    fn wire_posts(requests: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
        vec!["POST".to_string(); requests.lock().unwrap().len()]
    }

    /// Journal == wire. The recorder is what the server received; the
    /// journal is what the daemon *claims* it sent. Every send must appear
    /// in both, in order, and a journal line that no server saw is as much
    /// a defect as a send the journal missed (R4 in LIVE-TESTING). Wire
    /// entries are compared by method, which is what the scripted servers
    /// record; the journal is read back here, before the harness deletes it.
    fn assert_journal_matches_wire(log_path: &std::path::Path, wire: &[String]) {
        let journal = journal_of(log_path);
        let lines = read_journal(&journal);
        let (header, sends) = lines.split_first().expect("journal has a header");
        assert_eq!(header["event"], "open", "first journal line is the header");
        assert_eq!(
            header["clock"], "manual",
            "harness daemons run on the test clock"
        );
        let journaled: Vec<&str> = sends
            .iter()
            .map(|l| l["method"].as_str().expect("journal method"))
            .collect();
        assert_eq!(
            journaled, wire,
            "journal (left) disagrees with what the server received (right)"
        );
    }

    /// `finish_harness` for tests that hold a wire recorder.
    fn finish_harness_wire(
        dispatcher: tokio::task::JoinHandle<()>,
        log_path: &std::path::Path,
        wire: &Arc<Mutex<Vec<String>>>,
    ) {
        assert_journal_matches_wire(log_path, &wire.lock().unwrap());
        let sends = journal_sends(log_path);
        assert_pacing_follows_responses(&sends);
        assert_wire_contract(&sends);
        finish_harness(dispatcher, log_path);
    }

    /// The journal's send lines, header dropped.
    fn journal_sends(log_path: &std::path::Path) -> Vec<Value> {
        let mut lines = read_journal(&journal_of(log_path));
        assert_eq!(
            lines.first().map(|l| l["event"].clone()),
            Some("open".into())
        );
        lines.remove(0);
        lines
    }

    /// `wait_ms` of every send, in journal order.
    fn journal_waits(log_path: &std::path::Path) -> Vec<u64> {
        journal_sends(log_path)
            .iter()
            .map(|l| l["wait_ms"].as_u64().expect("wait_ms"))
            .collect()
    }

    fn full_hold_ms() -> u64 {
        (crate::ratelimit::RETRY_BUCKET_PAD + crate::ratelimit::BUFFER).as_millis() as u64
    }

    /// The rest of the wire contract the register walk (TESTING-NOTES,
    /// experiment 3) found expressible over the journal alone and true of
    /// the product with rails off:
    ///
    /// - N16/N24, probe-before-send: within one daemon lifetime the first
    ///   send on any API route is a HEAD. The token endpoint is never
    ///   probed.
    /// - N24, accounting: a HEAD is never counted; everything else is.
    /// - N34/R8: a 401 is answered by a token refresh before any other
    ///   send. (No offline breaker yet: the harness has no scenario in
    ///   which a 401 lands and the refresh does not follow.)
    fn assert_wire_contract(sends: &[Value]) {
        let mut seen: HashSet<(u64, String)> = HashSet::new();
        let mut owe_refresh: Option<&Value> = None;
        for send in sends {
            let pid = send["pid"].as_u64().unwrap();
            let route = send["route"].as_str().unwrap().to_string();
            let method = send["method"].as_str().unwrap();
            if let Some(unauthorized) = owe_refresh.take() {
                assert!(
                    method == "POST" && route == "oauth-token",
                    "after a 401 the next send must be the refresh, not: {send}\n401: {unauthorized}"
                );
            }
            if route != "oauth-token" && seen.insert((pid, route.clone())) {
                assert_eq!(
                    method, "HEAD",
                    "first send on {route} in pid {pid} was not a probe: {send}"
                );
            }
            assert_eq!(
                send["counted"],
                method != "HEAD",
                "HEADs are not counted and everything else is: {send}"
            );
            if send["status"] == 401 {
                owe_refresh = Some(send);
            }
        }
    }

    /// The pacing invariant, derived from N19 rather than from the code: a
    /// send on a route is held only because the previous landed response on
    /// that route was a 429, and then for at least its `Retry-After` and at
    /// most `Retry-After + RETRY_BUCKET_PAD + BUFFER`, the largest hold the
    /// limiter is allowed to impose — or because it was counted and came
    /// back without rate headers (rung 10's origin 503), in which case the
    /// hit is assumed counted and the hold may reach the longest window
    /// period of the route's last known policy plus the same pad. Everything
    /// else goes out at once. Runs over every harness journal, so a rewrite
    /// that paces slower (or stops pacing) fails here without any test
    /// naming the numbers.
    fn assert_pacing_follows_responses(sends: &[Value]) {
        let pad = full_hold_ms();
        let mut last_on_route: HashMap<String, &Value> = HashMap::new();
        let mut longest_period_ms: HashMap<String, u64> = HashMap::new();
        let headerless = |send: &Value| {
            send["counted"] == true
                && send["status"].as_u64().is_some()
                && send["rate"]
                    .as_object()
                    .is_none_or(|rate| !rate.keys().any(|k| k.starts_with("x-rate-limit-")))
        };
        for send in sends {
            let route = send["route"].as_str().unwrap().to_string();
            let wait = send["wait_ms"].as_u64().expect("wait_ms");
            match last_on_route.get(&route) {
                Some(prev) if headerless(prev) => {
                    let ceiling = longest_period_ms.get(&route).copied().unwrap_or(0) + pad;
                    assert!(
                        wait <= ceiling,
                        "after a headerless counted response the next send on {route} \
                         waited {wait} ms; allowed at most {ceiling}: {send}"
                    );
                }
                Some(prev) if prev["status"] == 429 => {
                    let retry_after: u64 = prev["rate"]["retry-after"]
                        .as_str()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let floor = retry_after * 1000;
                    assert!(
                        (floor..=floor + pad).contains(&wait),
                        "after a 429 with Retry-After {retry_after} the next send on \
                         {route} waited {wait} ms; allowed {floor}..={}: {send}",
                        floor + pad
                    );
                }
                _ => assert_eq!(
                    wait, 0,
                    "nothing demanded a hold on {route}, yet the send waited: {send}"
                ),
            }
            // Window periods from every `X-Rate-Limit-<rule>` header seen on
            // the route: "hits:period:restriction" triplets, comma-separated.
            if let Some(rate) = send["rate"].as_object() {
                for (key, value) in rate {
                    if !key.starts_with("x-rate-limit-") || key.ends_with("-state") {
                        continue;
                    }
                    for triplet in value.as_str().unwrap_or("").split(',') {
                        if let Some(period) = triplet
                            .split(':')
                            .nth(1)
                            .and_then(|p| p.parse::<u64>().ok())
                        {
                            let slot = longest_period_ms.entry(route.clone()).or_default();
                            *slot = (*slot).max(period * 1000);
                        }
                    }
                }
            }
            last_on_route.insert(route, send);
        }
    }

    #[tokio::test]
    async fn dispatcher_auth_waits_do_not_cap_independent_job_progress() {
        let (base, arrived, release, server) = delayed_token_server().await;
        let clock = Arc::new(ManualClock::new());
        let (mut daemon, log_path) = test_daemon(&base, clock);
        Arc::get_mut(&mut daemon).unwrap().credential_store = Arc::new(NoopCredentialStore);
        {
            let mut shared = daemon.shared.lock().unwrap();
            shared.auth.one_mut().refresh_token = Some("rt-old".into());
            shared.auth.rename("", "old-user");
        }
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let first = daemon
            .submit("whoami".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let second = daemon
            .submit("whoami".into(), json!({}), 0, "test".into(), None)
            .unwrap();

        arrived.await.unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let waiters = daemon
                    .shared
                    .lock()
                    .unwrap()
                    .auth
                    .one()
                    .refresh_flight
                    .as_ref()
                    .map_or(0, |flight| flight.result.receiver_count());
                if waiters == 1 {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("second job joined the refresh owner");

        let independent = daemon
            .submit(
                "sleep".into(),
                json!({ "seconds": 0.0 }),
                0,
                "test".into(),
                None,
            )
            .unwrap();
        let (info, _) = wait_terminal(&daemon, independent).await;
        assert_eq!(info.state, JobState::Done);

        release.send(()).unwrap();
        assert_eq!(wait_terminal(&daemon, first).await.0.state, JobState::Done);
        assert_eq!(wait_terminal(&daemon, second).await.0.state, JobState::Done);
        server.await.unwrap();
        finish_harness(dispatcher, &log_path);
    }

    #[tokio::test]
    async fn dispatcher_rate_waits_do_not_cap_independent_job_progress() {
        let (base, server) = saturated_probe_server().await;
        let clock = Arc::new(BlockingClock::new());
        let (daemon, log_path) = test_daemon(&base, clock.clone());
        daemon
            .choke
            .head(
                "character-list",
                &format!("{base}/character"),
                None,
                daemon.choke.now(),
            )
            .await
            .unwrap();
        daemon
            .choke
            .head(
                "stash-list",
                &format!("{base}/stash/Standard"),
                None,
                daemon.choke.now(),
            )
            .await
            .unwrap();
        server.await.unwrap();

        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let characters = daemon
            .submit("characters".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let stashes = daemon
            .submit(
                "stashes".into(),
                json!({ "league": "Standard" }),
                0,
                "test".into(),
                None,
            )
            .unwrap();
        clock.wait_for_sleepers(2).await;

        let independent = daemon
            .submit(
                "sleep".into(),
                json!({ "seconds": 0.0 }),
                0,
                "test".into(),
                None,
            )
            .unwrap();
        let (info, _) = wait_terminal(&daemon, independent).await;
        assert_eq!(info.state, JobState::Done);

        // The dispatcher pre-waits in one-second cancellation slices.
        clock.release(20);
        assert_eq!(
            wait_terminal(&daemon, characters).await.0.state,
            JobState::Failed
        );
        assert_eq!(
            wait_terminal(&daemon, stashes).await.0.state,
            JobState::Failed
        );
        finish_harness(dispatcher, &log_path);
    }

    #[tokio::test]
    async fn n6_integration_stress_mixes_policies_refresh_rotation_and_cancellation_under_the_limit()
     {
        let clock = Arc::new(ManualClock::new());
        let api_base = mockggg::start_with_clock(clock.clone()).await.unwrap();
        let (token_base, token_arrived, release_token, token_server) = delayed_token_server().await;
        let (mut daemon, log_path) = test_daemon(&api_base, clock.clone());
        let credential_store = Arc::new(RecordingCredentialStore::default());
        let daemon_mut = Arc::get_mut(&mut daemon).unwrap();
        daemon_mut.credential_store = credential_store.clone();
        daemon_mut.provider.token_url = format!("{token_base}/token");
        {
            let mut shared = daemon.shared.lock().unwrap();
            shared.auth.one_mut().refresh_token = Some("rt-old".into());
            shared.auth.rename("", "old-user");
            shared.auth.one_mut().access_token = Some("at-established".into());
            shared.auth.one_mut().access_expires_at =
                Some(clock.wall() + Duration::from_secs(3600));
        }

        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        // Establish every authenticated route before expiry so the refresh
        // phase is not serialized behind the one-at-a-time probe key.
        let established_routes = [
            daemon
                .submit("characters".into(), json!({}), 0, "test".into(), None)
                .unwrap(),
            daemon
                .submit(
                    "stashes".into(),
                    json!({ "league": "Standard" }),
                    0,
                    "test".into(),
                    None,
                )
                .unwrap(),
            daemon
                .submit(
                    "stash".into(),
                    json!({ "league": "Standard", "id": "cur1" }),
                    0,
                    "test".into(),
                    None,
                )
                .unwrap(),
        ];
        for id in established_routes {
            let (info, outcome) = wait_terminal(&daemon, id).await;
            assert_eq!(
                info.state,
                JobState::Done,
                "route-establishment job {id}: {outcome:?}"
            );
        }
        {
            let mut shared = daemon.shared.lock().unwrap();
            // Expired on the daemon's clock, not the machine's.
            shared.auth.one_mut().access_expires_at = Some(clock.wall());
        }

        // These jobs now have different learned scheduling keys, so all three
        // can enter valid_access_token while the localhost token body is held.
        let characters = daemon
            .submit("characters".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let stashes = daemon
            .submit(
                "stashes".into(),
                json!({ "league": "Standard" }),
                0,
                "test".into(),
                None,
            )
            .unwrap();
        let stash = daemon
            .submit(
                "stash".into(),
                json!({ "league": "Standard", "id": "cur1" }),
                0,
                "test".into(),
                None,
            )
            .unwrap();
        let fetches: Vec<_> = (0..7)
            .map(|sequence| {
                daemon
                    .submit(
                        "fetch".into(),
                        json!({ "sequence": sequence }),
                        0,
                        "test".into(),
                        None,
                    )
                    .unwrap()
            })
            .collect();
        let cancelled = *fetches.last().unwrap();
        daemon.cancel(cancelled).unwrap();

        token_arrived.await.unwrap();
        wait_for_refresh_waiters(&daemon, 2).await;
        release_token.send(()).unwrap();

        for id in [characters, stashes, stash] {
            let (info, outcome) = wait_terminal(&daemon, id).await;
            assert_eq!(
                info.state,
                JobState::Done,
                "shared-refresh API caller {id}: {outcome:?}"
            );
        }
        token_server.await.unwrap();

        let mut done = 0;
        let mut failed = 0;
        let mut cancelled_count = 0;
        for id in fetches {
            let (info, outcome) = wait_terminal(&daemon, id).await;
            match info.state {
                JobState::Done => done += 1,
                JobState::Failed => {
                    failed += 1;
                    assert_eq!(info.retries, MAX_429_RETRIES);
                    let Outcome::Failure { error } = outcome else {
                        panic!("failed fetch {id} had non-failure outcome")
                    };
                    assert!(error.contains("giving up"));
                }
                JobState::Cancelled => cancelled_count += 1,
                state => panic!("fetch {id} stopped in nonterminal state {state}"),
            }
        }
        // The counting mock makes N4 non-vacuous: every line's state is the
        // mock's own count, so "never over the limit" pins the product.
        // Checked before the job counts so a violation reports as itself.
        let sends = journal_sends(&log_path);
        assert_wire_contract(&sends);
        assert_never_over_the_limit(&sends);
        let fetch_waits: Vec<u64> = sends
            .iter()
            .filter(|s| s["route"] == "fetch" && s["method"] == "GET")
            .map(|s| s["wait_ms"].as_u64().unwrap())
            .collect();
        assert!(
            fetch_waits.len() == 6
                && fetch_waits[..5].iter().all(|&w| w == 0)
                && fetch_waits[5] > 0,
            "five fetches fill the 5-per-10 s window at once and the sixth is held: {fetch_waits:?}"
        );
        // Before 2026-08-24 this read (5, 1, 1): the mock's windows expired
        // in real time while the daemon's expired in virtual time, so the
        // sixth fetch's retries always met a still-restricted mock. On one
        // clock the limiter holds before the sixth send and nothing
        // violates (N4). 429 recovery is pinned by the scripted tests.
        assert_eq!((done, failed, cancelled_count), (6, 0, 1));

        let (refresh_token, access_token, refresh_flight) = {
            let shared = daemon.shared.lock().unwrap();
            (
                shared.auth.one().refresh_token.clone().unwrap(),
                shared.auth.one().access_token.clone().unwrap(),
                shared.auth.one().refresh_flight.is_some(),
            )
        };
        assert_eq!(refresh_token, "rt-rotated");
        assert_eq!(access_token, "at-new");
        assert!(!refresh_flight);
        let saves = credential_store.saves.lock().unwrap();
        assert_eq!(saves.len(), 1, "rotated refresh token persisted once");
        assert_eq!(saves[0].0, "acquisition-playground");
        assert_eq!(saves[0].1, "rt-rotated");
        assert_eq!(saves[0].2, "test-user");
        drop(saves);

        let jobs = daemon.shared.lock().unwrap();
        let probes: Vec<_> = jobs
            .jobs
            .values()
            .filter(|entry| entry.info.kind == "probe")
            .collect();
        assert_eq!(probes.len(), 4, "one probe per exercised API route");
        assert!(
            probes
                .iter()
                .all(|entry| entry.info.state == JobState::Done)
        );
        drop(jobs);

        let sends = daemon.choke.recent_sends();
        assert_eq!(sends.iter().filter(|send| send.method == "HEAD").count(), 4);
        assert_eq!(
            sends
                .iter()
                .filter(|send| send.method == "POST" && send.endpoint == "oauth-token")
                .count(),
            1
        );
        assert_eq!(
            sends
                .iter()
                .filter(|send| send.method == "GET" && send.endpoint == "fetch")
                .count(),
            6,
            "six fetches, none retried"
        );
        assert_eq!(daemon.choke.actual_send_occupancy(), (0, 2));

        let mut policies: Vec<_> = daemon
            .choke
            .policy_statuses()
            .into_iter()
            .map(|status| status.policy)
            .collect();
        policies.sort();
        assert_eq!(
            policies,
            [
                "character-list-request-limit",
                "mock-fetch-request-limit",
                "stash-list-request-limit",
                "stash-request-limit",
                "token-request-limit",
            ]
        );

        finish_harness(dispatcher, &log_path);
    }

    /// N4/N25 over the journal: no send is answered 429, and every window
    /// state a response reports is within its limit with no restriction
    /// active. Against the scripted server this is vacuous (it echoes what
    /// the script says); against `mockggg` on the test clock the state is
    /// the mock's own count, so a limiter that sends one too many fails
    /// here. Rule names come from `X-Rate-Limit-Rules`, so this reads any
    /// rule set, not only `Account`.
    fn assert_never_over_the_limit(sends: &[Value]) {
        for send in sends {
            assert_ne!(send["status"], 429, "a send was answered 429: {send}");
            let rate = &send["rate"];
            let Some(rules) = rate["x-rate-limit-rules"].as_str() else {
                continue;
            };
            for rule in rules.split(',').map(|r| r.trim().to_ascii_lowercase()) {
                let limits = rate[format!("x-rate-limit-{rule}")]
                    .as_str()
                    .unwrap_or_else(|| panic!("limits for rule {rule}: {send}"));
                let states = rate[format!("x-rate-limit-{rule}-state")]
                    .as_str()
                    .unwrap_or_else(|| panic!("state for rule {rule}: {send}"));
                for (limit, state) in limits.split(',').zip(states.split(',')) {
                    let max: u64 = limit.split(':').next().unwrap().parse().unwrap();
                    let mut parts = state.split(':');
                    let hits: u64 = parts.next().unwrap().parse().unwrap();
                    let _period = parts.next();
                    let restricted: u64 = parts.next().unwrap().parse().unwrap();
                    assert!(
                        hits <= max,
                        "{rule} window {limit} reported {state}: over the limit: {send}"
                    );
                    assert_eq!(
                        restricted, 0,
                        "{rule} window {limit} reported an active restriction: {send}"
                    );
                }
            }
        }
    }

    #[test]
    fn dispatcher_preserves_priority_within_a_scheduling_key() {
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon("http://127.0.0.1:1", clock);
        let low = daemon
            .submit("fetch".into(), json!({}), 1, "test".into(), None)
            .unwrap();
        let high = daemon
            .submit("fetch".into(), json!({}), 9, "test".into(), None)
            .unwrap();

        assert_eq!(daemon.pick_runnable(), vec![high]);
        assert_eq!(
            daemon
                .shared
                .lock()
                .unwrap()
                .active_jobs
                .get(&high)
                .map(String::as_str),
            Some("fetch")
        );
        assert!(!daemon.shared.lock().unwrap().active_jobs.contains_key(&low));
        let _ = std::fs::remove_file(log_path);
    }

    /// Rung 10 (2026-08-24) with the tripwire off: a 503 with no rate
    /// headers fails its job and is never retried, and the *next* job's send
    /// is paced as if the 503 counted — visible as `wait_ms` in the journal.
    #[tokio::test]
    async fn headerless_503_fails_its_job_and_paces_the_next_send() {
        let policy = "X-Rate-Limit-Policy: dispatcher-test-policy\r\nX-Rate-Limit-Rules: Account\r\nX-Rate-Limit-Account: 1:10:60\r\nX-Rate-Limit-Account-State: 0:10:0\r\n";
        let responses = vec![
            ScriptedResponse {
                method: "HEAD",
                path: "/fetch",
                status: "204 No Content",
                headers: policy.into(),
                body: String::new(),
            },
            ScriptedResponse {
                method: "GET",
                path: "/fetch",
                status: "503 Service Unavailable",
                headers: String::new(),
                body: "<html><center>openresty</center></html>".into(),
            },
            ScriptedResponse {
                method: "GET",
                path: "/fetch",
                status: "200 OK",
                headers: policy.replace("0:10:0", "1:10:0"),
                body: r#"{"items":["done"]}"#.into(),
            },
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon(&base, clock);
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let first = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let second = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();

        let (info, outcome) = wait_terminal(&daemon, first).await;
        assert_eq!(info.state, JobState::Failed);
        assert_eq!(info.retries, 0);
        let Outcome::Failure { error } = outcome else {
            panic!("503 did not fail")
        };
        assert!(error.contains("origin error page"), "{error}");
        let (info, outcome) = wait_terminal(&daemon, second).await;
        assert_eq!(info.state, JobState::Done);
        assert_eq!(fetch_payload_marker(&outcome), "done");
        assert_eq!(requests.lock().unwrap().as_slice(), ["HEAD", "GET", "GET"]);
        let waits = journal_waits(&log_path);
        assert_eq!(waits[..2], [0, 0]);
        // 10 s period + the 5 s initial-window bucket + 1 s buffer, measured
        // from the 503 itself — the only known in-window hit.
        assert_eq!(
            waits[2], 16_000,
            "the send after a headerless 503 waits as if the 503 filled the window: {waits:?}"
        );
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
    }

    /// Malformed listings fail the *job*, not just the store: a 2xx with
    /// no `stashes` array is refused by the job's own guard, and a listing
    /// whose entries lack ids is refused by the store (`MalformedBody`)
    /// and propagated through `record` — neither reports success, and
    /// neither leaves a response row a snapshot could cite as a basis.
    #[tokio::test]
    async fn malformed_listings_fail_the_job_not_just_the_store() {
        let policy = "X-Rate-Limit-Policy: stash-list-test\r\nX-Rate-Limit-Rules: Account\r\nX-Rate-Limit-Account: 100:1:60\r\nX-Rate-Limit-Account-State: 0:1:0\r\n";
        let responses = vec![
            ScriptedResponse {
                method: "HEAD",
                path: "/stash/Standard",
                status: "204 No Content",
                headers: policy.into(),
                body: String::new(),
            },
            ScriptedResponse {
                method: "GET",
                path: "/stash/Standard",
                status: "200 OK",
                headers: policy.replace("0:1:0", "1:1:0"),
                body: r#"{"error":"maintenance"}"#.into(),
            },
            ScriptedResponse {
                method: "GET",
                path: "/stash/Standard",
                status: "200 OK",
                headers: policy.replace("0:1:0", "2:1:0"),
                body: r#"{"stashes":[{"name":"NoId","type":"PremiumStash"}]}"#.into(),
            },
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (mut daemon, log_path) = test_daemon(&base, clock);
        let store_dir = std::env::temp_dir().join(format!(
            "acq-malformed-job-{}-{}",
            std::process::id(),
            TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&store_dir);
        Arc::get_mut(&mut daemon).expect("fresh daemon").store_dir = Some(store_dir.clone());
        {
            let mut s = daemon.shared.lock().unwrap();
            s.auth.rename("", "Alice#1234");
            let session = s.auth.one_mut();
            session.username = Some("Alice#1234".into());
            session.access_token = Some("at-test.Alice#1234".into());
            session.access_expires_at = Some(daemon.choke.wall() + Duration::from_secs(3600));
        }
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let account = Some("Alice#1234".to_string());
        let first = daemon
            .submit(
                "stashes".into(),
                json!({}),
                0,
                "test".into(),
                account.clone(),
            )
            .unwrap();
        let (info, outcome) = wait_terminal(&daemon, first).await;
        assert_eq!(info.state, JobState::Failed);
        let Outcome::Failure { error } = outcome else {
            panic!("missing array did not fail the job")
        };
        assert!(error.contains("stashes"), "{error}");
        let second = daemon
            .submit("stashes".into(), json!({}), 0, "test".into(), account)
            .unwrap();
        let (info, outcome) = wait_terminal(&daemon, second).await;
        assert_eq!(info.state, JobState::Failed);
        let Outcome::Failure { error } = outcome else {
            panic!("id-less entry did not fail the job")
        };
        assert!(error.contains("malformed stashes response"), "{error}");
        // Nothing recorded either way: no tabs retired, no false basis.
        let store = Store::open(&account_path(&store_dir, "Alice#1234")).unwrap();
        assert_eq!(store.status().unwrap().responses, 0);
        drop(store);
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
        let _ = std::fs::remove_dir_all(&store_dir);
    }

    /// The apply parent (tracer step 7): a plan's action set executed
    /// exactly — one child per admitted tuple, no send of its own, no
    /// expansion — with the children's responses recorded through the
    /// store like any other fetch.
    #[tokio::test]
    async fn apply_fans_out_exactly_its_tuples_and_records_through_the_store() {
        let base = mockggg::start().await.unwrap();
        let clock = Arc::new(ManualClock::new());
        let (mut daemon, log_path) = test_daemon(&base, clock);
        let store_dir = std::env::temp_dir().join(format!(
            "acq-apply-{}-{}",
            std::process::id(),
            TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&store_dir);
        Arc::get_mut(&mut daemon).expect("fresh daemon").store_dir = Some(store_dir.clone());
        {
            let mut s = daemon.shared.lock().unwrap();
            s.auth.rename("", "Alice#1234");
            let session = s.auth.one_mut();
            session.username = Some("Alice#1234".into());
            session.access_token = Some("at-test.Alice#1234".into());
            session.access_expires_at = Some(daemon.choke.wall() + Duration::from_secs(3600));
        }
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let id = daemon
            .submit(
                "apply".into(),
                json!({ "jobs": [
                    { "kind": "stashes", "params": { "league": "Standard" } },
                    { "kind": "stash", "params": { "league": "Standard", "id": "dump", "deep": false } },
                ] }),
                0,
                "test".into(),
                Some("Alice#1234".into()),
            )
            .unwrap();
        let (info, outcome) = wait_terminal(&daemon, id).await;
        let Outcome::Success { payload } = outcome else {
            panic!("apply failed: {outcome:?}")
        };
        assert_eq!(info.state, JobState::Done);
        assert_eq!(payload["requests"], json!(2));
        assert_eq!(payload["child_jobs"].as_array().unwrap().len(), 2);
        assert_eq!(payload["children"]["done"], json!(2));
        // Exactly the tuples became children — same kinds, same params,
        // same order (ids ascend in submission order), nothing more (the
        // probes the routes needed are daemon jobs, not children of the
        // apply) — and the payload's child_jobs echoes them in that order.
        let children: Vec<(JobId, String, Value, Option<String>)> = {
            let s = daemon.shared.lock().unwrap();
            let mut c: Vec<_> = s
                .jobs
                .values()
                .filter(|e| e.info.parent == Some(id))
                .map(|e| {
                    (
                        e.info.id,
                        e.info.kind.clone(),
                        e.params.clone(),
                        e.info.account.clone(),
                    )
                })
                .collect();
            c.sort_by_key(|(id, ..)| *id);
            c
        };
        let tuples: Vec<(&str, &Value)> = children
            .iter()
            .map(|(_, kind, params, _)| (kind.as_str(), params))
            .collect();
        assert_eq!(
            tuples,
            [
                ("stashes", &json!({ "league": "Standard" })),
                (
                    "stash",
                    &json!({ "league": "Standard", "id": "dump", "deep": false })
                ),
            ],
            "children must be the admitted tuples verbatim, in order"
        );
        let child_ids: Vec<JobId> = children.iter().map(|(id, ..)| *id).collect();
        assert_eq!(payload["child_jobs"], json!(child_ids));
        // Children run as the parent's account, and their responses landed
        // in that account's store file.
        assert!(
            children
                .iter()
                .all(|(_, _, _, a)| a.as_deref() == Some("Alice#1234"))
        );
        let store = Store::open(&account_path(&store_dir, "Alice#1234")).unwrap();
        assert_eq!(store.status().unwrap().responses, 2);
        drop(store);
        finish_harness(dispatcher, &log_path);
        let _ = std::fs::remove_dir_all(&store_dir);
    }

    /// Apply's admission (CONTEXT.md, decided 2026-09-01): vocabulary and
    /// budget are checked at submit, before a job id exists, so a refusal
    /// admits nothing — never a partial fan-out, and ids never advance.
    #[tokio::test]
    async fn apply_admission_refuses_bad_vocabulary_and_blown_budgets_whole() {
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon("http://127.0.0.1:9", clock);
        // The whole vocabulary (characters joined 2026-09-02): a character
        // list under poe2 is a legal tuple — the character family takes it.
        let ok_jobs = json!([
            { "kind": "stashes", "params": { "league": "Standard" } },
            { "kind": "stash", "params": { "league": "Standard", "id": "t1", "deep": false } },
            { "kind": "characters", "params": { "realm": "poe2" } },
            { "kind": "character", "params": { "realm": "pc", "name": "Exile" } },
        ]);
        let refusals: Vec<(Value, &str)> = vec![
            // The tuple list itself is required and non-empty.
            (json!({}), "needs a `jobs` array"),
            (json!({ "jobs": [] }), "empty"),
            // Only single-request kinds: a nested parent (refresh, apply)
            // or any other kind would expand or sidestep the reviewed set.
            (
                json!({ "jobs": [{ "kind": "refresh", "params": { "all": true } }] }),
                "not in the plan vocabulary",
            ),
            (
                json!({ "jobs": [{ "kind": "apply", "params": { "jobs": [] } }] }),
                "not in the plan vocabulary",
            ),
            (
                json!({ "jobs": [{ "kind": "sleep", "params": {} }] }),
                "not in the plan vocabulary",
            ),
            // A stash fetch that fans out expands the set; one without an
            // id could not have come from a plan.
            (
                json!({ "jobs": [{ "kind": "stash", "params": { "league": "Standard", "id": "m1", "deep": true } }] }),
                "never fans out",
            ),
            (
                json!({ "jobs": [{ "kind": "stash", "params": { "league": "Standard" } }] }),
                "needs an id",
            ),
            // A character fetch without a name could not have come from a
            // plan either.
            (
                json!({ "jobs": [{ "kind": "character", "params": { "realm": "pc" } }] }),
                "needs a name",
            ),
            // The budget: a bound under the tuple count refuses; a misread
            // bound refuses too, never "the limit was ignored".
            (
                json!({ "jobs": ok_jobs.clone(), "max_requests": 1 }),
                "exceeds the budget",
            ),
            (
                json!({ "jobs": ok_jobs.clone(), "max_requests": "ten" }),
                "non-negative integer",
            ),
        ];
        for (params, expected) in refusals {
            let err = daemon
                .submit("apply".into(), params.clone(), 0, "test".into(), None)
                .unwrap_err();
            assert!(err.contains(expected), "{params}: {err}");
            assert!(
                daemon.shared.lock().unwrap().jobs.is_empty(),
                "{params}: a refused apply must admit nothing"
            );
        }
        // A budget the plan fits inside admits (no dispatcher runs here,
        // so the job just sits waiting) — and it gets the daemon's first
        // id, proving the refusals above consumed none.
        let id = daemon
            .submit(
                "apply".into(),
                json!({ "jobs": ok_jobs, "max_requests": 4 }),
                0,
                "test".into(),
                None,
            )
            .unwrap();
        assert_eq!(id, 1, "refused applies must not advance job ids");
        assert_eq!(
            daemon.shared.lock().unwrap().jobs[&id].info.state,
            JobState::Waiting
        );
        remove_harness_files(&log_path);
    }

    /// The route knowledge declared for `/profile` and `/account/leagues`
    /// (first contact 2026-08-30), held against the mock that mirrors GGG:
    /// neither is probed; `/profile`'s headerless 200 is accepted and the
    /// endpoint becomes Policyless; `/account/leagues` teaches its policy
    /// from the GET. Removing a declaration fails here, not live.
    #[tokio::test]
    async fn declared_route_knowledge_holds_against_the_mock() {
        let base = mockggg::start().await.unwrap();
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon(&base, clock);
        {
            let mut s = daemon.shared.lock().unwrap();
            s.auth.rename("", "Alice#1234");
            let session = s.auth.one_mut();
            session.username = Some("Alice#1234".into());
            session.access_token = Some("at-test.Alice#1234".into());
            session.access_expires_at = Some(daemon.choke.wall() + Duration::from_secs(3600));
        }
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        // Resolution happens in the `Submit` handler; here the account is
        // passed as that handler would after resolving the sole session.
        let account = Some("Alice#1234".to_string());
        let profile = daemon
            .submit(
                "profile".into(),
                json!({}),
                0,
                "test".into(),
                account.clone(),
            )
            .unwrap();
        let (info, outcome) = wait_terminal(&daemon, profile).await;
        assert_eq!(info.state, JobState::Done, "{outcome:?}");
        let leagues = daemon
            .submit("leagues".into(), json!({}), 0, "test".into(), account)
            .unwrap();
        let (info, outcome) = wait_terminal(&daemon, leagues).await;
        assert_eq!(info.state, JobState::Done, "{outcome:?}");

        let sends: Vec<(String, String)> = daemon
            .choke
            .recent_sends()
            .into_iter()
            .map(|send| (url_path(&send.url), send.method))
            .collect();
        assert!(
            !sends.iter().any(|(_, method)| method == "HEAD"),
            "no-probe routes must not be probed: {sends:?}"
        );
        assert!(
            sends.contains(&("/profile".into(), "GET".into())),
            "{sends:?}"
        );
        assert!(
            sends.contains(&("/account/leagues".into(), "GET".into())),
            "{sends:?}"
        );
        assert_eq!(
            daemon.choke.endpoint_state("profile@Alice#1234"),
            EndpointState::Policyless
        );
        assert!(matches!(
            daemon.choke.endpoint_state("league@Alice#1234"),
            EndpointState::Policy(ref name) if name.starts_with("league-request-limit")
        ));
        dispatcher.abort();
        let _ = std::fs::remove_file(&log_path);
    }

    // ---- tracer step 2: uuid-at-login (CONTEXT.md, identity decision) ----

    fn login_tokens(access: &str, user: &str) -> auth::TokenResponse {
        auth::TokenResponse {
            access_token: access.into(),
            refresh_token: format!("rt-{user}"),
            expires_in: 3600,
            username: user.into(),
        }
    }

    /// A daemon wired for login tests: a recording credential store, a
    /// scratch store directory, and no pre-existing session.
    fn login_daemon(base: &str) -> (Arc<Daemon>, Arc<RecordingCredentialStore>, PathBuf, PathBuf) {
        let clock = Arc::new(ManualClock::new());
        let rails = Arc::new(Rails::with_config_and_clock(
            RailsConfig::default(),
            clock.clone(),
        ));
        let credentials = Arc::new(RecordingCredentialStore::default());
        let (mut daemon, log_path) = test_daemon_scenario(
            Provider::mock(base),
            clock,
            rails,
            credentials.clone() as Arc<dyn CredentialStore>,
        );
        let store_dir = std::env::temp_dir().join(format!(
            "acq-login-{}-{}",
            std::process::id(),
            TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&store_dir);
        Arc::get_mut(&mut daemon).expect("fresh daemon").store_dir = Some(store_dir.clone());
        // The harness pre-installs an empty legacy session; a login test
        // starts logged out.
        daemon.shared.lock().unwrap().auth.by_account.clear();
        (daemon, credentials, store_dir, log_path)
    }

    async fn run_login(daemon: &Arc<Daemon>, access: &str, user: &str) {
        let generation = daemon.begin_auth_flow();
        let username = daemon
            .stage_auth_flow(generation, login_tokens(access, user))
            .expect("current flow stages");
        daemon.login_with_profile(generation, username).await;
    }

    fn login_result(daemon: &Daemon) -> (bool, Option<String>, Option<String>) {
        match daemon.auth_status() {
            Response::Auth {
                logged_in,
                login_ok,
                login_error,
                ..
            } => (logged_in, login_ok, login_error),
            other => panic!("unexpected response: {other:?}"),
        }
    }

    /// A login is not a login until the profile fetch delivers the account
    /// uuid: until then nothing is registered — no session, no keyring
    /// entry, no index row — and when it lands, everything is, with the
    /// uuid (deterministic per username on the mock).
    #[tokio::test]
    async fn login_registers_the_session_only_when_the_profile_uuid_lands() {
        let base = mockggg::start().await.unwrap();
        let (daemon, credentials, store_dir, log_path) = login_daemon(&base);
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        let generation = daemon.begin_auth_flow();
        let username = daemon
            .stage_auth_flow(generation, login_tokens("at-x.Alice#1234", "Alice#1234"))
            .expect("current flow stages");
        // Staged: the flow is still pending and nothing is registered —
        // the tokens sit in the staging slot, outside the session map.
        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.pending, Some(generation));
            assert!(s.auth.by_account.is_empty(), "no session exists yet");
            assert!(s.auth.staging.is_some());
        }
        assert!(matches!(
            daemon.auth_status(),
            Response::Auth {
                logged_in: false,
                pending: true,
                login_ok: None,
                login_error: None,
                ..
            }
        ));
        assert!(credentials.saves.lock().unwrap().is_empty());
        assert!(!acquisition_store::index_path(&store_dir).exists());

        daemon.login_with_profile(generation, username).await;

        // Registered: session visible, keyring written, index row with
        // uuid, and the flow's own terminal result names the account.
        let index = Index::load(&store_dir).unwrap();
        let entry = index.get("Alice#1234").expect("indexed at login");
        let uuid = entry.uuid.clone().expect("uuid recorded at login");
        assert!(entry.persisted);
        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.pending, None);
            assert!(s.auth.staging.is_none());
            assert_eq!(s.auth.last_login.as_deref(), Some("Alice#1234"));
            let session = s.auth.find("Alice#1234").expect("registered");
            assert_eq!(session.keyring, "ok");
            assert_eq!(session.uuid.as_deref(), Some(uuid.as_str()));
        }
        assert_eq!(credentials.saves.lock().unwrap().len(), 1);
        assert_eq!(
            login_result(&daemon),
            (true, Some("Alice#1234".into()), None)
        );

        // A re-login as the same account lands the same uuid: the mock's
        // uuids are deterministic per username, and the index keeps one
        // entry per account.
        run_login(&daemon, "at-y.Alice#1234", "Alice#1234").await;
        let index = Index::load(&store_dir).unwrap();
        assert_eq!(index.entries().len(), 1);
        assert_eq!(
            index.get("Alice#1234").unwrap().uuid.as_deref(),
            Some(uuid.as_str())
        );

        dispatcher.abort();
        let _ = std::fs::remove_dir_all(&store_dir);
        let _ = std::fs::remove_file(&log_path);
    }

    /// A login whose profile fetch fails fails whole: no provisional
    /// identity — the staged tokens are dropped, no keyring entry is
    /// minted, the index is untouched, and the flow's terminal result is
    /// the failure.
    #[tokio::test]
    async fn a_login_whose_profile_fetch_fails_fails_whole() {
        // Nothing listens here: the profile GET dies in transport.
        let (daemon, credentials, store_dir, log_path) = login_daemon("http://127.0.0.1:1");
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        run_login(&daemon, "at-x.Alice#1234", "Alice#1234").await;

        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.pending, None, "the flow is closed");
            assert!(s.auth.by_account.is_empty(), "no session appeared");
            assert!(s.auth.staging.is_none(), "the staged tokens are gone");
            assert!(
                s.errors
                    .iter()
                    .any(|(_, m)| m.contains("login failed for Alice#1234")),
                "the failure is reported"
            );
        }
        let (logged_in, ok, error) = login_result(&daemon);
        assert!(!logged_in && ok.is_none());
        assert!(
            error
                .as_deref()
                .is_some_and(|e| e.contains("profile fetch")),
            "{error:?}"
        );
        assert!(credentials.saves.lock().unwrap().is_empty());
        assert!(!acquisition_store::index_path(&store_dir).exists());

        dispatcher.abort();
        let _ = std::fs::remove_dir_all(&store_dir);
        let _ = std::fs::remove_file(&log_path);
    }

    /// A failed *re-login* leaves the previously registered session exactly
    /// as it was (staging never touches the session map), and the flow's
    /// terminal result reports the failure even though `logged_in` stays
    /// true — so `acq auth` cannot mistake the surviving session for this
    /// login succeeding.
    #[tokio::test]
    async fn a_failed_relogin_keeps_the_old_session_and_reports_the_failure() {
        let base = mockggg::start().await.unwrap();
        let (daemon, credentials, store_dir, log_path) = login_daemon(&base);
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        run_login(&daemon, "at-x.Alice#1234", "Alice#1234").await;
        assert_eq!(login_result(&daemon).1.as_deref(), Some("Alice#1234"));
        let old_token = {
            let s = daemon.shared.lock().unwrap();
            s.auth.find("Alice#1234").unwrap().access_token.clone()
        };

        // A re-login whose profile fetch fails (simulated at the abort
        // step; the state machine is the same for every failure cause).
        let g2 = daemon.begin_auth_flow();
        daemon
            .stage_auth_flow(g2, login_tokens("at-y.Alice#1234", "Alice#1234"))
            .expect("current flow stages");
        daemon.abort_login(g2, "Alice#1234", "profile fetch failed: simulated");

        {
            let s = daemon.shared.lock().unwrap();
            let session = s.auth.find("Alice#1234").expect("old session survives");
            assert_eq!(
                session.access_token, old_token,
                "the old session is untouched — the staged tokens never reached it"
            );
            assert!(s.auth.staging.is_none());
        }
        let (logged_in, ok, error) = login_result(&daemon);
        assert!(logged_in, "the old session is still live");
        assert!(ok.is_none(), "the failed flow claims no success");
        assert!(error.as_deref().is_some_and(|e| e.contains("simulated")));
        // The keyring still holds exactly the first login's entry.
        assert_eq!(credentials.saves.lock().unwrap().len(), 1);

        dispatcher.abort();
        let _ = std::fs::remove_dir_all(&store_dir);
        let _ = std::fs::remove_file(&log_path);
    }

    /// A login the account index cannot record fails whole: the session
    /// never becomes visible and the keyring entry is rolled back — a
    /// session without its uuid mapping could neither be restored nor find
    /// its annotation file.
    #[tokio::test]
    async fn a_login_the_index_cannot_record_fails_whole_and_rolls_back_the_keyring() {
        let base = mockggg::start().await.unwrap();
        let (mut daemon, credentials, store_dir, log_path) = login_daemon(&base);
        // A regular file where the store directory should be: every index
        // read and write fails.
        let blocker = store_dir.with_extension("blocker");
        std::fs::write(&blocker, "not a directory").unwrap();
        Arc::get_mut(&mut daemon).expect("fresh daemon").store_dir = Some(blocker.clone());
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        run_login(&daemon, "at-x.Alice#1234", "Alice#1234").await;

        {
            let s = daemon.shared.lock().unwrap();
            assert!(s.auth.by_account.is_empty(), "the session never appeared");
            assert!(s.auth.staging.is_none());
        }
        let (logged_in, ok, error) = login_result(&daemon);
        assert!(!logged_in && ok.is_none());
        assert!(
            error
                .as_deref()
                .is_some_and(|e| e.contains("account index")),
            "{error:?}"
        );
        // The keyring write happened and was rolled back.
        assert_eq!(credentials.saves.lock().unwrap().len(), 1);
        assert_eq!(
            credentials.cleared.lock().unwrap().as_slice(),
            ["Alice#1234"]
        );

        dispatcher.abort();
        let _ = std::fs::remove_file(&blocker);
        let _ = std::fs::remove_dir_all(&store_dir);
        let _ = std::fs::remove_file(&log_path);
    }

    /// A superseded flow can neither stage nor complete, and its staged
    /// tokens do not linger.
    #[tokio::test]
    async fn a_superseded_login_cannot_complete_and_leaves_no_zombie() {
        let (daemon, credentials, _store_dir, log_path) = login_daemon("http://127.0.0.1:1");

        let g1 = daemon.begin_auth_flow();
        daemon
            .stage_auth_flow(g1, login_tokens("at-x.Alice#1234", "Alice#1234"))
            .expect("current flow stages");
        // The user restarted the login before the profile landed: the new
        // flow owns the login state from here.
        let g2 = daemon.begin_auth_flow();
        assert!(!daemon.complete_login(g1, "Alice#1234", "u-stale"));
        {
            let s = daemon.shared.lock().unwrap();
            assert!(s.auth.by_account.is_empty(), "no session appeared");
            assert!(s.auth.staging.is_none(), "no staged tokens linger");
            assert_eq!(s.auth.pending, Some(g2), "the new flow is untouched");
            assert!(
                s.auth.flow_result.is_none(),
                "the new flow's result is its own"
            );
        }
        assert!(credentials.saves.lock().unwrap().is_empty());
        // A stale token exchange cannot stage either.
        assert!(
            daemon
                .stage_auth_flow(g1, login_tokens("at-y.Alice#1234", "Alice#1234"))
                .is_none()
        );
        let _ = std::fs::remove_file(&log_path);
    }

    #[tokio::test]
    async fn dispatcher_retries_429_429_success_exactly_three_times_and_completes_once() {
        let responses = vec![
            ScriptedResponse::full("HEAD", "204 No Content", None, ""),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(0), "{}"),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(5), "{}"),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"items":["done"]}"#),
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon(&base, clock);
        let mut events = daemon.events.subscribe();
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let id = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();

        let (info, outcome) = wait_terminal(&daemon, id).await;
        assert_eq!(info.state, JobState::Done);
        assert_eq!(info.retries, MAX_429_RETRIES);
        assert_eq!(fetch_payload_marker(&outcome), "done");
        assert_eq!(
            requests.lock().unwrap().as_slice(),
            ["HEAD", "GET", "GET", "GET"]
        );
        assert_eq!(
            journal_waits(&log_path),
            [0, 0, full_hold_ms(), 5_000 + full_hold_ms()],
            "each retry waits its Retry-After plus the bucket pad"
        );
        assert_eq!(terminal_event_count(&mut events, id), 1);
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
    }

    #[tokio::test]
    async fn dispatcher_exhausts_on_third_429_without_a_fourth_send_or_final_sleep() {
        let responses = vec![
            ScriptedResponse::full("HEAD", "204 No Content", None, ""),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(0), "{}"),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(0), "{}"),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(0), "{}"),
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon(&base, clock.clone());
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let id = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();

        let (info, outcome) = wait_terminal(&daemon, id).await;
        assert_eq!(info.state, JobState::Failed);
        let Outcome::Failure { error } = outcome else {
            panic!("exhausted job did not fail")
        };
        assert!(error.contains("giving up"));
        assert_eq!(requests.lock().unwrap().len(), 4, "HEAD plus three GETs");
        assert_eq!(
            journal_waits(&log_path),
            [0, 0, full_hold_ms(), full_hold_ms()],
            "only the two retryable attempts wait, each behind the full hold"
        );
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
    }

    #[tokio::test]
    async fn dispatcher_requeue_preserves_fifo_job_identity() {
        let responses = vec![
            ScriptedResponse::full("HEAD", "204 No Content", None, ""),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(0), "{}"),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"items":["first"]}"#),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"items":["second"]}"#),
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon(&base, clock);
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let first = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let second = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();

        let (_, first_outcome) = wait_terminal(&daemon, first).await;
        let (_, second_outcome) = wait_terminal(&daemon, second).await;
        assert_eq!(fetch_payload_marker(&first_outcome), "first");
        assert_eq!(fetch_payload_marker(&second_outcome), "second");
        assert_eq!(requests.lock().unwrap().len(), 4);
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
    }

    #[tokio::test]
    async fn dispatcher_never_retries_403_or_503() {
        // The third case is rung 10's live 503 (2026-08-24): an origin page
        // with no rate headers. Named as such, still never retried.
        const ORIGIN_503: &str = "<html><head><title>503 Service Temporarily Unavailable</title></head><body><center><h1>503 Service Temporarily Unavailable</h1></center><hr><center>openresty</center></body></html>";
        for (status, body, shape) in [
            (
                "403 Forbidden",
                "{}",
                "unclassified body, possibly a Cloudflare block",
            ),
            (
                "503 Service Unavailable",
                "{}",
                "unclassified body, possibly a Cloudflare block",
            ),
            (
                "503 Service Unavailable",
                ORIGIN_503,
                "origin error page, not Cloudflare-shaped",
            ),
        ] {
            let responses = vec![
                ScriptedResponse::full("HEAD", "204 No Content", None, ""),
                ScriptedResponse::full("GET", status, None, body),
            ];
            let (base, requests, server) = scripted_server(responses).await;
            let clock = Arc::new(ManualClock::new());
            let (daemon, log_path) = test_daemon(&base, clock.clone());
            let dispatcher = tokio::spawn(daemon.clone().dispatcher());
            let id = daemon
                .submit("fetch".into(), json!({}), 0, "test".into(), None)
                .unwrap();

            let (info, outcome) = wait_terminal(&daemon, id).await;
            assert_eq!(info.state, JobState::Failed);
            assert_eq!(info.retries, 0);
            let Outcome::Failure { error } = outcome else {
                panic!("{status} did not fail")
            };
            assert!(error.contains("NOT retrying"));
            assert!(error.contains(shape), "{error}");
            assert_eq!(requests.lock().unwrap().as_slice(), ["HEAD", "GET"]);
            assert_eq!(
                journal_waits(&log_path),
                [0, 0],
                "a block is never waited on"
            );
            server.await.unwrap();
            finish_harness_wire(dispatcher, &log_path, &requests);
        }
    }

    #[tokio::test]
    async fn full_acceptable_head_429_establishes_under_hold_without_job_retry() {
        let responses = vec![
            ScriptedResponse::full("HEAD", "429 Too Many Requests", Some(0), ""),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"items":["after-probe"]}"#),
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon(&base, clock.clone());
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let id = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();

        let (info, outcome) = wait_terminal(&daemon, id).await;
        assert_eq!(info.state, JobState::Done);
        assert_eq!(info.retries, 0);
        assert_eq!(fetch_payload_marker(&outcome), "after-probe");
        let probe = daemon
            .shared
            .lock()
            .unwrap()
            .jobs
            .values()
            .find(|entry| entry.info.kind == "probe")
            .unwrap()
            .info
            .clone();
        assert_eq!(probe.state, JobState::Done);
        assert_eq!(probe.retries, 0);
        assert_eq!(requests.lock().unwrap().as_slice(), ["HEAD", "GET"]);
        assert_eq!(
            journal_waits(&log_path),
            [0, full_hold_ms()],
            "the GET waits behind the probe's 429 hold"
        );
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
    }

    #[tokio::test]
    async fn malformed_or_unacceptable_head_429_fails_under_cooldown() {
        for head in [
            ScriptedResponse::malformed_head_429(),
            ScriptedResponse::full("HEAD", "429 Too Many Requests", None, ""),
        ] {
            let (base, requests, server) = scripted_server(vec![head]).await;
            let clock = Arc::new(ManualClock::new());
            let (daemon, log_path) = test_daemon(&base, clock);
            let dispatcher = tokio::spawn(daemon.clone().dispatcher());
            let id = daemon
                .submit("fetch".into(), json!({}), 0, "test".into(), None)
                .unwrap();

            let (info, _) = wait_terminal(&daemon, id).await;
            assert_eq!(info.state, JobState::Failed);
            assert_eq!(info.retries, 0);
            assert!(matches!(
                daemon.choke.endpoint_state("fetch"),
                EndpointState::Degraded { .. }
            ));
            assert_eq!(requests.lock().unwrap().as_slice(), ["HEAD"]);
            let probe = daemon
                .shared
                .lock()
                .unwrap()
                .jobs
                .values()
                .find(|entry| entry.info.kind == "probe")
                .unwrap()
                .info
                .clone();
            assert_eq!(probe.state, JobState::Failed);
            assert_eq!(probe.retries, 0);
            server.await.unwrap();
            finish_harness_wire(dispatcher, &log_path, &requests);
        }
    }

    // ---- L0 live-test rails (LIVE-TESTING.md) ------------------------------

    fn tripwire_config() -> RailsConfig {
        RailsConfig {
            tripwire: true,
            ..RailsConfig::default()
        }
    }

    /// Poll a job until `pred` holds on its snapshot (3 s budget).
    async fn wait_until(daemon: &Daemon, id: JobId, pred: impl Fn(&JobInfo) -> bool) -> JobInfo {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let info = daemon.shared.lock().unwrap().jobs[&id].info.clone();
                if pred(&info) {
                    return info;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("job reached the expected state")
    }

    #[tokio::test]
    async fn tripped_daemon_leaves_queued_jobs_waiting_until_reset() {
        // HEAD establishes, the first GET lands a 429 (tripping). Nothing
        // else reaches the server until the reset; then both queued jobs
        // go out (CONTEXT.md, "A rails halt leaves queued network jobs
        // waiting").
        let responses = vec![
            ScriptedResponse::full("HEAD", "204 No Content", None, ""),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(0), "{}"),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"ok":true}"#),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"ok":true}"#),
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) =
            test_daemon_with(Provider::mock(&base), clock.clone(), tripwire_config());
        let rails = daemon.choke.rails().clone();
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        let first = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let info = wait_until(&daemon, first, |i| i.retries == 1).await;
        assert_eq!(
            info.state,
            JobState::Waiting,
            "re-queued by the 429, then held"
        );
        assert!(
            rails.halted().unwrap().contains("429 on GET /fetch"),
            "{:?}",
            rails.halted()
        );
        let second = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(
            requests.lock().unwrap().as_slice(),
            ["HEAD", "GET"],
            "a tripped daemon sends nothing"
        );
        for id in [first, second] {
            let state = daemon.shared.lock().unwrap().jobs[&id].info.state;
            assert_eq!(state, JobState::Waiting, "job {id} waits out the halt");
        }
        let logged = {
            let s = daemon.shared.lock().unwrap();
            s.errors
                .iter()
                .any(|(_, m)| m.contains("LIVE-TEST RAILS TRIPPED"))
        };
        assert!(logged, "the trip is announced once in the error ring");

        // The reset wakes the queue; the limiter's own hold from the 429
        // still applies.
        daemon.reset_rails();
        let (info, _) = wait_terminal(&daemon, first).await;
        assert_eq!(info.state, JobState::Done);
        let (info, _) = wait_terminal(&daemon, second).await;
        assert_eq!(info.state, JobState::Done);
        assert_eq!(
            requests.lock().unwrap().as_slice(),
            ["HEAD", "GET", "GET", "GET"]
        );
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
    }

    #[tokio::test]
    async fn ceiling_halts_after_n_sends_and_does_not_persist() {
        let responses = vec![
            ScriptedResponse::full("HEAD", "204 No Content", None, ""),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"ok":true}"#),
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon_with(
            Provider::mock(&base),
            clock,
            RailsConfig {
                max_sends: Some(2),
                ..RailsConfig::default()
            },
        );
        let rails = daemon.choke.rails().clone();
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        let first = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        let (info, _) = wait_terminal(&daemon, first).await;
        assert_eq!(
            info.state,
            JobState::Done,
            "HEAD + GET reach the ceiling exactly"
        );
        assert!(rails.halted().unwrap().contains("ceiling: 2 of 2"));

        let second = daemon
            .submit("fetch".into(), json!({}), 0, "test".into(), None)
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let state = daemon.shared.lock().unwrap().jobs[&second].info.state;
        assert_eq!(
            state,
            JobState::Waiting,
            "a ceiling halt holds the queue too"
        );
        assert_eq!(requests.lock().unwrap().len(), 2);
        assert_eq!(rails.status().sends, 2);
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
    }

    // ---- persistence (CONTEXT.md, "The job queue persists") -------------

    fn test_db_path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "acquisition-jobs-{}-{}.db",
            std::process::id(),
            TEST_LOG_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    /// A harness daemon on a file-backed `daemon.db` (test daemons default
    /// to a throwaway in-memory one), restored from it like `run` does.
    fn persisting_daemon(base: &str, db_path: &std::path::Path) -> (Arc<Daemon>, PathBuf) {
        let (mut daemon, log_path) = test_daemon(base, Arc::new(ManualClock::new()));
        Arc::get_mut(&mut daemon).unwrap().jobs_db = Mutex::new(JobDb::open(db_path).unwrap());
        daemon.restore_jobs(Retention::default()).unwrap();
        (daemon, log_path)
    }

    fn remove_db(path: &std::path::Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }

    #[tokio::test]
    async fn the_queue_and_results_survive_a_daemon_restart() {
        let db = test_db_path();
        remove_db(&db);
        // Lifetime 1: one job finished, one running, one waiting. No
        // dispatcher runs — the running state is written the way
        // `process` writes it — so nothing of this lifetime survives the
        // drop to write behind lifetime 2's back.
        let (daemon, log1) = persisting_daemon("http://127.0.0.1:1", &db);
        let done = daemon
            .submit(
                "sleep".into(),
                json!({ "seconds": 0.01 }),
                0,
                "a".into(),
                None,
            )
            .unwrap();
        daemon.start_and_finish(
            done,
            Outcome::Success {
                payload: json!({ "slept_seconds": 0.01 }),
            },
        );
        let running = daemon
            .submit(
                "sleep".into(),
                json!({ "seconds": 0.3 }),
                2,
                "b".into(),
                None,
            )
            .unwrap();
        {
            let mut s = daemon.shared.lock().unwrap();
            let entry = s.jobs.get_mut(&running).unwrap();
            entry.info.state = JobState::Running;
            daemon.persist(entry);
        }
        let waiting = daemon
            .submit(
                "sleep".into(),
                json!({ "seconds": 0.01 }),
                1,
                "c".into(),
                None,
            )
            .unwrap();
        daemon.set_priority(waiting, 5).unwrap();
        drop(daemon);

        // Lifetime 2: the running job is re-queued, the waiting one kept
        // with its reprioritization, ids continue, and the finished job's
        // result is still answerable over the protocol.
        let (daemon, log2) = persisting_daemon("http://127.0.0.1:1", &db);
        let jobs = daemon.shared.lock().unwrap().list(&daemon);
        let summary: Vec<(JobId, JobState, Priority)> =
            jobs.iter().map(|j| (j.id, j.state, j.priority)).collect();
        assert_eq!(
            summary,
            [
                (running, JobState::Waiting, 2),
                (waiting, JobState::Waiting, 5)
            ],
            "open jobs come back; terminal ones stay in the table"
        );
        assert_eq!(jobs[0].submitted_by, "b", "every field rides along");
        let fresh = daemon
            .submit(
                "sleep".into(),
                json!({ "seconds": 0.01 }),
                0,
                "d".into(),
                None,
            )
            .unwrap();
        assert_eq!(fresh, waiting + 1, "ids continue across the restart");
        match daemon
            .handle_request(Request::Result { id: done }, &mut None)
            .await
        {
            Response::Result {
                outcome: Outcome::Success { payload },
                ..
            } => {
                assert_eq!(payload["slept_seconds"], json!(0.01))
            }
            other => panic!("previous lifetime's result not served: {other:?}"),
        }
        match daemon
            .handle_request(Request::Result { id: 999 }, &mut None)
            .await
        {
            Response::Error { message } => assert_eq!(message, "no job 999"),
            other => panic!("{other:?}"),
        }

        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        for id in [running, waiting, fresh] {
            let (info, _) = wait_terminal(&daemon, id).await;
            assert_eq!(info.state, JobState::Done, "job {id} runs to completion");
        }
        dispatcher.abort();
        // Lifetime 3 has nothing to restore.
        drop(daemon);
        let (daemon, log3) = persisting_daemon("http://127.0.0.1:1", &db);
        assert!(daemon.shared.lock().unwrap().jobs.is_empty());
        for p in [log1, log2, log3] {
            remove_harness_files(&p);
        }
        remove_db(&db);
    }

    #[tokio::test]
    async fn restore_drops_probes_and_finishes_a_parent_whose_children_are_done() {
        let db = test_db_path();
        remove_db(&db);
        {
            let db = JobDb::open(&db).unwrap();
            let row = |id: u64, kind: &str, state: &str, parent: Option<u64>| {
                acquisition_store::jobs::JobRow {
                    id,
                    kind: kind.into(),
                    state: state.into(),
                    priority: 0,
                    submitted_by: "t".into(),
                    parent,
                    retries: 0,
                    account: Some("A#1".into()),
                    params: json!({ "route": "fetch@A#1", "league": "Standard" }),
                    outcome: (state == "done")
                        .then(|| json!({ "outcome": "success", "payload": {} })),
                    deferred: None,
                    cancel_requested: false,
                    submitted_at: 0,
                    updated_at: 0,
                }
            };
            // A refresh whose last child finished just before the daemon
            // died: its own result is held, deferred.
            let mut parent = row(1, "refresh", "running", None);
            parent.deferred =
                Some(json!({ "outcome": "success", "payload": { "tabs_listed": 1 } }));
            db.upsert(&parent).unwrap();
            db.upsert(&row(2, "stash", "done", Some(1))).unwrap();
            // Per-lifetime: a probe from the previous daemon.
            db.upsert(&row(3, "probe", "waiting", None)).unwrap();
            // A running job that was asked to cancel: it never ran again.
            let mut cancelling = row(4, "sleep", "running", None);
            cancelling.cancel_requested = true;
            db.upsert(&cancelling).unwrap();
            // Running on a declared no-probe route: a replay would send
            // against an empty limiter, so it is not replayed.
            db.upsert(&row(5, "leagues", "running", None)).unwrap();
            // Mid-fan-out: children submitted, held result not yet
            // written. Re-running would duplicate the children.
            db.upsert(&row(6, "refresh", "running", None)).unwrap();
            db.upsert(&row(7, "stash", "waiting", Some(6))).unwrap();
        }
        let (daemon, log) = persisting_daemon("http://127.0.0.1:1", &db);
        let jobs = daemon.shared.lock().unwrap().list(&daemon);
        let states: Vec<(JobId, JobState)> = jobs.iter().map(|j| (j.id, j.state)).collect();
        assert_eq!(
            states,
            [
                (1, JobState::Done),
                (2, JobState::Done),
                (4, JobState::Cancelled),
                (5, JobState::Failed),
                (6, JobState::Running),
                (7, JobState::Waiting)
            ],
            "no-probe replays fail; a mid-fan-out parent holds for its children"
        );
        let Outcome::Failure { error } = daemon.shared.lock().unwrap().jobs[&5]
            .outcome
            .clone()
            .unwrap()
        else {
            panic!("the leagues replay did not fail")
        };
        assert!(error.contains("not replayed"), "{error}");
        // The held parent finishes when its remaining child does — as
        // interrupted, never success: the full child set is unknown.
        daemon.cancel(7).unwrap();
        let Outcome::Failure { error } = daemon.shared.lock().unwrap().jobs[&6]
            .outcome
            .clone()
            .unwrap()
        else {
            panic!("held parent must finish as interrupted, not success")
        };
        assert!(error.contains("mid fan-out"), "{error}");
        let Outcome::Success { payload } = daemon.shared.lock().unwrap().jobs[&1]
            .outcome
            .clone()
            .unwrap()
        else {
            panic!("parent did not finish with its held result")
        };
        assert_eq!(payload["tabs_listed"], json!(1));
        assert_eq!(payload["children"]["done"], json!(1));
        assert!(
            JobDb::open(&db).unwrap().get(3).unwrap().is_none(),
            "the probe row is gone from the table too"
        );
        assert_eq!(daemon.shared.lock().unwrap().next_id, 8);
        remove_harness_files(&log);
        remove_db(&db);
    }

    #[tokio::test]
    async fn a_failed_queue_write_refuses_new_jobs_and_stops_dispatch() {
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        daemon
            .submit("sleep".into(), json!({}), 0, "t".into(), None)
            .unwrap();
        daemon.jobs_db.lock().unwrap().break_for_tests();
        let err = daemon
            .submit("sleep".into(), json!({}), 0, "t".into(), None)
            .unwrap_err();
        assert!(err.contains("refuses new jobs"), "{err}");
        assert_eq!(
            daemon.shared.lock().unwrap().next_id,
            2,
            "the refused submit rolled its id back"
        );
        assert!(
            daemon.pick_runnable().is_empty(),
            "a failed queue dispatches nothing"
        );
        assert!(
            !Daemon::has_live_jobs(&daemon.shared.lock().unwrap(), true),
            "the parked queue does not hold the daemon up"
        );
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn a_cancelled_parent_cannot_gain_children() {
        // The race: cancel() enumerates children while the fan-out loop
        // keeps submitting. submit_child now refuses under the same lock
        // cancel takes.
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        let parent = daemon
            .submit("refresh".into(), json!({}), 0, "t".into(), None)
            .unwrap();
        {
            let mut s = daemon.shared.lock().unwrap();
            s.jobs.get_mut(&parent).unwrap().info.state = JobState::Running;
        }
        daemon.cancel(parent).unwrap();
        let err = daemon.submit_child(parent, "stash", json!({})).unwrap_err();
        assert!(err.contains("cancelled"), "{err}");
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn a_broken_queue_is_fatal_to_restore() {
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        daemon.jobs_db.lock().unwrap().break_for_tests();
        let err = daemon.restore_jobs(Retention::default()).unwrap_err();
        assert!(err.to_string().contains("could not read"), "{err:#}");
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn a_transition_that_cannot_persist_does_not_run() {
        // waiting → running is the transition that gates a send: if its
        // write fails, disk still says waiting, and after a restart a
        // no-probe job would replay blind. The job must not run.
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        let id = daemon
            .submit(
                "sleep".into(),
                json!({ "seconds": 0.01 }),
                0,
                "t".into(),
                None,
            )
            .unwrap();
        daemon.jobs_db.lock().unwrap().break_for_tests();
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        tokio::time::sleep(Duration::from_millis(100)).await;
        let state = daemon.shared.lock().unwrap().jobs[&id].info.state;
        assert_eq!(
            state,
            JobState::Waiting,
            "the job reverted instead of running"
        );
        assert!(
            daemon.queue_failed().is_some(),
            "the failed write tripped the queue"
        );
        dispatcher.abort();
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn cancel_and_reprioritize_report_a_failed_queue_write() {
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        let id = daemon
            .submit("sleep".into(), json!({}), 3, "t".into(), None)
            .unwrap();
        daemon.jobs_db.lock().unwrap().break_for_tests();
        let err = daemon.set_priority(id, 9).unwrap_err();
        assert!(err.contains("priority unchanged"), "{err}");
        assert_eq!(daemon.shared.lock().unwrap().jobs[&id].info.priority, 3);
        let err = daemon.cancel(id).unwrap_err();
        assert!(err.contains("may not survive"), "{err}");
        assert_eq!(
            daemon.shared.lock().unwrap().jobs[&id].info.state,
            JobState::Cancelled,
            "the cancellation still holds for this lifetime"
        );
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn a_parent_cancelled_after_its_last_child_is_not_a_success() {
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        let parent = daemon
            .submit("refresh".into(), json!({}), 0, "t".into(), None)
            .unwrap();
        {
            let mut s = daemon.shared.lock().unwrap();
            s.jobs.get_mut(&parent).unwrap().info.state = JobState::Running;
        }
        let _child = daemon.submit_child(parent, "stash", json!({})).unwrap();
        // The cancellation lands after the fan-out submitted its last
        // child but before the parent's own outcome is installed.
        daemon.cancel(parent).unwrap();
        daemon.conclude(
            parent,
            Outcome::Success {
                payload: json!({ "tabs_listed": 1 }),
            },
        );
        let (state, outcome) = {
            let s = daemon.shared.lock().unwrap();
            let e = &s.jobs[&parent];
            (e.info.state, e.outcome.clone())
        };
        assert_eq!(state, JobState::Cancelled, "never a success");
        assert!(matches!(outcome, Some(Outcome::Cancelled)));
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn cancelled_children_do_not_make_a_parent_successful() {
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        let parent = daemon
            .submit("refresh".into(), json!({}), 0, "t".into(), None)
            .unwrap();
        {
            let mut s = daemon.shared.lock().unwrap();
            s.jobs.get_mut(&parent).unwrap().info.state = JobState::Running;
        }
        let child = daemon.submit_child(parent, "stash", json!({})).unwrap();
        daemon.cancel(child).unwrap();
        daemon.conclude(parent, Outcome::Success { payload: json!({}) });
        let Outcome::Failure { error } = daemon.shared.lock().unwrap().jobs[&parent]
            .outcome
            .clone()
            .unwrap()
        else {
            panic!("a parent with cancelled children must not succeed")
        };
        assert!(error.contains("cancelled"), "{error}");
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn a_result_read_failure_is_reported_not_no_job() {
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        daemon.jobs_db.lock().unwrap().break_for_tests();
        match daemon
            .handle_request(Request::Result { id: 42 }, &mut None)
            .await
        {
            Response::Error { message } => assert!(
                message.contains("could not read the persisted queue"),
                "{message}"
            ),
            other => panic!("{other:?}"),
        }
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn a_cancellation_pending_at_finish_wins_the_terminal_state() {
        // The gap this pins: a caller (conclude, maybe_finish_parent)
        // computes an outcome, releases the lock, and a cancel lands
        // before finish takes the final lock. The stale success must not
        // be written.
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        let id = daemon
            .submit("sleep".into(), json!({}), 0, "t".into(), None)
            .unwrap();
        {
            let mut s = daemon.shared.lock().unwrap();
            s.jobs.get_mut(&id).unwrap().info.state = JobState::Running;
        }
        daemon.cancel(id).unwrap();
        daemon.finish(
            id,
            Outcome::Success {
                payload: json!({ "stale": true }),
            },
        );
        let (state, outcome) = {
            let s = daemon.shared.lock().unwrap();
            let e = &s.jobs[&id];
            (e.info.state, e.outcome.clone())
        };
        assert_eq!(state, JobState::Cancelled);
        assert!(matches!(outcome, Some(Outcome::Cancelled)));
        remove_harness_files(&log);
    }

    #[tokio::test]
    async fn a_halted_daemon_with_only_waiting_jobs_is_idle() {
        let (daemon, log) = test_daemon("http://127.0.0.1:1", Arc::new(ManualClock::new()));
        let id = daemon
            .submit("fetch".into(), json!({}), 0, "t".into(), None)
            .unwrap();
        {
            let s = daemon.shared.lock().unwrap();
            assert!(
                Daemon::has_live_jobs(&s, false),
                "a waiting job keeps an unhalted daemon up"
            );
            assert!(
                !Daemon::has_live_jobs(&s, true),
                "…but not a halted one: the queue is on disk"
            );
        }
        daemon
            .shared
            .lock()
            .unwrap()
            .jobs
            .get_mut(&id)
            .unwrap()
            .info
            .state = JobState::Running;
        assert!(
            Daemon::has_live_jobs(&daemon.shared.lock().unwrap(), true),
            "a running job keeps even a halted daemon up"
        );
        remove_harness_files(&log);
    }

    async fn token_server_answering(
        statuses: Vec<&'static str>,
    ) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let task = tokio::spawn(async move {
            for status in statuses {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = mockggg::read_request(&mut stream).await.unwrap();
                assert_eq!(request.path, "/token");
                captured.lock().unwrap().push(request.body.clone());
                let headers = concat!(
                    "X-Rate-Limit-Policy: token-request-limit\r\n",
                    "X-Rate-Limit-Rules: Ip\r\n",
                    "X-Rate-Limit-Ip: 60:30:30\r\n",
                    "X-Rate-Limit-Ip-State: 1:30:0\r\n",
                );
                let body = if status.starts_with("200") {
                    json!({
                        "access_token": "at-new",
                        "refresh_token": "rt-rotated",
                        "expires_in": 3600,
                        "username": "test-user",
                    })
                    .to_string()
                } else {
                    r#"{"error":"invalid_grant"}"#.to_string()
                };
                mockggg::respond_with(&mut stream, status, "application/json", headers, &body)
                    .await;
            }
        });
        (base, requests, task)
    }

    /// Session in memory and the OS keyring swapped out: these tests must
    /// never touch the real keychain (slow, and may prompt).
    fn logged_in(daemon: &mut Arc<Daemon>) {
        Arc::get_mut(daemon).unwrap().credential_store =
            Arc::new(RecordingCredentialStore::default());
        let mut s = daemon.shared.lock().unwrap();
        s.auth.one_mut().refresh_token = Some("rt-old".into());
        s.auth.rename("", "test-user");
        s.auth.keyring = "ok".into();
    }

    #[tokio::test]
    async fn rejected_refresh_grant_disables_further_refreshes_until_login_or_logout() {
        let (base, requests, server) = token_server_answering(vec!["400 Bad Request"]).await;
        let clock = Arc::new(ManualClock::new());
        // Rails off: this is product behavior, not a ladder rail (L0-R13).
        let (mut daemon, log_path) =
            test_daemon_with(Provider::mock(&base), clock, RailsConfig::default());
        let rails = daemon.choke.rails().clone();
        logged_in(&mut daemon);

        let first = daemon.valid_access_token(None, false).await.unwrap_err();
        assert!(first.contains("400 Bad Request"), "{first}");
        assert_eq!(requests.lock().unwrap().len(), 1);
        assert!(
            rails
                .refresh_failed("test-user")
                .unwrap()
                .contains("HTTP 400")
        );

        let second = daemon.valid_access_token(None, false).await.unwrap_err();
        assert!(
            second.contains(
                "token refresh disabled for test-user: refresh token rejected with HTTP 400"
            ),
            "{second}"
        );
        assert_eq!(
            requests.lock().unwrap().len(),
            1,
            "the dead token is not re-sent"
        );
        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(
                s.auth.one().refresh_token.as_deref(),
                Some("rt-old"),
                "nothing is deleted"
            );
            assert!(s.auth.one().refresh_flight.is_none());
        }

        daemon.logout(None).unwrap_or(());
        assert_eq!(
            rails.refresh_failed("test-user"),
            None,
            "logout clears the mark"
        );
        server.await.unwrap();
        assert_journal_matches_wire(&log_path, &wire_posts(&requests));
        remove_harness_files(&log_path);
    }

    #[tokio::test]
    async fn transient_refresh_failures_do_not_mark_the_session() {
        let (base, requests, server) =
            token_server_answering(vec!["503 Service Unavailable", "200 OK"]).await;
        let clock = Arc::new(ManualClock::new());
        let (mut daemon, log_path) =
            test_daemon_with(Provider::mock(&base), clock, tripwire_config());
        let rails = daemon.choke.rails().clone();
        logged_in(&mut daemon);

        let first = daemon.valid_access_token(None, false).await.unwrap_err();
        assert!(first.contains("503"), "{first}");
        assert_eq!(
            rails.refresh_failed("test-user"),
            None,
            "5xx is not a rejected grant"
        );
        // The tripwire did trip on the 503 (Cloudflare shape); clear it so
        // the retry can show the refresh path itself is untouched.
        assert!(rails.halted().unwrap().contains("503"));
        rails.reset_tripwire();

        let (token, user) = daemon.valid_access_token(None, false).await.unwrap();
        assert_eq!((token.as_str(), user.as_str()), ("at-new", "test-user"));
        assert_eq!(requests.lock().unwrap().len(), 2);
        server.await.unwrap();
        assert_journal_matches_wire(&log_path, &wire_posts(&requests));
        remove_harness_files(&log_path);
    }

    #[tokio::test]
    async fn halted_daemon_refuses_refresh_before_any_send() {
        let (base, requests, server) = token_server_answering(vec![]).await;
        let clock = Arc::new(ManualClock::new());
        let (mut daemon, log_path) =
            test_daemon_with(Provider::mock(&base), clock, tripwire_config());
        let rails = daemon.choke.rails().clone();
        rails.record(&crate::rails::SendReport {
            method: "GET",
            route: "character-list",
            url_path: "/character",
            status: Some(429),
            error: None,
            ok: false,
            counted: true,
            rate: &Value::Null,
            shape: None,
            headers: &serde_json::Value::Null,
            wait: Duration::ZERO,
        });
        logged_in(&mut daemon);
        let error = daemon.valid_access_token(None, false).await.unwrap_err();
        assert!(error.contains("halted by live-test rails"), "{error}");
        assert!(requests.lock().unwrap().is_empty());
        server.abort();
        // The one journal line is the synthetic 429 recorded above, not a
        // send; nothing else may have been written.
        assert_journal_matches_wire(&log_path, &["GET".to_string()]);
        remove_harness_files(&log_path);
    }

    #[tokio::test]
    async fn whoami_is_refused_in_real_mode_without_a_token_request() {
        // Real provider, but no session: even if the refusal were missing
        // the job could not send. The assertion is on the refusal message.
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon_with(Provider::ggg(), clock, RailsConfig::default());
        let outcome = daemon
            .execute_inner(1, "whoami", json!({}), None, None, daemon.choke.now())
            .await
            .unwrap();
        let Outcome::Failure { error } = outcome else {
            panic!("whoami ran in real mode")
        };
        assert!(error.contains("mock-only"), "{error}");
        assert!(daemon.choke.recent_sends().is_empty());
        assert_journal_matches_wire(&log_path, &[]);
        remove_harness_files(&log_path);
    }

    /// The `quote` protocol request (tracer step 5): a read-only,
    /// non-reserving projection. A learned policy's scope carries
    /// per-window headroom and a forward-simulated ETA; an unlearned
    /// route is unquotable and says so; probes, OAuth, 429s, and
    /// non-sending jobs are named as not covered instead of silently
    /// omitted. Nothing is sent, enqueued, or remembered.
    #[tokio::test]
    async fn quote_projects_per_scope_reserves_nothing_and_names_what_it_cannot_see() {
        let clock = Arc::new(ManualClock::new());
        let base = mockggg::start_with_clock(clock.clone()).await.unwrap();
        let (daemon, log_path) = test_daemon(&base, clock);
        // Teach the mock's 5-per-10s fetch policy through the ordinary
        // probe path; `character-list` stays unlearned.
        daemon
            .choke
            .head("fetch", &format!("{base}/fetch"), None, daemon.choke.now())
            .await
            .unwrap();
        let sends_before = daemon.choke.recent_sends().len();

        let mut jobs: Vec<QuoteJob> = (0..8)
            .map(|_| QuoteJob {
                kind: "fetch".into(),
                params: json!({}),
            })
            .collect();
        jobs.push(QuoteJob {
            kind: "characters".into(),
            params: json!({}),
        });
        jobs.push(QuoteJob {
            kind: "sleep".into(),
            params: json!({ "seconds": 1.0 }),
        });
        let quote = match daemon
            .handle_request(
                Request::Quote {
                    jobs,
                    account: None,
                },
                &mut None,
            )
            .await
        {
            Response::Quote { quote } => quote,
            other => panic!("{other:?}"),
        };

        assert_eq!(quote.provider, "mock");
        assert!(quote.halted.is_none());
        // The work is echoed verbatim, in order — the quote's verifiable
        // basis for any carrier (a plan checks it against its actions).
        assert_eq!(quote.work.len(), 10);
        assert_eq!(quote.work[0].kind, "fetch");
        assert_eq!(quote.work[8].kind, "characters");
        assert_eq!(quote.work[9].kind, "sleep");
        let [unknown, learned] = quote.scopes.as_slice() else {
            panic!("expected two scopes: {:?}", quote.scopes);
        };
        // The learned route is keyed by its policy and quotes per-window
        // headroom; 8 sends through 5-per-10s cannot be immediate.
        assert_eq!(learned.key, "mock-fetch-request-limit");
        assert_eq!(learned.policy.as_deref(), Some("mock-fetch-request-limit"));
        assert_eq!(learned.endpoints, ["fetch"]);
        assert_eq!((learned.requests, learned.queued_ahead), (8, 0));
        let windows: Vec<(u32, u32)> = learned
            .rules
            .iter()
            .flat_map(|r| r.windows.iter().map(|w| (w.hits, w.max_hits)))
            .collect();
        assert!(
            windows.contains(&(0, 5)),
            "per-window headroom: {windows:?}"
        );
        assert!(
            learned.eta_seconds.is_some_and(|eta| eta >= 10),
            "{:?}",
            learned.eta_seconds
        );
        // The rules carry their own observation basis — they are as old
        // as the probe, not as fresh as the quote.
        assert!(learned.observed_seconds_ago.is_some());
        assert!(learned.notes.is_empty(), "{:?}", learned.notes);
        // The unlearned route is unquotable and says so, never guessed.
        assert_eq!(unknown.key, "character-list");
        assert_eq!(unknown.policy, None);
        assert_eq!(unknown.eta_seconds, None);
        assert_eq!(unknown.observed_seconds_ago, None);
        assert!(
            unknown.notes[0].contains("HEAD probe"),
            "{:?}",
            unknown.notes
        );
        // What the quote does not cover is named.
        let nc = quote.not_covered.join("\n");
        assert!(nc.contains("character-list"), "{nc}");
        assert!(nc.contains("OAuth token refresh"), "{nc}");
        assert!(nc.contains("429 re-sends"), "{nc}");
        assert!(nc.contains("`sleep` job(s) send nothing"), "{nc}");
        // Read-only and non-reserving: no job exists, nothing was sent.
        assert!(daemon.shared.lock().unwrap().jobs.is_empty());
        assert_eq!(daemon.choke.recent_sends().len(), sends_before);
        remove_harness_files(&log_path);
    }

    #[tokio::test]
    async fn quote_counts_queued_jobs_ahead_and_keys_scopes_per_account() {
        let clock = Arc::new(ManualClock::new());
        let base = mockggg::start_with_clock(clock.clone()).await.unwrap();
        let (daemon, log_path) = test_daemon(&base, clock);
        daemon.shared.lock().unwrap().auth.rename("", "Alice#1234");
        daemon
            .choke
            .head("fetch", &format!("{base}/fetch"), None, daemon.choke.now())
            .await
            .unwrap();
        // Three fetch jobs sit waiting (no dispatcher runs here).
        for _ in 0..3 {
            daemon
                .submit("fetch".into(), json!({}), 0, "test".into(), None)
                .unwrap();
        }
        // A refresh parent that already made its listing request and now
        // holds its deferred result while waiting for children: on the
        // stash-list scope, but no longer a future send.
        let parent = daemon
            .submit(
                "refresh".into(),
                json!({ "league": "Standard" }),
                0,
                "test".into(),
                Some("Alice#1234".into()),
            )
            .unwrap();
        {
            let mut s = daemon.shared.lock().unwrap();
            let e = s.jobs.get_mut(&parent).unwrap();
            e.info.state = JobState::Running;
            e.deferred = Some(Outcome::Success { payload: json!({}) });
        }
        let quote = match daemon
            .handle_request(
                Request::Quote {
                    jobs: vec![
                        QuoteJob {
                            kind: "fetch".into(),
                            params: json!({}),
                        },
                        QuoteJob {
                            kind: "stashes".into(),
                            params: json!({ "league": "Standard" }),
                        },
                    ],
                    account: None,
                },
                &mut None,
            )
            .await
        {
            Response::Quote { quote } => quote,
            other => panic!("{other:?}"),
        };
        // The queue competes: waiting jobs on the scope go ahead of the
        // quoted work.
        let fetch = quote
            .scopes
            .iter()
            .find(|s| s.key == "mock-fetch-request-limit")
            .unwrap();
        assert_eq!((fetch.requests, fetch.queued_ahead), (1, 3));
        // An account-holding job keys its scope per account, exactly as
        // the limiter and dispatcher would — and the deferred parent on
        // that scope is not counted ahead: its own send already happened.
        let listing = quote
            .scopes
            .iter()
            .find(|s| s.key == "stash-list@Alice#1234")
            .unwrap_or_else(|| panic!("{:?}", quote.scopes));
        assert_eq!(listing.queued_ahead, 0);
        assert_eq!(quote.account.as_deref(), Some("Alice#1234"));
        // A selector naming no live session refuses the quote whole — the
        // same rules as Submit, so a quote never keys the wrong state.
        // Even an empty job list judges its selector: a zero-action quote
        // must not launder an unknown account into `account: null`.
        match daemon
            .handle_request(
                Request::Quote {
                    jobs: Vec::new(),
                    account: Some("Bob".into()),
                },
                &mut None,
            )
            .await
        {
            Response::Error { message } => assert!(message.contains("Bob"), "{message}"),
            other => panic!("{other:?}"),
        }
        // Omitted with several sessions live is ambiguous too — even an
        // empty quote never guesses its headline account.
        daemon.shared.lock().unwrap().auth.by_account.insert(
            "Bob#9".into(),
            AuthSession {
                username: Some("Bob#9".into()),
                ..Default::default()
            },
        );
        match daemon
            .handle_request(
                Request::Quote {
                    jobs: Vec::new(),
                    account: None,
                },
                &mut None,
            )
            .await
        {
            Response::Error { message } => assert!(message.contains("several"), "{message}"),
            other => panic!("{other:?}"),
        }
        remove_harness_files(&log_path);
    }
}
