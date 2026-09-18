# legacy — Path of Building's unique variants against the owner's items

Status: first pass complete — 2026-09-18

- Of the 5,777 uniques the two stores hold, **4,020 fit exactly one** Path of Building variant,
  229 fit several, 767 fit none, and 761 bind to no entry at all. **171 fit both an old and a
  not-old variant** — the park's undecided case (C93).
- `none` is mostly structure, not drift: **147** of the 767 sit on entries whose `Has Alt Variant`
  header lets two variants be chosen at once, and **46** are a catalyst raising a roll past a range
  the entry was written without.
- The owner's Ashes of the Stars: 10 in the population — 5 `Current`, 2 `Pre 3.23.0`, 3 `none`,
  every one of the `none` catalysed — and 1 foil counted apart. None is `either`.
- **4,789 of 36,139 items (0.133)** carry a line outside implicit and explicit, over every rarity;
  helmets are 288 of the 1,686 `enchantMods` items. Two arrays give no text at all —
  `ultimatumMods` gives a mod id, `veiledMods` a slot id — and neither id resolves anywhere here.
- **No input on this machine says a line can no longer be made.** The nearest are `spawn_weights`
  in `poe1/data/mods.json` (all-zero means not rollable at that export), the two export histories
  back to 2019-11, and `Source: No longer obtainable` on 170 unique entries — which is about the
  item, never a line.

## What was measured

`variants.py` parses every `[[ … ]]` block of the clone's unique files at `16de4b82`; `match.py`
binds every unique in the owner's two stores to the entries of the same `name`; `arrays.py`
censuses every other mod array, over every rarity. Per-item detail is the CSV. Nothing here judges
whether a line can still be made — that is the sources-read table, every claim of which cites a
path in a clone at its commit (S178, R8). Five definitions were sharpened, none swapped:

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
  (lines 15, 673), so `Quality (Quantity)` on a map does not match; nothing is rescaled;
- **the frame type is a name, not an integer** in both stores: the population is `Unique`, and
  `SupporterFoil`, which also carries `isRelic`, is counted apart.

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
| Outcome: `one` / `several` / `none` / `unbound` | 4,020 / 229 / 767 / 761 |
| `either` — the fitting set holds an old and a not-old variant | 171 |
| Catalyst quality by outcome: `one` / `several` / `none` / `unbound` | 6 / 0 / 50 / 5 |
| `none` on a number outside every range / of those, with catalyst quality | 152 / 46 |
| Population items with a line outside implicit and explicit | utility 302, enchant 171, crucible 14, scourge 8, veiled 7 |
| Population items whose implicit or explicit lines carry a flag | mutated 78, crafted 9 |
| Distinct names binding to no entry | 108 |

By league, `one` / `several` / `none` / `unbound`: Standard 2,312 / 142 / 605 / 696; Solo
Self-Found 1,290 / 61 / 110 / 40; Hardcore 228 / 16 / 22 / 2; Allflame 190 / 10 / 30 / 23. The 761
`unbound` are mostly kinds the unique files do not carry — Heist contracts (`Contract: The Slaver
King` 62), unique maps (`Whakawairua Tuahu` 56) — then `Forbidden Flesh` and `Forbidden Flame`,
44 each, which `Special/Generated.lua` builds in code.

## Why a unique fits no variant

The reason is the first in the brief's order, measured against every candidate variant at once.

| Reason | Items | The commonest line |
| --- | ---: | --- |
| a displayed line with no text match in any variant | 513 | `Regenerate 8 Life over 1 second when you Cast a Spell` (Lifesprig, 28), against the entry's `… for each Spell you Cast` |
| a text match with a number outside every range | 152 | `+19 to all Attributes` against `+(10-16)`, 14 |
| a variant line the item does not display | 102 | `You can apply an additional Curse during Effect`, 14 |

Behind the first row sit `Extra gore` (14) and corrupted or eldritch implicits an entry never
carries. How near the 767 sit to a fit: **147** bind to an entry with a `Has Alt Variant` header,
which lets Path of Building select a second variant at the same time (`Item.lua` `hasAltVariant`,
lines 752-758) — the Circle of Fear / Watcher's Eye shape, where one item carries a line from each
of two axes and fits no single variant. **16** match once `increased` and `reduced` are exchanged:
Ventor's Gamble is written `(-40-40)% reduced Rarity of Items found`, and the game shows `17%
increased Rarity of Items found` when the roll is positive. **15** match once a trailing plural `s`
is ignored (`Gain 3 Frenzy Charges on use` against `Gain (1-3) Frenzy Charge on use`). And 46 of
the 152 out-of-range rows carry catalyst quality.

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
Both carry `+(10-16) to all Attributes`, `(5-10)% increased Experience Gain of Gems`, `+1 to Level
of all Skill Gems`, `+(20-30)% to Quality of all Skill Gems`. The store holds 10 in the population
and 1 apart. None is `either`: the Reservation Efficiency line decides each.

| Row keys | League | Fits | The lines that decide it |
| --- | --- | --- | --- |
| `32253274c7ad`, `4fe1659283f3`, `ac96c3d10ad1`, `b863a0d867f8`, `c6a47058fdc8` | Standard ×3, Allflame ×2 | `Current` | no Reservation Efficiency line; `+10`, `+13`, `+16 to all Attributes`; quality `+24%`…`+30%` |
| `8061f87d24e3`, `dc57fb4db12d` | Standard | `Pre 3.23.0` | `20% increased Reservation Efficiency of Skills`, inside `(10-20)`; `+16 to all Attributes` |
| `01fa6690afdc`, `a516661982fb`, `d8a9c2befa5d` | Standard | none | each carries `Quality (Attribute Modifiers) +20%` and shows `+19 to all Attributes`, outside `(10-16)`; two also show the Reservation Efficiency line, so the catalyst is all that keeps them from `Pre 3.23.0` |
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
- **Would the 147 `none` rows on `Has Alt Variant` entries fit a combination?** Re-run the fit over
  pairs and triples of variants there: settles how much of `none` is the one-variant-at-a-time rule
  and how much is real drift.

## Left out, and provenance

Cut, all restorable from the scripts: the full `Source:` value census (five printed), the
`Upgrade:` chains, the `none` reasons by league, the icon-directory breakdown beyond the commonest.
This page still runs past the index's 12 KB; the next cut would be the *lacks* clause of each
sources-read row, which is the reviewer's call.

Every number above is printed by `variants.py`, `match.py` or `arrays.py`, which regenerate the
four `data/` files byte for byte, checked twice. The store copies, the clones and their commits,
and the sibling-track files read — including `../item-facts/data/properties-census.csv`, which the
brief does not list — are rows in `MANIFEST.md`. The sources-read table carries no count of its
own: only the three scripts print numbers, and its dates and commits are citations.

## Review

The reviewer's rows (2026-09-18, the session that wrote the brief at `f7ab0957`); the text above is the runner's, unedited.

| # | Finding | Fate |
| --- | --- | --- |
| 1 | Acceptance checked by hand: the three scripts rerun and the four `data/` files are byte-identical; 4,020 + 229 + 767 + 761 = 5,777; the Ashes of the Stars rows reprinted as the table states; no `data/` file carries the account name. | holds |
| 2 | The three catalysed Ashes of the Stars show `+19`, which is 16 at `+20%` rounded down — the top of `(10-16)`. The catalyst cross-count (46 of 152) is a floor on what rescaling would move, not a measure of it: no row was rescaled. | stands as a count; rescaling is the owner's call |
| 3 | Open questions 1 and 2 name a read that is already on this machine: `../trade-query/data/stats-2026-09-12.json` has an `ultimatum` group (63 entries, display names such as `Choking Miasma II`, ids `ultimatum.umod_<n>`) and a `veiled` group (20, `of the Veil`, `Catarina's Veiled`, ids `veiled.mod_<n>`). Neither is keyed by what the item gives (`FrostInfection`, `Suffix02`), so the questions stay open, narrower: what joins the API's `type` to a `umod`, and a slot id to a `veiled.mod`. | open, restated here |
| 4 | "By rarity" in the arrays table counts lines, not items (`utilityMods` sums to its 3,225 lines); `enchantMods` sums to 2,939 of 2,948 lines and `veiledMods` to 67 of 68, so ten lines sit on items with no rarity the column names. | unlabelled unit; the CSV's `by_rarity` is the same |
| 5 | The page is past the index's 12 KB (rule 3). Nothing was cut to fit: the sources-read *lacks* clauses the runner offered are the finding. | budget tripped, recorded in the index row |
