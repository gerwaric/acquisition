//! End-to-end M-series driver.
//!
//! This target deliberately consumes the actor through its public handle and
//! takes gate evidence only from the mock.  The expected eligibility instants
//! below are scenario-script arithmetic, not values reported by the actor.

use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::time::Duration;

use http::{HeaderMap, HeaderValue, Method, StatusCode};
use rate_limit_core::actor::{GateError, spawn, with_correlation_header};
use rate_limit_core::conformance::{
    ClientBucketProfile, ContractCoverage, Gate, ReproductionRecord, RunEvidence,
    SHIPPED_ASSUMED_PROFILE, ScenarioAssertion, ScenarioId, ScenarioOracle, judge, scenario,
};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, Resolution};
use rate_limit_core::mock::model::{
    PolicyDefinition, RuleDefinition, WindowDefinition, first_bucket_boundary_ms,
};
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, ExchangeScript, MockConfig, MockController, MockService,
    MockStateChange, ResponseOverride, request,
};

fn engine(profile: ClientBucketProfile) -> PolicyEngine {
    let resolution = |millis| {
        if profile == SHIPPED_ASSUMED_PROFILE {
            Resolution::Assumed(Duration::from_millis(millis))
        } else {
            Resolution::Known(Duration::from_millis(millis))
        }
    };
    PolicyEngine::new(BucketModel::new(
        resolution(profile.burst_ms),
        resolution(profile.sustained_ms),
    ))
}

fn wire_request(endpoint: Endpoint) -> rate_limit_core::transport::WireRequest {
    with_correlation_header(
        request(Method::GET, endpoint, 0).expect("fixed mock request is valid"),
        CORRELATION_HEADER,
    )
}

async fn advance(millis: u64) {
    for _ in 0..millis.div_ceil(250) {
        tokio::time::advance(Duration::from_millis(250)).await;
        tokio::task::yield_now().await;
    }
}

fn pair_policy(name: &str, burst: u32) -> PolicyDefinition {
    PolicyDefinition::new(
        name,
        vec![
            RuleDefinition::new(
                "Account",
                vec![
                    WindowDefinition::new(burst, 10_000, 60_000, 5_000).unwrap(),
                    WindowDefinition::new(30, 60_000, 300_000, 60_000).unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn policy_headers(name: &str, triplets: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-rate-limit-policy", HeaderValue::from_str(name).unwrap());
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert(
        "x-rate-limit-account",
        HeaderValue::from_str(triplets).unwrap(),
    );
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_static("0:10:0, 0:60:0"),
    );
    headers
}

#[derive(Default)]
struct DriverOracle {
    /// Explicit scenario-script safe instants keyed by mock correlation.
    /// HEADs are independent probe exclusions, so their own handoff is their
    /// first eligible instant.
    eligible: BTreeMap<u64, u64>,
    m2_minimum_ms: Option<u64>,
}

impl ScenarioOracle for DriverOracle {
    fn independently_eligible_ms(&self, observation: &rate_limit_core::mock::Observation) -> u64 {
        self.eligible
            .get(&observation.correlation_id)
            .copied()
            .unwrap_or(observation.dispatch_ms)
    }

    fn independently_observable_ms(
        &self,
        state_change: &MockStateChange,
        _: &[rate_limit_core::mock::Observation],
    ) -> Option<u64> {
        Some(state_change.occurred_ms)
    }

    fn m2_theoretical_padded_minimum_ms(
        &self,
        _: &[rate_limit_core::mock::Observation],
    ) -> Option<u64> {
        self.m2_minimum_ms
    }
}

async fn evidence(
    controller: &MockController,
    id: ScenarioId,
    profile: ClientBucketProfile,
    assertion_passed: bool,
) -> RunEvidence {
    let spec = scenario(id);
    RunEvidence {
        scenario: id,
        reproduction: (spec.sweep == rate_limit_core::conformance::SweepKind::PhaseSwept)
            .then_some(ReproductionRecord {
                seed: controller
                    .observations()
                    .await
                    .first()
                    .expect("wire run")
                    .seed,
                phase_ms: controller
                    .observations()
                    .await
                    .first()
                    .expect("wire run")
                    .phase_ms,
                client_buckets: profile,
            }),
        observations: controller.observations().await,
        state_changes: controller.state_changes().await,
        unavoidable_exposure: None,
        assertions: vec![ScenarioAssertion {
            // Every row here runs a fragment of its scenario contract: the
            // per-row deltas are listed in `result-draft.md`. Declaring
            // `Fragment` is what keeps a green G5 from reading as a verdict.
            id: spec.required_assertion,
            coverage: ContractCoverage::Fragment,
            passed: assertion_passed,
        }],
    }
}

async fn run_m1_m13(phase_ms: u64) {
    // The two profile values make the known OAuth and shipped assumed legacy
    // lanes explicit.  Phase-swept rows run each representative phase below.
    let profiles = [
        (
            rate_limit_core::conformance::OAUTH_KNOWN_PROFILE,
            Endpoint::StashList,
        ),
        (SHIPPED_ASSUMED_PROFILE, Endpoint::LegacyStashIndex),
    ];
    let mut reports = Vec::new();

    for (index, id) in ScenarioId::ALL.into_iter().enumerate() {
        let (profile, endpoint) = profiles[index % profiles.len()];
        // The phase is the caller's, verbatim: every row runs at each swept
        // phase.  Folding the offset through a per-row modulus (the original
        // `(offset + index * 997) % 60_000`) collapsed the two sweeps to 1 ms
        // apart for every row but M1, because 59_999 == -1 (mod 60_000).
        let mut config = MockConfig::n23(100 + index as u64, phase_ms);
        config.dispatch_budget = 128;
        let (mock, controller) = MockService::new(config).unwrap();
        let gate = spawn(engine(profile), mock);

        // Each arm yields its scenario fragment's verdict rather than
        // asserting it, so a failure reaches G5 and is reported with the
        // whole gate matrix instead of panicking in isolation.
        let assertion_passed = match id {
            ScenarioId::M1 => {
                controller
                    .preload("stash-list-request-limit", controller.now(), 1)
                    .await
                    .unwrap();
                let ticket = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                advance(2_000).await;
                let granted = ticket.await.is_ok();
                let observations = controller.observations().await;
                granted
                    && observations
                        .iter()
                        .filter(|o| o.method == Method::HEAD)
                        .count()
                        == 1
                    && observations.len() == 2
            }
            ScenarioId::M2 => {
                let mut tickets = Vec::new();
                for _ in 0..10 {
                    tickets.push(
                        gate.submit(
                            EndpointLabel::from(Endpoint::StashList.label()),
                            wire_request(Endpoint::StashList),
                        )
                        .await
                        .unwrap(),
                    );
                }
                advance(6_000).await;
                let mut granted = true;
                for ticket in tickets {
                    granted &= ticket.await.is_ok();
                }
                granted
                    && controller
                        .observations()
                        .await
                        .iter()
                        .filter(|o| o.method == Method::GET)
                        .count()
                        == 10
            }
            ScenarioId::M3 => {
                controller
                    .script(
                        1,
                        ExchangeScript {
                            response: Some(ResponseOverride::PolicyOnly),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = gate
                    .submit(
                        EndpointLabel::from(endpoint.label()),
                        wire_request(endpoint),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let refused = matches!(ticket.await, Err(GateError::SetupFailed { .. }));
                refused && controller.observations().await.len() == 1
            }
            ScenarioId::M4 => {
                controller
                    .script(
                        1,
                        ExchangeScript {
                            response: Some(ResponseOverride::Raw {
                                status: StatusCode::NO_CONTENT,
                                headers: policy_headers("bad-shape", "1:10:60"),
                                body: Vec::new(),
                            }),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = gate
                    .submit(
                        EndpointLabel::from(endpoint.label()),
                        wire_request(endpoint),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let refused = matches!(ticket.await, Err(GateError::SetupFailed { .. }));
                refused && controller.observations().await.len() == 1
            }
            ScenarioId::M5 => {
                let first = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .rename_policy("stash-list-request-limit", pair_policy("renamed-limit", 10))
                    .await
                    .unwrap();
                let second = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let second_granted = second.await.is_ok();
                first_granted
                    && second_granted
                    && controller
                        .observations()
                        .await
                        .iter()
                        .filter(|o| o.method == Method::HEAD)
                        .count()
                        == 1
            }
            ScenarioId::M6 => {
                let first = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .replace_policy(pair_policy("stash-list-request-limit", 5))
                    .await
                    .unwrap();
                let second = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                first_granted && second.await.is_ok()
            }
            ScenarioId::M7 => {
                let first = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .inject_phantoms("stash-list-request-limit", 2)
                    .await
                    .unwrap();
                let second = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                first_granted && second.await.is_ok()
            }
            ScenarioId::M8 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response: Some(ResponseOverride::Full {
                                status: StatusCode::TOO_MANY_REQUESTS,
                                retry_after: Some("0".to_owned()),
                            }),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = gate
                    .submit(
                        EndpointLabel::from(endpoint.label()),
                        wire_request(endpoint),
                    )
                    .await
                    .unwrap();
                advance(70_000).await;
                let retried = ticket.await.is_ok();
                retried
                    && controller
                        .observations()
                        .await
                        .iter()
                        .filter(|o| o.method == Method::GET)
                        .count()
                        == 2
            }
            ScenarioId::M9 => {
                let first = gate
                    .submit(
                        EndpointLabel::from(Endpoint::Stash.label()),
                        wire_request(Endpoint::Stash),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .inject_phantoms("stash-request-limit", 1)
                    .await
                    .unwrap();
                let second = gate
                    .submit(
                        EndpointLabel::from(Endpoint::Stash.label()),
                        wire_request(Endpoint::Stash),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                first_granted && second.await.is_ok()
            }
            ScenarioId::M10 => {
                // Keep the first ordinary request on the wire while its
                // caller disappears.  The actor must retain the reservation
                // and still consume the response; only the caller detaches.
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response_delay: Duration::from_secs(5),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let mut tickets = Vec::new();
                tickets.push(
                    gate.submit(
                        EndpointLabel::from(endpoint.label()),
                        wire_request(endpoint),
                    )
                    .await
                    .unwrap(),
                );
                advance(1_000).await;
                drop(tickets.pop().unwrap());
                for _ in 0..15 {
                    tickets.push(
                        gate.submit(
                            EndpointLabel::from(endpoint.label()),
                            wire_request(endpoint),
                        )
                        .await
                        .unwrap(),
                    );
                }
                tickets.pop().unwrap().cancel().await.unwrap();
                advance(80_000).await;
                // Every surviving caller must be served, and the wire count is
                // pinned exactly: the boot HEAD, the dropped caller's request
                // (which stays dispatched), and the fourteen that remain --
                // the cancelled one never reaches the wire.  A `>= 2` bound
                // here would pass even if the caller drop wedged the queue.
                let mut served = 0usize;
                for ticket in tickets {
                    served += usize::from(ticket.await.is_ok());
                }
                served == 14 && controller.observations().await.len() == 16
            }
            ScenarioId::M11 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response: Some(ResponseOverride::Cloudflare),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = gate
                    .submit(
                        EndpointLabel::from(endpoint.label()),
                        wire_request(endpoint),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let halted_caller = matches!(ticket.await, Err(GateError::Halted));
                halted_caller && gate.subscribe_status().borrow().halted
            }
            ScenarioId::M12 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response: Some(ResponseOverride::Full {
                                status: StatusCode::UNAUTHORIZED,
                                retry_after: None,
                            }),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = gate
                    .submit(
                        EndpointLabel::from(endpoint.label()),
                        wire_request(endpoint),
                    )
                    .await
                    .unwrap();
                advance(1_000).await;
                let completed = ticket.await.is_ok();
                completed
                    && controller
                        .observations()
                        .await
                        .iter()
                        .filter(|o| o.method == Method::GET)
                        .count()
                        == 1
            }
            ScenarioId::M13 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response_delay: Duration::from_secs(2),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let first = gate
                    .submit(
                        EndpointLabel::from(Endpoint::StashList.label()),
                        wire_request(Endpoint::StashList),
                    )
                    .await
                    .unwrap();
                let second = gate
                    .submit(
                        EndpointLabel::from(Endpoint::Stash.label()),
                        wire_request(Endpoint::Stash),
                    )
                    .await
                    .unwrap();
                advance(6_000).await;
                let first_granted = first.await.is_ok();
                let second_granted = second.await.is_ok();
                first_granted
                    && second_granted
                    && controller
                        .observations()
                        .await
                        .iter()
                        .all(|o| o.in_flight_at_arrival <= 2 && !o.head_overlap)
            }
        };

        let evidence = evidence(&controller, id, profile, assertion_passed).await;
        let mut oracle = independently_scripted_oracle(id, &evidence.observations);
        if id == ScenarioId::M2 {
            // One boot HEAD at t=0, ten GETs at the 250ms D5 floor, and the
            // mock's fixed 50ms completion latency.  This is plain integer
            // arithmetic over the scenario script, not actor scheduling code.
            oracle.m2_minimum_ms = Some(2_550);
        }
        let report = judge(&evidence, &oracle).unwrap();
        assert!(report.passed(), "{id:?}: {report:?}");
        reports.push(report);
    }

    assert_eq!(
        reports.len(),
        ScenarioId::ALL.len(),
        "reachability: every M row ran"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.gate(Gate::G1).passed && report.gate(Gate::G2).passed)
    );
    // Coverage guard: this driver runs fragments, so no report here may be
    // read as fillable verdict evidence.  When a row's full contract lands,
    // this assertion is what forces the claim to be revisited deliberately.
    assert!(
        reports.iter().all(|report| !report.verdict_eligible()),
        "no fragment run may be verdict-eligible"
    );
}

async fn run_m8_oauth_lane(phase_ms: u64) {
    let profile = rate_limit_core::conformance::OAUTH_KNOWN_PROFILE;
    let (mock, controller) = MockService::new(MockConfig::n23(808, phase_ms)).unwrap();
    controller
        .script(
            2,
            ExchangeScript {
                response: Some(ResponseOverride::Full {
                    status: StatusCode::TOO_MANY_REQUESTS,
                    retry_after: Some("0".to_owned()),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(profile), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::Stash.label()),
            wire_request(Endpoint::Stash),
        )
        .await
        .unwrap();
    advance(70_000).await;
    let retried = ticket.await.is_ok();
    let evidence = evidence(&controller, ScenarioId::M8, profile, retried).await;
    let report = judge(
        &evidence,
        &independently_scripted_oracle(ScenarioId::M8, &evidence.observations),
    )
    .unwrap();
    assert!(report.passed(), "OAuth M8: {report:?}");
}

/// The two swept phases sit at opposite ends of the *boundary distance*, which
/// is the quantity that actually varies the server's bucket alignment: phi=0
/// puts the first boundary a full bucket away, phi=1 puts it 1 ms after t0.
///
/// `phase_ms` names the upcoming boundary, so phi=59,999 is 1 ms from phi=0,
/// not from an immediate boundary — round one's F1 and the re-review's F7 were
/// both that misreading. `swept_phases_are_separated_by_a_full_bucket` pins
/// the real distances so a third mistake fails a test instead of a review.
const SWEPT_PHASES_MS: [u64; 2] = [0, 1];

const BURST_BUCKET_MS: u64 = 5_000;
const SUSTAINED_BUCKET_MS: u64 = 60_000;

#[test]
fn swept_phases_are_separated_by_a_full_bucket() {
    let burst = NonZeroU64::new(BURST_BUCKET_MS).unwrap();
    let sustained = NonZeroU64::new(SUSTAINED_BUCKET_MS).unwrap();
    let [immediate_boundary, full_bucket_away] = [1, 0];

    // Both N23 bucket sizes, both swept phases, as literal distances.
    assert_eq!(first_bucket_boundary_ms(burst, full_bucket_away), 5_000);
    assert_eq!(
        first_bucket_boundary_ms(sustained, full_bucket_away),
        60_000
    );
    assert_eq!(first_bucket_boundary_ms(burst, immediate_boundary), 1);
    assert_eq!(first_bucket_boundary_ms(sustained, immediate_boundary), 1);

    // The trap itself, pinned: a phase just under the bucket length is 1 ms
    // from phase 0, which is why 59,999 was not an adversarial second phase.
    assert_eq!(first_bucket_boundary_ms(sustained, 59_999), 59_999);

    // The real guard, stated over whatever SWEPT_PHASES_MS holds: the sweep is
    // only a sweep if its phases move the first boundary by nearly a whole
    // bucket, in *both* bucket sizes.  [0, 59_999] fails this by 59,998 ms.
    for bucket in [burst, sustained] {
        let [first, second] = SWEPT_PHASES_MS.map(|phase| first_bucket_boundary_ms(bucket, phase));
        assert!(
            first.abs_diff(second) >= bucket.get() - 1,
            "phases must differ by nearly a whole {bucket} ms bucket, got {first} vs {second}"
        );
    }
    assert_eq!(SWEPT_PHASES_MS, [full_bucket_away, immediate_boundary]);
}

#[tokio::test(start_paused = true)]
async fn m1_m13_run_against_the_actor_and_the_judge() {
    // Every row executes at both swept phases.  The mock's phase is always
    // recorded into the judge's G6 reproduction record; adding a third phase
    // is a data-only change here.
    for phase_ms in SWEPT_PHASES_MS {
        run_m1_m13(phase_ms).await;
        run_m8_oauth_lane(phase_ms).await;
    }
}

fn independently_scripted_oracle(
    scenario_id: ScenarioId,
    observations: &[rate_limit_core::mock::Observation],
) -> DriverOracle {
    let mut oracle = DriverOracle::default();
    let mut prior_dispatch = 0_u64;
    let mut prior_completion = 0_u64;
    let mut prior_was_head = false;
    let mut ordinary = 0usize;
    for observation in observations {
        let eligible = if observation.ordinal == 0 {
            // Every driver run queues its first unknown endpoint at t=0.
            0
        } else if observation.method == Method::HEAD {
            // A later writer becomes eligible only when the prior reader has
            // completed; the scenario's wire log supplies that server-owned
            // fact, while writer preference/exclusivity remain G5 assertions.
            prior_completion.max(prior_dispatch.saturating_add(250))
        } else {
            ordinary += 1;
            if scenario_id == ScenarioId::M8 && ordinary == 2 {
                // M8's scripted Retry-After=0 still demands the applicable
                // 60s bucket plus its one-second buffer (N19).
                prior_dispatch.saturating_add(61_000)
            } else {
                // The default scripts have no policy debt after the preceding
                // observation, so D5's 250ms floor is the only eligibility
                // constraint.  This sees a late actor turn as G3 failure.
                let floor_open = prior_dispatch.saturating_add(250);
                let exclusive_until = if prior_was_head { prior_completion } else { 0 };
                exclusive_until.max(floor_open)
            }
        };
        oracle.eligible.insert(observation.correlation_id, eligible);
        prior_dispatch = observation.dispatch_ms;
        prior_completion = observation.completion_ms;
        prior_was_head = observation.method == Method::HEAD;
    }
    oracle
}
