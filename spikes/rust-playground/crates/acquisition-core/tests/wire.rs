//! The wire — the first of C12's two frontend surfaces — pinned (C85;
//! TESTING-NOTES item 3): one JSON document per
//! `Request` and `Response` variant, the bootstrap frames, the closed
//! error-kind set and the frame bound, under `tests/fixtures/wire/`. A
//! golden file, not a frozen one (P5): a wire change fails here as a diff
//! a reviewer reads, and
//! `ACQ_UPDATE_FIXTURES=1 cargo test -p acquisition-core --test wire`
//! rewrites the fixtures. The matches over the enums are exhaustive, so
//! a new variant does not compile until it is named here — and named
//! here, it needs a sample below and a fixture on disk.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use acquisition_core::job::{JobInfo, JobState, Outcome};
use acquisition_core::protocol::{
    Bootstrap, BootstrapReply, ErrorKind, ErrorRecord, MAX_FRAME_BYTES, Quote, QuoteJob,
    QuoteScope, Request, Response, SessionStatus, UNKNOWN, error_message,
};
use acquisition_core::rails::RailsStatus;
use acquisition_core::ratelimit::{
    DegradedEndpoint, PolicyStatus, RuleStatus, SendRecord, WindowStatus,
};
use serde::Serialize;
use serde_json::{Value, json};

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/wire");

fn update() -> bool {
    std::env::var_os("ACQ_UPDATE_FIXTURES").is_some()
}

/// Compare `actual` with the fixture at `name`, or rewrite it under
/// `ACQ_UPDATE_FIXTURES=1`. Returns the mismatch, so a test can report
/// every drifted document at once instead of the first.
fn check(name: &str, actual: &Value) -> Option<String> {
    let path = PathBuf::from(DIR).join(format!("{name}.json"));
    let mut text = serde_json::to_string_pretty(actual).expect("serializable");
    text.push('\n');
    if update() {
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&path, &text).expect("writing the fixture");
        return None;
    }
    match std::fs::read_to_string(&path) {
        Ok(on_disk) if on_disk == text => None,
        Ok(on_disk) => Some(format!(
            "{}:\n--- fixture\n{on_disk}--- this build\n{text}",
            path.display()
        )),
        Err(e) => Some(format!("{}: {e} — this build:\n{text}", path.display())),
    }
}

/// The fixture files under `dir`, by stem.
fn on_disk(dir: &str) -> BTreeSet<String> {
    let path = Path::new(DIR).join(dir);
    std::fs::read_dir(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .map(|entry| {
            let name = entry.expect("entry").file_name();
            let name = name.to_string_lossy();
            name.strip_suffix(".json")
                .unwrap_or_else(|| panic!("{name}: not a .json fixture"))
                .to_string()
        })
        .collect()
}

/// Pin one plane: every sample against its fixture, the fixture set equal
/// to the sample set, and every fixture readable back into the type and
/// re-serialized identically (so deserialization is pinned too).
fn pin<T: Serialize + serde::de::DeserializeOwned>(dir: &str, samples: &[(&str, T)]) {
    let mut problems = Vec::new();
    let mut names = BTreeSet::new();
    for (name, sample) in samples {
        assert!(
            names.insert(name.to_string()),
            "{dir}/{name}: sampled twice"
        );
        let value = serde_json::to_value(sample).expect("serializable");
        if let Some(problem) = check(&format!("{dir}/{name}"), &value) {
            problems.push(problem);
        }
        let back: T = serde_json::from_value(value.clone())
            .unwrap_or_else(|e| panic!("{dir}/{name}: does not read back: {e}"));
        assert_eq!(
            serde_json::to_value(&back).expect("serializable"),
            value,
            "{dir}/{name}: changes on a round trip"
        );
    }
    if !update() {
        let files = on_disk(dir);
        assert_eq!(
            files, names,
            "{dir}: the fixtures on disk are not the sampled variants (stale or missing files)"
        );
    }
    assert!(
        problems.is_empty(),
        "the wire changed; review the diff, then `ACQ_UPDATE_FIXTURES=1 cargo test -p acquisition-core --test wire` if it is intended:\n\n{}",
        problems.join("\n")
    );
}

// ---- samples ------------------------------------------------------------

fn job() -> JobInfo {
    JobInfo {
        id: 42,
        kind: "stash".into(),
        state: JobState::Waiting,
        priority: 0,
        submitted_by: "cli:1234".into(),
        eta_seconds: Some(17),
        parent: Some(41),
        retries: 1,
        account: Some("Alice#1234".into()),
        params: json!({ "league": "Standard", "id": "t1", "deep": false }),
    }
}

fn rails() -> RailsStatus {
    RailsStatus {
        tripwire_enabled: true,
        halted: Some("tripwire: 429 on stash@Alice#1234".into()),
        refresh_failed: None,
        sends: 12,
        max_sends: Some(200),
        journal: Some("/tmp/acq.sock.mock.sends.jsonl".into()),
    }
}

fn rule() -> RuleStatus {
    RuleStatus {
        name: "Account".into(),
        windows: vec![WindowStatus {
            hits: 3,
            max_hits: 45,
            period_secs: 60,
            restriction_secs: 60,
            restricted_secs: 0,
            bucket_secs: 1,
        }],
    }
}

fn quote() -> Quote {
    Quote {
        observed_at: 1_800_000_000,
        provider: "mock".into(),
        account: Some("Alice#1234".into()),
        halted: None,
        work: vec![QuoteJob {
            kind: "stash".into(),
            params: json!({ "league": "Standard", "id": "t1" }),
        }],
        scopes: vec![QuoteScope {
            key: "stash-request-limit@Alice#1234".into(),
            endpoints: vec!["stash@Alice#1234".into()],
            requests: 1,
            queued_ahead: 0,
            policy: Some("stash-request-limit@Alice#1234".into()),
            rules: vec![rule()],
            observed_seconds_ago: Some(4),
            eta_seconds: Some(0),
            notes: vec![],
        }],
        not_covered: vec!["429 re-sends (up to 3 per request) — possible, never predicted".into()],
    }
}

/// The wire name of a request, exhaustively: a new variant fails here
/// until it is named, and then wants a sample in `requests`.
fn request_name(r: &Request) -> &'static str {
    match r {
        Request::Submit { .. } => "submit",
        Request::Status { .. } => "status",
        Request::Result { .. } => "result",
        Request::Cancel { .. } => "cancel",
        Request::SetPriority { .. } => "set_priority",
        Request::List => "list",
        Request::Subscribe => "subscribe",
        Request::AuthStart => "auth_start",
        Request::AuthStatus => "auth_status",
        Request::AuthCheck { .. } => "auth_check",
        Request::AuthLogout { .. } => "auth_logout",
        Request::Quote { .. } => "quote",
        Request::DaemonStatus => "daemon_status",
        Request::ResetTripwire => "reset_tripwire",
        Request::Dashboard => "dashboard",
    }
}

fn requests() -> Vec<Request> {
    vec![
        Request::Submit {
            kind: "stash".into(),
            params: json!({ "league": "Standard", "id": "t1", "deep": false }),
            priority: 0,
            submitted_by: "cli:1234".into(),
            account: Some("Alice#1234".into()),
        },
        Request::Status { id: 42 },
        Request::Result { id: 42 },
        Request::Cancel { id: 42 },
        Request::SetPriority {
            id: 42,
            priority: 5,
        },
        Request::List,
        Request::Subscribe,
        Request::AuthStart,
        Request::AuthStatus,
        Request::AuthCheck {
            account: Some("Alice#1234".into()),
        },
        Request::AuthLogout { account: None },
        Request::Quote {
            jobs: vec![QuoteJob {
                kind: "stash".into(),
                params: json!({ "league": "Standard", "id": "t1" }),
            }],
            account: Some("Alice#1234".into()),
        },
        Request::DaemonStatus,
        Request::ResetTripwire,
        Request::Dashboard,
    ]
}

/// The wire name of a response, exhaustively (see `request_name`).
fn response_name(r: &Response) -> &'static str {
    match r {
        Response::Submitted { .. } => "submitted",
        Response::Status { .. } => "status",
        Response::Result { .. } => "result",
        Response::Ack => "ack",
        Response::Jobs { .. } => "jobs",
        Response::Subscribed => "subscribed",
        Response::AuthUrl { .. } => "auth_url",
        Response::Auth { .. } => "auth",
        Response::DaemonStatus { .. } => "daemon_status",
        Response::Quote { .. } => "quote",
        Response::Dashboard { .. } => "dashboard",
        Response::Error { .. } => "error",
        Response::Event { .. } => "event",
        Response::ResyncRequired { .. } => "resync_required",
    }
}

fn responses() -> Vec<Response> {
    vec![
        Response::Submitted { id: 42 },
        Response::Status { job: job() },
        Response::Result {
            id: 42,
            outcome: Outcome::Success {
                payload: json!({ "status": 200, "body": { "stash": { "id": "t1", "items": [] } } }),
            },
        },
        Response::Ack,
        Response::Jobs { jobs: vec![job()] },
        Response::Subscribed,
        Response::AuthUrl {
            authorize_url: "http://127.0.0.1:1234/authorize?client_id=acquisition&state=st-x"
                .into(),
        },
        Response::Auth {
            logged_in: true,
            pending: false,
            login_ok: Some("Alice#1234".into()),
            login_error: None,
            username: Some("Alice#1234".into()),
            access_expires_in_seconds: Some(3500),
            keyring: "ok".into(),
            provider: "mock".into(),
            accounts: vec![SessionStatus {
                username: "Alice#1234".into(),
                access_expires_in_seconds: Some(3500),
                keyring: "ok".into(),
            }],
        },
        Response::DaemonStatus {
            pid: 4242,
            version: "0.0.1 (runtime 0123456789ab)".into(),
            provider: "mock".into(),
            uptime_seconds: 61,
            connections: 2,
            jobs_waiting: 1,
            jobs_running: 1,
            policies_known: 1,
            in_flight: 1,
            max_in_flight: 2,
            rails: rails(),
            keyring: "ok".into(),
        },
        Response::Quote { quote: quote() },
        Response::Dashboard {
            pid: 4242,
            version: "0.0.1 (runtime 0123456789ab)".into(),
            provider: "mock".into(),
            uptime_seconds: 61,
            connections: 2,
            logged_in: true,
            username: Some("Alice#1234".into()),
            access_expires_in_seconds: Some(3500),
            keyring: "ok".into(),
            in_flight: 1,
            max_in_flight: 2,
            policies: vec![PolicyStatus {
                policy: "stash-request-limit@Alice#1234".into(),
                endpoints: vec!["stash@Alice#1234".into()],
                rules: vec![rule()],
                next_safe_in_seconds: 0.0,
                last_observed_seconds_ago: 4.5,
                history_len: 3,
                retry_after_secs: None,
                headers: json!({ "x-rate-limit-policy": "stash-request-limit" }),
            }],
            policyless_endpoints: vec!["profile@Alice#1234".into()],
            degraded_endpoints: vec![DegradedEndpoint {
                endpoint: "leagues".into(),
                seconds_left: 30.5,
                reason: "HEAD answered 403".into(),
            }],
            jobs: vec![job()],
            sends: vec![SendRecord {
                seconds_ago: 1.5,
                endpoint: "stash@Alice#1234".into(),
                method: "GET".into(),
                url: "http://127.0.0.1:1234/stash/Standard/t1".into(),
                outcome: "200 OK".into(),
                ok: true,
            }],
            rails: rails(),
            errors: vec![ErrorRecord {
                seconds_ago: 9.0,
                message: "submit refused: not logged in — run `acq auth`".into(),
            }],
        },
        Response::Error {
            kind: ErrorKind::UnknownJob,
            message: "no job 99".into(),
        },
        Response::Event { job: job() },
        Response::ResyncRequired { missed: 300 },
    ]
}

// ---- the pins -----------------------------------------------------------

#[test]
fn c85_every_request_variant_has_a_pinned_wire_shape() {
    let samples: Vec<(&str, Request)> = requests()
        .into_iter()
        .map(|r| (request_name(&r), r))
        .collect();
    pin("request", &samples);
}

#[test]
fn c85_every_response_variant_has_a_pinned_wire_shape() {
    let samples: Vec<(&str, Response)> = responses()
        .into_iter()
        .map(|r| (response_name(&r), r))
        .collect();
    pin("response", &samples);
}

/// The four bootstrap frames, and their lenient readers: a frame keyed
/// on the name alone is read, every other field optional, unknown
/// fields ignored — the property that lets any two revisions identify
/// each other and lets a human stop a daemon across a mismatch.
#[test]
fn c85_the_bootstrap_plane_is_pinned_and_read_leniently() {
    pin(
        "bootstrap",
        &[
            (
                "hello",
                serde_json::to_value(Bootstrap::Hello {
                    client_version: "0.0.1 (runtime 0123456789ab)".into(),
                })
                .unwrap(),
            ),
            (
                "daemon_stop",
                serde_json::to_value(Bootstrap::DaemonStop).unwrap(),
            ),
            (
                "hello_reply",
                serde_json::to_value(BootstrapReply::Hello {
                    daemon_version: "0.0.1 (runtime 0123456789ab)".into(),
                    pid: 4242,
                    provider: "mock".into(),
                })
                .unwrap(),
            ),
            (
                "stopping",
                serde_json::to_value(BootstrapReply::Stopping).unwrap(),
            ),
        ],
    );

    // A future client's hello: extra fields, a nested shape, no version.
    assert_eq!(
        Bootstrap::read(br#"{"req":"hello","contract":{"rev":"x"},"world":"/w"}"#),
        Some(Bootstrap::Hello {
            client_version: UNKNOWN.into()
        })
    );
    assert_eq!(
        Bootstrap::read(br#"{"req":"hello","client_version":"9.9.9","extra":1}"#),
        Some(Bootstrap::Hello {
            client_version: "9.9.9".into()
        })
    );
    assert_eq!(
        Bootstrap::read(br#"{"req":"daemon_stop","reason":"upgrade"}"#),
        Some(Bootstrap::DaemonStop)
    );
    assert_eq!(Bootstrap::read(br#"{"req":"list"}"#), None);
    assert_eq!(Bootstrap::read(b"not json"), None);
    assert_eq!(Bootstrap::read(br#"{"resp":"hello"}"#), None);

    // A future daemon's hello: identified as foreign, never unparsed.
    assert_eq!(
        BootstrapReply::read(
            br#"{"resp":"hello","daemon_version":"9.9.9 (runtime ffffffffffff)","pid":7,"provider":"ggg","world":"/w"}"#
        ),
        Some(BootstrapReply::Hello {
            daemon_version: "9.9.9 (runtime ffffffffffff)".into(),
            pid: 7,
            provider: "ggg".into(),
        })
    );
    assert_eq!(
        BootstrapReply::read(br#"{"resp":"hello"}"#),
        Some(BootstrapReply::Hello {
            daemon_version: UNKNOWN.into(),
            pid: 0,
            provider: UNKNOWN.into(),
        })
    );
    assert_eq!(
        BootstrapReply::read(br#"{"resp":"stopping","world":"/w"}"#),
        Some(BootstrapReply::Stopping)
    );
    let error = br#"{"resp":"error","kind":"future_kind","message":"busy"}"#;
    assert_eq!(BootstrapReply::read(error), None);
    assert_eq!(error_message(error).as_deref(), Some("busy"));
    assert_eq!(error_message(br#"{"resp":"ack"}"#), None);

    // The bootstrap requests are not versioned requests, and the versioned
    // parser refuses them: one parser per plane.
    for frame in [
        r#"{"req":"hello","client_version":"x"}"#,
        r#"{"req":"daemon_stop"}"#,
    ] {
        assert!(serde_json::from_str::<Request>(frame).is_err(), "{frame}");
    }
    for frame in [
        r#"{"resp":"hello","daemon_version":"x","pid":1,"provider":"mock"}"#,
        r#"{"resp":"stopping"}"#,
    ] {
        assert!(serde_json::from_str::<Response>(frame).is_err(), "{frame}");
    }
}

/// The error-kind set is closed: its wire names are a fixture, a kind is
/// required on every error frame, and an unknown name does not parse.
#[test]
fn c85_the_error_kind_set_is_closed_and_pinned() {
    let names: Vec<Value> = ErrorKind::ALL
        .iter()
        .map(|k| serde_json::to_value(k).unwrap())
        .collect();
    for (kind, name) in ErrorKind::ALL.iter().zip(&names) {
        assert_eq!(name, &json!(kind.as_str()), "{kind:?}");
        assert_eq!(
            serde_json::from_value::<ErrorKind>(name.clone()).unwrap(),
            *kind
        );
    }
    if let Some(problem) = check("error_kinds", &Value::Array(names)) {
        panic!("the error-kind set changed — a protocol change, reviewed as this diff:\n{problem}");
    }
    assert!(
        serde_json::from_str::<Response>(r#"{"resp":"error","message":"x"}"#).is_err(),
        "an error frame without a kind is not this version's"
    );
    assert!(
        serde_json::from_str::<Response>(r#"{"resp":"error","kind":"halted","message":"x"}"#)
            .is_err(),
        "a kind outside the closed set does not parse"
    );
}

/// The frame bound is a wire fact both sides share.
#[test]
fn c85_the_frame_bound_is_pinned() {
    if let Some(problem) = check("frame", &json!({ "max_frame_bytes": MAX_FRAME_BYTES })) {
        panic!("the frame bound changed:\n{problem}");
    }
}
