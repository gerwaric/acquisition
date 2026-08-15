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
Registry totals are 96 Full, 13 Partial, one accepted Untested
limitation (`x2-parser-cap-limitation`), and 13 Excluded. The
Partial set now includes `s7-4-replay-gate`, §7.4 calibration's
machine-checked slot (SD-R5-F11), and `m1-g1-sweep`, whose missing
delta is a generated-φ mock-side residue sweep (RE-9). The replay
slot's delta is a precisely specified feedback-consistent replacement
gate plus its green run; Tom's adjudication is recorded in
`result-draft.md` §9.
Empty `OPEN_UNTESTED` does not
imply verdict readiness; the Partial set and §7.4 replacement gate
work are itemized in §5. `obligation-map.md`
is the superseded prose ancestor; read its §8 for the audit's
discrepancy analysis (dated at `e2034807`), never for current
coverage.

## 2. Slice and review state

- **Open: scenario-driver slice** (M1–M13 driver/judge integration).
  The round-five independent review returned findings SD-R5-F2
  through SD-R5-F15; the 2026-08-14 repair session validated all
  fifteen (none invalid) and fixed them — dispositions,
  commits, and doc findings in `result-draft.md` §9's 2026-08-14
  repair entry. The residual RE-1..RE-9 re-review sweep was also
  validated and repaired on 2026-08-14; its dispositions and matrix
  are in the next §9 entry. **The round is not closed**: the repaired
  packet awaits independent re-review, and no verdict slot was filled.
  `scenario-driver-handoff.md` remains the live four-part review
  packet, updated in place by the repair.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

None. Tom approved both pending sign-offs on 2026-08-14; the exact
wording and consequences are recorded in `result-draft.md` §9.

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

None. The slice can undergo independent review and all remaining work
in §5 can proceed. §7.4 calibration closure still requires the
replacement gate, but no Tom decision or live-service input blocks it.

## 5. Next work

The 2026-08-13 ballot remains closed; this is the exact residual
set after the implementation swarm.

1. Independent re-review of the repaired round-five packet, including
   the RE-1..RE-9 residual sweep
   (`scenario-driver-handoff.md` plus the 2026-08-14 repair entry);
   do not flip its status or close the slice from the implementing
   or repairing session.
2. **M1 generated-φ mock-side residue sweep** — the exact Partial
   delta for `m1-g1-sweep` after RE-9 corrected C1's core-side mirror
   over-classification.
3. **Ballot G** — build the forced M9 phantom race at 14/15. It
   discharges `m9-recovery-survives-race`,
   `m9-race-exposure-attribution`, and the last scripted arm of
   `b12-scripted-delay`.
4. **M11a named binding evidence** — near-ceiling compliant sweep
   for `m11-compliant-never-trips`.
5. Specify, review, implement, and run the feedback-consistent §7.4
   replacement calibration gate authorized in §3. The active band-edge,
   exhaustive band enumeration, and 43/43 witness diagnostics remain
   load-bearing; the superseded open-loop every-phase assertion remains
   only as a finding reproduction until that replacement removes or
   recasts it deliberately.
6. Full-contract run last. It finishes the fragment-scale-only
   clauses `m6-g1-post-announcement`,
   `m6-queue-drains-new-pace`, `m7-no-client-violation`,
   `m8-no-follow-on-violation`, `g1-zero-client-violations`,
   `g2-ceilings-never-tripped`, and `g3-over-delay-bounded`.
   Verdict slots may be filled only when the run declares
   `FullContract` and the registry independently shows every owned
   clause `Full`; until then they remain blank.
