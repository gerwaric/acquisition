# Network Redesign: Typed Facade, Coroutine Pumps, and the Gate

**Status: ACCEPTED (simplified) July 19, 2026.** Drafted and accepted
July 18; reopened the same day by an external review that organized
its findings into four groups. Groups 1 (cancellation and lifetime)
and 2 (degraded HEAD and policy topology) were resolved through three
review rounds. On July 19 Tom directed a simplification pass under the
containment principle (below), which resolved groups 3 (gate scope)
and 4 (complexity) and **deleted the largest machinery the review
rounds had accreted**: the Discovery state, the steady-state
`Protocol` threshold, and rename containment. The full review backlog,
the reversal records, and the revision history are preserved in
Appendix B. No code has been written against this spec.

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
3. **Each policy manager becomes a coroutine "pump"** — one linear loop
   (take, wait, send, maybe retry, deliver) instead of a state machine
   smeared across timer and signal handlers. The 429 retry becomes a
   loop iteration that is impossible to wedge and trivial to test.

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
to the decisions that predated its statement. The reversals — the
degrade-and-proceed HEAD decision and the machinery built on it — are
recorded in the sections they touch and in Appendix B.

## Layer contracts

Five layers, each speaking one language, each testable alone:

| Layer | Class | Speaks | Owns |
|---|---|---|---|
| Orchestration | `ItemsManagerWorker` | tabs, characters, selections | what to fetch, in what order, when an update is done |
| Typed facade | `PoeApiClient` (new) | PoE domain types ↔ HTTP | request building, response parsing, error taxonomy |
| Hub | `RateLimiter` | endpoints, policies, the IP | endpoint→policy topology, HEAD setup, the gate, capture |
| Policy pump | `RateLimitManager` | one policy's pacing | queue, timing, send, 429 retry, history |
| Policy arithmetic | `RateLimitPolicy` | header math | next-safe-send calculation — arithmetic **unchanged** (N25, N26); gains a total validation front-end (D8) |

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
  `Protocol` (2xx during endpoint setup whose rate-limit headers fail
  validation — see D4/D8; steady-state degraded headers are not an
  error, see D8),
  `RateLimited` (429 retries exhausted — see D3),
  `Canceled` (the caller requested stop — see D2). `Canceled` is an
  outcome, not a failure: consumers treat it as "return silently" and
  it never counts toward request-failure accounting. **Futures are
  never canceled in the `QFuture` sense — every future finishes**
  (revised July 18 after external review; the whole cancellation design
  is D2).
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
  silently.
- **Pump checkpoints** (ER4): the token is
  checked at dequeue, after the pacing sleep, after gate acquisition,
  after the reply lands, and after a retry sleep — before every send.
  A stopped entry completes `Canceled` at the first checkpoint that
  sees it. Pacing and retry sleeps are **stop-interruptible** (the
  sleep primitive takes the token and wakes on stop), so an aborted
  update's queue drains promptly instead of at pacing speed. The
  failure mode of a *missed* checkpoint is one wasted — but paced,
  gated, harmless — send, not undefined behavior; that graceful
  degradation is a deliberate property of this design.
- **In-flight requests are never `QNetworkReply::abort()`ed by
  cancellation.** The server has already counted an in-flight request
  (N25: state headers are post-increment, 1:1); aborting it would
  discard the reply's headers and leave the pacing history missing an
  event the server's counters include — exactly the client-model
  divergence P-A warns against. A stopped in-flight request lands,
  is recorded in history and capture, and then completes `Canceled`.
  Hard `abort()` is reserved for application shutdown (see the
  shutdown contract), where history no longer matters.
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

Each `RateLimitManager` runs one long-lived coroutine per policy:

```
loop:
    entry = co_await m_queue.next()            // awaitable queue; closes at shutdown
    if stopped(entry): complete Canceled; continue
    co_await sleepUntil(m_policy->GetNextSafeSend(m_history), entry.token)
    if stopped(entry): complete Canceled; continue
    for attempt in 1..MAX_ATTEMPTS:            // 3 total sends: 1 original + 2 retries
        permit = co_await m_gate.acquire()     // global gate, D5
        if stopped(entry): complete Canceled; break
        reply = co_await permit.send(entry)
        classify per D8; feed capture; record history
        if Full headers and policy name matches:
            m_policy->Update(headers)
        else:
            log loudly; anomaly is captured    // policy unchanged (D8)
        emit signals
        if stopped(entry): complete Canceled; break   // landed & counted; history kept (D2)
        if 429 with valid Retry-After:                // strict-accepted, [1, 900]
            emit Violation
            co_await sleepUntil(max(now + retryAfter + RETRY_BUCKET_PAD + buffer,
                                    m_policy->GetNextSafeSend(m_history)),
                                entry.token)          // N19; reconciled deadline
            continue
        complete with payload or FetchError
        break
    else: complete FetchError{RateLimited}
```

- **One policy-update rule (July 19, replacing round 3's
  check-before-mutation ordering):** only a Full reply (D8) whose
  policy name matches updates the pump's policy. *Every* landed reply
  records its history event — the exchange happened and the server
  counted it (N25), so pacing must not forget it. A mismatched name or
  a less-than-Full header set is logged loudly and captured, its
  payload is still delivered (the data is real), and the pump keeps
  pacing on its last-known Full policy. Rationale and precedent in D8.
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
- Retries are bounded (`MAX_ATTEMPTS = 3`) so a systemically broken
  policy cannot hammer the API — each 429 still increments the violation
  counter and logs (N10: layer-4 goodwill is finite).
- A 429 **without** `Retry-After` is treated as non-retryable (today's
  behavior) and surfaces as `FetchError{Http}` with the violation logged.
- `RateLimitPolicy` and its history-based arithmetic are untouched —
  N25/N26 confirmed the pacing empirically exact. The pump calls the
  same `GetNextSafeSend(m_history)`.
- Existing UI signals (`PolicyUpdated`, `QueueUpdated`, `Paused`,
  `Violation`) are preserved — the status bar and rate-limit dialog keep
  working. The capture instrument keeps its hooks (record every
  exchange including retries, `scheduled` updated on retry).
- Requires two small primitives (new, `src/ratelimit/`): an awaitable
  FIFO (`co_await queue.next()`) and the gate below. Estimated ~70
  lines combined; unit-tested standalone. (Group 4 asked whether the
  long-lived awaitable FIFO is necessary; answer: yes, retained
  deliberately — it is small, standalone-tested, and the pump loop is
  built on it.)

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
  and it is the documented contract). Parked entries forward to the
  pump in submission order.
- **HEAD 429 with Full headers** (genuinely reachable: counters
  persist across restarts, N24, so booting during an active
  restriction can 429 the boot probe; GGG itself floated
  HEAD-after-429 resync, N16): the reply still teaches the topology —
  establish the pump, with its first send scheduled no earlier than
  `now + validated Retry-After + RETRY_BUCKET_PAD` (the D3 formula's
  degenerate case); the HEAD consumes no send attempt.
- **Setup failure — anything else:** a HEAD that fails in transport,
  returns a non-2xx status (including a 429 with less-than-Full
  headers), or returns 2xx/204 with a less-than-Full header set. Every
  parked entry for that endpoint completes with the corresponding
  `FetchError` (`Protocol` for degraded headers, `Network` or `Http`
  otherwise); the failure is logged at error, surfaced as a
  user-visible notification, and captured (probes are captured before
  validation — already implemented); the endpoint resets to `Unknown`
  under the cooldown below. In a worker update the first failure also
  triggers `request_stop()`, washing the update's remaining entries
  through `Canceled`.

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
  failure mode: it must never crash (the D8 validator removes today's
  UB path — a degraded reply crashes the current parser) and it must
  never run unpaced. A synthetic default policy remains rejected
  (inventing pacing numbers GGG never sent). `FatalError` leaves the
  hub entirely.
- **Setup-failure cooldown** (round 2): every failure-driven reset to
  `Unknown` records `earliest_next_probe = now +
  max(SETUP_RETRY_COOLDOWN, validated Retry-After when one was
  present)`; submissions during the cooldown fail fast with the prior
  failure's kind, no probe sent — a caller resubmitting from its
  completion handler cannot loop HEAD probes against N16's narrow
  sanction. `SETUP_RETRY_COOLDOWN = 60 s`, named, provisional.
- **Setup cancellation:** parked entries whose token stops are pruned —
  completed `Canceled` — without affecting the endpoint's state. An
  in-flight HEAD is never aborted (D2's let-it-land rule): its reply
  is processed for topology and capture even if every parked entry has
  been canceled — knowledge the server already charged us for is
  kept. A probe that succeeds with no live entries simply leaves the
  endpoint `Established` and idle; a probe that fails applies the
  normal failure path (the cooldown is failure-driven, not
  entry-driven). Cancellation alone never installs a cooldown.
- **Established is sticky:** an established endpoint keeps its mapping
  through ordinary request failures (a timeout must not discard valid
  topology and burn a HEAD — N16's sanction is one HEAD at boot), and
  with the steady-state threshold and rename machinery deleted (D8),
  **no steady-state event resets topology at all**. Pumps, once
  created, live until shutdown; a pump left idle suspends on its
  queue. The hub's maps cannot go stale: a pump's policy name never
  changes after creation (D8's update rule), unlike today, where
  `Update()` swaps the policy while `GetManager`'s maps keep the old
  key forever.
- **Ordering and identity during setup:** parked entries keep their
  own submission order. Capture records and queue-status signals label
  parked traffic by endpoint until the endpoint is established, then
  by policy name.

### D5. The gate: one object for layer 1

**Scope resolved July 19, 2026 (ER3) — by shrinking the promise, not
engineering the placement.**

A small async gate in the hub through which **every request the hub
dispatches** passes: the five rate-limited endpoints (the four API
endpoints plus the legacy stash index) and the forum-thread traffic
(`Shop` acquires the gate around its GET/POSTs; its scrape-based limit
protocol is unchanged, N22). Three properties, each a named constant:

- **In-flight cap: 2.** P-B requires re-parallelization to state its
  global burst bound explicitly; this is it. The point is to stop
  idling policy lanes (F56), not to maximize concurrency — under N4's
  "seconds per request" reality, lanes are rarely ready simultaneously,
  so 2 already captures nearly all of the win. Tunable; revisit with
  capture data.
- **HEAD exclusive:** a HEAD probe takes the whole gate (N18, F5, N2).
- **Minimum inter-send spacing: 250 ms** across everything the gate
  sees. This is F58's intent — silently dead today — implemented
  deliberately at the right scope: it flattens the ~0.2 s intra-burst
  spikes N26 observed (≈5 req/s) to ≤4 req/s without materially slowing
  small updates (5 tabs: ~1.25 s vs ~1 s). Layer 1 triggered on a burst
  of ~17 req/s sustained (N2); this keeps normal traffic an order of
  magnitude inside that. Tunable; revisit with capture data. F58's
  dead code is deleted.

**Scope rationale:** layer 1 watches the user's IP (N2, N22), so the
gate covers the traffic regimes that share that IP against GGG's API
infrastructure and can plausibly burst — API, legacy website, forum.
**Pre-session traffic stays outside**: the login league list and the
OAuth authorize/token exchanges happen before the hub exists and
amount to a handful of requests per session; layer 1's one known
trigger was over a thousand requests in a minute (N2), three orders of
magnitude away. Documented here as a deliberate exclusion rather than
worked around with pre-hub gate plumbing. Non-GGG hosts (GitHub,
imgur, Sentry) and CDN image traffic (poecdn, webcdn) are likewise
outside — they are not API infrastructure. The gate API is a **permit
acquired around dispatch** (GET/HEAD/POST alike, OAuth-managed
requests included), not a `send()` wrapper.

This resolves the Q10b design question: the redesign *coordinates* the
API, legacy, and forum regimes at layer 1 (their layer-2 handling is
untouched — the forum keeps its scrape-based detection in `Shop`) and
*tolerates* login traffic as-is.

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
- Failure semantics at the update level are unchanged from M1: a
  terminal failure aborts the update (`request_stop()` on the update's
  stop_source, no emit); atomic per-reply replacement already
  guarantees nothing is lost. Per-tab retry/durable progress remain M2
  concerns and are not blocked by anything here.
- The update generation tag is deleted outright, not shrunk (revised
  July 18): each update owns its stop_source, and identity is carried
  by the token. A straggler from a stopped update resolves `Canceled`
  and its coroutine returns silently — there is nothing to compare
  generations against, and awaiting the straggler is safe by D2's
  always-finish guarantee. `DiscardIfStale` and `m_update_generation`
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
- The worker never sees `QNetworkRequest`, `QNetworkReply`, or bytes;
  networking never sees items. The compiler enforces the boundary.
- The facade is intentionally boring: no coroutines inside, no state
  beyond references to the limiter and settings (realm/league move in as
  call parameters, as today).
- `Shop`'s legacy stash-index call moves to the facade
  (`getLegacyStashIndex`); the forum-thread POSTs keep their own path
  (their protocol is scrape-based, N22) but acquire the gate (D5).

### D8. Response classification and header validity

Added July 18, 2026 (group 2 — ER6, ER8; also the source of the N20
ledger correction). **Simplified July 19**: the tiers collapse to Full
vs. not-Full, and the steady-state `Protocol` threshold machinery is
deleted.

**Header validity.** A total validation front-end — new code in
`src/ratelimit/`, leaving `RateLimitPolicy`'s arithmetic untouched —
classifies every reply's rate-limit headers before anything else
touches them:

- **Full**: policy name present; rules list present; every named rule
  has limit and state lists of equal length; every triplet is exactly
  three in-range integers (hits ≥ 0, period > 0, restriction ≥ 0);
  each state period matches its limit period.
- **Not Full**: everything else. (July 19: the former `NameOnly` tier
  and its join-an-existing-pump shortcut are deleted — the observed
  topology is strictly 1:1, N23, so a name-only HEAD almost never has
  a pump to join. The policy name, when present, is logged and
  captured; that is the detection the containment principle asks for.)

**Only Full replies whose policy name matches the pump's update the
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
4. Clean 2xx → header validation (above) → success.

**Steady-state anomalies are logged, not errored (July 19).** At
steady state, a 2xx reply with a less-than-Full header set — or a Full
set bearing a mismatched policy name (never observed; N9's tempering
note records years of stable definitions) — **is not an error**: the
payload is delivered, the history event is recorded, the anomaly is
logged loudly and captured, and the pump keeps pacing on its
last-known Full policy. Pacing on a slightly stale model is exactly
what P-A already prices in; if the model drifts far enough to 429,
D3's retry path handles it. This replaces the former two-consecutive
`Protocol` threshold, its per-endpoint counters, the "consecutive"
bookkeeping rules, and the queue-extraction primitive — machinery
defending against a steady-state header regression that has never been
observed (both real regressions were HEAD-specific, N20). `Protocol`
as an error kind survives only in setup (D4), where strictness is
cheap and correct: there is no last-known policy to fall back on.

**429 under the tiers:** `Retry-After` is the retry authority — a
steady-state 429 whose rate-limit headers are less than Full still
retries per D3 (the pump has a policy for the deadline formula) but
does not update the policy. A 429 with no valid `Retry-After`
(missing, unparseable, or outside D3's [1, 900]) is non-retryable:
`FetchError{Http}`, violation logged. HEAD 429s are handled in D4.

**`FetchError` shape:** kind (`Network` / `Http` / `Parse` /
`Protocol` / `RateLimited` / `Canceled`), endpoint, URL, HTTP status
(when present), Qt error code and string (when present), message.
F50's diagnostics rewording rides along here.

## Shutdown and task ownership

Added July 18, 2026 (ER4): the coroutine design
needs an explicit teardown contract, not an implicit one.

**Ownership chain:** the hub owns the gate and the pumps; each pump
owns its queue and its loop coroutine; the worker owns its update
coroutines; the facade owns nothing stateful. The hub lives for the
application session — teardown below happens only at shutdown.

**Teardown sequence** (hub destruction):

1. The hub stops accepting submissions (submits after this point
   complete immediately with `Canceled`).
2. Queues close: every queued promise completes `Canceled`; each
   pump's `queue.next()` await resumes with a closed signal and the
   loop coroutine runs to completion.
3. In-flight replies are `QNetworkReply::abort()`ed — the one place
   hard abort is permitted (D2): history no longer matters at
   shutdown. The abort resumes the pump's reply await with an error;
   the iteration completes (promise finished) before the pump object
   is destroyed.
4. The gate wakes all waiters with a shutdown outcome and issues no
   further permits.
5. Only then are pumps destroyed.

**Invariant: no coroutine resumes through a destroyed `this`.** Every
suspension point a pump or worker task uses (queue, gate, sleep,
reply, future) must either resume before its owner is destroyed
(steps 2–4 guarantee this for pumps) or be safely destructible while
suspended — which is why QCoro ≥ 0.13 is a hard floor: it fixed the
leak when a task is destroyed while awaiting a `QFuture`.

**Exception policy:** boundary payloads are `expected`, so an
exception crossing a layer boundary is a bug — but coroutine bodies
can still throw (Qt, stdlib). Every top-level task entry point (pump
loops, the worker's update task) wraps its body in a catch-all that
logs and fails the operation safely; no fire-and-forget task is left
unobserved, so no exception can vanish silently (QCoro stores
unhandled task exceptions for a co_await that may never come).

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
replaced by the pump loop.

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
battle-tested code in the app. (Pruned July 19: the pins for the
deleted Discovery, `Protocol`-threshold, and rename machinery are
gone; the group-4 asks — an injected clock and integration coverage —
are folded in.)

1. **Manager harness before any rewrite.** A `RateLimitManager`-level
   test rig with a fake sender serving synthetic `X-Rate-Limit-*`
   headers (the F57 entry's demand — this layer has zero coverage today,
   which is exactly how F57 shipped). Pin current behavior: pacing
   timing against synthetic policies, saturation waits, non-retryable
   error surfacing, and the F57 wedge itself (as a failing-behavior
   pin, then flipped by the pump).
2. **The pump must pass the same harness**, minus the pinned bugs, plus
   new pins: 429 → retry with N19 padding → caller sees exactly one
   final completion; retries exhausted → `FetchError{RateLimited}`;
   stop while queued → completes `Canceled`, never sent; stop during
   the pacing sleep, a gate wait, or a retry sleep → wakes promptly,
   completes `Canceled`, nothing sent; stop while in flight → the
   reply lands, history and capture record it, then `Canceled` (and
   `QNetworkReply::abort()` is never called outside shutdown — N25);
   **the ER1 regression pin**: two awaited fetches, one fails
   and stops the update while the sibling is still suspended — the
   sibling must resume safely into a finished `Canceled` future;
   a steady-state reply with degraded headers or a mismatched policy
   name delivers its payload, records its history event, leaves the
   pump's policy byte-for-byte un-updated, and stops nothing;
   `Retry-After` validation vectors (missing / 0 / negative /
   non-numeric / > 900); shutdown mid-update → queues close, every
   promise finishes, no leaked coroutine frames.
3. **Setup and validation pins**: a Full-vs-not-Full validation suite
   over malformed headers — missing rules list, missing per-rule
   headers, short triplets, non-numeric fields, mismatched periods
   (the current parser's UB inputs become ordinary test vectors); each
   setup-failure flavor (transport error, non-2xx, degraded 2xx
   headers) fails every parked entry with the right `FetchError` kind
   and sets the cooldown; a caller resubmitting from its completion
   handler cannot provoke a probe inside `SETUP_RETRY_COOLDOWN`; a
   Full HEAD 429 establishes the pump with the first send held past
   Retry-After + pad, consuming no attempt; a non-Full HEAD 429 is a
   setup failure whose cooldown honors Retry-After; an established
   endpoint keeps its topology through an ordinary timeout (no HEAD
   re-probe); setup cancellation — canceled parked entries complete
   `Canceled` without disturbing the endpoint state, an in-flight
   probe's reply still establishes the endpoint after all entries
   cancel, and cancellation alone never installs a cooldown;
   concurrent endpoint setups serialize their HEADs at the gate; an
   idle pump persists to shutdown without leaking or waking.
4. **Primitives tested standalone**: awaitable queue (FIFO order,
   cancellation), gate (cap, exclusivity, spacing floor) — pure logic,
   fast tests, with an **injected monotonic clock/scheduler** (group
   4) so timing behavior is deterministic and the tests never sleep.
5. **Worker tests move to a facade fake** ("when asked for stash X,
   return this tab / this error") — simpler than today's byte-crafting
   `FakeRateLimiter`, and it closes the fake-bypasses-the-managers blind
   spot. Existing worker-update behavior pins (`tst_workerupdate.cpp`)
   are ported, not weakened.
6. **Integration coverage** (group 4): cancellation races across
   layers (abort mid-pacing-sleep, mid-gate-wait, mid-flight),
   host/gate classification (what does and does not acquire the gate),
   shutdown mid-update, and completed-future retention (memory does
   not grow with completed fetches across a long update).
7. Every commit compiles and passes `ctest` (working rule #2). The
   capture instrument's tests (`tst_networkcapture.cpp`) must keep
   passing — captures are live research data (Q5, Q9).

## Phasing sketch (an implementation plan derives from this)

1. Manager test harness against current code (no behavior change).
2. QCoro dependency + primitives (queue, gate) with their tests.
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
open items, the external review backlog, and the revision log. The
July 19 simplification pass superseded several earlier resolutions;
where it did, both the original resolution and the reversal are
recorded. Git history holds the full text of superseded designs.

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
   awaitable queue), which have no history and will be read forever.

### External review backlog

The July 18 external review is recorded here so its identifiers and
scope do not depend on a chat transcript. `ER` means external-review
finding and is deliberately distinct from the cleanup register's
permanent `F` numbers.

| ID | Group | Finding | Status |
|---|---|---|---|
| ER1 | Cancellation and lifetime | Canceling a `QFuture` already awaited by a sibling coroutine makes QCoro call `result()` before a post-await liveness guard can run. | Resolved in D1/D2: stop tokens replace QFuture cancellation and every future finishes with a value. |
| ER2 | Degraded HEAD and topology | A provisional pump does not guarantee exactly one unpaced request; multiple provisional/established pumps can overlap, and queue-only merging loses history and ordering. | Resolved in D4: explicit endpoint state machine; no provisional pump exists. (July 19: the Discovery state that resolution introduced was itself deleted — setup now fails cleanly instead of degrading through a discovery GET; the no-two-pumps property stands.) |
| ER3 | Gate ownership and scope | A hub-owned gate cannot cover pre-session login/OAuth traffic; the literal host wildcard also includes legacy CDN image traffic, and the API must accommodate GET/HEAD/POST. | Resolved July 19 in D5 by shrinking the scope: the gate covers hub-dispatched traffic only; pre-session login/OAuth traffic is a documented exclusion (N2 magnitude rationale); CDN hosts are outside; the API is a permit around dispatch. |
| ER4 | Cancellation and lifetime | Cancellation checkpoints, interruptible waits, top-level task ownership, and teardown were unspecified; a stopped request could still send or a coroutine could resume through a destroyed owner. | Resolved in D2/D3 and the shutdown/task-ownership section. |
| ER5 | Cancellation and lifetime | Retaining every facade future for abort can retain completed parsed payloads for a multi-hour update. | Resolved in D2: the stop source is the abort handle and no future registry exists. |
| ER6 | Degraded HEAD and topology | Header validity and policy-name/topology changes are not modeled; the current parser assumes structurally valid triplets and current hub maps can remain keyed by an old policy name. | Resolved in D8 (validation front-end — verification found the missing-header path is UB in today's parser, N20 corrected) and D4/D8 (July 19: rename handling simplified to log-and-capture with the policy un-updated; hub maps cannot go stale because a pump's name never changes). |
| ER7 | Degraded HEAD and topology | Retry timing is underspecified: the applicable bucket depends on Q4, the attempt count is ambiguous, and the retry deadline should be reconciled with policy and gate deadlines. | Resolved in D3: deadline = max(padded Retry-After, GetNextSafeSend) through the gate; unconditional 60 s pad (round 2) fully decouples Q4; Retry-After strict-accepted in [1, 900]; 3 = total sends. |
| ER8 | Degraded HEAD and topology | `FetchError` needs precise HTTP-vs-network precedence and a protocol/header failure that can represent a successful HTTP response with unusable rate-limit headers. | Resolved in D1/D8: status-first precedence, `Protocol` kind, full field inventory. (July 19: `Protocol` is now setup-only; steady-state degraded headers are logged, not errored.) |
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
queue-extraction primitive deleted (stale-policy pacing is P-A's
priced-in residual); rename handling reduced to log-and-capture; the
`NameOnly` tier deleted; the gate's scope shrunk to hub-dispatched
traffic (also resolving ER3); the awaitable FIFO **retained**
deliberately (small, standalone-tested, the pump loop's foundation);
the injected clock/scheduler and integration coverage folded into the
testing plan (items 4 and 6).

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
- **July 19, 2026 (simplification pass — this revision)** — Tom
  reviewed the accumulated design against the containment principle
  and directed a simplification; groups 3 and 4 resolved in the same
  pass. **Deleted:** the Discovery state and all its machinery
  (degraded or failed HEADs now fail the endpoint's requests cleanly
  under the setup cooldown — reversing the first round's
  degrade-and-proceed decision; rationale in D4); the steady-state
  `Protocol` threshold, per-endpoint consecutive counters, and the
  queue-extraction primitive (steady-state header anomalies are
  logged and captured while the pump paces on its last-known policy);
  rename containment (same treatment; a pump's policy name never
  changes after creation); the `NameOnly` tier and join shortcut.
  **Resolved:** ER3 by shrinking the gate to hub-dispatched traffic
  with pre-session login traffic as a documented exclusion; group 4
  by this pass itself, with the injected clock and integration
  coverage folded into the testing plan and the awaitable FIFO
  retained deliberately. **Kept from the review rounds:** everything
  in groups 1's resolution, the retry-timing formula and constants,
  the setup cooldown, the Full-HEAD-429 rule, transport-error
  precedence, and the let-it-land cancellation rules. The spec was
  rewritten clean from the surviving decisions; review history moved
  to this appendix. Status returned to **accepted**.
