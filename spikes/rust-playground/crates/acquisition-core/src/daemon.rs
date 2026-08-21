//! The daemon: single job queue, single worker, JSON-lines Unix socket server.
//!
//! Lifecycle follows the gpg-agent model: clients spawn it on demand, it exits
//! on its own after a stretch with no connections and no live jobs.

use std::collections::HashMap;
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
use crate::ratelimit::{ChokePoint, Paid};
use crate::{auth, mockggg};

const IDLE_SHUTDOWN: Duration = Duration::from_secs(60);
const IDLE_POLL: Duration = Duration::from_secs(5);
const ERROR_HISTORY: usize = 50;

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

struct Entry {
    info: JobInfo,
    params: Value,
    outcome: Option<Outcome>,
    cancel_requested: bool,
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
            match daemon.endpoint_url(&info.kind) {
                Some(url) => {
                    // Only same-endpoint jobs ahead of us compete for the
                    // same policy; counting them is what the estimate needs.
                    let ahead = queue
                        .iter()
                        .take_while(|&&q| q != id)
                        .filter(|q| {
                            self.jobs.get(q).and_then(|e| daemon.endpoint_url(&e.info.kind))
                                == Some(url.clone())
                        })
                        .count();
                    Some(daemon.choke.eta_for(&url, ahead as u32).as_secs())
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

    /// The URL a job kind sends to, if it sends at all. `fetch` is a fake
    /// data endpoint that exists only on the mock.
    fn endpoint_url(&self, kind: &str) -> Option<String> {
        match kind {
            "characters" => Some(format!("{}/character", self.provider.api_base)),
            "fetch" if !self.provider.is_real() => Some(format!("{}/fetch", self.provider.api_base)),
            _ => None,
        }
    }

    fn submit(&self, kind: String, params: Value, priority: Priority, submitted_by: String) -> JobId {
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
            };
            s.jobs.insert(
                id,
                Entry {
                    info: info.clone(),
                    params,
                    outcome: None,
                    cancel_requested: false,
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
        let emit = {
            let mut s = self.shared.lock().unwrap();
            let entry = s.jobs.get_mut(&id).ok_or_else(|| format!("no job {id}"))?;
            match entry.info.state {
                JobState::Waiting => {
                    entry.info.state = JobState::Cancelled;
                    entry.outcome = Some(Outcome::Cancelled);
                    Some(entry.info.clone())
                }
                JobState::Running => {
                    // The worker notices between execution slices and keeps
                    // any partial results (there are none in the playground).
                    entry.cancel_requested = true;
                    None
                }
                state => return Err(format!("job {id} already {state}")),
            }
        };
        if let Some(info) = emit {
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

    // ---- worker ---------------------------------------------------------

    async fn worker(self: Arc<Self>) {
        loop {
            let next = {
                let s = self.shared.lock().unwrap();
                s.queue_order().first().copied()
            };
            match next {
                Some(id) => self.process(id).await,
                None => self.work.notified().await,
            }
        }
    }

    async fn process(&self, id: JobId) {
        let url = {
            let s = self.shared.lock().unwrap();
            match s.jobs.get(&id) {
                Some(e) => self.endpoint_url(&e.info.kind),
                None => return,
            }
        };

        // Rate-limit wait happens while the job is still `waiting`, in short
        // slices so cancellation and reprioritization stay responsive. The
        // receipt is handed to `execute`, which spends it on the actual send.
        let mut paid: Option<Paid> = None;
        if let Some(url) = &url {
            loop {
                let step = {
                    let s = self.shared.lock().unwrap();
                    // A reprioritization may have put another job at the head
                    // of the queue; bail so the worker re-picks.
                    if s.queue_order().first() != Some(&id) {
                        return;
                    }
                    match s.jobs.get(&id) {
                        Some(e) if e.info.state == JobState::Waiting => {
                            self.choke.try_take(url)
                        }
                        _ => return, // cancelled out from under us
                    }
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

        let outcome = self.execute(id, &kind, params, paid).await;
        if let Outcome::Failure { error } = &outcome {
            self.note_error(&format!("job {id} ({kind}): {error}"));
        }
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
        self.emit(info);
    }

    async fn execute(&self, id: JobId, kind: &str, params: Value, paid: Option<Paid>) -> Outcome {
        match kind {
            // The one real API call: GET {api_base}/character. Same code in
            // both modes; only the provider's base URL differs. No retries on
            // any failure — a 429 or a Cloudflare-shaped block is reported,
            // never fought through (invariants 2 and 3 are read-only so far:
            // headers are logged and returned, not yet fed to the limiter).
            "characters" => {
                let paid = paid.expect("characters is a network kind");
                let (token, username) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Outcome::Failure { error },
                };
                let url = format!("{}/character", self.provider.api_base);
                match self.api_get(paid, &url, Some(&token)).await {
                    Ok((v, rate)) => Outcome::Success {
                        payload: json!({
                            "provider": self.provider.name,
                            "username": username,
                            "characters": v.get("characters").cloned().unwrap_or(v),
                            "rate_limit": rate,
                        }),
                    },
                    Err(error) => Outcome::Failure { error },
                }
            }
            "sleep" => {
                let seconds = params.get("seconds").and_then(Value::as_f64).unwrap_or(3.0);
                let deadline = Instant::now() + Duration::from_secs_f64(seconds);
                while Instant::now() < deadline {
                    if self.cancelled(id) {
                        return Outcome::Cancelled;
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
                    return Outcome::Failure {
                        error: "fetch is a mock-only kind; real mode has no fake data endpoint".into(),
                    };
                };
                let url = format!("{}/fetch", self.provider.api_base);
                match self.api_get(paid, &url, None).await {
                    Ok((v, rate)) => Outcome::Success {
                        payload: json!({
                            "note": "fake data from the in-process mock",
                            "params": params,
                            "items": v.get("items").cloned().unwrap_or(v),
                            "rate_limit": rate,
                        }),
                    },
                    Err(error) => Outcome::Failure { error },
                }
            }
            "profile" => {
                // The auth-required kind: exercises access-token expiry and
                // silent refresh through the daemon-owned session.
                let (token, username) = match self.valid_access_token(false).await {
                    Ok(pair) => pair,
                    Err(error) => return Outcome::Failure { error },
                };
                tokio::time::sleep(Duration::from_millis(300)).await;
                if self.cancelled(id) {
                    return Outcome::Cancelled;
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
            other => Outcome::Failure {
                error: format!("unknown job kind '{other}' (kinds: sleep, fetch, profile, characters)"),
            },
        }
    }

    /// One rate-limited GET: sends with the receipt, logs the rate headers,
    /// and turns non-2xx into a failure with the evidence. No retries on
    /// anything — a 429 or a Cloudflare-shaped block is reported, never
    /// fought through (invariant 3; 429 recovery is a later step).
    async fn api_get(
        &self,
        paid: Paid,
        url: &str,
        bearer: Option<&str>,
    ) -> Result<(Value, Value), String> {
        let response = match bearer {
            Some(token) => self.choke.get_bearer(paid, url, token).await,
            None => self.choke.get(paid, url).await,
        }
        .map_err(|e| format!("GET {url} failed: {e}"))?;
        let status = response.status();
        let rate = crate::ratelimit::rate_limit_snapshot(response.headers());
        let path = crate::ratelimit::endpoint_key(url);
        self.log(&format!("GET {path} -> {status} | rate headers: {rate}"));
        let body = response.text().await.unwrap_or_default();
        if !status.is_success() {
            let hint = match status.as_u16() {
                429 => " — rate limited; NOT retrying (the limiter now holds this policy until Retry-After + bucket)",
                403 | 503 => " — possibly a Cloudflare block; do NOT retry (invariant 3)",
                _ => "",
            };
            let body: String = body.chars().take(300).collect();
            return Err(format!("GET {path} returned {status}{hint}; rate headers {rate}; body: {body}"));
        }
        serde_json::from_str::<Value>(&body)
            .map(|v| (v, rate))
            .map_err(|e| format!("bad JSON from {path}: {e}"))
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
                    policies: self.choke.policy_statuses(),
                    policyless_endpoints: self.choke.policyless_endpoints(),
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

    tokio::spawn(daemon.clone().worker());
    tokio::spawn(daemon.clone().idle_watchdog());

    loop {
        let (stream, _addr) = listener.accept().await?;
        tokio::spawn(daemon.clone().handle_conn(stream));
    }
}
