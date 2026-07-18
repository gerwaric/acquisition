# Network Redesign: Typed Facade, Coroutine Pumps, and the Gate

**Status: spec accepted July 18, 2026** — drafted and reviewed with Tom
the same day; all open items resolved (see the final section). Next
step: derive the implementation plan from the phasing sketch. No code
has been written against this spec yet.

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

## Layer contracts

Five layers, each speaking one language, each testable alone:

| Layer | Class | Speaks | Owns |
|---|---|---|---|
| Orchestration | `ItemsManagerWorker` | tabs, characters, selections | what to fetch, in what order, when an update is done |
| Typed facade | `PoeApiClient` (new) | PoE domain types ↔ HTTP | request building, response parsing, error taxonomy |
| Hub | `RateLimiter` | endpoints, policies, the IP | endpoint→policy topology, HEAD setup, the gate, capture |
| Policy pump | `RateLimitManager` | one policy's pacing | queue, timing, send, 429 retry, history |
| Policy arithmetic | `RateLimitPolicy` | header math | next-safe-send calculation — **unchanged** (N25, N26) |

### D1. The boundary type: `QFuture<std::expected<T, FetchError>>`

- The limiter's submit returns
  `QFuture<std::expected<QByteArray, FetchError>>` (body bytes on
  success). The facade chains `.then(parse)` onto it and returns
  `QFuture<std::expected<Payload, FetchError>>` with `Payload` a typed
  struct (`poe::StashListWrapper`, `poe::StashWrapper`,
  `poe::CharacterWrapper`, …).
- **Errors are values, never exceptions.** No exception crosses a layer
  boundary. `FetchError` is one struct with a kind enum:
  `Network` (QNetworkReply error, no HTTP response),
  `Http` (non-2xx status; carries the status),
  `Parse` (facade-level JSON failure),
  `RateLimited` (429 retries exhausted — see D3).
  Cancellation is not an error kind: a canceled request's future is
  canceled, full stop.
- `RateLimitedReply` and `RateLimitedRequest` are deleted. The pump's
  internal queue entry holds the `QNetworkRequest`, the endpoint label,
  timestamps for capture, and the `QPromise`.
- Resolves F59 by construction (Qt's shared state is the single owner)
  and removes the object F57 destroyed.

### D2. Cancellation is the abort mechanism

- The worker retains the futures of the current update. Abort =
  `cancelChain()` on each (Qt ≥ 6.9; propagates backwards through the
  `.then()` chain to the limiter's promise).
- The pump checks for cancellation when a queued entry comes up for
  activation and drops it silently; an in-flight request that is
  canceled completes into a canceled future and its network reply is
  discarded on arrival (still recorded by capture — research data is
  research data).
- **Convention (load-bearing): never `co_await` a future after canceling
  it.** Verified July 18, 2026 against QCoro source: its `QFuture`
  awaiter resumes immediately on cancellation and calls `result()`
  naively — awaiting a canceled future is at best a default-constructed
  value, at worst undefined for non-default-constructible payloads. On
  abort, the worker cancels and returns; its own coroutines guard with
  a generation/liveness check after every `co_await` (see D6).
- This deletes the worker's abort machinery: the five `m_queue = {}`
  sites, the counter-snapping, and most of `DiscardIfStale`.

### D3. Policy managers become coroutine pumps

Each `RateLimitManager` runs one long-lived coroutine per policy:

```
loop:
    entry   = co_await m_queue.next()          // awaitable queue
    if entry canceled: continue
    co_await sleepUntil(m_policy->GetNextSafeSend(m_history))
    for attempt in 1..MAX_ATTEMPTS:            // MAX_ATTEMPTS = 3
        reply = co_await m_gate.send(entry)    // global gate, D5
        record history; m_policy->Update(headers); feed capture; emit signals
        if 429 with Retry-After:
            emit Violation
            co_await sleepFor(retryAfter + bucket + buffer)   // N19
            continue
        deliver result or FetchError into entry.promise
        break
    else: deliver FetchError{RateLimited}
```

- The retry is invisible to callers: the future completes only on a
  final outcome — success, non-retryable error, or retries exhausted.
  P-A (graceful 429 recovery is a first-class requirement) is satisfied
  structurally: a 429 can no longer wedge anything (F57).
- The retried send waits `Retry-After` **plus the applicable timing
  bucket plus buffer** (N19, provisional — the capture instrument will
  confirm; the padding constant is one line to adjust).
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
  lines combined; unit-tested standalone.

### D4. HEAD setup goes async inside the hub; the nested event loop dies

- First request to an unknown endpoint: the hub queues the request,
  fires the HEAD, and completes setup in the HEAD's continuation —
  creating or reusing the policy manager (same policy-name dedup as
  today) and forwarding the queued request(s) to it. Submissions
  arriving mid-setup for the same endpoint queue behind it.
- HEAD exclusivity is enforced at the gate (D5): a HEAD acquires the
  gate exclusively — nothing else in flight while a probe runs. This
  preserves N18/F5 **deliberately** rather than as a side effect of
  accidental serialization, and it holds even with multiple endpoints
  setting up concurrently (each HEAD serializes at the gate).
- The F5 standing constraint is amended, not violated: one HEAD at a
  time, ever, is now a property of the gate. The `QEventLoop` block in
  `SetupEndpoint` is deleted. The main-thread-only `Q_ASSERT` on submit
  stays.
- **Degraded or failed HEADs never kill the app** (decided July 18,
  2026; resolves the Q3-residual design question). The key property
  making this safe is new to the pump design: **exposure is exactly one
  request.** A pump sends one request at a time per policy, and every
  real reply carries the full header set and reseeds the policy — so
  "run unpaced until reseeded" means one GET, further bounded by the
  gate's spacing floor. Concretely, any HEAD outcome short of a full
  header set — rules missing (the Dec 2023 shape, N20), the policy
  header absent entirely, or the HEAD failing outright (the Nov 2023
  "insufficient scope" shape, N20) — is handled the same way: log at
  error, surface a user-visible notification, capture the probe
  (already implemented), and proceed with a **provisional pump** keyed
  by endpoint with an unknown policy. The first real reply supplies the
  policy name and definition; the hub then rekeys the pump into the
  policy-name dedup map (merging into an existing same-name pump if one
  exists, by forwarding its queue). Rationale: both historical HEAD
  regressions were HEAD-specific server bugs that lasted months —
  under fatal-on-failure, every user is dead until GGG ships a fix;
  under degrade-and-proceed the app keeps working, the error still
  gets reported, and a truly broken endpoint just yields a cleanly
  failed update (P-A: recovery over collapse). The N16 sanction is
  still respected — the probe is always sent; we just don't die when
  GGG's end of it breaks. A conservative synthetic default policy was
  considered and rejected: inventing pacing numbers GGG never sent is
  its own risk. `FatalError` leaves the hub entirely.

### D5. The gate: one object for layer 1

A small async gate in the hub through which **every request to
`*.pathofexile.com` passes** — API, legacy website, forum, login league
list. Three properties, each a named constant:

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

Rationale for scope: layer 1 watches the user's IP (N2, N22), so the
gate must see all traffic sharing that IP against GGG's infrastructure —
this is the principled version of the existing centralize-everything
instinct, and it newly covers the forum-shop and login requests that
today bypass the limiter entirely. Their protocols don't change (the
forum keeps its scrape-based limit detection in `Shop`); they simply
acquire the gate before sending. Non-GGG hosts (GitHub, imgur, Sentry,
poecdn) stay outside. This resolves the Q10b design question: the
redesign *coordinates* the other regimes at layer 1 while leaving their
layer-2 handling as-is.

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
  terminal failure aborts the update (cancel outstanding futures, no
  emit); atomic per-reply replacement already guarantees nothing is
  lost. Per-tab retry/durable progress remain M2 concerns and are not
  blocked by anything here.
- The update generation tag shrinks to a coroutine liveness guard
  (checked after each `co_await`, per D2) — the signal-connection
  staleness machinery (`DiscardIfStale` on every handler) goes away.

### D7. The typed facade: `PoeApiClient`

New class (suggested home `src/poe/poeapiclient.{h,cpp}`), owning the
boundary between domain and network:

- Methods (mirroring today's five rate-limited calls): `listStashes`,
  `getStash`, `listCharacters`, `getCharacter`, `getLegacyStashIndex`.
  Each builds the request (absorbing `poe::MakeStashRequest` et al.),
  submits to the limiter, and chains parsing
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

## What gets deleted

`RateLimitedReply`, `RateLimitedRequest`, the worker's `m_queue` /
`m_queue_id` / `SubmitNextItemRequest` / `FetchItems`'s queue walk, the
five error-path queue clears with counter-snapping, most of
`DiscardIfStale` and the generation-tag plumbing, the `SetupEndpoint`
nested event loop, the hub's `FatalError` paths (D4: setup failures
degrade instead of killing the app), F58's dead `last_send` block, and
`tests/fakenetwork.h`'s `FakeRateLimiter` (replaced by a facade fake).
The pinball machine (`ActivateRequest`/`SendRequest`/`ReceiveReply`
state smear) is replaced by the pump loop.

## Dependency: QCoro

- QCoro v0.11.x (verified July 18, 2026: current release, requires
  Qt ≥ 6.8 — satisfied by our 6.11 floor; MIT license), fetched by
  CMake like the other third-party libraries. Used for: awaiting
  `QFuture`/`QNetworkReply`/timers, `QCoro::Task<>` in the worker and
  pumps.
- Qt has **no native** `QFuture` coroutine support as of 6.11; an
  upstreaming effort exists targeting 6.12 at the earliest. If it
  lands, the future-awaiting uses of QCoro can migrate; the
  reply/timer awaitables and `Task<>` keep QCoro relevant regardless.
  No design decision here depends on which awaiter implementation is
  underneath.
- Compiler note: GCC < 13 has a known coroutine bug (captures in
  temporary lambdas); our C++23 toolchain floor already clears it, but
  the build docs should state it.

## Testing plan (harness-first, non-negotiable)

Ordering is part of the spec, because the pump rewrite touches the most
battle-tested code in the app:

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
   cancellation while queued → dropped, never sent; cancellation while
   in flight → canceled future, reply discarded, history still recorded.
3. **Primitives tested standalone**: awaitable queue (FIFO order,
   cancellation), gate (cap, exclusivity, spacing floor) — pure logic,
   fast tests.
4. **Worker tests move to a facade fake** ("when asked for stash X,
   return this tab / this error") — simpler than today's byte-crafting
   `FakeRateLimiter`, and it closes the fake-bypasses-the-managers blind
   spot. Existing worker-update behavior pins (`tst_workerupdate.cpp`)
   are ported, not weakened.
5. Every commit compiles and passes `ctest` (working rule #2). The
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

## Open items — all resolved July 18, 2026 (reviewed with Tom)

1. Gate constants: **blessed as spec'd** — in-flight cap 2, spacing
   floor 250 ms, retry cap 3. Named constants, provisional until
   capture data says otherwise; the revisit trigger for all three is
   capture evidence (lane idling at the gate, spike-correlated
   anomalies, retry exhaustion in the wild).
2. Degraded-HEAD behavior: **degrade and proceed** — decided; folded
   into D4 (one-request exposure, provisional pump, `FatalError`
   removed from the hub).
3. `cancelChain()` semantics: **verified** against Qt docs — available
   since Qt 6.10 (inside our 6.11 floor), propagates the canceled flag
   backwards up a continuation chain, fires `onCanceled()` handlers.
   Documented limitation: propagation does not enter a *nested* future
   returned by a continuation — not applicable here (the facade's
   `.then(parse)` is synchronous; no continuation returns a future).
   Remaining: one pin test in phase 2 asserting cancelChain on a
   facade future flips the pump-side `QPromise::isCanceled()`.
4. Naming: **deferred to phase 3 by design.** `RateLimiter` keeps its
   name (app-facing surface; continuity with this research corpus).
   `RateLimitManager`'s name is decided when phase 3 rewrites the file
   anyway — rename is free at that moment and never again; proposals
   welcome then. Naming care goes to the new primitives (the gate, the
   awaitable queue), which have no history and will be read forever.
