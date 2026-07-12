# Phase 6: Opportunistic MainWindow Slimming and Remainders

> **Status.** Revised to implementation grade in July 2026, verified against
> the post-Phase-5 codebase (branch `design-cleanup` at commit a69369a, after
> PR #155), then self-reviewed adversarially. This phase is still a curated
> list of remainders, not one coherent refactor — items ship as independent
> PRs — but the items are now ordered, and the list was re-cut after
> verification (6.4 dropped; 6.7 and 6.8 added). If implementation
> starts after further changes to `MainWindow::OnTabChange` /
> `ModelViewRefresh` / `OnLayoutChanged`, `SearchForm`'s bind/unbind,
> `BuyoutManager::Set`/`SetTab`, `LogPanel`, `currencymanager.h`, or
> `shop.cpp`'s error paths, re-verify the matching section first (plan
> working rule 5).

## Goal

Clean up what the earlier phases deliberately deferred, without introducing
a controller architecture. Findings addressed: F14, F32, F37, F9 remainder,
F20 (scoped down), plus F39 and F40 (found during this spec revision) and
optionally F34. F22 is documented rather than unified (6.4, dropped).

## Explicit non-goals, restated

- **No `MainController`/MVVM extraction.** Without a second UI consumer (the
  deferred QML work), a full controller layer is structure without payoff.
  "Slimming" means: move non-UI logic out when an item below touches it
  anyway; nothing more. This applies to 6.7 with force: making `MainWindow`
  testable is achieved with a test fixture, not by carving interfaces out of
  `MainWindow` (see D1).
- **No items-pipeline work.** GitHub issue #156 (items disappearing from the
  in-memory list after a partial refresh) is recorded under F28 and is a live
  bug in the worker/refresh path, not a Phase 6 item. Nothing here touches
  `ItemsManagerWorker`.
- F38 (the "Influenced" filter also matches fractured/synthesised items)
  stays as recorded; its current behavior is pinned by
  `tst_filters::booleanPredicates`.
- No persistence of searches, no datastore rework beyond relocating the
  `CurrencyUpdate` struct declaration (6.2), no rate-limiter changes.

## Verified current state (July 2026)

What Phases 0–5 actually left behind, including corrections to the
design-intent version of this doc:

- **F14 is intact and half-pinned.** `BuyoutManager::Set` and `SetTab` still
  return from the `IsNull()` path after calling the repo removal, without
  erasing `m_buyouts`/`m_tab_buyouts`. The Phase 0 pin exists —
  `tst_buyoutmanager::clearingItemBuyoutRemovesInMemoryValue`, with the
  expected `QEXPECT_FAIL` — but **only for the item path**; the `SetTab`
  path has no pin. 6.1 adds the tab-side pin before fixing both.
- **`MainWindow::columns` is dead.** The public
  `std::vector<Column *> columns` member has zero references anywhere in
  `src/` or `tests/`. Delete it (6.3). The commented-out
  `MainWindow::prepare()` block below the destructor is also dead.
- **6.3's ownership picture gained a Phase 5 constraint.**
  `m_searches` is still `std::vector<Search *>` with manual `delete`
  (destructor loop and `OnDeleteTabClicked`). New since the design-intent
  version: the form holds a bound-search pointer (`SearchForm::m_boundSearch`)
  that dynamic mods rows write through, so `OnDeleteTabClicked` calls
  `m_search_form->unbind(*search)` before deleting. The unique_ptr
  conversion must preserve the order unbind → null `m_current_search` →
  erase (destroys) → `removeTab`.
- **6.5's premise came true, and more.** The selection-model exceptions the
  `OnLayoutChanged` comment describes ("using clear can cause exceptions …
  for some reason that's not clear yet") were F12's signature; since Phase 3,
  `ItemsModel::sort` emits `layoutAboutToBeChanged`, snapshots and remaps
  persistent indexes (`changePersistentIndexList`), and
  `tst_itemsmodel::selectionSurvivesSort` pins that a `QTreeView` selection
  survives a sort with no help from `MainWindow`. Verified consequence:
  `layoutChanged` is now emitted from exactly one place, `ItemsModel::sort`
  (`FilterItems` and `SetViewMode` use begin/endResetModel via
  `beginUpdate`/`endUpdate`). So the `m_layout_changed_conn` connection is
  not simplifiable — it is *removable* (D3). Also verified: today's
  `OnLayoutChanged` does `selectionModel()->reset()` then reselects one row,
  which **collapses a multi-item selection to a single row on every header
  sort** and runs an O(items) `Search::index` scan per sort. Removing the
  connection fixes both.
- **6.6's terrain after Phase 5.** `OnTabChange` now starts with
  `FlushPendingSearchFormChange()`: a pending debounced edit is applied to
  the outgoing search (`OnSearchFormChange` → `SaveViewExpansion` +
  `ModelViewRefresh`) before the form is rebound. After the flush, the
  outgoing search's buckets and view are consistent, so 6.6's new expansion
  save sees settled state; and `Search::FilterItems`'s
  `RefreshReason::TabChanged` short-circuit consults `m_states_dirty`, so a
  tab switch alone leaves buckets untouched — `Search::index()` lookups for
  selection restore remain valid. The expansion *save* on the outgoing
  search is still missing in `OnTabChange` (both branches: switch and
  "+"/`NewSearch`), and the current item is still the global
  `m_current_item`. `SaveViewExpansion`/`RestoreViewExpansion` only apply to
  unfiltered By-Tab views (`Search::defaultExpanded()`); that scoping is
  existing behavior and stays.
- **F39 (new).** `MainWindow::m_current_bucket_location` is a raw
  `const ItemLocation *` into `Search`'s bucket vector, which
  `FilterItems` clears and rebuilds. Both dereferences
  (`UpdateCurrentBucket`, `UpdateCurrentBuyout`) currently happen
  synchronously right after assignment in `OnCurrentItemChanged` — but the
  `!has_bucket` warning branch leaves the stale (or initial null) pointer in
  place and still falls through to `UpdateCurrentBuyout`, which dereferences
  it. Latent null/dangling deref; recorded as F39 in `findings.md`; fixed
  structurally by 6.6 (store the location by value).
- **F40 (new, found while designing 6.7).** `LogPanel`'s constructor does
  `spdlog::get("main")->sinks().push_back(...)` twice (a widget-backed
  `qt_color_sink` and a callback sink) and there is no `~LogPanel` and no
  sink removal anywhere. Two consequences: (a) in a test binary, where
  `logging::init` never ran, `spdlog::get("main")` returns null and
  `MainWindow` construction crashes; (b) in the app,
  `Application::SetUserDir` destroys and recreates the session — each new
  `MainWindow` pushes two more sinks while the old sinks keep pointing at
  destroyed widgets, so any warn-level log after a user-directory switch
  writes to a dangling `QTextEdit`. Same family as F29 (logging teardown
  ordering). Recorded as F40; fixed in 6.7 because the fixture needs it, and
  it is a real app bug regardless.
- **F37's premise was overstated — MainWindow is nearly constructible in a
  test today.** Verified constructor-by-constructor: every dependency in
  `MainWindow`'s 9-reference constructor is inert to construct offline —
  `QSettings` on a temp file; `NetworkManager` (sets up a disk cache, no
  requests); `RateLimiter` (stores the reference, prepares a timer);
  `SqliteDataStore` on a `QTemporaryDir` (already the Phase 0 fixture);
  `BuyoutManager` (already a fixture); `ItemsManager` (timer only, not
  started when the `autoupdate` setting is false/absent); `CurrencyManager`
  (datastore reads + builds its dialog, unshown); `Shop` (settings/datastore
  reads only); `ImageCache` (mkpath). `MainWindow`'s own constructor builds
  widgets, loads settings, and runs `NewSearch` — no network. Tests already
  link `acquisition_core`, which contains `ACQ_UI` (so the `MainWindow`
  symbol and its generated `ui_mainwindow.h` are available), and
  `tst_searchform` already runs `QTEST_MAIN` under the offscreen platform.
  The only hard blocker is F40's null-logger crash. "Needs network fixtures"
  was wrong: nothing sends a request unless an update is triggered, and the
  tests never trigger one. Test-side item injection exists too:
  `ItemsManager::OnItemsRefreshed(items, tabs, initial)` is a public slot
  that sets `m_items`, runs the buyout propagation, and emits
  `ItemsRefreshed` — exactly the app's own path.
- **6.2's layering hole is bigger than "tidiness".** `datastore.h` includes
  `currencymanager.h` solely for the `CurrencyUpdate` struct — so the entire
  datastore interface, and everything that includes it, transitively
  includes `QDialog`/`QWidget`/`QCheckBox`/`QLabel`. Also verified:
  `CurrencyManager::Save()` reads the dialog's checkboxes
  (`m_dialog->ShowChaos()`/`ShowExalt()`), `Update()` drives
  `m_dialog->Update()`, and `ExportCurrency()` opens a `QFileDialog` —
  another dialog in a business class beyond those F9 lists. Destruction
  order matters for the redesign: in `Application::UserSession`,
  `main_window` is declared last, so it is destroyed *before*
  `currency_manager` — after extraction the manager's destructor must not
  reach for the dialog (D4 moves the checkbox persistence into the dialog).
- **Shop's four `QMessageBox::warning` calls carry no control flow.** Two in
  `SubmitShopToForum` (missing threads, missing POESESSID) precede
  `m_submitting = true` and are each followed only by `return`; two in
  `OnEditPageFinished` (login required, permission denied) are followed by a
  `StatusUpdate` emission and `m_submitting = false`. Nothing sequences on
  the modal block. One observable difference after conversion: today the
  modal box holds `m_submitting == true` (in the `OnEditPageFinished` cases)
  until dismissed; with a signal the flag clears immediately, so the user
  can re-submit while the notification is visible. Harmless — submission is
  still self-guarded.
- **`UpdateChecker`'s dialog is load-bearing before any window exists.**
  `Application::InitCoreServices` connects `UpdateAvailable` →
  `AskUserToUpdate` during the login phase (no MainWindow yet) and
  disconnects it at session start, after which the main window shows its
  status-bar button instead. The old doc's "judgment call" is hereby made:
  **keep the dialog in `UpdateChecker`** — it *is* the UI for the
  window-less phase of the app's life. No change, no finding.

## Design decisions (settled July 2026)

- **D1 — MainWindow testability is a fixture, not an architecture change.**
  F37 is closed by constructing the real `MainWindow` on the real dependency
  graph over temp storage, not by extracting a controller or introducing
  interfaces. The smallest change that buys a real test, verified against
  the code: **(a)** fix F40 so `LogPanel` tolerates and cleans up its sinks
  (a bug fix the app needs anyway), **(b)** register a "main" logger in the
  test binary's init, and **(c)** a `tests/` fixture that builds
  QSettings + NetworkManager + RateLimiter + SqliteDataStore +
  BuyoutManager(+BuyoutRepo) + ItemsManager + CurrencyManager + Shop +
  ImageCache on `QTemporaryDir`s (reusing `BuyoutManagerFixture`'s parts)
  and instantiates `MainWindow`. Zero production interfaces, zero mocks.
  Tests drive the window the way the user does: `findChild<QTabBar *>()`
  for tab switches, `findChild<QTreeView *>("treeView")` for the view,
  `QTest::keyClicks` on the form's line edits (fires `textEdited`, the real
  signal path), and `ItemsManager::OnItemsRefreshed(...)` to inject items
  over the app's own refresh path. If an assertion genuinely cannot be made
  through the widget tree, prefer `setObjectName` on the widget over adding
  accessors to `MainWindow`; record whatever is added.
- **D2 — per-search view state lives on `Search`; `MainWindow` stays the
  adapter.** `Search` gains `currentItem` (a `std::shared_ptr<Item>`) and
  `currentBucket` (`std::optional<ItemLocation>`, by value — the F39 fix).
  `MainWindow` keeps its working copies and swaps them at tab switches,
  exactly the Phase 4 shape (`Search` owns state, `MainWindow` adapts it to
  widgets). No signals, no observer machinery.
- **D3 — the `layoutChanged` connection is deleted, not simplified.**
  `layoutChanged` now means exactly "a sort happened", and the view already
  survives sorts via the model's persistent-index remap (pinned by
  `tst_itemsmodel::selectionSurvivesSort`). `OnLayoutChanged` becomes a
  private `ReselectCurrentItem()` called only from the end of
  `ModelViewRefresh` (where the model was reset or swapped and the selection
  is empty by construction — every caller path goes through `FilterItems`'s
  reset or `setModel`). The `selectionModel()->reset()` line and its
  "exceptions" comment die with the connection; `m_layout_changed_conn`
  disappears; multi-item selections survive sorts for the first time.
- **D4 — dialog extraction pattern.** `CurrencyWidget`/`CurrencyDialog`/
  `CurrencyLabels` move to `src/ui/currencydialog.{h,cpp}`. **MainWindow
  owns the dialog** (created lazily in its List-Currency handler);
  `CurrencyManager` keeps data + logic, gains an `Updated()` signal (emitted
  where it called `m_dialog->Update()`), and loses all dialog knowledge.
  The `show_chaos`/`show_exalt` persistence moves into the dialog (it
  receives `QSettings &` and saves on toggle), which also removes the
  destruction-order trap noted above. `ExportCurrency` takes the target
  filename as a parameter; the `QFileDialog` moves to a `MainWindow` slot.
  `CurrencyUpdate` moves into `datastore.h` next to the interface that
  speaks it, killing the datastore→widgets transitive include. Shop's four
  warnings become one `Shop::UserWarning(const QString &message)` signal
  presented by a new `MainWindow::OnShopWarning` slot as
  `QMessageBox::warning(this, "Acquisition Shop Manager", message)` with
  the four message texts verbatim (Phase 1's `NotifyUser` pattern; kept
  separate from `OnNotifyUser`, which shows an information box with a
  different title). Wire it in `ConnectMainWindow`
  (`ui/mainwindow_bridge.cpp`).
- **D5 — F40's fix is lifecycle symmetry, not a redesign.** `LogPanel`
  stores the two `sink_ptr`s it pushes and removes them from the same
  logger in a new destructor; the constructor null-checks
  `spdlog::get("main")` and skips sink attachment (with a warning through
  the default logger) when absent. That closes both the `SetUserDir`
  dangling-sink accumulation and the test-binary crash. The test harness
  still registers a "main" logger so the sink attach/detach path runs in
  tests instead of being skipped by the null-check.
- **D6 — curation.** What ships, what drops (the reasoning behind the
  recommended order below):
  - *6.4 (unify buyout persistence, F22) is dropped.* Nothing in this phase
    forces changes to `refresh_checked_state`'s storage; migrating working
    persistence purely for uniformity is risk without user value. 6.1 adds
    the explanatory comment at the split instead. F22 stays recorded.
  - *6.8 (F34, sort-order inversion) is in, as an explicitly optional
    last item.* It is a two-line fix for a user-visible wrongness whose
    findings entry already nominates Phase 6, and this phase is the last
    scheduled release-note vehicle. It flips long-shipped sort directions,
    so it must be its own PR with its own note — easy to drop if that
    trade reads badly at implementation time.
  - Everything else stays: 6.1 and 6.6 are the phase's reasons to exist
    (user-visible correctness), 6.7 is the safety net the behavior changes
    need (and closes F37/F40), 6.3 and 6.5 are cheap once 6.7 exists, and
    6.2 pays a real layering debt (the datastore→QDialog include).

## Items

### 6.1 Fix F14 (stale in-memory buyout after clear)

Two commits:

1. **Pin the tab path.** Add
   `tst_buyoutmanager::clearingTabBuyoutRemovesInMemoryValue` mirroring the
   existing item-path case, with the same
   `QEXPECT_FAIL("", "F14: …", Continue)`.
2. **Fix and flip.** In `BuyoutManager::Set`, the `IsNull()` branch also
   does `m_buyouts.erase(item.id())`; in `SetTab`, `m_tab_buyouts.erase(
   location.id())`. Remove both `QEXPECT_FAIL` lines — the assertions go
   live. No signal is emitted on clear today (the repo removal is a direct
   call, unlike the save path's signals); keep it that way.

While in the file, add the F22 comment at the `m_refresh_checked`
declaration: buyouts persist via `BuyoutRepo` (signal-driven), refresh-check
state via `DataStore` JSON (`Save`/`Load`) — intentional split, see F22.
This closes 6.4.

Smoke (the `Get`/`GetTab` callers that could have leaned on the stale
value): clear an item price and a tab price, then check the price/date
columns re-render empty, the shop template omits the item, propagation
re-inherits correctly on the next refresh, and `OnBuyoutChange`'s
`IsGameSet()` lock checks still behave. Release-note it (see below).

### 6.2 Extract dialogs from business classes (F9 remainder)

Three independent commits, any order:

1. **`CurrencyUpdate` out of `currencymanager.h`.** Move the struct into
   `datastore.h` (its only interface consumer); drop the
   `currencymanager.h` includes from `datastore.h` and
   `sqlitedatastore.cpp`. Gate:
   `grep -rn "currencymanager.h" src/datastore/` → empty.
2. **Currency dialog to `src/ui/`** per D4: new
   `src/ui/currencydialog.{h,cpp}`; `CurrencyManager` loses `m_dialog`,
   `DisplayCurrency`, and the two `setValue` lines in `Save()`; gains
   `Updated()`. MainWindow's `actionListCurrency` handler becomes a slot
   that lazily creates/raises the dialog; `actionExportCurrency` becomes a
   slot that runs the `QFileDialog` and calls
   `ExportCurrency(fileName)`. Parity notes: today the dialog exists from
   manager construction and is updated while hidden; lazy creation is
   equivalent because the dialog constructor builds from current manager
   data and `Updated()` keeps it fresh afterwards. Also fix the stale
   header comment ("Called in itemmanagerworker::ParseItem" — it isn't;
   `ParseSingleItem` is called from `CurrencyManager::Update`).
3. **Shop warnings to a signal** per D4. Behavior change (async instead of
   modal) — release-note it. The boxes also gain a parent (`this`), so they
   center on the window instead of the screen; acceptable, note it in the
   PR.

`UpdateChecker`: no change (decided in "Verified current state").

Gates: `grep -n "QMessageBox\|QFileDialog" src/shop.cpp
src/currencymanager.cpp` → empty. Do **not** extend the grep to
`updatechecker.cpp` — its dialog deliberately stays (the F31 lesson:
acceptance greps must match the non-goals).

### 6.3 Ownership cleanup in MainWindow (part of F20)

- Delete the dead public `columns` member and the commented-out
  `prepare()` block.
- `std::vector<Search *>` + manual `delete` →
  `std::vector<std::unique_ptr<Search>>`. `m_current_search` stays a raw
  non-owning pointer. `OnDeleteTabClicked` keeps its exact order: NewSearch
  replacement dance when deleting the last search, then
  `m_search_form->unbind(*search)` → null `m_current_search` if it matches
  → `erase` (destruction happens here) → `m_tab_bar->removeTab(index)`.
  The reentrancy that makes this safe is worth a comment: `removeTab` emits
  `currentChanged` synchronously, `OnTabChange` runs inside it, its
  `FlushPendingSearchFormChange` is a no-op because `m_current_search` is
  null, and the view receives its new model before any repaint can touch
  the destroyed one. The destructor's manual delete loop disappears;
  member-order note: `m_searches` is declared after `m_filter_catalog`, so
  searches are destroyed before the catalog they reference — say so where
  the members are declared.

Behavior-identical; gated by existing tests plus 6.7's delete-tab
characterization. Gate: `grep -n "delete search\|Search \*>" src/ui/mainwindow.{h,cpp}`
→ no raw owning vector left (the non-owning `m_current_search` and slot
parameters remain).

### 6.4 Unify buyout persistence (F22) — **dropped**

See D6. The documenting comment ships with 6.1. Revisit only if a future
change forces `BuyoutManager` storage work anyway.

### 6.5 Delete the layoutChanged reselect machinery

Per D3, one commit, sequenced inside the 6.6 PR (below) or immediately
before it:

- Remove `m_layout_changed_conn` (member, disconnect, connect).
- `OnLayoutChanged` → private non-slot `ReselectCurrentItem()`; drop the
  `selectionModel()->reset()` and the "exceptions" comment; keep the
  `Search::index(m_current_item)` lookup + select/clear logic; still called
  explicitly at the end of `ModelViewRefresh` (that call is what restores
  selection after every refilter/tab switch, and it now has exactly one
  caller).

User-visible changes (release notes): multi-item selections survive header
sorts instead of collapsing to one row; large-stash sorts lose an O(items)
rescan. Verify with 6.7's sort-selection characterization (pinned before,
flipped by this commit) and the plan's manual smoke list.

### 6.6 Fix F32 + F39 (per-search view state on tab switch)

One PR together with 6.5 (both rework the same current-item code). Steps:

1. *(pre-pinned by 6.7)* The current wrong behavior — expansion reverts,
   selection survives only via the global item, detail panel goes stale on
   tab switch — is already characterized; those pins flip in this PR.
2. **Expansion half.** In `OnTabChange`, after
   `FlushPendingSearchFormChange()` and before either branch:
   `if (m_current_search) SaveViewExpansion(*m_current_search);`. This
   covers both the switch branch and the "+"/`NewSearch` branch. Ordering
   argument (verified): if an edit was pending, the flush has already
   refiltered the outgoing search and restored its (possibly changed)
   expansion, so the save reads settled post-edit state; a second save is
   idempotent. `RestoreViewExpansion` already runs inside
   `ModelViewRefresh` for the incoming search — only the save was missing.
3. **Selection half (D2).** `Search` gains `currentItem()`/
   `setCurrentItem()` (`std::shared_ptr<Item>`) and `currentBucket()`/
   `setCurrentBucket()` (`std::optional<ItemLocation>`). In `OnTabChange`'s
   switch branch: stash `m_current_item`/`m_current_bucket_location` into
   the outgoing search, load the incoming search's pair, then
   `ModelViewRefresh()` (whose `ReselectCurrentItem` finds the restored
   item; the `TabChanged` short-circuit leaves buckets valid for the
   `Search::index` lookup).
4. **Panel + buyout-widget sync.** After the refresh, the detail panel must
   reflect the restored selection (it can differ per search now):
   `UpdateCurrentItem()` already handles both cases (item → tooltip flow,
   null → `ClearCurrentItem()`); call `UpdateCurrentBuyout()` only when an
   item or bucket is current, otherwise reset the buyout widgets to their
   disabled defaults. Today the panel and buyout boxes simply keep the
   previous tab's content; syncing them is the necessary consequence of
   per-search selection and part of the F32 release note.
5. **F39 fix.** `m_current_bucket_location` becomes
   `std::optional<ItemLocation>` (by value). The `!has_bucket` branch in
   `OnCurrentItemChanged` clears it instead of leaving it stale, and
   `UpdateCurrentBuyout` guards the no-item-no-bucket case instead of
   dereferencing.
6. Flip the 6.7 pins; release-note the behavior change.

Notes: after an items refresh, a background search's stored `currentItem`
is a stale `shared_ptr` no longer present in any bucket —
`ReselectCurrentItem` degrades to "clear selection" on next activation, and
the next switch-away overwrites the stored pointer; no special handling
needed (one retained `Item` per search is trivial memory). Expansion memory
still only applies where `defaultExpanded()` is false (unfiltered By-Tab
views) — existing semantics, unchanged.

### 6.7 MainWindow test fixture + characterization tests (F37, F40)

Per D1/D5. Steps, each green:

1. **F40 fix** (production): `LogPanel` keeps its two `sink_ptr`s as
   members, removes them from the "main" logger in a new destructor, and
   null-checks `spdlog::get("main")` in the constructor. Manual check:
   change user directory via the login dialog twice, then provoke a warning
   — no crash, no duplicate log-panel entries.
2. **Fixture** (tests only): `tests/mainwindowfixture.h` builds the real
   graph per D1 on `QTemporaryDir`s. Details that matter:
   `QStandardPaths::setTestModeEnabled(true)` before constructing
   `NetworkManager` (its disk cache lands under the test location);
   register a "main" logger before the window (a sinkless logger is fine —
   the point is that `LogPanel`'s attach/detach path runs, not that
   messages go anywhere);
   leave `autoupdate` unset so `ItemsManager`'s timer stays stopped; never
   call `ItemsManager::Update` (that is the only road to the network);
   don't `show()` the window; destroy via the fixture (destructor path —
   `closeEvent`'s confirm-quit box only runs on `close()`).
   Wire `ItemsManager::ItemsRefreshed` → `MainWindow::OnItemsRefreshed`
   exactly as `ConnectMainWindow` does.
3. **Characterization tests** (`tst_mainwindow.cpp`), the F37 list — the
   sequences F33/F35 actually broke, now end-to-end:
   - *tabChangeActivatesSelectedSearch*: two searches with different name
     filters (typed via `QTest::keyClicks`, debounce flushed by the tab
     switch itself); captions/counts and visible rows correct when each
     search is activated. F41 additionally pins the current stale outgoing
     caption immediately after the fast switch.
   - *itemsRefreshRefiltersBackgroundSearches*: inject a changed item set
     via `ItemsManager::OnItemsRefreshed`; background tab captions update
     (the window-level F33 regression guard).
   - *pendingEditFollowsOutgoingSearch*: edit, switch tabs within the
     350 ms debounce, assert the outgoing search's state and caption reflect
     the edit when reactivated (window-level pin of Phase 5's flush). The
     immediate stale caption remains an F41 pin, not a 6.7 fix.
   - *deleteTabDance*: middle-click-delete paths — deleting the last
     search creates a replacement; deleting the current search activates
     the left neighbor (guards 6.3).
   - *Pins for 6.5/6.6* (current behavior, flipped later): expansion
     reverts on tab round-trip; after selecting a bucket and switching
     tabs, the detail panel and buyout widgets keep showing the previous
     tab's bucket (the genuinely stale case — an *item* selection that
     survives the switch is carried deliberately today, per the amended
     F32); multi-selection collapses after a header sort.

Sequencing: 6.7 lands **before** 6.3, 6.5, and 6.6 — it is their gate, in
the same way Phase 5's step 0 was. If some driving path proves genuinely
unreachable through the widget tree at implementation time, prefer
`setObjectName` over new `MainWindow` accessors, and record what was added;
if the whole fixture approach founders on something unforeseen, stop and
record a finding rather than reaching for a controller extraction (the
non-goal stands).

### 6.8 Optional: fix F34 (sort order inverted vs. header indicator)

Swap the two branches in `Bucket::Sort` so `Qt::AscendingOrder` produces
ascending rows. `tst_itemsmodel` deliberately asserts only that re-sorting
changes the arrangement, so it survives; after the fix, add the direction
assertion (ascending indicator ⇒ ascending `NameColumn` order) that pins
the corrected mapping. Own PR, prominent release note (every existing
sort's direction flips relative to muscle memory). Drop freely if the
timing is wrong; the finding stays either way.

## Recommended order

1. **6.1** — smallest, independent, user-visible correctness; flips the
   Phase 0 XFAIL. Can ship any time a release window opens.
2. **6.7** — the safety net; F40 fix plus the pins that 6.3/6.5/6.6 need.
3. **6.3** — mechanical, now guarded by *deleteTabDance*.
4. **6.5 + 6.6** — one PR; the phase's headline behavior change; flips the
   6.7 pins.
5. **6.2** — independent layering payoff; three separable commits.
6. **6.8** — optional coda.

6.4 does not ship. Items remain independently shippable and the plan can
stop after any of them; the plan's one-phase-per-PR working rule is relaxed
to one-item-per-PR for this phase (6.5+6.6 counting as one item).

## Hazards

- **Flush-then-save ordering in `OnTabChange`.** The expansion save (6.6)
  must come after `FlushPendingSearchFormChange()`; the flush may refilter
  the outgoing search and change what "current expansion" means. Verified
  interaction: flush → `OnSearchFormChange` already saves and restores for
  the outgoing search, so the 6.6 save is a settled re-read, not a race.
- **`select()` does not fire `currentChanged`.** `ReselectCurrentItem` uses
  `QItemSelectionModel::select`, which does not emit `currentChanged`, so
  nothing downstream refreshes the detail panel — that is why 6.6 step 4
  syncs the panel explicitly. Do not "fix" this by switching to
  `setCurrentIndex` without noticing it would fire `OnCurrentItemChanged`
  (and a buyout-widget update) on every refilter.
- **Destruction order around `OnDeleteTabClicked`** (see 6.3): the safety
  argument depends on `currentChanged` being emitted synchronously inside
  `removeTab` and on the null-`m_current_search` guard in
  `FlushPendingSearchFormChange`. Keep the order; comment it.
- **Grep-shaped acceptance criteria** must be checked against non-goals
  before being trusted (the F31 lesson). This doc's gates deliberately
  exclude `updatechecker.cpp` and `util/fatalerror.cpp`
  (`legacy/legacybuyoutvalidator.cpp` likewise keeps its widgets, per F9's
  "acceptable" list).
- **Fixture discipline** (6.7): no `ItemsManager::Update` calls, ever — it
  is the only path from these tests to `RateLimiter::Submit` and real HTTP.
  Anything that needs "new items" goes through
  `ItemsManager::OnItemsRefreshed`.
- **The debounce timer in window tests.** Prefer deterministic flushes (a
  tab switch, or letting the timer fire under `QTRY_*`) over bare
  `qWait(350)`; the timer is single-shot and restartable, and flaky waits
  here would poison exactly the tests this phase exists to make trustworthy.
- **Shop async conversion**: the four messages must stay verbatim, and the
  `m_submitting` observation from "Verified current state" (flag now clears
  before the user dismisses the notification) is accepted, not accidental.
- **6.2's lazy currency dialog**: `Updated()` must be connected when the
  dialog is created, and creation must not regress the
  "update-while-hidden" behavior (data lives in the manager, so a freshly
  created dialog is current by construction — keep it that way; don't cache
  derived values in the dialog).

## Release notes (user-visible changes)

This project has no changelog; collect these into the release notes of
whichever release ships each item:

- **6.1 (F14):** Clearing an item or tab price now takes effect
  immediately. Previously the old price could keep appearing in the item
  list, shop template, and price propagation until the next restart.
- **6.5:** Selecting multiple items now survives sorting by a column;
  previously the selection collapsed to a single item. Sorting large
  stashes is also slightly faster.
- **6.6 (F32):** Switching between search tabs now remembers each tab's
  expanded stash tabs and selected item, and the item detail panel follows
  the tab you are looking at. Previously expansion reset to defaults and
  the panel kept showing the previous tab's item.
- **6.2 (Shop):** Forum shop submission errors are now shown as
  notifications from the main window instead of dialogs that block the
  application mid-operation. Same messages, same conditions.
- **6.8 (F34, if shipped):** Clicking a column header now sorts in the
  direction the header arrow shows. All sort directions flip relative to
  previous versions, which had them inverted.

6.3 and 6.7 have no user-visible changes (6.7's F40 fix prevents a crash
after switching user directories — worth a line if a release ships it
alone).

## Acceptance criteria

Global, per commit (plan working rule 2): configure + build + full `ctest`
green, including `filters_boundary`.

Per item:

- **6.1:** both buyout-clear tests are plain assertions (no `QEXPECT_FAIL`
  left in `tst_buyoutmanager.cpp`); smoke list from the item.
- **6.2:** `grep -rn "currencymanager.h" src/datastore/` → empty;
  `grep -n "QMessageBox\|QFileDialog" src/shop.cpp src/currencymanager.cpp`
  → empty; currency dialog opens, updates on refresh, remembers its
  checkboxes across restarts; shop error notifications appear for the four
  conditions.
- **6.3:** no owning raw `Search*` container or `delete` in
  `mainwindow.{h,cpp}`; `columns` member and `prepare()` block gone;
  *deleteTabDance* green; full tab create/rename/switch/delete smoke.
- **6.5/6.6:** the 6.7 pins are flipped to assert the new behavior; F39's
  pointer is gone (`grep -n "m_current_bucket_location" src/ui/mainwindow.h`
  shows the `std::optional<ItemLocation>`); manual smoke: tab round-trip
  preserves expansion + selection + panel; sort preserves multi-selection;
  switching through a tab that filters the item out no longer costs other
  tabs their selection.
- **6.7:** `tst_mainwindow` runs in `ctest` headlessly and passes in
  isolation and in repetition (`ctest --repeat until-fail:3` locally);
  `LogPanel` removes its sinks (change user dir twice + provoke a warning:
  no crash); every characterization case from the item list exists.
- **6.8:** direction assertion added and green; release note written in
  the PR description.

After the phase: run the full plan validation checklist (build, ctest,
manual smoke list) once over the final state.
