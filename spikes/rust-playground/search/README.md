# Item search — the slice's working directory

Item search is the next slice (owner, 2026-09-11: "the real heart of
acquisition"). This directory is where its research accumulates, one
track per directory, and later where the design and the build point.
It is evidence and working material, not deliberation and not rulings:
a session that thinks out loud writes a numbered `brainstorming-notes/`
note that cites a track by path; a ruling lands in `decisions/`. Nothing
here is an authority for either.

**Bootstrap for the design session:** load this file, then every track
`README.md` in the order below. Reach `data/` and `raw/` by pointer only.

## Rules in force

The only place process lives. Tighten by adding a line, loosen by
deleting one; nothing else encodes it. (Set 2026-09-12.)

1. **A track is a directory; its `README.md` is its one document.** Its
   top is fixed: a status line (`Status: <state> — <date>`; a review
   verdict goes there verbatim, dated), then a headline block of at most
   five bullets. Below that is the track's business. Findings are
   tables; numbers sit in one table; every open question names the one
   experiment or read that closes it; provenance rows point at the
   manifest.
2. **This index points and never summarizes.** One row per track. A new
   track is a directory plus a row; a retired one is deleted and its row
   says so with the commit.
3. **Budget: each track README about 12 KB**, measured in the table
   below; a check is added only when a budget trips.
4. **Three bins, three fates.** `raw/` — downloads, page saves, database
   copies: gitignored, local only, each described by a row in the
   track's `MANIFEST.md` (date, source, captured by, access method,
   sha256); dies with the machine. `data/` — extracts, regenerable:
   committed, first line names the script and the input; survives the
   close only if the build cites it. `scripts/` — committed, disposable;
   die at the close unless promoted to `tools/`. Exception, owner's call
   2026-09-12: GGG's static-data responses (stats, items, filters,
   static) are committed under `data/` with dated filenames so captures
   diff across leagues; page saves and the JS bundle stay in `raw/` and
   their extraction is what is committed (terms exposure, size).
5. **Narrative has one home per kind.** A session's story is its commit
   message; deliberation is a numbered note; a track README carries
   findings and open questions, never a diary. Review findings are rows
   in a `Review` table at the bottom of the track README, split out only
   when the table outgrows a screen.

## Traps (every track; add one when it bites twice)

- A `cd` inside a compound shell command moves the working directory for every later call in the turn, and parallel calls then fail on relative paths: use absolute paths under `search/`.
- Minified bundles defeat grep's regex limits; slice them with a short Python script.
- Generated extracts are strict JSON (a header comment broke loaders) and CSV written with `lineterminator="\n"` (the default is CRLF and every regeneration diffs); a scrub guard asserts on value patterns such as `"account": {`, never on bare key names.
- Write a track README last, from the script outputs, and its headline block last of all; the budget gets tight on a rich track, so per-item detail goes to `data/`.
- engine-bench: a throwaway Rust crate is never a workspace member, or the quality gate builds and lints it; copy the facts database with sqlite's `.backup`, never `cp`, because it is under WAL.
- agent-seat is written after item-facts and repoe so its example queries use real field names.
- Files from the game and from Path of Building end lines with CRLF — the `Mod*.lua` tables, the clipboard text in a `fetch`'s `extended.text`: strip `\r` before matching a line end (bit the mod-table parser and the `Item Class:` check in one session).

At the close this directory is expected to shrink to a closed record in
the mold of `PRICING-SLICE.md`, with the full text cited at a commit.

## Tracks

| Track | Question | Status | README bytes |
| --- | --- | --- | --- |
| [`item-facts/`](item-facts/README.md) | What does an item look like as GGG gives it, and how big is the corpus? | first pass complete — 2026-09-13 | 10782 |
| [`cpp-search/`](cpp-search/README.md) | What does the C++ app's search do, filter by filter, and how does it stay instant? | first pass — 2026-09-13 | 13037 |
| [`trade-query/`](trade-query/README.md) | What is the trade site's query language, and which of it means something for a private stash? | first pass complete — 2026-09-13 | 12746 |
| [`engine-bench/`](engine-bench/README.md) | Is SQLite over a derived search schema instant at a few hundred thousand items? Measured. | not started | 1201 |
| [`agent-seat/`](agent-seat/README.md) | What does an agent need to search a corpus that size well? | not started | 1139 |
| [`repoe/`](repoe/README.md) | What does the game's own data (RePoE's export, with Path of Building as corroboration) give a stash search that the item JSON does not? | first pass complete — 2026-09-13 | 12355 |

Standing rulings and parked items the design will revisit, so it knows
what it overrides: C48 (raw SQL is not a surface), C34 (derivations),
C12 (two surfaces), C79 (governed surfaces; `SURFACES.md`), and in
`decisions/store.md` "Parked": search-at-scale (FTS, a search crate) and
the user-scoped annotations home whose trigger lists saved searches.
