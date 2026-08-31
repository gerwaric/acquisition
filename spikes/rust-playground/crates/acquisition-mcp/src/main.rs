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
//! - **It refuses to submit jobs in real-GGG mode.** Agent traffic against
//!   GGG is deferred until GGG's policy stance on it is verified
//!   (CONTEXT.md, "Explicitly deferred"). Store reads and observing a live
//!   daemon are allowed in either mode — they send nothing.

use acquisition_core::client::{Client, ConnectOptions};
use acquisition_core::protocol::{Request, Response};
use acquisition_store::{Index, Store, account_path, store_dir};
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
    let dir = store_dir(provider());
    let index = Index::load(&dir)?;
    let entry = index.resolve(account).map_err(anyhow::Error::from)?.clone();
    Store::open(&account_path(&dir, &entry.username))
}

/// Connect to the daemon under the autonomous policy: never kill or
/// replace; lazy-spawn only in mock mode (spawning a real-GGG daemon is
/// the human's act, via the CLI).
async fn connect(spawn: bool) -> Result<Client> {
    let spawn = spawn && !acquisition_core::provider::ggg_mode();
    Client::connect(ConnectOptions::autonomous(spawn)).await
}

#[derive(Deserialize, schemars::JsonSchema)]
struct TabsParams {
    /// League name; defaults to "Standard".
    league: Option<String>,
    /// Account selector (username with or without `#discriminator`, or
    /// uuid); required only when several accounts are known.
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct CharactersParams {
    /// Restrict to one league; omitted lists every league.
    league: Option<String>,
    account: Option<String>,
}

#[derive(Deserialize, schemars::JsonSchema)]
struct SearchParams {
    /// Substring matched against item name, type line, and base type.
    text: String,
    /// Restrict to one league.
    league: Option<String>,
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
            .tabs(p.league.as_deref().unwrap_or("Standard"))
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
        let rows = store.characters(p.league.as_deref()).map_err(err)?;
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

    // ---- daemon jobs ----

    #[tool(
        description = "Submit a job to the daemon and return its id immediately; poll job_status then fetch job_result. Refused in real-GGG mode (agent traffic against GGG is deferred)."
    )]
    async fn submit_job(
        &self,
        Parameters(p): Parameters<SubmitParams>,
    ) -> Result<Json<Value>, ErrorData> {
        if acquisition_core::provider::ggg_mode() {
            return Err(ErrorData::internal_error(
                "refused: agent traffic against real GGG is deferred until GGG's policy stance \
                 is verified (CONTEXT.md). Unset ACQ_GGG to work against the mock provider.",
                None,
            ));
        }
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
             store directly — no daemon, no network. Job tools talk to the local daemon; \
             API requests are jobs (submit_job returns an id; poll job_status, then \
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
