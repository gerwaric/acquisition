//! A daemon refusal through the binary carries its closed kind beside
//! the message under `--json` (C85; additive under C53), and the text
//! mode prints the message alone: the kind is for a program to branch on,
//! never a prefix on prose.

use std::path::{Path, PathBuf};
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

fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

/// The test's scratch directory, removed on drop — declared before the
/// daemon so the daemon is gone first.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Daemon(Child);

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_daemon(base: &Path) -> Daemon {
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
        if sole_json(&out)["pid"].as_u64().is_some() {
            return daemon;
        }
        assert!(Instant::now() < deadline, "the mock daemon did not come up");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn c85_a_refusal_carries_its_kind_in_json_and_only_its_message_in_text() {
    let base = std::env::temp_dir().join(format!("acq-kind-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let _scratch = Scratch(base.clone());
    let _daemon = start_daemon(&base);

    let out = acq(&base, &["status", "999", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert_eq!(
        sole_json(&out),
        serde_json::json!({ "error": "no job 999", "kind": "unknown_job" })
    );

    let out = acq(&base, &["stashes", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let report = sole_json(&out);
    assert_eq!(report["kind"], "not_logged_in", "{report}");
    assert!(
        report["error"].as_str().unwrap().contains("acq auth"),
        "{report}"
    );

    let out = acq(&base, &["status", "999"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(stderr.trim(), "Error: no job 999", "{stderr}");

    // Not a daemon refusal: no kind.
    let _ = acq(&base, &["daemon", "stop"]);
    let out = acq(&base, &["status", "999", "--json"]);
    let report = sole_json(&out);
    assert!(report.get("kind").is_none(), "{report}");
    assert!(
        report["error"].as_str().unwrap().contains("not running"),
        "{report}"
    );
}
