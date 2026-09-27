# pseudo-stats — which mechanism answers each of the site's 298 `pseudo` stats

Status: first pass complete — 2026-09-18; second pass, part 1 complete — 2026-09-26, reviewed; the pilot and six checks captured and read — 2026-09-26; rounds two and three captured and read — 2026-09-26, 124 searches; e001–e015 to capture again and round four (f001–f073) with the owner.

Headline:
- **Part 1: 1,655 candidate pairs** (pseudo, line template) over 40 of the 58 pseudos in scope:
  2 `site`, 5 `sources-differ`, 15 `one-source`, **0 `sources-agree`**, 1,633 `text-only`. No tool
  names any contributor of the 21 `Adds # to #` pseudos; in the scope, no two tools name one line.
- **The percentile is the site's own stated rule**: its filter tip reads "The percentile of all base
  defence rolls, averaged". Rolls recovered as integers from the display reproduce **30 of 30**
  captured values, each a single value; APT's formula gives 20 exactly and 29 within rounding;
  every negative control fails. A private item displays everything the rule reads.
- **The site is the checker** (124 searches captured; 542 readings, none disagreeing): total life,
  total Dexterity and 21 gem totals are complete and sound; every other total lacks the two
  eldritch forms of its rows; a sum of nothing shows no total; one text under two ids may be
  counted under one.
- **The ranged family is one reading**: the site shows the average of a line's two numbers; a
  weapon's own line feeds the attacks' pseudo alone, the other twin the plain, the attacks' and
  the spells'.
- `+#% to All Resistances`, which APT names and the C++ tables leave out, is counted: one item
  shows it under the fire, lightning and chaos totals (e042, e044, e048).

Per-row detail: `data/candidates.csv` (`scripts/candidates.py`), `data/percentile-check.csv`
(`scripts/percentile.py`), `data/search-sheet.csv` (`scripts/search-sheet.py`), `data/captures.json`
(`scripts/captures.py`, the owner's captures scrubbed), `data/evidence.csv` (`scripts/evidence.py`).
Each regenerates byte-for-byte; run `percentile.py` before `candidates.py`, which reads its output.

## Part 1: the candidates by status

Scope (`BRIEF.md`): the 21 `pseudo_adds_*`, `pseudo_total_life`, `pseudo_base_defence_percentile`, and
the 35 shipped totals for the lines no total counts (`../MEASUREMENTS.md`, M6).

| Status | Ranged | Life | Totals | Percentile | All | Admitted by |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| `site` | 0 | 2 | 0 | 0 | 2 | `fetch-census.json`: the weights reproduce every q4 item (life 1, Strength 0.5) |
| `sources-differ` | 0 | 0 | 5 | 0 | 5 | APT names `+#% to All Resistances`; the C++ tables (and PoB, for four) list the pseudo without it |
| `one-source` | 0 | 3 | 0 | 12 | 15 | APT alone: Str&Dex, Str&Int, all Attributes at 0.5 into life; its 12 percentile lines |
| `sources-agree` | 0 | 0 | 0 | 0 | 0 | — |
| `text-only` | 439 | 35 | 1,077 | 82 | 1,633 | the template's row in `mod-templates.csv`, by the table below |
| **All** | **439** | **40** | **1,082** | **94** | **1,655** | |

The sources are three separate readings: the C++ app's tables (`pseudomods.toml`; the shipped totals
are generated from it, so they are one source), APT's `PSEUDO_RULES`, PoB's stat-to-pseudo map.
Path of Building's `Item.lua` names the percentile's lines by modifier *kind* (`"Armour", "BASE"`),
never by text, so a kind matched by its words is a note on the row, not a source (the cut that
leaves `sources-agree` empty). The 18 gem totals without a row are the ones whose every
`Level of Socketed … Gems` template a shipped total already counts.

## Why a template is a candidate

`WORDS` in `candidates.py` has one row per pseudo; a template is in reach when every group holds
one of its words, whole and with case. Rows differing only in the named word are grouped here.

| Pseudos | Scope | Groups (each must hold one) |
| --- | --- | --- |
| `adds_<t>_damage`, t = physical, lightning, cold, fire, chaos | all | `Adds # to #` · `<T>` · `Damage` |
| `adds_elemental_damage` | all | `Adds # to #` · Fire/Cold/Lightning/Elemental · `Damage` |
| `adds_damage` | all | `Adds # to #` · `Damage` |
| the seven above `…_to_attacks` / `…_to_spells` | all | the same · Attack/Attacks (Spell/Spells) or neither word |
| `total_life` | all | `maximum Life` |
| `total_<e>_resistance`, e = cold, fire, lightning | uncounted | `<E>`/Elemental/all · Resistance(s) |
| `total_elemental_resistance` | uncounted | Fire/Cold/Lightning/Elemental/all · Resistance(s) |
| `total_chaos_resistance` | uncounted | Chaos/all · Resistance(s) |
| `total_resistance` | uncounted | Resistance(s) |
| `total_<a>`, a = strength, dexterity, intelligence | uncounted | `<A>`/Attributes |
| `total_attack_speed` · `total_cast_speed` | uncounted | Attack(s) · Speed — Cast · Speed |
| `increased_physical_damage` | uncounted | increased/reduced · `Physical Damage` |
| `critical_strike_chance_for_spells` | uncounted | `Critical Strike Chance` · Spell(s) |
| `total_additional_gem_levels` · the 21 `…_<k>_gem_levels` | uncounted | `Level of Socketed` · Gem(s) — `<K>` |
| `base_defence_percentile` | all | Armour/Evasion Rating/Evasion/Energy Shield/Ward/Defences · `# to`/`#% increased`/`#% reduced` |

A template of a conditional shape keeps its row; the (pseudo, shape) family names one representative
for the sheet (a stat id first, then status, then corpus items) and the rest say `family:`.
**256 families hold 998 `family:` rows**: a monster's or an enemy's 294 (27 families), per 198 (35),
a minion's or ally's 156 (19), a passive's grant 127 (14), while 104 (25), a description 87 (30),
if or recently 77 (27), against 74 (25), a weapon's or hand's 55 (20), a socketed gem's or skill's
43 (13), during a flask's effect 39 (21). A shape whose words are in the pseudo's own text is not a
shape for it (`Level of Socketed` gems).

## The ranged family by what the export says is local

| Templates | Corpus items | Stats behind the text (poe1 `stats.json`) | Where the export spawns them (`mods.json`) |
| --- | ---: | --- | --- |
| `Adds # to # Physical / Lightning / Cold / Fire / Chaos Damage` | 485 / 269 / 266 / 224 / 67 | `local_{min,max}imum_added_<t>_damage` (local) **and** `global_…` (not local) | local: 107–262 mods each, spawning on weapon tags only (physical: also a veiled tag); global: 9–31 each — uniques, bench crafts, grafts, veiled (amulet, quiver, ring, shield), delve (gloves) |
| the same `… to Attacks` | 543 / 272 / 245 / 219 / 39 | `attack_…` (not local), one id each | — |
| the same `… to Spells` | 5 / 98 / 79 / 97 / 11 | `spell_…` (not local), one id each | — |
| `Adds # to # Elemental Damage`, `Adds # to # Damage` (and scoped) | — | no template: the site's aggregates | — |

The site gives the unsuffixed text two ids — `explicit.stat_709508406` "(Local)" and
`explicit.stat_321077055` for fire — so its pseudo may count one and not the other; a private item
shows only the text, and its class says which. How many of the owner's 224 fire lines sit on weapons
is not in any committed census (open question 7). Each unsuffixed line goes out as a `count` over
both ids (L002–L006, L021); the returned item's hash names which it carried.

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

## The searches part 1 counted, by form

Not run, and one command away (`search-sheet.py --method if`): the rows named L, P, C and R below
and in the open questions are that sheet's. The committed sheet is the pilot and the checks.

| Form | Pilot | Batch | Percentile lines | Percentile cases | Open question | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `--method if` | 2 | 219 lines | 34 | 8 | 1 (R1) | **264** |
| `--method and` | 2 | 520 pairs | 34 | 8 | 1 (R1) | **565** |

Lines by their best pair: `sources-differ` 1, `text-only` 215, `one-source` 3 (percentile: 27, 7).
Controls: `+# to Strength` required, showing `+# total maximum Life` (the site counted it, q4[7]), or
`+#% to Fire Resistance` showing its total (q5) when the candidates include what Strength feeds; a
pseudo the control also feeds is read by value; a result with no items decides nothing and is
rerun with the other control. L033 and L219 repeat the pilot's p2 and p1. 131 rows (72 of them
representatives or direct) have no stat id: a skill gem's text, a description, a buff.

## The captures: the pilot and the checks

The owner ran p1, p2 and c1–c6 on 2026-09-26 (`MANIFEST.md`). Per-pair detail is
`data/evidence.csv`: every capture read against each pseudo's latest rows (`scripts/rows.py`).

A line-by-line search reads ten items, and a pseudo's rows are a claim about every item. Rows are
**complete** when no listed item shows the pseudo while carrying none of them, **sound** when no
listed item carries one while showing no pseudo. A search that finds nothing proves nothing until
its mutant, the same search with one row changed, finds something.

| Search | Asked | Found | Says |
| --- | --- | ---: | --- |
| p1 | total life beside `Strength and Intelligence` | 10,000 | a composed link is taken; the hybrid attribute lines count at one half |
| p2 | an `if` group over three pseudos | 4,642 | an `if` group shows a value and filters nothing; the combined speed line feeds neither speed total |
| c1 | total life, complete on five rows | **0** | no listed item shows the total without one of the five |
| c2 | c1's mutant, `all Attributes` left out | 10,000 | the check can find: ten items, each the line at one half |
| c3 | total life, sound | 3,338 | two kinds, below |
| c4 | c3's mutant, `#% increased maximum Life` among the rows | 10,000 | a pseudo inside a `not` group is taken |
| c5 | two pseudos in one `if` group | 10,000 | both show on all ten, in either order: p2's caveat is closed |
| c6 | total attack speed, complete on the shipped row | 212 | ten of ten carry one line the total lacks |

What c3 and c6 found:

| Finding | Evidence | For the build |
| --- | --- | --- |
| **A sum of nothing shows no total.** `+21 to Strength` beside `-21 to Strength`; `+24 to maximum Life` beside a scourge's `-24` | c3[2], [3], [6], [8], [9] | ruled 2026-09-26, the owner: "Yes, let's make it absent to match the site." The item lacks the total |
| **One text, two ids, one counted.** `+# to Strength and Intelligence` under `explicit.stat_2543977012`, on the unique jewel That Which Was Taken, shows no total; under `stat_1535626285` it counts at one half | c3[0], [1], [4], [5], [7]; p1, eleven items | ruled 2026-09-26, the owner: "I believe this is a bug. Let's count the mod". The search counts the line wherever it is displayed, and the difference from the site is stated; b1 and b2 are the report's two searches |
| **A conditional line is counted.** `While a Unique Enemy is in your Presence, #% increased Attack Speed` feeds `+#% total Attack Speed` at 1 | c6, ten of ten | a row the shipped total lacks; what else is among the 212 is d03's |

The reading can fail, shown on the captures: the twin id counted disagrees on 5 items, the
conditional implicit left out on 10, `all Attributes` at 1 on 12, the combined speed line counted
on 10, a sum of nothing read as a value on 5.

Round two (d01–d24, b1, b2), one sitting of 26 searches in 3 min 15 s, recorded once and split by
`scripts/har-split.py` (`MANIFEST.md`): no file name is typed, since a search response carries its
query and the sheet says which row that is. The first export held no body; the second, of the same
sitting, held every one.

| Search | Asked | Found | Says |
| --- | --- | ---: | --- |
| d01 | total life, sound by the site's own sum | 1 | the one item is `+27 to maximum Life` beside a scourge's `-27`: a sum of nothing. Total life is sound |
| d02 | d01's mutant, the twin id among the rows | 3,298 | a `weight2` group beside a `not` is taken |
| d03 | total attack speed, complete on two rows | 18 | ten of ten carry `While a Pinnacle Atlas Boss is in your Presence, #% increased Attack Speed` |
| d04, d13, d16, d19, d22 | each plain `Adds # to # <type> Damage`, complete on its own text | **0** | the plain pseudo counts its own text and nothing else |
| the ten scoped, d05–d24 | each `to Attacks` and `to Spells`, complete on its own text | 10,000 | each counts lines beyond its own text, below |
| d07–d12 | the six aggregates | 10,000 | an aggregate is its types' rows together |
| b1, b2 | the twin id with total life required, and alone | 0; 3,297 | what the owner's report says |

The ranged family, one reading for the five types (`scripts/rows.py`, `ranged_changes`):

| The site | Evidence |
| --- | --- |
| shows the **average** of a line's two numbers, in both places: `Adds 54.5 to 54.5 Fire Damage to Attacks` over `Adds 31 to 78 Fire Damage` | every one of 160 readings; the low number read instead disagrees on all 160 |
| counts a weapon's own line — the `(Local)` twin — toward the attacks' pseudo and never the spells' | 54 readings toward attacks; 5 carrying it show a spells' pseudo without it |
| counts the other twin toward both | 21 readings toward spells, 3 toward attacks (chaos alone: thin) |
| counts `to Spells and Attacks` toward both | 35 readings toward spells, 2 toward attacks |
| leaves `to Attacks` lines out of the plain aggregate | 2 readings |

A private item shows one text for both twins. What tells them apart on it is what the item is: the
export spawns the `(Local)` stats on weapons alone (the table above, by what the export says is
local). A row that means a twin is the build's to answer by the item's class.

The reading can fail, shown on the captures: a weapon's own line counted toward spells disagrees on
5 items, left out of attacks on 54; the other twin left out of attacks on 3; `to Spells and Attacks`
left out on 37; lightning left out of the aggregates on 25; the Pinnacle implicit left out on 10.

The most the sitting used of what the site allows: 9 of 15 searches in 60 s, 28 of 60 in 300 s, 71
of 600 in three hours; 22 of 100 fetches in 300 s. No response asked it to slow down.

Round three (e001–e107), one sitting of 107 searches in 26 min, recorded once. The browser had
dropped the first fifteen searches' bodies by the export (e001–e015: their shape alone is
`data/recording-shape.csv`), and the site refused two as too complex (e047, e051).

| Pseudos | Complete | Sound | Says |
| --- | --- | --- | --- |
| total Dexterity and 21 of the 22 gem totals | 0 found | 0 found | closed on the shipped rows |
| the resistances, cast speed, increased physical damage, spell critical strike chance, attack speed | 17–645 found | 0 found, or refused | each lacks its rows' eldritch forms |
| the ten scoped ranged pseudos and four scoped aggregates, of those captured | 85–681 found | 0 found | the same; every row of round two's reading is counted wherever it appears |
| the plain ranged pseudos and the two plain aggregates | 0 found | 10,000 found | a weapon's own line is not counted: the plain pseudo is the other twin alone |
| total Strength, total Intelligence | 0 found | 3,296 found | the twin id of That Which Was Taken is left out of these two as of total life |
| total level of socketed skill gems | 0 found | 10,000 found | the pseudo's own text is not counted; the ten fetched are one unique, Edge of Madness |

What round three changes in the rows (`scripts/rows.py`), each a version the captures asked for:

| Change | Evidence |
| --- | --- |
| **A row's two eldritch forms are counted as the row**: `While a Unique Enemy is in your Presence, …` and `While a Pinnacle Atlas Boss is in your Presence, …`, for every row the site lists a form of | every complete check that found items found these and nothing else in the ten fetched; left out, 197 readings disagree in 20 searches |
| The plain ranged pseudo counts the other twin alone | e016, e021, e027, e033; a weapon's line counted disagrees on 40 |
| `#% to All Resistances` counts toward each resistance total | e042, e044, e048, one item; by the same arithmetic 3 toward the elemental total and 4 toward the total, unseen |
| The twin id is never counted toward total Strength or Intelligence | e053, e057, ten of ten each |
| `# to Level of Socketed Skill Gems` is not counted toward its own total | e103 |
| The enchant `No Physical Damage` carries the id of `#% increased Physical Damage` and is no row | e061: asked by id, 217 found; its text is another, so the search leaves the id out |

What the site takes: a weighted group costs 56 and 4 an id (`complexity` 96 at 10 ids, 128 at 18,
148 at 23) and was refused at 41 and 45; a count or a `not` of 71 ids was taken. The most the
sitting used of the allowance: 10 of 15 searches in 60 s, 32 of 60 in 300 s, 180 of 600 in three
hours.

Round four, f001–f073: the 33 pseudos whose rows moved, complete and sound at the versions written
in `ROUND_FOUR`, a weighted group of more than 20 ids asked in parts. With e001–e015 again, 88.

## Left out

- A shipped row where the sources differ: APT's `#% total increased Physical Damage` lists only the
  Global line (`index.ts` L168–L173); the C++ table also counts `#% increased Physical Damage`.
- PoB's kinds as sources (above); `# to Armour and Energy Shield`, a PoB kind with no template.
- The percentile formula read from APT's `renderer/src/parser/`, outside the brief's named
  `filters/pseudo/`, whose `item-property.ts` only consumes the value computed there.

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
| 1 | Do the 13 `unresolved` entries whose text matches a template exactly (movement speed, rarity, leech, mana regeneration) sum those lines? | a `WORDS` row each, then their sheet rows: this pass's method, not yet its scope |
| 2 | Which lines does each `Adds # to #` pseudo sum? | sheet rows L002–L009, L012, L013, L016, L021, L032, L045, L067, and the families they represent |
| 3 | Is `#% Base Defence Percentile` computable from the export? | **Closed 2026-09-26** on the captures: the site's stated rule, rolls recovered from `../repoe/data/base-defences.csv`, reproduces 30 of 30; what no capture reaches is C1–C8 |
| 4 | What do a logbook and a lake tablet look like in the private API? | capture one of each |
| 5 | Does `# total Resistances` count lines or resistance types? | sheet row R1: one all-elemental line, 1 or 3 |
| 6 | Are the 63 family rows right per entry? | a trade fetch of one unattested room and one unattested quality |
| 7 | How many of the owner's unsuffixed `Adds # to #` lines are on weapons? | a census of those templates by item class over the owner's copy |
| 8 | Are APT's percentile lines the site's, and the flat hybrids PoB models? | P001–P034, each scored as `percentile.py` does |

## Provenance

| Input | Used for |
| --- | --- |
| `../trade-query/data/stats-2026-09-12.json`, `stat-collisions.csv` | the entries; every id a text displays |
| `../trade-query/data/fetch-census.json` | `site` rows (q4, q5); the 30 percentile items; the controls |
| `../trade-query/data/grammar.json` | property fields; the percentile's tip; `extended` figures at 20% quality |
| `../cpp-search/data/pseudomods.toml`, `crates/acquisition-search/reference/totals-v1.toml` | `sum` rows; what the C++ tables list; what no total counts |
| `../item-facts/data/mod-templates.csv`, `properties-census.csv`, `field-census.csv` | templates and corpus counts; `field` rows |
| `../repoe/data/base-defences.csv`, `trade-stat-map.csv`, `template-vs-translation.csv` | defence ranges; stat ids behind a line or a template |
| `poe1` @ e2bd511a `data/stats.json`, `data/mods.json` | `is_local`; where the ranged stats spawn |
| `PathOfBuilding` @ 16de4b82 `src/Classes/Item.lua`, `TradeQueryGenerator.lua` | the per-type percentile; the stat-to-pseudo map |
| `awakened-poe-trade` @ ce551eb7 `…/filters/pseudo/index.ts`, `…/parser/calc-q20.ts`, `Parser.ts` | its pseudo rules; its percentile |

Trap met, for the index: `mod-templates.csv` carries a bare CR inside 260 templates; the scripts open
it with `newline="\n"` and cite a row by its first line.

## Review

The table outgrew a screen (index rule 5): `REVIEW.md`, nine rows, both passes.
