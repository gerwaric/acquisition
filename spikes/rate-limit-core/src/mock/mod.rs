//! In-process, paused-time mock used to judge wire behavior.

pub mod model;

use std::collections::{HashMap, HashSet};
use std::time::{Duration, UNIX_EPOCH};

use http::header::{CONTENT_TYPE, DATE, SERVER};
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode};
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::transport::{Transport, TransportError, WireRequest, WireResponse};
use model::{
    CounterModel, Layer1Judgment, MAX_REQUESTS_PER_RUN, ModelError, PolicyDefinition,
    PolicyJudgment, RuleDefinition, ServerTime, WindowDefinition,
};

pub const CORRELATION_HEADER: HeaderName = HeaderName::from_static("x-acq-test-correlation");
pub const DEFAULT_SERVICE_DELAY: Duration = Duration::from_millis(50);
pub const MAX_RAW_RESPONSE_BODY_BYTES: usize = 16 * 1024;
pub const MAX_RAW_RESPONSE_HEADERS: usize = 32;
pub const MAX_MOCK_HEADER_VALUE_BYTES: usize = 1_024;

const POLICY_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-policy");
const RULES_HEADER: HeaderName = HeaderName::from_static("x-rate-limit-rules");
const RETRY_AFTER_HEADER: HeaderName = HeaderName::from_static("retry-after");
const CF_MITIGATED_HEADER: HeaderName = HeaderName::from_static("cf-mitigated");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Endpoint {
    StashList,
    Stash,
    CharacterList,
    Character,
    LegacyStashIndex,
}

impl Endpoint {
    pub const ALL: [Self; 5] = [
        Self::StashList,
        Self::Stash,
        Self::CharacterList,
        Self::Character,
        Self::LegacyStashIndex,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::StashList => "list-stashes",
            Self::Stash => "get-stash",
            Self::CharacterList => "list-characters",
            Self::Character => "get-character",
            Self::LegacyStashIndex => "get-legacy-stash-index",
        }
    }

    pub fn uri(self) -> http::Uri {
        format!("/{}", self.label())
            .parse()
            .expect("fixed endpoint labels are valid URI paths")
    }

    fn from_uri(uri: &http::Uri) -> Option<Self> {
        let label = uri.path().strip_prefix('/')?;
        Self::ALL
            .into_iter()
            .find(|endpoint| endpoint.label() == label)
    }

    const fn head_status(self) -> StatusCode {
        match self {
            Self::LegacyStashIndex => StatusCode::OK,
            _ => StatusCode::NO_CONTENT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockConfigError {
    InvalidDispatchBudget { requested: usize, maximum: usize },
    DuplicateRoute(Endpoint),
    MissingRoutePolicy { endpoint: Endpoint, policy: String },
    InvalidPolicy(String),
    InvalidScript(String),
}

#[derive(Debug, Clone)]
pub struct MockConfig {
    pub seed: u64,
    pub phase_ms: u64,
    pub dispatch_budget: usize,
    pub policies: Vec<PolicyDefinition>,
    pub routes: Vec<(Endpoint, String)>,
}

impl MockConfig {
    pub fn n23(seed: u64, phase_ms: u64) -> Self {
        let known = (5_000, 60_000);
        let assumed = (60_000, 60_000);
        let policies = vec![
            one_rule_policy(
                "stash-list-request-limit",
                "Account",
                [(10, 15, 60), (30, 60, 300)],
                known,
            ),
            one_rule_policy(
                "stash-request-limit",
                "Account",
                [(15, 10, 60), (30, 300, 300)],
                known,
            ),
            one_rule_policy(
                "character-list-request-limit",
                "Account",
                [(2, 10, 60), (5, 300, 300)],
                known,
            ),
            one_rule_policy(
                "character-request-limit",
                "Account",
                [(5, 10, 60), (30, 300, 300)],
                known,
            ),
            PolicyDefinition::new(
                "backend-item-request-limit",
                vec![
                    rule("Account", [(30, 60, 60), (100, 1_800, 600)], assumed),
                    rule("Ip", [(45, 60, 120), (180, 1_800, 600)], assumed),
                ],
            )
            .expect("fixed N23 legacy policy is valid"),
        ];
        let routes = vec![
            (Endpoint::StashList, "stash-list-request-limit".to_owned()),
            (Endpoint::Stash, "stash-request-limit".to_owned()),
            (
                Endpoint::CharacterList,
                "character-list-request-limit".to_owned(),
            ),
            (Endpoint::Character, "character-request-limit".to_owned()),
            (
                Endpoint::LegacyStashIndex,
                "backend-item-request-limit".to_owned(),
            ),
        ];
        Self {
            seed,
            phase_ms,
            dispatch_budget: MAX_REQUESTS_PER_RUN,
            policies,
            routes,
        }
    }

    pub fn validate(&self) -> Result<(), MockConfigError> {
        if self.dispatch_budget == 0 || self.dispatch_budget > MAX_REQUESTS_PER_RUN {
            return Err(MockConfigError::InvalidDispatchBudget {
                requested: self.dispatch_budget,
                maximum: MAX_REQUESTS_PER_RUN,
            });
        }
        let mut names = HashSet::new();
        for policy in &self.policies {
            if !names.insert(policy.name()) {
                return Err(MockConfigError::InvalidPolicy(format!(
                    "duplicate policy {}",
                    policy.name()
                )));
            }
        }
        let mut endpoints = HashSet::new();
        for (endpoint, policy) in &self.routes {
            if !endpoints.insert(*endpoint) {
                return Err(MockConfigError::DuplicateRoute(*endpoint));
            }
            if !names.contains(policy.as_str()) {
                return Err(MockConfigError::MissingRoutePolicy {
                    endpoint: *endpoint,
                    policy: policy.clone(),
                });
            }
        }
        Ok(())
    }
}

fn one_rule_policy(
    name: &str,
    rule_name: &str,
    windows: [(u32, u64, u64); 2],
    buckets_ms: (u64, u64),
) -> PolicyDefinition {
    PolicyDefinition::new(name, vec![rule(rule_name, windows, buckets_ms)])
        .expect("fixed N23 policy is valid")
}

fn rule(name: &str, windows: [(u32, u64, u64); 2], buckets_ms: (u64, u64)) -> RuleDefinition {
    RuleDefinition::new(
        name,
        windows
            .into_iter()
            .zip([buckets_ms.0, buckets_ms.1])
            .map(
                |((max_hits, period_seconds, restriction_seconds), bucket_ms)| {
                    WindowDefinition::new(
                        max_hits,
                        period_seconds * 1_000,
                        restriction_seconds * 1_000,
                        bucket_ms,
                    )
                    .expect("fixed N23 window is valid")
                },
            )
            .collect(),
    )
    .expect("fixed N23 rule is valid")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseOverride {
    Full {
        status: StatusCode,
        retry_after: Option<String>,
    },
    PolicyOnly,
    Cloudflare,
    Raw {
        status: StatusCode,
        headers: HeaderMap,
        body: Vec<u8>,
    },
    TransportError(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExchangeScript {
    pub arrival_delay: Duration,
    pub response_delay: Duration,
    pub response: Option<ResponseOverride>,
}

impl Default for ExchangeScript {
    fn default() -> Self {
        Self {
            arrival_delay: Duration::ZERO,
            response_delay: DEFAULT_SERVICE_DELAY,
            response: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StimulusKind {
    Full(StatusCode),
    PolicyOnly,
    Cloudflare,
    Raw(StatusCode),
    TransportError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    pub ordinal: usize,
    pub seed: u64,
    pub phase_ms: u64,
    pub correlation_id: u64,
    pub arrival_ms: u64,
    pub completion_ms: u64,
    pub method: Method,
    pub endpoint: Endpoint,
    pub policy: String,
    pub in_flight_at_arrival: usize,
    pub head_overlap: bool,
    pub layer1: Layer1Judgment,
    pub policy_judgment: PolicyJudgment,
    pub stimulus: Option<StimulusKind>,
    pub response_status: Option<StatusCode>,
}

#[derive(Debug)]
struct MockState {
    seed: u64,
    dispatch_budget: usize,
    dispatches: usize,
    exhausted: bool,
    model: CounterModel,
    routes: HashMap<Endpoint, String>,
    scripts: HashMap<u64, ExchangeScript>,
    seen_correlations: HashSet<u64>,
    in_flight: HashMap<u64, ActiveExchange>,
    observations: Vec<Observation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActiveExchange {
    method: Method,
    completes_at: ServerTime,
}

#[derive(Debug, Clone)]
pub struct MockService {
    origin: Instant,
    state: std::sync::Arc<Mutex<MockState>>,
}

#[derive(Debug, Clone)]
pub struct MockController {
    origin: Instant,
    state: std::sync::Arc<Mutex<MockState>>,
}

impl MockService {
    pub fn new(config: MockConfig) -> Result<(Self, MockController), MockConfigError> {
        config.validate()?;
        let mut model = CounterModel::new(config.phase_ms);
        for policy in config.policies {
            model
                .insert_policy(policy)
                .map_err(|error| MockConfigError::InvalidPolicy(format!("{error:?}")))?;
        }
        let origin = Instant::now();
        let state = std::sync::Arc::new(Mutex::new(MockState {
            seed: config.seed,
            dispatch_budget: config.dispatch_budget,
            dispatches: 0,
            exhausted: false,
            model,
            routes: config.routes.into_iter().collect(),
            scripts: HashMap::new(),
            seen_correlations: HashSet::new(),
            in_flight: HashMap::new(),
            observations: Vec::new(),
        }));
        Ok((
            Self {
                origin,
                state: state.clone(),
            },
            MockController { origin, state },
        ))
    }

    fn now(&self) -> ServerTime {
        ServerTime::from_millis(
            u64::try_from(Instant::now().duration_since(self.origin).as_millis())
                .unwrap_or(u64::MAX),
        )
    }

    async fn send_inner(&self, request: WireRequest) -> Result<WireResponse, TransportError> {
        let endpoint = Endpoint::from_uri(request.uri()).ok_or_else(|| {
            TransportError::MockHarness("request URI is not a D5 endpoint label".to_owned())
        })?;
        let correlation_id = parse_correlation(request.headers())?;
        let method = request.method().clone();
        if method != Method::GET && method != Method::HEAD {
            return Err(TransportError::MockHarness(
                "mock accepts only GET and HEAD".to_owned(),
            ));
        }

        let script = {
            let mut state = self.state.lock().await;
            state.scripts.remove(&correlation_id).unwrap_or_default()
        };
        tokio::time::sleep(script.arrival_delay).await;

        let arrival = self.now();
        let (ordinal, policy, layer1, judgment, definition, in_flight_at_arrival, head_overlap) = {
            let mut state = self.state.lock().await;
            state
                .in_flight
                .retain(|_, exchange| arrival < exchange.completes_at);
            if state.exhausted || state.dispatches == state.dispatch_budget {
                state.exhausted = true;
                return Err(TransportError::MockHarness(format!(
                    "mock dispatch budget {} exhausted",
                    state.dispatch_budget
                )));
            }
            if !state.seen_correlations.insert(correlation_id) {
                return Err(TransportError::MockHarness(
                    "duplicate run-wide correlation identity".to_owned(),
                ));
            }
            state.dispatches += 1;
            let ordinal = state.dispatches - 1;
            let in_flight_at_arrival = state.in_flight.len() + 1;
            let head_overlap = if method == Method::HEAD {
                !state.in_flight.is_empty()
            } else {
                state
                    .in_flight
                    .values()
                    .any(|active| active.method == Method::HEAD)
            };
            let policy = state.routes.get(&endpoint).cloned().ok_or_else(|| {
                TransportError::MockHarness("endpoint has no policy route".to_owned())
            })?;
            let layer1 = state.model.record_layer1_arrival(arrival);
            let mut judgment = match state.model.judge(
                &policy,
                arrival,
                correlation_id,
                method != Method::HEAD && !layer1.tripped,
            ) {
                Ok(judgment) => judgment,
                Err(error) => {
                    state.exhausted = true;
                    return Err(model_transport_error(error));
                }
            };
            if layer1.tripped {
                judgment.organic_violation = false;
                judgment.retry_after_seconds = None;
            }
            let definition = state
                .model
                .definition(&policy)
                .map_err(model_transport_error)?
                .clone();
            let response_delay_ms = u64::try_from(script.response_delay.as_millis())
                .expect("script delays are structurally capped below u64 milliseconds");
            state.in_flight.insert(
                correlation_id,
                ActiveExchange {
                    method: method.clone(),
                    completes_at: ServerTime::from_millis(
                        arrival.as_millis().saturating_add(response_delay_ms),
                    ),
                },
            );
            (
                ordinal,
                policy,
                layer1,
                judgment,
                definition,
                in_flight_at_arrival,
                head_overlap,
            )
        };

        let mut response =
            build_default_response(endpoint, &method, &definition, &judgment, layer1.tripped)?;
        let stimulus = if layer1.tripped {
            None
        } else {
            apply_override(&mut response, script.response)?
        };
        if let Ok(wire_response) = &mut response {
            emit_date(wire_response.headers_mut(), arrival)?;
        }
        let response_status = response.as_ref().ok().map(Response::status);
        {
            let mut state = self.state.lock().await;
            let seed = state.seed;
            let phase_ms = state.model.phase_ms();
            let completion_ms = state
                .in_flight
                .get(&correlation_id)
                .expect("this exchange was inserted before response construction")
                .completes_at
                .as_millis();
            state.observations.push(Observation {
                ordinal,
                seed,
                phase_ms,
                correlation_id,
                arrival_ms: arrival.as_millis(),
                completion_ms,
                method,
                endpoint,
                policy,
                in_flight_at_arrival,
                head_overlap,
                layer1,
                policy_judgment: judgment,
                stimulus,
                response_status,
            });
        }
        tokio::time::sleep(script.response_delay).await;
        self.state.lock().await.in_flight.remove(&correlation_id);
        response
    }
}

impl Transport for MockService {
    fn send(
        &self,
        request: WireRequest,
    ) -> impl std::future::Future<Output = Result<WireResponse, TransportError>> + Send {
        let this = self.clone();
        async move { this.send_inner(request).await }
    }
}

impl MockController {
    pub fn now(&self) -> ServerTime {
        ServerTime::from_millis(
            u64::try_from(Instant::now().duration_since(self.origin).as_millis())
                .unwrap_or(u64::MAX),
        )
    }

    pub async fn script(
        &self,
        correlation_id: u64,
        script: ExchangeScript,
    ) -> Result<(), MockConfigError> {
        validate_script(&script)?;
        let mut state = self.state.lock().await;
        if state.scripts.contains_key(&correlation_id)
            || state.seen_correlations.contains(&correlation_id)
        {
            return Err(MockConfigError::InvalidScript(
                "duplicate correlation script".to_owned(),
            ));
        }
        if state.scripts.len() == state.dispatch_budget {
            return Err(MockConfigError::InvalidScript(
                "script count exceeds the run dispatch budget".to_owned(),
            ));
        }
        state.scripts.insert(correlation_id, script);
        Ok(())
    }

    pub async fn preload(
        &self,
        policy: &str,
        at: ServerTime,
        count: usize,
    ) -> Result<(), ModelError> {
        self.state.lock().await.model.preload(policy, at, count)
    }

    pub async fn inject_phantoms(&self, policy: &str, count: usize) -> Result<(), ModelError> {
        let now = self.now();
        self.state
            .lock()
            .await
            .model
            .inject_phantoms(policy, now, count)
    }

    pub async fn replace_policy(&self, definition: PolicyDefinition) -> Result<(), ModelError> {
        self.state.lock().await.model.replace_policy(definition)
    }

    pub async fn route(
        &self,
        endpoint: Endpoint,
        policy: impl Into<String>,
    ) -> Result<(), ModelError> {
        let policy = policy.into();
        let mut state = self.state.lock().await;
        state.model.definition(&policy)?;
        state.routes.insert(endpoint, policy);
        Ok(())
    }

    pub async fn observations(&self) -> Vec<Observation> {
        let mut observations = self.state.lock().await.observations.clone();
        observations.sort_by_key(|observation| observation.ordinal);
        observations
    }
}

pub fn request(
    method: Method,
    endpoint: Endpoint,
    correlation_id: u64,
) -> Result<WireRequest, http::Error> {
    Request::builder()
        .method(method)
        .uri(endpoint.uri())
        .header(CORRELATION_HEADER, correlation_id)
        .body(Vec::new())
}

fn parse_correlation(headers: &HeaderMap) -> Result<u64, TransportError> {
    let value = headers.get(&CORRELATION_HEADER).ok_or_else(|| {
        TransportError::MockHarness("missing test correlation identity".to_owned())
    })?;
    if value.as_bytes().len() > 20 {
        return Err(TransportError::MockHarness(
            "test correlation identity exceeds u64 text width".to_owned(),
        ));
    }
    let raw = value.to_str().map_err(|_| {
        TransportError::MockHarness("test correlation identity is not ASCII".to_owned())
    })?;
    if raw.is_empty() || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TransportError::MockHarness(
            "test correlation identity is not a decimal u64".to_owned(),
        ));
    }
    raw.parse().map_err(|_| {
        TransportError::MockHarness("test correlation identity does not fit u64".to_owned())
    })
}

fn model_transport_error(error: ModelError) -> TransportError {
    TransportError::MockHarness(format!("mock counter model: {error:?}"))
}

fn build_default_response(
    endpoint: Endpoint,
    method: &Method,
    definition: &PolicyDefinition,
    judgment: &PolicyJudgment,
    layer1_tripped: bool,
) -> Result<Result<WireResponse, TransportError>, TransportError> {
    if layer1_tripped {
        return Ok(Ok(cloudflare_response()));
    }
    let status = if judgment.organic_violation {
        StatusCode::TOO_MANY_REQUESTS
    } else if *method == Method::HEAD {
        endpoint.head_status()
    } else {
        StatusCode::OK
    };
    let mut response = Response::builder()
        .status(status)
        .body(Vec::new())
        .map_err(|error| {
            TransportError::MockHarness(format!("failed to construct mock response: {error}"))
        })?;
    emit_full_headers(response.headers_mut(), definition, judgment)?;
    if let Some(retry_after) = judgment.retry_after_seconds {
        insert_header(
            response.headers_mut(),
            RETRY_AFTER_HEADER,
            retry_after.to_string(),
        )?;
    }
    Ok(Ok(response))
}

fn emit_full_headers(
    headers: &mut HeaderMap,
    definition: &PolicyDefinition,
    judgment: &PolicyJudgment,
) -> Result<(), TransportError> {
    insert_header(headers, POLICY_HEADER, definition.name().to_owned())?;
    insert_header(
        headers,
        RULES_HEADER,
        definition
            .rules()
            .iter()
            .map(RuleDefinition::name)
            .collect::<Vec<_>>()
            .join(", "),
    )?;
    let mut observation_index = 0;
    for rule in definition.rules() {
        let limit_name = HeaderName::from_bytes(
            format!("x-rate-limit-{}", rule.name().to_ascii_lowercase()).as_bytes(),
        )
        .map_err(|_| TransportError::MockHarness("invalid configured rule name".to_owned()))?;
        let state_name = HeaderName::from_bytes(
            format!("x-rate-limit-{}-state", rule.name().to_ascii_lowercase()).as_bytes(),
        )
        .map_err(|_| TransportError::MockHarness("invalid configured rule name".to_owned()))?;
        let limits = rule
            .windows()
            .iter()
            .map(|window| {
                format!(
                    "{}:{}:{}",
                    window.max_hits(),
                    window.period_ms() / 1_000,
                    window.restriction_ms() / 1_000
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let states = rule
            .windows()
            .iter()
            .map(|window| {
                let observed = &judgment.windows[observation_index];
                observation_index += 1;
                format!(
                    "{}:{}:{}",
                    observed.current_hits,
                    window.period_ms() / 1_000,
                    observed.restriction_active_seconds
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        insert_header(headers, limit_name, limits)?;
        insert_header(headers, state_name, states)?;
    }
    Ok(())
}

fn emit_date(headers: &mut HeaderMap, now: ServerTime) -> Result<(), TransportError> {
    let date = httpdate::fmt_http_date(
        UNIX_EPOCH
            .checked_add(Duration::from_millis(now.as_millis()))
            .unwrap_or(UNIX_EPOCH),
    );
    insert_header(headers, DATE, date)?;
    Ok(())
}

fn insert_header(
    headers: &mut HeaderMap,
    name: HeaderName,
    value: String,
) -> Result<(), TransportError> {
    if value.len() > MAX_MOCK_HEADER_VALUE_BYTES {
        return Err(TransportError::MockHarness(format!(
            "generated header {name} exceeds mock bound"
        )));
    }
    let value = HeaderValue::from_str(&value)
        .map_err(|_| TransportError::MockHarness(format!("generated header {name} is invalid")))?;
    headers.insert(name, value);
    Ok(())
}

fn apply_override(
    response: &mut Result<WireResponse, TransportError>,
    response_override: Option<ResponseOverride>,
) -> Result<Option<StimulusKind>, TransportError> {
    let Some(response_override) = response_override else {
        return Ok(None);
    };
    match response_override {
        ResponseOverride::Full {
            status,
            retry_after,
        } => {
            let full = response.as_mut().map_err(|error| error.clone())?;
            *full.status_mut() = status;
            full.headers_mut().remove(&RETRY_AFTER_HEADER);
            if let Some(retry_after) = retry_after {
                insert_header(full.headers_mut(), RETRY_AFTER_HEADER, retry_after)?;
            }
            Ok(Some(StimulusKind::Full(status)))
        }
        ResponseOverride::PolicyOnly => {
            let full = response.as_mut().map_err(|error| error.clone())?;
            let policy = full.headers().get(&POLICY_HEADER).cloned();
            full.headers_mut().clear();
            if let Some(policy) = policy {
                full.headers_mut().insert(POLICY_HEADER, policy);
            }
            Ok(Some(StimulusKind::PolicyOnly))
        }
        ResponseOverride::Cloudflare => {
            *response = Ok(cloudflare_response());
            Ok(Some(StimulusKind::Cloudflare))
        }
        ResponseOverride::Raw {
            status,
            headers,
            body,
        } => {
            *response = Ok(Response::builder()
                .status(status)
                .body(body)
                .map_err(|error| {
                    TransportError::MockHarness(format!(
                        "failed to construct raw scripted response: {error}"
                    ))
                })?);
            response
                .as_mut()
                .map_err(|error| error.clone())?
                .headers_mut()
                .extend(headers);
            Ok(Some(StimulusKind::Raw(status)))
        }
        ResponseOverride::TransportError(message) => {
            *response = Err(TransportError::MockHarness(message));
            Ok(Some(StimulusKind::TransportError))
        }
    }
}

fn cloudflare_response() -> WireResponse {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header(CF_MITIGATED_HEADER, "challenge")
        .header(SERVER, "cloudflare")
        .header(CONTENT_TYPE, "text/html")
        .body(b"<!doctype html><title>challenge</title>".to_vec())
        .expect("fixed Cloudflare response is valid")
}

fn validate_script(script: &ExchangeScript) -> Result<(), MockConfigError> {
    const MAX_SCRIPT_DELAY: Duration = Duration::from_secs(6 * 60 * 60);
    if script.arrival_delay > MAX_SCRIPT_DELAY || script.response_delay > MAX_SCRIPT_DELAY {
        return Err(MockConfigError::InvalidScript(
            "script delay exceeds the six-hour harness bound".to_owned(),
        ));
    }
    if !script
        .arrival_delay
        .subsec_nanos()
        .is_multiple_of(1_000_000)
        || !script
            .response_delay
            .subsec_nanos()
            .is_multiple_of(1_000_000)
    {
        return Err(MockConfigError::InvalidScript(
            "script delays must be whole simulated milliseconds".to_owned(),
        ));
    }
    if let Some(ResponseOverride::Full {
        retry_after: Some(value),
        ..
    }) = &script.response
    {
        if value.len() > MAX_MOCK_HEADER_VALUE_BYTES {
            return Err(MockConfigError::InvalidScript(
                "scripted Retry-After exceeds the mock header bound".to_owned(),
            ));
        }
        if HeaderValue::from_str(value).is_err() {
            return Err(MockConfigError::InvalidScript(
                "scripted Retry-After is not a valid header value".to_owned(),
            ));
        }
    }
    if let Some(ResponseOverride::TransportError(message)) = &script.response
        && message.len() > 256
    {
        return Err(MockConfigError::InvalidScript(
            "scripted transport diagnostic exceeds 256 bytes".to_owned(),
        ));
    }
    let Some(ResponseOverride::Raw { headers, body, .. }) = &script.response else {
        return Ok(());
    };
    if headers.len() > MAX_RAW_RESPONSE_HEADERS
        || headers
            .values()
            .any(|value| value.as_bytes().len() > MAX_MOCK_HEADER_VALUE_BYTES)
        || body.len() > MAX_RAW_RESPONSE_BODY_BYTES
    {
        return Err(MockConfigError::InvalidScript(
            "raw response exceeds the mock's structural bounds".to_owned(),
        ));
    }
    Ok(())
}
