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
`src/obligations.rs` — `CLAUSES`, 122 entries migrated from
`obligation-map.md`'s 125 rows (two U-register pointer rows
collapsed, one deliberate omission: the dropped-ticket clause,
decision 3 below; arithmetic in `registry-handoff.md`) — verified by
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
`src/obligations.rs` — **not** any prose table. As of 2026-08-12 it
holds 13 ids; per REG-R1-F2 they are not thirteen owed tests
(`m7-threshold-tuning` is an ambiguity that may resolve by a
`scenarios.md` wording decision). `obligation-map.md` is the
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

1. **G3 epsilon** — model N13 padding in the oracle and tighten
   epsilon to ~100 ms, or keep ~500 ms and record that G3 cannot
   discriminate below the padding envelope. Doc finding 12(c),
   `result-draft.md` §3. Blocks the `scenarios.md` §6 G3/G4
   finalization, which blocks every verdict slot.
2. **What a spike-scope X2 structural test is** — the required
   "one client, no second send path" test is unbuilt and no
   production transport exists to pin; the accepted limitation covers
   the parser cap only. `obligation-map.md` §8.2 item 2.
3. **Owner for the dropped dispatched `RequestTicket` lifecycle** —
   confessed untested; no scenario, C row, or X row owns it, so it
   can stay untested forever without any row going red.
   `obligation-map.md` §8.1 item 2; carried as a finding in
   `registry-handoff.md` (it is deliberately absent from `CLAUSES`).
4. **The §8.5 ambiguity flags** — requirement text two readers would
   allocate differently; **M11a-vs-G2 ownership first** (who owns
   "the compliant client never trips the ceiling": M11's dedicated
   sweep or G2-everywhere). Also there: M7 threshold-tuning wording
   (= `m7-threshold-tuning` in `OPEN_UNTESTED`), M2's N25 reading,
   G4's "harness-computed", M1 boot-HEAD serialization scope, C3's
   "~500" vs the exact 500 pin. `obligation-map.md` §8.5.
5. **REG-R1-F4 deferral** — `c4-halt-semantics-shared` stays
   `Partial` as migrated; reclassification deferred until the
   latch/feed tests are sequenced. REG-R1 closure entry,
   `result-draft.md` §9.
6. **Registry payoff wiring** — derive `verdict_eligible()` /
   per-scenario `FullContract` from the registry, retiring the
   driver's hand-maintained fragment declarations; deliberately "a
   second, separate decision after the registry has bedded in"
   (`clause-registry-design.md` §6). Tracked here since DS-R1; no
   urgency implied.

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
3. G3/G4 finalization per `scenarios.md` §6 (needs decision 1).
4. §7.4 capture replay, when unblocked.
5. Verdict slots last — only a `verdict_eligible()` full-contract run
   may fill one; no fragment run qualifies.
