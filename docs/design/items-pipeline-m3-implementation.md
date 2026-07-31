# Items Pipeline M3: Implementation Sequence

Status: **ACCEPTED for implementation** (July 30, 2026; externally
reviewed the same day — round 1's four adjustments incorporated,
accepted on the follow-up pass). Implementation proceeds on branch
`items-pipeline-m3`, starting at S0. Round 1's adjustments, for the
record: two pin-to-stage traceability fixes —
pins whose closing machinery lands later than their declared stage,
and timer pins deleted in S4 while the By-Item fallback still
reached the timer; the column-gating qualifier on S2's conservative
re-sort; and tightened S6 verification (fallback-insensitive
revalidation, the storm's full input set, exact sub-budgets).
This document sequences the production implementation of the frozen
M3 spec (`items-pipeline-m3.md`, frozen at revision 4). It makes
**no design arguments and changes no decisions**: the spec is
frozen, and any conflict between this document and the spec is a bug
in this document. What is reviewable here is the *ordering* — stage
boundaries, dependencies, gates, test placement, and the
intermediate states the branch passes through.

The sequence below already incorporates four external sequencing
adjustments (Tom's reviewer, July 30, all accepted): the buyout
choke point lands **before** resident key caching goes live, so no
intermediate build can re-sort on stale cached keys; the D9
throttle is not retired universally until D4 exists — By-Item and
the final snapshot keep an explicit fallback; the selection-intent
contract lands **with** the first immediate delta application, not
after it; and the M2-pin supersession map is recorded here, up
front, as an input to the stages that delete those pins (the spec's
design-review criteria require exactly that).

Citation convention follows the spec: bare D-numbers are the M3
spec's decisions, `R<round>-<n>` its review findings, pinned test
names in `camelCase`. "M2" decisions and pins are cited qualified.

## Working rules

1. **Every commit is green and correct** — not merely compiling:
   no intermediate state may violate a spec contract that its
   already-landed machinery makes reachable. This is why the choke
   point precedes residency (a cached Price/Date key must never
   outlive a buyout edit uninvalidated) and why selection intent
   lands with immediate application (the M2 pin
   `reselectionSurvivesCrossTabMove` is green on master today and
   must never regress mid-sequence).
2. **Temporary fallbacks are loud seams.** The By-Item throttled
   path (deleted in S5) and the final-snapshot reset (deleted in
   S6) are retained deliberately; each is marked at the code site
   with the stage that deletes it, and its deletion is part of that
   stage's definition of done. **While a seam is reachable it stays
   pinned**: the M2 tests covering it survive, fallback-scoped,
   until the seam's deleting stage (see the supersession map).
3. **Conditional hold points (M2 vertical-slice lesson).** Stages
   S3, S4, and S5 run their applicable M1-M3 budget rows, Release
   build, spike presets, recorded environment. **A miss pauses the
   sequence at that stage with per-component attribution** (filter
   loop, key build, sort, model ops, merge — the M2-M2 discipline);
   Tom chooses the remedy before anything is built on top of the
   miss. S7 remains the formal complete-table gate regardless of
   earlier passes. The applicable rows are enumerated per stage
   below — "applicable" is never adjudicated on the fly.
4. **The spike branch (`spike/m3-sort-profile`) is reference-only.**
   Never merge it, never cherry-pick its instrumentation; the S0
   harness is built fresh (the M2 rule for `spike/s1-m2-throttle`,
   unchanged).

## Stage sequence

### S0 — datasets, anchors, probes

- Port the deterministic dataset generator fresh (presets
  smoke/100k/1m, the M2-M2 shapes) for the M1-M3 scenarios.
- Re-verify every staleness-preamble anchor in the spec against
  post-merge master (`a2ed4d96`); record drift in the result doc
  that M1-M3 will extend.
- Build the probe surface the pins require: key-build, comparator
  call, and sort counters; live resident-key bytes; model-update
  and reset counters; refilter and index-rebuild counters;
  capture/restore entry counters. Probes are test-observable
  without polluting production paths (the M2 pattern:
  test-only accessors, not logging).

No behavior change; suite stays green.

### S1 — D5 + keyed sorting, keys rebuilt per sort

- The one-token F67 fix (`Item::operator<` third element becomes
  `rhs.m_hash`); move F67 to the resolved ledger.
- The key representation (comparator-derived tuples, the D1 key
  shape including the `(PrettyName, uid, hash)` suffix and the
  `s2`/suffix buffer sharing) and keyed `Bucket::Sort`.
- **Keys are built at each sort and discarded** — no caching yet.
  This intermediate is the spike's measured 10× path, so the stage
  ships a real win on its own, and buyout correctness is trivial:
  keys can never be stale because they never outlive a sort.

Pins closed: `keyedOrderMatchesComparatorOrder`,
`intendedTieBreakRestored`.

### S2 — buyout choke point + the five batching rules

- Route every `BuyoutManager` mutation through the single choke
  point (item/tab set-and-clear, `MigrateItem`, the scoped and
  final pricing passes — D1 rule 4's exhaustive inventory).
- All five batching rules: command scope, pass scope, nested
  coalescing to the outermost boundary, the required snapshot batch
  around `MigrateBuyouts` → `ApplyAutoTabBuyouts` →
  `ApplyAutoItemBuyouts` → `PropagateTabBuyouts`, and
  column-independent cell repaint.
- Model response is conservative and column-gated: **with Price or
  Date active, one re-sort per outer batch with freshly built
  keys** (S1 semantics); with any other column active, the batch
  repaints affected cells and reorders nothing — the spec gates
  reordering on the active sort column at every batch boundary.
  Correctness is unconditional; the batching contract is exercised
  before cache correctness depends on it.

Pins closed: `pricingPassYieldsSingleModelUpdate`,
`multiSelectionBuyoutEditReordersOnce`,
`snapshotPricingSequenceEmitsOneModelBatch`,
`priceCellsRepaintUnderAnySortColumn`, and the behavioral half of
`priceKeysFollowBuyoutEdits` (reorder-at-batch-end; its
resident-key assertions are added in S3).

### S3 — D2 flags + the residency state machine; first hold point

- Per-bucket sorted flags and the full D2 transition table:
  sort-on-expand, collapse-evicts-keys-only, deferred re-sort,
  default-expanded filtered searches.
- D1 residency: resident vectors for the active search's
  materialized buckets, the hydration rule,
  activation-decides-dirtiness, eviction on deactivation, and the
  full invalidation contract (delta/switch/flip/buyout effects on
  resident vectors) replacing S2's conservative rebuilds.
  By-Item's flat-bucket residency machinery is included only as far
  as the shared bucket code takes it; its eager activation and
  merge are S5.
- `priceKeysFollowBuyoutEdits` gains its R3-2 assertions (resident
  By-Tab entries rebuild before the reorder; no re-sort on stale
  keys).

Pins closed: `collapsedBucketsDeferSorting`,
`restoredExpansionSortsRestoredBucketsOnly`,
`filteredSearchSortsAllVisibleBuckets`,
`sortedOrderSurvivesCollapse`,
`keyResidencyFollowsMaterialization`,
`reexpandedBucketFlipHydratesOnce`,
`sortColumnSwitchResortsVisibleBucketsOnly`,
`priceKeysFollowBuyoutEdits` (complete).

Two pins deliberately do **not** close here:
`residentKeysScopedToActiveSearch` is partially established (By-Tab
per-bucket laziness, eviction on deactivation, the aggregate-memory
assertion) and fully closes in S5, where its clean-By-Item
eager-hydration clause first becomes satisfiable.
`collapsedInvalidBucketResortsOnReexpand` requires content
replacement **by delta** and closes in S4 — S3 has no delta-grain
replacement primitive, and building one early just to close a pin
would be exactly the out-of-order scaffolding this sequence exists
to avoid.

**Hold-point budget rows (conditional):** worst-case unfiltered
By-Tab refilter (≤ 60 ms @100k, ≤ 500 ms @1m, sort share ≤ 5 ms);
single-bucket expand cold (≤ 10 ms at both scales); broad-filter
default-expanded refilter (≤ 150 ms @100k, ≤ 1.2 s @1m);
collapsed-default and background resident-key memory (≈ 0 /
exactly 0). By-Item rows wait for S5.

### S4 — By-Tab delta operations + selection intent; throttle retired for By-Tab only

- D3's bucket-scoped operations for the active search:
  source-scoped replace within the display bucket, visible-bucket
  sorted merge (R2-2), `ChildrenReconciled` row removals, new-tab
  insertion, metadata/move application (the R1-4 metadata half),
  incremental index maintenance, and the R1-7 dirty-flag
  renegotiation.
- **The complete selection-intent contract (R1-3/R2-1), landing
  with — not after — the first immediate application**: stable-id
  intent surviving mid-refresh removal, global re-adoption,
  user-selection precedence, and closure at **every**
  `RefreshFinished` outcome including terminal failure. S6 later
  changes only the success-boundary implementation (final reset →
  row reconciliation), never selection semantics.
- Throttle retirement is **By-Tab-scoped**: a By-Item active search
  keeps the D9 throttled-reset path as the rule-2 fallback seam
  (deleted in S5), and the final snapshot keeps its existing reset
  path (deleted in S6).

Pins closed: `unrelatedDeltaLeavesOtherBucketsUntouched`,
`deltaReplacesExactlyItsSourceRows`,
`childDeltaPreservesSiblingSourcesInParentBucket`,
`emptyDeltaEmptiesBucketWithoutRemovingIt`,
`deltaUpdatesVisibleIndexesIncrementally`,
`bucketRepositionsByMoveOnMetadataDelta`,
`metadataDeltaAppliesWithoutItemIntersection`,
`collapsedInvalidBucketResortsOnReexpand` (moved from S3: its
content replacement is delta-grain),
`staleOrderNeverSurvivesDelta`,
`selectionIntentSurvivesCrossTabMoveAcrossDeltas`,
`selectionIntentClearsOnTerminalFailure`.

`appliedDeltasLeaveActiveSearchClean` closes its **By-Tab half**
here; the By-Item half completes in S5. Under this stage's
fallback, a By-Item active search still refilters via the tick, so
the pin's full claim is not yet true and must not be asserted.

M2-pin turnover in this stage follows the supersession map below;
each deleted pin's commit cites its map entry. Pins the fallback
seams still exercise are **not** deleted here — see the map's
seam-reachability rule.

**Hold-point budget row (conditional):** delta application, By-Tab
visible bucket (≤ 5 ms @1m).

### S5 — D4 By-Item merge + eager activation; throttle fully retired

- The flat bucket's per-delta contract: erase by `FetchSourceKey`,
  keyed sort of arrivals, single merge pass, contiguous-run row
  ops.
- Eager key hydration at activation with dirtiness decided first
  (R3-1), completing the residency machinery S3 scaffolded.
- **Delete the By-Item throttle fallback** — the last D9 timer
  machinery goes here, with the remaining timer-encoding M2 pins
  (map below), including the fallback-scoped
  `tabSwitchBeforeTickPreservesDirty` and
  `scrollAndCaptureSurviveThrottledReset` kept green through S4;
  the latter's D6 retarget, **`scrollAndCaptureSurviveUserRefilter`**
  (capture/restore fidelity on the user-initiated refilter reset),
  lands here as its named replacement.

Pins closed: `byItemMergeMatchesFullSort`,
`byItemRemovalOnlyDeltaErasesInPlace`,
`byItemSelectionSurvivesMerge`,
`byItemActivationDecidesDirtinessFirst`,
`scrollAndCaptureSurviveUserRefilter`; and the halves deferred from
earlier stages complete here — `residentKeysScopedToActiveSearch`
(the clean-By-Item eager-hydration clause) and
`appliedDeltasLeaveActiveSearchClean` (By-Item, now
fallback-free).

**Hold-point budget rows (conditional):** By-Item full refilter
(≤ 250 ms @100k, ≤ 1.5 s @1m); By-Item merge (≤ 50 ms @1m); clean
By-Item reactivation hydration (≤ 100 ms @100k, ≤ 0.5 s @1m);
worst-shape resident key memory (≤ 300 MB @1m aggregate).

### S6 — final row reconciliation; last reset path deleted

- The R1-2 authoritative reconciliation on `ItemsRefreshed`:
  deleted buckets/rows removed, newly listed tabs inserted,
  metadata refreshed against rebased locations, order corrected by
  moves; O(collection) once per refresh; dirty flag cleared.
- **Delete the final-snapshot reset fallback.** Resets now exist
  only on D6's enumerated user-initiated paths and initial
  population.
- **Revalidate the final-reconciliation clauses of
  `selectionIntentSurvivesCrossTabMoveAcrossDeltas` and
  `byItemSelectionSurvivesMerge` against the new path explicitly.**
  Both clauses also pass under the temporary final-reset fallback,
  so merely staying green proves nothing about the reconciliation —
  the S6 assertions must show (probe: reset counter zero,
  reconciliation entered) that the row reconciliation, not a
  reset, performed the intent/selection clearing.
- Close the storm test with the full input set: content deltas,
  empty deltas, child reconciliations, metadata deltas, new-tab
  discoveries, and final reconciliations, randomized and
  interleaved with expansion changes, sort clicks, and view-mode
  switches.

Pins closed: `noModelResetDuringRefresh`,
`finalReconciliationRemovesDeletedTabs`,
`finalReconciliationInsertsNewlyListedEmptyTabs`,
`modelTesterPassesUnderDeltaStorm`.

### S7 — M1-M3: the formal complete-table gate (pause for Tom)

The full budget table from the spec's acceptance criteria, run
authoritatively on the finished model — Release, recorded
environment, spike presets, per-component attribution — regardless
of earlier conditional passes. Misses gate M3 completion the way
M2-M2's did: pause, attribute, Tom picks the remedy, rerun.
Result doc: `m1-m3-result.md` beside the spec.

### S8 — wrap-up

- Design-review criteria pass on the finished branch (no
  refresh-path reset; O(delta + bucket) everywhere but D4's stated
  merge; keys derived from comparators only; the buyout enumeration
  and batching rules; the stated M2 renegotiations — three after the
  spec's July 31 intersection-set amendment — and no
  others).
- Verify the supersession map below was executed exactly — every
  listed pin deleted/retargeted in its named stage, no strays.
- Turnover: spec open-items resolved, parent plan updated, findings
  register (F67 already moved in S1), this document marked
  IMPLEMENTED.

## M2-pin supersession map

The spec requires the D9-era pins deleted "by renegotiation, not
silent breakage — each mapped to its D3-era replacement in the
implementation plan." All live in `tests/tst_mainwindow.cpp`.

**Seam-reachability rule**: deletion stages are chosen against what
the fallback seams keep reachable, not against when a successor
lands. Through S4 the By-Item fallback still exercises the tick and
the throttled reset, so the pins covering them survive S4
fallback-scoped and die with the timer in S5. The S4 deletions were
checked against the seams: `reselectionSurvivesCrossTabMove`'s
successor and the S4 intent machinery cover both application paths
(intent is defined during an active refresh regardless of
mechanism), and `emptyDeltaMetadataLandsAtNextRefilter` has no
By-Item surface (the flat bucket renders no per-tab metadata rows),
so neither outlives its coverage.

Three categories:

**Superseded — deleted with the machinery, concern carried by a
named M3 pin:**

| M2 pin | Deleted in | Concern's M3 successor |
|---|---|---|
| `throttleDoesNotRearm` | S5 (timer deletion) | Bounded staleness → immediacy: `staleOrderNeverSurvivesDelta` |
| `tabSwitchBeforeTickPreservesDirty` | S5 — kept green, fallback-scoped, through S4 (the By-Item tick still reaches it) | R1-7 renegotiation: `appliedDeltasLeaveActiveSearchClean` (background half stays covered by `backgroundDeltaLeavesModelUntouched`, which survives) |
| `finalSnapshotCancelsPendingTick` | S5 (timer deletion) | `finalReconciliationRemovesDeletedTabs` / `finalReconciliationInsertsNewlyListedEmptyTabs` + the reconciliation's dirty-flag clear in `appliedDeltasLeaveActiveSearchClean` |
| `pendingTickSurvivesTerminalFailure` | S5 (timer deletion) | Applied-state persistence after failure: `metadataDeltaAppliesWithoutItemIntersection` (final clause) + `selectionIntentClearsOnTerminalFailure` |
| `emptyDeltaMetadataLandsAtNextRefilter` | S4 | R1-4 retired the M2 R7-2 exception it encodes: `metadataDeltaAppliesWithoutItemIntersection` |
| `reselectionSurvivesCrossTabMove` | S4 | Named successor in the spec: `selectionIntentSurvivesCrossTabMoveAcrossDeltas` — lands in the same stage, no green-test gap |
| `scrollAndCaptureSurviveThrottledReset` | S5 — kept green, fallback-scoped, through S4 (the By-Item throttled reset still runs it) | Refresh path: `unrelatedDeltaLeavesOtherBucketsUntouched` (nothing moves, nothing to restore); the capture/restore machinery is retargeted to D6's user-refilter reset as `scrollAndCaptureSurviveUserRefilter`, landing in S5 with the deletion |

**Deleted with the machinery, no successor needed (the hazard
disappears with the timer; residual interleavings covered by
`modelTesterPassesUnderDeltaStorm`):**

| M2 pin | Deleted in | Why no successor |
|---|---|---|
| `searchDeleteCancelsPendingTimer` | S5 | No timer to cancel; deltas for a deleted search are refused by search-scoped application |
| `successfulRefilterCancelsPendingTick` | S5 | No tick to cancel; refilter-vs-delta interleaving is storm-tested |

**Survive with adjusted assertions (mechanism changes, contract
does not):** `bucketsKeyOnStableIdDuringRefresh`,
`backgroundDeltaLeavesModelUntouched` (background searches keep M2
D9 rule 1 verbatim), `removalOnlyDeltaIntersects` (gains the
metadata half's boundary), `childReconciliationIntersectsVisibleGhosts`
(application becomes row removals),
`expansionSurvivesRenameByStableKey` (reset-restore → move ops;
companion `bucketRepositionsByMoveOnMetadataDelta`),
`selectionSurvivesReplacementByStableIdentity` (re-expressed over
the intent machinery in S4).

## Traps (carried from M2 and the spec)

- `reselectionSurvivesCrossTabMove` must never be red at any commit:
  its successor lands in the same S4 commit window that deletes the
  machinery satisfying it today.
- Keys are derived from the comparators — no code path defines
  order twice; regex work may be optimized *inside* the key build,
  never reintroduced per comparison.
- The two fallback seams (By-Item throttle, final-snapshot reset)
  are marked at their code sites with their deleting stage; S8
  verifies no seam survived.
- Budget runs are Release with the recorded environment; the
  10-char dataset-id caveat from M2-M2 applies to any comparison
  against recorded M2 numbers.
- Freshness after M3 is "immediate" for applied deltas; do not
  re-introduce any coalescing layer on the delta path (D3 retired
  the throttle *and* its reasoning).
