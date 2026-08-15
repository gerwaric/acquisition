# Result draft: rate-limit-core spike (agenda item 5)

Status: accreting evidence container, created 2026-08-09 per the
sibling-docs convention in `design-brief.md`. This is the **skeleton
the spike fills as work happens** — implementation sessions record
evidence here the day it exists; the end-of-spike step is a hoist to
`docs/redesign/topics/` on `redesign` (plus the register-row update
and candidate-N-claim transcription), not a writing project. Scope
guard: this branch touches nothing on `redesign`; every transcription
noted below happens at hoist time.

Slots marked `⟨…⟩` are unfilled. A slot is filled by citing evidence
(a test run, a commit, a fixture), never by assertion.

N-numbers cite `docs/design/network-ground-truth.md`. D-numbers cite
`docs/design/network-redesign.md`. Scenario/gate/budget IDs (M/C/X/U,
G1–G6, B/O) cite `scenarios.md`. Design types cite `core-design.md`.

---

## §1. The question and its answer

**Register question** (`docs/redesign/README.md`): can a Rust client
demonstrably honor the N-claims in `network-ground-truth.md` under
burst load, as a single serialized gate?

**Definition carried to the register at hoist** (plan-review
addendum, finding 4): "single serialized gate" = one serialized
scheduling authority — the actor — with wire concurrency inherited
from D5's gate contract in full: in-flight cap 2, HEAD-exclusive
with writer preference, FIFO among ordinary waiters. Not literal
one-request serialization.

**Terminal condition** (addendum, finding 7): the spike completes
when offline conformance and this document are complete. Live
validation is a designed follow-up instrument, not a completion
gate; §6 holds any optional supplemental runs.

### Verdict

Two lanes, per the bucket-knowledge split (`scenarios.md` §1).
Prerequisites for **either** verdict slot (external review round 3
— explicit, so a green G-table alone can never justify a verdict):
M1–M13 with G1–G6 green over the lane's policies, **and** C1–C5
green, **and** X1–X2 green, with U1–U5 carried into the scoped
conclusion. X2 is load-bearing, not auxiliary: "single serialized
gate" is part of the register question itself. The SHELL-owned
dropped-ticket clause (adopted 2026-08-13) counts among the
verdict prerequisites: both verdict slots require it `Full`,
alongside X1–X2 (amended 2026-08-13, ballot pass — SHELL
previously appeared in no prerequisite lane).

- **Unconditional** — the four OAuth policies, bucket resolution
  `Known(5s/60s)` (N12): ⟨verdict; requires the prerequisites
  above over these policies⟩
- **Conditional** — `backend-item-request-limit`, bucket resolution
  `Assumed(60s/60s)` (not provably pessimistic; N14/N21 give no
  upper bound): ⟨verdict, stated *with* its assumption; same
  prerequisites, conditional lane⟩

G3 (ε = 500 ms) and G4 (1.05×) were finalized 2026-08-13
(`scenarios.md` §6 amendments), satisfying the finalization
prerequisite this paragraph previously stated for the verdict
slots.

## §2. How to read the evidence

Two orthogonal lane taxonomies apply:

- **Claim lanes** (what kind of knowledge a statement is):
  measured / estimated / inferred / external (URL + retrieval
  date, per the credential-custody precedent).
- **Test lanes** (which instrument judged it): mock-judged wire
  behavior / core-level property tests / fault-injection &
  structural / declared-untested.

Tests cite scenario IDs; scenarios cite N-numbers; this doc cites
both. Mock verdicts are **measured-against-model**; the §5 replay
calibration is what grounds the model in the observed lane. Every
failure anywhere must report its G6 reproduction record —
(seed, φ) mandatory for swept and property-generated tests,
optional for phase-independent and structural checks.

## §3. Conformance results

Coverage deltas in these tables ("… remains pending") are historical
as of the clause registry's landing date (2026-08-12); live coverage
is the registry — `src/obligations.rs` verified by
`tests/obligations.rs`, with `OPEN_UNTESTED` as the open-untested
list (see `status.md` §1).

### Mock-judged wire scenarios

| ID | Scenario | Sweep | Gates exercised | Result | Evidence |
|---|---|---|---|---|---|
| M1 | Cold start with residue (flagship) | phase-swept | G1, G2, G6 | partial — actor/judge evidence covers residues 0/1/9/10 at φ=0/1, the generated-φ mock-side residue sweep (residues 0–12 × generated φ over the 60,000 ms cycle, judged by the mock with G1 armed), exclusive boot HEAD, probe-429 seeding, and first-GET confirmation/escalation; reports remain fragments | 2026-08-14: `scenario_driver`, `actor_safety`, and `m1_residue_sweep` targets green (the sweep also at 4,096 generated cases, mutation-checked through G5 and G3); all M1 registry clauses Full. |
| M2 | Clean cold-start saturation burst | phase-swept | G1–G4, G6 | partial — 40-request actor run crosses both burst and sustained stalls; G4 minimum is independent runtime arithmetic over the actual policy, queue depth, D5 floor, bucket padding, and 81 ms service delay | 2026-08-14: `scenario_driver` target green; `m2_g4_minimum_is_runtime_derived_and_reaches_both_stalls` pins 122,581 ms. |
| M3 | Degraded HEAD | independent | G1, G2, G5 | partial — scoped refusal, unaffected-policy flow, and 60 s cooldown re-entry are pinned through the public actor | 2026-08-14: `scenario_driver` and `actor_safety` targets green; registry clauses Full. |
| M4 | Unexpected policy shape | independent | G1, G2, G5 | partial — scoped D4 refusal, unaffected-policy flow, and cooldown watch publication are pinned; the scenario report remains a fragment | 2026-08-14: `scenario_driver` and `actor_safety` targets green; registry clauses Full. |
| M5 | Policy rename mid-session | phase-swept | G1, G2, G6 | partial — forced φ=0/1 stale-window test caps exposure at D5 and proves no post-merge violation | 2026-08-14: `transition_timing` target green; registry clauses Full. |
| M6 | Policy shrink mid-flight | phase-swept | G1, G2, G6 | partial — forced φ=0/1 pre-announcement request is the sole organic exposure and recovers at the shrunk pace; full-contract G1/queue scale remains pending | 2026-08-14: `transition_timing` target green; `m6-preannouncement-exposure` Full. |
| M7 | Phantom same-account hits | phase-swept | G1, G2, G6 | partial — actor/judge driver covers a mock-owned phantom observation at φ=0/1; bursty threshold case remains pending | 2026-08-12: `cargo test --locked --test scenario_driver m1_m13_run_against_the_actor_and_the_judge` green. |
| M8 | 429 recovery and escalation | phase-swept | G1, G2, G5, G6 | partial — concurrent delayed originals serialize to one confirmation in flight; organic Retry-After is captured and honored; escalation/malformed matrix remains pinned by focused core/actor tests; only full-contract follow-on G1 remains | 2026-08-14: `transition_timing` and `actor_safety` targets green. |
| M9 | Phantom race at saturation | phase-swept | G1, G2, G5, G6 + characterization | partial — forced φ=0/1 race at 14/15: the B12-scripted 2 s reservation-to-receipt window admits a mock-owned phantom, the raced 16th hit draws an organic 429 attributed as §2 race exposure through the public `ExposureAllowance` seam (proven load-bearing: the same evidence without the allowance fails G1), and recovery completes per M8's asserts; the headroom record stays U5-excluded | 2026-08-14: `transition_timing` target green; M9/B13 registry clauses Full. |
| M10 | Agent-loop stress | phase-swept | G1, G2, G3, G6 | partial — actor/judge driver runs M10 at its stated scale: 300 enqueues, 30 spread queued cancellations, one proven-dispatched cancellation, 66 simulated minutes at φ=0/1. It non-blockingly polls each cancellation at Tom's 25ms simulated-time bound, then proves the dispatched response reconciles; it also checks drain, fuse quiet, in-flight ≤ 2, and the spacing floor. The fuse false-positive property — "never trips on any floor-compliant trace", headroom included — is **C3-owned** (see the C3 row); this row supplies the integration instance (the trace the actor emits under caller pressure is floor-compliant, with `fuse_quiet` observed alongside) and X1 the true positive: C3 ⊗ M10 ⊗ X1 discharge the clause (§9 round-four entry, 2026-08-12). G3 uses an independent half-open-interval permit oracle. Remaining: G3's epsilon cannot be finalized until the oracle models N13 padding (doc finding 12c). Reprioritization is no longer required of this row: Tom amended M10's stimulus list 2026-08-12 | 2026-08-12: `cargo test --locked --test scenario_driver m1_m13_run_against_the_actor_and_the_judge` green. |
| M11 | Layer-1 ceiling + Cloudflare terminal | independent | G2, G5 | partial — actor/judge driver covers injected Cloudflare terminal/halt (M11b); M11a's near-ceiling compliant sweep drives 301 floor-paced dispatches under a synthetic high-limit policy to the compliant maxima — exactly 4 per rolling second and 240 per rolling minute against the 20/1,000 ceilings — with zero trips, under both bucket profiles | 2026-08-14: `scenario_driver` and `m11_ceiling_sweep` targets green; `m11-compliant-never-trips` Full. |
| M12 | 4xx-tripwire obligations | independent | G5 | partial — injected 401/generic 4xx dispositions plus both actual probe and ordinary tripwire feeds are pinned; internally seeded threshold composition proves shared latch/drain/publication without weakening D5 | 2026-08-14: actor unit and driver targets green; M12/C4 registry clauses Full. |
| M13 | Gate structure on the wire | independent | G2 + gate-definition assertions | partial — actor/judge driver covers two unknown endpoints, forced HEAD delay, no HEAD overlap, and in-flight cap; FIFO/writer-preference cross-product remains pinned by focused actor tests and awaits its scenario assertion | 2026-08-12: `cargo test --locked --test scenario_driver m1_m13_run_against_the_actor_and_the_judge` green; prior focused evidence retained below. |

Driver integration status (2026-08-12, as corrected by the two review rounds below): `tests/scenario_driver.rs` drives every M row through the public actor handle and sends its mock-owned observations/state changes through `conformance::judge`; **every** row now runs at both φ=0 and φ=1, and M8 additionally runs both the OAuth Known and legacy Assumed profiles. Those two phases are the extremes of *boundary distance* — the mock reads `phase_ms` as the upcoming boundary, so φ=0 puts the first bucket edge a full bucket away (5,000ms and 60,000ms) and φ=1 puts it 1ms after t₀; `scenario_driver::swept_phases_are_separated_by_a_full_bucket` pins all four distances. Every row declares `ContractCoverage::Fragment`, so no report is `verdict_eligible()` and a green G5 here cannot be read as a scenario verdict. It is evidence of the actor-to-judge seam, **not a final M-series verdict**: the row-level coverage deltas above and doc findings 11 and 12 remain open. The run uses the draft G3/G4 values solely to exercise the judge; it does not fill either verdict slot.

*[Superseded in part, 2026-08-13 (DS-R1 stale sweep — the one live-reading paragraph the doc-split missed): four review rounds have run, not two (§9 entries); doc finding 11 is resolved and 12(a)/(b) are resolved — only 12(c) remains open (`status.md` §3 item 1). The paragraph's fragment/verdict rule stands unchanged. Dated text above preserved.]*

G1, G2, G3, and G5 are armed in every mock-judged scenario; the
column lists the gates each scenario is the *binding evidence*
for.

### Core-property tests

| ID | Property | Result | Evidence |
|---|---|---|---|
| C1 | Padding arithmetic safe over all φ | green — full N13 per-window padding uses each explicit Known/Assumed resolution; shared policy history is judged across every rule/window and the maximum required `NotBefore` wins; headroom remains zero | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 19 passed, including a generated C1 property over arbitrary histories, multi-rule definitions, and independently generated server phases plus explicit just-before/on/after rollover and zero-headroom/order-statistic cases; focused `PROPTEST_CASES=4096 cargo test --locked --test c1_scheduling every_reserved_outcome_is_safe_for_every_server_phase` green (4,096 cases); independent oracle bucketizes hits on the server phase rather than calling production scheduling arithmetic; `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green. No skew sensitivity observed because this slice has no server-clock input; O5 remains out. Audit hardening (2026-08-09, same day): the property now asserts on every generated case — the earlier body was ~97% vacuous (§3 register, item 7) — and the `NotBefore` branch is re-asked and oracle-checked, pinning exactness; re-verified at 4,096 cases |
| C2 | Header parsing / shape validation | green for the implemented core slice — raw-header parsing, RulePair shape, and frozen response precedence are executable; remapping/shrink remain explicitly out of this slice | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 44 passed overall: the 7 parser tests remain green and 15 disposition tests pin Cloudflare-before-parse, malformed/out-of-model-before-429, valid-429 handling, and ordinary/probe outcomes; `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green |
| C3 | Fuse trip logic | green for the implemented actor boundary — exact burst (10/11 + half-open edge), sustained (499/500), and the floor-compliant property are green. **This row owns `scenarios.md`'s "never trips on any floor-compliant trace" property, including its headroom claim; M10 owns only the integration instance** (that the actor's own trace is floor-compliant), and the two compose — see the 2026-08-12 round-four changelog entry | 2026-08-12: `actor::tests::{c3_and_x1_fault_injection_pin_burst_and_sustained_boundaries,fuse_uses_the_documented_half_open_boundaries,x1_fault_injection_trips_at_the_actor_transport_boundary,c3_floor_compliant_cadence_holds_the_steady_state_maximum,c3_floor_compliant_traces_never_trip}`. The cadence pin runs 1,500 dispatches (6.25 simulated minutes) uniformly at D5's floor and asserts the trailing-window peaks reach **exactly 4/s and 240/min** — the legitimate maxima, against clause limits of 10 and 500, which is the headroom occupied. The property generates irregular floor-compliant gaps over 750 dispatches (≥3.1 simulated minutes, so the sustained clause prunes across several windows rather than filling one); independent backward-scan window arithmetic, asserting on every step of every case; green at 4,096 cases (21 s focused run). |
| C4 | 4xx tripwire logic | green for the implemented counter — burst, sustained, and half-open edge boundaries pinned | 2026-08-12: `actor::tests::c4_pins_burst_sustained_and_exact_window_edges` green in debug and release. |
| C5 | Lifecycle invariants | green — reservation/rollback/unknown-outcome identity and abandonment semantics remain green; raw ordinary responses and tokenless probes still share one count-max/synthetic-history reconciler; unknown confirmation outcomes stay counted; abandonment now covers the confirmation half (a dropped confirmation ages out as a failed attempt instead of wedging the policy — §3 register, item 2) | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 44 passed: all prior C1/C5/reconciliation tests remain green, and the disposition suite pins confirmation rollback plus pessimistic unknown retention; focused `PROPTEST_CASES=4096 cargo test --locked --test response_reconciliation` remains green (4,096 cases for each of two generated properties); `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green. Audit hardening (2026-08-09, same day): abandoned-confirmation expiry pinned in debug and release; interleaving property extended with observed responses and non-FIFO token resolution (2,048-case focused run); 59 tests total |

### Fault-injection and structural

| ID | Check | Result | Evidence |
|---|---|---|---|
| X1 | Fuse true-positive, burst and sustained fault shapes — the lane upgrade from declared-untested | green for the actor boundary — fault-injected counter contents reach `start_dispatch`, the last common hook before `Transport::send`; production D5 pacing remains enabled | 2026-08-12: `actor::tests::x1_fault_injection_trips_at_the_actor_transport_boundary` pins the 11th burst and 500th sustained trips without creating a second scheduling path. |
| X2 | Transport boundary: one HTTP client, private, no second send path | ⟨…⟩ | ⟨…⟩ |

### Gate summary

All rows unfilled pending full contracts; fragment-level gate
evidence is green at φ=0/1 — see the driver status note above.
(One-line preamble in place of per-cell markers, Tom's DS-R1
closure disposition, 2026-08-13; the cells stay `⟨…⟩` until a
`verdict_eligible()` run fills them.)

| Gate | Statement | Result | Evidence |
|---|---|---|---|
| G1 | Zero client-caused violations (incl. follow-on) | ⟨…⟩ | ⟨…⟩ |
| G2 | Neither B10 ceiling rule tripped, armed everywhere | ⟨…⟩ | ⟨…⟩ |
| G3 | Per-dispatch over-delay ≤ ε (final ε: 500 ms — Tom, 2026-08-13, doc finding 12(c) decision) | ⟨…⟩ | ⟨…⟩ |
| G4 | M2 duration ≤ multiplier × padded minimum (final: 1.05× — Tom, 2026-08-13, same §6 finalization) | ⟨…⟩ | ⟨…⟩ |
| G5 | Every scenario's own assertions (stimulus and structural alike) | ⟨…⟩ | ⟨…⟩ |
| G6 | Reproduction record for every failure ((seed, φ) mandatory where swept/generated) | ⟨…⟩ | ⟨…⟩ |

### Mock + M-series harness slice (2026-08-12 — reviewed and closed)

Baseline: reviewed bootstrap head `b3a0e7d5` (the user's named
`17363429` baseline plus its review-status reconciliation commit).
Implementation commits: `4353fb03`, `05ee15d1`, `12a799f8`,
`4c69f05e`; review corrections: `f013acd3`, `6428f46c`,
`1a6124ed`, `606df936`, `03e3cf91`, `a74f9d5d`.

This slice implements the §7 reusable test side of the transport
trait, not the Tokio actor: independent server-phase counter
arithmetic, all five N23 policies and endpoint routes, full raw header
emission, non-counting HEAD behavior, residue and phantom injection,
policy rename/shrink scripts, organic restrictions, both B10 ceilings,
the B11 stimulus family, deterministic arrival/response delay, and the
B13 observation log. `Date` is emitted on every response shape from the
zero-skew paused clock. The M1–M13 metadata and G1–G6 judge are
executable; the sweep-plan constructor refuses any plan that omits the
shipped provenance-typed `Assumed(60s/60s)` client configuration.
The §4 sanitizer is a bounded allowlist over the real NetworkCapture
JSONL schema and emits only canonical D5 labels, rebased timing, allowed
headers/status/error fields, and the required provenance block.

Review hardening (2026-08-12): B13 now records a mock-owned transport
handoff before any scripted arrival delay, preserving its run-wide identity
even if cancellation prevents server receipt. The judge takes G3
eligibility, authorized exclusions, and M2's padded minimum only from a
scenario-owned oracle over B13 observations; it no longer accepts
actor-reported dispatch or duration values. Each M-row has a typed,
required scenario assertion. G1 exposure allowances bind to a mock-owned
phantom-injection or policy-mutation record, reject overflow,
duplicate/detached correlations and post-handoff reservations, and reject a
reservation at or after the scenario-derived observable instant. The unused
arbitrary route mutator was removed, so a policy rename must carry server
facts through the dedicated mutation path.

Tom completed the re-review on 2026-08-12. The three evidence-seam
findings (arrival-delay cancellation, state-change attribution, and stale
dispatch-sample terminology) are fixed and covered; no accepted-not-fixed
items remain in this slice. The Tokio actor shell is unblocked.

No M row is marked green from infrastructure evidence alone. The
frozen build order puts the actor last, so there is not yet a client
that can drive the transport seam; client scheduling, caller outcomes,
watch publication, cancellation/reprioritization, and final G1–G6
verdicts remain the actor slice's work. This is a coverage boundary,
not an inferred pass. **[Superseded in part, 2026-08-12 — dated text
preserved. The actor slice and scenario driver shipped; reprioritization
was removed from M10 by Tom (CN6) and is owed by no slice. The M rows'
current boundaries are the per-row deltas in the table above.]**

Doc findings and conservative dispositions exposed by implementation:

1. **B13/run growth has no stated finite cap.** The mock accepts at
   most 10,000 transport handoffs, 10,000 retained events per policy,
   and 10,000 mock state-change records; header/rule/window/script
   quantities are constructor-bounded too.
   At request 10,001 the harness latches exhausted and returns a typed
   transport error without growing history or the observation log; the
   next call returns the same refusal.
2. **The docs do not say whether merely knocking during an already
   active restriction renews it.** The mock refuses the arrival but
   renews only when that arrival is independently over a current
   policy window. The next early call remains refused through the
   original deadline; at that deadline it is judged solely by the
   then-current counters.
3. **Restriction identity across an arbitrary rule reorder/reshape is
   unspecified.** Hits are retained as facts, and the latest old active
   restriction deadline is copied to every new rule/window slot. The
   next call therefore cannot escape a live restriction because its
   old positional slot disappeared.
4. **B10-vs-B2 evaluation order is unstated.** Layer 1 is modeled as
   outside layer 2: once a B10 ceiling trips, the Cloudflare reply wins
   and that arrival does not increment policy counters. The next call
   remains subject to the rolling B10 history; policy state remains at
   its pre-challenge count.
5. **CN5 already records the exact-boundary quantization gap.** The
   implemented most-adversarial reading assigns an arrival exactly on a
   boundary to the new `[start, end)` bucket, giving it a full bucket of
   expiry extension. Just-before/on/after cases and an independent
   generated oracle pin the consequence.
6. **B14 specifies zero skew, not a calendar epoch.** The mock maps
   simulated t₀ to Unix epoch and advances at second-precision HTTP-date
   granularity. The next reply's Date is therefore deterministic and
   consistent with its scripted completion instant without importing wall
   time.
7. **The actor contract forbids aborting in-flight work, but mock-future
   drop behavior is unstated.** A received arrival is logged before its
   scripted response delay and occupancy carries an age-out deadline.
   If its future is dropped, the next arrival after that deadline prunes
   occupancy instead of inheriting a permanent false overlap.
8. **§7.4's required sanitized July 18 fixture is absent from the
   branch/workspace.** The §4 sanitizer exists and is tested, but no
   132-record replay or observed-lane calibration is claimed and no
   record is reconstructed from prose. The next replay remains
   unavailable until raw input is passed through the sanitizer or an
   already-compliant fixture is supplied.
9. **“t0 = first record” does not choose among scheduled/sent/received
   on a reply.** Sanitization uses the first available client-side field
   in scheduled→sent→received order and never the rounded server Date.
   The next record therefore preserves signed relative timing; a first
   HEAD, which has only received, begins at `received_ms = 0`.
10. **§2 requires independent unavoidable-exposure attribution but does
   not specify a state-change evidence shape.** The mock now records each
phantom injection and policy mutation with a bounded run-local ID and
instant. A scenario oracle derives that event's client-observable instant
from the script and B13 observations; an allowance binds to the event ID,
so an absent, duplicated, unrelated, post-arrival, unobservable, or
too-late claim is a structural judge error rather than an exemption from
G1.
11. **M10's pressure run has not reached its stated scale.** ~~The driver
    runs 16 requests~~ *(resolved 2026-08-12, see below.)* `scenarios.md` M10
    calls for hundreds sustained over many simulated minutes. This was the
    row's real remaining gap. Until it landed, M10's most distinctive assert
    — the fuse's *false-positive absence under saturation* — was untested:
    C3's sustained fuse is 500/60s and a compliant client at the 250ms floor
    emits ≤240 req/min (M11's arithmetic), so the ~2× headroom is only
    demonstrated by running long enough to occupy it.

    *Corrected 2026-08-12 (round four; the paragraph above is preserved as
    written):* the "was untested" claim was wrong when written, and scaling
    M10 is not what discharged it. `scenarios.md` assigns the fuse
    false-positive property — headroom derivation included — to **C3**,
    whose floor-compliant property had already landed in `e3efb812` three
    hours before the re-scope above. The composition is: C3 owns the
    property (§3 C3 row), M10 owns the integration instance (the actor's
    own trace under caller pressure is floor-compliant), X1 the true
    positive — C3 ⊗ M10 ⊗ X1 discharge the clause with nothing left to
    build. See the §9 round-four entry.

    **Superseded in part 2026-08-12 by the F8 re-review fix below.** M10 now
    runs 300 enqueues with 30 cancellations spread through the queue and
    ~~one caller initially dropped while dispatched~~ *(corrected
    2026-08-12 per F10: one caller proven dispatched and then explicitly
    cancelled — a dropped dispatched `RequestTicket` is covered by no
    test)*,
    spanning **3,963,500 ms ≈ 66 simulated minutes** *(figure corrected
    2026-08-12: originally recorded as 3,963,250 ms, measured before the
    250→25 ms harness-step change — round-four owed list)* across many window
    rollovers. All five of M10's stated asserts are checked directly from
    mock-owned wire evidence and the actor's published status: 270/270
    served, 30/30 queued cancelled callers resolved as `Cancelled` within the
    Tom-approved 25ms simulated-time bound, fuse quiet,
    in-flight ≤ 2, and the spacing floor never violated (absolute arithmetic
    over the wire log, against D5's literal 250ms rather than the actor's
    constant). The wire count is bounded two-sided rather than pinned —
    a cancel issued under pressure may or may not beat its own dispatch, and
    one did — with `served` pinned exactly so no wedge can hide in the band.

    *Superseded sub-finding (recorded 2026-08-12, resolved same day):* this
    entry originally read "M10 requires reprioritization, but the actor has
    no reorder command," and described the driver as unable to claim M10's
    reprioritization assertion. That was miscast — reprioritization appears
    in M10's stimulus clause, never among its asserts, so there was no such
    assertion to claim. Tom amended M10 to drop the stimulus (see the dated
    note under M10 in `scenarios.md`), on the grounds that `design-brief.md`
    already scopes reorder out of the spike. See CN6 in §4 for the finding
    that came out of it.
12. **G3's epsilon is an unmodelled-padding allowance, not a slop
    tolerance — §6 cannot finalize it until that is decided.** Originally
    filed as a relative-versus-absolute anchor problem; scaling M10 and then
    measuring at 25ms resolution refined it three times. (a) and (b) are
    resolved; **(c) is the open decision.**

    *(a) Permit availability — resolved 2026-08-12.* The oracle assumed no
    policy debt, which held at 16 requests and collapsed at 300: G3 read
    every legitimate window wait as a violation (112,250 ms for the 30/60s
    window, 1,497,250 ms for the 100/1800s). §6 names permit availability as
    part of the padded-safe time and the mock's observation log as a
    client-independent source, so the oracle now computes it from both,
    mirroring the mock's own counting predicate — a hit stays active until
    `bucket_end(at) + period`, so hit *k* may go once hit *k−H* has aged
    out, and HEADs are excluded because the mock counts an arrival iff it is
    not a HEAD and did not trip layer 1.

    *(b) Queueing time — resolved 2026-08-12.* §6 says "whenever a request
    is **queued** and eligible," but the oracle had no submission instants, so
    a request submitted long after it became policy-eligible was scored as
    though the client sat on it. The oracle now raises each observation's
    eligibility to the latest script submission instant at or before its
    dispatch. Note the mapping trap found on the way: `RequestId` and the wire
    correlation are *independent* counters — the actor allocates a fresh
    correlation per **dispatch**, probes and retries included — so
    `ticket.id()` is not `observation.correlation_id` and a per-request map is
    not available. The bound above needs no such map. Soundness rests on the
    arms awaiting outstanding work before the next submission; M10's burst is
    safe because its submissions share one instant.

    The harness step also dropped from 250ms to 25ms. That step is the floor
    under any G3 measurement — nothing smaller than one step is observable —
    and at 25ms the whole target, M10's 300-request run included, costs ~2s.

    *(c) The oracle computes the server's permit instant, not the padded-safe
    time §6 asks for — and that is what epsilon is really absorbing.* With (a)
    and (b) in place, measured lateness at 25ms resolution decomposes cleanly:

    | Where | Max observed | What it is |
    |---|---|---|
    | Every row away from a window rollover | **25 ms** (M8: 50 ms) | One harness tick: scheduling latency, at the measurement floor |
    | M10 at each window rollover | **275 ms and 475 ms** | The client's N13 pessimism padding, which the oracle does not model |

    §6 asks G3 to measure against "the padded-safe time"; the debt term
    computes the *server's* raw permit instant, so the client's deliberate
    padding at a rollover reads as lateness. Consequence, measured both ways:
    at epsilon = 100 ms an **unmutated** actor fails M10 on padding alone,
    while the same epsilon **catches** the 250→600 ms floor regression that
    epsilon = 500 ms lets through (round one's demonstrated hole). So epsilon
    = 500 ms is not slop tolerance — it is an unmodelled-padding allowance,
    and the two cannot be separated until the oracle models N13 padding.
    The choice: model padding and tighten epsilon to roughly 100 ms, gaining a
    G3 that catches pacing regressions; or keep epsilon near 500 ms and record
    that G3 cannot discriminate below the padding envelope. **Open — flagged
    for Tom.**
    **Decided by Tom, 2026-08-13: keep ε ≈ 500 ms** and record the
    limitation — the oracle stays independent of the padding model
    rather than mirroring it (ε only bounds the too-late direction;
    too-early keeps zero tolerance either way). Recorded with the
    known consequence above: a floor regression like 250→600 ms
    escapes G3 at this ε; the accepted mitigation is G4's 1.05×
    duration bound on M2, whose theoretical minimum is computed from
    the contract floor and therefore fails loudly on any sustained
    floor regression. The `scenarios.md` §6 finalization (next-work
    item 3) lands this as a dated amendment with the final values.
13. **M10 required “prompt” cancellation but gave no deadline.** The scaled
    driver waits until its four-hour simulated run has completed before
    awaiting cancelled tickets, so it establishes only eventual
    `Cancelled` resolution. That could not support M10's promptness assertion.
    **Resolved by Tom, 2026-08-12:** command ingress completion → caller
    `Cancelled` within 25ms simulated time, one harness tick. The driver
    sends all selected cancels without advancing time, advances exactly 25ms,
    and then awaits all 30 tickets before beginning the multi-minute run.
    This claim was re-reviewed and corrected below: at this point the driver
    did not yet cancel its dispatched caller, and its awaits did not pin
    readiness at the boundary.

All evidence below was re-run 2026-08-12, offline, with no socket and
no live service contact:

- `cargo test --locked` — green, 103 tests.
- `cargo test --locked --release` — green, 101 tests (the two core drop-
  bomb tests remain debug-only).
- `PROPTEST_CASES=4096 cargo test --locked` — green; all ten properties
  ran 4,096 cases, including the new independent mock-phase oracle.
- `cargo clippy --locked --all-targets -- -D warnings` — green.
- `cargo fmt --all --check` and `git diff --check` — green.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p
  'test_sanitize_capture.py' -v` — green, 4 sanitizer tests.

Property reachability: the new B3 property asserts bucket assignment,
one millisecond before expiry, and exact expiry on every generated case;
its expected value is independent integer arithmetic in the test and
never calls mock or production scheduling helpers. The conformance
judge rejects empty observations or scenario assertions; obtains
transport-handoff timing from the mock rather than actor reports; requires
the typed assertion for the selected M-row; and cross-checks `(seed, φ)`
against every swept observation. Exposure records are validated against the
same B13 log, a mock-owned state-change record, and the scenario-derived
observable instant, so gate checks cannot pass on empty, detached, or
self-exempted evidence.

Earlier implementation finding (2026-08-09, response-reconciliation slice):
the frozen `core-design.md` sketches full response entry points over
`ObservedResponse` and an endpoint label, but reaching a valid policy
observation from those inputs necessarily crosses response precedence,
endpoint mapping/remapping, policy-shape adoption, and bucket-model
selection — behaviors explicitly deferred from this slice. The
initial conservative seam accepted an already parsed, valid
`PolicySnapshot`.
An ordinary response targets the token's existing policy; a probe
targets the snapshot's already configured policy. Policy-name mismatch
and unknown-policy cases are typed errors: they neither remap nor create
a policy, and a dispatched ordinary token remains counted and is still
consumed. The two public paths delegate only the count-max/synthetic-
history mechanism to one internal reconciler. This keeps the seam narrow
and left the full precedence/remap contract for its scheduled slice.
The response-disposition slice has now replaced that temporary public
seam with raw `ObservedResponse` entry points and kept the same internal
reconciler; only remap/shrink adoption remains deferred as requested.

Implementation finding (2026-08-09, 429 recovery/disposition slice):
the frozen spike documents require restriction through `Retry-After +
max configured bucket + buffer`, but do not place or name that buffer
in the Rust API. N13 and accepted design D3 identify the existing
buffer as 1 second (D3's maximum retry sleep is 900 + 60 + 1 seconds).
The conservative implementation therefore owns a non-caller-selectable
1-second constant in the core; it does not let a shell weaken recovery
padding. Raw response headers enter the core as sketched, and an
unacceptable `Retry-After` is refusal-shaped because no safe restriction
deadline can be derived. Policy remapping/shrink adoption remains
deferred: a parsed ordinary observation whose policy name differs from
the reservation is a policy-targeted refusal, while a probe may seed only
an already-configured policy.

### Audit findings register (2026-08-09)

An independent audit of the implemented slices (bugs, doc
consistency, test vacuity, idiom) produced the findings below. Each
carries its resolution or an explicit **open — flagged for Tom**
marker; resolved items cite the commit series landed the same day.

**Resolved in code (with tests):**

1. **Empty-rules policy was a reachable panic.** A probe 429 on a
   policy inserted with no rules aborted in bucket sizing.
   `Policy::new` is now fallible (`EmptyPolicy`); the state is
   unrepresentable and `RefusalReason::PolicyHasNoRules` is gone.
2. **Abandoned confirmation tokens wedged the policy.** In release
   builds (no drop bomb) a dropped confirmation left the episode's
   slot held forever — `NotBefore(MAX)` at every future instant,
   violating the C5 abandonment clause. The slot now ages out with
   its history entry (resolving as a failed attempt: First consumes
   the attempt, Final escalates), and the confirmation helpers
   tolerate a late response for an already-expired slot by routing
   it through the ordinary paths. Attribution choice taken
   conservatively: a stale-confirmation 429 **joins** the episode
   its attempt was already accounted against (the `open_or_join`
   generation guard widened to `<=`) rather than escalating —
   flagged here because the docs are silent on the case.
3. **Unusable `Retry-After` left the core unprotected.** A 429 with
   a valid policy observation but missing/invalid/above-cap
   `Retry-After` refused with *no restriction recorded*; the next
   `try_reserve` granted straight back into an active restriction.
   Both entry points now record a conservative `RETRY_AFTER_CAP`-
   length restriction (generation bump included) before refusing.
   Neither the precedence list nor the probe table names this case
   — a doc gap to close when `core-design.md` is amended or
   superseded by code authority.
4. **Phantom synthesis was unbounded.** `current-hits` is a raw u32;
   a legal `4294967295:10:0` state header would have materialized
   ~103 GB of history. Synthesis now targets
   `min(reported, largest configured max_hits)` — beyond that bound
   every configured window is saturated at `now` and further entries
   move no deadline. The pessimism property is restated as
   `local >= min(reported, cap)`.
5. **`RETRY_AFTER_CAP = 900 s` adoption recorded.** The constant is
   D3's product-policy cap (`RETRY_AFTER_CAP_SECS`, longest observed
   restriction 600 s per N23 plus headroom); it was implemented
   without an adoption record. The 900/901 boundary is test-pinned.
6. **`Retry-After` grammar pinned:** delay-seconds only, strict
   ASCII digits (no `+`, matching the triplet parser, which is now
   equally strict), surrounding whitespace trimmed in both parsers.
   The RFC 9110 HTTP-date form is deliberately out of model and
   lands in the conservative-restriction refusal above.
7. **The C1 property was ~97% vacuous** (measured): zero-max_hits
   rules and oversaturated generated histories meant a default run
   asserted ~7 times in 256 cases. The generator now draws
   `max_hits >= 1` (the zero boundary has its own pinned test) and
   the property asserts on *both* branches — a `NotBefore` answer
   must grant when re-asked at exactly that instant, oracle-checked,
   which also pins the arithmetic as exact rather than sufficient.
8. **The reconciliation "exact" oracle mirrored production** (it
   called `count_within`); it now computes from the test's own
   shadow timestamp list with plain integer arithmetic.
9. **C5 interleavings** now include observed responses and resolve
   tokens in generated (non-FIFO) order, per the scenario text.
10. **`StateChanged` emission rule defined:** emitted iff the call
    mutated engine state (synthesis, restriction, episode
    transition, newly halted); previously hard-coded `true` on the
    main paths and unasserted by any test.
11. **Coverage fills:** all `PolicyParseError` variants, all four
    required headers' absence, the O6 case/duplicate domain (pinned
    as current behavior: case-insensitive lookup, duplicate rule
    names yield duplicate rules, first header value wins), generic
    4xx completes and reconciles, `Retry-After: 0` exact deadline,
    Halt superseding the final-attempt matrix row, and the
    entry-point invariant swept across nine response shapes.

**Decided by Tom, 2026-08-09 (same-day review of the register):**

- **`NotBefore(SimInstant::MAX)` sentinel → `ReserveOutcome::Blocked`.**
  NotBefore is now always a real, sleepable future instant; the two
  wait-for-an-event cases (confirmation in flight, wire-legal
  zero-hit rule) return the distinct `Blocked` variant. Implemented
  and test-pinned. ⟨"Wire-legal zero-hit rule" superseded by D8
  rejection (external review finding 3): zero-hit rules no longer
  parse; the `Blocked` answer survives only for constructed
  policies, as defense in depth.⟩
- **Probe-429 episode follows the full confirmation matrix.**
  "Single confirmation attempt" in the M1 probe-429 prose meant one
  in flight at a time, not a one-attempt cap; the matrix's
  two-attempt design (external review F6) governs. Clarified in
  `scenarios.md` and `core-design.md`; pinned by a test (500 on the
  first GET preserves the episode and earns the final attempt).
- **Precedence-list ordering:** editorial note added to
  `core-design.md` — the numbered list orders dispositions, not
  execution; reconciliation of a valid observation precedes status
  handling, gated only by the Cloudflare and parse rules.
- **Stale-confirmation late 429 joins, not escalates** (register
  item 2's embedded judgment call): Tom confirmed the
  no-double-count reading — a trial already written off by
  abandonment aging charges its attempt once; its zombie 429 folds
  into the existing episode (restriction still recorded) rather
  than also tripping the shutdown. Behavior unchanged from the
  hardening pass; now decision-backed rather than implementer-read.

**Decided by Tom, 2026-08-09 — `bootstrap-seeding.md` accepted
(revision 2):** one global positional bucket constant at engine
construction (Assumed(60s/60s) shipped — C++ parity; flip to
Known(5s/60s) on U3 evidence); dynamic seeding of any valid observed
policy, no name-keyed refusal; `ProbeReady { policy }`; `RuleScope`
deleted; shape validation stays strict (acceptance review confirmed
the shape/identity split). The note's §3 amendment to `scenarios.md`
§1 is sanctioned and lands with the seeding slice; its §5 is the
slice contract. **The register has no open items** — the seeding
slice is the next work, and the first to run under the slice-review
process end to end.

- **The probe/bootstrap seam cannot discover a policy.** There is
  no `PolicySnapshot` → `Policy` construction path (nothing assigns
  `RuleScope` or `BucketModel`), `ProbeReady` carries no policy
  name for the actor's endpoint→policy map, the `scenarios.md` §1
  bucket-resolution table has no code home, and `RuleScope` —
  documented as parsed — is never produced by the parser. "Mapping
  seeded" is unreachable as written; a seeding design is needed
  before the actor slice.

  *[Marker, 2026-08-13 (DS-R1): this bullet predates the acceptance
  block above it — it is the finding that motivated
  `bootstrap-seeding.md`, whose slice built the path and closed
  2026-08-10 (§9). Resolved; preserved as the record of why the
  seeding design exists.]*

### External review register (2026-08-09, second independent review)

Tom commissioned an independent frontier-model review of the repo
after the audit hardening pass; it found six real defects at exactly
the seams-and-silences the post-mortem predicted — including two in
the audit's own fixes. All six are resolved the same day (commit
series 9ed954c1..; every finding reproduced by a failing test before
its fix). The composite four-part hand-off the review noted was
missing now exists (`core-handoff.md`).

1. **Terminal/suspended state ignored by late responses** — a 429
   after halt/suspension answered `Requeue` (probe lane:
   `ProbeReady`), promises only a refusing `try_reserve` could
   receive. Rule now encoded: send-promising dispositions (Requeue,
   ProbeReady) are gated on halted/suspended state and refuse
   (`RefusalCause::Halted` / `EscalationSuspended`);
   outcome-delivering dispositions still reach their callers.
2. **Reachable panic in the audit's own wedge fix**: late original
   429s advance `restriction_generation` past `opened_generation`,
   so an expired-stale confirmation carries a *newer* generation and
   its zombie 429 failed the `open_or_join` `<=` assert. The assert
   stated a false invariant and is removed — joining is conservative
   for every generation. **Doc finding (shell obligation)**: aging
   alone cannot distinguish a dropped token from a slow live one;
   the shell must resolve every token (response/unknown/rollback)
   well inside the aging horizon (per policy, the largest padded
   window — the horizon the implementation and hand-off use; a
   single shell-wide timeout must beat the smallest such horizon
   across policies) — and a shell that violates it gets the
   defined degraded behavior (attempt written off; brief
   double-confirmation exposure), never a wedge or an abort.
3. **D8 grammar enforced**: empty/whitespace policy names, zero
   limit hits, and zero periods are typed parse rejections
   (post-seeding they would have been wire inputs permanently
   blocking a policy). Engine-level zero-hit handling stays as
   defense in depth.
4. **Absolute wire ceilings** — the synthesis cap was circular once
   seeding makes configuration wire-derived. **Doc finding (chosen
   values)**: 8 rules/policy, 8 triplets/rule, `max_hits` ≤ 10 000,
   periods ≤ 3600 s; boundaries pinned at n/n+1. ⟨Superseded
   2026-08-10: the evidence claim here (hits ≤ ~45, periods
   ≤ 300 s) was wrong — N23's legacy Ip rule reaches 180 / 1800 s.
   Tom raised the period ceiling to 21 600 s on the corrected
   evidence; the follow-up register below has the decision.⟩
5. **Physical history retirement**: entries now retire at both
   mutation surfaces once aged past the largest padded window;
   token-consuming paths tolerate a retired entry. An observation
   window longer than every configured padded window may
   re-synthesize after retirement — the pessimistic direction.
6. **Typed internal deadlines**: `WindowDeadline { Open, At, Never }`
   replaces the internal `SimInstant::MAX` sentinel, so saturated
   deadline arithmetic can never read as "never".

Test-evidence fixes from the same review: the interleaving property
generates ≥ 1 operation; `assert_pessimistic` does its own windowing
arithmetic over raw entry timestamps; the synthesis cap is pinned at
511/512/513.

### Follow-up verifier register (2026-08-10, review of the fix series)

The same external reviewer verified the six-fix series: five fixes
held; two residual defects and four doc corrections came back, all
resolved same-day (Tom approved the batch):

1. **Probe-lane 429s skipped `record_restriction` when
   halted/suspended** — the finding-1 gates ran before the 429
   branch, so a suspended policy's valid probe 429 refused without
   recording the declared restriction, contradicting the hand-off's
   uniform-pessimism judgment. Bookkeeping now precedes the
   disposition choice in both lanes; both branches reproduced by
   failing tests first.
2. **Byte-length ceilings** completed the wire-bound rule: policy
   names ≤ 256 bytes, rule names ≤ 64, diagnostics truncate raw
   wire text to 64 — no wire field sizes an allocation. Pinned at
   n/n+1.
3. **Corrected evidence, Tom's ceiling decision**: observed wire
   maxima are 180 hits / 1800 s (N23 legacy Ip rule), not 45/300 —
   the 3600 s ceiling was only 2x observed. Tom chose 21 600 s
   (6 h): a period sizes no allocation, and an over-ceiling policy
   refuses the endpoint, so the ceiling favors availability under
   N9's dynamic-policy premise.
4. **Doc reconciliation**: aging-horizon wording aligned with the
   implementation (largest padded window per policy); the zero-hit
   "wire-legal" register wording marked superseded by D8; test
   counts corrected.

The verifier's second pass on this batch (same day) held the
behavioral fixes and found two more:

5. **P2 — wire length still sized parsing work**: the field ceilings
   bounded copies, but `to_str`/trim/split/digit scans ran over the
   full wire length first. A 1024-byte whole-value gate now checks
   raw bytes before any conversion or scan, in `required_header`
   and `parse_retry_after` both (`HeaderValueTooLong` typed; an
   oversized Retry-After is unusable and records the cap). Pinned
   at n/n+1 through values that parse fully at the gate.
6. **P3 — residual doc drift**: the `Blocked` rustdoc's "wire-legal
   zero-hit rule" (superseded by D8), the changelog's 67/65 (the
   pre-batch state was 68/66 — a miscount at writing), and the
   hand-off's too-strong "before any allocation" (now "without
   wire-sized allocations or wire-sized parsing work"). All fixed.

Suite after the batch: 73 debug / 71 release.

Open obligation carried to the actor slice: replace "any sane
transport timeout" with the exact enforced timeout (or its stated
relationship to the aging horizon) plus a test. *[Discharged
2026-08-12 by the actor slice — see the entry directly below and its
closure paragraph ("both obligations … are discharged"). Marker
added 2026-08-13, DS-R1.]*

### Tokio actor slice (2026-08-12 — closed)

Implementation currently uses a 30 s enforced transport timeout, classified
as an unknown outcome so a dispatched reservation remains counted. It is below
the smallest padded N23 horizon (10 s period + 60 s configured bucket); the
exact relationship and timeout test are recorded in `actor-handoff.md`.

Actor-owned queue capacity (10,000 pending entries), the five-endpoint D5
record bound, and C4's literal “same shape as C3” 4xx thresholds are new doc
findings: the docs did not name the first two bounds or independent C4
thresholds. The conservative implementation rejects overflow before queue
growth and uses C3's 11/1 s and 500/60 s thresholds for all observed 4xx.
The next call after capacity refusal remains independently schedulable when
capacity is available; the next call after a C4 trip sees terminal halt.

Review revision (2026-08-12): writer preference is now structural. The actor
drains ready command ingress before any permit decision and scans the explicit
deque for an unknown endpoint; a queued HEAD blocks a front GET until current
readers drain, then takes exclusive occupancy. `actor_shell` pins this with a
forced delayed reader. The shell now bounds a response body at 16 KiB and its
header map at 32 headers / 256-byte names / 1,024-byte values before clone or
signature scan; n/n+1 boundaries are pinned. Overflow resolves ordinary work
as unknown and probes as D4 failure, preserving pessimism. The trait itself
receives a materialized `Vec`, so every production transport implementation
must apply the same body read cap before constructing `WireResponse` (X2).

C3/C4 and X1 actor-boundary evidence now cover reviewed burst/sustained thresholds and the
floor-compliant generated trace; M8’s retry and first-confirmation escalation
and M13 writer preference, HEAD exclusivity, FIFO, and in-flight cap now have
actor tests. M5/M6 now preserve stable in-flight reservation anchors while
adopting the response's remapped name and current rule judgments; focused
actor scripts pin both paths. A policy name colliding with a separate existing
route is a documented conservative refusal, not an invented history merge.
Review revision: post-remap refusal targets resolve to the current visible
route, so actor scoped failure drains queued callers; the M6 actor script now
shrinks `stash-list-request-limit` from 10/30 to 5/5 with response-held state
and demonstrates no dispatch before the 120-second padded deadline. The
transport trait now returns only bounded `WireResponse`; raw response vectors
are validated before they can reach the actor. Accepted X2 limitation: the
spike cannot force a future HTTP parser's upstream allocation cap, so that
transport implementation must cap body collection itself.

Offline implementation evidence (2026-08-12): `cargo test --locked` — 124
debug tests green; `PROPTEST_CASES=4096 cargo test --locked` — all existing
generated properties green at 4,096 cases; `cargo test --locked --release` —
122 tests green; all-target clippy with warnings denied, fmt check, and diff
check green. `actor_shell` pins paused-time probe→GET pacing, distinct B13
correlations, queued and dispatched cancellation, D4 failure, timeout
retention, and Cloudflare halt/watch behavior. Final M-series scenario runs
remain unclaimed in `actor-handoff.md` §3. *[Marker, 2026-08-13: still true
that no full-contract M-series run exists, but the live location for what
remains is `status.md` §5, not a closed hand-off.]*

### Bootstrap-seeding slice (2026-08-10 — reviewed and closed)

Baseline: `a3245e8667f15524fc837618131d5f692cd2e860`.
Implementation commit: `708b32d8`.
Review-close commit: `17363429`.
The accepted `bootstrap-seeding.md` §5 contract is implemented and
Tom's review is complete; the mock slice is unblocked:

- `PolicyEngine::new` now requires one explicit positional
  `BucketModel`; there is no implicit `Default` construction.
- Valid probe observations dynamically build non-empty `Policy`
  values from their bounded parsed `RulePair`s, applying that one
  bucket model uniformly. Ordinary unknown/mismatched observations
  remain refusal-shaped for the deferred M5/M6 slice.
- `ProbeReady { policy }` exports the discovered identity without
  moving header parsing into the shell. `RuleScope` and its dead
  property-test dimension are removed.
- `scenarios.md` §1 now records the accepted evidence/runtime split:
  its name-keyed table scopes verdict evidence; runtime seeding uses
  the single provenance-typed global default.

Focused evidence: `response_reconciliation` pins fresh-policy
residue, observed-shape copying, uniform default buckets over two
rules, existing-policy isolation, zero-residue notification truth,
and repeat idempotence; `response_disposition` pins discovery on
valid 429, non-seeding on malformed 429, and valid-5xx
seed-without-readiness behavior. The pre-existing C1/C2/C5 and
response suites remain green after the constructor/API migration.

Gate evidence, all run 2026-08-10 in
`spikes/rate-limit-core/`, entirely offline:

- `cargo test --locked` — green, 75 tests.
- `cargo test --locked --release` — green, 73 tests (the two debug
  drop-bomb tests are intentionally absent in release).
- `PROPTEST_CASES=4096 cargo test --locked` — green; all nine
  property tests ran 4,096 generated cases, with the reachability
  accounting in `bootstrap-handoff.md` §3.
- `cargo clippy --locked --all-targets -- -D warnings` — green.
- `cargo fmt --all --check` — green.

No new wire quantity bypasses the existing parser ceilings: seeded
names, rule counts, triplet counts, hit limits, periods, and whole
header work are bounded before policy construction. The cumulative
number of boot-discovered policies remains an actor-owned structural
bound (D5's five endpoint labels plus N16 exactly-once probing), not
a second name-based refusal in the core; the actor is still unbuilt
and this obligation is confessed in the slice hand-off. *[Marker,
2026-08-13: the actor has since been built and closed (2026-08-12)
and pins the bound; dated text preserved.]*

## §4. Candidate N-claims

Transcribed to `network-ground-truth.md` at hoist, cited by
number there. Accumulated during reconciliation and design; each
survives or falls on its cited source, independent of the spike
verdict.

| # | Candidate claim | Claim lane | Source | Status |
|---|---|---|---|---|
| CN1 | Invalid-request (4xx) budget: too many 4xx responses in a short period restrict access; 429s double-dip (policy budget + invalid budget); threshold parameters undocumented (sibling of Q8) | DOC, Confirmed | developer docs, retrieved 2026-08-09 (verbatim quotes in the charter's 4xx entry) | awaiting transcription |
| CN2 | Cloudflare challenge-block signature: layer-1 blocks can present as 403 + `cf-mitigated: challenge` (not 1015), no rate-limit headers (extends N3) | external | community.cloudflare.com thread, retrieved 2026-08-09 (URL in the charter's bucketing entry) | awaiting transcription |
| CN3 | Recourse asymmetry: layer-1 blocks may be invisible to GGG support and unappealable for non-business Cloudflare users (informs Q7) | external | same thread as CN2 | awaiting transcription |
| CN4 | Trade-API rules carry three windows per rule — the RulePair shape is not universal; a non-pair policy is out-of-model, not impossible | external | pathofexile.com forum thread 3056323, retrieved 2026-08-09 (URL in the charter's bucketing entry) | awaiting transcription |
| CN5 | N11–N13 under-specify bucket quantization semantics (when a hit's age is measured / when it leaves the window); the spike's mock adopts the most-adversarial consistent reading — timestamp rounds up to bucket end, entry never quantized (N25 pins immediate 1:1 increment) | inferred (model choice); the gap itself is a doc finding | `scenarios.md` §7 B3, 2026-08-09 | awaiting transcription |
| CN6 | **Reprioritization is cheap in the actor shape and was not in the coroutine/facade shape — `network-redesign.md` R7/D6's "not cheap later" warning does not carry over.** It failed there for a specific reason: the stop token is per-update, so per-entry cancellation did not exist and reorder needed entry identity invented first. The Rust actor already has that identity (`RequestId`, `Command::Cancel` doing positional removal on the owned deque), and its dispatch loop reads only `queue.front()` — no scheduling decision depends on arrival order. FIFO is emergent from append-at-back/take-from-front, not assumed; the actor already dispatches out of arrival order for writer preference. Reorder is therefore `remove(pos) + insert(pos)`, and the expensive part is contract, not code: D5's "no lane starvation" clause has no rule, so whoever adds reorder decides it. **This is evidence for the `design-brief.md` thesis that queue-as-data picks the actor shape.** | measured (structural, this branch) | `src/actor.rs` dispatch loop and `Command::Cancel`; contrast `docs/design/network-redesign.md` R7/D6 and `network-redesign-reviews.md` (2026-07-19 errata, "stale cancel+resubmit reprioritization claim removed"); Tom's decision 2026-08-12 | awaiting transcription |

⟨New candidates minted during implementation land here the day
they appear — writing the mock exposing an N-claim ambiguity is a
finding in its own right (charter, step 1).⟩

**Tripwire on CN6 (the one thing that would invalidate it):** CN6 holds
because the actor has *one* global deque, so a queue position is
unambiguous. `network-redesign.md` describes per-policy FIFOs preserving
source traversal order; if this actor ever fans its queue into per-policy
lanes, "reorder to position N" stops having a single meaning and
cross-lane priority becomes a genuine design decision. **That is the
moment to revisit reorder — not before.** Recorded because a session that
hits per-policy lanes will not otherwise know a deferred decision was
resting on the single-deque property.

**Note for whoever reads this after the C++ docs are superseded:** CN6 is
a claim *about the comparison*, so it needs the superseded document to
stay legible. If `network-redesign.md` is retired or this spike moves to
its own repository, carry CN6's contrast with it — the R7 "not cheap
later" warning is exactly the kind of conclusion that outlives its
premises and gets re-inherited by a shape it never applied to.

## §5. Calibration: capture replay

Per `scenarios.md` §7.4 — the piece that grounds the model in the
observed lane:

- Fixture: `fixtures/capture-20260814-wired.json` — sanitized
  2026-08-14 capture, 387 records, sanitizer v2, capture schema
  v1; canonical per Tom 2026-08-14 (his typical network
  condition; condition labels attested by Tom). Supplemental
  latency comparison: `fixtures/capture-20260813-vpn.json`
  (1,129 records, sanitizer v2, operator-supplied UTC offset
  validated per record against server `Date`). Median
  `sent→received`: 81 ms wired / 148 ms VPN. The July 18 capture
  anticipated at drafting was superseded by these.
- Initialization: four boot HEADs seed their endpoint state exactly
  once at t₀ (canonical residue is zero); they are excluded from the
  383 ordinary replay arrivals. The bounded parser caps input at
  2 MiB, 32,768 JSON items, depth 16, and 4 KiB strings.
- Calibration gate: **Tom adjudicated the fixed-dispatch every-φ
  requirement as a frozen-contract expectation error on 2026-08-14.**
  The violating set
  remains 1,052 phases in 20 disjoint
  bands, φ=7,454–7,466 through 25,854–25,944 (initiating replies
  110–119 and 125–134, every band `stash-request-limit` sustained
  31/30) — SD-R5-F2's 2026-08-14 amendment of CR-R1-F1, whose
  single-band record was the asserting gate's first-failure abort.
  The band table is `VIOLATING_BANDS` in `tests/capture_replay.rs`
  (active band-edge test + ignored exhaustive enumeration). Band-one
  arithmetic is unchanged: counted reply 110 (fixture record 114)
  dispatches at t=727,453 ms, one millisecond before 25 earlier hits
  expire, where the server recorded `6:300:0`. `CounterModel` and
  independent arithmetic agree. B3 and both fixtures remain
  unchanged; the fixed-trace failure and exhaustive band set are
  retained as diagnostics. A precisely specified feedback-consistent
  replacement gate and its green run are still owed.
- Diagnostic — saturation-state agreement (15/15, 30/30; N25/N26):
  phase 0 matches all 43/43 recorded saturation components,
  including 15/15 and 30/30. This is evidence for the replacement
  calibration design, not a gate by itself.
- B12 delay re-anchor: canonical median `sent→received` is 81 ms
  across 383 samples; `DEFAULT_SERVICE_DELAY` is now 81 ms, replacing
  the 50 ms placeholder.

## §6. Supplemental live evidence (optional by design)

Not a completion gate (terminal condition, §1). Any entry here ran
under the charter's logged run protocol — production-
indistinguishable pace, halt-on-first-violation, fixed safety
rails, Tom's explicit go-ahead per run.

- Gentle confirmation epilogue: ⟨if run: date, scope, outcome —
  never the evidence, a sanity check only⟩
- Validation-run ledger: ⟨dated, machine-readable entries; first
  designated target is U3's named hypothesis — legacy burst
  resolution = 5s; one 429 falsifies decisively, passes accumulate
  phase-swept confidence only⟩
- GGG correspondence: ⟨tier-assignment / legacy-resolution reply,
  when it arrives — may retire U3 without spending runs; annotates
  the charter's bucketing entry⟩

## §7. Declared-untested register

Carried verbatim from `scenarios.md` U-series so the hoisted doc
states its own blind spots: U1 remap triggers (reactive handling
M5 is the tested surface), U2 server-side 4xx restriction behavior
(obligations tested, M12), U3 legacy bucket resolution (conditional
lane; §6 instrument designed), U4 real layer-1 rules (deliberately
uncharacterized, N4 strategy; B10's numbers are inferred-lane).

## §8. Reusable artifact

⟨If the conformance suite works out: the mock (counter engine +
delivery shim, `scenarios.md` §7.1 layering) plus M1–M13 are the
acceptance tests any future core's limiter must pass — including,
via a standalone delivery shim, the C++ client. This outcome
outlives the spike branch; record what exists and where.⟩

## §9. Changelog

- 2026-08-09 — skeleton created (agenda item 5); all five design
  items closed; no code yet. Candidate register seeded with
  CN1–CN5.
- 2026-08-09 — first-eyes pass (nine fixes) and external design
  review (Q&A round plus three findings rounds) completed; review
  closed with the design declared implementation-ready. Fuse
  gained its burst clause; probe-429 lifecycle, response
  precedence, and the probe outcome table pinned. The committed
  sibling docs are now the frozen implementation authority.
- 2026-08-09 — implementation began with the C2 header-parser and
  RulePair-shape slice. Seven offline tests pass (two property
  tests); response-precedence coverage remains before C2 is
  complete.
- 2026-08-09 — C5 reservation lifecycle landed: identity-bearing
  local and synthetic history, exact rollback, pessimistic unknown
  outcomes, and the guarded debug drop bomb. Fifteen offline tests
  pass overall; C1 padding and response reconciliation remain later
  slices.
- 2026-08-09 — C1 scheduling arithmetic landed: each saturated
  window adds its explicit Known/Assumed bucket resolution to the
  rolling expiry, and the shared-history policy answer is the
  maximum across every window and rule with zero headroom. Nineteen
  offline tests pass overall, including the independent-phase C1
  property (plus a focused 4,096-case run) and explicit
  rollover-boundary cases; all eight C5 tests remain green.
- 2026-08-09 — valid-observation response reconciliation landed:
  ordinary responses consume their reservation token, probes remain
  tokenless, and both use one pessimistic maximum-deficit merge into
  shared policy history. Twenty-nine offline tests pass overall;
  focused M1/M7 core cases and two 4,096-case reconciliation property
  runs cover boot residue, phantom hits, monotonicity, identity, and
  exact synthesis without starting the mock harness.
- 2026-08-09 — 429 recovery episodes and raw response disposition
  landed as one lifecycle slice. Forty-four offline tests pass overall;
  the 15 focused tests cover restriction generation/deadlines, arbitrary
  concurrent pre-restriction sets, single-confirmation concurrency, the
  complete two-attempt matrix, malformed-429 and Cloudflare precedence,
  the full probe table, and probe-429 seeding. The generation-set
  property plus the two reconciliation properties passed focused
  4,096-case runs; all C1/C5 tests remain green. Required fmt, locked
  test, and locked all-target clippy gates are green.
- 2026-08-09 — audit and mechanical hardening pass (see the §3 audit
  findings register). Four defects fixed with tests (empty-rules
  panic, abandoned-confirmation wedge, unbounded phantom synthesis,
  unusable-`Retry-After` unprotected refusal); C1 de-vacuized and
  strengthened to exactness on the `NotBefore` branch; reconciliation
  oracle made production-independent; C5 interleavings extended with
  observe and non-FIFO token order; strict digit parsing; StateChanged
  emission defined and asserted; coverage fills across C2 and the
  disposition suite; idiom pass (pub fields on plain-data types, one
  shape check, one confirmation-failure path) removed ~100 net lines.
  Fifty-nine offline tests pass in debug and release; fmt and
  all-target clippy green. Four design questions remain open in the
  register, flagged for Tom: the NotBefore(MAX) sentinel, the
  probe/bootstrap seeding seam, probe-episode attempt-count prose vs
  matrix, and the precedence-list ordering note.
- 2026-08-09 — Tom reviewed the audit register same-day (process
  change from the post-mortem: slices now end at review, codified in
  AGENTS.md). Three of the four open questions decided and landed:
  `ReserveOutcome::Blocked` replaces the NotBefore(MAX) sentinel;
  probe-opened episodes follow the full confirmation matrix
  (scenarios/core-design clarified, test-pinned); the precedence
  editorial note added. The stale-confirmation-429-joins judgment
  call is confirmed as decided. Sixty offline tests pass; the sole
  remaining open item is the probe/bootstrap seeding seam, which
  blocks the actor slice.
- 2026-08-09 — bootstrap seeding designed, challenged, and accepted:
  revision 1 (name-keyed resolution allow-list) was rejected in
  Tom's review; revision 2 (one global positional bucket constant,
  dynamic seeding, `ProbeReady { policy }`, `RuleScope` deleted) is
  accepted and its §5 is the seeding slice's contract.
- 2026-08-09 — second independent review (Tom-commissioned, external
  frontier model) found six real defects at the predicted
  seams-and-silences, two of them in the audit's own fixes (§3
  external review register). All six fixed same-day, each reproduced
  by a failing test first; three test-evidence fixes landed
  alongside; the composite four-part hand-off now exists
  (`core-handoff.md`). Sixty-eight offline tests pass in debug (66
  in release — drop-bomb tests are debug-only; the 67/65 recorded
  here originally was a miscount); fmt and all-target
  clippy green; focused 4,096-case property runs green. Third data
  point for the post-mortem's thesis: independent review keeps
  catching what green cannot, in whoever's code is newest.
- 2026-08-10 — accepted bootstrap-seeding slice implemented from
  `a3245e86` and presented for Tom's review: explicit engine-level
  positional bucket configuration, dynamic valid-probe policy
  registration, policy-bearing `ProbeReady`, dead `RuleScope`
  removal, the sanctioned `scenarios.md` §1 amendment, and focused
  core tests across success/residue/429/5xx/malformed/repeat seams.
  Gate matrix: 75 debug, 73 release, all nine properties at 4,096
  cases, all-target clippy with warnings denied, and fmt check green.
  Review artifact: `bootstrap-handoff.md`. Slice status remains
  awaiting Tom review; mock and actor work has not started.
- 2026-08-10 — bootstrap-seeding slice review completed per
  `slice-review.md`; no blocking findings. Three minor findings,
  dispositions per Tom (same day): (1) fixed — the probe lane's
  post-seeding `validate_observation_target` refusal, unreachable
  by construction today, now carries the `seeded` mutation bit so
  notification truth survives if a later slice makes it reachable;
  (2) fixed — the seeding `Policy::new` expect and the c2
  empty-rules test now cite each other, pinning the parser's
  ≥1-rule-per-Ok guarantee as a named contract; (3) `core-design.md`
  marked superseded-in-effect for `RuleScope` and the `ProbeReady`
  payload rather than edited, preserving the 2026-08-09 design
  record (reviewer recommendation; reversible if Tom prefers an
  in-place edit). Reviewer independently reproduced the debug (75
  tests), clippy, and fmt gates before and after the fixes; the
  release and 4,096-case gates were not re-run for the comment-level
  changes. Accepted-risk confirmations: cumulative discovery bound
  stays actor-owned; the halt-and-suspend × vacant-seeding cells
  stay confessed-untested. The slice is closed; mock is unblocked.
- 2026-08-10 — mock + M-series harness implementation presented for
  review from baseline `b3a0e7d5` (`4353fb03`, `05ee15d1`,
  `12a799f8`, `4c69f05e`). The
  independent in-process server model now covers B1–B14, typed and
  exact wire/log bounds, N23 topology, deterministic scripts, organic
  and injected failures, source-aware observations, and M1–M13/G1–G6
  judging. Sweep construction structurally includes the shipped
  `Assumed(60s/60s)` configuration. Offline gates: 99 debug, 97
  release, all ten properties at 4,096 cases, clippy with warnings
  denied, fmt, diff check, and four sanitizer tests green. Actor-driven
  M verdicts and the absent §7.4 capture replay remain explicitly
  pending; review artifact: `mock-handoff.md`.
- 2026-08-12 — Tom completed the mock + M-series re-review. The mock now
  reserves a bounded, run-wide handoff identity before arrival delay;
  unavoidable-exposure attribution binds to bounded mock state changes and
  rejects missing, unrelated, post-arrival, post-handoff, or too-late
  claims; the handoff wording now names mock-owned timing rather than the
  removed dispatch samples. Gate matrix re-run: 103 debug, 101 release,
  all ten properties at 4,096 cases, all-target clippy with warnings
  denied, fmt, diff check, and four sanitizer tests green. No
  accepted-not-fixed findings remain; the slice is closed and the actor is
  unblocked.
- 2026-08-12 — Tokio actor shell implemented, reviewed, and closed
  (`d0eabcae`, `e3efb812`, `02b60f47`; review artifact
  `actor-handoff.md`). The shell supplies D5's gate contract — in-flight
  cap 2, HEAD-exclusive with structural writer preference, FIFO among
  ordinary waiters — plus queue drain, cancellation, retry delivery, and
  watch publication, and it carries C3, C4, X1, and X2: fuse burst 10/1 s
  and sustained 500/60 s with exact half-open edges and a non-vacuous
  floor-compliant property, the 4xx tripwire over the same thresholds,
  both X1 fault shapes driven at the real `start_dispatch` hook, and the
  private-field transport boundary. Round-one findings were fixed in
  `e3efb812`; the re-review's findings — post-remap refusals resolving
  through the anchor's current route so scoped failure drains queued
  callers, a genuine same-policy M6 shrink with held history, and bounded
  `WireResponse` as the sole transport-to-actor ingress — were fixed in
  `02b60f47`. Tom's re-review then closed with no findings; no
  accepted-not-fixed findings remain and the slice is closed. Both
  obligations the core slice carried to the actor are discharged: the
  enforced 30 s transport timeout is pinned below the smallest padded N23
  horizon with a test, and the five-endpoint D5 record bound is
  actor-owned and enforced. New doc findings recorded above: the
  actor-owned 10,000-entry queue cap, that same D5 record bound, C4's
  absent independent thresholds (it reuses C3's), and the accepted X2
  limitation that this spike cannot force a future HTTP parser's upstream
  allocation cap — that transport implementation must cap body collection
  itself. Gate matrix re-run at closure: 124 debug, 122 release, all
  eleven properties at 4,096 cases, all-target clippy with warnings
  denied, fmt, diff check, and four sanitizer tests green. The charter
  build order is now complete. Remaining before either verdict slot: full
  M1–M13 scenario-driver runs, driver/judge integration, caller *drop*
  while dispatched, the G3/G4 finalization required by `scenarios.md` §6,
  and the §7.4 capture replay (still blocked on raw input).
- 2026-08-12 — scenario-driver/judge integration landed (`92db9f0b`),
  then its review round one landed the fixes below. Six findings, all
  fixed, none accepted-not-fixed:
  **F1 — the phase sweep was a no-op for twelve of thirteen rows.**
  `(phase_offset_ms + index * 997) % 60_000` folded the offset through a
  per-row modulus, and 59,999 ≡ −1 (mod 60,000), so the two sweeps landed
  1 ms apart (997/996, 1994/1993, …) for every row but M1. The driver,
  its hand-off, and twelve M rows all claimed φ=0/59,999. The phase is
  now passed through verbatim (`SWEPT_PHASES_MS`), so every row genuinely
  runs at both ends of the minute; the corrected sweep is green.
  **F2 — G5 was a hard-coded `true`.** Both `evidence(...)` call sites
  passed a literal, so G5 could not fail on any row while the driver
  reported each scenario's *complete* contract as passed — the exact
  "unrelated passing flag" the `ScenarioAssertionId` doc comment forbids,
  against fragments the coverage confession admits are partial. Each
  match arm now yields its fragment's verdict, and `ContractCoverage`
  (new, in `conformance.rs`) makes the overclaim unrepresentable:
  evidence declares `Fragment` or `FullContract`, `RunReport` carries it,
  and `verdict_eligible()` requires both a pass and full coverage. The
  driver asserts no report is verdict-eligible. Verified live: forcing
  M12's fragment false now reports `G5 failed: ["M12Tripwire"]`.
  **F3 — G3's oracle anchors on measured dispatch times**, so the draft
  ε bounds spacing rather than schedule; recorded as doc finding 12 with
  its measured detection band rather than rewritten, because the anchor
  choice is an input to the §6 G3 finalization, not an implementation
  detail.
  **F4 — M10's assertion was `observations.len() >= 2`** with every
  caller outcome discarded (`let _ = ticket.await`), while the run
  actually produces 16 observations; a caller drop that wedged the queue
  passed. Now pinned exactly: 14 surviving callers served, 16 wire
  observations.
  **F5 — the M10 register entry was misfiled** as item 12 under
  "Resolved in code (with tests)" in the 2026-08-09 audit register though
  it is open and undated there, and its three cross-references disagreed
  (row said 11, item said 12, hand-off said 12). It is now doc finding 11
  with an explicit open marker, and every reference agrees.
  **F6 — `AGENTS.md`'s hand-off table omitted `scenario-driver-handoff.md`**,
  so a fresh session would have read the actor hand-off as newest and
  missed this slice's live coverage confession — the failure mode
  `slice-review.md` §5 rule 3 exists to stop. Table updated.
  Gate matrix re-run after the fixes: `cargo test --locked` (126 debug,
  124 release), all-target clippy with warnings denied, `fmt --check`, and
  `git diff --check` green. `ContractCoverage` carries its own non-vacuous
  test (`conformance_harness::a_fragment_run_is_judged_but_is_never_verdict_eligible`):
  both branches assert, and a failing fragment still fails G5. The slice remains **open pending re-review**;
  no verdict slot is filled and doc findings 11 and 12 stay open.
- 2026-08-12 — scenario-driver review round **two**. One blocking finding,
  fixed in the commit carrying this entry; the fragment/verdict guard and
  the G5 failure path from round one were re-reviewed and held.
  **F7 — the phase sweep was still effectively adjacent.** Round one fixed
  the *arithmetic* that collapsed the sweep (F1) but kept the phase
  *values* `[0, 59_999]` on a wrong model of what `phase_ms` means. The
  mock's `bucket_end` treats `phase_ms` as the **upcoming** boundary, not
  an offset already elapsed: at t₀, φ=59,999 puts the 60s edge 59,999ms
  out versus 60,000ms for φ=0, and the 5s edge at 4,999ms versus 5,000ms.
  The two sweeps therefore still differed by 1ms — the same coverage
  defect as F1, re-created one layer up. The driver's own comment asserted
  the opposite of what the code does, and nothing could contradict it.
  Fixed by sweeping `[0, 1]`, which are the extremes of boundary distance
  (a full bucket away versus 1ms out) in *both* N23 bucket sizes.
  Root cause and the structural fix: the phase representation was only
  knowable by reading a private function, so two successive readings of it
  went unchecked. `mock::model::first_bucket_boundary_ms` now exposes the
  distance, and `scenario_driver::swept_phases_are_separated_by_a_full_bucket`
  pins all four literal distances (5,000 / 60,000 at φ=0; 1 / 1 at φ=1),
  pins the trap itself (φ=59,999 → 59,999ms), and asserts over whatever
  `SWEPT_PHASES_MS` holds that the phases move the first boundary by nearly
  a whole bucket in both sizes. Verified by reverting the constant: the
  guard fails with `phases must differ by nearly a whole 5000 ms bucket,
  got 5000 vs 4999`. A third misreading now fails a test rather than a
  review. The M rows above and the hand-off are corrected to φ=0/1.
  Gate matrix: 127 debug, 125 release, all-target clippy with warnings
  denied, fmt, and `git diff --check` green. The M-series is green at the
  corrected phases, so F7 — like F1 — was a defect in the evidence's claim,
  not in the actor. Slice stays **open pending re-review**; no verdict slot
  filled, doc findings 11 and 12 still open.
- 2026-08-12 — **Tom's decision on M10 and reprioritization.** Doc finding
  11 was raised as "M10 requires reprioritization, the actor has no reorder
  command." Reading the sources together showed the finding was miscast and
  the docs were in conflict: `scenarios.md` M10 lists reprioritization among
  its *stimuli* and never among its *asserts* (§6's G5 names
  "M10 drain-to-completion"), while `design-brief.md` — the charter — scopes
  it out of the spike outright: "the spike builds **none** of
  display/reorder/edit." Three actions, all decided by Tom:
  (1) **M10 amended** to drop reprioritization from its stimulus list, with
  the struck text preserved and the reasoning dated in `scenarios.md`. No
  assertion was lost; every M10 assert is order-independent, and enqueue
  pressure plus cancellation agitate the same invariants.
  (2) **Doc finding 11 re-scoped** to what actually remains: the run is 16
  requests against a stated hundreds-over-many-minutes, which is why M10's
  most distinctive assert — the fuse's false-positive absence under
  saturation — is still untested. C3's sustained fuse is 500/60s against a
  compliant ≤240 req/min, so that ~2× headroom is only demonstrated by
  occupying it. No new API needed; this is the row's real gap.
  (3) **CN6 minted** in §4: reprioritization is cheap in the actor shape and
  was not in the coroutine/facade shape, so `network-redesign.md` R7/D6's
  "not cheap later" warning does not carry over. It failed there because the
  per-update stop token gave no per-entry identity; the Rust actor has that
  identity already, its dispatch loop reads only `queue.front()`, and it
  already dispatches out of arrival order for writer preference — FIFO is
  emergent, not assumed. The expensive part of reorder is contract, not
  code: D5's "no lane starvation" clause has no rule. CN6 carries a tripwire
  (per-policy queue lanes would invalidate the single-deque premise and are
  the moment to revisit) and a portability note, since a claim about a
  comparison needs the superseded document to stay legible.
  Rationale for deferring rather than building: the starvation rule is a
  product decision with no product requirement yet to shape it, and the
  primitive that made deferral costly in C++ is already present here. The
  pain-saving action was recording *why* it is cheap, not building it — an
  inherited false warning was the actual risk. No code changed; gates re-run
  unchanged (127 debug, 125 release, clippy, fmt, diff check green).
- 2026-08-12 — **M10 scaled; G3 gained permit-availability arithmetic;
  epsilon still cannot be finalized.** Closing doc finding 11 and refining
  finding 12, after Tom's call to prefer the smallest instrument that keeps
  the row honest.
  **Finding 11 resolved.** M10 now runs 300 enqueues against
  `backend-item-request-limit` (Account 30/60s and 100/1800s) with 30
  cancellations spread through the queue rather than taken off one end, plus
  a caller dropped while dispatched. **Superseded for current coverage by
  F8/F10 below:** M10 now exercises explicit dispatched cancellation instead;
  a dropped dispatched ticket is not currently covered. The policy, not the
  driver, sets the duration: the run spans 3,963,500 ms ≈ 66 simulated minutes *(figure
  corrected 2026-08-12; originally 3,963,250 ms, measured before the 250→25 ms
  step change — round-four owed list)* across many
  window rollovers. All five of M10's stated asserts are read directly off
  mock-owned wire evidence or the actor's published status — 270/270 served,
  30/30 cancelled callers resolved as `Cancelled` (an assert the previous
  driver omitted entirely), fuse quiet, in-flight ≤ 2, and the spacing floor
  never violated, checked against D5's literal 250 ms rather than the
  actor's own constant. The wire count is bounded two-sided instead of
  pinned, because a cancel under pressure may or may not beat its dispatch
  and one did; `served` is pinned exactly so a wedge cannot hide in the band.
  **A planning error worth recording.** The first plan was to extend G4 to
  M10. Reading §6 showed G4's text is M2-scoped and G3, not G4, names M10 a
  binding stress row; M10's own asserts contain no duration bound at all. A
  1.05× ratio over a coarse padded minimum would also have produced spurious
  failures at this scale, since bucket quantization dominates the term. The
  instrument M10 actually needed was none — its asserts are all directly
  observable. That is the cheaper answer, and it was in the docs the whole
  time.
  **Finding 12(a) resolved.** Scaling did break G3, exactly as the row-level
  arithmetic predicted: the oracle assumed no policy debt, so every
  legitimate window wait read as a violation (112,250 ms on the 30/60s
  window, 1,497,250 ms on the 100/1800s). §6 names permit availability as
  part of the padded-safe time and the mock's observation log as a
  client-independent source, so the oracle now derives it from both. It
  mirrors the mock's counting predicate rather than the client's: a hit
  stays active until `bucket_end(at) + period`, so hit *k* is permitted once
  hit *k−H* has aged out. `MockController::definition` and
  `mock::model::bucket_end_ms` were exposed so the oracle reads the server's
  own windows and quantization instead of a hardcoded copy that could drift.
  One off-by-one was found and fixed by reading the mock rather than
  guessing: HEADs are not policy hits (`counted = method != HEAD &&
  !layer1.tripped`), and counting them overstated debt by one, reporting the
  boundary request as dispatched-before-eligible.
  **Finding 12(b) opened, and it blocks the epsilon decision.** With the
  debt term in place, measured lateness was *exactly* 500 ms — equal to
  epsilon — which looked like a gate doing real work. It is not. Re-running
  at 25 ms harness resolution splits the rows: the eight whose submissions
  are contemporaneous with eligibility drop to 25–50 ms, while
  M5/M6/M7/M9/M10 stay at exactly 500 ms. M5 shows why: its third GET
  dispatched at 1,250 ms against an oracle expectation of 750 ms, having
  been submitted at ≈1,250 ms. The actor dispatched it immediately; the
  oracle simply does not know when a request was queued, though §6's wording
  ("whenever a request is *queued* and eligible") makes queueing part of the
  definition. So the 500 ms maxima are an artifact of the instrument, the
  current data justifies neither keeping nor tightening epsilon, and the
  clean rows suggest the true bound is well under 50 ms. Recorded rather
  than fixed, because closing it means giving the oracle submission instants
  and that is a scope decision for Tom.
  Gate matrix: 127 debug, 125 release, all-target clippy with warnings
  denied, fmt, and `git diff --check` green. Slice remains **open pending
  re-review**; no verdict slot filled.
- 2026-08-12 — **G3 epsilon decomposed; finding 12(b) closed, 12(c) opened.**
  Tom's call was to close 12(b) if it cost the system nothing; it cost
  nothing, and the measurement it unlocked reframed the epsilon question.
  **12(b) resolved, after a wrong turn worth recording.** The first attempt
  keyed submission instants by `ticket.id()`, on the assumption that the
  actor stamps its `RequestId` into the correlation header. It does not:
  `RequestId` and the wire correlation are independent counters, because
  `start_probe` allocates a fresh correlation per *dispatch* (probes and
  retries included). The test caught it immediately — M5 flipped to
  "dispatched before independent eligibility". A second attempt used §6's
  authorized-exclusion mechanism, which also failed, for an instructive
  reason: the delay interval *straddles* the submission (M5's second GET was
  submitted at 1,000 ms and dispatched at 1,250 ms), so no quiet window
  bounded by the submission can contain it. What works needs no per-request
  map at all: raise each observation's eligibility to the latest script
  submission instant at or before its dispatch. A request is never expected
  before the script asked for one.
  **Harness step 250ms → 25ms.** The step is the floor under every G3
  measurement, and at 25ms the whole target — M10's 300-request, 66-minute
  run included — costs ~2s. There was no tradeoff to make.
  **12(c) opened: epsilon is not what it appears to be.** With the oracle
  correct, lateness decomposes cleanly. Away from window rollovers it is
  **25 ms** (M8: 50 ms) — one tick, i.e. the measurement floor, so the actor
  is tight enough that the harness cannot resolve it. At M10's window
  rollovers it is **275 ms and 475 ms**, and that is the client's N13
  pessimism padding: §6 asks G3 to measure against the *padded-safe* time,
  while the debt term computes the *server's* raw permit instant, so
  deliberate padding reads as lateness. Measured both directions: at
  epsilon = 100 ms an **unmutated** actor fails M10 on padding alone, and the
  same epsilon **catches** the 250→600 ms floor regression that epsilon =
  500 ms lets through — round one's demonstrated hole. So the draft
  epsilon = 500 ms is not slop tolerance; it is an allowance for padding the
  oracle does not model, and the two cannot be separated without modelling
  it. The §6 choice is now concrete: model N13 padding and tighten epsilon to
  roughly 100 ms, gaining a G3 that catches pacing regressions; or keep
  epsilon near 500 ms and record that G3 cannot discriminate below the
  padding envelope. Not decided here — it is a gate constant, and the docs
  make it Tom's.
  Gate matrix: 127 debug, 125 release, all-target clippy with warnings
  denied, fmt, and `git diff --check` green. Slice remains **open pending
  re-review**; no verdict slot filled.
- 2026-08-12 — scenario-driver re-review follow-up: **F9 fixed; F8
  recorded open.** G3's permit-availability oracle no longer calls the
  mock's production `bucket_end` helper. It now uses local bounded integer
  arithmetic, with explicit just-before, exact-boundary, and just-after
  tests, so a mock quantization defect can disagree with the expected-value
  oracle. **F8:** M10's scaled run established eventual cancellation only;
  its four-hour advance precedes awaiting cancelled callers, while M10 says
  “prompt” and supplies no deadline. Recorded as doc finding 13 for Tom's
  decision; no promptness claim is made. The slice remains **open pending
  re-review**.
- 2026-08-12 — **F8–F10 re-review findings fixed; awaiting re-review.**
  **F8:** M10 now first proves the delayed first GET is dispatched from the
  mock's handoff record and published active count, cancels that caller, and
  advances exactly one 25ms tick. It does the same for the 30 cancellations
  distributed through the later queue. Each ticket is then single-polled;
  `Poll::Pending` fails immediately, so an await or a later timer cannot
  manufacture promptness. After the long run, published in-flight state must
  be zero, proving the cancelled dispatched caller's delayed response still
  reconciled. **F9:** the G3 expiry oracle is now a separately structured
  half-open-interval reference model: a `[0, phase)` prefix followed by
  whole width intervals. Its phase-zero and phase-one before/on/after pins
  are independent of the mock's branch/index implementation. Gate matrix
  re-run: `cargo test --locked` (128 debug), `cargo test --locked --release`
  (126 release), all-target clippy with warnings denied, fmt, and `git diff
  --check` green. **F10:** the hand-off's stale claim that M10 still drops a
  dispatched caller was removed. M10 now covers dispatched cancellation;
  neither its driver nor a focused actor-shell test covers a dropped
  dispatched `RequestTicket`, which is stated as an omitted coverage cell.
  The slice remains **open pending re-review**; no verdict slot is filled.
- 2026-08-12 — scenario-driver review round **four**: **the fuse headroom
  finding was mislocated twice, and C3's property is strengthened.** No
  driver code changed.
  **What was claimed.** Doc finding 11's re-scope (`86c60f94`, 15:58) said
  M10's fuse false-positive assert "was untested" and that the ~2× headroom
  between the spacing-implied 240/min and the 500/60s clause "is only
  demonstrated by occupying it" — implying M10 needed a saturation lane.
  The round-four review then re-filed the same finding, having measured
  M10's real dispatch rate (peak **31 per 60 s**, 4 per 1 s, over 273
  dispatches spanning 3,963,500 ms) and concluded nothing exercised the
  headroom.
  **Both were wrong, for the same reason.** `scenarios.md` C3 assigns the
  property — "never trips on any floor-compliant trace" — to **C3**, and
  carries the headroom in C3's own derivation ("the clause sits at 2× the
  legitimate maximum"). M10's clause is the *integration instance*: that the
  trace the actor actually emits under caller pressure is floor-compliant,
  which its `paced` check measures directly over the whole wire log, with
  `fuse_quiet` observed alongside. C3 ⊗ M10 ⊗ X1 discharge the clause with
  nothing left to build. And C3's property had already landed in `e3efb812`
  at 13:03 — **three hours before** the re-scope that called it untested —
  reaching steady state at exactly 240/min. The C3 row of the same §3 table
  said so, 130 lines above the finding. The reviewer's repeat was a `tests/`
  search that missed a unit test inside `src/actor.rs`.
  **Process rule added**, since this is a failure mode the guide did not
  name: `slice-review.md` §1 lesson 4 and §3 step 4 — evidence rows are a
  seam like slice boundaries are, and before recording that something is
  untested, look up which scenario or property `scenarios.md` makes
  responsible for it and read that row.
  **C3's property strengthened** so its row's wording is literally true.
  It previously varied only trace *length* over a single uniform cadence,
  which reached the steady state by construction but tested one shape and
  stopped at ~75 s. Now split in two: a deterministic cadence pin (1,500
  dispatches at the floor, 6.25 simulated minutes) asserting the trailing
  peaks equal exactly 4 and 240 — the equalities are its reachability guard,
  and they are the headroom claim stated as a measurement; and the property
  proper, over generated irregular gaps (750 dispatches, ≥3.1 simulated
  minutes) so the sustained clause prunes across several windows instead of
  filling one. Both verified by mutation: shortening the cadence trace to
  100 fails with `left: 100, right: 240`, and putting the generated gaps 1 ms
  under the floor fails the oracle's `burst <= 4` premise — the fuse itself
  correctly stays quiet there, which is why the premise, not the clause, is
  what has to catch it.
  Gate matrix: `cargo test --locked` (129 debug), `cargo test --locked
  --release` (127 release), `PROPTEST_CASES=4096 cargo test --locked --lib
  c3_floor_compliant` green (21 s), all-target clippy with warnings denied,
  `cargo fmt --all --check`, and `git diff --check` green.
  **Still open and owed by this round:**
  finding 11's resolution text still reads as though scaling M10 discharged
  the fuse assert (it did not — C3 did) and still describes M10 as dropping a
  dispatched caller, superseded by F10; the §3 M10 row should cite the
  composition; `AGENTS.md`'s hand-off table still says "rounds one and two";
  the recorded M10 span 3,963,250 ms measures 3,963,500 ms since the 250→25 ms
  step change; and review findings F14–F16 (driver twin-guard, duplicated
  floor literal, mirror fallbacks) are unaddressed. The slice remains **open
  pending re-review**; no verdict slot is filled.
- 2026-08-12 — clause-registry slice landed (design accepted at
  `ce5730d4`; kickoff `kickoff-registry-slice.md`). `obligation-map.md`
  migrated row-for-row into `src/obligations.rs` — `CLAUSES`, 122
  entries, and `OPEN_UNTESTED`, 13 ids — verified by
  `tests/obligations.rs` (id uniqueness, owner vocabulary,
  coverage/citation-count consistency with the negative rule on
  exclusions, source-existence of every cited fn including the
  `src/` unit modules and proptest-macro properties, Untested
  disposition strings, exact `OPEN_UNTESTED` match, owner-series
  reachability). 122 = 125 map rows − 2 collapses (the M5 table's
  U1 pointer row and the M12 table's U2 pointer row are the U-register
  entries respelled) − 1 omission (the unowned dropped-dispatched-
  ticket clause: design §7 item 2 remained undecided, so it is
  carried as a hand-off finding, not an entry). Coverage classes
  transcribed from the audit unchanged; one transcription finding
  recorded in the table itself (`c4-halt-semantics-shared` was
  "partial" in the map on a code citation only — the registry cites
  the nearest real test and records the gap in its note). All six
  design-§5 mutation checks demonstrated to fail on their intended
  rule (removed citation, nonexistent fn, misspelled file, citation
  on an exclusion, `OPEN_UNTESTED` desync, duplicate id). The map is
  marked superseded-by-registry, dated text preserved. Kickoff
  commit 1 added the finding-ID namespaces subsection to
  `slice-review.md` §5. Gate matrix: `cargo test --locked` 135
  debug / 133 release (the 129/127 baseline plus 6 registry tests),
  `PROPTEST_CASES=4096 cargo test --locked` green (135),
  `cargo clippy --locked --all-targets -- -D warnings`,
  `cargo fmt --check`, and `git diff --check` green. The slice is
  **open pending Tom's review**; four-part hand-off in
  `registry-handoff.md`.
- 2026-08-12 — **clause-registry review round one closed (REG-R1 — first
  round under the new finding-ID namespace convention).** Before Tom's
  review, the obligation-map audit session re-performed the acceptance
  reconciliation independently of the implementing session: the map's
  125 rows confirmed by mechanical count; 122 = 125 − 2 − 1 verified;
  aggregate coverage classes 63/33/14/12 matched; per-section class
  distributions **exact across all 27 groups** (the check aggregates
  cannot carry — no silent swap anywhere); cross-ownership tallies
  reconciled (M8 = 10, C2 = 6, C3 = 5 incl. the M10 fuse row, B8/B10/
  B13 absorbing their M-section mock rows); `OPEN_UNTESTED`'s 13 ids
  equal to the set predicted from the map before reading the registry;
  ~16 entries deep-read including every high-risk row, all faithful.
  Two mutation checks were replayed on **fresh instances** (an
  `OPEN_UNTESTED` desync via `m9-headroom-record`; a nonexistent-fn
  citation on the M13 FIFO test) — each failed on exactly its intended
  rule; the restored tree ran 135 debug tests green. Tom then closed
  the round with **no blocking findings**. Four observations,
  dispositions Tom's, 2026-08-12:
  **REG-R1-F1** — the map's two `—` rows entered the registry as
  `Untested` (`m7-threshold-tuning` open, `x2-parser-cap-limitation`
  accepted); correct, since `Excluded` is structurally U-/O-only, but
  the transformation was disclosed only through the tallies — recorded
  here so it is explicit.
  **REG-R1-F2** — `m7-threshold-tuning` sits in `OPEN_UNTESTED` as an
  ambiguity (map §8.5 item 3), not test debt; it may resolve by a
  `scenarios.md` wording decision rather than a test. The open list is
  13 ids, not thirteen owed tests.
  **REG-R1-F3** — the hand-off's judgment-call-4 candidate collapse
  (`m1-probe-429-tripwire-feed` + `m12-tripwire-feed`) is **declined**:
  the rows are the two distinct feed call sites (`finish_probe` /
  `finish_ordinary`), and one merged clause could read as discharged
  after testing only one lane. Two rows keep one line per lane.
  **REG-R1-F4** — `c4-halt-semantics-shared` stays `Partial` as
  migrated, with the map's code-not-test citation mismatch recorded in
  its note (migration finding 1); reclassification is deferred until
  the latch/feed tests are sequenced.
  The slice is **closed**; `src/obligations.rs` + `tests/obligations.rs`
  are now the live coverage authority, and the doc-split slice
  (`kickoff-doc-split.md`) is unblocked. No code changed in this
  closure; the review's own suite run (135 debug, green) is the gate
  evidence, per the bootstrap-closure precedent for comment-level
  changes.
- 2026-08-12 — **live/history doc split presented for review**
  (`kickoff-doc-split.md`; commits `77aee08` corrections, `087dc56`
  the split, plus the hand-off commit carrying this entry). Commit 1
  paid the round-four owed list in full: doc finding 11's fuse text
  corrected to the C3 ⊗ M10 ⊗ X1 composition with the original
  preserved under a dated marker, and its dropped-dispatched-caller
  wording corrected per F10; the §3 M10 row cites the composition;
  the M10 span corrected to the measured 3,963,500 ms in both places;
  `AGENTS.md`'s driver row and `scenario-driver-handoff.md`'s status
  line brought current, naming F14–F16 as recorded and unaddressed
  (they remain the driver slice's debt, deliberately untouched).
  Commit 2 created `status.md` — the single live-state file: coverage
  truth (the registry + `OPEN_UNTESTED`), slice/review state linking
  the one live confession, six open decisions for Tom with register
  pointers, the blocked §7.4 replay, and next work — and re-pointed
  both authorities in the same commit: `AGENTS.md`'s read order now
  leads with `status.md` and its hand-off table keeps the chain but
  drops the status column; `slice-review.md` §5's status-flip act now
  updates `status.md` first and adds the historical marker on slice
  closure; the five closed hand-offs each carry that marker; this §3
  gained its deltas-are-historical preamble sentence. The gate-summary
  partial-evidence marker (audit §8.2 item 6) is **deferred to Tom**
  (`status.md` §3 item 6) because it edits register cells. The
  naive-reader probe ran twice on cold sessions (the kickoff's
  verbatim prompt plus a rephrasing): both matched `status.md` on
  open/next/blocked/coverage and cited no superseded source as live;
  verbatim transcripts in `doc-split-handoff.md`, including probe 1's
  honest catch of the not-yet-written hand-off file itself. Gate
  matrix re-run (docs-only): `cargo test --locked` 135 debug / 133
  release, `PROPTEST_CASES=4096` green (135), all-target clippy with
  warnings denied, `cargo fmt --all --check`, and `git diff --check`
  green — identical to the REG-R1 closure matrix. No dated paragraph
  was rewritten; no register cell, verdict slot, or `scenarios.md`
  text was touched. The slice is **open pending Tom's review**;
  four-part hand-off in `doc-split-handoff.md`.
- 2026-08-13 — **round-four findings F14–F16: substance recorded
  (scenario-driver register).** Round four filed the three findings
  as three-word glosses; their substance was written down nowhere
  (that lesson is DS-R1's, below). A targeted re-review of
  `tests/scenario_driver.rs` re-derived all three; verdicts and
  proposed fixes recorded here, code untouched — the fixes remain
  the driver slice's owed work.
  **F14 — driver twin-guard: real, two instances.** (a)
  `run_m8_oauth_lane` (`tests/scenario_driver.rs:632–660`) is a
  second copy of the M8 matrix arm (`:372–397`) whose assertion
  dropped the `GET count == 2` conjunct (`:653` asserts `retried`
  alone), and its report is never pushed into `reports`, so it
  bypasses both post-loop guards at `:613–629`, including the
  `!verdict_eligible()` coverage guard. Not a false green today —
  correlation numbering makes the 429 script apply in both lanes —
  but the lane would not notice if it stopped applying. Fix: one
  shared M8 helper returning the full conjunction, OAuth report
  routed through the same guards. (b) The D5 in-flight-cap guard is
  duplicated verbatim in the M10 and M13 arms (`:492–493`,
  `:578–582`), each hard-coding `2` rather than
  `conformance::D5_IN_FLIGHT_CAP`. Fix: one shared helper on the
  constant.
  **F15 — duplicated floor literal: real, unambiguous.** The D5
  send floor exists as `MIN_SEND_SPACING_MS = 250` (`:678`, used at
  `:504` and `:871`) and as a bare `250` in the oracle's HEAD branch
  (`:859`). The floor covers HEADs by contract, so the copies cannot
  legally diverge — the second can only ever silently disagree with
  the first (a floor mutation would leave the HEAD branch at 250,
  weakening the oracle exactly where N2's incident was a HEAD
  flood). Fix: use the constant at `:859`.
  **F16 — mirror fallbacks: structurally real, currently
  unreachable.** `independently_eligible_ms`
  (`tests/scenario_driver.rs:148–153`) falls back to
  `observation.dispatch_ms` for a correlation missing from the
  oracle map; that value satisfies both G3 comparisons by
  construction (`src/conformance.rs:647–660`), so any unkeyed
  observation would be silently exempt from G3 — a fail-open default
  under the gate that catches trivially-safe-by-being-slow. Today
  the oracle map is exhaustive over the judged set by construction
  (built from the same observation slice that is judged; correlation
  ids unique run-wide), so **G3 is not currently vacuous** and the
  fallback is dead code — but any future judging of a superset (a
  re-fetched log, a second `judge` call) would disarm G3 with no
  test noticing. Fix: `.unwrap_or(u64::MAX)` so an unkeyed
  observation fails loud as dispatched-before-eligible.
  Net: none of the three changes a gate result on current code —
  consistent with round four having changed no driver code; F14(a)
  has the most teeth (a live path missing an assertion conjunct and
  a coverage guard).
- 2026-08-13 — **doc-split review round one filed and fixed
  (DS-R1).** Tom commissioned an external consistency audit of the
  spike (five independent document/code readers plus a fresh test
  run) and adopted its findings as this slice's review round rather
  than new work. Fix policy per Tom's planning session: prefer
  removal or a pointer over a fresher copy wherever a finding was a
  drifted duplicate; dated history takes markers, never rewrites.
  Six findings against `status.md` itself — the file whose
  precedence rule makes its errors the costliest:
  **DS-R1-F1** — §2 said "doc findings 11–13 are fixed"; 12(c) is
  open and is §3 item 1 (the G3-epsilon decision). Fixed to match
  the driver hand-off: "11, 12(a), 12(b), and 13".
  **DS-R1-F2** — §2 called the driver hand-off "the one open
  hand-off" while listing a second open hand-off two lines later.
  Fixed by removal.
  **DS-R1-F3** — §3 item 6 quoted the kickoff's marker wording under
  the audit's citation; audit §8.2 item 6 actually proposes
  "partial — fragment evidence, see driver status note". Fixed: the
  item now carries both candidate wordings, correctly attributed;
  the choice stays Tom's.
  **DS-R1-F4** — §1 claimed "one entry per obligation-map row";
  `CLAUSES` has 122 entries against 125 map rows (two pointer-row
  collapses, one deliberate omission — `registry-handoff.md` §5).
  Fixed: §1 states the arithmetic.
  **DS-R1-F5** — §4 cited `result-draft.md` §5 (an empty slot
  template) for the capture-replay block; the block is mock-slice
  doc finding 8 (§3 register), and §4 also dropped the
  already-compliant-fixture escape hatch both sources state. Fixed:
  citation corrected, hatch restored.
  **DS-R1-F6** — §1 forbade reading the map for current coverage
  while §5 item 2 pointed next work at map §8.2, a list containing
  two since-discharged items and missing M12's narrowed delta
  (which lives in §8.1 item 1). Fixed: §5 frames the §8 lists as
  dated analysis and names both subsections.
  Stale-sweep, same round: two `src/obligations.rs` notes still
  claimed the debt `77aee08` paid (m10 fuse composition, m10 span) —
  notes updated (the only code-file edits, string literals in
  `note:` fields); the §3 "driver integration status" paragraph —
  the one live-reading paragraph the split missed ("two review
  rounds", "findings 11 and 12 remain open") — marked superseded in
  part; the orphaned seeding-seam bullet after "no open items"
  marked resolved; the core→actor timeout obligation and the
  "actor is still unbuilt" sentence marked discharged/dated; both
  kickoffs got Executed banners; `AGENTS.md`'s core-design entry
  brought to the in-effect tense and its confession rule pluralized
  for two open slices; `core-handoff.md`'s second confession
  paragraph marked dated (71/73 counts) and its missing closure
  record flagged; `bootstrap-handoff.md`'s §1/§3 deferrals marked
  built; `registry-handoff.md`'s drift-window and readiness
  paragraphs marked; `clause-registry-design.md` marked
  executed with §6 outcome notes; `design-brief.md`'s "nothing here
  is decided" scoped to its writing date; `core-design.md` §5's
  "pinned by X2's structural test" corrected to design intent (the
  test is unbuilt, `OPEN_UNTESTED`); the map's banner gained the
  `status.md` pointer, §1–§7 gained superseded-snapshot reminders,
  and its three discharged §8 items (8.2/1, 8.2/7, 8.3/7) plus the
  M10 table-cell note gained resolution markers; the F11–F13
  numbering gap recorded in `slice-review.md` §5. Newly tracked in
  `status.md`: the registry payoff-wiring decision (§3 item 7, from
  design §6) and the declined-collapse drift window (§5 item 2).
  Accepted-not-fixed, dispositions Tom's at closure: the
  gate-summary marker choice (§3 item 6) and the core
  closure-record gap. F14–F16 substance is the entry above; their
  code fixes stay the driver slice's debt — no driver code changed.
  `scenarios.md` untouched in the round's own commit per its
  authorship rule: two dated addenda (the M10 amendment's stale
  "real remaining gap" clause; §6's pre-implementation G3/G4 revisit
  notes) were drafted for Tom in the review conversation. Tom
  approved his reworded versions the same day; they are applied, as
  his amendments, in the follow-up commit.
  Gate matrix re-run 2026-08-13: `cargo test --locked` 135 debug /
  133 release, `PROPTEST_CASES=4096` green (135), all-target clippy
  with warnings denied, `cargo fmt --all --check`, and
  `git diff --check` green. The slice remains **open pending Tom's
  closure review**.
- 2026-08-13 — **doc-split review round one closed (DS-R1); the
  slice is closed.** Tom delegated the two open dispositions with a
  simplicity mandate (the system must come out simpler to
  understand, not just correct). Gate-summary marker: the six-cell
  edit is **declined**; the marker's content lands once as a
  preamble line above the §3 gate-summary table ("all rows unfilled
  pending full contracts; fragment-level gate evidence green at
  φ=0/1 — see the driver status note") — the kickoff's own
  say-it-once precedent, with "unfilled pending" chosen over the
  audit's "partial" because a partial-looking value in a verdict
  table could be read as a graded verdict, which fragments must
  never produce. Core closure-record gap: **accepted as-is** — the
  chain and the `core-handoff.md` marker are the record; no
  retroactive closure entry is manufactured. Three acts done:
  `status.md` §2 flipped (doc split joins the closed list;
  scenario-driver is again the only open slice, and §3 drops the
  resolved marker decision, renumbering payoff wiring to item 6),
  the hand-off status line flipped with the historical marker, this
  entry written. The approved `scenarios.md` addenda landed at
  `d953aeba` (previous entry). Gate evidence: `cargo test --locked`
  135 debug green, `git diff --check` clean (docs-only closure, per
  the comment-level precedent). Open work unchanged: F14–F16,
  fragment raising, G3/G4 finalization (decision 1), §7.4 replay,
  verdict slots last.
- 2026-08-13 — **decisions pass (Tom, working with the DS-R1
  auditor): all six standing decisions resolved; `status.md` §3 is
  empty.** Each disposition, in the day's order:
  **(1) G3 ε final at 500 ms** (doc finding 12(c) closed above): the
  oracle stays padding-independent rather than mirroring the padding
  model; ε bounds only the too-late direction. Known consequence
  recorded with the decision: a floor regression escapes G3, and the
  accepted mitigation is G4's harness-computed minimum. The §6
  finalization amendment is applied and the G3/G4 gate-summary
  statement cells carry the final values (500 ms, 1.05×) — verdict
  slots now wait on nothing but full-contract runs.
  **(2) X2 spike-scope test defined**: a structure pin — transport
  handle private to the actor, a single send call site, compile-fail
  on outside construction — with a recorded obligation that
  production integration re-pins X2 when a real HTTP client exists
  (clause note). The pin itself is owed work, next-work item 3.
  **(3) Dropped dispatched `RequestTicket` adopted**:
  `shell-dropped-dispatched-ticket` (owner `SHELL`, design §7 item 2
  as proposed) added to `CLAUSES` and `OPEN_UNTESTED`; the
  registry-handoff finding is discharged and no obligation lives
  solely in prose confessions.
  **(4) All §8.5 ambiguity flags resolved** as dated `scenarios.md`
  amendments in Tom's wording: G2 owns "never trips the ceiling"
  with M11a as its named binding evidence; the M7 threshold-tuning
  sentence **deleted outright** — it named no testable obligation —
  and `m7-threshold-tuning` removed from the registry
  (resolved-by-wording, as REG-R1-F2 predicted); N25 1:1 tracking is
  B4's fidelity obligation, no client-side instrument; G4's
  "harness-computed" means runtime-derived from the scenario's own
  policy and queue depth, precomputed literals do not qualify;
  M1's "strictly serialized" binds per endpoint with global HEAD
  exclusivity M13's; C3's sustained trip pinned **exactly 500**.
  The drop-don't-carry test was applied to all six: only M7's
  sentence failed it.
  **(5) REG-R1-F4 resolved: `c4-halt-semantics-shared` demoted to
  `Untested`** — the migrated Partial was borrowed evidence (a code
  citation plus a test proving a different clause's property); the
  owed wire-4xx halt/drain/publish test batches with the latch/feed
  tests.
  **(6) Registry payoff wiring declined as designed** — deriving run
  eligibility from the registry is circular, since clauses become
  `Full` because full-contract runs land, not the reverse. The
  driver keeps its per-run declarations as the primary source;
  replaced by the two-authority slot-fill rule now standing in
  `AGENTS.md`: a verdict slot fills only when the run's declaration
  and the registry agree.
  Registry after the pass: **122 clauses, `OPEN_UNTESTED` = 14 ids,
  every one a genuinely owed test** (the +shell/−m7/+c4 arithmetic
  against the migrated 122/13). Simplicity outcome: everything open
  in the spike is now plain owed work — no standing decisions, no
  ambiguity ids, no prose-only obligations. Gate matrix: `cargo
  test --locked` 135 debug / 133 release, `PROPTEST_CASES=4096`
  green (135), all-target clippy with warnings denied,
  `cargo fmt --all --check`, and `git diff --check` green.
- 2026-08-13 — **coverage ballot pass (Tom, with the auditor):
  every non-Full registry clause balloted and marked.** One row per
  clause: the 14 `OPEN_UNTESTED` ids plus the 32 `Partial` entries
  (46 rows; counts machine-verified against `src/obligations.rs`).
  Tom marked all rows as recommended: **38 BLOCK, 7 AMEND,
  1 DEMOTE**. The 38 BLOCKs collapse onto the lettered work items
  A–I plus five singles and the full-contract run — `status.md` §5
  is rewritten as that closed list (entries leave by discharge or
  approved amendment; new obligations require a failing test or a
  §3 decision). Complexity constraint adopted with the ballot: the
  list is final scope for the spike.
  **Applied immediately** (settled decisions, no new wording):
  `m2-state-tracks-post-increment` re-owned M2→B4 and flipped
  Partial→Full (decisions-pass amendment, `scenarios.md:209` — the
  registry had never been flipped to match);
  `m11-compliant-never-trips` re-owned M11→G2, stays Partial with
  the M11a sweep as G2's named binding evidence
  (`scenarios.md:369`); stale-note sweep applying the decisions
  pass to six registry strings (ε-final in the G3 note, the
  m2-g3-g4-bounds note and its must_assert, "harness-computed" in
  the G4 note, per-endpoint boot in the M1 note, exact-500 in the
  C3 sustained text).
  **Drafted, pending Tom's wording approval (`status.md` §3
  item 1)** — on approval six rows flip Partial→Full by composition
  and `m9-headroom-record` flips to Excluded, `OPEN_UNTESTED` 14→13:
  **(a) M4 row** (covers `m4-scoped-clean-failure`,
  `m4-at-most-one-request`, `m4-pending-errored`): "The one-triplet
  branch is the actor-exercised representative of the
  unexpected-shape family: C2 types both shapes
  (`rejects_non_pair_shapes`) and both cross the wire as raw headers
  (`b1_b7_m4_synthetic_one_and_three_window_policies_cross_as_raw_headers`);
  the actor consumes the same typed parse error for each.
  Pending-caller draining is discharged by the shared D4 drain path
  (`degraded_probe_cools_the_endpoint_and_errors_parked_callers`,
  `remap_then_malformed_response_drains_queued_callers_for_the_current_policy`)."
  **(b) M11 row** (`m11-pending-errored`): "Halt-time draining of
  pending callers is discharged by composition: the halt path errors
  the pending caller and publishes
  (`cloudflare_shaped_response_halts_the_gate_and_publishes_status`);
  the shared refusal path drains a queue
  (`remap_then_malformed_response_drains_queued_callers_for_the_current_policy`);
  both traverse the same drain loop."
  **(c) M12 row** (`m12-generic-4xx-no-retry-loop`): "The
  generic-4xx no-retry-loop clause is discharged by composition:
  disposition totality is core-pinned
  (`generic_4xx_with_valid_headers_completes_and_reconciles`; 429
  alone yields Requeue) and the M12 driver arm demonstrates a
  wire-level 4xx completing with exactly one GET."
  **(d) G6** (`g6-reproduction-records`): "G6's enforcement locus is
  the judge's hard-error path: a missing reproduction record aborts
  evaluation before any gate table exists
  (`evidence_cannot_pass_vacuously`; `ReproductionMismatch` is
  structural). The G6 gate row reports record-keeping the hard-error
  path has already enforced."
  **(e) U5 entry** (demotes `m9-headroom-record`; joins U1–U4 at
  `scenarios.md` §5's U block): "U5. Headroom instrumentation
  (added 2026-08-13, ballot pass). M9's 'records what nonzero
  headroom would have bought per contention level' is
  characterization for the headroom-zero decision (§6, decided
  2026-08-09, reconciliation log), not conformance: no gate consumes
  the record. Declared untested and unbuilt; carried like U1–U4
  into the scoped conclusion."
  **(f) §1 sentence** (this document, closing the ballot's SHELL
  finding — SHELL appears in no prerequisite lane): "The SHELL-owned
  dropped-ticket clause (adopted 2026-08-13) counts among the
  verdict prerequisites: both verdict slots require it `Full`,
  alongside X1–X2."
  Ballot findings recorded: the SHELL/§1 enumeration gap (draft f);
  `x2-parser-cap-limitation` sits on neither ballot list
  (Untested-but-accepted) — noted so its absence reads as a
  decision, not an oversight. Out of ballot scope by charter:
  F14–F16, the G3 ε decision (already final), the §7.4 replay.
  Gate evidence: `cargo test --locked` 135 debug green (incl. the
  six registry structural checks), `cargo fmt --all --check`,
  all-target clippy clean, `git diff --check` clean.
- 2026-08-13 — **ballot-pass amendments approved by Tom as-is and
  applied** (status.md §3 item 1 closed same-day). Landed: the five
  `scenarios.md` amendment blocks (M4 representative + D4-drain
  composition, M11 halt-drain composition, M12 generic-4xx
  composition, G6 enforcement locus, the U5
  headroom-instrumentation exclusion) and the §1 SHELL-prerequisite
  sentence (this document; §1's exclusion carry widened U1–U4 →
  U1–U5, and its stale "draft numbers" paragraph updated to the
  2026-08-13 finalization fact). Registry: six rows flipped
  Partial→Full by composition (`m4-scoped-clean-failure`,
  `m4-pending-errored`, `m4-at-most-one-request`,
  `m11-pending-errored` and `m12-generic-4xx-no-retry-loop` with
  composition citations added, `g6-reproduction-records`);
  `m9-headroom-record` re-owned M9→U5 and flipped
  Untested→Excluded, leaving **`OPEN_UNTESTED` = 13**; the owner
  vocabulary grew U to 5 in `tests/obligations.rs`. Registry after:
  122 clauses — 70 Full / 25 Partial / 14 Untested (13 open + the
  accepted X2 parser-cap limitation) / 13 Excluded. Gate evidence:
  `cargo test --locked` 135 debug green, `cargo fmt --all --check`,
  all-target clippy clean, `git diff --check` clean.
- 2026-08-13 — **sanitizer v2** (first real-capture integration
  finding): the capture instrument (`networkcapture.cpp`, schema v1)
  emits local-time timestamps with no UTC offset —
  `QDateTime::currentDateTime().toString(Qt::ISODateWithMs)` — and
  v1 of the sanitizer correctly refused the first real capture
  (2026-08-13, ~1,081 stash gets, VPN, zero 429s). v2 accepts such
  captures only with an explicit `--utc-offset`, validated per
  record against the server `Date` header (±10 s bound; a wrong
  whole-timezone offset misses by ≥ 15 min); the same bound is now
  a standing invariant for every capture, and a repeated boot HEAD
  per endpoint is refused as an append-mode multi-session file.
  Verified against the real capture: correct offset passes with max
  client/server Date disagreement 1,057 ms (within Date's 1 s wire
  precision — incidental real-world support for B14's zero-skew
  model); missing and wrong offsets both refuse; doubled file
  refuses. Owed to master (main-app code, not this branch): the
  one-line instrument fix to emit UTC (`currentDateTimeUtc`), after
  which `--utc-offset` becomes unnecessary for new captures.
- 2026-08-14 — **§7.4 fixtures landed; the replay is unblocked**
  (mock-slice doc finding 8 closed). Two committed fixtures, both
  §4-contract sanitized (sanitizer v2), condition labels attested
  by Tom: `fixtures/capture-20260814-wired.json` (387 records,
  wired ethernet no VPN, native-Z timestamps from the fixed
  instrument, no offset flag) designated **canonical** — Tom's
  typical condition; and `fixtures/capture-20260813-vpn.json`
  (1,129 records, VPN over wifi, offset-naive v1-instrument
  capture accepted via validated `--utc-offset -05:00`) as the
  supplemental B12 latency comparison. Median `sent→received`:
  81 ms wired / 148 ms VPN — same regime, both far below the
  250 ms floor, so the B12 re-anchor is not hypersensitive to
  condition choice; canonical-condition sensitivity was examined
  and found immaterial (any test whose outcome flipped between
  the two defaults would itself be a timing-sensitivity finding
  owing an explicit script). Both sessions: zero 429s; max
  client/server Date disagreement ≈ 1 s (Date wire precision).
  Network-conditions scope closed at two captures by design — no
  matrix. Master-branch instrument commits f53d8cb1 (UTC
  timestamps) and 6288e185 (per-session capture files) are the
  producer-side fixes; the spike branch inherits them at its next
  sync. Gate evidence: registry structural checks green,
  `git diff --check` clean (fixtures + docs otherwise).
- 2026-08-14 — **implementation swarm review packet prepared; slice
  remains open.** F14–F16 are fixed: both M8 lanes share the exact
  two-GET/D5/non-verdict guard, the floor and in-flight values come
  from their authorities, and missing G3 oracle keys fail closed.
  The driver now sweeps M1 residues 0/1/9/10 at φ=0/1 and drives M2
  through burst and sustained stalls; its independent G4 minimum is
  122,581 ms after the canonical 81 ms delay anchor. Focused public-
  actor evidence covers probe-429 seeding/confirmation, per-endpoint
  cooldown and watch publication, organic Retry-After, unaffected
  policies, and dropped-dispatched reconciliation. Focused transition
  evidence covers M5 stale exposure, M6 pre-announcement exposure,
  and M8 concurrent originals. Internal actor composition tests pin
  both real C4 feeds plus C3/X1 latch/drain/publication without
  weakening D5, and X2 now has one private send path plus a compile-
  fail boundary. Registry state: 97 Full, 11 Partial, one accepted
  Untested limitation, 13 Excluded, empty `OPEN_UNTESTED`.

  **SD-R5-F1 — wrong profile provenance in hard-coded OAuth rows,
  fixed.** The prior driver selected `Assumed` by endpoint lookup for
  OAuth scenarios whose endpoint labels were not in the shipped
  table, notably M2 and M6. That made their pacing more conservative
  and weakened the claimed binding evidence. Only the explicitly
  legacy M8/M10 lanes now use `Assumed`; other rows use OAuth
  `Known`. Consequence traced: the next M2 run uses the intended
  10/15 and 30/60 policy and must reach both stalls.

  **CR-R1-F1 — frozen §7.4 expectation conflicts with B3 at an exact
  boundary; open for Tom.** Exhaustive canonical replay finds exactly
  one overflow for each φ=7,454..7,466: counted reply 110 (fixture
  record 114), `stash-request-limit`, sustained 31/30. At the first
  phase, 25 earlier hits remain active until 727,454 ms and six new
  hits arrive at 727,453 ms. Independent arithmetic and the
  production `CounterModel` agree; φ=7,453/7,467 are safe. Phase 0
  matches all 43/43 recorded saturation components. The exhaustive
  gate is retained as an ignored known-finding test and its active
  exact-boundary test prevents disappearance. No model, fixture, or
  frozen scope was amended.

  *[Marker, 2026-08-14 (SD-R5-F2, round-five review): "exactly one
  overflow for each φ=7,454..7,466" is a first-failure artifact —
  the asserting gate aborted at the first violating phase and never
  examined the rest of the cycle. Full enumeration with the same
  production `CounterModel` finds 1,052 violating phases in 20
  disjoint bands (through φ=25,854–25,944; initiating replies
  110–119 and 125–134). Band-one arithmetic above stays correct.
  See the 2026-08-14 repair entry and `VIOLATING_BANDS` in
  `tests/capture_replay.rs`.]*

- 2026-08-14 — **round-five repair session: all fifteen independent-
  review findings validated and dispositioned; review not closed, no
  verdict slot filled, no status flipped to closed.** Every finding
  was independently reproduced against the repository before any
  change; none was invalid. Commits `3e82a963` through `1f61c22a`
  (this entry's commit closes the series). Process note (RE-8):
  `3e82a963` accidentally swept in Tom's unrelated uncommitted
  CMake version bump; `1f61c22a` restored the branch's committed
  CMake content, leaving a net-zero branch delta while preserving
  the bump as the working-tree change where the session found it.
  Dispositions:

  - **SD-R5-F2 (HIGH) — valid, fixed** (`3e82a963`). Reproduced
    exactly before any change: 1,052 violating phases in 20 bands,
    matching the reviewer to the digit; the recorded single-band
    claim was the asserting gate's first-failure abort. The band
    table is pinned (`VIOLATING_BANDS`), the active band-edge test
    covers all 40 edges plus clean neighbors and the 1,052-phase
    accounting, an ignored exhaustive enumeration proves nothing
    violates outside the table, and the four documents carrying the
    understated claim are amended (dated text kept under markers).
    The gate itself is untuned and still fails at φ=7,454/reply 110.
    Whether the 19 further bands were known to the implementation
    session remains undeterminable from the record.
  - **SD-R5-F3 (HIGH) — valid, fixed** (`ec147f96`). The driver now
    builds its per-row profile set through `SweepPlan::new` and takes
    its seeds from the plan, so losing the last shipped-Assumed row
    fails structurally; the `u3-legacy-resolution` note states the
    real guard chain. Profile-lane assignment itself is a recorded
    doc silence for Tom (status.md §3 decision 2).
  - **SD-R5-F4 (MEDIUM) — valid, fixed** (`e135b139`). The focused
    M5/M6/M8 transition lanes run under `OAUTH_KNOWN_PROFILE`; every
    asserted bound was derivation-checked as profile-invariant (the
    shared 60 s sustained resolution governs each) and the tests
    pass under the corrected profile, so the Full evidence for
    `m5-stale-window-exposure`, `m5-no-violation-after-merge`, and
    `m6-preannouncement-exposure` now comes from the bound lane's
    own pacing. Remaining Assumed-engined focused targets are
    deliberately unchanged pending decision 2.
  - **SD-R5-F5 (MEDIUM) — valid, fixed** (`acf22cdb`). The
    tautological `contains` is concat!-split; the send count keys on
    the receiver, not the argument name, and was demonstrated to
    detect a second textual send site. The pinned claim itself was
    re-verified true (one production send site).
  - **SD-R5-F6 (MEDIUM) — valid, fixed** (`5f5da2d7`). The oracle
    trait returns `Option<u64>`; the judge owns the single
    fail-closed branch; the convention is documented on the trait;
    `g3_fails_closed_when_the_oracle_has_no_eligibility_entry` is
    the exposing test. The `u64::MAX` sentinel no longer exists.
  - **SD-R5-F7 (MEDIUM) — valid, fixed** (`e135b139`). The M6 judge
    arm's `passed` is measured from the same wire facts as its raw
    asserts; the boot row anchors at the scripted t=0 instead of its
    own dispatch.
  - **SD-R5-F8 (MEDIUM) — valid, fixed** (`31d6fcfb`). The median
    test asserts `DEFAULT_SERVICE_DELAY` equals the computed fixture
    median; the two 81s are now one anchored fact.
  - **SD-R5-F9 (MEDIUM-LOW) — valid, fixed** (`14928996`). New test
    drives `schedule()` into `start_ordinary`'s trip branch;
    mutation-checked in a scratch copy for both loss modes (lost
    caller resolution, lost rollback). Confessed in the hand-off.
  - **SD-R5-F10 (LOW-MEDIUM) — valid, fixed** (`23f422cc`).
    `m1-g1-sweep` co-cites C1's generated-φ property with an
    ownership note; the contradiction between the two evidence rows
    is gone.
  - **SD-R5-F11 (LOW-MEDIUM) — valid, fixed** (`23f422cc`).
    `s7-4-replay-gate` minted (owner `§7.4`, a deliberate one-off
    like SHELL), Partial, citing the failing gate and both new
    tests; registry totals 123 = 97 Full / 12 Partial / 1 Untested /
    13 Excluded.
  - **SD-R5-F12 (LOW) — valid, fixed** (`31d6fcfb`). All four parser
    bounds pinned at n/n+1; the byte cap enforced at the single
    `bounded_parse` seam with its embedding limitation documented;
    `MAX_JSON_ITEMS` recalibrated 10,000 → 32,768 with derivation
    (the old cap sat below the committed VPN fixture's 15,804
    nodes — scratch-verified that the new VPN test fails under it).
  - **SD-R5-F13 (LOW) — valid, fixed** (`31d6fcfb`). The 43/43
    diagnostic carries the required "pinned as current behavior"
    provenance and states it does not substitute for the gate; its
    asserts carry φ/reply reproduction context.
  - **SD-R5-F14 (LOW) — valid, fixed** (`e135b139`, `83191310`).
    Bare 250/61,000/120,000/20,000s replaced by named
    contract-arithmetic constants with N19/N13/D5 derivations; both
    `D5_IN_FLIGHT_CAP` constants document the deliberate
    judge-independence, with `d5_in_flight_cap_restatements_agree`
    as the drift tripwire (a declaration-consistency check, not an
    oracle).
  - **SD-R5-F15 (LOW) — valid, fixed across the series.** Refusal
    causes pinned at the discriminating level in `actor_safety`
    (escalation vs. parse vs. unexpected-shape); the m4 watch
    clause's note states the doc silence instead of overclaiming;
    the compile_fail doctest pinned to E0603; the C3 latch comment
    describes the actual mechanism; `scenarios.md` §7.4 carries a
    supersession marker beside the stale July 18 fixture text; the
    supplemental VPN median (148 ms) is test-grounded. The fixture
    loader's schema accessors still panic test-locally by design —
    the *parser* is the refusing layer — and the hand-off no longer
    implies otherwise.

  **Doc findings recorded (conservative readings taken, next-call
  consequences traced):**
  (a) *M4 watch-status content* — scenarios.md M4 requires "status
  published on the watch channel" but never says what the snapshot
  must contain; `GateStatus` has no cooldown representation.
  Conservative reading: the drained, non-halted, non-probing
  snapshot discharges the clause. Next-call consequence: a new
  submission on the cooled endpoint is refused with the recorded
  parse cause (now pinned) until the 60 s cooldown expires, after
  which the endpoint re-probes — no wedge.
  (b) *Profile-lane assignment* — the frozen docs bind the shipped
  client default but are silent on which client profile each test
  lane's engine must build. Conservative reading applied where
  evidence claims changed: OAuth-endpoint lanes use Known, legacy
  lanes Assumed, shipped default structurally retained. Next-call
  consequence: none at runtime (test-harness configuration only);
  flagged as status.md §3 decision 2 rather than settled silently.

  **Proportional verification (exact results):**
  `cargo test --locked` — 159 passed, 0 failed, 2 ignored (the §7.4
  gate and the exhaustive enumeration). `cargo test --locked
  --release` — 157 passed, 2 ignored (the two debug-only drop-bomb
  cases absent, as previously recorded). `PROPTEST_CASES=4096 cargo
  test --locked` — 159 passed. `cargo clippy --locked --all-targets
  -- -D warnings` green; `cargo fmt --all --check` clean;
  `git diff --check` clean; `python3 -m unittest discover -s tests
  -p 'test_sanitize_capture.py'` — 4 passed. Explicitly running the
  ignored pair in release: the exhaustive enumeration **passes**
  (6.8 s) and the untuned gate **fails as expected** at φ=7,454,
  reply 110, `stash-request-limit` 31/30, restriction 301 s, with
  its full reproduction record. No live service was contacted; no
  verdict slot was filled; the hand-off status line still reads
  awaiting independent review (re-review of this repair packet is
  the next act).

  *[Marker, 2026-08-14 (RE-3): the paragraph below is the
  implementation-swarm packet's pre-repair matrix, preserved as dated
  history. It is superseded by the 159/157, two-ignored repair matrix
  immediately above and does not describe the enumeration's current
  existence or status.]*

  Proportional verification: `cargo test --locked` green (154 passed,
  one intentionally ignored); `cargo test --locked --release` green
  (152 passed, one intentionally ignored; two debug-only drop-bomb
  cases absent as previously recorded); `cargo clippy --locked
  --all-targets -- -D warnings` green. Explicitly running
  `cargo test --locked --test capture_replay -- --ignored --nocapture`
  fails as expected after 19.84 s at φ=7,454/reply 110 with burst 6/15,
  sustained 31/30, restriction 301 s. Formatting and diff checks are
  recorded with the review-ready commits. No live service was
  contacted and no verdict slot was filled.

- 2026-08-14 — **round-five residual repair sweep: RE-1..RE-9 all
  independently validated and repaired; review remains open.** None
  was invalid, no verdict slot was filled, and the implementing/
  repairing session did not close the round.

  - **RE-1 (MEDIUM) — valid, fixed** (`2afcb13b`). The ignored
    enumeration now accumulates the complete actual `(phase,
    initiating-window-overflow)` set across all 60,000 phases before
    comparing it with the independently expanded band table. It
    retains every overflowing window on the initiating reply and
    measures the total from discovery. Pre-fix mutation reproduced the
    first-φ abort in 0.90 s; the same two-separated-band mutation after
    repair completed the sweep in 6.93 s and reported both φ=7,454 and
    φ=25,944 discrepancies together.
  - **RE-2 (MEDIUM-LOW) — valid, fixed** (`8ed8a922`). M6's four
    wire-derived fragment facts now have one decider whose result
    reaches `judge`; the duplicate raw panic-asserts are gone.
    `m6_fragment_verdict_is_not_constant_true` proves each conjunct can
    make the fragment false. Mutation of the live organic fact reached
    a report with G5 false and `M6Shrink`, rather than aborting before
    the judge.
  - **RE-3 (MEDIUM-LOW) — valid, fixed** (this entry's documentation
    commit). The orphaned 154/152, one-ignored paragraph is explicitly
    marked as the pre-repair packet matrix and superseded by the
    adjacent 159/157, two-ignored repair record.
  - **RE-4 (LOW) — valid, fixed** (`12afb693`). The three orphaned
    tests are citation-visible under their actual owners: the popped-
    caller/rollback trip test under X1, the missing-eligibility test
    under G3, and the D5 restatement tripwire under both direct M10/M13
    cap clauses. Deleting any now fails registry verification.
  - **RE-5 (LOW) — valid, fixed** (documentation commit). §5 now
    states the implemented 32,768-JSON-item cap.
  - **RE-6 (LOW) — valid, fixed** (`8ed8a922`). The last bare 61,000
    became the named N19 derivation: 60 s applicable bucket + 1 s
    buffer, independently restated in the actor-shell test lane.
  - **RE-7 (LOW) — valid, fixed** (`8ed8a922`). Both degraded-probe
    assertions now discriminate `PolicyParseError::MissingHeader`,
    including the cause replayed to the next caller during cooldown.
    The next call after cooldown still re-probes; no new silence was
    exposed.
  - **RE-8 (LOW, process) — valid, fixed** in the earlier repair entry:
    it now records why `3e82a963` and `1f61c22a` touched CMake and that
    their branch delta is net zero while Tom's working-tree bump is
    preserved.
  - **RE-9 (LOW, judgment/nits) — valid, fixed** (`2afcb13b`,
    `10a5e7ef`, `12afb693`, documentation commit). `m1-g1-sweep` is
    conservatively Partial because C1's generated-φ core mirror never
    judges a mock-side boot-residue run; totals are now 123 = 96 Full /
    13 Partial / one accepted Untested / 13 Excluded, with empty
    `OPEN_UNTESTED`. Replay keeps all initiating-window overflows,
    outside neighbors compare against an empty collection, the dead
    neighbor and array guards are removed, the 2 MiB minimal-JSON
    ceiling is correctly described as ~1.05 million values, the
    rollback proof's ten grants are named, the X2 lexical limitation is
    stated beside the claim, and stale singular replay wording is
    pluralized.

  The residual sweep exposed **no new specification silence**. It
  strengthens evidence collection and classification under already
  settled G5, N19, M3, §7.4, and registry rules. Tom's two existing
  decisions (§7.4 adjudication and profile-lane ratification) remain
  exactly as recorded in `status.md` §3.

  **Proportional verification (exact results):** `cargo test
  --locked` — 160 passed, 0 failed, 2 ignored (doc-tests included);
  `cargo test --locked --release` — 158 passed, 0 failed, 2 ignored
  (the two debug-only drop-bomb tests are absent); `PROPTEST_CASES=4096
  cargo test --locked` — 160 passed, 0 failed, 2 ignored. `cargo
  clippy --locked --all-targets -- -D warnings`, `cargo fmt --all
  --check`, `git diff --check`, and the six registry verification
  tests were clean; the Python sanitizer suite passed 4/4. Explicit
  release ignored runs: the collect-first exhaustive enumeration
  passed all 60,000 phases in 6.76 s; the untuned canonical gate
  failed as expected at φ=7,454, reply 110, `stash-request-limit`
  sustained 31/30 with restriction 301 s and a full reproduction
  record. No live service was contacted; no verdict slot was filled;
  the round and slice remain open for independent re-review.

  *[Marker, 2026-08-14 (SD-R5 close): the re-review has since run and
  closed round five — see this changelog's final entry. The slice
  remains open; the dated text above is preserved.]*

- 2026-08-14 — **Tom approved both `status.md` §3 sign-offs.**

  **§7.4 adjudication:**

  > I adjudicate CR-R1-F1/SD-R5-F2 as a §7.4 frozen-contract
  > expectation error. A recorded feedback-dependent dispatch trace is
  > not required to remain safe when replayed open-loop under
  > counterfactual server phases. Preserve B3, both fixtures, and the
  > exhaustive counterexample diagnostic. Replace the every-phase
  > fixed-trace gate with feedback-consistent calibration; retain
  > every-phase safety requirements in the closed-loop C1/M-series
  > tests.

  Consequence: no model or fixture changed, and the 20-band / 1,052-
  phase set remains pinned. The old ignored assertion is now a finding
  reproduction, not the calibration gate. `s7-4-replay-gate` remains
  Partial until the feedback-consistent replacement is precisely
  specified, implemented, independently reviewed, and run green. No
  verdict slot was filled.

  **Profile-lane ratification:**

  > I ratify the test-evidence profile assignment: OAuth-bound scenario
  > evidence uses Known(5s/60s); explicitly legacy evidence uses
  > Assumed(60s/60s); and the shipped Assumed default remains
  > structurally represented. Generic focused tests may use the shipped
  > default only where their asserted behavior is demonstrably
  > profile-invariant. Any future burst-resolution-sensitive claim must
  > run under its bound profile.

  Consequence: the repaired driver and transition-lane assignments are
  accepted. `actor_safety` and `actor_shell` may retain their shipped-
  Assumed engines for their current profile-invariant assertions; that
  evidence cannot be extended to a profile-sensitive burst claim
  without a bound-profile run. This ratification is test-evidence
  bookkeeping and does not change `bootstrap-seeding.md`'s one global
  positional runtime default.

- 2026-08-14 — **round five closed (SD-R5): independent re-review of
  the repaired packet found no new findings.** A fresh session read
  the mandated documents in `AGENTS.md` order and validated every
  round-five disposition against code, tests, and docs — all fifteen
  SD-R5-F2..F15 repairs and all nine RE-1..RE-9 residual repairs
  confirmed as described, none overstated. Spot-verified in source:
  the 20-band `VIOLATING_BANDS` table with its 40-edge/clean-neighbor
  pins and 1,052-phase accounting (F2/RE-1); `SweepPlan::new`'s
  load-bearing shipped-Assumed guard feeding the driver's seeds (F3);
  the focused transition lanes' `Known` engines with profile-invariant
  bounds (F4); the concat!-split receiver-keyed X2 pin (F5); the
  judge-owned `Option<u64>` G3 fail-closed branch and its exposing
  test (F6); the M6 fragment's single wire-fact decider reaching
  `judge` plus its four-way falsifiability guard (F7/RE-2); the
  anchored B12 median assertion (F8); the `schedule()`-driven
  `start_ordinary` trip test asserting both loss modes with the named
  ten-grant rollback proof (F9/RE-9); the registry's `m1-g1-sweep`
  conservative Partial and `s7-4-replay-gate` slot encoding Tom's
  adjudication verbatim (F10/F11/RE-9); the n/n+1 parser-bound pins
  at the single `bounded_parse` seam with the 32,768-item derivation
  (F12/RE-5); the 43/43 diagnostic's pinned-as-current-behavior
  provenance (F13); the named contract-arithmetic constants and
  dual `D5_IN_FLIGHT_CAP` tripwire (F14/RE-6); the discriminated
  refusal causes, E0603 compile-fail, and stated doc silences
  (F15/RE-7); the marked pre-repair matrix (RE-3); the
  registry-cited X1/G3/D5 regression tests (RE-4); and the net-zero
  CMake branch delta with Tom's working-tree bump preserved (RE-8,
  re-verified by `git diff` across the repair series). Registry
  totals recomputed independently from source: 123 = 96 Full /
  13 Partial / 1 Untested / 13 Excluded, `OPEN_UNTESTED` empty.
  Re-run verification matrix, entirely offline: `cargo test --locked`
  160 passed / 0 failed / 2 ignored; `--release` 158 / 0 / 2;
  `PROPTEST_CASES=4096` 160 / 0 / 2; all-target clippy with warnings
  denied, fmt check, `git diff --check`, obligations 6/6, and the
  Python sanitizer suite 4/4 all clean. Explicit release run of the
  ignored pair: the exhaustive enumeration passed all 60,000 phases
  in 6.92 s; the superseded open-loop gate failed as expected at
  φ=7,454, reply 110, `stash-request-limit` sustained 31/30,
  restriction 301 s, with its full reproduction record. One
  non-blocking review note, recorded here rather than as a finding:
  the `client_buckets` half of a `ReproductionRecord` is
  producer-declared and cannot be cross-checked by the
  client-independent judge (the mock never observes the client's
  bucket profile; seed and φ are cross-checked via
  `ReproductionMismatch`) — inherent to the judging interface, worth
  knowing when future full-contract runs cite reproduction records
  as profile evidence. Closure acts per `slice-review.md` §5:
  `status.md` §2/§4/§5 brought current, the hand-off status line
  flipped to name the closed round, and this entry. The slice
  remains open on `status.md` §5's residual set; no verdict slot
  was filled; no live service was contacted.

- 2026-08-14 — **M1 generated-φ mock-side residue sweep landed**,
  discharging the exact RE-9 delta for `m1-g1-sweep` (residual item 1
  of `status.md` §5). New target `tests/m1_residue_sweep.rs`:
  proptest generates residue 0..=12 and φ over the full 60,000 ms
  cycle (with the three §3 rollover cases 4,999/5,000/5,001 pinned as
  explicit strategy arms), and every case boots the public actor
  against the mock, submits one stash-list request, and hands the
  mock's observations to `conformance::judge` with G1/G2/G3/G6 armed
  under the OAuth Known profile. The wire facts reach the judge as
  the M1 scenario assertion (the RE-2 sole-decider pattern) with a
  falsifiability guard; the sustained-window residue count is the
  per-case non-vacuity anchor (the mock provably judged against live
  residue at every generated φ, in both branches); zero-budget cases
  assert branch reachability (no early dispatch, before and after the
  coarse advance) and wait the independently derived 20 s bound. The
  residue cap of 12 — above the burst limit, strictly below the
  sustained 30 — is a recorded judgment call, not a doc silence.
  Mutation checks: a broken residue anchor reached the judge as
  `G5 failed: ["M1BootSequence"]`, and the weakened zero-budget
  oracle entry reached G3 as 19,875 ms measured lateness; the real
  bound's observed slack is 19 ms against ε=500 ms. Runs green at the
  default 256 and at 4,096 generated cases (0.96 s). Registry:
  `m1-g1-sweep` flipped to Full with the new citation (C1 retained
  as the core-side mirror, supporting only); totals now 97 Full /
  12 Partial / 1 Untested / 13 Excluded. No verdict slot was filled;
  no live service was contacted.

- 2026-08-14 — **Ballot G landed: the forced M9 phantom race at
  14/15** (residual item 2 of `status.md` §5), discharging
  `m9-recovery-survives-race`, `m9-race-exposure-attribution`, and
  the last scripted arm of `b12-scripted-delay`. New focused test
  `transition_timing::m9_forced_phantom_race_at_saturation_recovers_per_m8`
  at φ=0/1: 14 residue hits preload `stash-request-limit`'s 15/10 s
  burst window; the boot HEAD announces 14/15; the opening GET
  reserves and dispatches, and B12's explicit 2 s arrival delay holds
  it on the wire while a mock-owned phantom consumes the last slot
  strictly between transport hand-off and mock receipt (both
  inequalities asserted). The send lands as the 16th and draws an
  *organic* 429 — the mock's burst-window judgment pins the exact
  construction: 14 residue + 1 phantom + 1 client = 16 over 15.
  Recovery runs per M8's asserts: no immediate re-knock, exactly one
  confirmation in flight, the retry waits `Retry-After` + the 60 s
  applicable bucket + buffer (N19) past the raced completion, and no
  follow-on violation. The exposure is attributed through the public
  §2 seam — `ExposureAllowance::for_state_change` bound to the
  phantom injection with cap 1 (the in-flight set at injection time),
  the observable instant independently scripted as the raced
  response's completion — and the judge passes with G1 armed, while
  the identical evidence without the allowance fails G1 on the raced
  correlation, so the attribution is load-bearing, not decorative.
  Wire facts reach the judge as the sole G5 decider (RE-2 pattern)
  behind an eleven-way falsifiability guard. Mutation check: a
  phantom injected on a different policy fails the organic-429
  assertion (the race cannot pass vacuously). Registry: all three
  clauses flipped to Full; totals now 100 Full / 9 Partial /
  1 Untested / 13 Excluded. No verdict slot was filled; no live
  service was contacted.

- 2026-08-14 — **M11a near-ceiling compliant sweep landed** (residual
  item 3 of `status.md` §5), discharging `m11-compliant-never-trips`
  — the 2026-08-13 amendment's named binding evidence for G2's
  property. New target `tests/m11_ceiling_sweep.rs`: because every
  N23 policy caps the wire far below the D5 floor rate, the sweep
  uses B7's scriptable-synthetic-policy channel (a
  1,000/10 s + 10,000/60 s RulePair) so the 250 ms floor is the
  binding constraint, then drives 300 queued submissions through the
  public actor — the compliant maximum wire pressure, the closest a
  correct client can get to layer 1. The mock judges all 301
  dispatches with G2 armed; the peak rolling occupancies are pinned
  exactly — 4 of 20 in the rolling second, 240 of 1,000 in the
  rolling minute (the charter's "250 ms ⇒ ≤ 240 req/min" made
  executable, with the 5× headroom ordering checked at compile
  time) — and no arrival trips either ceiling. Run under both bucket
  profiles per the profile-lane ratification (the synthetic policy is
  generic; identical wire shape across profiles demonstrates
  invariance rather than arguing it). Wire facts reach the judge as
  the sole G5 decider (RE-2 pattern) behind a falsifiability guard
  that also rejects peaks *above* the compliant maxima; the G3 oracle
  is floor-only arithmetic over the mock's own dispatch log.
  Mutation check: shrinking the synthetic burst limit to 10 makes
  policy pacing dominate and the run fails on both axes — G5 (peak
  sustained falls to 41; near-ceiling reachability lost) and G3 (the
  floor-only oracle flags the unmodeled policy waits). Registry:
  `m11-compliant-never-trips` Full; totals now 101 Full / 8 Partial /
  1 Untested / 13 Excluded — the remaining Partial set is exactly
  `s7-4-replay-gate` plus the seven fragment-scale clauses the
  full-contract run owns. No verdict slot was filled; no live service
  was contacted.
