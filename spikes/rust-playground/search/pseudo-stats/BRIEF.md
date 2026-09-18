# Brief — pseudo-stats (a research track; run by a subagent)

You are an Opus subagent running one research track of the item-search
slice under `search/README.md`, "Rules in force". Read this brief, then
`search/README.md` (rules and traps), then the inputs below, in that
order. Work only inside `search/pseudo-stats/`. Never touch the index
(`search/README.md`), another track, `CONTEXT.md`, `decisions/`,
`SURFACES.md` or `brainstorming-notes/`. Never commit, push, fetch,
run a network request or spawn an agent. The reviewer sets the index
row and commits; this brief is deleted at the close, cited by hash.

## The question (the README's, restated once)

For each of the trade site's 298 `pseudo` stats, which mechanism of
the search model answers it, and what evidence in the committed
captures says so? A previous classification
(`search/trade-query/scripts/classify-pseudo.py`, 2026-09-17) sorted the
298 by identifier patterns and assigned every unmatched entry to
"weighted sum of lines". An external audit found that overstated:
the renderer maps `pseudo_ritual_unique_monsters` and
`pseudo_ritual_other_monsters` to *properties*, yet the script called
them line sums; its count class included `Has # Influences`; and the
captured Carnal Boots have one crafted modifier producing two lines,
so counting crafted lines does not establish a modifier count. The
audit's verdict: "Replace the aggregate certainty with per-entry
evidence and an unresolved category." That is this track.

## The classes (fixed; do not add one, do not rename one)

An entry is in a class only when a named input supports it; an entry
with no supporting evidence is **unresolved**, with the one read that
would resolve it. Never default to a substantive class.

| Class | Means | Evidence that admits an entry |
| --- | --- | --- |
| `sum` | a weighted sum of the slots of lines the item displays | a C++ pseudomod table with the same (or an equivalent) name in `pseudomods.toml`, or the site's Q5 observation in `fetch-census.json` |
| `count` | a count of displayed *lines* matching a set | the text counts things and the counted things are lines the private item displays (`mod-templates.csv`, `field-census.csv`); a count of *modifiers* (prefix, suffix, crafted, implicit, enchant, fractured as modifiers) is **not** a count of lines — see `mod-behind-line` and the Carnal Boots example |
| `ranged-total` | a sum over `Adds # to # …` lines, slot-wise | as `sum`, over a ranged template |
| `field` | a field or property the private item carries, or a line's presence | `property_type_to_field` in `grammar.json` maps a property type to the pseudo, or `field-census.csv` / `properties-census.csv` shows the field or property on private items, or `mod-templates.csv` shows the line (for "Has Room:", "Has Logbook …", "Reflection of …", "Has … Influence") |
| `computed` | a function over properties, lines and reference data outside the item | the text names a number no field or line carries (a percentile, a ratio) and the reference data it needs is named in a track README (`search/repoe/README.md` for base ranges) |
| `mod-behind-line` | needs which modifier made a line: affix type, tier, empty slots | the text names a modifier property the private item JSON does not carry; cite the `mods` list on fetched items in `fetch-census.json` (private items have none) and the Carnal Boots example |
| `unresolved` | no input decides it | name the read that would |

## Inputs (by path; read nothing else for evidence)

- `search/trade-query/data/stats-2026-09-12.json` — the 298 entries (group id `pseudo`).
- `search/trade-query/data/grammar.json` — `property_type_to_field` (property types → filter keys, several to `stat.pseudo.*`), `renderer_mod_arrays_in_order`, `hash_category_to_css_class`.
- `search/trade-query/data/fetch-census.json` — the seven owner searches with the site's answers (Q5 is the pseudo total check) and 70 fetched items with their `mods` lists; the Carnal Boots crafted example is one of them.
- `search/cpp-search/data/pseudomods.toml` — the C++ app's 35 pseudomod tables and the templates each sums.
- `search/item-facts/data/field-census.csv`, `properties-census.csv`, `mod-templates.csv` — which fields, properties and line templates the owner's private items actually carry, with counts.
- `search/repoe/README.md` — only to check whether base defence ranges exist in the export (for `computed`).
- `search/DIGEST.md` — only to cite an `S` id already stated (S16, S29, S42, S49, S52); never to add evidence.

Absent inputs: if a file named here is missing or lacks a column you
expected, say so in the README's open questions and continue; that is
a finding, not a stall.

## Outputs

- `data/pseudo-classes.csv` — one row per entry, 298 rows, columns
  `id,text,class,evidence_source,evidence_locator,note`. `evidence_source`
  is one input path above; `evidence_locator` is a key, row or line
  that a reader can open (`property_type_to_field.table.106`,
  `pseudomods.toml [[pseudomod]] name=…`, `mod-templates.csv row …`);
  for `unresolved`, `evidence_source` is empty and `note` names the
  read. Strict CSV, `lineterminator="\n"`, first line a header, no
  comment line (the README's trap).
- `scripts/classify.py` — regenerates the CSV from the inputs. Every
  rule in it must name the evidence it rests on; a rule may be a
  pattern only when its pattern is backed by an input (a name match
  against `pseudomods.toml`, a key in `property_type_to_field`). Its
  fallback is `unresolved` and nothing else. Prints the per-class counts.
- `README.md` — the track's one document, under rule 1: the status
  line, a headline block of at most five bullets, then findings as
  tables: the per-class counts (one table, `unresolved` included), the
  entries whose class the previous script got wrong with why, the
  open questions each naming the read that closes it, and provenance
  rows (the inputs above, by path). About 8 KB; per-entry detail is the
  CSV, never the README.

## Acceptance

`python3 search/pseudo-stats/scripts/classify.py` regenerates
`data/pseudo-classes.csv` byte-for-byte; every non-`unresolved` row
names an input path and a locator that exists in that input; the two
ritual entries are `field` with `property_type_to_field` as their
evidence; `Has # Influences` is not `count`; every "# … Modifiers"
entry is `mod-behind-line` or `unresolved`, never `count`, unless an
input shows the site counts lines.

## The report (at most 300 words)

When done, report: the per-class counts; what you left out; the one
cut you would most want reversed. The reviewer restores from that line.
