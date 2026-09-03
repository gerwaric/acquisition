# Pricing design packet — the second plan-bearing consumer

**Written 2026-09-03.** This is the document the owner rules on, in
the shape of 06: candidate decision lines in registry form, one chosen
slice, a parking lot with triggers. Its history is `git log` on this
file and the external review beside it (`11-pricing-design-review.md`);
nothing here restates either. Accepted lines are harvested
verbatim into a new `decisions/pricing.md` (one index row in
`CONTEXT.md`); the two lines addressed to `decisions/plans.md` go
there. Then this file is history, never a second authority.

Evidence read, in order: 00 (goal function, the three-sources rule), 07
("Pricing, seen through the tower", the ten patterns), 08 (the nine
boundaries), 06 (the packet shape); `decisions/plans.md`,
`decisions/store.md`; on master, `docs/user/pricing.md`,
`docs/user/forum-shop.md`, `docs/design/legacy-buyout-import.md`,
`src/buyout.h`, `src/buyoutmanager.cpp`, `src/shop.cpp`,
`src/currency.cpp`, the 0.18 `item_buyouts` / `location_buyouts`
tables (`src/datastore/buyoutrepo.cpp`) and `userstore.cpp` (WAL, the
username-named file); here, `annotations.rs`, `schema.sql` (characters
keyed by id, tabs by realm, league and id) and the planner's versioned
strict-parse path. The C++ app is evidence of rules, never of shape.

Tentative ids `C64`–`C78` are assigned in reading order; at harvest the
accepted lines are renumbered consecutively from the registry's next
id, and the ids in this file are never cited from code or tests.

## Frame

Pricing edits irreplaceable intent, offline. No daemon, no live run, no
outward traffic. Its validating consumer is real use, twice: the owner's
C++ buyouts flowing through the import, and a rendered shop page read
against the forum. Publishing is out of scope.

Two facts about the import's source that shape the lines below:

- **The 0.18 userstore keys character locations on the character id**
  (`ItemLocation` for a character sets `m_unique_id = character.id`),
  not the name. The import matches on ids throughout.
- **A substash item's C++ location id is its parent's** (`id()` stays
  the display tab; `fetch_id()` is the substash). Imported location rows
  land on parent tabs, which is what the inheritance rule (C70) means.

## Facts this slice will discover

Nobody knows what the trade site shows when an item carries an in-game
price note and is also posted in a forum shop at another price; the
price-note grammar and the currency vocabulary have moved since the
C++ tables were written; PoE2 may differ again. The design keeps every
one of those answers *outside* intent — in facts stored verbatim
(re-parseable without a refetch), in reference data versioned by the
build, and in derivations that re-derive whole when a rule changes —
so a discovery is a code change and a claim, never a migration of the
irreplaceable state. Three things make the discovery deliberate:

- **Trade and forum facts get numbered claims**, authored master-side
  beside the network ground truth (proposed: `docs/design/trade-ground-truth.md`,
  claims `T<n>`), cherry-picked here and cited by number — the same
  rule as for facts about the API.
- **The forum reading is an experiment**, not a proofread (step 8): a
  matrix the owner runs by hand with junk items, observed on the trade
  site after an indexing wait, each cell becoming a claim.
- **The currency table is authored from the trade site's published
  static data**, both realms, by a tool run outside the daemon at
  build time, with the fetch date and digest as its provenance (C68);
  rerunning the tool is how the table adapts. The owner's own tab
  names and item notes are the free corpus of current formats, read at
  the census (step 2c) and watched by a tripwire after it (C69).

## From the driver's seat

The design read as one system, by the agent that will drive it. The
lines below are shaped by this reading.

**The loop is the system.** Whatever layer a driver touches, the work
has one shape: *observe* with a basis → *state* the desired end state →
*compile* it against a named snapshot into an explicit plan that carries
its preconditions, its cost, its non-actions and its warnings → *review*
→ *apply* exactly → *observe the delta*. Refresh runs this loop with the
daemon as the apply target and wire sends as the cost. Pricing runs it
with the intent file as the target and no wire cost at all. Publishing,
when it comes, runs it against the forum with credentialed posts — one
per page — as the cost. The three targets cannot share a vocabulary;
they share the loop and, the hypothesis goes, the envelope. That is
pattern 9 as a system property. Whether the shared thing is a compiler
and an envelope discipline rather than a grammar is what this slice
tests (C75), not what it presumes.

**What each layer owes the driver.** Facts owe a basis and an age on
every read. Intent owes a revision and a writer on every row.
Derivations owe a cause on every result and a remedy on every gap.
Effects owe a receipt. With those four present, a driver never re-reads
to learn whether *intent* moved, never acts on what it has not seen,
and can always answer "did that land?" and "undo it". Facts live in a
separate file and can drift under any plan; drift is reported, never
silently absorbed and never a refusal (C44's rule, kept).

**The driver's economy.** Store reads are free and network-free by
construction; the scarce resources in an agent's loop are context and
turns. So a read's default is a summary, detail is bounded by an
explicit filter, every vocabulary a read uses is enumerable through the
same surface, and every gap names its remedy as a runnable command or a
plan — C53's density model applied to the JSON contract. This is a
slice hypothesis validated by the agent's own mock run (step 9), not a
ruling: pin after the consumer validates.

## Pricing through the four layers, plus one input

| Layer | Pricing content | Mutation path |
| --- | --- | --- |
| Facts | tab names, item notes (`~b/o 5 chaos`), `metadata.public`, tab index, item x/y, container, socketing, membership, freshness | the daemon's `record` only |
| Intent | a `buyout` row on an item, tab, substash or character: type, amount, currency, writer, revision; the receipt ledger | `PricePlan` apply, through the annotation API only |
| Derivations | the game observation; manual inheritance; the relation between them; the listing state; eligibility with reasons; the shop page and its hash; the situation summary | recomputed, never stored |
| Effects | none on the wire in this slice; forum posts are the parked one | — |
| Reference data | the currency table, versioned by the build (C68) | a build |

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
targeted review in the slice's second step is the check. C52 (an agent
never clobbers intent it has not read) is C71's precondition set with
one element and stays where it is.

### (b) Landings on open topics

- **C64 — Pricing is intent edited offline: manual listing intent is stored as explicit assertions only; game pricing, inheritance and every listing state are derived.** A `buyout` row sits on an item, a tab, a substash or a character; *inherit* is the absence of a row, never a value; a price observed in a tab name or item note is a derivation from a fact, never a row; what an item's listing is — manual side, game side, their relation, its eligibility — is recomputed on read. No pricing operation contacts the daemon, quotes, or creates a job. *Why:* C++ materialized inherited prices onto every item and then needed locks so a refresh could not fight an edit; a derivation cannot be clobbered (C34). *Details:* `acquisition-plan` doc, C64. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C65 — Every intent write carries structured provenance: the channel it came through (`written_via`), an optional claimed `actor`, and the hash of the plan that landed it (`applied_plan`, C71).** Stored on the row (annotations v3), returned on every read, required by the write API; origin detail (an import's digest and timestamps) lives on the receipt (C78), never per row. Rows written before v3 migrate as `unknown_legacy` — a migration never manufactures a writer. *Why:* pattern 4 — who set a price and through what cannot be reconstructed later, and C14 makes agents writers of intent. Cheap today, unrecoverable after the first import (P3). *Details:* `annotations.rs` doc, C65. — `decisions/pricing.md`
  *Recommend:* accept, before the first price is written. **Ruling:** ___

- **C66 — Intent values are typed at the write API: a kind declares its schema version and a strict parser, a value that does not parse under its stamp never lands, and a current-schema value re-serializes to exactly what was read.** The generic — version gate, unknown fields refused at every depth, exact round-trip, then compare-and-swap — is factored out of the sync policy's parser into the store crate over a per-kind trait; each kind's shape stays its owner's; an older stored value upgrades in memory, its raw JSON untouched. *Why:* a value shape that changes after data exists is a migration of the irreplaceable state (pattern 4), so strict-from-day-one must be structural, not each frontend's discipline. *Details:* `annotations.rs` doc, C66. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C67 — The `buyout` value, v1, and its target.** A typed `PriceTarget` — item and character by id (C55), tab by `(realm, id)`, substash by `(realm, parent, id)`, league absent so intent follows a tab through a league merge — is the public API; frontends never build raw keys. `type` is `buyout`, `fixed`, `no_price` or `ignore`; the first two carry `amount` and `currency` (a reference tag, C68). `amount` is an exact decimal in one canonical text, never a binary float; ratios and a precision bound are ruled after the census. `current_offer` is refused at write and kept at import as a non-action. *Why:* the key mirrors the store's identity (C54, C58); the forum tag is what the human typed. *Details:* `acquisition-plan` doc, C67. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* canonical text = no exponent, no leading or trailing zeros, so equality is textual; `no_price` and `ignore` carry neither amount nor currency, and their presence is a parse error.
  *Recommend:* accept; the grammar's edge is the one deliberately open clause. **Ruling:** ___

- **C69 — An item's listing is two independent resolutions and their relation, never one scalar.** The manual side resolves by specificity (C70); the game side resolves item note, then tab name, and carries whether its stash is public. The relation is manual-only, game-only, agree, conflict or none; `ignore` is a disposition on the manual side and never denies the observed game price. Every result carries both sides with causes, revisions, basis and age. What a relation *means* is each consumer's rule as a Rust derivation (C74), never a frontend's. *Why:* game observation, forum intent, disposition and renderability are four statements; C++ fused them and needed locks; "game wins" had no evidence. *Details:* `acquisition-plan` doc, C69. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* a read's output is a plan's input — the revisions and basis on the result are what the compiler turns into preconditions and provenance. The game side keeps what it cannot fully parse: a note with a known tag and an unrecognized currency is an observation ("game price, currency unknown"), never "no game price"; a tilde-prefixed note or tab name that parses as nothing is counted and inspectable in `price status` — the drift tripwire (C56) applied to price formats.
  *Recommend:* accept. The conflicting row is stored and reported as `conflict`. **Ruling:** ___

- **C70 — A priced tab covers that tab and its children, the way a policy id does (C37);** a substash row overrides its parent's for that substash; an item row overrides both; a character row covers its items. Coverage is the manual side's inheritance only — it says nothing about eligibility (C74). *Why:* the C++ store already lands substash items' location prices on the parent, and the house rule for tab-scoped intent should not have two shapes. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C71 — A `PricePlan` compiles a desired state against a named snapshot and is applied atomically to the intent file; its preconditions protect intent, and fact drift is reported, never a gate.** Desired state (handles, a document, or a foreign store, C73) compiles into the envelope, a precondition set (one revision per row touched, tombstone generations included, with the prior value), the mutations, non-actions with reasons, and mutation counts as its cost; duplicate targets are refused. Apply checks every precondition in one transaction and lands all or none; the result is every row as written plus the current basis. *Why:* what was reviewed is what lands (C38); C44 and C52 are this set with one element (pattern 9). *Details:* `acquisition-plan` doc. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* envelope = provider, uuid, operation, schema stamp, fact and reference bases; a never-existed row's revision is 0 and a deleted row's is its tombstone's, so "one moved revision refuses" is exactly true; counts = create, update, clear, unchanged, refused.
  *Recommend:* accept. **Ruling:** ___

- **C72 — Pricing never edits the sync policy, and a price never locks a tab into refresh.** The relationship between the two kinds of intent is reported, not enforced: the consumer that needs freshness — `shop render` first — names priced locations outside the policy's coverage and priced facts older than its stated window, each with the remedy (the policy edit, or the `RefreshPlan` it would take, C41). *Why:* C++'s "priced tabs are always refreshed" is one kind of intent silently rewriting another; a report keeps the policy what its author wrote (pattern 10, 08 boundary 7). — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C73 — The legacy import is a source of desired state for the one compiler (C71), read from the 0.18 userstore under one consistent SQLite snapshot, and every source row is accounted for.** Manual rows are the desired state; inherited, game, auto and `~c/o` rows are non-actions with those reasons; a target the facts lack is a non-action whose remedy is a refresh; an existing row is never overwritten; the realm is an explicit parameter stated in the plan. The receipt's origin digests the rows as read, not the file. The file is named by username, not bound by uuid: the plan states that binding and refuses a mismatch. *Why:* the wizard dissolves into the plan/apply path; a refusal keeps what it refused (pattern 2). *Details:* `acquisition-plan` doc. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* the userstore is WAL, so a hash of the `.db` file can omit uncheckpointed content; an equal existing row is `unchanged`, a differing one `existing_differs`; a second run is all `unchanged`; `REAL` amounts convert by the shortest round-trip text (what C++ itself rendered to the forum).
  *Recommend:* accept. **Ruling:** ___

### (c) New scope

- **C68 — Reference data is a fifth input, versioned by the build: authored from a named external source by a tool run outside the daemon, shipped inside the binary, read-only, never in a store file, enumerable through every surface, and cited by version wherever it is used.** The currency table is first; its version, source, fetch date and digest join the basis a plan or render stamps. Intent cites reference data by stable tag; a reader that meets a tag its build lacks reports it, never guesses; every frontend lists the table. The mod catalog ships the same way. *Why:* not an account fact, not intent, not a derivation — the tower had not placed it (pattern 8); a hand-typed table is how the C++ list went stale. *Details:* beside its consumer. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* the source is the trade site's published static currency data, per realm; a currency row = stable tag (`chaos`, `divine`, …), display name, the aliases the game and forum accept, realm applicability; the authoring tool is rerun to adapt, and its output is committed with the fetch date and digest.
  *Recommend:* accept now, at the first meeting. **Ruling:** ___

- **C74 — `shop render` is in scope as a derivation, publishing is not, and every item the page omits is counted with a named reason.** The page is a pure function of facts, intent, reference data and a template; it sends nothing; the template is a render-time input, not stored; the owner pastes the page by hand. Eligibility is derived per item and nothing is skipped silently. The C++ mechanics — spoiler grouping, link codes, the post limit, page splitting, the hash — are hypotheses until the owner's forum reading. *Why:* every human surface is a derivation over a machine surface (pattern 3); the posts stay parked behind their own boundary session. *Details:* `acquisition-plan` doc, C74. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* eligibility reasons in view — a disposition, a conflict (C69), a public game listing, socketing, an unsupported container, a missing position or tab index, removal, an unknown currency; the census (step 2c) checks which shapes the real facts hold.
  *Recommend:* accept. **Ruling:** ___

- **C75 — Pricing lives in `acquisition-plan` as a module; `PricePlan` is built operation-specific, and what the two plan-bearing consumers genuinely share is recorded after it exists, not presumed.** The hypothesis to test: the shared thing is the compile → review → apply loop and an envelope discipline (stamp, strict round-trip parse, provider and uuid, basis, precondition set, non-actions, counts), and not the action vocabulary or the apply target. A crate split waits for a dependency-graph property it would buy. *Why:* generalize after two consumers reveal the shared property (P3); pattern 9 predicts a family sharing a discipline, and ruling "never a grammar" up front would prejudge the test. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C78 — Every applied price plan leaves an intent receipt in the same transaction, from which a conditional inverse can be compiled.** A receipt (annotations v3) holds the plan's hash over its canonical serialization, when and through what it was applied, its origin, its counts, and each mutation with prior and written values; a no-op plan leaves none; a row's `applied_plan` (C65) names its receipt. `revert` compiles a new plan against current revisions and refuses whole if any row moved — history is evidence, never replayed. A row-granularity event log is not built. *Why:* the driver must answer "did that land?" and "undo it" for a batch of hundreds; the parked event log's trigger fires at pricing (pattern 5). *Details:* `annotations.rs` doc. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* "repriced since T" and "undo the import" are reads over receipts; non-actions are kept as counts, not rows, so an import's refusals live in the plan the human reviewed, not in every receipt.
  *Recommend:* accept. **Ruling:** ___

### Lines for `decisions/plans.md` (the two open topics)

- **C76 — One door, as direction: an explicit selection (`--tabs`, `--all`, `--characters`) compiles to a plan through the same envelope and apply, and the ad-hoc `refresh` kind retires when nothing submits it.** The selection plan's precondition set is empty (no stored intent was derived from); its listing, freshness and two-cycle semantics are ruled in its own slice, with `tools/persist-check.sh` moved onto it. *Why:* two doors to one task; the empty set is the third data point for pattern 9's precondition set (policy: one; price: many; selection: none). Four tracer runs not using the old door are weak evidence about one-off workflows, so the semantics wait for their slice. — `decisions/plans.md`
  *Recommend:* accept the direction; build after pricing. **Ruling:** ___

- **C77 — Freshness is evaluated at compile time against the window; a later replan that refetches facts which aged past the window during a long cycle is correct, not a fault.** A plan does not pre-fetch what is fresh at compile. Naming the facts that will age during the cycle (`aging`) belongs to a quote-bearing plan whose cycle estimate is trusted, and waits for that evidence. *Why:* the 2026-09-02 rerun's 6-request cycle 2 was the window doing its job; an offline plan has no honest cycle estimate; a cycle-based handle ("fresh as of every run") is the C++ model the policy replaced (C36). — `decisions/plans.md`
  *Recommend:* accept. **Ruling:** ___

### (d) Recorded rejections (so nothing is re-argued or adopted by not noticing)

- **The game-price lock** (C++: a game-set target refuses edits) and **unconditional shadowing** (a game price wins at equal grain). Both rejected by C69: the row is stored and the relation reported; refusal protects one ordering, shadowing invents an authority rule.
- **"Priced tabs are always refreshed"** (C++ `SetRefreshLocked`). Rejected by C72: intent never rewrites intent; C34's authority rule.
- **Materialized inheritance** (C++ `PropagateTabBuyouts`, the `inherited` bit). Rejected by C64: a derivation stored as data.
- **`~c/o` / current offer** as a value. Retired by C67; the census counts what the owner's store holds.
- **A fused "render and post"**. Rejected by C74 and the 00 gravity warning: the render is a derivation, the posts outward effects outside the choke-point invariant.
- **Prices as floats.** Rejected by C67: exact decimal in canonical text.
- **Keys on the GGG id alone** (the C++ upsert shape). Rejected by C67: the store already learned that a location is its full coordinate (C54).
- **A pricing crate now.** Rejected by C75 until a dependency-graph property asks for it.
- **A row-granularity annotation event log.** Rejected by C78 in favour of receipts; trigger below if receipt granularity proves too coarse.
- **The C++ shop's silent skip** of an item it cannot index. Rejected by C74: every omission is counted with its reason.
- **A stored situation cache.** `price status` is recomputed on every read (C48's spirit).

## 2. The slice: pricing, offline, validated by real use

Each step gate-green (`AGENTS.md`, run bare); observable behavior
unchanged until step 4. Findings from reviews go to a
`PRICING-SLICE.md` closed record in `REFRESH-SLICE.md`'s shape as they
happen, never to `CONTEXT.md`. Two ruling moments: the framework now;
the value grammar and the shop's publication rule after the census
(2b) and the forum reading (8).

1. **Harvest.** Accepted lines into `decisions/pricing.md` and
   `decisions/plans.md`, the index row in `CONTEXT.md`, the parking-lot
   entries below replacing the 2026-08-31 pricing entries. Nothing
   narrative.
2. **Audit, review, census — before the file carries a price.**
   (a) Clause audit of `decisions/plans.md` and C35: every clause of
   C35, C36–C44 either names a test (`c<n>_…`) or a doc comment cites
   it; `tools/docs-check.sh` reports C36, C39, C40, C41, C42, C52
   uncited today. (b) A targeted review of `annotations.rs` as the one
   durability boundary about to become load-bearing: the CAS under a
   batch, tombstone generations as preconditions, the v2 → v3 migration
   path, WAL and busy-timeout under two writers, export under load,
   `list` at thousands of rows, receipt growth. (c) **A read-only
   census** of the owner's real 0.18 userstore under a consistent
   snapshot (rows by `type`, `source`, `inherited`, `currency`,
   `location_type`; amount shapes; `~c/o` count; character and
   remove-only-tab rows) and of the current facts (`metadata.public`
   presence, tab index, positions, containers, socketing, and **every
   tilde-prefixed item note and tab name, parsed or not** — the corpus
   of current price formats) — the evidence for C67's grammar edge,
   C68's alias table and C74's eligibility reasons, recorded in
   `PRICING-SLICE.md`. Findings table opened.
3. **Annotations v3 + typed intent** (C65, C66, C71's store half,
   C78's table). `written_via`/`actor`/`applied_plan` columns with the
   `unknown_legacy` migration; `put_intent`/`get_intent` over the
   per-kind trait; tombstone revisions readable; `apply_batch`
   (preconditions, mutations, provenance, receipt) in one transaction;
   scope `substash` and `character`. `SyncPolicy` moves onto the typed
   path with no behavior change — the factoring rule's first payment.
   Pinned by id.
4. **Reference data, the target and the value** (C67, C68). The
   currency table with its version and digest and its enumeration; the
   `PriceTarget` and `Buyout` v1 strict types with the canonical amount
   (grammar edge ruled from 2c); the game-observation parser over tab
   names and item notes (the C++ regex and alias table are the rule
   evidence; the parser is a derivation over facts).
5. **The listing state and the situation** (C69, C70). One function
   over the store's items, tabs, characters, `metadata.public` and the
   buyout rows: manual side, game side, relation, causes, revisions,
   basis; `acq price status` (with the unparsed-note tripwire), `acq
   price show <target>`, `acq price list` with filters, under C53's
   three levels. Handles pinned: one
   location price, one inherited item, one item override, a substash
   under a priced parent, a `conflict`, an `agree`, an orphaned item's
   row, an unknown currency tag.
6. **`PricePlan`, apply, receipts, revert** (C71, C72, C78). Desired
   state from handles (`acq price set|clear <target> …`, immediate or
   `--plan`) and from a JSON document (`acq price plan FILE`, the MCP
   form); `acq price apply=FILE`; `acq price revert <hash>`; `acq price
   history [--since]` over receipts. Pinned: the stale-revision
   conflict, the tombstone-generation conflict, the duplicate target
   refused, the all-or-nothing batch, the no-op plan that leaves no
   receipt, rows-as-written plus basis in the result, revert of an
   applied batch and its refusal after a row moved, the receipt (a
   row's `applied_plan` names it).
7. **The import** (C73) from the owner's 0.18 userstore file, then
   **validation reading 1**: the owner runs it on the real store, reads
   the plan, applies it; every source row's outcome is in the plan; a
   second run is all `unchanged`; a revert and re-apply round-trip is
   rehearsed on a copy first. Evidence the run collects: whether
   remove-only tab ids matched, how many rows were `not_in_facts`
   against a fresh refresh, the realm and account-binding statements,
   and the owner's words, verbatim, from the conversation (pattern 6).
8. **`shop render`** (C74, C72's report) with eligibility and omission
   counts, then **validation reading 2, run as an experiment**: the
   owner pastes the rendered page by hand for their own shop and works
   a matrix with junk items — in-game note only; forum only; both,
   agreeing; both, conflicting; `~price` against `~b/o`; a fractional
   amount; a note in a private tab; a currency the C++ table lacks —
   then observes what the trade site shows after an indexing wait,
   repeated once. Each cell becomes a numbered trade claim; the forum
   mechanics and the publication rule for `conflict` and
   `game_listing_public` are ruled from those claims, not from C++.
   Further evidence: link codes for substash and character items, page
   splitting, whether the coverage and staleness lines change what the
   owner does next. Nothing in the system sends anything.
9. **MCP** (`price_status`, `prices`, `price_plan`, `price_apply`,
   `price_revert`, `shop_render`, the currency resource) as thin
   adapters over the same functions — the "no parallel semantics"
   check, as in tracer step 8; the agent drives the loop end to end in
   mock mode from a fresh context, and what it had to re-read is the
   evidence for the read-economy hypothesis. Then the C75 finding: what
   the two envelopes genuinely share is recorded, and factored only if
   it is literal duplication.
10. **Close.** `PRICING-SLICE.md` (step ledger, findings, what the
    census and readings taught, observations still open); the trade
    claims authored master-side and cherry-picked; the pattern-9
    verdict recorded as a ruling or a parking-lot trigger; whether
    receipts answered every "since" question the readings asked; the
    read-economy verdict.

**Done criterion:** the owner's manual buyouts are in the intent file
through the import with every source row accounted for, a second run
inert, and a revert rehearsed; a page rendered from them reads correct
against the forum in the owner's words, with every omission explained;
the eight handles, the atomic batch, the receipt and the revert are
pinned by id; an agent in mock mode drives observe → state → compile →
apply → observe-the-delta and its re-reads are recorded; and every
"what was repriced since T" the readings asked was answered from
receipts — or the row-granularity trigger has fired with the evidence.

**Anti-scope, with triggers** — in the parking lot below. Nothing in
this slice touches the daemon, the limiter, the network layer, or a
GUI.

**The method test this slice runs:** whether Plan is a family sharing a
compiler and an envelope discipline (pattern 9), tested by building the
second document without looking at the first's grammar until step 9;
whether the intent layer holds under thousands of rows and three
writers; whether provenance, strict values and receipts were cheap now,
as claimed; whether the census-then-rule order beats ruling from the
C++ code; and whether the read economy held, measured in step 9.

## 3. Parking lot (landings named; deferral never re-argued)

Replaces the pricing, legacy-import, shop and annotation-event-log
entries of the 2026-08-31 parking lot once the slice is harvested; the
rest of that lot stands.

- **Shop / forum publishing** (POESESSID, thread numbers, one post per
  page, auto-post after a clean refresh) → outward credentialed traffic
  outside the API choke-point invariant; its own boundary session
  before any code — the third apply target of the one loop, with its
  own cost dimension. Trigger: the render validated and the owner
  wanting the posts automated.
- **User-scoped intent** (shop template, currency ratios, saved
  searches) → `user.db` + scope taxonomy. Trigger: the first user-scoped
  kind actually written; v1 takes the template from a file.
- **Row-granularity annotation history** → trigger: a "since" question
  receipts (C78) cannot answer at the readings, or a conflict whose
  resolution needs more than the two plans involved.
- **One change cursor over facts and intent** (`item_events` and
  receipts read as one "since" stream) → trigger: the step-9 agent run
  shows two reads where one would do.
- **The read economy as a ruling** (summary by default, filters,
  bounded detail) → trigger: step 9's re-read record; until then it is
  surface design under C53.
- **Batch pricing by query** (price everything a search selects) →
  lands on search semantics. Trigger: a real batch workflow the handles
  and the JSON document cannot express (08 boundary 8).
- **`current_offer` as a value** → trigger: the census finding rows the
  owner wants kept, or a real use.
- **The amount grammar's edge** (ratios, precision bound) → ruled from
  the census (step 2c) before Buyout v1 freezes; not parked past it.
- **PoE2 currencies and per-realm tags** → trigger: a PoE2 stash
  endpoint or a poe2 price the owner wants to set; the table carries
  realm applicability from day one so this is rows, not shape.
- **Coverage advice in `refresh --plan`** (the planner naming priced
  locations outside coverage) → trigger: `price status` and the
  render's report proving the wrong place for it.
- **`aging` in the plan** (C77) → trigger: a quote-bearing plan whose
  cycle estimate is trusted.
- **Currency totals and history** → a derivation over facts, cheap;
  after pricing stands.
- **The explicit-selection door** (C76) → its own slice after this one,
  where its listing, freshness and two-cycle semantics are ruled.
- **Name→id resolution for targets** (`price set tab:"Maps"`) →
  trigger: authoring friction at the CLI, the same rule as C63.
- **Policy merge as a remedy** (the coverage gap's remedy today prints
  the whole edited policy because `policy set` replaces) → the parked
  per-realm merge; trigger unchanged (a second realm in daily use), or
  the remedy proving unusable at the readings.

## 4. Questions for the owner

1. C69: the relation model — two sides and a relation, the conflicting
   row stored — in place of both shadowing and refusal. Accept?
2. C74: render in scope, with the publication rule for `conflict` and
   `game_listing_public` deliberately left to the forum reading?
3. C77 as narrowed (window ruled, `aging` parked)?
4. The order: audit and census before any schema; import (reading 1)
   before render (reading 2).
5. C67: `current_offer` retired, the canonical amount ruled now, the
   grammar's edge ruled after the census?
6. C78 as narrowed: receipts with a compilable conditional inverse, in
   the intent file, before the first import?
7. C65's `actor`: keep the optional claimed identity, or channel only
   until a frontend actually has one?
8. The home for trade and forum facts: a second ground-truth file on
   master (`trade-ground-truth.md`, claims `T<n>`), or a section of the
   network one?
