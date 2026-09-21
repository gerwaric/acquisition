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
| 4b · the properties | what four audits of step 4 found by generating, put in the gate as properties; then a group's meaning computed once, so that no second reading of it can disagree (below, "Step 4b") — no new surface and no rule changed | the three properties green in the gate, each shown able to fail; the structural rule with its breaker; step 4's regression tests untouched and green; M3 again; one more outside audit, its yield reported beside 6, 5, 2, 3 |
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

## Step 4b — the properties, before step 5

Draft for the owner's edit, 2026-09-20 (owner, the same day, on the
diagnosis below: "I agree"). Run in a fresh session: the one that built
step 4 is long, and its author has now judged four rounds of his own
repairs.

**Why.** Four outside audits of step 4 found 6, 5, 2 and 3 defects, all
sixteen real (the record below). Fixing what each found did not bring
the yield down, for two reasons the findings share:

- *One fault, five times.* A group's meaning was read again, off its
  syntax, by whichever function needed it next: which sources it admits,
  its selector, its together bound, the slot check, the zero block. Each
  round fixed the reader it was shown. Measured at `c3de464d`: 29
  functions in eight files match on a group's tree; 15 build, print or
  validate it (`parse`, `print`, `json`, `tree`), and 14 read what it
  means, across `bind`, `eval`, `answer` and `template`. This is P5's
  case — a rule that can be broken silently — held so far by discipline.
- *The builder counted examples; the auditor generated.* Every finding
  came from a transformation the hand-counted fixtures never made:
  parenthesise the group, resolve the unknown either way, run the route.
  The third audit's own check ran counts against routes over 2,000
  generated queries.

**What, in this order** — the properties first, so that the refactor is
made under them and not beside them.

1. *Three properties, in the gate*, over generated queries and generated
   items, through the crate's boundary (a request in, an answer out, as
   JSON — rule 3). `proptest` is already a dev-dependency, and
   `tests/language.rs` has generators for trees and groups.
   - **Equivalence** — invariant 7 of the surface, the builder's
     proposal, which the owner approved (2026-09-20: "yes, i agree"): what
     four audits found broken was a rule nobody had written down. A
     rewrite that changes no meaning — parentheses around any of a
     group's conjuncts, a doubled not, the same for the item's tree —
     changes nothing in the answer but the canonical text
     and the paths: the same counts, together count, resolved values,
     zero block, rows, and the same error or none.
   - **Routes.** Every route an answer prints binds and is not refused
     (rule 5), returns exactly as many as it counted, and a term's four
     routes partition the scope (invariant 4).
   - **Completion.** C93's own definition, as a test. An item with
     something unread is given completions of it — each unread flag a yes
     and a no, an unread array as none and as lines of the templates the
     query names, an unread number as several — and whatever the answer
     called decided on the item as stored — a term, a sum, a sort scalar,
     the root — is the same under every one of them. That half binds: one
     counterexample is a defect. The other half is measured, never
     required: how often an answer called undecided is the same under
     every completion tried, each kind explained or fixed, as M2 explains
     its differences — a sampler cannot show that no completion differs,
     and a rule may be careful on purpose (a comparison on an incomplete
     subtotal is undecided whatever the subtotal already reaches). The
     evaluator over fully readable bodies is the oracle, which is the
     half step 4's hand counts and the real copy already exercise.
   The generators and the oracle are not the builder's alone to write
   (author is not test): the outside agent's harness is asked for as a
   committed test, and the builder's properties are audited as code that
   must be able to fail — a mutant per property, as step 4's fixes had.
2. *A group's meaning, once.* What a group means is computed when it is
   bound and read from there by every consumer. The boundary is the
   property, not the shape: outside the modules that build, print and
   validate the tree, and the one that computes its meaning, nothing
   matches on a group's tree — a rule `tools/docs-check.sh` refuses, with
   a breaker. Internals are the builder's; the answer's JSON does not
   move, and step 4's tests are the proof that it did not.
3. *Then one audit round more*, pointed where the four were not — the
   rows, the zero block, `show`, `--describe`, the CLI's text — and at
   the properties themselves.

**Asked of the outside agent** (2026-09-20, by the owner's hand, since it
cannot read this session): its check of counts against routes over
generated queries as a test file, through the crate's public boundary
alone; the list of transformations it tries on a fixture, so that the
properties cover them; and to fix nothing itself. Its next audit waits
for 4b's close, and will look at the rows, the zero block, `show`,
`--describe`, the CLI's text, and whether the property tests — its own
among them — can detect a meaningful fault.

*The routes property has landed*, the outside agent's file as it wrote
it: `crates/acquisition-search/tests/generated_routes.rs`. 512 cases of
three queries each over generated items with unread evidence — flags,
arrays, `hybrid`, `ilvl`, an influence — and six anchor items from the
audits; membership by id, the scope taken from the fixture's ids and
never from the evaluator, an empty scope, routes followed past the first
page, the together and root routes among them; negative controls on the
checker itself. Read whole by the builder before it was committed; 8.1 s
in a debug build here. Three of step 4's mutants tried against it, each
caught and shrunk to a query: the failed route ignoring an open selector,
the selector read off the syntax, and the selector's polarity under a
not — the last being the one the builder had recorded as something
routes could not see because they are made from the selector. They
could not over seven hand-made items; over a generated population they
do. As its own header says, it checks that an answer agrees with itself,
never that it is right: equivalence and completion are still 4b's to
write.

*The transformations the properties are to cover*, the outside agent's
list, as given:

1. Parenthesise and reassociate conjunctions at both item and occurrence
   levels.
2. Nest comparisons beside template, source, and flag restrictions.
3. Add double negation; exercise predicates beneath negation and
   disjunction.
4. Reorder predicates and occurrences; compare meaning while preserving
   authored text.
5. Complete unread flags as both true and false, independently of sibling
   flags.
6. Complete unread arrays as empty, nonmatching, and matching
   occurrences; complete unread numbers with several values.
7. Add irrelevant unread fields or excluded sources; established answers
   should survive.
8. Add readable witnesses beside unknown occurrences, including smaller,
   larger, and slotless candidates.
9. Probe bounds immediately below, at, and above values, including zero
   and negative values.
10. Duplicate occurrences and items separately; check sums, existential
    matches, and item counts.
11. Compare shorthand, projections, explicit groups, text requests, and
    returned-tree requests.
12. Run every printed route, compare membership, and partition the scope —
    including empty scopes and results beyond pagination.
13. Cross-check value probes against sort status, resolved selectors
    against the zero block, and row evidence against `show`.

Of these, 12 is the landed file's; 1 to 4 and 11 are equivalence; 5 to 8
are completion; 9, 10 and 13 belong to neither and want generators or
cross-checks of their own.

**Not in 4b.** No new surface. No rule of the language or the answer:
finding 7 (which identity names a store in the basis) and B1–B11 are the
owner's and wait on nothing here. Whatever a property finds that changes
what a user types stops and comes to him (rule 4, as the first audit's
finding 8 read it).

**If the properties find a lot.** They may: they make every audit's
transformation at once. Defects are fixed under them; a property that
cannot be made green without a ruling is reported with its
counterexample and marked as waiting, never loosened to pass.

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
| 4 · the first surface | `28606ed4` | `acquisition-search` links the store (C89) and gains `bind.rs` (the vocabulary as closed lists, near names, the one list of what is not built, a bare word's closed-set readings), `corpus.rs` (every live item of a scope derived from one `read_corpus`, held with its basis, its places and the coverage the scope block states), `eval.rs` (matched, failed, lacked, undecided; witnesses; three-valued composition and the `holds` interval; sums with their status; the sort scalar; the together count), `answer.rs` (the request and the answer: a route on every count that is not zero, what a `:` or `~` selector resolved to, rows with what matched, the zero block), `describe.rs`, `show.rs`. The CLI gains `acq search` and `acq show` (`search_cmd.rs`), text rendered from the value `--json` prints, the flags of steps 5 and 10 refused by name; README tour lines; `CLI-REFERENCE.md` regenerated. Tests at the crate's boundary, every count worked by hand first: `tests/answer.rs` (the worked example reduced, the route property, invariants 2 and 6, C93's composition, C92's scalar, C96's scope, C98 at the answer's boundary), `tests/acceptance.rs` (OQ1–OQ4, OQ7, AQ2–AQ5 as far as step 4 builds them; S12, S52 and S107 by their wording), `tests/refusal.rs` (the walk over `language.toml`, invariant 3, S53), and the CLI's `tests/search_json.rs`. Three mutants tried, each caught: unread beating a witness, a line's failed route as `-term`, not-undecided as true. M3 and M4 below. |
| 4b · the properties | `39e7667a`–`36a9738a` | Three properties beside the outside agent's routes, each through the crate's boundary over generated queries and generated items with unread evidence, from one set of generators (`tests/common/generated.rs`: a query tree of the test's own with its normal form, and bodies with holes that a completion fills): `tests/generated_equivalence.rs` (invariant 7), `tests/generated_completion.rs` (C93's definition, its measured half run by hand), `tests/generated_cross_checks.rs` (transformations 10 and 13). Then `group.rs`: what `line( … )` means, computed once at binding and read from there, the bound form private to it; `tools/docs-check.sh` §7 refuses a source file that names a group's tree outside `tree`, `parse`, `print`, `json`, `template` and `group`, with five breakers (`tools/docs-check-breakers.sh`, 56 ok). No test of step 4 was touched and the answer's JSON did not move. What they met is below. |

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

### Step 4 — M3, M4, and what the first surface met

**M3, M4** — `python3 search/item-facts/scripts/m3-ask.py
search/item-facts/raw/spike-GERWARIC_7694-2026-09-13.db`, after the three
builds its header names, 2026-09-20. The script writes a fresh `.backup`
of the copy under `raw/m3/`, wraps it as a store directory
(`crates/acquisition-search/examples/copy-as-store.rs`) and asks through
`acq --json search --realm pc … > /dev/null` under `ACQ_PROVIDER=mock`
and `ACQ_STORE_DIR`: 22,623 live pc items of the copy's 22,721 (the rest
are poe2's), revision 1126, facts v7. Twice: the build at `28606ed4`,
first ask 418 ms, and the build the two audits below left, first ask
270 ms. Warm, the median of ten, in ms:

| Ask | Release at `28606ed4` | Release, audited | Debug, audited |
| --- | ---: | ---: | ---: |
| the empty query | 254 | 268 | 1,727 |
| OQ1 (`base:ring` for the class) | 267 | 287 | 2,192 |
| OQ1 refined by a fractured line | 262 | 283 | 1,998 |
| OQ2 `name="…"` | 254 | 269 | 1,733 |
| OQ3 a mod · a base | 259 · 253 | 276 · 270 | 1,892 · 1,732 |
| OQ4 a base and a phrase | 264 | 281 | 1,969 |
| OQ7 one line everywhere | 255 | 271 | 1,741 |
| AQ2 OQ1 and a `sum` over `template:resistance` | 271 | 297 | 2,427 |
| AQ5 life at 90, sorted | 256 | 284 | 1,818 |
| M4 a phrase · `text~"explo(de\|sion)s?"` · `text~"^adds [0-9]+ to [0-9]+"` | 261 · 262 · 260 | 277 · 277 · 275 | 1,893 · 1,895 · 1,817 |

Every release ask is under 500 ms, so the persisted-projection park's
trigger does not fire. An ask is its load: the empty query, which
evaluates nothing, is 268 ms, the dearest query adds 29 ms, and `~` over
every displayed string of every item adds 9 ms (M4). The debug build is
1.7 to 2.4 s, which is what the seat feels behind the README's alias.

What the audits' fixes cost was measured apart, the two release binaries
asked in turn on the same copy in the same minutes (the pre-fix one built
from a worktree at `28606ed4`), the median of eleven: the empty query 257
against 269 ms, OQ1 277 against 295, AQ2 277 against 301, a flag asked
inside a group 262 against 293, `~` over all text 264 against 279 — 12
to 31 ms, five to twelve in a hundred, the deriver's flag checks in every
ask and the three-valued group in those that have one.

**Holes — each changes what a user types or which items it matches, so
they are the owner's.** None blocked the step: each is built in the
direction that breaks least if he rules the other way. B2–B11 ruled as
recommended, 2026-09-20 (B8's recommendation being the seat's); B1 ruled
the same day on the measurement beneath the table — "i agree with
any-case everywhere after this investigation" — as built, the builder's
split withdrawn: a quoted template that found two spellings lists them,
and step 5 gives such a pair's vocabulary rows a pattern that turns case
back on.

| # | Hole | Built as | Recommendation |
| --- | --- | --- | --- |
| B1 | The case of `=` on text. The reference gives any case to a phrase, `:` and `~`, and says nothing of `=`; `rarity=rare` must find GGG's `Rare`, so `=` is any-case on a closed set already | every text comparison is any-case: `name="kaom's heart"` finds `Kaom's Heart`, and a quoted template finds its line however its capitals were typed | keep: one rule, and tightening later removes matches where loosening only adds them |
| B2 | Which words `is:`, `source=` and a line's `is:` take. Invariant 3 forbids asking the corpus, and the deriver's lists are open (a key GGG adds is read the day it appears) | closed lists in `bind.rs`, GGG's spellings in any case, as the census copy holds them (measured 2026-09-20: 25 item flags with the four influences, 10 sources, 3 line flags); an unknown word is an authoring error with the near ones. A flag GGG adds is derived and shown by `acq show`, and cannot be asked for until the list gains it | keep; the differential (`m2-differential.py`) is where a word outside the list would first be seen, and nothing checks that yet |
| B3 | `tab:` on an item in a substash | tests the substash's name and its tab's, so `tab:maps` finds a map; a folder's name is no part of it | keep; a `folder:` field is an addition if asked for |
| B4 | What `id:` matches. The reference: "any id an answer printed", and a row prints its place's id beside its own | the item's id, or its tab's, substash's or character's, whole; `:` and `=` mean the same. `acq show` takes an item's id alone and says what a location's id is, with the search that lists it | keep |
| B5 | What `--sort` takes | a number: `ilvl`, `stack`, `line(P).<slot>`, `sum( … )`; a text field is an error that says so | keep until someone sorts by name |
| B6 | A bare word's readings. The reference shows two for `rare` (`rarity=rare · "rare"`) and the parser has offered `"rare" · line(template:rare)` since step 1 | the closed-set readings first, the parser's two after: three for `rare` | keep: a third valid reading costs a line |
| B7 | Failed against lacked on a line's group — both false, so no answer's members move, only how the false are split | the selector is the group without its slot comparisons: an item with no occurrence the selector picks lacked it; one with such an occurrence and none satisfying the whole failed. A group comparing no slot never fails | keep |
| B9 | A query that starts with `-`, the language's not, at a terminal, where it is a flag: `acq search -is:corrupted` is clap's error, and `-has:note` would be read as `-h` (audit finding 1) | it goes after `--`, as every route prints it and the verb's help says: `acq search --realm pc -- '-is:corrupted'` | keep: accepting a leading hyphen as the query would turn a mistyped flag into a query |
| B10 | The order of an item whose largest occurrence is not established (the second audit, finding 3). C92 gives no scalar to an item with no satisfying occurrence; it says nothing of one that has a readable occurrence and an unread source the group admits | it sorts last either way, as an incomplete sum does, the readable value shown with `incomplete` beside it. A term is another matter: a readable 95 is a witness to `>=90` still | keep: an order that may be wrong is worse than a place at the end that says why |
| B11 | Whether parentheses may change what is an error. Step 1 checked a slot against a quoted template only at the group's own level, so a nested template took any slot word — and the answer then printed a route the build refuses (the fourth audit, finding 1) | the check reads the group's conjuncts, through nested ands: the tighter reading, which the owner's own test prefers — allowing a query later breaks nothing, forbidding one later would. A template under an or or a not states no numbers still, as step 1 says | keep |
| B8 | Gap 2, as this plan recommended | (b): `--query-file <file\|->`; a route is printed shell-quoted, an apostrophe as `'\''`. (c) is untouched | the seat's |

*B1's measurement* (2026-09-20, `item-facts/data/mod-templates.csv`,
its three inputs): of 6,549 distinct templates, six pairs differ only by
capitals, every one a line GGG has spelled two ways — `# Maximum Stages`
16 items and `# maximum Stages` 30; `#% chance to Cause Bleeding on
Critical Strike` 7 and `… cause …` 21; `Fires Projectiles every #
seconds` 1 and `… projectiles …` 8; `Gain # Life per Enemy Killed` 209
and `… enemy killed` 2; `Socketed Gems are Supported by Level # Blind` 1
and `… supported …` 21; `… Supported by Level # Chance To Bleed` 1 and
`… supported by Level # Chance to Bleed` 3. Under any-case `=` a quoted
template selects both spellings of its pair, and step 5's two vocabulary
rows would carry one selector (C97); under an exact `=` the commoner
spelling typed misses the rarer one's items with no sign of it (S171's
R1).

**An outside audit of the build at `28606ed4` (2026-09-20), each finding
reproduced with a fixture of the builder's own before it was taken.**
Verdict of the audit: keep step 4 open. Finding 7 and B2–B11 were ruled
2026-09-20 (owner: "Otherwise agree with all you recommendations");
B1 the same day;
the reference carries them since (owner, on the text proposed: "R1-12
are approved"), and a seventh invariant of the surface with them — that
parentheses group and do nothing else — the builder's proposal, approved
the same day. Nothing of step 4 waits on the owner now; what is left is
4b's.

| # | Finding | Verdict | Held by |
| --- | --- | --- | --- |
| 1 | A route whose query starts with `-` is refused by clap, exit 2: every `-has:`, `-is:` and `-line(…)` route, which is most lacked and many failed ones — the plan's rule 5 broken. The CLI's route test had not met one: its fixture had no lacked count | confirmed | the query after `--` (B9); `tests/search_json.rs` runs a lacked route through a shell, and fails without the separator — tried |
| 2 | A flag that could not be read answered as a no: a line with unread `flags` matched `-is:crafted`; `"corrupted": "unread"` matched `-is:corrupted`; and `"corrupted": false` beside an unread `influences` was undecided. The builder had recorded the line's case as an observation and called it harmless; it was a false match against C93 | confirmed, and wider: half of it was step 2's deriver, which recorded no unread at all for a flag's value that is no boolean | the deriver: `ITEM_FLAGS` values checked, `Line::flags_unread`, `Part::Flags`; the evaluator: a line's group three-valued on each occurrence, a flag's own key and `influences` for the six that live there, a phrase left open only by what holds a displayed string. `tests/unread.rs`; M2 rerun, 0 unexplained and 0 unread |
| 3 | An unread `hybrid` was read as the absence of hybrid lines: `-line(source=hybrid)` admitted the item | confirmed | `unread_lines`; `tests/unread.rs`, which fails without it — tried |
| 4 | Parentheses changed an outcome: `source=explicit` ruled the unread implicit array out only as an immediate member of the group's and | confirmed | which sources a group admits is asked of its meaning — the group with its source tests answered and all else unknown (`eval.rs`, `admits`); five spellings pinned against three that must stay open |
| 5 | What a selector resolved to was filtered by the group's comparisons, so a template whose value failed was not listed, and a `sum`'s selector resolved to nothing | confirmed | resolved by the group's selector over the scope, a `sum`'s too; the worked example's count moved from 6 to 7 for that reason |
| 6 | Following an undecided route returned the members without their reasons, so past the ten listed there was no way to them | confirmed | an `undecided( … )` that matched shows the reasons of what it asked about (`Evidence::Undecided`); pinned over twelve items |
| 7 | The basis names the account and not the store, against C98's words; two stores of one account gave equal bases and `is_current` said yes across them | confirmed — and the owner's: which identity names a store (its path, the world's id of C83, an id the file carries) is the basis as the contract, one of the six lines the seat revisits first | **ruled 2026-09-20: "(a') now and park (c)"** — twelve hex digits of the SHA-256 of the file's canonical path, as C83 names a world: no path in an answer, no migration. `Basis::of`, and `is_current` compares the whole basis; the auditor's two stores pinned, and the check without the store tried and caught. An id the file carries is parked with the refetched file (`decisions/search.md`) |
| 8 | Rule 4 says an unstated rule that changes what a user types stops the step, and the record listed eight such holes and said none blocked | narrowed: the step was built as steps 1 and 2 were — in the direction that breaks least, the holes brought to the owner at the close, said before building — but a recommendation is no ruling, and the step is not closed until each is ruled and the reference carries it | open: B1–B9 |
| — | `eval::texts` and `answer::item_texts` were one accessor written twice | confirmed | one, in `eval.rs` |

Fixing finding 2 opened an edge of its own, closed in the same change: a
group whose selector asks a flag can fail on an item whose occurrence
leaves the selector open, and the failed route `line(S) -term` would not
return it. For such a group the route is `(line(S) or
undecided(line(S))) -term`; elsewhere it stays as short as the
reference's. One mutant per fix was tried, and each is caught by the
test that names it.

**A second audit, of the fixes at `ed1b5446` (2026-09-20), each finding
reproduced before it was taken.** Findings 7 and 8 of the first stand as
it left them.

| # | Finding | Verdict | Held by |
| --- | --- | --- | --- |
| 1 | The selector dropped a whole member of the group's and when any comparison sat inside it, its template and source restrictions with it: a lacked count of 1 whose route returned 0, and a nested group resolving to templates the unnested one did not | confirmed: the first audit's finding 4 over again — source eligibility had been moved from syntax to meaning and the selector left behind, the second of the six places the builder had said he trusted least | the selector is the group with each comparison taken as favourably as it can be — true, or false under a not — and folded (`bind.rs`, `selector`), so the ordinary route is still the reference's. Five nested spellings in the route property. A mutant that ignored the not survived the routes, which the builder put down to their being made from the selector — true of his seven hand-made items, and not in general: the generated routes of step 4b catch that mutant; the failed-against-lacked split of one item is stated by hand as well — then caught |
| 2 | An unread flag erased the known no beside it: with `{"crafted": false, "fractured": "unread"}`, `-is:crafted` was undecided, and `{"shaper": false, "hunter": "unread"}` left `-is:shaper` open | confirmed: the first fix was per object and had to be per flag | `Line::flags_unknown` names the flags of a readable object that could not be read, `flags_unread` is the object itself; an influence is unread under `influences.<flag>` |
| 3 | The sort scalar ignored an occurrence the group may select: 20 shown as the largest beside an open 95 | confirmed, and wider — a source the group admits and could not read may hold a larger one too, which the first build had as well | `Scalar::Incomplete`: what was readable, its status, no place in the order (B10). An open occurrence that cannot pass the largest changes nothing, pinned |
| 4 | `DERIVATION` stayed 1 though the same malformed body now derives to another item — against the rule written on the constant | confirmed | 2 |
| 5 | The timing note claimed more than it had: an unchanged reader running slower shows the machine changed, never that the evaluator had not | confirmed — and the builder's cause was wrong as well as unproven. The slowdown was put down to battery and low power mode because that was found first; on the same battery in the same mode, an hour on, the pre-fix binary ran at the morning's 257 ms. What slowed the machine is not known | the two builds measured against each other, above; the claim that a release ask is over 500 ms in that power state is withdrawn |

**A third audit, of the fixes at `ad5589c9` (2026-09-20)**, which found
the earlier reproductions passing and two gaps more; its own check of
counts against routes ran over 2,000 generated queries.

| # | Finding | Verdict | Held by |
| --- | --- | --- | --- |
| 1 | An occurrence the group may select, and which names no such number, left a value open: over one line `Cannot be Frozen` with an unread `crafted`, `sum(… .arg1)=0` was undecided and `undecided(… .arg1)` matched, though either reading of the flag gives a complete zero and no occurrence. And binding `undecided( … )` of a value threw its slot away, a sum and a projection made one probe | confirmed | an open occurrence counts only where it names the slot — in a sum, in the largest, and in the together count, which had the same fault and was not in the finding; `undecided( … )` of a value binds as `--sort` does and is open exactly when the value would sort as incomplete, one function for both (`eval.rs`, `scalar`) |
| 2 | Parentheses changed whether the together count applies: the bound was looked for among the group's immediate members | confirmed: the third time one fault — a meaning read off the syntax (source eligibility, the selector, now this) | the bound is the one comparison among the group's conjuncts, through every nested and and a doubled not; four spellings pinned to one count, one route and one selector, and four shapes that are no single lower bound pinned as not applicable |

After it the builder walked every function that reads a group's tree,
found the rest recursing and one reading by position — step 1's slot
check — and recorded it as never a wrong answer, only a missing error.
The fourth audit showed that record wrong.

**A fourth audit, of the fixes at `b2b8d5fb` (2026-09-20)**: both
third-round fixes held, no defect found in `show`, three more.

| # | Finding | Verdict | Held by |
| --- | --- | --- | --- |
| 1 | B11 breaks rule 5: `line(("# to maximum Life" source=explicit) arg3>=0)` was accepted, counted one item together, and printed a route — `sum(line("# to maximum Life" source=explicit).arg3)>=0` — that the same build refuses | confirmed; the builder's note of the round before was wrong | the slot check reads the group's conjuncts, one function with the together bound's (`template::conjuncts`), so the query is step 1's `slot_unknown` however it is parenthesised; and the together route is checked before it is offered, since folding can bring a template to the selector's and that the whole holds under an or — `("T" or false()) arg3>=0`, found while fixing, and caught only by that guard |
| 2 | The zero block said a group resolved to nothing while the terms block listed what it resolved to: `nothing()` took the group's first template test and asked it alone, out of its and, or and not. A `sum`'s selector was never diagnosed | confirmed: the fourth reading of a group's meaning kept apart from the others | resolved to nothing is the bound selector picking no occurrence in the scope, a `sum`'s too; a field's term is as before |
| 3 | Sorting by an unread number said `no satisfying occurrence`, a known absence, where `undecided(ilvl)` said unread of the same item | confirmed | `Scalar::Incomplete` for a field that could not be read |

**Observations — the builder's.**

- The walk found two cases of step 1's corpus using fields the builder
  had invented (`quality`, `weight`), which no step builds and no list
  could refuse; they now say `stack`. `reqlevel` (OQ5's) is the
  reference's, from its contract detail, and joined the not-built list at
  step 6, where OQ5 becomes askable.
- Three of the builder's hand counts were wrong before the code was: the
  order of a row's lines (by source word, step 2's observation), four
  resistance templates counted as five, and a suggestion ranked by a rule
  the builder had not read closely — a typed `Resistances` did not share
  a word with `Resistance`. The first two were the tests'; the third was
  the code's, and sharing a word now holds either way round.
- The text was changed by its first sight of the real copy, which no
  fixture had shown: the routes printed by default were the largest and
  least useful (8,600 items that are not rare), so the default is now the
  undecided and together routes, and each term's matched route when the
  total is zero, with `--routes` for all; a template over several rows
  broke the layout and prints its `\n`; ten resolved values ran off the
  line, so the text lists five and the JSON ten.
- `acq show` first loaded and derived the whole corpus to show one item;
  it is one pass of the read now, deriving the one row it finds, and what
  an item is socketed in is given by id, which `show` takes in turn.
- Nothing on the copy is unread, so every undecided outcome — a term's,
  a sum's, the root's — is exercised by fixtures alone, as at step 2.
- The scope says `in all leagues`; a league is a term, and the owner's
  seat forgetting it is the default-league park's trigger, not a step's.
- The gate failed once on the refusal walk, which found twelve items in a
  store that holds none: the fixture helper named its directory by the
  process id and never cleared it, the id came round again, and an
  earlier run's file — the acceptance fixture's — was opened as the new
  store. 871 such directories had gathered in a day. Both helpers clear
  the directory first now. The store's own tests name theirs the same
  way (`acq-corpus-<pid>-<n>`) and were not touched.
- Linking the store turned two of C89's breaker cases over, as step 1's
  own comment said it would: the case adding the store added a key twice,
  and the store linking the search became a cycle Cargo refuses before
  the rule is asked. Both say so now (`tools/docs-check-breakers.sh`, 51
  ok). The breakers are not in the gate, so nothing but running them by
  hand would have shown it.
- What B3 rests on, measured over the copy when the owner asked what
  `tab:maps` meant (2026-09-20): 15 map tabs hold 2,566 substashes and 29
  unique tabs 491, and a substash is named `1`, `4 (Remove-only)` or
  nothing, so without its tab's name an item in one could not be found by
  tab at all. `tab:` tests names and never GGG's `type`, of which the copy
  has 16; the type as a field is parked (`decisions/search.md`).
- `Folder` has its fourth consumer, as step 3 said it would
  (`corpus.rs`, `is_folder`).
- Over the copy, `name="Ashes of the Stars"` finds ten, which is the
  variant park's first test (below, "Parks whose triggers the build
  fires"): step 4 has reached OQ2, and the question is the owner's.

### Step 4b — what the properties met

**No defect of the build was found by any property** — of the first
surface as the fourth audit's fixes left it (`c3de464d`). That is a claim
about the properties as much as the build, so each was shown able to fail
before it was believed, and the outside audit this step closes on is the
check of both.

**Every property passed at its first run, and each time that was a
signal and not a result.** Equivalence: a third of its queries were
authoring errors (a slot its quoted template lacks) and 837 of 870 bodies
had a hole; rebalanced, 7 queries in 100 err and a third of bodies are
fully readable. Its one failure since was the test's — the reasons an
`undecided( … )` shows follow the order its terms were written in.
Completion: its completions were too few and too random to move an
equality or an element that is no line, so its measurement said little;
the fifteen it tries now are in its header. Cross-checks: believed only
after the three mutants below.

**The mutants** — each a fix of step 4's audits undone, or a fault of the
same kind; tried against the three generated properties before part 2,
and A–E, G and H again after it, in `group.rs`, with the same result.
`·` survived; blank, not tried.

| | Mutant | routes | equivalence | completion | cross-checks |
| --- | --- | --- | --- | --- | --- |
| A | the together bound looked for among the group's immediate members | · | caught | · | |
| B | which sources a group admits, read off its immediate members | caught | caught | caught | |
| C | the slot check at the group's own level only | · | caught | · | |
| D | the selector dropping a whole member that holds a comparison | caught | caught | caught | |
| E | the selector ignoring a not | caught | caught | caught | |
| F | unread beating a witness | · | · | · — seen by the measurement | |
| G | a line's unread flag read as a no | · | · | caught | |
| H | an unread array read as empty | caught | · | caught | |
| I | a sum ignoring an occurrence it may select | · | · | caught | |
| J | a largest ignoring a larger that may be selected | · | · | caught | |
| K | the zero block reading a group's first template alone | · | · | · | caught |
| L | an unread `ilvl` read as absent | · | · | caught | |
| N | `undecided(V)` missing an incomplete value with nothing readable | | | | caught |
| O | a sum counting one text once | | | | caught |

Every reading of a group off its syntax (A–E) is caught by equivalence,
which is what part 2 was made under. F makes an answer more careful and
never wrong, so nothing that binds can see it: step 4's hand test does
(`tests/answer.rs`), and so does the measurement. K is the same wrong in
every spelling and under every completion; only a second part of the
answer contradicts it.

**The completion property's measured half** —
`cargo test -p acquisition-search --test generated_completion -- --ignored --nocapture`,
2,000 cases from a fixed seed. Run 2026-09-20 before part 2 and after it,
the same numbers both times; the figures here are the run of 2026-09-21,
over the generators as the fifth audit left them (decimal lines, a second
spelling, selectors that resolve to nothing), 174 of the cases authoring
errors. What was decided as stored and held under all fifteen
completions: 3,400 terms, 1,288 roots, 3,782 sort scalars;
counterexamples, none. What was undecided as stored:

| Undecided | Completions differed | Every one said no | Every one said yes |
| --- | ---: | ---: | ---: |
| a field, a flag or a phrase | 167 | 49 | 0 |
| a line's group | 419 | 535 | 0 |
| a sum's comparison | 110 | 164 | 99 |
| the root | 227 | 226 | 85 |
| a sort scalar | 953 | 888 with one value | |

The kinds that did not differ, read from the examples the run prints:

- *The sampler cannot reach it.* An element that is no line becomes one
  line at most, and the term wants a template at one value (`arg1=11`); a
  selector nothing carries (`template:nothing`); a phrase no generated
  line says (`"Level: 8"` where an array is unread); an upper bound no
  generated value breaks (`sum( … )<=21`, values to 12). Each is
  undecided rightly: a completion that differs exists and was not tried.
- *A group no occurrence can satisfy* — two quoted templates in one and,
  `arg1<2 arg1>=12`. Every completion says no, and the evaluator says
  undecided while a source the group admits is unread: it asks which
  sources a group admits and never whether anything could satisfy it.
  Careful on purpose; no rule of the reference asks for more.
- *A comparison on an incomplete subtotal*, undecided whatever the
  subtotal already reaches: the reference's sum-status table, and this
  section's own example of a rule careful on purpose. The eight the
  first run printed of its 110 sums every completion made true were this
  kind or the first, and the eight roots it printed inherited a sum or a
  negated group; the rest were not read.
- The column that matters is the empty one: **no undecided group, field,
  flag or phrase was made true by every completion**, which is where a
  missed witness shows. With mutant F applied, 83 groups and 18 fields in
  600 cases are.

**Observations — the builder's.**

- *Failed against lacked under completion.* A line's group that failed as
  stored lacked under a completion 19 times in 2,000 cases (21 on the
  second day's generators): the selector
  was open on an occurrence with unread flags, the whole was false on it
  whatever the flag, and a completion answered the flag no. The property
  lets this through and counts it: *failed* on a group says the truth is
  established and the absence is not, *lacked* is the claim that needs
  everything readable (C93), and no answer's members move (B7). A
  matched, a lacked, and a failed that turned true would each be a defect.
- The transformations: 1–4 and 11 are equivalence's, the returned tree
  sent back among them; 5–8 completion's; 9 is the generators' one small
  range for values and bounds, zero and negatives in it; 10 and 13 are the
  cross-checks'; 12 is the landed file's. Of 13, a row's evidence is
  checked to be lines `show` derives of that item, never that they are the
  lines that satisfied the term.
- What no property here sees: a fault every spelling and every completion
  shares and no second part of the answer states; the text the CLI
  renders; `--describe`; whether an error's readings are good ones. The
  hand-counted tests hold the first as far as they go, and the next audit
  is pointed at the rest.
- The rule names and does not only match: `use … Member as M` was a way
  round the first wording, found by asking how the check could be passed
  without being obeyed, and is a breaker now.
- The gate's cost, a debug build here: equivalence 14 s, the cross-checks
  15 s, completion 3 to 6 s, beside the routes' 16 s. 256, 192 and 256
  cases; `PROPTEST_CASES` runs any of them longer by hand.
- The commit of part 2 first stated its line counts from memory, twice
  wrong, and was amended from `git diff --numstat` (the trap
  `search/README.md` already names).

**M3 again** — the same command as step 4's, 2026-09-20, the build at
`36a9738a`: first ask 445 ms; warm, release, 270 to 299 ms over the
thirteen asks (the empty query 270, OQ1 288, AQ2 299, `~` over all text
277), debug 1,739 to 2,438 ms. Every release ask is under 500 ms. Each
release figure is within 9 ms of the audited build's column above, taken
the same day (OQ2 the furthest, 269 to 278); the two builds were not run
against each other, so that part 2 cost nothing is not claimed.

**A fifth audit, of step 4b at `ef381909` (Astra, 2026-09-21)**, the one
this step closes on: keep `group.rs`; six defects of the first surface
that four generated suites had passed over, none of them the
extraction's, and two corrections. Each reproduced as a failing test
before it was taken (`tests/fifth_audit.rs`; the CLI's `search_json.rs`
for 5). Yield by round: 6, 5, 2, 3, 6 — and a seventh, found the same day
by the check the audit asked for.

| # | Finding | Verdict | Held by |
| --- | --- | --- | --- |
| 1 | A sum of decimals depended on the order of occurrences: 0.1, 0.2, 0.3 summed to 0.6000000000000001 one way round, so `sum( … )=0.6` matched one of two items carrying the same lines | confirmed, and wider: the mean of a ranged pair had it too (`avg=0.15` over 0.1 and 0.2) | `exact.rs`: arithmetic on whole hundred-thousandths, in integers, read back as the nearest float, which is the float a typed bound is; one function for a sum, the together count and `avg`. No rounding anywhere. Its first form, a decimal sum with a float fallback, is follow-up 1 below |
| 2 | A suggestion counted one spelling and offered an any-case `=` that returns every spelling: said 1, returned 2 | confirmed | a suggestion's count is its term's, asked of the scope as it will be typed; a second spelling of a value already offered is the same term and is not offered again |
| 3 | A pattern that resolved to nothing was missing from the zero block: being listed was coupled to having words to suggest by | confirmed | listed with no suggestion — a group's, a `sum`'s, a field's |
| 4 | What a row shows of one term was unbounded for a group and a sum (40 lines) and cut to three for a phrase without saying so (invariant 5) | confirmed | six, a sum's value beside them; `left_out` counts the rest and the CLI prints it with `acq show <id>` |
| 5 | Under `--realm all` a row did not say which realm its item is in | confirmed | the renderer: the realm before the place, as `show` prints it; under one realm the scope line has said it |
| 6 | `--describe` did not say how terms compose or what has a value, and refused the slot and operator words its own doc promised | confirmed | two blocks, `composition` and `values`, an entry a line and an example, every example bound by a test; an entry answers to each word of its name, the positional slots to any `arg<N>`, a block to its own. The reference stays the manual |
| 7 | — found by the both-ways zero-block check on its first day: a closed set's `:` selector that matched nothing (`rarity:ma` over rares) showed an empty resolved list and no zero-block entry | the builder's, the same fault as 3 | listed; pinned beside 3 |
| — | The record said sixteen completions: there are fifteen, sixteen bodies with the stored one | confirmed | here and above |
| — | The resolved-values continuation always said `--count line` | confirmed | `--count <field>` for a field's values |

**The audit's review of those fixes (`c4fc8f13`–`15fc914a`), the same
day**: they hold; three follow-ups, each reproduced as a failing test
first, fixed at `7f5fd962`.

| # | Follow-up | Verdict | Held by |
| --- | --- | --- | --- |
| 1 | `exact.rs` fell back to floats when its integers overflowed, and whether a sum fell back depended on the order its occurrences cancelled in: one order of 1e28, 1e28, −1e28, −1e28, 1e-10 gave 0, another 1e-10 | confirmed | a number is read once, where the body is read: ten whole digits and four decimals (measured over the census copy: 212,233 displayed numbers, none past two decimals or ten whole digits; the owner: "I have never seen more than 4 digits after the decimal"). One beyond it costs its line its numbers and leaves its array unread (C93), so nothing downstream has a fallback; 48 lines of code became 21. `DERIVATION` 3. M2 again: 0 unread, 0 unexplained |
| 2 | The reasons an `undecided( … )` shows were cut to six in the order the terms were written, so reordering a conjunction changed which was dropped — and the fixture, passed through the equivalence checker, was refused | confirmed, and wider: the total's undecided items listed their reasons with no bound at all | `eval::why`, one function for both: the bound is on the item's unread parts, the first six in the item's own order, every term's pair with each kept; `why_left_out`. The audit's fixture is a fixed case of `generated_equivalence.rs` |
| 3 | The `acq show` continuation dropped the account: with a second account known it does not run | confirmed, and wider: both offers of `show`'s errors dropped it too | `answer::command`, the one way the search prints a command — routes, the continuation, the offers. The CLI's rule-5 test runs every `acq …` a text prints, anywhere in a line, with two accounts known; the walk it replaces saw only lines that start with one |

The shared anchors gained an item past every cut (`1e951803`). With the
reasons ordered by authored term again, the fixed case fails at once;
the generated property passed its 256 gate cases and found it at case
531 of 2,000.

Why the suites had passed over them, which is the audit's other half:
the generators made whole numbers only and never reordered an item's
occurrences, though transformation 4 asks for it and this record had
claimed 1–4 covered; the zero-block check looked only at entries
present, so an emptied block passed it; a suggestion's count is named
`items`, which no walk over `count` sees; nothing bounded a row; and no
property reads the CLI's text or `--describe`, as the observations above
already said. The generators have decimal lines and bounds, a second
spelling of one line and selectors that resolve to nothing; the
cross-checks ask every occurrence in another order, the zero block both
ways, every suggestion's term, a row's bound, and two misspellings of
every corpus; negative controls refuse an emptied zero block, a
miscounted suggestion and a doubled row. Each fix undone is caught by
them: P a sum in float order, Q a suggestion counting its spelling
(survived 192 cases until the misspellings were asked of every corpus),
R a pattern dropped from the zero block, S a row unbounded.

Observations — the builder's. The derivation's version stays 2: a body
derives to the same item, what moved is a slot computed from it, and the
basis names no evaluator — so two builds can label different answers
with one basis, which was already true of every evaluator fix of step 4.
M3 once more, the build at `3c7d31f3`, 2026-09-21: first ask 491 ms;
warm, release, 266 to 296 ms (AQ2, the sum, 296), debug 1,729 to 2,441.

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
   4, and (c) left for the seat — adding it later breaks nothing. Built
   so at step 4 (hole B8).
3. **The values of `membership` — open; does not block before step 4.**
   The request carries it, the terminal synopsis has no word for it,
   and `live` is the only value shown; `acq items search --removed`
   exists today. *Recommendation:* `live` alone at the first seat; `all`
   (live and removed, each row marked) at step 11, so the old verb can
   retire; `removed` alone waits for someone asking. Step 4 builds `live`
   alone, which is all the store's read hands over.
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
