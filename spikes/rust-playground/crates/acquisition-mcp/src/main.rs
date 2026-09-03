//! `acq-mcp` — MCP server frontend (tracer bullet).
//!
//! A fourth thin client: it consumes exactly the two frontend surfaces —
//! the daemon protocol (via `acquisition_core::client`) and the shared
//! store's read API — and renders them as MCP tools over stdio.
//!
//! Two structural rules distinguish it from the CLI:
//!
//! - **It never kills or replaces a daemon** (`ConnectOptions::autonomous`).
//!   A version or provider mismatch may be a human's live GGG run; the MCP
//!   reports it and stops. It lazy-spawns only in mock mode, into an empty
//!   socket.
//! - **It never spawns a daemon in real-GGG mode.** A real-mode daemon is
//!   a human's act, via the CLI (it needs the keychain and the browser);
//!   the MCP talks to the one that is running or reports that none is.
//!   Agent traffic against GGG itself is allowed (owner ruling 2026-09-01,
//!   CONTEXT.md): humans, scripts and agents are all clients of the one
//!   daemon, and the daemon is the single gate that enforces GGG's rules.
//!
//! The refresh tracer's plan slice (step 8) is exposed as tools —
//! `sync_policy` / `set_sync_policy` (intent), `refresh_plan`
//! (derivation), `apply_plan` (effect) — through the same shared
//! semantics the CLI uses (`acquisition-plan`: validate-then-CAS policy
//! writes, the validating plan parse, the step-7 staleness gate).
//! Mode rules for the slice: intent reads/writes and offline plan
//! compilation send nothing and work in either mode (a policy write must
//! name the revision it replaces — an agent never clobbers intent it has
//! not read); `apply_plan` and `submit_job` spend through the running
//! daemon in either mode; quote enrichment asks the running daemon in
//! either mode (read-only, never spawning).
//!
//! # Decisions as recorded
//!
//! The rulings are `CONTEXT.md`'s registry (`C<n>`); what follows is each
//! entry's full text as recorded there, moved here on 2026-09-02 because
//! the mechanism it describes is this module's. The registry is current;
//! this is the mechanism as decided, kept beside the code that implements it.
//!
//! ## C13 — The MCP server is a fourth thin client (`acquisition-mcp`, binary `acq-mcp`, official `…
//!
//! **The MCP server is a fourth thin client (`acquisition-mcp`, binary `acq-mcp`, official `rmcp` SDK over stdio), never in-process with the daemon.** Same reasoning that moved reads to the store: daemon-hosted queries make the daemon an application server. The binary embeds `daemon run` like `acq` (lazy spawn execs `current_exe`). Two structural rules in the rail-6 mold: it never kills or replaces a daemon (autonomous connect policy above), and, while the agent-traffic deferral stood (2026-08-30 → 2026-09-01), it refused `submit_job` in real-GGG mode — store reads and observing a live daemon were always allowed, they send nothing. In real mode it still never spawns a daemon (a human's act: keychain, browser); it talks to the one that is running. It lazy-spawns only in mock mode; login stays human, via the CLI. The tracer is the consumer that validates the protocol: when it has proven the shape, the protocol gets pinned — the GUI arrives to a pinned boundary and proposes changes against it, rather than reopening the question. Decided 2026-08-30.
//!
//! ## C14 — Agent traffic against GGG is allowed; the daemon is the single gate.
//!
//! **Agent traffic against GGG is allowed; the daemon is the single gate.** Owner ruling 2026-09-01 on outside information: GGG permits agent use of the API as long as the API rules are respected. A CLI is already agent-drivable, and so increasingly is a desktop app, so the distinction between human, script and agent clients was never enforceable — what is enforceable is one gate that every client's traffic passes through, and that is the daemon (invariant 1). Consequences: the agent-traffic deferral is lifted; `acq-mcp` submits, applies and quotes in either mode against a running daemon; `quote` over MCP in real mode is simply allowed (it sends nothing). What stays: the MCP never spawns or replaces a daemon in real mode (login and the keychain are human), the live-test rails stay what `LIVE-TESTING.md` says, and every client — human or agent — is paced, journaled and halted by the same code. Decided 2026-09-01.

use std::path::PathBuf;

use acquisition_core::client::{Client, ConnectOptions, is_no_daemon};
use acquisition_core::protocol::{QuoteJob, Request, Response};
use acquisition_core::realm::Realm;
use acquisition_plan::{PlanError, RefreshPlan, plan_refresh, put_sync_policy};
use acquisition_store::{
    AccountEntry, Annotations, Index, SYNC_POLICY_KEY, SYNC_POLICY_KIND, SYNC_POLICY_SCOPE, Store,
    account_path, store_dir,
};
use anyhow::Result;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};
use serde::Deserialize;
use serde_json::{Value, json};

fn provider() -> &'static str {
    if acquisition_core::provider::ggg_mode() {
        "ggg"
    } else {
        "mock"
    }
}

/// anyhow errors become MCP tool errors with the full context chain.
fn err(e: anyhow::Error) -> ErrorData {
    ErrorData::internal_error(format!("{e:#}"), None)
}

/// Open one account's store file, resolved against the non-secret index —
/// no daemon involved (the store is the second frontend surface).
fn open_store(account: Option<&str>) -> Result<Store> {
    let (dir, entry) = resolve(account)?;
    Store::open(&account_path(&dir, &entry.username))
}

fn resolve(account: Option<&str>) -> Result<(PathBuf, AccountEntry)> {
    let dir = store_dir(provider());
    let index = Index::load(&dir)?;
    let entry = index.resolve(account).map_err(anyhow::Error::from)?.clone();
    Ok((dir, entry))
}

/// The selected account's provider directory, index entry, and annotations
/// file — the latter addressed by the uuid the index maps the account to
/// (an entry without one predates uuid-at-login; intent cannot be bound
/// to it). Same shape as the CLI's `open_intent`.
fn open_intent(account: Option<&str>) -> Result<(PathBuf, AccountEntry, Annotations)> {
    let (dir, entry) = resolve(account)?;
    let Some(uuid) = entry.uuid.as_deref() else {
        anyhow::bail!(
            "account {} has no recorded uuid (a login predating uuid-at-login); \
             a human can fix it with one `acq auth` login",
            entry.username
        );
    };
    let annotations = Annotations::open_for(&dir, uuid)?;
    Ok((dir, entry, annotations))
}

/// The quote attempt is bounded and best-effort, same as the CLI's: the
/// offline plan is the deliverable, and a wedged daemon must not keep it
/// from returning.
const QUOTE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Try to enrich the plan with a *running* daemon's read-only quote, in
/// either mode (owner ruling 2026-09-01: a quote sends nothing, and agent
/// use of the gate is allowed). The connection never spawns or replaces a
/// daemon — the plan promises to spend nothing, and a kill-and-respawn is
/// a spend (the successor resumes the persisted queue).
async fn try_quote(plan: RefreshPlan) -> (RefreshPlan, Option<String>) {
    let Some(account) = plan.account_name.clone() else {
        return (
            plan,
            Some("no quote: the facts record no account name to quote as".into()),
        );
    };
    let jobs: Vec<QuoteJob> = plan
        .actions
        .iter()
        .map(|action| {
            let (kind, params) = action.job();
            QuoteJob {
                kind: kind.into(),
                params,
            }
        })
        .collect();
    let attempt = async {
        let mut client = Client::connect(ConnectOptions::autonomous(false))
            .await
            .map_err(|e| {
                if is_no_daemon(&e) {
                    "no daemon running".to_string()
                } else {
                    format!("{e:#}")
                }
            })?;
        client
            .quote(jobs, Some(account))
            .await
            .map_err(|e| format!("{e:#}"))
    };
    match tokio::time::timeout(QUOTE_TIMEOUT, attempt).await {
        Err(_) => (
            plan,
            Some(format!(
                "no quote: the daemon did not answer within {QUOTE_TIMEOUT:?} — plan \
                 compiled offline"
            )),
        ),
        Ok(Err(why)) => (
            plan,
            Some(format!("no quote: {why} — plan compiled offline")),
        ),
        Ok(Ok(quote)) => match plan.clone().with_quote(quote) {
            Ok(enriched) => (enriched, None),
            Err(e) => (plan, Some(format!("daemon quote rejected: {e}"))),
        },
    }
}

/// Connect to the daemon under the autonomous policy: never kill or
/// replace; lazy-spawn only in mock mode (spawning a real-GGG daemon is
/// the human's act, via the CLI).
async fn connect(spawn: bool) -> Result<Client> {
    let spawn = spawn && !acquisition_core::provider::ggg_mode();
    Client::connect(ConnectOptions::autonomous(spawn)).await
}

/// A `realm` tool parameter: pc when omitted (as on the wire), else one
/// of the documented realms; anything else is a structured error.
fn realm_param(realm: Option<&str>) -> Result<Realm, ErrorData> {
    match realm {
        None => Ok(Realm::DEFAULT),
        Some(s) => Realm::parse(s).ok_or_else(|| {
            ErrorData::invalid_params(
                format!(
                    "unknown realm {s:?} (one of {})",
                    Realm::ALL.map(Realm::as_str).join(", ")
                ),
                None,
            )
        }),
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
struct TabsParams {
    /// League name; defaults to "Standard".
    league: Option<String>,
    /// Realm: pc (default), xbox, or sony.
    realm: Option<String>,
    /// Account selector (username with or without `#discriminator`, or
    /// uuid); required only when several accounts are known.
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct CharactersParams {
    /// Restrict to one league; omitted lists every league.
    league: Option<String>,
    /// Restrict to one realm (pc, xbox, sony, poe2); omitted lists every realm.
    realm: Option<String>,
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct SearchParams {
    /// Substring matched against item name, type line, and base type.
    text: String,
    /// Restrict to one league.
    league: Option<String>,
    /// Restrict to one realm (pc, xbox, sony, poe2).
    realm: Option<String>,
    /// Include items no longer seen at their last location.
    include_removed: Option<bool>,
    /// Maximum rows (default 50).
    limit: Option<usize>,
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct ItemParams {
    /// The GGG item id (from search results).
    id: String,
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct StoreParams {
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct EventsParams {
    /// How far back to look (default 24 hours).
    hours: Option<f64>,
    /// Maximum rows (default 200).
    limit: Option<usize>,
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct SubmitParams {
    /// Job kind: profile, characters, character, leagues, stashes, stash,
    /// refresh (mock daemon also: sleep, fetch, whoami).
    kind: String,
    /// Kind-specific params, e.g. {"league": "Standard", "id": "..."} for
    /// stash. Passed verbatim; visible to every connected client.
    params: Option<Value>,
    /// Account to run as; required when several sessions are live.
    account: Option<String>,
    /// Higher runs sooner (default 0).
    priority: Option<u8>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct JobParams {
    /// The job id, as returned by submit_job.
    id: u64,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct SetPolicyParams {
    /// The sync-policy value, e.g.
    /// {"version":3,"realms":{"pc":{"leagues":{"Standard":{"tabs":"all","characters":"all","max_age_seconds":3600}}}}}
    /// — per league, `tabs` and/or `characters` ("all" or an id list;
    /// absent means no coverage of that facet; an entry covering neither is
    /// refused). `tabs` is refused under realm poe2 (stashes are PoE1
    /// only); `characters` is taken under every realm. Older values still
    /// parse: a version-1 top-level `leagues` map as realm pc, and version
    /// 1/2 as tab coverage only.
    /// Validated strictly before anything lands — a typo'd field is
    /// refused, never half-honored.
    value: Value,
    /// The revision this write replaces, from sync_policy. Required when a
    /// policy exists (replacing intent you have not read is refused as a
    /// conflict naming the current revision); omit only to create the
    /// first policy.
    if_revision: Option<i64>,
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct PlanParams {
    /// League name; defaults to "Standard".
    league: Option<String>,
    /// Realm: pc (default), xbox, sony, or poe2 (a poe2 plan can carry
    /// character work only).
    realm: Option<String>,
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct ApplyParams {
    /// The plan envelope exactly as refresh_plan returned it (the `plan`
    /// field). Re-validated here; a tampered or hand-built envelope will
    /// not parse.
    plan: Value,
    /// Refuse at admission — before any child job exists — if the plan
    /// authorizes more logical requests than this.
    max_requests: Option<u64>,
    account: Option<String>,
}

struct AcqMcp;

#[tool_router]
impl AcqMcp {
    // ---- store reads: no daemon, no GGG traffic ----

    #[tool(
        description = "Accounts this machine has logged into (from the store's non-secret index; no daemon)."
    )]
    fn accounts(&self) -> Result<Json<Value>, ErrorData> {
        let dir = store_dir(provider());
        let index = Index::load(&dir).map_err(err)?;
        let rows: Vec<Value> = index
            .entries()
            .iter()
            .map(|e| {
                json!({
                    "username": e.username,
                    "last_login": e.last_login,
                    "persisted": e.persisted,
                    "uuid": e.uuid,
                })
            })
            .collect();
        Ok(Json(json!({ "provider": provider(), "accounts": rows })))
    }

    #[tool(
        description = "Stash tab tree of a league with item counts and fetch times, from the shared store (no daemon, no network)."
    )]
    fn tabs(&self, Parameters(p): Parameters<TabsParams>) -> Result<Json<Value>, ErrorData> {
        let store = open_store(p.account.as_deref()).map_err(err)?;
        let tabs = store
            .tabs(
                realm_param(p.realm.as_deref())?.as_str(),
                p.league.as_deref().unwrap_or("Standard"),
            )
            .map_err(err)?;
        serde_json::to_value(tabs)
            .map(Json)
            .map_err(|e| err(e.into()))
    }

    #[tool(
        description = "Characters known to the store, with class, level, league, item counts, and whether the full character (equipment + inventory) has been fetched (no daemon, no network). Fresh data is a `characters` or `character` job."
    )]
    fn characters(
        &self,
        Parameters(p): Parameters<CharactersParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let store = open_store(p.account.as_deref()).map_err(err)?;
        let realm = p
            .realm
            .as_deref()
            .map(|r| realm_param(Some(r)))
            .transpose()?;
        let rows = store
            .characters(realm.map(Realm::as_str), p.league.as_deref())
            .map_err(err)?;
        serde_json::to_value(rows)
            .map(Json)
            .map_err(|e| err(e.into()))
    }

    #[tool(
        description = "Substring search over item name/type/base in the shared store; socketed gems are rows too (no daemon, no network)."
    )]
    fn search_items(
        &self,
        Parameters(p): Parameters<SearchParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let store = open_store(p.account.as_deref()).map_err(err)?;
        let items = store
            .search(
                &p.text,
                p.realm
                    .as_deref()
                    .map(|r| realm_param(Some(r)))
                    .transpose()?
                    .map(Realm::as_str),
                p.league.as_deref(),
                p.include_removed.unwrap_or(false),
                p.limit.unwrap_or(50),
            )
            .map_err(err)?;
        serde_json::to_value(items)
            .map(Json)
            .map_err(|e| err(e.into()))
    }

    #[tool(description = "One item by id, verbatim as GGG returned it (no daemon, no network).")]
    fn get_item(&self, Parameters(p): Parameters<ItemParams>) -> Result<Json<Value>, ErrorData> {
        let store = open_store(p.account.as_deref()).map_err(err)?;
        match store.item(&p.id).map_err(err)? {
            Some(item) => serde_json::to_value(item)
                .map(Json)
                .map_err(|e| err(e.into())),
            None => Err(ErrorData::internal_error(format!("no item {}", p.id), None)),
        }
    }

    #[tool(description = "Store file path, size, and row counts (no daemon, no network).")]
    fn store_status(
        &self,
        Parameters(p): Parameters<StoreParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let store = open_store(p.account.as_deref()).map_err(err)?;
        let st = store.status().map_err(err)?;
        serde_json::to_value(st)
            .map(Json)
            .map_err(|e| err(e.into()))
    }

    #[tool(
        description = "Item events (added/moved/changed/removed) from recent ingests (no daemon, no network)."
    )]
    fn item_events(
        &self,
        Parameters(p): Parameters<EventsParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let store = open_store(p.account.as_deref()).map_err(err)?;
        let since = acquisition_store::now() - (p.hours.unwrap_or(24.0) * 3600.0) as i64;
        let ev = store
            .events_since(since, p.limit.unwrap_or(200))
            .map_err(err)?;
        serde_json::to_value(ev)
            .map(Json)
            .map_err(|e| err(e.into()))
    }

    // ---- the intent/plan slice (tracer step 8): policy → plan → apply ----

    #[tool(
        description = "The per-account sync policy (declared coverage + freshness, an annotation) with its revision — the intent refresh_plan compiles. Reads the store; no daemon, no network. `policy` is null when none is set."
    )]
    fn sync_policy(
        &self,
        Parameters(p): Parameters<StoreParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let (_, _, annotations) = open_intent(p.account.as_deref()).map_err(err)?;
        let row = annotations
            .get(SYNC_POLICY_SCOPE, SYNC_POLICY_KEY, SYNC_POLICY_KIND)
            .map_err(|e| err(e.into()))?;
        Ok(Json(json!({ "policy": row })))
    }

    #[tool(
        description = "Write the per-account sync policy (validated strictly, compare-and-swap on if_revision). Sends nothing — intent is local; the network cost appears only when a plan is applied. Replacing an existing policy requires if_revision (from sync_policy); a stale or missing revision is a conflict, never a clobber."
    )]
    fn set_sync_policy(
        &self,
        Parameters(p): Parameters<SetPolicyParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let (_, _, mut annotations) = open_intent(p.account.as_deref()).map_err(err)?;
        let row = put_sync_policy(&mut annotations, &p.value, p.if_revision)
            .map_err(|e| err(e.into()))?;
        serde_json::to_value(row)
            .map(Json)
            .map_err(|e| err(e.into()))
    }

    #[tool(
        description = "Compile the stored sync policy + facts on record into a RefreshPlan for one (realm, league): the explicit, bounded action set applying would execute — stash listing and tab fetches, the realm's character listing and character fetches, each facet as the policy covers it — with per-action reasons, skipped tabs and characters, unknown ids, and a coarse wire estimate. Offline — sends nothing, spends nothing. A running daemon adds its read-only quote (ETA + rate-limit headroom; never spawned for this); `quote_note` says why one is absent. Review the plan, then hand `plan` to apply_plan."
    )]
    async fn refresh_plan(
        &self,
        Parameters(p): Parameters<PlanParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let (dir, entry, annotations) = open_intent(p.account.as_deref()).map_err(err)?;
        let store = Store::open(&account_path(&dir, &entry.username)).map_err(err)?;
        let snapshot = store
            .refresh_snapshot(
                realm_param(p.realm.as_deref())?.as_str(),
                p.league.as_deref().unwrap_or("Standard"),
                &annotations,
            )
            .map_err(err)?;
        let plan = match plan_refresh(provider(), &snapshot, acquisition_store::now()) {
            Err(PlanError::NoSyncPolicy) => {
                return Err(ErrorData::internal_error(
                    format!(
                        "no sync policy is set for {} — declare one with set_sync_policy first",
                        entry.username
                    ),
                    None,
                ));
            }
            other => other.map_err(|e| err(e.into()))?,
        };
        let (plan, quote_note) = try_quote(plan).await;
        let mut out = json!({ "plan": plan });
        if let Some(note) = quote_note {
            out["quote_note"] = json!(note);
        }
        Ok(Json(out))
    }

    #[tool(
        description = "Execute a reviewed plan: exactly its actions, as one `apply` parent job the daemon admits or refuses whole at submit (single-request vocabulary + the max_requests budget). Refused offline — before any daemon contact — if the stored sync-policy revision moved since the plan (replan with refresh_plan). In real-GGG mode the daemon must already be running (a human starts it; this server never spawns one there). An empty plan is a no-op with no daemon contact. Returns the parent job id; poll job_status, then job_result."
    )]
    async fn apply_plan(
        &self,
        Parameters(p): Parameters<ApplyParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let (_, entry, annotations) = open_intent(p.account.as_deref()).map_err(err)?;
        let plan = RefreshPlan::from_value(&p.plan).map_err(|e| err(e.into()))?;
        plan.check_spendable(provider(), entry.uuid.as_deref(), &annotations)
            .map_err(|e| err(e.into()))?;
        if plan.actions.is_empty() {
            // A strict subset of zero actions is satisfied by doing
            // nothing; the plan's own skipped/unknown reporting says why
            // it is empty.
            return Ok(Json(json!({
                "applied": false,
                "requests": 0,
                "note": "nothing to do: the plan authorizes no requests",
            })));
        }
        let mut client = connect(true).await.map_err(err)?;
        let resp = client
            .request(&Request::Submit {
                kind: "apply".into(),
                params: plan.apply_params(p.max_requests),
                priority: 0,
                submitted_by: format!("mcp:{}", std::process::id()),
                account: Some(entry.username),
            })
            .await
            .map_err(err)?;
        match resp {
            Response::Submitted { id } => Ok(Json(json!({
                "job_id": id,
                "requests": plan.logical_requests,
            }))),
            Response::Error { message } => Err(ErrorData::internal_error(message, None)),
            other => Err(ErrorData::internal_error(
                format!("unexpected response: {other:?}"),
                None,
            )),
        }
    }

    // ---- daemon jobs ----

    #[tool(
        description = "Submit a job to the daemon and return its id immediately; poll job_status then fetch job_result. In real-GGG mode the daemon must already be running (a human starts it; this server never spawns one there)."
    )]
    async fn submit_job(
        &self,
        Parameters(p): Parameters<SubmitParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let mut client = connect(true).await.map_err(err)?;
        let resp = client
            .request(&Request::Submit {
                kind: p.kind,
                params: p.params.unwrap_or_else(|| json!({})),
                priority: p.priority.unwrap_or(0),
                submitted_by: format!("mcp:{}", std::process::id()),
                account: p.account,
            })
            .await
            .map_err(err)?;
        match resp {
            Response::Submitted { id } => Ok(Json(json!({ "job_id": id }))),
            Response::Error { message } => Err(ErrorData::internal_error(message, None)),
            other => Err(ErrorData::internal_error(
                format!("unexpected response: {other:?}"),
                None,
            )),
        }
    }

    #[tool(description = "Jobs the daemon knows about this lifetime, with states and ETAs.")]
    async fn list_jobs(&self) -> Result<Json<Value>, ErrorData> {
        let mut client = connect(false).await.map_err(err)?;
        match client.request(&Request::List).await.map_err(err)? {
            Response::Jobs { jobs } => serde_json::to_value(jobs)
                .map(Json)
                .map_err(|e| err(e.into())),
            other => Err(ErrorData::internal_error(
                format!("unexpected response: {other:?}"),
                None,
            )),
        }
    }

    #[tool(
        description = "One job's state, ETA, and retries. A waiting job with a large ETA is the rate limiter holding, not a hang (holds can reach ~5 minutes)."
    )]
    async fn job_status(
        &self,
        Parameters(p): Parameters<JobParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let mut client = connect(false).await.map_err(err)?;
        let job = client.status(p.id).await.map_err(err)?;
        serde_json::to_value(job)
            .map(Json)
            .map_err(|e| err(e.into()))
    }

    #[tool(
        description = "A finished job's payload or error; answered across daemon restarts (the queue persists). Payloads can be large — prefer the store tools for reading data."
    )]
    async fn job_result(
        &self,
        Parameters(p): Parameters<JobParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let mut client = connect(false).await.map_err(err)?;
        match client
            .request(&Request::Result { id: p.id })
            .await
            .map_err(err)?
        {
            Response::Result { outcome, .. } => serde_json::to_value(outcome)
                .map(Json)
                .map_err(|e| err(e.into())),
            Response::Error { message } => Err(ErrorData::internal_error(message, None)),
            other => Err(ErrorData::internal_error(
                format!("unexpected response: {other:?}"),
                None,
            )),
        }
    }

    #[tool(description = "Cancel a waiting or running job; cascades to its descendants.")]
    async fn cancel_job(
        &self,
        Parameters(p): Parameters<JobParams>,
    ) -> Result<Json<Value>, ErrorData> {
        let mut client = connect(false).await.map_err(err)?;
        client
            .expect_ack(&Request::Cancel { id: p.id })
            .await
            .map_err(err)?;
        Ok(Json(json!({ "job_id": p.id, "cancel_requested": true })))
    }

    #[tool(
        description = "Daemon vitals: provider, uptime, queue depths, rate-limit policies learned, rails state. Reports running=false if no daemon is up."
    )]
    async fn daemon_status(&self) -> Result<Json<Value>, ErrorData> {
        let mut client = match connect(false).await {
            Ok(c) => c,
            // A failed socket connect is "no daemon". Anything else — a
            // version/provider mismatch this client refuses to resolve —
            // must surface, not read as "not running".
            Err(e) if e.downcast_ref::<std::io::Error>().is_some() => {
                return Ok(Json(json!({ "running": false })));
            }
            Err(e) => return Err(err(e)),
        };
        let resp = client.request(&Request::DaemonStatus).await.map_err(err)?;
        serde_json::to_value(resp)
            .map(Json)
            .map_err(|e| err(e.into()))
    }
}

#[tool_handler]
impl ServerHandler for AcqMcp {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some(format!(
            "Acquisition playground (provider: {}). Store tools (accounts, tabs, \
             search_items, get_item, store_status, item_events) read the shared SQLite \
             store directly — no daemon, no network. The refresh slice: sync_policy / \
             set_sync_policy declare intent (local, sends nothing; replacing a policy \
             must name the revision it replaces), refresh_plan compiles it offline into \
             the explicit bounded action set for review, and apply_plan spends exactly \
             that plan as one parent job. Job tools talk to the local daemon; API \
             requests are jobs (submit_job returns an id; poll job_status, then \
             job_result). Rate-limit holds can reach ~5 minutes — a waiting job is the \
             limiter working. Login is done by the human via the `acq` CLI; this server \
             never replaces a running daemon and refuses submissions in real-GGG mode.",
            provider()
        ));
        info
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Lazy spawn execs `<current_exe> daemon run` (client.rs), so this
    // binary carries the daemon too, like `acq`.
    if args.iter().map(String::as_str).eq(["daemon", "run"]) {
        return acquisition_core::daemon::run().await;
    }
    let service = AcqMcp.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
