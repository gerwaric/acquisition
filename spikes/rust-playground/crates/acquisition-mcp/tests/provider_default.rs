//! C88 at the MCP boundary: `acq-mcp` refuses the retired `ACQ_GGG` and
//! an unknown provider word at start, on stderr, before it serves —
//! the host shows the line. The real-mode spawn rule itself is the
//! client's and is pinned in the CLI's `provider_default.rs`; this
//! server never spawns in real mode regardless (C13, `ggg_refusal.rs`).

use std::process::{Command, Stdio};

#[test]
fn c88_the_server_refuses_the_retired_knob_and_an_unknown_provider_at_start() {
    for (env, said) in [
        (("ACQ_GGG", "1"), "ACQ_GGG is no longer read"),
        (("ACQ_PROVIDER", "real"), "names no provider"),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_acq-mcp"))
            .env(env.0, env.1)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{env:?}: {out:?}");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.starts_with("acq-mcp: ") && stderr.contains(said) && stderr.contains("C88"),
            "{env:?}: {stderr}"
        );
    }
}
