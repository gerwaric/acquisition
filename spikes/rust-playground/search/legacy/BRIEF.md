# Brief — legacy, second pass (a research track; run by a subagent)

You are an Opus subagent running the second pass of one research track
of the item-search slice under `search/README.md`, "Rules in force".
Read this brief, then `search/README.md` (rules and traps), then
`search/legacy/README.md` and its `scripts/`, then the inputs below.
Work only inside `search/legacy/`. Never touch the index
(`search/README.md`), another track, `CONTEXT.md`, `decisions/`,
`SURFACES.md` or `brainstorming-notes/`. Never commit, push, pull,
fetch, run a network request or spawn an agent. The reviewer sets the
index row and commits; this brief is deleted at the close, cited by hash.

**A measurement, no design.** S178 stands. Nothing here proposes a
field, a word, a rule for the search or a recommendation; a finding is
a count, a table or a sentence about what an input holds. The inputs
rule of the first brief (at `f7ab0957`) holds: an unlisted input
already on this machine under the surface register may be read, with a
manifest row, and is named in the report; one that is not here is never
read or fetched — it is an open question, first in the report.

## The question

The first pass (`1cf77994`) fitted one variant at a time and rescaled
nothing. Its README shows two places where that rule does not follow
what Path of Building's own parser does with the same entry, and the
owner has asked for both to be fixed (2026-09-18: "go ahead with both
fixes including catalyst scaling"):

1. 147 `none` items sit on entries with a `Has Alt Variant` header,
   where an item carries a selection of two or more variants at once.
2. 61 uniques carry catalyst quality; 46 are `none` on a number
   outside every range, and 6 fit a variant unscaled — a fit nothing
   guarantees is the right one.

How do the outcome counts move when the fit follows Path of Building
on both, and what do the owner's items on those entries — his three
catalysed Ashes of the Stars among them — read then?

## Definitions (fixed; state any you had to sharpen, never swap one)

Everything the first brief defined stands — population, entry,
binding, old, the two-way fit over implicit and explicit lines — with
the sharpenings the first README lists. Added:

- **Selection.** For an entry with no alt-variant header, a selection
  is one variant, as before. For an entry with `Has Alt Variant`,
  `Has Alt Variant Two`, … headers, a selection is as many variants as
  Path of Building lets be chosen at once — one plus the alt headers
  the entry carries — and a line belongs to the selection when any
  chosen variant lists it (an untagged line belongs to every one).
  Read `src/Classes/Item.lua` for how it applies `variantAlt` and its
  siblings, including a repeated choice, follow it, and cite the lines.
- **Fit, outcome, either — over selections.** A selection fits as a
  variant did, over its lines. `one` / `several` / `none` count fitting
  selections. A selection is *old* when any chosen variant is old;
  *either* is a fitting set holding an old and a not-old selection.
- **Catalyst rescale.** For an item with catalyst quality *q* of a
  kind, a variant line is scaled when its `{tags:…}` prefix in the
  unique file holds a tag the kind affects — the kind-to-tags table is
  `Item.lua` lines 14–29, copied with its citation. A scaled line's
  range endpoints and fixed values are multiplied by `(100 + q) / 100`
  and floored toward zero at the line's precision, as
  `itemLib.formatValue` does (`src/Modules/ItemTools.lua`); the item's
  displayed value is never un-scaled. Say what Path of Building treats
  as unscalable and follow it. The two kinds that key on prefix and
  suffix have nothing to match on a unique's lines: count such items
  apart, unscaled. Trap: the items spell the kind as the game does
  (`Elemental Damage`, `Physical and Chaos Damage` — the first pass's
  `catalyst_descriptor` column); `Item.lua`'s descriptor list spells
  two of them shorter. Map by what the items show and state the map.
- **Strict beside final.** The first pass's rule — one variant, nothing
  rescaled — stays as its own column and must reproduce the first
  pass's counts exactly. The final outcome applies both fixes. Every
  item whose outcome differs is attributed: `selection`, `catalyst`,
  or `both` (neither fix alone moves it).

## Inputs

As the first brief: the Path of Building clone at
`16de4b82d57f1c0de6eb40f37143c32d4da36a02` (every script refuses
another commit; CRLF), now including `src/Classes/Item.lua` and
`src/Modules/ItemTools.lua` by hand; the two store backups already in
`raw/` (rows in `MANIFEST.md`; account data, never committed, never
quoted beyond name, base, mod lines and tab name); the first pass's
scripts and data in this directory, which you edit in place.

## Outputs

- `scripts/variants.py`, `common.py`, `match.py` edited in place;
  `arrays.py` untouched. `data/pob-variants.csv` gains the entry's
  selection size and each variant's tagged-line count.
  `data/unique-fit.csv` gains `outcome_strict`, `either_strict`, the
  final `outcome` and `either` over selections with rescale,
  `moved_by`, and the lines rescaled (count). Strict CSV,
  `lineterminator="\n"`; the scrub guard stays.
- `match.py` prints: the strict counts; the final counts; the
  strict-to-final transition table; movers by `moved_by`; the `none`
  reasons under the final rule; every catalysed item that fitted under
  the strict rule with what it fits after rescale; the Ashes of the
  Stars rows; the alt-variant entries with the owner's items by final
  outcome.
- `README.md` updated in place, not appended to: the headline block
  and the numbers table carry the final counts with the strict ones
  beside them; "Why a unique fits no variant" is restated under the
  final rule; the Ashes of the Stars table is restated; one new table
  for the alt-variant entries and one for the catalysed items that
  fitted unscaled. The sharpened-definitions list gains what you
  sharpened here. Leave "The arrays outside implicit and explicit",
  "The sources read" and the `## Review` section exactly as they are —
  the Review rows are the reviewer's. The page is already past the
  index's guide; replace a superseded number rather than keep both
  tellings, put per-item detail in the CSV, and cut nothing from the
  sources-read table. `MANIFEST.md` gains rows for the files newly
  read.

## Acceptance

`python3 search/legacy/scripts/variants.py`, `match.py` and
`arrays.py` regenerate every `data/` file byte-for-byte;
`data/other-arrays.csv` is unchanged from `1cf77994`; the strict
column's counts are 4,020 / 229 / 767 / 761 with 171 either; the
transition table sums to 5,777 by row and by column; every mover has a
`moved_by`; every number in the README is printed by a script; the
README states what each of the owner's catalysed Ashes of the Stars
fits after rescale; the `## Review` section is byte-identical; no
sentence proposes a design.

## The report (at most 300 words)

When done, report: first, any input you wanted that is not on this
machine, then any unlisted one you read; the final counts beside the
strict ones, the final either count, movers by cause; what the six
strictly-fitting catalysed items read after rescale; the Ashes of the
Stars reading; what you left out; the one cut you would most want
reversed. The reviewer restores from that line.
