# engine-bench — is an engine beyond a scan needed at all? Measured

Status: first pass complete — 2026-09-13

Headline:
- **The scan is instant at league scale and past it.** Over the owner's real 36,139 items, all twelve queries take 2.6 ms *together*; at a cloned 1,084,170 the median query is 4.5 ms and the whole set 88 ms. Nothing in the twelve reaches 100 ms before ~2.7 M items (F1).
- **SQLite over a derived search schema loses on eleven of twelve, by a median 20× at 1 M** (8.6× at real scale), and five of the twelve pass 100 ms there. It wins exactly one shape: a high-selectivity indexed equality — the one-tab lookup, 2.4× faster. Counting instead of delivering rows changes nothing, so the cost is the engine, not the plumbing (F2).
- **The cost is the load, not the query.** Parsing the corpus is 202 ms at 36 k and 5.9 s at 1.08 M; every query afterwards is free by comparison. Reading a stored projection instead of GGG JSON is 5.2× cheaper (38 ms / 1.14 s), so the choice is a long-lived process holding a corpus, not an engine (F3, F5).
- **Identity does not move latency; it moves reach and bytes.** Template-keyed and stat-id-keyed answers are the same within noise, but the stat id cannot name 32.6 % of mod lines (7,796 of them on equipment) and its tables are 36 % smaller (F4).
- The derived schema costs 765 MB and a 5.7 s build at 1.08 M against 289 MB resident for the scan; at the real 36,139 it is 24.9 MB and 162 ms against 11.0 MB (numbers table).

## Question

The owner's goal is a powerfully simple system, so the question is not which engine is fastest
but whether a second engine is needed at all: is a derived search schema in SQLite warranted, or
is an in-memory scan over the parsed corpus already instant at league scale? The C++ app's
unindexed filter loop cost about 0.33 s at a million items and was the one cost it never attacked
(`../cpp-search/README.md` F6, `data/delta-pipeline.md`).

## Inputs

| Source | Access | Used for |
| --- | --- | --- |
| `../item-facts/raw/` — the spike facts store and the C++ user store (the owner's account, approval 2026-09-13) | `.backup` copies in `raw/`, never `cp` (WAL); `MANIFEST.md` | the corpus: every item of both stores, deduped by id, newest fetch wins |
| `../item-facts/data/field-census.csv` | this directory | which columns the wide table and the struct materialize |
| `../repoe/data/template-vs-translation.csv` | this directory | identity B: display template → trade stat id |
| `../owner-seat/data/questions.md`, `../cpp-search/data/filters.toml`, `../trade-query/data/grammar.json` (`stat_groups`) | this directory | the twelve queries and their thresholds |

## Outputs

| File | What |
| --- | --- |
| `scripts/build-corpus.py` | the two stores → `raw/corpus.jsonl`, 36,139 items, 0 skipped |
| `scripts/identity-coverage.py` | `data/identity-coverage.csv`: what the stat identity can and cannot name |
| `bench/` | the throwaway Rust crate (its own `[workspace]`, never a member, `target/` gitignored); both engines, release build; the `--seat` mode of F7 |
| `data/queries.md` | the twelve queries: what each stands for, its predicate, its row count |
| `data/results.csv` | 216 rows: query × engine × identity × scale, cold and warm-median ms, rows |
| `data/numbers.csv` | corpus sizes, load and build times, resident bytes, file bytes |
| `scripts/seat-projection-check.py` | `data/seat-projection.md`: the seat projection read-only — DDL, counts, `kind` histogram (F7) |

## Findings

**F1 — The scan is instant, and stays instant well past league scale.** One pass over a compact
struct per item, with template, stat id, array, tab and league interned to `u32` at load; no index
of any kind. Warm medians, template identity, in ms:

| Corpus | median query | worst query | all twelve together |
| ---: | ---: | ---: | ---: |
| 36,139 (real) | 0.17 | 1.27 (`q03-name-substring`) | 2.6 |
| 361,390 (×10 clones) | 1.31 | 13.1 (same) | 28.1 |
| 1,084,170 (×30 clones) | 4.47 | 40.8 (same) | 88.0 |

Per-million cost is flat across the three scales (`q03` 35.5 → 37.6 ms/M; `q08` 6.45 → 6.30),
so the scan is linear and its crossover can be read off: at 100 ms the substring query
arrives at ~2.7 M items, the mod-value queries at ~23 M, the plain attribute queries at ~55–69 M.
The scan produces the matching id set, not a count; cold is within 4 % of warm.

**F2 — The derived schema loses, and counting instead of delivering rows does not save it.**
A wide `items` table (18 columns, six indexes) plus an indexed `lines(item, arr, line, value)`,
`ANALYZE` run after the build:

| Corpus | median sqlite/scan | worst | queries over 100 ms | queries SQLite wins |
| ---: | ---: | ---: | ---: | --- |
| 36,139 | 8.6× | 80× (`q12-leveling-set`) | 0 of 12 | none (`q06` 1.3×) |
| 361,390 | 16.8× | 80× (same) | 0 of 12 | `q06-one-tab` (0.42×) |
| 1,084,170 | 20.5× | 85× (same) | 5 of 12 | `q06-one-tab` (0.42×) |

The same SQL wrapped in `count(*)` runs within 4 % of the row-delivering form at every scale, so
the gap is the engine's own work — btree descents, row decoding, the join back to `items` — and
not rusqlite's row plumbing. What the schema buys is the shape it is built for: `tab = 'Flasks'`
is 0.30 ms against the scan's 1.57 ms at 1.08 M, an index turning 1 M comparisons into 25 k.
Every broader predicate is worse off, and the two trade-style group queries (`count`, `weight`)
are its worst cases — 130 ms and 203 ms at 1.08 M, against 6.8 and 7.1 for the scan.

**F3 — The load is the real cost, and it decides the shape, not the engine.** Parsing GGG JSON
into the struct is 5.4 µs per item: 202 ms at 36,139, 5.9 s at 1,084,170. Against that, every
query is free — but only a process outliving one query amortizes it. A process-per-command CLI
at 1 M items would pay 5.9 s to answer a 4.5 ms question, where SQLite's 77 ms wins by two orders
of magnitude. The spike already has the long-lived process (`acqd`), so the scan is available to
it; a search running inside `acq` itself would not have it. This is what the choice turns on.

**F4 — The identity choice moves reach and bytes, not latency.** The two identities answer the
twelve within noise on the scan (±5 % on ten of twelve, two mod-heavy queries swinging ±25 % run
to run) and 4–11 % apart on SQLite, tracking the smaller table and not the identity. What differs
is what can be named
(`data/identity-coverage.csv`, which holds the split per array and per template): 40,793 of
125,006 mod lines (32.6 %) carry no trade stat id — 23,796 gem skill text (58 %), 9,201 map,
ultimatum and flavour-shaped lines (23 %), and 7,796 equipment lines (19 %), the ones a gear
search wants. The stat id also merges variants (implicit/explicit/fractured/crafted of one stat)
that the template keeps apart, and 259 of the 3,478 mapped templates are ambiguous (several
entries; the first trade id wins). Keyed by stat id the `lines` table is 33 % shorter and the
file 36 % smaller.

**F5 — A projection removes four fifths of the load (store-as-built Q1, Q5).** Rebuilding the
same in-memory structs from the derived tables instead of from GGG JSON: 38 ms at 36,139
(against 202 ms) and 1.14 s at 1,084,170 (against 5.9 s) — 5.2× cheaper at both ends. The body
is not needed to *find* anything: the 18 columns and the line rows answer all twelve queries, and
keeping them makes a restart cheap. Resident cost is the same order either way (392 MB from the
projection against 289 MB from the parse at 1.08 M; the difference is per-item `Vec` slack).

**F6 — Against the C++ app.** Its filter loop was ~0.33 s at ~1 M items and was deferred as an
accepted cost (M3 D7); this scan runs all twelve predicates over 1.08 M in 88 ms, worst single
query 41 ms. Not a like-for-like — different machine and language, one predicate against 38 ANDed
filters — but it bounds the question: the loop the C++ app could not afford to fix is not
intrinsically expensive.

**F7 — The seat's projection** (2026-09-14; `raw/seat-projection.db`, 30.8 MB against the timed
file's 24.9 MB, ~0.5 s to build; its DDL, counts and `kind` histogram in `data/seat-projection.md`).
`cargo run --release --offline -- --seat`: a second mode writing a separate file for agent-seat's
phase two, so the timed schema and the twelve queries stay exactly as measured. It adds only what
a ruling already requires — the GGG fields the bench's columns were derived from, the whole
location coordinate (realm above league; the corpus carries no first-seen), and per line the C++
bucket as `kind`, its GGG `flags` and its numbers kept apart (`n`, `n0`, `n1`, `numbers`) beside
the mean, which hid a second number on 11,020 of the 125,006 lines — every column described by a
`--` comment inside its `CREATE TABLE`, which
`sqlite_master` keeps, so `.schema` is the manual. Nothing speculative went in: the arrays the C++
never read keep their array name as `kind`, and `ultimatumMods`, whose entries carry no display
line at all (`tier`/`type`), is a row in neither file. What else is missing is the seat's finding.

## Numbers

Apple M4, macOS 26.6.2, 32 GB; rustc 1.94.1, release profile; bundled SQLite 3.45.3
(`journal_mode=OFF`, `synchronous=OFF` for the build). Warm = median of 7; cold = the first
run on a fresh connection (SQLite's page cache empty; the OS file cache stays warm).

| | 36,139 items (real) | 361,390 (×10) | 1,084,170 (×30) |
| --- | ---: | ---: | ---: |
| Corpus JSONL bytes | 53,569,958 | — (cloned in memory) | — |
| Mod lines | 125,006 | 1,250,060 | 3,750,180 |
| Scan: parse + build (ms) | 202 | 1,918 | 5,884 |
| Scan: load from the projection (ms) | 38 | 373 | 1,142 |
| Scan: resident bytes | 10,982,951 | 96,894,800 | 288,532,800 |
| SQLite build, template / stat (ms) | 162 / 117 | 1,877 / 1,366 | 5,687 / 4,209 |
| SQLite file, template / stat (bytes) | 24,911,872 / 15,863,808 | 253,739,008 / 162,639,872 | 764,694,528 / 489,926,656 |
| SQLite line rows, template / stat | 125,006 / 84,213 | 1,250,060 / 842,130 | 3,750,180 / 2,526,390 |
| Warm median over the twelve: scan / SQLite (ms) | 0.17 / 1.55 | 1.31 / 25.5 | 4.47 / 76.9 |
| Worst of the twelve: scan / SQLite (ms) | 1.27 / 3.20 | 13.3 / 56.9 | 40.8 / 202.7 |

Distinct templates 6,513; distinct stat ids 2,939; bodies skipped as malformed: 0 of 36,139.
The ×10 and ×30 corpora are clones with fresh ids, so they repeat the real distribution exactly
— selectivity per query is identical at every scale, and the scaled rows measure size, not variety.

## Open questions

| # | Question | The one experiment that closes it |
| --- | --- | --- |
| Q1 | Does the ranking hold with a genuinely cold OS page cache — the first search after a boot? | Rerun the twelve at ×30 after `sudo purge`, from a terminal; compare the cold column. |
| Q2 | Is the substring scan, the only query approaching 100 ms before 3 M items, the shape that eventually needs help? | Build an FTS5 index over the pretty names at ×30 and time `q03` against it and against a packed-name-arena scan. |
| Q3 | What does an update cost each engine — a tab refetched, 600 items replaced — with six indexes live? | Time deleting and reinserting one tab's items in the derived schema against the same mutation on the `Vec`, at ×30. |
| Q4 | Does the struct stay this cheap if it materializes every field a search might want, not the 18 the queries needed? | Extend the struct to every census path over 1 % of items and re-measure load and resident bytes. |
| Q5 | Does the scan hold when several searches run at once against one corpus? | Run the twelve concurrently on N threads over a shared corpus at ×30 and compare per-query medians. |

## Provenance

| What | Where |
| --- | --- |
| The corpus, the store copies and the seat projection | `MANIFEST.md`; `raw/` is gitignored and never committed |
| The measurements | `data/results.csv`, `data/numbers.csv`, both written by `bench/src/main.rs` (`cargo run --release --offline`) |
| The queries, their sources, and the identity mapping | `data/queries.md`, `data/identity-coverage.csv`; `../owner-seat/data/questions.md`, `../cpp-search/data/filters.toml`, `../trade-query/data/grammar.json`, `../repoe/data/template-vs-translation.csv` |
| The C++ comparison | `../cpp-search/README.md` F6 and `../cpp-search/data/delta-pipeline.md`, read at `master@946a4f51` |

## Review

| Finding | Raised by | Fix |
| --- | --- | --- |
| F3 names `acqd` as the long-lived process that could hold the corpus. The daemon writes facts and never reads them (C2, C34; `README.md`, the store), so the holder F3 argues for is a frontend-side long-lived process — the reopening case note 00 set for a third leg (C48; `decisions/store.md`, "Parked: search-at-scale"), now with its measured number. At the real 36,139 the per-command load is 202 ms from JSON and 38 ms from a projection, inside a CLI's budget; the finding bites at the scaled corpus. | Fable, 2026-09-13, at the commit | this row; F3's finding stands with the corrected home |
