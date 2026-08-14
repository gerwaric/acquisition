# Scenario-driver and safety-closure hand-off

Status: **open — implementation is review-ready for round five
(2026-08-14); awaiting independent review.** Do not close this slice
from the implementation session. Rounds one–four and their findings
remain in `result-draft.md` §9. F14–F16 are fixed in this packet; no
run declares `FullContract`, and no verdict slot was filled.

## 1. Silences taken

| Silence or boundary | Conservative reading | Next-call consequence |
|---|---|---|
| The driver contract does not assign a profile source to hard-coded OAuth scenario endpoints. | OAuth rows use `Known`; only the two explicitly legacy M8/M10 lanes use `Assumed`. | A new hard-coded OAuth row cannot accidentally obtain the more conservative legacy policy. This fixed SD-R5-F1: M2/M6 had silently run under `Assumed`, weakening their claimed binding evidence. |
| M1's zero-remaining-budget residue does not state how a first GET reaches a permit after boot. | Wait the independently declared 15 s period plus 5 s bucket padding from HEAD completion; never ask production scheduling code for the answer. | A first GET before that boundary fails the residue sweep, and each generated residue/phase branch must reach its assertion. |
| Public actor traffic cannot reach the C3/C4 trip thresholds while D5 is intact. | Use the established internal `SafetyCounters` fault-injection seam, but finish through the real probe/ordinary feed, latch, drain, and watch-publication methods. | The next response-feed deletion fails a focused test. D5 is not weakened to manufacture an impossible public trace: its cap is at most 4 dispatches/s and 240/minute, plus at most two already-held completions, versus 11/s and 500/minute. |
| X2's spike-scope structure pin does not prescribe a reflection mechanism. | Collapse probe and ordinary sends into one private actor method, pin the single call site from source, and add a compile-fail example for outside `Actor` access. | A second `Transport::send` path or public actor owner fails structurally; a future production HTTP integration still owes its own pin. |
| The canonical fixture is finite but §7.4 does not bound parser resources. | Bound input at 2 MiB, 10,000 JSON nodes, depth 16, and 4 KiB strings. | An oversized or excessively nested next fixture refuses before allocation/recursion can grow without limit. |
| §7.4 requires zero violations for every server phase but does not say what to do when the adversarial model exceeds the recorded behavior. | Keep the exhaustive gate, pin the exact counterexample independently, and open a Tom decision. Do not tune the model, fixture, or phase set. | The exhaustive test stays ignored with a known-finding reason; running it explicitly fails at the recorded boundary until the frozen-contract conflict is adjudicated. |

Existing phase semantics still apply: `phase_ms` is the upcoming
boundary, and φ=0/1 are the two boundary-distance extremes. Focused
transition tests use those two phases only; the canonical replay is
exhaustive over φ=0..59,999 because every configured 5 s/60 s bucket
divides the 60,000 ms cycle.

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
- F16 fails closed: a missing oracle key returns `u64::MAX`, so the
  next dispatch is ineligible rather than silently eligible.
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

The registry is the coverage authority. At hand-off it contains 122
clauses: 97 Full, 11 Partial, one accepted Untested limitation, and 13
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
- Canonical 383-dispatch replay, 81 ms B12 median, and the 43/43
  saturation diagnostic.

Every scenario-driver and focused transition report remains
`ContractCoverage::Fragment` and explicitly fails
`verdict_eligible()`. Two-phase tests are boundary checks, not an
exhaustive property claim. Public actor tests cannot make fuse
thresholds reachable under intact D5; the internal trip tests are
deliberate fault-injection composition evidence.

Exact remaining ballot/closure items:

1. M9's forced 14/15 reservation-to-arrival race:
   `m9-recovery-survives-race`,
   `m9-race-exposure-attribution`, and the last arm of
   `b12-scripted-delay`.
2. M11a near-ceiling compliant sweep:
   `m11-compliant-never-trips`.
3. Tom's §7.4 adjudication, then the exhaustive every-phase gate.
4. A declared full-contract run for the seven fragment-scale clauses:
   `m6-g1-post-announcement`, `m6-queue-drains-new-pace`,
   `m7-no-client-violation`, `m8-no-follow-on-violation`,
   `g1-zero-client-violations`, `g2-ceilings-never-tripped`, and
   `g3-over-delay-bounded`.

The canonical replay is not green. Its exact counterexample is
φ=7,454..7,466 inclusive, one sustained-window violation at counted
reply 110 (fixture record 114), `stash-request-limit`, 31/30. Phase
7,453 and 7,467 are safe. At φ=7,454, 25 hits from
367,466..385,944 ms round to bucket end 427,454 and remain active
until 727,454; six new hits reach 31 one millisecond earlier. The
production `CounterModel` and independent arithmetic agree. The
phase-0 diagnostic still matches all 43 recorded saturation
components, including 15/15 and 30/30. This is a narrow calibration
finding, not evidence that the entire trace is broadly noncompliant.

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
- The ignored exhaustive replay gate is retained as an executable
  statement of the frozen contract. The active exact-boundary test
  prevents its finding from disappearing from ordinary CI.
- The OAuth/Assumed profile correction is recorded as SD-R5-F1
  because it was an evidence-validity defect found during integration,
  not a silent cleanup.

## 5. Verification presented with this packet

The implementation session runs the proportional matrix before
commit and records exact commands/results in `result-draft.md` §9.
The canonical exhaustive ignored test is expected to fail and is
reported separately from the green ordinary matrix. No command in
this slice contacts a live service.
