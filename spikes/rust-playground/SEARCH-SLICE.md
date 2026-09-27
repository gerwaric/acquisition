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
Nothing here is a second authority. A step writes here once, at its
close — its ledger line, its holes, the measurements it repeated — and
never during: until then the commit message is the journal
(2026-09-23, after the record had been kept as one).

Reviewing a search change: the shapes of fault below are the
checklist, and `REFRESH-SLICE.md`'s findings table beside them for the
store's read.

## How the design was reached

Moved from `search/README.md` when the record opened; the tracks the
stages read are that index's rows.

The synthesis runs in stages after the tracks close (the plan and the
owner's decisions: `brainstorming-notes/23-search-synthesis-plan.md`):
its briefs and drafts are numbered notes, each edited by the owner
before it runs — the digest brief was note 21 (at c56404ca, deleted at
acceptance), the framing is `brainstorming-notes/22-search-framing.md`;
the digest is `DIGEST.md`, accepted 2026-09-17; the proposal brief
for stage 3 was note 29 (at `e5d270bc`, deleted at the close), and
the two blind proposals it produced are
`brainstorming-notes/24-search-proposal-fable.md` and
`brainstorming-notes/25-search-proposal-astra.md` (merged f2b06a3e). Stage
4 is an audit (`proposal-audit/`), the author's repair
(note 32 at `a47fa1e5`), the review, a seat under note 30 (at
`a47fa1e5`) over anonymised copies (`search/designs/`, retired at
`548ecc4f`)
(`brainstorming-notes/26-search-review-by-fable.md`,
`brainstorming-notes/27-search-review-by-astra.md`), and the reconciliation,
`brainstorming-notes/31-search-reconciliation.md`, which Astra checks. Stage 5 is the ruling packet, `brainstorming-notes/28-search-ruling-packet.md` (the owner's answers, Astra's check under note 33 at `a0d85b23`), harvested into `decisions/search.md` 2026-09-17, its contract detail `DESIGN.md` with the language reference at its head (harvested 2026-09-19). Stage 6 is the build under `BUILD-PLAN.md`, this file its record.

## Step ledger

Steps as built, in the order they landed. A closed step is a line: the
step, its commits, and where what landed lives. What a module does is
its doc, a count is its table's or its test's, and the round's story is
the commit's. The first seat's row says more, since its run is
gitignored. The rows as they stood in full, steps 1 to 9b, are this
file at `880d9387`; what each step was to close on is the plan at the
same commit.

| Step | Commit | Where it lives |
| --- | --- | --- |
| 1 · the language | `080a8581` | `tree.rs`, `parse.rs`, `print.rs`, `json.rs`, `error.rs`; `tests/language.toml`, `tests/language.rs`; C89's edges, `tools/docs-check.sh` §5; H1–H5 |
| 2 · the derivation | `ef720323` | `derive.rs`; `tests/derive.rs`; M2; D1–D5 |
| 3 · the store's read | `c0a8f918` | the store's `corpus.rs` and its tests (C103, C108); M1 |
| 4 · the first surface | `28606ed4` | `bind.rs`, `corpus.rs`, `eval.rs`, `answer.rs`, `describe.rs`, `show.rs`; the CLI's `search_cmd.rs` and `tests/search_json.rs`; `tests/answer.rs`, `tests/acceptance.rs`, `tests/refusal.rs`; M3, M4; B1–B11 |
| 4b · the properties | `39e7667a`–`36a9738a` | `tests/common/generated.rs`; `tests/generated_equivalence.rs`, `tests/generated_completion.rs`, `tests/generated_cross_checks.rs`; `group.rs`; `tools/docs-check.sh` §7 |
| 5 · counts and the vocabulary | `31b5f09d` | `counts.rs`, `answer::Router`; `tests/counts.rs`; E1–E5 |
| 6 · class | `eeaec66e` | `class.rs`, `reference/classes-v1.toml`, `tools/class-table.py`; `tests/class.rs`; G1–G6 |
| 7 · computed values | `b5d62d92` | `totals.rs`, `pseudo.rs`, the totals table (v4 since 9c2), `tools/totals-table.py`; `tests/pseudo.rs`; M6; T1, T2, T5 |
| 7 · the third look | `ecb83b65` | `tests/common/generated.rs`: the generators reach the computed values; no source changed |
| 8 · sockets | `ec024ceb`; reviewed `dca017f5`, `7010f658` | `sockets.rs`, `derive.rs`; `tests/sockets.rs`; K1 |
| 9 · price | `57c2f78b`; reviewed `1e0d85c6`, `7ff3c947` | `price.rs`, and the store's and the planner's part in the commits; `tests/price.rs`; P1, P2 |
| the first seat | `94a3d18c`, the release build (sha256 `9ecf385d…`; the owner's choice, so the seat feels the number the budget judges); the run is `runs/seat-2026-09-26/` — brief, 65 journaled asks, report, `replay.sh` — gitignored, so the fix commits carry each fault's story | An agent's seat with the owner in the loop (the plan, "The first seat"): a second session drove `acq search` and `acq show` on the owner's store through a journaling wrapper, one binary, no daemon, no repository edit; the orchestrating session routed the report. 65 asks; 10 faults, F1–F10, held at 9b; 10 verdicts, V1–V10 ("Holes ruled"); the six lines (the observations below); the wall times in `search/MEASUREMENTS.md`, the seat block. |
| 9b · the seat's fixes | `0d5706d1`–`73d33419` | `tests/seat_faults.rs`, the review's `review_` tests among them; the replay of the seat's asks, each diff read: the message of `1f9c868c`; V1, V2, V8, V10; G2 revised |
| 9c · the trade site's computed values | `6362255a`–`6b033c60` | `search/pseudo-stats/README.md` and its `data/table-changes.csv`; the totals table; `tests/pseudo.rs` (`v6_…`, the two `c94_…`); the site-sitting skill; the ranged family's totals wait for 9d |
| 9c2 · the site's other totals | `88810610`–`77685777`, and the close | `search/pseudo-stats/data/other-totals.csv`, the track's README and `data/table-changes.csv`; `reference/totals-v4.toml`; `tests/pseudo.rs` (`c94_a_reduced_…`, `v6_the_other_…`); four pseudos are no sum and are the plan's 9c3 |

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

The yields by look, per step — what a review found each time it came
back, the last look's zero being what a step closed on: step 4, 6, 5,
2, 3; step 4b (Astra), 6, 3, 3, 1; step 5, 6, 3, 0; step 6, 5, 3, 0;
step 7 (Astra), 6, 4, 0 — the third look the generators' (the ledger,
"7 · the third look"): one harness fault of the look's own, none of the
product's. Step 8: 5, 2, 0 — a socket whose group is unread read as possibly none, then counted twice.
Step 9: 5, 2 — every one reproduced and held (`tests/price.rs`, the `review_` tests; `listing.rs`, `snapshot.rs`): the checklist's join row, and a decoder's depth limit turning a deep field's sibling absent.
Five blind seats at step 5 (`f357de39`; Sonnet, one question each over
the owner's real store, `--help` and `--describe` their only sources)
answered every question in three to six invocations and found two
faults of the surface text, fixed at `2cbf7787`; what they said beyond
is under "Observations still open", step 5.
The first seat (2026-09-26; the ledger row): 65 asks, ten faults —
one of the evaluator's (F2), one of the counts' (F3), eight of the
answer's words (F1, F4–F10) — each held by `tests/seat_faults.rs` since
9b, seen to fail at the seat's hash first; the two shapes that came
back more than once are the checklist's last two rows.

## What the measurements taught

Each measurement's tables, commands and numbers are
`search/MEASUREMENTS.md`, read by the block named and never whole; a
number there was measured, never recalled. The verdicts:

- M1 (step 3): the streaming read is 36 ms release and 8.7 MB resident
  over 22,721 items and 29.9 MB of bodies.
- M2 (step 2, rerun at every change of the deriver): 0 unread and 0
  unexplained against the census; seven departures, each a rule of
  `derive.rs`, two of them candidate ground-truth claims.
- M3, M4 (step 4, again at every build since): every release ask under
  500 ms, so the projection park does not fire; the one ask over it, the
  vocabulary whole at `4237d2f3`, was ruled at T5 and is under again;
  `~` over all text adds 9 ms; the debug build is 1.7 to 2.8 s.
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
- M3 (step 9): the price join raised every release ask 150–180 ms (the
  empty query 276 → 445); the load under 500, so the projection park
  does not fire; two totals asks over 500 on the higher floor, ruled at
  the seat (V9).
- M3 again at the seat's hash (9b, V9): the six asks the seat saw at
  695–715 ms are 446–459 on the copy — the spikes did not recur, their
  cause unmeasured; the totals batch stays parked. M3 at 9b's hash
  (`0e4c91ad`): the empty query 448, every ask 441–529, the same two
  totals asks over 500; the fixes moved no ask past the noise.
- M3 at totals v2 (`e81cc8ca`): every ask 445–488 but the totals' two,
  579 and 578, 50 and 55 more than at 9b: a total's cost is its rows,
  155 where there were 104.
- M3 at totals v3 (`9842d0c9`): the two are 588 and 587, moved with the
  floor and not by the table, whose 128 new rows are other totals';
  a third ask is over, a total of 10 rows at 508. An ask pays for the
  rows of the total it names, near 6 ms each over a floor of 452.

## Holes ruled, and where the rule went

What the build met that the reference did not state and the owner
ruled; none is unruled today. The rule is in the reference or the
registry and the mechanism in a module doc; the evidence each was
decided on — the census numbers, the builder's recommendation, the
owner's words verbatim — is this file at `aeeba6d3` and the ruling
commit's message. One line each.

| Step | Hole, as ruled | The rule is in | Ruled at |
| --- | --- | --- | --- |
| 1 | H1 a template alone means the line; H2 `+#` is spelling and dropped, `-#` an error offering `argN<0` | the reference, *Item-level*, *Strings* | `c58728a6` |
| 1 | H3 no whitespace around an operator; H4 a value of more than one word after `:` is quoted; H5 `AND`, `Or`, `NOT` in any case | `parse.rs`; `tests/language.toml` | `8a7bf03a` |
| 2 | D1 `rarity` and `frame` are two fields, each what GGG gives; D2 an `ilvl` of 0 is absent; D3 a vaal gem's base skill is the source `hybrid`, its properties displayed strings | the reference, *Item-level*, *Members*; `derive.rs` | `e29b698e` |
| 2 | D4 the other property-shaped arrays are not read until a question needs one; D5 requirements are one displayed row, each kept beneath it | the reference, *Item-level*; `derive.rs` | `890dca4d` |
| 4 | B1 every text comparison is any-case; a quoted template that found two spellings lists both | the reference, *Strings*; `bind.rs` | `774620b6` |
| 4 | B2 closed lists for `is:`, `source=` and a line's flags; B3 `tab:` tests a substash's name and its tab's; B4 `id:` is an item's or its place's, whole; B5 `--sort` takes a number; B6 a bare word's closed-set readings come first; B7 failed against lacked on a group; B9 a leading `-` goes after `--`; B10 a largest not established sorts last, its status shown; B11 the slot check reads the group's conjuncts; the basis names the store by twelve hex digits of its path's hash, an id the file carries parked | the reference (`f5701893`); `bind.rs`, `group.rs`, `eval.rs`, `corpus.rs`; the park in `decisions/search.md` | `7678ae27` |
| 4 | B8 the apostrophe: `--query-file`, every printed command shell-quoted; a single-quoted string (gap 2's (c)) closed — the fix, if typing one bites at the seat, is the adapter's or the help's, never the grammar's | the reference, *Strings*; `search_cmd.rs` | `28606ed4`, `3b7cf192` |
| 4b | what a number is and what one written longer becomes; nothing reaches a bound together without an occurrence that counts; what a row shows of one term | the reference, *Slots*; `eval.rs`, `answer.rs` | `2b01b1bf` |
| 5 | E1 a value bucket's term selects the counted spelling, a tab's its full coordinate with its substashes; E2 `--sum` takes the item's `sum( … )`, never a raw line projection; E3 the vocabulary's `undecided` is uncertainty about presence; E4 the view combinations stay errors and no grand total is needed; E5 `line` never crosses | C95, C105; `counts.rs`, `answer.rs` (`View::of`), `bind.rs` (`bind_sum`) | `0df86686`, `f357de39` |
| 6 | G1 class names are the game's plurals, `:` picks by word; G2 a base under several classes: the frame picks among them, the item each that remains (revised at 9b, owner 2026-09-26: "The frame picks among candidates, then any remaining match is true"; first: undecided, the table never chooses); G3 every release state enters, a stash keeps what the game removed; G4 no table for `poe2`, every item there undecided; G5 RePoE's licence read, the table carries base and class names only | `class.rs`; `tools/class-table.py`; `SURFACES.md` | `18e1f5be` |
| 7 | T1 a total is never rounded, `94.5` is `94.5`; T2 `has:` asks a derived field's presence — `-has:pseudo.dps` routes the lacked count — and a total's since 9c | `totals.rs`, `exact::Exact::halved`; `bind.rs`, `answer.rs`; the reference, *Values* | `c81ebe1e`; T1 as built at `b5d62d92`, T2 at `f098232a`, revised at `37faf903` |
| 7 | T5 `line` alone lists no computed values, a narrowing lists those it matches, `--describe` names them; an ask over budget fires the projection park only by its load, an evaluator cost the totals batch park; the batch parked with its trigger, a 7b of 4b's shape, generators first | `pseudo.rs`, `counts.rs`; the reference, `--count line`; the two parks in `decisions/search.md` | `66a20acf`; built at `57ec6a9e` |
| 6 | G6 the grouping above class stays parked; OQ5 pinned from the owner's words with the wearable classes spelled out, the park's trigger now the seat | the park in `decisions/search.md`; `tests/acceptance.rs` | `0d3c65af` |
| 8 | K1 the colour words are the reference's four; an abyssal (`A`) or resonator (`DV`) socket counts in `sockets` and is asked for by its line or base | `sockets.rs`; the reference, *Values* | `dca017f5`; as built at `ec024ceb` |
| 9 | P1 a listing resolved to `skip` or `no_price` lacks `has:priced`, known absence beside no row at all — a `price.kind` field waits for a seat that asks for skips (C101); P2 a decimal price lacks `price.lot`, a ratio has one | the reference, *Values*; `price.rs` | the ruling commit |
| seat | V1 a blighted map's class is read past the API's prefix (`Blighted Map (Tier 13)` is Maps); a beast's follows the trade site's categories (9d); V2 an invitation's class is its frame's — quest Quest Items, any other Misc Map Items; V4 the variant field stays parked; V5 `pseudo.defence_pct` is the site's Base Percentile, built at 9d, the ranged total's lines 9c's to evidence; V6 every total counts what the site's pseudo counts, 9c's human-run searches settling it, what cannot be mimicked listed; V7 C104 kept; V8 undecided by distinct reason, one example, one route, no items, no per-reason count; V9 the totals batch parked, the spikes measured first; V10 a tab's type a field at 9b | `class.rs`, `answer.rs` (the detail taken at 9b); the 9c line below; the plan, 9d, T3; the parks | the routing commit |
| 9c | a total of nothing is lacked and `has:` asks its presence, T2's "every item has a total" taken back ("A"); a row's eldritch forms are the row ("include the eldritch mods"); what the site leaves out and the search counts: the twin on That Which Was Taken ("I believe this is a bug. Let's count the mod"), the skill gems' own text ("yes, include the socketed skill gems"); a ranged total asked with no slot word is its `avg` ("yes, bare means average"), built at 9d; composed links ("yes"); the site's other totals at launch | `totals.rs`, `pseudo.rs`; the reference, *A sum's status*; `totals-v2.toml`; `SURFACES.md`; the plan, 9c2 and 9d | `37faf903`, `e81cc8ca`, `6362255a` |
| 9c2 | the two counts are in scope ("In"); explained is enough at launch ("Yes"); a row's `reduced` spelling is the row, counted below nothing ("yes, we need to be able to find reduced lines and totals. There are occasionally niche builds for which this is critically important."); the twin on That Which Was Taken counted toward mana ("yes, count it for mana."); life regenerated read from the line's text, a tenth a line under the site's at most ("yes, let's go with what we can observe directly from the text we have.") | `totals.rs`, *a weight*, *a limit*; `totals-v4.toml`; `tools/trade-changes.py`; the brief at `88810610` | `9842d0c9`, `77685777` |
| plan | gap 1 a line break inside a template; gap 4 a node forced true or false, `true()` and `false()`; gap 5 `name`, `typeline` and `base` each what GGG gives; gap 6 the totals example cites the C++ app's table, and whether a fractional total is ever rounded is step 7's to show | the reference, *Strings*, *Composition*, *Item-level*; the contract detail, C94 | `acfc37cd`, `152bfde3`, `fe9ca5f4`; 6 at `aeeba6d3` |
| plan | gap 3 membership is a scope value — `live` by default, `all` on request with every removed row marked, `removed` alone waits for a question, the prune verb advances the revision; `all` moved to step 10 | the reference, *Membership*; C108; the plan, step 10 | `3b7cf192`, `d829c25a`, `ea68d7c2` |

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
  undecided: each comparison is read over the interval on its own. For
  the seat did not reach it.
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

- *9c3:* a total read by a count or a least-of is asked three or four
  at a time of every item, and a totals ask is over budget already
  (M3, below the table). The four were asked one way, what shows the
  pseudo; whether an item showing every total a reading names ever
  shows no pseudo was not asked, so each is explained, never closed.
- *9d:* the two leech pseudos count the line under every id, the
  `(Local)` twin among them, so neither waits for a row that names what
  the item is; of 9c2's totals none does.
- *The trade translation (C99):* what the search counts and the site
  leaves out is said in the table's changes and nowhere a user reads.
- *A fix on the site:* the owner reported the twin id to GGG
  (2026-09-26); it is left out of total Strength, Intelligence and mana
  as of total life, and a fix moves four `not mimicked` rows to
  agreement.
