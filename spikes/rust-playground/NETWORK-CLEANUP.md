# Network cleanup control

This file is the execution ledger for the Rust playground's network cleanup.
It records package boundaries and handoffs; it does not replace the frozen
design or the ground-truth claims.

## Authority

When sources disagree, use this order:

1. `AGENTS.md` and `CONTEXT.md` for repository and spike invariants.
2. `../../docs/design/network-redesign.md` for the frozen target design.
3. `../../docs/design/network-ground-truth.md` for numbered observations and
   classifications.
4. `../../docs/cleanup/findings.md` for the findings register.
5. This file for execution state only.

Protecting the existing GGG OAuth registration and the invariants in
`CONTEXT.md` is the primary correctness criterion. Do not edit code from a
review-only session.

## Current checkpoint

- Date recorded: 2026-08-21.
- Branch: `spikes/rust-playground`.
- Frozen gate-contract baseline: `1e17e812`.
- Current implementation tip: `7f76e56d`.
- Active package: N1, in review.
- Exact N1 review range: `1e17e812..7f76e56d`.
- Do not start H0 or N2 until the N1 verdict has been reconciled here.

## Package ledger

Statuses move through `planned`, `building`, `built`, `reviewing`,
`changes-requested`, and `accepted`. Only an accepted package may unblock its
dependents.

| ID | Package | Depends on | Status | Commit or review range |
| --- | --- | --- | --- | --- |
| N0 | Ground truth and OAuth gate decision | — | accepted | through `1e17e812` |
| N1 | Strict parsing, observation/classification, and 429 recovery | N0 | reviewing | `1e17e812..7f76e56d` |
| H0 | Workspace formatting and strict-Clippy baseline | N1 | planned | — |
| N2 | OAuth refresh singleflight and session generations | H0 | planned | — |
| N3 | Send-lifetime gate primitive and fairness semantics | H0 | planned | — |
| N4 | Gate integration in `ChokePoint`; remove `Paid` | N2, N3 | planned | — |
| N5 | Dispatcher cleanup and removal of job-task head-of-line blocking | N4 | planned | — |
| N6 | Integration stress tests and final frozen-design reconciliation | N5 | planned | — |

N2 and N3 may be built in either order after H0. Keep them as separate commits
and review them separately. N4 is the first package allowed to depend on both.

## Package definitions

### N0 — accepted contract

The accepted target is one common gate inside the choke point, held for the
send lifetime of every daemon-owned GGG API GET, HEAD, OAuth code exchange,
and OAuth refresh. Browser authorization is outside the gate. Token traffic
uses the synthetic `oauth-token` policy before discovery and
`token-request-limit` afterward. Authentication completes before an API
request takes its final limiter check and gate permit. Until N33's hidden
resolution is confirmed, its one-window `60:30:30` rule uses conservative
60-second bucket padding.

### N1 — strict observation and recovery

Implementation commits:

- `f650e83a` — strict total rate-limit parsing and observation hardening.
- `7f76e56d` — response classification and bounded 429 recovery.

Scope:

- A partial or malformed steady-state observation cannot replace a valid
  policy.
- Rule, state, and policy headers are parsed as a coherent total observation.
- Probe and steady-state classifications follow the frozen design.
- `Retry-After` is parsed independently and conservatively bounded.
- 429 recovery installs route-local holds, escalates HEAD failures to full
  exclusion, and retries only within the explicit bound.
- Response bodies are consumed before retry classification where required.

Non-goals: OAuth refresh coordination, send-lifetime gate ownership,
dispatcher semantics, and final integration stress behavior.

Review verdict: pending. Record findings as `N1-R1`, `N1-R2`, and so on in the
review register below. If the verdict is `changes-requested`, fix only those
findings, append fix commits, and re-review the same package before proceeding.

### H0 — mechanical quality baseline

This is a deliberately separate, non-semantic package:

- Run workspace formatting and commit only the mechanical result.
- Require workspace tests, format check, and strict workspace Clippy to pass.

Do not mix H0 with N1 review fixes or N2 behavior. Its purpose is to make every
later handoff's quality-gate result meaningful.

### N2 — OAuth refresh ownership

Establish one refresh owner for concurrent callers, make waiters share its
result, and model session/access-token/refresh-token generations so a stale
completion cannot overwrite logout, re-authentication, or a rotated refresh
token. Preserve the existing registration, scopes, callback, user-agent,
keyring isolation, and no-plaintext-token rules.

Required characterization includes concurrent expiry, successful refresh-token
rotation, refresh failure, logout during refresh, and re-authentication during
refresh. N2 coordinates auth state but does not yet claim the global HTTP gate.

### N3 — gate primitive

Implement and test the frozen gate semantics independently of HTTP call sites:

- One global burst bound applies to actual sends, not waiting jobs.
- Per-policy serialization lasts through response/body completion.
- HEAD is globally exclusive as specified.
- Waiting writers receive preference over newly arriving readers.
- A permit is a live reservation with an explicit lifetime, not evidence that
  an earlier limiter check succeeded.

Use deterministic concurrency tests for permit lifetime, cancellation, writer
preference, HEAD exclusivity, and independent-policy progress.

### N4 — choke-point integration

Route every daemon-owned GGG request, including OAuth exchange and refresh,
through the common gate. Acquire authentication before the API request's final
limiter check/permit. Hold the permit for the complete send lifetime, and make
it impossible to call the transport with a stale `Paid`-style receipt. Token
requests use the N33 policy mapping recorded in N0.

### N5 — dispatcher cleanup

Stop treating jobs waiting on auth or rate limits as HTTP in-flight work.
Remove the resulting cross-policy head-of-line blocking while preserving job
priority, cancellation, route-probe behavior, and the actual-send bounds now
owned by N4's gate.

### N6 — integration and reconciliation

Stress mixed API policies, HEAD probes, token refresh, cancellation, 429s, and
rotated tokens. Then compare the implementation line by line with the frozen
design's strict parsing, classifications, permit lifetime, HEAD exclusivity,
and writer-preference rules. Record any intentional divergence in the
authoritative design or findings register, not only here.

## Review register

| Finding | Severity | Package | Summary | Status | Fix commit |
| --- | --- | --- | --- | --- | --- |
| — | — | N1 | Review in progress | pending | — |

For every finding, retain the concrete scenario, exact file and line references,
affected invariant/design rule/ground-truth claim, classification (confirmed
bug, architectural risk, or test gap), smallest fix direction, and required
tests. Preserve sound behavior explicitly in the review handoff.

## Session protocol

Each session owns exactly one role and package:

- A build session starts from an accepted dependency tip, implements only the
  package scope, runs its checks, commits, and records exact hashes.
- A review session is read-only, reviews an exact base-to-tip range, and returns
  `accepted` or `changes-requested` with stable finding IDs.
- A fix session addresses only recorded findings, commits them separately, and
  returns the package to review.
- A coordination session reconciles results into this ledger and chooses the
  next unblocked package. It does not make semantic code changes.

Every handoff must state:

- package ID and role;
- starting commit, ending commit, and whether the worktree is clean;
- scope completed and explicit non-goals;
- tests/checks run with exact outcomes;
- unresolved findings and the single next action.

Never describe a moving branch name as the review boundary when hashes are
available. Do not stack new semantic work on a package whose review is pending.

## Quality-gate baseline

Recorded at implementation tip `7f76e56d` on 2026-08-21:

- `cargo test --workspace --all-targets`: passed, 30 core tests and 0 CLI
  tests.
- `cargo fmt --all -- --check`: failed on pre-existing workspace-wide
  formatting drift.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.

The core crate's strict Clippy check remains the interim semantic-package gate.
After H0, all three workspace gates must be green for every package.

## Verdict routing

When the N1 review finishes:

1. Start a fresh coordination session at the current branch tip.
2. Ask it to read this file and reconcile the complete review report.
3. If accepted, mark N1 accepted and start H0.
4. If changes are requested, add stable N1 finding IDs, run an N1-only fix
   session, and re-review its expanded exact range.
5. Do not start N2 or N3 until N1 and H0 are accepted.
