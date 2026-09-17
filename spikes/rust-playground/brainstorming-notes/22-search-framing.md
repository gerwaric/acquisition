# 22 — Framing for the item-search design

Read after `search/DIGEST.md`; like every note here it is disposable:
anything it proposes becomes real only as a ruling in
`decisions/search.md` or `CONTEXT.md`.

## The goal function

The goal is to build "a powerfully simple, usable, comprehensive system
of search that eschews overcomplication and tricky edge cases and
implementations." It should be intuitive for agents to use and support
multiple clients such as agents and user interfaces.

We are not here to select one of the researched approaches, but to
synthesize a holistic search design. If it borrows from any of the
research tracks, that's great, but we should think tabula rasa about
this aspect, because the synthesis of ten sources might land somewhere
unexpectedly better than any of them; that possibility is the point of
the session, and this framing exists to protect it.

Search is "the real heart of acquisition" (the owner). It is the first
slice whose validating consumers are two seats at once — the owner at
a terminal and an agent over MCP — and the first whose product is a
*model*, not a verb: the query, the vocabulary, the answer, and it will
form the basis for multiple client interfaces, both agentic and
traditional.

## Headline stances

Think deeply about what you want from the search system, and how to make its
use a natural extension of how you already work and think - and generalizable enough
that it can also be encoded into other client interfaces such as a gui or tui.

1. *"We need a system that is flexible, simple, and generalizable,
   not one tuned to a specific set of questions from a specific single
   user."* (2026-09-13) — the seat questions test a model's reach;
   they never specify it.
2. *The trade site sets the floor of reach and the ceiling of
   vocabulary. Every question it can ask of an item, a stash search can
   ask; composition is at least the site's (and, not, at-least-N-of) and
   extends across fields. Its stat keys and listing filters are not the
   target: the corpus's own lines are the vocabulary. A computed stat the
   app can derive from lines is in reach; one needing game data the API
   does not carry is stance 4's.*
3. *"There are use cases when a specific modifier and/or value is
   needed on an item that has several other specific characteristics."*
4. *Game knowledge the API does not carry is the user's input*, never the
   app's - e.g. what unique variants, modifiers, or values are legacy.
5. Powerfully simple means that both the agent and the human can be maximally
   expressive in search with minimal cognition. This comes from being idiomatic
   and building on what agents and humans already know how to do, not from elegant
   but complex or alien framework that requires time and attention to understand.
   *Semantics live in the model; idiom lives in the adapter (C46). Each
   consumer gets the shape it already knows, and all of them render one
   query. The test is the cold start: the calls a stranger makes from
   zero to a correct answer, counted.*
6. *Reach is a property of the derivation, not the language. What the
   projection carries, every language over it can ask; SQL is a second
   language on the surface, never a door around it, and a file opened
   outside the store is the only bypass. A question the model cannot
   ask in one query is a gap in the model, listed in the appendix, never
   routed to SQL.* The MCP carries the SQL tool as the CLI does (the
   owner, 2026-09-14: "for symmetry, so an agent using it isn't tempted
   to waste tokens figuring out how to use the CLI for something the
   mcp can't do").

There is tension inherent in the design process. That is ok. Sit with it
and consider how things balance and relate. For example, responsiveness is
critical for gui clients, but this may be at odds with design simplicity. Use
this tension as a motivating creative pressure.

Take time to understand the higher order impacts of ideas and decisions. Much
of the structure of this project came from stepping back to look for meta-patterns
as tools for cutting the Gordian knot.

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
| agent-seat | the agent test: twelve questions, R9–R19 (S190–S200); that SQL made each one call at the price of silent misses (F12); the query the agent wanted to type (F13) | the specification (stance 1); a projection's conventions — it read them only after three silent misses |

When a source speaks outside its lane, discount it.

## The settled floor (not up for debate here)

Naming what is settled is what makes it safe to be radical everywhere
else:

- The five invariants; the daemon owns GGG traffic and never reads
  facts (C2, C34); a frontend consumes two surfaces (C12); shared
  semantics live in Rust and every frontend has an adapter (C46).
- The facts file is internal and never a surface (C48); the
  annotations file is never opened by SQL (C35), and intent is read
  through a read-only view (the owner, 2026-09-14: "read-only
  annotations should be viewable. This kind of thing is present in the
  c++ app ('priced') and the trade site in more detail ('Trade
  Filters')."). The search reads a projection derived from facts (C34),
  persisted as SQLite so that SQL is a language over it (the owner,
  2026-09-14; stance 6), and
  kept in step with its facts by construction, never by a refresh: a
  derivation, not the cache C48 forbids. What the design answers is
  what it carries, who holds it, its contract, and how the model reads
  it (SQL or a scan: the bench says the scan wins eleven of twelve and
  SQLite is instant at the real corpus either way); the number is
  measured: 0.5 s to build and 38 ms to load at the real corpus, 5.7 s
  and 1.14 s at a million (engine-bench F5, F7; S151, S187, S188).
- Annotations are the only irreplaceable state (C35); a saved search,
  if one exists, is intent and lands there or in the user-scoped home
  the store has parked.
- Output has three levels — the decision view, the audit view, and JSON
  as the contract (C53) — and a search answer inherits them.
- Governed surfaces stay governed (C79): the trade site, RePoE, Awakened
  are read as the register says, never fetched by the app.
- The realm is a coordinate above league (the characters ruling); PoE2
  is an axis, not a fork.

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

The full tables: item-filter F2–F3 (S89–S91), repoe F3 (S69, S70), trade-query F5 (S55).

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
- **(d) Conflicts with a ruling** — a search cache written outside the
  store, with its own lifetime; the daemon reading facts; RePoE fetched
  at runtime; the app judging legacy.

## Convergence signals

Independent agreement is the strongest evidence in the pile:

- **Several ids for one line is the normal case**, not an edge: the
  site (380 collisions), the export (150 ambiguous templates), Awakened
  (twins as a two-stat group, several ids emitted as a `count` OR).
- **The text is the moving part**: 2,436 re-wordings under stable trade
  ids (prior-art), 229 renamed lines in one capture (repoe F1; S61), the
  string-to-object format change of July 2026 (item-facts F3; S3).
- **Identity finer than text**: owner R1 (S171), Awakened's ref plus matchers
  plus category, the site's 14 categories.
- **Composition across fields is missing everywhere** (owner F3): the
  site composes stats only; the C++ app ANDs.
- **Every answer wants its location** (owner R6, S176; the read the store
  lacks, store-as-built F2; S122, S123).
- **The load, not the query** (engine-bench F3, S150–S153; cpp-search F6, S37: the
  filter loop was never the cost).
- **Unknown is shown, never guessed** (prior-art F4, S107; the C++ dropdown's
  failure by contrast).

## Gravity warnings

- **C++ detail gravity.** 38 filters and 26 columns are the most
  detailed source and will dominate an unframed discussion. Extract
  the rules; discard the widgets.
- **Trade-site gravity.** The site is the floor of reach and the
  ceiling of vocabulary (stance 2): its 93 filters and 14,193 stat keys
  are not the target, and 27 filters and four whole categories mean
  nothing for a stash. Take its shape of asking; leave its contents.
- **RePoE gravity.** The stat spine is seductive and a third of lines
  are outside it; its access method is unruled. A design that needs it
  says so as a decision, not an assumption.
- **Engine gravity.** The instinct to index for speed is the C++ app's
  and the bench says no: the projection is for load and reach, not
  latency (F3, F4); an index is added with a number, not an instinct.
- **Edge-case gravity.** Twice-numbered stats, the 83 unreachable
  collisions, the veiled placeholder, `[Tag|Display]` markup: each is a
  limits-register row unless a `main` claim depends on it.
- **One user's questions.** Stance 1. The seven questions are a test.
- **Derivation gravity.** The columns are what the model needs, never
  the model's source. A grammar that is a WHERE clause in new spelling
  was designed from the table.
- **Change gravity.** Patches, leagues, endpoints and stat sets cannot
  be predicted, so nothing is built for them and no compatibility layer
  is carried. The search follows the corpus, not the game: a new or
  moved line is a line it has not seen, shown, and searchable by its
  template without a code change; when the API or the item's shape
  changes enough, the user refreshes, since facts are refetchable
  (C35). The dated captures are a diff for a human, never an input. A
  layer that exists to survive change is a concept the inventory must
  justify.

## The acceptance test, and simplicity

A proposal passes when every owner-seat question and every agent-seat
scenario is written out in the appendix, each as one query in its model
or as a listed gap, and every limit it inherits is named; a gap is never
closed by SQL. Simplicity is judged, not capped:
each proposal carries a concept inventory — every concept named, with
one line on why the model cannot do without it — and the reconciliation
compares inventories.

## The questions the design must answer (ranked)

1. What is a line's identity, and what does the search do with a line
   it cannot name, or whose text has moved since it last could?
2. What is the query model — predicates, composition, tri-state flags,
   values, the location coordinates — and what is its one grammar?
3. What is the vocabulary read, and how is it served to a human and an
   agent from the corpus?
4. Who holds the corpus, for how long, and what does a restart cost;
   is the derivation persisted, and what is its contract (the DDL, its
   version, its freshness)?
5. What crosses the trade boundary, in each direction, and what does
   not?
6. What does a result carry — the item, its location, its freshness,
   how much of the body, and which intent (priced, legacy) it joins
   read-only?
7. What are the non-goals and the limits, written as outputs?

## Output shape (stage 3, each proposal)

The model on one page; the grammar in as few lines as it takes; the
derivation the model needs (columns, views, its contract), after the
model and never before it; an answer to each question above; the
appendix (every acceptance question in the model's notation, each
marked one query or a listed gap, discovery reads and refinements
apart); the cold start (the calls a stranger makes to a correct answer
to owner Q1 and one agent scenario, the stranger being agent-seat's
phase-one condition: tool descriptions and help only, no row seen); the
concept inventory; what the design refuses; the decisions left to the
owner. The model and its grammar fit one page; that limit is the test.
The rest is as long as honesty takes — about 16 KB is a guide, never a
target — and the proposal closes by naming what it left out and the one
cut it would most want reversed. Two proposals, blind, then
reconciliation.
