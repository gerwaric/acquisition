# The item search slice — closed record

The item search slice — the `acquisition-search` crate: the language,
the derivation, the store's read, the first surface, its properties and
the counts view, the owner's seat and the steps after it, the search on
the MCP and the reference in its homes — was built between 2026-09-19
and 2026-10-10 and is closed. This file is its permanent short record in
the mold of `PRICING-SLICE.md`: what landed, what the audits found and
what holds each finding now, what the measurements taught, and what is
still open. It was opened at step 5 rather than at the close (the
owner's ruling, 2026-09-21). The brief it was built under is
`search/BUILD-PLAN.md` at `453a4429`, deleted at the close: its
acceptance set, the parks it fired and the measurements it owed are
there, and its rules are below. The full text as it stood in the plan —
every audit round in its own section, the step 4b brief, the first
showing of the store's read — is the plan at `2d25cb03`.

The rulings are `decisions/search.md` (C89–C107) and `decisions/store.md`
(C108); the language is `acq search --describe` and the contract
detail the crate's module docs; the acceptance set and the limits register are
`search/DIGEST.md`; the properties are pinned by the tests named below.
Nothing here is a second authority.

Reviewing a search change: the rules and the shapes of fault below are
the checklist, and `REFRESH-SLICE.md`'s findings table beside them for
the store's read.

## Final state

- Tip at closure: the closing commit. Facts **v8**, derivation **11**,
  totals table **v6**, class, category, colour and defence tables **v1**
  each; gate green.
- The search code: the `acquisition-search` crate and its
  `reference/` tables; the store's `corpus.rs` (C103, C108); the CLI's
  `search_cmd.rs` (`acq search`, `acq show`); the MCP's `search`,
  `describe` and `show`; the table makers `tools/class-table.py`,
  `totals-table.py`, `category-table.py`, `defence-table.py`, and
  `tools/trade-sheet.py` for a sitting.
- Not built, each refused by name and printed by `--describe`:
  `--fields`, `--view locations`, `--rebind`, `show --basis`. Not in the
  slice: the trade translation (C99), parked.
- What stays under `search/` is what the build names
  (`search/README.md`); a pointer that finds no file is read at
  `40b55e79`.

## What is looked up

Three kinds of row are looked up by the row, never loaded whole:
`search/LEDGER.md`.

- **Step ledger** — a closed step is a line: the step, its commits,
  where what landed lives.
- **Holes ruled, and where the rule went** — a line per hole, by its
  id: the hole as ruled, where the rule is, the commit that ruled it.
  A crate doc or a test that names a hole names its row.
- **How the design was reached** — the synthesis's stages and their
  notes.

## The plan's rules

How a partial build stayed honest: the brief's rules, moved here whole
when the brief was deleted, since the crate's docs and tests cite them
by number ("rule 8 of the plan"). Each is in the build's tense; "the
reference" is the page the module docs and `--describe` are now
(`search/DESIGN.md` at `0d71054f`), and rule 4's two pointers read as
they do since the close. They bind a change to the crate as they bound
a step.

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
   the owner if it changes what a user types; otherwise it is a park
   with its trigger (`decisions/search.md`). A name the reference hands to the
   builder — a flag, a key, a field's spelling — is not a gap: the
   builder picks it and `--describe` or the help prints it. What a name
   *means* — which items it matches — is. A ruled one is a line in
   `search/LEDGER.md`, "Holes ruled", its evidence the ruling commit's
   message; none was unruled at the close.
5. **An answer never prints a command the same build refuses.** A
   block the reference bounds with a route to the whole (invariant 5)
   prints that route only when it runs; until then it prints the count
   left out and names the unbuilt construct. The route property's test
   covers it: every command an answer prints parses and is not refused.
6. **Spent at step 11e.** The language reference stayed whole in
   `search/DESIGN.md` until the first seat had ruled, and is the module
   docs, the clap help and `--describe` now; the rule as it stood is
   this file at `0d71054f`.
7. **Whose store.** The agent tests on fixtures built through the
   store's own ingest, and measures on a sqlite `.backup` copy under a
   track's `raw/`, always with `ACQ_STORE_DIR` set. The owner's store is
   opened only by the owner, at a terminal. No GGG traffic anywhere in
   this plan; the one daemon is M5's, a mock session.

Rules 8 to 10 are what step 4b's audits taught (`SEARCH-SLICE.md`,
"Findings"), each a fault that came back until it was named;
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

## Findings

Every step was reviewed from outside — three to five looks at each,
every finding reproduced by the builder as a failing test before it was
taken — and a finding a test holds is held there and nowhere else (P5):
the test names the fault, the commit that fixed it tells the story, and
`git log` over the step's range is the ledger of rounds. Every finding
of steps 1 to 7 is held by a test, by a measurement below, or by a
ruled hole; the full table is this file at `aeeba6d3`, and step 7's
two rounds at `e0420428`. What stays is the checklist a review reads
first: the shapes of fault that came back until they were named.

| Shape | Where it came back | Held by |
| --- | --- | --- |
| **A meaning read off the syntax.** Which sources a group admits, what its selector picks, where its together bound sits, which slot a template has, what the zero block reads — each answered at the group's own level and wrong one level down, so parentheses changed the answer; and which part of a mod a slot reads, answered once for the group from every template in it, so an alternative beside a comparison changed its number — then, mended by pushing every not down to the tests, answered apart from the template a not had held it with, so a group and its not both held; then, two rows being named together, answered by the mod's numbers in order, so a row named as a condition moved the number beside it, while the slot check held every slot against the group's one quoted template | five times across step 4's four audits; four times at 9c4, in three looks | `group.rs`, the one reader of a group's meaning, `Slots` what a slot is read beside (L3); `tools/docs-check.sh` §7; the equivalence property (rule 9), which asks a fourth way since: an alternative that holds nowhere beside every group; two laws asked of any generated group (`tests/generated_rows.rs`): a group or its not holds, and a group's or is the item's |
| **One thing counted under two names, and two taken for one.** The mod's first number and its first row's first, two parts read by two alternatives, added twice to a sum and to the together count; then, a place being added once, a number that is either of two rows' given the first row's place and dropped for the number read there | 9c4's review, the second look and the third; the builder had seen the first case and let it pass | `derive::Place`, a number known by where it sits in its mod, and none where it sits in no one place; `eval::Read`; `tests/rows.rs` (`c95_…`, `c93_a_number_that_is_either_…`), the rows-apart oracle |
| **Unknown said at the wrong grain.** A whole object unread for one flag that is no boolean, a whole array for one number too long, a whole requirements list for one bad row: too wide leaves open terms that never needed it, too narrow reads as absence, which a not makes a witness of | steps 4, 4b, 6, 7, 8, 9d-B | rule 8; `derive::Slot`, `Line::flags_unknown`, `Unread::line`, `Unread::name`; `sockets::Groups`; the completion property |
| **Two statuses for one thing.** A number kept beside the unread that may replace it, so a comparison, the sort and the sum disagreed; a total exact on the item and rounded again in the bucket | steps 5, 6, 9d-B | rule 8 (`reqlevel`'s one status; `class::unread`, a class reason only where the class has no name); `exact::Exact`, units carried to the print |
| **A second maker.** A route, a printed command, a reason, a selector's resolved list, each made in two places that then disagreed — a lacked count of 1 whose route returned 0, a `show` continuation that dropped the account | steps 4, 4b, 5, 9d-B, 10b, 10c, 11b (a realm offered on a refusal, the one request printed without its account) | rule 10; `answer::command`, `answer::Router`, `eval::why`; the CLI test that runs every printed command with a second account known; one opening for every command (`answer.rs`), `show`'s offer made by `Request::command` |
| **A printed command the build refuses.** A route starting with `-` at a shell, a reading that does not parse (`class=Body Armours`), a slot word the same build's slot check refuses, a continuation the key parser could not read back; a ranged total's slot word carried into its lacked route, where `has:` takes none | steps 4, 5, 6, 9e, 10a, 10b | rule 5; the route property; every offered reading bound by a test; `bind::closed` offering through the printer |
| **Arithmetic on floats.** A sum whose value depended on the order of its occurrences; a fallback whose use depended on how the integers cancelled; units read back off a float that does not print its decimal | steps 4b, 5 | `exact.rs`: a number read once, within a measured rule, and whole units from then on |
| **A cut in the query's order.** A reason made beyond the item's parts took its place from the term that met it first, so two spellings of one query showed different sixes | step 7 | rule 9; `eval::why` orders such a reason by what it says; the audit test in `tests/pseudo.rs` asks two spellings |
| **A block past its bound.** Rows appended after the cut — the vocabulary's computed values, 39 under a limit of 1 — with the omission uncounted | step 7 | invariant 5; every list an answer holds is cut by the limit and counts its rest, a new kind of row with its own count; the audit test |
| **A number no game displays, read as one.** Scientific notation through a length check; a product past the units rounded in silence | step 7 | `exact::reads` reads decimal syntax alone; `Exact::times` is none where the units cannot hold it; the input is unread to what asked it |
| **A join inherits the producer's grain.** A field the other area's read typed strictly — a note, a slot — failed that read whole on one malformed body, and the join made every such failure the search's, for queries that never asked the field; an override in the producer decided without the gate its own rule states (a note where no index sees it); a number from the intent file bypassed the crate's rule for numbers and two prices met in one bucket; coverage was derived from the locations that name a league, and a character with none went unpriced | step 9, the first outside review: four of five | rule 8 at the producer's grain (`ItemSnapshot::note_unread`, `inventory_id_unread`; `body_string`); C81's own gate (`game.public`) on the override; `exact::reads` on every number that enters (`price::number`); `price::leagues` |
| **A claim the code did not make.** A hand count wrong; `DERIVATION` not moved when a body derived to another item; a cause named before it was measured; "covered" said of a property whose generators could not reach the case; a record row crediting the wrong commit; a commit message claiming what a failed step had not shown; a module doc saying a group costs its size after a fix had made a not cost what it held again at every depth; a fixture that wrote its capture's input another way, a mod's rows apart, and agreement with the research tool reported as the build's; the instructions telling an agent to begin with a call the same build refuses over two realms, after 11b's look had found five sentences of the tool texts saying more than the build | every step; the MCP's texts at 11b and 11c | every count worked by hand and then run; the constant's rule on its own doc; a number stated only after measuring; the generators reaching what a fix touched (`reqlevel` joined them at step 6, a mod of several rows at 9c4); a command that commits stops at the first error, each check's exit read first; a capture entered as captured and asked through the boundary (`tests/pseudo.rs`, `c94_a_row_inside_…`); a cost claimed is a cost asked for, thirty nots deep (`tests/rows.rs`); a step a tool's text gives is run over a store that refuses it (the MCP's `no_arguments_over_two_realms_…`) |
| **Silent reach.** Right by its rule and wrong for the reader: an open-text `:` example reaching 87 bases with five shown (`base:ring`); two realms' place values merged under `--realm all`, one `Standard` whose route returned both | the first seat, twice (the examples; F3); 9d-B, a category term's picks cut at five in text | `tests/seat_faults.rs` (`f3_`); the help's examples teach `class:ring`, a closed set whose picks are all printed |
| **A refusal right in kind, wrong in size.** 82 names inline, twice, the meant one not singled out; one reason printed once per term, eighteen times; a near reading for a field and none for a computed value | the first seat, three times (F5, F9, F10) | `tests/seat_faults.rs` (`f5_`, `v8_`, `f10_`); `bind::near`, `bind::LISTED_INLINE`; `Total.undecided_reasons` |
| **A patch for a body composed to probe.** A look's finding on an input no real item has — a mark that is no yes or no, the quality enchant on a crucible node — mended as a case of its own; each case covered itself and opened its neighbour, four faults of the round's own making | 9d-A, the first look's round, found by the second | the probe measured on the owner's copy before a fix; one rule in place of the cases (`pseudo::mark_of`); `tests/percentile.rs` |

## What the measurements taught

Each measurement's tables, commands and numbers are
`search/MEASUREMENTS.md`, read by the block named and never whole; a
number there was measured, never recalled. The verdicts:

- M1 (step 3): the streaming read is 36 ms release and 8.7 MB resident
  over 22,721 items and 29.9 MB of bodies.
- M2 (step 2, rerun at every change of the deriver): 0 unread and 0
  unexplained against the census; seven departures, each a rule of
  `derive.rs`, two of them candidate ground-truth claims.
- M3 (step 4, rerun at every build since; as of `3eae4b4e`): the load,
  the empty query, is 478 ms release, under 500, so the projection park
  does not fire; the price join raised it from 276 at step 9, a mod's
  rows by 3 to 5 at 9c4, category by some 14 at 9d-B, and the table's
  135 more rows by some 9 at 9d-C, bound at every ask; at `a9733264`
  it is 481, the percentile's asks 486–489. Thirteen asks
  are over 500 in the run and nine were at `dc262f71`, each by an
  evaluator's cost, the totals batch park's (V9, revisited at step 11):
  an ask pays some 1 ms for each row of every total it reads. The
  owner, 2026-10-08: the slice is wrapped up before anything is
  optimized, and the budget was never a hard number. At step 10
  (`5a087d37`) the load is 492: the live read names the ids it left
  out by a scan of `items`, some 9 ms; explain is two more
  evaluations, 522; a continuation and `all` cost nothing more. The whole run at the
  tip (`e5e32adb`): the load 488–492, and on that floor 36 of its 67
  asks are over 500 where 18 of 60 were, the dearest Thread of Hope's
  counts at 623; six asked again alone are 3 to 18 over `a9733264`'s,
  the load 11 or 12, each within 8 of the load's own move. The category
  asks cost no more than the empty query. The six asks the seat saw at
  695–715 ms did not recur on the copy, their cause unmeasured. The
  debug build is 2.7 to 4.6 s. What each rerun showed is the block's.
- M4 (step 4): `~` over all text adds 9 ms.
- M5 (step 10): over a mock session, 126 asks answered across 18
  responses committed, none refused and none slower than idle; a
  resident consumer pays one whole load for each committed tab, 492 ms
  on the copy, which is the incremental-update park's.
- M7 (step 10): an ask that holds the copy's corpus peaks at 214 MB
  resident, some 9.5 KB an item, where the streaming read alone was
  8.7; the steady figure was not measured apart.
- The class table (step 6): 56 of 82 classes carried, 681 of 22,721
  undecided, the buckets summing to the copy (C105).
- M6 (step 7, the park fired): every row of the totals table carried by
  the corpus, the rows shipped unchanged; what no total counts was
  settled at 9c (V6).
- The completion property's measured half: no counterexample, at step
  4b and over step 8's generators.
- The vocabulary against M2 (step 5): 6,181 rows and 6,148 templates
  agree.
- Step 8: every socket bucket a value or `none`, `undecided` 0; every
  group on the copy one contiguous run.
- Category (9d-B): the count by category is
  `search/category/data/expected-copy.csv` on every id — 22,622 of the
  copy's 22,623 pc items under an id, 1 `none`, 0 undecided — and
  `undecided(class)` is 0 where it was 158, the beasts.
- The percentile (9d-A): on the copy 2,249 items have it, 20,373
  lack it and 99 are undecided — 98 by the realm, 1 by a crucible
  line — `search/pseudo-stats/data/percentile-expected-copy.csv`'s;
  none reads over 100 or below 0.

## Observations still open

What a step saw that became neither a ruling nor a finding was
`search/OBSERVATIONS.md`, whole at `453a4429`. At the close each entry
with a trigger became a park (`decisions/search.md`,
`decisions/frontends.md`, `decisions/store.md`), each with a home was
left to it, and the rest were dropped. One kind stays a pointer: the
entries headed "the review's next look" — what no test pins and no look
reached, a step's own choices no look weighed, the paths fixtures alone
exercise — are read there, by the module named, before that module is
changed.
