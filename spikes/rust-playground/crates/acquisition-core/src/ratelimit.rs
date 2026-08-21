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
//! Not yet built (each a separate baby step): HEAD-at-boot discovery of
//! server-side residue (N16, N24); 429 recovery beyond "don't send again
//! before Retry-After" (P-A); an explicit cross-policy burst bound (P-B) —
//! today the single worker keeps at most one API request in flight.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Server-side timing bucket for a rule's first (initial) window (N12).
pub const INITIAL_BUCKET: Duration = Duration::from_secs(5);
/// Server-side timing bucket for a rule's later (sustained) windows (N12).
pub const SUSTAINED_BUCKET: Duration = Duration::from_secs(60);
/// Extra margin on top of the bucket (N13 says the full bucket is the safe
/// margin; the shipped client adds one more second and has been clean).
pub const BUFFER: Duration = Duration::from_secs(1);

/// Which timing bucket applies to the `index`-th window of a rule.
/// Positional classification (Q4, Tom's hypothesis): the first window is
/// the initial limit, every later one is sustained. A single-window rule is
/// treated as initial. Conservative on every observed policy shape (N23).
pub fn bucket_for(index: usize) -> Duration {
    if index == 0 { INITIAL_BUCKET } else { SUSTAINED_BUCKET }
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
    /// Present on 429 responses.
    pub retry_after_secs: Option<u64>,
}

impl Policy {
    /// Parse from any header lookup (case-insensitive names expected from
    /// the caller). `None` when the response carries no policy at all.
    /// Never panics on partial header sets (N20's lesson): a missing or
    /// malformed rule header yields an empty rule, which imposes no wait.
    pub fn parse(get: impl Fn(&str) -> Option<String>) -> Option<Policy> {
        let name = get("x-rate-limit-policy")?;
        let rules = get("x-rate-limit-rules")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .map(|rule| {
                let key = rule.to_ascii_lowercase();
                let limits = get(&format!("x-rate-limit-{key}"))
                    .map(|v| triplets(&v))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(a, b, c)| Window { max_hits: a as u32, period_secs: b, restriction_secs: c })
                    .collect();
                let state = get(&format!("x-rate-limit-{key}-state"))
                    .map(|v| triplets(&v))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(a, b, c)| WindowState { hits: a as u32, period_secs: b, restricted_secs: c })
                    .collect();
                Rule { name: rule.to_string(), limits, state }
            })
            .collect();
        let retry_after_secs = get("retry-after").and_then(|v| v.trim().parse().ok());
        Some(Policy { name, rules, retry_after_secs })
    }

    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Option<Policy> {
        Policy::parse(|name| headers.get(name).and_then(|v| v.to_str().ok()).map(str::to_string))
    }
}

/// "a:b:c,d:e:f" → [(a,b,c),(d,e,f)]; malformed triplets are skipped.
fn triplets(s: &str) -> Vec<(u64, u64, u64)> {
    s.split(',')
        .filter_map(|t| {
            let mut it = t.trim().split(':').map(|n| n.trim().parse::<u64>().ok());
            Some((it.next()??, it.next()??, it.next()??))
        })
        .collect()
}

// ---- the limiter ----------------------------------------------------------

const HISTORY_CAP: usize = 256;

/// What the limiter remembers about one named policy.
struct PolicyState {
    policy: Policy,
    /// Arrival times of counted responses under this policy, oldest first.
    /// Shared by every endpoint that reports the same policy name (N6).
    history: VecDeque<Instant>,
    /// When `policy` was observed; the base for restriction/Retry-After.
    last_response: Instant,
    /// The raw headers, for the dashboard.
    raw: serde_json::Value,
}

#[derive(Default)]
pub struct Limiter {
    policies: HashMap<String, PolicyState>,
    /// Endpoint key (URL path) → policy name, learned from the first
    /// response. `None` once we've seen the endpoint answer without any
    /// policy header (e.g. the OAuth token endpoint).
    endpoints: HashMap<String, Option<String>>,
}

impl Limiter {
    pub fn new() -> Self {
        Limiter::default()
    }

    /// How long to wait before sending to `endpoint`. Zero for endpoints
    /// whose policy is still unknown — the first response teaches us.
    pub fn wait_for(&self, endpoint: &str, now: Instant) -> Duration {
        self.policy_for(endpoint)
            .and_then(next_safe_send)
            .map(|t| t.saturating_duration_since(now))
            .unwrap_or(Duration::ZERO)
    }

    /// Predicted wait for a request with `ahead` same-policy requests queued
    /// before it. Simulates the pacing rule forward: each simulated send is
    /// appended to a copy of the history, and window hit counts are taken
    /// from that history (a prediction of what the server will report —
    /// headers remain the truth for real sends). An estimate, not a promise.
    pub fn eta_for(&self, endpoint: &str, ahead: u32, now: Instant) -> Duration {
        let Some(state) = self.policy_for(endpoint) else { return Duration::ZERO };
        let mut history = state.history.clone();
        let mut t = now + self.wait_for(endpoint, now);
        for _ in 0..ahead {
            history.push_back(t);
            let mut next = t;
            for rule in &state.policy.rules {
                for (i, w) in rule.limits.iter().enumerate() {
                    let period = Duration::from_secs(w.period_secs);
                    let in_window = history.iter().filter(|&&h| t.duration_since(h) < period).count();
                    if in_window >= w.max_hits as usize
                        && let Some(&oldest) = history.get(history.len() - w.max_hits as usize)
                    {
                        next = next.max(oldest + period + bucket_for(i) + BUFFER);
                    }
                }
            }
            t = next;
        }
        t.saturating_duration_since(now)
    }

    /// Record a response. `counted` is false for requests the server does
    /// not count against the policy (HEAD probes, N24). A 429 is recorded
    /// as counted — over-estimating the wait is the safe direction.
    pub fn observe(
        &mut self,
        endpoint: &str,
        policy: Option<Policy>,
        raw: serde_json::Value,
        counted: bool,
        now: Instant,
    ) {
        let Some(policy) = policy else {
            self.endpoints.entry(endpoint.to_string()).or_insert(None);
            return;
        };
        self.endpoints.insert(endpoint.to_string(), Some(policy.name.clone()));
        let state = self.policies.entry(policy.name.clone()).or_insert_with(|| PolicyState {
            policy: policy.clone(),
            history: VecDeque::new(),
            last_response: now,
            raw: serde_json::Value::Null,
        });
        // Definitions are dynamic (N9): the latest response wins outright.
        state.policy = policy;
        state.last_response = now;
        state.raw = raw;
        if counted {
            if state.history.len() >= HISTORY_CAP {
                state.history.pop_front();
            }
            state.history.push_back(now);
        }
    }

    fn policy_for(&self, endpoint: &str) -> Option<&PolicyState> {
        self.endpoints.get(endpoint)?.as_ref().and_then(|name| self.policies.get(name))
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
                        .filter(|(_, p)| p.as_deref() == Some(name))
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
                                    bucket_secs: bucket_for(i).as_secs(),
                                }
                            })
                            .collect(),
                    })
                    .collect(),
                next_safe_in_seconds: next_safe_send(s)
                    .map(|t| t.saturating_duration_since(now).as_secs_f64())
                    .unwrap_or(0.0),
                last_observed_seconds_ago: now.saturating_duration_since(s.last_response).as_secs_f64(),
                history_len: s.history.len(),
                retry_after_secs: s.policy.retry_after_secs,
                headers: s.raw.clone(),
            })
            .collect();
        out.sort_by(|a, b| a.policy.cmp(&b.policy));
        out
    }

    /// Endpoints that have answered without any policy header.
    pub fn policyless_endpoints(&self) -> Vec<String> {
        let mut v: Vec<String> = self
            .endpoints
            .iter()
            .filter(|(_, p)| p.is_none())
            .map(|(k, _)| k.clone())
            .collect();
        v.sort();
        v
    }
}

/// The earliest instant the next request under this policy may be sent,
/// or `None` if it may go now. The pacing rule, per window of each rule:
///
/// - restriction active (`restricted-for > 0`): last response + restricted
///   + bucket + buffer;
/// - window saturated (`hits >= max`): the response that consumed the
///   oldest still-counted hit (`history[len - max]`) + period + bucket +
///   buffer (N25: post-increment, 1:1; N13: full bucket on top). If history
///   is shorter than `max` — hits carried over from before this daemon
///   started (N24: counters are server-side and persist) — assume they all
///   just happened: last response + period + bucket + buffer;
/// - 429 with Retry-After (N19): last response + Retry-After + the bucket
///   of the saturated window (the larger one if none is identifiable) +
///   buffer.
///
/// The result is the max over everything that applies.
fn next_safe_send(s: &PolicyState) -> Option<Instant> {
    let mut next: Option<Instant> = None;
    let mut bump = |t: Instant| next = Some(next.map_or(t, |n| n.max(t)));
    let mut saturated_bucket: Option<Duration> = None;

    for rule in &s.policy.rules {
        for (i, limit) in rule.limits.iter().enumerate() {
            let bucket = bucket_for(i);
            let Some(st) = rule.state.get(i) else { continue };
            if st.restricted_secs > 0 {
                bump(s.last_response + Duration::from_secs(st.restricted_secs) + bucket + BUFFER);
                saturated_bucket = Some(saturated_bucket.map_or(bucket, |b| b.max(bucket)));
            } else if st.hits >= limit.max_hits {
                let oldest = s
                    .history
                    .len()
                    .checked_sub(limit.max_hits as usize)
                    .and_then(|idx| s.history.get(idx))
                    .copied()
                    .unwrap_or(s.last_response);
                bump(oldest + Duration::from_secs(limit.period_secs) + bucket + BUFFER);
                saturated_bucket = Some(saturated_bucket.map_or(bucket, |b| b.max(bucket)));
            }
        }
    }
    if let Some(ra) = s.policy.retry_after_secs {
        let bucket = saturated_bucket.unwrap_or(SUSTAINED_BUCKET);
        bump(s.last_response + Duration::from_secs(ra) + bucket + BUFFER);
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

/// Receipt proving the limiter was consulted for this endpoint and said go.
/// Send methods that don't wait internally require one, and only `try_take`
/// mints them, so asking the limiter stays structural.
pub struct Paid {
    endpoint: String,
}

/// The limiter keys endpoints by URL path, so mock and real GGG share keys
/// (`/character` is `/character` on both hosts) and policy names learned on
/// one are meaningful on the other.
pub fn endpoint_key(url: &str) -> String {
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

pub struct ChokePoint {
    // Private on purpose: this is the only reqwest client in the workspace,
    // so every HTTP request must come through a method that consults the
    // limiter and reports the response back to it.
    http: reqwest::Client,
    limiter: Mutex<Limiter>,
    sends: Mutex<VecDeque<SentAt>>,
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
            sends: Mutex::new(VecDeque::new()),
        }
    }

    /// Ask to send to `url` now, or learn how long to wait. Callers that
    /// need cancellation-aware waiting (the job worker) loop on this; plain
    /// requests use `post_form`, which waits internally.
    pub fn try_take(&self, url: &str) -> Result<Paid, Duration> {
        let endpoint = endpoint_key(url);
        let wait = self.limiter.lock().unwrap().wait_for(&endpoint, Instant::now());
        if wait.is_zero() { Ok(Paid { endpoint }) } else { Err(wait) }
    }

    pub fn eta_for(&self, url: &str, ahead: u32) -> Duration {
        self.limiter.lock().unwrap().eta_for(&endpoint_key(url), ahead, Instant::now())
    }

    pub fn policy_statuses(&self) -> Vec<PolicyStatus> {
        self.limiter.lock().unwrap().statuses(Instant::now())
    }

    pub fn policyless_endpoints(&self) -> Vec<String> {
        self.limiter.lock().unwrap().policyless_endpoints()
    }

    /// Recent sends, newest first.
    pub fn recent_sends(&self) -> Vec<SendRecord> {
        let sends = self.sends.lock().unwrap();
        sends
            .iter()
            .rev()
            .map(|s| SendRecord {
                seconds_ago: s.at.elapsed().as_secs_f64(),
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
        method: &'static str,
        url: &str,
        result: &Result<reqwest::Response, String>,
        counted: bool,
    ) {
        let (outcome, ok) = match result {
            Ok(r) => (r.status().to_string(), r.status().is_success()),
            Err(e) => (format!("error: {e}"), false),
        };
        if let Ok(r) = result {
            let policy = Policy::from_headers(r.headers());
            let raw = rate_limit_snapshot(r.headers());
            self.limiter
                .lock()
                .unwrap()
                .observe(endpoint, policy, raw, counted, Instant::now());
        }
        let mut sends = self.sends.lock().unwrap();
        if sends.len() >= SEND_HISTORY {
            sends.pop_front();
        }
        sends.push_back(SentAt {
            at: Instant::now(),
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
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, String> {
        let paid = loop {
            match self.try_take(url) {
                Ok(paid) => break paid,
                Err(wait) => tokio::time::sleep(wait.max(Duration::from_millis(50))).await,
            }
        };
        let result = self
            .http
            .post(url)
            .form(params)
            .send()
            .await
            .map_err(|e| e.to_string());
        self.observe(&paid.endpoint, "POST", url, &result, true);
        result
    }

    /// Bearer-authenticated GET for callers that already consulted the
    /// limiter (the job worker waits while the job is still cancellable,
    /// then hands the receipt in here).
    pub async fn get_bearer(
        &self,
        paid: Paid,
        url: &str,
        bearer: &str,
    ) -> Result<reqwest::Response, String> {
        let result = self
            .http
            .get(url)
            .bearer_auth(bearer)
            .send()
            .await
            .map_err(|e| e.to_string());
        self.observe(&paid.endpoint, "GET", url, &result, true);
        result
    }

    /// Unauthenticated GET (mock-only fake data endpoints).
    pub async fn get(&self, paid: Paid, url: &str) -> Result<reqwest::Response, String> {
        let result = self.http.get(url).send().await.map_err(|e| e.to_string());
        self.observe(&paid.endpoint, "GET", url, &result, true);
        result
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

    fn parse(headers: &[(&str, &str)]) -> Option<Policy> {
        Policy::parse(|k| headers.iter().find(|(h, _)| *h == k).map(|(_, v)| v.to_string()))
    }

    /// Replay a row: feed every history point as a counted response under
    /// the row's policy, then ask for the wait at "now".
    fn run(limiter: &mut Limiter, endpoint: &str, headers: &[(&str, &str)], history: &[f64], now: Instant) {
        let policy = parse(headers);
        for &ago in history {
            let at = now - Duration::from_secs_f64(ago);
            limiter.observe(endpoint, policy.clone(), serde_json::Value::Null, true, at);
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
                Window { max_hits: 2, period_secs: 10, restriction_secs: 60 },
                Window { max_hits: 5, period_secs: 300, restriction_secs: 300 },
            ]
        );
        assert_eq!(
            r.state,
            vec![
                WindowState { hits: 1, period_secs: 10, restricted_secs: 0 },
                WindowState { hits: 1, period_secs: 300, restricted_secs: 0 },
            ]
        );
        assert_eq!(p.retry_after_secs, None);
    }

    #[test]
    fn n20_partial_headers_never_panic() {
        // Dec-2023-shaped reply: policy name only (N16/N20).
        let p = parse(&[("x-rate-limit-policy", "character-request-limit")]).expect("policy");
        assert!(p.rules.is_empty());
        // Rules named but definitions missing/malformed.
        let p = parse(&[
            ("x-rate-limit-policy", "p"),
            ("x-rate-limit-rules", "Account,Ip"),
            ("x-rate-limit-account", "garbage,1:2"),
        ])
        .expect("policy");
        assert_eq!(p.rules.len(), 2);
        assert!(p.rules[0].limits.is_empty());
        assert!(p.rules[1].limits.is_empty());
        let mut l = Limiter::new();
        let now = far_future();
        l.observe("/x", Some(p), serde_json::Value::Null, true, now);
        assert_eq!(l.wait_for("/x", now), Duration::ZERO);
        // No policy header at all → endpoint is known policyless, no wait.
        l.observe("/token", None, serde_json::Value::Null, true, now);
        assert_eq!(l.wait_for("/token", now), Duration::ZERO);
        assert_eq!(l.policyless_endpoints(), vec!["/token".to_string()]);
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
                name: "restriction active",
                claims: "restricted-for 30s from the last response + initial bucket + 1s",
                headers: with_state(CHAR_LIST, "2:10:30,2:300:0").leak(),
                history: &[5.0, 0.0],
                expect_wait: 30.0 + 5.0 + 1.0,
            },
            Row {
                name: "429 with Retry-After",
                claims: "N19: Retry-After is not enough — add the saturated window's bucket + 1s",
                headers: &[
                    ("x-rate-limit-policy", "character-list-request-limit"),
                    ("x-rate-limit-rules", "Account"),
                    ("x-rate-limit-account", "2:10:60,5:300:300"),
                    ("x-rate-limit-account-state", "2:10:60,2:300:0"),
                    ("retry-after", "60"),
                ],
                history: &[8.0, 0.0],
                expect_wait: 60.0 + 5.0 + 1.0,
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
            assert_wait(&format!("{} [{}]", row.name, row.claims), got, row.expect_wait);
        }
    }

    #[test]
    fn n6_same_policy_name_shares_counters_across_endpoints() {
        let now = far_future();
        let mut l = Limiter::new();
        let p = parse(&with_state(CHAR_LIST, "2:10:0,2:300:0"));
        l.observe("/character", p.clone(), serde_json::Value::Null, true, now - Duration::from_secs(4));
        l.observe("/character/pc", p, serde_json::Value::Null, true, now);
        // Both endpoints see the same two-event history: oldest 4s ago.
        assert_wait("shared", l.wait_for("/character", now), 10.0 + 5.0 + 1.0 - 4.0);
        assert_wait("shared", l.wait_for("/character/pc", now), 10.0 + 5.0 + 1.0 - 4.0);
    }

    #[test]
    fn n6_n7_different_policies_are_independent() {
        let now = far_future();
        let mut l = Limiter::new();
        l.observe("/character", parse(&with_state(CHAR_LIST, "2:10:0,2:300:0")), serde_json::Value::Null, true, now);
        let stash = parse(&[
            ("x-rate-limit-policy", "stash-list-request-limit"),
            ("x-rate-limit-rules", "Account"),
            ("x-rate-limit-account", "10:15:60,30:60:300"),
            ("x-rate-limit-account-state", "1:15:0,1:60:0"),
        ]);
        l.observe("/stash/Standard", stash, serde_json::Value::Null, true, now);
        assert!(l.wait_for("/character", now) > Duration::ZERO);
        assert_eq!(l.wait_for("/stash/Standard", now), Duration::ZERO);
    }

    #[test]
    fn n24_uncounted_responses_do_not_enter_history() {
        let now = far_future();
        let mut l = Limiter::new();
        let p = parse(&with_state(CHAR_LIST, "0:10:0,0:300:0"));
        l.observe("/character", p, serde_json::Value::Null, false, now);
        assert_eq!(l.statuses(now)[0].history_len, 0);
        assert_eq!(l.wait_for("/character", now), Duration::ZERO);
    }

    #[test]
    fn n9_new_definition_replaces_old() {
        let now = far_future();
        let mut l = Limiter::new();
        l.observe("/character", parse(&with_state(CHAR_LIST, "2:10:0,2:300:0")), serde_json::Value::Null, true, now - Duration::from_secs(1));
        assert!(l.wait_for("/character", now) > Duration::ZERO);
        // GGG loosens the policy: 4 per 10s now, and we've used 2.
        let loosened = parse(&[
            ("x-rate-limit-policy", "character-list-request-limit"),
            ("x-rate-limit-rules", "Account"),
            ("x-rate-limit-account", "4:10:60,5:300:300"),
            ("x-rate-limit-account-state", "2:10:0,2:300:0"),
        ]);
        l.observe("/character", loosened, serde_json::Value::Null, true, now);
        assert_eq!(l.wait_for("/character", now), Duration::ZERO);
    }

    #[test]
    fn eta_simulates_the_pacing_rule_forward() {
        let now = far_future();
        let mut l = Limiter::new();
        // 2 per 10s, both used: this instant and 4s ago.
        run(&mut l, "/character", &with_state(CHAR_LIST, "2:10:0,2:300:0"), &[4.0, 0.0], now);
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
    fn endpoint_keys_are_paths() {
        assert_eq!(endpoint_key("https://api.pathofexile.com/character"), "/character");
        assert_eq!(endpoint_key("http://127.0.0.1:5555/character"), "/character");
    }
}
