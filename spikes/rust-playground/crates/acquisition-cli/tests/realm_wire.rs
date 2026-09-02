//! Realm at the wire boundary, through the real binaries and the send
//! journal (the GGG-side contract surface, TESTING-NOTES.md): a real daemon
//! over the in-process mock, a scripted browserless login, and the CLI as
//! the only interface. The unit tests pin `route_for`; this pins what the
//! daemon actually sends and files (review request, 2026-09-02):
//!
//! - an explicit `pc` and an omitted realm are the same route and the
//!   same result — every pc send is byte-identical to the pre-realm ones;
//! - xbox work goes out on its own routes (`stash-list/xbox`, `stash/xbox`),
//!   each probed before its first counted send;
//! - the mock's empty console listing lands nothing under xbox while pc's
//!   tabs stay pc's — the store keeps realms apart;
//! - a realm the stash family does not take (poe2) is refused at admission,
//!   with nothing journaled.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

fn acq(base: &Path, args: &[&str]) -> Output {
    command(base, args).output().expect("spawning acq")
}

fn command(base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        // The socket path must stay short (Unix sockets cap ~104 bytes),
        // so everything lives directly under the platform temp dir.
        .env("ACQ_SOCKET", base.join("d.sock"))
        .env("ACQ_STORE_DIR", base.join("store"))
        .env("ACQ_NO_KEYRING", "1")
        // The daemon reads the journal path at start; every command
        // carries it so whichever one spawns the daemon sets it.
        .env("ACQ_JOURNAL", base.join("sends.jsonl"));
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

fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

/// One plain HTTP GET (loopback only), returning status, lowercased
/// headers, and body — enough to click the mock's approve link.
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

/// The scripted mock login from AGENTS.md, in-process.
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

/// A job command's successful payload.
fn payload(base: &Path, args: &[&str]) -> Value {
    let out = acq(base, args);
    assert!(out.status.success(), "{out:?}");
    let outcome = sole_json(&out);
    assert_eq!(outcome["outcome"], json!("success"), "{outcome}");
    outcome["payload"].clone()
}

/// The journal's sends as `(method, route, status)`, in order.
fn sends(base: &Path) -> Vec<(String, String, Option<u64>)> {
    let text = std::fs::read_to_string(base.join("sends.jsonl")).expect("journal");
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<Value>(l).expect("journal line"))
        .filter(|v| v.get("route").is_some())
        .map(|v| {
            (
                v["method"].as_str().unwrap().to_string(),
                v["route"].as_str().unwrap().to_string(),
                v["status"].as_u64(),
            )
        })
        .collect()
}

#[test]
fn realm_is_on_the_wire_exactly_as_ruled() {
    let base = std::env::temp_dir().join(format!("acq-rw-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    login(&base);

    // Explicit pc and omitted realm: the same route, the same listing.
    let omitted = payload(
        &base,
        &[
            "submit",
            "stashes",
            "--params",
            r#"{"league":"Standard"}"#,
            "--json",
        ],
    );
    let explicit = payload(
        &base,
        &[
            "submit",
            "stashes",
            "--params",
            r#"{"realm":"pc","league":"Standard"}"#,
            "--json",
        ],
    );
    assert_eq!(omitted["stashes"], explicit["stashes"]);
    assert!(!omitted["stashes"].as_array().unwrap().is_empty());

    // xbox: its own route, the mock's empty console listing, and a tab
    // fetch the mock answers 404 (the job fails honestly; the send is
    // journaled on the xbox route).
    let xbox = payload(&base, &["stashes", "--realm", "xbox", "--json"]);
    assert_eq!(xbox["stashes"], json!([]));
    let out = acq(&base, &["stash", "cur1", "--realm", "xbox", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");

    // poe2: the stash family does not take it — refused at admission,
    // before a job id exists.
    let out = acq(&base, &["stashes", "--realm", "poe2", "--json"]);
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let err = sole_json(&out);
    assert!(
        err["error"]
            .as_str()
            .unwrap()
            .contains("stashes endpoints do not take realm poe2"),
        "{err}"
    );

    // The store keeps realms apart: pc's tabs are on record under pc only.
    let pc_tabs = sole_json(&acq(&base, &["tabs", "--json"]));
    assert!(!pc_tabs.as_array().unwrap().is_empty(), "{pc_tabs}");
    let xbox_tabs = sole_json(&acq(&base, &["tabs", "--realm", "xbox", "--json"]));
    assert_eq!(xbox_tabs, json!([]));

    // The journal: pc sends on the bare routes (two GETs on one route —
    // explicit and omitted are indistinguishable), xbox sends on the
    // suffixed ones with a HEAD probe before each route's first GET,
    // and no poe2 send at all.
    let all = sends(&base);
    let account = all
        .iter()
        .find_map(|(_, r, _)| r.strip_prefix("stash-list@").map(str::to_string))
        .expect("a pc listing was journaled");
    let on = |route: &str| -> Vec<(String, Option<u64>)> {
        all.iter()
            .filter(|(_, r, _)| r == route)
            .map(|(m, _, st)| (m.clone(), *st))
            .collect()
    };
    assert_eq!(
        on(&format!("stash-list@{account}")),
        vec![
            ("HEAD".to_string(), Some(204)),
            ("GET".to_string(), Some(200)),
            ("GET".to_string(), Some(200)),
        ]
    );
    assert_eq!(
        on(&format!("stash-list/xbox@{account}")),
        vec![
            ("HEAD".to_string(), Some(204)),
            ("GET".to_string(), Some(200))
        ]
    );
    assert_eq!(
        on(&format!("stash/xbox@{account}")),
        vec![
            ("HEAD".to_string(), Some(204)),
            ("GET".to_string(), Some(404))
        ]
    );
    assert!(
        all.iter().all(|(_, r, _)| !r.contains("poe2")),
        "a poe2 send was journaled: {all:?}"
    );

    let out = acq(&base, &["daemon", "stop"]);
    assert!(out.status.success(), "{out:?}");
    let _ = std::fs::remove_dir_all(&base);
}
