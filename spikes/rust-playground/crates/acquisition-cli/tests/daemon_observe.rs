//! C10's observation tier through the real binary: an observer never
//! spawns or replaces a daemon, and reports absence, contract mismatch,
//! artifact mismatch and provider mismatch distinctly (C84). The trap the
//! first test pins is the ledger row of 2026-09-08: `acq daemon status`
//! typed in a shell without `ACQ_GGG` while a real-mode daemon ran
//! replaced it with a mock one. Provider mismatch is staged by a mock
//! daemon observed from a client that wants ggg. The artifact dimension
//! is staged with one build's binaries (the second test): a copy of `acq`
//! whose sibling `acqd` is another file sees the running daemon as another
//! artifact, a copy whose sibling is a copy of `acqd` sees the same one by
//! hash, and a copy with no sibling at all matches nothing. The contract
//! dimension takes the same path (`DaemonId::verdict`) and is pinned at
//! the unit level and in the client's contract tests, where a scripted
//! peer can claim any contract.
//!
//! `ACQ_GGG=1` appears here only on observing and stopping commands,
//! which cannot spawn by construction: no daemon in real mode ever exists
//! in this test, and nothing reaches GGG.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

fn command(base: &Path, args: &[&str]) -> Command {
    command_of(Path::new(env!("CARGO_BIN_EXE_acq")), base, args)
}

/// The same isolation for an `acq` at another path (a copy staged for
/// the artifact dimension).
fn command_of(exe: &Path, base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(exe);
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
struct Daemon(Child, PathBuf);

impl Drop for Daemon {
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

/// Every Unix socket under `dir`, recursively: what a daemon may have
/// bound in the scratch runtime directory.
fn sockets_under(dir: &Path) -> Vec<PathBuf> {
    use std::os::unix::fs::FileTypeExt;
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return found;
    };
    for entry in entries.flatten() {
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            found.extend(sockets_under(&entry.path()));
        } else if kind.is_socket() {
            found.push(entry.path());
        }
    }
    found.sort();
    found
}

/// The daemon `acq` would itself spawn: the `acqd` beside the binary
/// under test (C82, the locator's one rule). A missing one fails here,
/// before the test runs, naming the build step.
fn acqd() -> PathBuf {
    acquisition_client::locator::beside(Path::new(env!("CARGO_BIN_EXE_acq")))
        .unwrap_or_else(|e| panic!("{e}"))
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

fn start_daemon(base: &Path) -> (Daemon, u64) {
    let acqd = acqd();
    let child = daemon_command(base, &acqd)
        .spawn()
        .unwrap_or_else(|e| panic!("spawning {}: {e}", acqd.display()));
    let daemon = Daemon(child, acqd.clone());
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
        assert!(
            Instant::now() < deadline,
            "the mock daemon ({}) did not come up",
            acqd.display()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn c10_observation_never_spawns_or_replaces_and_reports_the_mismatch() {
    let base = std::env::temp_dir().join(format!("acq-obs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();

    // Absent: reported as a state by `daemon status`, as an error by a
    // reading verb — and neither spawns: the world's root is never
    // created (a spawn creates it first, C83), so no socket can derive
    // from it.
    let out = acq(&base, &["daemon", "status", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out), serde_json::json!({ "running": false }));
    for args in [&["jobs", "--json"][..], &["dash", "--json"][..]] {
        let out = acq(&base, args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
        assert!(msg.contains("not running"), "{args:?}: {msg}");
        assert!(
            !base.join("store").exists(),
            "{args:?} created the world's root — a spawn"
        );
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
    assert_eq!(report["contract_matches"], true, "{report}");
    assert_eq!(report["artifact_matches"], true, "{report}");
    assert_eq!(report["provider_matches"], false, "{report}");
    assert_eq!(report["world_matches"], true, "{report}");
    assert_eq!(report["wanted"]["provider"], "ggg", "{report}");
    let out = acq_wanting_ggg(&base, &["daemon", "status"]);
    assert!(out.status.success(), "{out:?}");
    let shown = text(&out);
    assert!(
        shown.contains("another provider")
            && !shown.contains("another contract")
            && !shown.contains("another artifact"),
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
    assert_eq!(stopped["legacy_socket"], false, "{stopped}");
    // The socket it was stopped through: this world's, in the scratch
    // runtime directory (C83), gone once the daemon has exited.
    let socket = PathBuf::from(stopped["socket"].as_str().unwrap());
    assert!(
        socket.starts_with(scratch_tmp(&base)) && socket.extension().is_some_and(|e| e == "sock"),
        "{stopped}"
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    while socket.exists() {
        assert!(Instant::now() < deadline, "the daemon did not exit");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(sockets_under(&scratch_tmp(&base)).is_empty());
    let out = acq(&base, &["daemon", "status", "--json"]);
    assert_eq!(sole_json(&out), serde_json::json!({ "running": false }));
    let _ = std::fs::remove_dir_all(&base);
}

/// A copy of the binary under test in `dir`, with `sibling` — if any —
/// placed beside it as `acqd`.
fn stage(dir: &Path, sibling: Option<&Path>) -> PathBuf {
    std::fs::create_dir_all(dir).unwrap();
    let acq = dir.join("acq");
    std::fs::copy(env!("CARGO_BIN_EXE_acq"), &acq).unwrap();
    if let Some(sibling) = sibling {
        std::fs::copy(sibling, dir.join("acqd")).unwrap();
    }
    acq
}

/// C84's artifact dimension through the binaries: the running daemon is
/// the `acqd` beside `acq`; a copy of `acq` whose sibling is a copy of
/// that daemon judges it the same artifact (different inode, same bytes:
/// the hash path); one whose sibling is another file judges it another
/// artifact and reports both sides, and never replaces it; one with no
/// sibling matches nothing and says it could not have started it. The
/// compatible report carries the daemon's contract and artifact beside
/// the vitals.
#[test]
fn c84_the_artifact_dimension_is_the_sibling_acqd_this_client_would_start() {
    let base = std::env::temp_dir().join(format!("acq-art-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let (_daemon, pid) = start_daemon(&base);
    let real_acqd = acqd();

    // The binary under test, whose sibling is the daemon itself.
    let out = acq(&base, &["daemon", "status", "--json"]);
    let status = sole_json(&out);
    assert_eq!(status["compatible"], true, "{status}");
    assert!(
        status["contract"].as_str().is_some_and(|c| c.len() == 12),
        "{status}"
    );
    let reported = &status["artifact"];
    assert_eq!(
        Path::new(reported["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        real_acqd.canonicalize().unwrap(),
        "{status}"
    );
    let sha = reported["sha256"].as_str().unwrap().to_string();
    assert!(
        sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
        "{status}"
    );
    // The compatible report says how the daemon's file relates to this
    // client's sibling, and names the sibling (review 2026-09-10); the
    // three dimensions are there in both shapes.
    assert_eq!(status["artifact_relation"], "same_file", "{status}");
    for key in [
        "contract_matches",
        "artifact_matches",
        "provider_matches",
        "world_matches",
    ] {
        assert_eq!(status[key], true, "{key}: {status}");
    }
    // The world (C83): the daemon's canonical root is this shell's store,
    // and the paths this shell resolves are reported beside it.
    assert_eq!(
        status["world"],
        base.join("store")
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        "{status}"
    );
    assert_eq!(status["wanted"]["world"], status["world"], "{status}");
    // The socket (C83): derived from the world into the runtime
    // directory — the one file bound there, named by the world's id —
    // the one this shell reached the daemon through.
    let world = acquisition_store::world::World::at(&base.join("store")).unwrap();
    assert_eq!(
        sockets_under(&scratch_tmp(&base)),
        vec![PathBuf::from(status["socket"].as_str().unwrap())],
        "{status}"
    );
    assert!(
        status["socket"]
            .as_str()
            .unwrap()
            .ends_with(&format!("/{}", world.socket_name())),
        "{status}"
    );
    assert_eq!(status["legacy_socket"], false, "{status}");
    assert!(
        status["log"]
            .as_str()
            .unwrap()
            .starts_with(&base.join("logs").display().to_string()),
        "{status}"
    );
    assert_eq!(
        Path::new(status["wanted"]["acqd"]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        real_acqd.canonicalize().unwrap(),
        "{status}"
    );
    let out = acq(&base, &["daemon", "status"]);
    let shown = text(&out);
    assert!(
        shown.contains(&sha[..12])
            && shown.contains("this client's sibling")
            && !shown.contains("another copy"),
        "{shown}"
    );

    // A copy of acq beside a copy of acqd: another inode, the same bytes —
    // the same artifact, settled by hash, and reported as a copy: the
    // daemon's path is the original, the sibling is the copy.
    let copy = stage(&base.join("copy"), Some(&real_acqd));
    let copied_acqd = base.join("copy").join("acqd").canonicalize().unwrap();
    let out = command_of(&copy, &base, &["daemon", "status", "--json"])
        .output()
        .unwrap();
    let status = sole_json(&out);
    assert_eq!(status["compatible"], true, "{status}");
    assert_eq!(status["pid"], pid, "{status}");
    assert_eq!(status["artifact_relation"], "same_bytes", "{status}");
    assert_eq!(
        Path::new(status["artifact"]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        real_acqd.canonicalize().unwrap(),
        "the daemon's file is the original: {status}"
    );
    assert_eq!(
        Path::new(status["wanted"]["acqd"]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        copied_acqd,
        "the sibling is the copy: {status}"
    );
    let out = command_of(&copy, &base, &["daemon", "status"])
        .output()
        .unwrap();
    let shown = text(&out);
    assert!(
        shown.contains("another copy") && shown.contains(&copied_acqd.display().to_string()),
        "{shown}"
    );

    // A copy of acq beside another file named acqd (here: acq itself):
    // another artifact — reported with both hashes, the daemon left alone.
    let bogus = stage(
        &base.join("bogus"),
        Some(Path::new(env!("CARGO_BIN_EXE_acq"))),
    );
    let out = command_of(&bogus, &base, &["daemon", "status", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let report = sole_json(&out);
    assert_eq!(report["running"], true, "{report}");
    assert_eq!(report["compatible"], false, "{report}");
    assert_eq!(report["contract_matches"], true, "{report}");
    assert_eq!(report["artifact_matches"], false, "{report}");
    assert_eq!(report["artifact_relation"], "different", "{report}");
    assert_eq!(report["provider_matches"], true, "{report}");
    assert_eq!(report["artifact"]["sha256"], sha, "{report}");
    assert_eq!(
        Path::new(report["wanted"]["acqd"]["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        base.join("bogus").join("acqd").canonicalize().unwrap(),
        "{report}"
    );
    assert!(
        report["artifact_mismatch"]
            .as_str()
            .unwrap()
            .contains("another file"),
        "{report}"
    );
    let out = command_of(&bogus, &base, &["daemon", "status"])
        .output()
        .unwrap();
    let shown = text(&out);
    assert!(
        shown.contains("another artifact")
            && !shown.contains("another contract")
            && !shown.contains("another provider")
            && shown.contains("acq daemon stop"),
        "{shown}"
    );
    for args in [&["jobs", "--json"][..], &["cancel", "1", "--json"][..]] {
        let out = command_of(&bogus, &base, args).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{args:?}: {out:?}");
        let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
        assert!(
            msg.contains("another artifact") && msg.contains(&format!("pid {pid}")),
            "{args:?}: {msg}"
        );
    }

    // A copy of acq with no acqd beside it: it could not have started the
    // daemon, and says so.
    let alone = stage(&base.join("alone"), None);
    let out = command_of(&alone, &base, &["daemon", "status", "--json"])
        .output()
        .unwrap();
    let report = sole_json(&out);
    assert_eq!(report["compatible"], false, "{report}");
    assert_eq!(report["artifact_matches"], false, "{report}");
    assert_eq!(report["artifact_relation"], "no_sibling", "{report}");
    assert_eq!(report["wanted"]["acqd"], Value::Null, "{report}");
    assert!(
        report["wanted"]["acqd_absent"]
            .as_str()
            .unwrap()
            .contains("no acqd beside"),
        "{report}"
    );
    let out = command_of(&alone, &base, &["version", "--json"])
        .output()
        .unwrap();
    let version = sole_json(&out);
    assert_eq!(version["acqd"], Value::Null, "{version}");
    assert_eq!(version["contract"], status["contract"], "{version}");

    // Nothing above replaced the daemon.
    let out = acq(&base, &["daemon", "status", "--json"]);
    let status = sole_json(&out);
    assert_eq!(status["pid"], pid, "the daemon was replaced: {status}");
    let out = acq(&base, &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let _ = std::fs::remove_dir_all(&base);
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
