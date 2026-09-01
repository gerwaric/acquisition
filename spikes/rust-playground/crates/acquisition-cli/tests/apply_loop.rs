//! The tracer's plan→apply→replan loop at process level (step 7): a real
//! daemon over the in-process mock provider, a scripted browserless login,
//! and the CLI as the only interface — the loop must *close*: each apply
//! executes exactly its plan's actions, discovery (substash stubs) waits
//! for the next plan instead of expanding the current one, and after a
//! bootstrap listing plus two reconciliation cycles the plan is empty.
//!
//! Slow-ish on purpose (one daemon, ~17 mock sends, real waits): this is
//! the one test that proves the whole slice through the real binaries.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use acquisition_plan::{RefreshAction, RefreshPlan};
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
        .env("ACQ_NO_KEYRING", "1");
    for var in [
        "ACQ_GGG",
        "ACQ_ACCOUNT",
        "ACQ_TRIPWIRE",
        "ACQ_MAX_SENDS",
        "ACQ_JOURNAL",
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
/// headers, and body. Just enough to click the mock's approve link the way
/// AGENTS.md scripts it with curl.
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

/// The scripted mock login from AGENTS.md, in-process: start
/// `acq auth --no-browser --json`, read the printed authorize URL, GET it
/// with `/authorize?` replaced by `/approve?`, and follow the 302 to the
/// daemon's callback (without the follow the login never completes).
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

/// `refresh --plan --json` as the planner's own validated envelope — what
/// stdout carries is what apply can trust.
fn plan(base: &Path) -> RefreshPlan {
    let out = acq(base, &["refresh", "--plan", "--json"]);
    assert!(out.status.success(), "{out:?}");
    RefreshPlan::from_value(&sole_json(&out)).expect("stdout is a valid plan envelope")
}

/// Apply and return the parent job's successful payload.
fn apply(base: &Path, args: &[&str]) -> Value {
    let out = acq(base, args);
    assert!(out.status.success(), "{out:?}");
    let outcome = sole_json(&out);
    assert_eq!(outcome["outcome"], json!("success"), "{outcome}");
    outcome["payload"].clone()
}

#[test]
fn the_plan_apply_replan_loop_closes_against_the_mock() {
    let base = std::env::temp_dir().join(format!("acq-al-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();

    login(&base);
    let out = acq(
        &base,
        &[
            "policy",
            "set",
            r#"{"version":1,"leagues":{"Standard":{"tabs":"all","max_age_seconds":3600}}}"#,
            "--json",
        ],
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["revision"], json!(1));

    // Cycle 1 — never listed: the plan is the listing alone (no membership
    // authority without a basis), and applying it runs exactly one request.
    let first = plan(&base);
    assert!(
        matches!(first.actions[..], [RefreshAction::ListStashes { .. }]),
        "{:?}",
        first.actions
    );
    let payload = apply(&base, &["refresh", "--apply", "--json"]);
    assert_eq!(payload["requests"], json!(1));
    assert_eq!(payload["children"]["done"], json!(1));

    // Cycle 2 — the listing is on record: every listed tab wants its first
    // fetch (the mock lists 5 top-level tabs plus 2 folder children; the
    // folder itself is skipped, never fetched), and nothing re-lists.
    let second = plan(&base);
    assert_eq!(second.logical_requests, 7, "{:?}", second.actions);
    assert!(
        second
            .actions
            .iter()
            .all(|a| matches!(a, RefreshAction::FetchTab { .. })),
        "{:?}",
        second.actions
    );

    // The admission budget refuses this plan whole, before any child runs:
    // the daemon's error comes back through the CLI as a structured
    // failure, and the next plan still owes all 7 fetches.
    let out = acq(
        &base,
        &["refresh", "--apply", "--max-requests", "1", "--json"],
    );
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    let err = sole_json(&out);
    assert!(
        err["error"]
            .as_str()
            .unwrap()
            .contains("exceeds the budget"),
        "{err}"
    );
    assert_eq!(plan(&base).logical_requests, 7);

    // The reviewed-envelope path: apply the exact plan that was printed.
    let reviewed = base.join("cycle2.json");
    std::fs::write(&reviewed, serde_json::to_vec(&second).unwrap()).unwrap();
    let apply_arg = format!("--apply={}", reviewed.display());
    let payload = apply(&base, &["refresh", &apply_arg, "--json"]);
    assert_eq!(payload["requests"], json!(7));
    assert_eq!(payload["children"]["done"], json!(7));

    // Cycle 3 — fetching the map/unique parents landed their substash
    // stubs, so discovery arrives one plan later (never as an expansion of
    // the plan that discovered it): 7 substashes, fetched under their
    // parents.
    let third = plan(&base);
    assert_eq!(third.logical_requests, 7, "{:?}", third.actions);
    assert!(
        third
            .actions
            .iter()
            .all(|a| matches!(a, RefreshAction::FetchSubstash { .. })),
        "{:?}",
        third.actions
    );
    let payload = apply(&base, &["refresh", "--apply", "--json"]);
    assert_eq!(payload["children"]["done"], json!(7));

    // Cycle 4 — closed: everything covered is fresh, and the no-op apply
    // spends nothing.
    let fourth = plan(&base);
    assert!(fourth.actions.is_empty(), "{:?}", fourth.actions);
    let out = acq(&base, &["refresh", "--apply", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["requests"], json!(0));

    let out = acq(&base, &["daemon", "stop"]);
    assert!(out.status.success(), "{out:?}");
    let _ = std::fs::remove_dir_all(&base);
}
