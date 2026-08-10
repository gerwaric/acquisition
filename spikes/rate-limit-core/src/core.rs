use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::Duration;

use http::{HeaderMap, HeaderName, StatusCode};

use crate::header::{PolicyParseError, PolicySnapshot, parse_policy};
pub use crate::header::{RulePair, Window};

pub const RESTRICTION_BUFFER: Duration = Duration::from_secs(1);
pub const RETRY_AFTER_CAP: Duration = Duration::from_secs(900);

const RETRY_AFTER_HEADER: HeaderName = HeaderName::from_static("retry-after");

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimInstant(u64);

impl SimInstant {
    pub const MAX: Self = Self(u64::MAX);

    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub const fn as_millis(self) -> u64 {
        self.0
    }

    pub fn saturating_add(self, duration: Duration) -> Self {
        let millis = duration.as_nanos().div_ceil(1_000_000);
        let millis = u64::try_from(millis).unwrap_or(u64::MAX);
        Self(self.0.saturating_add(millis))
    }
}

macro_rules! string_newtype {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

string_newtype!(PolicyName);
string_newtype!(EndpointLabel);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleScope {
    Account,
    Ip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Known(Duration),
    Assumed(Duration),
}

impl Resolution {
    pub const fn duration(self) -> Duration {
        match self {
            Self::Known(duration) | Self::Assumed(duration) => duration,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketModel {
    burst: Resolution,
    sustained: Resolution,
}

impl BucketModel {
    pub const fn new(burst: Resolution, sustained: Resolution) -> Self {
        Self { burst, sustained }
    }

    pub const fn burst(&self) -> Resolution {
        self.burst
    }

    pub const fn sustained(&self) -> Resolution {
        self.sustained
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    scope: RuleScope,
    pair: RulePair,
    buckets: BucketModel,
}

impl Rule {
    pub const fn new(scope: RuleScope, pair: RulePair, buckets: BucketModel) -> Self {
        Self {
            scope,
            pair,
            buckets,
        }
    }

    pub const fn scope(&self) -> RuleScope {
        self.scope
    }

    pub const fn pair(&self) -> &RulePair {
        &self.pair
    }

    pub const fn buckets(&self) -> BucketModel {
        self.buckets
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(u64);

impl EntryId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    LocalReservation,
    Synthetic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    id: EntryId,
    at: SimInstant,
    kind: EntryKind,
}

impl HistoryEntry {
    pub const fn id(&self) -> EntryId {
        self.id
    }

    pub const fn at(&self) -> SimInstant {
        self.at
    }

    pub const fn kind(&self) -> EntryKind {
        self.kind
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    entries: VecDeque<HistoryEntry>,
}

impl History {
    pub fn entries(&self) -> impl ExactSizeIterator<Item = &HistoryEntry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Counts entries in the rolling half-open interval `(now - period, now]`.
    ///
    /// Future-dated entries are retained conservatively if a caller supplies a
    /// non-monotonic simulated clock.
    pub fn count_within(&self, now: SimInstant, period: Duration) -> usize {
        self.entries
            .iter()
            .filter(|entry| is_within(entry.at, now, period))
            .count()
    }

    fn push(&mut self, entry: HistoryEntry) {
        self.entries.push_back(entry);
    }

    fn remove(&mut self, id: EntryId) -> Option<HistoryEntry> {
        let position = self.entries.iter().position(|entry| entry.id == id)?;
        self.entries.remove(position)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    name: PolicyName,
    rules: Vec<Rule>,
    history: History,
    restriction_generation: u64,
    restricted_until: Option<SimInstant>,
    recovery_episode: Option<RecoveryEpisode>,
    escalation_suspended: bool,
}

impl Policy {
    /// A policy without rules would have no windows to schedule against and
    /// no bucket to size a restriction with, so the shape is rejected here —
    /// downstream code never re-checks for emptiness.
    pub fn new(name: impl Into<PolicyName>, rules: Vec<Rule>) -> Result<Self, EmptyPolicy> {
        let name = name.into();
        if rules.is_empty() {
            return Err(EmptyPolicy(name));
        }
        Ok(Self {
            name,
            rules,
            history: History::default(),
            restriction_generation: 0,
            restricted_until: None,
            recovery_episode: None,
            escalation_suspended: false,
        })
    }

    pub fn name(&self) -> &PolicyName {
        &self.name
    }

    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    pub fn history(&self) -> &History {
        &self.history
    }

    pub const fn restriction_generation(&self) -> u64 {
        self.restriction_generation
    }

    pub const fn restricted_until(&self) -> Option<SimInstant> {
        self.restricted_until
    }

    pub const fn recovery_episode(&self) -> Option<&RecoveryEpisode> {
        self.recovery_episode.as_ref()
    }

    pub const fn is_escalation_suspended(&self) -> bool {
        self.escalation_suspended
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationAttempt {
    First,
    Final,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEpisode {
    opened_generation: u64,
    completed_attempts: u8,
    confirmation_entry: Option<(EntryId, ConfirmationAttempt)>,
}

impl RecoveryEpisode {
    pub const fn opened_generation(&self) -> u64 {
        self.opened_generation
    }

    pub const fn completed_attempts(&self) -> u8 {
        self.completed_attempts
    }

    pub const fn confirmation_in_flight(&self) -> bool {
        self.confirmation_entry.is_some()
    }
}

#[must_use = "a reservation must be consumed by rollback, on_response, or on_unknown_outcome"]
pub struct ReservationToken {
    policy: PolicyName,
    entry_id: EntryId,
    restriction_generation: u64,
    confirmation_attempt: Option<ConfirmationAttempt>,
    consumed: bool,
}

impl ReservationToken {
    pub fn policy(&self) -> &PolicyName {
        &self.policy
    }

    pub const fn entry_id(&self) -> EntryId {
        self.entry_id
    }

    pub const fn restriction_generation(&self) -> u64 {
        self.restriction_generation
    }

    pub const fn confirmation_attempt(&self) -> Option<ConfirmationAttempt> {
        self.confirmation_attempt
    }

    fn consume(&mut self) {
        self.consumed = true;
    }
}

impl fmt::Debug for ReservationToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReservationToken")
            .field("policy", &self.policy)
            .field("entry_id", &self.entry_id)
            .field("restriction_generation", &self.restriction_generation)
            .field("confirmation_attempt", &self.confirmation_attempt)
            .finish_non_exhaustive()
    }
}

impl Drop for ReservationToken {
    fn drop(&mut self) {
        if cfg!(debug_assertions) && !self.consumed && !std::thread::panicking() {
            panic!(
                "reservation token for policy {} and entry {:?} dropped without an outcome",
                self.policy, self.entry_id
            );
        }
    }
}

#[must_use = "a reservation decision must be handled"]
#[derive(Debug)]
pub enum ReserveOutcome {
    Reserved(ReservationToken),
    NotBefore(SimInstant),
    Refused(RefusalReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalReason {
    UnknownPolicy(PolicyName),
    EscalationSuspended(PolicyName),
    Halted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicatePolicy(pub PolicyName);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyPolicy(pub PolicyName);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reconciliation {
    synthesized_entries: usize,
}

impl Reconciliation {
    pub const fn synthesized_entries(self) -> usize {
        self.synthesized_entries
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationError {
    UnknownPolicy(PolicyName),
    PolicyMismatch {
        reserved: PolicyName,
        observed: PolicyName,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyClassification {
    Normal,
    CloudflareShaped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedResponse {
    status: StatusCode,
    headers: HeaderMap,
    classification: ReplyClassification,
}

impl ObservedResponse {
    pub fn new(
        status: StatusCode,
        headers: HeaderMap,
        classification: ReplyClassification,
    ) -> Self {
        Self {
            status,
            headers,
            classification,
        }
    }

    pub const fn status(&self) -> StatusCode {
        self.status
    }

    pub const fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub const fn classification(&self) -> ReplyClassification {
        self.classification
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryAfterError {
    Missing,
    Invalid,
    AboveCap { seconds: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalCause {
    PolicyObservation(PolicyParseError),
    ObservationTarget(ObservationError),
    RetryAfter(RetryAfterError),
    RecoveryEscalated,
    ProbeStatus(StatusCode),
    ProbeUnknownOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalTarget {
    Policy(PolicyName),
    Endpoint(EndpointLabel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disposition {
    CompleteRequest,
    ProbeReady,
    Requeue,
    Refuse {
        target: RefusalTarget,
        cause: RefusalCause,
    },
    Halt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    StateChanged,
}

#[must_use = "a response transition must be interpreted"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    disposition: Disposition,
    notifications: Vec<Notification>,
}

impl Transition {
    pub const fn disposition(&self) -> &Disposition {
        &self.disposition
    }

    pub fn notifications(&self) -> &[Notification] {
        &self.notifications
    }

    fn new(disposition: Disposition, state_changed: bool) -> Self {
        Self {
            disposition,
            notifications: if state_changed {
                vec![Notification::StateChanged]
            } else {
                Vec::new()
            },
        }
    }
}

#[derive(Debug, Default)]
pub struct PolicyEngine {
    policies: HashMap<PolicyName, Policy>,
    next_entry_id: u64,
    halted: bool,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_policy(&mut self, policy: Policy) -> Result<(), DuplicatePolicy> {
        let name = policy.name.clone();
        if self.policies.contains_key(&name) {
            return Err(DuplicatePolicy(name));
        }
        self.policies.insert(name, policy);
        Ok(())
    }

    pub fn policy(&self, name: &PolicyName) -> Option<&Policy> {
        self.policies.get(name)
    }

    pub const fn is_halted(&self) -> bool {
        self.halted
    }

    /// Makes one scheduling decision and records the send on a grant.
    pub fn try_reserve(&mut self, policy_name: &PolicyName, now: SimInstant) -> ReserveOutcome {
        if self.halted {
            return ReserveOutcome::Refused(RefusalReason::Halted);
        }
        {
            let Some(policy) = self.policies.get_mut(policy_name) else {
                return ReserveOutcome::Refused(RefusalReason::UnknownPolicy(policy_name.clone()));
            };
            expire_abandoned_confirmation(policy, now);
        }
        let policy = self
            .policies
            .get(policy_name)
            .expect("policy existence checked above");
        if policy.escalation_suspended {
            return ReserveOutcome::Refused(RefusalReason::EscalationSuspended(
                policy_name.clone(),
            ));
        }

        if let Some(not_before) = policy_not_before(policy, now) {
            return ReserveOutcome::NotBefore(not_before);
        }

        let confirmation_attempt = match policy.recovery_episode.as_ref() {
            Some(episode) if episode.confirmation_entry.is_some() => {
                return ReserveOutcome::NotBefore(SimInstant::MAX);
            }
            Some(episode) if episode.completed_attempts == 0 => Some(ConfirmationAttempt::First),
            Some(episode) if episode.completed_attempts == 1 => Some(ConfirmationAttempt::Final),
            Some(_) => unreachable!("an exhausted episode is escalation-suspended"),
            None => None,
        };

        let entry_id = self.allocate_entry_id();
        let policy = self
            .policies
            .get_mut(policy_name)
            .expect("policy existence checked above");
        policy.history.push(HistoryEntry {
            id: entry_id,
            at: now,
            kind: EntryKind::LocalReservation,
        });
        if let Some(attempt) = confirmation_attempt {
            policy
                .recovery_episode
                .as_mut()
                .expect("confirmation grants require an active episode")
                .confirmation_entry = Some((entry_id, attempt));
        }
        ReserveOutcome::Reserved(ReservationToken {
            policy: policy_name.clone(),
            entry_id,
            restriction_generation: policy.restriction_generation,
            confirmation_attempt,
            consumed: false,
        })
    }

    /// Removes exactly the undispatched reservation identified by `token`.
    pub fn rollback(&mut self, mut token: ReservationToken) {
        let policy = self
            .policies
            .get_mut(&token.policy)
            .expect("a reservation token always names its originating policy");
        let removed = policy
            .history
            .remove(token.entry_id)
            .expect("a live reservation token always names a history entry");
        assert_eq!(removed.kind, EntryKind::LocalReservation);
        if episode_owns_confirmation(policy, &token) {
            policy
                .recovery_episode
                .as_mut()
                .expect("ownership implies an active episode")
                .confirmation_entry = None;
        }
        token.consume();
    }

    /// Resolves an uncertain dispatched send pessimistically.
    ///
    /// The history entry is intentionally untouched. It remains visible to
    /// every applicable window until simulated time passes that window.
    pub fn on_unknown_outcome(
        &mut self,
        mut token: ReservationToken,
        _now: SimInstant,
    ) -> Transition {
        let entry = self
            .policies
            .get(&token.policy)
            .and_then(|policy| {
                policy
                    .history
                    .entries
                    .iter()
                    .find(|entry| entry.id == token.entry_id)
            })
            .expect("a live reservation token always names a history entry");
        assert_eq!(entry.kind, EntryKind::LocalReservation);
        let confirmation = self.confirmation_is_current(&token);
        let escalated = self.complete_failed_confirmation(&token);
        token.consume();
        Transition::new(
            if escalated {
                Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::RecoveryEscalated,
                }
            } else {
                Disposition::CompleteRequest
            },
            confirmation,
        )
    }

    /// Parses and resolves one ordinary response under the frozen precedence.
    pub fn on_response(
        &mut self,
        mut token: ReservationToken,
        now: SimInstant,
        response: &ObservedResponse,
    ) -> Transition {
        if response.classification == ReplyClassification::CloudflareShaped {
            self.halted = true;
            token.consume();
            return Transition::new(Disposition::Halt, true);
        }

        // A token granted as a confirmation may have been expired by
        // abandonment aging; a stale one flows the ordinary paths below.
        let confirmation = self.confirmation_is_current(&token);

        let observation = match parse_policy(&response.headers) {
            Ok(observation) => observation,
            Err(error) => {
                self.complete_failed_confirmation(&token);
                let disposition = Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::PolicyObservation(error),
                };
                token.consume();
                return Transition::new(disposition, confirmation);
            }
        };
        if let Err(error) = self.validate_observation_target(&token.policy, &observation) {
            self.complete_failed_confirmation(&token);
            let disposition = Disposition::Refuse {
                target: RefusalTarget::Policy(token.policy.clone()),
                cause: RefusalCause::ObservationTarget(error),
            };
            token.consume();
            return Transition::new(disposition, confirmation);
        }

        self.reconcile_observation(&token.policy, now, &observation)
            .expect("the observation target was validated above");

        let disposition = if response.status == StatusCode::TOO_MANY_REQUESTS {
            match parse_retry_after(&response.headers) {
                Ok(retry_after) => {
                    self.record_restriction(&token.policy, now, retry_after);
                    if confirmation {
                        self.escalate_confirmation(&token);
                        Disposition::Refuse {
                            target: RefusalTarget::Policy(token.policy.clone()),
                            cause: RefusalCause::RecoveryEscalated,
                        }
                    } else {
                        self.open_or_join_episode(&token);
                        Disposition::Requeue
                    }
                }
                Err(_) if confirmation => {
                    self.escalate_confirmation(&token);
                    Disposition::Refuse {
                        target: RefusalTarget::Policy(token.policy.clone()),
                        cause: RefusalCause::RecoveryEscalated,
                    }
                }
                Err(error) => Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::RetryAfter(error),
                },
            }
        } else if response.status.is_success() && confirmation {
            self.confirm_recovery(&token);
            Disposition::CompleteRequest
        } else if confirmation && self.complete_failed_confirmation(&token) {
            Disposition::Refuse {
                target: RefusalTarget::Policy(token.policy.clone()),
                cause: RefusalCause::RecoveryEscalated,
            }
        } else {
            Disposition::CompleteRequest
        };
        token.consume();
        Transition::new(disposition, true)
    }

    /// Parses and resolves one non-counting probe response.
    pub fn on_probe_response(
        &mut self,
        endpoint: &EndpointLabel,
        now: SimInstant,
        response: &ObservedResponse,
    ) -> Transition {
        if response.classification == ReplyClassification::CloudflareShaped {
            self.halted = true;
            return Transition::new(Disposition::Halt, true);
        }

        let observation = match parse_policy(&response.headers) {
            Ok(observation) => observation,
            Err(error) => {
                return Transition::new(
                    Disposition::Refuse {
                        target: RefusalTarget::Endpoint(endpoint.clone()),
                        cause: RefusalCause::PolicyObservation(error),
                    },
                    false,
                );
            }
        };
        let policy_name = PolicyName::from(observation.name());
        if let Err(error) = self.validate_observation_target(&policy_name, &observation) {
            return Transition::new(
                Disposition::Refuse {
                    target: RefusalTarget::Endpoint(endpoint.clone()),
                    cause: RefusalCause::ObservationTarget(error),
                },
                false,
            );
        }

        self.reconcile_observation(&policy_name, now, &observation)
            .expect("the observation target was validated above");

        let disposition = if response.status.is_success() {
            Disposition::ProbeReady
        } else if response.status == StatusCode::TOO_MANY_REQUESTS {
            match parse_retry_after(&response.headers) {
                Ok(retry_after) => {
                    self.record_restriction(&policy_name, now, retry_after);
                    self.open_probe_episode(&policy_name);
                    Disposition::ProbeReady
                }
                Err(error) => Disposition::Refuse {
                    target: RefusalTarget::Endpoint(endpoint.clone()),
                    cause: RefusalCause::RetryAfter(error),
                },
            }
        } else {
            Disposition::Refuse {
                target: RefusalTarget::Endpoint(endpoint.clone()),
                cause: RefusalCause::ProbeStatus(response.status),
            }
        };
        Transition::new(disposition, true)
    }

    pub fn on_probe_unknown_outcome(&self, endpoint: &EndpointLabel) -> Transition {
        Transition::new(
            Disposition::Refuse {
                target: RefusalTarget::Endpoint(endpoint.clone()),
                cause: RefusalCause::ProbeUnknownOutcome,
            },
            false,
        )
    }

    /// Whether `token` is still the episode's live confirmation attempt.
    ///
    /// False for ordinary tokens, and for confirmation tokens whose slot was
    /// already resolved by abandonment expiry — those are handled as ordinary
    /// traffic, never as a second confirmation outcome.
    fn confirmation_is_current(&self, token: &ReservationToken) -> bool {
        self.policies
            .get(&token.policy)
            .is_some_and(|policy| episode_owns_confirmation(policy, token))
    }

    fn validate_observation_target(
        &self,
        policy_name: &PolicyName,
        observation: &PolicySnapshot,
    ) -> Result<(), ObservationError> {
        let observed = PolicyName::from(observation.name());
        if policy_name != &observed {
            return Err(ObservationError::PolicyMismatch {
                reserved: policy_name.clone(),
                observed,
            });
        }
        if !self.policies.contains_key(policy_name) {
            return Err(ObservationError::UnknownPolicy(policy_name.clone()));
        }
        Ok(())
    }

    fn record_restriction(
        &mut self,
        policy_name: &PolicyName,
        now: SimInstant,
        retry_after: Duration,
    ) {
        let maximum_bucket = maximum_bucket_resolution(
            self.policies
                .get(policy_name)
                .expect("a valid observation targets a configured policy"),
        );
        let restricted_until = now
            .saturating_add(retry_after)
            .saturating_add(maximum_bucket)
            .saturating_add(RESTRICTION_BUFFER);
        let policy = self
            .policies
            .get_mut(policy_name)
            .expect("a valid observation targets a configured policy");
        policy.restriction_generation = policy
            .restriction_generation
            .checked_add(1)
            .expect("restriction generation space exhausted");
        policy.restricted_until = Some(
            policy
                .restricted_until
                .map_or(restricted_until, |current| current.max(restricted_until)),
        );
    }

    fn open_or_join_episode(&mut self, token: &ReservationToken) {
        let policy = self
            .policies
            .get_mut(&token.policy)
            .expect("a reservation token names a configured policy");
        match policy.recovery_episode.as_ref() {
            // `<`: a pre-restriction concurrent token bounced by the same
            // saturation. `==`: an expired-by-abandonment confirmation whose
            // late 429 arrives after the slot was resolved; it joins rather
            // than escalates because its attempt was already accounted.
            Some(episode) => assert!(
                token.restriction_generation <= episode.opened_generation,
                "a token granted after an episode opened cannot join it as ordinary traffic"
            ),
            None => {
                policy.recovery_episode = Some(RecoveryEpisode {
                    opened_generation: policy.restriction_generation,
                    completed_attempts: 0,
                    confirmation_entry: None,
                });
            }
        }
    }

    fn open_probe_episode(&mut self, policy_name: &PolicyName) {
        let policy = self
            .policies
            .get_mut(policy_name)
            .expect("a valid probe observation targets a configured policy");
        if policy.recovery_episode.is_none() {
            policy.recovery_episode = Some(RecoveryEpisode {
                opened_generation: policy.restriction_generation,
                completed_attempts: 0,
                confirmation_entry: None,
            });
        }
    }

    fn confirm_recovery(&mut self, token: &ReservationToken) {
        let policy = self
            .policies
            .get_mut(&token.policy)
            .expect("a reservation token names a configured policy");
        if !episode_owns_confirmation(policy, token) {
            return;
        }
        policy.recovery_episode = None;
    }

    fn complete_failed_confirmation(&mut self, token: &ReservationToken) -> bool {
        let Some(attempt) = token.confirmation_attempt else {
            return false;
        };
        let policy = self
            .policies
            .get_mut(&token.policy)
            .expect("a reservation token names a configured policy");
        if !episode_owns_confirmation(policy, token) {
            return false;
        }
        let episode = policy
            .recovery_episode
            .as_mut()
            .expect("ownership implies an active episode");
        episode.confirmation_entry = None;
        match attempt {
            ConfirmationAttempt::First => {
                episode.completed_attempts = 1;
                false
            }
            ConfirmationAttempt::Final => {
                policy.escalation_suspended = true;
                true
            }
        }
    }

    fn escalate_confirmation(&mut self, token: &ReservationToken) {
        let policy = self
            .policies
            .get_mut(&token.policy)
            .expect("a reservation token names a configured policy");
        if !episode_owns_confirmation(policy, token) {
            return;
        }
        let episode = policy
            .recovery_episode
            .as_mut()
            .expect("ownership implies an active episode");
        episode.confirmation_entry = None;
        policy.escalation_suspended = true;
    }

    /// Adds pessimistic history with distinct identity and synthetic provenance.
    ///
    /// This seeding primitive keeps arbitrary-history C1/C5 properties
    /// independent of response parsing and reconciliation.
    pub fn record_synthetic(
        &mut self,
        policy_name: &PolicyName,
        now: SimInstant,
        count: usize,
    ) -> Result<(), RefusalReason> {
        if !self.policies.contains_key(policy_name) {
            return Err(RefusalReason::UnknownPolicy(policy_name.clone()));
        }
        let entries = (0..count)
            .map(|_| HistoryEntry {
                id: self.allocate_entry_id(),
                at: now,
                kind: EntryKind::Synthetic,
            })
            .collect::<Vec<_>>();
        let policy = self
            .policies
            .get_mut(policy_name)
            .expect("policy existence checked above");
        policy.history.entries.extend(entries);
        Ok(())
    }

    fn reconcile_observation(
        &mut self,
        policy_name: &PolicyName,
        now: SimInstant,
        observation: &PolicySnapshot,
    ) -> Result<Reconciliation, ObservationError> {
        let policy = self
            .policies
            .get(policy_name)
            .ok_or_else(|| ObservationError::UnknownPolicy(policy_name.clone()))?;

        // Synthesis targets min(reported, largest configured max_hits).
        // Synthesized entries all share the timestamp `now`, so once every
        // configured window is saturated, further entries move no deadline —
        // while an uncapped wire-controlled u32 could materialize gigabytes
        // of history from a single state header.
        let cap = policy
            .rules
            .iter()
            .flat_map(|rule| [rule.pair.burst().max_hits(), rule.pair.sustained().max_hits()])
            .max()
            .expect("policies are constructed with at least one rule");
        let cap = usize::try_from(cap).expect("u32 always fits usize on supported Rust targets");

        let maximum_deficit = observation
            .rules()
            .iter()
            .flat_map(|rule| [rule.state.burst(), rule.state.sustained()])
            .map(|state| {
                let reported = usize::try_from(state.current_hits())
                    .expect("u32 always fits usize on supported Rust targets");
                reported
                    .min(cap)
                    .saturating_sub(policy.history.count_within(now, state.period()))
            })
            .max()
            .unwrap_or(0);

        let entries = (0..maximum_deficit)
            .map(|_| HistoryEntry {
                id: self.allocate_entry_id(),
                at: now,
                kind: EntryKind::Synthetic,
            })
            .collect::<Vec<_>>();
        self.policies
            .get_mut(policy_name)
            .expect("policy existence checked above")
            .history
            .entries
            .extend(entries);

        Ok(Reconciliation {
            synthesized_entries: maximum_deficit,
        })
    }

    fn allocate_entry_id(&mut self) -> EntryId {
        let id = EntryId(self.next_entry_id);
        self.next_entry_id = self
            .next_entry_id
            .checked_add(1)
            .expect("reservation entry id space exhausted");
        id
    }
}

/// Whether the policy's episode still holds `token` as its live confirmation.
fn episode_owns_confirmation(policy: &Policy, token: &ReservationToken) -> bool {
    token.confirmation_attempt.is_some()
        && policy
            .recovery_episode
            .as_ref()
            .and_then(|episode| episode.confirmation_entry)
            .is_some_and(|(id, _)| id == token.entry_id)
}

/// Resolves a confirmation reservation that was abandoned instead of consumed.
///
/// A dropped token is a bug path with emergency semantics: its entry stays
/// counted and ages out by window passage. The confirmation slot must age out
/// on the same clock, or the episode blocks every future grant. Once the
/// entry has left every padded window, the attempt resolves as failed —
/// exactly what an unknown outcome would have recorded.
fn expire_abandoned_confirmation(policy: &mut Policy, now: SimInstant) {
    let Some(episode) = policy.recovery_episode.as_ref() else {
        return;
    };
    let Some((entry_id, attempt)) = episode.confirmation_entry else {
        return;
    };
    let still_active = policy
        .history
        .entries
        .iter()
        .find(|entry| entry.id == entry_id)
        .is_some_and(|entry| {
            policy
                .rules
                .iter()
                .flat_map(|rule| {
                    [
                        (rule.pair.burst(), rule.buckets.burst()),
                        (rule.pair.sustained(), rule.buckets.sustained()),
                    ]
                })
                .any(|(window, resolution)| {
                    is_within_padded(entry.at, now, window.period(), resolution.duration())
                })
        });
    if still_active {
        return;
    }
    let episode = policy
        .recovery_episode
        .as_mut()
        .expect("episode presence checked above");
    episode.confirmation_entry = None;
    match attempt {
        ConfirmationAttempt::First => episode.completed_attempts = 1,
        ConfirmationAttempt::Final => policy.escalation_suspended = true,
    }
}

fn policy_not_before(policy: &Policy, now: SimInstant) -> Option<SimInstant> {
    let history_not_before = policy
        .rules
        .iter()
        .flat_map(|rule| {
            [
                (rule.pair.burst(), rule.buckets.burst()),
                (rule.pair.sustained(), rule.buckets.sustained()),
            ]
        })
        .filter_map(|(window, resolution)| {
            window_not_before(&policy.history, window, resolution, now)
        })
        .max();
    let restriction_not_before = policy.restricted_until.filter(|until| now < *until);
    history_not_before.max(restriction_not_before)
}

fn maximum_bucket_resolution(policy: &Policy) -> Duration {
    policy
        .rules
        .iter()
        .flat_map(|rule| {
            [
                rule.buckets.burst().duration(),
                rule.buckets.sustained().duration(),
            ]
        })
        .max()
        .expect("configured policies have at least one rule")
}

fn parse_retry_after(headers: &HeaderMap) -> Result<Duration, RetryAfterError> {
    let raw = headers
        .get(&RETRY_AFTER_HEADER)
        .ok_or(RetryAfterError::Missing)?
        .to_str()
        .map_err(|_| RetryAfterError::Invalid)?;
    let seconds = raw.parse::<u64>().map_err(|_| RetryAfterError::Invalid)?;
    if seconds > RETRY_AFTER_CAP.as_secs() {
        return Err(RetryAfterError::AboveCap { seconds });
    }
    Ok(Duration::from_secs(seconds))
}

/// Returns the earliest instant that reopens one zero-headroom slot.
///
/// The server's bucket phase is unknowable, so N13 keeps each hit active for
/// its rolling period plus one full, explicitly configured bucket resolution.
/// If history is already over the limit, the order statistic expires only as
/// many oldest hits as are needed to get strictly below `max_hits`.
fn window_not_before(
    history: &History,
    window: &Window,
    resolution: Resolution,
    now: SimInstant,
) -> Option<SimInstant> {
    let max_hits = usize::try_from(window.max_hits()).expect("u32 always fits usize");
    if max_hits == 0 {
        return Some(SimInstant::MAX);
    }

    let mut active = history
        .entries
        .iter()
        .filter(|entry| is_within_padded(entry.at, now, window.period(), resolution.duration()))
        .map(|entry| entry.at)
        .collect::<Vec<_>>();
    if active.len() < max_hits {
        return None;
    }
    active.sort_unstable();
    let entries_that_must_expire = active.len() - max_hits + 1;
    Some(
        active[entries_that_must_expire - 1]
            .saturating_add(window.period())
            .saturating_add(resolution.duration()),
    )
}

fn is_within(at: SimInstant, now: SimInstant, period: Duration) -> bool {
    at > now || now < at.saturating_add(period)
}

fn is_within_padded(
    at: SimInstant,
    now: SimInstant,
    period: Duration,
    resolution: Duration,
) -> bool {
    at > now || now < at.saturating_add(period).saturating_add(resolution)
}
