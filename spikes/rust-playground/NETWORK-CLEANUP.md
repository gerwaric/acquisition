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
- Current implementation tip: `510ea498a7f4fc9d75d04893eba6243768577fef`.
- Active package: none; N2 and N3 are accepted, and N4 is the next unblocked
  package.
- Accepted N1 review range:
  `1e17e812..412c840e155b01f626560fcb097393d6a24b797c`.
- N1 fix commit: `412c840e155b01f626560fcb097393d6a24b797c`
  (`N1-R1` through `N1-R3`).
- N1 review verdict: `accepted`; no unresolved N1 work remains.
- Accepted H0 review range:
  `694fb10c8a59ed54147ce3431a3962683ee5e4e6..f1fcb24e3a03b7d9e5d4faa9bbcee3cf244c61a7`.
- H0 formatting commit: `f1fcb24e3a03b7d9e5d4faa9bbcee3cf244c61a7`.
- H0 review verdict: `accepted`; the independent review found no findings and
  no unresolved H0 work remains.
- Accepted N2 implementation range:
  `a74341263676b4bdb5ebade23ef862ea0a0e4127..0a47efecdb78de1202e29c8fe7faaa4d39e66372`.
- N2 implementation commit:
  `c89ea6780cb8b3d438085ec959a4daf1f22fa7f2`.
- N2-R1 fix commit:
  `0a47efecdb78de1202e29c8fe7faaa4d39e66372`.
- N2 review verdict: initial review of
  `a74341263676b4bdb5ebade23ef862ea0a0e4127..c89ea6780cb8b3d438085ec959a4daf1f22fa7f2`
  returned `changes-requested` with `N2-R1`; fix-only re-review of
  `c89ea6780cb8b3d438085ec959a4daf1f22fa7f2..0a47efecdb78de1202e29c8fe7faaa4d39e66372`
  returned `accepted` with no findings.
- `N2-R1` is resolved. No unresolved N2 findings and no required N2 work
  remain.
- Accepted N3 implementation range:
  `7f205d846ecab73c119532416f1a132010562b4c..510ea498a7f4fc9d75d04893eba6243768577fef`.
- N3 implementation commit:
  `510ea498a7f4fc9d75d04893eba6243768577fef`.
- N3 review verdict: `accepted`; the independent review found no findings and
  no unresolved N3 work remains.
- N4 is unblocked. Start N4 next and keep it strictly separate from N5.

## Package ledger

Statuses move through `planned`, `building`, `built`, `reviewing`,
`changes-requested`, and `accepted`. Only an accepted package may unblock its
dependents.

| ID | Package | Depends on | Status | Commit or review range |
| --- | --- | --- | --- | --- |
| N0 | Ground truth and OAuth gate decision | — | accepted | through `1e17e812` |
| N1 | Strict parsing, observation/classification, and 429 recovery | N0 | accepted | `1e17e812..412c840e155b01f626560fcb097393d6a24b797c` |
| H0 | Workspace formatting and strict-Clippy baseline | N1 | accepted | `694fb10c8a59ed54147ce3431a3962683ee5e4e6..f1fcb24e3a03b7d9e5d4faa9bbcee3cf244c61a7` |
| N2 | OAuth refresh singleflight and session generations | H0 | accepted | `a74341263676b4bdb5ebade23ef862ea0a0e4127..0a47efecdb78de1202e29c8fe7faaa4d39e66372` |
| N3 | Send-lifetime gate primitive and fairness semantics | H0 | accepted | `7f205d846ecab73c119532416f1a132010562b4c..510ea498a7f4fc9d75d04893eba6243768577fef` |
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

Review verdict: `accepted` for exact range
`694fb10c8a59ed54147ce3431a3962683ee5e4e6..f1fcb24e3a03b7d9e5d4faa9bbcee3cf244c61a7`.
The independent review found no findings. The diff changed only `client.rs`,
`dash.rs`, `main.rs`, `auth.rs`, `job.rs`, and `mockggg.rs`, and was verified
as the exact mechanical result of `cargo fmt --all`.

Accepted-review validation:

- `cargo test --workspace --all-targets`: passed, 44 core tests and 0 CLI
  tests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.
- `cargo fmt --all -- --check`: passed.

### N2 — OAuth refresh ownership

Establish one refresh owner for concurrent callers, make waiters share its
result, and model session/access-token/refresh-token generations so a stale
completion cannot overwrite logout, re-authentication, or a rotated refresh
token. Preserve the existing registration, scopes, callback, user-agent,
keyring isolation, and no-plaintext-token rules.

Required characterization includes concurrent expiry, successful refresh-token
rotation, refresh failure, logout during refresh, and re-authentication during
refresh. N2 coordinates auth state but does not yet claim the global HTTP gate.

Implementation commits:

- `c89ea6780cb8b3d438085ec959a4daf1f22fa7f2` — establishes refresh
  singleflight ownership, shared waiter results, and session/access-token/
  refresh-token generation rejection.
- `0a47efecdb78de1202e29c8fe7faaa4d39e66372` — resolves `N2-R1` by making
  owner abandonment complete waiters and release only the matching flight.

Review verdict: `accepted` for the combined exact implementation range
`a74341263676b4bdb5ebade23ef862ea0a0e4127..0a47efecdb78de1202e29c8fe7faaa4d39e66372`.
The initial independent review of exact range
`a74341263676b4bdb5ebade23ef862ea0a0e4127..c89ea6780cb8b3d438085ec959a4daf1f22fa7f2`
returned `changes-requested` with the single High finding `N2-R1`. The
fix-only independent re-review of exact range
`c89ea6780cb8b3d438085ec959a4daf1f22fa7f2..0a47efecdb78de1202e29c8fe7faaa4d39e66372`
returned `accepted` with no findings. It confirmed that owner cancellation,
task abortion, future dropping, and other owner abandonment remove only the
matching flight by flight ID and captured session/access-token/refresh-token
generations, publish one stable abandonment result to every waiter, leave an
unchanged session immediately retryable, and cannot clear or overwrite a newer
flight, logout, re-authentication, rotated-token state, or an already-published
normal result.

The accepted review also confirmed preservation of concurrent expiry,
successful refresh-token rotation, ordinary refresh failure and retry, logout
during refresh, re-authentication during refresh, overlapping login
generations, and stale in-memory/keyring-write rejection. Registration,
scopes, callback, PKCE, user-agent, provider keyring isolation, and the
no-plaintext-token rules remain unchanged. `N2-R1` is resolved; no unresolved
N2 findings and no required N2 work remain.

Accepted-review validation at
`0a47efecdb78de1202e29c8fe7faaa4d39e66372`:

- `cargo test --workspace --all-targets`: passed, 51 core tests and 0 CLI
  tests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.
- `cargo fmt --all -- --check`: passed.

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

Implementation commit:

- `510ea498a7f4fc9d75d04893eba6243768577fef` — adds the independent
  send-lifetime gate primitive and exports it from `acquisition-core`.

Review verdict: `accepted` for exact implementation range
`7f205d846ecab73c119532416f1a132010562b4c..510ea498a7f4fc9d75d04893eba6243768577fef`.
The independent review found no findings and no unresolved N3 work. It
confirmed the frozen global cap, per-policy serialization, live-permit
lifetime, HEAD exclusivity and writer preference, eligible ordinary FIFO,
independent-policy progress, and cancellation safety. The implementation
changes only `crates/acquisition-core/src/gate.rs` and
`crates/acquisition-core/src/lib.rs`; its deterministic tests use no
wall-clock sleeps or HTTP calls. OAuth refresh ownership, `ChokePoint`
integration, `Paid`, dispatcher semantics, N4 behavior, and the frozen design
documents remain unchanged.

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
| N2-R1 | High | N2 | Abandoned refresh owner permanently strands waiters | resolved | `0a47efecdb78de1202e29c8fe7faaa4d39e66372` |

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

The format failure was not an N1 finding. At that review checkpoint N1 was
accepted, H0 was unblocked, and N2/N3 remained blocked pending H0 acceptance.

### N2-R1 — abandoned refresh owner permanently strands waiters

- **Concrete scenario:** one caller owns a refresh flight and reaches the
  token endpoint while concurrent callers subscribe as waiters. If the owner
  task is aborted, its future is dropped, or it is otherwise abandoned before
  normal result publication, the original implementation drops the only code
  path that clears the flight and publishes to the watch channel. The waiters
  remain pending forever and the unchanged session keeps joining the stranded
  flight, so it cannot retry.
- **Implementation references at
  `c89ea6780cb8b3d438085ec959a4daf1f22fa7f2`:** refresh-flight subscription
  and creation in `crates/acquisition-core/src/daemon.rs:1288`; the unguarded
  owner await and sole normal call to `finish_refresh` at line 1317; flight
  removal and result publication only inside `finish_refresh` at lines
  1327–1356.
- **Fix references at
  `0a47efecdb78de1202e29c8fe7faaa4d39e66372`:** the
  `RefreshOwnerGuard` and its cancellation-safe `Drop` publication at
  `crates/acquisition-core/src/daemon.rs:218–255`; guard installation and
  normal disarming at lines 1358–1369; deterministic abandonment coverage at
  lines 2065–2215.
- **Authority:** N2's shared-waiter-result and stale-generation contract;
  frozen design D1/D2's live-waiter completion requirement; ground-truth N33's
  rotating refresh-token behavior and the repository's no-stale-token and
  no-plaintext-token invariants.
- **Classification:** confirmed bug.
- **Smallest fix direction:** install an owner-lifetime guard before the
  refresh await. On abandonment, under the shared-state lock, clear only the
  flight whose ID and captured session/access-token/refresh-token generations
  still match, then publish one stable abandonment error through that flight's
  existing result channel. Disarm the guard after a normal result is published
  so it cannot overwrite it.
- **Required tests:** prove the owner reached the token endpoint and three
  waiters joined before aborting it; require every waiter to finish within a
  timeout with the identical abandonment error; prove the matching flight was
  removed without changing token state or generations; retry immediately with
  the unchanged refresh token and succeed. Repeat abandonment with no waiters
  and prove the next refresh succeeds. Existing generation/rotation tests must
  continue to prove that abandonment cannot clear or overwrite newer flights,
  logout, re-authentication, rotated-token state, keyring state, or a normally
  published result.

### N2 accepted-review validation

Independent review of exact combined range
`a74341263676b4bdb5ebade23ef862ea0a0e4127..0a47efecdb78de1202e29c8fe7faaa4d39e66372`
recorded:

- Initial verdict: `changes-requested` for exact range
  `a74341263676b4bdb5ebade23ef862ea0a0e4127..c89ea6780cb8b3d438085ec959a4daf1f22fa7f2`,
  with the single finding `N2-R1`.
- Fix-only verdict: `accepted` for exact range
  `c89ea6780cb8b3d438085ec959a4daf1f22fa7f2..0a47efecdb78de1202e29c8fe7faaa4d39e66372`;
  `N2-R1` is resolved, no new findings were reported, and no unresolved or
  required N2 work remains.
- `cargo test --workspace --all-targets`: passed, 51 core tests and 0 CLI
  tests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.
- `cargo fmt --all -- --check`: passed.

### N3 accepted-review validation

Independent review of exact implementation range
`7f205d846ecab73c119532416f1a132010562b4c..510ea498a7f4fc9d75d04893eba6243768577fef`
recorded:

- Package ID and role: N3 independent review.
- Verdict: `accepted`; no findings were reported, no unresolved findings
  remain, and no required N3 work remains.
- The implementation changes only `crates/acquisition-core/src/gate.rs` and
  `crates/acquisition-core/src/lib.rs`.
- The primitive satisfies the frozen global cap, per-policy serialization,
  live-permit lifetime, HEAD exclusivity and writer preference, eligible
  ordinary FIFO, independent-policy progress, and cancellation-safety rules.
- The deterministic tests use no wall-clock sleeps or HTTP calls.
- OAuth refresh ownership, `ChokePoint` integration, `Paid`, dispatcher
  semantics, N4 behavior, and frozen design documents remain unchanged.
- `cargo test --workspace --all-targets`: passed, 57 core tests and 0 CLI
  tests, with 0 failures.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.
- `cargo fmt --all -- --check`: passed.

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

Recorded at accepted N3 tip `510ea498a7f4fc9d75d04893eba6243768577fef`
on 2026-08-21:

- `cargo test --workspace --all-targets`: passed, 57 core tests and 0 CLI
  tests, with 0 failures.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`:
  passed.
- `cargo clippy -p acquisition-core --all-targets --all-features -- -D
  warnings`: passed.
- `cargo fmt --all -- --check`: passed.

All three workspace gates must remain green for every later package; the
core crate's strict Clippy check remains an additional semantic-package gate.

## Next action and exact kickoff prompt

The single next action is an N4 choke-point integration build session. Use this
prompt verbatim:

```text
Read AGENTS.md, CONTEXT.md, README.md, NETWORK-CLEANUP.md, and the frozen
network design documents referenced there before editing. This is an N4
build-only session. Start from the current clean spikes/rust-playground branch
tip, which must contain the coordination commit recording N2 as accepted and
unchanged, and N3 as accepted for exact implementation range
7f205d846ecab73c119532416f1a132010562b4c..510ea498a7f4fc9d75d04893eba6243768577fef,
with no findings or unresolved N3 work. Record the exact starting hash before
editing; if the worktree is not clean or that ledger state is absent, stop and
report the mismatch.

Implement and test only N4: integrate the common gate at `ChokePoint` for
every daemon-owned GGG request, including API GET, HEAD, OAuth code exchange,
and OAuth refresh. Authenticate before an API request's final limiter check
and permit acquisition. Hold each permit through complete response/body
consumption. Use the frozen token-policy mapping: serialize token traffic under
stable route key `oauth-token` before discovery and learned policy name
`token-request-limit` afterward, with no HEAD probe to the token endpoint.
Remove the stale `Paid`-style proof only as part of this integration.

Keep N4 strictly separate from N5. Preserve N2 refresh ownership and OAuth
behavior, N3's gate semantics, dispatcher semantics, and all frozen design
documents. Do not implement dispatcher cleanup or contact GGG.

Run cargo test --workspace --all-targets; cargo clippy --workspace
--all-targets --all-features -- -D warnings; cargo clippy -p acquisition-core
--all-targets --all-features -- -D warnings; and cargo fmt --all -- --check.
Record exact outcomes. Commit only N4 with an N4-labeled message. Do not mark
N4 accepted. Return the exact starting and ending hashes, clean worktree state,
scope completed, checks, unresolved findings, and the single next action: an
independent N4 review of the exact coordination-tip-to-N4-tip range.
```

Only an independent N4 review with no required work remaining may mark N4
accepted. N4 must remain a separate package and review from N2, N3, and N5.
