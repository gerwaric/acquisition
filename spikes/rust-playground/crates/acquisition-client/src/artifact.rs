//! The client's side of the artifact dimension (C84): the sibling `acqd`
//! this process would spawn, as the filesystem identifies it, and its
//! hash when the identity alone cannot settle the comparison.
//!
//! What a daemon reports in `hello` is its own executable's
//! [`FileIdentity`] and SHA-256, computed once at its startup. What this
//! process compares it with is the sibling the locator names (C82),
//! opened at the moment of the comparison — never cached: a long-lived
//! client (the MCP server) must see a rebuilt sibling — with the
//! identity taken from that handle's metadata and, when it must be
//! hashed, the bytes read from the same handle, so what a report names
//! and what it judged are one file (review 2026-09-11). An equal
//! identity is the same file. A different identity — a copy at another
//! path, or a replacement at the same path — is settled by hashing the
//! sibling (tens of milliseconds, the rare path) and comparing digests,
//! so a copy is still the same artifact and a rebuild is a named
//! mismatch. The hashing is one copy here and one in the daemon, because
//! the protocol crate, which defines the identity, links serde alone.

use std::io::Read;
use std::path::{Path, PathBuf};

use acquisition_protocol::artifact::{Artifact, FileIdentity};
use sha2::{Digest, Sha256};

use crate::locator::{self, LocateError};

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

/// The sibling `acqd`, open: its identity from the handle's metadata,
/// and the handle itself for the hash that may follow.
pub struct Sibling {
    pub identity: FileIdentity,
    file: std::fs::File,
}

impl Sibling {
    /// Open the file at `path` (canonicalised first) and identify it from
    /// the open handle.
    pub fn open(path: &Path) -> Result<Sibling, SiblingError> {
        let unreadable = |io: std::io::Error| SiblingError::Unreadable {
            path: path.to_path_buf(),
            io: io.to_string(),
        };
        let canonical = path.canonicalize().map_err(unreadable)?;
        let file = std::fs::File::open(&canonical).map_err(unreadable)?;
        let meta = file.metadata().map_err(unreadable)?;
        Ok(Sibling {
            identity: FileIdentity::of_open(&canonical, &meta),
            file,
        })
    }
}

/// The `acqd` this process would spawn (C82), opened and identified
/// right now — or why there is none.
pub fn sibling() -> Result<Sibling, SiblingError> {
    let path = locator::acqd().map_err(SiblingError::Absent)?;
    Sibling::open(&path)
}

/// No sibling to compare a daemon with.
#[derive(Debug, Clone)]
pub enum SiblingError {
    /// The locator found no `acqd` beside this executable.
    Absent(LocateError),
    /// There is one, but it could not be opened or identified.
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
/// this process would spawn. "The same artifact" is two cases kept
/// apart (review 2026-09-10): the sibling *is* the file the daemon runs
/// from, or the sibling is another file with the same bytes — a
/// supported case (a copied installation) that a report must not call
/// "the sibling this client would start".
#[derive(Debug, Clone)]
pub enum ArtifactVerdict {
    /// The daemon runs from this process's sibling itself: the same
    /// inode, the same length and modification time.
    SameFile,
    /// The daemon runs from another file whose bytes are the sibling's:
    /// the same artifact, another copy.
    SameBytes { sibling: FileIdentity },
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
    /// The same artifact, by inode or by bytes.
    pub fn matches(&self) -> bool {
        matches!(
            self,
            ArtifactVerdict::SameFile | ArtifactVerdict::SameBytes { .. }
        )
    }

    /// The relation as one word for a report: `same_file`, `same_bytes`,
    /// `different`, `unhashable`, `no_sibling`, `unreported`.
    pub fn relation(&self) -> &'static str {
        match self {
            ArtifactVerdict::SameFile => "same_file",
            ArtifactVerdict::SameBytes { .. } => "same_bytes",
            ArtifactVerdict::Different { .. } => "different",
            ArtifactVerdict::Unhashable { .. } => "unhashable",
            ArtifactVerdict::NoSibling(_) => "no_sibling",
            ArtifactVerdict::Unreported => "unreported",
        }
    }

    /// The comparison itself, with the sibling supplied open: the same
    /// inode is the same file without a hash; otherwise the sibling's
    /// bytes, read from the handle that was identified, decide.
    pub fn judge_against(
        daemon: Option<&Artifact>,
        sibling: Result<Sibling, SiblingError>,
    ) -> ArtifactVerdict {
        let Some(daemon) = daemon else {
            return ArtifactVerdict::Unreported;
        };
        let Sibling {
            identity: sibling,
            mut file,
        } = match sibling {
            Ok(s) => s,
            Err(e) => return ArtifactVerdict::NoSibling(e),
        };
        if sibling.same_file_as(&daemon.file) {
            return ArtifactVerdict::SameFile;
        }
        match sha256_of_open(&mut file) {
            Ok(sha256) if sha256 == daemon.sha256 => ArtifactVerdict::SameBytes { sibling },
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
    /// same file without a hash; a copy is the same artifact by hash,
    /// and the verdict keeps the two apart; a file of other bytes is
    /// another artifact; a daemon that reported nothing matches nothing.
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
        assert!(!ArtifactVerdict::judge_against(None, sibling()).matches());
        assert!(matches!(
            ArtifactVerdict::judge_against(None, sibling()),
            ArtifactVerdict::Unreported
        ));

        let daemon = Artifact {
            file: id.clone(),
            sha256: sha256_of(&original).unwrap(),
        };
        let judge = |sibling: &std::path::Path| {
            ArtifactVerdict::judge_against(Some(&daemon), Sibling::open(sibling))
        };
        let v = judge(&original);
        assert!(v.matches() && v.relation() == "same_file", "{v:?}");
        let v = judge(&copy);
        assert!(v.matches() && v.relation() == "same_bytes", "{v:?}");
        assert!(
            matches!(&v, ArtifactVerdict::SameBytes { sibling } if sibling.path == copy.canonicalize().unwrap().to_string_lossy()),
            "{v:?}"
        );
        let v = judge(&other);
        assert!(!v.matches() && v.relation() == "different", "{v:?}");
        let v = judge(&dir.join("missing"));
        assert!(!v.matches() && v.relation() == "no_sibling", "{v:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The single-handle guarantee (review 2026-09-11): a sibling opened
    /// and then replaced at its path — the rebuild-between-two-looks
    /// case — is judged from the handle that was opened, and the identity
    /// the verdict names is that handle's. A judge that reopened the path
    /// would see the replacement's bytes and fail here.
    #[test]
    fn c84_the_sibling_is_judged_from_the_handle_that_was_identified() {
        let dir = std::env::temp_dir().join(format!("acq-handle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let original = dir.join("acqd-original");
        std::fs::write(&original, b"the daemon's bytes").unwrap();
        let daemon = Artifact {
            file: FileIdentity::of(&original).unwrap(),
            sha256: sha256_of(&original).unwrap(),
        };
        let replacement = |path: &std::path::Path| {
            let staged = dir.join("staged");
            std::fs::write(&staged, b"a rebuilt daemon, other bytes").unwrap();
            std::fs::rename(&staged, path).unwrap();
        };

        // A copy of the daemon's bytes, opened, then replaced at its path.
        let copy = dir.join("acqd");
        std::fs::copy(&original, &copy).unwrap();
        let opened = Sibling::open(&copy).unwrap();
        let opened_identity = opened.identity.clone();
        replacement(&copy);
        assert_ne!(
            sha256_of(&copy).unwrap(),
            daemon.sha256,
            "the path now holds other bytes"
        );
        let v = ArtifactVerdict::judge_against(Some(&daemon), Ok(opened));
        assert!(v.relation() == "same_bytes", "{v:?}");
        assert!(
            matches!(&v, ArtifactVerdict::SameBytes { sibling } if *sibling == opened_identity),
            "the identity named is the opened handle's: {v:?}"
        );

        // The daemon's own file, opened, then replaced at its path: still
        // the same file, by the identity the handle had.
        let opened = Sibling::open(&original).unwrap();
        replacement(&original);
        let v = ArtifactVerdict::judge_against(Some(&daemon), Ok(opened));
        assert!(v.relation() == "same_file", "{v:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
