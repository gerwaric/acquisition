use std::time::Duration;

use http::{HeaderMap, HeaderName, HeaderValue};
use proptest::prelude::*;
use rate_limit_core::core::{
    BucketModel, EntryKind, HistoryEntry, ObservationError, Policy, PolicyEngine, PolicyName,
    ReserveOutcome, Resolution, Rule, RulePair, RuleScope, SimInstant, Window,
};
use rate_limit_core::header::{PolicySnapshot, parse_policy};

const NOW_MS: u64 = 100_000;

#[derive(Clone, Debug)]
struct ObservedRule {
    burst_hits: u32,
    burst_period_secs: u32,
    sustained_hits: u32,
    sustained_period_secs: u32,
}

fn policy_name() -> PolicyName {
    PolicyName::from("stash-request-limit")
}

fn configured_rule() -> Rule {
    let pair = RulePair::new(
        Window::new(512, Duration::from_secs(1_000), Duration::ZERO),
        Window::new(512, Duration::from_secs(2_000), Duration::ZERO),
    )
    .expect("configured periods increase");
    Rule::new(
        RuleScope::Account,
        pair,
        BucketModel::new(
            Resolution::Known(Duration::ZERO),
            Resolution::Known(Duration::ZERO),
        ),
    )
}

fn engine() -> PolicyEngine {
    let mut engine = PolicyEngine::new();
    engine
        .insert_policy(Policy::new(policy_name(), vec![configured_rule()]))
        .expect("the test inserts one policy");
    engine
}

fn reserve(
    engine: &mut PolicyEngine,
    policy: &PolicyName,
    now: SimInstant,
) -> rate_limit_core::core::ReservationToken {
    match engine.try_reserve(policy, now) {
        ReserveOutcome::Reserved(token) => token,
        other => panic!("expected reservation, got {other:?}"),
    }
}

fn add_history(engine: &mut PolicyEngine, entries: &[(bool, u64)]) {
    let policy = policy_name();
    for &(local, at_ms) in entries {
        let at = SimInstant::from_millis(at_ms);
        if local {
            let token = reserve(engine, &policy, at);
            engine.on_unknown_outcome(token, at);
        } else {
            engine
                .record_synthetic(&policy, at, 1)
                .expect("policy exists");
        }
    }
}

fn build_observation(rules: &[ObservedRule]) -> PolicySnapshot {
    build_observation_for("stash-request-limit", rules)
}

fn build_observation_for(policy: &str, rules: &[ObservedRule]) -> PolicySnapshot {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-rate-limit-policy",
        HeaderValue::try_from(policy).expect("valid policy-header value"),
    );
    let rule_names = (0..rules.len())
        .map(|index| format!("rule{index}"))
        .collect::<Vec<_>>();
    headers.insert(
        "x-rate-limit-rules",
        HeaderValue::try_from(rule_names.join(", ")).expect("valid rule-list value"),
    );

    for (rule_name, rule) in rule_names.iter().zip(rules) {
        let limit_name = HeaderName::try_from(format!("x-rate-limit-{rule_name}"))
            .expect("generated header name is valid");
        let state_name = HeaderName::try_from(format!("x-rate-limit-{rule_name}-state"))
            .expect("generated state-header name is valid");
        let limit = format!(
            "{}:{}:0, {}:{}:0",
            rule.burst_hits.max(100),
            rule.burst_period_secs,
            rule.sustained_hits.max(100),
            rule.sustained_period_secs,
        );
        let state = format!(
            "{}:{}:0, {}:{}:0",
            rule.burst_hits,
            rule.burst_period_secs,
            rule.sustained_hits,
            rule.sustained_period_secs,
        );
        headers.insert(
            limit_name,
            HeaderValue::try_from(limit).expect("generated limit value is valid"),
        );
        headers.insert(
            state_name,
            HeaderValue::try_from(state).expect("generated state value is valid"),
        );
    }

    parse_policy(&headers).expect("generated observation is valid")
}

fn expected_maximum_deficit(
    engine: &PolicyEngine,
    now: SimInstant,
    observation: &PolicySnapshot,
) -> usize {
    let history = engine
        .policy(&policy_name())
        .expect("policy exists")
        .history();
    observation
        .rules()
        .iter()
        .flat_map(|rule| [rule.state.burst(), rule.state.sustained()])
        .map(|state| {
            state.current_hits() as usize
                - history
                    .count_within(now, state.period())
                    .min(state.current_hits() as usize)
        })
        .max()
        .unwrap_or(0)
}

fn entries(engine: &PolicyEngine) -> Vec<HistoryEntry> {
    engine
        .policy(&policy_name())
        .expect("policy exists")
        .history()
        .entries()
        .cloned()
        .collect()
}

fn assert_pessimistic(
    engine: &PolicyEngine,
    now: SimInstant,
    observation: &PolicySnapshot,
) -> Result<(), TestCaseError> {
    let history = engine
        .policy(&policy_name())
        .expect("policy exists")
        .history();
    for state in observation
        .rules()
        .iter()
        .flat_map(|rule| [rule.state.burst(), rule.state.sustained()])
    {
        prop_assert!(
            history.count_within(now, state.period()) >= state.current_hits() as usize,
            "local count remained below reported post-increment count"
        );
    }
    Ok(())
}

fn observed_rules() -> impl Strategy<Value = Vec<ObservedRule>> {
    prop::collection::vec(
        (0_u32..80, 1_u32..40, 0_u32..80, 1_u32..40).prop_map(
            |(burst_hits, burst_period_secs, sustained_hits, sustained_delta_secs)| ObservedRule {
                burst_hits,
                burst_period_secs,
                sustained_hits,
                sustained_period_secs: burst_period_secs + sustained_delta_secs,
            },
        ),
        1..5,
    )
}

proptest! {
    // M1/M7 core mechanism: arbitrary mixed local/synthetic history and every
    // reported rule window participate, but shared history grows by one maximum.
    #[test]
    fn reconciliation_is_pessimistic_and_synthesizes_exactly_the_maximum_deficit(
        initial in prop::collection::vec((any::<bool>(), 0_u64..=NOW_MS), 0..64),
        rules in observed_rules(),
    ) {
        let now = SimInstant::from_millis(NOW_MS);
        let policy = policy_name();
        let mut engine = engine();
        add_history(&mut engine, &initial);
        let token = reserve(&mut engine, &policy, now);
        let observation = build_observation(&rules);
        let before = entries(&engine);
        let expected = expected_maximum_deficit(&engine, now, &observation);

        let result = engine
            .on_response(token, now, &observation)
            .expect("observation targets the reserved policy");

        prop_assert_eq!(result.synthesized_entries(), expected);
        let after = entries(&engine);
        prop_assert_eq!(&after[..before.len()], before.as_slice());
        prop_assert_eq!(after.len(), before.len() + expected);
        for entry in &after[before.len()..] {
            prop_assert_eq!(entry.at(), now);
            prop_assert_eq!(entry.kind(), EntryKind::Synthetic);
        }
        assert_pessimistic(&engine, now, &observation)?;
    }

    // Later lower observations and exact repeats cannot retract or duplicate
    // shared history. Each step independently obeys the same max-deficit rule.
    #[test]
    fn repeated_and_lower_probe_observations_are_monotone(
        initial in prop::collection::vec((any::<bool>(), 0_u64..=NOW_MS), 0..64),
        rules in observed_rules(),
    ) {
        let now = SimInstant::from_millis(NOW_MS);
        let mut engine = engine();
        add_history(&mut engine, &initial);
        let observation = build_observation(&rules);
        let before = entries(&engine);
        let expected = expected_maximum_deficit(&engine, now, &observation);

        let first = engine.on_probe_response(now, &observation).unwrap();
        let after_first = entries(&engine);
        prop_assert_eq!(first.synthesized_entries(), expected);
        prop_assert_eq!(&after_first[..before.len()], before.as_slice());
        assert_pessimistic(&engine, now, &observation)?;

        let repeated = engine.on_probe_response(now, &observation).unwrap();
        prop_assert_eq!(repeated.synthesized_entries(), 0);
        prop_assert_eq!(entries(&engine), after_first.clone());

        let lower_rules = rules
            .iter()
            .map(|rule| ObservedRule {
                burst_hits: rule.burst_hits / 2,
                burst_period_secs: rule.burst_period_secs,
                sustained_hits: rule.sustained_hits / 2,
                sustained_period_secs: rule.sustained_period_secs,
            })
            .collect::<Vec<_>>();
        let lower = build_observation(&lower_rules);
        let lower_result = engine.on_probe_response(now, &lower).unwrap();
        prop_assert_eq!(lower_result.synthesized_entries(), 0);
        prop_assert_eq!(entries(&engine), after_first);
    }
}

// M1 core mechanism: a non-counting boot probe seeds cross-session residue
// into an otherwise empty policy through ordinary reconciliation.
#[test]
fn boot_probe_residue_uses_the_largest_reported_window_count() {
    let now = SimInstant::from_millis(NOW_MS);
    let mut engine = engine();
    let observation = build_observation(&[
        ObservedRule {
            burst_hits: 3,
            burst_period_secs: 10,
            sustained_hits: 7,
            sustained_period_secs: 60,
        },
        ObservedRule {
            burst_hits: 5,
            burst_period_secs: 20,
            sustained_hits: 6,
            sustained_period_secs: 120,
        },
    ]);

    let result = engine.on_probe_response(now, &observation).unwrap();

    assert_eq!(result.synthesized_entries(), 7);
    assert_eq!(entries(&engine).len(), 7);
    assert!(
        entries(&engine)
            .iter()
            .all(|entry| entry.kind() == EntryKind::Synthetic && entry.at() == now)
    );
}

// N25 post-increment semantics: the reservation representing this send is
// already one of the reported hits and must not be synthesized a second time.
#[test]
fn ordinary_response_never_double_counts_its_reserved_send() {
    let now = SimInstant::from_millis(NOW_MS);
    let policy = policy_name();
    let mut engine = engine();
    add_history(
        &mut engine,
        &[
            (true, 99_000),
            (false, 99_000),
            (true, 99_500),
            (false, 99_500),
        ],
    );
    let token = reserve(&mut engine, &policy, now);
    let observation = build_observation(&[ObservedRule {
        burst_hits: 5,
        burst_period_secs: 10,
        sustained_hits: 5,
        sustained_period_secs: 60,
    }]);

    let result = engine.on_response(token, now, &observation).unwrap();

    assert_eq!(result.synthesized_entries(), 0);
    assert_eq!(entries(&engine).len(), 5);
}

// M7 core mechanism: only the unmodeled same-account hits are synthesized.
#[test]
fn ordinary_response_synthesizes_only_the_phantom_deficit() {
    let now = SimInstant::from_millis(NOW_MS);
    let policy = policy_name();
    let mut engine = engine();
    add_history(&mut engine, &[(true, 99_000), (false, 99_500)]);
    let token = reserve(&mut engine, &policy, now);
    let observation = build_observation(&[ObservedRule {
        burst_hits: 6,
        burst_period_secs: 10,
        sustained_hits: 5,
        sustained_period_secs: 60,
    }]);

    let result = engine.on_response(token, now, &observation).unwrap();

    assert_eq!(result.synthesized_entries(), 3);
    assert_eq!(entries(&engine).len(), 6);
}

#[test]
fn shared_history_inserts_maximum_deficit_not_sum() {
    let now = SimInstant::from_millis(NOW_MS);
    let mut engine = engine();
    add_history(
        &mut engine,
        &[(true, 80_000), (false, 95_000), (true, 99_000)],
    );
    let observation = build_observation(&[
        ObservedRule {
            burst_hits: 5,
            burst_period_secs: 10,
            sustained_hits: 6,
            sustained_period_secs: 60,
        },
        ObservedRule {
            burst_hits: 4,
            burst_period_secs: 5,
            sustained_hits: 7,
            sustained_period_secs: 120,
        },
    ]);

    // Local counts are [2, 3, 1, 3], so deficits are [3, 3, 3, 4].
    let result = engine.on_probe_response(now, &observation).unwrap();

    assert_eq!(result.synthesized_entries(), 4);
    assert_eq!(entries(&engine).len(), 7);
}

#[test]
fn same_instant_synthesis_does_not_weaken_exact_rollback_identity() {
    let now = SimInstant::from_millis(NOW_MS);
    let policy = policy_name();
    let mut engine = engine();
    engine.record_synthetic(&policy, now, 1).unwrap();
    let rollback_token = reserve(&mut engine, &policy, now);
    let rollback_id = rollback_token.entry_id();
    let observation = build_observation(&[ObservedRule {
        burst_hits: 5,
        burst_period_secs: 10,
        sustained_hits: 5,
        sustained_period_secs: 60,
    }]);

    let result = engine.on_probe_response(now, &observation).unwrap();
    assert_eq!(result.synthesized_entries(), 3);
    engine.rollback(rollback_token);

    let history = entries(&engine);
    assert_eq!(history.len(), 4);
    assert!(history.iter().all(|entry| entry.id() != rollback_id));
    assert!(
        history
            .iter()
            .all(|entry| entry.kind() == EntryKind::Synthetic)
    );
}

#[test]
fn ordinary_and_probe_paths_apply_the_same_reconciliation_mechanism() {
    let now = SimInstant::from_millis(NOW_MS);
    let policy = policy_name();
    let observation = build_observation(&[ObservedRule {
        burst_hits: 6,
        burst_period_secs: 10,
        sustained_hits: 8,
        sustained_period_secs: 60,
    }]);
    let mut ordinary = engine();
    let mut probe = engine();
    add_history(&mut ordinary, &[(true, 95_000), (false, 99_000)]);
    add_history(&mut probe, &[(true, 95_000), (false, 99_000)]);
    let ordinary_token = reserve(&mut ordinary, &policy, now);
    let probe_send = reserve(&mut probe, &policy, now);
    probe.on_unknown_outcome(probe_send, now);

    let ordinary_result = ordinary
        .on_response(ordinary_token, now, &observation)
        .unwrap();
    let probe_result = probe.on_probe_response(now, &observation).unwrap();

    assert_eq!(ordinary_result, probe_result);
    let shape = |engine: &PolicyEngine| {
        entries(engine)
            .into_iter()
            .map(|entry| (entry.at(), entry.kind()))
            .collect::<Vec<_>>()
    };
    assert_eq!(shape(&ordinary), shape(&probe));
}

#[test]
fn mismatched_ordinary_observation_consumes_without_rolling_back_the_send() {
    let now = SimInstant::from_millis(NOW_MS);
    let policy = policy_name();
    let mut engine = engine();
    let token = reserve(&mut engine, &policy, now);
    let reserved_id = token.entry_id();
    let observation = build_observation_for(
        "renamed-policy",
        &[ObservedRule {
            burst_hits: 1,
            burst_period_secs: 10,
            sustained_hits: 1,
            sustained_period_secs: 60,
        }],
    );

    let error = engine
        .on_response(token, now, &observation)
        .expect_err("remapping is deliberately deferred");

    assert!(matches!(
        error,
        ObservationError::PolicyMismatch { reserved, observed }
            if reserved == policy && observed == PolicyName::from("renamed-policy")
    ));
    assert!(
        entries(&engine)
            .iter()
            .any(|entry| entry.id() == reserved_id)
    );
}

#[test]
fn unknown_probe_policy_is_typed_and_does_not_mutate_existing_history() {
    let now = SimInstant::from_millis(NOW_MS);
    let mut engine = engine();
    add_history(&mut engine, &[(true, 99_000), (false, 99_500)]);
    let before = entries(&engine);
    let observation = build_observation_for(
        "unknown-policy",
        &[ObservedRule {
            burst_hits: 20,
            burst_period_secs: 10,
            sustained_hits: 20,
            sustained_period_secs: 60,
        }],
    );

    let error = engine
        .on_probe_response(now, &observation)
        .expect_err("policy creation is deliberately deferred");

    assert_eq!(
        error,
        ObservationError::UnknownPolicy(PolicyName::from("unknown-policy"))
    );
    assert_eq!(entries(&engine), before);
}
