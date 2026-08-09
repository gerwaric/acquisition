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
1. Enumerate the scenario list from the N-claims: cold-start burst,
   policy shrink mid-flight, multi-rule policies, 429 recovery,
   `Retry-After` honoring, whatever else the claims imply. Each
   scenario cites its N-numbers.
2. Define pass/fail criteria precisely (zero mock-judged violations
   across all scenarios? bounded over-delay too, so the client isn't
   trivially safe by being absurdly slow?).
3. Sketch the state machine's types and transitions on paper before
   any Rust — what is policy state, what updates it, what queries it.
4. Decide the mock's fidelity budget: which observed behaviors from
   captures must it reproduce; which are out of scope.
5. Define the result doc's skeleton (goes to
   `docs/redesign/topics/`, claim lanes, pass/fail cited to
   N-numbers) so evidence collection is designed in, not bolted on.

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

Still open: the brief's API-behavior claims (bucket quantization,
HEAD-probe exemption, spacing floor, fuse, shared public-client
pool, auth-transition remaps) remain unchecked against the
N-claims — those are the remaining item-0 agenda.

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
