# pseudo-stats — which mechanism answers each of the site's 298 `pseudo` stats

Status: first pass complete — 2026-09-18; second pass, part 1 complete — 2026-09-26, reviewed; the pilot and six checks captured and read — 2026-09-26; four rounds captured and read — 2026-09-26, 212 searches; the table's changes written as rows; those that need no twin applied (totals v2, `e81cc8ca`), a total of nothing lacked (`37faf903`); the ranged family waits for 9d's category.

Headline:
- **Part 1: 1,655 candidate pairs** (pseudo, line template) over 40 of the 58 pseudos in scope:
  2 `site`, 5 `sources-differ`, 15 `one-source`, **0 `sources-agree`**, 1,633 `text-only`. No tool
  names any contributor of the 21 `Adds # to #` pseudos; in the scope, no two tools name one line.
- **The percentile is the site's own stated rule**: its filter tip reads "The percentile of all base
  defence rolls, averaged". Rolls recovered as integers from the display reproduce **30 of 30**
  captured values, each a single value; APT's formula gives 20 exactly and 29 within rounding;
  every negative control fails. A private item displays everything the rule reads.
- **The site is the checker** (212 searches captured; 912 readings, none disagreeing): of 57
  pseudos 33 are closed — a complete and a sound check each found nothing — and 24 explained:
  what a check found shows, item for item, what the rows give. `data/table-changes.csv` is the
  totals table's changes as rows, 227, none applied.
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

## The captures, and what the site said

A line-by-line search reads ten items, and a pseudo's rows are a claim about every item. Rows are
**complete** when no listed item shows the pseudo while carrying none of them, **sound** when no
listed item carries one while showing no pseudo: each is one search over everything listed. A
search that finds nothing proves nothing until its mutant, the same search with one row changed,
finds something (c2, c4, d02). The rows are `scripts/rows.py`, versions appended and never edited;
every capture is read against each pseudo's latest (`scripts/evidence.py`, `data/evidence.csv`).

| Round | Searches | Asked | Captured |
| --- | --- | --- | --- |
| the pilot | p1, p2 | a composed link; an `if` group | by hand |
| the checks | c1–c6 | the method's mutants; total life; attack speed | by hand |
| two | d01–d24, b1, b2 | total life sound; the ranged family on its own text; the owner's report | one recording |
| three | e001–e107 | every pseudo in scope, complete and sound | two recordings; e047, e051 refused as too complex |
| four | f001–f073 | the 33 pseudos whose rows round three moved | three recordings |

A sitting is one recording of the browser's network panel, exported with its content and split by
`scripts/har-split.py` (`MANIFEST.md`): no file name is typed, since a search response carries its
query and the sheet says which row that is.

What the site counts, as the rows now stand:

| Finding | Evidence | For the build |
| --- | --- | --- |
| **A row's two eldritch forms are counted as the row**: `While a Unique Enemy is in your Presence, …`, `While a Pinnacle Atlas Boss is in your Presence, …` | every complete check that found items found these and nothing else; left out, 197 readings disagree | 147 rows added, one rule; ruled, the owner: "include the eldritch mods" |
| **A sum of nothing shows no total** | c3, d01, e043, f005: lines that cancel | ruled, the owner: "Yes, let's make it absent to match the site."; of an item with no such line as of one whose lines cancel: "A". Built: `has:` asks a total's presence |
| **A ranged pseudo is the average of a line's two numbers**, shown in both places | every reading of the family | a ranged total is one number |
| **A weapon's own line** — the `(Local)` twin — feeds the attacks' pseudo alone; **the other twin** feeds the plain, the attacks' and the spells'; `to Spells and Attacks` feeds both; an aggregate is its types' rows together | rounds two to four; the scoped pseudos' sound checks found nothing | 52 rows mean a twin: the build answers by the item's class, the export spawning the `(Local)` stats on weapons alone |
| `#% to All Resistances` counts toward each resistance total | e042, e044, e048 | 6 rows; the elemental total's 3 and the total's 4 are arithmetic, unseen |
| **One text, two ids, one counted**: `+# to Strength and Intelligence` on That Which Was Taken is left out of total life, Strength and Intelligence | c3, b1, b2, e053, e057 | ruled, the owner: "I believe this is a bug. Let's count the mod"; reported by him to GGG 2026-09-26 |
| `+# total to Level of Socketed Skill Gems` does not count `# to Level of Socketed Skill Gems` | e103: 10,000 found, the ten fetched one unique; f030, f031: without the row, complete and sound, nothing found | ruled, the owner: "yes, include the socketed skill gems": the row stays |

Two things the searches could not do, each a limit of what is closed:

- **A `not` on an eldritch form's id lets some items through.** 23 of round four's complete checks
  found 9 to 146 items each. Of the 229 fetched, 227 carry a form the search had excluded by that
  id, and two the enchant's id that round three had wrongly left out; each shows what the rows
  give. So 24 pseudos are explained and not closed: nothing new was found among what was fetched,
  and what was not fetched is unread.
- **A weighted group is refused past some cost** (56 and 4 an id; taken at 23 ids, refused at 41),
  so a total of many rows is asked sound in parts, and a part finds items whose whole sum is
  nothing: five parts of the elemental and the whole resistance totals found 152 to 2,284 each,
  and on all fifty fetched the rows' lines sum to nothing.

The reading can fail, shown on the captures each time the rows moved: the eldritch forms left out
disagree on 197 readings, the low number read for the average on 160, a weapon's line left out of
attacks on 54 and counted by the plain pseudos on 40, the twin id counted on 20 and 5.

The most any sitting used of what the site allows: 13 of 15 searches in 60 s (the last recording)
and 44 of 60 in 300 s (the one before it). No response asked a sitting to slow down.

## Candidate claims

For `docs/design/trade-ground-truth.md`, authored master-side; each is what the captures show, dated
2026-09-26, Standard, PC.

| Claim | Evidence |
| --- | --- |
| A search's id is its query: compact JSON, deflate, a fixed ten-byte gzip header, base64url without padding. The site takes a link composed so, and returns the query less every `"disabled":false` | `search-sheet.py --self-test`, 8 ids; 212 composed links opened |
| The site shows a pseudo's value only where the query names it; an `if` group shows every member's and filters nothing | q4, q5; p2, c5 |
| A pseudo is the sum of the displayed numbers of the lines it counts, each at its weight; a sum of nothing is no pseudo | every reading; c3, d01 |
| A ranged pseudo is the average of a line's two numbers, shown in both places | round two |
| A line's two eldritch forms are counted as the line | rounds three and four |
| Two stats displaying one text are counted apart: the `(Local)` twin toward attacks alone; one twin id of `+# to Strength and Intelligence` toward nothing | rounds two to four; c3, e053, e057 |
| A `not` on an eldritch form's id lets some items carrying it through | round four, 227 items |
| Rate limits, searches: 3 in 5 s an account; 8 in 10 s, 15 in 60 s, 60 in 300 s, 600 in three hours an address. Fetches: 6 in 4 s an account; 12 in 4 s, 16 in 12 s, 100 in 300 s, 1,000 in three hours an address | the recordings' `X-Rate-Limit-*` headers |
| A weighted group costs a `complexity` of 56 and 4 an id; a query is refused, status 400, "Query is too complex", somewhere between 148 and 220 | d01, d02, e039–e063; e047, e051 |

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
| `../cpp-search/data/pseudomods.toml`, `data/totals-v1.toml` (the table as shipped while the site was asked) | `sum` rows; what the C++ tables list; what no total counts |
| `../item-facts/data/mod-templates.csv`, `properties-census.csv`, `field-census.csv` | templates and corpus counts; `field` rows |
| `../repoe/data/base-defences.csv`, `trade-stat-map.csv`, `template-vs-translation.csv` | defence ranges; stat ids behind a line or a template |
| `poe1` @ e2bd511a `data/stats.json`, `data/mods.json` | `is_local`; where the ranged stats spawn |
| `PathOfBuilding` @ 16de4b82 `src/Classes/Item.lua`, `TradeQueryGenerator.lua` | the per-type percentile; the stat-to-pseudo map |
| `awakened-poe-trade` @ ce551eb7 `…/filters/pseudo/index.ts`, `…/parser/calc-q20.ts`, `Parser.ts` | its pseudo rules; its percentile |

Trap met, for the index: `mod-templates.csv` carries a bare CR inside 260 templates; the scripts open
it with `newline="\n"` and cite a row by its first line.

## Review

The table outgrew a screen (index rule 5): `REVIEW.md`, nine rows, both passes.
