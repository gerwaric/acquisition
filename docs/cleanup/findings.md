# Design Cleanup: Investigation Findings

This is the register of design and correctness problems found during the
July 2026 code investigation that motivated the cleanup plan
(`docs/cleanup/plan.md`). Each finding is anchored to symbols rather than line
numbers so it survives drift. Confidence levels:

- **Confirmed**: the defective code path has been verified by reading it
  end-to-end. "Confirmed" refers to the defect's existence; its user-visible
  impact may still need a characterization test before fixing.
- **Likely**: strong evidence, but the code path has a gap that needs a
  runtime check or test to close.

Findings are grouped by the phase that addresses them.

---

## Threading and update-flow correctness (Phase 2)

### F1. Detached parser thread mutates worker state — Confirmed

`ItemsManagerWorker::OnRePoEReady()` spawns a thread with
`QThread::create([this]{ LoadItems(); })`. `LoadItems()` runs on that thread
and mutates member state (`m_items`, `m_tabs`, `m_tab_id_index`,
`m_tabs_signature`) while the worker object itself lives on the main thread.
The only guards are `volatile bool m_initialized` / `m_updating`, and
`volatile` is not a synchronization primitive. The `QThread` object is never
joined or deleted; quitting during the parse is a crash window.

### F2. End-of-parse `Update()` runs network code on the parser thread — Confirmed

At the end of `LoadItems()`, if an update was queued
(`m_updateRequest`), the worker calls `Update(m_type, m_locations)` directly —
still on the parser thread. `Update()` leads to `Refresh()` →
`RateLimiter::Submit()`. For a new endpoint, `RateLimiter::SetupEndpoint()`
calls `m_network_manager.head()` **on the calling thread** and spins a nested
`QEventLoop` there. This drives `QNetworkAccessManager` from a thread that
does not own it (undefined behavior per Qt docs), and runs an event loop on a
thread that is about to exit.

### F3. `QMessageBox` created inside the worker — Confirmed

`ItemsManagerWorker::Update()` shows `QMessageBox::information(...)` in its
"still initializing" and "already updating" paths. This is UI inside a core
class, and if `Update()` is invoked from the parser thread (F2), it creates a
widget off the main thread — undefined behavior. Replace with signals the UI
listens to.

### F4. Error paths leave the update state machine inconsistent — Confirmed

- `OnStashListReceived` / `OnCharacterListReceived` / `OnStashReceived` set
  `m_updating = false` and return on network/parse errors, while sibling
  requests may still be in flight. The received/needed counters then never
  reach parity, so `FinishUpdate()` never runs — but `m_updating == false`
  permits a new overlapping update.
- `OnCharacterReceived` parse-failure paths return **without** resetting
  `m_updating` and without incrementing `m_characters_received`, so the update
  hangs permanently ("update already in progress" forever).

The update flow needs an explicit state machine with a single
completion/failure path.

### F5. `RateLimiter::SetupEndpoint` blocks the caller with a nested event loop — Confirmed

Documented in a code comment as deliberate (avoids flooding HEAD requests that
got users Cloudflare-blocked). Recorded here as a known re-entrancy hazard:
`Submit()` can re-enter arbitrary event processing on the calling thread. Any
Phase 2 change must preserve the "one HEAD at a time" property. Out of scope
to redesign; do not accidentally break it.

---

## Layering and dependency inversions (Phase 1)

### F6. Core code includes `ui/mainwindow.h` for the `ProgramState` enum — Confirmed

`ProgramState` is defined in `src/ui/mainwindow.h`. As a result,
`itemsmanager.h`, `shop.h`, `itemsmanagerworker.h`/`.cpp`, `repoe/repoe.cpp`,
`filters.cpp`, and `modsfilter.cpp` include the main window header. Moving the
enum to its own header removes the UI dependency from the entire item
pipeline.

### F7. Gratuitous `application.h` includes — Confirmed

`items_model.cpp`, `buyoutmanager.cpp`, `itemsmanagerworker.cpp`, `shop.cpp`,
and `itemsmanager.cpp` include `application.h` without needing it
structurally.

### F8. Filters locate `MainWindow` by walking the widget tree — Confirmed

`TabSearchFilter::Initialize` (and the same pattern in `NameSearchFilter`,
`CategorySearchFilter`, etc.) does
`qobject_cast<MainWindow *>(parent->parentWidget()->window())` to find the
main window and connect its debounce slot
(`MainWindow::OnDelayedSearchFormChange`). Filters should not know the main
window exists; the construction site should pass a signal target or callback.

### F9. Dialog UI defined inside business classes — Confirmed

- `currencymanager.h` defines `CurrencyWidget` and `CurrencyDialog` classes.
- `Shop` shows four `QMessageBox::warning` dialogs in its submission error
  paths (`shop.cpp`).
- `UpdateChecker` owns its update-prompt dialog.
- `ItemsManagerWorker` pops message boxes (F3).
- `util/fatalerror.cpp` and `legacy/legacybuyoutvalidator.cpp` also create
  widgets from non-UI code (acceptable for fatal errors; listed for
  completeness).

Scope note: Phase 1 fixes only the worker message boxes (F3, forced by the
threading work). The remainder — Shop, CurrencyManager, UpdateChecker — is
Phase 6 work, and converting Shop's modal error dialogs to signals is
behavior-affecting, so it must not be smuggled into a "mechanical" phase.

---

## Model/view signal hygiene (Phase 3)

### F10. Model consumers emit the model's signals — Confirmed

`MainWindow::OnCheckAll()` and `OnUncheckAll()` do
`emit ui->treeView->model()->layoutChanged();` from outside the model. Only
the model may emit its own signals, and `layoutChanged` requires a preceding
`layoutAboutToBeChanged` plus persistent-index updates.

### F11. `FilterItems` rebuilds the model's backing store with no reset — Confirmed

`Search::FilterItems()` clears and rebuilds `m_bucket_by_tab` /
`m_bucket_by_item` — the data `ItemsModel` reads — without any
`beginResetModel`/`endResetModel`. The code survives because
`Search::Activate()` calls `m_view.setModel()` afterwards (which resets the
view) and `SetViewMode()` calls `m_view.reset()` manually while blocking model
signals. These are compensating hacks; the model should own reset semantics
and the view-level hacks should be deleted.

### F12. Sort emits bare `layoutChanged` — Confirmed

`ItemsModel::sort()` reorders bucket contents via `Search::Sort()` and then
emits `layoutChanged()` with no `layoutAboutToBeChanged()` and no persistent
index remapping. Selection correctness after sort currently depends on
`MainWindow::OnLayoutChanged()` manually re-finding the current item
(`Search::index(item)` linear scan) and resetting the selection model — with a
comment noting `clear()` "can cause exceptions ... for some reason that's not
clear yet", which is this bug's signature.

### F25. `ItemsModel` mints out-of-contract indexes — Confirmed

`ItemsModel::index()` validates `row >= bucket_count` for top-level rows but
does not reject negative rows, out-of-range columns, or child rows beyond
the parent bucket's item count. `headerData()` indexes
`m_search.columns()[section]` with no bounds check. Because `data()` trusts
`index.column()`, an out-of-contract index minted by `index()` flows into
out-of-bounds `std::vector` access (undefined behavior).
`QAbstractItemModelTester` probes exactly these paths, so Phase 3's tester
test will hit them — bounds validation is explicit Phase 3 scope, not an
incidental discovery.

### F23. `ModelViewRefresh` accumulates duplicate signal connections — Confirmed

`MainWindow::ModelViewRefresh()` connects
`selectionModel()::currentChanged` → `OnCurrentItemChanged` and
`model()::layoutChanged` → `OnLayoutChanged` with no disconnect and no
`Qt::UniqueConnection`. It is called from `OnSearchFormChange`, `OnTabChange`,
`NewSearch`, and `OnItemsRefreshed` — i.e. on every debounced filter change.
Because each `Search` owns one `ItemsModel` for its lifetime, the
`layoutChanged` connection accumulates one duplicate per refresh, unboundedly
over a session: every sort/layout event then runs `OnLayoutChanged()` N
times, each doing an O(items) `Search::index()` scan.

The `currentChanged` half depends on `QAbstractItemView::setModel` behavior
when passed the already-set model: either the connection duplicates the same
way (if the view keeps its selection model), or the view creates a fresh
selection model per refresh and the abandoned ones accumulate as children of
the view. Both flavors are defects; the fix is the same. Fix in Phase 3:
`MainWindow` stores the two connections as `QMetaObject::Connection` handles
and explicitly disconnects/reconnects them at each activation. (An earlier
idea — "connect once per `Search` at creation" — is wrong on both counts:
the `currentChanged` connection targets the view's selection model, which
does not belong to any `Search`, and connecting every search's
`layoutChanged` at creation would fire `OnLayoutChanged` for background
searches' models during `OnItemsRefreshed`.)

---

## Suspicious, dead, or vestigial code (fix opportunistically, mostly Phase 1)

### F13. `BuyoutManager::ImportBuyouts` is a stub — Confirmed

Its body is a single log line, but `MainWindow::OnImportBuyouts()` presents a
directory picker and calls it per file as though it works. Either implement or
remove the menu action.

### F14. Clearing a buyout leaves a stale in-memory entry — Confirmed

`BuyoutManager::Set()` / `SetTab()`: when `buyout.IsNull()` (exactly the
default-constructed `Buyout()`), the code calls `m_repo.removeItemBuyout()` /
`removeLocationBuyout()` and returns **without erasing the entry from
`m_buyouts` / `m_tab_buyouts`**. `Get()` / `GetTab()` then keep serving the
stale entry from memory until restart, while the database says it is gone.
`ItemsManager::PropagateTabBuyouts()` uses `Set(item, Buyout())` precisely to
clear inherited buyouts, so the path is exercised routinely. The defect is
confirmed by reading; a Phase 0 characterization test should pin down the
user-visible impact (display/shop/export of the stale price) before the fix.

### F15. Tab-signature machinery is dead and incoherent — Confirmed; resolution: delete (Phase 1)

Two related defects in `ItemsManagerWorker`:

- In `OnStashListReceived()`, `old_tab_headers` is built from `m_tabs`, then
  the loop checks members of the same `m_tabs` against it. The condition can
  never be true; the "force refresh of moved or renamed tab" feature is dead
  code.
- `m_tabs_signature` has two producers that disagree about its meaning —
  `LoadItems()` stores `(tab_label, index-as-string)` while
  `CreateTabsSignatureVector()` stores `(name, id)` — and **zero readers**.
  The "used as consistency check" comment describes a consumer that no
  longer exists.

**Decision (July 2026): delete the machinery entirely** rather than repair
it. The mechanism dates to the reverse-engineered website API era; shipped
behavior has been "renamed/moved tabs keep stale cached metadata until they
are individually refreshed or a full refresh runs" for years, so deletion is
strictly behavior-preserving. Remove: the dead check block, the
`m_tabs_signature` member, both producers, `CreateTabsSignatureVector`, the
`TabSignature`/`TabsSignatureVector` typedefs, and the comment.

**Known limitation (accepted):** stale tab names/positions on cached tabs
after in-game renames/moves, until refresh. If this ever becomes a proven
problem, the right design is *not* re-fetching contents (renames do not
change items) but updating kept tabs' metadata from the fresh stash list a
partial refresh already receives — zero extra API calls. Recorded here so a
future need starts from that sketch instead of resurrecting the old code.

### F16. Leftover debug probe — Confirmed

`OnStashReceived()` contains `if (stash.parent == "fc672409b5") {
spdlog::info("FOUND"); }` — a hardcoded personal stash id. Delete.

### F24. Dead update-cancellation members in the worker — Confirmed

`ItemsManagerWorker::m_cancel_update` is assigned `false` in `Update()` and
read in three places (`OnStashReceived`, `OnCharacterReceived`,
`SendStatusUpdate`) but is never set `true` anywhere — cancellation cannot
occur, and the "Update cancelled." status branch is unreachable.
`m_selected_character` is only ever `clear()`ed, never assigned or read.
Remove both as part of the Phase 2 state-machine work.

### F26. `MemoryDataStore` is dead code — Confirmed

`MemoryDataStore` (`src/datastore/memorydatastore.*`) implements the
`DataStore` interface but is instantiated nowhere — it is compiled into the
binary with zero users. Phase 0 originally planned to make it the test
fixture, but an unexercised parallel implementation is exactly the drift
risk fakes are accused of; Phase 0 instead tests through the real
`SqliteDataStore` on a `QTemporaryDir` file. Delete `MemoryDataStore` in
Phase 1's dead-code sweep.

### F17. Signals declared with non-void return types — Confirmed

`BuyoutManager::SetItemBuyout` and `SetLocationBuyout` are declared as
`bool` signals. Signal return values are meaningless except in exotic
direct-connection use; these should be `void` (or plain method calls on the
repo).

---

## Structural coupling (Phases 4–6)

### F18. `Search` owns a `QTreeView&` — Confirmed

`Search` stores the view, sets its model, header sort indicator, and expansion
state (`Activate`, `SaveViewProperties`, `RestoreViewProperties`,
`SetViewMode`). Search state (items, buckets, filters, expansion-by-header)
should be view-independent; `MainWindow` should adapt it to the tree.

### F19. `Filter` subclasses are widgets-plus-logic — Resolved during Phase 5

Every filter class in `filters.h` owns `QLineEdit`/`QComboBox`/`QCheckBox`
members and builds its UI in `Initialize(QLayout*)`, while also implementing
`Matches()`. `FilterData` is an untyped grab-bag (text/min/max/rgb/checked/
mod_data) shared by all filter types. `ModsFilter` additionally owns a
dynamic grid of rows, a completer, and a debounce timer. Matching logic
should be separable and testable without instantiating widgets.

Resolved by Phase 5: definitions and matching live in `src/filters/`
(`acquisition_filters`, build-gated against `Qt6::Widgets`), widgets live in
`src/ui/searchform.*` and `src/ui/modsfilterform.*`, and `FilterData` is
replaced by the typed `FilterState` variant. `filters.{h,cpp}` and
`modsfilter.{h,cpp}` are gone.

### F33. Filter activity flags are shared across searches — Resolved during Phase 5

Found during the Phase 4 spec verification pass (July 2026).
`Filter::m_active` lives on the `Filter` object (`filters.h`), which is
shared by every `Search`; per-search values live in `FilterData`.
`m_active` is recomputed in `Filter::FromForm(FilterData *)` — i.e.
whenever a search reads the form, normally the *current* one.
`Search::FilterItems()` builds its active-filter list via
`filter->filter()->IsActive()`, so it filters with the current search's
activity flags against its own search's values. Exposed path:
`MainWindow::OnItemsRefreshed()` calls `FilterItems()` on every
*background* search — a background search with a saved name query will skip
that filter entirely if the current search's name box is empty, producing
wrong buckets and caption counts until that search is next activated.
"Likely" because traced end-to-end but not runtime-verified. Fix belongs to
Phase 5, whose per-search typed state absorbs activity — this upgrades its
"activity must move into the state structs" hazard from refactoring note to
correctness fix; pin the current wrong behavior with a characterization
test at the start of Phase 5. Note the phase-5 doc's hazard wording
("`FilterData::FromForm()` is also where `m_active` gets computed")
misplaces it: `m_active` is on `Filter`, not `FilterData` — the
misattribution is the bug.

Resolved by Phase 5: activity is derived (`FilterState::isActive()`) from
per-search state, so there is no flag to share. Pinned by
`tst_search::background*RefilterUsesOwnState` and, through the form,
`tst_searchform::backgroundSearchIgnoresCurrentFormState`.

### F35. `SocketsColorsFilter::ToForm` never clears unfilled boxes — stale colors leak across searches — Resolved during Phase 5

Found during the Phase 5 spec revision pass (July 2026). Every other
filter's `ToForm` unconditionally writes the widget (clearing it when the
search's value is unset), but `SocketsColorsFilter::ToForm` (`filters.cpp`,
shared by `LinksColorsFilter`) only calls `setText` when the corresponding
`r/g/b_filled` flag is set — it never clears a box. Switching from a search
with colors filled to one without (`MainWindow::OnTabChange` → `ToForm`)
leaves the previous search's text visible in the R/G/B boxes; the next
`FromForm` on the now-current search (any form change triggers one, and the
color boxes are on the *immediate*, undebounced path) reads that stale text
into the new search's `FilterData` — cross-search data corruption, not just
a display glitch. "Likely" because traced end-to-end but not
runtime-verified. Fix belongs to Phase 5: the colors widget adapter's
state→widget sync is symmetric by construction. Pin the current behavior in
the Phase 5 step-0 characterization pass and document the fix as a
deliberate behavior change (working rule 3).

Resolved by Phase 5: the colors adapter's `loadFrom` writes all three boxes
unconditionally, so a search without colors clears them. Deliberate
behavior change; verified in the Phase 5 manual smoke pass.

### F36. Mods filter form-sync quirks: unsaved new rows, stale combo text, orphaned visibility — Resolved during Phase 5

Found during the second (adversarial) review pass of the Phase 5 spec
(July 2026). Three related defects in `modsfilter.cpp`, all in the
widget↔data sync rather than in matching:

- **(a) New rows are not saved.** `ModsFilterSignalHandler::
  OnAddButtonClicked` → `AddNewMod()` never emits `SearchFormChanged`, so a
  newly added (still blank) row is never captured into `FilterData` by a
  `FromForm`. Switching tabs away and back discards the row, because
  `ToForm` rebuilds rows from data.
- **(b) Rebuilt rows display the wrong mod.** `SelectedMod`'s constructor
  stores the saved mod name in `m_data` but never sets the combo box's
  visible text, so rows rebuilt on a tab switch display the combo's default
  entry while matching against the saved name.
- **(c) Row-container visibility depends on add/delete history.** The
  container is shown only by `AddNewMod` and hidden only by `DeleteMod`
  (when the last row goes); `ResetForm`/`ToForm` never sync it. Deleting
  the last row on one search (container hides) and switching to a search
  with saved mods rebuilds those rows into a hidden container — they
  filter, invisibly.

"Likely" — each traced end-to-end, none runtime-verified. Fix belongs to
Phase 5 step 6: the natural mods-adapter shape (row edits mutate `ModsState`
directly, `loadFrom` writes the combo text, visibility derived from row
count) fixes all three structurally, and preserving them bug-for-bug would
require deliberate contortions. Pin the current behavior in the Phase 5
step-0 characterization pass and document the fix as the phase's third
deliberate behavior change alongside F33/F35 (working rule 3).

Resolved by Phase 5 step 6, in the full shared shape (D3, no hybrid
fallback): `src/ui/modsfilterform.*` owns the rows, all three defects are
structurally gone, and the behavior is covered by the `modsFormAdapter*`
cases in `tst_searchform`. Deliberate behavior changes; verified in the
Phase 5 manual smoke pass.

Two further mods behavior changes came with the adapter shape and are kept
deliberately — both are release-note items alongside (a)–(c):

- **(d) Deleting a row compacts the rows below it.** The old `DeleteMod`
  removed the row's widgets from the grid and left the hole; `repackRows()`
  rebuilds the layout, so the remaining rows close up. Pinned by
  `tst_searchform::modsFormAdapterRepacksRows`.
- **(e) A mod name typed but not chosen from the list now persists and
  filters.** The old `SelectedMod` stored the mod only on
  `currentIndexChanged`, so free text was dropped and never refiltered; the
  row now saves what is in the box (`editTextChanged`), which is how the
  Name box already behaves. The visible cost is that a half-typed mod name
  filters to nothing until it is finished. Pinned by
  `tst_searchform::modsFormAdapterFreeTextPersistsAndRestoresIndex`.

A third consequence needed a fix rather than a note: because row edits write
through to the bound search immediately while the refresh stays debounced, a
fast tab switch left the edited search displaying a criterion its buckets did
not reflect. `MainWindow` now flushes a pending debounced change onto the
outgoing search, and `Search::FilterItems`'s `TabChanged` short-circuit
consults a dirty flag. Pinned by
`tst_search::tabChangeRefiltersAfterStateChange` and
`tst_searchform::modsEditSurvivesTabSwitchBeforeDebouncedRefresh`.

### F20. `MainWindow` owns workflow state — Confirmed

Raw-pointer ownership of `Search*` objects with manual `delete`, current
search/bucket/item state, buyout editing rules, and all menu command logic
live in `MainWindow` (~1,300 lines). Scoped down in this plan (Phase 6):
extract opportunistically, no full controller architecture without a second
UI consumer.

### F37. The MainWindow tab-change / items-refresh sequence has no test — Confirmed

Found during the Phase 5 review (July 2026). `MainWindow::OnTabChange` →
`loadFrom` → `ModelViewRefresh` → `saveTo` → `FilterItems`, and
`OnItemsRefreshed`'s refilter of every background search, are the sequences
F33 and F35 actually broke, but nothing tests them end-to-end.
`tst_searchform` covers the pieces — the form save/restore cycle and a
background search refiltering while the form holds another search's state
(`backgroundSearchIgnoresCurrentFormState`) — which is why Phase 5 shipped
without this. Closing it properly needs `MainWindow` constructible in a test,
which today drags in `ItemsManager`, the rate limiter, and network fixtures;
that is exactly what Phase 6 (F20) makes tractable. Deferred to Phase 6
rather than built now, and covered in the interim by the manual smoke pass.

Amended during the Phase 6 spec upgrade (July 2026): the premise was
overstated. Verified constructor-by-constructor, every `MainWindow`
dependency is inert to construct offline (constructing `ItemsManager`, the
rate limiter, and `NetworkManager` touches no network; only
`ItemsManager::Update` does, and a test never calls it), tests already link
the `MainWindow` code via `acquisition_core`, and items can be injected
through the public `ItemsManager::OnItemsRefreshed` slot. The one hard
blocker found is F40 (`LogPanel` crashes on the unregistered "main" logger
in a test binary). Resolution is Phase 6 item 6.7: fix F40, then a
test-only fixture over the real dependency graph — no controller
extraction, no interfaces, no mocks.

### F39. `MainWindow`'s current-bucket pointer can dangle or start null — Likely

Found during the Phase 6 spec upgrade (July 2026).
`MainWindow::m_current_bucket_location` is a raw `const ItemLocation *`
aimed at an element of `Search`'s bucket vector
(`OnCurrentItemChanged` does
`m_current_bucket_location = &m_current_search->bucket(row).location()`),
but `Search::FilterItems` clears and rebuilds that vector on every
refilter. Today both dereferences (`UpdateCurrentBucket`,
`UpdateCurrentBuyout`) happen synchronously right after assignment, so the
common paths are safe — the reachable defect is the `!has_bucket` warning
branch in `OnCurrentItemChanged`, which leaves the pointer stale (or still
null, on a first-ever click) and then falls through to
`UpdateCurrentBuyout`, which dereferences it unconditionally when
`m_current_item` is null. "Likely" because traced but not reproduced (it
needs a click landing on a bucket row the search no longer has). Fix
belongs to Phase 6 item 6.6, which reworks exactly this state: store the
location by value (`std::optional<ItemLocation>`) and guard the
no-item-no-bucket case.

### F40. `LogPanel` leaks dangling spdlog sinks on window teardown — Confirmed

Found during the Phase 6 spec upgrade (July 2026), while designing the
`MainWindow` test fixture (F37). `LogPanel`'s constructor pushes two sinks
onto the "main" logger — a `qt_color_sink` writing into the panel's
`QTextEdit` and a callback sink updating the status button — and nothing
ever removes them: there is no `~LogPanel` and no sink cleanup. Two
consequences: (a) `Application::SetUserDir` destroys and recreates the
session including `MainWindow`, so each user-directory switch leaves two
sinks pointing at destroyed widgets and adds two more — the next
warn-or-higher log message after a switch writes to a dangling `QTextEdit`
(same family as F29: logging teardown must respect object lifetimes); and
(b) in a test binary, where `logging::init` never ran,
`spdlog::get("main")` returns null and the constructor dereferences it —
the actual blocker behind F37's "MainWindow can't be constructed in a
test". Fix in Phase 6 item 6.7 (it gates the fixture): `LogPanel` stores
its two `sink_ptr`s, removes them from the logger in a destructor, and
null-checks the logger lookup.

---

## Recorded but out of scope

### F38. The "Influenced" filter also matches fractured and synthesised items — Confirmed

Found during the Phase 5 review (July 2026) by the rewritten boolean
cross-check in `tst_filters`. The predicate is `Item::hasInfluence()`, which
is `!m_influenceList.empty()` — and `Item`'s constructor pushes `FRACTURED`,
`SYNTHESISED`, `SEARING_EXARCH` and `TANGLED_EATER` onto that list alongside
the six real influences (shaper, elder, crusader, redeemer, hunter,
warlord). So checking "Influenced" also returns every fractured and
synthesised item, which is unlikely to be what a user means: in game those
are separate concepts, and the form has its own Fractured and Synthesized
checkboxes.

Pre-existing and unchanged by Phase 5 (the old `InfluencedFilter::Matches`
called the same accessor), so it was recorded rather than fixed inline
(working rule 4). The influence list is also what drives the item's
influence icons, so narrowing `hasInfluence()` would affect display too; a
fix should give the filter its own predicate (the six influences only)
rather than change the accessor. The current behavior is pinned by
`tst_filters::booleanPredicates`, which asserts the overlap explicitly — so
a fix must flip that assertion deliberately.

### F34. `Bucket::Sort` inverts the Qt sort-order semantics — Confirmed

Found during the Phase 4 review while hardening
`tst_itemsmodel::selectionSurvivesSort`. `Bucket::Sort()` (`bucket.cpp`)
maps `Qt::AscendingOrder` to `column.lt(rhs, lhs)` — a *descending*
arrangement — and `Qt::DescendingOrder` to ascending; `Column::lt` is a
plain less-than, so nothing cancels the inversion. User-visible symptom:
the header sort-indicator arrow points opposite to the actual row order
(e.g. an "ascending" indicator shows Z→A). Pre-existing and long-shipped;
Phase 4 did not touch sorting. Fixing it flips behavior users may have
acclimated to, so it needs its own deliberate change (swap the two lambda
branches, re-check the persistent-index remap test, release-note it) —
out of scope for the cleanup phases unless Phase 6 wants it as an
opportunistic item. `tst_itemsmodel` deliberately asserts only that a
re-sort *changes* the arrangement, not which direction each enum produces,
so it will survive the fix. The Phase 6 spec upgrade (July 2026) took it up
as explicitly optional item 6.8 (own PR, prominent release note, drop
freely).

### F21. Every `Item` stores its raw JSON — Confirmed (impact unmeasured)

`Item::m_json` keeps the full JSON text of each item. With the large stash
counts this app targets (code comments mention hundreds of thousands of
items), this may be significant memory. Measure before acting; not part of
this cleanup.

### F22. Dual persistence paths in `BuyoutManager` — Confirmed

Buyouts persist through `BuyoutRepo` (signal-driven, newer) while
`refresh_checked_state` persists through `DataStore` JSON serialization
(older). Works, but the split is a trap for contributors. Unify only if a
phase touches it anyway. Decision (Phase 6 spec upgrade, July 2026):
nothing in Phase 6 forces storage changes here, so unification is dropped
(old item 6.4); item 6.1 documents the split with a comment at the
`m_refresh_checked` declaration instead. The finding stays recorded.

### F29. `spdlog::shutdown()` on `aboutToQuit` raced logging threads — Confirmed; fixed during Phase 2

Found during Phase 2 manual validation (quit-during-parse segfault, minidump
verified: `logger::should_log(this=nullptr)` on the parser thread under
`ItemsManagerWorker::LoadItems`). `main.cpp` connected `spdlog::shutdown()`
to `QCoreApplication::aboutToQuit`, which fires while the event loop is
exiting — *before* the `Application` object (and thus
`~ItemsManagerWorker`'s thread join) is destroyed. Any thread logging after
that point dereferenced a null default logger. This was part of the original
F1 "quit during parse crashes" symptom and survived the Phase 2 worker fixes
because it lives in `main.cpp`, not the worker. Fixed in Phase 2 (required
to meet its "no crash on quit during parse" acceptance criterion): the
shutdown moved to a `qScopeGuard` declared before `Application`, so it runs
after all application threads are joined. Note for the future: any log call
after `spdlog::shutdown()` crashes the same way, from any thread — logging
teardown must always come last.

### F30. `RateLimitManager` never surfaced failed replies — Confirmed; fixed during Phase 2

Found during Phase 2 manual validation (network-kill test): the entire log
history contained zero "Update failed" / "Aborting update" lines because the
worker's error paths were unreachable through the rate limiter.
`RateLimitManager::ReceiveReply` only emitted `RateLimitedReply::complete`
for successful replies. A reply with no `X-Rate-Limit-Policy` header (any
network-level failure) hit an early return, and a non-429 HTTP error hit a
log-only branch; in both cases the caller was never notified, and
`m_active_request` stayed set with no timer running — permanently jamming
that policy's queue until restart. User-visible symptom: killing the network
mid-refresh froze the update silently (looking like a rate-limit pause), and
every later request for that endpoint queued forever. Fixed in Phase 2
(required by its "network failure terminates the update cleanly" acceptance
criterion): non-retryable failures now emit `complete` (all consumers check
`reply->error()`), clear the active request, and activate the next queued
request. The 429 Retry-After resend path is unchanged. Related diagnostic
note: the frequent "policy is BORDERLINE" warnings during refreshes are
normal saturation pacing, not an error signal — arguably worth downgrading
from `warn` in a future pass.

### F27. Reply delivery during `FetchItems` submission can finish an update prematurely — Resolved during Phase 2

Found during Phase 2 review; pre-existing (the old per-handler completion
checks had the same exposure). `ItemsManagerWorker::FetchItems()` increments
`m_stashes_needed` / `m_characters_needed` one request at a time while
submitting through `RateLimiter::Submit()`. For a not-yet-seen endpoint,
`Submit()` spins the nested HEAD-request event loop (F5), which can deliver
completions for *already submitted* requests re-entrantly. If every request
submitted so far completes inside that window — plausible on the first
update of a session, when the burst allowance is full and the character
endpoint's HEAD handshake stalls the loop mid-queue — `CheckUpdateFinished()`
sees `received == needed` with the remaining requests not yet counted and
finishes the update early. The Phase 2 state guard
(`m_state != WorkerState::Updating`) prevents a double-finish, but late
replies then mutate `m_items` with no subsequent `ItemsRefreshed` emission.
"Likely" because the window was traced but not reproduced. Resolved by the
network-failure rework late in Phase 2: `FetchItems` now counts all needed
requests before submitting any, and item requests are held in a worker-side
queue with only one request in the rate limiter at a time
(`SubmitNextItemRequest`), so the completion check can no longer observe a
partially-submitted batch. This also narrows F28's exposure to at most one
stale in-flight reply.

### F28. In-flight replies from an aborted update are misattributed to the next one — Confirmed

Found during Phase 2 review; pre-existing. When an update fails (e.g. the
stash list request errors while the character list is still in flight), the
worker returns to idle but the outstanding replies stay connected to their
handlers. If a new update starts before they arrive, the stale replies are
processed as if they belonged to the current update: a stale list reply can
set `m_has_stash_list` / `m_has_character_list` early and queue duplicate
requests (partially deduplicated by `m_tab_id_index`), and a stale item reply
perturbs the received counters. Requests carry no generation tag, so handlers
cannot tell which update they answer. Convergent in practice (the next full
refresh repairs state) but a correctness hole; fixing it means tagging
requests with an update generation and discarding mismatches, or
disconnecting outstanding replies on terminal failure. Out of scope for
Phase 2.

**Possibly-related symptom observed during the Phase 5 manual smoke pass
(July 2026) — unconfirmed, and it is not even known how to reproduce it.**
After a session that included refreshing some tabs, a single item
("Damnation Hoof Two-Toned Boots") was missing from the *unfiltered* item
list and reappeared after restarting the app — so the datastore still had
it and only the in-memory list was short. That shape fits the
partial-refresh path: `RemoveUpdatingTabs` / `RemoveUpdatingItems` cull an
updating tab's items up front and the re-fetched items are added back only
as replies land, so a reply that is lost, failed, or misattributed (this
finding) leaves items culled until the next full refresh or a restart.
Filters are not implicated — they can only subset
`ItemsManager::items()`, and Phase 5 touched nothing in the items pipeline.

The session was too confounded to conclude anything: two app versions
(0.16.2 and 0.17.0) shared one data directory and migrated its datastore
version back and forth, at least one refresh in the window was heavily
rate-limited, and logging was at `info`, so the `debug`-level cull/re-add
accounting ("Keeping {} items and culling {}") was never written. Which
tabs were refreshed is not known, and no deliberate reproduction has
succeeded. A clean attempt needs a single app version on a private copy of
the data dir with logging at DEBUG, comparing the unfiltered item count
across a partial refresh. Recorded so the symptom is not lost — not because
the mechanism is established.

### F31. Phase 3 spec forced out a load-bearing view-signal guard — Resolved after Phase 3

Introduced by Phase 3 itself: the acceptance criterion
`grep -n 'blockSignals\|m_view.reset' src/search.cpp` → empty contradicted
the phase's own non-goal ("the `blockSignals` on the *view* in
`OnExpandAll`/`OnCollapseAll` stay — they suppress repeated column-resize
slots"). The `m_view.blockSignals` pair in `Search::RestoreViewProperties()`
served exactly that view-level purpose, but the grep criterion demanded its
removal, so the implementation deleted it. Result: every programmatic
expand/collapse during expansion restore fired `QTreeView::expanded`/
`collapsed` → `MainWindow::ResizeTreeColumns` (a full column-contents scan),
and the By Tab → By Item combo path additionally ran `OnExpandAll` after
`RestoreViewProperties` had already expanded — a duplicate full tree layout.
Noticeable slowdown when switching view modes; the unfiltered By Tab restore
loop (one resize per tab) was worse still, one resize per stash tab.
Resolved by coalescing: `expanded`/`collapsed` now start a 0 ms single-shot
timer (`m_delayed_resize_columns`) so any burst of expansion signals yields
one deferred `ResizeTreeColumns` per event-loop turn; all direct call sites
go through `ScheduleResizeTreeColumns()`; the now-redundant view
`blockSignals` in `OnExpandAll`/`OnCollapseAll` and the redundant
`OnExpandAll` in the view-mode combo handler were removed. Lesson recorded:
grep-shaped acceptance criteria must be checked against the phase's
non-goals before being treated as authoritative.

### F32. Search tab activation does not preserve per-search view state — Confirmed

Found during Phase 3 manual smoke; likely pre-existing because Phase 3 did
not add per-search state ownership, and `MainWindow::OnTabChange()` already
activated the selected `Search` without first saving the outgoing search's
selection/expansion state. Tab refresh checkbox state persists across search
tabs because it lives in `BuyoutManager`, not in the view. By contrast,
expanded bucket state is only saved on specific paths such as filter changes
and view-mode switches, and the currently selected item is held globally in
`MainWindow` rather than per `Search`. User-visible symptom: switch from one
search tab to another and back; the first search tab's bucket checkbox states
remain, but its buckets return to the default expanded/collapsed state and
the previously selected item is not restored. This is a view-state ownership
problem, not model signal hygiene. Defer until the Phase 4 `Search` state
cleanup or Phase 6 `MainWindow` slimming unless a regression is later traced
directly to a phase change.

Amended after the pre-merge review fix (see the Phase 3 post-implementation
amendments): `ModelViewRefresh()` now explicitly reselects the globally-held
current item at the end of every activation, so the selection half of the
symptom is narrower — switching tabs and back *does* restore the selected
item as long as every intermediate tab's search also matched it. Switching
through a tab that filters the item out clears the global pointer (and now
also clears the item detail panel, which previously went stale), so the
selection is lost in that case. The expansion-state symptom and the
underlying ownership problem are unchanged and remain deferred.

Decision (July 2026, Phase 4 spec upgrade): F32 is deferred to Phase 6, not
absorbed into Phase 4. Phase 4's verification gate is behavior-identical
relocation; fixing F32 changes user-visible behavior and would invalidate
that gate (the F31 lesson applied in advance). Phase 4 does make the fix
cheap: `MainWindow` owns the expansion save/restore adapters after it, so
the expansion half becomes a single `SaveViewExpansion()` call in
`OnTabChange()` before switching `m_current_search`; the selection half
requires per-search current-item state, which belongs with Phase 6's
`MainWindow` slimming (item 6.6, alongside 6.5's `OnLayoutChanged` work —
do the two together).
