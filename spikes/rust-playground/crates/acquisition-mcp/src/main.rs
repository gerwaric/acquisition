//! `acq-mcp` — MCP server frontend (tracer bullet).
//!
//! A fourth thin client: it consumes exactly the two frontend surfaces —
//! the daemon protocol (via `acquisition_client`) and the shared
//! store's read API — and renders them as MCP tools over stdio.
//!
//! Two structural rules distinguish it from the CLI:
//!
//! - **It never kills or replaces a daemon** (`ConnectOptions::autonomous`).
//!   A contract, artifact or provider mismatch (C84) may be a human's live
//!   GGG run; the MCP reports it and stops. It lazy-spawns only in mock
//!   mode, into an empty socket.
//! - **It never spawns a daemon in real-GGG mode.** A real-mode daemon is
//!   a human's act, via the CLI (it needs the keychain and the browser);
//!   the MCP talks to the one that is running or reports that none is.
//!   Agent traffic against GGG itself is allowed (C14, the owner's ruling
//!   of 2026-09-01): humans, scripts and agents are all clients of the one
//!   daemon, and the daemon is the single gate that enforces GGG's rules.
//!
//! The refresh tracer's plan slice (step 8) is exposed as tools —
//! `sync_policy` / `set_sync_policy` (intent), `refresh_plan`
//! (derivation), `apply_plan` (effect) — through the same shared
//! semantics the CLI uses (C46; `acquisition-plan`: validate-then-CAS policy
//! writes, the validating plan parse, the step-7 staleness gate).
//! Mode rules for the slice: intent reads/writes and offline plan
//! compilation send nothing and work in either mode (a policy write must
//! name the revision it replaces — an agent never clobbers intent it has
//! not read); `apply_plan` and `submit_job` spend through the running
//! daemon in either mode; quote enrichment asks the running daemon in
//! either mode (read-only, never spawning).
//!
//! # As built: the item search (the build plan, step 11b)
//!
//! `search`, `describe` and `show` are adapters of `acquisition-search`,
//! as `acq search` and `acq show` are (C46, C89): what a tool returns is
//! the document the CLI's `--json` prints, and nothing here evaluates.
//! The rulings are `search/LEDGER.md`, "11 · the opening":
//!
//! - **H1 — the tool takes the request itself.** The arguments of
//!   `search` are handed whole to the one reader of a request
//!   (`Asked::read`, what `--request` reads): a request, or what an
//!   answer holds one in — a count with its route, a `next`, an offer —
//!   sent back as printed. No token: a token is a terminal's encoding
//!   of the request a `next` already holds.
//! - **H2 — `scope.account` selects the store**, where a terminal's
//!   `--account` does; every request an answer holds names it.
//! - **H3 — an omitted realm is the terminal's rule** (C96): resolved
//!   over a store of one realm, refused over several.
//! - **H4 — the answer's JSON whole, no text.** The result's structured
//!   content is the answer, and its one content block is that same
//!   document serialized, for a host that reads no structured content.
//! - **H5 — an offer that is a command is a request too**: a refusal's
//!   `offers`, made in the search crate (`error::Offered`).
//! - **A refusal is a result marked `isError`** holding the CLI's
//!   failure document — `error`, `kind`, `readings`, `offers` — never a
//!   JSON-RPC error, whose `data` a host need not show the model (the
//!   owner, 2026-10-10). The server's other tools fail as they did.
//!
//! Three refusals are this adapter's own and no document of the CLI's,
//! a terminal's words for them being no agent's: no account's store
//! (`no_accounts`, naming the directory looked in), an account not
//! named or not known (`account_needed`, `account_unknown`, the known
//! names as readings), and arguments a tool does not take
//! (`arguments`).
//!
//! Pinned by `tests/search_tools.rs`: AQ1–AQ5 through the tools over a
//! fixture, each answer and each of the search crate's refusals held
//! equal to the CLI's document.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/frontends.md` for this
//! area, `CONTEXT.md` for the cross-cutting ones (`C<n>`); what follows is each
//! entry's full text as recorded there, moved here on 2026-09-02 because
//! the mechanism it describes is this module's. The registry is current;
//! this is the mechanism as decided, kept beside the code that implements it.
//!
//! ## C13 — The MCP server is a fourth thin client (`acquisition-mcp`, binary `acq-mcp`, official `…
//!
//! **The MCP server is a fourth thin client (`acquisition-mcp`, binary `acq-mcp`, official `rmcp` SDK over stdio), never in-process with the daemon.** Same reasoning that moved reads to the store: daemon-hosted queries make the daemon an application server. Two structural rules in the rail-6 mold: it never kills or replaces a daemon (autonomous connect policy above), and, while the agent-traffic deferral stood (2026-08-30 → 2026-09-01), it refused `submit_job` in real-GGG mode — store reads and observing a live daemon were always allowed, they send nothing. In real mode it still never spawns a daemon (a human's act: keychain, browser); it talks to the one that is running. It lazy-spawns only in mock mode; login stays human, via the CLI. The tracer is the consumer that validates the protocol: when it has proven the shape, the protocol gets pinned — the GUI arrives to a pinned boundary and proposes changes against it, rather than reopening the question. Decided 2026-08-30.
//!
//! ## C14 — Agent traffic against GGG is allowed; the daemon is the single gate.
//!
//! **Agent traffic against GGG is allowed; the daemon is the single gate.** Owner ruling 2026-09-01 on outside information: GGG permits agent use of the API as long as the API rules are respected. A CLI is already agent-drivable, and so increasingly is a desktop app, so the distinction between human, script and agent clients was never enforceable — what is enforceable is one gate that every client's traffic passes through, and that is the daemon (invariant 1). Consequences: the agent-traffic deferral is lifted; `acq-mcp` submits, applies and quotes in either mode against a running daemon; `quote` over MCP in real mode is simply allowed (it sends nothing). What stays: the MCP never spawns or replaces a daemon in real mode (login and the keychain are human), the live-test rails stay what `LIVE-TESTING.md` says, and every client — human or agent — is paced, journaled and halted by the same code. Decided 2026-09-01.

use std::path::PathBuf;

use acquisition_client::client::{
    Client, ConnectError, ConnectOptions, DaemonError, NotSpawned, Observed,
};
use acquisition_plan::{PlanError, RefreshPlan, plan_refresh, put_sync_policy};
use acquisition_protocol::protocol::{ErrorKind, QuoteJob, Request, Response};
use acquisition_protocol::realm::Realm;
use acquisition_search::SearchError;
use acquisition_search::answer::{Asked, QueryInput};
use acquisition_store::{
    AccountEntry, Annotations, Index, Provenance, Resolve, SYNC_POLICY_KEY, SYNC_POLICY_KIND,
    SYNC_POLICY_SCOPE, Store, account_path, store_dir,
};
use anyhow::Result;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{CallToolResult, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData, ServerHandler, ServiceExt, schemars, tool, tool_handler, tool_router};
use serde::Deserialize;
use serde_json::{Value, json};

/// The provider this process wants (`ACQ_PROVIDER`, C88): the store directory, the
/// plan's provider field and the daemon's `hello` all name the same one.
fn provider() -> &'static str {
    acquisition_protocol::provider::wanted()
}

/// anyhow errors become MCP tool errors with the full context chain.
/// An error for the MCP client. A daemon refusal anywhere in the chain
/// becomes [`daemon_error`], so the closed kind reaches the agent; a
/// door that did not open ([`ConnectError`]) becomes [`connect_error`].
fn err(e: anyhow::Error) -> ErrorData {
    if let Some(refusal) = DaemonError::find(&e) {
        return daemon_error(refusal.kind, format!("{e:#}"));
    }
    if let Some(door) = e.chain().find_map(|c| c.downcast_ref::<ConnectError>()) {
        return connect_error(door, format!("{e:#}"));
    }
    ErrorData::internal_error(format!("{e:#}"), None)
}

/// A door that did not open, as a JSON-RPC error: the server could not
/// serve the call (`internal_error`), and `data.connect` names which
/// door failed — `absent`, `incompatible`, `other_world`,
/// `spawn_failed`, `unresponsive`, `transport` — so an agent can branch
/// on it the way it branches on a daemon error's kind.
fn connect_error(door: &ConnectError, message: String) -> ErrorData {
    let connect = match door {
        ConnectError::Absent { .. } => "absent",
        ConnectError::Incompatible { .. } => "incompatible",
        ConnectError::OtherWorld { .. } => "other_world",
        ConnectError::SpawnFailed { .. } => "spawn_failed",
        ConnectError::Unresponsive(_) => "unresponsive",
        ConnectError::Transport(_) => "transport",
    };
    ErrorData::internal_error(message, Some(json!({ "connect": connect })))
}

/// A daemon refusal as a JSON-RPC error (C85): the code follows the kind
/// — the request was wrong (`invalid_request`), its parameters were
/// (`invalid_params`), it named nothing (`resource_not_found`), or the
/// daemon could not serve it (`internal_error`) — and the kind itself
/// rides in `data` so an agent can branch on the closed set.
fn daemon_error(kind: ErrorKind, message: String) -> ErrorData {
    let data = Some(json!({ "kind": kind }));
    match kind {
        ErrorKind::BadRequest => ErrorData::invalid_request(message, data),
        ErrorKind::Refused | ErrorKind::AmbiguousAccount | ErrorKind::WrongState => {
            ErrorData::invalid_params(message, data)
        }
        ErrorKind::UnknownJob => ErrorData::resource_not_found(message, data),
        ErrorKind::NotLoggedIn
        | ErrorKind::QueueFailed
        | ErrorKind::Upstream
        | ErrorKind::Internal => ErrorData::internal_error(message, data),
    }
}

/// Open one account's store file, resolved against the non-secret index —
/// no daemon involved (the store is the second frontend surface).
fn open_store(account: Option<&str>) -> Result<Store> {
    let (dir, entry) = resolve(account)?;
    Store::open(&account_path(&dir, &entry.username))
}

/// No account is known: the error names the directory looked in, and
/// no login for the caller to run (the agent's seat, F10: a server
/// pointed at the wrong root answered every call with `acq auth`, the
/// one act an agent cannot perform).
#[derive(Debug)]
struct NoAccounts(PathBuf);

impl std::fmt::Display for NoAccounts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "no account's store in {} (provider {}; ACQ_STORE_DIR sets the root this server \
             looks under): a store is made by a login, which is a human's, from a terminal",
            self.0.display(),
            provider()
        )
    }
}

impl std::error::Error for NoAccounts {}

fn resolve(account: Option<&str>) -> Result<(PathBuf, AccountEntry)> {
    let dir = store_dir(provider());
    let index = Index::load(&dir)?;
    match index.resolve(account) {
        Ok(entry) => Ok((dir, entry.clone())),
        Err(Resolve::Empty) => Err(NoAccounts(dir).into()),
        Err(e) => Err(e.into()),
    }
}

// ---- the item search: adapters of `acquisition-search` ----

/// What a tool of the search returns: the document the CLI's `--json`
/// prints — an answer, or a failure in a result marked an error.
type Told = Result<CallToolResult, ErrorData>;

fn told<T: serde::Serialize>(document: &T) -> Told {
    serde_json::to_value(document)
        .map(CallToolResult::structured)
        .map_err(|e| err(e.into()))
}

/// A refusal, as the search crate words it (C11): `error`, `kind`,
/// `readings`, `offers`.
fn refused(e: impl Into<SearchError>) -> CallToolResult {
    CallToolResult::structured_error(e.into().to_json())
}

/// The selected account's facts and its intent file, as `acq search`
/// opens them: the search reads both and writes neither. `named` is the
/// argument that selects an account, for the refusal that needs one.
fn open_search(
    account: Option<&str>,
    named: &str,
) -> std::result::Result<(Store, Annotations), CallToolResult> {
    let known = || {
        Index::load(&store_dir(provider()))
            .map(|index| {
                index
                    .entries()
                    .iter()
                    .map(|e| e.username.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };
    let opened = open_intent(account).and_then(|(dir, entry, intent)| {
        Ok((Store::open(&account_path(&dir, &entry.username))?, intent))
    });
    opened.map_err(|e| {
        if e.downcast_ref::<NoAccounts>().is_some() {
            return CallToolResult::structured_error(
                json!({ "error": format!("{e:#}"), "kind": "no_accounts" }),
            );
        }
        CallToolResult::structured_error(match e.downcast_ref::<Resolve>() {
            Some(Resolve::NotFound(selector)) => json!({
                "error": format!(
                    "no account matches `{selector}`: `{named}` takes a username, with or \
                     without its #discriminator, or a uuid"
                ),
                "kind": "account_unknown",
                "readings": known(),
            }),
            Some(Resolve::Ambiguous(names)) => json!({
                "error": format!(
                    "{} accounts are known and `{named}` selects one",
                    names.len()
                ),
                "kind": "account_needed",
                "readings": names,
            }),
            _ => json!({ "error": format!("{e:#}"), "kind": "store" }),
        })
    })
}

/// A tool's arguments, read here and not by the SDK, whose refusal of
/// arguments it cannot read is a line of text and no document: what
/// the tool does not take is refused as any refusal is, and a key it
/// does not know is never read past. The schema is `T`'s.
struct Args<T>(serde_json::Map<String, Value>, std::marker::PhantomData<T>);

impl<'de, T> Deserialize<'de> for Args<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Deserialize::deserialize(deserializer).map(|map| Args(map, std::marker::PhantomData))
    }
}

impl<T: schemars::JsonSchema> schemars::JsonSchema for Args<T> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        T::schema_name()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        T::json_schema(generator)
    }
}

impl<T: serde::de::DeserializeOwned> Args<T> {
    fn read(self) -> std::result::Result<T, CallToolResult> {
        serde_json::from_value(Value::Object(self.0)).map_err(|e| {
            CallToolResult::structured_error(json!({
                "error": format!("these are not arguments the tool takes: {e}"),
                "kind": "arguments",
            }))
        })
    }
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

/// The channel every intent write from this binary is stamped with (C65).
const WRITTEN_VIA: &str = "mcp";

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
        let mut client = match Client::observe().await {
            Ok(Observed::Compatible(client)) => client,
            Ok(Observed::Absent) => return Err("no daemon running".to_string()),
            Ok(Observed::Incompatible(found)) => return Err(found.to_string()),
            Ok(Observed::Unresponsive(found)) => return Err(found.to_string()),
            Err(e) => return Err(format!("{e:#}")),
        };
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

/// A spending tool's connect, under the autonomous policy: never kill or
/// replace; lazy-spawn only in mock mode (spawning a real-GGG daemon is
/// the human's act, via the CLI).
async fn connect(spawn: bool) -> Result<Client> {
    let real = acquisition_protocol::provider::ggg_mode();
    let spawn = spawn && !real;
    match Client::connect(ConnectOptions::autonomous(spawn)).await {
        Ok(client) => Ok(client),
        // Absent by this server's own policy: the remedy is the human's,
        // and the message says so rather than promising a spawn.
        Err(
            e @ ConnectError::Absent {
                because: NotSpawned::Policy,
            },
        ) if real => Err(anyhow::Error::from(e).context(
            "no daemon running in real-GGG mode, and this server never starts one there (C13) — \
             start it from the CLI with a job command (`acq profile`)",
        )),
        Err(e) => Err(e.into()),
    }
}

/// The running daemon, for a tool that observes it or acts on it (C10):
/// never spawns or replaces; absence and a mismatch are errors that say
/// which.
async fn attach() -> Result<Client> {
    client_of(Client::observe().await?)
}

/// The queue on disk, for a tool with no daemon to ask (C45): this
/// server's world and provider, `None` when neither has been created.
fn persisted_queue() -> Result<Option<acquisition_store::jobs::Persisted>> {
    acquisition_store::jobs::Persisted::observe(acquisition_protocol::provider::wanted())
}

/// The client an observation yields, or the error that says why not.
fn client_of(observed: Observed) -> Result<Client> {
    match observed {
        Observed::Compatible(client) => Ok(client),
        Observed::Absent => anyhow::bail!("no daemon running"),
        Observed::Unresponsive(found) => anyhow::bail!("{found}"),
        Observed::Incompatible(found) => anyhow::bail!(
            "{found}; this server never replaces a daemon — resolve it with the CLI (`acq daemon stop`{})",
            if found.world_matches() {
                ""
            } else {
                ", or point ACQ_STORE_DIR at its world"
            }
        ),
    }
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

/// The arguments of `search`: the request, read by the search crate's
/// own reader and by nothing here (H1). The schema says what a request
/// holds; the reader refuses what is none.
#[derive(Deserialize)]
#[serde(transparent)]
struct RequestParams(serde_json::Map<String, Value>);

impl schemars::JsonSchema for RequestParams {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SearchRequest".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "scope": {
                    "type": "object",
                    "description": "What is searched: {\"account\": a username or uuid, needed only when several accounts are known; \"realm\": pc, xbox, sony, poe2 or all, needed only when the store holds several; \"membership\": \"live\" (the default); \"all\", which adds removed items, each row marked with when it left; or \"removed\", the removed items alone: what has left the stash and the characters}. Under all and removed the answer's scope counts the live items as `items` and the removed as `removed`, whichever were searched."
                },
                "query": {
                    "type": "object",
                    "description": "{\"text\": a query in the language `describe` prints} or {\"tree\": the tree an answer returned}, never both. Omitted: every item in scope. A refinement is the old text in parentheses and a new term."
                },
                "view": {
                    "type": "object",
                    "description": "One of: {\"rows\": {\"limit\": N, \"sort\": a value, \"desc\": true}}, the default, 20 rows where no limit is given; {\"counts\": {\"keys\": [a key, …], \"sum\": a value, \"limit\": N}}, a table for each key and no rows; {\"cross\": {\"keys\": [two keys]}}, one crossed table; {\"explain\": {\"path\": a node's path, a term's as `terms[].path` gives it}}, the total with that node forced true and forced false. `describe` with names [\"counts\"] lists the keys. A sort's value is a field (reqlevel), a computed value (pseudo.total_res) or a line's number (line(\"# to maximum Life\").arg1); an item that lacks it is last, ascending or descending."
                },
                "context": {
                    "type": "string",
                    "description": "\"matches\" (the default), or \"corpus\": the view over every item in scope, the query set aside."
                },
                "basis": {
                    "type": "object",
                    "description": "Carried by a continuation, as an answer's `next` holds it; never written by hand."
                },
                "request": {
                    "type": "object",
                    "description": "In place of the fields above: a request as an answer holds one — a count (`count`, `denominator`, `request`), `view.rows.next`, or an offer — sent whole, as printed."
                },
                "count": {
                    "type": "integer",
                    "description": "Beside `request`, the count it was printed with: the answer then says under `before` where the count has moved."
                },
                "counted_at": {
                    "type": "object",
                    "description": "Beside `request` and `count`, the `basis` of the answer the count came from: the answer then says under `before` wherever the basis has moved."
                }
            }
        })
    }
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct DescribeParams {
    /// Only these entries: a section — `fields`, `line`, `linked`,
    /// `values`, `composition`, `operators`, `slots`, `computed`,
    /// `counts`, `explain` — or a name within one: `league`,
    /// `total_res`. Omitted: the whole language.
    names: Option<Vec<String>>,
}

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct ShowParams {
    /// An item's id, whole, as an answer printed it.
    id: String,
    /// Also return the body as the store holds it, verbatim.
    body: Option<bool>,
    /// Why the item does or does not match a query: `{"text": …}` or
    /// `{"tree": …}`, as a request's `query`.
    against: Option<Value>,
    /// Account selector (username with or without `#discriminator`, or
    /// uuid); required only when several accounts are known.
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
        description = "Every stash tab of a league, a flat list in which a substash names its `parent`, with item counts and fetch times, from the shared store (no daemon, no network)."
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
        description = "Item search over the shared store (no daemon, no network): a query in the language `describe` prints, asked of every item in scope. The arguments are the request: {} is every item; {\"query\": {\"text\": \"class:ring rarity=rare\"}} is a query. The answer is one JSON document: `query` (the text as understood, and its tree), `scope` (account, realm, what was and was not fetched), `basis` (what the answer was computed from), `terms` (for each term how many items matched, failed, lacked the thing or were undecided), `total`, `view`, and `rows` — id, name, base, rarity, place by name and id, and the lines each term touched, never the item's body, which is `show`. Rows past the limit are counted in `view.rows.left_out`, and `view.rows.next` is the request for the rows after: send it back whole. Every count that is not zero holds the request that returns exactly its members, and a zero total offers requests under `zero.offers`: any of them is sent back as printed. A count-first look is {\"view\": {\"counts\": {\"keys\": [\"tab\", \"rarity\"]}}}; the vocabulary of lines is the key `line`, or `line:<text>` narrowed. A key is one entry of `keys`: a terminal's list `line:resist,life` is [\"line:resist\", \"line:life\"], and a comma inside an entry is part of its text. A time is seconds since 1970, UTC. `describe` and the messages name a terminal's flags: --count, --cross, --sum, --sort, --desc, --limit and --explain are `view`'s fields, --context is `context`, --realm and --membership are `scope`'s, and --next is `view.rows.next` sent back. A refusal is a result marked isError: `error`, `kind`, `readings` (what may have been meant, as a terminal would type it: a query text, a name, a flag or a whole `acq` command) and, where a way on is a search, `offers` (the same as requests to send instead)."
    )]
    fn search(&self, Parameters(RequestParams(args)): Parameters<RequestParams>) -> Told {
        let asked = match Asked::read(&Value::Object(args)) {
            Ok(asked) => asked,
            Err(e) => return Ok(refused(e)),
        };
        // an authoring error is said before the store is read
        if let Err(e) = asked.request.check() {
            return Ok(refused(e));
        }
        let (store, intent) =
            match open_search(asked.request.scope.account.as_deref(), "scope.account") {
                Ok(opened) => opened,
                Err(refusal) => return Ok(refusal),
            };
        match asked
            .request
            .corpus(&store, &intent)
            .and_then(|corpus| asked.answer(&corpus))
        {
            Ok(answer) => told(&answer),
            Err(e) => Ok(refused(e)),
        }
    }

    #[tool(
        description = "The query language `search` takes, as this build knows it (no store read): fields, what a line has, operators, closed value sets, slots, the computed values with their definitions, what a count takes, what is not built and the limits stated — or only the entries named. The whole is over 100 KB, most of it under `computed`: name what is needed instead. The sections are fields (some 10 KB), line, linked, values, composition, operators, slots, counts and explain (each under 4 KB), and computed; a single name — league, total_res — returns its own entry. A name it does not know is refused, and the refusal lists every name it knows."
    )]
    fn describe(&self, Parameters(p): Parameters<Args<DescribeParams>>) -> Told {
        let p = match p.read() {
            Ok(p) => p,
            Err(refusal) => return Ok(refusal),
        };
        match acquisition_search::describe(&p.names.unwrap_or_default()) {
            Ok(described) => told(&described),
            Err(e) => Ok(refused(e)),
        }
    }

    #[tool(
        description = "One item as the search derives it, live or removed (no daemon, no network): its place, its fields, class, category and price, and every line with its kind, template and numbers. `against` adds why the item does or does not match a query: the root's outcome and each term's. `body` adds the stored body, verbatim. An id that is a tab's or a character's is refused and offers the search that lists it. A refusal is a result marked isError, as `search`'s is."
    )]
    fn show(&self, Parameters(p): Parameters<Args<ShowParams>>) -> Told {
        let p = match p.read() {
            Ok(p) => p,
            Err(refusal) => return Ok(refusal),
        };
        let against = match p.against.map(serde_json::from_value::<QueryInput>) {
            None => None,
            Some(Ok(query)) => Some(query),
            Some(Err(e)) => {
                return Ok(CallToolResult::structured_error(json!({
                    "error": format!(
                        "`against` is a query as a request holds one, {{\"text\": …}} or {{\"tree\": …}}: {e}"
                    ),
                    "kind": "tree",
                })));
            }
        };
        // an authoring error is said before the store is read
        if let Some(Err(e)) = against.as_ref().map(QueryInput::bind) {
            return Ok(refused(e));
        }
        let (store, intent) = match open_search(p.account.as_deref(), "account") {
            Ok(opened) => opened,
            Err(refusal) => return Ok(refusal),
        };
        let body = p.body.unwrap_or(false);
        match match &against {
            Some(query) => acquisition_search::show::against(&store, &intent, &p.id, body, query),
            None => acquisition_search::show(&store, &intent, &p.id, body),
        } {
            Ok(shown) => told(&shown),
            Err(e) => Ok(refused(e)),
        }
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
        let row = put_sync_policy(
            &mut annotations,
            &p.value,
            p.if_revision,
            &Provenance::via(WRITTEN_VIA),
        )
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
            Response::Error { kind, message } => Err(daemon_error(kind, message)),
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
            Response::Error { kind, message } => Err(daemon_error(kind, message)),
            other => Err(ErrorData::internal_error(
                format!("unexpected response: {other:?}"),
                None,
            )),
        }
    }

    #[tool(
        description = "Jobs the daemon knows about this lifetime, with states and ETAs. Observes only (C10): with no daemon running it never spawns one and answers from the ledger on disk instead (C45) — running: false, persisted: the non-terminal rows as the last daemon left them (jobs, without ETAs) and written_at, how old that picture is; persisted is null when this shell's world has no queue yet. A job command from the CLI spawns a successor that restores them."
    )]
    async fn list_jobs(&self) -> Result<Json<Value>, ErrorData> {
        let mut client = match Client::observe().await.map_err(err)? {
            Observed::Absent => {
                let persisted = persisted_queue().map_err(err)?;
                return Ok(Json(json!({ "running": false, "persisted": persisted })));
            }
            other => client_of(other).map_err(err)?,
        };
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
        let mut client = attach().await.map_err(err)?;
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
        let mut client = attach().await.map_err(err)?;
        match client
            .request(&Request::Result { id: p.id })
            .await
            .map_err(err)?
        {
            Response::Result { outcome, .. } => serde_json::to_value(outcome)
                .map(Json)
                .map_err(|e| err(e.into())),
            Response::Error { kind, message } => Err(daemon_error(kind, message)),
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
        let mut client = attach().await.map_err(err)?;
        client
            .expect_ack(&Request::Cancel { id: p.id })
            .await
            .map_err(err)?;
        Ok(Json(json!({ "job_id": p.id, "cancel_requested": true })))
    }

    #[tool(
        description = "The daemon running: its identity (C84, C83) — contract revision, the executable it runs from and its hash (artifact), the world it serves (its canonical store root), the socket it was reached on (derived from the world), how that file relates to the acqd beside this server (artifact_relation: same_file, same_bytes for another copy, different, unhashable, no_sibling, unreported), which of the four dimensions match (contract_matches, artifact_matches, provider_matches, world_matches), and under wanted this server's own version, contract, provider and world with that sibling — all from the one look that judged the daemon; and, when compatible, its vitals: provider, uptime, connections, queue counts, rate-limit policies learned, rails state (with the journal path), keyring health, and log — the log file the daemon opened, as it reports it. Observes only: running=false when no daemon is up, with persisted — what the ledger on disk holds (C45): waiting, recorded_running, written_at; null when this shell's world has no queue yet; running=true, compatible=false for a daemon of another contract, artifact, provider or world, reported by its identity alone and never replaced by this server; running=true, responsive=false (with socket and lock_holder, the pid the world's lock records) for something that accepted the connection and did not answer hello within 5s — a wedged daemon, for a human to stop by hand."
    )]
    async fn daemon_status(&self) -> Result<Json<Value>, ErrorData> {
        let mut client = match Client::observe().await.map_err(err)? {
            Observed::Compatible(c) => c,
            Observed::Absent => {
                let persisted = persisted_queue().map_err(err)?.map(|p| {
                    json!({
                        "waiting": p.waiting(),
                        "recorded_running": p.recorded_running(),
                        "written_at": p.written_at,
                    })
                });
                return Ok(Json(json!({ "running": false, "persisted": persisted })));
            }
            Observed::Incompatible(found) => {
                let mut report = found.report();
                report["running"] = json!(true);
                report["compatible"] = json!(false);
                return Ok(Json(report));
            }
            Observed::Unresponsive(found) => {
                return Ok(Json(json!({
                    "running": true,
                    "responsive": false,
                    "socket": found.at.socket,
                    "lock_holder": found.holder,
                })));
            }
        };
        let found = client.daemon().clone();
        let resp = client.request(&Request::DaemonStatus).await.map_err(err)?;
        let mut report = serde_json::to_value(resp).map_err(|e| err(e.into()))?;
        report["running"] = json!(true);
        report["compatible"] = json!(true);
        // The identity keys the incompatible report has (C84): the
        // daemon's contract and artifact, the artifact's relation to this
        // server's sibling, and the sibling under `wanted`.
        let identity = found.report();
        for key in [
            "contract",
            "artifact",
            "world",
            "socket",
            "contract_matches",
            "artifact_matches",
            "artifact_relation",
            "provider_matches",
            "world_matches",
            "wanted",
        ] {
            report[key] = identity[key].clone();
        }
        Ok(Json(report))
    }
}

#[tool_handler]
impl ServerHandler for AcqMcp {
    fn get_info(&self) -> ServerInfo {
        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.instructions = Some(format!(
            "Acquisition playground (provider: {}). Items: search answers a query over \
             every item the store holds — counts for each term, the total, then rows or \
             tables — describe prints its query language, and show is one item whole. \
             Their arguments and answers are JSON; a term's or a table's count that is \
             not zero holds the request that returns its members, sent back as printed, \
             and a refusal is a result marked isError with readings and offers to go on \
             from. Begin with search with no arguments, for what the store holds — over a \
             store of several realms that is refused, and the refusal's offers are the \
             request in each realm: send one back — and describe with names \
             [\"fields\"], for what a query may ask: describe whole is over 100 KB. No \
             tool says what an item is worth: a price is the owner's own listing. The \
             store tools (search, show, accounts, tabs, characters, \
             search_items, get_item, store_status, item_events) read the shared SQLite \
             store directly — no daemon, no network. \
             The refresh slice: sync_policy / \
             set_sync_policy declare intent (local, sends nothing; replacing a policy \
             must name the revision it replaces), refresh_plan compiles it offline into \
             the explicit bounded action set for review, and apply_plan spends exactly \
             that plan as one parent job. Job tools talk to the local daemon; API \
             requests are jobs (submit_job returns an id; poll job_status, then \
             job_result). Rate-limit holds can reach ~5 minutes — a waiting job is the \
             limiter working. Login is done by the human via the `acq` CLI; this server \
             never spawns or replaces a daemon in real-GGG mode — there it submits, applies \
             and quotes only through one a human already started (C14).",
            provider()
        ));
        info
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // C88: refused before serving — the host shows the line.
    if let Err(e) = acquisition_protocol::provider::check_environment() {
        eprintln!("acq-mcp: {e}");
        std::process::exit(2);
    }
    let service = AcqMcp.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
