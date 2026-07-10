# Phase 6: Opportunistic MainWindow Slimming and Remainders

> **Staleness warning.** Design intent, written July 2026. This phase is a
> curated list of remainders, not a single coherent refactor — items can be
> done independently, in any order, and the list should be re-checked against
> `findings.md` at implementation time (later phases may have added or
> resolved items).

## Goal

Clean up what the earlier phases deliberately deferred, without introducing
a controller architecture. Findings addressed: F20 (scoped down), F9
remainder, F14, F32. Optionally F22.

## Explicit non-goal, restated

No `MainController`/MVVM extraction. Without a second UI consumer (the
deferred QML work), a full controller layer is structure without payoff.
"Slimming" means: move non-UI logic out when an item below touches it
anyway; nothing more.

## Items

### 6.1 Fix F14 (stale in-memory buyout after clear)

In `BuyoutManager::Set`/`SetTab`, the `IsNull()` path must also erase the
entry from `m_buyouts`/`m_tab_buyouts`. Then remove the `QEXPECT_FAIL` from
the Phase 0 test — it flips to a real assertion. Check `Get()`/`GetTab()`
callers for any accidental reliance on the stale value (shop generation,
column display, propagation) during smoke testing.

This is Phase 6 rather than Phase 1 because it changes user-visible pricing
behavior and deserves its own release note, but it is small and can be
promoted earlier if a release window makes that convenient.

### 6.2 Extract dialogs from business classes (F9 remainder)

- Move `CurrencyWidget`/`CurrencyDialog` out of `currencymanager.h` into
  `src/ui/`; `CurrencyManager` keeps the data/logic and exposes what the
  dialog needs.
- Convert `Shop`'s four `QMessageBox::warning` calls to an error signal that
  `MainWindow` presents (same pattern as Phase 1's worker `NotifyUser`).
  Behavior change: errors become asynchronous notifications instead of
  blocking `Shop`'s control flow mid-method — read each call site and confirm
  nothing depends on the blocking (e.g. sequencing of subsequent requests).
- `UpdateChecker`'s dialog can stay if extraction proves noisy — it is
  UI-adjacent by nature. Judgment call at implementation time.

### 6.3 Ownership cleanup in MainWindow (part of F20)

- `std::vector<Search *>` + manual `delete` → `std::vector<
  std::unique_ptr<Search>>`. Watch `OnDeleteTabClicked`'s
  "delete last search creates replacement first" dance and
  `m_current_search` invalidation.
- While in the area: `columns` is a public member of `MainWindow` and appears
  unused (verify with grep — if so, delete; if used, make it private).

### 6.4 Optional: unify buyout persistence (F22)

Only if 6.1/6.2 already forced changes in `BuyoutManager`: migrate
`refresh_checked_state` from `DataStore` JSON blobs to `BuyoutRepo`-style
storage, or document the split clearly in the code. Skip freely.

### 6.5 Simplify `OnLayoutChanged` reselect logic

Deferred from Phase 3: if the selection-model exceptions its comments
describe are confirmed gone (post Phase 3 resets + persistent-index sorts),
the manual reset/reselect can likely shrink. Verify with the Phase 3 smoke
list before and after. Coordinate with 6.6 — both touch the current-item
handling.

### 6.6 Fix F32 (per-search view state on tab switch)

Deferred from Phase 4 (see the decision recorded in F32). Expansion half:
call the Phase 4 `SaveViewExpansion()` adapter on the outgoing search in
`OnTabChange()` before switching `m_current_search`. Selection half: the
current item is global (`m_current_item`); making it per-search interacts
with 6.5's `OnLayoutChanged` simplification — do the two together.
Deliberate behavior change: switching search tabs and back preserves
expansion and selection where it previously reverted to last-saved
expansion and (sometimes) lost the selection; note it in the release notes.

## Acceptance criteria

Per item: build + `ctest` green (6.1 flips the F14 XFAIL), plus the smoke
subset named in the item. There is no phase-wide gate; items ship
independently under the one-phase-per-PR rule relaxed to one-item-per-PR.
