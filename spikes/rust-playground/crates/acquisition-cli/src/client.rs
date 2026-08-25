//! Client side of the daemon protocol: connect, lazy-spawn, version handshake.

use std::time::Duration;

use acquisition_core::VERSION;
use acquisition_core::daemon::socket_path;
use acquisition_core::job::JobInfo;
use acquisition_core::protocol::{Request, Response};
use anyhow::{Context, Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

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
    /// Connect to the daemon, spawning one if needed (`spawn = true`) and
    /// replacing it if the handshake reports a version or provider mismatch
    /// (a mock-mode daemon can't serve an ACQ_GGG=1 client, or vice versa).
    pub async fn connect(spawn: bool) -> Result<Client> {
        let want_provider = if acquisition_core::provider::ggg_mode() {
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
        let spawn = spawn && !no_spawn();
        let mut respawned = false;
        let mut spawned = false;
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
                    if no_spawn() {
                        bail!(
                            "daemon (pid {pid}) reports version {daemon_version} / provider {provider}; wanted {VERSION} / {want_provider}, and ACQ_NO_SPAWN forbids replacing it"
                        );
                    }
                    // Stale daemon (older build, or wrong mode): kill and respawn.
                    let _ = client.request(&Request::DaemonStop).await;
                    respawned = true;
                    spawned = false;
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                Err(_) if spawn => {
                    if !spawned {
                        spawn_daemon()?;
                        spawned = true;
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
            "could not reach daemon at {} after 5s",
            socket_path().display()
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

    pub fn provider(&self) -> &str {
        &self.provider
    }

    pub async fn status(&mut self, id: u64) -> Result<JobInfo> {
        match self.request(&Request::Status { id }).await? {
            Response::Status { job } => Ok(job),
            Response::Error { message } => bail!("{message}"),
            other => bail!("unexpected response: {other:?}"),
        }
    }
}

fn spawn_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;
    std::process::Command::new(exe)
        .args(["daemon", "run"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("failed to spawn daemon")?;
    Ok(())
}
