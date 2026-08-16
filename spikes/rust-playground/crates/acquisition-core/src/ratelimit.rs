//! Fake token bucket standing in for the real GGG-header-driven limiter.
//!
//! The real thing will be a policy layer parsing `X-Rate-Limit-*` headers with
//! an enforcement mechanism underneath (see CONTEXT.md). This exists so the
//! daemon has something to wait on and predict ETAs from. Deliberately tight
//! (small burst, slow refill) so queueing is visible within seconds of play.

use std::time::{Duration, Instant};

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
    fn eta_scales_with_queue_depth() {
        let mut b = TokenBucket::new(1, Duration::from_secs(10));
        assert!(b.try_take().is_ok());
        let one_ahead = b.eta_for(0);
        let two_ahead = b.eta_for(1);
        assert!(two_ahead > one_ahead);
        assert!(two_ahead > Duration::from_secs(15));
    }
}
