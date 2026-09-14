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
posture, from a working system rather than an argument. The layout
below was verified on the owner's clone (`MANIFEST.md`): one stats
entry is `{ref, better, matchers[{string, value?}], trade.ids{category:
[ids]}}` — several ids per entry, by category, as a normal case.

## Inputs

| Source | Access | Supplied by |
| --- | --- | --- |
| `github.com/SnosMe/awakened-poe-trade` at `ce551eb7` (`MANIFEST.md`): `renderer/public/data/en/stats.ndjson` and `items.ndjson`, the item-text parser, the trade-query builder | `clone` — cloned by the owner 2026-09-13 beside this repository; tooling reads the checkout, cited by commit (`SURFACES.md`) | landed |
| The dataset generator — not in the clone, a sibling project | the same, if the read needs it and the owner clones it | the owner |
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
