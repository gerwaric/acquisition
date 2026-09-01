//! Shared process-test harness: spawn `acq-mcp` (as MCP server or as
//! `daemon run`) in an isolated environment and speak newline-delimited
//! JSON-RPC to the server's stdio.
#![allow(dead_code)] // each test binary uses the slice it needs

use std::io::{BufRead, BufReader, Lines, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

/// Spawn the `acq-mcp` binary with the scratch socket/store under `base`,
/// the live-run knobs scrubbed, and `extra_env` applied last (so a test
/// can opt into e.g. `ACQ_GGG=1` deliberately).
pub fn spawn(
    base: &Path,
    args: &[&str],
    extra_env: &[(&str, &str)],
    stdio: fn() -> Stdio,
) -> Child {
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
    for (key, value) in extra_env {
        cmd.env(key, value);
    }
    cmd.spawn().expect("spawning acq-mcp")
}

/// A newline-delimited JSON-RPC conversation with the MCP server's stdio.
pub struct Mcp {
    pub child: Child,
    stdin: ChildStdin,
    lines: Lines<BufReader<ChildStdout>>,
    next_id: i64,
}

impl Mcp {
    pub fn start(base: &Path, extra_env: &[(&str, &str)]) -> Mcp {
        let mut child = spawn(base, &[], extra_env, Stdio::piped);
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
                "clientInfo": { "name": "acq-mcp-test", "version": "0" }
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
    pub fn call(&mut self, tool: &str, args: Value) -> Result<Value, String> {
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

    pub fn expect_ok(&mut self, tool: &str, args: Value) -> Value {
        self.call(tool, args.clone())
            .unwrap_or_else(|e| panic!("{tool} {args} failed: {e}"))
    }

    pub fn expect_err(&mut self, tool: &str, args: Value) -> String {
        match self.call(tool, args) {
            Err(e) => e,
            Ok(v) => panic!("{tool} unexpectedly succeeded: {v}"),
        }
    }
}
