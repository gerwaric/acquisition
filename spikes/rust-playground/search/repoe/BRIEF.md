# repoe — startup brief for the first-pass session

Session material, not a record: deleted when this track's status line says the first pass is complete. Everything factual it points at lives in `README.md`, `MANIFEST.md`, the other tracks and `search/README.md` ("Traps"); this file only orders the work.

**Goal.** The track's first pass, before engine-bench (owner, 2026-09-13). Every extract is a script in `scripts/` that names the clone commit from `MANIFEST.md`; a pull of any clone is a new manifest row before anything is re-run.

**Order of work**, each step a committed extract and a findings row in `README.md`:

1. `trade-stat-map.csv` — re-measure coverage of `trade-query/data/stats-2026-09-12.json` by RePoE's `trade_stats` (scoping read: about 61 %; none of RePoE's ids unknown to the capture).
2. The Path of Building hash experiment (`hash-recipe.md`, `hash-check.csv`) under the acceptance rule in `README.md` — it decides whether the id mapping is an algorithm or a text join, which changes how every later extract is read. First question: does `poe1/data/stats.json` expose the `Stats.dat` hashes?
3. `base-taxonomy.csv` — the export's classes and categories against trade-query's 83 category ids and the class names in the seven fetches' `extended.text`.
4. `template-vs-translation.csv` — the census head (1,933 templates) first, the tail second.
5. `mod-stat-index.csv`, `pob-format.md`.

**Then:** the README's headline block last, from the numbers; status line; index row byte count; docs check; delete this file; commit with the story.
