//! C10's observation tier through the real binary: an observer never
//! spawns or replaces a daemon, and reports absence, identity mismatch and
//! provider mismatch distinctly. The trap this pins is the ledger row of
//! 2026-09-08: `acq daemon status` typed in a shell without `ACQ_GGG`
//! while a real-mode daemon ran replaced it with a mock one. Provider
//! mismatch is the dimension one binary can stage — a mock daemon observed
//! by a client that wants ggg; identity mismatch takes the same path
//! (`DaemonId::is_ours`).
//!
//! `ACQ_GGG=1` appears here only on observing and stopping commands,
//! which cannot spawn by construction: no daemon in real mode ever exists
//! in this test, and nothing reaches GGG.

use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

fn command(base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_SOCKET", base.join("d.sock"))
        .env("ACQ_STORE_DIR", base.join("store"))
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

fn acq(base: &Path, args: &[&str]) -> Output {
    command(base, args).output().expect("spawning acq")
}

/// The same command from a shell that wants the real provider. Only
/// observing and stopping verbs are run this way (module doc).
fn acq_wanting_ggg(base: &Path, args: &[&str]) -> Output {
    command(base, args)
        .env("ACQ_GGG", "1")
        .output()
        .expect("spawning acq")
}

fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A mock daemon started by the test itself, killed on drop if a failed
/// assertion leaves it behind.
struct Daemon(Child);

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_daemon(base: &Path) -> (Daemon, u64) {
    let child = command(base, &["daemon", "run"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawning the mock daemon");
    let daemon = Daemon(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let out = acq(base, &["daemon", "status", "--json"]);
        let status = sole_json(&out);
        if let Some(pid) = status["pid"].as_u64() {
            assert_eq!(status["running"], true, "{status}");
            assert_eq!(status["compatible"], true, "{status}");
            assert_eq!(status["provider"], "mock", "{status}");
            return (daemon, pid);
        }
        assert!(Instant::now() < deadline, "the mock daemon did not come up");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn c10_observation_never_spawns_or_replaces_and_reports_the_mismatch() {
    let base = std::env::temp_dir().join(format!("acq-obs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let socket = base.join("d.sock");

    // Absent: reported as a state by `daemon status`, as an error by a
    // reading verb — and neither spawns (the socket never appears).
    let out = acq(&base, &["daemon", "status", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out), serde_json::json!({ "running": false }));
    for args in [&["jobs", "--json"][..], &["dash", "--json"][..]] {
        let out = acq(&base, args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
        assert!(msg.contains("not running"), "{args:?}: {msg}");
        assert!(!socket.exists(), "{args:?} spawned a daemon");
    }

    // A mock daemon is up. A shell that wants ggg observes it: reported
    // as running and incompatible on the provider dimension alone, with
    // exit 0 — and it is still the same daemon afterwards.
    let (_daemon, pid) = start_daemon(&base);
    let out = acq_wanting_ggg(&base, &["daemon", "status", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let report = sole_json(&out);
    assert_eq!(report["running"], true, "{report}");
    assert_eq!(report["compatible"], false, "{report}");
    assert_eq!(report["pid"], pid, "{report}");
    assert_eq!(report["provider"], "mock", "{report}");
    assert_eq!(report["identity_matches"], true, "{report}");
    assert_eq!(report["provider_matches"], false, "{report}");
    assert_eq!(report["wanted"]["provider"], "ggg", "{report}");
    let out = acq_wanting_ggg(&base, &["daemon", "status"]);
    assert!(out.status.success(), "{out:?}");
    let shown = text(&out);
    assert!(
        shown.contains("another provider") && !shown.contains("another build"),
        "{shown}"
    );
    assert!(shown.contains("acq daemon stop"), "{shown}");

    // Reading verbs (the dashboard included) and an acting verb from that
    // shell refuse with the same report; nothing is replaced.
    for args in [
        &["jobs", "--json"][..],
        &["dash", "--json"][..],
        &["cancel", "1", "--json"][..],
    ] {
        let out = acq_wanting_ggg(&base, args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
        assert!(
            msg.contains("another provider") && msg.contains(&format!("pid {pid}")),
            "{args:?}: {msg}"
        );
    }
    let out = acq(&base, &["daemon", "status", "--json"]);
    let status = sole_json(&out);
    assert_eq!(status["pid"], pid, "the daemon was replaced: {status}");
    assert_eq!(status["compatible"], true, "{status}");

    // `daemon stop` from that shell stops the daemon it would not use —
    // the resolution the reports point at — and says which it stopped.
    let out = acq_wanting_ggg(&base, &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let stopped = sole_json(&out);
    assert_eq!(stopped["stopped"], true, "{stopped}");
    assert_eq!(stopped["pid"], pid, "{stopped}");
    assert_eq!(stopped["compatible"], false, "{stopped}");
    let deadline = Instant::now() + Duration::from_secs(10);
    while socket.exists() {
        assert!(Instant::now() < deadline, "the daemon did not exit");
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = acq(&base, &["daemon", "status", "--json"]);
    assert_eq!(sole_json(&out), serde_json::json!({ "running": false }));
    let _ = std::fs::remove_dir_all(&base);
}
