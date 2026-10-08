# pseudo-stats — the brief for two reads before 9d-C's sitting (2026-10-08)

9d-C's expected answers (`README.md`, "The ranged family's expected
answers"; the brief that ran them is this file at `624506b2`) rest on
two things no capture was composed to ask, and each reaches past the
ranged family:

- **Where a line sits does not change whether a total counts it.** Every
  row of the shipped table admits every source and every flag. The site
  files a line under a domain — the word after `stat.` in its `hash` —
  which a private item shows as its array and flags.
- **On a weapon the unsuffixed `Adds # to # <Type> Damage` is the
  weapon's own wherever it sits** (J7).

This run measures what is already on this machine says of each, so that
the owner's sitting asks only what is left. It builds nothing, sits
nothing and composes no search. The reviewer — the session that
launched you — checks the acceptance sentence by hand and commits; your
report is what it restores from.

## Rules that bind you

- Read first: this track's `README.md` — the headline, "The ranged
  family on the copy", "The ranged family's expected answers", "The
  captures, and what the site said"; `tools/trade_rows.py` and
  `tools/trade-evidence.py`; `scripts/ranged-census.py` and
  `scripts/ranged-expected.py`, the docstring and the readers each
  script below imports, never whole.
- Write only inside `search/pseudo-stats/`: two scripts, two files
  under `data/`, one section of the README. No existing script or
  extract changes: import what you need. Never the index
  (`search/README.md`), another track, `tools/`, `CONTEXT.md`,
  `decisions/`, `SURFACES.md`, the crates, the README's status line.
- Never commit, push, fetch or spawn. No network (C79). Restore no file
  with git. Do not run `acq` or `cargo`; read nothing under `target/`.
  The copy is read-only, as `scripts/ranged-census.py` opens it.
- Do not read the Path of Building clone: another reader has it, and
  the two answers are compared afterwards.
- A number is printed by a script after its last edit, never recalled.
  A count of nothing is written as nothing found, with what was looked
  through: it is no evidence that a thing is not counted.
- Extracts: CSV with `lineterminator="\n"`, the first line naming the
  script and its inputs. The clones, the raw files and the copy are
  local and may be absent: say which was, and go on.

## The work

1. **The evidence by domain**, `scripts/evidence-domains.py` writing
   `data/evidence-domains.csv`. Over every reading
   `tools/trade-evidence.py` makes — an item under a pseudo its search
   named, 1,503 printed on 2026-10-07 — and every line of the item that
   a row of the pseudo's latest version names: the line's domain and
   array, beside the pseudo. Per domain, and per pseudo and domain:
   the lines, the items, the readings that agree. Then the test: with
   that domain's lines left out of the rows' sum, how many readings
   disagree. A domain is **shown counted** where leaving it out breaks
   a reading; **shown left out** where a reading agrees only with it
   left out; otherwise **not shown**, and say which of the two it is:
   no line of it captured, or its lines captured only where the sum
   reads the same either way. Say how a domain maps to what a private
   item carries, array and flags, from the captures themselves, and
   name each domain where the captures do not settle the mapping.
2. **The copy beside it.** For every template a row of the shipped
   table (`crates/acquisition-search/reference/totals-v5.toml`) or of
   the 21 ranged totals (`data/table-changes.csv`) names: the copy's
   live `pc` lines by array and flags, each group beside step 1's
   verdict for its domain. In the same file, marked apart. Print the
   lines and items in groups shown counted, shown left out, not shown.
3. **The twin from the export**, `scripts/ranged-twins.py` writing
   `data/ranged-twins.csv`. `ranged-census.py` joins the export to the
   family's lines on uniques. Make the same join for every live `pc`
   line of the family on the copy, of any rarity: the mods of the
   generation types the line's array and the item's class admit, whose
   text is the line's template and whose ranges hold its two numbers;
   of each, whether the stats it translates from are local. One row a
   line: the item's base, class, side and rarity, the array and flags,
   and what the export says — local, global, both, or no mod found —
   with the mods' ids. How a generation type is chosen for an array and
   a class is a reading of the export you state at the top of the
   script; where two readings are open, run both and print both. Print
   the lines by side, array, flags and what the export says, and say
   whether the unique rows are `data/ranged-uniques.csv`'s.
4. **The README**, written last from the scripts' output: one section,
   "Before the sitting: the domain, and the twin from the export",
   about 1.5 KB — the domains by verdict, counted; the copy's lines
   under each; the family's lines by what the export says, and every
   line where the export and the item's side differ; then what is left
   for a sitting, as questions: a pseudo, a side, a domain. Detail
   stays in `data/`. Nothing else in the README changes.

## Acceptance

Each script exits 0 and a second run leaves its file byte for byte the
same; step 1 prints the readings it made and says whether the count is
`tools/trade-evidence.py`'s; every domain the captures hold and every
array and flag the copy holds under a row's template has a verdict in
the README as a script printed it.

## The report

At most 300 words, to the reviewer: the domains by verdict; the copy's
lines under each; what the export says of the family's lines by side
and array, and every difference from the item's side; the readings of
the export you had to choose between; what is left for a sitting;
which local input was absent; what you left out and why; the one cut
you would most want reversed; and that the status line and the index
row are not set.
