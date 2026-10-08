Moved from `../README.md` as it stood at `da2c1711`, when the page passed its guide: the second pass's part 1 by table — the candidates by status, why a template is a candidate, the ranged family by what the export says is local, the searches part 1 counted by form, and what it left out. Each block is verbatim under the heading it had; paths are relative to the track's directory. The counts are `scripts/candidates.py`'s, printed when it runs.

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

## The searches part 1 counted, by form

Not run, and one command away (`tools/trade-sheet.py --method if`): the rows named L, P, C and R below
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

## Left out

- A shipped row where the sources differ: APT's `#% total increased Physical Damage` lists only the
  Global line (`index.ts` L168–L173); the C++ table also counts `#% increased Physical Damage`.
- PoB's kinds as sources (above); `# to Armour and Energy Shield`, a PoB kind with no template.
- The percentile formula read from APT's `renderer/src/parser/`, outside the brief's named
  `filters/pseudo/`, whose `item-property.ts` only consumes the value computed there.
