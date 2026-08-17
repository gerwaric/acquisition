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

use crate::VERSION;
use crate::job::{JobId, JobInfo, JobState, Outcome, Priority};
use crate::protocol::{Request, Response};
use crate::ratelimit::{ChokePoint, Endpoint};
use crate::{auth, mockggg};

const IDLE_SHUTDOWN: Duration = Duration::from_secs(60);
const IDLE_POLL: Duration = Duration::from_secs(5);

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
    fn snapshot(&self, choke: &ChokePoint, id: JobId) -> Option<JobInfo> {
        let queue = self.queue_order();
        let entry = self.jobs.get(&id)?;
        let mut info = entry.info.clone();
        info.eta_seconds = if info.state == JobState::Waiting {
            let ahead = queue.iter().position(|&q| q == id).unwrap_or(0);
            // Only token-consuming jobs ahead of us actually delay dispatch,
            // but counting them all keeps this honest enough for a playground.
            Some(choke.eta_for(Endpoint::Api, ahead as u32).as_secs())
        } else {
            None
        };
        Some(info)
    }

    fn list(&self, choke: &ChokePoint) -> Vec<JobInfo> {
        let mut ids: Vec<JobId> = self.jobs.keys().copied().collect();
        ids.sort();
        ids.into_iter().filter_map(|id| self.snapshot(choke, id)).collect()
    }
}

pub struct Daemon {
    shared: Mutex<Shared>,
    events: broadcast::Sender<JobInfo>,
    work: Notify,
    log: Mutex<std::fs::File>,
    /// The single rate-limit choke point (CONTEXT invariant 1). It owns the
    /// HTTP client, so all outbound requests — OAuth included — pay a token.
    choke: ChokePoint,
    /// Base URL of the in-process mock OAuth provider (never GGG).
    provider_url: String,
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
        let needs_token = {
            let s = self.shared.lock().unwrap();
            match s.jobs.get(&id) {
                // Everything that would be real API traffic consumes a token.
                Some(e) => matches!(e.info.kind.as_str(), "fetch" | "profile"),
                None => return,
            }
        };

        // Rate-limit wait happens while the job is still `waiting`, in short
        // slices so cancellation and reprioritization stay responsive.
        if needs_token {
            loop {
                let wait = {
                    let s = self.shared.lock().unwrap();
                    // A reprioritization may have put another job at the head
                    // of the queue; bail so the worker re-picks.
                    if s.queue_order().first() != Some(&id) {
                        return;
                    }
                    match s.jobs.get(&id) {
                        Some(e) if e.info.state == JobState::Waiting => {
                            self.choke.try_take(Endpoint::Api).err()
                        }
                        _ => return, // cancelled out from under us
                    }
                };
                match wait {
                    None => break,
                    Some(d) => tokio::time::sleep(d.min(Duration::from_secs(1))).await,
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

        let outcome = self.execute(id, &kind, params).await;
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

    async fn execute(&self, id: JobId, kind: &str, params: Value) -> Outcome {
        match kind {
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
                // Simulated network time. Never a real request: the playground
                // has no GGG code paths at all, by design.
                tokio::time::sleep(Duration::from_millis(400)).await;
                if self.cancelled(id) {
                    return Outcome::Cancelled;
                }
                Outcome::Success {
                    payload: json!({
                        "note": "fake data — playground never talks to GGG",
                        "params": params,
                        "items": [
                            { "name": "Headhunter", "type": "Leather Belt" },
                            { "name": "Tabula Rasa", "type": "Simple Robe" },
                        ],
                    }),
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
                error: format!("unknown job kind '{other}' (playground kinds: sleep, fetch, profile)"),
            },
        }
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
        let redirect_uri = format!("http://127.0.0.1:{port}/callback");
        let (verifier, challenge) = auth::pkce_pair();
        let state = auth::random_token("st");
        let authorize_url = format!(
            "{}/authorize?response_type=code&client_id={}&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope=account:profile",
            self.provider_url,
            auth::CLIENT_ID,
            mockggg::urlencode(&redirect_uri),
            state,
            challenge,
        );
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
                        &daemon.provider_url,
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
                        Err(e) => daemon.log(&format!("token exchange failed: {e}")),
                    }
                }
                Ok(Err(e)) => daemon.log(&format!("auth callback failed: {e}")),
                Err(_) => daemon.log("auth flow timed out after 300s"),
            }
            daemon.shared.lock().unwrap().auth.pending = false;
        });
        Ok(authorize_url)
    }

    /// Store fresh tokens in memory and mirror the refresh token (which the
    /// provider rotates on every grant) into the keyring.
    fn install_tokens(&self, tokens: auth::TokenResponse) {
        let keyring = match auth::keyring_save(&tokens.refresh_token, &tokens.username) {
            Ok(()) => "ok".to_string(),
            Err(e) => {
                self.log(&format!("keyring save failed: {e} (session is in-memory only)"));
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
        let tokens = auth::refresh(&self.choke, &self.provider_url, &refresh_token)
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
                }
            }
            Request::Submit { kind, params, priority, submitted_by } => Response::Submitted {
                id: self.submit(kind, params, priority, submitted_by),
            },
            Request::Status { id } => match self.shared.lock().unwrap().snapshot(&self.choke, id) {
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
                jobs: self.shared.lock().unwrap().list(&self.choke),
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
                    self.log(&format!("auth check failed: {message}"));
                    Response::Error { message }
                }
            },
            Request::AuthLogout => {
                {
                    let mut s = self.shared.lock().unwrap();
                    let keyring = std::mem::take(&mut s.auth.keyring);
                    s.auth = AuthSession { keyring, ..AuthSession::default() };
                }
                if let Err(e) = auth::keyring_clear() {
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
                    uptime_seconds: s.started.elapsed().as_secs(),
                    connections: s.connections,
                    jobs_waiting: waiting,
                    jobs_running: running,
                    tokens_available: self.choke.available(Endpoint::Api),
                    oauth_tokens_available: self.choke.available(Endpoint::OauthToken),
                }
            }
            Request::DaemonStop => Response::Stopping,
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
        if req.method != "GET" || req.path != "/callback" {
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

    let provider_url = mockggg::start().await?;

    // A session in the keyring survives daemon restarts; the first
    // auth-required job will refresh its way to a live access token.
    let mut session = AuthSession::default();
    match auth::keyring_load() {
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
        }),
        events: broadcast::channel(256).0,
        work: Notify::new(),
        log: Mutex::new(log),
        choke: ChokePoint::playground_default(),
        provider_url,
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
    daemon.log(&format!(
        "mock oauth provider at {} | keyring: {} | session: {}",
        daemon.provider_url,
        keyring,
        username.as_deref().unwrap_or("none"),
    ));

    tokio::spawn(daemon.clone().worker());
    tokio::spawn(daemon.clone().idle_watchdog());

    loop {
        let (stream, _addr) = listener.accept().await?;
        tokio::spawn(daemon.clone().handle_conn(stream));
    }
}
