// A digest of checked-in sources, for a build script to `include!`: the
// protocol crate's (the contract revision, C84) and the search crate's
// (the build a basis names, C98). Not a module of either crate.
//
// The inputs are paths relative to the workspace root, files or
// directories, hashed as `path NUL length NUL bytes` in sorted path
// order under a domain prefix; the first twelve hex digits of the
// SHA-256 are the digest. Every input is a `rerun-if-changed`.
//
// Which files of a directory (the owner, 2026-10-10: "yes"): a closed
// list of kinds, `KINDS`, the ones a build reads. A dotfile is passed
// over — what an operating system or an editor writes unasked, a
// `.DS_Store` — and a file of any other kind stops the build and is
// named. So a leftover is never hashed into a digest in silence, which
// would name one tree two ways on two machines, and a new kind of
// source is never left out in silence, which would name two trees one
// way. Git is not consulted: an untracked file of a listed kind is in,
// being a difference in the tree that is built.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub(crate) fn digest(prefix: &[u8], inputs: &[&str]) -> String {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let root = manifest_dir
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    for input in inputs {
        println!("cargo:rerun-if-changed={}", root.join(input).display());
    }
    digest_under(&root, prefix, inputs)
}

/// The digest of `inputs` under `root`: a pure function of the files.
pub(crate) fn digest_under(root: &Path, prefix: &[u8], inputs: &[&str]) -> String {
    let mut files: Vec<PathBuf> = Vec::new();
    for input in inputs {
        collect(&root.join(input), &mut files);
    }
    files.sort();
    let mut hasher = Sha256::new();
    hasher.update(prefix);
    hasher.update([0]);
    for file in &files {
        let rel = file
            .strip_prefix(root)
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

/// The kinds of file a digest is of, by extension.
const KINDS: &[&str] = &["rs", "toml", "lock", "sql", "json", "txt"];

/// Every file of a listed kind under `path` (itself, if it is one).
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
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        let kind = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
        if name.starts_with('.') {
            return;
        }
        if !KINDS.contains(&kind) {
            panic!(
                "{} is a file of a kind the digest does not know ({}): add the kind or remove the file",
                path.display(),
                KINDS.join(", ")
            );
        }
        out.push(path.to_path_buf());
    } else {
        panic!("digest input missing: {}", path.display());
    }
}
