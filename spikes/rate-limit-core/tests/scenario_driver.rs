//! End-to-end M-series driver.
//!
//! This target deliberately consumes the actor through its public handle and
//! takes gate evidence only from the mock.  The expected eligibility instants
//! below are scenario-script arithmetic, not values reported by the actor.

use std::collections::BTreeMap;
use std::future::{Future, poll_fn};
use std::num::NonZeroU64;
use std::task::Poll;
use std::time::Duration;

use http::{HeaderMap, HeaderValue, Method, StatusCode};
use proptest::prelude::*;
use rate_limit_core::actor::{GateError, GateHandle, RequestTicket, with_correlation_header};
use rate_limit_core::conformance::{
    ClientBucketProfile, ContractCoverage, FullContractRun, Gate, RunReport,
    SHIPPED_ASSUMED_PROFILE, ScenarioId, ScenarioOracle, SweepConfiguration, SweepPlan, judge,
};
use rate_limit_core::core::EndpointLabel;
use rate_limit_core::mock::model::{
    PolicyDefinition, RuleDefinition, WindowDefinition, first_bucket_boundary_ms,
};
use rate_limit_core::mock::{
    CORRELATION_HEADER, DEFAULT_SERVICE_DELAY, Endpoint, ExchangeScript, MockConfig,
    MockController, MockStateChange, MockStateChangeKind, ResponseOverride, request,
};

/// SD-R8-F12: the single run-configuration source. The final external audit
/// forged a state in which the M8 actor ran under `SHIPPED_ASSUMED_PROFILE`
/// while its reproduction record still claimed `OAUTH_KNOWN_PROFILE`, and
/// both authorities passed — the record's profile was a run-owned label
/// unbound to the engine actually exercised. Per the recorded repair
/// approach (`result-draft.md` §9: by construction, not by check), the
/// engine the actor runs and the provenance the reproduction record claims
/// now flow from one `Lane` value: the fields are private to this module,
/// `Lane::start` is the driver's only engine-construction and spawn path,
/// and `Lane::evidence` is the only place a `ReproductionRecord` is built —
/// all three pinned structurally by
/// `f12_driver_has_one_engine_construction_and_one_provenance_path`.
/// A split profile is unrepresentable outside this module; editing the
/// module itself remains the recorded residual trust surface (the wire
/// cannot distinguish the M8 profiles, so no judge-side check can replace
/// this construction binding).
mod lane {
    use std::time::Duration;

    use rate_limit_core::actor::{GateHandle, spawn};
    use rate_limit_core::conformance::{
        ClientBucketProfile, ContractCoverage, ReproductionRecord, RunEvidence,
        SHIPPED_ASSUMED_PROFILE, ScenarioAssertion, ScenarioId, SweepKind, scenario,
    };
    use rate_limit_core::core::{BucketModel, PolicyEngine, Resolution};
    use rate_limit_core::mock::{Endpoint, MockConfig, MockController, MockService};

    pub struct Lane {
        gate: GateHandle,
        controller: MockController,
        profile: ClientBucketProfile,
        endpoint: Endpoint,
    }

    impl Lane {
        /// The driver's sole engine-construction path: the profile that
        /// builds the engine is the profile every later evidence record
        /// claims, because both read the same private field.
        pub fn start(profile: ClientBucketProfile, endpoint: Endpoint, config: MockConfig) -> Self {
            let resolution = |millis| {
                if profile == SHIPPED_ASSUMED_PROFILE {
                    Resolution::Assumed(Duration::from_millis(millis))
                } else {
                    Resolution::Known(Duration::from_millis(millis))
                }
            };
            let engine = PolicyEngine::new(BucketModel::new(
                resolution(profile.burst_ms),
                resolution(profile.sustained_ms),
            ));
            let (mock, controller) = MockService::new(config).expect("mock config is valid");
            let gate = spawn(engine, mock);
            Self {
                gate,
                controller,
                profile,
                endpoint,
            }
        }

        pub fn gate(&self) -> &GateHandle {
            &self.gate
        }

        pub fn controller(&self) -> &MockController {
            &self.controller
        }

        /// The driver's sole reproduction-record constructor. Seed and phase
        /// are read from the wire (and the judge binds them, with the
        /// endpoint, to every observation — SD-R8-F9); the profile can only
        /// be the one `start` built the engine from (SD-R8-F12).
        pub async fn evidence(
            &self,
            id: ScenarioId,
            coverage: ContractCoverage,
            assertion_passed: bool,
        ) -> RunEvidence {
            let spec = scenario(id);
            RunEvidence {
                scenario: id,
                reproduction: (spec.sweep == SweepKind::PhaseSwept).then_some(ReproductionRecord {
                    seed: self
                        .controller
                        .observations()
                        .await
                        .first()
                        .expect("wire run")
                        .seed,
                    endpoint: self.endpoint,
                    phase_ms: self
                        .controller
                        .observations()
                        .await
                        .first()
                        .expect("wire run")
                        .phase_ms,
                    client_buckets: self.profile,
                }),
                observations: self.controller.observations().await,
                state_changes: self.controller.state_changes().await,
                unavoidable_exposure: None,
                assertions: vec![ScenarioAssertion {
                    id: spec.required_assertion,
                    coverage,
                    passed: assertion_passed,
                }],
            }
        }
    }
}

fn wire_request(endpoint: Endpoint) -> rate_limit_core::transport::WireRequest {
    with_correlation_header(
        request(Method::GET, endpoint, 0).expect("fixed mock request is valid"),
        CORRELATION_HEADER,
    )
}

/// Submits through the gate, recording the script's own submission instant.
///
/// §6 measures G3 "whenever a request is **queued** and eligible", but the
/// harness cannot key submissions to wire correlations: `RequestId` and the
/// correlation counter are independent, because the actor allocates a fresh
/// correlation per *dispatch* (probes and retries included).  The oracle
/// consequently uses the latest script submission at or before each observed
/// dispatch as a lower bound. Nothing here is client-reported, which §6
/// forbids as an authorization source.
///
/// This is sound for the current driver shapes: either submissions share one
/// instant (M2, M10, M13) or an outstanding ticket is awaited before the next
/// submission. A scenario that interleaves distinct submission instants with
/// in-flight work needs an explicit script-owned identity map instead.
async fn submit_recorded(
    gate: &GateHandle,
    controller: &MockController,
    submitted_ms: &mut Vec<u64>,
    endpoint: Endpoint,
) -> RequestTicket {
    submitted_ms.push(controller.now().as_millis());
    gate.submit(
        EndpointLabel::from(endpoint.label()),
        wire_request(endpoint),
    )
    .await
    .expect("the gate accepts a submission")
}

/// Simulated-time step.  This is the floor under every G3 measurement: no
/// lateness smaller than one step is observable, so the step must stay well
/// below any epsilon §6 finalizes.  25ms costs ~2s for the whole target,
/// including M10's 300-request run, which is why it is not 250ms.
const ADVANCE_STEP_MS: u64 = 25;

async fn advance(millis: u64) {
    for _ in 0..millis.div_ceil(ADVANCE_STEP_MS) {
        tokio::time::advance(Duration::from_millis(ADVANCE_STEP_MS)).await;
        tokio::task::yield_now().await;
    }
}

/// Poll a caller outcome exactly once.  Unlike awaiting a ticket (or using a
/// timeout), this cannot drive Tokio's paused clock to a later wake-up: it is
/// the M10 assertion that the result was already observable at the boundary.
async fn already_cancelled(ticket: RequestTicket) -> bool {
    let mut ticket = Box::pin(ticket);
    matches!(
        poll_fn(|cx| Poll::Ready(ticket.as_mut().poll(cx))).await,
        Poll::Ready(Err(GateError::Cancelled))
    )
}

async fn serve_within(tickets: Vec<RequestTicket>, budget_ms: u64) -> usize {
    tokio::time::timeout(Duration::from_millis(budget_ms), async move {
        let mut served = 0usize;
        for ticket in tickets {
            served += usize::from(ticket.await.is_ok());
        }
        served
    })
    .await
    .expect("the bounded scenario queue must drain")
}

fn pair_policy(name: &str, burst: u32) -> PolicyDefinition {
    PolicyDefinition::new(
        name,
        vec![
            RuleDefinition::new(
                "Account",
                vec![
                    WindowDefinition::new(burst, 10_000, 60_000, 5_000).unwrap(),
                    WindowDefinition::new(30, 60_000, 300_000, 60_000).unwrap(),
                ],
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn policy_headers(name: &str, triplets: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-rate-limit-policy", HeaderValue::from_str(name).unwrap());
    headers.insert("x-rate-limit-rules", HeaderValue::from_static("Account"));
    headers.insert(
        "x-rate-limit-account",
        HeaderValue::from_str(triplets).unwrap(),
    );
    headers.insert(
        "x-rate-limit-account-state",
        HeaderValue::from_static("0:10:0, 0:60:0"),
    );
    headers
}

#[derive(Default)]
struct DriverOracle {
    /// Explicit scenario-script safe instants keyed by mock correlation.
    /// HEADs are independent probe exclusions, so their own handoff is their
    /// first eligible instant.
    eligible: BTreeMap<u64, u64>,
    /// Script-owned submission instants, ascending; see `submit_recorded`.
    submitted_ms: Vec<u64>,
    m2_minimum_ms: Option<u64>,
}

impl ScenarioOracle for DriverOracle {
    fn independently_eligible_ms(
        &self,
        observation: &rate_limit_core::mock::Observation,
    ) -> Option<u64> {
        // If the judged set ever grows beyond the set used to build this
        // oracle, `None` reaches the judge, which fails G3 closed (the
        // fail-closed branch lives there — SD-R5-F6, superseding the
        // per-implementation u64::MAX sentinel of round-four F16).
        self.eligible.get(&observation.correlation_id).copied()
    }

    fn independently_observable_ms(
        &self,
        state_change: &MockStateChange,
        _: &[rate_limit_core::mock::Observation],
    ) -> Option<u64> {
        Some(state_change.occurred_ms)
    }

    fn m2_theoretical_padded_minimum_ms(
        &self,
        _: &[rate_limit_core::mock::Observation],
    ) -> Option<u64> {
        self.m2_minimum_ms
    }
}

fn m8_fragment_passed(observations: &[rate_limit_core::mock::Observation]) -> bool {
    observations
        .iter()
        .filter(|observation| observation.method == Method::GET)
        .count()
        == 2
}

fn d5_wire_shape_holds(observations: &[rate_limit_core::mock::Observation]) -> bool {
    observations.iter().all(|observation| {
        observation.in_flight_at_arrival <= rate_limit_core::conformance::D5_IN_FLIGHT_CAP
            && !observation.head_overlap
    })
}

async fn run_m1_m13(phase_ms: u64, coverage: ContractCoverage) -> Vec<RunReport> {
    let mut reports = Vec::new();

    // M8 runs the required legacy lane here and its OAuth lane in the
    // shared helper below. M10 intentionally uses the legacy two-rule
    // policy for its long saturation run. Every other row uses the OAuth
    // Known profile so hard-coded OAuth endpoints never inherit the
    // legacy 60s/60s padding model by parity accident.
    let rows: Vec<(ScenarioId, ClientBucketProfile, Endpoint)> = ScenarioId::ALL
        .into_iter()
        .map(|id| match id {
            ScenarioId::M8 | ScenarioId::M10 => {
                (id, SHIPPED_ASSUMED_PROFILE, Endpoint::LegacyStashIndex)
            }
            // M9's arm drives stash-request-limit phantoms and submits to
            // Stash; the row label must state the wire endpoint, and the
            // judge now enforces that binding (SD-R8-F9).
            ScenarioId::M9 => (
                id,
                rate_limit_core::conformance::OAUTH_KNOWN_PROFILE,
                Endpoint::Stash,
            ),
            _ => (
                id,
                rate_limit_core::conformance::OAUTH_KNOWN_PROFILE,
                Endpoint::StashList,
            ),
        })
        .collect();
    // The driver's per-row profile set goes through `SweepPlan::new`, whose
    // constructor rejects a plan without the shipped `Assumed(60s/60s)`
    // default (SD-R5-F3): deleting the last legacy row fails here,
    // structurally, before any scenario body runs. The seeds below come from
    // the plan so this guard is load-bearing, not decorative.
    let plan = SweepPlan::new(
        rows.iter()
            .enumerate()
            .map(|(index, (_, profile, _))| SweepConfiguration {
                seed: 100 + index as u64,
                phase_ms,
                client_buckets: *profile,
            })
            .collect(),
    )
    .expect("the driver's rows must retain the shipped Assumed(60s/60s) default");

    for ((id, profile, endpoint), configuration) in rows.into_iter().zip(plan.configurations()) {
        // The phase is the caller's, verbatim: every row runs at each swept
        // phase.  Folding the offset through a per-row modulus (the original
        // `(offset + index * 997) % 60_000`) collapsed the two sweeps to 1 ms
        // apart for every row but M1, because 59_999 == -1 (mod 60_000).
        let mut config = MockConfig::n23(configuration.seed, configuration.phase_ms);
        config.dispatch_budget = 1_024;
        let lane = lane::Lane::start(profile, endpoint, config);
        let gate = lane.gate().clone();
        let controller = lane.controller().clone();
        let mut submitted_ms = Vec::new();

        // Each arm yields its scenario fragment's verdict rather than
        // asserting it, so a failure reaches G5 and is reported with the
        // whole gate matrix instead of panicking in isolation.
        let assertion_passed = match id {
            ScenarioId::M1 => {
                controller
                    .preload("stash-list-request-limit", controller.now(), 1)
                    .await
                    .unwrap();
                let ticket =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                advance(2_000).await;
                let granted = ticket.await.is_ok();
                let observations = controller.observations().await;
                granted
                    && observations
                        .iter()
                        .filter(|o| o.method == Method::HEAD)
                        .count()
                        == 1
                    && observations.len() == 2
            }
            ScenarioId::M2 => {
                let mut tickets = Vec::new();
                for _ in 0..M2_QUEUE_DEPTH {
                    tickets.push(
                        submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                            .await,
                    );
                }
                advance(M2_RUN_MS).await;
                let mut granted = true;
                for ticket in tickets {
                    granted &= ticket.await.is_ok();
                }
                let observations = controller.observations().await;
                let dispatches = observations
                    .iter()
                    .filter(|observation| observation.method == Method::GET)
                    .map(|observation| observation.dispatch_ms)
                    .collect::<Vec<_>>();
                let stalls = dispatches
                    .windows(2)
                    .enumerate()
                    .filter(|(index, pair)| {
                        (index + 1) % 10 == 0
                            && pair[1].saturating_sub(pair[0]) > MIN_SEND_SPACING_MS
                    })
                    .count();
                granted && dispatches.len() == M2_QUEUE_DEPTH && stalls == 3
            }
            ScenarioId::M3 => {
                controller
                    .script(
                        1,
                        ExchangeScript {
                            response: Some(ResponseOverride::PolicyOnly),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await;
                advance(1_000).await;
                let refused = matches!(ticket.await, Err(GateError::SetupFailed { .. }));
                refused && controller.observations().await.len() == 1
            }
            ScenarioId::M4 => {
                controller
                    .script(
                        1,
                        ExchangeScript {
                            response: Some(ResponseOverride::Raw {
                                status: StatusCode::NO_CONTENT,
                                headers: policy_headers("bad-shape", "1:10:60"),
                                body: Vec::new(),
                            }),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await;
                advance(1_000).await;
                let refused = matches!(ticket.await, Err(GateError::SetupFailed { .. }));
                refused && controller.observations().await.len() == 1
            }
            ScenarioId::M5 => {
                let first =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .rename_policy("stash-list-request-limit", pair_policy("renamed-limit", 10))
                    .await
                    .unwrap();
                let second =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                advance(1_000).await;
                let second_granted = second.await.is_ok();
                first_granted
                    && second_granted
                    && controller
                        .observations()
                        .await
                        .iter()
                        .filter(|o| o.method == Method::HEAD)
                        .count()
                        == 1
            }
            ScenarioId::M6 => {
                let first =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .replace_policy(pair_policy("stash-list-request-limit", 5))
                    .await
                    .unwrap();
                let second =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                advance(1_000).await;
                let second_granted = second.await.is_ok();
                if coverage == ContractCoverage::FullContract {
                    let mut queued = Vec::new();
                    for _ in 0..M6_POST_SHRINK_QUEUE {
                        queued.push(
                            submit_recorded(
                                &gate,
                                &controller,
                                &mut submitted_ms,
                                Endpoint::StashList,
                            )
                            .await,
                        );
                    }
                    let served = serve_within(queued, M6_DRAIN_BUDGET_MS).await;
                    let observations = controller.observations().await;
                    first_granted
                        && second_granted
                        && served == M6_POST_SHRINK_QUEUE
                        && observations
                            .iter()
                            .all(|observation| !observation.policy_judgment.organic_violation)
                } else {
                    first_granted && second_granted
                }
            }
            ScenarioId::M7 => {
                let first =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .inject_phantoms(
                        "stash-list-request-limit",
                        if coverage == ContractCoverage::FullContract {
                            M7_PHANTOM_BURST
                        } else {
                            2
                        },
                    )
                    .await
                    .unwrap();
                let second =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                advance(1_000).await;
                let second_granted = second.await.is_ok();
                if coverage == ContractCoverage::FullContract {
                    let mut queued = Vec::new();
                    for _ in 0..M7_POST_PHANTOM_QUEUE {
                        queued.push(
                            submit_recorded(
                                &gate,
                                &controller,
                                &mut submitted_ms,
                                Endpoint::StashList,
                            )
                            .await,
                        );
                    }
                    let served = serve_within(queued, M7_DRAIN_BUDGET_MS).await;
                    let observations = controller.observations().await;
                    first_granted
                        && second_granted
                        && served == M7_POST_PHANTOM_QUEUE
                        && observations.iter().any(|observation| {
                            observation
                                .policy_judgment
                                .windows
                                .iter()
                                .any(|window| window.phantom_hits >= M7_PHANTOM_BURST as u32)
                        })
                        && observations
                            .iter()
                            .all(|observation| !observation.policy_judgment.organic_violation)
                } else {
                    first_granted && second_granted
                }
            }
            ScenarioId::M8 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response: Some(ResponseOverride::Full {
                                status: StatusCode::TOO_MANY_REQUESTS,
                                retry_after: Some("0".to_owned()),
                            }),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let request_count = if coverage == ContractCoverage::FullContract {
                    M8_RECOVERY_QUEUE
                } else {
                    1
                };
                let mut tickets = Vec::new();
                for _ in 0..request_count {
                    tickets.push(
                        submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await,
                    );
                }
                let served = serve_within(tickets, M8_DRAIN_BUDGET_MS).await;
                let observations = controller.observations().await;
                if coverage == ContractCoverage::FullContract {
                    served == request_count
                        && observations
                            .iter()
                            .filter(|observation| {
                                observation.response_status == Some(StatusCode::TOO_MANY_REQUESTS)
                            })
                            .count()
                            == 1
                        && observations
                            .iter()
                            .all(|observation| !observation.policy_judgment.organic_violation)
                } else {
                    served == 1 && m8_fragment_passed(&observations)
                }
            }
            ScenarioId::M9 => {
                let first =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::Stash).await;
                advance(1_000).await;
                let first_granted = first.await.is_ok();
                controller
                    .inject_phantoms("stash-request-limit", 1)
                    .await
                    .unwrap();
                let second =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::Stash).await;
                advance(1_000).await;
                first_granted && second.await.is_ok()
            }
            ScenarioId::M10 => {
                // Keep the first ordinary request on the wire while its
                // caller cancels. The mock handoff and published active count
                // prove this ticket is dispatched before cancellation; the
                // delayed response must still reconcile later.
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response_delay: Duration::from_secs(5),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let dispatched_ticket =
                    submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await;
                advance(1_000).await;
                let was_dispatched = controller
                    .handoffs()
                    .await
                    .iter()
                    .any(|handoff| handoff.method == Method::GET)
                    && gate.subscribe_status().borrow().ordinary_in_flight == 1;
                dispatched_ticket.cancel().await.unwrap();
                advance(M10_PROMPT_CANCEL_MS).await;
                let dispatched_cancelled = already_cancelled(dispatched_ticket).await;

                let mut tickets = Vec::new();
                for _ in 0..M10_PRESSURE_REQUESTS {
                    tickets.push(
                        submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await,
                    );
                }
                // Cancellations are spread through the queue, not taken off
                // one end: a cancel that only ever hits the tail never tests
                // removal from the middle of the deque.
                let mut cancelled_tickets = Vec::new();
                let mut remaining = Vec::new();
                for (index, ticket) in tickets.into_iter().enumerate() {
                    if index % M10_CANCEL_EVERY == 0 {
                        ticket.cancel().await.unwrap();
                        cancelled_tickets.push(ticket);
                    } else {
                        remaining.push(ticket);
                    }
                }
                let cancelled = cancelled_tickets.len();
                advance(M10_PROMPT_CANCEL_MS).await;
                let mut promptly_cancelled = 0usize;
                for ticket in cancelled_tickets {
                    promptly_cancelled += usize::from(already_cancelled(ticket).await);
                }
                let served = serve_within(remaining, M10_RUN_MS).await;
                let observations = controller.observations().await;
                let expected_served = M10_PRESSURE_REQUESTS - cancelled;
                let reconciled = gate.subscribe_status().borrow().ordinary_in_flight == 0;

                // M10's stated asserts, each read off mock-owned wire evidence
                // or the actor's published status.
                //
                // The wire count is bounded on both sides rather than pinned to
                // a literal: a cancel issued under pressure may or may not beat
                // its own dispatch, so between zero and `cancelled` of them
                // legitimately reach the server.  Nothing else may.  With
                // `served` pinned exactly, a wedge cannot hide inside the band.
                let floor = expected_served + 2;
                let drained = served == expected_served
                    && promptly_cancelled == cancelled
                    && was_dispatched
                    && dispatched_cancelled
                    && reconciled
                    && (floor..=floor + cancelled).contains(&observations.len());
                let fuse_quiet = !gate.subscribe_status().borrow().halted;
                let capped = d5_wire_shape_holds(&observations);
                // "Spacing floor never violated" is absolute arithmetic over
                // the wire log: consecutive dispatches, HEADs included (N2's
                // incident was a HEAD flood), never closer than the floor.
                let mut dispatches = observations
                    .iter()
                    .map(|o| o.dispatch_ms)
                    .collect::<Vec<_>>();
                dispatches.sort_unstable();
                let paced = dispatches
                    .windows(2)
                    .all(|pair| pair[1].saturating_sub(pair[0]) >= MIN_SEND_SPACING_MS);
                // Sustained saturation is the point of the row: assert the run
                // actually spanned many minutes and many window rollovers,
                // otherwise "fuse did not trip" is a claim about a short run.
                let sustained = dispatches
                    .last()
                    .zip(dispatches.first())
                    .is_some_and(|(last, first)| last - first >= M10_MIN_SPAN_MS);

                drained && fuse_quiet && capped && paced && sustained
            }
            ScenarioId::M11 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response: Some(ResponseOverride::Cloudflare),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await;
                advance(1_000).await;
                let halted_caller = matches!(ticket.await, Err(GateError::Halted));
                halted_caller && gate.subscribe_status().borrow().halted
            }
            ScenarioId::M12 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response: Some(ResponseOverride::Full {
                                status: StatusCode::UNAUTHORIZED,
                                retry_after: None,
                            }),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let ticket = submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await;
                advance(1_000).await;
                let completed = ticket.await.is_ok();
                completed
                    && controller
                        .observations()
                        .await
                        .iter()
                        .filter(|o| o.method == Method::GET)
                        .count()
                        == 1
            }
            ScenarioId::M13 => {
                controller
                    .script(
                        2,
                        ExchangeScript {
                            response_delay: Duration::from_secs(2),
                            ..ExchangeScript::default()
                        },
                    )
                    .await
                    .unwrap();
                let first =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList)
                        .await;
                let second =
                    submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::Stash).await;
                advance(6_000).await;
                let first_granted = first.await.is_ok();
                let second_granted = second.await.is_ok();
                first_granted
                    && second_granted
                    && d5_wire_shape_holds(&controller.observations().await)
            }
        };

        let evidence = lane.evidence(id, coverage, assertion_passed).await;
        // These rows accrue policy debt at full-contract scale. M7's phantom
        // changes are merged into the B13-derived arrivals below; M1's residue
        // preload is not observable there and therefore stays on its dedicated
        // scenario oracle. See `PolicyDebt`.
        let debt = match id {
            ScenarioId::M2 | ScenarioId::M6 | ScenarioId::M7 | ScenarioId::M10 => Some(
                PolicyDebt::for_policy(
                    &controller,
                    &evidence.observations.first().expect("wire run").policy,
                )
                .await,
            ),
            _ => None,
        };
        let mut oracle = independently_scripted_oracle(
            id,
            &evidence.observations,
            &evidence.state_changes,
            debt.as_ref(),
            &submitted_ms,
        );
        if id == ScenarioId::M2 {
            let definition = controller
                .definition(&evidence.observations.first().expect("wire run").policy)
                .await
                .expect("M2 policy is configured by the scenario");
            oracle.m2_minimum_ms = Some(m2_theoretical_padded_minimum_ms(
                &definition,
                M2_QUEUE_DEPTH,
            ));
        }
        let report = judge(&evidence, &oracle).unwrap();
        if coverage == ContractCoverage::Fragment {
            assert!(report.passed(), "{id:?}: {report:?}");
        }
        reports.push(report);
    }

    assert_eq!(
        reports.len(),
        ScenarioId::ALL.len(),
        "reachability: every M row ran"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.gate(Gate::G1).passed && report.gate(Gate::G2).passed)
    );
    match coverage {
        ContractCoverage::Fragment => assert!(
            reports.iter().all(|report| !report.verdict_eligible()),
            "no fragment run may be verdict-eligible"
        ),
        ContractCoverage::FullContract => assert!(
            reports
                .iter()
                .all(|report| report.contract_coverage == ContractCoverage::FullContract)
        ),
    }

    reports
}

async fn run_m8_oauth_lane(phase_ms: u64, coverage: ContractCoverage) -> RunReport {
    let profile = rate_limit_core::conformance::OAUTH_KNOWN_PROFILE;
    let lane = lane::Lane::start(profile, Endpoint::Stash, MockConfig::n23(808, phase_ms));
    let gate = lane.gate().clone();
    let controller = lane.controller().clone();
    controller
        .script(
            2,
            ExchangeScript {
                response: Some(ResponseOverride::Full {
                    status: StatusCode::TOO_MANY_REQUESTS,
                    retry_after: Some("0".to_owned()),
                }),
                ..ExchangeScript::default()
            },
        )
        .await
        .unwrap();
    let mut submitted_ms = Vec::new();
    let request_count = if coverage == ContractCoverage::FullContract {
        M8_RECOVERY_QUEUE
    } else {
        1
    };
    let mut tickets = Vec::new();
    for _ in 0..request_count {
        tickets.push(submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::Stash).await);
    }
    let served = serve_within(tickets, M8_DRAIN_BUDGET_MS).await;
    let observations = controller.observations().await;
    let assertion_passed = if coverage == ContractCoverage::FullContract {
        served == request_count
            && observations
                .iter()
                .filter(|observation| {
                    observation.response_status == Some(StatusCode::TOO_MANY_REQUESTS)
                })
                .count()
                == 1
            && observations
                .iter()
                .all(|observation| !observation.policy_judgment.organic_violation)
    } else {
        served == 1 && m8_fragment_passed(&observations)
    };
    let evidence = lane
        .evidence(ScenarioId::M8, coverage, assertion_passed)
        .await;
    let report = judge(
        &evidence,
        &independently_scripted_oracle(
            ScenarioId::M8,
            &evidence.observations,
            &evidence.state_changes,
            None,
            &submitted_ms,
        ),
    )
    .unwrap();
    assert!(report.passed(), "OAuth M8: {report:?}");
    assert_eq!(
        report.verdict_eligible(),
        coverage == ContractCoverage::FullContract
    );
    report
}

/// SD-R8-F5's character-policy lanes: the M2 saturation shape run against a
/// character endpoint, whose N23 policies carry the topology's tightest
/// limits. The gate matrix, policy-debt oracle, and G4 minimum are the same
/// policy-generic machinery the main M2 row uses — nothing here is tuned to
/// the policy's numbers except the independently declared policy name the
/// assertion pins. The queue depth is identical at both coverage levels;
/// only the coverage flag differs, exactly like the main M2 row.
async fn run_character_policy_lane(
    endpoint: Endpoint,
    expected_policy: &str,
    seed: u64,
    phase_ms: u64,
    coverage: ContractCoverage,
) -> RunReport {
    let profile = rate_limit_core::conformance::OAUTH_KNOWN_PROFILE;
    let lane = lane::Lane::start(profile, endpoint, MockConfig::n23(seed, phase_ms));
    let gate = lane.gate().clone();
    let controller = lane.controller().clone();
    let mut submitted_ms = Vec::new();
    let mut tickets = Vec::new();
    for _ in 0..CHARACTER_LANE_QUEUE {
        tickets.push(submit_recorded(&gate, &controller, &mut submitted_ms, endpoint).await);
    }
    let served = serve_within(tickets, CHARACTER_LANE_DRAIN_BUDGET_MS).await;
    let observations = controller.observations().await;
    // The vacuity pin: every wire observation must have been judged under
    // the expected character policy. A routing mistake toward a looser
    // policy would otherwise let every gate pass without ever exercising
    // the tight limits this lane exists to cover.
    let on_policy = observations
        .iter()
        .all(|observation| observation.policy == expected_policy);
    let dispatched = observations
        .iter()
        .filter(|observation| observation.method == Method::GET)
        .count();
    let assertion_passed =
        served == CHARACTER_LANE_QUEUE && dispatched == CHARACTER_LANE_QUEUE && on_policy;
    let evidence = lane
        .evidence(ScenarioId::M2, coverage, assertion_passed)
        .await;
    let policy = evidence
        .observations
        .first()
        .expect("wire run")
        .policy
        .clone();
    let debt = PolicyDebt::for_policy(&controller, &policy).await;
    let mut oracle = independently_scripted_oracle(
        ScenarioId::M2,
        &evidence.observations,
        &evidence.state_changes,
        Some(&debt),
        &submitted_ms,
    );
    let definition = controller
        .definition(&policy)
        .await
        .expect("the lane's character policy is configured by N23");
    oracle.m2_minimum_ms = Some(m2_theoretical_padded_minimum_ms(
        &definition,
        CHARACTER_LANE_QUEUE,
    ));
    let report = judge(&evidence, &oracle).unwrap();
    assert!(report.passed(), "{endpoint:?} lane: {report:?}");
    assert_eq!(
        report.verdict_eligible(),
        coverage == ContractCoverage::FullContract
    );
    report
}

async fn run_m1_residue_sweep(phase_ms: u64) {
    const RESIDUE_CASES: [usize; 4] = [0, 1, 9, 10];
    let profile = rate_limit_core::conformance::OAUTH_KNOWN_PROFILE;

    for residue in RESIDUE_CASES {
        let lane = lane::Lane::start(
            profile,
            Endpoint::StashList,
            MockConfig::n23(1_000 + residue as u64, phase_ms),
        );
        let gate = lane.gate().clone();
        let controller = lane.controller().clone();
        if residue > 0 {
            controller
                .preload("stash-list-request-limit", controller.now(), residue)
                .await
                .unwrap();
        }
        let mut submitted_ms = Vec::new();
        let ticket =
            submit_recorded(&gate, &controller, &mut submitted_ms, Endpoint::StashList).await;
        advance(21_000).await;
        let served = ticket.await.is_ok();
        let observations = controller.observations().await;
        let head = observations
            .iter()
            .find(|observation| observation.method == Method::HEAD)
            .expect("M1 boot HEAD");
        let get = observations
            .iter()
            .find(|observation| observation.method == Method::GET)
            .expect("M1 opening GET");
        let zero_budget_waited = residue < 10 || get.dispatch_ms >= M1_ZERO_BUDGET_WAIT_MS;
        let assertion_passed = served
            && observations.len() == 2
            && !head.policy_judgment.counted
            && !get.policy_judgment.organic_violation
            && zero_budget_waited;

        let evidence = lane
            .evidence(ScenarioId::M1, ContractCoverage::Fragment, assertion_passed)
            .await;
        let mut oracle = DriverOracle {
            submitted_ms,
            ..DriverOracle::default()
        };
        oracle.eligible.insert(head.correlation_id, 0);
        // At the zero-budget boundary the HEAD completes with the residue
        // observation at the B12 default delay. Independent contract
        // arithmetic keeps those
        // observed hits active for the 15 s burst period plus its full 5 s
        // bucket (N13). Below the limit, only D5's 250 ms floor applies.
        oracle.eligible.insert(
            get.correlation_id,
            if residue == 10 {
                head.completion_ms.saturating_add(M1_ZERO_BUDGET_WAIT_MS)
            } else {
                MIN_SEND_SPACING_MS
            },
        );
        let report = judge(&evidence, &oracle).unwrap();
        assert!(
            report.passed(),
            "M1 residue={residue}, phase={phase_ms}: {report:?}"
        );
        assert!(!report.verdict_eligible());
    }
}

/// The two swept phases sit at opposite ends of the *boundary distance*, which
/// is the quantity that actually varies the server's bucket alignment: phi=0
/// puts the first boundary a full bucket away, phi=1 puts it 1 ms after t0.
///
/// `phase_ms` names the upcoming boundary, so phi=59,999 is 1 ms from phi=0,
/// not from an immediate boundary — round one's F1 and the re-review's F7 were
/// both that misreading. `swept_phases_are_separated_by_a_full_bucket` pins
/// the real distances so a third mistake fails a test instead of a review.
const SWEPT_PHASES_MS: [u64; 2] = [0, 1];

const BURST_BUCKET_MS: u64 = 5_000;
const SUSTAINED_BUCKET_MS: u64 = 60_000;

/// D5's dispatch floor, restated here as a literal.  The driver must not
/// import the actor's constant: "spacing floor never violated" is checked
/// against the contract's number, not against whatever the actor believes.
const MIN_SEND_SPACING_MS: u64 = 250;

/// N19's applicable bucket — the maximum configured resolution across the
/// policy's windows, 60 s in both lanes — plus the one-second buffer.
/// Scenario arithmetic, derived here rather than left as a bare 61,000
/// (SD-R5-F14).
const APPLICABLE_BUCKET_AND_BUFFER_MS: u64 = SUSTAINED_BUCKET_MS + 1_000;

/// The M1 endpoint's burst period from the N23 stash-list definition
/// (10 hits / 15 s), independently declared: the oracle must not read it
/// from the client.
const M1_BURST_PERIOD_MS: u64 = 15_000;
/// M1's zero-budget wait: the residue hits stay active for the burst period
/// plus its full Known 5 s bucket (N13). Derived, not a bare 20,000
/// (SD-R5-F14).
const M1_ZERO_BUDGET_WAIT_MS: u64 = M1_BURST_PERIOD_MS + BURST_BUCKET_MS;

/// Forty requests fill three ten-hit burst windows and then force the
/// sustained 30-hit window to age before the final burst can drain.
const M2_QUEUE_DEPTH: usize = 40;
const M2_RUN_MS: u64 = 130_000;

/// Full-contract closure scale for M6: enough post-announcement work to fill
/// the shrunk five-hit burst window twice and prove the queue resumes across
/// more than one new-pace stall.
const M6_POST_SHRINK_QUEUE: usize = 12;
const M6_DRAIN_BUDGET_MS: u64 = 500_000;

/// M7 calls for occasional, bursty phantom traffic rather than drizzle. Eight
/// same-instant hits leave one slot for the observing GET after the opening
/// client hit, then the queued tail must drain under the reconciled debt.
const M7_PHANTOM_BURST: usize = 8;
const M7_POST_PHANTOM_QUEUE: usize = 12;
const M7_DRAIN_BUDGET_MS: u64 = 500_000;

/// M8's full-contract lane keeps a queue behind the injected restriction so
/// the confirmation and every follow-on dispatch are mock-judged.
const M8_RECOVERY_QUEUE: usize = 12;
const M8_DRAIN_BUDGET_MS: u64 = 500_000;

/// Independently declared N23 character-policy facts (SD-R8-F5). The lanes'
/// reachability pins must not read these from the client or the mock's
/// runtime config: character-list is 2 hits/10 s burst and 5 hits/300 s
/// sustained — the topology's tightest limits — and character is
/// 5 hits/10 s burst.
const CHARACTER_LIST_BURST_HITS: usize = 2;
const CHARACTER_LIST_SUSTAINED_HITS: usize = 5;
const CHARACTER_BURST_HITS: usize = 5;

/// SD-R8-F5's character-policy lanes run the M2 saturation shape against
/// both character endpoints at every coverage level. Twelve requests cross
/// the character burst window twice and, on the character-list policy,
/// exceed its sustained window repeatedly, so the padded wait regime at
/// very small limits — one mistimed request is a 50% budget error at
/// limit 2 — is forced on every run rather than loaded-but-idle.
const CHARACTER_LANE_QUEUE: usize = 12;
/// Generous simulated ceiling: the character-list lane's padded greedy
/// minimum spans ~720 s (pinned below); the lane asserts it drained, not
/// that it fit a tight budget.
const CHARACTER_LANE_DRAIN_BUDGET_MS: u64 = 1_500_000;

/// Conservative resolution of SD-R8-F1's unspecified full-contract scale:
/// use the spike's established evidence scale rather than proptest's 256-case
/// default. The ignored run owns this declaration; ordinary fragment tests do
/// not inherit the cost.
const FULL_CONTRACT_PROPERTY_CASES: u32 = 4_096;

#[test]
fn full_contract_scale_reaches_every_fragment_closure_shape() {
    let property_cases = FULL_CONTRACT_PROPERTY_CASES;
    let m6_post_shrink_queue = M6_POST_SHRINK_QUEUE;
    let m7_phantom_burst = M7_PHANTOM_BURST;
    let m7_post_phantom_queue = M7_POST_PHANTOM_QUEUE;
    let m8_recovery_queue = M8_RECOVERY_QUEUE;

    assert_eq!(property_cases, 4_096);
    assert!(
        m6_post_shrink_queue > 2 * 5,
        "M6 must cross two complete five-hit new-pace windows"
    );
    assert!(m7_phantom_burst > 1, "M7's stimulus must be bursty");
    assert_eq!(1 + m7_phantom_burst + 1, 10);
    assert!(m7_post_phantom_queue > 10);
    assert!(m8_recovery_queue > 10);

    // SD-R8-F5 lane reachability: the queue must cross the character burst
    // window twice, and must exceed the character-list burst and sustained
    // limits, so every lane run is forced through padded waits at the
    // topology's tightest limits rather than passing while idle.
    let character_lane_queue = CHARACTER_LANE_QUEUE;
    assert!(
        character_lane_queue > 2 * CHARACTER_BURST_HITS,
        "the lane must cross two complete character burst windows"
    );
    assert!(character_lane_queue > 2 * CHARACTER_LIST_BURST_HITS);
    assert!(
        character_lane_queue > CHARACTER_LIST_SUSTAINED_HITS,
        "the character-list lane must exceed its five-hit sustained window"
    );
}

/// SD-R8-F12's structural pin, in the X2 single-send-path pattern: the
/// driver has exactly one engine-construction path, one actor spawn, and one
/// reproduction-record constructor, all inside `mod lane`, whose provenance
/// fields are private. A future second path — the shape the audit's
/// split-profile mutation needed — fails here by count. concat! keeps every
/// needle out of this test's own literals (the SD-R5-F5 vacuity lesson);
/// this is a lexical spike pin, not a Rust parser, so a rename requires
/// re-deriving the needles deliberately.
#[test]
fn f12_driver_has_one_engine_construction_and_one_provenance_path() {
    let source = include_str!("scenario_driver.rs");
    assert_eq!(
        source.matches(concat!("PolicyEngine", "::new(")).count(),
        1,
        "the driver must have exactly one engine-construction path"
    );
    assert_eq!(
        source.matches(concat!("spawn", "(")).count(),
        1,
        "the driver must spawn the actor from exactly one place"
    );
    assert_eq!(
        source.matches(concat!("fn ", "start(")).count(),
        1,
        "Lane::start must remain the unique lane constructor"
    );
    assert_eq!(
        source.matches(concat!("ReproductionRecord", " {")).count(),
        1,
        "reproduction provenance must be built only by Lane::evidence"
    );
    // The provenance fields are private to `mod lane`: present as private
    // declarations, never as `pub` fields a caller could overwrite after
    // construction.
    assert!(source.contains(concat!("        profile: ", "ClientBucketProfile,")));
    assert!(source.contains(concat!("        endpoint: ", "Endpoint,")));
    assert!(!source.contains(concat!("pub ", "profile:")));
    assert!(!source.contains(concat!("pub ", "endpoint:")));
}

/// M10's Tom-approved prompt-cancellation bound. It is one harness tick,
/// deliberately far below the D5 send floor: cancellation is command ingress,
/// not a paced send, and the actor `select!`s its inbox while waiting.
const M10_PROMPT_CANCEL_MS: u64 = ADVANCE_STEP_MS;

/// M10's scale, per `scenarios.md`: "hundreds of enqueues, cancellations,
/// sustained for many simulated minutes."  The endpoint routes to
/// `backend-item-request-limit` (Account 30/60s and 100/1800s), so a few
/// hundred requests is inherently a multi-window, multi-minute run — the
/// policy, not the driver, sets the duration.
const M10_PRESSURE_REQUESTS: usize = 300;
const M10_CANCEL_EVERY: usize = 10;
/// Generous simulated ceiling; the row asserts the run actually drained
/// rather than that it fit in this budget.
const M10_RUN_MS: u64 = 4 * 60 * 60 * 1_000;
/// Floor on the run's observed span, so "the fuse did not trip" is a claim
/// about sustained saturation across many 60s windows rather than a claim
/// about a short burst that never had the chance.
const M10_MIN_SPAN_MS: u64 = 30 * 60 * 1_000;

/// Independent G4 minimum for M2.
///
/// This is scenario arithmetic over the runtime policy definition and queue
/// depth. It does not call the core scheduler or mock counter helpers. Each
/// prior GET remains locally active for `period + bucket` (N13); the next
/// greedy dispatch is the first instant satisfying every window and D5's
/// global floor. The boot HEAD is at t=0 and the final response adds the
/// scenario's configured default service delay.
fn m2_theoretical_padded_minimum_ms(definition: &PolicyDefinition, queue_depth: usize) -> u64 {
    let windows = definition
        .rules()
        .iter()
        .flat_map(|rule| rule.windows())
        .map(|window| {
            (
                usize::try_from(window.max_hits()).expect("u32 fits usize"),
                window.period_ms().saturating_add(window.bucket_ms()),
            )
        })
        .collect::<Vec<_>>();
    assert!(!windows.is_empty(), "M2 policy must contain a window");
    assert!(
        windows.iter().all(|(max_hits, _)| *max_hits > 0),
        "M2 policy must be schedulable"
    );

    let mut dispatches = Vec::<u64>::with_capacity(queue_depth);
    let mut prior_dispatch = 0_u64; // boot HEAD
    for _ in 0..queue_depth {
        let mut candidate = prior_dispatch.saturating_add(MIN_SEND_SPACING_MS);
        loop {
            let mut required = candidate;
            for &(max_hits, padded_lifetime_ms) in &windows {
                let active = dispatches
                    .iter()
                    .copied()
                    .filter(|at| candidate < at.saturating_add(padded_lifetime_ms))
                    .collect::<Vec<_>>();
                if active.len() >= max_hits {
                    let must_expire = active.len() - max_hits + 1;
                    required =
                        required.max(active[must_expire - 1].saturating_add(padded_lifetime_ms));
                }
            }
            if required == candidate {
                break;
            }
            candidate = required;
        }
        dispatches.push(candidate);
        prior_dispatch = candidate;
    }

    dispatches.last().copied().unwrap_or(0).saturating_add(
        u64::try_from(DEFAULT_SERVICE_DELAY.as_millis()).expect("service delay fits u64"),
    )
}

#[test]
fn m2_g4_minimum_is_runtime_derived_and_reaches_both_stalls() {
    let config = MockConfig::n23(2, 0);
    let definition = config
        .policies
        .iter()
        .find(|definition| definition.name() == "stash-list-request-limit")
        .expect("N23 M2 policy");
    assert_eq!(M2_QUEUE_DEPTH, 40);
    assert_eq!(
        m2_theoretical_padded_minimum_ms(definition, M2_QUEUE_DEPTH),
        122_581
    );
    assert!(
        m2_theoretical_padded_minimum_ms(definition, M2_QUEUE_DEPTH)
            > m2_theoretical_padded_minimum_ms(definition, 30),
        "the final burst must wait for sustained-window capacity"
    );
}

/// SD-R8-F5 lane fingerprints, hand-derived from the padded greedy schedule
/// over the independently declared character policies (boot HEAD at t=0,
/// D5's 250 ms floor, N13 `period + bucket` lifetimes, final 81 ms service
/// delay). Character-list's 720,581 ms shows twelve requests forced through
/// both the 2-hit burst window and two full 5-hit/300 s sustained waves;
/// character's 30,581 ms shows two complete 5-hit burst windows crossed.
#[test]
fn character_lane_g4_minimums_are_runtime_derived_and_span_padded_waits() {
    let config = MockConfig::n23(2, 0);
    let definition = |name: &str| {
        config
            .policies
            .iter()
            .find(|definition| definition.name() == name)
            .expect("N23 character policy")
    };
    assert_eq!(
        m2_theoretical_padded_minimum_ms(
            definition("character-list-request-limit"),
            CHARACTER_LANE_QUEUE
        ),
        720_581
    );
    assert_eq!(
        m2_theoretical_padded_minimum_ms(
            definition("character-request-limit"),
            CHARACTER_LANE_QUEUE
        ),
        30_581
    );
    assert!(
        m2_theoretical_padded_minimum_ms(
            definition("character-list-request-limit"),
            CHARACTER_LANE_QUEUE
        ) > m2_theoretical_padded_minimum_ms(
            definition("character-list-request-limit"),
            CHARACTER_LIST_SUSTAINED_HITS
        ),
        "the lane must wait on sustained-window capacity, not only bursts"
    );
}

#[test]
fn swept_phases_are_separated_by_a_full_bucket() {
    let burst = NonZeroU64::new(BURST_BUCKET_MS).unwrap();
    let sustained = NonZeroU64::new(SUSTAINED_BUCKET_MS).unwrap();
    let [immediate_boundary, full_bucket_away] = [1, 0];

    // Both N23 bucket sizes, both swept phases, as literal distances.
    assert_eq!(first_bucket_boundary_ms(burst, full_bucket_away), 5_000);
    assert_eq!(
        first_bucket_boundary_ms(sustained, full_bucket_away),
        60_000
    );
    assert_eq!(first_bucket_boundary_ms(burst, immediate_boundary), 1);
    assert_eq!(first_bucket_boundary_ms(sustained, immediate_boundary), 1);

    // The trap itself, pinned: a phase just under the bucket length is 1 ms
    // from phase 0, which is why 59,999 was not an adversarial second phase.
    assert_eq!(first_bucket_boundary_ms(sustained, 59_999), 59_999);

    // The real guard, stated over whatever SWEPT_PHASES_MS holds: the sweep is
    // only a sweep if its phases move the first boundary by nearly a whole
    // bucket, in *both* bucket sizes.  [0, 59_999] fails this by 59,998 ms.
    for bucket in [burst, sustained] {
        let [first, second] = SWEPT_PHASES_MS.map(|phase| first_bucket_boundary_ms(bucket, phase));
        assert!(
            first.abs_diff(second) >= bucket.get() - 1,
            "phases must differ by nearly a whole {bucket} ms bucket, got {first} vs {second}"
        );
    }
    assert_eq!(SWEPT_PHASES_MS, [full_bucket_away, immediate_boundary]);
}

#[tokio::test(start_paused = true)]
async fn m1_m13_run_against_the_actor_and_the_judge() {
    // Every row executes at both swept phases.  The mock's phase is always
    // recorded into the judge's G6 reproduction record; adding a third phase
    // is a data-only change here.
    for phase_ms in SWEPT_PHASES_MS {
        run_m1_m13(phase_ms, ContractCoverage::Fragment).await;
        run_m1_residue_sweep(phase_ms).await;
        run_m8_oauth_lane(phase_ms, ContractCoverage::Fragment).await;
        run_character_policy_lane(
            Endpoint::CharacterList,
            "character-list-request-limit",
            809,
            phase_ms,
            ContractCoverage::Fragment,
        )
        .await;
        run_character_policy_lane(
            Endpoint::Character,
            "character-request-limit",
            810,
            phase_ms,
            ContractCoverage::Fragment,
        )
        .await;
    }
}

/// Full-contract phase generation covers the entire common 60 s alignment
/// cycle and gives the exact before/on/after cases for both configured bucket
/// widths dedicated strategy weight. A failing case already carries its
/// `(seed, phi)` through every swept report; proptest adds shrinking and its
/// persisted generator seed.
fn full_contract_phase_strategy() -> impl Strategy<Value = u64> {
    prop_oneof![
        12 => 0..SUSTAINED_BUCKET_MS,
        1 => Just(BURST_BUCKET_MS - 1),
        1 => Just(BURST_BUCKET_MS),
        1 => Just(BURST_BUCKET_MS + 1),
        1 => Just(SUSTAINED_BUCKET_MS - 1),
        1 => Just(0),
        1 => Just(1),
    ]
}

fn run_full_contract_case(phase_ms: u64) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .expect("the full-contract paused runtime builds");
    runtime.block_on(async move {
        let mut reports = run_m1_m13(phase_ms, ContractCoverage::FullContract).await;
        reports.push(run_m8_oauth_lane(phase_ms, ContractCoverage::FullContract).await);
        reports.push(
            run_character_policy_lane(
                Endpoint::CharacterList,
                "character-list-request-limit",
                809,
                phase_ms,
                ContractCoverage::FullContract,
            )
            .await,
        );
        reports.push(
            run_character_policy_lane(
                Endpoint::Character,
                "character-request-limit",
                810,
                phase_ms,
                ContractCoverage::FullContract,
            )
            .await,
        );

        let declaration = FullContractRun::declare(reports.clone()).unwrap_or_else(|error| {
            panic!("the run-owned FullContract declaration failed: {error:?}; reports={reports:#?}")
        });
        assert_eq!(
            declaration.reports().len(),
            ScenarioId::ALL.len() + 3,
            "all M rows plus M8's second provenance lane plus the two \
             SD-R8-F5 character-policy lanes"
        );
        assert!(
            declaration
                .reports()
                .iter()
                .all(RunReport::verdict_eligible),
            "the declaration may contain no green fragment"
        );
    });
}

#[test]
fn full_contract_pinned_boundary_declares_full_contract() {
    run_full_contract_case(0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(FULL_CONTRACT_PROPERTY_CASES))]

    /// The run's first authority: each generated configuration produces a
    /// mechanically declared `FullContract` M1-M13 report set, including
    /// both the OAuth Known and shipped legacy Assumed lanes and the
    /// SD-R8-F5 character-policy lanes over every routed endpoint. The
    /// clause registry is intentionally not consulted here; it is the
    /// independent second authority used only after this run lands.
    #[test]
    #[ignore = "4,096-case full-contract generated-phase run; explicit review evidence"]
    fn full_contract_m1_m13_mock_judged_suite_declares_full_contract(
        phase_ms in full_contract_phase_strategy(),
    ) {
        run_full_contract_case(phase_ms);
    }
}

/// Independently restated N13 padded-safe eligibility, derived only from the
/// mock's B13 observation log and the scenario's policy definition. A hit at
/// `at` remains client-side debt through `at + period + bucket`; with `H` hits
/// permitted per window, hit `k` may go once hit `k - H` has aged out.
///
/// Valid only where every server-side hit is present in the harness facts.
/// The caller merges B13 phantom state changes before asking this arithmetic;
/// residue preloads are not in that log, so M1 must keep its dedicated oracle.
struct PolicyDebt {
    /// `(max_hits, period_ms, bucket_ms)` for every window of every rule.
    windows: Vec<(usize, u64, NonZeroU64)>,
}

/// Independent arithmetic only: this function deliberately cannot inspect
/// client state or call the production scheduling implementation.
fn independent_padded_safe_expiry_ms(hit_ms: u64, period_ms: u64, bucket_ms: NonZeroU64) -> u64 {
    hit_ms
        .saturating_add(period_ms)
        .saturating_add(bucket_ms.get())
}

#[test]
fn g3_oracle_pins_its_independent_padded_safe_arithmetic() {
    let burst_bucket = NonZeroU64::new(5_000).unwrap();
    let sustained_bucket = NonZeroU64::new(60_000).unwrap();

    assert_eq!(
        independent_padded_safe_expiry_ms(2_015, 15_000, burst_bucket),
        22_015
    );
    assert_eq!(
        independent_padded_safe_expiry_ms(2_015, 60_000, sustained_bucket),
        122_015
    );
}

impl PolicyDebt {
    async fn for_policy(controller: &MockController, policy: &str) -> Self {
        let definition = controller
            .definition(policy)
            .await
            .expect("the scenario's policy is server-configured");
        let windows = definition
            .rules()
            .iter()
            .flat_map(|rule| rule.windows())
            .map(|window| {
                (
                    usize::try_from(window.max_hits()).expect("u32 fits usize"),
                    window.period_ms(),
                    NonZeroU64::new(window.bucket_ms()).expect("a window's bucket is non-zero"),
                )
            })
            .collect();
        Self { windows }
    }

    /// `prior_arrival_ms` holds every earlier hit's server-recorded instant,
    /// oldest first.
    fn eligible_ms(&self, prior_arrival_ms: &[u64]) -> u64 {
        self.windows
            .iter()
            .map(|&(max_hits, period_ms, bucket_ms)| {
                let Some(expiring) = prior_arrival_ms
                    .len()
                    .checked_sub(max_hits)
                    .map(|index| prior_arrival_ms[index])
                else {
                    // Fewer hits than the window permits: no debt yet.
                    return 0;
                };
                independent_padded_safe_expiry_ms(expiring, period_ms, bucket_ms)
            })
            .max()
            .unwrap_or(0)
    }
}

fn independently_scripted_oracle(
    scenario_id: ScenarioId,
    observations: &[rate_limit_core::mock::Observation],
    state_changes: &[MockStateChange],
    debt: Option<&PolicyDebt>,
    submitted_ms: &[u64],
) -> DriverOracle {
    let mut oracle = DriverOracle {
        submitted_ms: submitted_ms.to_vec(),
        ..DriverOracle::default()
    };
    let mut prior_arrival_ms = Vec::new();
    let phantom_hits = state_changes
        .iter()
        .flat_map(|change| match &change.kind {
            MockStateChangeKind::PhantomInjection { count, .. } => {
                vec![change.occurred_ms; *count]
            }
            MockStateChangeKind::PolicyReplacement { .. }
            | MockStateChangeKind::PolicyRename { .. } => Vec::new(),
        })
        .collect::<Vec<_>>();
    let mut prior_dispatch = 0_u64;
    let mut prior_completion = 0_u64;
    let mut prior_was_head = false;
    let mut ordinary = 0usize;
    for observation in observations {
        let eligible = if observation.ordinal == 0 {
            // Every driver run queues its first unknown endpoint at t=0.
            0
        } else if observation.method == Method::HEAD {
            // A later writer becomes eligible only when the prior reader has
            // completed; the scenario's wire log supplies that server-owned
            // fact, while writer preference/exclusivity remain G5 assertions.
            prior_completion.max(prior_dispatch.saturating_add(MIN_SEND_SPACING_MS))
        } else {
            ordinary += 1;
            if scenario_id == ScenarioId::M8 && ordinary == 2 {
                // M8's scripted Retry-After=0 still demands the applicable
                // 60s bucket plus its one-second buffer (N19).
                prior_dispatch.saturating_add(APPLICABLE_BUCKET_AND_BUFFER_MS)
            } else {
                // D5's 250ms floor, plus the server's permit availability when
                // the scenario runs long enough to accrue policy debt.  Without
                // the debt term a saturating row reads every legitimate window
                // wait as a G3 violation.
                let floor_open = prior_dispatch.saturating_add(MIN_SEND_SPACING_MS);
                let exclusive_until = if prior_was_head { prior_completion } else { 0 };
                let permitted = debt.map_or(0, |debt| {
                    let mut server_hits = prior_arrival_ms.clone();
                    server_hits.extend(
                        phantom_hits
                            .iter()
                            .copied()
                            .filter(|occurred_ms| *occurred_ms <= observation.dispatch_ms),
                    );
                    server_hits.sort_unstable();
                    debt.eligible_ms(&server_hits)
                });
                exclusive_until.max(floor_open).max(permitted)
            }
        };
        // A request cannot be dispatched before the script asked for one.
        // Without this the oracle scores a caller that was submitted long
        // after it became policy-eligible as though the client sat on it --
        // the artifact behind doc finding 12b's spurious 500ms maxima.
        let requested = oracle
            .submitted_ms
            .iter()
            .copied()
            .filter(|submitted| *submitted <= observation.dispatch_ms)
            .max()
            .unwrap_or(0);
        oracle
            .eligible
            .insert(observation.correlation_id, eligible.max(requested));
        prior_dispatch = observation.dispatch_ms;
        prior_completion = observation.completion_ms;
        prior_was_head = observation.method == Method::HEAD;
        // Mirror the server's own counting rule, not the wire log: the mock
        // counts an arrival iff it is not a HEAD and did not trip layer 1.
        // Counting HEAD probes here overstates debt by one hit and reports
        // the boundary request as dispatched-before-eligible.
        if observation.method != Method::HEAD && !observation.layer1.tripped {
            prior_arrival_ms.push(observation.arrival_ms);
        }
    }
    oracle
}
