//! M-series scenario metadata and client-independent gate judgment.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::mock::model::MAX_REQUESTS_PER_RUN;
use crate::mock::{Endpoint, MockStateChange, Observation};

pub const G3_EPSILON_MS: u64 = 500;
pub const MAX_SWEEP_CONFIGURATIONS: usize = 256;
/// The judge's restatement of D5's in-flight cap, deliberately independent of
/// `actor::D5_IN_FLIGHT_CAP` (the judge must not read the value it verifies
/// from the code under test). This copy also caps the G1 unavoidable-exposure
/// allowance, so unnoticed drift would widen forgiveness —
/// `d5_in_flight_cap_restatements_agree` trips on any divergence so the pair
/// only ever moves together, deliberately (SD-R5-F14).
pub const D5_IN_FLIGHT_CAP: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScenarioId {
    M1,
    M2,
    M3,
    M4,
    M5,
    M6,
    M7,
    M8,
    M9,
    M10,
    M11,
    M12,
    M13,
}

impl ScenarioId {
    pub const ALL: [Self; 13] = [
        Self::M1,
        Self::M2,
        Self::M3,
        Self::M4,
        Self::M5,
        Self::M6,
        Self::M7,
        Self::M8,
        Self::M9,
        Self::M10,
        Self::M11,
        Self::M12,
        Self::M13,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepKind {
    PhaseSwept,
    PhaseIndependent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Gate {
    G1,
    G2,
    G3,
    G4,
    G5,
    G6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScenarioSpec {
    pub id: ScenarioId,
    pub name: &'static str,
    pub sweep: SweepKind,
    pub binding_gates: &'static [Gate],
    pub required_assertion: ScenarioAssertionId,
}

/// One scenario-owned assertion represents the complete scenario contract
/// from `scenarios.md`; a run cannot substitute an unrelated passing flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ScenarioAssertionId {
    M1BootSequence,
    M2Saturation,
    M3DegradedProbe,
    M4UnexpectedShape,
    M5Rename,
    M6Shrink,
    M7Phantoms,
    M8Recovery,
    M9PhantomRace,
    M10Stress,
    M11Layer1,
    M12Tripwire,
    M13GateStructure,
}

const M1_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G6];
const M2_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G3, Gate::G4, Gate::G6];
const M3_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G5];
const M4_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G5];
const M5_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G6];
const M6_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G6];
const M7_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G6];
const M8_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G5, Gate::G6];
const M9_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G5, Gate::G6];
const M10_GATES: &[Gate] = &[Gate::G1, Gate::G2, Gate::G3, Gate::G6];
const M11_GATES: &[Gate] = &[Gate::G2, Gate::G5];
const M12_GATES: &[Gate] = &[Gate::G5];
const M13_GATES: &[Gate] = &[Gate::G2];

pub const SCENARIOS: [ScenarioSpec; 13] = [
    ScenarioSpec {
        id: ScenarioId::M1,
        name: "cold start with residue",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M1_GATES,
        required_assertion: ScenarioAssertionId::M1BootSequence,
    },
    ScenarioSpec {
        id: ScenarioId::M2,
        name: "clean cold-start saturation burst",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M2_GATES,
        required_assertion: ScenarioAssertionId::M2Saturation,
    },
    ScenarioSpec {
        id: ScenarioId::M3,
        name: "degraded HEAD",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M3_GATES,
        required_assertion: ScenarioAssertionId::M3DegradedProbe,
    },
    ScenarioSpec {
        id: ScenarioId::M4,
        name: "unexpected policy shape",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M4_GATES,
        required_assertion: ScenarioAssertionId::M4UnexpectedShape,
    },
    ScenarioSpec {
        id: ScenarioId::M5,
        name: "policy rename mid-session",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M5_GATES,
        required_assertion: ScenarioAssertionId::M5Rename,
    },
    ScenarioSpec {
        id: ScenarioId::M6,
        name: "policy shrink mid-flight",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M6_GATES,
        required_assertion: ScenarioAssertionId::M6Shrink,
    },
    ScenarioSpec {
        id: ScenarioId::M7,
        name: "phantom same-account hits",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M7_GATES,
        required_assertion: ScenarioAssertionId::M7Phantoms,
    },
    ScenarioSpec {
        id: ScenarioId::M8,
        name: "429 recovery and escalation ladder",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M8_GATES,
        required_assertion: ScenarioAssertionId::M8Recovery,
    },
    ScenarioSpec {
        id: ScenarioId::M9,
        name: "phantom race at saturation",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M9_GATES,
        required_assertion: ScenarioAssertionId::M9PhantomRace,
    },
    ScenarioSpec {
        id: ScenarioId::M10,
        name: "agent-loop stress",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M10_GATES,
        required_assertion: ScenarioAssertionId::M10Stress,
    },
    ScenarioSpec {
        id: ScenarioId::M11,
        name: "layer-1 ceiling and Cloudflare terminal",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M11_GATES,
        required_assertion: ScenarioAssertionId::M11Layer1,
    },
    ScenarioSpec {
        id: ScenarioId::M12,
        name: "4xx-tripwire obligations",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M12_GATES,
        required_assertion: ScenarioAssertionId::M12Tripwire,
    },
    ScenarioSpec {
        id: ScenarioId::M13,
        name: "gate structure on the wire",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M13_GATES,
        required_assertion: ScenarioAssertionId::M13GateStructure,
    },
];

pub fn scenario(id: ScenarioId) -> &'static ScenarioSpec {
    &SCENARIOS[id as usize]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionEvidence {
    Known,
    Assumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientBucketProfile {
    pub burst_ms: u64,
    pub sustained_ms: u64,
    pub evidence: ResolutionEvidence,
}

pub const SHIPPED_ASSUMED_PROFILE: ClientBucketProfile = ClientBucketProfile {
    burst_ms: 60_000,
    sustained_ms: 60_000,
    evidence: ResolutionEvidence::Assumed,
};

pub const OAUTH_KNOWN_PROFILE: ClientBucketProfile = ClientBucketProfile {
    burst_ms: 5_000,
    sustained_ms: 60_000,
    evidence: ResolutionEvidence::Known,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SweepConfiguration {
    pub seed: u64,
    pub phase_ms: u64,
    pub client_buckets: ClientBucketProfile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SweepPlanError {
    Empty,
    TooMany { limit: usize },
    MissingShippedAssumedDefault,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SweepPlan {
    configurations: Vec<SweepConfiguration>,
}

impl SweepPlan {
    pub fn new(configurations: Vec<SweepConfiguration>) -> Result<Self, SweepPlanError> {
        if configurations.is_empty() {
            return Err(SweepPlanError::Empty);
        }
        if configurations.len() > MAX_SWEEP_CONFIGURATIONS {
            return Err(SweepPlanError::TooMany {
                limit: MAX_SWEEP_CONFIGURATIONS,
            });
        }
        if !configurations
            .iter()
            .any(|configuration| configuration.client_buckets == SHIPPED_ASSUMED_PROFILE)
        {
            return Err(SweepPlanError::MissingShippedAssumedDefault);
        }
        Ok(Self { configurations })
    }

    pub fn configurations(&self) -> &[SweepConfiguration] {
        &self.configurations
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReproductionRecord {
    pub seed: u64,
    pub phase_ms: u64,
    pub client_buckets: ClientBucketProfile,
    /// The lane's routed endpoint — run-owned provenance, like
    /// `client_buckets`. The full-contract declaration keys its
    /// policy-coverage requirement to this field (SD-R8-F5): the verdict
    /// claims every N23 policy, so the declaration must see every endpoint.
    pub endpoint: Endpoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorizedExclusionKind {
    Probe,
    RecoveryEpisode,
    Cooldown,
    Suspension,
    Halt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizedExclusion {
    pub kind: AuthorizedExclusionKind,
    pub begins_ms: u64,
    pub ends_ms: u64,
}

/// Scenario-owned arithmetic independent of the actor under test.
///
/// The actor contributes no timing values to `RunEvidence`: G3 uses the
/// mock's hand-off record and asks this oracle only for the separately
/// computed safe time. The scenario fixture also owns every authorized
/// exclusion and M2's padded-minimum calculation.
pub trait ScenarioOracle {
    /// The independently derived instant at which this observation's dispatch
    /// became eligible, or `None` when the oracle has no entry for it.
    ///
    /// `None` fails closed *in the judge*: the dispatch is scored as a G3
    /// failure ("no independent eligibility"), never as trivially eligible.
    /// Implementations must return `None` for an unknown observation rather
    /// than substituting the dispatch under test — that fallback would
    /// manufacture zero lateness (round-four F16) — and must not encode the
    /// convention themselves with a sentinel (round-five SD-R5-F6: the
    /// previous `u64::MAX` sentinel lived in each implementation, where a
    /// one-token slip to `unwrap_or_default()` would have disarmed G3 with
    /// nothing failing).
    fn independently_eligible_ms(&self, observation: &Observation) -> Option<u64>;

    /// Returns when this particular mock-side change first becomes visible to
    /// the client in this scenario. The implementation must derive this from
    /// the scenario script and B13 evidence, never from actor state.
    fn independently_observable_ms(
        &self,
        state_change: &MockStateChange,
        observations: &[Observation],
    ) -> Option<u64>;

    fn authorizes_delay(&self, _begins_ms: u64, _ends_ms: u64) -> bool {
        false
    }

    fn m2_theoretical_padded_minimum_ms(&self, _observations: &[Observation]) -> Option<u64> {
        None
    }
}

/// Whether a run's assertion covers the whole `scenarios.md` contract for its
/// ID, or only the fragment a partial driver implements.
///
/// A fragment run is still judged — G5 fails if the fragment fails — but the
/// report carries the distinction so a passing G5 cannot be read as
/// contract-complete evidence. Without this, a driver that exercises one
/// branch of M8 reports `M8Recovery: passed`, which is exactly the
/// "unrelated passing flag" [`ScenarioAssertionId`] forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractCoverage {
    FullContract,
    Fragment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioAssertion {
    pub id: ScenarioAssertionId,
    pub coverage: ContractCoverage,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExposureAllowance {
    state_change_id: u64,
    reservations: BTreeMap<u64, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExposureError {
    AboveD5InFlightCap,
    TooManyPreObservableReservations { maximum: usize },
    DuplicateCorrelation { id: u64 },
}

impl ExposureAllowance {
    /// Binds an unavoidable-exposure claim to one mock-owned state change.
    /// The judge, not the evidence producer, obtains that change's
    /// independently derived observable instant from the scenario oracle.
    pub fn for_state_change(
        state_change_id: u64,
        reservations: impl IntoIterator<Item = (u64, u64)>,
        in_flight_cap: usize,
    ) -> Result<Self, ExposureError> {
        if in_flight_cap > D5_IN_FLIGHT_CAP {
            return Err(ExposureError::AboveD5InFlightCap);
        }
        let mut pre_observable = BTreeMap::new();
        for (correlation, reserved_at) in reservations {
            if pre_observable.insert(correlation, reserved_at).is_some() {
                return Err(ExposureError::DuplicateCorrelation { id: correlation });
            }
            if pre_observable.len() > in_flight_cap {
                return Err(ExposureError::TooManyPreObservableReservations {
                    maximum: in_flight_cap,
                });
            }
        }
        Ok(Self {
            state_change_id,
            reservations: pre_observable,
        })
    }

    fn contains(&self, correlation_id: u64) -> bool {
        self.reservations.contains_key(&correlation_id)
    }
}

#[derive(Debug, Clone)]
pub struct RunEvidence {
    pub scenario: ScenarioId,
    pub reproduction: Option<ReproductionRecord>,
    pub observations: Vec<Observation>,
    pub state_changes: Vec<MockStateChange>,
    pub unavoidable_exposure: Option<ExposureAllowance>,
    pub assertions: Vec<ScenarioAssertion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JudgeError {
    EvidenceBudgetExceeded {
        kind: &'static str,
        limit: usize,
    },
    MissingWireObservation,
    MissingScenarioAssertion,
    UnexpectedScenarioAssertion {
        expected: ScenarioAssertionId,
        got: ScenarioAssertionId,
    },
    DuplicateScenarioAssertion {
        id: ScenarioAssertionId,
    },
    MissingReproductionRecord,
    MissingM2Duration,
    InvalidM2Duration,
    DuplicateCorrelation {
        kind: &'static str,
        id: u64,
    },
    ExposureWithoutObservation {
        id: u64,
    },
    ExposureAfterTransportHandoff {
        id: u64,
    },
    ExposureUnrelatedStateChange {
        id: u64,
        state_change_id: u64,
    },
    ExposureStateChangeAfterArrival {
        id: u64,
        state_change_id: u64,
    },
    ExposureWithoutStateChange {
        id: u64,
    },
    DuplicateStateChange {
        id: u64,
    },
    ExposureAfterStateChangeObservable {
        id: u64,
        state_change_id: u64,
    },
    StateChangeNotObservable {
        id: u64,
    },
    ReproductionMismatch {
        id: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateResult {
    pub gate: Gate,
    pub applicable: bool,
    pub passed: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport {
    pub scenario: ScenarioId,
    pub reproduction: Option<ReproductionRecord>,
    /// `Fragment` if *any* assertion in the evidence was a fragment.
    pub contract_coverage: ContractCoverage,
    pub gates: Vec<GateResult>,
}

impl RunReport {
    pub fn passed(&self) -> bool {
        self.gates
            .iter()
            .all(|gate| !gate.applicable || gate.passed)
    }

    /// A verdict slot in `result-draft.md` may only be filled from a report
    /// that both passed and covered the whole scenario contract. A green
    /// fragment run is evidence of a seam, never of a scenario.
    pub fn verdict_eligible(&self) -> bool {
        self.passed() && self.contract_coverage == ContractCoverage::FullContract
    }

    pub fn gate(&self, gate: Gate) -> &GateResult {
        self.gates
            .iter()
            .find(|result| result.gate == gate)
            .expect("all six gates are always reported")
    }
}

/// A run-owned declaration that the complete mock-judged M-series contract
/// passed. This is deliberately independent of the clause registry: the
/// registry is the second authority consulted before evidence can fill a
/// verdict slot, not an input that can manufacture this declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullContractRun {
    reports: Vec<RunReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FullContractDeclarationError {
    MissingScenario { scenario: ScenarioId },
    ReportNotVerdictEligible { scenario: ScenarioId },
    MissingM8KnownLane,
    MissingM8AssumedLane,
    MissingEndpointLane { endpoint: Endpoint },
}

impl FullContractRun {
    /// Declares a full-contract run only when every M-row has at least one
    /// verdict-eligible report and M8 — the one row the contract requires in
    /// both provenance-typed bucket lanes (`m8-both-lanes`) — carries both.
    /// Scale and phase-domain reachability are asserted by the dedicated run
    /// producer; this constructor guards the report set itself.
    pub fn declare(reports: Vec<RunReport>) -> Result<Self, FullContractDeclarationError> {
        for report in &reports {
            if !report.verdict_eligible() {
                return Err(FullContractDeclarationError::ReportNotVerdictEligible {
                    scenario: report.scenario,
                });
            }
        }

        for scenario in ScenarioId::ALL {
            if !reports.iter().any(|report| report.scenario == scenario) {
                return Err(FullContractDeclarationError::MissingScenario { scenario });
            }
        }

        // The lane requirement is keyed to M8, not to the whole report set:
        // a whole-set check is satisfiable with no M8 Known lane at all
        // (M10's legacy row supplies Assumed and every ordinary row supplies
        // Known), so it cannot defend `m8-both-lanes` (SD-R8-F4).
        let m8_lane = |profile| {
            reports.iter().any(|report| {
                report.scenario == ScenarioId::M8
                    && report
                        .reproduction
                        .is_some_and(|record| record.client_buckets == profile)
            })
        };
        if !m8_lane(OAUTH_KNOWN_PROFILE) {
            return Err(FullContractDeclarationError::MissingM8KnownLane);
        }
        if !m8_lane(SHIPPED_ASSUMED_PROFILE) {
            return Err(FullContractDeclarationError::MissingM8AssumedLane);
        }

        // The verdict claims every policy in the N23 topology, so the
        // declaration requires every routed endpoint to appear in some
        // verdict-eligible report (SD-R8-F5). Without this, dropping the
        // character-policy lanes — the topology's tightest limits — would
        // still declare cleanly, the same silent-vanish shape as F4.
        for endpoint in Endpoint::ALL {
            let exercised = reports.iter().any(|report| {
                report
                    .reproduction
                    .is_some_and(|record| record.endpoint == endpoint)
            });
            if !exercised {
                return Err(FullContractDeclarationError::MissingEndpointLane { endpoint });
            }
        }

        Ok(Self { reports })
    }

    pub fn reports(&self) -> &[RunReport] {
        &self.reports
    }
}

pub fn judge(
    evidence: &RunEvidence,
    oracle: &impl ScenarioOracle,
) -> Result<RunReport, JudgeError> {
    for (kind, length) in [
        ("observations", evidence.observations.len()),
        ("state changes", evidence.state_changes.len()),
        ("assertions", evidence.assertions.len()),
    ] {
        if length > MAX_REQUESTS_PER_RUN {
            return Err(JudgeError::EvidenceBudgetExceeded {
                kind,
                limit: MAX_REQUESTS_PER_RUN,
            });
        }
    }
    if evidence.observations.is_empty() {
        return Err(JudgeError::MissingWireObservation);
    }
    if evidence.assertions.is_empty() {
        return Err(JudgeError::MissingScenarioAssertion);
    }
    let mut observation_ids = BTreeSet::new();
    for observation in &evidence.observations {
        if !observation_ids.insert(observation.correlation_id) {
            return Err(JudgeError::DuplicateCorrelation {
                kind: "observation",
                id: observation.correlation_id,
            });
        }
    }
    let mut state_change_ids = BTreeSet::new();
    for state_change in &evidence.state_changes {
        if !state_change_ids.insert(state_change.id) {
            return Err(JudgeError::DuplicateStateChange {
                id: state_change.id,
            });
        }
    }
    if let Some(exposure) = &evidence.unavoidable_exposure {
        let Some(state_change) = evidence
            .state_changes
            .iter()
            .find(|state_change| state_change.id == exposure.state_change_id)
        else {
            return Err(JudgeError::ExposureWithoutStateChange {
                id: exposure.state_change_id,
            });
        };
        let Some(observable_at_ms) =
            oracle.independently_observable_ms(state_change, &evidence.observations)
        else {
            return Err(JudgeError::StateChangeNotObservable {
                id: exposure.state_change_id,
            });
        };
        for (correlation_id, reserved_at_ms) in &exposure.reservations {
            let Some(observation) = evidence
                .observations
                .iter()
                .find(|observation| observation.correlation_id == *correlation_id)
            else {
                return Err(JudgeError::ExposureWithoutObservation {
                    id: *correlation_id,
                });
            };
            if !state_change.kind.affects_policy(&observation.policy) {
                return Err(JudgeError::ExposureUnrelatedStateChange {
                    id: *correlation_id,
                    state_change_id: state_change.id,
                });
            }
            if state_change.occurred_ms > observation.arrival_ms {
                return Err(JudgeError::ExposureStateChangeAfterArrival {
                    id: *correlation_id,
                    state_change_id: state_change.id,
                });
            }
            if *reserved_at_ms > observation.dispatch_ms {
                return Err(JudgeError::ExposureAfterTransportHandoff {
                    id: *correlation_id,
                });
            }
            if *reserved_at_ms >= observable_at_ms {
                return Err(JudgeError::ExposureAfterStateChangeObservable {
                    id: *correlation_id,
                    state_change_id: state_change.id,
                });
            }
        }
    }
    let spec = scenario(evidence.scenario);
    let mut assertion_ids = BTreeSet::new();
    for assertion in &evidence.assertions {
        if assertion.id != spec.required_assertion {
            return Err(JudgeError::UnexpectedScenarioAssertion {
                expected: spec.required_assertion,
                got: assertion.id,
            });
        }
        if !assertion_ids.insert(assertion.id) {
            return Err(JudgeError::DuplicateScenarioAssertion { id: assertion.id });
        }
    }
    if spec.sweep == SweepKind::PhaseSwept && evidence.reproduction.is_none() {
        return Err(JudgeError::MissingReproductionRecord);
    }
    if let Some(reproduction) = evidence.reproduction
        && let Some(observation) = evidence.observations.iter().find(|observation| {
            observation.seed != reproduction.seed || observation.phase_ms != reproduction.phase_ms
        })
    {
        return Err(JudgeError::ReproductionMismatch {
            id: observation.correlation_id,
        });
    }
    let m2_padded_minimum = (evidence.scenario == ScenarioId::M2)
        .then(|| oracle.m2_theoretical_padded_minimum_ms(&evidence.observations))
        .flatten();
    if evidence.scenario == ScenarioId::M2 && m2_padded_minimum.is_none() {
        return Err(JudgeError::MissingM2Duration);
    }
    if m2_padded_minimum == Some(0) {
        return Err(JudgeError::InvalidM2Duration);
    }

    let exposure = evidence.unavoidable_exposure.as_ref();
    let organic = evidence
        .observations
        .iter()
        .filter(|observation| observation.policy_judgment.organic_violation)
        .filter(|observation| {
            !exposure.is_some_and(|allowance| allowance.contains(observation.correlation_id))
        })
        .map(|observation| {
            format!(
                "correlation {} caused an organic policy violation",
                observation.correlation_id
            )
        })
        .collect::<Vec<_>>();
    let g1_failures = organic;

    let g2_failures = evidence
        .observations
        .iter()
        .filter(|observation| observation.layer1.tripped)
        .map(|observation| {
            format!(
                "correlation {} tripped a B10 ceiling",
                observation.correlation_id
            )
        })
        .collect::<Vec<_>>();

    let g3_failures = evidence
        .observations
        .iter()
        .filter_map(|observation| {
            // Fail closed here, in exactly one place: an observation the
            // oracle cannot vouch for is a G3 failure, not a free pass.
            let Some(independently_eligible_ms) = oracle.independently_eligible_ms(observation)
            else {
                return Some(format!(
                    "correlation {} has no independent eligibility entry",
                    observation.correlation_id
                ));
            };
            if observation.dispatch_ms < independently_eligible_ms {
                return Some(format!(
                    "correlation {} dispatched before independent eligibility",
                    observation.correlation_id
                ));
            }
            let deadline = independently_eligible_ms.saturating_add(G3_EPSILON_MS);
            if observation.dispatch_ms <= deadline
                || oracle.authorizes_delay(independently_eligible_ms, observation.dispatch_ms)
            {
                return None;
            }
            Some(format!(
                "correlation {} exceeded G3 by {} ms",
                observation.correlation_id,
                observation
                    .dispatch_ms
                    .saturating_sub(independently_eligible_ms)
            ))
        })
        .collect::<Vec<_>>();

    let g4_failures = m2_padded_minimum
        .into_iter()
        .filter_map(|theoretical_padded_minimum_ms| {
            let first_dispatch_ms = evidence
                .observations
                .iter()
                .map(|observation| observation.dispatch_ms)
                .min()
                .expect("non-empty wire evidence was checked above");
            let last_completion_ms = evidence
                .observations
                .iter()
                .map(|observation| observation.completion_ms)
                .max()
                .expect("non-empty wire evidence was checked above");
            let actual_ms = last_completion_ms.saturating_sub(first_dispatch_ms);
            let limit = theoretical_padded_minimum_ms.saturating_mul(105).div_ceil(100);
            (actual_ms > limit).then(|| {
                format!(
                    "M2 duration {actual_ms} ms exceeds 1.05x padded minimum {theoretical_padded_minimum_ms} ms"
                )
            })
        })
        .collect::<Vec<_>>();

    let g5_failures = evidence
        .assertions
        .iter()
        .filter(|assertion| !assertion.passed)
        .map(|assertion| format!("{:?}", assertion.id))
        .collect::<Vec<_>>();

    let g6_failures = if spec.sweep == SweepKind::PhaseSwept && evidence.reproduction.is_none() {
        vec!["phase-swept failure lacks (seed, phi)".to_owned()]
    } else {
        Vec::new()
    };

    let failures = HashMap::from([
        (Gate::G1, g1_failures),
        (Gate::G2, g2_failures),
        (Gate::G3, g3_failures),
        (Gate::G4, g4_failures),
        (Gate::G5, g5_failures),
        (Gate::G6, g6_failures),
    ]);
    let contract_coverage = if evidence
        .assertions
        .iter()
        .all(|assertion| assertion.coverage == ContractCoverage::FullContract)
    {
        ContractCoverage::FullContract
    } else {
        ContractCoverage::Fragment
    };
    Ok(RunReport {
        scenario: evidence.scenario,
        reproduction: evidence.reproduction,
        contract_coverage,
        gates: [Gate::G1, Gate::G2, Gate::G3, Gate::G4, Gate::G5, Gate::G6]
            .into_iter()
            .map(|gate| {
                let failures = failures.get(&gate).cloned().unwrap_or_default();
                let applicable = gate != Gate::G4 || evidence.scenario == ScenarioId::M2;
                GateResult {
                    gate,
                    applicable,
                    passed: failures.is_empty(),
                    failures,
                }
            })
            .collect(),
    })
}
