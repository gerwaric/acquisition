# pseudo-stats — the brief for 9d-A's expected answers (2026-10-08)

Piece A of 9d builds `pseudo.defence_pct`, the site's Base Percentile
(`search/BUILD-PLAN.md`, "The steps left", the row 9d-A). Before
anything is built its expected answers are a committed table in this
track, which the owner reads: what the field answers of an item, taken
from what the site showed and from the rulings, never from the build.
The builder then writes its tests from the table and changes no value
in it. This run writes that table. It builds nothing, sits nothing and
composes no search. The reviewer — the session that launched you —
checks the acceptance sentence by hand and commits; your report is what
it restores from.

The track has scored the percentile twice: from the site's stat ids
(`scripts/percentile.py`) and from text alone
(`scripts/percentile-shapes.py`). Neither holds what was ruled after it
— J5, the exact half, and J8, the defence twin told by the item's mark
— and neither answers an item the rule cannot read. This is the first
time the rulings are written as one function over what a private item
carries. Where writing a line made you choose between two readings, the
choice is not yours: the cell is `unruled` (below).

## Rules that bind you

- Read first: this track's `README.md` — the headline, "The
  percentile", "The percentile's round", and "The ranged family's
  expected answers" for the form of what you write; in
  `search/DESIGN.md` the contract detail's paragraph *C101 — the
  derived fields not yet built* (it starts at line 560: by `grep -n`
  and line range, never the file whole); `search/LEDGER.md`, "Holes
  ruled", the seat's row (V5), the three 9d rows that hold J5 and J6,
  J8, and J32, and step 7's row that holds T2; `decisions/search.md`,
  C93, C101, C102 and C106; the plan's rules 3, 4 and 8 and the 9d-A
  row; the module doc at the top of
  `crates/acquisition-search/src/pseudo.rs`, the doc alone;
  `scripts/percentile.py`, `scripts/percentile-shapes.py`,
  `scripts/defence-twins.py`; `scripts/ranged-expected.py`, its
  docstring and its shape, which piece C's builder worked from.
- Write only inside `search/pseudo-stats/`: one new script, two new
  files under `data/`, one new section of the README, and what step 1
  names. Never the index (`search/README.md`), another track, `tools/`,
  `CONTEXT.md`, `decisions/`, `search/LEDGER.md`, `search/DESIGN.md`,
  `SEARCH-SLICE.md`, `SURFACES.md`, the crates. The README's status
  line, its headline block and the index row are the reviewer's.
- Never commit, push, fetch or spawn. No network (C79). Restore no file
  with git: what a command changed stays changed, and is reported.
- Do not run `acq` or `cargo`, and read nothing under `target/`: an
  expected answer is never read off the build. The shipped tables under
  `crates/acquisition-search/reference/` are reviewed data and may be
  read. The copy is read-only, as `scripts/percentile-shapes.py` opens
  it; the owner's store is never opened.
- **The reading takes what a private item carries**: its realm, its
  base, its rarity, its properties as displayed, each line's text,
  array and flags, and the item's own marks (a top-level key whose
  value is true: `searing`, `tangled`, `synthesised`). Never a line's
  `hash` or `mods`, never `extended`. The site's value and the site's
  id are read beside the reading, to say what the site showed and
  which twin it filed a line under, and never by it.
  `data/captures.json` keeps no item marks: `scripts/defence-twins.py`
  reads them from `raw/searches/`, and `data/defence-twins.csv` holds
  them for every captured item carrying a twin line.
- **A cell the rulings do not answer is `unruled`**, with the readings
  it could take and what each would change, one line each (the plan,
  rule 4). Never fill it, and never leave it out. A rule that answers
  by its letter where the site was never asked is not unruled: it is
  answered, and marked as not asked.
- Every number written is printed by a script after its last edit.
  Where one restates a number the README already holds, say whether the
  two agree. Of a copy item write its base, rarity, properties and
  lines; a unique's name; never an id, a rare's name or a place.
- Extracts: CSV with `lineterminator="\n"`, the first line naming the
  script and its inputs. Raw files, the clones and the copy are local
  and may be absent: say which was, and go on with what is there.

## The work

1. **The extracts, regenerated first.** Run, in this order, from the
   workspace root, each with `python3`: `tools/trade-sheet.py`,
   `tools/trade-captures.py`, `tools/trade-evidence.py`,
   `tools/trade-changes.py`, then in `search/pseudo-stats/scripts/`:
   `classify.py`, `other-totals.py`, `percentile.py`, `candidates.py`,
   `ranged-census.py`, `percentile-shapes.py`, `defence-twins.py`,
   `evidence-domains.py`, `ranged-twins.py`, `ranged-expected.py`; then
   `tools/totals-table.py --check`. Then `git status --short`.

   The reviewer ran this list at `6f5da78e` on 2026-10-08, and it is
   not clean. `ranged-census.py` exits 1 (line 188: a captured line
   with no `hash`); four files change —
   `data/percentile-captures.csv`, `data/percentile-shapes.csv`,
   `data/defence-twins.csv`, `data/evidence-domains.csv` — round n's
   captures having entered `data/captures.json` after those scripts
   last ran. `percentile.py` then printed 637 distinct items, the
   stated rule reproducing 634, either integer at an exact half 637,
   each one value; the README says 609, 607 and 609. Say whether your
   run is the same, file by file and number by number.

   Then, and no more than this: mend `ranged-census.py` so that a line
   with no hash names neither twin, run it, and report whether its two
   files changed; bring the numbers of "The percentile's round" — its
   first paragraph and its paragraph "Left out, answered by the
   captures" — to what the scripts now print, changing no sentence's
   claim, and listing each number that moved. Leave "Before the
   sitting" as it is and report every number of it the regenerated
   `evidence-domains.csv` no longer gives: what becomes of that section
   is the reviewer's. Go on from the files as regenerated.
2. **The reading.** `scripts/percentile-expected.py` holds one
   function: from an item's facts, as the rule above lists them, to
   what `pseudo.defence_pct` answers of it — lacked, or a value, or
   undecided with the cause (its wording is the builder's; the cause is
   yours) — and beside the answer what it read: each defence type, the
   lines taken as flat and as increase, the lines left out, the quality
   term, the rolls each display admits and each roll's percentile. The
   ranges are `search/repoe/data/base-defences.csv`'s, read and never
   typed; which stat a line's text is, the join `percentile-shapes.py`
   makes. Every step of the function cites the sentence of the
   paragraph, the registry entry or the ruling it carries out; a step
   with nothing to cite is where a cell is `unruled`.
3. **The cases**, `data/percentile-expected.csv`, one row a case, three
   kinds:
   - `captured` — every item fetched by a capture, in
     `data/captures.json` (864 searches, 4819 fetched rows, 866
     carrying the site's value, counted 2026-10-08 — say whether yours
     agree) and the 30 of `search/trade-query/data/fetch-census.json`,
     the first row holding an item named beside a repeat. What the site
     showed is its value or none: an item the site shows no percentile
     of is a case too.
   - `ruled` — for each sentence of the paragraph, each of J5, J6's
     four pinned shapes and J8's four parts, each status C93 and the
     module doc give a derived field (lacked, a realm with no table, a
     base the table lacks, a property the field reads unread), and each
     thing the function had to decide that none of those names: at
     least one item the captures do not hold, named for what it
     answers — an item of the copy where the copy holds one, else
     composed, and marked which. Among them, whatever else the function
     turns up: a roll over the base's maximum and one under its
     minimum; a display more than one roll gives, where the rolls give
     different values; a base name the range table holds more than
     once; a base whose minimum is its maximum; a defence displayed
     that the base lacks; a `reduced` local line.
   - `worked` — as few items as leave no step of the function without
     a case that takes it: one defence type, two, a local flat line and
     a local increase with quality, an exact half. Each one's
     arithmetic is written out in its row, by hand, before the function
     is run on it (the plan, rule 3): say of each whether the function
     gave what the hand did.

   A row carries the case and its kind, the item's facts, the answer
   (status, value, cause), what was read, what the site showed or `not
   asked`, whether the two agree, for a twin line the side the site's
   id names beside the side the reading took, and the sentences the row
   rests on. Print, for each step of the function, how many cases of
   each kind take it, and name each step no case takes.
4. **Disagreements, and what the agreement is worth.** A cell where the
   reading and the site differ is a finding: listed, not settled, the
   reading not bent to it. Then break the reading once for each rule in
   it (a flag of the script) and print how many captured cells then
   disagree. A rule whose break no capture notices rests on the ruling
   alone; list each.
5. **What no capture has asked.** Over the live `pc` items of the copy
   the function reads to a value: each line it took or left out, by
   template, role, array, flags and the item's mark; and each item by
   shape (`percentile-shapes.py`'s list, added to where the function
   needs another) — each beside the captured cells of the same group
   that the site answered. A group the copy holds and no capture asked
   is a rule not asked: list each with its lines and items on the copy,
   the rule it rests on, and the one question a sitting would put,
   composing nothing.
6. **The copy**, `data/percentile-expected-copy.csv`: the function over
   every live item of the copy — by class, the items that have a
   value, those that lack one, and those undecided by cause. Say
   whether the armour items are `percentile-shapes.py`'s, and what
   became of each it left unread.
7. **The README**, written last from the script's output: one new
   section, "The percentile's expected answers", after "The
   percentile's round", about 2 KB — the cases by kind, counted; every
   `unruled` cell with its readings; every disagreement; the rules that
   rest on the ruling alone; the rules not asked, each with its
   question. The detail stays in `data/`. Beyond step 1's numbers,
   nothing else in the README changes.

## Acceptance

`python3 search/pseudo-stats/scripts/percentile-expected.py` exits 0
and a second run leaves the two files under `data/` byte for byte the
same; it prints how many captured items it read and how many it left
out and why; every step of the function is taken by a case or named as
taken by none; every `unruled` cell, every disagreement, every break's
count and every rule not asked is in the README as the script printed
it; every command of step 1 exits 0, and `git status --short` shows
nothing outside `search/pseudo-stats/`.

## The report

At most 300 words, to the reviewer: what the regeneration changed, and
the README numbers that moved; the cases by kind; the disagreements;
the `unruled` cells; the rules resting on the ruling alone; the rules
no capture has asked, with what a sitting would close; which local
input was absent; what you changed that the brief did not name; what
you left out and why; the one cut you would most want reversed; and
that the status line, the headline and the index row in
`search/README.md` are not set.
