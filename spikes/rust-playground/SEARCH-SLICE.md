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
measurements it repeated and its observations here — and never during: until then the commit message is the journal
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
| **A meaning read off the syntax.** Which sources a group admits, what its selector picks, where its together bound sits, which slot a template has, what the zero block reads — each answered at the group's own level and wrong one level down, so parentheses changed the answer | five times across step 4's four audits | `group.rs`, the one reader of a group's meaning; `tools/docs-check.sh` §7; the equivalence property (rule 9) |
| **Unknown said at the wrong grain.** A whole object unread for one flag that is no boolean, a whole array for one number too long, a whole requirements list for one bad row: too wide leaves open terms that never needed it, too narrow reads as absence, which a not makes a witness of | steps 4, 4b, 6, 7, 8 | rule 8; `derive::Slot`, `Line::flags_unknown`, `Unread::line`, `Unread::name`; `sockets::Groups`; the completion property |
| **Two statuses for one thing.** A number kept beside the unread that may replace it, so a comparison, the sort and the sum disagreed; a total exact on the item and rounded again in the bucket | steps 5, 6 | rule 8 (`reqlevel`'s one status); `exact::Exact`, units carried to the print |
| **A second maker.** A route, a printed command, a reason, a selector's resolved list, each made in two places that then disagreed — a lacked count of 1 whose route returned 0, a `show` continuation that dropped the account | steps 4, 4b, 5 | rule 10; `answer::command`, `answer::Router`, `eval::why`; the CLI test that runs every printed command with a second account known |
| **A printed command the build refuses.** A route starting with `-` at a shell, a reading that does not parse (`class=Body Armours`), a slot word the same build's slot check refuses, a continuation the key parser could not read back | steps 4, 5, 6 | rule 5; the route property; every offered reading bound by a test; `bind::closed` offering through the printer |
| **Arithmetic on floats.** A sum whose value depended on the order of its occurrences; a fallback whose use depended on how the integers cancelled; units read back off a float that does not print its decimal | steps 4b, 5 | `exact.rs`: a number read once, within a measured rule, and whole units from then on |
| **A cut in the query's order.** A reason made beyond the item's parts took its place from the term that met it first, so two spellings of one query showed different sixes | step 7 | rule 9; `eval::why` orders such a reason by what it says; the audit test in `tests/pseudo.rs` asks two spellings |
| **A block past its bound.** Rows appended after the cut — the vocabulary's computed values, 39 under a limit of 1 — with the omission uncounted | step 7 | invariant 5; every list an answer holds is cut by the limit and counts its rest, a new kind of row with its own count; the audit test |
| **A number no game displays, read as one.** Scientific notation through a length check; a product past the units rounded in silence | step 7 | `exact::reads` reads decimal syntax alone; `Exact::times` is none where the units cannot hold it; the input is unread to what asked it |
| **A join inherits the producer's grain.** A field the other area's read typed strictly — a note, a slot — failed that read whole on one malformed body, and the join made every such failure the search's, for queries that never asked the field; an override in the producer decided without the gate its own rule states (a note where no index sees it); a number from the intent file bypassed the crate's rule for numbers and two prices met in one bucket; coverage was derived from the locations that name a league, and a character with none went unpriced | step 9, the first outside review: four of five | rule 8 at the producer's grain (`ItemSnapshot::note_unread`, `inventory_id_unread`; `body_string`); C81's own gate (`game.public`) on the override; `exact::reads` on every number that enters (`price::number`); `price::leagues` |
| **A claim the code did not make.** A hand count wrong; `DERIVATION` not moved when a body derived to another item; a cause named before it was measured; "covered" said of a property whose generators could not reach the case; a record row crediting the wrong commit; a commit message claiming what a failed step had not shown | every step | every count worked by hand and then run; the constant's rule on its own doc; a number stated only after measuring; the generators reaching what a fix touched (`reqlevel` joined them at step 6); a command that commits stops at the first error, each check's exit read first |
| **Silent reach.** Right by its rule and wrong for the reader: an open-text `:` example reaching 87 bases with five shown (`base:ring`); two realms' place values merged under `--realm all`, one `Standard` whose route returned both | the first seat, twice (the examples; F3) | `tests/seat_faults.rs` (`f3_`); the help's examples teach `class:ring`, a closed set whose picks are all printed |
| **A refusal right in kind, wrong in size.** 82 names inline, twice, the meant one not singled out; one reason printed once per term, eighteen times; a near reading for a field and none for a computed value | the first seat, three times (F5, F9, F10) | `tests/seat_faults.rs` (`f5_`, `v8_`, `f10_`); `bind::near`, `bind::LISTED_INLINE`; `Total.undecided_reasons` |

## What the measurements taught

Each measurement's tables, commands and numbers are
`search/MEASUREMENTS.md`, read by the block named and never whole; a
number there was measured, never recalled. The verdicts:

- M1 (step 3): the streaming read is 36 ms release and 8.7 MB resident
  over 22,721 items and 29.9 MB of bodies.
- M2 (step 2, rerun at every change of the deriver): 0 unread and 0
  unexplained against the census; seven departures, each a rule of
  `derive.rs`, two of them candidate ground-truth claims.
- M3 (step 4, rerun at every build since; as of `c28c476a`): the load,
  the empty query, is 457 ms release, under 500, so the projection park
  does not fire; the price join raised it from 276 at step 9. Seven
  asks are over 500, each by an evaluator's cost, the totals batch
  park's (V9, revisited at step 11): an ask pays 5 to 6 ms for each row
  of every total it reads, a least stopping at the first total that is
  nothing. The six asks the seat saw at 695–715 ms did not recur on the
  copy, their cause unmeasured. The debug build is 2.4 to 3.8 s. What
  each rerun showed is the block's.
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

## Observations still open

The builder's observations that became neither a ruling nor a finding;
each is data for the step or the seat that touches it. What described a
mechanism the crate's module docs carry, and what a later fix made
history, was taken out on a read against those docs (the commit that
did so lists them); what stays is measurements of the owner's copy,
timings, coverage no fixture reaches, and questions for the seat.

An observation written since 9c names the step or the trigger that
will use it, and that step's close removes it or brings it to a
ruling. One written before is filed by the step that made it, and is
filed again by the step that touches it.

**Step 1 — the builder's.**

- Two quoted templates in one and-group is valid and matches nothing.
**Step 2.**

- Nothing was unread on this corpus, so C93's unread path is exercised
  by fixtures alone: a wrong type under a known key, an element that is
  no line, a `displayMode` outside 0–4, a body that is not an object.
**Step 3.**

- On this copy no character item's stamped league differs from its
  character's listing league, and no character is league-less: the
  league join is exercised by fixtures alone.
- 6 of 22,727 item rows are removed, all at live characters; no tab or
  character has been retired yet, so an item removed at a retired
  location exists in fixtures alone.
- 2,926 of 3,427 live tabs have never been fetched, the 17 folders
  apart, so "absent from a full refresh" is not a condition this
  account's store can often state.
- `Folder` is a word four consumers know — the planner, pricing, the
  CLI's counts and search — each comparing GGG's `type` for itself, and
  the store's read hands the type over verbatim as they do; the store
  saying it once is a small change if a fifth arrives or one of them
  gets it wrong.

**Step 4.**

- The scope says `in all leagues`; a league is a term, and the
  default-league park's trigger is the GUI's (2026-09-25), not a step's.
- What B3 rests on, measured over the copy when the owner asked what
  `tab:maps` meant (2026-09-20): 15 map tabs hold 2,566 substashes and 29
  unique tabs 491, and a substash is named `1`, `4 (Remove-only)` or
  nothing, so without its tab's name an item in one could not be found by
  tab at all. `tab:` tests names and never GGG's `type`, of which the copy
  has 16 — `tab.type` since 9b.

**Step 4b.**

- The gate's cost, a debug build here: equivalence 14 s, the cross-checks
  15 s, completion 3 to 6 s, beside the routes' 16 s. 256, 192 and 256
  cases; `PROPTEST_CASES` runs any of them longer by hand.
- The basis names no evaluator, so two builds can label different
  answers with one basis — true of every evaluator fix since step 4;
  `DERIVATION` moves only when a body derives to another item.

**Step 6.**

- The C++ app's category (cpp-search F4) is this same vocabulary, the
  RePoE class display name lowercased, so the owner's old dropdown and
  `--describe class` say the same words.

**Step 5.**

- Over the copy no value outside a closed list — a rarity, a source, a
  flag GGG adds — occurs, so the counted-with-no-route path is exercised
  by fixtures alone.
- A route's spelling for a bucket under the empty query is the term alone,
  never `() term`: the router folds an empty root away, which is the one
  simplification a generated tree makes (as the selector's folding is).

**Step 7.**

- Over the copy nothing is unread and every item is in a covered realm
  but poe2's 98, so a total's incomplete subtotal is exercised by
  fixtures alone; the unavailable status by the copy's poe2 items.
- A derived field's lacked item sorts last with the status `no
  satisfying occurrence`, which is a line's wording; a field's and a
  computed value's is the same status today.
- The generated properties reach `pseudo` since the third look and
  `price` since step 9, and still no `class`.
- `pseudo.dps` counts every damage property the item displays and
  `pseudo.pdps` the physical alone; the C++ app reads 0 where a property
  is missing and this build says *lacked* — the one departure, C93's.

**Step 8.**

- On a socket whose colour is unread `linked(red=1 or red=0)` is
  undecided: each comparison is read over the interval on its own. The
  seat did not reach it.
- A poe2 socket's `type`, a gem's own `colour` and `socket`: kept, asked
  by nothing.

**Step 9.**

- The generators reached the pricing area before any seat did — a body
  whose note is no string failed its snapshot whole — and the review
  found four more of that shape (the findings above; `PRICING-SLICE.md`).
  Over the copy no note is one, so the path is fixtures' alone.
- The join is consistent by a check, not a snapshot (`corpus.rs`); no
  consumer has met `basis_moved`. The intent revision on the basis is
  the whole file's, so a sync-policy write reloads a held corpus too;
  no consumer holds one across asks yet.
- *Uncovered* — a league-less character in a realm with no league on
  record — is a reason no fixture reaches now; a price whose amount or
  lot is past the crate's rule for numbers (ten whole digits, four
  decimals) is priced with its number unread, and the seat may meet a
  bulk lot written that long.

**The first seat.**

- The six lines, each kept (C91, C92, C93 with V8's display, C104 by
  V7). Still open of them: C95, a bucket whose every item lacks the
  value prints `sum 0 · N lacking`, the 0 reading as measured; C98, the
  basis repeated on every count in `--json` where it equals the
  answer's, and no evaluator version in it.
- `class:sword` and `class:map` reach past their word too, but print
  every pick; an open field's `:` prints five of 87 (the checklist's
  silent reach).
- After 9b every `class:` term carries 158 undecided, the captured
  beasts (70 bases), until category lands (9d); `undecided(class)` is
  the same 158 since G2's revision.
- `show`: an unnamed substash renders as a trailing separator; `price
  none (none)`; explicit lines before implicit, where the game shows
  the implicit first. A row prints `price.side`, which `--describe`
  lists as no field.
- A zero answer counts and routes every term; what was wanted first was
  which terms together empty it — `--explain`, step 10.
- `--json` for one row is 12,670 bytes: the scope's seven leagues where
  the question touched two, `"Rare"` where the language writes `rare`.
- 9b: the digest's S53 wording ("not a field") was overtaken by step 6;
  the help says the derivation, and the digest, accepted, is not edited.
- The owner's questions to the sitter, answered as evidence, no ruling:
  category — grouping bit once in 52 asks (OQ5's 18 terms), undecided
  noise five times, `base:ring` once; complementary to class, computed
  from it in one place (rule 10), never a second classifier. The
  variant field — three in ten uniques would answer undecided or
  lacking (the legacy track's numbers), the fit's rules in Python only,
  the per-ask cost unmeasured; kept parked (V4). `=` against `:` on
  class (V3): keep `:`, taught in the examples since 9b.

**Since 9c, by who uses it.**

- *The next sitting:* the four readings were asked one way, what
  shows the pseudo; whether an item showing every total a reading
  names ever shows no pseudo was not asked, nor the count of all four
  of an item with no resistance, nor the elemental least of an item
  with one or two of its totals. Each is explained, never closed.
- *9c4, and the review's next look:* three choices of 9c3 were the
  builder's (`c28c476a`). The first look weighed two, a count and a
  least where a total is open, and found no contradiction; the third,
  the definitions' home in the totals table, is unruled. What the
  builder doubted and no look has asked: what `--sum` and the undecided
  block print of a reading; `tests/generated_routes.rs` reaches no
  computed value; a narrowing lists every reading whose definition it
  matches, `resist` three; `has:` of a ranged total asks its `avg`,
  untested until 9d; `pseudo::evidence` of a reading that is lacked
  gives its totals still, reached by nothing.
- *9d:* the two leech pseudos count the line under every id, the
  `(Local)` twin among them, so neither waits for a row that names what
  the item is; of 9c2's totals none does.
- *The trade translation (C99):* what the search counts and the site
  leaves out is said in the table's changes and nowhere a user reads.
- *A fix on the site:* the owner reported the twin id to GGG
  (2026-09-26); it is left out of total Strength, Intelligence and mana
  as of total life, and a fix moves four `not mimicked` rows to
  agreement.
