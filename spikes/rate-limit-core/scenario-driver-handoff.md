# Scenario-driver and judge-integration hand-off

Status: **round-one review findings F1–F6 fixed 2026-08-12; awaiting
re-review.** The actor-to-judge seam is implemented and exercised; this is
not a final M-series close because every row is a `ContractCoverage::
Fragment` and M10's reprioritization requirement has no actor command/API
(doc finding 11 in `result-draft.md`). The review round and its fixes are
recorded in the `result-draft.md` §9 entry dated 2026-08-12.

## 1. Silences taken

| Silence | Conservative reading | Next-call consequence |
|---|---|---|
| `scenarios.md` does not state a cardinality for a phase sweep. | Run every driver case at φ=0 and φ=59,999 (`SWEPT_PHASES_MS`), retaining `(seed, φ)` in evidence. | A new phase is data-only; a failure still carries its exact reproduction record. This is baseline seam coverage, not a claim of exhaustive phase coverage. |
| `scenarios.md` does not say whether a partial run may report its scenario's assertion. | It may not. Evidence declares `ContractCoverage::Fragment`, and `RunReport::verdict_eligible()` requires a pass *and* full coverage. | The next run that wants to fill a verdict slot must first raise that row to `FullContract`; a green fragment cannot be mistaken for a scenario result. |
| G3/G4 label 500ms and 1.05× as draft and require a §6 finalization before verdicts. | Exercise the judge at those draft numbers, but leave both verdict slots untouched. | The next evidence run can report draft-gate behavior only; it cannot promote either lane to a verdict. |
| §6 does not say whether G3's ε applies to an absolute schedule or to inter-dispatch spacing. | Keep the relative anchor and record the consequence rather than invent an absolute timeline (doc finding 12). | The next §6 finalization must name the anchor: measured band is that a 250→600ms floor regression stays green and 1,000ms trips G3. |
| M10 calls for reprioritization but the closed actor only has enqueue/cancel. | Do not fabricate a reorder operation; test pressure, cancellation, and dispatched caller drop only. | The next literal M10 attempt stops at doc finding 11 until an accepted command/API or scenario amendment exists. |

## 2. Seam map and invariant walk

- The driver owns no policy state: it submits only through `GateHandle`; mock
  handoffs, observations, and state changes feed `RunEvidence`.
- **No permanent wedge:** the M10 caller-drop path leaves the actor's active
  reservation to finish normally; the later pressure queue continues to
  drain. This is now *tested*, not asserted: M10 pins 14 surviving callers
  served and exactly 16 wire observations, so a drop that wedged the queue
  fails the row. (Round one found this claim resting on
  `observations.len() >= 2` with every caller outcome discarded.)
- **One send, one entry:** every ordinary send still originates from the
  actor's existing `try_reserve` path; the driver never constructs a token.
- **Pessimism direction:** post-dispatch caller drop detaches only the
  oneshot, so the wire response still reconciles.
- **Single scheduling authority:** expected G3 eligibility is independent
  script/mock arithmetic; it cannot issue a permit or alter actor timing.
- **Entry-point invariant:** HEAD/GET observations traverse the actor's
  existing probe/ordinary lanes; no driver shortcut calls a core response API.
- **Truthful notifications:** M11 observes the actor's published halt state;
  the driver emits no synthetic notifications.

## 3. Coverage confession

`tests/scenario_driver.rs` runs M1–M13 through the public actor and judge,
at φ=0 *and* φ=59,999 for every row, plus both bucket lanes for M8. It checks
mock-owned G1/G2/G3/G4/G5/G6 evidence and includes dispatched caller drop.

**Every row is a fragment.** Each declares `ContractCoverage::Fragment`, so
no report is `verdict_eligible()`. It deliberately does **not** claim the
still-missing scenario variants listed in the M rows of `result-draft.md`:
M1 probe-429, M2 saturation stall, M4 three-triplet, M5/M6 forced in-flight
transition exposure, M7 bursty debt, M8 escalation/malformed matrix, M9
forced race/headroom record, M10 hundreds/minutes/reprioritization, M11
ceiling sweep, M12 generic 4xx/matrix, and M13's full FIFO/writer-preference
assertion.

Gate teeth, measured rather than assumed:

- **G1/G2** are mock-owned (`organic_violation`, B10 ceiling) and cannot be
  laundered by the driver.
- **G4** is the strongest gate here: M2's 2,550ms padded minimum is
  independent integer arithmetic and passes with ~128ms of slack; it is what
  catches a pacing regression.
- **G5** now reflects each fragment's computed result — verified by forcing
  M12 false and observing `G5 failed: ["M12Tripwire"]`. It was a hard-coded
  `true` in `92db9f0b`.
- **G3 is the weakest**: its oracle re-anchors on the previous *observed*
  dispatch, so it bounds spacing, not schedule. Measured: a 250→600ms floor
  regression leaves it green; 1,000ms trips it. Doc finding 12.
- `independently_observable_ms` is **not exercised** — `unavoidable_exposure`
  is always `None` here, so the M9 exposure-attribution seam is untested by
  this target.

This target has no property tests. Its reachability guards assert that every
M ID produced a report and that no report is verdict-eligible; the judge
itself rejects empty observations/assertions and missing swept records.

## 4. Judgment calls

- Two adversarially separated phases are a useful initial driver baseline,
  not an invented definition of “phase-swept.”
- `ContractCoverage` extends the closed conformance slice's API rather than
  documenting the fragment/whole distinction in prose. A different session
  could reasonably have left the judge alone and recorded the caveat in
  `result-draft.md`; this makes the overclaim unrepresentable instead, which
  is the direction the AGENTS.md structural-guards rule points.
- The G3 oracle derives timing from the scenario script plus mock-owned prior
  handoffs/completions; it never reads actor state or calls scheduling code.
  It is nonetheless *relative*, and the round-one review measured the
  consequence — see the confession and doc finding 12. Keeping the relative
  anchor is a deliberate deferral to the §6 finalization, not a claim that it
  is sufficient.
- Caller drop belongs in this integration slice: it spans the actor lifecycle,
  mock handoff, and final judge evidence without changing the actor contract.
- G3/G4 finalization is a separate decision pass, because §6 makes it a
  prerequisite for verdicts rather than an implementation parameter.
