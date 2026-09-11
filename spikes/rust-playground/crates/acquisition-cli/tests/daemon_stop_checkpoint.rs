//! Plan step 7, item 4 (`PRICING-SLICE.md`): the daemon leaves by
//! `process::exit`, so its SQLite connections are never closed and the
//! write-ahead logs stayed uncheckpointed beside the facts file and
//! `daemon.db` (the price-notes run, 2026-09-04: `tools/notes-check.py`
//! had to read through the WAL, `tools/census.py` refuses one). Pinned
//! here through the real binaries over the in-process mock: after a
//! fetch has landed facts, `acq daemon stop` leaves no `-wal` pages
//! behind either file. The pre-stop assertion keeps the check honest —
//! the WAL holds pages before the stop, so an empty one after it is the
//! checkpoint's doing.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

fn command(base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_STORE_DIR", base.join("store"))
        .env("TMPDIR", scratch_tmp(base))
        .env("XDG_RUNTIME_DIR", scratch_tmp(base))
        .env("ACQ_LOG_DIR", base.join("logs"))
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_JOURNAL", "0");
    for var in [
        "ACQ_GGG",
        "ACQ_ACCOUNT",
        "ACQ_TRIPWIRE",
        "ACQ_MAX_SENDS",
        "ACQ_IDLE_SHUTDOWN",
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

/// One plain HTTP GET (loopback only): status, lowercased headers, body.
fn http_get(url: &str) -> (u16, HashMap<String, String>, String) {
    let rest = url.strip_prefix("http://").expect("loopback http url");
    let (host, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let path = if path.is_empty() { "/" } else { path };
    let mut stream = TcpStream::connect(host).expect("connecting to mock/daemon");
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (head, body) = response
        .split_once("\r\n\r\n")
        .unwrap_or((response.as_str(), ""));
    let mut lines = head.lines();
    let status: u16 = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("unparseable response:\n{response}"));
    let headers = lines
        .filter_map(|l| l.split_once(": "))
        .map(|(k, v)| (k.to_ascii_lowercase(), v.to_string()))
        .collect();
    (status, headers, body.to_string())
}

/// The scripted mock login (the mock-session skill), in-process.
fn login(base: &Path) {
    let mut child = command(base, &["auth", "--no-browser", "--json"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawning acq auth");
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let first: Value = serde_json::from_str(&lines.next().unwrap().unwrap()).unwrap();
    let authorize = first["authorize_url"].as_str().expect("authorize_url");
    let approve = authorize.replace("/authorize?", "/approve?");
    let (status, headers, body) = http_get(&approve);
    assert_eq!(status, 302, "approve did not redirect: {body}");
    let (status, _, body) = http_get(&headers["location"]);
    assert_eq!(status, 200, "callback refused: {body}");
    assert!(child.wait().unwrap().success(), "login did not complete");
}

/// The `-wal` beside a database file: its size, or `None` when absent.
fn wal_len(db: &Path) -> Option<u64> {
    let mut wal = db.as_os_str().to_owned();
    wal.push("-wal");
    std::fs::metadata(wal).ok().map(|m| m.len())
}

#[test]
fn daemon_stop_checkpoints_the_facts_file_and_the_queue() {
    let base = std::env::temp_dir().join(format!("acq-ckpt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    login(&base);

    // A listing lands facts in the account's file through the daemon.
    let out = acq(&base, &["stashes", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let listing = sole_json(&out);
    assert!(
        !listing["payload"]["stashes"]
            .as_array()
            .unwrap_or_else(|| panic!("{listing}"))
            .is_empty(),
        "the mock lists tabs"
    );
    let mock = base.join("store").join("mock");
    let facts: PathBuf = std::fs::read_dir(&mock)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| {
            p.extension().is_some_and(|x| x == "db")
                && p.file_name().is_some_and(|n| n != "daemon.db")
                && !p.to_string_lossy().ends_with(".annotations.db")
        })
        .expect("the account's facts file");
    let queue = mock.join("daemon.db");
    // The daemon holds both open; their WALs carry what it wrote.
    assert!(
        wal_len(&facts).is_some_and(|n| n > 0),
        "before the stop the facts WAL holds pages: {:?}",
        wal_len(&facts)
    );
    assert!(
        wal_len(&queue).is_some_and(|n| n > 0),
        "before the stop the queue WAL holds pages: {:?}",
        wal_len(&queue)
    );

    let out = acq(&base, &["daemon", "stop"]);
    assert!(out.status.success(), "{out:?}");
    // The daemon checkpoints, removes its socket, then exits: wait until
    // nothing answers on this world's socket any more.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = sole_json(&acq(&base, &["daemon", "status", "--json"]));
        if status["running"] == Value::Bool(false) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the daemon did not exit: {status}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        wal_len(&facts).unwrap_or(0),
        0,
        "the facts WAL after the stop: {:?} bytes",
        wal_len(&facts)
    );
    assert_eq!(
        wal_len(&queue).unwrap_or(0),
        0,
        "the queue WAL after the stop: {:?} bytes",
        wal_len(&queue)
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// The scratch temp and runtime directory of one test, under `base`:
/// `TMPDIR` (macOS's runtime fallback) and
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
