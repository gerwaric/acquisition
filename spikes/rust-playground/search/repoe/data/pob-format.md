# The Path of Building import grammar against the C++ app's export

Read by hand from `PathOfBuilding/src/Classes/Item.lua` @ 16de4b82 (`ItemClass:BuildRaw`, lines 1772–2049, the writer; `ItemClass:ParseRaw`, line 424, the reader) and `master:src/item.cpp` @ 946a4f51 (`Item::POBformat`, line 681; driven by `OnCopyForPOB` in `src/ui/mainwindow.cpp`). One table: the lines in the order `BuildRaw` writes them, what the C++ app writes for each, and what a stash item carries for it.

| Order | Path of Building writes | C++ app writes | The item JSON has it as |
| --- | --- | --- | --- |
| 1 | `Rarity: <NORMAL\|MAGIC\|RARE\|UNIQUE\|RELIC>` (`ParseRaw` reads `^Rarity: (%a+)`; `Item Class:` is accepted *before* it when the clipboard text is pasted) | `Rarity:` from `frameType` 0–3; 9 and 10 (foil) as UNIQUE; gems, currency, cards, quest and necropolis frames refused with an error | `frameTypeId`, `rarity` |
| 2 | the title, then the base name (rare/unique), or `<prefix>base<suffix>` on one line | `name`, then `typeLine` — always two lines, so a magic item's `typeLine` (which carries its affix names) is read as the base name | `name`, `typeLine`, `baseType` |
| 3 | `Armour: n`, `Evasion: n`, `Energy Shield: n`, `Ward: n`, each with an optional `<Type>BasePercentile: n` | — (never written) | `properties` types 16–18 (armour, evasion, energy shield; trade-query F5); the percentile only on a `fetch` (trade-query F7) |
| 4 | `Intangibility: n%` | — | `properties` (PoE2 / 3.29 lines) |
| 5 | `Unique ID: <id>` | `Unique ID: <id>` | `id` |
| 6 | `League: <name>`, `Unreleased: true` | — | `league` |
| 7 | `<Influence> Item` per influence (Shaper, Elder, Warlord, Hunter, Crusader, Redeemer, Searing Exarch, Eater of Worlds, Synthesised, Fractured…) | — (influences dropped) | `influences.*`, `synthesised`, `fractured`, `searing`, `tangled` |
| 8 | `Crafted: true`, then `Prefix: {range:r}{fractured}<modId>` / `Suffix: …` — Path of Building's own affix identity, by mod id | — | no equivalent: the private API names no mod ids (trade-query F7) |
| 9 | `Catalyst: <name>`, `CatalystQuality: n` | — | `properties` |
| 10 | `Cluster Jewel Skill: …`, `Cluster Jewel Node Count: n`, `Talisman Tier: n` | — | `enchantMods` / `properties` |
| 11 | `Item Level: n`, `Memory Strands: n` | `Item Level: n` | `ilvl` |
| 12 | `Variant:` / `Version:` / `Selected …` blocks for uniques with variants | — | — |
| 13 | `Quality: n` | `Quality: n` with the `+` and `%` stripped | `properties` type 6, `+20%` |
| 14 | `Sockets: R-G-B W` (colour letters, `-` links, space breaks) | `Sockets:` from `sockets[]`: `attr` S/D/I/G → R/G/B/W, `-` while `group` repeats | `sockets[].{group, attr, sColour}` |
| 15 | `LevelReq: n`, `Radius: …`, `Limited to: n`, `Requires Class …` | `LevelReq: n` from `requirements` "Level" | `requirements` |
| 16 | `Implicits: n` = enchants + implicits + scourge lines, then those lines: `{enchant}` / `{scourge}` / plain, each with `{crafted}`, `{fractured}`, `{tags:…}`, `{range:…}` and other tags prepended | `Implicits: n` = enchants + implicits, then enchants as `{crafted}<line>`, then implicits plain | `enchantMods`, `implicitMods`, `scourgeMods` (`implicitMods` is an object line since 2026-07-31, item-facts F3) |
| 17 | explicit lines: `{fractured}`, `{crafted}`, `{mutated}`, `{prefix}`, `{suffix}`, `{exarch}`, `{eater}`, `{synthesis}`, `{unscalable}`, `{vestigial}`, `{modGroup:…}` prefixes as the line's flags say; then crucible lines | `fracturedMods` as `{fractured}`, `explicitMods` plain, `craftedMods` as `{crafted}`, `mutatedMods` as `{mutated}` — **the C++ reads arrays the private API no longer sends** (`craftedMods`, `fracturedMods` are gone; the flag is on the line object, trade-query F7) | `explicitMods[].{description, flags}` |
| 18 | `Split`, `Mirrored`, `Fractured Item`, `Corrupted`, `Foil Unique (<type>)` | `Corrupted` only | `split`, `duplicated`, `fractured`, `corrupted`, `foilVariant` |

Three facts a search design cares about:

- **The clipboard text is the union of both.** A listed item's `extended.text` (trade-query F9) is the game's own clipboard text, CRLF-terminated, headed `Item Class: <name>` then `Rarity:`; Path of Building's parser accepts it directly. The C++ export is a subset written from the JSON and omits the `Item Class:` line, the influences and the split/mirrored/fractured flags.
- **Affix identity is Path of Building's, not GGG's.** The `Prefix:`/`Suffix:` lines name Path of Building's mod ids (the export's `mods.json` keys); nothing on the private API carries them, and the trade `fetch` carries mod *names* and tiers instead. A stash search that wants affixes must derive them — the `hash-check.csv` join is the material.
- **The C++ app's export has rotted with the API.** It reads `craftedMods` and `fracturedMods` arrays the API stopped sending in July 2026 and sends every explicit line untagged, so a crafted line pastes into Path of Building as a natural one. Not a rule to reproduce; the flags on the line object are the source now.
