# 20 — The `Stats.dat` hash column: a fork of RePoE, its cost, and when it is worth it (2026-09-13)

**Written 2026-09-13 at `a95b339c`**, after the repoe track's first pass
(`search/repoe/README.md`, F2 and Q7; the mechanism is
`search/repoe/data/hash-recipe.md`). Deliberation, not a ruling: it is
cited from that README's Q8 and is history once Q8 is ruled. Disposable
like every note here (P1).

## The question

The export lacks the one column that would make the trade-id mapping an
algorithm. Could a fork of RePoE add it, and is that worth doing now,
later in the search design, or never? The owner, verbatim: "I would want
to prove the fix is durable through a game update before open an
upstream PR, but we could use it ourselves." and "I'm debating whether
it's worth the side quest to resolve this now or wait until later in
the search design process, or possibly not at all."

## Feasibility (read from the clones at the manifest's commits)

- The change is one line: `RePoE/parser/modules/stats.py` reads
  `IsLocal`, `IsWeaponLocal` and the two alias columns of `Stats.dat64`
  and never `Hash`; Path of Building's `Export/spec.lua` lists the
  column as an unsigned 32-bit integer, the value the recipe hashes.
- No game install: `run_parser.py` loads PyPoE's file system from GGG's
  patch CDN (`get_cdn_url`), and the upstream export runs entirely in
  GitHub Actions (`poe1/.github/workflows/export.yml`): a version poll
  every four hours against `ggpk.exposed`, then on a change a checkout
  of `repoe-fork/repoe-fork` and `repoe-fork/pypoe`, a schema import,
  `repoe all -l all`, a commit and a Pages deploy.
- The fork: the parser repository with the one line; the data
  repository with its workflow pointed at that parser fork, PyPoE left
  upstream, the schedule left on. The next patch re-exports by itself,
  and a check that runs the recipe from the export alone against the
  trade capture and Path of Building's tables is the durability proof —
  two exports across one patch with the recipe still exact. At the
  fork's cadence (README F6) that is two to four weeks of waiting after
  an afternoon of setup.
- Two things to settle first. Since 2026-08-03 the workflow imports a
  filtered schema (`repoe-fork.github.io/dat-export/poe/filtered.json`);
  if the filter drops unused columns `Hash` is absent, and the fallback
  is the unfiltered community schema, one workflow line. And the
  exporter fetches game data from GGG's patch server: a fork under the
  owner's account is a new access method on a surface `SURFACES.md`
  does not list — a row there before the first run (C79).

## What the column buys, and where

The core stash search never touches a trade id: a private line gives
text, the export turns text into a stat id (95 % of equipment lines,
F4), filters run on that. Trade ids enter only where a stash query
leaves for the site or a trade URL comes in (trade-query F9). On that
path one text can carry several numbers, and the site guarantees it
regardless of the export (380 collisions, trade-query F4), so the design
must carry "several ids for one line" as a normal case: a `count` group
with min 1 outward, an OR inward. The hash shrinks that set by about
110 stat ids of 7,000 (F2, Q7). Same branch, fewer cases; no component
added or removed.

## Second-order effects

Doing it now:

- presupposes Q8 — an investment in RePoE as a dependency before the
  design session rules whether the search depends on it, and sunk cost
  tilts that ruling;
- creates infrastructure held until upstream merges (a parser fork, a
  data fork, a scheduled fetch under the owner's account, a register
  row), and the merge is gated on a proof that takes a patch cycle;
- displaces engine-bench and agent-seat, which the design session needs
  and no later work substitutes for.

Waiting or never:

- the cost of waiting is latency, not shape — the fork is an afternoon
  and the proof one patch, and nothing built meanwhile changes because
  the multi-id branch exists either way;
- the cost of never is about 110 attributions living as OR groups and
  15 dormant ids sent needlessly (Q7); no user or agent would notice.

The one argument that survives: the hash is a join that cannot rot.
The text join breaks silently when GGG renames a line, and the 229
duplicate entries in the capture (F1) show renames happen; a hash join
is stable across them. That is a property of how the search follows
updates — the second half of Q8 as the owner framed it — so it is
decided inside that discussion, not ahead of it.

## Recommendation, as given 2026-09-13

Wait. Trigger: the design session rules that a line's identity is the
export's stat id or the trade id *and* the update-handling design wants
rename-proof joins. If the design keeps text templates as identity, the
C++ app's way, the column is never needed. The owner's decision is
recorded in the track README's Q8 when made.
