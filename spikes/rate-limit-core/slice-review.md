# Slice review guide

Status: process doc, created 2026-08-09 from the audit post-mortem.
Companion to the "Slice hardening rules" in `AGENTS.md`: that
section binds the implementing session; this one is the reviewer's
side — how Tom reviews a slice so the first slices' failure shape
cannot recur. Lessons-learned first, procedure second, so the
procedure's reasoning survives the people who remember why.

## 1. Lessons learned (why each step exists)

The audited slices were individually well-built: everything the
frozen docs specified was implemented faithfully and the specified
tests were real. Every defect came from one of three blind spots:

1. **Doc silence + no-improvise agents = literal implementations of
   incomplete specs.** The unbounded phantom synthesis and the
   unprotected `Retry-After` refusal were faithful readings of docs
   that simply didn't cover the case. An agent told "the docs are
   sole authority, don't improvise" will not add an unrequested
   bound. → Review hunts *silences*, not mistakes.
2. **Slice seams have no owner.** The abandonment/confirmation wedge
   was C5's drop semantics (built early) colliding with episode
   state (built late); the empty-rules panic was a guard present in
   one slice's entry point and absent from a later slice's. Fresh
   sessions never hold two slices at once. → Review walks the
   *seams* and the invariants list, not the new code's happy path.
3. **Green is not evidence.** A 97%-vacuous property test is green.
   A mirror-oracle is green. The design got four review rounds; the
   code got zero until the audit. → Review interrogates what green
   *means*, and no slice ends at green.
4. **Evidence rows are a seam too** (added 2026-08-12 from the
   scenario-driver round-four review). Lesson 2 is about seams
   between *slices*; the same failure happens between *rows of the
   same evidence table*. Doc finding 11 was re-scoped to say M10's
   fuse false-positive assert "was untested" and could only be
   demonstrated by occupying the fuse's headroom in a saturation
   run. It was already tested: `scenarios.md` assigns that property
   to **C3** ("never trips on any floor-compliant trace"), not to
   M10, and C3's row in the same §3 table — 130 lines up, updated
   three hours earlier the same day — already cited the green
   property. Two evidence rows contradicted each other and neither
   author looked at the other. The reviewer then filed the same
   finding a second time, from a `tests/` grep that missed a test
   living in a `src/` unit module. → **Before writing "X is
   untested", find the row that *owns* X and read it.** Ownership
   is in `scenarios.md`, not in whichever row you happen to be
   editing.

## 2. The hand-off (what the implementing session must present)

Per AGENTS.md, a slice arrives for review with four things. Refuse
the review until all four are present — their absence is the
finding:

1. **Silences taken**: every case the docs did not specify, the
   reading chosen, and the consequence traced one step further
   ("we refuse — *and the next `try_reserve` does what?*").
2. **Seam map**: every piece of state owned by an earlier slice
   that this slice reads or mutates, and every invariant from the
   AGENTS.md cross-slice list it re-verified, with one line each on
   *how* it still holds.
3. **Coverage confession**: what the tests deliberately do not
   cover, and for each property test, why it cannot pass vacuously.
4. **Judgment calls**: any decision the session made that a
   different reasonable session might have made differently,
   surfaced explicitly (the stale-429-joins call is the model).

## 3. The review walk (spend attention where the risk is)

Budget roughly: 10% happy path, 90% everything else. In order:

1. **Read the hand-off before any code.** It tells you where to
   look; the session that just held the context knows.
2. **Silences.** For each one: is the chosen reading conservative
   *in consequence*, not just in disposition? (The `Retry-After`
   bug was refusal-shaped — conservative-looking — and left the
   engine immediately grantable.) Does the doc finding exist in
   `result-draft.md`?
3. **Seams.** For each earlier-slice state touched: play the
   cross-product mentally — what does *their* mechanism do to *its*
   new state? Abandonment × new state, expiry × new state,
   rollback × new state, halt × new state. The wedge lived in
   exactly such a cell.
4. **Tests as evidence.** For each new match arm or branch: name
   the test that fails if it's wrong. For each property test: what
   makes it non-vacuous, and does the oracle avoid production code?
   For each bound: is the boundary itself pinned (n and n+1)?
   Before recording that something is *untested*, look up which
   scenario or property `scenarios.md` makes responsible for it and
   read that row's evidence — including unit tests inside `src/`,
   which a `tests/` search will not find (lesson 4). A claim owned
   by C-series or X-series is not M-series' to re-prove.
5. **Invariants walk.** Read the session's six one-liners (§2 item
   2) skeptically; spot-check one with the code open.
6. **Grep-level red flags** (each caught a real defect or is the
   class that would have):
   - `expect`/`assert!`/`unreachable!` reachable from public API on
     shell-timed or wire data
   - sentinel values crossing an API boundary (`MAX`, `0`-means-
     special, empty-means-special)
   - a guard at one entry point whose twin entry point lacks it
     (ask: why isn't this in a constructor?)
   - any wire-derived number used without a stated bound
   - a flag computed differently on different paths
     (`state_changed` was three different rules)
   - tests asserting an implementation artifact rather than a doc
     contract (pinning is fine — but the comment must say "pinned
     as current behavior" and cite why)

## 4. What not to spend time on

Formatting, naming, idiom, and structure inside a slice — clippy,
fmt, and the AGENTS.md working-style rules govern those, and the
audit found essentially nothing there. Do not re-derive the
arithmetic the property tests already prove; review the *tests'*
credibility instead (step 4) and let them carry the proof.

## 5. Closing a review

A slice closes when: findings are fixed or explicitly accepted,
anything accepted-not-fixed is in the `result-draft.md` register
with Tom's name on the decision, and the changelog entry cites the
review. Decision provenance in commit bodies, as ever.

**The commit that closes a round does all three of these, or the
round is not closed.** Fixing the findings is only the first.

1. Flip the slice hand-off's status line to name the round that just
   closed. A hand-off that still says "awaiting review" after its
   review is a trap for the next session, which has no other way to
   know.
2. Add or update the dated `result-draft.md` changelog entry: the
   round's findings and where each was fixed, anything
   accepted-not-fixed, the re-run gate matrix, and what the slice
   leaves open.
3. Mark superseded any *earlier* doc a reader could mistake for
   current — especially the coverage confessions in previous
   hand-offs, which go stale the moment a later slice builds what
   they call unbuilt. Preserve the dated text and add a marker
   beside it; do not rewrite the record.

This is not hypothetical. On 2026-08-12 the actor slice's re-review
findings were fixed in `02b60f47`, but the hand-off status line and
the register entry were not touched; separately, `core-handoff.md`
still listed C3/C4/X1/X2 as unbuilt long after they shipped, and a
session read that stale paragraph as the plan for what to build
next. Both were repaired in `693f0fc7` and `463ec155`.

Note for sessions and reviewers that keep private notes or memory:
this repository is the only channel every participant shares. A
process rule, a decision, or a status that lives only in one agent's
memory is invisible to the others — including external-model
reviewers. If it must survive the session, it goes in a file.
