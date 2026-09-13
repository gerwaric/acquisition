# trade-query — the trade site's query language as a spec

Status: in progress — 2026-09-12. Static data and the page read; the request/response and `fetch` samples are still to capture.

Headline:
- The site sends `{query, sort}` to `POST /api/trade/search[/<realm>]/<league>`, pc by omission. `query` is `status`, then `term` *or* `name`/`type` (each a string or `{option, discriminator}`), `filters` keyed by group, and `stats`: groups of eight types (`and`, `not`, `if`, `count`, `weight`, `weight2`, `crucible`, `mercenary`) whose semantics GGG states in tips, captured verbatim (F2).
- 12 filter groups, 93 filters. 59 read something a private-API item carries — the site's own renderer maps 70 numeric property types to filter keys (F5), 7 read market state, 27 are open; the 83-id item category taxonomy is the largest open item (F3, `data/gap.csv`).
- The stat vocabulary is 14 categories, 18,187 entries, 14,193 distinct stat keys. The key is the numeric stat id; the category is the mod's source. Display text does not determine the key: 380 (category, text) pairs map to several ids, 94 in `explicit` alone, and 93 texts carry `(Local)` (F4, `data/stat-collisions.csv`).
- 298 pseudo stats, the C++ pseudomods among them; 206 are enumerations (temple rooms, logbook areas, lake reflections) and the affix-count pseudos need knowledge the private API does not report (F4).
- The client computes nothing stat-specific: no stat id appears in the bundle, so how the server sums pseudos, picks among colliding ids, or reads multi-value lines is **not in this capture** (Q2–Q5).

## Question

What is the trade site's query language — filter groups and keys, option vocabularies, stat group types, sort keys — and what is observable about its semantics? Then the gap: which filters mean something for a private stash, which do not, and what a stash search adds.

## Inputs

| Source | Access | Status |
| --- | --- | --- |
| `/api/trade/data/{stats,items,static,filters}` | `browser`, owner, 2026-09-12; committed dated under `data/` (index rule 4) | landed |
| The search page, saved complete, with its bundles (`trade.*.js`, `main.*.js`) | `browser`, owner, 2026-09-12; `raw/`, local only | landed |
| One search request body + response; one `fetch` response for a rare item | `browser` network tab, owner | pending |

The site is a governed surface (C79; `SURFACES.md`, whose data-endpoint row records this capture). Every hash and byte count is in `MANIFEST.md`.

## Findings

**F1 — The request** (`data/grammar.json`, `request`; source: `PoE/Trade/Service`, `PoE/Trade/App.query()`).

| Element | Shape |
| --- | --- |
| URL | `POST /api/trade/search` + (`/<realm>` unless `pc`) + `/<league>`; exchange: `/api/trade/exchange` likewise |
| Body | `{query, sort}`; default sort `{"price": "asc"}`; a column click toggles `{<field>: asc\|desc}` |
| `query.status` | `{option}` from `available \| securable \| onlineleague \| online \| any` |
| `query.term` | free text; when set, `name` and `type` are not sent |
| `query.name`, `query.type` | a string, or `{option, discriminator}` when the known item carries a `disc` (variants) |
| `query.filters` | `{<group id>: {filters: {<filter id>: value}, disabled?}}`; a group is sent only when it has filters |
| filter value | `{min?, max?}` \| `{option}` \| sockets `{r?, g?, b?, w?, min?, max?}` \| text input \| a known-item pick |
| `query.stats` | `[{type, filters: [{id, value: {min?, max?, weight?, option?}, disabled}], value: {min?, max?}?}]` |
| Limits | 100 results per search, 500 per live search (`resultLimit`, `liveResultTotalLimit`) |
| Persisted state | `{tab, name, type, disc, term, realm, league, status: "any", filters: {}, stats: [{type: "and", filters: []}]}` |

**F2 — Stat group types**, GGG's tips verbatim (`grammar.json`, `stat_groups`).

| Type | Title | Group value | Tip |
| --- | --- | --- | --- |
| `and` | And | — | — |
| `not` | Not | — | — |
| `if` | If | — | "Match items that meet each stat's `min` and `max` requirements if the stat is present." |
| `count` | Count | min, max | "Count each stat that meets the `min` and `max` (if provided, otherwise existence) requirements. Use the group's `min` and `max` to filter items based on the count of matching stats." |
| `weight` | Weighted Sum | min, max; per-stat `weight` | "Check each stat meets the `min` and `max` (if provided, otherwise existence) requirements before multiplying the stat value by the `weight` and finally summing them together. …" |
| `weight2` | Weighted Sum v2 | min, max; per-stat `weight` | "Each stat value that meets the `min` and `max` (if provided, otherwise existence) requirements will be multiplied by the `weight` before being summed together. …" |
| `crucible` | Crucible Passive Tree Path | min only; not mutable | "Filter by the mods that you want to be able to allocate at once. Use a lower `min` value for partial matches." |
| `mercenary` | Mercenary Skill Group | min only; not mutable | "Filter by a skill and supports that a Mercenary Warrant should have. …" |

**F3 — Filter groups and the gap** (`data/filters-2026-09-12.json`; assessment `data/gap.csv`, rule table in `scripts/gap.py`). `local` = reads a property or field a private-API item carries, or is derived from them; `none` = market state; `open` = not settled by this capture.

| Group | Filters | local | none | open | Open items |
| --- | --- | --- | --- | --- | --- |
| `status_filters` | 1 | | 1 | | |
| `type_filters` | 2 | 1 | | 1 | `category` (83 ids) |
| `weapon_filters` | 6 | 6 | | | `dps`/`pdps`/`edps`/`damage` derived; computation not captured |
| `armour_filters` | 6 | 5 | | 1 | `base_defence_percentile` |
| `socket_filters` | 2 | 2 | | | derived from the sockets array |
| `req_filters` | 5 | 4 | | 1 | `class` |
| `map_filters` | 12 | 6 | | 6 | series, blighted ×2, chart ×2, completion reward |
| `heist_filters` | 16 | 13 | | 3 | the three `max_*` totals |
| `sanctum_filters` | 4 | 3 | | 1 | `sanctum_max_resolve` |
| `ultimatum_filters` | 4 | | | 4 | all |
| `misc_filters` | 29 | 19 | | 10 | transfigured, imbued, foreseeing, vestigial, intangibility, alternate art, corpse type, scourge tier, crucible, mutated |
| `trade_filters` | 6 | | 6 | | seller, collapse, indexed, sale type, fee, price |

What a stash search **adds** and the site has no word for: tab and character, container, realm and league, liveness (`removed_at`), first/last seen, socketed-in, and our own listing state — the analogue of `sale_type` (`priced_with_info`, `unpriced`), which the design decides.

**F4 — The stat vocabulary** (`data/stats-2026-09-12.json`; `data/stat-summary.csv`, `data/stat-collisions.csv`).

| Category | Entries | Distinct texts | Texts with 0 / 1 / 2 / 3+ `#` | Id kinds |
| --- | --- | --- | --- | --- |
| `pseudo` | 298 | 298 | 111 / 166 / 21 / 0 | `pseudo` 245, `lake` 53; 88 carry `option` lists |
| `explicit` | 7,896 | 7,660 | 2,282 / 5,419 / 190 / 5 | `stat` 7,409, `indexable` 459, `pseudo` 28 |
| `implicit` | 1,834 | 1,814 | 280 / 1,494 / 60 / 0 | `stat`, `pseudo` 16 |
| `fractured` | 1,833 | 1,818 | 516 / 1,197 / 116 / 4 | `stat` |
| `enchant` | 2,037 | 1,998 | 778 / 1,247 / 12 / 0 | `stat`, `delirium` 21 |
| `crucible` | 2,492 | 2,418 | all 0 | `mod`; 1,682 multi-line, tiered, values baked in |
| `scourge` 409, `crafted` 288, `mercenary` 534, `sanctum` 240, `imbued` 162, `delve` 81, `ultimatum` 63, `veiled` 20 | | | | `mercenary` = `skill`/`support` ids; `imbued` = `pseudo_built_in_support\|N` |

- A stat key appears in 1 category for 11,694 keys, in 2 for 1,744, in 3–6 for 755: the category is the mod's source, the id is the stat.
- Collisions: 380 (category, text) pairs with several ids. Three kinds: `stat` vs `stat` (two stats, one rendering: `+#% chance to Suppress Spell Damage`, the Mana Reservation Efficiency family, the aura `Grants Level #` family); `stat` vs `indexable_support_N` / `indexable_skill_N` (the "Socketed Gems are Supported by Level #" family, 150-odd); and `stat_N|a|b` variants (the Fresh Meat lines). 93 `(Local)` texts distinguish weapon- and armour-local from global for lines that read identically.
- Pseudo: the C++ pseudomods (`docs/user/mods-and-pseudomods.md`) are all present under `pseudo_total_*`; the site adds `pseudo_adds_*` damage families, `pseudo_number_of_{prefix,suffix,crafted,empty,fractured}_mods` (need affix knowledge), influence flags, jewellery and map quality pseudos, and 206 enumerations (temple rooms, logbook factions and areas, lake reflections).

**F5 — The bridge to the item JSON** (`grammar.json`, `property_type_to_field`, `renderer_mod_arrays_in_order`, `hash_category_to_css_class`; source: `PoE/Item/Popup` and `PoE/Trade/Component/Item`). The renderer maps `properties[].type` to a filter field for 70 type ids: 1–4 map tier/iiq/iir/pack size, 5 gem level, 6 quality, 9–13 physical/elemental/chaos damage, crit, aps, 15–18 block/armour/evasion/es, 20 gem experience, 32 stack size, 34 area level, 35–47 heist, 54 ward, 62–65 level/str/dex/int, 68–70 sanctum, 78 ilvl, 80–91 map pseudos, 97–107 more. The renderer's mod arrays, in display order: `utilityMods, enchantMods, runeMods, scourgeMods, implicitMods, fracturedMods, mutatedMods, explicitMods, bondedMods, craftedMods, desecratedMods, pseudoMods, cosmeticMods, crucibleMods` (`veiledMods`, `ultimatumMods`, `logbookMods` separately). Result-side, `extended.hashes[<category>]` is aligned to those arrays: `monster`, `delve`, `sanctum` land in `explicitMods`; `rune` and `desecrated` are PoE2. Rarity is `frameType` (0 normal … 3 unique, 9 foil, 10 supporter foil).

**F6 — The page.** Realms `pc`, `xbox`, `sony` ("PoE 1 PC/Xbox/Sony"), eight leagues each, no `poe2` on this site (T4 reaffirmed). `items.json`: 22 categories, 3,733 base types, 1,546 uniques flagged, 725 entries with a `disc` discriminator. `static.json`: the 23 exchange groups (T33).

## Numbers

| | |
| --- | --- |
| Filter groups / filters / option-bearing filters | 12 / 93 / 27 |
| Stat categories / entries / distinct keys / distinct texts | 14 / 18,187 / 14,193 / 13,975 |
| Property types mapped to fields | 70 |
| Capture bytes: stats / items / static / filters | 2,099,466 / 344,615 / 199,012 / 17,234 |

## Open questions

- **Q1 — The category taxonomy.** Where do the 83 `category` ids come from, given the private-API item has no class field? Closes: the `fetch` sample (does an item carry a class?), then the item-facts census against `items.json`'s 22 top-level groups.
- **Q2 — Which id does a colliding line get?** Closes: `fetch` samples whose items carry lines from `stat-collisions.csv`, read at `extended.hashes`.
- **Q3 — Multi-value lines.** `Adds # to # Physical Damage`: one value or two, and which does `weight` use? Closes: one search with a `min` on such a stat plus its `fetch`.
- **Q4 — Pseudo composition.** Which lines sum into `pseudo_total_fire_resistance`? Not in the capture; the C++ table is a hypothesis. Closes: hand tests on the site with known items, or `extended.mods` in `fetch` samples.
- **Q5 — Owner's hypothesis, verbatim (2026-09-12):** "the trade site likely has a collection of hard-coded edge cases. This has to do with how some specific stats are rendered, and how multiple stats can be additive to a single display modifier that shows up in json." "I'm not 100% confident in this at all." Evidence so far: the client holds no such table. Closes: `fetch` samples where several `extended.mods` entries contribute to one displayed line.
- **Q6 — The 27 open filters and the 13 result-side categories** against what the private API actually reports: the item-facts track.

Candidate claims for `trade-ground-truth.md` (T-candidates, for the design session to route): the URL's pc-by-omission (the same rule as C58); the eight stat group semantics verbatim; the 100/500 result limits.

## Provenance

`MANIFEST.md` (every file, hash, source). Generated: `scripts/extract-grammar.py` → `data/grammar.json`; `scripts/stat-collisions.py` → `data/stat-summary.csv`, `data/stat-collisions.csv`; `scripts/gap.py` → `data/gap.csv`. Nothing in this file comes from memory of the site; a fact not in a listed file is marked open.

## Review

| Date | Reviewer | Finding | Disposition |
| --- | --- | --- | --- |
