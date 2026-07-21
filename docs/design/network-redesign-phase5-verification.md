# Phase 5 Verification Contract — Network Redesign

**Status: REQUIRED evidence for phase 5, July 20, 2026.** Production behavior
is defined by `network-redesign.md` revision 10, items-pipeline M1, and findings
F49/F53/F55/F56. This document defines how phase completion is proved. A pin
may be changed when the replacement supplies equivalent or stronger evidence;
record the substitution and reason in the revision note at the end. Test names,
fake APIs, and runner layout remain implementation choices.

The execution order and package mapping live in
`network-redesign-phase5-plan.md`.

---

## 1. Coverage boundaries

| Layer | Production objects | Test seam | Responsibility |
| --- | --- | --- | --- |
| Worker unit | Real `ItemsManagerWorker` | `FakePoeApiClient` | Selection, staged batches, update identity, reconciliation, counters, task lifecycle |
| Facade unit | Real `PoeApiClient` | Fake limiter | Request construction, endpoint labels, typed parsing boundary |
| Manager/hub | Real limiter/managers | Fake sender/network + fake scheduler | Pump cancellation, pacing, gate, setup, reply ownership |
| Phase-5 integration | Real worker → facade → hub | Fake scheduler + `FakeNetworkManager` | Cross-layer cancellation, shutdown, retention |

The worker fake deliberately does not run the managers. `FakeSender` injects
inside `RateLimitManager` and cannot substitute for the full-chain integration
runner.

`R6-2` already exists in `tst_shop.cpp` as
`continuationDoesNotRunAfterShopDestruction`; keep it green but do not duplicate
it.

---

## 2. Worker harness contract

Before production batching changes, the worker harness must provide:

- Identity-based lookup by realm/league/stash id/substash id/character name as
  appropriate. Tests must not use a global call index as request identity.
- Pending/settled inspection and a guard against settling one fake call twice.
- Condition-driven event draining, including
  `sendPostedEvents(nullptr, QEvent::DeferredDelete)` for `deleteLater()` work
  posted outside a continuously running event loop.
- Pre-completed success and error futures for synchronous ready paths.
- Exceptional futures that rethrow at `co_await` for the R5-1 containment pin.
- Stop-token observations and a deterministic way to settle every stopped
  straggler.
- Read-only task-sweep observation: scheduling/execution counts, ready handles
  observed/destroyed, and remaining live handles, including old-update
  stragglers. Prefer a friend fixture, test-build-gated hook, or injected
  observer over a standing production API. The seam observes; it never drives
  lifecycle.

The harness migration is two-stage:

1. Port identity and drains while delivering calls in the current serial order.
   This commit must stay green against the existing worker.
2. After batching makes later calls available, reorder representative
   deliveries and remove all one-new-call-at-a-time expectations.

Ordinary worker tests must settle every pending fake future, drain all
resumptions and deferred sweeps, and assert that no pending call/task remains
before fixture teardown. Only the isolated shutdown runner may intentionally
destroy owners with suspended work.

---

## 3. Worker concurrency and lifecycle pins

| ID | Required evidence | Primary package |
| --- | --- | --- |
| `M-ER1` | Two manager fetches share a stop source; one terminal failure requests stop while its sibling is suspended; the sibling resumes from a finished `Canceled` value | 5A or adjacent green commit |
| `W-F56-LISTS` | With pending fake futures, an update requiring both lists submits both before the harness settles either; stash-only and character-only partial updates submit only their required list | 5C |
| `W-F56-CONTENT` | Each arriving list submits its complete content batch before any content reply is delivered; character calls exist while stash calls remain outstanding | 5C |
| `W-F56-CHILDREN` | Folder children join their list batch; Map/Unique reply-discovered children launch as one complete batch | 5C |
| `W-F56-ORDER` | Deliberately non-sorted identities retain source traversal order within each policy lane; no global lane order is asserted | 5C |
| `W-TOKEN` | Every list/content/child call in an update has the same non-default token; failure stops it; the next update uses a distinct unstopped token | 5B–5C |
| `W-INIT` | Ready success/error during initial, content, and child launch cannot finalize early or corrupt counters/parent bookkeeping; the complete initialized stage is still invoked with the stopped token after an inline failure | 5B–5C |
| `W-ER1` | One of two in-flight worker fetches fails; the shared token stops; the fake sibling settles `Canceled`; it mutates no state and adds no failure | 5C |
| `W-IDENTITY` | A stopped old update's successful straggler resumes while a subsequent update is active and mutates nothing belonging to the new update | 5D |
| `W-THROW` | An exceptional future is caught by its per-fetch root; it routes through the direct terminal abort (not counted as received), first-failure stop occurs, and the update aborts instead of wedging | 5B |
| `W-SWEEP` | The last completion schedules a sweep outside its own coroutine; only ready handles are destroyed; a later straggler schedules another sweep | 5B–5D |

### F56 headline construction

One scenario may cover several `W-F56-*` rows, but a test must fail if
`SubmitNextItemRequest`-style serialization returns. Use both required lists,
multiple deliberately non-sorted stash and character identities, a nested
Folder child, and reply-discovered Map/Unique children. Establish the complete
call population before delivering the first content result. Assert lane-local
order separately from cross-lane concurrency.

### Initialize-before-launch

Ready futures can complete inside the submit loop before `Update()` returns.
Initialize the whole initial batch before its first launch and initialize each
child count and `m_pending_children` entry before the corresponding child
launch. Cover ready success and ready terminal error; a ready child is not a
special case.

### Exception and sweep policies

Per-fetch and root catch-alls prove different behavior:

- A per-fetch catch-all routes that fetch through the first-failure path's
  direct terminal abort; the fetch is not counted as received.
- The update root catch-all aborts/finalizes if orchestration itself throws.

An exceptional facade future is defense in depth against an IR4 boundary
violation and supplies the real `co_await` throw shape. ASAN may supplement
`W-SWEEP`, but deterministic normal-`ctest` observability is required; an
ASAN-only crash does not satisfy the pin.

---

## 4. Preservation contract under reordered completion

Several behaviors predate phase 5. Pin the desired behavior, not incidental
serial call indices. When no pin exists yet, first demonstrate the current
worker's behavior so the baseline is evidence rather than assumption.

| ID | Required evidence | Authority |
| --- | --- | --- |
| `P-LIST-SIGNALS` | List-received and F53 reconciliation signals occur only for a successful fresh list processed while its update remains active | M1, F53 |
| `P-APPLIED` | A reply processed and atomically applied before a later terminal failure remains applied in memory and datastore | M1 |
| `P-FAILURE` | Terminal failure emits no `ItemsRefreshed`, performs no rebase, and produces one terminal Ready/failure transition | M1, D6 |
| `P-STATUS` | Counters are monotonic, include children before launch, and do not count canceled siblings as request failures | D6 |
| `P-CONTENTS-KNOWN` | A location becomes known only after successful content application; failure/cancellation leaves it retryable | F55 |
| `P-CHILD-CYCLE` | Starting a Map/Unique child cycle evicts the parent; only all enabled child successes restore it; failed/canceled children keep it retryable | F55 |
| `P-ORDER` | Content calls retain D6 source traversal order within each policy lane | D6 |
| `P-SELECTION` | Partial refreshes request only represented list types; `All`/`TabsOnly` request both; `TabsOnly` fetches no content | M1, D6 |
| `P-REFUSE` | `Update()` during an initialized active update notifies/refuses, submits nothing, and neither stops, replaces, nor queues behind it; initialization deferral remains separate | D6 |
| `P-REBASE` | Surviving shared `Item` locations rebase only on successful finalization immediately before the refresh emit | M1 |

The phrase “reply landed” is too ambiguous for `P-APPLIED`: a future may hold
a success before stop while its consumer resumes afterward. Such a consumer is
discarded by `W-IDENTITY`; only a consumer that already processed and applied
the reply is preserved.

Both child-discovery paths remain accepted. The generic reply path continues
to accept children on all three existing parent types. Preserve F49's
duplicate-fetch tripwire; this phase does not introduce deduplication behavior.

---

## 5. Identity-replacement mutation proof

While `IsStale` and the post-await token check coexist, either can make
`W-IDENTITY` pass. The test proves replacement only in the commit that deletes
the generation machinery.

The observable must include a subsequent active update. A successful old
straggler that mutates only state which is immediately discarded may be
observationally indistinguishable from correct behavior. Construct the test so
the old success would corrupt the next update's state, then:

1. Run the test with generation machinery removed and the token check present;
   it passes.
2. Delete or bypass the immediate post-await check; the test fails for the
   intended state corruption.
3. Restore the check; the test passes.

Record this fail/pass mutation evidence in the commit or review notes.

---

## 6. Full-chain integration, shutdown, and retention

| ID | Required evidence |
| --- | --- |
| `I-CANCEL-PACING` | A worker update aborts while its real pump is suspended in pacing sleep; the chain completes with the specified canceled behavior |
| `I-CANCEL-GATE` | Abort while a worker request is waiting at the gate wakes and completes it without sending |
| `I-CANCEL-FLIGHT` | Abort in flight lets the reply land/record, then delivers cancellation without aborting the reply |
| `I-SHUT-PACING` | Worker-then-hub destruction while pacing-suspended resumes nothing after destruction |
| `I-SHUT-RETRY` | The same proof while retry-suspended |
| `I-SHUT-GATE` | The same proof while gate-suspended |
| `I-SHUT-FLIGHT` | The same proof with an in-flight reply; the fake QNAM parent frees the reply |
| `I-LEAK-BOUND` | Detached leaks are limited to the transitive closure of detached frames; owners and replies outside that closure are destroyed |
| `I-RETENTION` | Across a long full-chain update, dynamic completed payload/future population does not accumulate; deferred sweep releases completed frames |

### Scenario-runner architecture

Isolation is required because each shutdown proof intentionally leaves
detached frames and depends on no event-loop iteration after owner destruction.
A shared Qt Test process must continue to later methods and violates that
premise. Use one purpose-built runner registered as multiple CTest tests with a
scenario argument, or one executable per scenario. One invocation must run
exactly one destructive scenario.

For each scenario:

1. Use `FakeScheduler` plus `FakeNetworkManager` on the real worker → facade →
   hub path.
2. Drive the event loop only until the named suspension is reached.
3. Destroy worker/facade consumers before the hub, and the fake network parent
   after the hub, matching `UserSession` ownership.
4. Make synchronous sentinel assertions for owner/reply destruction and “no
   callback resumed.”
5. Return without `processEvents()` or `exec()` after destruction.

Extend the fake in-flight reply minimally if response bodies are needed.
`FakeSender` remains manager-only and is insufficient for these pins.

### Leak and retention interpretation

Suspended QCoro 0.13 frames detach at shutdown and leak by accepted design.
Their transitive closure includes frame allocations, locals/awaiters,
`QFutureWatcher` and context QObjects, retained `QFuture` handles/shared state,
and sleep timers. Nothing outside that closure may leak—not a reply, owner, or
unrelated object. Any LSAN annotation/suppression must identify only the known
detached roots; never suppress the whole runner.

For retention, prefer counted payload lifetime over process RSS. Fake facade
call metadata and recorded tokens grow for the fixture lifetime, so its RSS
cannot isolate worker retention. The full-chain test must establish the narrow
contract: `takeResult()` moves the payload; body locals unwind when the task
completes; the completed frame may remain until the deferred sweep; dynamic
payload/future populations do not accumulate beyond that and the sweep releases
completed frames.

---

## 7. Shared traps and completion gate

- Ready futures and `.then()` fast paths may complete synchronously inside a
  submit call; never assume a count remains zero until a later event turn.
- Stop wakeups are queued, but completion of an awaited `QCoro::Task<>` resumes
  its awaiter synchronously on the completing stack.
- A stopped fake call remains pending until the test settles it.
- `QFuture`/`QPromise` are multi-result capable; exactly-once completion comes
  from the production `CompleteRequest` helper and the fake's settle guard, not
  from the Qt container type itself.
- Make each test outcome distinguish the intended path from its alternative.
- Keep all worker tests offline; no real `NetworkManager` success path belongs
  in the unit fixture.
- For every behavior fix, verify the new test fails without the fix.

Phase 5 is complete only when:

- Every `M-*`, `W-*`, `P-*`, and `I-*` row is green or has a recorded
  equivalent/stronger replacement.
- Every ordinary fixture proves no pending fake calls/tasks remain.
- Every shutdown scenario runs in its own process invocation.
- The full checked-in `ctest` suite passes, including
  `tst_networkcapture.cpp`.
- F56 is moved to the resolved ledger with the implementation reference.

---

## Revision note

- **July 21, 2026 — W-THROW failure counting.** Dropped the "completion is
  counted" clause from `W-THROW` (and the matching exception-policy bullet in
  §3). The implementation finalizes a failed update through a direct terminal
  `AbortUpdate()` that is independent of the completion counters; counting a
  failed fetch as received is functionally dead (no path reads the counter on
  failure) and would report a failed request as "received." The anti-wedge
  guarantee `W-THROW` protects is now supplied by the direct abort, which is
  stronger than counter reconciliation because it does not depend on every
  straggler counting. Equivalent-or-stronger evidence; no production behavior
  change. Success-path counting is unchanged.
- **July 20, 2026 — initial contract.** Extracted from the phase-5 planning
  handoff after rev 9 promoted staged batching, active-update behavior, shared
  token identity, and preservation outcomes into the accepted spec. This file
  retains the evidence mechanisms and stable local pin IDs without becoming a
  second production-behavior specification.
