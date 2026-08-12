use http::Method;
use rate_limit_core::conformance::{
    AuthorizedExclusion, AuthorizedExclusionKind, ClientBucketProfile, DispatchSample,
    DurationEvidence, ExposureAllowance, Gate, JudgeError, OAUTH_KNOWN_PROFILE, ReproductionRecord,
    RunEvidence, SCENARIOS, SHIPPED_ASSUMED_PROFILE, ScenarioAssertion, ScenarioId,
    SweepConfiguration, SweepKind, SweepPlan, SweepPlanError, judge, scenario,
};
use rate_limit_core::mock::{Endpoint, MockConfig, MockService, request};
use rate_limit_core::transport::Transport;

#[test]
fn m1_m13_metadata_is_total_and_tags_match_the_contract() {
    assert_eq!(SCENARIOS.len(), 13);
    for (index, id) in ScenarioId::ALL.into_iter().enumerate() {
        assert_eq!(scenario(id).id, id, "reachability: M-row {index}");
        assert!(!scenario(id).name.is_empty());
    }
    for id in [
        ScenarioId::M1,
        ScenarioId::M2,
        ScenarioId::M5,
        ScenarioId::M6,
        ScenarioId::M7,
        ScenarioId::M8,
        ScenarioId::M9,
        ScenarioId::M10,
    ] {
        assert_eq!(scenario(id).sweep, SweepKind::PhaseSwept);
    }
    for id in [
        ScenarioId::M3,
        ScenarioId::M4,
        ScenarioId::M11,
        ScenarioId::M12,
        ScenarioId::M13,
    ] {
        assert_eq!(scenario(id).sweep, SweepKind::PhaseIndependent);
    }
}

#[test]
fn sweep_plan_structurally_requires_the_shipped_assumed_60s_default() {
    let known_only = vec![SweepConfiguration {
        seed: 1,
        phase_ms: 0,
        client_buckets: OAUTH_KNOWN_PROFILE,
    }];
    assert_eq!(
        SweepPlan::new(known_only),
        Err(SweepPlanError::MissingShippedAssumedDefault)
    );

    let configurations = vec![
        SweepConfiguration {
            seed: 1,
            phase_ms: 0,
            client_buckets: OAUTH_KNOWN_PROFILE,
        },
        SweepConfiguration {
            seed: 2,
            phase_ms: 59_999,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        },
    ];
    let plan = SweepPlan::new(configurations).unwrap();
    assert_eq!(plan.configurations()[1].client_buckets.burst_ms, 60_000);
    assert_eq!(plan.configurations()[1].client_buckets.sustained_ms, 60_000);
}

async fn one_observation() -> rate_limit_core::mock::Observation {
    let (service, controller) = MockService::new(MockConfig::n23(77, 321)).unwrap();
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    controller.observations().await.remove(0)
}

fn base_evidence(observation: rate_limit_core::mock::Observation) -> RunEvidence {
    RunEvidence {
        scenario: ScenarioId::M1,
        reproduction: Some(ReproductionRecord {
            seed: observation.seed,
            phase_ms: observation.phase_ms,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        }),
        observations: vec![observation],
        dispatch_samples: vec![DispatchSample {
            correlation_id: 1,
            independently_eligible_ms: 0,
            dispatched_ms: 250,
        }],
        exclusions: Vec::new(),
        unavoidable_exposure: None,
        assertions: vec![ScenarioAssertion {
            name: "M1 one HEAD before first GET".to_owned(),
            passed: true,
        }],
        duration: None,
    }
}

#[tokio::test(start_paused = true)]
async fn all_global_gates_are_armed_and_judged_from_wire_evidence() {
    let evidence = base_evidence(one_observation().await);
    let report = judge(&evidence).unwrap();
    assert!(report.passed());
    assert_eq!(report.gates.len(), 6);
    for gate in [Gate::G1, Gate::G2, Gate::G3, Gate::G4, Gate::G5, Gate::G6] {
        assert!(report.gate(gate).passed, "reachability: judged {gate:?}");
    }
    assert!(!report.gate(Gate::G4).applicable);

    let mut violation = evidence.clone();
    violation.observations[0].policy_judgment.organic_violation = true;
    violation.observations[0].layer1.tripped = true;
    violation.dispatch_samples[0].dispatched_ms = 501;
    violation.assertions[0].passed = false;
    let report = judge(&violation).unwrap();
    assert!(!report.gate(Gate::G1).passed);
    assert!(!report.gate(Gate::G2).passed);
    assert!(!report.gate(Gate::G3).passed);
    assert!(!report.gate(Gate::G5).passed);
}

#[tokio::test(start_paused = true)]
async fn g3_exclusions_require_script_owned_bounded_intervals() {
    let mut evidence = base_evidence(one_observation().await);
    evidence.dispatch_samples[0].dispatched_ms = 1_000;
    evidence.exclusions.push(AuthorizedExclusion {
        kind: AuthorizedExclusionKind::Probe,
        begins_ms: 0,
        ends_ms: 1_000,
    });
    assert!(judge(&evidence).unwrap().gate(Gate::G3).passed);

    evidence.exclusions[0].ends_ms = 999;
    assert!(!judge(&evidence).unwrap().gate(Gate::G3).passed);
}

#[tokio::test(start_paused = true)]
async fn g1_unavoidable_exposure_is_pre_observation_only_and_capped() {
    let first = one_observation().await;
    let mut second = first.clone();
    second.correlation_id = 2;
    let mut third = first.clone();
    third.correlation_id = 3;
    let mut evidence = base_evidence(first);
    evidence.observations.push(second);
    evidence.observations.push(third);
    evidence.dispatch_samples.extend([
        DispatchSample {
            correlation_id: 2,
            independently_eligible_ms: 0,
            dispatched_ms: 250,
        },
        DispatchSample {
            correlation_id: 3,
            independently_eligible_ms: 0,
            dispatched_ms: 250,
        },
    ]);
    for observation in &mut evidence.observations {
        observation.policy_judgment.organic_violation = true;
    }
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::before_observable([(1, 10), (2, 11), (3, 13)], 12, 2).unwrap());

    let report = judge(&evidence).unwrap();
    assert!(!report.gate(Gate::G1).passed);
    assert!(report.gate(Gate::G1).failures[0].contains("correlation 3"));
}

#[tokio::test(start_paused = true)]
async fn g4_uses_independent_integer_arithmetic_at_the_exact_boundary() {
    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.scenario = ScenarioId::M2;
    evidence.duration = Some(DurationEvidence {
        actual_ms: 1_050,
        theoretical_padded_minimum_ms: 1_000,
    });
    let report = judge(&evidence).unwrap();
    assert!(report.gate(Gate::G4).applicable);
    assert!(report.gate(Gate::G4).passed);
    evidence.duration.as_mut().unwrap().actual_ms = 1_051;
    assert!(!judge(&evidence).unwrap().gate(Gate::G4).passed);
}

#[tokio::test(start_paused = true)]
async fn evidence_cannot_pass_vacuously() {
    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.observations.clear();
    assert_eq!(judge(&evidence), Err(JudgeError::MissingWireObservation));

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.dispatch_samples.clear();
    assert_eq!(judge(&evidence), Err(JudgeError::MissingDispatchSample));

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.assertions.clear();
    assert_eq!(judge(&evidence), Err(JudgeError::MissingScenarioAssertion));

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.reproduction = None;
    assert_eq!(judge(&evidence), Err(JudgeError::MissingReproductionRecord));
}

#[tokio::test(start_paused = true)]
async fn correlation_and_reproduction_seams_are_structural() {
    let observation = one_observation().await;
    let mut evidence = base_evidence(observation.clone());
    evidence.observations.push(observation);
    assert_eq!(
        judge(&evidence),
        Err(JudgeError::DuplicateCorrelation {
            kind: "observation",
            id: 1,
        })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.reproduction.as_mut().unwrap().phase_ms += 1;
    assert_eq!(
        judge(&evidence),
        Err(JudgeError::ReproductionMismatch { id: 1 })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.dispatch_samples.push(DispatchSample {
        correlation_id: 2,
        independently_eligible_ms: 0,
        dispatched_ms: 250,
    });
    assert_eq!(
        judge(&evidence),
        Err(JudgeError::MissingObservationForDispatch { id: 2 })
    );

    assert!(
        rate_limit_core::conformance::ExposureAllowance::before_observable([(1, 0)], 1, 3,)
            .is_err()
    );
}

#[test]
fn profile_equality_keeps_provenance_in_the_sweep_key() {
    assert_ne!(
        SHIPPED_ASSUMED_PROFILE,
        ClientBucketProfile {
            evidence: rate_limit_core::conformance::ResolutionEvidence::Known,
            ..SHIPPED_ASSUMED_PROFILE
        }
    );
}
