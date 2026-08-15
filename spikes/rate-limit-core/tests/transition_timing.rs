use std::collections::BTreeMap;
use std::time::Duration;

use http::Method;
use rate_limit_core::actor::{GateHandle, RequestTicket, spawn, with_correlation_header};
use rate_limit_core::conformance::{
    ContractCoverage, D5_IN_FLIGHT_CAP, ExposureAllowance, Gate, OAUTH_KNOWN_PROFILE,
    ReproductionRecord, RunEvidence, ScenarioAssertion, ScenarioAssertionId, ScenarioId,
    ScenarioOracle, judge,
};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, Resolution};
use rate_limit_core::mock::model::{PolicyDefinition, RuleDefinition, WindowDefinition};
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, ExchangeScript, MockConfig, MockController, MockService,
    Observation, ResponseOverride, request,
};

const STEP: Duration = Duration::from_millis(25);
/// N19's applicable bucket is the maximum configured resolution across the
/// policy's windows — 60 s in both lanes (the Known sustained resolution here,
/// the Assumed pair in the legacy lane) — plus the one-second buffer. Restated
/// as scenario arithmetic, not read from the engine.
const APPLICABLE_BUCKET_AND_BUFFER_MS: u64 = 60_000 + 1_000;
/// N13 padding for one sustained-window hit under the Known profile: the 60 s
/// period plus the 60 s sustained bucket. Scenario arithmetic, not engine
/// state.
const SUSTAINED_PERIOD_PLUS_BUCKET_MS: u64 = 60_000 + 60_000;
/// D5's dispatch floor, restated as the contract's literal (the same rule as
/// the driver's constant): oracle rows must not import the actor's value.
const MIN_SEND_SPACING_MS: u64 = 250;
const PHASES_MS: [u64; 2] = [0, 1];

struct TransitionOracle {
    eligible_ms: BTreeMap<u64, u64>,
    observable_ms: BTreeMap<u64, u64>,
}

impl ScenarioOracle for TransitionOracle {
    fn independently_eligible_ms(&self, observation: &Observation) -> Option<u64> {
        // A missing entry reaches the judge as `None`, which fails G3 closed
        // (SD-R5-F6: the fail-closed branch is the judge's, not ours).
        self.eligible_ms.get(&observation.correlation_id).copied()
    }

    fn independently_observable_ms(
        &self,
        state_change: &rate_limit_core::mock::MockStateChange,
        _: &[Observation],
    ) -> Option<u64> {
        self.observable_ms.get(&state_change.id).copied()
    }
}

/// These focused lanes submit `Endpoint::StashList`, a Known(5s/60s) OAuth
/// policy, so the client runs under the OAuth Known profile (SD-R5-F4): the
/// hand-off's profile rule — OAuth rows use `Known`, only the explicitly
/// legacy M8/M10 lanes use `Assumed` — has no carve-out for focused tests,
/// and the previous `Assumed(60s/60s)` engine paced ~4.7x more conservatively
/// than the lane these clauses bind.
fn engine() -> PolicyEngine {
    PolicyEngine::new(BucketModel::new(
        Resolution::Known(Duration::from_millis(OAUTH_KNOWN_PROFILE.burst_ms)),
        Resolution::Known(Duration::from_millis(OAUTH_KNOWN_PROFILE.sustained_ms)),
    ))
}

#[derive(Debug, Clone, Copy)]
struct M6FragmentFacts {
    organic_violation: bool,
    organic_violation_count: usize,
    recovered_dispatch_ms: u64,
    recovery_bound_ms: u64,
    recovered_violation: bool,
}

impl M6FragmentFacts {
    fn passed(self) -> bool {
        self.organic_violation
            && self.organic_violation_count == 1
            && self.recovered_dispatch_ms >= self.recovery_bound_ms
            && !self.recovered_violation
    }
}

// RE-2 reachability guard: every wire fact carried by the M6 scenario
// assertion can make G5 false. The integration test below hands this result
// directly to `judge`; no duplicate panic-assert may dominate that call.
#[test]
fn m6_fragment_verdict_is_not_constant_true() {
    let passing = M6FragmentFacts {
        organic_violation: true,
        organic_violation_count: 1,
        recovered_dispatch_ms: 200,
        recovery_bound_ms: 200,
        recovered_violation: false,
    };
    assert!(passing.passed());

    for failing in [
        M6FragmentFacts {
            organic_violation: false,
            ..passing
        },
        M6FragmentFacts {
            organic_violation_count: 2,
            ..passing
        },
        M6FragmentFacts {
            recovered_dispatch_ms: 199,
            ..passing
        },
        M6FragmentFacts {
            recovered_violation: true,
            ..passing
        },
    ] {
        assert!(
            !failing.passed(),
            "wire fact must reach G5 false: {failing:?}"
        );
    }
}

fn wire_request(endpoint: Endpoint) -> rate_limit_core::transport::WireRequest {
    with_correlation_header(
        request(Method::GET, endpoint, 0).expect("fixed mock request is valid"),
        CORRELATION_HEADER,
    )
}

fn transition_policy(name: &str, max_hits: u32) -> PolicyDefinition {
    let rule = RuleDefinition::new(
        "Account",
        vec![
            WindowDefinition::new(max_hits, 10_000, 60_000, 5_000)
                .expect("the scripted burst window is valid"),
            WindowDefinition::new(max_hits, 60_000, 300_000, 60_000)
                .expect("the scripted sustained window is valid"),
        ],
    )
    .expect("the scripted rule is valid");
    PolicyDefinition::new(name, vec![rule]).expect("the scripted policy is valid")
}

async fn settle() {
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }
}

async fn advance(duration: Duration) {
    tokio::time::advance(duration).await;
    settle().await;
}

async fn wait_for_handoffs(
    controller: &MockController,
    expected: usize,
    budget: Duration,
    phase_ms: u64,
) {
    let steps = budget.as_millis().div_ceil(STEP.as_millis());
    for _ in 0..steps {
        settle().await;
        if controller.handoffs().await.len() >= expected {
            return;
        }
        advance(STEP).await;
    }
    panic!(
        "phase {phase_ms}: timed out waiting for {expected} handoffs; observed {:?}",
        controller.handoffs().await
    );
}

async fn wait_for_observations(
    controller: &MockController,
    expected: usize,
    budget: Duration,
    phase_ms: u64,
) {
    let steps = budget.as_millis().div_ceil(STEP.as_millis());
    for _ in 0..steps {
        settle().await;
        if controller.observations().await.len() >= expected {
            return;
        }
        advance(STEP).await;
    }
    panic!(
        "phase {phase_ms}: timed out waiting for {expected} observations; observed {:?}",
        controller.observations().await
    );
}

async fn submit(gate: &GateHandle, endpoint: Endpoint) -> RequestTicket {
    gate.submit(
        EndpointLabel::from(endpoint.label()),
        wire_request(endpoint),
    )
    .await
    .expect("the scripted request enters the actor")
}

async fn boot(gate: &GateHandle, controller: &MockController, phase_ms: u64) {
    let ticket = submit(gate, Endpoint::StashList).await;
    wait_for_observations(controller, 2, Duration::from_secs(2), phase_ms).await;
    advance(Duration::from_millis(100)).await;
    assert!(
        ticket
            .await
            .expect("boot caller completes")
            .status()
            .is_success(),
        "phase {phase_ms}: boot GET must succeed"
    );

    let observations = controller.observations().await;
    assert_eq!(observations.len(), 2, "phase {phase_ms}: HEAD + GET");
    assert_eq!(observations[0].method, Method::HEAD);
    assert_eq!(observations[1].method, Method::GET);
}

fn by_correlation(observations: &[Observation], correlation_id: u64) -> &Observation {
    observations
        .iter()
        .find(|observation| observation.correlation_id == correlation_id)
        .unwrap_or_else(|| panic!("missing observation for correlation {correlation_id}"))
}

async fn script_staggered_originals(controller: &MockController) {
    controller
        .script(
            3,
            ExchangeScript {
                arrival_delay: Duration::from_secs(1),
                response_delay: Duration::from_millis(100),
                response: None,
            },
        )
        .await
        .unwrap();
    controller
        .script(
            4,
            ExchangeScript {
                arrival_delay: Duration::from_secs(2),
                response_delay: Duration::from_millis(100),
                response: None,
            },
        )
        .await
        .unwrap();
}

#[tokio::test(start_paused = true)]
async fn m5_forced_stale_mapping_window_caps_exposure_and_stays_safe_after_merge() {
    for phase_ms in PHASES_MS {
        let (mock, controller) = MockService::new(MockConfig::n23(505, phase_ms)).unwrap();
        script_staggered_originals(&controller).await;
        let gate = spawn(engine(), mock);
        boot(&gate, &controller, phase_ms).await;

        let first_stale = submit(&gate, Endpoint::StashList).await;
        let second_stale = submit(&gate, Endpoint::StashList).await;
        let post_merge = submit(&gate, Endpoint::StashList).await;
        wait_for_handoffs(&controller, 4, Duration::from_secs(2), phase_ms).await;

        // Move the server-side rename strictly after both reservations but
        // before either delayed arrival. The first response announces the
        // rename while correlation 4 is still travelling to the server.
        advance(Duration::from_millis(1)).await;
        let rename = controller
            .rename_policy(
                "stash-list-request-limit",
                transition_policy("renamed-limit", 3),
            )
            .await
            .unwrap();

        wait_for_observations(&controller, 3, Duration::from_secs(2), phase_ms).await;
        advance(Duration::from_millis(125)).await;
        settle().await;
        assert_eq!(
            controller.handoffs().await.len(),
            4,
            "phase {phase_ms}: the queued post-merge request must not use the stale limits"
        );

        wait_for_observations(&controller, 4, Duration::from_secs(2), phase_ms).await;
        advance(Duration::from_millis(125)).await;
        assert!(first_stale.await.unwrap().status().is_success());
        assert!(second_stale.await.unwrap().status().is_success());

        let transition_observations = controller.observations().await;
        let first = by_correlation(&transition_observations, 3);
        let second = by_correlation(&transition_observations, 4);
        let stale_handoffs = controller
            .handoffs()
            .await
            .into_iter()
            .filter(|handoff| {
                matches!(handoff.correlation_id, 3 | 4) && handoff.dispatch_ms < rename.occurred_ms
            })
            .count();
        assert_eq!(
            stale_handoffs, D5_IN_FLIGHT_CAP,
            "phase {phase_ms}: the script must reach the full bounded stale set"
        );
        assert!(
            first.arrival_ms > rename.occurred_ms,
            "phase {phase_ms}: rename must precede the announcing arrival"
        );
        assert!(
            second.arrival_ms > first.completion_ms,
            "phase {phase_ms}: correlation 4 must arrive after the rename is observable"
        );
        assert!(
            !first.policy_judgment.organic_violation && !second.policy_judgment.organic_violation,
            "phase {phase_ms}: bounded stale originals remain within the scripted limit"
        );

        wait_for_handoffs(&controller, 5, Duration::from_secs(125), phase_ms).await;
        wait_for_observations(&controller, 5, Duration::from_secs(1), phase_ms).await;
        advance(Duration::from_millis(100)).await;
        assert!(post_merge.await.unwrap().status().is_success());

        let observations = controller.observations().await;
        let opening_get = by_correlation(&observations, 2);
        let after_merge = by_correlation(&observations, 5);
        assert_eq!(after_merge.policy, "renamed-limit");
        assert!(
            after_merge.dispatch_ms >= opening_get.dispatch_ms + SUSTAINED_PERIOD_PLUS_BUCKET_MS,
            "phase {phase_ms}: the post-merge send must wait the independent 60s period + 60s sustained bucket"
        );
        assert!(
            observations
                .iter()
                .filter(|observation| observation.arrival_ms >= first.completion_ms)
                .all(|observation| !observation.policy_judgment.organic_violation),
            "phase {phase_ms}: no client-caused violation may follow the merge"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn m6_forced_preannouncement_original_recovers_at_the_shrunk_pace() {
    for phase_ms in PHASES_MS {
        let (mock, controller) = MockService::new(MockConfig::n23(606, phase_ms)).unwrap();
        script_staggered_originals(&controller).await;
        let gate = spawn(engine(), mock);
        boot(&gate, &controller, phase_ms).await;

        let announcing = submit(&gate, Endpoint::StashList).await;
        let exposed = submit(&gate, Endpoint::StashList).await;
        wait_for_handoffs(&controller, 4, Duration::from_secs(2), phase_ms).await;
        advance(Duration::from_millis(1)).await;
        let shrink = controller
            .replace_policy(transition_policy("stash-list-request-limit", 2))
            .await
            .unwrap();

        wait_for_observations(&controller, 3, Duration::from_secs(2), phase_ms).await;
        advance(Duration::from_millis(125)).await;
        assert!(announcing.await.unwrap().status().is_success());
        assert_eq!(
            controller.handoffs().await.len(),
            4,
            "phase {phase_ms}: no new reservation may escape the announced shrink"
        );

        wait_for_observations(&controller, 4, Duration::from_secs(2), phase_ms).await;
        advance(Duration::from_millis(125)).await;
        let transition_observations = controller.observations().await;
        let announcement = by_correlation(&transition_observations, 3);
        let organic = by_correlation(&transition_observations, 4);
        assert_eq!(
            organic.response_status,
            Some(http::StatusCode::TOO_MANY_REQUESTS)
        );
        assert!(
            organic.dispatch_ms < shrink.occurred_ms,
            "phase {phase_ms}: the violating original must have been reserved before the shrink"
        );
        assert!(
            organic.arrival_ms > announcement.completion_ms,
            "phase {phase_ms}: the bounded exposure must land after the shrink is observable"
        );
        wait_for_handoffs(&controller, 5, Duration::from_secs(500), phase_ms).await;
        wait_for_observations(&controller, 5, Duration::from_secs(1), phase_ms).await;
        advance(Duration::from_millis(100)).await;
        assert!(exposed.await.unwrap().status().is_success());

        let observations = controller.observations().await;
        let recovered = by_correlation(&observations, 5);
        let retry_after_ms = organic
            .policy_judgment
            .retry_after_seconds
            .expect("an organic mock 429 carries Retry-After")
            * 1_000;
        let recovery_bound_ms =
            organic.completion_ms + retry_after_ms + APPLICABLE_BUCKET_AND_BUFFER_MS;
        // RE-2: this is the sole decider for the four M6 scenario facts. A
        // broken fact reaches G5 and the report below; it is not intercepted
        // by a duplicate panic-assert first.
        let fragment_passed = M6FragmentFacts {
            organic_violation: organic.policy_judgment.organic_violation,
            organic_violation_count: observations
                .iter()
                .filter(|observation| observation.policy_judgment.organic_violation)
                .count(),
            recovered_dispatch_ms: recovered.dispatch_ms,
            recovery_bound_ms,
            recovered_violation: recovered.policy_judgment.organic_violation,
        }
        .passed();

        // Exercise the public attribution seam rather than merely declaring
        // correlation 4 exceptional. Eligibility is separately scripted and
        // fail-closed; the shrink becomes observable at response completion,
        // not at the mock-side mutation instant. The boot HEAD's row anchors
        // at the script-owned submission instant (t=0), not at its own
        // observed dispatch (SD-R5-F7's self-referential row could never
        // fail G3).
        let eligible_ms = BTreeMap::from([
            (1, 0),
            (2, observations[0].dispatch_ms + MIN_SEND_SPACING_MS),
            (3, observations[1].dispatch_ms + MIN_SEND_SPACING_MS),
            (4, observations[2].dispatch_ms + MIN_SEND_SPACING_MS),
            (5, recovery_bound_ms),
        ]);
        let oracle = TransitionOracle {
            eligible_ms,
            observable_ms: BTreeMap::from([(shrink.id, announcement.completion_ms)]),
        };
        let evidence = RunEvidence {
            scenario: ScenarioId::M6,
            reproduction: Some(ReproductionRecord {
                seed: 606,
                phase_ms,
                client_buckets: OAUTH_KNOWN_PROFILE,
                endpoint: Endpoint::StashList,
            }),
            observations,
            state_changes: controller.state_changes().await,
            unavoidable_exposure: Some(
                ExposureAllowance::for_state_change(
                    shrink.id,
                    [(organic.correlation_id, organic.dispatch_ms)],
                    D5_IN_FLIGHT_CAP - 1,
                )
                .expect("one pre-observation companion fits the transition exposure bound"),
            ),
            assertions: vec![ScenarioAssertion {
                id: ScenarioAssertionId::M6Shrink,
                coverage: ContractCoverage::Fragment,
                passed: fragment_passed,
            }],
        };
        let report = judge(&evidence, &oracle).expect("the exposure evidence is structural");
        assert!(report.passed(), "phase {phase_ms}: {report:?}");
        assert!(report.gate(Gate::G1).passed);
        assert!(
            !report.verdict_eligible(),
            "the focused M6 timing arm remains a fragment"
        );
    }
}

/// The stash-request-limit burst window from the N23 definition
/// (15 hits / 10 s), independently declared — M9's "at 14/15" is this
/// window one short of saturation.
const M9_BURST_LIMIT: u32 = 15;
const M9_PRELOADED_RESIDUE: u32 = M9_BURST_LIMIT - 1;

/// The wire facts M9's forced race must establish. They reach the judge as
/// the M9 scenario assertion (the RE-2 sole-decider pattern); the
/// falsifiability test below proves each one can make the verdict false.
#[derive(Debug, Clone, Copy)]
struct M9RaceFacts {
    /// The raced send drew an *organic* 429 — mock counter arithmetic, not
    /// an injected stimulus.
    raced_organic: bool,
    organic_violation_count: usize,
    /// The phantom landed strictly between the raced send's transport
    /// hand-off and its mock receipt — the §2 race window, forced by the
    /// B12 scripted delay.
    phantom_after_handoff: bool,
    phantom_before_receipt: bool,
    /// The 14/15 construction, read off the mock's own burst-window
    /// judgment at the raced arrival: 14 residue + 1 phantom + 1 client
    /// = 16 over the limit of 15.
    burst_residue_hits: u32,
    burst_phantom_hits: u32,
    burst_client_hits: u32,
    burst_current_hits: u32,
    /// M8's recovery asserts: one confirmation in flight, N19 wait, no
    /// follow-on violation.
    single_confirmation: bool,
    retry_dispatch_ms: u64,
    recovery_bound_ms: u64,
    retry_organic: bool,
}

impl M9RaceFacts {
    fn passed(self) -> bool {
        self.raced_organic
            && self.organic_violation_count == 1
            && self.phantom_after_handoff
            && self.phantom_before_receipt
            && self.burst_residue_hits == M9_PRELOADED_RESIDUE
            && self.burst_phantom_hits == 1
            && self.burst_client_hits == 1
            && self.burst_current_hits == M9_BURST_LIMIT + 1
            && self.single_confirmation
            && self.retry_dispatch_ms >= self.recovery_bound_ms
            && !self.retry_organic
    }
}

// RE-2 reachability guard: every wire fact carried by the M9 scenario
// assertion can make G5 false on its own.
#[test]
fn m9_race_verdict_is_not_constant_true() {
    let passing = M9RaceFacts {
        raced_organic: true,
        organic_violation_count: 1,
        phantom_after_handoff: true,
        phantom_before_receipt: true,
        burst_residue_hits: 14,
        burst_phantom_hits: 1,
        burst_client_hits: 1,
        burst_current_hits: 16,
        single_confirmation: true,
        retry_dispatch_ms: 126_500,
        recovery_bound_ms: 126_500,
        retry_organic: false,
    };
    assert!(passing.passed());

    for failing in [
        M9RaceFacts {
            raced_organic: false,
            ..passing
        },
        M9RaceFacts {
            organic_violation_count: 2,
            ..passing
        },
        M9RaceFacts {
            phantom_after_handoff: false,
            ..passing
        },
        M9RaceFacts {
            phantom_before_receipt: false,
            ..passing
        },
        M9RaceFacts {
            burst_residue_hits: 13,
            ..passing
        },
        M9RaceFacts {
            burst_phantom_hits: 0,
            ..passing
        },
        M9RaceFacts {
            burst_client_hits: 2,
            ..passing
        },
        M9RaceFacts {
            burst_current_hits: 15,
            ..passing
        },
        M9RaceFacts {
            single_confirmation: false,
            ..passing
        },
        M9RaceFacts {
            retry_dispatch_ms: 126_499,
            ..passing
        },
        M9RaceFacts {
            retry_organic: true,
            ..passing
        },
    ] {
        assert!(
            !failing.passed(),
            "wire fact must reach G5 false: {failing:?}"
        );
    }
}

/// M9's forced saturation race (Ballot G): at 14/15 a mock-owned phantom
/// consumes the last slot inside the scripted reservation-to-receipt window,
/// the client's send lands as the 16th and draws an organic 429, and M8's
/// recovery machinery carries it home. The exposure is attributed through
/// the public §2 seam — `ExposureAllowance` bound to the phantom state
/// change via B13 correlation identity — and the same evidence without the
/// allowance is proven to fail G1, so the attribution is load-bearing.
#[tokio::test(start_paused = true)]
async fn m9_forced_phantom_race_at_saturation_recovers_per_m8() {
    for phase_ms in PHASES_MS {
        let (mock, controller) = MockService::new(MockConfig::n23(909, phase_ms)).unwrap();
        controller
            .preload(
                "stash-request-limit",
                controller.now(),
                M9_PRELOADED_RESIDUE as usize,
            )
            .await
            .unwrap();
        // B12's M9 timing script: a 2 s server arrival delay opens the
        // reservation-to-receipt window the phantom must land inside.
        controller
            .script(
                2,
                ExchangeScript {
                    arrival_delay: Duration::from_secs(2),
                    response_delay: Duration::from_millis(100),
                    response: None,
                },
            )
            .await
            .unwrap();
        let gate = spawn(engine(), mock);
        let ticket = submit(&gate, Endpoint::Stash).await;

        wait_for_handoffs(&controller, 2, Duration::from_secs(2), phase_ms).await;
        advance(Duration::from_millis(1)).await;
        let phantom = controller
            .inject_phantoms("stash-request-limit", 1)
            .await
            .unwrap();

        wait_for_observations(&controller, 2, Duration::from_secs(3), phase_ms).await;
        advance(Duration::from_millis(200)).await;
        let raced_observations = controller.observations().await;
        let head = by_correlation(&raced_observations, 1);
        let raced = by_correlation(&raced_observations, 2);
        assert_eq!(
            raced.response_status,
            Some(http::StatusCode::TOO_MANY_REQUESTS),
            "phase {phase_ms}: the raced 16th hit must draw an organic 429"
        );
        let retry_after_ms = raced
            .policy_judgment
            .retry_after_seconds
            .expect("an organic mock 429 carries Retry-After")
            * 1_000;
        let recovery_bound_ms =
            raced.completion_ms + retry_after_ms + APPLICABLE_BUCKET_AND_BUFFER_MS;

        advance(Duration::from_secs(3)).await;
        assert_eq!(
            controller.handoffs().await.len(),
            2,
            "phase {phase_ms}: the raced caller must not immediately re-knock"
        );

        wait_for_handoffs(&controller, 3, Duration::from_secs(130), phase_ms).await;
        wait_for_observations(&controller, 3, Duration::from_secs(1), phase_ms).await;
        advance(Duration::from_millis(200)).await;
        assert!(
            ticket.await.unwrap().status().is_success(),
            "phase {phase_ms}: the raced caller eventually observes the recovered outcome"
        );

        let observations = controller.observations().await;
        let retry = by_correlation(&observations, 3);
        // The N23 stash-request rule orders its windows [burst, sustained].
        let raced_burst = &raced.policy_judgment.windows[0];
        let facts = M9RaceFacts {
            raced_organic: raced.policy_judgment.organic_violation,
            organic_violation_count: observations
                .iter()
                .filter(|observation| observation.policy_judgment.organic_violation)
                .count(),
            phantom_after_handoff: phantom.occurred_ms > raced.dispatch_ms,
            phantom_before_receipt: phantom.occurred_ms < raced.arrival_ms,
            burst_residue_hits: raced_burst.residue_hits,
            burst_phantom_hits: raced_burst.phantom_hits,
            burst_client_hits: raced_burst.client_hits,
            burst_current_hits: raced_burst.current_hits,
            single_confirmation: observations.len() == 3 && retry.in_flight_at_arrival == 1,
            retry_dispatch_ms: retry.dispatch_ms,
            recovery_bound_ms,
            retry_organic: retry.policy_judgment.organic_violation,
        };

        // Independent eligibility: floor-only for the pre-race rows, the N19
        // recovery bound for the confirmation. The exposure allowance binds
        // the raced reservation to the phantom state change; the phantom
        // first becomes client-observable at the raced response's
        // completion.
        let eligible_ms = BTreeMap::from([
            (1, 0),
            (2, head.dispatch_ms + MIN_SEND_SPACING_MS),
            (3, recovery_bound_ms),
        ]);
        let oracle = TransitionOracle {
            eligible_ms,
            observable_ms: BTreeMap::from([(phantom.id, raced.completion_ms)]),
        };
        let evidence = RunEvidence {
            scenario: ScenarioId::M9,
            reproduction: Some(ReproductionRecord {
                seed: 909,
                phase_ms,
                client_buckets: OAUTH_KNOWN_PROFILE,
                // This lane submits to Stash (stash-request-limit phantoms),
                // unlike the file's other lanes; the judge binds this label
                // to the wire (SD-R8-F9) and caught the earlier StashList
                // mislabel here.
                endpoint: Endpoint::Stash,
            }),
            observations,
            state_changes: controller.state_changes().await,
            unavoidable_exposure: Some(
                ExposureAllowance::for_state_change(
                    phantom.id,
                    [(raced.correlation_id, raced.dispatch_ms)],
                    1,
                )
                .expect("one in-flight racer fits the §2 race exposure bound"),
            ),
            assertions: vec![ScenarioAssertion {
                id: ScenarioAssertionId::M9PhantomRace,
                coverage: ContractCoverage::Fragment,
                passed: facts.passed(),
            }],
        };
        let report = judge(&evidence, &oracle).expect("the race exposure evidence is structural");
        assert!(report.passed(), "phase {phase_ms}: {facts:?} {report:?}");
        assert!(report.gate(Gate::G1).passed);
        assert!(
            !report.verdict_eligible(),
            "the focused M9 race arm remains a fragment"
        );

        // Attribution teeth: the identical evidence without the allowance
        // must fail G1 on the raced correlation — the organic violation is
        // real and only the attributed race exposure forgives it.
        let unattributed = RunEvidence {
            unavoidable_exposure: None,
            ..evidence
        };
        let unforgiven =
            judge(&unattributed, &oracle).expect("the unattributed evidence stays structural");
        assert!(
            !unforgiven.gate(Gate::G1).passed,
            "phase {phase_ms}: without the race allowance the organic 429 must fail G1"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn m8_forced_concurrent_originals_allow_only_one_confirmation_in_flight() {
    for phase_ms in PHASES_MS {
        let (mock, controller) = MockService::new(MockConfig::n23(808, phase_ms)).unwrap();
        for correlation_id in [3, 4] {
            controller
                .script(
                    correlation_id,
                    ExchangeScript {
                        arrival_delay: Duration::ZERO,
                        response_delay: Duration::from_secs(2),
                        response: Some(ResponseOverride::Full {
                            status: http::StatusCode::TOO_MANY_REQUESTS,
                            retry_after: Some("0".to_owned()),
                        }),
                    },
                )
                .await
                .unwrap();
        }
        controller
            .script(
                5,
                ExchangeScript {
                    arrival_delay: Duration::ZERO,
                    response_delay: Duration::from_secs(5),
                    response: None,
                },
            )
            .await
            .unwrap();

        let gate = spawn(engine(), mock);
        boot(&gate, &controller, phase_ms).await;
        let first = submit(&gate, Endpoint::StashList).await;
        let second = submit(&gate, Endpoint::StashList).await;

        wait_for_observations(&controller, 4, Duration::from_secs(2), phase_ms).await;
        let originals = controller.observations().await;
        let first_original = by_correlation(&originals, 3);
        let second_original = by_correlation(&originals, 4);
        assert_eq!(
            first_original.response_status,
            Some(http::StatusCode::TOO_MANY_REQUESTS)
        );
        assert_eq!(
            second_original.response_status,
            Some(http::StatusCode::TOO_MANY_REQUESTS)
        );
        assert_eq!(first_original.in_flight_at_arrival, 1);
        assert_eq!(second_original.in_flight_at_arrival, D5_IN_FLIGHT_CAP);
        assert!(
            second_original.dispatch_ms < first_original.completion_ms,
            "phase {phase_ms}: the explicit delays must force concurrent originals"
        );
        let originals_complete_ms = first_original
            .completion_ms
            .max(second_original.completion_ms);

        advance(Duration::from_secs(3)).await;
        assert_eq!(
            controller.handoffs().await.len(),
            4,
            "phase {phase_ms}: neither original may immediately re-knock"
        );

        wait_for_handoffs(&controller, 5, Duration::from_secs(65), phase_ms).await;
        wait_for_observations(&controller, 5, Duration::from_secs(1), phase_ms).await;
        let confirmation = controller.observations().await;
        let confirmation = by_correlation(&confirmation, 5);
        assert!(
            confirmation.dispatch_ms >= originals_complete_ms + APPLICABLE_BUCKET_AND_BUFFER_MS,
            "phase {phase_ms}: confirmation must wait past both original 429 observations"
        );
        assert_eq!(confirmation.in_flight_at_arrival, 1);

        advance(Duration::from_secs(1)).await;
        assert_eq!(
            controller.handoffs().await.len(),
            5,
            "phase {phase_ms}: the sibling retry must stay blocked while confirmation 5 is in flight"
        );
        assert_eq!(
            gate.subscribe_status().borrow().ordinary_in_flight,
            1,
            "phase {phase_ms}: exactly one post-restriction confirmation is in flight"
        );

        wait_for_handoffs(&controller, 6, Duration::from_secs(6), phase_ms).await;
        wait_for_observations(&controller, 6, Duration::from_secs(1), phase_ms).await;
        advance(Duration::from_millis(100)).await;
        assert!(first.await.unwrap().status().is_success());
        assert!(second.await.unwrap().status().is_success());

        let observations = controller.observations().await;
        let sibling_confirmation = by_correlation(&observations, 6);
        assert!(
            sibling_confirmation.dispatch_ms >= confirmation.completion_ms,
            "phase {phase_ms}: normal concurrency may resume only after confirmation 5 completes"
        );
        assert_eq!(
            observations
                .iter()
                .filter(|observation| observation.response_status
                    == Some(http::StatusCode::TOO_MANY_REQUESTS))
                .map(|observation| observation.correlation_id)
                .collect::<Vec<_>>(),
            vec![3, 4],
            "phase {phase_ms}: both and only the forced originals receive the stimulus"
        );
        assert!(
            observations
                .iter()
                .all(|observation| !observation.policy_judgment.organic_violation),
            "phase {phase_ms}: injected recovery must produce no follow-on violation"
        );
    }
}
