# Bootstrap-seeding slice hand-off

Status: reviewed and closed by Tom on 2026-08-10; review-close commit
`17363429`. This artifact was written per `slice-review.md` §2 for
the accepted `bootstrap-seeding.md` §5 slice implemented from baseline
`a3245e86`. Coding gates are green. No mock or actor work is included.

Historical record — live state lives in `status.md`. (2026-08-12)

## 1. Silences taken, with next-call consequences

No new doc gap required an invented policy rule. The accepted note
settles identity, bucket provenance, and probe output. These boundary
readings are made explicit because they are the places a later slice
could otherwise read differently:

| Boundary case | Reading taken | Next-call consequence |
|---|---|---|
| Valid observation on a non-success, non-429 probe | “Valid observation” is header validity, independent of status: register and reconcile first, then apply the unchanged probe table and refuse readiness | the actor receives no `ProbeReady` and cannot release or map the endpoint; a later successful probe reuses the bounded policy, reconciles again, and can return readiness |
| Malformed or out-of-model probe observation | precedence rule 2 runs before registration; invalid shape or identity never enters the policy map | `try_reserve` for that name remains `UnknownPolicy`; the next probe reparses from a clean bootstrap state |
| Valid probe 429 for a new policy | register and reconcile first, then record the restriction/open the episode exactly as for an existing policy | the next `try_reserve` answers `NotBefore` through the single authority; its first eventual grant is the episode confirmation |
| Cumulative number of dynamically discovered policies | the bound remains where the frozen architecture puts probe eligibility: D5's five endpoint labels and N16 exactly-once actor probing; the core adds no name allow-list or sixth-policy refusal | a conforming actor can bootstrap at most five policy identities; a shell that violates that future structural contract could grow the policy map, so the actor slice must pin the five-label/exactly-once bound |
| Manually inserted policy has buckets different from the bootstrap default | `insert_policy` remains the explicit test/configuration seam; the default applies only to policies discovered from probes | later observations reconcile against the inserted policy's existing buckets; the engine never overwrites them from a name or from its bootstrap default |

The first three consequences are test-pinned. The actor-owned bound is
not executable yet because the mandated build order leaves the actor
for last; it is carried in §3 rather than silently claimed as covered.

## 2. Seam map and six-invariant walk

Earlier-slice state touched by bootstrap seeding:

- parser snapshots → policy construction: already bounded, validated
  `RulePair`s are cloned into `Rule`s; `Policy::new` retains the
  non-empty structural guard;
- policy map → reconciliation/history: a vacant entry is installed
  before the existing maximum-deficit reconciler seeds boot residue;
- restriction generations/recovery episodes: a newly seeded valid
  429 follows the same bookkeeping and confirmation matrix as an
  existing policy;
- terminal/suspended state → probe dispositions: seeding does not
  weaken the existing gates on send-promising outcomes;
- notifications → registration: the new `seeded` mutation bit joins
  synthesis/restriction/episode mutation in the one `StateChanged`
  decision;
- test-configured policies → constructor default: `insert_policy`
  preserves their per-rule bucket models; only vacant probe discovery
  consumes `default_buckets`.

1. **No permanent wedge.** Registration creates no token, confirmation
   slot, or independent deadline. A seeded 429 uses the existing
   restriction/episode state, whose slot and entries explicitly resolve
   or age out on the padded-window horizon.
2. **One send, one entry.** Probe registration creates only bounded
   synthetic residue through reconciliation; probes remain tokenless.
   Ordinary reservations and remove-by-id token consumption are
   unchanged, and the full C5 interleaving suite remains green.
3. **Pessimism direction.** A fresh policy begins empty and synthesizes
   the capped maximum reported deficit at `now`; lower/repeated probes
   remove nothing. `Assumed(60s/60s)` is passed explicitly in the shipped
   test configuration, so no policy name silently receives faster
   reopening arithmetic.
4. **`try_reserve` is the single scheduling authority.** Seeding grants
   nothing and returns no send time. Even a probe 429 records state and
   leaves the next timing decision to `try_reserve`.
5. **Entry-point invariant.** `on_response` still cannot return
   `ProbeReady`; `on_probe_response` still cannot return
   `CompleteRequest` or `Requeue`. The policy payload changes data, not
   lane membership, and the nine-shape sweep remains green.
6. **Notifications tell the truth.** A vacant registration always emits
   `StateChanged`, including zero residue; a repeated zero-deficit probe
   emits none. Both halves are pinned in
   `zero_residue_probe_reports_only_the_initial_registration_mutation`.

## 3. Coverage confession and property-test reachability

Covered by focused examples: unknown-policy success with zero and
nonzero residue; two observed rules copied with the same explicit
default; preservation of an unrelated existing policy; repeat-probe
idempotence; valid 429 discovery through restriction and first
confirmation; malformed 429 non-registration; valid 5xx registration
without readiness; policy-bearing `ProbeReady`; existing-policy probe
behavior after the constructor migration.

Not covered, deliberately: endpoint→policy storage, exactly-once HEAD
eligibility, the five-endpoint cumulative discovery bound, mock/M-series
wire judgment, actor dispatch, U3's eventual default flip, M5 remap, and
M6 shape adoption. Those are later accepted slices. The cross-product of
vacant registration with already-halted/already-suspended state and with
every unusable `Retry-After` grammar case is not repeated; existing-policy
tests cover each terminal/restriction disposition, while focused vacant
tests cover success, 429, 5xx, and parse refusal. No live service was
contacted.

This slice adds example tests, not a new property. The full 4,096-case run
still covers nine properties, none assertion-free:

- C1 scheduling asserts on both generated branches: a grant is checked by
  independent server-phase arithmetic; `NotBefore` is re-asked exactly and
  its grant is checked. Zero-hit and refusal branches are excluded by the
  generator and separately pinned.
- C2's valid-pair property always asserts the parsed fields; its malformed
  text property always asserts a typed error. Neither oracle calls the
  parser under test to compute an expectation.
- C5 rollback always reserves below its generated capacity and asserts an
  exact history round trip; the interleaving property generates at least
  one operation and compares the complete id/kind map after every step;
  the debug drop property always catches the drop bomb and independently
  checks conservative retained history.
- The M8 generation property creates at least one original even when its
  generated tail is empty, then unconditionally checks generation,
  episode, confirmation, and blocked-second-confirmation state.
- Both reconciliation properties generate at least one observed rule and
  unconditionally assert exact max-not-sum growth, independent raw-
  timestamp pessimism, and repeat/lower monotonicity.

Removing `RuleScope` deletes only a semantically dead random bit from C1's
generator; scheduling and reconciliation were scope-blind before and
remain so. Gate evidence recorded in `result-draft.md`: 75 debug tests,
73 release tests, all nine properties at 4,096 cases, clippy with warnings
denied, and formatting check, all green on 2026-08-10.

## 4. Judgment calls

- Removed `PolicyEngine: Default` instead of silently choosing the shipped
  `Assumed(60s/60s)` model inside the core. Construction must name the
  provenance-typed model; the later actor owns the one production value.
- Used `HashMap::entry` for discovery so absence check and insertion are one
  structural operation. Policy creation still goes through `Policy::new`;
  no duplicate-insert race or second guard path exists.
- Registered any parse-valid observation before status disposition,
  including 5xx. This follows the accepted note's shape/identity split and
  retains bounded server knowledge without promising endpoint readiness.
- Kept the cumulative discovery limit actor-owned rather than introducing
  a core policy-name/cap refusal that would contradict the accepted dynamic-
  identity decision. This is safe only with the frozen five-endpoint,
  exactly-once probe structure and is therefore an explicit actor-slice
  obligation.
- Deleted `RuleScope` outright rather than retaining a telemetry-only enum:
  rule names remain in parser snapshots for header association, while the
  policy engine has no scope-dependent behavior to justify representable
  state.
