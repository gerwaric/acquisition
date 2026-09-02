//! The one real-GGG-mode rule left at the MCP boundary after the agent
//! traffic ruling (CONTEXT.md, 2026-09-01: agents may use the gate; the
//! daemon enforces GGG's rules): `acq-mcp` **never spawns a daemon in ggg
//! mode**. A real-mode daemon is a human's act via the CLI (keychain,
//! browser). Pinned in the mode it protects, with nothing else present —
//! no daemon, no store, no login — so the spending tools must fail by
//! reporting the absent daemon, and the scratch socket must never come
//! into existence (a regression that lazy-spawned here would create it).

mod harness;

use harness::Mcp;
use serde_json::json;

#[test]
fn ggg_mode_never_spawns_a_daemon_for_the_spending_tools() {
    let base = std::env::temp_dir().join(format!("acq-m8g-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();

    let mut mcp = Mcp::start(&base, &[("ACQ_GGG", "1")]);

    // submit_job: no daemon is running and none may be started here, so
    // the call fails on the absent daemon — not on the mode.
    let msg = mcp.expect_err("submit_job", json!({ "kind": "stashes" }));
    assert!(
        !msg.contains("deferred"),
        "the lifted deferral is still being cited: {msg}"
    );
    assert!(
        !base.join("d.sock").exists(),
        "submit_job in ggg mode spawned a daemon (socket appeared): {msg}"
    );

    // apply_plan: the offline gates run first (no account is known here),
    // and whatever refuses it, no daemon appears.
    let msg = mcp.expect_err("apply_plan", json!({ "plan": "not even an object" }));
    assert!(!msg.contains("deferred"), "{msg}");
    assert!(
        !base.join("d.sock").exists(),
        "apply_plan in ggg mode spawned a daemon (socket appeared): {msg}"
    );

    let _ = mcp.child.kill();
    let _ = std::fs::remove_dir_all(&base);
}
