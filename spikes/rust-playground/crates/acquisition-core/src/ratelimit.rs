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

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Which endpoint policy a request falls under. Every outbound request names
/// one; there is no "unlimited" variant on purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endpoint {
    Api,
    OauthToken,
}

pub struct TokenBucket {
    capacity: u32,
    refill_every: Duration,
    tokens: u32,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(capacity: u32, refill_every: Duration) -> Self {
        TokenBucket {
            capacity,
            refill_every,
            tokens: capacity,
            last_refill: Instant::now(),
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

pub struct ChokePoint {
    // Private on purpose: this is the only reqwest client in the workspace,
    // so every HTTP request must come through a method that takes a token.
    http: reqwest::Client,
    buckets: Mutex<Buckets>,
}

impl ChokePoint {
    /// Playground policies: API keeps the classic burst-of-5 bucket; the
    /// token endpoint gets its own, tight enough that spamming `auth check`
    /// makes the wait visible.
    pub fn playground_default() -> Self {
        ChokePoint {
            http: reqwest::Client::new(),
            buckets: Mutex::new(Buckets {
                api: TokenBucket::playground_default(),
                oauth_token: TokenBucket::new(2, Duration::from_secs(5)),
            }),
        }
    }

    /// Take a token now, or report how long until the next one exists.
    /// Callers that need cancellation-aware waiting (the job worker) loop on
    /// this; plain requests use `post_form`, which waits internally.
    pub fn try_take(&self, endpoint: Endpoint) -> Result<(), Duration> {
        self.buckets.lock().unwrap().get(endpoint).try_take()
    }

    pub fn eta_for(&self, endpoint: Endpoint, n: u32) -> Duration {
        self.buckets.lock().unwrap().get(endpoint).eta_for(n)
    }

    pub fn available(&self, endpoint: Endpoint) -> u32 {
        self.buckets.lock().unwrap().get(endpoint).available()
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
                Ok(()) => break,
                Err(wait) => tokio::time::sleep(wait.max(Duration::from_millis(50))).await,
            }
        }
        self.http
            .post(url)
            .form(params)
            .send()
            .await
            .map_err(|e| e.to_string())
    }
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
