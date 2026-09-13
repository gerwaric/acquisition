# Evidence manifest — repoe

Every input this track reads, and how it was obtained (index rule 4). Nothing here is fetched by tooling: the owner cloned three repositories beside this one on 2026-09-13 (`SURFACES.md`, the RePoE and Path of Building rows; the owner, verbatim: "Instead of doing a fetch, I can clone both repos locally" … "I have cloned both projects into directories parallel with acquisition." … "I have cloned poe1 as well."). Tooling reads the checkouts at the commits below; every script refuses to run against another commit — a pull is a new row here first. Whole data files are never committed; extracts under `data/` cite the commit on their first line.

## The clones (`../../../../<name>` from this directory: siblings of `acquisition/`)

| Clone | Remote | Commit | Date | Describe | Role |
| --- | --- | --- | --- | --- | --- |
| `repoe/` | `https://github.com/repoe-fork/repoe.git` | `1c72e40a99aa8c8f70e9a3526155f455d5eedd43` | 2026-08-27 | `1c72e40a` | the parser, schemas, and data history to 2025-06-07 |
| `poe1/` | `https://github.com/repoe-fork/poe1.git` | `e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9` | 2026-09-09 | `e2bd511a` | the PoE1 export with per-patch history |
| `PathOfBuilding/` | `https://github.com/PathOfBuildingCommunity/PathOfBuilding.git` | `16de4b82d57f1c0de6eb40f37143c32d4da36a02` | 2026-09-08 | `v2.67.2-75-g16de4b82d` | Path of Building Community, dev branch |

`poe1/version.txt` = `3.29.3.3`; `exported-version.txt` = `634a70498bc75676f9c938a3d590c18ee75cae8d refs/heads/main` (the game-data commit the export was generated from). Its history: 590 commits since 2025-06-07, one `Version <v>` commit when the fork's poll sees a new game version and one `Export <v>` when the export runs (README F6 measures the lag from these).

## The PoE1 export's files read (`poe1/data/`)

| File | Bytes | Read by |
| --- | --- | --- |
| `stat_translations.json` | 12,631,258 | `trade-stat-map.py`, `hash-check.py`, `template-vs-translation.py` — stat ids → English strings; the `trade_stats` the generator joins by text (it fetches `/api/trade/data/stats` live at each export, `stat_translations.py` line 308) |
| `mods.json` | 34,279,067 | `hash-check.py`, `mod-stat-index.py` — mod id → ordered stats with ranges, domain, generation type, weights, tags |
| `base_items.json` | 8,025,343 | `base-taxonomy.py` — base → class, tags, release state, domain |
| `item_classes.json` | 17,659 | `base-taxonomy.py` — the 103 classes, their display names and 66 categories |
| `stats.json` | 4,372,583 | by hand (README F2): locality and aliases only; no hash column |

Files scoped but not read by an extract: `mods_by_base.json` (which mods roll on which base; the design's, when it asks), `mod_types.json`, `uniques.json`, `stat_value_handlers.json`, `tags.json`, `stats_by_file.json`.

## The parser's files read (`repoe/RePoE/parser/modules/`), by hand

| File | Bytes | Read for |
| --- | --- | --- |
| `stat_translations.py` | 15,671 | the trade join: rendered text, `(Local)`/`(Maps)`/`(Legacy)`/`(Staves)` suffix rules, the per-line fallback for multi-line strings, the digits-to-`#` fallback; the live fetch of the stats endpoint |
| `stats.py` | 1,319 | what `stats.json` carries: `IsLocal`, `IsWeaponLocal`, the two alias columns — never `Hash` |

## Path of Building's files read (`PathOfBuilding/src/`)

| File | Bytes | Read by |
| --- | --- | --- |
| `Data/Mod*.lua` (23 files) | 15,340 KB | `hash-check.py` — one mod per line: id, description lines, `tradeHashes` |
| `Export/Scripts/mods.lua` | 13,385 | by hand — the recipe (lines 161–190), `hash-recipe.md` |
| `Modules/Common.lua` | 32,029 | by hand — `murmurHash2`, `intToBytes` (lines 332–371) |
| `Export/spec.lua` | 189,917 | by hand — the `Stats` table lists a `Hash` column |
| `Classes/Item.lua` | 105,512 | by hand — `BuildRaw`, `ParseRaw`, `pob-format.md` |
| `Data/TradeSiteStats.lua` | 2,762,047 | by hand — a copy of the stats endpoint at Path of Building's 2026-08-23 export, used once to confirm the site lists a renamed line twice under one id |
| `Data/QueryMods.lua` | 1,407,322 | scoped, not read by an extract (mod id → trade mod, corroboration) |

## Sibling tracks' files read

| File | Track | Read by |
| --- | --- | --- |
| `data/stats-2026-09-12.json`, `items-2026-09-12.json`, `filters-2026-09-12.json` | trade-query (`MANIFEST.md` there) | `trade-stat-map.py`, `hash-check.py`, `base-taxonomy.py` |
| `raw/searches/*-fetch.json` (local) | trade-query | `base-taxonomy.py` — the `Item Class:` clipboard line of 46 listed base types |
| `data/mod-templates.csv` | item-facts | `template-vs-translation.py` |
| `raw/spike-…db`, `raw/cpp-userstore-…db` (local), `scripts/census.py` | item-facts | `base-taxonomy.py` (the corpus's `baseType` values), `template-vs-translation.py` (imports the census's template rule to split lines by frame type) |

## The owner's site check (access method `browser`, the trade-site row of `SURFACES.md`)

| File | Captured | What |
| --- | --- | --- |
| `data/twice-numbered-verdicts.csv` | the owner, 2026-09-13, in a browser: Standard league, pc, status any; 68 searches, the URLs of `twice-numbered-urls.txt` | `found` yes/no per (stat id, trade number): whether the search returned any listing. Hand-kept; no script writes it |

The master-side file read by hand for `pob-format.md`: `master:src/item.cpp` @ 946a4f51, `Item::POBformat` (line 681).
