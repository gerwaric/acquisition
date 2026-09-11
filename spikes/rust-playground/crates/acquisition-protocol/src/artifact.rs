//! The daemon artifact — the executable a daemon runs from — as the
//! bootstrap plane carries it (C84): the file's identity and its hash.
//!
//! The runtime identity (C10) is two values from two sources. The
//! shared-contract revision ([`crate::CONTRACT_REVISION`]) answers "can I
//! use it"; the artifact answers "is it the one I would start". At
//! startup the daemon resolves its own executable, records the file's
//! identity ([`FileIdentity`]: device, inode, length, modification time)
//! and hashes it once (SHA-256, tens of milliseconds, once per lifetime),
//! and reports all of it in `hello`. A client reads the identity of the
//! sibling `acqd` it would spawn (C82) — one open, its metadata read,
//! under a millisecond — and compares: an equal identity is the same file, no hash needed; any
//! difference (a copy at another path, an atomic replacement at the same
//! path) is settled by hashing the sibling and comparing digests, so a
//! copy is still the same artifact and a rebuild under a live daemon is a
//! named mismatch. The identity is defined here, on the contract, because
//! both sides must compute it the same way for equality to mean anything;
//! the hashing itself needs `sha2` and lives on each side (the protocol
//! crate links serde alone).

use std::path::Path;

use serde::{Deserialize, Serialize};

/// A file as the filesystem identifies it: enough to tell "the same
/// file" from "another file at the same path" without reading it. On
/// Unix `dev` and `ino` name the inode; elsewhere they are 0 and the
/// length and modification time carry the comparison (the Windows arm is
/// parked with the rest of the platform work).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileIdentity {
    /// The canonical path, as resolved by whoever opened it.
    pub path: String,
    pub len: u64,
    /// Modification time, nanoseconds since the Unix epoch (0 when the
    /// filesystem reports none).
    pub mtime_ns: u64,
    pub dev: u64,
    pub ino: u64,
}

impl FileIdentity {
    /// The identity of the file at `path`, canonicalised first: a
    /// convenience for a look that reads nothing else. A side that also
    /// hashes the file takes both from one open handle
    /// ([`FileIdentity::of_open`]), so the identity and the bytes are one
    /// snapshot.
    pub fn of(path: &Path) -> std::io::Result<FileIdentity> {
        let canonical = path.canonicalize()?;
        let meta = std::fs::metadata(&canonical)?;
        Ok(FileIdentity::of_open(&canonical, &meta))
    }

    /// The identity of an open file, from its handle's metadata; `path`
    /// is the canonical path it was opened by. Whoever hashes the same
    /// handle afterwards has hashed exactly this file.
    pub fn of_open(path: &Path, meta: &std::fs::Metadata) -> FileIdentity {
        let mtime_ns = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX));
        #[cfg(unix)]
        let (dev, ino) = {
            use std::os::unix::fs::MetadataExt;
            (meta.dev(), meta.ino())
        };
        #[cfg(not(unix))]
        let (dev, ino) = (0, 0);
        FileIdentity {
            path: path.to_string_lossy().into_owned(),
            len: meta.len(),
            mtime_ns,
            dev,
            ino,
        }
    }

    /// The same inode with the same length and modification time: the
    /// same file, without reading it.
    pub fn same_file_as(&self, other: &FileIdentity) -> bool {
        self.dev == other.dev
            && self.ino == other.ino
            && self.len == other.len
            && self.mtime_ns == other.mtime_ns
    }
}

/// The daemon's executable as its `hello` reports it: the file identity
/// and the SHA-256 of its bytes, lower-case hex.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    #[serde(flatten)]
    pub file: FileIdentity,
    pub sha256: String,
}

impl Artifact {
    /// The hash, shortened for prose (the JSON carries it whole).
    pub fn short_hash(&self) -> &str {
        self.sha256.get(..12).unwrap_or(&self.sha256)
    }
}
