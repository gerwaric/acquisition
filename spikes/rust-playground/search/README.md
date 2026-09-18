# Item search — the slice's working directory

Item search is the next slice (owner, 2026-09-11: "the real heart of
acquisition"). This directory is where its research accumulates, one
track per directory, and later where the design and the build point.
It is evidence and working material, not deliberation and not rulings:
a session that thinks out loud writes a numbered `brainstorming-notes/`
note that cites a track by path; a ruling lands in `decisions/`. Nothing
here is an authority for either.

**Bootstrap:** load this file, then `DIGEST.md` — accepted 2026-09-17, the
entry point in place of the track READMEs. Reach a README only to verify a
claim, and `data/` and `raw/` by pointer only.

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
- agent-seat is written after item-facts, repoe and owner-seat, so its example queries use real field names and are checked against a human's questions.
- A track run as a subagent (the research-track skill) never touches this index; the reviewer sets its row. And `git add search` while another subagent is still writing sweeps its files into the commit — add the track's directory, never the parent.
- A byte budget in a runner's prompt is filled to the byte, by every model, and the cuts vanish into the runner's report: give a guide, hold the reviewer to the whole, and put every budget cut on the kill list by finding id (both pilot runners, 2026-09-16).
- A commit message that states a file's size states it from memory: measure with `wc -c` after the last edit, before writing the message (four amended messages, 2026-09-16).
- Astra's committed output runs two to three times any guide (note 25 at 38.8 KB against 16; the stage-5 check at 29 KB under a brief that set none): a file past about 40 KB is read by section — `grep -n '^##'`, then `sed -n` by line range — never whole, and a brief gives a guide even for a check.
- Files from the game and from Path of Building end lines with CRLF — the `Mod*.lua` tables, the clipboard text in a `fetch`'s `extended.text`: strip `\r` before matching a line end (bit the mod-table parser and the `Item Class:` check in one session).

At the close this directory is expected to shrink to a closed record in
the mold of `PRICING-SLICE.md`, with the full text cited at a commit.

## Tracks

| Track | Question | Status | README bytes |
| --- | --- | --- | --- |
| [`item-facts/`](item-facts/README.md) | What does an item look like as GGG gives it, and how big is the corpus? | first pass complete — 2026-09-13 | 12597 |
| [`cpp-search/`](cpp-search/README.md) | What does the C++ app's search do, filter by filter, and how does it stay instant? | first pass complete — 2026-09-13 | 12992 |
| [`trade-query/`](trade-query/README.md) | What is the trade site's query language, and which of it means something for a private stash? | first pass complete — 2026-09-13 | 12826 |
| [`repoe/`](repoe/README.md) | What does the game's own data (RePoE's export, with Path of Building as corroboration) give a stash search that the item JSON does not? | first pass complete — 2026-09-13 | 12586 |
| [`item-filter/`](item-filter/README.md) | What predicates does GGG's own item-filter language name over an item, and which does a stash search inherit? | first pass complete — 2026-09-13 | 12492 |
| [`prior-art/`](prior-art/README.md) | How does a maintained item-to-trade tool (Awakened PoE Trade) give a line its identity and follow patches? | first pass complete — 2026-09-13 | 12111 |
| [`store-as-built/`](store-as-built/README.md) | What does the store's read surface give a search consumer today, and where would the first store change fall? | first pass complete — 2026-09-13 | 11318 |
| [`engine-bench/`](engine-bench/README.md) | Is SQLite over a derived search schema instant at a few hundred thousand items? Measured. | first pass complete — 2026-09-13 | 13056 |
| [`owner-seat/`](owner-seat/README.md) | What does the owner actually ask of the stash, in their words, and what answer shape does each want? | first pass complete — 2026-09-13 | 9169 |
| [`agent-seat/`](agent-seat/README.md) | What does an agent need to search a corpus that size well? | first pass complete — 2026-09-14 | 12317 |
| [`proposal-audit/`](proposal-audit/README.md) | Do the two blind search designs hold against their own page and their evidence? (synthesis stage 4; not research) | closed: two audit rounds, two repair passes — 2026-09-17 | — |

Standing rulings and parked items the design will revisit, so it knows
what it overrides: C48 (raw SQL is not a surface), C34 (derivations),
C12 (two surfaces), C79 (governed surfaces; `SURFACES.md`), and in
`decisions/store.md` "Parked": search-at-scale (FTS, a search crate) and
the user-scoped annotations home whose trigger lists saved searches.

The synthesis runs in stages after the tracks close (the plan and the
owner's decisions: `brainstorming-notes/23-search-synthesis-plan.md`):
its briefs and drafts are numbered notes, each edited by the owner
before it runs — the digest brief was note 21 (at c56404ca, deleted at
acceptance), the framing is `brainstorming-notes/22-search-framing.md`;
the digest is `DIGEST.md`, accepted 2026-09-17; the proposal brief
for stage 3 is `brainstorming-notes/29-search-proposal-brief.md`, and
the two blind proposals it produced are
`brainstorming-notes/24-search-proposal-fable.md` and
`brainstorming-notes/25-search-proposal-astra.md` (merged f2b06a3e). Stage
4 is an audit (`proposal-audit/`), the author's repair
(`brainstorming-notes/32-search-repair-brief.md`), the review, a
seat under `brainstorming-notes/30-search-review-brief.md`
(`brainstorming-notes/26-search-review-by-fable.md`,
`brainstorming-notes/27-search-review-by-astra.md`), and the reconciliation,
`brainstorming-notes/31-search-reconciliation.md`, which Astra checks. Stage 5 is the ruling packet, `brainstorming-notes/28-search-ruling-packet.md`, checked by Astra under `brainstorming-notes/33-search-ruling-check-brief.md` and harvested into `decisions/search.md`.

Two topics the design session rules without a track, since no read
now would add evidence (owner's call, 2026-09-13): how the search
follows updates — patches, leagues, the API, the site — with the dated
captures under rule 4 as the diff, and repoe Q8 as the entry; and PoE2,
whose delta the census saw (a class property, pervasive tag markup, 98
items) and whose trigger is the December launch.
