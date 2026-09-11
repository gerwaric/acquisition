//! The MCP's `daemon_status` through the binary (C10, C84): absence is a
//! state; a running daemon this server may use is reported with its
//! identity — contract, the executable it runs from and its hash, how
//! that file relates to the `acqd` beside `acq-mcp` (`artifact_relation`)
//! and that sibling under `wanted`; a daemon of another provider is
//! reported as incompatible with the dimension named, and never
//! replaced. `ACQ_GGG=1` appears only on the observing server, which
//! cannot spawn in real mode by construction: nothing reaches GGG.

mod harness;

use std::path::{Path, PathBuf};
use std::process::Child;
use std::time::{Duration, Instant};

use harness::{Mcp, spawn_daemon};
use serde_json::{Value, json};

/// The scratch directory, removed on drop — a failed assertion leaves
/// nothing behind. Declared before the daemon, so the daemon is gone
/// first.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The daemon the test started, killed on drop and named while a test
/// is panicking (C82: a failure says which daemon ran).
struct Daemon(Child, PathBuf);

impl Drop for Daemon {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "the daemon under test was {} (pid {})",
                self.1.display(),
                self.0.id()
            );
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn c84_daemon_status_reports_the_identity_and_the_sibling_it_is_judged_against() {
    let base = std::env::temp_dir().join(format!("acq-mcp-status-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let _scratch = Scratch(base.clone());
    let acqd = acquisition_client::locator::beside(Path::new(env!("CARGO_BIN_EXE_acq-mcp")))
        .unwrap_or_else(|e| panic!("{e}"))
        .canonicalize()
        .unwrap();

    // Absent: a state, not an error — and no daemon appears.
    let mut mcp = Mcp::start(&base, &[]);
    assert_eq!(
        mcp.expect_ok("daemon_status", json!({})),
        json!({ "running": false })
    );
    assert!(
        !base.join("d.sock").exists(),
        "daemon_status spawned a daemon"
    );

    // Running and this server's: the vitals, and the identity keys the
    // incompatible report has, from one look at the sibling.
    let daemon = Daemon(spawn_daemon(&base, &[]), acqd.clone());
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        let status = mcp.expect_ok("daemon_status", json!({}));
        if status["running"] == json!(true) {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "the mock daemon ({}) did not come up",
            acqd.display()
        );
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(status["compatible"], true, "{status}");
    assert_eq!(status["provider"], "mock", "{status}");
    assert!(
        status["contract"].as_str().is_some_and(|c| c.len() == 12),
        "{status}"
    );
    assert_eq!(
        Path::new(status["artifact"]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        acqd,
        "{status}"
    );
    let sha = status["artifact"]["sha256"].as_str().unwrap();
    assert!(
        sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
        "{status}"
    );
    assert_eq!(status["artifact_relation"], "same_file", "{status}");
    assert_eq!(
        Path::new(status["wanted"]["acqd"]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        acqd,
        "{status}"
    );
    assert_eq!(status["wanted"]["contract"], status["contract"], "{status}");
    assert!(
        status["uptime_seconds"].is_number() && status["rails"].is_object(),
        "{status}"
    );
    let pid = status["pid"].as_u64().unwrap();
    assert_eq!(pid, u64::from(daemon.0.id()), "{status}");

    // A server that wants ggg observes the mock daemon: incompatible on
    // the provider dimension alone, the artifact still its sibling, the
    // daemon left running.
    let mut real = Mcp::start(&base, &[("ACQ_GGG", "1")]);
    let report = real.expect_ok("daemon_status", json!({}));
    assert_eq!(report["running"], true, "{report}");
    assert_eq!(report["compatible"], false, "{report}");
    assert_eq!(report["pid"], pid, "{report}");
    assert_eq!(report["contract_matches"], true, "{report}");
    assert_eq!(report["artifact_matches"], true, "{report}");
    assert_eq!(report["artifact_relation"], "same_file", "{report}");
    assert_eq!(report["provider_matches"], false, "{report}");
    assert_eq!(report["wanted"]["provider"], "ggg", "{report}");
    assert_eq!(
        report["artifact"]["sha256"],
        Value::String(sha.to_string()),
        "{report}"
    );
    let again = mcp.expect_ok("daemon_status", json!({}));
    assert_eq!(again["pid"], pid, "the daemon was replaced: {again}");
    // The guards kill the daemon and remove the scratch directory.
}
