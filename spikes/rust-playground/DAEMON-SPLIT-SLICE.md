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
| 2 consumers | — | plan, cli, mcp and the tests on the protocol crate; docs-check edges |
| 3 `acqd` and client | — | `acquisition-client`; core → `acquisition-daemon`; the `acqd` binary; `daemon run` gone; the sibling locator; the gate builds before it tests; C1 and C82 in the registry |
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

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the commit that touches it.

- The rails state today sits beside the socket in the per-user temp
  directory, which macOS clears at reboot: a tripped tripwire does not
  survive a restart (packet §1, verified). Closed by step 5.
- C85's *Pinned* path for `contract.rs` names the client crate, which
  exists from step 3; until then the contract tests live under
  `acquisition-core/tests/` and the registry entry names them there.
  The `wire.rs` pointer followed the file with step 1 (`bf47b6d8`);
  step 3 moves `contract.rs` and edits the other.
- The contract tests' daemon is the test executable re-run under
  `ACQ_CONTRACT_DAEMON=1` (`contract.rs`, `daemon_entry`), because no
  crate but the CLI has a binary before step 3 and the tests may not
  link a frontend. Step 3 replaces it with the sibling `acqd`.
- Two kinds beyond the packet's sketch, reviewed at the fixture diff:
  `wrong_state` (a result before terminal, a cancel after, a priority
  change off `waiting`) and `upstream` (a token refresh that failed on
  transport, a 5xx, exhausted 429s or a rails halt, the session still
  standing; a rejected grant is `not_logged_in`). The sketch's "rails
  halted" is not a kind of its own: a halted send waits and a quote
  names the halt, and the one request a halt refuses — `auth_check`'s
  forced refresh — is `upstream`. A veto is one variant and a fixture
  regeneration.
- The frame bound guards what each side reads; the daemon does not
  measure its own answers, so a `result` over 64 MiB would surface at
  the client as an oversize answer, not at the daemon. No such body
  exists today (a tab is a few megabytes).
- `frame.rs` — the bounded reader both sides use — needs tokio, and C1
  as ruled makes the protocol crate serde-only. Step 1 left it in core
  (one core crate exists until step 3; only the constant moved). Step 3
  places it: a feature on the protocol crate, or one copy each in the
  client and daemon crates.
- Step 1 re-exports the moved items at their old core paths for one
  commit, each marked for deletion: `job`, `protocol`, `realm` and the
  three version constants from `lib.rs`; `daemon::MAX_429_RETRIES`;
  `provider::ggg_mode`; `rails::RailsStatus`; the five status names in
  `ratelimit.rs`. Step 2 switches the consumers and deletes them; a
  re-export that survives step 2 is a consumer nobody switched.
- The protocol crate carries no `unwrap_used`/`expect_used` denial: C47
  names the store and plan crates, and the wire's lenient readers are
  `?`-shaped today. Whether the contract crate joins them is a P4
  promotion — a lint after a sentence of design — for step 2 or 3.
- A revision check that restores a scratch edit with `git checkout --
  <file>` discards every uncommitted edit in that file, and while the
  step-1 re-exports stand the build hides it: HEAD's `daemon.rs` still
  compiles against the new core. Seen once on 2026-09-10 (redone from
  the diff); the checks strip the scratch line instead. Data for step
  4's measurement, which edits the daemon on purpose.
- The process tests leak their scratch stores suite-wide: 567 `acq-*`
  directories sat in the temp directory on 2026-09-10 from every test
  file older than this slice (`acq-p`, `acq-reference`, `acq-story`,
  `acq-ann`, …), removed once by hand. The three tests this slice wrote
  hold theirs in a guard; the rest is for the packet's later change to
  one harness module per crate, where one guard serves every test.
- `acq jobs --watch` ends when the daemon goes away rather than waiting
  for it to return: an observer never spawns (C10), and a watch that
  waits for a daemon is a design choice for the GUI's subscriber, not
  the CLI's.
