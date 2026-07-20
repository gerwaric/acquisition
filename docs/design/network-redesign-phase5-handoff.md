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
- The **Standing constraints** section (F5, F29, F42 — see §7).

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
- **F42 — never mutate `logger->sinks()` outside `logging::init`.** Unrelated to
  the worker, listed for completeness.

---

## 8. The test plan phase 5 must produce (testing-plan items 5 & 6)

The worker suite keeps using `FakePoeApiClient`. These pins were harness-only or
absent until now; phase 5 makes them **live** (this is the first phase where
production can actually cancel). Map each to a concrete test:

**Item 5 (worker-level):**
- **ER1 regression pin** — two awaited fetches; one fails and stops the update
  while the sibling is still suspended on its own future; the sibling must
  resume safely into a finished `Canceled` future (never into `result()` on a
  resultless future). This is *the* pin the whole D2 design exists to satisfy.
- **IR2 initialize-before-launch** — an already-finished success future and an
  already-finished error future (fail-fast submit) run their continuations
  synchronously during the batch-submit loop without corrupting counters.
- **IR2 post-await identity** — a fetch that completes successfully an instant
  before `request_stop()` resumes its consumer afterward and mutates nothing.
- **R5-1 throwing body** — a per-fetch task whose body throws counts its
  completion and triggers the first-failure stop; the update finishes (aborted),
  never wedges; the handle sweep runs outside any completing coroutine (the last
  completion does not destroy its own frame).
- **R6-2** — a context-bound `Shop` continuation does not run after `Shop` is
  destroyed (Shop stays callback-style; verify it still holds).

**Item 6 (integration / shutdown):**
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
- Completed-future retention — memory does not grow with completed fetches
  across a long update (the deferred sweep releases them).

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
2. **Handle container + deferred sweep.** Concrete data structure for the
   per-fetch task handles; how the sweep is triggered (which completion) and
   how it runs *outside* the completing coroutine so the last completion does
   not free its own frame.
3. **Completion-counting vs. finalization ordering.** Today's handlers increment
   counters and then `CheckUpdateFinished()`. Preserve that the completion which
   reconciles the counters triggers finalization, under coroutines, without
   re-entrancy hazards (S1-9: awaiting a *task* resumes synchronously).
4. **Children appended mid-update.** Folder/Map/Unique children are still
   discovered when a parent reply lands and must be submitted then; work out how
   that composes with a batch-submitted update and the completion counters
   (`m_pending_children`, `m_contents_known` semantics from F55 must be
   preserved).
5. **Interaction with `ItemsManager` / status-bar signals.** `SendStatusUpdate`,
   `ItemsRefreshed`, and the F53 reconciliation signals must keep firing with
   the same semantics; the emitted `Items` still share `Item` objects with the
   UI (the rebase-on-finish rule).
6. **`FakePoeApiClient` capability gaps.** Confirm what the new pins need beyond
   what 4b built — e.g. a pre-completed ("ready") future for the
   initialize-before-launch pin, and asserting the recorded stop token. Note
   these as fake-extensions in the plan.
7. **Commit sequencing that keeps every intermediate green.** The rewrite is
   large; identify a sequence (e.g. introduce coroutine plumbing behind the
   existing flow, then batch-submit, then delete the queue, then delete the
   generation machinery, then add cancellation) where each step compiles and
   passes `ctest`.

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
- After the PR, CI (3-platform build + CodeQL) does **not** auto-run on PRs — it
  is dispatched manually via `workflow_dispatch`.
- Review happens in rounds; findings are verified (a fix's test must fail
  without the fix) before they are accepted.
