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
date at the point of use; where the source is private correspondence
(the GGG support emails behind N12 and N14), no public URL exists, so
the statement names the correspondence and the `network-ground-truth.md`
entry that carries it instead.

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

**Unconditional — yes, within the offline spike scope for the four
OAuth policies at `Known(5s/60s)`.** **[Measured against model]** The
offline outcome: the declared 4,096-case extended-contract run covered
M1–M13, both M8 provenance lanes, and the SD-R8-F5 character-policy
lanes — every routed N23 endpoint — with gates G1–G6 green as
applicable; C1–C5 and X1–X2 are green and the SHELL prerequisite is
Full. **[Measured]** The independently edited obligations registry
records every prerequisite clause Full and its structural verifier
passes; the semantic accuracy of the recorded coverage is
prose-reviewed, not machine-proven. **[External — private
correspondence: GGG support to Tom by email, as of the email date;
carried as N12 in `network-ground-truth.md`]** The `Known(5s/60s)`
bucket resolutions themselves are a premise from that correspondence —
the offline runs exercise them but cannot measure the live server's
resolutions. U1–U5, the accepted future-parser limitation, and the
ratified O-series carriage below are part of the verdict's scope.

**Conditional — yes for `backend-item-request-limit`, conditional on
`Assumed(60s/60s)` being no smaller than the server's actual bucket
resolution.** **[Measured against model]** The same declared run
includes the shipped Assumed lane and the same registry basis applies.
**[External — private correspondence (the N14 ask-GGG channel, GGG
email) and Tom's direct observation (N21), both carried in
`network-ground-truth.md`]** The premise that no upper bound is known
for the legacy bucket resolution comes from those lanes: the ask-us
channel has not answered for the legacy policy and N21's observations
do not bound it. That external gap is exactly why this claim is
conditional rather than unconditional.

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

**[Measured scope] What these verdicts do and do not cover.** Every
mock-judged wire test — the M-series, which both verdicts are grounded
in — ran against an in-process mock server on simulated time; the
remaining evidence lanes (the C-series properties, the parser suite,
and the X-series fault-injection and source-structure tests) exercise
the code directly. No test anywhere sent real network traffic. Both
verdicts therefore carry the following exclusions as part of their
meaning, alongside U1–U5:

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
  always agrees with its own clock. Skew between a server's clock and
  its stated time remains untested — the spike has no server-clock
  input, so no skew-sensitivity evidence exists and the conditional
  re-entry trigger has not fired. The exclusion stands with its
  trigger armed rather than waved off (corrected per Tom's SD-R8-F14
  acceptance, 2026-08-15).
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
the run's own full-contract declaration and the independently edited
obligations registry, whose verifier is structural — it machine-checks
registry structure, declared coverage labels, citation arity, cited
test-function existence, and the open-set match, while the semantic
accuracy of each recorded coverage class is prose-reviewed. A fragment
report is never verdict-eligible.

**[Measured]** Both the pinned run and the 4,096-case generated-phase
extended-contract run declared successfully after the SD-R8-F4 and
SD-R8-F5 repairs. The extended run produced 16 reports per case and its
declaration required every N23 endpoint. **[Measured]** The registry
records 110 Full, no Partial, one accepted Untested limitation, and 13
Excluded, with its structural verifier green.

**[Measured]** Every scenario report was judged against the six
pass/fail gates of `scenarios.md` §6, and all six were green wherever
armed: G1 (zero client-caused violations, with unavoidable exposure
bounded and harness-attributed), G2 (neither layer-1 ceiling ever
tripped, armed in every mock-judged scenario), G3 (bounded
per-dispatch over-delay against the harness's independent padded-safe
oracle, at the finalized tolerance **ε = 500 ms** simulated), G4
(M2-shape scenario duration within the finalized **1.05×** multiplier
of the harness-computed padded minimum), G5 (every scenario's own
assertions), and G6 (deterministic reproduction records). The ε and
multiplier are the acceptance thresholds "honors" is measured
against; both were finalized by Tom on 2026-08-13.

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
end while the history entry itself is not quantized, buckets are
**half-open** (an arrival exactly on a grid point takes the full
following bucket), and expiry is **exclusive** (a hit whose adversarial
expiry equals an arrival instant is no longer counted at that arrival)
— the two conventions Tom's 2026-08-15 B3 amendment pinned. All of
this is an explicit model choice, not a claim about the server's
actual boundary semantics.

**[Measured]** N32 records the structural CN6 comparison: reprioritizing
is cheap in the spike actor's single owned deque because request identity
and positional removal already exist, whereas the superseded C++
coroutine/facade shape lacked per-entry cancellation identity.
**[Measured]** The dispatch mechanism, stated exactly: ordinary GET
dispatch reads only the queue front, while probe writer selection
(`Actor::schedule` via `pending_probe()`) scans the whole deque for a
queued unknown endpoint — which is how the actor already dispatches
out of arrival order under writer preference.
**[Measured]** Its tripwire is the single-deque property: if the actor
fans out into per-policy queues, cross-lane priority becomes a design
decision and N32 must be revisited. **[Measured]** N32 preserves its
contrast with the superseded C++ design because that warning can outlive
the premises that made it true.

**[Measured]** These six entries are the transcriptions of CN1–CN6 from
the spike result; `docs/design/network-ground-truth.md` on `master` is
the citation authority, and `redesign` receives them on its next sync.

## Reusable foundation

**[Measured]** The reusable artifact is the self-contained
`spikes/rate-limit-core/` package: the counter engine and its
in-process trait-impl delivery shim in `src/mock/`, the M1–M13 scenario
contract and driver, the G1–G6 judge in `src/conformance.rs`, the
full-contract declaration machinery, the C1–C5/X1–X2 focused tests,
and the obligations registry in `src/obligations.rs` with its
structural verifier.

**[Measured]** What this delivers is a **reusable foundation** — the
independent counter engine plus the scenario contract — not a ready
cross-client acceptance suite: the current driver imports and spawns
the spike's Rust actor directly and runs on Tokio paused time against
the in-process mock. Wrapping the same engine in a standalone HTTP
server and writing a client-neutral driver are future adapter work
(`scenarios.md` §7.1's "delivery-shim job" framing); if ADR-0003 takes
the rewrite path, that shim is built there against real requirements
(claim narrowed per Tom's SD-R8-F16 decision, 2026-08-15).
**[Measured]** Because the greenfield package is contained
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
