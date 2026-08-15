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
Registry totals are 101 Full, 8 Partial, one accepted Untested
limitation (`x2-parser-cap-limitation`), and 13 Excluded. The
Partial set includes `s7-4-replay-gate`, §7.4 calibration's
machine-checked slot (SD-R5-F11), whose delta is a precisely
specified feedback-consistent replacement gate plus its green run;
Tom's adjudication is recorded in `result-draft.md` §9.
(`m1-g1-sweep` flipped to Full on 2026-08-14: the generated-φ
mock-side residue sweep landed — §9 changelog.)
Empty `OPEN_UNTESTED` does not
imply verdict readiness; the Partial set and §7.4 replacement gate
work are itemized in §5. `obligation-map.md`
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
  verdict slot was filled. The slice stays open on §5 items 4–5;
  `scenario-driver-handoff.md` remains the live four-part packet
  for that work.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

None. Tom ratified the §7.4 replacement-gate spec in full on
2026-08-15 — all five §6 asks, with the retrospective-P1 and
future-capture amendments folded in at ratification.
`s7-4-replacement-gate.md` is now contract; the adoption marker and
the B3 convention amendment are applied in `scenarios.md`; the
ratification record is in `result-draft.md` §9. (Both earlier
2026-08-14 sign-offs remain recorded there as well.)

- §7.4 is adjudicated as a frozen-contract expectation error: a
  feedback-dependent captured dispatch trace is not required to stay
  safe when replayed open-loop under counterfactual server phases.
  B3, both fixtures, and the exhaustive counterexample diagnostic stay
  unchanged. A feedback-consistent replacement calibration gate is
  open implementation work; every-phase safety remains in the
  closed-loop C1/M-series tests.
- Profile lanes are ratified: OAuth-bound scenario evidence uses
  `Known(5s/60s)`; explicit legacy evidence uses
  `Assumed(60s/60s)`; the shipped Assumed default remains
  structurally represented. Generic focused tests may use the shipped
  default only for demonstrably profile-invariant assertions.

## 4. Blocked

None. All remaining work in §5 can proceed. §7.4 calibration closure
still requires the replacement gate, but no Tom decision or
live-service input blocks it.

## 5. Next work

The 2026-08-13 ballot remains closed; this is the exact residual
set after the round-five close (SD-R5, 2026-08-14 re-review). Items
1–3 were implemented later on 2026-08-14 and their packet closed
independent review the same day (SD-R6, §2); items 4–5 are the live
work.

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
4. The feedback-consistent §7.4 replacement calibration gate.
   **Spec ratified 2026-08-15** (`s7-4-replacement-gate.md` — now
   contract; drafted with two adversarial fresh-context reviews,
   probe measurements, and a blind witness audit that hand-verified
   the first halo edge outside the shared machinery — §9
   changelog); next is
   implementation and its review round. The active band-edge,
   exhaustive band enumeration, and 43/43 witness diagnostics remain
   load-bearing; the superseded open-loop every-phase assertion
   remains only as a finding reproduction until the adopted
   replacement deletes it per the spec's §4.
5. Full-contract run last. It finishes the fragment-scale-only
   clauses `m6-g1-post-announcement`,
   `m6-queue-drains-new-pace`, `m7-no-client-violation`,
   `m8-no-follow-on-violation`, `g1-zero-client-violations`,
   `g2-ceilings-never-tripped`, and `g3-over-delay-bounded`.
   Verdict slots may be filled only when the run declares
   `FullContract` and the registry independently shows every owned
   clause `Full`; until then they remain blank.
