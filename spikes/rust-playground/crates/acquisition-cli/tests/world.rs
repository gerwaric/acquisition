//! The world (C83) through the binaries: the world lock and the
//! real-mode lock, each staged with a second daemon that refuses to start
//! and names the holder; a client on another world refusing the daemon
//! and never replacing it (C83, C10); the rails state moving into the
//! world once so a persisted trip survives (the tripwire's file used to
//! sit in a directory macOS clears at reboot); and the diagnostics under
//! the log directory, bounded by rotation.
//!
//! `ACQ_GGG=1` appears here on two daemons the real-mode lock test
//! starts directly — the owner asked for that lock's process test
//! (2026-09-11) — and on the commands that observe and stop them. Nothing
//! reaches GGG: the daemons start with no session (`ACQ_NO_KEYRING=1`,
//! an empty scratch store, so nothing to restore), nothing is submitted
//! to them, and belt and braces, `ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=0` halts
//! every send before it could happen. The real-mode lock is per OS user
//! at a fixed path, so this test contends with a live daemon of the
//! owner's if one is running while the gate runs — which the standing
//! rule forbids anyway (never run the gate under a live daemon); the
//! failure names the holder's pid.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

/// The scratch directory, removed on drop; declared first so the daemons
/// (declared after) are gone before it goes.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn scratch(tag: &str) -> Scratch {
    let base = std::env::temp_dir().join(format!("acq-w{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    Scratch(base)
}

/// `acq` under one isolation: a store root (the world, whose socket
/// derives from it, C83) and a log directory the caller names under
/// `base`.
fn command(base: &Path, store: &str, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args);
    isolate(&mut cmd, base, store);
    cmd
}

fn isolate(cmd: &mut Command, base: &Path, store: &str) {
    cmd.env("ACQ_STORE_DIR", base.join(store))
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
}

fn acq(base: &Path, store: &str, args: &[&str]) -> Output {
    command(base, store, args).output().expect("spawning acq")
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

/// A daemon the test started and owns, killed on drop, named while a
/// test is panicking (C82: a failure says which daemon ran).
struct Daemon(Child, PathBuf);

impl Drop for Daemon {
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

/// The `acqd` beside the binary under test (C82).
fn acqd() -> PathBuf {
    acquisition_client::locator::beside(Path::new(env!("CARGO_BIN_EXE_acq")))
        .unwrap_or_else(|e| panic!("{e}"))
}

/// The daemon's command under the same isolation as `command`, its
/// stderr piped so a refusal can be read (a lazy spawn's would go to the
/// log; a driver captures stderr as this does).
fn daemon_command(base: &Path, store: &str) -> Command {
    let mut cmd = Command::new(acqd());
    isolate(&mut cmd, base, store);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    cmd
}

/// Start a daemon and wait until `acq daemon status` (under `env`) sees
/// it running; returns it with its pid.
fn start_daemon(base: &Path, store: &str, env: &[(&str, &str)]) -> (Daemon, u64) {
    let mut cmd = daemon_command(base, store);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("spawning {}: {e}", acqd().display()));
    let daemon = Daemon(child, acqd());
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let mut status = command(base, store, &["daemon", "status", "--json"]);
        for (k, v) in env {
            status.env(k, v);
        }
        let out = status.output().unwrap();
        let status = sole_json(&out);
        if let Some(pid) = status["pid"].as_u64() {
            assert_eq!(status["compatible"], true, "{status}");
            return (daemon, pid);
        }
        assert!(
            Instant::now() < deadline,
            "the daemon ({}) did not come up",
            acqd().display()
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Start a daemon that is expected to refuse: wait for it to exit and
/// return its exit status and stderr.
fn refused_daemon(mut cmd: Command) -> (std::process::ExitStatus, String) {
    let child = cmd.spawn().expect("spawning acqd");
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut daemon = Daemon(child, acqd());
    loop {
        if let Some(status) = daemon.0.try_wait().unwrap() {
            let mut stderr = String::new();
            if let Some(mut pipe) = daemon.0.stderr.take() {
                use std::io::Read as _;
                let _ = pipe.read_to_string(&mut stderr);
            }
            return (status, stderr);
        }
        assert!(
            Instant::now() < deadline,
            "the second daemon did not exit: it should have refused"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn daemon_log(base: &Path, store: &str, provider: &str) -> PathBuf {
    let world = acquisition_store::world::World::at(&base.join(store)).expect("the world");
    base.join("logs")
        .join(world.id())
        .join(provider)
        .join("daemon.log")
}

/// `acq` under the isolation plus `env`, applied last.
fn acq_in(base: &Path, store: &str, args: &[&str], env: &[(&str, &str)]) -> Output {
    let mut cmd = command(base, store, args);
    for (k, v) in env {
        cmd.env(k, v);
    }
    cmd.output().expect("spawning acq")
}

/// Wait until `acq daemon status` under `env` says nothing is running.
fn wait_stopped(base: &Path, store: &str, env: &[(&str, &str)]) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = sole_json(&acq_in(base, store, &["daemon", "status", "--json"], env));
        if status["running"] == Value::Bool(false) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the daemon did not exit: {status}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Every Unix socket under `dir`, recursively: what a daemon may have
/// bound in a scratch runtime directory.
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

/// C83, C6: a second daemon on the same world refuses to start, naming
/// the holder by pid and the lock by path, in its stderr and in the
/// world's log; the first daemon is untouched.
#[test]
fn c83_a_second_daemon_on_the_same_world_refuses_naming_the_holder() {
    let scratch = scratch("lock");
    let base = &scratch.0;
    let (_first, pid) = start_daemon(base, "store", &[]);
    let lock = base
        .join("store")
        .canonicalize()
        .unwrap()
        .join("daemon.lock");
    assert!(lock.is_file(), "the world lock file exists");

    // The incumbent's log, grown past the cap while it runs: a contender
    // must not rotate it from under the incumbent (review 2026-09-11 —
    // the locks come before the log is touched).
    let log_path = daemon_log(base, "store", "mock");
    std::fs::OpenOptions::new()
        .append(true)
        .open(&log_path)
        .unwrap()
        .set_len(1u64 << 30)
        .unwrap();

    // Another socket, the same store root: refused on the lock, not on
    // the socket.
    let (status, stderr) = refused_daemon(daemon_command(base, "store"));
    eprintln!("the world lock's refusal, verbatim:\n{stderr}");
    assert!(!status.success(), "the second daemon started: {stderr}");
    assert!(
        !PathBuf::from(format!("{}.1", log_path.display())).exists(),
        "the contender rotated the incumbent's log"
    );
    assert!(
        std::fs::metadata(&log_path).unwrap().len() >= 1u64 << 30,
        "the incumbent's log was replaced"
    );
    assert!(
        stderr.contains("another daemon holds this world")
            && stderr.contains(&format!("pid {pid}"))
            && stderr.contains("daemon.lock")
            && stderr.contains("C83"),
        "{stderr}"
    );
    // The refusal is at the end of a file that is mostly a hole: read
    // the tail only (review 2026-09-11 — a gate must not read a gibibyte
    // into memory).
    let log = {
        use std::io::{Read as _, Seek as _, SeekFrom};
        let mut file = std::fs::File::open(&log_path).unwrap();
        let len = file.metadata().unwrap().len();
        file.seek(SeekFrom::Start(len.saturating_sub(4096)))
            .unwrap();
        let mut tail = Vec::new();
        file.read_to_end(&mut tail).unwrap();
        String::from_utf8_lossy(&tail).into_owned()
    };
    assert!(
        log.contains("STARTUP: another daemon holds this world")
            && log.contains(&format!("pid {pid}")),
        "the refusal reached the incumbent's log (appended, not rotated): {}",
        log.trim_start_matches('\0')
    );
    // A contender under another log directory whose world/provider
    // directory exists but holds no log: refused the same way, and it
    // creates no file there (round 21).
    let other_logs = base.join("other-logs");
    let other_log = other_logs
        .join(
            acquisition_store::world::World::at(&base.join("store"))
                .unwrap()
                .id(),
        )
        .join("mock")
        .join("daemon.log");
    std::fs::create_dir_all(other_log.parent().unwrap()).unwrap();
    let mut contender = daemon_command(base, "store");
    contender.env("ACQ_LOG_DIR", &other_logs);
    let (status, stderr) = refused_daemon(contender);
    assert!(
        !status.success() && stderr.contains("another daemon holds this world"),
        "{stderr}"
    );
    assert!(
        !other_log.exists(),
        "a refused contender created a log under its own directory"
    );

    // The first daemon still answers, and it is the same one.
    let out = acq(base, "store", &["daemon", "status", "--json"]);
    let status = sole_json(&out);
    assert_eq!(status["pid"], pid, "{status}");
    assert_eq!(
        status["world"],
        base.join("store")
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        "{status}"
    );
    let out = acq(base, "store", &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{out:?}");
}

/// C83, C31: a second real-mode daemon for this OS user refuses to start
/// whatever its world, naming the holder and the real-mode lock; a mock
/// daemon on that other world starts, since the lock is real mode's.
/// Nothing reaches GGG (module doc).
#[test]
fn c83_c31_a_second_real_mode_daemon_for_this_user_refuses_naming_the_holder() {
    let scratch = scratch("real");
    let base = &scratch.0;
    let real = [
        ("ACQ_GGG", "1"),
        ("ACQ_TRIPWIRE", "1"),
        ("ACQ_MAX_SENDS", "0"),
    ];
    let (_first, pid) = start_daemon(base, "one", &real);
    let out = command(base, "one", &["daemon", "status", "--json"])
        .env("ACQ_GGG", "1")
        .output()
        .unwrap();
    let status = sole_json(&out);
    assert_eq!(status["provider"], "ggg", "{status}");
    // Belt and braces (module doc): the ceiling is zero, so `Rails::halted`
    // refuses every send before it could happen (`rails.rs`); the status
    // document reports a halt only once a send has tripped one, so it
    // shows the ceiling, not a cause.
    assert_eq!(status["rails"]["max_sends"], 0, "{status}");
    assert_eq!(status["rails"]["tripwire_enabled"], true, "{status}");
    assert_eq!(status["rails"]["sends"], 0, "{status}");

    // Another world, another socket, real mode: refused on the
    // real-mode lock, and the refusal names it.
    let mut second = daemon_command(base, "two");
    for (k, v) in &real {
        second.env(k, v);
    }
    let (status, stderr) = refused_daemon(second);
    eprintln!("the real-mode lock's refusal, verbatim:\n{stderr}");
    assert!(
        !status.success(),
        "the second real-mode daemon started: {stderr}"
    );
    assert!(
        stderr.contains("another real-mode daemon is running for this OS user")
            && stderr.contains(&format!("pid {pid}"))
            && stderr.contains("ggg.lock")
            && stderr.contains("C31"),
        "{stderr}"
    );
    // The refused contender held no lock, so it made nothing in its
    // world's log directory (review 2026-09-11): its refusal is on stderr
    // alone — there was no log to append to yet.
    assert!(
        !daemon_log(base, "two", "ggg").exists(),
        "a refused contender created its world's log"
    );

    // The same other world in mock mode starts: the lock is real mode's.
    let (_mock, mock_pid) = start_daemon(base, "two", &[]);
    assert_ne!(mock_pid, pid);
    let out = acq(base, "two", &["daemon", "status", "--json"]);
    assert_eq!(sole_json(&out)["provider"], "mock");

    let out = command(base, "one", &["daemon", "stop", "--json"])
        .env("ACQ_GGG", "1")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let out = acq(base, "two", &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{out:?}");
}

/// C83, C10, through the binaries: the socket derives from the world,
/// so a shell on another world does not find this daemon at all — its
/// status says not running, a reading verb says so, and a job command
/// starts a daemon of its own on its own socket rather than replacing
/// this one; two worlds run two daemons at once, each shell seeing its
/// own; a shell whose root does not exist has no world, no socket, and
/// creates none by observing; `daemon stop` stops only the shell's own.
#[test]
fn c83_a_shell_on_another_world_has_its_own_rendezvous_and_never_replaces_this_daemon() {
    let scratch = scratch("other");
    let base = &scratch.0;
    std::fs::create_dir_all(base.join("elsewhere")).unwrap();
    let (_daemon, pid) = start_daemon(base, "store", &[]);
    let home = sole_json(&acq(base, "store", &["daemon", "status", "--json"]));
    let home_socket = PathBuf::from(home["socket"].as_str().unwrap());
    assert_eq!(sockets_under(&scratch_tmp(base)), vec![home_socket.clone()]);

    // From another world: not running, and a reading verb says so.
    let out = acq(base, "elsewhere", &["daemon", "status", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out), serde_json::json!({ "running": false }));
    let out = acq(base, "elsewhere", &["jobs", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
    assert!(msg.contains("not running"), "{msg}");
    assert_eq!(
        sockets_under(&scratch_tmp(base)),
        vec![home_socket.clone()],
        "a reading verb spawned a daemon"
    );

    // A job command from there starts that world's daemon — on its own
    // socket, beside this one, which it never touches. (`profile` fails
    // on the login, not on the door; the daemon it spawned stays up.)
    let out = acq(base, "elsewhere", &["profile", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
    assert!(
        !msg.contains("not running") && !msg.contains("another world"),
        "the job command did not reach a daemon of its own: {msg}"
    );
    let out = acq(base, "elsewhere", &["daemon", "status", "--json"]);
    let other = sole_json(&out);
    let other_pid = other["pid"].as_u64().expect("a daemon for the other world");
    assert_ne!(other_pid, pid, "{other}");
    assert_eq!(other["compatible"], true, "{other}");
    let other_socket = PathBuf::from(other["socket"].as_str().unwrap());
    assert_ne!(other_socket, home_socket, "another world, another socket");
    let mut both = vec![home_socket.clone(), other_socket.clone()];
    both.sort();
    assert_eq!(
        sockets_under(&scratch_tmp(base)),
        both,
        "two worlds, two sockets"
    );
    assert_eq!(
        other["world"],
        base.join("elsewhere")
            .canonicalize()
            .unwrap()
            .display()
            .to_string(),
        "{other}"
    );
    let out = acq(base, "store", &["daemon", "status", "--json"]);
    let status = sole_json(&out);
    assert_eq!(status["pid"], pid, "the daemon was replaced: {status}");
    assert_eq!(status["compatible"], true, "{status}");
    assert_eq!(
        status["socket"],
        home_socket.display().to_string(),
        "{status}"
    );

    // No root: no world, no socket, nothing created.
    let out = acq(base, "nowhere", &["daemon", "status", "--json"]);
    assert_eq!(sole_json(&out), serde_json::json!({ "running": false }));
    assert!(
        !base.join("nowhere").exists(),
        "observation created the root"
    );

    // Each shell stops its own and only its own.
    let out = acq(base, "elsewhere", &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let stopped = sole_json(&out);
    assert_eq!(stopped["pid"], other_pid, "{stopped}");
    assert_eq!(stopped["compatible"], true, "{stopped}");
    wait_stopped(base, "elsewhere", &[]);
    let out = acq(base, "store", &["daemon", "status", "--json"]);
    assert_eq!(
        sole_json(&out)["pid"],
        pid,
        "the other world's stop reached this daemon"
    );
    let out = acq(base, "store", &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["pid"], pid);
}

/// C83: a trip persisted beside the legacy socket by a daemon from
/// before step 5 moves into the world at the next start and is honoured
/// there; the legacy file is gone; `reset-tripwire` with no daemon clears
/// the world's file. The legacy file's home is the temp directory: the
/// scratch one every daemon and shell here runs under (`scratch_tmp`).
#[test]
fn c83_the_rails_state_moves_into_the_world_and_the_trip_survives() {
    let scratch = scratch("rails");
    let base = &scratch.0;
    let tmp = scratch_tmp(base);
    let env = [("ACQ_TRIPWIRE", "1")];
    let legacy = tmp.join("acquisition-playground.mock.rails.json");
    std::fs::write(
        &legacy,
        r#"{"tripped":"429 on GET /stash/Standard (staged legacy trip)"}"#,
    )
    .unwrap();

    let (_daemon, _pid) = start_daemon(base, "store", &env);
    let out = acq_in(base, "store", &["daemon", "status", "--json"], &env);
    let status = sole_json(&out);
    assert!(
        status["socket"]
            .as_str()
            .unwrap()
            .starts_with(&tmp.display().to_string()),
        "the daemon's socket is in the scratch runtime directory: {status}"
    );
    assert_eq!(status["rails"]["tripwire_enabled"], true, "{status}");
    assert!(
        status["rails"]["halted"]
            .as_str()
            .is_some_and(|h| h.contains("staged legacy trip")),
        "the trip did not survive the move: {status}"
    );
    assert!(!legacy.exists(), "the legacy file was left behind");
    let current = base.join("store").join("mock").join("rails.json");
    assert!(
        std::fs::read_to_string(&current)
            .unwrap()
            .contains("staged legacy trip"),
        "the trip is not in the world"
    );
    let log = std::fs::read_to_string(daemon_log(base, "store", "mock")).unwrap();
    assert!(
        log.contains("rails: state moved from") && log.contains("staged legacy trip"),
        "{log}"
    );
    let out = acq_in(base, "store", &["daemon", "stop", "--json"], &env);
    assert!(out.status.success(), "{out:?}");
    wait_stopped(base, "store", &env);

    // With no daemon, the reset clears the world's file so the next
    // daemon starts clear.
    let out = acq_in(base, "store", &["daemon", "reset-tripwire", "--json"], &env);
    assert!(out.status.success(), "{out:?}");
    let cleared = sole_json(&out);
    assert_eq!(cleared["cleared"], true, "{cleared}");
    // The file is gone, so compare its directory (canonical on both
    // sides) and its name.
    let state = PathBuf::from(cleared["state"].as_str().unwrap());
    assert_eq!(
        state.parent().unwrap().canonicalize().unwrap(),
        current.parent().unwrap().canonicalize().unwrap(),
        "{cleared}"
    );
    assert_eq!(state.file_name(), current.file_name(), "{cleared}");
    assert!(!current.exists(), "the world's rails state was not cleared");
    let out = acq_in(base, "store", &["daemon", "reset-tripwire", "--json"], &env);
    assert_eq!(sole_json(&out)["cleared"], false);

    // A directory in a state file's place (round 21): the daemon refuses
    // to start over it, naming this verb; the verb clears an empty one
    // and names a non-empty one for the hand.
    std::fs::create_dir(&legacy).unwrap();
    let mut cmd = daemon_command(base, "store");
    for (k, v) in &env {
        cmd.env(k, v);
    }
    let (status, stderr) = refused_daemon(cmd);
    assert!(
        !status.success()
            && stderr.contains("could not be read")
            && stderr.contains("reset-tripwire"),
        "{stderr}"
    );
    let out = acq_in(base, "store", &["daemon", "reset-tripwire", "--json"], &env);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["cleared"], true);
    assert!(!legacy.exists(), "the empty directory was cleared");
    std::fs::create_dir_all(legacy.join("inside")).unwrap();
    let out = acq_in(base, "store", &["daemon", "reset-tripwire", "--json"], &env);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
    assert!(
        msg.contains("not empty") && msg.contains("by hand"),
        "{msg}"
    );
    assert!(
        legacy.join("inside").exists(),
        "a non-empty directory was touched"
    );
    std::fs::remove_dir_all(&legacy).unwrap();
}

/// C83: the log and the default journal live under the log directory,
/// one subdirectory per world and provider, beside a `world` file naming
/// the root; they are bounded — a log over the cap is rotated once at
/// the daemon's start to `daemon.log.1`, and `daemon status` reports the
/// paths.
#[test]
fn c83_diagnostics_live_under_the_log_directory_and_are_bounded() {
    let scratch = scratch("logs");
    let base = &scratch.0;
    std::fs::create_dir_all(base.join("store")).unwrap();
    let log = daemon_log(base, "store", "mock");
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    // A log the last lifetime left over the cap — a gibibyte, sparse, so
    // the test stages no bytes and pins no number: the length is what
    // rotation reads (the cap is the daemon's, `daemon.rs`).
    let over = 1u64 << 30;
    std::fs::File::create(&log).unwrap().set_len(over).unwrap();

    // This daemon keeps the default journal (no `ACQ_JOURNAL`).
    let mut cmd = daemon_command(base, "store");
    cmd.env_remove("ACQ_JOURNAL");
    let child = cmd.spawn().unwrap();
    let _daemon = Daemon(child, acqd());
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        let out = acq(base, "store", &["daemon", "status", "--json"]);
        let status = sole_json(&out);
        if status["pid"].is_number() {
            break status;
        }
        assert!(Instant::now() < deadline, "the daemon did not come up");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(status["log"], log.display().to_string(), "{status}");
    assert_eq!(
        status["rails"]["journal"],
        log.with_file_name("sends.jsonl").display().to_string(),
        "{status}"
    );
    // The path is the daemon's own report: a shell with another log
    // directory, or none, still learns the file the daemon opened
    // (review 2026-09-11).
    for other in [Some(base.join("elsewhere-logs")), None] {
        let mut cmd = command(base, "store", &["daemon", "status", "--json"]);
        match &other {
            Some(dir) => {
                cmd.env("ACQ_LOG_DIR", dir);
            }
            None => {
                cmd.env_remove("ACQ_LOG_DIR");
            }
        }
        let out = cmd.output().unwrap();
        let status = sole_json(&out);
        assert_eq!(
            status["log"],
            log.display().to_string(),
            "{other:?}: {status}"
        );
        let out = {
            let mut cmd = command(base, "store", &["daemon", "status"]);
            match &other {
                Some(dir) => {
                    cmd.env("ACQ_LOG_DIR", dir);
                }
                None => {
                    cmd.env_remove("ACQ_LOG_DIR");
                }
            }
            cmd.output().unwrap()
        };
        assert!(
            text(&out).contains(&format!("log:    {}", log.display())),
            "{other:?}: {}",
            text(&out)
        );
    }
    let previous = PathBuf::from(format!("{}.1", log.display()));
    assert_eq!(
        std::fs::metadata(&previous).unwrap().len(),
        over,
        "the over-cap log became the previous generation"
    );
    let now = std::fs::read_to_string(&log).unwrap();
    assert!(
        now.len() < 4096,
        "the current log is fresh: {} bytes",
        now.len()
    );
    assert!(
        now.contains("log: rotated") && now.contains("daemon.log.1"),
        "{now}"
    );
    assert!(now.contains("world: "), "{now}");
    let marker = log.parent().unwrap().parent().unwrap().join("world");
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap().trim(),
        base.join("store")
            .canonicalize()
            .unwrap()
            .display()
            .to_string()
    );
    let out = acq(base, "store", &["daemon", "stop", "--json"]);
    assert!(out.status.success(), "{out:?}");

    // A log directory that is not valid UTF-8 refuses the start, before
    // anything is made: the path crosses the wire exactly or not at all
    // (round 21's pin).
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let odd = std::ffi::OsString::from(std::ffi::OsStr::from_bytes(b"/tmp/acq-\xff-logs"));
        let mut cmd = daemon_command(base, "store");
        cmd.env("ACQ_LOG_DIR", &odd);
        let (status, stderr) = refused_daemon(cmd);
        assert!(
            !status.success()
                && stderr.contains("not valid UTF-8")
                && stderr.contains("ACQ_LOG_DIR"),
            "{stderr}"
        );
        assert!(
            sockets_under(&scratch_tmp(base)).is_empty(),
            "the refused daemon bound the world's socket"
        );
    }
}

/// A scripted peer on the legacy socket: what a daemon from before the
/// derived rendezvous looks like to a client. Answers `hello` as pid
/// 4242 of another contract on the world the test names, `daemon_stop`
/// with `stopping` — then unbinds and counts the stop — and hangs up on
/// anything else; the daemon's bare probe (a connect that closes) is
/// tolerated.
struct LegacyPeer {
    thread: Option<std::thread::JoinHandle<u32>>,
}

impl LegacyPeer {
    fn bind(path: &Path, world: &str) -> LegacyPeer {
        use std::io::{BufRead as _, BufReader, Write as _};
        let _ = std::fs::remove_file(path);
        let listener = std::os::unix::net::UnixListener::bind(path).unwrap();
        let path = path.to_path_buf();
        let world = world.to_string();
        let thread = std::thread::spawn(move || {
            let mut stops = 0;
            'accept: for stream in listener.incoming() {
                let mut write = stream.unwrap();
                let mut reader = BufReader::new(write.try_clone().unwrap());
                let mut line = String::new();
                loop {
                    line.clear();
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
                    let Ok(frame) = serde_json::from_str::<Value>(&line) else {
                        break;
                    };
                    match frame["req"].as_str() {
                        Some("hello") => {
                            let reply = serde_json::json!({
                                "resp": "hello", "version": "0.0.1", "contract": "beefbeefbeef",
                                "pid": 4242, "provider": "mock", "world": world,
                            });
                            writeln!(write, "{reply}").unwrap();
                        }
                        Some("daemon_stop") => {
                            stops += 1;
                            writeln!(write, r#"{{"resp":"stopping"}}"#).unwrap();
                            let _ = std::fs::remove_file(&path);
                            break 'accept;
                        }
                        _ => break,
                    }
                }
            }
            stops
        });
        LegacyPeer {
            thread: Some(thread),
        }
    }

    /// How many stops the peer answered; waits for it to unbind.
    fn stops(mut self) -> u32 {
        self.thread.take().unwrap().join().unwrap()
    }
}

/// C83, the split's step 6 — the transition from the fixed socket to the
/// derived one, through the binaries, in a scratch temp and runtime
/// directory: with a daemon from before the rendezvous listening on the
/// legacy socket, `acqd` refuses to start naming it and the stop remedy;
/// `daemon status` finds it when the world's socket is silent and reports
/// it with `legacy_socket`; a reading verb refuses; the use door that
/// would replace a contract mismatch refuses and spawns nothing; `daemon
/// stop` stops it, once — and then this world's daemon starts on the
/// derived socket, the legacy path left alone.
#[test]
fn c83_a_daemon_from_before_the_rendezvous_is_refused_reported_and_stopped_once() {
    let scratch = scratch("legacy");
    let base = &scratch.0;
    let tmp = scratch_tmp(base);
    let env: [(&str, &str); 0] = [];
    std::fs::create_dir_all(base.join("store")).unwrap();
    let home = base.join("store").canonicalize().unwrap();
    let legacy = tmp.join("acquisition-playground.sock");
    let peer = LegacyPeer::bind(&legacy, &home.display().to_string());

    // The daemon refuses to start beside it, before it binds anything.
    let mut cmd = daemon_command(base, "store");
    for (k, v) in &env {
        cmd.env(k, v);
    }
    let (status, stderr) = refused_daemon(cmd);
    eprintln!("the legacy socket's refusal, verbatim:\n{stderr}");
    assert!(!status.success(), "the daemon started: {stderr}");
    assert!(
        stderr.contains("legacy socket")
            && stderr.contains(&legacy.display().to_string())
            && stderr.contains("acq daemon stop"),
        "{stderr}"
    );
    assert_eq!(
        sockets_under(&tmp),
        vec![legacy.clone()],
        "the daemon bound a socket"
    );

    // Observed: running, not this client's, on the legacy socket.
    let status = sole_json(&acq_in(
        base,
        "store",
        &["daemon", "status", "--json"],
        &env,
    ));
    assert_eq!(status["running"], true, "{status}");
    assert_eq!(status["compatible"], false, "{status}");
    assert_eq!(status["pid"], 4242, "{status}");
    assert_eq!(status["legacy_socket"], true, "{status}");
    assert_eq!(status["socket"], legacy.display().to_string(), "{status}");
    assert_eq!(status["world_matches"], true, "{status}");
    let shown = text(&acq_in(base, "store", &["daemon", "status"], &env));
    assert!(
        shown.contains("legacy socket")
            && shown.contains(&format!("socket: {}", legacy.display()))
            && shown.contains("next:")
            && shown.contains("once"),
        "{shown}"
    );

    // A reading verb refuses; the use door — which replaces a contract
    // mismatch on the world's socket — refuses too and spawns nothing.
    let out = acq_in(base, "store", &["jobs", "--json"], &env);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
    assert!(msg.contains("legacy socket"), "{msg}");
    let out = acq_in(base, "store", &["profile", "--json"], &env);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
    assert!(
        msg.contains("legacy socket") && msg.contains("acq daemon stop") && msg.contains("once"),
        "{msg}"
    );
    assert_eq!(
        sockets_under(&tmp),
        vec![legacy.clone()],
        "a daemon was spawned"
    );

    // Stopped, once, by hand.
    let out = acq_in(base, "store", &["daemon", "stop", "--json"], &env);
    assert!(out.status.success(), "{out:?}");
    let stopped = sole_json(&out);
    assert_eq!(stopped["stopped"], true, "{stopped}");
    assert_eq!(stopped["pid"], 4242, "{stopped}");
    assert_eq!(stopped["legacy_socket"], true, "{stopped}");
    assert_eq!(stopped["socket"], legacy.display().to_string(), "{stopped}");
    assert_eq!(peer.stops(), 1, "the peer was stopped exactly once");
    assert!(sockets_under(&tmp).is_empty(), "the legacy socket is gone");

    // From then on the world's socket is the rendezvous.
    let (_daemon, pid) = start_daemon(base, "store", &env);
    let status = sole_json(&acq_in(
        base,
        "store",
        &["daemon", "status", "--json"],
        &env,
    ));
    assert_eq!(status["pid"], pid, "{status}");
    assert_eq!(status["legacy_socket"], false, "{status}");
    let socket = PathBuf::from(status["socket"].as_str().unwrap());
    assert_ne!(socket, legacy);
    assert!(socket.starts_with(&tmp), "{}", socket.display());
    assert_eq!(
        socket.file_name().unwrap(),
        format!(
            "{}.sock",
            acquisition_store::world::World::at(&home).unwrap().id()
        )
        .as_str()
    );
    assert_eq!(sockets_under(&tmp), vec![socket.clone()]);
    let out = acq_in(base, "store", &["daemon", "stop", "--json"], &env);
    assert!(out.status.success(), "{out:?}");
    wait_stopped(base, "store", &env);
    assert!(!socket.exists(), "the socket is removed on exit");
}

/// C83: a socket path over the cap is refused by name — by the daemon
/// before it binds, and by a shell before it connects — rather than by
/// `bind`'s `ENAMETOOLONG`. Staged with a runtime directory deep enough
/// to push the path past 103 bytes.
#[test]
fn c83_a_socket_path_over_the_cap_is_refused_by_name() {
    let scratch = scratch("cap");
    let base = &scratch.0;
    let deep = base.join("tmp").join("x".repeat(96));
    std::fs::create_dir_all(&deep).unwrap();
    let deep_s = deep.display().to_string();
    let env = [
        ("TMPDIR", deep_s.as_str()),
        ("XDG_RUNTIME_DIR", deep_s.as_str()),
    ];
    let mut cmd = daemon_command(base, "store");
    for (k, v) in &env {
        cmd.env(k, v);
    }
    let (status, stderr) = refused_daemon(cmd);
    assert!(
        !status.success() && stderr.contains("at most 103"),
        "{stderr}"
    );
    assert!(sockets_under(&deep).is_empty());
    let out = acq_in(base, "store", &["daemon", "status", "--json"], &env);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
    assert!(msg.contains("at most 103"), "{msg}");
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
