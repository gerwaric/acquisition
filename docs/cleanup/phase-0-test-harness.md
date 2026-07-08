# Phase 0: Test Harness and Characterization Tests

## Assumptions

Written July 2026 against the codebase at the start of the cleanup
(`design-cleanup` branch). Key assumptions — re-verify if any fail:

- `CMakeLists.txt` builds a single `qt_add_executable(acquisition ...)` with
  `main.cpp` inside the `ACQ_CORE` source list.
- `Search`'s constructor requires a `QTreeView *` (changes in Phase 4).
- `Filter` subclasses construct widgets and look up `MainWindow` through the
  widget tree (changes in Phase 1/F8 and Phase 5).
- `BuyoutRepo`'s constructor takes `QSqlDatabase &`.
- `MemoryDataStore` exists in `src/datastore/`.

## Goal

A `ctest`-runnable test suite, built in CI on all three platforms, with
characterization tests for the behaviors later phases will refactor. No
production behavior changes except one mechanical extraction (tooltip text
generators).

## Non-Goals

- No fixing of any finding (F14's test *documents* the bug, it does not fix it).
- No new dependencies (Qt Test only).
- No test coverage targets; only the behaviors Phases 1–6 touch.

## Design decisions

1. **Qt Test, one executable per test class.** Standard Qt convention
   (`QTEST_MAIN`/`QTEST_GUILESS_MAIN` per file), integrates with CTest, no
   third-party assertion library.
2. **Split app sources into a static library.** `main.cpp` currently lives in
   `ACQ_CORE`, and tests must not duplicate `main`. Create
   `qt_add_library(acquisition_core STATIC ...)` holding all existing source
   groups except `main.cpp`; the `acquisition` executable becomes
   `main.cpp` + platform files, linking `acquisition_core`.
3. **In-memory SQLite for `BuyoutRepo`.** The constructor takes
   `QSqlDatabase &`, so tests create a `QSQLITE` connection with database name
   `:memory:`, open it, construct `BuyoutRepo`, and call `ensureSchema()`. No
   repo interface/fake is introduced (deliberate: prefer existing patterns).
4. **`MemoryDataStore` for `DataStore`** paths (e.g. `BuyoutManager`'s
   `refresh_checked_state`).
5. **Item fixtures from inline JSON.** Build `poe::Item` by parsing small JSON
   literals with glaze in a shared test helper, then construct `Item` with an
   `ItemLocation`. Do not add a test resource system yet.
6. **Offscreen platform for GUI-dependent tests.** `Search` needs a
   `QTreeView` and filters need widgets until Phases 4–5, so those tests use
   `QTEST_MAIN` with `QT_QPA_PLATFORM=offscreen` set via CTest environment.

## Steps

Each step leaves the build green.

### Step 1: CMake library split

- In `CMakeLists.txt`, remove `src/main.cpp` from `ACQ_CORE`.
- Add `qt_add_library(acquisition_core STATIC ...)` with all `ACQ_*` source
  groups. Move to it:
  - `qt_add_resources(...)` (from the executable),
  - `target_include_directories(... PUBLIC src ${acquisition_BINARY_DIR})`,
  - `target_link_libraries(... PUBLIC <all current Qt and external libs,
    including qdarkstyle>)`,
  - the `CXX_STANDARD 23` / `AUTORCC` properties and the MSVC warning flags.
- `qt_add_executable(acquisition WIN32 MACOSX_BUNDLE src/main.cpp ...)` keeps:
  the app icon, `version_info.rc` (Windows), the macOS bundle properties and
  dSYM step, the Linux strip/objcopy step, and the crashpad-handler copy
  command. It links `acquisition_core` PRIVATE.
- **Hazard (resources in static libs):** Qt 6's `qt_add_resources` on a static
  library normally propagates registration to consumers, but verify at
  runtime that the window icon and tooltip images load. If not, add
  `Q_IMPORT_PLUGIN`-style forced registration or call
  `Q_INIT_RESOURCE(<resource_name>)` early in `main()` and in a test helper.
- Validation: full build; run the app with `--data-dir /tmp/acq-data`; confirm
  icon, tooltip separators, and qdarkstyle themes still load. Confirm the
  three GitHub workflows still package the same executable (target name and
  output properties unchanged).

### Step 2: Test scaffolding + first test

- Add `include(CTest)` at top level and `add_subdirectory(tests)` guarded by
  `BUILD_TESTING`.
- Create `tests/CMakeLists.txt` with a helper:

  ```cmake
  function(acq_add_test name)
      qt_add_executable(${name} ${name}.cpp)
      target_link_libraries(${name} PRIVATE acquisition_core Qt6::Test)
      add_test(NAME ${name} COMMAND ${name})
      set_tests_properties(${name} PROPERTIES
          ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
  endfunction()
  ```

- First test `tests/tst_buyout.cpp` (`QTEST_GUILESS_MAIN`):
  - `BuyoutManager::StringToBuyout`: `"~b/o 5 chaos"`, `"~price 1.5 exalted"`,
    `"~gb/o 20 fusing"`, `"~c/o 3 chaos"` (obsolete type still parsed),
    garbage strings → inactive buyout, prefix/suffix tolerance of the regex.
    Note: `StringToBuyout` is an instance method; construct a `BuyoutManager`
    with the fixtures from Step 3, or move the method's test to Step 3 if
    construction order is annoying.
  - `Buyout` semantics: `IsNull()` true exactly for default-constructed;
    `IsInherited()`, `IsActive()`, `RequiresRefresh()` on representative
    values. These pin the semantics F14's fix will depend on.

### Step 3: BuyoutManager fixture + F14 characterization

- Shared helper (`tests/testfixtures.h`): creates a uniquely-named `QSQLITE`
  connection (`QUuid`-based name) with database `:memory:`, opens it,
  constructs `BuyoutRepo`, calls `ensureSchema()`. Tear down with
  `QSqlDatabase::removeDatabase` after the repo/manager are destroyed.
  **Hazard:** `BuyoutManager`'s destructor calls `Save()`, so the
  `MemoryDataStore` and repo must outlive it.
- `tests/tst_buyoutmanager.cpp` (`QTEST_GUILESS_MAIN`):
  - `Set`/`Get` and `SetTab`/`GetTab` round-trips.
  - Tab-buyout propagation logic: replicate `ItemsManager::PropagateTabBuyouts`
    preconditions (an item whose buyout `IsInherited()`, tab buyout active →
    item gets tab price with `inherited` flag; tab buyout inactive → item
    buyout cleared).
  - **F14 pin:** set an active buyout, then `Set(item, Buyout())`. Assert the
    *correct* behavior (`Get(item).IsNull()` or equivalent) and mark it
    `QEXPECT_FAIL("", "F14: stale in-memory buyout after clear", Continue)`.
    The XFAIL is removed by the phase that fixes F14.
- Item fixture helper: `makeTestItem(const char *json, ItemLocation loc)`
  parsing `poe::Item` via glaze. **Hazard:** `Item`'s constructor computes
  category and mod tables from RePoE-derived global state that tests do not
  initialize; keep assertions away from `category()` and `mod_table()` until
  a RePoE fixture exists (not in this phase).

### Step 4: Tooltip text extraction + test

- Mechanical extraction: move the file-static text generators from
  `src/ui/itemtooltip.cpp` — `ColorPropertyValue`, `FormatProperty`,
  `GenerateProperties`, `GenerateRequirements`, `getTextMods`,
  `GenerateMods`, `GenerateItemInfo` — into new files
  `src/ui/itemtooltiptext.h/.cpp` as declared free functions.
  `itemtooltip.cpp` includes the header. No signature or behavior changes
  beyond linkage.
- `tests/tst_itemtooltiptext.cpp` (`QTEST_GUILESS_MAIN`): golden-string tests
  for representative items (a rare with implicit+explicit mods, a corrupted
  unidentified item, an item with display_mode 3 properties, a talisman).
  These are the parity gate for any future tooltip work.

### Step 5: Search/FilterItems characterization

- `tests/tst_search.cpp` (`QTEST_MAIN`, offscreen): construct the filter
  vector the way `MainWindow::InitializeSearchForm` does but with a plain
  `QWidget`+layout host, a `BuyoutManager` fixture, a `QTreeView`, and a
  `Search`.
  - **Hazard:** filter `Initialize()` does
    `qobject_cast<MainWindow *>(parent->parentWidget()->window())`, gets
    `nullptr` outside the real main window, and `connect` with a null
    receiver logs a warning and no-ops. This is harmless here and goes away
    with Phase 1/F8. The layout **must** be installed on a widget or
    `parentWidget()` is null and `->window()` crashes.
  - Cases: bucket-by-tab vs bucket-by-item construction; empty tabs included
    only when no filter is active; `GetCaption()` count; item membership
    after a name filter matches/excludes.

### Step 6: Filter matching characterization

- `tests/tst_filters.cpp` (`QTEST_MAIN`, offscreen): direct `Matches()`
  coverage for representative filters — `NameSearchFilter` (case
  insensitivity, substring), `MinMaxFilter` family (min-only, max-only,
  missing property), `SocketsColorsFilter`/`LinksColorsFilter` (the
  white-socket wildcard logic in `Check()`), one boolean filter, and
  `RaritySearchFilter` (frame-type mapping including foil). Same widget-host
  hazard as Step 5.

### Step 7: CI and docs

- Add a `ctest --test-dir build --output-on-failure` step to
  `.github/workflows/build-linux.yml`, `build-macos.yml`, and
  `build-windows.yml` after the build step (offscreen env is set per-test by
  CMake, no workflow-level env needed).
- Update `BUILD.md` with the test instructions, and update the "no checked-in
  standalone test suite" statements in `AGENTS.md` and `BUILD.md`.

## Acceptance criteria

- `cmake -S . -B build && cmake --build build` produces the app and all test
  executables; `ctest --test-dir build` is green locally and on all three CI
  platforms.
- The F14 test is present and XFAIL.
- Manual smoke (plan.md checklist): app behavior unchanged, resources load.
- Release packaging assumptions intact: executable target name, bundle
  properties, installer inputs unchanged (`installer.iss` references the same
  artifacts).
