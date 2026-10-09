# pseudo-stats — which mechanism answers each of the site's 298 `pseudo` stats

Status: 9d-A's expected answers, run and reviewed: the site disagreeing on none of 4849 captured cells, 7 kinds of cell unruled and none of them captured — 2026-10-08. The states before it, each by its commit: step 9d-C built and closed `ad1d081b`; 9d-C's expected answers final `5f834e8c`; round n sat and read `22380d71`; round n composed `2b86eef4`; before 9d-C's sitting, the domain and the twin `0a6f7d93`; 9d-C's expected answers `00610359`; J8's k001 and k002 read `da2c1711`; the first pass `6ea3277b`; the second pass, part 1 `55713506`; the pilot `9166e88b`, the checks `a80748b4` and rounds two to four `ecd682e2`, `41ed6467`, `c2fc1ba0`; the changes that need no twin applied, totals v2 `e81cc8ca`, a total of nothing lacked `37faf903`; rounds five and six (step 9c2) `b22dccbe`, `85f338e3`, applied at totals v4 `77685777`; the four that are no sum, steps 9c3 (totals v5) `7a315fbb` and 9c4 `64a6a3af`; before 9d (the brief at `9280f559`), round i prepared and the ranged census `cb77add0`; round i read `b39e0799`; i07 `c58d3050`; open question 9 `3cf6893e`. The line as it stood, whole: this file at `da2c1711`.

Headline:
- **Part 1: 1,655 candidate pairs** (pseudo, line template) over 40 of the 58 pseudos in scope:
  2 `site`, 5 `sources-differ`, 15 `one-source`, **0 `sources-agree`**, 1,633 `text-only`. No tool
  names any contributor of the 21 `Adds # to #` pseudos; in the scope, no two tools name one line.
- **The percentile is the site's own stated rule**: its filter tip reads "The percentile of all base
  defence rolls, averaged". Rolls recovered as integers from the display reproduce **30 of 30**
  captured values, each a single value; APT's formula gives 20 exactly and 29 within rounding;
  every negative control fails. A private item displays everything the rule reads.
- **The site is the checker** (335 searches naming a pseudo captured; 1,648 readings, none disagreeing): of 86
  pseudos 42 are closed — a complete and a sound check each found nothing — and 44 explained:
  what a check found shows, item for item, what the rows give. `data/table-changes.csv` is the
  totals table's changes as rows, 376, applied (totals v6); the four `no sum` are written as readings.
- **The ranged family is one reading**: the site shows the average of a line's two numbers; a
  weapon's own line feeds the attacks' pseudo alone, the other twin the plain, the attacks' and
  the spells'.
- `+#% to All Resistances`, which APT names and the C++ tables leave out, is counted: one item
  shows it under the fire, lightning and chaos totals (e042, e044, e048).

Per-row detail: `data/candidates.csv` (`scripts/candidates.py`), `data/percentile-check.csv`
(`scripts/percentile.py`), `data/search-sheet.csv` (`tools/trade-sheet.py`), `data/captures.json`
(`tools/trade-captures.py`, the owner's captures scrubbed), `data/evidence.csv` (`tools/trade-evidence.py`).
Each regenerates byte-for-byte; run `percentile.py` before `candidates.py`, which reads its output.

Moved from this page when it passed its guide, verbatim: `data/part1-detail.md`, `data/sittings-detail.md`;
the page before the move is this file at `da2c1711`.

## Part 1: the candidates

The scope, the statuses, the three sources and the rule that puts a template in reach of a pseudo are
`scripts/candidates.py`'s docstring and its `WORDS` table; the script prints the counts by status and
group, the families by shape and the ranged templates by what the export says is local. The same as
tables, with what admits each status, the searches counted by form and what part 1 left out:
`data/part1-detail.md`.

- `site`, 2 pairs: `fetch-census.json`'s weights reproduce every q4 item (life 1, Strength 0.5).
- **256 families hold 998 `family:` rows**: a template of a conditional shape keeps its row, and its
  family names one representative for the sheet.
- Not run, and one command away (`tools/trade-sheet.py --method if`): the rows named L, P, C and R below and in the open questions are that sheet's.
  By form: 264 searches by `--method if`, 565 by `--method and`.

## The percentile

| Formula | Stated by | Exact | Within its rounding |
| --- | --- | ---: | ---: |
| `apt`: the first of armour, evasion, energy shield, ward; round(100 × (D ÷ (1+q) ÷ (1+inc) − flat − min) ÷ (max − min)) | APT `renderer/src/parser/calc-q20.ts` L116–L127, `Parser.ts` L1266–L1282 ("the same for all defences") | 20 | 29 |
| `pob`: the same quotient per type, 0–1, four places; no joining rule | PoB `Item.lua` L2346–L2385 | — | 30 averaged |
| `site`: each roll's percentile, averaged, rounded half up; rolls = the integers whose display (half up) is D | the site's tip, `grammar.json` `filter_groups[3].filters[5]` | **30** | 30 |

Negative controls, one part of the site's rule changed: display floored — 10 items admit no roll,
16 of 30; ceiled — 5 admit none, 24; hybrids joined by min, max or first type — 5 of 7; average
rounded half to even — 28 of 30; local lines not read — 0 of 23; quality not read — 0 of 8. The
site's own 20%-quality figures (`extended.ar|ev|es`) follow from the recovered roll on 30 of 30,
so quality multiplies, as both tools state.

APT's misses (site / APT): Carnal Boots q2[0] 76/78 and q2[4] **82/89, outside its 86–92** — the
evasion roll is 88.9%, the energy shield roll 75%, and the site averages them; APT's "same for all
defences" is contradicted by both. Of the other eight, seven differ by one point and Samite
Slippers is 0/6 (quality 9, the minimum roll): APT divides the rounded display instead of
recovering the integer roll.

What the captures do not pin: 15 of the 30 are 0, and only 2 hybrids discriminate the join. No
capture has three defence types, ward, a shield, a helmet or gloves, quality above 20, the
`Quality does not increase Defences` enchant, a unique's base, a reduced local line or the triple
`#% increased Armour, Evasion and Energy Shield`: cases C1–C8 and the percentile lines P001–P034.

## The percentile's round

`percentile.py` now scores the sittings' captures too (`data/percentile-captures.csv`, apart, as
`candidates.py` reads `percentile-check.csv`): of 637 distinct items the stated rule reproduces 634;
a display at an exact half read as either integer (J5), 637, each one value (half down: 629). Of 292
hybrids, min, max and first type give 173, 179, 181. The 20%-quality figure from the recovered roll:
812 of 812 rows, the enchant read as no quality. Read from text alone, as the build must, the items no
longer all read as from their ids: 10 of 637 do not, each carrying a defence twin under the stat its
text does not show — i05's eldritch implicits, k001's synthesised ones (`percentile-shapes.py`).

Round i (`tools/trade-sheet.py`, `round_i`): the percentile's read by `percentile.py`. The core is i01–i08.

| Search | Shape | Decides | Copy / captured | Read 2026-10-01 (`percentile.py`, round i by search) |
| --- | --- | --- | --- | --- |
| i01 | C1 | three rolls averaged, Sacrificial Garb (the copy's base) | 3 / 0 | **pinned**: 10 rows (1 a repeat), 10 reproduced; min 2, max 2, first type 4 |
| i02 | C7 | `Quality does not increase Defences`: the percentile, the 20%-quality figure | 2 / 0 | **pinned**: 10 of 10 reproduced with the quality term dropped, the figure 10 of 10; quality read after all, 0 and 0 |
| i03, i04 | C10 | whether the site shows over 100; i04 at 100 its mutant | 0 / 0 | **in part**: i03 found 0, its mutant i04 10000; i04's 10 rows reproduced, the highest 100, 0 with no roll in range: none shows a roll over the maximum, so whether the site clamps one is not pinned |
| i05 | C14 | a defence line under its global twin (text cannot tell): read or not | 1 / 0 | **pinned, left out**: 10 of 10 from the ids; 5 carry a twin of a type they read, and read as local, as text reads them, 0 of 5 |
| i09 | C2 | ward on any ward base | 13 / 2 | **pinned**: 10 rows, 10 reproduced, 9 of them by their ward roll |

J6's shapes: C1, C2, C7 and C14 are pinned; C10 is not. The site shows no armour item over 100, and
no fetched item needs a roll over its base's maximum: what closes it is such an item, one whose display
no roll in range gives, fetched at 100 or not found at 101.
The ranged searches i06–i08, by search: `data/sittings-detail.md`.

The defence twin on a private item (open question 9, `scripts/defence-twins.py`,
`data/defence-twins.csv`; i05 and J8's k001 and k002): over the captures, 17 twin lines the site lists
under the global stat sit on 17 items, 452 under the local one on 330. The array no longer separates
them: k001, armour with a twin text as an implicit under the `(Local)` stat, found 10000, and its 10
fetched are each a synthesised item whose implicit is the local twin (10 lines on 10 items). What
separates is the kind of line: an implicit on an eldritch item (`searing` or `tangled`), 17 global and
0 local, joined by its text to the export's eater of worlds implicits, whose 36 mods of a twin text are
all global; an implicit on a synthesised item, 0 global and 10 local, which no synthesis mod in the
export holds; an explicit, 0 global and 442 local on 326. k002, the mutant over the global implicit ids,
found i05's shape again: 7 eldritch items new to the captures, 3 repeats. No captured implicit twin is
on an item of neither mark, and none on the copy is. On the copy the eldritch kind touches 1 item
(Destiny Leather, C14's trace), the synthesised 8 (9 lines), the explicit 921; 3 explicit lines the
export's text join holds only as global.

Left out, answered by the captures (captured / reproduced): C3 shields 23 / 23, C4 helmets 235 / 233,
C5 gloves 152 / 152, C6 quality over 20 12 / 10, C8 uniques 176 / 176, C9 a defence the base lacks (its
own types read) 39 / 39. Not the site's: C11 poe2, no table (ruled); C12 an exact half, no filter
selects it; C13 a `reduced` local line, none on the copy, 1 captured and reproduced. The copy's 2,272 armour items:
2,240 read to a value, 22 poe2, 9 at an exact half, 1 under its minimum (C14's trace); 20 bases not in
the class table. Per shape: `data/percentile-shapes.csv`.

## The percentile's expected answers

`scripts/percentile-expected.py` (the brief, `BRIEF.md`): one function from a private item's realm, base,
properties, marks and lines — text, array, flags, never an id — to what `pseudo.defence_pct` answers, in exact
arithmetic, each of its 35 steps citing its sentence. `data/percentile-expected.csv`, a row a case;
`data/percentile-expected-copy.csv`, the function over the copy's 22721 live items.

- **Cases**: captured 4849 (the 4819 fetched rows of 864 searches and `fetch-census.json`'s 30 carrying the
  value, its other 40 left out; 3622 distinct items, 1227 repeats), ruled 45 (20 items of the copy, 25
  composed), worked 4 (the function gave what the hand did on 4 of 4). Every step is taken by a case, 14 by
  no captured one.
- **Disagreements**: 0 of 4849 cells (the site a value on 896, none on 3953); 0 not comparable. J5 answers 3
  distinct items (e014, h015, n22).
- **Unruled**, 7 kinds of cell, none captured, each with its readings: `realm-no-defence`, an item of a realm with no
  table displaying no defence (76 on the copy) — undecided as the realm's, or lacked; `crucible-line`, a
  defence line in `crucibleMods` (1 on the copy, El'Abin's Visage: 94 or 462) — the item's own, or left out
  as J29's; `twin-other-array`, a twin text outside `explicitMods` and `implicitMods`, and `twin-both-marks`,
  an implicit twin on an item eldritch and synthesised — own, left out or undecided; `base-type-not-shown`, a
  base ranging a defence the item does not display — undecided, or the displayed averaged; `flat-range`, a
  base whose minimum is its maximum (Kaom's Plate, Kaom's Greaves) — undecided, left out of the average, or 0
  or 100; `several-values`, a display several rolls give, their values differing — undecided, or an interval.
- **Broken once for each rule**, the captured cells that then disagree / are not comparable, of 4849:
  `lacked-is-zero` 3953 / 0, `lines-not-read` 593 / 14, `explicit-twin-left-out` 450 / 6, `average-floored`
  317 / 0, `first-type-only` 169 / 0, `quality-not-read` 132 / 26, `flat-after-increase` 90 / 70,
  `display-floored` 40 / 228, `average-half-even` 19 / 0, `ward-not-read` 13 / 0, `unflagged-only` 12 / 1,
  `enchant-ignored` 8 / 2, `eldritch-twin-own` 6 / 6, `increases-multiply` 3 / 1, `synthesised-twin-left-out`
  3 / 0, `reduced-as-increased` 1 / 0, `held-twice-other-row` 1 / 0, `base-lacks-type-undecided` 0 / 40,
  `j5-off` 0 / 7, `local-only-explicit-only` 0 / 1. **On the ruling alone**, 9, each 0 / 0: `realm`,
  `property-unread-ignored`, `base-lacking-is-lacked`, `array-unread-ignored`, `whole-text`,
  `flag-unread-opens`, `unmarked-twin-own`, `twin-of-no-roll-opens`, `range-bound`.
- **Not asked**: of the copy's 74 groups of lines 32, holding 91 of 1822 (each with its question in the copy
  file); no shape. An explicit twin the item's own (J8) under a flag — crafted 8 groups, 28 lines; fractured
  6, 13; mutated 1, 2 — or on a synthesised item, 1, 4. A text a local stat alone displays (the join): crafted
  5, 15; fractured 3, 6; on a synthesised item 4, 14. A line of a type no roll reads, 3, 7. A synthesised
  implicit `# to maximum Energy Shield`, 1, 2. Of each: does the site show the reading's value of such an
  item? By the rule's letter and no capture's: a roll over the maximum reads over 100 (C10, open), one under
  the minimum below 0, an unmarked implicit twin is undecided — what does the site show of each?
- **The copy**: its 2272 items displaying a defence are `percentile-shapes.py`'s — 2249 a value, 22 poe2
  undecided, 1 unruled. Of the 9 that script left at an exact half, 6 are J5's and 3 the stated rule's own:
  their product is a half that rounds up, which a float puts under it.

## The ranged family on the copy

`scripts/ranged-census.py`: 703 live unsuffixed lines on 559 items, one poe2's (no class); by class,
rarity, array and flags `data/ranged-census.csv`; each line on a unique, with the mods the text join
found, `data/ranged-uniques.csv` (339 lines, 95 uniques).

| Lines | On a weapon | Not a weapon |
| --- | ---: | ---: |
| all live | **573** | **129** |
| on uniques, export local: site local / global / not captured | 95 / 0 / 93 | 0 / 0 / 0 |
| on uniques, export global: the same | 0 / 0 / 0 | 0 / 26 / 4 |
| on uniques, export cannot say: the same | 17 / 0 / 19 | 0 / 69 / 16 |

The class decides as the export and the site do wherever either speaks. The limit: 36 weapon lines
the join cannot place (29 match local and global mods, 7 none), 19 uncaptured; i06–i08 found no listed
weapon carrying the line under the global stat, which says nothing of a weapon no one lists.
i06–i08 by search: `data/sittings-detail.md`.

## The ranged family's expected answers

`scripts/ranged-expected.py` (the brief, `BRIEF.md`): one function from a private item's realm, base,
rarity and lines — text, array, flags, never an id — to what each of the 21 totals answers; the rows are
`data/table-changes.csv`'s, 109 `add row` and 26 `twin`, each `global` (96 and 52 before ruling 2).
`data/ranged-expected.csv`, a row a case and a total; `data/ranged-expected-copy.csv`, the function over the
copy's 22623 live pc items.

- **The rulings**, 2026-10-08; no cell is `unruled`. 1: a total leaves out a crucible node's lines and a
  flask's own, the arrays `crucibleMods` and `utilityMods` and no other — the owner, of both: "Leave both out."
  2: where both twins of the unsuffixed line count toward a total at one weight, its row is the line whatever
  the item is; a side left open answers the attacks' totals, undecided under the plain and the spells'.
  3: a ranged total is nothing only where its low and its high both are. 2 and 3, the owner's assent: "yes
  and yes". J32, at the build: a flag that is no yes or no opens no total, no row asking one — three cells
  of `r-unread-flag` a value ("yes, i agree").
- **Cases**: captured 569 (458 distinct items, 111 repeats; of 112 searches, 9 of them round n's, 57 read
  and 55 left out as fetching no item), ruled 26 (15 items of the copy, 11 composed), worked 3 (the function gave what
  the hand did on 21 cells of 21, each). Every one of the 135 rows is counted by a case.
- **Disagreements**: 0 of 619 captured cells (the site a value on 459, none on 160); 0 not comparable.
- **Broken once for each rule**, the captured cells that then disagree, of 619: `weapon-own-as-other` 87,
  `other-as-weapon-own` 61, `low-for-avg` 459, `nothing-is-zero` 160, `first-line-only` 15, `explicit-only`
  300, `unflagged-only` 3, `no-local-row` 54, `no-eldritch-rows` 279, `no-spells-and-attacks-rows` 37,
  `no-aggregate-rows` 140, `crucible-counted` 20. **On the ruling alone**, 8, each 0: `realm`,
  `side-open-as-other`, `frame-picks-none`, `whole-text`, `unread-ignored`, `utility-counted` (the copy's
  2337 flasks hold 0 lines a row names), `pair-asks-side`, `cancels-is-lacked`.
- **Not asked**: of the copy's 71 groups 48, holding 258 lines of 1831 (each with its question in the copy
  file). Ruling 1, a type no capture asked, 8 groups, 35 lines, `crucibleMods`: the reading, none. J7, a
  weapon's own line, 10 groups, 18 lines — fractured 8, `implicitMods` 6, crafted 3, `enchantMods` 1. The
  other twin as an implicit, 1 group, 1 line. A row of `any` twin, 29 groups, 204 lines.

## Before the sitting: the domain, and the twin from the export

`scripts/evidence-domains.py` (`data/evidence-domains.csv`): `tools/trade-evidence.py`'s 1503 readings, each
made again with one domain's lines off the item; the counts here are the file's at `0a6f7d93`, before round n's
captures, and the file as it regenerates holds round n's. `Not shown` is no evidence a domain is left out.

- **Shown counted**, 7, by readings broken: `explicit` 489, `implicit` 732, `crafted` 9, `scourge` 4,
  `enchant` 2, `fractured` 1, a line with no hash 1. **Shown left out**, 0. **Not shown**, 7, no line
  captured: `crucible`, `veiled`, `delve`, `ultimatum`, `sanctum`, `imbued`, `mercenary`. Unflagged
  `explicitMods` holds `explicit`, `ultimatum` and hashless lines; the other five captured map one to one.
- **The ranged 21**: `explicit` counted under 16 (5 plain: a weapon's own, the same either way), `implicit`
  14 (1 the same, 6 uncaptured), `crafted` 1; no other domain.
- **The copy**, 19151 lines in 280 groups: 17987 shown counted (`explicitMods` 14823, `implicitMods` 2427,
  crafted 557, fractured 159, `scourgeMods` 17, `enchantMods` 4); 1164 not shown, uncaptured —
  `utilityMods` 1084, `crucibleMods` 77, crafted and fractured 2, `mutated` 1.

`scripts/ranged-twins.py` (`data/ranged-twins.csv`): 702 lines, four readings each — the generation types
the array names or every mod, any class or those the base admits; the unique rows are
`data/ranged-uniques.csv`'s. On a weapon (573) the export says local of 532, 531, 447, 481 and global of
none; off one (129), global of 36, 36, 19, 34. All four say the side's twin on 461, cannot say on 240. **One
differs**: a rare Gold Ring's `Adds 19 to 24 Cold Damage`, local under every mod and any class alone.

Path of Building @ 16de4b82, read apart from the export: the line is parsed with no item in hand
(`src/Modules/ModParser.lua` L129, L6818–L6824) and is a weapon's own where the base is a weapon and
nowhere else, whatever array it came from (`src/Classes/Item.lua` L2204, L2288–L2300); the scoped forms
carry a keyword flag and are never local. Fishing rods are weapons there, as in the category table.

**For a sitting**: an attacks' and a plain `Adds` pseudo of a weapon whose line is `crucible` (43),
`fractured` (8), `implicit` (9, one type captured), `crafted` (3), `enchant` (1); a plain one of an implicit
off a weapon (8); a resistance or speed pseudo of a flask's `utilityMods`; any of a `mutated` line.

## Round n, composed

`tools/trade-sheet.py`, `round_n`: 23 searches, none sent; one page, `raw/sitting/n-1.html`. A mutant is its
search with the pseudo shown (`if`), its ten fetched the reading. The core, deciding a row of the build: n01–n15.

| Search | Question | Items found | None found | Mutant |
| --- | --- | --- | --- | --- |
| n01 | 1: a crucible `#% increased Spell Damage` node, none of the total's rows, no pseudo | the node is not counted | a crucible line is counted | n02 |
| n03 | 2: a weapon's crucible `Adds # to # Fire Damage`, no attacks' pseudo | not counted | counted | n05 |
| n04 | 2: the same, no plain pseudo | the weapon's own, or not counted | the other twin | n05 |
| n06, n08 | 3: a flask showing the movement speed, the chaos resistance total | a flask shows a pseudo | its own line is not counted | n07, n09: Quicksilver, Amethyst |
| n10, n13 | 4: a helmet, a ring: the implicit cold line, no `(Local)`, no plain pseudo | not counted | counted off a weapon | n11, n14 |
| n12, n15 | 4: the same, an implicit under a `(Local)` stat | a weapon's own stat off a weapon | the class decides | n11, n14 |
| n16, n18, n20, n22 | 5: fractured (life), enchant (spell damage), scourge, crafted (total resistance), sound | a line not counted, or cancelled | counted | n17, n19, n21, n23 |

## Round n, read (2026-10-08; `tools/trade-split.py`, `tools/trade-evidence.py`)

All 23 captured, each the sheet's query. Found, a search beside its mutant:

| Question | Found | The site says |
| --- | --- | --- |
| 1: a crucible spell damage node | n01 1245, n02 1245 | **not counted**: n02's ten show no pseudo, the node's line under no stat |
| 2: a weapon's crucible `Adds # to # Fire Damage` | n03 3744, n04 3744, n05 3744 | **not counted**, under the attacks' pseudo or the plain: n05's ten show neither |
| 3: a flask's own line | n06 0, n07 10000; n08 0, n09 10000 | **not counted**: no flask shows either pseudo; the line is `utilityMods`, under no stat |
| 4: the implicit twin off a weapon | n10 0, n11 10000; n13 0, n14 622 | counted by the plain pseudo, ten of ten each |
| 4: an implicit under a `(Local)` stat off a weapon | n12 0, n15 0 | none listed on a helmet or a ring |
| 5: fractured, enchant | n16 0, n17 10000; n18 0, n19 920 | counted |
| 5: scourge, crafted | n20 2, n21 3947; n22 3, n23 10000 | counted: the five found are each a sum of nothing, the domain's lines cancelled by another's |

The rows leave out both arrays, `crucibleMods` and `utilityMods`, by the owner's ruling, 2026-10-08: "Leave
both out." 0 of 1648 readings disagree (`data/evidence.csv`); they were 80. A mutant each: `--count
crucibleMods` 60 disagree, `--count utilityMods` 20.

## The captures, and what the site said

A line-by-line search reads ten items, and a pseudo's rows are a claim about every item. Rows are
**complete** when no listed item shows the pseudo while carrying none of them, **sound** when no
listed item carries one while showing no pseudo: each is one search over everything listed. A
search that finds nothing proves nothing until its mutant, the same search with one row changed,
finds something (c2, c4, d02). The rows are `tools/trade_rows.py`, versions appended and never edited;
every capture is read against each pseudo's latest (`tools/trade-evidence.py`, `data/evidence.csv`).

Seven rounds, each a commit and its recordings a block of `MANIFEST.md`: the pilot (p1, p2) `9166e88b`;
the checks (c1–c6) `a80748b4`; two (d01–d24, b1, b2) `ecd682e2`; three (e001–e107; e047 and e051 refused
as too complex) `41ed6467`; four (f001–f073) `c2fc1ba0`; five (g001–g044, R1) `b22dccbe`; six
(h001–h053) `85f338e3`. What each asked, as a table: `data/sittings-detail.md`.

What the site counts, as the rows now stand:

| Finding | Evidence |
| --- | --- |
| **A row's two eldritch forms are counted as the row**: `While a Unique Enemy is in your Presence, …`, `While a Pinnacle Atlas Boss is in your Presence, …` | every complete check that found items found these and nothing else; left out, 197 readings disagree |
| **A sum of nothing shows no total** | c3, d01, e043, f005: lines that cancel |
| **A ranged pseudo is the average of a line's two numbers**, shown in both places | every reading of the family |
| **A weapon's own line** — the `(Local)` twin — feeds the attacks' pseudo alone; **the other twin** feeds the plain, the attacks' and the spells'; `to Spells and Attacks` feeds both; an aggregate is its types' rows together | rounds two to four; the scoped pseudos' sound checks found nothing |
| `#% to All Resistances` counts toward each resistance total | e042, e044, e048 |
| **One text, two ids, one counted**: `+# to Strength and Intelligence` on That Which Was Taken is left out of total life, Strength and Intelligence | c3, b1, b2, e053, e057 |
| `+# total to Level of Socketed Skill Gems` does not count `# to Level of Socketed Skill Gems` | e103: 10,000 found, the ten fetched one unique; f030, f031: without the row, complete and sound, nothing found |

What rounds five and six added, of the 29 other totals and of every total (each version's `why`
in `tools/trade_rows.py` names its captures and counts):

| Finding | Evidence |
| --- | --- |
| **Four pseudos are no sum of lines**: the two counts count the resistance totals an item shows, chaos among them, never lines; `total to all Elemental Resistances` and `total to all Attributes` are the least of their three totals, shown where all three are | R1, g001, g002, h040; g003, h039; h001, h002: 110 readings, none disagreeing. Read from a sibling, unasked: the count of all four on an item with no resistance, the elemental least on an item with one or two of its totals |
| **A row's `reduced` spelling is the row, below nothing**: the line is under the `increased` spelling's id | 60 readings of the 61 that carry one, the other a sum of nothing; left out, 59 disagree |
| Total mana is total life's rows with Intelligence for Strength, the twin id left out of it too | g005, h003, h004 (3,295 found, b2's 3,297) |
| A type's `increased` damage counts elemental damage; its spell damage, spell damage too; its damage with attack skills, elemental damage with attack skills too; burning damage counts fire and elemental damage | g014–g035, each tested complete and sound at round six |
| The two leech pseudos count the line under all ten ids, the `(Local)` twin among them | g039–g042 found nothing; h041, the bound's mutant, found 10,000 |
| Life regenerated is shown a tenth over the line's text on two items of ten: the site rounds what the text cuts | g037 |

What each finding means for the build — the rows it adds and their counts, the owner's ruling in his
words — is `data/table-changes.csv` (`change`, `evidence`) and `../LEDGER.md`; as this page told it,
finding by finding, `data/sittings-detail.md`. A weapon's own line is answered by the item's class, the
export spawning the `(Local)` stats on weapons alone: 52 rows mean a twin.

Two things the searches could not do, each a limit of what is closed (whole in `data/sittings-detail.md`,
with the mutants that show the reading can fail and the most a sitting used of what the site allows):

- **A `not` on an eldritch form's id lets some items through.** So 24 pseudos are explained and not
  closed: nothing new was found among what was fetched, and what was not fetched is unread.
- **A weighted group is refused past some cost** (56 and 4 an id; taken at 23 ids, refused at 41),
  so a total of many rows is asked sound in parts, and a part finds items whose whole sum is nothing.

## Candidate claims

For `docs/design/trade-ground-truth.md`, authored master-side; each is what the captures show, dated
2026-09-26 where no date is given, Standard, PC.

| Claim | Evidence |
| --- | --- |
| A search's id is its query: compact JSON, deflate, a fixed ten-byte gzip header, base64url without padding. The site takes a link composed so, and returns the query less every `"disabled":false` | `tools/trade-sheet.py --self-test`, 8 ids; 212 composed links opened |
| The site shows a pseudo's value only where the query names it; an `if` group shows every member's and filters nothing | q4, q5; p2, c5 |
| A pseudo is the sum of the displayed numbers of the lines it counts, each at its weight; a sum of nothing is no pseudo | every reading; c3, d01 |
| A ranged pseudo is the average of a line's two numbers, shown in both places | round two |
| A line's two eldritch forms are counted as the line | rounds three and four |
| Two stats displaying one text are counted apart: the `(Local)` twin toward attacks alone; one twin id of `+# to Strength and Intelligence` toward nothing | rounds two to four; c3, e053, e057 |
| A `not` on an eldritch form's id lets some items carrying it through | round four, 227 items |
| Rate limits, searches: 3 in 5 s an account; 8 in 10 s, 15 in 60 s, 60 in 300 s, 600 in three hours an address. Fetches: 6 in 4 s an account; 12 in 4 s, 16 in 12 s, 100 in 300 s, 1,000 in three hours an address | the recordings' `X-Rate-Limit-*` headers |
| A weighted group costs a `complexity` of 56 and 4 an id; a query is refused, status 400, "Query is too complex", somewhere between 148 and 220 | d01, d02, e039–e063; e047, e051 |
| A line's `reduced` spelling is under the `increased` spelling's id, and a pseudo counts it below nothing: `-10% total Attack Speed` over `10% reduced Attack Speed` (2026-09-27) | 60 readings of the 61 that carry one; g007, g034, h042–h052 |
| Four pseudos are readings of other pseudos and no sum of lines: the two counts are how many resistance totals an item shows, chaos among them; `total to all Elemental Resistances` and `total to all Attributes` are the least of their three, shown where all three are (2026-09-27) | R1, g001–g004; h001, h002, h039, h040 |
| `# Life Regenerated per Second` is rounded where the item's line is cut to a tenth: 1.2 over `1.1` (2026-09-27) | g037, two items of ten |
| A weighted group takes a maximum below nothing, and a minimum of a hundredth (2026-09-27) | h042–h053, 12 searches answered; g040, g042, h038, h041 |
| A crucible node is one stat with its numbers and tier in its text, and a fetched item's crucible lines are under no stat; no pseudo counts them (2026-10-08) | n01–n05: 1245 and 3744 items found carrying a node and showing no pseudo, ten fetched of each |
| A flask's own line, its `utilityMods`, is under no stat and no pseudo counts it (2026-10-08) | n06–n09: no flask shows the movement speed or the chaos resistance pseudo; ten Quicksilver and ten Amethyst fetched, none showing one |
| Fractured, enchant, scourge and crafted lines are counted by a pseudo as explicit ones are (2026-10-08) | n16–n23, each sound check finding nothing or a sum of nothing |

## First pass (2026-09-18; `data/pseudo-classes.csv`, unchanged by this pass)

| Class | Entries | Admitted by |
| --- | ---: | --- |
| `field` | 123 | `property_type_to_field` (13), `properties-census.csv` (103: 86 rooms, 17 quality), `field-census.csv` (7 influence) |
| `unresolved` | 109 | nothing; the `note` column names the read |
| `sum` | 36 | `pseudomods.toml` name match (34), the site's own `pseudoMods` line (2) |
| `ranged-total` | 15 | the same ranged template in `mod-templates.csv` |
| `mod-behind-line` | 14 | the fetch's `mods` list (12 modifier counts, 2 implicit tiers) |
| `count` | 1 | `# Notable Passive Skills`, over 196 `# Added Passive Skill is …` templates |
| `computed` | 0 | — (overtaken: the percentile is reproduced above) |
| **Total** | **298** | |

Against the previous script (`../trade-query/scripts/classify-pseudo.py`, which left none
unresolved) 116 labels changed and 182 agreed; 63 `field` rows rest on a sibling (`family:`).
The first pass's full text is this README at `d18fa6dd`.

## Open questions

| # | Question | The one read that closes it |
| --- | --- | --- |
| 1 | Do the 13 `unresolved` entries whose text matches a template exactly (movement speed, rarity, leech, mana regeneration) sum those lines? | **Closed 2026-09-27** (rounds five and six): each sums its own text, and eight of the fifteen whose text a line displays other lines too |
| 2 | Which lines does each `Adds # to #` pseudo sum? | sheet rows L002–L009, L012, L013, L016, L021, L032, L045, L067, and the families they represent |
| 3 | Is `#% Base Defence Percentile` computable from the export? | **Closed 2026-09-26** on the captures: the site's stated rule, rolls recovered from `../repoe/data/base-defences.csv`, reproduces 30 of 30; what no capture reaches is C1–C8 |
| 4 | What do a logbook and a lake tablet look like in the private API? | capture one of each |
| 5 | Does `# total Resistances` count lines or resistance types? | **Closed 2026-09-27**: resistances — R1's ten items each carry an all-elemental line and show 3 |
| 6 | Are the 63 family rows right per entry? | a trade fetch of one unattested room and one unattested quality |
| 7 | How many of the owner's unsuffixed `Adds # to #` lines are on weapons? | **Closed 2026-09-30**: 573 of the copy's 703, 129 not (`data/ranged-census.csv`); no weapon line the export or the site calls global (`data/ranged-uniques.csv`) |
| 8 | Are APT's percentile lines the site's, and the flat hybrids PoB models? | P001–P034, each scored as `percentile.py` does |
| 9 | i05: the site leaves a defence line under its global twin out, and the text reads it as local. Does anything a private item carries tell the two apart? | read with k001 and k002 (`scripts/defence-twins.py`): the item's mark separates an implicit twin, eldritch 17 global, synthesised 10 local, a candidate. Closed by k001 with `searing_item`, then `tangled_item`, required: none found, k002 the mutant, and an eldritch implicit twin is global wherever it is listed |

## Provenance

| Input | Used for |
| --- | --- |
| `../trade-query/data/stats-2026-09-12.json`, `stat-collisions.csv` | the entries; every id a text displays |
| `../trade-query/data/fetch-census.json` | `site` rows (q4, q5); the 30 percentile items; the controls |
| `../trade-query/data/grammar.json` | property fields; the percentile's tip; `extended` figures at 20% quality |
| `../cpp-search/data/pseudomods.toml`, `data/totals-v1.toml` (the table as shipped while the site was asked) | `sum` rows; what the C++ tables list; what no total counts |
| `../item-facts/data/mod-templates.csv`, `properties-census.csv`, `field-census.csv` | templates and corpus counts; `field` rows |
| `../repoe/data/base-defences.csv`, `trade-stat-map.csv`, `template-vs-translation.csv` | defence ranges; stat ids behind a line or a template |
| `poe1` @ e2bd511a `data/stats.json`, `data/mods.json` | `is_local`; where the ranged stats spawn |
| `PathOfBuilding` @ 16de4b82 `src/Classes/Item.lua`, `TradeQueryGenerator.lua` | the per-type percentile; the stat-to-pseudo map |
| `awakened-poe-trade` @ ce551eb7 `…/filters/pseudo/index.ts`, `…/parser/calc-q20.ts`, `Parser.ts` | its pseudo rules; its percentile |
| `../item-facts/raw/m3/store/mock/GERWARIC_7694.db` (local; read-only) | the copy: percentile shapes, the ranged census |
| `classes-v1.toml` (the crate's), `../repoe/data/class-to-trade-category.csv`, `poe1` `stat_translations.json` | class; weapon; a text's stats |

Trap met, for the index: `mod-templates.csv` carries a bare CR inside 260 templates; the scripts open
it with `newline="\n"` and cite a row by its first line.

## Review

The table outgrew a screen (index rule 5): `REVIEW.md`, nine rows, both passes.
