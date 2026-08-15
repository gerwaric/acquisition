# §7.4 replacement calibration gate — specification

Status: **adopted — ratified in full by Tom, 2026-08-15.** All five
§6 asks were ratified, with two amendments folded in at
ratification: the retrospective P1 anchor and the future-capture
re-derivation rule (both in §2.3/§4 below). The ratification and
its consequences are recorded in `result-draft.md` §9; the §7.4
adoption marker and the B3 convention amendment are applied in
`scenarios.md`. **This document is now contract for the gate's
implementation.**

Drafted 2026-08-14 by the SD-R6 review session at Tom's direction,
deliberately relaxing the session firewall (his call, same day).
Compensating controls: two independent adversarial fresh-context
reviews (§7) — the second re-derived the tables with its own replay
probe and every attack on the gate's conditions and tables failed —
plus probe measurements re-derivable from committed artifacts, and
the normal implementation-review round downstream. This document is
the "precisely specified feedback-consistent replacement gate" that
`s7-4-replay-gate` (minted per SD-R5-F11) names as its Partial
delta. Nothing here is contract until Tom adjudicates §6.

Authorities cited: `scenarios.md` §3/§4/§7.2 (B3, B14)/§7.4, Tom's
2026-08-14 adjudication (`result-draft.md` §9), the SD-R5-F2 band
table (`tests/capture_replay.rs`), and — a first for this spike —
the **production C++ client** in `src/ratelimit/`, which produced
the canonical capture and whose scheduling semantics are therefore
evidence about the capture, not spike code. The spike's Rust core is
*not* the captured client; nothing below may be argued from the Rust
core's reconciliation semantics (the first review's finding 6 caught
exactly that error in draft one).

## 1. What is being replaced, and why

The superseded gate replayed the canonical capture's fixed dispatch
schedule through the B3 model at **every** phase φ ∈ [0, 60,000) and
demanded zero violations. Tom adjudicated that expectation an error:
the captured schedule was shaped by the real server's feedback, so a
phase under which the feedback would have differed describes a run
the client would not have produced. The counterexample is 1,052
phases in 20 bands, every band initiating at 31/30 on
`stash-request-limit`'s sustained window. The recorded server states
at the twenty initiating replies are 6..15 (replies 110–119) and
21..30 (replies 125–134) — only reply 110 recorded `6:300:0`, and at
band twenty the margin is a single hit of B3's deliberate one-bucket
over-hold. The refutation of those phases is therefore *not* "the
model grossly contradicts the server"; it is precisely that **at
those alignments the mock's feedback would have differed on a
channel the client demonstrably reacts to** (a 429 in place of a
recorded 200).

The replacement keeps what can be kept of the gate's purpose, and is
explicit about what cannot:

- **Mock laxer than reality must fail — kept, strengthened.** The
  recorded server states are an external anchor; the gate asserts
  the model never understates them, at every quantified phase, at
  every component (previously never asserted anywhere).
- **Mock stricter than reality must fail — necessarily reduced.**
  Once the adjudication rejects the every-phase quantifier,
  over-strictness at a given phase can no longer *fail* a gate — it
  moves that phase out of the quantified set. What remains is (i)
  the absolute anchor that the consistent set is **non-empty** (a
  model too strict to explain the observed-compliant trace at any
  alignment fails outright), and (ii) the pin discipline: the
  refuted sets are frozen tables, so any model change that moves
  them fails loudly and may only be re-pinned with adjudicated
  provenance (the SD-R5-F13 rule). This is a real reduction in
  power, inherent to the adjudication, and is stated here so nobody
  later mistakes the gate for the stronger claim.

## 2. Feedback-consistency, grounded in the captured client

### 2.1 What the captured client actually reacts to

The capture was produced by the production C++ client. Its
scheduling layer (`src/ratelimit/`) determines dispatch timing from
server feedback as follows (constructs named so drift is checkable;
verified 2026-08-14, corrected by the second review):

- Per window, the state header is classified three ways against the
  limit (`RateLimitItem`): `state < limit` → OK, `state == limit` →
  BORDERLINE, `state > limit` → VIOLATION; a policy's status is the
  **maximum over its items**. **Sub-limit magnitudes are
  schedule-inert by code**: the only reads of `current_hits` are the
  three-way comparison and the below-BORDERLINE skip. On the OK
  path, dispatch adds only the feedback-independent
  `NORMAL_BUFFER_MSEC` (100 ms).
- At BORDERLINE, the wait is
  `max(request_time, received_time, reply_time) + period + bucket +
  TIMING_BUCKET_BUFFER_SECS (1 s)` over the event `history[n−1]`
  with `n = min(history.size(), max_hits)` — indexed by the
  **limit**, never by the observed state (which is what makes
  sub-limit magnitudes inert). `reply_time` is parsed from the
  server `Date` header; the client's **bucket** is 5 s for periods
  ≤ `INITIAL_VS_SUSTAINED_PERIOD_CUTOFF` (75 s), else 60 s.
- History events are appended only at reply receipt
  (`RecordLandedReply`); boot HEADs seed state via `Update` without
  pushing history. There is **no ledger-vs-state reconciliation
  channel** in this client (hunted for by both reviews; none
  exists).
- Two further dispatch-timing channels exist and are named so P1's
  enumeration is honest: the **gate** (`gate.cpp`: in-flight cap 2,
  250 ms spacing floor) times a third dispatch off a reply's
  *completion* — latency-mediated, so feedback-inert here only by
  the §2.4 stipulation that latencies are held to recorded values;
  and **`HoldUntil`/`m_earliest_send`** propagates a 429's hold to
  every manager sharing the policy — unexercised by this capture
  (all replies 200, HEADs 204; §3 preconditions) but part of the
  429 surface.

### 2.2 Layering: demand vs dispatch

The *demand* sequence (which requests the app enqueues, and when)
depends on reply bodies (a stash list names the tabs fetched next).
Bodies are outside the rate limiter (its code never reads them —
the §7.3 O3 scoping premise) and outside the capture (§4 strips
them). The gate's claim is therefore about **dispatch timing of a
fixed demand sequence**: in both worlds the demand is the recorded
one, and feedback-consistency concerns only the rate-limiting
layer's timing decisions. No claim is made about body-driven demand.

### 2.3 Premises

- **P1 (schedule-relevant feedback surface).** The captured client's
  dispatch timing depends on server feedback only through: (a) the
  per-window three-way state classification, aggregated to policy
  status by maximum; (b) response disposition (2xx/429/error),
  `Retry-After`, and the 429 hold-propagation channel; (c) the
  policy limit shape; (d) the server `Date` header, only via
  `max(request, received, Date)` in the BORDERLINE wait; (e)
  boot-HEAD state seeding; (f) reply *timing* via the in-flight
  gate — feedback-inert under the §2.4 latency stipulation, named
  here so the enumeration is exhaustive. Grounded in §2.1's code
  reading (twice adversarially checked), but ratified as
  *exhaustive* by Tom (the code's author) — the gate does not and
  cannot test this premise, and it is load-bearing. **P1 is
  anchored retrospectively** (amendment, ratification 2026-08-15):
  it describes the client *as of the capture date* (the 2026-08-14
  repository state that produced the fixture) — a frozen fact of
  the fixture's provenance. Later changes to `src/ratelimit/`
  cannot reopen it for this capture; P1 needs the captured client
  to be feedback-frugal, not correct, so field bugs in that client
  threaten nothing here unless they create a new feedback→timing
  channel.
- **P2 (single-actor capture).** No third-party traffic hit the
  account's counters during the capture. Supporting evidence — the
  recorded states never exceed the model's pure client-count at any
  phase — is weak on its own: the model's over-hold slack (25–228
  components at clean phases) could mask several foreign hits. The
  premise rests on Tom's knowledge of the capture conditions, not
  on the data.
- **P3 (common bucket epoch — inherited from B3).** The model's
  single scalar φ applies to both tiers as `φ mod bucket`, so the
  sweep explores only the diagonal of the (φ₅, φ₆₀) alignment
  space — the hypothesis that the server's 5 s and 60 s buckets
  share one epoch. This is B3's frozen instantiation, not a new
  choice, but §1's non-emptiness anchor quantifies over exactly
  that space, so it is a premise, stated for ratification alongside
  the others.

### 2.4 The consistency conditions and the equivalence argument

Replay as the existing machinery does: seed from the boot HEADs,
drive the 383 recorded `sent_ms` instants through the model, judge
each. A phase φ is **feedback-consistent** iff, at every counted
reply, with all 766 window components compared:

- **(C1) Disposition agreement:** no organic violation. Because the
  judge's `organic_violation` is exactly "some window's
  post-increment count exceeds its limit, or a restriction is
  active", C1 also discharges the VIOLATION arm of P1(a): at every
  counted reply, model ≤ limit on every component.
- **(C2) Pessimism:** no component understates the recorded state
  (model ≥ recorded).
- **(C3) No spurious borderline:** no component has model == limit
  where the server recorded state < limit.

Entailments, stated so no condition masquerades as independent:
given the §3 preconditions, **C1+C2 together** entail exact
model/recorded equality at every recorded-saturated component
(recorded = limit ≤ model ≤ limit) — the old 43/43 diagnostic is a
**corollary** of the gate, not a condition of it — and the same
conjunction entails the reverse borderline direction
(recorded == limit ⇒ model == limit). C3 is the only genuinely new
equivalence direction, and the halo measurement (§5) shows it is
binding.

One deliberate over-approximation, made explicit (second review,
finding 2): C3 is per-component, but the client's dispatch consults
the policy-aggregated status. Where another component of the same
policy was already recorded at its limit, a spurious borderline
changes *which* item's time wins the wait's `max`, not the
OK→BORDERLINE classification — and the second review measured that
**73% of the halo's edge witnesses are of that masked kind, and 54%
are pad-compatible with the recorded schedule outright**. C3 is
therefore *sufficient* for classification equivalence but not
*necessary* for a schedule change: the strict treatment excludes
some phases whose counterfactual schedule may well have matched the
recording. That is the conservative direction for a consistency
filter, and every halo band edge retains at least one genuinely
refuting witness (second review, measured: 0 of 56 edges are
masked-only) — but the honest statement is that the halo is
excluded because **P1(a) equivalence fails there**, not because the
recorded next send disproves padding. Draft one's claim to the
latter is withdrawn.

At a feedback-consistent φ, the mock's feedback is equivalent to the
recorded feedback **over the entire P1 surface**: dispositions
identical (C1); every per-window classification identical (C1 for
VIOLATION, C2+entailments and C3 for the borderline boundary),
hence every aggregated policy status identical; limits identical
(precondition); `Date` schedule-inert in both worlds because it
never exceeds `received` in the fixture (measured precondition, §3)
and never exceeds it in the mock by B14 zero-skew construction;
boot seeding identical (all-zero states, precondition); and
response latencies are not server-chosen feedback — they are held
to the recorded values in the counterfactual (B12's deterministic
scripted-delay mechanism exists for exactly this), which is the
stipulation that makes P1(f) inert. "Indistinguishable" here means
*made identical wherever the server could not have chosen
otherwise*. The conditions are checked at **every** reply, which is
what licenses the induction on reply index: given identical
feedback through reply k, the client's k+1-th dispatch decision has
identical inputs, so the recorded schedule is a run the client
could have produced against the mock at φ — the replay is a genuine
closed-loop trace, not a counterfactual.

Conversely, an inconsistent phase is refuted by recorded evidence in
one of two ways, and the gate distinguishes them:

- **Disposition-refuted** (the 20 SD-R5-F2 bands, 1,052 phases): the
  mock would have 429'd a request the server served.
- **Classification-refuted** (the borderline halo, measured §5: 28
  bands, 29,347 phases): the mock's state header would have crossed
  the one boundary the client's code reads.

Sub-limit overstatement (model above recorded but below limit)
occurs at every phase (§5), is schedule-inert under P1, and refutes
nothing — draft one's contrary "equivalently" clause is withdrawn.

**No bit-exact determinism is claimed**, and closed-loop safety
under arbitrary phases remains where it lives: C1 and the M-series.
This gate calibrates the mock's counter model against the observed
lane; it is never client-safety evidence.

## 3. The gate, precisely

Tests live beside the existing replay tests, reuse the same
loader/seeding/judgment path, and touch no live service. The
canonical wired fixture is the sole input; the supplemental VPN
fixture's role is unchanged.

**Preconditions — capture integrity, asserted phase-independently
(several currently live only inside the superseded test or nowhere
and must be (re)asserted here):**

1. 383 counted replies, every status 200; 4 boot HEADs, status 204,
   one per endpoint.
2. **Boot residue is zero** — currently asserted only by the
   superseded test; §4's deletion is contingent on this migration.
3. Every recorded state component ≤ its limit; every recorded
   `restriction_active_seconds` = 0.
4. Exactly 43 recorded-saturated components (recounted, not a bare
   literal), including at least one `15/15` and one `30/30`.
5. **RulePair ordering:** each policy's first limit period is
   strictly shorter than its second (pins the positional zip
   against the [5 s, 60 s] buckets). Note the deliberate scope: the
   *client's* bucket rule is a 75 s period cutoff, and for
   `stash-list-request-limit` both periods sit below it — the
   client pads that policy's 60 s window with a 5 s bucket (66 s
   total) while the mock models it with a 60 s bucket. This
   mock-more-adversarial-than-client divergence is latent in this
   capture (no V or halo witness lands on that policy — second
   review, measured) and is recorded as a **doc finding at
   implementation** rather than silently absorbed.
6. **Limit stability:** one distinct limit string per policy across
   the whole capture.
7. **`Date` bound:** every record has `date_ms ≤ received_ms`
   (measured fixture max −8 ms; what makes P1(d) inert).
8. Index correspondence is positional throughout (model
   `windows[i]` ↔ recorded `states[i]` ↔ `limits[i]`); the loader's
   period-equality check plus (5) pins it.
9. Layer-1: no arrival trips a B10 ceiling. Phase-*independent*
   (the rolling ceilings never read φ), asserted once, kept as a
   regression check, deliberately not part of the per-φ
   consistency definition.

**Tables.** The violating set **V** stays `VIOLATING_BANDS`
(SD-R5-F2, retained untouched). A new frozen table **HALO** pins the
classification-refuted bands (measured §5: 28 bands, 29,347
phases). The consistent set **Φ\*** is derived **arithmetically** as
the complement of V ∪ HALO over [0, 60,000) — 9 bands, 29,601
phases; no third hand-maintained table. The partition
29,601 + 29,347 + 1,052 = 60,000 is asserted. Both tables are pins:
a model or fixture change that moves either fails loudly and may be
re-pinned only with adjudicated provenance (§1's stated limit on
what this buys).

**Pinned quantization conventions** (2026-08-15 blind audit, §5/§7):
B3's prose underdetermines two arithmetic choices the tables depend
on, and the implementation must pin both explicitly: **half-open
buckets** (an arrival exactly on a grid point takes the full
following bucket) and **exclusive expiry** (a hit whose adversarial
expiry equals an arrival instant is not counted at that arrival) —
the readings the mock's `CounterModel` implements. The band edges
are convention-sensitive by construction: the audited first halo
edge (φ=2,298 vs 2,297) is decided by exactly the expiry
convention's 1 ms. This underdetermination was recorded as a doc
finding (2026-08-15 §9 entry) and **resolved at contract level at
ratification**: Tom's B3 amendment in `scenarios.md` pins both
readings. N13 safety is indifferent to the choice — the
client's 1 s buffer dwarfs the 1 ms slop — so this is a
model-definition precision issue, not a safety one.

**Active gate test** (ordinary CI; 98 phase replays — 18 consistent
edges + 24 stride interiors + 56 halo edges — measured well under
0.1 s release; state the debug time when it lands):

1. Assert the preconditions and the partition.
2. For each of the 9 consistent bands: at both edges and at a
   fixed-stride interior sweep (stride 991 ms), assert C1, C2, C3,
   the 766-component count (non-vacuity anchor), and **at least one
   component strictly over the recorded state** (the anti-echo
   anchor: a mock that returns the recording verbatim must fail).
   The stride's honest rationale: it spreads interior samples
   across 5 s bucket residues instead of repeating one (the
   midpoint mistake), as defence-in-depth against table drift — it
   is a smoke sample, two narrow bands get no interior point, and
   **interior coverage belongs to the exhaustive companion**, which
   is why the companion is review-mandatory below.
3. For each of the 28 HALO bands: at both edges, assert the replay
   is violation-free but produces ≥ 1 spurious-borderline
   component. Shared boundaries with consistent bands are thereby
   pinned n/n+1 from both sides.
4. Disposition-refuted edges are **owned by the retained band-edge
   test**, not re-asserted here (no duplicate decider).
5. Failure messages carry φ, reply index, policy, window, model and
   recorded counts.

**Exhaustive companion** (`#[ignore]`, ~7 s release): every
φ ∈ [0, 60,000) classifies as exactly one of consistent /
classification-refuted / disposition-refuted, matching the tables;
and the strict-overstatement envelope over Φ\* is pinned exactly
(measured min 25, max 57 of 766 — re-derived at implementation).
**A green companion run is part of the slice's review evidence**,
exactly as the enumeration was for SD-R5-F2.

**Mutation checks** (run and reverted at review, signatures
recorded):

- Shift any **HALO or V** band edge by ±1 (Φ\* is derived and has no
  independent edge to shift) → partition or edge-classification
  failure.
- Weaken the model (expire hits one bucket early) → C2
  understatement failure at a consistent phase.

  *[Erratum, 2026-08-15 (SD-R7 review, validated by reproduction):
  the predicted C2 signature does not occur on this fixture — under
  that exact weakening there are zero understatements at any of the
  60,000 phases, because the weakened model collapses to a perfect
  766-component echo of the recorded server at φ=0. The mutation is
  killed — the contractual point — by the anti-echo anchor at φ=0
  and by 30,399 misclassified phases in the companion. The spec
  author's prediction was wrong; the implementation recorded the
  measured signature rather than tuning asserts to force the
  predicted one, which is the correct behavior. Incidentally, the
  φ=0 echo is an empirical observation that the real server's
  quantization at its actual alignment matches the non-adversarial
  floor-rounding model exactly across this capture — noted as
  context only, claiming nothing.]*

  *[SD-R7-F1 (external audit, 2026-08-15): the erratum above was
  applied by the SD-R7 reviewer — the spec's author — without a
  recorded Tom adjudication, which ratified contract text requires.
  Its measurement is triple-verified; its ratification is pending —
  `status.md` §3.]*
- Strengthen the model (hold hits one bucket longer) → a consistent
  edge produces a violation or spurious borderline.
- Corrupt one recorded state in memory after load → precondition or
  C2/C3 failure.
- **Echo mutation**: replace the model's judgment with the recorded
  state verbatim → the anti-echo anchor fails at every sampled
  phase.

**Discharge:** the green run flips `s7-4-replay-gate` to Full,
citing the active gate (companion and the retained φ=0 pin as
supporting). No verdict slot is filled; the seven fragment-scale
clauses still await the declared full-contract run.

## 4. Dispositions of the existing pieces

- **The superseded open-loop every-phase test is deleted**, *after*
  precondition 2 migrates its residue-zero assertion. Its
  counterexample is preserved by the band-edge test and
  enumeration; its every-phase assertion is the adjudicated error;
  its swept saturation diagnostic is subsumed by the gate's
  entailed-corollary form at every consistent phase. Provenance in
  the commit body and §9 changelog.
- **The φ=0 43/43 pin, the band-edge test, and the enumeration are
  retained unchanged.** φ=0 is the first consistent band's lower
  edge, so the gate covers it independently; redundant, not wrong.
- **`sent_ms` stays the replay's arrival convention**, and the
  justification is the stated-approximation argument alone: the
  real server counted at receipt (`sent` + one-way transit; the
  fixture's `sent→received` spans 47–241 ms *including service
  time*, so one-way transit is smaller still), making the model's
  hit instants a bounded per-hit approximation whose empirical
  consequences are exactly what §5 measures under this convention.
  Draft one's claim that alternative conventions were "measured and
  change nothing qualitatively" is withdrawn as unsupported at
  sweep scale (second review, finding 1); the only recorded
  alternative-convention data is §5's two-phase `received_ms`
  witness, and it is labeled as exactly that. If the implementation
  ever revisits the convention, that is a model change and moves
  the pinned tables — with provenance, like any other.
- **`scenarios.md` §7.4 amendment** (**applied at ratification,
  2026-08-15** — marker text as adopted, describing what runs): *"The
  replacement calibration gate is specified in
  `s7-4-replacement-gate.md` (ratified in full 2026-08-15): the
  capture refutes
  phases two ways — the SD-R5-F2 bands by disposition, a pinned
  borderline halo by state classification — and on the remaining
  consistent bands the gate asserts disposition agreement,
  pessimism, and no spurious borderline at every band edge and at a
  stride interior sample, with an ignored exhaustive companion over
  all 60,000 phases required in review evidence. Saturation
  agreement at consistent phases is an entailed corollary; the
  diagnostic bullet below remains correct for unknown-φ matching."*
- **Registry:** `s7-4-replay-gate` per §3's discharge line, and its
  stale `scenarios.md:820` anchor is corrected while touched;
  `b12-scripted-delay`'s note updates its cross-reference; no new
  clause is minted. The precondition-5 divergence enters
  `result-draft.md` as a doc finding at implementation.
- **Future captures** (amendment, ratification 2026-08-15): P1–P3
  and C3 are properties of *this capture's client*. A future
  capture from a different client — including the spike's Rust
  actor once it becomes the reference implementation — re-derives
  its own P1 and its own consistency conditions before joining
  §7.4. The Rust client's surface includes ledger reconciliation,
  so its conditions will differ from C1–C3; the §5 ledger-floor
  measurement is a preview of that analysis, not a substitute for
  it.
- **Out of scope:** closed-loop every-phase client safety (C1 and
  the M-series own it); the full-contract run; the supplemental
  fixture; body-driven demand (§2.2); any live-service contact.

## 5. Drafting-time measurements (evidence, to be re-derived)

Measured 2026-08-14 with temporary probes reusing the committed
loader/seeding/judgment path textually (deleted before this spec was
committed; every number is re-derived by the implementation — none
may be trusted from this file alone). The second adversarial review
independently re-derived the starred items with its own probe.

- Capture integrity: 383 replies / 766 components, zero recorded
  over-limit states, zero active restrictions, one limit string per
  policy, `date_ms − received_ms` ∈ [−1,002, −8] ms across all
  records, scheduled→sent gap median 68 / p90 225 / max 614 ms,
  `sent→received` min 47 / median 81 / max 241 ms.
- Clean phases (C1 only): 58,948; disposition-refuted: 1,052
  (matches SD-R5-F2). ★
- **C2 holds everywhere**: zero understatements at all 60,000
  phases (refuted phases measured up to their initiating reply). ★
- **The borderline halo is real and large** (C3 binding): 29,347
  clean phases in 28 bands carry ≥ 1 spurious-borderline component
  (up to 11 at one phase). ★ Measured halo bands: (2298–2471),
  (6955–6965), (7205–7453), (7467–7704), (7718–7954), (7968–8204),
  (8218–8453), (8469–8703), (8720–8954), (8971–9204), (9222–9454),
  (9472–9704), (9722–23601), (23692–23852), (23942–24102),
  (24193–24351), (24442–24602), (24693–24852), (24943–25102),
  (25194–25353), (25445–25603), (25694–25853), (25945–36548),
  (37298–37471), (42298–42471), (47298–47471), (52298–52471),
  (57298–57471).
- Halo witness structure (second review's measurement, the §2.4
  over-approximation made quantitative): of 207 spurious-borderline
  witnesses at the 56 halo edges, 73% occur where the policy's
  other component was recorded at limit (classification unchanged
  at policy level), and 54% are pad-compatible with the recorded
  schedule; **every halo edge retains ≥ 1 genuinely refuting
  witness (0 of 56 masked-only)**. ★
- **Consistent set Φ\***: 29,601 phases in 9 bands — (0–2297),
  (2472–6954), (6966–7204), (36549–37297), (37472–42297),
  (42472–47297), (47472–52297), (52472–57297), (57472–59999). ★
- Strict-overstatement envelope over Φ\*: min 25, max 57 of 766
  (edge measurements 32..53 sit inside it ★; over all clean phases
  it reaches 228 — the high-overstatement phases are the halo).
- Saturation agreement is 43/43 at every clean phase — recorded as
  the entailment consistency-check it is, not as an independent
  gate condition. The 43 components split 31 on window 0 / 12 on
  window 1. ★
- Full 766-component equality is achieved at no phase (max 741,
  min-over-clean 538; the 30 argmax phases are six of the twelve
  5 ms bands at residue 4,722–4,726 mod 5,000 — the 60 s bucket
  breaks the mod-5,000 symmetry). Deliberately not gated: the
  residual is the adversarial over-hold doing its job.
- Single alternative-convention datum (drafting probe, two witness
  phases only, *not* a sweep): driving arrivals at `received_ms`
  at φ=0 and φ=4,722 gave zero violations, 43/43, zero
  understatements, full agreement 739 and 723 of 766.
- Draft one's candidate "ledger floor" check (reservation-stamped,
  receipt-read) **fails at 57,059 of 58,948 clean phases (always by
  exactly 1)** — the measurement that, with the production-code
  reading, retired that draft's argument in favor of §2's
  classification premise. Kept as the record of why.
- **Blind witness audit (2026-08-15).** A machinery-forbidden
  session (fixture + B3 prose only; no `src/mock/`, no test code,
  no spec) hand-derived the first halo edge: at φ=2,298, reply 47
  (`character-request-limit`), burst count **5 = limit** against
  recorded **4** under every prose-consistent reading — the
  spurious borderline confirmed independently; at φ=2,297 the count
  is **4** under the implementation's exclusive-expiry reading (5
  under the inclusive reading — the 1 ms edge *is* the convention
  boundary); sustained count **14**, equal to the recorded state
  exactly. This breaks the shared-machinery circle at one witness:
  every other number in this section was measured through the
  committed loader/model path, and this one was not.

## 6. What Tom adjudicates

**Ratified in full, 2026-08-15** — all five items below, with the
retrospective-P1 and future-capture amendments folded in at
ratification. The list is preserved as the record of what was
signed.

1. Ratify **P1** as the *exhaustive* schedule-relevant feedback
   surface of the production client (you wrote it; the gate cannot
   test this premise, and it is load-bearing), **P2**, and **P3**
   (the common-bucket-epoch premise inherited from B3).
2. Approve the consistency definition C1–C3 and the **strict** halo
   treatment, with the §2.4/§5 measurements in view: C3
   over-approximates the policy-level classification flip (73% of
   edge witnesses are masked at policy level; 54% are
   pad-compatible), so strictness excludes phases whose
   counterfactual schedule may have matched the recording. A
   documented refinement (forgive a spurious borderline when the
   policy-aggregated classification is unchanged, or when the
   recorded next dispatch already satisfies the counterfactual
   pad) would recover much of the 29,347-phase halo at real spec
   complexity. Recommended: **strict now** — it errs conservative,
   29,601 witnesses are ample, and necessity-precision adds no
   calibration strength; revisit only if a future capture leaves
   Φ\* thin.
3. Accept §1's stated reduction of the anti-strictness half to
   non-emptiness plus pin discipline.
4. Approve the superseded-test deletion (contingent on the
   residue-assert migration), the precondition-5 divergence being
   recorded as a doc finding, and the proposed §7.4 marker text.
5. Confirm the discharge line: green gate ⇒ `s7-4-replay-gate`
   Full; nothing else changes coverage.

## 7. Adversarial-review provenance

Two fresh-context adversarial reviews, 2026-08-14, each instructed
to refute the current draft.

**Round one** (twenty findings) forced the major restructure:
entailed conditions unmasked (43/43 as corollary), a false
"equivalently" withdrawn, the anti-strictness circularity conceded
into §1, the anti-echo anchor added, and — decisively — the
draft's ledger argument exposed as reasoning about the wrong
client, which forced the production-code reading that surfaced the
three-way classification, the borderline halo, and the `Date`
bound.

**Round two** attacked the revision with an independent replay
probe. **Every attack on the gate itself failed**: all 56 halo
edges and 18 consistent edges classified exactly per the tables,
the partition arithmetic held, C2 held at every tested phase, the
anti-echo envelope held, and the hunt for a sub-limit magnitude
dependence or reconciliation channel in the C++ came back empty.
Its findings were justification repairs, all incorporated: the
halo-refutation story corrected to classification-equivalence with
the masking measurement on the page (finding 2), an unsupported
convention claim withdrawn (1), the entailment attribution fixed
(3), C1's discharge of the VIOLATION arm stated (4), the
stash-list bucket-cutoff divergence recorded (5), the stride
rationale replaced (6), P1 extended with the gate and hold channels
(7), the §2.1 code account corrected (8), and P3 added (9).

**Round three — blind witness audit (2026-08-15).** Both prior
reviews and every drafting probe ran through the committed
loader/seeding/model machinery, so a bug there would have fooled
all of them identically. To break that circle before adjudication,
a fresh session was given only the fixture and B3's prose —
forbidden the mock source, the tests, and this spec — and asked to
hand-derive the first halo edge without being told any expected
value. Result: exact agreement with the machinery (§5), plus the
discovery that B3's prose underdetermines the expiry-instant and
bucket-boundary conventions, and that the band edges sit exactly
where that underdetermination decides the answer — now a pinned
convention in §3 and a doc finding for implementation.
