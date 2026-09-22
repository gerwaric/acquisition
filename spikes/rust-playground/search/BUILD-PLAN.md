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
   *means* — which items it matches — is. The unruled ones are at the
   foot of this file; the ruled ones, with their evidence, are in
   `SEARCH-SLICE.md`.
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

Rules 8 to 10 are what step 4b's audits taught (`SEARCH-SLICE.md`, the
findings of step 4b), each a fault that came back until it was named;
the owner approved their meaning from a plain account of each and left
the wording to the builder (2026-09-22: "the words that get written down
are for you"; "If so, go ahead"). Each names what already holds it: the
rule is for the moment before the code is written.

8. **Unknown is said once, where the evidence is read, and of no more
   than was lost.** What the deriver cannot read — a flag that is no yes
   or no, a number it does not read, an array that is no array — is
   unread there, at that grain: the flag, the slot, the array. Too small
   and it reads as absent, and a not makes a witness of it; too large and
   terms that never needed it are left open (C93). Nothing downstream
   has a fallback or a second way to fail. *Held by:* the completion
   property; `derive::Slot`, `Line::flags_unknown`, `Unread::line`.
9. **A cut follows the data, never the query.** A block that shows part
   of something orders it by the item or by the counts, so that two
   spellings of one question show the same part (invariant 7), chooses
   what is relevant before it cuts, and counts what it left out in the
   unit it cut — rows, values, unread parts. *Held by:* the equivalence
   property, where a generated or fixed case crosses the bound: a new
   bound needs a case past it.
10. **One maker for each kind of thing an answer holds.** What a group
    means (`group.rs`, refused elsewhere by `tools/docs-check.sh` §7), a
    command (`answer::command`: rule 5 is this rule for commands), why
    an item is undecided (`eval::why`), a sum (`exact.rs`). A second
    maker is how two parts of one answer come to disagree: look for the
    first before writing one. *Held by:* §7; the CLI test that runs
    every command printed; the cross-checks.

## Steps to the first seat

| Step | Builds | Closes on |
| --- | --- | --- |
| 1 · the language | the `acquisition-search` crate (C89's edges in `tools/docs-check.sh` §5, each with a breaker; C47's lints); the tree and its JSON form; the parser over the whole query language; lowering; the canonical printer; every error the grammar defines, with its readings | the round-trip test; the corpus table; the holes table |
| 2 · the derivation | a pure function, body and ingest columns in, the item out (C103): fields, displayed strings, lines as (kind, template, numbers) with slots and the ranged rule, and what could not be read, by collection | fixture tests; M2 against the census |
| 3 · the store's read | C103's read, the revision, the coverage rows — **shown to the owner first** (the showing: this file at `2d25cb03`; the read as built: `corpus.rs`'s doc, C108) | store tests, `REFRESH-SLICE.md`'s findings table as the checklist; the one-snapshot test; M1; C103's second decision given its own entry (C108) |
| 4 · the first surface | the binder (names, near names, closed sets); three-valued evaluation with witnesses (C93); occurrence binding, the sort scalar, the together count (C92); the answer (C100): query, scope, basis, terms, total, rows, the zero-total block, a route per term count; `--describe`; `--json`; the verbs `acq search` and `acq show <id>` (the item as the deriver sees it: fields, place, every line with its kind, template and numbers, what was unread; `--json` the same, structured; the stored body on request by a flag, which still works when deriving fails — the reference's `acq show`), their README tour lines, `CLI-REFERENCE.md` regenerated | the worked example reduced to what is built, hand-counted; the acceptance rows marked 4; the refusal walk; the route property; M3, M4 |
| 4b · the properties | what four audits of step 4 found by generating, put in the gate as properties; then a group's meaning computed once, so that no second reading of it can disagree (its brief: this file at `2d25cb03`) — no new surface and no rule changed | the three properties green in the gate, each shown able to fail; the structural rule with its breaker; step 4's regression tests untouched and green; M3 again; one more outside audit, its yield reported beside 6, 5, 2, 3 |
| 5 · counts and the vocabulary | `--count`, `--cross`, `--sum` (C95); `none` and `undecided` buckets with routes (C105); `--count line[:text,…]` (C97) | AQ1; C105's two invariants pinned on a one-value key; C95's sum over a fixture of three kinds — an item with the value, one lacking it (adds nothing, counted as lacking), one whose value is unread (the subtotal marked incomplete, never a total); the vocabulary's pasted term selects its row; M3 again; then the seat |

**The reference was brought level with what was ruled here before step
1** (2026-09-19, the owner approving the wording), since step 1's corpus
is complete against that page and this file is deleted at the close. A
hole ruled keeps its evidence in `SEARCH-SLICE.md` and its rule in
`search/DESIGN.md`.

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

All on a `.backup` copy, recorded in `SEARCH-SLICE.md` with the command
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

## Holes not yet ruled — the owner's, at the seat

Each changes what a user types or pastes, or which items it matches, and
a recommendation is no ruling (the record: step 4, audit 1, finding 8).
The ruled holes, with their evidence, are in `SEARCH-SLICE.md`; the rule
of each is in `search/DESIGN.md`. Gaps 2 and 3 are from the planning
review; their numbers are cited as such.

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

**Step 5's — each changes what a user types or pastes.** None blocked
the step: each is built in the direction that breaks least if he rules
the other way.

| # | Hole | Built as | Recommendation |
| --- | --- | --- | --- |
| E1 | What a value bucket's term is. The reference gives `none` and `undecided` theirs (C105) and says a crossed cell "carries both keys"; nothing says a value's | `<key>=<value>`; where the corpus holds another spelling of the value, `<key>~"(?-i)^…$"` — the reference's own form for a template, applied to every text; a `tab` bucket is the tab, routed by `id:<its id>`, since a name is no identity (two leagues' `Dump`, a substash named as a tab) and `tab=` would return what other buckets counted | keep |
| E2 | What `--sum` takes. The reference: `[--sum value]`, and `line(P).<slot>` is a value — for `--sort`, the largest satisfying occurrence. Summed over items it would add each item's largest, and `--sum '"# to maximum Life"'` would give 75 for an item with 20 and 75 | a field or `sum( … )`; a projection is an error offering `sum(line(P).<slot>)`, so the item's total is what a count adds | keep: allowing it later breaks nothing; a sum of largests, if wanted, is a value with a name of its own |
| E3 | The vocabulary's `undecided` bucket. C105 puts an item whose key "cannot be established" there; `line` has many values on an item, and the language has no term for "its lines could not all be read" | the bucket is `undecided(line(<narrowing>))`: no selected line read, and a part unread that could hold one. An item with a line read and an array unread is in its rows and not there — its row's count says how many items are known to carry the template, never how many might | keep; a term for the other question is a hole of the language, if a seat asks it |
| E4 | `--sort` beside `--count`, `--sum` alone, `--count` beside `--cross`. The synopsis puts the view on one line and `--sort`, `--limit` on the next | each is a view error saying which flag goes with which; `--limit` bounds a table as it bounds rows | keep |
| E5 | `line` in a crossed table (`--cross tab,line`) | refused: a crossed table takes two fields; the vocabulary is its own table | keep until a seat asks for one; adding it later breaks nothing |
