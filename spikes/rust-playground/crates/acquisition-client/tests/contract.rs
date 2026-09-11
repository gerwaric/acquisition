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
//! C82's rule for a test executable, which has no sibling: it names the
//! `acqd` its build wrote (amended 2026-09-10). Not a knob:
//! nothing production reads it. Since the identity split's step 4 the
//! client judges a daemon's artifact against the sibling the locator
//! names (C84), and this executable has none — so `acqd_for_tests` also
//! places a symlink named `acqd` beside the test executable, pointing at
//! that same `target/<profile>/acqd`: the test executable then names the
//! daemon its build wrote on both paths, the harness's and the client's,
//! and the locator's rule stays the one rule. A missing daemon fails
//! before any test runs, naming the build step; a present one is named
//! by path in every failure — the startup failures say it outright, and
//! `Daemon`'s drop prints it while a test is panicking, so a failed
//! assertion's output carries which daemon ran.
//!
//! The client reads the world from `ACQ_STORE_DIR` and derives the
//! socket from it (C83) into the runtime directory, process-wide
//! settings, so the tests here run one at a time under a lock and set
//! their own scratch store and log directory while they hold it — and a
//! scratch temp and runtime directory (`TMPDIR`, `XDG_RUNTIME_DIR`), so
//! the sockets these daemons bind, the legacy rendezvous a test stages
//! and the real-mode lock never touch the user's own; the store root is
//! created with the session (a scripted peer has no daemon to create it,
//! and the world it claims must exist to be this process's). Nothing
//! here reaches GGG: `ACQ_GGG` is scrubbed and the daemon runs the mock
//! provider.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use acquisition_client::client::{
    Client, ConnectError, ConnectOptions, NotReplaced, Observed, Signal, Subscription,
};
use acquisition_client::frame::{Frame, read_frame};
use acquisition_protocol::job::JobState;
use acquisition_protocol::protocol::{ErrorKind, MAX_FRAME_BYTES, Request, Response};
use acquisition_protocol::{CONTRACT_REVISION, VERSION};
use acquisition_store::world::{World, legacy_socket_path};
use serde_json::{Value, json};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{UnixListener, UnixStream};

/// The `acqd` the gate built (module doc): `target/<profile>/acqd`, one
/// level above the `deps/` directory this test executable runs from —
/// and, so the client in this process finds the same file as its
/// sibling (C84 judges the artifact against it), a symlink of that name
/// beside the test executable pointing one level up.
fn acqd_for_tests() -> PathBuf {
    let exe = std::env::current_exe().expect("current exe");
    let deps_dir = exe
        .parent()
        .unwrap_or_else(|| panic!("{} has no parent directory", exe.display()));
    let profile_dir = deps_dir
        .parent()
        .unwrap_or_else(|| panic!("{} is not under target/<profile>/deps", exe.display()));
    let acqd = profile_dir.join("acqd");
    assert!(
        acqd.is_file(),
        "no daemon at {}: the contract tests drive the acqd the workspace build writes there — \
         run `cargo build --workspace` first (the gate builds before it tests; a `-p` test run \
         does not build it)",
        acqd.display()
    );
    let link = deps_dir.join("acqd");
    let target = Path::new("..").join("acqd");
    if std::fs::read_link(&link).ok().as_deref() != Some(target.as_path()) {
        let _ = std::fs::remove_file(&link);
        std::os::unix::fs::symlink(&target, &link)
            .unwrap_or_else(|e| panic!("placing {} -> {}: {e}", link.display(), target.display()));
    }
    let sibling = acquisition_client::locator::beside(&exe).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        sibling.canonicalize().expect("the sibling resolves"),
        acqd.canonicalize().expect("the daemon resolves"),
        "the sibling the client sees is not the daemon the harness starts"
    );
    acqd
}

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// The temp directory as the process started, read once under the lock:
/// every session redirects `TMPDIR` into its scratch, and the next
/// scratch must not nest under the last.
static ORIGINAL_TMP: OnceLock<PathBuf> = OnceLock::new();

/// A scratch store, temp and runtime directory, held for the test's
/// duration: the process environment points at them while the lock is
/// held.
struct Session {
    _lock: MutexGuard<'static, ()>,
    base: PathBuf,
}

fn session(tag: &str) -> Session {
    let lock = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let original = ORIGINAL_TMP.get_or_init(std::env::temp_dir);
    let base = original.join(format!("acq-c{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("store")).expect("scratch dir");
    std::fs::create_dir_all(base.join("tmp")).expect("scratch dir");
    // SAFETY: every test in this binary takes `ONE_AT_A_TIME` before
    // touching the environment or the client, so no other thread reads
    // these variables while they change.
    unsafe {
        std::env::set_var("ACQ_STORE_DIR", base.join("store"));
        std::env::set_var("ACQ_LOG_DIR", base.join("logs"));
        // The runtime directory (the sockets, the real-mode lock) and the
        // legacy paths under the scratch, on either platform.
        std::env::set_var("TMPDIR", base.join("tmp"));
        std::env::set_var("XDG_RUNTIME_DIR", base.join("tmp"));
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
    // The daemon's step, so a socket path resolves in this process too:
    // a client never creates the runtime directory (C83).
    acquisition_store::world::app_runtime_dir().expect("the scratch runtime directory");
    Session { _lock: lock, base }
}

impl Drop for Session {
    /// The scratch store goes with the test; the daemon, declared after
    /// the session, is already gone by then.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

/// The session's socket: derived from its world (C83) — what the daemon
/// binds and a scripted peer must bind to be found.
fn session_socket() -> PathBuf {
    World::observe()
        .expect("the session's world")
        .socket_path()
        .expect("its socket")
}

/// The mock daemon, killed on drop if a failed assertion leaves it
/// behind — and named then, so every failure says which daemon ran.
struct Daemon(Child, PathBuf);

impl Drop for Daemon {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!(
                "the daemon under test was {} (pid {})",
                self.1.display(),
                self.0.id()
            );
        }
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
    let daemon = Daemon(child, acqd.clone());
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
        let stream = UnixStream::connect(session_socket())
            .await
            .expect("connect");
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
            .ask(json!({ "req": "hello", "version": VERSION, "contract": CONTRACT_REVISION }))
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
    // has never seen — and still the daemon names itself, in every
    // dimension a client judges (C84): contract, artifact, provider.
    let reply = raw
        .ask(json!({
            "req": "hello",
            "version": "9.9.9",
            "contract": { "rev": "abc" },
            "world": "/future",
        }))
        .await;
    assert_eq!(reply["resp"], "hello");
    assert_eq!(reply["version"], VERSION);
    assert_eq!(reply["contract"], CONTRACT_REVISION);
    assert_eq!(reply["provider"], "mock");
    let artifact = &reply["artifact"];
    assert_eq!(
        Path::new(artifact["path"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        acqd_for_tests().canonicalize().unwrap(),
        "the daemon names the executable it runs from: {reply}"
    );
    let sha = artifact["sha256"].as_str().unwrap();
    assert!(
        sha.len() == 64 && sha.bytes().all(|b| b.is_ascii_hexdigit()),
        "{reply}"
    );
    assert_eq!(
        sha,
        acquisition_client::artifact::sha256_of(&acqd_for_tests()).unwrap(),
        "the reported hash is the file's"
    );
    assert!(
        artifact["len"].as_u64().unwrap() > 0 && artifact["ino"].as_u64().unwrap() > 0,
        "{reply}"
    );
    assert_eq!(reply["pid"], daemon.pid());
    // The world (C83): the daemon's canonical root, this session's store.
    assert_eq!(
        reply["world"],
        World::observe().expect("the session's world").name(),
        "{reply}"
    );
    // resp, version, contract, artifact, pid, provider, world.
    assert_eq!(reply.as_object().unwrap().len(), 7, "{reply}");

    // A hello with nothing but its name.
    let reply = raw.ask(json!({ "req": "hello" })).await;
    assert_eq!(reply["contract"], CONTRACT_REVISION);

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
    assert!(!session_socket().exists(), "the socket is removed on exit");
}

/// A scripted daemon of a future revision on the session's socket: answers
/// every hello with `hello_reply`, every `daemon_stop` with `stop_reply`,
/// anything else with an error of a kind this build does not know.
async fn foreign_daemon(hello_reply: Value, stop_reply: Value) -> tokio::task::JoinHandle<()> {
    foreign_daemon_at(session_socket(), hello_reply, stop_reply).await
}

/// `foreign_daemon` bound at `path`.
async fn foreign_daemon_at(
    path: PathBuf,
    hello_reply: Value,
    stop_reply: Value,
) -> tokio::task::JoinHandle<()> {
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
/// never seen is identified as another contract — reported, not used —
/// and `stop_any` stops it; a stop it refuses is reported by message.
#[tokio::test]
async fn c85_a_client_identifies_and_stops_a_foreign_daemon_across_a_contract_mismatch() {
    let _s = session("foreign");
    // On this session's world, so the mismatch reported is the contract's
    // (another world is judged first and reported alone: the next test).
    let hello = json!({
        "resp": "hello",
        "version": "9.9.9",
        "contract": "ffffffffffff",
        "artifact": { "rev": "abc" },
        "pid": 4242,
        "provider": "mock",
        "world": World::observe().expect("the session's world").name(),
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
    assert_eq!(found.version(), "9.9.9");
    assert_eq!(found.contract(), "ffffffffffff");
    assert_eq!(
        found.artifact(),
        None,
        "an unreadable artifact reads as none"
    );
    assert_eq!(found.pid(), 4242);
    assert!(
        !found.contract_matches() && found.provider_matches(),
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
    assert!(err.contains("another contract"), "{err}");

    let stopped = Client::stop_any().await.expect("stop").expect("a daemon");
    assert_eq!(stopped.pid(), 4242);
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
    let _ = std::fs::remove_file(session_socket());
}

/// C83 at the client's door. The socket derives from the world, so a
/// daemon on another world is not on this shell's socket at all: from
/// another root it is absent, and a shell whose own root does not exist
/// has no world, no socket, and creates nothing by looking. What the
/// wire still pins: a daemon that answers on *this* world's socket
/// claiming another root (a scripted peer) is judged on the world first
/// and reported alone; a use door that would replace any other mismatch
/// answers `OtherWorld` and leaves it running; `stop_any` still stops it.
#[tokio::test]
async fn c83_a_daemon_on_another_world_is_refused_and_never_replaced() {
    let s = session("world");
    let mut daemon = start_daemon(&s).await;
    let pid = daemon.pid();
    let home = World::observe().expect("the session's world");
    let home_socket = session_socket();

    // Another world is another socket: this shell moves to another root
    // and finds nothing there — the daemon's socket derives from `home`.
    let elsewhere = s.base.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    // SAFETY: the session lock is held (module doc).
    unsafe {
        std::env::set_var("ACQ_STORE_DIR", &elsewhere);
    }
    assert_ne!(
        session_socket(),
        home_socket,
        "another world, another socket"
    );
    assert!(
        matches!(Client::observe().await.expect("observe"), Observed::Absent),
        "a daemon on another world is not on this world's socket"
    );
    // A shell whose root does not exist has no world and no socket:
    // observation finds nothing and creates nothing.
    unsafe {
        std::env::set_var("ACQ_STORE_DIR", s.base.join("nowhere"));
    }
    assert!(matches!(
        Client::observe().await.expect("observe"),
        Observed::Absent
    ));
    assert!(
        !s.base.join("nowhere").exists(),
        "observation created the root"
    );

    // Back home: the same daemon, untouched, is this client's; stopped
    // from home, on its own socket.
    unsafe {
        std::env::set_var("ACQ_STORE_DIR", s.base.join("store"));
    }
    let client = compatible_client().await;
    assert_eq!(client.daemon().pid(), pid, "the daemon was replaced");
    assert_eq!(
        client.daemon().socket(),
        home_socket.to_str().unwrap(),
        "{}",
        client.daemon()
    );
    assert!(!client.daemon().on_legacy_socket());
    drop(client);
    let stopped = Client::stop_any().await.expect("stop").expect("a daemon");
    assert_eq!(stopped.pid(), pid);
    daemon.wait_exit();
    assert!(!home_socket.exists(), "the socket is removed on exit");

    // What the wire pins: a peer on this world's socket claiming another
    // root — the world judged first and named alone, never replaced by
    // the use door, stopped by `stop_any`.
    let peer = foreign_daemon(
        json!({
            "resp": "hello", "version": VERSION, "contract": CONTRACT_REVISION,
            "pid": 4242, "provider": "mock", "world": "/somewhere/else",
        }),
        json!({ "resp": "stopping" }),
    )
    .await;
    let found = match Client::observe().await.expect("observe") {
        Observed::Incompatible(found) => found,
        Observed::Absent => panic!("absent"),
        Observed::Compatible(_) => panic!("a daemon on another world is never this client's"),
    };
    assert_eq!(found.pid(), 4242);
    assert_eq!(found.world(), "/somewhere/else", "{found}");
    assert!(
        !found.world_matches() && found.contract_matches(),
        "{found}"
    );
    let report = found.report();
    assert_eq!(report["world_matches"], false, "{report}");
    assert_eq!(report["wanted"]["world"], home.name(), "{report}");
    assert_eq!(
        report["socket"],
        home_socket.display().to_string(),
        "{report}"
    );
    assert_eq!(report["legacy_socket"], false, "{report}");
    let text = found.to_string();
    assert!(
        text.contains("another world") && !text.contains("another contract"),
        "{text}"
    );
    let err = match Client::connect(ConnectOptions::interactive(true)).await {
        Err(e) => e,
        Ok(_) => panic!("a daemon on another world was used"),
    };
    assert!(
        matches!(&err, ConnectError::OtherWorld { found } if found.pid() == 4242),
        "{err:?}"
    );
    assert!(err.to_string().contains("never replaces it"), "{err}");
    let stopped = Client::stop_any().await.expect("stop").expect("a daemon");
    assert_eq!(stopped.pid(), 4242);
    peer.abort();
    let _ = std::fs::remove_file(session_socket());
}

/// C83, the split's step 6: a daemon on the legacy rendezvous — the
/// fixed socket every daemon before the derived one listened on — is
/// found only when this world's socket is silent, identified over the
/// same handshake, and reported with its endpoint; it is never this
/// client's (every dimension may match and the endpoint still refuses),
/// never replaced (the interactive door — which would replace a contract
/// mismatch on the world's socket — answers `Incompatible` with
/// `LegacySocket` and spawns nothing), and `stop_any` stops it when the
/// world's socket is silent. A daemon on the world's socket is never
/// compared with it.
#[tokio::test]
async fn c83_a_daemon_on_the_legacy_socket_is_reported_never_replaced_and_stopped() {
    let s = session("legacy");
    let home = World::observe().expect("the session's world");
    let legacy = legacy_socket_path().unwrap();
    assert!(
        legacy.starts_with(s.base.join("tmp")),
        "the legacy socket is in the scratch: {}",
        legacy.display()
    );
    // A daemon from before the rendezvous, on this very world: only the
    // endpoint tells it apart.
    let peer = foreign_daemon_at(
        legacy.clone(),
        json!({
            "resp": "hello", "version": VERSION, "contract": CONTRACT_REVISION,
            "pid": 4242, "provider": "mock", "world": home.name(),
        }),
        json!({ "resp": "stopping" }),
    )
    .await;

    let found = match Client::observe().await.expect("observe") {
        Observed::Incompatible(found) => found,
        Observed::Absent => panic!("the legacy socket was not probed"),
        Observed::Compatible(_) => panic!("a daemon on the legacy socket is never this client's"),
    };
    assert_eq!(found.pid(), 4242);
    assert!(found.on_legacy_socket(), "{found}");
    assert_eq!(found.socket(), legacy.to_str().unwrap(), "{found}");
    assert!(
        found.world_matches() && found.contract_matches() && found.provider_matches(),
        "{found}"
    );
    let report = found.report();
    assert_eq!(report["legacy_socket"], true, "{report}");
    assert_eq!(report["socket"], legacy.display().to_string(), "{report}");
    let text = found.to_string();
    assert!(
        text.contains("legacy socket") && text.contains(&legacy.display().to_string()),
        "{text}"
    );

    // The interactive door refuses, typed, and starts nothing: the
    // world's socket stays absent.
    let err = match Client::connect(ConnectOptions::interactive(true)).await {
        Err(e) => e,
        Ok(_) => panic!("a daemon on the legacy socket was used"),
    };
    assert!(
        matches!(
            &err,
            ConnectError::Incompatible {
                found,
                because: NotReplaced::LegacySocket
            } if found.pid() == 4242
        ),
        "{err:?}"
    );
    assert!(
        err.to_string().contains("acq daemon stop") && err.to_string().contains("once"),
        "{err}"
    );
    assert!(!session_socket().exists(), "a daemon was spawned beside it");

    // Stopping falls through to it while the world's socket is silent.
    let stopped = Client::stop_any().await.expect("stop").expect("a daemon");
    assert_eq!(stopped.pid(), 4242);
    assert!(stopped.on_legacy_socket());
    peer.abort();
    let _ = std::fs::remove_file(&legacy);

    // With the legacy socket silent, the world's rendezvous works as
    // before, and a daemon there is never compared with the legacy one:
    // a peer staged back on the legacy path while this daemon runs is
    // neither reported nor stopped.
    let mut daemon = start_daemon(&s).await;
    let pid = daemon.pid();
    let peer = foreign_daemon_at(
        legacy.clone(),
        json!({
            "resp": "hello", "version": VERSION, "contract": CONTRACT_REVISION,
            "pid": 4243, "provider": "mock", "world": home.name(),
        }),
        json!({ "resp": "stopping" }),
    )
    .await;
    let client = compatible_client().await;
    assert_eq!(client.daemon().pid(), pid);
    assert!(!client.daemon().on_legacy_socket());
    drop(client);
    let stopped = Client::stop_any().await.expect("stop").expect("a daemon");
    assert_eq!(stopped.pid(), pid, "the world's daemon is stopped first");
    daemon.wait_exit();
    let stopped = Client::stop_any()
        .await
        .expect("stop")
        .expect("the legacy peer");
    assert_eq!(stopped.pid(), 4243, "then the legacy one");
    peer.abort();
    let _ = std::fs::remove_file(&legacy);
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
    assert_eq!(stopped.pid(), daemon.pid());
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

/// C83: a daemon exiting removes the socket file it bound and never a
/// successor's at the same path. Staged directly: with a connection to
/// the daemon held open, its socket is unlinked and another listener
/// bound in its place — what a successor on the same world does when it
/// starts while the predecessor is still on its way out — then the
/// daemon is stopped over the held connection; the new listener's file
/// must survive the exit (seen in a rehearsal, 2026-09-11: a stop
/// followed within a second by a start on the same world lost the new
/// daemon's rendezvous).
#[tokio::test]
async fn c83_a_daemon_on_its_way_out_never_unlinks_a_successors_socket() {
    let s = session("exit");
    let mut daemon = start_daemon(&s).await;
    let socket = session_socket();
    let mut held = Raw::open().await;
    held.hello().await;

    // The successor's socket, in the predecessor's place.
    std::fs::remove_file(&socket).expect("unlink the live daemon's socket");
    let successor = UnixListener::bind(&socket).expect("bind in its place");

    let reply = held
        .ask(json!({ "req": "daemon_stop", "reason": "exit pin" }))
        .await;
    assert_eq!(reply, json!({ "resp": "stopping" }));
    daemon.wait_exit();

    assert!(
        socket.exists(),
        "the exiting daemon unlinked the successor's socket"
    );
    let probe = UnixStream::connect(&socket).await;
    assert!(probe.is_ok(), "the successor's socket no longer connects");
    let (_accepted, _) = successor.accept().await.expect("the successor accepts");
    drop(successor);
    let _ = std::fs::remove_file(&socket);
}

/// C83, the legacy probe bounded: the legacy socket lives in the temp
/// directory, shared on Linux, so anything may be bound there. A
/// listener that accepts and never answers the handshake must not hang
/// observation, the use door or the stop remedy: each comes back within
/// the probe's deadline with an error naming the socket — never absence,
/// never a spawn (review 2026-09-11).
#[tokio::test]
async fn c83_a_legacy_peer_that_never_answers_is_an_error_within_the_deadline_not_a_hang() {
    let _s = session("legacy-hang");
    let legacy = legacy_socket_path().unwrap();
    let _ = std::fs::remove_file(&legacy);
    let mute = UnixListener::bind(&legacy).expect("bind the legacy path");
    // Accepts every connection and holds it without a byte in reply.
    let hold = tokio::spawn(async move {
        let mut held = Vec::new();
        loop {
            let (stream, _) = mute.accept().await.expect("accept");
            held.push(stream);
        }
    });
    let bounded = Duration::from_secs(10);

    let err = match tokio::time::timeout(bounded, Client::observe())
        .await
        .expect("observe hung on the mute legacy peer")
    {
        Err(e) => e,
        Ok(Observed::Absent) => panic!("a mute peer read as absence"),
        Ok(_) => panic!("a mute peer read as a daemon"),
    };
    assert!(
        err.to_string().contains("did not answer")
            && err.to_string().contains(&legacy.display().to_string()),
        "{err:#}"
    );

    let err = tokio::time::timeout(bounded, Client::connect(ConnectOptions::interactive(true)))
        .await
        .expect("the use door hung on the mute legacy peer")
        .err()
        .expect("the use door opened");
    assert!(matches!(err, ConnectError::Transport(_)), "{err:?}");
    assert!(err.to_string().contains("did not answer"), "{err}");
    assert!(
        !session_socket().exists(),
        "a daemon was spawned beside a peer the door could not identify"
    );

    let err = tokio::time::timeout(bounded, Client::stop_any())
        .await
        .expect("stop hung on the mute legacy peer")
        .expect_err("a mute peer cannot be stopped as a daemon");
    assert!(err.to_string().contains("did not answer"), "{err:#}");

    hold.abort();
    let _ = std::fs::remove_file(&legacy);
}
