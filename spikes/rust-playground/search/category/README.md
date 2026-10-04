# category — the trade site's grouping above class, and what a captured beast is

Status: first pass complete — 2026-09-30; reviewed 2026-09-30: the three scripts regenerate `data/` byte for byte and every category id of the site's filter has its row (the brief, `9280f559`, `1cbb1db7`); second run — 2026-10-01, reviewed: the `mods` reading measured, agreeing with the lure reading on every beast of the copy; round j composed, 38 searches, its checks run again at the review (the brief, `2e712e60`); round j sat 2026-10-01 and read, 40 of 40 (j39–j40 replacing j12–j13): open questions 1–5 closed but Growing Wombgift, 6 kept; round k composed 2026-10-04, 363 searches for the owner's sitting (the brief, `213578b1`)

- 65 of the site's 68 leaf ids are a function of the export's class (58) or class plus a name rule on the base (7); the three beast leaves are not; the site put 30 fetched beasts under Red Beast and 10 under Yellow, and the count of the export's bestiary mods sorts all 40, the lure word agreeing on 20 (F1, F3).
- The copy's 22,623 live pc items: one id 22,455, several 167 (158 beasts, 9 breachstones), none 1 (a quest-item contract) — C105's buckets sum to the copy (F5).
- A captured beast has no base and no class in the game's data: none of its 70 base names appears in any of the export's 30 files. On the body the frame is Rare on all 158; Genus, Group, Family and the name sort nothing; the count of bestiary mods splits them 122/36 exactly as the lure word does, base by base (F3, F4).
- The site's data states membership by the dot alone: 9 prefix parents; 7 groupings (`weapon.one`, `weapon.dagger`, …) whose members nothing captured names; `map` and `currency` are both a parent and a class's own id (F2).
- Coverage apart from use: 20 leaves checked by all four surfaces, 15 by three, 32 by the site's coarse list alone, `memoryline` by the export alone; 6 leaves hold no item. One base truly disagrees (Growing Wombgift). `map.invitation` reads "Maven's Invitation" and all 165 invitations on the copy are other kinds (F1, F7).

## Question

What is the trade site's category of every item the owner's store holds — computed from the game's own class and reviewed rules in one place, never a second classifier (the build plan, rule 10; the grouping park in `decisions/search.md`) — and what is a captured beast in the private API: by which fact of its body are the site's three beast leaves (`monster.beast`, `monster.yellowbeast`, `monster.redbeast`) told apart, and does the game's own data give a beast a class at all? The owner at the first seat (`runs/seat-2026-09-26/REPORT.md`, section 4, V1): "I'd like to mimic the trade site as much as practically feasible, because that's how most players will think."

## How it was measured

`scripts/categories.py`, `beasts.py`, `crosscheck.py` and `round-j.py` print every number below and regenerate `data/` byte for byte. The rule under test is `../repoe/scripts/base-taxonomy.py`'s (`CLASS_TO_TRADE`, `NAME_RULES`), imported, not copied. An item's class is read the way `class.rs` reads it: the base as shown, a blight prefix read past, and the frame picking among a name's classes (mirrored in `scripts/common.py`). A beast is an item whose `descrText` is the bestiary help line, the same test Awakened PoE Trade uses (`Parser.ts`, lines 974–975). Inputs are in `MANIFEST.md`.

## Findings

**F1 — The ids by how they are produced** (`data/categories.csv`, one row per id).

| Produced by | Ids |
| --- | ---: |
| a class | 58 |
| a class plus a name rule on the base | 7 |
| a rule on the body; which leaf, unsettled | 3 (`monster.*`) |
| nothing: the union of the ids under it by the dot | 7 |
| nothing: members not in the site's data (groupings) | 7 |
| total | 82 |

The residue by id is the three beast leaves: they have no base in the export. Seven export names fall under several ids: Bone Armour (`armour.chest`, `gem.activegem`), Energy Blade (three) and five breachstones (`map`, `map.breachstone`). Against the site's coarse list, 3,378 export bases agree, 742 are not listed by the site, and 2 disagree: Bone Armour's gem side, and Growing Wombgift, which the export classes `RemovedItem` (so `graft`) and the site lists under `wombgift`. The name rule `Invitation` puts 20 export bases under `map.invitation`, whose text is "Maven's Invitation"; the site's `map` list holds five of them, four not Maven's.

**F2 — Parents and leaves in the site's own data** (`data/categories.csv`, columns `kind`, `dotted_parent`).

| Shape | Count | Ids |
| --- | ---: | --- |
| prefix parent (another id sits under `<it>.`) | 9 | weapon 23 under it, armour 6, accessory 4, gem 3, jewel 3, map 4, heistequipment 4, heistmission 2, currency 7 |
| of them also a class's own id in the export | 2 | `map` (text "Map"), `currency` |
| grouping: members named by no dot | 7 | weapon.one, weapon.onemelee, weapon.twomelee, weapon.dagger, weapon.onesword, weapon.onemace, weapon.staff |
| dotted, the top token no id | 5 | monster.beast, monster.yellowbeast, monster.redbeast, sanctum.research, sanctum.relic |
| neither a parent nor under one | 12 | flask, leaguestone, memoryline, card, logbook, tincture, corpse, idol, graft, wombgift, enshrouded, chart |

No leaf sits under two prefix parents: the prefix is the first token. Whether a leaf sits under two groupings cannot be read, since no captured file names a grouping's members: the saved page and its 5 bundles hold none of the seven ids. Of the 20 texts beginning "Any", 12 are parents or groupings, so "Any" marks nothing. The coarse list has 22 categories: `monster` and `sanctum` are not filter ids, and `memoryline` is a filter id with no coarse category. The one leaf-level answer of the site's on this machine is the 2026-09-13 fetch under `weapon.onesword`: 11 Thrusting One Hand Swords and 9 One Hand Swords, so that grouping holds members of both `weapon.basesword` and `weapon.rapier`. A parent can be more than its leaves: the readings place 1,724 of the copy's items at `map` itself (MapKey 1,178; Misc Map Items 542, Inscribed Ultimatum 539 of them; Vault Keys 3; one base under Map and MapKey 1). For the Misc Map Items and Vault Keys the only evidence is the coarse list.

**F3 — A captured beast in the private API** (`data/beasts.csv`, one row per base; `data/beast-facts.csv`, one row per body fact).

158 items of 70 bases, all in Standard, all in `Stash1`. All 70 are in the site's `monster` list (361 names) and in Awakened PoE Trade's `CAPTURED_BEAST` list (220 names, all on the site's list). Of 32 body facts, 13 vary: `baseType`/`typeLine`, `name` (a generated name, never the base), `ilvl`/`itemLevel`, `explicitMods` (count and texts), `Genus`, `Group`, `Family`, and `id`, `x`, `y`. Constant on all 158: `frameTypeId` Rare, `rarity` Rare, `descrText` "Right-click to add this to your bestiary.", the icon `BestiaryOrbFull.png`, the three properties in order (types 21, 22, 23).

| Candidate reading (`scripts/beasts.py`, docstring) | Captured Beast | Yellow Beast | Red Beast |
| --- | ---: | ---: | ---: |
| apt: every beast is one category, "Captured Beast" | 158 | 0 | 0 |
| frame: Rare yellow, Unique red | 0 | 158 | 0 |
| lure: base begins with a word the export names a lure for (Craicic, Farric, Fenumal, Saqawine) | 0 | 122 (52 bases) | 36 (18 bases) |
| mods: the wiki, quoted by the owner (`MANIFEST.md`): one bestiary mod of the export yellow, two red | 0 | 122 (52 bases) | 36 (18 bases) |

The two splitting readings agree on 158 of 158 items and 70 of 70 bases; no beast carries none or more than two bestiary mods. Against the lure reading, 3 of 4 Families, 7 of 12 Groups and 8 of 32 Genera hold both kinds. A `<lure word> Presence` mod sits on 11 of the 36 lure-word beasts and on 17 of the others. All 158 carry at least one of the export's 24 bestiary mods. The wiki's named red beasts are not on the copy: of the site's names, `Black Mórrigan` and four holding ", First of the " (the quote's "First Ones"; the owner's to confirm), all five yellow under the lure reading; no input names the Harvest beasts. Every reading agrees with the site at the coarse level, since all 70 bases are on its `monster` list; the site's answer at the leaf is round j's, below. Awakened PoE Trade sends no category for a beast at all: it searches the exact base (`create-item-filters.ts`, lines 59–65).

What the site said (round j; `data/round-j-beasts.csv`, one row per search, base, rarity and count; measured, no reading adopted — that is the owner's). Of 70 beasts fetched, 40 were found under a coloured leaf, of four bases: Farric Ursa, Black Mórrigan and Craiceann, First of the Deep under Red Beast (30), Dune Hellion under Yellow (10); none of the four under the other colour.

| Fact on the body | Red Beast (30) | Yellow Beast (10) | Sorts all 40 |
| --- | --- | --- | --- |
| the export's bestiary mods, count | 0: 10, 2: 2, 3: 18 | 1: 10 | yes |
| rarity | Rare 11, Unique 19 | Rare 10 | no |
| lure word on the base | yes 10, no 20 | no 10 | no (agrees on 20) |
| the wiki as quoted (1 yellow, 2 red) | red 2, no colour 28 | yellow 10 | no (agrees on 12) |
| Family | The Deep 10, The Wilds 20 | The Wilds 10 | no |

Genus, Group and the base also sort all 40, as any fact constant on a base does with four bases; the count varies within a base (Farric Ursa, 2 and 3) and still sorts. What would break it: a beast under Yellow with other than one of the export's bestiary mods, or under Red with exactly one; Yellow is one base here. The 30 found under Captured Beast alone carry 1 (Dune Hellion, 10), 2 (13) or 3 (Farric Ursa, 7), every base with 2 or 3 a lure-word base; every fetched beast's clipboard says `Item Class: Stackable Currency`.

**F4 — The game's data gives a beast no base and no class.** No beast base is an export base name (0 of 70). No top-level file of `poe1/data` (30 files) holds any of the 70 names as a string. The only class whose id, name or category says beast, bestiary, monster or captured is `PantheonSoul`, which it matches as "Captured Soul". What the export does carry is the beasts' mods: 24 `bestiary` mods by display name, the texts the body's `explicitMods` print (for example `Farric Presence`).

**F5 — Every live item of the copy placed** (`data/residue.csv`, every item not under exactly one id).

| Bucket | Items | Why |
| --- | ---: | --- |
| one id | 22,455 | — |
| several ids | 158 | a beast: which of the three leaves is not settled (70 bases) |
| several ids | 9 | a base under several ids: five breachstones, `map` and `map.breachstone` |
| no id | 1 | a class with no id: Contract: Follow the Paper Trail, a Quest Item |
| total | 22,623 | |

The breachstones' two ids are nested (`map.breachstone` under `map` by the dot) only if `map` is a union; its text says "Map" (F2). No item on the copy has a base the export classes and the shipped table lacks. Of the 165 items under `map.invitation`, none is Maven's (Polaric 81, Incandescent 41, Writhing 26, Screaming 17).

**F6 — The classes with no id.** The export has 31. The copy holds 1 class of them, 1 item: `QuestItem` (the contract above).

**F7 — Coverage apart from use** (`data/category-crosscheck.csv`, one row per id, source and level; `data/category-disagreements.csv`, one row per base and pair of sources whose ids differ).

| Leaves (68, `map` and `currency` included) | Count | Ids |
| --- | ---: | --- |
| the export, the site's coarse list, Awakened PoE Trade and Path of Building, each at the leaf | 20 | weapon.bow, .claw, .oneaxe, .sceptre, .twoaxe, .twomace, .twosword, .wand, .rod; armour.chest, .boots, .gloves, .helmet, .shield, .quiver; accessory.amulet, .belt, .ring; jewel.abyss; flask |
| the export, the coarse list and Awakened PoE Trade | 15 | weapon.runedagger, weapon.warstaff, accessory.trinket, jewel.cluster, map, the four heistequipment and two heistmission leaves, sanctum.relic, tincture, idol, chart |
| the coarse list alone besides the export | 32 | every gem, map.* and currency.* leaf, the three beasts, leaguestone, card, logbook, sanctum.research, corpse, graft, wombgift, enshrouded; and six leaves both tools reach only through a grouping or parent: weapon.basedagger, .basesword, .rapier, .basemace, .basestaff, jewel.base |
| the export alone | 1 | memoryline |

The site's coarse list checks only at the parent's level, so no leaf under a prefix parent is checked by the site at the leaf. Awakened PoE Trade places 1,172 bases: 1,054 at the export's id, 113 at a grouping over it, 5 nested by the dot, none different. It lacks 311 export bases at the leaves it reaches. It sends `azmeri.charm`, which the site's 2026-09-12 filter lacks. Path of Building places 1,080 bases: 909 the same, 142 at a grouping, 8 nested, none different, and 21 that are no export base (its variant entries, charms, placeholders); its grafts and tinctures get no id. The disagreements file lists 306 rows. Only one relation is `different`, Growing Wombgift (export `graft`, site `wombgift`); 7 share an id (a name under several ids), 298 are a grouping against a leaf, and 4,347 pairs nested by the dot are counted and not listed. Leaves with no item on the copy, whose fixtures the build cuts by hand: `weapon.rod`, `leaguestone`, `memoryline`, `logbook`, `graft`, `chart` — and the three beast leaves, which hold 158 items between them in an unknown split.

## Round j

Open questions 1–6 as `tools/trade-sheet.py`'s `round_j`, 40 searches, each with its pair (`data/round-j.csv`; what each decides is its sheet row). Sat 2026-10-01. `scripts/round-j-read.py` reads every capture (`data/round-j-read.csv`: totals, fetched bases and clipboard classes, the pair) from `../pseudo-stats/data/captures.json`, the class line from the raw fetches. A zero is read only beside a pair that finds the type.

| Question | Searches | Decided |
| --- | --- | --- |
| J1 | j01–j07, j35–j38 | each of four bases under one colour only and under Captured Beast (F3); the lure reading fails on Black Mórrigan and Craiceann (j35–j38), the wiki as quoted on 3-mod Farric Ursa |
| J3 | j08–j09 | `map.invitation` does not hold `Polaric Invitation` |
| J2 | j10–j11, j39–j40 | `map` holds `Inscribed Ultimatum` and `Anarchy Scarab` (684 under `map` and `map.scarab` alike) |
| J4 | j14–j28 | every member asked is in, but Fishing Rods under `weapon.twomelee` (j24 0, j18 2832) |
| 5 | j29–j30 | `map.breachstone` and `map` each find `Xoph's Breachstone` (5610 and 5610) |

Pairs that decided nothing, both sides empty: j12–j13 (`Winged Bestiary Scarab`), j31–j32 (`Growing Wombgift`), j33–j34 (`memoryline`, `Alva's Memory`). Totals of 10000 are the site's cap, three marked inexact: no union is read by count.

## Round k

No published list gives a beast's colour, so the site is asked (C106, clause (a)): `tools/trade-sheet.py`'s `round_k`, read back by `scripts/round-k.py` (`data/round-k.csv`). k001–k002 are J8's pair: armour with a defence twin text as an implicit under the `(Local)` stat (5 ids), the percentile required; its mutant the global implicit ids (3), which must find i05's ten. k003–k363 ask each of the site's 361 `monster` names under `monster.yellowbeast`: the copy's 70 bases first (k003–k072), the rest by name (k073–k363). A name found is yellow by the site's word; one not found is asked under `monster.redbeast` in the next round, its pair by rule. Each row records what round j's mod count would say (yellow 52, red 18, no item on the copy 291), to hold against the answer, never as it. 363 searches, 15 pages, 1 h 0 min 30 s at one link every 10 s; first page `../pseudo-stats/raw/sitting/k-1.html` (local).

## Open questions

Each is closed by a sitting under the site-sitting skill (the owner runs the searches in a browser; Standard, status any) or by the read named.

1. **A beast's leaf** — closed for the four bases asked (round j; F3); which body fact the search reads is the owner's. Whether `monster.beast` is exactly the union is unread: its totals are capped.
2. **The groupings' members** — closed: `weapon.onemelee` holds Sceptres and Rune Daggers, `weapon.twomelee` Staves and Warstaves and not Fishing Rods, `weapon.one` Wands, `weapon.dagger` Rune Daggers, `weapon.onemace` Sceptres, `weapon.staff` Warstaves.
3. **What `map` holds** — closed: a Misc Map Item of its own and a child's base.
4. **`map.invitation`** — closed: not `Polaric Invitation`; which invitations it does hold is unread.
5. **Breachstones and Growing Wombgift** — breachstones closed (both ids). Growing Wombgift kept: no listing under either id, none on the copy; a listing closes it.
6. **`memoryline`** — kept: nothing listed under it, and no `Alva's Memory` at all; a listed Memory's search under it closes it.
7. **`azmeri.charm`.** It is absent from the 2026-09-12 filter and sent by Awakened PoE Trade; the export's Charms have no id. The next capture of `/api/trade/data/filters` closes it.
