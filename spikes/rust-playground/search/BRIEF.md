# Item search — the brief for building 9d-C, the ranged family (2026-10-08)

Piece C of 9d builds the ranged family's 21 totals
(`search/BUILD-PLAN.md`, "The steps left", the row 9d-C), and with them
one rule of every total that the piece's sitting brought (J29). You are
its one builder. Its expected answers are final and committed
(`search/pseudo-stats/data/`, at `5f834e8c`), and the owner has read
them: you write the tests from that table, then the code, and you
change no value in it. You commit nothing; the session that launched
you commits. After you an agent that has not read your report looks at
the build, then the owner.

The plan's rules bind this piece as they bound every step. This file
adds only what is this piece's.

## What the piece closes on

- Every row of `search/pseudo-stats/data/ranged-expected.csv` is
  answered by the build as the row says, asked through the crate's
  boundary.
- `Adds # to # Fire Damage` on a weapon is counted toward the attacks'
  total alone (the plan's row).
- No total reads a crucible node's lines or a flask's own (J29), the
  totals shipped before this piece among them.
- The build's counts over the owner's copy are
  `search/pseudo-stats/data/ranged-expected-copy.csv`'s.
- The quality gate is green (`AGENTS.md`, "Quality gate").

## Rules that bind you

- Read first, in this order: `AGENTS.md`, `README.md`, `CONTEXT.md`;
  the plan whole; `decisions/search.md`; in `search/DESIGN.md` the
  paragraph *C94 — the ranged family (9d)* and the reference's lines on
  a computed value and a ranged one (`grep -n 'ranged'`), by line
  range; `search/LEDGER.md`, "Holes ruled", the rows whose step is 9c,
  9c2, 9c4 or 9d; `SEARCH-SLICE.md` whole — its shapes of fault are the
  checklist the look will hold you to; `TESTING-NOTES.md` before a
  test; the module doc of every module before you touch it,
  `totals.rs` and `pseudo.rs` first, then `category.rs`;
  `search/pseudo-stats/README.md`, "The ranged family's expected
  answers" and "Round n, read"; the docstring of
  `search/pseudo-stats/scripts/ranged-expected.py` and what it says
  each column of its two files means; `tools/totals-table.py`'s
  docstring.
- Write in `crates/`, in `tools/totals-table.py`, and the generated
  references (`CLI-REFERENCE.md`, `MCP-REFERENCE.md`, by
  `ACQ_UPDATE_FIXTURES=1`). Nowhere else: never `search/` — the tables,
  their scripts, this file, the plan, the design, the ledger — nor the
  other `tools/trade*` scripts, `SEARCH-SLICE.md`, `decisions/`,
  `CONTEXT.md`, `SURFACES.md`. Those are the session's at the close.
- Never commit, push, stash, reset, check out, fetch or spawn. No
  network (C79). No daemon: any `acq` you run is run with
  `ACQ_PROVIDER=mock` and `ACQ_STORE_DIR` set, and the owner's store is
  never opened (the plan, rule 7).
- **No expected value changes.** A row the rules as written do not
  give is a finding about the table or the paragraph: leave its test
  failing, report the row and what the build answers, and never add a
  case to the build that exists to pass it.
- **A rule nobody stated stops you** (the plan, rule 4). Where the
  paragraph and the rulings do not say what a total answers of an
  item, or the build would change what a user types beyond the list
  below, stop and report the question with its readings. A name, a key
  or a wording is yours to pick.
- An existing test's expected value changes only where a ruling since
  names the change: J29, for an item whose crucible or flask line a
  total counted; the 9c ruling that a bare ranged total is its `avg`,
  for the refusal it replaces. Each is listed in your report with its
  ruling.

## What is built

**The 21 totals.** Each is a ranged total as `totals.rs` already
defines one: its rows are `search/pseudo-stats/data/table-changes.csv`'s,
the totals named `adds_…` — 135 rows, 109 that mean the line whatever
the item is and 26 that mean the twin that is no weapon's own, counted
there by `tools/trade-changes.py`. They enter the shipped table through
`tools/totals-table.py`, at a new version, as every row has: generated
from the committed sources, checked by `--check`, the version on every
answer's basis. `--check` fails today, on the header's count of the
rows that wait; it passes when you are done.

**What the 26 rows read.** A row that means the other twin counts its
line only where the item is no weapon. What a weapon is, is category's
answer for `weapon` and nothing else's (rule 10; `category.rs` left
the door). Where that answer is not established the row's contribution
is unread, as any contributor a row names is, and the total is the
incomplete subtotal `totals.rs` already gives — said once, at the row
(rule 8): a total none of whose rows reads the item's category is
never left open by it (J30). How a row says that it reads the category
is yours; C94's registry line lists a row's coordinates without it,
which the session knows and brings to the owner at the close — it is
no stop.

**J29, of every total.** No total reads a line in the arrays the site
files under no stat: a crucible node's lines and a flask's own, which
the body gives as `crucibleMods` and `utilityMods`, and no other
array. It is said once — in one place the table or the loader states
it, never on each of the table's rows — and every total's definition
prints it. An array a total does not read cannot leave that total
incomplete. A line's term, `sum( … )` of a line and the vocabulary are
untouched: the rule is a total's. The four readings follow their
totals and need nothing of their own.

**A ranged total with no slot word is its `avg`** (the owner,
2026-09-26: "yes, bare means average"), wherever a computed value is
asked: a comparison, `has:`, `--sort`, a count's sum. `low` and `high`
stay. Today the build refuses the bare name; that refusal goes. A
ranged total is nothing, and so lacked, only where its low and its
high both are (J31), as `pseudo.rs` reads it now.

**The limit, printed.** Every ranged total's definition prints that a
line under the other stat on a weapon is read as the weapon's own,
with what the site showed: no listed weapon was found to carry one
(round i, i06–i08).

## The surface the piece adds

The session holds the diff of `--describe --json` against this list.
Anything a user can type or a consumer can read that is not here is a
stop.

- 21 computed values, named as the table of changes names them or in
  that style: each a range, taking `low`, `high` and `avg` last, the
  bare name its `avg`.
- `--describe` of each: its rows, which of them read what the item is,
  the site's id and text, the limit, the sources and version.
- In every total's definition, and wherever the table is described:
  J29's rule and the two arrays.
- `bind::NOT_BUILT` loses `pseudo.<name>.<slot>`; the refusal of a
  bare ranged total is gone, and the near readings it offered with it.
- On the basis, the totals table's new version.
- No new reason: an item whose category is not established is
  undecided under a row that reads it by category's own reason, made
  where it is made today (`eval::why`).
- Answers that change with no change to what is typed: every total
  shipped before this piece, on an item whose crucible or flask line
  it counted.

Not this piece: the percentile and its entry in `bind::NOT_BUILT`; the
MCP; anything of step 10. The reference's lines that say a ranged
computed value needs a slot word, and none is shipped, are the
session's to change at the close: build to the ruling.

## The work, in order

1. **The tests, before any code, each seen to fail for the reason you
   expect.**
   - *The table.* One test reads `ranged-expected.csv` where it lies,
     unchanged, and puts each of its cases into a fixture store through
     the store's own ingest: the realm, the base, the rarity, and its
     lines in the arrays and under the flags the row gives — a line
     entered as the row gives it, a mod of several rows as one entry,
     never taken apart (the record's shape of fault "A claim the code did
     not make": a fixture that wrote its capture's input another way).
     Each of the 21 totals is then asked of each case: lacked; or its
     `low`, `high` and `avg`, each by its slot word and the bare name
     beside `avg`; or undecided. A total a case does not list is
     lacked. A cause pins what could not be established and never a
     wording. The `site` and `agrees` columns are evidence and assert
     nothing.
   - *The plan's closing row*, worked by hand before running: the
     table's two `worked` cases of the fire line, on a weapon and off
     one, asked by comparison, by `has:`, sorted, and summed in a
     count.
   - *J29 on the totals already shipped*, worked by hand before
     running, since no flask of the copy holds a line of the family: a
     flask whose own line is a row's template, lacking the total; the
     same line as an explicit on an item that is no flask, having it; a
     weapon with a row's line in its crucible array and the same
     template as an explicit, the total the explicit's alone; a body
     whose crucible array is unreadable, the total not left open by
     it. Then round n's captures as captured
     (`search/pseudo-stats/data/captures.json`, n02, n05, n07, n09):
     ten items each, the pseudo its search named lacked on every one.
   - *A row that reads the category, under `not` and `or`*, worked by
     hand from the table's side-open cases and C93: an item whose
     category is not established, asked by the not of a plain total's
     comparison and by an or of it with a term that holds.
   - One CLI `--json` smoke for the closing row and one for a flask,
     and `--describe` of one ranged total.
2. **The table and its generator**, `--check` passing.
3. **The reading.** Before you write a maker, look for the one that
   exists (rule 10): a row's sum is the evaluator's, a reason is
   `eval::why`'s, what a weapon is is category's.
4. **The module docs.** `totals.rs` and `pseudo.rs` carry the
   paragraph's rules and J29 to J31 under their decision id, the
   owner's quoted words verbatim, so that the session can take the
   paragraph out of `search/DESIGN.md` at the close.
5. **Three breaks.** Break the build three times, one rule each — one
   of them J29, one a row that reads the category — and name the test
   that noticed. A break nothing notices is a finding, reported, not
   fixed in silence.
6. **The copy.** Over the copy as M3 asks it (`search/MEASUREMENTS.md`,
   the M3 block; the release build): for each of the 21 totals the
   items that have it and the items undecided, against
   `ranged-expected-copy.csv`'s `total` rows, and the sums where one
   ask gives them; then, for J29, how many items have
   `increased_movement_speed` and `total_chaos_res` now and how many
   had them at `5f834e8c`'s build. A measurement, reported with its
   command, never a test. The copy is local and may be absent: say so
   and go on.
7. **The gate**, all seven commands, in `AGENTS.md`'s order, after the
   last edit; build again after `cargo fmt` changes a file.

## The report

To the session, about 400 words and the lists: what was built, by
file; every name a user can now type and every JSON key added; each
existing test whose expected value changed, with its ruling; each
choice that was yours, how a row reads the category and where J29 is
said first among them; the three breaks and what noticed; the copy's
numbers beside the table's, every difference by total; each gate
command and its result as it ran; what you left out; what you are
least sure of. If you stopped, the question comes first, with its
readings and the state of the tree.
