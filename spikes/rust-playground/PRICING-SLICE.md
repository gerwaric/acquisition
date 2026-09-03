# The pricing slice — record (open)

The pricing slice (the packet, `brainstorming-notes/10-pricing-design-packet.md`
§2: ten steps, offline, validated by real use) opened on 2026-09-03. This
file is its record in the mold of `REFRESH-SLICE.md`: the step ledger,
the findings table (one row per review round, each finding with what
holds it now), what the census and the readings taught, and the
observations still open. It closes at step 10.

The rulings are `decisions/pricing.md` (C64–C75, C78), `decisions/plans.md`
(C76, C77) and `CONTEXT.md` (C79); properties are pinned by the tests
named below; facts about the trade site and the forum become `T<n>`
claims authored master-side. Nothing here is a second authority.

## State

- Opened at step 2 on 2026-09-03. Annotations schema **v2**, facts **v7**,
  sync policy **v3**, plan schema **6**; no pricing code yet.
- Reading a store or intent change before reviewing one: `REFRESH-SLICE.md`'s
  findings table is the checklist, plus the rows below.

## Step ledger

| Slice | Step | Commits | What landed |
| --- | --- | --- | --- |
| Pricing | 1 harvest | `ecdb9d3d`, `889c941d`, `0e4a4d9f` | the accepted packet into the registry (C64–C79); parks routed to `decisions/<area>.md` |
| Pricing | 2a clause audit | `8aa7a507` | C35–C44 and C52 cited from the tests that hold them; C44's fact-drift half pinned; the dependency direction (C34, C39, C41) a `docs-check` check |

## Findings

One row per review round; the finding, then the property or test that
holds it now.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 2a clause audit | `8aa7a507` | (1) C41's caller-asserted freshness read ("fails with the exact `RefreshPlan` it would take") has no consumer and no code — held by nothing; its first consumer is C72's report at step 8. (2) "The daemon never reads the store" was described as graph-enforced, but the daemon crate links the store crate to *write* facts (C28); what the graph can enforce is daemon ∌ planner and daemon never names the intent API — now a check. (3) C44's "fact drift does not refuse" was stated in the doc and held by the shape of `check_spendable`, pinned by nothing. (4) The uncited ids were the ones whose pins existed under descriptive names; the fix was citation, not tests — 30 tests now name their id. | (1) `decisions/plans.md` C41 *Pinned:* says so; (2) `tools/docs-check.sh` check 4; (3) `c44_fact_drift_never_refuses`; (4) doc comments on the tests, `docs-check`'s uncited report |
| 2b annotations review, fixed now | (this commit) | (1) `synchronous=NORMAL` under WAL keeps the file consistent but lets the last commits before a power loss roll back — on the one file with no server to refetch from. (2) `export` wrote `dest` directly and never fsynced it: an interrupted export left a partial file that looked like a backup and, under never-overwrite, blocked every retry. | (1) FULL; `c35_the_intent_file_is_fully_synchronous`; a single-row put measured 87 µs on this machine. (2) `<dest>.partial` → `quick_check` → fsync → link; an existing `dest` refuses before anything is written; `export_is_a_consistent_snapshot_and_never_overwrites` |
| 2b annotations review, constraints on step 3 | — | (3) Tombstone generations are invisible: `get`/`list` hide a tombstoned row and `Conflict { current: None }` hides its revision, so C71's precondition "absent at generation *g*" can be neither stated nor checked, and `put(None)` conflates never-existed (revision 0) with tombstoned. (4) One IMMEDIATE transaction per `put` and no batch: all-or-none for a plan of hundreds is impossible through the API — cost is not the issue (10k single puts in 449 ms under NORMAL), atomicity is. (5) The migration shape `0 \| 1 => CREATE TABLE IF NOT EXISTS` cannot add a column; v3 (`written_via`, `actor`, `applied_plan`, the receipt table) needs stepwise `ALTER TABLE … ADD COLUMN … NOT NULL DEFAULT 'unknown_legacy'` inside the same IMMEDIATE transaction; a v3 file is already refused by a v2 build. (6) The `tab` key is realm-less and a substash is the caller's `parent/id` convention; C67's targets carry realm and the owner's file holds zero tab-scoped rows — the realm-bearing key must be defined before the first row lands (P3). (7) `list` filters by scope only; pricing reads want one `kind` across four scopes; at 10k rows `list` takes 35 ms, `get` 27 µs, export 7 ms — a kind filter is needed, an index is not. (8) A writer blocked past the 5 s busy timeout surfaces as `Db(…)`, indistinguishable by kind from any other SQLite error, though a driver must tell "retry later" from "re-read and retry". (9) Receipt growth: an import of this owner's file is ~1.4k mutations, ~300 KB per receipt at ~200 B each — megabytes a year at human pace; a `receipts` + `receipt_mutations` layout indexed by target keeps "since *T*" a query and is a representation of C78's receipt, not the parked event log. (10) `open_for` creates the file and stamps the uuid on read paths — a read that writes; harmless, noted. | step 3 builds against these rows; each becomes a pin there |

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the step that touches it.

- C38's annotation-revision basis is one row (the policy); a `PricePlan`'s
  is many (C71's precondition set). The envelope discipline C75 tests
  will meet this first at step 6.
