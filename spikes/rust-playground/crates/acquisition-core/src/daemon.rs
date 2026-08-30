//! The daemon: single job queue, single worker, JSON-lines Unix socket server.
//!
//! Lifecycle follows the gpg-agent model: clients spawn it on demand, it exits
//! on its own after a stretch with no connections and no live jobs.

use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};

use acquisition_store::{Endpoint, Index, Store, account_path, store_dir};
use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener, UnixStream};
use tokio::sync::{Notify, broadcast, watch};

use std::collections::VecDeque;

use crate::VERSION;
use crate::job::{JobId, JobInfo, JobState, Outcome, Priority};
use crate::protocol::{ErrorRecord, Request, Response};
use crate::provider::{CALLBACK_PATH, Provider, SCOPES, ggg_mode};
use crate::rails::{BlockShape, Rails, RailsConfig};
use crate::ratelimit::{
    ChokePoint, Clock, EndpointState, RetryAfter, SendError, SystemClock, url_path,
};
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

#[derive(Default)]
struct AuthSession {
    access_token: Option<String>,
    /// Wall-clock, not `Instant`: on macOS a monotonic clock does not advance
    /// while the machine sleeps, so an `Instant` deadline would wake believing
    /// an expired token still has hours left and send 401s until it caught up.
    access_expires_at: Option<SystemTime>,
    refresh_token: Option<String>,
    username: Option<String>,
    /// The session generation of a login flow waiting on the browser.
    pending: Option<u64>,
    /// "ok" or an error description shown in `auth status`.
    keyring: String,
    generations: AuthGenerations,
    refresh_flight: Option<RefreshFlight>,
    next_refresh_flight: u64,
}

impl AuthSession {
    fn advance_session(&mut self) -> u64 {
        self.generations.session = self.generations.session.wrapping_add(1);
        self.generations.session
    }

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
    auth: AuthSession,
    connections: usize,
    last_activity: Instant,
    started: Instant,
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
            match daemon.route_for(&info.kind, &entry.params) {
                Some((route, _)) => {
                    // Only same-route jobs ahead of us compete for the same
                    // policy; counting them is what the estimate needs.
                    let ahead = queue
                        .iter()
                        .take_while(|&&q| q != id)
                        .filter(|q| {
                            self.jobs
                                .get(q)
                                .and_then(|e| daemon.route_for(&e.info.kind, &e.params))
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
}

struct RefreshOwnerGuard<'a> {
    daemon: &'a Daemon,
    id: u64,
    generations: AuthGenerations,
    result: Option<watch::Sender<Option<AccessTokenResult>>>,
}

impl RefreshOwnerGuard<'_> {
    fn finish(mut self, refresh: Result<auth::TokenResponse, String>) -> AccessTokenResult {
        let result = self.result.as_ref().expect("refresh owner result exists");
        let outcome = self
            .daemon
            .finish_refresh(self.id, self.generations, refresh, result);
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
            let owns_current_flight = s.auth.generations == self.generations
                && s.auth.refresh_flight.as_ref().is_some_and(|flight| {
                    flight.id == self.id && flight.generations == self.generations
                });
            if owns_current_flight {
                s.auth.refresh_flight = None;
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
        let uptime = self.shared.lock().unwrap().started.elapsed().as_secs();
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

    /// Apply one change to the account index, logging (never failing) on
    /// error. No-op in tests (no store directory).
    fn with_index(&self, f: impl FnOnce(&mut Index) -> anyhow::Result<()>) {
        let Some(dir) = &self.store_dir else { return };
        let result = Index::load(dir).and_then(|mut index| f(&mut index));
        if let Err(e) = result {
            self.note_error(&format!("accounts index: {e:#}"));
        }
    }

    /// Hand a successful API body to the shared store. The daemon's whole
    /// involvement: endpoint + params + body; what is inside is the store's
    /// business. A store failure is logged, never a job failure — the job's
    /// payload still reaches the client that asked.
    fn record(&self, kind: &str, params: &Value, body: &Value) {
        let Some(dir) = &self.store_dir else { return };
        let Some(endpoint) = Endpoint::from_job(kind, params) else {
            return;
        };
        // The store is the session's account's file, opened on first use;
        // a re-login as another account switches files.
        let username = self.shared.lock().unwrap().auth.username.clone();
        let Some(username) = username else {
            self.note_error(&format!(
                "store: {kind} landed with no session username; not recorded"
            ));
            return;
        };
        let mut guard = self.store.lock().unwrap();
        if guard.as_ref().is_none_or(|(u, _)| *u != username) {
            let path = account_path(dir, &username);
            match Store::open(&path) {
                Ok(store) => {
                    self.log(&format!("store: {} opened for {username}", path.display()));
                    *guard = Some((username.clone(), store));
                }
                Err(e) => {
                    self.note_error(&format!("store: could not open {}: {e:#}", path.display()));
                    return;
                }
            }
        }
        let (_, store) = guard.as_mut().expect("store opened above");
        let result = store.record(&endpoint, params, 200, body, acquisition_store::now());
        match result {
            Ok(ingest) => self.log(&format!(
                "store: {kind} {} -> response {} | {} items (+{} ~{} >{} -{})",
                JobInfo {
                    id: 0,
                    kind: kind.into(),
                    state: JobState::Done,
                    priority: 0,
                    submitted_by: String::new(),
                    eta_seconds: None,
                    parent: None,
                    retries: 0,
                    params: params.clone()
                }
                .target(),
                ingest.response_id,
                ingest.items,
                ingest.added,
                ingest.changed,
                ingest.moved,
                ingest.removed
            )),
            Err(e) => self.note_error(&format!("store: recording {kind} failed: {e:#}")),
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
    /// not URLs, key the limiter: every league shares `stash-list`. `fetch`
    /// is a fake data endpoint that exists only on the mock.
    fn route_for(&self, kind: &str, params: &Value) -> Option<(String, String)> {
        let base = &self.provider.api_base;
        match kind {
            "characters" => Some(("character-list".into(), format!("{base}/character"))),
            // One character with its inventory/equipment: its own policy.
            "character" => {
                let name = params.get("name").and_then(Value::as_str)?;
                Some(("character".into(), format!("{base}/character/{name}")))
            }
            // The account's leagues (the OAuth `GET /league`, account:leagues).
            "leagues" => Some(("league".into(), format!("{base}/league"))),
            // The account profile (account:profile).
            "profile" => Some(("profile".into(), format!("{base}/profile"))),
            "stashes" => {
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard");
                Some(("stash-list".into(), format!("{base}/stash/{league}")))
            }
            // One tab, or one substash of a map/unique tab: same route, same
            // policy (stash-request-limit), one probe for all of them.
            "stash" => {
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard");
                let id = params.get("id").and_then(Value::as_str)?;
                let url = match params.get("sub").and_then(Value::as_str) {
                    Some(sub) => format!("{base}/stash/{league}/{id}/{sub}"),
                    None => format!("{base}/stash/{league}/{id}"),
                };
                Some(("stash".into(), url))
            }
            "refresh" => {
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard");
                Some(("stash-list".into(), format!("{base}/stash/{league}")))
            }
            "fetch" if !self.provider.is_real() => Some(("fetch".into(), format!("{base}/fetch"))),
            _ => None,
        }
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
        match self.route_for(&e.info.kind, &e.params) {
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

    /// Make sure a probe for `route` is queued or running; submit one if not.
    /// One probe per route per daemon lifetime in the normal case — the
    /// sanctioned "one HEAD at startup" (N16), sent lazily on first use.
    fn ensure_probe(&self, route: &str, url: &str) {
        let pending = Self::probe_pending(&self.shared.lock().unwrap(), route);
        if !pending {
            self.log(&format!(
                "route {route} unknown; probing {} first",
                url_path(url)
            ));
            self.submit(
                "probe".into(),
                json!({ "route": route, "url": url }),
                PROBE_PRIORITY,
                "daemon".into(),
            );
        }
    }

    fn submit(
        &self,
        kind: String,
        params: Value,
        priority: Priority,
        submitted_by: String,
    ) -> JobId {
        self.submit_with_parent(kind, params, priority, submitted_by, None)
    }

    /// A child inherits its parent's priority and submitter.
    fn submit_child(&self, parent: JobId, kind: &str, params: Value) -> Option<JobId> {
        let (priority, by) = {
            let s = self.shared.lock().unwrap();
            let p = s.jobs.get(&parent)?;
            (p.info.priority, p.info.submitted_by.clone())
        };
        Some(self.submit_with_parent(kind.into(), params, priority, by, Some(parent)))
    }

    fn submit_with_parent(
        &self,
        kind: String,
        params: Value,
        priority: Priority,
        submitted_by: String,
        parent: Option<JobId>,
    ) -> JobId {
        let info = {
            let mut s = self.shared.lock().unwrap();
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
                params: params.clone(),
            };
            s.jobs.insert(
                id,
                Entry {
                    info: info.clone(),
                    params,
                    outcome: None,
                    cancel_requested: false,
                    deferred: None,
                },
            );
            info
        };
        let id = info.id;
        self.emit(info);
        self.work.notify_one();
        id
    }

    fn cancel(&self, id: JobId) -> Result<(), String> {
        // Cancelling a parent cancels everything under it: waiting
        // descendants immediately, running ones at their next slice.
        let mut emits = Vec::new();
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
                    _ => {}
                }
            }
        }
        for info in emits {
            self.emit(info);
        }
        Ok(())
    }

    fn set_priority(&self, id: JobId, priority: Priority) -> Result<(), String> {
        let mut s = self.shared.lock().unwrap();
        let entry = s.jobs.get_mut(&id).ok_or_else(|| format!("no job {id}"))?;
        if entry.info.state != JobState::Waiting {
            return Err(format!("job {id} is {}, not waiting", entry.info.state));
        }
        entry.info.priority = priority;
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
        let mut busy: HashSet<String> = s.active_jobs.values().cloned().collect();
        let mut picks = Vec::new();
        for id in s.queue_order() {
            let entry = &s.jobs[&id];
            // A job whose route is still being probed has nothing to do yet.
            if let Some((route, _)) = self.route_for(&entry.info.kind, &entry.params)
                && self.choke.endpoint_state(&route) == EndpointState::Unknown
                && Self::probe_pending(&s, &route)
            {
                continue;
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
        let route = {
            let s = self.shared.lock().unwrap();
            match s.jobs.get(&id) {
                Some(e) => self.route_for(&e.info.kind, &e.params),
                None => return,
            }
        };

        // L0 rail 1: a halted daemon fails network jobs immediately — no
        // probe, no pacing wait behind a hold, no permit — with the cause.
        if let Some((route, _)) = &route
            && let Some(cause) = self.rails().halted()
        {
            let error = format!("route {route}: {cause}");
            // File log only: a halted fan-out must not evict the trip cause
            // from the dashboard's error ring.
            self.log(&format!("job {id}: {error}"));
            self.start_and_finish(id, Outcome::Failure { error });
            return;
        }

        // A route we've never heard from gets a probe first (N16, N24); a
        // degraded one (N20) fails its jobs cleanly until the cooldown ends.
        if let Some((route, url)) = &route {
            match self.choke.endpoint_state(route) {
                EndpointState::Unknown => {
                    self.ensure_probe(route, url);
                    return; // scheduling key released; the probe outranks us
                }
                EndpointState::Degraded { until, reason } => {
                    let left = until.saturating_duration_since(self.choke.now()).as_secs();
                    let error = format!("route {route} is degraded for another {left}s: {reason}");
                    self.note_error(&format!("job {id}: {error}"));
                    self.start_and_finish(id, Outcome::Failure { error });
                    return;
                }
                EndpointState::Policy(_) | EndpointState::Policyless => {}
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
            (entry.info.clone(), entry.params.clone())
        };
        let (info, params) = job;
        let kind = info.kind.clone();
        self.emit(info);

        let route = route.map(|(route, _)| route);
        let outcome = match self.execute(id, &kind, params, route, ready).await {
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
        // A job that spawned children holds its own result until they're
        // all done. It gives its scheduling key back so children can run.
        let has_children = {
            let mut s = self.shared.lock().unwrap();
            let spawned = s.jobs.values().any(|e| e.info.parent == Some(id));
            if spawned
                && let Some(entry) = s.jobs.get_mut(&id)
                && entry.info.state == JobState::Running
            {
                entry.deferred = Some(outcome.clone());
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
            let summary = json!({ "done": done, "failed": failed, "cancelled": cancelled, "failed_ids": failed_ids });
            let deferred = s.jobs.get_mut(&pid).unwrap().deferred.take().unwrap();
            match deferred {
                Outcome::Success { mut payload } if failed == 0 => {
                    payload["children"] = summary;
                    Outcome::Success { payload }
                }
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

    fn finish(&self, id: JobId, outcome: Outcome) {
        let info = {
            let mut s = self.shared.lock().unwrap();
            let Some(entry) = s.jobs.get_mut(&id) else {
                return;
            };
            entry.info.state = match &outcome {
                Outcome::Success { .. } => JobState::Done,
                Outcome::Failure { .. } => JobState::Failed,
                Outcome::Cancelled => JobState::Cancelled,
            };
            entry.outcome = Some(outcome);
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
        ready: Instant,
    ) -> Exec {
        // Network kinds bubble a 429 up as `Exec::RateLimited`; everything
        // else is an ordinary outcome.
        match self.execute_inner(id, kind, params, route, ready).await {
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
        ready: Instant,
    ) -> Result<Outcome, ApiError> {
        Ok(match kind {
            // The one real API call: GET {api_base}/character. Same code in
            // both modes; only the provider's base URL differs. The choke
            // point classifies and observes every response; the dispatcher
            // requeues only acceptable 429s, while Cloudflare-shaped blocks
            // are never retried.
            "characters" => {
                let (token, username) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let url = format!("{}/character", self.provider.api_base);
                let route = route.as_deref().expect("characters is a network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                self.record(kind, &params, &v);
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "username": username,
                        "characters": v.get("characters").cloned().unwrap_or(v),
                        "rate_limit": rate,
                    }),
                }
            }
            "character" | "leagues" | "profile" => {
                let (token, username) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let Some((_, url)) = self.route_for(kind, &params) else {
                    return Ok(Outcome::Failure {
                        error: format!("{kind} needs a name"),
                    });
                };
                let route = route.as_deref().expect("network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                self.record(kind, &params, &v);
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
                let (token, username) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let league = params
                    .get("league")
                    .and_then(Value::as_str)
                    .unwrap_or("Standard")
                    .to_string();
                let url = format!("{}/stash/{league}", self.provider.api_base);
                let route = route.as_deref().expect("stashes is a network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                self.record(kind, &params, &v);
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "username": username,
                        "league": league,
                        "stashes": v.get("stashes").cloned().unwrap_or(v),
                        "rate_limit": rate,
                    }),
                }
            }
            "stash" => {
                let (token, _) = match self.valid_access_token(false).await {
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
                self.record(kind, &params, &v);
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
                    let league = params.get("league").cloned().unwrap_or(json!("Standard"));
                    let tab = params.get("id").cloned().unwrap_or(Value::Null);
                    for child in &children {
                        if let Some(sub) = child.get("id").and_then(Value::as_str)
                            && let Some(cid) = self.submit_child(
                                id,
                                "stash",
                                json!({ "league": league, "id": tab, "sub": sub, "deep": false }),
                            )
                        {
                            submitted.push(cid);
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
                let (token, _) = match self.valid_access_token(false).await {
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
                let url = format!("{}/stash/{league}", self.provider.api_base);
                let route = route.as_deref().expect("refresh is a network kind");
                let (v, rate) = self.api_get(route, &url, Some(&token), ready).await?;
                // The list a refresh fetches is the same response `stashes`
                // records; the store wants it under that endpoint.
                self.record("stashes", &json!({ "league": params.get("league").cloned().unwrap_or(json!("Standard")) }), &v);
                let listed = v
                    .get("stashes")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
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
                    if let Some(cid) = self.submit_child(
                        id,
                        "stash",
                        json!({ "league": league, "id": tid, "deep": follow }),
                    ) {
                        submitted.push(cid);
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
                let (token, username) = match self.valid_access_token(false).await {
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
                    match self.valid_access_token(false).await {
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
                    "unknown job kind '{other}' (kinds: sleep, fetch, whoami, profile, characters, character, leagues, stashes, stash, refresh, probe)"
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
                        Ok(tokens) => {
                            let user = tokens.username.clone();
                            if daemon.finish_auth_flow(flow_generation, tokens) {
                                // A probe that failed for lack of a session would
                                // succeed now; don't make the user sit out the cooldown.
                                daemon.choke.forget_degraded();
                                daemon.log(&format!("logged in as {user}"));
                            } else {
                                daemon.log("stale auth flow completion ignored");
                            }
                        }
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

    fn begin_auth_flow(&self) -> u64 {
        let mut s = self.shared.lock().unwrap();
        let generation = s.auth.advance_session();
        s.auth.pending = Some(generation);
        s.auth.refresh_flight = None;
        generation
    }

    fn finish_auth_flow(&self, generation: u64, tokens: auth::TokenResponse) -> bool {
        let warning = {
            let mut s = self.shared.lock().unwrap();
            if s.auth.pending != Some(generation) || s.auth.generations.session != generation {
                return false;
            }
            s.auth.pending = None;
            self.install_tokens_locked(&mut s.auth, tokens)
        };
        // Only a current flow that installed a new token clears the
        // refresh-failed mark; a stale callback must not re-arm a dead one.
        self.rails().clear_refresh_failed();
        if let Some(warning) = warning {
            self.note_error(&warning);
        }
        true
    }

    fn finish_auth_flow_error(&self, generation: u64, error: &str) {
        let current = {
            let mut s = self.shared.lock().unwrap();
            if s.auth.pending == Some(generation) && s.auth.generations.session == generation {
                s.auth.pending = None;
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
    /// re-authentication.
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
        // The account index lists this login either way; `persisted` says
        // whether a restart will find it in the keyring.
        self.with_index(|index| {
            index.upsert(
                &tokens.username,
                warning.is_none(),
                acquisition_store::now(),
            )
        });
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
    async fn valid_access_token(&self, force_refresh: bool) -> Result<(String, String), String> {
        enum Decision {
            Owner {
                id: u64,
                generations: AuthGenerations,
                refresh_token: String,
                result: watch::Sender<Option<AccessTokenResult>>,
            },
            Waiter(watch::Receiver<Option<AccessTokenResult>>),
        }

        let decision = {
            let mut s = self.shared.lock().unwrap();
            if !force_refresh
                && let (Some(token), Some(expires)) =
                    (&s.auth.access_token, s.auth.access_expires_at)
                && expires
                    .duration_since(self.choke.wall())
                    .is_ok_and(|left| left > Duration::from_secs(5))
            {
                return Ok((token.clone(), s.auth.username.clone().unwrap_or_default()));
            }
            let refresh_token = match &s.auth.refresh_token {
                Some(rt) => rt.clone(),
                None => return Err("not logged in — run `acq auth`".into()),
            };
            // A grant the provider already rejected is terminal (CONTEXT.md):
            // it is not sent again until login or logout replaces it.
            if let Some(cause) = self.rails().refresh_failed() {
                return Err(format!("token refresh disabled: {cause}; run `acq auth`"));
            }
            let generations = s.auth.generations;
            if let Some(flight) = &s.auth.refresh_flight
                && flight.generations == generations
            {
                Decision::Waiter(flight.result.subscribe())
            } else {
                let id = s.auth.next_refresh_flight;
                s.auth.next_refresh_flight = s.auth.next_refresh_flight.wrapping_add(1);
                let (result, _) = watch::channel(None);
                s.auth.refresh_flight = Some(RefreshFlight {
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
            }
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
                        s.auth.generations == generations
                            && s.auth.refresh_flight.as_ref().is_some_and(|f| f.id == id)
                    };
                    let cause = format!(
                        "refresh token rejected with HTTP {}",
                        error.status.unwrap_or_default()
                    );
                    if current && self.rails().mark_refresh_failed(&cause) {
                        self.note_error(&format!(
                            "AUTH: {cause}; further refreshes disabled until `acq auth` ({error})"
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
        id: u64,
        generations: AuthGenerations,
        refresh: Result<auth::TokenResponse, String>,
        sender: &watch::Sender<Option<AccessTokenResult>>,
    ) -> AccessTokenResult {
        let mut warning = None;
        let outcome = {
            let mut s = self.shared.lock().unwrap();
            let owns_current_flight = s
                .auth
                .refresh_flight
                .as_ref()
                .is_some_and(|flight| flight.id == id && flight.generations == generations);
            if !owns_current_flight || s.auth.generations != generations {
                Err(SESSION_CHANGED_DURING_REFRESH.into())
            } else {
                s.auth.refresh_flight = None;
                match refresh {
                    Ok(tokens) => {
                        let result = (tokens.access_token.clone(), tokens.username.clone());
                        warning = self.install_tokens_locked(&mut s.auth, tokens);
                        Ok(result)
                    }
                    Err(error) => Err(error),
                }
            }
        };
        sender.send_replace(Some(outcome.clone()));
        if outcome.is_ok() {
            self.log("access token refreshed");
        }
        if let Some(warning) = warning {
            self.note_error(&warning);
        }
        outcome
    }

    fn logout(&self) -> Result<(), String> {
        self.rails().clear_refresh_failed();
        let result = {
            let mut s = self.shared.lock().unwrap();
            let keyring = std::mem::take(&mut s.auth.keyring);
            let mut generations = s.auth.generations;
            generations.session = generations.session.wrapping_add(1);
            generations.access_token = generations.access_token.wrapping_add(1);
            generations.refresh_token = generations.refresh_token.wrapping_add(1);
            let username = s.auth.username.clone();
            let clear = match &username {
                Some(username) => self
                    .credential_store
                    .clear(self.provider.keyring_service, username),
                None => Ok(()),
            };
            if let Some(username) = &username {
                self.with_index(|index| index.set_persisted(username, false));
            }
            let next_refresh_flight = s.auth.next_refresh_flight;
            s.auth = AuthSession {
                keyring,
                generations,
                next_refresh_flight,
                ..AuthSession::default()
            };
            clear
        };
        self.log("logged out");
        result
    }

    fn auth_status(&self) -> Response {
        let s = self.shared.lock().unwrap();
        Response::Auth {
            logged_in: s.auth.refresh_token.is_some(),
            pending: s.auth.pending.is_some(),
            username: s.auth.username.clone(),
            access_expires_in_seconds: s.auth.access_expires_at.map(|t| {
                t.duration_since(self.choke.wall())
                    .unwrap_or_default()
                    .as_secs()
            }),
            keyring: s.auth.keyring.clone(),
            provider: self.provider.name.to_string(),
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

    async fn handle_request(
        self: &Arc<Self>,
        req: Request,
        events: &mut Option<broadcast::Receiver<JobInfo>>,
    ) -> Response {
        match req {
            Request::Hello { client_version } => {
                if client_version != VERSION {
                    self.log(&format!(
                        "version mismatch: client {client_version}, daemon {VERSION}"
                    ));
                }
                Response::Hello {
                    daemon_version: VERSION.to_string(),
                    pid: std::process::id(),
                    provider: self.provider.name.to_string(),
                }
            }
            Request::Submit {
                kind,
                params,
                priority,
                submitted_by,
            } => Response::Submitted {
                id: self.submit(kind, params, priority, submitted_by),
            },
            Request::Status { id } => match self.shared.lock().unwrap().snapshot(self, id) {
                Some(job) => Response::Status { job },
                None => Response::Error {
                    message: format!("no job {id}"),
                },
            },
            Request::Result { id } => {
                let s = self.shared.lock().unwrap();
                match s.jobs.get(&id) {
                    Some(e) => match &e.outcome {
                        Some(outcome) => Response::Result {
                            id,
                            outcome: outcome.clone(),
                        },
                        None => Response::Error {
                            message: format!("job {id} is still {}", e.info.state),
                        },
                    },
                    None => Response::Error {
                        message: format!("no job {id}"),
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
            Request::AuthCheck => match self.valid_access_token(true).await {
                Ok(_) => self.auth_status(),
                Err(message) => {
                    self.note_error(&format!("auth check failed: {message}"));
                    Response::Error { message }
                }
            },
            Request::AuthLogout => {
                if let Err(e) = self.logout() {
                    self.log(&format!("keyring clear failed: {e}"));
                }
                Response::Ack
            }
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
                    version: VERSION.to_string(),
                    provider: self.provider.name.to_string(),
                    uptime_seconds: s.started.elapsed().as_secs(),
                    connections: s.connections,
                    jobs_waiting: waiting,
                    jobs_running: running,
                    policies_known: self.choke.policy_statuses().len(),
                    in_flight,
                    max_in_flight,
                    rails: self.rails().status(),
                    keyring: s.auth.keyring.clone(),
                }
            }
            Request::DaemonStop => Response::Stopping,
            Request::ResetTripwire => {
                self.rails().reset_tripwire();
                self.log("live-test rails reset by request");
                Response::Ack
            }
            Request::Dashboard => {
                let s = self.shared.lock().unwrap();
                let (in_flight, max_in_flight) = self.choke.actual_send_occupancy();
                Response::Dashboard {
                    pid: std::process::id(),
                    version: VERSION.to_string(),
                    provider: self.provider.name.to_string(),
                    uptime_seconds: s.started.elapsed().as_secs(),
                    connections: s.connections,
                    logged_in: s.auth.refresh_token.is_some(),
                    username: s.auth.username.clone(),
                    access_expires_in_seconds: s.auth.access_expires_at.map(|t| {
                        t.duration_since(self.choke.wall())
                            .unwrap_or_default()
                            .as_secs()
                    }),
                    keyring: s.auth.keyring.clone(),
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

    async fn idle_watchdog(self: Arc<Self>) {
        let idle_shutdown = idle_shutdown_from_env();
        loop {
            tokio::time::sleep(IDLE_POLL).await;
            let idle = {
                let s = self.shared.lock().unwrap();
                let live_jobs = s.jobs.values().any(|e| !e.info.state.is_terminal());
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
    let path = socket_path();
    if path.exists() {
        // Live daemon or stale socket from a crash?
        if UnixStream::connect(&path).await.is_ok() {
            anyhow::bail!("daemon already running on {}", path.display());
        }
        std::fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;

    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())?;

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

    // Sessions survive daemon restarts through the keyring, one entry per
    // account; the account index says which entries to look for. One live
    // session for now (the most recent login); the others stay in the
    // keyring untouched. One unreadable entry never blocks the rest.
    let dir = store_dir(provider.name);
    let mut session = AuthSession {
        keyring: "ok".into(),
        ..AuthSession::default()
    };
    match Index::load(&dir) {
        Ok(index) => {
            for entry in index.persisted() {
                match auth::keyring_load(provider.keyring_service, &entry.username) {
                    Ok(Some(refresh_token)) if session.refresh_token.is_none() => {
                        session.refresh_token = Some(refresh_token);
                        session.username = Some(entry.username.clone());
                    }
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        writeln!(
                            &log,
                            "ACCOUNTS: index lists {} as persisted but the keyring has no entry",
                            entry.username
                        )
                        .ok();
                    }
                    Err(e) => {
                        session.keyring = format!("unavailable: {e}");
                        writeln!(
                            &log,
                            "ACCOUNTS: keyring read for {} failed: {e}",
                            entry.username
                        )
                        .ok();
                    }
                }
            }
        }
        Err(e) => {
            writeln!(&log, "ACCOUNTS: could not read {}: {e:#}", dir.display()).ok();
        }
    }

    let daemon = Arc::new(Daemon {
        shared: Mutex::new(Shared {
            jobs: HashMap::new(),
            next_id: 1,
            auth: session,
            connections: 0,
            last_activity: Instant::now(),
            started: Instant::now(),
            errors: VecDeque::new(),
            active_jobs: HashMap::new(),
        }),
        events: broadcast::channel(256).0,
        work: Notify::new(),
        log: Mutex::new(log),
        choke,
        provider,
        credential_store: Arc::new(OsCredentialStore),
        store_dir: Some(dir),
        store: Mutex::new(None),
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
        (s.auth.keyring.clone(), s.auth.username.clone())
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
                auth: AuthSession {
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
                },
                connections: 0,
                last_activity: Instant::now(),
                started: Instant::now(),
                errors: VecDeque::new(),
                active_jobs: HashMap::new(),
            }),
            events: broadcast::channel(16).0,
            work: Notify::new(),
            log: Mutex::new(log),
            choke: ChokePoint::new(),
            provider: Provider::mock(base),
            credential_store: credential_store.clone(),
            store_dir: None,
            store: Mutex::new(None),
        });
        (daemon, credential_store, log_path)
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
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(false).await });
        delayed.arrived.await.unwrap();
        let waiter_daemon = daemon.clone();
        let waiter = tokio::spawn(async move { waiter_daemon.valid_access_token(false).await });
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
        let before = daemon.shared.lock().unwrap().auth.generations;

        let owner_daemon = daemon.clone();
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(false).await });
        responses.first_arrived.await.unwrap();

        let mut waiters = Vec::new();
        for _ in 0..3 {
            let waiter_daemon = daemon.clone();
            waiters.push(tokio::spawn(async move {
                waiter_daemon.valid_access_token(false).await
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
            assert!(s.auth.refresh_flight.is_none());
            assert_eq!(s.auth.generations, before);
            assert_eq!(s.auth.refresh_token.as_deref(), Some("rt-old"));
            assert_eq!(s.auth.access_token.as_deref(), Some("at-expired"));
        }
        assert!(store.saves.lock().unwrap().is_empty());

        responses.release_first.send(()).unwrap();
        let retry = tokio::time::timeout(Duration::from_secs(2), daemon.valid_access_token(false))
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
        let before = daemon.shared.lock().unwrap().auth.generations;

        let owner_daemon = daemon.clone();
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(false).await });
        responses.first_arrived.await.unwrap();
        owner.abort();
        assert!(owner.await.unwrap_err().is_cancelled());
        {
            let s = daemon.shared.lock().unwrap();
            assert!(s.auth.refresh_flight.is_none());
            assert_eq!(s.auth.generations, before);
            assert_eq!(s.auth.refresh_token.as_deref(), Some("rt-old"));
            assert_eq!(s.auth.access_token.as_deref(), Some("at-expired"));
        }
        assert!(store.saves.lock().unwrap().is_empty());

        responses.release_first.send(()).unwrap();
        let retry = tokio::time::timeout(Duration::from_secs(2), daemon.valid_access_token(false))
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
        let before = daemon.shared.lock().unwrap().auth.generations;

        let refresh_daemon = daemon.clone();
        let refresh = tokio::spawn(async move { refresh_daemon.valid_access_token(false).await });
        delayed.arrived.await.unwrap();
        delayed.release.send(()).unwrap();
        assert_eq!(
            refresh.await.unwrap().unwrap(),
            ("at-rotated".into(), "rotated-user".into())
        );

        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.refresh_token.as_deref(), Some("rt-rotated"));
            assert_eq!(s.auth.access_token.as_deref(), Some("at-rotated"));
            assert_eq!(s.auth.generations.session, before.session);
            assert_ne!(s.auth.generations.access_token, before.access_token);
            assert_ne!(s.auth.generations.refresh_token, before.refresh_token);
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
        let before = daemon.shared.lock().unwrap().auth.generations;

        let owner_daemon = daemon.clone();
        let owner = tokio::spawn(async move { owner_daemon.valid_access_token(false).await });
        delayed.arrived.await.unwrap();
        let waiter_daemon = daemon.clone();
        let waiter = tokio::spawn(async move { waiter_daemon.valid_access_token(false).await });
        tokio::task::yield_now().await;
        delayed.release.send(()).unwrap();

        let owner_error = owner.await.unwrap().unwrap_err();
        let waiter_error = waiter.await.unwrap().unwrap_err();
        assert_eq!(waiter_error, owner_error);
        assert!(owner_error.contains("400 Bad Request"));
        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.generations, before);
            assert_eq!(s.auth.refresh_token.as_deref(), Some("rt-old"));
            assert_eq!(s.auth.access_token.as_deref(), Some("at-expired"));
            assert!(s.auth.refresh_flight.is_none());
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
        let refresh = tokio::spawn(async move { refresh_daemon.valid_access_token(false).await });
        delayed.arrived.await.unwrap();
        daemon.logout().unwrap();
        delayed.release.send(()).unwrap();

        assert_eq!(
            refresh.await.unwrap().unwrap_err(),
            SESSION_CHANGED_DURING_REFRESH
        );
        {
            let s = daemon.shared.lock().unwrap();
            assert!(s.auth.access_token.is_none());
            assert!(s.auth.refresh_token.is_none());
            assert!(s.auth.username.is_none());
        }
        assert!(store.cleared.load(Ordering::SeqCst));
        assert!(store.saves.lock().unwrap().is_empty());
        delayed.server.await.unwrap();
        remove_test_log(&log_path);
    }

    #[tokio::test]
    async fn reauthentication_during_refresh_keeps_new_session_and_rejects_old_completion() {
        let delayed =
            delayed_token_response("200 OK", token_body("at-stale", "rt-stale", "stale-user"))
                .await;
        let (daemon, store, log_path) = test_daemon(&delayed.base);

        let refresh_daemon = daemon.clone();
        let refresh = tokio::spawn(async move { refresh_daemon.valid_access_token(false).await });
        delayed.arrived.await.unwrap();
        let flow_generation = daemon.begin_auth_flow();
        assert!(daemon.finish_auth_flow(
            flow_generation,
            tokens("at-reauth", "rt-reauth", "reauth-user")
        ));
        delayed.release.send(()).unwrap();

        assert_eq!(
            refresh.await.unwrap().unwrap_err(),
            SESSION_CHANGED_DURING_REFRESH
        );
        {
            let s = daemon.shared.lock().unwrap();
            assert_eq!(s.auth.access_token.as_deref(), Some("at-reauth"));
            assert_eq!(s.auth.refresh_token.as_deref(), Some("rt-reauth"));
            assert_eq!(s.auth.username.as_deref(), Some("reauth-user"));
            assert_eq!(s.auth.generations.session, flow_generation);
        }
        assert_eq!(
            store.saves.lock().unwrap().as_slice(),
            [(
                "acquisition-playground".into(),
                "rt-reauth".into(),
                "reauth-user".into()
            )]
        );
        delayed.server.await.unwrap();
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

        fn clear(&self, _service: &str, _username: &str) -> Result<(), String> {
            Ok(())
        }
    }

    struct ScriptedResponse {
        method: &'static str,
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
                assert_eq!(request.path, "/fetch");
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
                auth: AuthSession::default(),
                connections: 0,
                last_activity: Instant::now(),
                started: Instant::now(),
                errors: VecDeque::new(),
                active_jobs: HashMap::new(),
            }),
            events: broadcast::channel(256).0,
            work: Notify::new(),
            log: Mutex::new(log),
            choke: ChokePoint::with_clock_and_rails(clock, rails),
            provider,
            credential_store,
            store_dir: None,
            store: Mutex::new(None),
        });
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
            daemon.install_tokens_locked(
                &mut s.auth,
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
        let id = daemon.submit("characters".into(), json!({}), 0, "test".into());
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
            shared.auth.refresh_token = Some("rt-old".into());
            shared.auth.username = Some("old-user".into());
        }
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let first = daemon.submit("whoami".into(), json!({}), 0, "test".into());
        let second = daemon.submit("whoami".into(), json!({}), 0, "test".into());

        arrived.await.unwrap();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                let waiters = daemon
                    .shared
                    .lock()
                    .unwrap()
                    .auth
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

        let independent =
            daemon.submit("sleep".into(), json!({ "seconds": 0.0 }), 0, "test".into());
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
        let characters = daemon.submit("characters".into(), json!({}), 0, "test".into());
        let stashes = daemon.submit(
            "stashes".into(),
            json!({ "league": "Standard" }),
            0,
            "test".into(),
        );
        clock.wait_for_sleepers(2).await;

        let independent =
            daemon.submit("sleep".into(), json!({ "seconds": 0.0 }), 0, "test".into());
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
            shared.auth.refresh_token = Some("rt-old".into());
            shared.auth.username = Some("old-user".into());
            shared.auth.access_token = Some("at-established".into());
            shared.auth.access_expires_at = Some(clock.wall() + Duration::from_secs(3600));
        }

        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        // Establish every authenticated route before expiry so the refresh
        // phase is not serialized behind the one-at-a-time probe key.
        let established_routes = [
            daemon.submit("characters".into(), json!({}), 0, "test".into()),
            daemon.submit(
                "stashes".into(),
                json!({ "league": "Standard" }),
                0,
                "test".into(),
            ),
            daemon.submit(
                "stash".into(),
                json!({ "league": "Standard", "id": "cur1" }),
                0,
                "test".into(),
            ),
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
            shared.auth.access_expires_at = Some(clock.wall());
        }

        // These jobs now have different learned scheduling keys, so all three
        // can enter valid_access_token while the localhost token body is held.
        let characters = daemon.submit("characters".into(), json!({}), 0, "test".into());
        let stashes = daemon.submit(
            "stashes".into(),
            json!({ "league": "Standard" }),
            0,
            "test".into(),
        );
        let stash = daemon.submit(
            "stash".into(),
            json!({ "league": "Standard", "id": "cur1" }),
            0,
            "test".into(),
        );
        let fetches: Vec<_> = (0..7)
            .map(|sequence| {
                daemon.submit(
                    "fetch".into(),
                    json!({ "sequence": sequence }),
                    0,
                    "test".into(),
                )
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
                shared.auth.refresh_token.clone().unwrap(),
                shared.auth.access_token.clone().unwrap(),
                shared.auth.refresh_flight.is_some(),
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
        let low = daemon.submit("fetch".into(), json!({}), 1, "test".into());
        let high = daemon.submit("fetch".into(), json!({}), 9, "test".into());

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
                status: "204 No Content",
                headers: policy.into(),
                body: String::new(),
            },
            ScriptedResponse {
                method: "GET",
                status: "503 Service Unavailable",
                headers: String::new(),
                body: "<html><center>openresty</center></html>".into(),
            },
            ScriptedResponse {
                method: "GET",
                status: "200 OK",
                headers: policy.replace("0:10:0", "1:10:0"),
                body: r#"{"items":["done"]}"#.into(),
            },
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) = test_daemon(&base, clock);
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());
        let first = daemon.submit("fetch".into(), json!({}), 0, "test".into());
        let second = daemon.submit("fetch".into(), json!({}), 0, "test".into());

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
        let id = daemon.submit("fetch".into(), json!({}), 0, "test".into());

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
        let id = daemon.submit("fetch".into(), json!({}), 0, "test".into());

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
        let first = daemon.submit("fetch".into(), json!({}), 0, "test".into());
        let second = daemon.submit("fetch".into(), json!({}), 0, "test".into());

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
            let id = daemon.submit("fetch".into(), json!({}), 0, "test".into());

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
        let id = daemon.submit("fetch".into(), json!({}), 0, "test".into());

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
            let id = daemon.submit("fetch".into(), json!({}), 0, "test".into());

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

    #[tokio::test]
    async fn tripped_daemon_fails_queued_jobs_without_sending_until_reset() {
        // HEAD establishes, the first GET lands a 429 (tripping), and after
        // the reset one more GET succeeds. Nothing else may reach the server.
        let responses = vec![
            ScriptedResponse::full("HEAD", "204 No Content", None, ""),
            ScriptedResponse::full("GET", "429 Too Many Requests", Some(0), "{}"),
            ScriptedResponse::full("GET", "200 OK", None, r#"{"ok":true}"#),
        ];
        let (base, requests, server) = scripted_server(responses).await;
        let clock = Arc::new(ManualClock::new());
        let (daemon, log_path) =
            test_daemon_with(Provider::mock(&base), clock.clone(), tripwire_config());
        let rails = daemon.choke.rails().clone();
        let dispatcher = tokio::spawn(daemon.clone().dispatcher());

        let first = daemon.submit("fetch".into(), json!({}), 0, "test".into());
        let (info, outcome) = wait_terminal(&daemon, first).await;
        assert_eq!(info.state, JobState::Failed);
        let Outcome::Failure { error } = outcome else {
            panic!("tripped job did not fail")
        };
        assert!(error.contains("halted by live-test rails"), "{error}");
        assert!(
            rails.halted().unwrap().contains("429 on GET /fetch"),
            "{:?}",
            rails.halted()
        );
        assert_eq!(
            requests.lock().unwrap().as_slice(),
            ["HEAD", "GET"],
            "the 429 tripped before any retry"
        );
        assert_eq!(
            journal_waits(&log_path),
            [0, 0],
            "a requeued job fails at its next attempt instead of waiting behind the hold"
        );

        let second = daemon.submit("fetch".into(), json!({}), 0, "test".into());
        let (info, _) = wait_terminal(&daemon, second).await;
        assert_eq!(info.state, JobState::Failed);
        assert_eq!(
            requests.lock().unwrap().len(),
            2,
            "a tripped daemon sends nothing"
        );
        let logged = {
            let s = daemon.shared.lock().unwrap();
            s.errors
                .iter()
                .any(|(_, m)| m.contains("LIVE-TEST RAILS TRIPPED"))
        };
        assert!(logged, "the trip is announced once in the error ring");

        rails.reset_tripwire();
        // The limiter's own hold from the 429 still applies after the reset.
        let third = daemon.submit("fetch".into(), json!({}), 0, "test".into());
        let (info, _) = wait_terminal(&daemon, third).await;
        assert_eq!(info.state, JobState::Done);
        assert_eq!(requests.lock().unwrap().as_slice(), ["HEAD", "GET", "GET"]);
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

        let first = daemon.submit("fetch".into(), json!({}), 0, "test".into());
        let (info, _) = wait_terminal(&daemon, first).await;
        assert_eq!(
            info.state,
            JobState::Done,
            "HEAD + GET reach the ceiling exactly"
        );
        assert!(rails.halted().unwrap().contains("ceiling: 2 of 2"));

        let second = daemon.submit("fetch".into(), json!({}), 0, "test".into());
        let (info, outcome) = wait_terminal(&daemon, second).await;
        assert_eq!(info.state, JobState::Failed);
        let Outcome::Failure { error } = outcome else {
            panic!("ceiling job did not fail")
        };
        assert!(error.contains("ceiling"), "{error}");
        assert_eq!(requests.lock().unwrap().len(), 2);
        assert_eq!(rails.status().sends, 2);
        server.await.unwrap();
        finish_harness_wire(dispatcher, &log_path, &requests);
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
        s.auth.refresh_token = Some("rt-old".into());
        s.auth.username = Some("test-user".into());
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

        let first = daemon.valid_access_token(false).await.unwrap_err();
        assert!(first.contains("400 Bad Request"), "{first}");
        assert_eq!(requests.lock().unwrap().len(), 1);
        assert!(rails.refresh_failed().unwrap().contains("HTTP 400"));

        let second = daemon.valid_access_token(false).await.unwrap_err();
        assert!(
            second.contains("token refresh disabled: refresh token rejected with HTTP 400"),
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
                s.auth.refresh_token.as_deref(),
                Some("rt-old"),
                "nothing is deleted"
            );
            assert!(s.auth.refresh_flight.is_none());
        }

        daemon.logout().unwrap_or(());
        assert_eq!(rails.refresh_failed(), None, "logout clears the mark");
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

        let first = daemon.valid_access_token(false).await.unwrap_err();
        assert!(first.contains("503"), "{first}");
        assert_eq!(rails.refresh_failed(), None, "5xx is not a rejected grant");
        // The tripwire did trip on the 503 (Cloudflare shape); clear it so
        // the retry can show the refresh path itself is untouched.
        assert!(rails.halted().unwrap().contains("503"));
        rails.reset_tripwire();

        let (token, user) = daemon.valid_access_token(false).await.unwrap();
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
            wait: Duration::ZERO,
        });
        logged_in(&mut daemon);
        let error = daemon.valid_access_token(false).await.unwrap_err();
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
            .execute_inner(1, "whoami", json!({}), None, daemon.choke.now())
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
}
