# Kickoff: live/history doc split

**Do not start until the clause-registry slice has closed Tom's
review** — the split's live-status file points at the registry for
coverage truth; run early and you will write coverage prose the
registry immediately obsoletes. Written 2026-08-12 by the audit
session (`obligation-map.md`, `f03632b9`); motivating findings are
that report's §8.2 items 6–7 and the shape of §8.3 item 7.

This is a doc-only slice, but it changes what every future session
reads first, so it runs under the slice process: hand-off (silences
and judgment calls sections will be short; write them anyway), Tom's
review, register entry. No code changes of any kind.

## Read first, in this order

1. `AGENTS.md` — especially the hand-off chain table and the
   "repo is the only shared channel" rule you are about to serve.
2. `slice-review.md` §5 — the three-acts closure rule; your split
   must make act compliance *easier*, or it has failed.
3. `result-draft.md` §9, the round-four entry's "still open and owed"
   list (`result-draft.md:1273–1281` at `e2034807`) — commit 1 pays
   exactly this debt.
4. `obligation-map.md` §8.2 items 6–7 — the discrepancy classes this
   slice exists to end.

## The one rule over everything: relocate, never rewrite

Every dated paragraph survives verbatim where history lives. Moving
live state out of a document means adding a dated supersession
marker beside the old text and placing the live statement in its new
single home — the established `core-handoff.md` / `core-design.md`
precedent. If you find yourself editing a sentence inside a dated
entry, stop: that is commit 1's job (content corrections, each cited
to the owed list) or nobody's.

## Commit 1 — pay the round-four doc debt (content, not structure)

Separate commit, before any file moves, so Tom can review corrections
and relocations independently. Each item is already specified in the
owed list; line refs at `e2034807`:

- Finding 11's resolution text (`result-draft.md:242–263`) still
  reads as though scaling M10 discharged the fuse false-positive
  assert. Correct it to cite the composition the round-four entry
  establishes: C3 owns the property (`result-draft.md:111`), M10 owns
  the integration instance, X1 the true positive. Preserve the
  original wording under a dated marker.
- The §3 M10 row (`result-draft.md:94`) cites the composition the
  same way.
- The recorded M10 span, 3,963,250 ms → 3,963,500 ms
  (`result-draft.md:254` and `:1107`; measured value per the
  round-four entry).
- `AGENTS.md:60` — the scenario-driver hand-off row still says
  "rounds one and two"; bring it current, and note that review
  findings SD rounds recorded as F14–F16 exist and are unaddressed.
- `scenario-driver-handoff.md:3–5` — the status line names rounds
  one–three only; a reader of the hand-off chain cannot currently
  learn round four happened or that F14–F16 are open. Fix the status
  line; do **not** resolve F14–F16 themselves (driver twin-guard,
  duplicated floor literal, mirror fallbacks) — they are the driver
  slice's review debt, not yours.

## Commits 2+ — the split

**Create `status.md`** — the single live-state file, deliberately
small. Sections, in reading order:

1. **Coverage truth** — one paragraph: coverage is machine-checked;
   `src/obligations.rs` + `tests/obligations.rs`; how to run it; the
   open-untested list is `OPEN_UNTESTED`, not any prose table.
2. **Slice and review state** — which slice is open, which round,
   what that round still owes. (Today: scenario-driver open, round
   four, F14–F16 unaddressed — but write what is true when you run.)
3. **Open decisions (Tom)** — each with a one-line statement and its
   register pointer. Seed from: G3 ε / doc finding 12(c); what a
   spike-scope X2 test is (audit §8.2 item 2); an owner for the
   dropped dispatched-ticket lifecycle (§8.1 item 2); the §8.5
   ambiguity flags, M11a-vs-G2 ownership first among them; and the
   REG-R1-F4 deferral (`c4-halt-semantics-shared` reclassification,
   see the REG-R1 closure entry in `result-draft.md` §9). Also sweep
   that closure entry for anything newer this list predates.
4. **Blocked** — §7.4 capture replay, blocked on raw input from Tom;
   no synthetic stand-in permitted.
5. **Next work** — the current sequence, one line each.

**Re-point the authorities at it:**

- `AGENTS.md`: the hand-off table keeps the chain (who built what,
  which document) but its *status* column is replaced by "live state:
  `status.md`" — one authority, not two. The "what remains" paragraph
  (`AGENTS.md:39–47`) collapses to a pointer. The read-order list
  inserts `status.md` immediately after `AGENTS.md` itself.
- `slice-review.md` §5: closing a round updates `status.md` (this
  becomes the status-flip act's home) plus the register entry;
  hand-off status lines become historical markers on closure.
- Every hand-off whose slice has **closed** by the time you run — at
  minimum `core-`, `bootstrap-`, `mock-`, `actor-`, and
  `registry-handoff.md` — gets one line under its status line:
  "Historical record — live state lives in `status.md`." Dated text
  untouched. The **open** scenario-driver hand-off keeps its live
  confession until its slice closes; `status.md` §2 links to it and
  says exactly that, so there is never a moment with two live
  authorities and never a moment with none.
- `result-draft.md` stays the verdict skeleton + registers +
  changelog — history and evidence, no live status. Where its M-row
  "Result" cells carry coverage deltas the registry now owns, do
  **not** delete them; add one sentence to the §3 preamble: deltas
  in this table are historical as of the registry's landing date;
  live coverage is the registry. (Full row-by-row supersession
  markers would add 13 edits of noise for one fact — say it once.)
- The gate-summary rows (`result-draft.md:122–131`): add the
  partial-evidence marker the audit's §8.2 item 6 suggests —
  "unfilled pending full contracts; fragment-level gate evidence
  green at φ=0/1, see driver status note" — only if Tom confirms
  when reviewing this brief's slice; it edits register cells, so
  flag it in your hand-off if you defer it.

## Acceptance: the naive-reader probe

The property under test is "a fresh reader is not misled." Test the
reader, not a proxy. After commits land, run a cold context — a
fresh session or subagent that has seen none of your work — with
exactly this prompt:

> Read `spikes/rate-limit-core/` starting from `AGENTS.md` and
> following its stated read order. In under 200 words: what work is
> open, what is next, what is blocked, and which coverage claims are
> machine-checked versus open? Cite the file you got each answer
> from.

Pass iff: every answer matches `status.md`, and no superseded source
is cited as live. Include the probe's verbatim output in your
hand-off. If it fails, the fix is in your restructure, not in the
probe — iterate before presenting for review. One clean probe is the
gate; running a second with the prompt lightly varied is cheap
insurance against overfitting the read order to one phrasing.

## Out of scope

- Resolving F14–F16, raising any fragment, adding any test, touching
  `scenarios.md`, or filling any verdict or gate slot.
- Deleting or rewording any dated historical text (commit 1's
  enumerated corrections excepted, each with its marker).
- Restructuring `docs/` outside the spike directory.

## Definition of done

Commit 1's corrections each traceable to the owed list; `status.md`
live with the five sections; both authorities re-pointed in the same
commit that creates `status.md` (a split with a stale read-order is
the trap this slice exists to close); closed hand-offs marked; probe
transcript in the hand-off; full gate matrix re-run green (docs-only
changes still get the standard evidence); four-part hand-off; Tom's
review closes the slice.
