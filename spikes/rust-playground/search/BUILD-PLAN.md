# Item search — the build plan (stage 6)

Status: draft for the owner's edit — 2026-09-19. Nothing is built until
he has edited it. It is a brief: deleted at the slice's close and cited
by hash, when the closed record (`SEARCH-SLICE.md`, in
`PRICING-SLICE.md`'s mold) takes what survived.

Authorities, not restated here: `decisions/search.md` (C89–C107),
`search/DESIGN.md` (the language reference and the contract detail),
`search/DIGEST.md` (the acceptance set, the limits register), note 23
(row 6, decision 15). Under decision 15 everything after the first seat
is provisional, this plan's later steps included.

## How a partial build stays honest

1. **The parser knows the whole language from step 1; the evaluator
   knows part of it.** What is not built lives in one closed list in the
   crate, each entry a construct of the reference and the step that
   builds it. A request that uses one is refused before anything is
   evaluated — `not built: pseudo.* (step 7)` — an error of its own kind,
   never the unknown-name error, never undecided, never an empty answer.
   `--describe` prints the same list. One test walks it: every construct
   of the reference either evaluates or refuses by that name, and the
   list the test walks is the list `--describe` prints.
2. **Scope, basis and undecided are in the first answer or there is no
   answer.** The first surface prints the scope block, the basis, the
   four counts per term and the undecided items with their reasons. A
   reason whose source is not built yet (a base the class table lacks, a
   price unresolved) cannot occur, because the construct that needs it
   is refused.
3. **Tests pin the crate's boundary** — a request in, an answer out, as
   JSON — and the CLI's `--json`; text is a function of JSON (C53).
   Every count in a fixture answer is worked by hand first, as the
   reference's worked example is.
4. **A rule the reference does not state stops the step** and comes to
   the owner if it changes what a user types; otherwise it is a row in
   the slice record's observations. A name the reference hands to the
   builder — a flag, a key, a field's spelling — is not a gap: the
   builder picks it and `--describe` or the help prints it. What a name
   *means* — which items it matches — is. Those found while planning
   are at the foot of this file.
5. **An answer never prints a command the same build refuses.** A
   block the reference bounds with a route to the whole (invariant 5)
   prints that route only when it runs; until then it prints the count
   left out and names the unbuilt construct. The route property's test
   covers it: every command an answer prints parses and is not refused.
6. **The reference stays whole in `search/DESIGN.md` until the first
   seat has ruled.** Module docs cite it by section and take no
   paragraph before then: the seat reads that page and may reopen it,
   and two copies would rot. The header there says a paragraph leaves
   when a module doc carries it; this delays the carrying, not the rule.
   Owner, 2026-09-19: "accepted".
7. **Whose store.** The agent tests on fixtures built through the
   store's own ingest, and measures on a sqlite `.backup` copy under a
   track's `raw/`, always with `ACQ_STORE_DIR` set. The owner's store is
   opened only by the owner, at a terminal. No GGG traffic anywhere in
   this plan; the one daemon is M5's, a mock session.

## Steps to the first seat

| Step | Builds | Closes on |
| --- | --- | --- |
| 1 · the language | the `acquisition-search` crate (C89's edges in `tools/docs-check.sh` §5, each with a breaker; C47's lints); the tree and its JSON form; the parser over the whole query language; lowering; the canonical printer; every error the grammar defines, with its readings | the round-trip test; the corpus table; the holes table |
| 2 · the derivation | a pure function, body and ingest columns in, the item out (C103): fields, displayed strings, lines as (kind, template, numbers) with slots and the ranged rule, and what could not be read, by collection | fixture tests; M2 against the census |
| 3 · the store's read | C103's read, the revision, the coverage rows — **shown to the owner first** (below) | store tests, `REFRESH-SLICE.md`'s findings table as the checklist; the one-snapshot test; M1; C103's second decision given its own entry (below) |
| 4 · the first surface | the binder (names, near names, closed sets); three-valued evaluation with witnesses (C93); occurrence binding, the sort scalar, the together count (C92); the answer (C100): query, scope, basis, terms, total, rows, the zero-total block, a route per term count; `--describe`; `--json`; the verbs `acq search` and `acq show <id>` (the item as the deriver sees it: fields, place, every line with its kind, template and numbers, what was unread; `--json` the same, structured; the stored body on request by a flag, which still works when deriving fails — the reference's `acq show`), their README tour lines, `CLI-REFERENCE.md` regenerated | the worked example reduced to what is built, hand-counted; the acceptance rows marked 4; the refusal walk; the route property; M3, M4 |
| 5 · counts and the vocabulary | `--count`, `--cross`, `--sum` (C95); `none` and `undecided` buckets with routes (C105); `--count line[:text,…]` (C97) | AQ1; C105's two invariants pinned on a one-value key; C95's sum over a fixture of three kinds — an item with the value, one lacking it (adds nothing, counted as lacking), one whose value is unread (the subtotal marked incomplete, never a total); the vocabulary's pasted term selects its row; M3 again; then the seat |

**The reference was brought level with what was ruled here before step
1** (2026-09-19, the owner approving the wording), since step 1's corpus
is complete against that page and this file is deleted at the close. A
gap ruled below keeps its evidence here and its rule in
`search/DESIGN.md`.

**Step 1's evidence, in full**, since it is the step that runs first:

- *Round trip.* Over a generated population of trees (proptest, already
  a dev-dependency of the plan crate) and over the corpus: parsing the
  printed text of any tree gives that tree; printing a parsed canonical
  text gives that text; the JSON form survives the same trip; shorthand
  lowers once (`"T">=90` prints as `line("T" arg1>=90)`).
- *A slotless comparison is checked before it lowers* (C92). A quoted
  template states its own numbers, so the check is the parser's and
  needs no corpus: one `#` lowers to `arg1`; several is the error that
  lists them (`"Adds # to # Cold Damage">=20` offers `low`, `high`,
  `avg`, `arg1`, `arg2`); none is an error; `sum("T")` and `--sort`
  obey the same. Each is a corpus case.
- *Invariant 1* as a test: the printer never reorders, flattens, merges,
  deduplicates or simplifies — `(a b) c` and `a b c` stay two trees.
- *The corpus* — one committed file of cases, each a text with its
  canonical form or its error: every construct of the reference at least
  once, every error the reference names with the readings it shows, and
  the acceptance queries below. The table printed at the close maps
  reference construct → case, so a construct with no case is visible.
- *The holes table*: what the parser met that the reference does not
  state, each marked *changes what a user types* (the owner's) or not
  (an observation).
- Names are not step 1's: the parser accepts `field op value` shapes and
  the binder (step 4) knows the fields, so `mod(` stays an ordinary
  unknown name as the reference says.

**Steps 2–4, the evidence that is not obvious:**

- *M2 is a differential, not a unit test.* The committed census
  (`item-facts/data/mod-templates.csv`, 6,928 rows) was taken over three
  inputs, so the census script is first re-run over one — the spike copy
  of 2026-09-13 still in `item-facts/raw/` — and the Rust deriver over
  that same copy must reproduce its templates, counts and ranged split,
  or each difference is explained. It runs locally, never in the gate
  (the input is `raw/`).
- *The one-snapshot test*, in process, no daemon: a second handle
  records a fetch while a read is part-way through its items; the read's
  revision, coverage rows and bodies all describe the state before it,
  and the next read carries a higher revision and the change. Step 4
  repeats it at the answer's boundary. A held corpus's invalidation is
  tested when a consumer first holds one; the intent half arrives with
  price (step 9).
- *The route property*: every route the answer prints, run as the
  request it is, returns exactly the members it counted.
- *Invariants 2, 3 and 6* as tests: a `:` or `~` selector prints back as
  authored with its binding beside it; every corpus query is valid over
  an empty store; a dump of every table is the same before and after a
  search.
- *The limits register* (C102): S12, S52, S53, S107 each have a fixture
  that meets the limit and a test on the wording printed.

## The first seat — after step 5

After 5, not 4 (owner, 2026-09-19: "I'm ok delaying the seat for a good
reason, especially if it's related to discovering the design and
implementation"): the vocabulary is how a template is found without
knowing it, it is the route to everything a `:` or `~` selector resolved
to, and with `--sum` the seat exercises all six of the lines it revisits
first.

The owner, at a terminal, on his real store: `cargo build --workspace`,
then `acq search`. It reads the store directly, as `acq tabs` does — no
daemon is started or spoken to. If his store holds more than one realm
every search names `--realm` (C96); tiring of that is the sticky-realm
park's trigger, and it is his to fire.

What he can ask:

```
acq search --realm pc '"# to maximum Life">=90' --sort 'line("# to maximum Life").arg1' --desc --limit 10
acq search --realm pc 'rarity=rare base:ring line(template:resistance is:fractured)'
acq search --realm pc 'name="Ashes of the Stars"'
acq search --realm pc 'rarity=rare base:ring "+#% to Cold Resistance" -"+#% to Chaos Resistance"'
acq search --realm pc 'line(template:explode) -is:corrupted tab:dump'
acq search --realm pc 'line("Adds # to # Cold Damage" low>=15 high<=45)'
acq search --realm pc '(rarity=rare base:ring) sum("# to maximum Life")>=90'
acq search --realm pc 'undecided("# to maximum Life">=90)'
acq search --realm all 'id:<a handle an answer printed>'
acq show <an id an answer printed>
acq search --realm pc 'rarity=rare base:ring' --count line:resist,life
acq search --realm pc --count tab,league,rarity
acq search --realm pc 'rarity=unique' --cross league,tab
acq search --realm pc 'frame=currency' --count base --sum stack
acq search --describe
```

Composition whole (`and or not - ( ) holds undecided`); phrases and
`text:`; `name` and `base` with `:` `=` `~`; `rarity`, `frame`, `ilvl`, `is:`,
`has:`; place (`league: tab: character: container:`); `id:`; `line(…)`
whole, the shorthand, `sum(…)`; `--sort --desc --limit --json`;
`--count`, `--cross`, `--sum`, and the vocabulary.

A row shows the lines the query touched (C100), so an item's other
lines are read through `acq show <id>`, or by naming them in the query;
`--fields` comes later. What is bounded with no route yet, each printed
as a count and the unbuilt construct's name (rule 5): rows
past `--limit` (a larger `--limit` serves; `--next` is step 10); the
coverage list (`acq tabs` and `acq store characters` exist today;
`--view locations` is step 10).

What it refuses by name: `class:` (6), `pseudo.*` (7), `sockets` `links`
`linked(…)` (8), `has:priced` and `price.*` (9), `--fields`, `--next`, `--explain`,
`--context`, `--view locations`, `--print-request` `--request`
`--rebind`, `show --against` (10).

It exercises all six of the lines the seat revisits first
(`decisions/search.md`, "Standing"): C91's ambiguity error, C92's binder
and sort scalar, C93's undecided route, C95's sum rule, C98's basis as
printed, C104's text.

## After the seat — provisional, his to reorder

| Step | Builds | Needs | Closes on |
| --- | --- | --- | --- |
| 6 · class | the class table as reference data, its source chosen under C106's admission test (`item-facts/data/class-evidence.csv` is where the read starts); `class:`; the reason *base not in the class table* | 4 | OQ1's slot, OQ5; C105's test as worded (ten rare items by class) |
| 7 · computed values | the totals table (C94) after the coverage trial; `pseudo.total_res`; a weight may be a fraction (the site's own `+94.5 total maximum Life` over `+90` life and `+9` Strength, `pseudo-stats/README.md`), and whether a total is ever rounded is shown to the owner first (gap 6); the sum-status table; then `pseudo.dps`, `pseudo.pdps` (C101) | 6 for the worked example | AQ2; the reference's worked example whole, every count as printed there |
| 8 · sockets | `sockets`, `links`, `sockets.<colour>`, `linked(…)` (C101); undecoded shapes counted unread (S16) | 4 | OQ3's socket reading |
| 9 · price | the effective price joined read-only (C81, C100); the crate links `acquisition-plan`; the basis gains the intent revision; the reason *price unresolved* | 4 | OQ6: the item found with the owner's own price; a valuation asked for is a stated limit (C102) |
| 10 · continuing and exchanging | `--next` refused across a changed basis; `--print-request`, `--request`, `--rebind`; `show <id> --against`; `--explain`; `--context corpus`; `--view locations`; `--fields` | 4 | AQ3 whole; M5 |
| 11 · the close | the MCP tool over the same request, with an agent's seat as its own consumer (P2); `acq items search` retired (below); the closed record; this file deleted; `search/` shrunk; `DESIGN.md`'s paragraphs into module docs | all | AQ1–AQ5 driven through the MCP; the gate; the kill-list recheck |

Not in this plan: the trade translation (C99, headed "Direction"). No
acceptance question needs it; whether it is a step here or the next
slice is the owner's call, best made after a seat.

## The acceptance set as tests

One test per question over a fixture store, through the crate's
boundary, plus one CLI `--json` smoke each. The queries enter step 1's
corpus, so that every question can be written in the language is
evidence at step 1. Names are illustrative, as in the reference.

| Id | The test asks | Green at |
| --- | --- | --- |
| OQ1 | `class:ring rarity=rare (line(template:resistance) or line(template:strength))`: the query names the lines it wants to see, so the rows show them; refined: `line("T" is:fractured)`. The fixture holds a ring with neither line, which must not appear | with `base:ring` 4 · as worded 6 |
| OQ2 | `name="<a unique>"`: one item or a zero answer naming the scope, and its place; then `acq show <id>` for every line with its values, which is what the owner reads the version from; nothing says legacy (C107) | 4 — and it fires the variant park |
| OQ3 | a mod, a base, a unique, socket colours, each its own query | 4 · sockets 8 |
| OQ4 | a staff by what is remembered of it (`class:`, `base:`, a phrase), item and tab; `--describe league` says place, never origin (S177) | `base:` 4 · `class:` 6 |
| OQ5 | a level bracket across tabs, counted by tab: `(class:… or class:…) (reqlevel=..30 or -has:reqlevel)`. The fixture holds distractors the bracket alone admits — a low-level gem, a flask, a currency stack, which has no level requirement at all — and none may appear | askable 6, counts 5; **not covered** until the owner has ruled on the category park it fires |
| OQ6 | the item found, `has:priced`, the price as the owner set it; no valuation | 9 |
| OQ7 | one line across every tab and character, item and tab on each row; `--count tab` | rows 4 · count 5 |
| AQ1 | `--count tab,league,rarity`: three tables, no rows | 5 |
| AQ2 | `(<the OQ1 query>) pseudo.total_res>=60`: the old query parenthesised and one new term. The fixture holds an OQ1 match under 60, and the test fails unless it leaves | mechanism 4 (the new term a `sum`) · as worded 7 |
| AQ3 | each row names what matched; a zero answer names the scope searched; `show --against` | 4 · why-not 10 |
| AQ4 | `id:<handle>`, every printed id accepted back (S198) | 4 |
| AQ5 | `"# to maximum Life">=90` sorted: a count and a sorted list across every mod array | 4 |

## What happens to `acq items search`

Nothing, until step 11. It stays as built while the design can still
cycle — the old verb is also a control the owner can run beside the new
one. Step 11 retires, in one commit: the verb (`acq items show` is
already answered by `acq show <id>` with its raw flag, there since step
4, so no inspection is lost), the MCP's substring tool, `Store::search`, and the
`items_names` index it never used (S124, S136) — the README tour line,
`CLI-REFERENCE.md` and `MCP-REFERENCE.md` regenerated. Retiring it needs
the new verb to answer what `--removed` answers, which is gap 3 below.

## Parks whose triggers the build fires

| Park | Fired by | Then |
| --- | --- | --- |
| the totals coverage trial (`decisions/search.md`) | step 7, before the first recipe is accepted | one script over the census; the result decides the recipe's rows |
| a grouping above class | step 6 reaching OQ5 | brought to the owner as an acceptance task before OQ5 counts as covered |
| a unique's variant as a field (C107) | step 4 reaching OQ2 | the trigger reopens the question, not the build: brought to the owner with his Ashes of the Stars as the first test |
| the digest's kill list | step 11, the last acceptance test written | recheck against the tests and the reference before any removal |
| a persisted projection; the store's search-at-scale park | M3, only if a CLI ask exceeds 500 ms | reported with the numbers; it opens the experiment among the candidates, never persistence by default. The seat goes ahead: slow is not wrong |
| pricing on the MCP (`decisions/frontends.md`) | step 11, the read model landed | the owner's call whether it is built then |
| the sticky default realm | the owner's seat, not a step | his to fire |

Every other park's trigger is a recorded question or an ask, which a
seat may produce and a step cannot.

## Measurements the build owes

All on a `.backup` copy, recorded in the slice record with the command
that produced them.

| # | What | When |
| --- | --- | --- |
| M1 | the streaming body read: wall time and peak memory over the whole corpus | step 3 |
| M2 | the deriver against the census: templates, counts, the ranged split, unread shapes | step 2 |
| M3 | a CLI ask, process start to exit, `--json` to `/dev/null`, over the backup copy (its item count recorded), for every acceptance query built so far. Warm: the median of ten consecutive asks. First: the first ask after the copy is written, reported as seen — this machine's file cache is not controlled, so it is never the budget's number. Both profiles: the release build is judged against 500 ms, the debug build is what the seat feels | steps 4 and 5, before the seat; again at 7 and 9 as their queries land |
| M4 | `~` over all displayed text (the reference: "its cost … is unmeasured") | step 4 |
| M5 | a re-derive while a refresh is writing — a mock session over a fixture store | step 10 |
| M6 | the totals coverage trial | step 7 |

Not owed by this build: whether job events reach a client that did not
submit the job (C98) — a resident client's question, the GUI's.

## The store's read (C103) — the first showing

Shown again at the top of step 3, with real signatures and the
verification below done, before any of it is built. One public read
beside the snapshot reads, in one read transaction:

```
header   revision · the account's identity · the realms the store holds
         every live location in scope — realm, league, kind, id, parent,
         name, listed_at, fetched_at (none = never fetched)
items    streamed, one at a time, in a stable order:
         id · realm · league · location kind and id · container ·
         socketed_in · first_seen · last_seen · removed_at · the body as text
scope    one realm or all · live, or live and removed
```

- League is joined as `read_items` joins it (S131): a character's items
  take the character's current listing league, and a league-less
  character's items are carried with no league — `none` in a count.
- The body is handed over as text, unparsed. `Store::search` today turns
  a body that does not parse into a silent null; here the search crate
  parses, so that failure is an undecided reason on that item.
- The deriver takes the body and a plain struct of those columns that
  the search crate owns, so it names no store type (C103).
- **The revision — ruled 2026-09-19: the highest `responses.id`.** No
  schema change, facts stay v7, and sitting at the seat migrates nothing
  in the owner's store; a stamped counter (facts v8, a migration on
  first open) stays the fallback. Owner: "I agree with (a)". Read before
  he ruled: `record` is the one path that changes items, locations or
  membership and it inserts its response row in the same transaction; a
  refused body rolls back whole; `acq store import` goes through
  `record`; `rebuild` rewrites only the derived columns, which this read
  does not hand over; no code deletes a response. A schema migration
  moves rows with no response, so the basis prints the facts version
  beside the revision. Step 3 repeats the read line by line, and a path
  that fails it comes back to the owner before anything is built.
- **At this step's close C103 was split** (owner, 2026-09-19: "Yes, note
  it please"): what the store gains is C108, in `decisions/store.md`,
  pinned by the store's tests; C103 keeps the deriver and a pointer.
- **Which number is the revision is mechanism, not a registry line.**
  Its property is already C98's and `search/DESIGN.md`'s (the basis);
  the ruling above and its line-by-line verification are carried into
  the store's module doc under the decision id when the read is built.
- Observation, no proposal: `Store::open` takes a write lock for a
  moment on every open, as every store verb does today. Search inherits
  it.

## The record

What was built, and what it met. At the close this is the closed
record's step ledger and its observations.

| Step | Commit | What landed |
| --- | --- | --- |
| 1 · the language | `080a8581` | `acquisition-search`: the tree and its validity (`tree.rs`), the parser over the whole reference (`parse.rs`), the canonical printer (`print.rs`), the strict JSON form (`json.rs`), structured errors with stable kinds (`error.rs`). `tests/language.toml`: 133 cases over 86 constructs (123 over 82 at the step's commit; H1 and H2 added the rest) — every construct of the reference, every error the grammar defines with its readings, the acceptance queries — and `tests/language.rs` refuses a construct with no case. The round trip over generated trees (2,000 a run; 60,000 once, by hand), any finite number, and no text panics the parser (200,000 once). C89's edges in `tools/docs-check.sh` §5 with nine breaker cases: `tools/docs-check-breakers.sh`, 51 ok. The contract detail's sentences on the check and the lints left `DESIGN.md` for the crate doc. |
| 2 · the derivation | `ef720323` | `derive.rs`: `derive(facts, body) -> Item`, pure and total — the header, `rarity` and `frameTypeId` as given, `ilvl`, `stackSize`, the note, every yes the body says; properties and requirements as displayed strings, name and values kept apart; lines as (source, flags, template, numbers) with `slot` and the ranged rule; `displayed()`, the rows a phrase will be tested against; and what could not be read, by part, the readable rest still derived. `tests/derive.rs`: 12 fixtures worked by hand and a property test that no text and no JSON panics it. M2 below: no unexplained difference. `template::typed` gained the thousands comma. |
| 3 · the store's read | `c0a8f918` | `acquisition-store/src/corpus.rs`: `Store::read_corpus` — the header (the revision, the account, the realms the file holds, the latest listing of each list in scope, every live location in scope with its `type`, `listed_at` and `fetched_at`) and then every live item at a live location, streamed to the caller's closure inside one read transaction, league joined as `read_items` joins it, the body as text — and `Store::revision`. Seven store tests, `REFRESH-SLICE.md`'s findings as the checklist: the one-snapshot test over two handles on one file (it fails when the snapshot is ended after the header — tried), the league join over a character the listing moved and a league-less one, live by full coordinate (one tab id under `pc` and `xbox`) in one realm or all, the revision through every write door, no derived column in the read or its order, a store with no account refused, and the header's coverage — a folder told from an unfetched tab, a retired tab's substash no live location (it fails without the exclusion — tried), an empty listing still seen. The revision's line-by-line read is that module's doc. C103's second decision is C108 (`decisions/store.md`), its wording approved by the owner (2026-09-20: "All is approved"). M1 below. |

**Found by step 1's own tests, in the builder's code:** a computed value
named alone (`pseudo.total_res`) was answered as a bare word; a whole
number between 9.0e18 and 2^63 printed as digits the parser read back as
another tree (`Number::from_f64` now agrees with the parser at the edge
of an i64, pinned); `ilvl >= 84`, `tab:q-20` and a template typed with
two numbers each got a misleading error and now get their own.

### The holes table — what the parser met that the reference does not state

**Changes what a user types — the owner's.** None blocked step 1: each
is built in the direction that breaks nothing if he rules the other way.

| # | Hole | Built as | Recommendation |
| --- | --- | --- | --- |
| H1 | A phrase that is a template: `"# to maximum Life"` alone was a text search for a literal `#`. Over the census's store copy (22,721 items) no displayed string — name, type line, base, mod line, property — contains one, so it could never match | **ruled 2026-09-19: a template alone means the line** (the reference, *Item-level*); `text:"…#…"` keeps the literal search | — |
| H2 | A sign before `#` in a quoted template (`"+# to maximum Life"`, as the game, the C++ tables and the trade site write it): the sign is carried in the number (C90), and 0 of the census's 6,928 templates start with `+#` | **ruled 2026-09-19: `+#` is spelling and is dropped; `-#` is an error offering `argN<0`** (the reference, *Strings*) | — |
| H3 | Whitespace around an operator: `ilvl >= 84` | an error that says an operator sits against its name and value, since whitespace separates terms | keep |
| H4 | A quoted value after `:` — `template:"maximum life"`, `tab:"$ dump 1"`; the reference shows only a bare word there | accepted; a value that is more than one plain word must be quoted, and the error offers it quoted | keep |
| H5 | `AND`, `Or`, `NOT` | accepted in any case, printed as whitespace, `or`, `-` | keep |

H3, H4 and H5 stand as built (owner, 2026-09-19: "Otherwise I accept
your recommendations"). H1 and H2 were ruled the same day, his words in
the reference, and built in the commit after step 1's: the trade site
writes `+# to maximum Life`, and people and agents will type what it
writes. Measured over its stat texts (`trade-query/data/stats-2026-09-12.json`,
13,707 outside `pseudo`): 1,048 carry `+#` and none `-#`; typed as the
site writes them 0 name a census template, with the sign dropped 359 do
(the rest are lines this corpus does not hold); only 3 sign-free forms
are shared by two site texts (block chance, with and without `+`). `#%`
needs nothing: the `%` is part of a template here as there.

**Observations — the builder's.**

- Parentheses are spelling for nesting and nothing else: `(a) b` is
  `a b`, and a JSON tree holding a group of one is refused ("a group of
  one is its member"); only the root may be empty, the empty query.
- Canonical spellings the reference leaves open: and is whitespace, not
  is `-`; `text:x` prints as the phrase `"x"`; a pattern is always
  quoted; `sum("T")` prints lowered, `sum(line("T").arg1)`; a `holds`
  bound is `>=n`, `<=n`, `=n` or `=a..b` (`>1` is `>=2`); `90.0` is `90`.
- A quoted template means `template=` anywhere in a line's group, not
  only leading, and prints as the bare `"T"` where it sits (never
  reordered). Two in one and-group is valid and matches nothing.
- The slot checks run when a group selects exactly one quoted template
  at its own level; a template inside an or, under a not, or selected by
  `:` or `~` states no numbers, and its slots are the evaluator's.
- Not checked here: that a `~` pattern compiles (no regex dependency
  yet — the binder's, step 4); that a field, flag, class or computed
  value exists (the binder's). A bare word's readings are the two the
  grammar knows, `"word"` and `line(template:word)`; the closed-set
  readings (`rarity=rare`) arrive with the binder's vocabulary.
- The JSON keys beyond the worked example's: `holds` with `min`/`max`,
  `undecided` with `thing` or `term`, `const`, `has`, `is`, a range as
  `{from, to}`, `sum` and a projection as `{lines, slot}`. Reading is
  strict: an unknown or missing key is an error naming its path.
- The wire reads an extreme float (1e122) to within one unit in the
  last place (serde_json without `float_roundtrip`); the text is exact
  for every finite number, and no typed bound is near that.

### Step 2 — M2, and what the deriver met

**M2** — `python3 search/item-facts/scripts/m2-differential.py`, after
`cargo build --workspace`; input the live items of
`item-facts/raw/spike-GERWARIC_7694-2026-09-13.db` (22,721), 2026-09-19.
The script re-runs the census's own template rule over that one input,
hands the same rows to the deriver
(`crates/acquisition-search/examples/derive-census.rs`), applies each
departure below to the census's side with its own code, and compares
every (array, template) row: items, lines, flags, how many numbers, and
whether the ranged rule takes it (`ranged-split.py`'s cases A and B).

| | Census rule | Deriver |
| --- | ---: | ---: |
| templates / lines, all arrays | 6,476 / 88,128 | 6,606 / 86,450 |
| `explicitMods` | 5,304 / 72,868 | 5,278 / 72,506 |
| `implicitMods` | 446 / 6,050 | 440 / 6,050 |
| `utilityMods` | 26 / 2,911 | 23 / 2,911 |
| `ultimatumMods` | 34 / 3,567 | none: not lines |
| `hybrid.explicitMods` (D3) | never read | 199 / 2,251 |
| enchant, crucible, scourge, veiled, bonded, rune | 666 / 2,732 | the same |
| unexplained differences | | **0** |
| items with anything unread | | **0** of 22,721 |
| ranged (one `# to #`): rows / lines | | 201 / 3,708 — 186 the pair alone, 15 with a further number; 2 rows with two pairs, 387 with several numbers and no pair, 166 `(#-#)`, none of them ranged |

The departures, each counted by the script (lines whose template moved /
census templates touched):

| Departure | Count | Why |
| --- | ---: | --- |
| `ultimatumMods` is no source of lines | 3,567 lines | its elements are ids with a tier (`FrostInfection`, 3); the same item's `explicitMods` already displays them (`Blistering Cold III`) on all 539 items, with two lines more on 315. A candidate ground-truth claim |
| a vaal gem's base skill is the source `hybrid` | 2,251 lines added | D3, ruled; the census read top-level arrays only |
| an empty line displays nothing | 362 lines | the spacer rows of an essence's description |
| a row break `\r\n` is `\n` | 407 / 256 | the reference's strings escape `\n` alone, so a template holding a CR could never be typed |
| `<style>{Display}` reduced, nested | 478 / 359 | markup the digest does not name (S4 knows the brackets): a divination card's reward, `<uniqueitem>{Staff}`, `<size:31>{…}`; 821 tags, each followed by its brace. A candidate ground-truth claim |
| `[Tag|Display]` and `[Display]` reduced | 286 / 62 | S4, C90 |
| `1,500` is one number | 1 / 1 | read as 1 and 500 it is a wrong value, silently; the template is `#x Vivid Crystallised Lifeforce` |

The differential caught two misreadings in the builder's code before any
fixture did: those 362 empty lines became lines with an empty template,
and a heist trinket's `Any Heist member can equip this item.` —
`displayMode` 3, one empty value, no `{0}` — was counted unread on 135
items. Deriving took 1.4 s for the 22,721 in a debug build, the parse
included; M1 is step 3's.

**Holes — they change which items a query matches, so they are the
owner's.** None blocked step 2: the deriver reads and does not interpret
(`derive.rs`, "As built"). D1–D3 were ruled the day the step landed
(owner, 2026-09-19; his words in the reference), D4 and D5 after his two
questions on them; D2, D3 and D5 were built in the commits after the
step's; D1 needed no code, the deriver carrying both, and D4 none.

| # | Hole | Built as | Recommendation |
| --- | --- | --- | --- |
| D1 | `rarity` on an item whose body carries none: 8,297 of 22,721 — gems 5,747, currency 1,720, cards 490, normal-frame 339, a quest item. `frameTypeId` is on every item and differs from `rarity` on 7 (foils; one currency item marked normal) | **ruled: two fields, `rarity` and `frame`, each what GGG gives** (the reference, *Item-level*) — the builder's fallback from one to the other was not taken, and this plan's seat line became `frame=currency`. `--count rarity` puts those 8,297 under `none` (C105); `--count frame` has no `none` | — |
| D2 | `ilvl` is 0 on 7,903 items (every gem and card, most currency): the game shows them no item level | **ruled: 0 is absent** (the reference, *Item-level*) | — |
| D3 | A vaal gem's base skill sits under `hybrid` — 2,251 lines and its properties on 457 items — and is displayed on the item | **ruled: its lines are the source `hybrid`, its properties displayed strings** (the reference, *Members*) | — |
| D4 | The other property-shaped arrays: `nextLevelRequirements` (265 items), `supportGemRequirements` (19), `weaponRequirements` (6), and `gemTabs`, `grantedSkills` on poe2 (42, 4); `notableProperties` never seen | **ruled: not read until a question needs one** ("I agree with you on D4"); `nextLevelRequirements` as text would make `"Level 21"` find a level-20 gem | — |
| D5 | A requirement as a displayed string. The builder had made each its own string from memory of the tooltip; the trade site (the owner) and the C++ app (`src/ui/itemtooltiptext.cpp`) both show one row | **ruled: one displayed row, `Requires Level 67, 159 Str`, each requirement kept on its own beneath it** (the reference, *Item-level*), GGG's `displayMode` ordering each | — |

**Observations — the builder's.**

- S12 as built: a veiled line keeps the census's template (`Suffix#`)
  and carries no number, so it names no slot and a value query never
  matches it; its text shows the placeholder as given.
- A line names a slot only while its numbers are its template's `#`s, so
  a displayed literal `#` (none in this corpus) costs the line its slots
  and nothing else.
- The source word is the array's key without `Mods`, whatever GGG adds
  (`enchant`, `utility`, `crucible`, `scourge`, `veiled`, `bonded`,
  `rune` seen); a flag is any key the line's `flags` sets true, and an
  item's `is:` words are every top-level true and the true keys of
  `influences`, as GGG spells them (`abyssJewel`, `duplicated`,
  `isRelic`). The language's spellings of all three are the binder's.
- Lines are ordered by source word, then as the body orders them, so the
  order never depends on how a JSON map was held.
- Nothing was unread on this corpus, so C93's unread path is exercised
  by fixtures alone: a wrong type under a known key, an element that is
  no line, a `displayMode` outside 0–4, a body that is not an object.
- Sockets are step 8's and are not derived yet.

### Step 3 — M1, and what the read met

**M1** — `/usr/bin/time -l target/<profile>/examples/read-corpus
search/item-facts/raw/spike-GERWARIC_7694-2026-09-13.db <mode>`
(`crates/acquisition-store/examples/read-corpus.rs`), 2026-09-20: 22,721
items, 29.9 MB of bodies, 3,509 live locations, 4 listings, revision
1126 at facts v7; the file cache warm, the median of three runs, which
lie within 9 ms of each other.

| Mode | Release | Debug | Peak resident |
| --- | ---: | ---: | ---: |
| `stream`: each item dropped once counted | 36 ms | 94 ms | 8.7 MB |
| `hold`: every item kept as text to the end | 37 ms | 96 ms | 51.7 MB |
| `parse`: each body parsed as JSON, then dropped | 113 ms | 829 ms | 9.3 MB |

The header is 6 ms of each release read (16 ms debug). It was 13 ms
while the realms were read from `items` as well as from `tabs` and
`characters`: a scan of the largest table for a value its locations
already give — 4 ms without it, and 6 with the listings and the orphan
exclusion the audit below added. With M2's 1.4 s to derive in a
debug build, an ask at the seat's debug build is about 2 s before
anything is evaluated; the release build is what 500 ms is judged
against (M3).

**Departures from the first showing, each the builder's.** No membership
and no `removed_at`: removed items are not read until `all` is built
(gap 3), and a read that handed them over would change under a prune
with no response written (owner, 2026-09-20: retention starts as a verb,
"which means we can think about triggers for that verb later rather than
now"). The order is location kind and id, then realm, then item id:
realm first sorted the whole corpus, bodies included (43 MB resident
against 10 MB, 0.13 s against 0.04 s in the `sqlite3` shell). The realms
are those the file has a tab or a character under (owner, 2026-09-20: `pc`, `xbox`,
`sony` and `poe2` are "the full list").

**An outside audit of the build (2026-09-20), each finding verified
before it was taken.** All three are the header failing the reference's
scope line — `1 never fetched · location list seen 2h ago`.

| # | Finding | Verdict | Held by |
| --- | --- | --- | --- |
| 1 | A folder is a live row no fetch ever fills, and the header could not tell it from a tab never fetched (17 on this copy) | confirmed | `LocationRow::tab_type`, GGG's `type` verbatim as the snapshots carry it; the fixture's folder with a fetched child |
| 2 | A substash whose tab a listing retired keeps its row (the planner's orphan report) and was listed as a live, never-fetched location. The builder had seen it and followed `read_tabs` without saying so | confirmed | excluded: a location is live with its parent (C54); the fixture, which fails without the exclusion |
| 3 | A location's `listed_at` cannot say when a list was seen: an empty list has no rows, and a substash's is its parent's fetch | confirmed | `CorpusHeader::listings`, the bases the snapshots cite, read in the same transaction; the fixture's empty character listing |

**Observations — the builder's.**

- On this copy no character item's stamped league differs from its
  character's listing league, and no character is league-less: the
  league join is exercised by fixtures alone.
- 6 of 22,727 item rows are removed, all at live characters; no tab or
  character has been retired yet, so an item removed at a retired
  location exists in fixtures alone.
- Over its 12.8 days the file's logs weigh: `responses` 1,126 rows and
  2.5 MB, `item_events` 22,751 rows (22,727 the first `added`) and
  2.8 MB, `refused` 4 bodies and 135 KB, removed items 6.7 KB. Nothing
  reads a response again but the latest listing per realm and league,
  the v4 migration's re-stamp and now the highest id.
- 2,926 of 3,427 live tabs have never been fetched, the 17 folders
  apart, so "absent from a full refresh" is not a condition this
  account's store can often state.
- `Folder` is now a word four consumers know — the planner, pricing, the
  CLI's counts and, from step 4, search — each comparing GGG's `type`
  for itself. The read followed that precedent; the store saying it once
  is a small change if a fifth arrives or one of them gets it wrong.
- The header's listings are read by a scan of `responses`, which has no
  index on `endpoint`: nothing at 1,126 rows, unmeasured beyond, and it
  grows with every request. It is one of the costs the retention park's
  size trigger would see first (`decisions/store.md`, "Parked").

## Gaps found while planning — rules the reference did not state

Each changes what a user types, or which items what he types matches.
4, 5 and 6 came from an outside review of this plan (2026-09-19). Ruled
ones are in `search/DESIGN.md`'s reference; what stays here is the
evidence.

1. **A line break inside a template — ruled; the reference, *Strings*.**
   540 of the census's 6,928 templates are one mod displayed over
   several rows. M6's coverage trial counts such templates too.
2. **An apostrophe at a shell — open; does not block step 1 unless
   (c).** 306 templates carry one, and so does the reference's own
   `name="Kaom's Heart"`, which cannot be typed plainly inside the
   shell's single quotes. (a) nothing: the shell's `'\''`; (b) the
   adapter reads the query from stdin or a file; (c) the language also
   accepts `'…'` strings, printing `"…"`. *Recommendation:* (b) at step
   4, and (c) left for the seat — adding it later breaks nothing.
3. **The values of `membership` — open; does not block before step 4.**
   The request carries it, the terminal synopsis has no word for it,
   and `live` is the only value shown; `acq items search --removed`
   exists today. *Recommendation:* `live` alone at the first seat; `all`
   (live and removed, each row marked) at step 11, so the old verb can
   retire; `removed` alone waits for someone asking.
4. **A node forced true or false — ruled; the reference, *Composition*.**
5. **The header: `name`, `typeline`, `base` — ruled; the reference,
   *Item-level*.** Over the census's store copy (22,721 live items):
   every magic (2,423), normal (3,139), gem and currency item has no
   name, nor do 372 of 5,785 rares and 86 of 3,077 uniques; the type
   line differs from the base on 2,399 magic items (the affix names),
   234 normal (`Superior …`), 172 unique and 97 rare (`Synthesised …`).
   The draft's "its own name, else its type line" was withdrawn on
   those numbers.
6. **The totals example — ruled; the contract detail, C94, C95.** The
   sum's example now cites the C++ app's table and no longer names the
   site's unresolved count (`# total Resistances`,
   `pseudo-stats/data/pseudo-classes.csv`). Whether a fractional total
   is ever rounded is step 7's to show the owner: the one capture
   printed `+94.5`.

Observation, mine unless the owner wants it: C105's first test is
worded with groupings above class (`armour`, `weapon`), which are the
parked category — it will be pinned with class names and the same
numbers.
