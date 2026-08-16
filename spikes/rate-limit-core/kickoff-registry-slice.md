# Kickoff: clause-registry slice

> **Executed.** The slice ran and closed review 2026-08-12 (REG-R1;
> hand-off in `registry-handoff.md`). Preserved verbatim as the brief
> the slice ran under; its pinned numbers (e.g. the 129/127 test
> inventory) are of its writing date. (Marker added 2026-08-13,
> DS-R1.)

**Do not start until `clause-registry-design.md` carries Tom's
acceptance** (status line names the accepted revision). This brief
was written 2026-08-12 by the audit session that produced
`obligation-map.md` (`f03632b9`); the pointers below are that
session's context, frozen so you do not re-derive it.

You are implementing, not auditing and not raising coverage. Branch
`spike/rate-limit-core`, work in `spikes/rate-limit-core/`. All the
AGENTS.md hard constraints and slice hardening rules apply; this
slice ends at Tom's review, not at green.

## Read first, in this order

1. `AGENTS.md` — constraints, slice rules, hand-off chain.
2. `clause-registry-design.md` — the accepted contract. D1–D4 are
   settled there; do not re-litigate them.
3. `obligation-map.md` — your migration source **and acceptance
   oracle**. Every row becomes an entry; discrepancies you find are
   findings to record, never silent fixes.
4. `slice-review.md` — §1 lesson 4 especially; your verification test
   is that lesson made structural.

You do not need to read `design-brief.md`, `core-design.md`, or
`scenarios.md` end-to-end: the map already carries each clause's
`scenarios.md` line anchor. Consult `scenarios.md` only where a map
row's wording leaves you unsure what the clause *is* — and if that
happens, record it as a candidate ambiguity (audit report class 5)
rather than resolving it yourself.

## Scope

**Commit 1 — finding-ID namespaces (small, doc-only).**
`slice-review.md` §5 gains a short "Finding-ID namespaces"
subsection: (a) future review-round findings are round-scoped —
`<slice>-R<round>-F<n>`, e.g. `SD-R5-F1` for the scenario-driver
slice's round five; (b) a disambiguation key for the existing
collisions, added as a note, **never** by renumbering dated text:
bare `F1`–`F10` currently mean different findings in the external
design review register, the follow-up verifier register, and the
scenario-driver rounds (`F6` is both "confirmation matrix
two-attempt cap" and "AGENTS.md hand-off table omission"); `F57`/
`F61` belong to `docs/cleanup/findings.md`, a different register
entirely.

**Commits 2+ — the registry.**

- `src/obligations.rs`: `Clause`, `Citation`, `Coverage`,
  `pub const CLAUSES`, `OPEN_UNTESTED` — per the accepted design.
- `tests/obligations.rs`: the verification test with the full
  assertion set from design §4 (uniqueness, coverage/citation
  consistency, source-existence of every cited `fn`, disposition
  strings, `OPEN_UNTESTED` exact match, series reachability).
- Migrate every `obligation-map.md` row. Coverage classifications
  come **from the map** — you are transcribing an audited state, not
  re-auditing. If transcription reveals the map itself was wrong
  somewhere, that is a finding for your hand-off.
- Mark `obligation-map.md` superseded-by-registry at the top, dated,
  text preserved (the `core-design.md` supersession precedent).
- One dated changelog entry in `result-draft.md` §9. **No other
  result-draft edits** — the M-row prose, finding 11's text, and the
  gate table are explicitly out of scope (the doc-split slice owns
  relocations; the round-four owed corrections are that slice's
  commit 1).

## Pointers that save you a day

- The `#[cfg(test)]` modules a `tests/` search misses:
  `src/actor.rs:876` and `src/mock/model.rs:551`. Your
  source-existence check must handle both `tests/*.rs` and these.
- Citation targets in the map are already `path:line :: test_fn`;
  line numbers are at `e2034807` and may drift — cite by `fn` name,
  verify by content, not by line.
- The full test inventory is 129 functions (127 in release; the two
  drop-bomb tests are debug-gated). If your verification test's
  census disagrees with 129 at `e2034807`, stop and find out why
  before proceeding.
- Proptest properties define their `fn` inside the `proptest!` macro
  — a naive `^fn ` scan misses the indentation and a naive
  `#[test]`-adjacency scan misses the macro. The audit's extraction
  needed two passes; budget for that.
- `Excluded` entries: U1–U4 from `scenarios.md` §5, O1–O8 from §7.3.
  The negative rule (citations on these fail) is the whole point —
  test it by mutation like everything else.

## Judgment calls you are licensed to make

- Exact struct field names, module layout, error-message wording.
- Whether the verification logic is one test or several (keep the
  mutation checks meaningful either way).
- Collapsing a map row that turns out to be two spellings of one
  clause — record it in the hand-off.

## Judgment calls you are NOT licensed to make

- Changing any clause's coverage classification from what the map
  says (finding + hand-off instead).
- Adding tests to discharge `Untested` clauses — even cheap ones
  (the tripwire feed and C3 latch will tempt you; they are one-line
  tests and they are **out of scope**: a registry slice that also
  changes what is tested is two slices in one review). Note their
  readiness in the hand-off; Tom sequences them.
- Rewiring `ContractCoverage`, `verdict_eligible()`, or the driver.
- Editing `scenarios.md` or any register text beyond the changelog
  entry and the supersession marker.

## Definition of done

The five acceptance criteria in `clause-registry-design.md` §5,
verbatim — reconciliation against the map, the six mutation checks
documented in the hand-off, the supersession marker, the full gate
matrix, and the four-part hand-off presented for Tom's review.
Commit style: `spike(rate-limit-core): <what>`, decision provenance
in the body.
