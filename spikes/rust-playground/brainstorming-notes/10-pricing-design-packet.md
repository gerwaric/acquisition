# Pricing design packet — the second plan-bearing consumer

**Written 2026-09-03**, before any pricing code. This is the document
the owner rules on, in the shape of 06: candidate decision lines in
registry form, one chosen slice, a parking lot with triggers. Accepted
lines are harvested verbatim into a new `decisions/pricing.md` (one
index row in `CONTEXT.md`); the two lines addressed to
`decisions/plans.md` go there. Then this file is history, never a
second authority.

Evidence read, in order: 00 (goal function, the three-sources rule), 07
("Pricing, seen through the tower", the ten patterns), 08 (the nine
boundaries), 06 (the packet shape); `decisions/plans.md`,
`decisions/store.md`; on master, `docs/user/pricing.md`,
`docs/user/forum-shop.md`, `docs/design/legacy-buyout-import.md`,
`src/buyout.h`, `src/buyoutmanager.cpp`, `src/shop.cpp`,
`src/currency.cpp`, and the 0.18 `item_buyouts` / `location_buyouts`
tables (`src/datastore/buyoutrepo.cpp`); here, `annotations.rs` and the
planner's versioned strict-parse path. The C++ app is evidence of
rules, never of shape.

Tentative ids `C64`–`C77` are assigned in reading order; at harvest the
accepted lines are renumbered consecutively from the registry's next
id, and the ids in this file are never cited from code or tests.

## Frame

Pricing edits irreplaceable intent, offline. No daemon, no live run, no
outward traffic. Its validating consumer is real use, twice: the owner's
C++ buyouts flowing through the import, and a rendered shop page read
against the forum. Publishing is out of scope.

Two facts the evidence corrected on the way in:

- **The 0.18 userstore keys character locations on the character id**
  (`ItemLocation` for a character sets `m_unique_id = character.id`),
  not the name: 07's "resolve C++ character locations, which are names"
  was the pre-0.16 legacy shape. The import matches on ids throughout.
- **A substash item's C++ location id is its parent's** (`id()` stays
  the display tab; `fetch_id()` is the substash). Imported location rows
  therefore land on parent tabs, which is what the inheritance rule
  below already means.

## Pricing through the four layers (the design in one table)

| Layer | Pricing content | Mutation path |
| --- | --- | --- |
| Facts | tab names, item notes (`~b/o 5 chaos`), tab index, item x/y, membership, freshness | the daemon's `record` only |
| Intent | a `buyout` row on an item, tab or character: type, value, currency, author, revision | `PricePlan` apply, through the annotation API only |
| Derivations | the game-set price parsed from a fact; inheritance; the effective price with its explanation; coverage and freshness gaps; the shop page and its hash | recomputed, never stored |
| Effects | none in this slice; a forum post is the parked one | — |

The C++ `item_buyouts` row fuses the three top layers (`source` = fact
vs intent, `inherited` = a derivation stored as data, `value` =
intent). The locks exist because the layers were fused. Separated, the
locks dissolve into reports.

## 1. Candidate decision lines

Each is one bullet in registry form, under the 800-byte limit, with
its destination file. *Recommend* is the agent's; **Ruling** is the
owner's slot.

### (a) Already built; wants a name

The intent file's compare-and-swap, tombstoned delete, orphan
retention, uuid binding and store-managed export are C35 and need no
new line. Pricing exercises them for the first time at scale; the
targeted review in the slice's first step is the check.

### (b) Landings on open topics

- **C64 — Pricing is intent edited offline: a buyout is an annotation, only explicit assertions are stored, and everything else is derived.** A `buyout` row sits on an item, a tab, or a character; *inherit* is the absence of a row, never a value; a game-set price is a derivation from a fact (tab name, item note), never a row; the effective price is one function over facts and intent, recomputed on read. No pricing operation contacts the daemon, quotes, or creates a job. *Why:* C++ materialized inherited prices onto every item and then needed locks so a refresh could not fight an edit; a derivation cannot be clobbered (C34). *Details:* `acquisition-plan` doc, C64. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C65 — Every intent write names its author.** An annotation row carries `author` (the adapter that wrote it: `acq`, `acq-mcp`, `import`, later `acq-gui`) and an optional `source` (for an import, the source file's path and sha256; for a reviewed plan, the plan's hash), stored on the row (annotations schema v3), returned on every read, required by the write API. *Why:* pattern 4 — the table has scope, key, kind, value, revision and timestamps but no author; who set a price and through what cannot be reconstructed later, and C14 makes agents writers of intent. Cheap today, unrecoverable after the first import (P3). *Details:* `annotations.rs` doc, C65. — `decisions/pricing.md`
  *Recommend:* accept, before the first price is written. **Ruling:** ___

- **C66 — Intent values are typed at the write API: a kind declares its schema version and a strict parser, a value that does not parse under its stamp never lands, and what lands re-serializes to exactly what was read.** The generic — version gate, unknown fields refused at every depth, exact round-trip, then compare-and-swap — is factored out of the sync policy's parser into the store crate over a per-kind trait; each kind's shape stays its owner's; a stored value is upgraded on read, never rewritten in place. *Why:* a value shape that changes after data exists is a migration of the irreplaceable state (pattern 4), so strict-from-day-one must be structural, not each frontend's discipline. *Details:* `annotations.rs` doc, C66. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C67 — The `buyout` value, v1:** `type` is one of `buyout` (`~b/o`), `fixed` (`~price`), `no_price`, `ignore`; `buyout` and `fixed` carry `value` (a positive decimal written as a string, no float ever) and `currency` (a reference tag, C68); the other two carry neither. `current_offer` (`~c/o`) is not a value — C++ already logs it obsolete — and is refused at write and reported at import. Rows are keyed on the GGG id alone (item id; tab id, substash `parent/id`; character id, C55), never a coordinate: intent follows the thing through a league merge. *Why:* the forum tag must be what the human typed, and a remove-only tab keeps its id when a league ends (the import is the first test of that). — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C69 — The effective price is the finest explicit statement, and every result says why.** An item's own statement beats its location's; a location's covers every item in it without one (C70). At equal grain a game-set price (a fact, already public on the trade site) is the effective one and a manual row beside it is *shadowed*: kept, never rewritten, reported with the remedy (edit the note or tab name in game, or drop the row). Every effective price carries its cause — the note, the row, the tab name, the location it inherits from, or unpriced — and its shadowed rows. *Why:* the C++ lock refused the edit; a report lets the user state intent and be told exactly what contradicts it (patterns 1, 2, 10). *Details:* `acquisition-plan` doc, C69. — `decisions/pricing.md`
  *Recommend:* accept. 08's alternative (refuse the write against a game-priced target as a plan non-action) is recorded below under rejections; it covers only one ordering, since a note added in game after the row exists shadows it anyway. **Ruling:** ___

- **C70 — A priced tab covers that tab and its children, the way a policy id does (C37);** a substash row overrides its parent's for that substash; an item row overrides both. *Why:* the C++ store already lands substash items' location prices on the parent, and the house rule for tab-scoped intent should not have two shapes. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C71 — A `PricePlan` is a local authorization envelope, applied atomically to the intent file.** It cites provider, account uuid, operation and schema stamp, its fact basis, a *precondition set* — one `(scope, key, kind, revision-or-absent)` per row it touches — the explicit mutation set (a full new value or a delete each), and its non-actions with reasons. Apply checks every precondition under one transaction and lands all mutations or none; one moved revision refuses the whole plan. An immediate edit is plan-then-apply with the plan still in the JSON contract. *Why:* reviewing a document before applying it means the document is what lands (C38); the refresh gate (C44) is this set with one element (pattern 9). *Details:* `acquisition-plan` doc, C71. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C72 — Pricing never edits the sync policy, and a price never locks a tab into refresh.** The relationship between the two kinds of intent is reported, not enforced: the consumer that needs freshness — `shop render` first — names priced locations outside the policy's coverage and priced facts older than its stated window, each with the remedy (the policy edit, or the `RefreshPlan` it would take, C41). *Why:* C++'s "priced tabs are always refreshed" is one kind of intent silently rewriting another; a report keeps the policy what its author wrote (pattern 10, 08 boundary 7). — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C73 — The legacy import is a `PricePlan` generator over the 0.18 userstore, and every source row is accounted for.** It reads `item_buyouts` and `location_buyouts`; `source = manual` rows become mutations; `inherited = 1` (a derivation), `source = game` (a fact we re-derive), `source = auto`, and `~c/o` rows are non-actions with those reasons; a target the facts do not hold is `not_in_facts` with "refresh first" as the remedy; an existing row is never overwritten (`unchanged` when equal, `existing_differs` otherwise). A second run is all `unchanged`. The plan carries the source file's path and sha256 into C65's `source`. *Why:* the wizard dissolves into the ordinary plan/apply path (parking lot 2026-08-31); a refusal keeps what it refused (pattern 2). — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

### (c) New scope

- **C68 — Reference data is facts with build provenance: shipped inside the binary, versioned by the build, read-only, never in a store file.** The currency list is the first table: stable tag (`chaos`, `divine`, …), display name, the aliases the game and forum accept, realm applicability. Intent cites reference data by stable tag; a reader that meets a tag its build lacks reports it and never guesses. The mod catalog and item categories ship the same way when search needs them. *Why:* not an account fact, not intent, not a derivation — the tower had not placed it (pattern 8), and the currency list is where pricing meets it first. *Details:* the table lives beside its first consumer until a second one appears. — `decisions/pricing.md`
  *Recommend:* accept now, at the first meeting. **Ruling:** ___

- **C74 — `shop render` is in scope as a derivation, and publishing is not.** The page is a pure function of facts, intent and a template: items grouped by effective price under the forum's spoiler and link codes (tab index and x/y, or the character's inventory and name), split at the post limit, substituted for `[items]`, hashed; stamped with `as_of` and the gaps of C72. It sends nothing; the template is a render-time input, not stored; the owner pastes the page by hand, as the C++ guide documents. *Why:* every human surface is a derivation over a machine surface (pattern 3), and this one gives pricing a real-use reading before any outward traffic exists; the post stays parked behind its own boundary session. *Details:* `acquisition-plan` doc, C74. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C75 — Pricing lives in `acquisition-plan` as a module, and the envelope is factored after `PricePlan` exists, never before.** The crate's charter is intent, plans and apply; a crate split waits for a dependency-graph property it would buy (none is in view: the daemon links neither). What the two envelopes literally duplicate — stamp, strict round-trip parse, provider and uuid, basis, the precondition set — is factored as shared functions; a universal grammar or enum is not built. *Why:* generalize after two consumers reveal the shared property (P3); pattern 9 predicts a family sharing a discipline, and this slice is where the prediction is tested. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

### Lines for `decisions/plans.md` (the two open topics)

- **C76 — One door: an explicit selection compiles to a plan through the same envelope and apply.** `--tabs a,b`, `--all`, `--characters` name a selection the planner compiles like a policy — every listing and fetch explicit, freshness ignored (a selection means *now*), the precondition set empty because no stored intent was derived from — quoted and applied through the `apply` parent (C43). The ad-hoc `refresh` job kind retires from the CLI and MCP; the daemon's kind goes when nothing submits it (`tools/persist-check.sh` moves to a selection plan). *Why:* two doors to one task; four live runs and nobody reached for the ad-hoc door; the empty set is the second data point for pattern 9's precondition set (policy: one; price: many; selection: none). — `decisions/plans.md`
  *Recommend:* accept; build as its own small step after the pricing slice, not inside it. **Ruling:** ___

- **C77 — The freshness window stands as the handle for "keep these fresh"; a refetch cycle bought by facts aging past the window during a long cycle is correct, not a fault.** A plan is compiled against ages at compile time and names the facts it keeps as fresh that will pass the window before the cycle's estimate ends (`aging`, the driver's warning promoted into the plan) — it does not pre-fetch them. *Why:* the 2026-09-02 rerun's 6-request cycle 2 was the window doing its job; no owner friction was recorded across four runs, and a cycle-based handle ("fresh as of every run") is the C++ model this policy replaced (C36). — `decisions/plans.md`
  *Recommend:* accept, or park it with the trigger "owner friction with a refetch cycle". **Ruling:** ___

### (d) Recorded rejections (so nothing is re-argued or adopted by not noticing)

- **The game-price lock** (C++: a game-set target refuses edits). Rejected in favour of C69's shadowed report. 08's middle ground — the plan lists the write as a non-action `game_priced` — is recorded as the fallback if the owner prefers refusal to shadowing; it protects one ordering only.
- **"Priced tabs are always refreshed"** (C++ `SetRefreshLocked`). Rejected by C72: intent never rewrites intent; C34's authority rule.
- **Materialized inheritance** (C++ `PropagateTabBuyouts`, the `inherited` bit). Rejected by C64: a derivation stored as data.
- **`~c/o` / current offer** as a value. Retired by C67; C++ logs it obsolete already.
- **A fused "render and post"**. Rejected by C74 and the 00 gravity warning: the render is a derivation, the post an outward effect outside the choke-point invariant.
- **Prices as floats.** Rejected by C67: the forum tag is what the human typed.
- **A pricing crate now.** Rejected by C75 until a dependency-graph property asks for it.

## 2. The slice: pricing, offline, validated by real use

Each step gate-green (`AGENTS.md`, run bare); observable behavior
unchanged until step 4. Findings from reviews go to a
`PRICING-SLICE.md` closed record in `REFRESH-SLICE.md`'s shape as they
happen, never to `CONTEXT.md`.

1. **Harvest.** Accepted lines into `decisions/pricing.md` and
   `decisions/plans.md`, the index row in `CONTEXT.md`, the parking-lot
   entries below replacing the 2026-08-31 pricing entries. Nothing
   narrative.
2. **Clause audit and the targeted review — before the file carries a
   price.** (a) `decisions/plans.md` and C35: every clause of C35,
   C36–C44 either names a test (`c<n>_…`, the session-close convention)
   or a doc comment cites it; `tools/docs-check.sh` reports C36, C39,
   C40, C41, C42, C52 uncited today. (b) A review of `annotations.rs` as
   the one durability boundary about to become load-bearing — not a
   general code review: the CAS under a batch, tombstone semantics
   under a batch, the v2 → v3 migration path, WAL and busy-timeout under
   two writers, export under load, `list` at thousands of rows. Findings
   table opened in `PRICING-SLICE.md`.
3. **Annotations v3 + typed intent** (C65, C66, C71's store half).
   `author`/`source` columns; `put_intent`/`get_intent` over the
   per-kind trait; `apply_batch(preconditions, mutations, author)` in one
   transaction; scope `character`. `SyncPolicy` moves onto the typed
   path with no behavior change — the factoring rule's first payment.
   Pinned by id.
4. **Reference data and the buyout value** (C67, C68). The currency
   table with build provenance; the `Buyout` v1 strict type; the
   game-price parser over tab names and item notes (the C++ regex and
   alias table are the rule evidence; the parser is a derivation over
   facts).
5. **The effective price** (C69, C70). One function over the store's
   items, tabs, characters and the buyout rows, with explanations;
   `acq price show <target>` / `acq price list [--league] [--realm]`
   under C53's three levels. Handles pinned: one location price, one
   inherited item, one item override, a substash under a priced parent,
   a game-shadowed row, an orphaned item's row, an unknown currency tag.
6. **`PricePlan` and apply** (C71, C72). `acq price set|clear <target>
   …` immediate and `--plan`; `acq price apply=FILE`; the
   stale-revision conflict, the all-or-nothing batch, the no-op plan
   that says so. Whether the envelope pieces duplicate `RefreshPlan`'s
   is observed here and factored in step 9, not guessed now.
7. **The import** (C73) from the owner's 0.18 userstore file, then
   **validation reading 1**: the owner runs it on the real store, reads
   the plan, applies it; every source row's outcome is in the plan; a
   second run is all `unchanged`. Evidence the run collects: whether
   remove-only tab ids matched (C67's league-merge claim), how many rows
   were `not_in_facts` against a fresh refresh, and the owner's words,
   verbatim, from the conversation (pattern 6).
8. **`shop render`** (C74, C72's report), then **validation reading
   2**: the owner reads the rendered page against the forum for their
   own shop and pastes it by hand. Evidence: link codes for substash
   items (C++ skips a tab it cannot index), page splitting, whether the
   coverage and staleness lines change what the owner does next.
9. **MCP** (`prices`, `price_plan`, `price_apply`, `shop_render`) as
   thin adapters over the same functions — the "no parallel semantics"
   check, as in tracer step 8. Then the factoring of C75: what the two
   envelopes literally share moves into shared functions.
10. **Close.** `PRICING-SLICE.md` (step ledger, findings, what the
    readings taught, observations still open); the pattern-9 verdict
    (family with a shared discipline, or one grammar) recorded as a
    ruling or a parking-lot trigger; the event-log question answered.

**Done criterion:** the owner's manual buyouts are in the intent file
through the import with every source row accounted for and a second run
inert; a page rendered from them reads correct against the forum in the
owner's words; the seven handles and the atomic batch are pinned by id;
and the question "what was repriced since T" is either answerable from
rows (`updated_at`, `author`, tombstones — which rows, not from what) to
the owner's satisfaction, or the annotation event log's trigger has
fired with the evidence.

**Anti-scope, with triggers** — in the parking lot below. Nothing in
this slice touches the daemon, the limiter, the network layer, or a
GUI.

**The method test this slice runs:** whether Plan is a family sharing an
envelope discipline (pattern 9), tested by building the second document
without looking at the first's grammar until step 9; whether the intent
layer holds under thousands of rows and three authors; whether
provenance and strict values were cheap now, as claimed.

## 3. Parking lot (landings named; deferral never re-argued)

Replaces the pricing, legacy-import and shop entries of the 2026-08-31
parking lot once the slice is harvested; the rest of that lot stands.

- **Shop / forum publishing** (POESESSID, thread numbers, auto-post
  after a clean refresh) → outward credentialed traffic outside the API
  choke-point invariant; its own boundary session before any code.
  Trigger: the render validated and the owner wanting the post
  automated.
- **User-scoped intent** (shop template, currency ratios, saved
  searches) → `user.db` + scope taxonomy. Trigger: the first user-scoped
  kind actually written; v1 takes the template from a file.
- **Annotation event log** → trigger: the done criterion's "repriced
  since" question fails on rows alone, or a conflict needs history.
  Provenance and `updated_at` are on rows from step 3 so the log is an
  addition, not a migration.
- **Batch pricing by query** (price everything a search selects) →
  lands on search semantics. Trigger: a real batch workflow the seven
  handles cannot express (08 boundary 8).
- **`current_offer` as a value** → trigger: a real use; C++ calls it
  obsolete.
- **Public-tab awareness** (a game price counts only when the tab is
  public) → trigger: a false `shadowed` report on a private tab at
  validation reading 2.
- **PoE2 currencies and per-realm tags** → trigger: a PoE2 stash
  endpoint or a poe2 price the owner wants to set; the table carries
  realm applicability from day one so this is rows, not shape.
- **Coverage advice in `refresh --plan`** (the planner naming priced
  locations outside coverage) → trigger: the render's report proving
  the wrong place for it.
- **Currency totals and history** → a derivation over facts, cheap;
  after pricing stands.
- **Price history / "last price was"** → the event log's trigger.
- **The explicit-selection door** (C76) → built as its own step after
  this slice; not pricing's scope.
- **Name→id resolution for targets** (`price set tab:"Maps"`) →
  trigger: authoring friction at the CLI, the same rule as C63.

## 4. Questions for the owner

1. C69's shadowing versus 08's refusal: accept the report, or the
   plan-time non-action?
2. C74: is the render in scope, given it sends nothing?
3. C77: rule the window, or park it with the trigger?
4. The order inside the slice: the import (reading 1) before the render
   (reading 2), so the page is rendered from real prices.
5. `current_offer` retired (C67), or carried as a value for the import's
   sake?
