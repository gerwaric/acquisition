# Charge: draft the migration package (the F6 gate's step 1)

Written 2026-08-15. This is a self-contained charge for a fresh
session: drafting the documents that deliver the spike's results out
of this branch, per Tom's SD-R8-F6 adjudication (`result-draft.md`
§9). Read `AGENTS.md` and `status.md` first as always; then this
file is the task spec.

**This step produces drafts for review, not deliveries.** Per the F6
gate, the drafts go through the SD-R8 re-close review and the final
external audit (`final-audit-charge.md`) before any PR opens. Do not
open PRs, do not push, do not merge. F6 completes at landing, later.

## Why the branches work out (read before objecting)

The spike branch forked *from* `redesign` (at `2463eeba`), so it
already carries `docs/redesign/`. The redesign-side package is
therefore authored as ordinary final commits **on this spike
branch**; the eventual PR `spike/rate-limit-core` → `redesign`
delivers everything at once. The `AGENTS.md` "no commits to `master`
or `redesign`" rule is not violated: nothing here commits *on* those
branches (the ground-truth diff goes on a new branch *off* master),
and Tom has explicitly overridden the charter's "never merged"
snapshot line for the delivery (§9, F6 record).

## Deliverable 1 — the topic doc (spike-branch commit)

`docs/redesign/topics/rate-limit-core.md`, the distillation future
readers consume instead of this branch. Distill from
`result-draft.md` — read it in full first. It must contain:

- The register question and the ratified definition of "single
  serialized gate" (result-draft §1).
- **Both verdicts, verbatim in meaning, with their full scope**: the
  unconditional Known-lane verdict (all four OAuth policies
  exercised, including the SD-R8-F5 character-policy lanes), the
  conditional `backend-item-request-limit` verdict with its stated
  assumption, **and the complete scope carriage — U1–U5 and the
  ratified plain-English O-series block from §1, carried whole, not
  summarized away.** SD-R8-F7 was scope-carriage omission; the final
  audit checks for its recurrence by name.
- The evidence basis, briefly: the two-authorities rule, the
  declared pinned + 4,096-case extended-contract runs (16 reports
  per case, every N23 endpoint required by the declaration), the
  registry totals, and where the full evidence lives (this branch,
  named).
- Claim lanes per the charter: measured / estimated / inferred /
  external (with URL and retrieval date). Every claim gets a lane.
- The reusable-artifact section: the mock (counter engine + delivery
  shim) plus the M-series as the acceptance suite any future
  limiter must pass, C++ client included — and the register's
  standing note that it can hoist to its own repository without
  surgery.
- A pointer to CN1–CN6 as transcribed to ground truth (deliverable
  5), and the branch name `spike/rate-limit-core` recorded (register
  rule: the result doc records the branch).
- The findings-register lesson worth exporting: closure reviews must
  re-derive the claims from the evidence, not only verify machinery
  (the SD-R7/SD-R8 audit history in §9 is the citation).

Hard rule: **every sentence must be derivable from
`result-draft.md`.** The final audit re-derives the topic doc claim
by claim; anything broader than the evidence is a finding. When in
doubt, quote the §1 verdict language rather than paraphrasing it
looser.

## Deliverable 2 — the register row (spike-branch commit)

In `docs/redesign/README.md`, flip the rate-limit-core row's result
from `—` to a one-line result naming both verdict lanes and linking
the topic doc. Keep the row's question text unchanged.

## Deliverable 3 — the §8 record (spike-branch commit)

Fill `result-draft.md` §8's placeholder with the dated
reusable-artifact record: what exists (mock, M-series, judge,
declaration machinery, obligations registry), where it lives, and
the topic doc as its consumer-facing description.

## Deliverable 4 — the AGENTS.md hoist note (spike-branch commit)

A dated note in `AGENTS.md` recording that the hoist was authorized
by Tom's F6 adjudication and executed via this package, so the
"no commits to master or redesign" line cannot mislead a future
reader into the same false conflict this round hit.

## Deliverable 5 — the ground-truth diff (branch off `master`)

On a new branch cut from `master` (suggested name:
`rate-limit-core-ground-truth`), transcribe CN1–CN6 from
`result-draft.md` §4 into `docs/design/network-ground-truth.md`:

- Follow that document's existing N-numbering, lane labels, and
  citation style exactly — read several existing entries first.
- Carry each candidate's source citation (URLs and retrieval dates
  live in the charter's entries; CN6 cites this branch's code and
  the superseded C++ design docs — preserve its tripwire and its
  note about outliving superseded premises, both in §4).
- Do **not** edit the `redesign` branch's copy of the file —
  master's is the citation authority; redesign inherits on sync.
- Mark each transcribed CN row in `result-draft.md` §4 as
  transcribed (dated), on the spike branch.

## Wrap-up

Update `status.md` (§5 step 1 done, pointing at the commits), add a
dated §9 changelog entry describing the package, and commit in
small, one-concern commits per the house style. Never contact
pathofexile.com or any live service. The next step after this one
is the SD-R8 re-close review, then `final-audit-charge.md`.
