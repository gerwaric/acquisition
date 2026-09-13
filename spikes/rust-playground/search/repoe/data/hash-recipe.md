# The trade stat id: Path of Building's recipe, what the export lacks, and what the join shows

Read from `PathOfBuilding/src/Export/Scripts/mods.lua` (lines 161–190) and `src/Modules/Common.lua` (`murmurHash2`, `intToBytes`) @ 16de4b82; checked by `scripts/hash-check.py`, whose outputs are `hash-check.csv` and `stat-hashes.csv`.

## The recipe

For each stat of a mod, in `Mods.dat` column order (`Stat1`…`Stat6`): take the stat's `Stats.dat` `Hash` column as four little-endian bytes; when the stat id contains `minimum` and the next stat's contains `maximum`, append the next stat's four bytes and consume both (the `Adds # to #` lines). The trade number is MurmurHash2 (32-bit, Appleby; multiplier `0x5BD1E995`, rotate 24, seed **`0x02312233`**) of those bytes. Path of Building writes the result per mod into `src/Data/Mod*.lua` as `tradeHashes = { [number] = { description lines } }`.

The implementation in `hash-check.py` reproduces SMHasher's verification value for MurmurHash2 (`0x27864C1E`), so the function is the reference one.

## What the export lacks

RePoE's `stats.json` carries `is_local`, `is_aliased` and the aliases for 23,346 stats and nothing else: its parser (`RePoE/parser/modules/stats.py`) reads `IsLocal`, `IsWeaponLocal` and the two alias columns of `Stats.dat64` and never the `Hash` column (which Path of Building's `spec.lua` lists). No other file of the export carries it. The hash is not a function of the id string either: 19 common string hashes (FNV-1/1a 32 and 64, MurmurHash2 and 3 with seeds 0 and `0x02312233`, CRC32, Adler32, djb2, sdbm, Java, one-at-a-time; each plain, NUL-terminated and lower-cased) reproduce none of 6,678 single-stat trade numbers. **The recipe cannot run from the export alone.**

## What can be recovered without the column

MurmurHash2 restricted to a 4-byte key is a bijection on 32-bit values (every step — multiply by an odd constant, xor-shift — inverts), so a single-stat trade number *is* the `Stats.dat` hash, encoded. `stat-hashes.csv` inverts every single-stat number the export's text join or Path of Building attributes to a stat id: 6,958 stat ids with their hash, re-hashed to check. A pair line (two stats, eight bytes) does not invert — one equation, two unknowns — and the paired damage stats never appear alone, so their hashes stay unknown; their trade numbers are known from the join regardless.

## The join, under the acceptance rule

The owner's rule (2026-09-13): "we need exact matching, or we need to be able to deterministically determine when it fails." Every one of Path of Building's 29,288 (mod, number) rows lands in a class:

| Class | Rows | Distinct numbers | Meaning |
| --- | ---: | ---: | --- |
| reproduced | 23,400 | 4,745 | the export's text join gives the same number for the same stat group |
| reproduced, ambiguity resolved | 983 | 175 | the text join gave the group several numbers (a text collision); the hash picks one |
| explained: not in the capture | 4,479 | 423 | the recipe minted a number the site does not list — the recipe's own failure, checkable against the capture |
| explained: no translation entry for the group | 352 | 129 | Path of Building hashed a hidden or condition stat (`local_influence_mod_requires_…`) the export renders nowhere |
| explained: text join failed, the hash supplies it | 74 | 19 | the translation exists, renders differently from the site's text, and the hash names the id the text could not |
| unexplained | 0 | 0 | |

So: the text join is right wherever both sources speak; where they disagree the reason is mechanical and named. The stronger reading is the other way round — of the 5,489 distinct numbers Path of Building minted, 423 (7.7 %) do not exist on the site, so the recipe is *not* exact and the capture is the arbiter of what a trade id is.

Two deterministic detectors fall out:

- **A stat id with two numbers has at most one right.** One stat has one hash, so when the export's text join attaches two single-stat numbers to one id (123 ids), one of them came from a colliding text. Path of Building's hash picks the right one for 90 of them; for 32 it lists neither (its recipe failed on that mod) and both stay suspect.
- **A text with two ids is one hash each.** The trade site's 380 (category, text) collisions are two hashes under one rendering; the join attributes 175 of them to their stat groups. The remainder are collisions the export cannot see (ids for lines it renders identically, or hidden stats).

## What this means for the track

The id mapping is a text join corrected by a hash table: `trade-stat-map.csv` from the export, `stat-hashes.csv` as the correction where the join is ambiguous, and the capture as the list of what exists. Path of Building stays corroboration — its per-mod tables were the oracle for this check and are not a runtime input.
