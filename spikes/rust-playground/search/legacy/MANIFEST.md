# Evidence manifest — legacy

Every input this track read (index rule 4). `raw/` is local only, gitignored and never committed;
`data/` is regenerable from `raw/` and the clones. Both stores hold the owner's own account: they
are account data, and nothing beyond an item's name, base and mod lines leaves them. Nothing was
fetched — every read is a local file, or a local `git log` / `git show` in a clone.

## `raw/` — the store copies

Captured by the reviewer on 2026-09-18 with `sqlite3 .backup` (the index's trap: never `cp`, the
stores are under WAL). Both hashes equal the 2026-09-13 copies of `../item-facts/MANIFEST.md`, so
neither store has changed since that track ran.

| File | Bytes | Source | sha256 |
| --- | --- | --- | --- |
| `raw/spike-GERWARIC_7694-2026-09-18.db` | 54378496 | `~/Library/Application Support/gerwaric.acquisition-playground/store/ggg/GERWARIC_7694.db` — the spike's facts store | `fbae8786d79c67e3d9f277bae3e8ce60382f55eeabc2dc5bdb5f89965bccfda7` |
| `raw/cpp-userstore-GERWARIC_7694-2026-09-18.db` | 57196544 | `~/Library/Application Support/Acquisition/data/userstore-GERWARIC#7694.db` — the C++ app's store, which holds Standard tabs the spike has not fetched | `65f1622c3fa0dedc1229c8b5b778076f1beae2077ce89c7997f3d2b98b7ac0d1` |

## The clones read (siblings of `acquisition/`, `SURFACES.md`, access method `clone`)

The commits are the ones `../repoe/MANIFEST.md` and `../prior-art/MANIFEST.md` record; each was
re-read here with `git -C <clone> rev-parse HEAD`, and `scripts/common.py` refuses to run when
Path of Building is at any commit but the one below.

| Clone | Commit | Files read |
| --- | --- | --- |
| `PathOfBuilding/` | `16de4b82d57f1c0de6eb40f37143c32d4da36a02` | `src/Data/Uniques/*.lua` (22) and `src/Data/Uniques/Special/*.lua` (5) — the entries; `src/Data/Bases/*.lua` — the base names the parse checks a base line against; `src/Classes/Item.lua` — `ParseRaw`, for which lines its own parser reads as headers (535-561, 660-860), the catalyst list, descriptors and kind-to-tags table with `getCatalystScalar` (14-62, 673-677, 1082-1088), the alt-variant headers and choices (752-783) and `CheckModLineVariant` / `GetModLineVariantCount` (2120-2164); `src/Data/EnchantmentHelmet.lua`, `src/Data/Crucible.lua` — for the sources read |
| — read by the second pass | the same commit | `src/Modules/ItemTools.lua` — `applyValueScalar`, `formatValue` and `applyRange`, which decide how a catalyst scalar reaches a number (37-75, 96-355); `src/Modules/Common.lua` — `roundSymmetric`, `alwaysPositiveRound`, `floorSymmetric` (733-780); `src/Data/ModScalability.lua` — 15,202 keyed lines saying per number whether it scales and how it is formatted, loaded by `src/Modules/Data.lua` line 435 (`data.defaultHighPrecision` is line 434) |
| `poe1/` | `e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9` | `data/mods.json`, `data/uniques.json`, `data/stats.json` — for the sources read; `git log` / `git show` over `data/mods.json` |
| `repoe/` | `1c72e40a99aa8c8f70e9a3526155f455d5eedd43` | `git log` / `git show` over `RePoE/data/mods.json`, which the working tree no longer carries |
| `awakened-poe-trade/` | `ce551eb7a9b704fbdcc2478eebb26be8f91786c7` | `renderer/public/data/en/stats.ndjson`, `items.ndjson` — for the sources read |

Path arithmetic: the brief writes the clones as `../../../../PathOfBuilding/` from this directory;
from `search/legacy/` that path lands on `acquisition/`, and the scripts use one level more.

## Sibling tracks' files read

| File | Track | Read for |
| --- | --- | --- |
| `../item-facts/scripts/census.py` | item-facts | the store read: dedupe by GGG item id with the newer fetch winning, socketed gems lifted to their own rows, the icon-directory rule. Copied into `scripts/common.py`, never edited |
| `../item-facts/data/mod-templates.csv`, `field-census.csv` | item-facts | which mod arrays and fields the corpus carries, and that `frameTypeId` is a string in both stores |
| `../item-facts/data/properties-census.csv` | item-facts | **not listed in the brief**: the `Quality (… Modifiers)` property names the catalyst rule keys on, and that `Quality (Quantity)` is a map property the rule does not catch |
| `../item-facts/MANIFEST.md`, `../repoe/MANIFEST.md`, `../prior-art/MANIFEST.md` | item-facts, repoe, prior-art | the store hashes to compare against, and the clones' commits |
| `../DIGEST.md` | — | S165 and S178 only |

## `data/` — regenerable

| File | Written by | Bytes |
| --- | --- | --- |
| `data/pob-variants.csv` | `scripts/variants.py` | 278230 |
| `data/pob-parse-refusals.csv` | `scripts/variants.py` | 1885 |
| `data/unique-fit.csv` | `scripts/match.py` | 519620 |
| `data/other-arrays.csv` | `scripts/arrays.py` | 112197 |

Each run rewrites all four byte for byte (checked twice, `shasum -a 256`). A scrub guard in
`scripts/common.py` refuses to write a `data/` file carrying the account name or a 32-plus-character
hex run; the per-item key is `sha256("legacy:" + id)[:12]`, never the GGG item id.
