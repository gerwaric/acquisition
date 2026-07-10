# Phase 4: Decouple `Search` from `QTreeView`

> **Spec upgrade (July 2026).** Rewritten from design intent to an
> implementation-grade spec after Phases 0–3 merged; every code fact below
> was re-verified against `design-cleanup` @ 5d665da. The original
> staleness warning no longer applies.

## Assumptions

Code facts this spec was verified against — spot-check before implementing:

- Phase 3 landed as amended: `ItemsModel` brackets rebuilds with
  `beginUpdate()`/`endUpdate()`; `sort()` emits a proper
  `layoutAboutToBeChanged`/`layoutChanged` pair with persistent-index
  remapping; `MainWindow` stores the two view connections as
  `QMetaObject::Connection` handles and calls `OnLayoutChanged()` explicitly
  at the end of `ModelViewRefresh()`.
- `Search` takes `QTreeView *` in its constructor and stores
  `QTreeView &m_view`. The complete list of `m_view` uses (grep-verified):
  - `Activate()`: `setSortingEnabled(false)` → conditional `setModel` →
    `header()->setSortIndicator(...)` → `setSortingEnabled(true)` →
    `RestoreViewProperties()`.
  - `SaveViewProperties()`: `isExpanded()` per top-level row.
  - `RestoreViewProperties()`: `expandToDepth(0)` / per-row
    `expand()`/`collapse()`.
- **The rebuild's re-sort is view-triggered.** `FilterItems()` ends with
  `SetSorted(false)`; `Activate()`'s `setSortingEnabled(true)` makes
  `QTreeView` re-apply the header sort, calling `ItemsModel::sort()`, which
  actually sorts because the sorted flag is down. That sort is the only
  `layoutChanged` of a rebuild and fires while `ModelViewRefresh()` has the
  connections down — hence the explicit `OnLayoutChanged()` at its end
  (Phase 3 pre-merge amendment). On the `TabChanged` path `FilterItems()`
  early-returns (flag stays up) and the triggered sort no-ops via the
  `m_sorted` guard.
- `SetViewMode()` no longer touches the view directly, but calls
  `SaveViewProperties()`/`RestoreViewProperties()` internally around its
  reset bracket.
- `ItemsModel` is a private member of `Search` with **no public accessor**;
  `MainWindow` and the tests reach it only via `ui->treeView->model()` /
  `view.model()`.
- Call-site inventory (complete, grep-verified):
  - `Activate`: `MainWindow::ModelViewRefresh()` (only app caller);
    `tst_itemsmodel.cpp` ×2 (used there to attach the model to a view).
  - `SaveViewProperties`: `MainWindow::OnSearchFormChange()` (before
    `ModelViewRefresh()`); internally from `SetViewMode()`.
  - `RestoreViewProperties`: internally from `Activate()` and
    `SetViewMode()` only.
  - `SetViewMode`: the `viewComboBox` lambda in the `MainWindow`
    constructor; `tst_search.cpp` ×2; `tst_itemsmodel.cpp` ×1.
  - `ModelViewRefresh()` callers: `OnSearchFormChange()` (save-before),
    `OnTabChange()` (no save — F32), `NewSearch()`, `OnItemsRefreshed()`
    (no save).
  - `Search` construction: `MainWindow::NewSearch()`; `tst_search.cpp` ×2;
    `tst_itemsmodel.cpp` ×2.
- F31 aftermath: `QTreeView::expanded`/`collapsed` are connected to
  `ScheduleResizeTreeColumns()` (0 ms coalescing timer). Two `MainWindow`
  comments name code this phase moves: the connection comment (names
  `Search::RestoreViewProperties`) and the `viewComboBox` lambda comment
  (says `SetViewMode()` restores expansion).

## Current expansion semantics (behavior gate — preserve exactly)

`SaveViewProperties()` unconditionally clears the saved header set, then
repopulates it only when `!m_filtered && mode == ByTab`.
`RestoreViewProperties()` expands everything when `m_filtered || ByItem`;
otherwise it applies the saved set per row, with an empty set collapsing
every row. Consequences — all current behavior, none changed by this phase:

| Action | Behavior today |
|---|---|
| Form change, unfiltered ByTab → unfiltered ByTab | expansion preserved |
| Applying a filter | everything expanded |
| Form change while filtered | everything stays expanded |
| Clearing a filter (filtered → unfiltered) | **all tabs collapse** (save ran while still filtered → set wiped → empty set collapses all) |
| ByTab → ByItem | everything expanded |
| ByItem → ByTab | **all tabs collapse** (save ran in ByItem → set wiped) |
| Switching search tabs | incoming search restores its last-*saved* set; outgoing search saves nothing (F32 — unchanged, see Non-Goals) |
| Items refreshed | no save; restore from last-saved set |

The original draft's acceptance criterion "expansion preserved across filter
changes, tab switches, and view-mode switches" contradicted both this table
and the phase's own non-goal — the F31 shape. The criteria below are written
against the table.

## Goal

`Search` becomes pure state — items, buckets, filter data, columns, view
mode, sort state, and expansion state as data — with no `QTreeView`
references. `MainWindow` adapts that state to the view. Finding addressed:
F18.

## Non-Goals

- No filter rework (Phase 5). `Search` remains *transitively*
  widget-dependent through `filters.h` (`FromForm()` reads widgets) until
  Phase 5; this phase removes only the direct `QTreeView` coupling.
- No change to expansion/selection behavior: this is a relocation of
  responsibilities, verified by the behavior table staying identical —
  including its two "all tabs collapse" quirks, which are easy to
  accidentally "fix".
- **F32 is deferred to Phase 6** (decision, July 2026): fixing it changes
  user-visible behavior, which would destroy this phase's
  identical-behavior gate. This phase makes the fix cheap: once
  `MainWindow` owns the save adapter, the expansion half of F32 is one
  `SaveViewExpansion()` call in `OnTabChange()`; the selection half needs
  per-search current-item state (Phase 6, alongside item 6.5). See F32 in
  `findings.md` and item 6.6 in `phase-6-mainwindow-slimming.md`.
- The F31 resize coalescing (`ScheduleResizeTreeColumns`) is not touched.
- No renames beyond what the relocation forces; `Search::Activate` keeps
  its name (now data-only).

## Design

### New `Search` API

```cpp
// Search (public)
ItemsModel &model() { return m_model; }
const std::set<QString> &expandedHeaders() const { return m_expanded_property; }
void setExpandedHeaders(std::set<QString> headers);   // moves into m_expanded_property
bool defaultExpanded() const { return m_filtered || (m_current_mode == ViewMode::ByItem); }
```

`model()` is required because `MainWindow` takes over `setModel` and the
tests currently reach the model only through the view. `defaultExpanded()`
encodes both branches of today's logic: restore expands-all exactly when it
is true, and save records headers exactly when it is false (with two view
modes, `!defaultExpanded()` ≡ `!m_filtered && ByTab`).

### MainWindow adapters

Two private `MainWindow` methods, bodies lifted from today's `Search`
methods with `ui->treeView` in place of `m_view`:

```cpp
void MainWindow::SaveViewExpansion(Search &search)
{
    std::set<QString> expanded;
    if (!search.defaultExpanded()) {
        const int rows = search.model().rowCount();
        for (int row = 0; row < rows; ++row) {
            const QModelIndex index = search.model().index(row, 0);
            if (index.isValid() && ui->treeView->isExpanded(index) && search.has_bucket(row)) {
                expanded.emplace(search.bucket(row).location().GetHeader());
            }
        }
    }
    search.setExpandedHeaders(std::move(expanded));
}

void MainWindow::RestoreViewExpansion(Search &search)
{
    if (search.defaultExpanded()) {
        ui->treeView->expandToDepth(0);
        return;
    }
    const auto &headers = search.expandedHeaders();
    const int rows = search.model().rowCount();
    for (int row = 0; row < rows; ++row) {
        const QModelIndex index = search.model().index(row, 0);
        if (headers.empty()) {
            ui->treeView->collapse(index);
        } else if (headers.count(search.bucket(row).location().GetHeader()) > 0) {
            ui->treeView->expand(index);
        } else {
            ui->treeView->collapse(index);
        }
    }
}
```

The **unconditional** `setExpandedHeaders` in the save adapter is
load-bearing: it reproduces today's `m_expanded_property.clear()` at the top
of `SaveViewProperties()`, which produces the two "all tabs collapse" rows
of the behavior table. The empty-set-collapses-everything branch in restore
is likewise deliberate. Do not "fix" either here.

### Activation

`Search::Activate(items)` reduces to data work: `FromForm();
FilterItems(items);`. The view work moves into `ModelViewRefresh()`, in
exactly today's order, in the slot where `Activate()` is called today —
i.e. while the two connections are down:

```cpp
disconnect(m_current_item_conn);
disconnect(m_layout_changed_conn);
m_buyout_manager.Save();
m_current_search->Activate(m_items_manager.items());   // data only now
ItemsModel &model = m_current_search->model();
ui->treeView->setSortingEnabled(false);
if (ui->treeView->model() != &model) {
    ui->treeView->setModel(&model);                     // tab switch only
}
ui->treeView->header()->setSortIndicator(model.GetSortColumn(), model.GetSortOrder());
ui->treeView->setSortingEnabled(true);  // triggers the re-sort; the rebuild's only
                                        // layoutChanged fires here, unseen
RestoreViewExpansion(*m_current_search);
ScheduleResizeTreeColumns();
// reconnect both connections, viewComboBox index, tab text — unchanged
OnLayoutChanged();   // unchanged (Phase 3 amendment); reword its comment, which names Activate()
```

**Decision — the re-sort stays view-triggered.** The alternative (having
`Activate()` sort its own data) was considered and rejected: it would emit
the sort's layout signals before the view is attached on tab switches and
reorder today's signal sequence for no benefit. `setSortingEnabled(true)`
plus the model's `m_sorted` guard already behaves correctly on every path,
including the `TabChanged` no-op.

### SetViewMode

The internal save/restore calls move to the only app caller, the
`viewComboBox` lambda:

```cpp
SaveViewExpansion(*m_current_search);
m_current_search->SetViewMode(static_cast<Search::ViewMode>(n));
RestoreViewExpansion(*m_current_search);
ScheduleResizeTreeColumns();
```

`SetViewMode()` keeps its reset bracket + re-sort (data + model work). The
save must run *before* `SetViewMode()` (it reads expansion under the
*outgoing* mode's `defaultExpanded()`), the restore *after* (the reset
collapsed everything) — the same ordering as today's internal calls. Update
the lambda's comment, which currently says `SetViewMode()` restores
expansion.

### Comment fixes

Reword the comments that name moved code: the `expanded`/`collapsed`
connection comment in the `MainWindow` constructor
("e.g. Search::RestoreViewProperties after a model reset"), the
`OnLayoutChanged()` comment at the end of `ModelViewRefresh()` ("Activate()
rebuilds and re-sorts…"), the `viewComboBox` lambda comment, and
`search.h`'s comment on `Activate` ("will display items in passed
QTreeView").

## Steps

Each step compiles and passes `ctest` on its own.

1. **Add the `Search` accessors**: `model()`, `expandedHeaders()`,
   `setExpandedHeaders()`, `defaultExpanded()`. Pure addition.
2. **Relocate expansion save/restore.** Add `SaveViewExpansion`/
   `RestoreViewExpansion` to `MainWindow`. Re-anchor:
   `OnSearchFormChange()` calls `SaveViewExpansion` instead of
   `search->SaveViewProperties()`; the `viewComboBox` lambda wraps
   `SetViewMode()` with save/restore; `ModelViewRefresh()` calls
   `RestoreViewExpansion` right after `Activate()` (replacing the restore
   `Activate()` did internally). Delete `Search::SaveViewProperties`/
   `RestoreViewProperties` and their internal calls in
   `Activate()`/`SetViewMode()`. Fix the affected comments.
3. **Move `Activate()`'s remaining view work into `ModelViewRefresh()`**
   per the Design listing; `Activate()` becomes `FromForm();
   FilterItems(items);`.
4. **Drop the view from `Search`**: remove the `QTreeView *` constructor
   parameter and `m_view`; update `NewSearch()`; remove the `<QTreeView>`/
   `<QHeaderView>` includes from `search.cpp` and the `QTreeView` forward
   declaration from `search.h`. (After step 3 nothing uses `m_view`, so
   steps 3 and 4 can be separate commits.)
5. **Update tests.**
   - `tst_itemsmodel.cpp`: construct `Search` without a view; attach with
     `view.setModel(&search.model())` after `search.Activate(items)`; the
     `qobject_cast<ItemsModel *>(view.model())` indirection can become
     `&search.model()`. Note the initial sort state changes: today the
     view-attach inside `Activate()` leaves the model sorted (column 0
     descending); after this phase the model starts unsorted, and the
     explicit `model->sort(...)` calls still proceed (the `m_sorted` guard
     is down). Both tests' assertions hold as written — re-verify
     `selectionSurvivesSort`'s pre-sort row expectations when touching it.
     Keeps `QTEST_MAIN` (it constructs a `QTreeView` for the selection
     model).
   - `tst_search.cpp`: drop the `QTreeView` member from `SearchHarness` and
     the `&harness.view` arguments. Stays `QTEST_MAIN`: the fixture builds
     filter widgets (`QLineEdit` etc.), which need a `QApplication` until
     Phase 5.

## Acceptance criteria

Checked against the Non-Goals: the grep below is satisfiable because every
view manipulation moves to `mainwindow.cpp`, where view code belongs; no
non-goal protects anything the grep would flag (contrast F31, where the
grep demanded removing something a non-goal protected).

- `grep -n 'QTreeView' src/search.h src/search.cpp` → empty.
- Build + full `ctest` green; the `QAbstractItemModelTester` test still
  passes.
- Manual smoke, gated on the behavior table staying identical, with
  specific attention to:
  - the two "all tabs collapse" quirks (filter-clear, ByItem→ByTab round
    trip) still behaving exactly as before;
  - sort indicator correct per-search after switching tabs; filter changes
    still re-sort (view-triggered path intact);
  - current-item reselection / item-panel clearing after filter changes
    unchanged (the explicit `OnLayoutChanged()` still runs);
  - view-mode switches stay responsive (F31 coalescing: one deferred
    column resize per switch);
  - tab switches: expansion still reverts to last-saved state (F32
    unchanged — do not accidentally fix it here either).
