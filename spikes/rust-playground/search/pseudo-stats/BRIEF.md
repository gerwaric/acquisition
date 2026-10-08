# pseudo-stats — the brief for round n's rulings in the rows, and 9d-C's expected answers made final (2026-10-08)

Round n is read (`README.md`, "Round n, read"): the site counts no
crucible node's line and no flask's own line. The rows count a line by
its text wherever it sits, so 80 readings disagree. The owner has
ruled, and this run carries three rulings into the rows under test, the
table of changes and the expected answers of piece C, changing nothing
the build ships. The reviewer — the session that launched you — checks
the acceptance sentence by hand and commits; your report is what it
restores from.

## The rulings

1. **A total leaves out a crucible node's lines and a flask's own
   lines, as the site does.** The owner, 2026-10-08, of both: "Leave
   both out." On a private item these are the arrays `crucibleMods` and
   `utilityMods`. It is a rule of every total, by the array, and of no
   other array: a line with no stat elsewhere is not this rule's.
2. **Where both twins of the unsuffixed line count toward a total at
   one weight, the total's row is the line whatever the item is.** The
   seven attacks' totals hold a `local` and a `global` row of each
   unsuffixed line at weight 1; each pair becomes one row that means
   `any`. No answer of the site's changes. An item whose side cannot be
   established then answers its attacks' totals, since their rows do
   not ask the side, and is undecided under the plain and the spells'.
3. **A ranged total is nothing only where its low and its high are
   both nothing**, as `pseudo.rs` reads it today: a range that cancels
   in its average alone is a value.

Rulings 2 and 3 are the session's recommendations, put to the owner
2026-10-08 and not objected to; the session records them. Write no
quotation of the owner but ruling 1's.

## Rules that bind you

- Read first: `.claude/skills/site-sitting/SKILL.md`, "After the
  sitting", steps 8 to 10, and "Traps"; this track's `README.md`,
  "The ranged family's expected answers" and "Round n, read";
  `tools/trade_rows.py`, `tools/trade-evidence.py`,
  `tools/trade-changes.py`; `scripts/ranged-expected.py`, its docstring,
  its reading and its `unruled` cells.
- Write only: `tools/trade_rows.py` — versions appended, none edited,
  every search already captured still answering the version it names;
  `tools/trade-evidence.py`; `tools/trade-changes.py`;
  `scripts/ranged-expected.py`; the extracts those four write under
  `search/pseudo-stats/data/`; this track's README, the two sections
  named below. Never `tools/totals-table.py`, the crates or anything
  under `crates/acquisition-search/reference/`: the shipped table is
  the builder's to regenerate. Never the index, another track,
  `tools/trade-sheet.py`, `data/search-sheet.csv`, `CONTEXT.md`,
  `decisions/`, `search/LEDGER.md`, `search/DESIGN.md`, `SURFACES.md`,
  the README's status line.
- Never commit, push, fetch or spawn. No network. Restore no file with
  git. Do not run `acq` or `cargo`.
- A number is printed by a script after its last edit, never recalled.
  Where a number restates one the README holds, say whether they agree.

## The work

1. **Ruling 1 in the evidence.** `tools/trade-evidence.py` reads a line
   in either array as counted by no pseudo, said once where a line is
   read. Run it: say how many readings it makes and how many disagree.
   Then the mutants, a flag each, each seen to disagree: `crucibleMods`
   counted; `utilityMods` counted. Print each count.
2. **Ruling 1 in the table of changes.** One `rule` row of every total
   in `tools/trade-changes.py`, its evidence the captures by name
   (n01–n09) and the owner's words.
3. **Ruling 2 in the rows.** A version appended to each of the seven
   attacks' pseudos in `tools/trade_rows.py`, its `why` saying that the
   pair is the line whatever the item is and that no capture asks for
   it: the site's answers are the same under both. Show that: the
   evidence's readings and disagreements before the versions and after.
   Then `tools/trade-changes.py`: print the 21 totals' rows by `change`
   and `twin_or_reads`, beside what they were — 96 `add row`, 52 `twin`
   (39 `global`, 13 `local`).
4. **`tools/totals-table.py --check`**, run and not mended: say what it
   prints. If it now differs from the shipped file, say in what, to the
   line; the builder regenerates the table at a new version.
5. **The expected answers.** `scripts/ranged-expected.py` under the
   three rulings: the two arrays out of every total; the attacks' rows
   as the table of changes now holds them; the two questions' cells
   answered, each row citing its ruling. Add to the `ruled` cases a
   weapon whose only line of the family is a crucible node's, and a
   flask, the copy's where it holds one. Rerun everything the script
   prints — the cases by kind, the captured cells and how many
   disagree, the breaks, the copy, what no capture asked — and add a
   break for each array counted. Round n's captures name pseudos of
   the family: they are captured cases now. No cell is `unruled`, or
   the report says which and why.
6. **The README.** "The ranged family's expected answers" written
   again from the script's output, about 2 KB, the unruled paragraph
   replaced by the rulings each in a line. In "Round n, read", the last
   paragraph replaced by what step 1 printed. Nothing else changes.

## Acceptance

`tools/trade-evidence.py` and `tools/trade-changes.py` exit 0 and each
mutant of step 1 disagrees; `scripts/ranged-expected.py` exits 0, a
second run leaves its two files byte for byte the same, and no cell is
`unruled`; every number in the two README sections is one a script
printed in this run; `tools/trade-sheet.py --verify` still exits 0 and
`data/search-sheet.csv` is byte for byte `HEAD`'s.

## The report

At most 300 words, to the reviewer: the readings and disagreements
before and after each ruling; each mutant's count; the 21 totals' rows
by kind, before and after; what `totals-table.py --check` printed; the
expected answers' cases by kind, the captured cells and disagreements,
the breaks that no capture notices, what is still not asked on the
copy; any cell still `unruled`; what you changed that the brief did not
name; what you left out; the one cut you would most want reversed; and
that the status line and the index row are not set.
