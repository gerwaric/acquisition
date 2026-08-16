# Review charge: repeated SD-R8 re-close over the F11–F20 repairs

Status: **open — charge for the independent re-close reviewer**
(drafted 2026-08-15 by the analyst session that wrote
`f11-f20-repair-charge.md`; that authorship is a declared interest —
this charge encodes what that session knows about the defect class,
and the reviewer's job includes judging whether its prescribed
approach was *right*, not only whether it was implemented). Read the
mandated documents in AGENTS.md order first; `status.md` is live
authority; the packet under review is `scenario-driver-handoff.md`'s
current four parts. *(Range updated 2026-08-15: the F21/F22 sealing
repair extends the range — spike head `a4497304` (code `a09ef5ed`),
ground-truth head `3088d6e4` unchanged. §§1–2 below were verified
against the F11–F20 range by the previous execution of this charge;
re-verify only what the sealing commits touched, then run §3 and
the §3-addendum in full.)* *(Range updated again 2026-08-15: the F23
repair extends the range — code `1b483191`, ground-truth head
`3088d6e4` unchanged. The sealing range was verified in full by the
previous execution of this charge, which minted SD-R8-F23; re-verify
only what the F23 commits touched, then run the §3-addendum-2 below
in full.)* Close per `slice-review.md` §5 or
record findings and leave the round open. Never contact a live
service; commit before reverting any mutation you run.

## 1. Verify the repairs against the charge, item by item

`f11-f20-repair-charge.md` §§1–4 against the diffs and the §9 repair
entries. For every wording repair (F13–F18), use the final audit's
own method: re-derive each corrected sentence from the evidence
record — never accept that a sentence was softened; check the
softened sentence is *true*. F14/F16 must cite Tom's recorded
decisions. F19/F20 are on `rate-limit-core-ground-truth`: check N31
against B3's ratified conventions in `scenarios.md` §7.2 and N32
against `Actor::schedule`/`pending_probe()` in the actor source,
not against the topic's own summary.

## 2. Reproduce all five mutation signatures

The preserved F5/F9 pins and the three new refusals (F11 relabel →
`MissingScenarioEndpointLane`; F12 exact split → the structural pin;
F12 profile flip → `MissingM8KnownLane`), exact signatures per the
§9 repair entry. Then both authorities: pinned declaration, the
4,096-case declared run (scale is pinned in code), obligations.

## 3. Hunt the fourth generation — the part that matters most

Three generations of one class (F4: profile lane label; F9:
endpoint label; F11/F12: scenario label and profile-configuration
label) each survived the previous generation's repair. Do not
assume the third repair ended the class. Attack it:

- **The `mod lane` pin is load-bearing and possibly lexical.** If
  the structural pin is a source-text check (the X2 pattern),
  probe its blind spots: can an engine or record be constructed
  through a re-export, a helper added outside the module, a test
  building `RunReport` directly, or a second `spawn` call the
  lexical pattern misses? Write the bypass; if it passes both
  authorities, that is F21.
- **The pair requirement trusts the pair.** A report claiming
  `(M2, CharacterList)` whose wire evidence is not M2-shaped — is
  it caught by anything except the M2 assertion the label routes
  to? Check the judge's assertion-id routing actually forces the
  M2 facts for a report so labeled, and try one forgery the pins
  don't already cover.
- **The residual-trust registry note**: verify it exists, names the
  trust surface accurately, and does not claim more binding than
  the code performs.
- **Sweep for remaining run-owned labels** the declaration or
  judge consumes that are bound to nothing: any field of
  `ReproductionRecord` or `RunReport` that the judge does not
  cross-check against observations or construction is a candidate.
  List every one you find, even if you cannot forge a bypass —
  unlisted trust surfaces are how this class survives.

## 3-addendum. Sealing-specific attacks (added 2026-08-15 for the
F21/F22 repair range; from the repair *report* alone, unexamined)

- **Compile-fail doctests pass on any error.** For each of the
  five, verify the failure is the intended privacy/constructor
  error: change the forbidden line to a *legal* equivalent and
  confirm the snippet then compiles (proving the snippet isn't
  broken for an unrelated reason), or inspect the rustc error
  directly. A `compile_fail` green from a typo'd path is a
  fifth-generation bypass wearing a seal.
- **`seal_evidence` is new runtime semantics, not just
  visibility.** Its refusal rules and atomic snapshot define what
  the judge now sees. Check the snapshot point against what
  drivers previously handed the judge — the seal must change who
  can construct evidence, never which evidence a green run judges.
  Probe its refusal arms: are they reachable, tested, and do they
  fail closed?
- **The diff is ~750 lines; sealing should be mostly
  boilerplate.** Read the non-boilerplate residue of `a09ef5ed`
  for behavior changes hiding in the refactor — anything that
  alters judge/declare logic beyond construction control is
  finding material even if green.
- **Sweep the sealed types' public surface**: every `pub` item on
  `RunReport`, `ReproductionRecord`, `RunEvidence`, and the
  carriage types must be read-only; any setter, `pub` field,
  `pub(crate)` reachable from a path tests can use, serde/derive
  or `From`/`Into` that reconstructs, defeats the seal.
- **Name the in-crate boundary.** Rust privacy does not bind
  in-crate unit-test modules; they live inside the seal. The
  packet's trust-surface list must say so, or mint the finding.

## 3-addendum-2. F23-range checks (added 2026-08-15 for the F23
repair range)

*(Authorship note, added by the analyst session: unlike the
sections above, this addendum was written by the **F23 repair
session itself** — a declared interest. Treat it as a floor, not a
ceiling: it may encode the implementer's own blind spots as the
review's scope. Two sharpenings from the analyst: (1) re-derive the
compensating-control factual claims independently — that neither
`#[cfg(test)]` module in `src/` references `judge`, `declare`, or
any sealed type, and that `conformance` truly has no descendant
module — rather than accepting the repair's verification of them;
(2) anything the addendum's own framing would excuse, question.)*

- **The scoped claim must be exactly true.** Re-derive the corrected
  statements in the `MockEvidence` doc comment, the
  `b13-observation-log` note/`must_assert`, and the packet confession
  from the code: confirm Rust privacy binds exactly the
  library/integration-test crate boundary and nothing more, that the
  named descendant set is complete (`mock::model` today; any new
  child of `mock` joins it), and that no location claims more binding
  than the compiler performs. An overcorrected claim — understating
  what the seal does enforce — is also a finding.
- **The belt pin is detection and must be non-vacuous.** Verify the
  four pinned `mock/mod.rs` sites are what the pin says they are;
  reproduce the repair's recorded mutation (a forged `MockEvidence`
  literal in `mock::model`'s `#[cfg(test)]` module compiles and runs,
  demonstrating F23's fact in-tree, while the pin fails naming
  `model.rs`); probe evasions the pin's confession does not already
  name, and check nothing describes the pin as a binding.
- **Hunt the sixth generation.** The class is now "a claim about an
  enforcement mechanism that outruns the mechanism." Sweep the
  remaining enforcement claims — the compile-fail doctests' prose,
  the X2 and F12 pin comments, every trust-surface list — for any
  statement a compiler, test, or named reviewer step does not
  actually enforce. Unlisted or overstated surfaces are how this
  class has survived five generations.
- **Corrected `must_assert` fidelity.** Every `must_assert` the
  repair touched must state only what its cited test asserts.

## 4. The packet and the record

Four parts current at the repaired totals; §9 entries complete;
`status.md` accurate; verdict fills remain **suspended** — the
re-close restores them only if everything above holds, and the
repeated final audit still precedes delivery per the F6 gate. If
you close: three acts per `slice-review.md` §5, and state
explicitly that the repeated `final-audit-charge.md` audit is the
next gate. If you find anything: mint SD-R8-F21+ and leave the
round open.
