use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use http::{HeaderMap, HeaderValue, StatusCode};
use proptest::prelude::*;
use rate_limit_core::core::{
    BucketModel, ConfirmationAttempt, Disposition, EmptyPolicy, EntryKind, ObservedResponse,
    Policy, PolicyEngine, PolicyName, RefusalReason, ReplyClassification, ReservationToken,
    ReserveOutcome, Resolution, Rule, RulePair, SimInstant, Window,
};

fn policy_name() -> PolicyName {
    PolicyName::from("stash-request-limit")
}

fn rule(max_hits: u32, burst_ms: u64, sustained_ms: u64) -> Rule {
    let burst = Window::new(
        max_hits,
        Duration::from_millis(burst_ms),
        Duration::from_secs(60),
    );
    let sustained = Window::new(
        max_hits,
        Duration::from_millis(sustained_ms),
        Duration::from_secs(300),
    );
    let pair = RulePair::new(burst, sustained).expect("test periods are increasing");
    Rule::new(
        pair,
        BucketModel::new(
            Resolution::Known(Duration::from_secs(5)),
            Resolution::Known(Duration::from_secs(60)),
        ),
    )
}

fn default_buckets() -> BucketModel {
    BucketModel::new(
        Resolution::Assumed(Duration::from_secs(60)),
        Resolution::Assumed(Duration::from_secs(60)),
    )
}

fn engine(max_hits: u32, burst_ms: u64, sustained_ms: u64) -> PolicyEngine {
    let mut engine = PolicyEngine::new(default_buckets());
    engine
        .insert_policy(
            Policy::new(policy_name(), vec![rule(max_hits, burst_ms, sustained_ms)])
                .expect("the test policy has one rule"),
        )
        .expect("the test inserts one policy");
    engine
}

fn reserve(engine: &mut PolicyEngine, policy: &PolicyName, now: SimInstant) -> ReservationToken {
    match engine.try_reserve(policy, now) {
        ReserveOutcome::Reserved(token) => token,
        other => panic!("expected reservation, got {other:?}"),
    }
}

// C5: same-instant synthetic and local entries are distinguished by EntryId.
#[test]
fn rollback_removes_exact_local_entry() {
    let policy = policy_name();
    let now = SimInstant::from_millis(42);
    let mut engine = engine(10, 100, 1_000);
    engine
        .record_synthetic(&policy, now, 2)
        .expect("policy exists");
    let before = engine.policy(&policy).unwrap().history().clone();

    let token = reserve(&mut engine, &policy, now);
    assert_eq!(token.restriction_generation(), 0);
    let reserved_id = token.entry_id();
    engine.rollback(token);

    assert_eq!(engine.policy(&policy).unwrap().history(), &before);
    assert!(
        engine
            .policy(&policy)
            .unwrap()
            .history()
            .entries()
            .all(|entry| entry.id != reserved_id)
    );
}

// One reservation is one shared history entry, even when multiple rules judge it.
#[test]
fn reservation_is_not_duplicated_across_policy_rules() {
    let policy = policy_name();
    let mut engine = PolicyEngine::new(default_buckets());
    engine
        .insert_policy(
            Policy::new(
                policy.clone(),
                vec![rule(10, 100, 1_000), rule(10, 200, 2_000)],
            )
            .unwrap(),
        )
        .unwrap();

    let token = reserve(&mut engine, &policy, SimInstant::from_millis(5));

    assert_eq!(engine.policy(&policy).unwrap().history().len(), 1);
    let _ = engine.on_unknown_outcome(token, SimInstant::from_millis(6));
}

// C5: an unknown transport result commits the reservation pessimistically.
#[test]
fn unknown_outcome_stays_counted_until_every_window_passes() {
    let policy = policy_name();
    let mut engine = engine(1, 100, 1_000);
    let token = reserve(&mut engine, &policy, SimInstant::from_millis(0));
    let entry_id = token.entry_id();

    let _ = engine.on_unknown_outcome(token, SimInstant::from_millis(1));

    let history = engine.policy(&policy).unwrap().history();
    assert!(history.entries().any(|entry| entry.id == entry_id));
    assert_eq!(
        history.count_within(SimInstant::from_millis(99), Duration::from_millis(100)),
        1
    );
    assert_eq!(
        history.count_within(SimInstant::from_millis(100), Duration::from_millis(100)),
        0
    );
    assert!(matches!(
        engine.try_reserve(&policy, SimInstant::from_millis(999)),
        ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(61_000)
    ));
    let next = reserve(&mut engine, &policy, SimInstant::from_millis(61_000));
    engine.rollback(next);
}

// A rule-less policy would panic the probe path's bucket sizing the moment a
// restriction arrived; the constructor makes the shape unrepresentable instead.
#[test]
fn empty_rule_policies_are_unrepresentable() {
    assert_eq!(
        Policy::new(policy_name(), Vec::new()),
        Err(EmptyPolicy(policy_name()))
    );
}

#[test]
fn unknown_policy_is_refused_without_recording() {
    let mut engine = PolicyEngine::new(default_buckets());
    let policy = policy_name();

    assert!(matches!(
        engine.try_reserve(&policy, SimInstant::from_millis(0)),
        ReserveOutcome::Refused(RefusalReason::UnknownPolicy(name)) if name == policy
    ));
}

proptest! {
    // C5: reserving and rolling back is an exact policy-history round trip.
    #[test]
    fn rollback_restores_history_exactly(
        synthetic_times in prop::collection::vec(0_u64..2_000, 0..32),
        reservation_time in 0_u64..2_000,
    ) {
        let policy = policy_name();
        let mut engine = engine(64, 10_000, 20_000);
        for at in synthetic_times {
            engine.record_synthetic(&policy, SimInstant::from_millis(at), 1).unwrap();
        }
        let before = engine.policy(&policy).unwrap().history().clone();

        let token = reserve(
            &mut engine,
            &policy,
            SimInstant::from_millis(reservation_time),
        );
        engine.rollback(token);

        prop_assert_eq!(engine.policy(&policy).unwrap().history(), &before);
    }

    // C5: arbitrary serialized interleavings of reserve, rollback, observe,
    // and unknown-outcome — over tokens resolved in any order, not just
    // FIFO — retain each send exactly once unless its still-undispatched
    // token is explicitly rolled back.
    #[test]
    fn interleavings_neither_double_count_nor_lose_sends(
        // 1..: an empty operation list would execute no assertion (external
        // review — the vacuity rule applies to degenerate lengths too).
        operations in prop::collection::vec(
            (0_u8..6, 0_u16..20, any::<prop::sample::Index>()),
            1..128,
        ),
    ) {
        let policy = policy_name();
        let mut engine = engine(256, 10_000, 20_000);
        let mut now = 0_u64;
        let mut live_tokens = Vec::<ReservationToken>::new();
        let mut expected = BTreeMap::new();

        for (operation, delta, pick) in operations {
            now += u64::from(delta);
            match operation {
                0 | 5 => {
                    let token = reserve(&mut engine, &policy, SimInstant::from_millis(now));
                    prop_assert_eq!(token.policy(), &policy);
                    let id = token.entry_id();
                    prop_assert!(expected.insert(id, EntryKind::LocalReservation).is_none());
                    live_tokens.push(token);
                }
                1 if !live_tokens.is_empty() => {
                    let token = live_tokens.remove(pick.index(live_tokens.len()));
                    expected.remove(&token.entry_id());
                    engine.rollback(token);
                }
                2 if !live_tokens.is_empty() => {
                    let token = live_tokens.remove(pick.index(live_tokens.len()));
                    let _ = engine.on_unknown_outcome(token, SimInstant::from_millis(now));
                }
                3 if !live_tokens.is_empty() => {
                    // Observed: a valid zero-hit response resolves the token;
                    // its send stays exactly once, still locally attributed.
                    let token = live_tokens.remove(pick.index(live_tokens.len()));
                    let transition =
                        engine.on_response(token, SimInstant::from_millis(now), &ok_response());
                    prop_assert_eq!(transition.disposition, Disposition::CompleteRequest);
                }
                _ => {
                    engine.record_synthetic(&policy, SimInstant::from_millis(now), 1).unwrap();
                    let entry = engine
                        .policy(&policy)
                        .unwrap()
                        .history()
                        .entries()
                        .last()
                        .unwrap();
                    prop_assert!(expected.insert(entry.id, EntryKind::Synthetic).is_none());
                }
            }

            let actual = engine
                .policy(&policy)
                .unwrap()
                .history()
                .entries()
                .map(|entry| (entry.id, entry.kind))
                .collect::<BTreeMap<_, _>>();
            prop_assert_eq!(actual, expected.clone());
        }

        for token in live_tokens {
            let _ = engine.on_unknown_outcome(token, SimInstant::from_millis(now));
        }
    }
}

// C5: abandoning a token is detected, but cannot undo a maybe-sent request.
#[cfg(debug_assertions)]
proptest! {
    #[test]
    fn accidental_drop_bomb_preserves_conservative_history(
        synthetic_count in 0_usize..8,
        at in 0_u64..1_000,
    ) {
        let policy = policy_name();
        let mut engine = engine(32, 10_000, 20_000);
        engine
            .record_synthetic(&policy, SimInstant::from_millis(at), synthetic_count)
            .unwrap();
        let token = reserve(&mut engine, &policy, SimInstant::from_millis(at));
        let abandoned_id = token.entry_id();

        let bomb = catch_unwind(AssertUnwindSafe(|| drop(token)));

        prop_assert!(bomb.is_err());
        let matching_entries = engine
            .policy(&policy)
            .unwrap()
            .history()
            .entries()
            .filter(|entry| entry.id == abandoned_id)
            .collect::<Vec<_>>();
        prop_assert_eq!(matching_entries.len(), 1);
        prop_assert_eq!(matching_entries[0].kind, EntryKind::LocalReservation);
    }
}

// Valid headers for the test policy reporting zero hits, so reconciliation
// synthesizes nothing on top of the locally recorded sends.
fn policy_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-rate-limit-policy",
        HeaderValue::from_static("stash-request-limit"),
    );
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert(
        "x-rate-limit-account",
        HeaderValue::from_static("4:1:60, 4:2:300"),
    );
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_static("0:1:0, 0:2:0"),
    );
    headers
}

fn ok_response() -> ObservedResponse {
    ObservedResponse::new(
        StatusCode::OK,
        policy_headers(),
        ReplyClassification::Normal,
    )
}

// A valid 429 for the test policy: 0s Retry-After, so the restriction is the
// 60s max bucket plus the 1s buffer.
fn rate_limited_response() -> ObservedResponse {
    let mut headers = policy_headers();
    headers.insert("retry-after", HeaderValue::from_static("0"));
    ObservedResponse::new(
        StatusCode::TOO_MANY_REQUESTS,
        headers,
        ReplyClassification::Normal,
    )
}

fn open_episode(engine: &mut PolicyEngine, policy: &PolicyName) {
    let original = reserve(engine, policy, SimInstant::from_millis(0));
    let transition = engine.on_response(
        original,
        SimInstant::from_millis(0),
        &rate_limited_response(),
    );
    assert_eq!(transition.disposition, Disposition::Requeue);
}

// External review finding 5: history retires physically once aged out of
// every padded window — storage is bounded by the window horizon, not by
// process lifetime — and consuming a token whose entry was already retired
// resolves safely instead of panicking.
#[test]
fn history_retires_physically_and_retired_tokens_resolve_safely() {
    let policy = policy_name();
    // Horizon: max(100ms + 5s, 1s + 60s) = 61s.
    let mut engine = engine(4, 100, 1_000);
    let resolved = reserve(&mut engine, &policy, SimInstant::from_millis(0));
    let slow = reserve(&mut engine, &policy, SimInstant::from_millis(0));
    let _ = engine.on_unknown_outcome(resolved, SimInstant::from_millis(1));
    assert_eq!(engine.policy(&policy).unwrap().history().len(), 2);

    // Just inside the horizon both entries persist.
    let inside = reserve(&mut engine, &policy, SimInstant::from_millis(60_000));
    engine.rollback(inside);
    assert_eq!(engine.policy(&policy).unwrap().history().len(), 2);

    // Past the horizon the next scheduling call retires them.
    let later = reserve(&mut engine, &policy, SimInstant::from_millis(200_000));
    engine.rollback(later);
    assert!(engine.policy(&policy).unwrap().history().is_empty());

    // The still-unresolved token whose entry was retired resolves safely.
    let transition = engine.on_unknown_outcome(slow, SimInstant::from_millis(200_000));
    assert_eq!(transition.disposition, Disposition::CompleteRequest);
    assert!(engine.policy(&policy).unwrap().history().is_empty());
}

// C5 abandonment, confirmation half: a dropped confirmation token must not
// wedge the policy. The slot ages out with its history entry, resolving as a
// failed attempt — the episode advances to its final attempt instead of
// answering Blocked forever.
#[test]
fn abandoned_first_confirmation_ages_out_and_permits_the_final_attempt() {
    let policy = policy_name();
    let mut engine = engine(4, 100, 1_000);
    open_episode(&mut engine, &policy);

    let confirmation = reserve(&mut engine, &policy, SimInstant::from_millis(61_000));
    assert!(confirmation.confirmation_attempt().is_some());
    let abandoned = catch_unwind(AssertUnwindSafe(|| drop(confirmation)));
    if cfg!(debug_assertions) {
        assert!(abandoned.is_err(), "debug builds detect the abandonment");
    }

    // Well before the entry ages out, the slot is still held.
    assert!(matches!(
        engine.try_reserve(&policy, SimInstant::from_millis(61_500)),
        ReserveOutcome::Blocked
    ));

    // Once the entry has left every padded window (sustained 1s period plus
    // the 60s sustained bucket), the abandoned attempt resolves as failed and
    // the final attempt becomes reservable.
    let revived = reserve(&mut engine, &policy, SimInstant::from_millis(200_000));
    assert_eq!(
        revived.confirmation_attempt(),
        Some(ConfirmationAttempt::Final)
    );
    engine.rollback(revived);
}

// C5 abandonment: losing the final attempt is accounted as a failed final —
// suspend-and-surface, not an eternal block and not a silent reset.
#[test]
fn abandoned_final_confirmation_escalates_instead_of_wedging() {
    let policy = policy_name();
    let mut engine = engine(4, 100, 1_000);
    open_episode(&mut engine, &policy);

    let first = reserve(&mut engine, &policy, SimInstant::from_millis(61_000));
    let _ = engine.on_unknown_outcome(first, SimInstant::from_millis(61_000));
    let final_attempt = reserve(&mut engine, &policy, SimInstant::from_millis(61_000));
    assert_eq!(
        final_attempt.confirmation_attempt(),
        Some(ConfirmationAttempt::Final)
    );
    let _ = catch_unwind(AssertUnwindSafe(|| drop(final_attempt)));

    assert!(matches!(
        engine.try_reserve(&policy, SimInstant::from_millis(200_000)),
        ReserveOutcome::Refused(RefusalReason::EscalationSuspended(_))
    ));
}

// The bomb must not turn an unrelated panic into a double-panic abort.
#[cfg(debug_assertions)]
#[test]
fn drop_bomb_is_silent_during_unwind() {
    let policy = policy_name();
    let mut engine = engine(1, 100, 1_000);
    let token = reserve(&mut engine, &policy, SimInstant::from_millis(0));

    let panic = catch_unwind(AssertUnwindSafe(move || {
        let _token = token;
        panic!("primary panic");
    }))
    .expect_err("the primary panic should propagate");

    assert_eq!(panic.downcast_ref::<&str>(), Some(&"primary panic"));
}
