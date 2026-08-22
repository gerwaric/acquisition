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
- Current implementation tip: `412c840e155b01f626560fcb097393d6a24b797c`.
- Active package: none; N1 is accepted and H0 is the next unblocked package.
- Accepted N1 review range:
  `1e17e812..412c840e155b01f626560fcb097393d6a24b797c`.
- N1 fix commit: `412c840e155b01f626560fcb097393d6a24b797c`
  (`N1-R1` through `N1-R3`).
- N1 review verdict: `accepted`; no unresolved N1 work remains.
- Do not start N2 or N3 until H0 is accepted.

## Package ledger

Statuses move through `planned`, `building`, `built`, `reviewing`,
`changes-requested`, and `accepted`. Only an accepted package may unblock its
dependents.

| ID | Package | Depends on | Status | Commit or review range |
| --- | --- | --- | --- | --- |
| N0 | Ground truth and OAuth gate decision | — | accepted | through `1e17e812` |
| N1 | Strict parsing, observation/classification, and 429 recovery | N0 | accepted | `1e17e812..412c840e155b01f626560fcb097393d6a24b797c` |
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
- `412c840e155b01f626560fcb097393d6a24b797c` — resolves review findings
  `N1-R1` through `N1-R3`.

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

Review verdict: `accepted` for exact implementation range
`1e17e812..412c840e155b01f626560fcb097393d6a24b797c`. The accepted review
confirmed that fix commit `412c840e155b01f626560fcb097393d6a24b797c`
resolves `N1-R1` through `N1-R3`; it found no new findings and no unresolved
N1 work remains.

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
| N1-R1 | High | N1 | Strict parsing accepts values that can overflow deadline arithmetic and panic | resolved | `412c840e155b01f626560fcb097393d6a24b797c` |
| N1-R2 | Medium | N1 | Clean-2xx classification and send recording occur before body transfer completion | resolved | `412c840e155b01f626560fcb097393d6a24b797c` |
| N1-R3 | Low | N1 | Bounded retry/probe behavior lacks coverage through the real dispatcher lifecycle | resolved | `412c840e155b01f626560fcb097393d6a24b797c` |

For every finding, retain the concrete scenario, exact file and line references,
affected invariant/design rule/ground-truth claim, classification (confirmed
bug, architectural risk, or test gap), smallest fix direction, and required
tests. Preserve sound behavior explicitly in the review handoff.

### N1-R1 — numeric values can panic deadline arithmetic

- **Concrete scenario:** a syntactically numeric policy period or active
  restriction such as `18446744073709551615` parses as `u64`. Once that
  window is saturated or restricted, `next_safe_send` evaluates
  `Instant + Duration` and panics on overflow. This violates the total-parser
  containment guarantee.
- **Implementation references at `7f76e56d`:** integer parsing in
  `crates/acquisition-core/src/ratelimit.rs:240`, window parsing at line 337,
  and deadline arithmetic at line 938.
- **Authority:** frozen design D8's full-header numeric bounds and never-crash
  requirement; ground-truth N9; N20's degraded-input containment.
- **Classification:** confirmed bug. The reviewer reproduced the panic with
  `Instant::now() + Duration::from_secs(u64::MAX)`.
- **Smallest fix direction:** define named operational bounds for policy
  period and restriction fields, reject values above them during parsing, and
  use checked deadline addition so accepted or retained state cannot panic.
- **Required tests:** maximum accepted and one-above-bound values for limit
  period, limit restriction, state period, and active restriction. Rejected
  values must return `PolicyParseError`, and the paths through `observe` and
  `wait_for` must not panic.

### N1-R2 — response completion is split across two owners

- **Concrete scenarios:** a malformed-header 200 whose body is truncated is
  classified `Protocol` before the later transfer failure can take D8
  precedence. A Full-header 200 whose body is truncated is recorded as a
  successful `200 OK`; the later body failure becomes `ApiError::Other`
  instead of `Network`. OAuth token responses have the same split boundary.
- **Implementation references at `7f76e56d`:** header-time classification in
  `crates/acquisition-core/src/ratelimit.rs:1327`, premature send recording at
  line 1363, later body consumption in
  `crates/acquisition-core/src/daemon.rs:1041`, body-error collapse at
  `daemon.rs:1422`, and the token path in
  `crates/acquisition-core/src/auth.rs:91`.
- **Authority:** frozen design D3 requires bookkeeping and capture of the
  complete exchange; D8 gives a 2xx-plus-transfer-error `Network` precedence.
- **Classification:** confirmed bug.
- **Smallest fix direction:** defer clean-2xx `Protocol` classification and
  final send recording until body transfer resolves. Carry header observation
  with the response or consume body bytes inside the choke-point response
  package. Policy observation must still occur for every landed response.
- **Required real-local-stream tests:** Full 200 plus truncated body produces
  `Network` while updating policy; malformed-header 200 plus truncated body
  produces `Network`, not `Protocol`, and leaves policy unchanged; a complete
  malformed-header 200 produces `Protocol`; truncated 429 and 500 responses
  retain status precedence and record body-transfer evidence.

### N1-R3 — recovery is not pinned through dispatcher requeue

- **Concrete scenario:** three consecutive 429s arrive while a younger
  same-policy job waits. Existing helper and limiter tests would not catch a
  regression that sends a fourth attempt, sleeps after exhaustion, or moves
  the requeued job behind the younger job.
- **Implementation references at `7f76e56d`:** the actual transition in
  `crates/acquisition-core/src/daemon.rs:548`, helper-only bound coverage at
  line 1618, direct limiter 429 coverage in
  `crates/acquisition-core/src/ratelimit.rs:2247`, and direct probe coverage at
  line 2380.
- **Authority:** ground-truth P-A, N10, and N27; frozen design D3; bounded
  attempts, FIFO preservation, and immediate exhausted completion.
- **Classification:** test gap. Source inspection found the current
  transitions sound.
- **Smallest fix direction:** add an injected-clock/fake-server daemon test
  around the actual `ChokePoint -> api_get -> Exec::RateLimited -> dispatcher
  requeue` lifecycle. This is coverage of N1 behavior, not authorization to
  redesign dispatcher semantics.
- **Required tests:** `429, 429, success` sends exactly three times and
  completes once; three 429s fail immediately after the third response with
  no fourth send or exhausted-attempt sleep; the requeued job remains before a
  younger equal-priority job; 403 and 503 send once and never retry; a Full
  acceptable HEAD 429 establishes under hold with job retry count zero, while
  a malformed or unacceptable HEAD 429 fails under cooldown.

### N1 behavior the fix must preserve

- Full parsing requires nonempty policy and rule names, exact triplets,
  positive limit hits and periods, nonnegative state hits, equal limit/state
  lengths, and matching periods.
- `Retry-After` remains independent of Full-policy parsing: 0 and 900 are
  accepted; missing, negative, nonnumeric, overflow, and values above 900 are
  terminal.
- Landed 429s increment violations and counted history before retry
  classification. Malformed or mismatched observations do not replace an
  established policy or topology.
- Same-shape dynamic definitions retain history; ordered rule/period shape
  changes clear it.
- Retry holds use the maximum route/policy deadline and include the
  unconditional 60-second pad plus buffer.
- Requeueing retains job identity and therefore priority/FIFO position;
  exhaustion returns without sleeping. 403 and 503 never retry.
- Only a Full acceptable HEAD 429 establishes; other partial or unacceptable
  probes fail under cooldown.

### N1 accepted-review validation

Independent review of exact range
`1e17e812..412c840e155b01f626560fcb097393d6a24b797c` recorded:

- Verdict: `accepted`; fix commit
  `412c840e155b01f626560fcb097393d6a24b797c` resolves `N1-R1` through
  `N1-R3`, with no new findings and no unresolved N1 work.
- `cargo test --workspace --all-targets`: passed, 44 core tests and 0 CLI
  tests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.
- `cargo fmt --all -- --check`: failed only on the known H0 baseline files:
  `client.rs`, `dash.rs`, `main.rs`, `auth.rs`, `job.rs`, and `mockggg.rs`;
  there was no new N1 formatting drift.

The format failure is not an N1 finding. N1 is accepted, so H0 is unblocked;
N2 and N3 remain blocked until H0 is accepted.

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

Recorded at accepted N1 tip `412c840e155b01f626560fcb097393d6a24b797c`
on 2026-08-21:

- `cargo test --workspace --all-targets`: passed, 44 core tests and 0 CLI
  tests.
- `cargo fmt --all -- --check`: failed only on the known H0 baseline files
  `client.rs`, `dash.rs`, `main.rs`, `auth.rs`, `job.rs`, and `mockggg.rs`;
  no new N1 formatting drift was found.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.

The core crate's strict Clippy check remains the interim semantic-package gate.
After H0, all three workspace gates must be green for every package.

## Next action and exact kickoff prompt

The single next action is an H0 mechanical-formatting session. Use this prompt
verbatim:

```text
Read AGENTS.md, CONTEXT.md, README.md, NETWORK-CLEANUP.md, and the frozen
network design documents referenced there before editing. This is an H0
mechanical-formatting-only session. Start from the current clean
spikes/rust-playground branch tip, which must contain the coordination commit
recording N1 as accepted for exact range
1e17e812..412c840e155b01f626560fcb097393d6a24b797c. Record the exact starting
hash before editing; if the worktree is not clean or that ledger state is
absent, stop and report the mismatch.

Run cargo fmt --all and commit only its mechanical formatting result. The
known baseline files are client.rs, dash.rs, main.rs, auth.rs, job.rs, and
mockggg.rs. Review the formatter diff to confirm it contains no semantic
changes and no files outside the formatter's mechanical result.

Do not change behavior, begin N2 or N3, implement OAuth
singleflight/session-generation work, implement the send-lifetime gate, alter
dispatcher semantics, edit the frozen design documents, or contact GGG.

Run cargo test --workspace --all-targets; cargo clippy --workspace
--all-targets --all-features -- -D warnings; cargo clippy -p acquisition-core
--all-targets --all-features -- -D warnings; and cargo fmt --all -- --check.
Record exact outcomes. Commit only the formatting change with an H0-labeled
mechanical-formatting commit message. Do not mark H0 accepted. Return the exact
starting and ending hashes, clean worktree state, formatted files, confirmation
that the diff was mechanical only, checks, unresolved findings, and the single
next action: an independent H0 review of the exact coordination-tip-to-H0-tip
range.
```

Only an independent H0 review with no required work remaining may mark H0
accepted. N2 and N3 remain blocked until H0 is accepted.
