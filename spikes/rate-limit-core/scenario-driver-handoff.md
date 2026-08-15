# Scenario-driver and safety-closure hand-off

Status: **open — round five closed (SD-R5, 2026-08-14): the
independent re-review validated all twenty-four repairs
(SD-R5-F2..F15, RE-1..RE-9) with no new findings; the slice stays
open on the residual work in `status.md` §5.**
Rounds one–four and their findings remain in
`result-draft.md` §9; the round-five dispositions are its 2026-08-14
repair entries, and the re-review closure is the entry after them.
F14–F16 are fixed in this packet; no run declares
`FullContract`, and no verdict slot was filled.

## 1. Silences taken

| Silence or boundary | Conservative reading | Next-call consequence |
|---|---|---|
| The driver contract does not assign a profile source to hard-coded OAuth scenario endpoints. | OAuth rows use `Known`; only the two explicitly legacy M8/M10 lanes use `Assumed`. | A new hard-coded OAuth row cannot accidentally obtain the more conservative legacy policy. This fixed SD-R5-F1: M2/M6 had silently run under `Assumed`, weakening their claimed binding evidence. |
| M1's zero-remaining-budget residue does not state how a first GET reaches a permit after boot. | Wait the independently declared 15 s period plus 5 s bucket padding from HEAD completion; never ask production scheduling code for the answer. | A first GET before that boundary fails the residue sweep, and each generated residue/phase branch must reach its assertion. |
| Public actor traffic cannot reach the C3/C4 trip thresholds while D5 is intact. | Use the established internal `SafetyCounters` fault-injection seam, but finish through the real probe/ordinary feed, latch, drain, and watch-publication methods. | The next response-feed deletion fails a focused test. D5 is not weakened to manufacture an impossible public trace: its cap is at most 4 dispatches/s and 240/minute, plus at most two already-held completions, versus 11/s and 500/minute. |
| X2's spike-scope structure pin does not prescribe a reflection mechanism. | Collapse probe and ordinary sends into one private actor method, pin the single call site from source, and add a compile-fail example for outside `Actor` access. | A second `Transport::send` path or public actor owner fails structurally; a future production HTTP integration still owes its own pin. |
| The canonical fixture is finite but §7.4 does not bound parser resources. | Bound input at 2 MiB, 32,768 JSON items, depth 16, and 4 KiB strings — recalibrated by SD-R5-F12 (the original 10,000-item cap sat below the committed 15,804-item VPN fixture), every bound pinned at n/n+1, and the byte cap enforced at the single `bounded_parse` seam (it bounds parser work; `include_str!` embedding is bounded by §4 review, not at runtime). | An oversized or excessively nested next fixture refuses before allocation/recursion can grow without limit, and the supplemental VPN fixture parses (its median is now test-grounded). |
| §7.4's fixed-dispatch every-phase replay changes hypothetical server feedback without letting the captured client schedule react. | Tom adjudicated this as a frozen-contract expectation error on 2026-08-14. Preserve B3, both fixtures, and the complete 20-band / 1,052-phase counterexample diagnostic; replace the gate with feedback-consistent calibration. | The superseded assertion remains a finding reproduction until the replacement gate is precisely specified and implemented. Closed-loop C1/M-series every-phase safety is unchanged. |

Existing phase semantics still apply: `phase_ms` is the upcoming
boundary, and φ=0/1 are the two boundary-distance extremes. Focused
transition tests use those two phases only; the canonical replay is
exhaustive over φ=0..59,999 because every configured 5 s/60 s bucket
divides the 60,000 ms cycle.

The residual sweep exposed no new specification silence. RE-2, RE-6,
and RE-7 are already specified by G5, N19, and M3 respectively; RE-1
and RE-9 correct evidence collection/classification without changing
the recorded §7.4 counterexample. Tom's later adjudication changes the
gate expectation, not those repairs or their evidence.

## 2. Seam map and invariant walk

- The public driver submits only through `GateHandle`. Mock
  observations and watch state feed independent scenario oracles and
  `conformance::judge`; no oracle calls production scheduling code.
- F14 is structural now: both M8 lanes call one helper that requires
  exactly two GETs, the OAuth report repeats the non-verdict guard,
  and the D5 check uses `conformance::D5_IN_FLIGHT_CAP`.
- F15 is structural now: HEAD pacing uses `MIN_SEND_SPACING_MS`; M2's
  G4 minimum is derived from the policy definition, queue depth, D5
  floor, N13 periods/buckets, and the canonical service delay.
- F16 fails closed structurally (hardened for SD-R5-F6): the oracle
  trait returns `Option<u64>` and the judge itself scores a missing
  eligibility entry as a G3 failure — the fail-closed branch lives in
  one place, is documented on the trait, and
  `g3_fails_closed_when_the_oracle_has_no_eligibility_entry` fails if
  it is lost. The per-implementation `u64::MAX` sentinel is gone.
- RE-1 makes replay collection structural: every phase and every
  overflowing window on its initiating reply is accumulated before one
  set comparison. A two-separated-band mutation completes the 60,000-
  phase sweep and reports both discrepancies.
- RE-2 makes the M6 fragment verdict the sole decider for its four wire
  facts. A deliberately false fact reaches the judge as
  `G5 failed: ["M6Shrink"]`; no duplicate raw assertion intercepts it.
- **No permanent wedge:** dropped dispatched tickets reconcile in a
  detached task; M5/M6 transition queues eventually drain; M8 sibling
  callers resume after the sole confirmation; fuse/C4 trips drain all
  queued callers and latch terminal state.
- **One send, one entry:** reservation identity remains core-owned;
  the actor now has one `start_transport` method and exactly one
  `Transport::send` call site for probe and ordinary requests.
- **Pessimism direction:** zero-budget residue waits for the full
  independent window; M5/M6 preserve stale/pre-announcement facts;
  dropped dispatched work reconciles instead of rolling back.
- **Single scheduling authority:** all wire sends still originate in
  actor dispatch after `try_reserve`; timing tests delay transport
  arrival but never manufacture a second permit source.
- **Entry-point invariant:** probe tests finish through
  `finish_probe`; ordinary and organic-429 tests finish through
  `finish_ordinary`. No test swaps response entry points.
- **Truthful notifications:** D4 cooldown and both C4 feed paths assert
  the watch channel's changed state; fuse publication asserts Halted
  only after the actor mutates its terminal latch.

## 3. Coverage confession

The registry is the coverage authority. After the residual sweep it
contains 123 clauses: 96 Full, 13 Partial (`s7-4-replay-gate` is the
failing §7.4 gate's machine-checked slot, minted per SD-R5-F11, and
`m1-g1-sweep` is conservatively Partial per RE-9), one accepted
Untested limitation, and 13
Excluded; `OPEN_UNTESTED` is empty and
`cargo test --locked --test obligations` verifies the structure.

New or strengthened evidence:

- F14–F16; M1 residues 0/1/9/10 at φ=0/1; M2 burst and sustained
  stalls with runtime-derived G4; G5 unauthorized-refusal teeth.
- Probe-429 actor seeding and first-GET confirmation; per-endpoint D4
  cooldown/re-entry and unaffected-policy flow; D4 watch publication.
- Organic-429 Retry-After wire capture and honoring; dropped
  dispatched-ticket reconciliation.
- M5 stale-window exposure, M6 pre-announcement exposure, and M8
  concurrent-original serialization at φ=0/1.
- C3 latch/drain/publication, both C4 response feeds, X1 trip
  composition, and X2 one-send-path structure.
- The `start_ordinary` trip branch — the one path holding a popped
  caller and a granted reservation in neither collection — resolves
  its caller and rolls its reservation back (SD-R5-F9;
  mutation-checked for both loss modes).
- The same `start_ordinary` regression, the missing-G3 fail-closed
  regression, and the D5 declaration-consistency regression are now
  registry-cited, so deleting any of the three fails the coverage
  authority (RE-4).
- Canonical 383-dispatch replay, 81 ms B12 median, and the 43/43
  saturation diagnostic.

Every scenario-driver and focused transition report remains
`ContractCoverage::Fragment` and explicitly fails
`verdict_eligible()`. Two-phase tests are boundary checks, not an
exhaustive property claim. Public actor tests cannot make fuse
thresholds reachable under intact D5; the internal trip tests are
deliberate fault-injection composition evidence.

Exact remaining ballot/closure items:

1. M1's generated-φ mock-side residue sweep, the exact RE-9 delta
   for `m1-g1-sweep`.
2. M9's forced 14/15 reservation-to-arrival race:
   `m9-recovery-survives-race`,
   `m9-race-exposure-attribution`, and the last arm of
   `b12-scripted-delay`.
3. M11a near-ceiling compliant sweep:
   `m11-compliant-never-trips`.
4. The feedback-consistent §7.4 replacement calibration gate; retain
   the exhaustive fixed-trace counterexample as a diagnostic.
5. A declared full-contract run for the seven fragment-scale clauses:
   `m6-g1-post-announcement`, `m6-queue-drains-new-pace`,
   `m7-no-client-violation`, `m8-no-follow-on-violation`,
   `g1-zero-client-violations`, `g2-ceilings-never-tripped`, and
   `g3-over-delay-bounded`.

The canonical replay is not green. The violating set is **1,052
phases in 20 disjoint bands** (φ=7,454–7,466 through 25,854–25,944;
initiating replies 110–119 and 125–134), every band initiating on
`stash-request-limit`'s sustained 30/300 s window at 31/30 —
SD-R5-F2's amendment of CR-R1-F1, whose "exactly φ=7,454..7,466" came
from the asserting gate's first-failure abort. The full band table is
`VIOLATING_BANDS` in `tests/capture_replay.rs`, pinned by the active
band-edge test and the ignored exhaustive enumeration. Band-one
arithmetic is unchanged: at φ=7,454, 25 hits from
367,466..385,944 ms round to bucket end 427,454 and remain active
until 727,454; six new hits reach 31 one millisecond earlier; at that
reply the server recorded `6:300:0`. The production `CounterModel`
and independent arithmetic agree. The phase-0 diagnostic still
matches all 43 recorded saturation components, including 15/15 and
30/30. The trace replays cleanly at 98.25% of phases and the mismatch
is confined to one rule shape, but it is a systematic
model-vs-recorded-server disagreement, not a narrow single-band
coincidence.

## 4. Judgment calls

- The canonical wired median (81 ms across 383 samples) replaces the
  50 ms placeholder. The supplemental VPN median remains 148 ms; it
  is evidence of condition sensitivity, not the default.
- The M2 minimum includes the 81 ms service delay because the runtime
  bound measures caller-observed completion, not transport handoff.
- M5/M6/M8 timing tests are separate focused integration targets so
  their forced interleavings remain legible; they strengthen clause
  evidence without pretending to be full-contract scenario runs.
- The actual C3/C4 feed methods are load-bearing even though the
  pre-threshold counter state is injected internally. This preserves
  the safety contract instead of weakening D5 for test reachability.
- The superseded ignored every-phase assertion is retained temporarily
  as a reproduction of the adjudicated expectation error. The active
  exact-boundary test and exhaustive enumeration keep the counterexample
  from disappearing while the replacement calibration gate is designed.
- The OAuth/Assumed profile correction is recorded as SD-R5-F1
  because it was an evidence-validity defect found during integration,
  not a silent cleanup.
- (Repair session, 2026-08-14:) the focused M5/M6/M8 transition lanes
  now run under the Known profile too (SD-R5-F4) — every asserted
  bound is profile-invariant because the shared 60 s sustained
  resolution governs each one, verified by rerun. The remaining
  Assumed-engined focused targets (`actor_safety`, `actor_shell`) are
  deliberately unchanged under Tom's 2026-08-14 profile-lane
  ratification: their bounds are profile-invariant, and generic focused
  tests may retain the shipped default only on that condition.
- (Repair session, 2026-08-14:) the supplemental VPN median (148 ms)
  is now test-grounded rather than prose-only, which is also what
  exposed the parser item-cap miscalibration (SD-R5-F12).
- (Residual sweep, 2026-08-14:) `m1-g1-sweep` is Partial. C1's
  generated-φ property is the core-side mirror and never judges a
  mock-side boot-residue run; the exact delta is a generated-φ
  mock-side residue sweep. This is conservative evidence accounting,
  not a contract or verdict change (RE-9).

## 5. Verification presented with this packet

Residual-sweep matrix, entirely offline: `cargo test --locked` — 160
passed / 2 ignored; `cargo test --locked --release` — 158 / 2;
`PROPTEST_CASES=4096 cargo test --locked` — 160 / 2; all-target
clippy with warnings denied, fmt check, diff check, obligations 6/6,
and sanitizer tests 4/4 clean. Mutation checks: two separated band
edits were reported together only after the full replay sweep, and a
false M6 wire fact reached G5/M6Shrink.

The two ignored replay tests are reported separately: the collect-first
exhaustive band enumeration passed all 60,000 phases in 6.76 s, while
the now-superseded open-loop assertion reproduced the finding at
φ=7,454, reply 110, sustained 31/30 with restriction 301 s. No command
in this slice contacts a live service; no report declares
`FullContract`, and no verdict slot was filled.
