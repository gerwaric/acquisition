//! The agent-traffic deferral at the MCP boundary (CONTEXT.md,
//! "Explicitly deferred"), pinned in the mode it protects: an `acq-mcp`
//! started with `ACQ_GGG=1` refuses the spending tools immediately.
//! Nothing else exists here — no daemon, no store, no login — so the
//! refusal must be the *first* gate: a regression that consulted the
//! account index or contacted a daemon before refusing would answer with
//! that failure instead of the deferral, and the assertions would catch
//! the swap.

mod harness;

use harness::Mcp;
use serde_json::json;

#[test]
fn ggg_mode_refuses_the_spending_tools_before_anything_else() {
    let base = std::env::temp_dir().join(format!("acq-m8g-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();

    let mut mcp = Mcp::start(&base, &[("ACQ_GGG", "1")]);

    // apply_plan: refused on the mode alone — the deliberately absurd
    // "plan" proves not even the envelope parse ran first.
    let msg = mcp.expect_err("apply_plan", json!({ "plan": "not even an object" }));
    assert!(msg.contains("deferred") && msg.contains("ACQ_GGG"), "{msg}");

    // submit_job: the same deferral, same wording.
    let msg = mcp.expect_err("submit_job", json!({ "kind": "stashes" }));
    assert!(msg.contains("deferred") && msg.contains("ACQ_GGG"), "{msg}");

    // No daemon was contacted or spawned for either refusal: the scratch
    // socket was never even created.
    assert!(
        !base.join("d.sock").exists(),
        "a ggg-mode refusal touched the daemon socket"
    );

    let _ = mcp.child.kill();
    let _ = std::fs::remove_dir_all(&base);
}
