# Mock + M-series harness slice hand-off

Status: awaiting Tom review per `slice-review.md` §2. Implemented
2026-08-10 from reviewed bootstrap head `b3a0e7d5` (the named
`17363429` baseline plus its review-status reconciliation commit).
Implementation commits: `4353fb03`, `05ee15d1`, `12a799f8`,
`4c69f05e`.
No actor or live-service work is included.

## 1. Silences taken, with next-call consequences

| Silence | Conservative reading taken | Next-call consequence |
|---|---|---|
| B13 does not bound a run's wire-driven history/log growth | 10,000 received requests per run and 10,000 retained events per policy; mock definitions, raw stimuli, scripts, diagnostics, and delays have structural constructors/bounds | request 10,001 latches harness exhaustion without allocating/logging; every later send returns the same typed transport error |
| The docs do not say whether an arrival during an already-active restriction renews it | active restriction alone refuses but does not renew; a newly over-limit arrival may extend it under the active rule | another early call remains refused through the original/latest over-limit deadline; at the deadline the current counters alone decide |
| Restriction identity across arbitrary policy rule reorder/reshape is unspecified | retain hits as facts and copy the latest old active restriction deadline to every new rule/window slot | the first call after mutation cannot escape a live restriction because its old positional slot disappeared |
| B10-vs-B2 evaluation order is unspecified | layer 1 is outside layer 2; a B10 challenge wins before policy counting | the next call still sees the rolling layer-1 arrivals, while policy counters remain at their pre-challenge value |
| CN5/N11–N13 do not define exact-boundary bucket ownership | use the most-adversarial `[start, end)` reading: an arrival exactly on a boundary enters the new bucket and gets a full bucket extension | at `bucket_end + period - 1 ms` the hit is active; at the exact expiry it is gone |
| B14 requires zero skew but names no calendar epoch | map simulated t0 to Unix epoch; emit Date at HTTP-date second precision on full, degraded, malformed, and Cloudflare responses | the next response's Date advances deterministically with its B13 arrival timestamp and never reads wall time |
| B12 gives no delay bound or sub-millisecond convention | scripts accept whole simulated milliseconds up to six hours per arrival/response leg | an over-bound or fractional-millisecond script is rejected before insertion; the next request uses the ordinary/default script rather than an unrepresentable deadline |
| The actor contract says in-flight requests are never aborted, but the mock-future drop case is not specified | record the received-wire observation before response delay and give occupancy a deterministic completion deadline | dropping the future loses the caller outcome but not the arrival; after the deadline, the next arrival prunes occupancy and cannot be falsely marked overlapping |
| §7.4 requires the July 18 sanitized 132-record fixture, but no sanitized or raw capture exists in the branch/workspace | implement and test the §4 allowlist sanitizer, but do not invent a fixture or reconstruct observed records from prose | no replay verdict is claimed; the next replay remains unavailable until Tom supplies raw input to the sanitizer or a fixture already satisfying its contract |
| §4 says t0 is the first record but does not choose among a reply record's scheduled/sent/received fields | use the first available client-side instant in scheduled→sent→received order; never use second-precision server Date as origin | the next record preserves negative/positive relative timing honestly, while a first HEAD (received only) begins at `received_ms = 0` |

The exact-boundary item is already CN5 in `result-draft.md`; the
other implementation findings are recorded in its mock-slice §3
subsection. The absent fixture is a coverage/input finding, not a
mocked result.

## 2. Seam map and six-invariant walk

Earlier-slice state touched or consumed by this slice:

- raw `http` responses from the mock cross the existing production
  parser boundary unchanged; the counter model imports no production
  parser, scheduling, padding, reconciliation, or clock helper;
- the future actor will construct the existing `PolicyEngine` from
  each sweep's provenance-typed client bucket profile; the plan
  constructor refuses plans omitting shipped `Assumed(60s/60s)`;
- B13's run-wide correlation ID joins future actor dispatch records to
  mock arrivals; duplicate IDs are structurally refused;
- mock policy replacement preserves its own server hit/restriction
  facts but does not mutate core policy history; M5/M6 client adoption
  remains actor-slice work;
- the transport trait returns only a response/error. Timing samples and
  gate reports are evidence; neither can grant a send or inject state
  into `PolicyEngine`.

1. **No permanent wedge.** Mock in-flight occupancy has a scripted
   completion deadline and is pruned even if its future is dropped.
   Policy hits age by independent window passage. Harness-budget
   exhaustion is an explicit terminal test failure, not a sleeper or
   held permit. Existing core token/episode aging remains unchanged and
   green.
2. **One send, one entry.** Each received GET adds exactly one shared
   policy hit; HEAD adds none; layer-1-rejected traffic never reaches
   policy counters. Correlation identity is unique for the whole run,
   and every judged observation must have exactly one dispatch sample.
3. **Pessimism direction.** Mock mutation retains all hit facts and
   carries the latest active restriction across every new slot. G1
   excludes an organic violation only through a pre-observation
   correlation set capped at D5's two in-flight requests. The core is
   not mutated by the harness.
4. **`try_reserve` is the single scheduling authority.** The mock
   responds only after transport hand-off; the judge observes dispatch
   times but never supplies one. Retry stimuli carry headers, never a
   second client send-time channel.
5. **Entry-point invariant.** This slice adds no core response entry
   point and invokes neither one. HEAD/GET method identity is preserved
   for the future actor to route, and the pre-existing nine-shape core
   sweep remains green.
6. **Notifications tell the truth.** The harness creates no core
   notifications and cannot self-exempt G3 from client state. Future
   watch/publication behavior stays a required M3/M4/M11/M13 scenario
   assertion; the existing core `StateChanged` tests remain green.

## 3. Coverage confession and property reachability

Covered now: independent B1–B14 mechanics; N23's five-policy topology;
full/degraded/one-window/three-window/malformed/Cloudflare wire shapes;
post-increment state; residue/phantoms with client/phantom/residue
provenance; organic restrictions; rename/shrink; both layer-1 ceilings;
deterministic arrival/response delay and observable overlap; unique
correlation and reproduction records; G1–G6 judgment; exact n/n+1
bounds; and mandatory inclusion of shipped `Assumed(60s/60s)`.

Not covered, deliberately by build order: actor scheduling and every
end-to-end M-row verdict. In particular there is no proof yet of the
spacing floor, D5 gate behavior, endpoint probe lifecycle, queue drain,
cancellation/reprioritization, retry delivery, tripwire feed, watch
publication, client remap/shrink adoption, or G3/G4 under a real actor.
All M rows therefore remain partial in `result-draft.md`.

The §4 sanitizer is covered with synthetic secret-bearing raw records:
its output allowlist, five-endpoint mapping, relative-time rebasing,
provenance block, overwrite refusal, and exact record/line/header bounds
are executable. Not covered because the required input is absent:
§7.4's sanitized July 18 capture replay and state-matching diagnostic.
No synthetic stand-in is claimed as observed evidence. O1–O8 remain out
exactly as declared. Stochastic timing remains absent.

The new B3 property cannot pass vacuously: every generated case asserts
the assigned bucket end, then checks the hit is present one millisecond
before independently computed expiry and absent exactly at expiry. Its
oracle is plain test-local integer arithmetic and never calls mock or
production bucket/scheduling functions. The conformance judge is not a
property, but its anti-vacuity guards reject empty wire observations,
dispatch samples, and scenario assertions; reject duplicate/detached
correlations; and cross-check `(seed, phi)` against every swept
observation.

Gate evidence produced 2026-08-10, entirely offline:

- `cargo test --locked` — 99 debug tests green.
- `cargo test --locked --release` — 97 release tests green (two core
  drop-bomb tests are debug-only).
- `PROPTEST_CASES=4096 cargo test --locked` — all ten properties green
  at 4,096 cases.
- `cargo clippy --locked --all-targets -- -D warnings` — green.
- `cargo fmt --all --check` and `git diff --check` — green.
- `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests -p
  'test_sanitize_capture.py' -v` — four sanitizer tests green.

## 4. Judgment calls

- The mock is library code behind the same generic transport trait the
  actor will consume, rather than test-module-only code. That preserves
  §7.1's reusable-artifact/alternate-delivery-shim path.
- Server definitions use private fields plus fallible constructors.
  One- and three-window policies remain representable for M4; empty,
  oversized, zero-hit, invalid-name, and over-time shapes do not.
- A single scalar server phase is normalized independently by each
  window's bucket resolution. The recorded scalar is sufficient for G6
  and avoids a burst×sustained Cartesian phase vocabulary the docs do
  not define.
- Observations are appended at server receipt with their deterministic
  scripted completion instant, rather than only when the caller awaits
  completion. This preserves B13 truth if a future is dropped and is
  safe because scripts are deterministic whole-millisecond values.
- G4 is reported as not applicable outside M2, not vacuously passed.
  G1/G2/G3/G5 are evaluated globally on every run as §3 requires, even
  where the table names a different binding gate.
- Raw stimulus responses are bounded but otherwise preserved; B14 Date
  is applied afterward to every response. Policy-only still means the
  only *rate-limit* header is policy, while Date remains present.
- The 10,000-request/event ceilings are mine: comfortably above M10's
  “hundreds” and the 1,001-arrival B10 boundary, while making all
  wire-driven vectors and scans finite. Exact n/n+1 tests pin them.
- The sanitizer is a Python-stdlib tool rather than another Rust target:
  it stays dependency-free, can run beside the retained local capture,
  opens output in exclusive-create mode, and never needs the raw file in
  Cargo's test graph or the repository.
