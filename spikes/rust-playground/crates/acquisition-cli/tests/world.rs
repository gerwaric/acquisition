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
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

mod harness;

use harness::{Proc, acqd, scratch, scratch_tmp, sockets_under, sole_json, text};

/// `acq` under one isolation: a store root (the world, whose socket
/// derives from it, C83) and a log directory the caller names under
/// `base`.
fn command(base: &Path, store: &str, args: &[&str]) -> Command {
    harness::command_in(base, store, args)
}

fn acq(base: &Path, store: &str, args: &[&str]) -> Output {
    command(base, store, args).output().expect("spawning acq")
}

/// The daemon's command under the same isolation as `command`, its
/// stderr piped so a refusal can be read (a lazy spawn's would go to the
/// log; a driver captures stderr as this does).
fn daemon_command(base: &Path, store: &str) -> Command {
    let mut cmd = Command::new(acqd());
    harness::isolate_in(&mut cmd, base, store);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    cmd
}

/// Start a daemon and wait until `acq daemon status` (under `env`) sees
/// it running; returns it with its pid.
fn start_daemon(base: &Path, store: &str, env: &[(&str, &str)]) -> (Proc, u64) {
    let mut cmd = daemon_command(base, store);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let child = cmd
        .spawn()
        .unwrap_or_else(|e| panic!("spawning {}: {e}", acqd().display()));
    let daemon = Proc(child, acqd());
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
    let mut daemon = Proc(child, acqd());
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

    // From another world: not running, and a reading verb says so — with
    // nothing on disk there to read (C45: a root, no ledger).
    let absent = serde_json::json!({ "running": false, "persisted": null });
    let out = acq(base, "elsewhere", &["daemon", "status", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out), absent);
    let out = acq(base, "elsewhere", &["jobs", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out), absent);
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
    assert_eq!(
        sole_json(&out),
        serde_json::json!({ "running": false, "persisted": null })
    );
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

/// C83: the rails state persists in the world — a trip written there
/// halts the next daemon on that root; `reset-tripwire` with no daemon
/// clears the world's file, clears an empty directory in its place (the
/// daemon refuses to start over one, naming the verb) and names a
/// non-empty one for the hand.
#[test]
fn c83_the_rails_state_lives_in_the_world_and_reset_tripwire_clears_it() {
    let scratch = scratch("rails");
    let base = &scratch.0;
    let env = [("ACQ_TRIPWIRE", "1")];
    let current = base.join("store").join("mock").join("rails.json");
    std::fs::create_dir_all(current.parent().unwrap()).unwrap();
    std::fs::write(
        &current,
        r#"{"tripped":"429 on GET /stash/Standard (staged trip)"}"#,
    )
    .unwrap();

    let (_daemon, _pid) = start_daemon(base, "store", &env);
    let out = acq_in(base, "store", &["daemon", "status", "--json"], &env);
    let status = sole_json(&out);
    assert_eq!(status["rails"]["tripwire_enabled"], true, "{status}");
    assert!(
        status["rails"]["halted"]
            .as_str()
            .is_some_and(|h| h.contains("staged trip")),
        "the trip in the world was not honoured: {status}"
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

    // A directory in the state file's place (round 21): the daemon
    // refuses to start over it, naming this verb; the verb clears an
    // empty one and names a non-empty one for the hand.
    std::fs::create_dir(&current).unwrap();
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
    assert!(!current.exists(), "the empty directory was cleared");
    std::fs::create_dir_all(current.join("inside")).unwrap();
    let out = acq_in(base, "store", &["daemon", "reset-tripwire", "--json"], &env);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
    assert!(
        msg.contains("not empty") && msg.contains("by hand"),
        "{msg}"
    );
    assert!(
        current.join("inside").exists(),
        "a non-empty directory was touched"
    );
    std::fs::remove_dir_all(&current).unwrap();
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
    let _daemon = Proc(child, acqd());
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

/// C83: a runtime directory the environment names in bytes that are not
/// UTF-8 (Linux permits it in `XDG_RUNTIME_DIR`; `TMPDIR` anywhere)
/// cannot name a socket in a report, so the daemon refuses to start
/// naming the variable, and every JSON surface that would carry the
/// socket — `daemon status`, `daemon stop` — answers C11's total
/// `{"error": …}` with exit 1, never a panic (review 2026-09-11).
#[cfg(unix)]
#[test]
fn c83_a_runtime_directory_that_is_not_utf8_is_refused_by_name_and_never_panics_a_report() {
    use std::os::unix::ffi::OsStrExt;
    let scratch = scratch("utf8");
    let base = &scratch.0;
    std::fs::create_dir_all(base.join("store")).unwrap();
    let mut odd = base.join("tmp").into_os_string();
    odd.push(std::ffi::OsStr::from_bytes(b"-\xff"));
    // The daemon: refused before anything binds.
    let mut cmd = daemon_command(base, "store");
    cmd.env("TMPDIR", &odd).env("XDG_RUNTIME_DIR", &odd);
    let (status, stderr) = refused_daemon(cmd);
    assert!(
        !status.success()
            && stderr.contains("not valid UTF-8")
            && (stderr.contains("XDG_RUNTIME_DIR") || stderr.contains("TMPDIR")),
        "{stderr}"
    );
    // The shells: a total JSON error, exit 1, on both surfaces.
    for verb in [
        &["daemon", "status", "--json"][..],
        &["daemon", "stop", "--json"][..],
    ] {
        let mut cmd = command(base, "store", verb);
        cmd.env("TMPDIR", &odd).env("XDG_RUNTIME_DIR", &odd);
        let out = cmd.output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{verb:?}: {out:?}");
        let text = String::from_utf8_lossy(&out.stderr).into_owned();
        assert!(!text.contains("panicked"), "{verb:?}: {text}");
        let msg = sole_json(&out)["error"].as_str().unwrap().to_string();
        assert!(msg.contains("not valid UTF-8"), "{verb:?}: {msg}");
    }
}
