# Phase 1: Layering Fixes

## Assumptions

Written July 2026. Assumes Phase 0 has landed: `acquisition_core` static
library exists and `ctest` runs the characterization suite. Re-verify these
code facts if the phase is implemented much later:

- `ProgramState` is defined in `src/ui/mainwindow.h`; `src/repoe/repoe.h`
  forward-declares it as `enum class ProgramState;`.
- `Util::RefreshReason` / `Util::TabSelection` live in `src/util/util.h`
  inside `namespace Util` (`Q_NAMESPACE`), with top-level `using` aliases.
- Filters connect widget signals to **two different** `MainWindow` slots
  (see Step 3 mapping).
- `ItemsManagerWorker::Update()` shows two `QMessageBox::information` dialogs.

## Goal

Core code no longer includes `src/ui/mainwindow.h`; the worker no longer
creates widgets; confirmed dead/vestigial code (F13, F15, F16, F17, F26) is
removed. Findings addressed: F3, F6, F7, F8, F13, F15, F16, F17, F26.

Low-risk and behavior-preserving **by intent**, with two deliberate
exceptions: the retired Import Buyouts menu item (F13) and the worker
dialog-to-signal change (F3), which alters modality/timing.

## Non-Goals

- Shop/CurrencyManager/UpdateChecker dialogs stay where they are (F9
  remainder → Phase 6).
- `filters.cpp`/`modsfilter.h` keep including `ui/searchcombobox.h` — filters
  remain widget-owning until Phase 5. The acceptance criterion is specifically
  about `ui/mainwindow.h`.
- No threading changes (Phase 2), even though the worker is touched here.

## Steps

Each step compiles and passes tests on its own.

### Step 1: Move `ProgramState` out of `mainwindow.h` (F6)

- Create `src/util/programstate.h` containing the `ProgramState` enum moved
  verbatim from `ui/mainwindow.h` as a **plain top-level scoped enum** (not
  inside `namespace Util`). Rationale: `repoe.h`'s existing forward
  declaration (`enum class ProgramState;`) stays valid, and `namespace Util`
  already has a `Q_NAMESPACE` in `util.h` — a second `Q_NAMESPACE` for the
  same namespace in another header causes duplicate meta-object definitions.
- `ui/mainwindow.h` includes the new header (its API uses `ProgramState`).
- Replace `#include "ui/mainwindow.h"` with `#include "util/programstate.h"`
  in: `itemsmanager.h`, `shop.h`, `itemsmanagerworker.h`; and in
  `itemsmanager.cpp`, `itemsmanagerworker.cpp`, `shop.cpp`,
  `repoe/repoe.cpp` remove or replace the include depending on remaining use.
  (`filters.cpp`/`modsfilter.cpp` are handled in Step 3.)
- Note: `StatusUpdate(ProgramState, ...)` signals already cross threads today;
  Qt 6 auto-registers enum types for queued connections, so no
  `qRegisterMetaType` is needed. Verify status updates still appear during a
  refresh.

### Step 2: Worker dialogs become signals (F3)

- In `ItemsManagerWorker`, add a signal `void NotifyUser(const QString
  &message);` and replace the two `QMessageBox::information(...)` calls in
  `Update()` with emissions (keep the exact message texts). Remove the
  `QMessageBox` include; also check `itemsmanager.cpp`'s `QMessageBox`
  include, which appears unused.
- In `Application::InitUserSession()` (where worker↔UI wiring lives), connect
  `NotifyUser` to a new small `MainWindow` slot that shows
  `QMessageBox::information(this, "Acquisition", message)`.
- **Intended behavior change:** the dialog was modal *inside* `Update()`; now
  `Update()` returns immediately and the dialog appears asynchronously. The
  guard logic (`UpdateRequest` queueing / early return) is unchanged.
  This also removes a widget-off-main-thread hazard ahead of Phase 2.

### Step 3: Filters stop locating `MainWindow` (F8)

- Add to `filters.h`:

  ```cpp
  struct FilterCallbacks {
      QObject *receiver = nullptr;              // connection context/lifetime
      std::function<void()> onChanged;          // immediate refresh
      std::function<void()> onChangedDelayed;   // debounced refresh
  };
  ```

- Thread a `const FilterCallbacks &` through every filter constructor and
  `Initialize()` (store a copy). Replace each
  `qobject_cast<MainWindow *>(parent->parentWidget()->window())` + `connect`
  with `connect(widget, <signal>, callbacks.receiver, callbacks.onChangedX)`.
- **Preserve the slot mapping exactly** — this encodes which filters debounce.
  As of writing: `TabSearchFilter`, `NameSearchFilter`, `CategorySearchFilter`,
  `RaritySearchFilter`, `MinMaxFilter`, and `BooleanFilter` connect to
  `OnDelayedSearchFormChange` (→ `onChangedDelayed`);
  `SocketsColorsFilter`/`LinksColorsFilter` and `ModsFilter`
  (via `ModsFilterSignalHandler::SearchFormChanged`) connect to
  `OnSearchFormChange` (→ `onChanged`). **Re-derive this mapping from the
  code before editing** (`grep -n 'OnSearchFormChange\|OnDelayedSearchFormChange'
  src/filters.cpp src/modsfilter.cpp`); do not trust this list over the code.
- `MainWindow::InitializeSearchForm()` builds the callbacks:
  `{this, [this]{ OnSearchFormChange(); }, [this]{ OnDelayedSearchFormChange(); }}`.
- Remove `#include "ui/mainwindow.h"` from `filters.cpp` and
  `modsfilter.cpp`.
- Update Phase 0's `tst_filters`/`tst_search` to pass a dummy `QObject`
  receiver with no-op callbacks; the null-receiver connect warnings disappear.

### Step 4: Remove gratuitous `application.h` includes (F7)

- For each of `items_model.cpp`, `buyoutmanager.cpp`, `itemsmanagerworker.cpp`,
  `shop.cpp`, `itemsmanager.cpp`: verify nothing in the file references
  `Application`, then delete the include. If something does reference it,
  stop and record why in findings.md instead of forcing it.

### Step 5: Retire Import Buyouts (F13) — intended behavior change

- `BuyoutManager::ImportBuyouts` is a logging stub behind a working menu item
  and directory picker. Remove: the `actionImportBuyouts` entry (and the
  Buyouts menu if it becomes empty) from `mainwindow.ui`, the
  `MainWindow::OnImportBuyouts()` slot and its `connect`, and
  `BuyoutManager::ImportBuyouts`. The alternative (implementing it) was
  rejected: the import format is undefined and git history preserves the
  stub if the feature is ever specified.

### Step 6: Delete the debug probe (F16)

- Remove the `if (stash.parent == "fc672409b5") { spdlog::info("FOUND"); }`
  block from `ItemsManagerWorker::OnStashReceived()`.

### Step 7: Delete the tab-signature machinery (F15) — intended behavior codification

Per the F15 decision (see findings.md for rationale, accepted limitation,
and the future-design sketch), remove from `ItemsManagerWorker`: the dead
moved/renamed-tab check block in `OnStashListReceived()` (the
`old_tab_headers` loop), the `m_tabs_signature` member and both code paths
that populate it, `CreateTabsSignatureVector()`, the
`TabSignature`/`TabsSignatureVector` typedefs, and the "used as consistency
check" comment. This is pure deletion of write-only state and an
unreachable branch; doing it here keeps Phase 2's rewrite of this file
clean.

### Step 8: Delete `MemoryDataStore` (F26)

Remove `src/datastore/memorydatastore.cpp/.h` and their `CMakeLists.txt`
entries. Verify first that Phase 0 landed without using it (the spec says
`SqliteDataStore` + `QTemporaryDir`); if a test *does* use it, reconcile
with the Phase 0 spec before deleting.

### Step 9: Void signal returns (F17)

- Change `BuyoutManager::SetItemBuyout` / `SetLocationBuyout` signal
  declarations from `bool` to `void`. The connected `BuyoutRepo` slots may
  keep returning `bool` (slot return values are ignored by signal dispatch).

## Acceptance criteria

- `grep -rn '#include "ui/mainwindow.h"' src | grep -v '^src/ui/'` → empty.
- `grep -rn 'qobject_cast<MainWindow' src/filters.cpp src/modsfilter.cpp` → empty.
- `grep -rn 'TabSignature\|tabs_signature' src` → empty.
- Build green; `ctest` green; the Phase 0 filter/search tests run without
  connect warnings.
- Manual smoke, with particular attention to:
  - Typing quickly in the Name filter triggers **one** refresh after the
    350 ms debounce (slot mapping preserved).
  - Toggling a socket-color field refreshes immediately.
  - Triggering a refresh before the initial parse finishes still produces the
    "still initializing" notice (now asynchronous).
  - The Buyouts→Import menu item is gone; everything else in the menus works.
  - Status bar updates during refresh (ProgramState across threads).
