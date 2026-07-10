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

### F19. `Filter` subclasses are widgets-plus-logic — Confirmed

Every filter class in `filters.h` owns `QLineEdit`/`QComboBox`/`QCheckBox`
members and builds its UI in `Initialize(QLayout*)`, while also implementing
`Matches()`. `FilterData` is an untyped grab-bag (text/min/max/rgb/checked/
mod_data) shared by all filter types. `ModsFilter` additionally owns a
dynamic grid of rows, a completer, and a debounce timer. Matching logic
should be separable and testable without instantiating widgets.

### F20. `MainWindow` owns workflow state — Confirmed

Raw-pointer ownership of `Search*` objects with manual `delete`, current
search/bucket/item state, buyout editing rules, and all menu command logic
live in `MainWindow` (~1,300 lines). Scoped down in this plan (Phase 6):
extract opportunistically, no full controller architecture without a second
UI consumer.

---

## Recorded but out of scope

### F21. Every `Item` stores its raw JSON — Confirmed (impact unmeasured)

`Item::m_json` keeps the full JSON text of each item. With the large stash
counts this app targets (code comments mention hundreds of thousands of
items), this may be significant memory. Measure before acting; not part of
this cleanup.

### F22. Dual persistence paths in `BuyoutManager` — Confirmed

Buyouts persist through `BuyoutRepo` (signal-driven, newer) while
`refresh_checked_state` persists through `DataStore` JSON serialization
(older). Works, but the split is a trap for contributors. Unify only if a
phase touches it anyway.

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
