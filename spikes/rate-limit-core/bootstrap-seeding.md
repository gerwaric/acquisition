# Bootstrap seeding: how a probe's policy becomes a registered Policy

Status: **proposed amendment, awaiting Tom's review** — drafted
2026-08-09 to resolve the sole remaining open item in the
`result-draft.md` §3 audit register. Amends the frozen design under
the post-audit process (slices end at review); nothing here is
authority until Tom accepts it. On acceptance, the "What changes"
section becomes the next slice's contract and this doc joins the
sibling set.

N-numbers cite `docs/design/network-ground-truth.md`; D-numbers cite
`docs/design/network-redesign.md`; §-references without a filename
cite `scenarios.md`.

---

## 1. The problem

The boot HEAD exists to discover an endpoint's policy (D5, M1), but
the implemented core cannot complete the discovery. Four connected
gaps (audit register, 2026-08-09):

1. There is no `PolicySnapshot` → `Policy` construction path —
   nothing assigns `RuleScope` or `BucketModel`, so a valid probe
   observation naming an unregistered policy is *refused*
   (`ObservationError::UnknownPolicy`) and "mapping seeded"
   (`core-design.md`, `ProbeReady`) is unreachable.
2. `ProbeReady` carries no policy name, so even a successful probe
   on a pre-registered policy leaves the actor unable to learn the
   endpoint→policy mapping — while the design forbids the shell
   from parsing headers itself (F2/C2).
3. The §1 bucket-resolution table (the five policies and their
   provenance-typed resolutions) has no code home.
4. `RuleScope` is documented as parsed (`core-design.md` §2) but the
   parser never produces one; `Rule.scope` is caller-invented.

## 2. Constraints inherited from the frozen design

- Parsing is the core's job; the shell never reads rate-limit
  headers (F2; C2's verbatim-strings decision depends on it).
- Bucket resolution is provenance-typed and **never manufactured**:
  "an unknown policy without configured resolution is a refusal,
  not a guess" (§1). `Assumed` is an explicit, replaceable entry,
  never a default the code invents.
- Probe eligibility and the endpoint→policy map are actor-owned;
  the core never schedules a HEAD (`core-design.md` §5).
- The D4 blast radius for a failed boot probe is the endpoint.
- Remap (M5) and shrink (M6) stay deferred; this note does not
  touch them.

## 3. Proposed design

**The engine owns the resolution table; probes seed against it;
`ProbeReady` reports the mapping as data.**

### 3.1 Resolution table at engine construction

```rust
// Provenance-typed configuration, §1 verbatim. One BucketModel per
// policy, applied to each of its rules (legacy Account and Ip rules
// share the Assumed(60s/60s) entry).
pub struct ResolutionTable {
    entries: HashMap<PolicyName, BucketModel>,
}

impl PolicyEngine {
    pub fn new(resolutions: ResolutionTable) -> Self { ... }
}
```

The five §1 entries are the table's initial contents, constructed by
the shell (later: read from configuration) and handed to the engine
once. The engine is the right owner because the refusal decision —
unknown resolution ⇒ refuse — is a scheduling-safety decision, and
the core is the single scheduling authority.

### 3.2 Seeding via the probe path

`on_probe_response`, valid observation, policy not yet registered:

- **Table hit** → build the `Policy` from the observation (each
  `RuleSnapshot`'s parsed `RulePair` and scope, the table's
  `BucketModel`), register it, then proceed exactly as today:
  reconcile the state header (M1 residue), apply the probe
  disposition table (2xx → `ProbeReady`; 429 → restriction +
  episode + `ProbeReady`; etc.). The observed shape *is* the
  configured shape at seed time — this is the mapping being
  established, not a shrink (M6's deferral covers only *later*
  changes to an already-registered policy).
- **Table miss** → `Refuse` (endpoint target) with a new cause,
  `RefusalCause::UnknownResolution(PolicyName)` — the §1 rule made
  executable. No policy is created; nothing is reconciled.

Already-registered policy → unchanged (reconcile into existing
state; repeat probes are idempotent per the monotonicity property).
Ordinary `on_response` naming an unregistered or mismatched policy
→ unchanged refusal (that path is M5 remap territory, deferred).
`insert_policy` remains public for test seeding.

### 3.3 `ProbeReady` carries the policy name

```rust
Disposition::ProbeReady { policy: PolicyName }
```

The actor records endpoint→policy from this payload — data out of
the core, no header parsing in the shell, no side channel.

### 3.4 `RuleScope` parsed, unknown scopes out-of-model

`parse_policy` maps rule names case-insensitively: `account` →
`RuleScope::Account`, `ip` → `RuleScope::Ip`. Any other rule name →
new typed error `PolicyParseError::UnknownRuleScope { rule }`,
which travels the existing identical-refusal path (one cooldown
path, one exposure bound — the variant is telemetry, not
branching). Rationale: N23 names exactly these two rules; N8's
Client scope is charter-excluded; and CN4 already establishes that
out-of-model shapes refuse rather than guess. `RuleSnapshot` gains
the parsed scope; `Rule::scope` stops being caller-invented.

## 4. Alternatives considered

- **Shell builds the `Policy`** (from `ProbeReady` + its own
  parse): violates F2 — the shell would need to read headers or
  duplicate shape knowledge. Rejected.
- **Engine auto-registers with a default resolution** when the
  table misses: manufactures an `Assumed` the docs forbid
  manufacturing. Rejected.
- **Resolution table lives in the actor**, passed per-call to
  `on_probe_response`: makes the safety-relevant refusal depend on
  per-call shell behavior and widens the API for no gain. Rejected.

## 5. What changes (slice contract, if accepted)

| Item | Change |
|---|---|
| `PolicyEngine::new` | takes `ResolutionTable` |
| `Disposition::ProbeReady` | gains `{ policy: PolicyName }` |
| `RefusalCause` | gains `UnknownResolution(PolicyName)` |
| `PolicyParseError` | gains `UnknownRuleScope { rule }` |
| `parse_policy` | produces `RuleScope` per rule |
| `on_probe_response` | seeds on table hit; refuses on miss |
| Tests | M1 seeding assertions become executable end-to-end at the core level: unknown endpoint's probe registers the policy, `ProbeReady` names it, residue reconciles into the fresh policy; table-miss refusal pinned; unknown-rule-scope refusal pinned; C2 gains scope-parsing rows; existing `unknown_probe_policy_is_typed…` test changes meaning (it currently pins the refusal that seeding replaces — it becomes the table-miss test) |

Cross-slice invariants walk (per AGENTS.md): seeding creates a
policy only through `Policy::new` (non-empty rules enforced — a
valid `PolicySnapshot` always has ≥1 rule, but the constructor
still guards); no new scheduling path (seeding grants nothing;
`try_reserve` remains sole authority); entry-point invariant
unchanged (`ProbeReady` still probe-lane-only); pessimism unchanged
(seed-time reconciliation is the existing mechanism, M1 = empty
history case); no new wedge states (a refused table-miss leaves no
partial registration).

## 6. Questions folded in — flag any you want reopened

1. Unknown rule names refuse (out-of-model) rather than parse with
   a catch-all scope. Chosen for CN4-consistency; the alternative
   (a `RuleScope::Other(String)`) would silently accept shapes we
   have never observed.
2. The observed shape becomes the configured shape at seed time
   (no separate "expected shape" table). Chosen because the docs
   define no such table and the shape invariant is already
   parse-enforced.
3. The table keys on policy *name* alone. If GGG ever reuses a
   name with a different shape, reconciliation still governs
   scheduling and the shape mismatch surfaces via
   `StatePeriodsMismatch`/`UnexpectedPolicyShape` refusals — no
   silent trust.
