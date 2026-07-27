# Items Pipeline Milestone 2: Streaming Refresh Signal

Status: **draft for review — not frozen**. Written July 27, 2026 on
branch `items-pipeline-m2-spec`. This spec consumes the M2 inbox in
`items-pipeline.md` ("Inputs accumulated since this sketch") and its
four hard constraints; the traceability table at the end maps every
input to the decision, deferral, or acceptance criterion that consumed
it. Implementation does not begin until this document is reviewed and
frozen (working rule 1).

Citation convention: bare D-numbers (D1, D2, …) in this document are
this document's decisions. Decisions of the network redesign are always
cited qualified ("network-redesign D6"). F-numbers are the findings
register; pinned test names are quoted in
`camelCase`.

## Staleness preamble

Written against commit `d995840b` (master, July 27, 2026). The load-
bearing code assumptions; re-verify these anchors before implementing
against any section, and re-verify the whole list if the worker or the
refresh path has been touched since:

- **Worker signal surface** (`itemsmanagerworker.h:111-140`): snapshot
  signal `ItemsRefreshed(Items, tabs, initial_refresh)`; cosmetic
  `StatusUpdate`/`NotifyUser`; persistence signals `stashReceived`,
  `characterReceived`, `stashListReceived`, `characterListReceived`,
  and the authoritative-list signals `stashListReplaced`,
  `characterListReplaced`, `stashChildrenReplaced` (F53). No
  presentation delta signal exists yet.
- **Per-reply emit ordering** (`OnStashReceived`,
  `itemsmanagerworker.cpp:896-1026`; `OnCharacterReceived` symmetric):
  failure branch (`StopUpdateForFailure` + `AbortUpdate`) → persistence
  emit `stashReceived` → atomic replace (`RemoveItemsFetchedBy(
  location.fetch_id())` then `ParseItems`) → counter increment +
  `SendStatusUpdate` → Map/Unique/Folder child-batch discovery → the
  parent-reply ghost-child reconcile (`stashChildrenReplaced` emit and
  the `std::erase_if` over unexpected fetch ids) → `LaunchContent` →
  `CheckUpdateFinished`.
- **Success terminal** (`FinishUpdate`,
  `itemsmanagerworker.cpp:1227-1281`): status emit →
  `RebaseItemLocations` (deferred to success by design; see the M1
  section of `items-pipeline.md`) → sort tabs → sort items → emit
  `ItemsRefreshed(m_items, m_tabs, false)` → Idle.
- **Failure terminal** (`AbortUpdate`, `itemsmanagerworker.cpp:525`):
  idempotent, returns the worker to Idle immediately, no
  `ItemsRefreshed`; stopped stragglers settle later and apply nothing
  (post-await invariant, network-redesign D6, W-IDENTITY).
- **Initial cached load** (`itemsmanagerworker.cpp:274`): the parse
  thread's result is published as one snapshot emit
  (`ItemsRefreshed(..., true)`), not as deltas.
- **ItemsManager** (`itemsmanager.cpp:121-147`): `OnItemsRefreshed`
  copies the vector, runs the debug uncategorized scan (F46), then
  `SetStashTabLocations` → `MigrateBuyouts` → `ApplyAutoTabBuyouts` →
  `ApplyAutoItemBuyouts` → `PropagateTabBuyouts` → re-emits
  `ItemsRefreshed(bool)`. `PropagateTabBuyouts` opens with the global
  `ClearRefreshLocks()` (`itemsmanager.cpp:93`).
- **Application** (`application.cpp:404-414`): currency snapshot, shop
  expiry, and (non-initial, auto-update on) forum shop submission on
  every `ItemsRefreshed`.
- **MainWindow** (`ui/mainwindow.cpp:1090-1104` and `736-775`):
  `OnItemsRefreshed` refilters every non-current search
  (`Search::FilterItems` — a whole-collection scan per search), then
  `ModelViewRefresh()` for the current search;
  `ItemsModel::beginUpdate()` is literally `beginResetModel()`
  (`items_model.h:28`), and the reset is followed by the restore
  machinery (`RestoreViewExpansion`, `ReselectCurrentItem`,
  `ScheduleResizeTreeColumns`).
- **Search dirty mechanism** (`search.h:82-85`): a filter state changed
  while a search is in the background sets `m_states_dirty` and forces
  a refilter when the search is next shown. D9 extends this pattern.
- **ItemLocation** (`itemlocation.h:53-62`): `fetch_id()` is the id of
  the stash/character actually fetched; for Map/Unique children it is
  the child's id while `id()` stays the parent (display) tab.
  Deliberately excluded from `operator==`, `operator<`, and
  `GetLegacyHash` — anything keyed on location equality collapses
  sibling child replacements.
- **FetchError taxonomy** (`ratelimit/fetcherror.h`): `Network`,
  `Http`, `Parse`, `Protocol`, `RateLimited`, `Internal`, `Canceled` —
  already the deterministic/transient vocabulary D5 needs.
- **F62 is decided but NOT implemented** (findings register): the
  persistence lane will carry raw wire bytes
  (`stashReceived`/`characterReceived` gain an opaque `QByteArray`).
  The code anchors above are pre-F62; D1 sequences it.

## Scope

M2 makes refresh progress visible incrementally without triggering the
snapshot cascade: a per-fetch-source delta signal from the worker, a
streamed published copy in `ItemsManager` with scoped pricing, a typed
terminal outcome event, and conservative, freshness-bounded delta
application in `MainWindow`. The final `ItemsRefreshed` snapshot
contract is unchanged (D8). Out of scope, each with reasons: per-tab
retry mechanism and durable resume, whole-update replacement/cancel,
reprioritization (D11); delta-native model operations (M3).

## Decisions

### D1. Sequencing: F62 lands before M2 implementation begins

The F62 fix (raw bytes through the persistence lane, decided July 26)
is implemented and merged **before** M2's code starts. This spec is
therefore written against the post-F62 worker: `stashReceived`/
`characterReceived` carry an opaque `QByteArray` alongside the typed
payload, and network-redesign D7 reads "nothing above the boundary
*interprets* bytes".

Reasons:

- F62 changes the signatures of exactly the signals M2's new delta
  signal sits next to, and the facade return types the worker's
  handlers consume. Landing it first means M2's wiring is written once,
  against the final shape of the persistence lane, instead of being
  rebased mid-milestone.
- D3's two-lane split (persistence vs. presentation) is only crisp once
  the persistence lane has its final payload: API-domain object plus
  wire bytes. Confirming the split against a lane that is about to
  change shape would be confirming a fiction.
- F62 is independently targeted at "before final 0.18" and is not
  blocked by anything in this spec; ordering it first costs nothing.

M2's deltas inherit F62's boundary rule as a constraint: **a delta
never carries wire bytes or `poe::*` API objects** — the persistence
lane owns those. Deltas carry pipeline-native `Items` only (D3).

### D2. The atomic unit is the fetch source, not the display tab

The delta's unit of atomicity is one fetch source — one (location type,
fetch-source id): a stash tab, a character, or one Map/Unique child.
The alternative (publish a Map/Unique parent once all its enabled
children settle) is rejected.

Reasons:

- **It is the unit the worker already has.** The atomic replace is
  keyed by `fetch_id()` (`RemoveItemsFetchedBy`); emitting after it
  requires zero new worker state. Per-display-tab publication would
  reintroduce per-parent deferred-completion accounting — a join
  counter per parent, a withheld-buffer for settled children, and a
  failure story for "child failed, parent never publishes" — which is
  precisely the apparatus the F55 revision deleted.
- **Coalescing gives the settling for free.** Downstream application is
  coalesced anyway (D9); children of one parent that arrive within a
  coalescing window collapse into one visible update in the common
  case, without any worker-side accounting.
- **Failure behavior stays simple.** Under no-rollback (D4), per-fetch-
  source publication keeps the published copy a pure function of
  applied replacements. A withheld parent bucket would make worker
  memory run ahead of published state by an unbounded amount on an
  update that fails mid-parent.

Accepted cost, stated deliberately: **a Map/Unique display tab can
transiently mix old and new child data** while its children stream in.
This is qualitatively the same partial-refresh mixing the whole
collection already exhibits between per-tab updates (some tabs fresh,
some stale), confined to one display tab; each fetch source is
internally consistent at all times. Recorded as an acceptance criterion
(`parentBucketMayMixChildGenerationsMidRefresh` — a documented-behavior
pin, not a bug tripwire).

### D3. The delta signal: `TabRefreshed(location, items)`

A new worker signal, working name kept from the plan:

```
void TabRefreshed(const ItemLocation &location, const Items &items);
```

- **Key.** `location.fetch_id()` keys the replacement;
  `location.id()`/`location.type()` name the display tab the UI should
  attribute the change to. Carrying the full `ItemLocation` covers both
  ids (they differ only for Map/Unique children) and gives empty deltas
  a display anchor. Consumers must key application on
  (type, fetch id) — never on location equality, which deliberately
  ignores `fetch_id` (staleness preamble).
- **Payload.** The complete pipeline-native `Items` replacement for
  that fetch source — the exact set of `Item` objects the worker just
  parsed and appended, sharing the same `shared_ptr`s. An **empty
  `items` means an emptied fetch source, never a deletion** — tab
  deletion is a list-reconciliation effect and stays snapshot-boundary
  (D6). No wire bytes, no `poe::*` objects (D1).
- **Emit point.** In `OnStashReceived`/`OnCharacterReceived`,
  immediately after the atomic replace (`RemoveItemsFetchedBy` +
  `ParseItems`) and before the counter increment and
  `CheckUpdateFinished` — so every delta of an update precedes its
  terminal event (D4) by construction.
- **Ghost-child drops stream as empty deltas.** The parent-reply
  reconcile that erases items fetched from children the parent no
  longer lists (`itemsmanagerworker.cpp:1007-1019`) emits one empty
  `TabRefreshed` per dropped fetch id (display location: the parent).
  This expresses the multi-fetch-id mutation exactly in the delta
  vocabulary, keeps the published copy a pure function of deltas (D6),
  and needs no new signal shape.
- **The cached initial load does not stream.** `OnParseCompleted`
  publishes one `ItemsRefreshed(..., true)` snapshot as today: it is a
  single in-memory atomic load with nothing incremental about it.
- **Two-lane split, confirmed.** Persistence signals
  (`stashReceived`/`characterReceived` + list signals) carry API-domain
  payloads plus, post-F62, wire bytes, and fire **before** the atomic
  replace; the presentation delta carries pipeline-native `Items` and
  fires **after** it. The lanes never share payload types — that is the
  drift guard: any future field that tempts one lane to import the
  other's types is a spec change, not a convenience.

`ItemsManager` applies each delta to its published copy — erase items
whose (type, fetch id) match, append the delta's items — runs the
scoped pricing pass (D7), and re-emits a light signal with the same
shape for the UI:

```
void TabRefreshed(const ItemLocation &location, const Items &items);
```

No whole-collection work runs on this path (hard constraint;
acceptance criterion below). The debug uncategorized-items scan (F46)
stays on the final snapshot only.

### D4. A typed terminal event, and no rollback

A new worker signal, forwarded by `ItemsManager`:

```
struct RefreshOutcome {
    enum class Result { Completed, Failed };
    Result result;
    // Fetch sources skipped by D5's skip-and-continue, with the
    // failure kind for each; empty on a clean completion. Meaningful
    // for Completed.
    std::vector<SkippedTab> skipped;
    // The terminal error; meaningful for Failed.
    RateLimit::FetchError error;
};
void RefreshFinished(const RefreshOutcome &outcome);
```

- Emitted **exactly once per accepted `Update()`**: from `FinishUpdate`
  (after `ItemsRefreshed`, so data precedes lifecycle) and from the
  first `AbortUpdate` (later straggler/second-failure calls already
  return early and must not emit it twice).
- **Ordering invariant:** all `TabRefreshed` emits of an update precede
  its `RefreshFinished`; nothing of that update follows it. A delta
  arriving after a `RefreshFinished` belongs to a later update. This
  holds by construction: emits are synchronous in reply handlers, a
  failure branch aborts before any delta of that reply, and stopped
  stragglers never reach their handlers (post-await invariant). No
  update-id field is added to the signals — the ordering guarantee is
  the identity, matching the worker's token-is-identity design
  (network-redesign D6); pinned instead of parameterized.
- **Overlap tolerance:** the first terminal failure idles the worker
  immediately, so update N+1's deltas may interleave with update N's
  *settling* stragglers — but stragglers emit nothing, so consumers
  observe a clean `… deltas(N), RefreshFinished(N), deltas(N+1) …`
  sequence. Consumers must not assume the worker is quiescent after a
  `Failed` outcome; they only get the sequence guarantee.
- **No-rollback policy, stated explicitly:** deltas applied before a
  later terminal failure stay applied — in worker memory, in the
  published `ItemsManager` copy, in the datastore, and on screen
  (pinned: `appliedReplySurvivesLaterFailureInMemoryAndDatastore`; M2
  extends the pin to the published copy). A `Failed` outcome means "the
  refresh did not finish; everything you see is real but the update is
  incomplete" — the UI may say so (status bar text via existing
  `StatusUpdate` plus this typed event for anything that needs to
  branch), but nothing is undone. `StatusUpdate` remains cosmetic and
  is demoted from any semantic role (it never had a contractual one;
  now the typed event exists, nothing may grow one).

This is the concrete resolution of the emit-on-failure non-goal's
"revisit at M2", together with D5.

### D5. Per-tab failure policy: deterministic failures skip, transient failures stay terminal

Content-fetch failures (stash or character fetches; **not** list
fetches) are classified by `FetchError::Kind`:

- **`Parse` — skip and continue.** A payload that fails the facade's
  parse is deterministic: retrying is futile, and one bad tab must not
  abort an hours-long update (the 3.29 `flags: []` incident aborted
  whole updates on exactly this). The handler's failure branch, for
  `Parse` on a content fetch only: log, record the fetch source in the
  update's skipped list, count the fetch as received (the counter join
  must still reconcile; monotonicity preserved, P-STATUS), emit **no**
  delta — the atomic replace never ran, so the fetch source's previous
  items remain, in memory and datastore, exactly per M1's
  non-destructive semantics. The update completes; the terminal event
  reports `Completed` with the skipped list (D4), and the final
  `ItemsRefreshed` publishes as usual (the skipped tab's list metadata
  is still upserted; its contents are old — the same listed-but-cold
  state F55-revised already defines).
- **Everything else — unchanged: first failure is terminal.**
  `Network`, `Http`, `RateLimited`, `Protocol`, `Internal` keep the
  M1/network-redesign semantics (stop token, `AbortUpdate`, no final
  emit). Reasons: transient transport failures are exactly what a
  retry mechanism should absorb, and that mechanism is deferred (D11)
  — skipping instead of retrying would churn through a dead network
  for hours marking every tab skipped; `Protocol` is systemic (the API
  changed under us — continuing risks violating limits); `Http` mixes
  per-tab cases (404) with systemic ones (401/403) and is deferred to
  the retry design rather than split speculatively; `Internal` is a
  bug. List-fetch failures of any kind stay terminal — the update
  cannot even define its batches without lists.

The scope here is deliberately minimal: one kind, on one fetch class,
with the proven incident behind it. Extending skip-and-continue (or
retry) to other kinds is the deferred per-tab-retry design's decision
(D11), and it inherits D4's outcome vocabulary ready-made.

### D6. The published-state contract: what streams and what stays snapshot-boundary

`ItemsManager`'s published copy is, at every instant during an update:

> the pre-update snapshot, with each applied per-fetch-source
> replacement (including empty ones) substituted in, in arrival order.

Exhaustively, by worker mutation:

| Worker mutation | Published when |
|---|---|
| Content replacement (atomic replace, incl. emptied source) | **Streamed** — `TabRefreshed` (D3) |
| Ghost-child drop on parent reply | **Streamed** — empty deltas (D3) |
| List reconciliation: tab metadata upsert (renames/moves/colors, F15) | Snapshot-boundary — final `ItemsRefreshed` |
| List reconciliation: deleted tabs dropped with their items | Snapshot-boundary — final `ItemsRefreshed` |
| New-tab discovery (listed, unfetched) | Snapshot-boundary — final `ItemsRefreshed` |
| `RebaseItemLocations` of surviving items | Snapshot-boundary — success only (below) |
| Tab/item sorting | Snapshot-boundary — final `ItemsRefreshed` |

- **The rebase point stays `FinishUpdate` (success only).** This
  resolves the plan's fourth hard constraint by choosing its first
  branch: per-delta consumers tolerate stale embedded tab metadata on
  *surviving* (unrefetched) items until the final emit. The
  failed-update-mutates-published-state problem M1 solved does not
  return. What makes this tolerable: streamed items are **new** `Item`
  objects built from the update's reconciled (fresh) tab list — the
  delta lane never mutates a shared published object; only items
  untouched by the update keep old metadata, which is exactly today's
  between-refreshes reality. Mid-refresh, the UI may therefore show a
  mix of old and new tab labels across buckets — accepted; M2 renders
  nothing new that juxtaposes one tab's old and new metadata.
- **Tab-list publication stays final-only.** `SetStashTabLocations`,
  and thus tab-level buyout keys and shop tab indexing, update at the
  final emit as today. Consequence: a delta may arrive for a display
  tab absent from the published tab list (a newly discovered tab in a
  full refresh). Buckets are built from item locations, so the bucket
  appears with the delta's (fresh) location; tab-level buyouts for it
  default to none until the final pass. Accepted.
- **Ordering of the published vector is unspecified mid-refresh.**
  Erase+append leaves the vector unsorted between deltas; the final
  emit restores the deterministic sorted order. No consumer may assume
  sorted published items mid-refresh (today's consumers of mid-refresh
  state: none; D9's refilter does not care; shop reads final-only per
  D8).

### D7. Per-delta scoped pricing: yes — item-local, final pass stays authoritative

Streamed items visibly unpriced for hours are a real cost at the
hours-long-refresh scale, and pricing the delta is O(delta items) with
no whole-collection component — so M2 prices per delta, scoped:

On each applied delta, `ItemsManager` runs, **for the delta's items and
display tab only**:

1. Tab-name auto-buyout for the delta's display location
   (`StringToBuyout(tab_label)` → `SetTab`), mirroring
   `ApplyAutoTabBuyouts` for one tab.
2. Note-based item buyouts for the delta's items, mirroring
   `ApplyAutoItemBuyouts`'s per-item rule (including its
   clear-when-stale branch).
3. Tab-inheritance propagation for the delta's items, mirroring
   `PropagateTabBuyouts`'s per-item rule.

Explicitly excluded from the scoped pass, because each is
whole-collection or global-state semantics: `MigrateBuyouts`,
`CompressTabBuyouts`, **`ClearRefreshLocks` and all refresh-lock
bookkeeping** (locks are recomputed by the final pass exactly as
today), and any `Save()` scheduling beyond what `BuyoutManager`'s
existing dirty flag already does.

The safety property that makes this cheap to reason about: **the final
whole-collection pass at `ItemsRefreshed` is unchanged and remains
authoritative** — any divergence a scoped pass could introduce is
overwritten within the same update on success. The scoped pass is an
anticipation of the final pass, never a replacement. Pinned:
`scopedPricingConvergesToFinalPass` — scoped passes followed by the
final pass produce `BuyoutManager` state identical to the final pass
alone, modulo `last_update` timestamps.

Known transient, accepted: a renamed tab's streamed items key their tab
buyout lookup by fresh metadata while the published tab-buyout table
still holds the old key (D6) — such items may show as unpriced/inherit
until the final pass heals them.

### D8. The final-emit contract is unchanged

The worker's `ItemsRefreshed(m_items, m_tabs, initial_refresh)` and
everything downstream of it keep today's semantics and ordering:
`ItemsManager`'s copy + migration + three whole-collection buyout
passes + light re-emit; `Application`'s currency snapshot, shop expiry,
and forum submission; `MainWindow`'s full refilter of every search.
Specifically, restating the plan's hard constraints as design:

- **No per-delta forum submission** — `Shop` remains connected to the
  final `ItemsRefreshed` only, and never to `TabRefreshed`.
- **No per-delta whole-collection scans** — the delta path in
  `ItemsManager` (D3, D7) and `MainWindow` (D9) touches O(delta) items
  plus, at most, one coalesced current-search refilter per freshness
  window (D9's explicitly budgeted exception, removed in M3).
- **No per-delta uncoalesced model reset** — every model reset on the
  delta path goes through D9's throttle; `TabRefreshed` never reaches
  `beginResetModel()` directly.

M3 renegotiates the final-emit cascade; M2 does not.

### D9. UI application: intersection-gated, throttled, with a stated freshness bound

`MainWindow` consumes `ItemsManager::TabRefreshed` in two tiers:

- **Background tier (delta does not intersect the current search's
  visible result).** No model touch of any kind — scroll, selection,
  and expansion survive trivially. Every non-current search is marked
  **items-dirty** (a new flag beside `m_states_dirty`, using the same
  refilter-on-next-activation mechanism, `search.h:82-85`).
  Background tab captions are **not** recomputed per delta (each
  caption requires that search's whole-collection refilter — exactly
  the cascade M2 exists to avoid); they refresh when the search is
  activated or at the final emit, as today. Accepted cost: background
  captions may be stale during a long refresh.
- **Foreground tier (delta intersects the current search).** The delta
  is added to a pending set and published by a **non-resetting
  trailing throttle with period S** (the freshness bound): when the
  timer fires, the current search refilters, the model resets once,
  and the existing restore machinery runs. The timer is started by the
  first pending delta and is **not** re-armed by later arrivals — that
  is the anti-starvation half: under steady one-reply-per-20-seconds
  arrivals a resetting debounce would starve forever; this throttle
  guarantees the visible view is never more than S behind the applied
  state, and resets at most once per S. **Provisional S = 60
  seconds**, chosen to dominate the ~20 s/tab arrival cadence by a
  small integer factor; the exact value is a spike question (below),
  not an argued constant.

Intersection is decided on the delta alone, O(delta items): a delta
intersects the current search iff any of its items matches the current
filter set, **or** any item currently in the visible filtered result
was fetched from the delta's fetch id (the removal half — an empty or
shrunken replacement must count as a visible change). The mechanism
for the removal test (e.g., a fetch-id set maintained per refilter) is
implementation detail; both halves are the requirement.

The final `ItemsRefreshed` cancels any pending throttle tick and runs
the existing full path. A user-initiated refilter (search form change,
tab switch) also flushes the pending set — the user just paid for a
refilter; folding pending deltas into it is free.

**Spike, not paper (S1-M2):** whether one reset-plus-restore per
S = 60 s under the user's feet is acceptable steady-state UX cannot be
settled by argument. Before freeze-or-implement of this D's constants:
drive a scaled refresh (harness or live) with the throttle prototyped,
and judge scroll/selection/expansion survival by hand. Outcomes: (a)
acceptable → S is confirmed or tuned; (b) not acceptable → the
foreground tier degrades to "current search updates only at final emit
or on user action" for M2 (the freshness bound then applies only to
intersection *detection* — e.g. a "view is behind, N tabs updated"
affordance — and true in-place freshness waits for M3's bucket-scoped
model ops). Either outcome ships M2; the spike only picks between two
already-specified behaviors. The restore machinery itself (F23, F31,
F32) is reused as-is either way and is retired by M3, not extended by
M2.

### D10. Status-burst coalescing: measurement-gated

Network-redesign D6 left the batch-submit `QueueUpdated` burst
(thousands of synchronous emissions in one loop turn at the 2,000-tab
scale) to "coalesce on the UI side if the status dialog measurably
stutters". M2 keeps that as a **measurement gate, not a decision**:
instrument a 2,000-entry submission (the offline harness can drive it)
and measure status-widget frame time. If it stutters, the fix is a
UI-side coalesce of the existing signal (same throttle pattern as D9,
much smaller S); the limiter is not touched either way
(network-redesign is frozen). If it does not stutter, nothing is
built. Recorded as measurement M1-M2 in the open-items list.

### D11. Deferrals, each with its reason

- **Per-tab retry mechanism and durable resume.** D5 took the
  classification decision (the load-bearing part per the inbox) and D4
  built the outcome vocabulary a retry design needs; the mechanism —
  where bounded retries live without violating the frozen network
  spec's retry semantics, which `Http` kinds join the skip set, and
  resume-what-wasn't-fetched across aborts/restarts — is its own
  design. It is not required to ship streaming, it touches the frozen
  network layer, and datastore per-tab persistence already gives
  restart durability of applied content. Deferred to a follow-on spec;
  nothing in M2 forecloses it.
- **Whole-update replacement/coalescing and user cancel.** An
  `Update()` during an active update stays refused (network-redesign
  D6's update-state policy). Streaming actually sharpens the future
  design: once progress is visible and durable, "cancel" is just
  request_stop with a `Failed{Canceled}`-flavored outcome and
  everything applied stays — but the semantics of *replace* (which
  selection wins, what happens to in-flight batches) expand M2
  considerably, exactly as the plan warned. Deferred; D4's outcome
  event is the hook it will need.
- **Reprioritization.** Deferred with its network-redesign rationale
  intact (D6/R7: the stop token is per-update; per-entry cancellation
  does not exist and would need a mechanism deliberately not designed
  now). Nothing in M2 makes it harder; D2 keeps the fetch source as
  the natural unit a future prioritizer would reorder.
- **Guardian/skills ingestion (F63)** stays deferred by its own
  decision; noted here only because a future ingestion adds fetch
  sources — the delta vocabulary (D2/D3) absorbs new sources without
  spec change.

## Acceptance criteria

Worker-level (offline fake-network harness, extending the M1 suite):

- `deltaMatchesAppliedReplacement` — every accepted content reply
  emits exactly one `TabRefreshed` whose key is the reply's
  (type, fetch id) and whose items are exactly the applied
  replacement, emitted after the replace and before
  `CheckUpdateFinished`.
- `emptyDeltaEmptiesFetchSourceOnly` — an empty replacement empties
  that fetch source in the published copy and removes nothing else;
  no tab disappears from the published tab list mid-refresh.
- `ghostChildDropStreamsAsEmptyDelta` — a parent reply that stops
  listing a child yields an empty delta for the dropped child's fetch
  id, and the published parent bucket loses exactly those items.
- `deltasNeverFollowTerminalEvent` — per update: all deltas precede
  `RefreshFinished`; after a `Failed` outcome, settling stragglers
  emit nothing (extends W-IDENTITY to the delta signal).
- `terminalEventExactlyOncePerUpdate` — one `RefreshFinished` per
  accepted `Update()`, on both success and first failure; second
  failures and stragglers emit no second event.
- `parseFailureSkipsTabAndUpdateCompletes` — a deterministic `Parse`
  failure on one content fetch: no delta for that source, its previous
  items survive in memory and datastore, counters reconcile, the
  update completes, and the outcome lists the skipped source. A
  `Network` failure on the same fixture stays terminal.
- `publishedStateIsSnapshotPlusAppliedDeltas` — at any point
  mid-update, `ItemsManager::items()` equals the pre-update snapshot
  with applied replacements substituted; after a mid-update failure it
  stays there (extends
  `appliedReplySurvivesLaterFailureInMemoryAndDatastore` to the
  published copy).
- `scopedPricingConvergesToFinalPass` — scoped per-delta pricing
  followed by the final pass yields `BuyoutManager` state identical to
  the final pass alone, modulo `last_update` timestamps; no scoped
  pass ever calls `ClearRefreshLocks` or mutates lock state.
- `parentBucketMayMixChildGenerationsMidRefresh` — documented-behavior
  pin for D2's accepted transient, so the mixing is asserted
  deliberate rather than rediscovered as a bug.

Design-review criteria (checked in review, not runnable):

- No `TabRefreshed` connection reaches `Shop`, forum submission,
  currency, `MigrateBuyouts`, or any whole-collection buyout pass.
- The delta path performs O(delta) work everywhere except D9's single
  budgeted coalesced refilter.
- The persistence and presentation lanes share no payload types.
- A background-tab delta performs no model operation (scroll/selection
  /expansion survival by construction); UI-level verification is part
  of the S1-M2 spike checklist.

## Open items requiring spike or measurement (not argument)

- **S1-M2 (spike, blocks D9's constants only):** live/harness trial of
  the S = 60 s reset-plus-restore cadence; picks between D9's two
  specified foreground behaviors and tunes S.
- **M1-M2 (measurement, blocks nothing):** 2,000-entry `QueueUpdated`
  burst vs. status-widget frame time; builds the D10 coalesce only if
  it stutters.

## Input traceability

| Inbox item / constraint (`items-pipeline.md`) | Consumed by |
|---|---|
| Hard constraint: no per-delta forum submission | D8 |
| Hard constraint: no per-delta whole-collection scans | D8, D7, D9 |
| Hard constraint: no per-delta uncoalesced model reset | D8, D9 |
| Hard constraint: rebase does not compose with streaming | D6 (rebase stays at success-only `FinishUpdate`) |
| Persistence signal surface partly exists; confirm the split | D3 (two-lane split + drift guard) |
| D6 deferrals: whole-update replacement, reprioritization, per-tab retry/durable progress, `QueueUpdated` coalescing | D5 (classification decided), D10 (measurement-gated), D11 (reasoned deferrals) |
| Overlap: new update active while stragglers settle | D4 (ordering invariant + overlap tolerance) |
| F62 decided-not-implemented; deltas never carry bytes/API objects | D1 (sequencing + inherited constraint), D3 |
| Delta carries Items keyed by (type, fetch-source id); atomic-unit choice | D3 (shape), D2 (unit: fetch source) |
| Typed terminal event + explicit no-rollback policy | D4 |
| Buyout scoping per delta; mind `ClearRefreshLocks` | D7 |
| Persistence/presentation split; deleted-tab expression | D6 (table; deletion is snapshot-boundary; empty delta ≠ deletion) |
| Freshness bound, not just coalescing | D9 (non-resetting throttle, S, both halves stated) |

Every inbox item is consumed; the emit-on-failure non-goal's "revisit
at M2" is resolved by D4+D5.
