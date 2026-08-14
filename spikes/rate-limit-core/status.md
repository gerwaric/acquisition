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
`src/obligations.rs` — `CLAUSES` (122 entries as of 2026-08-13;
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
Registry totals are 97 Full, 11 Partial, one accepted Untested
limitation (`x2-parser-cap-limitation`), and 13 Excluded. Empty
`OPEN_UNTESTED` does not imply verdict readiness; the Partial set
and the failed §7.4 replay gate are itemized in §5. `obligation-map.md`
is the superseded prose ancestor; read its §8 for the audit's
discrepancy analysis (dated at `e2034807`), never for current
coverage.

## 2. Slice and review state

- **Open: scenario-driver slice** (M1–M13 driver/judge integration),
  **review-ready for round five**. F14–F16 are fixed: the twin M8
  arms share the full guard and non-verdict check, the D5/floor
  literals come from their authorities, and the oracle fallback is
  fail-closed. The 2026-08-14 implementation swarm also landed the
  ballot work listed in the prior status revision; it has not been
  independently reviewed and the slice is not closed. Its hand-off,
  `scenario-driver-handoff.md`, is the live four-part review packet.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

1. **§7.4 adversarial-replay adjudication.** The canonical replay's
   required every-phase zero-violation gate fails for exactly
   φ=7,454–7,466 ms: one `stash-request-limit` sustained-window
   overflow, 31/30, at counted reply 110 (fixture record 114),
   t=727,453 ms at the first phase. The B3 model and independent
   arithmetic agree: 25 earlier hits remain active until 727,454 ms
   and six new hits arrive one millisecond before expiry. Phase
   7,453 and 7,467 are safe, and phase 0 matches all 43 recorded
   saturation components. Decide whether this is a frozen-contract
   expectation error or requires a separately authorized model/
   fixture disposition. The implementation must not tune the gate
   away or amend the frozen scope silently.

## 4. Blocked

The slice can undergo independent review, and the remaining test
work in §5 can proceed. §7.4 calibration closure and any final
verdict are blocked on decision 1. No live-service input is needed.

## 5. Next work

The 2026-08-13 ballot remains closed; this is the exact residual
set after the implementation swarm.

1. Independent round-five review of `scenario-driver-handoff.md`;
   do not flip its status or close the slice in the implementation
   session.
2. **Ballot G** — build the forced M9 phantom race at 14/15. It
   discharges `m9-recovery-survives-race`,
   `m9-race-exposure-attribution`, and the last scripted arm of
   `b12-scripted-delay`.
3. **M11a named binding evidence** — near-ceiling compliant sweep
   for `m11-compliant-never-trips`.
4. Resolve §3 decision 1, then rerun the exhaustive canonical gate.
   The active exact-counterexample and 43/43 diagnostic tests remain
   load-bearing; the exhaustive gate remains ignored with its known-
   finding reason until adjudication.
5. Full-contract run last. It finishes the fragment-scale-only
   clauses `m6-g1-post-announcement`,
   `m6-queue-drains-new-pace`, `m7-no-client-violation`,
   `m8-no-follow-on-violation`, `g1-zero-client-violations`,
   `g2-ceilings-never-tripped`, and `g3-over-delay-bounded`.
   Verdict slots may be filled only when the run declares
   `FullContract` and the registry independently shows every owned
   clause `Full`; until then they remain blank.
