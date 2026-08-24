//! Playground core for the Acquisition Rust rewrite.
//!
//! By default nothing here talks to GGG: job kinds are fakes and OAuth runs
//! against the in-process mock provider. Starting the daemon with `ACQ_GGG=1`
//! opts into the real provider — real OAuth against the existing
//! "acquisition" registration and a real `GET /character` — behind the same
//! single rate-limit choke point, with deliberately conservative buckets.

pub mod auth;
pub mod daemon;
pub(crate) mod gate;
pub mod job;
pub mod mockggg;
pub mod protocol;
pub mod provider;
pub mod rails;
pub mod ratelimit;

/// Shared version used for the client/daemon handshake. Both binaries link
/// this crate, so a rebuilt CLI with a changed core will detect a stale daemon.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The git commit this binary was built from (`<short-hash>` or
/// `<short-hash>-dirty`; `unknown` outside a checkout). Written into the send
/// journal header and the daemon's startup line: the rails verify behavior,
/// this is the one place that says *which code* behaved.
pub const BUILD: &str = env!("ACQ_BUILD");

/// `<version> (<build>)`, what `acq --version` prints.
pub const VERSION_WITH_BUILD: &str =
    concat!(env!("CARGO_PKG_VERSION"), " (", env!("ACQ_BUILD"), ")");
