# Scenario-driver and safety-closure hand-off

Status: **open — §7.4 replacement-gate packet presented for
independent review, 2026-08-15.** The current packet implements
`status.md` §5 item 4 — the feedback-consistent §7.4 replacement
calibration gate per the ratified `s7-4-replacement-gate.md` §3/§4
(code commit `fdacd206`) — and flags one spec-expectation deviation
(the weaken mutation's kill signature) and one recorded doc finding
(the precondition-5 stash-list bucket-cutoff divergence) for the
reviewer. Its additions are marked "(§7.4 gate packet, 2026-08-15)"
in each section below; its evidence entry is the 2026-08-15
implementation entry in `result-draft.md` §9. The implementing
session does not close its own round. No run declares
`FullContract`, and no verdict slot was filled. The slice stays
open on `status.md` §5 item 5 (the full-contract run) after this
packet's review.

Prior state of this file: the residual-items packet below (M1
residue sweep, Ballot G's M9 race, M11a near-ceiling sweep) closed
independent review 2026-08-14 (SD-R6 — no findings; closure entry
in `result-draft.md` §9). Rounds one–five and their findings also
remain in `result-draft.md` §9. Dated text from those packets is
preserved below, not rewritten.

## 1. Silences taken

| Silence or boundary | Conservative reading | Next-call consequence |
|---|---|---|
| The driver contract does not assign a profile source to hard-coded OAuth scenario endpoints. | OAuth rows use `Known`; only the two explicitly legacy M8/M10 lanes use `Assumed`. | A new hard-coded OAuth row cannot accidentally obtain the more conservative legacy policy. This fixed SD-R5-F1: M2/M6 had silently run under `Assumed`, weakening their claimed binding evidence. |
| M1's zero-remaining-budget residue does not state how a first GET reaches a permit after boot. | Wait the independently declared 15 s period plus 5 s bucket padding from HEAD completion; never ask production scheduling code for the answer. | A first GET before that boundary fails the residue sweep, and each generated residue/phase branch must reach its assertion. |
| Public actor traffic cannot reach the C3/C4 trip thresholds while D5 is intact. | Use the established internal `SafetyCounters` fault-injection seam, but finish through the real probe/ordinary feed, latch, drain, and watch-publication methods. | The next response-feed deletion fails a focused test. D5 is not weakened to manufacture an impossible public trace: its cap is at most 4 dispatches/s and 240/minute, plus at most two already-held completions, versus 11/s and 500/minute. |
| X2's spike-scope structure pin does not prescribe a reflection mechanism. | Collapse probe and ordinary sends into one private actor method, pin the single call site from source, and add a compile-fail example for outside `Actor` access. | A second `Transport::send` path or public actor owner fails structurally; a future production HTTP integration still owes its own pin. |
| The canonical fixture is finite but §7.4 does not bound parser resources. | Bound input at 2 MiB, 32,768 JSON items, depth 16, and 4 KiB strings — recalibrated by SD-R5-F12 (the original 10,000-item cap sat below the committed 15,804-item VPN fixture), every bound pinned at n/n+1, and the byte cap enforced at the single `bounded_parse` seam (it bounds parser work; `include_str!` embedding is bounded by §4 review, not at runtime). | An oversized or excessively nested next fixture refuses before allocation/recursion can grow without limit, and the supplemental VPN fixture parses (its median is now test-grounded). |
| §7.4's fixed-dispatch every-phase replay changes hypothetical server feedback without letting the captured client schedule react. | Tom adjudicated this as a frozen-contract expectation error on 2026-08-14. Preserve B3, both fixtures, and the complete 20-band / 1,052-phase counterexample diagnostic; replace the gate with feedback-consistent calibration. | The superseded assertion remains a finding reproduction until the replacement gate is precisely specified and implemented. Closed-loop C1/M-series every-phase safety is unchanged. |
| M1 says "sweep residue magnitude" but states no upper bound for the generated sweep. | Cap generated residue at 12 — above the burst limit of 10 (so over-limit state headers are exercised) and strictly below the sustained 30 (so the zero-budget wait is always the burst window's 20 s bound). | A future residue ≥ 30 case saturates the sustained window and needs the 120 s bound instead; the sweep's G1 claim is scoped to residues ≤ 12, stated in the registry note. The driver's pinned 0/1/9/10 boundary cases are unchanged. |
| M11a's "compliant client never trips" names no traffic source that can approach the ceilings: every N23 policy caps the wire far below the D5 floor rate. | Use B7's scriptable-synthetic-policy channel (1,000/10 s + 10,000/60 s) so the 250 ms floor is the binding constraint and the actor reaches its compliant maximum — the closest a correct client can get to layer 1. | The sweep's "never trips" evidence is bound to floor-paced traffic at the compliant maxima (4/s, 240/min, pinned exactly); a floor-*violating* client evading the ceilings remains B13's wire-shape assertion per B10's recorded caveat, not this sweep's claim. |

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

The residual-items packet added the two boundary rows above (M1's
residue bound, M11a's pressure source). M9 needed none: the scenario
and B12 fully specify the race construction, and the phantom is
injected 1 ms after the transport hand-off — provably after the
client committed, still inside the scripted
reservation-to-receipt window §2 names.

(§7.4 gate packet, 2026-08-15) — silences taken against the
ratified spec:

| Silence or boundary | Conservative reading | Next-call consequence |
|---|---|---|
| Spec §3 gives the interior stride (991 ms) and the sample census (24) but not the exact sampling rule. | Interiors at band.start + k·991 for k ≥ 1, strictly inside the band (φ < end); edges asserted separately; the 98-phase census is itself asserted. | The two narrow bands — (6,966–7,204) and (36,549–37,297) — get no interior point, matching the spec's "two narrow bands" statement; a future re-reading of the rule that changes the sample set fails the census assert instead of drifting silently. |
| Spec §3 item 3 asserts only violation-free + ≥1 spurious borderline at halo edges, but §1 says pessimism is asserted "at every quantified phase, at every component". | C2 and the 766-component anchor are asserted at halo edges too (measured to hold there; the conservative direction for a calibration pin). | A future model change that understates only inside the halo fails the gate rather than hiding behind the halo's expected C3 witnesses. Flagged as a reading, not spec text — trivially removable if the reviewer reads §3 item 3 as exhaustive. |
| Precondition 4 says "recounted, not a bare literal" without naming the counting path. | The 43 is summed from the loaded records by the same `saturation_components` helper the diagnostics use; the pin is the assert against 43. | A fixture or loader change that moves the recorded saturation census fails the precondition before any phase replays. |
| Precondition 6 says "one distinct limit string per policy" — string or parse? | Verbatim header strings (`limits_raw`, added to the loader), compared per policy across all 387 records. | A re-serialized but semantically equal limit header fails the precondition — stricter than parse equality, which is the conservative side for a stability pin. |
| The spec's active-gate description says "state the debug time when it lands" but sets no bound. | Measured and recorded (0.25 s debug, under 0.1 s release); no bound asserted in code. | None — the numbers live in the §9 entry and this packet; CI cost stays negligible. |

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
- The three residual-item tests follow the same sole-decider pattern:
  each carries a facts struct with its own falsifiability guard, and
  the facts reach `conformance::judge` as the scenario assertion.
  All three consume only public seams — `GateHandle`, the mock
  controller, and the judge — with independently restated contract
  arithmetic (D5 floor, N13 padding, N19 recovery bound, B10
  ceilings); no oracle reads the engine or actor constants.
- M9's race exposure exercises the public §2 attribution seam for the
  first time in an integration run: `ExposureAllowance` binds the
  raced reservation to the phantom `MockStateChange` (cap 1, the
  in-flight set at injection), the observable instant is
  independently scripted as the raced response's completion, and the
  identical evidence without the allowance is asserted to fail G1 —
  the allowance is load-bearing, not decorative.
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

(§7.4 gate packet, 2026-08-15) — seams this packet touches:

- **Mock model (mock slice's state):** the gate consumes only the
  public `CounterModel` loader/seeding/judgment path the existing
  replay tests already own — `seeded_model` is shared by the gate,
  the band-edge test, and the 43/43 pin, so all §7.4 deciders replay
  through one seam. No production (Rust-core) scheduling code
  appears in any oracle; per the ratified spec, feedback-consistency
  is defined against the *captured C++ client's* semantics, and
  nothing in the gate reads the Rust core at all.
- **`bucket_end` conventions:** the gate's band tables are sensitive
  to the model's half-open-bucket and exclusive-expiry readings,
  pinned at contract level by Tom's 2026-08-15 B3 amendment; the
  mutation-2/3 checks demonstrated the gate fails loudly in both
  convention directions.
- **Clause registry (registry slice's state):** `s7-4-replay-gate`
  flipped Partial→Full as the spec's discharge line directs — a
  deliberate coverage-state diff verified by the obligations suite;
  two stale `scenarios.md` anchors corrected in the touched clauses
  (`s7-4-replay-gate`, `b12-scripted-delay`); no new clause, no
  owner change.
- **Cross-slice invariants:** this packet adds test code and
  registry rows only; no engine, actor, or mock *code* changed, so
  the six AGENTS.md invariants are untouched by construction — in
  order: (1) no history/episode state is created or held by the
  gate (wedge-free trivially); (2) reservation identity is never
  exercised (no reservations exist in a replay); (3) pessimism
  direction is *asserted*, not mutated — C2 is the gate's own
  condition; (4) `try_reserve` authority is untouched (the replay
  drives recorded arrivals, grants nothing); (5) entry-point
  invariant untouched (no `on_response`/`on_probe_response` calls);
  (6) notifications untouched (no watch channel in the replay
  path).

## 3. Coverage confession

The registry is the coverage authority. After the §7.4 gate packet
it contains 123 clauses: 102 Full, 7 Partial, one accepted Untested
limitation, and 13 Excluded; `OPEN_UNTESTED` is empty and
`cargo test --locked --test obligations` verifies the structure. The
remaining Partial set is exactly the seven fragment-scale clauses
only the declared full-contract run can finish (`s7-4-replay-gate`
flipped to Full 2026-08-15 per the ratified spec's discharge line).

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
- (Residual-items packet, 2026-08-14:) the M1 generated-φ mock-side
  residue sweep — residue 0..=12 × φ over the 60,000 ms cycle with
  the three §3 rollover phases pinned, sustained-window residue count
  as the per-case non-vacuity anchor, zero-budget branch reachability
  asserted around the coarse advance, green at 4,096 generated
  cases; the forced M9 race at 14/15 — the mock's burst judgment
  pins 14 residue + 1 phantom + 1 client = 16 over 15, both race
  inequalities asserted, M8's recovery asserts carried, exposure
  attributed through the public seam with a no-allowance G1-failure
  teeth check; and the M11a near-ceiling sweep — 301 floor-paced
  dispatches peaking at exactly 4/20 per rolling second and
  240/1,000 per rolling minute under both bucket profiles, zero
  trips, G2 armed. Each was mutation-checked (broken residue anchor
  → G5; weakened zero-budget oracle → 19,875 ms G3 lateness;
  wrong-policy phantom → organic-429 assertion; shrunk synthetic
  limit → G5 and G3 together).

- (§7.4 gate packet, 2026-08-15:) the replacement calibration gate —
  spec-§3 integrity preconditions 1–9 asserted phase-independently
  (residue-zero migrated from the deleted superseded test); the
  pinned 28-band HALO table; Φ\* derived arithmetically as the
  complement of V ∪ HALO with the exact
  29,601 + 29,347 + 1,052 = 60,000 partition; C1/C2/C3 with the
  766-component and anti-echo anchors at 18 consistent edges + 24
  stride interiors; violation-free ≥1-spurious replays at all 56
  halo edges; and the ignored exhaustive companion re-deriving the
  full three-way classification and pinning the strict-overstatement
  envelope at 25..57 of 766 (green release run in §5's matrix).

**What this packet's tests deliberately do not cover:**

- Interior phases beyond the 24 stride samples are the companion's
  job, and the companion is `#[ignore]` — ordinary CI does not run
  it; the green release run recorded in §5 is review evidence, not a
  standing CI guarantee.
- C3 stays strict per Tom's ratified §6 item 2 decision: phases
  whose spurious borderline is masked at policy level (73% of edge
  witnesses) or pad-compatible (54%) are excluded from Φ\* even
  though their counterfactual schedule may have matched the
  recording — conservative for a calibration filter, and a real
  reduction in the anti-strictness half (non-emptiness plus pin
  discipline is all that remains, spec §1).
- The `sent_ms` arrival convention is retained with its
  stated-approximation argument; no sweep-scale alternative-
  convention measurement exists (spec §4 withdrew that claim), and
  this packet adds none.
- The gate is mock calibration against the observed lane — never
  client-safety evidence; closed-loop every-phase safety stays with
  C1 and the M-series.

Every scenario-driver and focused transition report remains
`ContractCoverage::Fragment` and explicitly fails
`verdict_eligible()`. Two-phase tests are boundary checks, not an
exhaustive property claim. Public actor tests cannot make fuse
thresholds reachable under intact D5; the internal trip tests are
deliberate fault-injection composition evidence.

Exact remaining ballot/closure items (items 1–3 of the round-five
residual set were discharged by the residual-items packet; item 1
below is discharged by the §7.4 gate packet, pending its review):

1. ~~The feedback-consistent §7.4 replacement calibration gate;
   retain the exhaustive fixed-trace counterexample as a
   diagnostic.~~ — **implemented 2026-08-15** (this packet; the
   band-edge test and enumeration are retained unchanged).
2. A declared full-contract run for the seven fragment-scale clauses:
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

*[Marker, 2026-08-15 (§7.4 gate packet): "not green" above describes
the superseded open-loop every-phase expectation, adjudicated an
error 2026-08-14 and whose test the adopted spec's §4 deleted. Under
the ratified replacement gate the calibration IS green: the 1,052
violating phases and the 29,347-phase borderline halo are the pinned
refuted sets, and the gate holds on the 29,601-phase derived
consistent set. Dated text preserved, not rewritten.]*

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
- (Residual-items packet, 2026-08-14:) that delta is now discharged
  and `m1-g1-sweep` is Full; C1's citation is retained as supporting
  mirror evidence only. Packet-specific judgment calls:
  - The M1 sweep crosses the 20 s zero-budget wait with 500 ms
    coarse steps bracketed by observation-count asserts (no dispatch
    can occur inside the coarse region, proven per case), keeping
    the 4,096-case run under one second without loosening G3's
    25 ms fine-step floor where lateness is actually measured.
  - The M9 phantom is injected 1 ms after the transport hand-off
    rather than between reservation and hand-off: it is then
    provably unobservable to the committed send, while still inside
    §2's reservation-to-receipt window; the exposure cap is 1, the
    in-flight set at injection time.
  - M11a pins its peaks *exactly* (4 and 240) rather than as upper
    bounds, so both a slower client (lost reachability) and a
    floor-violating one (excess pressure) fail the fact; the 5×
    ceiling-headroom ordering is a compile-time assertion. The
    sweep runs under both bucket profiles instead of arguing
    profile invariance for a generic synthetic policy.
- (§7.4 gate packet, 2026-08-15:)
  - **The weaken-mutation deviation, flagged for the reviewer.** The
    spec's §3 predicts "expire hits one bucket early → C2
    understatement failure at a consistent phase". Measured: under
    that exact weakening (dropping `bucket_end`'s +1 bucket) *zero*
    C2 understatements occur at any of the 60,000 phases — the
    weakened model collapses toward a recording echo, so the
    mutation is killed by the **anti-echo anchor at φ=0** and, in
    the companion, by 30,399 misclassified phases. The gate kills
    the mutation; the spec's predicted signature does not occur on
    this fixture. Recorded rather than reordering asserts to force
    the predicted arm; the deviation is also in the §9 entry. A
    different reasonable session might have called this a spec
    erratum needing Tom — this one treats it as a measurement the
    review round adjudicates, since the mutation's *kill* (the
    contractual point) is intact.
  - Asserting C2 and the 766-component anchor at halo edges (spec
    §3 item 3 names only violation-free + ≥1 spurious) — the §1
    "every quantified phase" reading; trivially removable if the
    reviewer reads item 3 as exhaustive.
  - Interior sampling rule and verbatim `limits_raw` comparison as
    stated in §1's silence rows.
  - The corrupt-state mutation (spec: "corrupt one recorded state in
    memory after load") was instantiated as `records[50].states[1]
    += 1` — a mid-trace sustained-window bump chosen to land in the
    C2 arm rather than trivially in a precondition; the spec allows
    either ("precondition or C2/C3 failure").
  - Precondition 9 (layer-1) replays arrivals once through a φ=0
    model: `record_layer1_arrival` never reads φ, so one pass is the
    phase-independent assertion the spec asks for, and the per-phase
    replays keep their existing inline layer-1 assert as
    defense-in-depth.
  - `GATE_SAMPLED_PHASES = 98` is asserted even though it is
    derivable, so a future table or stride change that silently
    shrinks the sample set fails the census rather than passing
    thinner.

## 5. Verification presented with this packet

Residual-items matrix, entirely offline: `cargo test --locked` — 166
passed / 0 failed / 2 ignored; `cargo test --locked --release` —
164 / 0 / 2 (the two debug-only drop-bomb tests are absent);
`PROPTEST_CASES=4096 cargo test --locked` — 166 / 0 / 2 (37.6 s
total; the new M1 sweep contributes under one second at 4,096
generated cases); all-target clippy with warnings denied, fmt check,
`git diff --check`, obligations 6/6, and the Python sanitizer suite
4/4 clean. Mutation checks run and reverted for this packet: a broken
M1 residue anchor reached the judge as `G5 failed: ["M1BootSequence"]`;
a weakened M1 zero-budget oracle entry reached G3 as 19,875 ms
measured lateness (real slack 19 ms against ε=500 ms); an M9 phantom
injected on the wrong policy failed the organic-429 assertion; a
shrunk M11a synthetic burst limit failed on both axes (G5 peak loss
and G3 unmodeled policy waits).

The two ignored replay tests are reported separately, unchanged by
this packet: the collect-first exhaustive band enumeration passed all
60,000 phases in 6.78 s, while the superseded open-loop assertion
reproduced the adjudicated finding at φ=7,454, reply 110, sustained
31/30 with restriction 301 s. No command in this slice contacts a
live service; no report declares `FullContract`, and no verdict slot
was filled.

(§7.4 gate packet, 2026-08-15) — verification matrix, entirely
offline: `cargo test --locked` — 167 passed / 0 failed / 2 ignored;
`cargo test --locked --release` — 165 / 0 / 2 (the two debug-only
drop-bomb tests are absent); `PROPTEST_CASES=4096 cargo test
--locked` — 167 / 0 / 2; **both ignored replay tests green in
release** — the exhaustive band enumeration and the new exhaustive
classification companion (7.61 s alone; 7.7 s together), the
companion being the spec-mandated review evidence and the
re-derivation of every §5 number (tables, partition, 25..57
envelope); active gate 0.25 s debug / under 0.1 s release with its
98-phase census asserted; all-target clippy with warnings denied,
fmt check, `git diff --check`, obligations suite, and the Python
sanitizer suite 4/4 — all clean. Mutation checks run and reverted
(signatures verbatim in the 2026-08-15 §9 implementation entry):
HALO edge −1 → "the HALO table drifted" 29,348 ≠ 29,347; V edge −1 →
refuted-table overlap at φ=7,453; V edge +1 (inward) → V width sum
1,051 ≠ 1,052; weakened model → anti-echo anchor at φ=0 (the
recorded deviation above); strengthened model → C1 violation at
φ=0, reply 24, `stash-request-limit` burst 16/15; corrupted
recorded state → C2 at φ=0, reply 46, `character-request-limit`
sustained, model 13 vs recorded 14; echo mutation → anti-echo
anchor at φ=0. The superseded open-loop test is deleted by this
packet (spec §4; residue-zero assert migrated into the gate's
preconditions), so its finding-reproduction line above is now
historical. No command in this packet contacts a live service; no
report declares `FullContract`; no verdict slot was filled.
