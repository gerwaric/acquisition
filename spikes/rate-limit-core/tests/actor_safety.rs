use std::sync::{Arc, Mutex};
use std::time::Duration;

use http::{HeaderMap, HeaderValue, Method, StatusCode};
use rate_limit_core::actor::{GateError, spawn, with_correlation_header};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, RefusalCause, Resolution};
use rate_limit_core::header::PolicyParseError;
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, ExchangeScript, MockConfig, MockService, ResponseOverride,
    request,
};
use rate_limit_core::transport::{Transport, TransportError, WireRequest, WireResponse};

type RecordedResponses = Arc<Mutex<Vec<(StatusCode, Option<String>)>>>;

/// N19's applicable bucket — the maximum configured resolution across the
/// policy's windows, 60 s in both lanes — plus the one-second buffer.
/// Scenario arithmetic, derived here rather than left as a bare 61,000
/// (SD-R5-F14).
const APPLICABLE_BUCKET_AND_BUFFER_MS: u64 = 60_000 + 1_000;

fn engine() -> PolicyEngine {
    PolicyEngine::new(BucketModel::new(
        Resolution::Assumed(Duration::from_secs(60)),
        Resolution::Assumed(Duration::from_secs(60)),
    ))
}

fn wire_request(endpoint: Endpoint) -> WireRequest {
    with_correlation_header(
        request(Method::GET, endpoint, 0).expect("fixed mock request is valid"),
        CORRELATION_HEADER,
    )
}

fn policy_response(policy: &str, max_hits: u32, state_hits: u32) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-rate-limit-policy",
        HeaderValue::from_str(policy).expect("fixed policy name is a header value"),
    );
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert(
        "x-rate-limit-account",
        HeaderValue::from_str(&format!("{max_hits}:10:60, {max_hits}:60:300"))
            .expect("fixed limit triplets are header text"),
    );
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_str(&format!("{state_hits}:10:0, {state_hits}:60:0"))
            .expect("fixed state triplets are header text"),
    );
    headers
}

async fn advance_in_steps(duration: Duration) {
    const STEP: Duration = Duration::from_millis(250);
    let mut elapsed = Duration::ZERO;
    while elapsed < duration {
        let step = (duration - elapsed).min(STEP);
        tokio::time::advance(step).await;
        tokio::task::yield_now().await;
        elapsed += step;
    }
}

#[derive(Clone)]
struct HeaderRecordingTransport {
    inner: MockService,
    responses: RecordedResponses,
}

impl HeaderRecordingTransport {
    fn new(inner: MockService) -> (Self, RecordedResponses) {
        let responses = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                inner,
                responses: responses.clone(),
            },
            responses,
        )
    }
}

impl Transport for HeaderRecordingTransport {
    fn send(
        &self,
        request: WireRequest,
    ) -> impl Future<Output = Result<WireResponse, TransportError>> + Send {
        let inner = self.inner.clone();
        let responses = self.responses.clone();
        async move {
            let response = inner.send(request).await?;
            let retry_after = response.headers().get("retry-after").map(|value| {
                value
                    .to_str()
                    .expect("mock Retry-After is ASCII")
                    .to_owned()
            });
            responses
                .lock()
                .expect("response recorder lock is not poisoned")
                .push((response.status(), retry_after));
            Ok(response)
        }
    }
}

// M1 probe-429 actor arm: the HEAD seeds the endpoint and opens the episode;
// it is never requeued. The first GET is therefore the confirmation attempt,
// so its 429 escalates immediately instead of earning another retry.
#[tokio::test(start_paused = true)]
async fn m1_probe_429_through_actor_seeds_then_first_get_confirms_and_escalates() {
    let (mock, controller) = MockService::new(MockConfig::n23(1, 0)).unwrap();
    for correlation_id in [1, 2] {
        controller
            .script(
                correlation_id,
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
    }
    let gate = spawn(engine(), mock);
    let mut status = gate.subscribe_status();
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();

    advance_in_steps(Duration::from_secs(70)).await;

    // The cause pin distinguishes escalation from a shape cooldown
    // (SD-R5-F15): a cooldown-shaped refusal here would silently change what
    // this test proves.
    assert!(matches!(
        ticket.await,
        Err(GateError::SetupFailed {
            cause: RefusalCause::RecoveryEscalated,
            ..
        })
    ));
    let handoffs = controller.handoffs().await;
    assert_eq!(
        handoffs.len(),
        2,
        "the probe is not requeued and the failed confirmation earns no third knock"
    );
    assert_eq!(handoffs[0].method, Method::HEAD);
    assert_eq!(handoffs[1].method, Method::GET);
    assert!(
        handoffs[1].dispatch_ms >= handoffs[0].dispatch_ms + APPLICABLE_BUCKET_AND_BUFFER_MS,
        "the probe's Retry-After=0 still seeds the 60s bucket + 1s buffer restriction"
    );
    let observations = controller.observations().await;
    assert_eq!(
        observations
            .iter()
            .map(|observation| observation.response_status)
            .collect::<Vec<_>>(),
        vec![
            Some(StatusCode::TOO_MANY_REQUESTS),
            Some(StatusCode::TOO_MANY_REQUESTS)
        ]
    );
    status.changed().await.unwrap();
    assert!(
        !status.borrow().halted,
        "episode escalation is policy-scoped, not a global halt"
    );
}

// M3 / D4: a degraded probe cools only its endpoint. A repeat request on the
// cooled endpoint is refused without another wire attempt while a separate
// endpoint still performs its own one-HEAD bootstrap and GET normally.
#[tokio::test(start_paused = true)]
async fn degraded_probe_cools_one_endpoint_while_another_policy_keeps_flowing() {
    let (mock, controller) = MockService::new(MockConfig::n23(4, 0)).unwrap();
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

    let failed = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_secs(1)).await;
    // A parse-shaped cause, not escalation: the degraded HEAD's refusal must
    // be attributable to the missing headers (SD-R5-F15).
    assert!(matches!(
        failed.await,
        Err(GateError::SetupFailed {
            cause: RefusalCause::PolicyObservation(_),
            ..
        })
    ));
    assert_eq!(controller.handoffs().await.len(), 1);

    let cooled = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    let flowing = gate
        .submit(
            EndpointLabel::from(Endpoint::Stash.label()),
            wire_request(Endpoint::Stash),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_secs(3)).await;

    // The cooldown refusal carries the recorded parse cause forward, so the
    // repeat caller can see *why* the endpoint is cooling.
    assert!(matches!(
        cooled.await,
        Err(GateError::SetupFailed {
            cause: RefusalCause::PolicyObservation(_),
            ..
        })
    ));
    assert!(flowing.await.unwrap().status().is_success());

    // D4's 60 s cooldown is finite: the failed endpoint becomes probeable
    // again and does not retain a permanent refusal state.
    advance_in_steps(Duration::from_secs(61)).await;
    let reopened = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_secs(3)).await;
    assert!(reopened.await.unwrap().status().is_success());

    let handoffs = controller.handoffs().await;
    assert_eq!(
        handoffs
            .iter()
            .map(|handoff| (handoff.endpoint, handoff.method.clone()))
            .collect::<Vec<_>>(),
        vec![
            (Endpoint::StashList, Method::HEAD),
            (Endpoint::Stash, Method::HEAD),
            (Endpoint::Stash, Method::GET),
            (Endpoint::StashList, Method::HEAD),
            (Endpoint::StashList, Method::GET),
        ],
        "cooldown is scoped, the healthy endpoint probes once, and the cooled endpoint re-enters after 60s"
    );
}

#[tokio::test(start_paused = true)]
async fn unexpected_shape_cools_one_endpoint_while_another_policy_keeps_flowing() {
    let (mock, controller) = MockService::new(MockConfig::n23(5, 0)).unwrap();
    controller
        .script(
            1,
            ExchangeScript {
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::NO_CONTENT,
                    headers: {
                        let mut headers = HeaderMap::new();
                        headers
                            .insert("x-rate-limit-policy", HeaderValue::from_static("bad-shape"));
                        headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
                        headers.insert("x-rate-limit-account", HeaderValue::from_static("1:10:60"));
                        headers.insert(
                            "x-rate-limit-account-state",
                            HeaderValue::from_static("0:10:0"),
                        );
                        headers
                    },
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(), mock);
    let failed = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    let flowing = gate
        .submit(
            EndpointLabel::from(Endpoint::Stash.label()),
            wire_request(Endpoint::Stash),
        )
        .await
        .unwrap();

    advance_in_steps(Duration::from_secs(4)).await;
    // Pin the M4 trigger itself: a one-triplet rule must refuse as an
    // unexpected policy shape, not as some other parse failure (SD-R5-F15).
    assert!(matches!(
        failed.await,
        Err(GateError::SetupFailed {
            cause: RefusalCause::PolicyObservation(PolicyParseError::UnexpectedPolicyShape { .. }),
            ..
        })
    ));
    assert!(flowing.await.unwrap().status().is_success());
    let handoffs = controller.handoffs().await;
    assert_eq!(
        handoffs
            .iter()
            .map(|handoff| (handoff.endpoint, handoff.method.clone()))
            .collect::<Vec<_>>(),
        vec![
            (Endpoint::StashList, Method::HEAD),
            (Endpoint::Stash, Method::HEAD),
            (Endpoint::Stash, Method::GET),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn unexpected_shape_cooldown_is_published_on_the_watch_channel() {
    let (mock, controller) = MockService::new(MockConfig::n23(6, 0)).unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("x-rate-limit-policy", HeaderValue::from_static("bad-shape"));
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert("x-rate-limit-account", HeaderValue::from_static("1:10:60"));
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_static("0:10:0"),
    );
    controller
        .script(
            1,
            ExchangeScript {
                response_delay: Duration::from_secs(2),
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::NO_CONTENT,
                    headers,
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let gate = spawn(engine(), mock);
    let mut status = gate.subscribe_status();
    let failed = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_millis(500)).await;
    assert!(status.borrow_and_update().probing.is_some());

    advance_in_steps(Duration::from_secs(3)).await;
    status.changed().await.unwrap();
    assert!(matches!(
        failed.await,
        Err(GateError::SetupFailed {
            cause: RefusalCause::PolicyObservation(PolicyParseError::UnexpectedPolicyShape { .. }),
            ..
        })
    ));
    let published = status.borrow_and_update().clone();
    assert_eq!(published.queued, 0);
    assert_eq!(published.probing, None);
    assert!(!published.halted);
}

// B1 / M8: create an unavoidable same-account race after the actor has
// reserved correlation 3 but before the mock receives it. The wrapper records
// the actual organic wire header, and the retry must flow back through the
// core's single NotBefore authority using that value.
#[tokio::test(start_paused = true)]
async fn organic_429_emits_retry_after_and_actor_honors_the_wire_value() {
    let (mock, controller) = MockService::new(MockConfig::n23(2, 0)).unwrap();
    controller
        .script(
            3,
            ExchangeScript {
                arrival_delay: Duration::from_secs(1),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let (transport, recorded_responses) = HeaderRecordingTransport::new(mock);
    let gate = spawn(engine(), transport);

    let boot = gate
        .submit(
            EndpointLabel::from(Endpoint::Stash.label()),
            wire_request(Endpoint::Stash),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_secs(3)).await;
    assert!(boot.await.unwrap().status().is_success());

    let raced = gate
        .submit(
            EndpointLabel::from(Endpoint::Stash.label()),
            wire_request(Endpoint::Stash),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_millis(500)).await;
    assert_eq!(
        controller.handoffs().await.len(),
        3,
        "the original must be reserved before the phantom arrives"
    );
    controller
        .inject_phantoms("stash-request-limit", 15)
        .await
        .unwrap();

    advance_in_steps(Duration::from_secs(130)).await;
    assert!(raced.await.unwrap().status().is_success());

    let observations = controller.observations().await;
    let organic = observations
        .iter()
        .find(|observation| observation.correlation_id == 3)
        .expect("the delayed original reached the mock");
    assert!(organic.policy_judgment.organic_violation);
    assert_eq!(organic.response_status, Some(StatusCode::TOO_MANY_REQUESTS));

    let retry_after_secs: u64 = {
        let responses = recorded_responses
            .lock()
            .expect("response recorder lock is not poisoned");
        let (_, retry_after) = responses
            .iter()
            .find(|(status, _)| *status == StatusCode::TOO_MANY_REQUESTS)
            .expect("the organic 429 crossed the transport boundary");
        retry_after
            .as_ref()
            .expect("B1 requires Retry-After on an organic 429")
            .parse()
            .expect("mock emits Retry-After delay-seconds")
    };

    let handoffs = controller.handoffs().await;
    let retry = handoffs
        .iter()
        .find(|handoff| handoff.correlation_id == 4)
        .expect("the actor eventually retries the raced request");
    assert!(
        retry.dispatch_ms
            >= organic
                .completion_ms
                .saturating_add(retry_after_secs * 1_000)
                .saturating_add(APPLICABLE_BUCKET_AND_BUFFER_MS),
        "retry must wait the wire Retry-After plus the applicable 60s bucket and 1s buffer"
    );
}

// SHELL: caller drop is not cancellation. Once correlation 3 is confirmed at
// the mock boundary, dropping its ticket must leave the request alive; its
// response reports one extra server-visible hit, and that reconciliation must
// delay the queued follower instead of being discarded with the caller.
#[tokio::test(start_paused = true)]
async fn dropped_dispatched_ticket_still_reconciles_and_does_not_wedge_following_work() {
    let (mock, controller) = MockService::new(MockConfig::n23(3, 0)).unwrap();
    let gate = spawn(engine(), mock);
    let mut status = gate.subscribe_status();

    let boot = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_secs(3)).await;
    assert!(boot.await.unwrap().status().is_success());

    controller
        .script(
            3,
            ExchangeScript {
                response_delay: Duration::from_secs(5),
                response: Some(ResponseOverride::Raw {
                    status: StatusCode::OK,
                    headers: policy_response("stash-list-request-limit", 3, 3),
                    body: Vec::new(),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let dropped = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_millis(500)).await;
    assert_eq!(
        controller.handoffs().await.len(),
        3,
        "drop only after dispatch confirmation"
    );
    drop(dropped);

    // Let the detached response land before introducing the follower. A
    // concurrently reserved follower would be legitimate pre-observation
    // exposure; this assertion is specifically about the next scheduling
    // decision after reconciliation.
    advance_in_steps(Duration::from_secs(6)).await;
    let follower = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            wire_request(Endpoint::StashList),
        )
        .await
        .unwrap();
    advance_in_steps(Duration::from_secs(10)).await;
    assert_eq!(
        controller.handoffs().await.len(),
        3,
        "the dropped request's reported third hit must reconcile and hold the follower"
    );

    advance_in_steps(Duration::from_secs(125)).await;
    assert!(follower.await.unwrap().status().is_success());
    let handoffs = controller.handoffs().await;
    assert_eq!(
        handoffs
            .iter()
            .map(|handoff| handoff.method.clone())
            .collect::<Vec<_>>(),
        vec![Method::HEAD, Method::GET, Method::GET, Method::GET],
        "caller drop must neither reprobe nor wedge later scheduling"
    );
    status.changed().await.unwrap();
    assert_eq!(status.borrow().queued, 0);
    assert_eq!(status.borrow().ordinary_in_flight, 0);
    assert!(!status.borrow().halted);
}
