//! The runtime revision: a digest of the checked-in sources the daemon
//! is made of, injected as `ACQ_RUNTIME_REVISION` (C10, `client.rs`).
//!
//! Inputs, hashed as `path NUL length NUL bytes` in sorted path order
//! (paths relative to the workspace root) under the domain prefix
//! `acq-runtime-revision/1` (the format version): the root `Cargo.toml`,
//! `Cargo.lock`, the core and store manifests, and the core and store
//! source trees. Every one of them already recompiles this crate when it
//! changes, so the stamp adds no invalidation of its own — a commit, a
//! stage, a `git status` cost nothing, and git is not consulted at all.
//! Features, rustc, profile and target are not inputs: the revision is a
//! pure function of the listed files and claims nothing about the
//! binary. `Cargo.lock` stays in on purpose: a false respawn on an
//! unrelated lock change is cheaper than an invisible dependency change.
//!
//! Recompute from a checkout (the same bytes, the same digest):
//! `sha2` here so a shell or Python one-liner can, if a check ever
//! wants to.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

const PREFIX: &[u8] = b"acq-runtime-revision/1";

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let inputs = [
        "Cargo.toml",
        "Cargo.lock",
        "crates/acquisition-core/Cargo.toml",
        "crates/acquisition-store/Cargo.toml",
        "crates/acquisition-core/src",
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
    println!("cargo:rustc-env=ACQ_RUNTIME_REVISION={}", &hex[..12]);
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
        panic!("runtime-revision input missing: {}", path.display());
    }
}
