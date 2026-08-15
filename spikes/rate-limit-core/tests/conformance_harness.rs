use http::Method;
use rate_limit_core::conformance::{
    AuthorizedExclusion, AuthorizedExclusionKind, ClientBucketProfile, ContractCoverage,
    ExposureAllowance, ExposureError, FullContractDeclarationError, FullContractRun, Gate,
    JudgeError, OAUTH_KNOWN_PROFILE, ReproductionRecord, RunEvidence, SCENARIOS,
    SHIPPED_ASSUMED_PROFILE, ScenarioAssertion, ScenarioAssertionId, ScenarioId, ScenarioOracle,
    SweepConfiguration, SweepKind, SweepPlan, SweepPlanError, judge, scenario,
};
use rate_limit_core::mock::{
    Endpoint, MockConfig, MockService, MockStateChange, MockStateChangeKind, request,
};
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

#[derive(Default)]
struct TestOracle {
    eligible_ms: u64,
    /// Simulates an oracle with no entry for the judged observation; the
    /// judge must fail G3 closed (SD-R5-F6).
    withhold_eligibility: bool,
    observable_ms: Option<u64>,
    exclusions: Vec<AuthorizedExclusion>,
    m2_padded_minimum_ms: Option<u64>,
}

impl ScenarioOracle for TestOracle {
    fn independently_eligible_ms(&self, _: &rate_limit_core::mock::Observation) -> Option<u64> {
        (!self.withhold_eligibility).then_some(self.eligible_ms)
    }

    fn independently_observable_ms(
        &self,
        state_change: &MockStateChange,
        _: &[rate_limit_core::mock::Observation],
    ) -> Option<u64> {
        self.observable_ms.or(Some(state_change.occurred_ms))
    }

    fn authorizes_delay(&self, begins_ms: u64, ends_ms: u64) -> bool {
        self.exclusions
            .iter()
            .any(|exclusion| exclusion.begins_ms <= begins_ms && ends_ms <= exclusion.ends_ms)
    }

    fn m2_theoretical_padded_minimum_ms(
        &self,
        _: &[rate_limit_core::mock::Observation],
    ) -> Option<u64> {
        self.m2_padded_minimum_ms
    }
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
        state_changes: Vec::new(),
        unavoidable_exposure: None,
        assertions: vec![ScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
            coverage: ContractCoverage::FullContract,
            passed: true,
        }],
    }
}

fn phantom_change(id: u64, occurred_ms: u64) -> MockStateChange {
    MockStateChange {
        id,
        occurred_ms,
        kind: MockStateChangeKind::PhantomInjection {
            policy: "stash-request-limit".to_owned(),
            count: 1,
        },
    }
}

#[tokio::test(start_paused = true)]
async fn all_global_gates_are_armed_and_judged_from_wire_evidence() {
    let evidence = base_evidence(one_observation().await);
    let report = judge(&evidence, &TestOracle::default()).unwrap();
    assert!(report.passed());
    assert_eq!(report.gates.len(), 6);
    for gate in [Gate::G1, Gate::G2, Gate::G3, Gate::G4, Gate::G5, Gate::G6] {
        assert!(report.gate(gate).passed, "reachability: judged {gate:?}");
    }
    assert!(!report.gate(Gate::G4).applicable);

    let mut violation = evidence.clone();
    violation.observations[0].policy_judgment.organic_violation = true;
    violation.observations[0].layer1.tripped = true;
    violation.assertions[0].passed = false;
    let report = judge(
        &violation,
        &TestOracle {
            eligible_ms: 1,
            ..TestOracle::default()
        },
    )
    .unwrap();
    assert!(!report.gate(Gate::G1).passed);
    assert!(!report.gate(Gate::G2).passed);
    assert!(!report.gate(Gate::G3).passed);
    assert!(!report.gate(Gate::G5).passed);
}

#[tokio::test(start_paused = true)]
async fn a_fragment_run_is_judged_but_is_never_verdict_eligible() {
    // Both branches assert, so this cannot pass vacuously: a fragment and a
    // full-contract run differ *only* in coverage, and the pass/verdict
    // answers must diverge.
    let full = base_evidence(one_observation().await);
    let report = judge(&full, &TestOracle::default()).unwrap();
    assert!(report.passed());
    assert_eq!(report.contract_coverage, ContractCoverage::FullContract);
    assert!(report.verdict_eligible());

    let mut fragment = full.clone();
    fragment.assertions[0].coverage = ContractCoverage::Fragment;
    let report = judge(&fragment, &TestOracle::default()).unwrap();
    assert!(
        report.passed(),
        "a fragment is still judged on its own terms"
    );
    assert_eq!(report.contract_coverage, ContractCoverage::Fragment);
    assert!(!report.verdict_eligible());

    // G5 still has teeth under Fragment coverage: partial is not exempt.
    fragment.assertions[0].passed = false;
    let report = judge(&fragment, &TestOracle::default()).unwrap();
    assert!(!report.gate(Gate::G5).passed);
    assert!(!report.verdict_eligible());
}

#[tokio::test(start_paused = true)]
async fn full_contract_declaration_requires_every_m_row_and_both_m8_lanes() {
    let mut evidence = base_evidence(one_observation().await);
    evidence.reproduction.as_mut().unwrap().client_buckets = OAUTH_KNOWN_PROFILE;
    let template = judge(&evidence, &TestOracle::default()).unwrap();
    let mut reports = ScenarioId::ALL
        .into_iter()
        .map(|scenario| {
            let mut report = template.clone();
            report.scenario = scenario;
            report
        })
        .collect::<Vec<_>>();
    let mut assumed = template.clone();
    assumed.scenario = ScenarioId::M8;
    assumed.reproduction.as_mut().unwrap().client_buckets = SHIPPED_ASSUMED_PROFILE;
    reports.push(assumed);

    let declaration = FullContractRun::declare(reports.clone()).unwrap();
    assert_eq!(declaration.reports().len(), ScenarioId::ALL.len() + 1);

    let missing_m13 = reports
        .iter()
        .filter(|report| report.scenario != ScenarioId::M13)
        .cloned()
        .collect();
    assert_eq!(
        FullContractRun::declare(missing_m13),
        Err(FullContractDeclarationError::MissingScenario {
            scenario: ScenarioId::M13,
        })
    );

    // The SD-R8-F4 state: M8 present only in its Assumed lane while every
    // other row still supplies Known, so a whole-set profile check sees both
    // profiles and accepts. The M8-keyed guard must refuse it.
    let m8_assumed_lane_only = reports
        .iter()
        .filter(|report| {
            !(report.scenario == ScenarioId::M8
                && report
                    .reproduction
                    .is_some_and(|record| record.client_buckets == OAUTH_KNOWN_PROFILE))
        })
        .cloned()
        .collect();
    assert_eq!(
        FullContractRun::declare(m8_assumed_lane_only),
        Err(FullContractDeclarationError::MissingM8KnownLane)
    );

    let known_only = reports
        .iter()
        .filter(|report| {
            report
                .reproduction
                .is_some_and(|record| record.client_buckets == OAUTH_KNOWN_PROFILE)
        })
        .cloned()
        .collect();
    assert_eq!(
        FullContractRun::declare(known_only),
        Err(FullContractDeclarationError::MissingM8AssumedLane)
    );

    reports[0].contract_coverage = ContractCoverage::Fragment;
    assert_eq!(
        FullContractRun::declare(reports),
        Err(FullContractDeclarationError::ReportNotVerdictEligible {
            scenario: ScenarioId::M1,
        })
    );
}

#[tokio::test(start_paused = true)]
async fn g5_rejects_unauthorized_refusal_when_wire_safety_is_green() {
    // Model a client that sent one harmless request and then entered a
    // refusal state without any scenario-script trigger. The scenario-owned
    // assertion is the independent evidence that the expected continuation
    // never happened; G1/G2 staying green must not launder that refusal into
    // a pass (scenarios.md G5 unauthorized-refusal clause).
    let mut evidence = base_evidence(one_observation().await);
    evidence.assertions[0].passed = false;
    let report = judge(&evidence, &TestOracle::default()).unwrap();

    assert!(report.gate(Gate::G1).passed);
    assert!(report.gate(Gate::G2).passed);
    assert!(!report.gate(Gate::G5).passed);
    assert_eq!(
        report.gate(Gate::G5).failures,
        vec!["M1BootSequence".to_owned()]
    );
    assert!(!report.passed());
    assert!(!report.verdict_eligible());
}

// SD-R5-F6's exposing test: before the judge owned the fail-closed branch,
// an oracle with no entry for a judged observation depended on each
// implementation's u64::MAX sentinel, and an `unwrap_or_default()` slip would
// have made every observation trivially eligible with nothing failing.
#[tokio::test(start_paused = true)]
async fn g3_fails_closed_when_the_oracle_has_no_eligibility_entry() {
    let evidence = base_evidence(one_observation().await);
    let oracle = TestOracle {
        withhold_eligibility: true,
        ..TestOracle::default()
    };
    let report = judge(&evidence, &oracle).unwrap();
    assert!(!report.gate(Gate::G3).passed);
    assert!(
        report.gate(Gate::G3).failures[0].contains("no independent eligibility entry"),
        "unexpected G3 failure text: {:?}",
        report.gate(Gate::G3).failures
    );
    assert!(!report.passed());
    assert!(!report.verdict_eligible());
}

#[tokio::test(start_paused = true)]
async fn g3_exclusions_require_script_owned_bounded_intervals() {
    let mut evidence = base_evidence(one_observation().await);
    evidence.observations[0].dispatch_ms = 1_000;
    let oracle = TestOracle {
        exclusions: vec![AuthorizedExclusion {
            kind: AuthorizedExclusionKind::Probe,
            begins_ms: 0,
            ends_ms: 1_000,
        }],
        ..TestOracle::default()
    };
    assert!(judge(&evidence, &oracle).unwrap().gate(Gate::G3).passed);

    let short_oracle = TestOracle {
        exclusions: vec![AuthorizedExclusion {
            ends_ms: 999,
            ..oracle.exclusions[0]
        }],
        ..TestOracle::default()
    };
    assert!(
        !judge(&evidence, &short_oracle)
            .unwrap()
            .gate(Gate::G3)
            .passed
    );
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
    for observation in &mut evidence.observations {
        observation.policy_judgment.organic_violation = true;
        observation.dispatch_ms = 12;
        observation.arrival_ms = 12;
    }
    evidence.state_changes = vec![phantom_change(7, 9)];
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::for_state_change(7, [(1, 10), (2, 11)], 2).unwrap());

    let oracle = TestOracle {
        observable_ms: Some(12),
        ..TestOracle::default()
    };
    let report = judge(&evidence, &oracle).unwrap();
    assert!(!report.gate(Gate::G1).passed);
    assert!(report.gate(Gate::G1).failures[0].contains("correlation 3"));
    assert_eq!(
        ExposureAllowance::for_state_change(7, [(1, 10), (2, 11), (3, 11)], 2),
        Err(ExposureError::TooManyPreObservableReservations { maximum: 2 })
    );
}

#[tokio::test(start_paused = true)]
async fn g4_uses_independent_integer_arithmetic_at_the_exact_boundary() {
    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.scenario = ScenarioId::M2;
    evidence.observations[0].completion_ms = 1_050;
    let oracle = TestOracle {
        m2_padded_minimum_ms: Some(1_000),
        ..TestOracle::default()
    };
    evidence.assertions[0].id = ScenarioAssertionId::M2Saturation;
    let report = judge(&evidence, &oracle).unwrap();
    assert!(report.gate(Gate::G4).applicable);
    assert!(report.gate(Gate::G4).passed);
    evidence.observations[0].completion_ms = 1_051;
    assert!(!judge(&evidence, &oracle).unwrap().gate(Gate::G4).passed);
}

#[tokio::test(start_paused = true)]
async fn evidence_cannot_pass_vacuously() {
    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.observations.clear();
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::MissingWireObservation)
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.assertions.clear();
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::MissingScenarioAssertion)
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.reproduction = None;
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::MissingReproductionRecord)
    );
}

#[tokio::test(start_paused = true)]
async fn correlation_and_reproduction_seams_are_structural() {
    let observation = one_observation().await;
    let mut evidence = base_evidence(observation.clone());
    evidence.observations.push(observation);
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::DuplicateCorrelation {
            kind: "observation",
            id: 1,
        })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.reproduction.as_mut().unwrap().phase_ms += 1;
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ReproductionMismatch { id: 1 })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.assertions[0].id = ScenarioAssertionId::M2Saturation;
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::UnexpectedScenarioAssertion {
            expected: ScenarioAssertionId::M1BootSequence,
            got: ScenarioAssertionId::M2Saturation,
        })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.assertions.push(evidence.assertions[0].clone());
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::DuplicateScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
        })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::for_state_change(7, [(2, 0)], 1).unwrap());
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureWithoutStateChange { id: 7 })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.state_changes = vec![phantom_change(7, 0)];
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::for_state_change(7, [(2, 0)], 1).unwrap());
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureWithoutObservation { id: 2 })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.state_changes = vec![phantom_change(7, 0)];
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::for_state_change(7, [(1, 1)], 1).unwrap());
    assert_eq!(
        judge(
            &evidence,
            &TestOracle {
                observable_ms: Some(2),
                ..TestOracle::default()
            }
        ),
        Err(JudgeError::ExposureAfterTransportHandoff { id: 1 })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.state_changes = vec![phantom_change(7, 1)];
    evidence.observations[0].dispatch_ms = 20;
    evidence.observations[0].arrival_ms = 20;
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::for_state_change(7, [(1, 12)], 1).unwrap());
    assert_eq!(
        judge(
            &evidence,
            &TestOracle {
                observable_ms: Some(12),
                ..TestOracle::default()
            }
        ),
        Err(JudgeError::ExposureAfterStateChangeObservable {
            id: 1,
            state_change_id: 7,
        })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.observations[0].dispatch_ms = 20;
    evidence.observations[0].arrival_ms = 20;
    evidence.state_changes = vec![MockStateChange {
        id: 7,
        occurred_ms: 1,
        kind: MockStateChangeKind::PhantomInjection {
            policy: "character-request-limit".to_owned(),
            count: 1,
        },
    }];
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::for_state_change(7, [(1, 1)], 1).unwrap());
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureUnrelatedStateChange {
            id: 1,
            state_change_id: 7,
        })
    );

    let observation = one_observation().await;
    let mut evidence = base_evidence(observation);
    evidence.observations[0].dispatch_ms = 20;
    evidence.observations[0].arrival_ms = 20;
    evidence.state_changes = vec![phantom_change(7, 21)];
    evidence.unavoidable_exposure =
        Some(ExposureAllowance::for_state_change(7, [(1, 1)], 1).unwrap());
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureStateChangeAfterArrival {
            id: 1,
            state_change_id: 7,
        })
    );

    assert!(ExposureAllowance::for_state_change(7, [(1, 0)], 3).is_err());
}

// Drift tripwire, not an oracle: the two D5_IN_FLIGHT_CAP constants are
// deliberately independent restatements (judge vs. code under test), and the
// judge's copy also caps the G1 exposure allowance. If either moves alone,
// this fails and forces the other side to be re-derived deliberately
// (SD-R5-F14).
#[test]
fn d5_in_flight_cap_restatements_agree() {
    assert_eq!(
        rate_limit_core::conformance::D5_IN_FLIGHT_CAP,
        rate_limit_core::actor::D5_IN_FLIGHT_CAP,
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
