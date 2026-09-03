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

Tentative ids `C64`–`C79` are assigned in reading order; at harvest the
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
irreplaceable state. Some of that evidence lives on surfaces GGG does
not sanction — the trade site, the forums, third-party feeds — and
Acquisition has used such surfaces since before the API existed. They
are governed, not forbidden (C79): human-run, off the runtime path,
named, and cited. Four things make the discovery deliberate:

- **Trade and forum facts get numbered claims**, authored master-side
  in their own file (`docs/design/trade-ground-truth.md`, claims
  `T<n>`, separate from the network claims because their volatility
  and consumers differ), cherry-picked here and cited by number. Only
  provider facts go there; what Acquisition does about them is a
  pricing ruling.
- **The forum matrix is run by an instrument, not by the product**
  (step 8): a fixture renderer in `tools/` emits every matrix cell by
  explicit instruction, eligibility ignored; the owner posts by hand
  with junk items in a disposable thread and records realm,
  visibility, timestamps, the indexing signal and the observation. A
  negative cell is a claim only after a control item is seen indexed
  inside the same window. Production `shop render` receives the ruled
  policy afterwards and reports anything unruled as blocked — the
  derivation never generates the evidence that justifies itself.
- **The value type is broad before the grammar is known**: `amount` is
  an exact rational, so every matrix cell (decimals, fractions) is
  representable from day one; what the renderer emits and what the
  parser accepts are narrowed from claims, never the stored type.
- **The currency table is a reviewed, committed table whose every row
  cites its evidence** (C68): an official data export where one
  exists, a claim, or a recorded human observation — the trade site
  read in a browser counts, under C79. A tool may *propose* rows from
  a governed source; a human commits. The owner's own tab names and
  item notes are the free corpus of current formats, read at the census
  (step 2c) and watched by a tripwire after it (C69).

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
| Reference data | the currency table, versioned by the build (C68) | a reviewed commit |

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

- **C65 — Every intent write carries structured provenance: the channel it came through (`written_via`), an optional claimed `actor`, and the hash of the plan that landed it (`applied_plan`, C71).** Stored on the row (annotations v3), returned on every read, required by the write API; `actor` is untrusted audit metadata, never identity or authorization; origin detail lives on the receipt (C78), never per row. Rows written before v3 migrate as `unknown_legacy` with no plan — a migration never manufactures a writer or a receipt. *Why:* pattern 4 — who set a price and through what cannot be reconstructed later, and C14 makes agents writers of intent. Cheap today, unrecoverable after the first import (P3). *Details:* `annotations.rs` doc, C65. — `decisions/pricing.md`
  *Recommend:* accept, before the first price is written. **Ruling:** ___

- **C66 — Intent values are typed at the write API: a kind declares its schema version and a strict parser, a value that does not parse under its stamp never lands, and a current-schema value re-serializes to exactly what was read.** The generic — version gate, unknown fields refused at every depth, exact round-trip, then compare-and-swap — is factored out of the sync policy's parser into the store crate over a per-kind trait; each kind's shape stays its owner's; an older stored value upgrades in memory, its raw JSON untouched. *Why:* a value shape that changes after data exists is a migration of the irreplaceable state (pattern 4), so strict-from-day-one must be structural, not each frontend's discipline. *Details:* `annotations.rs` doc, C66. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C67 — The `buyout` value, v1, and its target.** A typed `PriceTarget` — item and character by id (C55), tab by `(realm, id)`, substash by `(realm, parent, id)`, league absent so intent follows a tab through a league merge — is the public API; frontends never build raw keys. `type` is `buyout`, `fixed`, `no_price` or `ignore`; the first two carry `amount` and `currency` (a reference tag, C68). `amount` is an exact positive rational in one canonical text, never a float; emitted and accepted forms are narrowed from trade claims, the stored type is not. `current_offer` is refused at write, kept at import as a non-action. *Why:* the key mirrors the store's identity (C54, C58); exact arithmetic without float rounding. *Details:* `acquisition-plan` doc, C67. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* canonical text = digits with at most one point, or `a/b` reduced with `b > 1`; no sign, no exponent, no redundant leading zeros (`0.5` keeps its zero), no trailing fractional zeros, non-zero; digit and scale bounds once evidence supplies them; equality is on the reduced rational. An imported `REAL` converts by its shortest round-trip decimal — an honest conversion, not recovery of lost text. `no_price` and `ignore` carry neither amount nor currency, and their presence is a parse error.
  *Recommend:* accept; the grammar's edge is the one deliberately open clause. **Ruling:** ___

- **C69 — An item's listing is two independent resolutions and their relation, never one scalar.** The manual side resolves by specificity (C70); the game side resolves item note, then tab name, and carries whether its stash is public. The relation is manual-only, game-only, agree, conflict or none; `ignore` is a disposition on the manual side and never denies the observed game price. Every result carries both sides with causes, revisions, basis, age, parser and reference versions. What a relation *means* is each consumer's rule as a Rust derivation (C74), never a frontend's. *Why:* game observation, forum intent, disposition and renderability are four statements; C++ fused them and needed locks; "game wins" had no evidence. *Details:* `acquisition-plan` doc. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* a read's output is a plan's input — the revisions and basis on the result become preconditions and provenance. Item-note precedence has four classes: absent or ordinary text falls through to the tab; a recognized price note is used; a price-looking note that does not parse is reported and the tab price is *not* substituted; a recognized syntax with an unknown currency keeps the parsed structure and the tag, reported. A tilde-prefixed note or tab name that parses as nothing is counted and inspectable in `price status` — the drift tripwire (C56) applied to price formats.
  *Recommend:* accept. The conflicting row is stored and reported as `conflict`. **Ruling:** ___

- **C70 — A priced tab covers that tab and its children, the way a policy id does (C37);** a substash row overrides its parent's for that substash; an item row overrides both; a character row covers its items. Coverage is the manual side's inheritance only — it says nothing about eligibility (C74). *Why:* the C++ store already lands substash items' location prices on the parent, and the house rule for tab-scoped intent should not have two shapes. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C71 — A `PricePlan` compiles a desired state against a named snapshot and is applied atomically to the intent file; its preconditions protect intent, and fact drift is reported, never a gate.** Desired state (handles, a document, a foreign store) compiles into the envelope, a precondition set (one revision per row touched, tombstone generations included, with the prior value), the mutations, non-actions with reasons, and mutation counts as its cost; duplicates are refused. Apply checks every precondition and cited reference tag in one transaction, all or none; the result is every row as written and whether facts moved. *Why:* what was reviewed is what lands (C38); C44 and C52 are this set with one element (pattern 9). *Details:* `acquisition-plan` doc. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* envelope = provider, uuid, operation, schema stamp, fact and reference bases; a never-existed row's revision is 0 and a deleted row's is its tombstone's, so "one moved revision refuses" is exactly true; counts = create, update, clear, unchanged, refused. Facts and intent are separate files and never share a transaction: the result's fact report is an observed-after basis and a changed/unchanged indication, not a snapshot. Reference compatibility = every cited tag still resolves with the same realm and meaning (C68).
  *Recommend:* accept. **Ruling:** ___

- **C72 — Pricing never edits the sync policy, and a price never locks a tab into refresh.** The relationship between the two kinds of intent is reported, not enforced: the consumer that needs freshness — `shop render` first — names priced locations outside the policy's coverage and priced facts older than its stated window, each with the remedy (the policy edit, or the `RefreshPlan` it would take, C41). *Why:* C++'s "priced tabs are always refreshed" is one kind of intent silently rewriting another; a report keeps the policy what its author wrote (pattern 10, 08 boundary 7). — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C73 — The legacy import is a source of desired state for the one compiler (C71), read from the 0.18 userstore under one consistent snapshot, with every source row accounted for in the reviewed plan.** Manual rows are the desired state; inherited, game, auto and `~c/o` rows are non-actions with those reasons; a target the facts lack is a non-action (remedy: refresh); an existing row is never overwritten; the realm is an explicit parameter. The file names a username and no uuid: the plan states the claimed binding, its evidence and confidence; an unverifiable binding needs acknowledgement at apply; only contradiction refuses. *Why:* the wizard dissolves into the plan/apply path; a refusal keeps what it refused (pattern 2). *Details:* `acquisition-plan` doc. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* the userstore is WAL, so a hash of the `.db` file can omit uncheckpointed content — the receipt's origin digests the rows as read. Binding confidence: `verified` (the filename's username maps to this uuid in the account index, current or former name), `corroborated` (a stated share of the source's item and location ids are in this account's facts), `unverified` (neither; apply requires an explicit acknowledgement), `contradicted` (the name maps to another uuid on record, or a large source overlaps nothing) — the last refuses. An equal existing row is `unchanged`, a differing one `existing_differs`; a second run is all `unchanged`; `REAL` amounts convert per C67. A `not_in_facts` row keeps enough detail to recover it after a refresh.
  *Recommend:* accept. **Ruling:** ___

### (c) New scope

- **C68 — Reference data is a fifth input, versioned by the build: a reviewed, committed table whose every row cites its evidence, shipped inside the binary, read-only, never in a store file, enumerable through every surface, cited by version wherever used.** A tool may propose rows from a governed source (C79); a human commits. The currency table is first, separating the immutable tag intent cites, the text the renderer emits, and the aliases a parser accepts (the last two from claims). Evolution is by semantic identity: a tag is never removed or reused; additions are reported. *Why:* not an account fact, not intent, not a derivation — the tower had not placed it (pattern 8); an evidence-free table is how the C++ list rotted. *Details:* beside its consumer. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* a currency row = stable tag (`chaos`, `divine`, …), display name, emitted text, accepted aliases, realm applicability, evidence (an official data export, a `T<n>` claim, or a dated human observation); the table's version and digest join the basis a plan or render stamps, and apply checks that every cited tag still resolves with the same realm and meaning (C71) — the digest is recorded, the semantic check gates.
  *Recommend:* accept now, at the first meeting. **Ruling:** ___

- **C74 — `shop render` is in scope as a derivation, publishing is not, and every item the page omits is counted with a named reason.** The page is a pure function of facts, intent, reference data, a template and a ruled publication policy; it sends nothing; the template is a render-time input, not stored; the owner pastes by hand. Nothing is skipped silently; a relation the policy has not ruled is reported as blocked, never guessed. The policy and the forum mechanics are ruled from trade claims gathered by a separate instrument, never from the renderer's own output. *Why:* every human surface is a derivation over a machine surface (pattern 3); the posts stay parked behind their own boundary session. *Details:* `acquisition-plan` doc, C74. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* eligibility reasons in view — a disposition, a conflict (C69), a public game listing, socketing, an unsupported container, a missing position or tab index, removal, an unknown currency, blocked-unruled; the policy is a table from relation to include-or-omit-with-reason, each row citing the claim that rules it; the census (step 2c) checks which shapes the real facts hold. Forum mechanics in question: spoiler grouping, link codes, the post limit, page splitting, the hash.
  *Recommend:* accept. **Ruling:** ___

- **C75 — Pricing lives in `acquisition-plan` as a module; `PricePlan` is built operation-specific, and what the two plan-bearing consumers genuinely share is recorded after it exists, not presumed.** The hypothesis to test: the shared thing is the compile → review → apply loop and an envelope discipline (stamp, strict round-trip parse, provider and uuid, basis, precondition set, non-actions, counts), and not the action vocabulary or the apply target. A crate split waits for a dependency-graph property it would buy. *Why:* generalize after two consumers reveal the shared property (P3); pattern 9 predicts a family sharing a discipline, and ruling "never a grammar" up front would prejudge the test. — `decisions/pricing.md`
  *Recommend:* accept. **Ruling:** ___

- **C78 — Every applied price plan leaves an intent receipt in the same transaction, from which a conditional inverse can be compiled.** A receipt (annotations v3) holds the plan's hash over its canonical serialization, when and through what it was applied, its origin, its counts, and each mutation with prior and written values; a no-op plan leaves none; a row's `applied_plan` (C65) names its receipt. `revert` compiles a new plan against current revisions and refuses whole if any row moved — history is evidence, never replayed. A row-granularity event log is not built. *Why:* the driver must answer "did that land?" and "undo it" for a batch of hundreds; the parked event log's trigger fires at pricing (pattern 5). *Details:* `annotations.rs` doc. — `decisions/pricing.md`
  *Mechanism (to the code at harvest):* "repriced since T" and "undo the import" are reads over receipts; non-actions are kept as counts, not rows, so the reviewed plan is the artifact that explains an import's refusals — which makes its retention a visible surface: an import always writes its reviewed plan to disk beside the store, named by hash, before apply, and the receipt cites that path.
  *Recommend:* accept. **Ruling:** ___

### A line for `CONTEXT.md` (cross-cutting)

- **C79 — Surfaces GGG does not sanction — the trade site, the forums, third-party feeds — are governed inputs, never runtime dependencies.** The daemon never touches them (invariant 1 covers the API; this covers the rest); no store read, plan compile or apply depends on one; each is registered with its status, terms exposure and cadence; it is consulted by a human or human-run tooling at authoring or experiment time, and what it yields lands as claims or reviewed reference data, source cited per row. One used as an *effect* (a forum post) needs its own boundary session first. *Why:* Acquisition predates the API and has always used such surfaces; the relationship is protected by keeping them deliberate and off the runtime path, not by pretending they are unused. — `CONTEXT.md`
  *Recommend:* accept; the first registered surface is the trade site, read in a browser for currency evidence and the forum matrix. **Ruling:** ___

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
- **An automated fetch of the trade site's internal static data as the currency table's source.** Rejected by C68 and C79: the endpoint is not in GGG's API reference or data exports, and automation is not more authoritative than a curated, evidence-cited table; a human may read it as evidence, a tool may propose from it, the product never depends on it.
- **A whole-file digest as the reference-data gate.** Rejected by C68: an added currency must not invalidate an old price plan; the semantic check on cited tags gates, the digest is recorded.

## 2. The slice: pricing, offline, validated by real use

Each step gate-green (`AGENTS.md`, run bare); observable behavior
unchanged until step 4. Findings from reviews go to a
`PRICING-SLICE.md` closed record in `REFRESH-SLICE.md`'s shape as they
happen, never to `CONTEXT.md`. Two ruling moments: the framework now;
the emitted and accepted price forms and the shop's publication rule
after the forum matrix (8), with the census (2c) as local evidence.

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
   of current price formats) — local evidence for C67's forms, C68's
   aliases and C74's eligibility reasons (what the owner stored and
   holds, never what the forum accepts), recorded in
   `PRICING-SLICE.md`. Findings table opened. (d) **The currency source
   established** under C79: the trade site registered as a surface;
   whether an official data export supplies the mapping; the v1 table
   drafted with a citation per row from the C++ list, the census, and
   dated browser observations.
3. **Annotations v3 + typed intent** (C65, C66, C71's store half,
   C78's table). `written_via`/`actor`/`applied_plan` columns with the
   `unknown_legacy` migration; `put_intent`/`get_intent` over the
   per-kind trait; tombstone revisions readable; `apply_batch`
   (preconditions, mutations, provenance, receipt) in one transaction;
   scope `substash` and `character`. `SyncPolicy` moves onto the typed
   path with no behavior change — the factoring rule's first payment.
   Pinned by id.
4. **Reference data, the target and the value** (C67, C68). The
   currency table from 2d with its version, digest and enumeration; the
   `PriceTarget` and `Buyout` v1 strict types with the exact rational
   amount; the game-observation parser over tab names and item notes
   with its four note classes (the C++ regex and alias table are the
   rule evidence; the parser is a derivation over facts, stamped with
   its version).
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
8. **The forum matrix, then `shop render`** (C74, C72's report), in
   three parts. (a) **The instrument**: a fixture renderer in `tools/`
   over the shared link-code functions, emitting every matrix cell by
   explicit instruction — in-game note only; forum only; both,
   agreeing; both, conflicting; `~price` against `~b/o`; a fractional
   amount; a note in a private tab; a currency the C++ table lacks;
   substash and character link codes; a page at the post limit. The
   owner posts by hand with junk items in a disposable thread and
   records realm, visibility, timestamps, the indexing signal for a
   control item, and the observation, repeated once. (b) **The
   claims**: each cell a `T<n>` claim authored master-side; from them
   the publication policy for `conflict` and `game_listing_public`,
   the emitted and accepted price forms, and the forum mechanics are
   ruled. (c) **Production `shop render`** built with the ruled policy
   table, anything unruled reported as blocked, then **validation
   reading 2**: the owner pastes the page for their own shop and reads
   it against the forum; whether the coverage and staleness lines
   change what the owner does next is the evidence. Nothing in the
   system sends anything.
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
as claimed; whether instrument-then-rule beats ruling from the
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
- **Third-party price feeds** (market prices, suggested prices) → a
  governed surface under C79 when the trigger fires: currency ratios or
  a "suggest a price" consumer.
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

1. C79 as a cross-cutting line in `CONTEXT.md` (the thirteenth of
   fifteen), with the trade site as the first registered surface?
2. C68's v1 source: a reviewed table with a citation per row, drafted
   from the C++ list, the census, and dated browser readings of the
   trade site, with any tool output treated as a proposal. Accept, or
   name a different permitted source?
3. C67's exact rational amount as the stored type, with emitted and
   accepted forms narrowed from claims?
4. C73's binding model: verified / corroborated / unverified (explicit
   acknowledgement at apply) / contradicted (refuse)?
5. Step 8's split: instrument in `tools/`, claims, then production
   render with a ruled policy table and blocked-unruled?
6. C78's plan retention: an import always writes its reviewed plan to
   disk before apply, cited by the receipt?
