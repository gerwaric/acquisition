//! Tracer step 8 at process level: the MCP server is the plan slice's
//! second consumer. Against a real daemon over the mock provider (the
//! daemon rides inside `acq-mcp`, so no other binary is involved; login
//! is driven over the protocol the way `acq auth` drives it), the tool
//! surface must carry the whole loop — declare intent, compile the plan,
//! spend it, replan — and every gate must fire through it: the
//! create-only CAS on a blind policy write, the daemon's admission
//! budget, and the step-7 staleness refusal. The offline claims are
//! proven offline: the daemon is stopped before the staleness and
//! empty-plan assertions, and the socket is checked afterwards so a
//! regression that contacted (or spawned) a daemon cannot pass.
//!
//! Loop *closure* (bootstrap listing + two reconciliation cycles) is
//! already pinned by the CLI's `apply_loop.rs`; this test pins the MCP
//! boundary, not the loop again.

mod harness;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::Stdio;
use std::time::{Duration, Instant};

use acquisition_core::client::{Client, ConnectOptions};
use acquisition_core::protocol::{Request, Response};
use acquisition_plan::{RefreshAction, RefreshPlan};
use harness::{Mcp, spawn};
use serde_json::{Value, json};

const POLICY: &str =
    r#"{"version":1,"leagues":{"Standard":{"tabs":"all","max_age_seconds":3600}}}"#;

/// A valid policy whose one id the facts lack (reported as unknown, never
/// fetched): the replan against it is an empty plan, which is how the
/// no-op branch gets exercised with no daemon. (An empty id list is not a
/// policy — it names no work and is refused since policy v3.)
const EMPTY_POLICY: &str =
    r#"{"version":1,"leagues":{"Standard":{"tabs":["no-such-tab"],"max_age_seconds":3600}}}"#;

/// One plain loopback HTTP GET — enough to click the mock's approve link
/// (same shape as `apply_loop.rs`).
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

/// The scripted mock login, over the protocol (no `acq` binary in this
/// crate): `auth_start` returns the authorize URL, the approve variant is
/// clicked, its 302 followed to the daemon callback, and `auth_status`
/// polled until this flow's own completion — `login_ok`, not merely a
/// live session — lands.
async fn login() {
    let mut client = Client::connect(ConnectOptions::autonomous(false))
        .await
        .expect("connecting to the daemon");
    let Response::AuthUrl { authorize_url } = client
        .request(&Request::AuthStart)
        .await
        .expect("auth_start")
    else {
        panic!("auth_start did not return an authorize URL");
    };
    let approve = authorize_url.replace("/authorize?", "/approve?");
    let (status, headers, body) = tokio::task::spawn_blocking(move || http_get(&approve))
        .await
        .unwrap();
    assert_eq!(status, 302, "approve did not redirect: {body}");
    let location = headers["location"].clone();
    let (status, _, body) = tokio::task::spawn_blocking(move || http_get(&location))
        .await
        .unwrap();
    assert_eq!(status, 200, "callback refused: {body}");
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match client
            .request(&Request::AuthStatus)
            .await
            .expect("auth_status")
        {
            Response::Auth {
                login_ok: Some(_), ..
            } => return,
            Response::Auth {
                login_error: Some(e),
                ..
            } => panic!("login failed: {e}"),
            _ if Instant::now() > deadline => panic!("login did not complete"),
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
}

/// Extract and re-validate the envelope a `refresh_plan` call returned:
/// what crosses the MCP boundary must be the planner's own self-validating
/// plan, not a lookalike.
fn validated_plan(result: &Value) -> RefreshPlan {
    RefreshPlan::from_value(&result["plan"]).expect("refresh_plan returned an invalid envelope")
}

/// Poll job_status until the job is terminal, then return job_result's
/// outcome (the MCP idiom: submit returns an id, the agent polls).
fn finish_job(mcp: &mut Mcp, id: &Value) -> Value {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let status = mcp.expect_ok("job_status", json!({ "id": id }));
        match status["state"].as_str() {
            Some("done") => break,
            Some("failed") | Some("cancelled") => panic!("job did not succeed: {status}"),
            _ if Instant::now() > deadline => panic!("job never finished: {status}"),
            _ => std::thread::sleep(Duration::from_millis(100)),
        }
    }
    mcp.expect_ok("job_result", json!({ "id": id }))
}

#[test]
fn the_mcp_tools_carry_the_plan_slice_and_its_gates() {
    let base = std::env::temp_dir().join(format!("acq-m8-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    // The in-process protocol client below reads the same knobs the
    // children inherit.
    // SAFETY: this is the binary's only test; nothing reads the
    // environment concurrently.
    unsafe {
        for var in ["ACQ_GGG", "ACQ_ACCOUNT", "ACQ_NO_SPAWN"] {
            std::env::remove_var(var);
        }
        std::env::set_var("ACQ_SOCKET", base.join("d.sock"));
        std::env::set_var("ACQ_STORE_DIR", base.join("store"));
        std::env::set_var("ACQ_NO_KEYRING", "1");
    }

    // The daemon is spawned directly (`acq-mcp daemon run`) rather than
    // lazily: lazy spawn execs the calling binary, which here would be
    // the test harness.
    let mut daemon = spawn(&base, &["daemon", "run"], &[], Stdio::null);
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Client::connect(ConnectOptions::autonomous(false))
            .await
            .is_err()
        {
            assert!(Instant::now() < deadline, "daemon never bound its socket");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        login().await;
    });

    let mut mcp = Mcp::start(&base, &[]);

    // Intent: no policy yet; the first write creates revision 1; a second
    // blind write is refused naming the revision to review — an agent
    // never replaces intent it has not read. With the reviewed revision
    // named, the CAS admits the replacement.
    let shown = mcp.expect_ok("sync_policy", json!({}));
    assert_eq!(shown["policy"], Value::Null, "{shown}");
    let policy: Value = serde_json::from_str(POLICY).unwrap();
    let row = mcp.expect_ok("set_sync_policy", json!({ "value": policy }));
    assert_eq!(row["revision"], json!(1), "{row}");
    let msg = mcp.expect_err("set_sync_policy", json!({ "value": policy }));
    assert!(msg.contains("revision 1"), "{msg}");

    // Derivation: the never-listed league plans the listing alone, and the
    // returned envelope is the planner's own validated plan — carrying the
    // running daemon's quote (the mock-mode enrichment path).
    let planned = mcp.expect_ok("refresh_plan", json!({}));
    let first = validated_plan(&planned);
    assert!(
        matches!(first.actions[..], [RefreshAction::ListStashes { .. }]),
        "{:?}",
        first.actions
    );
    assert!(
        first.quote.is_some(),
        "a running mock daemon should have quoted: {:?}",
        planned["quote_note"]
    );

    // Effect: apply returns the parent job id (the MCP idiom is
    // submit-then-poll, unlike the CLI's blocking apply), and the parent
    // reports exactly the plan's one request.
    let applied = mcp.expect_ok("apply_plan", json!({ "plan": planned["plan"] }));
    assert_eq!(applied["requests"], json!(1), "{applied}");
    let outcome = finish_job(&mut mcp, &applied["job_id"]);
    assert_eq!(outcome["outcome"], json!("success"), "{outcome}");
    assert_eq!(
        outcome["payload"]["children"]["done"],
        json!(1),
        "{outcome}"
    );

    // Replan: the listing is on record, so every listed tab owes its first
    // fetch (5 top-level + 2 folder children in the mock's league).
    let planned = mcp.expect_ok("refresh_plan", json!({}));
    let second = validated_plan(&planned);
    assert_eq!(second.logical_requests, 7, "{:?}", second.actions);

    // The daemon's admission budget fires through this frontend too:
    // refused whole, before any child job exists.
    let msg = mcp.expect_err(
        "apply_plan",
        json!({ "plan": planned["plan"], "max_requests": 1 }),
    );
    assert!(msg.contains("exceeds the budget"), "{msg}");

    // Everything below is claimed to happen with no daemon contact, so
    // prove it with no daemon: stop it and wait for the socket to die.
    rt.block_on(async {
        let mut client = Client::connect(ConnectOptions::autonomous(false))
            .await
            .expect("daemon should still be up");
        let _ = client.request(&Request::DaemonStop).await;
        let deadline = Instant::now() + Duration::from_secs(10);
        while Client::connect(ConnectOptions::autonomous(false))
            .await
            .is_ok()
        {
            assert!(Instant::now() < deadline, "daemon never stopped");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    // Intent moves to revision 2 — coverage now names only an id the facts
    // lack (reported, never fetched), so the honest
    // replan (compiled with the daemon down; the note says why there is
    // no quote) authorizes nothing.
    let empty_policy: Value = serde_json::from_str(EMPTY_POLICY).unwrap();
    let row = mcp.expect_ok(
        "set_sync_policy",
        json!({ "value": empty_policy, "if_revision": 1 }),
    );
    assert_eq!(row["revision"], json!(2), "{row}");
    let replanned = mcp.expect_ok("refresh_plan", json!({}));
    let third = validated_plan(&replanned);
    assert!(third.actions.is_empty(), "{:?}", third.actions);
    assert!(third.quote.is_none());
    let note = replanned["quote_note"].as_str().unwrap_or_default();
    assert!(note.contains("no quote"), "{replanned}");

    // The empty plan applies as a no-op with no daemon at all…
    let applied = mcp.expect_ok("apply_plan", json!({ "plan": replanned["plan"] }));
    assert_eq!(applied["applied"], json!(false), "{applied}");
    assert_eq!(applied["requests"], json!(0), "{applied}");

    // …and the step-7 staleness gate refuses the revision-1 plan with the
    // remedy named, before any daemon contact.
    let msg = mcp.expect_err("apply_plan", json!({ "plan": planned["plan"] }));
    assert!(
        msg.contains("revision 1") && msg.contains("revision 2") && msg.contains("replan"),
        "{msg}"
    );

    // Neither offline path may have contacted a daemon — which in mock
    // mode would have lazy-spawned one. The socket must still be dead.
    rt.block_on(async {
        assert!(
            Client::connect(ConnectOptions::autonomous(false))
                .await
                .is_err(),
            "an offline path raised a daemon"
        );
    });

    let _ = mcp.child.kill();
    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = std::fs::remove_dir_all(&base);
}
