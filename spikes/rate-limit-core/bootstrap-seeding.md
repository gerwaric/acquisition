# Bootstrap seeding: how a probe's policy becomes a registered Policy

Status: **accepted by Tom, 2026-08-09** — §5 is the next slice's
contract, and the §3 amendment to `scenarios.md` §1 is sanctioned.
Acceptance review confirmed the shape/identity split explicitly:
shape validation (triplet count, period ordering, state mismatch)
stays strict and refusal-shaped; only *identity* (policy and rule
names) seeds dynamically. Revision 1 (same day) proposed a
policy-name-keyed resolution allow-list; Tom's review challenged
the hard-coded names and the discussion collapsed the design to a
global constant (the decision record is §4).

N-numbers cite `docs/design/network-ground-truth.md`; D-numbers cite
`docs/design/network-redesign.md`; §-references without a filename
cite `scenarios.md`.

---

## 1. The problem

The boot HEAD exists to discover an endpoint's policy (D5, M1), but
the implemented core cannot complete the discovery. Four connected
gaps (audit register, 2026-08-09):

1. No `PolicySnapshot` → `Policy` construction path — a valid probe
   observation naming an unregistered policy is *refused*, so
   "mapping seeded" (`core-design.md`, `ProbeReady`) is unreachable.
2. `ProbeReady` carries no policy name, so the actor cannot learn
   the endpoint→policy mapping — while the design forbids the shell
   from parsing headers (F2/C2).
3. Bucket-resolution knowledge (§1) has no code home.
4. `RuleScope` is documented as parsed but never produced, and is
   read by nothing (reconciliation is scope-blind by charter).

## 2. The design

**Policies are discovered, never enumerated. Bucket resolution is
one global positional constant. No policy name appears anywhere in
configuration or code.**

### 2.1 One global bucket default

```rust
impl PolicyEngine {
    /// `default_buckets` applies positionally to every seeded rule:
    /// burst-slot windows get `.burst`, sustained-slot windows get
    /// `.sustained` (the §1 positional-tier assumption, Q4).
    pub fn new(default_buckets: BucketModel) -> Self { ... }
}
```

- **Shipped value, pre-U3:** `Assumed(60s) / Assumed(60s)` — the
  C++ client's field-proven conservative posture. Never worse than
  the shipped client for any policy.
- **Post-U3 flip:** `Known(5s) / Known(60s)` once U3's validation
  instrument (result-draft §6) or GGG evidence confirms the legacy
  buckets match N12's. The flip is the single point where U3
  evidence lands: two durations change, nothing else. `Resolution`'s
  existing provenance typing carries the evidence state.
- Restriction arithmetic is unaffected in every state: the maximum
  configured bucket is 60 s throughout (`core-design.md` F4), so
  429 deadlines are identical pre- and post-flip. Only saturated
  burst-window reopening latency differs.

`Rule` keeps its per-rule `BucketModel` field — seeding applies the
default uniformly, but the C1 property deliberately exercises mixed
per-window resolutions and the spike's unconditional lane still
tests OAuth policies under `Known(5s/60s)`; the generality is
already paid for. The M-series conformance sweep must include the
shipped `(60s, 60s)` default among its configurations.

### 2.2 Seeding via the probe path

`on_probe_response`, valid observation, policy not yet registered →
build the `Policy` from the observation (each `RuleSnapshot`'s
parsed `RulePair`; every rule gets the engine's default buckets),
register it, then proceed exactly as today: reconcile the state
header (M1 residue — this is the existing mechanism applied to an
empty history), apply the unchanged probe disposition table.

- Already-registered policy → unchanged (reconcile into existing
  state; repeat probes stay idempotent per the monotonicity
  property).
- Ordinary `on_response` naming an unregistered or mismatched
  policy → unchanged refusal (M5 remap territory, still deferred).
- `insert_policy` remains public for test seeding.
- There is no refusal for an unrecognized policy name — a GGG
  rename or new policy degrades to conservative scheduling, never
  to an unusable endpoint. (Revision 1's `UnknownResolution`
  refusal is deleted; see §4.)

### 2.3 `ProbeReady` carries the policy name

```rust
Disposition::ProbeReady { policy: PolicyName }
```

The actor records endpoint→policy from this payload — data out of
the core, no header parsing in the shell. This was the one part of
revision 1 the simplification kept intact: the *endpoint*
vocabulary is the legitimately static knowledge (D5), and the actor
still needs to learn which discovered policy each endpoint answers
to.

### 2.4 `RuleScope` is deleted

Reconciliation is scope-blind by charter and `Rule::scope` is read
by nothing (audit finding). The enum and field are removed; the
parsed rule *name* (already kept on `RuleSnapshot`, needed to find
the headers) covers telemetry. A rule with an unrecognized name but
a valid two-window shape schedules normally — shape is the
contract, identity is not. `core-design.md`'s `scope: RuleScope //
parsed` line becomes history when code becomes authority.

## 3. Frozen-doc amendment required (Tom's sign-off)

§1's rule — "an unknown policy without configured resolution is a
refusal, not a guess" — is amended to:

> An unknown policy seeds under the engine's global default bucket
> resolution, which is explicit, provenance-typed (`Assumed` until
> U3 evidence upgrades it), and configured in exactly one place.
> The prohibition stands against *implicit* or *manufactured*
> resolutions: there is still no code path that invents a
> resolution not traceable to that single configured value.

Rationale for the amendment: the refusal posture protected the
spike's verdict lanes, but lane discipline only needs provenance
*tagging* — anything scheduled under an `Assumed` default is
conditional-lane by construction, and the unconditional verdict
stays scoped to the four N12 policies as tested. Meanwhile the
refusal posture's production failure mode (a GGG rename bricks an
endpoint until a client update ships, with remap deferred) is
strictly worse than a conservative guess whose worst case — a 429 —
is absorbed by the restriction/episode/escalation machinery built
for exactly that. §1's per-policy table remains as the *evidence
record* informing the verdict lanes and the eventual flip; it stops
being runtime configuration.

## 4. Decision record — bridges considered while U3 is unconfirmed

The per-policy question only exists in the interval where OAuth
buckets are proven (N12) and legacy's are not. Confirming U3
collapses every option below to the same two constants.

| Bridge | Names | OAuth 5s benefit now | Verdict |
|---|---|---|---|
| Rev-1 allow-list (refuse unknown names) | policy names, load-bearing | yes | **rejected** — correctness coupled to GGG's namespace; rename bricks an endpoint |
| (b) Name-keyed overrides over a default | policy names, degrade-only | yes | declined — names as temporary evidence records; more machinery than the benefit warrants |
| (c) Endpoint-class keyed (OAuth-tier 5/60, legacy-tier 60/60) | none new (endpoint bit is already static) | yes | declined — clean, and the designated retrofit if burst latency demonstrably hurts |
| **(a) Global constant, flip on U3** | **none, ever** | no — defers to the flip | **chosen (Tom, 2026-08-09)** — matches the C++ client's field-proven posture, so never a regression; config is two durations; U3's instrument is precisely the flip trigger |

What the deferral costs: the 5s burst bucket affects how quickly a
*saturated burst window reopens* (bursty latency), not long-run
throughput (the sustained window dominates long runs) and not
restriction deadlines (identical either way). Until the flip, OAuth
burst behavior equals the shipped C++ client's.

## 5. What changes (slice contract, if accepted)

| Item | Change |
|---|---|
| `PolicyEngine::new` | takes `default_buckets: BucketModel` |
| `Disposition::ProbeReady` | gains `{ policy: PolicyName }` |
| `on_probe_response` | seeds any valid observation for an unregistered policy under the default buckets |
| `RuleScope` | deleted (enum, `Rule` field, and `Rule::new` parameter) |
| `scenarios.md` §1 | amended per §3; the table reframed as evidence record |
| Tests | M1 seeding assertions executable end-to-end at core level (probe on unknown policy registers it, `ProbeReady` names it, residue reconciles into the fresh policy); seeded-rule buckets pinned to the default; repeat-probe idempotence re-pinned across seeding; `unknown_probe_policy_is_typed…` inverts (it currently pins the refusal that seeding replaces — it becomes the seeds-under-default test); C2 loses the scope rows revision 1 would have added |

Cross-slice invariants walk (per AGENTS.md): seeding creates
policies only through `Policy::new` (non-empty guard still enforced;
a valid snapshot always has ≥1 rule); no new scheduling path
(seeding grants nothing — `try_reserve` remains sole authority);
entry-point invariant unchanged; pessimism unchanged (seed-time
reconciliation is the existing M1-residue mechanism); no new wedge
states (seeding either completes or the refusal paths that already
exist apply); notifications: seeding a policy is an engine mutation
→ `StateChanged`.

## 6. Questions folded in — flag any you want reopened

1. Seeded configured shape = observed shape (no expected-shape
   table); later shape changes remain M6-deferred territory and
   surface as `StatePeriodsMismatch`/`UnexpectedPolicyShape`
   refusals — no silent trust.
2. The default is engine-construction configuration, not a
   compile-time constant, so the U3 flip (and any spike sweep) is a
   config change, not a code change.
3. If bursty latency measurably hurts before U3 confirms, the
   designated retrofit is bridge (c) — endpoint-class keyed — not a
   return to policy names.
