//! The contract between the client and the daemon, black-box (C85): a
//! daemon process on its own socket, driven through the public client —
//! `Client`, `Subscription`, `stop_any` — and through raw frames where the
//! property is about what the daemon does with a frame the client would
//! never send. Bootstrap across a contract mismatch, stop across one, a
//! subscription's events-only discipline and its lag signal, malformed and
//! oversize frames, the disconnect and the restart, and the closed error
//! kinds on the domain failures a client can stage.
//!
//! The daemon is the `acqd` the quality gate built (`cargo build
//! --workspace` before `cargo test`, AGENTS.md): this crate has no binary
//! of its own and may not link the daemon, and a test executable lives in
//! `target/<profile>/deps/`, whose parent holds no daemon — so the tests
//! here start the one Cargo uplifts one level up, `target/<profile>/acqd`,
//! resolved from the test executable's own location (`acqd_for_tests`).
//! Not the locator's rule (C82: the sibling of the calling executable) —
//! a test that owns a daemon names its executable, as the live drivers
//! do — and not a knob: nothing production reads it. A missing daemon
//! fails before any test runs, naming the build step; a present one is
//! named by path in every failure.
//!
//! The client reads the socket from `ACQ_SOCKET`, a process-wide setting,
//! so the tests here run one at a time under a lock and set their own
//! scratch socket and store while they hold it. Nothing here reaches GGG:
//! `ACQ_GGG` is scrubbed and the daemon runs the mock provider.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

use acquisition_client::client::{Client, ConnectOptions, Observed, Signal, Subscription};
use acquisition_client::frame::{Frame, read_frame};
use acquisition_client::socket_path;
use acquisition_protocol::VERSION_WITH_RUNTIME;
use acquisition_protocol::job::JobState;
use acquisition_protocol::protocol::{ErrorKind, MAX_FRAME_BYTES, Request, Response};
use serde_json::{Value, json};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{UnixListener, UnixStream};

/// The `acqd` the gate built (module doc): `target/<profile>/acqd`, one
/// level above the `deps/` directory this test executable runs from.
fn acqd_for_tests() -> PathBuf {
    let exe = std::env::current_exe().expect("current exe");
    let profile_dir = exe
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("{} is not under target/<profile>/deps", exe.display()));
    let acqd = profile_dir.join("acqd");
    assert!(
        acqd.is_file(),
        "no daemon at {}: the contract tests drive the acqd the workspace build writes there — \
         run `cargo build --workspace` first (the gate builds before it tests; a `-p` test run \
         does not build it)",
        acqd.display()
    );
    acqd
}

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// A scratch socket and store, held for the test's duration: the process
/// environment points at them while the lock is held.
struct Session {
    _lock: MutexGuard<'static, ()>,
    base: PathBuf,
}

fn session(tag: &str) -> Session {
    let lock = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    // Short: Unix socket paths cap near 104 bytes and the temp dir is long.
    let base = std::env::temp_dir().join(format!("acq-c{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("scratch dir");
    // SAFETY: every test in this binary takes `ONE_AT_A_TIME` before
    // touching the environment or the client, so no other thread reads
    // these variables while they change.
    unsafe {
        std::env::set_var("ACQ_SOCKET", base.join("d.sock"));
        std::env::set_var("ACQ_STORE_DIR", base.join("store"));
        std::env::set_var("ACQ_NO_KEYRING", "1");
        std::env::set_var("ACQ_JOURNAL", "0");
        std::env::set_var("ACQ_IDLE_SHUTDOWN", "30");
        for var in [
            "ACQ_GGG",
            "ACQ_ACCOUNT",
            "ACQ_TRIPWIRE",
            "ACQ_MAX_SENDS",
            "ACQ_NO_SPAWN",
        ] {
            std::env::remove_var(var);
        }
    }
    Session { _lock: lock, base }
}

impl Drop for Session {
    /// The scratch store goes with the test; the daemon, declared after
    /// the session, is already gone by then.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

/// The mock daemon, killed on drop if a failed assertion leaves it behind.
struct Daemon(Child);

impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Daemon {
    fn pid(&self) -> u32 {
        self.0.id()
    }

    /// Wait for the process to exit on its own (after a stop).
    fn wait_exit(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if self.0.try_wait().expect("try_wait").is_some() {
                return;
            }
            assert!(Instant::now() < deadline, "the daemon did not exit");
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

async fn start_daemon(s: &Session) -> Daemon {
    let acqd = acqd_for_tests();
    let child = Command::new(&acqd)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|e| panic!("spawning {}: {e}", acqd.display()));
    let daemon = Daemon(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match Client::observe().await {
            Ok(Observed::Compatible(_)) => return daemon,
            Ok(Observed::Incompatible(found)) => {
                panic!("{} is not this build's daemon: {found}", acqd.display())
            }
            _ => {}
        }
        assert!(
            Instant::now() < deadline,
            "the daemon ({}) did not come up on {}",
            acqd.display(),
            s.base.display()
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

async fn compatible_client() -> Client {
    match Client::observe().await.expect("observe") {
        Observed::Compatible(client) => client,
        Observed::Absent => panic!("no daemon"),
        Observed::Incompatible(found) => panic!("{found}"),
    }
}

async fn open_subscription() -> Subscription {
    match Subscription::observe().await.expect("observe") {
        Observed::Compatible(subscription) => subscription,
        Observed::Absent => panic!("no daemon"),
        Observed::Incompatible(found) => panic!("{found}"),
    }
}

/// A raw framed connection: what the client never sends, sent anyway.
struct Raw {
    reader: BufReader<OwnedReadHalf>,
    write: OwnedWriteHalf,
}

impl Raw {
    async fn open() -> Raw {
        let stream = UnixStream::connect(socket_path()).await.expect("connect");
        let (read, write) = stream.into_split();
        Raw {
            reader: BufReader::new(read),
            write,
        }
    }

    async fn send(&mut self, bytes: &[u8]) {
        self.write.write_all(bytes).await.expect("write");
        self.write.write_all(b"\n").await.expect("write");
    }

    async fn next(&mut self) -> Value {
        match read_frame(&mut self.reader, MAX_FRAME_BYTES)
            .await
            .expect("read")
        {
            Frame::Line(bytes) => serde_json::from_slice(&bytes)
                .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&bytes))),
            other => panic!("{other:?}"),
        }
    }

    async fn ask(&mut self, frame: Value) -> Value {
        self.send(frame.to_string().as_bytes()).await;
        self.next().await
    }

    async fn hello(&mut self) -> Value {
        let reply = self
            .ask(json!({ "req": "hello", "client_version": VERSION_WITH_RUNTIME }))
            .await;
        assert_eq!(reply["resp"], "hello", "{reply}");
        reply
    }
}

fn error_kind(frame: &Value) -> ErrorKind {
    assert_eq!(frame["resp"], "error", "{frame}");
    assert!(
        frame["message"].as_str().is_some_and(|m| !m.is_empty()),
        "an error carries a message: {frame}"
    );
    serde_json::from_value(frame["kind"].clone()).unwrap_or_else(|e| panic!("{e}: {frame}"))
}

async fn submit(client: &mut Client, kind: &str, params: Value, account: Option<&str>) -> Response {
    client
        .request(&Request::Submit {
            kind: kind.into(),
            params,
            priority: 0,
            submitted_by: "contract".into(),
            account: account.map(str::to_string),
        })
        .await
        .expect("submit")
}

fn submitted(response: Response) -> u64 {
    match response {
        Response::Submitted { id } => id,
        other => panic!("{other:?}"),
    }
}

fn kind_of(response: Response) -> (ErrorKind, String) {
    match response {
        Response::Error { kind, message } => (kind, message),
        other => panic!("{other:?}"),
    }
}

/// Poll `list` until every job this daemon holds is terminal.
async fn wait_all_terminal(client: &mut Client, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        let jobs = match client.request(&Request::List).await.expect("list") {
            Response::Jobs { jobs } => jobs,
            other => panic!("{other:?}"),
        };
        if jobs.len() == expected && jobs.iter().all(|j| j.state.is_terminal()) {
            return;
        }
        assert!(Instant::now() < deadline, "jobs did not finish");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// ---- the bootstrap plane --------------------------------------------------

/// The daemon side: a hello of any shape is answered with this daemon's
/// identity, a request it cannot read is `bad_request` with the
/// connection kept, and a `daemon_stop` of any shape stops it.
#[tokio::test]
async fn c85_the_daemon_identifies_itself_and_stops_across_a_contract_mismatch() {
    let s = session("boot");
    let mut daemon = start_daemon(&s).await;
    let mut raw = Raw::open().await;

    // Every connection begins with hello: a versioned request before it
    // performs nothing, and the connection waits for the greeting.
    let reply = raw.ask(json!({ "req": "list" })).await;
    assert_eq!(error_kind(&reply), ErrorKind::BadRequest, "{reply}");
    assert!(
        reply["message"].as_str().unwrap().contains("hello"),
        "{reply}"
    );

    // A client of a future revision: a foreign version, fields this build
    // has never seen — and still the daemon names itself.
    let reply = raw
        .ask(json!({
            "req": "hello",
            "client_version": "9.9.9 (runtime ffffffffffff)",
            "contract": { "rev": "abc" },
            "world": "/future",
        }))
        .await;
    assert_eq!(reply["resp"], "hello");
    assert_eq!(reply["daemon_version"], VERSION_WITH_RUNTIME);
    assert_eq!(reply["provider"], "mock");
    assert_eq!(reply["pid"], daemon.pid());
    assert_eq!(reply.as_object().unwrap().len(), 4, "{reply}");

    // A hello with nothing but its name.
    let reply = raw.ask(json!({ "req": "hello" })).await;
    assert_eq!(reply["daemon_version"], VERSION_WITH_RUNTIME);

    // A request this version does not know is refused, not fatal.
    let reply = raw.ask(json!({ "req": "frobnicate", "x": 1 })).await;
    assert_eq!(error_kind(&reply), ErrorKind::BadRequest);
    let reply = raw.ask(json!({ "req": "list" })).await;
    assert_eq!(reply["resp"], "jobs", "the connection survived: {reply}");

    // A stop of a future shape stops this daemon.
    let reply = raw
        .ask(json!({ "req": "daemon_stop", "reason": "contract test", "world": "/future" }))
        .await;
    assert_eq!(reply, json!({ "resp": "stopping" }));
    daemon.wait_exit();
    assert!(!socket_path().exists(), "the socket is removed on exit");
}

/// A scripted daemon of a future revision on the session's socket: answers
/// every hello with `hello_reply`, every `daemon_stop` with `stop_reply`,
/// anything else with an error of a kind this build does not know.
async fn foreign_daemon(hello_reply: Value, stop_reply: Value) -> tokio::task::JoinHandle<()> {
    let path = socket_path();
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).expect("bind");
    tokio::spawn(async move {
        loop {
            let (stream, _) = listener.accept().await.expect("accept");
            let (read, mut write) = stream.into_split();
            let mut reader = BufReader::new(read);
            let hello_reply = hello_reply.clone();
            let stop_reply = stop_reply.clone();
            tokio::spawn(async move {
                while let Ok(Frame::Line(bytes)) = read_frame(&mut reader, MAX_FRAME_BYTES).await {
                    let frame: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
                    let reply = match frame["req"].as_str() {
                        Some("hello") => hello_reply.clone(),
                        Some("daemon_stop") => stop_reply.clone(),
                        _ => {
                            json!({ "resp": "error", "kind": "future_kind", "message": "unknown to 9.9.9" })
                        }
                    };
                    let mut line = reply.to_string();
                    line.push('\n');
                    if write.write_all(line.as_bytes()).await.is_err() {
                        break;
                    }
                }
            });
        }
    })
}

/// The client side: a daemon whose frames carry fields this build has
/// never seen is identified as another runtime — reported, not used —
/// and `stop_any` stops it; a stop it refuses is reported by message.
#[tokio::test]
async fn c85_a_client_identifies_and_stops_a_foreign_daemon_across_a_contract_mismatch() {
    let _s = session("foreign");
    let hello = json!({
        "resp": "hello",
        "daemon_version": "9.9.9 (runtime ffffffffffff)",
        "pid": 4242,
        "provider": "mock",
        "world": "/future",
        "contract": { "rev": "abc" },
    });
    let peer = foreign_daemon(
        hello.clone(),
        json!({ "resp": "stopping", "world": "/future" }),
    )
    .await;

    let found = match Client::observe().await.expect("observe") {
        Observed::Incompatible(found) => found,
        Observed::Absent => panic!("absent"),
        Observed::Compatible(_) => panic!("a foreign daemon is never this client's"),
    };
    assert_eq!(found.version, "9.9.9 (runtime ffffffffffff)");
    assert_eq!(found.pid, 4242);
    assert!(
        !found.identity_matches() && found.provider_matches(),
        "{found}"
    );

    let err = match Client::connect(ConnectOptions {
        spawn: false,
        replace: false,
    })
    .await
    {
        Err(e) => e.to_string(),
        Ok(_) => panic!("a foreign daemon is never used"),
    };
    assert!(err.contains("another runtime"), "{err}");

    let stopped = Client::stop_any().await.expect("stop").expect("a daemon");
    assert_eq!(stopped.pid, 4242);
    peer.abort();

    // The same daemon refusing the stop with an error of a future kind:
    // the message reaches the human; nothing counts as a stop.
    let peer = foreign_daemon(
        hello,
        json!({ "resp": "error", "kind": "future_kind", "message": "not while a world lock is held" }),
    )
    .await;
    let err = Client::stop_any().await.unwrap_err().to_string();
    assert!(
        err.contains("refused to stop") && err.contains("world lock"),
        "{err}"
    );
    peer.abort();
    let _ = std::fs::remove_file(socket_path());
}

// ---- frames ---------------------------------------------------------------

/// Malformed and oversize frames get `bad_request` and the connection
/// stays; frames are answered in order, one at a time; a blank line is
/// nothing.
#[tokio::test]
async fn c85_malformed_and_oversize_frames_get_bad_request_and_the_connection_stays() {
    let s = session("frames");
    let _daemon = start_daemon(&s).await;
    let mut raw = Raw::open().await;
    raw.hello().await;

    let malformed: [&[u8]; 6] = [
        b"not json",
        b"[1, 2]",
        br#"{"req":"submit"}"#,
        br#"{"req":"cancel","id":"forty-two"}"#,
        br#"{"resp":"ack"}"#,
        b"\xff\xfe{}",
    ];
    for frame in malformed {
        raw.send(frame).await;
        let reply = raw.next().await;
        assert_eq!(
            error_kind(&reply),
            ErrorKind::BadRequest,
            "{}: {reply}",
            String::from_utf8_lossy(frame)
        );
    }
    let reply = raw.ask(json!({ "req": "list" })).await;
    assert_eq!(reply["resp"], "jobs", "{reply}");

    // Over the bound by one byte, and valid JSON all the way: discarded
    // through its newline, answered, and the next frame is read intact.
    let head = br#"{"req":"list","pad":""#;
    let tail = br#""}"#;
    let mut oversize = Vec::with_capacity(MAX_FRAME_BYTES + 1);
    oversize.extend_from_slice(head);
    oversize.resize(MAX_FRAME_BYTES + 1 - tail.len(), b'x');
    oversize.extend_from_slice(tail);
    assert_eq!(oversize.len(), MAX_FRAME_BYTES + 1);
    raw.send(&oversize).await;
    let reply = raw.next().await;
    assert_eq!(error_kind(&reply), ErrorKind::BadRequest, "{reply}");
    assert!(
        reply["message"].as_str().unwrap().contains("exceeds"),
        "{reply}"
    );
    let reply = raw.ask(json!({ "req": "list" })).await;
    assert_eq!(reply["resp"], "jobs", "{reply}");

    // Exactly the bound is a frame.
    let mut at_bound = Vec::with_capacity(MAX_FRAME_BYTES);
    at_bound.extend_from_slice(head);
    at_bound.resize(MAX_FRAME_BYTES - tail.len(), b'x');
    at_bound.extend_from_slice(tail);
    raw.send(&at_bound).await;
    let reply = raw.next().await;
    assert_eq!(reply["resp"], "jobs", "{reply}");

    // Blank lines are skipped; pipelined frames are answered in order.
    raw.send(b"").await;
    raw.send(br#"{"req":"list"}"#).await;
    raw.send(br#"{"req":"daemon_status"}"#).await;
    assert_eq!(raw.next().await["resp"], "jobs");
    assert_eq!(raw.next().await["resp"], "daemon_status");
}

// ---- subscriptions ----------------------------------------------------------

/// After `subscribed` a connection carries events only — a request there
/// is `bad_request`, the bootstrap plane still answers — and a subscriber
/// the daemon's event channel overran is told `resync_required` with the
/// count, after which the snapshot over a request connection is whole.
#[tokio::test]
async fn c85_a_subscription_carries_events_only_and_lag_is_resync_required() {
    let s = session("lag");
    let _daemon = start_daemon(&s).await;

    let mut raw = Raw::open().await;
    raw.hello().await;
    assert_eq!(
        raw.ask(json!({ "req": "subscribe" })).await,
        json!({ "resp": "subscribed" })
    );
    let reply = raw.ask(json!({ "req": "list" })).await;
    assert_eq!(error_kind(&reply), ErrorKind::BadRequest, "{reply}");
    assert!(
        reply["message"].as_str().unwrap().contains("subscription"),
        "{reply}"
    );
    assert_eq!(
        raw.hello().await["provider"],
        "mock",
        "the bootstrap plane works on any connection"
    );
    drop(raw);

    // Subscribe, then snapshot (the pinned order), then leave the
    // subscription unread while the daemon runs more jobs than its event
    // channel holds: the socket fills, the receiver falls behind, and the
    // gap is reported — never skipped.
    let mut subscription = open_subscription().await;
    let mut client = compatible_client().await;
    assert!(
        matches!(client.request(&Request::List).await.unwrap(), Response::Jobs { jobs } if jobs.is_empty())
    );
    const JOBS: usize = 800;
    for _ in 0..JOBS {
        submitted(submit(&mut client, "sleep", json!({ "seconds": 0 }), None).await);
    }
    wait_all_terminal(&mut client, JOBS).await;

    let mut events_before = 0usize;
    let missed = loop {
        match tokio::time::timeout(Duration::from_secs(10), subscription.next()).await {
            Ok(Ok(Some(Signal::Event(_)))) => events_before += 1,
            Ok(Ok(Some(Signal::ResyncRequired { missed }))) => break missed,
            Ok(other) => panic!("{other:?}"),
            Err(_) => panic!(
                "no resync_required after {events_before} events of {} — the channel never overran",
                JOBS * 3
            ),
        }
    };
    assert!(missed >= 1, "{missed}");
    assert!(
        events_before + (missed as usize) <= JOBS * 3,
        "{events_before} seen + {missed} missed exceeds the {} emitted",
        JOBS * 3
    );
    // The snapshot is the authority; the stream was never complete.
    let jobs = match client.request(&Request::List).await.unwrap() {
        Response::Jobs { jobs } => jobs,
        other => panic!("{other:?}"),
    };
    assert_eq!(jobs.len(), JOBS);
    assert!(jobs.iter().all(|j| j.state == JobState::Done));

    // The rest of the sequence: the lagged subscription is dropped — what
    // it still holds predates the snapshot — and a fresh one carries only
    // what happens after it was opened.
    drop(subscription);
    let mut subscription = open_subscription().await;
    let jobs = match client.request(&Request::List).await.unwrap() {
        Response::Jobs { jobs } => jobs,
        other => panic!("{other:?}"),
    };
    let newest = jobs.iter().map(|j| j.id).max().unwrap();
    let next = submitted(submit(&mut client, "sleep", json!({ "seconds": 0 }), None).await);
    assert!(next > newest);
    let first = tokio::time::timeout(Duration::from_secs(10), subscription.next())
        .await
        .expect("an event")
        .expect("open");
    assert!(
        matches!(&first, Some(Signal::Event(job)) if job.id == next),
        "the fresh subscription's first event is the new job, not a leftover: {first:?}"
    );
}

/// A subscriber sees the daemon go as the end of its stream, the request
/// connection fails on its next request, and after a restart a new
/// subscription and a new snapshot work — with the old job's result still
/// answered (C27).
#[tokio::test]
async fn c85_a_subscriber_sees_the_disconnect_and_subscribes_and_snapshots_again_after_a_restart() {
    let s = session("restart");
    let mut daemon = start_daemon(&s).await;
    let mut subscription = open_subscription().await;
    let mut client = compatible_client().await;

    let id = submitted(submit(&mut client, "sleep", json!({ "seconds": 0 }), None).await);
    wait_all_terminal(&mut client, 1).await;
    let mut done = false;
    while !done {
        match tokio::time::timeout(Duration::from_secs(10), subscription.next())
            .await
            .expect("an event")
            .expect("open")
        {
            Some(Signal::Event(job)) => done = job.id == id && job.state == JobState::Done,
            other => panic!("{other:?}"),
        }
    }

    let stopped = Client::stop_any().await.expect("stop").expect("a daemon");
    assert_eq!(stopped.pid, daemon.pid());
    daemon.wait_exit();
    let end = tokio::time::timeout(Duration::from_secs(10), subscription.next())
        .await
        .expect("the stream ends");
    assert!(matches!(end, Ok(None) | Err(_)), "{end:?}");
    assert!(
        client.request(&Request::List).await.is_err(),
        "the request connection is gone"
    );
    assert!(matches!(Client::observe().await.unwrap(), Observed::Absent));

    let _daemon = start_daemon(&s).await;
    let mut subscription = open_subscription().await;
    let mut client = compatible_client().await;
    let live = match client.request(&Request::List).await.unwrap() {
        Response::Jobs { jobs } => jobs,
        other => panic!("{other:?}"),
    };
    assert!(
        live.is_empty(),
        "terminal jobs are history, not state: {live:?}"
    );
    assert!(matches!(
        client.request(&Request::Result { id }).await.unwrap(),
        Response::Result { id: got, .. } if got == id
    ));
    let next = submitted(submit(&mut client, "sleep", json!({ "seconds": 0 }), None).await);
    assert!(next > id, "ids continue across the restart");
    let event = tokio::time::timeout(Duration::from_secs(10), subscription.next())
        .await
        .expect("an event")
        .expect("open");
    assert!(
        matches!(&event, Some(Signal::Event(job)) if job.id == next),
        "{event:?}"
    );
}

// ---- error kinds ------------------------------------------------------------

/// Every domain failure a client can stage carries its kind, and none of
/// them is `internal`. (`queue_failed`, `upstream` and `internal` need a
/// fault the wire cannot inject; the daemon's unit tests stage those.)
#[tokio::test]
async fn c85_error_kinds_classify_the_domain_failures_a_client_can_stage() {
    let s = session("kinds");
    let _daemon = start_daemon(&s).await;
    let mut client = compatible_client().await;

    let (kind, message) = kind_of(
        submit(
            &mut client,
            "stashes",
            json!({ "league": "Standard" }),
            None,
        )
        .await,
    );
    assert_eq!(kind, ErrorKind::NotLoggedIn);
    assert!(message.contains("acq auth"), "{message}");
    let (kind, _) = kind_of(submit(&mut client, "stashes", json!({}), Some("nobody")).await);
    assert_eq!(kind, ErrorKind::NotLoggedIn);
    let (kind, _) = kind_of(
        client
            .request(&Request::AuthCheck { account: None })
            .await
            .unwrap(),
    );
    assert_eq!(kind, ErrorKind::NotLoggedIn);
    let (kind, _) = kind_of(
        client
            .request(&Request::Quote {
                jobs: vec![],
                account: Some("nobody".into()),
            })
            .await
            .unwrap(),
    );
    assert_eq!(kind, ErrorKind::NotLoggedIn);

    for request in [
        Request::Status { id: 999 },
        Request::Result { id: 999 },
        Request::Cancel { id: 999 },
        Request::SetPriority {
            id: 999,
            priority: 1,
        },
    ] {
        let (kind, message) = kind_of(client.request(&request).await.unwrap());
        assert_eq!(kind, ErrorKind::UnknownJob, "{request:?}");
        assert_eq!(message, "no job 999");
    }

    let id = submitted(submit(&mut client, "sleep", json!({ "seconds": 30 }), None).await);
    let (kind, message) = kind_of(client.request(&Request::Result { id }).await.unwrap());
    assert_eq!(kind, ErrorKind::WrongState);
    assert!(message.contains("still"), "{message}");
    assert!(matches!(
        client.request(&Request::Cancel { id }).await.unwrap(),
        Response::Ack
    ));
    // A running job is cancelled at its next slice; until then a second
    // cancel is the same request again, not a wrong state.
    wait_all_terminal(&mut client, 1).await;
    let (kind, message) = kind_of(client.request(&Request::Cancel { id }).await.unwrap());
    assert_eq!(kind, ErrorKind::WrongState);
    assert!(message.contains("already"), "{message}");
    let (kind, _) = kind_of(
        client
            .request(&Request::SetPriority { id, priority: 1 })
            .await
            .unwrap(),
    );
    assert_eq!(kind, ErrorKind::WrongState);

    // Admission refusals need a session: the mock's login page accepts any
    // name, and the flow completes once its own profile job lands (C50).
    login(&mut client, None).await;
    let (kind, message) = kind_of(
        submit(
            &mut client,
            "apply",
            json!({ "jobs": [{ "kind": "sleep", "params": {} }] }),
            None,
        )
        .await,
    );
    assert_eq!(kind, ErrorKind::Refused);
    assert!(message.contains("vocabulary"), "{message}");
    let (kind, message) = kind_of(
        submit(
            &mut client,
            "apply",
            json!({ "jobs": [{ "kind": "stashes", "params": {} }], "max_requests": 0 }),
            None,
        )
        .await,
    );
    assert_eq!(kind, ErrorKind::Refused);
    assert!(message.contains("budget"), "{message}");
    let (kind, message) = kind_of(
        submit(
            &mut client,
            "stashes",
            json!({ "realm": "ps5", "league": "Standard" }),
            None,
        )
        .await,
    );
    assert_eq!(kind, ErrorKind::Refused);
    assert!(message.contains("ps5"), "{message}");

    // Several sessions and no selector: ambiguous, never guessed. A bare
    // name two of them share is ambiguous too (C51; review finding
    // 2026-09-10); the full name selects.
    login(&mut client, Some("Alice#1234")).await;
    login(&mut client, Some("Alice#5678")).await;
    let (kind, message) = kind_of(
        submit(
            &mut client,
            "stashes",
            json!({ "league": "Standard" }),
            None,
        )
        .await,
    );
    assert_eq!(kind, ErrorKind::AmbiguousAccount);
    assert!(message.contains("--account"), "{message}");
    let (kind, _) = kind_of(
        client
            .request(&Request::AuthLogout { account: None })
            .await
            .unwrap(),
    );
    assert_eq!(kind, ErrorKind::AmbiguousAccount);
    for request in [
        Request::Submit {
            kind: "stashes".into(),
            params: json!({ "league": "Standard" }),
            priority: 0,
            submitted_by: "contract".into(),
            account: Some("alice".into()),
        },
        Request::Quote {
            jobs: vec![],
            account: Some("alice".into()),
        },
        Request::AuthCheck {
            account: Some("alice".into()),
        },
        Request::AuthLogout {
            account: Some("alice".into()),
        },
    ] {
        let (kind, message) = kind_of(client.request(&request).await.unwrap());
        assert_eq!(kind, ErrorKind::AmbiguousAccount, "{request:?}");
        assert!(message.contains("Alice#1234, Alice#5678"), "{message}");
    }
    submitted(
        submit(
            &mut client,
            "stashes",
            json!({ "league": "Standard" }),
            Some("Alice#5678"),
        )
        .await,
    );
}

/// Log in through the mock provider's page: start the flow, approve it as
/// `user` (the mock accepts any name), and wait for the daemon to
/// register the session.
async fn login(client: &mut Client, user: Option<&str>) {
    let url = match client.request(&Request::AuthStart).await.unwrap() {
        Response::AuthUrl { authorize_url } => authorize_url,
        other => panic!("{other:?}"),
    };
    let mut approve = url.replacen("/authorize?", "/approve?", 1);
    if let Some(user) = user {
        approve.push_str(&format!("&user={}", user.replace('#', "%23")));
    }
    // reqwest follows the redirect to the daemon's callback, which is what
    // completes the login.
    let response = reqwest::get(&approve).await.expect("approve");
    assert!(response.status().is_success(), "{}", response.status());
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match client.request(&Request::AuthStatus).await.unwrap() {
            Response::Auth {
                pending: false,
                login_ok: Some(_),
                ..
            } => return,
            Response::Auth {
                login_error: Some(e),
                pending: false,
                ..
            } => panic!("login failed: {e}"),
            _ => {}
        }
        assert!(Instant::now() < deadline, "login did not complete");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
