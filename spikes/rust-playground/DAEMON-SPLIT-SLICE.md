# The daemon split — closed record

The daemon split made the daemon its own artifact (`acqd`) and drew the
frontend boundary as crates and a wire — `acquisition-protocol`,
`acquisition-client`, `acquisition-daemon`, the world in the store, two
identities in the handshake. Opened 2026-09-10 at `10f9bab5`, built as
eight commits (−1, 0–6) with twenty-eight review rounds, run live on
2026-09-11 (step 7) and closed the same day. This file is its permanent
short record in the mold of `REFRESH-SLICE.md` and `PRICING-SLICE.md`;
the open record's full text — every row's narrative, the observations as
they stood — is the file at `abef9a45`.

The design is the packet, note 18 at `decda84e` (ruled by the owner on
2026-09-09; its §10, verbatim), history. The rulings
live in the registry — `CONTEXT.md` (C1 amended) and
`decisions/daemon.md` (C82, C83, C84, C85 new; C10, C31 amended; C39
amended in `decisions/plans.md`) — the mechanisms in the module docs
they name, the properties in the tests named below, the live run in
`RUN-LEDGER.md` (2026-09-11). Nothing here is a second authority.

## Final state

- Tip at closure: the commit that cut this record; step 7 ran at
  `1bc7c6c4` (contract `1c0895a8d44f`, `acqd` `309b6e93…`). Seven
  crates, `tools/docs-check.sh` refusing every forbidden edge; gate
  green; the legacy-socket transition deleted (`599e8577`) once step 7
  proved it a no-op on the owner's machine.
- **The measure the split is judged by** (§4 of the packet; one `pub fn`
  appended to one file on a warm tree, `cargo build --workspace` then
  `cargo test --workspace --all-targets --no-run`, both `-v`; the edit
  and the lock restored from copies outside the tree and `cmp`'d):

  | Edit | `cargo build --workspace` | `cargo test --all-targets --no-run` | Contract revision |
  | --- | --- | --- | --- |
  | floor (§1, before the split: any edit) | 6.7 s; 5 crates; 21 executables | — | moved |
  | `daemon.rs` | 2.2 s; `acquisition_daemon`, `acqd`; 1 executable | 2.9 s; 2 test executables | unchanged |
  | `protocol.rs` | 3.0 s; 7 crates; 3 executables | 6.3 s; 28 test executables + 2 bins | moved |
  | `store/src/lib.rs` | 2.8 s; 8 crates; 3 executables | 6.7 s; 29 test executables + 2 bins | moved |

  A daemon edit compiles the daemon crate alone, links the three
  executables that are the daemon's, and moves no identity but the
  artifact; a contract edit rebuilds everything and moves the revision.
- Reading a change to the protocol, the client, the daemon's lifecycle
  or the world before reviewing one: the findings table below is the
  checklist — the same reader-that-answers-partially shape was found
  six times in one check, and the same fail-open shape four times at
  the doors.

## Step ledger

| Step | Commits | What landed |
| --- | --- | --- |
| −1 restore green | `93ed626c` | the pricing property's inverse grammar fixed; the regression seed committed |
| 0 wire audit and pin | `a6c070b3` | the stable `hello`/`daemon_stop` plane (`protocol::Bootstrap`); the frame bound (`MAX_FRAME_BYTES`, `bad_request`); C85's semantics (`Subscription`, `resync_required`, subscribe-then-snapshot); a closed `ErrorKind` of nine, every daemon site classified at its origin (`daemon::Refusal`); one fixture per variant (`tests/wire.rs`); black-box contract tests (`tests/contract.rs`); the kind beside the message under `--json` and in the MCP error's `data` |
| 1 protocol crate | `bf47b6d8` | `acquisition-protocol` extracted, serde-only (`protocol.rs`, `job.rs`, `realm.rs`, `status.rs`, `provider.rs`, the build script); the 35 fixtures moved byte-identical; docs-check refuses the protocol manifest's edges (an allowlist per section) and the store's link to protocol; core re-exports for one commit |
| 2 consumers | `45d24717` | plan, cli, mcp and the tests import `acquisition_protocol`; every re-export deleted; the planner links protocol and store only; the planner-never-links-the-daemon edge; closures per crate: plan 41 (from 167), store 31, protocol 14, core 160, mcp 204, cli 220 |
| 3 `acqd` and client | `5a82e761` | `acquisition-client` extracted (`client.rs`, `locator.rs`, `frame.rs` one copy each side, typed `ConnectError`); `acquisition-core` → `acquisition-daemon` by `git mv`, the `acqd` binary with no arguments; `acq daemon run` and the MCP's argv interception deleted; the spawn path execs the sibling (`current_exe()` canonicalised, C82); tests and drivers start `acqd`; the gate builds before it tests (`cargo test --no-run` leaves `target/debug/acqd` absent); strict rustdoc over the workspace; the breaker suite at 42 cases; C1 amended, C82 |
| 4 identity | `fd1a4aa1` | the build script's inputs shrink to the contract (protocol and store sources and manifests, root manifest, lock; the daemon out); `CONTRACT_REVISION`; the artifact — `FileIdentity` and SHA-256 in `acquisition-protocol/src/artifact.rs`, computed once at startup by the daemon's `artifact.rs`; `hello` carries `version`, `contract`, `artifact`, `pid`, `provider`; `DaemonId` judges contract, artifact (`ArtifactVerdict`: same file without a hash, else the sibling hashed) and provider; `acq version` (`--json`: the sibling as found), `acq daemon status` with the identity beside the vitals; the journal header's `contract` and `daemon`; `provenance.json` hashes `acq` and `acqd`, `provenance_matches_journal`; the artifact-mismatch process test (`daemon_observe.rs`); C84 |
| 5 world | `e2a108d8` | `acquisition-store/src/world.rs` (C83): `World` — the canonical root (`ACQ_STORE_DIR` made absolute, else the platform data directory; `create` the use path, `observe` creates nothing), its twelve-hex id, `daemon.lock` and the real-mode lock under `flock`, `rails.json` in the world, the log base (`ACQ_LOG_DIR`) with one subdirectory per world and provider, rotation past `DIAGNOSTIC_CAP_BYTES`; the client links the store and judges the world first (`ConnectError::OtherWorld`, never replaced); `hello` carries `world` both ways; the drivers write the run directory's journal and log; C83, the C31 amendment; pinned in `world.rs`, `rails.rs`, `client.rs`, `contract.rs`, `acquisition-cli/tests/world.rs` |
| 6 rendezvous | `6e4059e2` | the socket derived from the world (`World::socket_path`: `<runtime>/acq-<uid>/<id>.sock`, the daemon makes the directory, a client only verifies it, refused by name over `SOCKET_PATH_MAX`); `ACQ_SOCKET` gone from every reader, harness, skill and driver (`tools/acq-as.sh` retired); every door resolves the world first (`world_at_door`); `DaemonId` carries its `Endpoint`; `BoundSocket` (an exit unlinks only the socket it bound); the harnesses set `TMPDIR` and `XDG_RUNTIME_DIR` into the scratch; the transition from the fixed socket, for one release (deleted at `599e8577`) |
| 7 live | run `1bc7c6c4`, recorded `abef9a45` | both drivers in mock at HEAD; the tracer under the rails on the owner's default root, twice — the first attempt exposed the driver's readback defect (round 28), the second is the record: 1/4/9 = 14, four probes 0 hits, apply 9/9, loop closed; the daemon's own report in the bundle (`daemon-c1-status.json`); the world proven on the owner's default root — locks, socket, 0700 root; the transition a no-op. Owner verdict, verbatim: "The row is accepted." |
| close | `c5efd6fe`, `599e8577`, this commit | the breaker suite's store → client case made a Cargo cycle (it had failed since step 5); the legacy transition deleted with its tests, breakers, driver probe and the skill's second stop; the record cut; the narrative audit |

## Findings

One row per review round (all external, after the commit named); the
finding, then the property or test that holds it now. The rounds' full
text is at `abef9a45`.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 1 (commit 0) | `40890893` | a bare name two live sessions share selected one of them; `jobs --watch` kept its lagged subscription after `resync_required`; the wire pin's exhaustive matches forced an arm, not a sample; versioned requests accepted before `hello`; unit tests asserted messages, not kinds; stale multi-account prose | `Sessions::matching` refuses `ambiguous_account`; the watch re-subscribes and snapshots, re-reading before printing; `wire.rs` enumerates the variants from the type (serde's unknown-variant listing); `handle_conn` refuses before `hello`; kind assertions; prose amended |
| 2 (round 1) | `51700f4a` | the watch trusted the event hint when its re-read failed and never checked both connections reached one daemon; its recovery was not pinned through the binary; docs taught snapshot-again without dropping the subscription | a failed read restarts the sequence, a pid mismatch starts over; `acquisition-cli/tests/watch_recovery.rs` under SIGSTOP/SIGCONT lag; both docs say drop, open a new one, snapshot |
| 3 (round 2) | `bb6de4b0` | `Signal::ResyncRequired`'s doc; `watch_recovery.rs` leaked its scratch store | the doc; a drop guard declared before the daemon in every process test this slice wrote |
| 4 (commit 1) | `66933556` | the dependency guard parsed the manifest's text — a `[target.'cfg(unix)'.dependencies]` table, a `[dependencies.tokio]` table and a quoted key bypassed it; `realm.rs` placed the realm table in core; the revision's doc said it never moves on a frontend change while the lock is an input | the edges read from `cargo metadata` (superseded by round 8); docs corrected; the quote sample built from the constant it describes |
| 5 (round 4) | `6952f782` | the guard failed open: every `jq` sat in a process substitution whose failure bash does not propagate, so an absent `jq` passed every edge | the metadata read once by a `jq` whose exit is checked; the table must name a dependency of each crate the edges are about (superseded by round 8) |
| 6 (commit 2) | `b4487d53` | one step-1 re-export survived (`MAX_FRAME_BYTES`); the closure measure was not recorded; the planner rule forbade a direct edge only | the line gone; the step-2 row carries the closures; `forbid` transitive (superseded by round 7) |
| 7 (round 6) | `95d0f6c2` | `cargo tree` reads the host's active closure — a `cfg(windows)` or optional path passed; a shadowed `cargo tree` answering the root alone satisfied the check | the declared-graph closure with partial-answer cross-checks (superseded by round 8) |
| 8 (round 7) | `ad2fe341` | an inactive optional dependency is declared but has no package entry, so its edges were invisible; the awk closure was consumed through process substitutions; vertices were names (one name at two versions gave a false path); `.source == null` stood in for membership | one graph: the `resolve` of `cargo metadata --all-features`, keyed by package id, membership from `.workspace_members`; a checked jq proves it whole, a second emits `direct` and `closure` rows, bash builtins read them; `tools/docs-check-breakers.sh` stages every case |
| 9 (round 8) | `47d77df3` | the wholeness check proved the graph, not the table: a table jq answering four self rows passed a staged path; the union-over-targets reading was undocumented | the first jq names every fact the table must hold; bash refuses a mismatch before any rule; the union is the stated reading (the boundary is what the manifests declare) |
| 10 (round 9) | `5362ac04` | matching counts did not prove correspondence: a row removed and another duplicated in its place passed | the two readers' answers compared row by row, by package id, refusing at the first difference |
| 11 (round 10) | `3056df2b` | ids were authenticated and names discarded, which the rules then read: a renamed dependency name passed | names carried into every expected line from the first reader's own map; the path column alone is printed, never read; what remains is a reader that forges a consistent whole table — accepted |
| 12 (commit 3) | `7df3258f` | `ConnectError::Absent` took its reason from `ACQ_NO_SPAWN` alone, so the MCP's real-mode absence named the wrong remedy and the MCP mapped every `ConnectError` to a bare error; C82's test-executable clause was in tension with the harness; C39 stale; four docs named removed paths | `NotSpawned::{Policy, NoSpawnEnv}`, the MCP's `data.connect` naming the door, `ggg_refusal.rs`; every daemon guard names its process while panicking; C82 and C39 amended by the owner (`755da4fe`); docs corrected |
| 13 (commit 4) | `b75d3412` | `provenance_matches_journal` read only the headers, so a headerless journal passed; the compatible report folded "the daemon's own file" into "the sibling"; the persist verifier's "no non-2xx" let a 3xx pass | the function requires a header before every send and every header naming the recorded contract and hash (`tools/preflight-breakers.sh`, nine cases); `ArtifactVerdict::{SameFile, SameBytes}` apart, `artifact_relation` in every status; `tools/persist-verify.py` with the tracer's predicate and a self-test |
| 14 (round 13) | `bb367d89` | `report()` looked at the sibling twice more after the verdict, so a rebuild between could contradict `compatible`; the provenance check refused a pid the OS reused; the MCP's `daemon_status` unpinned | one captured verdict per handshake (`DaemonId::judged`, `Verdict::of`); a lifetime is the latest header; `acquisition-mcp/tests/daemon_status.rs` |
| 15 (round 14) | `bef49afd` | `DaemonId`'s fields and verdict were public and independently mutable; the MCP test had no deadline and no guards; the sibling was `stat`ed then reopened by path | private fields, read-only accessors, `judged` the one constructor; deadline, `Scratch` and `Daemon` guards; `Sibling::open` — identity and hash from one handle (`sha256_of_open`) |
| 16 (round 15) | `4a74bd6e` | the MCP harness read stdout without a bound; the single-handle guarantee had no boundary test; the standing rule named the contract's inputs loosely | every MCP answer waited under `recv_timeout`; `c84_the_sibling_is_judged_from_the_handle_that_was_identified`; `LIVE-TESTING.md` names `build.rs`'s inputs |
| 17 (round 16) | `88fba6d5` | `Mcp` and `plan_loop.rs` owned raw children with no `Drop`; the two-artifact fact was narrower than stated — `cargo test --all-targets` rewrites `acq` and `acq-mcp`, not `acqd` | both guards kill and wait on drop; AGENTS.md, the README tour and the skills say what `cargo test` rewrites and to build again before a run |
| 18 (round 17) | `b53a0fc6` | the live-run skill's hand-run check covered fewer files than preflight; `plan_loop.rs` removed its scratch on success only; the mock-session skill promised `provider: mock` on a fresh socket | the skill checks the whole tree; a `Scratch` guard; "daemon is not running" for a fresh socket |
| 19 (commit 5) | `a54d1b6b` | `daemon status --json` recomputed `log` from the shell's own `ACQ_LOG_DIR`; the log was rotated before the world lock, so a losing contender could rotate the incumbent's; the runtime fallback was `/tmp/acq`, shared and open to pre-creation; a non-UTF-8 root read lossily; the world observed twice per report; the rails migration read errors as absence; a daemon from before the world reads as another world (not fixed — the owner: no users of it) | `DaemonStatus.log` is the file the daemon opened; `take_locks` before the log; `app_runtime_dir` via `private_dir` (0700 in the creating call, owned, no symlink); `World::canonical` refuses non-UTF-8; `Verdict::own_world` read alone; `migrate_legacy_state` names every failure (deleted at close) |
| 20 (round 19) | `e513a179` | the migration's failure was merely logged and its write was not atomic; the private directory was created loose and tightened after; the locking prose overstated (the log directory and marker were made before the locks); the lock test read a gibibyte into a `String` | fail-closed, typed, atomic (rename) — `verify_state` reads the world's file strictly before the rails are built and still refuses the start; `DirBuilder` mode 0700, exactly 0700 required; locks before anything opens, a refused contender creates nothing; the test reads 4 KiB |
| 21 (round 20) | `8c1c9e37` | the refusal path opened the log with `create(true)`; `reset-tripwire` could not clear a directory in a state's place though the remedy promised it; the non-UTF-8 `ACQ_LOG_DIR` refusal unpinned | `append(true)` alone; reset removes a file or an empty directory and names a non-empty one for the hand; pinned through the binary |
| 22 (commit 6) | `91d6a017` | the drivers' endpoint probe read every failure of `python3` as "nothing listening"; the client's legacy probe had no deadline; a direct start migrated state before probing the predecessor; a non-UTF-8 runtime directory panicked `json!` in every report; the derivation test read the environment | the probe answers 10/11 and the caller refuses anything else (deleted at close with the probe); `Endpoint.socket` a `String` by construction, `World::socket_path_under` refuses by name; the derivation pure, tested with directories the test names |
| 23 (round 22) | `8b4e6fd4` | the derivation test still unwrapped the real socket first (a 132-byte `TMPDIR` failed it at the unwrap); the daemon's probe counted any error but success as absence; the timeout remedy offered unlinking a live socket | the named cases first, the real door held to the pure derivation; absence is exactly `NotFound` and `ConnectionRefused` at both probes (the daemon's deleted at close); the remedy names the process, never the file |
| 24 (round 23) | `19065011`, gate `28c7a363` | the daemon's legacy connect had no deadline; the backlog test's platform checks were constant assertions | bounded under the client's 2 s; runtime comparisons (both deleted at close) |
| 25 (round 24) | `4fd54668` | the backlog fixture filled with blocking connects (a Linux hang); **the pricing test's one-off failure was a first-open race in the annotations store** — the read-only gate read `user_version` and `has_tables` as two statements, and a winner's create committing between them left the loser refusing a well-formed file | non-blocking under a deadline (deleted at close); the gate reads both facts inside one deferred transaction (C35, `annotations.rs`'s `init`), pinned by an authorizer that stages the create between the reads — `c35_a_create_attempted_between_the_gates_two_reads_cannot_commit_or_cause_a_refusal` |
| 26 (round 25) | `2fd6de1d` | the backlog fixture let `std` choose the backlog (`somaxconn`, 4096 on Linux); the authorizer pin's text was stronger than its assertions; rusqlite's `hooks` was a normal dependency | `listen(2)` again with a backlog of 4 (deleted at close); the pass path asserts `committed == 0`, the old outcome observed by staging the old gate back; `hooks` a dev-dependency |
| 27 (round 26) | `1dea253b` | the backlog observation conflated the test's backlog with a daemon's; the authorizer test's name said "a create committing" | the observation separated; the test named for what it asserts |
| 28 (step 7) | `1bc7c6c4` | the tracer driver's phase-4 readback matched a tab's id at the start of the row while the id has been the last column since `28db97c6` — every id-list run since 2026-09-09 passed its wire phases and aborted; the driver kept no `daemon status --json` | the match is on the last column; `save_status` writes each lifetime's report as `daemon-<tag>-status.json` and refuses one that names no socket |
| close | `c5efd6fe` | the breaker suite's "store links client" case expected the guard's refusal while the edge has been a Cargo cycle since step 5 (the client links the store); failing since, unrun | the case expects Cargo's refusal, beside store → daemon |

## What the build and the run taught

Properties, each now a ruling, a mechanism doc or a pinned test:

- The daemon-edit rebuild is one crate and one binary (the measure
  above); the contract revision moves on a protocol or store edit —
  including a comment edit, on purpose: whole files are inputs.
- A dev-dependency's features unify into every `--all-targets` build:
  at close the store's `test-hooks` feature (the daemon's dev-dependency)
  and its dev-only `hooks` feature gave every binary two forms, so
  `cargo test` rewrote `acq` and `acq-mcp` and left `acqd` (which no
  test asks for). Both retired 2026-09-11 (the queue-failure tests break
  `daemon.db` through a second connection; `hooks` compiled in always):
  `acq` and `acqd` have one form; `acq-mcp` keeps two through proptest's
  `num-traits/std` under rmcp's chrono. Two rules stay (AGENTS.md): the
  plain build before anything that starts a daemon; never the drivers
  while the gate runs (round 16's gate saw exactly that mismatch).
- A test executable has no sibling: the contract tests and the MCP
  harness name `target/<profile>/acqd` from their own location and
  place a symlink beside the test executable, so the locator's one rule
  stays the one rule (C82's clause).
- `acqd` takes no arguments and no `--version` (`main.rs` says why):
  the identity is two values nothing needs it to print.
- A world is ownership, a socket is discovery: the lock made C6 and C31
  structure; the socket's derivation made two spellings of one root one
  daemon; the log directory names a world by its hashed id beside a
  `world` file naming the root.
- A daemon on its way out must unlink only the socket it bound
  (`BoundSocket`): a stop followed within a second by a job command
  binds the same derived path, and the predecessor's exit unlinked the
  successor's socket — a race the fixed socket had for every world.
- The real-mode lock is per runtime directory: a relocated `TMPDIR`
  (macOS) or `XDG_RUNTIME_DIR` (Linux) is a second lock — true since
  step 5; the `ACQ_*` override was the one refused, and the tripwire
  bounds what a local lock cannot (`world.rs`).
- The world's socket path is short under the platform's defaults (75
  bytes on this machine, 81 with the largest uid) and refused by name
  over 103 — both variables are the environment's.
- The step-7 live run proved the world on the owner's default root with
  no migrated state: the transition was a no-op there, and the
  observation's one leftover (`$TMPDIR/acq/ggg.lock`, round 19's first
  layout, a dead pid) was removed by hand at close.
- `acq jobs --watch` ends when the daemon goes away: an observer never
  spawns (C10); a watch that waits is the GUI subscriber's design
  choice (parked, `decisions/frontends.md`).
- The persist driver takes the tracer's rule — one directory per attempt
  — after a stale journal in a reused directory refused a rehearsal
  with twelve foreign headers: the check doing its job on the wrong
  directory.
- A revision check that restores a scratch edit with `git checkout --`
  discards every uncommitted edit in that file; the breaker suites
  restore from copies kept outside the tree and `cmp` them.

## Owner verdicts, verbatim

- C82's test-executable clause (2026-09-10): "accept the 782-byte
  trimmed amendment. Removing the playground parenthetical is
  appropriate; the important exception is now explicit: test executables
  name the acqd written by their build" — the line carries no amendment
  date (one would put it at 802 bytes); `git log` dates it (`755da4fe`).
- C39 (2026-09-10): "change core + the store to protocol + the store,
  and use 'the daemon never links the planner' in the Why. Do not use
  'never reads facts or intent': because the daemon links the store, the
  dependency graph cannot enforce that it never reads facts. It does
  enforce the absence of the planner dependency; intent blindness is
  separately covered by C34".
- The standing rule's "Build before you run" bullet, reworded for the
  two identities (2026-09-11): "Ok, this makes sense. approved."
- The four help texts of step 4, reread (2026-09-11): "approved."
- Round 19's pre-world daemon (2026-09-11): "I think we can neglect 1,
  because there are no users of the pre-commit 5 daemon".
- `ACQ_LOG_DIR`, a knob step 5 added unasked (2026-09-11): "Keep
  `ACQ_LOG_DIR`, and keep the drivers pointing it into the run
  directory. It provides useful test isolation and makes the journal and
  log genuine components of one evidence bundle. Copying a rotating
  platform log afterward is less reliable. However, status must report
  the daemon's actual opened path before it can support either
  arrangement correctly." — the last clause is round 19's finding 2.
- A sentence naming the journal's home, proposed for `LIVE-TESTING.md`
  (2026-09-11): "Drop the proposed LIVE-TESTING sentence. The path and
  override mechanics already belong in the knob documentation/read site
  and the live-run procedure. Adding them to the safety rails would
  create another description that can drift. If LIVE-TESTING needs
  anything, a stable pointer such as 'the daemon reports the journal
  location; the live-run procedure owns bundle placement' would be
  preferable." The knob row, `world.rs` and the live-run skill hold it.
- The step-7 row (2026-09-11): "The row is accepted."

## Where the open observations went

Every observation the open record carried is now one of: deleted with
the transition it described (the wedged pre-transition daemon on a
saturated backlog; the rails migration's merge; the legacy probe's
bound), a line above, a doc comment at its mechanism, or a parked entry
with a trigger:

- `decisions/daemon.md`, "Parked": the unbounded handshake on the
  world's socket; the protocol crate and C47's lint (the owner's call,
  not ruled at close); one harness module per crate with the
  `test-hooks` retirement (the daemon-owning tests' own `daemon_command`
  each, the three symlink copies, the scratch-store leak of the older
  test files); the packet's §5 parking lot, moved whole from the
  disposable note.
- `decisions/frontends.md`, "Parked": a watch that waits for the daemon
  to return; the standalone queue TUI; `Dashboard` reshaped; the error
  taxonomy beyond `kind`.
- `tools/docs-check-breakers.sh`: the two §2.1 edges that cannot be
  staged — store → daemon and store → client are Cargo cycles, and
  daemon → frontend is a dependency Cargo drops on a bin-only package
  (a must-pass case that fails the day a frontend gains a library
  target, the Tauri GUI's shape).

## Process used

Build / external review / fix, one commit at a time, the packet ruled
before code (P1) and every ruling landing in the registry with the
commit that made it real. Twenty-eight review rounds found real defects
every time and were worth their cost — eleven of them on one shell
check, each finding the same shape (a reader whose partial answer
satisfied every rule) one step deeper, until what remained was a
consistent lie, accepted and said so. The doors had the dual shape:
fail-open on a condition that could not be told (an absent tool, a
timeout, an error read as absence), fixed four times. Recording each
round as a row in this record rather than in `CONTEXT.md` kept the
always-loaded documents at budget throughout; the record itself grew to
87 KB and was cut here, its narrative left to the commits.
