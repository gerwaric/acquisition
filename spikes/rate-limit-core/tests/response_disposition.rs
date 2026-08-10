use std::time::Duration;

use http::{HeaderMap, HeaderValue, StatusCode};
use proptest::prelude::*;
use rate_limit_core::core::{
    BucketModel, ConfirmationAttempt, Disposition, EndpointLabel, Notification, ObservedResponse,
    Policy, PolicyEngine, PolicyName, RefusalCause, RefusalReason, RefusalTarget,
    ReplyClassification, ReservationToken, ReserveOutcome, Resolution, Rule, RulePair, RuleScope,
    SimInstant, Window,
};
use rate_limit_core::header::PolicyParseError;

const POLICY: &str = "stash-request-limit";
const ENDPOINT: &str = "stash";

fn configured_rule(
    burst_resolution: Duration,
    sustained_resolution: Duration,
    scope: RuleScope,
) -> Rule {
    Rule::new(
        scope,
        RulePair::new(
            Window::new(1_000, Duration::from_secs(10), Duration::from_secs(60)),
            Window::new(1_000, Duration::from_secs(300), Duration::from_secs(300)),
        )
        .unwrap(),
        BucketModel::new(
            Resolution::Known(burst_resolution),
            Resolution::Assumed(sustained_resolution),
        ),
    )
}

fn engine_with_rules(rules: Vec<Rule>) -> PolicyEngine {
    let mut engine = PolicyEngine::new();
    engine
        .insert_policy(Policy::new(PolicyName::from(POLICY), rules).unwrap())
        .unwrap();
    engine
}

fn engine() -> PolicyEngine {
    engine_with_rules(vec![configured_rule(
        Duration::ZERO,
        Duration::ZERO,
        RuleScope::Account,
    )])
}

fn headers(retry_after: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-rate-limit-policy", HeaderValue::from_static(POLICY));
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert(
        "x-rate-limit-account",
        HeaderValue::from_static("1000:10:60, 1000:300:300"),
    );
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_static("0:10:0, 0:300:0"),
    );
    if let Some(retry_after) = retry_after {
        headers.insert(
            "retry-after",
            HeaderValue::try_from(retry_after).expect("test Retry-After is valid HTTP text"),
        );
    }
    headers
}

fn response(status: StatusCode) -> ObservedResponse {
    ObservedResponse::new(status, headers(None), ReplyClassification::Normal)
}

fn rate_limited(retry_after: &str) -> ObservedResponse {
    ObservedResponse::new(
        StatusCode::TOO_MANY_REQUESTS,
        headers(Some(retry_after)),
        ReplyClassification::Normal,
    )
}

fn rate_limited_with_state(retry_after: &str, state: &'static str) -> ObservedResponse {
    let mut observed = headers(Some(retry_after));
    observed.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_static(state),
    );
    ObservedResponse::new(
        StatusCode::TOO_MANY_REQUESTS,
        observed,
        ReplyClassification::Normal,
    )
}

fn malformed(status: StatusCode) -> ObservedResponse {
    let mut malformed = headers(Some("0"));
    malformed.remove("x-rate-limit-rules");
    ObservedResponse::new(status, malformed, ReplyClassification::Normal)
}

fn cloudflare(status: StatusCode) -> ObservedResponse {
    ObservedResponse::new(
        status,
        HeaderMap::new(),
        ReplyClassification::CloudflareShaped,
    )
}

fn reserve(engine: &mut PolicyEngine, now_ms: u64) -> ReservationToken {
    match engine.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(now_ms)) {
        ReserveOutcome::Reserved(token) => token,
        other => panic!("expected reservation, got {other:?}"),
    }
}

fn open_episode(engine: &mut PolicyEngine) {
    let original = reserve(engine, 0);
    let transition = engine.on_response(original, SimInstant::from_millis(0), &rate_limited("0"));
    assert_eq!(transition.disposition, Disposition::Requeue);
}

fn first_confirmation(engine: &mut PolicyEngine) -> ReservationToken {
    let token = reserve(engine, 1_000);
    assert_eq!(
        token.confirmation_attempt(),
        Some(ConfirmationAttempt::First)
    );
    token
}

fn final_confirmation(engine: &mut PolicyEngine) -> ReservationToken {
    let token = reserve(engine, 1_000);
    assert_eq!(
        token.confirmation_attempt(),
        Some(ConfirmationAttempt::Final)
    );
    token
}

proptest! {
    // M8: arbitrary sets granted under generation zero may land in any mix
    // after the first restriction. Every later 429 advances restriction state,
    // but all pre-restriction tokens attribute to the one existing episode.
    #[test]
    fn arbitrary_generation_tagged_in_flight_sets_join_one_episode(
        later_are_429 in prop::collection::vec(any::<bool>(), 0..32),
    ) {
        let mut engine = engine();
        let mut tokens = Vec::with_capacity(later_are_429.len() + 1);
        for _ in 0..=later_are_429.len() {
            tokens.push(reserve(&mut engine, 0));
        }
        prop_assert!(tokens.iter().all(|token| token.restriction_generation() == 0));

        let first = tokens.remove(0);
        let transition = engine.on_response(
            first,
            SimInstant::from_millis(0),
            &rate_limited("0"),
        );
        prop_assert_eq!(transition.disposition, Disposition::Requeue);

        let mut observed_429s = 1_u64;
        for (token, is_429) in tokens.into_iter().zip(later_are_429) {
            let reply = if is_429 {
                observed_429s += 1;
                rate_limited("0")
            } else {
                response(StatusCode::OK)
            };
            let transition = engine.on_response(token, SimInstant::from_millis(0), &reply);
            let expected = if is_429 {
                Disposition::Requeue
            } else {
                Disposition::CompleteRequest
            };
            prop_assert_eq!(transition.disposition, expected);
        }

        let policy = engine.policy(&PolicyName::from(POLICY)).unwrap();
        prop_assert_eq!(policy.restriction_generation(), observed_429s);
        let episode = policy.recovery_episode().expect("one episode remains open");
        prop_assert_eq!(episode.opened_generation, 1);
        prop_assert_eq!(episode.completed_attempts, 0);

        let confirmation = reserve(&mut engine, 1_000);
        prop_assert_eq!(
            confirmation.confirmation_attempt(),
            Some(ConfirmationAttempt::First)
        );
        prop_assert!(matches!(
            engine.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(1_000)),
            ReserveOutcome::Blocked
        ));
        engine.rollback(confirmation);
    }
}

// N19/M8: retry timing chooses the largest configured bucket over every
// window of every rule, then adds Retry-After and the frozen one-second buffer.
#[test]
fn restriction_uses_maximum_bucket_and_opens_at_the_exact_boundary() {
    let mut engine = engine_with_rules(vec![
        configured_rule(
            Duration::from_secs(5),
            Duration::from_secs(20),
            RuleScope::Account,
        ),
        configured_rule(
            Duration::from_secs(7),
            Duration::from_secs(90),
            RuleScope::Ip,
        ),
    ]);
    let original = reserve(&mut engine, 1_000);
    let transition = engine.on_response(
        original,
        SimInstant::from_millis(1_000),
        &rate_limited_with_state("2", "4:10:60, 6:300:300"),
    );
    assert_eq!(transition.disposition, Disposition::Requeue);

    let policy = engine.policy(&PolicyName::from(POLICY)).unwrap();
    assert_eq!(policy.restriction_generation(), 1);
    assert_eq!(policy.history().len(), 6);
    assert_eq!(
        policy.restricted_until(),
        Some(SimInstant::from_millis(94_000))
    );
    assert!(matches!(
        engine.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(93_999)),
        ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(94_000)
    ));
    let confirmation = reserve(&mut engine, 94_000);
    assert_eq!(
        confirmation.confirmation_attempt(),
        Some(ConfirmationAttempt::First)
    );
    engine.rollback(confirmation);
}

#[test]
fn retry_after_delay_second_boundaries_are_validated_without_a_retry_side_channel() {
    let mut at_cap = engine();
    let token = reserve(&mut at_cap, 0);
    assert_eq!(
        at_cap
            .on_response(token, SimInstant::from_millis(0), &rate_limited("900"))
            .disposition,
        Disposition::Requeue
    );
    assert_eq!(
        at_cap
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .restricted_until(),
        Some(SimInstant::from_millis(901_000))
    );

    // An unusable Retry-After refuses the request AND records a cap-length
    // conservative restriction: the server declared a restriction we cannot
    // size, so the core must not stay immediately grantable.
    // Surrounding whitespace is tolerated, matching the triplet parser.
    let mut trimmed = engine();
    let token = reserve(&mut trimmed, 0);
    assert_eq!(
        trimmed
            .on_response(token, SimInstant::from_millis(0), &rate_limited(" 5 "))
            .disposition,
        Disposition::Requeue
    );

    for reply in [
        rate_limited("901"),
        rate_limited("not-a-number"),
        rate_limited("+5"),
        rate_limited("Wed, 21 Oct 2026 07:28:00 GMT"),
        ObservedResponse::new(
            StatusCode::TOO_MANY_REQUESTS,
            headers(None),
            ReplyClassification::Normal,
        ),
    ] {
        let mut refused = engine();
        let token = reserve(&mut refused, 0);
        let transition = refused.on_response(token, SimInstant::from_millis(0), &reply);
        assert!(matches!(
            transition.disposition,
            Disposition::Refuse {
                cause: RefusalCause::RetryAfter(_),
                ..
            }
        ));
        let policy = refused.policy(&PolicyName::from(POLICY)).unwrap();
        assert_eq!(policy.restriction_generation(), 1);
        // 900s cap + zero-length bucket + 1s buffer.
        assert_eq!(
            policy.restricted_until(),
            Some(SimInstant::from_millis(901_000))
        );
        assert!(policy.recovery_episode().is_none());
        assert!(matches!(
            refused.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(900_999)),
            ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(901_000)
        ));
    }
}

// Probe lane of the same rule: a 429 whose policy observation is valid but
// whose Retry-After is unusable refuses the endpoint AND still blocks the
// observed policy for the conservative cap.
#[test]
fn probe_429_with_unusable_retry_after_records_the_cap_restriction() {
    let endpoint = EndpointLabel::from(ENDPOINT);
    let mut engine = engine();

    let transition = engine.on_probe_response(
        &endpoint,
        SimInstant::from_millis(0),
        &ObservedResponse::new(
            StatusCode::TOO_MANY_REQUESTS,
            headers(None),
            ReplyClassification::Normal,
        ),
    );

    assert!(matches!(
        transition.disposition,
        Disposition::Refuse {
            target: RefusalTarget::Endpoint(_),
            cause: RefusalCause::RetryAfter(_),
        }
    ));
    let policy = engine.policy(&PolicyName::from(POLICY)).unwrap();
    assert_eq!(policy.restriction_generation(), 1);
    assert_eq!(
        policy.restricted_until(),
        Some(SimInstant::from_millis(901_000))
    );
}

#[test]
fn only_one_confirmation_can_be_reserved_and_rollback_reopens_the_attempt() {
    let mut engine = engine();
    open_episode(&mut engine);
    let first = first_confirmation(&mut engine);

    assert!(matches!(
        engine.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(1_000)),
        ReserveOutcome::Blocked
    ));
    engine.rollback(first);

    let replacement = first_confirmation(&mut engine);
    engine.rollback(replacement);
}

#[test]
fn valid_2xx_on_first_confirmation_resets_the_episode() {
    let mut engine = engine();
    open_episode(&mut engine);
    let confirmation = first_confirmation(&mut engine);

    let transition = engine.on_response(
        confirmation,
        SimInstant::from_millis(1_000),
        &response(StatusCode::OK),
    );

    assert_eq!(transition.disposition, Disposition::CompleteRequest);
    assert!(
        engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .recovery_episode()
            .is_none()
    );
    let ordinary = reserve(&mut engine, 1_000);
    assert_eq!(ordinary.confirmation_attempt(), None);
    engine.rollback(ordinary);
}

#[test]
fn a_429_on_the_first_confirmation_escalates_even_after_a_late_original_429() {
    let mut engine = engine();
    let original_one = reserve(&mut engine, 0);
    let original_two = reserve(&mut engine, 0);
    assert_eq!(
        engine
            .on_response(original_one, SimInstant::from_millis(0), &rate_limited("0"))
            .disposition,
        Disposition::Requeue
    );
    let confirmation = first_confirmation(&mut engine);

    assert_eq!(
        engine
            .on_response(
                original_two,
                SimInstant::from_millis(1_000),
                &rate_limited("0")
            )
            .disposition,
        Disposition::Requeue
    );
    assert_eq!(confirmation.restriction_generation(), 1);
    assert_eq!(
        engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .restriction_generation(),
        2
    );

    let transition = engine.on_response(
        confirmation,
        SimInstant::from_millis(1_000),
        &rate_limited("0"),
    );
    assert!(matches!(
        transition.disposition,
        Disposition::Refuse {
            cause: RefusalCause::RecoveryEscalated,
            ..
        }
    ));
    assert!(matches!(
        engine.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(2_000)),
        ReserveOutcome::Refused(RefusalReason::EscalationSuspended(_))
    ));
}

#[test]
fn unknown_first_outcome_stays_counted_and_permits_a_successful_final_attempt() {
    let mut engine = engine();
    open_episode(&mut engine);
    let first = first_confirmation(&mut engine);
    let first_id = first.entry_id();

    assert_eq!(
        engine
            .on_unknown_outcome(first, SimInstant::from_millis(1_000))
            .disposition,
        Disposition::CompleteRequest
    );
    assert!(
        engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .history()
            .entries()
            .any(|entry| entry.id == first_id)
    );

    let final_attempt = final_confirmation(&mut engine);
    assert_eq!(
        engine
            .on_response(
                final_attempt,
                SimInstant::from_millis(1_000),
                &response(StatusCode::NO_CONTENT),
            )
            .disposition,
        Disposition::CompleteRequest
    );
    assert!(
        engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .recovery_episode()
            .is_none()
    );
}

#[test]
fn other_non_429_first_outcomes_preserve_the_episode_for_one_final_attempt() {
    for first_status in [StatusCode::UNAUTHORIZED, StatusCode::INTERNAL_SERVER_ERROR] {
        let mut engine = engine();
        open_episode(&mut engine);
        let first = first_confirmation(&mut engine);
        assert_eq!(
            engine
                .on_response(
                    first,
                    SimInstant::from_millis(1_000),
                    &response(first_status)
                )
                .disposition,
            Disposition::CompleteRequest
        );
        let episode = engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .recovery_episode()
            .unwrap();
        assert_eq!(episode.completed_attempts, 1);
        assert!(episode.confirmation_entry.is_none());
        let final_attempt = final_confirmation(&mut engine);
        engine.rollback(final_attempt);
    }
}

#[test]
fn every_non_success_final_outcome_escalates() {
    enum FinalOutcome {
        Unknown,
        TooManyRequests,
        Other,
        Malformed,
    }

    for outcome in [
        FinalOutcome::Unknown,
        FinalOutcome::TooManyRequests,
        FinalOutcome::Other,
        FinalOutcome::Malformed,
    ] {
        let mut engine = engine();
        open_episode(&mut engine);
        let first = first_confirmation(&mut engine);
        assert_eq!(
            engine
                .on_unknown_outcome(first, SimInstant::from_millis(1_000))
                .disposition,
            Disposition::CompleteRequest
        );
        let final_attempt = final_confirmation(&mut engine);
        let transition = match outcome {
            FinalOutcome::Unknown => {
                engine.on_unknown_outcome(final_attempt, SimInstant::from_millis(1_000))
            }
            FinalOutcome::TooManyRequests => engine.on_response(
                final_attempt,
                SimInstant::from_millis(1_000),
                &rate_limited("0"),
            ),
            FinalOutcome::Other => engine.on_response(
                final_attempt,
                SimInstant::from_millis(1_000),
                &response(StatusCode::BAD_GATEWAY),
            ),
            FinalOutcome::Malformed => engine.on_response(
                final_attempt,
                SimInstant::from_millis(1_000),
                &malformed(StatusCode::OK),
            ),
        };
        assert!(matches!(transition.disposition, Disposition::Refuse { .. }));
        assert!(
            engine
                .policy(&PolicyName::from(POLICY))
                .unwrap()
                .is_escalation_suspended()
        );
    }
}

// Precedence rule 4: a generic 4xx with valid headers is an ordinary
// response for a non-confirmation token — it reconciles and completes; the
// 4xx tripwire lives at the transport boundary, not here.
#[test]
fn generic_4xx_with_valid_headers_completes_and_reconciles() {
    let mut engine = engine();
    let token = reserve(&mut engine, 0);

    let transition = engine.on_response(
        token,
        SimInstant::from_millis(0),
        &response(StatusCode::NOT_FOUND),
    );

    assert_eq!(transition.disposition, Disposition::CompleteRequest);
    let policy = engine.policy(&PolicyName::from(POLICY)).unwrap();
    assert_eq!(policy.restriction_generation(), 0);
    assert!(policy.recovery_episode().is_none());
    assert_eq!(policy.history().len(), 1);
}

// N19 boundary: Retry-After of zero still buys the bucket-plus-buffer pad.
#[test]
fn retry_after_zero_restricts_for_exactly_bucket_plus_buffer() {
    let mut engine = engine(); // zero-length buckets
    let token = reserve(&mut engine, 5_000);

    let transition = engine.on_response(token, SimInstant::from_millis(5_000), &rate_limited("0"));

    assert_eq!(transition.disposition, Disposition::Requeue);
    assert_eq!(
        engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .restricted_until(),
        Some(SimInstant::from_millis(6_000))
    );
}

// Halt is terminal and outside the confirmation matrix on the FINAL attempt
// too: Cloudflare never receives the otherwise "permitted" escalation row.
#[test]
fn cloudflare_on_the_final_attempt_halts_instead_of_escalating() {
    let mut engine = engine();
    open_episode(&mut engine);
    let first = first_confirmation(&mut engine);
    let _ = engine.on_unknown_outcome(first, SimInstant::from_millis(1_000));
    let final_attempt = final_confirmation(&mut engine);

    let transition = engine.on_response(
        final_attempt,
        SimInstant::from_millis(1_000),
        &cloudflare(StatusCode::TOO_MANY_REQUESTS),
    );

    assert_eq!(transition.disposition, Disposition::Halt);
    assert!(engine.is_halted());
    assert!(
        !engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .is_escalation_suspended(),
        "halt supersedes escalation; it does not masquerade as one"
    );
}

// StateChanged means exactly "this call mutated engine state": an actor can
// trust an empty notification list as nothing-to-publish.
#[test]
fn state_changed_notification_tracks_actual_mutation() {
    // Zero-deficit 2xx on an ordinary token: no mutation, no notification.
    let mut quiet = engine();
    let token = reserve(&mut quiet, 0);
    let transition =
        quiet.on_response(token, SimInstant::from_millis(0), &response(StatusCode::OK));
    assert_eq!(transition.disposition, Disposition::CompleteRequest);
    assert!(transition.notifications.is_empty());

    // A malformed refusal on an ordinary token mutates nothing either.
    let mut refused = engine();
    let token = reserve(&mut refused, 0);
    let transition = refused.on_response(
        token,
        SimInstant::from_millis(0),
        &malformed(StatusCode::OK),
    );
    assert!(transition.notifications.is_empty());

    // A 429 records a restriction and opens an episode.
    let mut restricted = engine();
    let token = reserve(&mut restricted, 0);
    let transition = restricted.on_response(token, SimInstant::from_millis(0), &rate_limited("0"));
    assert_eq!(transition.notifications, [Notification::StateChanged]);

    // A 2xx whose state header reports unmodeled hits synthesizes history.
    let mut synthesized = engine();
    let token = reserve(&mut synthesized, 0);
    let mut observed = headers(None);
    observed.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_static("3:10:0, 3:300:0"),
    );
    let transition = synthesized.on_response(
        token,
        SimInstant::from_millis(0),
        &ObservedResponse::new(StatusCode::OK, observed, ReplyClassification::Normal),
    );
    assert_eq!(transition.notifications, [Notification::StateChanged]);

    // A zero-deficit probe success mutates nothing; the mapping is actor state.
    let mut probed = engine();
    let transition = probed.on_probe_response(
        &EndpointLabel::from(ENDPOINT),
        SimInstant::from_millis(0),
        &response(StatusCode::NO_CONTENT),
    );
    assert_eq!(transition.disposition, Disposition::ProbeReady);
    assert!(transition.notifications.is_empty());
}

// core-design entry-point invariant: on_response never yields ProbeReady;
// on_probe_response never yields CompleteRequest or Requeue. The type system
// does not enforce this (one shared Disposition enum), so the sweep does.
#[test]
fn entry_point_invariant_holds_across_response_shapes() {
    let replies = [
        response(StatusCode::OK),
        response(StatusCode::NO_CONTENT),
        response(StatusCode::NOT_FOUND),
        response(StatusCode::INTERNAL_SERVER_ERROR),
        rate_limited("0"),
        ObservedResponse::new(
            StatusCode::TOO_MANY_REQUESTS,
            headers(None),
            ReplyClassification::Normal,
        ),
        malformed(StatusCode::OK),
        malformed(StatusCode::TOO_MANY_REQUESTS),
        cloudflare(StatusCode::FORBIDDEN),
    ];
    let endpoint = EndpointLabel::from(ENDPOINT);

    for reply in &replies {
        let mut ordinary = engine();
        let token = reserve(&mut ordinary, 0);
        let transition = ordinary.on_response(token, SimInstant::from_millis(0), reply);
        assert_ne!(
            transition.disposition,
            Disposition::ProbeReady,
            "on_response yielded ProbeReady for {reply:?}"
        );

        let mut probe = engine();
        let transition = probe.on_probe_response(&endpoint, SimInstant::from_millis(0), reply);
        assert!(
            !matches!(
                transition.disposition,
                Disposition::CompleteRequest | Disposition::Requeue
            ),
            "on_probe_response yielded a request disposition for {reply:?}"
        );
    }
}

#[test]
fn malformed_429_takes_precedence_and_never_opens_a_retry_episode() {
    let mut engine = engine();
    let token = reserve(&mut engine, 0);

    let transition = engine.on_response(
        token,
        SimInstant::from_millis(0),
        &malformed(StatusCode::TOO_MANY_REQUESTS),
    );

    assert!(matches!(
        transition.disposition,
        Disposition::Refuse {
            target: RefusalTarget::Policy(_),
            cause: RefusalCause::PolicyObservation(PolicyParseError::MissingHeader { .. }),
        }
    ));
    let policy = engine.policy(&PolicyName::from(POLICY)).unwrap();
    assert_eq!(policy.restriction_generation(), 0);
    assert_eq!(policy.restricted_until(), None);
    assert!(policy.recovery_episode().is_none());
}

#[test]
fn malformed_429_on_first_confirmation_uses_refusal_path_and_permits_final_attempt() {
    let mut engine = engine();
    open_episode(&mut engine);
    let first = first_confirmation(&mut engine);
    let generation_before = engine
        .policy(&PolicyName::from(POLICY))
        .unwrap()
        .restriction_generation();

    let transition = engine.on_response(
        first,
        SimInstant::from_millis(1_000),
        &malformed(StatusCode::TOO_MANY_REQUESTS),
    );

    assert!(matches!(
        transition.disposition,
        Disposition::Refuse {
            cause: RefusalCause::PolicyObservation(_),
            ..
        }
    ));
    let policy = engine.policy(&PolicyName::from(POLICY)).unwrap();
    assert_eq!(policy.restriction_generation(), generation_before);
    assert_eq!(policy.recovery_episode().unwrap().completed_attempts, 1);
    let final_attempt = final_confirmation(&mut engine);
    engine.rollback(final_attempt);
}

#[test]
fn cloudflare_shape_halts_before_status_or_header_handling() {
    let mut engine = engine();
    open_episode(&mut engine);
    let confirmation = first_confirmation(&mut engine);

    let transition = engine.on_response(
        confirmation,
        SimInstant::from_millis(1_000),
        &cloudflare(StatusCode::TOO_MANY_REQUESTS),
    );

    assert_eq!(transition.disposition, Disposition::Halt);
    assert!(engine.is_halted());
    assert!(matches!(
        engine.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(1_000)),
        ReserveOutcome::Refused(RefusalReason::Halted)
    ));
}

#[test]
fn probe_outcome_table_is_total_for_non_429_rows() {
    let endpoint = EndpointLabel::from(ENDPOINT);

    let mut success = engine();
    assert_eq!(
        success
            .on_probe_response(
                &endpoint,
                SimInstant::from_millis(0),
                &response(StatusCode::NO_CONTENT),
            )
            .disposition,
        Disposition::ProbeReady
    );

    let mut malformed_engine = engine();
    assert!(matches!(
        malformed_engine
            .on_probe_response(
                &endpoint,
                SimInstant::from_millis(0),
                &malformed(StatusCode::NO_CONTENT),
            )
            .disposition,
        Disposition::Refuse {
            target: RefusalTarget::Endpoint(_),
            cause: RefusalCause::PolicyObservation(_),
        }
    ));

    let mut server_error = engine();
    assert!(matches!(
        server_error
            .on_probe_response(
                &endpoint,
                SimInstant::from_millis(0),
                &response(StatusCode::INTERNAL_SERVER_ERROR),
            )
            .disposition,
        Disposition::Refuse {
            cause: RefusalCause::ProbeStatus(StatusCode::INTERNAL_SERVER_ERROR),
            ..
        }
    ));

    assert!(matches!(
        engine().on_probe_unknown_outcome(&endpoint).disposition,
        Disposition::Refuse {
            cause: RefusalCause::ProbeUnknownOutcome,
            ..
        }
    ));

    let mut halted = engine();
    assert_eq!(
        halted
            .on_probe_response(
                &endpoint,
                SimInstant::from_millis(0),
                &cloudflare(StatusCode::FORBIDDEN),
            )
            .disposition,
        Disposition::Halt
    );
    assert!(halted.is_halted());
}

#[test]
fn valid_probe_429_seeds_restriction_and_first_get_is_confirmation() {
    let endpoint = EndpointLabel::from(ENDPOINT);
    let mut engine = engine_with_rules(vec![configured_rule(
        Duration::from_secs(5),
        Duration::from_secs(60),
        RuleScope::Account,
    )]);

    let transition = engine.on_probe_response(
        &endpoint,
        SimInstant::from_millis(5_000),
        &rate_limited_with_state("3", "4:10:60, 6:300:300"),
    );

    assert_eq!(transition.disposition, Disposition::ProbeReady);
    let policy = engine.policy(&PolicyName::from(POLICY)).unwrap();
    assert_eq!(policy.restriction_generation(), 1);
    assert_eq!(policy.history().len(), 6);
    assert_eq!(
        policy.restricted_until(),
        Some(SimInstant::from_millis(69_000))
    );
    assert!(policy.recovery_episode().is_some());
    assert!(matches!(
        engine.try_reserve(&PolicyName::from(POLICY), SimInstant::from_millis(68_999)),
        ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(69_000)
    ));
    let first_get = reserve(&mut engine, 69_000);
    assert_eq!(
        first_get.confirmation_attempt(),
        Some(ConfirmationAttempt::First)
    );
    let transition = engine.on_response(
        first_get,
        SimInstant::from_millis(69_000),
        &rate_limited("0"),
    );
    assert!(matches!(
        transition.disposition,
        Disposition::Refuse {
            cause: RefusalCause::RecoveryEscalated,
            ..
        }
    ));
}

// Probe-opened episodes obey the full confirmation matrix — "single
// confirmation attempt" in the scenario prose means one in flight at a time,
// not a one-attempt cap (clarified 2026-08-09, Tom-approved): a non-429
// non-2xx first GET preserves the episode and earns the matrix's final
// attempt, exactly as an ordinary-429-opened episode would.
#[test]
fn probe_opened_episode_permits_the_matrix_final_attempt() {
    let endpoint = EndpointLabel::from(ENDPOINT);
    let mut engine = engine();
    let transition =
        engine.on_probe_response(&endpoint, SimInstant::from_millis(0), &rate_limited("0"));
    assert_eq!(transition.disposition, Disposition::ProbeReady);

    // The seeded restriction (0s Retry-After + zero bucket + 1s buffer)
    // passes at 1s; the first GET is the episode's first confirmation.
    let first_get = reserve(&mut engine, 1_000);
    assert_eq!(
        first_get.confirmation_attempt(),
        Some(ConfirmationAttempt::First)
    );
    let transition = engine.on_response(
        first_get,
        SimInstant::from_millis(1_000),
        &response(StatusCode::INTERNAL_SERVER_ERROR),
    );
    assert_eq!(transition.disposition, Disposition::CompleteRequest);

    let episode = engine
        .policy(&PolicyName::from(POLICY))
        .unwrap()
        .recovery_episode()
        .expect("a server error must not reset the episode");
    assert_eq!(episode.completed_attempts, 1);
    let final_attempt = final_confirmation(&mut engine);
    engine.rollback(final_attempt);
}

#[test]
fn probe_429_without_valid_observation_refuses_without_restriction() {
    let endpoint = EndpointLabel::from(ENDPOINT);
    let mut engine = engine();

    let transition = engine.on_probe_response(
        &endpoint,
        SimInstant::from_millis(0),
        &malformed(StatusCode::TOO_MANY_REQUESTS),
    );

    assert!(matches!(
        transition.disposition,
        Disposition::Refuse {
            target: RefusalTarget::Endpoint(_),
            cause: RefusalCause::PolicyObservation(_),
        }
    ));
    assert_eq!(
        engine
            .policy(&PolicyName::from(POLICY))
            .unwrap()
            .restriction_generation(),
        0
    );
}
