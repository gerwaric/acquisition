//! Tokio owner for the single serialized scheduling gate.
//!
//! The actor owns the queue, endpoint knowledge, core, and every reservation
//! token.  Transport tasks only wait for wire outcomes; they never make a
//! scheduling decision.

use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::task::{Context, Poll};
use std::time::Duration;

use http::{HeaderName, HeaderValue, Method, StatusCode};
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until, timeout};

use crate::core::{
    Disposition, EndpointLabel, Notification, ObservedResponse, PolicyEngine, PolicyName,
    RefusalCause, RefusalReason, RefusalTarget, ReplyClassification, ReservationToken,
    ReserveOutcome, SimInstant,
};
use crate::transport::{Transport, TransportError, WireRequest, WireResponse};

pub const MIN_SEND_SPACING: Duration = Duration::from_millis(250);
pub const D5_IN_FLIGHT_CAP: usize = 2;
pub const SETUP_RETRY_COOLDOWN: Duration = Duration::from_secs(60);
/// This must stay below the smallest padded policy horizon used by the spike.
pub const TRANSPORT_TIMEOUT: Duration = Duration::from_secs(30);
pub const MAX_PENDING_REQUESTS: usize = 10_000;
pub const MAX_ENDPOINTS: usize = 5;
pub const COMMAND_CAPACITY: usize = 256;

const FUSE_BURST_LIMIT: usize = 10;
const FUSE_SUSTAINED_LIMIT: usize = 500;
const FUSE_BURST_WINDOW: Duration = Duration::from_secs(1);
const FUSE_SUSTAINED_WINDOW: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

impl RequestId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    CommandClosed,
    RequestIdExhausted,
    QueueFull {
        limit: usize,
    },
    EndpointLimitReached {
        limit: usize,
    },
    Cancelled,
    Halted,
    Blocked,
    Refused(RefusalReason),
    SetupFailed {
        endpoint: EndpointLabel,
        cause: RefusalCause,
    },
    Transport(TransportError),
    TimedOut,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GateStatus {
    pub queued: usize,
    pub ordinary_in_flight: usize,
    pub probing: Option<EndpointLabel>,
    pub halted: bool,
}

#[derive(Debug, Clone)]
pub struct GateHandle {
    commands: mpsc::Sender<Command>,
    next_id: Arc<AtomicU64>,
    status: watch::Receiver<GateStatus>,
}

impl GateHandle {
    pub fn subscribe_status(&self) -> watch::Receiver<GateStatus> {
        self.status.clone()
    }

    pub async fn submit(
        &self,
        endpoint: EndpointLabel,
        request: WireRequest,
    ) -> Result<RequestTicket, GateError> {
        let id = self.allocate_id()?;
        let (reply, response) = oneshot::channel();
        self.commands
            .send(Command::Enqueue(Box::new(QueuedRequest {
                id,
                endpoint,
                request,
                reply: Some(reply),
            })))
            .await
            .map_err(|_| GateError::CommandClosed)?;
        Ok(RequestTicket {
            id,
            commands: self.commands.clone(),
            response,
        })
    }

    pub async fn enqueue(
        &self,
        endpoint: EndpointLabel,
        request: WireRequest,
    ) -> Result<WireResponse, GateError> {
        self.submit(endpoint, request).await?.await
    }

    fn allocate_id(&self) -> Result<RequestId, GateError> {
        self.next_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                (id != u64::MAX).then_some(id + 1)
            })
            .map(RequestId)
            .map_err(|_| GateError::RequestIdExhausted)
    }
}

#[must_use = "dropping a ticket cancels only its caller outcome; dispatched work still lands"]
pub struct RequestTicket {
    id: RequestId,
    commands: mpsc::Sender<Command>,
    response: oneshot::Receiver<Result<WireResponse, GateError>>,
}

impl RequestTicket {
    pub const fn id(&self) -> RequestId {
        self.id
    }

    pub async fn cancel(&self) -> Result<(), GateError> {
        self.commands
            .send(Command::Cancel(self.id))
            .await
            .map_err(|_| GateError::CommandClosed)
    }
}

impl Future for RequestTicket {
    type Output = Result<WireResponse, GateError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match Pin::new(&mut self.response).poll(cx) {
            Poll::Ready(Ok(outcome)) => Poll::Ready(outcome),
            Poll::Ready(Err(_)) => Poll::Ready(Err(GateError::CommandClosed)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Starts the actor. `engine` is moved in, so no other task can call its
/// scheduling or response entry points.
pub fn spawn<T>(engine: PolicyEngine, transport: T) -> GateHandle
where
    T: Transport,
{
    let (commands, receiver) = mpsc::channel(COMMAND_CAPACITY);
    let (status_sender, status) = watch::channel(GateStatus::default());
    let handle = GateHandle {
        commands,
        next_id: Arc::new(AtomicU64::new(1)),
        status,
    };
    tokio::spawn(Actor::new(engine, Arc::new(transport), receiver, status_sender).run());
    handle
}

enum Command {
    Enqueue(Box<QueuedRequest>),
    Cancel(RequestId),
}

struct QueuedRequest {
    id: RequestId,
    endpoint: EndpointLabel,
    request: WireRequest,
    reply: Option<oneshot::Sender<Result<WireResponse, GateError>>>,
}

impl QueuedRequest {
    fn finish(&mut self, outcome: Result<WireResponse, GateError>) {
        if let Some(reply) = self.reply.take() {
            let _ = reply.send(outcome);
        }
    }

    fn caller_is_gone(&self) -> bool {
        self.reply.as_ref().is_none_or(oneshot::Sender::is_closed)
    }
}

struct ActiveRequest {
    queued: QueuedRequest,
    token: ReservationToken,
}

#[derive(Clone)]
enum EndpointState {
    Unknown,
    Probing,
    Established(PolicyName),
    Cooling { until: Instant, cause: RefusalCause },
}

enum Completed {
    Ordinary {
        id: RequestId,
        outcome: Result<WireResponse, GateError>,
    },
    Probe {
        endpoint: EndpointLabel,
        outcome: Result<WireResponse, GateError>,
    },
}

struct SafetyCounters {
    dispatches: VecDeque<Instant>,
    four_xx: VecDeque<Instant>,
    halted: bool,
}

impl SafetyCounters {
    fn dispatch(&mut self, now: Instant) -> bool {
        if self.halted {
            return true;
        }
        retain_recent(&mut self.dispatches, now, FUSE_SUSTAINED_WINDOW);
        self.dispatches.push_back(now);
        let burst = self
            .dispatches
            .iter()
            .filter(|&&at| at > now - FUSE_BURST_WINDOW)
            .count();
        self.halted = burst > FUSE_BURST_LIMIT || self.dispatches.len() >= FUSE_SUSTAINED_LIMIT;
        self.halted
    }

    fn four_xx(&mut self, now: Instant) -> bool {
        if self.halted {
            return true;
        }
        retain_recent(&mut self.four_xx, now, FUSE_SUSTAINED_WINDOW);
        self.four_xx.push_back(now);
        let burst = self
            .four_xx
            .iter()
            .filter(|&&at| at > now - FUSE_BURST_WINDOW)
            .count();
        // C4 says "same shape" as C3 but gives no independent threshold;
        // actor-handoff.md records this conservative, reviewable reading.
        self.halted = burst > FUSE_BURST_LIMIT || self.four_xx.len() >= FUSE_SUSTAINED_LIMIT;
        self.halted
    }
}

fn retain_recent(entries: &mut VecDeque<Instant>, now: Instant, window: Duration) {
    let edge = now - window;
    while entries.front().is_some_and(|at| *at <= edge) {
        entries.pop_front();
    }
}

struct Actor<T: Transport> {
    engine: PolicyEngine,
    transport: Arc<T>,
    commands: mpsc::Receiver<Command>,
    status: watch::Sender<GateStatus>,
    origin: Instant,
    queue: VecDeque<QueuedRequest>,
    endpoints: HashMap<EndpointLabel, EndpointState>,
    active: HashMap<RequestId, ActiveRequest>,
    probe: Option<EndpointLabel>,
    jobs: JoinSet<Completed>,
    last_dispatch: Option<Instant>,
    wake_at: Option<Instant>,
    safety: SafetyCounters,
    next_correlation: u64,
    halted: bool,
}

impl<T: Transport> Actor<T> {
    fn new(
        engine: PolicyEngine,
        transport: Arc<T>,
        commands: mpsc::Receiver<Command>,
        status: watch::Sender<GateStatus>,
    ) -> Self {
        Self {
            engine,
            transport,
            commands,
            status,
            origin: Instant::now(),
            queue: VecDeque::new(),
            endpoints: HashMap::new(),
            active: HashMap::new(),
            probe: None,
            jobs: JoinSet::new(),
            last_dispatch: None,
            wake_at: None,
            safety: SafetyCounters {
                dispatches: VecDeque::new(),
                four_xx: VecDeque::new(),
                halted: false,
            },
            next_correlation: 1,
            halted: false,
        }
    }

    async fn run(mut self) {
        let mut commands_open = true;
        loop {
            self.schedule();
            if !commands_open
                && self.queue.is_empty()
                && self.active.is_empty()
                && self.probe.is_none()
            {
                break;
            }

            if let Some(wake_at) = self.wake_at {
                tokio::select! {
                    command = self.commands.recv(), if commands_open => {
                        commands_open = self.handle_command(command);
                    }
                    completed = self.jobs.join_next(), if !self.jobs.is_empty() => self.handle_job(completed),
                    _ = sleep_until(wake_at) => self.wake_at = None,
                }
            } else {
                tokio::select! {
                    command = self.commands.recv(), if commands_open => {
                        commands_open = self.handle_command(command);
                    }
                    completed = self.jobs.join_next(), if !self.jobs.is_empty() => self.handle_job(completed),
                }
            }
        }
    }

    fn handle_command(&mut self, command: Option<Command>) -> bool {
        match command {
            Some(Command::Enqueue(request)) => {
                let mut request = *request;
                if self.halted || self.engine.is_halted() {
                    request.finish(Err(GateError::Halted));
                } else if self.queue.len() == MAX_PENDING_REQUESTS {
                    request.finish(Err(GateError::QueueFull {
                        limit: MAX_PENDING_REQUESTS,
                    }));
                } else if !self.endpoints.contains_key(&request.endpoint)
                    && self.endpoints.len() == MAX_ENDPOINTS
                {
                    request.finish(Err(GateError::EndpointLimitReached {
                        limit: MAX_ENDPOINTS,
                    }));
                } else {
                    self.endpoints
                        .entry(request.endpoint.clone())
                        .or_insert(EndpointState::Unknown);
                    self.queue.push_back(request);
                    self.publish();
                }
                true
            }
            Some(Command::Cancel(id)) => {
                if let Some(position) = self.queue.iter().position(|request| request.id == id)
                    && let Some(mut request) = self.queue.remove(position)
                {
                    request.finish(Err(GateError::Cancelled));
                    self.publish();
                } else if let Some(active) = self.active.get_mut(&id) {
                    // Caller cancellation never aborts an in-flight request:
                    // its reservation and response reconciliation still land.
                    active.queued.finish(Err(GateError::Cancelled));
                    self.publish();
                }
                true
            }
            None => false,
        }
    }

    fn schedule(&mut self) {
        self.wake_at = None;
        self.prune_cancelled();
        if self.halted || self.engine.is_halted() {
            self.halt();
            return;
        }
        loop {
            let Some(front) = self.queue.front() else {
                return;
            };
            let endpoint = front.endpoint.clone();
            let Some(state) = self.endpoints.get(&endpoint).cloned() else {
                self.halt();
                return;
            };
            match state {
                EndpointState::Cooling { until, cause } if until > Instant::now() => {
                    self.fail_endpoint(&endpoint, cause);
                    continue;
                }
                EndpointState::Cooling { .. } => {
                    self.endpoints.insert(endpoint, EndpointState::Unknown);
                    self.publish();
                    continue;
                }
                EndpointState::Unknown => {
                    // A queued probe is a writer: do not issue any new GET permit.
                    if !self.active.is_empty() || self.probe.is_some() {
                        return;
                    }
                    if !self.spacing_open() {
                        return;
                    }
                    self.start_probe(endpoint);
                    return;
                }
                EndpointState::Probing => return,
                EndpointState::Established(policy) => {
                    if self.probe.is_some() || self.active.len() == D5_IN_FLIGHT_CAP {
                        return;
                    }
                    if !self.spacing_open() {
                        return;
                    }
                    let now = self.now();
                    match self.engine.try_reserve(&policy, now) {
                        ReserveOutcome::Reserved(token) => self.start_ordinary(token),
                        ReserveOutcome::NotBefore(not_before) => {
                            self.wake_at = Some(self.at(not_before));
                            return;
                        }
                        ReserveOutcome::Blocked if self.active.is_empty() => {
                            if let Some(mut request) = self.queue.pop_front() {
                                request.finish(Err(GateError::Blocked));
                                self.publish();
                            }
                        }
                        ReserveOutcome::Blocked => return,
                        ReserveOutcome::Refused(reason) => {
                            self.fail_policy(&policy, GateError::Refused(reason));
                        }
                    }
                }
            }
        }
    }

    fn spacing_open(&mut self) -> bool {
        let Some(last) = self.last_dispatch else {
            return true;
        };
        let earliest = last + MIN_SEND_SPACING;
        if Instant::now() >= earliest {
            true
        } else {
            self.wake_at = Some(earliest);
            false
        }
    }

    fn start_probe(&mut self, endpoint: EndpointLabel) {
        let Some(request) = self.queue.front().map(|queued| queued.request.clone()) else {
            return;
        };
        let mut probe = request;
        *probe.method_mut() = Method::HEAD;
        *probe.body_mut() = Vec::new();
        let correlation = self.allocate_correlation();
        self.add_correlation(&mut probe, correlation);
        self.start_dispatch();
        if self.halted {
            return;
        }
        self.endpoints
            .insert(endpoint.clone(), EndpointState::Probing);
        self.probe = Some(endpoint.clone());
        let transport = self.transport.clone();
        self.jobs.spawn(async move {
            let outcome = match timeout(TRANSPORT_TIMEOUT, transport.send(probe)).await {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(error)) => Err(GateError::Transport(error)),
                Err(_) => Err(GateError::TimedOut),
            };
            Completed::Probe { endpoint, outcome }
        });
        self.publish();
    }

    fn start_ordinary(&mut self, token: ReservationToken) {
        let Some(mut queued) = self.queue.pop_front() else {
            self.engine.rollback(token);
            self.halt();
            return;
        };
        let correlation = self.allocate_correlation();
        self.add_correlation(&mut queued.request, correlation);
        self.start_dispatch();
        if self.halted {
            self.engine.rollback(token);
            queued.finish(Err(GateError::Halted));
            self.publish();
            return;
        }
        let id = queued.id;
        let request = queued.request.clone();
        self.active.insert(id, ActiveRequest { queued, token });
        let transport = self.transport.clone();
        self.jobs.spawn(async move {
            let outcome = match timeout(TRANSPORT_TIMEOUT, transport.send(request)).await {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(error)) => Err(GateError::Transport(error)),
                Err(_) => Err(GateError::TimedOut),
            };
            Completed::Ordinary { id, outcome }
        });
        self.publish();
    }

    fn start_dispatch(&mut self) {
        let now = Instant::now();
        if self.safety.dispatch(now) {
            self.halt();
        } else {
            self.last_dispatch = Some(now);
        }
    }

    fn handle_job(&mut self, completed: Option<Result<Completed, tokio::task::JoinError>>) {
        match completed {
            Some(Ok(Completed::Ordinary { id, outcome })) => self.finish_ordinary(id, outcome),
            Some(Ok(Completed::Probe { endpoint, outcome })) => {
                self.finish_probe(endpoint, outcome)
            }
            Some(Err(_)) => {
                // A panicking transport cannot be trusted to have delivered a reply.
                // Resolve every live token pessimistically and stop further sends.
                let now = self.now();
                for (_, mut active) in self.active.drain() {
                    let _ = self.engine.on_unknown_outcome(active.token, now);
                    active.queued.finish(Err(GateError::Halted));
                }
                self.halt();
            }
            None => {}
        }
    }

    fn finish_ordinary(&mut self, id: RequestId, outcome: Result<WireResponse, GateError>) {
        let Some(mut active) = self.active.remove(&id) else {
            return;
        };
        let now = self.now();
        match outcome {
            Ok(response) => {
                let observed = observed_response(&response);
                let transition = self.engine.on_response(active.token, now, &observed);
                if response.status().is_client_error() && self.safety.four_xx(Instant::now()) {
                    active.queued.finish(Err(GateError::Halted));
                    self.halt();
                } else {
                    self.interpret_ordinary(active.queued, response, transition);
                }
            }
            Err(error) => {
                let transition = self.engine.on_unknown_outcome(active.token, now);
                active.queued.finish(Err(error));
                self.interpret_unknown_transition(transition);
            }
        }
        self.publish();
    }

    fn finish_probe(&mut self, endpoint: EndpointLabel, outcome: Result<WireResponse, GateError>) {
        self.probe = None;
        let now = self.now();
        match outcome {
            Ok(response) => {
                let observed = observed_response(&response);
                let transition = self.engine.on_probe_response(&endpoint, now, &observed);
                if response.status().is_client_error() && self.safety.four_xx(Instant::now()) {
                    self.halt();
                } else {
                    match transition.disposition {
                        Disposition::ProbeReady { policy } => {
                            self.endpoints
                                .insert(endpoint, EndpointState::Established(policy));
                        }
                        Disposition::Refuse { cause, .. } => self.cool(endpoint, cause),
                        Disposition::Halt => self.halt(),
                        _ => self.halt(),
                    }
                }
            }
            Err(error) => {
                let transition = self.engine.on_probe_unknown_outcome(&endpoint);
                let cause = match transition.disposition {
                    Disposition::Refuse { cause, .. } => cause,
                    _ => RefusalCause::ProbeUnknownOutcome,
                };
                self.cool(endpoint, cause);
                let _ = error;
            }
        }
        self.publish();
    }

    fn interpret_ordinary(
        &mut self,
        mut queued: QueuedRequest,
        response: WireResponse,
        transition: crate::core::Transition,
    ) {
        match transition.disposition {
            Disposition::CompleteRequest => {
                queued.finish(Ok(response));
            }
            Disposition::Requeue => {
                self.queue.push_front(queued);
            }
            Disposition::Refuse { target, cause } => match target {
                RefusalTarget::Policy(policy) => {
                    queued.finish(Err(GateError::SetupFailed {
                        endpoint: queued.endpoint.clone(),
                        cause: cause.clone(),
                    }));
                    self.fail_policy(
                        &policy,
                        GateError::SetupFailed {
                            endpoint: queued.endpoint,
                            cause,
                        },
                    );
                }
                RefusalTarget::Endpoint(endpoint) => self.cool(endpoint, cause),
            },
            Disposition::Halt => {
                queued.finish(Err(GateError::Halted));
                self.halt();
            }
            Disposition::ProbeReady { .. } => self.halt(),
        }
        if transition
            .notifications
            .contains(&Notification::StateChanged)
        {
            self.publish();
        }
    }

    fn interpret_unknown_transition(&mut self, transition: crate::core::Transition) {
        if matches!(transition.disposition, Disposition::Refuse { .. }) {
            // The token was a final confirmation. Its caller already got the
            // transport error; policy suspension is represented by the core.
        }
        if transition
            .notifications
            .contains(&Notification::StateChanged)
        {
            self.publish();
        }
    }

    fn cool(&mut self, endpoint: EndpointLabel, cause: RefusalCause) {
        self.endpoints.insert(
            endpoint.clone(),
            EndpointState::Cooling {
                until: Instant::now() + SETUP_RETRY_COOLDOWN,
                cause: cause.clone(),
            },
        );
        self.fail_endpoint(&endpoint, cause);
    }

    fn fail_endpoint(&mut self, endpoint: &EndpointLabel, cause: RefusalCause) {
        self.queue = self
            .queue
            .drain(..)
            .filter_map(|mut queued| {
                if &queued.endpoint == endpoint {
                    queued.finish(Err(GateError::SetupFailed {
                        endpoint: endpoint.clone(),
                        cause: cause.clone(),
                    }));
                    None
                } else {
                    Some(queued)
                }
            })
            .collect();
        self.publish();
    }

    fn fail_policy(&mut self, policy: &PolicyName, error: GateError) {
        let endpoints: Vec<_> = self
            .endpoints
            .iter()
            .filter(|(_, state)| matches!(state, EndpointState::Established(existing) if existing == policy))
            .map(|(endpoint, _)| endpoint.clone())
            .collect();
        self.queue = self
            .queue
            .drain(..)
            .filter_map(|mut queued| {
                if endpoints
                    .iter()
                    .any(|endpoint| endpoint == &queued.endpoint)
                {
                    queued.finish(Err(error.clone()));
                    None
                } else {
                    Some(queued)
                }
            })
            .collect();
        self.publish();
    }

    fn halt(&mut self) {
        if self.halted {
            return;
        }
        self.halted = true;
        for mut queued in self.queue.drain(..) {
            queued.finish(Err(GateError::Halted));
        }
        self.publish();
    }

    fn prune_cancelled(&mut self) {
        let before = self.queue.len();
        self.queue.retain(|queued| !queued.caller_is_gone());
        if self.queue.len() != before {
            self.publish();
        }
    }

    fn allocate_correlation(&mut self) -> RequestId {
        let id = RequestId(self.next_correlation);
        self.next_correlation = self.next_correlation.saturating_add(1);
        id
    }

    fn add_correlation(&self, request: &mut WireRequest, id: RequestId) {
        // Production requests leave this absent; the in-process conformance
        // mock needs a run-wide correlation header for B13 attribution.
        let name = request
            .extensions()
            .get::<CorrelationHeader>()
            .map(|header| header.0.clone());
        if let Some(name) = name {
            request.headers_mut().insert(name, HeaderValue::from(id.0));
        }
    }

    fn now(&self) -> SimInstant {
        let millis = u64::try_from(Instant::now().duration_since(self.origin).as_millis())
            .unwrap_or(u64::MAX);
        SimInstant::from_millis(millis)
    }

    fn at(&self, instant: SimInstant) -> Instant {
        self.origin + Duration::from_millis(instant.as_millis())
    }

    fn publish(&self) {
        self.status.send_replace(GateStatus {
            queued: self.queue.len(),
            ordinary_in_flight: self.active.len(),
            probing: self.probe.clone(),
            halted: self.halted || self.engine.is_halted(),
        });
    }
}

/// Opt-in mock-only correlation injection. A request obtains this extension
/// through [`with_correlation_header`], keeping test metadata out of normal
/// traffic.
#[derive(Debug, Clone)]
pub struct CorrelationHeader(HeaderName);

pub fn with_correlation_header(mut request: WireRequest, header: HeaderName) -> WireRequest {
    request.extensions_mut().insert(CorrelationHeader(header));
    request
}

fn observed_response(response: &WireResponse) -> ObservedResponse {
    ObservedResponse::new(
        response.status(),
        response.headers().clone(),
        if response.status() == StatusCode::FORBIDDEN
            && response
                .headers()
                .get("cf-mitigated")
                .is_some_and(|value| value.as_bytes().eq_ignore_ascii_case(b"challenge"))
            && response
                .headers()
                .get("server")
                .is_some_and(|value| value.as_bytes().eq_ignore_ascii_case(b"cloudflare"))
            && response
                .body()
                .windows(4)
                .any(|chunk| chunk.eq_ignore_ascii_case(b"html"))
        {
            ReplyClassification::CloudflareShaped
        } else {
            ReplyClassification::Normal
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuse_uses_the_documented_half_open_boundaries() {
        let now = Instant::now();
        let mut counters = SafetyCounters {
            dispatches: VecDeque::new(),
            four_xx: VecDeque::new(),
            halted: false,
        };
        for _ in 0..FUSE_BURST_LIMIT {
            assert!(!counters.dispatch(now));
        }
        assert!(counters.dispatch(now));

        let mut counters = SafetyCounters {
            dispatches: VecDeque::from([now - FUSE_BURST_WINDOW; FUSE_BURST_LIMIT]),
            four_xx: VecDeque::new(),
            halted: false,
        };
        assert!(!counters.dispatch(now));
    }
}
