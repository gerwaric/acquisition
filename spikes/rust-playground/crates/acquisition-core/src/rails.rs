//! Live-test safety rails (`LIVE-TESTING.md`, package L0).
//!
//! The rails never add a send and never delay a halt; they only refuse or
//! record. `ChokePoint` consults them immediately before acquiring any gate
//! permit and reports every completed exchange to them afterward.
//!
//! - **Tripwire** (`ACQ_TRIPWIRE=1`, ladder-only): the first landed 429 on
//!   any route — HEAD and token included — or any 401/403/503 halts every later
//!   send until an explicit `reset_tripwire`. Persisted per provider so a
//!   respawned daemon stays tripped.
//! - **Dead-token stop** (with the tripwire): a 4xx other than 429 on a
//!   `refresh_token` grant marks the session refresh-failed; later refreshes
//!   fail fast without sending until login or logout. Persisted with the
//!   tripwire.
//! - **Send ceiling** (`ACQ_MAX_SENDS=<n>`, ladder-only): after `n` real
//!   sends in this daemon lifetime the daemon halts with cause `ceiling`.
//!   Per-lifetime and never persisted.
//! - **Send journal** (`ACQ_JOURNAL=<path>`, permanent): one JSON line per
//!   actual send, flushed per line, never containing a token or body.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
}

impl RailsConfig {
    /// Read the environment. `state_dir_hint` is the socket path: the
    /// state file sits beside it (so `ACQ_SOCKET` isolates parallel
    /// daemons) and is keyed by provider so mock and real never share it.
    pub fn from_env(
        provider_name: &str,
        socket_path: &Path,
        default_journal: &Path,
    ) -> RailsConfig {
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
        let state_path = Some(socket_path.with_extension(format!("{provider_name}.rails.json")));
        RailsConfig {
            tripwire,
            max_sends,
            journal_path,
            state_path,
            warnings,
        }
    }
}

/// What persists across daemon restarts. Only violation-class trips and
/// the refresh-failed mark; the ceiling is per lifetime by decision.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Persisted {
    #[serde(default)]
    tripped: Option<String>,
    /// Per account (username → cause). The pre-multi-account file had one
    /// `refresh_failed` string; it is ignored on load (one re-login).
    #[serde(default)]
    refresh_failed_by_account: HashMap<String, String>,
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

/// Serializable summary for `daemon status` and the dashboard.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RailsStatus {
    pub tripwire_enabled: bool,
    /// Why sends are refused, if they are.
    pub halted: Option<String>,
    pub refresh_failed: Option<String>,
    pub sends: u64,
    pub max_sends: Option<u64>,
    pub journal: Option<String>,
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
    /// build, and which clock. The per-send lines are unchanged. A reader
    /// that finds `"clock":"manual"` is looking at a scenario, not at GGG;
    /// one that finds a `build` that is not its checkout is looking at the
    /// rung-8 mistake.
    fn journal_header(&self) {
        let mut guard = self.journal.lock().unwrap();
        let Some(file) = guard.as_mut() else { return };
        let line = json!({
            "event": "open",
            "ts": iso_utc(self.clock.wall()),
            "pid": std::process::id(),
            "build": crate::BUILD,
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
        assert_eq!(lines[0]["build"], crate::BUILD);
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
