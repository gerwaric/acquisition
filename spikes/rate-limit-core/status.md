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
`src/obligations.rs` — `CLAUSES` (123 entries as of 2026-08-14, the
§7.4 gate clause minted per SD-R5-F11; lineage from
`obligation-map.md`'s 125 rows is recorded in `registry-handoff.md`
and the §9 changelog) — verified by `tests/obligations.rs`; run it
with

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
Registry totals are 109 Full, no Partial, one accepted Untested
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
declared run and registry must also agree. They now do, subject to
SD-R8 independent review (§2). `obligation-map.md`
is the superseded prose ancestor; read its §8 for the audit's
discrepancy analysis (dated at `e2034807`), never for current
coverage.

## 2. Slice and review state

- **Open: scenario-driver slice** (M1–M13 driver/judge integration).
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
  **Round eight is awaiting independent review** (SD-R8,
  2026-08-15): after Tom adjudicated F2, the harness-only G3 oracle
  now independently restates N13 padded-safe eligibility. The pinned
  φ=0 run and the full 4,096-case generated-phase run both declared
  `FullContract`; the latter passed in 298.84 s. The independent
  registry verifies 109 Full / 0 Partial / 1 accepted Untested / 13
  Excluded, including all seven former fragment-scale clauses. Both
  verdict lanes and G1–G6 are filled from the agreeing authorities.
  The updated four-part packet is `scenario-driver-handoff.md`; the
  implementing session does not close the round.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

None. Tom adjudicated SD-R8-F2 on 2026-08-15: **G3's eligibility is
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

None. SD-R8-F2's oracle-only fix landed and both the pinned and
4,096-case runs truthfully declared `FullContract`. The registry
independently agrees. SD-R8 now awaits review, not implementation.

## 5. Next work

The 2026-08-13 ballot remains closed; this is the exact residual
set after the round-five close (SD-R5, 2026-08-14 re-review). Items
1–3 were implemented later on 2026-08-14 and their packet closed
independent review the same day (SD-R6, §2); items 4–5 are now
implemented, with item 5 awaiting SD-R8 independent review.

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
5. ~~**Full-contract run last**~~ — **implemented and green
   2026-08-15; awaiting SD-R8 independent review.** The harness-only
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
   to Full, G1–G6 and both verdict lanes are filled, and the packet is
   presented in `scenario-driver-handoff.md`. The implementation
   session does not close SD-R8.
