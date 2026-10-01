# pseudo-stats — the brief for reading round i (2026-10-01)

The owner sat round i on 2026-10-01: eight of its nine searches, i07
left unopened (`MANIFEST.md`, the round's row). The recording is split
and its 13 captures are under `raw/searches/` (`i01`–`i06`, `i08`,
`i09`). This run reads them under the site-sitting skill, "After the
sitting", steps 7 to 9, and says what each search decided. The
reviewer — the session that launched you — reads your README changes
and `data/` afterwards, checks the acceptance sentence by hand, and
commits; your report is what it restores from.

## Rules that bind you

- Read first, in this order: `AGENTS.md`; `README.md` and `CONTEXT.md`
  as it says; `search/README.md` ("Rules in force", "Traps");
  `.claude/skills/site-sitting/SKILL.md` whole; this track's
  `README.md` by section — the status line, "The percentile", "The
  percentile's round", "The ranged family on the copy", "Open
  questions"; `MANIFEST.md`, the round's row; the docstrings of
  `tools/trade-captures.py`, `tools/trade-evidence.py`,
  `scripts/percentile.py`, `scripts/percentile-shapes.py`;
  `search/BUILD-PLAN.md`, the foot ("Holes not yet ruled"), J7; the
  contract detail under C101 in `search/DESIGN.md` (J5 and J6 as ruled
  2026-10-01); `tools/trade-sheet.py`, `round_i` only — what each
  search says it decides.
- Work inside `search/pseudo-stats/`. Do not run `tools/trade-sheet.py`
  or `tools/trade-pages.py` for any round but `i`, and do not edit any
  file under `tools/`: another run is adding a round to the sheet in
  parallel and regenerates `data/search-sheet.csv`; you read that file
  and never write it. Never `search/README.md` (the index), another
  track's files, `CONTEXT.md`, `decisions/`, `SURFACES.md`, the crates,
  `MANIFEST.md` (its row is written).
- Never commit, push, fetch or spawn. No network (C79). The owner's
  store is never opened; the copy is read-only as before. Do not run
  `acq` or `cargo`.
- Nothing is recalled: every number is printed by a script after the
  last edit. Add to the README by section, about 2 KB at most; rewrite
  nothing already there except the status line and the round's own
  section.

## The work

1. `python3 tools/trade-captures.py --check`, read; then without the
   flag: `data/captures.json` gains round i's items, scrubbed, the
   guard refusing seller data. Then `python3 tools/trade-evidence.py`:
   `data/evidence.csv` against every pseudo's latest rows — round i's
   plain ranged pseudos among them. Then `python3 scripts/percentile.py`:
   `data/percentile-captures.csv` now scores round i's armour items.
   Each run bare, its exit read.
2. Read what each search decided, by its own "decides" text and by the
   scores: i01 (C1, three rolls on Sacrificial Garb — the site's value
   against the average and against min, max and the first type); i02
   (C7, the enchant: the percentile and the 20%-quality figures with
   the quality term dropped); i03 and i04 (C10: 0 found at 101, 10,000
   at 100 — whether any fetched item's display no roll in range gives,
   which is a roll over the maximum shown clamped); i05 (C14: whether
   the site's value leaves a defence line under its global twin out or
   reads it); i09 (C2, ward). Under J5 as ruled, a display at an exact
   half admits both integers. For each: pinned, and by which items; or
   not pinned, and what would.
3. i06 and i08 found nothing and i07, their mutant, was not opened: a
   search that finds nothing proves nothing until its mutant finds
   something (the skill). Say so in the round's section, say nothing
   of J7 beyond it, and write the page of what is still missing:
   `python3 tools/trade-pages.py i`, then `--check`; its path goes in
   your report for the owner.
4. The README: the status line; "The percentile's round" gains what
   each search decided, in one table, and which of J6's shapes are now
   pinned; "Open questions" gains none unless a capture raised one,
   each naming the one read that closes it. Written last, from the
   script outputs.

## Acceptance

`tools/trade-captures.py --check`, `tools/trade-evidence.py` and
`scripts/percentile.py` each exit 0 and regenerate their files byte for
byte on a second run; `data/percentile-check.csv` and
`data/candidates.csv` are unchanged from `HEAD`; every number in the
README's new rows is printed by one of them; `tools/trade-pages.py i
--check` exits 0 over the page holding i07 alone.

## The report

At most 300 words, to the reviewer: which of J6's five shapes are
pinned and by what; what i05 said of the defence twin; what i03 and
i04 said of a roll over the maximum; the page's path for i07; what you
left out and why; the one cut you would most want reversed; and that
the index row in `search/README.md` is not set.
