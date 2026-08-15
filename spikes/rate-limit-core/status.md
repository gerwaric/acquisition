# Live state: rate-limit-core spike

This is the **single live-state file** for the spike, created
2026-08-12 by the doc-split slice. Everything else in this directory
is contract (frozen design docs) or history (dated hand-offs, the
`result-draft.md` registers and changelog). If a statement here
disagrees with a status line elsewhere, this file wins; if this file
is stale, that is a process failure — closing any review round
updates it (`slice-review.md` §5). Keep it small: one live statement
per fact, pointers instead of copies.

## 1. Coverage truth

Coverage is **machine-checked**, not prose. The authority is
`src/obligations.rs` — `CLAUSES` (124 entries as of 2026-08-15:
the §7.4 gate clause minted per SD-R5-F11, and
`m2-character-policy-lanes` minted per the SD-R8-F5 extension;
lineage from `obligation-map.md`'s 125 rows is recorded in
`registry-handoff.md` and the §9 changelog) — verified by
`tests/obligations.rs`; run it with

```
cargo test --locked --test obligations
```

The checks are structural (unique ids, known owners,
citation/coverage arity, cited test fns exist, exact open-set
match); coverage *class* and `must_assert` accuracy are reviewed
prose, per the registry's coverage confession
(`registry-handoff.md` §3 — still the binding statement of those
limits, though that hand-off is otherwise historical).

The open-untested list is the `OPEN_UNTESTED` constant in
`src/obligations.rs` — **not** any prose table. As of 2026-08-14 it
is empty: the implementation swarm discharged all 13 previously
open Untested ids.
Registry totals are 110 Full, no Partial, one accepted Untested
limitation (`x2-parser-cap-limitation`), and 13 Excluded. The seven
former fragment-scale clauses §5 item 5 names flipped to Full on
2026-08-15 after the declared 4,096-case run and are cited to that
run in the registry. (`s7-4-replay-gate` flipped to Full earlier on
2026-08-15:
the replacement calibration gate landed green per the ratified
spec's discharge line — §9 changelog. `m1-g1-sweep` flipped to
Full on 2026-08-14: the generated-φ mock-side residue sweep
landed — §9 changelog.)
Empty `OPEN_UNTESTED` alone does not imply verdict readiness; the
declared run and registry must also agree. The 2026-08-15 external
audit (SD-R8-F4, `result-draft.md` §9) showed both authorities
could pass with M8's required Known lane missing; **the F4 repair
landed the same day** — the declaration guard is now keyed to M8
(`MissingM8KnownLane`/`MissingM8AssumedLane`, negative test pins
the audit's exact state) and both authorities reran green over the
repaired guard (§9). The registry verifier remains structural by
design; its limits are unchanged. The SD-R8 re-close review
suspended the agreement again on F9 — endpoint coverage was read
from a reproduction label the judge did not bind to the wire — and
**the F9 repair landed 2026-08-15** (`3813c40a`): the judge now
binds `reproduction.endpoint` to every observation alongside seed
and phase, the structural seam test pins the relabel, both
authorities reran green under the binding, and the review's own
end-to-end mutation is refused (`ReproductionMismatch { id: 1 }`).
The binding also caught and fixed two real M9 label errors. The
repeated independent re-close review reproduced both authorities and
all three refusal mutations, found no new findings, and restored
verdict readiness (§2, `result-draft.md` §9).
`obligation-map.md`
is the superseded prose ancestor; read its §8 for the audit's
discrepancy analysis (dated at `e2034807`), never for current
coverage.

## 2. Slice and review state

- **Closed: scenario-driver slice** (M1–M13 driver/judge
  integration; closed with SD-R8 on 2026-08-15, then **reopened
  the same day by the external no-context audit of the closure**
  — the round history follows).
  **Round five is closed** (SD-R5, 2026-08-14): an independent
  re-review validated all twenty-four repairs — SD-R5-F2..F15 and
  RE-1..RE-9 — against code, tests, and docs, reproduced the full
  verification matrix, and found no new findings; dispositions live
  in `result-draft.md` §9's 2026-08-14 repair entries and the
  re-review closure entry. No verdict slot was filled — every
  driver/focused report remains `ContractCoverage::Fragment`. The
  slice itself stays open on the residual work in §5;
  `scenario-driver-handoff.md` remains the live four-part packet for
  that work.
  **Round six is closed** (SD-R6, 2026-08-14): an independent
  review validated the residual-items packet — §5 items 1–3, the M1
  generated-φ residue sweep, Ballot G's forced M9 race, and M11a's
  near-ceiling sweep — against the contract docs, code, and
  registry; reproduced the full verification matrix and all four
  mutation checks (exact claimed signatures); and found no
  findings. The closure entry is in `result-draft.md` §9. No
  verdict slot was filled.
  **Round seven is closed** (SD-R7, 2026-08-15): an independent
  review validated the §7.4 replacement-gate implementation against
  the ratified spec clause by clause, reproduced the full
  verification matrix and all seven mutation signatures, confirmed
  the flagged weaken-mutation deviation and adjudicated it a spec
  erratum (corrected by a dated marker in the spec's §3), and
  accepted the precondition-5 doc finding as recorded.
  Reopened the same day by the external no-context audit (Tom's
  independence check on the spec-author/reviewer conflict; two
  findings), then **re-closed the same day on Tom's recorded
  adjudications**: SD-R7-F1 discharged (alternate kill signature
  accepted, erratum ratified, disclosed silences ratified) and
  SD-R7-F2 fixed and verified (C1 failures now carry full component
  witnesses including the recorded count). The audit's verdict and
  both closure entries are in `result-draft.md` §9. The slice stays
  open on §5 item 5 only — the full-contract run;
  `scenario-driver-handoff.md` remains the live packet file for it.
  **Round eight closed (SD-R8, 2026-08-15) and was reopened the
  same day.** An independent review validated the
  full-contract packet across its whole range (the declaration
  machinery, the SD-R8-F3 padded-history reconciliation fix in the
  core — walked against the pessimism invariant — the scale pins,
  and the post-adjudication padded-safe G3 oracle), independently
  re-ran both declared runs (the 4,096-case run's scale is pinned
  in code; all green), verified the registry as the agreeing second
  authority, and reproduced all four mutation signatures verbatim,
  including resolving the pinned-refusal determinism question
  (`ReportNotVerdictEligible { scenario: M2 }` exactly). No
  findings against the implementation. Closure entry in
  `result-draft.md` §9.
  **Reopened 2026-08-15 by the external no-context audit** (the
  closure entry's own recommended audit, per the SD-R7
  precedent): five findings, **SD-R8-F4..F8**, every one
  confirmed by the processing session against code and docs —
  including reproducing F4's experiment — before minting; the
  audit entry is in `result-draft.md` §9. F4 (high): the
  full-contract declaration accepts a missing M8 Known lane
  while the registry verifier (structural by design) also passes
  — the two authorities can agree in exactly the state the
  two-authorities rule calls impossible. F5 (high): the
  unconditional verdict's "four OAuth policies" claim exceeds
  the evidence — only the stash-list and stash policies are
  exercised; the character policies never are. F6 (high): the
  charter's end-of-spike deliverables (hoist to `redesign`,
  register row, CN1–CN6 transcription, reusable-artifact record)
  are neither done nor renounced. F7 (medium): the scope ledger
  omits U5 and carries no explicit O-series statement. F8 (low):
  this file's own §4 self-contradiction, discharged by this
  flip. The audit affirms as standing: the `cc448b79`
  padded-history fix, both declared runs, and the registry
  totals. **Both verdict fills are suspended** (dated marker in
  `result-draft.md` §1); the slice and the spike are open again.
  `scenario-driver-handoff.md` returns to being the live packet
  file. The padded-history production fix and Tom's F2
  adjudication are unaffected.
  **The 2026-08-15 SD-R8 re-close review did not close the round.**
  It reproduced the committed matrix and both advertised F4/F5
  refusal signatures, but found **SD-R8-F9** (high): the new endpoint
  declaration guard trusts `ReproductionRecord.endpoint` while the
  judge's reproduction seam validates only seed and phase. In the
  review mutation, the CharacterList wire lane was replaced by a
  second Character lane while the reproduction record still claimed
  CharacterList; both the pinned declaration and the structural
  registry verifier passed. The two authorities can therefore agree
  without one of the four claimed OAuth policies being exercised.
  It also found **SD-R8-F10** (medium): the live four-part hand-off was
  never updated for the reopened F4/F5 range and still declared the
  pre-audit close with stale 123/109 totals. Per `slice-review.md` §2,
  that missing packet is itself a finding. The spike-side migration
  package and the `rate-limit-core-ground-truth` diff had no additional
  re-close findings. The verdict fills are suspended again pending the
  F9 repair, authority reruns, a complete reopened-range hand-off, and
  re-review. Review entry in `result-draft.md` §9.
  **Repeated independent re-close review passed 2026-08-15 with no
  new findings; SD-R8 and the scenario-driver slice re-close.** The
  reviewer accepted the complete F4/F5/F9 reopened-range hand-off,
  verified the endpoint binding and both corrected M9 labels against
  the wire, reran the debug/release/4,096-property matrix, the pinned
  and explicit 4,096-case declarations, obligations, exhaustive
  replay, clippy, fmt, and sanitizer, and reproduced the exact F4/F5/F9
  refusal signatures. Both verdict fills are restored. The unchanged
  migration diffs retain the prior re-close review's no-finding result.
  Closure entry in `result-draft.md` §9; the hand-off is historical
  again. The spike remains open only on the F6 delivery gate.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

None. Every Tom-routed SD-R8 audit finding is dispositioned. Tom ratified
the F7 O-series carriage on 2026-08-15 (plain-English form, his
requested rewrite); it is part of both verdict statements in
`result-draft.md` §1. The repeated re-close review has **restored both
verdict slots** after independently validating F9/F10. The scoreboard: F4
repaired, F5 extended and landed, F6 gated-hoist recorded, F7
repaired and ratified, F8 discharged, F9 repaired and mutation-verified,
F10 repaired and review-accepted.

Decided 2026-08-15: **SD-R8-F6 — complete via gated hoist; the
spike ends with a PR** (Tom; full record in `result-draft.md`
§9). Gate: open items close with the migration package drafted
alongside → SD-R8 re-close review → final external no-context
adversarial audit over the re-close packet **plus both migration
diffs** → delivery via PR `spike/rate-limit-core` → `redesign`
(charter's "never merged" snapshot line explicitly overridden by
Tom) plus a small CN1–CN6 docs PR into `master`
(`network-ground-truth.md` there is the citation authority; the
charter's "on `redesign`" is amended). **F6 completes at
landing, not readiness.**

Decided 2026-08-15: **SD-R8-F5 — extend, don't narrow** (Tom).
Coverage extends to the two remaining Known-profile policies
(`character-list-request-limit`, `character-request-limit`) so the
four-policy claim is earned; the declaration guard must grow to
require the new lanes (F4 principle). Implementation is round
work (§5); the decision record is in `result-draft.md` §9.

Closed earlier the same day: Tom adjudicated SD-R8-F2 on
2026-08-15: **G3's eligibility is
the padded-safe time** — the harness restates N13's
`hit + period + bucket` arithmetic over B13 mock-side facts;
"client-independent" means independent in derivation, not free of
padding; ε = 500 ms unchanged. The clarifying amendment is applied
in `scenarios.md` §6 (dated, Tom-attributed) and the decision is
recorded in `result-draft.md` §9. Consequence: the SD-R8 blocker is
an oracle-side fix only — no client, mock, or gate-machinery change
— and that fix plus the declared run have now landed (§2/§5).

Earlier decisions remain closed. Tom ratified the §7.4 replacement-
gate spec in full on 2026-08-15 — all five §6 asks, with the
retrospective-P1 and future-capture amendments folded in at
ratification.
`s7-4-replacement-gate.md` is now contract; the adoption marker and
the B3 convention amendment are applied in `scenarios.md`; the
ratification record is in `result-draft.md` §9. (Both earlier
2026-08-14 sign-offs remain recorded there as well.)

- §7.4 is adjudicated as a frozen-contract expectation error: a
  feedback-dependent captured dispatch trace is not required to stay
  safe when replayed open-loop under counterfactual server phases.
  B3, both fixtures, and the exhaustive counterexample diagnostic stay
  unchanged. The feedback-consistent replacement calibration gate is
  implemented and SD-R7-closed; every-phase safety remains in the
  closed-loop C1/M-series tests.
- Profile lanes are ratified: OAuth-bound scenario evidence uses
  `Known(5s/60s)`; explicit legacy evidence uses
  `Assumed(60s/60s)`; the shipped Assumed default remains
  structurally represented. Generic focused tests may use the shipped
  default only for demonstrably profile-invariant assertions.

## 4. Blocked

Nothing is externally blocked. SD-R8 and the scenario-driver slice
are re-closed, and the verdicts are restored. The remaining path is:
final external adversarial audit
(`final-audit-charge.md`) → the two delivery PRs. Each step waits
only on the one before it.

## 5. Next work

The 2026-08-13 ballot remains closed; items 1–4 below stand done
as recorded (SD-R6; SD-R7 re-close). Item 5's SD-R8 closure was
**reopened by the 2026-08-15 external audit**; the reopened work
list follows the historical items. What follows the spike — Tom's
reading of the verdicts and the feed into `docs/adr/0003` — stays
outside this file's scope, and is farther off than the closure
claimed: the spike is not concluded.

Reopened SD-R8 work (audit and re-close-review entries,
`result-draft.md` §9):

- ~~**F4 repair**~~ — **done 2026-08-15**: declare's profile
  guard keyed to M8, negative test pins the audit's exact state
  (accepted before, `MissingM8KnownLane` now), `m8-both-lanes`
  cites it; end-to-end mutation reproduced the audit experiment
  against the repaired guard and it is refused. Both authorities
  reran green (§9 changelog).
- ~~**F7 U5 repair**~~ — **done 2026-08-15**: U5 carried in
  `result-draft.md` §7 alongside U1–U4, per its own carriage
  mandate. The O-series half stays with Tom.
- ~~**F5 extension**~~ — **done 2026-08-15** (`7a2d49e5`): both
  character-policy lanes run at every coverage level and both φ;
  the declaration requires every N23 endpoint
  (`MissingEndpointLane`, negative test + end-to-end mutation);
  hand-derived G4 fingerprints matched on first run; registry
  minted `m2-character-policy-lanes`; both authorities green
  over the extended contract (§9 changelog).
- ~~**F7 O-series carriage**~~ — **ratified by Tom 2026-08-15**;
  part of both verdict statements; verdict fills restored by the
  repeated re-close review (§3, §9 changelog).
- ~~**F9 endpoint-provenance repair**~~ — **done 2026-08-15**
  (`3813c40a`): the judge binds `reproduction.endpoint` to every
  wire observation alongside seed and phase; the structural seam
  test pins the relabel; both authorities reran green under the
  binding; the review's own end-to-end mutation (Character wire
  lane retaining the CharacterList label, seed 809) now fails
  `ReproductionMismatch { id: 1 }` before declaration. Two real
  M9 mislabels (driver row; focused race fixture) were caught by
  the binding and corrected — the seam was live.
- ~~**F10 reopened-range hand-off**~~ — **done 2026-08-15**: all
  four `scenario-driver-handoff.md` parts carry dated
  reopened-range additions (F4/F5/F9 silences, seam/invariant
  walk, coverage confession at the current 124/110 totals,
  judgment calls) plus the packet's verification matrix and three
  mutation signatures; the status header presents the packet for
  the repeated independent re-close review.
- **Remaining, per the F6 gate (delivery, not evidence):**
  1. ~~Draft the migration package~~ — **done 2026-08-15** per
     `migration-package-charge.md`: topic doc `f0bbb92d`, register
     row `1177fa56`, §8 artifact record `82d7a434`, and AGENTS F6
     note `88d8266c` on this branch; CN1–CN6 transcribed as N27–N32
     in `c1d92417` on `rate-limit-core-ground-truth`, cut from
     `master`. Drafts only; no push or PR occurred.
  2. ~~**Repeated independent SD-R8 re-close review**~~ — **closed
     2026-08-15 with no new findings**; F9/F10 validated, both
     authorities and all three mutations reproduced, verdict fills
     restored (`result-draft.md` §9).
  3. **Next: the final external adversarial audit** —
     `final-audit-charge.md` holds the standing charge; its
     object includes both migration diffs.
  4. The two delivery PRs (`spike/rate-limit-core` → `redesign`;
     CN docs → `master`). **F6 — and the spike — complete at
     landing.**

1. ~~**M1 generated-φ mock-side residue sweep**~~ — **done
   2026-08-14** (`tests/m1_residue_sweep.rs`; `m1-g1-sweep` Full —
   `result-draft.md` §9 changelog).
2. ~~**Ballot G** — the forced M9 phantom race at 14/15~~ — **done
   2026-08-14** (`transition_timing::m9_forced_phantom_race_at_saturation_recovers_per_m8`;
   `m9-recovery-survives-race`, `m9-race-exposure-attribution`, and
   `b12-scripted-delay` Full — `result-draft.md` §9 changelog).
3. ~~**M11a named binding evidence**~~ — **done 2026-08-14**
   (`tests/m11_ceiling_sweep.rs`; `m11-compliant-never-trips` Full —
   `result-draft.md` §9 changelog).
4. ~~The feedback-consistent §7.4 replacement calibration gate~~ —
   **done 2026-08-15**: spec ratified (`s7-4-replacement-gate.md`,
   contract), implemented green the same day (code `fdacd206`;
   `s7-4-replay-gate` Full; superseded open-loop test deleted after
   its residue-zero assert migrated), reviewed (SD-R7), reopened by
   the external audit, and **re-closed 2026-08-15 on Tom's recorded
   adjudications** (F1: alternate kill signature accepted and
   erratum ratified, silences ratified; F2: C1 failure messages
   fixed to carry the recorded count, verified). §9 changelog. The
   band-edge, exhaustive-enumeration, and 43/43 witness diagnostics
   remain load-bearing, retained unchanged.
5. **Full-contract run** — implemented and review-closed
   2026-08-15, then the closure was **reopened the same day by
   the external audit** (§2; reopened work list above). The
   historical record of the closure follows. The harness-only
   G3 correction implements Tom's F2 adjudication as independent
   `hit + period + bucket` arithmetic over B13 facts. The pinned φ=0
   declaration passed, followed by the full 4,096-case generated-
   phase run (298.84 s), drawing from the complete 60,000 ms strategy
   domain and covering both bucket profiles and full M6/M7/M8 queue
   shapes. It
   finishes the former fragment-scale-only
   clauses `m6-g1-post-announcement`,
   `m6-queue-drains-new-pace`, `m7-no-client-violation`,
   `m8-no-follow-on-violation`, `g1-zero-client-violations`,
   `g2-ceilings-never-tripped`, and `g3-over-delay-bounded`.
   The run declared `FullContract` and the registry
   independently verifies every owned clause Full; all seven flipped
   to Full, G1–G6 and both verdict lanes were filled, and the
   packet's review closed the round and the slice (§2, §9
   changelog) — the fills are now suspended per the audit
   reopening (§2).
