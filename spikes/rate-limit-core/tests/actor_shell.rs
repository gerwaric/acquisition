use std::time::Duration;

use http::Method;
use rate_limit_core::actor::{GateError, MIN_SEND_SPACING, spawn, with_correlation_header};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, Resolution};
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, ExchangeScript, MockConfig, MockService, ResponseOverride,
    request,
};

fn engine() -> PolicyEngine {
    PolicyEngine::new(BucketModel::new(
        Resolution::Assumed(Duration::from_secs(60)),
        Resolution::Assumed(Duration::from_secs(60)),
    ))
}

fn wire_request() -> rate_limit_core::transport::WireRequest {
    with_correlation_header(
        request(Method::GET, Endpoint::StashList, 0).expect("fixed mock request is valid"),
        CORRELATION_HEADER,
    )
}

#[tokio::test(start_paused = true)]
async fn probe_then_get_share_the_actor_gate_and_keep_distinct_wire_ids() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    let gate = spawn(engine(), mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(),
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
            wire_request(),
        )
        .await
        .unwrap();
    let second = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(),
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
            wire_request(),
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
            wire_request(),
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
            wire_request(),
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
            wire_request(),
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
