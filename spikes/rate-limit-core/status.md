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
`src/obligations.rs` — **not** any prose table. As of 2026-08-13
(post ballot pass) it holds 13 ids, each a genuinely owed test
(`m9-headroom-record` left the list by demotion to the U5
exclusion; earlier the same day `m7-threshold-tuning` resolved by
wording and `c4-halt-semantics-shared` was demoted honestly —
decisions 4 and 5). `obligation-map.md` is the
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

**None.** The 2026-08-13 ballot-pass amendment drafts were
approved by Tom as-is the same day and are applied: five
`scenarios.md` amendment blocks (M4, M11, M12, G6, U5), the
`result-draft.md` §1 SHELL-prerequisite sentence, and the seven
registry flips. Earlier the same day the decisions pass resolved
all six prior standing items (full dispositions: the
`result-draft.md` §9 entries). A new decision gets a numbered
item here.

## 4. Blocked

- **§7.4 capture replay** — blocked on a sanitized fixture: raw
  `networkcapture` input from Tom passed through the `scenarios.md`
  §4 sanitizer, or a fixture already satisfying that contract
  (mock-slice doc finding 8, `result-draft.md` §3 register;
  `mock-handoff.md` §1). No record may be reconstructed from prose,
  and no synthetic stand-in may be claimed as observed evidence.

## 5. Next work

This list is the 2026-08-13 coverage ballot's outcome (register:
`result-draft.md` §9 ballot-pass entry) and it is **closed**:
entries leave by test discharge or approved amendment; a new
obligation enters only from a failing test or a numbered §3
decision. Every non-Full registry clause was balloted; each work
item below names the clauses it discharges.

1. Fix F14–F16 (substance in the `result-draft.md` §9 addendum,
   2026-08-13) and close scenario-driver round four.
2. Ballot work items (test code; letters per the ballot):
   - **A** — probe-429 wire arm (boot HEAD scripted 429 through
     the actor): `m1-probe-429-seeding`,
     `m1-probe-429-first-get-confirmation`,
     `m1-probe-429-tripwire-feed` (feed assert, ties to H).
   - **B** — two-endpoint rig (second policy flowing while the
     first is cooled/failed): `m3-other-policies-unaffected`,
     `m4-other-policies-flowing`, `m1-boot-head-discipline`.
   - **C** — M1 residue sweep incl. zero-remaining-budget:
     `m1-no-first-request-violation`, `m1-g1-sweep`.
   - **D** — M2 saturation depth + runtime-computed G4 minimum
     (retires the 2,550 ms literal per scenarios.md:571):
     `m2-burst-stall-drain`, `m2-g1`, `m2-g3-g4-bounds`,
     `g4-m2-duration-bound`.
   - **E** — B12 M5 timing script (forced stale window), plus one
     M6 shrink-variant arm riding it: `m5-stale-window-exposure`,
     `m5-no-violation-after-merge`, `m6-preannouncement-exposure`.
   - **F** — B12 M8 timing script (concurrent in-flight
     originals): `m8-b12-timing-script`,
     `m8-single-retry-in-flight`.
   - **G** — B12 M9 timing script (phantom race at 14/15):
     `m9-recovery-survives-race`, `m9-race-exposure-attribution`.
     E+F+G complete `b12-scripted-delay`'s required script set.
   - **H** — fuse-trip batch (latch re-ask, wire-4xx trip →
     halt/drain/publish, feed-deletion asserts): `c3-trip-latched`,
     `c4-halt-semantics-shared`, `x1-trip-drain-publish`,
     `m12-tripwire-feed`.
   - **I** — X2 structure pin per decision 2:
     `x2-single-send-path`.
   - Singles: `m3-cooldown-clean-failure` (60 s re-entry assert);
     `m4-watch-status-published` (watch assert on D4 cooldown);
     `m11-compliant-never-trips` (M11a near-ceiling sweep arm,
     G2's named binding evidence); `shell-dropped-dispatched-ticket`
     (drop-after-dispatch test per its note);
     `g5-scenario-assertions` (unauthorized-refusal teeth);
     `b1-header-protocol` (organic-429 Retry-After wire assert).
3. §7.4 capture replay, when unblocked (also retires
   `b12-scripted-delay`'s 50 ms placeholder confession).
4. Verdict slots last — the full-contract run (which also finishes
   the fragment-scale-only partials: `m6-g1-post-announcement`,
   `m6-queue-drains-new-pace`, `m7-no-client-violation`,
   `m8-no-follow-on-violation`, `g1-zero-client-violations`,
   `g2-ceilings-never-tripped`, `g3-over-delay-bounded`); the fill
   takes two agreeing authorities: the run's declaration and the
   registry showing every owned clause `Full` (AGENTS.md standing
   rule).
