# Scenarios and pass/fail criteria: rate-limit-core spike

Status: agenda-item output doc (items 1, 2, and 4 per the sibling-docs
convention in `design-brief.md`). Consolidated 2026-08-09 from the
closed item-0 reconciliation and the plan-review addendum
(`inputs/plan-review-2026-08-09.md`; findings 1–4 and 9 land here).
All decisions cited here are settled in the charter's reconciliation
log; this doc makes them citable by tests. Tests cite scenario IDs
the way designs cite N-numbers. This doc is hoisted with the
conformance suite if the suite outlives the spike.

N-numbers cite `docs/design/network-ground-truth.md`. D-numbers cite
`docs/design/network-redesign.md`. All three of this doc's items are
landed: §7 (item 4, the mock fidelity budget) was decided in the
2026-08-09 side-bar.

---

## §1. Bucket knowledge (finding 1 + follow-up 3)

Window shape and bucket-resolution knowledge are **separate facts**.
The shape invariant (`RulePair`: exactly two windows, first period
strictly shorter — parse-time validated) says nothing about bucket
resolution; a two-window policy must never silently imply known
buckets. Resolution knowledge is provenance-typed:

| Policy | Shape | Bucket resolution (burst/sustained) | Provenance | Verdict lane |
|---|---|---|---|---|
| `stash-list-request-limit` | RulePair | **Known(5s / 60s)** | N12 (GGG-EMAIL, per-policy fact) | unconditional |
| `stash-request-limit` | RulePair | **Known(5s / 60s)** | N12 | unconditional |
| `character-list-request-limit` | RulePair | **Known(5s / 60s)** | N12 | unconditional |
| `character-request-limit` | RulePair | **Known(5s / 60s)** | N12 | unconditional |
| `backend-item-request-limit` (legacy, Account + Ip rules) | RulePair per rule | **Assumed(60s / 60s)** | none — N21: unknown; N14 channel parked | conditional |

Rules for the `Assumed` variant:

- `Assumed(60s/60s)` is an explicit assumption, **not** provably
  pessimistic — N14/N21 give no upper bound on legacy bucket
  resolution. The server could use a larger bucket.
- It is replaceable via internal configuration or GGG evidence
  (the parked N14 ask-us thread), never an implicit default. There
  is no code path that manufactures a resolution for a policy the
  table doesn't cover — an unknown policy without configured
  resolution is a refusal, not a guess.
- The unconditional "honors the N-claims" verdict is scoped to the
  four OAuth policies. The legacy lane's verdict is stated as
  conditional on its assumption, and legacy conformance does not
  count toward the unconditional verdict.
- Working tier assumption (both lanes): positional — first triplet
  is the burst limit, second is sustained (Q4 hypothesis; what the
  validation-run instrument gives measured-lane support for).

## §2. Verdict vocabulary (finding 2)

"Zero violations" is ambiguous when scenarios deliberately inject
429s. The vocabulary:

- **Client-caused violation**: the mock's own counters — driven
  solely by received wire timestamps — exceed a rule's limit as a
  result of traffic the client under test scheduled. The global
  invariant is `client_caused_violations == 0` (§6, G1). Any 429
  the mock's counters generate organically is client-caused and
  fails the run, with exactly one exception: the
  unavoidable-exposure category below, which is explicit, bounded,
  and harness-attributed (external review F10 — the exception must
  be stated here, not implied).
- **Injected stimulus**: a 429 (or phantom counter increment, or
  Cloudflare-shaped reply) the scenario script injects regardless
  of the client's arithmetic. Stimuli are expected and do not count
  against G1; each stimulus scenario carries its **own assertions**
  (retry timing per N19, escalation behavior, halt semantics), and
  those assertions failing fails the scenario.
- **Follow-on violation**: a client-caused violation occurring
  *after* a stimulus (e.g. a retry re-scheduled too early lands
  inside the still-saturated window). Counts against G1 — recovery
  correctness is load-bearing (headroom-zero decision).
- **Unavoidable exposure (transition or race)** (external review
  refinements, 2026-08-09): an *organic* mock 429 whose request
  was reserved before the client could have observed the mock-side
  state change that made it violate — a scripted policy mutation
  (M5 remap, M6 shrink: *transition*) or an injected phantom
  (M7/M9: *race*). Bounded by the in-flight cap: at most
  in-flight-cap − 1 arrivals after the response that first
  announces a mutation, or the in-flight set at phantom-injection
  time. Does not count against G1 — but it must enter the M8
  recovery path and clear its assertions; any reservation granted
  after the change was observable is fully subject to G1. The
  harness attributes this category independently of the client,
  using B13's correlation identity.

## §3. Mock-oracle independence (finding 3 + follow-up 2)

The mock must not be a duplicate of the client's arithmetic — a
client and mock sharing one `ceil_to_bucket` interpretation could
agree while both embody the same mistaken assumption. Rules:

- **Server-owned bucket phase.** The mock's buckets have an explicit
  phase offset φ, owned by the mock, invisible to the client. The
  N-claims confirm buckets exist (N11) but not their alignment;
  the client's full-bucket padding (N13) must be safe for **every**
  φ — that is the property under test.
- **Black-box counters.** Mock counters are driven only by received
  wire timestamps quantized on the mock's own timeline. No reuse of
  client scheduling, padding, or reconciliation functions — the
  mock's window arithmetic is written independently (separate
  module, no shared helpers beyond header serialization types).
- **Property-style phase sweeps.** Every scenario whose verdict
  depends on bucket expiration, padding, shrinkage, residue, or
  retry timing is swept: proptest generates φ ∈ [0, bucket), the
  failing seed/phase is recorded for exact reproduction. Paused
  time makes this cheap.
- **Exact-boundary cases.** In addition to generated phases, each
  swept scenario pins three explicit cases: just before, exactly
  on, and just after a bucket rollover. No Cartesian expansion
  beyond that.
- **Tags.** Every scenario is tagged `phase-swept` or
  `phase-independent` in §5. Parsing, malformed shapes,
  cancellation, queue behavior, and structural checks stay out of
  the sweep.

Mock-architecture constraint (reconciliation log): the mock is an
in-process service sharing the client's paused tokio runtime, not a
standalone binary — timing assertions must be deterministic.

## §4. Fixture sanitization contract (finding 9)

`networkcapture` output contains account and stash identifiers and
is not committed (N23 note). Derived fixtures follow this contract:

- **Retained:** policy name, endpoint *label* (the D5 five-endpoint
  vocabulary), HTTP status, error code/string, the verbatim values
  of `x-rate-limit-*` and `retry-after` headers, and **relative**
  timing — `scheduled`/`sent`/`received` and the server `Date`
  header, all rebased to t₀ = first record of the capture. Both
  response order and dispatch timestamps are preserved (pacing
  analysis needs both).
- **Removed:** URLs (reduced to endpoint labels), account names,
  stash/tab identifiers, character names, tokens/cookies/auth
  headers of any kind, request and response payloads, absolute
  wall-clock times.
- **Committable:** fixtures satisfying this contract may be
  committed to the spike branch. Anything not listed under
  "retained" is removed by default — the sanitizer is an
  allowlist, not a blocklist.
- **Provenance without secrets:** each fixture file carries a
  header block — capture date, capture schema version (`v`),
  sanitizer version, session shape (e.g. "full refresh, ~121
  stashes, OAuth, PC realm"), and the claim lanes it supports —
  but no account-identifying material.
- The sanitizer is itself a small tool in the spike tree; fixtures
  are regenerable from retained raw captures, so a sanitizer bug
  is recoverable.

---

## §5. Scenario list (agenda item 1)

Lanes: **mock-judged** (wire behavior judged by the mock's
independent counters), **core-property** (pure-function unit and
proptest coverage), **fault-injection / structural** (X-series),
**declared-untested** (U-series register — the result doc states
these lanes explicitly).

### Mock-judged wire scenarios

**M1. Cold start with residue (flagship).** [phase-swept]
Mock pre-loads counters mid-window before the client boots (N24:
server counters are per-account and persist across restarts).
Client boots, issues exactly one HEAD per endpoint it touches,
before that endpoint's first GET, strictly serialized (N16, N18 —
the boot pattern N24 observed on all five endpoints), reads the
state header showing residue, and schedules its opening traffic
against the *remaining* budget. Sweep residue magnitude and φ;
include the boundary case where residue leaves zero budget in the
current bucket. Asserts: one HEAD per touched endpoint, never
repeated, never overlapping; HEAD does not increment
counters (N24); first-request violation does not occur — the
HEAD's state header is the only thing preventing it; G1 across the
sweep. Cites: N24, N16, N13, N12, N5.

**M2. Clean cold-start saturation burst.** [phase-swept]
Empty counters, deep queue (one policy). Client drains the burst
window, stalls padded, drains again into the sustained window,
stalls padded (the N26 burst-then-stall shape). Asserts: G1; state
tracks 1:1 post-increment (N25); over-delay bounds G3/G4 measured
here. Cites: N5, N12, N13, N25, N26.

**M3. Degraded HEAD.** [phase-independent]
HEAD returns `x-rate-limit-policy` only (the Dec 2023 regression
shape, N20). Asserts: header parse returns a typed error (no
out-of-bounds, no empty-policy adoption — the N20 correction);
endpoint fails cleanly under cooldown (D4 semantics); zero
requests sent on that endpoint; other policies unaffected; pending
callers get errors, not hangs. Cites: N20, D4.

**M4. Unexpected policy shape (one- and three-triplet).**
[phase-independent]
Mock serves a one-triplet rule, then (separate case) a
three-triplet rule (the trade-API shape — external lane, URL in
the charter's bucketing entry). Asserts: parse yields
`UnexpectedPolicyShape`; scoped per-policy clean failure (D4-style
cooldown), not app abort; pending requests errored to callers;
status published on the watch channel; **at most one request ever
sent under an unknown shape**; other policies keep flowing.
Cites: N5, N9, N20 (transient-server-bug premise).

**M5. Policy rename mid-session (reactive remap).** [phase-swept]
Mid-scenario, responses start carrying a new `X-Rate-Limit-Policy`
name (N5, N9). Asserts: client remaps the endpoint and
pessimistically merges history; at most in-flight-cap (2) requests
were scheduled under the stale mapping (any organic 429 among them
is unavoidable exposure, §2, and must clear M8's recovery
assertions); no client-caused violation after the merge; remap
*triggers* beyond the reactive path are U1. Timing script per B12:
the stale-mapping window is forced by scripted service delay, not
left to the default.
Cites: N5, N9, N6.

**M6. Policy shrink mid-flight.** [phase-swept]
Limits tighten mid-session (N9), e.g. `15:10:60` → `10:10:60`,
while history already holds more than the new limit allows.
Asserts: the shrink is honored from the response that announces
it. Mock judging rule (external Q&A, 2026-08-09): **hits are
facts, rules are judgments** — every arrival is judged against
the rule set active at that instant, existing window contents
included, no grace period (a real server gives none; no violation
is ever declared without an arrival). Pre-announcement in-flight
arrivals that draw an organic 429 are unavoidable exposure
(§2, bounded ≤ in-flight-cap − 1) and must clear M8's
recovery assertions; G1 applies fully from the first
post-announcement reservation; queue keeps draining at the new
pace (no wedge). Cites: N9, N13.

**M7. Phantom same-account hits.** [phase-swept]
Mock injects counter increments the client under test didn't cause
— occasional and bursty (a second tool launching, per the
corrected threat model; N23: Account-scoped rules), not constant
drizzle. Asserts: client reconciles pessimistically when observed
state exceeds its model (scope-blind — no per-scope machinery);
no client-caused violation. Thresholds must not be tuned against a
constant-drizzle world that doesn't exist. Cites: N23, N24, N25.

**M8. 429 recovery and escalation ladder.** [phase-swept]
Injected 429 with `Retry-After` (stimulus, §2). Runs over both an
OAuth policy and the legacy Account+Ip policy; the applicable
bucket for retry timing is the maximum resolution across the
policy's windows (core-design rule, external review F4). Timing
script per B12: concurrent in-flight originals are forced by
scripted delays, not incidental. Asserts
(the stimulus's own assertions): the retried send waits
`Retry-After` + applicable bucket + buffer (N19), not `Retry-After`
alone; the caller eventually observes the outcome (the F57 lesson);
a second consecutive 429 on the same policy escalates — back
off / suspend the policy's queue and surface, never politely
re-knock (the 4xx double-dip makes a third knock doubly
expensive); during recovery at most **one** post-restriction
reservation is in flight for the restricted policy (single retry
in flight — concurrent originals bounced by the same saturation
join one episode; core-design episode semantics, generation-
tagged tokens); the full confirmation matrix (core-design) is
exercised case by case; no follow-on violation (§2). Cites: N19, N15, P-A,
the 4xx-budget candidate claim.

**M9. Phantom race at saturation (characterization).** [phase-swept]
At 14/15 a phantom consumes the last slot between the client's
reservation and the mock's receipt — the client's send lands as
the 16th and draws an *organic* 429: unavoidable race exposure
(§2), not an injected stimulus. Timing script per B12: the
reservation-to-receipt interval is forced by scripted delay.
Asserts: recovery machinery survives it (per M8's assertions). Additionally *records* what
nonzero headroom would have bought at each contention level — the
headroom-zero decision's evidence base, so any future debate
happens over data. Cites: headroom entry, N23, P-A.

**M10. Agent-loop stress.** [phase-swept]
Pathological caller pressure: hundreds of enqueues, cancellations,
reprioritizations, sustained for many simulated minutes. Asserts:
wire stays paced — spacing floor never violated, in-flight ≤ 2,
G1 holds; the fuse does **not** trip (false-positive absence under
saturation — its true-positive path is X1); the queue drains to
completion; cancelled callers get prompt resolution. Cites: P-B,
N2, N4.

**M11. Layer-1 ceiling and Cloudflare-shaped terminal.**
[phase-independent]
Two parts. (a) The mock's layer-1 emulation enforces the B10
ceiling rules (§7; inferred lane — Cloudflare's real rules are
opaque, N1), which trip into a Cloudflare-shaped failure:
403 + `cf-mitigated: challenge`, `Server: cloudflare`, HTML body,
no rate-limit headers, no `Retry-After` (N2 made executable; the
challenge signature is the external-lane candidate claim). The
compliant client — 250 ms floor ⇒ ≤240 req/min — never trips it.
(b) Stimulus: the mock injects a Cloudflare-shaped reply directly.
Asserts: the client recognizes the shape generally and treats it
as a halt-shaped terminal condition — zero retries, halt
published, pending errored. Cites: N1, N2, N3, N4, P-B.

**M12. 4xx-tripwire obligations.** [phase-independent]
The mock models the client's documented obligations, not the
server's opaque threshold. Injected 401: zero retries. Injected
generic 4xx: no retry loop. 429: at-most-one-retry-then-escalate
(M8's ladder). All 4xx responses feed the tripwire counter
(trip logic itself is C4). Server-side restriction behavior is U2.
Cites: the 4xx-budget candidate claim (DOC lane), N10.

**M13. Gate structure on the wire.** [phase-independent]
Makes the addendum's gate definition executable from the mock's
viewpoint: never more than 2 ordinary requests in flight; a HEAD
never overlaps any other request (N18); once a HEAD is waiting, no
new ordinary permits are issued (writer preference); ordinary
permits granted in arrival order (FIFO — no lane starvation);
HEADs are ordinary citizens of the wire discipline — subject to
the spacing floor and counted by the fuse at the transport
boundary (N2's incident was a HEAD flood). Timing script per B12:
overlap windows are forced by scripted service delays.
Cites: N18, P-B, D5, the gate-definition addendum.

### Core-property scenarios (pure functions, proptest)

**C1. Padding arithmetic.** For arbitrary histories, rule
definitions, and phases: a granted reservation never falls inside
a saturated window as measured on *any* server phase, given
full-bucket padding (N13). The property quantifies over φ — this
is the core-side mirror of §3's sweep, without the mock.

**C2. Header parsing and shape validation.** Round-trip and
adversarial properties over header strings: two-triplet shapes
with strictly increasing periods parse to `RulePair`; one-, three-, and
malformed-triplet inputs return typed errors (never panic, never
index out of bounds — the N20 lesson); missing headers are typed
errors, not empty lists.

**C3. Fuse trip logic.** Pure function `(dispatch history, now) →
Tripped | Ok`. Properties: never trips on any trace at or below
the spacing-implied wire maximum (240/min); always trips at or
above the ceiling (~500/min, derived: strictly between 240/min and
the ~1000/min known-bad, N2); trip is latched.

**C4. 4xx tripwire logic.** Same shape as C3 over a windowed 4xx
counter; shares the fuse's halt semantics.

**C5. Lifecycle invariants.** Properties over interleavings of the
`Queued → Reserved → Dispatched → Observed/UnknownOutcome`
lifecycle (`core-design.md`): rolling back an undispatched
reservation restores policy state exactly; an `UnknownOutcome`
stays counted and ages out only by window passage; no
interleaving of reserve/rollback/observe double-counts or
loses a send; a deliberately generated accidental token
abandonment (dropped unconsumed) is detected by the debug drop
bomb *and* leaves engine state conservatively safe — the entry
stays counted, ages out, never double-counts (core-design
abandonment semantics).

### Fault-injection and structural (X-series)

**X1. Fuse fault-injection.** Pacing deliberately disabled;
transport boundary intact. Asserts: the fuse — counting actual
dispatch attempts at the transport boundary, incremented
immediately before hand-off, immutable to scheduling logic —
halts dispatch; pending deque errored back; `Halted` published.
This is the lane upgrade: the wire-level true positive is tested,
not declared-untested. Cites: fuse addendum (finding 6).

**X2. Transport boundary structural test.** The HTTP client is a
private field of one transport module; no second construction or
send path exists. Enforced by visibility (privacy is compile-time
in Rust) plus a structural test pinning it. A send bypassing the
boundary is an architectural failure outside the threat model.

### Declared-untested register (U-series)

**U1. Remap triggers.** Proactive provisionality at auth
transitions — dropped from scope; reactive handling (M5) is the
tested surface.

**U2. Server-side 4xx restriction behavior.** Threshold opaque, no
incident data; obligations tested (M12), server response untested.

**U3. Legacy bucket resolution.** Conditional on `Assumed(60s/60s)`
(§1). The sanctioned live-validation instrument is the designed
path to measured-lane evidence; executing it is not a spike gate
(terminal-condition addendum). **Named hypothesis (Tom,
2026-08-09): legacy burst resolution = 5s** — a designated
validation-run target under the logged run protocol (sufficiency
trials on the Account `30:60:60` window with 5s padding,
phase-randomized, halt-on-first-violation). CODE-lane prior: the
C++ 75s cutoff has effectively run 5s padding on this window for
years without observed violations. Asymmetry: one 429 falsifies
the hypothesis decisively; passing runs only accumulate
phase-swept confidence (N15 — quantization bites
intermittently). The shipped assumption stays 60s/60s until
evidence lands; the parked N14 ask to GGG (which now covers the
legacy resolutions) may retire this hypothesis for free and gets
a head start before runs are spent on it.

**U4. Real layer-1 rules.** Deliberately uncharacterized (N4
strategy); M11's ceiling number sits in the inferred lane.

---

## §6. Pass/fail criteria (agenda item 2 — headroom-free)

Headroom is zero (reconciliation log), so these thresholds carry
no headroom term anywhere.

- **G1 — zero client-caused violations.** Across every mock-judged
  scenario, every swept phase, every seed: the mock's independent
  counters record no violation attributable to client-scheduled
  traffic (§2 vocabulary: follow-on violations included;
  unavoidable exposure excluded — bounded and harness-attributed
  per §2). One counterexample fails the spike
  question; the failing (seed, φ) is recorded.
- **G2 — layer-1 ceiling never tripped.** The mock's rolling-window
  ceiling rules (M11a; both B10 rules) are armed in *every*
  mock-judged scenario, not just M11; no client-scheduled traffic
  trips either.
- **G3 — bounded per-dispatch over-delay (work conservation).**
  Whenever a request is queued and eligible — the padded-safe time
  (computed by the harness from the scenario script plus the
  mock's observation log, both client-independent: residue and
  phantom debt are mock-side facts B13 records), the spacing
  floor, and permit availability have all passed — the client
  dispatches within **ε = 500 ms simulated**. Intervals under
  episode confirmation, cooldown, suspension, halt, or probe
  exclusivity are excluded from the oracle and enumerated per
  scenario (external review F7 — the harness excludes what it
  cannot independently model rather than duplicating the core).
  **Exclusions must be authorized** by the scenario script or by
  independently observable evidence — never by client-reported
  state alone (external review round 3, closing a self-exemption
  loophole: a broken client must not remove its own delay from
  measurement by entering cooldown unnecessarily). Authorization
  sources: probe exclusion begins when the script routes traffic
  to an unknown endpoint; episode exclusion begins with an
  observed or injected 429; cooldown exclusion begins with a
  scripted malformed response; halt exclusion begins with the
  scripted Cloudflare reply. A refusal state the client enters
  without such authorization does not suppress G3 — it fails G5.
  Catches trivially-safe-by-being-slow. Armed in every mock-judged
  scenario, like G2; M2 and M10 are the binding stress
  measurements. *Draft number: tighten after the first
  implementation lands if the actor's scheduling makes a smaller
  ε reliable under paused time.*
- **G4 — scenario-level duration bound.** M2's total duration is
  ≤ **1.05×** the theoretical padded minimum for its queue depth
  and policy (harness-computed, full N13 padding, spacing floor
  included). *Draft multiplier, same revisit rule as G3.*
- **G5 — scenario-level assertions.** Every scenario's own
  assertions pass — the injected-stimulus ones (M8 retry timing,
  M8 escalation, M11b halt, M12 obligations, M3/M4 clean-failure
  semantics) and equally the structural ones (M1 per-endpoint
  HEAD discipline, M5 stale-mapping cap, M10
  drain-to-completion, M13 gate shape). An unauthorized
  client-entered refusal state — cooldown, suspension, or halt
  with no scripted trigger — is a G5 failure in any scenario
  (G3's authorization rule). No scenario passes on G1/G2 alone if
  its own assertions fail; a stimulus scenario in particular
  cannot pass on G1 alone. (Generalized from
  stimulus-only in the 2026-08-09 first-eyes review — the
  non-stimulus assertions previously had no covering gate.)
- **G6 — reproducibility.** Any failure anywhere reports a
  reproduction record sufficient to re-run it deterministically:
  (seed, φ) mandatory for swept and property-generated tests;
  optional for phase-independent and structural checks, which may
  have neither (external review F8).

Verdict scoping (§1): G1–G6 over the four OAuth policies support
the unconditional verdict; the same gates over
`backend-item-request-limit` support the conditional legacy
verdict, stated with its assumption.

---

## §7. Mock fidelity budget (agenda item 4 — decided 2026-08-09)

Fixed inputs, respected throughout: the mock is an in-process
service sharing the client's paused tokio runtime (reconciliation
constraint), and §3's independence rules are the floor under every
line below — nothing here relaxes them. The budget criterion is the
charter's scoping rule applied to the mock: a behavior is in scope
iff some scenario's verdict needs the mock to reproduce it;
everything else stays out until a scenario pulls it in. Every B/O
item names its consumers, so a future scenario change re-runs this
budget mechanically instead of by taste.

### §7.1 The judging interface

The mock sits at the **transport-trait boundary**, not on a socket.
The client's transport is a trait (`async fn send(Request) ->
Result<Response, TransportError>` over `http`-crate types; the
error arm is what feeds `Dispatched → UnknownOutcome` in the
lifecycle); production implements it with
the real HTTP client (the private field X2 pins), the harness
implements it with the mock. Consequences:

- No sockets anywhere in the test path. This retires the charter's
  pre-reconciliation "small HTTP server, e.g. axum" sketch: real
  socket IO under `tokio::time::pause` defeats auto-advance — the
  runtime cannot distinguish idle-waiting-for-time from
  waiting-for-a-peer, so paused-time determinism (the reason the
  in-process constraint exists) would be forfeit. The X1/X2
  boundary is unaffected: the fuse counts hand-offs to the
  transport trait, whichever implementation is behind it.
- **Verbatim header strings cross the boundary.** The mock emits
  real header name/value strings
  (`x-rate-limit-account: 15:10:60, 30:300:300` alongside
  `x-rate-limit-policy: stash-request-limit` — headers are named
  by *rule*, not by policy, N5), never pre-parsed structs — the client's production parser runs
  in every mock scenario, so M3/M4's parse assertions exercise the
  same code path reality does. §3's "header serialization types"
  allowance is hereby pinned to the `http` crate's vocabulary
  types (`HeaderMap`, `StatusCode`, `Method`); the mock's window
  arithmetic lives in its own module sharing nothing else.
- **All mock timing reads the paused clock.** Windows, restriction
  expiry, and service delays are driven by the same
  `tokio::time` instants the client sees; nothing in the test
  path touches wall time.
- **Layering for the reusable-artifact outcome.** The mock's
  counter engine (the §3-independent arithmetic module) stays
  cleanly separated from the thin trait-impl delivery shim. If the
  conformance suite outlives the spike (charter's reusable-artifact
  bullet), wrapping the same engine in a standalone HTTP server to
  acceptance-test other clients — including the C++ one — is a
  delivery-shim job, not a rewrite.

> **Idiom: a trait is the seam.** Where C++ injects behavior via
> virtual base classes or template parameters, Rust uses traits,
> consumed one of two ways: generically (`Client<T: Transport>` —
> static dispatch, the concrete type compiled in) or as a trait
> object (`Box<dyn Transport>` — dynamic dispatch through a
> vtable). The spike takes the generic form: which transport is in
> play is known at compile time per build (mock in tests, real in
> production), nothing swaps at runtime, and `async fn` in traits
> composes more cleanly with generics than with `dyn`.

### §7.2 Must-reproduce (B-series)

| # | Behavior | Source | Consumers |
|---|---|---|---|
| B1 | Full header protocol emission, verbatim strings: policy, rules, per-rule limit and state triplets, `Retry-After` on 429 | N5 | every M; M3/M4 parse paths |
| B2 | Black-box windowed counters (per §3) with *organic* 429 on violation and restriction enforcement for the rule's restriction period | N5, §2 | G1 in every scenario; M8 follow-on detection |
| B3 | Bucket quantization: server-owned φ, most-adversarial model (see below); resolutions Known(5s/60s) for the four OAuth policies, legacy instantiated at Assumed(60s/60s) per §1 — no legacy-resolution sweep (U3's instrument is the evidence path, not the mock) | N11–N13, N12, §1 | all phase-swept scenarios |
| B4 | Post-increment 1:1 state tracking | N25 | M2 |
| B5 | HEAD semantics: 204 (API) / 200 (legacy) + full headers, non-counting; scriptable partial-header degradation | N24, N20 | M1, M13; M3 |
| B6 | Pre-loadable counters (cross-session residue) | N24 | M1 |
| B7 | Five-policy topology with the N23 definitions as the default fixture; endpoint-label → policy routing; per-rule independent windows including the legacy Account+Ip pair; scriptable synthetic policies | N23, N6 | M4, M6; every "other policies unaffected" assertion |
| B8 | Mid-session policy mutation, scripted: redefinition/shrink and rename | N9 | M6, M5 |
| B9 | Phantom counter increments, scripted, bursty shape (corrected threat model — no drizzle) | N23, N24 | M7, M9 |
| B10 | Layer-1 ceiling, **two rules**, both armed in every scenario (G2), both tripping into the Cloudflare-shaped failure: burst **20 req / rolling 1 s** (sensitivity tripwire, derived from the declared defense — see note) and **1000 req / rolling 60 s** (N2 made executable); Cloudflare-shaped reply generation (403, `cf-mitigated: challenge`, `Server: cloudflare`, HTML body, no rate-limit headers, no `Retry-After`) | N1–N3 | M11, G2 |
| B11 | Stimulus injection channel: scripted 429 / 401 / generic-4xx / Cloudflare replies regardless of counter arithmetic | §2 | M8, M11b, M12 |
| B12 | Deterministic scriptable per-response service delay; the default is a **placeholder** (~50 ms simulated) until the §7.4 fixture lands, then re-anchored to the capture's median `sent→received` (rounded). The default only prevents vacuous zero-overlap tests — M5, M8, M9, and M13 must each specify an explicit deterministic delay/barrier schedule (timing script), because their verdicts depend on forced reordering the default cannot guarantee (external review F9) | §7.1 | M5, M8, M9, M13 |
| B13 | Observation log: per-request arrival instant, method, endpoint label, bucket assignment, counter values, verdicts, plus (seed, φ) and a per-request correlation identity (test-only header) linking mock arrivals to client dispatch records — the attribution evidence for §2's transition-exposure category | §3, G6 | M13/M10 wire-shape assertions (spacing floor, FIFO, in-flight); G6; §2 attribution |
| B14 | `Date` header emitted, consistent with the mock's clock, zero skew | N5 context | C1 cross-check (skew itself is O5) |

Notes on the load-bearing rows:

- **B3, the quantization model — a decision, and a candidate
  N-claim finding.** N11–N13 confirm buckets exist, give their
  resolutions, and give the safe margin, but do **not** specify the
  quantization semantics (when a hit's age is measured, when it
  leaves the window). The mock implements the *most adversarial
  reading consistent with the claims*: a hit's effective timestamp
  is rounded **up** to its bucket's end, understating its age by up
  to one bucket, so it leaves the window at the latest instant
  N13's full-bucket margin still covers (a hit at time `h` with
  bucket `r` and period `P` expires no later than `h + r + P` —
  exactly what waiting `period + bucket + buffer` protects
  against). Restriction expiry is quantized the same way: the
  violation's timestamp rounds up, so a retry at `Retry-After`
  alone can land inside the still-active restriction. **Entry is
  never quantized**: state increments immediately and 1:1 — N25
  pins this — so only expiry carries the adversarial rounding. Rationale: if full-bucket
  padding (N13) survives the harshest consistent model at every φ,
  it survives reality under any milder one — the mock asserts an
  upper bound on the threat, matching the layer-1 ceiling's
  philosophy. The under-specification itself is a **candidate
  N-claim clarification** for the result doc's register (charter
  step 1: mock-writing exposing an N-claim ambiguity is a finding).
  Adversarial restriction expiry is also what makes M8's N19
  assertion load-bearing: a retry at `Retry-After` alone *can*
  violate in the mock, so waiting `Retry-After + bucket + buffer`
  is tested, not decorative.
- **B10, the ceiling rules (amended 2026-08-09, veto-point
  review — the burst rule is Tom's).** Two rules with two jobs.
  The per-minute rule, 1000 / rolling 60 s, is the executable
  citation: N2's "over a thousand requests in a minute" is the
  only evidence-anchored number, and it is nearly untrippable by
  construction (4× the spacing-implied 240/min, sustained for a
  minute) — documentary, not a tuned threshold. The burst rule,
  20 / rolling 1 s, is B3's adversarial philosophy applied to
  layer 1: the threat is burst-shaped (N2) and Cloudflare's real
  rules are opaque (N1), so the mock asserts the tightest ceiling
  the client's *declared* defenses still clear with margin —
  floor-compliant traffic fits at most 4–5 sends in any rolling
  second (250 ms floor), giving ≥4× headroom at 20. This
  globalizes burst detection: a floor-violating bug becomes
  visible to G2 in every scenario, not only where the explicit
  spacing-floor assertion runs. Caveat, recorded so the ceiling is
  never mistaken for the floor's enforcement: a floor-broken
  client pacing at a sustained ~8/s evades the fuse (~500/min)
  and both ceiling rules — B13's wire-shape assertion is the
  binding detector; the ceilings are the mock-judged backstop.
  Both numbers are inferred-lane and neither models Cloudflare
  (U4). A mock ceiling defends nothing at runtime — the
  client-side floor and fuse are the defense (charter); what the
  tight rule buys is test sensitivity.
- **B12 refines the scaffold's out-of-scope candidate** rather than
  contradicting it: *stochastic* jitter stays out (O2), but with
  zero service time every exchange is instantaneous — no two
  requests ever overlap, and M13's in-flight-cap and
  HEAD-exclusivity assertions pass vacuously. A deterministic
  scripted delay is the minimum fidelity that makes concurrency
  observable at all.
- **B7 scope note:** both legacy rules are enforced as independent
  windows over the single client-under-test request stream.
  Distinct scope *semantics* (traffic from other accounts/IPs) are
  out (O4) — B9's phantom channel reproduces every effect the
  counters can observe.

### §7.3 Out of scope (O-series)

| # | Excluded | Rationale | Re-entry trigger |
|---|---|---|---|
| O1 | Sockets, TLS, connection reuse, HTTP/1.1-vs-2 framing | interface decision (§7.1) | productization, not the spike |
| O2 | Stochastic latency/jitter | B12's deterministic delay covers every scenario need; no verdict depends on transit randomness | a scenario whose verdict does |
| O3 | Request/response payloads, incl. synthdata userstores | the client under test never parses bodies; sole exception is B10's Cloudflare HTML signature | a scenario judging body handling |
| O4 | Multi-account / multi-IP scope semantics | B9 reproduces all counter-observable effects; client reconciliation is scope-blind (charter) | never for the spike |
| O5 | Server `Date`-header skew | mock emits zero-skew `Date` (B14) | **conditional, already armed:** C1 shows the arithmetic is sensitive to skew |
| O6 | Header case/order adversarial variants | C2's property domain (adversarial generation at the parser); mock emits canonical lowercase | none — covered elsewhere |
| O7 | Auth of any kind (tokens, POESESSID, OAuth flows) | no credential ever appears in mock traffic or fixtures — §4 hygiene by construction | out for the spike (charter flags OAuth as a later phase) |
| O8 | Server-side 4xx restriction (U2), real Cloudflare rules (U4), forum regime (D5, ungated), unlimited endpoints | declared elsewhere; listed to close the loop | per their registers |

### §7.4 Calibration: capture replay

One sanitized fixture (the July 18, 2026 capture, 132 records)
enters under the §4 contract. The replay test drives the capture's
*relative dispatch timestamps* through the mock's counters, swept
over φ:

- **Initialization** (external Q&A, 2026-08-09): replay seeds the
  mock's counters from the capture's boot-HEAD state headers via
  the core-design reconciliation mechanism (phantoms at t₀,
  pessimistic). Boot-HEAD records are *initialization evidence*,
  not replayed counter-producing requests — residue must never be
  applied twice (once as seed, once as arrival).
- **Gate: zero violations at every φ.** Real, observed-compliant
  traffic must be judged compliant — this catches a mock stricter
  than reality. The C++ client's full-bucket padding should survive
  even B3's adversarial model at every phase; if some φ fails, that
  is a **finding to adjudicate** (mock arithmetic bug vs. the
  adversarial model exceeding what N13's padding covers) — never
  something to tune away silently.
- **Diagnostic, not a gate:** at the capture's saturation points
  the mock's counters should agree with the recorded `15/15` and
  `30/30` states (N25/N26 grounding). Exact per-response state
  matching across the real server's unknown φ is not generally
  achievable; mismatches inform, they don't fail.

Lane bookkeeping: mock verdicts remain measured-against-model; the
replay test is the piece that grounds the model in the observed
lane (charter step 2).
