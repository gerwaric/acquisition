# category — the brief for round k: every beast's colour from the site, and J8's one search (2026-10-04)

No published list gives a beast's colour (the owner, 2026-10-04: "I
can't find a list of beasts by color anywhere on the internet, so
let's use a longer search sitting"). So the colour table is admitted
under C106's clause (a) by the site's own answers, each row citing its
capture, and the sitting is the source. This run composes it, and
J8's one search beside it, as round k. It does not sit. The reviewer
— the session that launched you — checks the acceptance sentence by
hand and commits; your report is what it restores from.

## Rules that bind you

- You have read the track before; read again `README.md`'s "Round j"
  and F3, `scripts/round-j.py` and `scripts/round-j-read.py`,
  `tools/trade-sheet.py`'s `round_i` (the `DEFENCE_TWINS` texts and
  how i05 asked them) and `round_j`, and `.claude/skills/site-sitting/SKILL.md`,
  "Before the sitting" and "Traps" (the pace: 15 searches in 60 s and
  60 in 300 s an address; one link every ten seconds is half of it).
- Work inside `search/category/`, with the one exception the skill
  requires: you may add one round function, `round_k`, to
  `tools/trade-sheet.py` and include it in the `site` sheet in
  `main()`; nothing already there is edited; the earlier rows of
  `search/pseudo-stats/data/search-sheet.csv` stay byte-identical.
  Write nothing else in `search/pseudo-stats/` but that regenerated
  sheet and round k's pages under `raw/sitting/`. Never
  `search/README.md` (the index), `CONTEXT.md`, `decisions/`,
  `SURFACES.md`, the crates, any other file under `tools/`.
- Never commit, push, fetch or spawn. No network (C79): a link is
  composed and laid on a page; no tool opens one. The owner's store is
  never opened; the copy read-only as before. Do not run `acq` or
  `cargo`, `tools/trade-captures.py` or `tools/trade-evidence.py`.
- Every number written is printed by a script after the last edit.
  The README gains one short section, about 1 KB; the detail is
  `data/round-k.csv`.

## The work

1. **J8's pair, first.** k001: armour carrying one of the five
   `DEFENCE_TWINS` texts as an *implicit* under the `(Local)` stat —
   the implicit-category ids that display each text with `(Local)`,
   as `search/trade-query/data/stats-2026-09-12.json` lists them, in a
   `count` of at least one — the percentile required
   (`PERCENTILE_CONTROL`). Found: a local defence line sits as an
   implicit, and the array alone does not separate the twins. k002:
   its mutant, the same with the global ids, which must find the ten
   i05 found. Say in each row what it decides, as `round_i` does.
2. **Every beast's colour.** The universe is the site's `monster`
   list, `search/trade-query/data/items-2026-09-12.json`: every name
   there (your script counts them). One search per name under
   `monster.yellowbeast` with `type` set to the name, in this order:
   the copy's beast bases first (`data/beasts.csv`), then the rest by
   name. A name found is yellow by the site's own word. A name not
   found is asked again under `monster.redbeast` in the round the
   reviewer composes after this sitting, so each of these rows names
   that second search as its pair, by rule and not by number. The row
   says what it decides.
3. **The read-back.** `scripts/round-k.py` → `data/round-k.csv`: one
   row per search of the round, its name, its category, the copy's
   items of that base, and what the mod-count regularity of
   `data/round-j-beasts.csv` would say of it (yellow at exactly one of
   the export's bestiary mods on the copy's items, red otherwise, or
   no item on the copy) — recorded so that the sitting's answer can be
   held against it afterwards, never read as the answer.
4. The checks, each bare with its exit read: `python3
   tools/trade-sheet.py`, `--verify`, `--self-test`; `cmp` of the
   earlier rows against `git show HEAD:`; `python3 tools/trade-pages.py k`
   and `--check`; `scripts/round-k.py` regenerating its file byte for
   byte on a second run.
5. The README: a section "Round k" — what it asks, the count, the
   pages, the pace and time, the first page's path; the status line.

## Acceptance

`tools/trade-sheet.py`, `--verify` and `--self-test` each exit 0; the
sheet's earlier rows are byte-identical to `HEAD`; `tools/trade-pages.py k
--check` exits 0; `scripts/round-k.py` regenerates `data/round-k.csv`
byte for byte; every name of the site's `monster` list has exactly one
search of the round, and the two J8 searches each name their control.

## The report

At most 250 words, to the reviewer: how many searches round k holds,
how many pages, the time at one link every ten seconds, and the first
page's path; how the J8 pair is composed; what you left out and why;
the one cut you would most want reversed; and that the index row in
`search/README.md` is not set.
