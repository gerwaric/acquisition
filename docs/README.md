# Documentation Map

One line per document, plus the traceability rules that connect them.
Each document also states its own status and provenance in its header;
when this map and a header disagree, the header wins.

## Active

- `design/items-pipeline.md` — plan for the delta-native redesign of the
  items refresh pipeline. Milestone 1 shipped July 2026; M2/M3 get their
  own specs before their code begins.
- `design/network-redesign.md` — accepted, frozen spec for the
  rate-limited networking redesign (typed facade, coroutine pumps, gate).
  Records current decisions only; cites review finding IDs inline.
- `design/network-redesign-reviews.md` — that spec's decision history:
  review-round finding tables (ER, IR, R4–R7), round narratives,
  reversal records, and the revision log.
- `design/network-ground-truth.md` — living ledger of numbered claims
  (N1, N2, …) about how the Path of Exile API actually limits requests,
  each with cited evidence. Designs are derived from these claims.
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
- Retired documents (the cleanup plan and its phase documents, superseded
  spec text) live in git history, not in the tree.
