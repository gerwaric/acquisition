use std::collections::BTreeMap;
use std::time::Duration;

use http::Method;
use rate_limit_core::actor::{GateHandle, RequestTicket, spawn, with_correlation_header};
use rate_limit_core::conformance::{
    ContractCoverage, D5_IN_FLIGHT_CAP, ExposureAllowance, Gate, ReproductionRecord, RunEvidence,
    SHIPPED_ASSUMED_PROFILE, ScenarioAssertion, ScenarioAssertionId, ScenarioId, ScenarioOracle,
    judge,
};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, Resolution};
use rate_limit_core::mock::model::{PolicyDefinition, RuleDefinition, WindowDefinition};
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, ExchangeScript, MockConfig, MockController, MockService,
    Observation, ResponseOverride, request,
};

const STEP: Duration = Duration::from_millis(25);
const ASSUMED_BUCKET_AND_BUFFER_MS: u64 = 61_000;
const PHASES_MS: [u64; 2] = [0, 1];

struct TransitionOracle {
    eligible_ms: BTreeMap<u64, u64>,
    observable_ms: BTreeMap<u64, u64>,
}

impl ScenarioOracle for TransitionOracle {
    fn independently_eligible_ms(&self, observation: &Observation) -> u64 {
        self.eligible_ms
            .get(&observation.correlation_id)
            .copied()
            .unwrap_or(u64::MAX)
    }

    fn independently_observable_ms(
        &self,
        state_change: &rate_limit_core::mock::MockStateChange,
        _: &[Observation],
    ) -> Option<u64> {
        self.observable_ms.get(&state_change.id).copied()
    }
}

fn engine() -> PolicyEngine {
    PolicyEngine::new(BucketModel::new(
        Resolution::Assumed(Duration::from_secs(60)),
        Resolution::Assumed(Duration::from_secs(60)),
    ))
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
            after_merge.dispatch_ms >= opening_get.dispatch_ms + 120_000,
            "phase {phase_ms}: the post-merge send must wait the independent 60s period + 60s assumed bucket"
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
        assert!(organic.policy_judgment.organic_violation);
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
        assert_eq!(
            transition_observations
                .iter()
                .filter(|observation| observation.policy_judgment.organic_violation)
                .count(),
            1,
            "phase {phase_ms}: exposure is bounded to the one stale companion"
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
        assert!(
            recovered.dispatch_ms
                >= organic.completion_ms + retry_after_ms + ASSUMED_BUCKET_AND_BUFFER_MS,
            "phase {phase_ms}: recovery must wait Retry-After + 60s bucket + 1s buffer"
        );
        assert!(
            !recovered.policy_judgment.organic_violation,
            "phase {phase_ms}: recovery may not create a follow-on violation"
        );

        // Exercise the public attribution seam rather than merely declaring
        // correlation 4 exceptional. Eligibility is separately scripted and
        // fail-closed; the shrink becomes observable at response completion,
        // not at the mock-side mutation instant.
        let eligible_ms = BTreeMap::from([
            (1, observations[0].dispatch_ms),
            (2, observations[0].dispatch_ms + 250),
            (3, observations[1].dispatch_ms + 250),
            (4, observations[2].dispatch_ms + 250),
            (
                5,
                organic.completion_ms + retry_after_ms + ASSUMED_BUCKET_AND_BUFFER_MS,
            ),
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
                client_buckets: SHIPPED_ASSUMED_PROFILE,
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
                passed: true,
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
            confirmation.dispatch_ms >= originals_complete_ms + ASSUMED_BUCKET_AND_BUFFER_MS,
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
