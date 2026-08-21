//! The rate-limit choke point and the fake token buckets behind it.
//!
//! `ChokePoint` is CONTEXT invariant 1 made structural: it privately owns the
//! workspace's only `reqwest::Client`, so nothing can send an HTTP request
//! without acquiring from the right endpoint's limiter first. Endpoints have
//! separate policies (the OAuth token endpoint has its own: ground-truth N33).
//!
//! The buckets stand in for the real GGG-header-driven limiter, which will be
//! a policy layer parsing `X-Rate-Limit-*` headers with an enforcement
//! mechanism underneath (see CONTEXT.md). Deliberately tight (small burst,
//! slow refill) so queueing is visible within seconds of play.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Which endpoint policy a request falls under. Every outbound request names
/// one; there is no "unlimited" variant on purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endpoint {
    Api,
    OauthToken,
}

impl Endpoint {
    pub fn name(self) -> &'static str {
        match self {
            Endpoint::Api => "api",
            Endpoint::OauthToken => "oauth_token",
        }
    }
}

/// One endpoint's limiter state at a point in time, for the dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketStatus {
    pub endpoint: String,
    pub capacity: u32,
    pub available: u32,
    pub refill_every_seconds: f64,
    /// Seconds until the next token exists; 0 while tokens are available.
    pub next_token_seconds: f64,
    /// Seconds until the bucket is back at capacity; 0 when already full.
    pub full_in_seconds: f64,
    /// Tokens granted since the daemon started.
    pub spent_total: u64,
    /// Tokens granted in the trailing 60 seconds.
    pub spent_last_60s: u32,
    pub last_grant_seconds_ago: Option<f64>,
    /// The most recent `X-Rate-Limit-*` headers seen on a response from this
    /// endpoint — the provider's own statement of the policy (invariant 2's
    /// ground truth, recorded but not yet obeyed).
    pub observed: Option<ObservedHeaders>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedHeaders {
    pub seconds_ago: f64,
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

/// Receipt proving a limiter token was spent. Send methods that don't wait
/// internally require one, and only `try_take` mints them, so paying the
/// limiter stays structural even for callers that wait on their own schedule.
pub struct Paid {
    endpoint: Endpoint,
}

const GRANT_HISTORY: usize = 200;

pub struct TokenBucket {
    capacity: u32,
    refill_every: Duration,
    tokens: u32,
    last_refill: Instant,
    spent_total: u64,
    /// When recent grants happened, oldest first (bounded ring).
    grants: VecDeque<Instant>,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_every: Duration) -> Self {
        TokenBucket {
            capacity,
            refill_every,
            tokens: capacity,
            last_refill: Instant::now(),
            spent_total: 0,
            grants: VecDeque::new(),
        }
    }

    /// Playground default: burst of 5, one token back every 3 seconds.
    pub fn playground_default() -> Self {
        TokenBucket::new(5, Duration::from_secs(3))
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        let earned = (elapsed.as_secs_f64() / self.refill_every.as_secs_f64()) as u32;
        if earned > 0 {
            self.tokens = (self.tokens + earned).min(self.capacity);
            if self.tokens == self.capacity {
                self.last_refill = Instant::now();
            } else {
                self.last_refill += self.refill_every * earned;
            }
        }
    }

    /// Take a token now, or report how long until the next one exists.
    pub fn try_take(&mut self) -> Result<(), Duration> {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            self.spent_total += 1;
            if self.grants.len() >= GRANT_HISTORY {
                self.grants.pop_front();
            }
            self.grants.push_back(Instant::now());
            Ok(())
        } else {
            Err(self
                .refill_every
                .saturating_sub(self.last_refill.elapsed()))
        }
    }

    /// Predicted wait until the (n+1)-th token from now could be taken, i.e.
    /// the ETA for a job with `n` token-consuming jobs ahead of it.
    pub fn eta_for(&mut self, n: u32) -> Duration {
        self.refill();
        if self.tokens > n {
            Duration::ZERO
        } else {
            let needed = n + 1 - self.tokens;
            (self.refill_every * needed)
                .saturating_sub(self.last_refill.elapsed().min(self.refill_every))
        }
    }

    pub fn available(&mut self) -> u32 {
        self.refill();
        self.tokens
    }

    fn status(&mut self, endpoint: Endpoint) -> BucketStatus {
        self.refill();
        let next_token = if self.tokens > 0 {
            Duration::ZERO
        } else {
            self.refill_every.saturating_sub(self.last_refill.elapsed())
        };
        let missing = self.capacity - self.tokens;
        let full_in = if missing == 0 {
            Duration::ZERO
        } else {
            (self.refill_every * missing)
                .saturating_sub(self.last_refill.elapsed().min(self.refill_every))
        };
        BucketStatus {
            endpoint: endpoint.name().to_string(),
            capacity: self.capacity,
            available: self.tokens,
            refill_every_seconds: self.refill_every.as_secs_f64(),
            next_token_seconds: next_token.as_secs_f64(),
            full_in_seconds: full_in.as_secs_f64(),
            spent_total: self.spent_total,
            spent_last_60s: self
                .grants
                .iter()
                .filter(|g| g.elapsed() <= Duration::from_secs(60))
                .count() as u32,
            last_grant_seconds_ago: self.grants.back().map(|g| g.elapsed().as_secs_f64()),
            observed: None, // filled in by the choke point
        }
    }
}

struct Buckets {
    api: TokenBucket,
    oauth_token: TokenBucket,
}

impl Buckets {
    fn get(&mut self, endpoint: Endpoint) -> &mut TokenBucket {
        match endpoint {
            Endpoint::Api => &mut self.api,
            Endpoint::OauthToken => &mut self.oauth_token,
        }
    }
}

/// A send the choke point performed, kept as an Instant internally and
/// converted to `seconds_ago` when snapshotted.
struct SentAt {
    at: Instant,
    endpoint: Endpoint,
    method: &'static str,
    url: String,
    outcome: String,
    ok: bool,
}

const SEND_HISTORY: usize = 100;

/// Last `X-Rate-Limit-*` headers seen per endpoint, with when.
#[derive(Default)]
struct ObservedMap {
    api: Option<(Instant, serde_json::Value)>,
    oauth_token: Option<(Instant, serde_json::Value)>,
}

impl ObservedMap {
    fn get_mut(&mut self, endpoint: Endpoint) -> &mut Option<(Instant, serde_json::Value)> {
        match endpoint {
            Endpoint::Api => &mut self.api,
            Endpoint::OauthToken => &mut self.oauth_token,
        }
    }
}

pub struct ChokePoint {
    // Private on purpose: this is the only reqwest client in the workspace,
    // so every HTTP request must come through a method that takes a token.
    http: reqwest::Client,
    buckets: Mutex<Buckets>,
    /// Recent sends, newest last. Every request goes through here, so this
    /// is a complete record of outbound traffic (bounded to SEND_HISTORY).
    sends: Mutex<VecDeque<SentAt>>,
    observed: Mutex<ObservedMap>,
}

impl ChokePoint {
    fn with_buckets(buckets: Buckets) -> Self {
        ChokePoint {
            // The user-agent goes on the client itself so no request — token
            // exchange included — can be sent without it (CONTEXT invariant 4).
            http: reqwest::Client::builder()
                .user_agent(crate::provider::USER_AGENT)
                .build()
                .expect("reqwest client builds"),
            buckets: Mutex::new(buckets),
            sends: Mutex::new(VecDeque::new()),
            observed: Mutex::new(ObservedMap::default()),
        }
    }

    /// Playground policies: API keeps the classic burst-of-5 bucket; the
    /// token endpoint gets its own, tight enough that spamming `auth check`
    /// makes the wait visible.
    pub fn playground_default() -> Self {
        ChokePoint::with_buckets(Buckets {
            api: TokenBucket::playground_default(),
            oauth_token: TokenBucket::new(2, Duration::from_secs(5)),
        })
    }

    /// Real-GGG policies, hard-coded far under the observed limits (the
    /// character-list policy is 2:10:60 + 5:300:300, so 1-per-60s satisfies
    /// both windows). Header-driven correction (invariant 2) is not built
    /// yet; until it is, the buckets stay this conservative.
    pub fn ggg_conservative() -> Self {
        ChokePoint::with_buckets(Buckets {
            api: TokenBucket::new(1, Duration::from_secs(60)),
            oauth_token: TokenBucket::new(2, Duration::from_secs(30)),
        })
    }

    /// Take a token now, or report how long until the next one exists.
    /// Callers that need cancellation-aware waiting (the job worker) loop on
    /// this; plain requests use `post_form`, which waits internally.
    pub fn try_take(&self, endpoint: Endpoint) -> Result<Paid, Duration> {
        self.buckets
            .lock()
            .unwrap()
            .get(endpoint)
            .try_take()
            .map(|()| Paid { endpoint })
    }

    pub fn eta_for(&self, endpoint: Endpoint, n: u32) -> Duration {
        self.buckets.lock().unwrap().get(endpoint).eta_for(n)
    }

    pub fn available(&self, endpoint: Endpoint) -> u32 {
        self.buckets.lock().unwrap().get(endpoint).available()
    }

    pub fn bucket_statuses(&self) -> Vec<BucketStatus> {
        let mut statuses: Vec<BucketStatus> = {
            let mut buckets = self.buckets.lock().unwrap();
            [Endpoint::Api, Endpoint::OauthToken]
                .into_iter()
                .map(|e| buckets.get(e).status(e))
                .collect()
        };
        let mut observed = self.observed.lock().unwrap();
        for (status, endpoint) in statuses.iter_mut().zip([Endpoint::Api, Endpoint::OauthToken]) {
            status.observed = observed.get_mut(endpoint).as_ref().map(|(at, headers)| {
                ObservedHeaders {
                    seconds_ago: at.elapsed().as_secs_f64(),
                    headers: headers.clone(),
                }
            });
        }
        statuses
    }

    /// Recent sends, newest first.
    pub fn recent_sends(&self) -> Vec<SendRecord> {
        let sends = self.sends.lock().unwrap();
        sends
            .iter()
            .rev()
            .map(|s| SendRecord {
                seconds_ago: s.at.elapsed().as_secs_f64(),
                endpoint: s.endpoint.name().to_string(),
                method: s.method.to_string(),
                url: s.url.clone(),
                outcome: s.outcome.clone(),
                ok: s.ok,
            })
            .collect()
    }

    fn record_send(
        &self,
        endpoint: Endpoint,
        method: &'static str,
        url: &str,
        result: &Result<reqwest::Response, String>,
    ) {
        let (outcome, ok) = match result {
            Ok(r) => (r.status().to_string(), r.status().is_success()),
            Err(e) => (format!("error: {e}"), false),
        };
        if let Ok(r) = result {
            let rate = rate_limit_snapshot(r.headers());
            if rate.as_object().is_some_and(|m| !m.is_empty()) {
                *self.observed.lock().unwrap().get_mut(endpoint) = Some((Instant::now(), rate));
            }
        }
        let mut sends = self.sends.lock().unwrap();
        if sends.len() >= SEND_HISTORY {
            sends.pop_front();
        }
        sends.push_back(SentAt {
            at: Instant::now(),
            endpoint,
            method,
            url: url.to_string(),
            outcome,
            ok,
        });
    }

    /// The one way to send a form POST: waits for the endpoint's limiter,
    /// then sends. The bucket lock is never held across an await.
    pub async fn post_form(
        &self,
        endpoint: Endpoint,
        url: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, String> {
        loop {
            match self.try_take(endpoint) {
                Ok(_paid) => break,
                Err(wait) => tokio::time::sleep(wait.max(Duration::from_millis(50))).await,
            }
        }
        let result = self
            .http
            .post(url)
            .form(params)
            .send()
            .await
            .map_err(|e| e.to_string());
        self.record_send(endpoint, "POST", url, &result);
        result
    }

    /// Bearer-authenticated GET for callers that already paid the limiter
    /// (the job worker waits while the job is still cancellable, then hands
    /// the receipt in here).
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
        self.record_send(paid.endpoint, "GET", url, &result);
        result
    }
}

/// The `X-Rate-Limit-*` (+ `Retry-After`) headers as a JSON object, for
/// logging and job payloads. Headers are the source of truth (invariant 2);
/// the spike records them but does not yet feed them back into the buckets.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn burst_then_wait() {
        let mut b = TokenBucket::new(2, Duration::from_secs(10));
        assert!(b.try_take().is_ok());
        assert!(b.try_take().is_ok());
        let wait = b.try_take().unwrap_err();
        assert!(wait > Duration::from_secs(9));
    }

    #[test]
    fn endpoints_have_independent_buckets() {
        let choke = ChokePoint::playground_default();
        while choke.try_take(Endpoint::OauthToken).is_ok() {}
        assert!(choke.try_take(Endpoint::Api).is_ok());
    }

    #[test]
    fn eta_scales_with_queue_depth() {
        let mut b = TokenBucket::new(1, Duration::from_secs(10));
        assert!(b.try_take().is_ok());
        let one_ahead = b.eta_for(0);
        let two_ahead = b.eta_for(1);
        assert!(two_ahead > one_ahead);
        assert!(two_ahead > Duration::from_secs(15));
    }
}
