//! Client side of the daemon protocol: connect, lazy-spawn, version handshake.
//!
//! Every frontend (CLI, MCP, GUI) reaches the daemon through this module,
//! by one of three doors — the policy tiers of C10 — and a use door that
//! may not open answers with a typed [`ConnectError`] rather than a
//! string, so the GUI and the MCP server can branch on it:
//!
//! - **Use** — [`Client::connect`] with a [`ConnectOptions`]: the verb is
//!   about to submit work. The interactive CLI spawns as asked and replaces
//!   a mismatched daemon, because its caller is the human expressing
//!   intent (`ConnectOptions::interactive`); an autonomous client (the MCP
//!   server) spawns only into an empty socket in mock mode and never
//!   replaces — the mismatch it sees may be a human's live GGG run
//!   (`ConnectOptions::autonomous`).
//! - **Observe** — [`Client::observe`]: the verb reads the daemon (`jobs`,
//!   `status`, `result`, `auth status`, `daemon status`, a quote) or acts
//!   on the one that is there (`cancel`, `set-priority`, `reset-tripwire`).
//!   It takes no options because there is nothing to allow: it never
//!   spawns or replaces, and it answers [`Observed::Absent`], a
//!   [`Observed::Compatible`] client, or the [`DaemonId`] of an
//!   [`Observed::Incompatible`] daemon it identified and did not use.
//! - **Stop** — [`Client::stop_any`]: `daemon stop` stops whatever is
//!   listening, this build's or not. Stopping is how a human resolves a
//!   mismatch, so it is the one verb that acts on a daemon it would not use.
//!
//! `ACQ_NO_SPAWN=1` turns every use door into an observation.
//!
//! What a use door spawns is the `acqd` beside this executable and nothing
//! else (`locator.rs`, C82): the daemon is its own artifact, never a mode
//! of the frontend's binary. Before it spawns, a use door creates the
//! world's root (C83: the use path creates, observation never does) so
//! the daemon and this process canonicalise the same directory.
//!
//! A daemon on another world (C83) is the one mismatch no door resolves:
//! `hello` names the daemon's canonical root, the client compares it with
//! its own before the other dimensions, and a use door answers
//! [`ConnectError::OtherWorld`] rather than replacing a daemon that is
//! legitimately serving its own world; an observer reports it like any
//! other mismatch, and `stop_any` still stops it.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/daemon.md` for this
//! area, `CONTEXT.md` for the cross-cutting ones (`C<n>`); what follows is
//! the entry's full text as recorded there, kept beside the code that
//! implements it. The registry is current; this is the mechanism as
//! decided and as built.
//!
//! ## C10 — Handshake protocol is single-version on purpose.
//!
//! **C10 — Handshake protocol is single-version on purpose.** Migration is
//! via kill-and-respawn only. A client uses a daemon only when its provider
//! and runtime identity match what it would itself spawn. That identity
//! changes automatically with the daemon and protocol implementation it
//! governs; it never derives from Git state or a hand-maintained value. A
//! use verb may replace a mismatch; observation never spawns or replaces,
//! just reports absence and mismatches distinctly; an autonomous client
//! (MCP) never replaces. *Why:* a compat matrix is the reconciliation
//! swamp; respawn is a one-line diff; an observer that replaced cost a live
//! run (2026-09-08). *Details:* `client.rs` doc, C10. *Pinned:*
//! `acquisition-cli/tests/daemon_observe.rs`. Amended 2026-09-09.
//!
//! ## C84 — The runtime identity (C10) is two values from two sources
//!
//! **C84 — The runtime identity (C10) is two values from two sources:
//! the shared-contract revision, a digest of the protocol and store
//! sources, manifests and lock both sides are compiled from, and the
//! daemon artifact — the executable, which the daemon identifies and
//! hashes at startup and a client compares with the sibling `acqd` it
//! would spawn.** A mismatch on either is reported by name; a rebuilt
//! `acqd` under a live daemon is an artifact mismatch. Daemon-only source
//! changes move only the artifact; nothing derives from Git or a
//! hand-kept number. *Why:* the contract answers "can I use it", the
//! artifact "is it the one I would start"; each sees a failure the other
//! cannot. *Details:* `client.rs` doc, C10. *Pinned:*
//! `daemon_observe.rs`. Ruled 2026-09-09.
//!
//! ## C10, C84 and C83 — as built
//!
//! The identity compared is the shared-contract revision and the daemon
//! artifact, reported together with the provider and the world
//! ([`DaemonId`]): the four dimensions a client judges, each with its own
//! source. The world is judged first and answered differently — refused,
//! never replaced (below).
//!
//! The **contract** is [`CONTRACT_REVISION`], defined by the protocol
//! crate (`acquisition-protocol/src/lib.rs`) so both sides carry the same
//! definition: a digest over the protocol and store sources, their
//! manifests, the root manifest and the lock (the protocol crate's
//! `build.rs`). It changes whenever any of those whole files changes: a
//! lock entry or a store function the daemon never uses does (a deliberate
//! false mismatch — a respawn is cheap, a missed contract change is
//! invisible); a source edit to the daemon, the planner, this crate or a
//! frontend does not, and no git state is consulted. The package version
//! alone is fixed at `0.0.1` across the playground, and comparing it let
//! a pre-realm daemon accept a console job and render the pc URL (review
//! finding 2026-09-02).
//!
//! The **artifact** is the executable the daemon runs from: at startup it
//! records its file identity and SHA-256 and reports both in `hello`
//! (`acquisition-protocol/src/artifact.rs` defines the identity; the
//! daemon's `artifact.rs` computes it). This process compares that with
//! the sibling `acqd` the locator names (C82), opened once at the moment of
//! the comparison (`artifact.rs` here): the same inode with the same
//! length and modification time is the same file; anything else is
//! settled by hashing the sibling — so a copy is the same artifact and a
//! rebuild under a live daemon is a named mismatch that the next use verb
//! resolves by respawning. A daemon-only edit moves this value and not the
//! contract, which is what lets a daemon edit leave every frontend
//! untouched (`DAEMON-SPLIT-SLICE.md`, step 4's measure). A process with
//! no sibling — no `acqd` beside it — matches no daemon on this dimension
//! and says so: it could not have started the one it found.
//!
//! The **provider** is the handshake's `provider` against what this
//! process wants (`ACQ_GGG`).
//!
//! The **world** (C83) is the handshake's `world` — the daemon's
//! canonical store root — against this process's own
//! (`acquisition_store::world::World::observe`, which creates nothing:
//! a root that does not exist is an absent world and matches no daemon).
//! Two spellings of one directory are one world because both sides
//! canonicalise; a daemon on another root is another world, and the
//! typed answer at a use door is [`ConnectError::OtherWorld`], since
//! killing it would stop a daemon that is serving its own world
//! correctly — the collision is the rendezvous, which step 6 derives from
//! the root.
//!
//! Why two values and not one: with the artifact alone, a frontend rebuilt
//! after a wire or store change runs against an old daemon that matches
//! itself — a silent contract mismatch, and for the store a new reader
//! migrating files an old daemon still writes; with the contract alone, a
//! daemon-only edit never changes the identity and a stale daemon keeps
//! serving. Together, a partial rebuild is loud: the client sees the
//! mismatch, replaces, spawns the same old file, and reports it "still,
//! after a respawn". Every frontend built from one tree carries the same
//! contract and finds the same sibling, which is what lets `acq-mcp`
//! accept a daemon `acq` spawned (C6, C31); two installations would thrash
//! by respawning each other's daemons — the design event C82 names.
//!
//! The trap the observe tier closes (ledger row 2026-09-08): `acq daemon
//! status` typed in a second terminal without `ACQ_GGG` connected under the
//! interactive policy, replaced the live daemon with a mock one on the
//! default socket, and the driver's next daemon refused to start over it.
//! Every observational verb of both frontends now goes through
//! [`Client::observe`]; the interactive `ConnectOptions` reach only the
//! verbs that submit work.
//!
//! Pinned through the binaries in `acquisition-cli/tests/daemon_observe.rs`:
//! the provider dimension by a mock daemon observed from a shell that
//! wants ggg, and the artifact dimension by a copy of `acq` whose sibling
//! is another file (a mismatch) or a copy of `acqd` (the same artifact by
//! hash) — the residual the split's step 4 closed.
//!
//! ## C85 — Connection semantics, as built on this side
//!
//! The ruling is recorded in full on the protocol crate's `protocol.rs`. Here: every connection
//! opens with the bootstrap `hello` exchange (`Client::handshake`),
//! written and read as [`Bootstrap`]/[`BootstrapReply`] frames outside the
//! versioned enums, so a daemon of any revision is identified — a frame
//! this build cannot read fully still yields a [`DaemonId`] reported as
//! another contract — and `stop_any` works across the same mismatch. A
//! [`Client`] is a request connection: [`Client::request`] takes `&mut
//! self`, writes one frame and reads exactly one, so one request is in
//! flight by construction; an `event` or `resync_required` frame arriving
//! there is a protocol violation and an error. A [`Subscription`] is a
//! connection of its own — hello, `subscribe`/`subscribed`, then
//! [`Subscription::next`] only — with no way to send a request on it; it
//! yields [`Signal::Event`], [`Signal::ResyncRequired`] with the count the
//! daemon dropped, and `None` when the daemon is gone. The subscriber's
//! sequence (subscribe, then snapshot over a `Client` to the same daemon,
//! re-read on every event, and after `resync_required` or a disconnect
//! drop the subscription, open a new one and snapshot again) is the
//! consumer's — `acq jobs --watch` is the reference.
//! Both sides read a frame no further than
//! [`acquisition_protocol::protocol::MAX_FRAME_BYTES`]; an answer over it is reported,
//! and the connection stays aligned on the next frame. A daemon error is
//! a [`DaemonError`] carrying the closed [`ErrorKind`] beside its message,
//! so a frontend's JSON can carry the kind (`acq --json`: `{"error", "kind"}`)
//! and the MCP server can pick its error code from it.

use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use crate::artifact::{ArtifactVerdict, SiblingError, sibling};
use crate::frame::{Frame, read_frame};
use crate::locator;
use acquisition_protocol::artifact::{Artifact, FileIdentity};
use acquisition_protocol::job::JobInfo;
use acquisition_protocol::protocol::{
    Bootstrap, BootstrapReply, ErrorKind, MAX_FRAME_BYTES, Request, Response, error_message,
};
use acquisition_protocol::{CONTRACT_REVISION, VERSION};
use acquisition_store::world::{World, socket_path};
use anyhow::{Context, Result, bail};
use serde::Serialize;
use serde_json::json;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

/// A request the daemon refused, as the wire carries it (C85): the closed
/// kind a frontend branches on and the message a person reads. Its
/// `Display` is the message alone — the kind is for JSON and error codes,
/// never a prefix on prose.
#[derive(Debug, Clone, Serialize)]
pub struct DaemonError {
    pub kind: ErrorKind,
    pub message: String,
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for DaemonError {}

impl DaemonError {
    /// The daemon error in an error chain, if one is there — how a
    /// frontend's top level finds the kind to put beside the message.
    pub fn find(error: &anyhow::Error) -> Option<&DaemonError> {
        error.chain().find_map(|e| e.downcast_ref::<DaemonError>())
    }
}

/// `Response::Error` as an error value, for the sites that unwrap a
/// response.
fn refused(kind: ErrorKind, message: String) -> anyhow::Error {
    DaemonError { kind, message }.into()
}

/// What a *use* verb may do to a daemon that isn't the one it wants.
/// `ACQ_NO_SPAWN=1` overrides both flags to false. Observation takes no
/// options: [`Client::observe`].
#[derive(Clone, Copy, Debug)]
pub struct ConnectOptions {
    /// Start a daemon if none is listening (lazy spawn: the `acqd` beside
    /// this executable, `locator.rs`).
    pub spawn: bool,
    /// Kill and respawn a daemon whose identity or provider doesn't match.
    pub replace: bool,
}

impl ConnectOptions {
    /// The interactive CLI's policy for a use verb: the caller is the
    /// human, so replacing a daemon of another contract, artifact or mode
    /// is them expressing intent.
    pub fn interactive(spawn: bool) -> Self {
        Self {
            spawn,
            replace: true,
        }
    }

    /// An autonomous client's policy (MCP): never kill or replace a daemon —
    /// the mismatched daemon may be a human's live GGG run. Spawning into an
    /// empty socket may still be allowed.
    pub fn autonomous(spawn: bool) -> Self {
        Self {
            spawn,
            replace: false,
        }
    }
}

/// Why a use door ([`Client::connect`]) did not open: typed, so a
/// frontend renders or branches on it — the GUI's door, the MCP's error
/// code — rather than parsing prose. `Display` is the sentence a person
/// reads; the CLI prints it as is.
#[derive(Debug)]
pub enum ConnectError {
    /// Nothing is listening, and this door may not spawn — the caller's
    /// policy, or `ACQ_NO_SPAWN=1` over a caller that would have.
    Absent { because: NotSpawned },
    /// The daemon listening is not this client's (C10: contract, artifact
    /// or provider), and this door may not replace it — or did, and the
    /// replacement is still not it.
    Incompatible {
        found: DaemonId,
        because: NotReplaced,
    },
    /// The daemon serves another world (C83): `hello` named a canonical
    /// root that is not this process's (`found.world()`). No door replaces
    /// it — a daemon on its own world is not stale — so the remedy is the
    /// human's: stop it, or point `ACQ_STORE_DIR` at its world.
    OtherWorld { found: DaemonId },
    /// The daemon could not be started: no `acqd` beside this executable
    /// (C82), the spawn failed, or the daemon exited or never bound its
    /// socket. `log` is the explanation, with what the daemon's log said
    /// after the spawn when there was a daemon to say it.
    SpawnFailed { acqd: Option<PathBuf>, log: String },
    /// The socket answered, and the conversation failed: a connect error
    /// that is not absence, or a handshake this build could not read.
    Transport(anyhow::Error),
}

/// Why an absent daemon was not started. The remedy differs: under the
/// caller's policy it is the caller's to name (the MCP server in real
/// mode says "start it from the CLI"), under the knob it is the knob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotSpawned {
    /// The caller's door does not spawn (`ConnectOptions::spawn` false).
    Policy,
    /// The caller would have spawned; `ACQ_NO_SPAWN=1` forbids it.
    NoSpawnEnv,
}

/// Why an incompatible daemon was left standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotReplaced {
    /// `ACQ_NO_SPAWN=1` forbids replacing it.
    NoSpawn,
    /// An autonomous client (the MCP server) never replaces (C13).
    NeverReplaces,
    /// This client replaced it, and the daemon it then found is still not
    /// its own — the sibling `acqd` is not this build's.
    StillAfterRespawn,
}

impl fmt::Display for ConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectError::Absent {
                because: NotSpawned::Policy,
            } => f.write_str("daemon is not running, and this client does not start one"),
            ConnectError::Absent {
                because: NotSpawned::NoSpawnEnv,
            } => {
                f.write_str("daemon is not running and ACQ_NO_SPAWN forbids starting one from here")
            }
            ConnectError::Incompatible { found, because } => match because {
                NotReplaced::NoSpawn => write!(f, "{found}, and ACQ_NO_SPAWN forbids replacing it"),
                NotReplaced::NeverReplaces => write!(
                    f,
                    "{found}, and this client never replaces a daemon — resolve it with the CLI (`acq daemon stop`)"
                ),
                NotReplaced::StillAfterRespawn => write!(f, "{found}, still, after a respawn"),
            },
            ConnectError::OtherWorld { found } => {
                write!(
                    f,
                    "{found}; this client refuses a daemon on another world and never replaces it — `acq daemon stop` stops it, or point ACQ_STORE_DIR at its world"
                )
            }
            ConnectError::SpawnFailed {
                acqd: Some(acqd),
                log,
            } => {
                write!(f, "could not start the daemon {}: {log}", acqd.display())
            }
            ConnectError::SpawnFailed { acqd: None, log } => f.write_str(log),
            ConnectError::Transport(e) => write!(f, "{e:#}"),
        }
    }
}

impl std::error::Error for ConnectError {}

/// "ggg" or "mock": the provider this process wants a daemon to serve.
fn want_provider() -> &'static str {
    acquisition_protocol::provider::wanted()
}

/// A daemon as its handshake identifies it, judged once. Whether it is
/// this client's is four dimensions, judged together (C10, C84, C83): the
/// shared-contract revision, the daemon artifact against the sibling
/// this process would spawn, the provider, and the world against this
/// process's own. The identity and its verdict are read-only from
/// outside this module — the verdict was made of exactly these fields,
/// and nothing may change one without the other (review 2026-09-11).
#[derive(Clone, Debug, Serialize)]
pub struct DaemonId {
    pid: u32,
    /// The daemon's package version; informational (C84 compares the
    /// contract, not this).
    version: String,
    /// The daemon's shared-contract revision.
    contract: String,
    /// The executable the daemon runs from, as it reported it; `None`
    /// from a daemon of a build before the field. Boxed so a
    /// [`ConnectError`] carrying a `DaemonId` stays small on the stack.
    artifact: Option<Box<Artifact>>,
    /// "mock" or "ggg".
    provider: String,
    /// The canonical store root the daemon serves (C83), as reported;
    /// `unknown` from a daemon of a build before the field.
    world: String,
    /// The judgement, captured once when this identity was read off the
    /// wire, from one look at the sibling: the compatibility flag, the
    /// relation and the sibling a report names all come from the same
    /// snapshot, so a rebuild between two looks cannot make a report
    /// contradict itself (review 2026-09-11). Boxed, like the artifact,
    /// so a [`ConnectError`] carrying a `DaemonId` stays small.
    #[serde(skip)]
    verdict: Box<Verdict>,
}

/// What a client concluded about a daemon, dimension by dimension, from
/// one look at the sibling (opened once: its identity from the handle,
/// and its bytes from the same handle when they must decide), kept with
/// the identity it judged.
#[derive(Debug, Clone)]
pub struct Verdict {
    pub contract: bool,
    pub artifact: ArtifactVerdict,
    pub provider: bool,
    /// The daemon serves this process's world (C83).
    pub world: bool,
    /// The sibling `acqd` as found in that one look, or why there is none.
    pub sibling: Result<FileIdentity, SiblingError>,
    /// This process's own world as found in that one look — its canonical
    /// root, or the root it intended and why there is no world there.
    /// The report and the prose read this, never a second look
    /// (review 2026-09-11).
    pub own_world: Result<String, AbsentWorld>,
}

/// This process has no world: the root it intended does not resolve.
#[derive(Debug, Clone)]
pub struct AbsentWorld {
    pub intended: String,
    pub reason: String,
}

impl Verdict {
    /// One look at the sibling — opened once; its identity and, when
    /// needed, its bytes from that handle — then every dimension judged
    /// against it.
    fn of(contract: &str, artifact: Option<&Artifact>, provider: &str, world: &str) -> Verdict {
        let opened = sibling();
        let identity = match &opened {
            Ok(s) => Ok(s.identity.clone()),
            Err(e) => Err(e.clone()),
        };
        let own_world = own_world();
        Verdict {
            contract: contract == CONTRACT_REVISION,
            artifact: ArtifactVerdict::judge_against(artifact, opened),
            provider: provider == want_provider(),
            world: own_world.as_deref().ok() == Some(world),
            sibling: identity,
            own_world,
        }
    }

    /// Every dimension matches: this client may use the daemon.
    pub fn is_ours(&self) -> bool {
        self.contract && self.artifact.matches() && self.provider && self.world
    }
}

/// This process's world (C83), observed — never created here: the
/// canonical root when it exists, else the root it intended and why
/// there is no world there.
fn own_world() -> Result<String, AbsentWorld> {
    World::observe().map(|w| w.name()).map_err(|e| AbsentWorld {
        intended: e.intended.display().to_string(),
        reason: e.to_string(),
    })
}

/// The intended root as `hello` names it from this side, whether or not
/// it exists yet: the daemon logs a mismatch, nothing more. A look of its
/// own, before the handshake; the verdict's look is the one every report
/// reads.
fn own_world_name() -> String {
    match own_world() {
        Ok(name) => name,
        Err(absent) => absent.intended,
    }
}

impl DaemonId {
    /// An identity as read off the wire, judged once, now, against what
    /// this process is and would spawn.
    pub fn judged(
        pid: u32,
        version: String,
        contract: String,
        artifact: Option<Artifact>,
        provider: String,
        world: String,
    ) -> DaemonId {
        let verdict = Verdict::of(&contract, artifact.as_ref(), &provider, &world);
        DaemonId {
            pid,
            version,
            contract,
            artifact: artifact.map(Box::new),
            provider,
            world,
            verdict: Box::new(verdict),
        }
    }

    /// The judgement captured with this identity.
    pub fn verdict(&self) -> &Verdict {
        &self.verdict
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// The daemon's package version; informational.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The daemon's shared-contract revision, as reported.
    pub fn contract(&self) -> &str {
        &self.contract
    }

    /// The executable the daemon runs from, as reported; `None` from a
    /// build before the field.
    pub fn artifact(&self) -> Option<&Artifact> {
        self.artifact.as_deref()
    }

    /// "mock" or "ggg", as reported.
    pub fn provider(&self) -> &str {
        &self.provider
    }

    /// The daemon was compiled against the contract this process was.
    pub fn contract_matches(&self) -> bool {
        self.verdict.contract
    }

    /// The daemon serves the provider this process wants.
    pub fn provider_matches(&self) -> bool {
        self.verdict.provider
    }

    /// The canonical store root the daemon serves, as reported (C83).
    pub fn world(&self) -> &str {
        &self.world
    }

    /// The daemon serves this process's world (C83).
    pub fn world_matches(&self) -> bool {
        self.verdict.world
    }

    /// Every dimension matches: this client may use the daemon.
    pub fn is_ours(&self) -> bool {
        self.verdict.is_ours()
    }

    /// The observer's report, one shape for every frontend: the daemon
    /// found, what this process wanted — its contract, its provider, its
    /// world (or the root it intended, with `world_absent` saying why
    /// there is none) and the sibling `acqd` as found in the one look
    /// that judged it, or `null` with the reason — and which dimensions
    /// differ.
    pub fn report(&self) -> serde_json::Value {
        let verdict = &self.verdict;
        let (acqd, acqd_absent) = match &verdict.sibling {
            Ok(id) => (serde_json::to_value(id).unwrap_or_default(), None),
            Err(e) => (serde_json::Value::Null, Some(e.to_string())),
        };
        let mut wanted = json!({
            "version": VERSION,
            "contract": CONTRACT_REVISION,
            "provider": want_provider(),
            "world": match &verdict.own_world {
                Ok(name) => name.clone(),
                Err(absent) => absent.intended.clone(),
            },
            "acqd": acqd,
        });
        if let Some(reason) = acqd_absent {
            wanted["acqd_absent"] = json!(reason);
        }
        if let Err(absent) = &verdict.own_world {
            wanted["world_absent"] = json!(absent.reason);
        }
        let mut report = json!({
            "pid": self.pid,
            "version": self.version,
            "contract": self.contract,
            "artifact": self.artifact,
            "provider": self.provider,
            "world": self.world,
            "contract_matches": verdict.contract,
            "artifact_matches": verdict.artifact.matches(),
            "artifact_relation": verdict.artifact.relation(),
            "provider_matches": verdict.provider,
            "world_matches": verdict.world,
            "wanted": wanted,
        });
        if let Some(why) = artifact_mismatch_reason(&verdict.artifact) {
            report["artifact_mismatch"] = json!(why);
        }
        report
    }
}

/// Why the artifact dimension did not match, when it did not: the
/// sentence the report and the prose share.
fn artifact_mismatch_reason(verdict: &ArtifactVerdict) -> Option<String> {
    match verdict {
        ArtifactVerdict::SameFile | ArtifactVerdict::SameBytes { .. } => None,
        ArtifactVerdict::Different {
            sibling,
            sibling_sha256,
        } => Some(format!(
            "the acqd beside this executable is another file ({}, sha256 {})",
            sibling.path,
            sibling_sha256.get(..12).unwrap_or(sibling_sha256)
        )),
        ArtifactVerdict::Unhashable { sibling, io } => Some(format!(
            "the acqd beside this executable ({}) differs and could not be hashed: {io}",
            sibling.path
        )),
        ArtifactVerdict::NoSibling(e) => {
            Some(format!("this executable could not have started it: {e}"))
        }
        ArtifactVerdict::Unreported => {
            Some("the daemon reported no artifact (a build before the identity split)".into())
        }
    }
}

impl fmt::Display for DaemonId {
    /// The mismatch sentence an observer prints: which dimensions differ,
    /// with both sides of each.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let verdict = &self.verdict;
        write!(f, "daemon (pid {})", self.pid)?;
        if verdict.is_ours() {
            return write!(
                f,
                " is this client's (contract {}, {}, {}, world {})",
                self.contract,
                match (&verdict.artifact, &self.artifact) {
                    (ArtifactVerdict::SameBytes { sibling }, Some(a)) => format!(
                        "acqd sha256 {} at {}, a copy of this client's sibling {}",
                        a.short_hash(),
                        a.file.path,
                        sibling.path
                    ),
                    (_, Some(a)) => {
                        format!("acqd sha256 {}, this client's sibling", a.short_hash())
                    }
                    (_, None) => "no artifact".to_string(),
                },
                self.provider,
                self.world
            );
        }
        if !verdict.world {
            // The first dimension judged, and the one no door resolves:
            // named first, and alone — a daemon on another world is not
            // compared further, since nothing this process would do to
            // it follows from the other three.
            return write!(
                f,
                " serves another world ({}; this process's is {})",
                self.world,
                match &verdict.own_world {
                    Ok(root) => root.clone(),
                    Err(absent) => format!("absent — {}", absent.reason),
                }
            );
        }
        let mut clauses: Vec<String> = Vec::new();
        if !verdict.contract {
            clauses.push(format!(
                "is another contract ({}; this is {CONTRACT_REVISION})",
                self.contract
            ));
        }
        if let Some(why) = artifact_mismatch_reason(&verdict.artifact) {
            clauses.push(format!(
                "runs another artifact ({}; {why})",
                self.artifact
                    .as_ref()
                    .map_or("none reported".to_string(), |a| format!(
                        "{}, sha256 {}",
                        a.file.path,
                        a.short_hash()
                    ))
            ));
        }
        if !verdict.provider {
            clauses.push(format!(
                "is on another provider ({}; this process wants {})",
                self.provider,
                want_provider()
            ));
        }
        write!(f, " {}", clauses.join(" and "))
    }
}

/// What [`Client::observe`] (or [`Subscription::observe`]) found on the
/// socket.
pub enum Observed<C = Client> {
    /// Nothing is listening.
    Absent,
    /// The daemon is this client's, and this is a connection to it.
    Compatible(C),
    /// A daemon that is not this client's: identified, reported, not used.
    Incompatible(DaemonId),
}

/// A request connection (C85): one request in flight, one response per
/// request, never an event.
pub struct Client {
    reader: BufReader<OwnedReadHalf>,
    write: OwnedWriteHalf,
    /// The daemon at the other end, as its handshake identified it.
    daemon: DaemonId,
}

/// What a [`Subscription`] is told next (C85).
#[derive(Debug, Clone)]
pub enum Signal {
    /// A job changed. An invalidation hint: re-read before relying on the
    /// view, never assume the stream is complete.
    Event(JobInfo),
    /// The daemon dropped `missed` events for this subscriber. Drop this
    /// subscription — what it still holds predates any snapshot taken
    /// now — open a new one, then snapshot again over a [`Client`] before
    /// trusting the view.
    ResyncRequired { missed: u64 },
}

/// A subscription connection (C8's event channel under C85's semantics):
/// hello, `subscribe`/`subscribed`, then
/// events only — there is no way to send a request on it, which is what
/// keeps requests and events on separate connections by construction.
/// The subscriber snapshots over a [`Client`] to the same daemon after
/// opening this, re-reads a job before relying on an event about it, and
/// after a [`Signal::ResyncRequired`] or after [`Subscription::next`]
/// returns `None` drops this subscription — what it still holds predates
/// any snapshot taken now — opens a new one, and snapshots again.
pub struct Subscription {
    reader: BufReader<OwnedReadHalf>,
    /// Held so the daemon sees the connection open; nothing is written.
    _write: OwnedWriteHalf,
    daemon: DaemonId,
}

impl Subscription {
    /// Observe the socket (C10) and, if this client's daemon is there,
    /// subscribe on a connection of its own. Never spawns or replaces:
    /// watching is observation.
    pub async fn observe() -> Result<Observed<Subscription>> {
        match Client::observe().await? {
            Observed::Absent => Ok(Observed::Absent),
            Observed::Incompatible(found) => Ok(Observed::Incompatible(found)),
            Observed::Compatible(client) => client.subscribe().await.map(Observed::Compatible),
        }
    }

    /// The daemon this subscription is on, as its handshake identified it.
    pub fn daemon(&self) -> &DaemonId {
        &self.daemon
    }

    /// The next signal; `None` when the daemon closed the connection.
    pub async fn next(&mut self) -> Result<Option<Signal>> {
        loop {
            let bytes = match read_frame(&mut self.reader, MAX_FRAME_BYTES).await? {
                Frame::Closed => return Ok(None),
                Frame::Oversize => {
                    bail!("the daemon sent a frame over {MAX_FRAME_BYTES} bytes on a subscription")
                }
                Frame::Line(bytes) => bytes,
            };
            if bytes.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            return match serde_json::from_slice::<Response>(&bytes)? {
                Response::Event { job } => Ok(Some(Signal::Event(job))),
                Response::ResyncRequired { missed } => Ok(Some(Signal::ResyncRequired { missed })),
                other => bail!("unexpected frame on a subscription: {other:?}"),
            };
        }
    }
}

fn no_spawn() -> bool {
    std::env::var_os("ACQ_NO_SPAWN").is_some_and(|v| v == "1")
}

/// A connect failure that means nothing is listening — the socket absent
/// or refusing — as opposed to a transport problem worth reporting.
fn is_absent(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused
    )
}

impl Client {
    /// Connect to the daemon for a *use* verb, spawning or replacing one as
    /// `opts` allows. What it spawns is the `acqd` beside this executable
    /// (C82). A door that does not open is a [`ConnectError`]: absence
    /// this door may not fill, a mismatch it may not resolve (a mock-mode
    /// daemon can't serve an `ACQ_GGG=1` client, or vice versa), a spawn
    /// that failed, or a transport failure — each naming what it found.
    pub async fn connect(opts: ConnectOptions) -> Result<Client, ConnectError> {
        // `ACQ_NO_SPAWN=1`: never start or replace a daemon from this
        // process. A daemon spawned from a non-interactive parent (cron,
        // launchd) has no keychain access on macOS — it comes up with no
        // session and every job fails "not logged in" (re-soak, 2026-08-25,
        // caught by rail 7). The live drivers set this so their scripts can
        // only talk to a daemon they started themselves.
        let spawn = opts.spawn && !no_spawn();
        let replace = opts.replace && !no_spawn();
        let mut respawned = false;
        // The spawned daemon, with where its log ended at spawn time: if it
        // exits instead of binding the socket, the lines after that offset
        // are its refusal (its stderr goes to null — the log is all there is).
        let mut child: Option<Spawned> = None;
        for _attempt in 0..100 {
            match UnixStream::connect(socket_path()).await {
                Ok(stream) => {
                    let mut client = Client::handshake(stream)
                        .await
                        .map_err(ConnectError::Transport)?;
                    if client.daemon.is_ours() {
                        return Ok(client);
                    }
                    // Another world is judged before anything else and
                    // is never replaced (C83): the daemon is serving its
                    // own world; only the socket collided.
                    if !client.daemon.world_matches() {
                        return Err(ConnectError::OtherWorld {
                            found: client.daemon,
                        });
                    }
                    if respawned {
                        return Err(ConnectError::Incompatible {
                            found: client.daemon,
                            because: NotReplaced::StillAfterRespawn,
                        });
                    }
                    if !replace {
                        let because = if no_spawn() {
                            NotReplaced::NoSpawn
                        } else {
                            NotReplaced::NeverReplaces
                        };
                        return Err(ConnectError::Incompatible {
                            found: client.daemon,
                            because,
                        });
                    }
                    // Stale daemon (older build, or wrong mode): kill and
                    // respawn — over the bootstrap plane, which works across
                    // the mismatch (C85).
                    let _ = client.stop().await;
                    respawned = true;
                    child = None;
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                // Once we've killed a mismatched daemon we must respawn it
                // even for a `spawn: false` caller — leaving nothing running
                // would turn a read into a stop.
                Err(_) if spawn || respawned => {
                    match child.as_mut() {
                        None => child = Some(spawn_daemon()?),
                        Some(spawned) => {
                            // An exited daemon will never bind the socket:
                            // report its refusal now, not after the timeout.
                            if let Some(status) = spawned.child.try_wait().ok().flatten() {
                                return Err(ConnectError::SpawnFailed {
                                    acqd: Some(spawned.acqd.clone()),
                                    log: format!(
                                        "the daemon exited during startup ({status}){}",
                                        startup_log_excerpt(spawned)
                                    ),
                                });
                            }
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
                Err(e) if is_absent(&e) => {
                    // The caller's policy first: a door that never spawns
                    // is absent by policy whatever the knob says.
                    let because = if opts.spawn {
                        NotSpawned::NoSpawnEnv
                    } else {
                        NotSpawned::Policy
                    };
                    return Err(ConnectError::Absent { because });
                }
                Err(e) => {
                    return Err(ConnectError::Transport(
                        anyhow::Error::from(e)
                            .context(format!("connecting to {}", socket_path().display())),
                    ));
                }
            }
        }
        Err(match child {
            Some(spawned) => ConnectError::SpawnFailed {
                log: format!(
                    "it did not bind {} within 5s{}",
                    socket_path().display(),
                    startup_log_excerpt(&spawned)
                ),
                acqd: Some(spawned.acqd),
            },
            None => ConnectError::Transport(anyhow::anyhow!(
                "could not reach daemon at {} after 5s",
                socket_path().display()
            )),
        })
    }

    /// Observe the socket (C10): never spawns or replaces. Nothing
    /// listening is [`Observed::Absent`]; this client's daemon comes back
    /// connected; any other daemon is identified and reported, and the
    /// connection to it is dropped unused.
    pub async fn observe() -> Result<Observed> {
        match UnixStream::connect(socket_path()).await {
            Ok(stream) => {
                let client = Client::handshake(stream).await?;
                Ok(if client.daemon.is_ours() {
                    Observed::Compatible(client)
                } else {
                    Observed::Incompatible(client.daemon)
                })
            }
            Err(e) if is_absent(&e) => Ok(Observed::Absent),
            Err(e) => Err(e).with_context(|| format!("connecting to {}", socket_path().display())),
        }
    }

    /// `daemon stop`: ask whatever daemon is listening to stop, this
    /// client's or not — stopping is how a human resolves a mismatch. Says
    /// which daemon acknowledged (the daemon writes `Stopping` before it
    /// exits; anything else is an error, not a stop); `None` when nothing
    /// was listening.
    pub async fn stop_any() -> Result<Option<DaemonId>> {
        match UnixStream::connect(socket_path()).await {
            Ok(stream) => Client::stop_over(stream).await.map(Some),
            Err(e) if is_absent(&e) => Ok(None),
            Err(e) => Err(e).with_context(|| format!("connecting to {}", socket_path().display())),
        }
    }

    /// The stop conversation over an open connection: identify the peer,
    /// ask it to stop, and count only its `Stopping` as a stop.
    async fn stop_over(stream: UnixStream) -> Result<DaemonId> {
        let mut client = Client::handshake(stream).await?;
        client.stop().await?;
        Ok(client.daemon)
    }

    /// `daemon_stop` over the bootstrap plane (C85): read leniently, so a
    /// daemon of any revision can acknowledge. Only `stopping` is a stop;
    /// an error frame is reported by its message, anything else as
    /// unexpected, and a hang-up as the daemon closing the connection.
    async fn stop(&mut self) -> Result<()> {
        let bytes = self.exchange(&Bootstrap::DaemonStop).await?;
        match BootstrapReply::read(&bytes) {
            Some(BootstrapReply::Stopping) => Ok(()),
            _ => match error_message(&bytes) {
                Some(message) => bail!("{} refused to stop: {message}", self.daemon),
                None => bail!(
                    "unexpected response to stop: {}",
                    String::from_utf8_lossy(&bytes)
                ),
            },
        }
    }

    /// The handshake over a fresh connection: who is at the other end,
    /// over the bootstrap plane (C85) so any revision answers. Decides
    /// nothing — the caller reads `daemon` and applies its policy.
    async fn handshake(stream: UnixStream) -> Result<Client> {
        let (read, write) = stream.into_split();
        let mut client = Client {
            reader: BufReader::new(read),
            write,
            daemon: DaemonId::judged(
                0,
                String::new(),
                String::new(),
                None,
                String::new(),
                String::new(),
            ),
        };
        let bytes = client
            .exchange(&Bootstrap::Hello {
                version: VERSION.to_string(),
                contract: CONTRACT_REVISION.to_string(),
                world: own_world_name(),
            })
            .await?;
        let Some(BootstrapReply::Hello {
            version,
            contract,
            artifact,
            pid,
            provider,
            world,
        }) = BootstrapReply::read(&bytes)
        else {
            bail!(
                "unexpected handshake response: {}",
                String::from_utf8_lossy(&bytes)
            );
        };
        client.daemon = DaemonId::judged(pid, version, contract, artifact, provider, world);
        Ok(client)
    }

    /// Turn this connection into a subscription: `subscribe`, then
    /// `subscribed`, after which no request is sent on it.
    async fn subscribe(mut self) -> Result<Subscription> {
        match self.request(&Request::Subscribe).await? {
            Response::Subscribed => Ok(Subscription {
                reader: self.reader,
                _write: self.write,
                daemon: self.daemon,
            }),
            Response::Error { kind, message } => Err(refused(kind, message)),
            other => bail!("unexpected response to subscribe: {other:?}"),
        }
    }

    /// Write one frame and read the next one, raw: the one-in-flight
    /// discipline both planes share. An answer over the frame bound is an
    /// error here, and the connection stays aligned on the next frame.
    async fn exchange<T: Serialize>(&mut self, frame: &T) -> Result<Vec<u8>> {
        let mut line = serde_json::to_string(frame)?;
        line.push('\n');
        self.write.write_all(line.as_bytes()).await?;
        loop {
            match read_frame(&mut self.reader, MAX_FRAME_BYTES).await? {
                Frame::Closed => bail!("daemon closed the connection"),
                Frame::Oversize => {
                    bail!("the daemon's answer exceeds {MAX_FRAME_BYTES} bytes and was discarded")
                }
                Frame::Line(bytes) if bytes.iter().all(u8::is_ascii_whitespace) => continue,
                Frame::Line(bytes) => return Ok(bytes),
            }
        }
    }

    /// The daemon this client reached, as its handshake identified it.
    pub fn daemon(&self) -> &DaemonId {
        &self.daemon
    }

    /// "mock" or "ggg", from the handshake of the daemon this client reached.
    pub fn provider(&self) -> &str {
        &self.daemon.provider
    }

    /// Send a request and return its response: one frame out, one frame
    /// in (C85). This connection never carries events — one arriving here
    /// is a protocol violation, reported as such.
    pub async fn request(&mut self, req: &Request) -> Result<Response> {
        let bytes = self.exchange(req).await?;
        match serde_json::from_slice::<Response>(&bytes)? {
            Response::Event { .. } | Response::ResyncRequired { .. } => {
                bail!("protocol violation: an event arrived on a request connection")
            }
            other => Ok(other),
        }
    }

    /// `request` variants that unwrap the expected response shape.
    pub async fn expect_ack(&mut self, req: &Request) -> Result<()> {
        match self.request(req).await? {
            Response::Ack => Ok(()),
            Response::Error { kind, message } => Err(refused(kind, message)),
            other => bail!("unexpected response: {other:?}"),
        }
    }

    pub async fn status(&mut self, id: u64) -> Result<JobInfo> {
        match self.request(&Request::Status { id }).await? {
            Response::Status { job } => Ok(job),
            Response::Error { kind, message } => Err(refused(kind, message)),
            other => bail!("unexpected response: {other:?}"),
        }
    }

    /// The daemon's read-only, non-reserving projection over `jobs`
    /// (see [`acquisition_protocol::protocol::Quote`]). Sends nothing; shared here so
    /// every frontend's quote surface speaks the same request.
    pub async fn quote(
        &mut self,
        jobs: Vec<acquisition_protocol::protocol::QuoteJob>,
        account: Option<String>,
    ) -> Result<acquisition_protocol::protocol::Quote> {
        match self.request(&Request::Quote { jobs, account }).await? {
            Response::Quote { quote } => Ok(quote),
            Response::Error { kind, message } => Err(refused(kind, message)),
            other => bail!("unexpected response: {other:?}"),
        }
    }
}

/// A daemon this client started: the executable it ran, its log, and
/// where that log ended at spawn time.
struct Spawned {
    child: std::process::Child,
    acqd: PathBuf,
    log: PathBuf,
    log_from: u64,
}

/// Start the `acqd` beside this executable (C82) with no arguments — its
/// knobs are the environment this process passes on — and its stdio to
/// null: the daemon log is where it speaks. The use path creates the
/// world's root first (C83), so the daemon locks and reports the same
/// canonical directory this process will compare; a relative
/// `ACQ_STORE_DIR` crosses into the daemon made absolute.
fn spawn_daemon() -> Result<Spawned, ConnectError> {
    let acqd = locator::acqd().map_err(|e| ConnectError::SpawnFailed {
        acqd: None,
        log: e.to_string(),
    })?;
    let world = World::create().map_err(|e| ConnectError::SpawnFailed {
        acqd: Some(acqd.clone()),
        log: format!("could not create the world's root before starting it: {e}"),
    })?;
    let log = world.log_path(want_provider());
    // Where the log ends now; lines past this offset are the new daemon's.
    let log_from = std::fs::metadata(&log).map_or(0, |m| m.len());
    let mut command = std::process::Command::new(&acqd);
    if std::env::var_os("ACQ_STORE_DIR").is_some() {
        command.env("ACQ_STORE_DIR", acquisition_store::world::intended_root());
    }
    let child = command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| ConnectError::SpawnFailed {
            acqd: Some(acqd.clone()),
            log: format!("failed to spawn it: {e}"),
        })?;
    Ok(Spawned {
        child,
        acqd,
        log,
        log_from,
    })
}

/// What the daemon wrote to its log after we spawned it — the only place a
/// lazy-spawned daemon's startup refusal lands. A log shorter than the
/// offset was rotated by the daemon at its start (the diagnostics are
/// bounded, C83), so everything in it is the new daemon's.
fn startup_log_excerpt(spawned: &Spawned) -> String {
    use std::io::{Read, Seek, SeekFrom};
    let path = &spawned.log;
    let tail = std::fs::File::open(path).ok().and_then(|mut f| {
        let len = f.metadata().ok()?.len();
        let from = if len < spawned.log_from {
            0
        } else {
            spawned.log_from
        };
        f.seek(SeekFrom::Start(from)).ok()?;
        let mut s = String::new();
        f.read_to_string(&mut s).ok()?;
        let lines: Vec<&str> = s.trim().lines().collect();
        // A refusal is a few lines; cap so a crash after a busy start
        // doesn't flood the terminal.
        let last = &lines[lines.len().saturating_sub(20)..];
        (!last.is_empty()).then(|| last.join("\n  "))
    });
    match tail {
        Some(t) => format!("; the daemon log says:\n  {t}"),
        None => format!(
            "; its log ({}) has nothing new — it may have failed before opening it",
            path.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tests that read or set the process environment take this
    /// first: `set_var` is unsafe beside a concurrent read. Tokio's, so
    /// the async test can hold it across its awaits.
    static ENV: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn id(contract: &str, provider: &str) -> DaemonId {
        DaemonId::judged(
            42,
            VERSION.into(),
            contract.into(),
            None,
            provider.into(),
            own_world_name(),
        )
    }

    /// C10, C84: the handshake compares the contract revision and the
    /// artifact, not the package version — a daemon reporting another
    /// contract is not this client's, one reporting no artifact matches
    /// nothing on that dimension (this test executable has no sibling
    /// `acqd`, so no daemon can match it there; the artifact's own cases
    /// are `artifact.rs` and the process test) — and the provider is the
    /// third dimension, reported with the others. The tests run without
    /// `ACQ_GGG`, so "mock" is the wanted provider.
    #[test]
    fn c84_a_daemon_of_another_contract_artifact_or_provider_is_not_this_clients() {
        let _env = ENV.blocking_lock();
        let dir = std::env::temp_dir().join(format!("acq-verdict-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // SAFETY: `ENV` is held; the crate's other environment-reading
        // tests take it too.
        unsafe {
            std::env::set_var("ACQ_STORE_DIR", &dir);
            std::env::remove_var("ACQ_GGG");
        }
        assert_eq!(CONTRACT_REVISION.len(), 12);
        assert!(CONTRACT_REVISION.bytes().all(|b| b.is_ascii_hexdigit()));
        assert!(acquisition_protocol::VERSION_WITH_CONTRACT.contains(CONTRACT_REVISION));

        let same_contract = id(CONTRACT_REVISION, "mock");
        let verdict = same_contract.verdict();
        assert!(verdict.contract && verdict.provider && verdict.world);
        assert!(matches!(verdict.artifact, ArtifactVerdict::Unreported));
        assert!(
            !same_contract.is_ours(),
            "no artifact reported matches nothing"
        );
        let report = same_contract.report();
        assert_eq!(report["contract_matches"], true);
        assert_eq!(report["artifact_matches"], false);
        assert_eq!(report["provider_matches"], true);
        assert_eq!(report["wanted"]["contract"], CONTRACT_REVISION);
        assert_eq!(report["wanted"]["provider"], "mock");
        assert!(
            report["artifact_mismatch"]
                .as_str()
                .unwrap()
                .contains("no artifact")
        );
        let text = same_contract.to_string();
        assert!(
            text.contains("another artifact") && !text.contains("another contract"),
            "{text}"
        );

        assert!(!id(CONTRACT_REVISION, "ggg").is_ours());
        assert!(!id("deadbeefdead", "mock").is_ours());
        let both = id("deadbeefdead", "ggg");
        let report = both.report();
        assert_eq!(report["contract_matches"], false);
        assert_eq!(report["provider_matches"], false);
        let text = both.to_string();
        assert!(
            text.contains("another contract")
                && text.contains("another artifact")
                && text.contains("another provider"),
            "{text}"
        );
        let text = id(CONTRACT_REVISION, "ggg").to_string();
        assert!(
            text.contains("another provider") && !text.contains("another contract"),
            "{text}"
        );

        // C83: another world is judged first and named alone — the other
        // dimensions are not compared for a daemon this process would
        // neither use nor replace; the report still carries them all.
        let elsewhere = DaemonId::judged(
            42,
            VERSION.into(),
            CONTRACT_REVISION.into(),
            None,
            "mock".into(),
            "/somewhere/else".into(),
        );
        assert!(!elsewhere.world_matches() && !elsewhere.is_ours());
        let report = elsewhere.report();
        assert_eq!(report["world"], "/somewhere/else");
        assert_eq!(report["world_matches"], false);
        assert_eq!(report["contract_matches"], true);
        assert_eq!(
            report["wanted"]["world"],
            dir.canonicalize().unwrap().display().to_string()
        );
        assert!(report["wanted"].get("world_absent").is_none(), "{report}");
        let text = elsewhere.to_string();
        assert!(
            text.contains("another world")
                && text.contains("/somewhere/else")
                && !text.contains("another artifact"),
            "{text}"
        );
        // An absent world of this process's own matches no daemon and
        // says why under `wanted`.
        unsafe {
            std::env::set_var("ACQ_STORE_DIR", dir.join("missing"));
        }
        let nowhere = id(CONTRACT_REVISION, "mock");
        assert!(!nowhere.world_matches(), "{nowhere}");
        let report = nowhere.report();
        assert!(
            report["wanted"]["world_absent"]
                .as_str()
                .unwrap()
                .contains("does not exist"),
            "{report}"
        );
        assert!(nowhere.to_string().contains("absent"), "{nowhere}");
        unsafe {
            std::env::remove_var("ACQ_STORE_DIR");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The absence reason is the caller's policy before the knob, and the
    /// two read differently: a door that never spawns says so, a door
    /// the knob closed names the knob (review, 2026-09-10: the MCP's
    /// real-mode absence had claimed "it spawns on demand").
    #[tokio::test]
    async fn an_absent_daemon_names_why_it_was_not_started() {
        let _env = ENV.lock().await;
        let dir = std::env::temp_dir().join(format!("acq-absent-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // SAFETY: `ENV` is held; the crate's other environment-reading
        // tests take it too.
        unsafe {
            std::env::set_var("ACQ_SOCKET", dir.join("none.sock"));
            std::env::remove_var("ACQ_NO_SPAWN");
        }
        let err = Client::connect(ConnectOptions::autonomous(false))
            .await
            .err()
            .expect("no daemon");
        assert!(
            matches!(
                err,
                ConnectError::Absent {
                    because: NotSpawned::Policy
                }
            ),
            "{err:?}"
        );
        let text = err.to_string();
        assert!(
            text.contains("does not start one") && !text.contains("on demand"),
            "{text}"
        );
        unsafe {
            std::env::set_var("ACQ_NO_SPAWN", "1");
        }
        let err = Client::connect(ConnectOptions::interactive(true))
            .await
            .err()
            .expect("no daemon");
        assert!(
            matches!(
                err,
                ConnectError::Absent {
                    because: NotSpawned::NoSpawnEnv
                }
            ),
            "{err:?}"
        );
        assert!(err.to_string().contains("ACQ_NO_SPAWN"), "{err}");
        unsafe {
            std::env::remove_var("ACQ_NO_SPAWN");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A scripted peer on a scratch socket: answers the handshake as a
    /// daemon would, then answers the stop request with `reply` — or
    /// hangs up when `reply` is `None`.
    async fn peer_that_answers_stop_with(reply: Option<serde_json::Value>) -> UnixStream {
        let dir = std::env::temp_dir().join(format!("acq-stop-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = dir.join(format!("{n}.sock"));
        let _ = std::fs::remove_file(&path);
        let listener = tokio::net::UnixListener::bind(&path).unwrap();
        let peer_path = path.clone();
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read, mut write) = stream.into_split();
            let mut lines = tokio::io::AsyncBufReadExt::lines(BufReader::new(read));
            let _hello = lines.next_line().await.unwrap().unwrap();
            let hello = BootstrapReply::Hello {
                version: VERSION.to_string(),
                contract: CONTRACT_REVISION.to_string(),
                artifact: None,
                pid: 7,
                provider: "mock".into(),
                world: own_world_name(),
            };
            let mut line = serde_json::to_string(&hello).unwrap();
            line.push('\n');
            write.write_all(line.as_bytes()).await.unwrap();
            let _stop = lines.next_line().await.unwrap().unwrap();
            if let Some(reply) = reply {
                let mut line = serde_json::to_string(&reply).unwrap();
                line.push('\n');
                write.write_all(line.as_bytes()).await.unwrap();
            }
            // Dropping `write` hangs up either way.
            let _ = std::fs::remove_file(&peer_path);
        });
        UnixStream::connect(&path).await.unwrap()
    }

    /// The defect of review round 1: a stop that the peer refuses, answers
    /// strangely, or drops must never come back as a stop. Only
    /// `Stopping` counts.
    #[tokio::test]
    async fn a_stop_the_daemon_did_not_acknowledge_is_not_a_stop() {
        let stream =
            peer_that_answers_stop_with(Some(json!({ "resp": "error", "message": "busy" }))).await;
        let err = Client::stop_over(stream).await.unwrap_err().to_string();
        assert!(
            err.contains("refused to stop") && err.contains("busy"),
            "{err}"
        );

        let stream = peer_that_answers_stop_with(Some(json!({ "resp": "ack" }))).await;
        let err = Client::stop_over(stream).await.unwrap_err().to_string();
        assert!(err.contains("unexpected response to stop"), "{err}");

        let stream = peer_that_answers_stop_with(None).await;
        let err = Client::stop_over(stream).await.unwrap_err().to_string();
        assert!(err.contains("closed the connection"), "{err}");

        let stream = peer_that_answers_stop_with(Some(json!({ "resp": "stopping" }))).await;
        let stopped = Client::stop_over(stream).await.unwrap();
        assert_eq!(stopped.pid(), 7);
    }
}
