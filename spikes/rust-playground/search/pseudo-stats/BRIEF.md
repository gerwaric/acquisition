# pseudo-stats — the brief for 9d-C's expected answers (2026-10-07)

Piece C of 9d builds the ranged family's 21 totals
(`search/BUILD-PLAN.md`, "The steps left", the row 9d-C). Before
anything is built its expected answers are a committed table in this
track, which the owner reads: what each of the 21 totals answers of an
item, taken from what the site showed and from the rulings, never from
the build. The builder then writes its tests from the table and changes
no value in it. This run writes that table. It builds nothing, sits
nothing and composes no search. The reviewer — the session that
launched you — checks the acceptance sentence by hand and commits; your
report is what it restores from.

Until now every reading of the family was made from the site's stat id,
which says which twin a line is. A private item shows the text alone.
This is the first time the family is read as the build must read it —
from an item's text and what the item is — and the first time its
rulings are written as code. Where writing a line made you choose
between two readings, the choice is not yours: the cell is `unruled`
(below).

## Rules that bind you

- Read first: this track's `README.md` — the headline, "The ranged
  family on the copy", "The captures, and what the site said" — and
  `data/sittings-detail.md`; in `search/DESIGN.md` the contract
  detail's paragraph *C94 — the ranged family (9d)*, by `grep -n` and
  line range, never the file whole; `search/LEDGER.md`, "Holes ruled",
  the rows whose step is 9c, 9c2 or 9c4 and the 9d row that holds J7;
  `decisions/search.md`, C92, C93 and C94; the plan's rules 3, 4 and 8
  and the 9d-C row; the module docs at the top of
  `crates/acquisition-search/src/totals.rs` and `pseudo.rs`, the docs
  alone; `tools/trade_rows.py` and `tools/trade-evidence.py`;
  `scripts/ranged-census.py`.
- Write only inside `search/pseudo-stats/`: one script, two files under
  `data/`, one section of the README. Never the index
  (`search/README.md`), another track, `tools/`, `CONTEXT.md`,
  `decisions/`, `SURFACES.md`, the crates. The README's status line and
  the index row are the reviewer's.
- Never commit, push, fetch or spawn. No network (C79). Restore no file
  with git: what a command changed stays changed, and is reported.
- Do not run `acq` or `cargo`, and read nothing under `target/`: an
  expected answer is never read off the build. The shipped tables under
  `crates/acquisition-search/reference/` are reviewed data and may be
  read. The copy is read-only, as `scripts/ranged-census.py` opens it;
  the owner's store is never opened.
- **The reading takes what a private item carries**: its base, its
  rarity, each line's text, array and flags. Never a line's `hash` or
  `mods`. The site's id is read beside the reading, to say which twin
  the site filed a line under, and never by it.
- **A cell the rulings do not answer is `unruled`**, with the readings
  it could take and what each would change, one line each (the plan,
  rule 4). Never fill it, and never leave it out. A rule that answers
  by its letter where the site was never asked is not unruled: it is
  answered, and marked as not asked.
- Every number written is printed by the script after the last edit.
  Where one restates a number the README already holds, say whether the
  two agree. Of a copy item write its base, rarity and lines; a
  unique's name; never an id, a rare's name or a place.
- Extracts: CSV with `lineterminator="\n"`, the first line naming the
  script and its inputs; `mod-templates.csv` opened with
  `newline="\n"`. Raw files, the clones and the copy are local and may
  be absent: say which was, and go on with what is there.

## The work

1. **The extracts, regenerated first.** Run, in this order, from the
   workspace root, each with `python3`: `tools/trade-sheet.py`,
   `tools/trade-captures.py`, `tools/trade-evidence.py`,
   `tools/trade-changes.py`, then in `search/pseudo-stats/scripts/`:
   `classify.py`, `other-totals.py`, `percentile.py`, `candidates.py`,
   `ranged-census.py`, `percentile-shapes.py`, `defence-twins.py`; then
   `tools/totals-table.py --check`. Then `git status --short`. Every
   one ran on 2026-10-07 at `5118a3a8` and left the tree clean. Report
   each file that changed, each command that failed, and each number a
   command printed that the README states otherwise. Go on from the
   files as regenerated.
2. **The reading.** `scripts/ranged-expected.py` holds one function:
   from an item's facts — realm, base, rarity, its lines by text, array
   and flags — to what each of the 21 totals answers of it: lacked, or
   its `low`, `high` and `avg`, or undecided with the cause (its
   wording is the builder's; the cause is yours). The rows are
   `data/table-changes.csv`'s, the 21 totals named `adds_…`, read by
   the script and never typed into it: 96 `add row` and 52 `twin`, 39
   `global` and 13 `local`, counted there 2026-10-07 — say whether
   yours agree. Which twin a line is comes from what the item is, as
   the paragraph and J7 rule it; what the item is, from the shipped
   class and category tables. `ranged-census.py` reads a weapon from
   another file: print, for every class of the class table, whether the
   two say the same. Every step of the function cites the sentence of
   the paragraph, the registry entry or the ruling it carries out.
3. **The cases**, `data/ranged-expected.csv`, one row a case and a
   total, three kinds:
   - `captured` — every item fetched by a capture whose returned query
     names a pseudo of the family (`data/captures.json`; 103 searches
     and 519 fetched rows, counted 2026-10-07 — say whether yours
     agree), the first row holding an item named beside a repeat. For
     each total its search named: what the site showed, a value or
     none, read as `tools/trade-evidence.py` reads it.
   - `ruled` — for each sentence of the paragraph, each rule of
     `totals.rs`'s C94 a ranged total meets (nothing summed, an unread
     contributor, a realm with no recipe), each `rule` row of the table
     of changes, and what C93 says of an item whose side cannot be
     established: at least one item the captures do not hold, named for
     the sentence it answers — an item of the copy where the copy holds
     one, else composed, and marked which.
   - `worked` — the plan's closing row, `Adds # to # Fire Damage` on a
     weapon, and the same line on an item that is no weapon; then as
     few composed items as leave no row of the 21 totals without a case
     that counts it. Each cell's arithmetic is written out in its row,
     by hand, before the function is run on it (the plan, rule 3): say
     of each whether the function gave what the hand did.

   A row carries the case and its kind, the item's facts, the total,
   the answer (status, `low`, `high`, `avg`, cause), the lines counted
   with the row that counted each, what the site showed or `not asked`,
   whether the two agree, the twin the site's id names beside the twin
   the reading took, and the sentences the row rests on. A total a case
   does not list is lacked. A line of the family whose text holds a
   line break is counted as the rulings say and marked. Print, for each
   row of the 21 totals, how many cases of each kind count it, and name
   each row no case counts.
4. **Disagreements, and what the agreement is worth.** A cell where the
   reading and the site differ is a finding: listed, not settled, the
   reading not bent to it. Then break the reading once for each rule in
   it (a flag of the script) and print how many captured cells then
   disagree. A rule whose break no capture notices rests on the ruling
   alone; list each.
5. **What no capture has asked.** The site files a line under a domain,
   which a private item shows as its array and flags. For every
   template a row of the 21 totals names, over the live `pc` items of
   the copy: the lines by template, side, array and flags, each beside
   the captured cells of the same four that a search asked and the site
   answered. A group the copy holds and no capture asked is a rule not
   asked: list each with its lines and items on the copy, the rule it
   rests on, and the one question a sitting would put — the pseudo, the
   side and the line — composing nothing.
6. **The copy**, `data/ranged-expected-copy.csv`: the function over
   every live `pc` item of the copy — for each total the items that
   have it, the sums of their `low`, `high` and `avg`, and the items
   undecided by cause. Say whether the items carrying a line of the
   family are `ranged-census.py`'s.
7. **The README**, written last from the script's output: one section,
   "The ranged family's expected answers", about 2 KB — the cases by
   kind, counted; every `unruled` cell with its readings; every
   disagreement; the rules that rest on the ruling alone; the rules not
   asked, each with its question. The detail stays in `data/`. Nothing
   else in the README changes.

## Acceptance

`python3 search/pseudo-stats/scripts/ranged-expected.py` exits 0 and a
second run leaves the two files under `data/` byte for byte the same;
it prints, of the captures whose query names a pseudo of the family,
how many it read and how many it left out and why; every row of the 21
totals is counted by a case or named as counted by none; every
`unruled` cell, every disagreement, every break's count and every rule
not asked is in the README as the script printed it.

## The report

At most 300 words, to the reviewer: what the regeneration changed, and
the numbers the README states otherwise; the cases by kind; the
disagreements; the `unruled` cells; the rules resting on the ruling
alone; the rules no capture has asked, with what a sitting would close;
which local input was absent; what you left out and why; the one cut
you would most want reversed; and that the status line and the index
row in `search/README.md` are not set.
