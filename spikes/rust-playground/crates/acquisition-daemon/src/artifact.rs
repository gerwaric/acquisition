//! The daemon's half of the artifact dimension (C84): the executable this
//! process runs from, identified and hashed once at startup, reported in
//! `hello`, the startup identity line in the log and the journal header.
//!
//! The identity's definition is the protocol crate's
//! (`acquisition_protocol::artifact::FileIdentity`), so the client
//! computes the same thing of the sibling it would spawn; the hashing is
//! one copy here and one in the client crate, because the protocol crate
//! links serde alone. A daemon that cannot read its own executable does
//! not start: without an artifact no client can use it, and the failure
//! is named in the log rather than discovered as a mismatch.

use std::io::Read;
use std::path::Path;

use acquisition_protocol::artifact::{Artifact, FileIdentity};
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// The SHA-256 of the file at `path`, lower-case hex, read in chunks.
pub fn sha256_of(path: &Path) -> std::io::Result<String> {
    sha256_of_open(&mut std::fs::File::open(path)?)
}

/// The SHA-256 of an open file, from its start.
pub fn sha256_of_open(file: &mut std::fs::File) -> std::io::Result<String> {
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// This process's executable: `current_exe()` canonicalised, opened
/// once, its identity from that handle's metadata and its hash from the
/// same handle — one snapshot. Once per lifetime, at startup.
pub fn of_current_exe() -> Result<Artifact> {
    let exe = std::env::current_exe().context("resolving the daemon's own executable")?;
    let canonical = exe
        .canonicalize()
        .with_context(|| format!("resolving the daemon's executable {}", exe.display()))?;
    let mut handle = std::fs::File::open(&canonical)
        .with_context(|| format!("opening the daemon's executable {}", canonical.display()))?;
    let meta = handle.metadata().with_context(|| {
        format!(
            "identifying the daemon's executable {}",
            canonical.display()
        )
    })?;
    let file = FileIdentity::of_open(&canonical, &meta);
    let sha256 = sha256_of_open(&mut handle)
        .with_context(|| format!("hashing the daemon's executable {}", file.path))?;
    Ok(Artifact { file, sha256 })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C84: the daemon's artifact is its own executable — the canonical
    /// path, a 64-hex SHA-256 of exactly that file — and the same file
    /// stat'ed twice is the same identity.
    #[test]
    fn c84_the_artifact_is_this_executable_identified_and_hashed() {
        let artifact = of_current_exe().unwrap();
        let exe = std::env::current_exe().unwrap().canonicalize().unwrap();
        assert_eq!(Path::new(&artifact.file.path), exe.as_path());
        assert_eq!(artifact.sha256.len(), 64);
        assert!(artifact.sha256.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(artifact.sha256, sha256_of(&exe).unwrap());
        assert!(artifact.file.same_file_as(&FileIdentity::of(&exe).unwrap()));
        assert!(artifact.file.len > 0);
    }
}
