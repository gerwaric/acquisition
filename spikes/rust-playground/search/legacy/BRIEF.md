# Brief — legacy (a research track; run by a subagent)

You are an Opus subagent running one research track of the item-search
slice under `search/README.md`, "Rules in force". Read this brief, then
`search/README.md` (rules and traps), then the inputs below, in that
order. Work only inside `search/legacy/`. Never touch the index
(`search/README.md`), another track, `CONTEXT.md`, `decisions/`,
`SURFACES.md` or `brainstorming-notes/`. Never commit, push, pull,
fetch, run a network request or spawn an agent. The reviewer sets the
index row and commits; this brief is deleted at the close, cited by hash.

**A measurement, no design.** S178 stands (R8: "the app never judges
what the game can produce"). Nothing here proposes a field, a word, a
rule for the search or a recommendation to the owner; a finding is a
count, a table or a sentence about what an input holds.

An input not listed below that is already on this machine under the
surface register — another file of a registered clone, another track's
`data/` or `raw/` — you may read, read-only; give it a manifest row and
name it in the report. An input that is not here — the wiki, poedb, the
trade site, a newer pull of a clone, a repository not cloned — you
never read or fetch: write it as an open question naming the one read
and what it would settle, put it first in the report, and finish what
you have. The owner may be able to procure it; that is his call.

## The question (the README's, restated once)

Path of Building's unique files label a unique's variants, many by game
version (`Variant: Pre 3.23.0`, `Variant: Current`). Over the unique
items the owner's store holds, how well do those variants match what
the item displays: how many items fit exactly one variant, several, or
none; how often does the fitting set hold both an old and a not-old
variant (the park's "fits the old and the current variant", which the
search would report as undecided, C93); and what does the owner's own
example, his Ashes of the Stars (S165), read as? Then, beyond unique
variants — the owner, 2026-09-18: "there are some helmet enchantments
that can no longer be created in-game. My standard tabs will have many
of these"; "Crucible is another, since crucible items can no longer be
created" — how much of the corpus carries lines outside the implicit
and explicit arrays, and what source, if any, do the inputs on this
machine hold for the same question about those lines and about a rare
item's mods? The last is a read, not a count.

## Definitions (fixed; state any you had to sharpen, never swap one)

- **Population.** Every item in the store inputs whose frame type is
  unique, deduplicated by GGG item id (the newer fetch wins). Relic or
  foil frame types, unidentified uniques and items of the `poe2` realm
  are counted apart and not matched. Name the frame type ids you
  included.
- **Entry.** One `[[ … ]]` block of a unique file: a name line, a base
  line, zero or more `Variant: <label>` lines, header lines
  (`Source:`, `League:`, `Requires …`, `Implicits: N`, …), then mod
  lines, each optionally prefixed `{variant:1,3}` (the 1-based variants
  it belongs to; no prefix is every variant) and other `{…}` tags. An
  entry with no `Variant:` line has one variant, labelled `only`.
- **Binding.** An item binds to the entries whose name equals its
  `name`. Report items binding to no entry, one, or several; when
  several, the variants of all of them are the candidates.
- **Fit.** A variant fits an item when the two agree both ways over
  the item's implicit and explicit lines: every such line the item
  displays matches a line of the variant — same text once numbers are
  replaced, every number inside that line's `(a-b)` range or equal to
  its fixed value — and every line of the variant is displayed by the
  item. Lines in any other mod array take no part in the fit and are
  measured on their own (below).
- **Other arrays.** Every mod array an item carries besides implicit
  and explicit — enchant, crafted, fractured, scourge, crucible and
  whatever else the census shows — over the whole corpus, every
  rarity, not the uniques alone: items and distinct line templates per
  array, by league and by item class where the inputs give one. No
  line is judged; whether a template can still be made is the sources
  read, never a column here.
- **Catalyst quality.** An item carries it when a property says so
  (the quality property's name or type names a modifier kind rather
  than plain quality; state the rule you used). It is a flag on the
  fit row and nothing is rescaled: the measurement is how many
  uniques carry it and how many `none` rows with a number outside
  every range do.
- **Old.** A variant is *old* when its label contains `Pre <version>`
  (1,336 bare `Pre <version>` labels, and compounds such as
  `Two Abyssal Sockets (Pre 3.21.0)`; 2,657 `Variant:` lines in all —
  a grep, to be replaced by your parse). Every other variant is
  *not-old*, `only` included. Labels that name an axis other than
  version (`Life`, `Fire Damage`, `Purity of Ice: Cold`) are censused
  by shape in one table.
- **Either.** The fitting set holds at least one old and at least one
  not-old variant.

## Inputs (by path; anything else only under the rule above)

- `../../../../PathOfBuilding/` from this directory — the owner's
  clone (`SURFACES.md`, the Path of Building row; access method
  `clone`), at `16de4b82d57f1c0de6eb40f37143c32d4da36a02`. Read
  `src/Data/Uniques/*.lua` and `src/Data/Uniques/Special/*.lua`
  (`Generated.lua` is code that builds entries rather than a list of
  them: say what you could and could not take from it). Every script
  refuses to run at another commit. The files end lines with CRLF:
  strip `\r` before matching a line end (the index's trap).
- `raw/spike-GERWARIC_7694-<date>.db` — a `sqlite3 .backup` of the
  spike's facts store, taken by the reviewer before launch. Account
  data: never committed, never quoted beyond an item's name, base, mod
  lines and tab name.
- `raw/cpp-userstore-GERWARIC_7694-<date>.db` — the same for the C++
  app's store, which holds Standard tabs the spike has not fetched
  (`../item-facts/README.md`, F1); the rule on account data is the
  same.
- `../item-facts/scripts/census.py` — how both stores are read and
  deduplicated by id. Import or copy from it; never edit it.
- `../item-facts/data/mod-templates.csv`, `field-census.csv` — which
  mod arrays and fields the owner's items carry (open the first with
  `newline="\n"`).
- For the sources read: the clones beside this repository that
  `SURFACES.md` registers — `PathOfBuilding/`, `poe1/` with its
  per-patch history, `repoe/` with its data history (commits in
  `../repoe/MANIFEST.md`), `awakened-poe-trade/` (commit in
  `../prior-art/MANIFEST.md`) — anywhere in them, by file read and by
  local `git log` / `git show`, never a pull.
- `../DIGEST.md` — only to cite an `S` id already stated (S165, S178).

Absent inputs: if a file named here is missing, holds no Ashes of the
Stars, or lacks a field you expected, say so in the README's open
questions and continue; that is a finding, not a stall.

## Outputs

- `MANIFEST.md` — one row per `raw/` file (date, source path, captured
  by, access method, bytes, sha256) and one per clone read (commit).
- `scripts/variants.py` → `data/pob-variants.csv` — one row per
  (file, entry, variant): name, base, label, old yes/no, label shape,
  line count; and the parse's own refusals (blocks it could not read)
  in `data/pob-parse-refusals.csv`. Prints entries, variants and
  refusals.
- `scripts/match.py` → `data/unique-fit.csv` — one row per unique
  item: a row key that is not the GGG id (a hash prefix), name, base,
  league, entries bound, variants considered, labels of the fitting
  set, outcome (`one`, `several`, `none`, `unbound`), either yes/no,
  catalyst quality yes/no, other arrays carried, and for `none` the
  first reason (a displayed line with no text match
  in any variant; a text match with a number outside every range; a
  variant line the item lacks) with the line. Prints the outcome
  counts, the either count, the `none` reasons by count, and the
  catalyst cross-count.
- `scripts/arrays.py` → `data/other-arrays.csv` — one row per (array,
  line template): items, by league, by rarity, an example line. Prints
  items and templates per array.
- Strict CSV, `lineterminator="\n"`, first line a header, no comment
  line; a scrub guard refuses an account name or a full item id in any
  `data/` file.
- `README.md` — the track's one document, under rule 1: the status
  line, a headline block of at most five bullets, then findings as
  tables: the numbers in one table (population and what was counted
  apart; bound / unbound; one / several / none; either; catalyst
  quality; each split by league); the `none` reasons; the label-shape
  census; the Ashes of the Stars row or rows, with the lines displayed
  and the fitting set; the other arrays, per array; the sources read,
  as one table with a row per kind — a rare's mods, helmet and other
  enchants, crucible, each further array the corpus shows — of
  candidate sources with what each holds (the lines that exist now,
  or also when a line existed or stopped), what it lacks, and the path
  and commit that show it — "none found" is an answer; open questions, each naming the one read or experiment
  that closes it; provenance rows pointing at the manifest. About
  10 KB as a guide, not a target; per-item detail is the CSV, never
  the README. Write it last, the headline last of all.

## Acceptance

`python3 search/legacy/scripts/variants.py`, `match.py` and
`arrays.py` regenerate every `data/` file byte-for-byte; the outcome counts sum to the population; every number
in the README is printed by a script; the Ashes of the Stars entry
parses to the two variants `Pre 3.23.0` and `Current`, and the README
states what the owner's item fits or that the store holds none; every
claim in the sources read points at a path in a clone at its commit; no
sentence proposes a design.

## The report (at most 300 words)

When done, report: first, any input you wanted that is not on this
machine, then any unlisted one you read; the outcome counts, the
either count and the catalyst cross-count; the Ashes of the Stars
reading; the sources read in one sentence per kind; what you left
out; the one cut you
would most want reversed. The reviewer restores from that line.
