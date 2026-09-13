<!-- extracted by hand (no script) from master@946a4f51: src/item.cpp, item.h, modlist.cpp, pseudomods.cpp, itemcategories.cpp, repoe/repoe.cpp, util/util.cpp, poe/types/item.h; regenerate by re-reading those files at the commit -->

# Acquisition (C++/Qt) — item JSON → searchable values

All facts read at commit `946a4f51`, branch `master`. Files: `src/item.h`,
`src/item.cpp`, `src/modlist.cpp`, `src/itemcategories.cpp`,
`src/repoe/repoe.cpp`, `src/util/util.cpp`, `src/poe/types/*.h`.
"JSON" paths are fields of `poe::Item` (`src/poe/types/item.h`), which is the
`glz`-parsed GGG wire object; the constructor is the only place JSON is read.

---

## 1. Accessor → JSON fields

`Item` is built once from `(const poe::Item&, const ItemLocation&)`
(`item.cpp:103-183`); every accessor is a getter over a member fixed there.

| Accessor (`item.h`) | JSON field(s) | Derivation | Value when source absent | Cite |
| --- | --- | --- | --- | --- |
| `id()` | `id` (`?string`) | copied verbatim | `""` (default `QString`) | item.cpp:161-163 |
| `name()` | `name` (`string`) | `fixup_name`: if `">>"` occurs, keep everything after its **last** occurrence (strips `<<set:X>>` prefixes); else verbatim | `""` | item.cpp:105, 56-64 |
| `typeLine()` | `hybrid.isVaalGem`, `hybrid.baseTypeName`, `typeLine` | if `hybrid` present and `hybrid.isVaalGem != true` → `hybrid.baseTypeName`; otherwise `typeLine`. Then `fixup_name` | `""` | item.cpp:108-120 |
| `PrettyName()` | as `name`+`typeLine` | `name.isEmpty() ? typeLine : name + " " + typeLine` | `typeLine` alone | item.cpp:185-192 |
| `identified()` | `identified` (`bool`, required) | copied | member default `true` | item.cpp:122 |
| `corrupted()` | `corrupted` (`?bool`) | `*item.corrupted` if the optional is engaged | `false` | item.cpp:124-126 |
| `crafted()` | **not a JSON field** — `craftedMods` bucket non-empty | `!craftedMods.empty()`, where `craftedMods` collects `implicitMods[]`/`explicitMods[]` entries whose `flags.crafted` is true | `false` | item.cpp:422 |
| `enchanted()` | `enchantMods` | `enchantMods` present **and** non-empty | `false` | item.cpp:421 |
| `fractured()` | `fractured` (`?bool`) | `*item.fractured` | `false` | item.cpp:127-129 |
| `split()` | `split` (`?bool`) | `*item.split` | `false` | item.cpp:130-132 |
| `synthesized()` | `synthesised` (`?bool`) | `*item.synthesised` | `false` | item.cpp:133-135 |
| `mutated()` | `mutated` (`?bool`) | `*item.mutated` (note: **not** the per-mod `flags.mutated`) | `false` | item.cpp:136-138 |
| `hasInfluence(t)` / `hasInfluence()` / `influenceLeft()` / `influenceRight()` | `influences.{shaper,elder,crusader,redeemer,hunter,warlord}`, `synthesised`, `fractured`, `searing`, `tangled` | linear search / `[0]` / `[0]`-or-`[1]` over `m_influenceList` (see §2) | `NONE` / `false` | item.h:92-107, item.cpp:194-228 |
| `w()`, `h()` | `w`, `h` (`uint`, required) | copied to `int` | 0 | item.cpp:142-143 |
| `frameType()` | `frameType` (`?uint as FrameType`) | `static_cast<int>(*item.frameType)` | **`-1`** | item.cpp:144-146 |
| `icon()` | `icon` (`string`) | copied, then `replace("quad=1","quad=0")` and `replace("scaleIndex=","scaleIndex=0&")` | `""` | item.cpp:148, 154-157 |
| `properties()` | `properties[].name` → `properties[].values[0][0]` | `map<name, first value string>`; `"Elemental Damage"` is **excluded**; `"Quality"`/`"Level"`/`"Stack Size"` are post-processed (§2) | empty map | item.cpp:168-170, 440-474 |
| `text_properties()` | whole `properties[]` array | one `ItemProperty{name, all values (str,type), display_mode}` per entry, in JSON order; includes `"Elemental Damage"` | empty vector | item.cpp:476-483 |
| `text_requirements()` | `requirements[].name`, `requirements[].values[0]` | one `{name, {str, type}}` per entry with ≥1 value, in JSON order | empty vector | item.cpp:487-499 |
| `text_mods()` | `enchantMods[]`, `implicitMods[].description`, `explicitMods[].description` + their `flags` | six fixed keys (§4); always all six present after construction | six keys, each an empty vector | item.cpp:424-429 |
| `text_sockets()` | `sockets[].group`, `sockets[].attr` ∥ `sockets[].sColour` | one `{group, attr}` per socket whose first attr/sColour character is non-NUL, in JSON order | empty vector | item.cpp:507-521 |
| `hash_v4()` | see §5 | MD5 over name/typeLine/mods/properties/sockets/legacy location | MD5 of the degenerate string | item.cpp:625-665 |
| `old_hash()` | see §5 | same with a `<<set:MS>><<set:M>><<set:S>>` prefix | — | item.cpp:627, 663 |
| `elemental_damage()` | `properties[].name == "Elemental Damage"` → all `values[i]` | `vector<pair<value-string, type-int>>`; `type` is `values[i][1]` (`ED_FIRE=4`, `ED_COLD=5`, `ED_LIGHTNING=6`) | empty vector | item.cpp:444-449, itemconstants.h:34-38 |
| `requirements()` | `requirements[].name` → `values[0][0]` | `map<name, str.toInt()>`; non-numeric → 0 | empty map | item.cpp:496 |
| `DPS()`, `pDPS()`, `eDPS()`, `cDPS()` | derived from `properties()` + `elemental_damage()` | computed on every call, not cached (§2) | 0 | item.cpp:571-623 |
| `sockets_cnt()` | `sockets[]` | `sockets.size()` — counts sockets that were skipped for having no attr/sColour | 0 | item.cpp:504 |
| `links_cnt()` | `sockets[].group` | max run length of consecutive equal `group` values (§2) | 0 | item.cpp:505-529 |
| `sockets()` | `sockets[].attr`/`sColour` | `{r,g,b,w}` counts from the `S`/`D`/`I`/`G` switch (§2) | `{0,0,0,0}` | item.cpp:530-547 |
| `socket_groups()` | `sockets[].group` + colours | per-group `{r,g,b,w}`, **with a spurious leading empty group** (§2) | `[{0,0,0,0}]` if `sockets` present, else empty | item.cpp:522-525, 549 |
| `location()` | `x`, `y`, `w`, `h`, `inventoryId`, `socket` folded onto the tab/character location | `base_location.getItemLocation(item)`: overwrites x/y/inventory-id when present, always sets w/h, sets `m_socketed` when `socket` present | the bare tab/character location | item.cpp:106, itemlocation.cpp:39-57 |
| `note()` | `note` (`?string`) | copied verbatim (this is the buyout-bearing text) | `""` | item.cpp:164-166 |
| `category()` | **RePoE**, keyed on `baseType` | §3 | `""` | item.cpp:159, 552-569 |
| `count()` | `properties[].name == "Stack Size"` → `values[0][0]` | text before the first `"/"`, `toInt()`; **`stackSize`/`maxStackSize` JSON fields are never read** | **1** (member default) | item.cpp:469-471, item.h:180 |
| `mod_table()` | `text_mods()` values | `map<template, double>` built by `AddModToTable` (§4) | empty map | item.cpp:431-435 |
| `ilvl()` | `ilvl` (`int`, required) | copied. The optional `itemLevel` field is **never read** | 0 (value-init) | item.cpp:182 |
| `operator<` | — | §5 | — | item.cpp:667-673 |
| `Wearable()` | `category()` | §2 | `false` | item.cpp:674-680 |
| `POBformat()` | `frameType`, name, typeLine, uid, ilvl, `Quality`, sockets, `Level` requirement, mod buckets, corrupted | Path-of-Building paste text; not a search input | — | item.cpp:681-783 |
| `RebaseLocation()` | — | mutator; copies tab colour/id/type/label/character from a fresh tab location; deliberately does **not** recompute `hash_v4()` | — | item.h:133-137 |

Not read anywhere in `Item`: `stackSize`, `maxStackSize`, `stackSizeText`,
`itemLevel`, `rarity`, `league`, `realm`, `verified`, `utilityMods`,
`runeMods`, `scourgeMods`, `crucibleMods`, `veiledMods`, `bondedMods`,
`cosmeticMods`, `logbookMods`, `notableProperties`, `nextLevelRequirements`,
`grantedSkills`, `socketedItems`, `extended`, `descrText`, `flavourText`,
`gemSockets`, `gemTabs`, `abyssJewel`, `delve`, `duplicated`, `replica`,
`isRelic`, `foilVariation`, `memoryItem`, `vestigial` (item-level),
`desecrated` (item-level), `doubleCorrupted`, `sanctified`, `colour`,
`socketedIcon`, `frameTypeId` (stored in `m_frameTypeId`, no accessor).
`additionalProperties` is read only by `CalculateHash`.

---

## 2. Derivation formulas

| Value | Exact formula | Cite |
| --- | --- | --- |
| `DPS()` | `pDPS() + eDPS() + cDPS()` | item.cpp:571-574 |
| `pDPS()` | `0` unless both `properties["Physical Damage"]` and `properties["Attacks per Second"]` exist; else `APS × AverageDamage(phys)` where `APS = QString::toDouble` of the APS string and `AverageDamage(s)` = split `s` on `"-"`; if <2 parts → `s.toDouble()`, else `(parts[0]+parts[1])/2`. **Quality is not applied**; the property strings are used exactly as GGG sent them | item.cpp:576-589, util.cpp:39-47 |
| `eDPS()` | `0` if `m_elemental_damage` empty or no APS property; else `APS × Σ_i AverageDamage(m_elemental_damage[i].first)` — sums **all** values of the `"Elemental Damage"` property, ignoring their type tag | item.cpp:591-606 |
| `cDPS()` | `0` unless `properties["Chaos Damage"]` and APS both exist; else `APS × AverageDamage(chaos)` | item.cpp:608-623 |
| `elemental_damage` | for the property named exactly `"Elemental Damage"`, `emplace_back(values[i][0], values[i][1])` for every value — the raw range string plus the integer type. This property is the one property **not** copied into `m_properties` | item.cpp:444-449 |
| `sockets_cnt` | `static_cast<int>(sockets.size())` — set before the loop, so PoE2 sockets (which carry `type`/`item`, not `attr`/`sColour`) are counted even though they contribute no colour and no link | item.cpp:504, itemsocket.h:15-23 |
| `links_cnt` | walk `sockets[]` in JSON order; `counter` resets to 0 whenever `socket.group` differs from the previous kept socket's group, then `++counter`, `links_cnt = max(links_cnt, counter)`. So it is the longest run of **consecutive** equal `group` values; a group split across non-adjacent entries would be undercounted | item.cpp:505-529 |
| `sockets` (r/g/b/w) | `attr = socket.attr ? attr[0] : (socket.sColour ? sColour[0] : '\0')`; a NUL first char skips the socket entirely. Then `'S'→r++`, `'D'→g++`, `'I'→b++`, `'G'→w++`; any other char (`'A'`, `'D'` from `"DV"`, and every `sColour` letter except `G`) falls through the switch and counts toward nothing. **Because `sColour` uses R/G/B/W, the fallback path mis-maps: `G`(green) increments `w`, and R/B/W increment nothing** | item.cpp:508-547 |
| `socket_groups` | on each group change the *previous* `current_group` is pushed and a fresh `{0,0,0,0}` started; the final group is pushed after the loop. Since `prev_group` starts at `-1`, the first socket always pushes the initial empty group → **`socket_groups[0]` is always `{0,0,0,0}`** and the real groups are `[1..]` | item.cpp:503, 522-525, 549 |
| `requirements` | for each `requirements[]` entry with ≥1 value: `m_requirements[name] = values[0][0].toInt()`. Duplicate names: **last wins** (map assignment). Non-numeric text → 0. `weaponRequirements`/`supportGemRequirements` (PoE2) are not read | item.cpp:487-499 |
| `ilvl` | `m_ilvl = item.ilvl` (required `int` on the wire), assigned last in the constructor | item.cpp:182 |
| `count` (stack) | `properties["Stack Size"]` first value, e.g. `"12/20"` → `n = indexOf("/")`, `m_count = strval.first(n).toInt()`. Default 1 when no such property. `open:` if a `"Stack Size"` value ever lacks a `/`, `indexOf` returns `-1` and `QString::first(-1)` is a precondition violation (Qt asserts / UB) — nothing guards it | item.cpp:469-471 |
| `properties` map | `map<QString,QString>`: key = `properties[].name` verbatim, value = **`values[0][0]` only** (later values dropped, the per-value `type` dropped, `display_mode` dropped). Entries with zero values are skipped. `"Elemental Damage"` never enters the map. Post-processing: `"Quality"` strips a leading `"+"` and a trailing `"%"` (`"+23%"`→`"23"`); `"Level"` chops the last 6 chars when it ends in `"(Max)"` (i.e. `" (Max)"`); `"Stack Size"` also sets `count`. Duplicate names: last wins | item.cpp:440-474 |
| `text_properties` | every `properties[]` entry, all values, `display_mode = displayMode.value_or(InsertedValues /*3*/)`; enum: 0 NameFirst, 1 ValuesFirst, 2 ProgressBar, 3 InsertedValues, 4 Separator | item.cpp:476-483, displaymode.h |
| `PrettyName` | `m_name.isEmpty() ? m_typeLine : m_name + " " + m_typeLine` (both already `fixup_name`-d) | item.cpp:185-192 |
| `Wearable` | `category == "flasks" \|\| category == "amulet" \|\| category == "ring" \|\| category == "belt" \|\| category.contains("armour") \|\| category.contains("weapons") \|\| category.contains("jewels")` — case-sensitive `contains` against the lowercased category | item.cpp:674-680 |
| `frameType` / rarity | `m_frameType = static_cast<int>(*item.frameType)` else `-1`. The `rarity` JSON string is never read; rarity is inferred from `frameType` at the filter (`0` Normal, `1` Magic, `2` Rare, `3` Unique, `9`/`10` foils) | item.cpp:144-146, frametype.h, filtermatchers.cpp:174-182 |
| influence list | pushed in this fixed order, one entry per true flag: `influences.shaper`→`SHAPER`, `influences.elder`→`ELDER`, `influences.crusader`→`CRUSADER`, `influences.redeemer`→`REDEEMER`, `influences.hunter`→`HUNTER`, `influences.warlord`→`WARLORD`, then top-level `synthesised`→`SYNTHESISED`, `fractured`→`FRACTURED`, **`searing`→`SEARING_EXARCH`**, **`tangled`→`EATER_OF_WORLDS`**. Top-level `elder`/`shaper` (the legacy duplicates) are **not** read — only `influences.*`. `influenceLeft/Right` are just `[0]`/`[1]` of this order, so the list order *is* the display order | item.cpp:194-228, item.h:64-76 |
| `identified` | `item.identified` (required `bool`); member default `true` | item.cpp:122 |
| `corrupted` | `item.corrupted` | item.cpp:124-126 |
| `crafted` | `!craftedMods.empty()` — i.e. at least one `implicitMods[]`/`explicitMods[]` entry with `flags.crafted == true` (after the flag-precedence cascade, §4) | item.cpp:422 |
| `enchanted` | `item.enchantMods && !item.enchantMods->empty()` — array non-emptiness, no flag involved | item.cpp:421 |
| `fractured` | top-level `item.fractured` only. Note the *bucket* `fracturedMods` is populated from `flags.fractured`, independently of this boolean | item.cpp:127-129 |
| `split` | `item.split` | item.cpp:130-132 |
| `synthesized` | `item.synthesised` | item.cpp:133-135 |
| `mutated` | top-level `item.mutated` only (not `flags.mutated`) | item.cpp:136-138 |

---

## 3. Category

`m_category` is a **lowercased RePoE item-class name**, looked up from the
item's `baseType`. There is no local hierarchy logic: `CalculateCategories`
is two lookups, and the `m_replace_map` table is dead code.

| Step | Behaviour | Cite |
| --- | --- | --- |
| 1 | `m_category = GetItemCategory(m_baseType)`; return if non-empty. `m_baseType` is `fixup_name(item.baseType)` | item.cpp:554-557, 121 |
| 2 | else, if `m_baseType` contains `" of "` (transfigured skill gems), retry with `m_baseType.first(indexOf(" of "))` | item.cpp:561-568 |
| 3 | `GetItemCategory(bt)`: if either RePoE map is empty → log error, return `""`. Else `m_itemBaseTypeToClass[bt]` → class **key**; then `m_itemClassKeyToValue[key]` → class **name**; return `name.toLower()`. Any miss → log at trace, return `""` | itemcategories.cpp:115-140 |
| Unknown base type | `m_category` stays the empty string. No exception, no fallback category, no "unknown" sentinel; the item is simply uncategorised (and therefore `Wearable() == false`) | item.cpp:552-569 |

`m_replace_map` — `static const std::array<CategoryReplaceMap,3>`, defined at
`item.cpp:22-37`, declared at `item.h:146-147`. **Nothing in `src/` reads it**
(`git grep m_replace_map master -- src` returns only the definition and the
declaration): it is a leftover from a former three-level category hierarchy.
Verbatim, level by level:

| Level | Replacement pairs (verbatim) |
| --- | --- |
| 0 | `"Divination" → "Divination Cards"`, `"QuestItems" → "Quest Items"` |
| 1 | `"BodyArmours" → "Body"`, `"VaalGems" → "Vaal"`, `"AtlasMaps" → "2.4"`, `"act4maps" → "2.0"`, `"OneHandWeapons" → "1Hand"`, `"TwoHandWeapons" → "2Hand"` |
| 2 | `"OneHandAxes" → "Axes"`, `"OneHandMaces" → "Maces"`, `"OneHandSwords" → "Swords"`, `"TwoHandAxes" → "Axes"`, `"TwoHandMaces" → "Maces"`, `"TwoHandSwords" → "Swords"` |

### RePoE at runtime

Fetched over HTTP from a fork's GitHub Pages site and cached on disk; parsed
at startup, before items are loaded.

| Thing | Verbatim | Cite |
| --- | --- | --- |
| Host | `constexpr const char *REPOE_URL = "https://repoe-fork.github.io"` | repoe.cpp:19 |
| Version probe | `REPOE_URL + "/version.txt"` | repoe.cpp:45 |
| Class/base files | `REPOE_FILES = {"item_classes.min.json", "base_items.min.json"}` | repoe.cpp:21 |
| Stat-translation files | `STAT_TRANSLATIONS = {"stat_translations.min.json", "stat_translations/necropolis.min.json"}` | repoe.cpp:23-24 |
| Download URL | `REPOE_URL + "/" + filename` | repoe.cpp:149 |
| Cache path | `<data_dir>/repoe/<filename>` (dirs `repoe/` and `repoe/stat_translations/` created on demand); local version in `<data_dir>/repoe/version.txt` | repoe.cpp:73-78, 89, 172 |
| Update rule | re-download **all** files if any cached file is missing **or** remote `version.txt` != local; otherwise use the cache as-is | repoe.cpp:81-121 |
| Load order | `InitItemClasses(item_classes.min.json)` → `InitItemBaseTypes(base_items.min.json)` → `InitStatTranslations()` → `AddStatTranslations()` per stat file → `InitModList()`; then `finished()` unblocks `ItemsManagerWorker` | repoe.cpp:191-207, application.cpp:326-332 |
| Nothing bundled | no Qt resource or asset path for RePoE data anywhere in `src/`; first run with no network leaves both maps empty and every `category()` empty | repoe.cpp (whole file) |

`InitItemClasses` parses `item_classes.min.json` as
`unordered_map<class_key, repoe::ItemClass{category, category_id, name, influence_tags}>`;
entries with an empty `name` are skipped; builds key→name and name→key, plus a
sorted unique `QStringList categories` of the **un-lowercased** names
(itemcategories.cpp:34-73). `InitItemBaseTypes` parses `base_items.min.json`
as `unordered_map<item_key, repoe::BaseItem{item_class, name, release_state}>`,
skipping `release_state == "unreleased"` and names that are empty or start with
`"[DO NOT USE]"`, `"[UNUSED]"`, `"[DNT"`; maps `name → item_class`
(itemcategories.cpp:76-113). Both are permissive (`error_on_unknown_keys =
false`) and a parse error logs and leaves the maps untouched.

Note (outside the derivation but the same pipeline): the Category filter
offers `GetItemCategories()` — the un-lowercased class names — and matches with
`item.category().contains(state.value)`, a case-sensitive `contains` against
the lowercased category (filterspec.cpp:114-120, filtermatchers.cpp:172-173).

---

## 4. The mod normalizer

### 4a. Which arrays are read, in what order

| # | Source JSON | How | Cite |
| --- | --- | --- | --- |
| 1 | `enchantMods[]` (`array of string`) | copied verbatim into the `enchantMods` bucket; no flags exist on this array | item.cpp:240-245 |
| 2 | `implicitMods[]` (`array of ItemMod{description, flags}`) | bucketed by flag (below) | item.cpp:247-336 |
| 3 | `explicitMods[]` (`array of ItemMod{description, flags}`) | bucketed by flag (below) | item.cpp:338-419 |

**Ignored entirely** by `LoadModifiers`: `utilityMods`, `runeMods` (PoE2),
`scourgeMods`, `crucibleMods`, `cosmeticMods`, `veiledMods`, `bondedMods`
(PoE2), `logbookMods`, `ultimatumMods`, `mercenarySkills`,
`hybrid.explicitMods`, and the mods of `socketedItems`. None of them reach
`m_text_mods` or `m_mod_table`, so none is searchable by mod.

Flag cascade, applied per mod, **first match wins** (`flags` absent or
`flags.any() == false` → the unflagged bucket):

| `flags` | from `implicitMods` → bucket | from `explicitMods` → bucket |
| --- | --- | --- |
| none / `[]` | `implicitMods` | `explicitMods` |
| `fractured` | `fracturedMods` (+ warn "fractured") | `fracturedMods` |
| `mutated` | `mutatedMods` (+ warn) | `mutatedMods` |
| `crafted` | `craftedMods` (+ warn) | `craftedMods` |
| `desecrated` | **dropped** (+ warn) | **dropped** |
| `vestigial` | `vestigialMods` — a **local variable that is never stored**, so these mods are silently dropped too | **dropped** (+ warn "vestigial") |
| flagged but no recognised flag | `implicitMods` (+ warn) | `explicitMods` (+ warn) |

Six buckets are then written into `m_text_mods`, a
`std::map<QString, ItemMods>`: `enchantMods`, `implicitMods`, `fracturedMods`,
`explicitMods`, `craftedMods`, `mutatedMods` (item.cpp:424-429). The mod table
is built by iterating that **`std::map`**, so the order is alphabetical by key,
not the insertion order above:

`craftedMods` → `enchantMods` → `explicitMods` → `fracturedMods` → `implicitMods` → `mutatedMods`

and within each bucket, JSON order (item.cpp:431-435). That order is what
decides duplicate-template outcomes below.

### 4b. Display line → template

| Rule | Behaviour | Cite |
| --- | --- | --- |
| Escape hatch | if the line starts with `"1 Added Passive Skill"` (cluster-jewel notables) the line is used **unmodified** as the lookup key — no digits are replaced | modlist.cpp:355-361 |
| Number replacement | `static const QRegularExpression rep("([0-9\\.]+)")` then `generic_mod.replace(rep, "#")` — every maximal run of ASCII digits and `.` becomes a single `#` | modlist.cpp:358-360 |
| Placeholder | `#` |  modlist.cpp:360 |
| What counts as a number | **only** `[0-9.]`. A sign is *not* part of it: `"+23%"` → `"+#%"`, `"-5"` → `"-#"`. A decimal is one run: `"1.5"` → `"#"`. A range is **two** placeholders with the hyphen kept: `"12-34"` → `"#-#"`. `"20%"` → `"#%"`. A trailing/leading `.` joins the run: `"1."` → `"#"` |  modlist.cpp:358-360 |
| Two or more numbers | the template keeps one `#` per run, and the stored value is the **arithmetic mean** of all of them: `Util::MatchMod` walks the template, consumes a `[0-9.]` run at each `#`, `result += strtod(...)`, `++cnt`, and finally `*output = result / cnt`. So `"Adds 12 to 34 Physical Damage"` against `"Adds # to # Physical Damage"` stores `(12+34)/2 = 23` | util.cpp:170-194, modlist.cpp:343-349 |
| No numbers | `Util::MatchMod` never enters the `#` branch, so `found` stays false, `Match` returns false, and `Generate` stores **nothing** — a flat mod such as `"Hits can't be Evaded"` is present in `mod_list_model` but can never be in an item's `mod_table`, so a mods-filter row naming it matches no item | modlist.cpp:329-349, filtermatchers.cpp:260-263 |
| Non-match | `MatchMod` returns false on the first literal mismatch, and true only if both strings are exhausted together — so the template must match the whole line, anchored at both ends | util.cpp:185-193 |
| Same template twice on one item | **natural mods: last wins.** `SumModGenerator::Generate` does `output[m_name] = result` — a plain assignment — so the last line processed in the §4a order overwrites earlier ones (two `+12%`/`+30%` fire-res lines leave 30, not 42). **Pseudo-mods: summed** (below) | modlist.cpp:343-349 |

### 4c. `InitStatTranslations` / `AddStatTranslations`

| Step | Behaviour | Cite |
| --- | --- | --- |
| Input | the RePoE `stat_translations.min.json` and `stat_translations/necropolis.min.json` bytes read from the disk cache (§3) | repoe.cpp:23-24, 199-202 |
| `InitStatTranslations()` | clears the namespace-static `std::set<QString> mods`, then seeds it with every **key** of `PseudoModManager::SUMMING_MODS` — the pseudo-mod names | modlist.cpp:244-253 |
| `AddStatTranslations(bytes)` | permissive glaze parse into `vector<repoe::StatTranslation>`, each with an `English[]` of `{format[], string, is_markup}`. On a parse error: log and return (the set keeps what it had) | modlist.cpp:255-270 |
| per English entry | skip if `is_markup == true` (3.24 necropolis duplicates). Then, **unless `format[0] == "ignore"`**, for `i` in `0..format.size()-1` replace every `"{i}"` in `string` with `format[i]`. Insert into `mods` if non-empty | modlist.cpp:272-293 |
| so a template is | RePoE's `string` with its `{0}`,`{1}`… slots filled by the format tokens (`"#"`, `"+#"`, `"#%"`, …) — which is exactly why an item line normalised to `#` matches | modlist.cpp:280-288 |
| `format[0] == "ignore"` | the loop is skipped and the string is inserted **with its `{0}` braces intact** — such an entry can never match a normalised mod line | modlist.cpp:281 |
| `open:` | `stat.format[0]` is indexed unconditionally; an English entry with an empty `format` array would be an out-of-bounds read. Whether RePoE ever emits one would be settled by scanning a cached `stat_translations.min.json` for an entry with `"format": []` | modlist.cpp:281 |

`InitModList()` then, for each string in the `mods` set: dedupe via a second
set (unreachable — `mods` is already a `std::set`, so the "duplicate mod" warn
is dead), build `SumModGenerator(mod, {mod})` — a one-element match list, i.e.
the template matches only itself — and register `mods_map[mod] = gen.get()`
with the shared pointer kept alive in `mod_generators` (modlist.cpp:297-322).

### 4d. `AddModToTable`: natural then pseudo

Both passes key on the same `generic_mod` (§4b) and both write into the same
`ModTable output` (`unordered_map<QString,double>`), which is the item's
`m_mod_table`.

| Pass | Rule | Cite |
| --- | --- | --- |
| natural | `mods_map.find(generic_mod)`; on a hit, `gen->Generate(raw_mod, output)` → `output[template] = mean-of-numbers`; **assignment, last wins** | modlist.cpp:365-372 |
| pseudo | `pseudo_mgr.SUMMING_MODS_LOOKUP.find(generic_mod)`; on a hit, for **each** pseudo-mod name in the hit's vector: `value = MatchMod(generic_mod, raw_mod)` and `output[pseudo] += value` (initialised to `value` if absent) — **summed across every contributing line on the item** | modlist.cpp:374-392 |
| pseudo value | note the match template here is the item's **own** normalised line, not the pseudo-mod's list entry — so `value` is again the mean of that line's numbers | modlist.cpp:385-386 |
| multiplicity | `SUMMING_MODS_LOOKUP` is `reverseMap(SUMMING_MODS)`, which `push_back`s per occurrence. A real mod listed N times under one pseudo-mod (e.g. `"+#% to all Elemental Resistances"` appears 3× under `"+#% total Elemental Resistance"`) therefore contributes **N × value** | pseudomods.cpp:43-58, 313-325 |
| when is a pseudo-mod present | exactly when at least one line on the item normalises to a template listed in that pseudo-mod's real-mod vector. Pseudo-mod names are also seeded into `mods` so they appear in the dropdown even for items that have none | modlist.cpp:248-252, 376-377 |
| `open:` (latent bug) | `double value;` is declared uninitialised and `MatchMod`'s return is ignored; `MatchMod` writes `*output` only on the success path. In practice the template *is* the line's own normalisation so it always matches, but a line where the two disagree would read an uninitialised double. A targeted unit test over `AddModToTable` with a crafted line (e.g. one already containing `#`) would settle whether the path is reachable | modlist.cpp:385-386, util.cpp:189 |

### 4e. `mod_list_model` (the dropdown)

| Property | Value | Cite |
| --- | --- | --- |
| Contents | the deduped union of (a) every `PseudoModManager::SUMMING_MODS` key and (b) every non-empty, non-markup RePoE English stat-translation string with its `{i}` slots substituted — i.e. exactly the keys of `mods_map` | modlist.cpp:248-252, 272-293, 297-313 |
| Built by | `InitModList()`, called once from `RePoE::FinishUpdate()` after the stat files are loaded; a `QStringListModel` at namespace scope, exposed by reference via `mod_list_model()` | modlist.cpp:26, 239-242, 314-322, repoe.cpp:203 |
| Order | iterated in `std::set<QString>` order (case-**sensitive** `QString` `<`), then `mod_list.sort(Qt::CaseInsensitive)` → the model is sorted case-insensitively | modlist.cpp:302, 314-321 |
| Consumer | each mods-filter row's editable `QComboBox` plus a `QCompleter`, both over a `TokenAndFilterProxy` that re-sorts case-**sensitively** | ui/modsfilterform.cpp:96-121 |
| Matching | a row `{mod, min, max}` matches an item iff `mod_table().count(mod)` and the stored double is within `[min,max]`; a template absent from the table fails the row outright | filters/filtermatchers.cpp:260-271 |

---

## 5. Sort identity

| Function | What it uses | Cite |
| --- | --- | --- |
| `Item::operator<` | `std::tie(PrettyName(), m_uid, m_hash) < std::tie(rhs.PrettyName(), rhs.m_uid, rhs.m_hash)` — lexicographic on `QString` (case-sensitive, code-unit order): pretty name, then the GGG item `id`, then `hash_v4`. Location, category, rarity and mods play no part | item.cpp:667-673 |
| `Item::CalculateHash` | MD5 (`Util::Md5`) of `name + "~" + typeLine + "~"` followed by `unique_common` = every `explicitMods[].description + "~"`, then every `implicitMods[].description + "~"` (raw JSON order, flags ignored, crafted/fractured/enchant not distinguished), then `item_unique_properties(properties)` + `"~"`, then `item_unique_properties(additionalProperties)` + `"~"` (each = `name + "~"` then every `values[i][0] + "~"`), then for each socket **with an `attr`** `group + "~" + attr + "~"`, then `"~" + location.GetLegacyHash()` (`"stash:<tab label>"` or `"character:<name>"`). `m_hash` = MD5 of that; `m_old_hash` = MD5 of the same with the literal prefix `"<<set:MS>><<set:M>><<set:S>>"` before the name. **Not folded:** id/uid, ilvl, requirements, note, influences, corrupted/split/synthesised flags, `enchantMods`, x/y, frameType, icon | item.cpp:625-665, 40-53, itemlocation.cpp:140-153, util.cpp:32-37 |
