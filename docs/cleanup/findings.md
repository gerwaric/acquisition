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
plausible live path was the legacy importer (`LegacyDataStore` reads
`db_version` from the old database) — but `LegacyDataStore` has no
callers outside `src/legacy/` (verified July 17 during the F55 work):
the importer is not wired into the application at all, so that path
cannot arm the migration today. Whether any other path can put a
`db_version < 5` into a live store remains unverified; check before
sizing the fix.

Fix shape: make the migration write through the repo (save the id-keyed
row, delete the hash-keyed one) inside `MigrateItem`, or have
`MigrateBuyouts` flush the affected entries after the loop. Related: F22
(dual persistence), F51 ledger entry (do not rekey `GetLegacyHash` — a
correct future v4→v5 migration depends on it).

### F56. Single-lane item-request serialization starves the character policy — Confirmed

Found July 17, 2026, from Tom's live observation that character requests
stall while the stash rate limit is pacing. `ea9dd95` (in v0.17.0) moved
request scheduling out of the rate limiter to fix the abort-after-failure
desync: `ItemsManagerWorker` now holds one FIFO (`m_queue`) mixing stash
and character requests and `SubmitNextItemRequest` keeps at most one in
flight, submitting the next only when a reply lands. The queue is
stashes-first by construction, and the two list requests are chained
else-if as well — so while the stash policy paces (~20s/tab at
saturation), the character policy's manager sits idle with its requests
still in the worker's FIFO, never submitted. Refresh time degrades from
max(stash, character) to stash + character. The per-policy managers were
designed for exactly this parallelism; M1's generation tag explicitly
anticipated re-parallelization ("protects any future re-parallelization")
and the failure paths hold under it (a failure snaps counters and goes
Idle; the other lane's late reply is swallowed by `DiscardIfStale`).

Fix shape (proposed July 17; **paused same day pending the network
ground-truth research phase** — see the note at the end of this entry):
keep worker-side queueing but one queue per `ItemLocationType` with one
in-flight each, and submit the two list requests concurrently again — no
rate limiter API changes, abort stays trivial (≤2 stale replies). The
larger question —
worker-owned scheduling with the limiter as a pure gatekeeper vs. queueing
returning to the limiter with cancellation APIs — is deliberately deferred
to the M2 spec, which must state where scheduling lives (priorities,
per-tab retry, durable progress all pull toward the worker) and formally
amend the "no rate limiter redesign" non-goal if it chooses the
gatekeeper. Testable today at worker level with `FakeRateLimiter`: assert
character requests are submitted while stash replies are outstanding.

Paused July 17 before implementation: all of F56–F59's *fix shapes* (the
findings themselves stand — they describe code as it is) rest on
code-derived premises about the API that have not been checked against
the API's actual contract: that stash and character policies are
independent and parallel-safe, that policy topology is stable and fully
discoverable via HEAD headers, that no account/IP-level limit sits above
the per-policy ones, that 429s are rare and cleanly retryable. Tom holds
unshared knowledge about the API that may contradict some of these — it
is even possible the accidental serialization is conservatively correct
and parallelism is the mistake. A dedicated research phase (official API
docs, real-world logs, Tom's knowledge) produces a network ground-truth
document first; fix shapes and the F56/F57 sequencing get re-derived
against it. **Re-derivation complete July 18, 2026:** the superseding
fix shapes for F56–F59 are specified in
`docs/design/network-redesign.md` (accepted July 19, 2026, revision 7;
frozen for implementation) — queueing
returns to the limiter with QFuture-based cancellation, policy managers
become coroutine pumps, and a global gate carries the F5/N18 and F58
obligations deliberately. Note the F5/HEAD interaction flagged during this analysis:
today's serialization incidentally guarantees first-use HEAD setups
never overlap; any parallelism must re-establish that property
deliberately — done in phase 3, where HEAD exclusivity became a gate
property. Status: spec phases 0–3 are implemented and merged (phase 3,
July 20, 2026: the pump, the gate wiring, async HEAD setup — F57, F58,
and F5-modernization resolved); F56 itself resolves in phase 5's worker
rewrite, after phase 4's `QFuture` boundary.

### F59. `RateLimitedReply` ownership contract is contradictory — Confirmed

Found July 17, 2026, during the F56 investigation.
`RateLimiter::Submit`'s comment says the caller is responsible for
freeing the `RateLimitedReply` with `deleteLater()` after `complete`;
meanwhile the pump's entry owns the same object via
`std::unique_ptr` and destroys it synchronously right after the
completion emit.
Callers (`ItemsManagerWorker` handlers, `DiscardIfStale`) do call
`deleteLater()` — benign today only because `complete` is a direct
same-thread connection (handlers finish before the unique_ptr delete)
and a QObject destructor cancels its own pending deferred delete. Any
reordering, queued connection, or threading change turns this into a
use-after-free. Status after network-redesign phase 3 (July 20, 2026):
the pump preserves these semantics deliberately (pinned in
`tst_ratelimitmanager.cpp` as the kept F59 behavior); the resolution is
phase 4's `QFuture` boundary, which deletes `RateLimitedReply` outright
so the contradiction stops being expressible (spec D1).

### F60. The legacy stash-index request has no transfer timeout — Confirmed

Found July 19, 2026, during the network-redesign round-5 review
(R5-3 in `docs/design/network-redesign-reviews.md`). `Shop::UpdateStashIndex`
builds its `QNetworkRequest` bare — no `setTransferTimeout` — unlike
the OAuth API builders (`poe::MakeApiRequest` sets 10 s) and the
forum-thread calls (300 s). A stalled legacy GET, or the endpoint's
HEAD probe (which inherits the request), therefore has no client-side
bound at all and can hang until the OS gives up on the connection,
leaving the shop update waiting on a reply that never finishes.
Under the network redesign this would also hold a gate permit
indefinitely — with a HEAD waiting under writer preference, the whole
hub stops — which is why the redesign makes the timeout a facade-owned
invariant (spec D5/D7, test over every builder). Fix shape: the facade
closes this by construction; if anything touches `UpdateStashIndex`
before the facade lands, add the timeout there directly.

---

## Standing constraints and lessons

Rules distilled from resolved findings that remain binding on future work.
The F-numbers refer to the ledger below.

- **F5 — one HEAD at a time.** Users got Cloudflare-blocked when HEAD
  requests flooded, so at most one HEAD may ever be in flight. Amended
  by network-redesign phase 3 (July 20, 2026, per spec D4/D5): the
  property is now enforced at the gate — a HEAD probe holds the gate's
  exclusive permit, so concurrent endpoint setups serialize there — and
  the old nested-event-loop implementation is deleted. Still binding:
  never call `RateLimiter::Submit` off the main thread (a `Q_ASSERT`
  enforces this), and any change to the gate's HEAD exclusivity must
  preserve one-HEAD-at-a-time deliberately.
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
| F49 | Folder children suspected of being fetched twice via two paths | Closed by live observation, July 2026: the paths are complementary in the live API — folder children arrive only via the stash list (Standard, 16 child lists; the two individually fetched folders returned no `children` and no items), map/unique children only via the individual reply (Mirage, 73 children). The `OnStashReceived` tripwire warning stays in the code as a permanent guard should the API ever change. July 17 amendment: the warning's "fetched twice this update" claim can be false — during a partial refresh a known-but-unselected child is never queued from the stash list, so the parent path would be its only fetch. Reworded to "may be fetched twice" in the F53/F55 follow-up PR |
| F48 | Character-list skip-check compared names against an id-keyed index (never matched), so a partial update re-added and re-fetched every character in the league, duplicating their tab entries and items | Found and fixed, items-pipeline M1 (character entries rebuilt from the fresh list, keyed by id) |
| F51 | Unnamed stash tabs (~30 on the validating account, real in-game data) collapse the label component of the legacy item-buyout hash, suspected of shadowing item buyouts across tabs | Reframed and closed, July 2026: active item buyouts key on the API item id (`BuyoutManager::Set`/`Get` use `item.id()`); the label-based `hash_v4` is consumed only by the one-time v4→v5 migration, where colliding tabs made that migration ambiguous — an accepted legacy quirk. Do not rekey `GetLegacyHash`: it would break future v4→v5 migrations without improving live lookups |
| F52 | `PropagateTabBuyouts` issued one no-op buyout DELETE per item on every refresh (~17k per refresh on an 18.5k-item account) | Fixed, PR #163: the clear path touches the repo only when the in-memory map holds an entry; per review, `removeItemBuyout`/`removeLocationBuyout` report success and the map entry is erased only afterward, so a failed DELETE is retried on the next clear (pinned by a `BEFORE DELETE RAISE(FAIL)` trigger test). Accepted, test-pinned: a row written behind the manager's back survives an in-session clear and heals at the next `Load()`; save-path failures are still discarded by the signal connection (deliberate asymmetry). Drift note: `Compress*` drifts the map only toward orphan repo rows, which the guard leaves alone; `MigrateItem` rekeys in memory only and so drifts both ways at once (old row orphaned, new key rowless — see F54); a failed save also leaves a rowless map entry; clearing a rowless entry is healed by a zero-row DELETE. Standing M2 constraint: the per-tab delta path must scope buyout propagation to the delta, not rerun the loop per tab reply |
| F53 | Deleted stash tabs and characters resurrected from the cache at restart: the repos only upserted listed rows and could not even express "everything was deleted" (empty lists returned early) | Fixed, F53/F55 follow-up PR: authoritative-list signals (`stashListReplaced`/`characterListReplaced`, emitted only for fresh top-level lists — never for `ProcessTab`'s folder-children re-emits) drive `reconcileStashList`/`reconcileCharacterList`, deleting rows absent from the recursively flattened list (realm-wide for characters, matching the endpoint) with empty lists handled; children of surviving Map/Unique parents are preserved and reconciled by `stashChildrenReplaced` on the parent's reply instead — scoped to Map/Unique parents only, because live folder replies carry no children (F49) and keying off them would wipe legitimate child rows. With child fetching disabled the parent reply deletes cached child rows, so re-enabling the setting refetches instead of showing stale cache (documented policy). Pinned at repo level (`tst_reconcile`) and end-to-end through the fake network |
| F55 | A terminal failure between list receipt and a new tab's first fetch consumed the tab's newness durably (metadata persists at list receipt), so later partial refreshes published the tab empty — release-blocking for M1's always-fetch note | Fixed, F53/F55 follow-up PR: the always-fetch decision keys on a contents-known set — seeded in `ParseCachedItems` from rows whose stash/character json was actually saved, extended on successful replies — instead of list membership (`previously_known` removed). No schema change: the `listed_at` vs `json_fetched_at`/`json_data` split already existed, and no path writes json without its timestamp (`LegacyDataStore` is unwired). Rejected: skipping the list-receipt metadata save (regresses the absorbed-F15 metadata refresh and misses the in-session case). Pinned by `failedFirstFetchDoesNotConsumeNewness` (the ledger-specified scenario) and `listedButNeverFetchedTabIsFetchedOnNextUpdate` (the restart shape). Review follow-up (July 17): a Map/Unique parent counts as contents-known only once every enabled child fetch has landed — completion is deferred to the last child reply in-session, and a cached parent whose saved reply records children with a missing child row stays "new" at startup (special children never appear in a top-level list, so nothing else would retry them); pinned by `failedChildFetchKeepsParentNew` and `cachedParentWithMissingChildRowStaysNew`. Round 2: starting a child-fetch cycle also *evicts* an already-known parent from contents-known (chosen over per-child-id completeness tracking: eviction is uniformly conservative — worst case one redundant refetch after a mid-cycle failure — while stale in-memory child known-ness could re-strand a child after rows were cleared under a disabled setting), pinned by `knownParentWithNewFailedChildIsRetried`; and the `ParseCachedItems` settings reads were hoisted to the main thread (the parser thread must not touch the shared `QSettings` instance the UI writes — reentrant, not thread-safe). Known residual, accepted: re-enabling `get_map_stashes`/`get_unique_stashes` mid-session leaves the parent known until it is next fetched (full refresh or selection) or the next restart's seed check. Release-note wording narrowed to "any content refresh": `TabsOnly` records a new tab without fetching it and the next content refresh picks it up |
| F57 | A 429 retry destroyed the caller's `RateLimitedReply`, dropped the retried completion, and wedged the update until restart (reproduced offline by the phase-1 harness) | Fixed, network-redesign phase 3 (July 20, 2026): the pump retries invisibly inside the drain loop — bounded attempts, padded deadline, permit-free sleep — and the caller sees exactly one final completion; the phase-1 wedge pin flipped to `retry429CompletesCallerExactlyOnce` |
| F58 | The minimum-send-interval spacing was dead code (`last_send` never assigned) | Fixed, network-redesign phase 3 (July 20, 2026): the dead block deleted with `ActivateRequest`; the intent is implemented deliberately at the right scope as the gate's `MIN_SEND_SPACING` floor (250 ms across everything the hub sends, measured from dispatch stamps — spec D5), pinned exactly on the injected clock |
