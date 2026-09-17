# The search digest

Status: partial — item-facts, cpp-search, trade-query, repoe — 2026-09-16

The slice's evidence in the shape a design can cite: one claim per
line, `S<n> · kind · weight · claim · pointer`, under the brief
(`brainstorming-notes/21-search-digest-brief.md`, d8502bfb). Ids are
stable and never reused; a pointer is a track README finding
(`item-facts F7`) or a data file in the track's `data/`. Reach a README only to verify a claim.

## item-facts

S1 · measure · — · 36,139 items from 1,800 tabs (Standard lists 3,347), 60 characters, 6 leagues, 2 realms; 44,083,581 JSON bytes, max 4,598 · F1, numbers.md
S2 · item · main · 13 fields on every item: `baseType, frameType, frameTypeId, h, icon, id, identified, ilvl, league, name, typeLine, verified, w`; `explicitMods` on 74.5 %, `properties` 74.4 %, `rarity` 70.2 %, `note` 0.5 % · F2, field-census.csv
S3 · ggg · main · `explicitMods`/`implicitMods` are `{description, flags}` objects from 2026-07-31, strings through 2026-07-23; no `hash` on 118,633 object lines · F3
S4 · ggg · main · `[Tag|Display]` markup in display text: pc 320 explicit lines, 18 property names, none pre-object; poe2 (98 items) 220 lines over 8 arrays · F3, numbers.md
S5 · measure · — · `explicitMods`: 26,930 items, 109,921 lines, 5,703 templates, 1,933 covering 90 %, 1,218 singletons; 4.08 lines per item, max 16 · F3, numbers.md
S6 · item · main · `properties`: 579 names over 66 type ids, 44,284 of 100,234 rows with no `type` (gem lines) · F4
S7 · ggg · main · `values[][1]` is the site's `ValueStyle` colour class (0 Default, 1 Augmented, 2 Unmet, 3–7 damage); only Augmented says anything about the value · F4
S8 · item · main · `baseType` joins a trade category for 28,313 items (78.3 %); unmatched: 7,512 maps, 272 transfigured gems (243 join by suffix), 98 poe2, 3 others · F5
S9 · item · main · PoE1 items carry no class field; poe2 items carry it as `properties[].type == 109` · F5
S10 · item · main · A map's `name`, `typeLine` and `baseType` all read `Map (Tier N)` (or `Ceremonial Map`, `Map of Miring`); none of 7,273 carries a `Map Tier` or any `Tier`-named property · F5, F7
S11 · measure · — · 20,132 of 20,228 same-id Standard items byte-identical 4–8 weeks apart; the 96 differ in `note`, `x`/`y`, `veiledMods`, one stack, never in mod text, icon URL or identity; 5,571 ids gone, 3 new · F6
S12 · limit · edge · `veiledMods` is a `Prefix#`/`Suffix#` placeholder re-rolled per response (11 of 12 veiled items; 55 carry it) · F3, F6
S13 · ggg · main · 85 of the reference's 105 top-level fields observed, none undocumented; `influences.{crusader,hunter,redeemer,warlord}` undocumented; 20 documented unobserved, 5 PoE2-only · F2, numbers.md
S14 · limit · main · A map's area exists only as icon art (178 names; 717 of 766 series-24 icons show none); no property, base type or trade base names it · F7
S15 · item · main · Ids survive a league's end: 40 Mirage items reappear under Standard with their ids · F1

## cpp-search

S21 · tool · — · 38 filters, all ANDed, bounds inclusive with either side blank; Crit., APS, the four defences, Level and Map Tier skip an item lacking the property, DPS, counts, requirements and Quality read 0; Priced (the buyout manager) is the one predicate outside the item · F1
S22 · tool · — · pDPS/cDPS = `APS × avg(range)`, quality not applied; eDPS ignores the damage type tag; links = the longest run of consecutive equal `group`; the `rarity` string is never read; only Quality, Level and Stack Size are normalised · F2
S23 · measure · — · 36,139 items: Crit. (3,269 items, `8.00%`) and Block (490, `23%`) parse as 0 on every item; Map Tier reads a property no map carries (0 of 7,273) · F2, accessor-census.csv
S24 · tool · — · Text filters are case-insensitive substring over the accessor; Name searches `name + " " + typeLine`; Category is a substring of the lowercased RePoE class name, uncategorised matches nothing · F1
S25 · tool · — · A searchable mod is a template: each `[0-9.]` run becomes `#`, the value the mean of the line's numbers; a numberless line never enters the table; nine other mod arrays and socketed items are never read · F3
S26 · ggg · main · The site's `extended.ar/ev/es` equal the property with the item's quality taken out and 20 % put in, within one, on all 37 fetched defence values; `_aug` absent when the item already has 20 % · F2
S27 · measure · — · 36,139 items: 905 lose 914 numbered lines to a duplicate template (last wins); 3,978 have a line in an unread array, 862 nothing else · F3, item-facts numbers.md
S28 · owner · — · "Ignored mods and duplicates are both design bugs in the c++." (2026-09-13) · F3
S29 · tool · — · A pseudomod sums its lines, a template listed N times counting N×: 35 pseudomods over 117 templates, 34 of 35 names matching trade's `pseudo` texts · F3, pseudomods.toml
S30 · tool · — · Taxonomy and mod dropdown are RePoE, fetched at runtime from `repoe-fork.github.io`, nothing bundled; with no network every category is empty and the dropdown holds the 35 pseudomods alone · F4
S31 · tool · — · By Tab buckets on `(location type, id)`, never label or position; By Item is one flat bucket; every column but Price and Date orders by the rendered string parsed, then (PrettyName, id, hash_v4) · F5
S32 · tool · — · Rarity is `frameType` alone; gems, currency, cards and `-1` match no rarity query · F1, F2
S37 · measure · — · 975,711 items, M4, 1 thread: a pre-M3 reset 5,562 ms, 93 % the sort; materialized keys cut a sort to 130 ms (266 MB/column); `FilterItems` then 329 ms (20.2 ms at 101,048), an unindexed scan deferred by M3 D7 · F6, delta-pipeline.md
S39 · idiom · — · A mod row is the `#`-templated line with inclusive min/max, either side blank — `+# to maximum Life`; rows AND · F1, F3

## trade-query

S41 · ggg · main · `stats` takes eight group types — `and`, `not`, `if`, `count`, `weight`, `weight2`, `crucible`, `mercenary`; `if` tip, verbatim: "Match items that meet each stat's `min` and `max` requirements if the stat is present." · F2
S42 · ggg · main · 93 filters in 12 groups: 59 read a field a private item carries or derives, 7 the market, 27 unsettled — the largest `category`, 83 option ids no private field gives · F3, gap.csv
S43 · ggg · main · 18,187 stat entries, 14,193 keys, 13,975 texts, 14 categories; key = the numeric stat id, category = the mod's source; 11,694 keys in one category, 2,499 in several · F4
S44 · ggg · main · Text does not identify a key: 380 (category, text) groups are shared, 369 by two ids, 8 by three, 3 by four; 93 entries have a `(Local)` twin · F4, stat-collisions.csv
S45 · ggg · edge · Of the one collision tested, `stat_3680664274` hit the capped total 10,000, its twin `stat_492027537` 0; the picker offers both; 379 untested · F8 Q2
S46 · owner · — · "I confirmed that the second modifier returns no results. I tested this across every poe1 league possible." (2026-09-13) · F8 Q2
S47 · ggg · main · A two-number line matches on its numbers' average: min 20 admitted lows of 15–27, all averages ≥ 20; max 25 admitted `Adds 17 to 30`, all averages ≤ 25 · F8 Q3
S48 · item · main · In 70 fetched items, 443 lines: 11 lines came from two mods each (life from a hybrid prefix plus a pure one) and 31 mods fed two lines each · F8 Q6
S49 · ggg · main · At `pseudo_total_fire_resistance` min 80 every total equalled the sum of the item's displayed fire, all-elemental and fire-and-X lines, explicit and crafted; a 46–48 mod shown `+60%` counted 60 · F8 Q5
S50 · item · main · Private-API items carry `explicitMods` and `implicitMods` as `{description, flags?}` objects, never with a hash; enchant, utility, veiled, crucible arrays stay strings — 18,383 items, 71,695 lines, 2026-09-02..11 · F7
S51 · ggg · main · A fetched line is `{description, domain, hash, mods: [{name?, tier?, level?, magnitudes: [{min, max}]}], flags?}`; crafted (12) and fractured (4) lines sit in `explicitMods`; no `extended.mods` · F7, fetch-census.json census
S52 · limit · main · Only the trade `fetch`'s `mods` list attributes a line to the mods that made it; 34 of 443 fetched lines lack it and `pseudoMods` name no contributors; a private item has none · F7, F8 Q6
S53 · limit · main · No item-class field on a fetched or private item; the class sits only in `extended.text`, base64 clipboard text whose first line is `Item Class: <class>` in all 70 fetched items — listed items only · F9
S54 · idiom · — · The owner's own search, verbatim: `{"id": "explicit.stat_3299347043", "value": {"min": 60}, …}` in an `and` group with `{"category": {"option": "accessory.ring"}, "rarity": {"option": "rare"}}` · fetch-census.json `queries.q1.request`
S55 · ggg · main · The site's renderer bridges 70 `properties[].type` ids to filter fields (1–4 map tier/iiq/iir/pack size, 5 gem level, 6 quality, 9–13 damage/crit/aps, 15–18 block/ar/ev/es, 62–65 level/str/dex/int, 78 ilvl) · F5
S56 · ggg · main · The search `id` in the response and the site's URL is the `query` object gzipped and base64url-encoded: q4's id decodes to exactly its request · F9

## repoe

S61 · tool · — · The export's `trade_stats` are its text join to the site's texts at export time: 7,170 of 11,257 translation entries carry a trade id, 4,087 none; 11,004 distinct ids named, 61.7 % of the capture's 18,187 entries, 72.2 % of `stat_N` keys · F1, Numbers
S62 · tool · — · Coverage per category: explicit 74.7 %, implicit 73.5 %, fractured 96.6 %, crafted 99.3 %, enchant 72.6 %, scourge 100 %; pseudo 10.1 %, sanctum 0.8 %, mercenary 0.2 %, imbued, veiled, delve, ultimatum, crucible 0 — those ids are names, not stat hashes · F1, trade-stat-coverage.csv
S63 · tool · — · The `Stats.dat` hash the trade-number recipe needs (MurmurHash2, seed `0x02312233`, over a mod's stats' hashes) is absent from the export (`stats.py` never reads the column) and no function of the id string: 19 string hashes × 3 variants reproduce none of 6,678 single-stat numbers · F2, hash-recipe.md
S64 · measure · — · MurmurHash2 on a 4-byte key is a bijection, so a single-stat trade number is its `Stats.dat` hash encoded: 6,958 stat ids recovered; a `minimum`/`maximum` pair hashes as eight bytes · F2, hash-recipe.md, stat-hashes.csv
S65 · measure · — · All 29,288 Path of Building (mod, number) rows classed under the owner's rule: 23,400 reproduced by the text join, 983 by the hash where the join gave several, 4,479 numbers not in the capture, 352 groups with no translation, 74 renderings the join missed, 0 unexplained; 423 of 5,489 distinct numbers (7.7 %) are not in the capture · F2
S66 · measure · — · Two deterministic detectors: a stat id with two single-stat numbers has at most one right (123 ids; the hash picks one for 90, neither for 32); a text with two ids is two hashes (175 of the site's 380 collisions attributed) · F2
S67 · limit · edge · The 32 undecided ids: the owner searched all 68 candidate numbers (2026-09-13, Standard, status any), 53 found, 15 not; 12 ids have one live number, 20 have both, so the site cannot decide; a `no` is dated dormancy in one league, never nonexistence ("it might possibly still exist") · F7, twice-numbered-verdicts.csv
S68 · owner · — · "let's keep min and max separate." (2026-09-13, two-value lines); the site averages, the C++ app takes the mean, Path of Building keeps them apart · F7
S69 · ggg · main · 65 of the site's 68 leaf category ids generate from the export's classes, 58 one class each, 7 by a name rule on the base; the 3 `monster.*beast` ids are not bases; 103 classes, 31 with no trade id · F3, class-to-trade-category.csv
S70 · measure · — · 35,651 PC items: 35,011 join an export base by `baseType` (98.2 %), 243 transfigured gems by stripping ` of …`, 397 do not (`Blighted Map (Tier N)`, beasts); `Map (Tier N)` is an export base · F3, Numbers
S71 · measure · — · A private line joined to a translation entry, both sides canonicalised (`[Tag|Display]` → display, sign before a placeholder dropped, `#`): equipment frames 95.3 % of explicit lines (75.4 % of 3,402 templates), 96.8 % of implicit; gem 15.5 %, other 16.8 %; utility 100 %, enchant 56 % · F4, template-vs-translation.csv
S72 · measure · — · 12,131 matched equipment explicit lines match several entries; the 150 ambiguous templates are local/global twins (`# to maximum Energy Shield`, attack speed, evasion, armour, accuracy, `Adds # to # Physical Damage`), split by the export's `is_local` per id or the mod table by item class · F4
S73 · item · main · A line fed by several mods is the game summing one stat id across mods and rendering it once: 4,369 prefix/suffix mods over 956 stat ids, 650 ids in two or more mods, 153 in ten or more (`base_maximum_life`: 55 mods, 9 groups); a private line's mods are recoverable only as the mod groups whose stat sets fit its value · F5
S74 · tool · — · The mod extract holds 15,920 item-domain mods with stats, ranges, generation type and spawn weights by tag; 24,435 (unique, crucible, non-item domains) left out; 237 multi-id translations carry trade ids, 178 of them `minimum`/`maximum` pairs · Outputs, F5, mod-stat-index.csv
S75 · tool · — · 86 game versions seen since 2025-06-07, 53 exported; lag from the fork's version poll to its export 0 days for 46, 1 for 3, 4–13 for 4; 33 never exported; the poll's date is not the patch's · F6
S76 · tool · — · Path of Building's affix identity is its own mod id (`Prefix: {range:r}{fractured}<modId>`), which nothing on the private API carries; its explicit lines take `{fractured}`, `{crafted}`, `{mutated}`, `{prefix}`, `{suffix}` and other tags; its parser accepts the game's clipboard text (`Item Class:` then `Rarity:`) directly · pob-format.md rows 1, 8, 17, notes

## The acceptance set

Written when owner-seat and agent-seat merge.

## Limits register

Every `limit` claim, with what the search says when it meets it (an
unknown shown, never guessed, is the model).

- S12 — a veiled line is shown as the placeholder it is; a value query never matches it.
- S14 — a map's area is not a fact the search holds; a query for one gets an unknown, never the icon's art.
- S52 — a line is matched as displayed text and value; which mods made it is not known and never guessed.
- S53 — an item's class is not a field; a class the search names is a derivation it owns, and an item it cannot class is shown unclassed.
- S67 — a line whose trade number the hash and the site both leave undecided is shown with both candidates, never one guessed.

## Kill list

- repoe: F1 229 ids listed twice, older text joined for 263 rows — budget; F2 the owner's acceptance-rule quote — budget; F3 site agreement 3,363/1/8/586, clipboard `Item Class:` = export class for 46 bases — budget; F4 unmatched equipment remainder (heist `Alert Level`, `Has # Abyssal Socket`, cluster-jewel passives, necropolis) — budget; pob-format.md the C++ export's `craftedMods`/`fracturedMods` reading — internals; Q8 access method — open question, not a fact.
- trade-query: F1 request body, `query` decomposition, realm by omission, default sort, 100/500/10,000 caps — budget; F1 `status` options — moot; F2 `count`/`weight`/`weight2`/`crucible`/`mercenary` tips — budget; F3 per-group counts, map/heist/sanctum/ultimatum open items — sizing; F3 "what a stash search adds" — budget; F4 per-category counts, id-kind prefixes, C++-pseudomod comparison — sizing; F5 renderer array order, `frameType` values — internals; F6 realms, leagues, `items.json`/`static.json` counts — sizing; F7 `extended.hashes`, the computed `dps`/`pdps`/`ar`/`ev`/`es` (`pdps` = average physical × APS × 1.2; ≈ S26) — budget; F8 Q3 the owner's expectation before q3b, Q4 `weight`/`count` shapes, `complexity` 85, `inexact` — budget.
- cpp-search: F1 refresh split, F6 mechanisms (not its numbers) — internals; F4 `m_replace_map` — moot.
- item-facts: F7 `mn`/`mg`/`mi`/`mc` and the icon payload past the tier — unread; F2 `crucible.nodes`, `frameTypeId` composition — internals; Numbers' per-store and file-size tables — sizing.

## The question index

Note 22's ranked questions, each with the claims that bear on it;
extended after every merge.

1. A line's identity, and a line it cannot name — S3, S4, S5, S11, S12, S25, S27, S28, S29, S43, S44, S45, S47, S48, S52, S63, S64, S65, S66, S67, S71, S72, S73, S76
2. The query model and its one grammar — S2, S6, S7, S9, S10, S21, S22, S24, S32, S39, S41, S47, S54, S68
3. The vocabulary read, served to a human and an agent — S2, S5, S6, S8, S13, S29, S30, S42, S43, S44, S55, S61, S62, S69, S70, S74
4. Who holds the corpus, the derivation and its contract — S1, S11, S13, S15, S31, S37, S50, S75
5. What crosses the trade boundary — S7, S8, S26, S29, S41, S42, S45, S46, S49, S51, S54, S55, S56, S61, S62, S66, S67, S69
6. What a result carries — S1, S11, S31, S53
7. The non-goals and limits, as outputs — S12, S14, S52, S53, S67

## Convergence and contradiction

- S23 ≈ S10: no map carries a `Map Tier` property (cpp-search measured it through the app's accessor, item-facts through the census).
- S27 ≈ item-facts F3: the same 905/914/3,978/862 counts, sized in item-facts' numbers.md for cpp-search's template.
- S50 ≈ S3: the object mod lines with no hash, seen in the spike store (18,383 items) and the census over both stores (36,139).
- S53 ≈ S9: no class field on a PoE1 item, from the trade captures and from the census.
- S49 ≈ S29: the site's `pseudo_total_*` contributor set matches the C++ pseudomod table (F8 Q5: "matches the C++ table").
- S66 ≈ S44: the site's 380 text collisions, counted by trade-query and 175 of them attributed to two hashes by repoe.
- S67 ≈ S45, S46: the live/dormant twin pattern, one collision tested by trade-query, 32 ids by repoe (12 one live, 20 both).
- S73 ≈ S48: a line fed by several mods, seen in 70 fetched items and explained by the mod table's stat-id summation.
- S70 ≈ S8, contrast: `baseType` joins an export base for 98.2 % of PC items but a trade category for 78.3 %, because `Map (Tier N)` is an export base and no trade base.
- S69 ≈ S42: the `category` filter's 83 option ids, 65 of the 68 leaves generated from the export's classes.
- S68 ≈ S47, S25: the two-value line averaged by the site, meaned by the C++ app, kept apart by Path of Building and the owner.
- trade-query F4 says "380 pairs"; stat-collisions.csv has 380 groups of two to four ids (369/8/3). S44 carries the data.
- item-facts F5 itemises the unmatched bases to 7,885 (7,512 + 272 + 98 + 3) while `data/numbers.md`'s `frameTypeId` table sums them to 7,826 (36,139 − 28,313); a 59-item gap the track does not explain. S8 carries F5's figures.
- item-facts F3 counts PoE2 markup over four arrays (192 lines); `data/numbers.md` lists eight (220). S4 carries the data file.
