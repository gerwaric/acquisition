# Terminal review charge: merged re-close + final audit

Status: **executed — terminal review passed 2026-08-15 with no blocking
finding and no accepted residual under Tom's recorded materiality
threshold** (`status.md` §2 / `result-draft.md` §9). Drafted
2026-08-15 by the analyst session under Tom's recorded terminal-gate
decision. This gate replaced
both the repeated re-close review and the repeated
`final-audit-charge.md` audit. You are a fresh no-context session by
design; the repository is the only channel. Read the mandated
documents in AGENTS.md order first; `status.md` is live authority.
Never contact a live service; commit before reverting any mutation;
leave the tree exactly as found.

## The materiality threshold (Tom's decision — binding)

A finding **blocks delivery** only if it:

- (a) forges or invalidates a **verdict authority** — the run-owned
  `FullContract` declaration chain or the registry's agreement with
  it; or
- (b) makes a **delivered claim false** — a statement in the filled
  verdicts, the consumer topic, or the migrated N27–N32 docs that
  the evidence record does not support.

Blocking findings: mint SD-R8-F26+, leave the round open, repairs
follow as before. **Everything below the bar** — wording
calibration, detection-layer scope, in-crate-test-only surfaces,
style — goes into **one accepted-residuals entry** in
`result-draft.md` §9 carrying Tom's name per his decision, and does
**not** reopen the round. Do not re-litigate named trust surfaces
or hunt further wording generations below the bar; the trust
surfaces are named by design, and Tom has priced the residual risk.

## Scope 1 — the unreviewed delta (F25 range)

The F25 repair (`3f777095`, packet `9221862e`) has not had its
independent review. Verify it against its §9 disposition (Tom's:
wording-only with single-sourcing; vector-pin strengthening
declined): the single-sourced belt-pin scope, the corrected claims
matching the scanned set exactly, and the review-committed mutation
pair evidence (`723578f4`/`0dd34a40`, `47885eab`/`351bfab6`).
Reproduce the pin's current refusal signature once.

## Scope 2 — the delivery object, whole

Per `final-audit-charge.md`'s method (kept; its unbounded standard
replaced by the threshold above):

- **The two authorities**: run the pinned declaration, the
  4,096-case declared run, and obligations yourself; confirm the
  declaration chain is registry-independent and the sealed evidence
  path holds at the crate boundary.
- **Every delivered claim**: the filled verdict paragraphs (both
  lanes, with the legacy condition and the O-series/U-series
  carriage), the consumer topic sentence by sentence against this
  record, and N27–N32 on `rate-limit-core-ground-truth` (head
  `3088d6e4`) against their cited sources.
- **The full offline matrix** once: debug/release/4,096-property
  suites, §7.4 ignored release pair, sanitizer, clippy, fmt, diff
  checks, both migration diff scopes.
- Spot-reproduce mutation signatures you doubt; all are recorded in
  §9 with exact messages.

## Closing

If nothing blocks: write the accepted-residuals entry (even if
empty — say so), close SD-R8 with the three acts of
`slice-review.md` §5, restore both verdict fills, close the slice,
and state plainly that the spike is delivery-ready per the F6 gate:
the two PRs (`spike/rate-limit-core` → `redesign`; ground-truth →
`master`) proceed on Tom's go, and the spike completes at landing.
If something blocks: record it and leave the round open — the
threshold bounds depth, never honesty.
