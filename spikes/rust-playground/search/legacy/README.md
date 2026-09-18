# legacy — Path of Building's unique variants against the owner's items

Status: second pass complete — 2026-09-18

- Of the 5,777 uniques the two stores hold, **4,090 fit exactly one** Path of Building selection,
  229 fit several, 697 fit none, and 761 bind to no entry at all. **171 fit both an old and a
  not-old member** — the park's undecided case (C93), and the one count neither fix moved.
- Following Path of Building on both counts — every variant an entry's `Has Alt Variant` header
  lets be chosen at once, and the catalyst's rescale of a tagged line — moves **70** rows, every
  one `none` → `one`: **46** by the rescale, **24** by the selection, none by the pair; the strict
  rule (one variant, nothing rescaled) still reproduces the first pass exactly. What is left of
  `none` is drift, not structure: **513** of the 697 display a line no selection carries, 110 of
  the 123 still on an alt-variant entry among them — 102 failing on one of the item's own implicits
  — and no `none` row is out of range on a catalysed number, where 46 were.
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
table, every claim of which cites a path in a clone at its commit (S178, R8). Definitions
sharpened, none swapped:

- **the fit pools implicit and explicit into one set** on each side, both ways, because the brief
  states it over the two together (`Implicits: N` is parsed but unused), and lines match by cover,
  not one to one;
- **a number's sign is folded into its value**, so a displayed `-24% to Fire Resistance` and a
  written `+(-25-50)%` reach one template; ranges written high to low (`(100-50)%`) are sorted; a
  `-` straight after a digit is a range dash (`59-88`), not a sign;
- **a header line is one Path of Building's own parser reads as one** (`src/Classes/Item.lua`
  `ParseRaw`, lines 535-561, 660-860), and a base line is one `src/Data/Bases/*.lua` declares — an
  entry may carry one base per variant, and a header may precede the base or be variant-tagged;
- **catalyst quality** is a `properties` row named `Quality (<kind> Modifiers)`, again `Item.lua`
  (lines 15, 673-677), so `Quality (Quantity)` on a map does not match;
- **the frame type is a name, not an integer** in both stores: the population is `Unique`, and
  `SupporterFoil`, which also carries `isRelic`, is counted apart;
- **a selection is a non-empty set of at most one plus the entry's `Has Alt Variant…` headers**,
  and a line belongs to it when any chosen variant lists it, an untagged line to every one
  (`Item.lua` `CheckModLineVariant` 2139-2145; the headers at 752-762, the choices at 774-783). The
  choices are independent, so a variant may be chosen twice, but a repeat shows nothing twice
  without `Allow Duplicate Variants` (2148-2164), which no entry here carries;
- **a line is rescaled when its `{tags:…}` prefix holds a tag the item's catalyst kind scales**
  (`getCatalystScalar` 31-62, over the kind-to-tags table at 16-29, copied into `common.py`): each
  scalable number is multiplied by `(100 + q) / 100` and floored **toward zero** at the line's
  precision, as `itemLib.formatValue` does (`ItemTools.lua` 61-75, `floorSymmetric` `Common.lua`
  766-773), and the item's own number is never un-scaled. A line with no `{tags:…}` never scales
  (line 36), nor one written `{unscalable}` or ending ` - Unscalable Value` (32-34, 1083-1085), of
  which the unique files hold none;
- **which numbers scale, and at what precision, is `src/Data/ModScalability.lua`**, which
  `findScalableLine` reaches by substituting values back into the line (`ItemTools.lua` 121-186,
  289-295): of the 915 distinct tagged lines, 853 carry a number, and the table answers 848 of
  them. The other 5 take the old method (301-355), scaling the first *n* numbers for *n* the count
  of `(a-b)` ranges, or 1. Only a value written as a fixed number is substituted back here: a
  range's stands for a roll, and never matches a key;
- **the catalyst kind maps by position, not by spelling**: the items spell it as the game does
  (`Elemental Damage`, `Physical and Chaos Damage`), `catalystDescriptorList` spells those two
  shorter, and its test is equality (line 675) — so Path of Building itself sets no catalyst from a
  `Quality (Elemental Damage Modifiers)` property. The two kinds keyed on `prefix` and `suffix`
  read a mod line's own flag, not `{tags:…}` (48-53), and so have nothing to match on a unique's
  line; no population item carries either kind.

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

| Entry | Variants / chosen at once | Items | Final outcome |
| --- | --- | ---: | --- |
| `ring.lua` #18 Circle of Fear | 7 / 2 | 26 | none 26 |
| `ring.lua` #19 Circle of Guilt | 7 / 2 | 23 | none 23 |
| `flask.lua` #13 Cinderswallow Urn | 18 / 2 | 21 | one 2, none 19 |
| `ring.lua` #17 Circle of Anguish | 7 / 2 | 21 | none 21 |
| `ring.lua` #21 Circle of Regret | 7 / 2 | 17 | none 17 |
| `ring.lua` #20 Circle of Nostalgia | 7 / 2 | 14 | none 14 |
| `jewel.lua` #178 Militant Faith | 20 / 3 | 12 | one 12 |
| `jewel.lua` #79 Split Personality | 9 / 2 | 4 | one 4 |
| `jewel.lua` #97 Might and Influence | 6 / 2 | 4 | one 4 |
| `gloves.lua` #46 Tombfist | 11 / 4 | 3 | one 2, none 1 |
| `ring.lua` #16 Circle of Ambition | 25 / 3 | 1 | none 1 |
| `amulet.lua` #29 Eyes of the Greatwolf | 33 / 2 | 1 | none 1 |

## The catalysed items

61 items carry catalyst quality, of nine kinds — `Attribute` 27, `Elemental Damage` 9,
`Resistance` 6, `Caster` 5, `Physical and Chaos Damage` 4, `Defence` 3, `Attack` 3, `Life and Mana`
3, `Critical` 1 — and their catalysts rescale 114 lines across the entries they bind. 46 of the 50
that fitted nothing now fit exactly one; the remaining 4 fail on a line no variant carries, never
on a number. The six that fitted *unscaled* under the strict rule fit the same variant after the
rescale, so it confirmed rather than reassigned them — though for one, Presence of Chayula, the
entry carries no line its kind touches, and nothing of it was rescaled:

| Row key | Item | Catalyst | Lines rescaled | Fits, strict → final |
| --- | --- | --- | ---: | --- |
| `2c6946b8e87b` | Astral Projector, Topaz Ring | Attribute +20% | 1 | `only` → `only` |
| `d96cc8e4b6ce` | Dyadian Dawn, Heavy Belt | Resistance +4% | 2 | `Current` → `Current` |
| `8cd23d45205c` | Le Heup of All, Iron Ring | Attribute +20% | 2 | `Current` → `Current` |
| `e1f69b6dd103` | Mark of the Elder, Steel Ring | Attack +4% | 2 | `only` → `only` |
| `953aeba68faf` | Presence of Chayula, Onyx Amulet | Attribute +5% | 0 | `only` → `only` |
| `a0ca6bbb2a40` | The Torrent's Reclamation, Cloth Belt | Caster +20% | 1 | `Pre 3.16.0` → `Pre 3.16.0` |

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

## Left out, and provenance

Cut, all restorable from the scripts: the full `Source:` value census (five printed), the
`Upgrade:` chains, the `none` reasons by league, the icon-directory breakdown beyond the commonest,
and — from this pass — the strict-to-final transition printed as a full four-by-four table (one
cell of it is non-zero), the per-kind rescaled-line counts beyond the nine kinds listed, and the
four catalysed items still `none`, which the CSV carries by `moved_by` and `none_reason`. This page
still runs past the index's 12 KB; the next cut would be the *lacks* clause of each sources-read
row, which is the reviewer's call.

Every number above is printed by `variants.py`, `match.py` or `arrays.py`, which regenerate the
four `data/` files byte for byte, checked twice; `arrays.py` and `data/other-arrays.csv` are
untouched by this pass. The store copies, the clones and their commits, and the sibling-track files
read — including `../item-facts/data/properties-census.csv`, which the brief does not list — are
rows in `MANIFEST.md`. The sources-read table carries no count of its own: only the three scripts
print numbers, and its dates and commits are citations.

## Review

The reviewer's rows (2026-09-18, the session that wrote the briefs: first pass `f7ab0957`, second pass `5a89e710`); the text above is the runners', unedited. Rows 1–5 were written at the first pass (`1cf77994`) and their numbers are that pass's; a fate that the second pass changed says so.

| # | Finding | Fate |
| --- | --- | --- |
| 1 | Acceptance checked by hand: the three scripts rerun and the four `data/` files are byte-identical; 4,020 + 229 + 767 + 761 = 5,777; the Ashes of the Stars rows reprinted as the table states; no `data/` file carries the account name. | holds |
| 2 | The three catalysed Ashes of the Stars show `+19`, which is 16 at `+20%` rounded down — the top of `(10-16)`. The catalyst cross-count (46 of 152) is a floor on what rescaling would move, not a measure of it: no row was rescaled. | the owner asked for the rescale ("go ahead with both fixes including catalyst scaling"); the second pass ran it, row 7 |
| 3 | Open questions 1 and 2 name a read that is already on this machine: `../trade-query/data/stats-2026-09-12.json` has an `ultimatum` group (63 entries, display names such as `Choking Miasma II`, ids `ultimatum.umod_<n>`) and a `veiled` group (20, `of the Veil`, `Catarina's Veiled`, ids `veiled.mod_<n>`). Neither is keyed by what the item gives (`FrostInfection`, `Suffix02`), so the questions stay open, narrower: what joins the API's `type` to a `umod`, and a slot id to a `veiled.mod`. | open, restated here |
| 4 | "By rarity" in the arrays table counts lines, not items (`utilityMods` sums to its 3,225 lines); `enchantMods` sums to 2,939 of 2,948 lines and `veiledMods` to 67 of 68, so ten lines sit on items with no rarity the column names. | unlabelled unit; the CSV's `by_rarity` is the same |
| 5 | The page is past the index's 12 KB (rule 3). Nothing was cut to fit: the sources-read *lacks* clauses the runner offered are the finding. | budget tripped, recorded in the index row; further past it after the second pass, same reason |
| 6 | Second pass, acceptance checked by hand: scripts rerun, `data/` byte-identical; `other-arrays.csv`, `arrays.py`, this section and the two protected sections unchanged from `1cf77994`; the strict column is 4,020 / 229 / 767 / 761 with 171 either; the only off-diagonal cell is `none` → `one`, 70 rows, each with a `moved_by` (46 `catalyst`, 24 `selection`); scrub clean. | holds |
| 7 | The rescale moved all 46 catalysed out-of-range rows and reassigned none of the six that had fitted unscaled. Its rounding is Path of Building's (`formatValue`, floor toward zero), not a rule GGG states anywhere on this machine; the owner's three Ashes of the Stars at `+19` agree with it, and no catalysed row is left out of range, which is the evidence it has. | stands; the game's own rounding is an open read |
| 8 | The reviewer told the owner before the run that the selection fix would clear most of the 147; it cleared 24. The six Circle entries hold 102 of the rest, and each fails first on an implicit the entry does not list — 87 distinct lines over the 102 items (`unique-fit.csv`, `none_line`), against an entry that lists one implicit. The headline's "drift, not structure" does not describe these: the wording did not drift, the entry has no line to compare. Whether their explicit lines would fit a selection was not measured; a fit that sets implicits aside is a different rule, and a definition is the owner's. | the reviewer's estimate was wrong; the question is open, named here |
| 9 | The runner found that Path of Building's own descriptor test (`Item.lua` 675, equality against `Elemental` and `Physical and Chaos`) sets no catalyst from the property as the game spells it; the rescale here maps by what the items show. | a finding about the source, recorded |
