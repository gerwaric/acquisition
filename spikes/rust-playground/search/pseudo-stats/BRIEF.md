# pseudo-stats — the brief for open question 9: the defence twin on a private item (2026-10-01)

Round i's i05 pinned that the trade site leaves a defence line out of
the base defence percentile when its stat is the global twin, and that
reading such a line as local — the only reading its text allows —
reproduces 0 of 5 (`README.md`, "The percentile's round"; open question
9). The owner ruled J8 on 2026-10-01 (`search/BUILD-PLAN.md`, the
foot): this read first; if nothing a private item carries tells the two
apart, the text decides and the definition prints the limit. This run
is that read. The reviewer — the session that launched you — reads your
README changes and `data/` afterwards, checks the acceptance sentence
by hand, and commits; your report is what it restores from.

## Rules that bind you

- You have read the track before; read again only this README's "The
  percentile's round" and open question 9, `scripts/percentile.py` and
  `scripts/percentile-shapes.py` (how a line is read as local from its
  ids and from its text), `scripts/ranged-census.py` (the text join to
  the export's mods, which this read repeats for defence lines), and
  `search/repoe/README.md` F4 (the local and global twins).
- Work inside `search/pseudo-stats/`. Never edit anything under
  `tools/`, `MANIFEST.md`, `data/captures.json`, `data/search-sheet.csv`,
  `search/README.md` (the index), another track's files, `CONTEXT.md`,
  `decisions/`, `SURFACES.md`, or the crates. Do not run
  `tools/trade-captures.py` without `--check`, `tools/trade-sheet.py`
  or `tools/trade-pages.py`: another run is reading round j in
  `search/category/` in parallel.
- Never commit, push, fetch or spawn. No network (C79). The owner's
  store is never opened; the copy at
  `search/item-facts/raw/m3/store/mock/GERWARIC_7694.db` is read with
  sqlite in read-only mode. Do not run `acq` or `cargo`.
- Nothing is recalled: every number is printed by a script after the
  last edit. Add to the README about 1.5 KB at most, by section.

## The work

`scripts/defence-twins.py` → `data/defence-twins.csv`:

1. Over the captures (`data/captures.json`), every armour item
   carrying a line whose text a local and a global defence stat both
   display: what the site's id says (local or global), and every fact
   of the item a private body also carries — the array the line sits
   in, its flags, the item's rarity and frame, its base and class,
   whether it is a unique, the line's position, the other lines beside
   it. Which facts, alone or together, separate the global twins from
   the local lines on the captured items, and with how many items on
   each side.
2. Over the copy, every live armour item carrying such a text: the
   same facts, and what the export's mods say of the line by the text
   join (local, global, both, none), as `ranged-census.py` does for the
   ranged lines. How many of the copy's items each candidate
   separator would call global.
3. Say plainly whether anything a private item carries tells the two
   apart: a fact that separates every captured item, tested on the
   copy; or nothing, with the counts that show it. A separator found
   on five items is a candidate, not a rule: say how many items it
   rests on and what would break it.

The README: open question 9's row closed or kept, with its number and
its file; one short paragraph under "The percentile's round"; the
status line. Written last, from the script's output.

## Acceptance

`python3 search/pseudo-stats/scripts/defence-twins.py` exits 0 and
regenerates `data/defence-twins.csv` byte for byte on a second run;
every number in the README's new lines is printed by it; no other data
file changes.

## The report

At most 250 words, to the reviewer: whether a private item can tell
the twin from the local line, by which fact and on how many items; how
many of the copy's armour items the answer touches; what you left out
and why; the one cut you would most want reversed; and that the index
row in `search/README.md` is not set.
