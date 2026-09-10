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

// The wire, the job model, the vocabulary and the runtime revision are the
// protocol crate's since the daemon split's step 1 (DAEMON-SPLIT-SLICE.md).
// Re-exported at their old paths for one commit, so the planner and the
// frontends compile unchanged until step 2 switches them; step 2 deletes
// these lines.
pub use acquisition_protocol::{
    RUNTIME_REVISION, VERSION, VERSION_WITH_RUNTIME, job, protocol, realm,
};
