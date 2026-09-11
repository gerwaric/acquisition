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
| 4 identity | `fd1a4aa1` | the build script's inputs shrink to the contract (protocol and store sources and manifests, root manifest, lock; the daemon crate out; domain `acq-contract-revision/1`), `ACQ_CONTRACT_REVISION` / `CONTRACT_REVISION` / `VERSION_WITH_CONTRACT` (`acq --version`: `0.0.1 (contract <rev>)`); the artifact — `FileIdentity` (canonical path, len, `mtime_ns`, dev, ino) and SHA-256 — defined in `acquisition-protocol/src/artifact.rs` and computed once at startup by the daemon's `artifact.rs` (a daemon that cannot read its own file does not start); `hello` carries `version`, `contract`, `artifact`, `pid`, `provider` (the world is step 5's) and the client's hello `version` + `contract`; `DaemonId` judges three dimensions (`Verdict`: contract, `ArtifactVerdict` — same inode without a hash, else the sibling hashed: a copy is the same artifact, other bytes another, no sibling matches nothing and says so, a daemon reporting none matches nothing —, provider), the report carries `contract_matches`/`artifact_matches`/`provider_matches`, the daemon found (`contract`, `artifact`) and `wanted` (`contract`, `provider`, `acqd` as found or `null` with `acqd_absent`); `acq version` reports the candidate (`--json`: `{version, contract, provider, acqd: {path, len, mtime_ns} \| null}`), `acq daemon status` the daemon running (its `contract` and `artifact` beside the vitals); the journal header's `runtime` became `contract` and `daemon` (the hash; `null` from the in-process harness), the startup identity line in the log names both (account and keyring diagnostics may precede it); `provenance.json` keeps `exe_sha256` and adds `acqd_sha256`, and `provenance_matches_journal` (`preflight.sh`, both drivers) holds every lifetime's header to it; the two literal provider comparisons (`dash.rs`, `auth status`) read `provider::GGG`; `acqd --version`: no flag (its `main.rs` says why); C84 in the registry (762 B); fixture diff: `bootstrap/hello.json`, `bootstrap/hello_reply.json`, nothing else; the revision moved once, `c1696aafb14a` → `3cd2835ff374`, and the measure (below) shows it still on a daemon edit; the artifact-mismatch process test (`daemon_observe.rs`, staged with one build's binaries: copies of `acq` beside a copy of `acqd`, beside another file, beside nothing); references regenerated; rehearsed in mock only: a CLI session, `acq-mcp` over stdio, both drivers (`runs/mock/2026-09-10-tracer-234740`, `runs/mock/2026-09-10-persist`) |
| 5 world | — | `world.rs`; rails state into the world, diagnostics bounded; the world lock and the real-mode lock; `hello` carries the world; C83 and the C31 amendment in the registry |
| 6 rendezvous | — | the socket derived into the runtime directory; `ACQ_SOCKET` removed, `tools/acq-as.sh` retired; legacy detection; the migration test |
| 7 live | — | both drivers in mock; the tracer under the rails; ledger row; `provenance.json` with `acqd`'s hash |

**Step 4's measure** — the daemon-edit rebuild, the number the split is
judged by (§4). One semantic edit (a `pub fn` appended) to one file on a
warm tree, then the gate's `cargo build --workspace` and `cargo test
--workspace --all-targets --no-run`, both `-v`; the edit and `Cargo.lock`
restored from copies kept outside the tree and `cmp`'d, never `git
checkout --` (the observation below), and both builds re-warmed before
the next edit. Against §1's floor (a daemon edit: 5 crates, 21
executables, 6.7 s):

| Edit | `cargo build --workspace` | `cargo test --all-targets --no-run` | Contract revision after |
| --- | --- | --- | --- |
| `daemon.rs` | 2.2 s; `acquisition_daemon`, `acqd`; 1 executable | 2.9 s; the daemon's lib and lib test, `acqd`'s harness; 2 test executables | unchanged (`3cd2835ff374`) |
| `protocol.rs` | 3.0 s; 7 crates; 3 executables | 6.3 s; 7 crates; 28 test executables + 2 bins | moved (`28e76726c9f7`) |
| `store/src/lib.rs` | 2.8 s; 8 crates; 3 executables | 6.7 s; 8 crates; 29 test executables + 2 bins | moved (`6c586b3267b0`) |

A daemon edit now compiles the daemon crate alone and links the three
executables that are the daemon's (its binary, its lib test, its
binary's harness) — the packet's projection was three targets — and
moves no identity but the artifact; a contract edit rebuilds everything
and moves the revision, as it should.

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
| 13 (external, after commit 4) | `b75d3412` | (1) `provenance_matches_journal` read only the `open` headers, so a journal with none — sends and no C84 identity, the failure the check exists for — produced no output and printed success, and both journal readers then synthesized an anonymous lifetime for headerless sends; (2) the compatible report called the daemon's own path "the sibling this client would start" while the supported copy case has the daemon at the original and the sibling at the copy, `ArtifactVerdict::Same` folded the two, the compatible JSON carried no `wanted`, and the copy test pinned only compatibility and pid; (3) pre-existing: the persistence verifier's "no non-2xx" rejected only `>= 400`, so a 3xx passed, while the tracer's predicate is `200 <= status < 300` with a 302 breaker | (1) the function requires at least one header, exactly one `open` header before every send for its pid, and every header naming the recorded contract and hash, and refuses a `provenance.json` without them; both readers record a headerless send as a lifetime that fails, never an anonymous one; breakers: `tools/preflight-breakers.sh` (nine cases: a header removed, an unmatched pid, no header, an empty journal, another daemon, another contract, a second header, the hashes missing, the whole journal passing) and both verifiers' `--self-test` (a header removed, an unmatched pid, no header), each naming its refusal; (2) `ArtifactVerdict::SameFile` and `SameBytes { sibling }` kept apart (`matches()` true for both; `relation()` one word), the report carries `artifact_relation`, the compatible status (CLI and MCP) carries `contract`, `artifact`, `artifact_relation` and `wanted` beside the vitals, the prose says "this client's sibling" or "the same artifact as this client's sibling <path>, another copy"; pinned by `judge_against` at the unit level and, through the binaries, the original (`same_file`, `wanted.acqd` the daemon's file) and the copy (`same_bytes`, `artifact.path` the original, `wanted.acqd.path` the copy, the prose naming the copy); (3) the verifier extracted to `tools/persist-verify.py` with the tracer's predicate and a self-test (a 302, a 429, a GET before its probe, the three header cases) |
| 14 (external, after round 13) | `bb367d89` | (1) the compatible status could contradict itself: `observe` judged the daemon on one look at the sibling, and `report()` — called later by the CLI and the MCP — looked twice more (the relation, then `wanted.acqd`), so a rebuild in between could say `compatible: true` beside `artifact_relation: different`, or relate the daemon to one identity and name another under `wanted`; (2) the shell provenance check counted headers by pid for the whole journal and refused a second `open` for a pid, while both readers start a lifetime at every header — a pid the OS reuses for the successor would refuse legitimate live evidence (fails closed, but the round-13 claim was too strong); (3) the MCP boundary the remediation added was not pinned: no process test called `daemon_status` or asserted `contract`, `artifact`, `artifact_relation`, `wanted`; and, documentation: the CLI help listed two relations of six, the MCP description named neither new field, the findings table had 13 above 12, the record listed the client's links without sha2, and "first log line" was not literally first (account and keyring diagnostics can precede it) | (1) the verdict — every dimension and the sibling as found — is captured once, in the handshake that reads the identity off the wire (`DaemonId::judged`, `Verdict::of`: one `sibling()` call), and kept on the `DaemonId`; `is_ours`, the report's flags, `artifact_relation` and `wanted.acqd` all read that one snapshot, and `ArtifactVerdict::judge` (the door that looked again) is gone; (2) a lifetime is the latest header: a send must carry the pid of the `open` line before it, a repeated pid is a second lifetime — the readers' model, now the shell's — with the breakers renamed ("under pid 1's header, not its own"), the first header removed added, and a pid-reuse pass case in all three suites; (3) `acquisition-mcp/tests/daemon_status.rs`: absent as a state with no daemon appearing, running and compatible with the vitals, the twelve-hex contract, the artifact path and hash, `same_file` and `wanted.acqd` both the sibling of `acq-mcp`, and a server wanting ggg reporting the mock daemon incompatible on the provider alone, the daemon left running; the help strings list every relation and the MCP description both fields (references regenerated), the findings table is in order, the client's links name sha2, and "first log line" reads "startup identity line" where it was mine (the standing rule's bullet, which says it too, is the owner's text: the proposal in the observations already says "startup identity line") |
| 15 (external, after round 14) | `bef49afd` | (1) the captured verdict was not held by the API: `DaemonId`'s identity fields and its `verdict` were public and independently mutable, so a caller could change the provider, contract or artifact after the handshake — or the verdict — and `is_ours()` and `report()` would combine a stale judgement with a changed identity; (2) the MCP `daemon_status` test's readiness loop had no deadline, so a daemon that never bound would hang the gate, and the test had no drop guard, so a failed assertion left the daemon and the scratch directory behind; (3) the sibling was still two filesystem operations — `stat` for the identity, then a fresh open by path for the hash — so a replacement between them could name the old file under `wanted` while the relation was decided on the new bytes | (1) the fields are private with read-only accessors (`pid()`, `version()`, `contract()`, `artifact()`, `provider()`, `verdict()`); the only constructor is `DaemonId::judged`, which makes the verdict of exactly the fields it stores; (2) a ten-second deadline naming the daemon, a `Scratch` guard that removes the directory and a `Daemon` guard that kills it and names it while panicking, declared in that order so the daemon is dropped first; (3) the sibling is opened once (`Sibling::open`): the identity comes from the handle's metadata (`FileIdentity::of_open`, on the contract) and the bytes, when they must decide, from the same handle (`sha256_of_open`) — the daemon identifies and hashes its own executable the same way — so what a report names and what it judged are one file; the protocol edit moved the contract revision `3cd2835ff374` → `b22a80cb3aa1` |
| 16 (external, after round 15) | `4a74bd6e` | (1) the MCP test's deadline was checked only between answers while the harness read stdout without a bound, so a server or socket-bound daemon that stopped answering would hang the gate with the guards never running; (2) the single-handle guarantee had no boundary test: the unit test judged unchanged files, so a judge that reopened the path would still pass; (3) documentation: the standing rule called the contract a digest over the sources alone (C84 and `build.rs` include the manifests, the root manifest and the lock) and said preflight refuses "a dirty tree" (it checks the rung's own files); the round-15 row named the guards in the wrong order; the record said 9350 B where the file is 9357; two as-built comments still said the sibling is `stat`ed | (1) the harness reads stdout on a thread into a channel and every answer is waited for with `recv_timeout` (30 s), failing by the method's name — for every MCP test, not only the new one; (2) `c84_the_sibling_is_judged_from_the_handle_that_was_identified`: a sibling opened, then replaced at its path by rename, judges `same_bytes` from the handle with the opened identity named, and the daemon's own file opened then replaced judges `same_file`; (3) the two clauses made exact in `LIVE-TESTING.md` (owner-approved wording, amended for accuracy the same day — flagged to the owner), the row's order and the byte count corrected, the comments say "opened once"; the protocol crate's comment edit moved the contract revision to `5fddc80b3b9a` (whole files are inputs, on purpose). The first gate run of this round failed one contract test with the daemon reported incompatible: the drivers were run alongside the gate, and their `cargo build --workspace` replaced `target/debug/acqd` under the test's live daemon — C84 seeing a rebuild, not a defect; the cause is the observation below, and the gate was rerun alone, green |
| 17 (external, after round 16) | `88fba6d5` | (1) the bounded answers could still leave processes behind: `Mcp` owned a raw child with no `Drop`, and `plan_loop.rs` owned its daemon as a raw child with cleanup reached only on success, so a timed-out answer left both running; (2) the two-artifact observation was narrower than the fact: from the plain build, `cargo test --workspace --all-targets` leaves `acqd` but rewrites `acq` and `acq-mcp` with their all-targets forms, so AGENTS.md, the README tour and the skills, which said tests rebuild neither binary, were wrong about the frontends; (3) the standing rule's preflight parenthetical was still inexact (the root manifest unnamed, "control documents" wider than the script's list) | (1) the harness's `Daemon` (returned by `spawn_daemon`) and `Mcp` both kill and wait on drop and name their process while a test is panicking; `plan_loop.rs` and `daemon_status.rs` hold them as guards; (2) reproduced by hash for all three binaries in both forms and after `cargo test` and `cargo clippy` (clippy rewrites none; the plain build restores all three); the observation records the six hashes and that the gate's process tests drive all-targets frontends against the plain daemon; AGENTS.md's gate note, the README tour line and both skills now say what `cargo test` rewrites and to build again before a run; (3) the parenthetical enumerates the script's list — `tools`, `crates`, `Cargo.toml`, `Cargo.lock`, `CONTEXT.md`, `decisions`, `LIVE-TESTING.md`, `RUN-LEDGER.md` — and points at `preflight.sh` as its home |

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
- One clause of C1 as ruled describes a step not yet built and landed
  verbatim with step 3 per the landing map (§10): the store "holds …
  the world (root, locks, socket name)" (step 5). The "shared-contract
  revision" clause became true with step 4. The registry describes the
  ruled graph; the code reaches it by step 5.
- The client crate links protocol, tokio, serde, anyhow and (since step
  4, to hash the sibling) sha2, not the
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
- `acqd` takes no arguments and no `--version` (step 4's answer, in
  `main.rs`): the environment is the one door to its knobs, and the
  daemon's identity is two values nothing needs it to print — the
  contract revision is the constant `acq --version` prints, and the
  artifact is a property of the file, which the drivers hash with
  `shasum` into `provenance.json` and a client learns from `hello`; a
  self-report would be a second door to the first and could not vouch
  for the second. The drivers needed no flag: `preflight.sh` reads
  `acq --version` and `acq version --json` and hashes both files.
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
- The standing rule's "Build before you run" bullet: reworded for the
  two identities in `LIVE-TESTING.md` (1233 B; the file at 9357 of
  15000). Presented as the record's proposal after round 15, explained
  in plain terms; the owner, 2026-09-11, verbatim: "Ok, this makes
  sense. approved."
- The help strings step 4 wrote — `acq version`, `acq daemon status`,
  `acq daemon stop`, the MCP's `daemon_status` — are in the regenerated
  `CLI-REFERENCE.md` and `MCP-REFERENCE.md` for the owner's approval
  (§10 item 8: the exact prose deferred to the real generated output).
- Three test executables drive the client in-process against a daemon
  they start (`contract.rs`, `watch_recovery.rs`, the MCP harness), and
  since step 4 the client judges that daemon's artifact against the
  `acqd` beside *its own* executable — which, in `target/<profile>/deps/`,
  does not exist. Each harness therefore places a symlink named `acqd`
  beside the test executable pointing one level up at
  `target/<profile>/acqd`, the file the gate's build writes and the
  harness starts, and asserts both resolve to the same file: the test
  executable names the daemon its build wrote on both paths (C82's
  clause), and the locator's one rule stays the one rule — no
  production code learned a second location. Three copies of the same
  twelve lines, for the parked one-harness-module-per-crate change.
  The alternatives were weighed and refused: a library function that
  names the comparison target (a configured path by another name, C82),
  and treating an absent sibling as "not judged" (fail-open on a use
  condition).
- The compatible `daemon status` reports the daemon's file and the
  sibling apart since round 13 (`artifact_relation`, `wanted.acqd`): a
  copied installation (the supported case) sees `same_bytes`, the
  original's path under `artifact` and its own under `wanted`.
- `DaemonStatus.version` and `Dashboard.version` on the versioned plane
  still carry the combined string (`0.0.1 (contract <rev>)`, what
  `--version` prints) while `hello` carries `version` and `contract`
  apart; `acq daemon status --json` adds `contract` and `artifact` from
  the handshake beside them, so the compatible and incompatible reports
  share the identity keys, but a `version` reads differently in the two
  shapes. Splitting the versioned field is a fixture diff outside the
  bootstrap frames — the owner's call, not step 4's; the two samples'
  literals still say `runtime` for the same reason.
- The packet's `acqd: {path, len, modified}` (§2.2) landed as `{path,
  len, mtime_ns}`: one integer field (nanoseconds since the epoch) is
  exact for the fast-path comparison and needs no formatter in the
  protocol crate; the name says what it holds.
- C39 amended 2026-09-10 (the owner, verbatim: "change core + the
  store to protocol + the store, and use 'the daemon never links the
  planner' in the Why. Do not use 'never reads facts or intent': because
  the daemon links the store, the dependency graph cannot enforce that
  it never reads facts. It does enforce the absence of the planner
  dependency; intent blindness is separately covered by C34"). 439
  bytes.
- Every binary has two forms, and the tree flips between them from
  cache in a tenth of a second (2026-09-11, verified by hash): the plain
  `cargo build --workspace` writes `acqd` `20093483f1cd`, `acq`
  `35af1c8a4d16`, `acq-mcp` `6c63c6471e17`; `cargo build --workspace
  --all-targets` writes `a1217dc36951`, `54f33e9ec381`, `33bb13a905a6` —
  under every target the store's `test-hooks` feature (a dev-dependency
  of the daemon) unifies into every build, so each bin is another
  artifact. From the plain build, the gate's `cargo test --workspace
  --all-targets` leaves `acqd` as written but rewrites `acq` and
  `acq-mcp` with their all-targets forms; `cargo clippy` rewrites none;
  the plain build restores all three. So the gate's process tests drive
  all-targets frontends against the plain daemon (the frontends' form
  does not enter the identity; the daemon's does), and AGENTS.md, the
  README tour and the skills, which said tests rebuild neither binary,
  were wrong about the frontends and now say this. Two rules follow:
  build with the gate's `cargo build --workspace`, never the all-targets
  form, before anything that starts a daemon; and never run the drivers
  while the gate runs — their preflight builds, and a build under a live
  daemon is an artifact mismatch (the round-16 gate saw exactly that).
  The `test-hooks` retirement the packet parked would remove the second
  form of every binary.
- `acq jobs --watch` ends when the daemon goes away rather than waiting
  for it to return: an observer never spawns (C10), and a watch that
  waits for a daemon is a design choice for the GUI's subscriber, not
  the CLI's.
