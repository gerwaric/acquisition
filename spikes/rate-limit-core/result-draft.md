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

Two lanes, per the bucket-knowledge split (`scenarios.md` §1).
Prerequisites for **either** verdict slot (external review round 3
— explicit, so a green G-table alone can never justify a verdict):
M1–M13 with G1–G6 green over the lane's policies, **and** C1–C5
green, **and** X1–X2 green, with U1–U4 carried into the scoped
conclusion. X2 is load-bearing, not auxiliary: "single serialized
gate" is part of the register question itself.

- **Unconditional** — the four OAuth policies, bucket resolution
  `Known(5s/60s)` (N12): ⟨verdict; requires the prerequisites
  above over these policies⟩
- **Conditional** — `backend-item-request-limit`, bucket resolution
  `Assumed(60s/60s)` (not provably pessimistic; N14/N21 give no
  upper bound): ⟨verdict, stated *with* its assumption; same
  prerequisites, conditional lane⟩

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
failure anywhere must report its G6 reproduction record —
(seed, φ) mandatory for swept and property-generated tests,
optional for phase-independent and structural checks.

## §3. Conformance results

### Mock-judged wire scenarios

| ID | Scenario | Sweep | Gates exercised | Result | Evidence |
|---|---|---|---|---|---|
| M1 | Cold start with residue (flagship) | phase-swept | G1, G2, G6 | partial — core probe table and valid probe-429 lifecycle green; mock/actor lane pending | 2026-08-09: `cargo test --locked --test response_disposition` — valid probe 429 returns `ProbeReady`, seeds the already-configured policy, records generation/deadline, never returns `Requeue`, and makes the first GET the first confirmation; valid 2xx, malformed, 5xx, transport-unknown, and Cloudflare rows also pinned |
| M2 | Clean cold-start saturation burst | phase-swept | G1–G4, G6 | ⟨…⟩ | ⟨…⟩ |
| M3 | Degraded HEAD | independent | G1, G2, G5 | ⟨…⟩ | ⟨…⟩ |
| M4 | Unexpected policy shape | independent | G1, G2, G5 | ⟨…⟩ | ⟨…⟩ |
| M5 | Policy rename mid-session | phase-swept | G1, G2, G6 | ⟨…⟩ | ⟨…⟩ |
| M6 | Policy shrink mid-flight | phase-swept | G1, G2, G6 | ⟨…⟩ | ⟨…⟩ |
| M7 | Phantom same-account hits | phase-swept | G1, G2, G6 | partial — core phantom-synthesis lane green (audit fill, 2026-08-09: this row should have been marked with the reconciliation slice); mock/actor lane pending | 2026-08-09: `cargo test --locked --test response_reconciliation` — phantom-deficit synthesis, max-not-sum shared-history insertion, boot-residue seeding, monotone repeated/lower observations, and same-instant identity all pinned; the pessimistic/exact property runs 4,096 focused cases against an oracle independent of production windowing |
| M8 | 429 recovery and escalation | phase-swept | G1, G2, G5, G6 | partial — complete core restriction/episode/disposition slice green; mock-judged wire lane pending | 2026-08-09: `cargo test --locked --test response_disposition` — 15 tests cover exact retry boundaries, maximum configured bucket across multiple rules/windows, one confirmation in flight, every confirmation-matrix cell, malformed-429 precedence, Cloudflare halt, and unknown retention; focused `PROPTEST_CASES=4096` generation-tagged in-flight-set property green |
| M9 | Phantom race at saturation | phase-swept | G1, G2, G5, G6 + characterization | ⟨…⟩ | ⟨headroom-zero evidence base: what nonzero headroom would have bought, per contention level⟩ |
| M10 | Agent-loop stress | phase-swept | G1, G2, G3, G6 | ⟨…⟩ | ⟨…⟩ |
| M11 | Layer-1 ceiling + Cloudflare terminal | independent | G2, G5 | partial — core Cloudflare terminal precedence green; mock ceiling lane pending | 2026-08-09: `cargo test --locked --test response_disposition cloudflare_shape_halts_before_status_or_header_handling` — Cloudflare-classified 429 with absent policy headers halts before parse/status handling and latches `try_reserve` refusal |
| M12 | 4xx-tripwire obligations | independent | G5 | ⟨…⟩ | ⟨…⟩ |
| M13 | Gate structure on the wire | independent | G2 + gate-definition assertions | ⟨…⟩ | ⟨…⟩ |

G1, G2, G3, and G5 are armed in every mock-judged scenario; the
column lists the gates each scenario is the *binding evidence*
for.

### Core-property tests

| ID | Property | Result | Evidence |
|---|---|---|---|
| C1 | Padding arithmetic safe over all φ | green — full N13 per-window padding uses each explicit Known/Assumed resolution; shared policy history is judged across every rule/window and the maximum required `NotBefore` wins; headroom remains zero | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 19 passed, including a generated C1 property over arbitrary histories, multi-rule definitions, and independently generated server phases plus explicit just-before/on/after rollover and zero-headroom/order-statistic cases; focused `PROPTEST_CASES=4096 cargo test --locked --test c1_scheduling every_reserved_outcome_is_safe_for_every_server_phase` green (4,096 cases); independent oracle bucketizes hits on the server phase rather than calling production scheduling arithmetic; `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green. No skew sensitivity observed because this slice has no server-clock input; O5 remains out. Audit hardening (2026-08-09, same day): the property now asserts on every generated case — the earlier body was ~97% vacuous (§3 register, item 7) — and the `NotBefore` branch is re-asked and oracle-checked, pinning exactness; re-verified at 4,096 cases |
| C2 | Header parsing / shape validation | green for the implemented core slice — raw-header parsing, RulePair shape, and frozen response precedence are executable; remapping/shrink remain explicitly out of this slice | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 44 passed overall: the 7 parser tests remain green and 15 disposition tests pin Cloudflare-before-parse, malformed/out-of-model-before-429, valid-429 handling, and ordinary/probe outcomes; `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green |
| C3 | Fuse trip logic | ⟨…⟩ | ⟨…⟩ |
| C4 | 4xx tripwire logic | ⟨…⟩ | ⟨…⟩ |
| C5 | Lifecycle invariants | green — reservation/rollback/unknown-outcome identity and abandonment semantics remain green; raw ordinary responses and tokenless probes still share one count-max/synthetic-history reconciler; unknown confirmation outcomes stay counted; abandonment now covers the confirmation half (a dropped confirmation ages out as a failed attempt instead of wedging the policy — §3 register, item 2) | 2026-08-09: `cargo test --locked` in `spikes/rate-limit-core/` — 44 passed: all prior C1/C5/reconciliation tests remain green, and the disposition suite pins confirmation rollback plus pessimistic unknown retention; focused `PROPTEST_CASES=4096 cargo test --locked --test response_reconciliation` remains green (4,096 cases for each of two generated properties); `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check` green. Audit hardening (2026-08-09, same day): abandoned-confirmation expiry pinned in debug and release; interleaving property extended with observed responses and non-FIFO token resolution (2,048-case focused run); 59 tests total |

### Fault-injection and structural

| ID | Check | Result | Evidence |
|---|---|---|---|
| X1 | Fuse true-positive, burst and sustained fault shapes — the lane upgrade from declared-untested | ⟨…⟩ | ⟨…⟩ |
| X2 | Transport boundary: one HTTP client, private, no second send path | ⟨…⟩ | ⟨…⟩ |

### Gate summary

| Gate | Statement | Result | Evidence |
|---|---|---|---|
| G1 | Zero client-caused violations (incl. follow-on) | ⟨…⟩ | ⟨…⟩ |
| G2 | Neither B10 ceiling rule tripped, armed everywhere | ⟨…⟩ | ⟨…⟩ |
| G3 | Per-dispatch over-delay ≤ ε (final ε: ⟨…⟩) | ⟨…⟩ | ⟨…⟩ |
| G4 | M2 duration ≤ multiplier × padded minimum (final: ⟨…⟩) | ⟨…⟩ | ⟨…⟩ |
| G5 | Every scenario's own assertions (stimulus and structural alike) | ⟨…⟩ | ⟨…⟩ |
| G6 | Reproduction record for every failure ((seed, φ) mandatory where swept/generated) | ⟨…⟩ | ⟨…⟩ |

Earlier implementation finding (2026-08-09, response-reconciliation slice):
the frozen `core-design.md` sketches full response entry points over
`ObservedResponse` and an endpoint label, but reaching a valid policy
observation from those inputs necessarily crosses response precedence,
endpoint mapping/remapping, policy-shape adoption, and bucket-model
selection — behaviors explicitly deferred from this slice. The
initial conservative seam accepted an already parsed, valid
`PolicySnapshot`.
An ordinary response targets the token's existing policy; a probe
targets the snapshot's already configured policy. Policy-name mismatch
and unknown-policy cases are typed errors: they neither remap nor create
a policy, and a dispatched ordinary token remains counted and is still
consumed. The two public paths delegate only the count-max/synthetic-
history mechanism to one internal reconciler. This keeps the seam narrow
and left the full precedence/remap contract for its scheduled slice.
The response-disposition slice has now replaced that temporary public
seam with raw `ObservedResponse` entry points and kept the same internal
reconciler; only remap/shrink adoption remains deferred as requested.

Implementation finding (2026-08-09, 429 recovery/disposition slice):
the frozen spike documents require restriction through `Retry-After +
max configured bucket + buffer`, but do not place or name that buffer
in the Rust API. N13 and accepted design D3 identify the existing
buffer as 1 second (D3's maximum retry sleep is 900 + 60 + 1 seconds).
The conservative implementation therefore owns a non-caller-selectable
1-second constant in the core; it does not let a shell weaken recovery
padding. Raw response headers enter the core as sketched, and an
unacceptable `Retry-After` is refusal-shaped because no safe restriction
deadline can be derived. Policy remapping/shrink adoption remains
deferred: a parsed ordinary observation whose policy name differs from
the reservation is a policy-targeted refusal, while a probe may seed only
an already-configured policy.

### Audit findings register (2026-08-09)

An independent audit of the implemented slices (bugs, doc
consistency, test vacuity, idiom) produced the findings below. Each
carries its resolution or an explicit **open — flagged for Tom**
marker; resolved items cite the commit series landed the same day.

**Resolved in code (with tests):**

1. **Empty-rules policy was a reachable panic.** A probe 429 on a
   policy inserted with no rules aborted in bucket sizing.
   `Policy::new` is now fallible (`EmptyPolicy`); the state is
   unrepresentable and `RefusalReason::PolicyHasNoRules` is gone.
2. **Abandoned confirmation tokens wedged the policy.** In release
   builds (no drop bomb) a dropped confirmation left the episode's
   slot held forever — `NotBefore(MAX)` at every future instant,
   violating the C5 abandonment clause. The slot now ages out with
   its history entry (resolving as a failed attempt: First consumes
   the attempt, Final escalates), and the confirmation helpers
   tolerate a late response for an already-expired slot by routing
   it through the ordinary paths. Attribution choice taken
   conservatively: a stale-confirmation 429 **joins** the episode
   its attempt was already accounted against (the `open_or_join`
   generation guard widened to `<=`) rather than escalating —
   flagged here because the docs are silent on the case.
3. **Unusable `Retry-After` left the core unprotected.** A 429 with
   a valid policy observation but missing/invalid/above-cap
   `Retry-After` refused with *no restriction recorded*; the next
   `try_reserve` granted straight back into an active restriction.
   Both entry points now record a conservative `RETRY_AFTER_CAP`-
   length restriction (generation bump included) before refusing.
   Neither the precedence list nor the probe table names this case
   — a doc gap to close when `core-design.md` is amended or
   superseded by code authority.
4. **Phantom synthesis was unbounded.** `current-hits` is a raw u32;
   a legal `4294967295:10:0` state header would have materialized
   ~103 GB of history. Synthesis now targets
   `min(reported, largest configured max_hits)` — beyond that bound
   every configured window is saturated at `now` and further entries
   move no deadline. The pessimism property is restated as
   `local >= min(reported, cap)`.
5. **`RETRY_AFTER_CAP = 900 s` adoption recorded.** The constant is
   D3's product-policy cap (`RETRY_AFTER_CAP_SECS`, longest observed
   restriction 600 s per N23 plus headroom); it was implemented
   without an adoption record. The 900/901 boundary is test-pinned.
6. **`Retry-After` grammar pinned:** delay-seconds only, strict
   ASCII digits (no `+`, matching the triplet parser, which is now
   equally strict), surrounding whitespace trimmed in both parsers.
   The RFC 9110 HTTP-date form is deliberately out of model and
   lands in the conservative-restriction refusal above.
7. **The C1 property was ~97% vacuous** (measured): zero-max_hits
   rules and oversaturated generated histories meant a default run
   asserted ~7 times in 256 cases. The generator now draws
   `max_hits >= 1` (the zero boundary has its own pinned test) and
   the property asserts on *both* branches — a `NotBefore` answer
   must grant when re-asked at exactly that instant, oracle-checked,
   which also pins the arithmetic as exact rather than sufficient.
8. **The reconciliation "exact" oracle mirrored production** (it
   called `count_within`); it now computes from the test's own
   shadow timestamp list with plain integer arithmetic.
9. **C5 interleavings** now include observed responses and resolve
   tokens in generated (non-FIFO) order, per the scenario text.
10. **`StateChanged` emission rule defined:** emitted iff the call
    mutated engine state (synthesis, restriction, episode
    transition, newly halted); previously hard-coded `true` on the
    main paths and unasserted by any test.
11. **Coverage fills:** all `PolicyParseError` variants, all four
    required headers' absence, the O6 case/duplicate domain (pinned
    as current behavior: case-insensitive lookup, duplicate rule
    names yield duplicate rules, first header value wins), generic
    4xx completes and reconciles, `Retry-After: 0` exact deadline,
    Halt superseding the final-attempt matrix row, and the
    entry-point invariant swept across nine response shapes.

**Decided by Tom, 2026-08-09 (same-day review of the register):**

- **`NotBefore(SimInstant::MAX)` sentinel → `ReserveOutcome::Blocked`.**
  NotBefore is now always a real, sleepable future instant; the two
  wait-for-an-event cases (confirmation in flight, wire-legal
  zero-hit rule) return the distinct `Blocked` variant. Implemented
  and test-pinned.
- **Probe-429 episode follows the full confirmation matrix.**
  "Single confirmation attempt" in the M1 probe-429 prose meant one
  in flight at a time, not a one-attempt cap; the matrix's
  two-attempt design (external review F6) governs. Clarified in
  `scenarios.md` and `core-design.md`; pinned by a test (500 on the
  first GET preserves the episode and earns the final attempt).
- **Precedence-list ordering:** editorial note added to
  `core-design.md` — the numbered list orders dispositions, not
  execution; reconciliation of a valid observation precedes status
  handling, gated only by the Cloudflare and parse rules.
- **Stale-confirmation late 429 joins, not escalates** (register
  item 2's embedded judgment call): Tom confirmed the
  no-double-count reading — a trial already written off by
  abandonment aging charges its attempt once; its zombie 429 folds
  into the existing episode (restriction still recorded) rather
  than also tripping the shutdown. Behavior unchanged from the
  hardening pass; now decision-backed rather than implementer-read.

**Decided by Tom, 2026-08-09 — `bootstrap-seeding.md` accepted
(revision 2):** one global positional bucket constant at engine
construction (Assumed(60s/60s) shipped — C++ parity; flip to
Known(5s/60s) on U3 evidence); dynamic seeding of any valid observed
policy, no name-keyed refusal; `ProbeReady { policy }`; `RuleScope`
deleted; shape validation stays strict (acceptance review confirmed
the shape/identity split). The note's §3 amendment to `scenarios.md`
§1 is sanctioned and lands with the seeding slice; its §5 is the
slice contract. **The register has no open items** — the seeding
slice is the next work, and the first to run under the slice-review
process end to end.

- **The probe/bootstrap seam cannot discover a policy.** There is
  no `PolicySnapshot` → `Policy` construction path (nothing assigns
  `RuleScope` or `BucketModel`), `ProbeReady` carries no policy
  name for the actor's endpoint→policy map, the `scenarios.md` §1
  bucket-resolution table has no code home, and `RuleScope` —
  documented as parsed — is never produced by the parser. "Mapping
  seeded" is unreachable as written; a seeding design is needed
  before the actor slice.

## §4. Candidate N-claims

Transcribed to `network-ground-truth.md` at hoist, cited by
number there. Accumulated during reconciliation and design; each
survives or falls on its cited source, independent of the spike
verdict.

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
- Initialization: seeded from the capture's boot-HEAD state
  headers (reconciliation mechanism, phantoms at t₀); boot-HEAD
  records are initialization evidence, never replayed as
  counter-producing arrivals — ⟨confirm no double-application⟩
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
- 2026-08-09 — first-eyes pass (nine fixes) and external design
  review (Q&A round plus three findings rounds) completed; review
  closed with the design declared implementation-ready. Fuse
  gained its burst clause; probe-429 lifecycle, response
  precedence, and the probe outcome table pinned. The committed
  sibling docs are now the frozen implementation authority.
- 2026-08-09 — implementation began with the C2 header-parser and
  RulePair-shape slice. Seven offline tests pass (two property
  tests); response-precedence coverage remains before C2 is
  complete.
- 2026-08-09 — C5 reservation lifecycle landed: identity-bearing
  local and synthetic history, exact rollback, pessimistic unknown
  outcomes, and the guarded debug drop bomb. Fifteen offline tests
  pass overall; C1 padding and response reconciliation remain later
  slices.
- 2026-08-09 — C1 scheduling arithmetic landed: each saturated
  window adds its explicit Known/Assumed bucket resolution to the
  rolling expiry, and the shared-history policy answer is the
  maximum across every window and rule with zero headroom. Nineteen
  offline tests pass overall, including the independent-phase C1
  property (plus a focused 4,096-case run) and explicit
  rollover-boundary cases; all eight C5 tests remain green.
- 2026-08-09 — valid-observation response reconciliation landed:
  ordinary responses consume their reservation token, probes remain
  tokenless, and both use one pessimistic maximum-deficit merge into
  shared policy history. Twenty-nine offline tests pass overall;
  focused M1/M7 core cases and two 4,096-case reconciliation property
  runs cover boot residue, phantom hits, monotonicity, identity, and
  exact synthesis without starting the mock harness.
- 2026-08-09 — 429 recovery episodes and raw response disposition
  landed as one lifecycle slice. Forty-four offline tests pass overall;
  the 15 focused tests cover restriction generation/deadlines, arbitrary
  concurrent pre-restriction sets, single-confirmation concurrency, the
  complete two-attempt matrix, malformed-429 and Cloudflare precedence,
  the full probe table, and probe-429 seeding. The generation-set
  property plus the two reconciliation properties passed focused
  4,096-case runs; all C1/C5 tests remain green. Required fmt, locked
  test, and locked all-target clippy gates are green.
- 2026-08-09 — audit and mechanical hardening pass (see the §3 audit
  findings register). Four defects fixed with tests (empty-rules
  panic, abandoned-confirmation wedge, unbounded phantom synthesis,
  unusable-`Retry-After` unprotected refusal); C1 de-vacuized and
  strengthened to exactness on the `NotBefore` branch; reconciliation
  oracle made production-independent; C5 interleavings extended with
  observe and non-FIFO token order; strict digit parsing; StateChanged
  emission defined and asserted; coverage fills across C2 and the
  disposition suite; idiom pass (pub fields on plain-data types, one
  shape check, one confirmation-failure path) removed ~100 net lines.
  Fifty-nine offline tests pass in debug and release; fmt and
  all-target clippy green. Four design questions remain open in the
  register, flagged for Tom: the NotBefore(MAX) sentinel, the
  probe/bootstrap seeding seam, probe-episode attempt-count prose vs
  matrix, and the precedence-list ordering note.
- 2026-08-09 — Tom reviewed the audit register same-day (process
  change from the post-mortem: slices now end at review, codified in
  AGENTS.md). Three of the four open questions decided and landed:
  `ReserveOutcome::Blocked` replaces the NotBefore(MAX) sentinel;
  probe-opened episodes follow the full confirmation matrix
  (scenarios/core-design clarified, test-pinned); the precedence
  editorial note added. The stale-confirmation-429-joins judgment
  call is confirmed as decided. Sixty offline tests pass; the sole
  remaining open item is the probe/bootstrap seeding seam, which
  blocks the actor slice.
