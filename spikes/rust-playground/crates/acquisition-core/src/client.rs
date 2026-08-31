//! Client side of the daemon protocol: connect, lazy-spawn, version handshake.
//!
//! Every frontend (CLI, MCP, GUI) reaches the daemon through this module; the
//! difference between them is the `ConnectOptions` policy. The interactive CLI
//! kills and respawns a version- or provider-mismatched daemon because its
//! caller is the human expressing intent. An autonomous client (the MCP
//! server) must never do that — a mismatch could be a live GGG daemon under
//! the rails — so it reports the mismatch and stops.

use std::time::Duration;

use crate::VERSION;
use crate::daemon::{log_path, socket_path};
use crate::job::JobInfo;
use crate::protocol::{Request, Response};
use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

/// What this client is allowed to do to a daemon that isn't the one it wants.
/// `ACQ_NO_SPAWN=1` overrides both flags to false.
#[derive(Clone, Copy, Debug)]
pub struct ConnectOptions {
    /// Start a daemon if none is listening (lazy spawn, via the calling
    /// binary's own `daemon run`).
    pub spawn: bool,
    /// Kill and respawn a daemon whose version or provider doesn't match.
    pub replace: bool,
}

impl ConnectOptions {
    /// The interactive CLI's policy: the caller is the human, so replacing a
    /// wrong-version or wrong-mode daemon is them expressing intent.
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

pub struct Client {
    lines: Lines<BufReader<OwnedReadHalf>>,
    write: OwnedWriteHalf,
    /// "mock" or "ggg", as the daemon reported in its handshake.
    provider: String,
}

fn no_spawn() -> bool {
    std::env::var_os("ACQ_NO_SPAWN").is_some_and(|v| v == "1")
}

impl Client {
    /// Connect to the daemon, spawning or replacing one as `opts` allows.
    /// A version or provider mismatch this client may not resolve (a
    /// mock-mode daemon can't serve an `ACQ_GGG=1` client, or vice versa)
    /// is an error naming the daemon it found.
    pub async fn connect(opts: ConnectOptions) -> Result<Client> {
        let want_provider = if crate::provider::ggg_mode() {
            "ggg"
        } else {
            "mock"
        };
        // `ACQ_NO_SPAWN=1`: never start or replace a daemon from this
        // process. A daemon spawned from a non-interactive parent (cron,
        // launchd) has no keychain access on macOS — it comes up with no
        // session and every job fails "not logged in" (re-soak, 2026-08-25,
        // caught by rail 7). The soak script sets this so cron can only
        // talk to a daemon a person started.
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
                    let mut client = Client::from_stream(stream);
                    let hello = client
                        .request(&Request::Hello {
                            client_version: VERSION.to_string(),
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
                    if daemon_version == VERSION && provider == want_provider {
                        client.provider = provider;
                        return Ok(client);
                    }
                    if respawned {
                        bail!(
                            "daemon (pid {pid}) still reports version {daemon_version} / provider {provider} after respawn; wanted {VERSION} / {want_provider}"
                        );
                    }
                    if !replace {
                        let why = if no_spawn() {
                            "ACQ_NO_SPAWN forbids replacing it"
                        } else {
                            "this client never replaces a daemon — resolve it with the CLI (`acq daemon stop`)"
                        };
                        bail!(
                            "daemon (pid {pid}) reports version {daemon_version} / provider {provider}; wanted {VERSION} / {want_provider}, and {why}"
                        );
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

    fn from_stream(stream: UnixStream) -> Client {
        let (read, write) = stream.into_split();
        Client {
            lines: BufReader::new(read).lines(),
            write,
            provider: String::new(),
        }
    }

    /// "mock" or "ggg", from the handshake of the daemon this client reached.
    pub fn provider(&self) -> &str {
        &self.provider
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
