# Phase 5 Planning Handoff — Network Redesign (Items-Worker Rewrite)

**This is planning scaffolding, not a spec.** Do not cite it the way the
frozen spec (`network-redesign.md`) is cited, and do not treat anything here as
an accepted decision unless it is also in the spec or the findings register.
Its job is to let a planning agent — with none of the authoring session's
context — produce a commit-by-commit implementation plan for phase 5 without
re-deriving what is already known. Iterate on the plan here or in a scratch
file; keep this document to *planning*. Delete or move it once phase 5 lands.

Everything below is either (a) distilled from the authoritative sources named
in each section — verify against them, they win on any conflict — or (b) ground
truth verified in the working tree on 2026-07-20 (marked "VERIFIED").

---

## 1. Where phase 5 sits

The network redesign (`docs/design/network-redesign.md`, ACCEPTED rev 8, frozen
for implementation) replaces the rate-limited networking layer. Phases 0–4b are
**implemented and merged to `master`**:

- **0** — QCoro spike + 2,000-tab batch measurement (no flow control needed).
- **1** — `RateLimitManager` harness (`tst_ratelimitmanager.cpp`).
- **2** — QCoro dependency + primitives: `Scheduler`/`TimerScheduler`
  (`scheduler.{h,cpp}`), stop-interruptible `SleepUntil` (`stopsleep.{h,cpp}`),
  the `Gate` (`gate.{h,cpp}`).
- **3** — the coroutine pump inside `RateLimitManager`, the D8 total parse,
  async HEAD setup with parking + cooldown. Resolved F57, F58, F5-modernization.
- **4a** — the `QFuture<FetchOutcome>` boundary (`SubmitFuture`), `FetchError`
  kinds, the `PoeApiClient` facade, `Shop` moved over, transfer-timeout
  invariant. Resolved F60, F50.
- **4b** — the items worker moved onto the facade; `RateLimitedReply` and the
  legacy `Submit()`/callback boundary deleted. Resolved F59.

**Phase 5 is the last phase.** After it, no phase remains.

---

## 2. What phase 5 delivers

The items-worker rewrite (spec **item 5**, design **D6**). It **resolves F56**
— single-lane request serialization that starves the character policy. The core
move: the worker stops doing flow control, submits *all* selected fetches at
once through the facade, and lets the per-policy pumps run their lanes in
parallel. Orchestration moves from a callback pyramid to `QCoro::Task<>`
coroutines. Cancellation becomes a real `std::stop_source` per update. The
queue and the update-generation machinery are deleted.

### "At once" means staged, not literally simultaneous

Content requests cannot be built until their list lands, so the batch is
staged — and within each stage submission is unconditional (no worker-side
pacing, count-downs, or waiting between entries):

1. Determine which lists the selection requires with the **existing M1
   semantics**: `All` / `TabsOnly` require both; a partial refresh requires
   only the location types represented by its selection. Submit every required
   list request immediately. When both are required, submit both before either
   settles (distinct policies — D6). Do **not** turn a stash-only partial
   refresh into character discovery, or vice versa.
2. As **each required** list arrives, submit that list's complete content batch
   immediately. Character content keys off the character list **only** —
   holding it behind a slow stash-list response would reintroduce the
   cross-policy head-of-line blocking that *is* F56. Treat this as decided,
   not an open choice.
3. Child discovery has two existing paths and both must remain explicit:
   Folder children normally arrive nested in the stash list and join that
   list's content batch during recursive `ProcessTab`; Map/Unique children
   normally arrive in the parent's content reply and are appended then as one
   complete child batch. The reply handler still accepts children on any of
   the three parent types and preserves the F49 duplicate-fetch tripwire; this
   phase does not invent a new deduplication policy (see §10 item 4).

### Explicitly OUT of scope (do not design these in)
- **Reprioritization / per-entry cancellation.** The stop token is per-update,
  not per-entry; there is no per-entry cancel and reprioritization would need a
  new mechanism. M2+ concern, deliberately not designed now (spec D6, R7).
- **Per-tab retry / durable progress / UI coalescing.** M2 concerns; they
  compose with this phase but are not blocked by it and are not built here.
- **Worker-side serialization of any kind.** One-at-a-time submission *is* F56.

---

## 3. Authoritative sources — read these first

Read in this order. The spec sections are the contract; the code shows the
current state.

**Spec (`docs/design/network-redesign.md`):**
- **D2** — cancellation: one `std::stop_token` per update; futures always
  finish; `QFuture::cancel()`/`cancelChain()` are never used anywhere; pump
  checkpoints; queued resumption (`request_stop()` returns before any suspended
  coroutine resumes); destroying a task handle is never cancellation; in-flight
  requests are never `abort()`ed; no future retention.
- **D6** — the worker: batch submit, count completions, no flow control. Task
  topology and handle ownership (one update task + one per-fetch task, owned
  members, deferred sweep outside any completing coroutine, abort never destroys
  live handles, sweep destroys completed tasks only); the **post-await identity
  invariant (IR2)**; the **initialize-before-launch invariant**; payload
  extraction via `co_await qCoro(future).takeResult()`; generation tag deleted
  outright.
- **Testing plan items 5 and 6** — the exact pins this phase must add (see §8).
- **Phasing sketch item 5** — the one-paragraph summary.

**Findings (`docs/cleanup/findings.md`):**
- **F56** (open; resolved by this phase) and the **F28** ledger note (the
  `IsStale` guard kept in 4b becomes removable here).
- The **Standing constraints** section (F5, F29 — see §7).

**Items pipeline (`docs/design/items-pipeline.md`) — the Milestone 1
section.** Required reading per `AGENTS.md` before any structural worker
change, and directly load-bearing here: it defines the semantics phase 5
must preserve (atomic per-reply replacement, tab-list reconciliation on
list receipt, terminal-failure-loses-nothing, the rebase-on-finish rule, and
partial updates requesting only the list types their selection needs), and
its M1 commit 1 is the very generation tag this phase deletes.

**Code — the current worker (post-4b):**
- `src/itemsmanagerworker.h` / `.cpp`. Note what 4b already did: takes a
  `PoeApiClient&`; the queue carries `StashFetch`/`CharacterFetch` intent, not a
  `QNetworkRequest`; the four handlers take
  `std::expected<T, RateLimit::FetchError>`; the transport/status/parse checks
  collapsed into one error branch per handler; `DiscardIfStale` is now the
  generation-only predicate `IsStale`. The callback pyramid, `m_queue` /
  `m_queue_id` / `SubmitNextItemRequest` / `FetchItems`'s queue walk, the
  `m_need_*` / `m_has_*` flag pairs, and `m_update_generation` / `IsStale` are
  what this phase removes.

**Code — the boundary and primitives (read for understanding, not to change):**
- `src/poe/poeapiclient.{h,cpp}` — each method returns
  `QFuture<std::expected<T, FetchError>>` and takes a `std::stop_token`.
- `src/ratelimit/ratelimiter.h` (`SubmitFuture`), `ratelimitedrequest.h` (the
  pump's internal queue entry; carries the stop token and the `QPromise`),
  `stopsleep.h`, `gate.h` — how a token flows into the pump and how
  stop-interruptible waits work.
- `src/ratelimit/ratelimitmanager.cpp` — **the existing `QCoro::Task<>`
  coroutine pump in this codebase.** Match its idioms: root-coroutine
  catch-alls, checkpoint placement, how it awaits.

**Tests:**
- `tests/tst_workerupdate.cpp` — the worker suite the new pins extend.
- `tests/fakeapiclient.h` — `FakePoeApiClient`, the typed facade fake the worker
  suite uses (see §6).
- `tests/fakenetworkmanager.h`, `tests/fakescheduler.h`, and
  `tests/fakesender.h` — the offline seams for item 6. The first two reach
  through the hub; `FakeSender` is manager-only (see §8).
- `tests/CMakeLists.txt` — the ordinary helper registers one executable as one
  CTest. Item 6 needs explicit per-scenario process registrations. Read
  `BUILD.md` before planning that CMake change, per `AGENTS.md`.

**Spike references (if present): `spikes/qcoro/`** — the phase-0 spike (S1-\*)
and batch measurement (S2-\*) the design leans on (distilled in §5).

---

## 4. Ground truth VERIFIED in the tree (2026-07-20) — don't re-derive

- **QCoro is already linked** into the core library (`QCoro6::Core`); the
  ratelimit pumps use it, so the worker (same lib) can use coroutines. No CMake
  work needed to *use* QCoro in the worker.
- **The worker runs on the main thread.** `Application` owns it; there is no
  `moveToThread`. Its `m_parser_thread` `QThread` is only for parsing cached
  items at startup and is unrelated to the network path. `Update()` and the
  reply continuations run on the main thread, where an event loop is present —
  so QCoro tasks are viable and resume on the main thread.
- **Destruction order already satisfies the shutdown contract.** In
  `src/application.h`'s `session()` struct the declaration order is
  `rate_limiter` (≈l.79), `api` (≈l.82), `items_manager`, `items_worker`
  (≈l.85), `shop` (≈l.86). Members destruct in reverse, so `shop` and
  `items_worker` are destroyed **before** `rate_limiter` — exactly what D2's
  shutdown contract requires (worker/facade consumers gone before the hub). **Do
  not reorder these members.**
- The one open precondition for this phase — the 2,000-tab batch-submit memory
  measurement — was settled in phase 0 (round S2): **no flow control needed**,
  memory is linear and acceptable (see §5).

---

## 5. Critical spike learnings the design leans on (phase 0: S1-\*, S2-\*)

These are load-bearing for the phase-5 design. A plan that contradicts one of
these is wrong.

- **S1-1 — destroying a suspended task handle DETACHES the frame; it is not
  cancellation.** QCoro 0.13 task handles are refcounted shared references.
  Destroying one mid-suspend does not run the frame's locals' destructors and
  does not stop it; the frame survives and resumes normally if its awaited event
  is later delivered, then self-destructs. No API destroys a suspended frame
  from outside. **Consequence:** the handle sweep must destroy *completed* tasks
  only; shutdown works by destruction-order + no-event-delivery, not by
  cancelling frames.
- **S1-4** — reinforces S1-1: the stop token is the only cancellation channel.
- **S1-5 — stop wakeups are queued.** The stop-interruptible sleep resumes via
  the event loop (`request_stop()` returns before the waiter resumes, ~22 ms
  wake in the spike). No `std::stop_callback` resumes a coroutine synchronously.
- **S1-6 — ready futures continue synchronously.** Qt runs `.then` continuations
  synchronously on an already-finished future, and QCoro resumes without
  suspending on a ready future. **This is why the initialize-before-launch
  invariant exists:** a fail-fast submission (setup cooldown / shutdown) can run
  its completion logic *inside* the batch-submit loop, so all per-update
  counters and batch state must be initialized before the first submission.
- **S1-7 — `co_await qCoro(future).takeResult()` moves the payload out with 0
  copies** (spike-measured). Plain `co_await future` copies via
  `QFuture::result()`. The worker's single-consumer path should use
  `takeResult()`.
- **S1-8 — an exception escaping an unawaited task vanishes silently.** Root
  coroutines need catch-alls. A per-fetch task whose body throws must still count
  its completion and trigger the first-failure stop (R5-1 pin).
- **S1-9 — completion of a *task* awaited by another coroutine resumes the
  awaiter SYNCHRONOUSLY on the completing stack;** `QFuture` boundaries, by
  contrast, resume through the event loop (`QFutureWatcher`). This distinction
  governs where re-entrancy can happen.
- **S2 (batch measurement) — no flow control needed.** ~6.5 KB heap per entry
  for the whole standing machinery (entry + promise + chain + suspended frames +
  watcher + handle), ~12.4 MB at 2,000 tabs, linear to 10,000. Abort ~2–4 ms at
  2,000; the deferred sweep of 2,000 completed frames ~0.3 ms and releases
  everything (heap back to baseline, zero leaks). **Caveat carried from S2:** the
  outcome type's real `sizeof` is baked into three compile-time frame slots per
  entry on the `takeResult` path — `sizeof(expected<poe::StashWrapper,
  FetchError>)` ≈ 336 B, the character wrapper outcome ≈ 744 B. Any memory
  reasoning must use the real payload size, not a stand-in.

---

## 6. What phase 4b left in place for phase 5

- **The worker tests use `FakePoeApiClient` (`tests/fakeapiclient.h`).** It
  records the domain arguments the worker asked with (realm, league, stash id,
  substash id, name), returns `QFuture`s the test settles by call index, and has
  typed `resolve*`/`reject` helpers. It **already records stop tokens** on each
  call (stubbed in 4b specifically for phase 5's cancellation pins — nothing
  asserts them yet). Single completion is enforced by the fake's own guard (a
  second settle `qFatal`s), **not** by `QFuture` semantics.
- **The fake's stop tokens are passive observations.** Requesting stop does not
  settle any pending fake future; production pumps do that, but the facade fake
  deliberately leaves delivery order under the test's control. Every ordinary
  worker test that aborts with siblings outstanding must therefore settle all
  remaining calls (normally as `Canceled`), drain their resumptions and the
  deferred sweep, and assert that no pending call/task remains before fixture
  teardown. Only the isolated shutdown scenarios in item 6 may intentionally
  destroy owners with suspended work.
- **Correct framing of "settles once":** a `QFuture` is a multi-result container
  (`addResult()` may be called repeatedly). Exactly-once completion in
  production comes from the pump's one-shot `CompleteRequest`; in the fake, from
  its guard. Do not write "a QPromise settles once" — it is false and was
  corrected in 4b.
- **`IsStale` / `m_update_generation` are still present** as a defensive check.
  4b established that the F28 stale-arrival scenario is *unreachable* today (the
  worker submits serially and only aborts inside a handler, so nothing is in
  flight when an update aborts). **Phase 5 makes cancellation real and deletes
  this machinery outright** — update identity moves to the per-update stop token
  under the post-await invariant (D6). The current test
  `failedUpdateDoesNotLeakIntoTheNext` pins the reachable half; phase 5's new
  cancellation pins replace what the generation guard used to (not) cover.
- **The "no facade fake" decision was reversed in 4b** — worker tests use
  `FakePoeApiClient`, not a real facade over `FakeRateLimiter`. Keep it.

---

## 7. Standing constraints (findings register) that bind this phase

- **F5 — one HEAD at a time.** Enforced at the gate now (HEAD holds the gate's
  exclusive permit). Nothing in the worker rewrite may reintroduce parallel
  HEADs. The hub runs on the main thread (a `Q_ASSERT` in `RateLimiter`).
- **F29 — logging teardown comes last.** No log call after `spdlog::shutdown()`.
  Not directly touched here, but relevant to any shutdown-path reasoning.

---

## 8. The test plan phase 5 must produce (testing-plan items 5 & 6)

The worker suite keeps using `FakePoeApiClient`. This is the first phase where
production can actually cancel, so most cancellation pins go live here — but
not every spec pin is missing. Status matrix (verified in the tree
2026-07-20):

| Pin | Spec home | Status |
| --- | --- | --- |
| ER1 — failed sibling; suspended sibling resumes into finished `Canceled` | item 2 (pump) | **Exists** — `tst_ratelimitmanager.cpp` (the ER1 regression pin test). |
| IR2 initialize-before-launch | item 5 | Missing — this phase. |
| IR2 post-await identity | item 5 | Missing — this phase. |
| R5-1 throwing body + deferred sweep | item 5 | Missing — this phase. |
| R6-2 — Shop continuation dropped after destruction | item 5 | **Exists** — `tst_shop.cpp` (`continuationDoesNotRunAfterShopDestruction`). Verify it still passes; nothing to write. |
| F56 batch-concurrency pin | neither — see below | Missing — this phase. |
| Item 6 integration / shutdown | item 6 | Missing — this phase (see the architecture note). |

A worker-level ER1-shaped test (the real worker's coroutines as the two
awaiters) is still worth writing, but label it *additional integration
coverage*, not an unmet item-5 requirement — the spec places ER1 in pump
testing, where it already exists.

### The F56 regression pin — the headline test of the phase

Absent from both the spec's item 5 and earlier drafts of this doc: nothing
would fail if `SubmitNextItemRequest`-style serialization quietly returned.
At least one worker test must pin the concurrent shape directly:

- In an update that requires both lists, both requests are submitted before
  either settles; stash-only and character-only partial updates still submit
  only their required list.
- A list's complete content batch is submitted before the first content
  reply is delivered.
- Character requests are in flight while stash requests are still
  outstanding (the exact starvation F56 names).
- List-discovered Folder children join the stash content batch, and
  reply-discovered Map/Unique children are appended as a complete batch rather
  than one at a time.

**Stop-token identity pins** (the fake records tokens — 4b stubbed that
exactly for this): every list/content/child call in one update carries the
same non-default stop token; a terminal failure stops that token; the next
update's calls carry a distinct, unstopped token.

### The porting trap (and the harness migration)

Item 5 says existing worker pins are "ported, not weakened" — but the
existing pins assert serialization *itself*: strict call indices with
exactly one new call after each delivery (`tst_workerupdate.cpp` ≈l.345),
and a single-pass `pump()` documented as sufficient only because the
worker is serial (≈l.255). Ported literally, they would pin the bug F56
removes.

The migration has **two steps**, because a serial worker has not submitted
later calls yet and therefore cannot settle them in arbitrary order:

1. Before behavior changes, add identity-based lookup, observable-condition
   drains, and pending-call inspection; port tests away from raw indices while
   retaining the only delivery order the serial worker can currently accept.
   This is the independently green harness commit.
2. With batch submission, reorder representative deliveries and remove the
   remaining one-new-call-at-a-time expectations. Every behavioral outcome pin
   remains at full strength (§10 item 7).

### Item 5 pins to write, with their test seams

- **IR2 initialize-before-launch** — an already-finished success future and
  an already-finished error future (fail-fast submit) run their
  continuations synchronously during the batch-submit loop without
  corrupting counters. **Seam:** the fake needs pre-completed ("ready")
  futures, because the fail-fast completion runs inside `Update()` before
  the test regains control (§10 item 6). The invariant extends per-child: a
  ready *child* future completes inline too, so counters and
  `m_pending_children` must be updated before **each** child launch, not
  only before the initial batch.
- **IR2 post-await identity** — a fetch that completes successfully an
  instant before `request_stop()` resumes its consumer afterward and
  mutates nothing. **Seam: probably none needed.** Stop wakeups are queued
  (S1-5) and the test settles futures itself: settle a sibling's failure,
  pump (stop is now requested), then settle the success, pump — the
  consumer resumes holding a success payload under a stopped token,
  observationally identical to completed-before-stop, and
  settle-after-stop is itself a legitimate production ordering (that is
  what stragglers are). Plan this ordering first; add a scripted
  cross-call seam only if it proves insufficient.
- **R5-1 throwing body** — a per-fetch task whose body throws counts its
  completion and triggers the first-failure stop; the update finishes
  (aborted), never wedges; the handle sweep runs outside any completing
  coroutine (the last completion does not destroy its own frame). **Seam:**
  the fake returns an *exceptional* `QFuture` — `co_await` rethrows at the
  await site inside the per-fetch body, exactly the R5-1 shape, and it
  doubles as a defense-in-depth pin against a facade that violated IR4.
  Catch-alls are two distinct policies the plan must define separately:
  per-fetch (count the completion + first-failure stop) and the root
  update task (abort + finalize) — S1-8, an escaped exception otherwise
  vanishes silently. "The last completion does not destroy its own frame"
  needs deterministic instrumentation in the normal `ctest` run
  (test-visible sweep executions / ready-handle counts). ASAN is useful
  supplemental coverage, but an ASAN-only crash is not the item-5 regression
  pin and does not satisfy the every-commit-green rule by itself.

**Overlapping-guard proof obligation:** while `IsStale` and the post-await
token check coexist, either guard can make the stale-mutation test pass. The
test only proves that cancellation *replaces* the generation guard after the
generation machinery is removed. In that removal commit, mutation-verify the
pin specifically by deleting/bypassing the post-await check and observing the
test fail; a pass while `IsStale` still masks the path is not evidence.

### Preservation matrix — "same semantics" made concrete

Once completions reorder, "keep the status / reconciliation /
`ItemsRefreshed` semantics" is too loose to implement against. Pin
explicitly (sources: items-pipeline M1, D6, F53/F55):

- List-received and F53 reconciliation signals fire only for successful fresh
  lists processed while their update is still active.
- Successful replies whose consumers **already processed and applied them**
  before a later terminal failure keep their atomic replacement (memory +
  datastore) — nothing already applied is lost. A future that completes before
  stop but whose consumer resumes afterward is instead discarded by the
  post-await invariant; "the reply landed" is too ambiguous for this boundary.
- A terminal failure emits no `ItemsRefreshed` and performs no rebase.
- Canceled siblings do not count as request failures in status accounting.
- Exactly one terminal status transition per update.
- Status counters stay monotonic and include children initialized before
  launch.
- Request selection is unchanged: stash-only / character-only partial updates
  request only their list type, while `All` / `TabsOnly` request both.

### Item 6 (integration / shutdown) — needs an architecture, not a list

The cases:

- Cancellation races across layers (abort mid-pacing-sleep, mid-gate-wait,
  mid-flight).
- **Destruction reaches every suspension** — destroying worker-then-hub in the
  session member order, with no further event-loop iterations, while a pump is
  mid-pacing-sleep / mid-retry-sleep / mid-gate-wait / mid-flight, resumes
  nothing and crashes nothing.
- **Leak bound** — suspended frames detach and leak *by design*, bounded to the
  transitive closure of the detached frames (frames + their awaiters +
  `QFutureWatcher`/context QObjects + retained `QFuture` handles + sleep
  timers). Outside that closure nothing may leak: not a reply (the QNAM parent
  frees it), not an owner object, not anything else. Promises die unfinished; no
  awaiter is resumed to observe them.
- Completed-future retention (made measurable below).

**Isolation is mandatory, not stylistic:** the leak closure includes live
sleep timers. In a shared test process, a later event-loop iteration can fire
one and resume a detached frame into freed pump state. The unit of isolation is
therefore **one destruction scenario per process**, not merely one executable
containing several Qt Test methods. A practical shape is one purpose-built
scenario runner registered as several CTest tests, with one scenario argument
per invocation: drive the event loop only until the named suspension is
reached, destroy worker/facade consumers before the hub and the fake network
parent after the hub, make synchronous sentinel assertions at the relevant
destruction points, and return without another `processEvents()` / `exec()`
turn. A separate executable per scenario is also valid; a multi-case process
is not.

Use `FakeScheduler` plus `FakeNetworkManager` for the real
worker → facade → hub path. `FakeNetworkManager::createRequest()` is the seam
that intercepts both HEAD probes and pump sends; `FakeSender` injects directly
into `RateLimitManager` and therefore cannot provide item-6 hub integration by
itself. Extend the in-flight reply minimally if a scenario needs a response
body. Manager-only cancellation coverage already in `tst_ratelimitmanager`
continues to use `FakeSender`.

The leak strategy must distinguish an accepted detached-frame closure from an
accidental leak: use counted/sentinel assertions for owner and reply destruction
and for "no callback resumed"; any LSAN annotation or suppression must target
only the known detached-frame roots, never blanket the scenario. The normal
worker suite must instead finish every fake future and sweep every task (§6).

**Completed-future retention, made measurable:** the spec places the full-chain
pin under item 6. `FakePoeApiClient` does **not** retain settled payloads — it
resets each promise — but its call metadata and recorded stop tokens still grow
for the fixture lifetime, so process RSS cannot isolate worker retention.
Counted payload lifetime is still useful worker-level support; the item-6 pin
must exercise the real facade/hub chain. State the contract narrowly:
`takeResult()` moves the payload out; body locals unwind when the per-fetch task
completes; the completed frame allocation (and any state inherently resident in
that frame) may persist until the deferred sweep. No dynamic payload/future
population may accumulate beyond that, and the sweep must release the completed
frames. Prefer counted lifetime instrumentation over process RSS; do not assert
that every shared-state reference disappears at the `takeResult()` expression
unless the harness measures that exact fact.

**Item 7 — every commit compiles and passes `ctest`;** `tst_networkcapture.cpp`
must keep passing (captures are live research data).

---

## 9. Testing & implementation gotchas (learned in earlier phases)

- **`deleteLater` posted outside a running event loop** needs
  `sendPostedEvents(nullptr, QEvent::DeferredDelete)` in the drain helper to
  actually fire.
- **Fast paths can complete synchronously.** The gate/pump fast path (S1-6) can
  send/complete inside the submit call when nothing contends — do not assert
  "count == 0" before the first settle.
- **A test whose two candidate outcomes are observationally identical pins
  nothing.** (E.g. "submit lock still held" is equally true whether or not the
  continuation ran.) Make the pin depend on something only the intended path
  produces.
- **Check what a fixture's real collaborators do on the success path.** A worker
  test holding a real `NetworkManager` once fired a live request at
  pathofexile.com; the worker suite must stay fully offline (that is why the
  facade fake exists).
- **A stopped fake call is still pending.** In ordinary worker tests, explicitly
  settle every stopped sibling and drain until the task sweep has run before
  destroying the fixture. Pending work at teardown is allowed only in the
  one-scenario-per-process shutdown runner.
- **For every behavior fix, verify the test FAILS without it.** Standing review
  rule from the phase-3/4 rounds.
- **`clang-format`** touched C++ files; respect `// clang-format off/on` blocks.
- Prefer existing Qt / local patterns over new abstractions
  (`ratelimitmanager.cpp` is the worked example of a coroutine pump).

---

## 10. Open questions / risks for the planning pass to resolve

1. **Coroutine boundaries.** Which exact methods become `QCoro::Task<>` (the
   update entry point, the two list fetches, per-tab/per-character fetches, the
   child-fetch appends), and where the single update task lives vs. the
   per-fetch tasks. Draw these deliberately — the post-await checks and the
   initialize-before-launch ordering depend on the boundaries, and they should
   not be discovered mid-rewrite.
2. **Handle container + deferred sweep — a complete state machine, not just
   a data structure.** The spec gives fragments (deferred sweep, abort never
   destroys live handles, sweep destroys completed tasks only, handles
   destroyed before observable members); the plan must compose them into a
   full lifecycle that answers all of:
   - A terminal failure returns the worker to idle immediately; it does not
     wait for canceled siblings to settle.
   - A new update may begin while stopped stragglers from the old update
     still exist — the container must not conflate the two populations.
   - A straggler completing *after* the abort-time sweep must trigger
     another deferred sweep: one sweep posted at abort or finalization
     cannot reclaim tasks that finish later.
   - Sweeps are queued/coalesced, run outside every completing coroutine,
     and destroy only ready handles (S1-1).
   - The update-task handle gets the same ownership answer as the per-fetch
     handles.
   - New task-handle members are declared so they destruct before any
     worker state their frames could observe (shutdown contract,
     `network-redesign.md` "Shutdown = destruction" section).
3. **Completion-counting vs. finalization ordering.** Today's handlers increment
   counters and then `CheckUpdateFinished()`. Preserve that the completion which
   reconciles the counters triggers finalization, under coroutines, without
   re-entrancy hazards (S1-9: awaiting a *task* resumes synchronously).
4. **Both child-discovery paths.** Folder children normally arrive nested in
   the stash list and must join that list's content batch during recursive
   processing; Map/Unique children normally arrive when the parent reply lands
   and must be appended then as one batch. The existing generic reply path and
   F49 duplicate-fetch tripwire remain (no new deduplication policy). Work out
   how both paths compose with completion counters while preserving
   `m_pending_children` / `m_contents_known` semantics from F55. The
   initialize-before-launch invariant applies **per child launch**, not just to
   the initial batch — a ready child future completes inline before the test
   (or the worker) regains control, so the complete child count and parent
   bookkeeping must exist before the first launch in that child batch (§8).
5. **Interaction with `ItemsManager` / status-bar signals.** `SendStatusUpdate`,
   `ItemsRefreshed`, and the F53 reconciliation signals must keep firing with
   the same semantics; the emitted `Items` still share `Item` objects with the
   UI (the rebase-on-finish rule).
6. **`FakePoeApiClient` capability gaps and teardown discipline.** Confirm what
   the new pins need beyond what 4b built, and note each as a fake-extension in
   the plan. Known so far (§8): identity-based call lookup; pending/settled
   inspection; pre-completed ("ready") success and error futures for the
   initialize-before-launch pin; an *exceptional* future for the R5-1
   throwing-body pin; assertions on recorded stop tokens; and a deterministic
   way to settle every stopped straggler and drain until the sweep is observed.
   The post-await identity pin probably needs **no** cross-call extension —
   settle-after-stop through the existing helpers produces the same
   consumer-visible state (§8); add scripting only if that proves insufficient.
7. **Commit sequencing where every intermediate is behaviorally correct** —
   not merely compiling and passing incomplete tests. An earlier draft's
   example sequence (batch-submit, then delete the generation machinery,
   then add cancellation) was unsafe: after batching, old requests can still
   complete, so the generation guard must outlive batching until cancellation
   and post-await checks are live. The reverse claim was also too strong:
   post-await identity cannot be black-box tested while the worker is serial,
   and while both guards coexist `IsStale` can mask a missing token check. The
   safe skeleton is:
   1. Harness foundation: identity-based call lookup, condition drains, and
      pending-call inspection — green against the current serial worker while
      retaining its only legal delivery order (§8's porting trap).
   2. Coroutine/task ownership plus one update stop source, still serial.
      Pin that every submitted call carries the same non-default token and a
      terminal failure stops it. Add the ready-success / ready-error coverage
      needed to make this eager launch lifecycle independently safe. Install
      the post-await check, but do not claim its stale-success race is
      independently proven yet.
   3. Batch-submit required lists/content/children. Add the F56 shape, ready-
      future batch/child variants, reordered-completion, and cancellation-race
      pins. Both the token and generation guard protect this intermediate.
   4. Delete the worker queue and finish porting representative tests to
      arbitrary completion order.
   5. Delete the generation machinery **last**. Now mutation-verify that the
      post-await identity pin fails when its token check is removed/bypassed;
      this is the proof that cancellation actually replaced the old guard.
   6. Land item-6 integration/retention and one-scenario-per-process shutdown
      coverage in the commit(s) that introduce the required harness, with every
      registered CTest green. The implementation plan may split this by harness
      layer, but may not defer it outside phase 5.

---

## 11. Deliverable shape (what a good phase-5 plan looks like)

An ordered, **commit-by-commit** plan, each commit compiling and passing
`ctest`, matching how phases 3 / 4a / 4b landed (small, reviewable, green
between commits). For each commit: the files/members/methods added, changed, or
deleted, and why. Plus the test plan from §8 mapped to concrete tests, and the
§10 risks resolved or explicitly deferred with a reason. It is a *plan* — no
code is written in the planning pass.

### Process norms (how this project's phases are run)
- Each commit is independently green; the phase lands as one PR with its tests.
- After the PR: the three packaging builds
  (`build-{linux,macos,windows}.yml`) do **not** auto-run — they trigger
  only on version tags or manual `workflow_dispatch`. CodeQL **does** run
  automatically on PRs targeting `master` (`codeql.yml`).
- Review happens in rounds; findings are verified (a fix's test must fail
  without the fix) before they are accepted.
