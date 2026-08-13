# Obligation map: rate-limit-core spike

> **Superseded by the clause registry, 2026-08-12.** Every row below
> was migrated row-for-row into `src/obligations.rs` (verified by
> `tests/obligations.rs`); the registry is now the authority for
> clause ownership, coverage state, and citations, and
> coverage-state changes are diffs to the registry, not edits here.
> The dated audit text below is preserved as history — including the
> §8 discrepancy report, which remains the record of the open
> findings as of `e2034807`. (Supersession per the `core-design.md`
> precedent; migration deltas are listed in the registry slice's
> hand-off.) Live state — what is open, blocked, and next — is
> `status.md`, nothing here. *(Sentence and the per-section
> reminders below added 2026-08-13, DS-R1: the banner fires once in
> a 500-line file, and everything below it reads as live to a reader
> who lands mid-file by grep or a §8 pointer.)*

Status: audit artifact, 2026-08-12, produced at head `e2034807`
("make C3 own the fuse headroom claim"). Audit only — no code, no
test, and no existing register or evidence text was changed. The
discrepancy report is §8; §9 lists the checks that came back clean.

Method, per the kickoff and `slice-review.md` §1 lesson 4: for every
assertion clause, the *owning* row was resolved from `scenarios.md`
before any test was searched for, and the search covered the
`#[cfg(test)]` modules inside `src/` (`src/actor.rs:876`,
`src/mock/model.rs:551`) as well as `tests/`. A clause owned by C- or
X-series is mapped to its owner's test, not re-demanded of an M row.

## §0. How to read

One row per assertion clause — a scenario with four asserts is four
rows. Columns:

- **Owning row**: which `scenarios.md` row the clause belongs to.
  Where the contract splits ownership (property vs. integration
  instance), both are named and the note says how they compose.
- **Discharging test**: exact target path and test name. Multiple
  tests are listed strongest-first; `⊗` marks a composition where no
  single test carries the clause alone.
- **Coverage**: `full` — a test fails if the clause is broken, at the
  clause's stated scope; `partial` — a fragment, one shape, one lane,
  or a smaller scale than the clause states; `none` — no test would
  fail if the behavior were deleted.

Line references are at `e2034807`. "driver" abbreviates
`tests/scenario_driver.rs::m1_m13_run_against_the_actor_and_the_judge`,
which runs every M row at φ=0 and φ=1 as `ContractCoverage::Fragment`
(no fragment is `verdict_eligible()`).

## §1. Mock-judged wire scenarios (M1–M13)

*Superseded snapshot at `e2034807` — current coverage is
`src/obligations.rs`; live state is `status.md`.*

### M1 — cold start with residue (`scenarios.md:170`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| One HEAD per touched endpoint, never repeated, never overlapping | M1 (overlap discipline itself is M13/N18) | driver M1 arm (`tests/scenario_driver.rs:238`, one HEAD, len==2); no-repeat-after-rename in driver M5 arm (`:330`); `tests/actor_shell.rs:60` `probe_then_get_share_the_actor_gate_and_keep_distinct_wire_ids` | partial | One endpoint only; multi-endpoint boot serialization untested (see ambiguity §8.5-6) |
| HEAD does not increment counters (N24) | M1; mock mechanics are B5 | `tests/mock_fidelity.rs:211` `b6_b9_residue_and_phantoms…` (residue visible without HEAD increment); `tests/mock_fidelity.rs:42` `b1_b4_b5_b7_b13_b14…` (`!counted` for all five HEADs) | full | Mock-side fact; pinned |
| First-request violation does not occur; HEAD state header is the only prevention | M1 | driver M1 arm (residue=1 preload, G1 green) | partial | Residue magnitude not swept; zero-remaining-budget boundary case pending |
| G1 across the sweep (residue × φ) | M1 → G1 | driver (G1 judged, φ∈{0,1}) | partial | Two phases, one residue value |
| Probe-429 variant: `ProbeReady` seeding (mapping, restriction, generation) | M1 variant; core semantics owned by the disposition suite | `tests/response_disposition.rs:1120` `valid_probe_429_discovers_policy_then_seeds_restriction_and_confirmation` | partial | Core-complete; the wire path (a boot HEAD answered 429 through the actor) has no test |
| Probe-429 variant: 4xx tripwire fed | M1 variant (feed obligation restated by M12) | — (feed exists at `src/actor.rs:624` but no test fails if deleted) | none | See §8.1 item 4 |
| Probe-429 variant: HEAD not requeued | M1 variant; entry-point invariant owned by core | `tests/response_disposition.rs:1018` `probe_outcome_table_is_total_for_non_429_rows`; `tests/response_disposition.rs:697` `entry_point_invariant_holds_across_response_shapes` | full | Core-level; probe lane can never yield `Requeue` |
| Probe-429 variant: first GET is the confirmation attempt, full matrix governs (F6) | M1 variant; matrix owned by core-design | `tests/response_disposition.rs:1170` `probe_opened_episode_permits_the_matrix_final_attempt` | partial | Core-complete; actor wire path pending |

### M2 — clean cold-start saturation burst (`scenarios.md:197`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Burst-then-stall drain shape (N26), padded stalls | M2; padding arithmetic owned by C1 | C1 property (`tests/c1_scheduling.rs:219`) ⊗ driver M2 arm (10 unsaturated GETs) | partial | No judged run has ever saturated a window and stalled; C1 proves the arithmetic, not the wire shape |
| G1 | M2 → G1 | driver | partial | Unsaturated fragment |
| State tracks 1:1 post-increment (N25) | M2; mock emission is B4; client identity is C5/reconciliation | `tests/mock_fidelity.rs:42` (B4) ⊗ `tests/response_reconciliation.rs:469` `ordinary_response_never_double_counts_its_reserved_send` | partial | Both halves proven separately; no wire assert compares them in a run (see ambiguity §8.5-4) |
| G3/G4 over-delay bounds measured here | M2 → G3/G4 | driver (judged at draft ε=500 / 1.05×; M2 minimum = hardcoded 2,550 ms) | partial | G4's N13-padding term has never been non-zero in a judged run (§8.3 item 4); ε undecided (doc finding 12c) |

### M3 — degraded HEAD (`scenarios.md:205`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Header parse returns typed error, no OOB, no empty-policy adoption (N20) | **C2**, not M3 | `tests/c2_headers.rs:68` `missing_headers_are_typed`; `tests/c2_headers.rs:132` `d8_non_full_policies_are_rejected` | full | M3 consumes C2's guarantee; wire seam green in driver M3 arm |
| Endpoint fails cleanly under cooldown (D4) | M3 | `tests/actor_shell.rs:121` `degraded_probe_cools_the_endpoint_and_errors_parked_callers`; driver M3 arm | partial | Cooldown *re-entry* (probe permitted again after 60 s) untested |
| Zero requests sent on that endpoint | M3 | driver M3 arm (`observations.len() == 1`, HEAD only) | full | At fragment scale |
| Other policies unaffected | M3 | — | none | No test drives a second endpoint while one is cooled |
| Pending callers get errors, not hangs | M3 | `tests/actor_shell.rs:121` (parked caller errored) | full | |

### M4 — unexpected policy shape (`scenarios.md:213`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Parse yields `UnexpectedPolicyShape` (one- and three-triplet) | **C2** for typing; B1/B7 for the wire crossing | `tests/c2_headers.rs:41` `rejects_non_pair_shapes` (both counts); `tests/mock_fidelity.rs:690` `b1_b7_m4_synthetic_one_and_three_window_policies_cross_as_raw_headers` | full | Both shapes typed and wire-crossed; only the actor-level three-triplet run is missing, and it is mechanism-identical to the tested one-triplet branch (§8.2 item 5) |
| Scoped per-policy clean failure (D4-style), not app abort | M4 | driver M4 arm (`SetupFailed`) | partial | One-triplet branch only |
| Pending requests errored to callers | M4 (shared D4 drain path) | `tests/actor_shell.rs:121`; `tests/actor_shell.rs:532` (drain via the shared refusal path) | partial | Not exercised with an M4-shaped trigger and a queue behind it |
| Status published on watch channel | M4 | — (watch publication tested only for halt: `tests/actor_shell.rs:222`) | none | No test observes the watch on a D4 cooldown |
| At most one request ever sent under an unknown shape | M4 | driver M4 arm (`observations.len() == 1`) | partial | One-triplet branch |
| Other policies keep flowing | M4 | — | none | Same gap as M3's cross-policy clause |

### M5 — policy rename mid-session (`scenarios.md:224`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Client remaps endpoint, pessimistically merges history | M5 | `tests/response_reconciliation.rs:671` `m5_remaps_an_ordinary_token_without_losing_in_flight_history`; `tests/actor_shell.rs:483` `m5_remap_updates_the_actor_endpoint_mapping`; driver M5 arm (no repeat HEAD) | full | Core + actor + driver seam |
| ≤ in-flight-cap (2) scheduled under stale mapping; organic 429 among them = unavoidable exposure clearing M8 recovery | M5; attribution machinery owned by the judge (§2/B13) | judge unit tests only (`tests/conformance_harness.rs:234`, `:311`) | none | B12 timing script for the forced stale window does not exist; `independently_observable_ms` is never exercised by an integration run (confessed, `scenario-driver-handoff.md:133`) |
| No client-caused violation after the merge | M5 → G1 | driver M5 arm | partial | Without the forced window, G1 is not stressed here |
| Remap triggers beyond reactive are U1 | U1 | — (declared untested) | — | Honored: no proactive-remap code or test claims exist |

### M6 — policy shrink mid-flight (`scenarios.md:236`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Shrink honored from the announcing response | M6 | `tests/actor_shell.rs:621` `m6_shrink_blocks_new_dispatches_from_the_announcing_response` (held 5/5 state; no dispatch before the 120 s padded deadline); `tests/response_reconciliation.rs:719` | full | The driver M6 arm's `replace_policy` is the *mock-side* mutation; the announcement-adoption clause lives in the actor test |
| Mock judging rule: hits are facts, rules are judgments, no grace | B8/B2 (mock side) | `tests/mock_fidelity.rs:265` `b8_policy_rename_and_shrink_keep_existing_hits` | full | |
| Pre-announcement in-flight exposure clears M8 recovery | M6 (same machinery as M5's) | — | none | Same B12/exposure gap as M5 |
| G1 from first post-announcement reservation | M6 → G1 | driver M6 arm | partial | |
| Queue keeps draining at the new pace (no wedge) | M6 | `tests/actor_shell.rs:621` (second caller served after the deadline) | partial | Single queued caller |

### M7 — phantom same-account hits (`scenarios.md:251`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Reconciles pessimistically when observed exceeds model, scope-blind | M7; the arithmetic is core-owned | `tests/response_reconciliation.rs:340` `reconciliation_is_pessimistic_and_synthesizes_exactly_the_maximum_deficit` (4,096 cases, arbitrary deficits); `tests/response_reconciliation.rs:498`; driver M7 arm (wire seam) | full | Core property covers any burst magnitude; the pending "bursty threshold case" adds wire shape, not arithmetic |
| No client-caused violation | M7 → G1 | driver M7 arm | partial | Small injection only |
| Thresholds must not be tuned against constant drizzle | ambiguous — see §8.5 item 3 | — | — | No client-side threshold exists to tune; clause has no test-shaped reading |

### M8 — 429 recovery and escalation ladder (`scenarios.md:260`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Retry waits `Retry-After` + applicable bucket + buffer (N19); applicable bucket = max resolution across windows (F4) | M8; boundary arithmetic core-owned | `tests/actor_shell.rs:412` `m8_429_requeues_through_the_core_not_before_deadline` (≥ 61,000 ms); `tests/response_disposition.rs:208` `restriction_uses_maximum_bucket_and_opens_at_the_exact_boundary`; `tests/response_disposition.rs:596` (Retry-After: 0 exact); driver G3 oracle lower-bounds the retry at +61,000 (`tests/scenario_driver.rs:862`) | full | B3's adversarial restriction expiry makes the assert load-bearing, not decorative |
| Both lanes: OAuth Known and legacy Assumed | M8 | driver (`run_m8_oauth_lane` + legacy in main loop) | full | At fragment scale, both φ |
| Caller eventually observes the outcome (F57) | M8 | driver M8 arm; `tests/actor_shell.rs:412` | full | |
| Second consecutive 429 escalates; never a third knock | M8 | `tests/actor_shell.rs:446` `m8_confirmation_429_escalates_without_a_third_get` (wire-level: exactly 3 handoffs); `tests/response_disposition.rs:519` `every_non_success_final_outcome_escalates` | full | Discharged at actor + core despite the M8 row's "escalation … remain pending" — see §8.2 item 3 |
| ≤ 1 post-restriction reservation in flight; concurrent originals join one episode | M8; episode identity core-owned | `tests/response_disposition.rs:358` `only_one_confirmation_can_be_reserved…`; `tests/response_disposition.rs:151` `arbitrary_generation_tagged_in_flight_sets_join_one_episode` | partial | Core-complete; the B12-scripted *wire* shape (concurrent in-flight originals forced by delays) has no test — this is the genuinely open M8 delta |
| Full confirmation matrix exercised case by case | core-design matrix, exercised under M8 | `tests/response_disposition.rs:374–577` (`valid_2xx_on_first…`, `a_429_on_the_first…`, `unknown_first_outcome…`, `other_non_429_first…`, `every_non_success_final…`) | full | Core-complete; do not rebuild (§8.2 item 3) |
| Malformed 429 → precedence rule 2, D4-style refusal, no retry episode | M8; precedence core-owned (C2 combined cases) | `tests/response_disposition.rs:944`, `:968`; `tests/actor_shell.rs:532` `remap_then_malformed_response_drains_queued_callers…` (wire-level malformed 429 draining a queue) | full | |
| No follow-on violation (§2) | M8 → G1 | driver (G1 armed) | partial | Fragment scale |
| B12 explicit timing script for forced reordering | M8 (per B12) | — | none | Confessed pending; blocks the concurrent-originals clause above |

### M9 — phantom race at saturation (`scenarios.md:283`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Recovery machinery survives the organic-429 race (per M8's asserts) | M9; recovery itself M8/core-owned | driver M9 arm injects a phantom but forces no race | partial | The race (phantom lands between reservation and receipt at 14/15) has never occurred in a test |
| Records what nonzero headroom would have bought per contention level | M9, solely | — | none | No instrument exists anywhere; the headroom-zero decision's evidence base is unstarted (§8.1 item 3) |
| Race exposure attributed via B13 correlation identity (§2) | judge/B13 | `tests/conformance_harness.rs:234` `g1_unavoidable_exposure_is_pre_observation_only_and_capped`; `:311` `correlation_and_reproduction_seams_are_structural` | partial | Machinery fully unit-tested; never exercised by an integration run (`unavoidable_exposure` is always `None` in the driver) |

### M10 — agent-loop stress (`scenarios.md:294`, as amended 2026-08-12)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Spacing floor never violated | M10 | driver M10 arm `paced` (absolute arithmetic over 273 dispatches vs literal 250 ms, HEADs included) | full | |
| In-flight ≤ 2 | M10 | driver M10 arm `capped` | full | |
| G1 holds | M10 → G1 | driver (both φ) | full | At the row's stated scale |
| Fuse does not trip (false-positive absence under saturation) | **C3** owns the property incl. headroom; M10 owns the integration instance; X1 owns true-positive | `src/actor.rs:1205` `c3_floor_compliant_traces_never_trip` (4,096 cases) + `src/actor.rs:1161` `c3_floor_compliant_cadence_holds_the_steady_state_maximum` (exact 4/s, 240/min peaks) ⊗ driver M10 `fuse_quiet` + `paced` ⊗ `src/actor.rs:936` (X1) | full | Composition per the round-four ruling. **Doc finding 11's text still claims this "was untested" — §8.2 item 1** *(since resolved at `77aee08` — see the item's marker)* |
| Queue drains to completion | M10 (named by G5) | driver M10 arm (`served == expected_served`, 270/270) | full | |
| Cancelled callers get prompt resolution (25 ms bound, queued and dispatched) | M10 (Tom's 2026-08-12 amendment) | driver M10 arm (single-poll after exactly one 25 ms tick, 30 queued + 1 mock-proven dispatched; dispatched response later reconciles to in-flight 0) | full | |
| Scale: hundreds of enqueues over many simulated minutes | M10 | driver M10 arm (300 enqueues; `sustained` asserts span ≥ 30 min; actual ≈ 66 min) | full | Recorded span figure stale in two places (§8.3 item 7) |
| (adjacent, unowned) Dropped dispatched `RequestTicket` lifecycle | **no row** — C5 owns core-token abandonment only | — | none | Confessed omitted cell (`scenario-driver-handoff.md:85`); see §8.1 item 2 |

### M11 — layer-1 ceiling and Cloudflare terminal (`scenarios.md:333`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Mock enforces both B10 ceilings tripping into the Cloudflare shape | B10 (mock side), consumed by M11a | `tests/mock_fidelity.rs:356` `b10_b11_layer1_and_injected_stimuli_are_distinct` (burst 20/21 at the wire; sustained 1000/1001 at the model) | full | Rolling-edge convention unpinned — §8.3 item 6 |
| Compliant client never trips either ceiling | M11a sweep *and* G2-armed-everywhere (allocation ambiguous — §8.5 item 2) | G2 judged green in every fragment run, including M10's 66-minute run at both φ | partial | Substantial standing evidence via G2; M11a's dedicated sweep pending |
| Client recognizes the Cloudflare shape generally; halt-shaped terminal, zero retries | M11b; precedence core-owned | `tests/actor_shell.rs:222` `cloudflare_shaped_response_halts_the_gate_and_publishes_status` (2 observations, no retry); `tests/response_disposition.rs:998` `cloudflare_shape_halts_before_status_or_header_handling`; `:615`; driver M11 arm | full | |
| Halt published | M11b | `tests/actor_shell.rs:222`; driver M11 arm (`status.halted`) | full | |
| Pending errored | M11b | driver M11 arm (the one caller errored) | partial | No test halts with a deep queue behind the halt |

### M12 — 4xx-tripwire obligations (`scenarios.md:347`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Injected 401: zero retries | M12 | driver M12 arm (completed, exactly 1 GET); `tests/mock_fidelity.rs:356` (stimulus channel) | full | At fragment scale |
| Generic 4xx: no retry loop | M12; disposition core-owned | `tests/response_disposition.rs:577` `generic_4xx_with_valid_headers_completes_and_reconciles` | partial | Core-complete (only 429 yields `Requeue`); no wire-level generic-4xx run |
| 429: at-most-one-retry-then-escalate (M8's ladder) | **M8** | `tests/actor_shell.rs:446`; `tests/response_disposition.rs:519` | full | Cross-owned; discharged under M8 |
| All 4xx responses feed the tripwire counter | M12 (the *feed*; trip logic is C4's) | — (feed exists at `src/actor.rs:601` and `:624`; no test fails if either call is deleted) | none | §8.1 item 4 — the one genuinely untested M12 clause |
| Trip logic thresholds/edges | **C4**, explicitly ("trip logic itself is C4") | `src/actor.rs:1086` `c4_pins_burst_sustained_and_exact_window_edges` | full | The M12 row's "full tripwire threshold matrix remains pending" claims C4's already-green property — §8.1 item 1 |
| Server-side restriction behavior | U2 | — (declared untested) | — | Honored |

### M13 — gate structure on the wire (`scenarios.md:355`)

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| Never > 2 ordinary in flight | M13 | `tests/actor_shell.rs:347` `m13_ordinary_waiters_are_fifo_and_never_exceed_two_in_flight`; driver M13 + M10 arms (`in_flight_at_arrival <= 2`) | full | |
| HEAD never overlaps any request (N18) | M13 | driver M13 arm (`!head_overlap` with forced 2 s delay); `tests/actor_shell.rs:259` | full | |
| Writer preference: queued HEAD blocks new ordinary permits | M13 | `tests/actor_shell.rs:259` `pending_head_writer_blocks_a_front_get_until_it_runs_exclusively` (forced delayed reader; HEAD hands off before the earlier queued GET) | full | Focused actor test; scenario-assertion integration pending per the M13 row, which correctly cites the composition |
| Ordinary permits in arrival order (FIFO) | M13 | `tests/actor_shell.rs:347` (correlations 2,3,4,5 in order) | full | |
| HEADs are ordinary citizens: spacing floor + fuse-counted | M13; fuse counting owned by C3/X1's boundary | `tests/actor_shell.rs:60` (GET ≥ HEAD + 250 ms); `src/actor.rs:936` (X1 counts at `start_dispatch`, the common HEAD/GET hook); driver M10 `paced` includes HEADs | full | |
| B12 explicit overlap timing script | M13 (per B12) | driver M13 arm (scripted 2 s delay) | full | |

## §2. Core-property scenarios (C1–C5)

*Superseded snapshot at `e2034807` — current coverage is
`src/obligations.rs`; live state is `status.md`.*

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| C1: granted reservation never inside a saturated window on any server φ, full-bucket padding | C1 | `tests/c1_scheduling.rs:219` `every_reserved_outcome_is_safe_for_every_server_phase` (4,096 cases, asserts both branches, independent phase-bucketizing oracle) | full | De-vacuized 2026-08-09 (was ~97% vacuous); `NotBefore` re-ask pinned exact |
| C1: explicit rollover boundary cases | C1 | `tests/c1_scheduling.rs:372`; `:292`; `:341`; `:268` | full | |
| C2: two-triplet increasing periods → `RulePair` | C2 | `tests/c2_headers.rs:30`; `:393` (round-trip property) | full | |
| C2: one-/three-/malformed-triplet → typed errors, never panic/OOB | C2 | `tests/c2_headers.rs:41`; `:371`; `:422` (`malformed_text_never_parses`) | full | |
| C2: missing headers are typed errors, not empty lists | C2 | `tests/c2_headers.rs:68` | full | |
| C2: combined precedence — malformed-429 refusal-shaped, generic 4xx + valid headers parses | C2 (pinning the core-design rule) | `tests/response_disposition.rs:944`; `:577` | full | |
| C3: never trips on any floor-compliant trace (incl. the 2× headroom claim) | C3 | `src/actor.rs:1205` (property, irregular gaps, ≥ 3 trailing minutes, every step asserts); `src/actor.rs:1161` (cadence pin, exact peaks 4 and 240) | full | Lives in `src/`, not `tests/` — the round-four lesson |
| C3: burst boundary — 10 pass, 11th trips, exact 1 s edge excluded (half-open) | C3 | `src/actor.rs:911` `fuse_uses_the_documented_half_open_boundaries`; `src/actor.rs:1058` | full | |
| C3: sustained never fires below 500/min | C3 | `src/actor.rs:1058` (499 safe, 500 trips) | full | |
| C3: trip is latched | C3 | — (latch exists at `src/actor.rs:240`; no test fails if removed) | none | §8.3 item 1 |
| C4: same shape as C3 over a 4xx counter — thresholds and edges | C4 | `src/actor.rs:1086` | full | Thresholds are C3's, adopted as a recorded doc finding (actor slice) |
| C4: shares the fuse's halt semantics | C4 | `src/actor.rs:254–268` guarded by the same `halted` latch; halt path shared with the fuse | partial | Latch untested (as C3); no test drives a wire 4xx across the trip threshold |
| C5: rollback of undispatched reservation restores state exactly | C5 | `tests/c5_lifecycle.rs:163` (property); `:65`; `:92` | full | |
| C5: `UnknownOutcome` stays counted, ages out only by window passage | C5 | `tests/c5_lifecycle.rs:113`; `tests/response_disposition.rs:449` | full | |
| C5: no interleaving double-counts or loses a send | C5 | `tests/c5_lifecycle.rs:189` (observed responses, non-FIFO token order, ≥ 1 op) | full | |
| C5: abandonment — drop bomb + conservatively safe state (both confirmation halves) | C5 | `tests/c5_lifecycle.rs:267`; `:390`; `:422`; `:445` | full | Drop-bomb tests debug-only (129 vs 127), recorded |

## §3. Fault-injection and structural (X1–X2)

*Superseded snapshot at `e2034807` — current coverage is
`src/obligations.rs`; live state is `status.md`.*

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| X1: burst fault trips at the 11th attempt at the transport boundary | X1 | `src/actor.rs:936` `x1_fault_injection_trips_at_the_actor_transport_boundary` (real `start_dispatch` hook, pacing enabled) | full | |
| X1: sustained fault trips at ~500/min while burst stays silent | X1 | `src/actor.rs:936` (second half); `src/actor.rs:1058` | full | |
| X1: on trip — pending deque errored back, `Halted` published | X1 | — (drain + publication tested only via the Cloudflare halt path, `tests/actor_shell.rs:222`; never via a fuse trip) | none | §8.3 item 2; the X1 §3 row reads broader than its test |
| X2: one private HTTP client, no second construction/send path, **pinned by a structural test** | X2 (load-bearing per `result-draft.md` §1) | — (only a doc comment, `src/transport.rs:86`; `WireResponse` bounds tests at `src/actor.rs:957`, `:1034` pin ingress bounds, not path uniqueness) | none | §8.2 item 2 — changelog says the actor slice "carries … X2" while the §3 row is unfilled and no test exists |
| X2 accepted limitation: a future HTTP parser's upstream allocation cap cannot be forced by the spike | X2 (accepted, recorded) | — | — | Honored as recorded (`result-draft.md:668`, actor-handoff §1) |

## §4. Declared-untested register (U1–U4) — exclusions verified

*Superseded snapshot at `e2034807` ("verified" is of that date) —
current coverage is `src/obligations.rs`; live state is `status.md`.*

| Row | Exclusion | Still honored? | Evidence |
|---|---|---|---|
| U1 | Proactive remap triggers out of scope; M5 reactive is the tested surface | yes | No proactive-remap code path or test claim found; M5 tests are all response-reactive |
| U2 | Server-side 4xx restriction untested; obligations only (M12) | yes | Mock models no 4xx-budget threshold; no test claims server behavior |
| U3 | Legacy resolution conditional on `Assumed(60s/60s)`; live instrument not a spike gate | yes | Shipped default structurally required (`tests/conformance_harness.rs:45`); §6 supplemental ledger empty; no run claimed |
| U4 | Real layer-1 rules uncharacterized; B10 inferred-lane | yes | B10 numbers documented as inferred; no external Cloudflare evidence claimed |

## §5. Gates (G1–G6)

*Superseded snapshot at `e2034807` — current coverage is
`src/obligations.rs`; live state is `status.md`.*

| Clause | Owning row | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| G1: zero client-caused violations, follow-on included, exposure excluded per §2 | G1 | judge (`src/conformance.rs:619–633`) exercised by every driver run at both φ; exposure validation `tests/conformance_harness.rs:234`, `:311`; teeth at `:148` | partial | Green across all 13 fragments × 2 φ × (M8 ×2 lanes); no full-contract run; exposure path never used in integration |
| G2: neither B10 ceiling tripped, armed everywhere | G2 | judge (`src/conformance.rs:635`) over every observation of every run, incl. M10's 66-min run; teeth at `tests/conformance_harness.rs:148` | partial | Same fragment qualifier |
| G3: over-delay ≤ ε against padded-safe time, exclusions authorized | G3 | judge (`src/conformance.rs:647`); permit-debt + queueing oracle (`tests/scenario_driver.rs:756–903`); independent boundary pins `:782`; exclusion authorization `tests/conformance_harness.rs:205` | partial | ε = 500 is an unmodelled-padding allowance, not slop — doc finding 12(c), Tom's decision; dispatched-before-eligible also enforced |
| G4: M2 duration ≤ 1.05× padded minimum | G4 | judge boundary pinned exactly (`tests/conformance_harness.rs:266`, 1050/1051); driver M2 judged at hardcoded 2,550 ms minimum | partial | The padding term has never been non-zero in a judged run — §8.3 item 4 |
| G5: every scenario's own assertions, fragments included; unauthorized refusal fails G5 | G5 | judge (`src/conformance.rs:705`); fragment teeth `tests/conformance_harness.rs:177` (forced-false fragment fails G5); driver arms yield computed verdicts | partial | "Unauthorized client-entered refusal state fails G5" has no test (needs a misbehaving client) |
| G6: reproduction record for every failure; (seed, φ) mandatory where swept | G6 | judge hard errors: `MissingReproductionRecord`, `ReproductionMismatch` (`src/conformance.rs:596–607`); `tests/conformance_harness.rs:284`, `:311`; driver records (seed, φ) per swept row | partial | The G6 *gate result* is structurally vacuous (§8.3 item 3); real enforcement is the error path |

The §3 gate-summary rows (`result-draft.md:122–131`) are all unfilled;
see §8.2 item 6 for how that interacts with the per-run evidence above.

## §6. Mock fidelity budget (§7.2 B-series)

*Superseded snapshot at `e2034807` — current coverage is
`src/obligations.rs`; live state is `status.md`.*

| Row | Behavior | Discharging test | Coverage | Notes |
|---|---|---|---|---|
| B1 | Full verbatim header protocol; Retry-After on 429 | `tests/mock_fidelity.rs:42` (parsed by the production parser); `:690` | partial | Everything pinned except the `retry-after` header *string* on an **organic** 429 — model value tested (`:191`), wire emission (`src/mock/mod.rs:837`) never asserted |
| B2 | Black-box counters, organic 429, restriction enforcement | `tests/mock_fidelity.rs:176` `b2_b3_quantized_expiry_and_restriction_are_independent`; `:211` | full | |
| B3 | Server-owned φ, most-adversarial quantization, §1 resolutions | `tests/mock_fidelity.rs:127` (independent-oracle property, 4,096 cases); `src/mock/model.rs:556` | full | CN5 records the model choice |
| B4 | Post-increment 1:1 state | `tests/mock_fidelity.rs:42` | full | |
| B5 | HEAD semantics 204/200 + full headers, non-counting; scriptable degradation | `tests/mock_fidelity.rs:42`; `:211`; `:599` | full | |
| B6 | Pre-loadable counters | `tests/mock_fidelity.rs:211` | full | |
| B7 | N23 five-policy topology, routing, legacy Account+Ip pair, synthetic policies | `tests/mock_fidelity.rs:42` (all five endpoints); `:690`; `src/mock/mod.rs:108` (fixture incl. Account+Ip) | full | |
| B8 | Scripted rename/shrink | `tests/mock_fidelity.rs:265` | full | |
| B9 | Scripted phantom increments, bursty shape | `tests/mock_fidelity.rs:211` | full | Burstiness is a script parameter; injections tested |
| B10 | Two ceilings, armed everywhere, Cloudflare-shaped trip | `tests/mock_fidelity.rs:356` (burst n/n+1 at the wire; sustained n/n+1 at the model); G2 armed in every judged run | full | Rolling-window edge convention unpinned (§8.3 item 6) |
| B11 | Stimulus injection channel (429/401/4xx/Cloudflare/transport-error) | `tests/mock_fidelity.rs:356`; `:599` | full | |
| B12 | Deterministic scripted delay; 50 ms placeholder default; M5/M8/M9/M13 explicit timing scripts | `tests/mock_fidelity.rs:430` (overlap observable); `src/mock/mod.rs:23` (placeholder, honored pending §7.4 fixture) | partial | Mechanism full. Of the four required timing scripts only M13's exists (`tests/scenario_driver.rs:558`); M5/M8/M9 pending — this single gap underlies the M5/M6 exposure, M8 concurrent-originals, and M9 race clauses above |
| B13 | Observation log + handoff-before-delay + correlation identity | `tests/mock_fidelity.rs:484`; `:509` (cancellation survival); `:565` (dropped future ages out); `:430`; `:42` | full | |
| B14 | Zero-skew Date | `tests/mock_fidelity.rs:42`; `:97` | full | |

Bounds hygiene (cross-cutting): `tests/mock_fidelity.rs:750`, `:803`,
`:821`, `:731`, `:547` pin every declared n/n+1 mock bound.

## §7. Out-of-scope register (§7.3 O-series) — exclusions verified

*Superseded snapshot at `e2034807` ("verified" is of that date) —
current coverage is `src/obligations.rs`; live state is `status.md`.*

| Row | Excluded | Still honored? | Evidence |
|---|---|---|---|
| O1 | Sockets/TLS/framing | yes | `Cargo.toml`: `http`, `httpdate`, `tokio` without `net`; no HTTP client/server dependency exists |
| O2 | Stochastic latency | yes | All delays are scripted constants (`ExchangeScript`); `proptest` randomness is test-input only |
| O3 | Payloads | yes | Mock bodies empty except the Cloudflare HTML signature; `Raw` override bodies are script-bounded, not payload modeling |
| O4 | Multi-scope semantics | yes | Both legacy rules judged over the single request stream (B7 note); no per-scope machinery anywhere |
| O5 | Date skew | yes; re-entry trigger armed but not fired | Zero-skew emission tested (B14); C1 row records no server-clock input, so the conditional trigger has not fired |
| O6 | Header case/order variants at the mock | yes | Mock emits canonical lowercase (http-crate types); the domain is C2's (`tests/c2_headers.rs:341`) |
| O7 | Auth of any kind | yes | No token/cookie/authorization material in `src/`, `tests/`, or fixtures; sanitizer is allowlist-based (`tools/sanitize_capture.py:95`) |
| O8 | U2/U4/forum regime/unlimited endpoints | yes | Pointed at their own registers; nothing here claims them |

---

## §8. Discrepancy report

Findings are numbered per the kickoff's five classes. Where a
discrepancy needs a decision, it is recorded and stopped at — nothing
below was fixed by this audit.

### 8.1 Clauses no row owns, or two rows both claim

1. **M12's evidence row claims C4's already-green property as its own
   pending work.** `result-draft.md:96` reads "generic-4xx and full
   tripwire threshold matrix remain pending", but `scenarios.md:352`
   assigns trip logic to C4 explicitly ("trip logic itself is C4"),
   and C4's row (`result-draft.md:112`) is green via
   `src/actor.rs:1086`. This is the identical shape to the M10/C3
   round-four defect — a threshold-matrix obligation filed against an
   M row while the owning C row, in the same table, already
   discharged it. What M12 actually still owes is narrower: the wire
   *feed* (item 4 below) and a generic-4xx wire run.
2. **The dropped dispatched `RequestTicket` lifecycle has no owner.**
   C5 owns core-token abandonment (tested); M10's stimulus is now
   explicit cancellation (tested); the external-review shell
   obligation (`result-draft.md:545–552`) makes shell-side token
   resolution load-bearing. A caller that *drops* its ticket while
   dispatched is confessed untested (`scenario-driver-handoff.md:85–89`)
   but no scenario, C row, or X row owns it — it can stay untested
   forever without any row going red. Needs an owner assigned (Tom).
3. **M9's headroom characterization record is owned but unstarted.**
   `scenarios.md:289–292` makes M9 the sole owner of the
   headroom-zero decision's evidence base; no instrument, no
   recording format, and no hand-off next-step exists for it beyond
   "headroom record remains pending" (`result-draft.md:93`). Not
   double-claimed — flagged because it is the only M-series clause
   whose output is *data*, and nothing will produce it incidentally.
4. **M12's "all 4xx responses feed the tripwire counter" is fed in
   code and pinned by nothing.** Both call sites exist
   (`src/actor.rs:601` ordinary, `src/actor.rs:624` probe); deleting
   either fails no test — C4's green pins the pure counter, not the
   feed. Because C4 is green and M12's row talks about the threshold
   matrix instead, this genuinely untested clause is camouflaged by
   the two rows around it.

### 8.2 Recorded untested in one place, discharged in another (the round-four class)

1. **Doc finding 11 still asserts the fuse false-positive claim "was
   untested."** `result-draft.md:244–249` retains the 15:58 re-scope
   text verbatim, while C3's row (`result-draft.md:111`) and the
   round-four changelog entry (`result-draft.md:1227–1272`) establish
   it was C3-owned and green three hours before that text was
   written. The round-four entry itself lists this correction as
   owed (`result-draft.md:1273–1276`) and it is unfixed at
   `e2034807`. The same owed list is the authority; this audit just
   confirms nothing has drifted further. *[Resolved at `77aee08`
   (doc-split commit 1), 2026-08-12 — marker added 2026-08-13,
   DS-R1.]*
2. **X2 is simultaneously unfilled and claimed carried.** The §3 row
   is `⟨…⟩` (`result-draft.md:120`), while the 2026-08-12 actor-slice
   changelog says the shell "carries C3, C4, X1, and X2"
   (`result-draft.md:961–963`) and `actor-handoff.md` §1 lists the
   ingress-bounds work under X2. No structural test exists: grep
   finds only the doc comment at `src/transport.rs:86`, and
   `scenarios.md:443–444` requires "a structural test pinning it."
   The bounds tests (`src/actor.rs:957`, `:1034`) pin *ingress
   shape*, not *path uniqueness*. Since `result-draft.md` §1 names X2
   load-bearing for the register question itself, this is the most
   consequential single gap in the map: either the changelog
   overclaims, or the row under-records, and either way the required
   test is unbuilt. Decision needed on what a spike-scope X2 test
   even is, given no production transport exists yet (the accepted
   limitation covers the *parser cap*, not the missing test).
3. **M8's row says "escalation/malformed/matrix rows remain pending"
   while all three are discharged elsewhere.** Escalation:
   `tests/actor_shell.rs:446` (wire-level, exactly three handoffs).
   Malformed-429 D4 refusal draining a queue:
   `tests/actor_shell.rs:532`. Confirmation matrix:
   `tests/response_disposition.rs:374–577`, core-complete. The row
   (`result-draft.md:92`) does say "prior focused evidence retained
   below", but the pending-list wording invites rebuilding all
   three. The genuinely open M8 delta is only: B12-scripted
   concurrent in-flight originals joining one episode, and
   single-retry-in-flight observed on the wire.
4. **M1's probe-429 variant "remains pending" while its core asserts
   are discharged.** Seeding: `tests/response_disposition.rs:1120`.
   Matrix-governed confirmation: `:1170`. HEAD-not-requeued: `:1018`
   plus the entry-point invariant sweep. What is actually missing is
   the actor wire path (a boot HEAD answered 429) and the
   tripwire-fed clause — much less than the row's wording implies.
5. **M4's "three-triplet branch remains pending" overstates the
   gap.** Parse typing for both shapes is C2-owned and green
   (`tests/c2_headers.rs:41`), and the three-window shape crosses the
   wire as raw headers in `tests/mock_fidelity.rs:690`. The remaining
   actor-level delta is mechanism-identical to the tested one-triplet
   branch; raising M4 to full contract should cite these, not re-prove
   parse behavior.
6. **The gate summary rows are all unfilled while every gate is
   judged green per fragment run.** `result-draft.md:122–131` vs the
   driver's per-run judgments at both phases
   (`tests/scenario_driver.rs:608–629`). Leaving the verdict-level
   slots empty is correct (fragments are not verdict-eligible), but a
   reader who checks only the gate table finds nothing and may re-run
   or re-derive what already exists. A "partial — fragment evidence,
   see driver status note" marker in those rows would close the seam
   without overclaiming; that is an edit to register text, so it is
   Tom's call, not this audit's.
7. **Stale status lines already owed by round four, still stale.**
   `AGENTS.md:60` says "rounds one and two findings fixed …
   awaiting re-review" (rounds three and four have happened);
   `scenario-driver-handoff.md:3–5` names rounds one–three but not
   round four, whose findings F14–F16 are recorded only inside the
   §9 round-four entry (`result-draft.md:1279–1280`). A session that
   reads the hand-off chain per AGENTS.md's own instructions will not
   learn F14–F16 exist. *[Resolved at `77aee08` and by the doc split
   itself: both status lines are current and `status.md` §2 names
   F14–F16 — marker added 2026-08-13, DS-R1.]*

### 8.3 Discharged claims weaker than their clause

1. **C3 "trip is latched" is untested.** The latch exists
   (`src/actor.rs:240` early-returns when halted); no test advances
   past a trip and re-asks. Removing the latch (recomputing `halted`
   each call) fails nothing today. `scenarios.md:405` states it as a
   property.
2. **X1's row is green but two of its three asserted consequences
   are unproven for the fuse trigger.** `scenarios.md:436–437`
   asserts "halts dispatch; pending deque errored back; `Halted`
   published." The test (`src/actor.rs:936`) pins the trip only.
   Drain-and-publish is tested solely via the Cloudflare halt path
   (`tests/actor_shell.rs:222`); the plumbing is shared
   (`self.halted` gates at `src/actor.rs:369`, `:411`), but that
   composition is asserted by nobody. The X1 row
   (`result-draft.md:119`) reads as if the whole clause is green.
3. **G6's gate result is structurally vacuous.** The judge hard-errors
   on a missing swept reproduction record at `src/conformance.rs:596`
   before `g6_failures` re-checks the identical condition at
   `src/conformance.rs:712`, so the G6 `GateResult` can never be
   `passed: false`. Enforcement is real (the error path is tested,
   `tests/conformance_harness.rs:284`), but any future reader of a
   `RunReport` sees G6 "passed" unconditionally. Cosmetic today;
   misleading the day someone adds a G6 condition the error path
   does not cover.
4. **G4 has never judged a run where its padding term was non-zero.**
   The only integration use hardcodes a 2,550 ms minimum for an
   unsaturated 10-GET fragment (`tests/scenario_driver.rs:602–607`);
   the 1.05× boundary itself is exactly pinned
   (`tests/conformance_harness.rs:266`). §6 describes the minimum as
   "harness-computed, full N13 padding" — no computation with a
   padding contribution exists yet. When M2's saturation depth lands,
   the literal must become real arithmetic or G4 silently measures
   the wrong thing.
5. **B1's Retry-After emission on an organic 429 is untested at the
   wire.** The model's `retry_after_seconds` is pinned
   (`tests/mock_fidelity.rs:191`), and the header insertion exists
   (`src/mock/mod.rs:837–841`), but no test parses `retry-after` off
   an organic wire 429. Scripted 429s (M8) supply their own value, so
   they don't cover it.
6. **B10's rolling-window edge convention is unpinned.** The trip
   comparison is `count > limit` (`src/mock/model.rs:371`) and the
   n/n+1 tests fire all arrivals at one instant
   (`tests/mock_fidelity.rs:359–381`); whether an arrival exactly
   1 s / 60 s old has left the layer-1 window is asserted nowhere.
   C3 pins its own half-open convention explicitly; B10 — the mock
   mirror of the same idea — does not. Same genus as CN5.
7. **M10's recorded span figure is stale in two places.**
   `result-draft.md:254` and `:1107` say 3,963,250 ms; round four
   measured 3,963,500 ms after the 250→25 ms step change and lists
   the correction as owed (`result-draft.md:1278`). Trivial, but it
   is a number in an evidence register. *[Resolved at `77aee08` —
   both places corrected; marker added 2026-08-13, DS-R1.]*

### 8.4 Evidence citing tests that no longer exist

**None found.** Every test name cited in `result-draft.md` §3 rows,
the §9 changelog, and all five hand-offs resolves to an existing test
at `e2034807` (C3's five names, C4's, X1's, the driver targets,
`swept_phases_are_separated_by_a_full_bucket`,
`a_fragment_run_is_judged_but_is_never_verdict_eligible`, the B3
property, the sanitizer suite). See also §9 below for the count
cross-check.

### 8.5 Requirement text two readers could allocate differently

M10's "saturation" is the known instance (resolved by the round-four
composition ruling). Flagged, not resolved:

1. **G3's "padded-safe time"** — already doc finding 12(c), Tom's
   open decision; listed for completeness because it is the live
   instance of the class.
2. **M11a's "the compliant client — … — never trips it"**
   (`scenarios.md:341–342`) vs G2 "armed in every mock-judged
   scenario" (`scenarios.md:489–492`). One reader files the
   never-trips obligation under M11's dedicated sweep; another says
   G2-everywhere already carries it (M10's 66-minute run is real
   evidence). The M11 row currently calls the sweep "pending" while
   G2 evidence accrues silently — the same divergence shape that
   produced the M10/C3 defect. Needs an ownership sentence, either
   way.
3. **M7's "Thresholds must not be tuned against a constant-drizzle
   world"** (`scenarios.md:256–258`). The client has no threshold to
   tune; the sentence could bind the client's reconciliation
   constants, the mock's B9 script shape, or the future scenario
   author. No test-shaped reading exists.
4. **M2's "state tracks 1:1 post-increment (N25)"**
   (`scenarios.md:202`). Readable as a mock-fidelity fact (B4, tested)
   or as a client-model assert (client history equals server counters
   during the run — no instrument exists). Two readers would build
   two different tests.
5. **G4's "harness-computed"** (`scenarios.md:521`). Whether a
   script-derived hardcoded literal qualifies is arguable today and
   stops being arguable the moment padding enters (8.3 item 4).
6. **M1's "strictly serialized" boot HEADs** (`scenarios.md:173–174`).
   Per-endpoint (each HEAD before that endpoint's first GET) or
   globally serialized across all five endpoints? M13 owns HEAD
   exclusivity generally, so the multi-endpoint boot ordering could be
   filed under either row.
7. **C3's "~500"** (`scenarios.md:392`) vs the exact 499/500 pin. The
   implementation chose exact 500 and pinned it; the tilde survives
   in the contract and a later reader could take it as licence to
   retune. One word ("exactly") would close it; that is a
   `scenarios.md` edit, so Tom's.

## §9. Checks that came back clean

- **No dangling test citations** (8.4). Additionally the arithmetic
  cross-checks: the latest gate matrix claims 129 debug / 127 release
  tests; independent extraction of every `#[test]`/`#[tokio::test]`
  function across `src/` and `tests/` finds exactly 129, of which
  exactly 2 are the debug-only drop-bomb tests — 127 in release.
  The four Python sanitizer tests are counted separately, as the
  matrix does.
- **U1–U4 exclusions all honored** (§4): nothing tests or claims the
  declared-untested surfaces.
- **O1–O8 exclusions all honored** (§7): no sockets, no stochastic
  timing, no payload modeling, no scope semantics, no skew, no
  adversarial casing at the mock, no credentials, and O8's pointers
  intact.
- **The fragment guard holds end to end**: every driver row declares
  `Fragment`, `verdict_eligible()` requires full coverage plus a pass
  (`src/conformance.rs:480`), the driver asserts no report is
  eligible (`tests/scenario_driver.rs:626`), and the property has
  its own non-vacuous test (`tests/conformance_harness.rs:177`).
  No verdict slot is filled anywhere, matching AGENTS.md's standing
  instruction.
- **Ownership of the round-four composition is now structural in the
  right place**: C3's row text, the strengthened property, and
  `slice-review.md` lesson 4 agree; only the trailing doc-finding-11
  text (8.2 item 1) lags, and it is already on the owed list.
