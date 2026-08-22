# Network cleanup — closed record

The Rust playground's network cleanup (packages N0–N6) is complete and
closed as of 2026-08-22. This file is the permanent short record: what was
built, in which exact ranges, and what was found. Per-package narratives,
fix-session notes, and validation transcripts were removed on closure and
live in git history (the file at `8198bef8` holds the full text).

No further network-cleanup package is authorized by this record. New work
on the network layer is defined under its own control document; the
properties the packages established are recorded in `CONTEXT.md`.

## Authority

When sources disagree: `AGENTS.md` and `CONTEXT.md` for repository and
spike invariants; `../../docs/design/network-redesign.md` for the frozen
C++ target design (a property source for the Rust code, see `CONTEXT.md`);
`../../docs/design/network-ground-truth.md` for numbered observations;
`../../docs/cleanup/findings.md` for the C++ findings register. This file is
history only.

## Final state

- Frozen gate-contract baseline: `1e17e812`.
- Accepted implementation tip: `29a2214be210097bfba1ac85cc3d78da2683d941`.
- Quality-gate baseline at that tip: `cargo test --workspace --all-targets`
  (69 core tests, 0 CLI tests, 0 failures); `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`; `cargo clippy -p
  acquisition-core --all-targets --all-features -- -D warnings`;
  `cargo fmt --all -- --check`; `git diff --check` — all passed. Later work
  on the spike keeps all of these green.

## Package ledger

| ID | Package | Accepted range |
| --- | --- | --- |
| N0 | Ground truth and OAuth gate decision | through `1e17e812` |
| N1 | Strict parsing, observation/classification, bounded 429 recovery | `1e17e812..412c840e` |
| H0 | Workspace formatting and strict-Clippy baseline | `694fb10c..f1fcb24e` |
| N2 | OAuth refresh singleflight and session generations | `a7434126..0a47efec` |
| N3 | Send-lifetime gate primitive and fairness semantics | `7f205d84..510ea498` |
| N4 | Gate integration in `ChokePoint`; remove `Paid` | `bd9732d1..32e591c7` |
| N5 | Dispatcher cleanup; remove job-task head-of-line blocking | `f6b1e6cb..cca89516` |
| N6 | Integration stress tests and frozen-design reconciliation | `cffbf8b6..29a2214b` |

Every package had an independent read-only review of its exact range and
was accepted; findings were fixed in separate commits and re-reviewed.

## Findings

| Finding | Severity | Summary | Fix commit |
| --- | --- | --- | --- |
| N1-R1 | High | Strict parsing accepted `u64` values that overflowed deadline arithmetic and panicked; bounded and checked | `412c840e` |
| N1-R2 | Medium | Clean-2xx classification and send recording happened before body completion; deferred to body resolution | `412c840e` |
| N1-R3 | Low | Bounded retry/probe behavior was not pinned through the real dispatcher requeue lifecycle | `412c840e` |
| N2-R1 | High | An abandoned refresh owner (abort/drop) stranded waiters forever; `RefreshOwnerGuard` publishes on drop | `0a47efec` |
| N4-R1 | Low | `Limiter::eta_for` bypassed the token policy's conservative 60 s bucket, understating ETA by 55 s | `32e591c7` |
| N6-R1 | Low | Integration stress did not actually reach concurrent refresh; rebuilt with a held token response as barrier | `29a2214b` |

## Process used (for reuse decisions)

Build / review / fix / coordination sessions, each owning one role and one
package; exact base-to-tip hashes as review boundaries; no stacking of new
semantic work on a package under review. This was the right cost for
replacing concurrency semantics under a frozen spec. It is not the default
for later spike work — choose the process per the risk of the work.
