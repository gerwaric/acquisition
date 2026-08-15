# Charge: final external adversarial audit (the F6 gate's step 3)

Written 2026-08-15 by the SD-R8 audit-processing session; the gate
it implements is Tom's SD-R8-F6 adjudication (`result-draft.md`
§9). This is the standing charge for the spike's **last** external
no-context review — the one that stands between the re-closed spike
and its delivery PRs. Hand this file, alone, to a session with no
other context; everything it needs is in the repo, by design.

## Why this audit exists

Two prior external audits of this spike each found real defects the
internal closes had certified as clean (SD-R7 reopened once; SD-R8
reopened once — both records in `result-draft.md` §9). The recurring
failure shape: **the machinery was verified meticulously while the
claim was never re-derived from the evidence.** "No findings against
the implementation" and "the claim matches the evidence" are
different reviews. This audit runs both, over everything a future
reader will consume.

## Prerequisites — do not run this audit until all are true

1. The F7 O-series carriage wording is ratified by Tom and both
   verdict slots are refilled (`result-draft.md` §1; `status.md`).
2. The migration package is drafted: the distilled topic doc under
   `docs/redesign/topics/`, the register-row flip in
   `docs/redesign/README.md`, the §8 reusable-artifact record, the
   dated `AGENTS.md` hoist note — all as spike-branch commits — and
   the CN1–CN6 transcription diff on a branch off `master` against
   `docs/design/network-ground-truth.md`.
3. The SD-R8 re-close review (an independent review, distinct from
   this audit) has closed the round over the reopened-range packet.

## The object

- The reopened SD-R8 range: every commit from `f6e024dc` (the
  reopening) through the re-close head, including the F4 guard
  repair, the F5 character-policy extension (`7a2d49e5`), the F7
  repairs, and the refilled verdicts.
- **Both migration diffs** — the spike-branch package commits and
  the master ground-truth branch diff. The distillation is the
  artifact future readers consume; it must survive adversarial
  eyes itself, not merely descend from audited evidence.
- The two delivery claims: that the package faithfully carries the
  verdicts *with their full scope* (U1–U5 and O1–O8), and that
  nothing in it overclaims relative to `result-draft.md`.

## The charge

Adversarial posture: try to refute the close, not to confirm it.
Specifically:

1. **Re-run both declared authorities** from the committed tree:
   the pinned full-contract declaration, the ignored 4,096-case
   generated run, and `cargo test --locked --test obligations`.
   Confirm the declaration requires every N23 endpoint and both M8
   lanes (mutate, observe the refusal, revert — the §9 entries
   record the expected signatures).
2. **Re-derive every claim in the topic doc** from the evidence in
   `result-draft.md` — gate by gate, verdict by verdict, scope
   clause by scope clause. Any sentence a reader could take as
   broader than the evidence is a finding.
3. **Check the CN1–CN6 transcriptions against their cited
   sources** (the charter's entries carry the URLs and retrieval
   dates) and against `network-ground-truth.md`'s N-numbering and
   lane conventions.
4. **Check completeness of scope carriage**: U1–U5 and O1–O8 must
   appear in the consumed documents, not only in spike-internal
   files. SD-R8-F7 was exactly this omission; do not let it
   recur in the hoisted form.
5. **Check the register row and branch references**: the row must
   record the result and the branch name; the topic doc must point
   back to the spike tree it summarizes.
6. Reproduce the full verification matrix (debug, release, clippy
   `-D warnings`, fmt) and leave the tree exactly as found.

Hard rules: **never contact pathofexile.com or any live service**;
raw captures never leave the §4 sanitization contract; the tree is
left clean. Findings are minted in the SD-R8 namespace continuing
the existing numbering (F11 onward; F9/F10 were minted by the
2026-08-15 re-close review), recorded with severity,
evidence, and a disposition proposal, in the style of the §9 audit
entries. The verdict is one of: **the close and the package stand
(deliver)**, or **reopen** with the findings. Per the F6 gate, the
delivery PRs open only after this audit passes; F6 itself completes
at landing, not at readiness.
