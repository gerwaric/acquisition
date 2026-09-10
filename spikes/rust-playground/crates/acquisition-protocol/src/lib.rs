//! The daemon's contract as a frontend sees it (C12's first surface), with
//! nothing in it that can change for a client-side reason: the wire
//! ([`protocol`]: `Request`/`Response`, the bootstrap plane, `ErrorKind`,
//! `Quote`, the frame bound), the job model ([`job`]), the job vocabulary
//! ([`realm`]), the status documents the daemon reports ([`status`]), the
//! provider names and which provider this process wants ([`provider`]),
//! and the runtime revision the handshake compares ([`VERSION_WITH_RUNTIME`],
//! computed by `build.rs`). Both sides of the socket link this crate; the
//! IPC that carries the frames does not live here — the client and the
//! daemon each own theirs — so this crate links serde and serde_json and
//! nothing else, and `tools/docs-check.sh` refuses more.
//!
//! Extracted from `acquisition-core` (now `acquisition-daemon`) as commit 1 of the daemon split
//! (`DAEMON-SPLIT-SLICE.md`, step ledger; the design is
//! `brainstorming-notes/18-the-daemon-split.md` §2.1). Until step 4 the
//! revision's inputs are the daemon's sources too, so a daemon edit still
//! moves the identity a running daemon is compared against (C10).

pub mod job;
pub mod protocol;
pub mod provider;
pub mod realm;
pub mod status;

/// The package version. Not the handshake identity on its own: it is
/// fixed at `0.0.1` across every commit of the playground, so comparing it
/// lets a daemon from an older build serve a newer client silently (review
/// finding 2026-09-02: a pre-realm daemon accepted a console job and sent
/// it to pc). [`VERSION_WITH_RUNTIME`] is what the handshake compares.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The runtime revision: twelve hex digits of a digest over the sources
/// the daemon is made of — the daemon, store and protocol crates, their
/// manifests, the root manifest and the lock (`build.rs` lists the inputs
/// and the format). It changes whenever any of those inputs changes —
/// including lock entries and store code the daemon does not use, a
/// deliberate false mismatch — never on a source edit to the planner, the
/// client crate or a frontend (a dependency change there moves the lock, and the lock is
/// an input on purpose), and never by consulting git. Written into the
/// send journal header and the daemon's startup line: the rails verify
/// behavior, this is the one place that says *which code* behaved; a run
/// record maps it to HEAD.
pub const RUNTIME_REVISION: &str = env!("ACQ_RUNTIME_REVISION");

/// `<version> (runtime <revision>)`: what `acq --version` prints and what
/// the client/daemon handshake compares (C10). Both binaries link this
/// crate, so a client whose runtime sources differ from the daemon's
/// finds it stale and — being a use verb of the interactive CLI —
/// replaces it (kill-and-respawn is the whole migration mechanism). The
/// structured form is `acq version --json`.
pub const VERSION_WITH_RUNTIME: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (runtime ",
    env!("ACQ_RUNTIME_REVISION"),
    ")"
);
