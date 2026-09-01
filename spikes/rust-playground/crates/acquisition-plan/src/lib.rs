//! The planner: policy compilation and Plan construction (CONTEXT.md,
//! decided 2026-08-31). This crate turns a neutral store snapshot
//! ([`acquisition_store::StashSnapshot`] — facts and intent named together,
//! nothing derived) plus the sync policy into a [`RefreshPlan`]: a
//! serializable, immutable authorization envelope, computable with the
//! daemon down.
//!
//! Boundaries this crate holds:
//! - It is linked by **frontends only**, never the daemon — "the daemon
//!   never reads the store" stays enforced by the dependency graph.
//! - Plans are **binding**: applying one executes exactly the listed
//!   actions or a strict subset, never an unreviewed addition. v1 plans
//!   therefore act only on facts already on record — a league that was
//!   never listed plans the listing alone, and newly discovered tabs wait
//!   for the next plan (honest eventual reconciliation). Dynamic `--deep`
//!   fan-out is excluded: substashes get actions only once their stubs are
//!   in the store.
//! - `metadata.items` counts are heuristic evidence: they can prove a tab
//!   changed (a disagreeing count on a listing newer than our fetch forces
//!   a fetch), never that it didn't (an agreeing count never skips one —
//!   only the policy's own freshness window does that).
//! - Work has two dimensions: `logical_requests` is exact for a refresh;
//!   `wire_sends` is a coarse range plus named prerequisites, never a
//!   precise accounting (the wire-budget feature is deferred).

// The lint ratchet (CONTEXT.md, "Panics are for broken internal invariants
// only"): the planner's production code panics on nothing external — a
// user-authored policy value or a store row is a structured error.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use acquisition_core::daemon::MAX_429_RETRIES;
use acquisition_core::protocol::Quote;
use acquisition_store::{ListingBasis, StashSnapshot, TabSnapshot};

/// The plan envelope schema this build writes ([`RefreshPlan::plan_schema`]).
/// A consumer handed a plan stamped newer refuses it rather than guessing.
pub const REFRESH_PLAN_SCHEMA: i64 = 1;

/// The sync-policy value schema this build reads ([`SyncPolicy::version`]).
/// The store carries the row opaquely; its shape is this crate's business.
pub const SYNC_POLICY_VERSION: i64 = 1;

/// A planner failure with a stable kind (CONTEXT.md: malformed external
/// input is a structured error, never a panic and never a bare string).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// The snapshot carries no sync-policy row: there is no declared
    /// intent to compile. Distinct from "covered and fresh" (a valid plan
    /// with zero actions).
    NoSyncPolicy,
    /// The sync-policy value did not parse as this build's schema. The
    /// policy is user-authored intent, so an unknown field is malformed
    /// too — a typo must not be silently ignored into a different policy.
    MalformedPolicy { detail: String },
    /// The policy declares a version this build does not read. Refused,
    /// never guessed at — same rule as the store's schema stamps.
    PolicyVersionUnsupported { found: i64, supported: i64 },
    /// The policy declares nothing for the snapshot's league; there is no
    /// authorized work to derive.
    LeagueNotCovered { league: String },
    /// A serialized plan failed validation: unknown fields, a wrong
    /// operation, a league its envelope does not name, or derived counts
    /// that do not recompute. A plan that will not parse is a plan apply
    /// must not trust.
    MalformedPlan { detail: String },
    /// A serialized plan carries a schema stamp this build does not
    /// read. Refused, never guessed at.
    PlanSchemaUnsupported { found: i64, supported: i64 },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanError::NoSyncPolicy => {
                write!(f, "no sync policy is set for this account")
            }
            PlanError::MalformedPolicy { detail } => {
                write!(f, "malformed sync policy: {detail}")
            }
            PlanError::PolicyVersionUnsupported { found, supported } => write!(
                f,
                "sync policy declares version {found}, newer than this build's v{supported}"
            ),
            PlanError::LeagueNotCovered { league } => {
                write!(f, "the sync policy does not cover league {league}")
            }
            PlanError::MalformedPlan { detail } => {
                write!(f, "malformed refresh plan: {detail}")
            }
            PlanError::PlanSchemaUnsupported { found, supported } => write!(
                f,
                "refresh plan declares schema {found}, not this build's v{supported}"
            ),
        }
    }
}

impl std::error::Error for PlanError {}

/// The per-account sync policy: an inspectable declaration of desired
/// coverage and freshness — not a scheduler. Stored as the
/// `("account", "", "sync-policy")` annotation; written by frontends,
/// compiled here into minimal requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SyncPolicyWire")]
pub struct SyncPolicy {
    /// [`SYNC_POLICY_VERSION`]; a different stamp is refused at parse.
    pub version: i64,
    /// Coverage per league. A league not named here compiles to
    /// [`PlanError::LeagueNotCovered`], never to implicit work.
    pub leagues: BTreeMap<String, LeaguePolicy>,
}

/// The raw JSON shape. Every deserialization path funnels through the
/// `TryFrom` below, so the version gate and the unknown-field refusal
/// cannot be bypassed by deserializing around [`SyncPolicy::from_value`].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyncPolicyWire {
    version: i64,
    leagues: BTreeMap<String, LeaguePolicy>,
}

impl TryFrom<SyncPolicyWire> for SyncPolicy {
    type Error = String;
    fn try_from(wire: SyncPolicyWire) -> Result<Self, String> {
        if wire.version != SYNC_POLICY_VERSION {
            return Err(format!(
                "sync policy declares version {}, not this build's v{SYNC_POLICY_VERSION}",
                wire.version
            ));
        }
        Ok(SyncPolicy {
            version: wire.version,
            leagues: wire.leagues,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaguePolicy {
    pub tabs: TabSelection,
    /// The freshness declaration: facts older than this want refreshing.
    /// Applies to the listing and to each selected tab's fetch alike.
    pub max_age_seconds: u32,
}

/// Which tabs the policy covers: `"all"` or an explicit id list (substash
/// ids included — their identity is their own GGG id; the `parent/id`
/// display convention is a frontend matter).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TabSelectionRepr", into = "TabSelectionRepr")]
pub enum TabSelection {
    All,
    Ids(Vec<String>),
}

impl TabSelection {
    fn selects(&self, id: &str) -> bool {
        match self {
            TabSelection::All => true,
            TabSelection::Ids(ids) => ids.iter().any(|i| i == id),
        }
    }
}

/// The JSON shape: the string `"all"` or an array of ids.
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum TabSelectionRepr {
    Word(String),
    Ids(Vec<String>),
}

impl TryFrom<TabSelectionRepr> for TabSelection {
    type Error = String;
    fn try_from(repr: TabSelectionRepr) -> Result<Self, String> {
        match repr {
            TabSelectionRepr::Word(w) if w == "all" => Ok(TabSelection::All),
            TabSelectionRepr::Word(w) => Err(format!(
                "tabs must be \"all\" or a list of tab ids, not \"{w}\""
            )),
            TabSelectionRepr::Ids(ids) => Ok(TabSelection::Ids(ids)),
        }
    }
}

impl From<TabSelection> for TabSelectionRepr {
    fn from(sel: TabSelection) -> TabSelectionRepr {
        match sel {
            TabSelection::All => TabSelectionRepr::Word("all".into()),
            TabSelection::Ids(ids) => TabSelectionRepr::Ids(ids),
        }
    }
}

impl SyncPolicy {
    /// Parse a stored sync-policy value. The version stamp is checked
    /// before the full shape so a genuinely newer policy reports
    /// [`PlanError::PolicyVersionUnsupported`], not a spurious
    /// unknown-field complaint.
    pub fn from_value(value: &Value) -> Result<SyncPolicy, PlanError> {
        let found = value
            .get("version")
            .and_then(Value::as_i64)
            .ok_or_else(|| PlanError::MalformedPolicy {
                detail: "missing integer `version`".into(),
            })?;
        if found != SYNC_POLICY_VERSION {
            return Err(PlanError::PolicyVersionUnsupported {
                found,
                supported: SYNC_POLICY_VERSION,
            });
        }
        serde_json::from_value(value.clone()).map_err(|e| PlanError::MalformedPolicy {
            detail: e.to_string(),
        })
    }
}

/// Why the plan re-lists the league.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ListingReason {
    NeverListed,
    Stale { age_seconds: i64 },
}

/// Why the plan fetches a tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FetchReason {
    NeverFetched,
    Stale {
        age_seconds: i64,
    },
    /// The heuristic arm: a listing newer than our fetch reported a
    /// different item count, which proves the tab changed (`listed` is the
    /// listing's count, `held` what the store holds from the last fetch).
    ListedCountDisagrees {
        listed: i64,
        held: i64,
    },
}

/// Why a covered tab is not in the action set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SkipReason {
    /// Fetched within the policy's window and nothing proved a change.
    Fresh { age_seconds: i64 },
    /// Folders hold tabs, never items, and are never fetched — their
    /// children are covered individually.
    Folder,
    /// The league has never been listed, so the plan is the listing
    /// alone: membership is unconfirmed, and every fetch waits for the
    /// facts the listing lands (D5a's eventual reconciliation).
    AwaitingListing,
    /// The tab's recorded parent is no longer on record (the store keeps
    /// substash rows when a listing retires their parent), so the fetch
    /// path under it cannot be rendered with confidence. The next listing
    /// or parent re-fetch reconciles it.
    OrphanedParent { parent: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkippedTab {
    pub id: String,
    pub name: String,
    pub reason: SkipReason,
}

/// One authorized request. Self-contained on purpose: an action can be
/// rendered, reviewed, or turned into a daemon job without the envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum RefreshAction {
    ListStashes {
        league: String,
        reason: ListingReason,
    },
    FetchTab {
        league: String,
        id: String,
        name: String,
        tab_type: String,
        reason: FetchReason,
    },
    /// A map/unique substash, fetched under its parent tab. Its stub is
    /// already on record — v1 plans never fan out dynamically.
    FetchSubstash {
        league: String,
        parent: String,
        id: String,
        name: String,
        tab_type: String,
        reason: FetchReason,
    },
}

impl RefreshAction {
    /// The league this action touches — validated against the envelope's
    /// on deserialization.
    pub fn league(&self) -> &str {
        match self {
            RefreshAction::ListStashes { league, .. }
            | RefreshAction::FetchTab { league, .. }
            | RefreshAction::FetchSubstash { league, .. } => league,
        }
    }

    /// The action in the daemon's job vocabulary: `(kind, params)` exactly
    /// as `Submit` wants them. `deep` is always false — a plan's actions
    /// are the reviewed set, and a fetch that fanned out would expand it.
    pub fn job(&self) -> (&'static str, Value) {
        match self {
            RefreshAction::ListStashes { league, .. } => ("stashes", json!({ "league": league })),
            RefreshAction::FetchTab { league, id, .. } => (
                "stash",
                json!({ "league": league, "id": id, "deep": false }),
            ),
            RefreshAction::FetchSubstash {
                league, parent, id, ..
            } => (
                "stash",
                json!({ "league": league, "id": parent, "sub": id, "deep": false }),
            ),
        }
    }
}

/// What the plan was derived from — enough for apply-time checks (the
/// snapshot carries the revision so the comparison is possible) and for a
/// human to see how old the inputs were.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanBasis {
    /// When the snapshot was taken (facts as of this read).
    pub snapshot_taken_at: i64,
    /// The listing the tab set derives from; `None` when the league was
    /// never listed (which is itself the plan's only action).
    pub listing: Option<ListingBasis>,
    /// The sync-policy annotation revision compiled from. Plans always
    /// derive from stored intent — there is no ad-hoc path — so apply can
    /// always check this against the current row (the step-7 staleness
    /// ruling needs the comparison to be possible).
    pub policy_revision: i64,
}

/// The coarse wire projection. The range covers the authorized requests
/// and their bounded 429 retries; `prerequisites` names sends the range
/// deliberately does not count — a precise accounting is the deferred
/// wire-budget feature, and pretending to one here would be false. The
/// whole of it, prerequisites included, is part of the reviewed
/// projection: deserialization recomputes it ([`wire_estimate`]) and
/// refuses a mismatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireEstimate {
    pub min: u64,
    pub max: u64,
    pub prerequisites: Vec<String>,
}

/// The one place the wire projection is computed — the compiler builds
/// it and the deserialization validator compares against it, so the two
/// cannot drift. Any change here (the retry bound, the prerequisite
/// wording) changes what serialized schema-1 plans claim, so it is a
/// plan-schema event, not a silent edit.
fn wire_estimate(logical_requests: u64) -> WireEstimate {
    WireEstimate {
        min: logical_requests,
        max: logical_requests * (1 + u64::from(MAX_429_RETRIES)),
        prerequisites: if logical_requests == 0 {
            Vec::new()
        } else {
            vec![
                "a HEAD probe on first contact with each route this daemon lifetime (N16)".into(),
                "an OAuth token refresh if the access token has expired (N33)".into(),
            ]
        },
    }
}

/// A refresh authorization: the bounded work the user reviewed, derived
/// from one snapshot of facts + intent. Immutable by convention — new
/// facts produce a new plan, never an edit — and serializable so it can
/// cross a socket, land in a journal, or be read back by apply.
///
/// Deserialization validates: every path funnels through
/// `TryFrom<RefreshPlanWire>`, which refuses unknown fields, a schema
/// stamp other than [`REFRESH_PLAN_SCHEMA`], an operation other than
/// `"refresh"`, an action whose league is not the envelope's, and
/// derived counts that do not recompute — so a tampered or hand-built
/// envelope will not parse into this type. [`RefreshPlan::from_value`]
/// is the friendly entry (a newer schema reports as such).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RefreshPlanWire")]
pub struct RefreshPlan {
    /// [`REFRESH_PLAN_SCHEMA`].
    pub plan_schema: i64,
    /// Always `"refresh"`; the envelope names its operation so a reader
    /// of the serialized form needs no out-of-band context.
    pub operation: String,
    /// Bound by the caller from the provider directory it opened — the
    /// snapshot deliberately does not carry it.
    pub provider: String,
    pub account_uuid: String,
    pub account_name: Option<String>,
    pub league: String,
    pub generated_at: i64,
    pub basis: PlanBasis,
    /// The freshness assumption applied: facts younger than this (with no
    /// count disagreement) were left alone.
    pub max_age_seconds: u32,
    /// The explicit action set. Applying executes exactly this or a
    /// strict subset.
    pub actions: Vec<RefreshAction>,
    /// Covered tabs left out, each with why — the plan shows its
    /// reasoning, not just its conclusions.
    pub skipped: Vec<SkippedTab>,
    /// Ids the policy names that the facts on record do not: vanished
    /// tabs (or typos). Reported, never invented into actions.
    pub unknown_tabs: Vec<String>,
    /// Exact for a refresh: one logical request per action.
    pub logical_requests: u64,
    pub wire_sends: WireEstimate,
    /// Optional enrichment: the daemon's quote for this plan's actions,
    /// with its own observation time inside. An observation, not a
    /// derivation — it cannot be recomputed at parse, only carried — so
    /// validation checks it speaks about this plan's provider and account
    /// and nothing more. Compiling never fills it (a plan needs no
    /// daemon); [`RefreshPlan::with_quote`] attaches one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quote>,
}

/// The one consistency a carried quote must have with its envelope: it
/// projects this plan's provider and account, not some other store's. A
/// mismatched quote would mislead exactly the review the plan exists for.
fn check_quote_matches(
    quote: &Quote,
    provider: &str,
    account_name: Option<&str>,
) -> Result<(), String> {
    if quote.provider != provider {
        return Err(format!(
            "the quote projects provider {:?}, but the plan is for {provider:?}",
            quote.provider
        ));
    }
    if let (Some(qa), Some(pa)) = (quote.account.as_deref(), account_name)
        && qa != pa
    {
        return Err(format!(
            "the quote projects account {qa:?}, but the plan is for {pa:?}"
        ));
    }
    Ok(())
}

impl RefreshPlan {
    /// Enrich the plan with the daemon's quote (decided 2026-08-31: a plan
    /// optionally carries one, with its own observation time). Consuming
    /// on purpose — the enriched plan is a new value, not an edit of a
    /// reviewed one.
    pub fn with_quote(mut self, quote: Quote) -> Result<RefreshPlan, PlanError> {
        check_quote_matches(&quote, &self.provider, self.account_name.as_deref())
            .map_err(|detail| PlanError::MalformedPlan { detail })?;
        self.quote = Some(quote);
        Ok(self)
    }
}

impl RefreshPlan {
    /// Parse a serialized plan back into a trusted envelope. The schema
    /// stamp is checked before the shape so a genuinely newer plan
    /// reports [`PlanError::PlanSchemaUnsupported`], not a spurious
    /// unknown-field complaint; everything else is the `TryFrom`
    /// validation below.
    pub fn from_value(value: &Value) -> Result<RefreshPlan, PlanError> {
        let found = value
            .get("plan_schema")
            .and_then(Value::as_i64)
            .ok_or_else(|| PlanError::MalformedPlan {
                detail: "missing integer `plan_schema`".into(),
            })?;
        if found != REFRESH_PLAN_SCHEMA {
            return Err(PlanError::PlanSchemaUnsupported {
                found,
                supported: REFRESH_PLAN_SCHEMA,
            });
        }
        serde_json::from_value(value.clone()).map_err(|e| PlanError::MalformedPlan {
            detail: e.to_string(),
        })
    }
}

/// The raw JSON shape of a plan; deserialization validates through the
/// `TryFrom` below on every path.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RefreshPlanWire {
    plan_schema: i64,
    operation: String,
    provider: String,
    account_uuid: String,
    account_name: Option<String>,
    league: String,
    generated_at: i64,
    basis: PlanBasis,
    max_age_seconds: u32,
    actions: Vec<RefreshAction>,
    skipped: Vec<SkippedTab>,
    unknown_tabs: Vec<String>,
    logical_requests: u64,
    wire_sends: WireEstimate,
    #[serde(default)]
    quote: Option<Quote>,
}

impl TryFrom<RefreshPlanWire> for RefreshPlan {
    type Error = String;
    fn try_from(wire: RefreshPlanWire) -> Result<Self, String> {
        if wire.plan_schema != REFRESH_PLAN_SCHEMA {
            return Err(format!(
                "refresh plan declares schema {}, not this build's v{REFRESH_PLAN_SCHEMA}",
                wire.plan_schema
            ));
        }
        if wire.operation != "refresh" {
            return Err(format!(
                "a RefreshPlan's operation must be \"refresh\", not {:?}",
                wire.operation
            ));
        }
        if let Some(stray) = wire.actions.iter().find(|a| a.league() != wire.league) {
            return Err(format!(
                "action for league {:?} inside a plan for league {:?}",
                stray.league(),
                wire.league
            ));
        }
        // The derived quantities are recomputed, never taken on faith:
        // admission-time budgeting (D8) trusts `logical_requests`, and a
        // forged range or prerequisite list would mislead the review the
        // plan exists for. `wire_estimate` is the same function the
        // compiler used, so nothing here can drift from it.
        let logical = wire.actions.len() as u64;
        if wire.logical_requests != logical {
            return Err(format!(
                "logical_requests says {} but the plan lists {logical} actions",
                wire.logical_requests
            ));
        }
        let expected = wire_estimate(logical);
        if wire.wire_sends != expected {
            return Err(format!(
                "wire_sends does not recompute from {logical} actions \
                 (got {}..{} with {} prerequisites, expected {}..{} with {})",
                wire.wire_sends.min,
                wire.wire_sends.max,
                wire.wire_sends.prerequisites.len(),
                expected.min,
                expected.max,
                expected.prerequisites.len()
            ));
        }
        // A carried quote is an observation and cannot recompute, but it
        // must at least be *about* this plan.
        if let Some(quote) = &wire.quote {
            check_quote_matches(quote, &wire.provider, wire.account_name.as_deref())?;
        }
        Ok(RefreshPlan {
            plan_schema: wire.plan_schema,
            operation: wire.operation,
            provider: wire.provider,
            account_uuid: wire.account_uuid,
            account_name: wire.account_name,
            league: wire.league,
            generated_at: wire.generated_at,
            basis: wire.basis,
            max_age_seconds: wire.max_age_seconds,
            actions: wire.actions,
            skipped: wire.skipped,
            unknown_tabs: wire.unknown_tabs,
            logical_requests: wire.logical_requests,
            wire_sends: wire.wire_sends,
            quote: wire.quote,
        })
    }
}

/// Compile the snapshot's own sync-policy row into a plan. The one way
/// in: plans always derive from stored intent and carry its revision —
/// an ad-hoc selection that bypasses the policy waits for a consumer
/// that demonstrably needs it.
pub fn plan_refresh(
    provider: &str,
    snapshot: &StashSnapshot,
    now: i64,
) -> Result<RefreshPlan, PlanError> {
    let row = snapshot.policy.as_ref().ok_or(PlanError::NoSyncPolicy)?;
    let policy = SyncPolicy::from_value(&row.value)?;
    compile(provider, snapshot, &policy, row.revision, now)
}

fn compile(
    provider: &str,
    snapshot: &StashSnapshot,
    policy: &SyncPolicy,
    policy_revision: i64,
    now: i64,
) -> Result<RefreshPlan, PlanError> {
    let league_policy =
        policy
            .leagues
            .get(&snapshot.league)
            .ok_or_else(|| PlanError::LeagueNotCovered {
                league: snapshot.league.clone(),
            })?;
    let max_age = i64::from(league_policy.max_age_seconds);
    let mut actions = Vec::new();
    // A league with no listing plans the listing **alone** — even tabs on
    // record from direct fetches wait: without a listing basis the plan
    // has no membership authority, so fetches defer to the next plan
    // (D5a's eventual reconciliation). Ages saturate: a corrupt store
    // timestamp must misread as "very stale", never wrap into "fresh"
    // (and never panic — the no-panic-on-store-rows rule).
    let listing_alone = match &snapshot.listing {
        None => {
            actions.push(RefreshAction::ListStashes {
                league: snapshot.league.clone(),
                reason: ListingReason::NeverListed,
            });
            true
        }
        Some(basis) => {
            let age = now.saturating_sub(basis.fetched_at).max(0);
            if age > max_age {
                actions.push(RefreshAction::ListStashes {
                    league: snapshot.league.clone(),
                    reason: ListingReason::Stale { age_seconds: age },
                });
            }
            false
        }
    };
    let mut skipped = Vec::new();
    for tab in &snapshot.tabs {
        if !league_policy.tabs.selects(&tab.id) {
            continue;
        }
        let verdict = if listing_alone {
            Err(SkipReason::AwaitingListing)
        } else if tab.r#type == "Folder" {
            Err(SkipReason::Folder)
        } else {
            fetch_verdict(tab, max_age, now).and_then(|reason| fetch_action(snapshot, tab, reason))
        };
        match verdict {
            Ok(action) => actions.push(action),
            Err(reason) => skipped.push(SkippedTab {
                id: tab.id.clone(),
                name: tab.name.clone(),
                reason,
            }),
        }
    }
    let unknown_tabs = match &league_policy.tabs {
        TabSelection::All => Vec::new(),
        TabSelection::Ids(ids) => ids
            .iter()
            .filter(|w| !snapshot.tabs.iter().any(|t| &t.id == *w))
            .cloned()
            .collect(),
    };
    let logical_requests = actions.len() as u64;
    let wire_sends = wire_estimate(logical_requests);
    Ok(RefreshPlan {
        plan_schema: REFRESH_PLAN_SCHEMA,
        operation: "refresh".into(),
        provider: provider.into(),
        account_uuid: snapshot.account_uuid.clone(),
        account_name: snapshot.account_name.clone(),
        league: snapshot.league.clone(),
        generated_at: now,
        basis: PlanBasis {
            snapshot_taken_at: snapshot.taken_at,
            listing: snapshot.listing,
            policy_revision,
        },
        max_age_seconds: league_policy.max_age_seconds,
        actions,
        skipped,
        unknown_tabs,
        logical_requests,
        wire_sends,
        quote: None,
    })
}

/// Fetch or skip, for one covered non-folder tab. `Err` is the skip
/// reason — the one place the freshness rules live:
/// never fetched → fetch; older than the window → fetch; a listing newer
/// than our fetch counting differently → fetch (proof of change); fresh
/// with no such proof → skip. An agreeing count is not proof of
/// freshness and cannot skip anything on its own.
fn fetch_verdict(tab: &TabSnapshot, max_age: i64, now: i64) -> Result<FetchReason, SkipReason> {
    let Some(fetched_at) = tab.fetched_at else {
        return Ok(FetchReason::NeverFetched);
    };
    let age = now.saturating_sub(fetched_at).max(0);
    if age > max_age {
        return Ok(FetchReason::Stale { age_seconds: age });
    }
    // The heuristic only reads forward: a listing older than (or racing,
    // same-second) our fetch says nothing about the fetch's contents.
    if let (Some(listed_at), Some(listed)) = (
        tab.listed_at,
        tab.metadata.get("items").and_then(Value::as_i64),
    ) && listed_at > fetched_at
        && listed != tab.item_count
    {
        return Ok(FetchReason::ListedCountDisagrees {
            listed,
            held: tab.item_count,
        });
    }
    Err(SkipReason::Fresh { age_seconds: age })
}

/// A fetch action for `tab`: under its parent when the parent is a
/// map/unique tab on record (a substash), by its own id when the parent
/// is anything else on record (a folder child — folders group, they
/// don't contain) or absent entirely (a top-level tab). `Err` is the one
/// unrenderable case: a recorded parent that is no longer on record —
/// the store keeps substash rows when a listing retires their parent, so
/// guessing an endpoint here would fetch the wrong path.
fn fetch_action(
    snapshot: &StashSnapshot,
    tab: &TabSnapshot,
    reason: FetchReason,
) -> Result<RefreshAction, SkipReason> {
    let parent = match tab.parent.as_deref() {
        None => None,
        Some(p) => match snapshot.tabs.iter().find(|t| t.id == p) {
            None => return Err(SkipReason::OrphanedParent { parent: p.into() }),
            Some(row) => {
                matches!(row.r#type.as_str(), "MapStash" | "UniqueStash").then_some(&row.id)
            }
        },
    };
    Ok(match parent {
        Some(parent) => RefreshAction::FetchSubstash {
            league: snapshot.league.clone(),
            parent: parent.clone(),
            id: tab.id.clone(),
            name: tab.name.clone(),
            tab_type: tab.r#type.clone(),
            reason,
        },
        None => RefreshAction::FetchTab {
            league: snapshot.league.clone(),
            id: tab.id.clone(),
            name: tab.name.clone(),
            tab_type: tab.r#type.clone(),
            reason,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use acquisition_store::{Annotations, Endpoint, SYNC_POLICY_KIND, Store};

    fn item(id: &str) -> Value {
        json!({ "id": id, "name": "Foo", "typeLine": "Imperial Bow", "baseType": "Imperial Bow", "x": 0, "y": 0 })
    }

    /// A store with its account identity on record, as every real store's
    /// is after one login.
    fn store() -> Store {
        let mut s = Store::open_memory().unwrap();
        s.record(
            &Endpoint::Profile,
            &json!({}),
            200,
            &json!({ "uuid": "u-1", "name": "tom" }),
            1,
        )
        .unwrap();
        s
    }

    fn list(s: &mut Store, stashes: Value, at: i64) {
        s.record(
            &Endpoint::Stashes {
                league: "Standard".into(),
            },
            &json!({ "league": "Standard" }),
            200,
            &json!({ "stashes": stashes }),
            at,
        )
        .unwrap();
    }

    fn fetch(s: &mut Store, stash: Value, at: i64) {
        let id = stash["id"].as_str().unwrap().to_string();
        s.record(
            &Endpoint::Stash {
                league: "Standard".into(),
                id,
                sub: None,
            },
            &json!({ "league": "Standard" }),
            200,
            &json!({ "stash": stash }),
            at,
        )
        .unwrap();
    }

    fn policy(value: Value) -> SyncPolicy {
        SyncPolicy::from_value(&value).unwrap()
    }

    fn all_policy_value(max_age_seconds: u32) -> Value {
        json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": "all", "max_age_seconds": max_age_seconds } }
        })
    }

    fn snapshot(s: &Store) -> StashSnapshot {
        let a = Annotations::open_memory_for("u-1").unwrap();
        s.stash_snapshot("Standard", &a).unwrap()
    }

    /// Snapshot with `policy` installed as the stored sync-policy row —
    /// the only way plans are made: from stored intent, at its revision.
    fn snapshot_with(s: &Store, policy: &Value) -> StashSnapshot {
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        a.put("account", "", SYNC_POLICY_KIND, policy, None)
            .unwrap();
        s.stash_snapshot("Standard", &a).unwrap()
    }

    fn plan(s: &Store, policy: &Value, now: i64) -> RefreshPlan {
        plan_refresh("mock", &snapshot_with(s, policy), now).unwrap()
    }

    #[test]
    fn the_policy_parses_strictly_and_versions_are_refused_not_guessed() {
        // The valid shapes: "all" and an explicit id list.
        let p = policy(json!({
            "version": 1,
            "leagues": {
                "Standard": { "tabs": "all", "max_age_seconds": 3600 },
                "Hardcore": { "tabs": ["t1", "s1"], "max_age_seconds": 60 },
            }
        }));
        assert_eq!(p.leagues["Standard"].tabs, TabSelection::All);
        assert_eq!(
            p.leagues["Hardcore"].tabs,
            TabSelection::Ids(vec!["t1".into(), "s1".into()])
        );
        // A typo'd field is malformed, never silently ignored — the policy
        // is intent, and intent that half-parses is worse than an error.
        let err = SyncPolicy::from_value(&json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": "all", "max_age_secs": 60 } }
        }))
        .unwrap_err();
        assert!(matches!(err, PlanError::MalformedPolicy { .. }), "{err}");
        // So is a selection word that isn't "all", and a missing version.
        let err = SyncPolicy::from_value(&json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": "everything", "max_age_seconds": 60 } }
        }))
        .unwrap_err();
        assert!(matches!(err, PlanError::MalformedPolicy { .. }), "{err}");
        let err = SyncPolicy::from_value(&json!({ "leagues": {} })).unwrap_err();
        assert!(matches!(err, PlanError::MalformedPolicy { .. }), "{err}");
        // A newer version is refused as such — checked before the shape,
        // so a v2 policy with v2 fields is not misreported as a typo.
        let err = SyncPolicy::from_value(&json!({
            "version": 2,
            "leagues": {},
            "some_v2_field": true
        }))
        .unwrap_err();
        assert_eq!(
            err,
            PlanError::PolicyVersionUnsupported {
                found: 2,
                supported: 1
            }
        );
        // The policy round-trips: what a frontend writes is inspectable.
        let back: SyncPolicy = serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
        assert_eq!(back, p);
    }

    #[test]
    fn no_policy_and_uncovered_league_are_distinct_structured_errors() {
        let s = store();
        // No sync-policy row: nothing declared, nothing to compile.
        let err = plan_refresh("mock", &snapshot(&s), 1000).unwrap_err();
        assert_eq!(err, PlanError::NoSyncPolicy);
        // A policy that does not name the league authorizes nothing there.
        let hc_only = json!({
            "version": 1,
            "leagues": { "Hardcore": { "tabs": "all", "max_age_seconds": 60 } }
        });
        let err = plan_refresh("mock", &snapshot_with(&s, &hc_only), 1000).unwrap_err();
        assert_eq!(
            err,
            PlanError::LeagueNotCovered {
                league: "Standard".into()
            }
        );
        // A stored policy row a newer build wrote surfaces its version,
        // and the version gate is not bypassable by deserializing the
        // type directly instead of calling from_value.
        let v2 = json!({ "version": 2, "leagues": {} });
        let err = plan_refresh("mock", &snapshot_with(&s, &v2), 1000).unwrap_err();
        assert_eq!(
            err,
            PlanError::PolicyVersionUnsupported {
                found: 2,
                supported: 1
            }
        );
        let err = serde_json::from_value::<SyncPolicy>(v2).unwrap_err();
        assert!(err.to_string().contains("version 2"), "{err}");
    }

    #[test]
    fn a_never_listed_league_plans_the_listing_alone() {
        let mut s = store();
        // A tab on record from a direct fetch, stale by any window — and
        // still not fetched: with no listing basis the plan has no
        // membership authority, so the listing goes alone and the tab is
        // reported as waiting (D5a's eventual reconciliation).
        fetch(
            &mut s,
            json!({ "id": "x1", "name": "Fetched", "type": "PremiumStash", "items": [item("i1")] }),
            10,
        );
        let plan = plan(&s, &all_policy_value(3600), 100_000);
        assert_eq!(
            plan.actions,
            vec![RefreshAction::ListStashes {
                league: "Standard".into(),
                reason: ListingReason::NeverListed,
            }]
        );
        assert_eq!(
            plan.skipped,
            vec![SkippedTab {
                id: "x1".into(),
                name: "Fetched".into(),
                reason: SkipReason::AwaitingListing,
            }]
        );
        assert_eq!(plan.basis.listing, None);
        assert_eq!(plan.logical_requests, 1);
        assert_eq!((plan.wire_sends.min, plan.wire_sends.max), (1, 3));
        assert_eq!(plan.wire_sends.prerequisites.len(), 2);
    }

    #[test]
    fn fresh_facts_compile_to_an_empty_plan_that_still_cites_its_basis() {
        let mut s = store();
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            1000,
        );
        fetch(
            &mut s,
            json!({ "id": "t1", "name": "One", "type": "PremiumStash", "items": [item("i1")] }),
            1010,
        );
        let row = a
            .put(
                "account",
                "",
                SYNC_POLICY_KIND,
                &json!({ "version": 1, "leagues": { "Standard": { "tabs": "all", "max_age_seconds": 3600 } } }),
                None,
            )
            .unwrap();
        let snap = s.stash_snapshot("Standard", &a).unwrap();
        let plan = plan_refresh("mock", &snap, 1100).unwrap();
        assert!(plan.actions.is_empty());
        assert_eq!(plan.logical_requests, 0);
        assert_eq!((plan.wire_sends.min, plan.wire_sends.max), (0, 0));
        assert!(plan.wire_sends.prerequisites.is_empty());
        // The plan shows its reasoning: the fresh tab is named as skipped.
        assert_eq!(
            plan.skipped,
            vec![SkippedTab {
                id: "t1".into(),
                name: "One".into(),
                reason: SkipReason::Fresh { age_seconds: 90 },
            }]
        );
        // And its basis: uuid, provider, listing row, policy revision.
        assert_eq!(plan.account_uuid, "u-1");
        assert_eq!(plan.provider, "mock");
        assert_eq!(plan.basis.listing, snap.listing);
        assert_eq!(plan.basis.policy_revision, row.revision);
        assert_eq!(plan.basis.snapshot_taken_at, snap.taken_at);
        assert_eq!(plan.plan_schema, REFRESH_PLAN_SCHEMA);
        assert_eq!(plan.operation, "refresh");
    }

    #[test]
    fn staleness_forces_fetches_and_a_stale_listing_relists() {
        let mut s = store();
        list(
            &mut s,
            json!([
                { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 },
                { "id": "t2", "name": "Two", "type": "PremiumStash", "index": 1 },
            ]),
            1000,
        );
        fetch(
            &mut s,
            json!({ "id": "t1", "name": "One", "type": "PremiumStash", "items": [item("i1")] }),
            1010,
        );
        // At t=5000 with a 3600s window: the listing (age 4000) is stale,
        // t1 (age 3990) is stale, t2 was never fetched.
        let plan = plan(&s, &all_policy_value(3600), 5000);
        assert_eq!(
            plan.actions,
            vec![
                RefreshAction::ListStashes {
                    league: "Standard".into(),
                    reason: ListingReason::Stale { age_seconds: 4000 },
                },
                RefreshAction::FetchTab {
                    league: "Standard".into(),
                    id: "t1".into(),
                    name: "One".into(),
                    tab_type: "PremiumStash".into(),
                    reason: FetchReason::Stale { age_seconds: 3990 },
                },
                RefreshAction::FetchTab {
                    league: "Standard".into(),
                    id: "t2".into(),
                    name: "Two".into(),
                    tab_type: "PremiumStash".into(),
                    reason: FetchReason::NeverFetched,
                },
            ]
        );
        assert_eq!(plan.logical_requests, 3);
        assert_eq!((plan.wire_sends.min, plan.wire_sends.max), (3, 9));
    }

    #[test]
    fn a_disagreeing_count_proves_change_and_an_agreeing_one_proves_nothing() {
        let mut s = store();
        list(
            &mut s,
            json!([{ "id": "m1", "name": "Maps", "type": "MapStash", "index": 0 }]),
            1000,
        );
        // Fetching the map tab lands the substash stub (metadata.items 2),
        // then the substash itself is fetched and holds one item.
        fetch(
            &mut s,
            json!({ "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
                { "id": "s1", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 2, "map": { "name": "Tier 16" } } } ] }),
            1010,
        );
        s.record(
            &Endpoint::Stash {
                league: "Standard".into(),
                id: "m1".into(),
                sub: Some("s1".into()),
            },
            &json!({ "league": "Standard" }),
            200,
            &json!({ "stash": { "id": "s1", "name": "", "type": "MapStash", "parent": "m1", "items": [item("map1")] } }),
            1020,
        )
        .unwrap();
        // Everything is fresh at t=1100 — the stub's count (2 vs 1 held)
        // predates our fetch, so it proves nothing.
        let quiet = plan(&s, &all_policy_value(3600), 1100);
        assert!(quiet.actions.is_empty(), "{:?}", quiet.actions);
        // Re-fetching the parent stamps a newer stub whose count disagrees
        // with what we hold: proof of change, fetched though fresh by age —
        // and under the parent, as a substash.
        fetch(
            &mut s,
            json!({ "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
                { "id": "s1", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 3, "map": { "name": "Tier 16" } } } ] }),
            1050,
        );
        let changed = plan(&s, &all_policy_value(3600), 1100);
        assert_eq!(
            changed.actions,
            vec![RefreshAction::FetchSubstash {
                league: "Standard".into(),
                parent: "m1".into(),
                id: "s1".into(),
                name: "".into(),
                tab_type: "MapStash".into(),
                reason: FetchReason::ListedCountDisagrees { listed: 3, held: 1 },
            }]
        );
        // An agreeing count cannot skip a stale fetch: at t=9000 the
        // substash is fetched again even if the counts matched — the
        // heuristic proves change, never freshness.
        let stale = plan(&s, &all_policy_value(3600), 9000);
        assert!(stale.actions.iter().any(|a| matches!(
            a,
            RefreshAction::FetchSubstash { id, reason: FetchReason::Stale { .. }, .. } if id == "s1"
        )));
    }

    #[test]
    fn an_orphaned_substash_is_skipped_never_rendered_by_the_wrong_path() {
        let mut s = store();
        list(
            &mut s,
            json!([{ "id": "m1", "name": "Maps", "type": "MapStash", "index": 0 }]),
            1000,
        );
        fetch(
            &mut s,
            json!({ "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
                { "id": "s1", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 2 } } ] }),
            1010,
        );
        // A later listing retires m1; the store keeps the never-listed
        // substash row, whose recorded parent is now off the record. A
        // fetch by its own id would hit /stash/{league}/s1 — the wrong
        // endpoint — so the plan reports it instead of guessing.
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            2000,
        );
        let plan = plan(&s, &all_policy_value(3600), 2100);
        assert!(
            !plan
                .actions
                .iter()
                .any(|a| matches!(a, RefreshAction::FetchTab { id, .. } if id == "s1")),
            "orphaned substash must not become a top-level fetch: {:?}",
            plan.actions
        );
        assert!(
            plan.skipped.contains(&SkippedTab {
                id: "s1".into(),
                name: "".into(),
                reason: SkipReason::OrphanedParent {
                    parent: "m1".into()
                },
            }),
            "{:?}",
            plan.skipped
        );
    }

    #[test]
    fn corrupt_store_timestamps_read_as_stale_never_panic_or_wrap() {
        let mut s = store();
        // Absurd timestamps a damaged file could hold: pre-fix, the age
        // subtraction overflowed (panic in debug, wrap-to-fresh in
        // release). They must saturate into "very stale".
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            i64::MIN,
        );
        fetch(
            &mut s,
            json!({ "id": "t1", "name": "One", "type": "PremiumStash", "items": [] }),
            i64::MIN,
        );
        let plan = plan(&s, &all_policy_value(3600), 100);
        assert!(matches!(
            plan.actions[0],
            RefreshAction::ListStashes {
                reason: ListingReason::Stale {
                    age_seconds: i64::MAX
                },
                ..
            }
        ));
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            RefreshAction::FetchTab { id, reason: FetchReason::Stale { age_seconds: i64::MAX }, .. } if id == "t1"
        )));
    }

    #[test]
    fn folders_are_never_fetched_and_their_children_go_by_own_id() {
        let mut s = store();
        list(
            &mut s,
            json!([
                { "id": "f1", "name": "Folder", "type": "Folder", "index": 0,
                  "children": [ { "id": "c1", "name": "In folder", "type": "PremiumStash", "index": 1 } ] },
            ]),
            1000,
        );
        let plan = plan(&s, &all_policy_value(3600), 1100);
        // The folder is skipped with its reason; the child is a plain tab
        // fetch by its own id (folders group, they don't contain).
        assert_eq!(
            plan.actions,
            vec![RefreshAction::FetchTab {
                league: "Standard".into(),
                id: "c1".into(),
                name: "In folder".into(),
                tab_type: "PremiumStash".into(),
                reason: FetchReason::NeverFetched,
            }]
        );
        assert_eq!(
            plan.skipped,
            vec![SkippedTab {
                id: "f1".into(),
                name: "Folder".into(),
                reason: SkipReason::Folder,
            }]
        );
    }

    #[test]
    fn ids_the_facts_do_not_know_are_reported_never_invented() {
        let mut s = store();
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            1000,
        );
        let p = json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": ["t1", "ghost"], "max_age_seconds": 3600 } }
        });
        let plan = plan(&s, &p, 1100);
        // The known tab gets its action; the vanished (or mistyped) id is
        // reported, and no action names it (D5a: reported skipped).
        assert_eq!(plan.actions.len(), 1);
        assert!(matches!(
            &plan.actions[0],
            RefreshAction::FetchTab { id, .. } if id == "t1"
        ));
        assert_eq!(plan.unknown_tabs, vec!["ghost".to_string()]);
    }

    #[test]
    fn actions_decode_through_the_stores_job_vocabulary_decoder() {
        // Not literals-vs-literals: `Endpoint::from_job` is the store's
        // production decoder of the daemon's job vocabulary (it is what
        // `Daemon::record` classifies through), so a vocabulary change
        // that moves the store breaks this test even if this crate's
        // strings were left behind.
        let listing = RefreshAction::ListStashes {
            league: "Standard".into(),
            reason: ListingReason::NeverListed,
        };
        let (kind, params) = listing.job();
        assert_eq!(
            Endpoint::from_job(kind, &params),
            Some(Endpoint::Stashes {
                league: "Standard".into()
            })
        );
        let tab = RefreshAction::FetchTab {
            league: "Standard".into(),
            id: "t1".into(),
            name: "One".into(),
            tab_type: "PremiumStash".into(),
            reason: FetchReason::NeverFetched,
        };
        let (kind, params) = tab.job();
        assert_eq!(
            Endpoint::from_job(kind, &params),
            Some(Endpoint::Stash {
                league: "Standard".into(),
                id: "t1".into(),
                sub: None,
            })
        );
        // `deep` is not part of the endpoint; assert it separately — a
        // plan's fetch must never fan out (D5a).
        assert_eq!(params["deep"], json!(false));
        let substash = RefreshAction::FetchSubstash {
            league: "Standard".into(),
            parent: "m1".into(),
            id: "s1".into(),
            name: "".into(),
            tab_type: "MapStash".into(),
            reason: FetchReason::NeverFetched,
        };
        let (kind, params) = substash.job();
        assert_eq!(
            Endpoint::from_job(kind, &params),
            Some(Endpoint::Stash {
                league: "Standard".into(),
                id: "m1".into(),
                sub: Some("s1".into()),
            })
        );
        assert_eq!(params["deep"], json!(false));
    }

    #[test]
    fn applying_a_plans_actions_through_the_store_satisfies_the_plan() {
        // The offline half of apply: each action's job tuple, decoded by
        // the store's own vocabulary and recorded, produces facts that
        // the next plan finds fresh. If job() rendered anything the
        // record pipeline does not recognize as the planned tab, the
        // replan would still want work.
        let mut s = store();
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            1000,
        );
        let p = all_policy_value(3600);
        let first = plan(&s, &p, 8000);
        assert_eq!(first.logical_requests, 2, "{:?}", first.actions);
        for action in &first.actions {
            let (kind, params) = action.job();
            let endpoint = Endpoint::from_job(kind, &params).unwrap();
            let body = match action {
                RefreshAction::ListStashes { .. } => {
                    json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] })
                }
                _ => {
                    json!({ "stash": { "id": "t1", "name": "One", "type": "PremiumStash", "items": [item("i1")] } })
                }
            };
            s.record(&endpoint, &params, 200, &body, 8000).unwrap();
        }
        let second = plan(&s, &p, 8000);
        assert!(second.actions.is_empty(), "{:?}", second.actions);
    }

    #[test]
    fn the_plan_round_trips_through_serde_unchanged() {
        let mut s = store();
        list(
            &mut s,
            json!([
                { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 },
                { "id": "f1", "name": "Folder", "type": "Folder", "index": 1,
                  "children": [ { "id": "c1", "name": "In folder", "type": "PremiumStash", "index": 2 } ] },
            ]),
            1000,
        );
        let p = json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": ["t1", "f1", "c1", "ghost"], "max_age_seconds": 60 } }
        });
        let plan = plan(&s, &p, 5000);
        // A plan crosses sockets and lands in journals; serialization is
        // part of its contract, not a debug convenience.
        let json = serde_json::to_value(&plan).unwrap();
        assert_eq!(RefreshPlan::from_value(&json).unwrap(), plan);
        let back: RefreshPlan = serde_json::from_value(json).unwrap();
        assert_eq!(back, plan);
    }

    #[test]
    fn a_quote_enriches_a_plan_optionally_and_must_speak_about_it() {
        use acquisition_core::protocol::QuoteScope;
        let mut s = store();
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            1000,
        );
        let bare = plan(&s, &all_policy_value(60), 5000);
        assert_eq!(bare.quote, None, "compiling never fills the quote");
        let quote = Quote {
            observed_at: 5000,
            provider: bare.provider.clone(),
            account: bare.account_name.clone(),
            halted: None,
            scopes: vec![QuoteScope {
                key: "stash-list".into(),
                endpoints: vec!["stash-list".into()],
                requests: 1,
                queued_ahead: 0,
                policy: None,
                rules: Vec::new(),
                eta_seconds: None,
                notes: vec!["policy not yet learned".into()],
            }],
            not_covered: vec!["a HEAD probe on first contact (N16)".into()],
        };
        let enriched = bare.clone().with_quote(quote.clone()).unwrap();
        assert_eq!(enriched.quote.as_ref(), Some(&quote));
        // The enriched plan round-trips like everything else it carries.
        let json = serde_json::to_value(&enriched).unwrap();
        assert_eq!(RefreshPlan::from_value(&json).unwrap(), enriched);
        // A smuggled field inside the carried quote refuses at parse.
        let mut nested = json.clone();
        nested["quote"]["extra"] = json!(true);
        assert!(RefreshPlan::from_value(&nested).is_err());
        let mut nested = json.clone();
        nested["quote"]["scopes"][0]["surprise"] = json!(true);
        assert!(RefreshPlan::from_value(&nested).is_err());
        // A quote about another provider or account refuses — at attach
        // and at parse alike: it would mislead the review the plan is for.
        let mut foreign = quote.clone();
        foreign.provider = "ggg".into();
        assert!(matches!(
            bare.clone().with_quote(foreign),
            Err(PlanError::MalformedPlan { .. })
        ));
        let mut tampered = json.clone();
        tampered["quote"]["provider"] = json!("ggg");
        assert!(RefreshPlan::from_value(&tampered).is_err());
        let mut other_account = quote.clone();
        other_account.account = Some("mallory#9999".into());
        assert!(bare.clone().with_quote(other_account).is_err());
        // An accountless quote (none of the work resolved to an account)
        // still attaches: there is no claim to contradict.
        let mut accountless = quote;
        accountless.account = None;
        assert!(bare.with_quote(accountless).is_ok());
    }

    #[test]
    fn a_tampered_or_newer_plan_does_not_parse_into_a_trusted_envelope() {
        let mut s = store();
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            1000,
        );
        let good = serde_json::to_value(plan(&s, &all_policy_value(60), 5000)).unwrap();
        // Sanity: 2 actions (re-list + fetch), and the untampered form parses.
        assert_eq!(good["logical_requests"], json!(2));
        assert!(RefreshPlan::from_value(&good).is_ok());
        // A newer schema reports as such — before any shape complaint —
        // and raw serde refuses it too (the validation is in the type,
        // not only in from_value).
        let mut newer = good.clone();
        newer["plan_schema"] = json!(2);
        assert_eq!(
            RefreshPlan::from_value(&newer).unwrap_err(),
            PlanError::PlanSchemaUnsupported {
                found: 2,
                supported: 1
            }
        );
        assert!(serde_json::from_value::<RefreshPlan>(newer).is_err());
        // Derived counts recompute or the envelope is refused: a forged
        // logical bound is exactly what admission-time budgeting (D8)
        // must not be able to trust away.
        let mut forged = good.clone();
        forged["logical_requests"] = json!(1);
        let err = RefreshPlan::from_value(&forged).unwrap_err();
        assert!(matches!(err, PlanError::MalformedPlan { .. }), "{err}");
        let mut forged = good.clone();
        forged["wire_sends"]["max"] = json!(999);
        assert!(RefreshPlan::from_value(&forged).is_err());
        // The prerequisites are part of the reviewed wire projection, not
        // display text: an emptied (or reworded) list refuses too.
        let mut forged = good.clone();
        forged["wire_sends"]["prerequisites"] = json!([]);
        let err = RefreshPlan::from_value(&forged).unwrap_err();
        assert!(err.to_string().contains("prerequisites"), "{err}");
        // A stray operation, a smuggled field, and an action for another
        // league are each refused whole.
        let mut wrong_op = good.clone();
        wrong_op["operation"] = json!("delete-everything");
        assert!(RefreshPlan::from_value(&wrong_op).is_err());
        let mut smuggled = good.clone();
        smuggled["extra_authorization"] = json!(true);
        assert!(RefreshPlan::from_value(&smuggled).is_err());
        // Unknown fields refuse at every depth, not only the top level:
        // inside the wire estimate, an action, a reason, and the basis
        // (its listing included).
        for path in [
            &["wire_sends", "extra"][..],
            &["actions", "0", "surprise"],
            &["actions", "0", "reason", "surprise"],
            &["basis", "extra"],
            &["basis", "listing", "extra"],
        ] {
            let mut nested = good.clone();
            let mut spot = &mut nested;
            for key in &path[..path.len() - 1] {
                spot = match key.parse::<usize>() {
                    Ok(i) => &mut spot[i],
                    Err(_) => &mut spot[*key],
                };
            }
            spot[*path.last().unwrap()] = json!(true);
            assert!(
                RefreshPlan::from_value(&nested).is_err(),
                "smuggled field at {path:?} was accepted"
            );
        }
        let mut stray = good.clone();
        stray["actions"][0]["league"] = json!("Hardcore");
        let err = RefreshPlan::from_value(&stray).unwrap_err();
        assert!(err.to_string().contains("Hardcore"), "{err}");
    }
}
