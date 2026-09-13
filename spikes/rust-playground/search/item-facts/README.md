# item-facts — the item as GGG gives it

Status: first pass complete — 2026-09-13. Census run over both stores; Q1–Q3 closed the same day (a map is a tier-only item, the value code is a display style, transfigured gems join by suffix); Q4 waits for the design session.

Headline:
- The corpus is 36,139 distinct items from 1,800 tabs and 60 characters across six leagues, 44 MB of JSON at 1,220 bytes an item (median 1,143). Two stores, the same account, keyed by GGG id; the newer fetch wins (F1).
- 85 top-level fields observed against the reference's 105; none undocumented at the top level, four undocumented influence flags, 20 documented fields never seen. 13 fields are universal; `rarity` is on 70 % and `frameTypeId` on all (F2).
- `explicitMods` and `implicitMods` became `{description, flags}` objects between 2026-07-23 and 2026-07-31 — every other array stays strings, no line ever carries a hash, and the reference read 2026-09-13 documents the `ItemMod` type and no longer lists `craftedMods` or `fracturedMods`. With the objects came `[Tag|Display]` markup inside display text: 338 PoE1 lines, pervasive on PoE2 (F3).
- `baseType` joins the trade taxonomy for 78 % of items; every map fails (7,512: `baseType` is `Map (Tier N)`) and so does every transfigured gem (272; the export joins 243 by suffix). PoE1 carries no class field; PoE2 carries one as property type 109 (F5). A map is a tier, a rarity and its mods: no map carries a `Map Tier` property or an area, the current series' icon is a tier number, and the trade site lists no area base type (F7).
- The same id's JSON is frozen across a month: 99.5 % byte-identical over 20,228 items; what moves is position, a price note, a stack size, and the veiled placeholder — never mod text, never the icon URL (F6).

## Question

What does an item look like in the facts the store already holds, field by field and class by class, and how big is the corpus a search must serve? The projection from verbatim JSON to a searchable attribute vector starts from this census.

## Inputs

| Source | Access | Status |
| --- | --- | --- |
| The spike's facts store (facts v7): Standard and Allflame tabs and characters, PoE2 characters, seen 2026-09-02..11 | `sqlite3 .backup`, owner's machine; `raw/`, local only | read |
| The C++ app's store (master `946a4f51`, bodies verbatim): Standard 2026-08-13, Mirage 2026-07-20; Allflame, Hardcore, Solo Self-Found refreshed by the owner 2026-09-13 with maps and uniques | same | read |
| The C++ app's 30 backup stores, 2026-06..09 | read in place | read, for the format dating only |
| The API reference's `Item`, `ItemProperty`, `ItemSocket`, `ItemMod` types | sanctioned; read online 2026-09-13 | read (`data/api-reference-item-fields-2026-09-13.txt`) |
| The trade site's base-type taxonomy | `../trade-query/data/items-2026-09-12.json` | read |

Every hash and byte count is in `MANIFEST.md`.

## Findings

**F1 — The corpus** (`data/numbers.md`, "Per store"). Ids are shared across the stores and survive a league's end: 40 Mirage items reappear under Standard with their ids; 21,309 Standard ids sit in both stores. Deduped by id:

| League | Items | Tabs fetched | Characters fetched | Captured |
| --- | ---: | ---: | ---: | --- |
| Standard (pc) | 26,882 | 1,080 (C++, maps and uniques included; the spike's 405 are among them) | 41 | 2026-08-13 (C++), 2026-09-08..11 (spike) |
| Solo Self-Found | 6,345 | 442 | 9 | 2026-09-13 |
| Allflame | 1,337 | 98 | 4 | 2026-09-07..13 |
| Hardcore | 1,221 | 142 | 1 | 2026-09-13 |
| Mirage | 256 | 38 | 0 | 2026-07-20 |
| Standard (poe2) | 98 | — | 5 | 2026-09-02 |

The owner's Standard stash lists 3,347 tabs (2,566 map and 509 unique substashes); no capture has fetched them all, so the full account is larger than this corpus.

**F2 — Fields** (`data/field-census.csv`: 236 paths, each with its value types, item share, `frameTypeId` breakdown and an example). Universal: `baseType, frameType, frameTypeId, h, icon, id, identified, ilvl, league, name, typeLine, verified, w`. `frameType` is deprecated in the reference for `frameTypeId` (`Rare` 8,435, `Normal` 8,086, `Gem` 7,040, `Unique` 5,904, `Magic` 3,511, `Currency` 2,270, `DivinationCard` 884, `SupporterFoil` 7, `Quest` 2). Shares: `x`/`y`/`inventoryId` 94.5 % (absent on socketed gems), `explicitMods` 74.5 %, `properties` 74.4 %, `rarity` 70.2 %, `requirements` 55 %, `implicitMods` 21.4 %, `sockets` 16.4 %, `note` 0.5 %. Against the reference: nothing undocumented at the top level; `influences.{hunter, warlord, crusader, redeemer}` undocumented (the reference lists elder, shaper, searing, tangled); 20 documented fields unobserved — PoE2-only (`unidentifiedTier`, `doubleCorrupted`, `sanctified`, `desecrated`, `tamedBeastProperties`), user text (`forum_note`, `flavourTextNote`), race rewards, `notableProperties`, `logbookMods`, `cosmeticMods`, `prophecyText`, `foreseeing`, `ruthless`, `extended`, and `socketedItems` (lifted). `crucible.nodes` is a dict keyed by node index.

**F3 — Mod lines** (`data/mod-templates.csv`: one row per array and display template, `#` for numbers, with items, lines, line kind, flags, example).

| Array | Items | Lines | Kind | Templates | Note |
| --- | ---: | ---: | --- | ---: | --- |
| `explicitMods` | 26,930 | 109,921 | object (224 strings: the 2026-07-20 capture) | 5,703 | flags `crafted` 1,041, `fractured` 289, `mutated` 81; 1,933 templates cover 90 % of lines, 1,218 are singletons, 540 hold `\n` |
| `implicitMods` | 7,733 | 8,712 | object | 460 | no flags seen |
| `ultimatumMods` | 554 | 3,634 | object `{type, tier}` | 36 | |
| `utilityMods` | 2,613 | 3,225 | string | 26 | flasks |
| `enchantMods` | 1,686 | 2,897 | string | 604 | |
| `crucibleMods`, `veiledMods`, `scourgeMods`, `bondedMods`, `runeMods` | 62, 55, 12, 1, 1 | | string | | `veiledMods` is `Prefix#`/`Suffix#`, a placeholder re-rolled per response (F6) |

Dating (`scripts/line-format-dates.sh`, the C++ backups): strings on every capture through 2026-07-23 (19,991 items on 07-22), objects on every capture from 2026-07-31 (17,734). No `hash` on any of 118,633 object lines. The C++ app's view (its own template, one table per item; cpp-search F3): 905 items lose 914 numbered lines to a repeated template, `+# to maximum Life` first (163); 3,978 items have a line in an array the app never reads, 862 have mod lines only there. `[Tag|Display]` markup (`Rare Monsters have [ElementalThorns|Elemental Thorns] reflecting…`, `[Intangibility|Intangibility]`): 320 PoE1 explicit lines and 18 property names, none in the string-era backups; on PoE2, 85 property names, 60 explicit, 32 requirement and 15 implicit lines of 98 items.

**F4 — Properties** (`data/properties-census.csv`: 1,175 rows of array × name × `type` × `displayMode` × value shape). `properties` has 579 names and 66 type ids; 44,284 of its 100,234 item-rows carry no `type` at all (gem lines: `Cast Time`, `Cost`, `Consumes {0} of {1} Charges on use`), so the trade renderer's 70-type bridge (trade-query F5) covers the typed rows only. Value shapes by item-rows: `#%` 30,627; `#` 24,659; empty 9,467; `# sec` 4,255; two values into a `{0}…{1}` name (`displayMode` 3) 4,091; `#/#` 3,582; `# Mana` 3,554; `#-#` damage ranges 2,427; `# (Max)` 1,784; `#% of base` 1,462. `values[][1]` is a code: 0 (58,558 spike rows), 1 (19,521; augmented), 4–7 on damage ranges (fire, cold, lightning, chaos as the names read), 10, 18–22, 24 on heist and sanctum lines. It is the site's `PoE/Item/DisplayProperty/ValueStyle` (the saved bundle, read 2026-09-13): 0 Default, 1 Augmented, 2 Unmet, 3–7 Physical, Fire, Cold, Lightning, Chaos damage, 8–10 Magic, Rare, Unique item, 11 NotableReminder, 15 Unreachable, 18 CurrencyItem, 19 ItemQuantity, 20 DivinationCard, 21–23 Sanctum Boon, Curse, Pact, 24 Italic, 25 Underlined — a colour class, of which only Augmented says anything about the value: modified from its base, the site's `_aug`. `requirements` has 13 names over 15 type ids; `hybrid.properties` (vaal gems) carries no `type`.

**F5 — Class evidence** (`data/class-evidence.csv`: `frameTypeId` × icon art directory × trade category by `baseType`). 28,313 items (78.3 %) join a trade category through `baseType`; one base (`Energy Blade`) joins two. Unmatched: 7,512 maps — every map's `name`, `typeLine` and `baseType` read `Map (Tier N)` (or `Ceremonial Map`, `Map of Miring`) and the trade `map` group's 282 numeric-id entries are all blighted discriminators (the icon: F7); 272 transfigured gems (`baseType` is the transfigured name; stripping ` of …` joins 243 to an export base, repoe F3); 98 PoE2 items; a scarab and two contracts. The icon's top directory is a coarse class (17 values) but `Currency/` holds 2,948 heist contracts and 439 map-group items. PoE2 items carry the class as `properties[].type == 109` (`Body Armour`, `[Quiver]`); PoE1 items carry nothing.

**F6 — Stability** (`scripts/drift.py`; 20,228 Standard stash ids in both stores, four to eight weeks apart). 20,132 byte-identical (99.5 %). The 96 that differ: `note` added on 54, `x`/`y` on 40/30, `veiledMods` re-rolled on 11 of the 12 veiled items, one currency stack size. No mod text, icon URL (signed query string included), socket, requirement or identity field ever differed; no item went from unidentified to identified. Churn: 5,571 ids gone and 3 new over the interval.

**F7 — Maps** (`scripts/map-identity.py` → `data/map-identity.csv`; 7,273 items with base `Map (Tier N)`, newest fetch wins). The icon URL's base64 segment is `{"f": "…/Atlas2Maps/New/<Art>", "mn", "mt", "mg"?, "mi"?, "mc"?}`. `mn` is the map series, 8–24, dated by league (every Allflame map is 24, Solo Self-Found mostly 21, Standard hoards 12–13); `mt` is 0 on all 7,273; `mg` 1–4, `mi` 2 and `mc` 1–4 sit on 134, 135 and 57 tier-14–16 maps and are unread. The art is 178 legacy area names plus `MapNumbers<tier>`: 717 of the 766 series-24 maps carry the tier number and no area, one area (`Cemetery`) spans tiers 4 to 16 across series, and no map carries a `Map Tier` property or any `Tier`-named one — the tier lives in the base type alone. `descrText` reads "Travel to a Map of this tier or lower by using this in a personal Map Device." The trade site's `map` group lists no area base type either (5 map types, 282 blighted ids, 96 discriminated variants, 305 other bases; the art names join none). So a map is a tier, a rarity and its mods; the area in an older map's icon is decoration.

## Numbers

| | |
| --- | --- |
| Items (distinct ids) / tabs fetched (distinct) / characters fetched | 36,139 / 1,800 / 60 |
| Item JSON bytes: total / mean / p50 / p95 / max | 44,083,581 / 1,220 / 1,143 / 1,950 / 4,598 |
| Store files: spike / C++ | 54,378,496 / 57,196,544 |
| Socketed gems | 1,978 |
| Distinct JSON paths / top-level fields observed / documented | 236 / 85 / 105 |
| Explicit templates / covering 90 % of lines / singletons | 5,703 / 1,933 / 1,218 |
| Explicit lines per item: mean / max | 4.08 / 16 |
| Property rows / `properties` names / type ids | 1,175 / 579 / 66 |
| `baseType` matched to a trade category | 28,313 (78.3 %) |
| Same-id items byte-identical across a month | 20,132 of 20,228 (99.5 %) |
| Maps (base `Map (Tier N)`) / distinct icon art / with a tier-number icon | 7,273 / 179 / 717 |
| C++ mod table: items losing a numbered line to a repeat / items with mod lines only in unread arrays | 905 / 862 |

## Open questions

- **Q4 — Does any consumer need the 20 unobserved fields?** Closes: the cpp-search catalogue's field list joined to `field-census.csv` in the design session.

Candidate claims for the ground truth (for the design session to route): the object line format and its dating; `influences` carrying four undocumented flags; a map being a tier-only item with no `Map Tier` property; the value code as a display style; the `[Tag|Display]` leak; the veiled placeholder; ids surviving a league's end.

## Provenance

`MANIFEST.md` (every file, hash, source). Generated: `scripts/census.py` → `data/field-census.csv`, `data/mod-templates.csv`, `data/properties-census.csv`, `data/class-evidence.csv`, `data/numbers.md`; `scripts/drift.py` → F6 (printed); `scripts/map-identity.py` → `data/map-identity.csv` (F7; the crosstabs printed); `scripts/line-format-dates.sh` → the F3 dating (printed; reads the backups in place). Hand-extracted: `data/api-reference-item-fields-2026-09-13.txt`. A fact not in a listed file is marked open.

## Review

| Date | Reviewer | Finding | Disposition |
| --- | --- | --- | --- |
