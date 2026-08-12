# Scenario-driver and judge-integration hand-off

Status: **awaiting review/direction, 2026-08-12**. The actor-to-judge seam is
implemented and exercised; this is not a final M-series close because M10's
reprioritization requirement has no actor command/API (finding 12 in
`result-draft.md`).

## 1. Silences taken

| Silence | Conservative reading | Next-call consequence |
|---|---|---|
| `scenarios.md` does not state a cardinality for a phase sweep. | Run each phase-swept driver case at φ=0 and φ=59,999, retaining `(seed, φ)` in evidence. | A new phase is data-only; a failure still carries its exact reproduction record. This is baseline seam coverage, not a claim of exhaustive phase coverage. |
| G3/G4 label 500ms and 1.05× as draft and require a §6 finalization before verdicts. | Exercise the judge at those draft numbers, but leave both verdict slots untouched. | The next evidence run can report draft-gate behavior only; it cannot promote either lane to a verdict. |
| M10 calls for reprioritization but the closed actor only has enqueue/cancel. | Do not fabricate a reorder operation; test pressure, cancellation, and dispatched caller drop only. | The next literal M10 attempt stops at finding 12 until an accepted command/API or scenario amendment exists. |

## 2. Seam map and invariant walk

- The driver owns no policy state: it submits only through `GateHandle`; mock
  handoffs, observations, and state changes feed `RunEvidence`.
- **No permanent wedge:** the M10 caller-drop path leaves the actor's active
  reservation to finish normally; the later pressure queue continues to
  drain.
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
with φ=0/59,999 for swept rows and both bucket lanes for M8. It checks mock
owned G1/G2/G3/G4/G5/G6 evidence and includes dispatched caller drop.

It deliberately does **not** claim the still-missing scenario variants listed
in the M rows of `result-draft.md`: M1 probe-429, M2 saturation stall, M4
three-triplet, M5/M6 forced in-flight transition exposure, M7 bursty debt,
M8 escalation/malformed matrix, M9 forced race/headroom record, M10
hundreds/minutes/reprioritization, M11 ceiling sweep, M12 generic 4xx/matrix,
and M13's full FIFO/writer-preference assertion. This target has no property
tests; its reachability guard asserts every M ID produced a report, and the
judge itself rejects empty observations/assertions and missing swept records.

## 4. Judgment calls

- Two adversarially separated phases are a useful initial driver baseline,
  not an invented definition of “phase-swept.”
- The G3 oracle derives timing from the scenario script plus mock-owned prior
  handoffs/completions; it never reads actor state or calls scheduling code.
- Caller drop belongs in this integration slice: it spans the actor lifecycle,
  mock handoff, and final judge evidence without changing the actor contract.
- G3/G4 finalization is a separate decision pass, because §6 makes it a
  prerequisite for verdicts rather than an implementation parameter.
