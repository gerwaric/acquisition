# Design Cleanup Plan

## Purpose

Improve the interior design of Acquisition — correctness, layering, and
testability — while keeping the current Qt Widgets UI and UX unchanged. This
plan supersedes the QML migration plan (see ADR 0002,
`docs/adr/0002-defer-qml-migration.md`).

This plan and the findings register come from an intensive
design/investigation effort in July 2026. The per-phase documents are being
written as **implementation-ready specs** — analysis and design judgment
front-loaded so implementation can proceed later with minimal re-derivation —
and each phase doc must exist and be reviewed before its phase begins. Until
then, the phase summaries below are design intent, not implementation detail.
The problems being fixed are cataloged with code anchors in
`docs/cleanup/findings.md` (referenced below as F1, F2, …).

## Goals

- Fix latent correctness bugs: worker threading (F1–F4), model/view signal
  discipline (F10–F12, F23), stale buyout cache (F14).
- Make the dependency graph honest: core code must not include `src/ui`
  headers (F6–F8), and the worker must not create dialogs (F3). Removing UI
  construction from core entirely (F9) is the long-term direction; this plan
  addresses it only partially (worker in Phase 1, remainder in Phase 6).
- Establish an automated test harness and characterization tests for the
  behaviors the cleanup touches.
- Keep the application shippable after every phase.

## Non-Goals

- No user-visible UI/UX changes.
- No QML, no theming changes, no packaging changes.
- No rework of the datastore, rate limiter internals, shop logic, or the
  `Item` class (F21, F22 are recorded but out of scope).
- No wholesale `MainWindow` controller architecture (scoped down; see
  Phase 6).

## Phases

Each phase has (or will have) a detailed document in `docs/cleanup/`. Phases
are ordered so that earlier phases make later ones safer; each is
independently shippable and the plan can stop after any phase with the
codebase strictly better off.

| Phase | Document | Addresses | Status |
|-------|----------|-----------|--------|
| 0. Test harness + characterization tests | `phase-0-test-harness.md` (to be written) | safety net for all later phases | Planned |
| 1. Layering fixes | `phase-1-layering.md` (to be written) | F3, F6–F8, F13, F16, F17 | Planned |
| 2. Worker threading + update state machine | `phase-2-worker-threading.md` (to be written) | F1, F2, F4, F5, F15 | Planned |
| 3. Model/view signal hygiene | `phase-3-model-signals.md` (to be written) | F10–F12, F23 | Planned |
| 4. Decouple `Search` from `QTreeView` | `phase-4-search-decoupling.md` (to be written) | F18 | Planned |
| 5. Filters as data + matching | `phase-5-filters-as-data.md` (to be written) | F19 | Planned |
| 6. Opportunistic `MainWindow` slimming | `phase-6-mainwindow-slimming.md` (to be written) | F20, F9 remainder, F14 | Planned |

Phases 0–3 are the committed core. Phases 4–6 are worthwhile continuations
with full design intent documented, but their step-by-step details must be
re-verified against the codebase state at implementation time.

### Phase summaries

**Phase 0 — Test harness.** Add a Qt Test-based test target (no new
dependencies). This includes real CMake restructuring, not just a new target:
the app sources currently build as a single `qt_add_executable` with
`main.cpp` mixed into `ACQ_CORE`, so Phase 0 must split reusable sources into
a static/object library consumed by both a thin `acquisition` executable and
the test executable, and link `Qt6::Test`. Test fixtures: `MemoryDataStore`
already covers the `DataStore` paths; `BuyoutManager` additionally needs a
`BuyoutRepo`, which takes a `QSqlDatabase&` directly — use an in-memory
SQLite database (`QSQLITE` / `:memory:`) plus `ensureSchema()` rather than
introducing a repo interface. First characterization targets:
`BuyoutManager::StringToBuyout` and buyout propagation
(`ItemsManager::PropagateTabBuyouts` logic), `Search::FilterItems` bucket
construction, filter `Matches()` behavior, and the tooltip text generators in
`src/ui/itemtooltip.cpp` (which need a small extraction to be callable
headlessly). These tests define the behavior later phases must preserve,
including pinning the user-visible impact of F14 (stale buyout cache).

**Phase 1 — Layering.** Move `ProgramState` out of `ui/mainwindow.h` into its
own header; remove every `ui/` include from core code; drop gratuitous
`application.h` includes; replace worker message boxes with signals; stop
filters from locating `MainWindow` via the widget tree. Also sweep the
confirmed dead/vestigial code: the `ImportBuyouts` stub (F13), the debug
probe (F16), and the bool-returning signals (F17). Low-risk and
behavior-preserving by intent, with two deliberate exceptions: explicitly
retired dead UI (F13), and the worker dialog-to-signal change (F3), which
alters presentation and timing — the current `QMessageBox` is modal and
blocks inside `Update()`. Out of scope here: the other embedded dialogs (F9
remainder — Shop, CurrencyManager, UpdateChecker), which are
behavior-affecting and belong to Phase 6.

**Phase 2 — Worker threading.** Keep the background parse (it exists for good
reason — parsing can take tens of seconds) but make it correct: the parse
runs as a pure function delivering results to the main thread via a queued
signal; no member mutation off-thread; `volatile` flags replaced by a proper
single-threaded state machine on the main thread; the end-of-parse `Update()`
marshaled to the main thread; the thread owned and cleaned up. Fix the error
paths so every update ends in exactly one of finished/failed (F4), and
resolve the dead moved/renamed-tab detection (F15). Constraint: preserve the
rate limiter's one-HEAD-at-a-time behavior (F5) — do not call
`RateLimiter::Submit` from any thread but main.

**Phase 3 — Model signals.** `ItemsModel` gains proper
`beginResetModel`/`endResetModel` around bucket rebuilds and correct
`layoutAboutToBeChanged`/`layoutChanged` pairs around sorts; `MainWindow`
stops emitting the model's signals (F10); the compensating hacks
(`Search::Activate`'s re-`setModel`, `SetViewMode`'s `m_view.reset()` +
`blockSignals`) are deleted. Connection lifecycle is fixed alongside (F23):
`ModelViewRefresh` currently re-connects `currentChanged`/`layoutChanged` on
every refresh with no disconnect, accumulating duplicate connections
unboundedly; after this phase, connections are made once per `Search` at
creation. Expected side benefit: the unexplained selection-model exceptions
noted in `MainWindow::OnLayoutChanged` comments should disappear.

**Phase 4 — Search decoupling.** `Search` becomes pure state (items, buckets,
filter data, expansion state keyed by location header, view mode, sort
state); `MainWindow` owns the adaptation to `QTreeView`. After Phases 3 and
4, `search.h`/`search.cpp` have no `QTreeView` references.

**Phase 5 — Filters as data.** Separate filter definition + matching (core,
testable) from widget construction (UI). Replace the `FilterData` grab-bag
with per-filter typed state. `ModsFilter` migrates last — it is the most
complex (dynamic rows, completer, debounce).

**Phase 6 — MainWindow slimming.** Opportunistic only: extract the dialog
classes out of `currencymanager.h`, move `Search*` ownership to
`std::unique_ptr`, and relocate non-UI logic encountered while doing Phases
3–5. No new controller architecture.

## Working rules

1. **One phase per PR**, branched from `design-cleanup` or its successor.
2. **Every commit compiles and passes tests.** Phase docs order their steps to
   make this possible.
3. **Tests gate refactors.** A phase that changes behavior covered by Phase 0
   tests must either keep them green or explicitly document the intended
   behavior change (bug fixes: F14, F15 may change behavior deliberately).
4. **No drive-by scope creep.** If new problems surface mid-phase, add them to
   `findings.md` instead of fixing them inline, unless the fix is required for
   the phase to proceed.
5. **Respect staleness preambles.** Each phase doc states the codebase
   assumptions it was written against. If an assumption no longer holds,
   re-verify that section against the code before following it.

## Validation checklist

After each phase, in addition to its own acceptance criteria:

- Configure and build: `cmake -S . -B build && cmake --build build`.
- Run tests: `ctest --test-dir build` (from Phase 0 onward).
- Manual smoke test with a scratch data dir:
  `./build/acquisition --data-dir /tmp/acq-data --log-level debug`.
  - Login; refresh tabs; watch status bar progress.
  - Create, rename, switch, and delete search tabs.
  - Change filters and verify item counts; reset filters.
  - Switch between "By Tab" and "By Item"; expand/collapse groups.
  - Select items and stash tabs; verify tooltip, image, minimap.
  - Edit buyouts for tabs and items; verify propagation and lock behavior.
  - Refresh selected, checked, and all tabs; interrupt with a second refresh.
  - Verify shop commands, currency dialog, theme switching, log panel, and
    rate-limit dialog.
