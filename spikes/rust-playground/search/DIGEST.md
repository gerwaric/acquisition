# The search digest

Status: partial — item-facts, cpp-search — 2026-09-16

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

## The acceptance set

Written when owner-seat and agent-seat merge.

## Limits register

Every `limit` claim, with what the search says when it meets it (an
unknown shown, never guessed, is the model).

- S12 — a veiled line is shown as the placeholder it is; a value query never matches it.
- S14 — a map's area is not a fact the search holds; a query for one gets an unknown, never the icon's art.

## Kill list

- cpp-search: F1 refresh split, F6 mechanisms (not its numbers) — internals; F4 `m_replace_map` — moot.
- item-facts: F7 `mn`/`mg`/`mi`/`mc` and the icon payload past the tier — unread; F2 `crucible.nodes`, `frameTypeId` composition — internals; Numbers' per-store and file-size tables — sizing.

## The question index

Note 22's ranked questions, each with the claims that bear on it;
extended after every merge.

1. A line's identity, and a line it cannot name — S3, S4, S5, S11, S12, S25, S27, S28, S29
2. The query model and its one grammar — S2, S6, S7, S9, S10, S21, S22, S24, S32, S39
3. The vocabulary read, served to a human and an agent — S2, S5, S6, S8, S13, S29, S30
4. Who holds the corpus, the derivation and its contract — S1, S11, S13, S15, S31, S37
5. What crosses the trade boundary — S7, S8, S26, S29
6. What a result carries — S1, S11, S31
7. The non-goals and limits, as outputs — S12, S14

## Convergence and contradiction

- S23 ≈ S10: no map carries a `Map Tier` property (cpp-search measured it through the app's accessor, item-facts through the census).
- S27 ≈ item-facts F3: the same 905/914/3,978/862 counts, sized in item-facts' numbers.md for cpp-search's template.
- item-facts F5 itemises the unmatched bases to 7,885 (7,512 + 272 + 98 + 3) while `data/numbers.md`'s `frameTypeId` table sums them to 7,826 (36,139 − 28,313); a 59-item gap the track does not explain. S8 carries F5's figures.
- item-facts F3 counts PoE2 markup over four arrays (192 lines); `data/numbers.md` lists eight (220). S4 carries the data file.
