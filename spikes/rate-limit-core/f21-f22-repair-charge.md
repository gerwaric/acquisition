# Repair charge: SD-R8-F21/F22 — seal the judged artifacts

Status: **open — charge for the repair session** (drafted
2026-08-15 by the analyst session; dispositions are the
SD-R8-F21/F22 entry and Tom's hybrid decision in `result-draft.md`
§9; live state is `status.md`). Read the mandated documents in
AGENTS.md order first. Never contact a live service; commit before
reverting any mutation; present a fresh four-part packet; close
nothing.

## The enforcement mechanism, named this time

The class rule ("unrepresentable over detected") failed twice
because its mechanism was never specified. It is now: **Rust
privacy across the integration-test crate boundary.** Integration
tests are external crates; a field that is private in the library
crate cannot be written there at all. Every forgery from all four
generations (F4, F9, F11/F12, F21/F22) mutated or constructed
evidence data from test code — under sealing, those programs fail
to compile. A lexical or check-time guard is not an acceptable
substitute anywhere in this repair.

## 1. Seal the judged artifacts (F21, F22)

- `RunReport`: private fields, constructible only by `judge`
  (no public constructor, no public field access that permits
  mutation; read accessors only). `Clone`/`Copy` are acceptable —
  copying sealed data cannot alter it; what must be impossible is
  clone-then-modify.
- `ReproductionRecord`: private fields with construction restricted
  to the lane/conformance seam, so the record is immutable from
  construction through judging — F21's
  overwrite-after-`Lane::evidence` must not compile.
- `declare` consumes only sealed reports.
- Keep the existing lexical lane pin as belt; it is no longer the
  claim-bearer.

## 2. The hybrid binding (Tom's decision, §9)

- **Bind**: the observation vector and the state-change vector
  reach the judge only inside carriage types the mock module alone
  can construct (same sealing move). The judge's evidence inputs
  become mock-authentic by construction.
- **Record**: assertion coverage and assertion-passed carriage are
  named trust surfaces — in the coverage confession and in a
  registry note — with compensating controls cited (per-scenario
  falsifiability guards; the scale-shape test). Do not build a
  pseudo-binding for them; the §9 rationale is part of the record.

## 3. Pins and verification

- The reviewer's two forgeries become **compile-fail pins** in
  their exact shapes: F21's two-line record overwrite, and F22's
  post-judge clone with endpoint overwritten (plus a
  direct-construction attempt on `RunReport`). The X2 compile-fail
  pattern is the precedent.
- All previously pinned runtime refusal signatures (F5/F9/F11/F12)
  must still compile and still refuse with their recorded
  signatures — sealing must not weaken a single existing pin.
- Both authorities re-run green: pinned declaration, the 4,096-case
  declared run, obligations at current totals. Full offline matrix
  in the packet. Registry: citation/note updates only unless a
  clause is genuinely owed; the named-trust-surface note lands with
  its owning clause.

## 4. Exit

Fresh four-part hand-off (dated additions in all four sections,
including the named trust surfaces in the confession), then the
repeated independent re-close review re-runs
`re-close-review-charge.md` against the new range — its §3 hunt is
now the fifth-generation hunt, and the enumerate-unbound-labels
sweep must come back empty or fully named. Then the repeated
`final-audit-charge.md` audit, then delivery per the F6 gate.
