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
close — its ledger row, its holes, the measurements it repeated — and
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
`brainstorming-notes/31-search-reconciliation.md`, which Astra checks. Stage 5 is the ruling packet, `brainstorming-notes/28-search-ruling-packet.md` (the model page, the owner's answers, Astra's check under note 33 at `a0d85b23`), harvested into `decisions/search.md` 2026-09-17; its contract detail is `DESIGN.md`, which opens with the language reference — the surface on one page, an example per construct (the stage-5 audit's finding 5, harvested 2026-09-19). Stage 6 is the build, under `BUILD-PLAN.md`: a brief the owner edits before it runs, deleted at the close; this file is its record.

## Step ledger

Steps as built, in the order they landed; what each was to close on is
the plan's step table. A row names what landed — modules, tests, ids,
commits, measured counts — and points at the measurement or hole it
left below; what a module does is its doc, and the round's story is the
commit's.

| Step | Commit | What landed |
| --- | --- | --- |
| 1 · the language | `080a8581` | `acquisition-search`: `tree.rs`, `parse.rs`, `print.rs`, `json.rs`, `error.rs` (C89, C104). `tests/language.toml`, 133 cases over 86 constructs (123 over 82 at the commit; H1 and H2 the rest), and `tests/language.rs`, which refuses a construct with no case; the round trip over generated trees (2,000 a run; 60,000 once), any finite number, and no text panicking the parser (200,000 once). C89's edges in `tools/docs-check.sh` §5, nine breakers in `tools/docs-check-breakers.sh`. |
| 2 · the derivation | `ef720323` | `derive.rs`: `derive(facts, body) -> Item`, pure and total (C103, rule 8); `template::typed` reads the thousands comma. `tests/derive.rs`: 12 fixtures worked by hand, and no text and no JSON panics it. M2 below. |
| 3 · the store's read | `c0a8f918` | `acquisition-store/src/corpus.rs`: `Store::read_corpus` — the header, then every live item at a live location streamed inside one read transaction, the body as text — and `Store::revision` (C103; C108 given its own entry, the wording approved 2026-09-20). Seven store tests under `REFRESH-SLICE.md`'s checklist: the one snapshot over two handles, the league join, live by full coordinate in one realm or all, the revision through every write door, no derived column, no account refused, the header's coverage — two shown to fail without their exclusion. M1 below. |
| 4 · the first surface | `28606ed4` | `acquisition-search` links the store (C89): `bind.rs`, `corpus.rs`, `eval.rs` (C92, C93), `answer.rs` (C100), `describe.rs`, `show.rs`; the CLI's `acq search` and `acq show` (`search_cmd.rs`, C53), the flags of steps 5 and 10 refused by name; README tour lines; `CLI-REFERENCE.md` regenerated. Tests at the crate's boundary, every count by hand: `tests/answer.rs` (the worked example reduced, the route property, invariants 2 and 6, C93, C92, C96, C98), `tests/acceptance.rs` (OQ1–OQ4, OQ7, AQ2–AQ5 as far as built; S12, S52, S107), `tests/refusal.rs` (the walk over `language.toml`, invariant 3, S53), the CLI's `tests/search_json.rs`. M3 and M4 below. |
| 4b · the properties | `39e7667a`–`36a9738a` | Three properties through the crate's boundary from one set of generators (`tests/common/generated.rs`): `tests/generated_equivalence.rs` (invariant 7), `tests/generated_completion.rs` (C93; its measured half below), `tests/generated_cross_checks.rs` (transformations 10 and 13). Then `group.rs`, a group's meaning computed once at binding, and `tools/docs-check.sh` §7 with five breakers. No test of step 4 touched; the answer's JSON did not move. |
| 5 · counts and the vocabulary | `31b5f09d` | `counts.rs`: `--count`, `--cross`, `--sum` (C95), the `none` and `undecided` buckets (C105), the vocabulary as the key `line` (C97), every route by the one maker (`answer::Router`), `bind::exact_pattern` and `bind::folded` for a value's two spellings; `--describe counts`; the CLI's three flags, a README tour line, `CLI-REFERENCE.md` regenerated. `tests/counts.rs` (AQ1; C105's two invariants on `rarity`; C95's sum over the three kinds; the vocabulary's pasted term; all-realms rows; the crossed table; the cut; every view error), fourteen items by hand, every route followed by id; `bind.rs`'s two properties, each shown to fail; the CLI's route check. |
| 6 · class | `eeaec66e` | `class.rs` and `reference/classes-v1.toml`: the class table as reference data (C106, C68) — 82 classes, 4,547 base names, the game's own names, by `tools/class-table.py` from the pinned RePoE export (`base_items.json`, `item_classes.json`, `gems.json` for 225 transfigured gems); `class` a closed-set field, `undecided(class)` with one reason each (C105); `reqlevel`; the basis gains `classes v1` (C98); `show` and `--describe class`. `tests/class.rs`, `tests/acceptance.rs` (OQ1 and OQ4 as worded, OQ5), `tests/counts.rs` (C105's ten rare items by class: Rings 6, Wands 3, undecided 1). `DERIVATION` 6. The census count and M3 below; the holes and observations of step 6 below. |
| 7 · computed values | `b5d62d92` | `totals.rs` and `reference/totals-v1.toml`: the totals table as reference data (C94, C68) — 35 totals, 104 rows, by `tools/totals-table.py` from the C++ app's pseudomod tables at `master@946a4f51`; `pseudo.rs`: the `pseudo.` namespace, `pseudo.dps` and `pseudo.pdps` (C101); the basis gains `totals v1` (C98); `--describe` lists every computed value. `pseudo.defence_pct` and a ranged total refused by name, the entry naming no step. `tests/pseudo.rs`, `tests/answer.rs` (the worked example whole), `tests/acceptance.rs` (AQ2 as worded). M6 and M3 below; T1, T2 and T5 ruled below, T3 and T4 at the plan's foot; the observations of step 7 below. |
| 7 · the third look | `ecb83b65` | The generators reach the computed values, nothing else (owner, 2026-09-24, `66a20acf`): resistance lines and a `properties` array with holes in the bodies; a total's comparison, the derived fields' comparisons and `has:`, the probe, the case alt and the three sorts in the query tree; an eighth anchor past the reasons bound, a fixed case beside the rare find. Shown to catch the first round's 1 and 5 and a mutant per property; its 2 is held by `exact.rs`'s unit test. No source changed. |
| 8 · sockets | `ec024ceb`; reviewed `dca017f5`, `7010f658` | `sockets.rs`, and `derive.rs` reads the socket collection at the socket's grain (C101, rule 8); a count is an interval of what was read, decided where the whole interval agrees (`eval.rs`); `linked( … )` bound in `group.rs` beside a line's group; `show` and a row print the layout, made once; `--describe` gains the fields and the `linked` block. `DERIVATION` 9. `tests/sockets.rs` (every count by hand; three mutants caught), `tests/derive.rs`, `tests/acceptance.rs` (OQ3 as worded); the generators reach the sockets (`tests/common/generated.rs`). M2, M3 and the copy's counts below; K1 ruled below; the observations of step 8 below. |
| 9 · price | `57c2f78b`; reviewed `1e0d85c6`, `7ff3c947` | `price.rs`: the effective price joined read-only from the pricing area's listing state (C81, C100) — one snapshot and one `resolve` per (realm, league) the corpus names, keyed by item id; the crate links `acquisition-plan` (C89); the basis gains the intent revision, the currency table's and the note parser's versions (C98), the facts and intent revisions read again after the join and the corpus read again where either moved; `has:priced`, `price.amount`, `price.currency` (the table's tags, a closed set), `price.lot`; the reason *price unresolved* (`Part::Price`); a count by a number keeps its decimals (`Of::Number` is `Exact`); `show` carries the price; the CLI opens the intent file beside the store and prints the price and the basis whole. The store gains `Annotations::revision` and carries a note that is no string as unread (`ItemSnapshot::note_unread`); the planner leaves such an item's price unresolved (C81). `tests/price.rs` (thirteen items by hand, every route followed), `tests/acceptance.rs` (OQ6), the generators reach the price, the CLI's `search_json.rs`. M3 below; P1 and P2 at the plan's foot; the observations of step 9 below. |

## Findings

Every step was reviewed from outside — three to five looks at each,
every finding reproduced by the builder as a failing test before it was
taken — and a finding a test holds is held there and nowhere else (P5):
the test names the fault, the commit that fixed it tells the story, and
`git log` over the step's range is the ledger of rounds. Every finding
of steps 1 to 7 is held by a test, by a measurement below, or by a
ruled hole; the full table — each finding as the auditor found it, its
verdict, its fix commit and what holds it — is this file at `aeeba6d3`,
and step 7's two rounds at `e0420428`.
What this section keeps is the checklist a review of a search change
reads first: the shapes of fault that came back until they were named.

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
| **A claim the code did not make.** A hand count wrong; `DERIVATION` not moved when a body derived to another item; a cause named before it was measured; "covered" said of a property whose generators could not reach the case; a record row crediting the wrong commit | every step | every count worked by hand and then run; the constant's rule on its own doc; a number stated only after measuring; the generators reaching what a fix touched (`reqlevel` joined them at step 6) |

The yields by look, per step — what a review found each time it came
back, the last look's zero being what a step closed on: step 4, 6, 5,
2, 3; step 4b (Astra), 6, 3, 3, 1; step 5, 6, 3, 0; step 6, 5, 3, 0;
step 7 (Astra), 6, 4, 0 — the third look the generators' (the ledger,
"7 · the third look"): one harness fault of the look's own, none of the
product's. Step 8: 5, 2, 0 — a socket whose group is unread read as possibly none, then counted twice.
Step 9: 5, 2 — every one reproduced and held (`tests/price.rs`, the `review_` tests; `listing.rs`, `snapshot.rs`): a join that inherited the store's failure grain, twice; an override past the gate its own rule states; a producer's SQL type taken for the JSON's; a number from another area past the crate's rule; then a decoder's depth limit turning a deep field's sibling absent, and the gate mirrored in the decision but not in the reason.
Five blind seats at step 5 (`f357de39`; Sonnet, one question each over
the owner's real store, `--help` and `--describe` their only sources)
answered every question in three to six invocations and found two
faults of the surface text, fixed at `2cbf7787`; what they said beyond
is under "Observations still open", step 5.

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
  the corpus, the rows shipped unchanged; what no total counts is T4.
- The completion property's measured half: no counterexample, at step
  4b and over step 8's generators.
- The vocabulary against M2 (step 5): 6,181 rows and 6,148 templates
  agree.
- Step 8: every socket bucket a value or `none`, `undecided` 0; every
  group on the copy one contiguous run.
- M3 (step 9): the price join raised every release ask by 150–180 ms
  (the empty query 276 → 445); the load stays under 500 ms, so the
  projection park does not fire by the owner's rule, and two totals
  asks crossed 500 with the higher floor — a shape the ruling did not
  name, for the owner (the observations of step 9).

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
| 6 | G1 class names are the game's plurals, `:` picks by word; G2 a base under several classes is undecided, the table never chooses; G3 every release state enters, a stash keeps what the game removed; G4 no table for `poe2`, every item there undecided; G5 RePoE's licence read, the table carries base and class names only | `class.rs`; `tools/class-table.py`; `SURFACES.md` | `18e1f5be` |
| 7 | T1 a total is never rounded, `94.5` is `94.5`; T2 `has:` applies to a derived field, never to a total — `-has:pseudo.dps` routes the lacked count, `has:pseudo.total_res` an error with readings | `totals.rs`, `exact::Exact::halved`; `bind.rs`, `tree::has_on_computed`, `answer.rs`; the reference, *Values* | `c81ebe1e`; T1 as built at `b5d62d92`, T2 at `f098232a` |
| 7 | T5 `line` alone lists no computed values, a narrowing lists those it matches, `--describe` names them; an ask over budget fires the projection park only by its load, an evaluator cost the totals batch park; the batch parked with its trigger, a 7b of 4b's shape, generators first | `pseudo.rs`, `counts.rs`; the reference, `--count line`; the two parks in `decisions/search.md` | `66a20acf`; built at `57ec6a9e` |
| 6 | G6 the grouping above class stays parked; OQ5 pinned from the owner's words with the wearable classes spelled out, the park's trigger now the seat | the park in `decisions/search.md`; `tests/acceptance.rs` | `0d3c65af` |
| 8 | K1 the colour words are the reference's four; an abyssal (`A`) or resonator (`DV`) socket counts in `sockets` and is asked for by its line or base | `sockets.rs`; the reference, *Values* | `dca017f5`; as built at `ec024ceb` |
| 9 | P1 a listing resolved to `skip` or `no_price` lacks `has:priced`, known absence beside no row at all — a `price.kind` field waits for a seat that asks for skips (C101); P2 a decimal price lacks `price.lot`, a ratio has one | the reference, *Values*; `price.rs` | the ruling commit |
| plan | gap 1 a line break inside a template; gap 4 a node forced true or false, `true()` and `false()`; gap 5 `name`, `typeline` and `base` each what GGG gives; gap 6 the totals example cites the C++ app's table, and whether a fractional total is ever rounded is step 7's to show | the reference, *Strings*, *Composition*, *Item-level*; the contract detail, C94 | `acfc37cd`, `152bfde3`, `fe9ca5f4`; 6 at `aeeba6d3` |
| plan | gap 3 membership is a scope value — `live` by default, `all` on request with every removed row marked, `removed` alone waits for a question, the prune verb advances the revision; `all` moved to step 10 | the reference, *Membership*; C108; the plan, step 10 | `3b7cf192`, `d829c25a`, `ea68d7c2` |

## Observations still open

The builder's observations that became neither a ruling nor a finding;
each is data for the step or the seat that touches it. What described a
mechanism the crate's module docs carry, and what a later fix made
history, was taken out on a read against those docs (the commit that
did so lists them); what stays is measurements of the owner's copy,
timings, coverage no fixture reaches, and questions for the seat.

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
  has 16; the type as a field is parked (`decisions/search.md`).
- Over the copy, `name="Ashes of the Stars"` finds ten, which is the
  variant park's first test (the plan, "Parks whose triggers the build
  fires"): step 4 has reached OQ2, and the question is the owner's.

**Step 4b.**

- The gate's cost, a debug build here: equivalence 14 s, the cross-checks
  15 s, completion 3 to 6 s, beside the routes' 16 s. 256, 192 and 256
  cases; `PROPTEST_CASES` runs any of them longer by hand.
- The basis names no evaluator, so two builds can label different
  answers with one basis — true of every evaluator fix since step 4;
  `DERIVATION` moves only when a body derives to another item.

**Step 6.**

- The class table over the copy (the measurement above): 56 of its 82
  classes are carried; of 22,721 items 681 are undecided — 388 whose
  base is not in the table, 230 of them blighted and blight-ravaged
  maps (`Blighted Map (Tier N)` is no export base, though `Blighted
  Map` is) and the rest itemised beasts foremost (71 distinct names in
  all); 195 under several classes, the Eldritch and Maven's invitations
  foremost; 98 in poe2. No gem is among them: `gems.json` supplied every
  transfigured gem of the copy, 272 items step 2's census could not join
  by base. Whether a blighted map's class, or an invitation's, is worth a
  reviewed row of a later table version is the owner's (G2).
- `class:staff` is answered with the whole list of 82 names, `staff`
  being three edits from `Staves` (G1): the first seat is where to learn
  whether the game's plurals bite.
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
- The five blind seats (the findings above), beyond their two fixes:
  the together count steered one from `"# to maximum Life">=70` to
  `sum("# to maximum Life")>=70` inside `holds`, 76 items to 79, and it
  still said a first-timer writes the line form and undercounts unseen
  — evidence for C95's sum rule, one of the six lines the seat revisits
  first. "My stash" was read as `has:tab`, which nothing labels. With no
  `class:`, the added-damage seat found its template by `--count
  line:physical`, told it from the `to Attacks` twin and checked bases by
  eye, calling the vocabulary-first walk essential and non-obvious (data
  for step 6). Three of five wanted the fetch age and the unfetched count
  more prominent than one line of the scope block: an opinion on order,
  the seat's to weigh. Five recoveries from C96's refusal in one step
  each is not the sticky-realm park's trigger, which is the GUI's
  (2026-09-25).

**Step 7.**

- Over the copy nothing is unread and every item is in a covered realm
  but poe2's 98, so a total's incomplete subtotal is exercised by
  fixtures alone; the unavailable status by the copy's poe2 items.
- The site's `+94.5 total maximum Life` is one item (q4, `Oblivion
  Sanctuary Crusader Plate`: `+9 to Strength`, `+90 to maximum Life`);
  whether `+# to Strength and Intelligence` or `+# to all Attributes`
  count at the half is unread (q5's items carry them, and the site
  answered fire resistance alone), so `total_life` ships in no table
  until one trade search per candidate line closes it (pseudo-stats,
  open question 1's shape).
- A derived field's lacked item sorts last with the status `no
  satisfying occurrence`, which is a line's wording; a field's and a
  computed value's is the same status today.
- The generated properties reach `pseudo` since the third look
  (`ecb83b65`), and still no `class`; the completion's measured half
  after it: 9 groups failed as stored and lacked under a completion in
  2,000 cases (19 and 21 with step 4b's generators).
- `has:Pseudo.DPS` is refused as no field the language knows: the
  `pseudo.` namespace is grammar, read by the parser as typed, where a
  name after it is matched in any case (B1). For the seat.
- `pseudo.dps` counts every damage property the item displays and
  `pseudo.pdps` the physical alone; the C++ app reads 0 where a property
  is missing and this build says *lacked* — the one departure, C93's.

**Step 8.**

- On a socket whose colour is unread `linked(red=1 or red=0)` is
  undecided: each comparison is read over the interval on its own. For
  the seat.
- A poe2 socket's `type`, a gem's own `colour` and `socket`: kept, asked
  by nothing.

**Step 9.**

- M3's two asks over budget (AQ2 as worded 529 ms, the worked example
  whole 528) are the totals' unchanged 84 ms on a floor the price join
  raised by 170: neither a load over budget (the empty query is 445)
  nor an evaluator cost that grew; by the letter of the totals batch
  park's trigger, "a totals ask over budget", it fired. Three candidates,
  each unmeasured: join the price only when the request names it; the
  note as an ingest column, which would take the snapshot's read from
  about 80 ms to about 14; the persisted projection. The owner's
  disposition (2026-09-25): a good signal, addressed after the seat and
  the last steps, not before, even at the cost of slower testing. What
  the discussion settled for then: the load is under its budget, so the
  projection's trigger has not fired and the projection would not touch
  the join; the effective price crosses facts and intent and so stays a
  read-time derivation, its inputs the only lever; a note column makes
  `note` an ingest fact, which under C103 moves the field from the
  deriver to the store's read; the CLI pays the join on every ask, a
  consumer that holds a corpus once per basis change, so the choice step
  11 fixes in place is when a corpus joins — at load or on first ask.
- The generators reached the pricing area before any seat did: a body
  whose note is no string failed the pricing snapshot whole, and would
  have failed `acq price status` the same way; fixed at the store and
  the planner (`57c2f78b`), a finding of the pricing area's and held
  by its tests. Over the copy no note is one, so the path is fixtures'
  alone.
- The price join is read after the corpus's transaction, so its
  consistency is a check, not a snapshot: the facts and intent
  revisions read again, and the corpus read again where either moved,
  three times before an error. A store read that hands the pricing
  snapshot over inside the corpus's transaction would make the reread
  unnecessary; no consumer has met the error.
- The intent revision on the basis is the whole file's — a sync-policy
  write moves it too, and a held corpus reloads for it. No consumer
  holds a corpus across asks yet.
- `price.from` on a row names the tab, substash or character a
  statement came from and nothing when it is the item's own; the
  cross-checks' copied item showed the address otherwise. The same
  shape bit the unresolved reason, now worded from the listing's parts.
- A skip or a no-price row is known absence for `has:priced` (P1); a
  decimal price lacks `price.lot` (P2): both at the plan's foot, for the
  seat, built on the builder's recommendation.
- `show`'s price is the item's league's listing state, or every league
  of its realm for a league-less character, and one read under the empty
  league name for a realm that names none (the review, 4); *uncovered*
  stays a reason of its own that no fixture reaches now.
- A price whose amount or lot is past the crate's rule for numbers (ten
  whole digits, four decimals) is priced and its number unread: the
  pricing area accepts what the search will not read as a number, and
  the seat may meet a bulk lot written that long.
