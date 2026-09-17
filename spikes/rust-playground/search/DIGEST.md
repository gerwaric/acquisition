# The search digest

Status: partial — item-facts, cpp-search, trade-query, repoe, item-filter, prior-art — 2026-09-16

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

## item-filter

S81 · ggg · main · Conditions in a `Show`/`Hide`/`Minimal` block are ANDed — "If there are multiple conditions in a block then all of them must be matched for the block to match an item."; the page names blocks, `Continue`, `Import` and conditions, and no OR or grouping · "The language, as the page states it"
S82 · measure · — · The 2026-09-13 page save names 61 conditions and 14 actions (no action reads an item); ribbons: 6 `PoE2-only`, 3 `New`, 1 `Updated`, 1 `Deleted` (`GemQualityType`); operands: 27 boolean, 22 numeric, 4 name, 3 enum, 5 compound · Numbers, conditions.csv, actions.csv
S83 · ggg · main · One operator table of eight (`=` `!` `!=` `<` `<=` `>` `>=` `==`) serves every condition, no per-condition restriction stated; `conditions.csv`'s `operators` column is this track's derivation, not a page statement · "The language, as the page states it"
S84 · ggg · main · What a bare operator-less value matches — prefix, substring or exact — is unstated for `BaseType`, `Class`, `HasEnchantment`, `HasExplicitMod`, as is whether `HasExplicitMod`'s count ranges over the named mods or all mods; no real filter was read, so nothing here is checked against usage · F1, Q1, "Not done"
S85 · measure · — · Of the 61 conditions, 32 read a stash-export field directly, 7 are derivable, 5 need base-item data (RePoE), 17 have no source found · F6, conditions.csv `item_field`
S86 · measure · — · Of the 61, 22 are named by both other surfaces, 14 by trade alone, 1 by the C++ app alone, 24 by neither (7 joins land on a trade stat, not a filter id); in reverse it names 19 of the C++ app's 38 filters (14 exact, 5 divergent) and 27 of trade's 86 item-reading filters (21 exact, 6 divergent) · Numbers, conditions.csv, coverage.csv
S87 · ggg · main · It names no computed number and no requirement: no DPS of any kind, no APS, crit or block, no total defence, no `R. Level`/`R. Str`/`R. Dex`/`R. Int` and no trade `damage`/`lvl`/`str`/`dex`/`int`; nothing about a listing; and no `Crafted`, `Veiled` or `Split`, which both other surfaces name · F5, coverage.csv `kind=none`
S88 · owner · — · "item filters are design by GGG to be unable to require specific numbers, because GGG wants people to experience the randomness of checking loot" (2026-09-13) · F5
S89 · ggg · main · One thing carries a spelling per surface: the Foulborn flag is `Foulborn` here, C++ `Mutated`, trade id `mutated` with the text "Foulborn", export `mutated`; synthesised is `SynthesisedItem`/`Synthesized`/`synthesised_item`/`synthesised`; five more rows at the pointer · F2
S90 · ggg · main · Two conditions diverge in referent, not spelling: `AreaLevel` is the area the item dropped in, trade's `area_level` a map's own level; `BaseArmour`/`BaseEvasion`/`BaseEnergyShield`/`BaseWard` read base values before mods where C++ `Armour`/`Evasion`/`Shield` and trade `ar`/`ev`/`es`/`ward` read the total, and nothing here reads a total · F3
S91 · ggg · edge · `HasSearingExarchImplicit` and `HasEaterOfWorldsImplicit` compare an implicit tier 1–6 while trade's `searing_item`/`tangled_item` and export `searing`/`tangled` are yes/no; `Scourged` is yes/no while trade's `scourge_tier` and export `scourged.tier` carry a tier · F3
S92 · ggg · main · It names `Width` and `Height`; the export carries `w` and `h` on every item in the census (share 1.000) and neither other surface offers a filter for either · F4, item-facts field-census.csv
S93 · ggg · main · No class vocabulary is on the page: `Class` takes an "Item class name" with the one example `Class Currency`, and `class-names.csv` holds one `class_value` row (Currency) plus five tokens occurring only inside other phrases; the class join rests on repoe F3 · F7, Q3, class-names.csv
S94 · ggg · main · One page serves PoE1 and PoE2: six conditions are ribboned `PoE2-only` (listed at the pointer), among them `WaystoneTier`, which is PoE2's `MapTier` renamed; whether those ribbons enumerate the whole PoE1/PoE2 divergence is open · F8, Q4
S95 · idiom · — · GGG's own example on the page, verbatim: `HasExplicitMod >=2 "of Haast" "of Tzteosh" "of Ephij"` — a page example, not a filter observed in use · F1, conditions.csv row `HasExplicitMod`

## prior-art

S101 · tool · — · Identity is the displayed line's own English text: fnv1a-32 of the matcher string, binary-searched over (hash, byte offset) pairs into one ndjson line of a 2.5 MB file held as a single string; the key is itself a sentence · F1
S102 · tool · — · `ref` is English in every language file (`ru/stats.ndjson` line 1 has the English `ref` with Russian matchers); only `matchers[].string` is translated · F1
S103 · tool · — · Numbers are removed by trying every placement: up to 11 candidates of "which numbers stay literal, which become `#`", most-placeholders-first with raw text last, each looked up until one hits whose `trade.ids` holds the modifier type; reversed wordings carry `negate: true` (1653 matchers) and flip the roll's sign; the advanced-mod-description wording is a second key, `advanced` (1420 matchers), indexed instead of `string` when present · F2
S104 · tool · — · Ambiguity is a two-stat `StatGroup` with a named resolver: 100 of 9178 lines, each exactly two stats, resolver one of `trivial-merge` 43, `select` 41 (by the item's category), `percent-merge` 11, `flag-merge` 5 · F3
S105 · tool · — · The local/global twin is one line with two trade ids: `#% increased Armour` appears twice with identical `ref` and identical matchers, told apart only by `resolve.test: ["ARMOUR", null]` · F3, stat-model.md "A local/global twin — line 489"
S106 · tool · — · No entry carries two ids in one modifier category (0 of 9278); 2294 carry ids in several. A group's second id is merged into the list at match time, and a filter with several ids becomes a trade `count` group with `value.min = 1`, an OR · F3, Numbers
S107 · limit · main · Unknown lines are kept, never guessed: the leftover goes to `item.unknownModifiers` with its modifier type and renders as an orange "Not recognized modifier"; no fuzzy match on mod text exists anywhere (the only Levenshtein is over OCR'd gem names) · F4
S108 · measure · — · Its 12507 distinct trade ids are all known to the 2026-09-12 trade capture (0 unknown) and cover 12507 of that capture's 17958; the 5451 lacked include crucible 2492, scourge 409, delve 81, ultimatum 63 at zero, 187 of 240 sanctum, 944 enchants, 1026 explicits; all 88 option-bearing capture ids are carried · F5, stat-model.md "Coverage of the trade capture"
S109 · measure · — · Of the 380 trade-id collisions the trade-query track found: 57 resolved by a group (30 trivial-merge, 24 select, 3 percent-merge), 83 kept as separate entries, 143 with one id carried, 97 with neither, 0 as several ids in one entry · F5, Numbers, coverage.csv `collision_verdict`
S110 · measure · — · Over 55 data commits to `stats.ndjson` (2022-03-24…2026-09-09, median gap 12 days, max 205) refs grew 6139 → 9198, 31 of 54 diffs removed at least one ref, `d876a14` added 1402 refs, `9a012b5` ("update stats for 3.26.0.15") removed 930 refs and 1434 trade ids; 2436 times a trade id survived while the text it displays changed, 526 of them in `9a012b5` · F6, cadence.csv
S111 · limit · main · A rename fails loudly for the 210 refs hard-coded as `stat()` string literals in 10 source files, each asserted at startup with a `Cannot find stat: X` throw, and silently — one orange line — for the other ~9000 · F6
S112 · tool · — · The dataset generator is not in the repository: no script writes `stats.ndjson`, the only build step over it regenerates the four `*.index.bin` files, data arrives as opaque "update data" commits, and what generates it from what is unknown · F6, Q1
S113 · tool · — · Its stash search does no matching: a saved string goes on the clipboard and Ctrl+F, Ctrl/Cmd+V, Enter are synthesised into the game window; the corpus is the game client's own box, the tool never parses the string, and the only GGG endpoints it calls are `/api/trade/search` and `/api/trade/fetch` — no stash endpoint in `renderer/src`, `main/src` or `ipc/` · F7, stash-search.md
S114 · tool · — · The game highlights the matching cells and the tool learns nothing back; there is no ranking, and the editor turns the input red past 250 characters, the game's field length · stash-search.md "The two features" (Result, Ranking, Limits rows)
S115 · idiom · — · Ten stash-search strings ship as defaults, four for map rolling and six for dump sorting, e.g. `"Pack Size: +3"` and `"Map Device" "Rarity: Normal"` · stash-search.md "Defaults shipped"

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
- S107 — the model for this register: a line the search cannot name is shown as unknown, with what it knows of it, and never fuzzy-matched.
- S111 — a line whose text has moved since the search last named it falls to the unknown path; nothing re-guesses it, and a name the code itself relies on fails loudly.

## Kill list

- prior-art: F8 (no corpus, no persistence, no ranking, no mod-level identity, `modFamily` 187, no PoE2) — budget; F7 the item-search widget (two `items.ndjson` slices, ≥ 3 characters, cap of 5, OCR ranking) — budget; F4 the markup half (0 lines carry `[Tag|Display]`; `<<set:…>>`/`<if:…>` handled only on the name plate) — budget; F1 the hash-collision re-check, Q2 — internals; Q3 — moot; Q4 — sizing.
- item-filter: F3 `Rarity` ordering, `Sockets`/`SocketGroup` compound operand, `HasInfluence` granularity, `Class` substring-matched on the C++ side — budget; F4 `BaseDefencePercentile` and the conditions with no export field — budget; F6 the 17 absent conditions by name — budget; Q2 `duplicated` as the unverified candidate for `Mirrored` — budget; the 14 actions — moot; the scripts' join-validation aborts — internals.
- repoe: F1 229 ids listed twice, older text joined for 263 rows — budget; F2 the owner's acceptance-rule quote — budget; F3 site agreement 3,363/1/8/586, clipboard `Item Class:` = export class for 46 bases — budget; F4 unmatched equipment remainder (heist `Alert Level`, `Has # Abyssal Socket`, cluster-jewel passives, necropolis) — budget; pob-format.md the C++ export's `craftedMods`/`fracturedMods` reading — internals; Q8 access method — open question, not a fact.
- trade-query: F1 request body, `query` decomposition, realm by omission, default sort, 100/500/10,000 caps — budget; F1 `status` options — moot; F2 `count`/`weight`/`weight2`/`crucible`/`mercenary` tips — budget; F3 per-group counts, map/heist/sanctum/ultimatum open items — sizing; F3 "what a stash search adds" — budget; F4 per-category counts, id-kind prefixes, C++-pseudomod comparison — sizing; F5 renderer array order, `frameType` values — internals; F6 realms, leagues, `items.json`/`static.json` counts — sizing; F7 `extended.hashes`, the computed `dps`/`pdps`/`ar`/`ev`/`es` (`pdps` = average physical × APS × 1.2; ≈ S26) — budget; F8 Q3 the owner's expectation before q3b, Q4 `weight`/`count` shapes, `complexity` 85, `inexact` — budget.
- cpp-search: F1 refresh split, F6 mechanisms (not its numbers) — internals; F4 `m_replace_map` — moot.
- item-facts: F7 `mn`/`mg`/`mi`/`mc` and the icon payload past the tier — unread; F2 `crucible.nodes`, `frameTypeId` composition — internals; Numbers' per-store and file-size tables — sizing.

## The question index

Note 22's ranked questions, each with the claims that bear on it;
extended after every merge.

1. A line's identity, and a line it cannot name — S3, S4, S5, S11, S12, S25, S27, S28, S29, S43, S44, S45, S47, S48, S52, S63, S64, S65, S66, S67, S71, S72, S73, S76, S84, S101, S102, S103, S104, S105, S106, S107, S109, S110, S111
2. The query model and its one grammar — S2, S6, S7, S9, S10, S21, S22, S24, S32, S39, S41, S47, S54, S68, S81, S83, S90, S91, S95, S106
3. The vocabulary read, served to a human and an agent — S2, S5, S6, S8, S13, S29, S30, S42, S43, S44, S55, S61, S62, S69, S70, S74, S82, S85, S86, S87, S89, S92, S93, S94, S108, S112
4. Who holds the corpus, the derivation and its contract — S1, S11, S13, S15, S31, S37, S50, S75, S110, S112
5. What crosses the trade boundary — S7, S8, S26, S29, S41, S42, S45, S46, S49, S51, S54, S55, S56, S61, S62, S66, S67, S69, S86, S90, S106, S108, S109
6. What a result carries — S1, S11, S31, S53, S107
7. The non-goals and limits, as outputs — S12, S14, S52, S53, S67, S84, S107, S111

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
- S92 ≈ S2: `w` and `h` on every item, from the census and from the filter language's `Width`/`Height`.
- S93 ≈ S9, S53, S69: no class vocabulary on the filter page, no class field on the item; the join rests on the export's classes.
- S94 ≈ S10: a map's tier is a base-type fact on PoE1 items and a `WaystoneTier` condition on PoE2's page.
- S87 ≈ S21, S22, contrast: the filter language names no computed number where the C++ app derives six and the site serves them.
- S90 ≈ S26: the filter language's `Base*` defences read base values; the C++ app and the site read totals, the site's with quality normalised.
- S101 ≈ S25, S71: identity by the displayed line's text in Awakened, in the C++ template and in repoe's text join; three tools, one key, a sentence.
- S105 ≈ S72, S44: the local/global twin, one text with two ids, resolved by Awakened per item category, by repoe per `is_local`.
- S109 ≈ S44, S66: the 380 collisions, counted by trade-query, 175 attributed by repoe's hashes, 57 grouped and 97 uncarried by Awakened.
- S108 ≈ S61: the capture's 17958 distinct ids are its 18,187 entries less the 229 listed twice (repoe F1); Awakened carries 12507, the export names 11,004.
- S106 ≈ S41: several ids for one line go outward as a trade `count` group with min 1, the shape trade-query's F2 documents.
- S110 ≈ S75: the two maintained datasets move on their own cadence, Awakened's at a median 12 days with 2436 re-texted ids, the export's with 33 of 86 game versions never exported.
- trade-query F4 says "380 pairs"; stat-collisions.csv has 380 groups of two to four ids (369/8/3). S44 carries the data.
- item-facts F5 itemises the unmatched bases to 7,885 (7,512 + 272 + 98 + 3) while `data/numbers.md`'s `frameTypeId` table sums them to 7,826 (36,139 − 28,313); a 59-item gap the track does not explain. S8 carries F5's figures.
- item-facts F3 counts PoE2 markup over four arrays (192 lines); `data/numbers.md` lists eight (220). S4 carries the data file.
