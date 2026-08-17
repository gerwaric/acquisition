//! Playground core for the Acquisition Rust rewrite.
//!
//! IMPORTANT: nothing in this workspace talks to GGG. Job kinds are fakes
//! (`sleep`, `fetch`, `profile`) and the rate limiter is a set of simulated
//! token buckets, so we can exercise queueing, ETAs, and long-wait ergonomics
//! with zero risk to the OAuth registration.

pub mod auth;
pub mod daemon;
pub mod job;
pub mod mockggg;
pub mod protocol;
pub mod ratelimit;

/// Shared version used for the client/daemon handshake. Both binaries link
/// this crate, so a rebuilt CLI with a changed core will detect a stale daemon.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
