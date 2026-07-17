# Items Pipeline Redesign

## Purpose and context

This plan succeeds the July 2026 interior design cleanup, which is complete
(its seven phase documents are retired; see git history). The cleanup's
findings register remains live at `docs/cleanup/findings.md` and this plan
uses its F-numbers.

The subject here is how item data flows from the Path of Exile API to the
UI. The current design dates to 2014 and is **snapshot-oriented**: every
layer speaks "here is the new world" (the full item vector) rather than
"this tab changed". That one contract decision forces every cost and most
of the correctness problems below; this plan replaces it, one shippable
milestone at a time, with a **delta-native** pipeline.

### Why now

- **Correctness (F28).** An update that fails mid-flight leaves stale
  in-flight replies connected to the next update's handlers, and — the more
  load-bearing hole — the update *begins destructively*:
  `ItemsManagerWorker::Update()` culls the updating tabs' items from
  `m_items` up front and re-adds them only as replies land. A terminal
  failure returns to idle without emitting, silently leaving `m_items`
  short; the next successful partial refresh then publishes the short list.
  This matches the "single item missing until restart" symptom recorded
  under F28.
- **Scale.** Users with hundreds to thousands of stash tabs exist (one
  account dates to 2014 with over two thousand tabs). Under the API's rate
  limit policies, large refreshes are paced at roughly one tab per 20
  seconds — a full refresh takes **hours**. At that timescale, refresh
  progress must be durable (a blip at hour ten must not discard hour one)
  and should be visible incrementally (the datastore already persists each
  tab as it arrives; memory and UI should too).
- **Cost.** The snapshot contract makes every `ItemsRefreshed` emit do
  whole-collection work at every layer (see the cascade below), ending in a
  full `beginResetModel()` that destroys view state which `MainWindow` then
  labors to restore. The cleanup fixed the worst symptoms (F23, F31, F32)
  but the design cost is structural.

### The snapshot cascade today

One `ItemsManagerWorker::ItemsRefreshed` emit currently triggers:

1. `ItemsManager::OnItemsRefreshed` (`itemsmanager.cpp`): full item-vector
   copy; an O(items) uncategorized-items scan that exists only for debug
   logging (F46); three whole-collection buyout passes
   (`ApplyAutoTabBuyouts`, `ApplyAutoItemBuyouts`, `PropagateTabBuyouts`).
2. `Application::OnItemsRefreshed` (`application.cpp`): currency snapshot,
   shop-data expiry, and (when enabled) a forum shop submission.
3. `MainWindow::OnItemsRefreshed` (`ui/mainwindow.cpp`):
   `Search::FilterItems()` for **every** search tab — each a full
   O(items × active filters) scan rebuilding all buckets — then
   `ModelViewRefresh()` for the current search, whose
   `ItemsModel::beginUpdate()` is literally `beginResetModel()`. The reset
   invalidates expansion, selection, and scroll state, which
   `RestoreViewExpansion` / `ReselectCurrentItem` / the resize coalescing
   then reconstruct.

No layer knows *what changed*, so no layer can do less.

## Direction

Change the contract between layers from snapshots to deltas ("tab X was
refreshed"), starting at the source. Three milestones, each independently
shippable and valuable, each specified doc-first before its implementation
begins:

### Milestone 1 — Delta-native worker (next PR)

Make `ItemsManagerWorker` know and control what changes, and make updates
non-destructive. Two commits:

**Commit 1 — update generation tag (F28 proper).** The worker keeps an
update generation counter, bumped by every `Update()`. Reply handlers
capture the generation they were submitted under and discard replies whose
generation is stale. Kills the misattribution hole outright. (Post-Phase-2
serialization — lists submitted else-if, item requests one at a time via
`SubmitNextItemRequest` — already narrows the window to at most one
outstanding reply, so this is partly insurance; it also protects any future
re-parallelization.)

**Commit 2 — atomic per-reply replacement.** The engine of the redesign:

- `ItemLocation` gains a **fetch-source id**: the id of the stash or
  character actually fetched. For children of MapStash/UniqueStash tabs
  this is the child's own id even though the display location stays the
  parent's. (Folder children are ordinary tabs: they arrive via the stash
  list and display under their own id — see the F49 ledger entry.)
  Excluded from `operator==`, `operator<`, and
  `GetLegacyHash()` so buyout keys and sort order are untouched.
  `ParseCachedItems` sets it too (the datastore keys child stashes by
  their own ids), so cached and live items agree.
- Every stash/character reply is an **atomic replace**: erase items whose
  fetch-source id matches the fetched id, parse, append — synchronously,
  in one event-loop slot. Nothing is culled before its replacement is in
  hand. A terminal failure at any point loses nothing already applied.
- **Tab-list reconciliation on list receipt** replaces the up-front tab
  cull: tabs present in the fresh stash/character list are upserted
  (metadata refreshed in place — this absorbs the F15 accepted-limitation
  sketch: renamed/moved tabs now get fresh names/colors/positions on any
  refresh, at zero extra API calls; deliberate behavior change,
  release-note it); tabs absent from the fresh list are removed along
  with their items — in memory, for the session: the datastore keeps
  their rows until F53 (follow-up PR) makes the deletion durable.
- **Update modes unify**: `All` becomes "selection = every tab",
  `TabsOnly` becomes "selection with contents off", `Checked`/`Selected`
  are the general case. `RemoveUpdatingTabs`, `RemoveUpdatingItems`,
  `m_first_stash_request_index`, and `m_first_character_request_name` are
  deleted.
- **Failure semantics unchanged at the boundary**: no `ItemsRefreshed`
  emit on terminal failure — but now that's safe, because `m_items` is
  never left culled. Emit-on-failure / partial-application policy is a
  deliberate non-goal (below).

Validation (complete, July 2026): the offline fake-network harness covers
the worker's update cycle, the live network-kill ran July 16, and the
recorded missing-item repro was retired as moot once the destructive cull
path was deleted — see the F28 ledger entry.

### Milestone 2 — Streaming refresh signal (next PR)

Surface per-tab progress without triggering the snapshot cascade:

- A new lightweight delta signal from the worker (working name:
  `TabRefreshed(location)`), emitted after each atomic replace.
- `ItemsManager` applies the same delta to its copy and re-emits a light
  signal. It must **not** run buyout migration, auto-buyouts, shop expiry,
  or shop submission per delta — those stay on the final `ItemsRefreshed`,
  whose contract does not change.
- `MainWindow` applies deltas conservatively: coalesced (timer-based)
  refiltering scoped to the current search, background captions updated
  cheaply or deferred. The M2 spec must resolve how disruptively the
  visible view may update — a model reset every 20 seconds with restore
  machinery is not acceptable as a steady state; scroll and selection must
  survive a background tab landing.

Hard constraints for the M2 spec, from the cascade recon: no per-delta
forum submission, no per-delta whole-collection scans, no per-delta
uncoalesced model reset.

### Milestone 3 — Delta-native items model (later)

Make Layer 3 consume deltas natively, eliminating the full reset:

- The bucket architecture (bucket-per-tab, items under buckets) already
  matches the delta shape. A tab delta becomes fine-grained model ops
  (`begin/endInsertRows` / `begin/endRemoveRows` scoped to one bucket, or
  a one-bucket replace) instead of `beginResetModel()`.
- The "By Item" view's single flat bucket needs a sorted merge per delta
  rather than an append — the one structure that fights the delta shape.
- Buyout passes scope to the delta's items (mind
  `PropagateTabBuyouts`'s global `ClearRefreshLocks`).
- Success criterion: refreshing one tab leaves the expansion, selection,
  and scroll state of everything else untouched, with no restore
  machinery involved. A "full refresh" is then just N deltas — no special
  destructive path left in the pipeline.

## Non-goals

- **Emit-on-failure / partial-application policy.** The worker keeps
  no-emit-on-terminal-failure through M1. Revisit once M2's streaming
  makes "what the user sees during/after a failed refresh" a designed
  surface rather than an accident.
- **Rate limiter redesign.** Preserve the one-HEAD-at-a-time property
  (F5) and the existing retry semantics.
- **Datastore schema changes.** Per-tab persistence already works.
- **UI/UX redesign** beyond refresh behavior; no theming, packaging, or
  `Item` class rework.

## Working rules

Carried over from the cleanup, which they served well:

1. **Doc-first.** Each milestone has an implementation-grade spec reviewed
   before its code begins — M1's is this document; M2 and M3 get their own
   before they start.
2. **Every commit compiles and passes `ctest`.**
3. **New problems go to the register** (`docs/cleanup/findings.md`), not
   inline fixes, unless required for the milestone to proceed.
4. **Staleness preambles.** M2/M3 specs must state the codebase
   assumptions they were written against; re-verify before following
   stale sections.
