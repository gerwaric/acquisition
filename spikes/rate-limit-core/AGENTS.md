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

## Working style

- Tom is hands-on, and this spike doubles as his first substantial
  Rust work (charter provenance note): explain idioms as they arise;
  don't just emit them.
- Commit style: `spike(rate-limit-core): <what>` — small commits, one
  concern each, decision provenance in the body.
- Sketch-level names in `core-design.md` may shift in implementation;
  contract semantics may not.
