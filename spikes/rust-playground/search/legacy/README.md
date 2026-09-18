# legacy — Path of Building's unique variants against the owner's items

Status: second pass complete — 2026-09-18

- Of the 5,777 uniques the two stores hold, **4,090 fit exactly one** Path of Building selection,
  229 fit several, 697 fit none, and 761 bind to no entry at all. **171 fit both an old and a
  not-old member** — the park's undecided case (C93), and the one count neither fix moved.
- Following Path of Building on both counts — every variant an entry's `Has Alt Variant` header
  lets be chosen at once, and the catalyst's rescale of a tagged line — moves **70** rows, every
  one `none` → `one`: **46** by the rescale, **24** by the selection; the strict rule still
  reproduces the first pass exactly. **513** of the 697 left at `none` display a line no selection
  carries ("Why a unique fits no variant"; `REVIEW.md`, row 8).
- The owner's Ashes of the Stars: 10 in the population — 6 `Current`, 4 `Pre 3.23.0`, none
  unfitted — and 1 foil counted apart. The three catalysed ones show `+19 to all Attributes`, which
  the `+20%` rescale of `+(10-16)` to `(12-19)` admits. None is `either`.
- **4,789 of 36,139 items (0.133)** carry a line outside implicit and explicit, over every rarity;
  helmets are 288 of the 1,686 `enchantMods` items. Two arrays give no text at all —
  `ultimatumMods` gives a mod id, `veiledMods` a slot id — and neither id resolves anywhere here.
- **No input on this machine says a line can no longer be made.** The nearest are `spawn_weights`
  in `poe1/data/mods.json` (all-zero means not rollable at that export), the two export histories
  back to 2019-11, and `Source: No longer obtainable` on 170 unique entries — which is about the
  item, never a line.

## What was measured

`variants.py` parses every `[[ … ]]` block of the clone's unique files at `16de4b82`; `match.py`
binds every unique in the owner's two stores to the entries of the same `name` and fits it twice —
under the **strict** rule, one variant at a time with nothing rescaled, and under the **final** one,
a selection of variants rescaled by the item's catalyst; `arrays.py` censuses every other mod
array, over every rarity. Per-item detail is the CSV, which carries both outcomes and what moved a
row between them. Nothing here judges whether a line can still be made — that is the sources-read
table, every claim of which cites a path in a clone at its commit (S178, R8).

The rules the fit follows are documented where they run, each with the Path of Building lines it follows:
`scripts/common.py` — which line is a header or a base, the sign folded into a number and ranges
sorted, a selection on an alt-variant entry, the catalyst kind mapped by position, which lines and
which numbers a catalyst rescales and how they round; `scripts/match.py` — the two-way fit by
cover over implicit and explicit lines pooled, the frame types matched and counted apart. Nine
definitions were sharpened from the briefs, none swapped; the list as prose is this page at
`77bca830`.

## The numbers

| | |
| --- | --- |
| Unique files read / entries parsed / blocks refused | 27 / 1,317 / 14 |
| Distinct entry names / names with more than one entry | 1,310 / 4 |
| Variants: total / old (`Pre <version>`) / not-old | 3,203 / 1,495 / 1,708 |
| Entries with no `Variant:` line (one variant, `only`) | 558 |
| Entries whose base varies by variant / is not in `Data/Bases` | 56 / 3 |
| Entries with `Source:` / `League:` / `Upgrade:` | 673 / 488 / 31 |
| Entries whose `Source:` is `No longer obtainable` | 170 |
| Items in both stores, deduplicated by GGG id | 36,139 |
| Counted apart: unidentified / relic or foil frame type / realm `poe2` | 126 / 7 / 1 |
| **Population matched** | **5,777** |
| **Outcome, final**: `one` / `several` / `none` / `unbound` | **4,090 / 229 / 697 / 761** |
| Outcome, strict (one variant, nothing rescaled) | 4,020 / 229 / 767 / 761 |
| `either` — the fitting set holds an old and a not-old member — final / strict | 171 / 171 |
| Rows that moved, all `none` → `one`: by the catalyst / by the selection / by both | 46 / 24 / 0 |
| Catalyst quality by final outcome: `one` / `several` / `none` / `unbound` | 52 / 0 / 4 / 5 |
| Lines the 61 catalysts rescale, over every candidate entry | 114 |
| `none` on a number outside every range, final / strict — of those, with catalyst quality | 106 / 152 — 0 / 46 |
| Population items with a line outside implicit and explicit | utility 302, enchant 171, crucible 14, scourge 8, veiled 7 |
| Population items whose implicit or explicit lines carry a flag | mutated 78, crafted 9 |
| Distinct names binding to no entry | 108 |

The strict-to-final transition is diagonal but for one cell: 70 rows go `none` → `one`, and nothing
else moves. By league, final `one` / `several` / `none` / `unbound`: Standard 2,380 / 142 / 537 /
696; Solo Self-Found 1,291 / 61 / 109 / 40; Hardcore 229 / 16 / 21 / 2; Allflame 190 / 10 / 30 /
23. The 761 `unbound` are mostly kinds the unique files do not carry — Heist contracts
(`Contract: The Slaver King` 62), unique maps (`Whakawairua Tuahu` 56) — then `Forbidden Flesh` and
`Forbidden Flame`, 44 each, which `Special/Generated.lua` builds in code.

## Why a unique fits no variant

Under the final rule: the reason is the first in the brief's order, measured against every
candidate selection at once.

| Reason | Items | The commonest line |
| --- | ---: | --- |
| a displayed line with no text match in any selection | 513 | `Regenerate 8 Life over 1 second when you Cast a Spell` (Lifesprig, 28), against the entry's `… for each Spell you Cast` |
| a text match with a number outside every range | 106 | `33% increased Global Critical Strike Chance` (White Wind 9, Bloodsoaked Medallion 4), 20 by template |
| a selection line the item does not display | 78 | `(10-15)% increased Stun and Block Recovery` (Fairgraves' Tricorne 11, Hrimnor's Resolve 8), 27 |

Behind the first row sit `Extra gore` (14) and corrupted or eldritch implicits an entry never
carries. Neither thing the fixes address survives as a reason: no `none` row is out of range on a
catalysed number (46 were), and **123** of the 697 still bind to an alt-variant entry — down from
147 — but 110 of those fail on a line no variant carries and 13 on a line no selection displays,
not on the one-at-a-time rule. **16** match once `increased` and `reduced` are exchanged:
Ventor's Gamble is written `(-40-40)% reduced Rarity of Items found`, and the game shows `17%
increased Rarity of Items found` when the roll is positive. **15** match once a trailing plural `s`
is ignored (`Gain 3 Frenzy Charges on use` against `Gain (1-3) Frenzy Charge on use`).

## The alt-variant entries, and what the owner's items do on them

12 of the 14 entries carrying a `Has Alt Variant` header bind an item. Choosing every variant the
header allows moves 24 rows, all of them `none` → `one`; the six Circle entries move nothing. Their
102 items are exactly the 102 of the 123 remaining `none` rows that fail on a line the item shows
as one of its own implicits (`13% increased Lightning Damage`), which no variant of the entry
carries. The alt-variant shape is there in their explicit lines; the implicit sinks the fit first.

Per entry, items and final outcome: the six Circles (`ring.lua` #16–#21) 102, all `none`;
Cinderswallow Urn 21 (`one` 2, `none` 19); Militant Faith 12, Split Personality 4 and Might and
Influence 4, all `one`; Tombfist 3 (`one` 2, `none` 1); Eyes of the Greatwolf 1, `none`. `match.py`
prints the table with variants and selection size; the rows are `data/unique-fit.csv` by `name`.

## The catalysed items

61 items carry catalyst quality, of nine kinds — `Attribute` 27, `Elemental Damage` 9,
`Resistance` 6, `Caster` 5, `Physical and Chaos Damage` 4, `Defence` 3, `Attack` 3, `Life and Mana`
3, `Critical` 1 — and their catalysts rescale 114 lines across the entries they bind. 46 of the 50
that fitted nothing now fit exactly one; the remaining 4 fail on a line no variant carries, never
on a number. The six that fitted *unscaled* under the strict rule fit the same variant after the
rescale, so it confirmed rather than reassigned them — though for one, Presence of Chayula, the
entry carries no line its kind touches, and nothing of it was rescaled.

The six: Astral Projector, Mark of the Elder and Presence of Chayula (`only`); Dyadian Dawn and Le
Heup of All (`Current`); The Torrent's Reclamation (`Pre 3.16.0`). `match.py` prints them with kind,
quality and lines rescaled; in `data/unique-fit.csv` they are `catalyst_quality` yes with no
`moved_by` and an outcome of `one`.

## The label-shape census

| Shape | Variants |
| --- | ---: |
| bare `Pre <version>` | 1,333 |
| `Current` | 703 |
| `only` (the entry has no `Variant:` line) | 558 |
| names an axis other than a version (`Life`, `Cold Damage`) | 287 |
| compound holding `Pre <version>` (`Two Abyssal Sockets (Pre 3.21.0)`) | 162 |
| parenthesised qualifier (`Two-Stone Ring (Cold/Lightning)`) | 83 |
| `<thing>: <axis>` (`Purity of Ice: Cold`) | 73 |
| names a version, but not as `Pre` | 4 |

Of the 14 refusals, 13 are `Special/Generated.lua`: Lua code whose `[[ … ]]` blocks are fragments
of the strings it concatenates, so no entry, variant or label can be taken from it. What can be
read there by eye is the 15 names it builds — Megalomaniac, Watcher's Eye, Forbidden Flame and
Flesh, Skin of the Lords, Pearl of Tsoatha among them — and that their labels come from runtime
data (cluster-jewel notables, gem names, tree notables), so they cannot be enumerated without
running the Lua. The 14th is `shield.lua` #25, Mutewind Pennant, whose mod lines name a
`{variant:5}` in a four-variant entry. The 3 entries whose base is not in `Data/Bases` are
Duskdawn, its Replica and Taryn's Shiver, all `Maelström Staff`, which `Data/Bases/staff.lua`
spells `Maelstrom Staff`; the store's items spell it as the unique files do.

## Ashes of the Stars (S165)

The entry is `amulet.lua` #5, `Onyx Amulet`, and parses to the two variants the brief expects.
`Pre 3.23.0` carries `(10-20)% increased Reservation Efficiency of Skills`; `Current` does not.
Both carry `+(10-16) to all Attributes` — the one line tagged `{tags:attribute}`, and so the one an
`Attribute` catalyst rescales — `(5-10)% increased Experience Gain of Gems`, `+1 to Level of all
Skill Gems`, `+(20-30)% to Quality of all Skill Gems`. The store holds 10 in the population and 1
apart. Under the final rule every one of the ten fits exactly one variant, and none is `either`:
the Reservation Efficiency line decides each. At `+20%` the attribute range reads `(12-19)`, which
is what admits the three the strict rule left at `none`.

| Row keys | League | Fits | The lines that decide it |
| --- | --- | --- | --- |
| `32253274c7ad`, `4fe1659283f3`, `ac96c3d10ad1`, `b863a0d867f8`, `c6a47058fdc8` | Standard ×3, Allflame ×2 | `Current` | no Reservation Efficiency line; `+10`, `+13`, `+16 to all Attributes`; quality `+24%`…`+30%` |
| `8061f87d24e3`, `dc57fb4db12d` | Standard | `Pre 3.23.0` | `20% increased Reservation Efficiency of Skills`, inside `(10-20)`; `+16 to all Attributes` |
| `01fa6690afdc` | Standard | `Current` after the rescale (strict: none) | `Quality (Attribute Modifiers) +20%`, `+19 to all Attributes` — inside the rescaled `(12-19)`, outside the written `(10-16)`; no Reservation Efficiency line |
| `a516661982fb`, `d8a9c2befa5d` | Standard | `Pre 3.23.0` after the rescale (strict: none) | the same `+20%` and `+19 to all Attributes`, and each also shows `20% increased Reservation Efficiency of Skills` |
| `32ab63d8cbe4` | Standard | counted apart | `SupporterFoil`, `isRelic`; shows `+19 to all Attributes` too |

## The arrays outside implicit and explicit

4,789 of the 36,139 items — 0.133 of the corpus — carry at least one line outside implicit and
explicit. `crafted`, `fractured` and `mutated` are not arrays: they are flags on `explicitMods`
lines (1,041 / 289 / 81), so the fit reads them like any other line.

| Array | Items | Templates | Lines | By rarity |
| --- | ---: | ---: | ---: | --- |
| `utilityMods` | 2,613 | 26 | 3,225 | Magic 1,510, Normal 1,352, Unique 363 |
| `enchantMods` | 1,686 | 611 | 2,948 | Rare 1,873, Magic 532, Normal 346, Unique 186, Foil 2 |
| `ultimatumMods` | 554 | 36 | 3,634 | Currency 3,634 |
| `crucibleMods` | 62 | 68 | 146 | Rare 57, Unique 34, Normal 28, Magic 27 |
| `veiledMods` | 55 | 2 | 68 | Rare 56, Unique 11 |
| `scourgeMods` | 12 | 20 | 28 | Unique 20, Rare 8 |
| `runeMods` | 1 | 3 | 3 | Magic 3, realm `poe2` |
| `bondedMods` | 1 | 6 | 6 | Currency 6, realm `poe2` |

Item class is only the icon directory, which the CSV carries per template: `utilityMods` is
`Flasks/*` throughout; `enchantMods` is `Jewels/NewGemBase2` 761, `NewGemBase3` 593,
**`Armours/Helmets` 288**, `Maps/Atlas2Maps` 248; `crucibleMods` is `Weapons/TwoHandWeapons` 76.
Every array but `veiledMods` is mostly Standard; crucible, scourge, rune and bonded are Standard
only. Two give no text at all: `ultimatumMods` gives `{tier, type}` with `type` an id
(`FrostInfection` 220), and `veiledMods` gives `Prefix01`/`Suffix02`. `enchantMods` is dominated by
cluster-jewel lines (`Adds 4 Passive Skills` 557), not by helmets.

## The sources read

One row per kind: what a source here holds, what it lacks, where. Paths are relative to the clones'
parent, at the commits in `MANIFEST.md`; `mods.json` is `poe1/data/mods.json`, and the Lua files
are under `PathOfBuilding/src/Data/`.

| Kind | Holds / lacks | Path |
| --- | --- | --- |
| a rare item's mods | per mod id: `domain`, `generation_type` (`prefix`/`suffix`), `text`, `stats`, `required_level`, `spawn_weights` as tag→weight — all-zero means it cannot be rolled at that export / never says a mod was removed, or why a weight is zero | `mods.json` |
| — when it existed | `git log -- data/mods.json` in `poe1/` spans 2025-06-14 (3.26.0.1) to 2026-09-04 (3.29.3.3); the same over `RePoE/data/mods.json` in `repoe/` spans 2019-11-10 to 2025-06-07, where `99757db2` moved the data out of the tree (`git show <rev>:…`); together they date a mod id per export / nothing before 2019-11, whose schema has no `text` | `poe1/`, `repoe/` |
| helmet and other enchants | `generation_type: "enchantment"`, domain `item`, with per-base-tag `spawn_weights` and `required_level`; `EnchantmentHelmet.lua` and its Body/Boots/Gloves/Belt/Weapon/Flask siblings, keyed by skill then labyrinth difficulty / neither says an enchant can no longer be made, and the Lua tables carry no weight column | `mods.json`, `EnchantmentHelmet.lua` |
| crucible | `generation_type: "crucible_tree"` and `"crucible_unique_tree"`, domain `crucible_remnant` for map mods; `Crucible.lua` has the same nodes with `weightKey`/`weightVal` / a zero weight, not a date | `mods.json`, `Crucible.lua` |
| ultimatum | **none found**: the item's `type` (`Flamethrower`) is not a key or value in `mods.json`, `mod_types.json` or `stats.json`, nor in awakened-poe-trade's `stats.ndjson`; hits on that string are the Flamethrower Trap skill | — |
| veiled | domain `veiled` holds `VeiledPrefix`/`VeiledSuffix` with `veiled_mod_type` and `veiled_mod_seed`; the revealed mods sit in domain `unveiled` / nothing maps the item's `Suffix02` to either, and there is no `veiled_mod_type` translation | `mods.json`, `poe1/data/stats.json` |
| scourge, flask utility | `generation_type: "scourge_benefit"`, `"scourge_detriment"`, `"scourge_gimmick"`, and domain `flask` beside the flask-tagged prefixes and suffixes, each with `text` and `spawn_weights` / same limit as the rare mods | `mods.json`, `ModFlask.lua` |
| rune, bonded (PoE2) | **none found**: `poe1/`, this Path of Building clone and awakened-poe-trade's data are all PoE1 | — |
| a unique's own lines | `Variant:`, `Source:`, `League:`, `Upgrade:`, maintained patch by patch — `git log -S'No longer obtainable' -- src/Data/Uniques/` shows `Tag Serle's Masterwork as unobtainable (#9771)`, 2026-04-18 / `Source:` is about the item, never a line, and `Pre <version>` says a variant existed before a version, not when it stopped being creatable | `Uniques/` |
| — when, for one line | `Generated.lua`'s `watchersEyeLegacyMods`: keyed by mod id, a `version` naming the patch the mod changed in and a `legacyMod` rewriting the old range, under a comment listing three legacy scenarios — the only place here that says when a line changed / Watcher's Eye and neighbours only, in Lua | `Uniques/Special/Generated.lua` |
| the uniques themselves | one row per unique with `item_class`, art, `renamed_version` and `base_version`; presence across the export history dates when a unique entered or left / no mod lines, no variants | `poe1/data/uniques.json` |
| a line as the trade site names it | a display template with its matchers and its trade stat ids per line kind — `explicit`, `implicit`, `fractured`, `enchant`, `crafted`, `veiled`, `pseudo`, `imbued`; `items.ndjson` lists uniques with base and disenchant value / no legacy or obtainability marker, and no variant | `awakened-poe-trade/…/data/en/` |

## Open questions

- **What do the `ultimatumMods` `type` ids mean?** Read GGG's `/api/trade/data/stats` for an
  `ultimatum` group, or an export carrying `UltimatumModifiers`: settles whether the id has display
  text anywhere, or whether the API's `type` is all there is.
- **What does a `veiledMods` `Suffix02` name?** Read the trade site's `veiled.*` stat list against
  one of the 55 veiled items: settles whether the id names a mod family or only a slot and a seed.
- **When did a line stop being creatable, before 2019-11?** Nothing here reaches further back; a
  deeper game-data history, or a source stating patch notes per mod, would settle it.
- **Is `Maelstrom Staff` in `Data/Bases/staff.lua` a typo or a second base?** `git log -S'Maelstrom
  Staff' -- src/Data/` in the same clone settles whether the parse's fallback covers a bug.
- **Do the 513 `no-text-match` rows fail on a line the entry never carried, or on one whose wording
  changed?** Match each failing line against `awakened-poe-trade`'s `stats.ndjson` matchers, and
  against the same entry across the clone's history: separates an item Path of Building does not
  describe from a template that drifted.

## Provenance

What each pass left out is named in its commit message (`1cf77994`, `77bca830`) and is restorable
from the scripts; the page before text moved to the scripts and `REVIEW.md` is at `77bca830`.

Every number above is printed by `variants.py`, `match.py` or `arrays.py`, which regenerate the
four `data/` files byte for byte, checked twice; `arrays.py` and `data/other-arrays.csv` are
untouched by this pass. The store copies, the clones and their commits, and the sibling-track files
read — including `../item-facts/data/properties-census.csv`, which the brief does not list — are
rows in `MANIFEST.md`. The sources-read table carries no count of its own: only the three scripts
print numbers, and its dates and commits are citations.

## Review

Nine rows, split out under the index's rule 5: [`REVIEW.md`](REVIEW.md). Row 8 is the open one — the
six Circle entries' 102 items fail on an implicit the entry does not list, and a fit that sets
such implicits aside was not measured.
