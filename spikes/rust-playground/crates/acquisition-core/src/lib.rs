//! Playground core for the Acquisition Rust rewrite.
//!
//! By default nothing here talks to GGG: job kinds are fakes and OAuth runs
//! against the in-process mock provider. Starting the daemon with `ACQ_GGG=1`
//! opts into the real provider — real OAuth against the existing
//! "acquisition" registration and a real `GET /character` — behind the same
//! single rate-limit choke point, with deliberately conservative buckets.

pub mod auth;
pub mod client;
pub mod daemon;
pub(crate) mod gate;
pub mod job;
pub mod mockggg;
pub mod protocol;
pub mod provider;
pub mod rails;
pub mod ratelimit;
pub mod realm;

/// The package version. Not the handshake identity on its own: it is
/// fixed at `0.0.1` across every commit of the playground, so comparing it
/// lets a daemon from an older build serve a newer client silently (review
/// finding 2026-09-02: a pre-realm daemon accepted a console job and sent
/// it to pc). [`VERSION_WITH_RUNTIME`] is what the handshake compares.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The runtime revision: twelve hex digits of a digest over the sources
/// the daemon is made of — the core and store crates, their manifests,
/// the root manifest and the lock (`build.rs` lists the inputs and the
/// format). It changes whenever any of those inputs changes — including
/// lock entries and store code the daemon does not use, a deliberate
/// false mismatch — never when the planner or a frontend changes, and
/// never by consulting git. Written into the send journal header and the
/// daemon's startup line: the rails verify behavior, this is the one
/// place that says *which code* behaved; a run record maps it to HEAD.
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
