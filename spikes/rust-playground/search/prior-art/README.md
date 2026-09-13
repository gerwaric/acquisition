# prior-art — how a maintained item-to-trade tool gives a line its identity

Status: not started — 2026-09-13

Headline: (none yet)

## Question

Awakened PoE Trade turns an item's clipboard text into a trade query
and has lived for years with what the repoe track just measured: the
local/global twins, several trade ids for one displayed line, lines
fed by several mods, and renames across patches. How does its stats
dataset represent a stat — one entry per displayed text, several
trade ids per entry, matchers for the local twin, negation, options;
how is that dataset generated and from what (the trade endpoint, the
game data, both); and what does it do when the game renames a line or
a patch adds a stat. The answer is evidence for the one undecided
design question — a line's identity — and for repoe Q8's update
posture, from a working system rather than an argument. Everything
below about its layout is memory and is verified at the read.

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| `github.com/SnosMe/awakened-poe-trade` at a pinned commit: the stats dataset (`renderer/public/data/en/*.ndjson`, to verify), the item-text parser, the trade-query builder | `clone` — the owner clones beside this repository; tooling reads the checkout, cited by commit (`SURFACES.md`) | the owner |
| The dataset generator, if it lives in a sibling repository | the same, if the owner clones it | the owner |
| `../repoe/data/trade-stat-map.csv`, `../trade-query/data/stats-2026-09-12.json` | this directory | the sibling tracks |

## Outputs planned

- `data/stat-model.md` — the shape of one stat entry, with three real
  entries (a plain stat, a local twin, a several-ids stat), one table.
- `data/coverage.csv` — its stat entries joined to the trade capture
  and to the export's text join: how many of the 380 collisions it
  carries as several ids, how many it resolves, how.
- The findings table: identity, ambiguity handling, update handling,
  and what it does not attempt.

## Findings

## Open questions

## Provenance
