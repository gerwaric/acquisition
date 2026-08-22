//! Send-lifetime admission gate.
//!
//! N3 deliberately keeps this primitive independent of HTTP. A caller owns a
//! [`SendPermit`] from immediately before dispatch until the response body is
//! complete; dropping it is the only completion signal the gate needs.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use tokio::sync::oneshot;

/// Cloudflare-facing cap over live daemon-owned sends (ground truth P-B).
pub(crate) const MAX_ACTUAL_SENDS: usize = 2;

#[derive(Clone, Debug)]
pub(crate) struct SendGate {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    state: Mutex<State>,
}

#[derive(Debug, Default)]
struct State {
    next_id: u64,
    waiters: VecDeque<Waiter>,
    active: HashMap<u64, Active>,
    active_policies: HashSet<String>,
}

#[derive(Debug)]
struct Waiter {
    id: u64,
    kind: Kind,
    ready: oneshot::Sender<()>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Kind {
    Ordinary { policy: String },
    Head,
}

#[derive(Debug)]
struct Active {
    kind: Kind,
}

impl SendGate {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                state: Mutex::new(State::default()),
            }),
        }
    }

    /// Acquire one ordinary actual-send slot under `policy`.
    ///
    /// At most one permit for a policy can be live. Waiters are considered in
    /// arrival order, while a waiter blocked by its already-live policy does
    /// not idle a free global slot that an independent policy can use.
    pub(crate) async fn acquire(&self, policy: impl Into<String>) -> SendPermit {
        self.acquire_kind(Kind::Ordinary {
            policy: policy.into(),
        })
        .await
    }

    /// Acquire the exclusive HEAD writer permit.
    ///
    /// Once queued, a HEAD blocks all later ordinary grants until every live
    /// ordinary permit drains and the HEAD permit itself is released.
    pub(crate) async fn acquire_head(&self) -> SendPermit {
        self.acquire_kind(Kind::Head).await
    }

    async fn acquire_kind(&self, kind: Kind) -> SendPermit {
        let (id, ready) = self.inner.enqueue(kind);
        let mut reservation = Reservation {
            inner: Arc::clone(&self.inner),
            id: Some(id),
        };

        ready
            .await
            .expect("the gate owns every grant sender until it resolves");
        reservation.id = None;

        SendPermit {
            reservation: Reservation {
                inner: Arc::clone(&self.inner),
                id: Some(id),
            },
        }
    }
}

impl Default for SendGate {
    fn default() -> Self {
        Self::new()
    }
}

/// A live reservation for one actual send.
///
/// This value is intentionally neither `Clone` nor detachable. It releases
/// its global/policy occupancy on drop, including when its owning task is
/// canceled or unwinds.
#[derive(Debug)]
#[must_use = "dropping the permit releases the actual-send reservation"]
pub(crate) struct SendPermit {
    reservation: Reservation,
}

#[derive(Debug)]
struct Reservation {
    inner: Arc<Inner>,
    id: Option<u64>,
}

impl Drop for Reservation {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            self.inner.cancel_or_release(id);
        }
    }
}

impl Inner {
    fn enqueue(&self, kind: Kind) -> (u64, oneshot::Receiver<()>) {
        let (ready, receiver) = oneshot::channel();
        let mut state = self.state.lock().unwrap();
        let id = state.next_id;
        state.next_id = state
            .next_id
            .checked_add(1)
            .expect("send-gate waiter IDs exhausted");
        state.waiters.push_back(Waiter { id, kind, ready });
        state.grant_waiters();
        (id, receiver)
    }

    fn cancel_or_release(&self, id: u64) {
        let mut state = self.state.lock().unwrap();
        if let Some(position) = state.waiters.iter().position(|waiter| waiter.id == id) {
            state.waiters.remove(position);
        } else if let Some(active) = state.active.remove(&id)
            && let Kind::Ordinary { policy } = active.kind
        {
            let removed = state.active_policies.remove(&policy);
            debug_assert!(removed, "active policy must accompany its permit");
        }
        state.grant_waiters();
    }
}

impl State {
    fn grant_waiters(&mut self) {
        loop {
            if self.active.values().any(|active| active.kind == Kind::Head) {
                return;
            }

            if let Some(writer) = self
                .waiters
                .iter()
                .position(|waiter| waiter.kind == Kind::Head)
            {
                if self.active.is_empty() {
                    self.grant(writer);
                }
                return;
            }

            if self.active.len() >= MAX_ACTUAL_SENDS {
                return;
            }

            let Some(reader) = self.waiters.iter().position(|waiter| match &waiter.kind {
                Kind::Ordinary { policy } => !self.active_policies.contains(policy),
                Kind::Head => unreachable!("writers were handled above"),
            }) else {
                return;
            };
            self.grant(reader);
        }
    }

    fn grant(&mut self, position: usize) {
        let waiter = self
            .waiters
            .remove(position)
            .expect("waiter position exists");
        let kind = waiter.kind;
        if waiter.ready.send(()).is_err() {
            return;
        }

        if let Kind::Ordinary { policy } = &kind {
            let inserted = self.active_policies.insert(policy.clone());
            debug_assert!(inserted, "a policy cannot receive two live permits");
        }
        let replaced = self.active.insert(waiter.id, Active { kind });
        debug_assert!(replaced.is_none(), "waiter IDs are unique");
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll, Waker};

    use super::*;

    fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
        let mut context = Context::from_waker(Waker::noop());
        future.poll(&mut context)
    }

    fn ready<F: Future>(future: Pin<&mut F>) -> F::Output {
        match poll_once(future) {
            Poll::Ready(output) => output,
            Poll::Pending => panic!("future should be ready"),
        }
    }

    fn pending<F: Future>(future: Pin<&mut F>) {
        assert!(poll_once(future).is_pending(), "future should be pending");
    }

    #[test]
    fn permit_lifetime_enforces_global_cap_and_per_policy_serialization() {
        let gate = SendGate::new();
        let mut first_a = Box::pin(gate.acquire("policy-a"));
        let first_a = ready(first_a.as_mut());
        let mut second_a = Box::pin(gate.acquire("policy-a"));
        pending(second_a.as_mut());

        let mut first_b = Box::pin(gate.acquire("policy-b"));
        let first_b = ready(first_b.as_mut());
        let mut first_c = Box::pin(gate.acquire("policy-c"));
        pending(first_c.as_mut());

        drop(first_a);
        let second_a = ready(second_a.as_mut());
        pending(first_c.as_mut());

        drop(first_b);
        let first_c = ready(first_c.as_mut());

        drop((second_a, first_c));
    }

    #[test]
    fn cancellation_removes_waiters_and_releases_unclaimed_grants() {
        let gate = SendGate::new();
        let mut live_a = Box::pin(gate.acquire("policy-a"));
        let live_a = ready(live_a.as_mut());
        let mut live_b = Box::pin(gate.acquire("policy-b"));
        let live_b = ready(live_b.as_mut());

        let mut canceled = Box::pin(gate.acquire("policy-c"));
        pending(canceled.as_mut());
        let mut next = Box::pin(gate.acquire("policy-d"));
        pending(next.as_mut());
        drop(canceled);

        // Releasing one slot grants `next` before its future is polled. Dropping
        // that now-granted future must return the live reservation as well.
        drop(live_a);
        drop(next);

        let mut replacement = Box::pin(gate.acquire("policy-e"));
        let replacement = ready(replacement.as_mut());
        drop((live_b, replacement));
    }

    #[test]
    fn waiting_head_has_writer_preference_and_cancellation_unblocks_readers() {
        let gate = SendGate::new();
        let mut live = Box::pin(gate.acquire("policy-a"));
        let live = ready(live.as_mut());
        let mut head = Box::pin(gate.acquire_head());
        pending(head.as_mut());

        let mut reader = Box::pin(gate.acquire("policy-b"));
        pending(reader.as_mut());
        drop(head);
        let reader = ready(reader.as_mut());

        let mut preferred_head = Box::pin(gate.acquire_head());
        pending(preferred_head.as_mut());
        let mut later_reader = Box::pin(gate.acquire("policy-c"));
        pending(later_reader.as_mut());
        drop((live, reader));

        let preferred_head = ready(preferred_head.as_mut());
        pending(later_reader.as_mut());
        drop(preferred_head);
        let later_reader = ready(later_reader.as_mut());
        drop(later_reader);
    }

    #[test]
    fn head_is_globally_exclusive() {
        let gate = SendGate::new();
        let mut first = Box::pin(gate.acquire("policy-a"));
        let first = ready(first.as_mut());
        let mut second = Box::pin(gate.acquire("policy-b"));
        let second = ready(second.as_mut());
        let mut head = Box::pin(gate.acquire_head());
        pending(head.as_mut());

        drop(first);
        pending(head.as_mut());
        drop(second);
        let head = ready(head.as_mut());

        let mut reader = Box::pin(gate.acquire("policy-c"));
        pending(reader.as_mut());
        let mut another_head = Box::pin(gate.acquire_head());
        pending(another_head.as_mut());
        drop(head);

        let another_head = ready(another_head.as_mut());
        pending(reader.as_mut());
        drop(another_head);
        let reader = ready(reader.as_mut());
        drop(reader);
    }

    #[test]
    fn ordinary_waiters_are_fifo_when_their_policies_are_eligible() {
        let gate = SendGate::new();
        let mut blocker_a = Box::pin(gate.acquire("blocker-a"));
        let blocker_a = ready(blocker_a.as_mut());
        let mut blocker_b = Box::pin(gate.acquire("blocker-b"));
        let blocker_b = ready(blocker_b.as_mut());

        let mut first = Box::pin(gate.acquire("policy-a"));
        pending(first.as_mut());
        let mut second = Box::pin(gate.acquire("policy-b"));
        pending(second.as_mut());
        let mut third = Box::pin(gate.acquire("policy-c"));
        pending(third.as_mut());

        drop(blocker_a);
        let first = ready(first.as_mut());
        pending(second.as_mut());
        pending(third.as_mut());

        drop(blocker_b);
        let second = ready(second.as_mut());
        pending(third.as_mut());

        drop(first);
        let third = ready(third.as_mut());
        drop((second, third));
    }

    #[test]
    fn blocked_policy_does_not_stop_an_independent_policy() {
        let gate = SendGate::new();
        let mut live_a = Box::pin(gate.acquire("policy-a"));
        let live_a = ready(live_a.as_mut());

        let mut next_a = Box::pin(gate.acquire("policy-a"));
        pending(next_a.as_mut());
        let mut independent = Box::pin(gate.acquire("policy-b"));
        let independent = ready(independent.as_mut());

        drop(live_a);
        let next_a = ready(next_a.as_mut());
        drop((independent, next_a));
    }
}
