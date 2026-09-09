//! Client side of the daemon protocol: connect, lazy-spawn, version handshake.
//!
//! Every frontend (CLI, MCP, GUI) reaches the daemon through this module,
//! by one of three doors — the policy tiers of C10:
//!
//! - **Use** — [`Client::connect`] with a [`ConnectOptions`]: the verb is
//!   about to submit work. The interactive CLI spawns as asked and replaces
//!   a mismatched daemon, because its caller is the human expressing
//!   intent (`ConnectOptions::interactive`); an autonomous client (the MCP
//!   server) spawns only into an empty socket in mock mode and never
//!   replaces — the mismatch it sees may be a human's live GGG run
//!   (`ConnectOptions::autonomous`).
//! - **Observe** — [`Client::observe`]: the verb reads the daemon (`jobs`,
//!   `status`, `result`, `auth status`, `daemon status`, a quote) or acts
//!   on the one that is there (`cancel`, `set-priority`, `reset-tripwire`).
//!   It takes no options because there is nothing to allow: it never
//!   spawns or replaces, and it answers [`Observed::Absent`], a
//!   [`Observed::Compatible`] client, or the [`DaemonId`] of an
//!   [`Observed::Incompatible`] daemon it identified and did not use.
//! - **Stop** — [`Client::stop_any`]: `daemon stop` stops whatever is
//!   listening, this build's or not. Stopping is how a human resolves a
//!   mismatch, so it is the one verb that acts on a daemon it would not use.
//!
//! `ACQ_NO_SPAWN=1` turns every use door into an observation.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/daemon.md` for this
//! area, `CONTEXT.md` for the cross-cutting ones (`C<n>`); what follows is
//! the entry's full text as recorded there, kept beside the code that
//! implements it. The registry is current; this is the mechanism as
//! decided and as built.
//!
//! ## C10 — Version handshake in the protocol; the protocol is single-version on purpose.
//!
//! **Version handshake in the protocol; the protocol is single-version on
//! purpose.** Kill-and-respawn is the entire migration mechanism. A client
//! uses a daemon only when its provider and runtime identity match the
//! runtime it would itself spawn. That identity changes automatically with
//! the daemon and protocol implementation it governs; it never derives from
//! Git state or a hand-maintained compatibility number. A use verb may
//! replace a mismatch; observation never spawns or replaces and reports
//! absence, identity mismatch, and provider mismatch distinctly; an
//! autonomous client (MCP) never replaces. *Why:* a compat matrix is the
//! reconciliation swamp; respawn is a one-line diff; an observer that
//! replaced cost a live run (2026-09-08). Amended 2026-09-09.
//!
//! ## C10 — as built
//!
//! The identity compared is [`VERSION_WITH_RUNTIME`]: the package version
//! plus the runtime revision, a digest over the core and store sources,
//! their manifests, the root manifest and the lock (`build.rs`). It
//! changes when the daemon's code changes and only then: an uncommitted
//! edit to `daemon.rs` makes a running daemon stale, an edit to the
//! planner or a frontend does not, and no git state is consulted. The
//! package version alone is fixed at `0.0.1` across the playground, and
//! comparing it let a pre-realm daemon accept a console job and render
//! the pc URL (review finding 2026-09-02). The provider is the
//! handshake's `provider` against what this process wants (`ACQ_GGG`).
//! The two dimensions are reported together ([`DaemonId::report`])
//! because both can differ at once. Every frontend built from one tree
//! carries the same revision, which is what lets `acq-mcp` accept a
//! daemon `acq` spawned (C6, C31); two frontends built from different
//! trees would thrash by respawning each other's daemons — theoretical in
//! a one-workspace playground, recorded so it isn't relearned live. After
//! the `acqd` split the identity becomes the daemon artifact plus the
//! protocol crate's revision.
//!
//! The trap the observe tier closes (ledger row 2026-09-08): `acq daemon
//! status` typed in a second terminal without `ACQ_GGG` connected under the
//! interactive policy, replaced the live daemon with a mock one on the
//! default socket, and the driver's next daemon refused to start over it.
//! Every observational verb of both frontends now goes through
//! [`Client::observe`]; the interactive `ConnectOptions` reach only the
//! verbs that submit work.
//!
//! Accepted residual: the identity dimension has no process-level test (one
//! binary per test run); the provider dimension takes the same path and is
//! pinned through the binaries in `acquisition-cli/tests/daemon_observe.rs`.

use std::fmt;
use std::time::Duration;

use crate::VERSION_WITH_RUNTIME;
use crate::daemon::{log_path, socket_path};
use crate::job::JobInfo;
use crate::protocol::{Request, Response};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

/// What a *use* verb may do to a daemon that isn't the one it wants.
/// `ACQ_NO_SPAWN=1` overrides both flags to false. Observation takes no
/// options: [`Client::observe`].
#[derive(Clone, Copy, Debug)]
pub struct ConnectOptions {
    /// Start a daemon if none is listening (lazy spawn, via the calling
    /// binary's own `daemon run`).
    pub spawn: bool,
    /// Kill and respawn a daemon whose identity or provider doesn't match.
    pub replace: bool,
}

impl ConnectOptions {
    /// The interactive CLI's policy for a use verb: the caller is the
    /// human, so replacing a wrong-build or wrong-mode daemon is them
    /// expressing intent.
    pub fn interactive(spawn: bool) -> Self {
        Self {
            spawn,
            replace: true,
        }
    }

    /// An autonomous client's policy (MCP): never kill or replace a daemon —
    /// the mismatched daemon may be a human's live GGG run. Spawning into an
    /// empty socket may still be allowed.
    pub fn autonomous(spawn: bool) -> Self {
        Self {
            spawn,
            replace: false,
        }
    }
}

/// "ggg" or "mock": the provider this process wants a daemon to serve.
fn want_provider() -> &'static str {
    if crate::provider::ggg_mode() {
        "ggg"
    } else {
        "mock"
    }
}

/// A daemon as its handshake identifies it. Whether it is this client's
/// is two dimensions, reported together (C10): the runtime identity and
/// the provider.
#[derive(Clone, Debug, Serialize)]
pub struct DaemonId {
    pub pid: u32,
    /// The daemon's `VERSION_WITH_RUNTIME`.
    pub version: String,
    /// "mock" or "ggg".
    pub provider: String,
}

impl DaemonId {
    /// The daemon runs the same runtime this process would spawn.
    pub fn identity_matches(&self) -> bool {
        self.version == VERSION_WITH_RUNTIME
    }

    /// The daemon serves the provider this process wants.
    pub fn provider_matches(&self) -> bool {
        self.provider == want_provider()
    }

    /// Both dimensions match: this client may use the daemon.
    pub fn is_ours(&self) -> bool {
        self.identity_matches() && self.provider_matches()
    }

    /// The observer's report, one shape for every frontend: the daemon
    /// found, what this process wanted, and which dimensions differ.
    pub fn report(&self) -> serde_json::Value {
        json!({
            "pid": self.pid,
            "version": self.version,
            "provider": self.provider,
            "identity_matches": self.identity_matches(),
            "provider_matches": self.provider_matches(),
            "wanted": { "version": VERSION_WITH_RUNTIME, "provider": want_provider() },
        })
    }
}

impl fmt::Display for DaemonId {
    /// The mismatch sentence an observer prints: which dimensions differ,
    /// with both sides of each.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "daemon (pid {})", self.pid)?;
        match (self.identity_matches(), self.provider_matches()) {
            (true, true) => write!(f, " is this client's ({}, {})", self.version, self.provider),
            (false, true) => write!(
                f,
                " is another runtime ({}; this is {VERSION_WITH_RUNTIME})",
                self.version
            ),
            (true, false) => write!(
                f,
                " is on another provider ({}; this process wants {})",
                self.provider,
                want_provider()
            ),
            (false, false) => write!(
                f,
                " is another runtime ({}; this is {VERSION_WITH_RUNTIME}) on another provider ({}; this process wants {})",
                self.version,
                self.provider,
                want_provider()
            ),
        }
    }
}

/// What [`Client::observe`] found on the socket.
pub enum Observed {
    /// Nothing is listening.
    Absent,
    /// The daemon is this client's, and this is a connection to it.
    Compatible(Client),
    /// A daemon that is not this client's: identified, reported, not used.
    Incompatible(DaemonId),
}

pub struct Client {
    lines: Lines<BufReader<OwnedReadHalf>>,
    write: OwnedWriteHalf,
    /// The daemon at the other end, as its handshake identified it.
    daemon: DaemonId,
}

fn no_spawn() -> bool {
    std::env::var_os("ACQ_NO_SPAWN").is_some_and(|v| v == "1")
}

/// A connect failure that means nothing is listening — the socket absent
/// or refusing — as opposed to a transport problem worth reporting.
fn is_absent(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
    )
}

impl Client {
    /// Connect to the daemon for a *use* verb, spawning or replacing one as
    /// `opts` allows. A mismatch this client may not resolve (a mock-mode
    /// daemon can't serve an `ACQ_GGG=1` client, or vice versa) is an error
    /// naming the daemon it found.
    pub async fn connect(opts: ConnectOptions) -> Result<Client> {
        // `ACQ_NO_SPAWN=1`: never start or replace a daemon from this
        // process. A daemon spawned from a non-interactive parent (cron,
        // launchd) has no keychain access on macOS — it comes up with no
        // session and every job fails "not logged in" (re-soak, 2026-08-25,
        // caught by rail 7). The live drivers set this so their scripts can
        // only talk to a daemon they started themselves.
        let spawn = opts.spawn && !no_spawn();
        let replace = opts.replace && !no_spawn();
        let mut respawned = false;
        // The spawned daemon, with where its log ended at spawn time: if it
        // exits instead of binding the socket, the lines after that offset
        // are its refusal (its stderr goes to null — the log is all there is).
        let mut child: Option<(std::process::Child, u64)> = None;
        for _attempt in 0..100 {
            match UnixStream::connect(socket_path()).await {
                Ok(stream) => {
                    let mut client = Client::handshake(stream).await?;
                    if client.daemon.is_ours() {
                        return Ok(client);
                    }
                    let found = &client.daemon;
                    if respawned {
                        bail!("{found}, still, after a respawn");
                    }
                    if !replace {
                        let why = if no_spawn() {
                            "ACQ_NO_SPAWN forbids replacing it"
                        } else {
                            "this client never replaces a daemon — resolve it with the CLI (`acq daemon stop`)"
                        };
                        bail!("{found}, and {why}");
                    }
                    // Stale daemon (older build, or wrong mode): kill and respawn.
                    let _ = client.request(&Request::DaemonStop).await;
                    respawned = true;
                    child = None;
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                // Once we've killed a mismatched daemon we must respawn it
                // even for a `spawn: false` caller — leaving nothing running
                // would turn a read into a stop.
                Err(_) if spawn || respawned => {
                    match child.as_mut() {
                        None => child = Some(spawn_daemon()?),
                        Some((c, log_from)) => {
                            // An exited daemon will never bind the socket:
                            // report its refusal now, not after the timeout.
                            if let Some(status) = c.try_wait().ok().flatten() {
                                bail!(
                                    "daemon exited during startup ({status}){}",
                                    startup_log_excerpt(*log_from)
                                );
                            }
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                Err(e) if no_spawn() => {
                    return Err(e).context(
                        "daemon is not running and ACQ_NO_SPAWN forbids starting one from here",
                    );
                }
                Err(e) => {
                    return Err(e)
                        .context("daemon is not running (it spawns on demand for job commands)");
                }
            }
        }
        bail!(
            "could not reach daemon at {} after 5s{}",
            socket_path().display(),
            child.map_or_else(String::new, |(_, log_from)| startup_log_excerpt(log_from))
        )
    }

    /// Observe the socket (C10): never spawns or replaces. Nothing
    /// listening is [`Observed::Absent`]; this client's daemon comes back
    /// connected; any other daemon is identified and reported, and the
    /// connection to it is dropped unused.
    pub async fn observe() -> Result<Observed> {
        match UnixStream::connect(socket_path()).await {
            Ok(stream) => {
                let client = Client::handshake(stream).await?;
                Ok(if client.daemon.is_ours() {
                    Observed::Compatible(client)
                } else {
                    Observed::Incompatible(client.daemon)
                })
            }
            Err(e) if is_absent(&e) => Ok(Observed::Absent),
            Err(e) => Err(e).with_context(|| format!("connecting to {}", socket_path().display())),
        }
    }

    /// `daemon stop`: ask whatever daemon is listening to stop, this
    /// client's or not — stopping is how a human resolves a mismatch. Says
    /// which daemon acknowledged (the daemon writes `Stopping` before it
    /// exits; anything else is an error, not a stop); `None` when nothing
    /// was listening.
    pub async fn stop_any() -> Result<Option<DaemonId>> {
        match UnixStream::connect(socket_path()).await {
            Ok(stream) => Client::stop_over(stream).await.map(Some),
            Err(e) if is_absent(&e) => Ok(None),
            Err(e) => Err(e).with_context(|| format!("connecting to {}", socket_path().display())),
        }
    }

    /// The stop conversation over an open connection: identify the peer,
    /// ask it to stop, and count only its `Stopping` as a stop.
    async fn stop_over(stream: UnixStream) -> Result<DaemonId> {
        let mut client = Client::handshake(stream).await?;
        match client.request(&Request::DaemonStop).await? {
            Response::Stopping => Ok(client.daemon),
            Response::Error { message } => {
                bail!("{} refused to stop: {message}", client.daemon)
            }
            other => bail!("unexpected response to stop: {other:?}"),
        }
    }

    /// The handshake over a fresh connection: who is at the other end.
    /// Decides nothing — the caller reads `daemon` and applies its policy.
    async fn handshake(stream: UnixStream) -> Result<Client> {
        let (read, write) = stream.into_split();
        let mut client = Client {
            lines: BufReader::new(read).lines(),
            write,
            daemon: DaemonId {
                pid: 0,
                version: String::new(),
                provider: String::new(),
            },
        };
        let hello = client
            .request(&Request::Hello {
                client_version: VERSION_WITH_RUNTIME.to_string(),
            })
            .await?;
        let Response::Hello {
            daemon_version,
            pid,
            provider,
        } = hello
        else {
            bail!("unexpected handshake response: {hello:?}");
        };
        client.daemon = DaemonId {
            pid,
            version: daemon_version,
            provider,
        };
        Ok(client)
    }

    /// The daemon this client reached, as its handshake identified it.
    pub fn daemon(&self) -> &DaemonId {
        &self.daemon
    }

    /// "mock" or "ggg", from the handshake of the daemon this client reached.
    pub fn provider(&self) -> &str {
        &self.daemon.provider
    }

    /// Send a request and return the next non-event response. Events arriving
    /// in between (on subscribed connections) are dropped here; use `recv` in
    /// event-driven flows instead.
    pub async fn request(&mut self, req: &Request) -> Result<Response> {
        let mut line = serde_json::to_string(req)?;
        line.push('\n');
        self.write.write_all(line.as_bytes()).await?;
        loop {
            match self.recv().await? {
                Response::Event { .. } => continue,
                other => return Ok(other),
            }
        }
    }

    pub async fn recv(&mut self) -> Result<Response> {
        let line = self
            .lines
            .next_line()
            .await?
            .context("daemon closed the connection")?;
        Ok(serde_json::from_str(&line)?)
    }

    /// `request` variants that unwrap the expected response shape.
    pub async fn expect_ack(&mut self, req: &Request) -> Result<()> {
        match self.request(req).await? {
            Response::Ack => Ok(()),
            Response::Error { message } => bail!("{message}"),
            other => bail!("unexpected response: {other:?}"),
        }
    }

    pub async fn status(&mut self, id: u64) -> Result<JobInfo> {
        match self.request(&Request::Status { id }).await? {
            Response::Status { job } => Ok(job),
            Response::Error { message } => bail!("{message}"),
            other => bail!("unexpected response: {other:?}"),
        }
    }

    /// The daemon's read-only, non-reserving projection over `jobs`
    /// (see [`crate::protocol::Quote`]). Sends nothing; shared here so
    /// every frontend's quote surface speaks the same request.
    pub async fn quote(
        &mut self,
        jobs: Vec<crate::protocol::QuoteJob>,
        account: Option<String>,
    ) -> Result<crate::protocol::Quote> {
        match self.request(&Request::Quote { jobs, account }).await? {
            Response::Quote { quote } => Ok(quote),
            Response::Error { message } => bail!("{message}"),
            other => bail!("unexpected response: {other:?}"),
        }
    }
}

fn spawn_daemon() -> Result<(std::process::Child, u64)> {
    // Where the log ends now; lines past this offset are the new daemon's.
    let log_from = std::fs::metadata(log_path()).map_or(0, |m| m.len());
    let exe = std::env::current_exe()?;
    let child = std::process::Command::new(exe)
        .args(["daemon", "run"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("failed to spawn daemon")?;
    Ok((child, log_from))
}

/// What the daemon wrote to its log after we spawned it — the only place a
/// lazy-spawned daemon's startup refusal lands.
fn startup_log_excerpt(log_from: u64) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let path = log_path();
    let tail = std::fs::File::open(&path).ok().and_then(|mut f| {
        f.seek(SeekFrom::Start(log_from)).ok()?;
        let mut s = String::new();
        f.read_to_string(&mut s).ok()?;
        let lines: Vec<&str> = s.trim().lines().collect();
        // A refusal is a few lines; cap so a crash after a busy start
        // doesn't flood the terminal.
        let last = &lines[lines.len().saturating_sub(20)..];
        (!last.is_empty()).then(|| last.join("\n  "))
    });
    match tail {
        Some(t) => format!("; the daemon log says:\n  {t}"),
        None => format!(
            "; its log ({}) has nothing new — it may have failed before opening it",
            path.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(version: &str, provider: &str) -> DaemonId {
        DaemonId {
            pid: 42,
            version: version.into(),
            provider: provider.into(),
        }
    }

    /// C10: the handshake compares the runtime revision, not the package
    /// version — a daemon reporting the bare `0.0.1` (every build before
    /// this check, and any other revision's build) is not this client's —
    /// and the provider is a second dimension, reported with the first.
    /// The tests run without `ACQ_GGG`, so "mock" is the wanted provider.
    #[test]
    fn a_daemon_from_another_build_or_provider_is_not_this_clients_daemon() {
        assert!(id(VERSION_WITH_RUNTIME, "mock").is_ours());
        assert!(!id(VERSION_WITH_RUNTIME, "ggg").is_ours());
        assert!(!id(crate::VERSION, "mock").is_ours());
        assert!(!id("0.0.1 (deadbeef)", "mock").is_ours());
        assert!(VERSION_WITH_RUNTIME.contains(crate::RUNTIME_REVISION));
        assert_eq!(crate::RUNTIME_REVISION.len(), 12);
        assert!(
            crate::RUNTIME_REVISION
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        );

        let both = id("0.0.1 (deadbeef)", "ggg");
        assert!(!both.identity_matches() && !both.provider_matches());
        let report = both.report();
        assert_eq!(report["identity_matches"], false);
        assert_eq!(report["provider_matches"], false);
        assert_eq!(report["wanted"]["provider"], "mock");
        let text = both.to_string();
        assert!(
            text.contains("another runtime") && text.contains("another provider"),
            "{text}"
        );
        let text = id(VERSION_WITH_RUNTIME, "ggg").to_string();
        assert!(
            text.contains("another provider") && !text.contains("another runtime"),
            "{text}"
        );
    }

    /// A scripted peer on a scratch socket: answers the handshake as a
    /// daemon would, then answers the stop request with `reply` — or
    /// hangs up when `reply` is `None`.
    async fn peer_that_answers_stop_with(reply: Option<Response>) -> UnixStream {
        let dir = std::env::temp_dir().join(format!("acq-stop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = dir.join(format!("{n}.sock"));
        let _ = std::fs::remove_file(&path);
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let peer_path = path.clone();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read, mut write) = stream.into_split();
            let mut lines = BufReader::new(read).lines();
            let _hello = lines.next_line().await.unwrap().unwrap();
            let hello = Response::Hello {
                daemon_version: VERSION_WITH_RUNTIME.to_string(),
                pid: 7,
                provider: "mock".into(),
            };
            let mut line = serde_json::to_string(&hello).unwrap();
            line.push('\n');
            write.write_all(line.as_bytes()).await.unwrap();
            let _stop = lines.next_line().await.unwrap().unwrap();
            if let Some(reply) = reply {
                let mut line = serde_json::to_string(&reply).unwrap();
                line.push('\n');
                write.write_all(line.as_bytes()).await.unwrap();
            }
            // Dropping `write` hangs up either way.
            let _ = std::fs::remove_file(&peer_path);
        });
        UnixStream::connect(&path).await.unwrap()
    }

    /// The defect of review round 1: a stop that the peer refuses, answers
    /// strangely, or drops must never come back as a stop. Only
    /// `Stopping` counts.
    #[tokio::test]
    async fn a_stop_the_daemon_did_not_acknowledge_is_not_a_stop() {
        let stream = peer_that_answers_stop_with(Some(Response::Error {
            message: "busy".into(),
        }))
        .await;
        let err = Client::stop_over(stream).await.unwrap_err().to_string();
        assert!(
            err.contains("refused to stop") && err.contains("busy"),
            "{err}"
        );

        let stream = peer_that_answers_stop_with(Some(Response::Ack)).await;
        let err = Client::stop_over(stream).await.unwrap_err().to_string();
        assert!(err.contains("unexpected response to stop"), "{err}");

        let stream = peer_that_answers_stop_with(None).await;
        let err = Client::stop_over(stream).await.unwrap_err().to_string();
        assert!(err.contains("closed the connection"), "{err}");

        let stream = peer_that_answers_stop_with(Some(Response::Stopping)).await;
        let stopped = Client::stop_over(stream).await.unwrap();
        assert_eq!(stopped.pid, 7);
    }
}
