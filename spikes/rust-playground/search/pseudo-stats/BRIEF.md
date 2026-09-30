# pseudo-stats — the brief for the run before 9d (2026-09-30)

Two of step 9d's holes (`search/BUILD-PLAN.md`, the 9d row) have their
inputs on this machine and belong to this track: the base defence
percentile's shapes no capture pins, and the ranged family's census
over the owner's copy (this README's open question 7). This run
prepares the sitting for the first and closes the second. It does not
sit: the owner opens every link himself, later (the site-sitting
skill), and the captures are read under a brief of their own. The
reviewer — the session that launched you — reads your README changes
and `data/` afterwards, checks the acceptance sentence by hand, and
commits; your report is what it restores from.

## Rules that bind you

- Read first, in this order: `AGENTS.md`; `README.md` and `CONTEXT.md`
  as it says; `search/README.md` ("Rules in force" and "Traps");
  `.claude/skills/site-sitting/SKILL.md` whole — "Before the sitting"
  is your procedure and its traps bind; this track's `README.md` by
  section: the status line, the headline, "The percentile", "The
  ranged family by what the export says is local", "Open questions",
  "Provenance"; `MANIFEST.md`; the docstrings of `tools/trade-sheet.py`,
  `tools/trade-pages.py`, `tools/trade_rows.py` and
  `scripts/percentile.py`; `decisions/search.md` C101 (its contract
  detail in `search/DESIGN.md`), C94, C102, C106; `search/LEDGER.md`,
  "Holes ruled", the seat row (V5, V6) and the 9c rows; the module doc
  of `crates/acquisition-search/src/totals.rs` ("What waits").
- Work inside `search/pseudo-stats/`, with one exception the sitting
  skill requires: you may add to `tools/trade-sheet.py` — a new round
  is a function that pins versions by number, added to the `site`
  sheet in `main()`, and nothing already there is edited; the earlier
  rows of `data/search-sheet.csv` stay byte-identical. Write nothing
  else outside the track: never `search/README.md` (the index — its
  row is the reviewer's), another track's files, `CONTEXT.md`,
  `decisions/`, `SURFACES.md`, the crates, the totals table or its
  generator.
- Never commit, push, fetch or spawn. No network at all (C79): a link
  is composed and laid on a page; no tool opens one. The inputs are the
  committed captures and the owner's pinned clones, which every script
  refuses at any other commit.
- The owner's store is never opened. The copy under
  `search/item-facts/raw/m3/store/mock/GERWARIC_7694.db` is read with
  sqlite in read-only mode (`file:…?mode=ro`); it may be absent, and
  then every count that needs it is an open question. Do not run `acq`
  or `cargo`.
- Nothing is recalled: every number written is printed by a script
  after the last edit. Every claim points at a file and a row. The
  README already runs twice the index's guide: add about 3 KB at most,
  by section, with the detail in `data/`; nothing already there is
  rewritten except the status line and open question 7's row.

## Part one — the percentile's round, prepared

The site's stated rule reproduces 30 of 30 captured values ("The
percentile"); the shapes no capture reaches are C1–C8, already drafted
as `PERCENTILE_CASES` in `tools/trade-sheet.py` and never written to
the site sheet.

1. Read each drafted case against what the captures have taught since
   it was written (the README's "The percentile" and the rows of
   `tools/trade_rows.py`): whether its query still isolates the shape
   it names, whether it carries a control (`PERCENTILE_CONTROL`), and
   whether the eight are the shapes 9d needs — the contract detail
   under C101 names a realm with no table, a roll over the base's
   maximum, and a quality-normalised reading beside them. A shape
   missing gets a case; a case that isolates nothing is said so and
   left out, with why.
2. Write the round: the cases as a round of the `site` sheet, named
   as `trade-pages.py` reads a round's rows (a letter, then digits),
   each row saying what it decides. Then, each run bare and its exit
   read: `tools/trade-sheet.py`, `--verify`, `--self-test`; `cmp` of
   the sheet's earlier rows against `git show HEAD:` of the file;
   `tools/trade-pages.py <round>` and `--check`.
3. Extend `scripts/percentile.py` so that it also scores every item of
   `data/captures.json` that carries `extended.base_defence_percentile`,
   under the same three formulas, with nothing changed for the 30 it
   scores today (`data/percentile-check.csv` regenerates
   byte-for-byte for those rows). After the sitting the new captures
   are read by this script and no other.
4. `scripts/percentile-shapes.py` → `data/percentile-shapes.csv`: the
   copy's live armour items by shape (each C case, and "none of them"),
   how many, and for each shape whether the stated rule reads the item
   — a value, or what it cannot read, at which grain (rule 8 of the
   plan) — with the bases the class table and `base-defences.csv` do
   not hold counted apart. This is what the build's fixtures are cut
   from; it decides nothing about the site.

## Part two — the ranged family's census (open question 7)

`scripts/ranged-census.py` → `data/ranged-census.csv` and
`data/ranged-uniques.csv`:

5. Every live line on the copy whose template is one of the ranged
   family's unsuffixed texts (`Adds # to # <type> Damage`, the five
   types; `search/item-facts/data/mod-templates.csv` is the census's
   spelling), by the item's class (the shipped `classes-v1.toml`), by
   whether that class is a weapon on the site
   (`search/repoe/data/class-to-trade-category.csv`, the `weapon.*`
   ids), by rarity, by source array and flags. The one number the
   plan's row needs is how many such lines sit on weapons and how many
   do not.
6. For every unique item carrying such a line: the unique's name and
   base, and whether the export says the mod behind the line is the
   local stat or the global one (`poe1/data/mods.json` at `e2bd511a`,
   the unique-domain mods, their stats' `is_local` in `stats.json`;
   the join by the line's text, as `search/repoe/scripts/` join text).
   Where the join cannot be made for an item, the row says so and why;
   the count of those is the finding's limit, not a guess.
7. If the census shows a weapon whose line the export says is global,
   or cannot say, compose one search that would ask the site whether
   such an item shows the plain pseudo — sound in the skill's sense,
   with its control and its mutant — and add it to the round. If the
   census shows none, say so and compose nothing.

## Outputs

- `tools/trade-sheet.py`: the one round added; `data/search-sheet.csv`
  regenerated, earlier rows identical.
- `raw/sitting/<round>-1.html` (and further pages), local only;
  `MANIFEST.md` unchanged unless you read a raw file it does not list.
- `scripts/percentile.py` extended; `scripts/percentile-shapes.py`,
  `scripts/ranged-census.py` new; their `data/` files, each regenerable,
  first line naming the script and its inputs.
- `README.md`: the status line; a short section "The percentile's
  round" (the searches, what each decides, the cases left out and why)
  and "The ranged family on the copy" (one table); open question 7's
  row closed with its number and its file; provenance rows for what
  is new. Written last, from the script outputs.

## Acceptance

`tools/trade-sheet.py`, `tools/trade-sheet.py --verify` and
`tools/trade-sheet.py --self-test` each exit 0; `cmp` of the sheet's
earlier rows against `HEAD` finds no difference;
`tools/trade-pages.py <round> --check` exits 0; the three scripts
regenerate their `data/` files byte-for-byte; and every number in the
README's new rows is printed by one of them.

## The report

At most 300 words, to the reviewer: how many searches the round holds,
which are its core, the pace, and the first page's path — the owner
sits from that line; the ranged family's one number; what you left
out and why; the one cut you would most want reversed; the inputs
that were absent; and that the index row in `search/README.md` is not
set, since it is forbidden you.
