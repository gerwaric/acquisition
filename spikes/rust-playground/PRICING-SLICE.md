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
| Pricing | 2a clause audit | (this commit) | C35–C44 and C52 cited from the tests that hold them; C44's fact-drift half pinned; the dependency direction (C34, C39, C41) a `docs-check` check |

## Findings

One row per review round; the finding, then the property or test that
holds it now.

| Round | Commit | Findings | Held by |
| --- | --- | --- | --- |
| 2a clause audit | (this commit) | (1) C41's caller-asserted freshness read ("fails with the exact `RefreshPlan` it would take") has no consumer and no code — held by nothing; its first consumer is C72's report at step 8. (2) "The daemon never reads the store" was described as graph-enforced, but the daemon crate links the store crate to *write* facts (C28); what the graph can enforce is daemon ∌ planner and daemon never names the intent API — now a check. (3) C44's "fact drift does not refuse" was stated in the doc and held by the shape of `check_spendable`, pinned by nothing. (4) The uncited ids were the ones whose pins existed under descriptive names; the fix was citation, not tests — 30 tests now name their id. | (1) `decisions/plans.md` C41 *Pinned:* says so; (2) `tools/docs-check.sh` check 4; (3) `c44_fact_drift_never_refuses`; (4) doc comments on the tests, `docs-check`'s uncited report |

## Observations still open

Agent observations that became neither a ruling nor a finding; each is
data for the step that touches it.

- C38's annotation-revision basis is one row (the policy); a `PricePlan`'s
  is many (C71's precondition set). The envelope discipline C75 tests
  will meet this first at step 6.
