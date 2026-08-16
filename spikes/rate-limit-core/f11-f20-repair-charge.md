# Repair charge: final-audit findings SD-R8-F11–F20

Status: **open — charge for the repair session** (drafted
2026-08-15 by the analyst session after Tom's F14/F16 decisions;
authority for every disposition is `result-draft.md` §9's final-
audit entry and the decisions entry that follows it; live state is
`status.md`). The repair session reads the mandated documents in
AGENTS.md order first; this charge adds no authority — it only
collects the assignment.

## Scope

Repair all ten findings per their §9 dispositions, Tom's two
decisions (F14, F16 — §9), and the binding approach note (§9).
Never contact a live service. Commit before reverting any mutation.
Present a fresh four-part packet; do not close the round.

## 1. Authority repairs (F11, F12 — high; the F4→F9 class)

The class rule, now binding for these repairs: **prefer making the
forged state unrepresentable over detecting it** (§9 approach
note).

- **F12**: introduce a single run-configuration source from which
  *both* the actor's engine construction and
  `ReproductionRecord.client_buckets` flow. The split-profile state
  the audit forged must become unrepresentable in the driver, not
  merely detected. Add a structural pin that no second
  engine-construction path exists in the driver (the X2
  single-send-path pattern). Pin the audit's split-profile mutation
  end to end: it must now fail before declaration, with its
  signature recorded.
- **F11**: `FullContractRun::declare` requires the explicit
  `(M2, CharacterList)` and `(M2, Character)` scenario/endpoint
  pairs (and audit the other required lanes for the same
  pair-shape while there). Pin both audit bypasses as negative
  tests (the M2→M5 relabel with real CharacterList traffic, and
  the endpoint-only satisfaction). Add the honest registry note:
  scenario identity remains driver-owned, bound through each
  scenario's sole-decider assertion plus the pair requirement —
  record the residual trust surface explicitly.
- Rerun both authorities after each repair: pinned declaration,
  the 4,096-case declared run, `cargo test --locked --test
  obligations`.

## 2. Package and carriage repairs (F13, F15, F17, F18)

Per their §9 dispositions, in every affected location (consumer
topic, result-record §§1/7/8 carriage, status prose):

- **F13**: restrict the "in-process mock and simulated time"
  instrument claim to the M-series mock-judged wire evidence;
  preserve the true no-live-traffic scope for everything.
- **F15**: the registry *records* the prerequisites Full and its
  **structural** verifier passes; semantic accuracy is
  prose-reviewed (`registry-handoff.md` §3's limitation). Sweep
  every sentence that says or implies "verifies … Full"
  semantically.
- **F17**: separate the offline measured outcome from the external
  premises in both verdict paragraphs; label N12's bucket
  resolutions and N14/N21's no-upper-bound premise with their
  actual provenance lanes and retrieval context.
- **F18**: carry G1–G6 and the finalized tolerances (G3 ε = 500 ms,
  G4 1.05×) into the consumer evidence basis.

## 3. Tom-decided repairs (F14, F16 — decisions recorded in §9)

- **F14**: correct the O5 sentence in all three locations (topic,
  ratified §1 carriage, `scenarios.md` §7.3 trigger note) to say
  skew remains untested; the exclusion and its re-entry trigger
  stand. Tom's acceptance is recorded; cite it.
- **F16**: narrow the reusable-artifact claim to a reusable
  foundation (independent counter engine + scenario contract) with
  the standalone HTTP shim and client-neutral driver named as
  future adapter work; amend the migration-package charge's
  cross-client wording per Tom's decision; cite it.

## 4. Ground-truth transcription repairs (F19, F20)

On `rate-limit-core-ground-truth` (and mirrored in the consumer
topic):

- **F19**: N31 carries B3's ratified half-open-bucket and
  exclusive-expiry conventions explicitly, staying in the
  model-choice (not server-fact) lane.
- **F20**: N32 states that ordinary dispatch reads the queue front
  while probe writer selection scans the whole deque
  (`Actor::schedule` / `pending_probe()`); correct CN6's mirror.
  The single-deque conclusion stands once the mechanism is
  narrowed.

## 5. Exit

Fresh four-part hand-off in `scenario-driver-handoff.md` (dated
additions per section, current totals), full offline verification
matrix, mutation signatures recorded. Then, per the F6 gate in
`status.md` §3/§5: repeated independent re-close review → repeated
`final-audit-charge.md` audit over the repaired tree and both
migration diffs → the two delivery PRs on Tom's go. The repair
session closes nothing.
