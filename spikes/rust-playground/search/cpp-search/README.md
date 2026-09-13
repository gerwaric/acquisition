# cpp-search — the C++ app's search, catalogued

Status: not started — 2026-09-12

Headline: (none yet)

## Question

What does the C++ app's search do, filter by filter and column by
column: the exact extraction rule behind each, the derivation formulas
(DPS, sockets and links, colours, pseudomods, the mod normalizer), the
rarity and category taxonomies, the sorts and view modes, and what the
delta pipeline paid to keep results instant?

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| `master` at a named commit: `src/filters/`, `src/search.*`, `src/item.*`, `src/modlist.*`, `src/pseudomods.cpp`, `src/column.*`, `src/items_model.*` | `git show master:<path>` | — |
| `docs/user/searching.md`, `docs/user/mods-and-pseudomods.md`, `docs/design/items-pipeline.md` (master) | same | — |

## Outputs planned

- `data/filters.toml` — one entry per filter: caption, group, payload
  kind, the extraction rule in words, the JSON fields it reads, refresh
  mode.
- `data/columns.toml` — one entry per table column with its value rule.
- `data/pseudomods.toml` — each pseudomod and the templates it sums.
- The derivation formulas as one table (DPS variants, links, colours).

## Findings

## Open questions

## Provenance
