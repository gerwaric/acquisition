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

### Milestone 1 — Delta-native worker (shipped July 17, 2026 — PR #162)

**Post-M1 status (July 23, 2026).** The network redesign's phase 5
(PR #175) subsequently rewrote the worker onto coroutine batch
submission (`network-redesign.md`, D6). Commit 1's generation-tag
mechanism was deleted along with the worker queue and
`SubmitNextItemRequest`: update identity is now the per-update
`std::stop_token` under the post-await invariant. Batch submission
puts several fetches in flight at once, and an abort leaves them
there — they resolve `Canceled` as accounted stopped siblings. F28's
misattribution stays impossible not because nothing is outstanding,
but because every fetch carries its update's captured stop token,
every consumer checks it immediately after its await before touching
worker state, and each future is completed exactly once by the pump.
Commit 2's semantics — atomic per-reply replacement, list
reconciliation, selection-only fetching (F55 revised / F61),
rebase-on-success, no emit on terminal failure — are preserved by the
rewrite. This section remains the record of what M1 shipped; read D6
for the worker's current shape before writing anything against it.

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
  refresh, at zero extra API calls; deliberate behavior change, final
  wording in the release notes below); tabs absent from the fresh list
  are removed along with their items — in memory as the list arrives,
  and durably: the datastore reconciles its rows against fresh top-level
  lists and against Map/Unique parent replies (F53, fixed in the
  follow-up PR).
- **Update modes unify**: `All` becomes "selection = every tab",
  `TabsOnly` becomes "selection with contents off", `Checked`/`Selected`
  are the general case. `RemoveUpdatingTabs`, `RemoveUpdatingItems`,
  `m_first_stash_request_index`, and `m_first_character_request_name` are
  deleted. A partial refresh (`Checked`/`Selected`) fetches **only the
  tabs in its selection**. A tab or character newly discovered in a fresh
  list is added to the tab list (its metadata surfaces in the UI) but is
  left unfetched until a full refresh — or an explicit selection — picks
  it up; a `TabsOnly` refresh fetches no contents at all. The children of
  a selected Map/Unique parent are still fetched, because they are
  discovered in the parent's reply and ride its fetch decision — they
  never appear in a top-level list, so nothing else would reach them.

  **F55, revised (supersedes the original F55 always-fetch rule).** The
  original design auto-fetched any not-previously-known tab whenever it
  appeared in a list the selection required, keying "new" on whether the
  tab's *contents* were already cached. That conflated a genuinely new tab
  with an existing tab whose contents were merely cold, so a partial
  refresh ballooned into a full one whenever the contents cache was cold —
  a fresh install, an upgrade from an older Acquisition (whose contents
  live in a different datastore that is never migrated), a datastore
  that had only ever stored tab lists, or — since 0.18's payload
  versioning — a `json_version` invalidation: version-mismatched rows
  keep tab metadata but yield no contents (pinned by
  `staleRowsKeepMetadataButYieldNoJson`), and the 3.29 wire-format
  change puts every upgrader in exactly this listed-but-cold state;
  contents stay cold until a full refresh or an explicit selection
  refills them. The revised rule keys purely on the
  selection: partial refreshes never fetch outside their selection, so the
  whole `m_contents_known` apparatus (parse-time seeding, the never-consume
  failure edge, the Map/Unique deferred-completion accounting) is deleted.
  Cost of the revision: a newly created tab waits for a full refresh (or an
  explicit selection) instead of filling in on the next partial refresh.
- **Failure semantics unchanged at the boundary**: no `ItemsRefreshed`
  emit on terminal failure — but now that's safe, because `m_items` is
  never left culled. Emit-on-failure / partial-application policy is a
  deliberate non-goal (below). One nuance: list-reconciliation effects
  (fresh tab metadata, deleted tabs dropped with their items) apply to
  worker memory as the lists arrive and are kept even if the update then
  fails terminally — unpublished until the next successful emit. "A
  failed update loses nothing" means nothing *the server still has*.
  Surviving items' embedded locations, by contrast, are rebased onto the
  fresh tab metadata only in `FinishUpdate`, because the emitted `Items`
  share `Item` objects with `ItemsManager` and the UI — shared state may
  only be mutated at the moment an emit rebuilds everything downstream.

Validation (complete, July 2026): the offline fake-network harness covers
the worker's update cycle, the live network-kill ran July 16, and the
recorded missing-item repro was retired as moot once the destructive cull
path was deleted — see the F28 ledger entry.

**M1 release notes (final user-facing wording).** Two deliberate
behavior changes ship with M1; this is the source text for the release
(copy into the PR body / release entry):

- *Stash tab renames and moves now show up on any refresh.* Renaming,
  moving, or recoloring a stash tab in the game is reflected by the next
  refresh of any kind, without refetching the tab's contents. Previously
  the old name could persist until that specific tab was refreshed.
- *Newly created tabs and characters show up right away and fill in on a
  full refresh.* A stash tab or character created since your last refresh
  appears in the tab list as soon as any refresh consults the
  corresponding list, and its contents are fetched by the next full
  refresh — or immediately, if you select it. A partial refresh (refresh
  selected or refresh checked) fetches only the tabs you asked for, so it
  never turns into a full refresh just because some tabs have not been
  fetched yet.

The second note originally promised the opposite — that a partial refresh
would auto-fetch newly created tabs. That was revised (see "F55, revised"
above): keying "new" on cached contents made a partial refresh balloon
into a full one on a cold contents cache, so the auto-fetch was dropped in
favor of fetching strictly the selection.

### Milestone 2 — Streaming refresh signal (spec frozen at revision 9, July 29, 2026: `items-pipeline-m2.md`; implementation next)

Surface per-tab progress without triggering the snapshot cascade:

- A new delta signal from the worker (working name: `TabRefreshed`),
  emitted after each atomic replace, carrying the complete
  pipeline-native `Items` replacement for one fetch source, keyed by
  (location type, fetch-source id) — a location alone gives
  `ItemsManager` nothing to append; see the delta-shape input below.
  Separate from the persistence signals
  (`stashReceived`/`characterReceived`), which carry API-domain
  payloads and fire before the atomic replace.
- `ItemsManager` applies the delta to its copy and re-emits a light
  signal. It must **not** run whole-collection work per delta —
  buyout migration, the whole-collection auto-buyout and propagation
  passes, shop expiry, and shop submission stay on the final
  `ItemsRefreshed`, whose contract does not change. Whether
  item-local scoped pricing runs per delta is a spec decision (see
  the buyout-scoping input below).
- `MainWindow` applies deltas conservatively: coalesced refiltering
  scoped to the current search with a stated maximum staleness (see
  the freshness input below), background captions updated cheaply or
  deferred. The M2 spec must resolve how disruptively the
  visible view may update — a model reset every 20 seconds with restore
  machinery is not acceptable as a steady state; scroll and selection must
  survive a background tab landing.

Hard constraints for the M2 spec, from the cascade recon: no per-delta
forum submission, no per-delta whole-collection scans, no per-delta
uncoalesced model reset.

A fourth constraint, from the M1 review (July 2026): **the
rebase-on-success design does not compose with streaming deltas.** M1
defers `RebaseItemLocations` to the successful `FinishUpdate` path
precisely because publication is single-shot — shared `Item` objects may
only be mutated when an emit immediately rebuilds everything downstream.
Streaming publication breaks that assumption: once `TabRefreshed`
consumers hold references mid-update, the spec must choose a rebase
point. Either per-delta consumers tolerate stale embedded tab metadata
until the final `ItemsRefreshed` (and the M2 UI must not render anything
that would expose the mismatch), or the rebase moves earlier and the
failed-update-mutates-published-state problem M1 solved returns and needs
a new answer.

**Inputs accumulated since this sketch was written (July 2026), for
the M2 spec:**

- **The per-tab signal surface partly exists.** The worker already
  emits typed per-tab signals wired to the datastore repos:
  `stashReceived`, `characterReceived`, `stashListReplaced`,
  `characterListReplaced`, `stashChildrenReplaced`
  (`itemsmanagerworker.h`). These are persistence signals, not
  presentation deltas — see the delta-shape input below for why the
  M2 signal is expected to be separate; the spec should confirm that
  split and keep the two lanes from drifting.
- **Explicit M2 deferrals from the network redesign (D6):**
  whole-update replacement/coalescing (an `Update()` during an active
  update is refused today); reprioritization (flagged as *not* cheap
  later — the stop token is per-update, so per-entry cancellation
  does not exist); per-tab retry and durable progress; UI-side
  coalescing of the batch-submit `QueueUpdated` burst. These are
  decisions the M2 spec must make, not scope it must implement:
  whole-update replacement and reprioritization are not required to
  ship streaming and could expand M2 considerably. Per-tab failure
  policy is the most load-bearing of the set, and it must distinguish
  deterministic failures (parse/protocol — retrying is futile; the
  3.29 `flags: []` incident showed one deterministic parse failure
  aborting whole updates) from transient ones.
- **New behavioral fact from phase 5:** the first terminal failure
  returns the worker to idle immediately, so a new update may be
  active while the stopped update's canceled stragglers are still
  settling. Stragglers apply nothing (post-await invariant), but the
  delta-signal design must tolerate the overlap.
- **F62 (implemented July 28, 2026):** raw reply bytes enter the
  datastore through the persistence lane only — the facade returns the
  parsed payload plus the raw sub-object bytes
  (`poe::StashPayload`/`poe::CharacterPayload`), and
  `stashReceived`/`characterReceived` carry the bytes opaquely to the
  repos (full decision in the F62 ledger entry,
  `docs/cleanup/findings.md`). Network-redesign D7 now reads "nothing
  above the boundary *interprets* bytes"; the M2 spec's D1 sequencing
  precondition is satisfied. Consequence for M2: deltas never carry
  wire bytes or `poe::*` API objects — the persistence lane owns
  those — but they **must** carry pipeline-native items, per the next
  input.
- **The delta must carry items, not just a location (second-opinion
  review, July 2026).** `ItemsManager` copies the worker's vector
  only at the final snapshot (`OnItemsRefreshed`); newly parsed items
  exist nowhere downstream until then. A location-only signal tells
  `ItemsManager` what to erase and gives it nothing to append. A
  streaming delta therefore carries the complete pipeline-native
  `Items` replacement for one fetch source (possibly empty — an
  emptied fetch source, never a deletion), keyed explicitly by (location type,
  fetch-source id): `fetch_id()` is deliberately excluded from
  `ItemLocation` comparison, so anything keyed on location equality
  would collapse Map/Unique child replacements into one. The signal
  is separate from `stashReceived`/`characterReceived`, which carry
  API-domain payloads and fire before the worker's atomic
  replacement. The spec must also pick the user-visible atomic unit
  for special tabs: per fetch source (finer progress, but a parent
  bucket can transiently mix old and new child data) or per display
  tab (coarser, publishes the parent once its enabled children
  settle).
- **A typed terminal event and an explicit no-rollback policy.**
  Deltas applied before a later terminal failure stay applied in
  memory and datastore by design (pinned:
  `appliedReplySurvivesLaterFailureInMemoryAndDatastore`). Once
  deltas stream, `StatusUpdate` strings are not a semantic boundary:
  downstream consumers need an explicit completed/failed outcome
  event, and the spec must state the no-rollback policy it implies.
  (This is the concrete form of the emit-on-failure non-goal's
  "revisit at M2".)
- **Buyout scoping per delta.** The constraint is "no
  whole-collection buyout pass per delta", not "no pricing until the
  final emit": streamed items that sit visibly unpriced for hours
  (auto-item and inherited tab pricing) are a real cost at the
  hours-long-refresh scale, so item-local scoped pricing per delta is
  a design option the spec should weigh. Mind `PropagateTabBuyouts`'s
  global `ClearRefreshLocks` either way.
- **State the persistence/presentation split.** Content replacements
  stream; other worker mutations (list reconciliation's deletions and
  renames, the location rebase) may stay final-only. "ItemsManager
  applies the same delta" is under-specified — the spec must say
  exactly which worker mutations are mirrored per delta and which
  remain snapshot-boundary effects. In particular: whether a deleted
  tab is expressed as a separate list/removal delta or stays a
  snapshot-boundary effect — an empty content replacement means an
  emptied fetch source, never a deletion.
- **A freshness bound, not just coalescing.** A resetting trailing
  debounce can starve under steady one-reply-per-20-seconds
  arrivals. The coalescing contract needs both halves stated: no
  uncoalesced per-delta reset (already a constraint above) and a
  maximum staleness — a throttle that guarantees the visible view is
  never more than a stated interval behind the applied state.

### Milestone 3 — Delta-native items model (spec drafted July 30, 2026: `items-pipeline-m3.md`, revision 3, in review — rounds 1–2 incorporated)

The "profile before choosing levers" obligation below was discharged
July 30, 2026 by the S1-M3 sort-profiling spike
(`m3-sort-profile-result.md`; throwaway branch `spike/m3-sort-profile`,
never merged) and the same-day lever hold point, whose decisions the
spec records. The inputs below are kept as written; the spec's
traceability table maps each to its consuming decision.

Make Layer 3 consume deltas natively, eliminating the full reset:

- The bucket architecture (bucket-per-tab, items under buckets) already
  matches the delta shape. A tab delta becomes fine-grained model ops
  (`begin/endInsertRows` / `begin/endRemoveRows` scoped to one bucket, or
  a one-bucket replace) instead of `beginResetModel()`.
- The "By Item" view's single flat bucket needs a sorted merge per delta
  rather than an append — the one structure that fights the delta shape.
- Pricing semantics are settled by the M2 spec (the buyout-scoping
  input); M3 inherits them — its concern is the fine-grained model
  operations, not pricing.
- Success criterion: refreshing one tab leaves the expansion, selection,
  and scroll state of everything else untouched, with no restore
  machinery involved. A "full refresh" is then just N deltas — no special
  destructive path left in the pipeline.

Inputs accumulated for the M3 spec (from the S1-M2 spike, July 29,
2026 — evidence in `s1-m2-spike-result.md`):

- **The reset's dominant cost is the post-reset whole-model re-sort**,
  not the filter or the model reset: ~0.4 s of a ~0.46 s reset at
  101k items, ~5.0 s of ~5.4 s at ~976k (Release, worst-case
  unfiltered search). Bucket-scoped ops retire this for the
  *streaming* path by construction, but a **user-initiated full
  refilter still rebuilds and re-sorts everything** and pays the same
  cost at scale. Levers for M3 to weigh, all model-layer: precomputed
  sort keys (the string/QVariant-heavy column comparators are the
  suspected driver — the spike timed the sort as a whole, not the
  comparators; profile before choosing), lazily sorting only
  expanded/visible buckets, born-sorted buckets (filtering from a
  pre-sorted master).
- M2's R6-3 fidelity machinery — stable `(type, id)` expansion keys
  and stable-identity reselection — is collectively negligible at
  both scales (bounded by the few-ms residual after filter and sort;
  not individually timed) and carries forward as M3's bucket keying.

## Non-goals

- **Emit-on-failure / partial-application policy.** The worker keeps
  no-emit-on-terminal-failure through M1. Revisit once M2's streaming
  makes "what the user sees during/after a failed refresh" a designed
  surface rather than an accident.
- **Rate limiter redesign.** ~~Preserve the one-HEAD-at-a-time property
  (F5) and the existing retry semantics.~~ **Amended July 18, 2026:**
  the redesign is now in scope, specified in
  `docs/design/network-redesign.md` (which preserves the F5 property
  deliberately via its gate). That spec also answers the
  where-does-scheduling-live question this plan deferred to M2.
  **Complete July 23, 2026:** phases 0–5 merged to master (PR #175);
  the network layer is settled ground for the M2 spec, not concurrent
  work.
- **Datastore redesign / delta persistence.** Per-tab persistence
  already works and this plan does not restructure it. (The original
  "no schema changes" wording is retired: 0.18 added the
  `json_version` payload-versioning column, and F62 changed what the
  persistence lane carries (raw wire bytes since July 28, 2026) — both
  independent correctness work, neither driven by this plan.)
- **UI/UX redesign** beyond refresh behavior; no theming, packaging, or
  `Item` class rework.

## Working rules

Carried over from the cleanup, which they served well:

1. **Doc-first.** Each milestone has an implementation-grade spec reviewed
   and frozen before its production code begins — M1's is this document;
   M2 and M3 get their own before they start. A spec may name a bounded
   pre-freeze evidence spike whose prototype lives on a dedicated
   non-production branch or in an isolated harness and is discarded or left
   unmerged; the spec records the result and freezes before any production
   implementation begins. M2 names one such exception: the S1-M2 UX spike
   (`items-pipeline-m2.md`, D9/R3-4; ran July 29, 2026 — outcome (a),
   S = 60 s, recorded in the M2 spec's revision 9, which froze it). The
   M2-M2 storage/frame measurement
   runs instead as the first checkpoint of M2's production implementation
   (R7-3; the spec freezes a determinate conditional for D3's storage).
2. **Every commit compiles and passes `ctest`.**
3. **New problems go to the register** (`docs/cleanup/findings.md`), not
   inline fixes, unless required for the milestone to proceed.
4. **Staleness preambles.** M2/M3 specs must state the codebase
   assumptions they were written against; re-verify before following
   stale sections.
