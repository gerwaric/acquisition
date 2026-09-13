# store-as-built — what a search consumer gets from the store today

Status: not started — 2026-09-13

Headline: (none yet)

## Question

Search sits on the store crate's read API under C48 (raw SQL is not a
surface), C34 (derivations reproducible from declared inputs) and C12
(two surfaces). What does the crate expose to a frontend today — the
facts tables and their columns, the neutral snapshots, `item_events`,
liveness and freshness, the realm and container coordinates — and what
of an item is a column against what is verbatim JSON; what a search
consumer can build without a store change and where the first store
change would fall. Read from the code, so the design session starts
from what is rather than from memory of it.

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| `crates/acquisition-store/src/lib.rs` ("As built"), `snapshot.rs`, the schema and its migrations | the repository | — |
| `decisions/store.md` (C28–C63 and its parked search-at-scale entry), `CONTEXT.md` (C34, C48, C12) | the repository | — |
| `../item-facts/data/field-census.csv` (what the JSON carries, to set against what the columns carry) | this directory | the item-facts track |

## Outputs planned

- `data/read-surface.md` — one table: every public read type and
  function, what it returns, what it is keyed on, its cost class.
- `data/columns-vs-json.csv` — the items row's columns beside the JSON
  paths they are lifted from, and the paths nothing lifts.
- The findings table: what a search can do through the surface as it
  stands, and the first thing it cannot.

## Findings

## Open questions

## Provenance
