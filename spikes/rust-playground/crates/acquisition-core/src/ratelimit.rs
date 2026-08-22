//! The header-driven rate limiter and the choke point in front of it.
//!
//! Source of truth for every rule here is `docs/design/network-ground-truth.md`,
//! cited by claim number (N…/Q…/P-…). The limiter knows nothing except what
//! GGG's responses told it (invariant 2): the last `X-Rate-Limit-*` headers
//! per policy plus when recent responses arrived. There is no local token
//! counting. The tests at the bottom are the spec — a table of "these
//! headers + this history → wait this long".
//!
//! `ChokePoint` is invariant 1 made structural: it privately owns the
//! workspace's only `reqwest::Client`, so nothing can send a request without
//! first asking the limiter, and every response is observed by it.
//!
//! Endpoints are discovered by a HEAD probe before their first real send
//! (N16's sanctioned pattern; N24: HEADs report the policy without counting),
//! so server-side residue and other tools' hits are known before we add to
//! them. A degraded probe (N20) puts the endpoint in a cooldown.
//!
//! The send-lifetime gate bounds actual daemon-owned HTTP exchanges across
//! API GETs, HEAD probes, and OAuth token requests (P-B/N33). The dispatcher
//! cap remains separate job-scheduling machinery.

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::gate::{SendGate, SendPermit};

/// Server-side timing bucket for a rule's first (initial) window (N12).
pub const INITIAL_BUCKET: Duration = Duration::from_secs(5);
/// Server-side timing bucket for a rule's later (sustained) windows (N12).
pub const SUSTAINED_BUCKET: Duration = Duration::from_secs(60);
/// Extra margin on top of the bucket (N13 says the full bucket is the safe
/// margin; the shipped client adds one more second and has been clean).
pub const BUFFER: Duration = Duration::from_secs(1);
/// A retry always pays the largest timing bucket GGG has named, independent
/// of whether the response's policy headers are usable (D3/N19).
pub const RETRY_BUCKET_PAD: Duration = Duration::from_secs(60);
/// Product policy: longer server-requested pauses are contained as terminal
/// failures rather than silently parking work for an unobserved duration.
pub const RETRY_AFTER_CAP_SECS: u64 = 900;
/// Operational ceiling for any policy/state window period. The largest
/// observed period is 1,800s (N23); a full day leaves ample dynamic-policy
/// headroom while containing values that cannot represent a useful daemon
/// pacing decision (D8/N9).
pub const MAX_POLICY_PERIOD_SECS: u64 = 24 * 60 * 60;
/// Operational ceiling for declared and active restriction durations. The
/// largest observed restriction is 600s (N23); as with periods, a full day
/// contains unexpected input without narrowing any known policy.
pub const MAX_POLICY_RESTRICTION_SECS: u64 = 24 * 60 * 60;

/// Which timing bucket applies to the `index`-th window of a rule.
/// Positional classification (Q4, Tom's hypothesis): the first window is
/// the initial limit, every later one is sustained. A single-window rule is
/// treated as initial. Conservative on every observed policy shape (N23).
pub fn bucket_for(index: usize) -> Duration {
    if index == 0 {
        INITIAL_BUCKET
    } else {
        SUSTAINED_BUCKET
    }
}

/// N33's token policy has one window and therefore cannot be classified by
/// the paired API-policy positional rule. Until GGG confirms its hidden
/// resolution, the frozen policy uses the conservative 60-second bucket.
fn bucket_for_policy(policy: &str, window_count: usize, index: usize) -> Duration {
    if policy == "token-request-limit" && window_count == 1 {
        SUSTAINED_BUCKET
    } else {
        bucket_for(index)
    }
}

// ---- the header contract (N5) ---------------------------------------------

/// One `hits:period:restriction` triplet from `X-Rate-Limit-<rule>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window {
    pub max_hits: u32,
    pub period_secs: u64,
    pub restriction_secs: u64,
}

/// One `current-hits:period:restricted-for` triplet from
/// `X-Rate-Limit-<rule>-State`. Post-increment: includes the request that
/// carried it (N25).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowState {
    pub hits: u32,
    pub period_secs: u64,
    pub restricted_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub limits: Vec<Window>,
    pub state: Vec<WindowState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policy {
    pub name: String,
    pub rules: Vec<Rule>,
}

impl Policy {
    /// Parse one Full N5 header set. Every malformed or partial input is a
    /// value-level error: callers never receive a policy that can only pace
    /// part of the server's counter shape (D8/N20).
    pub fn parse(get: impl Fn(&str) -> Option<String>) -> Result<Policy, PolicyParseError> {
        let name = required(&get, "x-rate-limit-policy")?;
        if name.trim().is_empty() {
            return Err(PolicyParseError::EmptyPolicyName);
        }
        let names = required(&get, "x-rate-limit-rules")?;
        if names.trim().is_empty() {
            return Err(PolicyParseError::EmptyRules);
        }
        let mut rules = Vec::new();
        for raw_rule in names.split(',') {
            let rule = raw_rule.trim();
            if rule.is_empty() {
                return Err(PolicyParseError::EmptyRuleName);
            }
            let key = rule.to_ascii_lowercase();
            let limit_header = format!("x-rate-limit-{key}");
            let state_header = format!("x-rate-limit-{key}-state");
            let limits = parse_limits(&limit_header, &required(&get, &limit_header)?)?;
            let state = parse_state(&state_header, &required(&get, &state_header)?)?;
            if limits.len() != state.len() {
                return Err(PolicyParseError::WindowCountMismatch {
                    rule: rule.to_string(),
                    limits: limits.len(),
                    state: state.len(),
                });
            }
            for (index, (limit, status)) in limits.iter().zip(&state).enumerate() {
                if limit.period_secs != status.period_secs {
                    return Err(PolicyParseError::PeriodMismatch {
                        rule: rule.to_string(),
                        index,
                        limit: limit.period_secs,
                        state: status.period_secs,
                    });
                }
            }
            rules.push(Rule {
                name: rule.to_string(),
                limits,
                state,
            });
        }
        if rules.is_empty() {
            return Err(PolicyParseError::EmptyRules);
        }
        Ok(Policy {
            name: name.trim().to_string(),
            rules,
        })
    }

    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Result<Policy, PolicyParseError> {
        Policy::parse(|name| {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
    }

    /// Ordered rule names and ordered window periods identify which counter
    /// set a policy's local history describes (F65). Limit values may change
    /// dynamically within that shape without invalidating the history.
    fn has_same_shape(&self, other: &Policy) -> bool {
        self.rules.len() == other.rules.len()
            && self.rules.iter().zip(&other.rules).all(|(left, right)| {
                left.name == right.name
                    && left
                        .limits
                        .iter()
                        .map(|w| w.period_secs)
                        .eq(right.limits.iter().map(|w| w.period_secs))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyParseError {
    MissingHeader {
        name: String,
    },
    EmptyPolicyName,
    EmptyRules,
    EmptyRuleName,
    MalformedTriplet {
        header: String,
        raw: String,
    },
    OutOfRangeTriplet {
        header: String,
        raw: String,
    },
    WindowCountMismatch {
        rule: String,
        limits: usize,
        state: usize,
    },
    PeriodMismatch {
        rule: String,
        index: usize,
        limit: u64,
        state: u64,
    },
}

impl fmt::Display for PolicyParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyParseError::MissingHeader { name } => write!(f, "missing {name}"),
            PolicyParseError::EmptyPolicyName => write!(f, "empty X-Rate-Limit-Policy"),
            PolicyParseError::EmptyRules => write!(f, "empty X-Rate-Limit-Rules"),
            PolicyParseError::EmptyRuleName => write!(f, "empty rule name"),
            PolicyParseError::MalformedTriplet { header, raw } => {
                write!(f, "malformed {header} triplet '{raw}'")
            }
            PolicyParseError::OutOfRangeTriplet { header, raw } => {
                write!(f, "out-of-range {header} triplet '{raw}'")
            }
            PolicyParseError::WindowCountMismatch {
                rule,
                limits,
                state,
            } => write!(
                f,
                "rule '{rule}' has {limits} limit windows but {state} state windows"
            ),
            PolicyParseError::PeriodMismatch {
                rule,
                index,
                limit,
                state,
            } => write!(
                f,
                "rule '{rule}' window {index} has limit period {limit} but state period {state}"
            ),
        }
    }
}

impl std::error::Error for PolicyParseError {}

fn required(get: &impl Fn(&str) -> Option<String>, name: &str) -> Result<String, PolicyParseError> {
    get(name).ok_or_else(|| PolicyParseError::MissingHeader {
        name: name.to_string(),
    })
}

fn parse_integer(raw: &str) -> Option<u64> {
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    raw.parse().ok()
}

/// `Retry-After` is retry authority, not part of the Full policy grammar.
/// Keeping this result total and separate lets a malformed policy still
/// install the conservative hold required for a retryable 429 (D8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetryAfter {
    Acceptable { seconds: u64 },
    Missing,
    Malformed { raw: String },
    Negative { raw: String },
    OverCap { raw: String },
}

impl RetryAfter {
    pub fn seconds(&self) -> Option<u64> {
        match self {
            RetryAfter::Acceptable { seconds } => Some(*seconds),
            RetryAfter::Missing
            | RetryAfter::Malformed { .. }
            | RetryAfter::Negative { .. }
            | RetryAfter::OverCap { .. } => None,
        }
    }

    pub fn is_acceptable(&self) -> bool {
        matches!(self, RetryAfter::Acceptable { .. })
    }
}

impl fmt::Display for RetryAfter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RetryAfter::Acceptable { seconds } => write!(f, "{seconds}s"),
            RetryAfter::Missing => write!(f, "missing Retry-After"),
            RetryAfter::Malformed { raw } => write!(f, "malformed Retry-After '{raw}'"),
            RetryAfter::Negative { raw } => write!(f, "negative Retry-After '{raw}'"),
            RetryAfter::OverCap { raw } => write!(
                f,
                "Retry-After '{raw}' exceeds the {RETRY_AFTER_CAP_SECS}s product cap"
            ),
        }
    }
}

pub fn parse_retry_after(get: impl Fn(&str) -> Option<String>) -> RetryAfter {
    let Some(raw) = get("retry-after") else {
        return RetryAfter::Missing;
    };
    let value = raw.trim();
    if value.starts_with('-') {
        return RetryAfter::Negative { raw };
    }
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return RetryAfter::Malformed { raw };
    }
    match value.parse::<u64>() {
        Ok(seconds) if seconds <= RETRY_AFTER_CAP_SECS => RetryAfter::Acceptable { seconds },
        Ok(_) | Err(_) => RetryAfter::OverCap { raw },
    }
}

pub fn retry_after_from_headers(headers: &reqwest::header::HeaderMap) -> RetryAfter {
    parse_retry_after(|name| {
        headers.get(name).map(|value| {
            value
                .to_str()
                .map(str::to_string)
                .unwrap_or_else(|_| "<non-utf8>".into())
        })
    })
}

fn parse_triplet(header: &str, raw: &str) -> Result<[u64; 3], PolicyParseError> {
    let mut fields = raw.trim().split(':');
    let parsed = [fields.next(), fields.next(), fields.next()];
    if fields.next().is_some() || parsed.iter().any(Option::is_none) {
        return Err(PolicyParseError::MalformedTriplet {
            header: header.to_string(),
            raw: raw.to_string(),
        });
    }
    let parsed = parsed.map(|field| field.and_then(|value| parse_integer(value.trim())));
    let [Some(first), Some(second), Some(third)] = parsed else {
        return Err(PolicyParseError::MalformedTriplet {
            header: header.to_string(),
            raw: raw.to_string(),
        });
    };
    Ok([first, second, third])
}

fn parse_limits(header: &str, raw: &str) -> Result<Vec<Window>, PolicyParseError> {
    raw.split(',')
        .map(|triplet| {
            let [max_hits, period_secs, restriction_secs] = parse_triplet(header, triplet)?;
            let max_hits =
                u32::try_from(max_hits).map_err(|_| PolicyParseError::OutOfRangeTriplet {
                    header: header.to_string(),
                    raw: triplet.to_string(),
                })?;
            if max_hits == 0
                || period_secs == 0
                || period_secs > MAX_POLICY_PERIOD_SECS
                || restriction_secs > MAX_POLICY_RESTRICTION_SECS
            {
                return Err(PolicyParseError::OutOfRangeTriplet {
                    header: header.to_string(),
                    raw: triplet.to_string(),
                });
            }
            Ok(Window {
                max_hits,
                period_secs,
                restriction_secs,
            })
        })
        .collect()
}

fn parse_state(header: &str, raw: &str) -> Result<Vec<WindowState>, PolicyParseError> {
    raw.split(',')
        .map(|triplet| {
            let [hits, period_secs, restricted_secs] = parse_triplet(header, triplet)?;
            let hits = u32::try_from(hits).map_err(|_| PolicyParseError::OutOfRangeTriplet {
                header: header.to_string(),
                raw: triplet.to_string(),
            })?;
            if period_secs == 0
                || period_secs > MAX_POLICY_PERIOD_SECS
                || restricted_secs > MAX_POLICY_RESTRICTION_SECS
            {
                return Err(PolicyParseError::OutOfRangeTriplet {
                    header: header.to_string(),
                    raw: triplet.to_string(),
                });
            }
            Ok(WindowState {
                hits,
                period_secs,
                restricted_secs,
            })
        })
        .collect()
}

// ---- the limiter ----------------------------------------------------------

const HISTORY_CAP: usize = 256;
/// How long an endpoint whose probe came back degraded (N20) stays closed
/// before another probe is tried. Login clears it early.
pub const PROBE_COOLDOWN: Duration = Duration::from_secs(60);

/// What the limiter knows about one endpoint (URL path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointState {
    /// Never heard from: needs a probe before the first real send.
    Unknown,
    /// Governed by the named policy.
    Policy(String),
    /// Legacy dashboard state from before N33 established that the OAuth
    /// token endpoint has a policy. Strict observation never creates it.
    Policyless,
    /// The probe failed or came back without a policy (N20); closed until
    /// `until`, then probed again.
    Degraded { until: Instant, reason: String },
}

/// A degraded endpoint, for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DegradedEndpoint {
    pub endpoint: String,
    pub seconds_left: f64,
    pub reason: String,
}

/// What the limiter remembers about one named policy.
struct PolicyState {
    policy: Policy,
    /// Arrival times of counted responses under this policy, oldest first.
    /// Shared by every endpoint that reports the same policy name (N6).
    history: VecDeque<Instant>,
    /// When the Full policy was observed; the base for restriction state.
    last_response: Instant,
    /// The raw headers, for the dashboard.
    raw: serde_json::Value,
    /// A 429 hold is independent of policy validity. It is installed from an
    /// acceptable Retry-After even when the landed policy set is malformed.
    retry_hold_until: Option<Instant>,
    last_retry_after_secs: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyObservationError {
    Parse(PolicyParseError),
    PolicyMismatch {
        established: String,
        observed: String,
    },
}

impl fmt::Display for PolicyObservationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyObservationError::Parse(error) => write!(f, "{error}"),
            PolicyObservationError::PolicyMismatch {
                established,
                observed,
            } => write!(
                f,
                "endpoint is established under policy '{established}' but response reported '{observed}'"
            ),
        }
    }
}

impl std::error::Error for PolicyObservationError {}

struct LandedObservation<'a> {
    policy: Result<Policy, PolicyParseError>,
    retry_after: &'a RetryAfter,
    raw: serde_json::Value,
    counted: bool,
    status: u16,
    now: Instant,
}

#[derive(Default)]
pub struct Limiter {
    policies: HashMap<String, PolicyState>,
    /// Endpoint key (URL path) → what we know, learned from probes and
    /// responses. Absent means `Unknown`.
    endpoints: HashMap<String, EndpointState>,
    /// Every landed 429, including setup failures and stopped/malformed
    /// responses. Bookkeeping happens before caller classification.
    violations: u64,
    /// Route-local fallback holds. A 429 can carry an acceptable Retry-After
    /// while its policy headers are malformed or the route is not established
    /// yet; that must still prevent the requeued job from sending early.
    retry_holds: HashMap<String, (Instant, u64)>,
}

impl Limiter {
    pub fn new() -> Self {
        Limiter::default()
    }

    /// How long to wait before sending to `endpoint`. An unknown endpoint is
    /// normally ready, except when a malformed-policy 429 installed its
    /// route-local Retry-After hold.
    pub fn wait_for(&self, endpoint: &str, now: Instant) -> Duration {
        let policy = self.policy_for(endpoint).and_then(next_safe_send);
        let route = self.retry_holds.get(endpoint).map(|(until, _)| *until);
        policy
            .into_iter()
            .chain(route)
            .max()
            .map(|t| t.saturating_duration_since(now))
            .unwrap_or(Duration::ZERO)
    }

    /// Predicted wait for a request with `ahead` same-policy requests queued
    /// before it. Simulates the pacing rule forward: each simulated send is
    /// appended to a copy of the history, and window hit counts are taken
    /// from that history (a prediction of what the server will report —
    /// headers remain the truth for real sends). An estimate, not a promise.
    pub fn eta_for(&self, endpoint: &str, ahead: u32, now: Instant) -> Duration {
        let Some(state) = self.policy_for(endpoint) else {
            return Duration::ZERO;
        };
        let mut history = state.history.clone();
        let mut t = now.checked_add(self.wait_for(endpoint, now)).unwrap_or(now);
        for _ in 0..ahead {
            history.push_back(t);
            let mut next = t;
            for rule in &state.policy.rules {
                for (i, w) in rule.limits.iter().enumerate() {
                    let period = Duration::from_secs(w.period_secs);
                    let bucket = bucket_for_policy(&state.policy.name, rule.limits.len(), i);
                    let in_window = history
                        .iter()
                        .filter(|&&h| t.duration_since(h) < period)
                        .count();
                    if in_window >= w.max_hits as usize
                        && let Some(deadline) =
                            window_frees_at(&history, t, w.max_hits, period, bucket)
                    {
                        next = next.max(deadline);
                    }
                }
            }
            t = next;
        }
        t.saturating_duration_since(now)
    }

    /// What the limiter knows about `endpoint` right now; an expired
    /// cooldown reads as `Unknown` (time to probe again).
    pub fn endpoint_state(&self, endpoint: &str, now: Instant) -> EndpointState {
        match self.endpoints.get(endpoint) {
            None => EndpointState::Unknown,
            Some(EndpointState::Degraded { until, .. }) if *until <= now => EndpointState::Unknown,
            Some(state) => state.clone(),
        }
    }

    /// Record a probe's outcome: a Full policy teaches the policy without
    /// counting a hit (N24); every partial/malformed set and transport/HTTP
    /// failure closes the endpoint for `PROBE_COOLDOWN` (N20/D8).
    pub fn observe_probe(
        &mut self,
        endpoint: &str,
        outcome: Result<Policy, String>,
        raw: serde_json::Value,
        now: Instant,
    ) {
        let reason = match outcome {
            Ok(policy) => match self.observe(endpoint, Ok(policy), raw, false, now) {
                Ok(()) => return,
                Err(error) => format!("probe observation failed: {error}"),
            },
            Err(e) => format!("probe failed: {e}"),
        };
        self.endpoints.insert(
            endpoint.to_string(),
            EndpointState::Degraded {
                until: now.checked_add(PROBE_COOLDOWN).unwrap_or(now),
                reason,
            },
        );
    }

    /// Frozen D4 behavior for a setup probe that lands as 429. Only a Full
    /// policy plus acceptable Retry-After establishes the endpoint. The HEAD
    /// itself consumes no job retry, but it is a counted violation and the
    /// first ordinary send is held past Retry-After + 60s + 1s.
    pub fn observe_probe_429(
        &mut self,
        endpoint: &str,
        observation: Result<Policy, PolicyParseError>,
        retry_after: &RetryAfter,
        raw: serde_json::Value,
        now: Instant,
    ) -> Result<(), String> {
        self.violations = self.violations.saturating_add(1);
        match (observation, retry_after) {
            (Ok(policy), RetryAfter::Acceptable { .. }) => {
                self.observe_impl(endpoint, Ok(policy), raw, true, now)
                    .map_err(|error| error.to_string())?;
                self.install_retry_hold(endpoint, retry_after, now);
                Ok(())
            }
            (Ok(_), retry_after) => {
                let reason = format!("HEAD returned 429 with {retry_after}");
                self.degrade_for(endpoint, reason.clone(), PROBE_COOLDOWN, now);
                Err(reason)
            }
            (Err(policy_error), retry_after) => {
                let cooldown = retry_after
                    .seconds()
                    .map(Duration::from_secs)
                    .unwrap_or(Duration::ZERO)
                    .max(PROBE_COOLDOWN);
                let reason = format!(
                    "HEAD returned 429 with non-Full policy headers ({policy_error}) and {retry_after}"
                );
                self.degrade_for(endpoint, reason.clone(), cooldown, now);
                Err(reason)
            }
        }
    }

    fn degrade_for(&mut self, endpoint: &str, reason: String, cooldown: Duration, now: Instant) {
        self.endpoints.insert(
            endpoint.to_string(),
            EndpointState::Degraded {
                until: now.checked_add(cooldown).unwrap_or(now),
                reason,
            },
        );
    }

    /// Forget every degraded endpoint (login changed what a probe would
    /// see, so don't make the user sit out the cooldown).
    pub fn forget_degraded(&mut self) {
        self.endpoints
            .retain(|_, st| !matches!(st, EndpointState::Degraded { .. }));
    }

    /// Record a response. `counted` is false for requests the server does
    /// not count against the policy (HEAD probes, N24). A 429 is recorded
    /// as counted — over-estimating the wait is the safe direction.
    pub fn observe(
        &mut self,
        endpoint: &str,
        observation: Result<Policy, PolicyParseError>,
        raw: serde_json::Value,
        counted: bool,
        now: Instant,
    ) -> Result<(), PolicyObservationError> {
        self.observe_impl(endpoint, observation, raw, counted, now)
    }

    /// Record and reconcile one landed ordinary response. Counted history
    /// and 429 violations are recorded before policy parsing/classification;
    /// an acceptable Retry-After then installs its hold regardless of whether
    /// reconciliation succeeds.
    fn observe_landed(
        &mut self,
        endpoint: &str,
        landed: LandedObservation<'_>,
    ) -> Result<(), PolicyObservationError> {
        if landed.status == 429 {
            self.violations = self.violations.saturating_add(1);
        }
        let reconciled = self.observe_impl(
            endpoint,
            landed.policy,
            landed.raw,
            landed.counted,
            landed.now,
        );
        if landed.status == 429 {
            self.install_retry_hold(endpoint, landed.retry_after, landed.now);
        }
        reconciled
    }

    fn observe_impl(
        &mut self,
        endpoint: &str,
        observation: Result<Policy, PolicyParseError>,
        raw: serde_json::Value,
        counted: bool,
        now: Instant,
    ) -> Result<(), PolicyObservationError> {
        let established = match self.endpoints.get(endpoint) {
            Some(EndpointState::Policy(name)) => Some(name.clone()),
            _ => None,
        };

        // The exchange was counted even when its headers are malformed or
        // name a different policy. Attribute that fact to the policy under
        // which the request was admitted, before attempting reconciliation
        // (N25/D8).
        if counted
            && let Some(state) = established
                .as_ref()
                .and_then(|name| self.policies.get_mut(name))
        {
            push_history(&mut state.history, now);
        }

        let policy = observation.map_err(PolicyObservationError::Parse)?;
        if let Some(name) = &established
            && *name != policy.name
        {
            return Err(PolicyObservationError::PolicyMismatch {
                established: name.clone(),
                observed: policy.name,
            });
        }

        let policy_name = policy.name.clone();
        self.endpoints.insert(
            endpoint.to_string(),
            EndpointState::Policy(policy_name.clone()),
        );
        let state = self
            .policies
            .entry(policy_name)
            .or_insert_with(|| PolicyState {
                policy: policy.clone(),
                history: VecDeque::new(),
                last_response: now,
                raw: serde_json::Value::Null,
                retry_hold_until: None,
                last_retry_after_secs: None,
            });

        // A same-name shape transition describes a different counter set
        // (F65): discard incompatible history. Ordinary dynamic limit/state
        // changes within the established shape retain it (N9).
        if !state.policy.has_same_shape(&policy) {
            state.history.clear();
        } else if counted && established.is_none() {
            push_history(&mut state.history, now);
        }
        state.policy = policy;
        state.last_response = now;
        state.raw = raw;
        Ok(())
    }

    fn install_retry_hold(&mut self, endpoint: &str, retry_after: &RetryAfter, now: Instant) {
        let Some(seconds) = retry_after.seconds() else {
            return;
        };
        let Some(until) = checked_deadline(
            now,
            [Duration::from_secs(seconds), RETRY_BUCKET_PAD, BUFFER],
        ) else {
            return;
        };
        self.retry_holds
            .entry(endpoint.to_string())
            .and_modify(|(old_until, old_seconds)| {
                if until > *old_until {
                    *old_until = until;
                    *old_seconds = seconds;
                }
            })
            .or_insert((until, seconds));
        let Some(name) = self.endpoints.get(endpoint).and_then(|state| match state {
            EndpointState::Policy(name) => Some(name.clone()),
            _ => None,
        }) else {
            return;
        };
        let Some(state) = self.policies.get_mut(&name) else {
            return;
        };
        state.retry_hold_until = Some(state.retry_hold_until.map_or(until, |old| old.max(until)));
        state.last_retry_after_secs = Some(seconds);
    }

    pub fn violation_count(&self) -> u64 {
        self.violations
    }

    fn policy_for(&self, endpoint: &str) -> Option<&PolicyState> {
        match self.endpoints.get(endpoint)? {
            EndpointState::Policy(name) => self.policies.get(name),
            _ => None,
        }
    }

    pub fn statuses(&self, now: Instant) -> Vec<PolicyStatus> {
        let mut out: Vec<PolicyStatus> = self
            .policies
            .iter()
            .map(|(name, s)| PolicyStatus {
                policy: name.clone(),
                endpoints: {
                    let mut e: Vec<String> = self
                        .endpoints
                        .iter()
                        .filter(|(_, st)| matches!(st, EndpointState::Policy(p) if p == name))
                        .map(|(k, _)| k.clone())
                        .collect();
                    e.sort();
                    e
                },
                rules: s
                    .policy
                    .rules
                    .iter()
                    .map(|r| RuleStatus {
                        name: r.name.clone(),
                        windows: r
                            .limits
                            .iter()
                            .enumerate()
                            .map(|(i, w)| {
                                let st = r.state.get(i);
                                WindowStatus {
                                    hits: st.map(|s| s.hits).unwrap_or(0),
                                    max_hits: w.max_hits,
                                    period_secs: w.period_secs,
                                    restriction_secs: w.restriction_secs,
                                    restricted_secs: st.map(|s| s.restricted_secs).unwrap_or(0),
                                    bucket_secs: bucket_for_policy(
                                        &s.policy.name,
                                        r.limits.len(),
                                        i,
                                    )
                                    .as_secs(),
                                }
                            })
                            .collect(),
                    })
                    .collect(),
                next_safe_in_seconds: next_safe_send(s)
                    .map(|t| t.saturating_duration_since(now).as_secs_f64())
                    .unwrap_or(0.0),
                last_observed_seconds_ago: now
                    .saturating_duration_since(s.last_response)
                    .as_secs_f64(),
                history_len: s.history.len(),
                retry_after_secs: s
                    .retry_hold_until
                    .filter(|until| *until > now)
                    .and(s.last_retry_after_secs),
                headers: s.raw.clone(),
            })
            .collect();
        out.sort_by(|a, b| a.policy.cmp(&b.policy));
        out
    }

    /// Whether any policy still carries state worth keeping: a pending wait,
    /// or counted hits inside its longest window. The daemon declines to
    /// idle-exit while this is true, so a quick restart doesn't throw away
    /// the history that lets it wait less than a full period (N24).
    pub fn is_live(&self, now: Instant) -> bool {
        self.retry_holds.values().any(|(until, _)| *until > now)
            || self.policies.values().any(|s| {
                if next_safe_send(s).is_some_and(|t| t > now) {
                    return true;
                }
                let longest = s
                    .policy
                    .rules
                    .iter()
                    .flat_map(|r| r.limits.iter())
                    .map(|w| Duration::from_secs(w.period_secs))
                    .max()
                    .unwrap_or(Duration::ZERO);
                s.history
                    .back()
                    .is_some_and(|&h| now.saturating_duration_since(h) < longest)
            })
    }

    /// Endpoints that have answered without any policy header.
    pub fn policyless_endpoints(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .endpoints
            .iter()
            .filter(|(_, st)| **st == EndpointState::Policyless)
            .map(|(k, _)| k.clone())
            .collect();
        v.sort();
        v
    }

    pub fn degraded_endpoints(&self, now: Instant) -> Vec<DegradedEndpoint> {
        let mut v: Vec<DegradedEndpoint> = self
            .endpoints
            .iter()
            .filter_map(|(k, st)| match st {
                EndpointState::Degraded { until, reason } if *until > now => {
                    Some(DegradedEndpoint {
                        endpoint: k.clone(),
                        seconds_left: until.saturating_duration_since(now).as_secs_f64(),
                        reason: reason.clone(),
                    })
                }
                _ => None,
            })
            .collect();
        v.sort_by(|a, b| a.endpoint.cmp(&b.endpoint));
        v
    }
}

fn push_history(history: &mut VecDeque<Instant>, now: Instant) {
    if history.len() >= HISTORY_CAP {
        history.pop_front();
    }
    history.push_back(now);
}

/// When a saturated window frees up, given what we know. Only our hits that
/// are still inside the window (as of `as_of`, the moment the server
/// reported the count) are *known*; the rules are account-scoped (N23), so
/// other tools' hits can be in the server's count without being in our
/// history. Unknown hits are assumed to be the most recent — the window
/// can't free before its oldest hit ages out, and the oldest hit we can
/// name is the latest that oldest hit could possibly be. With no known
/// in-window hits, assume everything just happened.
fn checked_deadline(
    base: Instant,
    durations: impl IntoIterator<Item = Duration>,
) -> Option<Instant> {
    durations
        .into_iter()
        .try_fold(base, |deadline, duration| deadline.checked_add(duration))
}

fn window_frees_at(
    history: &VecDeque<Instant>,
    as_of: Instant,
    max_hits: u32,
    period: Duration,
    bucket: Duration,
) -> Option<Instant> {
    let known: Vec<Instant> = history
        .iter()
        .copied()
        .filter(|&h| as_of.saturating_duration_since(h) < period)
        .collect();
    let oldest = match known.len().checked_sub(max_hits as usize) {
        Some(idx) => known[idx],
        None => known.first().copied().unwrap_or(as_of),
    };
    checked_deadline(oldest, [period, bucket, BUFFER])
}

/// The earliest instant the next request under this policy may be sent,
/// or `None` if it may go now. The pacing rule, per window of each rule:
///
///
/// - restriction active (`restricted-for > 0`): last response, plus the
///   restriction, bucket and buffer;
///
/// - window saturated (`hits >= max`): the oldest hit still in the window,
///   plus period, bucket and buffer (N25: post-increment, 1:1; N13: full
///   bucket on top) — see `window_frees_at` for how hits we didn't make
///   (other tools on the account, N23; residue from before this daemon
///   started, N24) are accounted for;
///
/// - an independently parsed acceptable Retry-After (N19): response time,
///   plus Retry-After, the unconditional 60s pad and buffer. This hold does
///   not depend on a usable policy observation.
///
/// The result is the max over everything that applies.
fn next_safe_send(s: &PolicyState) -> Option<Instant> {
    let mut next = s.retry_hold_until;
    let mut bump = |t: Instant| next = Some(next.map_or(t, |n| n.max(t)));

    for rule in &s.policy.rules {
        for (i, limit) in rule.limits.iter().enumerate() {
            let bucket = bucket_for_policy(&s.policy.name, rule.limits.len(), i);
            let Some(st) = rule.state.get(i) else {
                continue;
            };
            if st.restricted_secs > 0 {
                if let Some(deadline) = checked_deadline(
                    s.last_response,
                    [Duration::from_secs(st.restricted_secs), bucket, BUFFER],
                ) {
                    bump(deadline);
                }
            } else if st.hits >= limit.max_hits
                && let Some(deadline) = window_frees_at(
                    &s.history,
                    s.last_response,
                    limit.max_hits,
                    Duration::from_secs(limit.period_secs),
                    bucket,
                )
            {
                bump(deadline);
            }
        }
    }
    next
}

// ---- dashboard views ------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowStatus {
    pub hits: u32,
    pub max_hits: u32,
    pub period_secs: u64,
    pub restriction_secs: u64,
    pub restricted_secs: u64,
    pub bucket_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleStatus {
    pub name: String,
    pub windows: Vec<WindowStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatus {
    pub policy: String,
    pub endpoints: Vec<String>,
    pub rules: Vec<RuleStatus>,
    pub next_safe_in_seconds: f64,
    pub last_observed_seconds_ago: f64,
    pub history_len: usize,
    pub retry_after_secs: Option<u64>,
    /// The raw `x-rate-limit-*` / `retry-after` headers last seen.
    pub headers: serde_json::Value,
}

/// One HTTP request the choke point actually sent (there is no other way to
/// send one), for the dashboard's sent-requests table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRecord {
    pub seconds_ago: f64,
    pub endpoint: String,
    pub method: String,
    pub url: String,
    /// HTTP status ("200 OK") or the transport error.
    pub outcome: String,
    pub ok: bool,
}

// ---- the choke point ------------------------------------------------------

#[derive(Debug)]
pub enum SendError {
    Transport(String),
    Protocol(PolicyObservationError),
}

/// A landed exchange after its body transfer has resolved. Non-2xx statuses
/// retain their status precedence even when `body` is a transfer failure;
/// clean 2xx transfer failures are returned as `SendError::Transport` before
/// callers can inspect this package (D3/D8).
pub struct CompletedResponse {
    pub status: reqwest::StatusCode,
    pub rate: serde_json::Value,
    pub retry_after: RetryAfter,
    pub body: Result<String, String>,
}

impl fmt::Display for SendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SendError::Transport(error) => write!(f, "{error}"),
            SendError::Protocol(error) => {
                write!(f, "rate-limit protocol failure: {error}")
            }
        }
    }
}

impl std::error::Error for SendError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResponseClassification {
    Success,
    RateLimited(RetryAfter),
    Http(u16),
    Network,
    Protocol(PolicyObservationError),
}

/// D8 precedence in one total function. Observation is already complete when
/// this runs; it can update pacing but never override status/network outcome.
fn classify_response(
    status: Option<u16>,
    network_error: bool,
    observation: Result<(), PolicyObservationError>,
    retry_after: &RetryAfter,
) -> ResponseClassification {
    if status == Some(429) {
        return ResponseClassification::RateLimited(retry_after.clone());
    }
    if let Some(status) = status
        && !(200..300).contains(&status)
    {
        return ResponseClassification::Http(status);
    }
    if network_error {
        return ResponseClassification::Network;
    }
    match observation {
        Ok(()) => ResponseClassification::Success,
        Err(error) => ResponseClassification::Protocol(error),
    }
}

/// Callers name the *route* they're sending on (`character-list`,
/// `stash-list`, …), not the URL: one route covers every league/id variant
/// of a path, so it gets one probe and one policy, and a per-tab route can
/// never turn into a probe per tab (the shape of the 2024 incident, N2).
/// The URL path, for logs.
pub fn url_path(url: &str) -> String {
    url::Url::parse(url)
        .map(|u| u.path().to_string())
        .unwrap_or_else(|_| url.to_string())
}

struct SentAt {
    at: Instant,
    endpoint: String,
    method: &'static str,
    url: String,
    outcome: String,
    ok: bool,
}

const SEND_HISTORY: usize = 100;

pub(crate) trait Clock: Send + Sync {
    fn now(&self) -> Instant;

    fn sleep(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn sleep(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(tokio::time::sleep(duration))
    }
}

pub struct ChokePoint {
    // Private on purpose: this is the only reqwest client in the workspace,
    // so every HTTP request must come through a method that consults the
    // limiter and reports the response back to it.
    http: reqwest::Client,
    limiter: Mutex<Limiter>,
    gate: SendGate,
    sends: Mutex<VecDeque<SentAt>>,
    clock: Arc<dyn Clock>,
}

impl Default for ChokePoint {
    fn default() -> Self {
        ChokePoint::new()
    }
}

impl ChokePoint {
    /// Same construction in mock and real mode: the limiter starts empty and
    /// learns policies from responses.
    pub fn new() -> Self {
        ChokePoint {
            // The user-agent goes on the client itself so no request — token
            // exchange included — can be sent without it (CONTEXT invariant 4).
            http: reqwest::Client::builder()
                .user_agent(crate::provider::USER_AGENT)
                .build()
                .expect("reqwest client builds"),
            limiter: Mutex::new(Limiter::new()),
            gate: SendGate::new(),
            sends: Mutex::new(VecDeque::new()),
            clock: Arc::new(SystemClock),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_clock(clock: Arc<dyn Clock>) -> Self {
        ChokePoint {
            http: reqwest::Client::builder()
                .user_agent(crate::provider::USER_AGENT)
                .build()
                .expect("reqwest client builds"),
            limiter: Mutex::new(Limiter::new()),
            gate: SendGate::new(),
            sends: Mutex::new(VecDeque::new()),
            clock,
        }
    }

    pub(crate) fn now(&self) -> Instant {
        self.clock.now()
    }

    pub(crate) async fn sleep(&self, duration: Duration) {
        self.clock.sleep(duration).await;
    }

    /// Ask whether `route` may send now, or learn how long to wait. This is a
    /// pacing hint for the cancellation-aware dispatcher; every transport
    /// method repeats the final check under a live gate permit.
    pub fn check(&self, route: &str) -> Result<(), Duration> {
        let wait = self.limiter.lock().unwrap().wait_for(route, self.now());
        if wait.is_zero() { Ok(()) } else { Err(wait) }
    }

    pub fn eta_for(&self, route: &str, ahead: u32) -> Duration {
        self.limiter
            .lock()
            .unwrap()
            .eta_for(route, ahead, self.now())
    }

    /// The key under which in-flight requests on `route` must be
    /// serialized: its policy name once known (same-name policies share
    /// counters across routes, N6), else the route itself.
    pub fn serial_key(&self, route: &str) -> String {
        match self
            .limiter
            .lock()
            .unwrap()
            .endpoint_state(route, self.now())
        {
            EndpointState::Policy(name) => name,
            _ => route.to_string(),
        }
    }

    /// Acquire an ordinary actual-send permit and repeat the limiter check
    /// immediately before dispatch. The pre-check keeps rate-limit sleeps
    /// permit-free. Rechecking both the route mapping and limiter state after
    /// admission closes races with landed responses, including N33's
    /// `oauth-token` -> `token-request-limit` discovery transition.
    async fn acquire_send(&self, route: &str) -> SendPermit {
        loop {
            if let Err(wait) = self.check(route) {
                self.sleep(wait.max(Duration::from_millis(50))).await;
                continue;
            }

            let serial_key = self.serial_key(route);
            let permit = self.gate.acquire(serial_key.clone()).await;
            let mapping_is_current = self.serial_key(route) == serial_key;
            match (mapping_is_current, self.check(route)) {
                (true, Ok(())) => return permit,
                (_, pacing) => {
                    drop(permit);
                    if let Err(wait) = pacing {
                        self.sleep(wait.max(Duration::from_millis(50))).await;
                    }
                }
            }
        }
    }

    /// HEAD uses the same permit-free pacing loop, then takes the gate's
    /// exclusive writer reservation. A final check after admission handles a
    /// landed response that installed a hold while the probe was queued.
    async fn acquire_head(&self, route: &str) -> SendPermit {
        loop {
            if let Err(wait) = self.check(route) {
                self.sleep(wait.max(Duration::from_millis(50))).await;
                continue;
            }

            let permit = self.gate.acquire_head().await;
            match self.check(route) {
                Ok(()) => return permit,
                Err(wait) => {
                    drop(permit);
                    self.sleep(wait.max(Duration::from_millis(50))).await;
                }
            }
        }
    }

    pub fn policy_statuses(&self) -> Vec<PolicyStatus> {
        self.limiter.lock().unwrap().statuses(self.now())
    }

    /// Requests holding N4's live send gate and its global bound.
    pub fn actual_send_occupancy(&self) -> (usize, usize) {
        self.gate.occupancy()
    }

    pub fn policyless_endpoints(&self) -> Vec<String> {
        self.limiter.lock().unwrap().policyless_endpoints()
    }

    pub fn is_live(&self) -> bool {
        self.limiter.lock().unwrap().is_live(self.now())
    }

    pub fn endpoint_state(&self, route: &str) -> EndpointState {
        self.limiter
            .lock()
            .unwrap()
            .endpoint_state(route, self.now())
    }

    pub fn degraded_endpoints(&self) -> Vec<DegradedEndpoint> {
        self.limiter.lock().unwrap().degraded_endpoints(self.now())
    }

    pub fn forget_degraded(&self) {
        self.limiter.lock().unwrap().forget_degraded();
    }

    /// Close an endpoint without a probe round-trip (e.g. no session to
    /// probe with). Same cooldown as a failed probe.
    pub fn degrade(&self, route: &str, reason: &str) {
        self.limiter.lock().unwrap().observe_probe(
            route,
            Err(reason.to_string()),
            serde_json::Value::Null,
            self.now(),
        );
    }

    /// The HEAD probe (N16). Not counted by the server (N24); teaches the
    /// limiter the endpoint's policy, or degrades the endpoint (N20).
    /// Returns the raw rate headers on success for the probe job's payload.
    pub async fn head(
        &self,
        route: &str,
        url: &str,
        bearer: Option<&str>,
    ) -> Result<(reqwest::StatusCode, Policy, serde_json::Value), String> {
        let _permit = self.acquire_head(route).await;
        let mut req = self.http.head(url);
        if let Some(token) = bearer {
            req = req.bearer_auth(token);
        }
        let sent = req.send().await.map_err(|error| error.to_string());
        let (policy, retry_after, raw) = sent.as_ref().map_or_else(
            |_| (None, RetryAfter::Missing, serde_json::Value::Null),
            |response| {
                (
                    Some(Policy::from_headers(response.headers())),
                    retry_after_from_headers(response.headers()),
                    rate_limit_snapshot(response.headers()),
                )
            },
        );
        // A HEAD normally has no body, but draining it keeps the exclusive
        // permit live through the complete exchange even for a malformed
        // server response that does carry one.
        let completed = match sent {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.map_err(|error| error.to_string());
                Ok((status, body))
            }
            Err(error) => Err(error),
        };
        let outcome: Result<Policy, String> = match &completed {
            Ok((status, _)) if status.as_u16() == 429 => policy
                .clone()
                .expect("landed responses have a policy parse result")
                .map_err(|error| error.to_string()),
            Ok((status, Ok(_))) if status.is_success() => policy
                .clone()
                .expect("landed responses have a policy parse result")
                .map_err(|error| error.to_string()),
            Ok((status, Err(error))) if status.is_success() => {
                Err(format!("HEAD body transfer failed: {error}"))
            }
            Ok((status, _)) => Err(format!("HEAD returned {status}")),
            Err(error) => Err(format!("HEAD failed: {error}")),
        };
        let now = self.now();
        if completed
            .as_ref()
            .is_ok_and(|(status, _)| status.as_u16() == 429)
        {
            let _ = self.limiter.lock().unwrap().observe_probe_429(
                route,
                policy.expect("a 429 is a landed response"),
                &retry_after,
                raw.clone(),
                now,
            );
        } else {
            self.limiter
                .lock()
                .unwrap()
                .observe_probe(route, outcome.clone(), raw.clone(), now);
        }
        let protocol_failure = match (&completed, &outcome) {
            (Ok((status, Ok(_))), Err(error)) if status.is_success() => Some(error.as_str()),
            _ => None,
        };
        self.record_completed(route, "HEAD", url, &completed, protocol_failure);
        // The limiter decided whether that was good enough; report what it
        // concluded so the probe job's outcome matches the endpoint state.
        match self
            .limiter
            .lock()
            .unwrap()
            .endpoint_state(route, self.now())
        {
            EndpointState::Policy(_) => {
                let Ok(policy) = outcome else {
                    unreachable!("policy state implies a parsed policy")
                };
                Ok((completed.unwrap().0, policy, raw))
            }
            EndpointState::Degraded { reason, .. } => {
                Err(format!("{reason}; endpoint closed for a cooldown"))
            }
            other => Err(format!("unexpected endpoint state after probe: {other:?}")),
        }
    }

    /// Recent sends, newest first.
    pub fn recent_sends(&self) -> Vec<SendRecord> {
        let sends = self.sends.lock().unwrap();
        sends
            .iter()
            .rev()
            .map(|s| SendRecord {
                seconds_ago: self.now().saturating_duration_since(s.at).as_secs_f64(),
                endpoint: s.endpoint.clone(),
                method: s.method.to_string(),
                url: s.url.clone(),
                outcome: s.outcome.clone(),
                ok: s.ok,
            })
            .collect()
    }

    /// Every response comes through here: the limiter learns from it and
    /// the send log records it.
    fn observe(
        &self,
        endpoint: &str,
        result: &Result<reqwest::Response, String>,
        retry_after: &RetryAfter,
        counted: bool,
    ) -> Result<(), PolicyObservationError> {
        if let Ok(r) = result {
            let policy = Policy::from_headers(r.headers());
            let raw = rate_limit_snapshot(r.headers());
            let mut limiter = self.limiter.lock().unwrap();
            let observation = limiter.observe_landed(
                endpoint,
                LandedObservation {
                    policy,
                    retry_after,
                    raw,
                    counted,
                    status: r.status().as_u16(),
                    now: self.now(),
                },
            );
            if let EndpointState::Policy(policy) = limiter.endpoint_state(endpoint, self.now()) {
                // Header discovery happens before body completion. Rekeying
                // the live permit and queued waiters atomically prevents the
                // stable route key and learned policy name from overlapping.
                self.gate.rekey_policy(endpoint, &policy);
            }
            return observation;
        }
        Ok(())
    }

    /// Observation is independent of status classification. Full matching
    /// headers update pacing on every landed response, while a malformed or
    /// mismatched observation becomes a protocol failure only when the HTTP
    /// response would otherwise be a clean success (D8/R6-3).
    async fn finish_send(
        &self,
        endpoint: &str,
        method: &'static str,
        url: &str,
        result: Result<reqwest::Response, String>,
        counted: bool,
    ) -> Result<CompletedResponse, SendError> {
        let retry_after = result.as_ref().map_or(RetryAfter::Missing, |response| {
            retry_after_from_headers(response.headers())
        });
        let observation = self.observe(endpoint, &result, &retry_after, counted);
        let status = result
            .as_ref()
            .ok()
            .map(|response| response.status().as_u16());
        let rate = result
            .as_ref()
            .map(|response| rate_limit_snapshot(response.headers()))
            .unwrap_or(serde_json::Value::Null);
        let result = match result {
            Ok(response) => {
                let status = response.status();
                let body = response.text().await.map_err(|error| error.to_string());
                Ok((status, body))
            }
            Err(error) => Err(error),
        };
        let classification = classify_response(
            status,
            match &result {
                Ok((_, body)) => body.is_err(),
                Err(_) => true,
            },
            observation,
            &retry_after,
        );
        let protocol_failure = match &classification {
            ResponseClassification::Protocol(error) => Some(error.to_string()),
            _ => None,
        };
        self.record_completed(endpoint, method, url, &result, protocol_failure.as_deref());
        match (classification, result) {
            (ResponseClassification::Success, Ok((status, Ok(body))))
            | (ResponseClassification::RateLimited(_), Ok((status, Ok(body))))
            | (ResponseClassification::Http(_), Ok((status, Ok(body)))) => Ok(CompletedResponse {
                status,
                rate,
                retry_after,
                body: Ok(body),
            }),
            (ResponseClassification::RateLimited(_), Ok((status, Err(error))))
            | (ResponseClassification::Http(_), Ok((status, Err(error)))) => {
                Ok(CompletedResponse {
                    status,
                    rate,
                    retry_after,
                    body: Err(error),
                })
            }
            (ResponseClassification::Protocol(error), _) => Err(SendError::Protocol(error)),
            (ResponseClassification::Network, Err(error)) => Err(SendError::Transport(error)),
            (ResponseClassification::Network, Ok((_, Err(error)))) => {
                Err(SendError::Transport(error))
            }
            (classification, result) => {
                unreachable!(
                    "classification {classification:?} disagrees with transport {result:?}"
                )
            }
        }
    }

    fn record_completed(
        &self,
        endpoint: &str,
        method: &'static str,
        url: &str,
        result: &Result<(reqwest::StatusCode, Result<String, String>), String>,
        protocol_failure: Option<&str>,
    ) {
        let (outcome, ok) = match (result, protocol_failure) {
            (_, Some(error)) => (format!("protocol failure: {error}"), false),
            (Ok((status, Err(error))), None) => {
                (format!("{status}; body transfer failure: {error}"), false)
            }
            (Ok((status, Ok(_))), None) => (status.to_string(), status.is_success()),
            (Err(error), None) => (format!("error: {error}"), false),
        };
        let mut sends = self.sends.lock().unwrap();
        if sends.len() >= SEND_HISTORY {
            sends.pop_front();
        }
        sends.push_back(SentAt {
            at: self.now(),
            endpoint: endpoint.to_string(),
            method,
            url: url.to_string(),
            outcome,
            ok,
        });
    }

    /// The one way to send a form POST: waits for the limiter, then sends.
    /// The limiter lock is never held across an await.
    pub async fn post_form(
        &self,
        route: &str,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<CompletedResponse, SendError> {
        let _permit = self.acquire_send(route).await;
        let result = self
            .http
            .post(url)
            .form(params)
            .send()
            .await
            .map_err(|e| e.to_string());
        self.finish_send(route, "POST", url, result, true).await
    }

    /// Bearer-authenticated GET. The dispatcher may pre-wait while the job is
    /// cancellable; this method owns the final limiter check and live permit.
    pub async fn get_bearer(
        &self,
        route: &str,
        url: &str,
        bearer: &str,
    ) -> Result<CompletedResponse, SendError> {
        let _permit = self.acquire_send(route).await;
        let result = self
            .http
            .get(url)
            .bearer_auth(bearer)
            .send()
            .await
            .map_err(|e| e.to_string());
        self.finish_send(route, "GET", url, result, true).await
    }

    /// Unauthenticated GET (mock-only fake data endpoints).
    pub async fn get(&self, route: &str, url: &str) -> Result<CompletedResponse, SendError> {
        let _permit = self.acquire_send(route).await;
        let result = self.http.get(url).send().await.map_err(|e| e.to_string());
        self.finish_send(route, "GET", url, result, true).await
    }
}

/// The `X-Rate-Limit-*` (+ `Retry-After`) headers as a JSON object, for
/// logging, job payloads, and the dashboard.
pub fn rate_limit_snapshot(headers: &reqwest::header::HeaderMap) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (name, value) in headers {
        let name = name.as_str();
        if name.starts_with("x-rate-limit") || name == "retry-after" {
            map.insert(name.into(), value.to_str().unwrap_or("<non-utf8>").into());
        }
    }
    serde_json::Value::Object(map)
}

// ---- the spec: test tables -------------------------------------------------
//
// Each row: a policy's headers as the server sent them, when counted
// responses arrived (seconds before "now"), and the wait the limiter must
// compute. Times are exact because the limiter's arithmetic is; the
// tolerance only absorbs float rounding.

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::{mpsc, oneshot};

    /// A row of the pacing table.
    struct Row {
        name: &'static str,
        claims: &'static str,
        headers: &'static [(&'static str, &'static str)],
        /// Seconds before now at which counted responses arrived, oldest
        /// first. The headers above are taken to be from the last one.
        history: &'static [f64],
        expect_wait: f64,
    }

    const CHAR_LIST: &[(&str, &str)] = &[
        ("x-rate-limit-policy", "character-list-request-limit"),
        ("x-rate-limit-rules", "Account"),
        ("x-rate-limit-account", "2:10:60,5:300:300"),
    ];

    fn with_state(
        base: &[(&'static str, &'static str)],
        state: &'static str,
    ) -> Vec<(&'static str, &'static str)> {
        let mut v = base.to_vec();
        v.push(("x-rate-limit-account-state", state));
        v
    }

    fn parse(headers: &[(&str, &str)]) -> Result<Policy, PolicyParseError> {
        Policy::parse(|k| {
            headers
                .iter()
                .find(|(h, _)| *h == k)
                .map(|(_, v)| v.to_string())
        })
    }

    fn parse_single_window(
        limit_period: u64,
        limit_restriction: u64,
        state_period: u64,
        active_restriction: u64,
    ) -> Result<Policy, PolicyParseError> {
        let limit = format!("1:{limit_period}:{limit_restriction}");
        let state = format!("1:{state_period}:{active_restriction}");
        Policy::parse(|key| match key {
            "x-rate-limit-policy" => Some("bounded-policy".into()),
            "x-rate-limit-rules" => Some("Account".into()),
            "x-rate-limit-account" => Some(limit.clone()),
            "x-rate-limit-account-state" => Some(state.clone()),
            _ => None,
        })
    }

    fn retry(raw: Option<&str>) -> RetryAfter {
        parse_retry_after(|key| {
            (key == "retry-after")
                .then(|| raw.map(str::to_string))
                .flatten()
        })
    }

    async fn serve_one_raw(response: String) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/test", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        (url, task)
    }

    fn raw_response(status: &str, rate_headers: &str, content_length: usize, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\n{rate_headers}Content-Type: application/json\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n{body}"
        )
    }

    fn full_rate_headers(policy: &str, retry_after: Option<u64>) -> String {
        let retry_after = retry_after
            .map(|seconds| format!("Retry-After: {seconds}\r\n"))
            .unwrap_or_default();
        format!(
            "X-Rate-Limit-Policy: {policy}\r\nX-Rate-Limit-Rules: Account\r\nX-Rate-Limit-Account: 2:10:60\r\nX-Rate-Limit-Account-State: 1:10:0\r\n{retry_after}"
        )
    }

    /// Replay a row: feed every history point as a counted response under
    /// the row's policy, then ask for the wait at "now".
    fn run(
        limiter: &mut Limiter,
        endpoint: &str,
        headers: &[(&str, &str)],
        history: &[f64],
        now: Instant,
    ) {
        let policy = parse(headers);
        for &ago in history {
            let at = now - Duration::from_secs_f64(ago);
            limiter
                .observe(endpoint, policy.clone(), serde_json::Value::Null, true, at)
                .expect("table rows carry Full matching policies");
        }
    }

    fn far_future() -> Instant {
        Instant::now() + Duration::from_secs(24 * 3600)
    }

    fn assert_wait(name: &str, got: Duration, expect: f64) {
        let got = got.as_secs_f64();
        assert!(
            (got - expect).abs() < 0.01,
            "{name}: expected wait {expect}s, limiter said {got}s"
        );
    }

    #[test]
    fn n5_header_contract_parses() {
        let p = parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")).expect("policy");
        assert_eq!(p.name, "character-list-request-limit");
        assert_eq!(p.rules.len(), 1);
        let r = &p.rules[0];
        assert_eq!(r.name, "Account");
        assert_eq!(
            r.limits,
            vec![
                Window {
                    max_hits: 2,
                    period_secs: 10,
                    restriction_secs: 60
                },
                Window {
                    max_hits: 5,
                    period_secs: 300,
                    restriction_secs: 300
                },
            ]
        );
        assert_eq!(
            r.state,
            vec![
                WindowState {
                    hits: 1,
                    period_secs: 10,
                    restricted_secs: 0
                },
                WindowState {
                    hits: 1,
                    period_secs: 300,
                    restricted_secs: 0
                },
            ]
        );
    }

    #[test]
    fn retry_after_is_independent_of_full_policy_validity() {
        let mut headers = with_state(CHAR_LIST, "1:10:0,1:300:0");
        headers.push(("retry-after", "not-a-number"));
        assert!(parse(&headers).is_ok(), "Retry-After is not part of Full");
        assert_eq!(
            parse_retry_after(|key| headers
                .iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| value.to_string())),
            RetryAfter::Malformed {
                raw: "not-a-number".into()
            }
        );
    }

    #[test]
    fn full_policy_accepts_observed_one_and_three_window_shapes() {
        let token = parse(&[
            ("x-rate-limit-policy", "token-request-limit"),
            ("x-rate-limit-rules", "Ip"),
            ("x-rate-limit-ip", "60:30:30"),
            ("x-rate-limit-ip-state", "2:30:0"),
        ])
        .unwrap();
        assert_eq!(token.rules[0].limits.len(), 1, "N33");

        let trade = parse(&[
            ("x-rate-limit-policy", "trade-policy"),
            ("x-rate-limit-rules", "Ip"),
            ("x-rate-limit-ip", "5:1:10,20:10:60,100:60:300"),
            ("x-rate-limit-ip-state", "1:1:0,1:10:0,1:60:0"),
        ])
        .unwrap();
        assert_eq!(trade.rules[0].limits.len(), 3, "N30");
    }

    #[test]
    fn d8_full_header_grammar_rejects_every_partial_or_malformed_shape() {
        let cases: &[(&str, &[(&str, &str)])] = &[
            ("missing policy", &[]),
            ("empty policy", &[("x-rate-limit-policy", "  ")]),
            (
                "missing rules",
                &[("x-rate-limit-policy", "character-request-limit")],
            ),
            (
                "empty rules",
                &[("x-rate-limit-policy", "p"), ("x-rate-limit-rules", "")],
            ),
            (
                "empty rule name",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account,"),
                    ("x-rate-limit-account", "2:10:60"),
                    ("x-rate-limit-account-state", "1:10:0"),
                ],
            ),
            (
                "missing rule state",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:10:60"),
                ],
            ),
            (
                "short triplet",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:10"),
                    ("x-rate-limit-account-state", "1:10:0"),
                ],
            ),
            (
                "long triplet",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:10:60:9"),
                    ("x-rate-limit-account-state", "1:10:0"),
                ],
            ),
            (
                "non-numeric triplet",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "two:10:60"),
                    ("x-rate-limit-account-state", "1:10:0"),
                ],
            ),
            (
                "zero limit hits",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "0:10:60"),
                    ("x-rate-limit-account-state", "0:10:0"),
                ],
            ),
            (
                "zero limit period",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:0:60"),
                    ("x-rate-limit-account-state", "1:0:0"),
                ],
            ),
            (
                "zero state period",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:10:60"),
                    ("x-rate-limit-account-state", "1:0:0"),
                ],
            ),
            (
                "hit count outside u32",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "4294967296:10:60"),
                    ("x-rate-limit-account-state", "1:10:0"),
                ],
            ),
            (
                "window-count mismatch",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:10:60,5:300:300"),
                    ("x-rate-limit-account-state", "1:10:0"),
                ],
            ),
            (
                "period mismatch",
                &[
                    ("x-rate-limit-policy", "p"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:10:60"),
                    ("x-rate-limit-account-state", "1:11:0"),
                ],
            ),
        ];

        for (name, headers) in cases {
            assert!(parse(headers).is_err(), "{name} unexpectedly parsed");
        }
    }

    #[test]
    fn policy_deadline_fields_accept_the_operational_bounds() {
        let cases = [
            (
                "limit period",
                MAX_POLICY_PERIOD_SECS,
                0,
                MAX_POLICY_PERIOD_SECS,
                0,
            ),
            ("limit restriction", 1, MAX_POLICY_RESTRICTION_SECS, 1, 0),
            (
                "state period",
                MAX_POLICY_PERIOD_SECS,
                0,
                MAX_POLICY_PERIOD_SECS,
                0,
            ),
            ("active restriction", 1, 0, 1, MAX_POLICY_RESTRICTION_SECS),
        ];
        for (name, limit_period, limit_restriction, state_period, active_restriction) in cases {
            assert!(
                parse_single_window(
                    limit_period,
                    limit_restriction,
                    state_period,
                    active_restriction,
                )
                .is_ok(),
                "{name} maximum was rejected"
            );
        }
    }

    #[test]
    fn policy_deadline_fields_reject_one_above_the_operational_bounds() {
        let cases = [
            (
                "limit period",
                MAX_POLICY_PERIOD_SECS + 1,
                0,
                MAX_POLICY_PERIOD_SECS,
                0,
            ),
            (
                "limit restriction",
                1,
                MAX_POLICY_RESTRICTION_SECS + 1,
                1,
                0,
            ),
            (
                "state period",
                MAX_POLICY_PERIOD_SECS,
                0,
                MAX_POLICY_PERIOD_SECS + 1,
                0,
            ),
            (
                "active restriction",
                1,
                0,
                1,
                MAX_POLICY_RESTRICTION_SECS + 1,
            ),
        ];
        for (name, limit_period, limit_restriction, state_period, active_restriction) in cases {
            assert!(
                matches!(
                    parse_single_window(
                        limit_period,
                        limit_restriction,
                        state_period,
                        active_restriction,
                    ),
                    Err(PolicyParseError::OutOfRangeTriplet { .. })
                ),
                "{name} one-above value did not return PolicyParseError"
            );
        }
    }

    #[test]
    fn observe_and_wait_for_are_total_at_and_above_deadline_bounds() {
        let now = Instant::now();
        let mut limiter = Limiter::new();
        limiter
            .observe(
                "/max-period",
                parse_single_window(
                    MAX_POLICY_PERIOD_SECS,
                    MAX_POLICY_RESTRICTION_SECS,
                    MAX_POLICY_PERIOD_SECS,
                    0,
                ),
                serde_json::Value::Null,
                true,
                now,
            )
            .expect("maximum period remains observable");
        assert!(limiter.wait_for("/max-period", now) > Duration::ZERO);

        limiter
            .observe(
                "/max-restriction",
                parse_single_window(
                    1,
                    MAX_POLICY_RESTRICTION_SECS,
                    1,
                    MAX_POLICY_RESTRICTION_SECS,
                ),
                serde_json::Value::Null,
                true,
                now,
            )
            .expect("maximum active restriction remains observable");
        assert!(limiter.wait_for("/max-restriction", now) > Duration::ZERO);

        let rejected = limiter.observe(
            "/rejected",
            parse_single_window(
                MAX_POLICY_PERIOD_SECS + 1,
                MAX_POLICY_RESTRICTION_SECS + 1,
                MAX_POLICY_PERIOD_SECS + 1,
                MAX_POLICY_RESTRICTION_SECS + 1,
            ),
            serde_json::Value::Null,
            true,
            now,
        );
        assert!(matches!(
            rejected,
            Err(PolicyObservationError::Parse(
                PolicyParseError::OutOfRangeTriplet { .. }
            ))
        ));
        assert_eq!(limiter.wait_for("/rejected", now), Duration::ZERO);
    }

    #[test]
    fn pacing_table() {
        let rows: Vec<Row> = vec![
            Row {
                name: "unknown endpoint sends immediately",
                claims: "no headers seen yet — nothing to obey (HEAD-at-boot is a later step, N16)",
                headers: &[],
                history: &[],
                expect_wait: 0.0,
            },
            Row {
                name: "under both windows",
                claims: "N25 post-increment: 1 of 2 used → go",
                headers: with_state(CHAR_LIST, "1:10:0,1:300:0").leak(),
                history: &[0.0],
                expect_wait: 0.0,
            },
            Row {
                name: "initial window saturated",
                claims: "N25 lookback to the oldest counted hit (3s ago) + 10s period + 5s bucket (N12) + 1s (N13)",
                headers: with_state(CHAR_LIST, "2:10:0,2:300:0").leak(),
                history: &[3.0, 0.0],
                expect_wait: 10.0 + 5.0 + 1.0 - 3.0,
            },
            Row {
                name: "sustained window saturated",
                claims: "oldest of the last 5 (200s ago) + 300s + 60s bucket (N12, second window) + 1s",
                headers: with_state(CHAR_LIST, "1:10:0,5:300:0").leak(),
                history: &[200.0, 150.0, 100.0, 50.0, 0.0],
                expect_wait: 300.0 + 60.0 + 1.0 - 200.0,
            },
            Row {
                name: "both windows saturated → the later one wins",
                claims: "max over saturated windows",
                headers: with_state(CHAR_LIST, "2:10:0,5:300:0").leak(),
                history: &[200.0, 150.0, 100.0, 3.0, 0.0],
                expect_wait: 300.0 + 60.0 + 1.0 - 200.0,
            },
            Row {
                name: "Q4 danger case: stash-list sustained rule has a 60s period",
                claims: "positional classification pads the second window with 60s, not 5s (Q4 hypothesis, N12); the old 75s cutoff would say 66s",
                headers: &[
                    ("x-rate-limit-policy", "stash-list-request-limit"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "10:15:60,30:60:300"),
                    ("x-rate-limit-account-state", "10:15:0,30:60:0"),
                ],
                history: &[
                    29.0, 28.0, 27.0, 26.0, 25.0, 24.0, 23.0, 22.0, 21.0, 20.0, //
                    19.0, 18.0, 17.0, 16.0, 15.0, 14.0, 13.0, 12.0, 11.0, 10.0, //
                    9.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0,
                ],
                // initial: oldest-of-10 (9s ago) + 15 + 5 + 1 = 12s; sustained: 29s ago + 60 + 60 + 1 = 92s
                expect_wait: 60.0 + 60.0 + 1.0 - 29.0,
            },
            Row {
                name: "server-side residue: saturated but history too short",
                claims: "N24 counters persist across restarts; assume unseen hits just happened → last response + period + bucket + 1s",
                headers: with_state(CHAR_LIST, "2:10:0,2:300:0").leak(),
                history: &[0.0],
                expect_wait: 10.0 + 5.0 + 1.0,
            },
            Row {
                name: "shared account: server says 2/2, our in-window history has 1",
                claims: "N23 rules are account-scoped — another tool's hit is in the count but not our history; the unknown hit is assumed recent, so the window frees from our known one: 0 + 10 + 5 + 1",
                headers: with_state(CHAR_LIST, "2:10:0,2:300:0").leak(),
                // Our older hit aged out of the 10s window long ago; the
                // positional lookback would have landed on it and said "go".
                history: &[303.0, 0.0],
                expect_wait: 10.0 + 5.0 + 1.0,
            },
            Row {
                name: "restriction active",
                claims: "restricted-for 30s from the last response + initial bucket + 1s",
                headers: with_state(CHAR_LIST, "2:10:30,2:300:0").leak(),
                history: &[5.0, 0.0],
                expect_wait: 30.0 + 5.0 + 1.0,
            },
            Row {
                name: "N26 observed burst: 15 sends at 0.2s spacing on stash-request-limit",
                claims: "N26: the long-wait send landed at history[max_hits-1] + period + bucket + buffer",
                headers: &[
                    ("x-rate-limit-policy", "stash-request-limit"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "15:10:60,30:300:300"),
                    ("x-rate-limit-account-state", "15:10:0,15:300:0"),
                ],
                history: &[
                    2.8, 2.6, 2.4, 2.2, 2.0, 1.8, 1.6, 1.4, 1.2, 1.0, 0.8, 0.6, 0.4, 0.2, 0.0,
                ],
                expect_wait: 10.0 + 5.0 + 1.0 - 2.8,
            },
        ];

        for row in rows {
            let now = far_future();
            let mut limiter = Limiter::new();
            run(&mut limiter, "/ep", row.headers, row.history, now);
            let got = limiter.wait_for("/ep", now);
            assert_wait(
                &format!("{} [{}]", row.name, row.claims),
                got,
                row.expect_wait,
            );
        }
    }

    #[test]
    fn n6_same_policy_name_shares_counters_across_endpoints() {
        let now = far_future();
        let mut l = Limiter::new();
        let p = parse(&with_state(CHAR_LIST, "2:10:0,2:300:0"));
        l.observe(
            "/character",
            p.clone(),
            serde_json::Value::Null,
            true,
            now - Duration::from_secs(4),
        )
        .unwrap();
        l.observe("/character/pc", p, serde_json::Value::Null, true, now)
            .unwrap();
        // Both endpoints see the same two-event history: oldest 4s ago.
        assert_wait(
            "shared",
            l.wait_for("/character", now),
            10.0 + 5.0 + 1.0 - 4.0,
        );
        assert_wait(
            "shared",
            l.wait_for("/character/pc", now),
            10.0 + 5.0 + 1.0 - 4.0,
        );
    }

    #[test]
    fn n6_n7_different_policies_are_independent() {
        let now = far_future();
        let mut l = Limiter::new();
        l.observe(
            "/character",
            parse(&with_state(CHAR_LIST, "2:10:0,2:300:0")),
            serde_json::Value::Null,
            true,
            now,
        )
        .unwrap();
        let stash = parse(&[
            ("x-rate-limit-policy", "stash-list-request-limit"),
            ("x-rate-limit-rules", "Account"),
            ("x-rate-limit-account", "10:15:60,30:60:300"),
            ("x-rate-limit-account-state", "1:15:0,1:60:0"),
        ]);
        l.observe("/stash/Standard", stash, serde_json::Value::Null, true, now)
            .unwrap();
        assert!(l.wait_for("/character", now) > Duration::ZERO);
        assert_eq!(l.wait_for("/stash/Standard", now), Duration::ZERO);
    }

    #[test]
    fn n24_uncounted_responses_do_not_enter_history() {
        let now = far_future();
        let mut l = Limiter::new();
        let p = parse(&with_state(CHAR_LIST, "0:10:0,0:300:0"));
        l.observe("/character", p, serde_json::Value::Null, false, now)
            .unwrap();
        assert_eq!(l.statuses(now)[0].history_len, 0);
        assert_eq!(l.wait_for("/character", now), Duration::ZERO);
    }

    #[test]
    fn n23_shared_account_with_no_known_in_window_hits() {
        // Our hits are all older than the window; a HEAD at `now` (N24:
        // uncounted) reports the window saturated by someone else. Nothing
        // we know explains the count → assume it all just happened.
        let now = far_future();
        let mut l = Limiter::new();
        let p = parse(&with_state(CHAR_LIST, "2:10:0,2:300:0"));
        l.observe(
            "/character",
            p.clone(),
            serde_json::Value::Null,
            true,
            now - Duration::from_secs(30),
        )
        .unwrap();
        l.observe(
            "/character",
            p.clone(),
            serde_json::Value::Null,
            true,
            now - Duration::from_secs(25),
        )
        .unwrap();
        l.observe("/character", p, serde_json::Value::Null, false, now)
            .unwrap();
        assert_wait(
            "all unknown",
            l.wait_for("/character", now),
            10.0 + 5.0 + 1.0,
        );
    }

    #[test]
    fn is_live_while_history_is_inside_a_window() {
        let now = far_future();
        let mut l = Limiter::new();
        assert!(!l.is_live(now));
        l.observe(
            "/character",
            parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")),
            serde_json::Value::Null,
            true,
            now,
        )
        .unwrap();
        assert!(l.is_live(now));
        // Longest window is 300s: still live at +299s, not at +301s.
        assert!(l.is_live(now + Duration::from_secs(299)));
        assert!(!l.is_live(now + Duration::from_secs(301)));
    }

    #[test]
    fn n9_new_definition_replaces_old() {
        let now = far_future();
        let mut l = Limiter::new();
        l.observe(
            "/character",
            parse(&with_state(CHAR_LIST, "2:10:0,2:300:0")),
            serde_json::Value::Null,
            true,
            now - Duration::from_secs(1),
        )
        .unwrap();
        assert!(l.wait_for("/character", now) > Duration::ZERO);
        // GGG loosens the policy: 4 per 10s now, and we've used 2.
        let loosened = parse(&[
            ("x-rate-limit-policy", "character-list-request-limit"),
            ("x-rate-limit-rules", "Account"),
            ("x-rate-limit-account", "4:10:60,5:300:300"),
            ("x-rate-limit-account-state", "2:10:0,2:300:0"),
        ]);
        l.observe("/character", loosened, serde_json::Value::Null, true, now)
            .unwrap();
        assert_eq!(l.wait_for("/character", now), Duration::ZERO);
    }

    #[test]
    fn eta_simulates_the_pacing_rule_forward() {
        let now = far_future();
        let mut l = Limiter::new();
        // 2 per 10s, both used: this instant and 4s ago.
        run(
            &mut l,
            "/character",
            &with_state(CHAR_LIST, "2:10:0,2:300:0"),
            &[4.0, 0.0],
            now,
        );
        // Head of queue: 10 + 5 + 1 - 4 = 12s.
        assert_wait("head", l.eta_for("/character", 0, now), 12.0);
        // One ahead: the head lands at +12s. By then the hit at `now` is
        // 12s old — already outside the 10s window (the bucket padding is
        // what creates that slack) — so only the head's own hit counts and
        // the next request goes out right behind it. (First draft of this
        // row said 16s; the arithmetic, not the limiter, was wrong.)
        assert_wait("one ahead", l.eta_for("/character", 1, now), 12.0);
        // Two ahead: two hits at +12 fill the window → +12 + 10 + 5 + 1 = 28s.
        assert_wait("two ahead", l.eta_for("/character", 2, now), 28.0);
    }

    #[test]
    fn eta_uses_policy_aware_bucket_selection() {
        let now = far_future();
        let token_headers = [
            ("x-rate-limit-policy", "token-request-limit"),
            ("x-rate-limit-rules", "Ip"),
            ("x-rate-limit-ip", "60:30:30"),
            ("x-rate-limit-ip-state", "1:30:0"),
        ];
        let generic_headers = [
            ("x-rate-limit-policy", "generic-single-window"),
            ("x-rate-limit-rules", "Ip"),
            ("x-rate-limit-ip", "60:30:30"),
            ("x-rate-limit-ip-state", "1:30:0"),
        ];

        let mut limiter = Limiter::new();
        run(&mut limiter, "oauth-token", &token_headers, &[0.0], now);
        assert_wait(
            "N33 token ETA uses the conservative 60s bucket",
            limiter.eta_for("oauth-token", 59, now),
            30.0 + 60.0 + 1.0,
        );

        let mut limiter = Limiter::new();
        run(&mut limiter, "/generic", &generic_headers, &[0.0], now);
        assert_wait(
            "non-token single-window ETA keeps the generic 5s bucket",
            limiter.eta_for("/generic", 59, now),
            30.0 + 5.0 + 1.0,
        );
    }

    #[test]
    fn probes_teach_without_counting_and_degrade_on_n20_shapes() {
        let now = far_future();
        let mut l = Limiter::new();
        assert_eq!(l.endpoint_state("/character", now), EndpointState::Unknown);

        // A good probe: policy learned, nothing counted, state respected.
        l.observe_probe(
            "/character",
            parse(&with_state(CHAR_LIST, "2:10:0,2:300:0")).map_err(|error| error.to_string()),
            serde_json::Value::Null,
            now,
        );
        assert_eq!(
            l.endpoint_state("/character", now),
            EndpointState::Policy("character-list-request-limit".into())
        );
        assert_eq!(l.statuses(now)[0].history_len, 0);
        // Server says 2/2 from hits we never made → assume recent → 16s.
        assert_wait("residue via probe", l.wait_for("/character", now), 16.0);

        // N20 shapes: 2xx but no policy header, or (the Dec-2023 regression)
        // a policy name with no rule definitions → degraded for the cooldown.
        l.observe_probe(
            "/stash",
            parse(&[]).map_err(|error| error.to_string()),
            serde_json::Value::Null,
            now,
        );
        assert!(matches!(
            l.endpoint_state("/stash", now),
            EndpointState::Degraded { .. }
        ));
        l.observe_probe(
            "/stash/x",
            parse(&[("x-rate-limit-policy", "stash-request-limit")])
                .map_err(|error| error.to_string()),
            serde_json::Value::Null,
            now,
        );
        assert!(matches!(
            l.endpoint_state("/stash/x", now),
            EndpointState::Degraded { .. }
        ));
        assert_eq!(l.degraded_endpoints(now).len(), 2);
        assert_eq!(
            l.endpoint_state("/stash", now + PROBE_COOLDOWN),
            EndpointState::Unknown
        );

        // Transport/HTTP failure degrades too; login clears it.
        l.observe_probe(
            "/fetch",
            Err("HEAD returned 401".into()),
            serde_json::Value::Null,
            now,
        );
        assert!(matches!(
            l.endpoint_state("/fetch", now),
            EndpointState::Degraded { .. }
        ));
        l.forget_degraded();
        assert_eq!(l.endpoint_state("/fetch", now), EndpointState::Unknown);
        assert_eq!(
            l.endpoint_state("/character", now),
            EndpointState::Policy("character-list-request-limit".into())
        );
    }

    #[test]
    fn malformed_observation_never_erases_a_known_policy() {
        let now = far_future();
        let mut l = Limiter::new();
        let raw = serde_json::json!({ "source": "established" });
        l.observe(
            "/character",
            parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")),
            raw.clone(),
            true,
            now,
        )
        .unwrap();
        let original = l
            .policies
            .get("character-list-request-limit")
            .unwrap()
            .policy
            .clone();
        let result = l.observe("/character", parse(&[]), serde_json::Value::Null, true, now);
        assert!(matches!(result, Err(PolicyObservationError::Parse(_))));
        assert_eq!(
            l.endpoint_state("/character", now),
            EndpointState::Policy("character-list-request-limit".into())
        );
        let state = l.policies.get("character-list-request-limit").unwrap();
        assert_eq!(state.policy, original);
        assert_eq!(state.raw, raw);
        assert_eq!(state.history.len(), 2);

        let result = l.observe("/token", parse(&[]), serde_json::Value::Null, true, now);
        assert!(matches!(result, Err(PolicyObservationError::Parse(_))));
        assert_eq!(l.endpoint_state("/token", now), EndpointState::Unknown);
    }

    #[test]
    fn mismatched_steady_state_observation_preserves_policy_and_topology() {
        let now = far_future();
        let mut limiter = Limiter::new();
        limiter
            .observe(
                "/character",
                parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")),
                serde_json::json!({ "source": "established" }),
                true,
                now - Duration::from_secs(1),
            )
            .unwrap();
        let original = limiter
            .policies
            .get("character-list-request-limit")
            .unwrap()
            .policy
            .clone();

        let mismatch = parse(&[
            ("x-rate-limit-policy", "renamed-policy"),
            ("x-rate-limit-rules", "Account"),
            ("x-rate-limit-account", "2:10:60,5:300:300"),
            ("x-rate-limit-account-state", "2:10:0,2:300:0"),
        ]);
        assert!(matches!(
            limiter.observe(
                "/character",
                mismatch,
                serde_json::json!({ "source": "mismatch" }),
                true,
                now,
            ),
            Err(PolicyObservationError::PolicyMismatch { .. })
        ));

        assert_eq!(
            limiter.endpoint_state("/character", now),
            EndpointState::Policy("character-list-request-limit".into())
        );
        assert_eq!(limiter.policies.len(), 1);
        let state = limiter
            .policies
            .get("character-list-request-limit")
            .unwrap();
        assert_eq!(state.policy, original);
        assert_eq!(state.raw, serde_json::json!({ "source": "established" }));
        assert_eq!(
            state.history.len(),
            2,
            "the landed response was still counted"
        );
    }

    #[test]
    fn policy_shape_changes_clear_history_but_value_changes_retain_it() {
        let now = far_future();
        let mut limiter = Limiter::new();
        limiter
            .observe(
                "/character",
                parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")),
                serde_json::Value::Null,
                true,
                now - Duration::from_secs(2),
            )
            .unwrap();

        // Same ordered rule names and periods: N9's dynamic values replace
        // the definition without throwing away compatible hit history.
        limiter
            .observe(
                "/character",
                parse(&[
                    ("x-rate-limit-policy", "character-list-request-limit"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "4:10:90,8:300:600"),
                    ("x-rate-limit-account-state", "2:10:0,2:300:0"),
                ]),
                serde_json::Value::Null,
                true,
                now - Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(limiter.statuses(now)[0].history_len, 2);

        // A period change identifies a different counter shape. The just-
        // landed event was admitted under the old shape too, so none of the
        // old history is carried into the replacement (F65).
        limiter
            .observe(
                "/character",
                parse(&[
                    ("x-rate-limit-policy", "character-list-request-limit"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "4:20:90,8:300:600"),
                    ("x-rate-limit-account-state", "1:20:0,1:300:0"),
                ]),
                serde_json::Value::Null,
                true,
                now,
            )
            .unwrap();
        assert_eq!(limiter.statuses(now)[0].history_len, 0);

        let account_only = parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")).unwrap();
        let account_and_ip = parse(&[
            ("x-rate-limit-policy", "character-list-request-limit"),
            ("x-rate-limit-rules", "Account,Ip"),
            ("x-rate-limit-account", "2:10:60,5:300:300"),
            ("x-rate-limit-account-state", "1:10:0,1:300:0"),
            ("x-rate-limit-ip", "4:10:60,10:300:300"),
            ("x-rate-limit-ip-state", "1:10:0,1:300:0"),
        ])
        .unwrap();
        assert!(!account_only.has_same_shape(&account_and_ip));
    }

    #[test]
    fn retry_after_product_vectors_are_total_and_capped() {
        let cases = [
            (None, RetryAfter::Missing),
            (Some("0"), RetryAfter::Acceptable { seconds: 0 }),
            (Some(" 900 "), RetryAfter::Acceptable { seconds: 900 }),
            (Some("soon"), RetryAfter::Malformed { raw: "soon".into() }),
            (Some("-1"), RetryAfter::Negative { raw: "-1".into() }),
            (Some("901"), RetryAfter::OverCap { raw: "901".into() }),
            (
                Some("18446744073709551616"),
                RetryAfter::OverCap {
                    raw: "18446744073709551616".into(),
                },
            ),
        ];
        for (raw, expected) in cases {
            assert_eq!(retry(raw), expected, "Retry-After vector {raw:?}");
        }
    }

    #[test]
    fn retryable_429_records_before_classification_and_holds_despite_bad_policy() {
        let now = far_future();
        let mut limiter = Limiter::new();
        let raw = serde_json::json!({ "source": "established" });
        limiter
            .observe(
                "/character",
                parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")),
                raw.clone(),
                true,
                now - Duration::from_secs(1),
            )
            .unwrap();
        let original = limiter
            .policies
            .get("character-list-request-limit")
            .unwrap()
            .policy
            .clone();
        let retry_after = retry(Some("2"));

        let observation = limiter.observe_landed(
            "/character",
            LandedObservation {
                policy: parse(&[]),
                retry_after: &retry_after,
                raw: serde_json::json!({ "source": "malformed-429" }),
                counted: true,
                status: 429,
                now,
            },
        );
        assert!(matches!(
            classify_response(Some(429), false, observation, &retry_after),
            ResponseClassification::RateLimited(RetryAfter::Acceptable { seconds: 2 })
        ));

        let state = limiter
            .policies
            .get("character-list-request-limit")
            .unwrap();
        assert_eq!(state.history.len(), 2, "the counted 429 is retained");
        assert_eq!(limiter.violation_count(), 1);
        assert_eq!(
            state.policy, original,
            "non-Full headers cannot update policy"
        );
        assert_eq!(
            state.raw, raw,
            "non-Full headers cannot replace observation"
        );
        assert_wait(
            "malformed-policy 429 hold",
            limiter.wait_for("/character", now),
            2.0 + 60.0 + 1.0,
        );
    }

    #[test]
    fn retryable_429_holds_an_unknown_route_when_policy_headers_are_bad() {
        let now = far_future();
        let mut limiter = Limiter::new();
        let retry_after = retry(Some("2"));
        let observation = limiter.observe_landed(
            "oauth-token",
            LandedObservation {
                policy: parse(&[]),
                retry_after: &retry_after,
                raw: serde_json::Value::Null,
                counted: true,
                status: 429,
                now,
            },
        );

        assert!(observation.is_err());
        assert_eq!(
            limiter.endpoint_state("oauth-token", now),
            EndpointState::Unknown
        );
        assert_wait(
            "unknown-route malformed-policy 429 hold",
            limiter.wait_for("oauth-token", now),
            2.0 + 60.0 + 1.0,
        );
        assert_eq!(limiter.violation_count(), 1);
        assert!(limiter.is_live(now));
    }

    #[test]
    fn token_policy_uses_the_frozen_conservative_single_window_bucket() {
        let now = far_future();
        let mut limiter = Limiter::new();
        limiter
            .observe(
                "oauth-token",
                parse(&[
                    ("x-rate-limit-policy", "token-request-limit"),
                    ("x-rate-limit-rules", "Ip"),
                    ("x-rate-limit-ip", "60:30:30"),
                    ("x-rate-limit-ip-state", "60:30:0"),
                ]),
                serde_json::Value::Null,
                true,
                now,
            )
            .unwrap();

        assert_wait(
            "N33 single-window token policy",
            limiter.wait_for("oauth-token", now),
            30.0 + 60.0 + 1.0,
        );
        assert_eq!(limiter.statuses(now)[0].rules[0].windows[0].bucket_secs, 60);
    }

    #[test]
    fn unacceptable_retry_after_vectors_are_terminal_and_install_no_hold() {
        let cases = [
            retry(None),
            retry(Some("soon")),
            retry(Some("-1")),
            retry(Some("901")),
        ];
        for retry_after in cases {
            let now = far_future();
            let mut limiter = Limiter::new();
            limiter
                .observe(
                    "/character",
                    parse(&with_state(CHAR_LIST, "0:10:0,0:300:0")),
                    serde_json::Value::Null,
                    false,
                    now,
                )
                .unwrap();
            let observation = limiter.observe_landed(
                "/character",
                LandedObservation {
                    policy: parse(&[]),
                    retry_after: &retry_after,
                    raw: serde_json::Value::Null,
                    counted: true,
                    status: 429,
                    now,
                },
            );
            let classification = classify_response(Some(429), false, observation, &retry_after);
            assert_eq!(
                classification,
                ResponseClassification::RateLimited(retry_after.clone())
            );
            assert!(!retry_after.is_acceptable());
            assert_eq!(limiter.wait_for("/character", now), Duration::ZERO);
            assert_eq!(limiter.statuses(now)[0].history_len, 1);
            assert_eq!(limiter.violation_count(), 1);
        }
    }

    #[test]
    fn full_head_429_establishes_and_holds_without_a_get_attempt() {
        let now = far_future();
        let mut limiter = Limiter::new();
        let retry_after = retry(Some("0"));
        limiter
            .observe_probe_429(
                "/character",
                parse(&with_state(CHAR_LIST, "0:10:0,0:300:0")),
                &retry_after,
                serde_json::json!({ "retry-after": "0" }),
                now,
            )
            .unwrap();

        assert_eq!(
            limiter.endpoint_state("/character", now),
            EndpointState::Policy("character-list-request-limit".into())
        );
        assert_eq!(limiter.violation_count(), 1);
        assert_eq!(limiter.statuses(now)[0].history_len, 1);
        assert_wait(
            "Full HEAD 429",
            limiter.wait_for("/character", now),
            60.0 + 1.0,
        );
    }

    #[test]
    fn every_unacceptable_full_head_429_is_a_setup_failure() {
        let cases = [
            retry(None),
            retry(Some("soon")),
            retry(Some("-1")),
            retry(Some("901")),
        ];
        for retry_after in cases {
            let now = far_future();
            let mut limiter = Limiter::new();
            let result = limiter.observe_probe_429(
                "/character",
                parse(&with_state(CHAR_LIST, "0:10:0,0:300:0")),
                &retry_after,
                serde_json::Value::Null,
                now,
            );
            assert!(result.is_err(), "{retry_after} unexpectedly established");
            assert!(matches!(
                limiter.endpoint_state("/character", now),
                EndpointState::Degraded { .. }
            ));
            assert!(limiter.statuses(now).is_empty());
            assert_eq!(limiter.violation_count(), 1);
        }
    }

    #[test]
    fn non_full_head_429_is_setup_failure_even_with_acceptable_retry_after() {
        let now = far_future();
        let mut limiter = Limiter::new();
        let retry_after = retry(Some("120"));
        assert!(
            limiter
                .observe_probe_429(
                    "/character",
                    parse(&[]),
                    &retry_after,
                    serde_json::Value::Null,
                    now,
                )
                .is_err()
        );
        let EndpointState::Degraded { until, .. } = limiter.endpoint_state("/character", now)
        else {
            panic!("non-Full HEAD 429 must degrade setup")
        };
        assert_eq!(until.duration_since(now), Duration::from_secs(120));
        assert_eq!(limiter.violation_count(), 1);
    }

    #[test]
    fn response_classification_precedence_is_status_network_then_protocol() {
        let malformed = PolicyObservationError::Parse(PolicyParseError::MissingHeader {
            name: "x-rate-limit-policy".into(),
        });
        let retry_after = retry(Some("3"));
        assert_eq!(
            classify_response(Some(429), true, Err(malformed.clone()), &retry_after),
            ResponseClassification::RateLimited(retry_after.clone())
        );
        assert_eq!(
            classify_response(Some(500), true, Err(malformed.clone()), &retry_after),
            ResponseClassification::Http(500)
        );
        assert_eq!(
            classify_response(Some(200), true, Err(malformed.clone()), &retry_after),
            ResponseClassification::Network
        );
        assert_eq!(
            classify_response(Some(200), false, Err(malformed.clone()), &retry_after),
            ResponseClassification::Protocol(malformed)
        );
        assert_eq!(
            classify_response(Some(204), false, Ok(()), &retry_after),
            ResponseClassification::Success
        );
        assert_eq!(
            classify_response(None, true, Ok(()), &retry_after),
            ResponseClassification::Network
        );
    }

    #[tokio::test]
    async fn full_200_with_truncated_body_is_network_after_observing_policy() {
        let route = "full-truncated";
        let raw = raw_response(
            "200 OK",
            &full_rate_headers("full-truncated-policy", None),
            100,
            "{}",
        );
        let (url, server) = serve_one_raw(raw).await;
        let choke = ChokePoint::new();

        let result = choke.get(route, &url).await;
        assert!(matches!(result, Err(SendError::Transport(_))));
        assert_eq!(
            choke.endpoint_state(route),
            EndpointState::Policy("full-truncated-policy".into()),
            "landed Full headers must update policy before body transfer fails"
        );
        let send = choke.recent_sends().pop().unwrap();
        assert!(!send.ok);
        assert!(send.outcome.contains("body transfer failure"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn malformed_200_with_truncated_body_is_network_and_preserves_policy() {
        let route = "malformed-truncated";
        let choke = ChokePoint::new();
        let established_raw = serde_json::json!({ "source": "established" });
        choke
            .limiter
            .lock()
            .unwrap()
            .observe(
                route,
                parse(&with_state(CHAR_LIST, "1:10:0,1:300:0")),
                established_raw.clone(),
                false,
                Instant::now(),
            )
            .unwrap();
        let (url, server) = serve_one_raw(raw_response("200 OK", "", 100, "{}")).await;

        let result = choke.get(route, &url).await;
        assert!(matches!(result, Err(SendError::Transport(_))));
        let status = choke.policy_statuses().pop().unwrap();
        assert_eq!(status.headers, established_raw);
        assert_eq!(
            choke.endpoint_state(route),
            EndpointState::Policy("character-list-request-limit".into())
        );
        server.await.unwrap();
    }

    #[tokio::test]
    async fn complete_malformed_200_is_protocol() {
        let route = "malformed-complete";
        let (url, server) = serve_one_raw(raw_response("200 OK", "", 2, "{}")).await;
        let choke = ChokePoint::new();

        assert!(matches!(
            choke.get(route, &url).await,
            Err(SendError::Protocol(_))
        ));
        let send = choke.recent_sends().pop().unwrap();
        assert!(!send.ok);
        assert!(send.outcome.contains("protocol failure"));
        server.await.unwrap();
    }

    #[tokio::test]
    async fn truncated_429_and_500_keep_status_and_record_transfer_evidence() {
        for (status_line, status, retry_after) in [
            ("429 Too Many Requests", 429, Some(0)),
            ("500 Internal Server Error", 500, None),
        ] {
            let route = format!("truncated-{status}");
            let raw = raw_response(
                status_line,
                &full_rate_headers(&format!("policy-{status}"), retry_after),
                100,
                "{}",
            );
            let (url, server) = serve_one_raw(raw).await;
            let choke = ChokePoint::new();

            let response = choke
                .get(&route, &url)
                .await
                .expect("non-2xx status keeps precedence over body failure");
            assert_eq!(response.status.as_u16(), status);
            assert!(response.body.is_err());
            let send = choke.recent_sends().pop().unwrap();
            assert!(!send.ok);
            assert!(send.outcome.starts_with(status_line));
            assert!(send.outcome.contains("body transfer failure"));
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn common_gate_caps_gets_until_response_bodies_complete() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let (arrived_tx, mut arrived_rx) = mpsc::unbounded_channel();
        let mut release_senders = Vec::new();
        let mut release_receivers = Vec::new();
        for _ in 0..3 {
            let (sender, receiver) = oneshot::channel();
            release_senders.push(Some(sender));
            release_receivers.push(receiver);
        }
        let server = tokio::spawn(async move {
            let mut handlers = Vec::new();
            for (index, release) in release_receivers.into_iter().enumerate() {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut request = [0; 4096];
                let _ = stream.read(&mut request).await.unwrap();
                arrived_tx.send(index).unwrap();
                handlers.push(tokio::spawn(async move {
                    let headers = full_rate_headers(&format!("body-policy-{index}"), None);
                    let response = raw_response("200 OK", &headers, 2, "");
                    stream.write_all(response.as_bytes()).await.unwrap();
                    release.await.unwrap();
                    stream.write_all(b"{}").await.unwrap();
                }));
            }
            for handler in handlers {
                handler.await.unwrap();
            }
        });

        let choke = Arc::new(ChokePoint::new());
        let sends: Vec<_> = (0..3)
            .map(|index| {
                let choke = choke.clone();
                let url = format!("{base}/{index}");
                tokio::spawn(async move { choke.get(&format!("route-{index}"), &url).await })
            })
            .collect();

        let first = arrived_rx.recv().await.unwrap();
        let second = arrived_rx.recv().await.unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(50), arrived_rx.recv())
                .await
                .is_err(),
            "a third actual send bypassed the global cap"
        );
        release_senders[first].take().unwrap().send(()).unwrap();
        let third = tokio::time::timeout(Duration::from_secs(1), arrived_rx.recv())
            .await
            .expect("a body completion releases the permit")
            .unwrap();
        for index in [second, third] {
            release_senders[index].take().unwrap().send(()).unwrap();
        }
        for send in sends {
            send.await.unwrap().unwrap();
        }
        server.await.unwrap();
    }

    #[tokio::test]
    async fn head_waits_exclusively_for_an_incomplete_get_body() {
        let get_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let get_url = format!("http://{}/get", get_listener.local_addr().unwrap());
        let (get_arrived_tx, get_arrived) = oneshot::channel();
        let (release_get, release_get_rx) = oneshot::channel();
        let get_server = tokio::spawn(async move {
            let (mut stream, _) = get_listener.accept().await.unwrap();
            let mut request = [0; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            let response =
                raw_response("200 OK", &full_rate_headers("ordinary-policy", None), 2, "");
            stream.write_all(response.as_bytes()).await.unwrap();
            get_arrived_tx.send(()).unwrap();
            release_get_rx.await.unwrap();
            stream.write_all(b"{}").await.unwrap();
        });

        let head_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let head_url = format!("http://{}/head", head_listener.local_addr().unwrap());
        let (head_arrived_tx, head_arrived) = oneshot::channel();
        let head_server = tokio::spawn(async move {
            let (mut stream, _) = head_listener.accept().await.unwrap();
            let request = crate::mockggg::read_request(&mut stream).await.unwrap();
            assert_eq!(request.method, "HEAD");
            head_arrived_tx.send(()).unwrap();
            let response = raw_response(
                "204 No Content",
                &full_rate_headers("head-policy", None),
                0,
                "",
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        let choke = Arc::new(ChokePoint::new());
        let get = {
            let choke = choke.clone();
            tokio::spawn(async move { choke.get("ordinary-route", &get_url).await })
        };
        get_arrived.await.unwrap();
        let head = {
            let choke = choke.clone();
            tokio::spawn(async move { choke.head("head-route", &head_url, None).await })
        };
        let mut head_arrived = Box::pin(head_arrived);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut head_arrived)
                .await
                .is_err(),
            "HEAD overlapped an ordinary response body"
        );
        release_get.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(1), &mut head_arrived)
            .await
            .expect("HEAD starts after the ordinary permit drains")
            .unwrap();

        get.await.unwrap().unwrap();
        head.await.unwrap().unwrap();
        get_server.await.unwrap();
        head_server.await.unwrap();
    }

    #[tokio::test]
    async fn waiting_head_has_writer_preference_over_later_mixed_policy_send() {
        async fn held_get(
            policy: &'static str,
        ) -> (
            String,
            oneshot::Receiver<()>,
            oneshot::Sender<()>,
            tokio::task::JoinHandle<()>,
        ) {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/get", listener.local_addr().unwrap());
            let (arrived_tx, arrived) = oneshot::channel();
            let (release, release_rx) = oneshot::channel();
            let server = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.unwrap();
                let request = crate::mockggg::read_request(&mut stream).await.unwrap();
                assert_eq!(request.method, "GET");
                let response = raw_response("200 OK", &full_rate_headers(policy, None), 2, "");
                stream.write_all(response.as_bytes()).await.unwrap();
                arrived_tx.send(()).unwrap();
                release_rx.await.unwrap();
                stream.write_all(b"{}").await.unwrap();
            });
            (url, arrived, release, server)
        }

        let (first_url, first_arrived, release_first, first_server) =
            held_get("writer-first-policy").await;
        let (second_url, second_arrived, release_second, second_server) =
            held_get("writer-second-policy").await;
        let choke = Arc::new(ChokePoint::new());
        let first = {
            let choke = choke.clone();
            tokio::spawn(async move { choke.get("writer-first-route", &first_url).await })
        };
        let second = {
            let choke = choke.clone();
            tokio::spawn(async move { choke.get("writer-second-route", &second_url).await })
        };
        first_arrived.await.unwrap();
        second_arrived.await.unwrap();

        let canceled_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let canceled_url = format!("http://{}/get", canceled_listener.local_addr().unwrap());
        let canceled = {
            let choke = choke.clone();
            tokio::spawn(async move { choke.get("writer-canceled-route", &canceled_url).await })
        };
        let mut canceled_accept = Box::pin(canceled_listener.accept());
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut canceled_accept)
                .await
                .is_err(),
            "ordinary send bypassed the full gate before cancellation"
        );
        canceled.abort();
        match canceled.await {
            Err(error) => assert!(error.is_cancelled()),
            Ok(_) => panic!("canceled gate waiter completed normally"),
        }
        drop(canceled_accept);

        let head_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let head_url = format!("http://{}/head", head_listener.local_addr().unwrap());
        let (head_arrived_tx, head_arrived) = oneshot::channel();
        let (release_head, release_head_rx) = oneshot::channel();
        let head_server = tokio::spawn(async move {
            let (mut stream, _) = head_listener.accept().await.unwrap();
            let request = crate::mockggg::read_request(&mut stream).await.unwrap();
            assert_eq!(request.method, "HEAD");
            head_arrived_tx.send(()).unwrap();
            release_head_rx.await.unwrap();
            let response = raw_response(
                "204 No Content",
                &full_rate_headers("writer-head-policy", None),
                0,
                "",
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });
        let head = {
            let choke = choke.clone();
            tokio::spawn(async move { choke.head("writer-head-route", &head_url, None).await })
        };
        let mut head_arrived = Box::pin(head_arrived);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut head_arrived)
                .await
                .is_err(),
            "HEAD overlapped the two live ordinary bodies"
        );

        let (later_url, later_arrived, release_later, later_server) =
            held_get("writer-later-policy").await;
        let later = {
            let choke = choke.clone();
            tokio::spawn(async move { choke.get("writer-later-route", &later_url).await })
        };
        let mut later_arrived = Box::pin(later_arrived);
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut later_arrived)
                .await
                .is_err(),
            "later ordinary send bypassed the full gate"
        );

        release_first.send(()).unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut head_arrived)
                .await
                .is_err(),
            "exclusive HEAD started before every ordinary permit drained"
        );
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut later_arrived)
                .await
                .is_err(),
            "later ordinary send bypassed a waiting HEAD writer"
        );

        release_second.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(1), &mut head_arrived)
            .await
            .expect("HEAD starts when the last ordinary body completes")
            .unwrap();
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut later_arrived)
                .await
                .is_err(),
            "later ordinary send overlapped the live HEAD writer"
        );

        release_head.send(()).unwrap();
        head.await.unwrap().unwrap();
        tokio::time::timeout(Duration::from_secs(1), &mut later_arrived)
            .await
            .expect("ordinary send resumes after the HEAD writer completes")
            .unwrap();
        release_later.send(()).unwrap();

        first.await.unwrap().unwrap();
        second.await.unwrap().unwrap();
        later.await.unwrap().unwrap();
        first_server.await.unwrap();
        second_server.await.unwrap();
        head_server.await.unwrap();
        later_server.await.unwrap();
    }

    #[test]
    fn url_path_strips_host() {
        assert_eq!(
            url_path("https://api.pathofexile.com/stash/Standard"),
            "/stash/Standard"
        );
        assert_eq!(url_path("http://127.0.0.1:5555/character"), "/character");
    }
}
