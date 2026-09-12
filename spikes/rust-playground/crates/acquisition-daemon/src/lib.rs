//! The daemon of the Acquisition Rust rewrite, as its own artifact (C1,
//! C82; the binary is `acqd`, `main.rs`): the header-driven rate limiter
//! and its choke point (`ratelimit.rs`, `gate.rs`), the live-test rails
//! (`rails.rs`), OAuth and sessions (`auth.rs`), the provider (`provider.rs`),
//! the mock provider (`mockggg.rs`), the daemon's own executable identified
//! and hashed at startup (`artifact.rs`, C84), and the daemon itself
//! (`daemon.rs`: queue, dispatcher, Unix-socket server, idle watchdog). The only GGG
//! sender. It links the protocol crate and the store, never the client
//! crate, the planner or a frontend — `tools/docs-check.sh` refuses the
//! edges — and no package but this one names it, so "never in-process
//! with the daemon" (C13) is a Cargo fact.
//!
//! The daemon serves the real provider by default (C88) — real OAuth
//! against the existing "acquisition" registration and the real API —
//! behind the single rate-limit choke point; `ACQ_PROVIDER=mock` starts
//! it against the in-process mock instead, where job kinds are fakes and
//! nothing talks to GGG: the test double and the rehearsal stage.
//!
//! Renamed from `acquisition-core` by `git mv` at the daemon split's step
//! 3 (`DAEMON-SPLIT-SLICE.md`); the client side (`client.rs`) moved to
//! `acquisition-client` in the same commit, and `frame.rs` — the bounded
//! reader both sides need — is one copy here and one there, because the
//! protocol crate stays serde-only and the daemon never links the client.

pub mod artifact;
pub mod auth;
pub mod daemon;
pub mod frame;
pub(crate) mod gate;
pub mod mockggg;
pub mod provider;
pub mod rails;
pub mod ratelimit;
