//! `acqd` — the Acquisition daemon as its own executable (C1, C82). A
//! frontend spawns the `acqd` beside it (`acquisition-client`'s locator)
//! and the live drivers start it directly; nothing else does. It takes no
//! arguments: its knobs are the environment (`README.md`, "Knobs" —
//! `ACQ_GGG`, `ACQ_STORE_DIR`, `ACQ_LOG_DIR`, the rails), the one door to
//! them, so that a flag can never disagree with the environment the
//! frontend that spawned it read (the packet's rejected "flags on
//! `acqd`"); the socket is derived from the world, never named (C83).
//! Nor a `--version`: the daemon's identity is two values
//! (C84) that nothing needs it to print — the shared-contract revision
//! is the same constant `acq --version` prints, and the artifact is a
//! property of this file, which a driver hashes with `shasum` and a
//! client learns from `hello`; a self-report would be a second door to
//! the first and could not vouch for the second. It runs in the
//! foreground until stopped or idle (`daemon::run`); a refusal to start
//! is written to the daemon log, the only place a lazily spawned
//! daemon's stderr can reach anyone.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        eprintln!(
            "acqd: the Acquisition daemon takes no arguments (got {:?}); it is started by the \
             frontend beside it (acq, acq-mcp) or by a driver, and configured by the environment \
             (README.md, Knobs)",
            args
        );
        std::process::exit(2);
    }
    acquisition_daemon::daemon::run().await
}
