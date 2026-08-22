//! The daemon: single job queue, single worker, JSON-lines Unix socket server.
//!
//! Lifecycle follows the gpg-agent model: clients spawn it on demand, it exits
//! on its own after a stretch with no connections and no live jobs.

use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::Result;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, UnixListener, UnixStream};
use tokio::sync::{Notify, broadcast};

use std::collections::VecDeque;

use crate::VERSION;
use crate::job::{JobId, JobInfo, JobState, Outcome, Priority};
use crate::protocol::{ErrorRecord, Request, Response};
use crate::provider::{CALLBACK_PATH, Provider, SCOPES, ggg_mode};
use crate::ratelimit::{ChokePoint, EndpointState, Paid, SendError, url_path};
use crate::{auth, mockggg};

const IDLE_SHUTDOWN: Duration = Duration::from_secs(60);
const IDLE_POLL: Duration = Duration::from_secs(5);
const ERROR_HISTORY: usize = 50;
/// Probes outrank everything: every job on that route is waiting on one.
const PROBE_PRIORITY: Priority = u8::MAX;
/// The global burst bound (ground truth P-B). Policies count independently
/// (N6, N7), so the header layer alone would let one request per policy go
/// out at the same instant; Cloudflare in front of it watches bursts across
/// everything (N1, N2) and compliant traffic is slow anyway (N4), so a small
/// cap costs nothing and keeps that layer invisible. On top of this cap: at
/// most one request in flight per policy (the limiter's lookback assumes
/// responses arrive before the next decision) and at most one probe in
/// flight, ever (N18).
pub const MAX_IN_FLIGHT: usize = 2;
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

/// What a network call can fail with; 429 is its own arm so the job can
/// be re-queued rather than failed.
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

struct Entry {
    info: JobInfo,
    params: Value,
    outcome: Option<Outcome>,
    cancel_requested: bool,
    /// A parent's own result, held back until its descendants finish. Set
    /// means "running, waiting on children, not holding a slot".
    deferred: Option<Outcome>,
}

#[derive(Default)]
struct AuthSession {
    access_token: Option<String>,
    access_expires_at: Option<Instant>,
    refresh_token: Option<String>,
    username: Option<String>,
    /// A login flow is waiting on the browser.
    pending: bool,
    /// "ok" or an error description shown in `auth status`.
    keyring: String,
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
    /// Jobs holding an in-flight slot → the key they serialize on.
    in_flight: HashMap<JobId, String>,
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
        ids.into_iter().filter_map(|id| self.snapshot(daemon, id)).collect()
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

    /// The route label and URL a job sends on, if it sends at all. Routes,
    /// not URLs, key the limiter: every league shares `stash-list`. `fetch`
    /// is a fake data endpoint that exists only on the mock.
    fn route_for(&self, kind: &str, params: &Value) -> Option<(String, String)> {
        let base = &self.provider.api_base;
        match kind {
            "characters" => Some(("character-list".into(), format!("{base}/character"))),
            "stashes" => {
                let league = params.get("league").and_then(Value::as_str).unwrap_or("Standard");
                Some(("stash-list".into(), format!("{base}/stash/{league}")))
            }
            // One tab, or one substash of a map/unique tab: same route, same
            // policy (stash-request-limit), one probe for all of them.
            "stash" => {
                let league = params.get("league").and_then(Value::as_str).unwrap_or("Standard");
                let id = params.get("id").and_then(Value::as_str)?;
                let url = match params.get("sub").and_then(Value::as_str) {
                    Some(sub) => format!("{base}/stash/{league}/{id}/{sub}"),
                    None => format!("{base}/stash/{league}/{id}"),
                };
                Some(("stash".into(), url))
            }
            "refresh" => {
                let league = params.get("league").and_then(Value::as_str).unwrap_or("Standard");
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
            self.log(&format!("route {route} unknown; probing {} first", url_path(url)));
            self.submit(
                "probe".into(),
                json!({ "route": route, "url": url }),
                PROBE_PRIORITY,
                "daemon".into(),
            );
        }
    }

    fn submit(&self, kind: String, params: Value, priority: Priority, submitted_by: String) -> JobId {
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
                    s.jobs.values().filter(|e| e.info.parent == Some(p)).map(|e| e.info.id),
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

    /// Hands out in-flight slots under the P-B rules and runs each picked
    /// job on its own task. Woken by submits, completions, and reprioritization.
    async fn dispatcher(self: Arc<Self>) {
        loop {
            let picks = self.pick_runnable();
            for id in picks {
                tokio::spawn(self.clone().run_slot(id));
            }
            self.work.notified().await;
        }
    }

    /// Waiting jobs, in dispatch order, that can take a slot right now.
    fn pick_runnable(&self) -> Vec<JobId> {
        let mut s = self.shared.lock().unwrap();
        let mut busy: HashSet<String> = s.in_flight.values().cloned().collect();
        let mut picks = Vec::new();
        for id in s.queue_order() {
            if s.in_flight.len() + picks.len() >= MAX_IN_FLIGHT {
                break;
            }
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
            s.in_flight.insert(*id, key.clone());
        }
        picks.into_iter().map(|(id, _)| id).collect()
    }

    async fn run_slot(self: Arc<Self>, id: JobId) {
        self.process(id).await;
        self.shared.lock().unwrap().in_flight.remove(&id);
        self.work.notify_one();
    }

    async fn process(&self, id: JobId) {
        let route = {
            let s = self.shared.lock().unwrap();
            match s.jobs.get(&id) {
                Some(e) => self.route_for(&e.info.kind, &e.params),
                None => return,
            }
        };

        // A route we've never heard from gets a probe first (N16, N24); a
        // degraded one (N20) fails its jobs cleanly until the cooldown ends.
        if let Some((route, url)) = &route {
            match self.choke.endpoint_state(route) {
                EndpointState::Unknown => {
                    self.ensure_probe(route, url);
                    return; // slot released; the probe outranks us
                }
                EndpointState::Degraded { until, reason } => {
                    let left = until.saturating_duration_since(Instant::now()).as_secs();
                    let error = format!("route {route} is degraded for another {left}s: {reason}");
                    self.note_error(&format!("job {id}: {error}"));
                    self.start_and_finish(id, Outcome::Failure { error });
                    return;
                }
                EndpointState::Policy(_) | EndpointState::Policyless => {}
            }
        }

        // Rate-limit wait happens while the job is still `waiting`, in short
        // slices so cancellation and reprioritization stay responsive. The
        // receipt is handed to `execute`, which spends it on the actual send.
        let mut paid: Option<Paid> = None;
        if let Some((route, _)) = &route {
            loop {
                let step = {
                    let s = self.shared.lock().unwrap();
                    let Some(me) = s.jobs.get(&id) else { return };
                    if me.info.state != JobState::Waiting {
                        return; // cancelled out from under us
                    }
                    // A higher-priority job on the same key may have arrived;
                    // give the slot back so the dispatcher picks it instead.
                    let my_key = self.serial_key(me);
                    let outranked = s.queue_order().into_iter().take_while(|&q| q != id).any(|q| {
                        s.jobs.get(&q).is_some_and(|e| self.serial_key(e) == my_key)
                    });
                    if outranked {
                        return;
                    }
                    self.choke.try_take(route)
                };
                match step {
                    Ok(receipt) => {
                        paid = Some(receipt);
                        break;
                    }
                    Err(d) => tokio::time::sleep(d.min(Duration::from_secs(1))).await,
                }
            }
        }

        let job = {
            let mut s = self.shared.lock().unwrap();
            let Some(entry) = s.jobs.get_mut(&id) else { return };
            if entry.info.state != JobState::Waiting {
                return;
            }
            entry.info.state = JobState::Running;
            (entry.info.clone(), entry.params.clone())
        };
        let (info, params) = job;
        let kind = info.kind.clone();
        self.emit(info);

        let outcome = match self.execute(id, &kind, params, paid).await {
            Exec::Done(outcome) => outcome,
            Exec::RateLimited(evidence) => {
                // P-A: a 429 is recovered from, not surfaced — unless it keeps
                // happening. The limiter already holds the policy for
                // Retry-After + bucket (N19); putting the job back to waiting
                // (it keeps its place: order is priority, then id) makes it
                // go out after that hold, with the ETA visible meanwhile.
                let requeued = {
                    let mut s = self.shared.lock().unwrap();
                    let Some(entry) = s.jobs.get_mut(&id) else { return };
                    if entry.info.retries < MAX_429_RETRIES && !entry.cancel_requested {
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
                        return; // slot released; the dispatcher re-picks after the hold
                    }
                    None => Outcome::Failure {
                        error: format!("rate limited (429) {MAX_429_RETRIES} times; giving up (N10): {evidence}"),
                    },
                }
            }
        };
        if let Outcome::Failure { error } = &outcome {
            self.note_error(&format!("job {id} ({kind}): {error}"));
        }
        // A job that spawned children holds its own result until they're
        // all done. It gives its slot back (we return) so children can run.
        let has_children = {
            let mut s = self.shared.lock().unwrap();
            let spawned = s.jobs.values().any(|e| e.info.parent == Some(id));
            if spawned && let Some(entry) = s.jobs.get_mut(&id) && entry.info.state == JobState::Running {
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
            let Some(parent) = s.jobs.get(&pid) else { return };
            if parent.deferred.is_none() {
                return;
            }
            let children: Vec<&Entry> = s.jobs.values().filter(|e| e.info.parent == Some(pid)).collect();
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
                    error: format!("{failed} of {total} child jobs failed: {failed_ids:?} (acq result <id> for each)"),
                },
                other => other,
            }
        };
        self.finish(pid, final_outcome);
    }

    fn finish(&self, id: JobId, outcome: Outcome) {
        let info = {
            let mut s = self.shared.lock().unwrap();
            let Some(entry) = s.jobs.get_mut(&id) else { return };
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
            let Some(entry) = s.jobs.get_mut(&id) else { return };
            if entry.info.state != JobState::Waiting {
                return;
            }
            entry.info.state = JobState::Running;
            entry.info.clone()
        };
        self.emit(info);
        self.finish(id, outcome);
    }

    async fn execute(&self, id: JobId, kind: &str, params: Value, paid: Option<Paid>) -> Exec {
        // Network kinds bubble a 429 up as `Exec::RateLimited`; everything
        // else is an ordinary outcome.
        match self.execute_inner(id, kind, params, paid).await {
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
        paid: Option<Paid>,
    ) -> Result<Outcome, ApiError> {
        Ok(match kind {
            // The one real API call: GET {api_base}/character. Same code in
            // both modes; only the provider's base URL differs. No retries on
            // any failure — a 429 or a Cloudflare-shaped block is reported,
            // never fought through (invariants 2 and 3 are read-only so far:
            // headers are logged and returned, not yet fed to the limiter).
            "characters" => {
                let paid = paid.expect("characters is a network kind");
                let (token, username) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let url = format!("{}/character", self.provider.api_base);
                let (v, rate) = self.api_get(paid, &url, Some(&token)).await?;
                Outcome::Success {
                    payload: json!({
                        "provider": self.provider.name,
                        "username": username,
                        "characters": v.get("characters").cloned().unwrap_or(v),
                        "rate_limit": rate,
                    }),
                }
            }
            // The stash list: one request under stash-list-request-limit, the
            // second real policy. Tab contents are a later step.
            "stashes" => {
                let paid = paid.expect("stashes is a network kind");
                let (token, username) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let league = params.get("league").and_then(Value::as_str).unwrap_or("Standard").to_string();
                let url = format!("{}/stash/{league}", self.provider.api_base);
                let (v, rate) = self.api_get(paid, &url, Some(&token)).await?;
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
                let paid = paid.expect("stash is a network kind");
                let (token, _) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let Some((_, url)) = self.route_for("stash", &params) else {
                    return Ok(Outcome::Failure { error: "stash needs an id".into() });
                };
                let (v, rate) = self.api_get(paid, &url, Some(&token)).await?;
                let stash = v.get("stash").cloned().unwrap_or(v);
                // Map/unique tabs carry their substashes as stubs; following
                // them is opt-in per tab (--deep) because one map tab can
                // hold hundreds. Each substash becomes a child job.
                let deep = params.get("deep").and_then(Value::as_bool).unwrap_or(false);
                let children = stash.get("children").and_then(Value::as_array).cloned().unwrap_or_default();
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
                let paid = paid.expect("refresh is a network kind");
                let (token, _) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Ok(Outcome::Failure { error }),
                };
                let league = params.get("league").and_then(Value::as_str).unwrap_or("Standard").to_string();
                let deep = params.get("deep").and_then(Value::as_bool).unwrap_or(false);
                let all = params.get("all").and_then(Value::as_bool).unwrap_or(false);
                let wanted: Vec<String> = params
                    .get("tabs")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
                    .unwrap_or_default();
                if !all && wanted.is_empty() {
                    return Ok(Outcome::Failure { error: "refresh needs --all or --tabs <id,...>".into() });
                }
                let url = format!("{}/stash/{league}", self.provider.api_base);
                let (v, rate) = self.api_get(paid, &url, Some(&token)).await?;
                let listed = v.get("stashes").and_then(Value::as_array).cloned().unwrap_or_default();
                // Flatten: top-level tabs plus folder children; skip folders.
                let mut tabs: Vec<(String, String, String)> = Vec::new();
                for t in &listed {
                    let ty = t.get("type").and_then(Value::as_str).unwrap_or("");
                    if ty == "Folder" {
                        for c in t.get("children").and_then(Value::as_array).into_iter().flatten() {
                            if let Some(cid) = c.get("id").and_then(Value::as_str) {
                                tabs.push((cid.into(), c.get("name").and_then(Value::as_str).unwrap_or("").into(), c.get("type").and_then(Value::as_str).unwrap_or("").into()));
                            }
                        }
                    } else if let Some(tid) = t.get("id").and_then(Value::as_str) {
                        tabs.push((tid.into(), t.get("name").and_then(Value::as_str).unwrap_or("").into(), ty.into()));
                    }
                }
                let selected: Vec<&(String, String, String)> =
                    tabs.iter().filter(|(tid, _, _)| all || wanted.contains(tid)).collect();
                let unknown: Vec<&String> = wanted.iter().filter(|w| !tabs.iter().any(|(tid, _, _)| tid == *w)).collect();
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
                let Some(paid) = paid else {
                    return Ok(Outcome::Failure {
                        error: "fetch is a mock-only kind; real mode has no fake data endpoint".into(),
                    });
                };
                let url = format!("{}/fetch", self.provider.api_base);
                let (v, rate) = self.api_get(paid, &url, None).await?;
                Outcome::Success {
                    payload: json!({
                        "note": "fake data from the in-process mock",
                        "params": params,
                        "items": v.get("items").cloned().unwrap_or(v),
                        "rate_limit": rate,
                    }),
                }
            }
            "profile" => {
                // The auth-required kind: exercises access-token expiry and
                // silent refresh through the daemon-owned session.
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
                    params.get("route").and_then(Value::as_str).map(str::to_string),
                    params.get("url").and_then(Value::as_str).map(str::to_string),
                ) else {
                    return Ok(Outcome::Failure { error: "probe needs a route and a url".into() });
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
                let paid = loop {
                    match self.choke.try_take(&route) {
                        Ok(p) => break p,
                        Err(wait) => tokio::time::sleep(wait.min(Duration::from_secs(1))).await,
                    }
                    if self.cancelled(id) {
                        return Ok(Outcome::Cancelled);
                    }
                };
                match self.choke.head(paid, &url, bearer.as_deref()).await {
                    Ok((status, policy, headers)) => {
                        let name = policy.name;
                        self.log(&format!("HEAD {} -> {status} | policy {name} | {headers}", url_path(&url)));
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
                error: format!("unknown job kind '{other}' (kinds: sleep, fetch, profile, characters, stashes, stash, refresh, probe)"),
            },
        })
    }

    /// One rate-limited GET: sends with the receipt, logs the rate headers,
    /// and turns non-2xx into a typed error. A 429 is distinguishable so the
    /// caller can re-queue the job behind the limiter's hold (P-A); a
    /// Cloudflare-shaped 403/503 is never retried (invariant 3).
    async fn api_get(
        &self,
        paid: Paid,
        url: &str,
        bearer: Option<&str>,
    ) -> Result<(Value, Value), ApiError> {
        let response = match bearer {
            Some(token) => self.choke.get_bearer(paid, url, token).await,
            None => self.choke.get(paid, url).await,
        }
        .map_err(|error| match error {
            SendError::Protocol(error) => ApiError::Protocol(format!(
                "GET {}: rate-limit protocol failure: {error}",
                url_path(url)
            )),
            SendError::Transport(error) => ApiError::Other(format!("GET {url} failed: {error}")),
        })?;
        let status = response.status();
        let rate = crate::ratelimit::rate_limit_snapshot(response.headers());
        let path = url_path(url);
        self.log(&format!("GET {path} -> {status} | rate headers: {rate}"));
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            let body: String = body.chars().take(300).collect();
            let evidence = format!("GET {path} returned {status}; rate headers {rate}; body: {body}");
            return Err(match status.as_u16() {
                429 => ApiError::RateLimited(evidence),
                403 | 503 => ApiError::Other(format!("{evidence} — possibly a Cloudflare block; NOT retrying (invariant 3)")),
                _ => ApiError::Other(evidence),
            });
        }
        serde_json::from_str::<Value>(&body)
            .map(|v| (v, rate))
            .map_err(|e| ApiError::Other(format!("bad JSON from {path}: {e}")))
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
        self.shared.lock().unwrap().auth.pending = true;
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
                            daemon.install_tokens(tokens);
                            // A probe that failed for lack of a session would
                            // succeed now; don't make the user sit out the cooldown.
                            daemon.choke.forget_degraded();
                            daemon.log(&format!("logged in as {user}"));
                        }
                        Err(e) => daemon.note_error(&format!("token exchange failed: {e}")),
                    }
                }
                Ok(Err(e)) => daemon.note_error(&format!("auth callback failed: {e}")),
                Err(_) => daemon.note_error("auth flow timed out after 300s"),
            }
            daemon.shared.lock().unwrap().auth.pending = false;
        });
        Ok(authorize_url)
    }

    /// Store fresh tokens in memory and mirror the refresh token (which the
    /// provider rotates on every grant) into the keyring.
    fn install_tokens(&self, tokens: auth::TokenResponse) {
        let keyring = match auth::keyring_save(
            self.provider.keyring_service,
            &tokens.refresh_token,
            &tokens.username,
        ) {
            Ok(()) => "ok".to_string(),
            Err(e) => {
                self.note_error(&format!("keyring save failed: {e} (session is in-memory only)"));
                format!("unavailable: {e}")
            }
        };
        let mut s = self.shared.lock().unwrap();
        s.auth.access_token = Some(tokens.access_token);
        s.auth.access_expires_at = Some(Instant::now() + Duration::from_secs(tokens.expires_in));
        s.auth.refresh_token = Some(tokens.refresh_token);
        s.auth.username = Some(tokens.username);
        s.auth.keyring = keyring;
    }

    /// Current access token, refreshing through the provider if it is
    /// expired (or about to be). Jobs call this; clients never see tokens.
    /// `force_refresh` skips the cached token so the provider round-trip is
    /// guaranteed — that's what makes `auth check` an actual proof.
    async fn valid_access_token(&self, force_refresh: bool) -> Result<(String, String), String> {
        let refresh_token = {
            let s = self.shared.lock().unwrap();
            if !force_refresh
                && let (Some(token), Some(expires)) = (&s.auth.access_token, s.auth.access_expires_at)
                && expires.saturating_duration_since(Instant::now()) > Duration::from_secs(5)
            {
                return Ok((token.clone(), s.auth.username.clone().unwrap_or_default()));
            }
            match &s.auth.refresh_token {
                Some(rt) => rt.clone(),
                None => return Err("not logged in — run `acq auth`".into()),
            }
        };
        // May wait on the token endpoint's limiter; the shared lock is not
        // held here, so a limited refresh delays only its own caller.
        let tokens = auth::refresh(&self.choke, &self.provider, &refresh_token)
            .await
            .map_err(|e| format!("token refresh failed: {e}"))?;
        self.log("access token refreshed");
        let result = (tokens.access_token.clone(), tokens.username.clone());
        self.install_tokens(tokens);
        Ok(result)
    }

    fn auth_status(&self) -> Response {
        let s = self.shared.lock().unwrap();
        Response::Auth {
            logged_in: s.auth.refresh_token.is_some(),
            pending: s.auth.pending,
            username: s.auth.username.clone(),
            access_expires_in_seconds: s
                .auth
                .access_expires_at
                .map(|t| t.saturating_duration_since(Instant::now()).as_secs()),
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
            Request::Submit { kind, params, priority, submitted_by } => Response::Submitted {
                id: self.submit(kind, params, priority, submitted_by),
            },
            Request::Status { id } => match self.shared.lock().unwrap().snapshot(self, id) {
                Some(job) => Response::Status { job },
                None => Response::Error { message: format!("no job {id}") },
            },
            Request::Result { id } => {
                let s = self.shared.lock().unwrap();
                match s.jobs.get(&id) {
                    Some(e) => match &e.outcome {
                        Some(outcome) => Response::Result { id, outcome: outcome.clone() },
                        None => Response::Error {
                            message: format!("job {id} is still {}", e.info.state),
                        },
                    },
                    None => Response::Error { message: format!("no job {id}") },
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
                {
                    let mut s = self.shared.lock().unwrap();
                    let keyring = std::mem::take(&mut s.auth.keyring);
                    s.auth = AuthSession { keyring, ..AuthSession::default() };
                }
                if let Err(e) = auth::keyring_clear(self.provider.keyring_service) {
                    self.log(&format!("keyring clear failed: {e}"));
                }
                self.log("logged out");
                Response::Ack
            }
            Request::DaemonStatus => {
                let s = self.shared.lock().unwrap();
                let (waiting, running) = s.jobs.values().fold((0, 0), |(w, r), e| {
                    match e.info.state {
                        JobState::Waiting => (w + 1, r),
                        JobState::Running => (w, r + 1),
                        _ => (w, r),
                    }
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
                    in_flight: s.in_flight.len(),
                    max_in_flight: MAX_IN_FLIGHT,
                }
            }
            Request::DaemonStop => Response::Stopping,
            Request::Dashboard => {
                let s = self.shared.lock().unwrap();
                Response::Dashboard {
                    pid: std::process::id(),
                    version: VERSION.to_string(),
                    provider: self.provider.name.to_string(),
                    uptime_seconds: s.started.elapsed().as_secs(),
                    connections: s.connections,
                    logged_in: s.auth.refresh_token.is_some(),
                    username: s.auth.username.clone(),
                    access_expires_in_seconds: s
                        .auth
                        .access_expires_at
                        .map(|t| t.saturating_duration_since(Instant::now()).as_secs()),
                    keyring: s.auth.keyring.clone(),
                    in_flight: s.in_flight.len(),
                    max_in_flight: MAX_IN_FLIGHT,
                    policies: self.choke.policy_statuses(),
                    policyless_endpoints: self.choke.policyless_endpoints(),
                    degraded_endpoints: self.choke.degraded_endpoints(),
                    jobs: s.list(self),
                    sends: self.choke.recent_sends(),
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
        loop {
            tokio::time::sleep(IDLE_POLL).await;
            let idle = {
                let s = self.shared.lock().unwrap();
                let live_jobs = s
                    .jobs
                    .values()
                    .any(|e| !e.info.state.is_terminal());
                s.connections == 0 && !live_jobs && s.last_activity.elapsed() >= IDLE_SHUTDOWN
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

/// Wait for the browser to hit the loopback redirect with an auth code.
/// Ignores stray requests (favicons etc.) until `/callback` arrives.
async fn wait_callback(listener: TcpListener, expected_state: &str) -> Result<String, String> {
    loop {
        let (mut stream, _) = listener.accept().await.map_err(|e| e.to_string())?;
        let Some(req) = mockggg::read_request(&mut stream).await else { continue };
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
                mockggg::respond(&mut stream, "400 Bad Request", "text/plain", "state mismatch").await;
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
    let choke = ChokePoint::new();

    // A session in the keyring survives daemon restarts; the first
    // auth-required job will refresh its way to a live access token.
    let mut session = AuthSession::default();
    match auth::keyring_load(provider.keyring_service) {
        Ok(Some((refresh_token, username))) => {
            session.refresh_token = Some(refresh_token);
            session.username = Some(username);
            session.keyring = "ok".into();
        }
        Ok(None) => session.keyring = "ok".into(),
        Err(e) => session.keyring = format!("unavailable: {e}"),
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
            in_flight: HashMap::new(),
        }),
        events: broadcast::channel(256).0,
        work: Notify::new(),
        log: Mutex::new(log),
        choke,
        provider,
    });

    daemon.log(&format!(
        "daemon {} listening on {} (pid {})",
        VERSION,
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

    tokio::spawn(daemon.clone().dispatcher());
    tokio::spawn(daemon.clone().idle_watchdog());

    loop {
        let (stream, _addr) = listener.accept().await?;
        tokio::spawn(daemon.clone().handle_conn(stream));
    }
}
