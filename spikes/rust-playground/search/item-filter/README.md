# item-filter — GGG's own predicate language over an item

Status: not started — 2026-09-13

Headline: (none yet)

## Question

The in-game loot filter is a GGG-authored, documented, player-known
language of predicates over exactly the items the store holds: class,
base type, rarity, item level, sockets and links, socket groups,
influence, enchantment, map tier, gem level, quality, transfigured,
corrupted, mirrored, an explicit mod by name. What predicates does it
name, with what operators, vocabulary and semantics; which of them
the C++ catalogue (cpp-search F1) and the trade filters (trade-query
F3) also name, under which spelling; and which a stash search inherits
as a third, GGG-authored naming of the same things — its class names
being one more source for the taxonomy (repoe F3).

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| GGG's item-filter documentation, `pathofexile.com/item-filter/about` (the syntax, every condition and action) | `browser` — the owner saves a dated copy under `raw/`; the extraction is what is committed (`SURFACES.md`) | the owner |
| One or two widely used filters as worked examples of the grammar in practice (NeverSink's, at a pinned release) | `clone` or a saved copy under `raw/`, cited by release | the owner |
| `../cpp-search/data/filters.toml`, `../trade-query/data/grammar.json` | this directory | the sibling tracks |

## Outputs planned

- `data/conditions.csv` — one row per condition: name, operand type,
  operators, the item field it reads, the C++ filter and the trade
  filter that read the same thing (or none), notes.
- `data/class-names.csv` — the class vocabulary the documentation
  lists, joined to the export's class names (repoe
  `class-to-trade-category.csv`).
- The findings table: what the three namings agree on, where they
  diverge, and what only the filter language names.

## Findings

## Open questions

## Provenance
