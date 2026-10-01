# category — the brief for the second run: a beast's colour, and round j (2026-10-01)

The first run (`README.md`, reviewed at `50f11270`) left seven open
questions, each a sitting search. The owner, 2026-10-01: J2, J3 and
J4 go to searches; for J1 he supplied the wiki's account of a beast's
colour, below. This run adds that reading, measures it on the copy,
and composes the searches as round j of the site sheet so that the
owner sits once. It does not sit. The reviewer — the session that
launched you — reads your README changes and `data/` afterwards,
checks the acceptance sentence by hand, and commits; your report is
what it restores from.

## What the owner supplied for J1

From `https://www.poewiki.net/wiki/Beast_(Bestiary)`, read by the
owner in a browser 2026-10-01 and quoted by him (the wiki is a
registered surface, corroboration only, `SURFACES.md`; nothing here
fetches it):

> Beasts can start appearing in Act 2. Areas which contain these
> beasts will have at least two to five Yellow Beasts and at least one
> Red Beast. Yellow Beasts have one Bestiary mod and have a yellow icon
> on the minimap. Red Beasts have two Bestiary mods and have a red icon
> on the minimap. Red beasts have significantly higher life and spawn
> monsters periodically. Beasts cannot have their life reduced below 1
> and are immune to culling strike-like effects.
>
> While Yellow Beasts can be any normally spawnable monster with the
> Beast monster category, Red Beasts are monsters that cannot be
> encountered normally. These include the four Spirit Beasts which can
> be used in beastcrafting to create six portals that lead to a boss
> fight with an avatar of the First Ones. The Black Mórrigan and
> "Harvest Beasts" (capturable beasts from Harvest encounters) have a
> chance to appear in tier 14+ maps.
>
> Beasts can also be found in a Beast node in the Azurite Mine. These
> beast can be killed and do not seem to get added to the Menagerie
> once defeated.

His words on it: "may help us determine the beast types". It is a
reading to measure and for the sitting to test, not a ruling.

## Rules that bind you

- Read first: `AGENTS.md`; `README.md` and `CONTEXT.md` as it says;
  `search/README.md` ("Rules in force", "Traps");
  `.claude/skills/site-sitting/SKILL.md` whole — "Before the sitting"
  is the procedure for round j and its traps bind; this track's
  `README.md` whole, `MANIFEST.md`, and the docstrings of
  `scripts/beasts.py` and `scripts/common.py`; `tools/trade-sheet.py`'s
  docstring and `round_i` (the shape of a round: versions pinned by
  number, each row saying what it decides and naming its control, a
  mutant beside a search that may find nothing); `tools/trade-pages.py`'s
  docstring; `search/BUILD-PLAN.md`, the foot, J1–J4.
- Work inside `search/category/`, with the one exception the skill
  requires: you may add one round function, `round_j`, to
  `tools/trade-sheet.py` and include it in the `site` sheet in
  `main()`; nothing already there is edited; the earlier rows of
  `search/pseudo-stats/data/search-sheet.csv` stay byte-identical
  (`cmp` against `git show HEAD:…`). Another run is reading round i's
  captures in `search/pseudo-stats/` in parallel: write nothing there
  but the regenerated sheet and round j's pages under `raw/sitting/`,
  and do not run `tools/trade-captures.py`, `tools/trade-evidence.py`
  or `tools/trade-pages.py i`. Never `search/README.md` (the index),
  `CONTEXT.md`, `decisions/`, `SURFACES.md`, the crates, any other file
  under `tools/`.
- Never commit, push, fetch or spawn. No network (C79): a link is
  composed and laid on a page; no tool opens one. The owner's store is
  never opened; the copy read-only as before. Do not run `acq` or
  `cargo`.
- Nothing is recalled: every number is printed by a script after the
  last edit. The README is about 12.8 KB: add about 2 KB at most, by
  section, detail in `data/`.

## The work

1. **The colour reading.** Add to `scripts/beasts.py` a fourth
   reading, `mods`, cited to the owner's quote above: a beast with one
   of the export's bestiary mods is yellow, with two red (what a beast
   with none or more than two is, say). Measure it on the copy's 158:
   its split, its agreement with the lure reading base by base, and
   whether the wiki's named red beasts — the four Spirit Beasts, the
   Black Mórrigan, the Harvest beasts — are on the copy and how each
   reading places them. `data/beasts.csv` and `data/beast-facts.csv`
   regenerate with the new columns; the README's F3 table gains the
   row.
2. **Round j.** The searches of open questions 1 to 6, each row saying
   what it decides, each with its control, and a mutant beside any
   search that may find nothing: for J1 the six searches of open
   question 1, the two base types chosen where the `mods` and `lure`
   readings agree on one red and one yellow, and a seventh that asks
   whether `monster.beast` holds both; for J4 one per member in doubt
   (open question 2); for J2 open question 3's two; for J3 open
   question 4; open questions 5 and 6. A category search finding
   nothing proves nothing until the same type under another id finds
   something: compose the pair. Then, each run bare and its exit read:
   `python3 tools/trade-sheet.py`, `--verify`, `--self-test`; `cmp` of
   the earlier rows; `python3 tools/trade-pages.py j` and `--check`.
3. **The README**: the status line; a short section "Round j" — one
   table, the searches and what each decides — and "Open questions"
   each pointing at its searches; written last, from the script
   outputs. `MANIFEST.md` gains one row: the wiki page, read by the
   owner, the excerpt in this brief at its commit.

## Acceptance

`python3 search/category/scripts/beasts.py` regenerates `data/` byte
for byte; `tools/trade-sheet.py`, `--verify` and `--self-test` each
exit 0; `cmp` of the sheet's earlier rows against `HEAD` finds no
difference; `tools/trade-pages.py j --check` exits 0; every number in
the README's new rows is printed by a script.

## The report

At most 300 words, to the reviewer: the `mods` reading's split and
whether it agrees with the lure reading; how many searches round j
holds, which are its core, the pace, and the first page's path; what
you left out and why; the one cut you would most want reversed; and
that the index row in `search/README.md` is not set.
