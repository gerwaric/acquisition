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
Nothing here is a second authority.

Reading a search change before reviewing one: the findings table below
is the checklist, and `REFRESH-SLICE.md`'s beside it for the store's
read.

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

## Findings

Every outside audit and review of the build, its verdict, then one row
per finding: the round, the commit that fixed it, the finding as the
auditor found it (its verdict where it said more than *confirmed*), and
the property, test or code that holds it now. Each finding was reproduced
by the builder before it was taken.

- **An outside audit of the build (2026-09-20), each finding verified
  before it was taken.** All three are the header failing the reference's
  scope line — `1 never fetched · location list seen 2h ago`.
- **An outside audit of the build at `28606ed4` (2026-09-20), each
  finding reproduced with a fixture of the builder's own before it was
  taken.** Verdict of the audit: keep step 4 open. Finding 7 and B2–B11
  were ruled 2026-09-20 (owner: "Otherwise agree with all you
  recommendations"); B1 the same day; the reference carries them since
  (owner, on the text proposed: "R1-12 are approved"), and a seventh
  invariant of the surface with them — that parentheses group and do
  nothing else — the builder's proposal, approved the same day. Nothing of
  step 4 waits on the owner now; what is left is 4b's.
- **A second audit, of the fixes at `ed1b5446` (2026-09-20), each
  finding reproduced before it was taken.** Findings 7 and 8 of the first
  stand as it left them.
- **A third audit, of the fixes at `ad5589c9` (2026-09-20)**, which
  found the earlier reproductions passing and two gaps more; its own check
  of counts against routes ran over 2,000 generated queries. After it the
  builder walked every function that reads a group's tree, found the rest
  recursing and one reading by position — step 1's slot check — and
  recorded it as never a wrong answer, only a missing error. The fourth
  audit showed that record wrong.
- **A fourth audit, of the fixes at `b2b8d5fb` (2026-09-20)**: both
  third-round fixes held, no defect found in `show`, three more.
- **A fifth audit, of step 4b at `ef381909` (Astra, 2026-09-21)**, the
  one this step closes on: keep `group.rs`; six defects of the first
  surface that four generated suites had passed over, none of them the
  extraction's, and two corrections. Each reproduced as a failing test
  before it was taken (`tests/fifth_audit.rs`; the CLI's `search_json.rs`
  for 5). Yield by round: 6, 5, 2, 3, 6 — and a seventh, found the same
  day by the check the audit asked for.
- **The audit's review of those fixes (`c4fc8f13`–`15fc914a`), the same
  day**: they hold; three follow-ups, each reproduced as a failing test
  first, fixed at `7f5fd962`.
- **Its review of `783461e7`, 2026-09-22**: the command and the reasons
  hold; two defects and a stale check, each reproduced first, fixed at
  `4e591ddc` and `f7318421`. Yield of the three looks at step 4b: 6, 3, 3
  — and one each day found by the checks they asked for.
- **Its last look, at `8083c465`, the same day**: the three fixes hold, no
  new matching or arithmetic defect; one defect of explanation (the row
  below). Yields of the four looks: 6, 3, 3, 1.
- **Step 4b closed** (owner, 2026-09-22, on the audit's last word: "With
  that, i think we are ready to close"). The audit's, of `b4d7a7d0`: the
  occurrence association fixes the remaining finding, no further blocker,
  "No additional abstraction or audit round is warranted by the evidence
  here. Step 5, then the owner's seat, is the right next move"; it asked
  that the fixed occurrence case stay beside the generated checks, which
  cover different failures, and it did not repeat the full gate or the
  census measurement, which the builder ran (M2: 0 unread, 0 unexplained).
  What the step closed on, against its ledger row: the properties green in
  the gate, each shown able to fail; §7's rule with its breakers (56 ok
  at the close); step 4's tests green — untouched by part 2, and step 2's
  `tests/derive.rs` re-pinned for cause at the third review; the outside
  audit, four looks, 6, 3, 3, 1; M3 (below).
- **An outside audit of step 5 at `57539f65` (2026-09-22)**: six
  findings, each reproduced by the auditor's own fixture and then by the
  builder before it was taken; all six confirmed, one wider than found.
  Its verdict: localized fixes, no new layer, E1–E5 to the owner as they
  stand. Fixed at `6f352523`; the regressions live in
  `tests/counts.rs` and the CLI's `search_json.rs`, never in a suite of
  the audit's own. Its seven assertions rerun against the fixes: six pass,
  and the seventh asserts the old continuation text is still printed,
  which is what the fix removed. M2 again: 0 unread, 0 unexplained. M3
  again, release: 269–306 ms, within noise of the step's row. The audit
  also asked that `--sum`'s operator error stop offering a line's slot
  while E2 stands: it names `ilvl, stack, or the item's sum( … )` now.
- **Its review of those fixes (`6f352523`), the same day**: 1, 5 and 6
  hold; three follow-ups on 2–4, each reproduced first, fixed at
  `99bd0962`; its three assertions rerun against the fixes pass. M2
  again: 0 unread, 0 unexplained. M3 again, release: 270–306 ms.
- **Its last look, at `99bd0962`**: the three hold, its reproducers
  pass, "no remaining blocker"; the one ignored test it saw is the
  completion property's measured half, run by hand. Yield of the three
  looks at step 5: 6, 3, 0. "The owner's seat, including E1–E5, is the
  right next step."
- **An outside review of step 6 at `eeaec66e` (2026-09-23)**: keep the
  step open — three bugs, each reproduced by the reviewer and then by the
  builder as a failing test before it was taken, and two closure gaps;
  "the fixes appear localized; I found no reason for another abstraction
  or restructuring", and it asked that the generated properties reach
  `reqlevel`, where the first two escaped. Fixed at `20558843`; the
  regressions are in `tests/class.rs`; `reqlevel` is in the generators
  (`tests/common/generated.rs`), its hole the first finding's shape, and
  the completion property catches that finding undone — tried.
- **Five blind seats at `f357de39` (2026-09-22)**: Sonnet, one question
  each over the owner's real store, read-only under `ACQ_NO_SPAWN=1`, the
  tool's own `--help` and `--describe` their only sources; the brief and
  the five questions are in the message of the commit that added this
  entry. Every answer matched the builder's reference command, in 3 to 6
  invocations; none needed a stronger model, none read the repository.
  Two findings in the CLI's surface text, fixed at `2cbf7787`; what the
  seats said beyond them is data for the owner's seat ("Observations
  still open", step 5).

| Round | Fixed at | # | Finding | Held by |
| --- | --- | --- | --- | --- |
| 3, audit | `c0a8f918`, before the step's commit | 1 | A folder is a live row no fetch ever fills, and the header could not tell it from a tab never fetched (17 on this copy) | `LocationRow::tab_type`, GGG's `type` verbatim as the snapshots carry it; the fixture's folder with a fetched child |
| 3, audit | `c0a8f918`, before the step's commit | 2 | A substash whose tab a listing retired keeps its row (the planner's orphan report) and was listed as a live, never-fetched location. The builder had seen it and followed `read_tabs` without saying so | excluded: a location is live with its parent (C54); the fixture, which fails without the exclusion |
| 3, audit | `c0a8f918`, before the step's commit | 3 | A location's `listed_at` cannot say when a list was seen: an empty list has no rows, and a substash's is its parent's fetch | `CorpusHeader::listings`, the bases the snapshots cite, read in the same transaction; the fixture's empty character listing |
| 4, audit 1 | `ed1b5446` | 1 | A route whose query starts with `-` is refused by clap, exit 2: every `-has:`, `-is:` and `-line(…)` route, which is most lacked and many failed ones — the plan's rule 5 broken. The CLI's route test had not met one: its fixture had no lacked count | the query after `--` (B9); `tests/search_json.rs` runs a lacked route through a shell, and fails without the separator — tried |
| 4, audit 1 | `ed1b5446` | 2 | A flag that could not be read answered as a no: a line with unread `flags` matched `-is:crafted`; `"corrupted": "unread"` matched `-is:corrupted`; and `"corrupted": false` beside an unread `influences` was undecided. The builder had recorded the line's case as an observation and called it harmless; it was a false match against C93 *Confirmed, and wider: half of it was step 2's deriver, which recorded no unread at all for a flag's value that is no boolean* | the deriver: `ITEM_FLAGS` values checked, `Line::flags_unread`, `Part::Flags`; the evaluator: a line's group three-valued on each occurrence, a flag's own key and `influences` for the six that live there, a phrase left open only by what holds a displayed string. `tests/unread.rs`; M2 rerun, 0 unexplained and 0 unread. The fix opened an edge of its own, closed in the same change: a group whose selector asks a flag can fail on an item whose occurrence leaves the selector open, and the failed route `line(S) -term` would not return it — for such a group the route is `(line(S) or undecided(line(S))) -term`, elsewhere as short as the reference's |
| 4, audit 1 | `ed1b5446` | 3 | An unread `hybrid` was read as the absence of hybrid lines: `-line(source=hybrid)` admitted the item | `unread_lines`; `tests/unread.rs`, which fails without it — tried |
| 4, audit 1 | `ed1b5446` | 4 | Parentheses changed an outcome: `source=explicit` ruled the unread implicit array out only as an immediate member of the group's and | which sources a group admits is asked of its meaning — the group with its source tests answered and all else unknown (`eval.rs`, `admits`); five spellings pinned against three that must stay open |
| 4, audit 1 | `ed1b5446` | 5 | What a selector resolved to was filtered by the group's comparisons, so a template whose value failed was not listed, and a `sum`'s selector resolved to nothing | resolved by the group's selector over the scope, a `sum`'s too; the worked example's count moved from 6 to 7 for that reason |
| 4, audit 1 | `ed1b5446` | 6 | Following an undecided route returned the members without their reasons, so past the ten listed there was no way to them | an `undecided( … )` that matched shows the reasons of what it asked about (`Evidence::Undecided`); pinned over twelve items |
| 4, audit 1 | `7678ae27` | 7 | The basis names the account and not the store, against C98's words; two stores of one account gave equal bases and `is_current` said yes across them *Confirmed — and the owner's: which identity names a store (its path, the world's id of C83, an id the file carries) is the basis as the contract, one of the six lines the seat revisits first* | **ruled 2026-09-20: "(a') now and park (c)"** — twelve hex digits of the SHA-256 of the file's canonical path, as C83 names a world: no path in an answer, no migration. `Basis::of`, and `is_current` compares the whole basis; the auditor's two stores pinned, and the check without the store tried and caught. An id the file carries is parked with the refetched file (`decisions/search.md`) |
| 4, audit 1 | `7678ae27`, `774620b6`, `f5701893` | 8 | Rule 4 says an unstated rule that changes what a user types stops the step, and the record listed eight such holes and said none blocked *Narrowed: the step was built as steps 1 and 2 were — in the direction that breaks least, the holes brought to the owner at the close, said before building — but a recommendation is no ruling, and the step is not closed until each is ruled and the reference carries it* | closed: B1–B11 ruled 2026-09-20 (`7678ae27`, `774620b6`) and the reference carries them (`f5701893`) |
| 4, audit 1 | `ed1b5446` | — | `eval::texts` and `answer::item_texts` were one accessor written twice | one, in `eval.rs` |
| 4, audit 2 | `ad5589c9` | 1 | The selector dropped a whole member of the group's and when any comparison sat inside it, its template and source restrictions with it: a lacked count of 1 whose route returned 0, and a nested group resolving to templates the unnested one did not *Confirmed: the first audit's finding 4 over again — source eligibility had been moved from syntax to meaning and the selector left behind, the second of the six places the builder had said he trusted least* | the selector is the group with each comparison taken as favourably as it can be — true, or false under a not — and folded (`bind.rs`, `selector`), so the ordinary route is still the reference's. Five nested spellings in the route property. A mutant that ignored the not survived the routes, which the builder put down to their being made from the selector — true of his seven hand-made items, and not in general: the generated routes of step 4b catch that mutant; the failed-against-lacked split of one item is stated by hand as well — then caught |
| 4, audit 2 | `ad5589c9` | 2 | An unread flag erased the known no beside it: with `{"crafted": false, "fractured": "unread"}`, `-is:crafted` was undecided, and `{"shaper": false, "hunter": "unread"}` left `-is:shaper` open *Confirmed: the first fix was per object and had to be per flag* | `Line::flags_unknown` names the flags of a readable object that could not be read, `flags_unread` is the object itself; an influence is unread under `influences.<flag>` |
| 4, audit 2 | `ad5589c9` | 3 | The sort scalar ignored an occurrence the group may select: 20 shown as the largest beside an open 95 *Confirmed, and wider — a source the group admits and could not read may hold a larger one too, which the first build had as well* | `Scalar::Incomplete`: what was readable, its status, no place in the order (B10). An open occurrence that cannot pass the largest changes nothing, pinned |
| 4, audit 2 | `ad5589c9` | 4 | `DERIVATION` stayed 1 though the same malformed body now derives to another item — against the rule written on the constant | 2 |
| 4, audit 2 | `ad5589c9` | 5 | The timing note claimed more than it had: an unchanged reader running slower shows the machine changed, never that the evaluator had not *Confirmed — and the builder's cause was wrong as well as unproven. The slowdown was put down to battery and low power mode because that was found first; on the same battery in the same mode, an hour on, the pre-fix binary ran at the morning's 257 ms. What slowed the machine is not known* | the two builds measured against each other (M3, below); the claim that a release ask is over 500 ms in that power state is withdrawn |
| 4, audit 3 | `b2b8d5fb` | 1 | An occurrence the group may select, and which names no such number, left a value open: over one line `Cannot be Frozen` with an unread `crafted`, `sum(… .arg1)=0` was undecided and `undecided(… .arg1)` matched, though either reading of the flag gives a complete zero and no occurrence. And binding `undecided( … )` of a value threw its slot away, a sum and a projection made one probe | an open occurrence counts only where it names the slot — in a sum, in the largest, and in the together count, which had the same fault and was not in the finding; `undecided( … )` of a value binds as `--sort` does and is open exactly when the value would sort as incomplete, one function for both (`eval.rs`, `scalar`) |
| 4, audit 3 | `b2b8d5fb` | 2 | Parentheses changed whether the together count applies: the bound was looked for among the group's immediate members *Confirmed: the third time one fault — a meaning read off the syntax (source eligibility, the selector, now this)* | the bound is the one comparison among the group's conjuncts, through every nested and and a doubled not; four spellings pinned to one count, one route and one selector, and four shapes that are no single lower bound pinned as not applicable |
| 4, audit 4 | `c3de464d` | 1 | B11 breaks rule 5: `line(("# to maximum Life" source=explicit) arg3>=0)` was accepted, counted one item together, and printed a route — `sum(line("# to maximum Life" source=explicit).arg3)>=0` — that the same build refuses *Confirmed; the builder's note of the round before was wrong* | the slot check reads the group's conjuncts, one function with the together bound's (`template::conjuncts`), so the query is step 1's `slot_unknown` however it is parenthesised; and the together route is checked before it is offered, since folding can bring a template to the selector's and that the whole holds under an or — `("T" or false()) arg3>=0`, found while fixing, and caught only by that guard |
| 4, audit 4 | `c3de464d` | 2 | The zero block said a group resolved to nothing while the terms block listed what it resolved to: `nothing()` took the group's first template test and asked it alone, out of its and, or and not. A `sum`'s selector was never diagnosed *Confirmed: the fourth reading of a group's meaning kept apart from the others* | resolved to nothing is the bound selector picking no occurrence in the scope, a `sum`'s too; a field's term is as before |
| 4, audit 4 | `c3de464d` | 3 | Sorting by an unread number said `no satisfying occurrence`, a known absence, where `undecided(ilvl)` said unread of the same item | `Scalar::Incomplete` for a field that could not be read |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | 1 | A sum of decimals depended on the order of occurrences: 0.1, 0.2, 0.3 summed to 0.6000000000000001 one way round, so `sum( … )=0.6` matched one of two items carrying the same lines *Confirmed, and wider: the mean of a ranged pair had it too (`avg=0.15` over 0.1 and 0.2)* | `exact.rs`: arithmetic on whole hundred-thousandths, in integers, read back as the nearest float, which is the float a typed bound is; one function for a sum, the together count and `avg`. No rounding anywhere. Its first form, a decimal sum with a float fallback, is follow-up 1 below |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | 2 | A suggestion counted one spelling and offered an any-case `=` that returns every spelling: said 1, returned 2 | a suggestion's count is its term's, asked of the scope as it will be typed; a second spelling of a value already offered is the same term and is not offered again |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | 3 | A pattern that resolved to nothing was missing from the zero block: being listed was coupled to having words to suggest by | listed with no suggestion — a group's, a `sum`'s, a field's |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | 4 | What a row shows of one term was unbounded for a group and a sum (40 lines) and cut to three for a phrase without saying so (invariant 5) | six, a sum's value beside them; `left_out` counts the rest and the CLI prints it with `acq show <id>` |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | 5 | Under `--realm all` a row did not say which realm its item is in | the renderer: the realm before the place, as `show` prints it; under one realm the scope line has said it |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | 6 | `--describe` did not say how terms compose or what has a value, and refused the slot and operator words its own doc promised | two blocks, `composition` and `values`, an entry a line and an example, every example bound by a test; an entry answers to each word of its name, the positional slots to any `arg<N>`, a block to its own. The reference stays the manual |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | 7 | — found by the both-ways zero-block check on its first day: a closed set's `:` selector that matched nothing (`rarity:ma` over rares) showed an empty resolved list and no zero-block entry *The builder's, the same fault as 3* | listed; pinned beside 3 |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | — | The record said sixteen completions: there are fifteen, sixteen bodies with the stored one | this record |
| 4b, audit 5 | `c4fc8f13`; the properties `6bf844d2`, `3c7d31f3` | — | The resolved-values continuation always said `--count line` | `--count <field>` for a field's values |
| 4b, review 1 | `7f5fd962`; the anchors `1e951803` | 1 | `exact.rs` fell back to floats when its integers overflowed, and whether a sum fell back depended on the order its occurrences cancelled in: one order of 1e28, 1e28, −1e28, −1e28, 1e-10 gave 0, another 1e-10 | a number is read once, where the body is read: ten whole digits and four decimals (measured over the census copy: 212,233 displayed numbers, none past two decimals or ten whole digits; the owner: "I have never seen more than 4 digits after the decimal"). One beyond it is unread (C93), so nothing downstream has a fallback; 48 lines of code became 21. M2 again: 0 unread, 0 unexplained. Its first form — the line's numbers cleared, its array unread — was too blunt twice over: the third review, 1 |
| 4b, review 1 | `7f5fd962`; the anchors `1e951803` | 2 | The reasons an `undecided( … )` shows were cut to six in the order the terms were written, so reordering a conjunction changed which was dropped — and the fixture, passed through the equivalence checker, was refused *Confirmed, and wider: the total's undecided items listed their reasons with no bound at all* | `eval::why`, one function for both: the bound is on the item's unread parts, the first six in the item's own order, every term's pair with each kept; `why_left_out`. The audit's fixture is a fixed case of `generated_equivalence.rs` |
| 4b, review 1 | `7f5fd962`; the anchors `1e951803` | 3 | The `acq show` continuation dropped the account: with a second account known it does not run *Confirmed, and wider: both offers of `show`'s errors dropped it too* | `answer::command`, the one way the search prints a command — routes, the continuation, the offers. The CLI's rule-5 test runs every `acq …` a text prints, anywhere in a line, with two accounts known; the walk it replaces saw only lines that start with one |
| 4b, review 2 | `4e591ddc`, `f7318421` | 1 | A number the search does not read was made an absent slot and an unread array. Absent: its comparison was a no, and a not made a witness of it — `line("# to Spirit" -arg1>=0)` matched `1.12345 to Spirit`. The array: `-line(template:life)` and a sibling number that was read were left open though neither needs it *Confirmed; the per-flag fix of the second audit was the precedent, and was not followed* | `Line::numbers` is `Vec<Option<f64>>` and `Line::slot` returns `Slot` — absent, unread, or a number; a comparison on an unread slot is undecided, as a flag is (`group.rs`); `Part::Numbers`; a sum, a largest and the together count ask one question of an occurrence, `leaves_the_slot_open`. `DERIVATION` 4. Step 2's `tests/derive.rs` pins the new types at twenty places. M2: 0 unread, 0 unexplained |
| 4b, review 2 | `4e591ddc`, `f7318421` | 2 | A total past 2^53 units was divided as a float and rounded twice: nineteen of `9999999999.9997` did not equal `189999999999.9943` | the integer total is written out as its decimal and read by the parser a typed bound is read by |
| 4b, review 2 | `4e591ddc`, `f7318421` | 3 | The evidence cross-check still refused more than seven entries, and `answer.rs` still promised six reasons, after the bound became six unread parts | the checker counts parts; eight terms on one part is a fixed case; the doc says what `left_out` counts |
| 4b, review 2 | `4e591ddc`, `f7318421` | 4 | — found by the completion property on its first run over generators that write an unread number, and not again in six: the together count took an item with no occurrence that counts, since a sum of nothing is 0 and 0 is at least any bound of zero or less; its route, which asks for a selected line, returned 11 of 12 *The builder's; nothing to do with an unread number* | only occurrences that count reach a bound together; pinned by hand, since the generators reach it seldom |
| 4b, review 3 | `b4d7a7d0` | 1 | A reason was joined to an open occurrence by its *source*, so six unread Spirit lines before the one Life line that mattered filled the six parts shown and the relevant reason, `explicitMods[6]`, was the one left out — one defect of explanation, no new matching or arithmetic defect. *Confirmed, on both paths* | an `Unread` of a line's flags or numbers says which occurrence it is of (`Unread::line`), reasons are chosen by occurrence, and by what the group asked of it — its flags where a flag is asked, its numbers where a number is — before the six-part cut. `DERIVATION` 5. The audit's fixture is `tests/fifth_audit.rs`'s last case; a generated check holds the second half (a reason is of what its term asked) and refuses the old join |
| 5, audit | `6f352523` | 1 | A tab bucket was its id alone: one id under two leagues and two realms was one bucket, labelled with the first's name and league *Confirmed — the store's own identity is the coordinate (C54), and the refresh slice met one id under two realms* | `counts::TabAt`: realm, league and id; the route `id:<id> league=<L>` over its realm; the label carries all three, the text the id. A crossed cell's route names the tab's realm |
| 5, audit | `6f352523` | 2 | `--count`'s key parser stripped quotes before reading syntax: `line:"Life` was taken, `line:"~Life"` became a pattern, any escape was accepted | per-character quoting: what was quoted is text as written, a quote that never closes or an escape outside the language's three is a `view` error; `~` outside the quotes still marks a pattern |
| 5, audit | `6f352523` | 3 | The resolved-values continuation said `--count tab lists them` and could not: a count by tab groups by the tab, the selector resolved to names. And wider, as the audit said: a count *under the query* is not the selector's domain — `base:ring rarity=unique` resolves rings of every rarity | `Resolved::rest` is a route: the term alone over the scope, counted by its field, or a group's one template test as `line:<text>` / `line~<pattern>` (`Group::sole_template_test`); none for `tab` (E1) or a selector of more than one test, and the text then says the count and no command. `Route::command` prints a view. The CLI test runs the printed command and holds it to every value |
| 5, audit | `6f352523` | 4 | A bucket's sum of item totals was rounded again: `units` multiplied a float by 100,000, past 2^53 for a total; 23 × `9999999999.9997` summed to `.9931` on the item and `.99313` in the bucket *Confirmed; within the declared rule* | `exact::units` reads the decimal the float prints as — the decimal it was made from — so units are exact through every level of adding; pinned in `exact.rs` and over two items and an incomplete subtotal |
| 5, audit | `6f352523` | 5 | A vocabulary flag `Crafted: 1` routed to `is:crafted` and returned 0: the count matched the flag any-case and the evaluator exactly *Confirmed, and older than step 5: `is:` and a line's `is:` were bound any-case (B2) and compared exactly since step 4; the census's flags are all lower-case, so no real item met it* | one any-case compare in the evaluator (`group.rs`, `eval.rs`), and the vocabulary counts a kind by its legal spelling — `Crafted` and `crafted` one kind of two, routed to both; a spelling outside the list counted with no route and `needs` |
| 5, audit | `6f352523` | 6 | The text printed a crossed cell with no route as an ordinary row, and a kind's `needs` never | `cross_text`, `bucket_text`: each value or kind with no route says why, once |
| 5, review | `99bd0962` | 1 | Exactness ended at the item: units were read back off the float an item total became, and a total's decimal can be longer than a float prints — nine means of large pairs, their negatives and `0.0001` summed to `0.00011` split over two items, `0.0001` on one *Confirmed; the builder's claim in `exact.rs`, that a float's shortest print is the decimal it was made from, was false past fifteen significant digits* | `exact::Exact`, units carried: an item's sum, a bucket's sum of them and a sort scalar are `Exact`, and a float is made only to compare or print. Pinned in `exact.rs` and in `tests/counts.rs`, the review's own arrangement |
| 5, review | `99bd0962` | 2 | A continuation's key was shell-quoted and not list-quoted: `--count 'line~Marker [a-z]{1,2}'` split at the comma | `answer::listed_text`: a text with a comma, a quote, a backslash, a row break, a leading `~` or whitespace at an end is quoted in the list's grammar with the language's escapes; the CLI runs continuations for a pattern with a comma and texts with a comma and a backslash, and a unit test reads every spelling back |
| 5, review | `99bd0962` | 3 | The binder trimmed a narrowing text: `line:"Life "` became `line:Life` | `bind_key` trims a name and never a text; pinned at the terminal |
| 6, review 1 | `20558843` | 1 | A `Level` row with no value read as absence, so it matched `-has:reqlevel`; several values, or two `Level` rows, silently took the first number | `derive::read_reqlevel`: one row, one value, a whole number, and every other shape unread under `reqlevel`; pinned over five items |
| 6, review 1 | `20558843` | 2 | Every unread element of `requirements` left `reqlevel` open: a read `Level 10` beside a malformed `Str` made `reqlevel>30` undecided while `--sum reqlevel` summed a complete 10 *Confirmed: the unread was recorded a grain too wide (rule 8), and the sort and the sum read the number where the comparison read the part* | the deriver says under `reqlevel` when the `Level` row may be among what was lost — the array not an array, an element unread whose name is `Level` or unreadable — and the evaluator asks that key alone, so a read `Level` is a witness and the three consumers agree |
| 6, review 1 | `20558843` | 3 | A class error's readings were spelled by hand: `class:bodyarmour` offered `class=Body Armours`, which does not parse (rule 5 for readings) | `bind::closed` offers through the printer, which quotes a value with a space; a test binds every reading a class error offers |
| 6, review 1 | — | 4 | OQ5's test still pins the plan's wording — three wearable classes and a bracket, a flask a distractor — after the owner's definition changed the reading *Narrowed: the record never claimed OQ5 covered, and the owner's line is pending (G6)* | the test's doc says it pins the constructs and moves to his words when he rules; nothing built for it, as the review agreed it need not be |
| 6, review 1 | `20558843` | 5 | The reference's worked example still used the illustrative `class=ring` and C105's example the parked `armour`/`weapon`; the ledger row carried the table's numbers from before G3 was reversed (81, 4,483, 213 against 82, 4,547, 225) | `search/DESIGN.md` says `class=Rings` and `Rings 6, Wands 3`; the row says what the shipped table holds |
| 5, seats | `2cbf7787` | 1 | The `--realm` help read as optional, and all five seats omitted it once over a two-realm store; C96's refusal, which lists the choices, recovered each in one step | the help says required over several realms, one of them or `all`; `CLI-REFERENCE.md` |
| 5, seats | `2cbf7787` | 2 | The scope line's `2940 never fetched` was read as items by four of five; C96 says locations never fetched, and the render had dropped the word | `2940 locations never fetched`, `1 location` for one; the `search_json` needle |

## What the measurements taught

**Step 1's own tests (`080a8581`).** In the builder's code: a computed
value named alone (`pseudo.total_res`) was answered as a bare word; a
whole number between 9.0e18 and 2^63 printed as digits the parser read
back as another tree (`Number::from_f64` now agrees with the parser at
the edge of an i64, pinned); `ilvl >= 84`, `tab:q-20` and a template
typed with two numbers each got a misleading error and now get their
own.

**M2 — the deriver against the census (step 2, `ef720323`).** `python3
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

The header is 6 ms of each release read (16 ms debug). It was 13 ms
while the realms were read from `items` as well as from `tabs` and
`characters`: a scan of the largest table for a value its locations
already give — 4 ms without it, and 6 with the listings and the orphan
exclusion the step 3 audit added (Findings). With M2's 1.4 s to derive in a
debug build, an ask at the seat's debug build is about 2 s before
anything is evaluated; the release build is what 500 ms is judged
against (M3).

**Departures from the first showing of the read (the plan at
`2d25cb03`), each the builder's.** No membership and no `removed_at`:
removed items are not read until `all` is built (gap 3, ruled), and a read that
handed them over would change under a prune with no response written
(owner, 2026-09-20: retention starts as a verb, "which means we can
think about triggers for that verb later rather than now"). The order is
location kind and id, then realm, then item id: realm first sorted the
whole corpus, bodies included (43 MB resident against 10 MB, 0.13 s
against 0.04 s in the `sqlite3` shell). The realms are those the file
has a tab or a character under (owner, 2026-09-20: `pc`, `xbox`, `sony`
and `poe2` are "the full list").

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

What the audits' fixes cost was measured apart, the two release binaries
asked in turn on the same copy in the same minutes (the pre-fix one built
from a worktree at `28606ed4`), the median of eleven: the empty query 257
against 269 ms, OQ1 277 against 295, AQ2 277 against 301, a flag asked
inside a group 262 against 293, `~` over all text 264 against 279 — 12
to 31 ms, five to twelve in a hundred, the deriver's flag checks in every
ask and the three-valued group in those that have one.

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

**M3 at each later build** — the same command, the release build judged
against 500 ms; every release ask of every build below is under it, so
the persisted-projection park's trigger has not fired. The debug build
is what the seat feels behind the README's alias. Step 4's per-ask
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

Step 6's three asks at its build: OQ1 as worded (`class:ring`) 297,
OQ5 by class and level 277, `--count class` 277 — the class table's
parse is within the noise of the empty query (278 against 270 at the
step 5 rows). The run before this one, on the same copy the same hour,
gave its first thirteen asks at 554–621 ms release and 3.1–4.9 s debug
and the last nine at 277–309: what slowed the machine for its first
half is not known, as at the second audit's finding 5, and the second
run is what is claimed.

At `36a9738a` each release figure was within 9 ms of the audited build's
column, taken the same day (OQ2 the furthest, 269 to 278); the two builds
were not run against each other, so that part 2 cost nothing is not
claimed. Step 5's six asks at `31b5f09d`: AQ1 276, OQ7's count by tab
274, the crossed table 274, the sum 273, the vocabulary twice narrowed
286, the vocabulary whole 302 — every template of 22,623 items ranked,
6,113 rows, 20 listed.

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

**The mutants (step 4b)** — each a fix of step 4's audits undone, or a
fault of the same kind; tried against the three generated properties
before part 2, and A–E, G and H again after it, in `group.rs`, with the
same result. `·` survived; blank, not tried.

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

The generators write a number with five decimals, which a completion
fills; with an unread slot read as a no again the completion property
refuses at once, shrunk to `line("# to maximum Life" arg1>=0)` over
`+0.12345 to maximum Life`.

**Why the suites had passed over what the fifth audit found**, which is the audit's other half:
the generators made whole numbers only and never reordered an item's
occurrences, though transformation 4 asks for it and this record had
claimed 1–4 covered; the zero-block check looked only at entries
present, so an emptied block passed it; a suggestion's count is named
`items`, which no walk over `count` sees; nothing bounded a row; and no
property reads the CLI's text or `--describe`, as the step 4b observations below
already said. The generators have decimal lines and bounds, a second
spelling of one line and selectors that resolve to nothing; the
cross-checks ask every occurrence in another order, the zero block both
ways, every suggestion's term, a row's bound, and two misspellings of
every corpus; negative controls refuse an emptied zero block, a
miscounted suggestion and a doubled row. Each fix undone is caught by
them: P a sum in float order, Q a suggestion counting its spelling
(survived 192 cases until the misspellings were asked of every corpus),
R a pattern dropped from the zero block, S a row unbounded.

**The vocabulary against M2 (step 5, `31b5f09d`).** Over all realms the
vocabulary is 6,181 rows keyed by (realm, template); the deriver's
census rows of M2 (`item-facts/raw/m2/rust.json`, 6,606 by (source,
template)) hold 6,148 distinct templates, and 6,181 less the 33
templates the copy carries in both realms is 6,148.

**Every property passed at its first run, and each time that was a
signal.** The fold property was written over one pool of characters drawn
at random for both texts, and the fold swapped for `to_lowercase` survived
4,000 cases: a pair like `s` and `ſ` was never drawn. Both texts are now
drawn from one case family at each place, and two mutants — `to_lowercase`,
and ASCII folded down where the rest folds up — fail within a few cases.
The exact pattern's mutant (only `\` escaped) failed at once, at `(`.

## Holes ruled, and where the rule went

What the build met that the reference did not state and the owner
ruled; none is unruled today. A ruled hole keeps its evidence here and
its rule in the registry's cited contract detail or implementing module
doc.

**Step 5 — the owner's verdicts, 2026-09-22.** The accepted behavior
already exists; these clarify C95 and C105, whose implementation is
`crates/acquisition-search/src/counts.rs` ("As built") and
`View::of` in `answer.rs`, with E2 in `bind_sum` in `bind.rs`.
The terminal reference is `search/DESIGN.md`.

| # | Question and evidence | Owner's verdict, verbatim |
| --- | --- | --- |
| E1 | What a value bucket's term selects. Case-insensitive equality can select two separately counted spellings; a tab's name can select several tabs or a substash. The accepted links select the counted spelling or the tab's full coordinate (realm, league, id), including its substashes. | "accepted" |
| E2 | What `--sum` takes when one item has several matching lines. Reusing the sort's largest occurrence would contribute 75 from lines of 20 and 75; the accepted rule rejects a raw line projection and offers the item's explicit `sum( … )`, which contributes 95. A pseudo value already has its own definition of how contributors combine; its implementation waits for step 7. | "ok, agreed" |
| E3 | Whether the vocabulary's `undecided` bucket includes an item with both a readable selected line and unread lines. The accepted meaning is uncertainty about presence, not a list of every incompletely read item. | "agree to keep" |
| E4 | Which view options combine: counts with row sorting, a sum without a count, or both count shapes. Keep the existing errors and table limits; no standalone grand-total view is needed. | "I don't fully understand these options, so i accept your recommendation to keep these as errors for now. For sum, we don't need a grand total." |
| E5 | Whether `line` can be a key in a crossed table. Keep it in its own vocabulary table; crossing two ordinary fields remains supported. | "accepted." |

All five E-numbered holes are ruled. No runtime behavior changed with
these verdicts.

**Step 6 — each changes which items `class:` matches, so they are the
owner's.** None blocked the step: each is built in the direction that
breaks least if he rules the other way, and every one is a rule of the
table's generator (`tools/class-table.py`, its docstring) or of
`class.rs`, so ruling the other way is a regenerated file or one arm.
The park the step fires — a grouping above class — is G6, the acceptance
task the plan names before OQ5 counts as covered.

| # | Hole | Built as | Recommendation |
| --- | --- | --- | --- |
| G1 | The class vocabulary. The reference's names are illustrative (`class:ring`, `class:staff`); three GGG-authored surfaces name a class — the clipboard's `Item Class: Rings`, an item filter's `Class "Rings"`, the trade site's category `Staff` — and the export carries the first two's spelling as each class's display name (repoe F3: 46 of 46 listed bases agree) | the game's display names, plural: `Rings`, `Staves`, `Warstaves`, `Body Armours`; `:` picks by the word, so `class:ring` is `Rings` and `class:sword` three classes, `=` names one in any case; a word no name holds is an authoring error with the near names, and `class:staff` — three edits from `Staves` — is offered the whole list of 81 | keep: the names are the game's, not the builder's, and `--describe class` and `--count class` list them; the trade site's singular words belong to the category park (G6) |
| G2 | A base the source lists under more than one class: 31 names — the Maven's and the Eldritch invitations (`Quest Items` and `Misc Map Items`), the breachstones, the resonators, `Energy Blade` (three) — 300 items on the census copy | listed under each; the item is undecided, *base under several classes*, naming them; the table never chooses | keep: a rule that picks one is a reading of the source the source does not make (C106); if the owner names the class an invitation is, it is one reviewed row of a later table version |
| G3 | The export's release states: 54 `unreleased` bases, 9 `legacy`, 15 `unique_only`. The first draft excluded `unreleased` as what no stash holds, and the copy refuted it in its first count: the owner holds a Blade Trap in a Remove-only tab, a gem the game removed and the export marks `unreleased` | every state enters: a stash keeps what the game removed, and a release state is the export's history, which the search never judges (C107) | keep |
| G4 | The realm `poe2`: the export is PoE1's, and a poe2 body carries its class itself, as `properties[].type == 109` (item-facts F5; 98 items on the copy) | no table for the realm: every poe2 item undecided with that reason (C94's rule for a realm with no recipe, C101's for no table); nothing read from the body | the PoE2 park's December trigger, not this step's: reading the body's class for one realm is a second deriver of one fact (C103) and is the owner's |
| G5 | The licence. RePoE's `LICENSE.md`: the generated data is GGG's and "shall not be used or published without being in accordance with their terms of use" (`SURFACES.md`, the RePoE row); C106 says a surface's licence is read when its first definition needs it | the table carries base names and class names — the words the trade site, the wiki and every item filter publish — with the source cited on the file and the row; the C++ app on `master` fetches the same export at runtime with no attribution | the owner's read; nothing else in the table is GGG's beyond those names |
| G6 | **The category park, fired**: OQ5 as the owner asked it — "my leveling gear" — names a grouping the class table has no word for (`armour`, `weapon`, `equipment`), and C105's first test as worded counts by `armour` and `weapon`. Note 31 K7: the site's 14 parents are unions of leaves, seven leaves need base-name rules, and "which categories count as equipment" is a definition nobody has given | not built: OQ5 is askable by class names — `(class:boots or class:gloves or class:helmet)` — and pinned so; C105's test pinned with class names and the same numbers | the owner's acceptance task before OQ5 counts as covered: which convention (C106) `category` follows — the trade site's category ids as its authority, generated as repoe F3 generates them, or a smaller equipment grouping of his own — and whether it is a second field beside `class` or a value `class:` also takes |

G1–G5 stand as built (owner, 2026-09-23, verbatim: "Otherwise, accept
all."). G6, his words verbatim the same day: "leveling gear is generally
not defined by class, except indirectly. leveling gear is generally
items that can be equipped at a low character level with basic
resistance, life, and damage modifiers (although some leveling builds
have some additional specific needs I have not mentioned). I'm not sure
how that impacts us here." The builder's reading, a recommendation and
no ruling: two of the three parts exist — `reqlevel=..30 or
-has:reqlevel`, and the lines by template — and the third, *can be
equipped*, is what nothing names: frame does not say it (a contract is
frame rare, a map and a gem carry damage lines), and the class says it
indirectly, as a spelled-out list of the wearable classes today, or as
the category park's one word, whose definition his sentence supplies
(equipment is what can be equipped: the wiki page C106 cites, the trade
site's parents). Recommended: pin OQ5 from his words with the class list
spelled out, and re-set the park's trigger to the seat showing that
typing the list bites — adding `category` later breaks nothing. A flask
is no distractor under his definition; whether it is leveling gear is
his.

**Step 1 — what the parser met that the reference does not state.** None
blocked step 1: each is built in the direction that breaks nothing if he
rules the other way.

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

**Step 2 — they change which items a query matches, so they are the
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

**Step 4 — each changes what a user types or which items it matches, so
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
| B8 | Gap 2, as the plan recommended | (b): `--query-file <file\|->`; a route is printed shell-quoted, an apostrophe as `'\''`. (c) closed 2026-09-23 (gap 2, below) | — |

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

**The basis's store** — the first audit's finding 7 (above): ruled 2026-09-20, "(a') now and park (c)"; the reference carries it (`f5701893`), the park is `decisions/search.md`'s.

**Step 4b — three rules the builder had built in the direction that
breaks least, which the reference did not state**, are in
`search/DESIGN.md` since the close (`2b01b1bf`; owner, 2026-09-22: "Add the
rules"; the wording the builder's): what a number is and what one written
longer becomes (*Slots*), that nothing reaches a bound together without
an occurrence that counts (C92's detail), and what a row shows of one
term (C100's); the synopsis's `--describe` line says what it prints now.

**Gaps found while planning, ruled** — each changes what a user types,
or which items what he types matches; 4, 5 and 6 came from an outside
review of the plan (2026-09-19); 2 and 3 were ruled 2026-09-23. The
rule is in `search/DESIGN.md`'s reference; the evidence is here.

1. **A line break inside a template — ruled; the reference, *Strings*.**
   540 of the census's 6,928 templates are one mod displayed over
   several rows. M6's coverage trial counts such templates too.
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
2. **An apostrophe at a shell — (b) built at step 4; (c) closed
   2026-09-23.** 306 templates carry one, and so does the reference's own
   `name="Kaom's Heart"`, which cannot be typed plainly inside the
   shell's single quotes. (a) nothing: the shell's `'\''`; (b) the
   adapter reads the query from stdin or a file; (c) the language also
   accepts `'…'` strings, printing `"…"`. *Recommendation:* (b) at step
   4, and (c) left for the seat — adding it later breaks nothing. Built
   so at step 4 (hole B8). (c) closed on the builder's read, owner
   2026-09-23, verbatim: "Yes, close (c)". The read: a single-quoted
   string would need an escape for the very apostrophe that motivated
   it; the shell's double quotes with the language's `\"`, `--query-file`,
   and every printed command shell-quoted already answer it; and `'` as
   a delimiter would reach bare words and the `line:` keys, for one
   surface only. If typing an apostrophe by hand bites at the seat, the
   fix is the adapter's or the help's, never the grammar's.
3. **The values of `membership` — ruled 2026-09-23.**
   The request carries it, the terminal synopsis has no word for it,
   and `live` is the only value shown; `acq items search --removed`
   exists today. *Recommendation:* `live` alone at the first seat; `all`
   (live and removed, each row marked) at step 11, so the old verb can
   retire; `removed` alone waits for someone asking. Step 4 builds `live`
   alone, which is all the store's read hands over. Ruled on the
   builder's line — membership is a scope value, `live` by default and
   `all` on request with every removed row marked; `removed` alone waits
   for a question; the prune verb advances the revision — owner,
   2026-09-23, verbatim: "accepted." The case that decided it is an
   agent's: a handle held from an earlier answer that no longer answers.
   So under either scope an `id:` term that matches nothing live says
   whether the id is among the removed, with its removal time, and
   prints the `all` route (rule 5 until `all` runs); `acq show` shows a
   removed item, marked; the scope block states membership and the count
   of each; `--describe` lists the values and the default. The store's
   half is C108 as ruled (the revision advances on a membership change,
   and nothing that prunes may take it back) and the retention park,
   whose trigger names search reading removed items. The builder
   proposed moving `all` from step 11 to step 10, whose theme is
   continuing across a changed basis, so the MCP's agent seat validates
   it (P2); owner, 2026-09-23, verbatim: "yes, let's go with --membership
   and move it to step 10". The reference carries it (*Membership*,
   under the terminal section).

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
  generate no `class` or `reqlevel` term; both are held by hand-counted
  tests alone, as `rarity` and `frame` are.
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
- The reference's line "computed values whose name or definition matches
  are listed beside them, marked computed" waits on step 7: none exists.
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
