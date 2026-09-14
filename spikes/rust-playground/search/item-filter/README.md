# item-filter — GGG's own predicate language over an item

Status: first pass complete — 2026-09-13

- GGG's page names **61 conditions** over an item and **14 actions** that
  read nothing. 54 conditions apply to PoE1: 6 carry a `PoE2-only`
  ribbon, 1 (`GemQualityType`) is ribboned `Deleted`.
- **It is not a query language.** Conditions inside a `Show`/`Hide`/
  `Minimal` block are ANDed; rules are tried in order and matching stops
  at the first hit unless the block says `Continue`. There is no OR, no
  grouping and no cross-condition negation — only the per-condition
  operators.
- Of the 61 conditions, **22** are also named by both other surfaces,
  **14** by trade alone, **1** by the C++ app alone, **24** by neither.
  Reverse: of the C++ app's 38 filters the language names **19** (14
  exact, 5 divergent); of trade's 86 item-reading filters, **27** (21
  exact, 6 divergent).
- Five divergences would bite a shared vocabulary: `Foulborn`/`Mutated`,
  `Synthesised`/`Synthesized`, `AreaLevel` (drop area vs a map's own),
  `Base*` (base value vs total after mods), and tier-vs-boolean on the
  Exarch/Eater/Scourge flags.
- Only **32** of the 61 read a field the stash export carries: 17 have no
  export field at all, 5 need base-item data (RePoE), 7 are derived.

## The language, as the page states it

Blocks: `Show`, `Hide` (Normal filters only), `Minimal` (Ruthless only),
plus `Continue` and `Import "file" [Optional]`. The page's own wording:
"If there are multiple conditions in a block then all of them must be
matched for the block to match an item."

One operator table serves every condition, and the page states no
per-condition restriction:

| Operator | The page says |
| --- | --- |
| `=` | Equal |
| `!`, `!=` | Not equal |
| `<`, `<=`, `>`, `>=` | Less/greater than (or equal) |
| `==` | Exact match |

`data/conditions.csv`'s `operators` column is therefore **derived**: the
meaningful set for the operand (all eight where the operand is ordered,
equality-only where it is a boolean or a set), not a page statement.

## Findings

**F1 — What the page does not say about matching.** It gives `==` as
"Exact match" and never says what a bare value matches. So the semantics
of `BaseType "Thicket Bow"`, `Class Currency` and the mod names in
`HasExplicitMod` are unresolved here (Q1) — prefix, substring or exact.
Likewise `HasExplicitMod >=2 "of Haast" "of Tzteosh" "of Ephij"`: the
page says "mod name with numeric condition for number of modifiers" and
does not say whether the count ranges over the named set or over all
mods. Every claim about looseness in this track is marked open, not
assumed.

**F2 — Three namings, one thing, three spellings.** From
`data/conditions.csv` (`cpp_filter`, `trade_filter`) and
`data/coverage.csv`:

| Thing | Filter language | C++ caption | Trade |
| --- | --- | --- | --- |
| the Foulborn flag | `Foulborn` | `Mutated` | id `mutated`, text "Foulborn" |
| synthesised | `SynthesisedItem` | `Synthesized` | `synthesised_item` |
| unidentified | `Identified` | `Unidentified` (negated) | `identified` |
| gem level | `GemLevel` | `Level` | `gem_level` |
| item level | `ItemLevel` | `ilvl` | `ilvl` |
| largest link group | `LinkedSockets` | `Links` | `links` |
| enchanted at all | `AnyEnchantment` | `Enchanted` | — (`enchant` stats only) |

The export field is a fourth spelling again (`mutated`, `synthesised`,
`identified`, `ilvl`), recorded per condition in `item_field`.

**F3 — Where they diverge in meaning, not spelling.**

| Condition | Divergence |
| --- | --- |
| `AreaLevel` | The level of the area the item **dropped in**. Trade's `area_level` reads a map's own area level. Same spelling, different referent. |
| `BaseArmour`, `BaseEvasion`, `BaseEnergyShield`, `BaseWard` | Base values before mods. The C++ `Armour`/`Evasion`/`Shield` and trade's `ar`/`ev`/`es`/`ward` all read the total. Nothing in the filter language reads a total. |
| `HasSearingExarchImplicit`, `HasEaterOfWorldsImplicit` | Compare an implicit **tier** 1–6. Trade's `searing_item`/`tangled_item` and the export's `searing`/`tangled` are yes/no. |
| `Scourged` | Yes/no. Trade's `scourge_tier` and the export's `scourged.tier` carry a tier — the mirror image of the row above. |
| `Rarity` | **Ordered**: `Rarity > Magic` is legal. The C++ combo and trade's `rarity` option are unordered sets, and trade adds `uniquefoil` and `nonunique`. |
| `HasInfluence` / `ShaperItem` / `ElderItem` | Names each influence separately (7 values incl. `None`). The C++ `Influenced` is one boolean over six; trade has no influence filter at all — it lives in pseudo stats (`pseudo.pseudo_has_*_influence`). |
| `Class` | A third class vocabulary, coarser on trade's side (`repoe/data/class-to-trade-category.csv`) and substring-matched on the C++ side. |
| `Sockets` / `SocketGroup` | Count and colours in **one** condition (`Sockets >= 5GGG`). The C++ app splits count (`Sockets`/`Links`) from colours (`Colors`/`Linked`); trade's `sockets`/`links` carry both shapes in one filter. |

**F4 — What only the filter language names.** 24 conditions have neither
a C++ filter nor a trade filter (`data/conditions.csv`, both join columns
empty). The ones a stash search could actually evaluate today:

- `Width`, `Height` — the export carries `w` and `h` on **every** item
  (`item-facts/data/field-census.csv`, share 1.000) and neither other
  surface offers a filter.
- `Replica` — export field `replica`.
- `HasImplicitMod` — "has at least one implicit", no count, no name.
- `AlternateQuality`, `Exceptional`, `ArchnemesisMod`, `ElderMap`,
  `ShapedMap`, `ZanaMemory`, `MirageMap`, `CorruptedMods` — named by GGG,
  by nobody else, and with no export field found here.
- `DropLevel` and the four `Base*` defences — base-item data, so RePoE,
  not the export (`repoe/data/base-taxonomy.csv` is the entry).
- `BaseDefencePercentile` is the one condition where GGG's filter name,
  trade's filter id and the definition all coincide exactly, and where
  the export carries nothing: it must be derived.

**F5 — What the filter language does not name** (`data/coverage.csv`,
`kind=none`): 19 of the C++ app's 38 filters and 59 of trade's 86
item-reading filters. The pattern is sharp — the filter language names
**no computed number and no requirement**: no DPS of any kind, no APS,
no crit, no block, no total defence, no `R. Level`/`Str`/`Dex`/`Int`,
and no trade `damage`/`lvl`/`str`/`dex`/`int`. The owner's reading of why, 2026-09-13, verbatim: "item filters are design by GGG to be unable to require specific numbers, because GGG wants people to experience the randomness of checking loot" — so the absence says nothing about whether a search needs values; it does (owner-seat F1). It also names nothing
about a listing (`Tab`, `Priced`, trade's `price`/`account`/`indexed`
are excluded or unnamed) and, oddly, no `Crafted`, `Veiled` or `Split`,
which both other surfaces do name. Its league-mechanic vocabulary is a
handful of booleans against trade's Heist/Sanctum/Ultimatum blocks.

**F6 — What a stash search inherits.** By `item_field` in
`data/conditions.csv`: 32 conditions read an export field directly, 5
need base-item data, 7 are derivable, 17 have no source found. The 17
are mostly drop-time or PoE2-only state (`AlwaysShow`, `AreaLevel`,
`TwiceCorrupted`, `WaystoneTier`, `UnidentifiedItemTier`,
`HasVaalUniqueMod`, `IsVaalUnique`) plus league booleans. So a stash
search can inherit roughly two thirds of GGG's own vocabulary with no
new data, and a further eighth by joining RePoE.

**F7 — The class vocabulary is not on the page.** `Class` is documented
as taking an "Item class name" with one example, `Class Currency`, and
the page lists no values. `data/class-names.csv` records that honestly:
one `class_value` row (Currency → export `Currency` → trade `currency`)
and five incidental tokens that only occur inside other phrases (`Bow`
in "Thicket Bow", `Shield` in "Energy Shield", `Jewels` in "Cluster
Jewels", `Maps` in "Mirage Maps", and the condition name
`ArchnemesisMod`). The taxonomy join therefore still rests on repoe F3,
not on this page (Q3).

**F8 — PoE1 and PoE2 share one page.** Six conditions carry a
`PoE2-only` ribbon (`AlwaysShow`, `HasVaalUniqueMod`, `IsVaalUnique`,
`TwiceCorrupted`, `UnidentifiedItemTier`, `WaystoneTier`), three are
`New`, one `Updated`, one `Deleted`. `WaystoneTier` is PoE2's `MapTier`
under a different name — a filter-language instance of the realm
coordinate the store already carries.

**Actions.** 14, in `data/actions.csv`: eight drop-sound actions and six
label/minimap/effect actions. None reads an item; they do not concern a
search.

## Numbers

| Quantity | Value | Source |
| --- | --- | --- |
| Conditions on the page | 61 | `data/conditions.csv` |
| — PoE2-only / New / Updated / Deleted ribbons | 6 / 3 / 1 / 1 | same |
| — boolean / numeric / name / enum / compound operands | 27 / 22 / 4 / 3 / 5 | same |
| Actions | 14 (8 drop sound, 6 other) | `data/actions.csv` |
| Operators in the page's one table | 8 | page, Operators |
| Block keywords | 5 (`Show`, `Hide`, `Minimal`, `Continue`, `Import`) | page, Blocks |
| Conditions named by both other surfaces / trade only / C++ only / neither | 22 / 14 / 1 / 24 | `data/conditions.csv` |
| — of the joins, ones that are a trade **stat**, not a filter id | 7 | same |
| C++ filters: exact / divergent / not named | 14 / 5 / 19 of 38 | `data/coverage.csv` |
| Trade item-reading filters: exact / divergent / not named | 21 / 6 / 59 of 86 | same |
| Conditions by source of the value: export field / derived / base data / none found | 32 / 7 / 5 / 17 | `data/conditions.csv` |
| Class values the page gives | 1 (`Currency`) | `data/class-names.csv` |

## Open questions

| # | Question | The one read that closes it |
| --- | --- | --- |
| Q1 | What does a bare (operator-less) value match for `BaseType`, `Class`, `HasEnchantment`, `HasExplicitMod` — prefix, substring or exact? And does `HasExplicitMod`'s count range over the named set or over all mods? | The worked-example filter that has not landed: a widely used filter at a pinned release, read against this page — its usage of `==` versus a bare value settles both. |
| Q2 | Which export field, if any, carries `Mirrored`, `BlightedMap`, `UberBlightedMap`, `TransfiguredGem`, `GemQualityType`? `duplicated` is the candidate for the first and is unverified — GGG's field reference documents names only. | A value census over the item-facts store: join `duplicated` and the map/gem `typeLine`s against known items, one script in `item-facts/`. |
| Q3 | What is the class vocabulary the filter language accepts, and does it equal RePoE's class display names? | The class names a worked-example filter uses, diffed against `repoe/data/class-to-trade-category.csv`. |
| Q4 | Do the `PoE2-only` ribbons enumerate the whole PoE1/PoE2 divergence in the language? | A dated save of PoE2's own filter documentation (a second `SURFACES.md` capture), diffed against this one; the trigger is the December launch. |

**Not done:** the brief's second input, one or two widely used filters at
a pinned release, never landed, so nothing in this track is checked
against the grammar in practice. Q1 and Q3 both wait on it.

## Provenance

| What | Where |
| --- | --- |
| The page save, its sha256 and how it was obtained | `MANIFEST.md` |
| Conditions, operands, examples, the three-way join | `data/conditions.csv` ← `scripts/extract-conditions.py` |
| Actions, by name | `data/actions.csv` ← same script |
| The reverse join (every C++ and trade filter) | `data/coverage.csv` ← `scripts/coverage.py` |
| The class evidence | `data/class-names.csv` ← `scripts/extract-class-names.py` |
| Sibling extracts joined to | `../cpp-search/data/filters.toml`, `../trade-query/data/grammar.json`, `../trade-query/data/stats-2026-09-12.json`, `../repoe/data/class-to-trade-category.csv`, `../item-facts/data/field-census.csv` |

GGG's prose descriptions are not copied into `data/`: `conditions.csv`
carries the page's vocabulary (names, values, examples) and a hand-kept
`notes` column that paraphrases whatever the description added.

Both scripts abort if a hand-kept join names a C++ caption, a trade
filter id, a trade stat id or an export field path that its sibling
extract does not contain, so the joins cannot rot silently against a
regenerated sibling.

## Review

| # | Finding | Fix |
| --- | --- | --- |
