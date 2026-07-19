# Network Redesign: Typed Facade, Coroutine Pumps, and the Gate

**Status: ACCEPTED July 19, 2026 (revision 5).** Drafted and accepted
July 18; reopened the same day by an external review (four finding
groups); groups 1–2 resolved through three review rounds. On July 19
Tom directed a simplification pass under the containment principle
(below), which resolved groups 3–4 and deleted the Discovery state,
the steady-state `Protocol` threshold, and rename containment. Later
the same day Tom's **implementation-readiness review** (line-level,
against the code and Qt/QCoro documentation) found six correctness
gaps in the simplified spec and three further simplifications
(IR1–IR6). A **fourth review round** the same day (R4-1–R4-4 plus
minor items) pinned four implementation ambiguities without reopening
any design decision. A **fifth round** — Tom's implications review of
revision 4 against the code it must live in — found four blockers
(R5-1–R5-4, Appendix B), two of which resolved by deletion: exception
supervision moved into every root coroutine (owned handles supervise
nothing), the asynchronous teardown choreography and the hub shutdown
stop_source were deleted (shutdown is destruction order, which
`Application` already implements), the facade now establishes the
transfer-timeout invariant (false-for-legacy today, F60), and raw
reply ownership is assigned. The full review backlog, the reversal
records, and the revision history are in Appendix B. No code has been
written against this spec.

This document specifies the redesign of acquisition's rate-limited
networking: how the items worker, the rate limiter, and the network
boundary between them are restructured. It is the design output of the
network ground-truth research phase
(`docs/design/network-ground-truth.md`) and the re-derivation of the
paused F56–F59 fix shapes (`docs/cleanup/findings.md`). It also answers
the question `docs/design/items-pipeline.md` deferred to the M2 spec:
**where does scheduling live?** Answer: pacing and retry live in the
rate limiter; selection and ordering live in the worker; flow control
between them is cancellation, not queue management.

Every load-bearing design choice cites the ground-truth claims (N-numbers,
P-A/P-B) or findings (F-numbers) it rests on. If a cited claim falls,
the decision that cites it must be re-examined.

**Staleness preamble** (per the items-pipeline working rules): written
against branch `fix-f57-f59-ratelimit-retry` at `b135ff5` (code
identical to master merge `03b15a3` plus the capture instrument). Code
anchors: `src/ratelimit/ratelimiter.{h,cpp}`,
`src/ratelimit/ratelimitmanager.{h,cpp}`,
`src/ratelimit/ratelimitpolicy.{h,cpp}`, `src/itemsmanagerworker.{h,cpp}`,
`tests/fakenetwork.h`. Re-verify anchors before following this spec if
those files have moved on.

---

## The problem being solved

Today there are two queues and two in-flight trackers stacked on top of
each other: each `RateLimitManager` has a queue and an active-request
slot, and `ItemsManagerWorker` has another queue (`m_queue`) with its own
one-in-flight discipline. The worker layer exists only as a workaround
(`ea9dd95`, v0.17.0) for an abort-desync problem that itself traces to an
unclear ownership contract on `RateLimitedReply` (F59). The visible
damage: cross-policy starvation (F56), a 429 retry that wedges updates
until restart (F57), dead pacing code (F58), and a blocking nested event
loop for HEAD setup (F5).

The redesign dissolves these with three moves rather than four fixes:

1. **Ownership and completion move to Qt-native types** — `QFuture` with
   `std::expected` payloads. The custom `RateLimitedReply` /
   `RateLimitedRequest` classes are deleted; cancellation becomes the
   abort mechanism; F57 and F59 stop being expressible.
2. **Queueing returns to the rate limiter** — the worker batch-submits
   and counts completions; per-policy parallelism returns automatically
   (F56); the worker stops doing flow control at all.
3. **Each policy manager becomes a coroutine "pump"** — one linear
   drain loop (take, wait, send, maybe retry, deliver) instead of a
   state machine smeared across timer and signal handlers. The 429
   retry becomes a loop iteration that is impossible to wedge and
   trivial to test.

## Design principle: contain the unexpected (added July 19, 2026)

Stated by Tom after the round-3 review, as both a design rule and the
convergence criterion for further review rounds:

> Prefer a simple system that **errors when something unexpected
> happens** over a complex one that continues through it.

Operationally: **continuation semantics are designed only for observed
reality** — 429s, network failures, cancellation, the documented
header contract. Events that are unobserved or observed once
(policy renames, multi-endpoint policies, degraded header sets) get
**containment**: detect, log loudly, capture, fail the affected
requests cleanly, and re-establish on a later update. **Build the
detection, not the response**: for a never-observed event, the capture
of its first real occurrence is worth more than any pre-designed
choreography, which would be guessing at the event's shape anyway.
P-A already prices in the residual — a mishandled rare corner costs at
most a mistimed request or one violation, which the design absorbs by
construction; what it must never cost is a wedge, a crash, or silent
continuation on bad state.

The July 19 simplification pass applied this principle retroactively
to the decisions that predated its statement; the
implementation-readiness review then applied it once more to the
simplification's own output (IR1: log-and-continue on degraded
headers turned out to be silent continuation on bad state — it is now
a clean error). The reversals are recorded in the sections they touch
and in Appendix B.

## Layer contracts

Five layers, each speaking one language, each testable alone:

| Layer | Class | Speaks | Owns |
|---|---|---|---|
| Orchestration | `ItemsManagerWorker` | tabs, characters, selections | what to fetch, in what order, when an update is done |
| Typed facade | `PoeApiClient` (new) | PoE domain types ↔ HTTP | request building, response parsing, error taxonomy |
| Hub | `RateLimiter` | endpoints, policies, the IP | endpoint→policy topology, HEAD setup, the gate, capture |
| Policy pump | `RateLimitManager` | one policy's pacing | queue, timing, send, 429 retry, history |
| Policy arithmetic | `RateLimitPolicy` | header math | next-safe-send calculation — arithmetic **unchanged** (N25, N26); parsing becomes a total function (D8) |

### D1. The boundary type: `QFuture<std::expected<T, FetchError>>`

- The limiter's submit returns
  `QFuture<std::expected<QByteArray, FetchError>>` (body bytes on
  success). The facade chains `.then(parse)` onto it and returns
  `QFuture<std::expected<Payload, FetchError>>` with `Payload` a typed
  struct (`poe::StashListWrapper`, `poe::StashWrapper`,
  `poe::CharacterWrapper`, …).
- **Errors are values, never exceptions.** No exception crosses a layer
  boundary. `FetchError` is one struct with a kind enum:
  `Network` (transport failure — including a reply truncated after a
  2xx status arrived; carries the Qt error; precedence is D8's),
  `Http` (non-2xx status; carries the status),
  `Parse` (facade-level JSON failure),
  `Protocol` (2xx whose rate-limit headers fail to parse or carry a
  mismatched policy name — setup or steady state, D4/D8),
  `RateLimited` (429 retries exhausted — see D3),
  `Internal` (an exception escaped inside the pump or facade — a bug,
  contained as a value; see the exception policy),
  `Canceled` (the caller requested stop — see D2). `Canceled` is an
  outcome, not a failure: consumers treat it as "return silently" and
  it never counts toward request-failure accounting. **Futures are
  never canceled in the `QFuture` sense — every future finishes**
  while any consumer is alive to await it (revised July 18 after
  external review; scoped to live sessions by R5-2 — at application
  destruction promises may die unobserved, safely, because every
  awaiter is destroyed first: see the shutdown section; the whole
  cancellation design is D2).
- `RateLimitedReply` and `RateLimitedRequest` are deleted. The pump's
  internal queue entry holds the `QNetworkRequest`, the endpoint label,
  timestamps for capture, and the `QPromise`.
- Resolves F59 by construction (Qt's shared state is the single owner)
  and removes the object F57 destroyed.

### D2. Cancellation: one `std::stop_token` per update; futures always finish

**Rewritten July 18, 2026** after ER1 demonstrated the
original `cancelChain()` design was unsafe: QCoro's `QFuture` awaiter
(verified at v0.13.0) resumes *immediately* when a future is canceled
and calls `result()` unconditionally in `await_resume` — before any
guard in the coroutine body can run. A sibling coroutine suspended on
its own future when another request fails and cancels the batch would
resume straight into `result()` on a canceled, resultless future:
undefined behavior, unpreventable by any check written *after*
`co_await`. The fix is not a safer guard but a design in which the
hazard is unconstructible:

- **`QFuture::cancel()` / `cancelChain()` are never used, anywhere.**
  The only cancellation channel is `std::stop_token`. Every promise
  reaches a final state — payload, `FetchError`, or `Canceled` —
  so awaiting is unconditionally safe, even through QCoro's naive
  awaiter, even for a coroutine suspended at the moment its update is
  aborted (the ER1 regression pin in the test plan).
- The worker holds one `std::stop_source` per update; every facade
  call takes its token, which travels with the queue entry into the
  pump. Abort = `request_stop()` — the worker proceeds with its own
  abort bookkeeping immediately and never waits on stragglers; they
  resolve `Canceled` asynchronously and their coroutines return
  silently. (The token is the update's *identity* only under the
  worker's post-await invariant — D6.)
- **Pump checkpoints** (ER4): the stop state is
  checked at dequeue, after the pacing sleep, after gate acquisition,
  after the reply lands, and after a retry sleep — before every send.
  "Stopped" means the entry's token — the *only* token anywhere
  (R5-2 deleted the hub shutdown token: shutdown is destruction, not
  a signal — see the shutdown section). A stopped entry completes
  `Canceled` at the first checkpoint that sees it. Pacing and retry
  sleeps are **stop-interruptible** (the sleep primitive takes the
  entry token and wakes on stop), so an aborted update's queue drains
  promptly instead of at pacing speed. The failure mode of a *missed* checkpoint is one
  wasted — but paced, gated, harmless — send, not undefined behavior;
  that graceful degradation is a deliberate property of this design.
- **Queued resumption (R4-2):** stop-triggered wakeups are delivered
  through the event loop — `request_stop()` returns before any
  suspended coroutine resumes; no stop callback resumes a coroutine
  synchronously. The worker's first-failure `request_stop()` runs
  inside a promise-completion continuation; synchronous resumption
  there (`std::stop_callback` runs on the requesting thread, inside
  the `request_stop()` call) would cascade every suspended pump drain
  and worker coroutine onto the current stack — exactly the unbounded
  re-entrancy D5's complete-promise-last rule exists to avoid. The
  stop-interruptible sleep and the gate implement wake-on-stop as a
  queued resume; verified by the phase-0 spike.
- **In-flight requests are never `QNetworkReply::abort()`ed by
  cancellation.** The server has already counted an in-flight request
  (N25: state headers are post-increment, 1:1); aborting it would
  discard the reply's headers and leave the pacing history missing an
  event the server's counters include — exactly the client-model
  divergence P-A warns against. A stopped in-flight request lands,
  is recorded in history and capture (a 429 still records its
  violation — D3), and then completes `Canceled`. Hard `abort()` is
  reserved for application shutdown (see the shutdown contract), where
  history no longer matters.
- **No future retention** (resolves ER5): the
  worker does not keep a registry of outstanding futures — the
  stop_source is the abort handle. Each per-fetch coroutine holds its
  future as a local, whose shared state is released when the task
  ends; completed results are not accumulated anywhere a multi-hour
  update could bloat. The facade likewise retains no parent futures
  after chaining.
- This deletes the worker's abort machinery: the five `m_queue = {}`
  sites, the counter-snapping, `DiscardIfStale`, and the generation
  tag (see D6).

### D3. Policy managers become coroutine pumps

**Revised July 19 (IR round):** each `RateLimitManager` owns a plain
`std::deque` of entries and an **on-demand drain coroutine** — the
earlier long-lived awaitable FIFO is deleted (its retention rationale
was circular; Appendix B). Submission pushes an entry and starts the
drain task if none is running; since submission is main-thread-only
(the `Q_ASSERT` stays), "is one running" is a plain bool with no
races. The drain processes entries until the deque is empty, then
returns. There is no permanently suspended task and no queue-close
protocol.

Illustrative shape (the attempt table below is normative):

```
drain():                                   // started by submit when none is running
    while not m_deque.empty():
        entry = m_deque.pop_front()
        if stopped(entry): complete Canceled; continue
        co_await sleepUntil(m_policy->GetNextSafeSend(m_history), entry)  // stop-interruptible
        if stopped(entry): complete Canceled; continue
        for attempt in 1..MAX_ATTEMPTS:            // 3 total sends: 1 original + 2 retries
            permit = co_await m_gate.acquire()     // global gate, D5
            if stopped(entry): complete Canceled; break
            reply = co_await permit.send(entry)    // permit released when the reply finishes (D5)
            // no permit is held past this line — a retry sleep is permit-free (R4-1)
            record history; feed capture
            if 429: record violation               // always — before stop/retry decisions
            if Full headers and policy name matches: m_policy->Update(headers)
            emit signals
            outcome per the attempt table; retryable 429 sleeps and continues,
            everything else breaks with a final outcome
        reach a stable pump state; only then complete the promise
        (the permit was already released at reply-finish — D5)
```

**Per-send bookkeeping (always first):** every landed reply records
its history event and its capture record, and every 429 records a
violation (counter + log) — *before* the stop check and *before* any
retry decision, because the server counted the exchange (N25)
regardless of what the client does next.

**Attempt table (normative):**

| Landed reply (after bookkeeping) | Outcome |
|---|---|
| entry stopped (its token) | complete `Canceled` (the violation of a stopped 429 is already recorded) |
| 429, valid `Retry-After`, attempt < `MAX_ATTEMPTS` | stop-interruptible sleep to `max(now + RetryAfter + RETRY_BUCKET_PAD + buffer, GetNextSafeSend(history))`, then re-send |
| 429, valid `Retry-After`, attempt = `MAX_ATTEMPTS` | complete `FetchError{RateLimited}` **immediately — never sleep on an exhausted attempt** |
| 429, no valid `Retry-After` | complete `FetchError{Http}` (non-retryable — today's behavior; violation already recorded) |
| any other non-2xx status | complete `FetchError{Http}` |
| Qt transport error (incl. alongside a 2xx — D8 precedence) | complete `FetchError{Network}` |
| clean 2xx, Full headers, matching policy name | `Update(headers)`; complete success (payload) |
| clean 2xx, headers fail to parse **or** mismatched policy name | complete `FetchError{Protocol}` (strict — D8/IR1) |

- The retry is invisible to callers: the future completes only on a
  final outcome — success, non-retryable error, or retries exhausted.
  P-A (graceful 429 recovery is a first-class requirement) is satisfied
  structurally: a 429 can no longer wedge anything (F57).
- **Retry timing (ER7, resolved):** the retried send is scheduled at
  `max(now + RetryAfter + maxBucket + buffer, GetNextSafeSend(updated
  history))`, and it passes the gate like every send — so the three
  deadlines (server's, policy's, gate's) reconcile by construction.
  `maxBucket` is an **unconditional named constant,
  `RETRY_BUCKET_PAD_SECS = 60`** (round 2 — "the policy's maximum
  bucket" still required classifying rules, and the legacy policy's
  bucket tiers are unknown, N21b): no policy inspection at all, so the
  retry path is genuinely immune to the initial-vs-sustained question
  (Q4). Recorded assumption: 60 s is the largest bucket tier GGG has
  named for any policy (N12) and is assumed to be a ceiling for the
  legacy policy too — the N14 ask-GGG channel is the remedy if
  evidence ever contradicts. A rare retry over-pays ≤ 55 s for this;
  accepted. (N19's padding remains provisional pending Q9 capture
  data either way.) `Retry-After` is validated by **strict
  acceptance, not clamping** (round 2 wording fix): a value is
  accepted iff it parses as an integer in **[1, 900]** — the longest
  observed restriction is 600 s (N23) plus 50% headroom; anything
  else is treated as absent (non-retryable, clean failure), because
  silently obeying a corrupt server-supplied wait would be an
  invisible stall — a stealth F57.
- **Retry sleeps are permit-free (R4-1):** the permit is released the
  moment the reply finishes (D5's permit span) and each attempt
  re-acquires the gate — a permit is never held across a retry sleep.
  A retry sleep can run to ~961 s (900 + 60 + 1); holding one of the
  gate's two permits through it, with a waiting HEAD blocking the
  rest under writer preference, would stall the entire hub behind one
  429 — a gate-shaped F57.
- **Raw reply ownership (R5-4):** after dispatch the pump is the
  reply's sole owner — `NetworkManager` never enables Qt's
  auto-deletion (default false), and rev 4 assigned no owner at all:
  the contradictory-ownership mistake this redesign exists to kill
  (F59), re-created one level down. The moment the reply await
  returns, the reply is wrapped in RAII with a `deleteLater` deleter;
  body copied and headers consumed before any completion or retry
  decision. Every path — success, error, each retry attempt,
  exception (composes with the completion guard),
  cancellation-after-landing, hub destruction — cleans up through
  that one wrapper; no per-path deletion code. The gate permit
  observes `finished` but never owns or deletes the reply. The setup
  path owns its HEAD reply under the same rule (D4).
- Retries are bounded (`MAX_ATTEMPTS = 3`) so a systemically broken
  policy cannot hammer the API — each 429 still increments the violation
  counter and logs (N10: layer-4 goodwill is finite).
- **Completion ordering (IR6):** the promise is completed only after
  the permit is released and the pump has reached a stable state —
  completion can synchronously run facade parsing and worker
  continuations (D6), which may immediately re-enter the limiter.
- `RateLimitPolicy` and its history-based arithmetic are untouched —
  N25/N26 confirmed the pacing empirically exact. The pump calls the
  same `GetNextSafeSend(m_history)`.
- Existing UI signals (`PolicyUpdated`, `QueueUpdated`, `Paused`,
  `Violation`) are preserved — the status bar and rate-limit dialog keep
  working. The capture instrument keeps its hooks (record every
  exchange including retries, `scheduled` updated on retry).
- Requires two small primitives (new, `src/ratelimit/`), unit-tested
  standalone with an injected clock: a **stop-interruptible sleep**
  (QCoro's sleeps take no stop token, so this is required regardless
  of queue design) and the gate (D5). The deque + drain is ordinary
  code in the pump, not a primitive.

### D4. Endpoint setup: probe once, establish or fail cleanly

**Rewritten July 19, 2026 (simplification pass).** The Discovery state
and everything built on it — the discovery GET, adoption-seeding
rules, the discovery-429 split, the setup-cancellation promotion
protocol — are **deleted** (reversal recorded in Appendix B). Every
endpoint the hub knows is in exactly one state:

```
Unknown → Probing → Established
```

with any setup failure returning to `Unknown` under a cooldown. No
provisional pump object exists in any state; two pumps can never serve
one policy (ER2 resolved structurally).

- **Unknown → Probing:** the first submission for an unknown endpoint
  parks in the hub and fires the HEAD asynchronously (no nested event
  loop — the `QEventLoop` block in `SetupEndpoint` is deleted; the
  main-thread `Q_ASSERT` on submit stays). Submissions arriving
  mid-setup park behind it, keeping submission order.
- **HEAD exclusivity is enforced at the gate (D5):** a HEAD acquires
  the gate exclusively — nothing else in flight while a probe runs.
  This preserves N18/F5 **deliberately** rather than as a side effect
  of accidental serialization, and it holds even with multiple
  endpoints setting up concurrently (each HEAD serializes at the
  gate). The F5 standing constraint is amended, not violated: one HEAD
  at a time, ever, is now a property of the gate.
- **Probing → Established** (the HEAD returns a **Full** header set,
  per D8): create-or-join the pump by policy name (N6: same-named
  policies share counters; the observed topology is strictly 1:1 —
  five endpoints, five policies, N23 — but the join is a map lookup
  and it is the documented contract). A probe that joins an existing
  pump updates that pump's policy under the same rule as any reply
  (Full headers, matching name — D3/D8; R4 minor). Parked entries
  forward to the pump in submission order.
- **HEAD 429 with Full headers and a valid `Retry-After`** (genuinely
  reachable: counters persist across restarts, N24, so booting during
  an active restriction can 429 the boot probe; GGG itself floated
  HEAD-after-429 resync, N16): the reply still teaches the topology —
  establish the pump, with its first send scheduled no earlier than
  `now + RetryAfter + RETRY_BUCKET_PAD + buffer` (the D3 formula's
  degenerate case); the HEAD consumes no send attempt. The 429 is a
  real server-side violation and is recorded — counter, log, capture
  — like any pump-path 429 (R4 minor). A Full HEAD
  429 whose `Retry-After` is missing or invalid (IR5) is a **setup
  failure** under the cooldown below — containment declines to invent
  a hold time.
- **Setup failure — anything else:** a HEAD that fails in transport,
  returns a non-2xx status (including any 429 not covered above), or
  returns 2xx/204 with headers that fail to parse. Every parked entry
  for that endpoint completes with the corresponding `FetchError`
  (`Protocol` for unparseable headers, `Network` or `Http` otherwise);
  the failure is logged at error, surfaced as a user-visible
  notification, and captured (probes are captured before validation —
  already implemented); the endpoint resets to `Unknown` under the
  cooldown below. In a worker update the first failure also triggers
  `request_stop()`, washing the update's remaining entries through
  `Canceled`.

  **This is the July 19 reversal of the first round's
  degrade-and-proceed decision** (Tom's call, under the containment
  principle). The deleted discovery fallback existed for a server-side
  HEAD-regression class last observed in Dec 2023 (N20) — fixed after
  Tom's report, confirmed healthy on all five endpoints in the first
  instrumented capture (N24), and with GGG aware acquisition depends
  on the mechanism (N16, N17). If it ever regresses again: updates
  fail with a clear error, the capture records the regression's actual
  shape from the first affected session, and a fallback can then be
  built against evidence instead of guesses. Two invariants bound the
  failure mode: it must never crash (the D8 total parse removes
  today's UB path — a degraded reply crashes the current parser) and
  it must never run unpaced. A synthetic default policy remains
  rejected (inventing pacing numbers GGG never sent). `FatalError`
  leaves the hub entirely.
- **Setup-failure cooldown** (round 2): every failure-driven reset to
  `Unknown` records `earliest_next_probe = now +
  max(SETUP_RETRY_COOLDOWN, validated Retry-After when one was
  present)`; submissions during the cooldown fail fast with the prior
  failure's kind, no probe sent — a caller resubmitting from its
  completion handler cannot loop HEAD probes against N16's narrow
  sanction. `SETUP_RETRY_COOLDOWN = 60 s`, named, provisional.
- **Setup cancellation:** parked entries whose token stops are pruned —
  completed `Canceled` — without affecting the endpoint's state. An
  in-flight HEAD is never aborted outside shutdown (D2's let-it-land
  rule): its reply is processed for topology and capture even if every
  parked entry has been canceled — knowledge the server already
  charged us for is kept. A probe that succeeds with no live entries
  simply leaves the endpoint `Established` and idle; a probe that
  fails applies the normal failure path (the cooldown is
  failure-driven, not entry-driven). Cancellation alone never installs
  a cooldown.
- **Established is sticky:** an established endpoint keeps its mapping
  through ordinary request failures (a timeout must not discard valid
  topology and burn a HEAD — N16's sanction is one HEAD at boot), and
  **no steady-state event resets topology**: a steady-state `Protocol`
  error (D8) fails the request, not the endpoint, and self-heals — if
  the server recovers, the next update's Full replies update the
  policy normally. Pumps, once created, live until shutdown; a pump
  with an empty deque simply has no drain task running. The hub's maps
  cannot go stale: a pump's policy name never changes after creation
  (D8's update rule), unlike today, where `Update()` swaps the policy
  while `GetManager`'s maps keep the old key forever.
- **Ordering and identity during setup:** parked entries keep their
  own submission order. Capture records and queue-status signals label
  parked traffic by endpoint until the endpoint is established, then
  by policy name.

### D5. The gate: one object for layer 1

**Scope resolved July 19 (ER3) by shrinking the promise; contract
sharpened and forum traffic removed later the same day (IR round).**

The gate is **internal to the hub**: every send the pumps and the
setup path dispatch — the five rate-limited endpoints (the four API
endpoints plus the legacy stash index) — acquires it. It has no
public API. Three properties, each a named constant, plus three
contract rules:

- **In-flight cap: 2.** P-B requires re-parallelization to state its
  global burst bound explicitly; this is it. The point is to stop
  idling policy lanes (F56), not to maximize concurrency — under N4's
  "seconds per request" reality, lanes are rarely ready simultaneously,
  so 2 already captures nearly all of the win. Tunable; revisit with
  capture data.
- **HEAD exclusive, with writer preference (IR6):** a HEAD probe takes
  the whole gate (N18, F5, N2), and once a HEAD is *waiting*, no new
  ordinary permits are issued — without this, a busy established pump
  could starve endpoint setup indefinitely.
- **Minimum inter-send spacing: 250 ms** across everything the gate
  sees. This is F58's intent — silently dead today — implemented
  deliberately at the right scope: it flattens the ~0.2 s intra-burst
  spikes N26 observed (≈5 req/s) to ≤4 req/s without materially slowing
  small updates (5 tabs: ~1.25 s vs ~1 s). Layer 1 triggered on a burst
  of ~17 req/s sustained (N2); this keeps normal traffic an order of
  magnitude inside that. Tunable; revisit with capture data. F58's
  dead code is deleted.
- **Permit span (IR6; sharpened R4-1):** a permit is held from
  acquisition until the reply finishes (`QNetworkReply::finished`) —
  released at dispatch, the in-flight cap would bound nothing — and
  never longer: release happens at reply-finish, so a permit is never
  held across a retry sleep (D3).
- **Liveness rests on the transfer timeout (R4-4; corrected R5-3):**
  the gate's liveness — permit turnover, HEAD exclusivity ever
  becoming acquirable, prompt drain of stopped in-flight entries
  under the never-abort rule (D2) — depends on every reply finishing
  in bounded time. R4 called the 10 s timeout "existing behavior";
  that was **false for the legacy stash-index request**, which today
  has no transfer timeout at all (`Shop::UpdateStashIndex` builds a
  bare request; only the OAuth builders set one — F60). The invariant
  is therefore *established by the facade* (D7), which sets the
  timeout on every request it builds — all five — with a test over
  every builder. Removing or unbounding the timeout requires
  redesigning the gate.
- **Completion ordering (IR6):** the permit is released, and the pump
  reaches a stable state, *before* the entry's promise is completed —
  promise completion may synchronously re-enter the limiter (D3, D6).

**Scope rationale:** layer 1 watches the user's IP (N2, N22), and the
traffic that can plausibly burst is exactly the traffic the hub
paces. **Login and OAuth traffic stays outside**: the login league
list and the OAuth authorize/token exchanges precede the hub, and
mid-session token refreshes add a handful more (R4 wording fix —
refresh is not pre-session, but the magnitude argument is
unchanged); layer 1's one known trigger was over a thousand requests
in a minute (N2), three orders of magnitude away. **Forum traffic is likewise outside (IR round,
reversing this spec's earlier revisions):** `Shop` is strictly
sequential today — one thread at a time with deliberate delays
(`shop.cpp:388`) — user-triggered, and has never brushed layer 1; the
same magnitude argument applies. Gating it bought a public permit
API, external reply-lifetime tracking, and shutdown obligations for
non-hub code, for no demonstrated risk reduction. Non-GGG hosts
(GitHub, imgur, Sentry) and CDN image traffic (poecdn, webcdn) are
likewise outside — they are not API infrastructure. P-B still holds:
the stated global bound is the gate cap over the parallel lanes, and
forum adds at most one slow, serialized request on top.

This resolves the Q10b design question: the redesign *coordinates* the
API and legacy regimes (both flow through the hub) and *tolerates*
forum and login traffic as-is, documented here.

### D6. The worker: batch submit, count completions, no flow control

- On update start the worker submits **all** selected fetches
  immediately through the facade (folder/Map/Unique children are
  appended as parent replies land, same as today). Per-policy FIFOs in
  the pumps preserve submission order — ordering is priority, which
  satisfies "walk requests as the items model iterates them";
  reprioritization stays out of scope (M2+, cheap later via
  cancel+resubmit).
- Worker-side serialization is explicitly rejected: one-at-a-time
  submission *is* F56 — it recreates idle policy lanes by construction.
  Batching is also strictly less state: no `m_queue`, no
  `SubmitNextItemRequest`, no re-entrancy dance.
- The two list requests are submitted concurrently again (they are
  distinct policies — N6, N7; the else-if chaining is deleted).
- Orchestration becomes coroutines (`QCoro::Task<>` methods,
  `co_await` on facade futures): the update sequence reads top-to-bottom
  and the callback pyramid with its flag pairs
  (`m_need_*` / `m_has_*`) collapses into control flow. Completion
  counting (`m_stashes_received` etc.) stays — it drives the status bar.
- **Task topology and handle ownership (R4-3):** the worker owns one
  update task plus one per-fetch `QCoro::Task<>` per submission, all
  held in owned members (a handle container for the per-fetch tasks)
  — no fire-and-forget anywhere. A task handle owns a coroutine
  frame, not a payload, so this does not conflict with D2's
  no-future-retention rule; but 2,000 frames are memory, which is
  exactly what the phase-0 measurement weighs. Ownership is for
  **lifetime control only** — it supervises nothing (R5-1, exception
  policy). Handles are reclaimed by a **deferred sweep that runs
  outside any completing coroutine** (R5-1): finalization triggered
  by the last completion must not clear the container holding the
  frame it is running in, and **abort never destroys live handles**
  — rev 4's released-on-abort rule contradicted D2's
  stragglers-resolve-later contract; stragglers finish `Canceled`
  (bounded by the transfer timeout) and are then swept. No
  incremental per-completion reclamation beyond that sweep until the
  measurement demands it. "Update done" remains completion counting:
  each per-fetch coroutine increments its counter after its await
  resolves (post-await check first), and the completion that
  reconciles the counters triggers finalization, exactly as today's
  handlers do.
- **Batch-submit signal volume (R4 minor):** submitting thousands of
  entries fires as many synchronous `QueueUpdated` emissions in one
  loop. Not a correctness issue; if the status dialog measurably
  stutters, coalesce on the UI side (M2's coalescing work), not in
  the limiter.
- **Post-await identity invariant (IR2):** the update token is the
  update's identity *only because* every worker coroutine checks
  `stop_requested()` immediately after every `co_await`, before
  touching worker state. A future can complete successfully an
  instant before `request_stop()`; its consumer resumes afterward and
  must not mutate state that now belongs to a later update. Each
  per-fetch coroutine has essentially one await site, so the check
  surface is small; pinned with ready-future tests.
- **Initialize-before-launch invariant (IR2):** Qt runs `.then`
  continuations synchronously when attached to an already-finished
  future, and QCoro resumes without suspending on ready futures — so
  a fail-fast submission (setup cooldown, shutdown) can run
  completion logic *during the batch-submit loop*. All per-update
  counters and batch state are initialized before the first
  submission is made. Pinned with ready-success and ready-error
  futures.
- Failure semantics at the update level are unchanged from M1: a
  terminal failure aborts the update (`request_stop()` on the update's
  stop_source, no emit); atomic per-reply replacement already
  guarantees nothing is lost. Per-tab retry/durable progress remain M2
  concerns and are not blocked by anything here.
- The update generation tag is deleted outright, not shrunk (revised
  July 18): each update owns its stop_source, and identity is carried
  by the token under the post-await invariant above. A straggler from
  a stopped update resolves `Canceled` and its coroutine returns
  silently, and a straggler that resolved successfully is discarded by
  the invariant's check. `DiscardIfStale` and `m_update_generation`
  go away.

### D7. The typed facade: `PoeApiClient`

New class (suggested home `src/poe/poeapiclient.{h,cpp}`), owning the
boundary between domain and network:

- Methods (mirroring today's five rate-limited calls): `listStashes`,
  `getStash`, `listCharacters`, `getCharacter`, `getLegacyStashIndex`.
  Each takes the caller's `std::stop_token` (D2; callers with no abort
  story pass a default never-stopped token), builds the request
  (absorbing `poe::MakeStashRequest` et al.),
  submits to the limiter with the token attached, and chains parsing
  (absorbing the `json::read*Wrapper` calls and their error handling)
  into a `.then()` continuation. Returns
  `QFuture<std::expected<Payload, FetchError>>`.
- **The parse continuation is exception-tight (IR4):** the `.then`
  body wraps its work in a catch-all that converts any escaped
  exception into `FetchError{Parse}` (or `Internal` for a non-parsing
  failure) — otherwise a throwing continuation produces an
  exceptional `QFuture`, which crosses the value-only boundary and
  rethrows out of `co_await`.
- The worker never sees `QNetworkRequest`, `QNetworkReply`, or bytes;
  networking never sees items. The compiler enforces the boundary.
- The facade is intentionally boring: no coroutines inside, no state
  beyond references to the limiter and settings (realm/league move in as
  call parameters, as today).
- **The facade owns the transfer-timeout invariant (R5-3):** every
  request it builds carries the 10 s transfer timeout — the gate's
  liveness depends on it (D5). Today only the OAuth builders set it;
  the legacy stash-index request is built bare (F60) — the facade
  closes that gap by construction, and a test covers every builder.
- **Everything runs on the main thread (R4 minor, stated
  explicitly):** submission is main-thread-only, pumps complete
  promises on the main thread, and the parse continuations attach
  with no threading context, so they run there too — where today's
  wrapper parsing already happens. This is an invariant, not an
  accident: giving a continuation a threaded context invalidates the
  single-threaded assumptions in D2–D6.
- `Shop`'s legacy stash-index call moves to the facade
  (`getLegacyStashIndex`); the forum-thread POSTs keep their own path
  entirely — scrape-based protocol and sequential pacing unchanged
  (N22), outside the gate (D5).

### D8. Response classification and header validity

Added July 18, 2026 (group 2 — ER6, ER8). Simplified July 19 (tiers
collapse to Full vs. not-Full; the steady-state threshold machinery
deleted), then **corrected in the IR round: steady-state anomalies
are strict errors, and validation and parsing are one total
function.**

**Total parsing (IR simplification).** Header validation *is* the
parser: a total factory in the shape
`RateLimitPolicy::parse(headers) → std::expected<RateLimitPolicy,
error>` replaces the current throwing/UB constructors. A separate
validator in front of the existing parser would implement the same
grammar twice and let the two drift. The arithmetic methods
(`GetNextSafeSend` and friends) are untouched — they are the
empirically validated part (N25, N26). **Full** means the parse
succeeds, which requires: policy name present; rules list present;
every named rule has limit and state lists of equal length; every
triplet is exactly three in-range integers (hits ≥ 0, period > 0,
restriction ≥ 0); each state period matches its limit period.
(July 19: the former `NameOnly` tier and its join-an-existing-pump
shortcut are deleted — the observed topology is strictly 1:1, N23.
The policy name, when present in a failed parse, is logged and
captured; that is the detection the containment principle asks for.)

**Only a Full reply whose policy name matches the pump's updates the
policy**, and nothing downstream ever indexes an unvalidated triplet.
This deletes a live UB path found while verifying ER6: today a missing
header parses to a one-element `[""]` list (Qt's `split` on an empty
array), passes `RateLimitRule`'s only size check, and `RateLimitData`
reads `parts[1]` out of bounds (`ratelimitpolicy.cpp:52`) — so the
ground-truth ledger's N20 claim that a degraded HEAD "runs unpaced"
was wrong; it crashes. (Ledger corrected in place, July 18, 2026.)

**Classification precedence** (ER8; amended in round 2 — a 2xx status
must not override a transport error, because Qt can report a status
and then fail mid-body: `RemoteHostClosedError` is defined as the
remote closing "before the complete response was received"):

1. Status 429 → the rate-limit path (D3), regardless of any reply
   error Qt also raises for it (`Retry-After` is validated either way).
2. Any other non-2xx status → `FetchError{Http}` carrying the status
   (Qt reports many of these as reply errors too; the status governs).
3. Any remaining Qt error — **including alongside a 2xx status** (the
   truncated-body case), and the no-status-at-all transport failures
   (connection refused, timeout, SSL, …) → `FetchError{Network}`,
   carrying the Qt error code and string.
4. Clean 2xx → total parse (above) → success, or `Protocol`:

**Steady-state anomalies are strict errors (IR1 — reversing the
simplification pass's log-and-continue rule).** A clean 2xx whose
rate-limit headers fail to parse, or parse to a policy name that does
not match the pump's, records its history event and capture record,
logs loudly, and completes `FetchError{Protocol}`. The
log-and-continue rule was unsound: `GetNextSafeSend` is
*state-driven*, not history-driven — it returns "send now" whenever
the stored status is below BORDERLINE (`ratelimitpolicy.cpp:274`) and
consults history only for rules the stored state says are saturated.
With updates frozen at an OK state, the pump would send at gate speed
until the server 429s, retry, succeed with another degraded reply,
and resume unpaced — *repeated* violations (N10), not P-A's priced-in
mistimed request. Strictness is also the only honest boundary:
`expected<Payload, FetchError>` cannot deliver a payload and flag an
anomaly at once. The worker's existing first-failure `request_stop()`
ends the update cleanly; the endpoint stays `Established` and
self-heals (D4). No threshold, no counters, no reset — this is
containment at its simplest: one anomalous reply, one clean error.

**429 under the tiers:** `Retry-After` is the retry authority — a
steady-state 429 whose rate-limit headers fail to parse still retries
per D3 (the pump has a policy for the deadline formula) but does not
update the policy; a successful retry whose reply then fails to parse
completes `Protocol` per the rule above. A 429 with no valid
`Retry-After` (missing, unparseable, or outside D3's [1, 900]) is
non-retryable: `FetchError{Http}`, violation recorded. (Deliberate
taxonomy split, R4 minor: "the server 429ed us" spans two kinds —
`Http` here, `RateLimited` only for exhausted retries. Today's
consumers treat all failures alike; a future consumer that
special-cases rate limiting must match `Http` with status 429 as
well.) HEAD 429s are handled in D4.

**`FetchError` shape:** kind (`Network` / `Http` / `Parse` /
`Protocol` / `RateLimited` / `Internal` / `Canceled`), endpoint, URL,
HTTP status (when present), Qt error code and string (when present),
message. F50's diagnostics rewording rides along here.

## Shutdown and task ownership

Added July 18, 2026 (ER4); rewritten in the IR round (IR3, IR4);
**rewritten again July 19 (R5-2): the asynchronous teardown
choreography is deleted.** Rev 4 required queued resumptions and a
live event loop during shutdown — but `Application` is destroyed
after `a.exec()` returns (`src/main.cpp`), when the loop is already
gone, and `UserSession`'s member order destroys `Shop` and
`ItemsManagerWorker` *before* `RateLimiter` (`src/application.h`), so
a hub-driven drain of worker tasks was unreachable. Rather than build
an asynchronous shutdown subsystem for consumers that are already
destroyed, shutdown is now **destruction order, not choreography** —
which the existing member order already implements.

**Ownership chain:** the hub owns the gate and the pumps; each pump
owns its deque and its drain-task handle (a member — never
fire-and-forget); the worker owns its update-task handle and its
per-fetch handles as members (D6); the facade owns nothing stateful.
The hub lives for the application session. There is **no shutdown
`std::stop_source`** — every wait takes the entry's token only, one
token in every signature (R5-2 simplification; rev 4's second token
existed solely for the deleted choreography).

**Shutdown = destruction, in the existing order:**

1. Consumers die first (`UserSession` member order): `Shop`, then
   `ItemsManagerWorker`. Each destroys its task handles **while
   suspended** — a destroyed coroutine never resumes, so no awaiter
   outlives this step. Within each destructor, task handles are
   destroyed before any member they might observe.
2. The hub dies next: its destructor destroys every drain task
   (while suspended — nothing resumes into a dying pump), then
   `QNetworkReply::abort()`s and releases in-flight replies —
   including a HEAD probe — the one place hard abort is permitted
   (D2; history no longer matters). Aborting *after* the awaiters
   are gone means the `finished` signal resumes nothing. Pumps and
   the gate are destroyed last.
3. Promises die unfinished, **safely by construction**: the
   every-future-finishes guarantee (D1/D2) exists for live awaiters,
   and steps 1–2 guarantee none exists — every consumer coroutine is
   destroyed before the promise it awaited. `.then()` continuations
   on broken promises do not run (Qt cancels the chain).

This works identically whether the event loop is alive or not —
destruction is synchronous and depends on no event delivery. **QCoro
0.13's destroy-while-suspended semantics are load-bearing here, not
a fallback:** the phase-0 spike must verify destruction while
suspended on each awaitable we use (future, reply, timer, gate) and
that a destroyed task's reply-awaiter connection dies with it — a
later `finished` emission resumes nothing.

**Invariant: no coroutine resumes through a destroyed `this`.**
During a live session the invariant holds because owners (hub,
worker) outlive their tasks and no live-session path destroys them;
at shutdown every suspension point (sleep, gate, reply, future) is
destroyed while suspended and never resumed — which is why QCoro ≥
0.13 is a hard floor: it fixed the leak when a task is destroyed
while awaiting a `QFuture`.

**Exception policy (IR4; rewritten R5-1).** Boundary payloads are
`expected`, so an exception crossing a layer boundary is a bug — but
coroutine bodies can still throw (Qt, stdlib). Rev 4's rationale —
"handles are owned members, so no exception can vanish" — was
unsound: QCoro stores an unhandled exception in the task and
rethrows it only to an awaiting consumer or `.then()` continuation;
a retained-but-never-awaited handle observes nothing. A drain that
escaped would never run the code that sets the terminal state, and a
per-fetch task that escaped would never count its completion — a
wedge (F57's ghost), with the exception sitting silently in the
handle. Logging alone is not containment either, and destroying a
`QPromise` unfinished mid-session is the resultless state D2 forbids.
Therefore:

- **No exception ever escapes a root coroutine.** Every root task —
  the pump drain, the worker's update task, each per-fetch task —
  wraps its entire body in a catch-all and finishes
  non-exceptionally. QCoro's stored-exception machinery is
  deliberately never exercised; handle ownership is for lifetime
  only and supervises nothing (D6).
- The drain's catch-all is what *implements* the terminal failed
  state: it completes the active entry `Internal` (the guard below
  already guarantees that), transitions the pump, and fails the
  remaining deque entries fast. A pump in the terminal failed state
  stays there — the event is logged loudly and captured, later
  submissions complete `FetchError{Internal}` fast, and no restart
  is attempted (restart-on-throw risks a tight crash loop;
  terminal-and-loud is the containment answer — the app keeps
  running, the next session starts clean).
- A per-fetch task's catch-all feeds the same first-failure path as
  a `FetchError{Internal}` result: count the completion, then
  `request_stop()` — the update aborts cleanly instead of waiting
  forever on a counter that will never move.
- Every dequeued or parked entry keeps its **scoped completion
  guard**: if the promise has not been completed when the guard
  unwinds — any exceptional path included — it completes
  `FetchError{Internal}`.
- The facade's parse continuations are exception-tight (D7).

## What gets deleted

`RateLimitedReply`, `RateLimitedRequest`, the worker's `m_queue` /
`m_queue_id` / `SubmitNextItemRequest` / `FetchItems`'s queue walk, the
five error-path queue clears with counter-snapping, `DiscardIfStale`
and the generation-tag plumbing outright (D2/D6), the `SetupEndpoint`
nested event loop, the hub's `FatalError` paths (D4: setup failures
fail the affected requests cleanly instead of killing the app), F58's
dead `last_send` block, and `tests/fakenetwork.h`'s `FakeRateLimiter`
(replaced by a facade fake). The pinball machine
(`ActivateRequest`/`SendRequest`/`ReceiveReply` state smear) is
replaced by the drain loop.

## Dependency: QCoro

- QCoro **v0.13.0** (Feb 2026), pinned exactly, fetched via CMake
  FetchContent (whose include-path handling was fixed in this
  release). Requires Qt ≥ 6.8 — satisfied by our 6.11 floor; MIT
  license. Used for: awaiting `QFuture`/`QNetworkReply`/timers,
  `QCoro::Task<>` in the worker and pumps. **0.13 is a hard floor, not
  a preference**: it fixed a memory leak when a task is destroyed
  while awaiting a `QFuture`, which the shutdown contract leans on.
  (The spec's first draft said 0.11.x was current — stale; caught by
  ER9.)
- **FetchContent hygiene (IR round):** QCoro's examples default ON
  and its tests inherit `BUILD_TESTING`, which acquisition enables
  globally — both must be disabled explicitly in the FetchContent
  configuration.
- Qt has **no native** `QFuture` coroutine support as of 6.11; an
  upstreaming effort exists targeting 6.12 at the earliest. If it
  lands, the future-awaiting uses of QCoro can migrate; the
  reply/timer awaitables and `Task<>` keep QCoro relevant regardless.
  Because D2 never cancels futures, even QCoro's naive canceled-future
  handling (`await_resume` calls `result()` unconditionally — verified
  at v0.13.0) is safe by construction; no design decision depends on
  which awaiter implementation is underneath.
- Compiler floor: `CXX_STANDARD 23` in CMake does **not** enforce a
  compiler version (ER9). CMake must check and
  reject explicitly — GCC ≥ 13 (earlier GCC miscompiles coroutines
  with captures in temporary lambdas) plus the Clang/MSVC minimums the
  pinned QCoro release documents — and `BUILD.md` must state the
  floors.

## Testing plan (harness-first, non-negotiable)

Ordering is part of the spec, because the pump rewrite touches the most
battle-tested code in the app. (Revised in the IR round: pins added
for IR1–IR6; primitives are now the gate and the stop-interruptible
sleep.)

1. **Manager harness before any rewrite.** A `RateLimitManager`-level
   test rig with a fake sender serving synthetic `X-Rate-Limit-*`
   headers (the F57 entry's demand — this layer has zero coverage today,
   which is exactly how F57 shipped). Pin current behavior: pacing
   timing against synthetic policies, saturation waits, non-retryable
   error surfacing, and the F57 wedge itself (as a failing-behavior
   pin, then flipped by the pump).
2. **The pump must pass the same harness**, minus the pinned bugs, plus
   new pins: 429 → retry with N19 padding → caller sees exactly one
   final completion; retries exhausted → `FetchError{RateLimited}`
   **completed immediately after the final 429, with no retry sleep**;
   stop while queued → completes `Canceled`, never sent; stop during
   the pacing sleep, a gate wait, or a retry sleep → wakes promptly,
   completes `Canceled`, nothing sent; stop while in flight → the
   reply lands, history and capture record it — **and a stopped
   in-flight 429 still records its violation** — then `Canceled` (and
   `QNetworkReply::abort()` is never called outside shutdown — N25);
   **the ER1 regression pin**: two awaited fetches, one fails
   and stops the update while the sibling is still suspended — the
   sibling must resume safely into a finished `Canceled` future;
   a steady-state reply with unparseable headers or a mismatched
   policy name records history and capture, leaves the policy
   byte-for-byte un-updated, and completes `Protocol` — and the pump
   sends nothing unpaced afterward (IR1); `Retry-After` validation
   vectors (missing / 0 / negative / non-numeric / > 900); **drain
   lifecycle**: submit while a drain is running joins it, submit
   after it finished starts a new one, the drain exits when the deque
   empties; **exception pins (IR4)**: an exception escaping the drain
   completes the active entry `Internal`, puts the pump in its
   terminal failed state, and later submissions fail fast `Internal`;
   a throwing facade parse continuation yields a `Parse`/`Internal`
   *value*, never an exceptional future; **permit-free retry
   (R4-1)**: a retrying entry holds no permit during its retry sleep
   — a sibling pump acquires the gate and sends while that sleep
   runs; **reply ownership (R5-4)**: every reply the fake sender
   hands out is destroyed exactly once on every path — success,
   non-retryable error, each retry attempt, cancel-after-landing,
   drain exception (leak and double-deletion pins).
3. **Setup and validation pins**: a total-parse suite over malformed
   headers — missing rules list, missing per-rule headers, short
   triplets, non-numeric fields, mismatched periods (the current
   parser's UB inputs become ordinary test vectors); each
   setup-failure flavor (transport error, non-2xx, unparseable 2xx
   headers) fails every parked entry with the right `FetchError` kind
   and sets the cooldown; a caller resubmitting from its completion
   handler cannot provoke a probe inside `SETUP_RETRY_COOLDOWN`; a
   Full HEAD 429 with a valid Retry-After establishes the pump with
   the first send held past `Retry-After + pad + buffer`, consuming
   no attempt; a Full HEAD 429 **without** a valid Retry-After, and a
   non-Full HEAD 429, are setup failures whose cooldown honors
   Retry-After when validly present; an established endpoint keeps
   its topology through an ordinary timeout (no HEAD re-probe);
   setup cancellation — canceled parked entries complete `Canceled`
   without disturbing the endpoint state, an in-flight probe's reply
   still establishes the endpoint after all entries cancel, and
   cancellation alone never installs a cooldown; concurrent endpoint
   setups serialize their HEADs at the gate; an idle pump persists to
   shutdown without leaking or waking.
4. **Primitives tested standalone** with an **injected monotonic
   clock/scheduler** (group 4) so timing behavior is deterministic
   and the tests never sleep: the stop-interruptible sleep (one
   token per R5-2; wakes on stop, completes on deadline, and resumes
   via the event loop — `request_stop()` returns before the waiter
   resumes, R4-2)
   and the gate — cap, HEAD exclusivity **with writer preference** (a
   waiting HEAD blocks new ordinary permits; a busy pump cannot
   starve setup), spacing floor, and **permit span** (a permit is
   held until — and released at — reply-finish; the cap genuinely
   bounds in-flight requests, and no permit survives into a retry
   sleep).
5. **Worker tests move to a facade fake** ("when asked for stash X,
   return this tab / this error") — simpler than today's byte-crafting
   `FakeRateLimiter`, and it closes the fake-bypasses-the-managers blind
   spot. Existing worker-update behavior pins (`tst_workerupdate.cpp`)
   are ported, not weakened. **IR2 invariant pins**: an
   already-finished success and an already-finished error future
   (fail-fast submit) run their continuations synchronously during
   the batch-submit loop without corrupting counters
   (initialize-before-launch); a fetch that completed successfully
   just before `request_stop()` resumes its consumer afterward and
   mutates nothing (post-await identity); **R5-1 pin**: a per-fetch
   task whose body throws counts its completion and triggers the
   first-failure stop — the update finishes (aborted), never wedges;
   the handle sweep runs outside any completing coroutine (the last
   completion does not destroy its own frame).
6. **Integration coverage** (group 4 + IR3; shutdown pins rewritten
   R5-2): cancellation races across layers (abort mid-pacing-sleep,
   mid-gate-wait, mid-flight); **destruction reaches every
   suspension** — destroying worker-then-hub in the `UserSession`
   member order while a pump is mid-pacing-sleep, mid-retry-sleep,
   mid-gate-wait, and mid-flight resumes nothing and leaks nothing
   (leak-checked; promises may die unfinished — no awaiter exists to
   observe them); destruction mid-update; completed-future retention
   (memory does not grow with completed fetches across a long
   update).
7. Every commit compiles and passes `ctest` (working rule #2). The
   capture instrument's tests (`tst_networkcapture.cpp`) must keep
   passing — captures are live research data (Q5, Q9).

## Phasing sketch (an implementation plan derives from this)

0. **QCoro integration spike (IR round; gates everything
   QCoro-dependent).** A throwaway target exercising: task-handle
   ownership and destruction while suspended on each awaitable we use
   (future, reply, timer); awaiting already-finished futures;
   `result()` vs `takeResult()` on the single-consumer path (the
   worker should move the parsed payload out, not copy a stash
   wrapper); synchronous promise-completion re-entrancy; stopped
   waits; queued-vs-synchronous resumption on stop and on `abort()`
   (R4-2 requires queued); **destroy-while-suspended as the shutdown
   mechanism (R5-2)** — destruction while suspended on each awaitable,
   with no live event loop, and that a destroyed task's reply-awaiter
   connection dies with it (a later `finished` emission resumes
   nothing); task destruction from inside its own continuation vs a
   deferred sweep (R5-1); the stored-exception behavior of an
   unawaited task (confirming R5-1's premise). Plus the FetchContent hygiene flags (QCoro examples off,
   tests out of our CTest). Findings feed back into D2/D6/shutdown
   before any production code. **Alongside it, one measurement:**
   batch submission at the 2,000-tab scale the items-pipeline doc
   names — peak memory and abort cost for the full
   promise/future/frame/token population. Measure first; bounded
   flow control is speculative machinery until numbers demand it.
1. Manager test harness against current code (no behavior change).
2. QCoro dependency + primitives (stop-interruptible sleep, gate)
   with their tests.
3. Pump rewrite inside `RateLimitManager`; hub gains the gate and async
   HEAD setup. Boundary still the old `Submit` shape via a thin adapter
   so the worker compiles unchanged. Resolves F57, F58, F5-modernization.
4. `QFuture` boundary + `PoeApiClient`; `Shop` and worker call sites
   move over; `RateLimitedReply`/`RateLimitedRequest` deleted. Resolves
   F59.
5. Worker rewrite: batch submission, coroutine orchestration, abort via
   cancellation; worker queue and generation machinery deleted. Resolves
   F56.

Each phase is independently shippable; 3, 4, 5 each land with their
tests in the same PR.

## Interaction with the items-pipeline plan

- This spec formally amends the items-pipeline **"Rate limiter
  redesign" non-goal** (anticipated by F56's pause note): the redesign
  is now in scope, specified here. The F5 property is preserved (D4/D5).
- For the M2 spec: scheduling lives in the limiter; the worker owns
  selection and ordering. M2's per-tab delta signal, durable progress,
  and UI coalescing are orthogonal to this spec and compose with it —
  per-tab completion is exactly a facade future resolving.
- N26's even-pacing idea stays a research experiment, not a design
  input: padded honestly for buckets, even pacing is
  throughput-identical to burst-then-stall
  ((300+60)/30 = 12.0 s/request vs 12.03 observed), and burst wins the
  everyday small-update case outright. The doc's ~20% claim compared
  padded burst against unpadded pacing; unpadded pacing is exactly the
  N15 violation trap. (Recorded here so the ground-truth note doesn't
  get re-litigated.)
- Q4 (initial-vs-sustained classification, the stash-list under-padding
  suspect) lives entirely inside `RateLimitPolicy` and is deliberately
  **not** part of this redesign — it composes as a small independent
  change whenever Tom confirms the positional hypothesis with GGG.

---

## Appendix B — Review history

This appendix preserves the decision history: the first review round's
open items, the external review backlog, the July 19 simplification
pass, and the implementation-readiness review. Where a later round
superseded an earlier resolution, both are recorded. Git history holds
the full text of superseded designs.

### Open items from the first review round — resolved July 18, 2026

1. Gate constants: **blessed as spec'd** — in-flight cap 2, spacing
   floor 250 ms, retry cap 3. Named constants, provisional until
   capture data says otherwise; the revisit trigger for all three is
   capture evidence (lane idling at the gate, spike-correlated
   anomalies, retry exhaustion in the wild).
2. Degraded-HEAD behavior: **degrade and proceed** — decided; folded
   into D4. *(Rescinded July 19, 2026: the decision predated the
   containment principle; re-examined under it, Tom chose clean
   failure — see D4. The Discovery mechanism built for it is
   deleted.)*
3. `cancelChain()` semantics: verified against Qt docs (available
   since Qt 6.10, backwards propagation, nested-future limitation) —
   then **superseded the same day** by the external-review revision:
   the design no longer uses QFuture cancellation at all. See D2
   (stop_token; futures always finish) and the revision log.
4. Naming: **deferred to phase 3 by design.** `RateLimiter` keeps its
   name (app-facing surface; continuity with this research corpus).
   `RateLimitManager`'s name is decided when phase 3 rewrites the file
   anyway — rename is free at that moment and never again; proposals
   welcome then. Naming care goes to the new primitives (the gate, the
   stop-interruptible sleep), which have no history and will be read
   forever.

### External review backlog

The July 18 external review is recorded here so its identifiers and
scope do not depend on a chat transcript. `ER` means external-review
finding and is deliberately distinct from the cleanup register's
permanent `F` numbers.

| ID | Group | Finding | Status |
|---|---|---|---|
| ER1 | Cancellation and lifetime | Canceling a `QFuture` already awaited by a sibling coroutine makes QCoro call `result()` before a post-await liveness guard can run. | Resolved in D1/D2: stop tokens replace QFuture cancellation and every future finishes with a value. |
| ER2 | Degraded HEAD and topology | A provisional pump does not guarantee exactly one unpaced request; multiple provisional/established pumps can overlap, and queue-only merging loses history and ordering. | Resolved in D4: explicit endpoint state machine; no provisional pump exists. (July 19: the Discovery state that resolution introduced was itself deleted — setup now fails cleanly instead of degrading through a discovery GET; the no-two-pumps property stands.) |
| ER3 | Gate ownership and scope | A hub-owned gate cannot cover pre-session login/OAuth traffic; the literal host wildcard also includes legacy CDN image traffic, and the API must accommodate GET/HEAD/POST. | Resolved July 19 in D5 by shrinking the scope: pre-session login/OAuth traffic is a documented exclusion (N2 magnitude rationale); CDN hosts are outside. (IR round, same day: forum traffic also excluded — the gate is fully internal to the hub, no public permit API.) |
| ER4 | Cancellation and lifetime | Cancellation checkpoints, interruptible waits, top-level task ownership, and teardown were unspecified; a stopped request could still send or a coroutine could resume through a destroyed owner. | Resolved in D2/D3 and the shutdown/task-ownership section (IR3 later added the hub shutdown stop_source the teardown sequence was missing). |
| ER5 | Cancellation and lifetime | Retaining every facade future for abort can retain completed parsed payloads for a multi-hour update. | Resolved in D2: the stop source is the abort handle and no future registry exists. |
| ER6 | Degraded HEAD and topology | Header validity and policy-name/topology changes are not modeled; the current parser assumes structurally valid triplets and current hub maps can remain keyed by an old policy name. | Resolved in D8 (verification found the missing-header path is UB in today's parser, N20 corrected) and D4/D8 (July 19: rename handling simplified; hub maps cannot go stale because a pump's name never changes; IR1 made steady-state anomalies strict `Protocol` errors). |
| ER7 | Degraded HEAD and topology | Retry timing is underspecified: the applicable bucket depends on Q4, the attempt count is ambiguous, and the retry deadline should be reconciled with policy and gate deadlines. | Resolved in D3: deadline = max(padded Retry-After, GetNextSafeSend) through the gate; unconditional 60 s pad (round 2) fully decouples Q4; Retry-After strict-accepted in [1, 900]; 3 = total sends. (IR5 later fixed the exhausted-attempt sleep and violation-ordering holes and made the attempt table normative.) |
| ER8 | Degraded HEAD and topology | `FetchError` needs precise HTTP-vs-network precedence and a protocol/header failure that can represent a successful HTTP response with unusable rate-limit headers. | Resolved in D1/D8: status-first precedence, `Protocol` kind, full field inventory. (IR round added `Internal`.) |
| ER9 | Cancellation and lifetime | QCoro 0.11 was stale; 0.13 contains directly relevant QFuture-destruction and FetchContent fixes, and C++23 alone does not enforce the claimed compiler floor. | Resolved in the dependency section. |

**Round 2 (July 18, group 2 re-review):** nine corrections to the
group-2 resolutions, all verified and adopted at the time — evidence
is endpoint-scoped, setup traffic is gate-exclusive, the
discovery-429 split, transport-error-over-2xx precedence, the
unconditional 60 s retry pad, the setup-failure cooldown, reset-scope
limits, and two documentation fixes. Of these, the retry pad, the
cooldown, the precedence rule, and established-keeps-topology survive
in the spec; the discovery and per-endpoint-counter machinery was
deleted July 19.

**Round 3 (July 19, group 2 re-re-review):** five blockers and four
smaller gaps, all resolved — the substantive outcome was the
**containment principle** (see the design-principle section). Of the
round-3 resolutions, the HEAD-429 definition and the
cancellation-is-not-failure cooldown rule survive; the rename
containment path, the name-check-before-mutation ordering rule, and
the discovery-seeding rules were deleted in the July 19 simplification
(subsumed by the single policy-update rule in D3/D8 and the removal of
Discovery).

**Group 4 — complexity and testing: resolved July 19, 2026.** The
simplification pass *was* the complexity review, conducted with Tom
against the containment principle. Outcomes: the Discovery state
deleted (degraded HEAD = clean setup failure — Tom's explicit call:
"I'd rather have an error … if that allows us to simplify"); the
steady-state `Protocol` threshold, per-endpoint counters, and
queue-extraction primitive deleted; rename handling reduced to
detection; the `NameOnly` tier deleted; the gate's scope shrunk (also
resolving ER3); the injected clock/scheduler and integration coverage
folded into the testing plan. *(The pass initially retained the
awaitable FIFO; the implementation-readiness review below reversed
that hours later — deque + on-demand drain — and corrected the pass's
log-and-continue rule for steady-state anomalies, IR1.)*

### Implementation-readiness review — July 19, 2026 (Tom)

After the simplification pass, Tom reviewed the spec line-by-line
against the code and the Qt/QCoro documentation: six findings, three
further simplifications, and four pre-implementation investigations —
all verified and adopted in revision 3.

| ID | Finding | Resolution |
|---|---|---|
| IR1 | "Pace on the last-known policy" was unsound: `GetNextSafeSend` is state-driven (`ratelimitpolicy.cpp:274` returns *now* whenever the stored status is OK), so frozen-OK state means unpaced sends until 429 — repeatedly, since the retry then succeeds degraded. | D8: a steady-state parse failure or name mismatch records history + capture and completes `Protocol` immediately; the worker's first-failure stop ends the update; the endpoint stays Established and self-heals. Simpler than the rule it replaces. |
| IR2 | A stop token is not an update-generation barrier by itself: a future can succeed just before `request_stop()` and its consumer resumes after; Qt runs `.then` continuations synchronously on already-finished futures. | D6: post-await identity invariant and initialize-before-launch invariant, both pinned (ready-success/ready-error futures; late-resuming successful straggler). Generation tag stays deleted. |
| IR3 | The teardown sequence could not reach a pump suspended in a pacing/retry sleep or gate wait (queue closure only wakes a dequeue), and an aborted reply's entry could still retry during shutdown; worker task ownership was vague. | Shutdown rewritten: hub-owned shutdown stop_source observed at every wait and checkpoint; explicit completion inventory (active entries, deque remainders, parked setup entries, HEAD probes, gate waiters); task handles are owned members with destruction order verified by the spike. |
| IR4 | Always-finish was incomplete under exceptions: a dead pump loop wedges queued promises behind a dead consumer, and a destroyed `QPromise` notifies its future without a result — the exact resultless state D2 forbids. | D1 `Internal` kind; scoped completion guard on every entry; terminal pump-failed state that fail-fasts later submissions; facade parse continuations exception-tight (D7). |
| IR5 | Retry-loop holes: an exhausted attempt slept up to ~961 s before reporting; violation recording was inconsistent (non-retryable and stopped 429s uncounted); Full-HEAD-429-without-valid-Retry-After was undefined; D3/D4 disagreed on the buffer term. | D3: normative attempt table; bookkeeping (history, capture, violation) always precedes stop/retry decisions; exhaustion completes immediately. D4: Full HEAD 429 without valid Retry-After is a setup failure. Buffer term unified. |
| IR6 | Gate contract underspecified: a permit released at dispatch makes the cap meaningless; without writer preference a busy pump starves HEAD setup; completing a promise before releasing the permit invites synchronous re-entrancy. | D5: permit span = acquisition through reply-finished; HEAD writer preference; release-permit-and-stabilize-before-completing rule (restated in D3). |

**Simplifications adopted:** the awaitable FIFO deleted — deque +
on-demand drain coroutine (no permanently suspended task, no
queue-close semantics; the earlier "the pump is built on it"
retention rationale was circular, and the genuinely required
primitive was the stop-interruptible sleep all along, since QCoro's
sleeps take no stop token); forum traffic removed from the gate
(`Shop` is strictly sequential with deliberate delays, `shop.cpp:388`
— the login-traffic magnitude argument applies identically, and the
gate becomes fully internal with no public API); validation and
parsing unified into a single total `parse(headers) → expected`
(the same grammar implemented twice would drift; arithmetic
untouched).

**Pre-implementation investigations:** the phase-0 QCoro spike (task
destruction, ready futures, `result()` vs `takeResult()`, synchronous
completion re-entrancy, stopped waits); the 2,000-tab batch-scale
measurement (memory and abort cost — measure before adding flow
control); QCoro FetchContent example/test flags. All folded into the
phasing sketch.

**Standing boundary (restated from the review):** no further design
effort on policy-rename recovery, degraded-HEAD discovery, endpoint
migration, reprioritization, or gate-constant tuning. Detecting those
conditions and failing cleanly is the right level until real evidence
arrives.

### Review round 4 — July 19, 2026

A post-acceptance review of revision 3 (design bugs, inconsistencies,
gaps, complexity; conducted by Claude, adopted by Tom). No design
decision was reopened; four ambiguities were pinned and six minor
items fixed in place. The complexity pass came up empty: everything
remaining (writer preference, the setup cooldown, the terminal
pump-failed state, the completion guard) carries observed weight, and
nothing in the spec designs choreography for an unobserved event.

| ID | Finding | Resolution |
|---|---|---|
| R4-1 | D3's pseudocode read as holding the gate permit across a retry sleep (up to ~961 s) — one retrying entry plus a waiting HEAD under writer preference would stall the whole hub: a gate-shaped F57. | D3/D5: permits release at reply-finish and are never held across a retry sleep; each attempt re-acquires the gate. Pseudocode corrected; pump and gate pins added. |
| R4-2 | Resumption discipline was unspecified: `std::stop_callback` runs synchronously inside `request_stop()`, which the worker calls from a completion continuation — an unbounded synchronous cascade; and the teardown sequence presumed a wait-for-drains mechanism it never named. | D2: stop wakeups resume via the event loop; `request_stop()` returns before any coroutine resumes. Shutdown: teardown needs a live event loop; the concrete await mechanism is a phase-0 spike deliverable. |
| R4-3 | Worker task topology was unspecified: where per-fetch task handles live, who owns them, how "update done" is detected — with apparent tension against D2's no-retention rule at the 2,000-tab scale. | D6: handles are owned members released at update end; a handle owns a frame, not a payload (no conflict with D2); update completion is counter reconciliation, as today. |
| R4-4 | Gate liveness silently depends on the 10 s transfer timeout (permit turnover, HEAD acquirability, stopped-entry drain under never-abort) — stated only in the ground-truth appendix. | D5: the timeout is an explicit invariant of the gate design. |

Minor items, fixed in place: HEAD 429s record their violation (D4); a
probe joining an existing pump updates its policy under the normal
rule (D4); the `Http`/`RateLimited` split for 429s is documented as
deliberate (D8); the D5 exclusion rationale no longer claims all
OAuth traffic is pre-session (token refresh is mid-session; the
magnitude argument is unchanged); the main-thread invariant is stated
explicitly (D7); batch-submit `QueueUpdated` volume is noted as a
UI-side concern (D6).

### Review round 5 — July 19, 2026 (Tom)

Tom's implications review of revision 4: not new edge cases, but
whether the accumulated design fits the code it must live in. Four
blockers, each verified against the code; two resolutions are net
deletions. One framing correction recorded: R4-2/R4-3 patched the
shutdown story instead of asking whether an asynchronous shutdown
should exist at all — R5-2 is the containment answer that question
deserved.

| ID | Finding | Resolution |
|---|---|---|
| R5-1 | Owned task handles do not supervise exceptions: QCoro rethrows a stored exception only to an awaiting consumer, so an escaped drain never runs the terminal-state transition and an escaped per-fetch task never counts its completion — a wedge with the exception sitting silently in the handle. Also: rev 4's release-on-abort rule contradicted D2's stragglers-resolve-later contract, and finalization triggered by the last completion could destroy the frame it runs in. | Exception policy rewritten: no exception ever escapes a root coroutine — every root task catch-alls and finishes non-exceptionally; the drain's catch implements the terminal state; a per-fetch catch feeds the first-failure path. Handles are lifetime-only; deferred sweep outside any completing coroutine; abort never destroys live handles. Spike + pump/worker pins. |
| R5-2 | The rev-4 shutdown required queued resumptions and a live event loop — but `Application` is destroyed after `a.exec()` returns (`src/main.cpp`) and `UserSession` destroys `Shop`/`ItemsManagerWorker` before `RateLimiter` (`src/application.h`): the hub-driven teardown was unreachable at the only moment it would run. | Teardown choreography and the hub shutdown stop_source deleted. Shutdown is destruction order — which `Application` already implements: consumers first, task handles destroyed while suspended, hub aborts replies after all awaiters are gone. Every-future-finishes is scoped to live sessions (safe: no awaiter survives to observe a broken promise). Every wait takes one token. Destroy-while-suspended becomes a load-bearing spike deliverable. |
| R5-3 | D5's timeout-liveness premise was false for the legacy endpoint: `Shop::UpdateStashIndex` builds its request with no transfer timeout (only the OAuth builders set one) — a stalled legacy request could hold a permit indefinitely; with a HEAD waiting under writer preference, the whole hub stops. | D5/D7: the facade establishes the timeout invariant on every request it builds (all five), test over every builder. The live pre-existing gap is recorded as F60 in the findings register. |
| R5-4 | Raw `QNetworkReply` ownership was never assigned — `NetworkManager` does not enable auto-deletion (default false), and today's callers double-`deleteLater`: the contradictory-ownership mistake that motivated the redesign (F59), re-created one level down. | D3: after dispatch the pump (or setup path, for HEADs) solely owns the reply via RAII with a `deleteLater` deleter; body and headers consumed before any completion or retry decision; the permit observes `finished` but never owns. Leak and double-deletion pins. |

### Revision log

- **July 18, 2026** — drafted; open items reviewed with Tom and
  resolved; accepted.
- **July 18, 2026 (later)** — external review reopened the spec:
  correctness gaps in cancellation/lifetime, degraded-HEAD handling,
  and gate ownership; findings organized into four groups. **Group 1
  (cancellation and lifetime — ER1/ER4/ER5/ER9) resolved:**
  `std::stop_token` replaces QFuture cancellation wholesale (D2
  rewritten — ER1's UB is now unconstructible; futures always
  finish), pump token checkpoints and stop-interruptible sleeps
  specified (D3), never-abort-in-flight rule stated with its N25
  rationale, shutdown and task-ownership contract added, the
  future-retention registry deleted, QCoro pinned to 0.13.0 as a hard
  floor, explicit compiler floors required in CMake.
- **July 18, 2026 (group 2)** — degraded HEAD and policy topology
  (ER2/ER6/ER7/ER8) resolved: D4 rewritten as an explicit endpoint
  state machine with a Discovery fallback; D8 added (validity tiers,
  status-first classification, the `Protocol` error kind); D3 retry
  timing resolved. Verification found the degraded-header path is
  **UB in today's parser** (`ratelimitpolicy.cpp:52`): ground-truth
  N20 corrected in place, Q3 residual closed.
- **July 18, 2026 (group 2, round 2)** — nine re-review corrections
  verified and adopted (endpoint-scoped evidence, gate-exclusive
  discovery, unconditional 60 s retry pad, 60 s setup-failure
  cooldown, per-endpoint `Protocol` counter, transport-error
  precedence, discovery-429 split, reset-scope limits, two
  documentation fixes).
- **July 19, 2026 (group 2, round 3 + design principle)** — round-3
  findings resolved under the newly stated **containment principle**:
  rename handling rewritten from live migration to containment; name
  check ordered before mutation; discovery seeding restricted to new
  pumps; HEAD 429 defined; setup cancellation specified; pumps live
  to shutdown; round-3 pins added.
- **July 19, 2026 (simplification pass)** — Tom reviewed the
  accumulated design against the containment principle and directed a
  simplification; groups 3 and 4 resolved in the same pass.
  **Deleted:** the Discovery state and all its machinery (degraded or
  failed HEADs now fail the endpoint's requests cleanly under the
  setup cooldown — reversing the first round's degrade-and-proceed
  decision); the steady-state `Protocol` threshold, per-endpoint
  consecutive counters, and the queue-extraction primitive; rename
  containment; the `NameOnly` tier and join shortcut. **Resolved:**
  ER3 by shrinking the gate with pre-session login traffic as a
  documented exclusion; group 4 by the pass itself. The spec was
  rewritten clean from the surviving decisions; review history moved
  to this appendix. Status returned to **accepted**.
- **July 19, 2026 (implementation-readiness review — rev. 3)** —
  Tom's line-level review against the code and Qt/QCoro
  docs; six findings (IR1–IR6), three simplifications, four
  investigations, all adopted. **Substantive reversals:** steady-state
  degraded or mismatched headers are a strict `Protocol` error (IR1 —
  the simplification pass's log-and-continue rule was silent
  continuation on bad state: the arithmetic is state-driven and a
  frozen-OK policy paces nothing); the awaitable FIFO is deleted
  (deque + on-demand drain); forum traffic leaves the gate (fully
  internal now). **Added — invariants and error paths, not
  mechanism:** hub shutdown stop_source observed at every wait;
  `Internal` error kind with scoped completion guards and a terminal
  pump-failed state; worker post-await and initialize-before-launch
  invariants; gate permit span, HEAD writer preference, and
  complete-promise-last ordering; the normative attempt table
  (exhaustion never sleeps; violations recorded before stop/retry
  decisions; Full HEAD 429 without valid Retry-After = setup
  failure); phase-0 QCoro spike and 2,000-tab measurement;
  exception-tight facade continuations; QCoro FetchContent hygiene.
- **July 19, 2026 (review round 4 — rev. 4)** — a post-acceptance
  review pinned four implementation ambiguities (R4-1–R4-4): permits
  are never held across retry sleeps (the pseudocode read otherwise —
  a ~16-minute stall hazard); stop wakeups resume via the event loop,
  never synchronously inside `request_stop()`, and teardown's
  drain-await mechanism became a phase-0 spike deliverable; worker
  task-handle ownership and update completion are specified; the 10 s
  transfer timeout is named a gate liveness invariant. Six minor
  items fixed in place (Appendix B). No design decision reopened; the
  complexity pass found nothing to delete.
- **July 19, 2026 (review round 5 — this revision, rev. 5)** — Tom's
  implications review of rev 4 against the code it must live in: four
  blockers (R5-1–R5-4), two resolved by deletion. **Exception
  supervision moved into the coroutines** — no exception ever escapes
  a root task; owned handles supervise nothing (QCoro rethrows only
  to an awaiter); the drain's catch-all implements the terminal pump
  state; deferred handle sweep; abort never destroys live handles.
  **The asynchronous teardown choreography and the hub shutdown
  stop_source are deleted** — rev 4's sequence needed a live event
  loop after `a.exec()` had returned and a worker the destruction
  order had already destroyed; shutdown is now destruction order
  (consumers first, handles destroyed while suspended, replies
  aborted after awaiters are gone), every wait takes one token, and
  every-future-finishes is scoped to live sessions. **The facade
  establishes the transfer-timeout invariant** — rev 4 called it
  existing behavior, false for the legacy stash-index request (F60).
  **Raw reply ownership assigned** — the pump/setup path solely owns
  replies via RAII (F59's lesson, applied one level down). Net
  complexity: negative.
