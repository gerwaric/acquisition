//! M-series scenario metadata and client-independent gate judgment.

use std::collections::{BTreeSet, HashMap};

use crate::mock::Observation;
use crate::mock::model::MAX_REQUESTS_PER_RUN;

pub const G3_EPSILON_MS: u64 = 500;
pub const MAX_SWEEP_CONFIGURATIONS: usize = 256;
pub const D5_IN_FLIGHT_CAP: usize = 2;
pub const MAX_ASSERTION_NAME_BYTES: usize = 256;

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
    },
    ScenarioSpec {
        id: ScenarioId::M2,
        name: "clean cold-start saturation burst",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M2_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M3,
        name: "degraded HEAD",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M3_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M4,
        name: "unexpected policy shape",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M4_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M5,
        name: "policy rename mid-session",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M5_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M6,
        name: "policy shrink mid-flight",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M6_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M7,
        name: "phantom same-account hits",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M7_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M8,
        name: "429 recovery and escalation ladder",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M8_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M9,
        name: "phantom race at saturation",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M9_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M10,
        name: "agent-loop stress",
        sweep: SweepKind::PhaseSwept,
        binding_gates: M10_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M11,
        name: "layer-1 ceiling and Cloudflare terminal",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M11_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M12,
        name: "4xx-tripwire obligations",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M12_GATES,
    },
    ScenarioSpec {
        id: ScenarioId::M13,
        name: "gate structure on the wire",
        sweep: SweepKind::PhaseIndependent,
        binding_gates: M13_GATES,
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

impl AuthorizedExclusion {
    fn covers(self, begins_ms: u64, ends_ms: u64) -> bool {
        self.begins_ms <= begins_ms && ends_ms <= self.ends_ms
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DispatchSample {
    pub correlation_id: u64,
    pub independently_eligible_ms: u64,
    pub dispatched_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioAssertion {
    pub name: String,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurationEvidence {
    pub actual_ms: u64,
    pub theoretical_padded_minimum_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExposureAllowance {
    correlations: BTreeSet<u64>,
    maximum: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExposureError {
    AboveD5InFlightCap,
}

impl ExposureAllowance {
    /// Attributes only reservations made before the mutation/race became
    /// observable, capped independently at the D5 in-flight limit.
    pub fn before_observable(
        reservations: impl IntoIterator<Item = (u64, u64)>,
        observable_at_ms: u64,
        in_flight_cap: usize,
    ) -> Result<Self, ExposureError> {
        if in_flight_cap > D5_IN_FLIGHT_CAP {
            return Err(ExposureError::AboveD5InFlightCap);
        }
        let correlations = reservations
            .into_iter()
            .filter_map(|(correlation, reserved_at)| {
                (reserved_at < observable_at_ms).then_some(correlation)
            })
            .take(in_flight_cap)
            .collect();
        Ok(Self {
            correlations,
            maximum: in_flight_cap,
        })
    }

    fn contains(&self, correlation_id: u64) -> bool {
        self.correlations.contains(&correlation_id)
    }
}

#[derive(Debug, Clone)]
pub struct RunEvidence {
    pub scenario: ScenarioId,
    pub reproduction: Option<ReproductionRecord>,
    pub observations: Vec<Observation>,
    pub dispatch_samples: Vec<DispatchSample>,
    pub exclusions: Vec<AuthorizedExclusion>,
    pub unavoidable_exposure: Option<ExposureAllowance>,
    pub assertions: Vec<ScenarioAssertion>,
    pub duration: Option<DurationEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JudgeError {
    EvidenceBudgetExceeded { kind: &'static str, limit: usize },
    MissingWireObservation,
    MissingDispatchSample,
    MissingScenarioAssertion,
    MissingReproductionRecord,
    MissingM2Duration,
    InvalidM2Duration,
    UnexpectedDurationEvidence,
    DuplicateCorrelation { kind: &'static str, id: u64 },
    MissingDispatchForObservation { id: u64 },
    ReproductionMismatch { id: u64 },
    AssertionNameTooLong { limit: usize },
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
    pub gates: Vec<GateResult>,
}

impl RunReport {
    pub fn passed(&self) -> bool {
        self.gates
            .iter()
            .all(|gate| !gate.applicable || gate.passed)
    }

    pub fn gate(&self, gate: Gate) -> &GateResult {
        self.gates
            .iter()
            .find(|result| result.gate == gate)
            .expect("all six gates are always reported")
    }
}

pub fn judge(evidence: &RunEvidence) -> Result<RunReport, JudgeError> {
    for (kind, length) in [
        ("observations", evidence.observations.len()),
        ("dispatch samples", evidence.dispatch_samples.len()),
        ("exclusions", evidence.exclusions.len()),
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
    if evidence.dispatch_samples.is_empty() {
        return Err(JudgeError::MissingDispatchSample);
    }
    if evidence.assertions.is_empty() {
        return Err(JudgeError::MissingScenarioAssertion);
    }
    if evidence
        .assertions
        .iter()
        .any(|assertion| assertion.name.len() > MAX_ASSERTION_NAME_BYTES)
    {
        return Err(JudgeError::AssertionNameTooLong {
            limit: MAX_ASSERTION_NAME_BYTES,
        });
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
    let mut dispatch_ids = BTreeSet::new();
    for dispatch in &evidence.dispatch_samples {
        if !dispatch_ids.insert(dispatch.correlation_id) {
            return Err(JudgeError::DuplicateCorrelation {
                kind: "dispatch sample",
                id: dispatch.correlation_id,
            });
        }
    }
    if let Some(id) = observation_ids.iter().find(|id| !dispatch_ids.contains(id)) {
        return Err(JudgeError::MissingDispatchForObservation { id: *id });
    }
    let spec = scenario(evidence.scenario);
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
    match (evidence.scenario, &evidence.duration) {
        (ScenarioId::M2, None) => return Err(JudgeError::MissingM2Duration),
        (
            ScenarioId::M2,
            Some(DurationEvidence {
                theoretical_padded_minimum_ms: 0,
                ..
            }),
        ) => return Err(JudgeError::InvalidM2Duration),
        (ScenarioId::M2, Some(_)) | (_, None) => {}
        (_, Some(_)) => return Err(JudgeError::UnexpectedDurationEvidence),
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
    let allowed_exposure_count = evidence
        .observations
        .iter()
        .filter(|observation| observation.policy_judgment.organic_violation)
        .filter(|observation| {
            exposure.is_some_and(|allowance| allowance.contains(observation.correlation_id))
        })
        .count();
    let mut g1_failures = organic;
    if exposure.is_some_and(|allowance| allowed_exposure_count > allowance.maximum) {
        g1_failures.push("unavoidable exposure exceeded its independent cap".to_owned());
    }

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
        .dispatch_samples
        .iter()
        .filter(|sample| {
            if sample.dispatched_ms < sample.independently_eligible_ms {
                return true;
            }
            let deadline = sample
                .independently_eligible_ms
                .saturating_add(G3_EPSILON_MS);
            if sample.dispatched_ms <= deadline {
                return false;
            }
            !evidence.exclusions.iter().any(|exclusion| {
                exclusion.covers(sample.independently_eligible_ms, sample.dispatched_ms)
            })
        })
        .map(|sample| {
            if sample.dispatched_ms < sample.independently_eligible_ms {
                format!(
                    "correlation {} dispatched before independent eligibility",
                    sample.correlation_id
                )
            } else {
                format!(
                    "correlation {} exceeded G3 by {} ms",
                    sample.correlation_id,
                    sample
                        .dispatched_ms
                        .saturating_sub(sample.independently_eligible_ms)
                )
            }
        })
        .collect::<Vec<_>>();

    let g4_failures = evidence
        .duration
        .iter()
        .filter(|duration| {
            let limit = duration
                .theoretical_padded_minimum_ms
                .saturating_mul(105)
                .div_ceil(100);
            duration.actual_ms > limit
        })
        .map(|duration| {
            format!(
                "M2 duration {} ms exceeds 1.05x padded minimum {} ms",
                duration.actual_ms, duration.theoretical_padded_minimum_ms
            )
        })
        .collect::<Vec<_>>();

    let g5_failures = evidence
        .assertions
        .iter()
        .filter(|assertion| !assertion.passed)
        .map(|assertion| assertion.name.clone())
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
    Ok(RunReport {
        scenario: evidence.scenario,
        reproduction: evidence.reproduction,
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
