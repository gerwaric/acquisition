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

use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

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
    #[serde(default)]
    refresh_failed: Option<String>,
}

#[derive(Debug, Default)]
struct State {
    /// Violation/Cloudflare trip cause (persisted).
    tripped: Option<String>,
    /// Ceiling trip cause (this lifetime only).
    ceiling_tripped: Option<String>,
    refresh_failed: Option<String>,
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
}

pub struct Rails {
    config: RailsConfig,
    state: Mutex<State>,
    journal: Mutex<Option<File>>,
    /// Why the journal could not be opened, if it could not.
    journal_error: Option<String>,
}

impl Rails {
    /// Everything off and nothing persisted: the default for unit tests and
    /// for `ChokePoint::new()` callers that predate the rails.
    pub fn disabled() -> Rails {
        Rails::with_config(RailsConfig::default())
    }

    pub fn with_config(config: RailsConfig) -> Rails {
        let mut state = State::default();
        // A persisted trip belongs to the tripwire: a daemon started without
        // it (the post-baseline default) neither honors nor deletes it.
        if config.tripwire
            && let Some(path) = &config.state_path
            && let Ok(text) = std::fs::read_to_string(path)
            && let Ok(persisted) = serde_json::from_str::<Persisted>(&text)
        {
            state.tripped = persisted.tripped;
            state.refresh_failed = persisted.refresh_failed;
        }
        let (journal, journal_error) = match &config.journal_path {
            None => (None, None),
            Some(path) => match OpenOptions::new().create(true).append(true).open(path) {
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
        Rails {
            config,
            state: Mutex::new(state),
            journal: Mutex::new(journal),
            journal_error,
        }
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
                Some(status @ (403 | 503)) => Some(format!(
                    "{status} on {} {} — possibly a Cloudflare block",
                    report.method, report.url_path
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
    /// reason, never a response body (CONTEXT invariant 5).
    pub fn mark_refresh_failed(&self, cause: &str) -> bool {
        if !self.config.tripwire {
            return false;
        }
        let mut s = self.state.lock().unwrap();
        s.refresh_failed = Some(cause.to_string());
        self.persist_locked(&s);
        true
    }

    pub fn refresh_failed(&self) -> Option<String> {
        self.state.lock().unwrap().refresh_failed.clone()
    }

    pub fn clear_refresh_failed(&self) {
        let mut s = self.state.lock().unwrap();
        if s.refresh_failed.take().is_some() {
            self.persist_locked(&s);
        }
    }

    pub fn status(&self) -> RailsStatus {
        let s = self.state.lock().unwrap();
        RailsStatus {
            tripwire_enabled: self.config.tripwire,
            halted: s.tripped.clone().or_else(|| s.ceiling_tripped.clone()),
            refresh_failed: s.refresh_failed.clone(),
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
            refresh_failed: s.refresh_failed.clone(),
        };
        if persisted.tripped.is_none() && persisted.refresh_failed.is_none() {
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
            "ts": iso_utc_now(),
            "pid": std::process::id(),
            "method": report.method,
            "route": report.route,
            "path": report.url_path,
            "status": report.status,
            "error": report.error,
            "ok": report.ok,
            "counted": report.counted,
            "rate": report.rate,
        });
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

/// ISO 8601 UTC with milliseconds, without pulling in a date crate.
fn iso_utc_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
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
        }
    }

    #[test]
    fn tripwire_off_never_halts() {
        let rails = Rails::disabled();
        assert!(rails.record(&report(Some(429))).is_none());
        assert!(rails.record(&report(Some(503))).is_none());
        assert_eq!(rails.halted(), None);
        assert!(!rails.mark_refresh_failed("400"));
        assert_eq!(rails.refresh_failed(), None);
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
            assert!(rails.mark_refresh_failed("400 invalid_grant"));
            rails.record(&report(Some(429)));
            assert!(rails.halted().is_some());
        }
        {
            let rails = Rails::with_config(config.clone());
            assert!(rails.halted().unwrap().contains("429"));
            assert_eq!(rails.refresh_failed().as_deref(), Some("400 invalid_grant"));
            assert_eq!(rails.status().sends, 0);
            rails.reset_tripwire();
            rails.clear_refresh_failed();
        }
        {
            let rails = Rails::with_config(config.clone());
            assert_eq!(rails.halted(), None);
            assert_eq!(rails.refresh_failed(), None);
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
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["status"], 200);
        assert_eq!(
            lines[0]["rate"]["X-Rate-Limit-Policy"],
            "character-list-request-limit"
        );
        assert_eq!(lines[0]["counted"], true);
        assert_eq!(lines[1]["status"], Value::Null);
        assert_eq!(lines[1]["error"], "connection reset");
        assert!(lines[0]["ts"].as_str().unwrap().ends_with('Z'));
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
            r#"{"tripped":"429 on GET /x","refresh_failed":"400"}"#,
        )
        .unwrap();
        let rails = Rails::with_config(RailsConfig {
            state_path: Some(path.clone()),
            ..RailsConfig::default()
        });
        assert_eq!(rails.halted(), None);
        assert_eq!(rails.refresh_failed(), None);
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
        let ts = iso_utc_now();
        assert_eq!(ts.len(), 24, "{ts}");
        assert!(ts.starts_with("20"));
        assert_eq!(&ts[10..11], "T");
    }
}
