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
//! Recompute from a checkout (the same bytes, the same digest):
//! `sha2` here so a shell or Python one-liner can, if a check ever
//! wants to.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const PREFIX: &[u8] = b"acq-contract-revision/1";

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let inputs = [
        "Cargo.toml",
        "Cargo.lock",
        "crates/acquisition-protocol/Cargo.toml",
        "crates/acquisition-store/Cargo.toml",
        "crates/acquisition-protocol/src",
        "crates/acquisition-store/src",
    ];
    let mut files: Vec<PathBuf> = Vec::new();
    for input in inputs {
        let path = root.join(input);
        println!("cargo:rerun-if-changed={}", path.display());
        collect(&path, &mut files);
    }
    files.sort();
    let mut hasher = Sha256::new();
    hasher.update(PREFIX);
    hasher.update([0]);
    for file in &files {
        let rel = file
            .strip_prefix(&root)
            .expect("input under the workspace root")
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = std::fs::read(file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        hasher.update(rel.as_bytes());
        hasher.update([0]);
        hasher.update(bytes.len().to_string().as_bytes());
        hasher.update([0]);
        hasher.update(&bytes);
    }
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    println!("cargo:rustc-env=ACQ_CONTRACT_REVISION={}", &hex[..12]);
}

/// Every regular file under `path` (itself, if it is one).
fn collect(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(path)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
            .map(|e| e.expect("dir entry").path())
            .collect();
        entries.sort();
        for entry in entries {
            collect(&entry, out);
        }
    } else if path.is_file() {
        out.push(path.to_path_buf());
    } else {
        panic!("contract-revision input missing: {}", path.display());
    }
}
