use std::time::Duration;

use http::{HeaderMap, HeaderValue, Method, StatusCode};
use rate_limit_core::actor::{GateError, MIN_SEND_SPACING, spawn, with_correlation_header};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, Resolution};
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, ExchangeScript, MockConfig, MockService, ResponseOverride,
    request,
};

/// N19's applicable bucket is the largest configured policy resolution
/// (60 s in this lane) plus the core's fixed one-second buffer. This is
/// scenario arithmetic, independent of the engine under test (RE-6).
const APPLICABLE_BUCKET_AND_BUFFER_MS: u64 = 60_000 + 1_000;

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

fn policy_response(policy: &str, max_hits: u32, state_hits: u32) -> HeaderMap {
    policy_response_with_periods(policy, max_hits, state_hits, 10, 60)
}

fn policy_response_with_periods(
    policy: &str,
    max_hits: u32,
    state_hits: u32,
    burst_period_secs: u64,
    sustained_period_secs: u64,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-rate-limit-policy",
        HeaderValue::from_str(policy).unwrap(),
    );
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert(
        "x-rate-limit-account",
        HeaderValue::from_str(&format!(
            "{max_hits}:{burst_period_secs}:60, {max_hits}:{sustained_period_secs}:300"
        ))
        .unwrap(),
    );
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_str(&format!(
            "{state_hits}:{burst_period_secs}:0, {state_hits}:{sustained_period_secs}:0"
        ))
        .unwrap(),
    );
    headers
}

#[tokio::test(start_paused = true)]
async fn probe_then_get_share_the_actor_gate_and_keep_distinct_wire_ids() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    let gate = spawn(engine(), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;

    assert!(ticket.await.unwrap().status().is_success());
    let observations = controller.observations().await;
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].method, Method::HEAD);
    assert_eq!(observations[1].method, Method::GET);
    assert_ne!(
        observations[0].correlation_id,
        observations[1].correlation_id
    );
    assert!(
        observations[1].dispatch_ms
            >= observations[0].dispatch_ms + MIN_SEND_SPACING.as_millis() as u64
    );
}

#[tokio::test(start_paused = true)]
async fn queued_cancellation_never_reaches_the_wire() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    let gate = spawn(engine(), mock);
    let first = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    let second = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    second.cancel().await.unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(4)).await;
    tokio::task::yield_now().await;

    assert!(first.await.unwrap().status().is_success());
    assert!(matches!(second.await, Err(GateError::Cancelled)));
    assert_eq!(controller.observations().await.len(), 2);
}

#[tokio::test(start_paused = true)]
async fn degraded_probe_cools_the_endpoint_and_errors_parked_callers() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
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
    let gate = spawn(engine(), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;

    assert!(matches!(ticket.await, Err(GateError::SetupFailed { .. })));
    let observations = controller.observations().await;
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].method, Method::HEAD);
}

#[tokio::test(start_paused = true)]
async fn ordinary_transport_timeout_is_an_unknown_outcome_without_a_reprobe() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    controller
        .script(
            2,
            ExchangeScript {
                response_delay: Duration::from_secs(31),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(35)).await;
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(1)).await;

    assert!(matches!(ticket.await, Err(GateError::TimedOut)));
    let observations = controller.observations().await;
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].method, Method::HEAD);
    assert_eq!(observations[1].method, Method::GET);
}

#[tokio::test(start_paused = true)]
async fn cancelling_dispatched_work_detaches_only_the_caller() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
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
    let gate = spawn(engine(), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    ticket.cancel().await.unwrap();
    assert!(matches!(ticket.await, Err(GateError::Cancelled)));

    tokio::time::advance(Duration::from_secs(6)).await;
    tokio::task::yield_now().await;
    let observations = controller.observations().await;
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[1].method, Method::GET);
}

#[tokio::test(start_paused = true)]
async fn cloudflare_shaped_response_halts_the_gate_and_publishes_status() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
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
    let gate = spawn(engine(), mock);
    let mut status = gate.subscribe_status();
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;

    assert!(matches!(ticket.await, Err(GateError::Halted)));
    status.changed().await.unwrap();
    assert!(status.borrow().halted);
    assert_eq!(controller.observations().await.len(), 2);
}

// M13: force an ordinary reader to remain in flight, then place an unknown
// endpoint behind an established GET. Writer preference means the later HEAD
// must dispatch before that queued GET receives a new ordinary permit.
#[tokio::test(start_paused = true)]
async fn pending_head_writer_blocks_a_front_get_until_it_runs_exclusively() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    let gate = spawn(engine(), mock);
    let boot = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;
    assert!(boot.await.unwrap().status().is_success());

    // Correlation 3 is the first established GET after boot.
    controller
        .script(
            3,
            ExchangeScript {
                response_delay: Duration::from_secs(5),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let reader = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;

    // The first is an established GET at the queue front; the second is an
    // unknown endpoint later in FIFO order. The 250 ms spacing wait lets both
    // commands enter the deque before a second ordinary permit is possible.
    let front_get = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    let writer_get = gate
        .submit(
            EndpointLabel::from(Endpoint::Stash.label()),
            wire_request(Endpoint::Stash),
        )
        .await
        .unwrap();
    for _ in 0..8 {
        tokio::task::yield_now().await;
    }
    tokio::time::advance(Duration::from_millis(300)).await;
    tokio::task::yield_now().await;

    // The delayed reader is still occupying the gate. Once it completes, the
    // unknown endpoint's HEAD, not the earlier queued GET, must hand off next.
    tokio::time::advance(Duration::from_secs(6)).await;
    tokio::task::yield_now().await;
    let handoffs = controller.handoffs().await;
    assert!(handoffs.len() >= 4);
    assert_eq!(handoffs[0].method, Method::HEAD);
    assert_eq!(handoffs[1].method, Method::GET);
    assert_eq!(handoffs[2].method, Method::GET);
    assert_eq!(handoffs[3].method, Method::HEAD, "{handoffs:?}");
    assert_eq!(handoffs[3].endpoint, Endpoint::Stash);

    let observations = controller.observations().await;
    let writer_probe = observations
        .iter()
        .find(|observation| observation.correlation_id == handoffs[3].correlation_id)
        .unwrap();
    assert_eq!(writer_probe.in_flight_at_arrival, 1);
    assert!(!writer_probe.head_overlap);

    tokio::time::advance(Duration::from_secs(3)).await;
    assert!(reader.await.unwrap().status().is_success());
    assert!(front_get.await.unwrap().status().is_success());
    assert!(writer_get.await.unwrap().status().is_success());
}

#[tokio::test(start_paused = true)]
async fn m13_ordinary_waiters_are_fifo_and_never_exceed_two_in_flight() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    for correlation in [3, 4] {
        controller
            .script(
                correlation,
                ExchangeScript {
                    response_delay: Duration::from_secs(5),
                    ..ExchangeScript::default()
                },
            )
            .await
            .unwrap();
    }
    let gate = spawn(engine(), mock);
    let boot = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(4)).await;
    assert!(boot.await.unwrap().status().is_success());

    let mut tickets = Vec::new();
    for _ in 0..3 {
        tickets.push(
            gate.submit(
                EndpointLabel::from(Endpoint::StashList.label()),
                wire_request(Endpoint::StashList),
            )
            .await
            .unwrap(),
        );
    }
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(15)).await;
    tokio::task::yield_now().await;

    for ticket in tickets {
        assert!(ticket.await.unwrap().status().is_success());
    }
    let observations = controller.observations().await;
    let ordinary: Vec<_> = observations
        .iter()
        .filter(|observation| observation.method == Method::GET)
        .collect();
    assert_eq!(ordinary.len(), 4);
    assert_eq!(
        ordinary
            .iter()
            .map(|observation| observation.correlation_id)
            .collect::<Vec<_>>(),
        vec![2, 3, 4, 5]
    );
    assert!(
        observations
            .iter()
            .all(|observation| observation.in_flight_at_arrival <= 2)
    );
}

#[tokio::test(start_paused = true)]
async fn m8_429_requeues_through_the_core_not_before_deadline() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    controller
        .script(
            2,
            ExchangeScript {
                response: Some(ResponseOverride::Full {
                    status: http::StatusCode::TOO_MANY_REQUESTS,
                    retry_after: Some("0".to_owned()),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(70)).await;
    tokio::task::yield_now().await;
    assert!(ticket.await.unwrap().status().is_success());
    let handoffs = controller.handoffs().await;
    assert_eq!(handoffs.len(), 3);
    assert!(handoffs[2].dispatch_ms >= handoffs[1].dispatch_ms + APPLICABLE_BUCKET_AND_BUFFER_MS);
}

#[tokio::test(start_paused = true)]
async fn m8_confirmation_429_escalates_without_a_third_get() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    for correlation in [2, 3] {
        controller
            .script(
                correlation,
                ExchangeScript {
                    response: Some(ResponseOverride::Full {
                        status: http::StatusCode::TOO_MANY_REQUESTS,
                        retry_after: Some("0".to_owned()),
                    }),
                    ..ExchangeScript::default()
                },
            )
            .await
            .unwrap();
    }
    let gate = spawn(engine(), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(70)).await;
    tokio::task::yield_now().await;
    assert!(matches!(ticket.await, Err(GateError::SetupFailed { .. })));
    assert_eq!(controller.handoffs().await.len(), 3);
}

// M5: an ordinary response announces a new policy name. The actor must adopt
// the core's Remapped notification, so the next request stays a GET rather
// than treating its already-known endpoint as an unknown HEAD writer.
#[tokio::test(start_paused = true)]
async fn m5_remap_updates_the_actor_endpoint_mapping() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    controller
        .script(
            2,
            ExchangeScript {
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::OK,
                    headers: policy_response("renamed-policy", 100, 1),
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(), mock);
    let first = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::time::advance(Duration::from_secs(4)).await;
    assert!(first.await.unwrap().status().is_success());

    let second = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(2)).await;
    assert!(second.await.unwrap().status().is_success());
    assert_eq!(
        controller
            .handoffs()
            .await
            .iter()
            .map(|handoff| handoff.method.clone())
            .collect::<Vec<_>>(),
        vec![Method::HEAD, Method::GET, Method::GET]
    );
}

#[tokio::test(start_paused = true)]
async fn remap_then_malformed_response_drains_queued_callers_for_the_current_policy() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    controller
        .script(
            2,
            ExchangeScript {
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::OK,
                    headers: policy_response("renamed-policy", 100, 1),
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(), mock);
    let boot = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::time::advance(Duration::from_secs(4)).await;
    assert!(boot.await.unwrap().status().is_success());

    controller
        .script(
            3,
            ExchangeScript {
                response_delay: Duration::from_secs(5),
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::TOO_MANY_REQUESTS,
                    headers: HeaderMap::new(),
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    controller
        .script(
            4,
            ExchangeScript {
                response_delay: Duration::from_secs(10),
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
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    let queued = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(6)).await;

    assert!(matches!(
        queued.await,
        Err(GateError::SetupFailed { endpoint, .. }) if endpoint == EndpointLabel::from(Endpoint::StashList.label())
    ));
    assert_eq!(controller.handoffs().await.len(), 4);
    assert!(matches!(first.await, Err(GateError::SetupFailed { .. })));
    drop(second);
}

// M6: `stash-list-request-limit` shrinks from its boot policy's 10/30 limits
// to 5/5. Its reported five hits synthesize the conservative deficit, so a
// later GET cannot leave the actor before the adopted padded 60-second window.
#[tokio::test(start_paused = true)]
async fn m6_shrink_blocks_new_dispatches_from_the_announcing_response() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    controller
        .script(
            2,
            ExchangeScript {
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::OK,
                    headers: policy_response_with_periods("stash-list-request-limit", 5, 5, 15, 60),
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(), mock);
    let first = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::time::advance(Duration::from_secs(4)).await;
    assert!(first.await.unwrap().status().is_success());

    let second = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(119)).await;
    assert_eq!(controller.handoffs().await.len(), 2);
    tokio::time::advance(Duration::from_secs(2)).await;
    assert!(second.await.unwrap().status().is_success());
    assert_eq!(controller.handoffs().await.len(), 3);
}
