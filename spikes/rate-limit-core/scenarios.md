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
`docs/design/network-redesign.md`. Item 4 (mock fidelity budget) is
scaffolded in §7 but not started — it is the next side-bar.

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
  fails the run.
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
Client boots, issues exactly one HEAD (N16), reads the state
header showing residue, and schedules its opening traffic against
the *remaining* budget. Sweep residue magnitude and φ; include the
boundary case where residue leaves zero budget in the current
bucket. Asserts: single boot HEAD; HEAD does not increment
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
were scheduled under the stale mapping; no client-caused violation
after the merge; remap *triggers* beyond the reactive path are U1.
Cites: N5, N9, N6.

**M6. Policy shrink mid-flight.** [phase-swept]
Limits tighten mid-session (N9), e.g. `15:10:60` → `10:10:60`,
while history already holds more than the new limit allows.
Asserts: the shrink is honored from the response that announces
it; no client-caused violation against the new limits; queue keeps
draining at the new pace (no wedge). Cites: N9, N13.

**M7. Phantom same-account hits.** [phase-swept]
Mock injects counter increments the client under test didn't cause
— occasional and bursty (a second tool launching, per the
corrected threat model; N23: Account-scoped rules), not constant
drizzle. Asserts: client reconciles pessimistically when observed
state exceeds its model (scope-blind — no per-scope machinery);
no client-caused violation. Thresholds must not be tuned against a
constant-drizzle world that doesn't exist. Cites: N23, N24, N25.

**M8. 429 recovery and escalation ladder.** [phase-swept]
Injected exogenous 429 with `Retry-After` (stimulus, §2). Asserts
(the stimulus's own assertions): the retried send waits
`Retry-After` + applicable bucket + buffer (N19), not `Retry-After`
alone; the caller eventually observes the outcome (the F57 lesson);
a second consecutive 429 on the same policy escalates — back
off / suspend the policy's queue and surface, never politely
re-knock (the 4xx double-dip makes a third knock doubly
expensive); no follow-on violation (§2). Cites: N19, N15, P-A,
the 4xx-budget candidate claim.

**M9. Phantom race at saturation (characterization).** [phase-swept]
At 14/15 a phantom consumes the last slot between the client's
reservation and the mock's receipt — the client's send lands as
the 16th and draws an exogenous 429. Asserts: recovery machinery
survives it (per M8's assertions). Additionally *records* what
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
Two parts. (a) The mock's layer-1 emulation enforces a
rolling-window burst ceiling (inferred lane — Cloudflare's real
rules are opaque, N1) that trips into a Cloudflare-shaped failure:
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
permits granted in arrival order (FIFO — no lane starvation).
Cites: N18, P-B, D5, the gate-definition addendum.

### Core-property scenarios (pure functions, proptest)

**C1. Padding arithmetic.** For arbitrary histories, rule
definitions, and phases: a granted reservation never falls inside
a saturated window as measured on *any* server phase, given
full-bucket padding (N13). The property quantifies over φ — this
is the core-side mirror of §3's sweep, without the mock.

**C2. Header parsing and shape validation.** Round-trip and
adversarial properties over header strings: two-triplet
strictly-increasing shapes parse to `RulePair`; one-, three-, and
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
loses a send.

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
(terminal-condition addendum).

**U4. Real layer-1 rules.** Deliberately uncharacterized (N4
strategy); M11's ceiling number sits in the inferred lane.

---

## §6. Pass/fail criteria (agenda item 2 — headroom-free)

Headroom is zero (reconciliation log), so these thresholds carry
no headroom term anywhere.

- **G1 — zero client-caused violations.** Across every mock-judged
  scenario, every swept phase, every seed: the mock's independent
  counters record no violation attributable to client-scheduled
  traffic (§2 vocabulary, follow-on violations included). One
  counterexample fails the spike question; the failing (seed, φ)
  is recorded.
- **G2 — layer-1 ceiling never tripped.** The mock's rolling-window
  ceiling (M11a) is armed in *every* mock-judged scenario, not just
  M11; no client-scheduled traffic trips it.
- **G3 — bounded per-dispatch over-delay (work conservation).**
  Whenever a request is queued and eligible — the padded-safe time
  (computed independently by the harness from the policy
  definition and the spec formula, not by asking the client), the
  spacing floor, and permit availability have all passed — the
  client dispatches within **ε = 500 ms simulated**. Catches
  trivially-safe-by-being-slow. *Draft number: tighten after the
  first implementation lands if the actor's scheduling makes a
  smaller ε reliable under paused time.*
- **G4 — scenario-level duration bound.** M2's total duration is
  ≤ **1.05×** the theoretical padded minimum for its queue depth
  and policy (harness-computed, full N13 padding, spacing floor
  included). *Draft multiplier, same revisit rule as G3.*
- **G5 — stimulus assertions.** Every injected-stimulus scenario's
  own assertions (M8 retry timing, M8 escalation, M11b halt, M12
  obligations, M3/M4 clean-failure semantics) pass. A stimulus
  scenario cannot pass on G1 alone.
- **G6 — reproducibility.** Any failure anywhere reports the
  (seed, φ) pair sufficient to re-run it deterministically.

Verdict scoping (§1): G1–G6 over the four OAuth policies support
the unconditional verdict; the same gates over
`backend-item-request-limit` support the conditional legacy
verdict, stated with its assumption.

---

## §7. Mock fidelity budget (agenda item 4 — scaffold only, not started)

The next side-bar. Fixed inputs it must respect:

- The mock is an in-process service sharing the client's paused
  tokio runtime (reconciliation constraint; §3).
- §3's independence rules are non-negotiable floor, not budget
  items.
- Candidate must-reproduce behaviors (from captures/N-claims, to
  be decided): post-increment 1:1 state tracking (N25); HEAD 204 +
  full headers, non-counting (N24); cross-session residue (N24);
  the five-policy topology and the legacy Account+Ip rule pair
  (N23); 429 + `Retry-After` emission on violation (N5).
- Candidate out-of-scope: network jitter/latency modeling, TLS,
  connection reuse, HTTP/2 framing, multi-account topology,
  server `Date`-header skew (unless C1 shows the arithmetic is
  sensitive to it — then it moves into scope).
- Capture-replay fixtures enter under the §4 sanitization
  contract; replay grounds the model in observed reality
  (measured-against-model vs observed lanes, charter step 2).

Budget decisions land here when the side-bar runs.
