# Live state: rate-limit-core spike

This is the **single live-state file** for the spike, created
2026-08-12 by the doc-split slice. Everything else in this directory
is contract (frozen design docs) or history (dated hand-offs, the
`result-draft.md` registers and changelog). If a statement here
disagrees with a status line elsewhere, this file wins; if this file
is stale, that is a process failure — closing any review round
updates it (`slice-review.md` §5). Keep it small: one live statement
per fact, pointers instead of copies.

## 1. Coverage truth

Coverage is **machine-checked**, not prose. The authority is
`src/obligations.rs` — `CLAUSES` (122 entries as of 2026-08-13;
lineage from `obligation-map.md`'s 125 rows is recorded in
`registry-handoff.md` and the §9 changelog) — verified by
`tests/obligations.rs`; run it with

```
cargo test --locked --test obligations
```

The checks are structural (unique ids, known owners,
citation/coverage arity, cited test fns exist, exact open-set
match); coverage *class* and `must_assert` accuracy are reviewed
prose, per the registry's coverage confession
(`registry-handoff.md` §3 — still the binding statement of those
limits, though that hand-off is otherwise historical).

The open-untested list is the `OPEN_UNTESTED` constant in
`src/obligations.rs` — **not** any prose table. As of 2026-08-13 it
holds 14 ids, each a genuinely owed test (the one ambiguity id,
`m7-threshold-tuning`, resolved by wording and was removed;
`c4-halt-semantics-shared` demoted honestly — decisions 4 and 5). `obligation-map.md` is the
superseded prose ancestor; read its §8 for the audit's discrepancy
analysis (dated at `e2034807`), never for current coverage.

## 2. Slice and review state

- **Open: scenario-driver slice** (M1–M13 driver/judge integration),
  in review round four. Rounds one–three findings (F1–F10, driver
  register) and doc findings 11, 12(a), 12(b), and 13 are fixed;
  12(c) is decision 1 below. Round four (2026-08-12) changed no
  driver code but recorded findings **F14–F16**, **unaddressed** —
  substance re-derived 2026-08-13 in the `result-draft.md` §9
  addendum (twin M8 arm missing a conjunct + duplicated cap guard;
  duplicated floor literal; fail-open oracle fallback, currently
  unreachable). They are the next coding work this slice owes. Its
  hand-off, `scenario-driver-handoff.md`, keeps its **live coverage
  confession** until the slice closes; read it there, not here.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

**None.** The 2026-08-13 decisions pass resolved all six standing
items: G3 ε final at 500 ms; the X2 spike-scope test defined as a
structure pin (production re-pins later); the dropped-ticket
lifecycle adopted into the registry; the six §8.5 ambiguities
amended into `scenarios.md` (one sentence deleted outright);
`c4-halt-semantics-shared` demoted honestly to Untested; registry
payoff wiring declined in favor of the two-authority slot-fill rule
(AGENTS.md). Full dispositions and reasoning: the `result-draft.md`
§9 decisions-pass entry. A new decision gets a numbered item here.

## 4. Blocked

- **§7.4 capture replay** — blocked on a sanitized fixture: raw
  `networkcapture` input from Tom passed through the `scenarios.md`
  §4 sanitizer, or a fixture already satisfying that contract
  (mock-slice doc finding 8, `result-draft.md` §3 register;
  `mock-handoff.md` §1). No record may be reconstructed from prose,
  and no synthetic stand-in may be claimed as observed evidence.

## 5. Next work

1. Fix F14–F16 (substance in the `result-draft.md` §9 addendum,
   2026-08-13) and close scenario-driver round four.
2. Raise M-row fragments toward full contracts. The open ids are
   `OPEN_UNTESTED`; the audit's narrowed per-row deltas (M8/M1/M4 in
   `obligation-map.md` §8.2 items 3–5, M12 in §8.1 item 1) are dated
   analysis at `e2034807` — a map, not an authority. The
   `result-draft.md` §3 M-row prose was deliberately not collapsed
   onto registry pointers (kickoff downgrade of registry-design §6),
   so those cells can drift from the registry; where they disagree,
   the registry wins.
3. The X2 structure pin per decision 2 (G3/G4 finalization landed
   2026-08-13 — no longer owed).
4. §7.4 capture replay, when unblocked.
5. Verdict slots last — only a `verdict_eligible()` full-contract
   run may fill one, and the fill takes two agreeing authorities:
   the run's declaration and the registry showing every owned
   clause `Full` (AGENTS.md standing rule).
