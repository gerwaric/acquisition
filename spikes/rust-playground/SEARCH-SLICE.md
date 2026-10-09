# The item search slice — the record

The item search slice — the `acquisition-search` crate: the language,
the derivation, the store's read, the first surface, its properties and
the counts view, then the owner's seat and the steps after it — is being
built under `search/BUILD-PLAN.md`, the brief. This file is its record,
opened at step 5 rather than at the close (the owner's ruling, in the
plan's header), in the mold of `PRICING-SLICE.md`: what landed, in which
commits, what the audits found and what holds each finding now, what the
measurements taught, which holes were ruled and where the rule went, and
what is still open. The full text as it stood in the plan — every audit
round in its own section, the step 4b brief, the first showing of the
store's read — is the plan at `2d25cb03`.

The rulings are `decisions/search.md` (C89–C107) and `decisions/store.md`
(C108); the language reference and the contract detail are
`search/DESIGN.md`; the acceptance set and the limits register are
`search/DIGEST.md`; the properties are pinned by the tests named below.
Nothing here is a second authority. A step writes once, at its
close — its ledger line and its holes to `search/LEDGER.md`, the
measurements it repeated here, its observations to
`search/OBSERVATIONS.md` — and never during: until then the commit message is the journal
(2026-09-23, after the record had been kept as one).

Reviewing a search change: the shapes of fault below are the
checklist, and `REFRESH-SLICE.md`'s findings table beside them for the
store's read.

## What is looked up

Four kinds of row grow by a line at every step's close and are looked
up by the row, never loaded whole: `search/LEDGER.md`.

- **Step ledger** — a closed step is a line: the step, its commits,
  where what landed lives.
- **The yields by look** — what a review found each time it came back,
  per step, and what the seats found.
- **Holes ruled, and where the rule went** — a line per hole, by its
  id: the hole as ruled, where the rule is, the commit that ruled it.
  Looked up before a hole is brought to the owner.
- **How the design was reached** — the synthesis's stages and their
  notes.

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
| **A second maker.** A route, a printed command, a reason, a selector's resolved list, each made in two places that then disagreed — a lacked count of 1 whose route returned 0, a `show` continuation that dropped the account | steps 4, 4b, 5, 9d-B | rule 10; `answer::command`, `answer::Router`, `eval::why`; the CLI test that runs every printed command with a second account known |
| **A printed command the build refuses.** A route starting with `-` at a shell, a reading that does not parse (`class=Body Armours`), a slot word the same build's slot check refuses, a continuation the key parser could not read back; a ranged total's slot word carried into its lacked route, where `has:` takes none | steps 4, 5, 6, 9e | rule 5; the route property; every offered reading bound by a test; `bind::closed` offering through the printer |
| **Arithmetic on floats.** A sum whose value depended on the order of its occurrences; a fallback whose use depended on how the integers cancelled; units read back off a float that does not print its decimal | steps 4b, 5 | `exact.rs`: a number read once, within a measured rule, and whole units from then on |
| **A cut in the query's order.** A reason made beyond the item's parts took its place from the term that met it first, so two spellings of one query showed different sixes | step 7 | rule 9; `eval::why` orders such a reason by what it says; the audit test in `tests/pseudo.rs` asks two spellings |
| **A block past its bound.** Rows appended after the cut — the vocabulary's computed values, 39 under a limit of 1 — with the omission uncounted | step 7 | invariant 5; every list an answer holds is cut by the limit and counts its rest, a new kind of row with its own count; the audit test |
| **A number no game displays, read as one.** Scientific notation through a length check; a product past the units rounded in silence | step 7 | `exact::reads` reads decimal syntax alone; `Exact::times` is none where the units cannot hold it; the input is unread to what asked it |
| **A join inherits the producer's grain.** A field the other area's read typed strictly — a note, a slot — failed that read whole on one malformed body, and the join made every such failure the search's, for queries that never asked the field; an override in the producer decided without the gate its own rule states (a note where no index sees it); a number from the intent file bypassed the crate's rule for numbers and two prices met in one bucket; coverage was derived from the locations that name a league, and a character with none went unpriced | step 9, the first outside review: four of five | rule 8 at the producer's grain (`ItemSnapshot::note_unread`, `inventory_id_unread`; `body_string`); C81's own gate (`game.public`) on the override; `exact::reads` on every number that enters (`price::number`); `price::leagues` |
| **A claim the code did not make.** A hand count wrong; `DERIVATION` not moved when a body derived to another item; a cause named before it was measured; "covered" said of a property whose generators could not reach the case; a record row crediting the wrong commit; a commit message claiming what a failed step had not shown; a module doc saying a group costs its size after a fix had made a not cost what it held again at every depth; a fixture that wrote its capture's input another way, a mod's rows apart, and agreement with the research tool reported as the build's | every step | every count worked by hand and then run; the constant's rule on its own doc; a number stated only after measuring; the generators reaching what a fix touched (`reqlevel` joined them at step 6, a mod of several rows at 9c4); a command that commits stops at the first error, each check's exit read first; a capture entered as captured and asked through the boundary (`tests/pseudo.rs`, `c94_a_row_inside_…`); a cost claimed is a cost asked for, thirty nots deep (`tests/rows.rs`) |
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
  optimized, and the budget was never a hard number. The category
  asks cost no more than the empty query. The six asks the seat saw at
  695–715 ms did not recur on the copy, their cause unmeasured. The
  debug build is 2.6 to 3.9 s. What each rerun showed is the block's.
- M4 (step 4): `~` over all text adds 9 ms.
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

What a step saw that became neither a ruling nor a finding is
`search/OBSERVATIONS.md`, looked up by the step or the trigger that
will use it and never loaded whole: a step's close writes its own
there and removes or rules each one it used.
