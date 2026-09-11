//! `acq jobs --watch` under lag, through the binary (C85's subscriber
//! sequence, as the reference consumer). The watcher is stopped with
//! SIGSTOP while the daemon runs more jobs than its event channel holds,
//! so its subscription lags; on SIGCONT it must drop that subscription,
//! subscribe afresh and print the snapshot again — and after that
//! snapshot no event about an older job may appear, because a fresh
//! subscription carries only what happens after it was opened. The
//! original implementation kept the lagged subscription and would print
//! its queued events after the snapshot; this test fails on it.
//!
//! What is not observable from outside: that each printed event line is
//! a re-read of the job rather than the hint. The watch has no other
//! path (`main.rs`, `watch_jobs`), and a read that fails restarts the
//! sequence; that stays a reading of the code, not a pin.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use acquisition_client::client::{Client, Observed};
use acquisition_protocol::protocol::{Request, Response};
use serde_json::{Value, json};

fn command(base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_STORE_DIR", base.join("store"))
        .env("TMPDIR", scratch_tmp(base))
        .env("XDG_RUNTIME_DIR", scratch_tmp(base))
        .env("ACQ_LOG_DIR", base.join("logs"))
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_JOURNAL", "0")
        .env("ACQ_IDLE_SHUTDOWN", "30");
    for var in [
        "ACQ_GGG",
        "ACQ_ACCOUNT",
        "ACQ_TRIPWIRE",
        "ACQ_MAX_SENDS",
        "ACQ_NO_SPAWN",
    ] {
        cmd.env_remove(var);
    }
    cmd
}

/// The test's scratch directory, removed on drop — declared before the
/// daemon so the daemon is gone first.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Proc(Child, PathBuf);

impl Drop for Proc {
    /// A failing test names the executable it ran (the daemon by path:
    /// C82's "named in every failure").
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "process under test: {} (pid {})",
                self.1.display(),
                self.0.id()
            );
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn signal(child: &Child, sig: &str) {
    let status = Command::new("kill")
        .args([sig, &child.id().to_string()])
        .status()
        .expect("kill");
    assert!(status.success(), "kill {sig}");
}

/// The daemon `acq` would itself spawn: the `acqd` beside the binary
/// under test (C82, the locator's one rule). A missing one fails here,
/// before the test runs, naming the build step. This test also drives
/// the client in-process, which judges the daemon's artifact against the
/// `acqd` beside *this* executable (C84) — a test executable in `deps/`
/// has none — so a symlink of that name is placed beside it, pointing
/// one level up at the same file: the test executable names the daemon
/// its build wrote on both paths (C82's clause for test executables).
fn acqd() -> PathBuf {
    let acqd = acquisition_client::locator::beside(Path::new(env!("CARGO_BIN_EXE_acq")))
        .unwrap_or_else(|e| panic!("{e}"));
    let exe = std::env::current_exe().expect("current exe");
    let link = exe.parent().expect("a parent").join("acqd");
    let target = Path::new("..").join("acqd");
    if std::fs::read_link(&link).ok().as_deref() != Some(target.as_path()) {
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link)
            .unwrap_or_else(|e| panic!("placing {} -> {}: {e}", link.display(), target.display()));
    }
    assert_eq!(
        link.canonicalize().expect("the sibling resolves"),
        acqd.canonicalize().expect("the daemon resolves"),
        "the sibling the in-process client sees is not the daemon acq would spawn"
    );
    acqd
}

/// The daemon started directly by the test, which owns its pid, under
/// the same isolation as `command`; its stdio to null, as a lazy spawn's.
fn daemon_command(base: &Path, acqd: &Path) -> Command {
    let mut cmd = Command::new(acqd);
    cmd.env("ACQ_STORE_DIR", base.join("store"))
        .env("TMPDIR", scratch_tmp(base))
        .env("XDG_RUNTIME_DIR", scratch_tmp(base))
        .env("ACQ_LOG_DIR", base.join("logs"))
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_JOURNAL", "0")
        .env("ACQ_IDLE_SHUTDOWN", "30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for var in [
        "ACQ_GGG",
        "ACQ_ACCOUNT",
        "ACQ_TRIPWIRE",
        "ACQ_MAX_SENDS",
        "ACQ_NO_SPAWN",
    ] {
        cmd.env_remove(var);
    }
    cmd
}

fn start_daemon(base: &Path) -> Proc {
    let acqd = acqd();
    let child = daemon_command(base, &acqd)
        .spawn()
        .unwrap_or_else(|e| panic!("spawning {}: {e}", acqd.display()));
    let daemon = Proc(child, acqd.clone());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let out = command(base, &["daemon", "status", "--json"])
            .output()
            .expect("acq");
        let status: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
        if status["pid"].as_u64().is_some() {
            return daemon;
        }
        assert!(
            Instant::now() < deadline,
            "the mock daemon ({}) did not come up",
            acqd.display()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// One thing the watcher printed under `--json`: a snapshot (a pretty
/// array) or an event (one object per line).
#[derive(Debug)]
enum Item {
    Snapshot(Vec<Value>),
    Event(Value),
}

struct Output {
    lines: mpsc::Receiver<String>,
}

impl Output {
    fn line(&self, deadline: Instant) -> String {
        let left = deadline.saturating_duration_since(Instant::now());
        self.lines
            .recv_timeout(left)
            .expect("the watcher printed nothing more before the deadline")
    }

    fn item(&self, deadline: Instant) -> Item {
        let first = self.line(deadline);
        if first == "[]" {
            return Item::Snapshot(Vec::new());
        }
        if first.starts_with('[') {
            let mut text = first;
            loop {
                let line = self.line(deadline);
                text.push('\n');
                text.push_str(&line);
                if line == "]" {
                    break;
                }
            }
            return Item::Snapshot(serde_json::from_str(&text).expect("a snapshot array"));
        }
        Item::Event(serde_json::from_str(&first).unwrap_or_else(|e| panic!("{e}: {first}")))
    }
}

fn watcher(base: &Path) -> (Proc, Output) {
    let mut child = command(base, &["jobs", "--watch", "--json"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawning the watcher");
    let stdout = child.stdout.take().expect("piped");
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    (
        Proc(child, PathBuf::from(env!("CARGO_BIN_EXE_acq"))),
        Output { lines: rx },
    )
}

async fn client() -> Client {
    match Client::observe().await.expect("observe") {
        Observed::Compatible(client) => client,
        other => panic!(
            "{}",
            match other {
                Observed::Absent => "no daemon".to_string(),
                Observed::Incompatible(found) => found.to_string(),
                Observed::Compatible(_) => unreachable!(),
            }
        ),
    }
}

async fn submit_sleep(client: &mut Client) -> u64 {
    match client
        .request(&Request::Submit {
            kind: "sleep".into(),
            params: json!({ "seconds": 0 }),
            priority: 0,
            submitted_by: "watch-test".into(),
            account: None,
        })
        .await
        .expect("submit")
    {
        Response::Submitted { id } => id,
        other => panic!("{other:?}"),
    }
}

async fn wait_all_terminal(client: &mut Client, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let jobs = match client.request(&Request::List).await.expect("list") {
            Response::Jobs { jobs } => jobs,
            other => panic!("{other:?}"),
        };
        if jobs.len() == expected && jobs.iter().all(|j| j.state.is_terminal()) {
            return;
        }
        assert!(Instant::now() < deadline, "jobs did not finish");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[test]
fn c85_the_watch_subscribes_and_reads_again_after_it_lagged_with_no_leftover_events() {
    let base: PathBuf = std::env::temp_dir().join(format!("acq-watch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let _scratch = Scratch(base.clone());
    // The in-process client below reads the socket from the environment;
    // this binary holds one test, so nothing else reads it.
    unsafe {
        std::env::set_var("ACQ_STORE_DIR", base.join("store"));
        std::env::set_var("TMPDIR", scratch_tmp(&base));
        std::env::set_var("XDG_RUNTIME_DIR", scratch_tmp(&base));
        std::env::set_var("ACQ_LOG_DIR", base.join("logs"));
        std::env::set_var("ACQ_NO_KEYRING", "1");
        for var in ["ACQ_GGG", "ACQ_ACCOUNT", "ACQ_NO_SPAWN"] {
            std::env::remove_var(var);
        }
    }
    let _daemon = start_daemon(&base);
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let (watch, out) = watcher(&base);
    let deadline = Instant::now() + Duration::from_secs(30);
    match out.item(deadline) {
        Item::Snapshot(jobs) => assert!(jobs.is_empty(), "{jobs:?}"),
        other => panic!("the watch opens with a snapshot: {other:?}"),
    }

    // Stopped, the watcher reads nothing; the daemon's writes to its
    // subscription block once the socket is full, and the event channel
    // overruns the receiver behind them.
    signal(&watch.0, "-STOP");
    const JOBS: usize = 800;
    runtime.block_on(async {
        let mut client = client().await;
        for _ in 0..JOBS {
            submit_sleep(&mut client).await;
        }
        wait_all_terminal(&mut client, JOBS).await;
    });
    signal(&watch.0, "-CONT");

    // Some events the socket held, then the snapshot after `resync_required`.
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut events_before = 0usize;
    let snapshot = loop {
        match out.item(deadline) {
            Item::Event(_) => events_before += 1,
            Item::Snapshot(jobs) => break jobs,
        }
    };
    assert_eq!(
        snapshot.len(),
        JOBS,
        "{events_before} events, then a partial snapshot"
    );
    assert!(
        snapshot.iter().all(|j| j["state"] == "done"),
        "{snapshot:?}"
    );
    let newest_old = snapshot
        .iter()
        .map(|j| j["id"].as_u64().unwrap())
        .max()
        .unwrap();

    // After that snapshot, the only events are about jobs that came later:
    // the lagged subscription and everything it held are gone.
    let next = runtime.block_on(async {
        let mut client = client().await;
        submit_sleep(&mut client).await
    });
    assert!(next > newest_old);
    let deadline = Instant::now() + Duration::from_secs(30);
    match out.item(deadline) {
        Item::Event(job) => {
            let id = job["id"].as_u64().unwrap();
            assert_eq!(
                id, next,
                "an event about job {id} (before {next}) after the fresh snapshot: a leftover of the lagged subscription: {job}"
            );
            assert!(job["state"].is_string(), "{job}");
        }
        Item::Snapshot(jobs) => panic!("a second resync: {jobs:?}"),
    }
}

/// The scratch temp and runtime directory of one test, under `base`:
/// `TMPDIR` (macOS's runtime fallback and the legacy paths) and
/// `XDG_RUNTIME_DIR` (Linux's runtime directory) both point here, so the
/// sockets the daemons bind — and, for a daemon a test kills rather than
/// stops, the socket files it leaves — never touch the user's own
/// runtime directory (C83). Created here, since the runtime directory
/// is made under an existing parent.
fn scratch_tmp(base: &std::path::Path) -> std::path::PathBuf {
    let tmp = base.join("tmp");
    let _ = std::fs::create_dir_all(&tmp);
    tmp
}
