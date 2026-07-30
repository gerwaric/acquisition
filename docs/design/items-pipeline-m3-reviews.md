# Items Pipeline Milestone 3: Review Findings

Review-round findings for `items-pipeline-m3.md`, recorded from
round 1 per the M2 convention: each finding gets an `R<round>-<n>`
number, a verdict (accepted / rejected / amended, with reasons), and
a resolution naming the decision or criterion it changed. The spec
cites findings by number; this file is the record.

## Revision log

- **Revision 1 (July 30, 2026)** — initial draft. Written after the
  S1-M3 sort-profiling spike (evidence in `m3-sort-profile-result.md`)
  and the same-day lever hold point, whose decisions it records:
  levers A + B both committed (D1, D2), the intended
  `(name, uid, hash)` tie-break order (D5, fixing F67), and the
  key-memory measurement run pre-freeze as a spike extension
  (~286 B/item accounted, ~266 MB naive at 1m). No review rounds yet.
- **Revision 2 (July 30, 2026)** — round 1 incorporated (external
  review, eight findings R1-1…R1-8, all verified against the code and
  accepted; none challenged the hold-point decisions). Key changes:
  content deltas erase **source-scoped** within the stable-key display
  bucket (R1-1); the final snapshot performs authoritative **row**
  reconciliation, covering deletions and new unfetched tabs (R1-2);
  selection becomes an *intent* keyed by stable item id that survives
  mid-refresh removal and is re-adopted globally (R1-3); the
  intersection contract gains a metadata half, retiring M2 R7-2's
  metadata-only exception (R1-4); the key/sortedness state machine is
  specified with **transient** By-Tab keys and resident keys only for
  By-Item — a simplification that also shrinks the memory story
  (R1-5); buyout invalidation gains a batching/observability contract
  and the migration paths (R1-6); deltas the active search applies
  leave it clean, renegotiating M2 D9 rule 1 for the active search
  only (R1-7); the broad-filter default-expanded worst case is
  documented and budgeted (R1-8).

## Round 1 (July 30, 2026 — external review of revision 1)

Eight findings, all **accepted** after claim-by-claim verification
(M2 D2/D6/D9/R6-3 texts; `buyoutmanager.cpp:369` (`MigrateItem`),
`search.cpp:261-290`, `column.cpp`). None challenges the settled
lever or tie-break decisions.

| # | Severity | Finding | Verdict and resolution |
|---|---|---|---|
| R1-1 | High | D3 confused fetch-source replacement with display-bucket replacement: a Map/Unique child delta, applied as "replace the bucket's rows", would erase sibling sources sharing the parent display bucket — contradicting M2 D2's accepted mixed-generation behavior. | Accepted. Verified against M2 D2 (atomic unit = fetch source) and D6/R5-1 (buckets key on stable `(type, id)`, aggregating sources). D3's replacement is now **source-scoped within the display bucket**: erase exactly the rows whose items were fetched from the delta's `FetchSourceKey`, insert the arrivals; sibling sources untouched. Source-scoped erase (over rebuild-from-published) chosen because it stays within the signal payload and mirrors the reconciliation's erase-by-predicate shape. New pin `childDeltaPreservesSiblingSourcesInParentBucket`. |
| R1-2 | High | The final reconciliation (metadata + moves only) omitted final-only additions and deletions: M2 D6 keeps deleted tabs (with items) and newly discovered unfetched tabs snapshot-boundary-only, so no delta removes/inserts them and the model would retain deleted content and miss new empty tabs. | Accepted. Verified against M2 D6's publication table. The final snapshot now performs **authoritative row reconciliation**: one pass diffing the model against the post-snapshot published state per stable key — deleted buckets/rows removed, newly listed tabs inserted (unfiltered searches), metadata refreshed, bucket order fixed by moves; row operations only, never a reset. New pins `finalReconciliationRemovesDeletedTabs`, `finalReconciliationInsertsNewlyListedEmptyTabs`. |
| R1-3 | High | Immediate remove-then-insert application loses M2 R6-3's global cross-tab selection fidelity: when the removal delta arrives before the insertion delta, naive clearing breaks `reselectionSurvivesCrossTabMove`; `byItemSelectionSurvivesMerge` as drafted said "removed clears" immediately. | Accepted. Verified against M2 R6-3 (global identity lookup, cross-tab pin). D3 gains a **selection-intent contract**: intent is keyed by stable item id, survives row removal during an active refresh, is re-adopted globally when any later delta inserts the id, and is cleared at the final reconciliation if the id is absent or by user action; outside an active refresh, removal clears immediately. New pin `selectionIntentSurvivesCrossTabMoveAcrossDeltas` (the M2 pin's successor under no-reset machinery); `byItemSelectionSurvivesMerge` reworded accordingly. |
| R1-4 | High | D3 promised immediate new-tab insertion and rename/move/color updates but inherited M2's item-based intersection gate, which deliberately excludes metadata-only (empty) deltas — post-M3 those would wait indefinitely, especially after terminal failure (no throttled tick, no guaranteed refilter). | Accepted. Verified against M2 D6 ("D9 deliberately has no metadata-only intersection trigger") and R7-2. The intersection contract gains a **metadata half**: every delta's location anchor lands in the canonical inventory immediately (existing M2 machinery), and a delta whose stable key owns a visible bucket — or would create one in an unfiltered search — applies metadata now (`dataChanged`, moves, empty-bucket insertion), item intersection notwithstanding. M2 R7-2's exception is explicitly renegotiated and retired. New pin `metadataDeltaAppliesWithoutItemIntersection` (empty rename/move/color/new-tab, no final snapshot, persisting after terminal failure). |
| R1-5 | Medium-high | The lazy bucket/key state machine was incomplete and internally inconsistent: D1 said collapsed buckets hold no keys, D2 said they retain arrival order; expand → sort → collapse left key eviction, order retention, and arrival-order reconstruction unspecified, and each candidate resolution broke a stated claim. | Accepted, resolved by simplification: **By-Tab keys are transient sorting scratch** (built for a sort, discarded after; what persists is the bucket's order and a sorted-validity flag), and **resident keys exist only for the By-Item flat bucket** (D4 needs them for merges). Collapse changes nothing (order and flag persist; arrival order is never reconstructed); invalidation acts on flags, not caches. D1/D2 now state the full transition table (expand, collapse, delta, column switch, direction flip, buyout invalidation). Strengthens the memory claim: resident key memory is By-Item only; By-Tab peak is one bucket (~0.2 MB). New pins `sortedOrderSurvivesCollapse`, `byTabKeysAreTransient`; `sortColumnSwitchRebuildsMaterializedKeysOnly` superseded by `sortColumnSwitchResortsVisibleBucketsOnly`. |
| R1-6 | Medium-high | The buyout invalidation contract lacked observability and batching: no statement of when a visible bucket reorders, how Price/Date **cells** repaint when another column is active, whether migration (`MigrateItem`/`MigrateTab`) counts as mutation, or how bulk pricing passes avoid one reorder per `Set` (quadratic risk on the flat bucket). | Accepted. Verified `MigrateItem` at `buyoutmanager.cpp:369` mutates the lookup result. D1 rule 4 now specifies: the choke-point inventory includes item/tab set-and-clear, **migration**, and the scoped and final pricing passes; pricing passes batch — one invalidation batch, hence at most one reorder/model update per pass; user edits apply immediately; and Price/Date cells repaint via `dataChanged` for affected visible rows regardless of the active sort column (reordering alone is gated on the active column). New pins `pricingPassYieldsSingleModelUpdate`, `priceCellsRepaintUnderAnySortColumn`; `priceKeysFollowBuyoutEdits` extended to migration. |
| R1-7 | Medium | Inheriting M2 D9 rule 1 unchanged makes the active search dirty on every delta it just applied, so switching away and back triggers a spurious full refilter; the final reconciliation didn't clear the flag either. | Accepted. Verified against M2 D9 rule 1's wording. Renegotiated **for the active search only**: a delta the active search applied — including one correctly adjudicated as no visible change — leaves it clean; any skipped application leaves it dirty (fail-safe); the final reconciliation clears it. Background searches keep rule 1 verbatim. New pin `appliedDeltasLeaveActiveSearchClean`. |
| R1-8 | Medium | "Filtered results are small by construction" is not a bound: a broad filter can match nearly the whole collection while default-expanding every bucket, approaching full keyed sorting — a case no budget covered. | Accepted. Verified `m_filtered` semantics (`search.cpp:261-290` — any excluded item sets it). D2 drops the claim and states the honest ceiling (key build + sort of ~everything atop the filter loop, ~0.9 s estimated at 1m; memory unaffected under R1-5's transient keys). M1-M3 gains a broad-filter default-expanded scenario with a ≤ 1.2 s budget at 1m. |

Round narrative: the round's through-line is that revision 1
specified the delta path's *happy* grain (one content delta, one
visible By-Tab bucket) and under-specified the boundaries — source
vs. display-bucket grain (R1-1), snapshot-boundary effects (R1-2),
cross-delta state (R1-3), metadata-only deltas (R1-4), state-machine
transitions (R1-5), batching (R1-6). The lesson mirrors M2 round 2's:
contracts that hold at the single-event grain must be re-checked at
the sequence grain (two deltas in flight, a pass of many mutations, a
whole refresh). One finding (R1-5) was resolved by simplifying the
design rather than adding state — transient keys retire the cache
lifetime questions instead of answering them.
