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
Registry totals are 97 Full, 12 Partial, one accepted Untested
limitation (`x2-parser-cap-limitation`), and 13 Excluded. The
twelfth Partial is `s7-4-replay-gate`, the failing §7.4 gate's
machine-checked slot (SD-R5-F11); its delta is §3 decision 1's
adjudication plus the green rerun. Empty `OPEN_UNTESTED` does not
imply verdict readiness; the Partial set and the failed §7.4 replay
gate are itemized in §5. `obligation-map.md`
is the superseded prose ancestor; read its §8 for the audit's
discrepancy analysis (dated at `e2034807`), never for current
coverage.

## 2. Slice and review state

- **Open: scenario-driver slice** (M1–M13 driver/judge integration).
  The round-five independent review returned findings SD-R5-F2
  through SD-R5-F15; the 2026-08-14 repair session validated all
  fifteen (none invalid) and fixed them — dispositions,
  commits, and doc findings in `result-draft.md` §9's 2026-08-14
  repair entry. **The round is not closed**: the repaired packet
  awaits independent re-review, and no verdict slot was filled.
  `scenario-driver-handoff.md` remains the live four-part review
  packet, updated in place by the repair.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

1. **§7.4 adversarial-replay adjudication.** The canonical replay's
   required every-phase zero-violation gate fails for **1,052 of the
   60,000 phases, in 20 disjoint bands** (φ=7,454–7,466 first,
   φ=25,854–25,944 last; two clusters, initiating replies 110–119
   and 125–134, all on `stash-request-limit`'s sustained
   30-hits/300 s window at 31/30 under the 60 s adversarial
   bucket). The earlier "exactly φ=7,454–7,466" record (CR-R1-F1)
   came from a first-failure abort of the asserting gate and is
   amended by SD-R5-F2: the full enumeration is pinned as
   `VIOLATING_BANDS` in `tests/capture_replay.rs`, verified by the
   active band-edge test and the ignored exhaustive enumeration.
   This is a systematic B3-model-vs-recorded-server mismatch across
   1.75% of phase space and 20 initiating records — at counted
   reply 110 (fixture record 114) the server itself recorded
   `6:300:0` where the model computes 31 — not a knife-edge
   coincidence at one band. Band-one arithmetic is unchanged: 25
   earlier hits remain active until 727,454 ms and six new hits
   arrive one millisecond before expiry; phase 0 still matches all
   43 recorded saturation components. Decide whether this is a
   frozen-contract expectation error or requires a separately
   authorized model/fixture disposition. The implementation must
   not tune the gate away or amend the frozen scope silently.

2. **Profile-lane assignment ratification (non-blocking).** Which
   client bucket profile each test lane runs under is a doc silence
   the frozen docs support both ways (SD-R5-F3/F4). The repair
   session's conservative reading, now uniform in the driver and the
   focused transition lanes: OAuth-endpoint lanes run
   `Known(5s/60s)`, explicitly legacy lanes run `Assumed(60s/60s)`,
   and the shipped-Assumed default is structurally unlosable
   (`SweepPlan::new` on the driver's path). The remaining
   `actor_safety`/`actor_shell` focused targets still build Assumed
   engines over OAuth endpoints; their asserted bounds are
   profile-invariant (the shared 60 s sustained resolution governs
   every one), so they were left unchanged rather than churned ahead
   of your statement. Ratify or correct the assignment; recorded as
   a doc finding in the 2026-08-14 repair entry.

## 4. Blocked

The slice can undergo independent review, and the remaining test
work in §5 can proceed. §7.4 calibration closure and any final
verdict are blocked on decision 1. No live-service input is needed.

## 5. Next work

The 2026-08-13 ballot remains closed; this is the exact residual
set after the implementation swarm.

1. Independent re-review of the repaired round-five packet
   (`scenario-driver-handoff.md` plus the 2026-08-14 repair entry);
   do not flip its status or close the slice from the implementing
   or repairing session.
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
