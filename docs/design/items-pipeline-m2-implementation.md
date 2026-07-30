# Items Pipeline M2: Implementation Sequence

Status: **IMPLEMENTED** (July 29, 2026) — all stages 0–8 landed on
`items-pipeline-m2`, every commit green on the full suite. The
stage-4 M2-M2 gate FIRED (both whole-path budgets missed); Tom
selected the spec's named remedy pair at the hold point
(`SourceKeyedItems` on both sides), and the R6-6 validation rerun
passes every budget — full record in `m2-m2-result.md`. The
wrap-up's design-review criteria were re-verified on the finished
branch; M1-M2 stays a post-M2 follow-up (blocks nothing). The
document below is retained as the reviewed sequencing record.

Previous status: **ACCEPTED for implementation** (July 29, 2026; externally
reviewed the same day in two passes — round 1: explicit
`ChildrenReconciled` treatment in stage 3 with its emission
condition disambiguated, the R2-1 capture front-loaded to stage 2,
M2-M2 attribution made exhaustive, and the stage 3 ↔ stage 5
question resolved in favor of the early gate; round 2: the
front-load widened from capture-only — which left two verified holes
— to the complete single-active-job shop foundation, with three more
pins moved to stage 2). This document
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
- **D8** hangs off D4 and touches no per-reply code — with one
  named exception: the single-active-job shop-safety foundation
  (full-input capture, render-from-capture, revision guard) lands
  in stage 2 **before** streaming is enabled, because forum posting
  is an external side effect and must never read partially streamed
  state (see stage 2).

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
  expected set (R5-2/R6-2). Its emission condition **follows the
  worker's own reconcile predicate** — every top-level parent reply
  (`fetch_id == id`, `itemsmanagerworker.cpp:991`), where the
  worker's ghost erase runs — **not** the narrower Map/Unique-only
  condition that gates the persistence-lane `stashChildrenReplaced`
  emit (`:1005-1007`, an F53 datastore concern). The two signals are
  emitted from the same reconcile block but on different conditions;
  the lanes stay split (D3).
- The cached initial load stays one snapshot emit (no streaming).

Files: `itemsmanagerworker.{h,cpp}`, key type header.
Tests (worker-observable halves): `deltaMatchesAppliedReplacement`,
`parentReplyReconcilesChildrenAgainstExpectedSet` (emit ordering and
payload; published-copy assertions complete in stage 2).

### Stage 2 — Shop foundation front-load, then D3 manager half + D7 scoped pricing

Ordered within the stage, and the ordering is load-bearing: forum
posting is an external side effect, so the single-active-job shop
foundation must exist **before** the published copy starts streaming
— "single PR" does not contain a stale forum post made from an
intermediate build run against a real profile.

1. **First**, the complete single-active-job foundation in `Shop`
   (external review, round 2 — capture alone left two holes):
   - Submission input captured **by value** at request time
     (automatic or manual): the postable items' identity, location,
     and buyout fields, plus **every other output-affecting input**
     — template, realm/league, and target thread list (the spec's
     job-capture enumeration, D8). The legacy stash index is applied
     to the capture when it arrives; the continuation never reads
     live `ItemsManager::items()` after capture.
   - **Every submission renders from its capture**, independent of
     preview-cache freshness. Hole 1: today's continuation reuses
     cached `m_shop_data` whenever `m_shop_data_outdated` is false
     (`shop.cpp:256-258`), so a manual submission after a streamed
     delta would post pre-delta text.
   - **Monotonic input/cache revisions replace the single
     `m_shop_data_outdated` flag.** Hole 2: a render resets the
     flag unconditionally (`shop.cpp:300`, `:356`), so an older
     captured job finishing after a local `ExpireShopData()` would
     mark newer input clean. Completing job N advances only N's
     revision; a rendered job publishes the preview cache only if
     not older than the cache already there. Deltas do not advance
     the input revision (R4-2's accepted blind spot).
   - The active job is unaffected by local edits — its capture is
     immutable; the edit reaches the forum through a later
     submission (R6-4's active-job half).
   What remains for stage 8 is strictly the multi-job machinery:
   latest-eligible queueing, the waiting automatic capture with its
   drop/drain transitions, and waiting-vs-active transport-state
   isolation.
2. **Then** streaming: `ItemsManager` applies each delta to the
   published copy — one predicate-only `erase_if` by
   `FetchSourceKey` (the permitted linear pass, R1-2) + append;
   applies `ChildrenReconciled` by running the expected-set
   predicate against its own baseline; re-emits both signals for
   the UI.
3. D7 scoped pricing per delta, delta items only: note-based item
   buyouts, tab-inheritance from published tab-buyout state, monotone
   refresh-lock additions. Tab-name auto-pricing stays
   final-pass-only; `ClearRefreshLocks` stays exclusive to the final
   pass.
4. F46 absorbed (R1-9): the debug uncategorized scan in
   `OnItemsRefreshed` gated behind `spdlog::should_log` or deleted;
   F46's register entry moves to the resolved ledger in the same
   commit.

Files: `shop.{h,cpp}`, `itemsmanager.{h,cpp}`, findings register.
Tests: the four foundation pins — `shopSubmissionUsesCapturedSnapshot`,
`manualSubmissionRendersCapturedPublishedState`,
`olderSubmissionCannotCleanNewerInput`,
`activeJobUnaffectedByLocalEdits` (the foundation lands first, but
the delta-dependent tests need streaming to drive, so they land at
the end of this stage) — plus `publishedStateIsSnapshotPlusAppliedDeltas`,
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
- **`ChildrenReconciled` is a first-class input to the state
  machine**, not an afterthought of the primary delta (external
  review, finding 1): rule 1's marking covers aggregate
  reconciliations ("every delta marks every search items-dirty —
  aggregate reconciliations included", spec D9 rule 1), and the
  intersection test has a third form — a `ChildrenReconciled`
  intersects iff any **visible** item under its parent carries a key
  outside the expected set (spec D9, R5-2/R6-2). Without this, a
  reconciliation that erases published ghost children schedules no
  refilter, and after a terminal failure those ghosts stay visible
  indefinitely — exactly the R6-2 published-baseline divergence the
  signal exists to fix.

Files: `search.{h,cpp}`, `mainwindow.{h,cpp}`, `items_model` only if
the bucketing mechanism requires it.
Tests: `bucketsKeyOnStableIdDuringRefresh`,
`emptyDeltaMetadataLandsAtNextRefilter`,
`backgroundDeltaLeavesModelUntouched`, `removalOnlyDeltaIntersects`,
`throttleDoesNotRearm`, `tabSwitchBeforeTickPreservesDirty`,
`searchDeleteCancelsPendingTimer`, `finalSnapshotCancelsPendingTick`,
`successfulRefilterCancelsPendingTick`,
`pendingTickSurvivesTerminalFailure` — the last asserting the
**amended freshness bound: S plus one reset-plus-restore duration** —
and `childReconciliationIntersectsVisibleGhosts`, a **plan-level
addition** beyond the spec's pinned list (external review, finding
1): a `ChildrenReconciled` whose expected set excludes visible ghost
children schedules the throttled refilter; followed by a terminal
failure, the tick still fires and the ghosts leave the view. It pins
behavior the spec's D9 intersection sentence already requires; it
adds no new design.

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
- **Per-component attribution, exhaustive** (external review,
  finding 3): worker erase, **parse + append**, persistence, manager
  erase, pricing, UI intersection/fan-out — parsing and append are
  in the spec's own enumeration of the synchronous unit (D3) and get
  a named bucket, not an implicit remainder. The buckets must sum to
  approximately the measured total, with any residual **explicitly
  reported as a residual**; a whole-path miss whose dominant cost
  hid in an unattributed remainder would leave the mandatory remedy
  unselectable. Worker and manager stay separately attributed before
  any remedy is selected (R3-3/R4-1).
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

The stage-4 checkpoint build is **unsuitable for UX validation or
normal profile use** (external review, stage 3 ↔ 5 resolution): the
R6-3 fidelity machinery has not landed, so mid-refresh throttled
resets lose selection and scroll. It measures the synchronous reply
path; it demonstrates nothing about feel.

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

- The single-active-job foundation (capture, render-from-capture,
  revisions) is already in place from stage 2; this stage adds the
  multi-job machinery: each job owns all its transport state
  (capture, force bit, legacy index, rendered data + hash, request
  counter, thread progress) — nothing mutable shared with a waiting
  job.
- Latest-eligible desired state (R3-1): at most one active job, at
  most one waiting automatic capture, newest clean capture wins;
  automatic admission captures before the busy policy; manual
  admission may still refuse while a job is active.
- Terminal exits: success (including unchanged-hash no-post) drains
  the newest waiting capture; failure drains nothing, advances no
  clean revision, discards the waiting capture, leaves input dirty
  (R4-3: no kind discrimination).
- Waiting-capture transitions: disabling auto-update drops it
  (R5-6); a completed-with-skips or failed refresh does **not**
  invalidate it — keep-and-drain (R5-6); an output-affecting local
  edit drops it while the active job is unaffected (R6-4).

Files: `shop.{h,cpp}`, `application.cpp`.
Tests: `newestCleanSnapshotSubmitsAfterActive`,
`automaticSubmissionCoalescesLatestEligible`,
`failedSubmissionDoesNotDrainPendingAutomatic`,
`disablingAutoUpdateDropsWaitingCapture`,
`skippedRefreshDoesNotInvalidateWaitingCapture`,
`expireDropsWaitingCapture`.

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

1. **The R2-1 hazard is closed before it can open** (external
   review, finding 2 — the original wording here understated it:
   *automatic* submissions, not just manual ones, can start for
   clean update N, wait asynchronously on the legacy stash index,
   and then read N+1's partially streamed state; and forum posting
   is an external side effect, so a single end-of-branch PR is not
   containment for anyone running an intermediate build against a
   real profile). Resolved by front-loading the complete
   single-active-job foundation to stage 2, ordered before streaming
   lands — capture alone was not enough (round 2): without
   render-from-capture a manual submission could reuse cached
   pre-delta `m_shop_data`, and without revisions an older job could
   mark newer input clean. With the foundation in place, what
   remains across stages 2–7: automatic submission still fires from
   `ItemsRefreshed` until the gate moves (stage 7) — today's trigger
   semantics; a busy shop still refuses rather than queueing until
   stage 8 — today's busy policy; and every submission now renders
   from its capture — a deliberate behavior change in the safe
   direction (never staler than the request, never partially
   streamed). No path remains by which a submission reads or posts
   mid-stream state.
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

## Resolved question

**Stage 3 ↔ stage 5 adjacency — early gate retained** (external
review, July 29): M2-M2 runs before the fidelity stage because it
determines the storage shape before more UI identity machinery is
built on top of it, and that placement best matches R7-3. The cost
is intermediate state 2 above, mitigated by labeling: the stage-4
checkpoint build is unsuitable for UX validation or normal profile
use (noted in stage 4).

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
| `shopSubmissionUsesCapturedSnapshot` | 2 (foundation front-loaded; was 8) |
| `manualSubmissionRendersCapturedPublishedState` | 2 (foundation; was 8) |
| `olderSubmissionCannotCleanNewerInput` | 2 (foundation; was 8) |
| `activeJobUnaffectedByLocalEdits` | 2 (foundation; was 8) |
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
| `childReconciliationIntersectsVisibleGhosts` (plan-level addition) | 3 |
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
| `newestCleanSnapshotSubmitsAfterActive` | 8 |
| `automaticSubmissionCoalescesLatestEligible` | 8 |
| `failedSubmissionDoesNotDrainPendingAutomatic` | 8 |
| `disablingAutoUpdateDropsWaitingCapture` | 8 |
| `skippedRefreshDoesNotInvalidateWaitingCapture` | 8 |
| `expireDropsWaitingCapture` | 8 |
| Design-review criteria (6 items) | wrap-up |
| M1-M2 measurement | wrap-up (blocks nothing) |
