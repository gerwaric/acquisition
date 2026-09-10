//! Where `acqd` is: beside the calling executable, and nowhere else.
//!
//! # Decisions as recorded
//!
//! ## C82 — At most one installation targets a world.
//!
//! **C82 — At most one installation targets a world.** Acquisition's
//! installation places one canonical `acqd` and every frontend beside
//! it; the implementation requires it as a sibling of the calling
//! executable (`current_exe()` canonicalised) — no `PATH`, no configured
//! path, no embedded mode; no sibling, no spawn, reported. A second
//! installation targeting the same world is a design event, recorded
//! here first (the playground beside the shipped app targets another
//! world). Tests and drivers locate it the same way; packaging smoke
//! tests are the acceptance criterion. *Why:* C10's "the runtime it would
//! itself spawn" must name one file, or two installations thrash; one
//! directory of siblings is what Acquisition ships. *Details:*
//! `acquisition-client/src/locator.rs` doc. Ruled 2026-09-09.
//!
//! ## C82 — as built
//!
//! One rule, one function: [`beside`] takes an executable's path and
//! names the `acqd` in its directory, refusing when no such file exists;
//! [`acqd`] applies it to this process's own executable, canonicalised
//! first so a symlink into `PATH` (the shipped CLI's shape) resolves to
//! the installation directory that holds the daemon. There is no search
//! order because there is nothing to search: not `PATH` (another
//! installation's daemon would thrash with this one's under C10), not a
//! configured path (no consumer needs one; a knob would be read by the
//! production client), not an embedded `daemon run` (the door C13 closed
//! — the daemon is never in a frontend's process). A missing sibling is
//! a [`LocateError`] naming the executable and the path it looked for,
//! and what puts one there: the installation, or in a source tree
//! `cargo build --workspace` — which is why the quality gate builds
//! before it tests (`AGENTS.md`), since `cargo test` alone does not
//! uplift `target/debug/acqd`.
//!
//! The frontends' process tests start the daemon of the binary they
//! drive through [`beside`] on that binary's path (`CARGO_BIN_EXE_acq`,
//! `CARGO_BIN_EXE_acq-mcp`) — the same file the binary itself would
//! spawn — and a missing one fails before the test runs, with this
//! module's message. `current_exe()` is documented as platform-specific
//! around symlinks and renames, so the layout is a contract the packaging
//! must meet, checked by the packaging smoke tests the packet names (§5).

use std::fmt;
use std::path::{Path, PathBuf};

/// The daemon executable's file name beside every frontend.
pub const ACQD: &str = "acqd";

/// No `acqd` beside the executable that wanted one.
#[derive(Debug, Clone)]
pub struct LocateError {
    /// The executable whose sibling was wanted.
    pub exe: PathBuf,
    /// Where the daemon was looked for.
    pub looked_for: PathBuf,
    /// Why the executable itself could not be resolved, when that is the
    /// failure (`current_exe()` or its canonicalisation).
    pub io: Option<String>,
}

impl fmt::Display for LocateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.io {
            Some(io) => write!(
                f,
                "could not resolve this executable ({}) to find the daemon beside it: {io}",
                self.exe.display()
            ),
            None => write!(
                f,
                "no {ACQD} beside {} (looked for {}): Acquisition's installation places the \
                 daemon beside every frontend (C82); in a source tree, `cargo build --workspace` \
                 builds it",
                self.exe.display(),
                self.looked_for.display()
            ),
        }
    }
}

impl std::error::Error for LocateError {}

/// The `acqd` beside `exe`: the file named [`ACQD`] in `exe`'s directory,
/// which must exist. The rule itself, on a path the caller names.
pub fn beside(exe: &Path) -> Result<PathBuf, LocateError> {
    let dir = exe.parent().unwrap_or_else(|| Path::new("."));
    let looked_for = dir.join(ACQD);
    if looked_for.is_file() {
        Ok(looked_for)
    } else {
        Err(LocateError {
            exe: exe.to_path_buf(),
            looked_for,
            io: None,
        })
    }
}

/// The `acqd` beside this process's own executable, canonicalised first
/// (a symlinked CLI resolves to its installation directory).
pub fn acqd() -> Result<PathBuf, LocateError> {
    let exe = std::env::current_exe().map_err(|e| LocateError {
        exe: PathBuf::from("<current_exe>"),
        looked_for: PathBuf::new(),
        io: Some(e.to_string()),
    })?;
    let canonical = exe.canonicalize().map_err(|e| LocateError {
        exe: exe.clone(),
        looked_for: PathBuf::new(),
        io: Some(e.to_string()),
    })?;
    beside(&canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// C82: the sibling is the rule — the file named `acqd` in the
    /// executable's directory, and nothing else; absent, the error names
    /// the executable, the path looked for, and the build step.
    #[test]
    fn c82_the_daemon_is_the_sibling_named_acqd_or_nothing() {
        let dir = std::env::temp_dir().join(format!("acq-locator-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let exe = dir.join("bin").join("acq");
        std::fs::write(&exe, b"").unwrap();

        let err = beside(&exe).unwrap_err();
        assert_eq!(err.looked_for, dir.join("bin").join("acqd"));
        let text = err.to_string();
        assert!(
            text.contains("no acqd beside")
                && text.contains("cargo build --workspace")
                && text.contains(&exe.display().to_string()),
            "{text}"
        );

        // A daemon elsewhere — on a PATH-like directory, one level up —
        // is not found: only the sibling counts.
        std::fs::write(dir.join("acqd"), b"").unwrap();
        assert!(beside(&exe).is_err());
        std::fs::write(dir.join("bin").join("acqd"), b"").unwrap();
        assert_eq!(beside(&exe).unwrap(), dir.join("bin").join("acqd"));
        // A directory named acqd is not the daemon.
        std::fs::remove_file(dir.join("bin").join("acqd")).unwrap();
        std::fs::create_dir(dir.join("bin").join("acqd")).unwrap();
        assert!(beside(&exe).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
