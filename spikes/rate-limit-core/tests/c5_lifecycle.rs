use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

use proptest::prelude::*;
use rate_limit_core::core::{
    BucketModel, EmptyPolicy, EntryKind, Policy, PolicyEngine, PolicyName, RefusalReason,
    ReservationToken, ReserveOutcome, Resolution, Rule, RulePair, RuleScope, SimInstant, Window,
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
        RuleScope::Account,
        pair,
        BucketModel::new(
            Resolution::Known(Duration::from_secs(5)),
            Resolution::Known(Duration::from_secs(60)),
        ),
    )
}

fn engine(max_hits: u32, burst_ms: u64, sustained_ms: u64) -> PolicyEngine {
    let mut engine = PolicyEngine::new();
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
            .all(|entry| entry.id() != reserved_id)
    );
}

// One reservation is one shared history entry, even when multiple rules judge it.
#[test]
fn reservation_is_not_duplicated_across_policy_rules() {
    let policy = policy_name();
    let mut engine = PolicyEngine::new();
    engine
        .insert_policy(
            Policy::new(policy.clone(), vec![rule(10, 100, 1_000), rule(10, 200, 2_000)]).unwrap(),
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
    assert!(history.entries().any(|entry| entry.id() == entry_id));
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
    let mut engine = PolicyEngine::new();
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

    // C5: arbitrary serialized interleavings retain each send exactly once
    // unless its still-undispatched token is explicitly rolled back.
    #[test]
    fn interleavings_neither_double_count_nor_lose_sends(
        operations in prop::collection::vec((0_u8..4, 0_u16..20), 0..128),
    ) {
        let policy = policy_name();
        let mut engine = engine(256, 10_000, 20_000);
        let mut now = 0_u64;
        let mut live_tokens = Vec::<ReservationToken>::new();
        let mut expected = BTreeMap::new();

        for (operation, delta) in operations {
            now += u64::from(delta);
            match operation {
                0 | 3 => {
                    let token = reserve(&mut engine, &policy, SimInstant::from_millis(now));
                    prop_assert_eq!(token.policy(), &policy);
                    let id = token.entry_id();
                    prop_assert!(expected.insert(id, EntryKind::LocalReservation).is_none());
                    live_tokens.push(token);
                }
                1 if !live_tokens.is_empty() => {
                    let token = live_tokens.remove(0);
                    expected.remove(&token.entry_id());
                    engine.rollback(token);
                }
                2 if !live_tokens.is_empty() => {
                    let token = live_tokens.remove(0);
                    let _ = engine.on_unknown_outcome(token, SimInstant::from_millis(now));
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
                    prop_assert!(expected.insert(entry.id(), EntryKind::Synthetic).is_none());
                }
            }

            let actual = engine
                .policy(&policy)
                .unwrap()
                .history()
                .entries()
                .map(|entry| (entry.id(), entry.kind()))
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
            .filter(|entry| entry.id() == abandoned_id)
            .collect::<Vec<_>>();
        prop_assert_eq!(matching_entries.len(), 1);
        prop_assert_eq!(matching_entries[0].kind(), EntryKind::LocalReservation);
    }
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
