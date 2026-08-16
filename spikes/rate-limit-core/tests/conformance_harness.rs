use http::Method;
use rate_limit_core::conformance::{
    AuthorizedExclusion, AuthorizedExclusionKind, ClientBucketProfile, ContractCoverage,
    ExposureAllowance, ExposureError, FullContractDeclarationError, FullContractRun, Gate,
    JudgeError, OAUTH_KNOWN_PROFILE, RunEvidence, RunReport, SCENARIOS, SHIPPED_ASSUMED_PROFILE,
    ScenarioAssertion, ScenarioAssertionId, ScenarioId, ScenarioOracle, SweepConfiguration,
    SweepKind, SweepPlan, SweepPlanError, judge, scenario,
};
use rate_limit_core::mock::{
    Endpoint, ExchangeScript, MockConfig, MockEvidence, MockService, MockStateChange, request,
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

async fn mock_evidence(
    scenario: ScenarioId,
    endpoint: Endpoint,
    seed: u64,
    phase_ms: u64,
) -> MockEvidence {
    let (service, controller) = MockService::new(MockConfig::n23(seed, phase_ms)).unwrap();
    service
        .send(request(Method::HEAD, endpoint, 1).unwrap())
        .await
        .unwrap();
    let evidence = controller.seal_evidence().await.unwrap();
    assert_eq!(evidence.observations().len(), 1, "{scenario:?} fixture");
    evidence
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

fn evidence_for(
    mock: MockEvidence,
    scenario_id: ScenarioId,
    profile: ClientBucketProfile,
    endpoint: Endpoint,
    coverage: ContractCoverage,
    passed: bool,
    exposure: Option<ExposureAllowance>,
) -> RunEvidence {
    let spec = scenario(scenario_id);
    let assertions = vec![ScenarioAssertion {
        id: spec.required_assertion,
        coverage,
        passed,
    }];
    match spec.sweep {
        SweepKind::PhaseSwept => {
            let observation = mock.observations().first().expect("wire observation");
            RunEvidence::phase_swept(
                scenario_id,
                SweepConfiguration {
                    seed: observation.seed,
                    phase_ms: observation.phase_ms,
                    client_buckets: profile,
                },
                endpoint,
                mock,
                exposure,
                assertions,
            )
        }
        SweepKind::PhaseIndependent => {
            RunEvidence::phase_independent(scenario_id, mock, exposure, assertions)
        }
    }
}

#[tokio::test(start_paused = true)]
async fn all_global_gates_are_armed_and_judged_from_wire_evidence() {
    let evidence = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 77, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        None,
    );
    let report = judge(&evidence, &TestOracle::default()).unwrap();
    assert!(report.passed());
    assert_eq!(report.gates().len(), 6);
    for gate in [Gate::G1, Gate::G2, Gate::G3, Gate::G4, Gate::G5, Gate::G6] {
        assert!(report.gate(gate).passed, "reachability: judged {gate:?}");
    }
    assert!(!report.gate(Gate::G4).applicable);

    let violation = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 78, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        false,
        None,
    );
    let report = judge(
        &violation,
        &TestOracle {
            eligible_ms: 1,
            ..TestOracle::default()
        },
    )
    .unwrap();
    assert!(!report.gate(Gate::G3).passed);
    assert!(!report.gate(Gate::G5).passed);
}

#[tokio::test(start_paused = true)]
async fn a_fragment_run_is_judged_but_is_never_verdict_eligible() {
    // Both branches assert, so this cannot pass vacuously: a fragment and a
    // full-contract run differ *only* in coverage, and the pass/verdict
    // answers must diverge.
    let full = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 77, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        None,
    );
    let report = judge(&full, &TestOracle::default()).unwrap();
    assert!(report.passed());
    assert_eq!(report.contract_coverage(), ContractCoverage::FullContract);
    assert!(report.verdict_eligible());

    let fragment = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 78, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::Fragment,
        true,
        None,
    );
    let report = judge(&fragment, &TestOracle::default()).unwrap();
    assert!(
        report.passed(),
        "a fragment is still judged on its own terms"
    );
    assert_eq!(report.contract_coverage(), ContractCoverage::Fragment);
    assert!(!report.verdict_eligible());

    // G5 still has teeth under Fragment coverage: partial is not exempt.
    let failing_fragment = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 79, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::Fragment,
        false,
        None,
    );
    let report = judge(&failing_fragment, &TestOracle::default()).unwrap();
    assert!(!report.gate(Gate::G5).passed);
    assert!(!report.verdict_eligible());
}

/// A minimal report set satisfying every declaration requirement, shaped
/// like the real driver's: one row per scenario, M8's legacy Assumed lane on
/// its endpoint, and the StashList plus SD-R8-F5 character-policy M2 lanes
/// the endpoint and pair requirements watch.
async fn report_for(
    scenario_id: ScenarioId,
    endpoint: Endpoint,
    profile: ClientBucketProfile,
    coverage: ContractCoverage,
    seed: u64,
) -> RunReport {
    let evidence = evidence_for(
        mock_evidence(scenario_id, endpoint, seed, 321).await,
        scenario_id,
        profile,
        endpoint,
        coverage,
        true,
        None,
    );
    let oracle = TestOracle {
        m2_padded_minimum_ms: (scenario_id == ScenarioId::M2).then_some(1_000),
        ..TestOracle::default()
    };
    judge(&evidence, &oracle).unwrap()
}

async fn full_contract_report_set() -> Vec<RunReport> {
    let mut reports = Vec::new();
    for (index, scenario_id) in ScenarioId::ALL.into_iter().enumerate() {
        let endpoint = if scenario_id == ScenarioId::M8 {
            Endpoint::Stash
        } else {
            Endpoint::StashList
        };
        reports.push(
            report_for(
                scenario_id,
                endpoint,
                OAUTH_KNOWN_PROFILE,
                ContractCoverage::FullContract,
                100 + index as u64,
            )
            .await,
        );
    }
    reports.push(
        report_for(
            ScenarioId::M8,
            Endpoint::LegacyStashIndex,
            SHIPPED_ASSUMED_PROFILE,
            ContractCoverage::FullContract,
            208,
        )
        .await,
    );
    for (seed, endpoint) in [(209, Endpoint::CharacterList), (210, Endpoint::Character)] {
        reports.push(
            report_for(
                ScenarioId::M2,
                endpoint,
                OAUTH_KNOWN_PROFILE,
                ContractCoverage::FullContract,
                seed,
            )
            .await,
        );
    }
    reports
}

#[tokio::test(start_paused = true)]
async fn full_contract_declaration_requires_every_m_row_and_both_m8_lanes() {
    let mut reports = full_contract_report_set().await;

    let declaration = FullContractRun::declare(reports.clone()).unwrap();
    assert_eq!(declaration.reports().len(), ScenarioId::ALL.len() + 3);

    let missing_m13 = reports
        .iter()
        .filter(|report| report.scenario() != ScenarioId::M13)
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
            !(report.scenario() == ScenarioId::M8
                && report
                    .reproduction()
                    .is_some_and(|record| record.client_buckets() == OAUTH_KNOWN_PROFILE))
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
                .reproduction()
                .is_none_or(|record| record.client_buckets() == OAUTH_KNOWN_PROFILE)
        })
        .cloned()
        .collect();
    assert_eq!(
        FullContractRun::declare(known_only),
        Err(FullContractDeclarationError::MissingM8AssumedLane)
    );

    // The SD-R8-F5 state: both M8 lanes present, but a character-policy
    // lane dropped. Before the endpoint requirement this declared cleanly —
    // the same silent-vanish shape as F4, aimed at the tightest policy.
    let character_list_missing = reports
        .iter()
        .filter(|report| {
            report
                .reproduction()
                .is_none_or(|record| record.endpoint() != Endpoint::CharacterList)
        })
        .cloned()
        .collect();
    assert_eq!(
        FullContractRun::declare(character_list_missing),
        Err(FullContractDeclarationError::MissingEndpointLane {
            endpoint: Endpoint::CharacterList,
        })
    );

    reports.retain(|report| report.scenario() != ScenarioId::M1);
    reports.push(
        report_for(
            ScenarioId::M1,
            Endpoint::StashList,
            OAUTH_KNOWN_PROFILE,
            ContractCoverage::Fragment,
            999,
        )
        .await,
    );
    assert_eq!(
        FullContractRun::declare(reports),
        Err(FullContractDeclarationError::ReportNotVerdictEligible {
            scenario: ScenarioId::M1,
        })
    );
}

// SD-R8-F11's two audit bypasses, pinned. The final audit relabeled the
// seed-809 CharacterList lane's scenario from M2 to phase-swept M5 with its
// real CharacterList wire traffic intact: endpoint coverage held, M2 still
// existed through its other lanes, and both the pinned declaration and the
// obligations verifier passed. The guard's flaw, stated set-wise, is
// endpoint-only satisfaction: every routed endpoint can be present while a
// required (scenario, endpoint) saturation lane is absent. Both states must
// now refuse by pair.
#[tokio::test(start_paused = true)]
async fn full_contract_declaration_refuses_a_relabeled_required_lane() {
    let reports = full_contract_report_set().await;

    // Bypass one: construct the audit's relabel before judging — real
    // CharacterList wire traffic and an M5 assertion, but no M2 saturation
    // report for that endpoint. Sealed reports cannot be relabeled later.
    let mut relabeled = reports
        .iter()
        .filter(|report| {
            !(report.scenario() == ScenarioId::M2
                && report
                    .reproduction()
                    .is_some_and(|record| record.endpoint() == Endpoint::CharacterList))
        })
        .cloned()
        .collect::<Vec<_>>();
    relabeled.push(
        report_for(
            ScenarioId::M5,
            Endpoint::CharacterList,
            OAUTH_KNOWN_PROFILE,
            ContractCoverage::FullContract,
            809,
        )
        .await,
    );
    // The pre-F11 guard accepted exactly this state: every routed endpoint
    // is still present in some verdict-eligible report.
    for endpoint in Endpoint::ALL {
        assert!(
            relabeled.iter().any(|report| report
                .reproduction()
                .is_some_and(|record| record.endpoint() == endpoint)),
            "the bypass must satisfy the endpoint-only requirement to be a bypass"
        );
    }
    assert_eq!(
        FullContractRun::declare(relabeled),
        Err(FullContractDeclarationError::MissingScenarioEndpointLane {
            scenario: ScenarioId::M2,
            endpoint: Endpoint::CharacterList,
        })
    );

    // Bypass two: endpoint-only satisfaction for the other character pair —
    // the Character endpoint carried by an M9 report instead of its
    // required M2 saturation lane.
    let mut swapped = reports
        .into_iter()
        .filter(|report| {
            !(report.scenario() == ScenarioId::M2
                && report
                    .reproduction()
                    .is_some_and(|record| record.endpoint() == Endpoint::Character))
        })
        .collect::<Vec<_>>();
    swapped.push(
        report_for(
            ScenarioId::M9,
            Endpoint::Character,
            OAUTH_KNOWN_PROFILE,
            ContractCoverage::FullContract,
            810,
        )
        .await,
    );
    for endpoint in Endpoint::ALL {
        assert!(
            swapped.iter().any(|report| report
                .reproduction()
                .is_some_and(|record| record.endpoint() == endpoint)),
            "the bypass must satisfy the endpoint-only requirement to be a bypass"
        );
    }
    assert_eq!(
        FullContractRun::declare(swapped),
        Err(FullContractDeclarationError::MissingScenarioEndpointLane {
            scenario: ScenarioId::M2,
            endpoint: Endpoint::Character,
        })
    );
}

// The SD-R8-F11 pair-shape audit of the M8 lanes: each profile check binds
// its endpoint, so the two M8 lanes cannot swap policies — a Known-profile
// legacy lane plus an Assumed-profile stash lane satisfies a profile-only
// check while the conditional verdict's evidence lane has silently vanished.
#[tokio::test(start_paused = true)]
async fn full_contract_declaration_binds_each_m8_profile_to_its_endpoint() {
    let mut reports = full_contract_report_set()
        .await
        .into_iter()
        .filter(|report| report.scenario() != ScenarioId::M8)
        .collect::<Vec<_>>();
    reports.push(
        report_for(
            ScenarioId::M8,
            Endpoint::LegacyStashIndex,
            OAUTH_KNOWN_PROFILE,
            ContractCoverage::FullContract,
            811,
        )
        .await,
    );
    reports.push(
        report_for(
            ScenarioId::M8,
            Endpoint::Stash,
            SHIPPED_ASSUMED_PROFILE,
            ContractCoverage::FullContract,
            812,
        )
        .await,
    );
    assert_eq!(
        FullContractRun::declare(reports),
        Err(FullContractDeclarationError::MissingM8KnownLane)
    );
}

#[tokio::test(start_paused = true)]
async fn g5_rejects_unauthorized_refusal_when_wire_safety_is_green() {
    // Model a client that sent one harmless request and then entered a
    // refusal state without any scenario-script trigger. The scenario-owned
    // assertion is the independent evidence that the expected continuation
    // never happened; G1/G2 staying green must not launder that refusal into
    // a pass (scenarios.md G5 unauthorized-refusal clause).
    let evidence = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 77, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        false,
        None,
    );
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
    let evidence = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 77, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        None,
    );
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
    let (service, controller) = MockService::new(MockConfig::n23(77, 321)).unwrap();
    tokio::time::advance(std::time::Duration::from_millis(1_000)).await;
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let evidence = evidence_for(
        controller.seal_evidence().await.unwrap(),
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        None,
    );
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
    let (service, controller) = MockService::new(MockConfig::n23(77, 321)).unwrap();
    let change = controller
        .inject_phantoms("stash-request-limit", 15)
        .await
        .unwrap();
    for correlation in 1..=3 {
        service
            .send(request(Method::GET, Endpoint::Stash, correlation).unwrap())
            .await
            .unwrap();
    }
    let evidence = evidence_for(
        controller.seal_evidence().await.unwrap(),
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        Some(ExposureAllowance::for_state_change(change.id, [(1, 0), (2, 0)], 2).unwrap()),
    );

    let oracle = TestOracle {
        observable_ms: Some(1),
        ..TestOracle::default()
    };
    let report = judge(&evidence, &oracle).unwrap();
    assert!(!report.gate(Gate::G1).passed);
    assert!(report.gate(Gate::G1).failures[0].contains("correlation 3"));
    assert_eq!(
        ExposureAllowance::for_state_change(change.id, [(1, 0), (2, 0), (3, 0)], 2),
        Err(ExposureError::TooManyPreObservableReservations { maximum: 2 })
    );
}

#[tokio::test(start_paused = true)]
async fn g4_uses_independent_integer_arithmetic_at_the_exact_boundary() {
    let oracle = TestOracle {
        m2_padded_minimum_ms: Some(1_000),
        ..TestOracle::default()
    };
    let evidence_at = |delay_ms| async move {
        let (service, controller) = MockService::new(MockConfig::n23(77, 321)).unwrap();
        controller
            .script(
                1,
                ExchangeScript {
                    response_delay: std::time::Duration::from_millis(delay_ms),
                    ..ExchangeScript::default()
                },
            )
            .await
            .unwrap();
        service
            .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
            .await
            .unwrap();
        evidence_for(
            controller.seal_evidence().await.unwrap(),
            ScenarioId::M2,
            SHIPPED_ASSUMED_PROFILE,
            Endpoint::Stash,
            ContractCoverage::FullContract,
            true,
            None,
        )
    };
    let evidence = evidence_at(1_050).await;
    let report = judge(&evidence, &oracle).unwrap();
    assert!(report.gate(Gate::G4).applicable);
    assert!(report.gate(Gate::G4).passed);
    let evidence = evidence_at(1_051).await;
    assert!(!judge(&evidence, &oracle).unwrap().gate(Gate::G4).passed);
}

#[tokio::test(start_paused = true)]
async fn evidence_cannot_pass_vacuously() {
    let (_service, controller) = MockService::new(MockConfig::n23(77, 321)).unwrap();
    let evidence = RunEvidence::phase_swept(
        ScenarioId::M1,
        SweepConfiguration {
            seed: 77,
            phase_ms: 321,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        },
        Endpoint::Stash,
        controller.seal_evidence().await.unwrap(),
        None,
        vec![ScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
            coverage: ContractCoverage::FullContract,
            passed: true,
        }],
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::MissingWireObservation)
    );

    let mock = mock_evidence(ScenarioId::M1, Endpoint::Stash, 78, 321).await;
    let evidence = RunEvidence::phase_swept(
        ScenarioId::M1,
        SweepConfiguration {
            seed: 78,
            phase_ms: 321,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        },
        Endpoint::Stash,
        mock,
        None,
        Vec::new(),
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::MissingScenarioAssertion)
    );

    let mock = mock_evidence(ScenarioId::M1, Endpoint::Stash, 79, 321).await;
    let evidence = RunEvidence::phase_independent(
        ScenarioId::M1,
        mock,
        None,
        vec![ScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
            coverage: ContractCoverage::FullContract,
            passed: true,
        }],
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::MissingReproductionRecord)
    );
}

#[tokio::test(start_paused = true)]
async fn correlation_and_reproduction_seams_are_structural() {
    let mock = mock_evidence(ScenarioId::M1, Endpoint::Stash, 77, 321).await;
    let evidence = RunEvidence::phase_swept(
        ScenarioId::M1,
        SweepConfiguration {
            seed: 77,
            phase_ms: 322,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        },
        Endpoint::Stash,
        mock,
        None,
        vec![ScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
            coverage: ContractCoverage::FullContract,
            passed: true,
        }],
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ReproductionMismatch { id: 1 })
    );

    // SD-R8-F9's exact seam: a record relabeled to an endpoint the wire
    // never carried must be refused, or the declaration's endpoint
    // coverage rests on run-owned provenance instead of mock facts.
    let mock = mock_evidence(ScenarioId::M1, Endpoint::Stash, 78, 321).await;
    let evidence = RunEvidence::phase_swept(
        ScenarioId::M1,
        SweepConfiguration {
            seed: 78,
            phase_ms: 321,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        },
        Endpoint::CharacterList,
        mock,
        None,
        vec![ScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
            coverage: ContractCoverage::FullContract,
            passed: true,
        }],
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ReproductionMismatch { id: 1 })
    );

    let mock = mock_evidence(ScenarioId::M1, Endpoint::Stash, 79, 321).await;
    let evidence = RunEvidence::phase_swept(
        ScenarioId::M1,
        SweepConfiguration {
            seed: 79,
            phase_ms: 321,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        },
        Endpoint::Stash,
        mock,
        None,
        vec![ScenarioAssertion {
            id: ScenarioAssertionId::M2Saturation,
            coverage: ContractCoverage::FullContract,
            passed: true,
        }],
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::UnexpectedScenarioAssertion {
            expected: ScenarioAssertionId::M1BootSequence,
            got: ScenarioAssertionId::M2Saturation,
        })
    );

    let mock = mock_evidence(ScenarioId::M1, Endpoint::Stash, 80, 321).await;
    let assertion = ScenarioAssertion {
        id: ScenarioAssertionId::M1BootSequence,
        coverage: ContractCoverage::FullContract,
        passed: true,
    };
    let evidence = RunEvidence::phase_swept(
        ScenarioId::M1,
        SweepConfiguration {
            seed: 80,
            phase_ms: 321,
            client_buckets: SHIPPED_ASSUMED_PROFILE,
        },
        Endpoint::Stash,
        mock,
        None,
        vec![assertion.clone(), assertion],
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::DuplicateScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
        })
    );

    let evidence = evidence_for(
        mock_evidence(ScenarioId::M1, Endpoint::Stash, 81, 321).await,
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        Some(ExposureAllowance::for_state_change(7, [(2, 0)], 1).unwrap()),
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureWithoutStateChange { id: 7 })
    );

    let (service, controller) = MockService::new(MockConfig::n23(82, 321)).unwrap();
    let change = controller
        .inject_phantoms("stash-request-limit", 1)
        .await
        .unwrap();
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let evidence = evidence_for(
        controller.seal_evidence().await.unwrap(),
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        Some(ExposureAllowance::for_state_change(change.id, [(2, 0)], 1).unwrap()),
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureWithoutObservation { id: 2 })
    );

    let (service, controller) = MockService::new(MockConfig::n23(83, 321)).unwrap();
    let change = controller
        .inject_phantoms("stash-request-limit", 1)
        .await
        .unwrap();
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let evidence = evidence_for(
        controller.seal_evidence().await.unwrap(),
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        Some(ExposureAllowance::for_state_change(change.id, [(1, 1)], 1).unwrap()),
    );
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

    let (service, controller) = MockService::new(MockConfig::n23(84, 321)).unwrap();
    let change = controller
        .inject_phantoms("stash-request-limit", 1)
        .await
        .unwrap();
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let evidence = evidence_for(
        controller.seal_evidence().await.unwrap(),
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        Some(ExposureAllowance::for_state_change(change.id, [(1, 0)], 1).unwrap()),
    );
    assert_eq!(
        judge(
            &evidence,
            &TestOracle {
                observable_ms: Some(0),
                ..TestOracle::default()
            }
        ),
        Err(JudgeError::ExposureAfterStateChangeObservable {
            id: 1,
            state_change_id: change.id,
        })
    );

    let (service, controller) = MockService::new(MockConfig::n23(85, 321)).unwrap();
    let change = controller
        .inject_phantoms("character-request-limit", 1)
        .await
        .unwrap();
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let evidence = evidence_for(
        controller.seal_evidence().await.unwrap(),
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        Some(ExposureAllowance::for_state_change(change.id, [(1, 0)], 1).unwrap()),
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureUnrelatedStateChange {
            id: 1,
            state_change_id: change.id,
        })
    );

    let (service, controller) = MockService::new(MockConfig::n23(86, 321)).unwrap();
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let change = controller
        .inject_phantoms("stash-request-limit", 1)
        .await
        .unwrap();
    let evidence = evidence_for(
        controller.seal_evidence().await.unwrap(),
        ScenarioId::M1,
        SHIPPED_ASSUMED_PROFILE,
        Endpoint::Stash,
        ContractCoverage::FullContract,
        true,
        Some(ExposureAllowance::for_state_change(change.id, [(1, 0)], 1).unwrap()),
    );
    assert_eq!(
        judge(&evidence, &TestOracle::default()),
        Err(JudgeError::ExposureStateChangeAfterArrival {
            id: 1,
            state_change_id: change.id,
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
