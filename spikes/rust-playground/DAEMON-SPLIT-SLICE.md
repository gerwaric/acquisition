# The daemon split — open record

The daemon split makes the daemon its own artifact (`acqd`) and draws
the frontend boundary as crates and a wire: `acquisition-protocol`,
`acquisition-client`, `acquisition-daemon`, the world in the store, two
identities in the handshake. Opened 2026-09-10 at `10f9bab5`, in the
mold of `REFRESH-SLICE.md` and `PRICING-SLICE.md`; while the slice is
open this is where its findings and observations land, and at closure
it is cut to the permanent short record.

The design is `brainstorming-notes/18-the-daemon-split.md`, ruled by the
owner on 2026-09-09 (its §10, verbatim): §2 the boundary, §3 the ruled
lines (C1 amended, C82–C85 new, C31 amended), §4 the commit sequence, §5
the parking lot. This file never restates it. The rulings become real
in the registry as each commit lands (§10's landing map): C85 with
commit 0, C1 and C82 with the `acqd`/client split, C84 with the
identity, C83 and C31 with the world. Until then the packet is the
contract the code is held to (P1).

## Step ledger

One row per commit of §4; a row is filled when the commit lands.

| Step | Commits | What landed |
| --- | --- | --- |
| −1 restore green | `93ed626c` | the pricing property's inverse grammar fixed; the regression seed committed; gate green |
| 0 wire audit and pin | `a6c070b3` | stable `hello`/`daemon_stop` plane (`protocol::Bootstrap`, read by name alone); frame bound `MAX_FRAME_BYTES` with `bad_request` (`frame.rs`); C85's semantics (`Subscription`, `resync_required { missed }`, subscribe-then-snapshot in `acq jobs --watch`); closed `ErrorKind` of nine, every daemon site classified at its origin (`daemon::Refusal`); one fixture per variant (`tests/wire.rs`); black-box contract tests (`tests/contract.rs`); the kind beside the message under `--json` and in the MCP error's `data`; C85 in the registry with today's Pinned paths; TESTING-NOTES item 3 struck; rehearsed in mock only |
| 1 protocol crate | `bf47b6d8` | `acquisition-protocol` extracted, serde-only: `protocol.rs`, `job.rs`, `realm.rs` moved whole; `status.rs` cut from `rails.rs` and `ratelimit.rs` (`RailsStatus`, `PolicyStatus`, `RuleStatus`, `WindowStatus`, `SendRecord`, `DegradedEndpoint`); `provider.rs` (`ggg_mode`, the names); `MAX_429_RETRIES` beside the retries it bounds; `VERSION_WITH_RUNTIME` and the build script with today's input set (core, store and protocol sources, manifests, lock — a daemon edit still moves it, a frontend source edit does not, both shown); `tests/wire.rs` and its 35 fixtures moved byte-identical; `frame.rs` stays in core; docs-check refuses the edges that hold now (the protocol manifest an allowlist per section, the store never links protocol), broken four ways and seen to fail; core re-exports the old paths for one commit; C85's and C12's pointers; rehearsed in mock only |
| 2 consumers | `45d24717` | plan, cli, mcp and the tests import from `acquisition_protocol` directly; every step-1 re-export deleted (`job`, `protocol`, `realm`, the three version constants, `daemon::MAX_429_RETRIES`, `provider::ggg_mode`, `rails::RailsStatus`, the five status names; `frame.rs`'s `MAX_FRAME_BYTES`, missed, went with round 6) — none had to survive; the planner links protocol and store only, its §2.1 row; the frontends add protocol and keep core for `client` and `daemon` until step 3; `contract.rs` switched in place, its harness untouched; the provider-name literals at the consumers are `provider::wanted()`; the planner-never-links-the-daemon edge in docs-check, broken three ways in code and seen to fail, and the table-completeness check moved into `forbid`/`allow` so an edge about an unknown crate fails closed (three tool breakers); revision c0c4ad7aab70 → d03b1daa2135 (the lock and one core import line, both inputs); fixture diff empty; closures per crate (`cargo tree -e normal --prefix none \| sort -u \| wc -l`, §1's method): store 31, protocol 14, core 160, **plan 41 (from 167)**, mcp 204, cli 220 — a 126-entry reduction, the daemon's closure the planner no longer carries; rehearsed in mock only |
| 3 `acqd` and client | `5a82e761` | `acquisition-client` extracted (`client.rs`, `locator.rs`, a `frame.rs` copy, `contract.rs` moved; typed `ConnectError`; links protocol and tokio, not yet the store); `acquisition-core` → `acquisition-daemon` by `git mv`, its modules untouched, the `acqd` binary (`main.rs`, no arguments); `acq daemon run` and the MCP's argv interception deleted, the spawn path execs the sibling `acqd` (`current_exe()` canonicalised, its parent, nowhere else); the frontends link client/protocol/store/plan and never the daemon; `socket_path`/`log_path` through the client crate's root; `frame.rs` one copy each side (the allowlist decided: a tokio feature on the protocol crate is refused); tests and drivers start `acqd` — the contract tests `target/<profile>/acqd` from their own location (the `ACQ_CONTRACT_DAEMON` harness retired), the CLI/MCP daemon-owning tests `locator::beside` on their binary, the drivers `target/debug/acqd`; the gate builds before it tests, checked here (`cargo test --workspace --all-targets --no-run` left `target/debug/acqd` absent with `acqd-<hash>` under `deps/`; `cargo build --workspace` wrote it); strict rustdoc widened to `--workspace` with the five links fixed; docs-check: every `acquisition-core` rule renamed, client ∌ daemon/plan, daemon ∌ client/plan/cli/mcp, nothing but the daemon names the daemon (`only_self`), plan ∌ client, store ∌ client/daemon, the intent grep over client/src — the breaker suite at 42 cases (17 new), 0 failed, store → daemon a Cargo cycle and daemon → frontend a dropped bin-only dependency (observations); revision 5f9cdb34ce21 → 8e205a53d16d (the input list renamed; `client.rs` no longer an input); fixture diff empty; C1 amended (794 B) and C82 (792 B) verbatim, C85's pointer (799 B); references regenerated; rehearsed in mock only: a CLI session, the MCP process tests, both drivers (`runs/mock/2026-09-10-tracer`, `runs/2026-09-10-persist-mock`) |
| 4 identity | — | shared-contract revision; artifact identity and hash; `hello`, `DaemonId`, `acq version`, journal header, `provenance.json` with both hashes; the artifact-mismatch test; C84 in the registry; standing-rule prose presented |
| 5 world | — | `world.rs`; rails state into the world, diagnostics bounded; the world lock and the real-mode lock; `hello` carries the world; C83 and the C31 amendment in the registry |
| 6 rendezvous | — | the socket derived into the runtime directory; `ACQ_SOCKET` removed, `tools/acq-as.sh` retired; legacy detection; the migration test |
| 7 live | — | both drivers in mock; the tracer under the rails; ledger row; `provenance.json` with `acqd`'s hash |

## Findings

One row per review round; the finding, then the property or test that
holds it now. The packet's own review rounds are its §8 and §9.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 1 (external, after commit 0) | `40890893` | (1) a bare name two live sessions share selected one of them (`Sessions::matching` was a `find`), through submit, quote, auth check and logout; (2) `jobs --watch` kept its lagged subscription after `resync_required`, so queued pre-snapshot events could follow the fresh snapshot, and printed events as if they were reads; (3) the wire pin's exhaustive matches forced a match arm, not a sample, and `ErrorKind::ALL` was a hand-kept list; (4) the daemon accepted versioned requests before `hello`; (5) the unit tests staging `queue_failed`, `upstream` and `internal` asserted messages, not kinds, the MCP `data.kind` claim had no process-level pin, and the `Refusal` table contradicted itself on the rails halt; (6) the daemon module doc and a startup comment still described multi-account as future work | (1) `matching` refuses an ambiguous selector as `ambiguous_account`: `a_selector_matching_several_sessions_is_ambiguous`, and the kinds contract test with `Alice#1234`/`Alice#5678`; (2) the watch re-subscribes and snapshots on `resync_required` and re-reads a job before printing it; the lag contract test now finishes the sequence and asserts the fresh subscription's first event is new; (3) `wire.rs` enumerates the variants from the type itself (serde's unknown-variant listing) and holds the sample set equal to it, `ErrorKind::ALL` included — broken and seen to fail; (4) `handle_conn` refuses a versioned request before `hello` (`bad_request`), pinned in the bootstrap contract test; (5) kind assertions on the seven unit-test sites, `plan_loop.rs` asserts `data.kind` and the code, the table names `auth_check` as the halt's one wire path; (6) the two comments and the C31 module text amended |
| 2 (external, after round 1) | `51700f4a` | (1) `jobs --watch` fell back to the event hint when its re-read failed, and never checked that its subscription and its request connection were to the same daemon instance; (2) the watch's recovery was not pinned by its stated test — the contract test walked the sequence by hand, so the original watch left it green; (3) the `Subscription` docs and the protocol's as-built paragraph taught snapshot-again without dropping the lagged subscription; (4) the round-1 row lacked its commit and the rails-halt observation contradicted the audit; (5) the `Refusal` table omitted the `Sessions::matching` ambiguity site | (1) a failed read restarts the subscribe-then-snapshot sequence, a transport failure on the subscription is a disconnect, a pid mismatch between the two connections starts over; (2) `acquisition-cli/tests/watch_recovery.rs` drives the binary under SIGSTOP/SIGCONT lag and requires a second snapshot with no event about an older job after it — broken and seen to fail on the kept subscription; that each printed line is a re-read is a reading of the code, said so in the test's doc; (3) both docs say drop, open a new one, snapshot; (4) the row and the observation corrected; (5) the row added |
| 3 (external, after round 2) | `bb6de4b0` | (1) `Signal::ResyncRequired`'s own doc still said only "snapshot again"; (2) `watch_recovery.rs` left its scratch store in the temp directory on every run | (1) the variant doc says drop, open a new one, snapshot; (2) a guard declared before the daemon removes the store on drop, in `watch_recovery.rs`, `error_kind.rs` and `contract.rs` |
| 4 (external, after commit 1) | `66933556` | (1) the dependency guard parsed the manifest's text and three valid Cargo forms bypassed it — a `[target.'cfg(unix)'.dependencies]` table, a `[dependencies.tokio]` table, a quoted key — while it printed "protocol = serde only"; (2) `realm.rs`'s as-built paragraph still placed the realm table in core, linkable because the planner depends on core; (3) the revision's doc said it never moves when a frontend changes, while the root manifest and the lock are inputs; (4) the quote sample's `not_covered` line said "up to 3" re-sends beside `MAX_429_RETRIES` = 2 (pre-existing, moved byte-identically) | (1) the edges are read from `cargo metadata --no-deps` through `jq` — every dependency Cargo resolves, with its kind, whatever section or spelling declared it — for the daemon and store edges too; eight breakers seen to fail (the four from `bf47b6d8`, the three bypass forms, a `[dependencies.tokio]` table in the store); (2) the paragraph names the protocol crate and why every consumer of the wire links it; (3) `lib.rs`, the C10 doc on `client.rs` and the step-1 row say a frontend *source* edit does not move it; (4) the sample is built from the constant, so it cannot contradict the promise beside it — the fixture diff is that one string |
| 5 (external, after round 4) | `6952f782` | (1) the dependency guard failed open: every `jq` call sat inside a process substitution, whose failure bash does not propagate to `comm` even under `set -euo pipefail`, so a `jq` exiting 127 — or absent — passed every edge with an empty set and printed "protocol = serde only" | (1) the metadata is read once into a flat `package kind name` table by a `jq` whose exit status is checked directly, `cargo metadata`'s failure is checked the same way, and the table must name a dependency of each crate the edges are about; the queries read it with `awk`. Breakers seen to fail: jq exiting 127, jq exiting 0 with no output, jq answering for one crate only, cargo metadata failing; the `[target]` tokio table still refused |
| 6 (external, after commit 2) | `b4487d53` | (1) one step-1 public re-export survived: `frame.rs` re-exported `MAX_FRAME_BYTES` for its own doc link, with no consumer, while the ledger said every re-export was deleted; (2) the packet's step-2 measure, closures per crate, was not recorded; (3) advisory: the planner rule forbade a direct `acquisition-core` edge only, so `plan → helper → daemon` would have passed | (1) the doc link names the protocol path, the line is gone, and no `pub use acquisition_protocol` remains in core; (2) the step-2 row carries the closures; (3) `forbid` made transitive over `cargo tree`'s closure — the host's active closure, superseded by round 7, which holds it now |
| 7 (external, after round 6) | `95d0f6c2` | (1) the round-6 closure omitted inactive targets and features: `cargo tree` reads the host's active closure under default features, so `plan → helper` with `helper → core` under `[target.'cfg(windows)']`, or optional behind a helper feature, passed; (2) a shadowed `cargo tree` answering with the root package alone satisfied "contains the package itself" while a staged path stood; (3) the step-2 row's derived number was wrong ("120-crate closure" for a 126-entry reduction) | (1)–(2) the declared-graph closure with partial-answer cross-checks, superseded by round 8, which holds both now; (3) the row corrected |
| 8 (external, after round 7) | `ad2fe341` | (1) an inactive optional dependency is declared but has no package entry, so the declared closure reached its name and not its edges: `plan → helper → external-bridge (optional) → core` passed (221 such declarations in this tree); (2) the awk closure was consumed through process substitutions, so a shadowed awk answering nothing passed with a staged path standing — the round-5 shape one step later; (3) vertices were names: `duplicate-helper` v1 under the store and v2 under the CLI linking protocol gave a false path through the store (295 packages, 269 names); (4) `.source == null` stood in for membership, refusing an excluded path crate whose dev-only path dependency Cargo never resolves | one graph, a design change: the `resolve` of `cargo metadata --all-features` — every package Cargo resolves with every member feature on, the edges the union over targets — keyed by package id, membership from `.workspace_members`; one checked jq finds it whole (every member a node, every edge on a node, every node with a package), a second emits `direct` and `closure` rows with paths, and the rules read the table with bash builtins alone, refusing a table without a package's own row or with a direct edge its closure lacks. Breakers seen to fail: the direct edge; the `cfg(windows)`, optional and plain helpers; the inactive-optional bridge (the four-step path printed); a member node missing; a package entry missing; the table jq answering nothing; the closure rows dropped; jq absent; cargo metadata misspelled. Seen to pass: the two-version helper; the excluded crate with a dev-only path dependency. A graph that lies consistently passes by design; a self-consistent partial *table* did not yet refuse — round 9 |
| 9 (external, after round 8) | `47d77df3` | (1) the wholeness check proved the graph, not the table: with the metadata intact and only the table jq answering with the four self rows, the staged path passed; (2) the all-target closure is a union over targets — `plan → helper` under `cfg(unix)` with `helper → core` under `cfg(windows)` is refused though no single target links both — while the comment and this record said "what Cargo can link on any target"; (3) the tree's package, name and declaration counts sat in the mechanism comment, the record's facts | (1) the first jq counts the members' direct edges and closure sizes by a set fixpoint of its own; bash counts the table's rows with builtins and refuses a mismatch before any rule — breakers: the four self rows alone, every row two or more steps deep dropped, the one deep row of the staged path dropped, the direct rows dropped, each naming both counts; what remained — a table of the right size with the wrong rows — is round 10's; (2) the union is the intended reading, stated in the comment — the boundary is what the manifests declare, not what one platform builds — and the union case is staged and seen refused with its path; (3) the counts removed from the comment |
| 10 (external, after round 9) | `5362ac04` | (1) matching counts did not prove correspondence: with the graph and its counts intact, the `plan → core` closure row removed and the `plan → helper` row duplicated in its place, both counts matched and the staged violation passed | (1) the first jq names every fact the table must hold, by package id, sorted — one line per direct edge and per reached package; the second emits the table, tab-separated with ids beside the names, in that order; bash projects each row to its identity and compares the two sequences element-wise with builtins, refusing at the first differing row and naming both sides. Breakers seen to fail: the reviewer's swap (row 938, the core id against the helper id); the row removed and a fabricated one appended; two rows swapped; the four self rows alone; the deep rows dropped; the direct rows dropped; the table empty; a member node missing; a package entry missing; jq absent; cargo metadata misspelled; the direct edge; the three helper forms and the inactive-optional bridge with their paths; the union case. Seen to pass: the two-version helper; the excluded crate with a dev-only path dependency. What remained — the names beside authenticated ids, unchecked — is round 11's |
| 11 (external, after round 10) | `3056df2b` | (1) the comparison authenticated ids and kinds and discarded the names, which the rules then read: with every row and id intact and the dependency name of the staged `plan → core` closure row changed to `masked-core`, the sequences matched, the forbid found no row, and the check passed | (1) the first reader carries the names into every expected line, from an id-to-name map of its own over packages already found whole; bash projects each table row to (ids, kind, member, name) before comparing, so a name that does not belong to its id refuses at that row; the rules read names so authenticated; the path column alone is not compared — printed on a hit, never read, said so in the comment. Breakers seen to fail: the reviewer's rename, the member-name column renamed, a direct row's dependency name renamed, and every earlier case. Seen to pass: only the path column altered (the hit still refuses); the two-version helper; the excluded crate with a dev-only path dependency. What remains is a reader that forges a whole table, its ids and its names to match: the consistent lie |
| 12 (external, after commit 3) | `7df3258f` | (1) `ConnectError::Absent` took its reason from `ACQ_NO_SPAWN` alone, so the MCP's real-mode absence — the one caller that connects without spawning — was told "it spawns on demand for job commands", and the MCP mapped every `ConnectError` to a bare internal error, the typed failure unconsumed; `ggg_refusal.rs` checked only that no socket appeared; (2) C82 says tests locate `acqd` the way frontends do while the contract harness said its lookup was "not the locator's rule"; and its claim that a present daemon is named by path in every failure held only for startup failures; (3) C39 stayed stale though its amendment was proposed for commit 3; (4) live docs named removed paths or the wrong dependency: C85's recorded pin and the frame reader's home in `protocol.rs`, the provider pointer in `provider.rs`, C43's mechanism in `daemon.rs` ("cannot link the store") | (1) `Absent { because: NotSpawned }` — `Policy` (the door does not spawn) judged before `NoSpawnEnv` (the knob), each with its own text; the MCP's `connect` wraps a policy absence in real mode with "this server never starts one there (C13) — start it from the CLI with a job command", and its `err` maps a `ConnectError` to a JSON-RPC error whose `data.connect` names the door; pinned by a client unit test for both reasons and by `ggg_refusal.rs`, which asserts the policy text and the CLI remedy in real mode and the knob's name under `ACQ_NO_SPAWN=1` in mock; (2) the harness doc states the tension as it stands and points at the observation, where the proposed C82 clause sits with byte counts for the owner; every daemon guard (`contract.rs`, the three CLI tests) prints its executable's path and pid while a test is panicking, so a failed assertion's output names the daemon; (3) not fixed — a ruled line; three measured variants in the observation for the owner; (4) the four docs corrected: C43's daemon links the store to write facts and keep its queue, cannot link the planner, reads no intent (C34) |

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the commit that touches it.

- The rails state today sits beside the socket in the per-user temp
  directory, which macOS clears at reboot: a tripped tripwire does not
  survive a restart (packet §1, verified). Closed by step 5.
- The contract tests find `acqd` as `target/<profile>/acqd` from the
  test executable's own location (`acqd_for_tests`), the file the
  gate's `cargo build --workspace` writes: a test executable has no
  sibling. Ruled into C82 on 2026-09-10 (the owner, verbatim: "accept
  the 782-byte trimmed amendment. Removing the playground parenthetical
  is appropriate; the important exception is now explicit: test
  executables name the acqd written by their build") — the line carries
  no amendment date, since one would put it at 802 bytes; this entry
  and `git log` date it.
- Since step 3 the runtime revision's inputs are the daemon, store and
  protocol crates: `client.rs` left the daemon crate, so a client or
  locator edit no longer moves the identity — the direction step 4
  completes, arrived at early by the move rather than by narrowing.
- Two clauses of C1 as ruled describe steps not yet built and landed
  verbatim with step 3 per the landing map (§10): the store "holds …
  the world (root, locks, socket name)" (step 5) and the protocol crate
  "the shared-contract revision" (step 4; today the runtime revision).
  The registry describes the ruled graph; the code reaches it by step 5.
- The client crate links protocol, tokio, serde and anyhow, not the
  store: nothing in it reads the store until the world (step 5), and
  §2.1's row lists the store for that. `socket_path`/`log_path` are two
  copies of one convention until then (`acquisition-client/src/lib.rs`,
  `daemon.rs`), each reading `ACQ_SOCKET` the same way.
- Two edges of §2.1 cannot be staged for the breaker suite as written:
  store → daemon is a Cargo cycle (the daemon links the store), refused
  by `cargo metadata` before any guard reads the graph — the suite
  expects Cargo's refusal, named as such; and daemon → frontend is
  dropped by Cargo as an invalid dependency on a bin-only package
  ("missing a lib target", a warning, not an error), so the guard prints
  ok over a manifest that declares it while nothing can link — recorded
  as a must-pass case that fails the day a frontend gains a library
  target (the Tauri GUI's shape).
- `acqd` takes no arguments: the environment is the one door to its
  knobs (the packet's rejected "flags on `acqd`"), and `--version` or
  `--help` is refused with exit 2 and a sentence saying what it is. The
  drivers read `acq --version`; step 4's provenance hashes the `acqd`
  file. A `--version` on the daemon is step 4's question, with the
  artifact identity.
- The daemon-owning process tests (`error_kind.rs`, `daemon_observe.rs`,
  `watch_recovery.rs`; the MCP harness's `spawn_daemon`) each carry
  their own `daemon_command`; a `-p acquisition-cli` run with no
  `target/debug/acqd` fails naming `cargo build --workspace`, as
  designed (§2.6). The one-harness-module-per-crate change is already
  parked.
- The protocol crate carries no `unwrap_used`/`expect_used` denial: C47
  names the store and plan crates. The sentence of design (P4), step 3:
  the crate's production code has no `unwrap` or `expect` today (its
  readers are `?`-shaped; the two in `realm.rs` are in its tests), and
  its build script — a separate target the lint would not reach — is
  the one place that panics on purpose (a missing input is the right
  failure). The lint would pin what already holds, at no cost; whether
  the contract crate joins C47's list is the owner's call, and the lint
  is not added unasked.
- A revision check that restores a scratch edit with `git checkout --
  <file>` discards every uncommitted edit in that file, and while the
  step-1 re-exports stood the build hid it: HEAD's `daemon.rs` still
  compiled against the new core. Seen once on 2026-09-10 (redone from
  the diff); the checks restore a scratch edit from a copy kept outside
  the tree and `cmp` it, never from git (step 2's breakers ran that
  way, `Cargo.lock` included — `cargo metadata` rewrites the lock when
  a breaker adds a dependency). Data for step 4's measurement, which
  edits the daemon on purpose.
- The process tests leak their scratch stores suite-wide: 567 `acq-*`
  directories sat in the temp directory on 2026-09-10 from every test
  file older than this slice (`acq-p`, `acq-reference`, `acq-story`,
  `acq-ann`, …), removed once by hand; 46 more by 2026-09-10 after
  step 3's suite runs, removed again. The three tests this slice wrote
  hold theirs in a guard; the rest is for the packet's later change to
  one harness module per crate, where one guard serves every test.
- Two provider comparisons stay literal after step 2: `dash.rs` and the
  CLI's `daemon status` compare the provider a daemon *reported*
  (`s.provider == "ggg"`) against a literal, not the wanted one, so
  `provider::wanted()` is the wrong tool there and `provider::GGG` is the
  constant nobody reached for. Data for step 4, which reshapes what
  `hello` and the observer report carry.
- C39 amended 2026-09-10 (the owner, verbatim: "change core + the
  store to protocol + the store, and use 'the daemon never links the
  planner' in the Why. Do not use 'never reads facts or intent': because
  the daemon links the store, the dependency graph cannot enforce that
  it never reads facts. It does enforce the absence of the planner
  dependency; intent blindness is separately covered by C34"). 439
  bytes.
- `acq jobs --watch` ends when the daemon goes away rather than waiting
  for it to return: an observer never spawns (C10), and a watch that
  waits for a daemon is a design choice for the GUI's subscriber, not
  the CLI's.
