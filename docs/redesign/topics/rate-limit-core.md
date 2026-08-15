# Rate-limit core spike

Status: migration-package draft reopened by the final external audit
(SD-R8-F11–F20, 2026-08-15); not accepted for delivery.

Provenance: **[Measured]** Distilled 2026-08-15 from
`spikes/rate-limit-core/result-draft.md` on branch
`spike/rate-limit-core`. **[Measured]** The complete implementation,
evidence, review history, and reproduction machinery remain on that
branch.

## Claim lanes

**[Measured]** A `Measured` statement below is established by the
spike's code, offline runs, fixtures, registry, or recorded repository
history. **[Measured]** A verdict described as measured is
*measured against the model*: it is not a live-service result.

**[Estimated]** The spike's two verdicts rely on no estimated claim.

**[Inferred]** An `Inferred` statement is a model choice or conclusion
drawn from the recorded evidence rather than an external authority.

**[External]** An `External` statement names its URL and retrieval
date at the point of use.

## Question and gate definition

**[Measured]** Register question: can a Rust client demonstrably honor
the N-claims in `network-ground-truth.md` under burst load, as a single
serialized gate?

**[Measured]** “Single serialized gate” means one serialized scheduling
authority — the actor — with wire concurrency inherited from D5's gate
contract in full: in-flight cap 2, HEAD-exclusive with writer preference,
and FIFO among ordinary waiters. **[Measured]** It does not mean literal
one-request serialization.

## Verdicts

**[Measured against model] Unconditional — yes, within the offline
spike scope for the four OAuth policies at `Known(5s/60s)` (N12).**
All four were exercised: the declared 4,096-case extended-contract run
covered M1–M13, both M8 provenance lanes, and the SD-R8-F5
character-policy lanes, covering every routed N23 endpoint. C1–C5 and
X1–X2 are green, the SHELL prerequisite is Full, and the independent
registry verifies every prerequisite clause Full. U1–U5, the accepted
future-parser limitation, and the ratified O-series carriage below are
part of the verdict's scope.

**[Measured against model] Conditional — yes for
`backend-item-request-limit`, conditional on `Assumed(60s/60s)` being
no smaller than the server's actual bucket resolution.** N14/N21 provide
no upper bound, so this is not an unconditional claim. The same declared
run includes the shipped Assumed lane, the same prerequisite registry set
is Full, and the same scope carriage applies.

**[Measured]** The accepted future-parser limitation is that the spike
cannot force an upstream allocation cap in a future HTTP parser; it is
scoped outside the verdict prerequisites and remains Untested.

### U1–U5 declared-untested carriage

**[Measured scope — U1, remap triggers.]** Proactive provisionality at
auth transitions was dropped from scope; reactive handling (M5) is the
tested surface.

**[Measured scope — U2, server-side 4xx restriction behavior.]** The
threshold is opaque and has no incident data; the obligations are tested
by M12, while the server response is untested.

**[Measured scope — U3, legacy bucket resolution.]** The verdict is
conditional on `Assumed(60s/60s)`. The sanctioned live-validation
instrument is the designed path to measured-lane evidence; executing it
is not a spike gate. **[Inferred — U3 named hypothesis, Tom,
2026-08-09.]** The legacy burst resolution is 5s; the designated target
is a validation run against the Account `30:60:60` window with 5s
padding, randomized phases, and halt on the first violation.
**[Inferred — U3 CODE-lane prior.]** The C++ 75s cutoff has effectively
run 5s padding on this window for years without observed violations.
**[Inferred]** One 429 falsifies the hypothesis decisively, while passing
runs only accumulate phase-swept confidence because N15 says
quantization bites intermittently. **[Measured scope]** The shipped
assumption stays 60s/60s until evidence lands; the parked N14 ask to GGG
may retire the hypothesis before runs are spent on it.

**[Measured scope — U4, real layer-1 rules.]** They are deliberately
uncharacterized under the N4 strategy; M11's ceiling numbers sit in the
inferred lane.

**[Measured scope — U5, headroom instrumentation.]** M9's record of what
nonzero headroom would have bought at each contention level is
characterization for the headroom-zero decision, not conformance; no
gate consumes it. It is declared untested and unbuilt and is carried
into the scoped conclusion like U1–U4.

### O-series carriage

**[Measured scope] What these verdicts do and do not cover.** Every test
in this spike ran against an in-process mock server on simulated time;
no real network traffic was ever sent. Both verdicts therefore carry the
following exclusions as part of their meaning, alongside U1–U5:

- **[Measured scope — O1, no real network plumbing.]** No sockets, no TLS
  handshakes, no connection reuse, and no HTTP/1.1-versus-2 differences.
  The client and mock exchange requests in memory, so anything that can
  go wrong at the transport layer is untested.
- **[Measured scope — O2, no random timing.]** The mock answers after a
  fixed, deterministic delay. Real-world jitter — a response arriving
  unusually early or late — was not simulated.
- **[Measured scope — O3, no message bodies.]** The client under test
  never reads response payloads, so none were tested. The one exception
  is recognizing the Cloudflare block page by its HTML signature.
- **[Measured scope — O4, one account and one IP.]** The limiter was
  never tested sharing a rate budget with traffic from other accounts or
  addresses. The mock can inject “phantom” hits that mimic the countable
  effects of such traffic, but the real sharing semantics are out of
  scope.
- **[Measured scope — O5, perfect clocks.]** The mock's `Date` header
  always agrees with its own clock. Skew between a server's clock and its
  stated time was not tested, and the C1 property tests show the timing
  arithmetic is sensitive to skew, so this exclusion is flagged for
  re-entry rather than waved off.
- **[Measured scope — O6, well-formed headers only on the wire.]** The
  mock always emits canonical lowercase headers. Adversarial header
  casing and ordering are covered at the parser by C2's generated inputs,
  not end to end over the wire.
- **[Measured scope — O7, no authentication.]** No OAuth flow, tokens, or
  POESESSID were used; no credential appears in the tests or fixtures.
  Acquiring, refreshing, or attaching credentials is a later phase.
- **[Measured scope — O8, the declared leftovers.]** Server-side
  punishment for repeated 4xx errors (U2), the real Cloudflare rules
  (U4), the forum-posting regime, and endpoints that carry no rate limit
  are already declared out of scope in their own registers.

**[Measured scope]** These verdicts say the client's scheduling logic
honors the modeled rate-limit contract. They say nothing about transport,
authentication, or the live service; neither verdict is live-service
validation.

## Evidence basis

**[Measured]** Verdict eligibility requires two agreeing authorities:
the run's own full-contract declaration and the independently edited,
machine-verified obligations registry. A fragment report is never
verdict-eligible.

**[Measured]** Both the pinned run and the 4,096-case generated-phase
extended-contract run declared successfully after the SD-R8-F4 and
SD-R8-F5 repairs. The extended run produced 16 reports per case and its
declaration required every N23 endpoint. **[Measured]** The independent
registry totals were 110 Full, no Partial, one accepted Untested
limitation, and 13 Excluded.

**[Measured]** The mock verdicts are measured against the model; the
sanitized capture replay grounds the model in the observed lane.
**[Measured]** The full evidence lives on branch
`spike/rate-limit-core`, principally in
`spikes/rate-limit-core/result-draft.md`, `status.md`, `scenarios.md`,
`src/obligations.rs`, and the tests and fixtures under that directory.

## Ground-truth follow-ups

**[External, retrieved 2026-08-09]** N27 records the official
invalid-request budget from
<https://www.pathofexile.com/developer/docs/index>: too many 4xx
responses in a short period restrict access, 429 belongs to that budget
as well as the policy budget, and the threshold parameters are
undocumented.

**[External, retrieved 2026-08-09]** N28 and N29 record the
challenge-shaped Cloudflare block signature and recourse asymmetry from
<https://community.cloudflare.com/t/blocked-from-path-of-exile-api-but-not-allowed-to-contact-support/549055>.

**[External, community-observed February 2021; retrieved 2026-08-09]**
N30 records that trade-API rules can carry three windows per rule from
<https://www.pathofexile.com/forum/view-thread/3056323>; the spike's
two-window `RulePair` shape is therefore out-of-model for such a policy,
not a claim that such policies are impossible.

**[Inferred]** N31 records that N11–N13 do not specify the exact bucket
quantization boundary semantics and that the mock uses the
most-adversarial consistent reading: a timestamp rounds up to the bucket
end while the history entry itself is not quantized.

**[Measured]** N32 records the structural CN6 comparison: reprioritizing
is cheap in the spike actor's single owned deque because request identity
and positional removal already exist, whereas the superseded C++
coroutine/facade shape lacked per-entry cancellation identity.
**[Measured]** Its tripwire is the single-deque property: if the actor
fans out into per-policy queues, cross-lane priority becomes a design
decision and N32 must be revisited. **[Measured]** N32 preserves its
contrast with the superseded C++ design because that warning can outlive
the premises that made it true.

**[Measured]** These six entries are the transcriptions of CN1–CN6 from
the spike result; `docs/design/network-ground-truth.md` on `master` is
the citation authority, and `redesign` receives them on its next sync.

## Reusable acceptance suite

**[Measured]** The reusable artifact is the self-contained
`spikes/rate-limit-core/` package: the counter engine and delivery shim
in `src/mock/`, the M1–M13 scenario contract and driver, the G1–G6 judge
in `src/conformance.rs`, the full-contract declaration machinery, the
C1–C5/X1–X2 focused tests, and the machine-checked obligations registry
in `src/obligations.rs` with its verifier.

**[Measured]** The mock plus the M-series are the acceptance suite any
future limiter must pass, including the C++ client through a standalone
delivery shim. **[Measured]** Because the greenfield package is contained
under `spikes/rate-limit-core/`, it can be hoisted to its own repository
without surgery.

## Exported review lesson

**[Measured]** Closure reviews must re-derive claims from evidence, not
only verify the machinery that produced them. **[Measured]** The SD-R7
and SD-R8 audit history in `spikes/rate-limit-core/result-draft.md` §9
is the evidence: later no-context audits reopened apparently closed
rounds, and SD-R8 in particular exposed a declaration/registry agreement
that still omitted a required lane and a result statement that omitted
required scope carriage.
