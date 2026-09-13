# item-facts — the item as GGG gives it

Status: not started — 2026-09-12

Headline: (none yet)

## Question

What does an item look like in the facts the store already holds, field
by field and class by class, and how big is the corpus a search must
serve? The projection from verbatim JSON to a searchable attribute
vector starts from this census.

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| A real facts database (`<store dir>/ggg/<account>.db`, facts v7) | read-only copy in `raw/` | the owner names the path |
| The store's schema and derived columns (`crates/acquisition-store/src/schema.sql`, `lib.rs` `rebuild`) | the repo at the commit in the status line | — |
| The API reference's item type (`pathofexile.com/developer/docs/reference`) | sanctioned; read online | — |

## Outputs planned

- `data/field-census.csv` — one row per JSON field: path, item classes it
  appears on, count, share, example value.
- `data/mod-templates.csv` — distinct display-line templates (`#` for
  numbers) with counts, by mod array (implicit, explicit, crafted, …).
- The numbers table: items, tabs, characters, distinct templates, mods
  per item, JSON bytes per item, database size.

## Findings

## Open questions

- **The mod line format, by date and array.** trade-query observed on 2026-09-13, reading the spike's own store (json verbatim, C29): `explicitMods` and `implicitMods` are objects `{description, flags?}` on every item seen 2026-09-02 to 2026-09-11, both realms, while `enchantMods`, `utilityMods`, `veiledMods` and `crucibleMods` are still strings; `rarity` and `frameTypeId` appear beside `frameType`. The API reference says strings. Census it here: when it changed, which arrays, and whether a private line ever carries a `hash` (the trade `fetch` lines do; none of 71,695 stored lines did).

## Provenance
