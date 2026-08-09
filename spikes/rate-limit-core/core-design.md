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
cooldown — or the *endpoint's*, when no policy mapping was ever
established, e.g. a malformed boot HEAD (`RefusalTarget`) — error
pending requests, publish on the watch channel. At
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

`History` is one deque per policy, shared across the policy's
rules; each rule evaluates it against its own `max_hits`/`period`.
Entries carry identity and provenance (external review F3):

```rust
struct HistoryEntry {
    id: EntryId,          // the token holds this; rollback removes by id
    at: SimInstant,
    kind: EntryKind,      // LocalReservation | Synthetic
}
```

Identity is not optional polish: under paused time, same-instant
entries are the *common* case (two reservations in one tick; a
synthetic entry colliding with a real send at `now`), so "remove
the reservation exactly" is only well-defined by id. Tokens never
reference `Synthetic` entries, and provenance also feeds M9's
characterization (synthetic vs. real hits per contention level).
Reconciliation in `on_response` and `on_probe_response` (both
delegate to one internal function) is a per-rule **pessimistic
count-max with phantom synthesis**: for
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

    /// Reserved-response entry point: reconcile, decide disposition.
    fn on_response(&mut self, token: ReservationToken, now: SimInstant,
                   response: &ObservedResponse) -> Transition;

    /// Probe entry point (external review F1): HEADs have no
    /// reservation by design (N24 — nothing to reserve), so the
    /// token-consuming path cannot serve them, and an optional token
    /// would weaken the lifecycle contract. Both entry points
    /// delegate to one internal reconciliation function.
    fn on_probe_response(&mut self, endpoint: &EndpointLabel,
                         now: SimInstant, response: &ObservedResponse)
        -> Transition;

    /// Returns a Transition like the response entry points: a final
    /// confirmation attempt ending unknown escalates (confirmation
    /// matrix below), which the shell must be told about.
    fn on_unknown_outcome(&mut self, token: ReservationToken,
                          now: SimInstant) -> Transition;

    /// Returns nothing by design: rollback's only effect is internal
    /// state restoration (remove by EntryId, F3); the actor
    /// publishes a watch snapshot after every mutating call anyway.
    fn rollback(&mut self, token: ReservationToken);
}

enum ReserveOutcome {
    Reserved(ReservationToken),
    NotBefore(SimInstant),       // earliest re-ask time; actor sleeps on it,
                                 // select!-ing against the inbox (StopSleep,
                                 // generalized)
    Refused(RefusalReason),      // shape-cooldown, escalation-suspended, halted
}

// Normalized input (external review F2): everything the core needs
// about a reply, in one place. Headers stay RAW — parsing is the
// core's job (C2; §7.1's verbatim-strings decision depends on it);
// `classification` carries only what the core cannot see (the body
// shape, judged at the transport).
struct ObservedResponse {
    status: StatusCode,
    headers: HeaderMap,
    classification: ReplyClassification,   // Normal | CloudflareShaped
}

// Structured output (F2 + the Vec<Effect> finding): exactly one
// disposition — invalid combinations (requeue + suspend, remap +
// cooldown) are unrepresentable, and interpretation order is a
// non-question — plus supplementary notifications.
struct Transition {
    disposition: Disposition,
    notifications: Vec<Notification>,
}

enum Disposition {
    CompleteRequest,                 // ordinary response: deliver the
                                     // outcome to the request's caller
    ProbeReady,                      // successful probe: mapping seeded,
                                     // release the endpoint's parked
                                     // requests (a HEAD has no caller —
                                     // one name per meaning, so actor
                                     // confusion is hard to express)
    Requeue,                         // first 429 on a policy (M8)
    Refuse {                         // M3 degraded probe, M4 shape,
        target: RefusalTarget,       // M8 escalation suspend — the typed
        cause: RefusalCause,         // PolicyParseError travels inside
    },                               // the cause: success-plus-parse-
                                     // error is unrepresentable
    Halt,                            // M11b Cloudflare-shaped terminal
}

// A refusal needs a target even when no policy mapping was ever
// established — a malformed boot HEAD dies before yielding a policy
// name, and the D4 blast radius is then the endpoint.
enum RefusalTarget {
    Policy(PolicyName),
    Endpoint(EndpointLabel),
}

enum Notification {
    Remapped { from: PolicyName, to: PolicyName },   // M5
    StateChanged,                                    // watch-snapshot cue
}
```

Entry-point invariant (documented at sketch level; implementation
may split the return types per entry point if the invariant earns
compile-time teeth): `on_response` never yields `ProbeReady`;
`on_probe_response` never yields `CompleteRequest` or `Requeue`.

**Response precedence** (external review round 4): a reply can
satisfy several conditions at once — a 429 with malformed policy
headers, a 429 classified Cloudflare-shaped, a 2xx carrying both a
remap and an unexpected shape — and `Disposition` is mutually
exclusive, so evaluation order is part of the contract:

1. Cloudflare classification → `Halt` (supersedes everything);
2. malformed / out-of-model policy observation → `Refuse` — this
   is the safety-sensitive case: a 429 whose headers do not yield
   a valid policy observation becomes a D4 refusal, **not** a
   retry episode, because scheduling a retry safely requires
   restriction context we provably do not have;
3. 429 with usable restriction context → episode handling;
4. other status handling (a generic 4xx with valid headers
   reconciles normally, reaches the caller as `CompleteRequest`,
   and feeds the tripwire at the boundary);
5. reconciliation and remap notifications, where valid.

C2 and M8 carry combined cases so this ordering is executable,
not prose.

**Probe outcome table** (external review round 4 — completes
`on_probe_response`, which cannot return `Requeue`):

| Probe outcome | Disposition |
|---|---|
| valid 2xx + policy observation | `ProbeReady` |
| malformed or out-of-model policy | `Refuse` (endpoint target, D4) |
| 429 **with** valid policy observation | `ProbeReady` — a 429 reply still carries the full header set (N5), which is exactly the mid-window state the boot HEAD exists to discover; the restriction is recorded in the seeded policy state and the first GET waits out `Retry-After` + bucket + buffer through `try_reserve` as usual. No endpoint-owned retry timing is needed |
| 429 **without** valid policy observation | `Refuse` (endpoint target, D4) — precedence rule 2 |
| 5xx | `Refuse` (endpoint target, D4 cooldown); no ordinary GET released |
| transport unknown outcome | `Refuse` (endpoint target, D4 cooldown); no ordinary GET released |
| Cloudflare-shaped | `Halt` |

The 429-with-valid-observation row, pinned (review close-out —
this list removes the ambiguity over whether the first GET is
ordinary traffic or recovery confirmation): that probe 429

- feeds the 4xx tripwire (a 4xx observed at the boundary);
- seeds the restriction into the newly mapped policy state and
  increments its restriction generation;
- opens an unconfirmed recovery episode **without requeuing the
  HEAD** — a probe is never retried; the mapping is established;
- makes the first subsequent GET on that policy the episode's
  **single confirmation attempt**;
- escalates immediately if that GET draws a 429
  (confirmation-matrix first-attempt row);
- falls back to endpoint refusal when the observation is invalid
  (precedence rule 2).

A 429 carries no retry *time* in its transition — that would hand
the shell a second scheduling path. Instead the core records the
restriction in policy state (active until `Retry-After` + the
applicable bucket + buffer, N19), the shell re-enqueues the
request (`Disposition::Requeue`), and the next `try_reserve`
answers `NotBefore` from that state. **The applicable bucket is
the maximum configured resolution across all windows of all rules
reported for that policy** (external review F4) — pessimistic and
citable in one sentence; in both lanes today that maximum is 60 s.
The refinement of identifying the violated rule from the state
header's restriction-active flag (N5) is declined: it would buy
back at most 55 s on a rare path while adding a parsing dependency
to the safety path. M8 exercises this rule over the legacy
Account+Ip policy as well as an OAuth policy. Retry timing flows through the same single authority as
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

- only a **2xx carrying a valid policy observation** (parseable
  rate-limit headers) confirms recovery — episode reset, normal
  concurrency resumes (external review F5);
- any other non-429 outcome (5xx, 401, malformed, Cloudflare-
  shaped) **preserves the episode** while its independent path
  applies on top (D4 cooldown, 4xx tripwire, halt) — a 500 storm
  proves nothing about the limiter and must not reset the ladder —
  and **consumes a confirmation attempt** (otherwise a 500 storm
  recreates the unbounded-probe loop F6 closed);
- 429 → escalation (`Refuse`, suspend-and-surface) — a genuine
  failed retry;
- unknown outcome → the send stays counted (lifecycle rule) and
  consumes a confirmation attempt. The cap is **two attempts per
  episode** (external review F6); repeated unknown outcomes
  suggest connectivity trouble, suspend-and-surface is the right
  posture, and every unknown send stays counted so wire exposure
  stays bounded throughout.

The full confirmation matrix (external review clarification; M8
exercises it case by case — 429 on the *first* attempt escalates
immediately and never earns a second attempt):

| Attempt | Outcome | Result |
|---|---|---|
| first | 2xx + valid policy observation | reset |
| first | 429 | escalate immediately |
| first | unknown, or other non-429 | one final attempt permitted (episode preserved; independent paths apply) |
| final | 2xx + valid policy observation | reset |
| final | anything else (429, unknown, other non-429) | escalate |

`Halt` is terminal and outside the matrix (external review round
4): a Cloudflare-shaped reply at *any* point halts immediately
under precedence rule 1 — Cloudflare never receives the otherwise
"permitted" final attempt.

`CloudflareShapedReply` carries one wrinkle (first-eyes review,
2026-08-09): the signature includes the *body* (HTML), and the
core never sees bodies. Classification is therefore the
transport/shell's job — it hands `on_response` a typed reply
classification rather than raw evidence — while the *decision*
(halt-shaped terminal condition, never a retry) stays in the core
with every other policy decision.

> **Idiom: transitions as data.** The core never performs IO; it
> *returns* what should happen (a `Transition`) and the actor
> interprets — errors callers, publishes on the watch channel,
> suspends a queue. `Disposition` is one enum, so mutually
> exclusive outcomes are exclusive *by construction*: requeue plus
> suspend cannot be expressed, and parse errors travel inside
> `Refuse`, so success-plus-error is equally
> unrepresentable. "Make invalid states unrepresentable," applied
> to the output side. Every branch of shell behavior stays
> reachable from a unit test that just constructs the transition.

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
  but neither counter is ever fed by reservations. The fuse
  carries **two clauses** (burst clause approved at review
  close-out, 2026-08-09): sustained ~500/min, and burst — at most
  10 dispatches in any trailing half-open 1 s window, the 11th
  trips — HEAD and ordinary dispatches counted identically; C3
  pins both, X1 fault-injects each shape separately.
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
