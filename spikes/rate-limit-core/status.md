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
`src/obligations.rs` — `CLAUSES` (124 entries as of 2026-08-15:
the §7.4 gate clause minted per SD-R5-F11, and
`m2-character-policy-lanes` minted per the SD-R8-F5 extension;
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
`src/obligations.rs` — **not** any prose table. As of 2026-08-14 it
is empty: the implementation swarm discharged all 13 previously
open Untested ids.
Registry totals are 110 Full, no Partial, one accepted Untested
limitation (`x2-parser-cap-limitation`), and 13 Excluded. The seven
former fragment-scale clauses §5 item 5 names flipped to Full on
2026-08-15 after the declared 4,096-case run and are cited to that
run in the registry. (`s7-4-replay-gate` flipped to Full earlier on
2026-08-15:
the replacement calibration gate landed green per the ratified
spec's discharge line — §9 changelog. `m1-g1-sweep` flipped to
Full on 2026-08-14: the generated-φ mock-side residue sweep
landed — §9 changelog.)
Empty `OPEN_UNTESTED` alone does not imply verdict readiness; the
declared run and registry must also agree. The 2026-08-15 external
audit (SD-R8-F4, `result-draft.md` §9) showed both authorities
could pass with M8's required Known lane missing; **the F4 repair
landed the same day** — the declaration guard is now keyed to M8
(`MissingM8KnownLane`/`MissingM8AssumedLane`, negative test pins
the audit's exact state) and both authorities reran green over the
repaired guard (§9). The registry verifier remains structural by
design; its limits are unchanged. The SD-R8 re-close review
suspended the agreement again on F9 — endpoint coverage was read
from a reproduction label the judge did not bind to the wire — and
**the F9 repair landed 2026-08-15** (`3813c40a`): the judge now
binds `reproduction.endpoint` to every observation alongside seed
and phase, the structural seam test pins the relabel, both
authorities reran green under the binding, and the review's own
end-to-end mutation is refused (`ReproductionMismatch { id: 1 }`).
The binding also caught and fixed two real M9 label errors. The
repeated independent re-close review reproduced both authorities and
all three refusal mutations, found no new findings, and restored
verdict readiness (§2, `result-draft.md` §9). **The final external audit
then reopened that agreement with SD-R8-F11/F12:** both the pinned
declaration and the structural registry pass when the CharacterList
report is carried by M5 instead of its required M2 lane, and when the M8
actor is configured Assumed while its reproduction record claims Known.
The registry totals remain accurate; verdict readiness is suspended.
**The F11/F12 repairs landed 2026-08-15** (`23ecbd0d`, `f3865ef9`): the
declaration requires the named (M2, endpoint) saturation pairs and
binds each M8 profile check to its endpoint, and the driver's engine
construction and record provenance flow from one structurally pinned
lane source (`mod lane`). Both audit bypasses were pinned as negative
tests, and both authorities reran green over the repaired guards
(pinned declaration; 4,096-case declared run in 306.83 s; obligations
6/6 at unchanged 124/110 totals), and all five mutation signatures —
the preserved F5/F9 pins and the new F11/F12 refusals — are recorded
in `result-draft.md` §9. **The repeated re-close review (2026-08-15)
verified all ten repairs and reproduced the matrix and all five
signatures, but found SD-R8-F21/F22: the two authorities still agree
in two forged states** — the F12 split profile re-expressed outside
`mod lane` by post-construction mutation of the record's `pub` field,
and the (M2, CharacterList) pair satisfied by a post-judge cloned
report with zero CharacterList wire traffic. Verdict readiness stays
suspended pending the F21/F22 repairs, a fresh packet, re-review, and
the repeated final audit.
**The F21/F22 repairs landed 2026-08-15** (`a09ef5ed`) under Tom's
hybrid decision. Rust privacy across the library/integration-test crate
boundary now seals `ReproductionRecord`, judge-only `RunReport`, and
the mock-only observation/state-change carriage. The exact F21 record
overwrite, F22 post-judge clone/endpoint overwrite, and direct report
construction are compile-fail doctests; the existing lexical lane pin
is retained only as a belt. `seal_evidence` refuses incomplete
handoff/observation carriage, snapshots both vectors atomically, and
prevents later mock traffic or logged state changes. Assertion coverage
and assertion-passed remain named test-authorship trust surfaces in the
packet and `g5-scenario-assertions`, with the per-scenario
falsifiability guards and scale-shape test cited. Registry totals remain
124/110 and both verdict fills remain suspended: this repair session
closes nothing and awaits the repeated independent re-close review,
then the repeated final audit.
**The repeated re-close review over the F21/F22 sealing range (2026-08-15)
re-verified the whole sealing repair — the five compile-fail doctests
each fail with their exact annotated privacy code (legal equivalents and
the reviewer's original forgery shapes checked), `seal_evidence` changes
who constructs evidence but not which evidence a green run judges, the
public surface is read-only, and the full offline matrix, both
authorities, and all five preserved mutation signatures reproduced — but
found SD-R8-F23 (medium): the evidence-carriage seal is stated as
absolute construction privacy ("Only `seal_evidence` can construct";
"a test cannot filter either vector and rebuild a judge input"), yet
Rust privacy does not bind the `mock` module's in-crate descendants.
`mock::model` (a descendant with a live `#[cfg(test)]` module) can write
a forged `MockEvidence` struct literal directly. It is not a verdict
bypass — the verdict path runs only across the integration-test crate
boundary where the seal holds — but the boundary is unnamed (the `b13`
note mis-states it), which charge §3-addendum pre-committed as a finding.
Verdict readiness stays suspended pending the F23 repair, a fresh packet,
re-review, and the repeated final audit.
**The F23 repair landed 2026-08-15** (`1b483191`): the seal claim is
scoped to the library/integration-test crate boundary and the `mock`
module's in-crate descendants are named as a residual trust surface
in the doc comment, the `b13-observation-log` note/`must_assert`, and
the packet confession, with the compensating controls stated (the
verdict path resides entirely across the crate boundary; no in-crate
test reaches `judge` or `declare`); a lexical belt pin — detection,
not a binding — makes in-crate forgery loud, and the post-commit
mutation demonstrated both the forgery compiling in `mock::model` and
the pin catching it. No enforcement mechanism changed; both
authorities reran green (pinned + 4,096-case declaration; obligations
6/6 at 124/110) and all five preserved mutation signatures and
compile-fail doctests hold. Both verdict fills remain suspended
awaiting the repeated re-close review and the repeated final audit.
`obligation-map.md`
is the superseded prose ancestor; read its §8 for the audit's
discrepancy analysis (dated at `e2034807`), never for current
coverage.

## 2. Slice and review state

- **Open: scenario-driver slice** (M1–M13 driver/judge
  integration; closed with SD-R8 on 2026-08-15, then **reopened
  the same day by the external no-context audit of the closure**
  — the round history follows).
  **Round five is closed** (SD-R5, 2026-08-14): an independent
  re-review validated all twenty-four repairs — SD-R5-F2..F15 and
  RE-1..RE-9 — against code, tests, and docs, reproduced the full
  verification matrix, and found no new findings; dispositions live
  in `result-draft.md` §9's 2026-08-14 repair entries and the
  re-review closure entry. No verdict slot was filled — every
  driver/focused report remains `ContractCoverage::Fragment`. The
  slice itself stays open on the residual work in §5;
  `scenario-driver-handoff.md` remains the live four-part packet for
  that work.
  **Round six is closed** (SD-R6, 2026-08-14): an independent
  review validated the residual-items packet — §5 items 1–3, the M1
  generated-φ residue sweep, Ballot G's forced M9 race, and M11a's
  near-ceiling sweep — against the contract docs, code, and
  registry; reproduced the full verification matrix and all four
  mutation checks (exact claimed signatures); and found no
  findings. The closure entry is in `result-draft.md` §9. No
  verdict slot was filled.
  **Round seven is closed** (SD-R7, 2026-08-15): an independent
  review validated the §7.4 replacement-gate implementation against
  the ratified spec clause by clause, reproduced the full
  verification matrix and all seven mutation signatures, confirmed
  the flagged weaken-mutation deviation and adjudicated it a spec
  erratum (corrected by a dated marker in the spec's §3), and
  accepted the precondition-5 doc finding as recorded.
  Reopened the same day by the external no-context audit (Tom's
  independence check on the spec-author/reviewer conflict; two
  findings), then **re-closed the same day on Tom's recorded
  adjudications**: SD-R7-F1 discharged (alternate kill signature
  accepted, erratum ratified, disclosed silences ratified) and
  SD-R7-F2 fixed and verified (C1 failures now carry full component
  witnesses including the recorded count). The audit's verdict and
  both closure entries are in `result-draft.md` §9. The slice stays
  open on §5 item 5 only — the full-contract run;
  `scenario-driver-handoff.md` remains the live packet file for it.
  **Round eight closed (SD-R8, 2026-08-15) and was reopened the
  same day.** An independent review validated the
  full-contract packet across its whole range (the declaration
  machinery, the SD-R8-F3 padded-history reconciliation fix in the
  core — walked against the pessimism invariant — the scale pins,
  and the post-adjudication padded-safe G3 oracle), independently
  re-ran both declared runs (the 4,096-case run's scale is pinned
  in code; all green), verified the registry as the agreeing second
  authority, and reproduced all four mutation signatures verbatim,
  including resolving the pinned-refusal determinism question
  (`ReportNotVerdictEligible { scenario: M2 }` exactly). No
  findings against the implementation. Closure entry in
  `result-draft.md` §9.
  **Reopened 2026-08-15 by the external no-context audit** (the
  closure entry's own recommended audit, per the SD-R7
  precedent): five findings, **SD-R8-F4..F8**, every one
  confirmed by the processing session against code and docs —
  including reproducing F4's experiment — before minting; the
  audit entry is in `result-draft.md` §9. F4 (high): the
  full-contract declaration accepts a missing M8 Known lane
  while the registry verifier (structural by design) also passes
  — the two authorities can agree in exactly the state the
  two-authorities rule calls impossible. F5 (high): the
  unconditional verdict's "four OAuth policies" claim exceeds
  the evidence — only the stash-list and stash policies are
  exercised; the character policies never are. F6 (high): the
  charter's end-of-spike deliverables (hoist to `redesign`,
  register row, CN1–CN6 transcription, reusable-artifact record)
  are neither done nor renounced. F7 (medium): the scope ledger
  omits U5 and carries no explicit O-series statement. F8 (low):
  this file's own §4 self-contradiction, discharged by this
  flip. The audit affirms as standing: the `cc448b79`
  padded-history fix, both declared runs, and the registry
  totals. **Both verdict fills are suspended** (dated marker in
  `result-draft.md` §1); the slice and the spike are open again.
  `scenario-driver-handoff.md` returns to being the live packet
  file. The padded-history production fix and Tom's F2
  adjudication are unaffected.
  **The 2026-08-15 SD-R8 re-close review did not close the round.**
  It reproduced the committed matrix and both advertised F4/F5
  refusal signatures, but found **SD-R8-F9** (high): the new endpoint
  declaration guard trusts `ReproductionRecord.endpoint` while the
  judge's reproduction seam validates only seed and phase. In the
  review mutation, the CharacterList wire lane was replaced by a
  second Character lane while the reproduction record still claimed
  CharacterList; both the pinned declaration and the structural
  registry verifier passed. The two authorities can therefore agree
  without one of the four claimed OAuth policies being exercised.
  It also found **SD-R8-F10** (medium): the live four-part hand-off was
  never updated for the reopened F4/F5 range and still declared the
  pre-audit close with stale 123/109 totals. Per `slice-review.md` §2,
  that missing packet is itself a finding. The spike-side migration
  package and the `rate-limit-core-ground-truth` diff had no additional
  re-close findings. The verdict fills are suspended again pending the
  F9 repair, authority reruns, a complete reopened-range hand-off, and
  re-review. Review entry in `result-draft.md` §9.
  **Repeated independent re-close review passed 2026-08-15 with no
  new findings; SD-R8 and the scenario-driver slice re-close.** The
  reviewer accepted the complete F4/F5/F9 reopened-range hand-off,
  verified the endpoint binding and both corrected M9 labels against
  the wire, reran the debug/release/4,096-property matrix, the pinned
  and explicit 4,096-case declarations, obligations, exhaustive
  replay, clippy, fmt, and sanitizer, and reproduced the exact F4/F5/F9
  refusal signatures. Both verdict fills are restored. The unchanged
  migration diffs retain the prior re-close review's no-finding result.
  Closure entry in `result-draft.md` §9; the hand-off is historical
  again. The spike remains open only on the F6 delivery gate.
  **The F6 final external audit ran 2026-08-15 and reopened SD-R8 with
  SD-R8-F11–F20; do not deliver.** F11/F12 are high declaration-chain
  bypasses reproduced against both authorities. F13–F18 are migration-
  package overclaims or incomplete carriage, and F19/F20 are defects in
  the N31/N32 transcriptions. The committed debug/release/4,096-property
  matrix, both declarations, obligations, exhaustive replay, clippy,
  fmt, sanitizer, and diff checks all remained green; the audit's point
  is that green machinery does not establish the claims as written.
  Both verdict fills are suspended, the scenario-driver hand-off is live
  again, and delivery waits on repair, re-close, and a repeated final
  audit. Full evidence and proposed dispositions are in
  `result-draft.md` §9.
  **The F11–F20 repair session completed 2026-08-15**: all ten findings
  repaired per their §9 dispositions and Tom's F14/F16 decisions — the
  F11/F12 authority repairs (§1), the F13–F18 package and carriage
  corrections across the record, the topic, `scenarios.md` §7.3, and
  the migration-package charge, and the F19/F20 ground-truth
  corrections (`3088d6e4` on `rate-limit-core-ground-truth`). The
  fresh four-part packet is presented in `scenario-driver-handoff.md`;
  the repair session closed nothing. SD-R8 remains open awaiting the
  repeated independent re-close review, then the repeated
  `final-audit-charge.md` audit over the repaired tree and both
  migration diffs.
  **The repeated re-close review ran 2026-08-15 and did not close the
  round: SD-R8-F21/F22.** It verified all ten F11–F20 repairs item by
  item (wording repairs re-derived from evidence; F19 against B3's
  ratified conventions; F20 against the actor source), reproduced the
  full offline matrix, both authorities, and all five mutation
  signatures exactly — and then falsified the repair's central claim:
  the F12 split-profile state is representable outside `mod lane`
  (post-construction field mutation on the `Copy` record; F21, high),
  and the F11 pair requirement is forgeable at declaration time by a
  post-judge cloned report while zero CharacterList wire traffic
  exists (F22, high). In both experiments the pinned declaration and
  obligations passed. Three further unbound run-owned labels are
  enumerated in the review entry (assertion coverage, assertion-passed
  carriage, observation-vector carriage). Findings and proposed
  dispositions in `result-draft.md` §9; the hand-off is flipped back
  to owing an F21/F22 repair-range packet. Both verdict fills stay
  suspended.
  **The F21/F22 repair session completed 2026-08-15** at `a09ef5ed`.
  The fresh four-part packet is presented in
  `scenario-driver-handoff.md`; compile-time privacy, not a lexical or
  judge-time substitute, is the enforcement boundary. Tom's two named
  assertion trust surfaces and their compensating controls are carried
  in both packet and registry note. The session closes nothing: SD-R8
  remains open awaiting repeated independent re-close review and then
  the repeated final audit.
  **The repeated re-close review over the sealing range ran 2026-08-15
  and did not close the round: SD-R8-F23 (medium).** The reviewer
  re-verified everything the sealing commits touched and ran the §3
  fifth-generation hunt and the §3-addendum in full: the five
  compile-fail doctests each fail with their exact annotated privacy
  code (E0616/E0616/E0451/E0451/E0603, legal equivalents and the
  reviewer's original forgery shapes compiled to confirm), the seal's
  snapshot judges the same evidence the old accessors handed over, the
  sealed types' public surface is read-only, the non-boilerplate diff
  residue is construction control only, and the full offline matrix,
  both authorities (pinned + 4,096-case declaration; obligations 6/6),
  and all five preserved mutation signatures reproduced exactly. It then
  found that the evidence-carriage seal is stated as absolute
  construction privacy while Rust privacy does not bind the `mock`
  module's in-crate descendants: `mock::model` (a descendant carrying a
  live `#[cfg(test)]` module) can construct a forged `MockEvidence`
  struct literal directly, so the claims "Only `seal_evidence` can
  construct this type" and "a test cannot filter either vector and
  rebuild a judge input" (`b13-observation-log`) overstate the mechanism
  — the residual-trust concern of charge §3 bullet 3, and the exact
  boundary the §3-addendum required be named or minted. It is not a
  reproduced verdict bypass (the verdict-eligible report set is produced
  solely by `scenario_driver.rs` across the integration-test crate
  boundary, where the seal is absolute, and the conformance types have
  no in-crate descendant module), so it is medium, not high. Findings and
  proposed disposition in `result-draft.md` §9; the hand-off is flipped
  back to owing an F23 repair-range packet. Both verdict fills stay
  suspended.
  **The F23 repair session completed 2026-08-15** at `1b483191`. The
  fresh four-part packet is presented in `scenario-driver-handoff.md`;
  the seal claim is scoped honestly rather than re-armored — naming
  the in-crate descendant boundary in every carrying location, per
  Tom's standing "record honestly, don't pseudo-bind" pattern — and a
  lexical belt pin makes descendant construction loud without
  claiming a binding. The session closes nothing: SD-R8 remains open
  awaiting repeated independent re-close review and then the repeated
  final audit.
- Every other slice (core, bootstrap, mock, actor, clause registry,
  doc split) is closed; their hand-offs are historical records.
  (Doc split closed 2026-08-13, DS-R1 — `result-draft.md` §9.)

## 3. Open decisions (Tom)

None open. Tom decided the F21/F22 follow-on on 2026-08-15
(record in `result-draft.md` §9): **hybrid** — the observation
vector (and the state-change carriage, same class) is **bound** by
sealing: mock-module-only construction, so the evidence reaching
the judge is mock-authentic by construction; assertion coverage
and assertion-passed carriage are **recorded as named trust
surfaces** in the coverage confession and registry notes, with
their compensating controls cited (the falsifiability guards and
the scale-shape test) — they sit at the test-authorship boundary
no in-process binding can cross, and naming that boundary honestly
is the repair. The F21/F22 sealing repairs proceed per their §9
proposed dispositions; `f21-f22-repair-charge.md` collects the
assignment.

None from the final audit remain open. Tom decided both routed
findings on 2026-08-15 (records in `result-draft.md` §9):

- **F14 — corrected O5 carriage accepted**: all three locations
  (consumer topic, the ratified §1 carriage block, and
  `scenarios.md` §7.3's trigger note) say skew remains **untested**
  — the slice has no server-clock input and the `o5-date-skew`
  re-entry trigger has not fired. The O5 exclusion stands with an
  honest trigger; no skew evidence is manufactured.
- **F16 — narrow the claim**: the package delivers a **reusable
  foundation** (the independent counter engine plus the scenario
  contract), with the standalone HTTP delivery shim and a
  client-neutral driver explicitly named as future adapter work —
  matching `scenarios.md` §7.1's own "delivery-shim job" framing.
  The migration-package charge's cross-client acceptance wording is
  amended accordingly, by Tom. If ADR-0003 takes the rewrite path,
  the shim is built there against real requirements.

F11–F13, F15, and F17–F20 have evidence-preserving repair shapes
recorded in `result-draft.md` §9 and need no further contract
decision. (All ten repairs, F14/F16 included, landed 2026-08-15 —
§2, §5.)

Decided 2026-08-15: **SD-R8-F6 — complete via gated hoist; the
spike ends with a PR** (Tom; full record in `result-draft.md`
§9). Gate: open items close with the migration package drafted
alongside → SD-R8 re-close review → final external no-context
adversarial audit over the re-close packet **plus both migration
diffs** → delivery via PR `spike/rate-limit-core` → `redesign`
(charter's "never merged" snapshot line explicitly overridden by
Tom) plus a small CN1–CN6 docs PR into `master`
(`network-ground-truth.md` there is the citation authority; the
charter's "on `redesign`" is amended). **F6 completes at
landing, not readiness.**

Decided 2026-08-15: **SD-R8-F5 — extend, don't narrow** (Tom).
Coverage extends to the two remaining Known-profile policies
(`character-list-request-limit`, `character-request-limit`) so the
four-policy claim is earned; the declaration guard must grow to
require the new lanes (F4 principle). Implementation is round
work (§5); the decision record is in `result-draft.md` §9.

Closed earlier the same day: Tom adjudicated SD-R8-F2 on
2026-08-15: **G3's eligibility is
the padded-safe time** — the harness restates N13's
`hit + period + bucket` arithmetic over B13 mock-side facts;
"client-independent" means independent in derivation, not free of
padding; ε = 500 ms unchanged. The clarifying amendment is applied
in `scenarios.md` §6 (dated, Tom-attributed) and the decision is
recorded in `result-draft.md` §9. Consequence: the SD-R8 blocker is
an oracle-side fix only — no client, mock, or gate-machinery change
— and that fix plus the declared run have now landed (§2/§5).

Earlier decisions remain closed. Tom ratified the §7.4 replacement-
gate spec in full on 2026-08-15 — all five §6 asks, with the
retrospective-P1 and future-capture amendments folded in at
ratification.
`s7-4-replacement-gate.md` is now contract; the adoption marker and
the B3 convention amendment are applied in `scenarios.md`; the
ratification record is in `result-draft.md` §9. (Both earlier
2026-08-14 sign-offs remain recorded there as well.)

- §7.4 is adjudicated as a frozen-contract expectation error: a
  feedback-dependent captured dispatch trace is not required to stay
  safe when replayed open-loop under counterfactual server phases.
  B3, both fixtures, and the exhaustive counterexample diagnostic stay
  unchanged. The feedback-consistent replacement calibration gate is
  implemented and SD-R7-closed; every-phase safety remains in the
  closed-loop C1/M-series tests.
- Profile lanes are ratified: OAuth-bound scenario evidence uses
  `Known(5s/60s)`; explicit legacy evidence uses
  `Assumed(60s/60s)`; the shipped Assumed default remains
  structurally represented. Generic focused tests may use the shipped
  default only for demonstrably profile-invariant assertions.

## 4. Blocked

Delivery is blocked on re-closing SD-R8. The F21/F22 sealing repairs
are present and re-verified; the F23 repair is present and verified
(`1b483191`): the seal claim is scoped to the crate boundary, the
in-crate descendant boundary is named in all three carrying locations,
and the belt pin is in CI. What remains before either delivery PR:
another repeated independent re-close review over the F23 range, then
the repeated `final-audit-charge.md` audit over the repaired tree and
both migration diffs. Both verdict fills stay suspended until then. No
Tom decision is open: the executed disposition followed his standing
"record honestly, don't pseudo-bind" pattern (naming the boundary, not
adding a binding Rust privacy cannot provide against a module's own
descendants).

## 5. Next work

The 2026-08-13 ballot remains closed; items 1–4 below stand done
as recorded (SD-R6; SD-R7 re-close). Item 5's SD-R8 closure was
**reopened by the 2026-08-15 external audit**; the reopened work
list follows the historical items. What follows the spike — Tom's
reading of the verdicts and the feed into `docs/adr/0003` — stays
outside this file's scope, and is farther off than the closure
claimed: the spike is not concluded.

Reopened SD-R8 work (audit and re-close-review entries,
`result-draft.md` §9):

- ~~**Final-audit F11/F12 authority repairs**~~ — **done 2026-08-15**
  (`23ecbd0d`, `f3865ef9`): named (M2, endpoint) pairs plus
  endpoint-bound M8 profile checks in the declaration; one structurally
  pinned lane source for engine and record provenance; both bypasses
  negative-pinned; both authorities rerun green; all four hand-off
  parts updated (§1, §2, `result-draft.md` §9).
- ~~**Final-audit F13–F18 package repairs**~~ — **done 2026-08-15**
  (`58cfdb67`, `39a86163`): instrument claim restricted to the
  M-series, O5 corrected per Tom's F14 acceptance, registry wording
  corrected to records-plus-structural-verifier, reusable claim
  narrowed to a foundation per Tom's F16 decision (charge amendment
  applied with his attribution), external premises labeled with their
  provenance lanes, and G1–G6 with the finalized tolerances carried
  into the consumer evidence basis.
- ~~**Final-audit F19/F20 ground-truth repairs**~~ — **done
  2026-08-15** (`3088d6e4` on `rate-limit-core-ground-truth`): N31
  carries B3's half-open and exclusive-expiry conventions in the
  model-choice lane; N32 narrowed to front-only ordinary dispatch with
  deque-scanning probe writer selection; both mirrored in the topic
  and §4's CN6 row.
- ~~**F4 repair**~~ — **done 2026-08-15**: declare's profile
  guard keyed to M8, negative test pins the audit's exact state
  (accepted before, `MissingM8KnownLane` now), `m8-both-lanes`
  cites it; end-to-end mutation reproduced the audit experiment
  against the repaired guard and it is refused. Both authorities
  reran green (§9 changelog).
- ~~**F7 U5 repair**~~ — **done 2026-08-15**: U5 carried in
  `result-draft.md` §7 alongside U1–U4, per its own carriage
  mandate. The O-series half stays with Tom.
- ~~**F5 extension**~~ — **done 2026-08-15** (`7a2d49e5`): both
  character-policy lanes run at every coverage level and both φ;
  the declaration requires every N23 endpoint
  (`MissingEndpointLane`, negative test + end-to-end mutation);
  hand-derived G4 fingerprints matched on first run; registry
  minted `m2-character-policy-lanes`; both authorities green
  over the extended contract (§9 changelog).
- ~~**F7 O-series carriage**~~ — **ratified by Tom 2026-08-15**;
  part of both verdict statements; verdict fills restored by the
  repeated re-close review (§3, §9 changelog).
- ~~**F9 endpoint-provenance repair**~~ — **done 2026-08-15**
  (`3813c40a`): the judge binds `reproduction.endpoint` to every
  wire observation alongside seed and phase; the structural seam
  test pins the relabel; both authorities reran green under the
  binding; the review's own end-to-end mutation (Character wire
  lane retaining the CharacterList label, seed 809) now fails
  `ReproductionMismatch { id: 1 }` before declaration. Two real
  M9 mislabels (driver row; focused race fixture) were caught by
  the binding and corrected — the seam was live.
- ~~**F10 reopened-range hand-off**~~ — **done 2026-08-15**: all
  four `scenario-driver-handoff.md` parts carry dated
  reopened-range additions (F4/F5/F9 silences, seam/invariant
  walk, coverage confession at the current 124/110 totals,
  judgment calls) plus the packet's verification matrix and three
  mutation signatures; the status header presents the packet for
  the repeated independent re-close review.
- **Remaining, per the F6 gate (delivery, not evidence):**
  1. ~~Draft the migration package~~ — **done 2026-08-15** per
     `migration-package-charge.md`: topic doc `f0bbb92d`, register
     row `1177fa56`, §8 artifact record `82d7a434`, and AGENTS F6
     note `88d8266c` on this branch; CN1–CN6 transcribed as N27–N32
     in `c1d92417` on `rate-limit-core-ground-truth`, cut from
     `master`. Drafts only; no push or PR occurred.
  2. ~~**Repeated independent SD-R8 re-close review**~~ — **closed
     2026-08-15 with no new findings**; F9/F10 validated, both
     authorities and all three mutations reproduced, verdict fills
     restored (`result-draft.md` §9).
  3. ~~**Final external adversarial audit**~~ — **ran 2026-08-15 and
     reopened SD-R8 with F11–F20; package not deliverable.**
  4. Repair and independently re-close the reopened round, then repeat
     `final-audit-charge.md` over the repaired tree and both diffs.
     **F11–F20 repairs done 2026-08-15** (the three bullets above);
     the repeated re-close review ran the same day, verified them,
     and **found SD-R8-F21/F22 — the round stays open**. Now next:
     ~~the F21/F22 repairs and fresh four-part packet~~ — **done
     2026-08-15** (`a09ef5ed`; packet presented without closure); the
     repeated re-close review over the sealing range ran the same day,
     re-verified the repair, and **found SD-R8-F23 — the round stays
     open**. Then: ~~the F23 repair (scope the seal claim to the
     integration-test crate boundary and name the `mock` module's
     in-crate descendant modules as inside the seal) and fresh four-part
     packet~~ — **done 2026-08-15** (`1b483191`; packet presented
     without closure). Now next: another repeated independent re-close
     review over the F23 range, then the repeated final audit.
  5. The two delivery PRs (`spike/rate-limit-core` → `redesign`;
     CN docs → `master`). **F6 — and the spike — complete at
     landing.**

1. ~~**M1 generated-φ mock-side residue sweep**~~ — **done
   2026-08-14** (`tests/m1_residue_sweep.rs`; `m1-g1-sweep` Full —
   `result-draft.md` §9 changelog).
2. ~~**Ballot G** — the forced M9 phantom race at 14/15~~ — **done
   2026-08-14** (`transition_timing::m9_forced_phantom_race_at_saturation_recovers_per_m8`;
   `m9-recovery-survives-race`, `m9-race-exposure-attribution`, and
   `b12-scripted-delay` Full — `result-draft.md` §9 changelog).
3. ~~**M11a named binding evidence**~~ — **done 2026-08-14**
   (`tests/m11_ceiling_sweep.rs`; `m11-compliant-never-trips` Full —
   `result-draft.md` §9 changelog).
4. ~~The feedback-consistent §7.4 replacement calibration gate~~ —
   **done 2026-08-15**: spec ratified (`s7-4-replacement-gate.md`,
   contract), implemented green the same day (code `fdacd206`;
   `s7-4-replay-gate` Full; superseded open-loop test deleted after
   its residue-zero assert migrated), reviewed (SD-R7), reopened by
   the external audit, and **re-closed 2026-08-15 on Tom's recorded
   adjudications** (F1: alternate kill signature accepted and
   erratum ratified, silences ratified; F2: C1 failure messages
   fixed to carry the recorded count, verified). §9 changelog. The
   band-edge, exhaustive-enumeration, and 43/43 witness diagnostics
   remain load-bearing, retained unchanged.
5. **Full-contract run** — implemented and review-closed
   2026-08-15, then the closure was **reopened the same day by
   the external audit** (§2; reopened work list above). The
   historical record of the closure follows. The harness-only
   G3 correction implements Tom's F2 adjudication as independent
   `hit + period + bucket` arithmetic over B13 facts. The pinned φ=0
   declaration passed, followed by the full 4,096-case generated-
   phase run (298.84 s), drawing from the complete 60,000 ms strategy
   domain and covering both bucket profiles and full M6/M7/M8 queue
   shapes. It
   finishes the former fragment-scale-only
   clauses `m6-g1-post-announcement`,
   `m6-queue-drains-new-pace`, `m7-no-client-violation`,
   `m8-no-follow-on-violation`, `g1-zero-client-violations`,
   `g2-ceilings-never-tripped`, and `g3-over-delay-bounded`.
   The run declared `FullContract` and the independently edited
   registry records every owned clause Full under its structural
   verifier (semantic accuracy is prose-reviewed — SD-R8-F15
   wording); all seven flipped
   to Full, G1–G6 and both verdict lanes were filled, and the
   packet's review closed the round and the slice (§2, §9
   changelog) — the fills are now suspended per the audit
   reopening (§2).
