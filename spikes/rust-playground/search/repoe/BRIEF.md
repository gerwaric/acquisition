# Brief — repoe, one added extract: base defence ranges (run by a subagent)

You are an Opus subagent adding one extract to a closed research track
of the item-search slice, under `search/README.md`, "Rules in force".
Read this brief, then `search/README.md` (rules and traps), then
`search/repoe/README.md` and `search/repoe/MANIFEST.md`, then
`search/repoe/scripts/base-taxonomy.py` as the model for conventions.
Work only inside `search/repoe/`. Never touch the index
(`search/README.md`), another track, `CONTEXT.md`, `decisions/`,
`SURFACES.md`, `brainstorming-notes/`, `audits/`. Never commit, push,
fetch, run a network request or spawn an agent. The reviewer sets the
index row and commits; this brief is deleted at the close, cited by hash.

## Why

The stage-5 audit (2026-09-18) left the base-defence percentile — the one
`computed` pseudo the site names — unresolved because this track's five
extracts never pulled the per-base defence ranges, and the pseudo-stats
runner was allowed only this README as its input. The ranges are in the
export: `base_items.json` carries `properties.armour`, `.evasion`,
`.energy_shield` (and `.ward`, `.block`) as `{min, max}` or null on every
base (the reviewer checked: 466 bases carry a non-null defence; Splintered
Tower Shield is armour 9–12). The owner chose RePoE over poedb for this,
2026-09-18. This extract makes the ranges a committed, regenerable table.

## Input (one file, read-only)

`../../../../../poe1/data/base_items.json` at the manifest's commit
`e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9` — the script refuses another
commit, exactly as `base-taxonomy.py` does (copy its commit check and
path derivation).

## Outputs

- `scripts/base-defences.py` — the track's conventions: a docstring
  naming inputs and outputs, the commit pin, `lineterminator="\n"`, the
  CSV's first line naming the script and its input (as every `data/*.csv`
  here does), then the header. Prints: bases with any non-null defence,
  and a per-class count of them.
- `data/base-defences.csv` — one row per base that carries at least one
  non-null defence or block value: `name, item_class, release_state,
  domain, armour_min, armour_max, evasion_min, evasion_max,
  energy_shield_min, energy_shield_max, ward_min, ward_max, block`,
  empty where null. Bases with every defence null are left out (say how
  many in the README line).
- `README.md` — one row in the Outputs table and one finding paragraph
  (`F7 — Base defences`, ≤ 600 bytes: how many bases, per class in one
  short table or a sentence, and that the base-defence percentile is
  computable from this table plus the item's displayed defence — the
  site's `extended.base_defence_percentile` exists on fetched items,
  `search/trade-query/data/fetch-census.json`, if you want one sentence of
  corroboration; do not compute it here). The README is already at its
  budget; add nothing else.
- `MANIFEST.md` — the `base_items.json` row gains this script in its role.

## Acceptance

`python3 search/repoe/scripts/base-defences.py` regenerates
`data/base-defences.csv` byte-for-byte; Splintered Tower Shield appears
with armour 9 and 12; every row has at least one non-empty defence or
block value; the printed count equals the CSV's row count.

## The report (at most 200 words)

The counts; what you left out; the one cut you would most want reversed.
