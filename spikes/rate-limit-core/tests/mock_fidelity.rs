use std::time::{Duration, UNIX_EPOCH};

use http::header::{DATE, SERVER};
use http::{Method, StatusCode};
use proptest::prelude::*;
use rate_limit_core::header::parse_policy;
use rate_limit_core::mock::model::{
    CounterModel, DefinitionError, LAYER1_BURST_LIMIT, LAYER1_SUSTAINED_LIMIT,
    MAX_EVENTS_PER_POLICY, MAX_REQUESTS_PER_RUN, ModelError, PolicyDefinition, RuleDefinition,
    ServerTime, WindowDefinition,
};
use rate_limit_core::mock::{
    Endpoint, ExchangeScript, MAX_MOCK_HEADER_VALUE_BYTES, MAX_RAW_RESPONSE_BODY_BYTES,
    MAX_RAW_RESPONSE_HEADERS, MockConfig, MockConfigError, MockEvidenceSealError,
    MockStateChangeError, MockStateChangeKind, ResponseOverride, StimulusKind, request,
};
use rate_limit_core::transport::{Transport, TransportError};

fn tiny_policy(
    name: &str,
    max_hits: u32,
    period_ms: u64,
    restriction_ms: u64,
    bucket_ms: u64,
) -> PolicyDefinition {
    PolicyDefinition::new(
        name,
        vec![
            RuleDefinition::new(
                "Account",
                vec![
                    WindowDefinition::new(max_hits, period_ms, restriction_ms, bucket_ms).unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

#[tokio::test(start_paused = true)]
async fn b1_b4_b5_b7_b13_b14_full_protocol_and_n23_topology() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(0x5eed, 731))
            .expect("N23 fixture is valid");
    let mut dates = Vec::new();

    for (index, endpoint) in Endpoint::ALL.into_iter().enumerate() {
        let response = service
            .send(request(Method::HEAD, endpoint, index as u64).unwrap())
            .await
            .unwrap();
        let expected = if endpoint == Endpoint::LegacyStashIndex {
            StatusCode::OK
        } else {
            StatusCode::NO_CONTENT
        };
        assert_eq!(response.status(), expected, "B5 {endpoint:?}");
        let parsed = parse_policy(response.headers()).expect("B1 headers use wire grammar");
        assert!(parsed.rules.iter().all(|rule| {
            rule.state.burst.current_hits == 0 && rule.state.sustained.current_hits == 0
        }));
        let date = httpdate::parse_http_date(response.headers()[DATE].to_str().unwrap()).unwrap();
        dates.push(date);
    }

    let response = service
        .send(request(Method::GET, Endpoint::Stash, 10).unwrap())
        .await
        .unwrap();
    let parsed = parse_policy(response.headers()).unwrap();
    assert_eq!(parsed.name, "stash-request-limit");
    assert_eq!(parsed.rules[0].state.burst.current_hits, 1, "B4");
    assert_eq!(parsed.rules[0].state.sustained.current_hits, 1, "B4");

    let observations = controller.observations().await;
    assert_eq!(observations.len(), 6);
    assert_eq!(observations[0].seed, 0x5eed);
    assert_eq!(observations[0].phase_ms, 731);
    assert_eq!(observations[5].correlation_id, 10);
    assert!(
        observations[..5]
            .iter()
            .all(|observation| !observation.policy_judgment.counted)
    );
    assert!(observations[5].policy_judgment.counted);
    for (date, observation) in dates.into_iter().zip(&observations) {
        assert_eq!(
            date,
            UNIX_EPOCH + Duration::from_secs(observation.arrival_ms / 1_000),
            "B14 Date is the zero-skew mock clock at second precision"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn b14_date_uses_the_scripted_response_completion_instant() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(0xdade, 0)).unwrap();
    controller
        .script(
            1,
            ExchangeScript {
                response_delay: Duration::from_secs(2),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();

    let response = service
        .send(request(Method::GET, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let date = httpdate::parse_http_date(response.headers()[DATE].to_str().unwrap()).unwrap();
    let observation = controller.observations().await.remove(0);
    assert_eq!(observation.arrival_ms, 0);
    assert_eq!(observation.completion_ms, 2_000);
    assert_eq!(date, UNIX_EPOCH + Duration::from_secs(2));
}

#[tokio::test(start_paused = true)]
async fn sealed_evidence_is_complete_final_and_mock_authentic() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(0x5ea1, 19)).unwrap();
    controller
        .script(
            1,
            ExchangeScript {
                arrival_delay: Duration::from_secs(1),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();

    let pending = tokio::spawn({
        let service = service.clone();
        async move {
            service
                .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
                .await
        }
    });
    tokio::task::yield_now().await;
    assert_eq!(
        controller.seal_evidence().await,
        Err(MockEvidenceSealError::PendingObservations {
            handoffs: 1,
            observations: 0,
        })
    );

    tokio::time::advance(Duration::from_secs(1)).await;
    pending.await.unwrap().unwrap();
    let evidence = controller.seal_evidence().await.unwrap();
    assert_eq!(evidence.observations().len(), 1);
    assert!(evidence.state_changes().is_empty());
    assert_eq!(controller.seal_evidence().await.unwrap(), evidence);

    assert!(matches!(
        service
            .send(request(Method::HEAD, Endpoint::Stash, 2).unwrap())
            .await,
        Err(TransportError::MockHarness(message))
            if message == "mock evidence has already been sealed"
    ));
    assert_eq!(
        controller.inject_phantoms("stash-request-limit", 1).await,
        Err(MockStateChangeError::EvidenceSealed)
    );
}

/// SD-R8-F23's lexical belt, in the X2/F12 pattern: Rust privacy cannot bar
/// `MockEvidence` construction from the `mock` module's own in-crate
/// descendants — their `#[cfg(test)]` modules live inside the seal — so that
/// boundary is a named residual trust surface (the type's doc comment and
/// the `b13-observation-log` registry note carry it). This test is
/// detection, not a binding: it fails on any `MockEvidence` literal-shaped
/// text in `src/` beyond the four known sites in `src/mock/mod.rs`
/// (declaration, impl header, the compile-fail doctest forgery, and
/// `seal_evidence`'s own construction), so an in-crate forgery must
/// re-derive this pin deliberately. concat! keeps the needle out of this
/// test's own literals (the SD-R5-F5 vacuity lesson); this is a lexical
/// spike pin, not a Rust parser.
#[test]
fn f23_mock_evidence_literal_sites_in_src_are_pinned() {
    fn rust_sources(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rust_sources(&path, files);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_sources(&src, &mut files);
    assert!(
        files.len() >= 9,
        "the src/ walk must see the whole library, saw {}",
        files.len()
    );

    let needle = concat!("MockEvidence", " {");
    for path in files {
        let source = std::fs::read_to_string(&path).unwrap();
        let expected = if path.ends_with("mock/mod.rs") { 4 } else { 0 };
        assert_eq!(
            source.matches(needle).count(),
            expected,
            "unpinned MockEvidence literal-shaped text in {}",
            path.display()
        );
    }
}

proptest! {
    // B3: the test oracle is independent arithmetic in this test module. It
    // never calls mock::model::bucket_end or any production-core helper, and
    // every generated case asserts the assignment and both expiry sides.
    #[test]
    fn b3_generated_phase_sweep_has_non_vacuous_independent_oracle(
        at_ms in 0_u64..600_000,
        bucket_ms in 1_u64..60_001,
        phase_ms in 0_u64..60_000,
        period_seconds in 1_u64..601,
    ) {
        let period_ms = period_seconds * 1_000;
        let normalized_phase = phase_ms % bucket_ms;
        let expected_end = if at_ms < normalized_phase {
            normalized_phase
        } else {
            let quotient = (at_ms - normalized_phase) / bucket_ms;
            normalized_phase + (quotient + 1) * bucket_ms
        };
        let mut model = CounterModel::new(phase_ms);
        model.insert_policy(tiny_policy(
            "phase-property",
            10,
            period_ms,
            1_000,
            bucket_ms,
        )).unwrap();
        let assigned = model
            .judge("phase-property", ServerTime::from_millis(at_ms), 1, true)
            .unwrap();
        prop_assert_eq!(assigned.windows[0].bucket_end_ms, expected_end);

        let just_before = model
            .judge(
                "phase-property",
                ServerTime::from_millis(expected_end + period_ms - 1),
                2,
                false,
            )
            .unwrap();
        prop_assert_eq!(just_before.windows[0].current_hits, 1);
        let exact = model
            .judge(
                "phase-property",
                ServerTime::from_millis(expected_end + period_ms),
                3,
                false,
            )
            .unwrap();
        prop_assert_eq!(exact.windows[0].current_hits, 0);
    }
}

#[test]
fn b2_b3_quantized_expiry_and_restriction_are_independent() {
    let mut model = CounterModel::new(250);
    model
        .insert_policy(tiny_policy("p", 1, 1_000, 60_000, 1_000))
        .unwrap();

    let first = model
        .judge("p", ServerTime::from_millis(250), 1, true)
        .unwrap();
    assert!(!first.organic_violation);
    assert_eq!(first.windows[0].bucket_end_ms, 1_250);

    let second = model
        .judge("p", ServerTime::from_millis(251), 2, true)
        .unwrap();
    assert!(second.organic_violation, "B2 organic 429 on limit+1");
    assert_eq!(second.retry_after_seconds, Some(61));

    // Both hits have expired from the 1 s policy window at 2.250 s, but the
    // separately quantized restriction remains active. This early arrival is
    // refused without renewing the deadline merely because it was early.
    let early = model
        .judge("p", ServerTime::from_millis(2_250), 3, true)
        .unwrap();
    assert!(early.organic_violation);
    assert_eq!(early.windows[0].current_hits, 1);
    assert_eq!(early.retry_after_seconds, Some(59));

    let recovered = model
        .judge("p", ServerTime::from_millis(61_250), 4, false)
        .unwrap();
    assert!(!recovered.organic_violation);
}

#[tokio::test(start_paused = true)]
async fn b6_b9_residue_and_phantoms_are_mock_owned_counter_facts() {
    let (service, controller) = rate_limit_core::mock::MockService::new(MockConfig::n23(7, 0))
        .expect("N23 fixture is valid");
    controller
        .preload("stash-request-limit", ServerTime::from_millis(0), 14)
        .await
        .unwrap();

    let head = service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    assert_eq!(
        parse_policy(head.headers()).unwrap().rules[0]
            .state
            .burst
            .current_hits,
        14,
        "B6 residue appears without HEAD increment"
    );
    let last_slot = service
        .send(request(Method::GET, Endpoint::Stash, 2).unwrap())
        .await
        .unwrap();
    assert_eq!(last_slot.status(), StatusCode::OK);

    let phantom_change = controller
        .inject_phantoms("stash-request-limit", 1)
        .await
        .unwrap();
    assert_eq!(phantom_change.id, 0);
    assert!(matches!(
        phantom_change.kind,
        MockStateChangeKind::PhantomInjection { ref policy, count: 1 }
            if policy == "stash-request-limit"
    ));
    let raced = service
        .send(request(Method::GET, Endpoint::Stash, 3).unwrap())
        .await
        .unwrap();
    assert_eq!(raced.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(
        controller.observations().await[2]
            .policy_judgment
            .organic_violation
    );
    let observations = controller.observations().await;
    let burst = &observations[2].policy_judgment.windows[0];
    assert_eq!(burst.client_hits, 2);
    assert_eq!(burst.phantom_hits, 1);
    assert_eq!(burst.residue_hits, 14);
}

#[tokio::test(start_paused = true)]
async fn b8_policy_rename_and_shrink_keep_existing_hits() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(8, 0)).unwrap();

    controller
        .preload("stash-request-limit", ServerTime::from_millis(0), 14)
        .await
        .unwrap();
    let last_slot = service
        .send(request(Method::GET, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    assert_eq!(last_slot.status(), StatusCode::OK);
    let violation = service
        .send(request(Method::GET, Endpoint::Stash, 2).unwrap())
        .await
        .unwrap();
    assert_eq!(violation.status(), StatusCode::TOO_MANY_REQUESTS);

    let renamed = PolicyDefinition::new(
        "renamed-policy",
        vec![
            RuleDefinition::new(
                "Account",
                vec![
                    WindowDefinition::new(15, 10_000, 60_000, 5_000).unwrap(),
                    WindowDefinition::new(30, 300_000, 300_000, 60_000).unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let rename_change = controller
        .rename_policy("stash-request-limit", renamed)
        .await
        .unwrap();
    assert!(matches!(
        rename_change.kind,
        MockStateChangeKind::PolicyRename { ref old_policy, ref new_policy }
            if old_policy == "stash-request-limit" && new_policy == "renamed-policy"
    ));
    let renamed_reply = service
        .send(request(Method::HEAD, Endpoint::Stash, 3).unwrap())
        .await
        .unwrap();
    assert_eq!(renamed_reply.status(), StatusCode::TOO_MANY_REQUESTS);
    let renamed_state = parse_policy(renamed_reply.headers()).unwrap();
    assert_eq!(renamed_state.name, "renamed-policy");
    assert_eq!(renamed_state.rules[0].state.burst.current_hits, 16);

    controller
        .preload("renamed-policy", ServerTime::from_millis(0), 10)
        .await
        .unwrap();
    let shrunk = PolicyDefinition::new(
        "renamed-policy",
        vec![
            RuleDefinition::new(
                "Account",
                vec![
                    WindowDefinition::new(10, 10_000, 60_000, 5_000).unwrap(),
                    WindowDefinition::new(30, 300_000, 300_000, 60_000).unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap();
    let shrink_change = controller.replace_policy(shrunk).await.unwrap();
    assert!(matches!(
        shrink_change.kind,
        MockStateChangeKind::PolicyReplacement { ref policy } if policy == "renamed-policy"
    ));
    assert_eq!(
        controller
            .state_changes()
            .await
            .into_iter()
            .map(|state_change| state_change.id)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    let response = service
        .send(request(Method::GET, Endpoint::Stash, 4).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test(start_paused = true)]
async fn b10_b11_layer1_and_injected_stimuli_are_distinct() {
    let (service, controller) = rate_limit_core::mock::MockService::new(MockConfig::n23(10, 0))
        .expect("N23 fixture is valid");
    for correlation_id in 0..=LAYER1_BURST_LIMIT as u64 {
        controller
            .script(
                correlation_id,
                ExchangeScript {
                    response_delay: Duration::ZERO,
                    ..ExchangeScript::default()
                },
            )
            .await
            .unwrap();
        let response = service
            .send(request(Method::HEAD, Endpoint::Stash, correlation_id).unwrap())
            .await
            .unwrap();
        if correlation_id < LAYER1_BURST_LIMIT as u64 {
            assert_ne!(response.status(), StatusCode::FORBIDDEN);
        } else {
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(response.headers()[SERVER], "cloudflare");
            assert!(!response.headers().contains_key("x-rate-limit-policy"));
        }
    }
    assert!(
        controller
            .observations()
            .await
            .last()
            .unwrap()
            .layer1
            .tripped
    );

    let mut layer = CounterModel::new(0);
    for index in 0..LAYER1_SUSTAINED_LIMIT {
        let judgment = layer.record_layer1_arrival(ServerTime::from_millis(index as u64 * 50));
        assert!(!judgment.tripped, "reachability: checked arrival {index}");
    }
    let sustained =
        layer.record_layer1_arrival(ServerTime::from_millis(LAYER1_SUSTAINED_LIMIT as u64 * 50));
    assert!(sustained.tripped);

    let (stimulus_service, stimulus_controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(11, 0)).unwrap();
    stimulus_controller
        .script(
            99,
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
    let injected = stimulus_service
        .send(request(Method::GET, Endpoint::Character, 99).unwrap())
        .await
        .unwrap();
    assert_eq!(injected.status(), StatusCode::UNAUTHORIZED);
    let observation = &stimulus_controller.observations().await[0];
    assert_eq!(
        observation.stimulus,
        Some(StimulusKind::Full(StatusCode::UNAUTHORIZED))
    );
    assert!(!observation.policy_judgment.organic_violation);
}

#[tokio::test(start_paused = true)]
async fn b12_b13_explicit_delay_makes_overlap_and_correlation_observable() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(12, 444)).unwrap();
    for correlation_id in [1, 2] {
        controller
            .script(
                correlation_id,
                ExchangeScript {
                    response_delay: Duration::from_secs(1),
                    ..ExchangeScript::default()
                },
            )
            .await
            .unwrap();
    }

    let first = service.send(request(Method::GET, Endpoint::Stash, 1).unwrap());
    let second = service.send(request(Method::HEAD, Endpoint::Character, 2).unwrap());
    let (first, second) = tokio::join!(first, second);
    first.unwrap();
    second.unwrap();

    let observations = controller.observations().await;
    assert_eq!(observations.len(), 2);
    assert!(
        observations
            .iter()
            .any(|observation| observation.in_flight_at_arrival == 2)
    );
    assert!(
        observations
            .iter()
            .any(|observation| observation.head_overlap)
    );
    assert!(
        observations
            .iter()
            .all(|observation| observation.completion_ms - observation.arrival_ms == 1_000)
    );
    assert_eq!(
        observations
            .iter()
            .map(|observation| observation.correlation_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let duplicate = service
        .send(request(Method::GET, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap_err();
    assert!(matches!(duplicate, TransportError::MockHarness(_)));
}

#[tokio::test(start_paused = true)]
async fn b13_records_mock_owned_transport_handoff_before_arrival_delay() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(0xb13, 0)).unwrap();
    controller
        .script(
            1,
            ExchangeScript {
                arrival_delay: Duration::from_secs(1),
                response_delay: Duration::ZERO,
                response: None,
            },
        )
        .await
        .unwrap();

    service
        .send(request(Method::GET, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    let observation = controller.observations().await.remove(0);
    assert_eq!(observation.dispatch_ms, 0);
    assert_eq!(observation.arrival_ms, 1_000);
}

#[tokio::test(start_paused = true)]
async fn b13_handoff_survives_arrival_delay_cancellation_and_reserves_identity() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(0xb13, 0)).unwrap();
    controller
        .script(
            1,
            ExchangeScript {
                arrival_delay: Duration::from_secs(1),
                response_delay: Duration::ZERO,
                response: None,
            },
        )
        .await
        .unwrap();

    let delayed_service = service.clone();
    let task = tokio::spawn(async move {
        delayed_service
            .send(request(Method::GET, Endpoint::Stash, 1).unwrap())
            .await
    });
    tokio::task::yield_now().await;
    task.abort();

    let handoffs = controller.handoffs().await;
    assert_eq!(handoffs.len(), 1);
    assert_eq!(handoffs[0].correlation_id, 1);
    assert_eq!(handoffs[0].dispatch_ms, 0);
    assert!(controller.observations().await.is_empty());

    let duplicate = service
        .send(request(Method::GET, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap_err();
    assert!(matches!(duplicate, TransportError::MockHarness(_)));
}

#[tokio::test(start_paused = true)]
async fn mock_state_change_timeline_has_an_exact_run_bound() {
    let (_, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(0x5ca1e, 0)).unwrap();
    for _ in 0..MAX_REQUESTS_PER_RUN {
        controller
            .inject_phantoms("stash-request-limit", 1)
            .await
            .unwrap();
    }
    assert_eq!(
        controller.inject_phantoms("stash-request-limit", 1).await,
        Err(MockStateChangeError::BudgetExceeded {
            limit: MAX_REQUESTS_PER_RUN,
        })
    );
}

#[tokio::test(start_paused = true)]
async fn dropped_mock_transport_future_is_logged_and_ages_out_of_in_flight_state() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(17, 0)).unwrap();
    controller
        .script(
            1,
            ExchangeScript {
                response_delay: Duration::from_secs(1),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let first_service = service.clone();
    let task = tokio::spawn(async move {
        first_service
            .send(request(Method::GET, Endpoint::Stash, 1).unwrap())
            .await
    });
    tokio::task::yield_now().await;
    task.abort();
    tokio::time::advance(Duration::from_secs(1)).await;

    service
        .send(request(Method::HEAD, Endpoint::Character, 2).unwrap())
        .await
        .unwrap();
    let observations = controller.observations().await;
    assert_eq!(observations.len(), 2, "B13 retains the received arrival");
    assert_eq!(observations[1].in_flight_at_arrival, 1);
    assert!(!observations[1].head_overlap);
}

#[tokio::test(start_paused = true)]
async fn b11_m3_m4_script_channel_covers_every_response_shape() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(14, 0)).unwrap();
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
    let degraded = service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    assert!(degraded.headers().contains_key("x-rate-limit-policy"));
    assert!(!degraded.headers().contains_key("x-rate-limit-rules"));
    assert!(degraded.headers().contains_key(DATE));

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
    let cloudflare = service
        .send(request(Method::GET, Endpoint::Character, 2).unwrap())
        .await
        .unwrap();
    assert_eq!(cloudflare.status(), StatusCode::FORBIDDEN);
    assert_eq!(cloudflare.headers()["cf-mitigated"], "challenge");
    assert!(cloudflare.headers().contains_key(DATE));

    let mut malformed_headers = http::HeaderMap::new();
    malformed_headers.insert("x-rate-limit-policy", "bad".parse().unwrap());
    controller
        .script(
            3,
            ExchangeScript {
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::TOO_MANY_REQUESTS,
                    headers: malformed_headers,
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let malformed = service
        .send(request(Method::GET, Endpoint::Character, 3).unwrap())
        .await
        .unwrap();
    assert_eq!(malformed.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(parse_policy(malformed.headers()).is_err());
    assert!(malformed.headers().contains_key(DATE));

    controller
        .script(
            4,
            ExchangeScript {
                response: Some(ResponseOverride::TransportError("injected".to_owned())),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    assert!(
        service
            .send(request(Method::GET, Endpoint::Character, 4).unwrap())
            .await
            .is_err()
    );

    let observations = controller.observations().await;
    assert_eq!(observations[0].stimulus, Some(StimulusKind::PolicyOnly));
    assert_eq!(observations[1].stimulus, Some(StimulusKind::Cloudflare));
    assert_eq!(
        observations[2].stimulus,
        Some(StimulusKind::Raw(StatusCode::TOO_MANY_REQUESTS))
    );
    assert_eq!(observations[3].stimulus, Some(StimulusKind::TransportError));
}

#[tokio::test(start_paused = true)]
async fn b1_b7_m4_synthetic_one_and_three_window_policies_cross_as_raw_headers() {
    for window_count in [1, 3] {
        let mut config = MockConfig::n23(20 + window_count as u64, 0);
        config
            .policies
            .retain(|policy| policy.name() != "stash-request-limit");
        let windows = (0..window_count)
            .map(|index| {
                WindowDefinition::new(
                    10 + index as u32,
                    (index as u64 + 1) * 10_000,
                    60_000,
                    5_000,
                )
                .unwrap()
            })
            .collect();
        config.policies.push(
            PolicyDefinition::new(
                "stash-request-limit",
                vec![RuleDefinition::new("Account", windows).unwrap()],
            )
            .unwrap(),
        );
        let (service, _) = rate_limit_core::mock::MockService::new(config).unwrap();
        let response = service
            .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
            .await
            .unwrap();
        let error = parse_policy(response.headers()).unwrap_err();
        assert!(matches!(
            error,
            rate_limit_core::header::PolicyParseError::UnexpectedPolicyShape {
                triplet_count,
                ..
            } if triplet_count == window_count
        ));
    }
}

#[tokio::test(start_paused = true)]
async fn wire_driven_run_budget_latches_without_growing_the_log() {
    let mut config = MockConfig::n23(13, 0);
    config.dispatch_budget = 1;
    let (service, controller) = rate_limit_core::mock::MockService::new(config).unwrap();
    service
        .send(request(Method::HEAD, Endpoint::Stash, 1).unwrap())
        .await
        .unwrap();
    for correlation_id in [2, 3] {
        let error = service
            .send(request(Method::HEAD, Endpoint::Stash, correlation_id).unwrap())
            .await
            .unwrap_err();
        assert!(matches!(error, TransportError::MockHarness(_)));
    }
    assert_eq!(controller.observations().await.len(), 1);
}

#[test]
fn typed_mock_definitions_pin_every_n_and_n_plus_one_boundary() {
    let maximum = WindowDefinition::new(10_000, 21_600_000, 21_600_000, 21_600_000).unwrap();
    assert_eq!(
        WindowDefinition::new(10_001, 21_600_000, 21_600_000, 21_600_000),
        Err(DefinitionError::InvalidWindow)
    );
    assert_eq!(
        WindowDefinition::new(10_000, 21_601_000, 21_600_000, 21_600_000),
        Err(DefinitionError::InvalidWindow)
    );

    let eight_windows = vec![maximum.clone(); 8];
    assert!(RuleDefinition::new("r", eight_windows.clone()).is_ok());
    let nine_windows = vec![maximum.clone(); 9];
    assert_eq!(
        RuleDefinition::new("r", nine_windows),
        Err(DefinitionError::InvalidWindowCount)
    );
    assert!(RuleDefinition::new("r".repeat(64), vec![maximum.clone()]).is_ok());
    assert_eq!(
        RuleDefinition::new("r".repeat(65), vec![maximum.clone()]),
        Err(DefinitionError::InvalidRuleName)
    );

    let rules = (0..8)
        .map(|index| RuleDefinition::new(format!("r{index}"), vec![maximum.clone()]).unwrap())
        .collect::<Vec<_>>();
    assert!(PolicyDefinition::new("p".repeat(256), rules.clone()).is_ok());
    assert_eq!(
        PolicyDefinition::new("p".repeat(257), rules.clone()),
        Err(DefinitionError::InvalidPolicyName)
    );
    let mut nine_rules = rules;
    nine_rules.push(RuleDefinition::new("r8", vec![maximum]).unwrap());
    assert_eq!(
        PolicyDefinition::new("p", nine_rules),
        Err(DefinitionError::InvalidRuleCount)
    );

    let mut config = MockConfig::n23(15, 0);
    config.dispatch_budget = MAX_REQUESTS_PER_RUN;
    assert!(config.validate().is_ok());
    config.dispatch_budget += 1;
    assert_eq!(
        config.validate(),
        Err(MockConfigError::InvalidDispatchBudget {
            requested: MAX_REQUESTS_PER_RUN + 1,
            maximum: MAX_REQUESTS_PER_RUN,
        })
    );
}

#[test]
fn per_policy_event_budget_is_exact_and_refuses_n_plus_one() {
    let mut model = CounterModel::new(0);
    model
        .insert_policy(tiny_policy("bounded", 10_000, 1_000, 1_000, 1_000))
        .unwrap();
    model
        .preload("bounded", ServerTime::from_millis(0), MAX_EVENTS_PER_POLICY)
        .unwrap();
    assert_eq!(
        model.inject_phantoms("bounded", ServerTime::from_millis(0), 1),
        Err(ModelError::EventBudgetExceeded {
            policy: "bounded".to_owned(),
            limit: MAX_EVENTS_PER_POLICY,
        })
    );
}

#[tokio::test(start_paused = true)]
async fn raw_script_and_correlation_wire_bounds_are_exact() {
    let (service, controller) =
        rate_limit_core::mock::MockService::new(MockConfig::n23(16, 0)).unwrap();
    let mut headers = http::HeaderMap::new();
    for index in 0..MAX_RAW_RESPONSE_HEADERS {
        let name = http::HeaderName::from_bytes(format!("x-test-{index}").as_bytes()).unwrap();
        headers.insert(
            name,
            http::HeaderValue::from_str(&"v".repeat(MAX_MOCK_HEADER_VALUE_BYTES)).unwrap(),
        );
    }
    controller
        .script(
            1,
            ExchangeScript {
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::OK,
                    headers: headers.clone(),
                    body: vec![0; MAX_RAW_RESPONSE_BODY_BYTES],
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();

    let extra_name = http::HeaderName::from_static("x-test-extra");
    headers.insert(extra_name, "v".parse().unwrap());
    let error = controller
        .script(
            2,
            ExchangeScript {
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::OK,
                    headers,
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap_err();
    assert!(matches!(error, MockConfigError::InvalidScript(_)));

    let mut overlong = request(Method::HEAD, Endpoint::Stash, 3).unwrap();
    overlong.headers_mut().insert(
        rate_limit_core::mock::CORRELATION_HEADER,
        "100000000000000000000".parse().unwrap(),
    );
    assert!(service.send(overlong).await.is_err());
}
