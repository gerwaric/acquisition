# M1-M3 measurement result (items-pipeline M3, S7 gate)

Status: **IN PROGRESS — created at S0 (July 30, 2026) with the anchor
verification the implementation sequence requires there.** The budget
table itself runs at S7 (Release, recorded environment, spike presets,
per-component attribution), with conditional hold-point rows at S3-S5;
those runs extend this document as they happen. Until S7 passes, M3 is
not complete.

## S0 — staleness-preamble anchor verification (July 30, 2026)

The M3 spec's staleness preamble was written against master `3549b214`
(the M2 merge). This verification ran against post-merge master
`a2ed4d96` (the spec merge, PR #186 — no model-layer or delta-path code
changed between the two), immediately before S0's first implementation
commit.

**Verdict: every load-bearing claim holds. One citation was wrong at
freeze; no code drifted.** The drift record:

- **`search.cpp:461-466` does not exist** — the file is 444 lines at
  both `3549b214` and `a2ed4d96` (451 after S0's own probe edits), so
  the range was invalid when the spec froze (the claim appears twice:
  the staleness preamble's "sort actually runs on activation" and D2's
  "existing pattern" reason). The *claim* is correct; its actual sites
  are:
  `Search::FilterItems` ends by invalidating the model's sorted flag
  (`m_model.SetSorted(false)`, `search.cpp:352`), and the sort runs on
  activation via `MainWindow::ModelViewRefresh`'s
  `setSortingEnabled(true)` (`ui/mainwindow.cpp:854`) → the header's
  sort-indicator path → `ItemsModel::sort` (`items_model.cpp:220`,
  lazy `m_sorted` gate at `:227`) → `Search::Sort` (`search.cpp:200`)
  → `Bucket::Sort` (`bucket.cpp:42`). `Search::SetViewMode`
  (`search.cpp:435-444`) is the other sort-on-activation site.

Every other anchor verified at (or within a line of) its citation:

| Preamble claim | Cited | Verified at `a2ed4d96` |
|---|---|---|
| `OnTabRefreshed` / `OnChildrenReconciled`: rule-1 dirty marking, intersection gate, `ScheduleThrottledRefilter()` | `ui/mainwindow.cpp:709-736` | `:708-737` |
| Non-resetting trailing throttle, S = 60 s, `m_delta_throttle` | `ui/mainwindow.cpp:756` | `ScheduleThrottledRefilter` `:755-765`; `DELTA_THROTTLE_INTERVAL_MS = 60 * 1000` `:71`, single-shot `:138` |
| `OnDeltaThrottleTimeout` → `ModelViewRefresh` (capture, refilter, reset, restore) | `ui/mainwindow.cpp:828` | `ModelViewRefresh` `:827` (timeout handler `:767`); capture `:842-844`, restore `:855/:876/:877` |
| `beginUpdate()` is literally `beginResetModel()` | `items_model.h:28` | exact (moved by S0's own probe edit, behavior identical) |
| One sort column/order per model; pinned default column 0 = Name, descending; lazy `m_sorted` | — | `items_model.cpp:21-23` |
| 26 columns | `search.cpp:33-59` | `:32-58`, 26 entries |
| `Column::lt`/`multivalue`, two static regexes, string fallback | `column.cpp:28-67` | `multivalue` `:27-57`, `lt` `:59-62` |
| `PriceColumn::lt` / `DateColumn::lt` read `BuyoutManager` per comparison | — | `:422-436` / `:453-458` |
| `Item::operator<` hash term dead (F67) | `item.cpp` | `:667-672`; `:671` compares `m_hash` on both sides |
| Search state: `m_bucket_by_tab`/`m_bucket_by_item`, `m_expanded_keys`, `m_visible_by_id`, `m_visible_sources`(`_by_tab`), scroll anchor | `search.h` | members present (`:155-160`, `ScrollAnchor` `:59`) |
| `defaultExpanded()` is `m_filtered \|\| ByItem` | `search.h:86` | exact |
| M2 signal surface: `TabRefreshed`, `ChildrenReconciled`, typed `RefreshFinished`, final `ItemsRefreshed`, `SourceKeyedItems` both sides | — | `itemsmanager.h:74-79/:98`, `itemsmanagerworker.h:115-151/:328` |
| Scoped per-delta pricing in `ItemsManager` (M2 D7) | — | `OnTabRefreshed` `itemsmanager.cpp:160` calls `ApplyScopedPricing` `:176` (defined `:202`) |
| Buyout mutation surface: `OnBuyoutChange` `SetTab`/`Set` per selected row, trailing `PropagateTabBuyouts`; `MigrateItem`; snapshot pricing sequence | `ui/mainwindow.cpp:541-567/:578`, `buyoutmanager.cpp:370`, `itemsmanager.cpp:152-155` | `:541-577` (`Set` `:567`) / `:578`; `:370` exact; `:152-155` exact |
| Per-tab caps bound a display bucket at 576 items (quad tab) | — | structural, not an enforced cap in model code: a display bucket aggregates one tab, and a quad stash is 24×24 = 576 slots (`QuadStash` handling `itemlocation.cpp:102-106`; the dataset generator encodes the same cap, `tests/spikedataset.h`) |
| Initial cached load: the parse thread publishes one snapshot (`ItemsRefreshed(..., true)`) | — | `StartParseThread` runs `ParseCachedItems` on a dedicated `QThread` (`itemsmanagerworker.cpp:119-140/:142`); completion emits `ItemsRefreshed(..., true)` `:274`; `ItemsManager` re-emits `itemsmanager.cpp:125/:157` |

## S0 — probe surface and datasets

- **Probes** (`src/modelprobes.h`): comparator-call (all three `lt`
  implementations), bucket-sort (attributed by the stable `(type, id)`
  display key), model-reset (attributed per model), refilter,
  index-rebuild, and capture/restore entry counters, wired at their
  production sites; key-build, keyed-compare, model-update (batch),
  and live-resident-key-bytes fields declared with their landing
  stages (S1/S2/S3) so pins assert them from zero. **Disabled by
  default**: production takes one predicted branch per site and
  accumulates no state, and measurement windows stay unperturbed
  unless a row needs attribution; tests enable explicitly.
  `reset()` clears counters but preserves the `live_key_bytes` gauge
  (it tracks live ownership; zeroing it mid-residency would go
  negative at the next eviction). Nothing logs, nothing in production
  reads them. Pinned by `probeCountersTrackRefilterAndSort`
  (tst_search) and `probeCountersTrackCaptureRestore`
  (tst_mainwindow).
- **Datasets** (`tests/spikedataset.h`): the recorded M2-M2/spike
  shapes are now named presets — `smoke` (the S1-M2 harness's
  recorded 50 tabs / mean 20, ~1k items, functional runs), `100k`
  (2000 tabs / mean 50 / quad 0.10, ~101k published), `1m`
  (2600 / 400 / 0.80, ~976k published), all seed 20260729. Pinned by
  `namedPresetsMatchRecordedShapes`; `m2m2_benchmark` consumes the
  named presets. The M2-M2 caveat carries: reruns against numbers
  recorded before the July 30 10-char stash-id change are not
  byte-identical in dataset.

## S3 — conditional hold-point rows (July 30, 2026): **PASS, all rows**

Run at the end of S3 (D2 flags + D1 residency landed; branch
`items-pipeline-m3`) with the new `m3_holdpoint_benchmark` harness
(`tests/m3_holdpoint_benchmark.cpp`, Release, offscreen, spike presets,
seed 20260729). Environment matches the M2-M2 record: Apple M4, 32 GB
(macMini-class), macOS 26.6, Apple clang 21, Qt 6.11.1 release. The
sequence continues to S4 — no remedy decision needed.

| Row | 100k | 1m | Budget (100k / 1m) | Verdict |
|---|---|---|---|---|
| Worst-case unfiltered By-Tab refilter, default collapsed (median of 5, end-to-end `OnSearchFormChange`) | 39.1 ms | 418.3 ms | ≤ 60 ms / ≤ 500 ms | **PASS** |
| — sort share of the above | 0 ms | 0 ms | ≤ 5 ms | **PASS** (probe-attributed: 0 bucket sorts, 0 key builds, 0 keyed compares — the toll disappears, not merely cheapens) |
| Single-bucket expand, cold keys (576-item quad, median of 5) | 0.55 ms | 0.59 ms | ≤ 10 ms | **PASS** |
| Broad-filter default-expanded refilter (ilvl ≥ 2, ~99% match, every bucket visible — R1-8) | 94.9 ms | 980.8 ms | ≤ 150 ms / ≤ 1.2 s | **PASS** |
| Collapsed-default resident key memory | 0 B | 0 B | ≈ 0 | **PASS** (gauge exactly 0) |
| Background-search resident key memory after deactivation | 0 B | 0 B | exactly 0 | **PASS** (gauge exactly 0; was > 0 while active) |

Per-component attribution (M2-M2 discipline, micros on identical data
outside the timed windows):

- Unfiltered refilter is filter-loop-bound as designed: bare
  `FilterItems` micro 35.5 ms @100k / 464.8 ms @1m of the 39.1 / 418.3
  end-to-end; the remainder is the model reset + restore machinery.
- Broad refilter @1m: 471.4 ms filter loop + the eager key build/sort
  of ~everything (982.0 ms measured as one flat-bucket build+sort micro,
  an upper bound on the per-bucket work, which the live window's
  980.8 ms total confirms is heavily amortized across 2600 small
  sorts — 2600 key builds, 9.7M keyed compares, zero comparator calls).
- Broad-filter resident key footprint by the gauge: 37.0 MB @100k,
  356.7 MB @1m. The gauge deliberately over-estimates (fixed 24-byte
  header per string; the `uid`/`hash` CoW copies counted although they
  cost ~nothing while their items live), so this is not comparable to
  the spec's ~222-266 MB process-level figure; the authoritative
  ≤ 300 MB aggregate row is measured at process level when By-Item
  lands (S5) and at S7.

**S3 review round 1 rerun (July 30, 2026).** The round's fixes changed
`Bucket::Sort`'s mechanics (the permutation now applies in place —
sorting no longer transiently duplicates the ~144 B/item key vector,
which at a one-million-item By-Item sort would have added ~144 MB of
untracked peak on top of the resident estimate) and scoped the buyout
batch's layout operation to the affected materialized set. All rows
rerun and PASS, slightly faster: unfiltered refilter 35.3 / 374.3 ms,
sort share 0, cold expand 0.53 / 0.55 ms, broad-filter 85.1 / 912.6 ms
(100k / 1m), memory rows exactly 0.

## S4 — conditional hold-point row (July 31, 2026): **PASS**

Run at the end of S4 (D3 bucket-scoped delta operations + selection
intent landed; By-Tab throttle retired) with the same harness, build,
environment, and presets as the S3 rows. The sequence continues to S5 —
no remedy decision needed.

| Row | 100k | 1m | Budget (1m) | Verdict |
|---|---|---|---|---|
| Delta application on the current search, By-Tab visible bucket (576-item full source replacement into the expanded quad bucket — removal runs plus a maximal merge; median of 5, end-to-end `OnTabRefreshed`) | 0.344 ms | 0.352 ms | ≤ 5 ms | **PASS** (100k informational — the spec states the budget at 1m) |

Attribution (probe-attributed, same run): exactly one bucket sort — the
R2-2 merge itself, counted as the bucket's order-refresh event — zero
key builds (the visible bucket's resident vector is reused; arrival
keys are built per item inside the merge), ~5.9k keyed compares, zero
index rebuilds, zero refilters, zero model resets. The cost is flat
across presets, as D3 predicts: the operation is O(delta + affected
bucket) and never sees collection scale.

The S3 rows were incidentally rerun on the S4 code and all still pass
(unfiltered refilter 32.1 / 361.0 ms, sort share 0, cold expand
0.52 / 0.56 ms, broad-filter 83.3 / 878.8 ms, memory rows exactly 0).

**S4 review round 1 correction and rerun (July 31, 2026).** The
review found the row above did not measure the shape it claimed: the
harness replaced an ordinary single-source tab's entire source, so the
retained vector was empty — one contiguous removal run and no merge
against retained sibling-source rows. That run validated arrival
sorting and simple replacement, not the R2-2 interleaved path. The
harness now aggregates a synthetic child fetch source (a same-sized
donor tab's items, fresh ids, re-homed under the largest quad tab)
into the expanded bucket and replaces the child per delta: removal
runs scatter through 576 retained parent rows and the merge's 576
arrivals interleave against them. Rerun after the round's fixes,
same build and environment:

| Row | 100k | 1m | Budget (1m) | Verdict |
|---|---|---|---|---|
| Delta application, By-Tab visible bucket — **R2-2 interleaved child-source replacement** (576 retained + 576 arrivals, median of 5) | 0.887 ms | 1.079 ms | ≤ 5 ms | **PASS** |
| — single-source full replacement (the previously recorded shape, kept informational) | 0.380 ms | 0.459 ms | — | informational |

The interleaved shape costs ~2.3× the simple replacement — the run
count is what scales, exactly why the review flagged the original row
— and stays flat across presets (O(delta + bucket), never collection
scale). Attribution: one bucket sort (the merge), ~11.5k keyed
compares, zero key builds (resident vector reused), zero index
rebuilds, zero refilters, zero resets. The S3 rows rerun with the
round's fixes and still pass (unfiltered refilter 36.0 / 374.6 ms,
sort share 0, cold expand 0.53 / 0.59 ms, broad-filter
87.1 / 917.9 ms, memory rows exactly 0).

## S5 — conditional hold-point rows (July 31, 2026): **MISS — sequence PAUSED** *(resolved the same day — the remedy subsection at the end of this section records the final PASS)*

Run at the end of S5 (D4 By-Item merge + eager activation landed; D9
throttle fully retired) with the same harness, build, environment, and
presets as the S3/S4 rows. Three of four budgeted rows pass; the
By-Item merge row misses by ~28× and, per working rule 3, **the
sequence is paused here** pending a remedy decision.

| Row | 100k | 1m | Budget (100k / 1m) | Verdict |
|---|---|---|---|---|
| By-Item full refilter (median of 3, end-to-end `OnSearchFormChange`) | 116.7 ms | 1471.5 ms | ≤ 250 ms / ≤ 1.5 s | **PASS** |
| Clean By-Item reactivation — eager key hydration, R3-1 (end-to-end tab switch; probe-attributed: 0 refilters, 1 key build, 0 resets) | 46.3 ms | 446.6 ms | ≤ 100 ms / ≤ 0.5 s | **PASS** |
| Worst-shape resident key memory (process footprint delta across entering By-Item; gauge 35.7 MB / 344.2 MB is the known over-estimate) | 21.8 MB | 223.9 MB | ≤ 300 MB aggregate at 1m | **PASS** |
| **By-Item merge** (D4 rule 2: interleaved child-source replacement, 576 retained-source rows scattered through the resident order + 576 arrivals, median of 5) | 168.0 ms | **1397.9 ms** | ≤ 50 ms at 1m | **MISS (~28×)** |

Mode switch into By-Item (flat rebuild + keyed flat sort, D6 boundary,
informational): 81.0 ms / 1059.9 ms.

**Attribution (M2-M2 discipline).** Probes on the live merge: exactly
one bucket sort (the merge itself), zero key builds (the eager-hydrated
resident vector is reused), ~17.7k keyed compares, zero index rebuilds,
zero refilters, zero resets — all of which is microseconds. The micro
on identical data settles where the time goes: the same flat delta
against a bare `Search` with **no view attached costs 1395.5 ms of the
1397.9 ms live row**. The cost is the bucket-vector shuffle itself, not
view-side batch handling:

- The child source's ~576 rows scatter through the collection-sized
  sorted order, so the erase is ~576 contiguous runs, and the merge's
  arrivals land as ~360 more insert runs.
- Qt's `begin/endRemoveRows` contract requires the model to be
  consistent after **every** run, so each run's `vector::erase`/
  `insert` must complete before the next begins — and each one moves
  the tail of a ~1m-element vector, twice over (the item vector and
  the index-aligned resident key vector, whose `ItemSortKey` elements
  are string-heavy composites). Total work is O(runs · n), ~10⁸-scale
  element moves per delta at 1m.

*(The analysis and remedy options below are superseded in part by the
S5 review round 1 subsection that follows — the "mutually
unsatisfiable" claim was too strong, and remedy A as stated violates
Qt's model contract.)*

**The two halves of D4 rule 2 are mutually unsatisfiable at collection
scale.** The rule's mechanism ("contiguous-run `removeRows` batches …
inserted as contiguous-run `insertRows` batches") forces O(runs · n);
the rule's complexity claim and budget ("a single merge pass … O(n + d)
per delta … at 1m that is tens of milliseconds", ≤ 50 ms) describe a
single compaction+merge pass, which Qt permits only under **one** model
operation per delta (the bucket-scoped layout-change protocol
`ApplySort` already uses, with persistent-index remapping) — not under
per-run row-op batches. R2-2 corrected the batch count from O(1) to
O(runs) without anyone noticing that per-run consistency multiplies the
run count into the vector length. The S4 By-Tab row never sees this
because its buckets are capped at 576 rows; the flat bucket is the one
structure where runs and n are both large.

Remedy options (Tom decides; nothing is built on top of the miss):

- **A (recommended): post-freeze amendment to D4 rule 2's mechanism.**
  The flat bucket applies a delta under a single bucket-scoped layout
  operation — snapshot persistent indexes, one O(n + d) erase+merge
  pass over the item and key vectors, remap (removed rows map to
  invalid), `layoutChanged` scoped to the flat bucket. Model ops become
  O(1) per delta (stronger than the stated O(runs)); the complexity
  claim, the ≤ 50 ms budget, selection/intent behavior, and
  `byItemMergeMatchesFullSort` are unchanged.
  `byItemRemovalOnlyDeltaErasesInPlace`'s row-op signal assertions
  would be renegotiated to layout-op assertions (same observable
  contract: no reset, no full re-sort, order preserved).
- **B: keep the row-op mechanism and renegotiate the budget** to the
  measured ~1.4 s per delta at 1m — rejected on its face by D3's
  immediacy rationale (a delta arrives every ~20 s per tab; 1.4 s of
  synchronous UI work per delta makes By-Item unusable during a
  refresh at scale).
- **C: hybrid** — row ops below a flat-bucket size threshold, layout op
  above it. Adds a second code path and still needs A's amendment; only
  worth it if the row-op signals carry value at small scale that the
  layout op lacks.

The S3/S4 rows were rerun on the S5 code and all still pass (unfiltered
refilter 33.3 / 426.1 ms, sort share 0, cold expand 0.52 / 0.59 ms,
broad-filter 88.7 / 981.7 ms, S4 interleaved delta 0.85 / 0.92 ms,
memory rows exactly 0).

**S5 review round 1 (July 31, 2026 — Tom's review of the pause
record, four findings, all verified and accepted).** The findings and
what each changed:

1. *S5 misses its performance contract* — accepted; this is the
   recorded miss and the pause above. No change.
2. *Remedy A is invalid* — accepted, verified against the Qt 6.11.1
   docs: `VerticalSortHint` "carr[ies] the meaning that items are …
   not filtered out or in", and rows appearing or disappearing outside
   `begin/endInsertRows`/`begin/endRemoveRows` is outside the model
   contract; a pure layout operation covers equal-cardinality
   permutation only. The recommendation above is withdrawn (marker
   added). The round also corrects this section's **"mutually
   unsatisfiable" claim, which was too strong**: Qt's contract binds
   the model's *observable answers* at each notification boundary, not
   its physical representation — the O(runs · n) cost came from this
   implementation equating per-run notification with per-run
   `vector::erase`/`insert`. A mutate-once/notify-per-run
   implementation (option A′ below) satisfies D4's stated mechanism
   *and* its complexity claim; the spec is not internally
   inconsistent, the implementation was naive.
3. *The benchmark omits production-path pricing* — accepted; the
   harness gained manager-path rows (`ItemsManager::OnTabRefreshed`
   end to end: source-keyed replacement, inventory ingest, scoped
   pricing, then the window's merge), with priced arrivals carrying a
   fresh `~b/o N chaos` note per rep so every rep's 576 `Set`s are
   real state changes. Measured (same build/environment):

   | Shape | 100k | 1m |
   |---|---|---|
   | Manager path, unpriced arrivals, Name active | 167.1 ms | 1414.9 ms |
   | — same, with the view's row list laid out first | 167.3 ms | 1410.6 ms |
   | Manager path, priced arrivals, Name active | 4634.7 ms | **43,982.3 ms** |
   | Manager path, priced arrivals, Price active | 98.1 ms | 532.5 ms |

   Attribution: the unpriced manager path costs the same as the
   window-direct row (the pricing pass no-ops when no buyout state
   changes), so the recorded 1397.9 ms was the honest number for
   unpriced deltas — but a **lower bound** overall, exactly as the
   finding said. The priced-Name blowup is NOT the pricing writes
   (576 sqlite upserts are milliseconds) and NOT the merge: it is the
   **rule-5 repaint's single spanning `dataChanged` rectangle**.
   `RepaintBuyoutCells` emits one first-to-last rectangle per bucket;
   in By-Item under a non-buyout sort the affected ids scatter, so the
   span approaches the whole flat bucket, and Qt's per-row handling
   costs ~44 µs/row — a constant that reproduces across all three
   scales (smoke ~53 ms/1.2k rows, 100k ~4.4 s, 1m ~43 s). A new
   sub-finding, S2-era in origin: the one-rectangle strategy is
   O(span), invisible in By-Tab where buckets cap at 576 rows.
   Priced-Price at 532 ms is the R3-2 batch re-sort (~2.0M keyed
   compares) plus a *narrow* repaint band and a nearly run-free merge
   (same-price arrivals cluster under the Price order).
4. *Retired throttle leaves source-index machinery unconsumed* —
   accepted, and stronger than stated: `HasVisibleSource` and
   `HasVisibleGhostUnder` now have **zero callers anywhere**, tests
   included; `m_visible_sources`/`m_visible_sources_by_tab` are
   write-only production state maintained per delta. S6's
   reconciliation does not need them as specced. Disposition is Tom's
   call with the remedy: delete under an explicit amendment (the
   acceptance text's D9-intersection references are M2 history), or
   carry to S8's design-review pass.

**Corrected remedy options** (superseding the list above; Tom
decides):

- **A′ — mutate once, notify per run (translation shim).** Compute
  the final item and key vectors in one O(n + d) pass; emit the same
  contiguous-run `removeRows`/`insertRows` batches D4 rule 2 states,
  answering row queries during the notification window through a
  prefix-offset translation over the old/final vectors, and swap the
  final vectors in after the last batch. Keeps the frozen mechanism
  verbatim — no amendment — with standard signals
  (`modelTesterPassesUnderDeltaStorm`-safe). Work
  O(n + d + runs·log runs). Caveat: the batch count stays O(runs), so
  any per-batch view-side cost survives; measured nil offscreen, and
  the "laid out" probe row above was inconclusive because the harness
  window is never shown — if chosen, an on-screen spot-check is part
  of the remedy's definition of done.
- **A″ — cardinality adjustment + layout remap** (the review's
  sketch): one tail `insertRows`/`removeRows` to establish the new
  row count, then a bucket-scoped layout operation remapping
  persistent indexes. O(1) model operations per delta. Needs a
  post-freeze amendment to D4 rule 2's mechanism, and it is
  contract-gray: the layout step still replaces content in and out,
  which the sort hints explicitly exclude and `NoLayoutChangeHint`
  leaves undocumented; `QAbstractItemModelTester` under the S6 storm
  would adjudicate it in practice.
- **B — renegotiate the budget** to the measured cost: unchanged from
  above, rejected on its face by D3's immediacy rationale.
- **Regardless of the merge remedy, the repaint span needs its own
  fix**: rule 5 mandates that affected cells repaint, not that one
  rectangle cover them — emitting one rectangle per affected
  contiguous run (O(runs), like the row ops) removes the O(span)
  term. Without it, even a 50 ms merge leaves a priced delta at ~43 s
  at 1m under a non-buyout sort. Implementation-level, no amendment
  required; folded into the remedy stage if approved.

**S5 remedy (July 31, 2026): implemented, all rows PASS — the pause
is lifted and the sequence resumes at S6.** Tom's decision on the
round-1 options: **A′** for the merge (the mechanism stays; A″ remains
a fallback experiment only if on-screen signal handling had missed),
the repaint-run fix approved, and the dead source indexes deleted now
under an explicit amendment. Three commits, in the ordered sequence
Tom prescribed:

1. `594b2b1a` — the spec's second post-freeze amendment: the D9
   intersection sets (`m_visible_sources`, `m_visible_sources_by_tab`,
   `HasVisibleSource`, `HasVisibleGhostUnder`) deleted; their final
   consumer disappeared with the throttle, and the intersection
   contract's semantics are executed by application itself.
   `deltaUpdatesVisibleIndexesIncrementally` narrowed.
2. `b6442903` — rule-5 repaint scoped to affected runs: O(affected
   runs) rectangles, By-Item tab-level changes resolve to the tab's
   rows by item location, `everything` keeps the single full
   rectangle. New pin `buyoutRepaintScopesToAffectedRuns`.
3. `a737ad3a` — **A′**: `Bucket::ReplaceSourceRows` — one O(n + d)
   rebuild (removal-run scan; arrival sort + insertion runs against
   the compacted retained keys; removals notified back-to-front on
   the intact old vector, the final vector then built by MOVING
   retained rows, insertions notified forward on it; final keys
   realized in place by a backward run-replay merge — no second
   collection-sized key buffer). D4 rule 2's stated row-op batches
   are emitted unchanged; a row translation keeps the model's
   observable answers consistent at every notification boundary.
   Gate pin `byItemReplaceSatisfiesModelTester` (shrink / equal /
   grow / empty / removal-only under `QAbstractItemModelTester`,
   selection + persistent index surviving, end state equal to a
   from-scratch refilter).

Rerun of the full S5 hold point (same build discipline, environment,
presets):

| Row | 100k | 1m | Budget (1m) | Verdict |
|---|---|---|---|---|
| **By-Item merge** (child-source replacement) | 4.0 ms | **32.4 ms** | ≤ 50 ms | **PASS** (was 168.0 / 1397.9 ms — 43× at 1m) |
| — window shown and laid out (A′ view-overhead gate) | 4.2 ms | 32.2 ms | ≤ 50 ms | **PASS** (fully on-screen variant, `QT_QPA_PLATFORM=cocoa` on the live desktop: 34.8 ms — PASS; per-batch view overhead is ~2.5 ms over the bare rebuild) |
| — bare micro, no view attached | 4.4 ms | 32.9 ms | — | attribution: the cost IS the single-pass rebuild |
| Manager path, unpriced arrivals, Name | 5.1 ms | 33.0 ms | — | informational (pricing pass no-ops) |
| Manager path, priced arrivals, Name | 74.7 ms | 310.4 ms | — | informational (was 4634.7 / 43,982.3 ms); residual = 576 buyout upserts + the O(n) affected-row scan + per-run rectangles |
| Manager path, priced arrivals, Price | 103.3 ms | 545.0 ms | — | informational; dominated by the **mandated R3-2 batch re-sort** (~2.0M keyed compares) plus a near-run-free merge — NOT closed by the repaint fix, recorded separately |

Peak-memory gate (Tom's A′ definition of done): lifetime peak
(`ru_maxrss`) across the shown-window merge reps +3.9 MB at 100k (the
final-vector scale) and **+0.0 MB at 1m** — no key-vector-sized
transient; the resident-key gauge and process-footprint rows are
unchanged from the round-1 record. Attribution probes unchanged in
shape: one bucket sort (the merge), zero key builds, zero index
rebuilds, zero refilters, zero resets. All S3/S4/S5 budgeted rows
rerun and PASS (unfiltered refilter 25.3 / 248.6 ms, broad-filter
79.1 / 754.1 ms, S4 interleaved delta 0.88 / 0.86 ms, By-Item
refilter 107.4 / 1267.0 ms, reactivation 44.8 / 455.2 ms — the
across-the-board improvement over the round-1 numbers is partly the
repaint fix and partly run-to-run environment variance; the budget
verdicts are what is load-bearing).

## Budget table (S7 — to be run)

The acceptance-criteria table from `items-pipeline-m3.md` runs here
when the model is complete; conditional hold-point rows are recorded
per stage (S3, S4, S5) as the sequence reaches them.
