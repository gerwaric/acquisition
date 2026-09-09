# The daemon split — the frontend boundary drawn as crates and a wire

**Written 2026-09-09, revised the same day after review rounds 1 (§8) and 2 (§9)**,
the design session `17-framing-the-daemon-split.md` framed. Disposable
(P1): what it proposes is real only as a ruling in the registry or as
code under the gate. Current state only: superseded content is edited
in place and git holds round 1. No code was written; the tree is at
`16283b81` with the measurement edits reverted. Read 17 first; this
note does not restate its floor.

The question, in the owner's words: the daemon is becoming its own
artifact, and "what is the daemon, as a thing every frontend shares"
is asked for the first time. This note answers it, measures the ground,
proposes decision lines in registry form, a commit sequence, and a
parking lot.

## 1. What was measured (tree at `16283b81`, same machine as 16, clean `target/`)

| Experiment | Result |
| --- | --- |
| a semantic edit to `daemon.rs` (a new `pub fn`), then `cargo test --workspace --all-targets --no-run` | 6.7 s; core, plan, cli, mcp recompiled; **21 executables relinked** |
| the same edit to `protocol.rs` | 6.6 s, the same set |
| the same edit to `store/src/lib.rs` | 6.5 s (build), the same set plus store |
| the same edit to `cli/src/main.rs` | 2.8 s; the CLI and its 12 test executables only |
| `cargo test --workspace --all-targets`, full run | 17.0 s wall; 21 executables; 12 CLI + 3 MCP integration files, each a full-stack link with its own copy of a `command(base, args)` harness |
| dependency closure (`cargo tree -e normal`) | store 31 crates, core 159, plan 167, mcp 203, cli 219 |
| `acquisition_core::` import sites outside core | 46 lines in 20 files: `realm` 15, `protocol` 6, `client` 5, `provider::ggg_mode` 5, `job` 4, the version constants 4, `daemon` 4 (`run`, `MAX_429_RETRIES`, `socket_path`, `log_path`), `ratelimit` status types 2, `rails::RailsStatus` 1 |
| `daemon.rs` | 8,138 lines; the test modules begin at line 3,891 (about half the file is its in-process harness: 60 tests, driving `handle_request` directly, not the socket) |
| the embedded daemon | one spawn site (`client.rs::spawn_daemon`, `current_exe()` + `daemon run`); two handlers (`acq daemon run`; `acq-mcp` intercepts argv before `rmcp`); two tests start it directly (`daemon_observe.rs`, `plan_loop.rs`); two drivers do (`tracer-rung.sh:210`, `persist-check.sh:110`) |
| `target/debug/acq`, `acq-mcp` | 35.6 MB, 34.6 MB |
| SHA-256 of one 35 MB executable (`shasum`, page-cached) | 60–70 ms; a `stat` under 1 ms |
| cargo artifact dependencies (`artifact = "bin"`) | **unstable** in cargo 1.94.1: `requires -Z bindeps` |
| the `test-hooks` feature | one five-line hook (`jobs.rs:122`), five callers, all in `daemon.rs`'s tests; core's dev-dependency turns it on, so the workspace still builds twice per configuration (16 §1.3) |

Verified for round 2 (§8), all from the code:

| Claim | Where | What is true |
| --- | --- | --- |
| the rails state lives beside the socket | `rails.rs:84`, `main.rs` reset-tripwire | `<socket>.<provider>.rails.json`; the socket's default directory is `std::env::temp_dir()`, which on macOS is the per-user `$TMPDIR` the OS clears at reboot — **a tripped tripwire does not survive a restart today** |
| a request an old daemon cannot parse | `daemon.rs:2978` | answered `Error { message: "bad request: …" }`; the client's handshake then fails with "unexpected handshake response", not with a revision — a mismatch is diagnosable only while `Hello` still parses |
| events are lossy | `daemon.rs:3666`, `client.rs::request` | a lagged broadcast receiver skips what it missed; the client drops events while awaiting a response |
| frames are unbounded | `daemon.rs:2971` | `BufReader::lines()`, no maximum |
| `ACQ_SOCKET` setters | `tools/acq-as.sh`, the harnesses, the mock-session skill | the script is the rung-11 helper: "EXPERIMENT-ONLY. Two real daemons on one machine violate P-B" — a closed experiment |
| the status types on the wire | `ratelimit.rs:1305–1350`, `rails.rs:156` | already DTOs ("for the dashboard"), distinct from the internal `Policy`/`Window`/`Limiter` |
| `cargo test --workspace --all-targets` uplifts every bin | a scratch workspace: package `d` (bin only), package `f` (bin + one integration test) | **it does not**: `target/debug/f` appears, `target/debug/d` does not — `d`'s bin is built only as a hashed test harness under `deps/`. The gate must build before it tests (2.6) |
| the failing pricing property (§7) | `game_side.rs:283`, `:607`; the reader at `:225` | the reader tolerates a suffix after the currency word by documented design ("tolerated, not read"); the property's inverse grammar takes everything after the amount as the word for a note — **the test is the defective side**, one line |

On this clean tree the seconds are small; the 16 s incremental rebuilds
of 16 §1 were the bloated tree's doing. What the split buys in build
time is modest in seconds and large in *count*: a daemon edit today
relinks 21 executables and recompiles two frontends and the planner,
none of which can feel the change. The payoff to record is the
boundary; the build graph is its measurable shadow.

## 2. The boundary

### 2.1 The crates

Four library crates below the frontends, each with one job and one set
of permitted edges. `acquisition-core` empties and is renamed
`acquisition-daemon` with `git mv`, history intact.

| Crate | Holds | Links | Never links |
| --- | --- | --- | --- |
| `acquisition-store` | facts, intent, the effects-ledger facade (C45), and **the world** (`world.rs`): the canonical provider-neutral root (`ACQ_STORE_DIR` or the platform data dir, canonicalised), the per-provider directories, `daemon.db`, the rails state, the lock files, the socket name for the world and the private runtime directory — pure path functions, no IPC | rusqlite, directories, serde | protocol, client, daemon, plan, reqwest, tokio |
| `acquisition-protocol` | the wire: `Request`/`Response`, the job model, `Quote`, the status DTOs (`RailsStatus`, `PolicyStatus`, `RuleStatus`, `WindowStatus`, `SendRecord`, `DegradedEndpoint`); the job vocabulary typed — `target_of`, `Realm`, `Family::accepts`; the daemon's promises a consumer computes with (`MAX_429_RETRIES`); the stable bootstrap plane (2.7); the shared-contract revision (2.2, the build script moved here) | serde, serde_json (sha2 at build time) | tokio, anyhow, store, client, daemon |
| `acquisition-client` | the IPC over tokio, the three policy doors of C10, `DaemonId` in three dimensions, the `acqd` locator (2.3), world verification (2.4), typed connect and startup errors | protocol, store (the world), tokio | daemon, plan |
| `acquisition-daemon` (binary `acqd`) | `daemon.rs`, `ratelimit.rs`, `gate.rs`, `rails.rs`, `auth.rs`, `mockggg.rs`, the `Provider` (URLs, client id, user-agent, keyring service) — untouched inside (rewrite gravity); the only GGG sender | protocol, store | client, plan, any frontend |
| `acquisition-plan` | the planner | protocol, store | client, daemon |
| frontends (`acquisition-cli`, `acquisition-mcp`, the GUI and TUIs to come) | presentation | client, protocol, store, plan — as each needs them; a queue TUI links no planner | daemon |

`ggg_mode()` and the provider names go to the protocol crate (which
provider this process wants is part of the handshake); `socket_path`
and `log_path` become the world's. What the protocol crate is: **the
daemon's contract as a frontend sees it**, with nothing in it that can
change for a client-side reason — so the revision that hashes it moves
only when the contract does. The client crate exists because a client
or locator edit must not recompile the daemon or move the compatibility
revision, and because the GUI and MCP need typed failures at the door
(`ConnectError::{Absent, Incompatible(DaemonId), OtherWorld, SpawnFailed
{log}, Transport}`) rather than strings. `Realm` goes to the protocol
crate: three consumers already, all consumers of the vocabulary; not a
one-file model crate (ceremony), not the store (C58: string-typed, never
validates).

One consequence, stated: a queue TUI links client, which links the
store, so no frontend is SQLite-free. If a store-free frontend ever
matters, `world.rs` becomes its own tiny crate (parked).

The edges `tools/docs-check.sh` refuses, from the table:

```
store     ∌ protocol, client, daemon, plan, reqwest, tokio
protocol  ∌ tokio, anyhow, store, client, daemon
client    ∌ daemon, plan
daemon    ∌ client, plan, any frontend
plan      ∌ client, daemon
frontends ∌ daemon           (no package but acquisition-daemon names it)
daemon/src, protocol/src, client/src ∌ the intent API
```

C13's "never in-process with the daemon" stops being a sentence.

### 2.2 The identity model: two values, two sources

C10 as amended (16 §7.1) was written so the split changes a doc comment
and not the ruling; here is the doc comment's content.

- **Shared-contract revision** — "can I use it". A digest of the two
  surfaces every frontend shares (C12): the protocol crate's sources and
  the store crate's sources, plus the manifests and `Cargo.lock` —
  today's input set with the daemon's own sources removed. Computed by
  the build script that moves to the protocol crate (its
  `rerun-if-changed` on store sources adds no invalidation: a store edit
  already recompiles everything). Both sides link the protocol crate, so
  both carry it. The lock stays in: a frontend rebuilt alone after a
  lock change has new dependencies against an old sibling, and nothing
  else would see it.
- **Daemon artifact** — "is it the one I would start". At startup the
  daemon resolves `current_exe()`, records its file identity (device,
  inode, length, mtime) and SHA-256s it once (~60 ms, once per
  lifetime), and reports all of it in `hello`; the hash also goes to the
  journal header and the first log line, where the run record's
  `provenance.json` matches it exactly. A client reads the file identity
  of the sibling `acqd` it would spawn: equal identity is the same file;
  any difference — a copy at another path, an atomic replacement at the
  same path — is hashed (60 ms, the rare path) and compared, so a copy
  is still the same artifact and a rebuild under a live daemon is a
  named mismatch ("the daemon's file changed since it started").
- **Provider** — as today.

Why both (17's Q5): each has a failure the other cannot see. With the
artifact alone, `cargo build -p acquisition-cli` after a wire or store
change runs a new `acq` against an old `acqd` that matches itself — a
silent contract mismatch, and for the store a new reader migrating
files an old daemon still writes. With the contract alone, a daemon-only
edit never changes the identity and a stale daemon keeps serving.
Together, the partial-rebuild case is loud: the client sees a contract
mismatch, replaces, spawns the same old file, and reports "still, after
a respawn" (the loop the client already detects). Daemon-only sources
move neither value but the artifact, which is where the build-graph win
comes from.

Whether artifact equality is a *use* condition or a freshness signal is
settled by the installation premise (2.3): with one installation per
world there is exactly one artifact, C10 stands as ruled, and the
alternative — contract as the hard condition, artifact as interactive
replacement policy — is recorded as the design for a multi-installation
future, not built.

What this closes: "the identity dimension has no process-level test"
(the accepted residual in `client.rs`). A test copies `acqd` to a
scratch path, starts it, and runs `acq` whose sibling is the original:
an artifact mismatch, staged with one binary per test run.

What it changes on the surfaces: `acq --version` says `acq 0.0.1
(contract <rev>)`; `acq version --json` prints `{version, contract,
provider, acqd: {path, len, modified} | null}` (the sibling as found,
no hash — the preflight hashes `acqd` itself); `hello` carries
`version`, `contract`, `artifact`, `provider`, `world`, `pid`; the
observer report has `contract_matches`, `artifact_matches`,
`provider_matches`; the journal header's `runtime` field becomes
`contract` and `daemon` (the artifact hash). **This supersedes note 16
§6.4** ("no second hash in the journal"): once no revision names the
daemon's own code, the journal needs the identity that does, and the
artifact hash is it; `provenance.json` maps it to HEAD as before. The
standing rule's "Build before you run" bullet (owner-approved
2026-09-09) changes wording once more; the text is proposed in step 4's
commit for approval.

### 2.3 Installation topology, then the locator

**The premise this session needs ruled, not assumed.** Acquisition ships
as a desktop app. The way a desktop app with a companion CLI ships on
every platform is one directory holding every binary: a Tauri bundle
carries `acqd`, `acq` and `acq-mcp` as sidecars beside the GUI
executable and the CLI reaches the shell through a symlink into `PATH`
(the VS Code and Docker Desktop shape); Windows and Linux are one
install directory each. Per-frontend installation — `cargo install` of
the CLI alone — is how libraries and standalone tools ship, not how a
GUI product with a shared daemon does; it buys nothing here and costs
the precedence machinery a multi-installation design needs. So, phrased as
what **Acquisition requires** rather than what every platform provides:
**at most one installation targets a world; Acquisition's installation
places one canonical `acqd` and every frontend beside it.** A second
installation targeting the *same world* is a design event, recorded
first (C46's mold). The developer's machine always has two
installations — the shipped app and the playground — and they target
different worlds (their data directories differ), so they are not that
event; the real-mode lock (2.4) is what keeps them from making two GGG
gates, the scenario `tools/acq-as.sh` was written to violate on
purpose. Tauri's public contract promises name-based sidecar launching,
not that every frontend rediscovers the sidecar as
`current_exe().parent()/acqd`, and `current_exe()` is documented as
platform-specific around symlinks and renames: so the layout is a
contract Acquisition's packaging must meet, and packaging smoke tests
(§5) are its acceptance criterion — GUI → `acqd`, the canonicalised CLI
symlink → the same `acqd`, MCP → the same `acqd`, updater replacement
and restart. The
Tauri updater replacing the bundle under a live daemon is an artifact
mismatch the next use verb resolves: kill-and-respawn doing its job.

**The executable.** `acqd` is found beside the calling executable —
`current_exe()` canonicalised, then its parent — and nowhere else: no
`PATH` search (another install's daemon), no configured path (no
consumer needs one), no embedded mode. A frontend without a sibling
`acqd` cannot spawn and says so (`ConnectError::SpawnFailed`). Tests
and drivers locate the daemon the same way — 17's seed realised: one
way to find the daemon, used by everything.

### 2.4 The world: rendezvous and ownership

A socket name is discovery, not ownership: it can be unlinked under a
live daemon, an alias of the root hashes differently, and two custom
roots start two real-mode daemons with two GGG gates. C6 (one daemon
per store directory) and C31 (one per machine in real mode) are held
today by documentation. The world makes them structure:

- **The world root** is canonical (`fs::canonicalize`) and
  provider-neutral: `ACQ_STORE_DIR` or the platform data directory,
  above the `<provider>/` directories the store helper returns today.
  `canonicalize` fails on a path that does not exist, which is the
  first-run spawn, so the behaviour is defined: a **use** path creates
  the root (mode 0700), then canonicalises and locks it; an
  **observation** path creates nothing — a missing root is an absent
  world (the legacy endpoint still checked); a relative `ACQ_STORE_DIR`
  is made absolute in the client before it crosses into `acqd`. In the
  daemon the lock is taken before the rails migration, before
  `daemon.db` opens, before the provider initialises, before anything
  that can send.
- **The world lock**: `<root>/daemon.lock`, an exclusive advisory lock
  (`flock`) held for the daemon's lifetime; a second daemon on the root
  refuses to start naming the holder.
- **The real-mode lock**: `<runtime>/acq/ggg.lock`, per OS user, held
  by any real-mode daemon regardless of root. Two live-test roots cannot
  make two GGG gates. Per user is not per machine: a second logged-in
  OS user is outside this lock, as are the C++ Acquisition on the same
  machine and other machines behind the same address. C31's *ruling*
  stands (one daemon, many sessions; never one per account); its
  **enforcement scope** is named honestly as one real daemon per OS
  user, with those three as external concurrency the daemon cannot
  structurally control and the tripwire exists for — a machine-wide
  lock would cover only the first of them at the cost of shared-file
  permissions, and is not worth it. Proposed as a C31 wording edit
  (§3).
- **The socket**: `<runtime>/acq/<12 hex of the root>.sock`, in a
  private per-user runtime directory (`$XDG_RUNTIME_DIR` on Linux, the
  per-user `$TMPDIR` on macOS, mode 0700), short by construction.
  Nothing chooses it by hand: `ACQ_SOCKET` is removed, its harness and
  skill setters with it, and `tools/acq-as.sh` retires (rung 11 closed;
  what it did is what C31 forbids).
- **Verification**: `hello` carries the daemon's canonical root and
  provider; a client compares the root with its own before anything
  else and refuses a daemon on another world (`ConnectError::OtherWorld`)
  — a truncated hash is discovery, never proof.
- **Durable state moves into the world; diagnostics stay bounded.**
  Before the rendezvous changes, the rails state moves to
  `<root>/<provider>/rails.json` beside `daemon.db` (a one-time move of
  the legacy file if present, so no trip is lost): today's placement in
  the temp directory loses a tripped tripwire at reboot. The log and
  the send journal are append-only diagnostics and do not become
  unbounded durable state: they live in the platform's log or state
  directory, one subdirectory per world and provider, and are bounded —
  rotated once at daemon start past a cap (the size and mechanism are
  internals; "diagnostics are bounded" is the property). A live run
  keeps overriding the journal into its evidence directory
  (`ACQ_JOURNAL`), where the ledger cites it.
- **Migration, with its boundary stated**: a daemon on the old fixed
  socket (`<tmp>/acquisition-playground.sock`) would be invisible to a
  new client, and a second daemon would start. For one release the
  daemon probes the legacy default path at start and refuses if it is
  alive, and a client that finds its world's socket absent probes it
  and reports what it found with the stop remedy; `stop_any` stops
  either. Pinned by a process test that stages a daemon on the legacy
  path. What is *not* discovered automatically: the arbitrary
  `ACQ_SOCKET` values of history (`/tmp/acq-tracer.sock`,
  `/tmp/acq-persist.sock`, the rung-11 `A`/`B` pair, a session's
  `/tmp/acq-<name>.sock`). Those are closed procedurally: the
  transition commit's preflight and the live-run skill prove no old
  playground daemon remains, the known driver endpoints named, before
  the first run on the new rendezvous. A controlled migration, not
  comprehensive detection.

### 2.5 Lifecycle as a product surface

With one artifact, `acq daemon status` answers the question every
frontend will ask the same way: pid, provider, world, contract, the file
it runs from and whether it is the sibling this client would start, and
which dimensions differ. The GUI, a queue TUI and an agent over MCP
render the same `DaemonId::report`.

- **Who starts it.** The interactive CLI and the GUI are use frontends:
  they spawn the sibling `acqd` on first need and replace a mismatch (a
  human's intent). The daemon is never hosted in the GUI process — the
  embedding the owner called terrible, one door over. A GUI holding a
  subscription keeps the daemon alive (C3's idle rule): the gpg-agent
  model doing its job.
- **Frontends that never spawn** — a real-mode MCP, a cron sync under
  `ACQ_NO_SPAWN` — are told `Absent` with the remedy, which today is
  "run a job command". An explicit `acq daemon start` was proposed in
  round 1 and withdrawn: a daemon started with no work idles out in
  60 s and the MCP connects per call, so the verb serves neither. What
  those consumers need is a keeper — a supervisor model with a
  foreground `acqd` contract (readiness, signals, exit codes, graceful
  checkpointing) — parked with that trigger; the daemon verbs stay
  debugging-only (C3).
- **The MCP's mock-mode spawn** stays (17's Q4): the split changes what
  it spawns, not whether. The trigger to revisit is a second spawner on
  one machine (the GUI).
- **The drivers** start `acqd` directly and own the pid, as they own
  `daemon run` today.

### 2.6 Tests find the daemon the way frontends do

Two packages (17's Q2): a package carrying both `acq` and `acqd` would
link the daemon crate for both, and "frontends never link the daemon"
could not be a Cargo.toml fact. `CARGO_BIN_EXE_acqd` therefore exists
only in the daemon package's own tests; the CLI and MCP process tests
spawn the sibling of their own binary through the same locator, and a
missing sibling fails **before the test runs**, naming `cargo build
--workspace`.

Hermeticity comes from the gate, and the gate changes: `cargo test
--workspace --all-targets` does **not** uplift a bin from a package that
has no integration tests — verified in a scratch workspace (§1) — so a
`target/debug/acqd` built by the test run alone is not guaranteed. The
authoritative gate becomes `cargo build --workspace` **then** `cargo
test --workspace --all-targets` (AGENTS.md, in step 3's commit): the
exact sibling is rebuilt before any test runs, with no override and no
unstable artifact dependency. A `-p` run of a process test is
unsupported, and the harness says so when the sibling is absent; when it is present but stale, the
harness prints the daemon's path and artifact hash with any failure,
so a wrong daemon is never a mystery. A test-only path override was
considered and refused: the process tests drive the real `acq`, so the
override would be read by the production client — a knob by another
name. Artifact dependencies (`-Z bindeps`, unstable) are the proper
fix, parked with that trigger.

The daemon's in-module harness stays where it is (TESTING-NOTES item 1
fires on a fresh build, not on this split). The `test-hooks` retirement
and the one-test-binary-per-crate consolidation are test-build
economics, not split requirements, and go to a later change; the named
landing for the hook is a job-store trait the daemon's tests substitute
— not an unconditional `#[doc(hidden)]` method, which would be a
destructive door in production code to save a duplicate build of a
17-second suite.

### 2.7 Pinning the wire: the contract, then the fixtures

TESTING-NOTES item 3 — "pin the frontend boundary, after the consumer
has validated the protocol" — has its consumers (the tracer, the MCP
plan loop, the pricing readings). But fixtures pin serialization, not
behaviour, and four boundaries have to be settled before the current
shape is canonised, because the GUI and the TUI arrive to them:

- **A stable bootstrap plane.** `hello` and `daemon_stop` are parsed
  outside the versioned enum, as raw frames every version reads
  (`{"req":"hello", …}` → a `Value`; the daemon's reply likewise), so a
  wire change can always report itself as a contract mismatch and a
  human can always stop the daemon across one. Today an incompatible
  enum fails before the revision is compared (§1).
- **A frame bound.** A maximum line length on both sides; an oversize
  or malformed frame is answered with an error and the connection
  stays, never a silent close. JSON lines stay (C9).
- **Connection semantics.** One request in flight per connection; a
  subscription takes a connection of its own and carries no requests —
  a `Subscription` type in the client crate, distinct from `Client`,
  makes that structural; events are invalidation hints, never a
  complete stream. A loss the client cannot observe cannot be recovered
  from, and today the daemon skips lagged events silently: so on
  broadcast lag the daemon sends `resync_required` (with the count),
  and the subscriber then re-reads, as it does after any disconnect.
  The race is pinned as a sequence: subscribe first, then take the
  authoritative snapshot over a request connection (`List`), then treat
  every event as a reason to re-read, and on `resync_required` or
  disconnect subscribe and snapshot again. This is what `dash` (polls)
  and `jobs --watch` already approximate; making it the contract avoids
  correlation ids until evidence asks for multiplexing.
- **Structured errors.** C47 wants stable kinds; the wire has
  `Error { message }`. The audit adds `kind` additively (C53: JSON
  changes are additive) as a **closed set** — an enum, pinned by the
  fixtures, never a free-form string — classifying the daemon's
  existing error sites (bad request, not logged in, unknown job,
  refused at admission, ambiguous account, rails halted, queue write
  failed, internal); subcategories and context grow additively with
  the first typed consumer, the GUI slice (P2). Designing the full
  taxonomy now is its own session, if the owner wants it first.

Then the pin: one JSON document per `Request`/`Response` variant under
`ACQ_UPDATE_FIXTURES=1` like the references, so every wire change from
here is a diff a reviewer sees, plus black-box contract tests between
the client crate and `acqd`: bootstrap across a mismatch, stop across a
mismatch, events and lag, malformed and oversize frames, disconnect,
restart. Commit the check, then break the code (TESTING-NOTES): this is
the first commit.

## 3. Candidate decision lines (registry form; byte counts after the block)

**C1 — amended (CONTEXT.md, cross-cutting):**

- **C1 — Cargo workspace, library-centric; the daemon is its own artifact.** `acquisition-store` holds facts, intent and the world (root, locks, socket name); `acquisition-protocol` the wire, the job vocabulary and the shared-contract revision, serde-only; `acquisition-client` the IPC, the spawn and observe policies and the `acqd` locator; `acquisition-daemon` (binary `acqd`) its only implementation and the only GGG sender; `acquisition-plan` the planner. A frontend links client, protocol, store and plan as it needs them, never the daemon; the daemon links protocol and store, never client, plan or a frontend; the store links none of them. *Why:* write/test logic once, and C12's two surfaces as edges the check refuses. *Pinned:* `tools/docs-check.sh`. Amended 2026-09-09.

**C82 — new (decisions/daemon.md):**

- **C82 — At most one installation targets a world.** Acquisition's installation places one canonical `acqd` and every frontend beside it; the implementation requires it as a sibling of the calling executable (`current_exe()` canonicalised) — no `PATH`, no configured path, no embedded mode; no sibling, no spawn, reported. A second installation targeting the same world is a design event, recorded here first (the playground beside the shipped app targets another world). Tests and drivers locate it the same way; packaging smoke tests are the acceptance criterion. *Why:* C10's "the runtime it would itself spawn" must name one file, or two installations thrash; one directory of siblings is what Acquisition ships. *Details:* `acquisition-client/src/locator.rs` doc. Ruled 2026-09-09.

**C83 — new (decisions/daemon.md):**

- **C83 — A world is a canonical, provider-neutral store root; its daemon holds an exclusive lock on it for its lifetime, and in real mode a per-OS-user lock no root bypasses.** A use path creates the root, then canonicalises and locks; observation creates nothing. The socket is derived from the root into a private per-user runtime directory, never chosen by hand (`ACQ_SOCKET` is gone); `hello` names the root and a client refuses a daemon on another world. Durable state (`daemon.db`, rails state) lives in the world; the log and journal are bounded diagnostics elsewhere. *Why:* a socket name is discovery, not ownership — C6 and C31 were held by documentation, and the tripwire's state lived in a directory the OS clears at reboot. *Details:* `world.rs` doc. Ruled 2026-09-09.

**C84 — new (decisions/daemon.md); C10's text is unchanged, this names what its "runtime identity" is:**

- **C84 — The runtime identity (C10) is two values from two sources: the shared-contract revision, a digest of the protocol and store sources, manifests and lock both sides are compiled from, and the daemon artifact — the executable, which the daemon identifies and hashes at startup and a client compares with the sibling `acqd` it would spawn.** A mismatch on either is reported by name; a rebuilt `acqd` under a live daemon is an artifact mismatch. Daemon-only sources move neither value but the artifact; nothing derives from git or a hand-kept number. *Why:* the contract answers "can I use it", the artifact "is it the one I would start"; each sees a failure the other cannot. *Details:* `client.rs` doc, C10. *Pinned:* `daemon_observe.rs`. Ruled 2026-09-09.

**C85 — new (decisions/daemon.md, beside C8 and C9):**

- **C85 — Connection semantics: one request in flight per connection; a subscription takes a connection of its own and carries no requests; events are invalidation hints, never a complete stream — a subscriber snapshots after subscribing, re-reads on every event, is told `resync_required` when it lagged, and after that or any disconnect subscribes and snapshots again; a frame has a bound, and an oversize or malformed one is answered with an error, never a closed socket; `hello` and `daemon_stop` are a stable plane every version parses.** *Why:* a loss a client cannot observe cannot be recovered from; the GUI and the TUI arrive to these, not to what `dash` happens to do. *Pinned:* `acquisition-protocol/tests/wire.rs`, `acquisition-client/tests/contract.rs`. Ruled 2026-09-09.

**C31 — one clause added (decisions/daemon.md), the ruling otherwise verbatim:** after "two daemons on one machine make a 4-wide burst neither sees" add "— enforced per OS user by the real-mode lock (C83); another OS user, the C++ Acquisition on the same machine and other machines behind the same address are external concurrency the tripwire exists for". The owner trims to the gate.

**Pointer edits, no new text:** C12 gains *Pinned:* `acquisition-protocol/tests/wire.rs`; C13's *Details* doc drops "embeds `daemon run` like `acq`"; C10's *Details* doc says "shared-contract revision and daemon artifact" (anticipated in 16 §7.1); C6's "one daemon per store directory is an invariant, not a lock" becomes "…is the world lock (C83)". README: "What exists" lists the crates; the knob table drops `ACQ_SOCKET` and reads `ACQ_GGG` in `acquisition-protocol`; the mock-session skill sets one knob. AGENTS.md's gate gains `cargo build --workspace` before `cargo test` (2.6). LIVE-TESTING's "Build before you run" bullet: proposed with step 4.

Rejected on the way, so nothing is re-argued or adopted by not noticing:

- One package for `acq` and `acqd` (test discovery for free) — the edge cannot be expressed; discipline where P5 wants structure.
- `acqd` on `PATH`, or `ACQ_DAEMON=<path>` — a second daemon to thrash with; no consumer.
- Per-frontend installation as a supported topology — recorded as a design event instead (2.3).
- Artifact mismatch as freshness only, with the contract as the sole use condition — the multi-installation design; not needed under one installation, kept as the recorded alternative.
- "Keep `daemon run` for the tests" — the embedding one door down (17's shortcut gravity).
- A hand-kept protocol version — refused by C10's text.
- The protocol revision without the store sources or the lock — round 1's error (§8.1).
- Hash-only artifact identity — 60 ms per `acq` call; file identity first, hash on any difference.
- Stat-only without device and inode — an atomic replacement at the same path could pass.
- One protocol crate holding the client and the locator — a client edit would move the revision and recompile the daemon (§8.5).
- A one-file model crate for `Realm` — ceremony; the vocabulary is the protocol's.
- Flags on `acqd` for the socket, store and rails — a second door to the same knobs; env stays the one door.
- The socket inside the store root (`<root>/acqd.sock`) — 77 bytes on the default macOS path, within the cap but not by enough; the runtime-directory name is short by construction.
- `ACQ_SOCKET` kept as an override — contradicts "never chosen by hand".
- `acq daemon start` — serves no non-spawning consumer (2.5).
- A test-only daemon path knob — read by production code (2.6).
- An unconditional `#[doc(hidden)]` destructive hook — a production door for a test-build saving (2.6).
- A machine-wide real-mode lock — covers a second OS user only, at the cost of shared-file permissions; the C++ app and NAT peers stay outside any lock (2.4).
- Closing the subscription on lag instead of signalling — needs the same reconnect logic and loses the count; `resync_required` is one variant (2.7).
- A free-form `kind` string — would not satisfy C47; a closed set does (2.7).
- The log and journal as durable world state — unbounded append-only files; diagnostics are bounded instead (2.4).

## 4. Commit sequence (each under the gate; 3, 4, 5 and 6 rehearsed with both drivers in `--mock`)

| # | Commit | Behaviour | Measure |
| --- | --- | --- | --- |
| −1 | restore green: the pricing property's inverse grammar takes the first whitespace-delimited word for a note, as the reader does; the proptest regression file committed so the case replays (§7) | none | the gate passes |
| 0 | the wire audit and pin (2.7): stable bootstrap frames, the frame bound, C85's semantics — `Subscription`, `resync_required`, the pinned subscribe-then-snapshot sequence — in client and daemon, the closed `kind` set, fixtures, black-box contract tests — the check before the code | bootstrap, framing, lag | — |
| 1 | `acquisition-protocol` extracted: the wire, the vocabulary, the promises, the bootstrap plane; the build script moves with **today's** input set (core, store, protocol sources, manifests, lock) so the identity keeps moving on daemon edits until step 4 | none | — |
| 2 | consumers switched (plan, cli, mcp, the tests) to the protocol crate; the docs-check edges added; every citation renamed | none | closures per crate |
| 3 | `acquisition-client` extracted (client, policies, locator, typed errors); `acquisition-core` → `acquisition-daemon` (`git mv`); the `acqd` binary; `acq daemon run` and the MCP argv interception deleted; the sibling locator; tests and drivers start `acqd`; the gate becomes build-then-test (AGENTS.md) and the uplift is checked empirically in this tree; references regenerated; skills and README updated | the spawn path | — |
| 4 | the identity: build-script inputs shrink to the shared contract; artifact identity and hash at startup; `hello`, `DaemonId`, the observer report, `acq version`, the journal header (supersedes 16 §6.4), `provenance.json` (hashes `acqd`); the artifact-mismatch process test; the standing-rule wording proposed for approval | the handshake | **the daemon-edit rebuild: crates compiled, executables relinked, seconds** — the number the split is judged by |
| 5 | the world (2.4): `world.rs` with first-run creation on the use path; the rails state into the provider's directory (legacy file moved once); the log and journal to the platform log directory, bounded; the world lock and the real-mode lock, taken before anything opens or sends; `hello` carries the world; the client verifies; the C31 clause | durable state and ownership | — |
| 6 | the rendezvous: the socket derived into the runtime directory; `ACQ_SOCKET` removed, `tools/acq-as.sh` retired; legacy default-socket detection in daemon start, client and `stop_any`; the migration process test; the preflight and the live-run skill prove no old daemon remains on the known driver endpoints; harnesses and the mock-session skill set one knob | the rendezvous | — |
| 7 | process tests green with both drivers in mock; then live: `tools/tracer-rung.sh` under the rails — the spawn path and the rendezvous changed; ledger row; `provenance.json` with `acqd`'s hash | — | the run |

Later, its own change, not this sequence: the job-store trait replacing
`test-hooks` (one artifact set), one test binary per crate with one
harness module (21 executables → 8; twelve harness copies → one).

Projected for step 4, to be replaced by the measurement: a daemon edit
compiles the daemon crate and links `acqd` and its one test executable
— 3 targets against today's 5 crates and 21 executables; a protocol or
store edit still rebuilds everything, as a contract change should.

## 5. Parking lot (landings named; deferral never re-argued)

- Packaging: bundle layout, signed binaries, an installer, the CLI symlink → ADR 0003, with C82 as its acceptance criterion: smoke tests that the GUI, the canonicalised CLI symlink and the MCP each resolve the same `acqd`, and that an updater replacement is an artifact mismatch the next use verb resolves. Trigger: a shipping decision.
- A second installation on one machine → a design event under C82: the contract-as-use-condition model of 2.2 is the recorded alternative. Trigger: a consumer that needs one.
- A keeper for frontends that never spawn (cron, a real-mode MCP): the foreground `acqd` contract — readiness, signals, exit codes, graceful checkpointing, supervisor interaction (launchd, systemd) → the daemon. Trigger: the first non-spawning consumer in daily use.
- The structured-error taxonomy beyond the additive `kind` → the wire, with the GUI slice as its validating consumer (P2). Trigger: the GUI slice, or the owner asking for it first.
- Correlation ids and multiplexed requests on one connection → C85 amended. Trigger: evidence a GUI needs concurrent requests on one connection.
- Windows: named pipes, `acqd.exe`, the runtime directory → the world's and locator's Windows arms; the protocol doesn't care. Trigger: a Windows build.
- Artifact dependencies (`-Z bindeps`) → `acqd` as a dev-dependency of the frontends' tests, replacing the sibling lookup there. Trigger: bindeps stabilises.
- A standalone queue TUI (`dash` as its own binary) → a new frontend package linking client. Trigger: the queue-monitoring TUI slice.
- `world.rs` as its own crate → trigger: a frontend that must not link SQLite.
- A GUI-hosted daemon → refused; the GUI spawns `acqd` like the CLI (2.5). Trigger: the GUI slice records its spawn policy.
- `spawned_by` on the daemon (which frontend started it) → `hello`/`daemon status`. Trigger: two spawners on one machine and a human needing to know which.
- The daemon's in-module harness moved to an integration test over the socket → TESTING-NOTES item 1, unchanged. Trigger: a fresh daemon build.
- Hash-only artifact identity → trigger: a false match from file identity, or a second install location.
- `Dashboard` reshaped for a GUI/TUI (one fat response today) → trigger: the TUI slice.

## 6. Questions for the owner

1. C1's text (§3). Accept, amend, refuse.
2. C82 with the installation premise stated in it — one installation per world, a second one a design event. The owner's own lean, 2026-09-09: "a canonical single-installation feels like a better fit for this project".
3. C83: the world, its two locks, the private runtime directory, `ACQ_SOCKET` removed, the rails state relocated, the legacy migration.
4. C84: shared-contract revision plus artifact.
5. C85 and the wire audit as the first commit.
6. Structured errors: additive `kind` now and the taxonomy with the GUI — or the taxonomy now, as its own session.
7. The journal header carrying the artifact hash, superseding 16 §6.4.
8. The standing-rule and `acq version` wording: proposed for approval with step 4, not before.
9. The C31 clause naming the enforcement scope (per OS user) and the external concurrency.
10. Commit −1, the test fix, before step 0 — code before the rulings, but the gate is broken without it.

## 7. Session facts

- **A failing test, unrelated to this session.** The full run found
  `game_side::tests::properties::c47_c69_any_text_reads_and_a_price_is_its_own_grammar`
  failing: proptest shrank it to `text = "~price 1 chaos 0"`, `Source::Note`.
  The reader accepts the note as one chaos with trailing text; the
  property's inverse grammar takes everything after the amount as the
  currency word and `unwrap`s the resolve (`game_side.rs:607`). Verified in round 3: the reader tolerates a suffix after the word by
  documented design (`game_side.rs:225`, "tolerated, not read"), so
  the property is the defective side — for a note it should take the
  first whitespace-delimited word, as it already does for a tab name.
  A one-line test fix, proposed as commit −1 (§4). Not applied here
  (no code before the rulings; pricing is C69's area). proptest wrote
  `crates/acquisition-plan/proptest-regressions/game_side.txt`, left
  untracked in the tree for the owner: proptest recommends committing
  it so the case replays.
- The measurement edits (a marker function appended to `daemon.rs`,
  `protocol.rs`, `main.rs`) were reverted; `git status` shows only the
  regression file.

## 8. Review round 1 (2026-09-09) and the author's responses

The reviewer's text is with the owner; this section carries each point,
what was verified (§1's second table), and the disposition. The
reviewer accepted the direction — `acqd` as a separate artifact,
frontends structurally unable to link the daemon, the boundary made
explicit — and would not accept round 1 as written. Nor would the
author, now.

### 8.1 The identity model forgot the store surface — accepted; round 1's error

A store change with only the frontend rebuilt passes both of round 1's
checks and puts a new reader on files an old daemon still writes, with
in-place migrations concurrent with that writer. Today's input set
prevents exactly this. Fixed as the reviewer recommends: the
shared-contract revision hashes the protocol and store sources,
manifests and the lock (2.2); daemon-only sources are excluded and
covered by the artifact, so the build-graph win is unchanged. Removing
the lock was independently wrong for the same partial-rebuild reason;
it stays.

### 8.2 Exact sibling identity against a shared daemon — accepted as a premise to rule, with the single-installation answer

True that "every frontend beside one `acqd`" was a `target/debug` fact
and not a product invariant, and that avoiding `PATH` makes each
installation surer of its own copy. The reviewer offered two designs
and favoured the second (contract as the hard use condition, artifact
as freshness and interactive policy) "unless Acquisition will
deliberately install one canonical helper". The owner's lean is the
canonical installation; the author's assessment on the merits agrees
and 2.3 gives the reasons (a desktop app ships one directory of
siblings; per-frontend installs are a library's shape and would need
precedence machinery). So the premise is ruled rather than assumed
(C82), artifact equality stays a use condition, C10 stands as ruled,
and the reviewer's second design is recorded as the multi-installation
alternative. Device and inode join the fast path, as recommended. The
sibling rule is proposed as a ruling because it now carries a premise
the GUI and packaging must honour, not as the playground's mechanism
alone; the owner may still demote it.

### 8.3 C83 did not enforce one daemon — accepted whole, and worse than stated

Every recommendation taken (2.4): the canonical provider-neutral root,
the lifetime world lock, the per-user real-mode lock independent of the
root, the world in `hello` and verified, the private runtime directory,
`ACQ_SOCKET` removed rather than kept, the rails state relocated first,
the legacy-socket path with a process test. Verified while checking:
the rails state sits in a directory macOS clears at reboot, so a
tripped tripwire is already lost across a restart — the relocation is
owed regardless of the split. The one script that set `ACQ_SOCKET` for
a reason is the closed rung-11 experiment and retires with the knob.

### 8.4 Fixtures pin serialization, not the contract — accepted

Verified: an unparseable request is answered "bad request", so the
client reports "unexpected handshake response" rather than a revision;
frames are unbounded; events are lossy on both sides. The four
boundaries are settled before the pin (2.7) and become C85; the pin is
fixtures plus the black-box contract tests the reviewer lists. One
partial pushback: the full structured-error taxonomy is a wire design
with a consumer it does not yet have, and C13's own model has the GUI
propose changes against a pinned boundary; the audit adds `kind`
additively now and the taxonomy waits for the GUI slice unless the
owner wants it first (question 6).

### 8.5 The protocol crate was too broad — accepted

The decisive argument is invalidation scope, not consumer count: a
client edit must not move the compatibility revision, and it would if
the client lived in the crate the revision hashes; the store dependency
for the world root made the crate heavier still. Split as recommended
into protocol (pure), client and daemon (2.1), with the world's path
functions in the store crate so the daemon never links the client.
Typed connect errors at the door, as recommended. One correction to the
review: the status types are already DTOs built for the dashboard,
distinct from the internal `Policy`/`Window`/`Limiter`; moving them is
the mapping the reviewer asks for, already done (§1).

### 8.6 `acq daemon start` — withdrawn

The reviewer is right on the facts: a daemon started with no work idles
out in 60 s and the MCP connects per call, so the verb serves neither
cron nor a real-mode MCP, and calling it operational contradicted C3's
"debugging only". Parked as the keeper model (2.5, §5).

### 8.7 The `-p` residual — same outcome, one disagreement on means

Agreed that a test must never silently consume whatever `acqd` exists:
a missing sibling fails before the test runs, a present one is named
with its hash in every failure, and the workspace gate is the stated
source of hermeticity. Not taken: a test-only path override — the
process tests drive the real `acq`, so the override would be read by
production code, a knob by another name (2.6). Artifact dependencies
are the proper fix and are parked on their stabilisation.

### 8.8 Steps 5 and 6, and the unconditional hook — accepted

Test-build economics, moved out of the sequence. The hook's landing is
a job-store trait the daemon's tests substitute; the unconditional
`#[doc(hidden)]` method is refused for the reason the reviewer gives.

### 8.9 The inconsistencies — all resolved

The journal hash supersedes 16 §6.4 and says so (2.2); `ACQ_SOCKET` is
gone (2.4); the TUI links client and therefore the store, stated (2.1);
the C3 amendment is withdrawn; `acq-as.sh` retires with the knob; "the
default root" reads "the canonical provider-neutral root" (2.4, C83).

### 8.10 The reviewer's answers to the eight questions, and the sequence

Taken as the shape of §6 and §4: contract-plus-artifact identity, the
installation premise settled, the world with locks and migration, the
audit before the pin, `daemon start` held, the wording deferred to the
commit that changes it, and the reviewer's ordering (audit and pin →
pure protocol → consumers and edges → `acqd` → identity → durable state
and locks → rendezvous with legacy detection → process tests and the
live run), with test consolidation kept separate.

## 9. Review round 2 (2026-09-09) and the author's responses

The reviewer approved the direction — C1, C84, the narrow protocol and
client split, the withdrawal of `daemon start`, the reordered sequence
— and asked for five amendments before treating the packet as settled.
All five are taken; two were verified rather than believed.

### 9.1 The gate must build `acqd` — accepted, verified

The reviewer doubted that `cargo test --workspace` guarantees an
unhashed sibling `target/debug/acqd`. Tested in a scratch workspace
(§1): a bin-only package's binary is built only as a hashed test
harness under `deps/` and never uplifted; a package with an integration
test gets its uplift. Round 2's hermeticity claim was false. The gate
becomes `cargo build --workspace` then the test run (2.6), checked
again empirically in this tree at step 3. The `-p` policy and the
refusal of a runtime knob stand; the disagreement is resolved by the
build step, as the reviewer says.

### 9.2 C85 needed an observable lag signal — accepted

True: the daemon skips lagged events silently, so "a client that lags
re-reads" was not implementable. `resync_required` with the count,
rather than closing the subscription (the reconnect logic is needed
either way and the signal keeps the count); the subscribe-then-snapshot
sequence pinned; a `Subscription` type distinct from `Client` so a
subscription connection carries no requests structurally (2.7, C85).

### 9.3 The world in three places — accepted

First-run canonicalisation defined (use creates and locks, observation
creates nothing, relative roots made absolute in the client, the lock
before anything opens or sends). Per user is not per machine: taken as
the reviewer proposes — C31's ruling stands, its enforcement scope is
named as one real daemon per OS user, and another OS user, the C++
Acquisition on the same machine and other machines behind the same
address are external concurrency named in the entry (a C31 clause,
§3); a machine-wide lock is refused as covering one of the three at a
permissions cost. Legacy custom sockets: the boundary stated — the
default endpoint discovered, the known driver endpoints proven empty by
the preflight and the skill, arbitrary history not discoverable; a
controlled migration (2.4).

### 9.4 Diagnostics must stay bounded — accepted

Round 2 would have made the log and the journal durable world state.
Only the rails state and `daemon.db` are durable; the log and journal
are bounded diagnostics in the platform's log directory, one
subdirectory per world and provider, rotated at start past a cap; live
runs keep overriding the journal into their evidence directory (2.4,
C83).

### 9.5 C82 as a packaging contract — accepted

Rephrased to what Acquisition requires: at most one installation
targets a world; the installation places one canonical `acqd` and the
implementation requires it as a sibling; a second installation
targeting the same world is the design event — which also removes
round 2's inconsistency between "a second installation on one machine"
and the app-plus-playground machine, whose two installations target
different worlds. Packaging stays parked as implementation with C82 as
its acceptance criterion and the four smoke tests named (2.3, §5).

### 9.6 The disputed points, and the wording

The status DTOs: agreed by the reviewer. The error taxonomy: agreed to
wait for the GUI, with the reviewer's condition taken — the initial
`kind` is a closed set pinned by the fixtures, never a free-form string
(2.7). C1: "the only GGG sender", and frontends link the crates "as
they need them" so a queue TUI links no planner.

### 9.7 Restore green first — accepted, and diagnosed

The pricing property failure in §7 makes "each commit under the gate"
unevaluable. Verified: the reader tolerates a suffix after the currency
word by documented design, so the property's inverse grammar is the
defective side; the fix is one line in the test plus the regression
file, proposed as commit −1 (§4, question 10).
