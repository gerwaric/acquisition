//! The client's side of the artifact dimension (C84): the sibling `acqd`
//! this process would spawn, as the filesystem identifies it, and its
//! hash when the identity alone cannot settle the comparison.
//!
//! What a daemon reports in `hello` is its own executable's
//! [`FileIdentity`] and SHA-256, computed once at its startup. What this
//! process compares it with is the sibling the locator names (C82),
//! `stat`ed at the moment of the comparison — never cached: a long-lived
//! client (the MCP server) must see a rebuilt sibling. An equal identity
//! is the same file. A different identity — a copy at another path, or a
//! replacement at the same path — is settled by hashing the sibling here
//! (tens of milliseconds, the rare path) and comparing digests, so a copy
//! is still the same artifact and a rebuild is a named mismatch. The
//! hashing is one copy here and one in the daemon, because the protocol
//! crate, which defines the identity, links serde alone.

use std::io::Read;
use std::path::{Path, PathBuf};

use acquisition_protocol::artifact::{Artifact, FileIdentity};
use sha2::{Digest, Sha256};

use crate::locator::{self, LocateError};

/// The SHA-256 of the file at `path`, lower-case hex, read in chunks.
pub fn sha256_of(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
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

/// The `acqd` this process would spawn (C82), as the filesystem
/// identifies it right now — or why there is none.
pub fn sibling() -> Result<FileIdentity, SiblingError> {
    let path = locator::acqd().map_err(SiblingError::Absent)?;
    FileIdentity::of(&path).map_err(|e| SiblingError::Unreadable {
        path,
        io: e.to_string(),
    })
}

/// No sibling to compare a daemon with.
#[derive(Debug, Clone)]
pub enum SiblingError {
    /// The locator found no `acqd` beside this executable.
    Absent(LocateError),
    /// There is one, but it could not be `stat`ed.
    Unreadable { path: PathBuf, io: String },
}

impl std::fmt::Display for SiblingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SiblingError::Absent(e) => write!(f, "{e}"),
            SiblingError::Unreadable { path, io } => {
                write!(
                    f,
                    "could not read the acqd beside this executable ({}): {io}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for SiblingError {}

/// Whether `daemon` — the artifact a daemon reported — is the sibling
/// this process would spawn.
#[derive(Debug, Clone)]
pub enum ArtifactVerdict {
    /// The same file, or another file with the same bytes.
    Same,
    /// The sibling here is another artifact: its hash, for the report.
    Different {
        sibling: FileIdentity,
        sibling_sha256: String,
    },
    /// The sibling exists but could not be hashed to settle a differing
    /// identity; treated as a mismatch, and the reason is reported.
    Unhashable { sibling: FileIdentity, io: String },
    /// This process has no sibling to compare with.
    NoSibling(SiblingError),
    /// The daemon reported no artifact (a build before the field).
    Unreported,
}

impl ArtifactVerdict {
    pub fn matches(&self) -> bool {
        matches!(self, ArtifactVerdict::Same)
    }

    /// Judge `daemon` against the sibling as found now.
    pub fn judge(daemon: Option<&Artifact>) -> ArtifactVerdict {
        let Some(daemon) = daemon else {
            return ArtifactVerdict::Unreported;
        };
        let sibling = match sibling() {
            Ok(s) => s,
            Err(e) => return ArtifactVerdict::NoSibling(e),
        };
        if sibling.same_file_as(&daemon.file) {
            return ArtifactVerdict::Same;
        }
        match sha256_of(Path::new(&sibling.path)) {
            Ok(sha256) if sha256 == daemon.sha256 => ArtifactVerdict::Same,
            Ok(sha256) => ArtifactVerdict::Different {
                sibling,
                sibling_sha256: sha256,
            },
            Err(e) => ArtifactVerdict::Unhashable {
                sibling,
                io: e.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C84, the comparison alone (the process-level pin is
    /// `acquisition-cli/tests/daemon_observe.rs`): the same inode is the
    /// same file without a hash; a copy is the same artifact by hash; a
    /// file of other bytes is another artifact; a daemon that reported
    /// nothing matches nothing.
    #[test]
    fn c84_a_copy_is_the_same_artifact_and_other_bytes_are_another() {
        let dir = std::env::temp_dir().join(format!("acq-artifact-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let original = dir.join("acqd");
        std::fs::write(&original, b"the daemon's bytes").unwrap();
        let copy = dir.join("copy");
        std::fs::copy(&original, &copy).unwrap();
        let other = dir.join("other");
        std::fs::write(&other, b"other bytes entirely").unwrap();

        let id = FileIdentity::of(&original).unwrap();
        assert!(id.same_file_as(&FileIdentity::of(&original).unwrap()));
        assert!(!id.same_file_as(&FileIdentity::of(&copy).unwrap()));
        assert_eq!(sha256_of(&original).unwrap(), sha256_of(&copy).unwrap());
        assert_ne!(sha256_of(&original).unwrap(), sha256_of(&other).unwrap());
        assert_eq!(sha256_of(&original).unwrap().len(), 64);
        assert!(!ArtifactVerdict::judge(None).matches());
        assert!(matches!(
            ArtifactVerdict::judge(None),
            ArtifactVerdict::Unreported
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
