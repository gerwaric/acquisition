# 21 — The search digest: brief for stage 1 of the synthesis (2026-09-13)

**Written 2026-09-13, before the run.** The synthesis of the item-search
research runs in stages (the plan the owner accepted 2026-09-13: digest,
framing, two blind proposals, cross-review and reconciliation, ruling,
build plan); this is the brief for the first. It is edited by the owner
before it runs, deleted when `search/DIGEST.md` is accepted, and cited
by hash. Runner: one Opus subagent per track, then a Fable review of the
whole against the READMEs before the index names the digest as the
entry point.

## Why a digest

Ten track READMEs are about 120 KB and hold everything the design must
not forget, in the shape research leaves it: findings, tables, open
questions. A design session that reads them whole skims or drowns, and
a design shaped by what was read last is what this stage prevents. The
digest is the slice's evidence in the shape a design can cite — the
same move the ground-truth claims make for facts about GGG — and from
the day it is accepted it is the entry point: a session reaches a README
only to verify a claim.

## The claim

One line each, under a track heading, in this shape:

`S<n> · <kind> · <weight> · <the claim, one sentence> · <pointer>`

- `S<n>` — stable, never reused; the design cites it.
- `kind` — `item` (a fact about the item as GGG gives it), `ggg` (a fact
  about a GGG surface: the API, the site, the filter language), `tool`
  (a fact about the C++ app, RePoE, Path of Building, Awakened PoE
  Trade), `store` (a fact about the spike as built), `measure` (a
  number this slice measured), `ruling` (the owner's words, verbatim,
  quoted), `requirement` (owner-seat's R-lines and agent-seat's, as
  written), `limit` (an edge the search will not promise).
- `weight` — `main` if a search over every stash meets it; `edge` if a
  few items or one league do. The design treats `edge` as a limit
  unless a `main` claim depends on it.
- The claim is a fact or a ruling, never a recommendation; a number
  stays a number; a ruling is verbatim in quotes.
- The pointer is the README finding (`item-facts F7`) or the data file
  and row that carries it. No pointer, no claim.

Ten to fifteen claims per track. The track's headline block is the seed
and is not enough: a claim the design would need and the headline
dropped goes in.

## Two more sections

- **Limits register** — every `limit` claim gathered, with what the
  search says when it meets one (an unknown line shown, never guessed —
  prior-art F4 — is the model). This section rides through every later
  stage unchanged.
- **Kill list, per track** — findings the design may safely ignore, one
  line each with why: a detail of a tool's internals, a number that
  only sizes a data file, an open question whose answer changes nothing.
  Ignored is not deleted; the README keeps it.

## Rules that bind the run

- Read `search/README.md`, then the one track's README and, for a claim
  that needs it, the data file it points at. Nothing from memory.
- Write only `search/DIGEST.md`'s section for that track, into a
  scratch file the reviewer merges; never edit a README or the index.
- Budget for the whole digest: about 12 KB. If a track's section runs
  past 1.5 KB, the claims are too small or too many.
- The reviewer checks every claim against its pointer, removes any that
  is a recommendation in disguise, and only then edits `search/README.md`
  to name the digest as the entry point.

## Acceptance

A design session that has read only `search/DIGEST.md` and the framing
(note 22) can write a proposal that cites evidence for every decision
and names every limit it inherits, and a reviewer can check any claim
in one hop.
