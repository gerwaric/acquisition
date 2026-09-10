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
pub mod frame;
pub(crate) mod gate;
pub mod mockggg;
pub mod provider;
pub mod rails;
pub mod ratelimit;
