# pseudo-stats — the brief for round n: composed for the owner's sitting (2026-10-08)

The two reads before 9d-C's sitting (`README.md`, "Before the sitting:
the domain, and the twin from the export"; their brief is this file at
`3ea81a86`) left four things only the site can say. This run composes
the searches that ask them, as round n of the sheet, and lays out the
pages the owner sits. It sends nothing and opens no link (C79;
`SURFACES.md`, the trade site's rows: a person opens every link). The
reviewer — the session that launched you — runs the checks again and
commits; your report is what it restores from.

## Rules that bind you

- Read first, in order: `.claude/skills/site-sitting/SKILL.md`, whole —
  its "Before the sitting" is the procedure and its "Traps" bind you;
  this track's `README.md`, the section named above and "The captures,
  and what the site said"; `tools/trade_rows.py`; in
  `tools/trade-sheet.py` the docstring, `query`, `group`,
  `batch_query`, `category`, `complete`, `sound`, `round_i`, `round_m`,
  `report`, `verify` and `main`, by `grep -n` and line range;
  `tools/trade_query.py`; `tools/trade-pages.py`'s docstring.
- Write only: `tools/trade-sheet.py` — one function, `round_n`, its
  constants, and its place in `main` and wherever a round is named —
  and `tools/trade-pages.py` only if a round must be named there;
  `search/pseudo-stats/data/search-sheet.csv`, by the tool;
  `search/pseudo-stats/raw/sitting/`, by the tool; one section of this
  track's README. Append no version to `tools/trade_rows.py`: this
  round asks the rows as they stand. Never the index, another track,
  `CONTEXT.md`, `decisions/`, `SURFACES.md`, the crates, the README's
  status line.
- Never commit, push, fetch or spawn. **No network: no tool or command
  of yours opens a link or sends a request.** Restore no file with git.
  Do not run `acq` or `cargo`.
- Every earlier row of the sheet stays byte for byte what it is. Every
  pseudo's version is pinned by number, written out. No query of the
  round is one the sheet already holds (the skill's trap): check, and
  print that none is.
- A search that can find nothing has its mutant in the round, the same
  search with one thing changed, which must find something. Each row's
  `decides` says what finding items means and what finding none means;
  its `control` names the mutant or the control.
- A stat id is read from `search/trade-query/data/stats-2026-09-12.json`
  by its text, by the script, never typed. A crucible entry there is a
  whole node with its numbers and tier in the text, several rows to a
  node: 64 entries hold a row `Adds # to # Fire Damage`, counted
  2026-10-08 by replacing each number with `#` — say whether yours
  agrees. The site refuses a query past some cost: keep a `count` or a
  `not` to 71 ids, a weighted group to 20.
- A number is printed by a script after its last edit, never recalled.

## The questions

Each is asked of every listed item at once, as the skill's sound check
does, and once more by a search whose ten fetched items show the
pseudo's value beside the line. The forms are yours; what each must
decide is not.

1. **Does a pseudo count a crucible line?** No capture holds one. For
   one total that is no ranged one and whose row a crucible node
   displays (the copy holds `#% increased Spell Damage`, 8 lines, and
   `#% increased Attack Speed`, 5): is there an item carrying such a
   node while showing no pseudo?
2. **Is a weapon's crucible `Adds # to # <Type> Damage` its own?** For
   one type at least, on weapons: carrying such a node, is there one
   showing no attacks' pseudo; and is there one showing no plain
   pseudo? The two answers together say which of three holds — counted
   as the weapon's own, counted as the other twin, or not counted — and
   the row says which pair of answers means which.
3. **Does a pseudo count a flask's own line?** The copy holds 1,084
   lines in flasks' `utilityMods` under templates the shipped totals
   name — `#% increased Movement Speed` 276, `#% to Chaos Resistance`
   145, `#% to Cold Resistance` 122 — and no capture holds a flask
   under a pseudo. For two of those totals: does any flask show the
   pseudo? And ten flasks of a base that carries the line, the pseudo
   beside them, so that what the site files the line under is read.
4. **The implicit twin off a weapon.** The copy holds 8 such lines, on
   a helmet base and a unique ring, and the export cannot place them.
   Off a weapon — the site's category cannot be negated, so by the
   categories that hold the copy's — carrying the unsuffixed line as an
   implicit under the stat without `(Local)`: is there one showing no
   plain pseudo? And does any item that is no weapon carry it under the
   `(Local)` stat at all?
5. **The thin domains.** Fractured, enchant, scourge and crafted lines
   are shown counted on 1, 2, 4 and 9 readings. One sound check each,
   on a total with ids in that domain.

## The work

1. `round_n` in `tools/trade-sheet.py`, rows `n01` onward, question 1
   first; the sheet written; then, each run bare, `--verify`,
   `--self-test`, and `cmp` of the sheet's earlier rows against
   `git show HEAD:` of the same file.
2. `tools/trade-pages.py n`, and its `--check`.
3. The README, one section, "Round n, composed", about 1 KB: a table
   of the searches — the question, what items found means, what none
   means, its mutant — and the core, the searches that decide a row of
   the build. Nothing else in the README changes.

## Acceptance

`tools/trade-sheet.py --verify` and `--self-test` exit 0; the sheet's
rows before `n01` are byte for byte `HEAD`'s; `tools/trade-pages.py n
--check` exits 0; every question above has its searches in the README's
table, and every search that can find nothing has a mutant named there.

## The report

At most 300 words, to the reviewer: the searches, counted, by question;
the core; each mutant; the ids each search carries, counted, and the
largest group against the cost the site refuses; any question you could
not put as one search and why; what you changed in the tools beyond the
round's function; what you left out; the one cut you would most want
reversed; the first page's path; and that the status line and the index
row are not set.
