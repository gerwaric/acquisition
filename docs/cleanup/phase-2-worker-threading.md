# Phase 2: Worker Threading and Update State Machine

## Assumptions

Written July 2026. Assumes Phases 0 and 1 have landed:

- `acquisition_core` library and `ctest` suite exist (Phase 0).
- The worker's `QMessageBox` calls are already signals (`NotifyUser`, Phase
  1/F3), so nothing in the worker creates widgets.
- `ProgramState` lives in `util/programstate.h` (Phase 1/F6).

Code facts this spec was written against — re-verify before implementing:

- `ItemsManagerWorker` lives on the main thread; `OnRePoEReady()` starts a
  detached `QThread::create([this]{ LoadItems(); })`.
- `LoadItems()` mutates members (`m_items`, `m_tabs`, `m_tab_id_index`,
  `m_tabs_signature`) and, when `m_updateRequest` is set, calls `Update()`
  directly on the parser thread.
- `RateLimiter::Submit` → `SetupEndpoint` (new endpoints) calls
  `NetworkManager::head()` and spins a nested `QEventLoop` on the calling
  thread (F5).
- `m_initialized` / `m_updating` are `volatile bool`.
- `m_cancel_update` is never set true; `m_selected_character` is never used
  (F24).
- `SqliteDataStore::getThreadLocalDatabase()` already creates per-thread
  SQLite connections keyed by thread id, with mutex-protected cleanup, so
  reading the cache from a worker thread is supported by design.

## Goal

The initial cached-items parse still runs on a background thread (it can take
tens of seconds), but with no shared mutable state, a correct handoff, an
owned thread, and an update flow whose every outcome ends in exactly one of
finished/failed. Findings addressed: F1, F2, F4, F15, F24. Constraint
honored: F5.

## Non-Goals

- The worker stays on the main thread. Do **not** move it to a dedicated
  thread — that would force the rate limiter and its nested-event-loop HEAD
  handshake (F5) to become thread-aware.
- No rate limiter redesign. Its one-HEAD-at-a-time blocking behavior is
  load-bearing (documented Cloudflare-ban avoidance).
- No retry logic or new error UX beyond making failures terminate the update
  cleanly.

## Design

### Background parse: thread-local accumulation + move handoff

The parse must not touch worker members. Restructure `LoadItems()` into a
method that builds and returns a result struct:

```cpp
struct ParseResult {
    std::vector<ItemLocation> tabs;
    Items items;
    std::set<QString> tab_id_index;
    TabsSignatureVector tabs_signature;
};
ParseResult ParseCachedItems() const;   // runs on the parser thread
```

`ParseCachedItems()` may read members that are set in the constructor and
never written afterwards (`m_realm`, `m_league`, `m_account`, `m_settings`
reads) and may `emit StatusUpdate(...)` for progress (signal emission is
thread-safe; delivery is queued). It creates its own `UserStore` exactly as
today (per-thread SQLite connections are already supported). It must not
write any member.

Handoff by move, not by queued-signal argument copy. A queued signal copies
its arguments (for `Items`, an O(N) vector-of-`shared_ptr` copy — the current
code pays this twice on the initial parse). A move-capturing lambda posted
with `QMetaObject::invokeMethod` transfers ownership O(1) and leaves exactly
one owner on exactly one thread:

```cpp
// at the end of the thread body:
QMetaObject::invokeMethod(this,
    [this, result = std::move(result)]() mutable {
        OnParseCompleted(std::move(result));
    },
    Qt::QueuedConnection);
```

`OnParseCompleted` (main thread): move the result into the members, set state
to Idle, emit `ItemsRefreshed(m_items, m_tabs, true)` — now a same-thread
direct connection with no copies — and, if an update was queued during the
parse, call `Update()` **from here**, which fixes F2 (all
`RateLimiter::Submit` calls happen on the main thread from now on).

Non-issue, for the record: `Item` objects created on the parser thread and
destroyed on the main thread are plain heap objects with no thread affinity;
the queued event provides the happens-before edge.

### Thread lifecycle

- Keep `QThread::create`, but store the pointer, `connect(thread,
  &QThread::finished, thread, &QThread::deleteLater)`.
- Add `std::atomic<bool> m_shutdown{false}`. `ParseCachedItems` checks it at
  each stash/character iteration and returns early when set.
- The worker destructor sets `m_shutdown`, then `wait()`s on the thread if
  running. Quitting mid-parse now blocks briefly instead of racing a live
  thread (previously a crash window).

### Update state machine (F4)

Replace `volatile bool m_initialized` / `m_updating` with one plain enum on
the main thread (no atomics needed once nothing off-thread touches it):

```cpp
enum class WorkerState { Initializing, Idle, Updating };
```

- `Update()` guards: `Initializing` → queue the request + `NotifyUser`;
  `Updating` → reject + `NotifyUser`; `Idle` → proceed.
- Every reply handler (`OnStashListReceived`, `OnCharacterListReceived`,
  `OnStashReceived`, `OnCharacterReceived`) must, on **every** exit path —
  success, network error, parse error — count the reply as completed and then
  call a single `CheckUpdateFinished()`. Failures increment a
  `m_request_failures` counter instead of writing state directly. This fixes
  both F4 flavors: the "error while siblings in flight" case (state no longer
  flips to idle early) and the "parse error forgets to count" hang in
  `OnCharacterReceived`.
- `CheckUpdateFinished()`: when the needed lists have arrived and
  received == needed for both stashes and characters —
  - failures == 0 → `FinishUpdate()` as today (sort, emit
    `ItemsRefreshed(..., false)`, state Idle);
  - failures > 0 → status update "Update failed: N requests failed", state
    Idle, **no** `ItemsRefreshed` emission. This preserves the current
    "abort on error" semantics but with consistent state: the next refresh
    works instead of hanging or overlapping.
  - A failed *list* request means no item requests were queued; the counters
    are zero on both sides, so the same check terminates the update
    immediately.
- Remove `m_cancel_update` and `m_selected_character` (F24), including the
  unreachable "Update cancelled." branch in `SendStatusUpdate`.

### Defensive assertion (F5 guard)

Add to the top of `RateLimiter::Submit`:

```cpp
Q_ASSERT(QThread::currentThread() == QCoreApplication::instance()->thread());
```

Cheap, and turns any future reintroduction of cross-thread submission into an
immediate debug-build failure instead of a latent race.

### Moved/renamed tab detection (F15) — behavior restoration

In `OnStashListReceived`, the current check builds `old_tab_headers` from
`m_tabs` and then tests members of the same `m_tabs` — it can never fire.
Additionally, `m_tabs_signature` is overwritten with the *new* list before
the comparison. Fix:

1. Capture `previous_signature = std::move(m_tabs_signature)` **before**
   assigning the new one.
2. For each incoming stash tab whose id is already in `m_tab_id_index` (i.e.
   a kept tab not already scheduled for refresh), compare its (name, index)
   against `previous_signature`; if it moved or was renamed, queue a request
   for it (`m_update_tab_contents` permitting).

This restores a dead feature, so treat it as an intended behavior change:
stale item locations in renamed/moved tabs get re-fetched during partial
refreshes. If implementation turns out gnarly (signature bookkeeping across
`TabSelection` modes), the fallback is to delete the dead block and record a
finding that renamed-tab contents stay stale until manually refreshed —
do not ship a half-working version.

## Steps

1. **Parse restructure + handoff + thread lifecycle** (F1, F2): everything
   under "Background parse" and "Thread lifecycle" above, plus converting the
   two flags to `WorkerState` (the guards in `Update()` move over
   mechanically). Compiles and runs standalone.
2. **Reply-handler completion discipline** (F4): centralize
   `CheckUpdateFinished()`, make every handler exit path count, add
   `m_request_failures`, remove F24 members.
3. **F15 fix** as described, or the documented fallback.
4. **Rate limiter assertion** (one line, any time).

## Testing

- Optional but recommended unit test (`tests/tst_workerparse.cpp`): seed a
  `UserStore` in a `QTemporaryDir` with one stash (via
  `StashRepo::saveStashList`/`saveStash` with fixture JSON), run
  `ParseCachedItems()` synchronously on the test thread, assert tab and item
  counts. This pins the parse output shape across the restructure.
- The network flow has no seam for unit testing (no network abstraction, out
  of scope), so F4/F15 validation is manual.

## Acceptance criteria

- `grep -n 'volatile' src/itemsmanagerworker.h` → empty; no member writes in
  any code executed on the parser thread (review `ParseCachedItems` and the
  thread body line by line).
- Build green, `ctest` green, including the new parse test if implemented.
- Manual smoke, with specific attention to:
  - Fresh login with a large league: progress messages during parse, items
    appear, no crash on quit *during* the parse.
  - Trigger a refresh before the initial parse completes: "still
    initializing" notice appears; the queued update runs after the parse
    (this exact path previously ran network code on the parser thread).
  - Kill network connectivity mid-refresh: status reports failure, state
    returns to idle, and a subsequent refresh works (previously: permanent
    "update in progress" or inconsistent state).
  - F15: rename a stash tab in game, refresh a *different* tab via Refresh
    Selected, confirm the renamed tab's contents are re-fetched.
  - Debug build: full session with no `Q_ASSERT` trips in
    `RateLimiter::Submit`.
