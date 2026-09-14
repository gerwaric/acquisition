# engine-bench — brief for the first-pass run (2026-09-13)

## Rules that bind this run

- Read, in order: `search/README.md` (rules in force, traps), this track's `README.md`, then only the inputs named below. Do not read the other tracks' READMEs except where an input points there.
- Work only inside this track's directory. Never modify `search/README.md`, any other track, the workspace `Cargo.toml`, `CONTEXT.md`, `decisions/`, or `SURFACES.md`. Never commit, push, fetch from the network, or spawn agents.
- Absolute paths under `search/`; never `cd` in a compound command. Extracts are strict JSON or CSV with `lineterminator="\n"`; every `data/` file's first line names the script and the inputs; `raw/` is never committed.
- Every finding traces to a `data/` file or a cited path at a commit. A number you did not measure is marked open.
- Write the README last, from the results, and its headline block last of all: a status line (`Status: first pass complete — 2026-09-13`), at most five headline bullets, findings as tables or short paragraphs headed `**F<n> — …**`, numbers in one table, every open question naming the one experiment that closes it, Provenance, an empty Review table. Budget about 12 KB; per-query detail goes to `data/`.
- Finish with a report of at most 300 words: what landed, what is open, what surprised you, what you could not do. This brief is deleted at the track's close.

## This track

The question, narrowed by the owner's goal of a powerfully simple system: **is a derived search schema in SQLite needed at all, or is an in-memory scan over the parsed corpus already instant at league scale?** The C++ app's unindexed filter loop cost about 0.33 s at a million items (`search/cpp-search/data/delta-pipeline.md`). If the scan is instant at a few hundred thousand items, the simplest engine wins and this track closes early with that number; if not, the measured crossover is the finding.

Inputs (all local, read-only, never committed): the two item-facts stores under `search/item-facts/raw/` (the owner's approval, 2026-09-13; copy with sqlite's `.backup` into this track's `raw/` if you need a writable file — never `cp`, they are under WAL). Columns to materialize: `search/item-facts/data/field-census.csv`. The line identities: the display template (item-facts `mod-templates.csv`) and the export's stat id through `search/repoe/data/template-vs-translation.csv`. The queries: the owner's questions in `search/owner-seat/data/questions.md` (the table), the C++ catalogue in `search/cpp-search/data/filters.toml`, and the trade stat-group semantics in `search/trade-query/data/grammar.json` (`stat_groups`).

Method:
- A throwaway Rust crate at `search/engine-bench/bench/` with its **own `[workspace]` table** so cargo treats it as standalone — never a member of the playground workspace, which the quality gate builds and lints (index trap). Its `target/` is ignored by the root `.gitignore`.
- Corpus: every item of both stores, deduped by id (newest fetch wins, as `item-facts/scripts/census.py` does), about 36,000; then scaled by cloning with fresh ids to about 360,000 — say so wherever the scaled number appears, since the clones repeat the real distribution.
- Engines: (A) an in-memory scan over a compact parsed struct per item (one parse at load; record the load time and the resident bytes); (B) SQLite with a wide attributes table plus an indexed `(item, line, value)` table, built twice — keyed by template text and keyed by stat id — with the build time and file size recorded.
- Queries, named in `data/queries.md`, at most twelve: each owner-seat question that a predicate can express (a mod with a value at least v; a name substring; a base with a mod; a rarity with an ilvl range; the items in one tab; a count of items per tab), a trade-style `count` group (at least 2 of 3 stats) and a `weight` sum, a boolean OR across fields (armour over 1000 or required level under 80), and the C++ Mods filter as it is (two template rows ANDed). Every query runs on every engine at both scales, cold and warm, several repetitions; report the median.

Outputs: `data/queries.md`; `data/results.csv` (one row per query, engine, identity, scale: cold, warm, rows returned); the numbers table (corpus sizes, load and build times, resident bytes, file bytes). Findings: whether the scan is instant and at what scale it stops being; what the derived schema buys and costs; whether the identity choice moves the numbers at all; what the parse costs and whether a projection would remove it (store-as-built Q1, Q5). Keep the crate small and disposable; it dies at the close unless promoted.

Acceptance: a reader can say, with numbers, whether the search needs an engine beyond a scan, and at what corpus size that changes.
