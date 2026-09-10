//! The wire — the first of C12's two frontend surfaces — pinned (C85;
//! TESTING-NOTES item 3): one JSON document per
//! `Request` and `Response` variant, the bootstrap frames, the closed
//! error-kind set and the frame bound, under `tests/fixtures/wire/`. A
//! golden file, not a frozen one (P5): a wire change fails here as a diff
//! a reviewer reads, and
//! `ACQ_UPDATE_FIXTURES=1 cargo test -p acquisition-protocol --test wire`
//! rewrites the fixtures. The matches over the enums are exhaustive, so
//! a new variant does not compile until it is named here — and named
//! here, it needs a sample below and a fixture on disk.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use acquisition_protocol::artifact::{Artifact, FileIdentity};
use acquisition_protocol::job::{JobInfo, JobState, MAX_429_RETRIES, Outcome};
use acquisition_protocol::protocol::{
    Bootstrap, BootstrapReply, ErrorKind, ErrorRecord, MAX_FRAME_BYTES, Quote, QuoteJob,
    QuoteScope, Request, Response, SessionStatus, UNKNOWN, error_message,
};
use acquisition_protocol::status::{
    DegradedEndpoint, PolicyStatus, RailsStatus, RuleStatus, SendRecord, WindowStatus,
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

/// The variant names of an internally tagged enum, from the type itself:
/// serde's unknown-variant error lists every one it expected. This is
/// what makes "one fixture per variant" a property the compiler and serde
/// hold, not a list kept by hand (review finding 2026-09-10, round 1). A
/// change in serde's message shape fails loudly below, never vacuously.
fn variants_of<T: serde::de::DeserializeOwned>(probe: Value) -> BTreeSet<String> {
    let message = serde_json::from_value::<T>(probe)
        .err()
        .expect("the probe names no variant")
        .to_string();
    // "expected one of `a`, `b`, `c`" for three or more variants,
    // "expected `a` or `b`" for two.
    let listed = message
        .split_once("expected ")
        .map(|(_, rest)| rest.trim_start_matches("one of "))
        .unwrap_or_else(|| panic!("serde no longer lists the variants: {message}"));
    let names: BTreeSet<String> = listed
        .split(',')
        .flat_map(|part| part.split(" or "))
        .map(|part| part.trim().trim_matches('`').to_string())
        .filter(|name| !name.is_empty())
        .collect();
    assert!(names.len() > 1, "{message}");
    names
}

/// Pin one plane: every sample against its fixture, the sample set equal
/// to the type's variants and to the fixture set on disk, and every
/// fixture readable back into the type and re-serialized identically (so
/// deserialization is pinned too).
fn pin<T: Serialize + serde::de::DeserializeOwned>(
    dir: &str,
    variants: &BTreeSet<String>,
    samples: &[(&str, T)],
) {
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
    assert_eq!(
        &names, variants,
        "{dir}: the sampled variants are not the type's variants — a variant without a sample, or a sample under the wrong name"
    );
    if !update() {
        let files = on_disk(dir);
        assert_eq!(
            files, names,
            "{dir}: the fixtures on disk are not the sampled variants (stale or missing files)"
        );
    }
    assert!(
        problems.is_empty(),
        "the wire changed; review the diff, then `ACQ_UPDATE_FIXTURES=1 cargo test -p acquisition-protocol --test wire` if it is intended:\n\n{}",
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

fn artifact() -> Artifact {
    Artifact {
        file: FileIdentity {
            path: "/opt/acquisition/acqd".into(),
            len: 22_605_208,
            mtime_ns: 1_800_000_000_000_000_000,
            dev: 16_777_233,
            ino: 4_242_424,
        },
        sha256: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
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
        // The daemon's own sentence (`daemon.rs`, the quote's `not_covered`),
        // built from the promise so the sample cannot contradict it (review
        // finding 2026-09-10: the fixture said 3 beside a constant of 2).
        not_covered: vec![format!(
            "429 re-sends (up to {MAX_429_RETRIES} per request) — possible, never predicted"
        )],
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
    pin(
        "request",
        &variants_of::<Request>(json!({ "req": "__none__" })),
        &samples,
    );
}

#[test]
fn c85_every_response_variant_has_a_pinned_wire_shape() {
    let samples: Vec<(&str, Response)> = responses()
        .into_iter()
        .map(|r| (response_name(&r), r))
        .collect();
    pin(
        "response",
        &variants_of::<Response>(json!({ "resp": "__none__" })),
        &samples,
    );
}

/// The four bootstrap frames, and their lenient readers: a frame keyed
/// on the name alone is read, every other field optional, unknown
/// fields ignored — the property that lets any two revisions identify
/// each other and lets a human stop a daemon across a mismatch.
#[test]
fn c85_the_bootstrap_plane_is_pinned_and_read_leniently() {
    let mut bootstrap = variants_of::<Bootstrap>(json!({ "req": "__none__" }));
    for reply in variants_of::<BootstrapReply>(json!({ "resp": "__none__" })) {
        // The reply plane's `hello` is a distinct frame from the request's.
        bootstrap.insert(if reply == "hello" {
            "hello_reply".into()
        } else {
            reply
        });
    }
    pin(
        "bootstrap",
        &bootstrap,
        &[
            (
                "hello",
                serde_json::to_value(Bootstrap::Hello {
                    version: "0.0.1".into(),
                    contract: "0123456789ab".into(),
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
                    version: "0.0.1".into(),
                    contract: "0123456789ab".into(),
                    artifact: Some(artifact()),
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

    // A future client's hello: extra fields, a field reshaped, none of
    // this build's fields at all (a client from before the identity split
    // sent `client_version` alone).
    assert_eq!(
        Bootstrap::read(br#"{"req":"hello","contract":{"rev":"x"},"world":"/w"}"#),
        Some(Bootstrap::Hello {
            version: UNKNOWN.into(),
            contract: UNKNOWN.into(),
        })
    );
    assert_eq!(
        Bootstrap::read(br#"{"req":"hello","client_version":"0.0.1 (runtime 0123456789ab)"}"#),
        Some(Bootstrap::Hello {
            version: UNKNOWN.into(),
            contract: UNKNOWN.into(),
        })
    );
    assert_eq!(
        Bootstrap::read(
            br#"{"req":"hello","version":"9.9.9","contract":"ffffffffffff","extra":1}"#
        ),
        Some(Bootstrap::Hello {
            version: "9.9.9".into(),
            contract: "ffffffffffff".into(),
        })
    );
    assert_eq!(
        Bootstrap::read(br#"{"req":"daemon_stop","reason":"upgrade"}"#),
        Some(Bootstrap::DaemonStop)
    );
    assert_eq!(Bootstrap::read(br#"{"req":"list"}"#), None);
    assert_eq!(Bootstrap::read(b"not json"), None);
    assert_eq!(Bootstrap::read(br#"{"resp":"hello"}"#), None);

    // A future daemon's hello: identified as foreign, never unparsed —
    // with the artifact read when it is whole, dropped when it is not.
    assert_eq!(
        BootstrapReply::read(
            br#"{"resp":"hello","version":"9.9.9","contract":"ffffffffffff","artifact":{"path":"/w/acqd","len":1,"mtime_ns":2,"dev":3,"ino":4,"sha256":"ab","future":true},"pid":7,"provider":"ggg","world":"/w"}"#
        ),
        Some(BootstrapReply::Hello {
            version: "9.9.9".into(),
            contract: "ffffffffffff".into(),
            artifact: Some(Artifact {
                file: FileIdentity {
                    path: "/w/acqd".into(),
                    len: 1,
                    mtime_ns: 2,
                    dev: 3,
                    ino: 4,
                },
                sha256: "ab".into(),
            }),
            pid: 7,
            provider: "ggg".into(),
        })
    );
    assert_eq!(
        BootstrapReply::read(
            br#"{"resp":"hello","version":"9.9.9","contract":"ffffffffffff","artifact":{"sha256":"ab"},"pid":7,"provider":"ggg"}"#
        ),
        Some(BootstrapReply::Hello {
            version: "9.9.9".into(),
            contract: "ffffffffffff".into(),
            artifact: None,
            pid: 7,
            provider: "ggg".into(),
        })
    );
    // A daemon from before the identity split: `daemon_version` alone.
    assert_eq!(
        BootstrapReply::read(
            br#"{"resp":"hello","daemon_version":"0.0.1 (runtime 0123456789ab)","pid":7,"provider":"mock"}"#
        ),
        Some(BootstrapReply::Hello {
            version: UNKNOWN.into(),
            contract: UNKNOWN.into(),
            artifact: None,
            pid: 7,
            provider: "mock".into(),
        })
    );
    assert_eq!(
        BootstrapReply::read(br#"{"resp":"hello"}"#),
        Some(BootstrapReply::Hello {
            version: UNKNOWN.into(),
            contract: UNKNOWN.into(),
            artifact: None,
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
        r#"{"req":"hello","version":"x","contract":"y"}"#,
        r#"{"req":"daemon_stop"}"#,
    ] {
        assert!(serde_json::from_str::<Request>(frame).is_err(), "{frame}");
    }
    for frame in [
        r#"{"resp":"hello","version":"x","contract":"y","pid":1,"provider":"mock"}"#,
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
    let listed: BTreeSet<String> = names
        .iter()
        .map(|n| n.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        listed,
        variants_of::<ErrorKind>(json!("__none__")),
        "ErrorKind::ALL is not the type's variants"
    );
    assert_eq!(listed.len(), ErrorKind::ALL.len(), "a kind listed twice");
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
