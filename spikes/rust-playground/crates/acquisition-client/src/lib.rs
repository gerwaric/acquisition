//! The client side of the daemon protocol (C1, C12's first surface as a
//! frontend reaches it): the IPC over tokio and the three policy doors of
//! C10 ([`client`]), the `acqd` locator ([`locator`], C82), the artifact
//! comparison against that sibling ([`artifact`], C84), the world
//! comparison against this process's own (C83; the world itself is the
//! store's `world.rs`), the bounded frame reader ([`frame`]) and the
//! typed failures at the door ([`client::ConnectError`]). Every frontend
//! links this crate and the protocol crate; this crate links the protocol
//! crate, the store and tokio, never the daemon (`acquisition-daemon`) or
//! the planner, and `tools/docs-check.sh` refuses both edges.
//!
//! Extracted from `acquisition-core` as commit 3 of the daemon split
//! (`DAEMON-SPLIT-SLICE.md`; the design is
//! `brainstorming-notes/18-the-daemon-split.md` §2.1, §2.3): a client or
//! locator edit must not recompile the daemon, and the GUI and the MCP
//! server need typed failures rather than strings. The store joined at
//! step 5 with the world: the socket, the log and the root every side
//! reads are one definition there (`acquisition_store::world`), where two
//! copies of the socket convention had stood before.

pub mod artifact;
pub mod client;
pub mod frame;
pub mod locator;
