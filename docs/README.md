# Documentation Map

One line per document, plus the traceability rules that connect them.
Each document also states its own status and provenance in its header;
when this map and a header disagree, the header wins.

## Active

- `design/items-pipeline.md` — plan for the delta-native redesign of the
  items refresh pipeline. Milestone 1 shipped July 2026; M2 merged and
  M3 implemented July 30–31, 2026, each against its own frozen spec.
  The M1-M2 status-widget burst measurement ran July 31, 2026 (gate
  fired; coalesce built — `design/m1-m2-result.md`).
- `design/items-pipeline-m2.md` — **frozen** spec (revision 9,
  July 29, 2026, amended post-freeze the same day) for Milestone 2,
  the streaming refresh signal: D-numbered decisions consuming the
  plan's M2 inbox, with an input-traceability table. Review rounds
  1–7 are incorporated (round 4 was an in-repo audit); the S1-M2
  spike selected D9's outcome (a) with S = 60 s (evidence:
  `design/s1-m2-spike-result.md`). M2-M2 ran at its stage-4
  checkpoint (missed, remedied by the source-keyed stores, rerun
  passed — `design/m2-m2-result.md`); M1-M2 ran July 31, 2026
  (stuttered, remedied by the D10 dialog-side coalesce, rerun
  passed — `design/m1-m2-result.md`).
- `design/items-pipeline-m2-reviews.md` — that spec's review history:
  round-scoped finding tables (`R1-*`, …) with verdicts and
  resolutions, round narratives, and the revision log.
- `design/items-pipeline-m3.md` — **frozen** spec (revision 4,
  July 30, 2026; rounds 1–3's eighteen findings incorporated, review
  series closed) for Milestone 3, the delta-native items model:
  bucket-scoped model operations replacing the refresh-path
  reset, precomputed cached sort keys plus deferred bucket sorting
  (the July 30 lever hold point), the By-Item per-delta merge, and
  the F67 tie-break fix. Pre-spec evidence:
  `design/m3-sort-profile-result.md` (S1-M3 spike, run on the
  never-merged branch `spike/m3-sort-profile`, including the
  hold-point key-memory extension). The M1-M3 budget gate PASSED at
  S7 (July 31, 2026) — stage rows, the S5 miss and its A′ remedy,
  and the complete table: `design/m1-m3-result.md`.
- `design/items-pipeline-m3-reviews.md` — that spec's review history
  and revision log; rounds 1–3 recorded (R1-1…R1-8, R2-1…R2-6,
  R3-1…R3-4, all accepted) and the freeze.
- `design/items-pipeline-m3-implementation.md` — **implemented**
  (July 31, 2026) implementation sequence for the frozen M3 spec
  (externally reviewed July 30, round 1's four adjustments
  incorporated): stages S0–S8, conditional budget hold points at
  S3–S5, the formal M1-M3 gate at S7, and the M2-pin supersession
  map with its seam-reachability rule — verified executed exactly at
  S8. Sequencing only — no design authority; the spec wins on any
  conflict.
- `design/items-pipeline-m2-implementation.md` — **implemented**
  (July 29, 2026) implementation sequence for the frozen M2 spec
  (externally reviewed July 29, three findings incorporated): stage
  boundaries, gates (M2-M2 at stage 4), pin-to-stage traceability,
  and the branch's known intermediate states. Sequencing only — no
  design authority; the spec wins on any conflict.
- `design/network-redesign.md` — accepted, frozen spec for the
  rate-limited networking redesign (typed facade, coroutine pumps, gate),
  revision 11. Records current decisions only; cites review finding IDs inline.
- `design/network-redesign-reviews.md` — that spec's decision history:
  review-round finding tables (ER, IR, R4–R7, S1/S2), round narratives,
  reversal records, and the revision log, including the phase-5
  planning-readiness amendment.
- `design/network-redesign-phase5-verification.md` — phase-5 verification
  contract: the stable evidence-ID registry (`M-*`/`W-*`/`P-*`/`I-*`), worker
  preservation matrix, and the full-chain shutdown/retention harness. Retained
  as the permanent home of the evidence IDs the F56 resolved-ledger entry cites
  by name; the transient execution plan and per-session LSan handoff notes that
  accompanied it were removed after phase 5 landed (git history retains them).
- `design/network-ground-truth.md` — living ledger of numbered claims
  (N1, N2, …) about how the Path of Exile API actually limits requests,
  each with cited evidence. Designs are derived from these claims.
- `design/network-ggg-email-draft.md` — transient: the consolidated
  ask-GGG email (Q4 positional hypothesis, legacy bucket resolutions,
  retry-pad ceiling). Deleted once sent and the answers are transcribed
  into the ground-truth ledger.
- `cleanup/findings.md` — project-wide register of design/correctness
  findings (F1, F2, …): open findings, standing constraints, and a
  resolved ledger. The `cleanup/` path is historical — the register
  outlived the July 2026 cleanup it was created for, and stays put so
  its git history remains easy to browse.

## Historical

- `adr/0001-qml-ui-migration-strategy.md` — superseded: incremental QML
  migration proposal (its implementation plan remains on the unmerged
  `prepare-qml` branch).
- `adr/0002-defer-qml-migration.md` — accepted: defer the QML migration,
  do the interior design cleanup instead.

## Citation rules

- Problems get F-numbers in `cleanup/findings.md`, are never renumbered,
  and are cited by number everywhere else.
- Ground-truth claims get N-numbers in `network-ground-truth.md` and are
  never renumbered; designs cite claims rather than restating evidence.
  When a claim falls, every design that cites it falls with it.
- Spec review findings get round-scoped IDs (ER, IR, R4-\*, …) recorded
  in the review-history file; specs cite the IDs inline and record only
  current decisions.
- Phase-5 verification IDs (`M-*`, `W-*`, `P-*`, `I-*`) are local to the
  phase-5 verification contract, which is retained as their permanent home
  (the F56 ledger cites them). They organize completion evidence but are not
  permanent project-wide finding IDs.
- Retired documents (the cleanup plan, the phase-5 execution plan and its
  per-session handoff notes, the phase-0 QCoro spike `spikes/qcoro/` — retired
  July 2026 once the network redesign was no longer in flight; its S1/S2
  findings live in the review history — and superseded spec text) live in git
  history, not in the tree.
