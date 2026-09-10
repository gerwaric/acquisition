//! The client side of the daemon protocol (C1, C12's first surface as a
//! frontend reaches it): the IPC over tokio and the three policy doors of
//! C10 ([`client`]), the `acqd` locator ([`locator`], C82), the artifact
//! comparison against that sibling ([`artifact`], C84), the bounded
//! frame reader ([`frame`]) and the typed failures at the door
//! ([`client::ConnectError`]). Every frontend links this crate and the
//! protocol crate; this crate links the protocol crate and tokio, never
//! the daemon (`acquisition-daemon`) or the planner, and
//! `tools/docs-check.sh` refuses both edges. The store joins at the
//! split's step 5, when the world (root, locks, socket name) becomes the
//! store's and the client verifies it.
//!
//! Extracted from `acquisition-core` as commit 3 of the daemon split
//! (`DAEMON-SPLIT-SLICE.md`; the design is
//! `brainstorming-notes/18-the-daemon-split.md` §2.1, §2.3): a client or
//! locator edit must not recompile the daemon, and the GUI and the MCP
//! server need typed failures rather than strings.
//!
//! Until step 5 the rendezvous is a convention both sides read from the
//! environment — [`socket_path`] here, the same function in the daemon's
//! `daemon.rs` — and the frontends reach the socket and log paths through
//! this crate; step 5 makes them the world's.

pub mod artifact;
pub mod client;
pub mod frame;
pub mod locator;

use std::path::PathBuf;

/// The socket a daemon listens on and a client connects to: `ACQ_SOCKET`,
/// or `acquisition-playground.sock` in the temp directory. The daemon
/// reads the same knob the same way (`daemon.rs`, the one place that
/// binds it); the two copies agree by construction of the convention,
/// and step 5 replaces both with the world's derived socket. Must
/// stay short: Unix socket paths cap out around 104 bytes (`SUN_LEN`).
pub fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("ACQ_SOCKET") {
        return PathBuf::from(p);
    }
    std::env::temp_dir().join("acquisition-playground.sock")
}

/// The daemon's log, beside its socket: where a lazily spawned daemon's
/// startup refusal lands (its stderr goes to null), read back by
/// [`client::Client::connect`] when a spawn fails, and printed by
/// `acq daemon status`.
pub fn log_path() -> PathBuf {
    socket_path().with_extension("log")
}
