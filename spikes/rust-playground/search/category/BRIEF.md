# category — the brief for the run (2026-09-30)

You are running one research track of the item-search slice:
`search/category/`. The question is `README.md`'s; this brief is the
procedure. The reviewer — the session that launched you — reads your
README and `data/` afterwards, checks the acceptance sentence by hand,
and commits; your report is what it restores from.

## Rules that bind you

- Read first, in this order: `AGENTS.md`; `README.md` and `CONTEXT.md`
  as it says; `search/README.md` ("Rules in force" and "Traps" bind
  this directory); `decisions/search.md` — C106 and its contract
  detail in `search/DESIGN.md` ("C106 — what reference data admits,
  and the test"), C101, C93, C105, and the park "A grouping above
  class"; `search/LEDGER.md`, "Holes ruled", the seat row (V1) and
  the step 6 row (G1–G6); `SEARCH-SLICE.md`, "Observations still
  open", "The first seat" (the category paragraph); the module doc of
  `crates/acquisition-search/src/class.rs` and the docstring of
  `tools/class-table.py` (the shape a generated table already takes);
  `search/repoe/README.md` F3 and `search/repoe/scripts/base-taxonomy.py`;
  `brainstorming-notes/31-search-reconciliation.md`, K7 only.
- Work only inside `search/category/`. Read anything; write nothing
  outside it: never `search/README.md` (the index — its row is the
  reviewer's, and your report says so), another track's files,
  `CONTEXT.md`, `decisions/`, `SURFACES.md`, the crates, `tools/`.
- Never commit, push, fetch or spawn. No network at all (C79): the
  inputs are the owner's pinned clones and the committed captures. A
  script that reads a clone refuses any other commit, as
  `base-taxonomy.py` does. poedb is unregistered and the wiki is
  corroboration from a saved copy only: a question that needs a surface
  not on this machine is an open question naming the surface and the
  one read that closes it, never a fetch.
- The owner's store is never opened. The copy under
  `search/item-facts/raw/m3/store/mock/GERWARIC_7694.db` is read with
  sqlite in read-only mode (`file:…?mode=ro`); it may be absent, and
  then every count that needs it is an open question, not a stalled
  run. Do not run `acq` or `cargo`: this is a read of data.
- Nothing is recalled: every number in the README is printed by a
  script in `scripts/` after the last edit. Every claim points at a
  file and a row.
- The README is written last, from the script outputs, its headline
  block last of all; per-item detail lives in `data/`. Guide: the
  README about 12 KB (index rule 3); findings are tables; every open
  question names the one experiment or read that closes it.

## Inputs, by path

- The trade site, captured 2026-09-12 (`search/trade-query/data/`,
  its `MANIFEST.md`): `filters-2026-09-12.json` — the category ids
  with their texts (`type_filters.category`); `items-2026-09-12.json`
  — the 22 coarse categories and their base types, the `monster`
  category among them; `grammar.json`, `item_categories`.
- The export, `search/repoe/data/class-to-trade-category.csv` and
  `base-taxonomy.csv`, and behind them the clone `poe1` at
  `e2bd511a` (`search/repoe/MANIFEST.md`: `base_items.json`,
  `item_classes.json`, and any other file of `poe1/data/` that says
  what a beast is — record what you read in `MANIFEST.md` here).
- The shipped class table, `crates/acquisition-search/reference/classes-v1.toml`.
- Awakened PoE Trade at `ce551eb7` (`search/prior-art/MANIFEST.md`):
  how its parser and its trade-query builder treat a captured beast
  and the site's categories, cited by file and line.
- Path of Building at `16de4b82`, only if it says something of beasts
  or categories the others do not.
- The copy: every live `pc` item's body (`items.json`, the `json`
  column) — its `baseType`, frame, name, properties (a beast carries
  `Genus`, `Group`, `Family`), `descrText`, and whatever else varies.
- The seat's report, `runs/seat-2026-09-26/REPORT.md`, section 4, V1,
  local and gitignored; may be absent.

## Outputs

- `scripts/categories.py` → `data/categories.csv`: one row per
  category id the site's filter lists — its text, its parent where the
  id is dotted, how the row is produced (a class, a name rule on the
  base, a rule on the body, or nothing produces it), the classes and
  rules named, the copy's item count under it, and the site's base
  types under it that agree or disagree with the prediction.
- `scripts/beasts.py` → `data/beasts.csv`: one row per beast base on
  the copy — items, the frames and rarities seen, `Genus`, `Group`,
  `Family`, the name pattern, `descrText`, and which of the three
  leaves each candidate reading would give it; and `data/beast-facts.csv`,
  one row per body fact that varies across the copy's beasts, with its
  values and counts.
- `data/residue.csv`: every live item on the copy that no reading
  places under exactly one id — the base, the count, and why (no base
  in any table, a base under several ids, a class with no id).
- `MANIFEST.md`: one row per raw or clone input read, pointing at the
  manifest that already describes it.
- `README.md`, last.

## Findings expected

Answers, not restatements: measure each and say what was found, and
where a fact is not on this machine say which read would settle it.

1. Whether every id the site lists is a function of the game's class
   and a reviewed rule on the base name, from the pinned export — and
   the residue named by id.
2. How the dotted parents relate to the leaves in the site's own data:
   leaves per parent, any leaf under two parents, any id under
   neither; whether a parent is anything but the set of its leaves.
3. What a captured beast is in the private API: which facts of its
   body vary across the copy's beasts, and which of them, alone or
   together, sort the beasts so that what the site lists under
   `monster` agrees. If nothing on this machine says which leaf a beast
   type belongs to, say so and name the sitting search that would.
4. Whether the game's own data gives a beast a class (a base, a class
   with a display name) or does not.
5. Every live item of the copy placed by the readings above: the
   buckets — one id, none, several — summing to the copy, so that C105's
   shape is visible before any table is written.
6. The classes with no id on the site (F3 counted 31) and what the copy
   holds of them, by class.

## Acceptance

`python3 search/category/scripts/categories.py` and
`python3 search/category/scripts/beasts.py` regenerate `data/`
byte-for-byte; every count in the README is printed by one of them;
and every category id of `filters-2026-09-12.json` has a row of
`data/categories.csv` saying how it is produced or that nothing
produces it.

## The report

At most 300 words, to the reviewer: the headline in two lines; what
you left out and why; the one cut you would most want reversed; the
open questions by number; the inputs that were absent; and that the
index row in `search/README.md` is not set, since it is forbidden you.
