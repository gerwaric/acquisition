# Items Pipeline M2: Implementation Sequence

Status: **PROPOSED — awaiting review** (July 29, 2026). This document
sequences the production implementation of the frozen M2 spec
(`items-pipeline-m2.md`, frozen at revision 9 plus the July 29
post-freeze amendments). It makes **no design arguments and changes
no decisions**: the spec is frozen, and any conflict between this
document and the spec is a bug in this document. What is reviewable
here is the *ordering* — stage boundaries, dependencies, gates, test
placement, and the intermediate states the branch passes through.

Citation convention follows the spec: bare D-numbers are the spec's
decisions, `R*-*` are review-round findings, pinned test names are
`camelCase`. "The spec" means `items-pipeline-m2.md` throughout.

## Baseline

- Branch: `items-pipeline-m2`, created off master `eb0ebd77` (the
  PR #184 merge that landed the frozen spec). The branch merges to
  master as **one PR at the end**, so intermediate states documented
  below never reach master.
- `spike/s1-m2-throttle` is **reference-only** (R3-4, R7-3): it is
  never merged, and its D9/R6-3 prototype code is never cherry-picked
  — that code is non-production by charter and is reimplemented
  against the spec. The single sanctioned import is
  `tests/spikedataset.h` (spec, S1-M2 discharge note), ported fresh.
- Working rules: every commit compiles and passes
  `ctest --test-dir build --output-on-failure`; new problems found
  along the way go to `docs/cleanup/findings.md`, not inline fixes;
  each spec section's staleness anchors are re-verified immediately
  before implementing against it.

### Staleness-preamble re-verification (July 29, post-F62 master)

The spec's staleness preamble was written against `d995840b`
(pre-F62). Re-verified against `eb0ebd77`: **every named anchor still
exists**, with minor line drift and one deliberate shape change:

| Anchor | Preamble | Now |
|---|---|---|
| Worker signal surface | `itemsmanagerworker.h:111-140` | `:111-149`; handlers/signals now carry F62's typed payloads (`Result<poe::StashPayload>` / `CharacterPayload`, raw bytes in the persistence lane) — the final shape D1 requires |
| `OnStashReceived` | `:896-1026` | `:896` (ordering unchanged) |
| `RemoveItemsFetchedBy` | `:451` | `:448` |
| `AbortUpdate` | `:525` | `:525` |
| `StopUpdateForFailure` | — | `:555` |
| `FinishUpdate` | `:1227-1281` | `:1226`; rebase at `:1261-1262` |
| `ItemsManager::OnItemsRefreshed` | `itemsmanager.cpp:121-147` | `:121`; `ClearRefreshLocks` at `:93` |
| Shop auto-submit | `application.cpp:404-414` | `:404-411` |
| `ModelViewRefresh` / expansion / reselect | `mainwindow.cpp:736-775`, `:709` | `:736`, `:709`, `ReselectCurrentItem` at `:833` |
| `m_states_dirty` | `search.h:82-85` | `:85` |

The preamble's "F62 is decided but NOT implemented" bullet is the one
entry that is now stale in the right direction: F62 is merged
(PR #183), so D1's precondition is satisfied and the worker anchors
above are already the post-F62 shapes.

## Ordering principle

**Vertical reply path first.** M2-M2 is the spec's first
implementation checkpoint (R7-3) and must run on the real production
reply path — worker handler → persistence → manager apply → scoped
pricing → UI intersection/fan-out — not on a throwaway slice. So the
stages before the measurement build exactly that path and nothing
else; everything *not* in M2-M2's measured synchronous unit (the
R6-3 fidelity machinery, the D4 terminal event, the D8 shop rework)
comes after the gate. Rationale per exclusion:

- **R6-3 fidelity** runs on the throttled *reset* path, not the
  synchronous per-reply path M2-M2 times.
- **D4 `RefreshFinished`** is per-update, not per-reply; it adds
  nothing to the measured unit. (D5's skip path changes only the
  failure branch; M2-M2's fixed shapes are successful replies and
  removals.)
- **D8** hangs off D4 and touches no per-reply code.

## Stage sequence

### Stage 0 — Test infrastructure

Port `tests/spikedataset.h` fresh
(`git show spike/s1-m2-throttle:tests/spikedataset.h`), review to
production standard, wire into `tests/CMakeLists.txt`. Verify the
branch baseline builds and the full suite passes before any product
code changes. No other spike code crosses over.

### Stage 1 — D3, worker half

- `FetchSourceKey` `{ItemLocationType, QString fetch_id}` as a named
  type shared by both sides of the signal (R1-3).
- `TabRefreshed(location, items)` emitted in
  `OnStashReceived`/`OnCharacterReceived` immediately after the
  atomic replace, before the counter increment (D3's emit point).
- The worker's erase predicate gains the type
  (`RemoveItemsFetchedBy`), so worker and published-copy erases can
  never diverge (R1-3). Scope boundary per R6-5: only M2's new
  predicates are rekeyed; legacy bare-id stores stay as recorded in
  F66.
- `ChildrenReconciled(parent, expected)` carrying the authoritative
  expected set (R5-2/R6-2), emitted beside the persistence-lane
  `stashChildrenReplaced` (the lanes stay split, D3).
- The cached initial load stays one snapshot emit (no streaming).

Files: `itemsmanagerworker.{h,cpp}`, key type header.
Tests (worker-observable halves): `deltaMatchesAppliedReplacement`,
`parentReplyReconcilesChildrenAgainstExpectedSet` (emit ordering and
payload; published-copy assertions complete in stage 2).

### Stage 2 — D3 manager half + D7 scoped pricing

- `ItemsManager` applies each delta to the published copy: one
  predicate-only `erase_if` by `FetchSourceKey` (the permitted linear
  pass, R1-2) + append; applies `ChildrenReconciled` by running the
  expected-set predicate against its own baseline; re-emits both
  signals for the UI.
- D7 scoped pricing per delta, delta items only: note-based item
  buyouts, tab-inheritance from published tab-buyout state, monotone
  refresh-lock additions. Tab-name auto-pricing stays final-pass-only;
  `ClearRefreshLocks` stays exclusive to the final pass.
- F46 absorbed (R1-9): the debug uncategorized scan in
  `OnItemsRefreshed` gated behind `spdlog::should_log` or deleted;
  F46's register entry moves to the resolved ledger in the same
  commit.

Files: `itemsmanager.{h,cpp}`, findings register.
Tests: `publishedStateIsSnapshotPlusAppliedDeltas`,
`emptyDeltaEmptiesFetchSourceOnly`,
`reconcileErasesGhostsAcrossFailedUpdates`,
`scopedPricingConvergesToFinalPass`,
`scopedPricingIsFailSafeAcrossFailedUpdate`,
`parentBucketMayMixChildGenerationsMidRefresh`, and completion of the
stage-1 pins.

### Stage 3 — D6 stable-identity bucketing + D9 five-rule consumer

Ordered within the stage: the bucketing rule lands **first**, because
mid-refresh refilters are unsound without it (R5-1's three failure
shapes — split, stale-header, merge).

- Canonical-location inventory ingesting every delta's location
  anchor, empty deltas included (R6-1); `Search` buckets keyed by
  stable `(type, id)`, rendering the freshest metadata seen per key;
  the unfiltered empty-bucket source list resolves through the
  inventory.
- D9's five-rule state machine in `MainWindow`: per-search
  items-dirty flags beside `m_states_dirty` with the extended
  activation gate; intersection decided on the delta alone, both
  halves (filter-match + visible fetch-source removal);
  non-resetting trailing throttle with **injectable S** (production
  value 60 s); rules 3–5 (tick refilters and clears own flag;
  refilter/tab-switch/deletion cancel semantics; final snapshot
  cancels and clears all).

Files: `search.{h,cpp}`, `mainwindow.{h,cpp}`, `items_model` only if
the bucketing mechanism requires it.
Tests: `bucketsKeyOnStableIdDuringRefresh`,
`emptyDeltaMetadataLandsAtNextRefilter`,
`backgroundDeltaLeavesModelUntouched`, `removalOnlyDeltaIntersects`,
`throttleDoesNotRearm`, `tabSwitchBeforeTickPreservesDirty`,
`searchDeleteCancelsPendingTimer`, `finalSnapshotCancelsPendingTick`,
`successfulRefilterCancelsPendingTick`,
`pendingTickSurvivesTerminalFailure` — the last asserting the
**amended freshness bound: S plus one reset-plus-restore duration**.

### Stage 4 — M2-M2 measurement (GATE)

The spec's first implementation checkpoint (D3, R7-3; open-items
list). Harness written fresh against production code (the spike
harness is wiring reference only), reusing the stage-0 dataset
generator:

- Complete synchronous reply application on representative **100k
  and 1m** datasets, **Release** build, offscreen; recorded
  environment (hardware, OS, compiler, Qt, allocator); **fixed**
  recorded reply/removal shapes; repetitions and the reported
  statistic recorded.
- **Per-component attribution**: worker erase, manager erase,
  persistence, pricing, UI intersection/fan-out (R3-3/R4-1 — worker
  and manager attributed separately before any remedy is selected).
- Budgets: manager marginal erase **< 2 ms @ 100k, < 16 ms @ 1m**
  gates the *required* manager remedy (source-keyed map with lazily
  rebuilt flat vector); whole-path budget (same thresholds, one
  frame at 1m) gates a *required real remedy* for the dominant
  component. A selected remedy is validated by **rerun** before M2
  is complete (R6-6).
- Result recorded in an addendum document beside the spec (the
  `s1-m2-spike-result.md` convention), pointed to from the spec's
  open-items entry.

**Hold point: implementation pauses here and the numbers go to Tom
before proceeding** — a budget miss changes the shape of the
remaining work (remedy implementation + rerun joins the sequence).

### Stage 5 — R6-3 restore-fidelity contract (outcome (a))

Reimplemented against the spec, not ported from the spike:

1. Expansion keyed by stable `(type, id)` (not header text).
2. Reselection by stable item identity via a **global, index-backed
   identity lookup** — explicitly *not* the spike's bucket-scoped
   lookup, which the post-freeze amendment records as a narrowing;
   `reselectionSurvivesCrossTabMove` pins the production behavior
   (global, index-backed — not a whole-model scan per reselect).
3. Scroll preserved across throttled resets: top-row anchor with
   **raw-scrollbar-value fallback** when the anchored row was removed
   — never scrolling the anchor's bucket header to the top (the
   post-freeze amendment; the spike initially got this wrong).
4. Capture of expansion, selection, and scroll **immediately before
   every reset**, including the refresh path that today restores
   without saving.

Files: `mainwindow.{h,cpp}`, `search.{h,cpp}`.
Tests: `expansionSurvivesRenameByStableKey`,
`selectionSurvivesReplacementByStableIdentity`,
`scrollAndCaptureSurviveThrottledReset`,
`reselectionSurvivesCrossTabMove`.

### Stage 6 — D4 typed terminal event

- `RefreshOutcome` = `variant<CompletedRefresh, FailedRefresh>`,
  `SkippedSource` vocabulary (empty skipped list until D5 lands next
  stage); `RefreshFinished` emitted exactly once per accepted
  `Update()`, Idle observed before the emit on both paths.
- First-error plumbing: every value-level failure branch hands its
  `FetchError` to `StopUpdateForFailure`, stored **before** the test
  fault hook; catch-alls store `Internal`; reset at next accepted
  update.
- Delivering-terminal flag: `Update()` during terminal fan-out is
  refused (R4-4); chained restarts queue.
- `Canceled` with an unstopped token maps to
  `FailedRefresh{Canceled}`.
- `ItemsManager` forwards the signal.

Files: `itemsmanagerworker.{h,cpp}`, `itemsmanager.{h,cpp}`.
Tests: `terminalEventExactlyOncePerUpdate`,
`deltasNeverFollowTerminalEvent` (extends W-IDENTITY to both delta
signals), `terminalFanOutRefusesReentrantUpdate`.

### Stage 7 — D8 gate move, then D5 skip policy

Ordered within the stage to avoid a hazard window, and this ordering
is load-bearing:

1. **First** move automatic forum submission from `ItemsRefreshed` to
   `RefreshFinished`, gated on `CompletedRefresh` with an empty
   skipped list (R1-1). Behavior-equivalent at this commit — no skip
   path exists yet and failures never reached `ItemsRefreshed` — so
   the gate is in place before anything can produce a
   completed-with-skips outcome.
2. **Then** land D5: `Parse` failures on content fetches
   skip-and-continue (log, record skipped source, count received,
   no delta, previous items survive); everything else stays
   first-failure-terminal; list fetches always terminal; skips
   user-visible in the final status text. The missing-wrapper case
   arrives via F62's facade classification — verify the worker's
   untyped is-empty branches are fully deleted (D5/R2-4) and the
   facade `Parse` error takes the skip path.

If the commits landed in the opposite order, a completed-with-skips
refresh would auto-post stale contents for tabs the user selected —
the exact hazard R1-1 closes.

Files: `application.cpp`, `itemsmanagerworker.cpp`, `shop.{h,cpp}`
(connection only).
Tests: `shopSubmitsOnlyOnCleanCompletion`,
`parseFailureSkipsTabAndUpdateCompletes`,
`missingStashWrapperSkipsTab`, `missingCharacterWrapperSkipsTab`.

### Stage 8 — D8 full shop machinery

- Submission input captured **by value** at request time (R2-1);
  each job owns all its transport state (capture, force bit, legacy
  index, rendered data + hash, request counter, thread progress) —
  nothing mutable shared with a waiting job.
- Latest-eligible desired state (R3-1): at most one active job, at
  most one waiting automatic capture, newest clean capture wins;
  automatic admission captures before the busy policy; manual
  admission may still refuse while a job is active.
- Monotonic input/cache revisions replace `m_shop_data_outdated`;
  completing N can clean only N's revision; a rendered job publishes
  the preview cache only if not older than what is there. Deltas do
  not advance the input revision (R4-2's accepted blind spot).
- Terminal exits: success (including unchanged-hash no-post) drains
  the newest waiting capture; failure drains nothing, advances no
  clean revision, discards the waiting capture, leaves input dirty
  (R4-3: no kind discrimination).
- Waiting-capture transitions: disabling auto-update drops it
  (R5-6); a completed-with-skips or failed refresh does **not**
  invalidate it — keep-and-drain (R5-6); an output-affecting local
  edit drops it while the active job is unaffected (R6-4).

Files: `shop.{h,cpp}`, `application.cpp`.
Tests: `shopSubmissionUsesCapturedSnapshot`,
`newestCleanSnapshotSubmitsAfterActive`,
`automaticSubmissionCoalescesLatestEligible`,
`manualSubmissionRendersCapturedPublishedState`,
`olderSubmissionCannotCleanNewerInput`,
`failedSubmissionDoesNotDrainPendingAutomatic`,
`disablingAutoUpdateDropsWaitingCapture`,
`skippedRefreshDoesNotInvalidateWaitingCapture`,
`expireDropsWaitingCapture`, `activeJobUnaffectedByLocalEdits`.

### Wrap-up

- Design-review criteria pass (the spec's six checked-in-review
  items: no `TabRefreshed` reaching shop/currency/whole-collection
  passes; job-local shop transport state; O(delta) everywhere except
  the permitted passes; no shared payload types across lanes; one
  erase predicate both sides; no synchronous update from terminal
  fan-out + M2-M2 attribution recorded).
- M1-M2 (`QueueUpdated` burst vs. status-widget frame time, D10):
  blocks nothing; run opportunistically here or record as follow-up.
- Findings register and spec open-items housekeeping (M2-M2 addendum
  pointer; F46 resolution confirmed).
- PR to master.

## Known intermediate states on the branch

Stated so the review can judge them rather than discover them. None
reach master (single end-of-branch PR):

1. **Stages 2–7: manual shop submission mid-refresh can read
   partially streamed published state** (the R2-1 hazard — published
   `items()` now changes between snapshots, but capture-by-value
   lands in stage 8). Pre-existing manual submission still works; the
   exposure is a mid-refresh manual submission reading a mix, which
   today's code cannot experience. Closed by stage 8.
2. **Stages 3–4: throttled resets run with today's restore machinery**
   (fidelity lands in stage 5), so mid-refresh ticks on the branch
   lose selection/scroll exactly as the spec's R6-3 verification
   described. No test pins fidelity until stage 5 adds them. Closed
   by stage 5.
3. **Stages 1–5: no typed terminal event yet** — terminal failure
   behaves exactly as on master (AbortUpdate, no signal). D9's
   `pendingTickSurvivesTerminalFailure` is testable throughout (it
   depends on the absence of a final snapshot, not on
   `RefreshFinished`).

## Open question for the review

**Stage 3 ↔ stage 5 adjacency.** Fidelity (stage 5) is sequenced
after M2-M2 because it is not part of the measured synchronous unit
and the spec wants the measurement as early as the real path exists
(R7-3). The alternative — fidelity immediately after the D9 machine,
before M2-M2 — eliminates intermediate state 2 above at the cost of
delaying the gate by one stage. The proposal prefers the early gate;
the review may prefer the other trade.

## Pin-to-stage traceability

Every pinned test in the spec's acceptance criteria, mapped to the
stage that lands it (outcome-(b) pins are record-only and not
implemented, per revision 9):

| Pin | Stage |
|---|---|
| `deltaMatchesAppliedReplacement` | 1 (completed in 2) |
| `parentReplyReconcilesChildrenAgainstExpectedSet` | 1 (completed in 2) |
| `emptyDeltaEmptiesFetchSourceOnly` | 2 |
| `reconcileErasesGhostsAcrossFailedUpdates` | 2 |
| `publishedStateIsSnapshotPlusAppliedDeltas` | 2 |
| `scopedPricingConvergesToFinalPass` | 2 |
| `scopedPricingIsFailSafeAcrossFailedUpdate` | 2 |
| `parentBucketMayMixChildGenerationsMidRefresh` | 2 |
| `bucketsKeyOnStableIdDuringRefresh` | 3 |
| `emptyDeltaMetadataLandsAtNextRefilter` | 3 |
| `backgroundDeltaLeavesModelUntouched` | 3 |
| `removalOnlyDeltaIntersects` | 3 |
| `throttleDoesNotRearm` | 3 |
| `tabSwitchBeforeTickPreservesDirty` | 3 |
| `searchDeleteCancelsPendingTimer` | 3 |
| `finalSnapshotCancelsPendingTick` | 3 |
| `successfulRefilterCancelsPendingTick` | 3 |
| `pendingTickSurvivesTerminalFailure` | 3 |
| M2-M2 measurement + addendum | 4 (gate) |
| `expansionSurvivesRenameByStableKey` | 5 |
| `selectionSurvivesReplacementByStableIdentity` | 5 |
| `scrollAndCaptureSurviveThrottledReset` | 5 |
| `reselectionSurvivesCrossTabMove` | 5 |
| `terminalEventExactlyOncePerUpdate` | 6 |
| `deltasNeverFollowTerminalEvent` | 6 |
| `terminalFanOutRefusesReentrantUpdate` | 6 |
| `shopSubmitsOnlyOnCleanCompletion` | 7 |
| `parseFailureSkipsTabAndUpdateCompletes` | 7 |
| `missingStashWrapperSkipsTab` / `missingCharacterWrapperSkipsTab` | 7 |
| `shopSubmissionUsesCapturedSnapshot` | 8 |
| `newestCleanSnapshotSubmitsAfterActive` | 8 |
| `automaticSubmissionCoalescesLatestEligible` | 8 |
| `manualSubmissionRendersCapturedPublishedState` | 8 |
| `olderSubmissionCannotCleanNewerInput` | 8 |
| `failedSubmissionDoesNotDrainPendingAutomatic` | 8 |
| `disablingAutoUpdateDropsWaitingCapture` | 8 |
| `skippedRefreshDoesNotInvalidateWaitingCapture` | 8 |
| `expireDropsWaitingCapture` | 8 |
| `activeJobUnaffectedByLocalEdits` | 8 |
| Design-review criteria (6 items) | wrap-up |
| M1-M2 measurement | wrap-up (blocks nothing) |
