# pseudo-stats — which mechanism answers each of the site's 298 `pseudo` stats

Status: first pass complete — 2026-09-18; second pass, part 1 complete — 2026-09-26, reviewed.

Headline:
- **Part 1: 1,655 candidate pairs** (pseudo, line template) over 40 of the 58 pseudos in scope:
  2 `site`, 5 `sources-differ`, 15 `one-source`, **0 `sources-agree`**, 1,633 `text-only`. No tool
  names any contributor of the 21 `Adds # to #` pseudos; in the scope, no two tools name one line.
- **The percentile is the site's own stated rule**: its filter tip reads "The percentile of all base
  defence rolls, averaged". Rolls recovered as integers from the display reproduce **30 of 30**
  captured values, each a single value; APT's formula gives 20 exactly and 29 within rounding;
  every negative control fails. A private item displays everything the rule reads.
- **The export separates** the unsuffixed `Adds # to # <type> Damage` on a weapon (`local_*`, is_local)
  from the same text elsewhere (`global_*`: uniques, bench crafts, grafts, veiled mods, delve gloves),
  as the site does with its `(Local)` ids; the committed census cannot split the owner's lines by class.
- **Searches: 264 in the `if` form** (committed), 565 in the `and` form; 131 rows no stat id can search.
- `+#% to All Resistances` is named by APT under five resistance totals and left out by the C++
  tables (`sources-differ`, sheet row L001); the owner's corpus holds none.

Per-row detail: `data/candidates.csv` (`scripts/candidates.py`), `data/percentile-check.csv`
(`scripts/percentile.py`), `data/search-sheet.csv` (`scripts/search-sheet.py`, from the candidates).
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

## The searches, by form

| Form | Pilot | Batch | Percentile lines | Percentile cases | Open question | Total |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `--method if` (committed) | 2 | 219 lines | 34 | 8 | 1 (R1) | **264** |
| `--method and` | 2 | 520 pairs | 34 | 8 | 1 (R1) | **565** |

Lines by their best pair: `sources-differ` 1, `text-only` 215, `one-source` 3 (percentile: 27, 7).
Controls: `+# to Strength` required, showing `+# total maximum Life` (the site counted it, q4[7]), or
`+#% to Fire Resistance` showing its total (q5) when the candidates include what Strength feeds; a
pseudo the control also feeds is read by value; a result with no items decides nothing and is
rerun with the other control. L033 and L219 repeat the pilot's p2 and p1. 131 rows (72 of them
representatives or direct) have no stat id: a skill gem's text, a description, a buff.

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
