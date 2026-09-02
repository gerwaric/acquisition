//! Characters in the refresh plan at the wire boundary, through the real
//! binaries and the send journal (the GGG-side contract surface,
//! TESTING-NOTES.md): a real daemon over the in-process mock, a scripted
//! browserless login, and the CLI as the only interface. The planner's
//! unit tests pin the plan; this pins what applying a character-only
//! PoE2 policy actually sends and files (CONTEXT.md, "Characters in the
//! refresh plan", 2026-09-02):
//!
//! - a poe2 entry covering characters only is the ordinary v3 shape, and
//!   its plan is the realm's character listing alone, then the fetch;
//! - the sends go out on the realm-suffixed character routes
//!   (`character-list/poe2`, `character/poe2`), each probed before its
//!   first counted send, and nothing goes out on a stash route;
//! - the PoE2 body's `skills` array is lifted, so the skill item is on
//!   record at the character with container `skills`;
//! - the loop closes: the next plan is empty and applies with no daemon.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use acquisition_plan::{FetchReason, ListingReason, RefreshAction, RefreshPlan};
use serde_json::{Value, json};

fn acq(base: &Path, args: &[&str]) -> Output {
    command(base, args).output().expect("spawning acq")
}

fn command(base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_SOCKET", base.join("d.sock"))
        .env("ACQ_STORE_DIR", base.join("store"))
        .env("ACQ_NO_KEYRING", "1")
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

/// One plain HTTP GET (loopback only) — enough to click the mock's approve
/// link.
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

/// `refresh --plan --realm poe2 --json` as the planner's own validated
/// envelope.
fn plan(base: &Path) -> RefreshPlan {
    let out = acq(base, &["refresh", "--plan", "--realm", "poe2", "--json"]);
    assert!(out.status.success(), "{out:?}");
    RefreshPlan::from_value(&sole_json(&out)).expect("stdout is a valid plan envelope")
}

/// Apply the poe2 plan and return the parent job's successful payload.
fn apply(base: &Path) -> Value {
    let out = acq(base, &["refresh", "--apply", "--realm", "poe2", "--json"]);
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
fn a_character_only_poe2_policy_closes_its_loop_on_the_poe2_routes() {
    let base = std::env::temp_dir().join(format!("acq-cw-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    login(&base);

    // A character-only PoE2 entry is the ordinary v3 shape; a tabs facet
    // under poe2 is refused at the write, nothing stored.
    let out = acq(
        &base,
        &[
            "policy",
            "set",
            r#"{"version":3,"realms":{"poe2":{"leagues":{"Standard":{"tabs":"all","max_age_seconds":3600}}}}}"#,
            "--json",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(
        sole_json(&out)["error"].as_str().unwrap().contains("poe2"),
        "{out:?}"
    );
    let out = acq(
        &base,
        &[
            "policy",
            "set",
            r#"{"version":3,"realms":{"poe2":{"leagues":{"Standard":{"characters":"all","max_age_seconds":3600}}}}}"#,
            "--json",
        ],
    );
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["revision"], json!(1));

    // Cycle 1 — the realm's characters were never listed: the listing
    // alone, realm-wide (no league on the action), no stash work at all.
    let first = plan(&base);
    assert!(
        matches!(
            first.actions[..],
            [RefreshAction::ListCharacters {
                reason: ListingReason::NeverListed,
                ..
            }]
        ),
        "{:?}",
        first.actions
    );
    assert!(first.basis.stash_listing.is_none() && first.basis.character_listing.is_none());
    let payload = apply(&base);
    assert_eq!(payload["children"]["done"], json!(1));

    // Cycle 2 — the mock's one PoE2 character owes its first fetch, by
    // the listed name.
    let second = plan(&base);
    assert!(
        matches!(
            &second.actions[..],
            [RefreshAction::FetchCharacter { id, name, reason: FetchReason::NeverFetched, .. }]
                if id == "fake2001" && name == "SecondExile"
        ),
        "{:?}",
        second.actions
    );
    assert!(second.basis.character_listing.is_some());
    let payload = apply(&base);
    assert_eq!(payload["children"]["done"], json!(1));

    // The PoE2 body's `skills` array was lifted: the skill is an item at
    // the character (located by id, container `skills`), under poe2 only.
    let out = acq(
        &base,
        &[
            "items",
            "search",
            "Falling Thunder",
            "--realm",
            "poe2",
            "--json",
        ],
    );
    assert!(out.status.success(), "{out:?}");
    let items = sole_json(&out);
    let items = items.as_array().unwrap();
    assert_eq!(items.len(), 1, "{items:?}");
    assert_eq!(items[0]["location_kind"], "character");
    assert_eq!(items[0]["location_id"], "fake2001");
    assert_eq!(items[0]["container"], "skills");
    assert_eq!(items[0]["realm"], "poe2");
    let out = acq(
        &base,
        &[
            "items",
            "search",
            "Falling Thunder",
            "--realm",
            "pc",
            "--json",
        ],
    );
    assert_eq!(sole_json(&out), json!([]));
    let out = acq(&base, &["store", "characters", "--realm", "poe2", "--json"]);
    let rows = sole_json(&out);
    assert_eq!(rows.as_array().unwrap().len(), 1, "{rows}");
    assert!(rows[0]["fetched_at"].is_i64() && rows[0]["item_count"].as_i64().unwrap() > 0);

    // Cycle 3 — closed: the character is fresh, the plan is empty, and the
    // no-op apply spends nothing.
    let third = plan(&base);
    assert!(third.actions.is_empty(), "{:?}", third.actions);
    assert_eq!(
        third.skipped_characters.len(),
        1,
        "{:?}",
        third.skipped_characters
    );
    let out = acq(&base, &["refresh", "--apply", "--realm", "poe2", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["requests"], json!(0));

    // The journal: exactly the poe2 character routes, each probed first,
    // every send 2xx, and nothing on a stash route or a pc route.
    let all = sends(&base);
    let account = all
        .iter()
        .find_map(|(_, r, _)| r.strip_prefix("character-list/poe2@").map(str::to_string))
        .expect("a poe2 character listing was journaled");
    let on = |route: &str| -> Vec<(String, Option<u64>)> {
        all.iter()
            .filter(|(_, r, _)| r == route)
            .map(|(m, _, st)| (m.clone(), *st))
            .collect()
    };
    assert_eq!(
        on(&format!("character-list/poe2@{account}")),
        vec![
            ("HEAD".to_string(), Some(204)),
            ("GET".to_string(), Some(200))
        ]
    );
    assert_eq!(
        on(&format!("character/poe2@{account}")),
        vec![
            ("HEAD".to_string(), Some(204)),
            ("GET".to_string(), Some(200))
        ]
    );
    let data_routes: Vec<&String> = all
        .iter()
        .map(|(_, r, _)| r)
        .filter(|r| !r.starts_with("oauth-token") && !r.starts_with("profile"))
        .collect();
    assert!(
        data_routes.iter().all(|r| r.contains("/poe2@")),
        "a send left the poe2 character routes: {all:?}"
    );
    assert!(
        all.iter().all(|(_, _, st)| matches!(st, Some(200..=299))),
        "a non-2xx send: {all:?}"
    );

    let out = acq(&base, &["daemon", "stop"]);
    assert!(out.status.success(), "{out:?}");
    let _ = std::fs::remove_dir_all(&base);
}
