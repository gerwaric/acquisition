# Design brief: rate-limit-core spike

Status: design charter for a brainstorming/design session — not an
execution brief. Committed on `spike/rate-limit-core` (never merged;
this branch is a snapshot answering one question). The spike ends
when its distilled result doc lands in `docs/redesign/topics/` on
`redesign`, with the register row updated.
Provenance: prepared 2026-08-08 in Tom's session with Claude,
distilling that session's approach discussion. Nothing here is
decided — the design session's job is to pressure-test and refine
this, with Tom hands-on (this spike doubles as his first substantial
Rust work; explain idioms as they arise, don't just emit them).

## The question (spike register, `docs/redesign/README.md`)

Can a Rust client demonstrably honor the N-claims in
`docs/design/network-ground-truth.md` under burst load, as a single
serialized gate?

Context: Acquisition was once blacklisted by GGG for rate-limit
violations; Tom wrote the current C++ limiter that ended that. This
spike de-risks the Rust-core direction of ADR 0003
(`docs/adr/0003-rewrite-vs-evolve.md`, proposed;
`docs/redesign/topics/migration-order.md` for the strangler plan).
The bar is not "Rust can rate limit" — it is "a Rust implementation
makes the N-claims *executable*."

## Required reading before designing

- `docs/design/network-ground-truth.md` — the N-claims; the spike's
  entire specification derives from these.
- `docs/design/network-redesign.md` — the accepted C++ design;
  especially the single-serialized-gate decision and D5 (endpoint
  scope: exactly which traffic is gated; the forum regime is
  deliberately ungated — see `docs/redesign/topics/shop-write-path.md`
  §2, and do not "fix" that).
- `src/ratelimit/` — the reference implementation, including
  `networkcapture` (credential-clean captures; a fixture source).
- `tools/synthdata/` (merged via PR #195) — RePoE-driven synthetic
  userstore generation, if realistic payloads are needed.
- `inputs/rate-limiter-design-brief.md` — an independently produced
  design (written without access to this codebase; see its
  provenance header). Reconcile it with this charter's starting
  position: divergences between the two are agenda items for the
  design session, not one document overriding the other, and its
  API-behavior claims must be checked against the N-claims before
  they influence the design.

## Starting position (from the session discussion — challenge it)

**Test harness first — it decides what "demonstrably" means.** The
one experiment this project can never run is burst-loading GGG's
live API. Proposed, in order:

1. **Mock GGG server** (small Rust HTTP server, e.g. axum) that
   *implements* the N-claims: emits the rate-limit header protocol,
   tracks its own windows, answers violations with 429 +
   `Retry-After`. Pass/fail is judged by the mock, not by the
   client's self-report. The mock doubles as an executable
   transcription of the ground-truth doc — if writing it exposes an
   ambiguity in an N-claim, that is a finding in its own right
   (candidate N-claim clarification, cited by number).
2. **Capture replay** — seed mock scenarios from real header
   sequences recorded by the C++ app's `networkcapture`, so the mock
   emulates observed GGG behavior, not just the documented protocol.
   (Lanes: mock results are measured-against-model; replay grounds
   the model in observed reality.)
3. **Gentle live confirmation, last** — short, low-volume, with
   Tom's explicit go-ahead; a sanity epilogue, never the evidence.

**Architecture: sans-IO core + thin async shell.** The policy engine
is a pure state machine — (policy state, clock, response headers) →
next permitted send time — no sockets, no tokio. A thin tokio actor
wraps it: one task owns the gate state, requests arrive by channel,
so "single serialized gate" is enforced by ownership rather than
discipline. (Shell shape settled in reconciliation — see the log
below for the decision, the sharpened shape, and its consequence
for the core's contract.) Consequences to verify in design: the core is unit- and
property-testable (proptest) under simulated time
(`tokio::time::pause` makes burst scenarios deterministic and fast);
Tom learns ownership on pure data before touching async lifetimes —
build order is state machine first, shell second.

**Considered and set aside (rebut if wrong):** the `governor` crate
models static client-declared quotas, but GGG's protocol is
server-driven — policies arrive in response headers and mutate
mid-flight, which is precisely the hard part; tower middleware adds
layering the spike doesn't need to make the gate legible. Revisit
tower at productization, not now.

## Design-session agenda (suggested)

0. **Interactive reconciliation first — a gate, not a formality.**
   Before any design work, discuss with Tom: walk the divergences
   between this charter and `inputs/rate-limiter-design-brief.md`,
   surface his questions, and settle what the spike is actually
   testing. This discussion is allowed to change anything below —
   including the starting position and the agenda itself. If it
   does, amend this charter and commit the revision to the spike
   branch before proceeding, so the branch records what changed and
   why. Settled divergences land in the reconciliation log below.
   Do not begin item 1 until Tom says the reconciliation is done.
1. Enumerate the scenario list from the N-claims: cold-start burst
   (upgraded in reconciliation to cold-start-with-residue, per N24
   — see the log), policy shrink mid-flight, multi-rule policies,
   429 recovery, `Retry-After` honoring, whatever else the claims
   imply. Each scenario cites its N-numbers.
2. Define pass/fail criteria precisely (zero mock-judged violations
   across all scenarios? bounded over-delay too, so the client isn't
   trivially safe by being absurdly slow?).
3. Sketch the state machine's types and transitions on paper before
   any Rust — what is policy state, what updates it, what queries it.
4. Decide the mock's fidelity budget: which observed behaviors from
   captures must it reproduce; which are out of scope. (Constraint
   from reconciliation: the mock is an in-process service sharing
   the client's paused runtime, not a standalone binary — see the
   log.)
5. Define the result doc's skeleton (goes to
   `docs/redesign/topics/`, claim lanes, pass/fail cited to
   N-numbers) so evidence collection is designed in, not bolted on.
   (The scope entry's test-lane taxonomy — mock-judged wire
   behavior / core-level property tests / declared-untested —
   feeds this.)

## Reconciliation log (agenda item 0 outcomes)

Each entry records a divergence between this charter and
`inputs/rate-limiter-design-brief.md`, the decision, and the why.
Entries land here as they are settled; the item-0 gate stays open
until Tom declares the reconciliation done.

### 2026-08-09 — Shell shape: actor confirmed (charter position)

The brief starts with `Arc<Mutex<PolicyStore>>` and treats the
actor as "the growth path if the mutex version gets awkward."
Declined — the actor is the starting position, for reasons the
brief's session (no codebase access) could not weigh:

- **Queue-as-data is a latent product requirement, and it picks
  the shape.** Agents and users will want to see and edit the
  pending queue: hundreds of stash tabs refreshing over hours,
  reprioritized mid-run when the user changes their mind, or tab
  groups refreshing at different cadences. In the mutex design the
  queue is control flow — a queued request is a local variable in
  a sleeping caller's suspended future, unreachable in principle;
  inspection/reorder/cancel forces reifying the queue plus
  distributed re-check wakeups, which converges on an actor by
  erosion. In the actor, the queue is an owned deque mutated in
  exactly one loop. The spike builds **none** of
  display/reorder/edit — the desire only picks the shape, which
  leaves those doors open at near-zero carrying cost.
- The brief's own 429 escalation ("suspend that policy's queue and
  surface it to the user") already presumes a queue something owns
  and can suspend.
- The C++ reference implementation is already actor-shaped: the
  event-loop pump drains an explicit deque
  (`src/ratelimit/ratelimitmanager.h`), with per-entry stop-token
  cancellation (`stopsleep.h`) and a QueueUpdated signal.
  Production-proven since 2023; the mutex shape has no precedent
  in this codebase.
- Enforced-by-ownership is the brief's own first idiom ("make
  invalid states unrepresentable") applied to the shell — which
  the brief relaxes to a discipline rule ("appears in at most one
  place") exactly where the stakes are highest.

Shape, sharpened from the naive actor sketch: the mpsc channel is
**command ingress** (an enum: Enqueue, Cancel, later Reorder), not
the queue — a channel mailbox is opaque and unreorderable. The
actor owns the explicit deque, `select!`s over pacing-sleep vs.
inbox so it stays responsive mid-pace (the `StopSleep` contract,
generalized), and publishes a status snapshot via
`tokio::sync::watch` after each state change. Caller cancellation
is dropped-oneshot.

Consequence for the core contract (feeds agenda item 3): core
transitions are **reservations, not predictions** — deciding to
send records the send in policy state at decision time. A
query-shaped core ("when would it be safe?") invites check-then-act
races in any shell.

### 2026-08-09 — Component scope: spacing floor, fuse, and HEAD
probing all in

The brief carries three components beyond the charter's sketch: a
global spacing floor, a global fuse, and the HEAD probe state
machine. All three are in spike scope, with rationales that amend
the seed position:

- **Spacing floor: in, with the mock asserting the threat, not the
  defense.** Correction to the discussion's premise: F58 is no
  longer dead code — the C++ redesign fixed it in phase 3 (July 20,
  2026) as the gate's `MIN_SEND_SPACING`, a 250 ms floor across all
  gated traffic, pinned on the injected clock (spec D5). The spike
  inherits that precedent number and test shape rather than
  inventing one. The mock's layer-1 emulation asserts a
  rolling-window burst ceiling that trips into a Cloudflare-shaped
  failure (no `Retry-After`, outside the polite protocol — N2 made
  executable); the gap floor is the client's declared defense
  (P-B's explicit global bound), and the two connect
  arithmetically: 250 ms ⇒ ≤240 requests/min on the wire. The
  ceiling number sits in the inferred lane — Cloudflare's real
  rules are opaque (N1). Mock-architecture consequence: mock and
  client must share one paused tokio runtime so timing assertions
  are deterministic — the mock is an in-process service, not a
  standalone binary (feeds agenda item 4).
- **Fuse: in unconditionally — the seed position's "in iff the
  stress scenario is in" was a false coupling.** A correct client's
  fuse cannot trip in the agent-loop stress scenario: the queue
  absorbs the storm and the wire stays paced, so the scenario only
  certifies absence of false positives under saturation. The fuse's
  true-positive path (limiter bug floods the wire → halt) is
  testable only by fault injection, which the mock suite does not
  carry: trip logic is a pure function (`history, now → Tripped |
  Ok`) covered by core-level unit/property tests, and the
  wire-level true positive is declared untested in the result doc.
  The fuse ceiling is derived, not picked: it must sit strictly
  between the spacing-implied wire maximum (240/min) and the
  known-bad threshold (~1000/min, N2) — ~500/min is defensible and
  citable. Actor payoff: halt semantics are clean — the loop stops
  draining, errors the pending deque back to callers, publishes
  `Halted` on the watch channel.
- **HEAD probing: in, at the actor's natural fidelity.** The
  brief's expensive machinery (`Probing` variant holding an
  awaitable handle, concurrent callers awaiting the in-flight
  probe, guarded exactly-once transitions) is the mutex-shape
  solution to a concurrency problem the actor doesn't have: one
  loop owns the endpoint map, issues one probe, and parks
  dependents in the deque it already owns — exactly-once (N16) and
  strict serialization (N18) are structural. Mock cost is modest:
  HEAD answers 204 + full headers without incrementing counters
  (N24, now observed-lane), plus one degraded-HEAD scenario that
  pins D4 (clean failure under cooldown) and exercises the
  `Result`-returning header parser in one stroke. Scenario upgrade
  for agenda item 1: the flagship cold-start scenario is
  **cold-start with residue** — N24 observed server counters
  persisting across restarts, so boot lands mid-window with budget
  already consumed; the HEAD's state header is the only thing
  preventing an immediate first-request violation. That also
  settles skip-the-probe-and-seed-from-first-GET: not just
  unsanctioned, actually dangerous.

Cross-cutting refinement to the scoping criterion: "scoped in"
means *some test can fail you on it, and the result doc states
which lane the verdict lives in* — mock-judged wire behavior,
core-level property tests, or declared-untested. This taxonomy
feeds agenda item 5's claim-lane structure.

Still open: **bucket quantization** — the brief models one bucket
per policy (pessimistic 60s default, "only learned by getting
429'd") vs. the N12 initial/sustained tiers, the N14 ask-GGG
channel, and the Q4 positional-classification hypothesis. Tom has
further thoughts on server-side bucketing; discussion in progress.

Also open, deliberately deferred: the brief's **headroom**
(effective limit = `max_hits − headroom`). Deferred twice over: it
trades directly against agenda item 2's bounded-over-delay
criterion (decide with the thresholds, not in isolation), and it
shares a margin-against-counter-mismatch role with bucket padding,
so the bucketing discussion may reshape it. Framing settled
2026-08-09 so the eventual decision is small: bucket padding covers
quantization (N11–N13) and pessimistic reconciliation covers
*observed* phantoms, so headroom's unique threat is only the
send-time race with a concurrent same-account tool (model says
14/15, a sibling lands first, yours is the 16th → exogenous 429 —
which P-A's recovery machinery must survive anyway). Recommendation
on the table: default headroom to 0 (three years of production and
the first capture's zero violations price the threat as negligible)
but keep `effective_limit = max_hits − headroom` in the core's
types so the decision reverses with data, not redesign; follow the
spacing-floor precedent on configurability (not an external knob in
the spike). The item-1 phantom-client-at-saturation scenario doubles
as the executable version of this trade-off: with headroom 0 it
judges recovery from the exogenous 429, and its data goes in the
result doc either way.

### 2026-08-09 — Pool killed, remaps split, 4xx budget documented

Three of the brief's four unchecked API-behavior claims, resolved
in one batch (bucket quantization stays open above):

**Shared public-client pool: killed; the design it motivated
survives under a corrected threat model.** N10 (acquisition is a
registered confidential client) plus N23 (under OAuth all API rules
are Account-scoped; no Client rule observed) leaves nothing of the
communal-pool premise standing. What survives, deliberately:

- Pessimistic reconciliation is unchanged — phantom hits are real,
  sourced from same-account tools and cross-session residue
  (N23/N24), not strangers.
- The exogenous-429 branch survives the kill: a same-account
  sibling can consume the last slot between model check and send.
  Do not let the pool correction take that branch down with it.
- Mock consequence: one phantom mechanism ("counter increments the
  client under test didn't cause") — the mechanics are identical
  under either fiction, so only scenario labels and magnitudes
  change. Same-account phantoms are occasional and bursty (user
  launches a second tool), not constant background drizzle;
  pass/fail thresholds must not be tuned against a world that
  doesn't exist. Rule scope varies per rule (the legacy policy has
  an Ip rule, N23) but reconciliation is scope-blind — no per-scope
  phantom machinery.
- Flag beyond spike scope: the brief's OAuth section (public
  client, PKCE-only, communal pool) was written from the same wrong
  premise; do not let it leak unexamined into the later OAuth
  phase.

**Auth-transition remaps: split into reaction and prediction;
reaction kept, prediction dropped.** The brief bundled two
separable mechanisms with different evidentiary standing:

- *Reactive remap handling* — every response carries
  `X-Rate-Limit-Policy` (N5); on mismatch with the endpoint map,
  remap and pessimistically merge history. Needs no precedent
  claim: derives from N5 + N9. Cheap in the actor (the loop sees
  every response). In scope, mock-judged: one scenario renames a
  policy mid-session. Exposure bound: with the serialized gate /
  small in-flight cap, at most in-flight-cap requests are ever
  scheduled under a stale mapping before the first response
  corrects it — residual exposure P-A's recovery covers by design.
- *Proactive provisionality at auth transitions* — the brief's
  login-state remap precedent appears in no N-claim, and the
  machinery buys only the shaving of that one-request exposure
  recovery already covers. Dropped from spike scope; remap
  *triggers* are declared-untested in the result doc while remap
  *handling* is mock-judged. Evidence hunt declined: captures are
  single-auth-mode sessions, and even a confirmed precedent would
  change the scenario list, not the design.
- The brief's "first request after idleness is provisional" is
  subsumed by reactive handling: an idle endpoint's local window
  has aged out, so its one request is safe under the old
  definition, and the response corrects the model before request
  two.

**Invalid-request / 4xx budget: verified — documented, so the
ground-truth doc has a gap (finding), not the brief.** Checked
`https://www.pathofexile.com/developer/docs/index` (retrieved
2026-08-09), verbatim:

> "Applications (and users) that make too many invalid requests in
> a short period of time will be restricted from further access to
> our service."

> "Invalid requests include any response codes in the HTTP 4xx
> range. This includes common codes such as 401 (Unauthorized),
> 403 (Forbidden), and 429 (Too Many Requests)."

> "Reasonable attempts **must** be made in order to avoid passing
> the threshold."

Candidate N-claim (DOC lane, Confirmed; threshold parameters
undocumented — a sibling of Q8) to be transcribed into
`network-ground-truth.md` when the result doc lands on `redesign`
(scope guard: this branch cannot touch that doc). Consequences:

- **429s double-dip**: a violation spends policy budget *and*
  invalid-request budget. Retroactively strengthens the brief's
  never-politely-re-knock ladder, P-A's careful recovery, and the
  stakes of the Q4 danger case.
- **New scoped-in component: a 4xx tripwire.** The spacing floor
  and fuse are request-count defenses; a bug that retry-loops
  politely (respecting the 250 ms floor, under 240 req/min) trips
  neither yet blows the documented threshold. Cheap in the actor:
  a windowed 4xx counter sharing the fuse's halt semantics. Same
  lane split as the fuse: trip logic is a pure function
  (core-level unit/property tests); the wire-level true positive
  is declared untested.
- **Mock models the client's obligation, not the server's
  threshold.** The threshold is opaque with no incident data to
  anchor even an inferred-lane number. Wire scenarios assert the
  documented obligations instead: 401 → zero retries; 429 →
  at-most-one-retry-then-escalate (already scoped). Server-side
  restriction behavior: declared untested.

## Conventions and scope guards

- Greenfield code lives in `spikes/rate-limit-core/` on this branch
  only (hoistable later without surgery). No changes to `src/`, no
  commits to `master` or `redesign` except the eventual result doc
  and register update on `redesign`.
- **No burst or load testing against pathofexile.com. Ever.** Live
  contact only as step 3 above, low-volume, with Tom's explicit
  go-ahead in-session.
- The forum regime stays ungated by design (spec D5); the spike
  covers the header-driven API regime.
- Claim lanes apply to the result doc: measured / estimated /
  inferred (+ external with URL and retrieval date, per the
  credential-custody precedent).
- If the conformance suite works out, note it in the result doc as a
  reusable artifact: the mock + scenarios are the acceptance tests
  any future core's limiter must pass — that outcome outlives the
  spike branch.
