//! The world (C83; C1: "the store holds … the world (root, locks, socket
//! name)"): pure path functions and the two lock files, no IPC. A world
//! is a canonical, provider-neutral store root — `ACQ_STORE_DIR` or the
//! platform data directory, canonicalised — above the `<provider>/`
//! directories the store keeps facts, intent and `daemon.db` in. Every
//! side of the socket computes the same paths from the same environment:
//! the daemon (`acquisition-daemon`) to create, lock and write, the
//! client (`acquisition-client`) to compare the world a daemon reports
//! with its own and to read the log after a failed spawn, the frontends
//! to print them.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/daemon.md` for this
//! area (`C<n>`); what follows is the entry's full text as recorded
//! there, kept beside the code that implements it.
//!
//! ## C83 — A world is a canonical, provider-neutral store root
//!
//! **C83 — A world is a canonical, provider-neutral store root; its
//! daemon holds an exclusive lock on it for its lifetime, and in real
//! mode a per-OS-user lock no root bypasses.** A use path creates the
//! root, then canonicalises and locks; observation creates nothing. The
//! socket is derived from the root into a private per-user runtime
//! directory, never chosen by hand (`ACQ_SOCKET` is gone); `hello` names
//! the root and a client refuses a daemon on another world. Durable state
//! (`daemon.db`, rails state) lives in the world; the log and journal are
//! bounded diagnostics elsewhere. *Why:* a socket name is discovery, not
//! ownership — C6 and C31 were held by documentation, and the tripwire's
//! state lived in a directory the OS clears at reboot. *Details:*
//! `world.rs` doc. Ruled 2026-09-09.
//!
//! ## C83 — as built (the daemon split's steps 5 and 6)
//!
//! - **The root.** [`intended_root`] is the path the environment names:
//!   `ACQ_STORE_DIR` (a relative value made absolute against the current
//!   directory, so the same spelling crosses into a spawned `acqd`), else
//!   the platform data directory's `store` — the base [`store_dir`] has
//!   always put the provider directories under. [`World::create`] is the
//!   use path: it creates the root (mode 0700 on Unix) and canonicalises
//!   it; [`World::observe`] is the observation path: it canonicalises
//!   what exists and creates nothing, so a missing root is an absent
//!   world, never a new one; the use path sets the root to mode 0700 on
//!   every start, not only when it creates it, and refuses if it cannot
//!   (a root from before step 5 is brought to the documented state).
//!   `fs::canonicalize` is what makes two spellings of one directory one
//!   world — a symlinked data directory, macOS's `/var` and
//!   `/private/var` — and what a daemon reports in `hello` (`world`),
//!   which a client compares with its own before any other dimension. A
//!   root that is not valid UTF-8 is refused rather than carried lossily:
//!   the name crosses the wire as a string, and two distinct byte paths
//!   must never read as one world (review 2026-09-11).
//! - **The world lock.** [`World::lock_path`] is `<root>/daemon.lock`;
//!   [`Lock::acquire`] takes an exclusive advisory lock on it (`flock`,
//!   through `std::fs::File::try_lock`) and writes the holder's pid into
//!   the file; the lock lives as long as the [`Lock`] — the daemon keeps
//!   it for its lifetime, and the kernel releases it however the process
//!   ends. A second daemon on the root fails to acquire, reads the pid,
//!   and refuses to start naming the holder ([`LockError`]). This is what
//!   makes C6's "one daemon per store directory" structure rather than a
//!   sentence: two daemons on one `daemon.db` would each restore and run
//!   the same queue.
//! - **The real-mode lock.** [`real_mode_lock_path`] is
//!   `<runtime>/ggg.lock`, in the private per-user runtime directory
//!   ([`app_runtime_dir`]: `$XDG_RUNTIME_DIR/acq` where the platform sets
//!   it, else `<temp dir>/acq-<uid>` — `/tmp` is shared on Linux, so the
//!   fallback carries the uid; either way the directory is created with
//!   mode 0700 in the creating call — never made loose and tightened
//!   after —, must not be a symlink, must be owned by this user and must
//!   be exactly 0700, restored if it drifted or refused, — the
//!   pre-creation attack on a shared temp directory, review 2026-09-11).
//!   Every real-mode daemon takes it whatever its root, so two live-test
//!   roots cannot make two GGG gates (C31's Cloudflare bound is
//!   per-process state). Per user, not per machine: another OS user, the
//!   C++ Acquisition and other machines behind the same address are
//!   outside it — C31 names them as external concurrency the tripwire
//!   exists for. The mock never takes it.
//! - **Durable state in the world.** `daemon.db` (`jobs::daemon_db_path`)
//!   and the rails state ([`World::rails_state_path`]:
//!   `<root>/<provider>/rails.json`) live beside the account files, where
//!   a reboot cannot clear them.
//! - **Diagnostics elsewhere, bounded.** The daemon log and the default
//!   send journal live under [`log_base_dir`] — `ACQ_LOG_DIR`, else the
//!   platform's log directory (`~/Library/Logs/acquisition-playground` on
//!   macOS, the XDG state directory's `log` on Linux) — one subdirectory
//!   per world ([`World::id`], twelve hex digits of the canonical root's
//!   SHA-256, beside a `world` file naming the root for a human browsing
//!   there) and provider: [`World::log_path`] is
//!   `<base>/<id>/<provider>/daemon.log`, [`World::journal_path`]
//!   `<base>/<id>/<provider>/sends.jsonl`. They are append-only and
//!   never durable state; the daemon bounds them (rotated once at its
//!   start past a cap — the mechanism and the size are the daemon's,
//!   `daemon.rs`). `ACQ_JOURNAL` still overrides the journal — a live run
//!   points it into its evidence directory, where the ledger cites it —
//!   and `0` disables it (`rails.rs`).
//! - **The socket.** [`World::socket_path`] is `<runtime>/<id>.sock` —
//!   the world's twelve-hex id in the private per-user runtime directory
//!   (the split's step 6). Nothing chooses it by hand, and two spellings
//!   of one root reach one socket because the id is the canonical root's. Short under the platform's defaults: the
//!   runtime directory is `$XDG_RUNTIME_DIR` (`/run/user/<uid>`) or
//!   macOS's fixed-shape `$TMPDIR`, so the whole path is about 75 bytes
//!   under the macOS fallback — but both variables are the
//!   environment's, so the derivation ([`World::socket_path_under`])
//!   refuses by name a path over [`SOCKET_PATH_MAX`] or one that is not
//!   valid UTF-8 (the socket is named in every report; Linux permits
//!   other bytes in `XDG_RUNTIME_DIR`), rather than leaving the first to
//!   `bind`'s `ENAMETOOLONG` and the second to a report that cannot be
//!   built. The daemon creates the runtime directory
//!   ([`app_runtime_dir`]) before it binds; a client only verifies it
//!   ([`existing_private_dir`]: this user's, 0700, no symlink) before it
//!   connects, so a socket in a directory another user made is never
//!   used, and an observation creates nothing — no runtime directory yet
//!   is no daemon yet. A world that does not exist has no socket: a use
//!   door creates the root first, an observer reports absence.
//!
//! Windows has no arm here yet (`README.md`, known gaps): the paths are
//! computed, the locks use `std`'s portable file locking, the mode bits
//! are Unix-only.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::{Read as _, Write as _};
use std::path::{Path, PathBuf};

use directories::{BaseDirs, ProjectDirs};
use sha2::{Digest, Sha256};

/// The `directories` identity every platform path derives from.
const QUALIFIER: &str = "";
const ORGANIZATION: &str = "gerwaric";
const APPLICATION: &str = "acquisition-playground";

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
}

/// The world's root before canonicalisation: `ACQ_STORE_DIR` made
/// absolute, else the platform data directory's `store` (no home
/// directory at all — a bare service account — falls back to `store`
/// under the current directory). What [`store_dir`] has always put the
/// provider directories under; the daemon and the frontends read the
/// same value.
pub fn intended_root() -> PathBuf {
    let base = match std::env::var_os("ACQ_STORE_DIR") {
        Some(d) => PathBuf::from(d),
        None => project_dirs()
            .map(|p| p.data_dir().join("store"))
            .unwrap_or_else(|| PathBuf::from("store")),
    };
    absolute(&base)
}

/// `path` made absolute against the current directory, lexically —
/// nothing is resolved or created.
fn absolute(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(path),
        Err(_) => path.to_path_buf(),
    }
}

/// The provider's store directory: `<intended root>/<provider>`, the
/// path the daemon writes under and the frontends read; not
/// canonicalised (a read needs no world). One directory per provider so
/// mock data never mixes with real.
pub fn store_dir(provider: &str) -> PathBuf {
    intended_root().join(provider)
}

/// Why a world could not be resolved: the path the environment named
/// and what went wrong with it.
#[derive(Debug, Clone)]
pub struct WorldError {
    pub intended: PathBuf,
    pub io: String,
    /// True when the root does not exist and this was an observation,
    /// which creates nothing.
    pub absent: bool,
}

impl std::fmt::Display for WorldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.absent {
            write!(
                f,
                "no world at {} (the store root does not exist; a job command creates it)",
                self.intended.display()
            )
        } else {
            write!(
                f,
                "could not resolve the store root {}: {}",
                self.intended.display(),
                self.io
            )
        }
    }
}

impl std::error::Error for WorldError {}

/// A world: a canonical store root. Constructed only by [`World::create`]
/// (the use path) or [`World::observe`] (creates nothing), so a `World`
/// in hand is a directory that exists, named the one way the OS names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    root: PathBuf,
    /// The root as a string: exact, since a root is valid UTF-8 by
    /// construction.
    name: String,
}

impl World {
    /// The use path (C83): create the intended root if it is missing,
    /// set it to mode 0700 on Unix every time (the store holds intent and
    /// `daemon.db`; a root from before step 5 is brought to that state),
    /// then canonicalise it. A failure to create or to set the mode is a
    /// refusal, never ignored.
    pub fn create() -> Result<World, WorldError> {
        let intended = intended_root();
        let failed = |io: std::io::Error| WorldError {
            intended: intended.clone(),
            io: io.to_string(),
            absent: false,
        };
        if !intended.is_dir() {
            std::fs::create_dir_all(&intended).map_err(failed)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&intended, std::fs::Permissions::from_mode(0o700))
                .map_err(failed)?;
        }
        Self::canonical(intended)
    }

    /// The observation path (C83): the world as it exists, or an error
    /// saying it does not — nothing is created.
    pub fn observe() -> Result<World, WorldError> {
        let intended = intended_root();
        if !intended.exists() {
            return Err(WorldError {
                intended,
                io: "not found".into(),
                absent: true,
            });
        }
        Self::canonical(intended)
    }

    /// A world at a root that exists, for a daemon or a test that names
    /// one directly rather than through the environment.
    pub fn at(root: &Path) -> Result<World, WorldError> {
        Self::canonical(absolute(root))
    }

    fn canonical(intended: PathBuf) -> Result<World, WorldError> {
        let root = intended.canonicalize().map_err(|e| WorldError {
            intended: intended.clone(),
            io: e.to_string(),
            absent: e.kind() == std::io::ErrorKind::NotFound,
        })?;
        if !root.is_dir() {
            return Err(WorldError {
                intended,
                io: "not a directory".into(),
                absent: false,
            });
        }
        let Some(name) = root.to_str().map(str::to_string) else {
            return Err(WorldError {
                intended,
                io: "the store root is not valid UTF-8; it names the world on the wire and must be"
                    .into(),
                absent: false,
            });
        };
        Ok(World { root, name })
    }

    /// The canonical root: what `hello` carries and a client compares.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The root as `hello` carries it: exact, since a root is valid UTF-8
    /// by construction.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Twelve hex digits of the SHA-256 of the canonical root's exact
    /// bytes: the world's short id, which names its subdirectory under the
    /// log base directory and its socket.
    pub fn id(&self) -> String {
        let digest = Sha256::digest(self.name.as_bytes());
        digest.iter().take(6).map(|b| format!("{b:02x}")).collect()
    }

    /// `<root>/<provider>`: the provider's directory in this world,
    /// where the account files, `accounts.json`, `daemon.db` and the
    /// rails state live.
    pub fn provider_dir(&self, provider: &str) -> PathBuf {
        self.root.join(provider)
    }

    /// `<root>/daemon.lock`: the world lock's file.
    pub fn lock_path(&self) -> PathBuf {
        self.root.join("daemon.lock")
    }

    /// `<root>/<provider>/rails.json`: the tripwire and refresh-failed
    /// marks, durable in the world (`rails.rs` reads and writes it).
    pub fn rails_state_path(&self, provider: &str) -> PathBuf {
        self.provider_dir(provider).join("rails.json")
    }

    /// `<log base>/<id>/<provider>`: where this world's diagnostics for
    /// one provider live.
    pub fn log_dir(&self, provider: &str) -> PathBuf {
        log_base_dir().join(self.id()).join(provider)
    }

    /// The daemon log: `<log dir>/daemon.log`.
    pub fn log_path(&self, provider: &str) -> PathBuf {
        self.log_dir(provider).join("daemon.log")
    }

    /// The default send journal: `<log dir>/sends.jsonl` (`ACQ_JOURNAL`
    /// overrides it; `0` disables it — read in `rails.rs`).
    pub fn journal_path(&self, provider: &str) -> PathBuf {
        self.log_dir(provider).join("sends.jsonl")
    }

    /// `<log base>/<id>/world`: a file naming this world's root, written
    /// by the daemon beside its logs so a person browsing the log
    /// directory can tell the hashed subdirectories apart.
    pub fn log_marker_path(&self) -> PathBuf {
        log_base_dir().join(self.id()).join("world")
    }

    /// `<id>.sock`: the socket's file name, derived from the canonical
    /// root and nothing else.
    pub fn socket_name(&self) -> String {
        format!("{}.sock", self.id())
    }

    /// The socket this world's daemon listens on and its clients connect
    /// to (C83): [`socket_name`](Self::socket_name) in the private
    /// per-user runtime directory, which must be nameable
    /// ([`Self::socket_path_under`]: valid UTF-8, under the cap) and must
    /// already exist and be this user's ([`existing_private_dir`]) —
    /// nothing is created here, so an observer that finds no runtime
    /// directory finds no daemon (`NotFound`), and a socket in a directory
    /// someone else made is never used. The daemon makes the directory
    /// first ([`app_runtime_dir`]).
    pub fn socket_path(&self) -> std::io::Result<PathBuf> {
        let runtime = runtime_dir_path();
        let path = self.socket_path_under(&runtime)?;
        existing_private_dir(&runtime)?;
        Ok(path)
    }

    /// The pure derivation: this world's socket under `runtime`, refused
    /// by name when the path is not valid UTF-8 (it names the socket in
    /// every report, on the wire as a string — a runtime directory the
    /// environment names in other bytes cannot) or longer than
    /// [`SOCKET_PATH_MAX`]. Nothing is looked at or created.
    pub fn socket_path_under(&self, runtime: &Path) -> std::io::Result<PathBuf> {
        let path = runtime.join(self.socket_name());
        if path.to_str().is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "the runtime directory {} is not valid UTF-8 (XDG_RUNTIME_DIR or TMPDIR); it names the socket in every report and must be",
                    runtime.display()
                ),
            ));
        }
        let len = path.as_os_str().len();
        if len > SOCKET_PATH_MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "the socket path {} is {len} bytes; a Unix socket path may be at most {SOCKET_PATH_MAX} (the runtime directory is too deep)",
                    path.display()
                ),
            ));
        }
        Ok(path)
    }
}

/// The longest path a Unix socket may have: `sun_path` is 104 bytes on
/// macOS including its terminator (108 on Linux), so 103 is the bound
/// that holds on both. [`World::socket_path`] refuses a longer one by
/// name instead of letting `bind` fail with `ENAMETOOLONG`.
pub const SOCKET_PATH_MAX: usize = 103;

/// Where this application's private per-user runtime directory is,
/// computed and not touched: `$XDG_RUNTIME_DIR/acq` where the platform
/// provides that directory (Linux; it is per user by the XDG spec), else
/// `<temp dir>/acq-<uid>` — the temp directory is shared on Linux
/// (`/tmp`) and per user on macOS (`$TMPDIR`), and the uid in the name
/// keeps the fallback per user either way.
fn runtime_dir_path() -> PathBuf {
    match BaseDirs::new().and_then(|b| b.runtime_dir().map(Path::to_path_buf)) {
        Some(runtime) => runtime.join("acq"),
        None => std::env::temp_dir().join(format!("acq-{}", current_uid())),
    }
}

/// This application's private per-user runtime directory
/// (`runtime_dir_path`), created on demand and verified on every use.
/// Holds the real-mode lock and every world's socket. [`private_dir`]
/// does the creating and the checking; the daemon calls this before it
/// binds, a client never creates it ([`World::socket_path`]).
pub fn app_runtime_dir() -> std::io::Result<PathBuf> {
    let runtime = runtime_dir_path();
    // Nameable before it is made: a runtime directory in other bytes
    // could not name a socket in any report (review 2026-09-11).
    if runtime.to_str().is_none() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "the runtime directory {} is not valid UTF-8 (XDG_RUNTIME_DIR or TMPDIR); it names the socket in every report and must be",
                runtime.display()
            ),
        ));
    }
    private_dir(&runtime)
}

/// The calling user's uid (Unix); 0 elsewhere, where the checks below
/// are not made.
fn current_uid() -> u32 {
    #[cfg(unix)]
    {
        // SAFETY: getuid has no preconditions and cannot fail.
        unsafe { libc::getuid() }
    }
    #[cfg(not(unix))]
    {
        0
    }
}

/// A directory private to this user at `path`: created with mode 0700
/// in the creating call if missing (its parent must exist: the runtime
/// or temp directory); refused if it is a symlink or not a directory,
/// if another user owns it, or if its mode is not exactly 0700 and
/// cannot be made so — the pre-creation attack on a shared temp
/// directory, where a path the attacker made first, or made loose
/// between creation and a later chmod, would be used as ours (review
/// 2026-09-11, twice). On a non-Unix platform only the existence check
/// is made.
pub fn private_dir(path: &Path) -> std::io::Result<PathBuf> {
    use std::io::{Error, ErrorKind};
    if let Err(e) = std::fs::symlink_metadata(path)
        && e.kind() == ErrorKind::NotFound
    {
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(path).map_err(|e| {
            Error::new(
                e.kind(),
                format!(
                    "could not create the private directory {}: {e}",
                    path.display()
                ),
            )
        })?;
    }
    existing_private_dir(path)
}

/// [`private_dir`]'s checks alone, on a directory that must already
/// exist: a missing one is `NotFound`, nothing is created. What a client
/// uses before it connects to a socket in the runtime directory.
pub fn existing_private_dir(path: &Path) -> std::io::Result<PathBuf> {
    use std::io::{Error, ErrorKind};
    match std::fs::symlink_metadata(path) {
        Err(e) => return Err(e),
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!(
                    "{} is a symlink; the runtime directory must be a plain directory this user made",
                    path.display()
                ),
            ));
        }
        Ok(meta) if !meta.is_dir() => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("{} exists and is not a directory", path.display()),
            ));
        }
        Ok(_) => {}
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let meta = std::fs::symlink_metadata(path)?;
        let uid = current_uid();
        if meta.uid() != uid {
            return Err(Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "{} is owned by uid {}, not this user ({uid}); refusing to use it",
                    path.display(),
                    meta.uid()
                ),
            ));
        }
        if meta.mode() & 0o7777 != 0o700 {
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
            let again = std::fs::symlink_metadata(path)?;
            if again.mode() & 0o7777 != 0o700 {
                return Err(Error::new(
                    ErrorKind::PermissionDenied,
                    format!(
                        "{} is mode {:04o}, not 0700, and could not be made so; refusing to use it",
                        path.display(),
                        again.mode() & 0o7777
                    ),
                ));
            }
        }
    }
    Ok(path.to_path_buf())
}

/// `<runtime>/ggg.lock`: the real-mode lock's file, per OS user,
/// whatever the root (C31, C83).
pub fn real_mode_lock_path() -> std::io::Result<PathBuf> {
    Ok(app_runtime_dir()?.join("ggg.lock"))
}

/// Where the daemon log and the default journal live: `ACQ_LOG_DIR`
/// (made absolute), else the platform's log directory —
/// `~/Library/Logs/acquisition-playground` on macOS, the XDG state
/// directory's `log` on Linux, the local data directory's `logs`
/// elsewhere; `logs` under the current directory with no home at all.
/// One subdirectory per world and provider under it ([`World::log_dir`]).
pub fn log_base_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("ACQ_LOG_DIR") {
        return absolute(Path::new(&d));
    }
    if cfg!(target_os = "macos")
        && let Some(home) = BaseDirs::new().map(|b| b.home_dir().to_path_buf())
    {
        return home.join("Library").join("Logs").join(APPLICATION);
    }
    match project_dirs() {
        Some(p) => match p.state_dir() {
            Some(state) => state.join("log"),
            None => p.data_local_dir().join("logs"),
        },
        None => PathBuf::from("logs"),
    }
}

/// An exclusive advisory lock on a file, held while this value lives; the
/// holder's pid is written into the file for the refusal that names it.
#[derive(Debug)]
pub struct Lock {
    _file: File,
    path: PathBuf,
}

/// The lock is held by another process, or could not be taken.
#[derive(Debug, Clone)]
pub struct LockError {
    pub path: PathBuf,
    /// The pid the holder wrote, when the file could be read and parsed.
    pub holder: Option<u32>,
    /// An I/O failure other than the lock being held.
    pub io: Option<String>,
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.io, self.holder) {
            (Some(io), _) => write!(f, "could not take the lock {}: {io}", self.path.display()),
            (None, Some(pid)) => write!(
                f,
                "{} is held by another process (pid {pid})",
                self.path.display()
            ),
            (None, None) => write!(
                f,
                "{} is held by another process (its pid is not recorded yet)",
                self.path.display()
            ),
        }
    }
}

impl std::error::Error for LockError {}

impl Lock {
    /// Take the lock at `path` (created if missing), or say who holds it.
    /// Never blocks.
    pub fn acquire(path: &Path) -> Result<Lock, LockError> {
        let io_error = |e: std::io::Error| LockError {
            path: path.to_path_buf(),
            holder: None,
            io: Some(e.to_string()),
        };
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .map_err(io_error)?;
        match file.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => {
                let mut text = String::new();
                let _ = file.read_to_string(&mut text);
                return Err(LockError {
                    path: path.to_path_buf(),
                    holder: text.trim().parse().ok(),
                    io: None,
                });
            }
            Err(TryLockError::Error(e)) => return Err(io_error(e)),
        }
        // Held: record who, for the next contender's refusal.
        let _ = file.set_len(0);
        let _ = writeln!(file, "{}", std::process::id());
        let _ = file.flush();
        Ok(Lock {
            _file: file,
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "acq-world-{tag}-{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// C83: a world is a canonical root — one id for two spellings of
    /// one directory — with its provider directories, lock file and
    /// rails state under it, and its diagnostics keyed by that id under
    /// the log base.
    #[test]
    fn c83_a_world_is_a_canonical_root_and_its_paths_derive_from_it() {
        let base = scratch("paths");
        std::fs::create_dir_all(base.join("real")).unwrap();
        let world = World::at(&base.join("real")).unwrap();
        assert!(world.root().is_absolute());
        // A dotted spelling of the same directory is the same world.
        let dotted = World::at(&base.join("real").join(".").join("..").join("real")).unwrap();
        assert_eq!(world, dotted);
        assert_eq!(world.id(), dotted.id());
        assert_eq!(world.id().len(), 12);
        assert!(world.id().bytes().all(|b| b.is_ascii_hexdigit()));
        assert_eq!(world.provider_dir("mock"), world.root().join("mock"));
        assert_eq!(world.lock_path(), world.root().join("daemon.lock"));
        assert_eq!(
            world.rails_state_path("ggg"),
            world.root().join("ggg").join("rails.json")
        );
        assert_eq!(
            world.log_path("mock"),
            log_base_dir()
                .join(world.id())
                .join("mock")
                .join("daemon.log")
        );
        assert_eq!(
            world.journal_path("mock"),
            log_base_dir()
                .join(world.id())
                .join("mock")
                .join("sends.jsonl")
        );
        // Another directory is another world.
        std::fs::create_dir_all(base.join("other")).unwrap();
        let other = World::at(&base.join("other")).unwrap();
        assert_ne!(world, other);
        assert_ne!(world.id(), other.id());
        // A missing root is an error naming it, never a world.
        let err = World::at(&base.join("missing")).unwrap_err();
        assert!(err.absent, "{err}");
        assert!(err.to_string().contains("missing"), "{err}");
        let _ = std::fs::remove_dir_all(&base);
    }

    /// C83: the world lock is exclusive and names its holder; it is
    /// released when the `Lock` is dropped, however that happens.
    #[test]
    fn c83_the_lock_is_exclusive_and_names_its_holder() {
        let base = scratch("lock");
        std::fs::create_dir_all(&base).unwrap();
        let path = base.join("daemon.lock");
        let held = Lock::acquire(&path).unwrap();
        assert_eq!(held.path(), path.as_path());
        assert_eq!(
            std::fs::read_to_string(&path).unwrap().trim(),
            std::process::id().to_string(),
            "the holder wrote its pid"
        );
        // A second acquisition from this process is a second open file
        // description: flock refuses it, and the refusal names the holder.
        let err = Lock::acquire(&path).unwrap_err();
        assert_eq!(err.holder, Some(std::process::id()), "{err}");
        assert!(err.io.is_none(), "{err}");
        assert!(
            err.to_string().contains("held by another process")
                && err.to_string().contains(&std::process::id().to_string()),
            "{err}"
        );
        drop(held);
        let again = Lock::acquire(&path).expect("released on drop");
        drop(again);
        let _ = std::fs::remove_dir_all(&base);
    }

    /// C83: the private runtime directory is this user's: made 0700,
    /// brought back to 0700 when it drifted, refused when it is a symlink
    /// or a file — and a non-UTF-8 root is refused as a world.
    #[cfg(unix)]
    #[test]
    fn c83_the_private_dir_is_this_users_and_a_root_is_utf8() {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let base = scratch("private");
        std::fs::create_dir_all(&base).unwrap();
        let fresh = private_dir(&base.join("fresh")).unwrap();
        assert_eq!(std::fs::metadata(&fresh).unwrap().mode() & 0o777, 0o700);
        for (name, drifted) in [("loose", 0o755), ("tight", 0o500), ("shut", 0o000)] {
            let dir = base.join(name);
            std::fs::create_dir(&dir).unwrap();
            std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(drifted)).unwrap();
            private_dir(&dir).unwrap();
            assert_eq!(
                std::fs::metadata(&dir).unwrap().mode() & 0o7777,
                0o700,
                "{name}: restored to exactly 0700"
            );
        }
        let err = private_dir(&base.join("absent-parent").join("acq")).unwrap_err();
        assert!(err.to_string().contains("could not create"), "{err}");
        std::os::unix::fs::symlink(&fresh, base.join("link")).unwrap();
        let err = private_dir(&base.join("link")).unwrap_err();
        assert!(err.to_string().contains("symlink"), "{err}");
        std::fs::write(base.join("file"), b"").unwrap();
        let err = private_dir(&base.join("file")).unwrap_err();
        assert!(err.to_string().contains("not a directory"), "{err}");
        let runtime = app_runtime_dir().unwrap();
        assert_eq!(std::fs::metadata(&runtime).unwrap().mode() & 0o077, 0);
        assert_eq!(
            real_mode_lock_path().unwrap().parent(),
            Some(runtime.as_path())
        );

        use std::os::unix::ffi::OsStrExt;
        // APFS refuses an invalid byte sequence in a name (EILSEQ), so on
        // macOS the filesystem holds the property; where a directory can
        // carry one, the world refuses it.
        let odd = base.join(std::ffi::OsStr::from_bytes(b"r\xff"));
        if std::fs::create_dir(&odd).is_ok() {
            let err = World::at(&odd).unwrap_err();
            assert!(err.to_string().contains("UTF-8"), "{err}");
        }
        let _ = std::fs::remove_dir_all(&base);
    }

    /// C83: the socket derives from the world into the runtime directory
    /// and from nothing else — two spellings of one root reach one
    /// socket, another root another. That no knob names it is held by
    /// `tools/docs-check.sh`: a knob read anywhere in the crates needs a
    /// README row, and the socket has none. The derivation's refusals —
    /// over the cap, not UTF-8 — are pinned on the pure helper with
    /// runtime directories this test names, since the real one is the
    /// environment's (review 2026-09-11).
    #[test]
    fn c83_the_socket_derives_from_the_world_into_the_runtime_directory() {
        let base = scratch("socket");
        std::fs::create_dir_all(base.join("real")).unwrap();
        std::fs::create_dir_all(base.join("other")).unwrap();
        // The derivation under a runtime directory this test names: the
        // world's id names the socket, two spellings of one root reach
        // one socket, another root another, all in that directory.
        let named = Path::new("/run/user/1000/acq");
        let world = World::at(&base.join("real")).unwrap();
        let socket = world.socket_path_under(named).unwrap();
        assert_eq!(socket, named.join(world.socket_name()));
        assert_eq!(world.socket_name(), format!("{}.sock", world.id()));
        let dotted = World::at(&base.join("real").join(".").join("..").join("real")).unwrap();
        assert_eq!(dotted.socket_path_under(named).unwrap(), socket);
        let other = World::at(&base.join("other")).unwrap();
        assert_ne!(other.socket_path_under(named).unwrap(), socket);
        assert_eq!(
            other.socket_path_under(named).unwrap().parent(),
            socket.parent()
        );
        // The environment's runtime directory, whatever it is: the real
        // door agrees with the pure derivation over it — the same path
        // when that derivation accepts it, and a refusal of the same
        // kind when it does not (a deep or non-UTF-8 runtime directory
        // is a valid environment; review 2026-09-11).
        let runtime = runtime_dir_path();
        match world.socket_path_under(&runtime) {
            Ok(expected) => {
                app_runtime_dir().unwrap();
                assert_eq!(world.socket_path().unwrap(), expected);
                assert_eq!(dotted.socket_path().unwrap(), expected);
            }
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::InvalidInput, "{e}");
                let real = world.socket_path().unwrap_err();
                assert_eq!(real.kind(), std::io::ErrorKind::InvalidInput, "{real}");
                assert_eq!(real.to_string(), e.to_string());
            }
        }
        let deep = PathBuf::from("/").join("x".repeat(SOCKET_PATH_MAX));
        let err = world.socket_path_under(&deep).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(
            err.to_string().contains("at most 103") && err.to_string().contains("bytes"),
            "{err}"
        );
        // Exactly at the cap passes; one over is refused: the bound is the
        // path's byte length, terminator excluded.
        let name_len = world.socket_name().len();
        // "/" + the y's + "/" + the name: two separators.
        let at_cap = PathBuf::from("/".to_string() + &"y".repeat(SOCKET_PATH_MAX - name_len - 2));
        assert_eq!(
            world.socket_path_under(&at_cap).unwrap().as_os_str().len(),
            SOCKET_PATH_MAX
        );
        let over = PathBuf::from("/".to_string() + &"y".repeat(SOCKET_PATH_MAX - name_len - 1));
        assert!(world.socket_path_under(&over).is_err());
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let odd = PathBuf::from(std::ffi::OsStr::from_bytes(b"/run/user/1000/acq-\xff"));
            let err = world.socket_path_under(&odd).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
            assert!(
                err.to_string().contains("not valid UTF-8")
                    && err.to_string().contains("XDG_RUNTIME_DIR"),
                "{err}"
            );
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}
