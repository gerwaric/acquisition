# item-filter — brief for the first-pass run (2026-09-13)

## Rules that bind this run

- Read, in order: `search/README.md` (rules in force, traps), this track's `README.md` and `MANIFEST.md` if present, then only the inputs named below. Do not read the other tracks' READMEs except where an input points there.
- Work only inside this track's directory. Never modify `search/README.md`, any other track, `CONTEXT.md`, `decisions/`, or `SURFACES.md`. Never commit, push, fetch from the network, or spawn agents.
- Absolute paths under `search/`; never `cd` in a compound command. Extracts are strict JSON or CSV with `lineterminator="\n"`; every `data/` file's first line names the script and the inputs; `raw/` is never committed.
- Every finding traces to a `data/` file or a cited path at a commit. Nothing from memory of the app, the game, the site or the tool: a fact you cannot point at is marked open.
- Write the README last, from the script outputs, and its headline block last of all: a status line (`Status: first pass complete — 2026-09-13`), at most five headline bullets, findings as tables or short paragraphs headed `**F<n> — …**`, numbers in one table, every open question naming the one read or experiment that closes it, Provenance, an empty Review table. Budget about 12 KB; per-item detail goes to `data/`.
- Finish with a report of at most 300 words: what landed, what is open, what surprised you, what you could not do. This brief is deleted at the track's close.

## This track

Input: `raw/Path of Exile - Item Filters.html`, GGG's own documentation of the loot-filter language, saved by the owner (`MANIFEST.md`). Sibling data to join to: `search/cpp-search/data/filters.toml`, `search/trade-query/data/grammar.json` (its filter groups), `search/repoe/data/class-to-trade-category.csv`. The worked-example filter named in the README's inputs has not landed; skip it and say so.

Outputs:
- `data/conditions.csv`, by `scripts/extract-conditions.py` over the saved page: one row per condition — name, operand type, operators the page allows, the value vocabulary or example, the item field it reads; then the hand-kept join columns `cpp_filter` (the caption in `filters.toml` that reads the same thing, or none) and `trade_filter` (the id in `grammar.json`, or none). Actions are listed by name only in a second small file or a README line; they do not concern a search.
- `data/class-names.csv` — the class vocabulary the page lists (or the page's examples use), joined to the export's class names and the trade categories.
- Findings: what the three namings agree on; where they diverge (a spelling, an operator, a semantic such as "matches any of" vs exact); what only the filter language names; the operators' semantics as the page states them, quoted sparingly — GGG's text is paraphrased, never committed whole.

Acceptance: a reader can say, for every C++ filter and every trade filter that reads an item, whether GGG's filter language names the same thing and under what name.
