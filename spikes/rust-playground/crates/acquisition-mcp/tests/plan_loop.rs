//! Tracer step 8 at process level: the MCP server is the plan slice's
//! second consumer. Against a real daemon over the mock provider (the
//! daemon rides inside `acq-mcp`, so no other binary is involved; login
//! is driven over the protocol the way `acq auth` drives it), the tool
//! surface must carry the whole loop — declare intent, compile the plan,
//! spend it, replan — and every gate must fire through it: the
//! create-only CAS on a blind policy write, the daemon's admission
//! budget, and the step-7 staleness refusal before any daemon contact.
//!
//! Loop *closure* (bootstrap listing + two reconciliation cycles) is
//! already pinned by the CLI's `apply_loop.rs`; this test pins the MCP
//! boundary, not the loop again.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Lines, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use acquisition_core::client::{Client, ConnectOptions};
use acquisition_core::protocol::{Request, Response};
use acquisition_plan::{RefreshAction, RefreshPlan};
use serde_json::{Value, json};

const POLICY: &str =
    r#"{"version":1,"leagues":{"Standard":{"tabs":"all","max_age_seconds":3600}}}"#;

fn spawn(base: &Path, args: &[&str], stdio: fn() -> Stdio) -> Child {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq-mcp"));
    cmd.args(args)
        // Short socket path (Unix sockets cap ~104 bytes).
        .env("ACQ_SOCKET", base.join("d.sock"))
        .env("ACQ_STORE_DIR", base.join("store"))
        .env("ACQ_NO_KEYRING", "1")
        .stdin(stdio())
        .stdout(stdio())
        .stderr(Stdio::null());
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
    cmd.spawn().expect("spawning acq-mcp")
}

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

/// A newline-delimited JSON-RPC conversation with the MCP server's stdio.
struct Mcp {
    child: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
    next_id: i64,
}

impl Mcp {
    fn start(base: &Path) -> Mcp {
        let mut child = spawn(base, &[], Stdio::piped);
        let stdin = child.stdin.take().unwrap();
        let lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut mcp = Mcp {
            child,
            stdin,
            lines,
            next_id: 0,
        };
        let init = mcp.rpc(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "plan-loop-test", "version": "0" }
            }),
        );
        assert!(init.get("result").is_some(), "initialize failed: {init}");
        mcp.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        mcp
    }

    fn send(&mut self, msg: &Value) {
        let mut line = msg.to_string();
        line.push('\n');
        self.stdin.write_all(line.as_bytes()).unwrap();
        self.stdin.flush().unwrap();
    }

    fn rpc(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let line = self
                .lines
                .next()
                .expect("MCP server closed stdout")
                .expect("reading MCP stdout");
            let msg: Value = serde_json::from_str(&line)
                .unwrap_or_else(|e| panic!("not JSON-RPC ({e}): {line}"));
            if msg["id"] == json!(id) {
                return msg;
            }
        }
    }

    /// Call one tool: `Ok(structured result)` or `Err(message)` whether
    /// the failure came back as a JSON-RPC error or an isError result.
    fn call(&mut self, tool: &str, args: Value) -> Result<Value, String> {
        let resp = self.rpc("tools/call", json!({ "name": tool, "arguments": args }));
        if let Some(error) = resp.get("error") {
            return Err(error["message"].as_str().unwrap_or_default().to_string());
        }
        let result = &resp["result"];
        let text = || {
            result["content"][0]["text"]
                .as_str()
                .unwrap_or_default()
                .to_string()
        };
        if result["isError"] == json!(true) {
            return Err(text());
        }
        match result.get("structuredContent") {
            Some(v) => Ok(v.clone()),
            None => Ok(serde_json::from_str(&text())
                .unwrap_or_else(|e| panic!("unstructured tool result ({e}): {resp}"))),
        }
    }

    fn expect_ok(&mut self, tool: &str, args: Value) -> Value {
        self.call(tool, args.clone())
            .unwrap_or_else(|e| panic!("{tool} {args} failed: {e}"))
    }

    fn expect_err(&mut self, tool: &str, args: Value) -> String {
        match self.call(tool, args) {
            Err(e) => e,
            Ok(v) => panic!("{tool} unexpectedly succeeded: {v}"),
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
    let mut daemon = spawn(&base, &["daemon", "run"], Stdio::null);
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

    let mut mcp = Mcp::start(&base);

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

    // The step-7 staleness gate, offline: intent moved to revision 2, so
    // the revision-1 plan is refused with the remedy named — before any
    // daemon contact.
    let row = mcp.expect_ok(
        "set_sync_policy",
        json!({ "value": policy, "if_revision": 1 }),
    );
    assert_eq!(row["revision"], json!(2), "{row}");
    let msg = mcp.expect_err("apply_plan", json!({ "plan": planned["plan"] }));
    assert!(
        msg.contains("revision 1") && msg.contains("revision 2") && msg.contains("replan"),
        "{msg}"
    );

    let _ = mcp.child.kill();
    rt.block_on(async {
        if let Ok(mut client) = Client::connect(ConnectOptions::autonomous(false)).await {
            let _ = client.request(&Request::DaemonStop).await;
        }
    });
    let _ = daemon.kill();
    let _ = daemon.wait();
    let _ = std::fs::remove_dir_all(&base);
}
