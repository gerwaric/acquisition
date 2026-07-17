# Findings Register

This is the living register of design and correctness problems, begun during
the July 2026 code investigation that motivated the (now completed) design
cleanup and carried forward by the items-pipeline redesign
(`docs/design/items-pipeline.md`). Findings are anchored to symbols rather
than line numbers so they survive drift.

Rules of the register:

- **F-numbers are permanent and never reused.** Code comments, commit
  messages, and design docs cite them. New findings continue from the
  highest number ever assigned, even though most earlier numbers now live
  only in the resolved ledger.
- Resolved findings are compacted to one ledger line below. Their full
  prose — mechanisms, decisions, amendments — is in git history
  (`git log -p -- docs/cleanup/findings.md`, before July 2026's register
  trim).
- Confidence levels: **Confirmed** means the defective code path was
  verified by reading it end-to-end; **Likely** means strong evidence with
  a gap needing a runtime check.

---

## Open findings

### F22. Dual persistence paths in `BuyoutManager` — Confirmed; won't fix unless touched

Buyouts persist through `BuyoutRepo` (signal-driven, newer) while
`refresh_checked_state` persists through `DataStore` JSON serialization
(older). Works, but the split is a trap for contributors. Decision (Phase 6
spec upgrade, July 2026): nothing forces storage changes here, so
unification was dropped; a comment at the `m_refresh_checked` declaration
documents the split. Unify only if other work touches this storage anyway.

### F46. `ItemsManager::OnItemsRefreshed` runs an O(items) scan purely for logging — Confirmed

Found during the July 2026 items-pipeline recon. The uncategorized-items
loop in `ItemsManager::OnItemsRefreshed` (`itemsmanager.cpp`) walks the
entire item vector calling `category()` on every refresh, and its only
output is a per-item `trace` line and one `debug` count. The scan runs even
when neither level is enabled. Cheap per item but whole-collection per
emit; gate it behind `spdlog::should_log` or drop it. Opportunistic —
absorb into items-pipeline M2/M3 work on this function rather than fixing
inline (working rule 3).

### F50. Network failures are logged as rate-limit-header anomalies — Confirmed

Found during the items-pipeline M1 manual validation (July 2026,
network-kill test). The handling is correct — this is the F30 fix working
as designed: `RateLimitManager::ReceiveReply` surfaces a header-less reply
to the caller as a failed request and moves the queue along, and the
worker aborts cleanly with items intact. But the diagnostic wording is
wrong: "The rate limit manager received a reply for stash-request-limit
without rate limit headers: error TimeoutError" frames a plain network
failure as a rate-limit anomaly, and it reads as if the rate limiter
misinterpreted the event (it was misread exactly that way in testing).
The branch also conflates two distinct cases: `reply->error() != NoError`
(an expected network-level failure — should be a `warn` saying "network
error for <endpoint>: <errorString>; failing the request") and a reply
with *no* error but missing headers (a genuine API anomaly deserving
`error`). Split the two and reword. Same diagnostics family as the F30
BORDERLINE note.

### F52. `PropagateTabBuyouts` issues a database delete per item on every refresh — Confirmed

Observed July 16, 2026, during the M1 folder validation (Standard league,
18,491 items): every `ItemsRefreshed` emit produced ~17,250 consecutive
`BuyoutRepo: removing item buyout` lines — once after the initial cached
parse, again after a 4-tab checked refresh, identical both times (they are
the bulk of a 5.8 MB debug log). Mechanism, verified in code:
`ItemsManager::PropagateTabBuyouts` calls `Set(item, Buyout())` for every
item whose buyout is inherited when its tab has no active buyout — which
for most accounts is nearly every item — and `BuyoutManager::Set`'s
`IsNull` branch calls `m_repo.removeItemBuyout(item)` **unconditionally**,
even when neither the memory map nor the database has an entry for that
item. Net effect: one SQL DELETE (plus a debug log line) per item per
refresh, ~17k no-op statements each time for this account. The fix shape:
only call `removeItemBuyout` when the map actually holds an entry for the
item. Caveat on the guard (July 17 review of PR #163): the memory
map does **not** strictly mirror the repo — `CompressTabBuyouts` (runs
every refresh via `ApplyAutoTabBuyouts`) erases vanished tabs' entries
from the map without repo deletes, and `MigrateItem` rekeys map entries
without repo writes (see F54). Both drift in the same direction — repo
holds rows the map lacks — which is exactly the case the guard declines
to delete, so the fix loses nothing; orphan rows persist in the DB and
are re-erased from the map each session (accepted tradeoff, pinned by
`clearingAbsentBuyoutSkipsRepo` / `clearingAbsentTabBuyoutSkipsRepo`).
Second caveat (same review): erasing the map entry *before* the repo
delete would make a failed DELETE unretryable — the guard would skip
every later clear and the row would resurrect at the next `Load()`.
Fixed shape: `removeItemBuyout`/`removeLocationBuyout` return success
and the map entry is erased only afterward, so a failed delete is
retried on the next clear (pinned by `failedDeleteKeepsEntryForRetry`).
The save path still discards failures — `saveItemBuyout` returns bool
but is invoked via signal connection (`application.cpp`), so a failed
save silently drops the buyout at the next restart; accepted for now,
noted here so the delete/save asymmetry is deliberate. Same
snapshot-cascade family as F46; also a hard constraint on
items-pipeline M2 — the per-tab delta path must scope buyout
propagation to the delta, not rerun this loop per tab reply.
Fix assigned: PR #163.

### F53. Deleted stash tabs and characters resurrect from the cache at restart — Confirmed; follow-up PR assigned

Found during the July 2026 review of items-pipeline M1 (PR #162). The
worker treats the fresh stash/character lists as authoritative and drops
server-deleted tabs with their items — but only in memory. The connected
repositories only upsert the rows a list returns and never delete absent
ones (`StashRepo::saveStashList`, `CharacterRepo::saveCharacterList`),
and both return early for an empty list, so even "everything was deleted"
cannot be expressed. `getStashList`/`getCharacterList` return every row
for the realm/league, so `ParseCachedItems()` reloads deleted locations
and their saved item JSON at the next startup.

**Fix shape (amended July 17 — the original "delete rows absent from the
top-level list" would have erased every Map/Unique child's cached JSON,
since special children are never in any list; see the F49 ledger entry).**
Three parts:

1. **On a fresh top-level list:** delete rows absent from the
   *recursively flattened* list (folder children included) — except rows
   whose `parent` is a surviving Map/Unique tab, which live only in
   parent replies. Handle the empty-list case. Key deletion off top-level
   lists only: `ProcessTab` re-emits `stashListReceived` for folder
   children, and a partial child-list save must never drive deletion.
2. **On a parent stash reply:** reconcile that parent's child rows
   against its fresh child list — the datastore mirror of the worker's
   in-memory ghost-child reconcile (without it, dropped ghosts resurrect
   via `ParseCachedItems` at the next startup). Cascade-delete child rows
   when the parent itself disappears from the list.
3. **Policy note:** with `get_map_stashes`/`get_unique_stashes` off, the
   parent-reply reconcile deletes cached child rows, matching the
   in-memory and pre-M1 full-refresh semantics; toggling the setting back
   on then requires a refetch rather than showing stale cache.

Assigned: standalone PR after M1 merges (deliberately kept out of
PR #162).

### F54. The v4→v5 buyout migration never persists under the repo-backed store — Confirmed (mechanism); reachability unverified

Found July 17, 2026, while validating the F52 fix's map-mirrors-repo
assumption. Pre-existing on master; unrelated to PRs #162/#163.

`BuyoutManager::MigrateItem` moves `m_buyouts` entries from legacy hash
keys to item-id keys **in memory only**. The `m_buyout_manager.Save()`
call that follows the migration loop in `ItemsManager::MigrateBuyouts`
persists nothing but `refresh_checked_state` (the F22 split), and no
path writes the migrated rows to `BuyoutRepo` or deletes the old-keyed
ones. So a store that reaches `MigrateBuyouts` at `db_version <= 4`
migrates in memory, works for one session, then loses those buyouts at
the next restart: `Load()` refills the map with hash-keyed rows that
id-based `Get()` never reads, and the `db_version == 5` guard prevents
re-migration. The buyout data still exists in the repo under keys
nothing consults — silent, permanent-looking loss from the user's view.

Reachability: dormant for any install already at `db_version` 5. The
plausible live path is the legacy importer (`LegacyDataStore` reads
`db_version` from the old database); whether the import writes a
`db_version < 5` into the new store — arming the migration — is
unverified. Check that before sizing the fix.

Fix shape: make the migration write through the repo (save the id-keyed
row, delete the hash-keyed one) inside `MigrateItem`, or have
`MigrateBuyouts` flush the affected entries after the loop. Related: F22
(dual persistence), F51 ledger entry (do not rekey `GetLegacyHash` — a
correct future v4→v5 migration depends on it).

### F55. A failed first fetch permanently consumes a new tab's newness — Confirmed; release-blocking for M1

Found July 17, 2026, in the post-review pass over items-pipeline M1
(PR #162). M1's always-fetch contract promises that tabs and characters
created server-side since the last list are fetched even by partial
refreshes. The mechanism breaks under failure: list reconciliation adds a
newly discovered tab to `m_tabs`/`m_tab_id_index` as soon as the list is
processed (`ProcessTab`), and `OnStashListReceived` emits
`stashListReceived` *before* any item fetch, persisting the tab's
metadata to the datastore at list receipt. If the update then fails
terminally before that tab's first item request completes (every terminal
failure path clears `m_queue`), the tab is already "previously known" to
all later refreshes — in memory for the session, and across restarts via
`ParseCachedItems` — so nothing fetches it unless it is selected,
checked, or part of a full refresh. The next successful partial refresh
then publishes the tab empty. The trigger window (list received, queue
discarded before the new tab's reply) is exactly the environment M1
exists for: hours-long refreshes that will fail mid-flight.

Fix shape: track "contents known" separately from "metadata known". A
session-only set is insufficient — metadata persistence at list receipt
consumes newness across restarts — so derive contents-known from the
datastore: a tab whose items were never saved has no items row, which is
distinguishable from "fetched and legitimately empty" if the datastore
exposes it. Add an offline regression pin to the fake-network harness:
fail an update after list receipt but before the new tab's first reply,
run a successful partial refresh, and assert the new tab's contents are
still fetched.

Assigned: the post-M1 follow-up PR, alongside F53 (both are
datastore-persistence edges of list reconciliation). Release-blocking
for M1: the release notes advertise the always-fetch behavior, which is
false in this window until fixed.

---

## Standing constraints and lessons

Rules distilled from resolved findings that remain binding on future work.
The F-numbers refer to the ledger below.

- **F5 — one HEAD at a time.** `RateLimiter::SetupEndpoint` deliberately
  blocks the caller with a nested event loop to avoid flooding HEAD
  requests (users got Cloudflare-blocked). Any change must preserve the
  one-HEAD-at-a-time property, never call `RateLimiter::Submit` off the
  main thread (a `Q_ASSERT` enforces this), and treat `Submit()` as
  re-entrant — it can deliver completions for already-submitted requests
  before it returns.
- **F29 — logging teardown comes last.** Any log call after
  `spdlog::shutdown()` crashes from any thread. Shutdown lives in a
  `qScopeGuard` declared before `Application` in `main.cpp` so it runs
  after all threads are joined; keep it that way.
- **F30 — BORDERLINE is not an error.** The frequent "policy is
  BORDERLINE" rate-limit warnings during refreshes are normal saturation
  pacing, not a failure signal (arguably worth downgrading from `warn`).
- **F31 — check acceptance criteria against non-goals.** A grep-shaped
  acceptance criterion once forced out a load-bearing guard the same
  spec's non-goals said to keep. Mechanical criteria are subordinate to
  stated intent.
- **F42 — never mutate `logger->sinks()` outside `logging::init`.**
  UI-lifetime sinks attach/detach through the permanent
  `dist_sink_mt` hub, whose mutex makes detach safe against worker-thread
  logging.

---

## Resolved ledger

Full prose for every entry is in git history (see the register rules
above). "PR #161" refers to the post-Phase-6 follow-ups branch
(`cleanup-followups`).

| F | Finding | Resolution |
|----|---------|------------|
| F1 | Detached parser thread mutated worker state | Fixed, Phase 2 |
| F2 | End-of-parse `Update()` ran network code on the parser thread | Fixed, Phase 2 |
| F3 | `QMessageBox` created inside the worker | Fixed, Phase 1 |
| F4 | Error paths left the update state machine inconsistent | Fixed, Phase 2 |
| F5 | `SetupEndpoint` nested event loop | Standing constraint (above) |
| F6 | Core included `ui/mainwindow.h` for `ProgramState` | Fixed, Phase 1 |
| F7 | Gratuitous `application.h` includes | Fixed, Phase 1 |
| F8 | Filters located `MainWindow` via the widget tree | Fixed, Phase 1 |
| F9 | Dialog UI defined inside business classes | Fixed, Phase 1 (worker) + Phase 6 (Shop, CurrencyManager, UpdateChecker) |
| F10 | Model consumers emitted the model's signals | Fixed, Phase 3 |
| F11 | `FilterItems` rebuilt the model's store with no reset | Fixed, Phase 3 |
| F12 | Sort emitted bare `layoutChanged` | Fixed, Phase 3 |
| F13 | `ImportBuyouts` was a stub behind a working menu action | Retired, Phase 1 |
| F14 | Clearing a buyout left a stale in-memory entry | Fixed, Phase 6 |
| F15 | Tab-signature machinery dead and incoherent | Deleted, Phase 1; its metadata-refresh sketch is absorbed by items-pipeline M1 |
| F16 | Leftover hardcoded debug probe | Deleted, Phase 1 |
| F17 | Signals declared with non-void return types | Fixed, Phase 1 |
| F18 | `Search` owned a `QTreeView&` | Fixed, Phase 4 |
| F19 | Filter classes were widgets-plus-logic | Fixed, Phase 5 |
| F20 | `MainWindow` owns workflow state | Scoped down and done, Phase 6 (opportunistic extraction only) |
| F21 | Every `Item` stores its raw JSON | Overtaken by events (glaze migration); dead persistence path swept, PR #161 |
| F23 | `ModelViewRefresh` accumulated duplicate connections | Fixed, Phase 3 |
| F24 | Dead update-cancellation members | Removed, Phase 2 |
| F25 | `ItemsModel` minted out-of-contract indexes | Fixed, Phase 3 |
| F26 | `MemoryDataStore` dead code | Deleted, Phase 1 |
| F27 | Re-entrant completions could finish an update early | Resolved by the Phase 2 network rework (single request in flight) |
| F28 | In-flight replies from an aborted update were misattributed to the next one, and updates began destructively — a terminal failure left `m_items` silently short, published by the next successful partial refresh (the likely "item missing until restart" mechanism) | Fixed, items-pipeline M1 (update generation tag + atomic per-reply replacement). Validated by the offline fake-network harness (mutation-verified stale-discard and fail-mid-update pins) and the July 16 live network-kill; the recorded missing-item repro was retired as moot once the destructive cull path was deleted |
| F29 | `spdlog::shutdown()` raced logging threads | Fixed, Phase 2; standing lesson (above) |
| F30 | Rate limiter never surfaced failed replies | Fixed, Phase 2; BORDERLINE note (above) |
| F31 | Phase 3 spec forced out a load-bearing view-signal guard | Resolved after Phase 3 (coalesced resize); standing lesson (above) |
| F32 | Per-search view state not preserved across tab switches | Fixed, Phase 6 (items 6.5/6.6) |
| F33 | Filter activity flags shared across searches | Fixed, Phase 5 |
| F34 | `Bucket::Sort` inverted Qt sort-order semantics | Fixed, Phase 6 item 6.8 |
| F35 | Socket-color boxes never cleared across searches | Fixed, Phase 5 |
| F36 | Mods filter form-sync quirks (a–e) | Fixed, Phase 5 step 6 |
| F37 | No `MainWindow` end-to-end test | Fixture built, Phase 6 item 6.7 |
| F38 | "Influenced" filter matched fractured/synthesised items | Fixed, PR #161 |
| F39 | Current-bucket pointer could dangle or start null | Fixed, Phase 6 item 6.6 |
| F40 | `LogPanel` leaked dangling spdlog sinks | Fixed, Phase 6 item 6.7 |
| F41 | Fast tab switch left the outgoing tab's caption stale | Fixed, PR #161 |
| F42 | `LogPanel` sink teardown lifetime race | Fixed, PR #161; sink-hub rule (above) |
| F43 | Restored bucket selection not highlighted in the tree | Fixed, PR #161 |
| F44 | Item-path warning branches kept stale selection state | Fixed, PR #161 |
| F45 | Shop threads could not be cleared; no-threads warning unreachable | Fixed, July 2026 (own change) |
| F47 | `ItemLocation::FixUid()` was dead code | Deleted, items-pipeline M1 |
| F49 | Folder children suspected of being fetched twice via two paths | Closed by live observation, July 2026: the paths are complementary in the live API — folder children arrive only via the stash list (Standard, 16 child lists; the two individually fetched folders returned no `children` and no items), map/unique children only via the individual reply (Mirage, 73 children). The `OnStashReceived` tripwire warning stays in the code as a permanent guard should the API ever change. July 17 amendment: the warning's "fetched twice this update" claim can be false — during a partial refresh a known-but-unselected child is never queued from the stash list, so the parent path would be its only fetch. Reword to "may be fetched twice" or check a per-update set of queued fetch ids; assigned alongside F55 |
| F48 | Character-list skip-check compared names against an id-keyed index (never matched), so a partial update re-added and re-fetched every character in the league, duplicating their tab entries and items | Found and fixed, items-pipeline M1 (character entries rebuilt from the fresh list, keyed by id) |
| F51 | Unnamed stash tabs (~30 on the validating account, real in-game data) collapse the label component of the legacy item-buyout hash, suspected of shadowing item buyouts across tabs | Reframed and closed, July 2026: active item buyouts key on the API item id (`BuyoutManager::Set`/`Get` use `item.id()`); the label-based `hash_v4` is consumed only by the one-time v4→v5 migration, where colliding tabs made that migration ambiguous — an accepted legacy quirk. Do not rekey `GetLegacyHash`: it would break future v4→v5 migrations without improving live lookups |
