# Phase-5E LSan — Session A (local, self-contained)

Paste this whole file as the task for a fresh session. It assumes no prior
conversation and no persistent memory. Everything you need is inlined; verify
claims against the repo before relying on them.

## Goal

Add a Linux CI job that runs the phase-5 network-redesign integration tests under
AddressSanitizer + LeakSanitizer, to enforce **I-LEAK-BOUND** leak-boundedness
(the last open item of network-redesign phase 5E). This session does only the
**local, deterministic** part: author the workflow and a seeded suppression file,
wire the docs, and push so the first CI run starts. Suppression tuning and the
mutation check happen in a follow-up (Session B), because LeakSanitizer cannot run
on macOS and needs real CI output.

Work on branch `ratelimit-phase5-worker`.

## Context you need (verify in the repo)

- Repo: `acquisition`, a C++23 Qt 6.11 Widgets app; coroutines via QCoro v0.13.0.
  Build/test basics are in `BUILD.md`. Compiler floors: GCC 13+, Clang 15+.
- The phase-5E full-chain integration runner already exists:
  `tests/tst_workerintegration.cpp`. It drives the REAL worker → real
  `PoeApiClient` → real `RateLimiter` hub with a `FakeScheduler` +
  `FakeNetworkManager`. It has a **custom `main()`** that requires exactly one
  scenario argument (rejects 0 or >1 with exit 2). `tests/CMakeLists.txt`'s
  `acq_add_scenario_test` registers ONE CTest per scenario, named
  `tst_workerintegration.<slot>`. Scenarios: `fullChainStashListSucceeds`,
  `i_cancel_pacing`, `i_cancel_gate`, `i_cancel_flight`, `i_shut_pacing`,
  `i_shut_gate`, `i_shut_flight`, `i_shut_retry`, `i_leak_bound`, `i_retention`.
- A test/CI-only CMake knob **`ACQ_SANITIZE` already exists** (in `CMakeLists.txt`,
  placed before the FetchContent block): `-DACQ_SANITIZE=address` injects
  `-fsanitize=address -fno-omit-frame-pointer -g` globally so the whole linked
  image is instrumented. OFF by default; release packaging never sets it. `BUILD.md`
  has a "Sanitizer Builds (test/CI only)" section that currently calls the LSan CI
  job "a separate follow-up".
- The verification contract for this work is
  `docs/design/network-redesign-phase5-verification.md` §6 ("Leak and retention
  interpretation"). Read it. Key line: *"Any LSAN annotation/suppression must
  identify only the known detached roots; never suppress the whole runner."*

## The load-bearing design fact

The `i_shut_*` and `i_leak_bound` scenarios **intentionally leak** suspended/detached
QCoro coroutine frames at shutdown — accepted by the spec
(`docs/design/network-redesign.md`, "Shutdown and task ownership", S1-1/S1-2: at
shutdown a suspended frame is *detached, not destroyed*, and leaks a few hundred
bytes). The non-shutdown scenarios (`i_cancel_*`, `i_retention`,
`fullChainStashListSucceeds`) run to completion and must be leak-CLEAN.

So the LSan gate uses a suppression naming ONLY the known detached-frame roots. Any
leak OUTSIDE that closure (a `RateLimitManager`, the `Gate`, a `QNetworkReply`, an
unrelated allocation) has a different stack, is NOT suppressed, and FAILS the job.
That failing behavior IS the I-LEAK-BOUND enforcement. The known roots are the
detached coroutine frames and their in-frame awaiter closures:
- worker per-fetch coroutines: `ItemsManagerWorker::FetchStashList`,
  `FetchCharacterList`, `FetchStash`, `FetchCharacter` (`src/itemsmanagerworker.cpp`).
- hub drain/setup coroutines: `RateLimitManager::Drain` / `ProcessEntry`, and
  `RateLimiter::ProbeEndpoint` (`src/ratelimit/ratelimitmanager.cpp`,
  `src/ratelimit/ratelimiter.cpp`).
- their in-frame awaiters: QCoro's `QFutureWatcher`, gate/sleep `QTimer`s.

## CI conventions to follow (copy from `.github/workflows/codeql.yml`)

- `runs-on: ubuntu-latest`; `actions/checkout@v6` with `submodules: recursive`.
- Setup Qt: `jurplel/install-qt-action@v4`, `version: '6.11.1'`,
  `modules: 'qtnetworkauth'`, `cache: true`. It exports Qt into the env so CMake
  finds it — no manual `CMAKE_PREFIX_PATH` needed in CI.
- apt deps: `openssl libcurl4-openssl-dev libgl-dev libssl-dev libvulkan-dev
  libxcb-cursor0 libxcb-cursor-dev zlib1g-dev`.
- Triggers: `pull_request` to `master` + `workflow_dispatch` (matching codeql; you
  may also add `push` to `master`).

## Step 1 — the workflow `.github/workflows/sanitizers.yml`

- Configure a throwaway tree:
  `cmake -B build-asan -S . -DACQ_SANITIZE=address -DCMAKE_BUILD_TYPE=RelWithDebInfo`.
  On Linux, `-fsanitize=address` includes LeakSanitizer (`detect_leaks=1` at exit).
- Build ONLY the test target:
  `cmake --build build-asan --target tst_workerintegration --parallel`.
  Do NOT build the `acquisition` app target (it links crashpad). `acquisition_core`
  plus sentry/crashpad DO compile under ASan (confirmed previously), but
  crashpad-under-ASan is a heavy from-scratch build — allow generous build time.
- Ensure symbolized leak stacks: install/confirm `llvm-symbolizer` (clang) or
  `addr2line` (gcc). Consider pinning clang for cleaner ASan symbolization, or keep
  ubuntu's default gcc (≥13 satisfies the QCoro floor). If you pin clang, set
  `CC`/`CXX` before configure.
- Run the scenarios with:
  ```
  ASAN_OPTIONS=abort_on_error=1:detect_leaks=1
  LSAN_OPTIONS=suppressions=${{ github.workspace }}/tests/lsan.supp:print_suppressions=1
  ctest --test-dir build-asan -R tst_workerintegration --output-on-failure
  ```
  Each scenario is its own process, so LSan runs at each process exit; the shutdown
  scenarios' detached-frame leaks are suppressed, everything else must be clean.

## Step 2 — seed `tests/lsan.supp`

You cannot run LSan on macOS, so you cannot author the exact suppression now — seed
it with `leak:` entries for the known detached-frame roots (Session B tunes it
against real CI stacks). Format is one `leak:<substring matched against a frame in
the leak stack>` per line, e.g.:
```
# CI-tuned LeakSanitizer suppressions for tst_workerintegration.
# Suppress ONLY the accepted detached-QCoro-frame roots at shutdown
# (network-redesign.md "Shutdown and task ownership", S1-1/S1-2). Any leak with a
# stack outside this closure is a real defect and MUST fail the job — never widen
# this file to silence an unexplained leak. Session B refines these against the
# first CI run's actual stacks.
leak:ItemsManagerWorker::FetchStashList
leak:ItemsManagerWorker::FetchCharacterList
leak:ItemsManagerWorker::FetchStash
leak:ItemsManagerWorker::FetchCharacter
leak:RateLimitManager::Drain
leak:RateLimitManager::ProcessEntry
leak:RateLimiter::ProbeEndpoint
```
Prebuilt-Qt statics (font/plugin/`QCoreApplication` one-time allocs) may also leak;
if the first CI run shows them, Session B adds a few narrowly-scoped, commented Qt
entries. Keep the file TIGHT — never a blanket suppression.

## Step 3 — local doc wiring (no CI needed)

- `BUILD.md` "Sanitizer Builds" section: replace the "separate follow-up" wording
  with a pointer to `.github/workflows/sanitizers.yml` and `tests/lsan.supp`,
  describing that the job enforces leak-boundedness with a tight suppression.
  State plainly that the suppression is CI-tuned.
- Do NOT yet flip any "5E complete / leak-boundedness enforced" claims in
  `docs/design/network-redesign-phase5-plan.md` or the `docs/cleanup/findings.md`
  F56 ledger entry — those wait until the CI job is actually green (Session B).

## Guardrails

- Test/CI-only. No production behavior changes. Do not sanitize or run the app target.
- Keep the suppression tight (only known detached roots + unavoidable Qt statics).
- You cannot validate LSan locally on macOS. Do the local build once with
  `-DACQ_SANITIZE=address` (ASan-only value it locally proves it configures/compiles;
  leak checking simply won't fire on macOS) if you want a compile sanity check, but
  do not claim leak enforcement works until CI is green.
- Follow the repo's commit conventions (see recent `git log`), including its
  `Co-Authored-By:` trailer; adapt the trailer to whatever model/agent you are.

## End-of-session report

- Is `sanitizers.yml` committed and the first CI run triggered (link/id)?
- Is `tests/lsan.supp` seeded and `BUILD.md` updated?
- Explicitly: suppression tuning, the out-of-closure-leak mutation check, and the
  "5E complete" status reconciliation are NOT done here — they are Session B, which
  needs the first CI run's leak output. Hand off cleanly.
