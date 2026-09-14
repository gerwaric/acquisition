# prior-art — brief for the first-pass run (2026-09-13)

## Rules that bind this run

- Read, in order: `search/README.md` (rules in force, traps), this track's `README.md` and `MANIFEST.md` if present, then only the inputs named below. Do not read the other tracks' READMEs except where an input points there.
- Work only inside this track's directory. Never modify `search/README.md`, any other track, `CONTEXT.md`, `decisions/`, or `SURFACES.md`. Never commit, push, fetch from the network, or spawn agents.
- Absolute paths under `search/`; never `cd` in a compound command. Extracts are strict JSON or CSV with `lineterminator="\n"`; every `data/` file's first line names the script and the inputs; `raw/` is never committed.
- Every finding traces to a `data/` file or a cited path at a commit. Nothing from memory of the app, the game, the site or the tool: a fact you cannot point at is marked open.
- Write the README last, from the script outputs, and its headline block last of all: a status line (`Status: first pass complete — 2026-09-13`), at most five headline bullets, findings as tables or short paragraphs headed `**F<n> — …**`, numbers in one table, every open question naming the one read or experiment that closes it, Provenance, an empty Review table. Budget about 12 KB; per-item detail goes to `data/`.
- Finish with a report of at most 300 words: what landed, what is open, what surprised you, what you could not do. This brief is deleted at the track's close.

## This track

Input: the owner's clone at `/Users/tom/Development/GitHub/gerwaric/awakened-poe-trade` — refuse to run if `git rev-parse HEAD` there is not `ce551eb7a9b704fbdcc2478eebb26be8f91786c7`; cite every file as `path:line` at that commit. Read: `renderer/public/data/en/stats.ndjson` and `items.ndjson`; `renderer/src/parser/` (`Parser.ts`, `modifiers.ts`, `stat-translations.ts`, `advanced-mod-desc.ts`, `calc-q20.ts`); `renderer/src/web/price-check/` (the trade-query builder), and `renderer/src/web/stash-search/` and `item-search/` — a stash search exists here and is prior art in its own right; `docs/` and `DEVELOPING.md` for where the dataset is generated; `git log -- renderer/public/data/en/stats.ndjson` (55 commits) for how often and how much the data moves. Sibling data to join to: `search/trade-query/data/stats-2026-09-12.json`, `search/repoe/data/trade-stat-map.csv`, `search/trade-query/data/stat-collisions.csv`.

Outputs:
- `data/stat-model.md` — the shape of one stats entry with three real entries pasted (a plain stat, a local/global twin, a several-ids stat), and one table of counts by `scripts/stat-model.py`: entries, matchers per entry, matchers carrying `value`, entries with several ids in one category, entries with ids in several categories, `better` values, entries with options.
- `data/coverage.csv`, by the same script: its trade ids joined to the trade capture and the export's text join — capture ids it carries, capture ids it lacks, how many of the 380 collisions it carries as several ids in one entry, how many it resolves and how.
- `data/stash-search.md` — what its stash search and item search do: the query model, the fields, how a stash item is matched, what it shows; one table.
- Findings: identity (what a line is matched on — the template, the ref, the matcher string; how `[Tag|Display]` and numbers are handled); ambiguity (a line matching several refs, local vs global, unknown lines); update handling (the generator, the data's commit cadence and diff size across patches); its stash search; what it does not attempt. Cite; never recall.

Acceptance: a reader can say how a maintained tool gives a displayed line its stat identity and what it does when the site or the game moves, with a file and line for each claim.
