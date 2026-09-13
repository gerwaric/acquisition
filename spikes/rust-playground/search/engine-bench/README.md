# engine-bench — is SQLite over a derived schema instant? Measured

Status: not started — 2026-09-12

Headline: (none yet)

## Question

Materialize a wide attributes table and an indexed (item, stat, value)
table from real facts, scale a copy to a few hundred thousand rows, run
the C++ form's representative queries and trade-style stat groups, and
measure latency and file size — against an in-memory Rust scan, so the
engine choice is evidence rather than argument.

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| A real facts database, copied and scaled | `raw/`; never committed (account data) | the owner names the path |
| `item-facts/data/` — the census that says which columns to materialize | this directory | the item-facts track |
| `cpp-search/data/filters.toml` — the queries to time | this directory | the cpp-search track |

## Outputs planned

- `data/queries.sql` (or `.json`) — the timed queries, named.
- `data/results.csv` — one row per query per corpus size per engine:
  cold and warm latency, rows returned.
- The numbers table: corpus sizes, database bytes, build time of the
  derived tables.

## Findings

## Open questions

## Provenance
