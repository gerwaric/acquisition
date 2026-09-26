# Brief — pseudo-stats, second pass, part 1 (a research track; run by a subagent)

You are an Opus subagent running one research track of the item-search
slice under `search/README.md`, "Rules in force". Read this brief, then
`search/README.md` (rules and traps), then this track's `README.md` and
`MANIFEST.md`, then the inputs below, in that order. Work only inside
`search/pseudo-stats/`. Never touch the index (`search/README.md`),
another track, `CONTEXT.md`, `decisions/`, `SURFACES.md`,
`brainstorming-notes/`, the crates or `tools/`. Never commit, push,
fetch, open a link, run a network request or spawn an agent: the links
this track writes are opened by the owner, in a browser, and by no one
else. The reviewer sets the index row and commits; this brief is
deleted at the close, cited by hash.

## The question

The search's computed values borrow the trade site's names, and each is
to count what the site's pseudo of that name counts (the plan, step 9c).
The site publishes the names and never the definitions: a definition is
what the site answers when asked. The owner will ask it, by opening
searches this track writes. Part 1 is everything that can be settled
before he does, from what is already committed or cloned:

1. For each pseudo in scope, which lines might contribute, who says so,
   and which search would decide it.
2. How the site's Base Percentile is computed, tested against the items
   already captured that carry the site's own value.

Part 1 decides no contributor. It finds the candidates, says what each
rests on, and writes the searches. What the site answers is part 2.

## Scope

- The ranged family: the 21 `pseudo.pseudo_adds_*` entries.
- `pseudo.pseudo_total_life`.
- The 35 totals the build ships
  (`crates/acquisition-search/reference/totals-v1.toml`), for the lines
  the coverage trial found no total counts (`search/MEASUREMENTS.md`,
  the block headed **M6**, read by that block alone).
- `pseudo.pseudo_base_defence_percentile`.

Every other entry of `data/pseudo-classes.csv` keeps its row as it is.
The site's other sum-like pseudos follow this pass by the same method,
so nothing you write may assume the scope is closed: a pseudo is a row,
never a rule in a script.

## The candidate table

One row per pair (pseudo, line template). A pair is a candidate when any
source below names the line under that pseudo, or when the template's
own words put it in reach of the pseudo's text. A row's `status` is one
of these and nothing else:

| Status | Means | Admitted by |
| --- | --- | --- |
| `site` | a committed capture shows the site counting the line under this pseudo, or returning the pseudo without it | `search/trade-query/data/fetch-census.json`, by locator |
| `sources-agree` | two or more of the separate sources name the line, at one weight | a locator in each |
| `sources-differ` | two sources name it at different weights, or one names it and another, listing this pseudo's lines, leaves it out | a locator in each |
| `one-source` | one source names it and no other speaks of this pseudo | its locator |
| `text-only` | no source names it; the template's words alone make it a candidate | the template's row in `mod-templates.csv` |

What puts a template in reach of a pseudo by its words is a table in
`scripts/candidates.py`, one entry per pseudo, printed in the README:
the reader must be able to see why a template is a `text-only`
candidate and why another is not. Templates of one conditional shape
(`during Flask effect`, `while on Low Life`, a minion's, a monster's, a
passive's grant) are one family: each keeps its row, the family names
one representative for the sheet, and the others say `family:` in their
note, as the first pass's rooms do. Drop no shape because it looks
unlikely; the count of what a family holds is the finding.

Never raise a row's status on likelihood. Whether the three tools are
independent readings of the site is not known, and agreement among them
is not the site's answer. Whether a `sources-agree` row is admitted
without a search is the owner's ruling, made on your counts: count, and
do not decide.

## Inputs (by path; read nothing else for evidence)

Committed:

- `search/pseudo-stats/data/pseudo-classes.csv`, `README.md` — the first
  pass; open questions 1, 2, 3 and 5 are this pass's.
- `search/trade-query/data/stats-2026-09-12.json` — every stat id and
  its text; `stat-collisions.csv` — the texts two ids display.
- `search/trade-query/data/fetch-census.json` — seven searches, 70
  fetched items; 30 carry `extended.base_defence_percentile`.
- `search/trade-query/data/grammar.json` — the group types and their
  tips (`stat_groups`).
- `search/cpp-search/data/pseudomods.toml` — the C++ app's 35 tables.
- `crates/acquisition-search/reference/totals-v1.toml` — what ships.
- `search/item-facts/data/mod-templates.csv` — the owner's templates,
  with counts (the index's trap: open with `newline="\n"`).
- `search/repoe/data/base-defences.csv`, `mod-stat-index.csv`,
  `trade-stat-map.csv`, `template-vs-translation.csv`.

Cloned beside `acquisition/` by the owner, read at the commit named and
refused at any other (`search/repoe/MANIFEST.md`,
`search/prior-art/MANIFEST.md`):

- `poe1/` at `e2bd511a` — `data/stats.json` (a stat's locality),
  `data/mods.json` (which stats a mod carries, its domain and spawn
  tags): for the ranged family, which stats display each template, which
  are local, and on which classes they can appear.
- `PathOfBuilding/` at `16de4b82` — `src/Classes/Item.lua`, the block
  that sets `BasePercentile` from an item's displayed defences;
  `src/Classes/TradeQueryGenerator.lua`, the map from a stat to a
  pseudo.
- `awakened-poe-trade/` at `ce551eb7` —
  `renderer/src/web/price-check/filters/pseudo/`, its rules for which
  lines it folds into which pseudo, and at what multiplier.

A tool's code is a third party's reading of the site: a source of
candidates and of a formula to test, never the site's answer. Commit no
file of theirs; a locator is a path and a line range at the commit.

Absent inputs: a clone missing, at another commit, or a file lacking
what is named here is an open question in the README, and you continue.

## Outputs

- `data/candidates.csv` — columns
  `pseudo,pseudo_text,template,stat_ids,status,weights,sources,corpus_items,local,search,note`.
  `stat_ids` is every id displaying that text, `;`-separated; `sources`
  is `;`-separated locators, each opening; `weights` the weight each
  source gives, in the same order; `local` what the export says of the
  stats behind the template, empty where it says nothing; `search` the
  sheet row that decides the pair, empty only for status `site`.
- `data/percentile-check.csv` — one row per captured item carrying the
  site's value: the item's base, its displayed defences and quality, the
  lines read as local, the site's value, and the value each formula
  under test gives — as the lowest and the highest it could be, since a
  displayed number is rounded and the base's roll is recovered from it.
- `scripts/candidates.py`, `scripts/percentile.py` — each regenerates
  its file byte-for-byte and prints its counts. Strict CSV,
  `lineterminator="\n"`, a header line, no comment line.
- `scripts/search-sheet.py`, extended — it writes
  `data/search-sheet.csv`. Keep `compose`, `--self-test` and the two
  pilot rows as they are. Add the batch, generated from
  `data/candidates.csv`, in two forms chosen by a flag, since the
  pilot's answer is not in yet: `--method if` (one search per candidate
  line, every pseudo it might feed in one `if` group) and `--method and`
  (one search per pair). Every search names what it decides and its
  control — a pseudo the returned items are known to show, so that a
  value absent is the site's answer. A line two ids display goes out as
  a `count` of at least one. Order the sheet by what a search is worth:
  `sources-differ` first, then `text-only`, `one-source`,
  `sources-agree`. Add the searches the percentile still needs.
- `README.md` — written last, from the scripts' output, its headline
  block last of all. Keep the first pass's sections, corrected where
  this pass overtakes them. Add: the candidate counts by status, one
  table; the ranged family by what the export says is local; the
  percentile's formula as far as the captures pin it, the items it
  misses and by how much, and what no capture reaches; the searches by
  form, counted. Close open questions only on evidence: a question a
  search will close stays open and names its sheet row. About 12 KB is
  the index's guide; per-row detail is `data/`.

## Findings expected

- For the ranged family: whether the export separates the unsuffixed
  `Adds # to # <type> Damage` on a weapon from the same text elsewhere,
  and what the owner's corpus holds of each.
- For the percentile: every formula a source states, each scored
  against the 30 items, none preferred before it is scored; whether the
  value can be recovered exactly from what a private item displays.
- The number of searches in each form, since the owner's time is what
  the sitting costs.

## Acceptance

Each script regenerates its file byte-for-byte; every locator in
`data/candidates.csv` opens at the commit named; no row's status is
above what its locators show; `python3
search/pseudo-stats/scripts/search-sheet.py --self-test` reports none
differing; every link in `data/search-sheet.csv` decodes to the query
beside it; every search names a control; the pilot rows are unchanged;
no file outside `search/pseudo-stats/` is touched and no request was
made.

## The report (at most 300 words)

When done, report: the candidate counts by status; the searches by
form; how many of the 30 items each percentile formula reproduces; what
you left out; the one cut you would most want reversed. The reviewer
restores from that line.
