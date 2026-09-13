# trade-query — the trade site's query language as a spec

Status: first pass complete — 2026-09-13. Static data, the page and the owner's seven searches read; every question closed (Q1 and Q7 by repoe and item-facts), the candidate claims wait for the close.

Headline:
- The site sends `{query, sort}` to `POST /api/trade/search[/<realm>]/<league>`, pc by omission. `query` is `status`, then `term` *or* `name`/`type`, `filters` keyed by group, and `stats`: groups of eight types (`and`, `not`, `if`, `count`, `weight`, `weight2`, `crucible`, `mercenary`) whose semantics GGG states in tips, captured verbatim; the owner's captures pin every shape (F1, F2).
- 93 filters in 12 groups: 59 read something a private-API item carries — the renderer maps 70 property types to filter keys — 7 read the market, 27 are open, the 83-id category taxonomy first (F3). For weapons and armour the `fetch` response carries the computed `dps`/`pdps`/`edps`/`ar`/`ev`/`es` values, so the formulas are checkable (F7).
- The stat vocabulary is 14,193 keys in 14 categories; the key is the numeric stat id, the category the mod's source. Display text does not determine the key: 380 (category, text) collisions, 93 `(Local)` twins (F4). For the one collision tested, one id is live and the other has no listings (F8).
- **The owner's hypothesis is confirmed in the data:** one displayed line is fed by several mods (a hybrid prefix plus a pure one → one `+186 to maximum Life`), and one mod feeds several lines; a pseudo total sums the *displayed* values of its contributing lines, exactly the C++ pseudomod table (F8).
- **GGG changed the item format, not only the site.** `fetch` lines are objects with `description`, `domain`, `hash` and the contributing `mods` (name, tier, level, magnitudes). The private stash API, as the spike's store holds it since 2026-09-02, returns `explicitMods`/`implicitMods` as objects too — `{description, flags?}`, no hash — while other arrays stay strings (F7; the census is item-facts F3).

## Question

What is the trade site's query language and what is observable about its semantics; then which filters mean something for a private stash, which do not, and what a stash search adds.

## Inputs

| Source | Access | Status |
| --- | --- | --- |
| `/api/trade/data/{stats,items,static,filters}` | `browser`, owner, 2026-09-12; committed dated under `data/` | landed |
| The search page with its bundles | `browser`, owner, 2026-09-12; `raw/`, local | landed |
| Seven searches: request, response, first `fetch` (`raw/searches/`) | `browser` network tab, owner, 2026-09-13 | landed; q2b had no results so no fetch |
| The spike's own facts store (`GERWARIC_7694.db`, 22,727 items seen 2026-09-02..11) | read-only, for the private-API format only | read |

The site is a governed surface (C79; `SURFACES.md`). Every file, hash and the owner's note on the searches: `MANIFEST.md`.

## Findings

**F1 — The request** (`data/grammar.json`; pinned by the captured bodies in `data/fetch-census.json`, `queries.*.request`).

| Element | Shape |
| --- | --- |
| URL | `POST /api/trade/search` + (`/<realm>` unless `pc`) + `/<league>`; body `{query, sort}`; default sort `{"price": "asc"}` |
| Response | `{id, complexity, result: [ids], total, inexact?}`; 100 ids per page, `total` capped at 10,000; the page then fetches ten |
| `query.status` | `{option}`: `available \| securable \| onlineleague \| online \| any` |
| `query.term` / `name` / `type` | free text, or a string / `{option, discriminator}` each |
| `query.filters` | `{<group>: {disabled, filters: {<id>: {min?, max?} \| {option} \| {r?,g?,b?,w?,min?,max?} \| text}}}`; a disabled group is still sent (q2b–q6) |
| `query.stats` | `[{type, filters: [{id, value?: {min?, max?, weight?, option?}, disabled?}], value?: {min?, max?}}]`; an empty `and` group is sent as `{type: "and", filters: []}` (q4) |
| Limits | 100 results per search, 500 per live search |

**F2 — Stat group types**, tips verbatim (`grammar.json`, `stat_groups`).

| Type | Group value | Tip |
| --- | --- | --- |
| `and`, `not` | — | — |
| `if` | — | "Match items that meet each stat's `min` and `max` requirements if the stat is present." |
| `count` | min, max | "Count each stat that meets the `min` and `max` (if provided, otherwise existence) requirements. Use the group's `min` and `max` to filter items based on the count of matching stats." |
| `weight` | min, max; per-stat `weight` | "Check each stat meets the `min` and `max` (if provided, otherwise existence) requirements before multiplying the stat value by the `weight` and finally summing them together. …" |
| `weight2` | min, max; per-stat `weight` | "Each stat value that meets the `min` and `max` (if provided, otherwise existence) requirements will be multiplied by the `weight` before being summed together. …" |
| `crucible`, `mercenary` | min only; not mutable | "…Use a lower `min` value for partial matches." |

**F3 — Filter groups and the gap** (`data/gap.csv`, rule table in `scripts/gap.py`). `local` = a property or field a private-API item carries or a derivation of them; `none` = market; `open` = not settled here.

| Group | Filters | local / none / open | Open items |
| --- | --- | --- | --- |
| `status_filters`, `trade_filters` | 1, 6 | 0 / 7 / 0 | |
| `type_filters` | 2 | 1 / 0 / 1 | `category` (83 ids) |
| `weapon_filters`, `socket_filters` | 6, 2 | 8 / 0 / 0 | `dps` family and links derived (F7) |
| `armour_filters` | 6 | 5 / 0 / 1 | `base_defence_percentile` (the `fetch` carries it) |
| `req_filters` | 5 | 4 / 0 / 1 | `class` |
| `map_filters` | 12 | 6 / 0 / 6 | series, blighted ×2, chart ×2, completion reward |
| `heist_filters`, `sanctum_filters` | 16, 4 | 16 / 0 / 4 | the `max_*` totals |
| `ultimatum_filters` | 4 | 0 / 0 / 4 | all |
| `misc_filters` | 29 | 19 / 0 / 10 | transfigured, imbued, foreseeing, vestigial, intangibility, alt art, corpse type, scourge tier, crucible, mutated |

What a stash search **adds**: tab and character, container, realm and league, liveness, first/last seen, socketed-in, and our listing state (the analogue of `sale_type`).

**F4 — The stat vocabulary** (`data/stat-summary.csv`, `data/stat-collisions.csv`). 14 categories, 18,187 entries, 14,193 keys, 13,975 texts. `explicit` 7,896 entries (2,282 texts with no `#`, 5,419 with one, 195 with two or more; 459 `indexable_*` ids); `implicit` 1,834; `fractured` 1,833; `enchant` 2,037; `crucible` 2,492 multi-line tiered texts with values baked in; `pseudo` 298 (245 `pseudo_*`, 53 `lake_*`, 88 with option lists). A key appears in one category for 11,694 keys and in several for 2,499. Collisions: 380 pairs; three kinds — two stats with one rendering (`+#% chance to Suppress Spell Damage`, the Mana Reservation Efficiency and aura `Grants Level #` families), `stat_N` beside `indexable_support_N` (the "Socketed Gems are Supported by Level #" family), and `stat_N|a|b` variants. The C++ pseudomods are all present under `pseudo_total_*`; the site adds `pseudo_adds_*`, affix-count pseudos, influence flags, quality pseudos and 206 enumerations.

**F5 — The bridge to the item JSON** (`grammar.json`, `property_type_to_field`): 70 `properties[].type` ids map to filter fields (1–4 map tier/iiq/iir/pack size, 5 gem level, 6 quality, 9–13 damage/crit/aps, 15–18 block/ar/ev/es, 32 stack size, 62–65 level/str/dex/int, 78 ilvl, …). Renderer mod arrays in order: `utility, enchant, rune, scourge, implicit, fractured, mutated, explicit, bonded, crafted, desecrated, pseudo, cosmetic, crucible`. Rarity is `frameType` (0 normal … 3 unique, 9 foil, 10 supporter foil).

**F6 — The page.** Realms `pc`, `xbox`, `sony`, eight leagues each, no `poe2` here (T4). `items.json`: 22 categories, 3,733 base types, 1,546 uniques, 725 `disc` variants; its `map` group lists no area base type — 5 map types, 282 blighted ids, 96 discriminated variants, 305 other bases (item-facts F7). No fetched item carries a class field; the class name is in `extended.text` (F9), and the export's classes generate 65 of the site's 68 leaf category ids (repoe F3). `static.json`: the 23 exchange groups (T33).

**F7 — The `fetch` item and the private item** (`data/fetch-census.json`; 70 items, 443 lines). A fetched item's mod arrays hold objects: `{description, domain, hash, mods: [{name?, tier?, level?, magnitudes: [{min, max}]}], flags?}`; crafted and fractured lines sit inside `explicitMods` under `domain` and `flags` — no `craftedMods` array; `pseudoMods` carry computed descriptions with no contributing list; `extended.hashes[<category>]` is `[[hash, [mod indexes]]]` into a mod list this response no longer carries, and `extended.mods` is gone. `extended` also carries the computed filter values: `dps`, `pdps`, `edps` with `_aug` flags on weapons; `ar`/`ev`/`es`/`ward` and `base_defence_percentile` on armour. `pdps` checks as average physical × attacks per second × 1.2 (the 20 % quality of the filter tip) to within rounding; `edps` exactly. The item carries `rarity` and `frameTypeId` beside `frameType`, and `properties[].type`. **The private API** (the spike's store): `explicitMods`, `implicitMods` are `{description, flags?}` objects on every item seen 2026-09-02..11 (18,383 items, 71,695 lines, both realms), never with a `hash`; `enchantMods`, `utilityMods`, `veiledMods`, `crucibleMods` stay strings; `rarity` and `frameTypeId` present. The API reference, read 2026-09-13, documents the object as `ItemMod`; the switch landed between 2026-07-23 and 2026-07-31 on those two arrays only, never with a hash (item-facts F3). `extended.hashes` indexes point into a mod list this response no longer carries; the same facts sit in each line's `mods`.

**F8 — What the searches closed.**

| Q | Capture | Result |
| --- | --- | --- |
| Q2 collision | q2 `stat_3680664274`: 10,000 results, every Suppress line carries that id (explicit or crafted); q2b `stat_492027537`: **0 results**. Owner, 2026-09-13: "I confirmed that the second modifier returns no results. I tested this across every poe1 league possible." | For this line one id is live, the other dormant everywhere; the picker offers both. Not generalized to the other 379. |
| Q3 two-value line | q3, min 20 on `Adds # to # Physical Damage (Local)`: lows 15–27 admitted, averages all ≥ 20; q3b, max 25: `Adds 17 to 30` (high 30) admitted, averages all ≤ 25 | **The value is the average of the two numbers**: the low is ruled out by q3, the high by q3b. The owner's expectation, stated before q3b, verbatim: "I believe the trade site uses the 'min' stat value as an average". |
| Q4 groups | q4 request: `weight` group `value: {min: 60}`, per-stat `value: {weight: 1}`; `count` group `value: {min: 2}`; response `inexact: true`, `complexity: 85` | Shapes pinned. |
| Q5 pseudo | `pseudo_total_fire_resistance` min 80: every item's total equals the sum of the displayed fire, all-elemental, fire-and-X lines, explicit and crafted; a Simplex Amulet's `+60% to Fire Resistance` (mod range 46–48, scaled by its implicit) counts as 60 | The displayed value is summed; the contributor set matches the C++ table. |
| Q6 hypothesis | 11 lines fed by two mods (life from a hybrid prefix plus a pure one; evasion; stun recovery); 31 mods feeding two lines each | **Confirmed**: several stats add into one displayed line, and a line's value cannot name its mods without the `mods` list. |

**F9 — Two more facts from the captures** (checked 2026-09-13). The search `id` in the response and in the site's URL is the query object itself, gzipped and base64url-encoded: q4's id decodes to exactly its request's `query`. A search URL is therefore a portable query, and any saved search on the site can be read without the site. And `extended.text` on a fetched item is the item's clipboard text, base64: its first line is `Item Class: <class>` for all 70 items (six classes seen), so listed items carry their class name there even though no JSON field does — a partial source for Q1, for listed items only.

## Open questions

None open.

Candidate claims for `trade-ground-truth.md`: pc-by-omission in the URL (C58's rule); the eight group semantics verbatim; result limits 100/500 and the 10,000 total cap; the `fetch` line object and its `mods`; the private API's `{description, flags}` lines (dated by the store).

## Provenance

`MANIFEST.md`. Generated: `scripts/extract-grammar.py` → `grammar.json`; `scripts/stat-collisions.py` → `stat-summary.csv`, `stat-collisions.csv`; `scripts/gap.py` → `gap.csv`; `scripts/fetch-census.py` → `fetch-census.json` (scrubbed; a guard refuses seller data). Nothing here comes from memory of the site; a fact not in a listed file is marked open.

## Review

| Date | Reviewer | Finding | Disposition |
| --- | --- | --- | --- |
