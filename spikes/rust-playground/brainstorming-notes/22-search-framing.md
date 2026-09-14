# 22 — Framing for the item-search design (draft for the owner's edit)

**Drafted 2026-09-13 by Fable, before the session, in the mold of the
2026-08-31 framing (note 00, at `a6a8f53e~1`).** The owner edits this in
place — the headline stances in particular are his to write — and the
proposal stage does not run until he has. Read after `search/DIGEST.md`;
like every note here it is disposable: anything it proposes becomes real
only as a ruling in `decisions/search.md` or `CONTEXT.md`.

## The goal function

The owner's words, 2026-09-13: the research is synthesized "into a
powerfully simple, usable, comprehensive system of search that eschews
overcomplication and tricky edge cases and implementations." Not a port
of the C++ search, not a copy of the trade site, not a thin shell over
the store. The synthesis of ten sources might land somewhere
unexpectedly better than any of them; that possibility is the point of
the session, and this framing exists to protect it.

Search is "the real heart of acquisition" (the owner, 2026-09-11). It
is the first slice whose validating consumers are two seats at once —
the owner at a terminal and an agent over MCP — and the first whose
product is a *model*, not a verb: the query, the vocabulary, the answer.

## Headline stances (owner — to write)

These go first so they cannot be buried. Candidates drawn from the
owner's own words this week, each to keep, amend or strike:

1. *"We need a system that is flexible, simple, and generalizable here,
   not one tuned to a specific set of questions from a specific single
   user."* (2026-09-13) — the seat questions test a model's reach;
   they never specify it.
2. *"There are many times where a specific modifier value is needed."*
   (2026-09-13) — a mod's value is a first-class predicate.
3. *"Legacy means an item or modifer or modifier value or unique or
   unique variant that cannot be found or created by playing the game"*
   and *"We should let users provide that knowledge for now"* — game
   knowledge the API does not carry is the user's input, never the
   app's.
4. *"let's keep min and max separate."* (repoe Q5) — a two-value line
   is two values.
5. *"I do not want acquisition updating forum shops directly"* — and no
   pricing engine; a trade query out and a trade URL in are the
   boundary, and integrations (Better Trading, Awakened) are "fun, but
   not a core features".
6. What "powerfully simple" means when you are holding it. (Blank on
   purpose. The proposals are judged by this line.)

## The ten sources, and the one rule about them

**Let each source be authoritative only where it is evidence.**

| Source | Evidence of | Not evidence of |
| --- | --- | --- |
| item-facts | what an item *is* as GGG gives it, and how big the corpus is | what anyone asks of it |
| cpp-search | rules users relied on (pseudomods, the six buckets, the columns); what instant cost | shape — three of its twenty min/max filters were dead against real payloads and nobody noticed |
| trade-query | the grammar players already know, and the boundary (a URL is a query) | a stash: 7 of 93 filters read a listing, and it composes stats only |
| repoe | what the game knows behind the text: the stat spine, the taxonomy, the ranges | a runtime dependency — its access method is unruled (Q8) and the stat id names 67 % of lines |
| item-filter | GGG's own naming of the predicates; the divergences between the three namings | a query language — it ANDs and stops at the first match, and it withholds numbers on purpose |
| prior-art | how a maintained tool survives the moving text: identity as the displayed line, twins resolved by category, unknown lines shown | a corpus, a saved query, a stash search |
| store-as-built | what a search gets today, and where the first change falls (a read) | what should be |
| engine-bench | that no engine beyond a scan is needed, that the load is the cost, that identity moves reach not latency | who holds the corpus — that is a design question |
| owner-seat | the human test: seven questions, eight requirements, three non-goals | the specification (stance 1) |
| agent-seat | the agent test: schema discovery, facets, stable ids, explain, refinement, the failure of a query that pulls fifty thousand rows | (not yet run) |

When a source speaks outside its lane, discount it.

## The settled floor (not up for debate here)

Naming what is settled is what makes it safe to be radical everywhere
else:

- The five invariants; the daemon owns GGG traffic and never reads
  facts (C2, C34); a frontend consumes two surfaces (C12); shared
  semantics live in Rust and every frontend has an adapter (C46).
- Raw SQL is not a surface, and there is no cached search service —
  with the reopening trigger now *measured*: engine-bench says the
  per-command load is 202 ms at the real corpus and 5.9 s at a million
  items (C48; `decisions/store.md`, "Parked: search-at-scale"). The
  design may reopen it with that number, and must say so if it does.
- Annotations are the only irreplaceable state (C35); a saved search,
  if one exists, is intent and lands there or in the user-scoped home
  the store has parked.
- Governed surfaces stay governed (C79): the trade site, RePoE, Awakened
  are read as the register says, never fetched by the app.
- The realm is a coordinate above league (the characters ruling); PoE2
  is an axis, not a fork.
- Min and max are separate (stance 4); publishing is parked.

## Shared vocabulary: one thing, four names

Do this mapping before debating proposals, or the session will argue
about spellings:

| Thing | API / store | C++ | trade | filter language |
| --- | --- | --- | --- | --- |
| a displayed mod line | `explicitMods[].description` | template (`#`) | stat id + text | `HasExplicitMod` name |
| the kind of a line | the array, plus `flags` | six buckets | 14 categories | — |
| the item's class | none (PoE1); property 109 (PoE2) | RePoE class, substring | 83 category ids | `Class` |
| rarity | `frameTypeId` | `frameType` | `rarity` option | `Rarity`, ordered |
| the Foulborn flag | `mutated` | `Mutated` | `mutated` ("Foulborn") | `Foulborn` |
| defence | `Armour` property (total) | `Armour` | `ar` (20 % quality normalised) | `BaseArmour` (base) |
| where it is | realm, league, location, container | tab header | — | — |

The full tables: item-filter F2–F3, repoe F3, trade-query F5.

## Triage: four buckets before any evaluation

- **(a) Already built, or a read away** — the corpus and every field
  (store-as-built: pull, then filter in the frontend); the location
  read that does not exist but whose index does.
- **(b) Fits an open topic** — search-at-scale (now measured); the
  user-scoped annotations home whose trigger lists saved searches; the
  MCP pricing read model parked behind "item search's read model
  landed".
- **(c) New scope** — a trade URL in, a trade query out; Awakened for
  pricing; Better Trading. Real, and must not crowd out (b).
- **(d) Conflicts with a ruling** — a cached search service without the
  reopening; the daemon reading facts; RePoE fetched at runtime; the
  app judging legacy.

## Convergence signals

Independent agreement is the strongest evidence in the pile:

- **Several ids for one line is the normal case**, not an edge: the
  site (380 collisions), the export (150 ambiguous templates), Awakened
  (twins as a two-stat group, several ids emitted as a `count` OR).
- **The text is the moving part**: 2,436 re-wordings under stable trade
  ids (prior-art), 229 renamed lines in one capture (repoe F1), the
  string-to-object format change of July 2026 (item-facts F3).
- **Identity finer than text**: owner R1, Awakened's ref plus matchers
  plus category, the site's 14 categories.
- **Composition across fields is missing everywhere** (owner F3): the
  site composes stats only; the C++ app ANDs.
- **Every answer wants its location** (owner R6; the read the store
  lacks, store-as-built F2).
- **The load, not the query** (engine-bench F3; cpp-search F6: the
  filter loop was never the cost).
- **Unknown is shown, never guessed** (prior-art F4; the C++ dropdown's
  failure by contrast).

## Synthesis seeds — where "unexpectedly better" might live

- **The query as a value.** Immutable, serializable, named: saved,
  shared, sent to the site as a search, received from the site as a
  URL, handed from an agent to a human. C38's plan is the mold — an
  object every frontend can carry because it is data.
- **The vocabulary is a read.** Autocomplete for the owner and schema
  discovery for the agent are the same read (C53's views): what lines
  exist in *this* corpus, by kind, ranked by how many items carry them.
  The site's instant categorised autocomplete, built from the corpus
  instead of a 14,000-key list.
- **Identity is the template plus its kind; the stat id lives at the
  boundary.** Awakened's way, and engine-bench says it costs nothing:
  the template names every line, the stat id names 67 %, and only the
  trade boundary needs the id.
- **Limits are an output.** The search says what it does not promise —
  an unrecognised line, a legacy it cannot judge, a league of origin
  no field carries — instead of silently returning less.
- **The holder.** A corpus parsed once and held is what makes every
  query free; at the real scale a per-command parse is cheap. The
  design says who holds it, for how long, and what a restart costs.
- **One grammar, four adapters** (C46). The CLI's syntax, the MCP tool's
  arguments, a GUI's form and a trade URL are four renderings of one
  query type.

## Gravity warnings

- **C++ detail gravity.** 38 filters and 26 columns are the most
  detailed source and will dominate an unframed discussion. Extract
  the rules; discard the widgets.
- **Trade-site gravity.** 93 filters and 14,193 stat keys are the
  ceiling of what players already understand, not the target; 27
  filters and four whole categories mean nothing for a stash.
- **RePoE gravity.** The stat spine is seductive and a third of lines
  are outside it; its access method is unruled. A design that needs it
  says so as a decision, not an assumption.
- **Engine gravity.** The instinct to index is the C++ app's and the
  bench says no. Reopen it with a number or not at all.
- **Edge-case gravity.** Twice-numbered stats, the 83 unreachable
  collisions, the veiled placeholder, `[Tag|Display]` markup: each is a
  limits-register row unless a `main` claim depends on it.
- **One user's questions.** Stance 1. The seven questions are a test.

## The acceptance test, and simplicity (owner — to edit)

A proposal passes when every owner-seat question and every agent-seat
scenario is expressible in its model, written out in the appendix, and
every limit it inherits is named. Simplicity is judged, not capped:
each proposal carries a concept inventory — every concept named, with
one line on why the model cannot do without it — and the reconciliation
compares inventories. **Ruled 2026-09-14: judged, not capped** — the
owner, offered capped, judged, or both: "I agree with you on 1." No
concept limit binds a proposal; the inventory and the appendix are
what the reconciliation weighs.

## The questions the design must answer (ranked)

1. What is a line's identity, and what does the search do with a line
   it cannot name?
2. What is the query model — predicates, composition, tri-state flags,
   values, the location coordinates — and what is its one grammar?
3. What is the vocabulary read, and how is it served to a human and an
   agent from the corpus?
4. Who holds the corpus, for how long, and what does a restart cost?
5. What crosses the trade boundary, in each direction, and what does
   not?
6. What does a result carry — the item, its location, its freshness,
   and how much of the body?
7. What are the non-goals and the limits, written as outputs?

## Output shape (stage 3, each proposal)

The model on one page; the grammar in as few lines as it takes; an
answer to each question above; the appendix (every acceptance question
in the model's notation); the concept inventory; what the design
refuses; the decisions left to the owner. Under 10 KB. Two proposals,
blind, then reconciliation (the plan of 2026-09-13).
