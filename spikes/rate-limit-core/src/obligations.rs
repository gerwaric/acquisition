//! Machine-checked clause registry.
//!
//! Every assertion clause of the spike's contract appears here exactly
//! once, with its owning `scenarios.md` row, its audited coverage
//! state, and the tests that discharge it. Migrated row-for-row from
//! `obligation-map.md` (audit at `f03632b9`, code at `e2034807`) per
//! the accepted `clause-registry-design.md`; the map is superseded by
//! this table. `tests/obligations.rs` verifies the table's internal
//! rules and that every cited test function exists in source.
//!
//! Rust note (this spike doubles as a Rust introduction): this is a
//! `const` table of plain structs, not an enum — nothing dispatches on
//! clauses at runtime, so exhaustiveness comes from the verification
//! test rather than the compiler, and adding a clause stays a
//! one-entry diff. `&'static str`/`&'static [..]` mean the data lives
//! in the compiled binary itself; no allocation ever happens.

/// One discharging test cited for a clause.
///
/// `file` is a path from the crate root; `test_fn` is the exact
/// function name (cite by name, verify by content — line numbers
/// drift). `must_assert` is the reviewer's hook against silent
/// weakening: the one thing to check the test still asserts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Citation {
    pub file: &'static str,
    pub test_fn: &'static str,
    pub must_assert: &'static str,
}

/// Audited coverage state of a clause, per `obligation-map.md` §0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    /// A cited test fails if the clause is broken, at the clause's
    /// stated scope.
    Full,
    /// Citations exist but cover a fragment, one shape, one lane, or
    /// a smaller scale; the note states the exact missing delta.
    Partial,
    /// No test would fail if the behavior were deleted. Zero
    /// citations; the note names a disposition (a register decision,
    /// or the literal `open — flagged for Tom`).
    Untested,
    /// U-/O-series exclusion. Zero citations — a citation on an
    /// excluded clause fails verification (exclusion drift is the
    /// failure this state exists to catch).
    Excluded,
}

/// One assertion clause of the contract.
pub struct Clause {
    /// Stable kebab ID, namespaced by the map section the row lives
    /// in (`m12-tripwire-feed`, `c3-trip-latched`).
    pub id: &'static str,
    /// The owning `scenarios.md` row, resolved by the audit — not
    /// necessarily the scenario whose table the clause appears in
    /// (lesson 4: M3's parse clause is C2's, M12's trip logic is
    /// C4's).
    pub owner: &'static str,
    /// One-line clause text with its `scenarios.md` anchor.
    pub text: &'static str,
    pub coverage: Coverage,
    pub citations: &'static [Citation],
    /// Delta (Partial), reason and disposition (Untested), or
    /// provenance notes.
    pub note: &'static str,
}

/// The driver test: runs every M row at φ=0 and φ=1 as
/// `ContractCoverage::Fragment`. Cited per-arm below.
const DRIVER: &str = "tests/scenario_driver.rs";
const DRIVER_FN: &str = "m1_m13_run_against_the_actor_and_the_judge";

/// Exactly the `Untested` clauses whose disposition is
/// `open — flagged for Tom`. Changing any clause's coverage state
/// must touch this list too, so every state transition is a
/// deliberate two-line diff (the `swept_phases` pattern).
pub const OPEN_UNTESTED: &[&str] = &[
    // Empty: every formerly open Untested clause now has a discharging test.
    // This does not imply verdict readiness: the seven full-contract Partial
    // clauses remain live work in status.md.
];

pub const CLAUSES: &[Clause] = &[
    // ── M1 — cold start with residue (scenarios.md:170) ──────────
    Clause {
        id: "m1-boot-head-discipline",
        owner: "M1",
        text: "One boot HEAD per touched endpoint, never repeated, never \
               overlapping; overlap discipline itself is M13/N18 \
               (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M1 arm: exactly one HEAD, observations len == 2; \
                              M5 arm: no repeat HEAD after rename",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "probe_then_get_share_the_actor_gate_and_keep_distinct_wire_ids",
                must_assert: "probe and GET share the actor gate with distinct wire ids",
            },
            Citation {
                file: "tests/actor_safety.rs",
                test_fn: "degraded_probe_cools_one_endpoint_while_another_policy_keeps_flowing",
                must_assert: "two endpoint records bootstrap independently; the healthy endpoint has exactly one HEAD before GET",
            },
        ],
        note: "Two-endpoint rig landed 2026-08-14; global HEAD exclusivity remains M13-owned",
    },
    Clause {
        id: "m1-head-does-not-count",
        owner: "M1",
        text: "HEAD does not increment counters (N24); mock mechanics are B5 \
               (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b6_b9_residue_and_phantoms_are_mock_owned_counter_facts",
                must_assert: "residue visible without HEAD increment",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
                must_assert: "!counted for all five HEADs",
            },
        ],
        note: "Mock-side fact; pinned",
    },
    Clause {
        id: "m1-no-first-request-violation",
        owner: "M1",
        text: "First-request violation does not occur; the HEAD state header \
               is the only prevention (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M1 residue sweep: residues 0/1/9/10 at phi 0/1; zero-budget opening GET waits and G1 stays green",
        }],
        note: "Residue magnitude and zero-remaining-budget boundary swept",
    },
    Clause {
        id: "m1-g1-sweep",
        owner: "M1",
        text: "G1 across the residue × φ sweep (M1 → G1) (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "G1 judged over residues 0/1/9/10 at φ ∈ {0, 1}",
            },
            Citation {
                file: "tests/m1_residue_sweep.rs",
                test_fn: "m1_generated_phi_mock_side_residue_sweep_holds_g1",
                must_assert: "generated residue 0..=12 × generated φ over the \
                              60,000 ms cycle (plus pinned 4,999/5,000/5,001 \
                              rollover arms) boots the public actor and is \
                              judged by the mock with G1 armed; the \
                              sustained-window residue count anchors \
                              non-vacuity and zero-budget cases wait the \
                              independent 20 s bound under G3",
            },
            Citation {
                file: "tests/c1_scheduling.rs",
                test_fn: "every_reserved_outcome_is_safe_for_every_server_phase",
                must_assert: "generated-φ safety of every granted reservation \
                              (the §3 sweep property C1 owns — the core-side \
                              mirror, kept as supporting evidence only)",
            },
        ],
        note: "Discharged 2026-08-14: the generated-φ mock-side residue sweep \
               closes RE-9's delta. The driver's residues 0/1/9/10 at \
               φ ∈ {0, 1} remain the pinned boundary cases; the sweep caps \
               residue at 12 (above the burst limit, strictly below the \
               sustained 30) so the zero-budget bound is always the burst \
               window's — recorded in the hand-off",
    },
    Clause {
        id: "m1-probe-429-seeding",
        owner: "M1",
        text: "Probe-429 variant: ProbeReady seeding (mapping, restriction, \
               generation); core semantics owned by the disposition suite \
               (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "valid_probe_429_discovers_policy_then_seeds_restriction_and_confirmation",
                must_assert: "probe 429 discovers the policy, seeds restriction and confirmation state",
            },
            Citation {
                file: "tests/actor_safety.rs",
                test_fn: "m1_probe_429_through_actor_seeds_then_first_get_confirms_and_escalates",
                must_assert: "wire HEAD 429 establishes mapping/restriction and releases the first GET only after 61 seconds",
            },
        ],
        note: "Core semantics and actor wire path compose",
    },
    Clause {
        id: "m1-probe-429-tripwire-feed",
        owner: "M1",
        text: "Probe-429 variant: 4xx tripwire fed; feed obligation restated \
               by M12 (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "src/actor.rs",
            test_fn: "c4_probe_feed_trips_shared_halt_and_drains_and_publishes",
            must_assert: "the actual finish_probe 429 feed supplies the 11th 4xx and trips halt",
        }],
        note: "Internal fault injection is required because intact D5 pacing cannot reach C4 thresholds through public traffic",
    },
    Clause {
        id: "m1-probe-429-head-not-requeued",
        owner: "M1",
        text: "Probe-429 variant: the HEAD is not requeued; entry-point \
               invariant owned by core (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "probe_outcome_table_is_total_for_non_429_rows",
                must_assert: "probe outcome table is total; no Requeue row",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "entry_point_invariant_holds_across_response_shapes",
                must_assert: "on_probe_response never yields CompleteRequest \
                              or Requeue",
            },
        ],
        note: "Core-level; the probe lane can never yield Requeue",
    },
    Clause {
        id: "m1-probe-429-first-get-confirmation",
        owner: "M1",
        text: "Probe-429 variant: the first GET is the confirmation attempt, \
               full matrix governs (F6, external design review register); \
               matrix owned by core-design (scenarios.md:170)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "probe_opened_episode_permits_the_matrix_final_attempt",
                must_assert: "a probe-opened episode is governed by the confirmation matrix",
            },
            Citation {
                file: "tests/actor_safety.rs",
                test_fn: "m1_probe_429_through_actor_seeds_then_first_get_confirms_and_escalates",
                must_assert: "the first actor GET after probe 429 is the confirmation and a 429 earns no third knock",
            },
        ],
        note: "Core matrix and actor wire path compose",
    },
    // ── M2 — clean cold-start saturation burst (scenarios.md:197) ─
    Clause {
        id: "m2-burst-stall-drain",
        owner: "M2",
        text: "Burst-then-stall drain shape (N26) with padded stalls; padding \
               arithmetic owned by C1 (scenarios.md:197)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c1_scheduling.rs",
                test_fn: "every_reserved_outcome_is_safe_for_every_server_phase",
                must_assert: "granted reservations never land in a saturated \
                              window for any server phase",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M2 arm: 40 GETs force three padded stalls, including the sustained-window stall, then drain",
            },
        ],
        note: "C1 arithmetic and the judged 40-request wire shape compose",
    },
    Clause {
        id: "m2-g1",
        owner: "M2",
        text: "G1 (M2 → G1) (scenarios.md:197)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "40-request saturation arm judged with G1 armed at phi 0/1",
        }],
        note: "Saturation-depth G1 evidence",
    },
    Clause {
        id: "m2-state-tracks-post-increment",
        owner: "B4",
        text: "State tracks 1:1 post-increment (N25) — the mock's fidelity \
               obligation, owned by B4 per the 2026-08-13 amendment; client \
               identity is C5/reconciliation (scenarios.md:197, :209)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
                must_assert: "B4 post-increment state emission",
            },
            Citation {
                file: "tests/response_reconciliation.rs",
                test_fn: "ordinary_response_never_double_counts_its_reserved_send",
                must_assert: "client identity: the reserved send is not \
                              double-counted on reconciliation",
            },
        ],
        note: "Re-owned M2→B4 and flipped Full at the 2026-08-13 ballot \
               pass, applying the decisions-pass amendment \
               (scenarios.md:209): no client-side counter instrument owed; \
               the cited B4 emission and reconciliation-identity tests \
               discharge the clause at scope",
    },
    Clause {
        id: "m2-g3-g4-bounds",
        owner: "M2",
        text: "G3/G4 over-delay and duration bounds measured here (M2 → \
               G3/G4) (scenarios.md:197)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M2 judged at ε=500 / 1.05× using a runtime minimum derived from the policy windows and queue depth",
        }],
        note: "The 40-request run reaches burst and sustained padding; the independent minimum is runtime-derived, never a literal",
    },
    // ── M3 — degraded HEAD (scenarios.md:205) ────────────────────
    Clause {
        id: "m3-header-parse-typed",
        owner: "C2",
        text: "Header parse returns typed error, no OOB, no empty-policy \
               adoption (N20) — owned by C2, not M3 (scenarios.md:205)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "missing_headers_are_typed",
                must_assert: "missing headers yield typed errors, not empty \
                              lists",
            },
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "d8_non_full_policies_are_rejected",
                must_assert: "non-full policies are rejected, never adopted \
                              empty",
            },
        ],
        note: "M3 consumes C2's guarantee; wire seam green in the driver M3 \
               arm",
    },
    Clause {
        id: "m3-cooldown-clean-failure",
        owner: "M3",
        text: "Endpoint fails cleanly under cooldown (D4) (scenarios.md:205)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "degraded_probe_cools_the_endpoint_and_errors_parked_callers",
                must_assert: "degraded probe cools the endpoint",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M3 arm: degraded HEAD yields clean failure",
            },
            Citation {
                file: "tests/actor_safety.rs",
                test_fn: "degraded_probe_cools_one_endpoint_while_another_policy_keeps_flowing",
                must_assert: "the endpoint refuses during cooldown and successfully probes/serves again after 60 seconds",
            },
        ],
        note: "D4 cooldown refusal and finite 60-second re-entry are both pinned",
    },
    Clause {
        id: "m3-zero-requests-sent",
        owner: "M3",
        text: "Zero requests sent on that endpoint (scenarios.md:205)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M3 arm: observations.len() == 1, HEAD only",
        }],
        note: "At fragment scale",
    },
    Clause {
        id: "m3-other-policies-unaffected",
        owner: "M3",
        text: "Other policies unaffected (scenarios.md:205)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/actor_safety.rs",
            test_fn: "degraded_probe_cools_one_endpoint_while_another_policy_keeps_flowing",
            must_assert: "Stash HEAD+GET succeeds while StashList is cooled",
        }],
        note: "Two-endpoint scoped-cooldown rig",
    },
    Clause {
        id: "m3-pending-callers-errored",
        owner: "M3",
        text: "Pending callers get errors, not hangs (scenarios.md:205)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/actor_shell.rs",
            test_fn: "degraded_probe_cools_the_endpoint_and_errors_parked_callers",
            must_assert: "the parked caller receives an error",
        }],
        note: "",
    },
    // ── M4 — unexpected policy shape (scenarios.md:213) ──────────
    Clause {
        id: "m4-unexpected-shape-typed",
        owner: "C2",
        text: "Parse yields UnexpectedPolicyShape for one- and three-triplet \
               shapes — C2 for typing; B1/B7 for the wire crossing \
               (scenarios.md:213)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "rejects_non_pair_shapes",
                must_assert: "one- and three-triplet counts are both typed \
                              rejections",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b7_m4_synthetic_one_and_three_window_policies_cross_as_raw_headers",
                must_assert: "both synthetic shapes cross the wire as raw \
                              headers",
            },
        ],
        note: "Both shapes typed and wire-crossed; only the actor-level \
               three-triplet run is missing, mechanism-identical to the \
               tested one-triplet branch (map §8.2 item 5)",
    },
    Clause {
        id: "m4-scoped-clean-failure",
        owner: "M4",
        text: "Scoped per-policy clean failure (D4-style), not app abort \
               (scenarios.md:213)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M4 arm: SetupFailed, no abort",
        }],
        note: "Full per the 2026-08-13 ballot amendment (scenarios.md M4): \
               the one-triplet branch is the actor-exercised representative; \
               both shapes typed and wire-crossed under \
               m4-unexpected-shape-typed's citations",
    },
    Clause {
        id: "m4-pending-errored",
        owner: "M4",
        text: "Pending requests errored to callers via the shared D4 drain \
               path (scenarios.md:213)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "degraded_probe_cools_the_endpoint_and_errors_parked_callers",
                must_assert: "parked callers are errored on cooldown",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "remap_then_malformed_response_drains_queued_callers_for_the_current_policy",
                must_assert: "queued callers drain via the shared refusal path",
            },
        ],
        note: "Full per the 2026-08-13 ballot amendment (scenarios.md M4): \
               discharged by the shared D4 drain path — cooldown and \
               refusal-drain cited; M4 trigger provenance pinned by C2's \
               typing tests",
    },
    Clause {
        id: "m4-watch-status-published",
        owner: "M4",
        text: "Status published on the watch channel (scenarios.md:213)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/actor_safety.rs",
            test_fn: "unexpected_shape_cooldown_is_published_on_the_watch_channel",
            must_assert: "watch changes from probing to a non-halted drained cooldown snapshot",
        }],
        note: "SD-R5-F15: `GateStatus` carries no cooldown representation, so \
               what is observed is the drained non-halted post-cooldown \
               snapshot, not a cooldown field; what M4's published status \
               must contain is a recorded doc silence (result-draft.md, \
               2026-08-14 repair entry) with the conservative reading that \
               this snapshot discharges the clause",
    },
    Clause {
        id: "m4-at-most-one-request",
        owner: "M4",
        text: "At most one request ever sent under an unknown shape \
               (scenarios.md:213)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M4 arm: observations.len() == 1",
        }],
        note: "Full per the 2026-08-13 ballot amendment (scenarios.md M4): \
               one-triplet branch as the actor-exercised representative",
    },
    Clause {
        id: "m4-other-policies-flowing",
        owner: "M4",
        text: "Other policies keep flowing (scenarios.md:213)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/actor_safety.rs",
            test_fn: "unexpected_shape_cools_one_endpoint_while_another_policy_keeps_flowing",
            must_assert: "unexpected-shape endpoint fails while the other policy completes HEAD+GET",
        }],
        note: "M4-specific two-endpoint rig",
    },
    // ── M5 — policy rename mid-session (scenarios.md:224) ────────
    Clause {
        id: "m5-remap-pessimistic-merge",
        owner: "M5",
        text: "Client remaps the endpoint and pessimistically merges history \
               (scenarios.md:224)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_reconciliation.rs",
                test_fn: "m5_remaps_an_ordinary_token_without_losing_in_flight_history",
                must_assert: "remap loses no in-flight history",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m5_remap_updates_the_actor_endpoint_mapping",
                must_assert: "the actor endpoint mapping follows the rename",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M5 arm: no repeat HEAD after the rename",
            },
        ],
        note: "Core + actor + driver seam",
    },
    Clause {
        id: "m5-stale-window-exposure",
        owner: "M5",
        text: "≤ in-flight-cap (2) scheduled under the stale mapping; an \
               organic 429 among them is unavoidable exposure clearing M8 \
               recovery; attribution machinery owned by the judge (§2/B13) \
               (scenarios.md:224)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/transition_timing.rs",
            test_fn: "m5_forced_stale_mapping_window_caps_exposure_and_stays_safe_after_merge",
            must_assert: "exactly the D5-cap stale handoffs precede the \
                          observable rename; the queued post-merge request is \
                          not attributed to that window",
        }],
        note: "Forced at both boundary phases (phi=0 and phi=1); the test \
               exercises the observable stale window and independent exposure \
               attribution without declaring full-contract coverage",
    },
    Clause {
        id: "m5-no-violation-after-merge",
        owner: "M5",
        text: "No client-caused violation after the merge (M5 → G1) \
               (scenarios.md:224)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M5 arm judged with G1 armed",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m5_forced_stale_mapping_window_caps_exposure_and_stays_safe_after_merge",
                must_assert: "post-merge dispatch waits for both independent \
                              windows and produces no organic violation",
            },
        ],
        note: "The focused timing test supplies the forced stale-window shape \
               that the fragment driver arm cannot reach",
    },
    // M5's "remap triggers beyond reactive are U1" row is the U1
    // exclusion itself — collapsed into `u1-proactive-remap`
    // (recorded in the slice hand-off).

    // ── M6 — policy shrink mid-flight (scenarios.md:236) ─────────
    Clause {
        id: "m6-shrink-honored-from-announcement",
        owner: "M6",
        text: "Shrink honored from the announcing response \
               (scenarios.md:236)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m6_shrink_blocks_new_dispatches_from_the_announcing_response",
                must_assert: "held 5/5 state; no dispatch before the 120 s \
                              padded deadline",
            },
            Citation {
                file: "tests/response_reconciliation.rs",
                test_fn: "m6_replaces_rules_immediately_while_retaining_history_facts",
                must_assert: "rules replaced immediately, history facts kept",
            },
        ],
        note: "The driver M6 arm's replace_policy is the mock-side mutation; \
               the announcement-adoption clause lives in the actor test",
    },
    Clause {
        id: "m6-mock-hits-are-facts",
        owner: "B8",
        text: "Mock judging rule: hits are facts, rules are judgments, no \
               grace — B8/B2 mock side (scenarios.md:236)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/mock_fidelity.rs",
            test_fn: "b8_policy_rename_and_shrink_keep_existing_hits",
            must_assert: "rename/shrink keep existing hits as facts",
        }],
        note: "",
    },
    Clause {
        id: "m6-preannouncement-exposure",
        owner: "M6",
        text: "Pre-announcement in-flight exposure clears M8 recovery (same \
               machinery as M5's) (scenarios.md:236)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/transition_timing.rs",
            test_fn: "m6_forced_preannouncement_original_recovers_at_the_shrunk_pace",
            must_assert: "the sole organic 429 was reserved before the shrink, \
                          arrived after it became observable, and is accepted \
                          only by the bounded ExposureAllowance",
        }],
        note: "Forced at phi=0 and phi=1 with a fail-closed independent \
               scenario oracle; the resulting report is explicitly a \
               non-verdict fragment",
    },
    Clause {
        id: "m6-g1-post-announcement",
        owner: "M6",
        text: "G1 from the first post-announcement reservation (M6 → G1) \
               (scenarios.md:236)",
        coverage: Coverage::Partial,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M6 arm judged with G1 armed",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m6_forced_preannouncement_original_recovers_at_the_shrunk_pace",
                must_assert: "G1 passes from the observable shrink while only \
                              the pre-announcement handoff is excluded",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m6_fragment_verdict_is_not_constant_true",
                must_assert: "each of the four wire-derived M6 fragment facts \
                              can make the scenario assertion false instead \
                              of being panic-asserted before judge",
            },
        ],
        note: "Focused transition evidence is stronger than the driver arm; \
               the full-contract scale run remains the closure delta",
    },
    Clause {
        id: "m6-queue-drains-new-pace",
        owner: "M6",
        text: "Queue keeps draining at the new pace — no wedge \
               (scenarios.md:236)",
        coverage: Coverage::Partial,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m6_shrink_blocks_new_dispatches_from_the_announcing_response",
                must_assert: "the second caller is served after the deadline",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m6_forced_preannouncement_original_recovers_at_the_shrunk_pace",
                must_assert: "the exposed request retries only after the wire \
                              Retry-After and independent bucket padding",
            },
        ],
        note: "Focused evidence covers the transition and retry; full-contract \
               queue scale remains the closure delta",
    },
    // ── M7 — phantom same-account hits (scenarios.md:251) ────────
    Clause {
        id: "m7-pessimistic-reconciliation",
        owner: "M7",
        text: "Reconciles pessimistically when observed exceeds model, \
               scope-blind; the arithmetic is core-owned (scenarios.md:251)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_reconciliation.rs",
                test_fn: "reconciliation_is_pessimistic_and_synthesizes_exactly_the_maximum_deficit",
                must_assert: "4,096 cases, arbitrary deficits, exact maximum \
                              synthesis",
            },
            Citation {
                file: "tests/response_reconciliation.rs",
                test_fn: "ordinary_response_synthesizes_only_the_phantom_deficit",
                must_assert: "only the phantom deficit is synthesized",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M7 arm: phantom injection reconciled at the \
                              wire seam",
            },
        ],
        note: "Core property covers any burst magnitude; the pending \"bursty \
               threshold case\" adds wire shape, not arithmetic",
    },
    Clause {
        id: "m7-no-client-violation",
        owner: "M7",
        text: "No client-caused violation (M7 → G1) (scenarios.md:251)",
        coverage: Coverage::Partial,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M7 arm judged with G1 armed",
        }],
        note: "Small injection only",
    },
    // m7-threshold-tuning was removed 2026-08-13: Tom deleted the
    // contract sentence (scenarios.md M7 amendment) — it named no
    // testable obligation, resolving map §8.5 item 3 by wording, as
    // REG-R1-F2 predicted the id might.
    // ── M8 — 429 recovery and escalation ladder (scenarios.md:260) ─
    Clause {
        id: "m8-retry-wait-arithmetic",
        owner: "M8",
        text: "Retry waits Retry-After + applicable bucket + buffer (N19); \
               applicable bucket = max resolution across windows (F4, \
               external design review register); boundary arithmetic \
               core-owned (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m8_429_requeues_through_the_core_not_before_deadline",
                must_assert: "requeue waits ≥ 61,000 ms",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "restriction_uses_maximum_bucket_and_opens_at_the_exact_boundary",
                must_assert: "maximum bucket resolution; exact boundary open",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "retry_after_zero_restricts_for_exactly_bucket_plus_buffer",
                must_assert: "Retry-After: 0 restricts exactly bucket + buffer",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "G3 oracle lower-bounds the retry at +61,000 ms",
            },
        ],
        note: "B3's adversarial restriction expiry makes the assert \
               load-bearing, not decorative",
    },
    Clause {
        id: "m8-both-lanes",
        owner: "M8",
        text: "Both lanes: OAuth Known and legacy Assumed (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "run_m8_oauth_lane plus the legacy lane in the main \
                          loop, both φ",
        }],
        note: "At fragment scale, both φ",
    },
    Clause {
        id: "m8-caller-observes-outcome",
        owner: "M8",
        text: "Caller eventually observes the outcome (F57, \
               docs/cleanup/findings.md register) (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M8 arm: caller completes with the outcome",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m8_429_requeues_through_the_core_not_before_deadline",
                must_assert: "the caller receives the retried outcome",
            },
        ],
        note: "",
    },
    Clause {
        id: "m8-second-429-escalates",
        owner: "M8",
        text: "Second consecutive 429 escalates; never a third knock \
               (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m8_confirmation_429_escalates_without_a_third_get",
                must_assert: "wire-level: exactly 3 handoffs, no third GET",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "every_non_success_final_outcome_escalates",
                must_assert: "every non-success final outcome escalates",
            },
        ],
        note: "Discharged at actor + core despite the M8 evidence row's \
               \"escalation … remain pending\" wording — map §8.2 item 3",
    },
    Clause {
        id: "m8-single-retry-in-flight",
        owner: "M8",
        text: "≤ 1 post-restriction reservation in flight; concurrent \
               originals join one episode; episode identity core-owned \
               (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "only_one_confirmation_can_be_reserved_and_rollback_reopens_the_attempt",
                must_assert: "a second confirmation cannot be reserved while \
                              one is in flight",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m8_forced_concurrent_originals_allow_only_one_confirmation_in_flight",
                must_assert: "two delayed originals overlap, only one \
                              confirmation is published in flight, and the \
                              sibling cannot dispatch until it completes",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "arbitrary_generation_tagged_in_flight_sets_join_one_episode",
                must_assert: "arbitrary in-flight sets join one episode",
            },
        ],
        note: "Core and forced B12 wire shape are both pinned; the timing test \
               is a focused fragment, not a verdict-producing run",
    },
    Clause {
        id: "m8-confirmation-matrix",
        owner: "M8",
        text: "Full confirmation matrix exercised case by case; the matrix is \
               core-design's, exercised under M8 (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "valid_2xx_on_first_confirmation_resets_the_episode",
                must_assert: "2xx on first confirmation resets",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "a_429_on_the_first_confirmation_escalates_even_after_a_late_original_429",
                must_assert: "429 on first confirmation escalates despite a \
                              late original 429",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "unknown_first_outcome_stays_counted_and_permits_a_successful_final_attempt",
                must_assert: "unknown first outcome stays counted, final \
                              attempt permitted",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "other_non_429_first_outcomes_preserve_the_episode_for_one_final_attempt",
                must_assert: "other non-429 first outcomes preserve the \
                              episode",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "every_non_success_final_outcome_escalates",
                must_assert: "every non-success final outcome escalates",
            },
        ],
        note: "Core-complete; do not rebuild (map §8.2 item 3)",
    },
    Clause {
        id: "m8-malformed-429-refusal",
        owner: "M8",
        text: "Malformed 429 → precedence rule 2, D4-style refusal, no retry \
               episode; precedence core-owned (C2 combined cases) \
               (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "malformed_429_takes_precedence_and_never_opens_a_retry_episode",
                must_assert: "malformed 429 never opens a retry episode",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "malformed_429_on_first_confirmation_uses_refusal_path_and_permits_final_attempt",
                must_assert: "refusal path on first confirmation; final \
                              attempt permitted",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "remap_then_malformed_response_drains_queued_callers_for_the_current_policy",
                must_assert: "wire-level malformed 429 drains a queue",
            },
        ],
        note: "",
    },
    Clause {
        id: "m8-no-follow-on-violation",
        owner: "M8",
        text: "No follow-on violation (§2) (M8 → G1) (scenarios.md:260)",
        coverage: Coverage::Partial,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M8 arms judged with G1 armed",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m8_forced_concurrent_originals_allow_only_one_confirmation_in_flight",
                must_assert: "both callers complete after serialized \
                              confirmation and no follow-on organic violation",
            },
        ],
        note: "Focused timing evidence covers the forced overlap; the \
               full-contract run remains the closure delta",
    },
    Clause {
        id: "m8-b12-timing-script",
        owner: "M8",
        text: "B12 explicit timing script for forced reordering \
               (scenarios.md:260)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/transition_timing.rs",
            test_fn: "m8_forced_concurrent_originals_allow_only_one_confirmation_in_flight",
            must_assert: "explicit 2 s original delays and a 5 s confirmation \
                          delay force the required reordering",
        }],
        note: "Pinned at phi=0 and phi=1",
    },
    // ── M9 — phantom race at saturation (scenarios.md:283) ───────
    Clause {
        id: "m9-recovery-survives-race",
        owner: "M9",
        text: "Recovery machinery survives the organic-429 race, per M8's \
               asserts; recovery itself M8/core-owned (scenarios.md:283)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M9 arm: phantom injected, recovery completes",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m9_forced_phantom_race_at_saturation_recovers_per_m8",
                must_assert: "at 14/15 the phantom lands strictly between the \
                              raced send's transport hand-off and its mock \
                              receipt; the organic 16th-hit 429 recovers per \
                              M8 (single confirmation in flight, N19 wait \
                              bound, no follow-on violation) at φ ∈ {0, 1}",
            },
        ],
        note: "Forced reservation-to-receipt race landed 2026-08-14 \
               (Ballot G); mutation-checked — a phantom missing the raced \
               policy fails the organic-429 assertion",
    },
    Clause {
        id: "m9-headroom-record",
        owner: "U5",
        text: "Records what nonzero headroom would have bought per contention \
               level — demoted to the U5 exclusion 2026-08-13 (ballot pass): \
               characterization for the settled headroom-zero decision, not \
               conformance (scenarios.md:283, :289, U5)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion adopted at the 2026-08-13 ballot pass (scenarios.md \
               U5): no gate consumes the record; instrument unbuilt and \
               declared so. Pre-demotion state: no instrument existed \
               anywhere (map §8.1 item 3)",
    },
    Clause {
        id: "m9-race-exposure-attribution",
        owner: "B13",
        text: "Race exposure attributed via B13 correlation identity (§2); \
               judge/B13-owned (scenarios.md:283)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m9_forced_phantom_race_at_saturation_recovers_per_m8",
                must_assert: "unavoidable_exposure is Some over a real \
                              integration run: the judge validates the \
                              correlation-bound allowance against the phantom \
                              state change, and the identical evidence \
                              without the allowance fails G1",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "g1_unavoidable_exposure_is_pre_observation_only_and_capped",
                must_assert: "exposure is pre-observation only and capped",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "correlation_and_reproduction_seams_are_structural",
                must_assert: "correlation identity is structural",
            },
        ],
        note: "Integration run landed 2026-08-14 (Ballot G); the allowance \
               is proven load-bearing by the no-allowance G1 failure",
    },
    // ── M10 — agent-loop stress (scenarios.md:294, amended
    //    2026-08-12) ──────────────────────────────────────────────
    Clause {
        id: "m10-spacing-floor",
        owner: "M10",
        text: "Spacing floor never violated (scenarios.md:294)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M10 `paced`: absolute arithmetic over 273 \
                          dispatches vs the literal 250 ms, HEADs included",
        }],
        note: "",
    },
    Clause {
        id: "m10-in-flight-cap",
        owner: "M10",
        text: "In-flight ≤ 2 (scenarios.md:294)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M10 `capped` flag",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "d5_in_flight_cap_restatements_agree",
                must_assert: "the actor and independent judge restatements of \
                              D5's cap remain equal",
            },
        ],
        note: "",
    },
    Clause {
        id: "m10-g1",
        owner: "M10",
        text: "G1 holds (M10 → G1) (scenarios.md:294)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M10 judged with G1 armed at both φ",
        }],
        note: "At the row's stated scale",
    },
    Clause {
        id: "m10-fuse-quiet",
        owner: "C3",
        text: "Fuse does not trip — false-positive absence under saturation; \
               C3 owns the property incl. headroom, M10 owns the integration \
               instance, X1 owns true-positive (scenarios.md:294, :298)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "src/actor.rs",
                test_fn: "c3_floor_compliant_traces_never_trip",
                must_assert: "4,096-case property: no floor-compliant trace \
                              trips",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "c3_floor_compliant_cadence_holds_the_steady_state_maximum",
                must_assert: "exact steady-state peaks 4/s and 240/min",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M10 `fuse_quiet` + `paced` (integration \
                              instance)",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "x1_fault_injection_trips_at_the_actor_transport_boundary",
                must_assert: "true-positive: the fuse does trip on violation",
            },
        ],
        note: "Composition per the round-four ruling. Doc finding 11's text \
               was corrected to cite the composition at 77aee08 (doc-split \
               commit 1), closing map §8.2 item 1",
    },
    Clause {
        id: "m10-queue-drains",
        owner: "M10",
        text: "Queue drains to completion (named by G5) (scenarios.md:294)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M10 arm: served == expected_served (270/270)",
        }],
        note: "",
    },
    Clause {
        id: "m10-cancel-prompt-resolution",
        owner: "M10",
        text: "Cancelled callers get prompt resolution — 25 ms bound, queued \
               and dispatched (Tom's 2026-08-12 amendment) \
               (scenarios.md:294)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "single-poll after exactly one 25 ms tick, 30 queued \
                          + 1 mock-proven dispatched; dispatched response \
                          later reconciles to in-flight 0",
        }],
        note: "",
    },
    Clause {
        id: "m10-scale",
        owner: "M10",
        text: "Scale: hundreds of enqueues over many simulated minutes \
               (scenarios.md:294)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "300 enqueues; `sustained` asserts span ≥ 30 min \
                          (actual ≈ 66 min)",
        }],
        note: "Recorded span figure was stale in two places (map §8.3 \
               item 7); corrected to the measured 3,963,500 ms in both at \
               77aee08 (doc-split commit 1)",
    },
    // The dropped-dispatched-RequestTicket lifecycle has no owning
    // row in scenarios.md (map §8.1 item 2). Design §7 item 2 left a
    // SHELL-owned entry as Tom's call; it was omitted at slice time
    // and adopted by Tom 2026-08-13 (DS decisions pass) so that no
    // obligation lives solely in prose confessions.
    Clause {
        id: "shell-dropped-dispatched-ticket",
        owner: "SHELL",
        text: "A dispatched RequestTicket dropped without cancel resolves \
               cleanly: the reservation still reconciles, scheduling \
               continues, nothing wedges (external-review shell \
               obligation; stated by no scenario — adopted by Tom \
               2026-08-13, design §7 item 2)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/actor_safety.rs",
            test_fn: "dropped_dispatched_ticket_still_reconciles_and_does_not_wedge_following_work",
            must_assert: "drop occurs only after mock handoff; detached \
                          completion reconciles and the following request \
                          eventually dispatches without a reprobe or wedge",
        }],
        note: "The public actor test pins the lifecycle at the transport \
               handoff boundary",
    },
    // ── M11 — layer-1 ceiling and Cloudflare terminal
    //    (scenarios.md:333) ────────────────────────────────────────
    Clause {
        id: "m11-b10-ceilings-mock",
        owner: "B10",
        text: "Mock enforces both B10 ceilings tripping into the Cloudflare \
               shape; B10 mock side, consumed by M11a (scenarios.md:333)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/mock_fidelity.rs",
            test_fn: "b10_b11_layer1_and_injected_stimuli_are_distinct",
            must_assert: "burst 20/21 at the wire; sustained 1000/1001 at the \
                          model",
        }],
        note: "Rolling-edge convention unpinned — map §8.3 item 6",
    },
    Clause {
        id: "m11-compliant-never-trips",
        owner: "G2",
        text: "Compliant client never trips either ceiling; G2 owns the \
               property, M11a's sweep is its named binding evidence \
               (2026-08-13 amendment) (scenarios.md:333, :369)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/m11_ceiling_sweep.rs",
                test_fn: "m11a_near_ceiling_compliant_sweep_never_trips_either_layer1_ceiling",
                must_assert: "301 floor-paced dispatches under a synthetic \
                              high-limit policy peak at exactly 4 per rolling \
                              second and 240 per rolling minute — the \
                              compliant maxima against the 20/1,000 B10 \
                              ceilings — with zero layer-1 trips, judged \
                              with G2 armed under both bucket profiles",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "G2 judged green in every fragment run, including \
                              M10's 66-minute run at both φ",
            },
        ],
        note: "Re-owned M11→G2 at the 2026-08-13 ballot pass \
               (scenarios.md:369). M11a's dedicated near-ceiling sweep — the \
               amendment's named binding evidence — landed 2026-08-14; \
               mutation-checked (a policy-paced run loses the near-ceiling \
               peaks and fails G5)",
    },
    Clause {
        id: "m11-cloudflare-terminal",
        owner: "M11",
        text: "Client recognizes the Cloudflare shape generally; halt-shaped \
               terminal, zero retries (M11b); precedence core-owned \
               (scenarios.md:333)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "cloudflare_shaped_response_halts_the_gate_and_publishes_status",
                must_assert: "2 observations, no retry",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "cloudflare_shape_halts_before_status_or_header_handling",
                must_assert: "halt precedes status/header handling",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "cloudflare_on_the_final_attempt_halts_instead_of_escalating",
                must_assert: "halt wins over escalation on the final attempt",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M11 arm: terminal recognized on the wire",
            },
        ],
        note: "",
    },
    Clause {
        id: "m11-halt-published",
        owner: "M11",
        text: "Halt published (scenarios.md:333)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "cloudflare_shaped_response_halts_the_gate_and_publishes_status",
                must_assert: "halt visible on the watch channel",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M11 arm: status.halted",
            },
        ],
        note: "",
    },
    Clause {
        id: "m11-pending-errored",
        owner: "M11",
        text: "Pending errored (scenarios.md:333)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M11 arm: the one pending caller errored",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "cloudflare_shaped_response_halts_the_gate_and_publishes_status",
                must_assert: "the halt path errors the pending caller and \
                              publishes",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "remap_then_malformed_response_drains_queued_callers_for_the_current_policy",
                must_assert: "the shared refusal path drains a queue",
            },
        ],
        note: "Full per the 2026-08-13 ballot amendment (scenarios.md M11): \
               halt-time draining discharged by composition — both cited \
               paths traverse the same drain loop",
    },
    // ── M12 — 4xx-tripwire obligations (scenarios.md:347) ────────
    Clause {
        id: "m12-injected-401-zero-retries",
        owner: "M12",
        text: "Injected 401: zero retries (scenarios.md:347)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M12 arm: completed, exactly 1 GET",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b10_b11_layer1_and_injected_stimuli_are_distinct",
                must_assert: "the 401 stimulus channel is distinct and \
                              scriptable",
            },
        ],
        note: "At fragment scale",
    },
    Clause {
        id: "m12-generic-4xx-no-retry-loop",
        owner: "M12",
        text: "Generic 4xx: no retry loop; disposition core-owned \
               (scenarios.md:347)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "generic_4xx_with_valid_headers_completes_and_reconciles",
                must_assert: "generic 4xx completes and reconciles; only 429 \
                              yields Requeue",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M12 arm: a wire-level 4xx completes with \
                              exactly 1 GET",
            },
        ],
        note: "Full per the 2026-08-13 ballot amendment (scenarios.md M12): \
               core disposition totality + the M12 driver arm's wire-level \
               4xx compose",
    },
    Clause {
        id: "m12-429-single-retry-ladder",
        owner: "M8",
        text: "429: at-most-one-retry-then-escalate — M8's ladder, \
               cross-owned (scenarios.md:347)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m8_confirmation_429_escalates_without_a_third_get",
                must_assert: "no third GET",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "every_non_success_final_outcome_escalates",
                must_assert: "final non-success escalates",
            },
        ],
        note: "Cross-owned; discharged under M8",
    },
    Clause {
        id: "m12-tripwire-feed",
        owner: "M12",
        text: "All 4xx responses feed the tripwire counter — M12 owns the \
               feed; trip logic is C4's (scenarios.md:347)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "src/actor.rs",
                test_fn: "c4_probe_feed_trips_shared_halt_and_drains_and_publishes",
                must_assert: "probe 4xx completion feeds the actual tripwire \
                              path and trips the shared halt",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "c4_ordinary_feed_trips_shared_halt_and_drains_and_publishes",
                must_assert: "ordinary 4xx completion feeds the actual \
                              tripwire path and trips the shared halt",
            },
        ],
        note: "Internal fault seeding is required because intact D5 pacing \
               makes the public actor unable to accumulate either threshold; \
               both real response-feed call sites remain load-bearing",
    },
    Clause {
        id: "m12-trip-logic-thresholds",
        owner: "C4",
        text: "Trip logic thresholds/edges — C4's explicitly (\"trip logic \
               itself is C4\", scenarios.md:352)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "src/actor.rs",
            test_fn: "c4_pins_burst_sustained_and_exact_window_edges",
            must_assert: "burst, sustained, and exact window edges pinned",
        }],
        note: "The M12 evidence row's \"full tripwire threshold matrix \
               remains pending\" claims C4's already-green property — map \
               §8.1 item 1",
    },
    // M12's "server-side restriction behavior" row is the U2
    // exclusion itself — collapsed into `u2-server-side-restriction`
    // (recorded in the slice hand-off).

    // ── M13 — gate structure on the wire (scenarios.md:355) ──────
    Clause {
        id: "m13-never-more-than-two",
        owner: "M13",
        text: "Never > 2 ordinary requests in flight (scenarios.md:355)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "m13_ordinary_waiters_are_fifo_and_never_exceed_two_in_flight",
                must_assert: "in-flight cap of 2 held",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M13 + M10 arms: in_flight_at_arrival <= 2",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "d5_in_flight_cap_restatements_agree",
                must_assert: "the actor and independent judge restatements of \
                              D5's cap remain equal",
            },
        ],
        note: "",
    },
    Clause {
        id: "m13-head-never-overlaps",
        owner: "M13",
        text: "HEAD never overlaps any request (N18) (scenarios.md:355)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M13 arm: !head_overlap with a forced 2 s delay",
            },
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "pending_head_writer_blocks_a_front_get_until_it_runs_exclusively",
                must_assert: "the HEAD runs exclusively",
            },
        ],
        note: "",
    },
    Clause {
        id: "m13-writer-preference",
        owner: "M13",
        text: "Writer preference: a queued HEAD blocks new ordinary permits \
               (scenarios.md:355)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/actor_shell.rs",
            test_fn: "pending_head_writer_blocks_a_front_get_until_it_runs_exclusively",
            must_assert: "forced delayed reader; the HEAD hands off before \
                          the earlier queued GET",
        }],
        note: "Focused actor test; scenario-assertion integration pending per \
               the M13 evidence row, which correctly cites the composition",
    },
    Clause {
        id: "m13-fifo",
        owner: "M13",
        text: "Ordinary permits granted in arrival order (FIFO) \
               (scenarios.md:355)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/actor_shell.rs",
            test_fn: "m13_ordinary_waiters_are_fifo_and_never_exceed_two_in_flight",
            must_assert: "correlations 2, 3, 4, 5 served in order",
        }],
        note: "",
    },
    Clause {
        id: "m13-heads-ordinary-citizens",
        owner: "M13",
        text: "HEADs are ordinary citizens: spacing floor + fuse-counted; \
               fuse counting owned by C3/X1's boundary (scenarios.md:355)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/actor_shell.rs",
                test_fn: "probe_then_get_share_the_actor_gate_and_keep_distinct_wire_ids",
                must_assert: "GET ≥ HEAD + 250 ms",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "x1_fault_injection_trips_at_the_actor_transport_boundary",
                must_assert: "counting happens at start_dispatch, the common \
                              HEAD/GET hook",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M10 `paced` includes HEADs",
            },
        ],
        note: "",
    },
    Clause {
        id: "m13-b12-overlap-script",
        owner: "M13",
        text: "B12 explicit overlap timing script (scenarios.md:355)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: DRIVER,
            test_fn: DRIVER_FN,
            must_assert: "M13 arm: scripted 2 s delay",
        }],
        note: "The only one of B12's four required timing scripts that \
               exists (see b12-scripted-delay)",
    },
    // ── C1–C5 core properties (scenarios.md:367) ─────────────────
    Clause {
        id: "c1-reserved-never-saturated",
        owner: "C1",
        text: "Granted reservation never inside a saturated window on any \
               server φ, full-bucket padding (scenarios.md:369)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/c1_scheduling.rs",
            test_fn: "every_reserved_outcome_is_safe_for_every_server_phase",
            must_assert: "4,096 cases, asserts on both branches, independent \
                          phase-bucketizing oracle",
        }],
        note: "De-vacuized 2026-08-09 (was ~97% vacuous); NotBefore re-ask \
               pinned exact",
    },
    Clause {
        id: "c1-rollover-boundaries",
        owner: "C1",
        text: "Explicit rollover boundary cases (scenarios.md:369)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c1_scheduling.rs",
                test_fn: "rollover_cases_are_safe_just_before_exactly_on_and_just_after",
                must_assert: "just-before / exactly-on / just-after all safe",
            },
            Citation {
                file: "tests/c1_scheduling.rs",
                test_fn: "not_before_takes_maximum_across_all_saturated_windows_and_rules",
                must_assert: "NotBefore is the maximum across windows and \
                              rules",
            },
            Citation {
                file: "tests/c1_scheduling.rs",
                test_fn: "zero_headroom_uses_the_max_hits_th_newest_entry",
                must_assert: "zero headroom keys off the max_hits-th newest \
                              entry",
            },
            Citation {
                file: "tests/c1_scheduling.rs",
                test_fn: "zero_max_hits_blocks_forever",
                must_assert: "zero max_hits blocks",
            },
        ],
        note: "",
    },
    Clause {
        id: "c2-two-triplet-rulepair",
        owner: "C2",
        text: "Two-triplet increasing periods → RulePair \
               (scenarios.md:375)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "parses_two_triplets_as_burst_and_sustained",
                must_assert: "two triplets parse as burst + sustained",
            },
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "valid_pairs_round_trip",
                must_assert: "round-trip property over valid pairs",
            },
        ],
        note: "",
    },
    Clause {
        id: "c2-bad-triplet-typed",
        owner: "C2",
        text: "One-/three-/malformed-triplet → typed errors, never panic/OOB \
               (scenarios.md:375)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "rejects_non_pair_shapes",
                must_assert: "non-pair shapes rejected with typed errors",
            },
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "malformed_triplet_is_typed",
                must_assert: "malformed triplet is a typed error",
            },
            Citation {
                file: "tests/c2_headers.rs",
                test_fn: "malformed_text_never_parses",
                must_assert: "property: malformed text never parses, never \
                              panics",
            },
        ],
        note: "",
    },
    Clause {
        id: "c2-missing-headers-typed",
        owner: "C2",
        text: "Missing headers are typed errors, not empty lists \
               (scenarios.md:375)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/c2_headers.rs",
            test_fn: "missing_headers_are_typed",
            must_assert: "typed error, not an empty policy list",
        }],
        note: "",
    },
    Clause {
        id: "c2-combined-precedence",
        owner: "C2",
        text: "Combined precedence — malformed-429 is refusal-shaped, generic \
               4xx + valid headers parses (pinning the core-design rule) \
               (scenarios.md:375)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "malformed_429_takes_precedence_and_never_opens_a_retry_episode",
                must_assert: "malformed-429 precedence",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "generic_4xx_with_valid_headers_completes_and_reconciles",
                must_assert: "generic 4xx with valid headers parses and \
                              reconciles",
            },
        ],
        note: "",
    },
    Clause {
        id: "c3-floor-compliant-never-trips",
        owner: "C3",
        text: "Fuse never trips on any floor-compliant trace, incl. the 2× \
               headroom claim (scenarios.md:385)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "src/actor.rs",
                test_fn: "c3_floor_compliant_traces_never_trip",
                must_assert: "property with irregular gaps, ≥ 3 trailing \
                              minutes, every step asserts",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "c3_floor_compliant_cadence_holds_the_steady_state_maximum",
                must_assert: "cadence pin, exact peaks 4 and 240",
            },
        ],
        note: "Lives in src/, not tests/ — the round-four lesson (a tests/ \
               grep misses it)",
    },
    Clause {
        id: "c3-burst-boundary",
        owner: "C3",
        text: "Burst boundary — 10 pass, 11th trips, exact 1 s edge excluded \
               (half-open) (scenarios.md:385)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "src/actor.rs",
                test_fn: "fuse_uses_the_documented_half_open_boundaries",
                must_assert: "half-open boundary convention pinned",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "c3_and_x1_fault_injection_pin_burst_and_sustained_boundaries",
                must_assert: "10 pass, 11th trips",
            },
        ],
        note: "",
    },
    Clause {
        id: "c3-sustained-threshold",
        owner: "C3",
        text: "Sustained never fires below 500/min — pinned exactly 500 by \
               the 2026-08-13 decisions pass (scenarios.md:385, :392)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "src/actor.rs",
            test_fn: "c3_and_x1_fault_injection_pin_burst_and_sustained_boundaries",
            must_assert: "499 safe, 500 trips",
        }],
        note: "",
    },
    Clause {
        id: "c3-trip-latched",
        owner: "C3",
        text: "Trip is latched (scenarios.md:405)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "src/actor.rs",
            test_fn: "c3_x1_trip_is_latched_and_drains_and_publishes",
            must_assert: "after a fuse trip, a later dispatch attempt remains \
                          halted and the pending queue stays drained",
        }],
        note: "Uses the established internal SafetyCounters fault-injection \
               seam so the test does not weaken D5 to manufacture a trip",
    },
    Clause {
        id: "c4-thresholds-edges",
        owner: "C4",
        text: "Same shape as C3 over a 4xx counter — thresholds and edges \
               (scenarios.md:409)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "src/actor.rs",
            test_fn: "c4_pins_burst_sustained_and_exact_window_edges",
            must_assert: "burst, sustained, and exact window edges pinned",
        }],
        note: "Thresholds are C3's, adopted as a recorded doc finding (actor \
               slice)",
    },
    Clause {
        id: "c4-halt-semantics-shared",
        owner: "C4",
        text: "Shares the fuse's halt semantics (scenarios.md:409)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "src/actor.rs",
                test_fn: "c4_probe_feed_trips_shared_halt_and_drains_and_publishes",
                must_assert: "probe-fed trip drains queued callers and \
                              publishes Halted",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "c4_ordinary_feed_trips_shared_halt_and_drains_and_publishes",
                must_assert: "ordinary-fed trip drains queued callers and \
                              publishes Halted",
            },
        ],
        note: "Both response entry paths compose the C4 counter with the same \
               latched halt/drain/publication semantics",
    },
    Clause {
        id: "c5-rollback-exact",
        owner: "C5",
        text: "Rollback of an undispatched reservation restores state exactly \
               (scenarios.md:412)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "rollback_restores_history_exactly",
                must_assert: "property: exact restoration",
            },
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "rollback_removes_exact_local_entry",
                must_assert: "remove-by-id, not by position",
            },
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "reservation_is_not_duplicated_across_policy_rules",
                must_assert: "one send, one entry across rules",
            },
        ],
        note: "",
    },
    Clause {
        id: "c5-unknown-outcome-ages-out",
        owner: "C5",
        text: "UnknownOutcome stays counted, ages out only by window passage \
               (scenarios.md:412)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "unknown_outcome_stays_counted_until_every_window_passes",
                must_assert: "counted until every window passes",
            },
            Citation {
                file: "tests/response_disposition.rs",
                test_fn: "unknown_first_outcome_stays_counted_and_permits_a_successful_final_attempt",
                must_assert: "unknown outcome stays counted through the \
                              episode",
            },
        ],
        note: "",
    },
    Clause {
        id: "c5-no-double-count-or-loss",
        owner: "C5",
        text: "No interleaving double-counts or loses a send \
               (scenarios.md:412)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/c5_lifecycle.rs",
            test_fn: "interleavings_neither_double_count_nor_lose_sends",
            must_assert: "observed responses, non-FIFO token order, ≥ 1 op \
                          per case",
        }],
        note: "",
    },
    Clause {
        id: "c5-abandonment",
        owner: "C5",
        text: "Abandonment — drop bomb + conservatively safe state, both \
               confirmation halves (scenarios.md:412)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "accidental_drop_bomb_preserves_conservative_history",
                must_assert: "dropped token leaves conservative history",
            },
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "abandoned_first_confirmation_ages_out_and_permits_the_final_attempt",
                must_assert: "first-confirmation abandonment ages out",
            },
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "abandoned_final_confirmation_escalates_instead_of_wedging",
                must_assert: "final-confirmation abandonment escalates, never \
                              wedges",
            },
            Citation {
                file: "tests/c5_lifecycle.rs",
                test_fn: "drop_bomb_is_silent_during_unwind",
                must_assert: "no double panic during unwind",
            },
        ],
        note: "Drop-bomb tests are debug-only (129 debug vs 127 release), \
               recorded",
    },
    // ── X1/X2 fault-injection and structural (scenarios.md:424) ──
    Clause {
        id: "x1-burst-trip-boundary",
        owner: "X1",
        text: "Burst fault trips at the 11th attempt at the transport \
               boundary (scenarios.md:426)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "src/actor.rs",
            test_fn: "x1_fault_injection_trips_at_the_actor_transport_boundary",
            must_assert: "real start_dispatch hook, pacing enabled, trips at \
                          the 11th",
        }],
        note: "",
    },
    Clause {
        id: "x1-sustained-trip",
        owner: "X1",
        text: "Sustained fault trips at ~500/min while burst stays silent \
               (scenarios.md:426)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "src/actor.rs",
                test_fn: "x1_fault_injection_trips_at_the_actor_transport_boundary",
                must_assert: "sustained trip with burst silent",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "c3_and_x1_fault_injection_pin_burst_and_sustained_boundaries",
                must_assert: "boundary pins 499/500",
            },
        ],
        note: "",
    },
    Clause {
        id: "x1-trip-drain-publish",
        owner: "X1",
        text: "On trip — pending deque errored back, Halted published \
               (scenarios.md:436)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "src/actor.rs",
                test_fn: "c3_x1_trip_is_latched_and_drains_and_publishes",
                must_assert: "fuse trip errors the pending deque, publishes \
                              Halted, and remains latched on a later dispatch",
            },
            Citation {
                file: "src/actor.rs",
                test_fn: "fuse_trip_inside_start_ordinary_rolls_back_and_resolves_the_popped_caller",
                must_assert: "a trip inside schedule resolves the popped \
                              caller as Halted, publishes Halted, and rolls \
                              back the undispatched reservation",
            },
        ],
        note: "Composes the X1/C3 injected trip with the actor's actual \
               drain and watch-publication path",
    },
    Clause {
        id: "x2-single-send-path",
        owner: "X2",
        text: "One private HTTP client, no second construction/send path, \
               pinned by a structural test (scenarios.md:440, :443) — \
               load-bearing per result-draft.md §1",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "src/actor.rs",
            test_fn: "x2_transport_owner_is_private_and_has_one_send_call_site",
            must_assert: "Actor remains private and source structure contains \
                          exactly one transport send call site",
        }],
        note: "The public spawn API also carries a compile-fail doctest that \
               outside code cannot import the private Actor; production \
               integration still owes its own re-pin when a real client lands",
    },
    Clause {
        id: "x2-parser-cap-limitation",
        owner: "X2",
        text: "Accepted limitation: a future HTTP parser's upstream \
               allocation cap cannot be forced by the spike \
               (scenarios.md:440)",
        coverage: Coverage::Untested,
        citations: &[],
        note: "Honored as recorded. accepted — result-draft.md:668 and \
               actor-handoff.md §1",
    },
    // ── U1–U4 declared-untested exclusions (scenarios.md:446) ────
    Clause {
        id: "u1-proactive-remap",
        owner: "U1",
        text: "Proactive remap triggers out of scope; M5 reactive is the \
               tested surface (scenarios.md:448)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: no proactive-remap \
               code path or test claim found; M5 tests are all \
               response-reactive. Absorbs the M5 table's \"remap triggers \
               beyond reactive are U1\" row (same clause, two spellings)",
    },
    Clause {
        id: "u2-server-side-restriction",
        owner: "U2",
        text: "Server-side 4xx restriction behavior untested; obligations \
               only (M12) (scenarios.md:452)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: the mock models no \
               4xx-budget threshold; no test claims server behavior. Absorbs \
               the M12 table's \"server-side restriction behavior\" row (same \
               clause, two spellings)",
    },
    Clause {
        id: "u3-legacy-resolution",
        owner: "U3",
        text: "Legacy bucket resolution conditional on Assumed(60s/60s); the \
               live instrument is not a spike gate (scenarios.md:455)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: the shipped default \
               is structurally required by `SweepPlan::new` \
               (sweep_plan_structurally_requires_the_shipped_assumed_60s_default), \
               and since SD-R5-F3 the M-series driver builds its row plan \
               through that constructor, so losing the last Assumed driver \
               row fails structurally; the §6 supplemental ledger is empty; \
               no run claimed",
    },
    Clause {
        id: "u4-layer1-uncharacterized",
        owner: "U4",
        text: "Real layer-1 rules deliberately uncharacterized; B10 is the \
               inferred lane (scenarios.md:472)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: B10 numbers are \
               documented as inferred; no external Cloudflare evidence \
               claimed",
    },
    // ── G1–G6 gates (scenarios.md:477) ───────────────────────────
    Clause {
        id: "g1-zero-client-violations",
        owner: "G1",
        text: "Zero client-caused violations, follow-on included, exposure \
               excluded per §2 (scenarios.md:482)",
        coverage: Coverage::Partial,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "G1 judged in every run at both φ",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "g1_unavoidable_exposure_is_pre_observation_only_and_capped",
                must_assert: "exposure validation: pre-observation only, \
                              capped",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "correlation_and_reproduction_seams_are_structural",
                must_assert: "attribution seams structural",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "all_global_gates_are_armed_and_judged_from_wire_evidence",
                must_assert: "teeth: the gate is armed and judged from wire \
                              evidence",
            },
        ],
        note: "Judge at src/conformance.rs (G1 section). Green across all 13 \
               fragments × 2 φ × (M8 × 2 lanes); no full-contract run; the \
               exposure path is never used in integration",
    },
    Clause {
        id: "g2-ceilings-never-tripped",
        owner: "G2",
        text: "Neither B10 ceiling tripped, armed everywhere \
               (scenarios.md:489)",
        coverage: Coverage::Partial,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "G2 judged over every observation of every run, \
                              incl. M10's 66-minute run",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "all_global_gates_are_armed_and_judged_from_wire_evidence",
                must_assert: "teeth: G2 armed",
            },
        ],
        note: "Same fragment qualifier as G1",
    },
    Clause {
        id: "g3-over-delay-bounded",
        owner: "G3",
        text: "Over-delay ≤ ε against padded-safe time, exclusions authorized \
               (scenarios.md:493)",
        coverage: Coverage::Partial,
        citations: &[
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "permit-debt + queueing oracle; \
                              dispatched-before-eligible also enforced",
            },
            Citation {
                file: DRIVER,
                test_fn: "g3_oracle_pins_its_independent_padded_safe_arithmetic",
                must_assert: "independent N13 hit + period + bucket pins for \
                              the oracle",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "g3_exclusions_require_script_owned_bounded_intervals",
                must_assert: "exclusions must be script-owned bounded \
                              intervals",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "g3_fails_closed_when_the_oracle_has_no_eligibility_entry",
                must_assert: "a missing eligibility entry produces the exact \
                              G3 failure and cannot pass or become \
                              verdict-eligible",
            },
        ],
        note: "ε = 500 ms final (2026-08-13, scenarios.md:562): an \
               unmodelled-padding allowance, not slop; the oracle stays \
               padding-independent. Delta: fragment scale — finished by \
               the full-contract run",
    },
    Clause {
        id: "g4-m2-duration-bound",
        owner: "G4",
        text: "M2 duration ≤ 1.05× the padded minimum (scenarios.md:520)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "g4_uses_independent_integer_arithmetic_at_the_exact_boundary",
                must_assert: "boundary pinned exactly at 1050/1051",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M2 saturates burst and sustained windows and is \
                              judged against the runtime-derived padded minimum",
            },
            Citation {
                file: DRIVER,
                test_fn: "m2_g4_minimum_is_runtime_derived_and_reaches_both_stalls",
                must_assert: "independent arithmetic derives the minimum from \
                              the policy definition, queue depth, D5 floor, \
                              bucket padding, and service delay",
            },
        ],
        note: "The judged M2 run now crosses both the 5 s burst and 60 s \
               sustained boundaries; the oracle uses independent arithmetic \
               and no production scheduling helper",
    },
    Clause {
        id: "g5-scenario-assertions",
        owner: "G5",
        text: "Every scenario's own assertions, fragments included; \
               unauthorized refusal fails G5 (scenarios.md:524)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "a_fragment_run_is_judged_but_is_never_verdict_eligible",
                must_assert: "fragment teeth: a forced-false fragment fails \
                              G5",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "driver arms yield computed verdicts, never \
                              constants",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "g5_rejects_unauthorized_refusal_when_wire_safety_is_green",
                must_assert: "G1 and G2 remain green while an independent \
                              unauthorized-refusal assertion alone fails G5",
            },
        ],
        note: "Judge at src/conformance.rs (G5 section); the dedicated teeth \
               test proves the refusal condition cannot hide behind a wire \
               safety failure",
    },
    Clause {
        id: "g6-reproduction-records",
        owner: "G6",
        text: "Reproduction record for every failure; (seed, φ) mandatory \
               where swept (scenarios.md:537)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "evidence_cannot_pass_vacuously",
                must_assert: "missing reproduction records hard-error",
            },
            Citation {
                file: "tests/conformance_harness.rs",
                test_fn: "correlation_and_reproduction_seams_are_structural",
                must_assert: "ReproductionMismatch is structural",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "the driver records (seed, φ) per swept row",
            },
        ],
        note: "Full per the 2026-08-13 ballot amendment (scenarios.md §6 \
               G6): the enforcement locus is the judge's hard-error path \
               (MissingReproductionRecord / ReproductionMismatch), which the \
               citations pin; the G6 GateResult reports record-keeping that \
               path has already enforced (it is never passed: false — map \
               §8.3 item 3, accepted by the amendment)",
    },
    // ── B1–B14 mock fidelity budget (scenarios.md:612, §7.2) ─────
    Clause {
        id: "b1-header-protocol",
        owner: "B1",
        text: "Full verbatim header protocol; Retry-After on 429 \
               (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
                must_assert: "headers parsed by the production parser",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b7_m4_synthetic_one_and_three_window_policies_cross_as_raw_headers",
                must_assert: "synthetic shapes cross verbatim",
            },
            Citation {
                file: "tests/actor_safety.rs",
                test_fn: "organic_429_emits_retry_after_and_actor_honors_the_wire_value",
                must_assert: "an organic mock 429 emits Retry-After as wire \
                              text and the actor delays the retry by the parsed \
                              value plus policy padding",
            },
        ],
        note: "The organic path is load-bearing: a capturing wrapper observes \
               the emitted wire text before the public actor consumes it",
    },
    Clause {
        id: "b2-black-box-counters",
        owner: "B2",
        text: "Black-box counters, organic 429, restriction enforcement \
               (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b2_b3_quantized_expiry_and_restriction_are_independent",
                must_assert: "quantized expiry and restriction are \
                              independent",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b6_b9_residue_and_phantoms_are_mock_owned_counter_facts",
                must_assert: "counters are mock-owned facts",
            },
        ],
        note: "",
    },
    Clause {
        id: "b3-server-owned-phase",
        owner: "B3",
        text: "Server-owned φ, most-adversarial quantization, §1 resolutions \
               (scenarios.md:633)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b3_generated_phase_sweep_has_non_vacuous_independent_oracle",
                must_assert: "independent-oracle property, 4,096 cases",
            },
            Citation {
                file: "src/mock/model.rs",
                test_fn: "bucket_boundaries_use_the_adversarial_new_bucket_reading",
                must_assert: "the adversarial new-bucket reading is pinned",
            },
        ],
        note: "CN5 records the model choice",
    },
    Clause {
        id: "b4-post-increment-state",
        owner: "B4",
        text: "Post-increment 1:1 state (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/mock_fidelity.rs",
            test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
            must_assert: "state emitted post-increment, 1:1",
        }],
        note: "",
    },
    Clause {
        id: "b5-head-semantics",
        owner: "B5",
        text: "HEAD semantics 204/200 + full headers, non-counting; \
               scriptable degradation (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
                must_assert: "HEAD returns full headers without counting",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b6_b9_residue_and_phantoms_are_mock_owned_counter_facts",
                must_assert: "HEAD sees residue without incrementing",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b11_m3_m4_script_channel_covers_every_response_shape",
                must_assert: "degradation is scriptable",
            },
        ],
        note: "",
    },
    Clause {
        id: "b6-preloadable-counters",
        owner: "B6",
        text: "Pre-loadable counters (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/mock_fidelity.rs",
            test_fn: "b6_b9_residue_and_phantoms_are_mock_owned_counter_facts",
            must_assert: "preloaded residue is visible",
        }],
        note: "",
    },
    Clause {
        id: "b7-topology",
        owner: "B7",
        text: "N23 five-policy topology, routing, legacy Account+Ip pair, \
               synthetic policies (scenarios.md:612, :691)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
                must_assert: "all five endpoints routed",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b7_m4_synthetic_one_and_three_window_policies_cross_as_raw_headers",
                must_assert: "synthetic policies cross the wire",
            },
        ],
        note: "Fixture incl. the Account+Ip pair (src/mock/mod.rs)",
    },
    Clause {
        id: "b8-scripted-rename-shrink",
        owner: "B8",
        text: "Scripted rename/shrink (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/mock_fidelity.rs",
            test_fn: "b8_policy_rename_and_shrink_keep_existing_hits",
            must_assert: "rename and shrink are scriptable, hits kept",
        }],
        note: "",
    },
    Clause {
        id: "b9-scripted-phantoms",
        owner: "B9",
        text: "Scripted phantom increments, bursty shape (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/mock_fidelity.rs",
            test_fn: "b6_b9_residue_and_phantoms_are_mock_owned_counter_facts",
            must_assert: "phantom injections land as counter facts",
        }],
        note: "Burstiness is a script parameter; injections tested",
    },
    Clause {
        id: "b10-two-ceilings",
        owner: "B10",
        text: "Two ceilings, armed everywhere, Cloudflare-shaped trip \
               (scenarios.md:659)",
        coverage: Coverage::Full,
        citations: &[Citation {
            file: "tests/mock_fidelity.rs",
            test_fn: "b10_b11_layer1_and_injected_stimuli_are_distinct",
            must_assert: "burst n/n+1 at the wire; sustained n/n+1 at the \
                          model; trip is Cloudflare-shaped",
        }],
        note: "G2 armed in every judged run. Rolling-window edge convention \
               unpinned (map §8.3 item 6)",
    },
    Clause {
        id: "b11-stimulus-channel",
        owner: "B11",
        text: "Stimulus injection channel: 429/401/4xx/Cloudflare/transport \
               error (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b10_b11_layer1_and_injected_stimuli_are_distinct",
                must_assert: "injected stimuli are distinct from organic \
                              behavior",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b11_m3_m4_script_channel_covers_every_response_shape",
                must_assert: "the script channel covers every response shape",
            },
        ],
        note: "",
    },
    Clause {
        id: "b12-scripted-delay",
        owner: "B12",
        text: "Deterministic scripted delay; canonical 81 ms default; \
               M5/M8/M9/M13 explicit timing scripts (scenarios.md:726)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b12_b13_explicit_delay_makes_overlap_and_correlation_observable",
                must_assert: "explicit delay makes overlap observable",
            },
            Citation {
                file: DRIVER,
                test_fn: DRIVER_FN,
                must_assert: "M13's timing script (the scripted 2 s delay)",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m5_forced_stale_mapping_window_caps_exposure_and_stays_safe_after_merge",
                must_assert: "M5 forced stale companion timing at both boundary phases",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m8_forced_concurrent_originals_allow_only_one_confirmation_in_flight",
                must_assert: "M8 delayed concurrent originals and delayed confirmation",
            },
            Citation {
                file: "tests/transition_timing.rs",
                test_fn: "m9_forced_phantom_race_at_saturation_recovers_per_m8",
                must_assert: "M9's explicit 2 s arrival delay is the forced \
                              reservation-to-receipt race window the phantom \
                              lands inside",
            },
            Citation {
                file: "tests/capture_replay.rs",
                test_fn: "b12_canonical_sent_to_received_median_is_81_ms",
                must_assert: "all 383 canonical samples produce an 81 ms \
                              median AND DEFAULT_SERVICE_DELAY equals that \
                              computed median (the SD-R5-F8 anchor)",
            },
        ],
        note: "Mechanism, canonical default, and all four required timing \
               scripts (M5, M8, M9, M13) are pinned; the M9 arm landed \
               2026-08-14 (Ballot G). The separate §7.4 fixed-trace mismatch \
               was adjudicated as an expectation error and the \
               feedback-consistent replacement gate landed 2026-08-15 \
               (see s7-4-replay-gate)",
    },
    Clause {
        id: "s7-4-replay-gate",
        owner: "§7.4",
        text: "Feedback-consistent replacement calibration gate per \
               s7-4-replacement-gate.md (ratified 2026-08-15; adoption \
               marker scenarios.md:861); preserve B3, both fixtures, and \
               the fixed-trace counterexample diagnostic (scenarios.md:873)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/capture_replay.rs",
                test_fn: "section_7_4_replacement_gate_holds_on_the_derived_consistent_set",
                must_assert: "the spec-§3 capture-integrity preconditions \
                              (incl. the residue-zero assert migrated from \
                              the deleted open-loop test), the exact \
                              V/HALO/derived-consistent partition \
                              (29,601 + 29,347 + 1,052 = 60,000), and \
                              C1+C2+C3 with the 766-component and anti-echo \
                              anchors at all 18 consistent edges and 24 \
                              stride interiors, plus violation-free \
                              at-least-one-spurious-borderline replays at \
                              all 56 halo edges",
            },
            Citation {
                file: "tests/capture_replay.rs",
                test_fn: "section_7_4_exhaustive_phase_classification_matches_the_pinned_tables",
                must_assert: "every phase in [0, 60,000) re-derives from the \
                              replay alone to exactly the pinned three-way \
                              classification; C2 holds at every phase; the \
                              strict-overstatement envelope over the \
                              consistent set is pinned at 25..57 of 766 \
                              (ignored: exhaustive; a green release run is \
                              part of the review evidence)",
            },
            Citation {
                file: "tests/capture_replay.rs",
                test_fn: "section_7_4_violating_band_edges_are_pinned_for_all_twenty_bands",
                must_assert: "disposition-refuted edges (the gate's sole \
                              decider for them): all 40 band edges produce \
                              exactly the recorded initiating overflow and \
                              their outside neighbors replay cleanly; band \
                              widths sum to 1,052",
            },
            Citation {
                file: "tests/capture_replay.rs",
                test_fn: "section_7_4_exhaustive_enumeration_matches_the_band_table",
                must_assert: "collects the complete actual (phase, initiating \
                              window overflow) set before comparing it with \
                              VIOLATING_BANDS, including every overflowing \
                              window on the initiating reply (ignored: \
                              exhaustive; run explicitly)",
            },
            Citation {
                file: "tests/capture_replay.rs",
                test_fn: "section_7_4_saturation_diagnostic_has_a_43_of_43_witness_phase",
                must_assert: "phase zero retains all 43 recorded saturation \
                              agreements (supporting: an entailed corollary \
                              of C1+C2 at the first consistent band's lower \
                              edge, kept as the unknown-phi diagnostic)",
            },
        ],
        note: "SD-R5-F11 minted the slot; Tom's 2026-08-14 adjudication \
               rejected the open-loop every-phase expectation; the \
               replacement gate was specified, ratified in full 2026-08-15, \
               and implemented the same day (green, incl. the release \
               companion run). The superseded open-loop test is deleted per \
               the spec's §4, its residue-zero assert migrated into the \
               gate's preconditions. HALO and VIOLATING_BANDS are pins under \
               the SD-R5-F13 provenance rule. This discharge fills no \
               verdict slot",
    },
    Clause {
        id: "b13-observation-log",
        owner: "B13",
        text: "Observation log + handoff-before-delay + correlation identity \
               (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b13_records_mock_owned_transport_handoff_before_arrival_delay",
                must_assert: "handoff recorded before the arrival delay",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b13_handoff_survives_arrival_delay_cancellation_and_reserves_identity",
                must_assert: "cancellation survival, identity reserved",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "dropped_mock_transport_future_is_logged_and_ages_out_of_in_flight_state",
                must_assert: "a dropped future is logged and ages out",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b12_b13_explicit_delay_makes_overlap_and_correlation_observable",
                must_assert: "correlation observable under delay",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
                must_assert: "observation log carries the protocol crossing",
            },
        ],
        note: "",
    },
    Clause {
        id: "b14-zero-skew-date",
        owner: "B14",
        text: "Zero-skew Date (scenarios.md:612)",
        coverage: Coverage::Full,
        citations: &[
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology",
                must_assert: "Date crosses the protocol",
            },
            Citation {
                file: "tests/mock_fidelity.rs",
                test_fn: "b14_date_uses_the_scripted_response_completion_instant",
                must_assert: "Date is the scripted completion instant, zero \
                              skew",
            },
        ],
        note: "",
    },
    // ── O1–O8 out-of-scope exclusions (scenarios.md:697, §7.3) ───
    Clause {
        id: "o1-sockets-tls",
        owner: "O1",
        text: "Sockets/TLS/framing out of scope (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: Cargo.toml carries \
               http, httpdate, tokio without net; no HTTP client/server \
               dependency exists",
    },
    Clause {
        id: "o2-stochastic-latency",
        owner: "O2",
        text: "Stochastic latency out of scope (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: all delays are \
               scripted constants (ExchangeScript); proptest randomness is \
               test-input only",
    },
    Clause {
        id: "o3-payloads",
        owner: "O3",
        text: "Payload modeling out of scope (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: mock bodies are empty \
               except the Cloudflare HTML signature; Raw override bodies are \
               script-bounded, not payload modeling",
    },
    Clause {
        id: "o4-multi-scope-semantics",
        owner: "O4",
        text: "Multi-scope semantics out of scope (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: both legacy rules are \
               judged over the single request stream (B7 note); no per-scope \
               machinery anywhere",
    },
    Clause {
        id: "o5-date-skew",
        owner: "O5",
        text: "Date skew out of scope; conditional re-entry trigger armed \
               (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: zero-skew emission is \
               tested (B14); the C1 evidence row records no server-clock \
               input, so the conditional re-entry trigger has not fired",
    },
    Clause {
        id: "o6-header-case-variants",
        owner: "O6",
        text: "Header case/order variants at the mock out of scope \
               (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: the mock emits \
               canonical lowercase (http-crate types); the adversarial-casing \
               domain is C2's \
               (header_case_and_duplicates_resolve_first_value_and_preserve_spelling)",
    },
    Clause {
        id: "o7-auth",
        owner: "O7",
        text: "Auth of any kind out of scope (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: no \
               token/cookie/authorization material in src/, tests/, or \
               fixtures; the sanitizer is allowlist-based \
               (tools/sanitize_capture.py)",
    },
    Clause {
        id: "o8-excluded-regimes",
        owner: "O8",
        text: "U2/U4/forum regime/unlimited endpoints out of scope here \
               (scenarios.md:697)",
        coverage: Coverage::Excluded,
        citations: &[],
        note: "Exclusion verified honored at the audit: pointed at their own \
               registers; nothing here claims them",
    },
];
