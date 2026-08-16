//! M11a near-ceiling compliant sweep — the named binding evidence for
//! `m11-compliant-never-trips` (G2 owns the property; Tom's 2026-08-13
//! amendment).
//!
//! The N23 policies cap the wire far below the D5 floor rate, so a
//! compliant client under them never *approaches* the B10 ceilings and
//! "never trips" would be a claim about traffic that never got close.
//! This target uses B7's scriptable-synthetic-policy channel to
//! configure limits far above floor throughput, so the 250 ms spacing
//! floor is the binding constraint and the actor runs at its compliant
//! maximum: the closest any correct client can get to layer 1. The mock
//! then judges the whole run with G2 armed and reports the peak rolling
//! occupancies, which are pinned exactly — 4 of 20 in the rolling
//! second, 240 of 1,000 in the rolling minute — so the ceiling headroom
//! the charter claims (250 ms floor ⇒ ≤ 240 req/min) is measured, not
//! asserted.
//!
//! Per Tom's 2026-08-14 profile-lane ratification, the synthetic policy
//! is generic rather than OAuth- or legacy-bound, so the sweep runs
//! under *both* bucket profiles instead of arguing invariance: neither
//! window's limit is reachable under either profile's padding, which
//! the identical wire shape across the two runs demonstrates.

use std::collections::BTreeMap;
use std::time::Duration;

use http::Method;
use rate_limit_core::actor::{spawn, with_correlation_header};
use rate_limit_core::conformance::{
    ClientBucketProfile, ContractCoverage, Gate, OAUTH_KNOWN_PROFILE, RunEvidence,
    SHIPPED_ASSUMED_PROFILE, ScenarioAssertion, ScenarioAssertionId, ScenarioId, ScenarioOracle,
    judge,
};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, Resolution};
use rate_limit_core::mock::model::{PolicyDefinition, RuleDefinition, WindowDefinition};
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, MockConfig, MockService, Observation, request,
};

/// D5's dispatch floor, restated as the contract's literal — the oracle and
/// the pacing assertion must not import the actor's constant.
const MIN_SEND_SPACING_MS: u64 = 250;
/// B10's two ceiling rules, restated from `scenarios.md` §7.2 as contract
/// literals rather than read from the mock under test.
const B10_BURST_CEILING: usize = 20;
const B10_SUSTAINED_CEILING: usize = 1_000;
/// The compliant maxima the floor permits: 4 arrivals fit a rolling second
/// at 250 ms spacing, 240 fit a rolling minute. These are the charter's
/// "250 ms ⇒ ≤ 240 req/min" made executable.
const COMPLIANT_BURST_MAXIMUM: usize = 4;
const COMPLIANT_SUSTAINED_MAXIMUM: usize = 240;
// The headroom ordering the charter claims, checked at compile time: the
// compliant maxima sit 5x below the burst ceiling and more than 4x below
// the sustained one.
const _: () = assert!(COMPLIANT_BURST_MAXIMUM * 5 == B10_BURST_CEILING);
const _: () = assert!(COMPLIANT_SUSTAINED_MAXIMUM * 5 > B10_SUSTAINED_CEILING);
/// Enough pressure to saturate a full rolling minute at the floor rate
/// (240) with headroom, plus the boot HEAD: ~75 simulated seconds of
/// maximum-rate wire traffic.
const PRESSURE_REQUESTS: usize = 300;
const RUN_MS: u64 = 90_000;
const ADVANCE_STEP_MS: u64 = 25;

async fn advance(millis: u64) {
    for _ in 0..millis.div_ceil(ADVANCE_STEP_MS) {
        tokio::time::advance(Duration::from_millis(ADVANCE_STEP_MS)).await;
        tokio::task::yield_now().await;
    }
}

/// A synthetic RulePair whose limits sit far above floor throughput in both
/// profiles: at 4/s the burst window holds at most 60 (Known) / 280
/// (Assumed) padded in-window hits against 1,000, the sustained window at
/// most 480 against 10,000. The floor, not the policy, paces the wire.
fn high_throughput_policy() -> PolicyDefinition {
    PolicyDefinition::new(
        "layer1-headroom-limit",
        vec![
            RuleDefinition::new(
                "Account",
                vec![
                    WindowDefinition::new(1_000, 10_000, 60_000, 5_000)
                        .expect("the synthetic burst window is valid"),
                    WindowDefinition::new(10_000, 60_000, 60_000, 60_000)
                        .expect("the synthetic sustained window is valid"),
                ],
            )
            .expect("the synthetic rule is valid"),
        ],
    )
    .expect("the synthetic policy is valid")
}

fn engine(profile: ClientBucketProfile) -> PolicyEngine {
    let resolution = |millis| {
        if profile == SHIPPED_ASSUMED_PROFILE {
            Resolution::Assumed(Duration::from_millis(millis))
        } else {
            Resolution::Known(Duration::from_millis(millis))
        }
    };
    PolicyEngine::new(BucketModel::new(
        resolution(profile.burst_ms),
        resolution(profile.sustained_ms),
    ))
}

/// The wire facts M11a must establish. They reach the judge as the M11
/// scenario assertion (the RE-2 sole-decider pattern); trip judgment itself
/// stays with G2. The falsifiability test below proves each fact can make
/// the verdict false.
#[derive(Debug, Clone, Copy)]
struct M11aFacts {
    all_served: bool,
    dispatch_count: usize,
    /// Compliance premise: consecutive dispatches, HEAD included, never
    /// closer than the floor.
    paced: bool,
    /// Near-ceiling reachability: the run provably reached the compliant
    /// maxima, so "never tripped" is a claim about maximum legal pressure.
    peak_burst_count: usize,
    peak_sustained_count: usize,
}

impl M11aFacts {
    fn passed(self) -> bool {
        self.all_served
            && self.dispatch_count == PRESSURE_REQUESTS + 1
            && self.paced
            && self.peak_burst_count == COMPLIANT_BURST_MAXIMUM
            && self.peak_sustained_count == COMPLIANT_SUSTAINED_MAXIMUM
    }
}

#[test]
fn m11a_verdict_is_not_constant_true() {
    let passing = M11aFacts {
        all_served: true,
        dispatch_count: PRESSURE_REQUESTS + 1,
        paced: true,
        peak_burst_count: COMPLIANT_BURST_MAXIMUM,
        peak_sustained_count: COMPLIANT_SUSTAINED_MAXIMUM,
    };
    assert!(passing.passed());

    for failing in [
        M11aFacts {
            all_served: false,
            ..passing
        },
        M11aFacts {
            dispatch_count: PRESSURE_REQUESTS,
            ..passing
        },
        M11aFacts {
            paced: false,
            ..passing
        },
        M11aFacts {
            peak_burst_count: 3,
            ..passing
        },
        M11aFacts {
            peak_sustained_count: 239,
            ..passing
        },
    ] {
        assert!(
            !failing.passed(),
            "wire fact must reach G5 false: {failing:?}"
        );
    }
    // The maxima are ceilings too: a floor-violating client exceeding them
    // must fail the fact, not read as "even nearer the ceiling".
    assert!(
        !M11aFacts {
            peak_burst_count: 5,
            ..passing
        }
        .passed()
    );
}

struct CeilingOracle {
    eligible_ms: BTreeMap<u64, u64>,
}

impl ScenarioOracle for CeilingOracle {
    fn independently_eligible_ms(&self, observation: &Observation) -> Option<u64> {
        // A missing entry reaches the judge as `None`, which fails G3 closed.
        self.eligible_ms.get(&observation.correlation_id).copied()
    }

    fn independently_observable_ms(
        &self,
        _: &rate_limit_core::mock::MockStateChange,
        _: &[Observation],
    ) -> Option<u64> {
        // No mock-side state change occurs in this scenario.
        None
    }
}

#[tokio::test(start_paused = true)]
async fn m11a_near_ceiling_compliant_sweep_never_trips_either_layer1_ceiling() {
    for (seed, profile) in [
        (1_111, OAUTH_KNOWN_PROFILE),
        (1_112, SHIPPED_ASSUMED_PROFILE),
    ] {
        let config = MockConfig {
            seed,
            // M11 is phase-independent: the layer-1 rules are rolling
            // windows with no bucket phase, and the policy windows never
            // bind. Phase 0 is used and recorded in every observation.
            phase_ms: 0,
            dispatch_budget: rate_limit_core::mock::model::MAX_REQUESTS_PER_RUN,
            policies: vec![high_throughput_policy()],
            routes: vec![(Endpoint::StashList, "layer1-headroom-limit".to_owned())],
        };
        let (mock, controller) = MockService::new(config).unwrap();
        let gate = spawn(engine(profile), mock);

        let mut tickets = Vec::new();
        for _ in 0..PRESSURE_REQUESTS {
            tickets.push(
                gate.submit(
                    EndpointLabel::from(Endpoint::StashList.label()),
                    with_correlation_header(
                        request(Method::GET, Endpoint::StashList, 0)
                            .expect("fixed mock request is valid"),
                        CORRELATION_HEADER,
                    ),
                )
                .await
                .expect("the gate accepts a submission"),
            );
        }
        advance(RUN_MS).await;

        let mut all_served = true;
        for ticket in tickets {
            all_served &= ticket.await.is_ok();
        }

        let observations = controller.observations().await;
        let mut dispatches = observations
            .iter()
            .map(|observation| observation.dispatch_ms)
            .collect::<Vec<_>>();
        dispatches.sort_unstable();
        let paced = dispatches
            .windows(2)
            .all(|pair| pair[1].saturating_sub(pair[0]) >= MIN_SEND_SPACING_MS);
        let facts = M11aFacts {
            all_served,
            dispatch_count: observations.len(),
            paced,
            peak_burst_count: observations
                .iter()
                .map(|observation| observation.layer1.burst_count)
                .max()
                .unwrap_or(0),
            peak_sustained_count: observations
                .iter()
                .map(|observation| observation.layer1.sustained_count)
                .max()
                .unwrap_or(0),
        };

        // Independent eligibility: the boot HEAD at the shared t=0
        // submission instant, every later dispatch at the prior observed
        // dispatch plus the contract floor (the HEAD's 81 ms completion is
        // inside the first floor interval, so probe exclusivity never
        // extends a row).
        let mut eligible_ms = BTreeMap::new();
        let mut prior_dispatch = None;
        for observation in &observations {
            let eligible = match prior_dispatch {
                None => 0,
                Some(prior) => prior + MIN_SEND_SPACING_MS,
            };
            eligible_ms.insert(observation.correlation_id, eligible);
            prior_dispatch = Some(observation.dispatch_ms);
        }
        let evidence = RunEvidence::phase_independent(
            ScenarioId::M11,
            controller.seal_evidence().await.unwrap(),
            None,
            vec![ScenarioAssertion {
                id: ScenarioAssertionId::M11Layer1,
                coverage: ContractCoverage::Fragment,
                passed: facts.passed(),
            }],
        );
        let report = judge(&evidence, &CeilingOracle { eligible_ms }).expect("structural evidence");
        assert!(report.passed(), "profile {profile:?}: {facts:?} {report:?}");
        assert!(
            report.gate(Gate::G2).passed && report.gate(Gate::G1).passed,
            "profile {profile:?}: G2 must hold at maximum compliant pressure"
        );
        assert!(
            observations
                .iter()
                .all(|observation| !observation.layer1.tripped),
            "profile {profile:?}: no arrival may trip a B10 ceiling"
        );
        assert!(
            !report.verdict_eligible(),
            "the M11a sweep runs a fragment and must never be verdict-eligible"
        );
    }
}
