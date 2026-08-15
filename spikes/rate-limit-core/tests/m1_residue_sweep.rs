//! M1 generated-φ mock-side residue sweep — the exact `m1-g1-sweep`
//! Partial delta after RE-9.
//!
//! RE-9 established that C1's generated-φ property is the *core-side*
//! mirror of the §3 sweep and never judges a mock-side boot-residue
//! run; the driver's residue arm covers residues 0/1/9/10 only at the
//! two boundary-distance phases. This target closes that delta: every
//! generated (residue, φ) case boots the public actor against the
//! mock, and the mock's independent counters judge G1 through
//! `conformance::judge`. Every case asserts — there is no skip path —
//! and the sustained-window residue count is the per-case non-vacuity
//! anchor proving the mock judged against live residue.
//!
//! Rust note (this spike doubles as a Rust introduction): proptest's
//! `proptest!` macro generates a synchronous test body, so each case
//! builds its own current-thread paused-time runtime and `block_on`s
//! the scenario. `#[tokio::test]` cannot be combined with `proptest!`
//! directly; this is the standard composition.

use std::collections::BTreeMap;
use std::time::Duration;

use http::Method;
use proptest::prelude::*;
use rate_limit_core::actor::{spawn, with_correlation_header};
use rate_limit_core::conformance::{
    ContractCoverage, Gate, OAUTH_KNOWN_PROFILE, ReproductionRecord, RunEvidence,
    ScenarioAssertion, ScenarioAssertionId, ScenarioId, ScenarioOracle, judge,
};
use rate_limit_core::core::{BucketModel, EndpointLabel, PolicyEngine, Resolution};
use rate_limit_core::mock::{
    CORRELATION_HEADER, Endpoint, MockConfig, MockService, Observation, request,
};

/// D5's dispatch floor, restated as the contract's literal — the oracle must
/// not import the actor's constant.
const MIN_SEND_SPACING_MS: u64 = 250;
/// The stash-list burst window from the N23 definition (10 hits / 15 s),
/// independently declared: the oracle must not read it from the client.
const M1_BURST_LIMIT: u32 = 10;
const M1_BURST_PERIOD_MS: u64 = 15_000;
/// The Known burst bucket for the OAuth lane (N12).
const BURST_BUCKET_MS: u64 = 5_000;
/// M1's zero-budget wait: residue hits stay active for the burst period plus
/// its full Known bucket (N13). Derived, not a bare 20,000.
const M1_ZERO_BUDGET_WAIT_MS: u64 = M1_BURST_PERIOD_MS + BURST_BUCKET_MS;
/// The stash-list sustained window admits 30 hits / 60 s; the residue sweep
/// is capped strictly below it so the zero-budget wait is always the burst
/// window's 20 s bound (a residue ≥ 30 would demand the 120 s sustained
/// bound instead — a deliberate scope cap, recorded in the hand-off).
const MAX_SWEPT_RESIDUE: u32 = 12;
/// Every configured 5 s / 60 s bucket divides the 60,000 ms cycle, so this
/// range generates every distinct server alignment (same rationale as the
/// canonical replay's exhaustive sweep).
const PHASE_CYCLE_MS: u64 = 60_000;

/// Fine steps must stay well below G3's 500 ms ε so observed dispatch
/// lateness is real, not step-quantization.
const FINE_STEP_MS: u64 = 25;
/// Coarse steps cross the zero-budget wait quickly; they are safe only while
/// no dispatch can occur, which the case asserts before and after.
const COARSE_STEP_MS: u64 = 500;

async fn advance_steps(millis: u64, step_ms: u64) {
    for _ in 0..millis.div_ceil(step_ms) {
        tokio::time::advance(Duration::from_millis(step_ms)).await;
        tokio::task::yield_now().await;
    }
}

/// The wire facts one generated case must establish. They reach the judge as
/// the M1 scenario assertion (the sole decider — the RE-2 pattern), and the
/// falsifiability test below proves each one can make the verdict false.
#[derive(Debug, Clone, Copy)]
struct M1SweepFacts {
    served: bool,
    observation_count: usize,
    head_counted: bool,
    /// Non-vacuity anchor: the sustained window (60 s period, ≥ 60 s
    /// mock-side expiry at every φ) still holds every preloaded residue hit
    /// when the opening GET arrives, in both branches.
    sustained_residue_hits: u32,
    expected_residue: u32,
    zero_budget: bool,
    get_dispatch_ms: u64,
    zero_budget_bound_ms: u64,
}

impl M1SweepFacts {
    fn passed(self) -> bool {
        self.served
            && self.observation_count == 2
            && !self.head_counted
            && self.sustained_residue_hits == self.expected_residue
            && (!self.zero_budget || self.get_dispatch_ms >= self.zero_budget_bound_ms)
    }
}

#[test]
fn m1_sweep_verdict_is_not_constant_true() {
    let passing = M1SweepFacts {
        served: true,
        observation_count: 2,
        head_counted: false,
        sustained_residue_hits: 10,
        expected_residue: 10,
        zero_budget: true,
        get_dispatch_ms: 20_081,
        zero_budget_bound_ms: 20_081,
    };
    assert!(passing.passed());

    for failing in [
        M1SweepFacts {
            served: false,
            ..passing
        },
        M1SweepFacts {
            observation_count: 3,
            ..passing
        },
        M1SweepFacts {
            head_counted: true,
            ..passing
        },
        M1SweepFacts {
            sustained_residue_hits: 9,
            ..passing
        },
        M1SweepFacts {
            get_dispatch_ms: 20_080,
            ..passing
        },
    ] {
        assert!(
            !failing.passed(),
            "wire fact must reach G5 false: {failing:?}"
        );
    }

    // The below-budget branch keeps its own teeth through the residue anchor
    // and observation count; the wait bound is deliberately inert there.
    let below_budget = M1SweepFacts {
        zero_budget: false,
        get_dispatch_ms: 331,
        sustained_residue_hits: 4,
        expected_residue: 4,
        ..passing
    };
    assert!(below_budget.passed());
    assert!(
        !M1SweepFacts {
            sustained_residue_hits: 5,
            ..below_budget
        }
        .passed()
    );
}

struct SweepOracle {
    eligible_ms: BTreeMap<u64, u64>,
}

impl ScenarioOracle for SweepOracle {
    fn independently_eligible_ms(&self, observation: &Observation) -> Option<u64> {
        // A missing entry reaches the judge as `None`, which fails G3 closed.
        self.eligible_ms.get(&observation.correlation_id).copied()
    }

    fn independently_observable_ms(
        &self,
        _: &rate_limit_core::mock::MockStateChange,
        _: &[Observation],
    ) -> Option<u64> {
        // No mock-side state change occurs in this scenario; the judge only
        // consults this for an exposure allowance, which the sweep never
        // claims.
        None
    }
}

fn run_case(residue: u32, phase_ms: u64) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .expect("current-thread paused runtime builds");
    runtime.block_on(run_case_paused(residue, phase_ms));
}

async fn run_case_paused(residue: u32, phase_ms: u64) {
    // The seed is derived from the generated inputs so the G6 reproduction
    // record identifies the case; proptest's own persistence carries the
    // generator state.
    let seed = 41_000 + u64::from(residue) * 100_000 + phase_ms;
    let (mock, controller) = MockService::new(MockConfig::n23(seed, phase_ms))
        .expect("the fixed N23 mock configuration is valid");
    if residue > 0 {
        controller
            .preload(
                "stash-list-request-limit",
                controller.now(),
                residue as usize,
            )
            .await
            .expect("preloading the swept residue succeeds");
    }
    // OAuth endpoint → Known(5s/60s) profile (the SD-R5-F1 rule; Tom's
    // 2026-08-14 profile-lane ratification).
    let engine = PolicyEngine::new(BucketModel::new(
        Resolution::Known(Duration::from_millis(OAUTH_KNOWN_PROFILE.burst_ms)),
        Resolution::Known(Duration::from_millis(OAUTH_KNOWN_PROFILE.sustained_ms)),
    ));
    let gate = spawn(engine, mock);
    let ticket = gate
        .submit(
            EndpointLabel::from(Endpoint::StashList.label()),
            with_correlation_header(
                request(Method::GET, Endpoint::StashList, 0).expect("fixed request is valid"),
                CORRELATION_HEADER,
            ),
        )
        .await
        .expect("the gate accepts the boot submission");

    let zero_budget = residue >= M1_BURST_LIMIT;
    advance_steps(600, FINE_STEP_MS).await;
    if zero_budget {
        // Branch reachability: with zero remaining budget the opening GET
        // must not have dispatched yet — only the boot HEAD is on the wire.
        assert_eq!(
            controller.observations().await.len(),
            1,
            "residue={residue} phase={phase_ms}: the zero-budget GET dispatched early"
        );
        // The actor sleeps on its NotBefore (~20 s) and the mock holds no
        // pending exchange, so coarse steps cannot delay any timer; the
        // post-advance observation count proves it.
        advance_steps(19_000, COARSE_STEP_MS).await;
        assert_eq!(
            controller.observations().await.len(),
            1,
            "residue={residue} phase={phase_ms}: a dispatch occurred inside the coarse advance"
        );
        advance_steps(1_600, FINE_STEP_MS).await;
    }
    let served = ticket.await.is_ok();

    let observations = controller.observations().await;
    let head = observations
        .iter()
        .find(|observation| observation.method == Method::HEAD)
        .expect("M1 boot HEAD");
    let get = observations
        .iter()
        .find(|observation| observation.method == Method::GET)
        .expect("M1 opening GET");
    // The N23 stash-list rule orders its windows [burst, sustained].
    let sustained = &get.policy_judgment.windows[1];

    let facts = M1SweepFacts {
        served,
        observation_count: observations.len(),
        head_counted: head.policy_judgment.counted,
        sustained_residue_hits: sustained.residue_hits,
        expected_residue: residue,
        zero_budget,
        get_dispatch_ms: get.dispatch_ms,
        zero_budget_bound_ms: head.completion_ms.saturating_add(M1_ZERO_BUDGET_WAIT_MS),
    };

    // Independent contract arithmetic: below the limit only D5's floor
    // applies; at or above it the residue hits stay active for the burst
    // period plus its full Known bucket from the HEAD's completed
    // observation (N13).
    let eligible_ms = BTreeMap::from([
        (head.correlation_id, 0),
        (
            get.correlation_id,
            if zero_budget {
                facts.zero_budget_bound_ms
            } else {
                MIN_SEND_SPACING_MS
            },
        ),
    ]);
    let evidence = RunEvidence {
        scenario: ScenarioId::M1,
        reproduction: Some(ReproductionRecord {
            seed,
            phase_ms,
            client_buckets: OAUTH_KNOWN_PROFILE,
            endpoint: Endpoint::StashList,
        }),
        observations: observations.clone(),
        state_changes: controller.state_changes().await,
        unavoidable_exposure: None,
        assertions: vec![ScenarioAssertion {
            id: ScenarioAssertionId::M1BootSequence,
            coverage: ContractCoverage::Fragment,
            passed: facts.passed(),
        }],
    };
    let report = judge(&evidence, &SweepOracle { eligible_ms }).expect("structural evidence");
    assert!(
        report.passed(),
        "residue={residue} phase={phase_ms}: {facts:?} {report:?}"
    );
    assert!(report.gate(Gate::G1).passed && report.gate(Gate::G2).passed);
    assert!(
        !report.verdict_eligible(),
        "the generated sweep runs fragments and must never be verdict-eligible"
    );
}

/// Generated φ over the full 60,000 ms cycle, with §3's three explicit
/// rollover cases (just before, exactly on, just after a burst-bucket
/// boundary) pinned as always-reachable strategy arms.
fn phase_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        6 => 0..PHASE_CYCLE_MS,
        1 => Just(BURST_BUCKET_MS - 1),
        1 => Just(BURST_BUCKET_MS),
        1 => Just(BURST_BUCKET_MS + 1),
    ]
}

proptest! {
    /// The mock-side §3 sweep M1 owns: for generated residue magnitude and
    /// server phase, the boot HEAD's state header is the only thing between
    /// the opening GET and a first-request violation, and the mock's
    /// independent counters judge the run green with G1 armed.
    #[test]
    fn m1_generated_phi_mock_side_residue_sweep_holds_g1(
        residue in 0..=MAX_SWEPT_RESIDUE,
        phase_ms in phase_strategy(),
    ) {
        run_case(residue, phase_ms);
    }
}
