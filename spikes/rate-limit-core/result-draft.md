# Result draft: rate-limit-core spike (agenda item 5)

Status: accreting evidence container, created 2026-08-09 per the
sibling-docs convention in `design-brief.md`. This is the **skeleton
the spike fills as work happens** — implementation sessions record
evidence here the day it exists; the end-of-spike step is a hoist to
`docs/redesign/topics/` on `redesign` (plus the register-row update
and candidate-N-claim transcription), not a writing project. Scope
guard: this branch touches nothing on `redesign`; every transcription
noted below happens at hoist time.

Slots marked `⟨…⟩` are unfilled. A slot is filled by citing evidence
(a test run, a commit, a fixture), never by assertion.

N-numbers cite `docs/design/network-ground-truth.md`. D-numbers cite
`docs/design/network-redesign.md`. Scenario/gate/budget IDs (M/C/X/U,
G1–G6, B/O) cite `scenarios.md`. Design types cite `core-design.md`.

---

## §1. The question and its answer

**Register question** (`docs/redesign/README.md`): can a Rust client
demonstrably honor the N-claims in `network-ground-truth.md` under
burst load, as a single serialized gate?

**Definition carried to the register at hoist** (plan-review
addendum, finding 4): "single serialized gate" = one serialized
scheduling authority — the actor — with wire concurrency inherited
from D5's gate contract in full: in-flight cap 2, HEAD-exclusive
with writer preference, FIFO among ordinary waiters. Not literal
one-request serialization.

**Terminal condition** (addendum, finding 7): the spike completes
when offline conformance and this document are complete. Live
validation is a designed follow-up instrument, not a completion
gate; §6 holds any optional supplemental runs.

### Verdict

Two lanes, per the bucket-knowledge split (`scenarios.md` §1):

- **Unconditional** — the four OAuth policies, bucket resolution
  `Known(5s/60s)` (N12): ⟨verdict; requires G1–G6 green across all
  mock-judged scenarios and phase sweeps over these policies⟩
- **Conditional** — `backend-item-request-limit`, bucket resolution
  `Assumed(60s/60s)` (not provably pessimistic; N14/N21 give no
  upper bound): ⟨verdict, stated *with* its assumption; same gates,
  conditional lane⟩

G3 (ε = 500 ms) and G4 (1.05×) are draft numbers; they must be
finalized (`scenarios.md` §6 revisit rule) before either verdict
slot is filled.

## §2. How to read the evidence

Two orthogonal lane taxonomies apply:

- **Claim lanes** (what kind of knowledge a statement is):
  measured / estimated / inferred / external (URL + retrieval
  date, per the credential-custody precedent).
- **Test lanes** (which instrument judged it): mock-judged wire
  behavior / core-level property tests / fault-injection &
  structural / declared-untested.

Tests cite scenario IDs; scenarios cite N-numbers; this doc cites
both. Mock verdicts are **measured-against-model**; the §5 replay
calibration is what grounds the model in the observed lane. Every
failure anywhere must report its (seed, φ) pair (G6).

## §3. Conformance results

### Mock-judged wire scenarios

| ID | Scenario | Sweep | Gates exercised | Result | Evidence |
|---|---|---|---|---|---|
| M1 | Cold start with residue (flagship) | phase-swept | G1, G2, G6 | ⟨…⟩ | ⟨…⟩ |
| M2 | Clean cold-start saturation burst | phase-swept | G1–G4, G6 | ⟨…⟩ | ⟨…⟩ |
| M3 | Degraded HEAD | independent | G1, G2, G5 | ⟨…⟩ | ⟨…⟩ |
| M4 | Unexpected policy shape | independent | G1, G2, G5 | ⟨…⟩ | ⟨…⟩ |
| M5 | Policy rename mid-session | phase-swept | G1, G2, G6 | ⟨…⟩ | ⟨…⟩ |
| M6 | Policy shrink mid-flight | phase-swept | G1, G2, G6 | ⟨…⟩ | ⟨…⟩ |
| M7 | Phantom same-account hits | phase-swept | G1, G2, G6 | ⟨…⟩ | ⟨…⟩ |
| M8 | 429 recovery and escalation | phase-swept | G1, G2, G5, G6 | ⟨…⟩ | ⟨…⟩ |
| M9 | Phantom race at saturation | phase-swept | G1, G2, G5, G6 + characterization | ⟨…⟩ | ⟨headroom-zero evidence base: what nonzero headroom would have bought, per contention level⟩ |
| M10 | Agent-loop stress | phase-swept | G1, G2, G3, G6 | ⟨…⟩ | ⟨…⟩ |
| M11 | Layer-1 ceiling + Cloudflare terminal | independent | G2, G5 | ⟨…⟩ | ⟨…⟩ |
| M12 | 4xx-tripwire obligations | independent | G5 | ⟨…⟩ | ⟨…⟩ |
| M13 | Gate structure on the wire | independent | G2 + gate-definition assertions | ⟨…⟩ | ⟨…⟩ |

### Core-property tests

| ID | Property | Result | Evidence |
|---|---|---|---|
| C1 | Padding arithmetic safe over all φ | ⟨…⟩ | ⟨…; if skew-sensitivity appears here, O5 re-enters the mock budget⟩ |
| C2 | Header parsing / shape validation | ⟨…⟩ | ⟨…⟩ |
| C3 | Fuse trip logic | ⟨…⟩ | ⟨…⟩ |
| C4 | 4xx tripwire logic | ⟨…⟩ | ⟨…⟩ |
| C5 | Lifecycle invariants | ⟨…⟩ | ⟨…⟩ |

### Fault-injection and structural

| ID | Check | Result | Evidence |
|---|---|---|---|
| X1 | Fuse true-positive (pacing disabled) — the lane upgrade from declared-untested | ⟨…⟩ | ⟨…⟩ |
| X2 | Transport boundary: one HTTP client, private, no second send path | ⟨…⟩ | ⟨…⟩ |

### Gate summary

| Gate | Statement | Result | Evidence |
|---|---|---|---|
| G1 | Zero client-caused violations (incl. follow-on) | ⟨…⟩ | ⟨…⟩ |
| G2 | Neither B10 ceiling rule tripped, armed everywhere | ⟨…⟩ | ⟨…⟩ |
| G3 | Per-dispatch over-delay ≤ ε (final ε: ⟨…⟩) | ⟨…⟩ | ⟨…⟩ |
| G4 | M2 duration ≤ multiplier × padded minimum (final: ⟨…⟩) | ⟨…⟩ | ⟨…⟩ |
| G5 | All stimulus-scenario assertions | ⟨…⟩ | ⟨…⟩ |
| G6 | (seed, φ) reproducibility of every failure | ⟨…⟩ | ⟨…⟩ |

## §4. Candidate N-claims (transcribed to
`network-ground-truth.md` at hoist, cited by number there)

Accumulated during reconciliation and design; each survives or
falls on its cited source, independent of the spike verdict.

| # | Candidate claim | Claim lane | Source | Status |
|---|---|---|---|---|
| CN1 | Invalid-request (4xx) budget: too many 4xx responses in a short period restrict access; 429s double-dip (policy budget + invalid budget); threshold parameters undocumented (sibling of Q8) | DOC, Confirmed | developer docs, retrieved 2026-08-09 (verbatim quotes in the charter's 4xx entry) | awaiting transcription |
| CN2 | Cloudflare challenge-block signature: layer-1 blocks can present as 403 + `cf-mitigated: challenge` (not 1015), no rate-limit headers (extends N3) | external | community.cloudflare.com thread, retrieved 2026-08-09 (URL in the charter's bucketing entry) | awaiting transcription |
| CN3 | Recourse asymmetry: layer-1 blocks may be invisible to GGG support and unappealable for non-business Cloudflare users (informs Q7) | external | same thread as CN2 | awaiting transcription |
| CN4 | Trade-API rules carry three windows per rule — the RulePair shape is not universal; a non-pair policy is out-of-model, not impossible | external | pathofexile.com forum thread 3056323, retrieved 2026-08-09 (URL in the charter's bucketing entry) | awaiting transcription |
| CN5 | N11–N13 under-specify bucket quantization semantics (when a hit's age is measured / when it leaves the window); the spike's mock adopts the most-adversarial consistent reading — timestamp rounds up to bucket end, entry never quantized (N25 pins immediate 1:1 increment) | inferred (model choice); the gap itself is a doc finding | `scenarios.md` §7 B3, 2026-08-09 | awaiting transcription |

⟨New candidates minted during implementation land here the day
they appear — writing the mock exposing an N-claim ambiguity is a
finding in its own right (charter, step 1).⟩

## §5. Calibration: capture replay

Per `scenarios.md` §7.4 — the piece that grounds the model in the
observed lane:

- Fixture: ⟨sanitized July 18, 2026 capture (132 records), §4
  contract; sanitizer version, capture schema `v`⟩
- Gate — zero violations at every φ: ⟨result; any failing φ is a
  finding to adjudicate (mock bug vs. model exceeding N13's
  margin), recorded here, never tuned away⟩
- Diagnostic — saturation-state agreement (15/15, 30/30; N25/N26):
  ⟨observations; informs, does not fail⟩
- B12 delay re-anchor: ⟨capture median `sent→received` → final
  default; replaces the 50 ms placeholder⟩

## §6. Supplemental live evidence (optional by design)

Not a completion gate (terminal condition, §1). Any entry here ran
under the charter's logged run protocol — production-
indistinguishable pace, halt-on-first-violation, fixed safety
rails, Tom's explicit go-ahead per run.

- Gentle confirmation epilogue: ⟨if run: date, scope, outcome —
  never the evidence, a sanity check only⟩
- Validation-run ledger: ⟨dated, machine-readable entries; first
  designated target is U3's named hypothesis — legacy burst
  resolution = 5s; one 429 falsifies decisively, passes accumulate
  phase-swept confidence only⟩
- GGG correspondence: ⟨tier-assignment / legacy-resolution reply,
  when it arrives — may retire U3 without spending runs; annotates
  the charter's bucketing entry⟩

## §7. Declared-untested register

Carried verbatim from `scenarios.md` U-series so the hoisted doc
states its own blind spots: U1 remap triggers (reactive handling
M5 is the tested surface), U2 server-side 4xx restriction behavior
(obligations tested, M12), U3 legacy bucket resolution (conditional
lane; §6 instrument designed), U4 real layer-1 rules (deliberately
uncharacterized, N4 strategy; B10's numbers are inferred-lane).

## §8. Reusable artifact

⟨If the conformance suite works out: the mock (counter engine +
delivery shim, `scenarios.md` §7.1 layering) plus M1–M13 are the
acceptance tests any future core's limiter must pass — including,
via a standalone delivery shim, the C++ client. This outcome
outlives the spike branch; record what exists and where.⟩

## §9. Changelog

- 2026-08-09 — skeleton created (agenda item 5); all five design
  items closed; no code yet. Candidate register seeded with
  CN1–CN5.
