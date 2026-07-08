# Phase 4: Decouple `Search` from `QTreeView`

> **Staleness warning.** This spec is design intent, written July 2026 before
> Phases 0–3 were implemented. It assumes their landed state — in particular
> Phase 3's reset discipline, without which this phase is not safe. Re-derive
> the step details from the code at implementation time; trust the intent,
> not the line-level claims.

## Assumptions

- Phase 3 landed: `ItemsModel` brackets rebuilds with resets; `Search` no
  longer calls `m_view.reset()`/`blockSignals`; `MainWindow` owns the two
  view connections.
- `Search` still takes `QTreeView *` in its constructor, stores `m_view`, and
  uses it in `Activate` (setModel + sort indicator) and
  `SaveViewProperties`/`RestoreViewProperties` (expansion read/write).

## Goal

`Search` becomes pure state — items, buckets, filter data, columns, view
mode, sort state, and *expansion state as data* — with no widget references.
`MainWindow` adapts that state to the `QTreeView`. Finding addressed: F18.

## Non-Goals

- No filter rework (Phase 5).
- No change to expansion/selection behavior; this is a relocation of
  responsibilities, verified by identical smoke behavior.

## Design

- **Constructor:** drop the `QTreeView *` parameter. `Search` keeps owning
  `ItemsModel` (it already constructs it with `*this`).
- **Activation:** `Search::Activate(items)` reduces to data work (`FromForm`,
  `FilterItems`, restore sort). The view work moves to a `MainWindow` helper
  (natural home: `ModelViewRefresh`):
  - `setModel` when the view's model differs,
  - header sort indicator from `model.GetSortColumn()/GetSortOrder()`,
  - expansion restore (below).
- **Expansion state:** `Search` keeps the *data* (`std::set<QString>` of
  expanded location headers — same keying as today) with accessors roughly:

  ```cpp
  const std::set<QString> &expandedHeaders() const;
  void setExpandedHeaders(std::set<QString> headers);
  bool defaultExpanded() const;   // filtered || ByItem mode → expand all
  ```

  `MainWindow` implements the two adapters: *save* (walk top-level rows, ask
  the view `isExpanded`, map row → header via `Search::bucket()`) and
  *restore* (apply header set / defaultExpanded to the view). These are the
  bodies of today's `SaveViewProperties`/`RestoreViewProperties` with the
  view passed in from outside; preserve their exact semantics, including the
  "empty set means collapse everything" branch.
- **`SetViewMode`:** stays in `Search` (it is data + model reset after Phase
  3); the expansion save/restore around it moves to the caller.
- **Call-path inventory before editing:** every `SaveViewProperties`/
  `RestoreViewProperties`/`Activate` call site in `mainwindow.cpp` must be
  re-anchored; list them with grep first and keep the save-before-rebuild /
  restore-after-rebuild ordering established in Phase 3.

## Steps

1. Add the expansion-state accessors to `Search`; reimplement
   `Save/RestoreViewProperties` as `MainWindow` helpers using them; delete
   the `Search` methods.
2. Move `Activate`'s view work into `ModelViewRefresh`; shrink `Activate` to
   data work.
3. Remove `QTreeView *` from the constructor and the `m_view` member; update
   `NewSearch()`.
4. Update tests: `tst_search` no longer needs a `QTreeView`; switch it to
   `QTEST_GUILESS_MAIN` if nothing else in it requires widgets (filters still
   do until Phase 5 — if the fixture builds filters, it stays `QTEST_MAIN`).

## Acceptance criteria

- `grep -n 'QTreeView' src/search.h src/search.cpp` → empty.
- Build + `ctest` green; `QAbstractItemModelTester` test still passes.
- Manual smoke focus: expansion preserved across filter changes, tab
  switches, and view-mode switches; sort indicator correct after switching
  tabs; current-item reselection after filtering unchanged.
