# Clause registry design note

Status: **accepted by Tom, 2026-08-12 — rev 1, D1–D4 all as
recommended.** This note is now the implementation contract for the
registry slice; `kickoff-registry-slice.md` is cleared to fire.
Disposition of the §7 open questions at acceptance: item 1 (X2 as
`Untested` / open) proceeds as proposed — it is the conservative
default and adds no scope; item 2 (a `SHELL`-owned entry for the
dropped dispatched-ticket lifecycle) **remains Tom's call** — if
still undecided when the slice runs, the implementing session omits
the entry and records the unowned gap as a finding in its hand-off,
per the kickoff brief's licensing rules; item 3 stands as written
(row collapses during review are rev-2 edits, not design changes).

Produced 2026-08-12 by the obligation-map audit session, at head
`f03632b9` (map) / `e2034807` (code). The migration source and
acceptance oracle is `obligation-map.md`.

*[Marker, 2026-08-13 (DS-R1): the slice ran and closed review
2026-08-12 (REG-R1) — this note is now history. Two §6 updates: the
first bullet (deriving `verdict_eligible()` from the registry) was
**declined as designed by Tom 2026-08-13** — the derivation runs
backwards (clauses become `Full` because full-contract runs land),
so the driver keeps its per-run declarations and a two-authority
slot-fill rule in `AGENTS.md` replaces the wiring; the second
bullet's prediction did not happen as written — the doc-split
kickoff downgraded the M-row collapse to a preamble sentence, and
the resulting drift risk is noted in `status.md` §5 item 2.]*

## 1. Problem

Since the actor slice closed, every review round's findings have been
defects in what the evidence *claimed*, not in the code: a phase sweep
that measured nothing (F1, F7), a hard-coded G5 (F2), an obligation
filed against the wrong row twice in one day (round four), and the
audit's §8 findings — X2 claimed-carried with no test, M12's row
claiming C4's already-green matrix, finding 11 contradicting the C3
row 130 lines above it. The mechanism is always the same: coverage
truth is maintained by hand, in prose, in five to eight places at
once, and the ownership rule ("find the row that owns X before
writing 'X is untested'") asks each reader to perform a join across a
42 KB contract and a 90 KB register from working memory.

Every time a prose rule was converted into a structure, that failure
class stopped: `ContractCoverage` ended verdict-overclaim,
`swept_phases_are_separated_by_a_full_bucket` ended the phase
misreading, `ScenarioAssertionId` ended the unrelated-passing-flag.
This note applies the same move to the coverage ledger itself.
`obligation-map.md` is the right content in the wrong medium — prose
that starts going stale with the next slice.

## 2. Goal

A machine-checked clause registry such that:

- every assertion clause has **exactly one owner** (class 1 of the
  audit report becomes unrepresentable);
- every clause is either **cited to discharging tests that provably
  exist** or carries an **explicit untested/excluded record with
  provenance** (classes 2 and 4 become test failures);
- U-series and O-series exclusions are enforced **negatively** — a
  test claiming an excluded clause fails the build (exclusion drift
  becomes visible);
- coverage-state changes are **deliberate diffs to one file**,
  reviewable in isolation, instead of synchronized prose edits.

What it cannot do, stated honestly: a test whose assert is weaker
than its clause (class 3) is a judgment call no registry detects.
Mitigation is a one-line `must_assert` note per citation giving the
reviewer something concrete to check the test against. Ambiguity
(class 5) stays human and stays in the registers.

## 3. Non-goals (slice scope control)

- **No judge or verdict rewiring.** Deriving a scenario's
  `ContractCoverage::FullContract` from "all its registry clauses are
  Full" is the natural payoff, but it is a second, separate decision —
  this slice only creates the data and its verification.
- **No result-draft restructuring.** The M-row delta prose stays
  where it is; relocating live state is the doc-split slice's job.
- **No new coverage.** The registry *records* the untested clauses
  (tripwire feed, C3 latch, X1 drain, X2); writing those tests is
  later, separately reviewable work. A registry slice that also
  changes what is tested is two slices in one commit stream.

## 4. Decisions for review

### D1. Granularity: one entry per obligation-map row (~120)

- **(a) Recommended — full map granularity.** Every row of
  `obligation-map.md` §§1–6 becomes one entry: M-series clauses,
  C-series properties, X1/X2, G1–G6, B1–B14. U1–U4 and O1–O8 enter as
  `Excluded` entries (see D4) so both registers are enforced, not
  merely listed. The misfiled obligations that motivated this all
  lived at clause level, not scenario level — coarser granularity
  rebuilds the blind spot.
- (b) Coarser: registry only at the scenario-assert level (13 M
  entries + series rows). Rejected: this is what
  `ScenarioAssertionId` already is, and it was not enough — the
  round-four defect was two rows *inside* M10/C3 granularity.
- (c) Clause level for M/C/X only, prose for G/B/U/O. Rejected: the
  G-summary and X2 seams are exactly where the audit found the
  unfilled-vs-claimed contradiction.

Clause IDs are stable kebab strings namespaced by owner —
`m12-tripwire-feed`, `c3-trip-latched`, `x2-single-send-path`,
`b1-retry-after-emission` — chosen for greppability and stable diffs.

### D2. Location: new `src/obligations.rs`, const data, no new deps

- **(a) Recommended — a new pub module `src/obligations.rs`** holding
  `pub const CLAUSES: &[Clause]`, in the `SCENARIOS` style. It must be
  a lib module (not `tests/common`) because the citations point into
  both `tests/*.rs` and `#[cfg(test)]` modules inside `src/`, and the
  verification test needs the table from an integration-test context.
  A few KB of const data in the production build is an acceptable
  spike cost, and it keeps `conformance.rs` about judging runs, not
  cataloguing tests.
- (b) Extend `conformance.rs`. Workable, but that file is the judge;
  mixing in ~120 rows of evidence metadata makes both harder to
  review.
- (c) A TOML/JSON data file parsed by the test. Rejected: adds a
  parser dependency or hand-rolled parsing, loses compiler-checked
  structure, and the repo precedent (`SCENARIOS`) is const Rust.

A plain struct table, **not** a 120-variant enum: nothing dispatches
on clauses at runtime; exhaustiveness is supplied by the verification
test (uniqueness + completeness), and a table keeps additions to
one-line diffs.

### D3. Tagging: central citations + source-existence verification

- **(a) Recommended — citations live in the registry, verified
  against source.** Each clause carries
  `Citation { file: &str, test_fn: &str, must_assert: &str }`. The
  verification test (an integration test, `tests/obligations.rs`)
  reads each cited file from the crate root and confirms a
  `fn <test_fn>` definition exists — the audit's class-4 check,
  automated, including the `src/` unit modules a `tests/` grep
  misses (the round-four trap). No annotations in the 129 existing
  tests, no churn, one file to review.
- (b) Encode clause IDs in test names and scan for them. Rejected:
  renames churn history, multi-clause tests get unwieldy names, and
  the map direction (clause → tests) is the one reviewers need.
- (c) A proc-macro attribute on tests. Rejected: a new dependency and
  compile machinery for what one file-reading test does.

Known limitation of (a): a citation can point at a test that exists
but was weakened. That is class 3, inherently judgment; `must_assert`
is the reviewer's hook, and the slice-review walk (step 4) already
owns it.

### D4. Coverage states and the untested discipline

```rust
pub enum Coverage {
    Full,                         // citations discharge the clause at its stated scope
    Partial,                      // citations + note stating the exact missing delta
    Untested,                     // zero citations; note carries reason AND disposition
    Excluded,                     // U/O rows; citations on these FAIL verification
}

pub struct Clause {
    pub id: &'static str,         // "m8-single-retry-in-flight"
    pub owner: &'static str,      // "M8", "C3", "X2", "G6", "B12", "U1", "O5"
    pub text: &'static str,       // one-line clause + its scenarios.md line anchor
    pub coverage: Coverage,
    pub citations: &'static [Citation],
    pub note: &'static str,       // delta / reason / decision provenance
}
```

Two rules give `Untested` teeth:

1. Every `Untested` note must name a disposition: a register decision
   ("accepted, `result-draft.md` §3 …") or the literal
   `open — flagged for Tom`. Silent untested state is
   unrepresentable.
2. The verification test compares the set of `Untested`-and-open IDs
   against an explicit expected list (`OPEN_UNTESTED: &[&str]`) in the
   same file. Changing coverage state therefore touches two lines —
   the entry and the list — making every state transition a
   deliberate, reviewable diff rather than a drive-by edit. (This is
   the `swept_phases` pattern: state the rule over whatever the
   constant holds.)

### Verification test — full assertion set

- IDs unique; owners drawn from the known vocabulary.
- `Full`/`Partial` ⇒ ≥ 1 citation; `Untested`/`Excluded` ⇒ 0
  citations.
- Every citation's file exists and contains `fn <test_fn>`.
- Every `Untested` note carries a disposition (string check).
- `OPEN_UNTESTED` equals the computed open set exactly.
- Reachability: the table is non-empty and covers every owner series
  (M1–M13, C1–C5, X1–X2, G1–G6, B1–B14, U1–U4, O1–O8 each appear).

## 5. Acceptance criteria (the slice is not done at green)

1. **Row-for-row reconciliation against `obligation-map.md` at
   `f03632b9`.** Every map row has a registry entry with matching
   owner, coverage class, and citations. A discrepancy found during
   migration is a *finding to record*, never a silent fix — the map
   is the oracle for its own replacement.
2. **Mutation-verified, each documented in the hand-off:** remove one
   citation → fails; point a citation at a nonexistent fn → fails;
   misspell a cited file → fails; add a citation to an `Excluded`
   clause → fails; remove an ID from `OPEN_UNTESTED` without changing
   the entry → fails; duplicate an ID → fails.
3. **`obligation-map.md` marked superseded-by-registry** (dated text
   preserved, marker at top — the `core-design.md` precedent), so the
   prose copy cannot mislead a later reader.
4. Full gate matrix green: `cargo test --locked` debug and release,
   4,096-case properties, clippy `-D warnings`, fmt, diff check.
5. Standard slice close: four-part hand-off, register/changelog
   entry, Tom's review.

## 6. Payoff wiring (recorded, deferred)

- `verdict_eligible()` / per-scenario `FullContract` derived from the
  registry ("all owned clauses Full") — retires the hand-maintained
  fragment declarations in the driver. Separate decision after the
  registry has bedded in.
- The result-draft M-row "remaining" prose collapses to registry
  pointers — the doc-split slice's job, which is why this slice runs
  first.

## 7. Open questions for Tom (rev 1)

1. **X2's entry.** Propose `Untested` / `open — flagged for Tom`,
   with the note recording that scenarios.md demands a structural
   test and none exists (audit §8.2 item 2). Deciding what a
   spike-scope X2 test *is* stays outside this slice.
2. **The dropped dispatched-ticket lifecycle** (audit §8.1 item 2)
   has no owner row in `scenarios.md` at all. Propose a registry
   entry under a new owner tag `SHELL` (the external-review shell
   obligation) marked `Untested` / open, so the gap is carried by the
   machine-checked list instead of a confession paragraph. Accepting
   that entry is a small scope decision — it adds an obligation no
   scenario currently states.
3. **Granularity sanity check.** ~120 entries is the audit's row
   count. If any series feels over-split in review (B-series in
   particular), collapsing adjacent rows is a rev-2 edit, not a
   design change.
