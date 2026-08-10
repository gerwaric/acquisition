use std::time::Duration;

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use proptest::prelude::*;
use rate_limit_core::core::{
    BucketModel, Disposition, EndpointLabel, EntryKind, HistoryEntry, ObservationError,
    ObservedResponse, Policy, PolicyEngine, PolicyName, RefusalCause, RefusalTarget,
    ReplyClassification, ReserveOutcome, Resolution, Rule, RulePair, RuleScope, SimInstant, Window,
};
use rate_limit_core::header::{PolicySnapshot, parse_policy};

const NOW_MS: u64 = 100_000;

// Shared by the configured policy and the pessimism oracle: reconciliation
// promises local count >= min(reported, largest configured max_hits).
const CONFIGURED_MAX_HITS: u32 = 512;

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
        Window::new(
            CONFIGURED_MAX_HITS,
            Duration::from_secs(1_000),
            Duration::ZERO,
        ),
        Window::new(
            CONFIGURED_MAX_HITS,
            Duration::from_secs(2_000),
            Duration::ZERO,
        ),
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
        .insert_policy(
            Policy::new(policy_name(), vec![configured_rule()])
                .expect("the test policy has one rule"),
        )
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

/// Seeds history and returns the test's own record of every timestamp it
/// injected — the shadow list the independent oracle computes from.
fn add_history(engine: &mut PolicyEngine, entries: &[(bool, u64)]) -> Vec<u64> {
    let policy = policy_name();
    for &(local, at_ms) in entries {
        let at = SimInstant::from_millis(at_ms);
        if local {
            let token = reserve(engine, &policy, at);
            let _ = engine.on_unknown_outcome(token, at);
        } else {
            engine
                .record_synthetic(&policy, at, 1)
                .expect("policy exists");
        }
    }
    entries.iter().map(|&(_, at_ms)| at_ms).collect()
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
        // Limit hits clamp into the D8 wire ceiling; state hits are the
        // unbounded field under test.
        let limit = format!(
            "{}:{}:0, {}:{}:0",
            rule.burst_hits.clamp(100, 10_000),
            rule.burst_period_secs,
            rule.sustained_hits.clamp(100, 10_000),
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

fn response(observation: &PolicySnapshot) -> ObservedResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-rate-limit-policy",
        HeaderValue::try_from(observation.name.as_str()).expect("valid policy name"),
    );
    headers.insert(
        "x-rate-limit-rules",
        HeaderValue::try_from(
            observation
                .rules
                .iter()
                .map(|rule| rule.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )
        .expect("valid rule names"),
    );
    for rule in &observation.rules {
        let limit_name = HeaderName::try_from(format!("x-rate-limit-{}", rule.name)).unwrap();
        let state_name = HeaderName::try_from(format!("x-rate-limit-{}-state", rule.name)).unwrap();
        let limit = format!(
            "{}:{}:{}, {}:{}:{}",
            rule.pair.burst().max_hits,
            rule.pair.burst().period.as_secs(),
            rule.pair.burst().restriction.as_secs(),
            rule.pair.sustained().max_hits,
            rule.pair.sustained().period.as_secs(),
            rule.pair.sustained().restriction.as_secs(),
        );
        let state = format!(
            "{}:{}:{}, {}:{}:{}",
            rule.state.burst.current_hits,
            rule.state.burst.period.as_secs(),
            rule.state.burst.restriction_active.as_secs(),
            rule.state.sustained.current_hits,
            rule.state.sustained.period.as_secs(),
            rule.state.sustained.restriction_active.as_secs(),
        );
        headers.insert(limit_name, HeaderValue::try_from(limit).unwrap());
        headers.insert(state_name, HeaderValue::try_from(state).unwrap());
    }
    ObservedResponse::new(StatusCode::OK, headers, ReplyClassification::Normal)
}

fn probe(
    engine: &mut PolicyEngine,
    now: SimInstant,
    observation: &PolicySnapshot,
) -> rate_limit_core::core::Transition {
    engine.on_probe_response(&EndpointLabel::from("stash"), now, &response(observation))
}

/// Independent oracle (audit, 2026-08-09: the previous version re-derived
/// the deficit through production's own `count_within`, mirror-testing the
/// "exactly" half): plain u64 arithmetic over the test's shadow timestamp
/// list. An entry occupies the half-open window `(now - period, now]`, and
/// future-dated entries are retained — `at + period > now` states both.
fn expected_maximum_deficit(shadow_ms: &[u64], now_ms: u64, observation: &PolicySnapshot) -> usize {
    observation
        .rules
        .iter()
        .flat_map(|rule| [&rule.state.burst, &rule.state.sustained])
        .map(|state| {
            let period_ms = u64::try_from(state.period.as_millis()).expect("test periods fit u64");
            let local = shadow_ms
                .iter()
                .filter(|&&at_ms| at_ms + period_ms > now_ms)
                .count();
            let reported = state.current_hits.min(CONFIGURED_MAX_HITS) as usize;
            reported.saturating_sub(local)
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
    // Entry timestamps are read as raw data; the windowing arithmetic is the
    // test's own (external review: production count_within must not be the
    // instrument that certifies production pessimism).
    let entry_times = engine
        .policy(&policy_name())
        .expect("policy exists")
        .history()
        .entries()
        .map(|entry| entry.at.as_millis())
        .collect::<Vec<_>>();
    let now_ms = now.as_millis();
    for state in observation
        .rules
        .iter()
        .flat_map(|rule| [&rule.state.burst, &rule.state.sustained])
    {
        let period_ms = u64::try_from(state.period.as_millis()).expect("test periods fit u64");
        let local = entry_times
            .iter()
            .filter(|&&at_ms| at_ms + period_ms > now_ms)
            .count();
        prop_assert!(
            local >= (state.current_hits.min(CONFIGURED_MAX_HITS)) as usize,
            "local count remained below the capped reported post-increment count"
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
        let mut shadow_ms = add_history(&mut engine, &initial);
        let token = reserve(&mut engine, &policy, now);
        shadow_ms.push(NOW_MS);
        let observation = build_observation(&rules);
        let before = entries(&engine);
        let expected = expected_maximum_deficit(&shadow_ms, NOW_MS, &observation);

        let result = engine.on_response(token, now, &response(&observation));

        prop_assert_eq!(result.disposition, Disposition::CompleteRequest);
        let after = entries(&engine);
        prop_assert_eq!(after.len() - before.len(), expected);
        prop_assert_eq!(&after[..before.len()], before.as_slice());
        prop_assert_eq!(after.len(), before.len() + expected);
        for entry in &after[before.len()..] {
            prop_assert_eq!(entry.at, now);
            prop_assert_eq!(entry.kind, EntryKind::Synthetic);
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
        let shadow_ms = add_history(&mut engine, &initial);
        let observation = build_observation(&rules);
        let before = entries(&engine);
        let expected = expected_maximum_deficit(&shadow_ms, NOW_MS, &observation);

        let first = probe(&mut engine, now, &observation);
        let after_first = entries(&engine);
        prop_assert_eq!(first.disposition, Disposition::ProbeReady);
        prop_assert_eq!(after_first.len() - before.len(), expected);
        prop_assert_eq!(&after_first[..before.len()], before.as_slice());
        assert_pessimistic(&engine, now, &observation)?;

        let repeated = probe(&mut engine, now, &observation);
        prop_assert_eq!(repeated.disposition, Disposition::ProbeReady);
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
        let lower_result = probe(&mut engine, now, &lower);
        prop_assert_eq!(lower_result.disposition, Disposition::ProbeReady);
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

    let before = entries(&engine).len();
    let result = probe(&mut engine, now, &observation);

    assert_eq!(result.disposition, Disposition::ProbeReady);
    assert_eq!(entries(&engine).len() - before, 7);
    assert_eq!(entries(&engine).len(), 7);
    assert!(
        entries(&engine)
            .iter()
            .all(|entry| entry.kind == EntryKind::Synthetic && entry.at == now)
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

    let result = engine.on_response(token, now, &response(&observation));

    assert_eq!(result.disposition, Disposition::CompleteRequest);
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

    let before = entries(&engine).len();
    let result = engine.on_response(token, now, &response(&observation));

    assert_eq!(result.disposition, Disposition::CompleteRequest);
    assert_eq!(entries(&engine).len() - before, 3);
    assert_eq!(entries(&engine).len(), 6);
}

// A wire-controlled current-hits value cannot force unbounded synthesis:
// the deficit targets min(reported, largest configured max_hits). Beyond
// that bound every configured window is already saturated at `now`, so
// further entries would move no scheduling deadline.
#[test]
fn reported_hits_beyond_the_largest_configured_limit_are_capped() {
    let now = SimInstant::from_millis(NOW_MS);
    let mut engine = engine(); // configured max_hits: 512 on both windows
    let observation = build_observation(&[ObservedRule {
        burst_hits: 4_000_000,
        burst_period_secs: 10,
        sustained_hits: 4_000_000_000,
        sustained_period_secs: 60,
    }]);

    let result = probe(&mut engine, now, &observation);

    assert_eq!(result.disposition, Disposition::ProbeReady);
    assert_eq!(entries(&engine).len(), 512);

    // A repeat of the same over-limit report finds every window saturated
    // up to the cap and synthesizes nothing further.
    let repeated = probe(&mut engine, now, &observation);
    assert_eq!(repeated.disposition, Disposition::ProbeReady);
    assert_eq!(entries(&engine).len(), 512);
}

// The cap boundary itself, pinned at n-1 / n / n+1 (external review: a test
// far above the cap does not pin where the cap sits).
#[test]
fn synthesis_cap_boundary_is_exact() {
    for (reported, expected) in [(511_u32, 511_usize), (512, 512), (513, 512)] {
        let now = SimInstant::from_millis(NOW_MS);
        let mut engine = engine(); // configured max_hits: 512
        let observation = build_observation(&[ObservedRule {
            burst_hits: reported,
            burst_period_secs: 10,
            sustained_hits: reported,
            sustained_period_secs: 60,
        }]);

        let result = probe(&mut engine, now, &observation);

        assert_eq!(result.disposition, Disposition::ProbeReady);
        assert_eq!(
            entries(&engine).len(),
            expected,
            "reported {reported} should synthesize {expected}"
        );
    }
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
    let before = entries(&engine).len();
    let result = probe(&mut engine, now, &observation);

    assert_eq!(result.disposition, Disposition::ProbeReady);
    assert_eq!(entries(&engine).len() - before, 4);
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

    let before = entries(&engine).len();
    let result = probe(&mut engine, now, &observation);
    assert_eq!(result.disposition, Disposition::ProbeReady);
    assert_eq!(entries(&engine).len() - before, 3);
    engine.rollback(rollback_token);

    let history = entries(&engine);
    assert_eq!(history.len(), 4);
    assert!(history.iter().all(|entry| entry.id != rollback_id));
    assert!(
        history
            .iter()
            .all(|entry| entry.kind == EntryKind::Synthetic)
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
    let _ = probe.on_unknown_outcome(probe_send, now);

    let ordinary_before = entries(&ordinary).len();
    let probe_before = entries(&probe).len();
    let ordinary_result = ordinary.on_response(ordinary_token, now, &response(&observation));
    let probe_result = self::probe(&mut probe, now, &observation);

    assert_eq!(ordinary_result.disposition, Disposition::CompleteRequest);
    assert_eq!(probe_result.disposition, Disposition::ProbeReady);
    assert_eq!(entries(&ordinary).len() - ordinary_before, 5);
    assert_eq!(entries(&probe).len() - probe_before, 5);
    let shape = |engine: &PolicyEngine| {
        entries(engine)
            .into_iter()
            .map(|entry| (entry.at, entry.kind))
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

    let transition = engine.on_response(token, now, &response(&observation));

    assert!(matches!(
        &transition.disposition,
        Disposition::Refuse {
            target: RefusalTarget::Policy(target),
            cause: RefusalCause::ObservationTarget(ObservationError::PolicyMismatch {
                reserved,
                observed,
            }),
        } if target == &policy
            && reserved == &policy
            && observed == &PolicyName::from("renamed-policy")
    ));
    assert!(entries(&engine).iter().any(|entry| entry.id == reserved_id));
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

    let transition = probe(&mut engine, now, &observation);

    assert!(matches!(
        &transition.disposition,
        Disposition::Refuse {
            target: RefusalTarget::Endpoint(endpoint),
            cause: RefusalCause::ObservationTarget(ObservationError::UnknownPolicy(policy)),
        } if endpoint == &EndpointLabel::from("stash")
            && policy == &PolicyName::from("unknown-policy")
    ));
    assert_eq!(entries(&engine), before);
}
