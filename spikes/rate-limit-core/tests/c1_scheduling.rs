use std::time::Duration;

use proptest::prelude::*;
use rate_limit_core::core::{
    BucketModel, Policy, PolicyEngine, PolicyName, ReserveOutcome, Resolution, Rule, RulePair,
    RuleScope, SimInstant, Window,
};

#[derive(Clone, Debug)]
struct RuleCase {
    burst_hits: u32,
    sustained_hits: u32,
    burst_period_ms: u64,
    sustained_period_ms: u64,
    burst_restriction_ms: u64,
    sustained_restriction_ms: u64,
    burst_resolution_ms: u64,
    sustained_resolution_ms: u64,
    burst_assumed: bool,
    sustained_assumed: bool,
    scope: RuleScope,
}

impl RuleCase {
    fn rule(&self) -> Rule {
        let pair = RulePair::new(
            Window::new(
                self.burst_hits,
                Duration::from_millis(self.burst_period_ms),
                Duration::from_millis(self.burst_restriction_ms),
            ),
            Window::new(
                self.sustained_hits,
                Duration::from_millis(self.sustained_period_ms),
                Duration::from_millis(self.sustained_restriction_ms),
            ),
        )
        .expect("generated sustained periods are strictly greater");
        Rule::new(
            self.scope,
            pair,
            BucketModel::new(
                resolution(self.burst_assumed, self.burst_resolution_ms),
                resolution(self.sustained_assumed, self.sustained_resolution_ms),
            ),
        )
    }
}

fn resolution(assumed: bool, millis: u64) -> Resolution {
    let duration = Duration::from_millis(millis);
    if assumed {
        Resolution::Assumed(duration)
    } else {
        Resolution::Known(duration)
    }
}

fn rule_cases() -> impl Strategy<Value = Vec<RuleCase>> {
    prop::collection::vec(
        (
            (
                0_u32..8,
                0_u32..8,
                1_u64..250,
                1_u64..250,
                0_u64..500,
                0_u64..500,
                1_u64..80,
                1_u64..80,
            ),
            (any::<bool>(), any::<bool>(), any::<bool>()),
        )
            .prop_map(
                |(
                    (
                        burst_hits,
                        sustained_hits,
                        burst_period_ms,
                        sustained_delta_ms,
                        burst_restriction_ms,
                        sustained_restriction_ms,
                        burst_resolution_ms,
                        sustained_resolution_ms,
                    ),
                    (burst_assumed, sustained_assumed, ip_scope),
                )| RuleCase {
                    burst_hits,
                    sustained_hits,
                    burst_period_ms,
                    sustained_period_ms: burst_period_ms + sustained_delta_ms,
                    burst_restriction_ms,
                    sustained_restriction_ms,
                    burst_resolution_ms,
                    sustained_resolution_ms,
                    burst_assumed,
                    sustained_assumed,
                    scope: if ip_scope {
                        RuleScope::Ip
                    } else {
                        RuleScope::Account
                    },
                },
            ),
        1..5,
    )
}

fn policy_name() -> PolicyName {
    PolicyName::from("generated-policy")
}

fn build_engine(cases: &[RuleCase], history_ms: &[u64]) -> PolicyEngine {
    let policy = policy_name();
    let mut engine = PolicyEngine::new();
    engine
        .insert_policy(Policy::new(
            policy.clone(),
            cases.iter().map(RuleCase::rule).collect(),
        ))
        .expect("the test inserts one policy");
    for &at in history_ms {
        engine
            .record_synthetic(&policy, SimInstant::from_millis(at), 1)
            .expect("the generated policy exists");
    }
    engine
}

// Independent server oracle: roll each hit forward to the end of the server's
// phase-owned bucket, then apply the rolling-window period. Production never
// sees `phase` and does not use this rollover calculation.
fn server_hit_is_active(
    hit_ms: u64,
    now_ms: u64,
    period_ms: u64,
    resolution_ms: u64,
    phase_ms: u64,
) -> bool {
    if hit_ms > now_ms {
        return false;
    }
    now_ms < server_bucket_end(hit_ms, resolution_ms, phase_ms).saturating_add(period_ms)
}

fn server_bucket_end(hit_ms: u64, resolution_ms: u64, phase_ms: u64) -> u64 {
    let phase_ms = phase_ms % resolution_ms;
    let distance = (phase_ms + resolution_ms - (hit_ms % resolution_ms)) % resolution_ms;
    hit_ms.saturating_add(if distance == 0 {
        resolution_ms
    } else {
        distance
    })
}

fn assert_reserved_is_server_safe(
    engine: &PolicyEngine,
    cases: &[RuleCase],
    now_ms: u64,
    phase_seeds: &[u64; 8],
) -> Result<(), TestCaseError> {
    let policy = engine.policy(&policy_name()).expect("policy exists");
    let history = policy
        .history()
        .entries()
        .map(|entry| entry.at().as_millis())
        .collect::<Vec<_>>();

    for (rule_index, case) in cases.iter().enumerate() {
        for (window_index, (max_hits, period_ms, resolution_ms)) in [
            (
                case.burst_hits,
                case.burst_period_ms,
                case.burst_resolution_ms,
            ),
            (
                case.sustained_hits,
                case.sustained_period_ms,
                case.sustained_resolution_ms,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let phase_seed = phase_seeds[rule_index * 2 + window_index];
            let phase_ms = phase_seed % resolution_ms;
            let server_hits = history
                .iter()
                .filter(|&&hit_ms| {
                    server_hit_is_active(hit_ms, now_ms, period_ms, resolution_ms, phase_ms)
                })
                .count();
            prop_assert!(
                server_hits <= max_hits as usize,
                "reservation exceeded {max_hits} hits for period={period_ms}ms, \
                 resolution={resolution_ms}ms, phase={phase_ms}ms: history={history:?}"
            );
        }
    }
    Ok(())
}

proptest! {
    // C1 / N13: arbitrary shared histories, multi-rule definitions, explicit
    // Known/Assumed resolutions, and independently generated server phases.
    // A Reserved result includes the new send, so <= max_hits proves that send
    // is safe under the server's independently bucketized windows.
    #[test]
    fn every_reserved_outcome_is_safe_for_every_server_phase(
        cases in rule_cases(),
        history_ms in prop::collection::vec(0_u64..1_000, 0..64),
        now_ms in 0_u64..1_000,
        phase_seeds in any::<[u64; 8]>(),
    ) {
        let policy = policy_name();
        let mut engine = build_engine(&cases, &history_ms);

        if let ReserveOutcome::Reserved(token) =
            engine.try_reserve(&policy, SimInstant::from_millis(now_ms))
        {
            assert_reserved_is_server_safe(&engine, &cases, now_ms, &phase_seeds)?;
            engine.rollback(token);
        }
    }
}

// C1: one shared history is judged by every rule/window, and the scheduling
// answer is the maximum padded expiry. This also pins mixed Known/Assumed use.
#[test]
fn not_before_takes_maximum_across_all_saturated_windows_and_rules() {
    let cases = vec![
        RuleCase {
            burst_hits: 1,
            sustained_hits: 10,
            burst_period_ms: 100,
            sustained_period_ms: 1_000,
            burst_restriction_ms: 0,
            sustained_restriction_ms: 0,
            burst_resolution_ms: 10,
            sustained_resolution_ms: 20,
            burst_assumed: false,
            sustained_assumed: true,
            scope: RuleScope::Account,
        },
        RuleCase {
            burst_hits: 10,
            sustained_hits: 1,
            burst_period_ms: 200,
            sustained_period_ms: 300,
            burst_restriction_ms: 0,
            sustained_restriction_ms: 0,
            burst_resolution_ms: 30,
            sustained_resolution_ms: 200,
            burst_assumed: true,
            sustained_assumed: false,
            scope: RuleScope::Ip,
        },
    ];
    let policy = policy_name();
    let mut engine = build_engine(&cases, &[0]);

    assert!(matches!(
        engine.try_reserve(&policy, SimInstant::from_millis(50)),
        ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(500)
    ));
    assert!(matches!(
        engine.try_reserve(&policy, SimInstant::from_millis(499)),
        ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(500)
    ));
    let token = match engine.try_reserve(&policy, SimInstant::from_millis(500)) {
        ReserveOutcome::Reserved(token) => token,
        other => panic!("expected reservation at maximum padded expiry, got {other:?}"),
    };
    engine.rollback(token);
}

// C1: zero headroom means a reservation is granted when exactly one slot is
// left. When history is already over the limit, only enough oldest entries to
// recover that one slot need expire.
#[test]
fn zero_headroom_uses_the_max_hits_th_newest_entry() {
    let cases = vec![RuleCase {
        burst_hits: 3,
        sustained_hits: 10,
        burst_period_ms: 100,
        sustained_period_ms: 1_000,
        burst_restriction_ms: 0,
        sustained_restriction_ms: 0,
        burst_resolution_ms: 5,
        sustained_resolution_ms: 10,
        burst_assumed: false,
        sustained_assumed: true,
        scope: RuleScope::Account,
    }];
    let policy = policy_name();
    let mut engine = build_engine(&cases, &[0, 10, 20, 30]);

    assert!(matches!(
        engine.try_reserve(&policy, SimInstant::from_millis(40)),
        ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(115)
    ));
    let token = match engine.try_reserve(&policy, SimInstant::from_millis(115)) {
        ReserveOutcome::Reserved(token) => token,
        other => panic!("expected the zero-headroom slot to be usable, got {other:?}"),
    };
    engine.rollback(token);
}

// C1 / N13 exact boundary cases: the hit lands just before, exactly on, and
// just after a server bucket rollover. The independent oracle gives different
// actual expiries, while full-bucket client padding safely covers all three.
#[test]
fn rollover_cases_are_safe_just_before_exactly_on_and_just_after() {
    let phase_ms = 50;
    let resolution_ms = 100;
    let period_ms = 100;

    for (hit_ms, expected_server_expiry) in [(249, 350), (250, 450), (251, 450)] {
        let cases = vec![RuleCase {
            burst_hits: 1,
            sustained_hits: 10,
            burst_period_ms: period_ms,
            sustained_period_ms: 1_000,
            burst_restriction_ms: 0,
            sustained_restriction_ms: 0,
            burst_resolution_ms: resolution_ms,
            sustained_resolution_ms: 100,
            burst_assumed: false,
            sustained_assumed: true,
            scope: RuleScope::Account,
        }];
        let policy = policy_name();
        let mut engine = build_engine(&cases, &[hit_ms]);
        let padded_expiry = hit_ms + period_ms + resolution_ms;

        assert_eq!(
            server_bucket_end(hit_ms, resolution_ms, phase_ms) + period_ms,
            expected_server_expiry
        );
        assert!(matches!(
            engine.try_reserve(&policy, SimInstant::from_millis(padded_expiry - 1)),
            ReserveOutcome::NotBefore(at) if at == SimInstant::from_millis(padded_expiry)
        ));
        let token = match engine.try_reserve(&policy, SimInstant::from_millis(padded_expiry)) {
            ReserveOutcome::Reserved(token) => token,
            other => panic!("expected rollover-case reservation, got {other:?}"),
        };
        assert_reserved_is_server_safe(
            &engine,
            &cases,
            padded_expiry,
            &[phase_ms, 0, 0, 0, 0, 0, 0, 0],
        )
        .expect("explicit rollover reservation is server-safe");
        engine.rollback(token);
    }
}
