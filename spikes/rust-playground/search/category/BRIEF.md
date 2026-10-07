# category — the brief for 9d-B's expected answers (2026-10-05)

Piece B of 9d builds category (`search/BUILD-PLAN.md`, "The steps
left", the row 9d-B). Before anything is built its expected answers
are a committed table in this track, which the owner reads: what each
category id answers of an item, taken from what the site answered and
from the rulings, never from the build. The builder then writes its
tests from the table and changes no value in it. This run writes that
table. It builds nothing, sits nothing and composes no search. The
reviewer — the session that launched you — checks the acceptance
sentence by hand and commits; your report is what it restores from.

This is also the first time the ruled rules are written as code. Where
writing a line made you choose between two readings, the choice is not
yours: the cell is `unruled` (below).

## Rules that bind you

- Read first: this track's `README.md`; in `search/DESIGN.md` the
  contract detail's two paragraphs *C106 — category (9d)* and *C106 —
  what reference data admits, and the test*, by `grep -n` and line
  range, never the file whole; `search/LEDGER.md`, "Holes ruled", the
  two rows whose step is 9d; `decisions/search.md`, C93, C105 and C106;
  the plan's rules 3, 4 and 8 and the 9d-B row; the module doc at the
  top of `crates/acquisition-search/src/class.rs`; `scripts/common.py`,
  `round-j-read.py` and `round-k-read.py`;
  `.claude/skills/site-sitting/SKILL.md`, "Traps".
- Write only inside `search/category/`: one script, three files under
  `data/`, one section of the README and a clause on its status line.
  Never the index (`search/README.md`), another track, `tools/`,
  `CONTEXT.md`, `decisions/`, `SURFACES.md`, the crates.
- Never commit, push, fetch or spawn. No network (C79).
- Do not run `acq` or `cargo`, and read nothing under `target/`: an
  expected answer is never read off the build. The copy is read-only
  through `common.copy_items`; the owner's store is never opened.
- Of a raw fetch (`../pseudo-stats/raw/searches/`, `../trade-query/raw/searches/`)
  read an item's `baseType`, `frameTypeId`, `descrText` and the
  clipboard's `Item Class:` line, and nothing else of it. Raw files,
  the clones and the copy are local and may be absent: say which was, and go on with what is there.
- **A cell the rulings do not answer is `unruled`**, with the readings
  it could take and what each would change, one line each (the plan,
  rule 4). Never fill it, and never leave it out. A rule that answers
  by its letter where the site was never asked is not unruled: it is
  answered, and marked as not asked.
- Every number written is printed by the script after the last edit.
  Where one restates a number the README already holds (F3, F5, the
  rounds), say whether the two agree.

## The work

1. **The reading.** `scripts/expected.py` holds one function: from an
   item's facts — realm, base as shown, frame, whether it carries the
   bestiary line — to its class, the ids that answer yes, the ids that
   are undecided and why, and the one id a count lists it under (J11).
   Every step of it cites the sentence of the paragraph or the ruling
   (J1–J4, J9–J11) it carries out. It reads the
   site's ids from `../trade-query/data/filters-2026-09-12.json`, the
   class as `common.py` reads it, the class's id and the name rules
   from `../repoe/scripts/base-taxonomy.py` (imported; where a ruling
   overrides one, the override is a named step here), the colours from
   `data/beast-colours.csv`, and the groupings from the file of step 2.
2. **The groupings' members**, `data/grouping-members.csv`: one row
   for each of the seven groupings and each leaf under `weapon` — in,
   out or `unruled` — and what says so: the captures by id, or Awakened
   PoE Trade's `meta.ts` marked not asked (J4), or nothing.
3. **The cases**, `data/expected-answers.csv`, one row an item:
   - `captured` — every distinct base, frame and bestiary line among
     the items fetched by a capture whose query names a category:
     `../pseudo-stats/data/captures.json`, every round, and the
     2026-09-13 fetches `common.site_fetches` reads. With each: the ids
     the site found it under, by capture; the ids a search of its type
     found nothing under beside a pair that found the type, both
     named; the class its clipboard states.
   - `ruled` — for each sentence of the paragraph and each of J1–J4
     and J9–J11, at least one item the captures do not hold, named for the
     sentence it answers: its base and frame as an item of the copy has
     them where the copy holds one, else as the export names the base.
     Also what C93 says of an item with no class, with no id for its
     class, and of a realm with no table.
   - `fixture` — the items of the OQ5 test in
     `crates/acquisition-search/tests/acceptance.rs`, as they stand
     there; read that one test and nothing else under `tests/`.

   Each row carries the facts above, the class, the ids that answer
   yes, the ids undecided with the cause (its wording is the builder's;
   the cause is yours), the id a count lists it under, the ids
   `unruled`, what the site said, whether
   the reading and the site agree on class and on every id asked, and
   the sentences the row rests on. An id a row does not list answers
   no. Every id of the filter answers yes on some row, or the README
   names it as having none and why.
4. **Disagreements and what the agreement is worth.** A row where the
   reading and the site differ is a finding: listed, not settled, the
   reading not bent to it. Then break the reading once for each rule
   in it (a flag of the script) and print how many captured rows then
   disagree. A rule whose break no capture notices rests on the ruling
   alone; list each.
5. **The copy**, `data/expected-copy.csv`: the function over every
   live `pc` item of the copy — for each id the items that answer yes,
   the items a count lists under it and the items undecided, and the
   items for which no id answers yes,
   split into none undecided and some undecided. Say whether items
   with a yes, with none, and undecided without a yes sum to the copy.
6. **The README**, written last from the script's output: one section,
   "Expected answers", about 2 KB — the cases by kind, counted; every
   `unruled` cell with its readings; every disagreement; the rules that
   rest on the ruling alone; each cell one search would settle, as the
   category and the type to ask. The detail stays in
   `data/`. Then the status line's clause. Nothing else in the README
   changes.

## Acceptance

`python3 search/category/scripts/expected.py` exits 0 and a second
run leaves the three files under `data/` byte for byte the same; it
prints, of the captures whose query names a category, how many it read
and how many it left out and why; every id of the filter is on a row
or named in the README; every `unruled` cell, every disagreement and
every break's count is in the README as the script printed it.

## The report

At most 300 words, to the reviewer: the cases by kind; the
disagreements; the `unruled` cells; the rules resting on the ruling
alone; what a sitting would close; which local input was absent; what
you left out and why; the one cut you would most want reversed; and
that the index row in `search/README.md` is not set.
