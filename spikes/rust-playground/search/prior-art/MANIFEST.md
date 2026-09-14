# Evidence manifest — prior-art

Every input this track reads and how it was obtained (index rule 4). Awakened PoE Trade is read from the owner's clone only (`SURFACES.md`, its row: `clone`, cited by commit; no fetch by tooling, no runtime dependency; extracts committed with the source cited, its data files never).

| Input | Access | Pinned at | What is read |
| --- | --- | --- | --- |
| `../../../awakened-poe-trade/` — `github.com/SnosMe/awakened-poe-trade`, MIT (© 2020 Alexander Drozdov) | cloned by the owner 2026-09-13 beside the repository | `ce551eb7a9b704fbdcc2478eebb26be8f91786c7` (2026-09-09, "bump site ver"; `v3.29.108-1-gce551eb`) | `renderer/public/data/en/stats.ndjson` (2,565,295 bytes: one entry per stat — `ref`, `better`, `matchers[]`, `trade.ids{category: [ids]}`), `renderer/public/data/en/items.ndjson` (1,459,983 bytes), `renderer/src/parser/` (the item-text parser), the trade-query builder under `renderer/src/web/`. The dataset generator is not in this repository (no `dataParser`; a sibling project) — whether it is cloned is the read's call, recorded here if so |
| `../repoe/data/trade-stat-map.csv`, `../trade-query/data/stats-2026-09-12.json` | this directory | their manifests | the export's text join and the trade capture, for the coverage join |
