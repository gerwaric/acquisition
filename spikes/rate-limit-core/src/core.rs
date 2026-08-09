use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::time::Duration;

pub use crate::header::{RulePair, Window};

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
}

impl Policy {
    pub fn new(name: impl Into<PolicyName>, rules: Vec<Rule>) -> Self {
        Self {
            name: name.into(),
            rules,
            history: History::default(),
            restriction_generation: 0,
        }
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
}

#[must_use = "a reservation must be consumed by rollback, on_response, or on_unknown_outcome"]
pub struct ReservationToken {
    policy: PolicyName,
    entry_id: EntryId,
    restriction_generation: u64,
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
    PolicyHasNoRules(PolicyName),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicatePolicy(pub PolicyName);

#[derive(Debug, Default)]
pub struct PolicyEngine {
    policies: HashMap<PolicyName, Policy>,
    next_entry_id: u64,
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

    /// Makes one scheduling decision and records the send on a grant.
    pub fn try_reserve(&mut self, policy_name: &PolicyName, now: SimInstant) -> ReserveOutcome {
        let Some(policy) = self.policies.get(policy_name) else {
            return ReserveOutcome::Refused(RefusalReason::UnknownPolicy(policy_name.clone()));
        };
        if policy.rules.is_empty() {
            return ReserveOutcome::Refused(RefusalReason::PolicyHasNoRules(policy_name.clone()));
        }

        if let Some(not_before) = policy_not_before(policy, now) {
            return ReserveOutcome::NotBefore(not_before);
        }

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
        ReserveOutcome::Reserved(ReservationToken {
            policy: policy_name.clone(),
            entry_id,
            restriction_generation: policy.restriction_generation,
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
        token.consume();
    }

    /// Resolves an uncertain dispatched send pessimistically.
    ///
    /// The history entry is intentionally untouched. It remains visible to
    /// every applicable window until simulated time passes that window.
    pub fn on_unknown_outcome(&mut self, mut token: ReservationToken, _now: SimInstant) {
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
        token.consume();
    }

    /// Adds pessimistic history with distinct identity and synthetic provenance.
    ///
    /// Response reconciliation will decide the deficit count in a later slice;
    /// this primitive keeps C5's identity and rollback behavior testable now.
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

    fn allocate_entry_id(&mut self) -> EntryId {
        let id = EntryId(self.next_entry_id);
        self.next_entry_id = self
            .next_entry_id
            .checked_add(1)
            .expect("reservation entry id space exhausted");
        id
    }
}

fn policy_not_before(policy: &Policy, now: SimInstant) -> Option<SimInstant> {
    policy
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
        .max()
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
