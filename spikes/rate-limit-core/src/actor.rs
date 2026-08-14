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

/// Starts the actor. `engine` and `transport` are moved in, so no other task
/// can call the scheduling/response entry points or reach the wire handle.
///
/// The concrete owner is intentionally private:
///
/// ```compile_fail,E0603
/// use rate_limit_core::actor::Actor;
/// ```
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
            // Commands that have already reached ingress take effect before a
            // permit decision. Without this drain, a ready pacing timer could
            // grant a reader between two queued commands and miss a writer
            // that was already waiting in the mpsc mailbox.
            while let Ok(command) = self.commands.try_recv() {
                self.handle_command(Some(command));
            }
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
        // Writer preference is structural: a queued unknown endpoint is a
        // pending HEAD writer even when an established GET appears earlier in
        // FIFO order. No new ordinary permit can pass it; when the current
        // readers drain, the earliest queued writer gets exclusive occupancy.
        if let Some(endpoint) = self.pending_probe() {
            if !self.active.is_empty() || self.probe.is_some() || !self.spacing_open() {
                return;
            }
            self.start_probe(endpoint);
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
        let Some(request) = self
            .queue
            .iter()
            .find(|queued| queued.endpoint == endpoint)
            .map(|queued| queued.request.clone())
        else {
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
        self.start_transport(probe, move |outcome| Completed::Probe { endpoint, outcome });
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
        self.start_transport(request, move |outcome| Completed::Ordinary { id, outcome });
        self.publish();
    }

    /// The sole hand-off to `Transport::send` (X2). Probe and ordinary
    /// callers supply only the typed completion wrapper; timeout and error
    /// classification cannot drift between two wire paths.
    fn start_transport(
        &mut self,
        request: WireRequest,
        complete: impl FnOnce(Result<WireResponse, GateError>) -> Completed + Send + 'static,
    ) {
        let transport = self.transport.clone();
        self.jobs.spawn(async move {
            let outcome = match timeout(TRANSPORT_TIMEOUT, transport.send(request)).await {
                Ok(Ok(response)) => Ok(response),
                Ok(Err(error)) => Err(GateError::Transport(error)),
                Err(_) => Err(GateError::TimedOut),
            };
            complete(outcome)
        });
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
        for notification in &transition.notifications {
            if let Notification::Remapped { from, to } = notification {
                for state in self.endpoints.values_mut() {
                    if matches!(state, EndpointState::Established(policy) if policy == from) {
                        *state = EndpointState::Established(to.clone());
                    }
                }
            }
        }
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

    fn pending_probe(&self) -> Option<EndpointLabel> {
        self.queue.iter().find_map(|queued| {
            self.endpoints
                .get(&queued.endpoint)
                .is_some_and(|state| matches!(state, EndpointState::Unknown))
                .then(|| queued.endpoint.clone())
        })
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
    // WireResponse is constructed only through the transport boundary's
    // bounded constructor, so cloning its header view and scanning its body
    // have fixed maximum work.
    let headers = response.headers().clone();
    let is_cloudflare_candidate = response.status() == StatusCode::FORBIDDEN
        && headers
            .get("cf-mitigated")
            .is_some_and(|value| value.as_bytes().eq_ignore_ascii_case(b"challenge"))
        && headers
            .get("server")
            .is_some_and(|value| value.as_bytes().eq_ignore_ascii_case(b"cloudflare"));
    let classification = if is_cloudflare_candidate {
        if response
            .body()
            .windows(4)
            .any(|chunk| chunk.eq_ignore_ascii_case(b"html"))
        {
            ReplyClassification::CloudflareShaped
        } else {
            ReplyClassification::Normal
        }
    } else {
        ReplyClassification::Normal
    };
    ObservedResponse::new(response.status(), headers, classification)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{BucketModel, Resolution};
    use crate::header::MAX_HEADER_VALUE_BYTES;
    use crate::transport::{
        MAX_RESPONSE_BODY_BYTES, MAX_RESPONSE_HEADER_NAME_BYTES, MAX_RESPONSE_HEADERS,
        ResponseBoundsError,
    };
    use http::{HeaderMap, HeaderName, HeaderValue, Response};
    use proptest::prelude::*;

    struct FaultTransport;

    impl Transport for FaultTransport {
        async fn send(&self, _request: WireRequest) -> Result<WireResponse, TransportError> {
            Err(TransportError::MockHarness("fault injection".to_owned()))
        }
    }

    fn actor_at_transport_boundary() -> Actor<FaultTransport> {
        let (_, commands) = mpsc::channel(1);
        let (status, _) = watch::channel(GateStatus::default());
        Actor::new(
            PolicyEngine::new(BucketModel::new(
                Resolution::Assumed(Duration::from_secs(60)),
                Resolution::Assumed(Duration::from_secs(60)),
            )),
            Arc::new(FaultTransport),
            commands,
            status,
        )
    }

    fn queued_request(
        id: u64,
        endpoint: &EndpointLabel,
    ) -> (
        QueuedRequest,
        oneshot::Receiver<Result<WireResponse, GateError>>,
    ) {
        let (reply, response) = oneshot::channel();
        (
            QueuedRequest {
                id: RequestId(id),
                endpoint: endpoint.clone(),
                request: http::Request::new(Vec::new()),
                reply: Some(reply),
            },
            response,
        )
    }

    fn policy_wire_response(status: StatusCode) -> WireResponse {
        let response = Response::builder()
            .status(status)
            .header("x-rate-limit-policy", "safety-policy")
            .header("x-rate-limit-rules", "Account")
            .header("x-rate-limit-account", "10:10:60, 30:60:300")
            .header("x-rate-limit-account-state", "0:10:0, 0:60:0")
            .header("retry-after", "0")
            .body(Vec::new())
            .unwrap();
        WireResponse::try_new(response).unwrap()
    }

    fn seed_safety_policy(
        actor: &mut Actor<FaultTransport>,
        endpoint: &EndpointLabel,
    ) -> PolicyName {
        let response = policy_wire_response(StatusCode::NO_CONTENT);
        let transition =
            actor
                .engine
                .on_probe_response(endpoint, actor.now(), &observed_response(&response));
        match transition.disposition {
            Disposition::ProbeReady { policy } => policy,
            other => panic!("valid policy seed returned {other:?}"),
        }
    }

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

    // X1: this drives the actor's actual `start_dispatch` hook, which is the
    // last common point before either HEAD or GET reaches `Transport::send`.
    // Counter contents are fault-injected so D5's 250 ms floor need not be
    // disabled just to exercise the fuse's otherwise unreachable trip path.
    #[test]
    fn x1_fault_injection_trips_at_the_actor_transport_boundary() {
        let mut burst = actor_at_transport_boundary();
        for _ in 0..FUSE_BURST_LIMIT {
            burst.start_dispatch();
            assert!(!burst.halted);
        }
        burst.start_dispatch();
        assert!(burst.halted);

        let mut sustained = actor_at_transport_boundary();
        let now = Instant::now();
        sustained.safety.dispatches = (0..FUSE_SUSTAINED_LIMIT - 1)
            .map(|index| {
                now - Duration::from_millis(59_800) + Duration::from_millis(index as u64 * 120)
            })
            .collect();
        sustained.start_dispatch();
        assert!(sustained.halted);
    }

    #[test]
    fn c3_x1_trip_is_latched_and_drains_and_publishes() {
        let mut actor = actor_at_transport_boundary();
        let mut status = actor.status.subscribe();
        let endpoint = EndpointLabel::from("stash-list");
        let (queued, mut outcome) = queued_request(1, &endpoint);
        actor.queue.push_back(queued);
        let now = Instant::now();
        actor.safety.dispatches = VecDeque::from([now; FUSE_BURST_LIMIT]);

        actor.start_dispatch();

        assert!(actor.halted, "the 11th dispatch attempt trips X1");
        assert!(actor.queue.is_empty(), "trip drains the pending deque");
        assert!(status.borrow_and_update().halted, "Halted is published");
        assert!(matches!(outcome.try_recv(), Ok(Err(GateError::Halted))));

        // Advance beyond both rolling windows, erase the historical shape,
        // and re-ask. The terminal latch, not recomputation, keeps the actor
        // halted (C3).
        actor.safety.dispatches.clear();
        actor.start_dispatch();
        assert!(actor.halted);
        assert!(actor.safety.halted);
    }

    #[test]
    fn c4_probe_feed_trips_shared_halt_and_drains_and_publishes() {
        let mut actor = actor_at_transport_boundary();
        let mut status = actor.status.subscribe();
        let endpoint = EndpointLabel::from("stash-list");
        let (queued, mut outcome) = queued_request(1, &endpoint);
        actor.queue.push_back(queued);
        actor
            .endpoints
            .insert(endpoint.clone(), EndpointState::Probing);
        actor.probe = Some(endpoint.clone());
        actor.safety.four_xx = VecDeque::from([Instant::now(); FUSE_BURST_LIMIT]);

        actor.finish_probe(
            endpoint,
            Ok(policy_wire_response(StatusCode::TOO_MANY_REQUESTS)),
        );

        assert_eq!(actor.safety.four_xx.len(), FUSE_BURST_LIMIT + 1);
        assert!(actor.halted, "the probe 429 feed shares terminal halt");
        assert!(actor.queue.is_empty());
        assert!(status.borrow_and_update().halted);
        assert!(matches!(outcome.try_recv(), Ok(Err(GateError::Halted))));
    }

    #[test]
    fn c4_ordinary_feed_trips_shared_halt_and_drains_and_publishes() {
        let mut actor = actor_at_transport_boundary();
        let mut status = actor.status.subscribe();
        let endpoint = EndpointLabel::from("stash-list");
        let policy = seed_safety_policy(&mut actor, &endpoint);
        actor
            .endpoints
            .insert(endpoint.clone(), EndpointState::Established(policy.clone()));
        let token = match actor.engine.try_reserve(&policy, actor.now()) {
            ReserveOutcome::Reserved(token) => token,
            other => panic!("seeded policy did not reserve: {other:?}"),
        };
        let (active, mut active_outcome) = queued_request(1, &endpoint);
        actor.active.insert(
            RequestId(1),
            ActiveRequest {
                queued: active,
                token,
            },
        );
        let (queued, mut queued_outcome) = queued_request(2, &endpoint);
        actor.queue.push_back(queued);
        actor.safety.four_xx = VecDeque::from([Instant::now(); FUSE_BURST_LIMIT]);

        actor.finish_ordinary(
            RequestId(1),
            Ok(policy_wire_response(StatusCode::UNAUTHORIZED)),
        );

        assert_eq!(actor.safety.four_xx.len(), FUSE_BURST_LIMIT + 1);
        assert!(actor.halted, "the ordinary 4xx feed shares terminal halt");
        assert!(actor.queue.is_empty());
        assert!(status.borrow_and_update().halted);
        assert!(matches!(
            active_outcome.try_recv(),
            Ok(Err(GateError::Halted))
        ));
        assert!(matches!(
            queued_outcome.try_recv(),
            Ok(Err(GateError::Halted))
        ));
    }

    #[test]
    fn x2_transport_owner_is_private_and_has_one_send_call_site() {
        let source = include_str!("actor.rs");
        assert!(
            source
                .lines()
                .any(|line| line.trim() == concat!("struct Actor", "<T: Transport> {"))
        );
        assert!(!source.lines().any(|line| {
            line.trim_start()
                .starts_with(concat!("pub struct Actor", "<T: Transport>"))
        }));
        // concat! keeps every needle out of this test's own string literals;
        // `source` is this very file, so an unsplit needle matches itself and
        // the assertion becomes vacuous (round-five SD-R5-F5 caught exactly
        // that on the field check below).
        assert!(source.contains(concat!("transport: ", "Arc<T>")));
        // Count transport sends by receiver, not by argument name: the
        // pre-refactor two-path shape sent from start_probe with an argument
        // named `probe`, which a needle ending in `(request)` cannot see.
        assert_eq!(
            source.matches(concat!("transport", ".send(")).count(),
            1,
            "probe and ordinary traffic must share one transport hand-off"
        );
        assert_eq!(
            source.matches(concat!("fn start_", "transport(")).count(),
            1,
            "the single send path must remain a unique actor method"
        );
    }

    #[test]
    fn response_header_boundary_is_checked_before_clone() {
        let mut headers = HeaderMap::new();
        for index in 0..MAX_RESPONSE_HEADERS {
            let name = HeaderName::from_bytes(format!("x-bound-{index}").as_bytes()).unwrap();
            headers.insert(name, HeaderValue::from_static("v"));
        }
        let mut exact = Response::new(Vec::new());
        *exact.headers_mut() = headers.clone();
        assert!(WireResponse::try_new(exact).is_ok());

        headers.insert("x-bound-over", HeaderValue::from_static("v"));
        assert_eq!(
            WireResponse::try_new({
                let mut response = Response::new(Vec::new());
                *response.headers_mut() = headers;
                response
            })
            .unwrap_err(),
            ResponseBoundsError::TooManyHeaders {
                limit: MAX_RESPONSE_HEADERS,
            }
        );

        let mut name_boundary = HeaderMap::new();
        let exact_name = format!("x-{}", "n".repeat(MAX_RESPONSE_HEADER_NAME_BYTES - 2));
        name_boundary.insert(
            HeaderName::from_bytes(exact_name.as_bytes()).unwrap(),
            HeaderValue::from_static("v"),
        );
        let mut exact = Response::new(Vec::new());
        *exact.headers_mut() = name_boundary;
        assert!(WireResponse::try_new(exact).is_ok());

        let mut over_name = HeaderMap::new();
        let overlong_name = format!("x-{}", "n".repeat(MAX_RESPONSE_HEADER_NAME_BYTES - 1));
        over_name.insert(
            HeaderName::from_bytes(overlong_name.as_bytes()).unwrap(),
            HeaderValue::from_static("v"),
        );
        assert_eq!(
            WireResponse::try_new({
                let mut response = Response::new(Vec::new());
                *response.headers_mut() = over_name;
                response
            })
            .unwrap_err(),
            ResponseBoundsError::HeaderNameTooLong {
                limit: MAX_RESPONSE_HEADER_NAME_BYTES,
            }
        );

        let mut value_boundary = HeaderMap::new();
        value_boundary.insert(
            "x-bound-value",
            "v".repeat(MAX_HEADER_VALUE_BYTES).parse().unwrap(),
        );
        let mut exact = Response::new(Vec::new());
        *exact.headers_mut() = value_boundary.clone();
        assert!(WireResponse::try_new(exact).is_ok());
        value_boundary.insert(
            "x-bound-value",
            "v".repeat(MAX_HEADER_VALUE_BYTES + 1).parse().unwrap(),
        );
        assert_eq!(
            WireResponse::try_new({
                let mut response = Response::new(Vec::new());
                *response.headers_mut() = value_boundary;
                response
            })
            .unwrap_err(),
            ResponseBoundsError::HeaderValueTooLong {
                limit: MAX_HEADER_VALUE_BYTES,
            }
        );
    }

    #[test]
    fn cloudflare_body_boundary_is_checked_before_html_scan() {
        let exact = Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("cf-mitigated", "challenge")
            .header("server", "cloudflare")
            .body(vec![b'x'; MAX_RESPONSE_BODY_BYTES])
            .unwrap();
        assert!(WireResponse::try_new(exact).is_ok());

        let over = Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("cf-mitigated", "challenge")
            .header("server", "cloudflare")
            .body(vec![b'x'; MAX_RESPONSE_BODY_BYTES + 1])
            .unwrap();
        assert_eq!(
            WireResponse::try_new(over).unwrap_err(),
            ResponseBoundsError::BodyTooLong {
                limit: MAX_RESPONSE_BODY_BYTES,
            }
        );
    }

    #[test]
    fn c3_and_x1_fault_injection_pin_burst_and_sustained_boundaries() {
        let now = Instant::now();
        let mut burst = SafetyCounters {
            dispatches: VecDeque::new(),
            four_xx: VecDeque::new(),
            halted: false,
        };
        for _ in 0..FUSE_BURST_LIMIT {
            assert!(!burst.dispatch(now));
        }
        assert!(burst.dispatch(now), "X1 burst trips on the 11th hand-off");

        let mut below_sustained = SafetyCounters {
            dispatches: VecDeque::from([now - Duration::from_secs(2); FUSE_SUSTAINED_LIMIT - 2]),
            four_xx: VecDeque::new(),
            halted: false,
        };
        assert!(!below_sustained.dispatch(now), "C3: 499/min is safe");

        let mut sustained = SafetyCounters {
            dispatches: VecDeque::from([now - Duration::from_secs(2); FUSE_SUSTAINED_LIMIT - 1]),
            four_xx: VecDeque::new(),
            halted: false,
        };
        assert!(sustained.dispatch(now), "X1 sustained trips at 500/min");
    }

    #[test]
    fn c4_pins_burst_sustained_and_exact_window_edges() {
        let now = Instant::now();
        let mut burst = SafetyCounters {
            dispatches: VecDeque::new(),
            four_xx: VecDeque::new(),
            halted: false,
        };
        for _ in 0..FUSE_BURST_LIMIT {
            assert!(!burst.four_xx(now));
        }
        assert!(burst.four_xx(now), "C4 trips on its 11th 4xx");

        let mut edge = SafetyCounters {
            dispatches: VecDeque::new(),
            four_xx: VecDeque::from([now - FUSE_BURST_WINDOW; FUSE_BURST_LIMIT]),
            halted: false,
        };
        assert!(!edge.four_xx(now), "the 1 s boundary is half-open");

        let mut below_sustained = SafetyCounters {
            dispatches: VecDeque::new(),
            four_xx: VecDeque::from([now - Duration::from_secs(2); FUSE_SUSTAINED_LIMIT - 2]),
            halted: false,
        };
        assert!(!below_sustained.four_xx(now), "C4: 499/min is safe");

        let mut sustained = SafetyCounters {
            dispatches: VecDeque::new(),
            four_xx: VecDeque::from([now - Duration::from_secs(2); FUSE_SUSTAINED_LIMIT - 1]),
            halted: false,
        };
        assert!(sustained.four_xx(now), "C4 trips at 500/min");
    }

    /// C3's spacing-implied legitimate maxima, derived from D5's floor rather
    /// than copied from the clause being tested: a half-open trailing second
    /// admits four 250 ms-spaced instants, a half-open trailing minute 240.
    const FLOOR_BURST_MAXIMUM: usize = 4;
    const FLOOR_SUSTAINED_MAXIMUM: usize = 240;

    /// Long enough that the sustained clause prunes across several trailing
    /// minutes instead of filling one.  At the floor these are 6.25 and 3.1
    /// simulated minutes respectively.
    const UNIFORM_FLOOR_TRACE_LEN: usize = 1_500;
    const IRREGULAR_FLOOR_TRACE_LEN: usize = 750;

    /// Independent trailing-window counts over a generated trace, using C3's
    /// half-open convention — an entry exactly one window old has left it.
    ///
    /// Never consults the counter under test.  The trace is ascending, so the
    /// backward scan stops at the first entry outside the sustained window and
    /// costs the window's occupancy rather than the whole trace.
    fn trailing_counts(trace: &[Instant], now: Instant) -> (usize, usize) {
        let (mut burst, mut sustained) = (0_usize, 0_usize);
        for &at in trace.iter().rev() {
            if at <= now - FUSE_SUSTAINED_WINDOW {
                break;
            }
            sustained += 1;
            if at > now - FUSE_BURST_WINDOW {
                burst += 1;
            }
        }
        (burst, sustained)
    }

    /// The worst floor-compliant trace — a uniform cadence exactly at the
    /// floor — run past several trailing minutes, so the sustained clause is
    /// exercised at maximum density rather than assumed.
    ///
    /// The two equalities are this pin's reachability guard: they fail if the
    /// trace never reaches steady state, which is how it could go quietly
    /// vacuous.  C3's headroom claim is exactly this: the legitimate maxima
    /// sit at 4 of 10 and 240 of 500.
    #[test]
    fn c3_floor_compliant_cadence_holds_the_steady_state_maximum() {
        let start = Instant::now();
        let mut counters = SafetyCounters {
            dispatches: VecDeque::new(),
            four_xx: VecDeque::new(),
            halted: false,
        };
        let mut trace = Vec::new();
        let (mut peak_burst, mut peak_sustained) = (0_usize, 0_usize);
        for index in 0..UNIFORM_FLOOR_TRACE_LEN {
            let now =
                start + MIN_SEND_SPACING * u32::try_from(index).expect("trace length fits u32");
            trace.push(now);
            let (burst, sustained) = trailing_counts(&trace, now);
            peak_burst = peak_burst.max(burst);
            peak_sustained = peak_sustained.max(sustained);
            assert!(
                !counters.dispatch(now),
                "a floor-compliant cadence must never trip either clause"
            );
        }
        assert_eq!(peak_burst, FLOOR_BURST_MAXIMUM);
        assert_eq!(peak_sustained, FLOOR_SUSTAINED_MAXIMUM);
    }

    /// Mostly at the floor, with occasional pauses and idle stretches, so the
    /// generated traces are irregular without drifting so sparse that they
    /// stop pressing on the clauses.
    fn floor_compliant_extra_gap_ms() -> impl Strategy<Value = u64> {
        prop_oneof![
            8 => Just(0_u64),
            1 => 1_u64..1_000,
            1 => 1_000_u64..20_000,
        ]
    }

    proptest! {
        /// C3's stated property: *no* floor-compliant trace trips either
        /// clause.  The gaps are generated rather than uniform, so the claim
        /// covers irregular callers — bursty-then-idle included — and not one
        /// cadence; `c3_floor_compliant_cadence_holds_the_steady_state_maximum`
        /// pins the worst case exactly.  Every step of every case asserts, so
        /// the property cannot pass vacuously.
        #[test]
        fn c3_floor_compliant_traces_never_trip(
            extra_gaps_ms in prop::collection::vec(
                floor_compliant_extra_gap_ms(),
                IRREGULAR_FLOOR_TRACE_LEN,
            ),
        ) {
            let start = Instant::now();
            let mut counters = SafetyCounters {
                dispatches: VecDeque::new(),
                four_xx: VecDeque::new(),
                halted: false,
            };
            let mut trace = Vec::new();
            let mut offset = Duration::ZERO;
            for extra_ms in extra_gaps_ms {
                let now = start + offset;
                trace.push(now);
                let (burst, sustained) = trailing_counts(&trace, now);
                // Independent arithmetic over the generated trace, not the
                // counter under test.
                prop_assert!(burst <= FLOOR_BURST_MAXIMUM);
                prop_assert!(sustained <= FLOOR_SUSTAINED_MAXIMUM);
                prop_assert!(!counters.dispatch(now));
                offset += MIN_SEND_SPACING + Duration::from_millis(extra_ms);
            }
            // Guards the trace length constant, not the generator: shortening
            // it below three trailing minutes would silently stop exercising
            // the pruning this property exists to reach.
            let span = trace.last().expect("the trace is non-empty").duration_since(start);
            prop_assert!(span >= FUSE_SUSTAINED_WINDOW * 3, "trace spanned only {span:?}");
        }
    }
}
