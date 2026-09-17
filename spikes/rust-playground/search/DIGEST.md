# The search digest

Status: accepted — 2026-09-17

The slice's evidence in the shape a design can cite: one claim per
line, `S<n> · kind · weight · claim · pointer`, under the digest brief
(note 21 at c56404ca). Ids are stable and never reused; a pointer is a
track README finding (`item-facts F7`) or a data file in the track's
`data/`; a trailing `≈ S<m>` names a claim another track carries for
the same fact. This is the entry point for item search: reach a README
only to verify a claim.

**How to read it:** the question index first, then the claims it names
for the question in hand, then the limits register. The track sections
are the source order; the kill list is for a finding that seems missing
and the contradictions for a number that seems wrong.

## item-facts

S1 · measure · — · 36,139 items from 1,800 tabs (Standard lists 3,347), 60 characters, 6 leagues, 2 realms; 44,083,581 JSON bytes, max 4,598 · F1, numbers.md ≈ S141
S2 · item · main · 13 fields on every item: `baseType, frameType, frameTypeId, h, icon, id, identified, ilvl, league, name, typeLine, verified, w`; `explicitMods` on 74.5 %, `properties` 74.4 %, `rarity` 70.2 %, `note` 0.5 % · F2, field-census.csv ≈ S92, S129, S130, S184
S3 · ggg · main · `explicitMods`/`implicitMods` are `{description, flags}` objects from 2026-07-31, strings through 2026-07-23; no `hash` on 118,633 object lines · F3 ≈ S50
S4 · ggg · main · `[Tag|Display]` markup in display text: pc 320 explicit lines, 18 property names, none pre-object; poe2 (98 items) 220 lines over 8 arrays · F3, numbers.md
S5 · measure · — · `explicitMods`: 26,930 items, 109,921 lines, 5,703 templates, 1,933 covering 90 %, 1,218 singletons; 4.08 lines per item, max 16 · F3, numbers.md
S6 · item · main · `properties`: 579 names over 66 type ids, 44,284 of 100,234 rows with no `type` (gem lines) · F4
S7 · ggg · main · `values[][1]` is the site's `ValueStyle` colour class (0 Default, 1 Augmented, 2 Unmet, 3–7 damage); only Augmented says anything about the value · F4
S8 · item · main · `baseType` joins a trade category for 28,313 items (78.3 %); unmatched: 7,512 maps, 272 transfigured gems (243 join by suffix), 98 poe2, 3 others · F5
S9 · item · main · PoE1 items carry no class field; poe2 items carry it as `properties[].type == 109` · F5 ≈ S53, S93
S10 · item · main · A map's `name`, `typeLine` and `baseType` all read `Map (Tier N)` (or `Ceremonial Map`, `Map of Miring`); none of 7,273 carries a `Map Tier` or any `Tier`-named property · F5, F7 ≈ S23, S94
S11 · measure · — · 20,132 of 20,228 same-id Standard items byte-identical 4–8 weeks apart; the 96 differ in `note`, `x`/`y`, `veiledMods`, one stack, never in mod text, icon URL or identity; 5,571 ids gone, 3 new · F6 ≈ S132
S12 · limit · edge · `veiledMods` is a `Prefix#`/`Suffix#` placeholder re-rolled per response (11 of 12 veiled items; 55 carry it) · F3, F6
S13 · ggg · main · 85 of the reference's 105 top-level fields observed, none undocumented; `influences.{crusader,hunter,redeemer,warlord}` undocumented; 20 documented unobserved, 5 PoE2-only · F2, numbers.md ≈ S177
S14 · limit · main · A map's area exists only as icon art (178 names; 717 of 766 series-24 icons show none); no property, base type or trade base names it · F7
S15 · item · main · Ids survive a league's end: 40 Mirage items reappear under Standard with their ids · F1 ≈ S177

## cpp-search

S21 · tool · — · 38 filters, all ANDed, bounds inclusive with either side blank; Crit., APS, the four defences, Level and Map Tier skip an item lacking the property, DPS, counts, requirements and Quality read 0; Priced (the buyout manager) is the one predicate outside the item · F1 ≈ S158, S173
S22 · tool · — · pDPS/cDPS = `APS × avg(range)`, quality not applied; eDPS ignores the damage type tag; links = the longest run of consecutive equal `group`; the `rarity` string is never read; only Quality, Level and Stack Size are normalised · F2
S23 · measure · — · 36,139 items: Crit. (3,269 items, `8.00%`) and Block (490, `23%`) parse as 0 on every item; Map Tier reads a property no map carries (0 of 7,273) · F2, accessor-census.csv ≈ S10
S24 · tool · — · Text filters are case-insensitive substring over the accessor; Name searches `name + " " + typeLine`; Category is a substring of the lowercased RePoE class name, uncategorised matches nothing · F1 ≈ S121, S182
S25 · tool · — · A searchable mod is a template: each `[0-9.]` run becomes `#`, the value the mean of the line's numbers; a numberless line never enters the table; nine other mod arrays and socketed items are never read · F3 ≈ S68, S101
S26 · ggg · main · The site's `extended.ar/ev/es` equal the property with the item's quality taken out and 20 % put in, within one, on all 37 fetched defence values; `_aug` absent when the item already has 20 % · F2 ≈ S90
S27 · measure · — · 36,139 items: 905 lose 914 numbered lines to a duplicate template (last wins); 3,978 have a line in an unread array, 862 nothing else · F3, item-facts numbers.md ≈ item-facts F3
S28 · owner · — · "Ignored mods and duplicates are both design bugs in the c++." (2026-09-13) · F3
S29 · tool · — · A pseudomod sums its lines, a template listed N times counting N×: 35 pseudomods over 117 templates, 34 of 35 names matching trade's `pseudo` texts · F3, pseudomods.toml ≈ S49
S30 · tool · — · Taxonomy and mod dropdown are RePoE, fetched at runtime from `repoe-fork.github.io`, nothing bundled; with no network every category is empty and the dropdown holds the 35 pseudomods alone · F4 ≈ S174, S190
S31 · tool · — · By Tab buckets on `(location type, id)`, never label or position; By Item is one flat bucket; every column but Price and Date orders by the rendered string parsed, then (PrettyName, id, hash_v4) · F5 ≈ S122, S176
S32 · tool · — · Rarity is `frameType` alone; gems, currency, cards and `-1` match no rarity query · F1, F2
S37 · measure · — · 975,711 items, M4, 1 thread: a pre-M3 reset 5,562 ms, 93 % the sort; materialized keys cut a sort to 130 ms (266 MB/column); `FilterItems` then 329 ms (20.2 ms at 101,048), an unindexed scan deferred by M3 D7 · F6, delta-pipeline.md
S39 · idiom · — · A mod row is the `#`-templated line with inclusive min/max, either side blank — `+# to maximum Life`; rows AND · F1, F3

## trade-query

S41 · ggg · main · `stats` takes eight group types — `and`, `not`, `if`, `count`, `weight`, `weight2`, `crucible`, `mercenary`; `if` tip, verbatim: "Match items that meet each stat's `min` and `max` requirements if the stat is present." · F2 ≈ S106
S42 · ggg · main · 93 filters in 12 groups: 59 read a field a private item carries or derives, 7 the market, 27 unsettled — the largest `category`, 83 option ids no private field gives · F3, gap.csv ≈ S69
S43 · ggg · main · 18,187 stat entries, 14,193 keys, 13,975 texts, 14 categories; key = the numeric stat id, category = the mod's source; 11,694 keys in one category, 2,499 in several · F4
S44 · ggg · main · Text does not identify a key: 380 (category, text) groups are shared, 369 by two ids, 8 by three, 3 by four; 93 entries have a `(Local)` twin · F4, stat-collisions.csv ≈ S66, S105, S109, S171
S45 · ggg · edge · Of the one collision tested, `stat_3680664274` hit the capped total 10,000, its twin `stat_492027537` 0; the picker offers both; 379 untested · F8 Q2 ≈ S67
S46 · owner · — · "I confirmed that the second modifier returns no results. I tested this across every poe1 league possible." (2026-09-13) · F8 Q2 ≈ S67
S47 · ggg · main · A two-number line matches on its numbers' average: min 20 admitted lows of 15–27, all averages ≥ 20; max 25 admitted `Adds 17 to 30`, all averages ≤ 25 · F8 Q3 ≈ S68
S48 · item · main · In 70 fetched items, 443 lines: 11 lines came from two mods each (life from a hybrid prefix plus a pure one) and 31 mods fed two lines each · F8 Q6 ≈ S73
S49 · ggg · main · At `pseudo_total_fire_resistance` min 80 every total equalled the sum of the item's displayed fire, all-elemental and fire-and-X lines, explicit and crafted; a 46–48 mod shown `+60%` counted 60 · F8 Q5 ≈ S29
S50 · item · main · Private-API items carry `explicitMods` and `implicitMods` as `{description, flags?}` objects, never with a hash; enchant, utility, veiled, crucible arrays stay strings — 18,383 items, 71,695 lines, 2026-09-02..11 · F7 ≈ S3
S51 · ggg · main · A fetched line is `{description, domain, hash, mods: [{name?, tier?, level?, magnitudes: [{min, max}]}], flags?}`; crafted (12) and fractured (4) lines sit in `explicitMods`; no `extended.mods` · F7, fetch-census.json census
S52 · limit · main · Only the trade `fetch`'s `mods` list attributes a line to the mods that made it; 34 of 443 fetched lines lack it and `pseudoMods` name no contributors; a private item has none · F7, F8 Q6 ≈ S171
S53 · limit · main · No item-class field on a fetched or private item; the class sits only in `extended.text`, base64 clipboard text whose first line is `Item Class: <class>` in all 70 fetched items — listed items only · F9 ≈ S9, S93
S54 · idiom · — · The owner's own search, verbatim: `{"id": "explicit.stat_3299347043", "value": {"min": 60}, …}` in an `and` group with `{"category": {"option": "accessory.ring"}, "rarity": {"option": "rare"}}` · fetch-census.json `queries.q1.request`
S55 · ggg · main · The site's renderer bridges 70 `properties[].type` ids to filter fields (1–4 map tier/iiq/iir/pack size, 5 gem level, 6 quality, 9–13 damage/crit/aps, 15–18 block/ar/ev/es, 62–65 level/str/dex/int, 78 ilvl) · F5
S56 · ggg · main · The search `id` in the response and the site's URL is the `query` object gzipped and base64url-encoded: q4's id decodes to exactly its request · F9 ≈ S175

## repoe

S61 · tool · — · The export's `trade_stats` are its text join to the site's texts at export time: 7,170 of 11,257 translation entries carry a trade id, 4,087 none; 11,004 distinct ids named, 61.7 % of the capture's 18,187 entries, 72.2 % of `stat_N` keys · F1, Numbers ≈ S108, S155, S175
S62 · tool · — · Coverage per category: explicit 74.7 %, implicit 73.5 %, fractured 96.6 %, crafted 99.3 %, enchant 72.6 %, scourge 100 %; pseudo 10.1 %, sanctum 0.8 %, mercenary 0.2 %, imbued, veiled, delve, ultimatum, crucible 0 — those ids are names, not stat hashes · F1, trade-stat-coverage.csv
S63 · tool · — · The `Stats.dat` hash the trade-number recipe needs (MurmurHash2, seed `0x02312233`, over a mod's stats' hashes) is absent from the export (`stats.py` never reads the column) and no function of the id string: 19 string hashes × 3 variants reproduce none of 6,678 single-stat numbers · F2, hash-recipe.md
S64 · measure · — · MurmurHash2 on a 4-byte key is a bijection, so a single-stat trade number is its `Stats.dat` hash encoded: 6,958 stat ids recovered; a `minimum`/`maximum` pair hashes as eight bytes · F2, hash-recipe.md, stat-hashes.csv
S65 · measure · — · All 29,288 Path of Building (mod, number) rows classed under the owner's rule: 23,400 reproduced by the text join, 983 by the hash where the join gave several, 4,479 numbers not in the capture, 352 groups with no translation, 74 renderings the join missed, 0 unexplained; 423 of 5,489 distinct numbers (7.7 %) are not in the capture · F2
S66 · measure · — · Two deterministic detectors: a stat id with two single-stat numbers has at most one right (123 ids; the hash picks one for 90, neither for 32); a text with two ids is two hashes (175 of the site's 380 collisions attributed) · F2 ≈ S44, S109
S67 · limit · edge · The 32 undecided ids: the owner searched all 68 candidate numbers (2026-09-13, Standard, status any), 53 found, 15 not; 12 ids have one live number, 20 have both, so the site cannot decide; a `no` is dated dormancy in one league, never nonexistence ("it might possibly still exist") · F7, twice-numbered-verdicts.csv ≈ S45, S46
S68 · owner · — · "let's keep min and max separate." (2026-09-13, two-value lines); the site averages, the C++ app takes the mean, Path of Building keeps them apart · F7 ≈ S47, S25
S69 · ggg · main · 65 of the site's 68 leaf category ids generate from the export's classes, 58 one class each, 7 by a name rule on the base; the 3 `monster.*beast` ids are not bases; 103 classes, 31 with no trade id · F3, class-to-trade-category.csv ≈ S42, S93
S70 · measure · — · 35,651 PC items: 35,011 join an export base by `baseType` (98.2 %), 243 transfigured gems by stripping ` of …`, 397 do not (`Blighted Map (Tier N)`, beasts); `Map (Tier N)` is an export base · F3, Numbers
S71 · measure · — · A private line joined to a translation entry, both sides canonicalised (`[Tag|Display]` → display, sign before a placeholder dropped, `#`): equipment frames 95.3 % of explicit lines (75.4 % of 3,402 templates), 96.8 % of implicit; gem 15.5 %, other 16.8 %; utility 100 %, enchant 56 % · F4, template-vs-translation.csv ≈ S101, S155
S72 · measure · — · 12,131 matched equipment explicit lines match several entries; the 150 ambiguous templates are local/global twins (`# to maximum Energy Shield`, attack speed, evasion, armour, accuracy, `Adds # to # Physical Damage`), split by the export's `is_local` per id or the mod table by item class · F4 ≈ S105
S73 · item · main · A line fed by several mods is the game summing one stat id across mods and rendering it once: 4,369 prefix/suffix mods over 956 stat ids, 650 ids in two or more mods, 153 in ten or more (`base_maximum_life`: 55 mods, 9 groups); a private line's mods are recoverable only as the mod groups whose stat sets fit its value · F5 ≈ S48
S74 · tool · — · The mod extract holds 15,920 item-domain mods with stats, ranges, generation type and spawn weights by tag; 24,435 (unique, crucible, non-item domains) left out; 237 multi-id translations carry trade ids, 178 of them `minimum`/`maximum` pairs · Outputs, F5, mod-stat-index.csv
S75 · tool · — · 86 game versions seen since 2025-06-07, 53 exported; lag from the fork's version poll to its export 0 days for 46, 1 for 3, 4–13 for 4; 33 never exported; the poll's date is not the patch's · F6 ≈ S110
S76 · tool · — · Path of Building's affix identity is its own mod id (`Prefix: {range:r}{fractured}<modId>`), which nothing on the private API carries; its explicit lines take `{fractured}`, `{crafted}`, `{mutated}`, `{prefix}`, `{suffix}` and other tags; its parser accepts the game's clipboard text (`Item Class:` then `Rarity:`) directly · pob-format.md rows 1, 8, 17, notes

## item-filter

S81 · ggg · main · Conditions in a `Show`/`Hide`/`Minimal` block are ANDed — "If there are multiple conditions in a block then all of them must be matched for the block to match an item."; the page names blocks, `Continue`, `Import` and conditions, and no OR or grouping · "The language, as the page states it"
S82 · measure · — · The 2026-09-13 page save names 61 conditions and 14 actions (no action reads an item); ribbons: 6 `PoE2-only`, 3 `New`, 1 `Updated`, 1 `Deleted` (`GemQualityType`); operands: 27 boolean, 22 numeric, 4 name, 3 enum, 5 compound · Numbers, conditions.csv, actions.csv
S83 · ggg · main · One operator table of eight (`=` `!` `!=` `<` `<=` `>` `>=` `==`) serves every condition, no per-condition restriction stated; `conditions.csv`'s `operators` column is this track's derivation, not a page statement · "The language, as the page states it"
S84 · ggg · main · What a bare operator-less value matches — prefix, substring or exact — is unstated for `BaseType`, `Class`, `HasEnchantment`, `HasExplicitMod`, as is whether `HasExplicitMod`'s count ranges over the named mods or all mods; no real filter was read, so nothing here is checked against usage · F1, Q1, "Not done"
S85 · measure · — · Of the 61 conditions, 32 read a stash-export field directly, 7 are derivable, 5 need base-item data (RePoE), 17 have no source found · F6, conditions.csv `item_field`
S86 · measure · — · Of the 61, 22 are named by both other surfaces, 14 by trade alone, 1 by the C++ app alone, 24 by neither (7 joins land on a trade stat, not a filter id); in reverse it names 19 of the C++ app's 38 filters (14 exact, 5 divergent) and 27 of trade's 86 item-reading filters (21 exact, 6 divergent) · Numbers, conditions.csv, coverage.csv
S87 · ggg · main · It names no computed number and no requirement: no DPS of any kind, no APS, crit or block, no total defence, no `R. Level`/`R. Str`/`R. Dex`/`R. Int` and no trade `damage`/`lvl`/`str`/`dex`/`int`; nothing about a listing; and no `Crafted`, `Veiled` or `Split`, which both other surfaces name · F5, coverage.csv `kind=none` ≈ S162
S88 · owner · — · "item filters are design by GGG to be unable to require specific numbers, because GGG wants people to experience the randomness of checking loot" (2026-09-13) · F5 ≈ S162
S89 · ggg · main · One thing carries a spelling per surface: the Foulborn flag is `Foulborn` here, C++ `Mutated`, trade id `mutated` with the text "Foulborn", export `mutated`; synthesised is `SynthesisedItem`/`Synthesized`/`synthesised_item`/`synthesised`; five more rows at the pointer · F2
S90 · ggg · main · Two conditions diverge in referent, not spelling: `AreaLevel` is the area the item dropped in, trade's `area_level` a map's own level; `BaseArmour`/`BaseEvasion`/`BaseEnergyShield`/`BaseWard` read base values before mods where C++ `Armour`/`Evasion`/`Shield` and trade `ar`/`ev`/`es`/`ward` read the total, and nothing here reads a total · F3 ≈ S26
S91 · ggg · edge · `HasSearingExarchImplicit` and `HasEaterOfWorldsImplicit` compare an implicit tier 1–6 while trade's `searing_item`/`tangled_item` and export `searing`/`tangled` are yes/no; `Scourged` is yes/no while trade's `scourge_tier` and export `scourged.tier` carry a tier · F3
S92 · ggg · main · It names `Width` and `Height`; the export carries `w` and `h` on every item in the census (share 1.000) and neither other surface offers a filter for either · F4, item-facts field-census.csv ≈ S2
S93 · ggg · main · No class vocabulary is on the page: `Class` takes an "Item class name" with the one example `Class Currency`, and `class-names.csv` holds one `class_value` row (Currency) plus five tokens occurring only inside other phrases; the class join rests on repoe F3 · F7, Q3, class-names.csv ≈ S9, S53, S69
S94 · ggg · main · One page serves PoE1 and PoE2: six conditions are ribboned `PoE2-only` (listed at the pointer), among them `WaystoneTier`, which is PoE2's `MapTier` renamed; whether those ribbons enumerate the whole PoE1/PoE2 divergence is open · F8, Q4 ≈ S10
S95 · idiom · — · GGG's own example on the page, verbatim: `HasExplicitMod >=2 "of Haast" "of Tzteosh" "of Ephij"` — a page example, not a filter observed in use · F1, conditions.csv row `HasExplicitMod`

## prior-art

S101 · tool · — · Identity is the displayed line's own English text: fnv1a-32 of the matcher string, binary-searched over (hash, byte offset) pairs into one ndjson line of a 2.5 MB file held as a single string; the key is itself a sentence · F1 ≈ S25, S71, S171
S102 · tool · — · `ref` is English in every language file (`ru/stats.ndjson` line 1 has the English `ref` with Russian matchers); only `matchers[].string` is translated · F1
S103 · tool · — · Numbers are removed by trying every placement: up to 11 candidates of "which numbers stay literal, which become `#`", most-placeholders-first with raw text last, each looked up until one hits whose `trade.ids` holds the modifier type; reversed wordings carry `negate: true` (1653 matchers) and flip the roll's sign; the advanced-mod-description wording is a second key, `advanced` (1420 matchers), indexed instead of `string` when present · F2
S104 · tool · — · Ambiguity is a two-stat `StatGroup` with a named resolver: 100 of 9178 lines, each exactly two stats, resolver one of `trivial-merge` 43, `select` 41 (by the item's category), `percent-merge` 11, `flag-merge` 5 · F3
S105 · tool · — · The local/global twin is one line with two trade ids: `#% increased Armour` appears twice with identical `ref` and identical matchers, told apart only by `resolve.test: ["ARMOUR", null]` · F3, stat-model.md "A local/global twin — line 489" ≈ S72, S44
S106 · tool · — · No entry carries two ids in one modifier category (0 of 9278); 2294 carry ids in several. A group's second id is merged into the list at match time, and a filter with several ids becomes a trade `count` group with `value.min = 1`, an OR · F3, Numbers ≈ S41
S107 · limit · main · Unknown lines are kept, never guessed: the leftover goes to `item.unknownModifiers` with its modifier type and renders as an orange "Not recognized modifier"; no fuzzy match on mod text exists anywhere (the only Levenshtein is over OCR'd gem names) · F4 ≈ S186
S108 · measure · — · Its 12507 distinct trade ids are all known to the 2026-09-12 trade capture (0 unknown) and cover 12507 of that capture's 17958; the 5451 lacked include crucible 2492, scourge 409, delve 81, ultimatum 63 at zero, 187 of 240 sanctum, 944 enchants, 1026 explicits; all 88 option-bearing capture ids are carried · F5, stat-model.md "Coverage of the trade capture" ≈ S61
S109 · measure · — · Of the 380 trade-id collisions the trade-query track found: 57 resolved by a group (30 trivial-merge, 24 select, 3 percent-merge), 83 kept as separate entries, 143 with one id carried, 97 with neither, 0 as several ids in one entry · F5, Numbers, coverage.csv `collision_verdict` ≈ S44, S66
S110 · measure · — · Over 55 data commits to `stats.ndjson` (2022-03-24…2026-09-09, median gap 12 days, max 205) refs grew 6139 → 9198, 31 of 54 diffs removed at least one ref, `d876a14` added 1402 refs, `9a012b5` ("update stats for 3.26.0.15") removed 930 refs and 1434 trade ids; 2436 times a trade id survived while the text it displays changed, 526 of them in `9a012b5` · F6, cadence.csv ≈ S75
S111 · limit · main · A rename fails loudly for the 210 refs hard-coded as `stat()` string literals in 10 source files, each asserted at startup with a `Cannot find stat: X` throw, and silently — one orange line — for the other ~9000 · F6
S112 · tool · — · The dataset generator is not in the repository: no script writes `stats.ndjson`, the only build step over it regenerates the four `*.index.bin` files, data arrives as opaque "update data" commits, and what generates it from what is unknown · F6, Q1
S113 · tool · — · Its stash search does no matching: a saved string goes on the clipboard and Ctrl+F, Ctrl/Cmd+V, Enter are synthesised into the game window; the corpus is the game client's own box, the tool never parses the string, and the only GGG endpoints it calls are `/api/trade/search` and `/api/trade/fetch` — no stash endpoint in `renderer/src`, `main/src` or `ipc/` · F7, stash-search.md
S114 · tool · — · The game highlights the matching cells and the tool learns nothing back; there is no ranking, and the editor turns the input red past 250 characters, the game's field length · stash-search.md "The two features" (Result, Ranking, Limits rows)
S115 · idiom · — · Ten stash-search strings ship as defaults, four for map rolling and six for dump sorting, e.g. `"Pack Size: +3"` and `"Map Device" "Rarity: Normal"` · stash-search.md "Defaults shipped"

## store-as-built

S121 · store · — · Server-side item predicates are four — case-insensitive substring over `name`/`type_line`/`base_type`, realm, league, live-or-removed (`Store::search(text, realm, league, include_removed, limit)`) — `Store::item` takes an id, and no read takes a rarity, frame type, container, `ilvl`, mod or location · F1 ≈ S24, S182
S122 · store · — · `items_location (location_kind, location_id)`, the index `Store::tabs`, `Store::characters` and `read_items` all seek on and the coordinate C29/C54 make authoritative, has no public read keyed on it; `TabRow.item_count` gives a tab's live count, not its items · F2, query-plans.txt ≈ S31, S176, S197
S123 · store · — · The only routes to "this tab's items": `pricing_snapshot`, which returns the whole league, needs an `Annotations` handle and refuses unless the file's account uuid pairs; or a `Store::search` substring that happens to match; raw SQL is not a surface (C48) · F2
S124 · store · — · `Store::search` builds `%{text}%`, planned `SCAN items USING INDEX items_location` + `USE TEMP B-TREE FOR LAST 2 TERMS OF ORDER BY`; `items_names (name, type_line, base_type)` is written on every ingest and every `rebuild` and serves no read in the crate · F3, query-plans.txt (plans under SQLite 3.53.4; shipped `acq` links 3.46.0, Q6)
S125 · store · — · Anchored `LIKE 'Kaom%'` also plans `SCAN items` (LIKE is case-insensitive by default, `items_names` collates BINARY); under `case_sensitive_like=ON` it becomes `SEARCH items USING INDEX items_names (name>? AND name<?)`, and `name = ?` becomes `SEARCH items USING INDEX items_names (name=?)` · query-plans.txt, the three counterfactual rows
S126 · store · — · A whole league's live items plans as `SCAN items`: no index spans `(realm, league, removed_at)` · query-plans.txt, "a whole league's live items"
S127 · store · — · `ItemRow` carries `json: Value`, so each row `Store::search` returns costs one full `serde_json` body parse on top of the scan and the temp sort · F3, read-surface.md `Store::search` ≈ S150, S183
S128 · measure · — · `items`' 21 columns: 9 derived from the row's own json, 6 ingest facts the body does not carry (realm, league, location_kind, location_id, container, socketed_in), 4 bookkeeping, the id, the body; 53 columns across `items`, `tabs`, `characters`, `item_events` · Numbers, F4, columns-vs-json.csv
S129 · measure · — · 224 census paths have no column (73 top-level), and 7 top-level paths present on every one of the 36,139 census items have none: `frameType`, `frameTypeId`, `icon`, `identified`, `ilvl`, `league`, `verified` · Numbers, columns-vs-json.csv ≈ S2
S130 · store · — · `rarity` is a json-derived column returned in `ItemRow`, present on 25,370 of the 36,139 census items, and no read filters on it · columns-vs-json.csv row `column,items,rarity` ≈ S2
S131 · store · — · Two reads mean different things by "in this league": `Store::search` filters `items.league` — for a character item, the character row's listing-owned league as it stood at ingest — while `read_items` ignores `items.league` for character items and joins the character's current league, deliberately carrying league-less characters (C61); a league-filtered search silently drops a league-less character's items and can answer from a stamp the last listing has already moved · F5
S132 · store · — · Per item the store gives `first_seen`, `last_seen`, `removed_at` and `seen_response` (that last only through `ItemSnapshot`); membership is per response, never per clock (C54) · F6 ≈ S11
S133 · store · — · The only aggregates on the surface are the per-tab and per-character `item_count` and the 12 counts in `Status`; `events_since(since, limit)` is keyed on time alone, with no filter by kind, location or item, name/type from a LEFT JOIN to the live row; there is no grouping · F6
S134 · store · — · With no store change every predicate in the census is reachable by pulling the corpus and filtering in the frontend: `Store::search("")` matches every row whose extracted columns are non-NULL (the current extractor writes `""`, not NULL) and returns each whole body, at one full scan, one temp sort and one `serde_json` parse per item, with realm, league, `location_kind`, `location_id`, `container`, `socketed_in` on `ItemRow`; no predicate can be pushed down · F8, "What a search consumer can do today"
S135 · store · — · Three classes of change and their schema cost: a new read (`items_at(location)`, a container/location filter on `search`, an events filter) needs none, `items_location` already serving it (C48); a derived column (`frame_type`, `identified`, `ilvl`, `corrupted`, `note`) is one `rebuild` re-extracts from each row's own json (C29); a derived table (a mod/stat index, FTS at ingest) is a new table reproducible from `items.json` (C34; `decisions/store.md` "Parked: search-at-scale", trigger *a real consumer with a measured latency or duplication case*) · F8 table ≈ S151, S188
S136 · idiom · — · The whole item search a user has today is `acq items search` and MCP `search_items`, both on `Store::search` · read-surface.md `Store::search` row, "Called by today"

## engine-bench

S141 · measure · — · The corpus is 36,139 items (both stores deduped by id, 0 skipped as malformed), 53,569,958 JSONL bytes, 125,006 mod lines, 6,513 templates, 2,939 stat ids; the ×10 (361,390) and ×30 (1,084,170) corpora are clones with fresh ids, so selectivity per query is identical at every scale and the scaled rows measure size, not variety · Numbers, numbers.csv ≈ S1
S142 · measure · — · Every number is Apple M4 / macOS 26.6.2 / 32 GB, rustc 1.94.1 release, bundled SQLite 3.45.3 (`journal_mode=OFF`, `synchronous=OFF` for the build), warm = median of 7, cold = first run on a fresh connection with SQLite's page cache empty and the OS file cache still warm · Numbers preamble
S143 · measure · — · The scan is one pass over a compact struct per item (template, stat id, array, tab and league interned to `u32` at load; no index), producing the matching id set, not a count · F1
S144 · measure · — · All twelve queries take 2.6 ms together at 36,139; at 1,084,170 the warm median query is 4.47 ms and the twelve together 88.0 ms; cold is within 4 % of warm · F1, results.csv
S145 · measure · — · Per-million scan cost is flat across the three scales (`q03` 35.5 → 37.6 ms/M, `q08` 6.45 → 6.30); extrapolated to 100 ms, the substring query — the scan's worst, 40.8 ms at 1,084,170 — arrives at ~2.7 M items, the mod-value queries at ~23 M, the plain attribute queries at ~55–69 M · F1, results.csv
S146 · measure · — · SQLite over the derived schema (18 columns, six indexes, an indexed `lines` table, `ANALYZE` run) is slower on eleven of twelve, by a median 8.6× at 36,139, 16.8× at 361,390, 20.5× at 1,084,170, worst 85× (`q12`) · F2
S147 · measure · — · Five of the twelve pass 100 ms on SQLite at 1,084,170 (`q02` 101.7, `q11` 104.5, `q08` 129.6, `q12` 135.7, `q09` 202.7 ms) and none do on the scan; the trade-style group queries `q08` and `q09` are SQLite's worst, against 6.8 and 8.2 ms scanned · F2, results.csv
S148 · measure · — · SQLite wins one shape, a high-selectivity indexed equality: `q06-one-tab` at 1,084,170 is 0.66 ms delivering ids, 0.32 ms as `count(*)`, against the scan's 1.57 ms — an index turning 1 M comparisons into 25 k; at the real 36,139 it wins nothing, `q06` there being 1.3× slower · F2, results.csv ≈ S187
S149 · measure · — · The same SQL wrapped in `count(*)` runs within 4 % of the row-delivering form at every scale, so the gap is the engine's own work — btree descents, row decoding, the join back to `items` — not rusqlite's row plumbing · F2, results.csv
S150 · measure · — · Parsing GGG JSON into the struct costs 5.4 µs per item: 202 ms at 36,139, 5,884 ms at 1,084,170 · F3, numbers.csv ≈ S127
S151 · measure · — · Rebuilding the same structs from the derived tables costs 38 ms at 36,139 and 1,142 ms at 1,084,170, 5.2× cheaper than from GGG JSON; the 18 columns plus line rows answer all twelve without the item body; resident is 392 MB from the projection against 289 MB from the parse at 1,084,170 · F5, numbers.csv ≈ S135, S187
S152 · measure · — · A process that does not outlive one query cannot use the scan: at 1,084,170 it pays 5,884 ms of load to answer a 4.5 ms question, where SQLite's 77 ms median wins by two orders of magnitude · F3
S153 · store · — · `acqd`, the spike's long-lived process, writes facts and never reads them (C2, C34), so the corpus-holding process these numbers argue for is frontend-side and does not exist today; at 36,139 a per-command load is 202 ms from JSON, 38 ms from a projection · F3, Review row (Fable, 2026-09-13)
S154 · measure · — · Template and stat-id identities answer the twelve within noise on the scan (±5 % on ten of twelve; two mod-heavy queries swing ±25 % run to run) and 4–11 % apart on SQLite, tracking the smaller table, not the identity · F4, results.csv
S155 · measure · — · 40,793 of 125,006 mod lines (32.6 %) carry no trade stat id — 23,796 gem skill text, 9,201 map, ultimatum and flavour-shaped, 7,796 equipment; keyed by stat id the `lines` table is 33 % shorter (84,213 rows) and the file 36 % smaller (15,863,808 against 24,911,872 bytes) at 36,139 · F4, identity-coverage.csv, numbers.csv ≈ S71, S61
S156 · measure · — · The trade stat id merges variants of one stat (implicit/explicit/fractured/crafted) that the template keeps apart, and 259 of 3,478 mapped templates are ambiguous — several entries, the first trade id winning · F4
S157 · measure · — · The derived schema costs 764,694,528 file bytes and a 5,687 ms build at 1,084,170 against 288,532,800 bytes resident for the scan; at 36,139, 24,911,872 bytes and 162 ms against 10,982,951 · Numbers, numbers.csv
S158 · idiom · — · The owner's query shape as `q10` records it: "(Armour > 1000) OR (Required Level < 80)" — 35,956 of 36,139 items match, because a requirement the item lacks reads as 0 · queries.md, `q10-boolean-or` row and the absence-rule note ≈ S21, S169

## owner-seat

S161 · measure · — · The recorded set is seven questions in three kinds: find one remembered item (Q2, Q4, Q6), which items carry mod X (Q1, Q3, Q7), a set by level (Q5); none of the seven filters on armour, evasion or damage numbers · `owner-seat F1`, `data/questions.md` table (corpus: the owner's seven questions, 2026-09-13)
S162 · owner · — · "My own examples also missed the mark on this. There are many times where a specific modifier value is needed. Examples include attributes, resistances, crit, and spell suppression, but there are many others. It will vary heavily depending on what the user is trying to put together." · `data/questions.md`, "On specific values, verbatim (2026-09-13, after the item-filter read)" ≈ S87, S88, S194
S163 · owner · — · "Be careful not to over-specify what a mod question means. PoE's itemization is complex, and the build system is complex, so for any given possible definition, someone probably wants to use it. We need a system that is flexible, simple, and generalizable here, not one tuned to a specific set of questions from a specific single user." · `data/questions.md`, "Two corrections to the reading"; `owner-seat F5`
S164 · owner · — · "keeping track of what is possible to create in the game is a complex task beyond the scope of acquisition. We should let users provide that knowledge for now instead of trying to embed or access it from within the app." · `data/questions.md`, "Two corrections to the reading"; `owner-seat F5`
S165 · owner · — · "Legacy means an item or modifier or modifier value or unique or unique variant that cannot be found or created by playing the game, so it only exists in Standard stashes" — examples given: the Ashes of the Stars onyx amulet's 3.23.0 variant with "10-20% increased Reservation Efficiency of Skills", and "a bugged body armour with +25,244% fire resistance"; "I don't have many legacy items, but there are many legacy flasks, uniques, rares, and other items in standard." · `data/questions.md`, "Refinements, verbatim (after the first pass, 2026-09-13)"
S166 · owner · — · "When I'm asking for specific mods, I likely have a base in mind--either something specific like Titan Gauntlet or Spiked Gloves, or I do care about the base and it's attributes." · `data/questions.md`, "Refinements, verbatim (after the first pass, 2026-09-13)"
S167 · owner · — · "Usually well-crafted endgame gear is already on bases with the right armour or evasion, so I don't worry about those numbers as much." … "I don't play melee builds, but people who do care very much about the modifiers on their weapon, and many builds care about other damage mods--sometimes they are hit-based, sometimes they are spell-based, sometimes they are damage over time, sometimes they are other ailments." · `data/questions.md`, "Refinements, verbatim" (2026-09-13)
S168 · owner · — · "The trade site lets me select yes/no/any for binary flags. It's stat field's autocomplete distinguishes implicit, explicit, pseudo, fractured, and other modifier types. The stat field autocomplete is also almost instantly responsive to keystrokes and lists autocomplete options that makes sense in an order that makes sence. The c++ mod search box is terrible by comparison on both fronts." · `data/questions.md`, "The trade site against the stash, verbatim (prompt 4, 2026-09-13)" ≈ S174
S169 · idiom · — · The composition neither search allows, in the owner's own writing: "The c++ does not allow for complex boolean searches such as (A or (B AND C)). The trade site allows that kind of logic, but only for stat modifiers, not for any of the other search fields, so I can't ask for something like "(Armour > 1000) OR (Required Level < 80)"." · `data/questions.md`, "The trade site against the stash, verbatim (prompt 4, 2026-09-13)" ≈ S158
S170 · owner · — · Pricing: "I don't use acquisition for managing sales" … "There are a small number of users who depend upon this however, but I don't know their workflows." … "I do not want acquisition updating forum shops directly. Those users can have their agents drive their web browsers, or maybe we re-enable this feature later, but I don't want to support direct updates out of the gate because of the use of POESESSID." Organising: "I generally don't use acquisition to manage my stash at all, because it cannot perform actions like move items or rename tabs. However, knowing counts and total of different currencies, equipable items, and other non-equipable items such as maps and fragments might be useful, or at least interesting." · `data/questions.md`, prompt-2 and prompt-3 answers; `owner-seat F2`
S171 · requirement · — · R1: "A line has an identity finer than its text: its kind (implicit, explicit, fractured, crafted, pseudo, …) and, where a mod is named, the mod"; prevents "Q1, Q3, Q7 missing a mod that renders several ways, or a template that hides its kind" · `owner-seat R1` ≈ S44, S101, S52, S194
S172 · requirement · — · R2: "Boolean composition across every field, not stats alone — a mod with a base, by name or by the base's attributes, being one instance"; prevents "the gap both existing searches share (F3); a model tuned to one user's questions (F5)" · `owner-seat R2` ≈ S194
S173 · requirement · — · R3: "Tri-state flags: yes, no, any"; prevents "the C++ checkbox that can only demand" · `owner-seat R3` ≈ S21
S174 · requirement · — · R4: "The mod vocabulary served instantly, categorised by kind, ranked sensibly — a read-model property"; prevents "the C++ dropdown (F3); the same need as an agent's schema discovery" · `owner-seat R4` ≈ S30, S168, S190
S175 · requirement · — · R5: "A trade search, as a URL or its JSON, is a stash query; the join is the trade id → stat id map (repoe F1, F2)"; prevents "re-typing a query the site already holds (F4)" · `owner-seat R5` ≈ S56, S61
S176 · requirement · — · R6: "Every answer carries the item's location — tab or character, realm, league"; prevents "Q2, Q4, Q6 answered without "where"" · `owner-seat R6` ≈ S31, S122, S197
S177 · requirement · — · R7: "League of origin is not promised: no field carries it; a derivation from league-specific mods or first-seen is the most a search can offer"; prevents "Q4 answered wrongly" · `owner-seat R7` ≈ S13, S15
S178 · requirement · — · R8: "Legacy is the user's knowledge: the search finds an item by the mod, value or variant the user names, and the app never judges what the game can produce"; prevents "Q2, Q6; and the scope creep of tracking the game (F5)" · `owner-seat R8`
S179 · owner · — · "Being able to use a trade search query against my stash would be fantastic, especially if there was an integrated way to make this happen--e.g. with a simple browser addon, or even a basic copy/paste. There's already a browser extension called Better Trading that people use to manage trade searches. Integrating with that might be fun, but not a core features." … "integrating with Awakened PoE trade to price items from within acquisition somehow (gui? cli? mcp?) would be useful. Some newer players have asked for this." · `data/questions.md`, "Anything else, verbatim (prompt 5, 2026-09-13)"; `owner-seat F4`
S180 · requirement · — · Non-goals: "acting on the stash; direct forum updates; a pricing engine"; prevents: F2, F4 · `owner-seat` requirements table, last row

## agent-seat

S181 · store · — · From the tool list and `acq --help` alone the store surfaces yield the corpus size, tabs with counts, characters with counts, the realm vocabulary and the currency table — no field, rarity, class or mod of an item; the first row pulled (call 5, 3.5 KB, a gem, `rarity: null`) is the only schema. · agent-seat F1
S182 · store · — · `search_items` / `items search` has one predicate — a case-insensitive substring over name, type line and base type — plus `league`, `realm` and `include_removed` filters and `limit`; there is no rarity, class, level, mod, tab, sort, offset or count, and the order is ingest order (characters first), deterministic and undocumented. · agent-seat F2 ≈ S121, S24
S183 · measure · — · A row is the store's columns plus the whole GGG JSON: 50 default rows are 91.5 KB compact (MCP) / 149 KB pretty (CLI); the corpus of 22,721 items is 38,820,498 B / 5,644 ms (MCP) / 63,534,917 B / 2,335 ms (CLI); `search_items {text:"Staff"}` returned 50 of 81 with no total and no flag. · agent-seat F3, Numbers table ("dump" row, Q4) ≈ S127
S184 · store · — · The only count surface is `tabs` (1,006,740 B pretty on the CLI, 745,672 B on the MCP, which has no `limit` parameter; Standard only, the league named); `store_status` reports `leagues: 0`; per-league, per-rarity and per-class counts came from the dump and jq (Standard 21,409 / Allflame 1,312, poe2 realm 98; `rarity` null on 8,297 of 22,721). · agent-seat F4, data/questions.json A1 ≈ S2
S185 · measure · — · The CLI seat answered all twelve questions and the MCP seat five: the CLI's route to the other seven was `items search "" --limit 100000 --json` to a file (63.5 MB) then jq at 0.8–1.5 s a question; Q3, Q5's items, Q6, Q7, A1's league and rarity counts and A5 have no MCP route (112 calls: 29 CLI, 23 MCP, 22 jq, 38 SQL). · agent-seat F5, data/calls.csv
S186 · store · — · Nothing explains: a row carries no matched-on field, an empty answer is `[]` (JSON) or `0 item(s)` (text) and never the scope searched, and `items search Explode` is empty on both seats while 32 items of the store copy's 22,721 carry an explode line. · agent-seat F7 ≈ S107
S187 · store · — · Under read-only SQL each of the twelve was one call (two for Q5 and Q7) on both files, at the price of body idioms — a number inside display text (`substr`/`instr`/`CAST`), entries that are objects in one array and bare strings in another, "every array" as a `UNION ALL` per array name the schema does not list: A5 is nine lines, 353 items, 196 ms on the store copy against one indexed template equality at 2 ms on the projection (477 items over its 36,139); class is absent, so Q6 went by base. · agent-seat F12, data/questions.json A5 ≈ S148, S151
S188 · measure · — · The seat projection cannot answer sockets, links or colours (Q3: `no such column: links`), names a character location by its id (`tab_name`, Q6, Q7), has no column saying which of the two merged stores a row came from, and holds the template in `lines.line`, so an explain quoting the verbatim line needs the body. · agent-seat F12, data/questions.json phase2_notes.projection_could_not ≈ S135
S189 · idiom · — · The query the agent would have typed for A1's facets, recorded per question during the run: `SELECT league, rarity, count(*) FROM items GROUP BY 1, 2`. · agent-seat F13, data/questions.json A1.phase1.temptation
S190 · requirement · — · "The schema and its vocabularies are readable before any row: fields, value sets (rarity, class, realm, league, container), mod templates ranked by count" · agent-seat R9 (prevents F1; same need as owner-seat R4 — "the owner's dropdown is the agent's discovery") ≈ S174, S30
S191 · requirement · — · "A count mode: facets by tab, league, rarity, class, template, returning no bodies" · agent-seat R10 (prevents F4; same need as "the owner's 'counts and totals' candidate")
S192 · requirement · — · "Every truncated answer says total, returned and how to continue" · agent-seat R11 (prevents F3: 50 of 81, silently)
S193 · requirement · — · "The caller names the fields; the default is a decision view (id, name, base, rarity, location name, the matching lines), never the body" · agent-seat R12 (prevents F3: 31.8 KB for twenty gloves; same need as C53's decision view)
S194 · requirement · — · "Predicates over derived facts — rarity, class, level, links and colours, a mod by template with a numeric value across every array — composed with boolean logic" · agent-seat R13 (prevents F5: seven of twelve unanswerable without the dump; same need as owner-seat R1, R2 and the owner's "specific values") ≈ S171, S172, S162
S195 · requirement · — · "A query composes: the refinement is the previous query plus one predicate (a query object, not a text argument)" · agent-seat R14 (prevents F6; same need as owner-seat R2)
S196 · requirement · — · "Explain: a matched-on per row, an empty answer that names the scope searched, a why-not for one id" · agent-seat R15 (prevents F7)
S197 · requirement · — · "The location's name on the row, beside its id" · agent-seat R16 (prevents F9; same need as owner-seat R6) ≈ S176, S122
S198 · requirement · — · "Every id the surface prints is one `show` accepts: full ids in text, or prefix lookup" · agent-seat R17 (prevents F8)
S199 · requirement · — · "Errors name the wrong value and the fix that applies here: an unknown league lists the known ones; a missing store names the directory" · agent-seat R18 (prevents F10)
S200 · requirement · — · "If a lines table is ever exposed (C48 stands), its template convention and the location names are in the schema, or the miss is silent" · agent-seat R19 (prevents F12)

## The acceptance set

One line per owner question and per agent scenario, verbatim, with
the seat's reading of the answer wanted. The proposals write each out
(note 22); stage 6 reads them as its acceptance tests.

Owner (`owner-seat/data/questions.md`, the table and refinements, 2026-09-13):

- OQ1 · "Do I have a rare that I can use to flesh out the resistances or attribute requirements for a build I'm testing?" · a short list of candidates for one slot, with their resist and attribute lines visible, to pick from; refined: "Sometimes a specific item will have a specific stat, like a crafting base with a specific fractured modifier, or a key build-enabling modifier, or a stat that is needed to complete a defensive layer."
- OQ2 · "Do I have a legacy version of a specific unique item?" · one item or "no", and where it is; legacy is the user's knowledge (S165, S178)
- OQ3 · "I just read about a new interaction someone found and I want gear to test it out." · a list, defined by whatever the interaction needs: a mod, a base, socket colours, a unique
- OQ4 · "Where is that staff from Crucible league someone tried to pay me real money for?" · one item and its tab; league of origin is not a field (S177)
- OQ5 · "I want to practice leveling, so I need to find my leveling gear." · a set of items across tabs, by level bracket
- OQ6 · "What is that legacy explode chest I have worth?" · a price, outside the stash search, after finding the item and knowing it is the legacy version
- OQ7 · "Do I have any gear with the modifier GGG just announced is going away except on Standard?" · a list across all tabs and characters, item and tab, to decide keep or sell

Agent (`agent-seat/data/questions.json`, 2026-09-14; the seven owner questions were also run in this seat, their `wanted_query` readings in the same file):

- AQ1 · "Facets before rows: how many items per tab, per league, per rarity?" · three small tables of counts, no rows
- AQ2 · "Refine the previous answer without restating it: of the Q1 rares, only those with 60 or more total resistance." · a shorter list, the same shape; the previous query plus one predicate
- AQ3 · "Explain why an item matched or did not." · a per-row matched-on field; an empty answer that names the scope searched
- AQ4 · "Find one item again by id." · one item, exact
- AQ5 · "One whole-corpus mod query with a value: +# to maximum Life at 90 or more." · a count and a sorted list, across every mod array

## Limits register

Every `limit` claim, with what the search says when it meets it (an
unknown shown, never guessed, is the model).

- S12 — a veiled line is shown as the placeholder it is; a value query never matches it.
- S14 — a map's area is not a fact the search holds; a query for one gets an unknown, never the icon's art.
- S52 — a line is matched as displayed text and value; which mods made it is not known and never guessed.
- S53 — an item's class is not a field; a class the search names is a derivation it owns, and an item it cannot class is shown unclassed.
- S67 — a line whose trade number the hash and the site both leave undecided is shown with both candidates, never one guessed.
- S107 — the model for this register: a line the search cannot name is shown as unknown, with what it knows of it, and never fuzzy-matched.
- S111 — a line whose text has moved since the search last named it falls to the unknown path; nothing re-guesses it, and a name the code itself relies on fails loudly.
- S177 — league of origin is not a fact the search holds; a derivation from league-specific mods or first-seen is offered as a derivation, never as the answer.
- S178 — legacy is the user's knowledge; the search finds what the user names and never says whether the game can still produce it.

## Kill list

- agent-seat: F6, F8, F9 — budget (each carried by its R-line's *prevents*: R14, R17, R16); F10 the error census — budget, its `…` registration trap and the stdio driving — internals; F11 the MCP-text audit against C53 — budget, the first candidate for a new id when a proposal reaches for it; "The projection's requirements" paragraph — budget (S188 carries it); the two open questions — experiments, not findings.
- owner-seat: F1's identity refinement quote — carried by OQ1 in the acceptance set; `data/questions.md`'s *Wants back* and *Today* columns — the first is the acceptance set, the second cpp-search's route.
- engine-bench: Inputs, Outputs, Provenance tables — internals; F6 (the ~0.33 s C++ filter loop at ~1 M against this scan's 88 ms, not like-for-like) — budget; F7 (the seat projection's DDL, per-column derivations, `kind` histogram, the mean hiding a second number on 11,020 of 125,006 lines, `ultimatumMods` in neither file) — budget; F1's per-scale worst column (`q03` 1.27 ms at 36,139, 13.3 at 361,390) — budget; Q2–Q5, the experiments not run (FTS5 over the pretty names, update cost with six indexes live, a struct materializing every census field over 1 % of items, several searches at once over one corpus) — budget.
- store-as-built: F7 — moot (registry rulings, cited as `C<n>`, never re-digested); F4 column exposure (`w`/`h` in no read type, `x`/`y` only in `ItemSnapshot`, read-time `json_extract`s `$.note` on 184 and `$.inventoryId` on 34,161 of 36,139 items) — budget; the surface-sizing Numbers rows (17 public reads, one uncalled: `Store::orphaned_item_annotations`; 8 tables, 4 indexes; 12 queries planned, 5 as full scans) — sizing.
- prior-art: F8 (no corpus, no persistence, no ranking, no mod-level identity, `modFamily` 187, no PoE2) — budget; F7 the item-search widget (two `items.ndjson` slices, ≥ 3 characters, cap of 5, OCR ranking) — budget; F4 the markup half (0 lines carry `[Tag|Display]`; `<<set:…>>`/`<if:…>` handled only on the name plate) — budget; F1 the hash-collision re-check, Q2 — internals; Q3 — moot; Q4 — sizing.
- item-filter: F3 `Rarity` ordering, `Sockets`/`SocketGroup` compound operand, `HasInfluence` granularity, `Class` substring-matched on the C++ side — budget; F4 `BaseDefencePercentile` and the conditions with no export field — budget; F6 the 17 absent conditions by name — budget; Q2 `duplicated` as the unverified candidate for `Mirrored` — budget; the 14 actions — moot; the scripts' join-validation aborts — internals.
- repoe: F1 229 ids listed twice, older text joined for 263 rows — budget; F2 the owner's acceptance-rule quote — budget; F3 site agreement 3,363/1/8/586, clipboard `Item Class:` = export class for 46 bases — budget; F4 unmatched equipment remainder (heist `Alert Level`, `Has # Abyssal Socket`, cluster-jewel passives, necropolis) — budget; pob-format.md the C++ export's `craftedMods`/`fracturedMods` reading — internals; Q8 access method — open question, not a fact.
- trade-query: F1 request body, `query` decomposition, realm by omission, default sort, 100/500/10,000 caps — budget; F1 `status` options — moot; F2 `count`/`weight`/`weight2`/`crucible`/`mercenary` tips — budget; F3 per-group counts, map/heist/sanctum/ultimatum open items — sizing; F3 "what a stash search adds" — budget; F4 per-category counts, id-kind prefixes, C++-pseudomod comparison — sizing; F5 renderer array order, `frameType` values — internals; F6 realms, leagues, `items.json`/`static.json` counts — sizing; F7 `extended.hashes`, the computed `dps`/`pdps`/`ar`/`ev`/`es` (`pdps` = average physical × APS × 1.2; ≈ S26) — budget; F8 Q3 the owner's expectation before q3b, Q4 `weight`/`count` shapes, `complexity` 85, `inexact` — budget.
- cpp-search: F1 refresh split, F6 mechanisms (not its numbers) — internals; F4 `m_replace_map` — moot.
- item-facts: F7 `mn`/`mg`/`mi`/`mc` and the icon payload past the tier — unread; F2 `crucible.nodes`, `frameTypeId` composition — internals; Numbers' per-store and file-size tables — sizing.

## The question index

Note 22's ranked questions, each with the claims that bear on it;
extended after every merge.

1. A line's identity, and a line it cannot name — S3, S4, S5, S11, S12, S25, S27, S28, S29, S43, S44, S45, S47, S48, S52, S63, S64, S65, S66, S67, S71, S72, S73, S76, S84, S101, S102, S103, S104, S105, S106, S107, S109, S110, S111, S154, S155, S156, S165, S171, S178, S186, S196, S200
2. The query model and its one grammar — S2, S6, S7, S9, S10, S21, S22, S24, S32, S39, S41, S47, S54, S68, S81, S83, S90, S91, S95, S106, S121, S125, S158, S162, S163, S166, S167, S169, S172, S173, S182, S189, S194, S195
3. The vocabulary read, served to a human and an agent — S2, S5, S6, S8, S13, S29, S30, S42, S43, S44, S55, S61, S62, S69, S70, S74, S82, S85, S86, S87, S89, S92, S93, S94, S108, S112, S168, S174, S181, S184, S190, S191
4. Who holds the corpus, the derivation and its contract — S1, S11, S13, S15, S31, S37, S50, S75, S110, S112, S122, S124, S126, S127, S128, S129, S130, S131, S134, S135, S141, S142, S143, S144, S145, S146, S147, S148, S149, S150, S151, S152, S153, S157, S183, S185, S187, S188
5. What crosses the trade boundary — S7, S8, S26, S29, S41, S42, S45, S46, S49, S51, S54, S55, S56, S61, S62, S66, S67, S69, S86, S90, S106, S108, S109, S147, S155, S175, S179
6. What a result carries — S1, S11, S31, S53, S107, S127, S132, S133, S134, S143, S161, S176, S177, S183, S186, S192, S193, S197, S198
7. The non-goals and limits, as outputs — S12, S14, S52, S53, S67, S84, S107, S111, S164, S170, S177, S178, S180, S192, S199

## Contradictions and contrasts

Two claims that disagree, or two numbers that look alike and are not, each with its denominators; a convergence is the `≈ S<m>` on the claim line.

- S70 ≈ S8, contrast: `baseType` joins an export base for 98.2 % of PC items but a trade category for 78.3 %, because `Map (Tier N)` is an export base and no trade base.
- S87 ≈ S21, S22, contrast: the filter language names no computed number where the C++ app derives six and the site serves them.
- S144 ≈ S37, contrast, not like-for-like (engine-bench F6): the C++ app's unindexed `FilterItems` at 329 ms for 975,711 items against this scan's 88 ms for twelve queries at 1,084,170.
- S156 against S72: 259 of 3,478 mapped templates ambiguous here (all arrays, first trade id wins) and 150 of 3,402 in repoe (equipment explicit, the twins); different sets, not a contradiction.
- engine-bench F2's prose gives `q06` at 1,084,170 as 0.30 ms and `q09`'s scan as 7.1 ms; results.csv has 0.659 (delivering rows; 0.317 is the `count(*)` row) and 8.208. F1's table gives the ×10 worst as 13.1 ms; results.csv and the Numbers table 13.3. S147 and S148 carry the data file.
- S200 against S25, contrast: the seat projection's template folds the sign into the number (`# to maximum Life`); the C++ template keeps it (`+#%`); repoe's canonical form drops a sign before a placeholder (S71); three conventions for one line.
- agent-seat F7 counts 32 items with an explode line (phase one, the store copy, 22,721 items); phase two found 271 bodies containing the word on the same copy and 58 explode lines on the 36,139-item projection — three measures, not one; S186 carries F7's.
- trade-query F4 says "380 pairs"; stat-collisions.csv has 380 groups of two to four ids (369/8/3). S44 carries the data.
- item-facts F5 itemises the unmatched bases to 7,885 (7,512 + 272 + 98 + 3) while `data/numbers.md`'s `frameTypeId` table sums them to 7,826 (36,139 − 28,313); a 59-item gap the track does not explain. S8 carries F5's figures.
- item-facts F3 counts PoE2 markup over four arrays (192 lines); `data/numbers.md` lists eight (220). S4 carries the data file.
