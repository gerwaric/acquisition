# Implementation-phase instructions: rate-limit-core spike

Scope: any agent doing implementation work in `spikes/rate-limit-core/`
on branch `spike/rate-limit-core`. The design phase is complete,
externally reviewed, and frozen (2026-08-09). The committed documents
are the sole authority — sessions start with no other context, by
design; if that ever proves insufficient, that is itself a finding.

## Read before writing any code, in this order

1. `design-brief.md` — the charter. Frozen decision record: the
   reconciliation log and plan-review addendum are settled. Do not
   re-litigate them.
2. `scenarios.md` — the test contract: scenarios M1–M13 / C1–C5 /
   X1–X2 / U1–U4, gates G1–G6, §7 mock fidelity budget (B/O series).
   Tests cite scenario IDs the way designs cite N-numbers.
3. `core-design.md` — the API contract being implemented: types,
   request lifecycle, reconciliation and phantom synthesis, episode
   semantics and confirmation matrix, response precedence, probe
   outcome table. When code lands, code becomes authority and this
   doc becomes history.
4. `result-draft.md` — the evidence skeleton. Fill slots as results
   land, the day they land; never by assertion, always by citing a
   test run, commit, or fixture.

Authority for cited claims: `docs/design/network-ground-truth.md`
(N-numbers) and `docs/design/network-redesign.md` (D-numbers).

## Build order (charter — not negotiable)

1. Sans-IO core state machine, driven by the C1–C5 property tests.
2. Mock + conformance harness (M-series, §7 budget) after the core
   is green.
3. Tokio actor shell last.

## Hard constraints

- **Never contact pathofexile.com or any live service. Ever.** All
  testing is offline (`tokio::time::pause`; the mock is in-process).
- Greenfield code stays inside `spikes/rate-limit-core/`. No changes
  to `src/`. No commits to `master` or `redesign`.
- The docs are the whole spec. If they are ambiguous or silent on
  something needed, that is a **doc finding**: record it in
  `result-draft.md`, take the conservative reading, and flag it for
  Tom — do not silently improvise.
- Raw `networkcapture` output is never committed; fixtures enter only
  through the §4 sanitization contract in `scenarios.md`.

## Slice hardening rules (2026-08-09 audit post-mortem)

An implementation audit found that defects clustered exactly where
the docs were silent and at seams between slices built by separate
sessions (register in `result-draft.md` §3). These rules exist so
that shape of failure cannot recur. They bind every session.

- **A slice is not complete at green.** Green tests, clippy, and fmt
  end the *coding*; the slice ends when Tom has reviewed it. Present
  the slice for review with the four-part hand-off defined in
  `slice-review.md` §2: silences taken (with consequences traced),
  the seam map with the invariants walk, the coverage confession,
  and your judgment calls.
- **Bound every wire-derived quantity** before it sizes an
  allocation, a loop, or a deadline. "The docs state no bound" is
  not permission for unboundedness — it is a doc finding plus a
  conservative cap, recorded together.
- **Property tests carry a reachability guard.** A property that can
  skip its assertion must demonstrate it cannot pass vacuously —
  assert on every generated branch, or count and assert assertion
  reachability. "Green at 4,096 cases" is meaningless without this.
- **Oracles never call production code.** If the expected value is
  computed via the functions under test, it is a mirror, not an
  oracle. Independent arithmetic only.
- **The doc-finding trigger is "the docs don't mention this case"** —
  not "and the answer seems non-obvious." Record it even when the
  conservative reading feels self-evident, and trace the
  *consequence* one step past the disposition (what does the engine
  do on the very next call?). The unusable-`Retry-After` bug lived
  in that untraced second step.
- **Guards are structural, not per-call-site.** If you find yourself
  re-checking a shape an existing entry point already guards, the
  invariant belongs in a constructor (make the state
  unrepresentable), not copied to the new path.
- **Closing a review round is three acts, not one:** fix the
  findings, flip the hand-off status line, and write the register
  entry — `slice-review.md` §5. Fixed findings under a status line
  still reading "awaiting review" is the failure mode this rule
  exists to stop; it has happened.
- **The repo is the only shared channel.** Sessions differ — some
  agents keep private memory, external reviewers keep none, and
  neither can read the other's. Any status, decision, or process
  rule that must outlive your session belongs in a file here.

### Cross-slice invariants — re-verify against every new slice

Each of these spans slices, so no single session's tests defend it
automatically. When your slice touches state another slice owns,
state in your review hand-off how each of these still holds:

1. **No permanent wedge.** Any state a token or entry can hold
   (history entries, episode confirmation slots, anything added
   later) must resolve by explicit consumption *or* age out by
   window passage. A dropped token may degrade throughput, never
   schedulability.
2. **One send, one entry.** Reservation identity is exact:
   remove-by-id, no double-count, no loss, under any interleaving —
   including operations added by later slices.
3. **Pessimism direction.** Local state never understates
   server-visible state (up to the synthesis cap); entries leave
   history only by window passage or rollback of an undispatched
   reservation.
4. **`try_reserve` is the single scheduling authority.** No new
   path may hand the shell a second source of send timing.
5. **Entry-point invariant.** `on_response` never yields
   `ProbeReady`; `on_probe_response` never yields `CompleteRequest`
   or `Requeue`.
6. **Notifications tell the truth.** `StateChanged` is emitted iff
   the call mutated engine state.

## Working style

- Tom is hands-on, and this spike doubles as his first substantial
  Rust work (charter provenance note): explain idioms as they arise;
  don't just emit them.
- Commit style: `spike(rate-limit-core): <what>` — small commits, one
  concern each, decision provenance in the body.
- Sketch-level names in `core-design.md` may shift in implementation;
  contract semantics may not.
