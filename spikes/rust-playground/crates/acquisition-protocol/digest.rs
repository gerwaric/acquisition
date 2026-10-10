// A digest of checked-in sources, for a build script to `include!`: the
// protocol crate's (the contract revision, C84) and the search crate's
// (the build a basis names, C98). Not a module of either crate.
//
// The inputs are paths relative to the workspace root, files or
// directories, hashed as `path NUL length NUL bytes` in sorted path
// order under a domain prefix; the first twelve hex digits of the
// SHA-256 are the digest. Every input is a `rerun-if-changed`.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn digest(prefix: &[u8], inputs: &[&str]) -> String {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let mut files: Vec<PathBuf> = Vec::new();
    for input in inputs {
        let path = root.join(input);
        println!("cargo:rerun-if-changed={}", path.display());
        collect(&path, &mut files);
    }
    files.sort();
    let mut hasher = Sha256::new();
    hasher.update(prefix);
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
    let hex: String = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    hex[..12].to_string()
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
        panic!("digest input missing: {}", path.display());
    }
}
