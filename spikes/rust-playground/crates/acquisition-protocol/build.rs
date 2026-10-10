//! The shared-contract revision: a digest of the two surfaces every
//! frontend shares (C12) — the protocol crate and the store crate — as
//! the checked-in sources both sides are compiled from, injected as
//! `ACQ_CONTRACT_REVISION` (C84, C10; `client.rs`). Lives in the protocol
//! crate so both sides of the socket carry it. The daemon's own sources
//! are not inputs: a daemon-only edit moves the daemon artifact — the
//! executable's hash, the identity's other half — and nothing else
//! (`DAEMON-SPLIT-SLICE.md`, step 4).
//!
//! Inputs, hashed as `path NUL length NUL bytes` in sorted path order
//! (paths relative to the workspace root) under the domain prefix
//! `acq-contract-revision/1` (the format version): the root `Cargo.toml`,
//! `Cargo.lock`, the protocol and store manifests, and the protocol and
//! store source trees. The lock stays in on purpose: a frontend rebuilt
//! alone after a lock change has new dependencies against an old sibling,
//! and nothing else would see it. Git is not consulted at all: a commit,
//! a stage, a `git status` cost nothing. The inputs are deliberately
//! whole files, not the parts the wire uses, so the revision is
//! conservative in two ways. It moves on changes a peer cannot feel — a
//! lock entry or a root-manifest line its own dependency graph does not
//! include, store code the daemon never calls — and each such move is a
//! false mismatch that respawns a daemon (cheap) rather than a missed
//! contract change (invisible). And a whole-lock or root-manifest change
//! that would not otherwise touch this crate now recompiles it and its
//! dependents once; every other input already did. Features, rustc,
//! profile and target are not inputs: the revision is a pure function
//! of the listed files and claims nothing about the binary — that is the
//! artifact's job.
//!
//! Which files of a source tree are hashed is the routine's rule, stated
//! there: a closed list of kinds, a dotfile passed over, any other file
//! a stop.
//!
//! The routine is `digest.rs`, included here and by the search crate's
//! build script, which names a build by the same rule
//! (`acquisition-search/build.rs`): one maker of a digest of sources.
//!
//! Recompute from a checkout (the same bytes, the same digest):
//! `sha2` here so a shell or Python one-liner can, if a check ever
//! wants to.

include!("digest.rs");

fn main() {
    let revision = digest(
        b"acq-contract-revision/1",
        &[
            "Cargo.toml",
            "Cargo.lock",
            "crates/acquisition-protocol/Cargo.toml",
            "crates/acquisition-store/Cargo.toml",
            "crates/acquisition-protocol/src",
            "crates/acquisition-store/src",
        ],
    );
    println!("cargo:rustc-env=ACQ_CONTRACT_REVISION={revision}");
}
