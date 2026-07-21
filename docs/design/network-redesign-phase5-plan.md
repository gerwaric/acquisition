# Phase 5 Execution Plan — Network Redesign (Items-Worker Rewrite)

**Status: ACTIVE execution plan, updated July 21, 2026.** Packages 5A–5C are
implemented, committed, and green; 5D is next. This document organizes the work;
it does not define production
behavior or the evidence required for acceptance. Required behavior lives in
the accepted network spec, the shipped items-pipeline M1 contract, and the
findings register. Required evidence lives in
`network-redesign-phase5-verification.md`.

This plan is deliberately temporary. Update its package status and session
handoffs while phase 5 is in flight, then remove it after the phase lands; git
history retains the implementation record.

---

## Outcome and boundary

Phase 5 is the final network-redesign phase. It resolves F56 by replacing the
items worker's mixed, one-in-flight queue with staged batch submission through
`PoeApiClient`. Orchestration becomes owned `QCoro::Task<>` coroutines; every
request in an update shares one `std::stop_source`; the worker queue and update
generation machinery are deleted.

The phase remains one acceptance unit and one PR. The packages below are
development/session boundaries, not independently accepted design phases.
Splitting the acceptance unit would either ship lifecycle scaffolding without
the F56 result or ship batching without the full-stack cancellation and
shutdown evidence required by testing-plan item 6.

Out of scope:

- Per-entry cancellation or reprioritization.
- Per-tab retry, durable progress, or UI signal coalescing.
- Whole-update replacement/coalescing when `Update()` is called during an
  active update.
- Any worker-side pacing or one-at-a-time submission.
- A new child-fetch deduplication policy.

---

## Authority and change rules

| Layer | Source | Owns | How it changes |
| --- | --- | --- | --- |
| Behavior contract | `network-redesign.md` rev 9, items-pipeline M1, findings F49/F53/F55/F56 | Required production behavior and observable semantics | Amend deliberately before implementing a conflicting behavior |
| Verification contract | `network-redesign-phase5-verification.md` | Evidence required to call phase 5 complete | Replace only with equivalent or stronger evidence, recording the reason |
| Execution plan | This document | Packages, commits, files, seams, and session order | Expected to evolve while the two contracts remain satisfied |

Do not copy behavior into this plan as a competing rule. Cite the behavior
contract. Do not silently drop a verification ID because a planned seam became
awkward; revise the verification contract with equivalent evidence.

### Required reading by package

| Package | Read before working |
| --- | --- |
| 5A | Verification §§1–2 and §7; `tests/tst_workerupdate.cpp`; `tests/fakeapiclient.h`; the manager ER1 test when that debt is included |
| 5B | Spec D2, D6, “Shutdown and task ownership,” and testing-plan item 5; spike findings S1-1/S1-5–S1-9; current worker header/implementation |
| 5C | Spec D6; items-pipeline M1; findings F49/F53/F55/F56; verification §§3–4 |
| 5D | Spec D6's post-await identity and generation deletion; verification §§3–5 |
| 5E | Spec D2, shutdown contract, and testing-plan item 6; verification §6; `BUILD.md`; test CMake and the fake scheduler/network seams |

The spec and source documents win if a code anchor in this plan drifts.

---

## Verified entry state

Verified in the working tree on July 20, 2026; recheck only if the named code
has changed:

- Phases 0–4b are merged. The worker already uses `PoeApiClient`; the legacy
  reply/callback boundary is gone.
- QCoro is linked into the core library. The worker runs on the main thread and
  has an event loop; its parser `QThread` is unrelated to the network path.
- `Application::UserSession` already destroys `Shop` and the items worker
  before the facade and rate limiter. Do not reorder those members.
- The phase-0 measurement accepted full batching: about 12.4 MB at 2,000 tabs,
  linear growth, millisecond abort, and a cheap deferred sweep. It used the real
  outcome sizes (stash expected about 336 B; character expected about 744 B);
  do not replace them with a smaller stand-in in later memory reasoning.
- `FakePoeApiClient` records typed call arguments and stop tokens. Its tokens
  are passive observations: requesting stop does not settle a pending fake
  future.
- The worker still contains the callback pyramid, `m_queue`, `m_queue_id`,
  `SubmitNextItemRequest`, flag pairs, `m_update_generation`, and `IsStale`.
- `tst_workerupdate` assumes serial call indices and a single-pass event pump;
  the harness must change before batching.

---

## Cross-package invariants

Every package must preserve these, including intermediate commits:

1. Initialize every counter and parent/child bookkeeping entry before launching
   a future that may already be ready.
2. Check the update token immediately after every `co_await` and before touching
   worker state.
3. Own one root update-task handle and every per-fetch task handle; no
   fire-and-forget task exists.
4. A deferred, queued/coalesced sweep runs outside completing coroutines and
   destroys completed handles only. Abort never destroys a live handle.
5. A failed old update may have stopped stragglers while a new update is active;
   task storage and sweep state must not conflate those populations.
6. No exception escapes a root coroutine. Per-fetch and root catch-alls have
   distinct completion/finalization responsibilities.
7. `QFuture::cancel()`/`cancelChain()` and `QNetworkReply::abort()` are never
   cancellation mechanisms.
8. The generation guard outlives batching and is deleted last, after the token
   check is live and mutation-verified.
9. Every ordinary worker test settles all fake futures and drains the deferred
   sweep before fixture teardown. Only isolated shutdown processes may leave
   suspended work.
10. Every commit compiles and passes the full checked-in `ctest` suite;
    `tst_networkcapture` remains green.
11. F5's one-HEAD-at-a-time gate property and F29's logging-last teardown order
    remain binding even when a package does not intend to touch them.

New task-handle members must be declared so they destruct before every worker
member their frames could observe.

---

## Dependency map and package status

```text
5A Harness foundation
        |
        v
5B Task lifecycle + update token, still serial
        |
        v
5C Staged batching + cancellation behavior
        |
        v
5D Queue/generation deletion + replacement proof
        |
        v
5E Final integration, shutdown, and retention evidence

5A --------------------> 5E runner scaffolding may begin early
```

| Package | Status | Exit artifact |
| --- | --- | --- |
| 5A — harness foundation | **Complete** | Serial worker tests use identity/condition helpers; inherited manager ER1 debt is closed or explicitly separated |
| 5B — task lifecycle | **Complete** | Owned coroutine topology and one stop source work while request submission remains serial |
| 5C — staged batching | **Complete** | F56 shape and worker cancellation pass under reordered completion |
| 5D — cleanup and proof | **Next** | Queue and generation machinery are gone; post-await identity mutation test fails without the check |
| 5E — full-stack verification | Pending | Every item-6 scenario and retention pin is registered and green in isolation |

Only this table carries package status. Avoid duplicating progress prose across
the verification contract or authoritative documents.

---

## 5A — Harness foundation

**Objective:** make the existing worker suite capable of describing batched,
out-of-order work without changing production behavior.

Work:

- Add identity-based call lookup to `FakePoeApiClient` using domain identity,
  not a global call index.
- Add pending/settled inspection and a deterministic way to settle every
  stopped straggler.
- Replace one-pass `pump()` assumptions with condition-driven drains that also
  deliver deferred deletes.
- Port existing tests away from raw indices while retaining the only delivery
  order the current serial worker can accept.
- Add pre-completed success/error and exceptional-future capabilities when
  their first consumer test lands; do not add speculative scripting.
- Close `M-ER1` in its own green commit. It may share 5A only if it genuinely
  shares harness work; otherwise keep it as a separate adjacent commit.

Required evidence: harness rules in verification §2 and `M-ER1`.

Exit criteria:

- Production worker behavior is unchanged.
- Existing semantic assertions are not weakened.
- Pending fake work is observable and fixture teardown can prove it is empty.
- The suite is ready to deliver calls in arbitrary identity-selected order
  once later calls become concurrently available.

---

## 5B — Task lifecycle and update identity, still serial

**Objective:** install the lifetime/cancellation foundation before enabling
multiple in-flight worker calls.

Before code, draw and record in the commit plan:

- The exact root-update, list-fetch, content-fetch, and child-fetch coroutine
  boundaries.
- The update-task and per-fetch handle containers, including old stopped
  stragglers and the coalesced deferred-sweep state machine.
- Which method performs first-failure stop, which counts a completion, and
  which performs the single terminal transition.
- How ready child launches initialize both counters and `m_pending_children`
  before each launch.

Implementation:

- Add the owned task topology and one `std::stop_source` per update.
- Thread the same non-default token through every facade call.
- Use `co_await qCoro(future).takeResult()` and check the token immediately
  afterward.
- Add per-fetch and root catch-alls; no stored task exception is relied on.
- Schedule a sweep after every completion that may make another handle ready,
  including a late straggler after an earlier abort-time sweep.
- Keep the current serial queue temporarily. Keep `IsStale` and the generation
  tag as an overlapping guard.

Required evidence: `W-TOKEN`, `W-INIT`, `W-THROW`, and `W-SWEEP`. Install the
post-await check, but do not claim `W-IDENTITY` has proved replacement of the
generation guard yet.

Exit criteria: the worker is behaviorally serial and green, but coroutine
lifetime, exception containment, ready-future behavior, and update stop
identity are independently exercised.

---

## 5C — Staged batching and cancellation behavior

**Objective:** resolve F56 while both token and generation guards protect the
intermediate implementation.

Work:

- Launch all required lists without awaiting one another. With ordinary
  pending futures they are concurrently outstanding; synchronous ready paths
  remain valid and are covered separately.
- Initialize the complete launch set first and invoke every member of a stage
  even if an earlier ready completion stops the token.
- Let each successful required list independently launch its complete content
  batch.
- Include recursively discovered Folder children in the list batch and launch
  reply-discovered Map/Unique children as complete batches.
- Preserve source traversal order within each policy lane without imposing a
  global stash/character order.
- Support arbitrary completion order and the first-failure stop path.
- Initialize every batch and child count before its first launch, including
  ready futures that finish inline.

Required evidence: all `W-F56-*` pins, `W-ER1`, the ready batch/child variants
of `W-INIT`, and the preservation contract under reordered completion.

Exit criteria: character work is demonstrably in flight while stash work is
outstanding; a stopped sibling resumes safely and mutates nothing; both child
paths and per-lane priority are pinned.

---

## 5D — Queue deletion and identity replacement proof

**Objective:** remove obsolete state only after its replacement is active and
observable.

Recommended green commits:

1. Delete `m_queue`, `m_queue_id`, `QueueRequest`, `FetchItems`,
   `SubmitNextItemRequest`, and the remaining one-new-call-at-a-time test
   assumptions. Keep the generation guard.
2. Construct `W-IDENTITY` with a subsequent update active, remove the generation
   machinery, and mutation-verify that bypassing the post-await token check now
   fails the test.

The first commit completes the harness migration begun in 5A. The second is
the proof that cancellation identity actually replaced `IsStale`, rather than
being masked by it.

Required evidence: `W-IDENTITY`, every `P-*` preservation pin, and the full
worker suite under representative reordered completion.

Carried from the 5C review (deferred here deliberately, not gaps in 5C):

- Missing `P-*` pins to add with the rest of the preservation contract:
  `P-REFUSE` (an `Update()` during an initialized active update notifies/refuses,
  submits nothing, and leaves the active token/update untouched); the negative
  half of `P-LIST-SIGNALS` (a failed or stopped list emits no list-received or
  F53 reconciliation signal); and reply-discovered children accepted on all three
  parent types — the suite covers Map reply children and Folder list children,
  but not Folder/Unique reply children. Note when writing the Folder case: a live
  Folder reply carries no children (F49), so that pin exercises a defensive code
  path the live API does not produce — say so in the test.
- Test-seam consolidation: the worker header now exposes several standing
  test-only entry points (`SetSweepObserver`, `SetFaultHook`,
  `OutstandingFetchTasksForTest`, `ProgressCountersForTest`). Verification §2
  prefers a friend fixture / test-build-gated hook / injected observer over a
  standing production API; consolidate these behind one seam before 5D/5E add
  more observability.
- Note: `FetchItems`/`SubmitNextItemRequest` were already deleted in 5C, so the
  first commit above only needs to remove `m_queue`/`m_queue_id`/`QueueRequest`
  and the one-new-call-at-a-time test assumptions.

Exit criteria: no queue/generation symbols remain; old stopped stragglers and a
new active update coexist safely; the mutation test detects state corruption
when the token check is removed.

---

## 5E — Full-stack integration, shutdown, and retention

**Objective:** prove the real worker → facade → hub chain, including the
post-event-loop destruction contract that the facade fake cannot exercise.

Work:

- Build the purpose-specific scenario runner described in verification §6
  using `FakeScheduler` and `FakeNetworkManager`.
- Register one CTest invocation per destruction scenario. Each process drives
  only to its named suspension, destroys worker/facade consumers before the
  hub and the fake network parent after it, performs synchronous sentinels,
  and exits without another event-loop turn.
- Exercise cross-layer abort during pacing sleep, gate wait, and in-flight
  transfer.
- Prove every shutdown suspension, the accepted detached-frame leak closure,
  reply/owner destruction, absence of callback resumption, and bounded
  completed-future/payload retention.

Runner scaffolding may begin after 5A, but final scenarios depend on the 5D
worker topology. Read `BUILD.md` before changing test registration.

Required evidence: every `I-*` pin in verification §6.

Exit criteria: all scenario processes and the ordinary suite are green; no
ordinary test leaks pending fake calls/tasks; item-6 coverage is not deferred
outside phase 5.

---

## Commit and session discipline

The expected commit sequence is 5A, 5B, 5C, the two 5D commits, then the 5E
runner/scenarios; 5E scaffolding may be pulled earlier when independently
green. This is a recommended decomposition, not a behavior contract.

At the beginning of a development session, read this document through the
current package, that package's routed sources, and only the referenced rows of
the verification contract. End the session with:

- Package/commit state and the last green test command.
- Any verification IDs added, still failing, or deliberately replaced.
- Remaining fake futures/tasks and sweep state (normally zero).
- The exact next safe edit and any invariant that makes ordering important.

For every behavior fix, demonstrate that its regression test fails without the
fix before accepting it. Format touched C++ with `.clang-format` and preserve
nearby Qt/local patterns.

At PR closeout, CodeQL runs automatically for PRs targeting `master`. The
Linux/macOS/Windows packaging workflows run only for version tags or manual
`workflow_dispatch`; invoke them deliberately if release validation is wanted.
