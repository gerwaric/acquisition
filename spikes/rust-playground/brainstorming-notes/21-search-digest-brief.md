# 21 — The search digest: brief for stage 1 of the synthesis (2026-09-13, amended 2026-09-16, twice)

**Written 2026-09-13, before the run; amended 2026-09-16 after a dry
run of the process (note 23, decision 10).** The synthesis of the
item-search research runs in stages (the plan the owner accepted
2026-09-13: digest, framing, two blind proposals, cross-review and
reconciliation, ruling, build plan); this is the brief for the first.
It is committed before each run, deleted when `search/DIGEST.md` is
accepted, and cited by hash. Runner: one subagent per track, in series,
in the index's order; the reviewer — a Fable session — checks and
merges each part before the next runs, so the process is judged on the
first track before the rest are spent. The first track is the pilot
("The run, in series" below).

## Why a digest

Ten track READMEs are about 120 KB and hold everything the design must
not forget, in the shape research leaves it: findings, tables, open
questions. A design session that reads them whole skims or drowns, and
a design shaped by what was read last is what this stage prevents. The
digest is the slice's evidence in the shape a design can cite — the
same move the ground-truth claims make for facts about GGG — and from
the day it is accepted it is the entry point: a session reaches a README
only to verify a claim.

Its value is as much the ids as the compression: stages 4 and 5 name
the claims that decide a disagreement, and a blind pair of proposers
starts from one page. So fidelity of pointers outranks prose, and the
digest need not be exhaustive: a proposal may cite a finding the digest
dropped, marked undigested, and the reconciliation adds it (the plan's
guardrails). The digest is a floor, never a cap.

## The claim

One line each, under a track heading, in this shape:

`S<n> · <kind> · <weight> · <the claim, one sentence> · <pointer>`

- `S<n>` — stable, never reused; the design cites it. Ids come in
  blocks of twenty by the index's order: item-facts S1–S20, cpp-search
  S21–S40, trade-query S41–S60, repoe S61–S80, item-filter S81–S100,
  prior-art S101–S120, store-as-built S121–S140, engine-bench
  S141–S160, owner-seat S161–S180, agent-seat S181–S200. Gaps stay;
  the reviewer never renumbers.
- `kind` — `item` (a fact about the item as GGG gives it), `ggg` (a fact
  about a GGG surface: the API, the site, the filter language), `tool`
  (a fact about the C++ app, RePoE, Path of Building, Awakened PoE
  Trade), `store` (a fact about the spike as built), `measure` (a
  number this slice measured), `owner` (the owner's words as the track
  recorded them, verbatim, quoted — an utterance, not a ruling: a
  registry decision is cited as `C<n>` and never re-digested),
  `requirement` (owner-seat's R-lines and agent-seat's, as written),
  `idiom` (one query, filter or call as a source's users write it,
  verbatim — evidence of what they already know, never a template),
  `limit` (an edge the search will not promise).
- `weight` — for `item`, `ggg` and `limit` claims: `main` if a search
  over every stash meets it; `edge` if a few items or one league do.
  Every other kind carries `—`. PoE2 is a realm, not an edge (the
  characters ruling): its claims take the weight they would have at
  PoE2's launch. The design treats `edge` as a limit unless a `main`
  claim depends on it.
- The claim is a fact or a quotation, never a recommendation; a number
  stays a number, with the corpus it was measured on; a quotation is
  verbatim.
- The pointer is the README finding (`item-facts F7`) or the data file
  and row that carries it. No pointer, no claim.

About ten claims per track, fifteen where the track earns it, about
3 KB as a guide (the pilot measured 150–200 bytes a claim; 2 KB cut
facts, not words); tracks are not equal, and the reviewer is held to
the whole digest's budget, not the track's. The track's headline block is the seed and is not enough: a claim
the design would need and the headline dropped goes in. One `idiom`
per source that has one.

## More sections

- **The acceptance set** — outside the per-track budget, about 3 KB:
  one line per owner-seat question and per agent-seat scenario,
  verbatim, with the seat's reading of the answer wanted and a pointer.
  The proposals write each out in their appendix (note 22); stage 6
  reads it as its acceptance tests. The reviewer writes it from the
  seats' data files when those tracks merge.
- **Limits register** — an index: every `limit` claim's id with what
  the search says when it meets it (an unknown line shown, never
  guessed — prior-art F4 — is the model). It is the evidence floor a
  design inherits and rides through every later stage unchanged; the
  design's own limits are its output (note 22, question 7).
- **Kill list** — one line per track: the finding ids the design may
  safely ignore, a one-word reason each (internals, sizing, moot,
  budget). A claim cut for budget goes here by finding id, so the
  loss is visible from the digest alone and reversible under the
  floor-not-cap guardrail; a runner's report is a transcript, not a
  record.
  Ignored is not deleted; the README keeps it, and a proposal may
  reach for it under the guardrail above.
- **The question index** — the reviewer's: one line per ranked
  framing question (note 22), the `S` ids that bear on it. A
  source-ordered digest made question-addressable without the runners
  seeing the questions.
- **Convergence and contradiction** — the reviewer's: a fact another
  track also carries gets its sibling's id appended (`≈ S<m>`), since
  independent agreement is the strongest evidence in the pile; two
  claims that disagree are listed on their own lines with why.

## Rules that bind the runner

- Read `search/README.md`, then the one track's README and, for a
  claim that needs it, the data file it points at. Nothing else: never
  `brainstorming-notes/`, another track, `CONTEXT.md`, `decisions/` or
  `DIGEST.md`; never commit, push, fetch or spawn. Nothing from memory.
- Write only the track's section, into `search/<track>/DIGEST.part.md`;
  never a README, the index or the digest. The reviewer merges the
  part and deletes it.
- The prompt's register is exacting, not aspirational, and says why:
  the design that follows can only be as brave as this digest is
  honest, so a softened claim, a rounded number or a fact chosen to
  fit a conclusion is a decision made for the designer.
- Budget for the whole digest: about 30 KB, the acceptance set
  included.

## The run, in series

1. **The pilot is item-facts, run twice — once on Opus, once on
   Fable** — and the reviewer diffs the two parts by what each missed,
   softened or recommended, and by pointer accuracy. The model for the
   other nine is chosen from that diff and recorded in note 23 with the
   diff's counts. (The owner, 2026-09-16, on the model: "I'm on the
   fence"; the pilot answers with a measurement instead of a prior.)
2. Then one track per run, in the index's order. After each, the
   reviewer merges and commits the part, with the digest's top reading
   `Status: partial — <tracks merged> — <date>` until acceptance, the
   commit citing the brief's current hash; a brief amendment between
   runs is committed before the next run.
3. Acceptance is the last merge, when every section below is complete
   and the reviewer's steps have run for every track.

## The reviewer, after each run

1. Check each claim by hopping to its pointer — the finding row, the
   data row — never by reading the README whole; a claim its pointer
   does not carry is fixed or cut, and the fix is noted in the commit.
2. Remove recommendations in disguise: a measurement's conclusion posing
   as a fact ("no engine is needed" is one; "the scan won eleven of
   twelve" is the claim behind it).
3. Mark convergences and contradictions against the parts already
   merged; extend the question index and the limits register; for a
   seat track, write its half of the acceptance set.
4. Delete the part; commit the track's directory and `DIGEST.md`,
   never the parent (the research-track trap).

At acceptance: re-point note 22's F- and R-citations to `S` ids — a
citation with no claim behind it becomes a verified claim or loses its
evidence, which is the one independent check the framing gets; edit
`search/README.md` to name the digest the entry point; add
`search/DIGEST.md` to `tools/docs-check.sh`'s decision-id scan, since
a stale `C` id there would otherwise pass; `git rm` this brief and
cite it by hash.

## Acceptance

A design session that has read only `search/DIGEST.md` and the framing
(note 22) can write a proposal that cites evidence for every decision
and names every limit it inherits, and a reviewer can check any claim
in one hop.
