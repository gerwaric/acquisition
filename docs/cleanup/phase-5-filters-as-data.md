# Phase 5: Filters as Data + Matching

> **Staleness warning.** Design intent, written July 2026 before earlier
> phases were implemented. Assumes Phases 0–4 landed (notably: filter
> characterization tests exist, `FilterCallbacks` from Phase 1/F8 replaced
> the MainWindow widget-tree lookup). This is the largest and least
> mechanical phase; re-plan the step details against the code before
> starting, and expect this doc to need a revision pass at that point.

## Goal

Separate what a filter *is* (definition + matching, core, headless-testable)
from how it is *rendered* (widgets, UI). After this phase, filter matching
logic compiles without any widget headers, and the `FilterData` grab-bag is
replaced by typed per-filter state. Finding addressed: F19.

## Non-Goals

- No visual or behavioral changes to the search form: same controls, same
  layout, same debounce/immediate semantics (the mapping preserved by Phase
  1/F8 must survive verbatim).
- No change to which filters exist or how they match — the Phase 0
  characterization tests are the gate and must pass unmodified (except
  mechanical API renames).
- `SearchComboBox` stays a widget in `src/ui/`.

## Target shape (sketch — refine at implementation time)

- **Core** (`src/filters/` or similar): per-filter-family definition + state:

  ```cpp
  struct MinMaxState { std::optional<double> min, max; };
  bool matchesMinMax(const Item &, const MinMaxState &, /*property accessor*/);
  ```

  Families: text (tab/name), combo (category/rarity), min-max (properties,
  DPS methods, sockets/links counts), socket-colors (the R/G/B + wildcard
  `Check()` logic), boolean flags, and mods. State must be serializable-in-
  principle (plain values), because per-`Search` storage replaces
  `FilterData`.
- **UI adapters** (`src/ui/filterwidgets/` or similar): one widget class per
  family that renders controls, reads/writes the state struct, and calls the
  Phase 1 `FilterCallbacks`. These absorb the `Initialize()` bodies and the
  `FromForm`/`ToForm`/`ResetForm` triple.
- **Per-search storage:** `Search` holds a vector of typed states (or a
  variant), replacing its `std::vector<std::unique_ptr<FilterData>>`.
  `FilterItems` consumes states + matchers directly.

## Migration order

One family per commit, characterization tests green after each:

1. Boolean flag filters (smallest surface, many classes, pure `data->checked`
   logic).
2. Min-max family (`MinMaxFilter` + subclasses; the `GetValue`/
   `IsValuePresent` virtuals become per-instance accessors/lambdas).
3. Text and combo filters (tab, name, category, rarity).
4. Socket/link color filters (port `Check()`'s wildcard logic carefully —
   it is characterization-tested in Phase 0 for exactly this reason).
5. `ModsFilter` last: dynamic row set, `SearchComboBox`+completer,
   `TokenAndFilterProxy`, its own debounce timer, and the
   `ModsFilterSignalHandler` indirection. Budget as much time for this one
   as for families 1–4 combined; if it resists the shared shape, it is
   acceptable to leave it as a self-contained widget+state hybrid with only
   its *matching* logic extracted, and record that as a finding.

## Hazards

- `FilterData::FromForm()` is also where `m_active` gets computed; activity
  ("does this filter currently constrain anything") must move into the state
  structs, and `Search::FilterItems`'s active-filter fast path must keep
  working.
- `CategorySearchFilter::k_Default` / `RaritySearchFilter` default sentinels
  (`"<any>"`) are part of matching semantics, not just UI.
- Filter state save/restore across tab switches (`ToForm`/`FromForm` cycle
  driven by `MainWindow::OnTabChange`) must survive the restructure; the
  Phase 0 `tst_search` tab-switching cases guard this.

## Acceptance criteria

- Filter matching code includes no widget headers;
  `tests/tst_filters.cpp` runs `QTEST_GUILESS_MAIN`.
- All Phase 0 characterization tests pass unmodified in their assertions.
- Manual smoke: every filter group produces identical result counts on a
  reference stash before/after; debounced vs immediate refresh behavior
  unchanged; filter state restores correctly when switching search tabs.
