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

/// The package version. Not the handshake stamp on its own: it is fixed
/// at `0.0.1` across every commit of the playground, so comparing it lets a
/// daemon from an older build serve a newer client silently (review
/// finding 2026-09-02: a pre-realm daemon accepted a console job and sent
/// it to pc). [`VERSION_WITH_BUILD`] is what the handshake compares.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The git commit this binary was built from (`<short-hash>` or
/// `<short-hash>-dirty`; `unknown` outside a checkout). Written into the send
/// journal header and the daemon's startup line: the rails verify behavior,
/// this is the one place that says *which code* behaved.
pub const BUILD: &str = env!("ACQ_BUILD");

/// `<version> (<build>)`: what `acq --version` prints and what the
/// client/daemon handshake compares. Both binaries link this crate, so a
/// client from another commit finds the daemon stale and — being the
/// interactive CLI — replaces it (kill-and-respawn is the whole migration
/// mechanism). Accepted residual: a `-dirty` stamp is the same for any
/// dirty tree, which is why a live daemon is never rebuilt under
/// (`LIVE-TESTING.md`, "verify the binary, not the checkout").
pub const VERSION_WITH_BUILD: &str =
    concat!(env!("CARGO_PKG_VERSION"), " (", env!("ACQ_BUILD"), ")");
