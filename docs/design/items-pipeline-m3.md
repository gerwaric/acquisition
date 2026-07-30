# Items Pipeline Milestone 3: Delta-Native Items Model

Status: **DRAFT, revision 1** (July 30, 2026). Not reviewed, not
frozen; production implementation must not begin against this
document. Unlike M2, the pre-spec evidence is already in hand: the
parent plan's "profile before choosing levers" obligation was
discharged by the S1-M3 sort-profiling spike (July 30, 2026, branch
`spike/m3-sort-profile`, evidence in `m3-sort-profile-result.md`
beside this spec), and the lever hold point ran the same day. **At
that hold point Tom selected: levers A and B both committed (D1, D2),
the intended `(name, uid, hash)` tie-break order (D5, fixing F67 as a
side effect), and the key-memory measurement run pre-freeze rather
than deferred** — it ran the same day as a spike extension (~286
B/item accounted; ~266 MB naive at 1m; section 4 of the spike
result). This spec consumes the M3 inbox in `items-pipeline.md`
("Inputs accumulated for the M3 spec") and the spike's evidence; the
traceability table at the end maps every input to the decision,
deferral, or acceptance criterion that consumed it.

Citation convention: bare D-numbers (D1, D2, …) in this document are
this document's decisions; M2's decisions are cited qualified ("M2
D9"), as are the network redesign's. F-numbers are the findings
register (`docs/cleanup/findings.md`); `R1-*` (and later `R2-*`, …)
are this spec's review-round findings, recorded with verdicts and
resolutions in `items-pipeline-m3-reviews.md` from round 1; pinned
test names are quoted in `camelCase`. "The spike" unqualified means
S1-M3.

## Staleness preamble

Written against commit `3549b214` (master, July 30, 2026 — the M2
merge, PR #185). The load-bearing code assumptions; re-verify these
anchors before implementing against any section, and re-verify the
whole list if the model layer or the delta path has been touched
since:

- **Delta application (M2 D9)**: `MainWindow::OnTabRefreshed` /
  `OnChildrenReconciled` (`ui/mainwindow.cpp:709-736`) mark every
  search items-dirty (rule 1), intersection-gate against the current
  search, and call `ScheduleThrottledRefilter()`
  (`ui/mainwindow.cpp:756`) — a non-resetting trailing throttle with
  period S = 60 s (`m_delta_throttle`). `OnDeltaThrottleTimeout` →
  `ModelViewRefresh` (`ui/mainwindow.cpp:828`), which captures
  expansion and scroll, refilters, resets the model, and restores
  (`RestoreViewExpansion`, `ReselectCurrentItem`, the scroll anchor).
- **The reset**: `ItemsModel::beginUpdate()` is literally
  `beginResetModel()` (`items_model.h:28`). One sort column and order
  per model (`m_sort_column`/`m_sort_order`; the pinned default is
  column 0 = Name, descending) with a lazy `m_sorted` flag:
  `Search::FilterItems` ends with `SetSorted(false)` and the sort
  actually runs on activation (`search.cpp:461-466`) via
  `ItemsModel::sort` → `Search::Sort` (`search.cpp:201`) →
  `Bucket::Sort` (`bucket.cpp:41`), a per-bucket `std::sort` calling
  `Column::lt` per comparison.
- **Comparator surface**: 26 columns (`search.cpp:33-59`), three
  comparator implementations — the `Column::lt`/`Column::multivalue`
  base (`column.cpp:28-67`; two static `QRegularExpression`s, string
  fallback path), and the `PriceColumn::lt`/`DateColumn::lt`
  overrides, which read `BuyoutManager` per comparison. Ties fall
  through to `Item::operator<` (`item.cpp`), whose hash term is dead
  code (F67).
- **Search state rebuilt by every refilter** (`search.h`):
  `m_bucket_by_tab`/`m_bucket_by_item` (`ViewMode::ByTab`/`ByItem`),
  expansion keyed by stable `(type, id)` (M2 R6-3,
  `m_expanded_keys`), the reselection index `m_visible_by_id`, the
  intersection sets `m_visible_sources`(`_by_tab`), and the scroll
  anchor. `defaultExpanded()` is `m_filtered || ByItem`
  (`search.h:86`). Buckets hold `std::shared_ptr<Item>` vectors;
  per-tab caps bound a display bucket at 576 items (quad tab).
- **The M2 signal surface is settled ground**: worker
  `TabRefreshed(location, items)` and
  `ChildrenReconciled(parent, expected)` deltas, the typed
  `RefreshFinished` terminal, the unchanged final `ItemsRefreshed`
  snapshot (M2 D8), scoped per-delta pricing in `ItemsManager` (M2
  D7), and `SourceKeyedItems` on both worker and manager (the M2-M2
  remedy pair). M3 changes none of it; this spec touches only Layer 3
  (search/model/view).
- **Initial cached load**: the parse thread publishes one snapshot
  (`ItemsRefreshed(..., true)`); the first population of a search's
  model has no view state to preserve.

## Scope

M3 makes Layer 3 consume deltas natively and retires the two costs
the evidence localized: the destructive whole-model reset on the
refresh path, and the comparator-bound re-sort. Concretely: a tab
delta becomes bucket-scoped model operations instead of a throttled
reset (D3); sorting runs on precomputed cached keys (D1, hold-point
lever A) and only for buckets whose order is visible (D2, hold-point
lever B); the By-Item flat bucket gets a per-delta sorted merge (D4);
the tie-break becomes the intended `(name, uid, hash)` order, fixing
F67 (D5). The user-initiated full refilter survives as a
non-refresh-driven path and gets cheap by the same levers (D6).

Out of scope, each with reasons: the filter loop, born-sorted
masters, regex micro-optimization as a lever, F66's legacy-store
rekeying, key persistence (D7); pricing semantics (settled, M2 D7 —
M3's concern is model operations, not pricing).

## Decisions

### D1. The central lever: precomputed, cached sort keys (hold point, lever A)

Sorting never calls `Column::lt` per comparison. Each materialized
bucket (D2) carries a key vector parallel to its items; `Bucket::Sort`
orders `(key, item)` pairs with plain tuple comparison and the bucket
adopts the resulting item order.

**Key shape.** The key is the comparator's own tuple, materialized
once per item — the comparator remains the single source of ordering
truth, and keys are derived from it, never a second definition:

- Base columns: the `multivalue` tuple `(d1, s1, d2, s2)` exactly as
  `Column::multivalue` computes it (`column.cpp:28-61`), plus the
  tie-break suffix.
- `PriceColumn`: `(currency rank, value)` plus the suffix.
- `DateColumn`: `(last_update)` plus the suffix.
- The tie-break suffix is `(PrettyName, uid, hash)` — the intended
  order per D5.

The two identical `PrettyName` strings on the base string path (`s2`
and the suffix's first element) share one buffer; `uid` and `hash`
are CoW copies of the `Item`'s own members and cost nothing while the
item lives. Both facts are measured (spike section 4).

**Lifetime and invalidation contract.** Keys are a cache of
`(item content, active sort column, buyout state for Price/Date)`:

1. Keys are built lazily, per bucket, the first time that bucket
   sorts (composing with D2: collapsed buckets hold no keys). Build
   cost is O(bucket) — measured 33 ms per 100k items, 368 ms per 1m,
   so a 576-item bucket keys in well under a millisecond.
2. A delta that replaces a bucket's items discards that bucket's
   keys; they rebuild at the bucket's next sort (for a materialized
   bucket, immediately — still O(delta)).
3. Switching the active sort column discards all key vectors
   (order-only changes — ascending/descending — do not; the
   comparison flips, the keys stand). The rebuild is user-initiated
   and O(materialized set); the whole-collection worst case is
   ~0.37 s at 1m, paid once per column switch.
4. When and only when the active column is Price or Date, every
   `BuyoutManager` mutation path (item set/clear, tab set/clear, the
   scoped and final pricing passes) marks dependent buckets'
   key vectors stale. The implementation must enumerate these paths
   at a single choke point; a design-review criterion checks the
   enumeration, and `priceKeysFollowBuyoutEdits` pins the observable
   behavior. Other columns' keys are indifferent to buyout state by
   construction.

**Memory.** Measured (spike section 4): ~286 B/item accounted for the
string-heavy worst case. Under D2's lazy per-bucket build the
resident set is proportional to the materialized set — a collapsed
default view holds approximately no keys. The documented worst case
is the By-Item flat bucket at 1m with a base column active: ~266 MB
naive, ~222 MB with the required `s2`/suffix sharing (D4 accepts
this; the budget is pinned in the acceptance criteria).

Reasons:

- The spike proved the sort comparator-bound and the comparator
  regex-bound (~3/4 of every call); keys retire the whole per-
  comparison toll: 40× on cached keys, 10× even rebuilding every
  refilter, and `std::sort` machinery is ~12–13 ns/compare.
- Keys are the only measured lever that helps every structure,
  including the By-Item flat bucket (12.9 s → ~0.77 s at 1m even
  rebuilding; D4 does better by merging).
- The cache contract fits the delta shape: deltas arrive per fetch
  source, buckets key per display tab, so invalidation is naturally
  bucket-scoped and O(delta).

### D2. Deferred sorting: only visible order is paid for (hold point, lever B)

A bucket's item order is established only when that order can be
seen. Each bucket carries a sorted flag (per search, per model):

1. A **collapsed** By-Tab bucket is never sorted; it keeps arrival
   order. Unsorted is not unstable: the order is deterministic until
   a delta or refilter changes the bucket's contents, so persistent
   indexes and model invariants hold.
2. Expanding a bucket sorts it first (building its keys per D1 if
   absent), then the expansion proceeds. Cost is bounded by the
   576-item bucket cap: ~3 ms even with the *current* comparator,
   microseconds keyed — imperceptible against the expand paint
   itself. The same rule serves `RestoreViewExpansion`: restoring N
   expanded buckets sorts exactly those N.
3. Filtered searches are default-expanded (`search.h:86`), so every
   visible bucket of a filtered result sorts eagerly — acceptable
   because a filtered result is small by construction; the expensive
   case (whole collection, unfiltered) is exactly the case that
   starts fully collapsed.
4. The By-Item flat bucket is always visible and therefore always
   sorted (D4); lever B deliberately does not apply to it.
5. Sort-column header clicks re-sort **materialized** buckets only
   (per D1 rule 3) via the existing `layoutChanged` path; collapsed
   buckets simply drop their sorted flag.

Reasons:

- Measured: sorting only the expanded set costs 15–63 ms independent
  of collection size, and the default unfiltered view starts fully
  collapsed — the whole-model sort at first paint disappears rather
  than being merely cheapened.
- The hold point weighed B's bounded marginal value post-A (~0.13 s
  of a ~0.55 s filter-bound refilter at 1m) against its bookkeeping
  and committed it anyway, buying the collection-size-independent
  first paint and the lazy key memory profile (D1) — B is what makes
  "approximately no keys resident by default" true.
- The lazy machinery extends an existing pattern (`m_sorted` +
  sort-on-activation, `search.cpp:461-466`) one level down, from
  model to bucket, rather than inventing a new one.

### D3. Refresh-driven model changes are bucket-scoped operations; the reset is retired from the refresh path

The delta path stops resetting the model. When a delta intersects the
current search (the M2 D9 intersection machinery, unchanged), it is
applied as fine-grained operations scoped to the affected bucket:

- **Content replacement** (`TabRefreshed`): one bucket-scoped
  replace — remove that bucket's filtered rows, insert the arriving
  items that pass the active filters, sorted if the bucket is
  materialized (D2), arrival-ordered if collapsed. Item-level
  diffing is deliberately rejected: M2 D2/D3 define the delta as a
  whole-fetch-source replacement, so a one-bucket replace is the
  delta's native grain.
- **Child reconciliation** (`ChildrenReconciled`): the erase becomes
  row removals scoped to the parent's bucket.
- **New tab discovered**: the bucket row is inserted at its display
  position (`begin/endInsertRows` at top level). **Deletion stays a
  snapshot-boundary effect** (M2 D6): an empty delta empties a
  bucket, it never removes one.
- **Metadata changes** (rename, move, color): the bucket row updates
  in place (`dataChanged`) and repositions with `beginMoveRows` when
  display ordering changes; expansion and selection follow the
  stable `(type, id)` key exactly as in M2 R6-3 — the machinery the
  parent plan carries forward as M3's bucket keying.
- **Search-side indexes are maintained incrementally**: the delta
  updates `m_visible_by_id`, `m_visible_sources`(`_by_tab`), and the
  bucket in O(delta); no whole-collection rebuild rides along. The
  full rebuild remains the refilter's job (D6).

**The D9 throttle is retired for the current search.** Its reason to
exist — reset-plus-restore too expensive to pay per delta — is gone:
a bucket-scoped application is O(delta) work (filter the arriving
items with `MatchesActiveFilters`, keyed sort of ≤ 576 items, row
ops). Deltas apply as they arrive; the freshness bound M2 D9 stated
becomes trivially "immediate", and M2's hard constraint ("no
per-delta uncoalesced model reset") is honored by having no reset at
all. Rule 1 (every delta marks every search items-dirty) and
dirty-on-activation for background searches are unchanged.

**The final snapshot stops being a destructive event for the open
window.** On `ItemsRefreshed` after a refresh, the current search —
already delta-fresh — performs one non-destructive reconciliation:
bucket metadata refresh against the rebased locations
(`RebaseItemLocations` runs at the success terminal, a
snapshot-boundary effect per M2 D6) and bucket reordering via move
ops. No `beginResetModel`. Background searches keep the existing
flag-and-refilter-on-activation path. A full refresh with the window
open is then, as the parent plan requires, just N deltas plus one
cheap reconciliation — no special destructive path left.

**Where resets remain legitimate**: initial population (nothing to
preserve) and user-initiated structural changes — filter edits, view-
mode switches, search switches — where the R6-3 capture/restore
machinery already covers fidelity (D6). Never on the refresh path;
`noModelResetDuringRefresh` pins this.

Reasons:

- The spike killed the alternative framing: begin/endResetModel are
  microseconds, so the reset was never the cost — but the *restore*
  obligations it creates (recapture, refilter, re-sort, reselect) are
  the whole cascade. Scoped ops make the untouched state simply never
  move, which is the milestone's success criterion.
- Per-delta cost is bounded and small (O(delta) + a ≤ 576-item keyed
  sort), so no coalescing layer is needed; deleting the throttle
  removes a whole class of staleness/starvation reasoning (M2 D9
  spent most of its length on it).
- Rejecting item-level diffs keeps the operation count per delta at
  O(1) row-op batches, which is what keeps persistent-index handling
  tractable (`modelTesterPassesUnderDeltaStorm`).

### D4. The By-Item flat bucket: per-delta sorted merge

The By-Item view (single flat bucket holding the whole filtered
result) is the one structure that fights the delta shape, and lever B
cannot help it. It gets its own contract:

1. The flat bucket is always materialized: keys resident (D1), order
   maintained.
2. A delta applies as: remove the source's rows (erase by
   `FetchSourceKey`, contiguous-run `removeRows` batches), then merge
   the arriving filtered items — sort the ≤ 576 arrivals by key,
   then a single merge pass against the resident order, inserted as
   contiguous-run `insertRows` batches. Work is O(n + d) per delta
   (the vector shuffle dominates; at 1m that is tens of
   milliseconds), model ops are O(runs), and the result must equal a
   full keyed re-sort (`byItemMergeMatchesFullSort`).
3. The memory worst case is accepted and documented: ~222–266 MB
   resident keys at 1m on a base column (D1). The user paying it
   today pays 12.9 s per refilter for the same view; the trade is
   deliberate. The budget is pinned (≤ 300 MB at 1m, acceptance
   criteria).
4. A full By-Item refilter (user-initiated) is the keyed build + flat
   sort: measured ~0.77 s at 1m against 12.9 s today.

Reason: the parent plan names this structure explicitly ("the one
structure that fights the delta shape") and the spike quantified both
the threat (12.9 s, immune to lever B) and the remedy's ceiling. An
O(n) merge pass per delta is the honest cost of a single sorted
container; pretending a flat bucket can take O(d) updates would be
designing against arithmetic.

### D5. The tie-break becomes the intended `(name, uid, hash)` order (hold point; fixes F67)

`Item::operator<`'s third tuple element becomes `rhs.m_hash` (the
one-token F67 fix), restoring the intended hash-level determinism for
id-less items, and D1's key suffix carries the hash so keyed and
comparator order agree everywhere —
`keyedOrderMatchesComparatorOrder` is the equivalence pin and
`intendedTieBreakRestored` the determinism pin. The only visible
change is that items with no server id that tie on name now order
deterministically instead of arbitrarily. F67's register entry is
moved to the resolved ledger when this lands.

Reason: the hold point chose intended over actual order — determinism
for the price of one CoW QString per key (measured free while the
item lives), versus preserving an ordering accident nobody depends
on.

### D6. The user-initiated refilter: kept, and cheap by the same levers

Filter edits, view-mode switches, and search switches keep today's
shape — capture, `FilterItems`, reset, restore (R6-3 machinery
unchanged) — because a changed filter genuinely invalidates the whole
derived view. What changes is the price: the post-reset sort
disappears into D1/D2 (collapsed buckets sort nothing; restored
expansions sort their N buckets keyed; By-Item pays the keyed flat
sort). The refilter's residual cost is then the filter loop itself —
~34 ms at 100k, ~391 ms at 1m, now the dominant term. **Optimizing
the filter loop is explicitly out of scope** (D7) but the fact is
recorded: post-M3, "refilter cost" means "filter-loop cost".

Reason: the refresh path (D3) and the user-refilter path have
different invalidation semantics — a delta invalidates one bucket, a
filter edit invalidates everything — and forcing them through one
mechanism would either reintroduce whole-view work per delta or
incremental-filter complexity M3 does not need. The levers make the
honest full rebuild affordable.

### D7. Deferrals, each with its reason

- **Filter-loop optimization.** Post-M3 the dominant refilter term
  (~0.4 s at 1m). Deferred: it is orthogonal to the delta-native
  model and the sort levers, and it needs its own profile-first pass
  (per-filter attribution) before choosing a lever. Routed to the
  parent plan as a follow-up input, not silently dropped.
- **Born-sorted master buckets.** Never prototyped; its marginal win
  over cached keys is bounded by the numbers at ~0.13 s per 1m
  refilter, against pipeline-wide sortedness maintenance and a
  master-per-sort-order problem. Reopen only if D1's invalidation
  proves intractable in practice.
- **Regex micro-optimization as a lever.** Bounded strictly below A
  (the toString/PrettyName floor leaves ~1.5–2.5 s at 1m) and
  unprototyped. Permitted silently as an implementation detail
  *inside* the key build (D1 rule 3's 0.37 s column-switch rebuild is
  ~90% multivalue machinery), forbidden as a substitute for keys.
- **F66 legacy-store rekeying.** The register names M3 "the natural
  opportunity" if the stores are reworked — M3 reworks the model
  layer, not the legacy persisted stores, so the hook stays parked in
  the register. Nothing in this spec touches location-keyed
  persistence.
- **Key persistence across sessions.** Keys rebuild from live items
  in well under a second at any realistic scale; persisting a cache
  of derived strings buys nothing and adds an invalidation surface to
  the datastore. Rejected, not deferred.
- **`QueueUpdated` coalescing (M1-M2)** stays a post-M2 follow-up
  where M2 left it; nothing here changes its standing.

## Acceptance criteria

Model-level (Qt Test, against the `MainWindow` fixture and a direct
`Search`/`ItemsModel` harness):

- `unrelatedDeltaLeavesOtherBucketsUntouched` — the milestone's
  success criterion as a test: refresh one tab; every other bucket's
  expansion, selection, scroll anchor, and persistent indexes are
  bit-identical, and no restore machinery runs (probe: capture/
  restore entry counts stay zero).
- `noModelResetDuringRefresh` — a complete refresh (N deltas, child
  reconciliations, terminal, final snapshot) with the window open
  emits zero `begin`/`endResetModel` on the current search's model;
  the final snapshot performs only the D3 reconciliation
  (metadata `dataChanged` + `beginMoveRows` repositioning).
- `deltaReplacesExactlyItsBucketRows` — a content delta's row
  operations touch exactly the affected bucket; row accounting,
  filtered membership, and (for a materialized bucket) keyed order
  all correct after application.
- `emptyDeltaEmptiesBucketWithoutRemovingIt` — the M2 D6 boundary at
  the model layer: an empty replacement leaves an empty bucket row
  (unfiltered search), never a bucket removal.
- `deltaUpdatesVisibleIndexesIncrementally` — after a delta,
  `m_visible_by_id`, `m_visible_sources`, and `HasVisibleGhostUnder`
  answer as if freshly refiltered, with no whole-collection rebuild
  (probe: rebuild counters).
- `bucketRepositionsByMoveOnMetadataDelta` — a rename/move delta
  repositions the bucket via move operations; expansion and selection
  follow the stable `(type, id)` key (extends M2 R6-3's pins from
  reset-restore to move).
- `modelTesterPassesUnderDeltaStorm` — `QAbstractItemModelTester`
  attached through a randomized storm of deltas (content, empty,
  reconciliation, metadata, new-tab) interleaved with expansion
  changes, sort clicks, and view-mode switches.

Sort-correctness:

- `keyedOrderMatchesComparatorOrder` — for every column: the keyed
  sort of a mixed dataset (double path, range paths, string path,
  heavy ties, id-less items) is identical to a direct `Column::lt`
  sort with the D5-fixed comparator, both directions.
- `intendedTieBreakRestored` — id-less items tying on `PrettyName`
  order deterministically by hash across repeated sorts and merges
  (F67 resolved).
- `collapsedBucketsDeferSorting` — an unfiltered By-Tab refilter
  sorts no collapsed bucket and builds no keys for one (probe:
  key-build and compare counters at zero); expanding one bucket sorts
  exactly that bucket, correctly.
- `restoredExpansionSortsRestoredBucketsOnly` — a user refilter with
  N saved expansions sorts exactly those N buckets on restore.
- `filteredSearchSortsAllVisibleBuckets` — a filtered
  (default-expanded) search presents every bucket sorted.
- `keyCacheInvalidatedByDelta` — a delta replacing a materialized
  bucket's items yields fresh keyed order immediately; stale keys
  never order fresh items (probe: key rebuild counted for that bucket
  alone).
- `sortColumnSwitchRebuildsMaterializedKeysOnly` — switching the
  active column rebuilds keys for materialized buckets only;
  flipping only the direction rebuilds none.
- `priceKeysFollowBuyoutEdits` — with Price active, a user buyout
  edit (item and tab level, set and clear) reorders affected rows;
  Date symmetric; with Name active the same edits rebuild nothing.

By-Item:

- `byItemMergeMatchesFullSort` — after any sequence of deltas, the
  flat bucket's order equals a from-scratch keyed sort of the same
  filtered collection.
- `byItemRemovalOnlyDeltaErasesInPlace` — an empty replacement
  removes exactly the source's rows via row operations; no reset, no
  full re-sort.
- `byItemSelectionSurvivesMerge` — a selected item retains selection
  through a merge that moves its row; a removed item clears it
  (stable-identity reselection at merge grain).

Performance and memory (the M1-M3 checkpoint, Release, recorded
environment, spike presets; budgets are the spike's measured ceilings
with headroom, misses gate completion the way M2-M2's did):

- Worst-case unfiltered By-Tab refilter (user-initiated, default
  collapsed): ≤ 60 ms at 100k, ≤ 500 ms at 1m end-to-end
  (filter-loop-bound; the sort share ≤ 5 ms).
- Single-bucket expand (cold keys): ≤ 10 ms at both scales.
- By-Item full refilter: ≤ 250 ms at 100k, ≤ 1.5 s at 1m.
- Delta application on the current search, By-Tab materialized
  bucket: ≤ 5 ms at 1m; By-Item merge: ≤ 50 ms at 1m.
- Resident key memory, active column, By-Item at 1m: ≤ 300 MB;
  By-Tab default view: proportional to the materialized set
  (≈ 0 collapsed).

Design-review criteria (checked in review, not runnable):

- No refresh-driven path reaches `beginResetModel`; resets exist only
  on initial population and user-initiated structural changes (D3's
  enumerated list).
- The delta path is O(delta + affected bucket) everywhere except
  D4's stated O(n + d) By-Item merge pass; no whole-collection scan
  or rebuild rides on any delta (M2 hard constraint, extended to the
  model layer).
- Keys are derived from the comparators, which remain the single
  source of ordering truth; no code path defines order twice.
- D1 rule 4's `BuyoutManager` mutation enumeration is complete —
  every mutation path flows through the single choke point that
  marks Price/Date keys stale.
- M2's intersection, dirty-flag, and background-search machinery is
  unchanged; the persistence and presentation lanes still share no
  payload types, and keys carry no wire or `poe::*` types.
- The retirement of the D9 throttle deletes its machinery and its
  pinned timer tests by renegotiation, not silent breakage — the M2
  pins that encode "throttled reset" behavior are explicitly
  superseded here, each mapped to its D3-era replacement in the
  implementation plan.

## Open items requiring spike or measurement (not argument)

- **S1-M3 (spike, pre-spec — RESOLVED before this draft):** the
  sort-profiling spike, run July 30, 2026 on `spike/m3-sort-profile`
  (never merged; off master `3549b214`). Proved the sort
  comparator-bound and the comparator regex-bound; quantified levers
  A (~10–40×), B (proportional to expanded set), and the By-Item
  worst case; measured the key-memory extension at the hold point
  (~286 B/item; ~266 MB naive at 1m). Evidence:
  `m3-sort-profile-result.md`. Consumed by D1–D6 and the hold-point
  record.
- **M1-M3 (measurement, implementation checkpoint — blocks M3
  completion):** the performance/memory budget table above, run on
  the implemented model with the spike presets and a recorded
  environment, plus per-component attribution (filter loop, key
  build, sort, model ops, merge) so a miss names its component
  before a remedy is chosen — the M2-M2 discipline. Delta-application
  scenarios use fixed recorded reply shapes as M2-M2's did.

## Input traceability

| Input (parent plan M3 section, spike, register, hold point) | Consumed by |
|---|---|
| Bucket architecture matches the delta shape; tab delta → fine-grained ops or one-bucket replace | D3 (one-bucket replace chosen) |
| By-Item single flat bucket needs a sorted merge per delta | D4 |
| Pricing semantics settled by M2 D7; M3 inherits | Scope (out), staleness preamble |
| Success criterion: refreshing one tab leaves everything else untouched; full refresh = N deltas, no destructive path | D3, `unrelatedDeltaLeavesOtherBucketsUntouched`, `noModelResetDuringRefresh` |
| Inbox: reset cost is the re-sort; user refilter still pays at scale; levers to weigh (keys / expanded-only / born-sorted); profile before choosing | S1-M3 ran (profile obligation); D1, D2 (chosen levers), D7 (born-sorted deferred) |
| Inbox: R6-3 fidelity machinery negligible; carries forward as M3's bucket keying | D3 (metadata/move rule), D6 (restore path kept) |
| Spike 1: comparator regex-bound; any lever skipping `multivalue` per comparison captures the win; regex-only variant unprototyped | D1, D7 (regex micro-opt bounded and demoted) |
| Spike 2: levers compose (cost per comparison × number of comparisons), ceilings 10–40× and proportional-to-expanded | D1 + D2 committed together (hold point) |
| Spike 3: By-Item flat bucket worst by far, immune to lever B; must be answered | D4 |
| Spike 4: begin/end reset ~free; filter loop is the next term once sort stops dominating | D3 (reset retired as *path*, not cost), D6, D7 (filter loop routed as follow-up) |
| Spike section 4: key memory ~286 B/item, ~266 MB naive at 1m; uid/hash CoW free; s2 dedupe saves ~44 MB | D1 (memory rules), D4 rule 3, memory budget |
| F67: dead hash tie-break; keyed design must choose intended vs actual order | D5 (intended; hold point) |
| F66: M3 named as natural opportunity if stores reworked | D7 (explicitly not exercised; hook stays) |
| Hold point (Tom, July 30, 2026): levers A+B committed; intended order; memory measured pre-freeze | D1, D2, D5; spike section 4 |
| M2 hard constraint: no per-delta uncoalesced model reset; freshness bound | D3 (no reset at all; throttle retired, bound trivially met) |

Every named input is consumed. Review rounds begin at round 1 in
`items-pipeline-m3-reviews.md`, which also carries the revision log.
