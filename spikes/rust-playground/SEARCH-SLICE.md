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
the plan's step table.

| Step | Commit | What landed |
| --- | --- | --- |
| 1 · the language | `080a8581` | `acquisition-search`: the tree and its validity (`tree.rs`), the parser over the whole reference (`parse.rs`), the canonical printer (`print.rs`), the strict JSON form (`json.rs`), structured errors with stable kinds (`error.rs`). `tests/language.toml`: 133 cases over 86 constructs (123 over 82 at the step's commit; H1 and H2 added the rest) — every construct of the reference, every error the grammar defines with its readings, the acceptance queries — and `tests/language.rs` refuses a construct with no case. The round trip over generated trees (2,000 a run; 60,000 once, by hand), any finite number, and no text panics the parser (200,000 once). C89's edges in `tools/docs-check.sh` §5 with nine breaker cases: `tools/docs-check-breakers.sh`, 51 ok. The contract detail's sentences on the check and the lints left `DESIGN.md` for the crate doc. |
| 2 · the derivation | `ef720323` | `derive.rs`: `derive(facts, body) -> Item`, pure and total — the header, `rarity` and `frameTypeId` as given, `ilvl`, `stackSize`, the note, every yes the body says; properties and requirements as displayed strings, name and values kept apart; lines as (source, flags, template, numbers) with `slot` and the ranged rule; `displayed()`, the rows a phrase will be tested against; and what could not be read, by part, the readable rest still derived. `tests/derive.rs`: 12 fixtures worked by hand and a property test that no text and no JSON panics it. M2 below: no unexplained difference. `template::typed` gained the thousands comma. |
| 3 · the store's read | `c0a8f918` | `acquisition-store/src/corpus.rs`: `Store::read_corpus` — the header (the revision, the account, the realms the file holds, the latest listing of each list in scope, every live location in scope with its `type`, `listed_at` and `fetched_at`) and then every live item at a live location, streamed to the caller's closure inside one read transaction, league joined as `read_items` joins it, the body as text — and `Store::revision`. Seven store tests, `REFRESH-SLICE.md`'s findings as the checklist: the one-snapshot test over two handles on one file (it fails when the snapshot is ended after the header — tried), the league join over a character the listing moved and a league-less one, live by full coordinate (one tab id under `pc` and `xbox`) in one realm or all, the revision through every write door, no derived column in the read or its order, a store with no account refused, and the header's coverage — a folder told from an unfetched tab, a retired tab's substash no live location (it fails without the exclusion — tried), an empty listing still seen. The revision's line-by-line read is that module's doc. C103's second decision is C108 (`decisions/store.md`), its wording approved by the owner (2026-09-20: "All is approved"). M1 below. |
| 4 · the first surface | `28606ed4` | `acquisition-search` links the store (C89) and gains `bind.rs` (the vocabulary as closed lists, near names, the one list of what is not built, a bare word's closed-set readings), `corpus.rs` (every live item of a scope derived from one `read_corpus`, held with its basis, its places and the coverage the scope block states), `eval.rs` (matched, failed, lacked, undecided; witnesses; three-valued composition and the `holds` interval; sums with their status; the sort scalar; the together count), `answer.rs` (the request and the answer: a route on every count that is not zero, what a `:` or `~` selector resolved to, rows with what matched, the zero block), `describe.rs`, `show.rs`. The CLI gains `acq search` and `acq show` (`search_cmd.rs`), text rendered from the value `--json` prints, the flags of steps 5 and 10 refused by name; README tour lines; `CLI-REFERENCE.md` regenerated. Tests at the crate's boundary, every count worked by hand first: `tests/answer.rs` (the worked example reduced, the route property, invariants 2 and 6, C93's composition, C92's scalar, C96's scope, C98 at the answer's boundary), `tests/acceptance.rs` (OQ1–OQ4, OQ7, AQ2–AQ5 as far as step 4 builds them; S12, S52 and S107 by their wording), `tests/refusal.rs` (the walk over `language.toml`, invariant 3, S53), and the CLI's `tests/search_json.rs`. Three mutants tried, each caught: unread beating a witness, a line's failed route as `-term`, not-undecided as true. M3 and M4 below. |
| 4b · the properties | `39e7667a`–`36a9738a` | Three properties beside the outside agent's routes, each through the crate's boundary over generated queries and generated items with unread evidence, from one set of generators (`tests/common/generated.rs`: a query tree of the test's own with its normal form, and bodies with holes that a completion fills): `tests/generated_equivalence.rs` (invariant 7), `tests/generated_completion.rs` (C93's definition, its measured half run by hand), `tests/generated_cross_checks.rs` (transformations 10 and 13). Then `group.rs`: what `line( … )` means, computed once at binding and read from there, the bound form private to it; `tools/docs-check.sh` §7 refuses a source file that names a group's tree outside `tree`, `parse`, `print`, `json`, `template` and `group`, with five breakers (`tools/docs-check-breakers.sh`, 56 ok). No test of step 4 was touched and the answer's JSON did not move. What they met: the findings and measurements below. |
| 5 · counts and the vocabulary | `31b5f09d` | `counts.rs`: the view is one of rows, `--count` and `--cross` (C95); a bucket is a term — `<key>=<value>`, `none` by `-has:<key>`, `undecided` by `undecided(<key>)` (C105) — its count the evaluator's own outcome on the matches and its route the query and that term, made by the one maker of a route (`answer::Router`); `tab` counted by the tab and routed by its id; two spellings of a value two buckets, each routed by a pattern that turns case back on (`bind::exact_pattern`), told apart by the matcher's own fold (`bind::folded`, `regex-syntax`); a value outside a closed list counted with no route; tables cut by the data, `none` and `undecided` never; the vocabulary as the key `line`, narrowed by `line:text` and `line~pattern`, a row per (realm, template) with its term, each number's range, and its sources and flags each with a term (C97); `--sum` over the three kinds, exact (C95); a tally by what was unread beneath `undecided`. `--describe counts`. The CLI: the three flags real, `line:` taking the rest of the list, the tables and their routes rendered; `CLI-REFERENCE.md` regenerated; README tour line. `tests/counts.rs` (AQ1; C105's two invariants on `rarity`; C95's sum over an item with the value, one lacking it, one unread; the vocabulary's pasted term selecting its row, source and flag both; all-realms rows; the crossed table; the cut; every view error) — fourteen items, every count by hand, every route followed by id; `bind.rs`'s two properties, the fold against any-case `=` and the exact pattern against its text, each shown to fail (below); the CLI's terminal-level route check. What it met: the findings and measurements below. |
| 6 · class | `eeaec66e` | `class.rs` and `reference/classes-v1.toml` in `acquisition-search`: the class table as reference data (C106, C68) — 82 classes, 4,547 base names, the game's own class names (`Rings`, `Staves`), generated by `tools/class-table.py` from the pinned RePoE export (`base_items.json`, `item_classes.json`, and `gems.json` for the 225 transfigured-gem names no base carries), compiled in, parsed once, checked by the loader; `class` a closed-set field (`class:ring`, `class=Rings`, `class:sword` three), `undecided(class)` with one reason each — base not in the table, base under several classes, no table for the realm, no base — tallied beneath a count's `undecided` (C105), the deriver's unread base said once (rule 8); `reqlevel`, the `Level` requirement as a number, absent where the requirements were read and hold none; the basis gains `classes v1` (C98); `show` prints the class; `--describe class` the definition and source. `tests/class.rs` (the shipped file, every reason, the tally, `show`, `reqlevel`), `tests/acceptance.rs` (OQ1 and OQ4 as worded, OQ5 askable), `tests/counts.rs` (C105's ten rare items by class: Rings 6, Wands 3, undecided 1). `DERIVATION` 6. M2 and M3 below; the holes and observations of step 6 below. |
| 7 · computed values | `b5d62d92` | `totals.rs` and `reference/totals-v1.toml` in `acquisition-search`: the totals table as reference data (C94, C68) — 35 totals, 104 rows, generated by `tools/totals-table.py` from the C++ app's pseudomod tables at `master@946a4f51`, each naming the trade site's pseudo stat; `pseudo.rs`: the `pseudo.` namespace over the table's totals and the derived fields `pseudo.dps` and `pseudo.pdps` (C101), what a comparison, a sort, a count's sum and `undecided( … )` consume when they name one, shown on a row with what it counted; the basis gains `totals v1` (C98); `--describe` lists every computed value with its definition and provenance. `pseudo.defence_pct` and a ranged total (`pseudo.<name>.<slot>`) refused by name, the entry naming no step. How a total and a field are decided is the two modules' docs. `tests/pseudo.rs`, `tests/answer.rs` (the worked example whole), `tests/acceptance.rs` (AQ2 as worded). M6 and M3 below; T1–T4 at the plan's foot; the observations of step 7 below. |

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
| **Unknown said at the wrong grain.** A whole object unread for one flag that is no boolean, a whole array for one number too long, a whole requirements list for one bad row: too wide leaves open terms that never needed it, too narrow reads as absence, which a not makes a witness of | steps 4, 4b, 6, 7 | rule 8; `derive::Slot`, `Line::flags_unknown`, `Unread::line`, `Unread::name`; the completion property |
| **Two statuses for one thing.** A number kept beside the unread that may replace it, so a comparison, the sort and the sum disagreed; a total exact on the item and rounded again in the bucket | steps 5, 6 | rule 8 (`reqlevel`'s one status); `exact::Exact`, units carried to the print |
| **A second maker.** A route, a printed command, a reason, a selector's resolved list, each made in two places that then disagreed — a lacked count of 1 whose route returned 0, a `show` continuation that dropped the account | steps 4, 4b, 5 | rule 10; `answer::command`, `answer::Router`, `eval::why`; the CLI test that runs every printed command with a second account known |
| **A printed command the build refuses.** A route starting with `-` at a shell, a reading that does not parse (`class=Body Armours`), a slot word the same build's slot check refuses, a continuation the key parser could not read back | steps 4, 5, 6 | rule 5; the route property; every offered reading bound by a test; `bind::closed` offering through the printer |
| **Arithmetic on floats.** A sum whose value depended on the order of its occurrences; a fallback whose use depended on how the integers cancelled; units read back off a float that does not print its decimal | steps 4b, 5 | `exact.rs`: a number read once, within a measured rule, and whole units from then on |
| **A cut in the query's order.** A reason made beyond the item's parts took its place from the term that met it first, so two spellings of one query showed different sixes | step 7 | rule 9; `eval::why` orders such a reason by what it says; the audit test in `tests/pseudo.rs` asks two spellings |
| **A block past its bound.** Rows appended after the cut — the vocabulary's computed values, 39 under a limit of 1 — with the omission uncounted | step 7 | invariant 5; every list an answer holds is cut by the limit and counts its rest, a new kind of row with its own count; the audit test |
| **A number no game displays, read as one.** Scientific notation through a length check; a product past the units rounded in silence | step 7 | `exact::reads` reads decimal syntax alone; `Exact::times` is none where the units cannot hold it; the input is unread to what asked it |
| **A claim the code did not make.** A hand count wrong; `DERIVATION` not moved when a body derived to another item; a cause named before it was measured; "covered" said of a property whose generators could not reach the case; a record row crediting the wrong commit | every step | every count worked by hand and then run; the constant's rule on its own doc; a number stated only after measuring; the generators reaching what a fix touched (`reqlevel` joined them at step 6) |

The yields by look, per step — what a review found each time it came
back, the last look's zero being what a step closed on: step 4, 6, 5,
2, 3; step 4b (Astra), 6, 3, 3, 1; step 5, 6, 3, 0; step 6, 5, 3, 0;
step 7 (Astra), 6, 4, and the third look is the next session's, its
scope the owner's (2026-09-24): "Step 7's third look is generators reaching pseudo, nothing else: total_res, dps and pdps, in the existing composition, shown to catch a fault from these rounds."
Five blind seats at step 5 (`f357de39`; Sonnet, one question each over
the owner's real store, `--help` and `--describe` their only sources)
answered every question in three to six invocations and found two
faults of the surface text, fixed at `2cbf7787`; what they said beyond
is under "Observations still open", step 5.

## What the measurements taught

Each on the `.backup` copy the plan names, with the command that
produced it; a number here was measured, never recalled. The story
behind each — what a run first said and what was withdrawn, the
mutants tried against the properties, why the suites had passed over
what an audit found — is this file at `aeeba6d3`; what a property
means and what it cannot reach is its own header.

**M2 — the deriver against the census (step 2, `ef720323`; rerun at
every change of the deriver since — 0 unread, 0 unexplained each time).** `python3
search/item-facts/scripts/m2-differential.py`, after `cargo build
--workspace`; input the live items of
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

The departures, each a rule of `derive.rs` ("As built"), counted by the
script (lines whose template moved / census templates touched):

| Departure | Count | Why |
| --- | ---: | --- |
| `ultimatumMods` is no source of lines | 3,567 lines | its elements are ids with a tier (`FrostInfection`, 3); the same item's `explicitMods` already displays them (`Blistering Cold III`) on all 539 items, with two lines more on 315. A candidate ground-truth claim |
| a vaal gem's base skill is the source `hybrid` | 2,251 lines added | D3, ruled; the census read top-level arrays only |
| an empty line displays nothing | 362 lines | the spacer rows of an essence's description |
| a row break `\r\n` is `\n` | 407 / 256 | the reference's strings escape `\n` alone, so a template holding a CR could never be typed |
| `<style>{Display}` reduced, nested | 478 / 359 | markup the digest does not name (S4 knows the brackets): a divination card's reward, `<uniqueitem>{Staff}`, `<size:31>{…}`; 821 tags, each followed by its brace. A candidate ground-truth claim |
| `[Tag|Display]` and `[Display]` reduced | 286 / 62 | S4, C90 |
| `1,500` is one number | 1 / 1 | read as 1 and 500 it is a wrong value, silently; the template is `#x Vivid Crystallised Lifeforce` |

Deriving took 1.4 s for the 22,721 in a debug build, the parse included.

**M1 — the streaming body read (step 3, `c0a8f918`).** `/usr/bin/time -l
target/<profile>/examples/read-corpus
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

The header is 6 ms of each release read (16 ms debug): 4 ms with the
realms read from the locations alone, 6 with the listings and the
orphan exclusion. With M2's 1.4 s to derive in a debug build, an ask at
the seat's debug build is about 2 s before anything is evaluated; the
release build is what 500 ms is judged against (M3). The read's order —
location kind and id, then realm, then item id — was chosen on a
measurement: realm first sorted the whole corpus, bodies included,
43 MB resident against 10 MB and 0.13 s against 0.04 s in the `sqlite3`
shell.

**M3, M4 — a CLI ask, and `~` over all text (step 4, `28606ed4`).**
`python3 search/item-facts/scripts/m3-ask.py
search/item-facts/raw/spike-GERWARIC_7694-2026-09-13.db`, after the
three builds its header names, 2026-09-20. The script writes a fresh
`.backup` of the copy under `raw/m3/`, wraps it as a store directory
(`crates/acquisition-search/examples/copy-as-store.rs`) and asks through
`acq --json search --realm pc … > /dev/null` under `ACQ_PROVIDER=mock`
and `ACQ_STORE_DIR`: 22,623 live pc items of the copy's 22,721 (the rest
are poe2's), revision 1126, facts v7. Twice: the build at `28606ed4`,
first ask 418 ms, and the build the first two audits left (`ad5589c9`),
first ask 270 ms. Warm, the median of ten, in ms:

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
What step 4's audit fixes cost, the two release binaries asked in turn
on the same copy in the same minutes, the median of eleven: 12 to 31 ms
an ask, five to twelve in a hundred — the deriver's flag checks in
every ask and the three-valued group in those that have one.

**The class table against the census (step 6, `eeaec66e`).**
`target/release/acq --json search --realm all --count class --limit 100`
over the M3 copy (`raw/m3/store`, 22,721 items, revision 1126, facts v7),
2026-09-23, after the table was regenerated with every release state
(G3). 56 classes carried, the six largest Skill Gems 3,596, Contracts
2,641, Utility Flasks 2,337, Support Gems 2,100, Rings 1,133, Jewels
1,035; `undecided` 681, its tally *base not in the table* 388, *base
under several classes* 195, *no table for the realm* 98; the buckets sum
to 22,721 (C105). The same ask before G3 was reversed had Blade Trap
among the 393 not in the table and 686 undecided. M2 rerun at the step:
0 unread, 0 unexplained; `reqlevel` reads a whole number on every item
of the copy that has a `Level` requirement.

**M6 — the totals coverage trial (step 7, `b5d62d92`; the park fired).**
`python3 search/pseudo-stats/scripts/coverage-trial.py` over the
deriver's census of the copy (`item-facts/raw/m2/rust.json`: 22,721
items, 6,148 templates) against the shipped table (46 templates named by
a total), 2026-09-23. For each word, the templates holding it, the lines
displaying those templates, and how many a total counts — lines, since
the census counts items per (source, template) and an item carrying a
template twice, or two of them, is two incidences and one item (outside
audit, 2026-09-24); a word-based trial measures coverage, never
membership. Every row of the table is carried by the corpus.

| Word | Templates · lines | Counted by a total | Counted by none | What none counts |
| --- | ---: | ---: | ---: | --- |
| resist | 187 · 7,499 | 11 · 5,047 | 176 · 2,452 | maximum resistances (144, 133, 128), `Resistant Monsters` (123), players' and monsters' (119, 105, 105), per Alert Level (84–45), `Added Small Passive Skills also grant` (75–30), during flask effect (62, 7), minions' and allies' (47, 42, 32), exposure and penetration, conditional (`while on Low Life` 10, `while affected by Herald of …` 13, 9, 6; `when Socketed with a … Gem` 4 each), `(#-#)` descriptions |
| strength | 70 · 1,021 | 4 · 827 | 66 · 194 | `Added Small Passive Skills also grant: # to Strength` (41), `#% increased Strength` (26), requirements, per-Strength lines, `Other Item: (#-#) to Strength` (6) |
| dexterity | 54 · 1,049 | 4 · 846 | 50 · 203 | the same shapes (40, 35) |
| intelligence | 55 · 844 | 4 · 660 | 51 · 184 | the same shapes (39, 23) |
| attack speed | 70 · 1,465 | 1 · 404 | 69 · 1,061 | `Reinforcements have …` (357), monsters' (128), supported skills' (64, 49), allies' (37), during effect (34), `Grants …` (28); **`#% increased Attack and Cast Speed` (46), counted by neither speed total** |
| cast speed | 65 · 1,307 | 1 · 268 | 64 · 1,039 | the same shapes (357, 128, 71), and the 46 above |
| level of socketed | 29 · 395 | 22 · 346 | 7 · 49 | `# to Level of Socketed AoE Gems` (20) and `Duration Gems` (12): no C++ table, no total |

The verdict on the recipe's rows: every template the C++ rows name is
carried by the corpus, and the site's own answer on the ten fetched
items of q5 (`search/trade-query/data/fetch-census.json`) agrees with
`total_fire_res` on every one, worked by hand — 14 + 48 + 18 = 80,
18 + 48 + 15 = 81, 16 + 48 + 18 = 82, and the seven with one line — so
the rows shipped unchanged. What the trial found that no total counts —
the combined speed line, the AoE and Duration gem levels, the
conditional resistances — is coverage the rows never claimed; whether
any joins a recipe is the owner's (the plan's foot, T4).

**M3 at each later build** — the same command, the release build judged
against 500 ms; every release ask of every build below is under it but
one — the vocabulary whole at the audited step 7 build, measured beneath
the table, ruled at T5 and under again at `57ec6a9e`. What an ask over
budget fires is the owner's (2026-09-24): "M3 reports asks over 500 ms,
but only a load over budget triggers the projection experiment. An
evaluator cost over budget lands against the batch park." The debug
build is what the seat feels behind the README's alias. Step 4's per-ask
table is above; here the first ask after the copy is written, and the
range of the warm medians, in ms.

| Build | Where | First ask | Warm, release | Warm, debug |
| --- | --- | ---: | ---: | ---: |
| `28606ed4` | step 4 as committed | 418 | 253–271 | — |
| `ad5589c9` | step 4, the first two audits' fixes | 270 | 268–297 | 1,727–2,427 |
| `36a9738a` | step 4b, part 2 (`group.rs`) | 445 | 270–299 | 1,739–2,438 |
| `3c7d31f3` | step 4b, after the fifth audit | 491 | 266–296 | 1,729–2,441 |
| `b4d7a7d0` | step 4b, the close | 486 | 269–304 | 1,762–2,489 |
| `31b5f09d` | step 5, with its six asks added to the script | 435 | 269–305 | 1,756–2,480 |
| `6f352523` | step 5, the audit's fixes | — | 269–306 | — |
| `99bd0962` | step 5, the review's fixes | — | 270–306 | — |
| `eeaec66e` | step 6, with its three asks added to the script | 291 | 275–309 | 1,789–2,514 |
| `b5d62d92` | step 7, with its four asks added to the script | 507 | 278–359 | 1,851–2,748 |
| `4237d2f3` | step 7, the two audits' fixes | 465 | 275–360, the vocabulary whole 954 | 1,852–2,748, the vocabulary whole 6,555 |
| `57ec6a9e` | step 7, T5 | — | the vocabulary whole 316 | the vocabulary whole 2,130 |

Step 7's four asks at its build (release · debug): AQ2 as worded,
`pseudo.total_res>=60` over OQ1, 359 · 2,748; the worked example whole
356 · 2,405; `pseudo.dps>=100` sorted by it 280 · 1,867; `--count class
--sum pseudo.total_res` over the rares 301 · 2,021 — a total asked of
every item is the dearest ask yet, its 19 rows each a group asked of
each line, 80 ms over the empty query; the totals table's parse is
within the noise (the empty query 278, step 6's 275).
**The vocabulary whole at `4237d2f3`, 954 ms** (an outside review's
finding, 2026-09-24, its own medians 284 · 988 · 312 for the empty
query, the vocabulary whole and the vocabulary twice narrowed;
reproduced here as 275 · 954 · 306): step 7's own build asked the
vocabulary whole in 302, and the cost came with the second audit round
(`e63c86a8`), on whose advice `line` alone lists every computed value.
The cause, measured and not read: the same binary with that scan
skipped asks it in 323, so the scan is about 630 of the 954; it costs
by the table's rows, not its names — `line:total_res` (one total, 19
rows) 356, `line:total_fire_res` 315, `line:pdps` 279, `line:resist`
(11 totals) 524 — each row a group asked of every line of every match,
walked three times in `eval::sum`, and the limit cannot cut it since
the ranking needs every count first. The load is unchanged (the empty
query 275), so the park's candidates — a persisted projection of the
lines — would not remove this cost.
Step 6's three asks at its build: OQ1 as worded (`class:ring`) 297,
OQ5 by class and level 277, `--count class` 277 — the class table's
parse is within the noise of the empty query. Step 5's six asks at
`31b5f09d`: AQ1 276, OQ7's count by tab 274, the crossed table 274, the
sum 273, the vocabulary twice narrowed 286, the vocabulary whole 302 —
every template of 22,623 items ranked, 6,113 rows, 20 listed. A run can
have a slow half of unknown cause (the run before step 6's row: 554–621
ms release for its first thirteen asks, 277–309 for the last nine), so
a run is claimed only when its halves agree, and a slowdown's cause is
never named before it is measured.

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

The column that matters is the empty one: no undecided group, field,
flag or phrase was made true by every completion, which is where a
missed witness would show — with an unread slot read as a no again, 83
groups and 18 fields in 600 cases are, and the property refuses at once.
The rest are undecided rightly: a case the sampler cannot reach (a
template at one value, a selector nothing carries, a bound no generated
value breaks), a group no occurrence can satisfy (two quoted templates
in one and), and a comparison on an incomplete subtotal, undecided
whatever the subtotal already reaches.

**The vocabulary against M2 (step 5, `31b5f09d`).** Over all realms the
vocabulary is 6,181 rows keyed by (realm, template); the deriver's
census rows of M2 (`item-facts/raw/m2/rust.json`, 6,606 by (source,
template)) hold 6,148 distinct templates, and 6,181 less the 33
templates the copy carries in both realms is 6,148.

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
| 7 | T5 `line` alone lists no computed values, a narrowing lists those it matches, `--describe` names them (owner: "line alone lists no computed values, but a narrowing lists matches and --describe discovers names"); an ask over budget fires the projection park only by its load, an evaluator cost the totals batch park; the batch parked with its trigger, a 7b of 4b's shape, generators first | `pseudo.rs`, `counts.rs`; the reference, `--count line`; the two parks in `decisions/search.md` | `57ec6a9e` |
| 6 | G6 the grouping above class stays parked; OQ5 pinned from the owner's words with the wearable classes spelled out, the park's trigger now the seat | the park in `decisions/search.md`; `tests/acceptance.rs` | `0d3c65af` |
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
- The JSON keys beyond the worked example's: `holds` with `min`/`max`,
  `undecided` with `thing` or `term`, `const`, `has`, `is`, a range as
  `{from, to}`, `sum` and a projection as `{lines, slot}`. Reading is
  strict: an unknown or missing key is an error naming its path.
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
- Over its 12.8 days the file's logs weigh: `responses` 1,126 rows and
  2.5 MB, `item_events` 22,751 rows (22,727 the first `added`) and
  2.8 MB, `refused` 4 bodies and 135 KB, removed items 6.7 KB. Nothing
  reads a response again but the latest listing per realm and league,
  the v4 migration's re-stamp and now the highest id.
- 2,926 of 3,427 live tabs have never been fetched, the 17 folders
  apart, so "absent from a full refresh" is not a condition this
  account's store can often state.
- `Folder` is a word four consumers know — the planner, pricing, the
  CLI's counts and search — each comparing GGG's `type` for itself, and
  the store's read hands the type over verbatim as they do; the store
  saying it once is a small change if a fifth arrives or one of them
  gets it wrong.
- The header's listings are read by a scan of `responses`, which has no
  index on `endpoint`: nothing at 1,126 rows, unmeasured beyond, and it
  grows with every request. It is one of the costs the retention park's
  size trigger would see first (`decisions/store.md`, "Parked").

**Step 4.**

- Nothing on the copy is unread, so every undecided outcome — a term's,
  a sum's, the root's — is exercised by fixtures alone, as at step 2.
- The scope says `in all leagues`; a league is a term, and the owner's
  seat forgetting it is the default-league park's trigger, not a step's.
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

- A line's group that failed as stored lacked under a completion 19
  times in 2,000 cases (21 on the second day's generators); why that is
  allowed and counted is `group.rs`'s doc and the completion property's
  header.
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
- The generated properties (equivalence, completion, the cross-checks)
  generate no `class` term: it is held by hand-counted tests alone, as
  `rarity` and `frame` are. `reqlevel` joined the generators at the
  first review, its hole a `Level` row with no value.
- A poe2 body carries its class as a property (G4); the table covers
  none, and `--realm all` counts every poe2 item under `undecided` with
  that reason.
- Blade Trap (G3) is the kind of thing a first count over the owner's
  copy finds and no fixture would have: the table's rules were checked
  against the copy before the step's commit, and the check is the
  `--count class` ask of the measurement above.

**Step 5.**

- Over the copy no value outside a closed list — a rarity, a source, a
  flag GGG adds — occurs, so the counted-with-no-route path is exercised
  by fixtures alone.
- A route's spelling for a bucket under the empty query is the term alone,
  never `() term`: the router folds an empty root away, which is the one
  simplification a generated tree makes (as the selector's folding is).
- The M2 output read above is the third review's (derivation 4); the
  fifth changed how an unread part is joined to its occurrence and no
  template, so the count stands. Rerun if in doubt.
- For the owner unless he wants it: C105's first test is worded with
  groupings above class (`armour`, `weapon`), which are the parked
  category — it will be pinned with class names and the same numbers.
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
  each is not the sticky-realm park's trigger, which is the owner's.

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
- The generated properties (equivalence, completion, the cross-checks)
  generate no `pseudo` term, as they generate no `class`: a computed
  value is held by hand-counted tests alone.
- `pseudo.dps` counts every damage property the item displays and
  `pseudo.pdps` the physical alone; the C++ app reads 0 where a property
  is missing and this build says *lacked* — the one departure, C93's.
- The vocabulary whole was the one release ask over 500 ms (M3 at
  `4237d2f3`), the computed-value scan and not the load; T5 ruled it,
  and the projection park's trigger reads by the load now. A total still
  costs by the table's rows, not by the lines an item carries — 80 ms
  for 19 rows over 22,623 items — and grows with T3 and T4. What would
  make it cheap without a second maker: a group knowing at binding the
  one exact template it requires, and the evaluator asking many groups
  of an item in one walk over its lines, the single sum being the batch
  of one; held by rule 10 as a property over generated items, which
  needs the generators to reach `pseudo` first. That is the totals
  batch park (`decisions/search.md`), a step 7b of 4b's shape: no new
  surface, no rule changed, the answer's JSON unmoved.
