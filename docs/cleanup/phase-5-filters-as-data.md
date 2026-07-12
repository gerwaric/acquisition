# Phase 5: Filters as Data + Matching

> **Status.** Revised to implementation grade in July 2026, verified against
> the post-Phase-4 codebase (branch `phase-4-search-decoupling`, after
> commit 6beb539), then hardened by a second adversarial review pass (which
> surfaced F36 and the catalog-ownership, state-construction, transitional,
> and build-gate specifications below). The original design-intent version
> assumed characterization coverage that turned out not to exist (see
> "Verified current state"); this revision replaces it. If implementation
> starts after further structural changes to `filters.{h,cpp}`,
> `modsfilter.{h,cpp}`, `search.{h,cpp}`, or
> `MainWindow::InitializeSearchForm`, re-verify that section first (plan
> working rule 5).

## Goal

Separate what a filter *is* (definition + matching, core, headless-testable)
from how it is *rendered* (widgets, UI). After this phase, filter matching
logic compiles without any widget headers — build-enforced, not just
grep-checked — and the `FilterData` grab-bag is replaced by typed per-search
state. Findings addressed: F19 (widgets-plus-logic), F33 (shared activity
flags mis-filter background searches), F35 (stale socket-color text leaks
across searches), F36 (mods filter form-sync quirks).

## Non-Goals

- No visual or behavioral changes to the search form: same controls, same
  layout, same debounce/immediate semantics (the mapping preserved by Phase
  1/F8 must survive verbatim — see the hazards table). Three deliberate,
  documented exceptions: the F33, F35, and F36 correctness fixes.
- No change to which filters exist or how they match. Characterization-test
  assertions are the gate and must pass unmodified except (a) mechanical API
  renames and (b) the step-0 tests that deliberately pin F33/F35/F36 wrong
  behavior, which flip when their fix lands.
- `SearchComboBox` stays a widget in `src/ui/`.
- No persistence of searches across restarts. State must be
  serializable-in-principle (plain values), but no save/load is added.

## Verified current state (July 2026)

What Phases 0–4 actually left behind, including corrections to the original
version of this doc:

- `FilterCallbacks` (`filters.h`) exists as assumed: `receiver` +
  `onChanged` (immediate → `MainWindow::OnSearchFormChange`) +
  `onChangedDelayed` (→ `OnDelayedSearchFormChange`, a single-shot
  `m_delayed_search_form_change` timer). Built once in
  `MainWindow::InitializeSearchForm()`.
- `Search` is `QTreeView`-free (Phase 4). It owns
  `std::vector<std::unique_ptr<FilterData>>` (index-aligned with
  `MainWindow::m_filters`), the `ItemsModel`, buckets, expansion headers,
  view mode, and sort. `Search::Activate` = `FromForm()` + `FilterItems()`.
- F33 confirmed in code: `Filter::m_active` is on the shared `Filter`
  object; `Search::FilterItems` trusts `filter->filter()->IsActive()` for
  its *own* data; `MainWindow::OnItemsRefreshed` filters every background
  search with the current search's flags.
- **Stale:** the original doc claimed "Phase 0 `tst_search` tab-switching
  cases" guard the `ToForm`/`FromForm` save/restore cycle. **They do not
  exist.** `tst_search.cpp` has only `bucketConstruction` and
  `nameFilterMembership`. Step 0 builds this safety net.
- **Stale:** characterization coverage in `tst_filters.cpp` is name,
  min-max, socket/link colors, one boolean (corrupted), rarity. Untested:
  tab filter, category (including the `<any>` sentinel), every other
  boolean predicate (altart, priced, unidentified, influenced, crafted,
  enchanted, fractured, split, synthesized, mutated),
  `DefaultPropertyFilter`, `RequiredStatFilter`, and `ModsFilter::Matches`.
- **Stale:** `SelectedMod` no longer uses `SearchComboBox` (commented out);
  it uses a plain `QComboBox` + `TokenAndFilterProxy` + `QCompleter` with a
  per-row 350 ms debounce for completer refresh. `TokenAndFilterProxy` is
  defined in `ui/searchcombobox.h`, and core `modsfilter.h` includes it — a
  live core→ui include this phase removes.
- The mods filter has three form-sync defects (F36): added rows are never
  saved until some other form change fires; rows rebuilt on tab switch
  don't display their saved mod name; the row container's visibility
  depends on add/delete history rather than row count. See the finding.
- The category choice list is runtime data: `GetItemCategories()` returns a
  reference to a singleton populated when RePoE loads
  (`repoe.cpp` → `InitItemClasses`); today's `QStringListModel` snapshots
  it at `InitializeSearchForm` time. The rarity list is a static core
  constant that doubles as matching vocabulary (the rarity matcher's switch
  compares against its entries), while category choices are
  presentation-only (the matcher is a substring test on
  `item->category()`).
- `tst_search.cpp`'s harness duplicates the entire 38-filter construction
  list from `InitializeSearchForm`. The catalog (D2 below) makes the
  canonical list definable once and removes this duplication.
- Signal-mapping quirk to preserve: the socket/link **color text boxes are
  on the immediate path** (`textEdited` → `onChanged`), unlike every other
  text box (debounced). Checkboxes use `clicked` (immediate, user-only).
  Combos use `currentIndexChanged` (debounced, **fires on programmatic
  changes too** — a tab switch that changes a combo schedules a debounced
  refresh; that is current behavior and stays).
- Build reality check: `acquisition_core` is a single static library that
  includes `ACQ_UI` and links `Qt6::Widgets` PUBLIC, so neither
  `QTEST_GUILESS_MAIN` nor an include-grep proves anything about widget
  linkage. D5 makes the boundary build-enforced.

## Design decisions (settled July 2026)

- **D1 — state is a variant of per-family structs, activity is derived.**
  `FilterState = std::variant<TextState, ComboState, MinMaxState,
  ColorsState, BoolState, ModsState>`; plain values, no heap, no
  inheritance. Each struct exposes `bool isActive() const` computed from its
  own values (`MinMaxState::isActive` = `min || max`, etc.). No stored
  activity flag anywhere — F33 becomes structurally impossible rather than
  patched. States are **never default-constructed as a bare variant** (that
  would always yield the first alternative): they are created by
  `MakeDefaultState(const FilterSpec&)`, which returns the alternative the
  spec's family requires. Matching dispatches via `std::get_if` on the
  expected alternative and asserts on mismatch instead of letting
  `std::bad_variant_access` throw at match time; `Search` construction
  asserts state/spec family agreement slot-by-slot. With `MakeDefaultState`
  as the only construction path these assertions should be unreachable —
  they exist to catch future misuse, not to paper over a live hazard.
- **D2 — the filter list is an immutable core-owned catalog with explicit
  ownership.** `BuildFilterCatalog(const BuyoutManager&)` returns the
  canonical ordered `FilterCatalog` (a wrapper over
  `std::vector<FilterSpec>`). **MainWindow owns the catalog**; `SearchForm`
  and every `Search` hold `const FilterCatalog&`. Lifetimes are nested and
  documented at the owner: MainWindow owns the searches, so the catalog
  outlives them; `BuyoutManager` is owned by `Application` and outlives
  MainWindow, which matters because the priced predicate captures it by
  reference. No `shared_ptr` — reference-with-documented-ownership is this
  codebase's idiom (`Search` already holds `BuyoutManager&`). Tests build
  their own catalog in the fixture, exactly as they build their own
  `BuyoutManager` today.
- **D3 — `ModsFilter` migrates to the full shared shape.** Its existing
  `ToForm` already rebuilds rows from data, so an adapter that rebuilds row
  widgets from `ModsState` is the natural port — and that shape fixes F36
  structurally (see step 6). If row management resists (budget: as much
  time as families 1–4 combined), fall back to a hybrid — matching + state
  in core, row widgets self-contained in `src/ui/` — and record a finding.
- **D4 — two test layers.** Matching assertions move verbatim onto state
  structs in a guiless `tst_filters` (`QTEST_GUILESS_MAIN`). Widget-adapter
  round-trip tests (state → widgets → state, plus the tab-switch cycle and
  the immediate/debounced wiring) live in a GUI-linked `tst_searchform`.
- **D5 — the widget-free boundary is build-enforced.** A new
  `acquisition_filters` static/object library compiles `src/filters/*` and
  links only the non-widget Qt modules its include closure needs (expected:
  `Qt6::Core`, `Qt6::Gui`, `Qt6::Network` via `Item`'s `ItemLocation`
  formatter; possibly `Qt6::Sql` via `buyoutmanager.h`) —
  **never `Qt6::Widgets`**. Any transitive widget header fails the build.
  `acquisition_core` (and thus the app and tests) consume this target. Note
  the gate proves *widget*-freedom; the "no `src/ui/` includes under
  `src/filters/`" rule is still checked by grep.

## Target shape

New core directory `src/filters/`, compiled as the `acquisition_filters`
target (D5; no widget includes anywhere in it):

- `filterstate.h` — the per-family state structs + `FilterState` variant +
  `isActive()` + `FilterState MakeDefaultState(const FilterSpec&)`. Values
  use `std::optional` where today a `*_filled` bool pairs with a value
  (`MinMaxState{std::optional<double> min, max}`,
  `ColorsState{std::optional<int> r, g, b}`,
  `ModsState{std::vector<ModRow>}` with
  `ModRow{QString mod; std::optional<double> min, max}`).
- `filterspec.h/.cpp` — `FilterSpec`: family payload (itself a variant),
  display caption, form group (`TopForm`, `Offense`, `Defense`, `Sockets`,
  `Requirements`, `Misc`, `MiscFlags`, `MiscFlags2`, `Mods`), and refresh
  mode (`Immediate`/`Debounced`). Family payloads absorb today's
  constructor args and virtuals:
  - text: item-string accessor (tab header vs `PrettyName`);
  - combo: match kind (category-contains vs rarity-switch), `<any>`
    sentinel, and a **choices provider** (`std::function<QStringList()>`) —
    for category it calls the core `GetItemCategories()`, for rarity it
    returns the static rarity list (which stays a core constant because the
    matcher's switch depends on its entries). The catalog stores no model
    pointers; `SearchForm` evaluates the provider when it builds its
    widgets and owns the resulting `QStringListModel`s, preserving today's
    snapshot-at-form-construction timing without freezing runtime RePoE
    data into the catalog;
  - min-max: `std::function<double(const Item&)>` value accessor +
    `std::function<bool(const Item&)>` presence accessor (covers
    `SimplePropertyFilter`, `DefaultPropertyFilter`, `RequiredStatFilter`,
    `ItemMethodFilter` lambdas, sockets/links counts, ilvl);
  - bool: `std::function<bool(const Item&)>` predicate (the altart icon
    list moves into `filtermatchers.cpp`; `PricedFilter`'s predicate
    captures `const BuyoutManager&`);
  - colors: sockets-vs-links mode;
  - mods: no payload.
  Also here: `FilterCatalog` (immutable ordered spec list, D2) and
  `BuildFilterCatalog(const BuyoutManager&)` returning the canonical list
  in today's `InitializeSearchForm` order.
- `filtermatchers.h/.cpp` — per-family free functions
  `bool matches(const Item&, const State&, const Payload&)`, ported
  line-for-line from today's `Matches()` bodies (including the socket-color
  `Check()` wildcard logic and `ModsFilter`'s skip-empty-name rule), plus a
  `MatchesFilter(item, spec, state)` dispatch that `std::get_if`s the
  alternative the spec's family expects and asserts on mismatch (D1).

UI side:

- `src/ui/searchform.{h,cpp}` — `SearchForm` holds `const FilterCatalog&`,
  owns the per-filter widget adapters (index-aligned with the catalog,
  length-asserted), builds the group layouts, and owns the combo models it
  materializes from the choices providers. One adapter class per family;
  each owns its widgets, connects the exact signals used today
  (`textEdited` / `clicked` / `currentIndexChanged`) to the spec's refresh
  mode via `FilterCallbacks`, and implements
  `loadFrom(const FilterState&)` / `saveTo(FilterState&)`. These absorb the
  `Initialize()` bodies and the `FromForm`/`ToForm`/`ResetForm` triple.
- `src/ui/modsfilterform.{h,cpp}` — the mods adapter: row widgets
  (combo + completer + `TokenAndFilterProxy` + per-row debounce, moved here
  from `modsfilter.{h,cpp}` and `ui/searchcombobox.h`), rebuilt from
  `ModsState` on `loadFrom`, mutating the state (via `saveTo`) on row
  add/edit/delete.

`Search` after the phase:

- Constructor takes `const FilterCatalog&` (retained for the search's
  lifetime, D2); `m_filter_states` is a `std::vector<FilterState>` built by
  calling `MakeDefaultState` per spec, index-aligned with the catalog.
  Construction asserts both the length and per-slot family agreement
  (document the alignment invariant where both are declared).
- `FilterItems` builds its active list from `state.isActive()` (visit) and
  matches via `MatchesFilter(item, spec, state)`.
- `FromForm`/`ToForm`/`ResetForm`/`Activate` leave `Search`. `MainWindow`
  drives: `ModelViewRefresh` does `m_search_form->saveTo(*m_current_search)`
  then `FilterItems`; `OnTabChange` does `loadFrom`; `NewSearch` does
  `SearchForm::reset()` + `saveTo`.

## Migration order

One commit per step; build + full `ctest` green after each (plan working
rule 2). The transitional structure is **three index-aligned collections,
all derived from the catalog**, so widget construction order (and therefore
layout) is unchanged throughout:

- The **catalog** (core) always contains pure `FilterSpec`s — never a
  widget-owning object. During transition, a not-yet-migrated spec's family
  payload is a transitional "legacy" alternative (a tag only), deleted in
  step 7.
- **`SearchForm`** owns a parallel vector of `FormSlot` =
  `std::unique_ptr<Filter>` (legacy) or a new adapter. The legacy
  constructor sequence moves **verbatim** from `InitializeSearchForm` into
  `searchform.cpp`, index-aligned with the catalog and length-asserted:
  a slot whose spec has a real payload gets an adapter; a legacy-payload
  slot invokes the corresponding legacy constructor. The 38 constructors
  exist in exactly one place from step 1 onward, shrinking per family
  commit.
- **`Search`** owns a parallel per-slot choice of `FilterData` (legacy) or
  `FilterState` (migrated); `FilterItems` consults whichever the slot has.

All transitional scaffolding is deleted in the final step.

0. **Safety net (tests only).** Pin the F33 wrong behavior (name filter as
   exemplar: background search with saved name query is skipped after
   `OnItemsRefreshed`-style refilter when the current search's box is
   empty). Pin the F35 wrong behavior (colors survive `ToForm` to a search
   without colors). Pin the F36 behaviors (added blank row lost on tab
   round-trip; rebuilt row's combo text vs saved data; container-visibility
   history). Add tab-switch round-trip cases to `tst_search` (fill filters
   on A, switch to B, back to A; assert state restores). Fill the matching
   gaps: tab filter; category **normal matching** and the `<any>` sentinel
   mapping as separate cases; **every distinct boolean predicate** — altart
   (a few representative icon needles, not the whole list), priced (with
   its `BuyoutManager` interplay), unidentified, influenced, crafted,
   enchanted, fractured, split, synthesized, mutated — asserting both the
   checked behavior and the unchecked-returns-true convention;
   `DefaultPropertyFilter`; `RequiredStatFilter`; `ModsFilter::Matches`
   (empty-name rows skipped; min/max bounds; missing mod fails); and the
   colors garbage-text parse rule (non-empty non-numeric text is *active*
   and parses to 0 via `toInt()`, mirroring min-max's `toDouble`).
1. **Skeleton.** Add `src/filters/` types (states + `MakeDefaultState`,
   spec/catalog with all payloads still the transitional legacy tag,
   matcher stubs) as the `acquisition_filters` target **with the D5
   no-Widgets link gate active from day one**, and `SearchForm` holding the
   catalog-aligned slot vector — every slot still a legacy `Filter`,
   constructed by the sequence moved verbatim out of
   `InitializeSearchForm`. `MainWindow::InitializeSearchForm` reduces to
   building the catalog + `SearchForm`; `tst_search`'s duplicated harness
   dies here — it constructs a `SearchForm` (or calls the same UI-side
   factory) instead of hand-building 38 filters. Pure mechanical move; zero
   behavior change.
2. **Boolean family** (11 instances: altart, priced, unidentified,
   influenced, crafted, enchanted, corrupted, fractured, split,
   synthesized, mutated). `BoolState` + predicate payloads + checkbox
   adapter (`clicked` → immediate). First real state migration; F33 is
   fixed for booleans from here on.
3. **Min-max family** (crit/DPS/pDPS/eDPS/cDPS/APS, armour/evasion/
   shield/block, sockets/links counts, R. level/str/dex/int, quality/
   level/map-tier/ilvl). Accessor payloads replace the `GetValue`/
   `IsValuePresent` virtuals. Preserve parse semantics exactly:
   filled ⇔ text non-empty; value = `QString::toDouble()` (0.0 on
   unparseable text).
4. **Text + combo** (tab, name, category, rarity). Sentinel handling moves
   into adapter (`saveTo` maps `<any>`/empty ↔ `""`) and stays out of the
   matcher, matching today's split. `SearchForm` materializes the combo
   models from the choices providers here. Reproduce `ToForm`'s
   `findText` + `setCurrentIndex(std::max(0, index))` fallback-to-`<any>`.
   The F33 pin from step 0 flips to asserting correct behavior here.
5. **Socket/link colors.** Port `Check()` wildcard logic verbatim
   (characterization-tested since Phase 0 for exactly this reason).
   Adapter keeps the **immediate** path. `loadFrom` writes all three boxes
   unconditionally — this is the F35 fix; flip the step-0 pin and
   release-note the behavior change.
6. **Mods** (D3, full shared shape; budget = steps 2–5 combined).
   `ModsState` + matcher to core (empty-name rows must round-trip through
   state, not be dropped); row widgets to `src/ui/modsfilterform.{h,cpp}`;
   `TokenAndFilterProxy` moves out of `ui/searchcombobox.h` to the mods
   form; `ModsFilterSignalHandler` indirection dissolves into the adapter.
   The natural adapter shape fixes F36 deliberately: row add/edit/delete
   mutate `ModsState` (so a new blank row survives tab switches),
   `loadFrom` sets each row's combo text from `ModRow::mod`, and container
   visibility derives from the row count. Flip the step-0 F36 pins and
   release-note the changes. Delete `src/modsfilter.{h,cpp}`. Fallback if
   row management resists: hybrid (matching+state in core, self-contained
   row widget in ui), recorded as a finding — the F36 fixes still apply in
   the hybrid.
7. **Teardown.** Delete `Filter`, `FilterData`, the transitional legacy
   payload alternative, and the slot machinery; `filters.{h,cpp}` disappear
   (contents live on in `src/filters/` and `src/ui/searchform.*`). `Search`
   loses the form triple; `tst_filters` becomes `QTEST_GUILESS_MAIN`;
   `tst_searchform` holds the adapter round-trips. Verify the D5 gate is
   airtight (no widget module in `acquisition_filters`'s link closure) and
   grep-check for `ui/` includes under `src/filters/`.

## Hazards

- **Signal mapping must be reproduced signal-for-signal**, not
  re-derived from intent:

  | Widget | Signal | Path | Fires programmatically? |
  |---|---|---|---|
  | tab/name/min/max text boxes | `textEdited` | debounced | no |
  | color R/G/B text boxes | `textEdited` | **immediate** | no |
  | checkboxes | `clicked` | immediate | no |
  | category/rarity combos | `currentIndexChanged` | debounced | **yes** |
  | mods rows / add / delete | via signal handler | debounced | partly |

  The combo row means `loadFrom` during a tab switch can schedule a
  debounced refresh — existing behavior, keep it. Conversely `setText`/
  `setChecked` in `loadFrom` must not notify (they don't, with the signals
  above) — switching to `textChanged`/`toggled`/`activated` anywhere would
  change semantics.
- **F33/F35/F36 fixes are deliberate behavior changes** riding on a
  behavior-preserving phase. Keep them visible: pinned in step 0, flipped
  in steps 4–6, called out in the release notes.
- **Parse rules are semantics.** Min-max: filled ⇔ text non-empty, value =
  `toDouble()` (0.0 on garbage). Colors: filled ⇔ text non-empty, value =
  `toInt()` (0 on garbage) — and garbage therefore makes the filter
  *active*. Adapters must encode exactly this into the `std::optional`s.
- `CategorySearchFilter::k_Default` / `RaritySearchFilter::k_Default`
  (`"<any>"`) are matching semantics, not just UI: empty state string means
  "inactive". Category queries are lowercased in `saveTo` today
  (`FromForm`); rarity is not. Preserve both.
- **Catalog lifetime.** `SearchForm` and every `Search` hold
  `const FilterCatalog&`; MainWindow must construct the catalog before, and
  destroy it after, every search (it already deletes searches explicitly).
  The priced predicate captures `const BuyoutManager&` — `Application`'s
  ownership of `BuyoutManager` guarantees it outlives the catalog; say so
  at the catalog's declaration.
- **Catalog↔states↔slots alignment** is a hard invariant; a mismatch
  silently matches items against the wrong state. Guard it three ways:
  states are only created by `MakeDefaultState` per spec; `Search`
  construction and `SearchForm` construction assert lengths and per-slot
  family agreement; `MatchesFilter` uses `std::get_if` + assert rather than
  throwing `std::bad_variant_access`.
- `ModsFilter` semantics to **preserve**: activity is `!rows.empty()` (a
  single empty row makes it "active" yet match-everything); `Matches` skips
  empty-name rows; empty rows round-trip through state. `ModsFilter`
  behaviors that **change deliberately** (F36): new rows persist without
  another form event, rebuilt rows display their saved mod, visibility
  follows row count.
- `NewSearch` relies on reset-then-save producing a default state for the
  new search while other searches keep theirs — the step-0 tab-switch tests
  cover this.
- `RefreshReason::TabChanged` early-out in `FilterItems` must keep working
  when `Activate` dissolves into MainWindow calls.

## Acceptance criteria

- **Build-enforced boundary (D5):** `acquisition_filters` compiles and
  links with no `Qt6::Widgets` dependency; the app and tests consume it.
  Grep gate (for what the linker can't see): no `src/ui/` includes under
  `src/filters/`. `src/modsfilter.{h,cpp}` and `src/filters.{h,cpp}` are
  gone; `TokenAndFilterProxy` is no longer reachable from core.
- `tests/tst_filters.cpp` runs `QTEST_GUILESS_MAIN`; all ported
  characterization assertions unmodified except the documented
  F33/F35/F36 flips. `tst_searchform` covers adapter round-trips, the
  tab-switch save/restore cycle, and immediate-vs-debounced wiring.
- F33: background searches filter with their own activity (step-0 pin,
  flipped). F35: colors clear on tab switch (likewise). F36: added rows
  survive tab switches, rebuilt rows display their saved mod, container
  visibility follows row count (likewise).
- Manual smoke per plan checklist, plus: every filter group produces
  identical result counts on a reference stash before/after; debounced vs
  immediate refresh behavior unchanged; filter state restores across
  search-tab switches; mods completer + row add/edit/delete behave as
  before (modulo the documented F36 fixes).

## Outcome (July 2026)

Implemented and reviewed. Corrections to the spec above, recorded because
they matter to anyone reading it as a description of the code:

- **D5 was weaker than this doc claimed; it is now enforced explicitly.**
  "Any transitive widget header fails the build" is not what
  `target_link_libraries` on a STATIC library buys: an archive has no link
  step, so it can never reject a widget symbol, and whether a widget header
  even *compiles* under `acquisition_filters` depends on the Qt layout (it
  does not with macOS frameworks; a normal Unix install exposes it through
  the umbrella include dir). The boundary is now a CTest
  (`cmake/filters_boundary_check.cmake`, test `filters_boundary`) that bans
  `ui/` and widget includes under `src/filters/` and asserts the archive
  asks the linker for no widget symbols.
- **The catalog has 38 filters, not 37** (corrected above). The old
  `InitializeSearchForm` list had the same 38.
- **Two mods behavior changes beyond F36's three.** Deleting a row now
  compacts the remaining rows (the old grid left the hole), and a mod name
  typed but not chosen from the list now persists and filters (the old form
  only stored the mod on `currentIndexChanged`, so free text was dropped).
  Both follow from the adapter shape, both are tested, and both are
  release-note items.
- **A fourth deliberate behavior change: pending edits follow their
  search.** The mods form writes through to the bound search immediately but
  only arms the one global debounced refresh, so a fast tab switch left the
  edited search showing a criterion its buckets did not reflect.
  `MainWindow` now flushes a pending debounced change onto the outgoing
  search before rebinding the form — an edit applies to the search it was
  made in rather than being discarded — and `FilterItems`'s `TabChanged`
  short-circuit consults a dirty flag instead of assuming a tab change means
  nothing changed.
- **`RefreshMode` drives the wiring.** As first written, adapters hardcoded
  their signal path and merely asserted the spec agreed — an assert that
  compiles out in release. Adapters now resolve the callback from
  `spec.refreshMode` (`FilterCallbacks::forMode`), so the catalog is the
  single source of truth for the debounce mapping.
- **The `<any>` sentinel belongs to the filter, not to the item data.**
  `InitItemClasses` no longer appends it to the category list (which had
  made `itemcategories` depend on `filterspec`); the category choices
  provider prepends it instead.
- **F38 surfaced**: the "Influenced" filter also matches fractured and
  synthesised items, because `Item::hasInfluence()` reads an influence list
  carrying those markers. Pre-existing; recorded, pinned by test, not fixed.
