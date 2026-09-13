# cpp-search — the C++ app's search, catalogued

Status: first pass — 2026-09-13. Every input read at `master@946a4f51`; the catalogues are under `data/`. No rule here has been run against a real item payload yet (Q1).

Headline:
- 38 filters in 9 groups over six payload kinds, ANDed in catalog order: 20 min/max, 11 flags, 2 text, 2 combos, 2 socket-colour, 1 mods. The min/max filters split on absence: eight property filters reject an item lacking the property, twelve read it as 0 (F1).
- A searchable mod is a template: every run of `[0-9.]` in a display line becomes `#`, the value is the mean of the line's numbers. Six buckets feed the table (implicit, explicit, crafted, fractured, mutated, enchant); every other array on the item is unsearchable. A duplicate template keeps the last; a pseudomod sums (F3).
- The category taxonomy and the mod dropdown are not in the app: both are RePoE, fetched at runtime from `repoe-fork.github.io` and cached, nothing bundled — a governed surface with no spike access method (F4, `SURFACES.md`).
- 26 columns; one ordering, defined once as a materialized key with a (PrettyName, id, hash) tie-break; two views, keyed on the stable (type, id) location key; default sort Name descending (F5).
- What instant cost: at ~1m items the sort was 93 % of a refresh's reset and was retired by materialized keys and sort-on-expand; what remains is the filter loop, ~0.33 s at ~1m — an unindexed linear scan the app never attacked, deferred by M3 D7 (F6).

## Question

What does the C++ app's search do, filter by filter and column by column: the exact extraction rule behind each, the derivation formulas (DPS, sockets and links, colours, pseudomods, the mod normalizer), the rarity and category taxonomies, the sorts and view modes, and what the delta pipeline paid to keep results instant?

## Inputs

| Source | Access | Status |
| --- | --- | --- |
| `master@946a4f51`: `src/filters/`, `search.*`, `item.*`, `modlist.*`, `pseudomods.cpp`, `column.*`, `items_model.*`, `itemcategories.cpp`, `repoe/`, `sortkey.h`, `bucket.h`, `ui/searchform.cpp` | `git show master:<path>` | read |
| `docs/user/searching.md`, `docs/user/mods-and-pseudomods.md`, `docs/design/items-pipeline*.md`, `m1-m2-result.md`, `m1-m3-result.md`, `m2-m2-result.md`, `m3-sort-profile-result.md` (master) | same | read |
| A real item payload, to check the property strings the rules parse | the item-facts track's database | pending (Q1) |

## Outputs

| File | What | Rows |
| --- | --- | --- |
| `data/filters.toml` | one `[[filter]]` per catalog entry: caption, group, payload, refresh, accessor, the match rule, cite; then `[consumption]` | 38 |
| `data/columns.toml` | one `[[column]]` per column: value, colour, icon, sort key; then `[sort]`, `[views]`, `[model]` | 26 |
| `data/pseudomods.toml` | each pseudomod and the templates it sums, by `scripts/extract-pseudomods.py` | 35 |
| `data/derivations.md` | accessor → JSON field, every derivation formula, the category lookup and RePoE load path, the normalizer, sort identity | 5 tables |
| `data/delta-pipeline.md` | the mechanisms, every measured number, the implications | 3 tables |

## Findings

**F1 — Filters** (`data/filters.toml`). All ANDed; an inactive filter is never evaluated; every item-level predicate is in the catalog. Text filters are case-insensitive substring over the accessor; Name searches `name + " " + typeLine`, Tab searches the header `#<index+1>, "<label>"` or the character name.

| Group | Filters | Absence rule |
| --- | --- | --- |
| TopForm | Tab, Name, Category, Rarity | Category: substring of the lowercased RePoE class name; uncategorised matches nothing. Rarity by `frameType`: 0/1/2 Normal/Magic/Rare, 3 Unique, 9/10 Unique (Foil), "Any Non-Unique" = 0/1/2; any other frame (gems, currency, cards, `-1`) matches no query |
| Offense | Crit., DPS, pDPS, eDPS, cDPS, APS | Crit., APS: **skip** the item when the property is absent; the four DPS: read 0 |
| Defense | Armour, Evasion, Shield, Block | skip when absent |
| Sockets | Sockets, Links, Colors, Linked | counts read 0. Colors: per-colour at-least over the whole item, white sockets covering any shortfall across the three; Linked: the same test satisfied by any one socket group |
| Requirements | R. Level, R. Str, R. Dex, R. Int | read 0 |
| Misc | Quality, Level, Map Tier, ilvl | Quality defaults to 0; Level and Map Tier skip when absent; ilvl reads the field |
| MiscFlags, MiscFlags2 | Alt. art, Priced; Unidentified, Influenced, Crafted, Enchanted, Corrupted, Fractured, Split, Synthesized, Mutated | checked = predicate; Influenced = the six conqueror/elder-war flags; Alt. art = 117 icon-URL needles; Priced reads the buyout manager, the one predicate outside the item |
| Mods | Mods (one entry, any number of rows) | a row matches iff `mod_table` has the exact template and the value is within [min, max], both inclusive; rows AND |

Refresh: 25 debounced (one 350 ms timer), 13 immediate (flags, colours). Bounds inclusive, either side blank. Doc-vs-code: the doc's "Type" is the caption "Category"; Name also searches the type line; the white-socket substitution is undocumented; no eldritch filter exists though the enum has both.

**F2 — Derivations** (`data/derivations.md`, §2). Property strings are used as GGG sent them; only Quality (`+`/`%` stripped), Level (` (Max)` chopped) and Stack Size (`n/m` → n) are normalised. `properties` keeps `values[0][0]` only.

| Value | Formula |
| --- | --- |
| pDPS, cDPS | `APS × avg(range)` of "Physical Damage" / "Chaos Damage"; 0 unless both present; **quality not applied** |
| eDPS | `APS × Σ avg(range)` over every value of "Elemental Damage", type tag ignored |
| DPS | pDPS + eDPS + cDPS |
| sockets_cnt | `sockets.length` (PoE2 sockets counted, contribute no colour) |
| links_cnt | longest run of **consecutive** equal `group` values |
| colours | `attr` S/D/I/G → r/g/b/w; the `sColour` fallback mis-maps (G → w, R/B/W → nothing); `socket_groups[0]` is always an empty group |
| requirements | `values[0][0].toInt()` per name, last wins |
| influences | `influences.{shaper…warlord}`, then `synthesised`, `fractured`, `searing`, `tangled`, in that order |
| flags | crafted = a `flags.crafted` mod exists; enchanted = `enchantMods` non-empty; corrupted, fractured, split, synthesised, mutated = the top-level booleans |
| rarity | `frameType` only; the `rarity` string is never read |

**F3 — The normalizer** (`data/derivations.md`, §4). Arrays read: `enchantMods`, `implicitMods`, `explicitMods`; an entry's `flags` routes it to the crafted, fractured or mutated bucket, `desecrated` and `vestigial` are dropped; `utilityMods`, `runeMods`, `scourgeMods`, `crucibleMods`, `veiledMods`, `bondedMods`, `logbookMods`, `ultimatumMods`, `hybrid.explicitMods` and socketed items are never read. Template: regex `([0-9.]+)` → `#` (sign and hyphen survive: `+#%`, `#-#`); a line starting `1 Added Passive Skill` is kept verbatim. Value: the mean of the line's numbers (`Adds 12 to 34` → 23). A line with no number is in the dropdown but never in a table. Table order is the bucket map's alphabetical order (crafted, enchant, explicit, fractured, implicit, mutated), and a duplicate template **assigns** (last wins), so two fire-resistance lines keep one. Pseudomods **sum** every contributing line, a template listed N times counting N× (`data/pseudomods.toml`, header). The dropdown is the 35 pseudomod names plus every RePoE English translation string, slots substituted. Against the trade site's `pseudo` texts (trade-query F4): 34 of 35 match exactly; the physical-damage one differs by its leading `+`.

**F4 — Category and RePoE** (`data/derivations.md`, §3). `category` = RePoE `base_items[typeLine].item_class` → `item_classes[key].name`, lowercased; a transfigured gem retries with ` of …` stripped; unknown → empty. The choice list is the sorted class names. `m_replace_map` (three levels) is dead code. RePoE is fetched from `https://repoe-fork.github.io` (`version.txt` probe; `item_classes`, `base_items`, `stat_translations`, `stat_translations/necropolis`, all `.min.json`), cached under `<data dir>/repoe/`, nothing bundled: with no network on first run every category is empty and the mod dropdown holds the pseudomods alone.

**F5 — Columns, sorts, views** (`data/columns.toml`). Name · Price · Last Update · Q · Stack · Corr · Mast · Ench · Inf · PD · ED ×3 · CD · APS · DPS · pDPS · eDPS · cDPS · Crit · Ar · Ev · ES · B · Lvl · ilvl. Every column but Price and Date orders by `Column::parts` over the rendered string (a number, else `a-b` → mean, else `a/b` → PrettyName then a, else the string), then (PrettyName, id, hash_v4). Price orders by a fixed currency rank then amount; Date by the buyout's last change. Default: Name descending, never persisted. Views: By Tab buckets on `(location type, id)` — never label or position; Map/Unique children fold into the parent bucket; By Item is one flat bucket. A filtered search hides empty buckets. Inf renders icons only, its text is always empty (the doc's S/E/H/W/C/R is discarded code).

**F6 — What instant cost** (`data/delta-pipeline.md`). Measured at 101,048 and 975,711 items, single-threaded, Apple M4.

| Term | 100k | ~1m |
| --- | --- | --- |
| Pre-M3 reset (refilter + sort + restore) | 422 ms | 5,562 ms |
| — of which the per-bucket sort | 387 ms | 5,163 ms (93 %) |
| — of which the filter loop + bucketing | 34 ms | 391 ms |
| Comparator, per call; the two regex evaluations | ~530 ns; ~⅔–¾ | same |
| By-Item flat sort, real comparator | 1,017 ms | 12,875 ms |
| Sort with materialized keys, sort alone | 8.5 ms | 130 ms |
| Key memory, one column, whole collection | 27.5 MB | 266 MB (~286 B/item) |
| Post-M3 unfiltered refilter, sort share | 23.6 ms, 0 | 282 ms, 0 |
| — bare `FilterItems` inside it | 20.2 ms | 329 ms |
| Delta into a visible By-Tab bucket | 0.87 ms | 0.86 ms |
| By-Item merge per delta, before / after the S5 fix | 168 / 3.9 ms | 1,398 / 32.7 ms |

Paid: per-search dirty flags, the active-filter test per arrival, sorted merge into resident order, lazy rebuilds and staleness marks, resident key vectors with eviction and a byte gauge, sort-on-expand, a visible-by-id index, one authoritative reconciliation per refresh. Accepted, not solved: the O(collection) filter pass, the O(n + d) flat merge, the filter loop. The report's §3 pins the implications: index the filter predicates (post-M3, ~93 % of a refilter is the unindexed scan); define ordering once and materialize keys, budgeted and scoped to what is visible; stable global item ids and container keys are load-bearing; one flat sorted result fights incremental update.

## Numbers

| | |
| --- | --- |
| Filters / groups / payload kinds | 38 / 9 / 6 (minmax 20, bool 11, text 2, combo 2, colors 2, mods 1) |
| Debounced / immediate; the timer | 25 / 13; 350 ms |
| Columns; buyout-dependent | 26; 2 |
| Pseudomods; templates summed; trade-text matches | 35; 117; 34 |
| Mod arrays read / ignored | 3 (6 buckets) / 9 and socketed items |
| Alt-art icon needles | 117 |
| Frame types in the enum / matched by Rarity | 14 / 6 |

## Open questions

- **Q1 — The property strings.** Do "Critical Strike Chance" and "Chance to Block" carry a trailing `%`? Only Quality is stripped; `toDouble` of `6.50%` is 0, which would make those two filters and the sort silently wrong. Master's one fixture and the mock carry no such property. Closes: one weapon and one shield from the item-facts database.
- **Q2 — Duplicate templates.** How often does one item carry the same template twice (last-wins loses a value)? Closes: the item-facts template census, counting repeats per item.
- **Q3 — The ignored arrays.** How many items in a real corpus carry only unsearchable mods (`scourgeMods`, `crucibleMods`, `veiledMods`, `utilityMods`, …)? Closes: the item-facts field census.
- **Q4 — RePoE as a surface.** The taxonomy and the dropdown depend on a third-party feed with no spike access method. Closes: the design session rules an access method into its `SURFACES.md` row, or replaces the source (the trade site's `items.json` categories are a candidate, trade-query Q1).
- **Q5 — Two latent faults**, for the C++ side: `Stack Size` without `/` is unguarded; the pseudo pass reads an uninitialised double if a line's own template fails to match it. Closes: a unit test over `AddModToTable` on master.

## Provenance

Every file under `data/` names its inputs on its first line; all are extracts from `master@946a4f51` and regenerate by re-reading those files at that commit (`data/pseudomods.toml` by its script). Nothing here comes from memory of the app or the game; a rule not in a listed file is marked open. The pseudomod match read `../trade-query/data/stats-2026-09-12.json`.

## Review

| Date | Reviewer | Finding | Disposition |
| --- | --- | --- | --- |
