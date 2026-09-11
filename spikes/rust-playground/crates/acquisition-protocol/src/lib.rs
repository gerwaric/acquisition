//! The daemon's contract as a frontend sees it (C12's first surface), with
//! nothing in it that can change for a client-side reason: the wire
//! ([`protocol`]: `Request`/`Response`, the bootstrap plane, `ErrorKind`,
//! `Quote`, the frame bound), the job model ([`job`]), the job vocabulary
//! ([`realm`]), the status documents the daemon reports ([`status`]), the
//! provider names and which provider this process wants ([`provider`]),
//! the daemon artifact's identity as `hello` carries it ([`artifact`]),
//! and the shared-contract revision the handshake compares
//! ([`CONTRACT_REVISION`], computed by `build.rs`; C84). Both sides of the
//! socket link this crate; the IPC that carries the frames does not live
//! here — the client and the daemon each own theirs — so this crate links
//! serde and serde_json and nothing else, and `tools/docs-check.sh`
//! refuses more.
//!
//! Extracted from `acquisition-core` (now `acquisition-daemon`) as commit 1
//! of the daemon split (`DAEMON-SPLIT-SLICE.md`, step ledger; the design
//! is `brainstorming-notes/18-the-daemon-split.md` §2.1). Since step 4 the
//! revision's inputs are the contract alone — this crate and the store —
//! so a daemon-only edit moves the daemon artifact and nothing here.

// The lint ratchet (CONTEXT.md, "Panics are for broken internal invariants
// only", C47; ruled for this crate 2026-09-11): the wire crate's production
// code panics on nothing external — a malformed frame or document is a
// structured error. Tests may unwrap; `build.rs` is a separate target the
// attribute does not reach, and panics on purpose.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

pub mod artifact;
pub mod job;
pub mod protocol;
pub mod provider;
pub mod realm;
pub mod status;

/// The package version. Not the handshake identity on its own: it is
/// fixed at `0.0.1` across every commit of the playground, so comparing it
/// lets a daemon from an older build serve a newer client silently (review
/// finding 2026-09-02: a pre-realm daemon accepted a console job and sent
/// it to pc). [`CONTRACT_REVISION`] is what the handshake compares, with
/// the daemon artifact beside it (C84).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The shared-contract revision (C84): twelve hex digits of a digest over
/// the two surfaces every frontend shares — the protocol and store
/// crates' sources and manifests, the root manifest and the lock
/// (`build.rs` lists the inputs and the format). It changes whenever any
/// of those whole files changes — including lock entries and store code
/// the daemon does not use, a deliberate false mismatch — never on a
/// source edit to the daemon, the planner, the client crate or a frontend
/// (a dependency change there moves the lock, and the lock is an input on
/// purpose), and never by consulting git. A daemon-only edit moves the
/// daemon artifact instead (`artifact`). Written into the send journal
/// header and the daemon's startup line beside the artifact hash: the
/// rails verify behavior, these are the two values that say *which code*
/// behaved; a run record maps them to HEAD.
pub const CONTRACT_REVISION: &str = env!("ACQ_CONTRACT_REVISION");

/// `<version> (contract <revision>)`: what `acq --version` prints and
/// what the versioned status documents carry as `version`. The handshake
/// compares the parts — [`CONTRACT_REVISION`] and the artifact — not this
/// string (C10, C84). Both binaries link this crate, so a client whose
/// contract sources differ from the daemon's finds it stale and — being
/// a use verb of the interactive CLI — replaces it (kill-and-respawn is
/// the whole migration mechanism). The structured form is
/// `acq version --json`.
pub const VERSION_WITH_CONTRACT: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (contract ",
    env!("ACQ_CONTRACT_REVISION"),
    ")"
);
