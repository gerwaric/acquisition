# The item search slice — the ledger

The rows the record (`SEARCH-SLICE.md`) grows by at every step's close:
the step ledger, the yields by look, the holes ruled, and how the design
was reached. Looked up by the row — a step, a hole's id, a commit — and
never loaded whole; what every step reads is the record. No budget: a
line per step and per hole by construction. A step writes its rows here
once, at its close. Nothing here is a second authority: a hole's rule
is in the reference, the registry or a module doc, and its row says
which. The checklist, the verdicts and the observations these rows name
are the record's.

## Step ledger

Steps as built, in the order they landed. A closed step is a line: the
step, its commits, and where what landed lives. What a module does is
its doc, a count is its table's or its test's, and the round's story is
the commit's. The first seat's row says more, since its run is
gitignored. The rows as they stood in full, steps 1 to 9b, are the
record at `880d9387`; what each step was to close on is the plan at the
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
| 7 · computed values | `b5d62d92` | `totals.rs`, `pseudo.rs`, the totals table (v5 since 9c3), `tools/totals-table.py`; `tests/pseudo.rs`; M6; T1, T2, T5 |
| 7 · the third look | `ecb83b65` | `tests/common/generated.rs`: the generators reach the computed values; no source changed |
| 8 · sockets | `ec024ceb`; reviewed `dca017f5`, `7010f658` | `sockets.rs`, `derive.rs`; `tests/sockets.rs`; K1 |
| 9 · price | `57c2f78b`; reviewed `1e0d85c6`, `7ff3c947` | `price.rs`, and the store's and the planner's part in the commits; `tests/price.rs`; P1, P2 |
| the first seat | `94a3d18c`, the release build (sha256 `9ecf385d…`; the owner's choice, so the seat feels the number the budget judges); the run is `runs/seat-2026-09-26/` — brief, 65 journaled asks, report, `replay.sh` — gitignored, so the fix commits carry each fault's story | An agent's seat with the owner in the loop (the plan, "The first seat"): a second session drove `acq search` and `acq show` on the owner's store through a journaling wrapper, one binary, no daemon, no repository edit; the orchestrating session routed the report. 65 asks; 10 faults, F1–F10, held at 9b; 10 verdicts, V1–V10 ("Holes ruled"); the six lines (the record's observations); the wall times in `search/MEASUREMENTS.md`, the seat block. |
| 9b · the seat's fixes | `0d5706d1`–`73d33419` | `tests/seat_faults.rs`, the review's `review_` tests among them; the replay of the seat's asks, each diff read: the message of `1f9c868c`; V1, V2, V8, V10; G2 revised |
| 9c · the trade site's computed values | `6362255a`–`6b033c60` | `search/pseudo-stats/README.md` and its `data/table-changes.csv`; the totals table; `tests/pseudo.rs` (`v6_…`, the two `c94_…`); the site-sitting skill; the ranged family's totals wait for 9d |
| 9c2 · the site's other totals | `88810610`–`77685777`, and the close | `search/pseudo-stats/data/other-totals.csv`, the track's README and `data/table-changes.csv`; the totals table; `tests/pseudo.rs` (`c94_a_reduced_…`, `v6_the_other_…`); four pseudos are no sum, built at 9c3 |
| 9c3 · the four that are no sum | `c28c476a`, `1662e8ab`, and the close | `totals.rs` (`Reading`), `pseudo.rs` (a count, a least, `compared`, `present`, `sorted_by`), `[[reading]]` in the totals table; `tests/pseudo.rs` (the two `c101_a_…`, `c93_a_reading_…`, `c97_a_reading_…`), `tests/generated_cross_checks.rs` (`a_reading_is_what_its_totals_make`), the CLI's `tests/search_json.rs` (`step_9c3_…`); reviewed from outside — the brief at `ef2ca6c1`, the findings and the probe at `a77400ed` — and closed with 9c4, which holds the first finding |
| 9c4 · a row of a mod | `6d5847df`, `bd4679e1`, and the close; reviewed `0962588b`, `ca9f53bc`, `e8422ac6`, `80609d19`, `d9d7c9da`, `62a6d7c8` | `derive.rs` (`Line::rows`, `Line::names`, `Numbers`), `group.rs` (a quoted template's name, `Asked::parts`, `Slots`, `Group::resolved`), `counts.rs` (the vocabulary), `answer.rs`, `show`; `tests/rows.rs`, `tests/generated_rows.rs`, `tests/pseudo.rs` (`c94_a_row_inside_…`, the reviewer's probe), `tests/derive.rs`, `tests/generated_equivalence.rs` (the fourth way), the CLI's `tests/search_json.rs` (`step_9c4_…`); `search/pseudo-stats/scripts/rows-of-a-mod.py`; what it was to close on is the plan at `6d5847df`; L1, L2 and L3 are ruled |

## The yields by look

The yields by look, per step — what a review found each time it came
back, the last look's zero being what a step closed on: step 4, 6, 5,
2, 3; step 4b (Astra), 6, 3, 3, 1; step 5, 6, 3, 0; step 6, 5, 3, 0;
step 7 (Astra), 6, 4, 0 — the third look the generators' (the ledger,
"7 · the third look"): one harness fault of the look's own, none of the
product's. Step 8: 5, 2, 0 — a socket whose group is unread read as possibly none, then counted twice.
Step 9: 5, 2 — every one reproduced and held (`tests/price.rs`, the `review_` tests; `listing.rs`, `snapshot.rs`): the checklist's join row, and a decoder's depth limit turning a deep field's sibling absent.
Step 9c3: 2 — a row inside a mod of several rows fed no total, step 7's fault, which the step's fixture hid by writing the mod's rows apart: held at 9c4 (`tests/pseudo.rs`, the captures' mods entered unchanged); and a sentence of `pseudo.rs`, corrected (`b909524c`).
Step 9c4: 2, 2, 2, 2 — an alternative beside a comparison changed the number it read, one part having been read for the whole group: held (`tests/rows.rs`, `c92_an_alternative_…`; the equivalence property's fourth way; the rows-apart oracle's alternatives); and an observation of step 1 the step had made false, deleted. The second look, both of the first fix's making: a not parted a comparison from the template that said what it read, so a group and its not both held; and the mod's first number, read by its row and by the mod in order, was added twice — held (`tests/rows.rs`, `c93_a_not_is_…`, `c95_a_number_…`; `tests/generated_rows.rs`, the two laws). The third, of the second fix's making: a not inside a not weighed what it held again for each thing a template may name, ten deep 2.7 s on one item; and a number that is either of two rows' was taken for the number read at the first row's place, its being unread dropped — held (`tests/rows.rs`, `a_not_inside_a_not_…`, `c93_a_number_that_is_either_…`). The fourth, of a choice the step had made and not brought: a slot beside two rows named together read the mod's numbers in order, so a row named as a condition moved the number beside it, and a not of one row's number held beside another row named — ruled and refused (L3); and the slot check held every slot against the group's one quoted template, where the evaluator read another part — held (`tests/rows.rs`, the two `c92_a_…`, `what_a_slot_is_read_beside_…`; the equivalence property, a reading followed as a route is).
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

## Holes ruled, and where the rule went

What the build met that the reference did not state and the owner
ruled; none is unruled today. The rule is in the reference or the
registry and the mechanism in a module doc; the evidence each was
decided on — the census numbers, the builder's recommendation, the
owner's words verbatim — is the record at `aeeba6d3` and the ruling
commit's message. One line each, with the owner's words where they
state the rule or its reason, and never a bare assent.

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
| 9c4 | an exact template names a mod displayed over several rows by a row of it, in a line's term as in a total's row, and the mod stays one occurrence ("the two-line mods should contribute to relevant pseudo-lines, but it makes sense they are a single occurrence.") | the reference, *Strings*; `group.rs`, `derive.rs`, `totals.rs` | `47c9ba84`, `d24b9678`; built at `6d5847df` |
| 9c4 | L1 the words: the reference's *Strings* says a mod of several rows is one occurrence whose template is its whole text, named by that or by any one row, a pattern's ends the whole text's; C90 stands as it was, a line's identity being its whole template and a row a name; `--describe template` stands as the build printed it | the reference, *Strings*; `describe.rs` | the ruling commit |
| 9c4 | L2 the vocabulary lists a mod displayed over several rows by its whole text alone; a template's row counts the mods its term names by a row, so a count stays its term's, and a row no mod displays alone is no row of it — every row listed would add 856 rows to the copy's 6,148 | `counts.rs`, the vocabulary; `tests/rows.rs`, `tests/generated_rows.rs` | the ruling commit; built at `6d5847df` |
| 9c4 | L3 a number word beside two rows of a mod named together is refused ("refuse"), which row's number never guessed: each reading quotes the row meant and asks the other of the mod's text, `template:`, which names nothing; the slot check holds a slot against the one quoted template only where that template alone says what it reads | the reference, *Strings*, *Slots*; `group.rs` (`Slots`), `tree.rs`; `tests/rows.rs` (the two `c92_a_…`), `tests/language.toml` | the ruling commit |
| plan | gap 1 a line break inside a template; gap 4 a node forced true or false, `true()` and `false()`; gap 5 `name`, `typeline` and `base` each what GGG gives; gap 6 the totals example cites the C++ app's table, and whether a fractional total is ever rounded is step 7's to show | the reference, *Strings*, *Composition*, *Item-level*; the contract detail, C94 | `acfc37cd`, `152bfde3`, `fe9ca5f4`; 6 at `aeeba6d3` |
| 9d | J5 a display at an exact half admits both integers as rolls, the definition printing it as its one departure from the site's tip ("Adopt J5 as proposed."); J6 the shapes round i asks are answered by the stated rule and listed as unpinned until the round is read — the owner's assent, the rule the builder's recommendation | the contract detail, C101 (`search/DESIGN.md`); `pseudo.rs` when built | the ruling commit |
| plan | gap 3 membership is a scope value — `live` by default, `all` on request with every removed row marked, `removed` alone waits for a question, the prune verb advances the revision; `all` moved to step 10 | the reference, *Membership*; C108; the plan, step 10 | `3b7cf192`, `d829c25a`, `ea68d7c2` |

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
`brainstorming-notes/31-search-reconciliation.md`, which Astra checks. Stage 5 is the ruling packet, `brainstorming-notes/28-search-ruling-packet.md` (the owner's answers, Astra's check under note 33 at `a0d85b23`), harvested into `decisions/search.md` 2026-09-17, its contract detail `DESIGN.md` with the language reference at its head (harvested 2026-09-19). Stage 6 is the build under `BUILD-PLAN.md`, `SEARCH-SLICE.md` its record.
