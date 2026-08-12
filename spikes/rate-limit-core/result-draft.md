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
green, **and** X1–X2 green, with U1–U4 carried into the scoped
conclusion. X2 is load-bearing, not auxiliary: "single serialized
gate" is part of the register question itself.

- **Unconditional** — the four OAuth policies, bucket resolution
  `Known(5s/60s)` (N12): ⟨verdict; requires the prerequisites
  above over these policies⟩
- **Conditional** — `backend-item-request-limit`, bucket resolution
  `Assumed(60s/60s)` (not provably pessimistic; N14/N21 give no
  upper bound): ⟨verdict, stated *with* its assumption; same
  prerequisites, conditional lane⟩

G3 (ε = 500 ms) and G4 (1.05×) are draft numbers; they must be
finalized (`scenarios.md` §6 revisit rule) before either verdict
slot is filled.

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

### Mock-judged wire scenarios

| ID | Scenario | Sweep | Gates exercised | Result | Evidence |
|---|---|---|---|---|---|
| M1 | Cold start with residue (flagship) | phase-swept | G1, G2, G6 | partial — core bootstrap/recovery and independent residue/HEAD mock capability green; actor wire run pending | 2026-08-10: bootstrap evidence below plus mock commits `4353fb03`/`05ee15d1`; `mock_fidelity::b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology` and `b6_b9_residue_and_phantoms_are_mock_owned_counter_facts` green. |
| M2 | Clean cold-start saturation burst | phase-swept | G1–G4, G6 | partial — independent quantized counters and G1–G4 judge executable; actor saturation run pending | 2026-08-10: generated phase oracle plus exact expiry, post-increment, G3, and exact 1.05× G4 boundary tests green; mock-slice evidence below. |
| M3 | Degraded HEAD | independent | G1, G2, G5 | partial — policy-only HEAD stimulus and typed production-parser failure executable; actor cooldown/caller behavior pending | 2026-08-10: `mock_fidelity::b11_m3_m4_script_channel_covers_every_response_shape` green. |
| M4 | Unexpected policy shape | independent | G1, G2, G5 | partial — verbatim one-/three-window synthetic policies executable; actor scoped failure/watch behavior pending | 2026-08-10: `mock_fidelity::b1_b7_m4_synthetic_one_and_three_window_policies_cross_as_raw_headers` green. |
| M5 | Policy rename mid-session | phase-swept | G1, G2, G6 | partial — route mutation, deterministic delay, unique correlation, and exposure attribution executable; focused core/actor remap adoption and post-remap refusal drain green, phase-swept scenario-judge run pending | 2026-08-10: `mock_fidelity::b8_policy_rename_and_shrink_keep_existing_hits`, B12/B13 tests, and `conformance_harness::correlation_and_reproduction_seams_are_structural` green; 2026-08-12 `response_reconciliation::m5_remaps_an_ordinary_token_without_losing_in_flight_history` plus `actor_shell::{m5_remap_updates_the_actor_endpoint_mapping,remap_then_malformed_response_drains_queued_callers_for_the_current_policy}` green. |
| M6 | Policy shrink mid-flight | phase-swept | G1, G2, G6 | partial — hits-facts/rules-judgments replacement preserves history and restrictions pessimistically; focused core/actor same-policy limit shrink with held history green, phase-swept scenario-judge run pending | 2026-08-10: `mock_fidelity::b8_policy_rename_and_shrink_keep_existing_hits` green; 2026-08-12 `response_reconciliation::m6_replaces_rules_immediately_while_retaining_history_facts` and `actor_shell::m6_shrink_blocks_new_dispatches_from_the_announcing_response` green. |
| M7 | Phantom same-account hits | phase-swept | G1, G2, G6 | partial — core synthesis plus mock-owned bursty phantom injection/provenance green; actor observation loop pending | 2026-08-09 core evidence above; 2026-08-10 `mock_fidelity::b6_b9_residue_and_phantoms_are_mock_owned_counter_facts` green. |
| M8 | 429 recovery and escalation | phase-swept | G1, G2, G5, G6 | partial — core ladder plus independent organic/injected 429, restriction, malformed, and transport-error shapes green; focused actor retry/first-confirmation-escalation tests green, phase-swept wire matrix pending | 2026-08-09 core evidence above; 2026-08-10 B2/B3 and B11 mock-fidelity tests green; 2026-08-12 `actor_shell::{m8_429_requeues_through_the_core_not_before_deadline,m8_confirmation_429_escalates_without_a_third_get}` green. |
| M9 | Phantom race at saturation | phase-swept | G1, G2, G5, G6 + characterization | partial — arrival delay, phantom injection, source counts, correlation, and capped pre-observation exposure executable; actor race/headroom record pending | 2026-08-10: B6/B9, B12/B13, and capped G1 unavoidable-exposure tests green. |
| M10 | Agent-loop stress | phase-swept | G1, G2, G3, G6 | partial — bounded 10,000-dispatch mock/log and global G1/G2/G3 judge executable; actor pressure/cancel/reprioritize run pending | 2026-08-10: exact budget/latch, ceiling, and non-vacuous judge tests green. |
| M11 | Layer-1 ceiling + Cloudflare terminal | independent | G2, G5 | partial — core halt plus both independent B10 ceilings and injected/organic Cloudflare shapes green; compliant-client run pending actor | 2026-08-09 core evidence above; 2026-08-10 `mock_fidelity::b10_b11_layer1_and_injected_stimuli_are_distinct` green. |
| M12 | 4xx-tripwire obligations | independent | G5 | partial — full/raw 401/generic-4xx/429 stimulus channel executable; tripwire feed/retry obligations pending actor | 2026-08-10: B11 script-channel tests green. |
| M13 | Gate structure on the wire | independent | G2 + gate-definition assertions | partial — deterministic overlap, arrival order, HEAD overlap, in-flight, spacing timestamps, and run-wide identities observable; focused actor writer-preference/FIFO/cap/exclusivity tests green, compliant scenario-judge run pending | 2026-08-10: `mock_fidelity::b12_b13_explicit_delay_makes_overlap_and_correlation_observable` green; 2026-08-12 `actor_shell::{pending_head_writer_blocks_a_front_get_until_it_runs_exclusively,m13_ordinary_waiters_are_fifo_and_never_exceed_two_in_flight}` green. |

G1, G2, G3, and G5 are armed in every mock-judged scenario; the
column lists the gates each scenario is the *binding evidence*
for.

### Core-property tests

| ID | Property | Result | Evidence |
|---|---|---|---|
| C1 | Padding arithmetic safe over all φ | green — full N13 per-window padding uses each explicit Known/Assumed resolution; shared policy history is judged across every rule/window and the maximum required `NotBefore` wins; headroom remains zero | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 19 passed, including a generated C1 property over arbitrary histories, multi-rule definitions, and independently generated server phases plus explicit just-before/on/after rollover and zero-headroom/order-statistic cases; focused `PROPTEST_CASES=4096 cargo test --locked --test c1_scheduling every_reserved_outcome_is_safe_for_every_server_phase` green (4,096 cases); independent oracle bucketizes hits on the server phase rather than calling production scheduling arithmetic; `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green. No skew sensitivity observed because this slice has no server-clock input; O5 remains out. Audit hardening (2026-08-09, same day): the property now asserts on every generated case — the earlier body was ~97% vacuous (§3 register, item 7) — and the `NotBefore` branch is re-asked and oracle-checked, pinning exactness; re-verified at 4,096 cases |
| C2 | Header parsing / shape validation | green for the implemented core slice — raw-header parsing, RulePair shape, and frozen response precedence are executable; remapping/shrink remain explicitly out of this slice | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 44 passed overall: the 7 parser tests remain green and 15 disposition tests pin Cloudflare-before-parse, malformed/out-of-model-before-429, valid-429 handling, and ordinary/probe outcomes; `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green |
| C3 | Fuse trip logic | green for the implemented actor boundary — exact burst (10/11 + half-open edge), sustained (499/500), and a non-vacuous floor-compliant property are green | 2026-08-12: `actor::tests::{c3_and_x1_fault_injection_pin_burst_and_sustained_boundaries,fuse_uses_the_documented_half_open_boundaries,x1_fault_injection_trips_at_the_actor_transport_boundary,c3_floor_compliant_traces_never_trip}`; the property ran at 4,096 cases with independent timestamp-window arithmetic. |
| C4 | 4xx tripwire logic | green for the implemented counter — burst, sustained, and half-open edge boundaries pinned | 2026-08-12: `actor::tests::c4_pins_burst_sustained_and_exact_window_edges` green in debug and release. |
| C5 | Lifecycle invariants | green — reservation/rollback/unknown-outcome identity and abandonment semantics remain green; raw ordinary responses and tokenless probes still share one count-max/synthetic-history reconciler; unknown confirmation outcomes stay counted; abandonment now covers the confirmation half (a dropped confirmation ages out as a failed attempt instead of wedging the policy — §3 register, item 2) | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 44 passed: all prior C1/C5/reconciliation tests remain green, and the disposition suite pins confirmation rollback plus pessimistic unknown retention; focused `PROPTEST_CASES=4096 cargo test --locked --test response_reconciliation` remains green (4,096 cases for each of two generated properties); `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green. Audit hardening (2026-08-09, same day): abandoned-confirmation expiry pinned in debug and release; interleaving property extended with observed responses and non-FIFO token resolution (2,048-case focused run); 59 tests total |

### Fault-injection and structural

| ID | Check | Result | Evidence |
|---|---|---|---|
| X1 | Fuse true-positive, burst and sustained fault shapes — the lane upgrade from declared-untested | green for the actor boundary — fault-injected counter contents reach `start_dispatch`, the last common hook before `Transport::send`; production D5 pacing remains enabled | 2026-08-12: `actor::tests::x1_fault_injection_trips_at_the_actor_transport_boundary` pins the 11th burst and 500th sustained trips without creating a second scheduling path. |
| X2 | Transport boundary: one HTTP client, private, no second send path | ⟨…⟩ | ⟨…⟩ |

### Gate summary

| Gate | Statement | Result | Evidence |
|---|---|---|---|
| G1 | Zero client-caused violations (incl. follow-on) | ⟨…⟩ | ⟨…⟩ |
| G2 | Neither B10 ceiling rule tripped, armed everywhere | ⟨…⟩ | ⟨…⟩ |
| G3 | Per-dispatch over-delay ≤ ε (final ε: ⟨…⟩) | ⟨…⟩ | ⟨…⟩ |
| G4 | M2 duration ≤ multiplier × padded minimum (final: ⟨…⟩) | ⟨…⟩ | ⟨…⟩ |
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
not an inferred pass.

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
relationship to the aging horizon) plus a test.

### Tokio actor slice (2026-08-12 — awaiting review)

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
remain unclaimed in `actor-handoff.md` §3.

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
and this obligation is confessed in the slice hand-off.

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

⟨New candidates minted during implementation land here the day
they appear — writing the mock exposing an N-claim ambiguity is a
finding in its own right (charter, step 1).⟩

## §5. Calibration: capture replay

Per `scenarios.md` §7.4 — the piece that grounds the model in the
observed lane:

- Fixture: ⟨sanitized July 18, 2026 capture (132 records), §4
  contract; sanitizer version, capture schema `v`⟩
- Initialization: seeded from the capture's boot-HEAD state
  headers (reconciliation mechanism, phantoms at t₀); boot-HEAD
  records are initialization evidence, never replayed as
  counter-producing arrivals — ⟨confirm no double-application⟩
- Gate — zero violations at every φ: ⟨result; any failing φ is a
  finding to adjudicate (mock bug vs. model exceeding N13's
  margin), recorded here, never tuned away⟩
- Diagnostic — saturation-state agreement (15/15, 30/30; N25/N26):
  ⟨observations; informs, does not fail⟩
- B12 delay re-anchor: ⟨capture median `sent→received` → final
  default; replaces the 50 ms placeholder⟩

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
