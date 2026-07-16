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

### F28. In-flight replies from an aborted update are misattributed to the next one — Confirmed; fix in progress (items-pipeline M1)

Found during Phase 2 review; pre-existing. When an update fails, the worker
returns to idle but outstanding replies stay connected to their handlers. If
a new update starts before they arrive, the stale replies are processed as
if they belonged to the current update: a stale list reply can set
`m_has_stash_list` / `m_has_character_list` early and queue duplicate
requests, and a stale item reply perturbs the received counters. Requests
carry no generation tag, so handlers cannot tell which update they answer.
The Phase 2 network rework narrowed the exposure (list requests are
submitted serially and item requests go one at a time through
`SubmitNextItemRequest`), so at most one reply can be outstanding when a
terminal failure lands — but the hole is real and unguarded.

**Mechanism established (July 2026 items-pipeline recon):** the
more load-bearing adjacent hole is that updates *begin destructively*.
`ItemsManagerWorker::Update()` culls the updating tabs' items up front
(`RemoveUpdatingTabs` / `RemoveUpdatingItems`) and re-adds them only as
replies land. On terminal failure, `CheckUpdateFinished()` returns to idle
*without* emitting `ItemsRefreshed`, silently leaving `m_items` short; the
UI survives that moment because `ItemsManager` holds the last emitted copy,
but the **next successful partial refresh** emits the still-short `m_items`
and the missing items propagate to the UI — gone until a full refresh or
restart (the datastore is unaffected, which is why a restart restores them).

**Possibly-related symptom (Phase 5 manual smoke, July 2026) —
unconfirmed.** A single item ("Damnation Hoof Two-Toned Boots") was missing
from the *unfiltered* item list after a session that included partial
refreshes, and reappeared after restart. That two-step shape fits the
mechanism above exactly. The session was too confounded to conclude
anything (two app versions sharing one data dir, heavy rate limiting,
logging at `info` so the cull/re-add accounting was never written). A clean
repro attempt needs a single app version on a private copy of the data dir
with `--log-level debug`, comparing unfiltered item counts across a partial
refresh.

**Fix assigned:** items-pipeline Milestone 1 (`docs/design/items-pipeline.md`)
— an update generation tag (discard mismatched replies), plus atomic
per-reply replacement so updates never begin destructively. The repro
protocol above is M1's manual validation.

### F46. `ItemsManager::OnItemsRefreshed` runs an O(items) scan purely for logging — Confirmed

Found during the July 2026 items-pipeline recon. The uncategorized-items
loop in `ItemsManager::OnItemsRefreshed` (`itemsmanager.cpp`) walks the
entire item vector calling `category()` on every refresh, and its only
output is a per-item `trace` line and one `debug` count. The scan runs even
when neither level is enabled. Cheap per item but whole-collection per
emit; gate it behind `spdlog::should_log` or drop it. Opportunistic —
absorb into items-pipeline M2/M3 work on this function rather than fixing
inline (working rule 3).

### F47. `ItemLocation::FixUid()` is dead code — Confirmed

`ItemLocation::FixUid()` (`itemlocation.cpp`) has no callers anywhere in
the tree. Its legacy-API uid-trimming concern survives elsewhere
(`ItemsManagerWorker::ProcessTab` trims long ids inline). Delete the method
next time `ItemLocation` is touched — items-pipeline M1 touches the class
and may sweep it.

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
