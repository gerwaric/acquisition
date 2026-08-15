# Scenario-driver and safety-closure hand-off

Status: **open — the reopened-range four-part packet (F4/F5/F9
repairs; dated 2026-08-15 additions in each section below, current
124/110 totals) is present and awaits the repeated independent
re-close review.** The F9 endpoint binding is repaired and
mutation-verified; this packet is the SD-R8-F10 repair.

Prior status: **open — the 2026-08-15 SD-R8 re-close review found
SD-R8-F9/F10; this file requires a reopened-range four-part update
before re-review.** F9 is the unbound endpoint-provenance seam:
`ReproductionRecord.endpoint` can disagree with the wire observations,
letting the pinned declaration and registry both pass after the
CharacterList wire lane is replaced by a duplicate Character lane.
F10 is this packet's own stale state: it still carried the pre-audit
close, 123/109 totals, and none of the F4/F5 additions in the four
required sections. Live disposition and the reproduced matrix are in
`status.md` and `result-draft.md` §9. The implementing/repair session
must update the silences, seam map/invariant walk, coverage confession,
and judgment calls; this review session does not manufacture its
missing hand-off for it.

Prior status: **closed — SD-R8 closed by independent review, 2026-08-15,
with no findings against the implementation; this closes the
scenario-driver slice itself.** Historical record — live state
lives in `status.md`. The reviewer independently re-ran the pinned
and full 4,096-case declared runs (scale pinned in code), verified
the registry's 109 Full / 0 Partial / 1 accepted Untested / 13
Excluded as the agreeing second authority, reproduced all four
mutation signatures verbatim, and walked the SD-R8-F3 core
reconciliation change against the pessimism invariant. Closure
entry in `result-draft.md` §9.

Prior status: **open — SD-R8 final full-contract packet presented
2026-08-15; the implementing session does not close it.** After Tom's
F2 adjudication, the harness-only G3 oracle independently restates
N13 padded-safe time. The pinned φ=0 run and all 4,096 generated
phases declared `FullContract`; the generated run completed green in
298.84 s. The independent registry verifies 109 Full / 0 Partial / 1
accepted Untested / 13 Excluded. G1–G6 and both verdict lanes are
filled from the agreeing authorities. This four-part packet awaits an
independent reviewer.

Prior status: **SD-R7 re-closed (2026-08-15) after the external
audit's reopening; the slice stays open on `status.md` §5 item 5
(the full-contract run) only.** Tom adjudicated both audit
findings the same day: F1's erratum ratified (alternate kill
signature accepted; disclosed silences ratified with it) and F2
fixed and verified — dispositions in `result-draft.md` §9. The
reopening record below is preserved as dated text.
The in-repo SD-R7 close validated the gate against the ratified
spec, reproduced the matrix and all seven mutation signatures, and
confirmed the weaken-mutation deviation — but an external
no-context audit found two findings: **SD-R7-F1** (high) — the
reviewer, who authored the spec, applied the weaken-mutation
erratum to ratified contract text without a recorded Tom
adjudication (AGENTS.md's flag-for-Tom rule; `slice-review.md` §5's
Tom-named-decision rule); and **SD-R7-F2** (low) — the C1 failure
path prints `CounterOverflow` (model vs limit) and omits the
recorded count §3's failure-message requirement names, despite
having it in hand. The audit independently re-verified the
measurements, tables, and matrix ("the gate's calibration result
remains credible") and its verdict is transcribed in
`result-draft.md` §9. The round re-closes on Tom's F1 adjudication
plus the F2 fix. The slice stays open on `status.md` §5 item 5
afterward. The reviewed packet implements
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
| The frozen docs do not specify the exact full-contract scale or what mechanically constitutes a declaration (SD-R8-F1). | Use 4,096 generated cases across all 60,000 phases with dedicated before/on/after boundary cases; require all M1–M13 rows, both provenance profiles, and exclusively verdict-eligible reports in a run-owned declaration that does not consult the registry. | A missing row/profile or one fragment/failing report refuses declaration; only after declaration may the independent registry authority be considered. |
| Reconciliation says to compare reported server counts with local in-window history, but does not say whether that local window is raw or N13-padded (SD-R8-F3). | Count local entries through each configured padded window. The server observation still supplies the lower bound, and synthesis remains capped by configured limits. | The next `try_reserve` sees retained N13-padded client hits once, not again as synthetic phantoms; no state understates the server, and entries still age out at the configured horizon. |
| “Client-independent” and “padded-safe” appeared to conflict in G3 (SD-R8-F2). | Tom's dated amendment resolves the silence: independently restate N13 as `hit + period + bucket` over B13 arrivals and scenario policy definitions; never read client state. ε remains 500 ms. | The next eligibility calculation includes the contractual bucket once; the pinned and generated declarations judge client work conservation without penalizing N13's safety margin. |

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

(SD-R8 reopened-range packet, 2026-08-15) — silences taken across
the F4/F5/F9 repairs:

| Silence or boundary | Conservative reading | Next-call consequence |
|---|---|---|
| The contract mandates M8 in both provenance lanes (`m8-both-lanes`) but did not say how a *declaration* proves that (SD-R8-F4). | Key `declare`'s lane checks to the M8 scenario itself — `MissingM8KnownLane` / `MissingM8AssumedLane` — never to whole-set profile presence, which M10's legacy row and any ordinary row satisfy vacuously. | A driver edit that drops M8's Known lane refuses declaration by name; the audit's exact experiment is the pinned negative case. |
| Tom's F5 amendment requires character-policy evidence but defines no character-specific scenario shape. | Reuse M2's contract semantics against the character endpoints as extra lanes — no new M-row, no new scenario id; queue 12 pinned above the character burst threshold and character-list's sustained limit; policy facts independently declared, never read from config at assert time. | Adding coverage needs no scenario-table change; dropping a lane refuses declaration (`MissingEndpointLane`); shrinking the queue below the pinned thresholds fails the scale test. |
| The declaration's endpoint requirement stated no proof obligation binding record labels to the wire (SD-R8-F9's root — the F4 repair introduced `ReproductionRecord.endpoint` as run-owned provenance, the same trust shape F4 itself condemned). | Endpoint is a wire-observable mock fact (`Observation.endpoint`); the judge binds `reproduction.endpoint` to **every** observation with exact equality, valid because every phase-swept lane is single-endpoint. | A relabeled or re-routed lane fails `judge` as `ReproductionMismatch` before any declaration; the two real M9 mislabels the binding immediately caught (driver row and focused race fixture, both labeled StashList over a Stash wire) are the demonstration that the seam was live, not theoretical. |

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

(SD-R8 full-contract packet, 2026-08-15) — seams and invariant walk:

- **Run declaration ↔ registry:** `FullContractRun::declare` checks
  only the run's reports (all M rows, both profiles, eligibility).
  It cannot read or mutate `src/obligations.rs`; the registry remains
  the genuinely independent second authority. In the final attempt,
  every generated case declares and the separately updated registry
  has no Partial clause, so the authorities agree without circularity.
- **Driver ↔ actor ↔ mock ↔ judge:** full-scale M6/M7/M8 stimuli use
  the same public `GateHandle`, in-process mock observations, and
  independent conformance oracles as the fragment driver. Queue and
  burst constants are structurally pinned, including two complete
  shrunk M6 windows. `try_reserve` remains the only permit source.
- **Reconciliation ↔ padded history:** the only production-state
  change is at the existing response reconciler. It counts the
  client's already-pessimistic history using configured resolutions;
  the wire-derived reported count is still capped before synthesis.
  The six invariants hold: (1) no new permanent state — entries age
  out at the padded horizon; (2) reservation ids and consumption are
  unchanged; (3) reported counts can add debt but never remove it,
  while own retained hits are no longer double-counted; (4)
  `try_reserve` remains sole scheduling authority; (5) ordinary and
  probe entry-point result types are unchanged and share only the
  reconciler; (6) eliminating spurious synthesis eliminates a
  spurious `StateChanged`, while real synthesis still emits it iff
  state mutates.
- **G3 contract seam:** only the harness oracle changed after Tom's
  adjudication. It consumes B13 arrival instants and mock-owned policy
  definitions, then performs its own `hit + period + bucket`
  arithmetic. It does not read actor/core state, call production
  scheduling code, or alter the mock. Client, mock, actor, and gate
  machinery diffs are zero in the continuation commit.

(SD-R8 reopened-range packet, 2026-08-15) — seams and invariant
walk for the F4/F5/F9 repairs:

- **Coverage chain is now mock-fact-anchored end to end:** the mock
  records the endpoint on every observation (unchanged mock code —
  the field predates this range) → the judge binds each reproduction
  record to those observations (F9, one added comparison) → the
  declaration requires every N23 endpoint and both M8 lanes from
  bound records (F4/F5). No link in the chain reads run-owned prose
  alone anymore; that was F9's defect.
- **Character lanes consume only public seams:** `GateHandle`, the
  mock controller, and `conformance::judge`, with the policy-generic
  `PolicyDebt` padded-safe oracle and the runtime-derived G4 minimum
  the main M2 row already used. Each lane pins its independently
  declared policy name against every wire observation (the vacuity
  guard against routing toward a looser policy).
- **Diff scope of the whole range:** conformance judge/declaration,
  driver tests, harness tests, and the registry. No engine, actor,
  or mock production code changed, so the six cross-slice invariants
  are untouched by construction — in order: (1) no new token or
  entry state exists anywhere in the range; (2) reservation identity
  is never exercised by declaration machinery; (3) pessimism
  direction is unchanged (no reconciliation or scheduling change);
  (4) `try_reserve` remains the sole scheduling authority (the new
  lanes submit through the same public gate); (5) entry-point
  invariant untouched (no `on_response`/`on_probe_response`
  change); (6) notifications untouched (no watch-channel change).

## 3. Coverage confession

The registry is the coverage authority. After the final SD-R8 run it
contains 123 clauses: 109 Full, no Partial, one accepted Untested
limitation, and 13 Excluded; `OPEN_UNTESTED` is empty and
`cargo test --locked --test obligations` verifies the structure. The
seven former fragment-scale clauses cite the declared run directly;
the accepted parser limitation is explicitly outside the verdict
prerequisite list.

Historical first attempt: SD-R8 added full-scale reachability and
declaration machinery but did not change the totals. At pinned φ=0, all M rows executed;
M1–M5 and M7–M13 (including M8's second profile lane) produced green
FullContract-labeled reports. M6 produced a non-eligible report:
G1/G2/G4/G5/G6 green, G3 false twice by 725 ms. Because
`FullContractRun::declare` refused that report, no completed run
existed, no one-report subset was verdict evidence, and all seven ids
remained Partial. The generated 4,096-case phase sweep was blocked by
the same deterministic contract conflict. Registry verification is
6/6 with `OPEN_UNTESTED` empty.

Final attempt after F2: the pinned φ=0 report set declared, then the
4,096-case generated-phase test declared every case in 298.84 s.
Every M row and both M8 provenance lanes passed G1–G6 as applicable.
The registry independently promoted exactly
`m6-g1-post-announcement`, `m6-queue-drains-new-pace`,
`m7-no-client-violation`, `m8-no-follow-on-violation`,
`g1-zero-client-violations`, `g2-ceilings-never-tripped`, and
`g3-over-delay-bounded`; obligations remains 6/6. No Partial clause
remains.

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

(SD-R8 reopened-range packet, 2026-08-15) — coverage after the
F4/F5/F9 repairs. **The registry now contains 124 clauses: 110
Full, no Partial, one accepted Untested limitation, and 13
Excluded** — this corrects the stale 123/109 totals above, which
described the pre-audit close (SD-R8-F10). The addition is
`m2-character-policy-lanes` (owner M2), citing the lane driver and
the declaration's negative test. New or strengthened evidence:

- The M8-keyed declaration lanes (F4): negative declare test pins
  the audit's exact missing-Known-lane state; `m8-both-lanes` now
  cites the guard as well as the driver's happy path.
- The character-policy lanes (F5): the M2 saturation shape against
  both character endpoints at both coverage levels and both φ,
  G1–G4 armed, hand-derived G4 fingerprints (720,581 ms
  character-list crossing two full sustained waves; 30,581 ms
  character crossing two burst windows) matched by the runtime
  arithmetic; every N23 endpoint required by `declare`
  (`MissingEndpointLane`).
- The endpoint binding (F9): `ReproductionMismatch` on any
  label/wire disagreement, pinned in the structural seam test and
  demonstrated end-to-end by the review's own mutation, now
  refused.

**What the character lanes deliberately do not cover:** they reuse
M2's saturation semantics only — no 429/recovery, phantom,
rename/shrink, or cancellation stimuli run against the character
policies (those behaviors are policy-generic and evidenced on the
other policies); the claim the lanes earn is the four-policy scope
of G1–G4 saturation conduct, not a per-policy repeat of the whole
M-series. The endpoint binding is exact single-endpoint equality; a
future phase-swept lane that legitimately spans endpoints will need
a deliberate relaxation (the judge comment states this). The
registry's `must_assert` semantic accuracy remains reviewed prose —
that standing confession is unchanged by this range.

Every ordinary scenario-driver and focused transition report remains
`ContractCoverage::Fragment` and explicitly fails
`verdict_eligible()`. Only the dedicated SD-R8 pinned/generated
producer labels reports FullContract, and its run-owned constructor
guards the declaration. Two-phase fragment tests remain boundary
checks, not an exhaustive property claim. Public actor tests cannot
make fuse thresholds reachable under intact D5; the internal trip
tests are deliberate fault-injection composition evidence.

Exact remaining ballot/closure items (items 1–3 of the round-five
residual set were discharged by the residual-items packet; item 1
below is discharged by the §7.4 gate packet, pending its review):

1. ~~The feedback-consistent §7.4 replacement calibration gate;
   retain the exhaustive fixed-trace counterexample as a
   diagnostic.~~ — **implemented 2026-08-15** (this packet; the
   band-edge test and enumeration are retained unchanged).
2. ~~A declared full-contract run for the seven fragment-scale
   clauses~~ — **green after Tom's SD-R8-F2 adjudication; awaiting
   independent review**:
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

(SD-R8 full-contract packet, 2026-08-15):

- 4,096 cases is the conservative existing property-test scale; the
  phase strategy spans the full 60,000 ms common cycle and explicitly
  weights before/on/after 5 s and 60 s boundaries. The run producer,
  not `FullContractRun`, owns this reachability claim so the declaration
  cannot manufacture scale from a small report set.
- M6/M7/M8 use 12/12/12 queued follow-on requests, with M7's eight-hit
  phantom burst. These are the smallest round-number shapes chosen
  above the relevant one-window thresholds; runtime assertions make
  loss of the intended scale fail.
- Reconciliation reads “local in-window” in client terms: its window
  includes configured N13 padding. Using raw server periods there made
  the client re-synthesize its own retained entries and broke both
  performance and truthfulness.
- After Tom's adjudication, the G3 oracle changed and ε did not. The
  implementation is deliberately smaller than the decision surface:
  replace phase-dependent raw `bucket_end + period` with independently
  derived `hit + period + bucket`; preserve every client, mock, actor,
  and judge mechanism.
- The full generated run remains `#[ignore]` because its 298.84 s cost
  is review evidence, not ordinary CI cost. The pinned declaration is
  active in the normal suite so a regression cannot silently restore
  the old blocker.
- **Proposed result statement for Tom/reviewer:** “The Rust actor/core
  demonstrably honors the modeled N-claims as one serialized gate in
  the offline calibrated harness for all four OAuth policies under
  Known 5 s/60 s bucket resolutions. It does so conditionally for
  `backend-item-request-limit`, assuming 60 s/60 s is no smaller than
  the server's actual bucket resolution. The conclusion carries U1–U5
  and the accepted future-parser limitation; it is not live-service
  validation.”

(SD-R8 reopened-range packet, 2026-08-15) — judgment calls:

- **Strict every-observation endpoint equality, not at-least-one.**
  Every current phase-swept lane is single-endpoint, so exact
  binding is available and conservative — and it immediately caught
  two real mislabels (the driver M9 row and the focused M9 race
  fixture, both StashList labels over Stash wires), which
  at-least-one semantics would also have caught here but exactness
  leaves no room for a mixed-wire lane to smuggle one labeled
  observation. The cost is explicit: a future legitimately
  multi-endpoint phase-swept lane must relax the check
  deliberately.
- **`MissingEndpointLane` requires every `Endpoint::ALL` member**,
  not only the two character endpoints: the verdict claims the
  whole N23 topology, and a guard that lists only the two newest
  lanes would re-create the F4 shape for the older ones.
- **Lane queue depth 12 at both coverage levels** (unlike the M8
  OAuth lane's fragment count of 1): the lanes arm G4, whose 1.05×
  bound at a one-request span would be fragile against the fixed
  service delay; uniform depth keeps fragment lanes meaningful and
  identical to full-contract up to the coverage flag, mirroring the
  main M2 row.
- **The G4 fingerprints were hand-derived before first run**
  (720,581 / 30,581 ms) and pinned as literals; they matched the
  runtime-derived minimum exactly, which is independent evidence
  the greedy arithmetic and the lane construction agree.
- **The M9 row correction is part of the F9 repair, not a separate
  finding**: the row label was wrong before the binding existed,
  harmless to coverage (StashList presence was over-supplied) but
  exactly the class of unbound label F9 condemns; it is fixed with
  a comment at the row.

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

(SD-R8 full-contract packet, 2026-08-15) — final matrix, entirely
offline: `cargo test --locked` — 170 passed / 0 failed / 4 ignored;
`cargo test --locked --release` — 168 / 0 / 4 (two debug-only drop-
bomb tests absent); `PROPTEST_CASES=4096 cargo test --locked` — 170 /
0 / 4; all-target clippy with warnings denied, fmt, and
`git diff --check` clean; obligations 6/6; sanitizer 4/4; both ignored
§7.4 release tests 2/2 green in 7.70 s. The full-contract scale-shape
test is green on the restored tree. The explicitly ignored pinned
full-contract attempt is the expected blocker reproduction, not a
green matrix member: all 14 reports are present, M6 G3 fails by 725 ms
at correlations 9 and 14, and declaration refuses M6.

Mutation checks were run from committed implementation and reverted
with `git checkout --`: (1) raw-period rather than padded-window
reconciliation → focused regression `left: 2, right: 0`; (2) remove
the declaration eligibility guard → fragment refusal returns
`Ok(FullContractRun { ... M1 Fragment ... })` instead of
`Err(ReportNotVerdictEligible { scenario: M1 })`; (3) reduce M6's
post-shrink queue 12→10 → `M6 must cross two complete five-hit
new-pace windows`. The lint-only local-binding adjustment was committed
before mutation 3 was repeated; its signature was unchanged. No live
service was contacted. No run declared `FullContract`, no registry row
was promoted, no gate/verdict slot was filled, and this implementing
session does not close SD-R8.

(SD-R8 continuation after Tom's F2 adjudication, 2026-08-15) —
final verification, entirely offline:

- Pinned φ=0 declaration: green, 14/14 reports, 0.07 s focused.
- Full generated-phase declaration: green, 4,096/4,096 cases,
  298.84 s; every case passed the run-owned declaration.
- `cargo test --locked`: 171 passed / 0 failed / 3 ignored.
- `cargo test --locked --release`: 169 / 0 / 3 (two debug-only
  drop-bomb tests absent).
- `PROPTEST_CASES=4096 cargo test --locked`: 171 / 0 / 3.
- Obligations: 6/6, independently reporting 109 Full / 0 Partial /
  1 accepted Untested / 13 Excluded; `OPEN_UNTESTED` empty.
- All-target clippy with warnings denied, fmt, and
  `git diff --check`: clean. Sanitizer: 4/4. Both ignored §7.4
  release tests: 2/2 green in 7.72 s.

Oracle mutation, run from committed `68100590` and reverted with
`git checkout --`: removing N13's bucket term made
`g3_oracle_pins_its_independent_padded_safe_arithmetic` fail
`left: 17015, right: 22015`. The pinned declaration then failed
`ReportNotVerdictEligible { scenario: M2 }`: M2 correlations
12/22 exceeded G3 by exactly 5,000 ms and correlation 32 by 60,000
ms; the same run also restored M6's 725 ms bucket-complement failures
and M10's 60,000 ms failures. On the restored tree, the focused
scenario-driver target and both declaration runs are green.

Diff-scope audit: the continuation changes the scenario-driver oracle
and its registry citation/coverage only. No client core, mock, actor,
judge/gate machinery, or network boundary changed. No live service was
contacted. Both verdict authorities agree and the evidence slots are
filled, but this implementing session does not close SD-R8.

(SD-R8 reopened-range packet, 2026-08-15) — verification presented
with this packet, entirely offline, on the tree at the F9 repair
commit:

- `cargo test --locked`: 172 passed / 0 failed / 3 ignored.
- `cargo test --locked --release`: 170 / 0 / 3.
- `PROPTEST_CASES=4096 cargo test --locked`: 172 / 0 / 3.
- Pinned φ=0 declaration: green over all sixteen reports (13 rows,
  M8's OAuth lane, both character lanes).
- Declared 4,096-case full-contract run: green in 26.81 s under
  the endpoint binding.
- Obligations: 6/6, independently reporting 110 Full / 0 Partial /
  1 accepted Untested / 13 Excluded; `OPEN_UNTESTED` empty.
- Both ignored §7.4 release tests: 2/2 green in 7.76 s. Sanitizer:
  4/4. All-target clippy with warnings denied, fmt, and
  `git diff --check`: clean.

Mutation checks, each run from committed code and reverted with
`git checkout --` (or a clean scripted revert), signatures exact:

1. F4: `run_m8_oauth_lane` profile flipped to Assumed → pinned
   declaration refuses `MissingM8KnownLane`.
2. F5: the CharacterList lane push deleted → pinned declaration
   refuses `MissingEndpointLane { endpoint: CharacterList }`.
3. F9 (the review's own mutation, repeated post-repair): the
   CharacterList wire lane replaced by a second Character lane
   with the CharacterList reproduction label retained for seed
   809 → the lane's judge call fails
   `ReproductionMismatch { id: 1 }` before any declaration is
   reached — the exact state that previously passed both
   authorities.

No live service was contacted. The verdict fills remain suspended
per the re-close review; this repairing session does not re-close
SD-R8 — the packet awaits the repeated independent re-close
review.
