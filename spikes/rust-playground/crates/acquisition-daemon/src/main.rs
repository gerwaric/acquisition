//! `acqd` — the Acquisition daemon as its own executable (C1, C82). A
//! frontend spawns the `acqd` beside it (`acquisition-client`'s locator)
//! and the live drivers start it directly; nothing else does. It takes no
//! arguments: its knobs are the environment (`README.md`, "Knobs" —
//! `ACQ_GGG`, `ACQ_SOCKET`, `ACQ_STORE_DIR`, the rails), the one door to
//! them, so that a flag can never disagree with the environment the
//! frontend that spawned it read (the packet's rejected "flags on
//! `acqd`"). It runs in the foreground until stopped or idle
//! (`daemon::run`); a refusal to start is written to the daemon log, the
//! only place a lazily spawned daemon's stderr can reach anyone.

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
