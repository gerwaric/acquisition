//! Live-test safety rails (`LIVE-TESTING.md`, package L0).
//!
//! The rails never add a send and never delay a halt; they only refuse or
//! record. `ChokePoint` consults them immediately before acquiring any gate
//! permit and reports every completed exchange to them afterward.
//!
//! - **Tripwire** (`ACQ_TRIPWIRE=1`, ladder-only): the first landed 429 on
//!   any route — HEAD and token included — or any 401/403/503 halts every later
//!   send until an explicit `reset_tripwire`. Persisted per provider so a
//!   respawned daemon stays tripped — in the world since the daemon
//!   split's step 5 (`<root>/<provider>/rails.json`, C83), where a
//!   reboot cannot clear it; a state file from before, beside the socket
//!   in the temp directory, is moved in once by [`migrate_legacy_state`]
//!   so no trip is lost.
//! - **Dead-token stop** (with the tripwire): a 4xx other than 429 on a
//!   `refresh_token` grant marks the session refresh-failed; later refreshes
//!   fail fast without sending until login or logout. Persisted with the
//!   tripwire.
//! - **Send ceiling** (`ACQ_MAX_SENDS=<n>`, ladder-only): after `n` real
//!   sends in this daemon lifetime the daemon halts with cause `ceiling`.
//!   Per-lifetime and never persisted.
//! - **Send journal** (`ACQ_JOURNAL=<path>`, permanent): one JSON line per
//!   actual send, flushed per line, never containing a token or body.
//!   The default path is the world's `sends.jsonl` under the log
//!   directory (`acquisition_store::world`; `acq daemon status` prints
//!   it), bounded by the daemon at its start like the log; `0`
//!   disables; the directory is created on demand, and a journal that
//!   cannot be opened is reported in `daemon status`, never silently
//!   dropped. Each daemon lifetime opens with
//!   `{"event":"open","pid","contract","daemon","clock"}` — the two
//!   values that identify which code sent (C84): the shared-contract
//!   revision the daemon was compiled against and the SHA-256 of the
//!   `acqd` file it ran from (`null` from the in-process harness, which
//!   runs from no executable of its own; journals before 2026-09-10
//!   carry the runtime revision as `runtime`, and before 2026-09-09 a git
//!   commit as `build`), and whether time was the system's or a test's
//!   manual clock. A send line carries method, `route`, status, `counted`,
//!   `wait_ms` and every `X-Rate-Limit-*` header. `route` is the
//!   limiter's endpoint key — `stash@Alice#1234` for a send on an account,
//!   `oauth-token` for the account-blind token endpoint — so the journal
//!   names the account of every send; a realm other than pc suffixes it
//!   (`stash-list/xbox@Alice#1234`). A non-2xx line adds a `headers`
//!   object (the `X-Rate-Limit-*`, `Retry-After`, `cf-*`, `content-type`,
//!   `server`, `date` headers — for a failed HEAD probe, the whole of the
//!   evidence), and a 403/503 line carries `shape`: `cloudflare` (N3/N28
//!   page markers), `origin` (an openresty/nginx error page that passed
//!   through Cloudflare — rung 10's 503, N35), or `unclassified`; all three
//!   are equally never retried.
//! - Misunderstood values (`ACQ_TRIPWIRE=maybe`, `ACQ_MAX_SENDS=ten`) are
//!   logged at startup as `RAILS CONFIG` errors and the rail stays off; a
//!   persisted trip is honored only by a daemon started with the tripwire.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/network.md` for this
//! area, `CONTEXT.md` for the cross-cutting ones (`C<n>`); what follows is each
//! entry's full text as recorded there, moved here on 2026-09-02 because
//! the mechanism it describes is this module's. The registry is current;
//! this is the mechanism as decided, kept beside the code that implements it.
//!
//! ## C25 — A rails halt leaves queued network jobs waiting; nothing fails for lack of a send.
//!
//! **A rails halt leaves queued network jobs waiting; nothing fails for lack of a send.** The dispatcher does not pick a network job while halted, a job that finds the halt after being picked gives its key back, and `reset-tripwire` wakes the queue. A halted daemon with only waiting jobs counts as idle and exits — the queue is on disk, and its successor (started with the tripwire, which honors the persisted trip) holds it until the reset. Rationale: rung 10 (2026-08-24) failed 82 never-sent children on a 503 and the rerun refetched all 322 tabs; the two reasons for failing them — results died with the daemon, and a daemon with waiting jobs never idled out — are both gone with persistence. Caveat for `LIVE-TESTING.md`: the ceiling is per lifetime and not persisted, so a queue halted by `ACQ_MAX_SENDS` resumes under the next daemon's fresh ceiling — `acq jobs` and `acq cancel` before respawning. Decided 2026-08-30.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use acquisition_protocol::status::RailsStatus;

use crate::ratelimit::{Clock, SystemClock};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// How the rails are configured for one daemon lifetime.
#[derive(Clone, Debug, Default)]
pub struct RailsConfig {
    pub tripwire: bool,
    pub max_sends: Option<u64>,
    pub journal_path: Option<PathBuf>,
    /// Where the tripwire and refresh-failed marks persist. `None` keeps
    /// them in memory only (tests).
    pub state_path: Option<PathBuf>,
    /// Environment values that were not understood, for the startup log.
    /// A rail the operator believed armed must not fail open silently.
    pub warnings: Vec<String>,
    /// What the journal header names as `daemon`: the SHA-256 of the
    /// executable this daemon runs from (C84), set by `daemon::run` after
    /// it identifies itself. `None` in the in-process harness, whose
    /// daemons have no executable of their own.
    pub daemon_artifact: Option<String>,
}

impl RailsConfig {
    /// Read the environment. `state_path` is where the tripwire and
    /// refresh-failed marks persist — the world's `rails.json` for the
    /// provider (C83) — and `default_journal` where the journal goes
    /// when `ACQ_JOURNAL` says nothing.
    pub fn from_env(state_path: &Path, default_journal: &Path) -> RailsConfig {
        let mut warnings = Vec::new();
        let tripwire = match std::env::var("ACQ_TRIPWIRE") {
            Ok(v) if matches!(v.trim(), "1" | "true" | "yes" | "on") => true,
            Ok(v) if matches!(v.trim(), "" | "0" | "false" | "no" | "off") => false,
            Ok(v) => {
                warnings.push(format!(
                    "ACQ_TRIPWIRE={v:?} not understood; tripwire is OFF"
                ));
                false
            }
            Err(_) => false,
        };
        let max_sends = match std::env::var("ACQ_MAX_SENDS") {
            Ok(v) => match v.trim().parse::<u64>() {
                Ok(n) => Some(n),
                Err(_) => {
                    warnings.push(format!("ACQ_MAX_SENDS={v:?} is not a number; no ceiling"));
                    None
                }
            },
            Err(_) => None,
        };
        let journal_path = match std::env::var("ACQ_JOURNAL") {
            Ok(p) if p.trim().is_empty() || p.trim() == "0" => None,
            Ok(p) => Some(PathBuf::from(p)),
            Err(_) => Some(default_journal.to_path_buf()),
        };
        let state_path = Some(state_path.to_path_buf());
        RailsConfig {
            tripwire,
            max_sends,
            journal_path,
            state_path,
            warnings,
            daemon_artifact: None,
        }
    }
}

/// What persists across daemon restarts. Only violation-class trips and
/// the refresh-failed mark; the ceiling is per lifetime by decision.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct Persisted {
    #[serde(default)]
    tripped: Option<String>,
    /// Per account (username → cause). The pre-multi-account file had one
    /// `refresh_failed` string; it is ignored on load (one re-login).
    #[serde(default)]
    refresh_failed_by_account: HashMap<String, String>,
}

/// Why the rails state could not be brought into the world. The daemon
/// refuses to start on any of these (fail closed: a persisted trip must
/// never be absent from a lifetime, review 2026-09-11); the message names
/// the remedy — with no daemon running, `acq daemon reset-tripwire`
/// removes both files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationError(pub String);

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} — the daemon does not start over rails state it cannot carry; with no daemon running, `acq daemon reset-tripwire` clears both paths (a file or an empty directory; anything else it names for removal by hand)",
            self.0
        )
    }
}

impl std::error::Error for MigrationError {}

/// The one-time move of the rails state into the world (C83; the daemon
/// split's step 5). Before it, the state sat beside the socket in the
/// temp directory, which macOS clears at reboot. Called by the daemon
/// after it holds the world lock and before the rails read `current`:
/// a legacy file with no current one moves whole; both present are
/// merged so no trip is lost — a trip in either is a trip, the
/// refresh-failed marks are the union, the current file's cause wins a
/// conflict. The world's file is replaced atomically (written beside it,
/// then renamed), so it is either the previous state or the merged one,
/// never a partial write; the legacy file is removed only after that
/// rename. Every failure is a [`MigrationError`] and the daemon refuses
/// to start on it (review 2026-09-11: a failure that was merely logged
/// let a lifetime run without its trip): a legacy or world state that
/// cannot be read or parsed, a directory that cannot be made, a write,
/// a rename, or a removal that fails. The one state a failure can leave
/// behind — the world merged, the legacy file still present — is
/// idempotent: the next start merges the same content again. Returns
/// what happened, for the daemon log; `None` when there was no legacy
/// file.
pub fn migrate_legacy_state(
    legacy: &Path,
    current: &Path,
) -> Result<Option<String>, MigrationError> {
    let fail = |what: String| MigrationError(what);
    let text = match std::fs::read_to_string(legacy) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(fail(format!(
                "rails: the legacy state {} could not be read ({e})",
                legacy.display()
            )));
        }
    };
    let old = serde_json::from_str::<Persisted>(&text).map_err(|e| {
        fail(format!(
            "rails: the legacy state {} is not readable ({e})",
            legacy.display()
        ))
    })?;
    let merged = match std::fs::read_to_string(current) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => old,
        Err(e) => {
            return Err(fail(format!(
                "rails: the world's state {} could not be read ({e}); the legacy state {} is left in place",
                current.display(),
                legacy.display()
            )));
        }
        Ok(text) => {
            let mut now = serde_json::from_str::<Persisted>(&text).map_err(|e| {
                fail(format!(
                    "rails: the world's state {} is not readable ({e}) and is not overwritten; the legacy state {} is left in place",
                    current.display(),
                    legacy.display()
                ))
            })?;
            if now.tripped.is_none() {
                now.tripped = old.tripped;
            }
            for (account, cause) in old.refresh_failed_by_account {
                now.refresh_failed_by_account
                    .entry(account)
                    .or_insert(cause);
            }
            now
        }
    };
    if let Some(dir) = current.parent() {
        std::fs::create_dir_all(dir).map_err(|e| {
            fail(format!(
                "rails: could not create {} ({e}); the legacy state {} is left in place",
                dir.display(),
                legacy.display()
            ))
        })?;
    }
    let text = serde_json::to_string(&merged).map_err(|e| {
        fail(format!(
            "rails: the merged state could not be serialised ({e})"
        ))
    })?;
    // Atomic replacement: the world's file is the old state or the merged
    // one, never a truncated one.
    let tmp = current.with_extension("json.tmp");
    std::fs::write(&tmp, text).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        fail(format!(
            "rails: could not write {} ({e}); the world's state and the legacy state {} are unchanged",
            tmp.display(),
            legacy.display()
        ))
    })?;
    std::fs::rename(&tmp, current).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        fail(format!(
            "rails: could not replace {} ({e}); the world's state and the legacy state {} are unchanged",
            current.display(),
            legacy.display()
        ))
    })?;
    std::fs::remove_file(legacy).map_err(|e| {
        fail(format!(
            "rails: the world at {} now holds the merged state, but the legacy file {} could not be removed ({e}); it would be merged again at the next start",
            current.display(),
            legacy.display()
        ))
    })?;
    Ok(Some(format!(
        "rails: state moved from {} into the world at {}{}",
        legacy.display(),
        current.display(),
        match &merged.tripped {
            Some(cause) => format!(" (tripped: {cause})"),
            None => String::new(),
        }
    )))
}

/// The persisted state at `path`, read strictly: `None` when there is no
/// file, an error when there is one that cannot be read or parsed. The
/// daemon calls this before the rails are built, and refuses to start on
/// an error (fail closed, review 2026-09-11); [`Rails::with_config`]
/// itself reads leniently, for the harness and the tests.
pub fn verify_state(path: &Path) -> Result<Option<()>, MigrationError> {
    match std::fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(MigrationError(format!(
            "rails: the world's state {} could not be read ({e})",
            path.display()
        ))),
        Ok(text) => serde_json::from_str::<Persisted>(&text)
            .map(|_| Some(()))
            .map_err(|e| {
                MigrationError(format!(
                    "rails: the world's state {} is not readable ({e})",
                    path.display()
                ))
            }),
    }
}

#[derive(Debug, Default)]
struct State {
    /// Violation/Cloudflare trip cause (persisted).
    tripped: Option<String>,
    /// Ceiling trip cause (this lifetime only).
    ceiling_tripped: Option<String>,
    /// Accounts whose refresh grant the provider rejected (persisted).
    refresh_failed: HashMap<String, String>,
    sends: u64,
    /// A trip nobody has logged yet; drained by the daemon's `announce_trip`.
    unannounced: Option<String>,
}

/// What a 403/503 body looked like. Both shapes are treated the same way
/// today — never retried (invariant 3) and a tripwire trip on the ladder —
/// but the evidence is recorded separately so a future retry decision has
/// data to cite. Rung 10 (2026-08-24) saw an origin 503 (openresty page,
/// no rate headers — ground truth N35) that the code labelled "possibly a
/// Cloudflare block".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockShape {
    /// Cloudflare's own error page: ground truth N3 (error 1015) and N28
    /// (a 403 challenge page); "Ray ID" appears on every Cloudflare page.
    Cloudflare,
    /// An origin error page that reached us through Cloudflare unchanged
    /// (N35).
    Origin,
    /// Nothing recognisable (empty body, JSON, transport failure reading it).
    Unclassified,
}

impl BlockShape {
    pub fn of(body: &str) -> BlockShape {
        let lower = body.to_ascii_lowercase();
        if lower.contains("cloudflare") || lower.contains("ray id") || lower.contains("error 1015")
        {
            BlockShape::Cloudflare
        } else if lower.contains("openresty") || lower.contains("nginx") {
            BlockShape::Origin
        } else {
            BlockShape::Unclassified
        }
    }

    /// The phrase that goes in error text, the trip cause, and the journal.
    pub fn describe(self) -> &'static str {
        match self {
            BlockShape::Cloudflare => "Cloudflare-shaped block (N3/N28)",
            BlockShape::Origin => "origin error page, not Cloudflare-shaped (N35)",
            BlockShape::Unclassified => "unclassified body, possibly a Cloudflare block",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BlockShape::Cloudflare => "cloudflare",
            BlockShape::Origin => "origin",
            BlockShape::Unclassified => "unclassified",
        }
    }
}

/// One completed exchange, as the choke point reports it. Never carries a
/// token, an Authorization header, or a body.
pub struct SendReport<'a> {
    pub method: &'a str,
    pub route: &'a str,
    pub url_path: &'a str,
    pub status: Option<u16>,
    pub error: Option<&'a str>,
    pub ok: bool,
    pub counted: bool,
    /// The `X-Rate-Limit-*` and `Retry-After` snapshot (an object), or Null.
    pub rate: &'a Value,
    /// The response headers of a non-2xx (`response_headers_snapshot`), or
    /// Null. A HEAD has no body, so for a failed probe this is the evidence.
    pub headers: &'a Value,
    /// For a 403/503, what the body looked like; `None` for every other status.
    pub shape: Option<BlockShape>,
    /// How long the send was held before it reached the transport: from
    /// the moment it was ready (a job picked by the dispatcher, a token
    /// refresh entering the choke) to dispatch, on the monotonic clock.
    /// Pacing made observable without recording reasons.
    pub wait: Duration,
}

pub struct Rails {
    config: RailsConfig,
    state: Mutex<State>,
    journal: Mutex<Option<File>>,
    /// Why the journal could not be opened, if it could not.
    journal_error: Option<String>,
    /// Stamps journal lines. Under a manual clock the timestamps are the
    /// scenario's, not the machine's — which is what makes "finished within
    /// N virtual seconds" a real assertion instead of one that cannot fail.
    clock: Arc<dyn Clock>,
}

impl Rails {
    /// Everything off and nothing persisted: the default for unit tests and
    /// for `ChokePoint::new()` callers that predate the rails.
    pub fn disabled() -> Rails {
        Rails::with_config(RailsConfig::default())
    }

    pub fn with_config(config: RailsConfig) -> Rails {
        Rails::with_config_and_clock(config, Arc::new(SystemClock))
    }

    pub(crate) fn with_config_and_clock(config: RailsConfig, clock: Arc<dyn Clock>) -> Rails {
        let mut state = State::default();
        if let Some(path) = &config.state_path
            && let Ok(text) = std::fs::read_to_string(path)
            && let Ok(persisted) = serde_json::from_str::<Persisted>(&text)
        {
            // A persisted trip belongs to the tripwire: a daemon started
            // without it (the post-baseline default) neither honors nor
            // deletes it. The refresh-failed mark is product behavior
            // (CONTEXT.md: a rejected grant is terminal) and is honored by
            // every daemon.
            if config.tripwire {
                state.tripped = persisted.tripped;
            }
            state.refresh_failed = persisted.refresh_failed_by_account;
        }
        let (journal, journal_error) = match &config.journal_path {
            None => (None, None),
            // The journal's directory is created on demand: a run directory
            // that does not exist yet must not cost the run its evidence.
            Some(path) => match path
                .parent()
                .filter(|dir| !dir.as_os_str().is_empty())
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| OpenOptions::new().create(true).append(true).open(path))
            {
                Ok(file) => (Some(file), None),
                Err(error) => (
                    None,
                    Some(format!(
                        "journal {} could not be opened: {error}",
                        path.display()
                    )),
                ),
            },
        };
        let rails = Rails {
            config,
            state: Mutex::new(state),
            journal: Mutex::new(journal),
            journal_error,
            clock,
        };
        rails.journal_header();
        rails
    }

    /// One line per daemon lifetime, before any send: which process, which
    /// code (the contract it was compiled against and the executable it
    /// ran from, C84), and which clock. The per-send lines are unchanged.
    /// A reader that finds `"clock":"manual"` is looking at a scenario,
    /// not at GGG; one that finds a `daemon` that is not the `acqd` its
    /// run record hashed is looking at the rung-8 mistake.
    fn journal_header(&self) {
        let mut guard = self.journal.lock().unwrap();
        let Some(file) = guard.as_mut() else { return };
        let line = json!({
            "event": "open",
            "ts": iso_utc(self.clock.wall()),
            "pid": std::process::id(),
            "contract": acquisition_protocol::CONTRACT_REVISION,
            "daemon": self.config.daemon_artifact,
            "clock": self.clock.kind(),
        });
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }

    /// Startup diagnostics the daemon should log: misunderstood environment
    /// values and a journal that is not being written.
    pub fn startup_warnings(&self) -> Vec<String> {
        let mut warnings = self.config.warnings.clone();
        warnings.extend(self.journal_error.clone());
        warnings
    }

    pub fn config(&self) -> &RailsConfig {
        &self.config
    }

    /// Why a send must be refused right now, if it must. Consulted before
    /// any gate permit is acquired.
    pub fn halted(&self) -> Option<String> {
        let s = self.state.lock().unwrap();
        s.tripped
            .clone()
            .or_else(|| s.ceiling_tripped.clone())
            .or_else(|| {
                (self.config.max_sends == Some(0)).then(|| "ceiling: 0 sends allowed".to_string())
            })
            .map(|cause| format!("halted by live-test rails: {cause}"))
    }

    /// Journal the exchange, count it toward the ceiling, and trip on a
    /// violation or Cloudflare-shaped status when the tripwire is enabled.
    /// Returns the trip cause if this report tripped anything.
    pub fn record(&self, report: &SendReport<'_>) -> Option<String> {
        self.journal_line(report);
        let mut s = self.state.lock().unwrap();
        s.sends = s.sends.saturating_add(1);
        let mut newly = None;
        if self.config.tripwire && s.tripped.is_none() {
            let cause = match report.status {
                Some(429) => Some(format!(
                    "429 on {} {} (rate headers {})",
                    report.method, report.url_path, report.rate
                )),
                // A 403 that names its own cause (`WWW-Authenticate` with
                // `insufficient_scope`, an invalid token…) is an auth
                // error, not a block; say what the server said.
                Some(403) if report.headers.get("www-authenticate").is_some() => Some(format!(
                    "403 on {} {} — auth error: {}",
                    report.method, report.url_path, report.headers["www-authenticate"]
                )),
                // A HEAD has no body to classify; say so rather than
                // reporting an "unclassified body", and carry the headers.
                Some(status @ (403 | 503)) if report.method == "HEAD" => Some(format!(
                    "{status} on HEAD {} — no body to classify (HEAD); response headers {}",
                    report.url_path, report.headers
                )),
                Some(status @ (403 | 503)) => Some(format!(
                    "{status} on {} {} — {}{}",
                    report.method,
                    report.url_path,
                    report.shape.unwrap_or(BlockShape::Unclassified).describe(),
                    if report.headers.is_null() {
                        String::new()
                    } else {
                        format!("; response headers {}", report.headers)
                    }
                )),
                // An unauthorized request repeating on a timer (a token the
                // daemon wrongly believes valid) is not a violation, but it
                // is traffic GGG should never see twice.
                Some(401) => Some(format!(
                    "401 on {} {} — token rejected",
                    report.method, report.url_path
                )),
                _ => None,
            };
            if let Some(cause) = cause {
                s.tripped = Some(cause.clone());
                newly = Some(cause);
                self.persist_locked(&s);
            }
        }
        if let Some(max) = self.config.max_sends
            && s.sends >= max
            && s.ceiling_tripped.is_none()
        {
            let cause = format!(
                "ceiling: {} of {max} sends used this daemon lifetime",
                s.sends
            );
            s.ceiling_tripped = Some(cause.clone());
            if newly.is_none() {
                newly = Some(cause);
            }
        }
        if let Some(cause) = &newly
            && s.unannounced.is_none()
        {
            s.unannounced = Some(cause.clone());
        }
        newly
    }

    /// The most recent trip that has not been logged yet, once.
    pub fn take_unannounced_trip(&self) -> Option<String> {
        self.state.lock().unwrap().unannounced.take()
    }

    /// Clear both trip kinds. The refresh-failed mark is not a trip; it
    /// clears on login or logout.
    pub fn reset_tripwire(&self) {
        let mut s = self.state.lock().unwrap();
        s.tripped = None;
        s.ceiling_tripped = None;
        self.persist_locked(&s);
    }

    /// Active only with the tripwire. Returns whether the mark was set.
    /// `cause` is persisted to disk: callers pass a status and a fixed
    /// reason, never a response body (CONTEXT invariant 5). Not gated on
    /// the tripwire: a rejected `refresh_token` grant is terminal by
    /// decision (CONTEXT.md, 2026-08-24), so this is product behavior
    /// rather than a ladder rail. Returns whether the mark was newly set.
    pub fn mark_refresh_failed(&self, account: &str, cause: &str) -> bool {
        let mut s = self.state.lock().unwrap();
        if s.refresh_failed.contains_key(account) {
            return false;
        }
        s.refresh_failed
            .insert(account.to_string(), cause.to_string());
        self.persist_locked(&s);
        true
    }

    pub fn refresh_failed(&self, account: &str) -> Option<String> {
        self.state
            .lock()
            .unwrap()
            .refresh_failed
            .get(account)
            .cloned()
    }

    pub fn clear_refresh_failed(&self, account: &str) {
        let mut s = self.state.lock().unwrap();
        if s.refresh_failed.remove(account).is_some() {
            self.persist_locked(&s);
        }
    }

    pub fn status(&self) -> RailsStatus {
        let s = self.state.lock().unwrap();
        RailsStatus {
            tripwire_enabled: self.config.tripwire,
            halted: s.tripped.clone().or_else(|| s.ceiling_tripped.clone()),
            refresh_failed: (!s.refresh_failed.is_empty()).then(|| {
                let mut v: Vec<String> = s
                    .refresh_failed
                    .iter()
                    .map(|(a, c)| format!("{a}: {c}"))
                    .collect();
                v.sort();
                v.join("; ")
            }),
            sends: s.sends,
            max_sends: self.config.max_sends,
            journal: match (&self.journal_error, &self.config.journal_path) {
                (Some(error), _) => Some(format!("NOT WRITTEN — {error}")),
                (None, Some(path)) => Some(path.display().to_string()),
                (None, None) => None,
            },
        }
    }

    fn persist_locked(&self, s: &State) {
        let Some(path) = &self.config.state_path else {
            return;
        };
        let persisted = Persisted {
            tripped: s.tripped.clone(),
            refresh_failed_by_account: s.refresh_failed.clone(),
        };
        if persisted.tripped.is_none() && persisted.refresh_failed_by_account.is_empty() {
            let _ = std::fs::remove_file(path);
            return;
        }
        if let Ok(text) = serde_json::to_string(&persisted) {
            let _ = std::fs::write(path, text);
        }
    }

    /// Write and flush one line. The daemon exits via `process::exit`, so
    /// nothing may be left in a buffer.
    fn journal_line(&self, report: &SendReport<'_>) {
        let mut guard = self.journal.lock().unwrap();
        let Some(file) = guard.as_mut() else { return };
        let line = json!({
            "ts": iso_utc(self.clock.wall()),
            "pid": std::process::id(),
            "method": report.method,
            "route": report.route,
            "path": report.url_path,
            "status": report.status,
            "error": report.error,
            "ok": report.ok,
            "counted": report.counted,
            "rate": report.rate,
            "wait_ms": report.wait.as_millis() as u64,
        });
        let mut line = line;
        if let Some(shape) = report.shape {
            line["shape"] = Value::String(shape.as_str().to_string());
        }
        if !report.headers.is_null() {
            line["headers"] = report.headers.clone();
        }
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

/// ISO 8601 UTC with milliseconds, without pulling in a date crate.
fn iso_utc(at: SystemTime) -> String {
    let now = at.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // Howard Hinnant's civil_from_days.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}.{millis:03}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(status: Option<u16>) -> SendReport<'static> {
        SendReport {
            method: "GET",
            route: "character-list",
            url_path: "/character",
            status,
            error: None,
            ok: status.is_some_and(|s| (200..300).contains(&s)),
            counted: true,
            rate: &Value::Null,
            shape: None,
            headers: &Value::Null,
            wait: Duration::ZERO,
        }
    }

    #[test]
    fn block_shape_separates_cloudflare_from_origin() {
        // Rung 10's actual 503 body (openresty, no Cloudflare markers).
        let origin = "<html>\r\n<head><title>503 Service Temporarily Unavailable</title></head>\r\n<body>\r\n<center><h1>503 Service Temporarily Unavailable</h1></center>\r\n<hr><center>openresty</center>";
        assert_eq!(BlockShape::of(origin), BlockShape::Origin);
        // N28's shape: a Cloudflare challenge/block page.
        assert_eq!(
            BlockShape::of("<title>Attention Required! | Cloudflare</title> Ray ID: 8a1"),
            BlockShape::Cloudflare
        );
        assert_eq!(
            BlockShape::of("Error 1015 — you are being rate limited"),
            BlockShape::Cloudflare
        );
        assert_eq!(BlockShape::of(""), BlockShape::Unclassified);
        assert_eq!(BlockShape::of("{}"), BlockShape::Unclassified);
    }

    #[test]
    fn trip_cause_names_the_shape() {
        let rails = Rails::with_config(RailsConfig {
            tripwire: true,
            ..RailsConfig::default()
        });
        let cause = rails
            .record(&SendReport {
                shape: Some(BlockShape::Origin),
                headers: &Value::Null,
                ..report(Some(503))
            })
            .expect("trips");
        assert_eq!(
            cause,
            "503 on GET /character — origin error page, not Cloudflare-shaped (N35)"
        );
        rails.reset_tripwire();
        let cause = rails.record(&report(Some(403))).expect("trips");
        assert!(
            cause.ends_with("unclassified body, possibly a Cloudflare block"),
            "{cause}"
        );
    }

    #[test]
    fn tripwire_off_never_halts() {
        let rails = Rails::disabled();
        assert!(rails.record(&report(Some(429))).is_none());
        assert!(rails.record(&report(Some(503))).is_none());
        assert_eq!(rails.halted(), None);
        // The dead-grant mark is not a rail: it holds with the tripwire off.
        assert!(rails.mark_refresh_failed("A#1", "400"));
        assert_eq!(rails.refresh_failed("A#1").as_deref(), Some("400"));
        assert!(!rails.mark_refresh_failed("A#1", "400 again"), "set once");
        assert_eq!(rails.refresh_failed("A#1").as_deref(), Some("400"));
        assert_eq!(rails.refresh_failed("B#2"), None, "per account");
    }

    #[test]
    fn tripwire_trips_on_first_violation_and_resets_explicitly() {
        let rails = Rails::with_config(RailsConfig {
            tripwire: true,
            ..RailsConfig::default()
        });
        assert!(rails.record(&report(Some(200))).is_none());
        assert_eq!(rails.halted(), None);
        let cause = rails.record(&report(Some(429))).expect("trips");
        assert!(cause.starts_with("429 on GET /character"));
        assert!(rails.halted().unwrap().contains("429 on GET /character"));
        // A second violation does not replace the recorded cause.
        assert!(rails.record(&report(Some(403))).is_none());
        assert!(rails.halted().unwrap().contains("429"));
        rails.reset_tripwire();
        assert_eq!(rails.halted(), None);
        assert!(rails.record(&report(Some(503))).is_some());
        rails.reset_tripwire();
        assert!(rails.record(&report(Some(401))).unwrap().contains("401"));
    }

    #[test]
    fn ceiling_is_per_lifetime_and_counts_every_method() {
        let rails = Rails::with_config(RailsConfig {
            max_sends: Some(2),
            ..RailsConfig::default()
        });
        assert!(rails.record(&report(Some(200))).is_none());
        let second = SendReport {
            method: "HEAD",
            counted: false,
            ..report(Some(200))
        };
        assert!(
            rails
                .record(&second)
                .unwrap()
                .starts_with("ceiling: 2 of 2")
        );
        assert!(rails.halted().unwrap().contains("ceiling"));
        assert_eq!(rails.status().sends, 2);
        rails.reset_tripwire();
        assert_eq!(rails.halted(), None);
    }

    #[test]
    fn violation_trip_and_refresh_mark_persist_but_ceiling_does_not() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("acq-rails-test-{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let config = RailsConfig {
            tripwire: true,
            max_sends: Some(1),
            journal_path: None,
            state_path: Some(path.clone()),
            warnings: Vec::new(),
            daemon_artifact: None,
        };
        {
            let rails = Rails::with_config(config.clone());
            assert!(rails.mark_refresh_failed("A#1", "400 invalid_grant"));
            rails.record(&report(Some(429)));
            assert!(rails.halted().is_some());
        }
        {
            let rails = Rails::with_config(config.clone());
            assert!(rails.halted().unwrap().contains("429"));
            assert_eq!(
                rails.refresh_failed("A#1").as_deref(),
                Some("400 invalid_grant")
            );
            assert_eq!(rails.status().sends, 0);
            rails.reset_tripwire();
            rails.clear_refresh_failed("A#1");
        }
        {
            let rails = Rails::with_config(config.clone());
            assert_eq!(rails.halted(), None);
            assert_eq!(rails.refresh_failed("A#1"), None);
            assert!(!path.exists(), "an all-clear state removes the file");
            rails.record(&report(Some(200)));
            assert!(rails.halted().unwrap().contains("ceiling"));
        }
        let rails = Rails::with_config(config);
        assert_eq!(rails.halted(), None, "ceiling trips never persist");
        let _ = std::fs::remove_file(&path);
    }

    /// C83: a trip persisted beside the socket before step 5 moves into
    /// the world once and is honoured there; with a state in both places
    /// the two are merged, no trip lost, and the legacy file goes.
    #[test]
    fn c83_the_legacy_rails_state_moves_into_the_world_and_loses_no_trip() {
        let dir = std::env::temp_dir().join(format!("acq-rails-move-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let legacy = dir.join("d.sock").with_extension("mock.rails.json");
        let current = dir.join("store").join("mock").join("rails.json");
        assert_eq!(
            migrate_legacy_state(&legacy, &current),
            Ok(None),
            "nothing to move"
        );
        assert_eq!(verify_state(&current), Ok(None));

        std::fs::write(
            &legacy,
            r#"{"tripped":"429 on GET /stash","refresh_failed_by_account":{"A#1":"400"}}"#,
        )
        .unwrap();
        let line = migrate_legacy_state(&legacy, &current)
            .unwrap()
            .expect("moved");
        assert!(line.contains("moved") && line.contains("429"), "{line}");
        assert!(!legacy.exists(), "the legacy file is gone");
        assert!(
            !current.with_extension("json.tmp").exists(),
            "no scratch left"
        );
        assert_eq!(verify_state(&current), Ok(Some(())));
        let config = RailsConfig {
            tripwire: true,
            state_path: Some(current.clone()),
            ..RailsConfig::default()
        };
        let rails = Rails::with_config(config.clone());
        assert!(rails.halted().unwrap().contains("429 on GET /stash"));
        assert_eq!(rails.refresh_failed("A#1").as_deref(), Some("400"));
        drop(rails);

        // Both present: the current cause wins, marks are the union.
        std::fs::write(
            &legacy,
            r#"{"tripped":"503 on GET /profile","refresh_failed_by_account":{"B#2":"401"}}"#,
        )
        .unwrap();
        let line = migrate_legacy_state(&legacy, &current)
            .unwrap()
            .expect("merged");
        assert!(line.contains("429 on GET /stash"), "{line}");
        assert!(!legacy.exists());
        let rails = Rails::with_config(config);
        assert!(rails.halted().unwrap().contains("429 on GET /stash"));
        assert_eq!(rails.refresh_failed("A#1").as_deref(), Some("400"));
        assert_eq!(rails.refresh_failed("B#2").as_deref(), Some("401"));

        // A legacy file of another shape: a typed failure, both files
        // where they were, the remedy named (fail closed, review
        // 2026-09-11).
        std::fs::write(&legacy, "{nope").unwrap();
        let before = std::fs::read_to_string(&current).unwrap();
        let err = migrate_legacy_state(&legacy, &current).unwrap_err();
        assert!(err.0.contains("not readable") && legacy.exists(), "{err}");
        assert!(err.to_string().contains("reset-tripwire"), "{err}");
        assert_eq!(std::fs::read_to_string(&current).unwrap(), before);
        // A world state that cannot be read is never overwritten, and
        // the daemon's own check on it refuses too.
        std::fs::write(
            &legacy,
            r#"{"tripped":"401 on GET /profile","refresh_failed_by_account":{}}"#,
        )
        .unwrap();
        std::fs::write(&current, "{corrupt").unwrap();
        let err = migrate_legacy_state(&legacy, &current).unwrap_err();
        assert!(
            err.0.contains("not overwritten") && legacy.exists(),
            "{err}"
        );
        assert_eq!(std::fs::read_to_string(&current).unwrap(), "{corrupt");
        assert!(verify_state(&current).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn journal_writes_one_flushed_line_without_secrets() {
        let path =
            std::env::temp_dir().join(format!("acq-journal-test-{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let rails = Rails::with_config(RailsConfig {
            journal_path: Some(path.clone()),
            ..RailsConfig::default()
        });
        let rate = json!({ "X-Rate-Limit-Policy": "character-list-request-limit" });
        rails.record(&SendReport {
            rate: &rate,
            ..report(Some(200))
        });
        rails.record(&SendReport {
            status: None,
            error: Some("connection reset"),
            ok: false,
            ..report(None)
        });
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<Value> = text
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 3, "header plus two sends");
        assert_eq!(lines[0]["event"], "open");
        assert_eq!(lines[0]["clock"], "system");
        assert_eq!(
            lines[0]["contract"],
            acquisition_protocol::CONTRACT_REVISION
        );
        assert_eq!(lines[0]["daemon"], Value::Null, "no executable of its own");
        assert_eq!(lines[0]["pid"], std::process::id());
        assert_eq!(lines[1]["status"], 200);
        assert_eq!(
            lines[1]["rate"]["X-Rate-Limit-Policy"],
            "character-list-request-limit"
        );
        assert_eq!(lines[1]["counted"], true);
        assert_eq!(lines[2]["status"], Value::Null);
        assert_eq!(lines[2]["error"], "connection reset");
        assert!(lines[1]["ts"].as_str().unwrap().ends_with('Z'));
        for key in ["authorization", "token", "body", "bearer"] {
            assert!(!text.to_lowercase().contains(key), "journal leaked {key}");
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn persisted_trip_is_ignored_without_the_tripwire() {
        let path = std::env::temp_dir().join(format!("acq-rails-off-{}.json", std::process::id()));
        std::fs::write(
            &path,
            r#"{"tripped":"429 on GET /x","refresh_failed_by_account":{"A#1":"400"}}"#,
        )
        .unwrap();
        let rails = Rails::with_config(RailsConfig {
            state_path: Some(path.clone()),
            ..RailsConfig::default()
        });
        assert_eq!(rails.halted(), None);
        assert_eq!(
            rails.refresh_failed("A#1").as_deref(),
            Some("400"),
            "the dead-grant mark is product behavior and survives rails-off"
        );
        assert!(
            path.exists(),
            "a rails-off daemon does not delete the ladder's state"
        );
        let armed = Rails::with_config(RailsConfig {
            tripwire: true,
            state_path: Some(path.clone()),
            ..RailsConfig::default()
        });
        assert!(armed.halted().is_some());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unopenable_journal_is_reported_not_silent() {
        let rails = Rails::with_config(RailsConfig {
            journal_path: Some(PathBuf::from("/nonexistent-dir/acq.jsonl")),
            ..RailsConfig::default()
        });
        assert!(rails.status().journal.unwrap().starts_with("NOT WRITTEN"));
        assert_eq!(rails.startup_warnings().len(), 1);
    }

    #[test]
    fn zero_ceiling_refuses_before_the_first_send() {
        let rails = Rails::with_config(RailsConfig {
            max_sends: Some(0),
            ..RailsConfig::default()
        });
        assert!(rails.halted().unwrap().contains("0 sends"));
    }

    #[test]
    fn iso_timestamp_is_well_formed() {
        let ts = iso_utc(SystemTime::now());
        assert_eq!(ts.len(), 24, "{ts}");
        assert!(ts.starts_with("20"));
        assert_eq!(&ts[10..11], "T");
    }
    #[test]
    fn a_head_trip_reports_headers_not_a_body() {
        let rails = Rails::with_config(RailsConfig {
            tripwire: true,
            ..RailsConfig::default()
        });
        let headers = json!({ "cf-ray": "8a1-SJC", "content-type": "text/html" });
        let cause = rails
            .record(&SendReport {
                method: "HEAD",
                url_path: "/profile",
                counted: false,
                headers: &headers,
                ..report(Some(403))
            })
            .expect("a 403 trips");
        assert!(cause.contains("no body to classify (HEAD)"), "{cause}");
        assert!(cause.contains("cf-ray"), "{cause}");
        assert!(!cause.contains("unclassified body"), "{cause}");
    }
    #[test]
    fn a_403_with_www_authenticate_is_named_an_auth_error() {
        let rails = Rails::with_config(RailsConfig {
            tripwire: true,
            ..RailsConfig::default()
        });
        let headers = json!({
            "www-authenticate": "Bearer realm=\"pathofexile:production\", error=\"insufficient_scope\""
        });
        let cause = rails
            .record(&SendReport {
                method: "HEAD",
                url_path: "/league",
                headers: &headers,
                ..report(Some(403))
            })
            .expect("a 403 still trips");
        assert!(
            cause.starts_with("403 on HEAD /league — auth error: "),
            "{cause}"
        );
        assert!(cause.contains("insufficient_scope"), "{cause}");
    }
}
