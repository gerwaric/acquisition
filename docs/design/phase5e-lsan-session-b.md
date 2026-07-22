# Phase-5E LSan — Session B (CI-bound, self-contained)

Paste this whole file as the task for a fresh session. It assumes no prior
conversation and no persistent memory. Re-derive all state by reading the repo and
the GitHub Actions logs — do not trust this file's description of "current state"
over what you actually find.

## Goal

Finish enforcing **I-LEAK-BOUND** leak-boundedness for the phase-5 network-redesign
integration tests (the last open item of phase 5E). Session A already added a Linux
CI job that runs the tests under AddressSanitizer + LeakSanitizer with a *seeded*
suppression. This session: tune the suppression against real CI leak output until
the job is green, prove the gate has teeth (an out-of-closure leak must turn it
red), then reconcile the "5E complete" status marks.

Work on branch `ratelimit-phase5-worker`.

## Re-derive the state first (do not assume)

Read, in this order:
1. `.github/workflows/sanitizers.yml` — the CI job Session A wrote (build with
   `-DACQ_SANITIZE=address`, run `ctest -R tst_workerintegration` under
   `ASAN_OPTIONS`/`LSAN_OPTIONS=suppressions=tests/lsan.supp`).
2. `tests/lsan.supp` — the seeded suppression.
3. `tests/tst_workerintegration.cpp` — the runner (real worker → facade → hub;
   `i_shut_*` and `i_leak_bound` intentionally leak detached QCoro frames at
   shutdown; the other scenarios must be leak-clean).
4. `docs/design/network-redesign-phase5-verification.md` §6 — the contract. Key
   rule: *"Any LSAN annotation/suppression must identify only the known detached
   roots; never suppress the whole runner."*
5. The latest CI run for this branch:
   `gh run list --branch ratelimit-phase5-worker --workflow sanitizers.yml`, then
   `gh run view <id> --log` (or `--log-failed`). This is your leak-stack source —
   LeakSanitizer does NOT run on macOS, so CI logs are the only place you see them.

## Why this is CI-bound

LeakSanitizer cannot run on macOS (arm64 Darwin). Every tuning iteration is
push → wait for GitHub Actions → read logs → refine → repeat, and each run is a
heavy from-scratch ASan build of `acquisition_core` + crashpad (minutes). Expect a
few iterations. Do NOT try to shortcut this locally; you can't.

## Step 1 — tune `tests/lsan.supp` to green

- From the failing run's log, collect the leak stacks. The ACCEPTED leaks are the
  detached QCoro frames at shutdown and their in-frame awaiters: the worker
  coroutines `FetchStashList`/`FetchCharacterList`/`FetchStash`/`FetchCharacter`,
  the hub coroutines `RateLimitManager::Drain`/`ProcessEntry` and
  `RateLimiter::ProbeEndpoint`, and their `QFutureWatcher`/`QTimer` awaiters.
  Prebuilt-Qt statics (font/plugin/`QCoreApplication` globals) may also leak.
- Refine the suppression to cover EXACTLY those stacks and nothing more. Each entry
  should match a frame that is provably part of a detached-frame closure or an
  unavoidable Qt static. Comment each Qt entry with why it is unavoidable. Keep it
  TIGHT — if a leak stack is not clearly a known detached root, it is a real defect;
  investigate it, do not silence it.
- Watch symbolization: if the stacks are unsymbolized (raw addresses), fix the CI
  tooling first (install `llvm-symbolizer`/`addr2line`, ensure `-g`, don't strip)
  before writing suppressions against garbage frames.
- Iterate via CI until `tst_workerintegration.*` is green: shutdown scenarios pass
  with only their known frames suppressed, non-shutdown scenarios leak nothing.

## Step 2 — prove the gate has teeth (mutation)

The suppression must not be so broad that a real leak slips through. Verify:
- Temporarily introduce a leak of an object OUTSIDE the detached closure — e.g. in a
  shutdown scenario leak an owner that should be freed (a stray `new RateLimitManager`,
  or skip freeing a reply/owner), or in a non-shutdown scenario leak any allocation.
- Push and confirm the CI job goes RED because that stack is not in the suppression.
- Revert the mutation; confirm green again. Record the failing stack in the commit or
  session notes as the evidence that the gate catches out-of-closure leaks.
- If you genuinely cannot trigger CI runs, STOP and leave precise instructions for a
  human to run this check — do not claim it passed.

## Step 3 — reconcile the status marks (only once CI is green)

- `docs/design/network-redesign-phase5-plan.md`: the 5E section's "Implemented" note
  calls the LSan job a "separate follow-up" — mark it done; 5E status is now genuinely
  Complete. (This plan doc is deliberately temporary and is deleted at PR merge.)
- `docs/cleanup/findings.md`: the F56 resolved-ledger entry references the
  `tst_workerintegration` runner — add the LSan CI job (`sanitizers.yml`) to its
  evidence.
- `BUILD.md`: confirm the "Sanitizer Builds" section points at the workflow and the
  tuned suppression (Session A wired most of this; finish/correct it).
- If this environment has a persistent project-memory system, update it to note the
  LSan job landed and 5E is fully complete. If it does not, skip — the repo docs above
  are the durable record.

## Guardrails

- Test/CI-only. No production behavior changes (the mutation in Step 2 must be
  reverted; the committed tree has no deliberate leak). Do not sanitize or run the app
  target.
- Keep the suppression tight — a broad suppression re-creates the false confidence this
  whole exercise exists to remove.
- Do not claim "complete" until the CI job is actually green and the Step 2 mutation
  has shown the gate turns red on an out-of-closure leak.
- Follow the repo's commit conventions (see recent `git log`), including its
  `Co-Authored-By:` trailer; adapt it to whatever model/agent you are.

## End-of-session report

- Is `tst_workerintegration.*` green under ASan+LSan in CI (run link/id)?
- Is `tests/lsan.supp` tight and first-run-tuned (list what each entry covers)?
- Did the Step 2 out-of-closure-leak mutation turn the job red (paste the stack), and
  is it reverted?
- Which status marks were reconciled? Is anything still pending human CI validation?
