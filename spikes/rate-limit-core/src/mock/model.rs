//! Independent server-side counter arithmetic for the conformance mock.
//!
//! This module deliberately does not import the production core or parser.
//! Its only input is a received-wire timestamp plus typed mock configuration.

use std::collections::{HashMap, VecDeque};
use std::num::NonZeroU64;

pub const MAX_REQUESTS_PER_RUN: usize = 10_000;
pub const MAX_EVENTS_PER_POLICY: usize = 10_000;
pub const LAYER1_BURST_LIMIT: usize = 20;
pub const LAYER1_SUSTAINED_LIMIT: usize = 1_000;

const MAX_RULES_PER_POLICY: usize = 8;
const MAX_WINDOWS_PER_RULE: usize = 8;
const MAX_HITS: u32 = 10_000;
const MAX_TIME_MS: u64 = 21_600_000;
const MAX_POLICY_NAME_BYTES: usize = 256;
const MAX_RULE_NAME_BYTES: usize = 64;

const ONE_SECOND_MS: u64 = 1_000;
const ONE_MINUTE_MS: u64 = 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ServerTime(u64);

impl ServerTime {
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowDefinition {
    max_hits: u32,
    period_ms: u64,
    restriction_ms: u64,
    bucket_ms: u64,
}

impl WindowDefinition {
    pub fn new(
        max_hits: u32,
        period_ms: u64,
        restriction_ms: u64,
        bucket_ms: u64,
    ) -> Result<Self, DefinitionError> {
        if max_hits == 0
            || max_hits > MAX_HITS
            || period_ms == 0
            || period_ms > MAX_TIME_MS
            || restriction_ms > MAX_TIME_MS
            || bucket_ms == 0
            || bucket_ms > MAX_TIME_MS
            || !period_ms.is_multiple_of(1_000)
            || !restriction_ms.is_multiple_of(1_000)
        {
            return Err(DefinitionError::InvalidWindow);
        }
        Ok(Self {
            max_hits,
            period_ms,
            restriction_ms,
            bucket_ms,
        })
    }

    pub const fn max_hits(&self) -> u32 {
        self.max_hits
    }

    pub const fn period_ms(&self) -> u64 {
        self.period_ms
    }

    pub const fn restriction_ms(&self) -> u64 {
        self.restriction_ms
    }

    pub const fn bucket_ms(&self) -> u64 {
        self.bucket_ms
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDefinition {
    name: String,
    windows: Vec<WindowDefinition>,
}

impl RuleDefinition {
    pub fn new(
        name: impl Into<String>,
        windows: Vec<WindowDefinition>,
    ) -> Result<Self, DefinitionError> {
        let name = name.into();
        if name.is_empty()
            || name.len() > MAX_RULE_NAME_BYTES
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(DefinitionError::InvalidRuleName);
        }
        if windows.is_empty() || windows.len() > MAX_WINDOWS_PER_RULE {
            return Err(DefinitionError::InvalidWindowCount);
        }
        Ok(Self { name, windows })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn windows(&self) -> &[WindowDefinition] {
        &self.windows
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyDefinition {
    name: String,
    rules: Vec<RuleDefinition>,
}

impl PolicyDefinition {
    pub fn new(
        name: impl Into<String>,
        rules: Vec<RuleDefinition>,
    ) -> Result<Self, DefinitionError> {
        let name = name.into();
        if name.is_empty() || name.len() > MAX_POLICY_NAME_BYTES {
            return Err(DefinitionError::InvalidPolicyName);
        }
        if rules.is_empty() || rules.len() > MAX_RULES_PER_POLICY {
            return Err(DefinitionError::InvalidRuleCount);
        }
        Ok(Self { name, rules })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn rules(&self) -> &[RuleDefinition] {
        &self.rules
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionError {
    InvalidPolicyName,
    InvalidRuleCount,
    InvalidRuleName,
    InvalidWindowCount,
    InvalidWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitSource {
    Client { correlation_id: u64 },
    Phantom,
    Residue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Hit {
    at: ServerTime,
    source: HitSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PolicyState {
    definition: PolicyDefinition,
    hits: VecDeque<Hit>,
    restricted_until: Vec<Vec<Option<ServerTime>>>,
}

impl PolicyState {
    fn new(definition: PolicyDefinition) -> Self {
        let restricted_until = definition
            .rules
            .iter()
            .map(|rule| vec![None; rule.windows.len()])
            .collect();
        Self {
            definition,
            hits: VecDeque::new(),
            restricted_until,
        }
    }

    fn retire(&mut self, now: ServerTime, phase_ms: u64) {
        let horizon = self
            .definition
            .rules
            .iter()
            .flat_map(|rule| &rule.windows)
            .map(|window| window.period_ms.saturating_add(window.bucket_ms))
            .max()
            .unwrap_or(0);
        self.hits.retain(|hit| {
            now.0
                < bucket_end(hit.at.0, hit_bucket_ceiling(&self.definition), phase_ms)
                    .saturating_add(horizon)
        });
    }

    /// Carries server-observed facts into a new set of policy judgments.
    ///
    /// A rename and a shrink differ only in the definition name: neither
    /// permits the server to forget received hits or an active restriction.
    fn redefined(definition: PolicyDefinition, previous: Self) -> Self {
        let mut replacement = Self::new(definition);
        replacement.hits = previous.hits;
        // The docs do not define how an active restriction maps across an
        // arbitrary rule reorder/reshape. Carrying the latest old deadline to
        // every new slot is conservative and independent of rule position.
        let latest_restriction = previous
            .restricted_until
            .iter()
            .flatten()
            .flatten()
            .copied()
            .max();
        if let Some(latest_restriction) = latest_restriction {
            for new_window in replacement.restricted_until.iter_mut().flatten() {
                *new_window = Some(latest_restriction);
            }
        }
        replacement
    }
}

// Retention is permitted to be looser than any individual window, but never
// shorter. Using the largest bucket avoids dropping a hit another window can
// still count without sharing the client's per-window deadline helper.
fn hit_bucket_ceiling(definition: &PolicyDefinition) -> u64 {
    definition
        .rules
        .iter()
        .flat_map(|rule| &rule.windows)
        .map(|window| window.bucket_ms)
        .max()
        .unwrap_or(1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    DuplicatePolicy(String),
    UnknownPolicy(String),
    EventBudgetExceeded { policy: String, limit: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BucketObservation {
    pub rule: String,
    pub window_index: usize,
    pub bucket_end_ms: u64,
    pub current_hits: u32,
    pub client_hits: u32,
    pub phantom_hits: u32,
    pub residue_hits: u32,
    pub restriction_active_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyJudgment {
    pub policy: String,
    pub counted: bool,
    pub organic_violation: bool,
    pub retry_after_seconds: Option<u64>,
    pub windows: Vec<BucketObservation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer1Judgment {
    pub burst_count: usize,
    pub sustained_count: usize,
    pub tripped: bool,
}

#[derive(Debug)]
pub struct CounterModel {
    phase_ms: u64,
    policies: HashMap<String, PolicyState>,
    layer1_arrivals: VecDeque<ServerTime>,
}

impl CounterModel {
    pub fn new(phase_ms: u64) -> Self {
        Self {
            phase_ms,
            policies: HashMap::new(),
            layer1_arrivals: VecDeque::new(),
        }
    }

    pub const fn phase_ms(&self) -> u64 {
        self.phase_ms
    }

    pub fn insert_policy(&mut self, definition: PolicyDefinition) -> Result<(), ModelError> {
        let name = definition.name.clone();
        if self.policies.contains_key(&name) {
            return Err(ModelError::DuplicatePolicy(name));
        }
        self.policies.insert(name, PolicyState::new(definition));
        Ok(())
    }

    /// Replaces judgments while retaining arrival facts (M6: hits are facts,
    /// rules are judgments).
    pub fn replace_policy(&mut self, definition: PolicyDefinition) -> Result<(), ModelError> {
        let name = definition.name.clone();
        let Some(previous) = self.policies.remove(&name) else {
            return Err(ModelError::UnknownPolicy(definition.name));
        };
        self.policies
            .insert(name, PolicyState::redefined(definition, previous));
        Ok(())
    }

    /// Renames a policy without treating the new name as a fresh server
    /// counter. M5 requires the response identity to change while all prior
    /// hits and restrictions remain server facts.
    pub fn rename_policy(
        &mut self,
        old_name: &str,
        definition: PolicyDefinition,
    ) -> Result<(), ModelError> {
        let new_name = definition.name.clone();
        if old_name == new_name {
            return self.replace_policy(definition);
        }
        if self.policies.contains_key(&new_name) {
            return Err(ModelError::DuplicatePolicy(new_name));
        }
        let Some(previous) = self.policies.remove(old_name) else {
            return Err(ModelError::UnknownPolicy(old_name.to_owned()));
        };
        self.policies
            .insert(new_name, PolicyState::redefined(definition, previous));
        Ok(())
    }

    pub fn definition(&self, policy: &str) -> Result<&PolicyDefinition, ModelError> {
        self.policies
            .get(policy)
            .map(|state| &state.definition)
            .ok_or_else(|| ModelError::UnknownPolicy(policy.to_owned()))
    }

    pub fn record_layer1_arrival(&mut self, now: ServerTime) -> Layer1Judgment {
        self.layer1_arrivals
            .retain(|at| in_rolling_window(at.0, now.0, ONE_MINUTE_MS));
        self.layer1_arrivals.push_back(now);
        let burst_count = self
            .layer1_arrivals
            .iter()
            .filter(|at| in_rolling_window(at.0, now.0, ONE_SECOND_MS))
            .count();
        let sustained_count = self.layer1_arrivals.len();
        Layer1Judgment {
            burst_count,
            sustained_count,
            tripped: burst_count > LAYER1_BURST_LIMIT || sustained_count > LAYER1_SUSTAINED_LIMIT,
        }
    }

    pub fn preload(
        &mut self,
        policy: &str,
        at: ServerTime,
        count: usize,
    ) -> Result<(), ModelError> {
        self.inject(policy, at, count, HitSource::Residue)
    }

    pub fn inject_phantoms(
        &mut self,
        policy: &str,
        at: ServerTime,
        count: usize,
    ) -> Result<(), ModelError> {
        self.inject(policy, at, count, HitSource::Phantom)
    }

    fn inject(
        &mut self,
        policy_name: &str,
        at: ServerTime,
        count: usize,
        source: HitSource,
    ) -> Result<(), ModelError> {
        let Some(policy) = self.policies.get_mut(policy_name) else {
            return Err(ModelError::UnknownPolicy(policy_name.to_owned()));
        };
        policy.retire(at, self.phase_ms);
        if policy.hits.len().saturating_add(count) > MAX_EVENTS_PER_POLICY {
            return Err(ModelError::EventBudgetExceeded {
                policy: policy_name.to_owned(),
                limit: MAX_EVENTS_PER_POLICY,
            });
        }
        policy.hits.extend((0..count).map(|_| Hit { at, source }));
        Ok(())
    }

    pub fn judge(
        &mut self,
        policy_name: &str,
        now: ServerTime,
        correlation_id: u64,
        counted: bool,
    ) -> Result<PolicyJudgment, ModelError> {
        let Some(policy) = self.policies.get_mut(policy_name) else {
            return Err(ModelError::UnknownPolicy(policy_name.to_owned()));
        };
        policy.retire(now, self.phase_ms);
        if counted {
            if policy.hits.len() == MAX_EVENTS_PER_POLICY {
                return Err(ModelError::EventBudgetExceeded {
                    policy: policy_name.to_owned(),
                    limit: MAX_EVENTS_PER_POLICY,
                });
            }
            policy.hits.push_back(Hit {
                at: now,
                source: HitSource::Client { correlation_id },
            });
        }

        let mut organic_violation = false;
        let mut retry_after_ms = 0;
        let mut windows = Vec::new();
        for (rule_index, rule) in policy.definition.rules.iter().enumerate() {
            for (window_index, window) in rule.windows.iter().enumerate() {
                let active_hits = policy.hits.iter().filter(|hit| {
                    now.0
                        < bucket_end(hit.at.0, window.bucket_ms, self.phase_ms)
                            .saturating_add(window.period_ms)
                });
                let (mut current_hits, mut client_hits, mut phantom_hits, mut residue_hits) =
                    (0_usize, 0_usize, 0_usize, 0_usize);
                for hit in active_hits {
                    current_hits += 1;
                    match hit.source {
                        HitSource::Client { .. } => client_hits += 1,
                        HitSource::Phantom => phantom_hits += 1,
                        HitSource::Residue => residue_hits += 1,
                    }
                }
                let active_until = policy.restricted_until[rule_index][window_index];
                let already_restricted = active_until.is_some_and(|until| now < until);
                let newly_over_limit = counted
                    && current_hits
                        > usize::try_from(window.max_hits).expect("u32 fits supported usize");
                if newly_over_limit {
                    let candidate = ServerTime::from_millis(
                        bucket_end(now.0, window.bucket_ms, self.phase_ms)
                            .saturating_add(window.restriction_ms),
                    );
                    policy.restricted_until[rule_index][window_index] =
                        Some(active_until.map_or(candidate, |current| current.max(candidate)));
                }
                let until = policy.restricted_until[rule_index][window_index];
                if already_restricted || newly_over_limit {
                    organic_violation = true;
                }
                let window_retry_after_ms = until
                    .filter(|until| now < *until)
                    .map_or(0, |until| until.0 - now.0);
                retry_after_ms = retry_after_ms.max(window_retry_after_ms);
                windows.push(BucketObservation {
                    rule: rule.name.clone(),
                    window_index,
                    bucket_end_ms: if counted {
                        bucket_end(now.0, window.bucket_ms, self.phase_ms)
                    } else {
                        now.0
                    },
                    current_hits: u32::try_from(current_hits)
                        .expect("the structural event budget is below u32::MAX"),
                    client_hits: u32::try_from(client_hits)
                        .expect("the structural event budget is below u32::MAX"),
                    phantom_hits: u32::try_from(phantom_hits)
                        .expect("the structural event budget is below u32::MAX"),
                    residue_hits: u32::try_from(residue_hits)
                        .expect("the structural event budget is below u32::MAX"),
                    restriction_active_seconds: window_retry_after_ms.div_ceil(ONE_SECOND_MS),
                });
            }
        }

        Ok(PolicyJudgment {
            policy: policy_name.to_owned(),
            counted,
            organic_violation,
            retry_after_seconds: organic_violation.then(|| retry_after_ms.div_ceil(ONE_SECOND_MS)),
            windows,
        })
    }
}

/// Distance from t0 to the first `bucket_ms` boundary under `phase_ms`.
///
/// `phase_ms` names the *upcoming* boundary; it is not an offset already
/// elapsed. Phase 0 therefore puts the first boundary a full bucket away,
/// and phase 1 puts it 1 ms out — a phase just *under* the bucket length is
/// 1 ms from phase 0, not from an immediate boundary.
///
/// Exposed because a phase sweep cannot otherwise assert what its own phases
/// mean: reading this backwards is what produced review findings F1 and F7,
/// and the second reading survived because nothing could pin it.
pub fn first_bucket_boundary_ms(bucket_ms: NonZeroU64, phase_ms: u64) -> u64 {
    bucket_end(0, bucket_ms.get(), phase_ms)
}

fn in_rolling_window(at_ms: u64, now_ms: u64, window_ms: u64) -> bool {
    at_ms > now_ms || now_ms < window_ms || at_ms > now_ms - window_ms
}

/// Returns the end of the `[start, end)` bucket containing `at`.
///
/// An arrival exactly on a boundary belongs to the new bucket and therefore
/// receives a full bucket of pessimistic age extension.
fn bucket_end(at_ms: u64, bucket_ms: u64, phase_ms: u64) -> u64 {
    debug_assert!(bucket_ms > 0);
    let phase = phase_ms % bucket_ms;
    if at_ms < phase {
        return phase;
    }
    let bucket_index = (at_ms - phase) / bucket_ms;
    phase.saturating_add(bucket_index.saturating_add(1).saturating_mul(bucket_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_boundaries_use_the_adversarial_new_bucket_reading() {
        assert_eq!(bucket_end(999, 1_000, 0), 1_000);
        assert_eq!(bucket_end(1_000, 1_000, 0), 2_000);
        assert_eq!(bucket_end(1_001, 1_000, 0), 2_000);
        assert_eq!(bucket_end(0, 1_000, 250), 250);
        assert_eq!(bucket_end(250, 1_000, 250), 1_250);
    }
}
