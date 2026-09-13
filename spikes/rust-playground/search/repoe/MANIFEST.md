# Evidence manifest — repoe

Every input this track reads, and how it was obtained (index rule 4). Nothing here is fetched by tooling: the owner cloned three repositories beside this one on 2026-09-13 (`SURFACES.md`, the RePoE and Path of Building rows; the owner, verbatim: "Instead of doing a fetch, I can clone both repos locally" … "I have cloned both projects into directories parallel with acquisition." … "I have cloned poe1 as well."). Tooling reads the checkouts at the commits below; a reading against another commit is a new capture and a new row. Whole data files are never committed; extracts under `data/` cite the commit.

## The clones (`../../../../<name>` from this directory: siblings of `acquisition/`)

| Clone | Remote | Commit | Date | Describe | Role |
| --- | --- | --- | --- | --- | --- |
| `repoe/` | `https://github.com/repoe-fork/repoe.git` | `1c72e40a99aa8c8f70e9a3526155f455d5eedd43` | 2026-08-27 | `1c72e40a` | the parser, schemas, and data history to 2025-06-07 |
| `poe1/` | `https://github.com/repoe-fork/poe1.git` | `e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9` | 2026-09-09 | `e2bd511a` | the PoE1 export with per-patch history |
| `PathOfBuilding/` | `https://github.com/PathOfBuildingCommunity/PathOfBuilding.git` | `16de4b82d57f1c0de6eb40f37143c32d4da36a02` | 2026-09-08 | `v2.67.2-75-g16de4b82d` | Path of Building Community, dev branch |

`poe1/version.txt` = `3.29.3.3` (game version the export was generated from; its history is one commit per patch since 2025-06-07).

## The PoE1 export's files read by this track (`poe1/data/`, 30 JSON files at the root plus `base_items/`, `stat_translations/` and per-language subdirectories)

| File | Bytes | Read for |
| --- | --- | --- |
| `mods.json` | 34,279,067 | mods: stats with ranges, domain, generation type, spawn weights, tags, pre-rendered text |
| `mods_by_base.json` | 21,245,791 | which mods roll on which base |
| `stat_translations.json` | 12,631,258 | stat ids → display strings with `#` formats; the `trade_stats` join |
| `base_items.json` | 8,025,343 | bases: item class, tags, implicits, requirements, release state |
| `stats.json` | 4,372,583 | per-stat locality and aliasing; the numeric stat hashes if present |
| `mod_types.json` | 1,203,551 | mod type → tags |
| `uniques.json` | 591,061 | unique names |
| `stat_value_handlers.json` | 250,281 | value handlers named by the translations |
| `tags.json` | 32,003 | the tag vocabulary |
| `item_classes.json` | 17,659 | classes and categories |

Other files in the export (gems, areas, audio, buffs, …) are not read by this track.
