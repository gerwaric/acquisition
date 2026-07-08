# Phase 3: Model/View Signal Hygiene

## Assumptions

Written July 2026. Assumes Phases 0–2 have landed. Code facts this spec was
written against — re-verify before implementing:

- `Search` still owns a `QTreeView &` and calls `setModel`/`reset`/
  `blockSignals` on it (`Activate`, `SetViewMode`, `RestoreViewProperties`);
  that coupling is removed in Phase 4, not here. This phase fixes *what*
  signals are emitted; Phase 4 fixes *who* owns the view.
- `Search::FilterItems()` rebuilds `m_bucket_by_tab`/`m_bucket_by_item` (the
  model's backing store) with no model signals.
- `ItemsModel::sort()` emits bare `layoutChanged()`; bucket order is stable
  under sort (only items *within* buckets reorder).
- `MainWindow::OnCheckAll`/`OnUncheckAll` emit the model's `layoutChanged`
  externally.
- `MainWindow::ModelViewRefresh()` re-connects `currentChanged` and
  `layoutChanged` on every call with no disconnect (F23).
- `MainWindow::OnSearchFormChange()` calls `SaveViewProperties()` *before*
  `ModelViewRefresh()` — expansion state is read from the view before the
  rebuild. This ordering must survive.

## Goal

`ItemsModel` becomes a well-behaved `QAbstractItemModel`: every backing-store
mutation is bracketed by the correct signals, nobody outside the model emits
its signals, out-of-contract indexes are rejected, and the view-level
compensating hacks are deleted. Findings addressed: F10, F11, F12, F23, F25.

## Non-Goals

- No `Search`/`QTreeView` decoupling (Phase 4).
- No change to what the model displays or how sorting orders items.
- The `blockSignals` on the *view* in `OnExpandAll`/`OnCollapseAll` stay —
  they suppress repeated column-resize slots, a view-level concern, not model
  hygiene.

## Design

### Reset bracketing for bucket rebuilds (F11)

`beginResetModel`/`endResetModel` are protected, so expose them:

```cpp
// ItemsModel
void beginUpdate() { beginResetModel(); }
void endUpdate()   { endResetModel(); }
```

`Search::FilterItems()` wraps its entire rebuild (clearing `m_items`, both
bucket vectors, re-bucketing) in `beginUpdate()`/`endUpdate()`. Early return
(the `TabChanged` short-circuit) must not emit anything.
`Search::SetViewMode()` swaps the active bucket source, which is also a
reset: bracket the mode change + re-sort with `beginUpdate()`/`endUpdate()`
and delete the `m_view.reset()` + `m_model.blockSignals(...)` block.

After `endResetModel()` the view has collapsed everything; expansion is
restored by the existing `RestoreViewProperties()` call — which is why the
save-before/restore-after ordering in the assumptions matters. Verify each
call path: save happens before the reset, restore after.

With resets in place, `Search::Activate()` no longer needs to re-`setModel`
on every activation: only set the model when the view's model actually
differs (tab switch). Keep the header sort indicator update.

### Sort emits a proper layout change (F12)

In `ItemsModel::sort(column, order)`:

1. `emit layoutAboutToBeChanged({}, QAbstractItemModel::VerticalSortHint);`
2. Snapshot `persistentIndexList()`; for each persistent index that refers to
   an item row, record its `(bucket_row, shared_ptr<Item>)`.
3. Perform `m_search.Sort(column, order)`.
4. For each snapshot entry, find the item's new row within its bucket and
   build the replacement index; call `changePersistentIndexList(from, to)`.
   Bucket rows are stable under sort (verify — this spec relies on it), so
   only item rows need remapping.
5. `emit layoutChanged({}, QAbstractItemModel::VerticalSortHint);`

This makes the view's own bookkeeping (selection, expansion, current index)
survive sorts correctly, which is the precondition for simplifying
`MainWindow::OnLayoutChanged`. **Keep `OnLayoutChanged`'s manual reselect
logic in this phase** (it also serves the filter-rebuild path); once resets
and persistent-index updates are in, the mysterious selection-model
exceptions mentioned in its comments should stop. If they provably do,
simplification can follow in Phase 6.

### Check-state changes emit dataChanged (F10)

Replace the external `emit ui->treeView->model()->layoutChanged()` in
`OnCheckAll`/`OnUncheckAll` with a model method:

```cpp
// ItemsModel
void refreshCheckStates()
{
    const int rows = rowCount();
    if (rows > 0) {
        emit dataChanged(index(0, 0), index(rows - 1, 0), {Qt::CheckStateRole});
    }
}
```

`MainWindow` calls `m_buyout_manager.SetRefreshChecked(...)` per bucket as
today, then `refreshCheckStates()` once. Note `MainWindow::CheckSelected`
currently emits nothing at all after mutating check state (the view repaints
lazily); route it through the same method for consistency.

### Connection lifecycle (F23)

`MainWindow` stores the two connections and replaces them explicitly:

```cpp
QMetaObject::Connection m_current_item_conn;
QMetaObject::Connection m_layout_changed_conn;
```

At the top of `ModelViewRefresh()`: `disconnect(m_current_item_conn);
disconnect(m_layout_changed_conn);` (disconnecting a default-constructed
connection is a safe no-op), then connect and store. This is robust
regardless of `QAbstractItemView::setModel`'s same-model/selection-model
behavior, which is deliberately not relied upon.

## Steps

1. `beginUpdate`/`endUpdate` + bracketing in `FilterItems` and `SetViewMode`;
   delete the view-reset/blockSignals compensations; conditionalize
   `Activate`'s `setModel`.
2. Proper sort signaling with persistent-index remapping.
3. `refreshCheckStates()` replacing external emissions (F10).
4. Connection handles in `ModelViewRefresh` (F23).
5. Bounds hardening (F25): `index()` returns `QModelIndex()` for negative
   rows, out-of-range columns, and child rows beyond the parent bucket's
   item count; `headerData()` returns `QVariant()` for out-of-range
   sections; audit `data()`/`flags()`/`setData()` for the same trust in
   caller-supplied indexes.
6. Add `tests/tst_itemsmodel.cpp`: construct a `Search` with fixture items and
   run **`QAbstractItemModelTester`** (failure mode `Fatal`) over the model
   while exercising `FilterItems`, `SetViewMode`, and `sort`. Add
   `QSignalSpy` checks: exactly one `modelAboutToBeReset`/`modelReset` pair
   per rebuild; `layoutAboutToBeChanged` precedes `layoutChanged` on sort;
   `dataChanged` with `CheckStateRole` on check-all.

Step 6 may be written first (TDD-style) — the tester will fail loudly against
the current model (the F25 defects guarantee it) and go quiet as steps 1–5
land. **Hazard:** the tester may surface further pre-existing bugs beyond
F25; fix them if small, otherwise record as findings and constrain the
tester's exercise surface, documenting why.

## Acceptance criteria

- `grep -rn 'layoutChanged' src/ui/mainwindow.cpp` shows no emissions, only
  connects.
- `grep -n 'blockSignals\|m_view.reset' src/search.cpp` → empty.
- `QAbstractItemModelTester` passes over filter/mode/sort exercise.
- Build and full `ctest` green.
- Manual smoke, with specific attention to:
  - Expansion state preserved across filter changes and view-mode switches
    exactly as before (the reset path must not lose `RestoreViewProperties`).
  - Selected item stays selected (and visible) after sorting and after
    filter changes that keep it in the result set.
  - Check All / Uncheck All / Check Selected update checkboxes immediately.
  - Extended session (many filter changes): interaction stays responsive —
    previously duplicate connections made every layout event O(N·items).
