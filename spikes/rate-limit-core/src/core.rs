use std::collections::{HashMap, VecDeque, hash_map::Entry};
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

// Plain-data types carry public fields; see the same note in header.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketModel {
    pub burst: Resolution,
    pub sustained: Resolution,
}

impl BucketModel {
    pub const fn new(burst: Resolution, sustained: Resolution) -> Self {
        Self { burst, sustained }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub pair: RulePair,
    pub buckets: BucketModel,
}

impl Rule {
    pub const fn new(pair: RulePair, buckets: BucketModel) -> Self {
        Self { pair, buckets }
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
    pub id: EntryId,
    pub at: SimInstant,
    pub kind: EntryKind,
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

// Readable state, not settable: instances are only ever reachable by
// reference through Policy, whose fields stay private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryEpisode {
    pub opened_generation: u64,
    pub completed_attempts: u8,
    pub confirmation_entry: Option<(EntryId, ConfirmationAttempt)>,
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
    /// The earliest re-ask time. Always a real, future instant — an actor
    /// may sleep on it.
    NotBefore(SimInstant),
    /// No clock time can be named: a confirmation attempt is in flight, or
    /// no window can ever grant (a zero-hit rule — wire-refused since D8,
    /// so reachable only from constructed policies). Re-ask when
    /// engine state changes, never on a timer. (Tom-approved resolution of
    /// the audit's NotBefore(MAX) sentinel flag, 2026-08-09: an outcome an
    /// actor must not sleep on deserves its own variant, not a magic value.)
    Blocked,
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
    pub synthesized_entries: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppliedObservation {
    state_changed: bool,
    remapped: Option<(PolicyName, PolicyName)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObservationError {
    UnknownPolicy(PolicyName),
    PolicyMismatch {
        reserved: PolicyName,
        observed: PolicyName,
    },
    PolicyCollision {
        observed: PolicyName,
        existing: PolicyName,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplyClassification {
    Normal,
    CloudflareShaped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub classification: ReplyClassification,
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
    // Terminal/suspended state at response time (external review finding 1):
    // a late 429 on a halted engine or suspended policy refuses rather than
    // requeueing into a try_reserve that can only refuse it.
    Halted,
    EscalationSuspended,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalTarget {
    Policy(PolicyName),
    Endpoint(EndpointLabel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Disposition {
    CompleteRequest,
    ProbeReady {
        policy: PolicyName,
    },
    Requeue,
    Refuse {
        target: RefusalTarget,
        cause: RefusalCause,
    },
    Halt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notification {
    Remapped { from: PolicyName, to: PolicyName },
    StateChanged,
}

// Public fields on purpose: the design wants every branch of shell behavior
// reachable from a unit test that just constructs the transition.
#[must_use = "a response transition must be interpreted"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub disposition: Disposition,
    pub notifications: Vec<Notification>,
}

impl Transition {
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

    fn remapped(mut self, from: PolicyName, to: PolicyName) -> Self {
        self.notifications
            .insert(0, Notification::Remapped { from, to });
        self
    }
}

#[derive(Debug)]
pub struct PolicyEngine {
    // Keys are stable policy anchors. The separately routed server-visible
    // name may change after a response announces M5's reactive remap, while
    // in-flight tokens must still remove/reconcile their exact entry.
    policies: HashMap<PolicyName, Policy>,
    policy_routes: HashMap<PolicyName, PolicyName>,
    default_buckets: BucketModel,
    next_entry_id: u64,
    halted: bool,
}

impl PolicyEngine {
    /// Creates an engine whose explicit positional bucket model is applied
    /// to every policy discovered through a valid probe observation.
    pub fn new(default_buckets: BucketModel) -> Self {
        Self {
            policies: HashMap::new(),
            policy_routes: HashMap::new(),
            default_buckets,
            next_entry_id: 0,
            halted: false,
        }
    }

    pub fn insert_policy(&mut self, policy: Policy) -> Result<(), DuplicatePolicy> {
        let name = policy.name.clone();
        if self.policies.contains_key(&name) {
            return Err(DuplicatePolicy(name));
        }
        self.policies.insert(name.clone(), policy);
        self.policy_routes.insert(name.clone(), name);
        Ok(())
    }

    pub fn policy(&self, name: &PolicyName) -> Option<&Policy> {
        self.policy_routes
            .get(name)
            .and_then(|anchor| self.policies.get(anchor))
    }

    pub const fn is_halted(&self) -> bool {
        self.halted
    }

    /// Makes one scheduling decision and records the send on a grant.
    pub fn try_reserve(&mut self, policy_name: &PolicyName, now: SimInstant) -> ReserveOutcome {
        if self.halted {
            return ReserveOutcome::Refused(RefusalReason::Halted);
        }
        let Some(anchor) = self.policy_routes.get(policy_name).cloned() else {
            return ReserveOutcome::Refused(RefusalReason::UnknownPolicy(policy_name.clone()));
        };
        {
            let Some(policy) = self.policies.get_mut(&anchor) else {
                // Constructors and remapping maintain route integrity. This
                // conservative refusal keeps a corrupted route from issuing.
                return ReserveOutcome::Refused(RefusalReason::UnknownPolicy(policy_name.clone()));
            };
            retire_aged_entries(policy, now);
            expire_abandoned_confirmation(policy, now);
        }
        let policy = self
            .policies
            .get(&anchor)
            .expect("policy existence checked above");
        if policy.escalation_suspended {
            return ReserveOutcome::Refused(RefusalReason::EscalationSuspended(
                policy_name.clone(),
            ));
        }

        match policy_deadline(policy, now) {
            WindowDeadline::Open => {}
            WindowDeadline::At(not_before) => {
                return ReserveOutcome::NotBefore(not_before);
            }
            WindowDeadline::Never => return ReserveOutcome::Blocked,
        }

        let confirmation_attempt = match policy.recovery_episode.as_ref() {
            Some(episode) if episode.confirmation_entry.is_some() => {
                return ReserveOutcome::Blocked;
            }
            Some(episode) if episode.completed_attempts == 0 => Some(ConfirmationAttempt::First),
            Some(episode) if episode.completed_attempts == 1 => Some(ConfirmationAttempt::Final),
            Some(_) => unreachable!("an exhausted episode is escalation-suspended"),
            None => None,
        };

        let entry_id = self.allocate_entry_id();
        let policy = self
            .policies
            .get_mut(&anchor)
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
            policy: anchor,
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
        // An absent entry was physically retired by aging — there is
        // nothing left to undo (external review finding 5).
        if let Some(removed) = policy.history.remove(token.entry_id) {
            assert_eq!(removed.kind, EntryKind::LocalReservation);
        }
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
        // The entry may have been physically retired by aging while the
        // request was in flight (external review finding 5) — an aged
        // unknown outcome keeps nothing, so its absence is already the
        // pessimistic end state.
        if let Some(entry) = self.policies.get(&token.policy).and_then(|policy| {
            policy
                .history
                .entries
                .iter()
                .find(|entry| entry.id == token.entry_id)
        }) {
            assert_eq!(entry.kind, EntryKind::LocalReservation);
        }
        let confirmation = self.confirmation_is_current(&token);
        let escalated = self.fail_confirmation(&token, false);
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
            let newly_halted = !self.halted;
            self.halted = true;
            token.consume();
            return Transition::new(Disposition::Halt, newly_halted);
        }

        // A token granted as a confirmation may have been expired by
        // abandonment aging; a stale one flows the ordinary paths below.
        let confirmation = self.confirmation_is_current(&token);

        let observation = match parse_policy(&response.headers) {
            Ok(observation) => observation,
            Err(error) => {
                self.fail_confirmation(&token, false);
                let disposition = Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::PolicyObservation(error),
                };
                token.consume();
                return Transition::new(disposition, confirmation);
            }
        };
        let applied = match self.apply_ordinary_observation(&token.policy, &observation) {
            Ok(applied) => applied,
            Err(error) => {
                self.fail_confirmation(&token, false);
                let disposition = Disposition::Refuse {
                    target: RefusalTarget::Policy(self.current_policy_name(&token.policy)),
                    cause: RefusalCause::ObservationTarget(error),
                };
                token.consume();
                return Transition::new(disposition, confirmation);
            }
        };

        let reconciliation = self
            .reconcile_observation(&token.policy, now, &observation)
            .expect("the observation target was applied above");
        // StateChanged means exactly that: this call mutated engine state.
        // Synthesis, restrictions, and every episode transition set it; a
        // zero-deficit ordinary completion leaves it unset.
        let mut state_changed = applied.state_changed || reconciliation.synthesized_entries > 0;

        let disposition = if response.status == StatusCode::TOO_MANY_REQUESTS {
            state_changed = true;
            // Always record: a usable Retry-After as given, an unusable one
            // at the conservative cap — the server declared a restriction
            // whose length we cannot read, and try_reserve must not send
            // straight back into it.
            let retry_after = parse_retry_after(&response.headers);
            let duration = *retry_after.as_ref().unwrap_or(&RETRY_AFTER_CAP);
            self.record_restriction(&token.policy, now, duration);
            // Terminal/suspended state governs late responses too (external
            // review finding 1): Requeue promises a future send, and a
            // halted engine or suspended policy can never keep it — the
            // request would only bounce off a refusing try_reserve.
            if self.halted {
                Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::Halted,
                }
            } else if self.policy_is_suspended(&token.policy) {
                Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::EscalationSuspended,
                }
            } else if confirmation {
                self.fail_confirmation(&token, true);
                Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::RecoveryEscalated,
                }
            } else {
                match retry_after {
                    Ok(_) => {
                        self.open_or_join_episode(&token);
                        Disposition::Requeue
                    }
                    // No usable Retry-After: the refusal (never an episode)
                    // is the disposition — there is no schedulable retry.
                    Err(error) => Disposition::Refuse {
                        target: RefusalTarget::Policy(token.policy.clone()),
                        cause: RefusalCause::RetryAfter(error),
                    },
                }
            }
        } else if response.status.is_success() && confirmation {
            self.confirm_recovery(&token);
            state_changed = true;
            Disposition::CompleteRequest
        } else if confirmation {
            state_changed = true;
            if self.fail_confirmation(&token, false) {
                Disposition::Refuse {
                    target: RefusalTarget::Policy(token.policy.clone()),
                    cause: RefusalCause::RecoveryEscalated,
                }
            } else {
                Disposition::CompleteRequest
            }
        } else {
            Disposition::CompleteRequest
        };
        token.consume();
        let transition = Transition::new(disposition, state_changed);
        match applied.remapped {
            Some((from, to)) => transition.remapped(from, to),
            None => transition,
        }
    }

    /// Parses and resolves one non-counting probe response.
    pub fn on_probe_response(
        &mut self,
        endpoint: &EndpointLabel,
        now: SimInstant,
        response: &ObservedResponse,
    ) -> Transition {
        if response.classification == ReplyClassification::CloudflareShaped {
            let newly_halted = !self.halted;
            self.halted = true;
            return Transition::new(Disposition::Halt, newly_halted);
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
        let policy_name = PolicyName::from(observation.name.as_str());
        let default_buckets = self.default_buckets;
        let (anchor, seeded) = match self.policy_routes.get(&policy_name).cloned() {
            Some(anchor) => (anchor, false),
            None => match self.policies.entry(policy_name.clone()) {
                Entry::Occupied(_) => {
                    // A previously remapped policy has retained this stable
                    // anchor. Seeing the old name again restores the route;
                    // it does not allocate a second history.
                    self.policy_routes
                        .insert(policy_name.clone(), policy_name.clone());
                    (policy_name.clone(), false)
                }
                Entry::Vacant(slot) => {
                    let rules = observation
                        .rules
                        .iter()
                        .map(|rule| Rule::new(rule.pair.clone(), default_buckets))
                        .collect();
                    // `parse_policy` never returns an empty rules list: an empty
                    // rules header is `InvalidRuleName` (pinned by c2_headers::
                    // invalid_rule_names_are_typed), and the observation is parsed
                    // in this function, so no other construction path exists.
                    let policy = Policy::new(policy_name.clone(), rules)
                        .expect("a valid policy observation contains at least one rule");
                    slot.insert(policy);
                    self.policy_routes
                        .insert(policy_name.clone(), policy_name.clone());
                    (policy_name.clone(), true)
                }
            },
        };
        let applied = match self.apply_ordinary_observation(&anchor, &observation) {
            Ok(applied) => applied,
            Err(error) => {
                return Transition::new(
                    Disposition::Refuse {
                        target: RefusalTarget::Endpoint(endpoint.clone()),
                        cause: RefusalCause::ObservationTarget(error),
                    },
                    seeded,
                );
            }
        };

        let reconciliation = self
            .reconcile_observation(&anchor, now, &observation)
            .expect("the observation target was applied above");
        let mut state_changed =
            seeded || applied.state_changed || reconciliation.synthesized_entries > 0;

        // Valid-429 bookkeeping precedes the disposition choice, as in the
        // ordinary lane (follow-up review 2026-08-10): the server declared a
        // restriction, and a terminal refusal must not discard it — uniform
        // pessimism over minimal mutation. An unusable Retry-After records
        // the conservative cap rather than leaving the policy grantable.
        let retry_after = (response.status == StatusCode::TOO_MANY_REQUESTS)
            .then(|| parse_retry_after(&response.headers));
        if let Some(retry_after) = &retry_after {
            state_changed = true;
            let duration = *retry_after.as_ref().unwrap_or(&RETRY_AFTER_CAP);
            self.record_restriction(&anchor, now, duration);
        }

        // ProbeReady releases parked requests — a promise of future sends,
        // like Requeue — so terminal/suspended state gates it (external
        // review finding 1, probe lane).
        let disposition = if self.halted {
            Disposition::Refuse {
                target: RefusalTarget::Endpoint(endpoint.clone()),
                cause: RefusalCause::Halted,
            }
        } else if self.policy_is_suspended(&anchor) {
            Disposition::Refuse {
                target: RefusalTarget::Endpoint(endpoint.clone()),
                cause: RefusalCause::EscalationSuspended,
            }
        } else if response.status.is_success() {
            Disposition::ProbeReady {
                policy: policy_name.clone(),
            }
        } else if let Some(retry_after) = retry_after {
            match retry_after {
                Ok(_) => {
                    self.open_probe_episode(&anchor);
                    Disposition::ProbeReady {
                        policy: policy_name.clone(),
                    }
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
        Transition::new(disposition, state_changed)
    }

    fn policy_is_suspended(&self, policy_name: &PolicyName) -> bool {
        self.policies
            .get(policy_name)
            .is_some_and(|policy| policy.escalation_suspended)
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

    /// Updates the server-visible name and rule judgments for an existing
    /// stable policy anchor. History is deliberately not rebuilt: M5/M6 say
    /// hits are facts, while names and limits are current judgments.
    fn apply_ordinary_observation(
        &mut self,
        anchor: &PolicyName,
        observation: &PolicySnapshot,
    ) -> Result<AppliedObservation, ObservationError> {
        let observed = PolicyName::from(observation.name.as_str());
        let Some(current) = self.policies.get(anchor).map(|policy| policy.name.clone()) else {
            return Err(ObservationError::UnknownPolicy(anchor.clone()));
        };
        if let Some(existing_anchor) = self.policy_routes.get(&observed)
            && existing_anchor != anchor
        {
            return Err(ObservationError::PolicyCollision {
                observed,
                existing: self.current_policy_name(existing_anchor),
            });
        }

        let remapped = (current != observed).then(|| (current.clone(), observed.clone()));
        if remapped.is_some() {
            self.policy_routes.remove(&current);
            self.policy_routes.insert(observed.clone(), anchor.clone());
        }

        // A response changes the server's rule *pair*, not our established
        // provenance for each bucket resolution. Preserve matching slots;
        // only a newly introduced rule receives the explicit discovery
        // default. This keeps M6's new limits authoritative without silently
        // replacing Known timing knowledge with Assumed timing.
        let prior_buckets = self
            .policies
            .get(anchor)
            .expect("policy existence checked above")
            .rules
            .iter()
            .map(|rule| rule.buckets)
            .collect::<Vec<_>>();
        let rules = observation
            .rules
            .iter()
            .enumerate()
            .map(|(index, rule)| {
                Rule::new(
                    rule.pair.clone(),
                    prior_buckets
                        .get(index)
                        .copied()
                        .unwrap_or(self.default_buckets),
                )
            })
            .collect::<Vec<_>>();
        let policy = self
            .policies
            .get_mut(anchor)
            .expect("policy existence checked above");
        let shape_changed = policy.rules != rules;
        policy.name = observed;
        policy.rules = rules;
        Ok(AppliedObservation {
            state_changed: remapped.is_some() || shape_changed,
            remapped,
        })
    }

    fn current_policy_name(&self, anchor: &PolicyName) -> PolicyName {
        self.policies
            .get(anchor)
            .expect("a reservation token names a configured policy")
            .name
            .clone()
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
            // Any 429 reaching this point joins the existing episode without
            // a generation comparison (external review finding 2 removed the
            // assert here): a pre-restriction concurrent original carries an
            // older generation, while an expired-by-abandonment confirmation
            // can carry a NEWER one than opened_generation — late original
            // 429s advance the policy generation between episode-open and
            // confirmation-grant. Both cases are already accounted (the
            // original by the episode itself, the confirmation by aging
            // expiry), so joining — never escalating, never aborting — is
            // the conservative resolution for every generation.
            Some(_) => {}
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

    /// Resolves the episode's live confirmation as failed.
    ///
    /// `escalate` forces suspension regardless of attempt (the 429 rows of
    /// the confirmation matrix); otherwise a First failure consumes the
    /// attempt and a Final failure suspends. Returns whether this call
    /// suspended the policy. A no-op for ordinary and expired-stale tokens.
    fn fail_confirmation(&mut self, token: &ReservationToken, escalate: bool) -> bool {
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
        if escalate || attempt == ConfirmationAttempt::Final {
            policy.escalation_suspended = true;
            true
        } else {
            episode.completed_attempts = 1;
            false
        }
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
        let anchor = self
            .policy_routes
            .get(policy_name)
            .cloned()
            .ok_or_else(|| RefusalReason::UnknownPolicy(policy_name.clone()))?;
        let entries = (0..count)
            .map(|_| HistoryEntry {
                id: self.allocate_entry_id(),
                at: now,
                kind: EntryKind::Synthetic,
            })
            .collect::<Vec<_>>();
        let policy = self
            .policies
            .get_mut(&anchor)
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
        {
            let policy = self
                .policies
                .get_mut(policy_name)
                .ok_or_else(|| ObservationError::UnknownPolicy(policy_name.clone()))?;
            retire_aged_entries(policy, now);
        }
        let policy = self
            .policies
            .get(policy_name)
            .expect("policy existence checked above");

        // Synthesis targets min(reported, largest configured max_hits).
        // Synthesized entries all share the timestamp `now`, so once every
        // configured window is saturated, further entries move no deadline —
        // while an uncapped wire-controlled u32 could materialize gigabytes
        // of history from a single state header.
        let cap = policy
            .rules
            .iter()
            .flat_map(|rule| [rule.pair.burst().max_hits, rule.pair.sustained().max_hits])
            .max()
            .expect("policies are constructed with at least one rule");
        let cap = usize::try_from(cap).expect("u32 always fits usize on supported Rust targets");

        let maximum_deficit = observation
            .rules
            .iter()
            .flat_map(|rule| [&rule.state.burst, &rule.state.sustained])
            .map(|state| {
                let reported = usize::try_from(state.current_hits)
                    .expect("u32 always fits usize on supported Rust targets");
                reported
                    .min(cap)
                    .saturating_sub(policy.history.count_within(now, state.period))
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

/// Physically retires entries aged out of every padded window.
///
/// External review finding 5: logical expiry was always by window passage,
/// but entries were never removed, so memory and per-call scan cost grew
/// with process lifetime instead of being bounded by the window horizon. A
/// retired entry is unobservable to configured-window scheduling; an
/// *observation* window longer than every configured padded window may
/// count fewer local hits afterward and re-synthesize — the pessimistic
/// direction. Token-consuming paths tolerate a retired entry.
fn retire_aged_entries(policy: &mut Policy, now: SimInstant) {
    let horizon = maximum_padded_window(policy);
    policy
        .history
        .entries
        .retain(|entry| entry.at > now || now < entry.at.saturating_add(horizon));
}

/// The longest period-plus-bucket span across the policy's windows.
fn maximum_padded_window(policy: &Policy) -> Duration {
    policy
        .rules
        .iter()
        .flat_map(|rule| {
            [
                (rule.pair.burst(), rule.buckets.burst),
                (rule.pair.sustained(), rule.buckets.sustained),
            ]
        })
        .map(|(window, resolution)| window.period.saturating_add(resolution.duration()))
        .max()
        .expect("policies are constructed with at least one rule")
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
                        (rule.pair.burst(), rule.buckets.burst),
                        (rule.pair.sustained(), rule.buckets.sustained),
                    ]
                })
                .any(|(window, resolution)| {
                    is_within_padded(entry.at, now, window.period, resolution.duration())
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

/// A window's answer to "when does one slot reopen?", typed so saturated
/// deadline arithmetic can never be mistaken for "never" (external review
/// finding 6 — the former Option<SimInstant> used MAX for both). The derive
/// order is load-bearing: `Open < At(_) < Never`, and `At` compares by
/// instant, so folding with `max` picks the binding answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WindowDeadline {
    Open,
    At(SimInstant),
    Never,
}

fn policy_deadline(policy: &Policy, now: SimInstant) -> WindowDeadline {
    let history_deadline = policy
        .rules
        .iter()
        .flat_map(|rule| {
            [
                (rule.pair.burst(), rule.buckets.burst),
                (rule.pair.sustained(), rule.buckets.sustained),
            ]
        })
        .map(|(window, resolution)| window_deadline(&policy.history, window, resolution, now))
        .max()
        .unwrap_or(WindowDeadline::Open);
    let restriction_deadline = policy
        .restricted_until
        .filter(|until| now < *until)
        .map_or(WindowDeadline::Open, WindowDeadline::At);
    history_deadline.max(restriction_deadline)
}

fn maximum_bucket_resolution(policy: &Policy) -> Duration {
    policy
        .rules
        .iter()
        .flat_map(|rule| {
            [
                rule.buckets.burst.duration(),
                rule.buckets.sustained.duration(),
            ]
        })
        .max()
        .expect("configured policies have at least one rule")
}

/// Delay-seconds only: the RFC 9110 HTTP-date form is deliberately out of
/// model (the API sends bare seconds) and lands in `Invalid`, which now
/// carries the same conservative cap-length restriction as any other
/// unusable value.
fn parse_retry_after(headers: &HeaderMap) -> Result<Duration, RetryAfterError> {
    let value = headers
        .get(&RETRY_AFTER_HEADER)
        .ok_or(RetryAfterError::Missing)?;
    // Byte-gated before any conversion or scan, like every policy header
    // (follow-up review P2). An oversized value is unusable, and unusable
    // values already record the conservative cap.
    if value.as_bytes().len() > crate::header::MAX_HEADER_VALUE_BYTES {
        return Err(RetryAfterError::Invalid);
    }
    let raw = value.to_str().map_err(|_| RetryAfterError::Invalid)?.trim();
    if !crate::header::ascii_digits_only(raw) {
        return Err(RetryAfterError::Invalid);
    }
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
fn window_deadline(
    history: &History,
    window: &Window,
    resolution: Resolution,
    now: SimInstant,
) -> WindowDeadline {
    let max_hits = usize::try_from(window.max_hits).expect("u32 always fits usize");
    if max_hits == 0 {
        // Wire-unreachable (D8 refuses zero-hit limits at parse); defense in
        // depth for constructed policies.
        return WindowDeadline::Never;
    }

    let mut active = history
        .entries
        .iter()
        .filter(|entry| is_within_padded(entry.at, now, window.period, resolution.duration()))
        .map(|entry| entry.at)
        .collect::<Vec<_>>();
    if active.len() < max_hits {
        return WindowDeadline::Open;
    }
    active.sort_unstable();
    let entries_that_must_expire = active.len() - max_hits + 1;
    WindowDeadline::At(
        active[entries_that_must_expire - 1]
            .saturating_add(window.period)
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
