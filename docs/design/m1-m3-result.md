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

## Budget table (S7 — to be run)

The acceptance-criteria table from `items-pipeline-m3.md` runs here
when the model is complete; conditional hold-point rows are recorded
per stage (S3, S4, S5) as the sequence reaches them.
