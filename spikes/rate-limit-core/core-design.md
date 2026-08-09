# Core design sketch: types and transitions (agenda item 3)

Status: agenda-item output doc per the sibling-docs convention in
`design-brief.md`. Consolidated 2026-08-09; resolves plan-review
finding 5 (the request lifecycle). **Expected to be superseded by
code** — when the Rust types land, they are the authority and this
doc becomes history. Idiom notes are deliberate: this spike doubles
as Tom's first substantial Rust work (charter provenance note).
Amended later the same day (simplification review, Tom-approved):
`Effect::RetryAt` removed — 429 retry timing folded into policy
state so `try_reserve` remains the single scheduling authority.

Scenario IDs cite `scenarios.md`; N-numbers cite
`docs/design/network-ground-truth.md`.

---

## 1. The core's contract

Sans-IO: the core is a pure state machine. No sockets, no tokio, no
clock reads — every method takes `now` as a parameter. Inputs are
(current state, `now`, parsed headers or commands); outputs are
reservations, typed errors, and effect values the shell interprets.
This is what makes C1–C5 cheap: proptest drives the core with
synthetic time and adversarial inputs, no runtime needed.

**Reservations, not predictions** (reconciliation log): deciding to
send *records* the send in policy state at decision time. There is
no "when would it be safe?" query for a shell to act on later — a
query-shaped core invites check-then-act races in any shell. The
actor asks to reserve; the answer is a reservation or a
not-before time.

> **Idiom: `&mut self` as the serialization primitive.** The core
> is a plain struct; every mutating method takes `&mut self`. Rust
> allows exactly one mutable borrow at a time, so a single owner —
> the actor task — is the only thing that can call these methods,
> checked at compile time. The "single serialized scheduling
> authority" is ownership, not a lock. No `Mutex` appears anywhere
> in the core.

## 2. Types

Sketch-level Rust; names will shift in implementation.

```rust
// Newtypes: zero-cost wrappers that make argument mixups a type
// error. A SimInstant cannot be passed where a Duration belongs,
// and policy names can't be confused with endpoint labels.
struct SimInstant(u64);          // milliseconds on the injected clock
struct PolicyName(String);
struct EndpointLabel(String);    // the D5 five-endpoint vocabulary

struct Window {
    max_hits: u32,
    period: Duration,
    restriction: Duration,
}

// Shape invariant, enforced at parse time: exactly two windows,
// burst.period strictly < sustained.period. If a RulePair exists,
// the invariant holds — no runtime re-checking.
struct RulePair {
    burst: Window,
    sustained: Window,
}

// Bucket knowledge is provenance-typed, separate from shape (§1 of
// scenarios.md). Known cites N12; Assumed is explicit and
// replaceable, never a default the code manufactures.
enum Resolution {
    Known(Duration),     // N12: 5s burst / 60s sustained, four OAuth policies
    Assumed(Duration),   // legacy: 60s/60s, conditional verdict lane
}

struct BucketModel {
    burst: Resolution,
    sustained: Resolution,
}

struct Rule {
    scope: RuleScope,            // Account | Ip — parsed, but reconciliation
    pair: RulePair,              // is scope-blind (charter: no per-scope
    buckets: BucketModel,        // phantom machinery)
}

struct Policy {
    name: PolicyName,
    rules: Vec<Rule>,            // legacy carries Account + Ip (N23)
    history: History,            // shared across the policy's rules; each
                                 // rule looks back its own max_hits
}
```

> **Idiom: enums are sum types.** `Resolution` is either `Known`
> *or* `Assumed`, each carrying data. `match` on it is exhaustive —
> adding a third variant later forces every match site to handle
> it, at compile time. This is "make invalid states
> unrepresentable": there is no way to have a resolution that is
> both, neither, or an uninitialized default.

### Parsing: the `Result` split

```rust
enum PolicyParseError {
    MissingHeader { name: HeaderName },          // typed absence, not [""]
    MalformedTriplet { raw: String },
    UnexpectedPolicyShape { rule: String, triplet_count: usize },
}

fn parse_policy(headers: &Headers) -> Result<PolicySnapshot, PolicyParseError>
```

> **Idiom: `Result<T, E>` instead of exceptions.** A fallible
> function returns `Result`; the caller *must* do something with it
> (the compiler warns on an ignored `Result`). The `?` operator
> propagates the error upward with one character. Contrast with the
> C++ parser bug in the N20 correction: a missing header silently
> became `[""]`, passed a size check, and indexed out of bounds.
> Here absence is a *variant* — there is no code path where a
> missing header looks like data. C2 property-tests exactly this.

The shell owns the response to `UnexpectedPolicyShape` (charter:
D4-style scoped clean failure): refuse that policy's sends under
cooldown, error pending requests, publish on the watch channel. At
most one request is ever sent under an unknown shape (M4). Process
abort is reserved for the test harness. All `PolicyParseError`
variants get this **identical** behavior — `MissingHeader`,
`MalformedTriplet`, and `UnexpectedPolicyShape` differ in
diagnostics, never in response (external Q&A, 2026-08-09): one
cooldown path, one exposure bound. The variant is for telemetry
and the result doc, not for branching.

## 3. The request lifecycle (finding 5)

```
Queued → Reserved → Dispatched → Observed
                 ↘ (rollback)      ↘ UnknownOutcome
```

| Transition | Trigger | State effect |
|---|---|---|
| `Queued → Reserved` | core grants `try_reserve(now)` | the send is recorded in policy history **now** — reservation is the record |
| `Reserved → Dispatched` | shell hands the request to the transport | fuse counter increments at the transport boundary (not a core transition — see §5) |
| `Reserved → (gone)` | local dispatch failure, or caller cancels before dispatch | **rollback**: the history entry is removed; state restored exactly (C5 property) |
| `Dispatched → Observed` | response headers arrive | reconcile: parse headers, pessimistic merge (server-reported state wins when higher), remap on policy-name mismatch (M5) |
| `Dispatched → UnknownOutcome` | timeout / cancel / transport error after bytes may have been sent | the reservation **stays counted** — the server may have counted it (N24: counters are server-side). It ages out only as its windows pass |
| caller drops mid-`Queued` | dropped oneshot observed by actor | remove from deque, nothing reserved, nothing to undo |
| caller drops mid-`Dispatched` | dropped oneshot | detach the caller; the request stays counted and its response still updates state (the C++ counted-in-flight vs stopped-caller distinction) |

The asymmetry is the point: **undispatched reservations roll back;
unknown outcomes never do.** Rollback is safe only while we are
certain no bytes reached the wire; past dispatch, uncertainty is
resolved pessimistically.

Token abandonment (external Q&A, 2026-08-09): explicit consumption
is the *rule* — every ordinary actor path resolves a token through
`rollback`, `on_response`, or `on_unknown_outcome`, and for a
known-undispatched token rollback is the correct, normal action. A
token dropped unconsumed is a *bug path* with defined emergency
semantics: the reservation stays counted and ages out by window
passage — pessimistic retention, safe at the cost of throughput,
never the design intent. Enforcement: `#[must_use]` on the token
warns at construction sites; in debug/test builds the token
carries a drop bomb — its `Drop` impl panics on unconsumed drop
*unless the thread is already panicking*
(`std::thread::panicking()` guard, so a bomb firing during an
unwind cannot escalate into a double-panic abort). C5 deliberately
generates an accidental-drop interleaving and asserts both halves:
the bomb detects it, and engine state stays conservatively safe.

> **Idiom: move-only tokens.** `try_reserve` returns a
> `ReservationToken` that is not `Copy` and not `Clone`. Every
> consuming method takes it *by value* — `rollback(token)`,
> `on_response(token, ...)`, `on_unknown_outcome(token, ...)` — so
> after one of them runs, the token is gone and the compiler
> rejects any second use. Double-rollback or
> rollback-after-response is a compile error, not a runtime check.
> (This is the cheap end of the "typestate" idiom.)

### Reconciliation and phantom synthesis (external Q&A, 2026-08-09)

`History` is a timestamped deque of send instants, one per policy,
shared across the policy's rules; each rule evaluates it against
its own `max_hits`/`period`. Reconciliation in `on_response` is a
per-rule **pessimistic count-max with phantom synthesis**: for
each rule, compare the server's reported `current-hits` (N25:
post-increment) with the local in-window count; where the server
reports more, synthesize the deficit as entries timestamped `now`
— the newest possible placement, so phantoms age out latest.
Because the deque is shared, insert the **max** deficit across the
policy's rules, not the sum: one entry at `now` is in-window for
every rule simultaneously. Two properties stated deliberately:

- Synthesis may overstate rules other than the one driving the
  deficit — by design; pessimism is scope-blind (charter).
- Reconciliation is monotone: a later, lower server count never
  removes local or synthetic history. Entries leave the deque
  only by window passage or rollback of an undispatched
  reservation.

Boot residue is not a special case: M1's HEAD-reported state is
this same mechanism applied to an empty history. One mechanism
covers M1 (residue), M7 (phantoms), and post-429 reality.

## 4. Core API sketch

```rust
impl PolicyEngine {
    /// Reservation, not prediction. Recording happens on grant.
    fn try_reserve(&mut self, policy: &PolicyName, now: SimInstant)
        -> ReserveOutcome;

    /// Response observed: reconcile, and tell the shell what changed.
    fn on_response(&mut self, token: ReservationToken, now: SimInstant,
                   headers: &Headers) -> Vec<Effect>;

    fn on_unknown_outcome(&mut self, token: ReservationToken, now: SimInstant);
    fn rollback(&mut self, token: ReservationToken);
}

enum ReserveOutcome {
    Reserved(ReservationToken),
    NotBefore(SimInstant),       // earliest re-ask time; actor sleeps on it,
                                 // select!-ing against the inbox (StopSleep,
                                 // generalized)
    Refused(RefusalReason),      // shape-cooldown, escalation-suspended, halted
}

enum Effect {
    Remapped { from: PolicyName, to: PolicyName },       // M5
    ShapeCooldown { policy: PolicyName },                // M4 → shell errors pending
    EscalationSuspend { policy: PolicyName },            // M8 second consecutive 429
    CloudflareShapedReply,                               // M11b → shell halts
    RequeueForRetry,                                     // first 429 on a policy:
                                                         // re-enqueue this request
}
```

A 429 carries no retry *time* in its effect — that would hand the
shell a second scheduling path. Instead the core records the
restriction in policy state (active until `Retry-After` +
applicable bucket + buffer, N19), the shell re-enqueues the
request, and the next `try_reserve` answers `NotBefore` from that
state. Retry timing flows through the same single authority as
every other send; the once-then-escalate ladder stays in the core
(consecutive-429 count per policy → `EscalationSuspend`).

Episode semantics for that ladder (external Q&A + refinement,
2026-08-09): each recorded restriction increments a per-policy
**restriction generation**, and every `ReservationToken` carries
the generation it was granted under — generations, not
timestamps, do the attribution, because exact boundaries and
successive restrictions make timestamp ordering fragile. A 429
whose token predates the current generation joins the existing
episode: concurrent in-flight originals bounced by the same
saturation never double-count. While an episode awaits
confirmation, `try_reserve` grants at most **one**
post-restriction reservation for that policy (single retry in
flight; other callers queue behind `NotBefore`). That probe's
outcome decides:

- any non-429 response breaks the consecutive-429 chain — episode
  reset, normal concurrency resumes (its headers reconcile as
  usual; a degraded reply takes the D4 path independently);
- 429 → `EscalationSuspend` (a genuine failed retry);
- unknown outcome → the episode stays unconfirmed, the send stays
  counted (lifecycle rule), and the single probe slot reopens at
  the recomputed `NotBefore`.

`CloudflareShapedReply` carries one wrinkle (first-eyes review,
2026-08-09): the signature includes the *body* (HTML), and the
core never sees bodies. Classification is therefore the
transport/shell's job — it hands `on_response` a typed reply
classification rather than raw evidence — while the *decision*
(halt-shaped terminal condition, never a retry) stays in the core
with every other policy decision.

> **Idiom: effects as data.** The core never performs IO; it
> *returns* what should happen (`Vec<Effect>`) and the actor
> interprets — errors callers, publishes on the watch channel,
> suspends a queue. This keeps every branch of shell behavior
> reachable from a unit test that just constructs the effect.

What updates state: reservations (grant time), responses
(reconcile/remap/shrink per N9), unknown outcomes (pessimistic
keep), rollbacks (exact undo), phantom observations (pessimistic
merge, M7). What queries state without updating: none exposed —
snapshots for the watch channel are produced by the actor after
each transition, not pulled by callers.

## 5. What is deliberately outside the core

- **The fuse and 4xx tripwire counters** live at the transport
  boundary (plan-review addendum, finding 6), immutable to
  scheduling logic, pinned by X2's structural test. Their feeds
  differ (split stated explicitly in the 2026-08-09 first-eyes
  review; the earlier wording lumped both under "dispatch
  attempts", which is wrong for the tripwire): the fuse increments
  immediately before hand-off to the HTTP client; the tripwire
  increments on observed 4xx response statuses at the same
  boundary. Their *trip logic* is pure and core-adjacent (C3/C4),
  but neither counter is ever fed by reservations.
- **The spacing floor and in-flight cap** are gate properties the
  actor enforces around dispatch (M13); the core's reservations are
  per-policy and know nothing about the wire.
- **Probe eligibility is actor-owned; the core never schedules a
  HEAD** (external Q&A, 2026-08-09). There is deliberately nothing
  to reserve: HEADs do not count against policy counters (N24),
  and pre-probe the endpoint has no policy mapping to reserve
  against. Bootstrap sequence (charter component-scope entry +
  D5): unknown endpoint label → park the request in the actor's
  deque, mark the endpoint probing (exactly-once, N16, and strict
  serialization, N18, are structural in the one loop), drain the
  gate under writer preference, take exclusive occupancy, respect
  the spacing floor, dispatch. A HEAD is an ordinary citizen at
  the transport boundary: counted by the fuse, subject to the
  spacing floor and X2's single-send-path rule — N2's incident
  *was* a HEAD flood. Core involvement is limited to parsing the
  reply and seeding state via reconciliation.
- **The clock.** `SimInstant` arrives as a parameter, always.
  Nothing in the core calls `Instant::now()` — that single rule is
  what makes `tokio::time::pause` testing deterministic.

Build order stays as chartered: this state machine first, under C1–C5
and the mock scenarios; the actor shell second.
