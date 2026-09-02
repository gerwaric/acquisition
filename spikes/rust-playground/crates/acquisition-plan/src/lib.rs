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
//! - A policy id covers the tab **and its children** (decided
//!   2026-09-01, tracer rung): a map/unique tab's substashes are planned
//!   once their stubs are on record — the cycle after the parent's first
//!   fetch — and a folder's children at once, since the listing carries
//!   them. One rule, no per-type logic; binding is untouched (every action
//!   is still an explicit tuple). A substash stub reporting 0 items with
//!   nothing held is skipped as `empty_stub` (GGG appears to list only
//!   non-empty substashes; a guard, not a saving).
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
use acquisition_core::realm::{Family, Realm};
use acquisition_store::{
    AnnotationError, AnnotationRow, Annotations, ListingBasis, SYNC_POLICY_KEY, SYNC_POLICY_KIND,
    SYNC_POLICY_SCOPE, StashSnapshot, TabSnapshot,
};

/// The plan envelope schema this build writes ([`RefreshPlan::plan_schema`]).
/// A consumer handed a plan stamped newer refuses it rather than guessing.
/// History: v1 = the tracer-step-4 envelope; v2 (2026-09-01) added the
/// optional `quote` enrichment; v3 (same day) added `Quote::work`, the
/// quote's verifiable work basis. A shape change anywhere in the envelope —
/// the embedded `Quote` included — is a schema bump, so an older reader
/// reports "newer schema" instead of "malformed" on a newer plan.
/// v4 added the `empty_stub` skip kind; v5 (2026-09-02) put `realm`
/// beside `league` on the envelope and on every action.
pub const REFRESH_PLAN_SCHEMA: i64 = 5;

/// The sync-policy value schema this build reads ([`SyncPolicy::version`]).
/// The store carries the row opaquely; its shape is this crate's business.
/// v2 (2026-09-02) nests leagues under realms
/// (`realms.<R>.leagues.<L>`); a v1 policy (`leagues.<L>`) still parses,
/// upgraded on the way in as realm pc — the only realm v1 could mean.
pub const SYNC_POLICY_VERSION: i64 = 2;

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
    /// The policy declares nothing for the snapshot's (realm, league);
    /// there is no authorized work to derive.
    LeagueNotCovered { realm: Realm, league: String },
    /// The snapshot names a realm this build does not know. Facts are
    /// stamped with the request's realm, so this is a store written by a
    /// build with a wider table — refused, never guessed at.
    UnknownRealm { realm: String },
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
            PlanError::LeagueNotCovered { realm, league } => {
                write!(
                    f,
                    "the sync policy does not cover league {league} on realm {realm}"
                )
            }
            PlanError::UnknownRealm { realm } => {
                write!(
                    f,
                    "the snapshot names realm {realm:?}, which this build does not know"
                )
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
/// compiled here into minimal requests. In memory it is always the v2
/// shape; a stored v1 value is upgraded on parse (realm pc) and stays
/// stored as written — what the human typed is what `policy show` shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SyncPolicy {
    /// [`SYNC_POLICY_VERSION`] once parsed (a v1 value reads as v2).
    pub version: i64,
    /// Coverage per realm, then per league. A (realm, league) not named
    /// here compiles to [`PlanError::LeagueNotCovered`], never to
    /// implicit work.
    pub realms: BTreeMap<Realm, RealmPolicy>,
}

/// One realm's coverage: its leagues. The realm key is what the requests
/// are rendered with; a league entry that names work an endpoint family
/// does not take on this realm (tabs under `poe2`, PoE1-only stashes) is
/// a parse error, so no policy can ask for an unobserved URL shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RealmPolicy {
    pub leagues: BTreeMap<String, LeaguePolicy>,
}

/// The raw JSON shapes, one strict struct per version — `deny_unknown_fields`
/// on each, so a stray top-level field is refused whatever the stamp (an
/// untagged enum would lose that: review finding 2026-09-02). The stamp
/// picks exactly one shape ([`parse_policy`]), and `SyncPolicy`'s
/// `Deserialize` goes through the same function, so the version gate, the
/// unknown-field refusal, and the per-realm family check cannot be
/// bypassed by deserializing around [`SyncPolicy::from_value`].
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyncPolicyWireV1 {
    #[allow(dead_code)]
    version: i64,
    leagues: BTreeMap<String, LeaguePolicy>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SyncPolicyWireV2 {
    #[allow(dead_code)]
    version: i64,
    realms: BTreeMap<Realm, RealmPolicy>,
}

impl<'de> Deserialize<'de> for SyncPolicy {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(d)?;
        parse_policy(&value).map_err(serde::de::Error::custom)
    }
}

/// The one parse: the `version` stamp selects the strict shape, the
/// shape is validated whole, a v1 value upgrades to realm pc, and no
/// realm may name work its endpoint family does not take.
fn parse_policy(value: &Value) -> Result<SyncPolicy, String> {
    let version = value
        .get("version")
        .and_then(Value::as_i64)
        .ok_or_else(|| "missing integer `version`".to_string())?;
    let realms = match version {
        1 => {
            let wire: SyncPolicyWireV1 =
                serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
            BTreeMap::from([(
                Realm::Pc,
                RealmPolicy {
                    leagues: wire.leagues,
                },
            )])
        }
        2 => {
            let wire: SyncPolicyWireV2 =
                serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
            wire.realms
        }
        other => {
            return Err(format!(
                "sync policy declares version {other}, not this build's v{SYNC_POLICY_VERSION}"
            ));
        }
    };
    for (realm, policy) in &realms {
        if !Family::Stashes.accepts(*realm) && !policy.leagues.is_empty() {
            return Err(format!(
                "tabs under realm {realm}: the stash endpoints do not take it (PoE1 only)"
            ));
        }
    }
    Ok(SyncPolicy {
        version: SYNC_POLICY_VERSION,
        realms,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaguePolicy {
    pub tabs: TabSelection,
    /// The freshness declaration: facts older than this want refreshing.
    /// Applies to the listing and to each selected tab's fetch alike.
    pub max_age_seconds: u32,
}

/// Which tabs the policy covers: `"all"` or an explicit id list. An id
/// covers the tab and its children: a map/unique tab's substashes (their
/// own GGG ids; the `parent/id` display convention is a frontend matter)
/// and a folder's children. A child named directly is covered too.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TabSelectionRepr", into = "TabSelectionRepr")]
pub enum TabSelection {
    All,
    Ids(Vec<String>),
}

impl TabSelection {
    /// Covered when the tab's own id is listed or its parent's is.
    fn covers(&self, tab: &TabSnapshot) -> bool {
        match self {
            TabSelection::All => true,
            TabSelection::Ids(ids) => ids
                .iter()
                .any(|i| i == &tab.id || Some(i.as_str()) == tab.parent.as_deref()),
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
    /// unknown-field complaint. v1 and v2 both parse (v1 upgrades to
    /// realm pc).
    pub fn from_value(value: &Value) -> Result<SyncPolicy, PlanError> {
        let found = value
            .get("version")
            .and_then(Value::as_i64)
            .ok_or_else(|| PlanError::MalformedPolicy {
                detail: "missing integer `version`".into(),
            })?;
        if found > SYNC_POLICY_VERSION {
            return Err(PlanError::PolicyVersionUnsupported {
                found,
                supported: SYNC_POLICY_VERSION,
            });
        }
        parse_policy(value).map_err(|detail| PlanError::MalformedPolicy { detail })
    }
}

/// Why a sync-policy write was refused: the value is not a policy this
/// build compiles, or the store's compare-and-swap (or the store itself)
/// refused the put. Split so a caller can render a CAS conflict — which
/// carries the current row to re-read — differently from a typo.
#[derive(Debug)]
pub enum PutPolicyError {
    Invalid(PlanError),
    Store(AnnotationError),
}

impl std::fmt::Display for PutPolicyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PutPolicyError::Invalid(e) => write!(f, "{e}"),
            PutPolicyError::Store(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PutPolicyError {}

/// Write the sync policy, validated first — the one shared write path for
/// every frontend's policy surface. Validation precedes the put on
/// purpose: the policy is intent, and the planner refuses a typo'd or
/// newer-versioned value on every parse — storing one anyway would just
/// move that error to plan time, with the typo now on disk.
///
/// `expected_revision` is the store's compare-and-swap, verbatim:
/// `Some(r)` replaces exactly the revision the caller reviewed, `None`
/// creates (refused if a policy exists). A frontend that wants a softer
/// default (the CLI's "replace whatever is stored") reads the current
/// revision itself and passes it here — the blind form is a frontend
/// policy, not this function's.
pub fn put_sync_policy(
    annotations: &mut Annotations,
    value: &Value,
    expected_revision: Option<i64>,
) -> Result<AnnotationRow, PutPolicyError> {
    SyncPolicy::from_value(value).map_err(PutPolicyError::Invalid)?;
    annotations
        .put(
            SYNC_POLICY_SCOPE,
            SYNC_POLICY_KEY,
            SYNC_POLICY_KIND,
            value,
            expected_revision,
        )
        .map_err(PutPolicyError::Store)
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
    /// A map/unique substash whose stub reports 0 items while the store
    /// holds none: there is nothing to fetch. A guard — GGG appears to list
    /// only non-empty substashes — not a freshness claim (a count never
    /// proves a tab unchanged; a held item against a 0 count is the
    /// disagreement arm, which fetches).
    EmptyStub,
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
        realm: Realm,
        league: String,
        reason: ListingReason,
    },
    FetchTab {
        realm: Realm,
        league: String,
        id: String,
        name: String,
        tab_type: String,
        reason: FetchReason,
    },
    /// A map/unique substash, fetched under its parent tab. Its stub is
    /// already on record — v1 plans never fan out dynamically.
    FetchSubstash {
        realm: Realm,
        league: String,
        parent: String,
        id: String,
        name: String,
        tab_type: String,
        reason: FetchReason,
    },
}

impl RefreshAction {
    /// The realm this action touches — validated against the envelope's
    /// on deserialization.
    pub fn realm(&self) -> Realm {
        match self {
            RefreshAction::ListStashes { realm, .. }
            | RefreshAction::FetchTab { realm, .. }
            | RefreshAction::FetchSubstash { realm, .. } => *realm,
        }
    }

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
    /// as `Submit` wants them. The realm is always explicit — pc included
    /// — so a tuple says where it goes without a decode default; `deep`
    /// is always false — a plan's actions are the reviewed set, and a
    /// fetch that fanned out would expand it.
    pub fn job(&self) -> (&'static str, Value) {
        match self {
            RefreshAction::ListStashes { realm, league, .. } => {
                ("stashes", json!({ "realm": realm, "league": league }))
            }
            RefreshAction::FetchTab {
                realm, league, id, ..
            } => (
                "stash",
                json!({ "realm": realm, "league": league, "id": id, "deep": false }),
            ),
            RefreshAction::FetchSubstash {
                realm,
                league,
                parent,
                id,
                ..
            } => (
                "stash",
                json!({ "realm": realm, "league": league, "id": parent, "sub": id, "deep": false }),
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
    /// The coordinate above league; every action is on this realm.
    pub realm: Realm,
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
    /// validation pins what *is* checkable: the plan's provider, exactly
    /// its account, the quote's echoed `work` being exactly this plan's
    /// actions as job tuples, and scope totals that sum (checked) to the
    /// logical bound ([`check_quote_matches`]). Compiling never fills it
    /// (a plan needs no daemon); [`RefreshPlan::with_quote`] attaches
    /// one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<Quote>,
}

/// What a carried quote must have in common with its envelope. The quote
/// is an observation and cannot be recomputed, but it must be *about*
/// this plan: the same provider; exactly the plan's account (`None`
/// matches only a plan bound to no account — an accountless quote on an
/// account-bound plan is a quote for someone else's limiter state); the
/// quote's echoed `work` must be exactly the plan's actions rendered as
/// job tuples, in order — the verifiable work basis, where matching
/// totals alone would let a quote for other work of the same size stand
/// in; and the scope totals must still sum (checked — a malformed
/// envelope must not wrap past validation) to the plan's logical bound.
/// A mismatch on any of these would mislead exactly the review the plan
/// exists for.
fn check_quote_matches(
    quote: &Quote,
    provider: &str,
    account_name: Option<&str>,
    actions: &[RefreshAction],
) -> Result<(), String> {
    if quote.provider != provider {
        return Err(format!(
            "the quote projects provider {:?}, but the plan is for {provider:?}",
            quote.provider
        ));
    }
    if quote.account.as_deref() != account_name {
        return Err(format!(
            "the quote projects account {:?}, but the plan is for {:?}",
            quote.account, account_name
        ));
    }
    if quote.work.len() != actions.len() {
        return Err(format!(
            "the quote projects {} job(s), but the plan authorizes {} action(s)",
            quote.work.len(),
            actions.len()
        ));
    }
    for (i, (job, action)) in quote.work.iter().zip(actions).enumerate() {
        let (kind, params) = action.job();
        if job.kind != kind || job.params != params {
            return Err(format!(
                "quoted job {i} ({} {}) is not the plan's action {i} ({kind} {params})",
                job.kind, job.params
            ));
        }
    }
    let quoted = quote
        .scopes
        .iter()
        .try_fold(0u64, |sum, s| sum.checked_add(s.requests))
        .ok_or_else(|| "the quote's scope request total overflows".to_string())?;
    if quoted != actions.len() as u64 {
        return Err(format!(
            "the quote's scopes project {quoted} request(s), but the plan authorizes {}",
            actions.len()
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
        check_quote_matches(
            &quote,
            &self.provider,
            self.account_name.as_deref(),
            &self.actions,
        )
        .map_err(|detail| PlanError::MalformedPlan { detail })?;
        self.quote = Some(quote);
        Ok(self)
    }
}

/// Why a plan must not be spent — the step-7 staleness/identity gate
/// (CONTEXT.md, decided 2026-09-01), shared by every frontend's apply
/// surface. A plan is authorization *derived from intent at a revision*;
/// intent edited since revokes the derivation, and a plan for another
/// identity is never spent here. Fact drift deliberately does not refuse:
/// the authorization is the bounded action set, not a world-state
/// assertion, and the next plan reconciles.
#[derive(Debug)]
pub enum SpendError {
    /// The plan names a different provider than this frontend runs against.
    WrongProvider { plan: String, running: String },
    /// The plan names a different account uuid than the selected account
    /// (`None`: the selected account has no recorded uuid at all).
    WrongAccount {
        plan_uuid: String,
        selected_uuid: Option<String>,
    },
    /// The sync policy the plan derives from no longer exists.
    PolicyGone { plan_revision: i64 },
    /// The sync policy moved since the plan was compiled.
    PolicyMoved {
        plan_revision: i64,
        stored_revision: i64,
    },
    /// The policy row could not be read — an error, never "no policy".
    Annotations(AnnotationError),
}

impl std::fmt::Display for SpendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpendError::WrongProvider { plan, running } => write!(
                f,
                "the plan is for provider {plan:?}, but this frontend runs against {running:?}"
            ),
            SpendError::WrongAccount {
                plan_uuid,
                selected_uuid,
            } => write!(
                f,
                "the plan is for account uuid {plan_uuid}, but the selected account is {} — \
                 replan as the right account",
                selected_uuid.as_deref().unwrap_or("unmapped")
            ),
            SpendError::PolicyGone { plan_revision } => write!(
                f,
                "the sync policy is gone (the plan cites revision {plan_revision}); \
                 declare one and replan"
            ),
            SpendError::PolicyMoved {
                plan_revision,
                stored_revision,
            } => write!(
                f,
                "the sync policy moved: the plan cites revision {plan_revision}, but revision \
                 {stored_revision} is stored — review and replan"
            ),
            SpendError::Annotations(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SpendError {}

impl RefreshPlan {
    /// The apply-time gate, run frontend-side immediately before submit
    /// (the daemon is intent-blind, so only a frontend can compare):
    /// this plan may be spent only against the provider it names, the
    /// account uuid it was derived for, and the sync-policy revision it
    /// cites — checked against a fresh read of the stored row. The check
    /// races a concurrent policy write between read and submit; that is
    /// the accepted human-boundary residual, same register as an
    /// unconditional `policy set`.
    pub fn check_spendable(
        &self,
        provider: &str,
        selected_uuid: Option<&str>,
        annotations: &Annotations,
    ) -> Result<(), SpendError> {
        if self.provider != provider {
            return Err(SpendError::WrongProvider {
                plan: self.provider.clone(),
                running: provider.into(),
            });
        }
        if selected_uuid != Some(self.account_uuid.as_str()) {
            return Err(SpendError::WrongAccount {
                plan_uuid: self.account_uuid.clone(),
                selected_uuid: selected_uuid.map(String::from),
            });
        }
        match annotations
            .get(SYNC_POLICY_SCOPE, SYNC_POLICY_KEY, SYNC_POLICY_KIND)
            .map_err(SpendError::Annotations)?
        {
            None => Err(SpendError::PolicyGone {
                plan_revision: self.basis.policy_revision,
            }),
            Some(row) if row.revision != self.basis.policy_revision => {
                Err(SpendError::PolicyMoved {
                    plan_revision: self.basis.policy_revision,
                    stored_revision: row.revision,
                })
            }
            Some(_) => Ok(()),
        }
    }

    /// The plan's actions rendered as the `apply` parent's params — the
    /// explicit `(kind, params)` child tuples the daemon admits or
    /// refuses whole at submit, plus the caller's logical budget. One
    /// rendering for every frontend, so what apply submits cannot drift
    /// from what the plan authorized.
    pub fn apply_params(&self, max_requests: Option<u64>) -> Value {
        let jobs: Vec<Value> = self
            .actions
            .iter()
            .map(|action| {
                let (kind, params) = action.job();
                json!({ "kind": kind, "params": params })
            })
            .collect();
        let mut params = json!({ "jobs": jobs });
        if let Some(max) = max_requests {
            params["max_requests"] = json!(max);
        }
        params
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
    realm: Realm,
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
        if let Some(stray) = wire
            .actions
            .iter()
            .find(|a| a.realm() != wire.realm || a.league() != wire.league)
        {
            return Err(format!(
                "action for {}/{:?} inside a plan for {}/{:?}",
                stray.realm(),
                stray.league(),
                wire.realm,
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
        // must be *about* this plan — provider, account, and the plan's
        // own actions as its work basis.
        if let Some(quote) = &wire.quote {
            check_quote_matches(
                quote,
                &wire.provider,
                wire.account_name.as_deref(),
                &wire.actions,
            )?;
        }
        Ok(RefreshPlan {
            plan_schema: wire.plan_schema,
            operation: wire.operation,
            provider: wire.provider,
            account_uuid: wire.account_uuid,
            account_name: wire.account_name,
            realm: wire.realm,
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
    let realm = Realm::parse(&snapshot.realm).ok_or_else(|| PlanError::UnknownRealm {
        realm: snapshot.realm.clone(),
    })?;
    let league_policy = policy
        .realms
        .get(&realm)
        .and_then(|r| r.leagues.get(&snapshot.league))
        .ok_or_else(|| PlanError::LeagueNotCovered {
            realm,
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
                realm,
                league: snapshot.league.clone(),
                reason: ListingReason::NeverListed,
            });
            true
        }
        Some(basis) => {
            let age = now.saturating_sub(basis.fetched_at).max(0);
            if age > max_age {
                actions.push(RefreshAction::ListStashes {
                    realm,
                    league: snapshot.league.clone(),
                    reason: ListingReason::Stale { age_seconds: age },
                });
            }
            false
        }
    };
    let mut skipped = Vec::new();
    for tab in &snapshot.tabs {
        if !league_policy.tabs.covers(tab) {
            continue;
        }
        let verdict = if listing_alone {
            Err(SkipReason::AwaitingListing)
        } else if tab.r#type == "Folder" {
            Err(SkipReason::Folder)
        } else if is_empty_substash(snapshot, tab) {
            Err(SkipReason::EmptyStub)
        } else {
            fetch_verdict(tab, max_age, now)
                .and_then(|reason| fetch_action(realm, snapshot, tab, reason))
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
        realm,
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

/// A substash (its recorded parent is a map/unique tab on record) whose
/// stub counts 0 items while the store holds none at it. Only substash
/// stubs carry the count; a listing entry or a folder child never trips
/// this.
fn is_empty_substash(snapshot: &StashSnapshot, tab: &TabSnapshot) -> bool {
    let Some(parent) = tab.parent.as_deref() else {
        return false;
    };
    let parent_is_container = snapshot
        .tabs
        .iter()
        .any(|t| t.id == parent && matches!(t.r#type.as_str(), "MapStash" | "UniqueStash"));
    parent_is_container
        && tab.metadata.get("items").and_then(Value::as_i64) == Some(0)
        && tab.item_count == 0
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
    realm: Realm,
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
            realm,
            league: snapshot.league.clone(),
            parent: parent.clone(),
            id: tab.id.clone(),
            name: tab.name.clone(),
            tab_type: tab.r#type.clone(),
            reason,
        },
        None => RefreshAction::FetchTab {
            realm,
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
                realm: "pc".into(),
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
                realm: "pc".into(),
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
        s.stash_snapshot("pc", "Standard", &a).unwrap()
    }

    /// Snapshot with `policy` installed as the stored sync-policy row —
    /// the only way plans are made: from stored intent, at its revision.
    fn snapshot_with(s: &Store, policy: &Value) -> StashSnapshot {
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        a.put("account", "", SYNC_POLICY_KIND, policy, None)
            .unwrap();
        s.stash_snapshot("pc", "Standard", &a).unwrap()
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
        // A v1 policy reads as v2 on realm pc — the only realm it could
        // have meant — while the stored value stays what was written.
        assert_eq!(p.version, SYNC_POLICY_VERSION);
        let pc = &p.realms[&Realm::Pc].leagues;
        assert_eq!(pc["Standard"].tabs, TabSelection::All);
        assert_eq!(
            pc["Hardcore"].tabs,
            TabSelection::Ids(vec!["t1".into(), "s1".into()])
        );
        // The v2 shape: leagues under realms.
        let v2 = policy(json!({
            "version": 2,
            "realms": {
                "pc": { "leagues": { "Standard": { "tabs": "all", "max_age_seconds": 3600 } } },
                "xbox": { "leagues": { "Standard": { "tabs": ["x1"], "max_age_seconds": 60 } } },
            }
        }));
        assert_eq!(
            v2.realms[&Realm::Xbox].leagues["Standard"].tabs,
            TabSelection::Ids(vec!["x1".into()])
        );
        // Tabs under poe2 name a URL shape the stash endpoints do not
        // have (PoE1 only): a parse error, never a request.
        let err = SyncPolicy::from_value(&json!({
            "version": 2,
            "realms": { "poe2": { "leagues": { "Standard": { "tabs": "all", "max_age_seconds": 60 } } } }
        }))
        .unwrap_err();
        assert!(err.to_string().contains("poe2"), "{err}");
        // An unknown realm key, a v1 stamp on a v2 body, and a stray
        // top-level field under either stamp are all malformed — the
        // strict parse is per stamped shape, top level included.
        for bad in [
            json!({ "version": 2, "realms": { "ps5": { "leagues": {} } } }),
            json!({ "version": 1, "realms": { "pc": { "leagues": {} } } }),
            json!({ "version": 2, "leagues": {} }),
            json!({ "version": 1, "leagues": {}, "typo": true }),
            json!({ "version": 2, "realms": {}, "leagues": {} }),
        ] {
            let err = SyncPolicy::from_value(&bad).unwrap_err();
            assert!(
                matches!(err, PlanError::MalformedPolicy { .. }),
                "{bad}: {err}"
            );
        }
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
        // so a v3 policy with v3 fields is not misreported as a typo.
        let err = SyncPolicy::from_value(&json!({
            "version": 3,
            "realms": {},
            "some_v3_field": true
        }))
        .unwrap_err();
        assert_eq!(
            err,
            PlanError::PolicyVersionUnsupported {
                found: 3,
                supported: 2
            }
        );
        // The policy round-trips (in its v2 form): what a frontend writes
        // is inspectable.
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
                realm: Realm::Pc,
                league: "Standard".into()
            }
        );
        // Coverage is per (realm, league): Standard on xbox does not cover
        // Standard on pc.
        let xbox_only = json!({
            "version": 2,
            "realms": { "xbox": { "leagues": { "Standard": { "tabs": "all", "max_age_seconds": 60 } } } }
        });
        let err = plan_refresh("mock", &snapshot_with(&s, &xbox_only), 1000).unwrap_err();
        assert!(
            matches!(
                err,
                PlanError::LeagueNotCovered {
                    realm: Realm::Pc,
                    ..
                }
            ),
            "{err}"
        );
        // A stored policy row a newer build wrote surfaces its version,
        // and the version gate is not bypassable by deserializing the
        // type directly instead of calling from_value.
        let v3 = json!({ "version": 3, "realms": {} });
        let err = plan_refresh("mock", &snapshot_with(&s, &v3), 1000).unwrap_err();
        assert_eq!(
            err,
            PlanError::PolicyVersionUnsupported {
                found: 3,
                supported: 2
            }
        );
        let err = serde_json::from_value::<SyncPolicy>(v3).unwrap_err();
        assert!(err.to_string().contains("version 3"), "{err}");
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
                realm: Realm::Pc,
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
        let snap = s.stash_snapshot("pc", "Standard", &a).unwrap();
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
                    realm: Realm::Pc,
                    league: "Standard".into(),
                    reason: ListingReason::Stale { age_seconds: 4000 },
                },
                RefreshAction::FetchTab {
                    realm: Realm::Pc,
                    league: "Standard".into(),
                    id: "t1".into(),
                    name: "One".into(),
                    tab_type: "PremiumStash".into(),
                    reason: FetchReason::Stale { age_seconds: 3990 },
                },
                RefreshAction::FetchTab {
                    realm: Realm::Pc,
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
            &Endpoint::Stash { realm: "pc".into(),
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
                realm: Realm::Pc,
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
                realm: Realm::Pc,
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

    fn ids_policy_value(ids: &[&str], max_age_seconds: u32) -> Value {
        json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": ids, "max_age_seconds": max_age_seconds } }
        })
    }

    #[test]
    fn a_named_map_tab_covers_its_substashes_the_cycle_after_its_first_fetch() {
        let mut s = store();
        list(
            &mut s,
            json!([
                { "id": "m1", "name": "Maps", "type": "MapStash", "index": 0 },
                { "id": "t1", "name": "Other", "type": "PremiumStash", "index": 1 },
            ]),
            1000,
        );
        let policy = ids_policy_value(&["m1"], 3600);
        // Cycle 1: the parent alone — no stubs are on record yet, and a
        // plan never expands itself (binding). t1 is outside the policy.
        let plan1 = plan(&s, &policy, 1100);
        assert_eq!(
            plan1.actions,
            vec![RefreshAction::FetchTab {
                realm: Realm::Pc,
                league: "Standard".into(),
                id: "m1".into(),
                name: "Maps".into(),
                tab_type: "MapStash".into(),
                reason: FetchReason::NeverFetched,
            }]
        );
        // The parent's fetch lands two stubs: one with items, one empty.
        fetch(
            &mut s,
            json!({ "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
                { "id": "s1", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 2, "map": { "name": "Tier 16" } } },
                { "id": "s0", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 0, "map": { "name": "Tier 1" } } } ] }),
            1100,
        );
        // Cycle 2: the same policy now covers the substashes through their
        // parent; the empty stub is skipped with its reason, the parent is
        // fresh.
        let plan2 = plan(&s, &policy, 1200);
        assert_eq!(
            plan2.actions,
            vec![RefreshAction::FetchSubstash {
                realm: Realm::Pc,
                league: "Standard".into(),
                parent: "m1".into(),
                id: "s1".into(),
                name: "".into(),
                tab_type: "MapStash".into(),
                reason: FetchReason::NeverFetched,
            }]
        );
        assert_eq!(
            plan2.skipped,
            vec![
                SkippedTab {
                    id: "m1".into(),
                    name: "Maps".into(),
                    reason: SkipReason::Fresh { age_seconds: 100 },
                },
                SkippedTab {
                    id: "s0".into(),
                    name: "".into(),
                    reason: SkipReason::EmptyStub,
                },
            ]
        );
        // The new skip kind travels through the strict wire parse.
        let round = RefreshPlan::from_value(&serde_json::to_value(&plan2).unwrap()).unwrap();
        assert_eq!(round, plan2);
        // A child named directly is covered as before.
        let direct = plan(&s, &ids_policy_value(&["s1"], 3600), 1200);
        assert_eq!(direct.actions, plan2.actions);
        assert!(direct.skipped.is_empty());
        // An empty stub that nevertheless holds items (a stale 0 against a
        // real fetch) is not "empty": the disagreement arm decides.
        let mut s2 = store();
        list(
            &mut s2,
            json!([{ "id": "m1", "name": "Maps", "type": "MapStash", "index": 0 }]),
            1000,
        );
        fetch(
            &mut s2,
            json!({ "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
                { "id": "s0", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 1 } } ] }),
            1100,
        );
        s2.record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "m1".into(),
                sub: Some("s0".into()),
            },
            &json!({ "league": "Standard" }),
            200,
            &json!({ "stash": { "id": "s0", "name": "", "type": "MapStash", "parent": "m1",
                                "metadata": { "items": 1 }, "items": [item("x")] } }),
            1110,
        )
        .unwrap();
        fetch(
            &mut s2,
            json!({ "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
                { "id": "s0", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 0 } } ] }),
            1120,
        );
        let p = plan(&s2, &ids_policy_value(&["m1"], 3600), 1200);
        assert!(
            matches!(
                p.actions.as_slice(),
                [RefreshAction::FetchSubstash { id, reason: FetchReason::ListedCountDisagrees { listed: 0, held: 1 }, .. }] if id == "s0"
            ),
            "{:?}",
            p.actions
        );
    }

    #[test]
    fn a_named_folder_covers_its_children_at_once() {
        let mut s = store();
        list(
            &mut s,
            json!([
                { "id": "f1", "name": "Folder", "type": "Folder", "index": 0,
                  "children": [ { "id": "c1", "name": "In folder", "type": "PremiumStash", "index": 1 } ] },
                { "id": "t1", "name": "Other", "type": "PremiumStash", "index": 2 },
            ]),
            1000,
        );
        // Folder children are in the listing, so naming the folder plans
        // them in the same cycle; the folder itself is skipped as before
        // and t1 stays outside the policy.
        let plan = plan(&s, &ids_policy_value(&["f1"], 3600), 1100);
        assert_eq!(
            plan.actions,
            vec![RefreshAction::FetchTab {
                realm: Realm::Pc,
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
        assert!(plan.unknown_tabs.is_empty());
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
                realm: Realm::Pc,
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
            realm: Realm::Pc,
            league: "Standard".into(),
            reason: ListingReason::NeverListed,
        };
        let (kind, params) = listing.job();
        assert_eq!(
            Endpoint::from_job(kind, &params),
            Some(Endpoint::Stashes {
                realm: "pc".into(),
                league: "Standard".into()
            })
        );
        let tab = RefreshAction::FetchTab {
            realm: Realm::Pc,
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
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t1".into(),
                sub: None,
            })
        );
        // `deep` is not part of the endpoint; assert it separately — a
        // plan's fetch must never fan out (D5a).
        assert_eq!(params["deep"], json!(false));
        let substash = RefreshAction::FetchSubstash {
            realm: Realm::Pc,
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
                realm: "pc".into(),
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
        use acquisition_core::protocol::{QuoteJob, QuoteScope};
        let mut s = store();
        list(
            &mut s,
            json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]),
            1000,
        );
        let bare = plan(&s, &all_policy_value(60), 5000);
        assert_eq!(bare.quote, None, "compiling never fills the quote");
        assert_eq!(bare.logical_requests, 2, "{:?}", bare.actions);
        assert!(bare.account_name.is_some(), "the plan is account-bound");
        // The quote's verifiable work basis: exactly the plan's actions
        // rendered as job tuples, in order.
        let work: Vec<QuoteJob> = bare
            .actions
            .iter()
            .map(|a| {
                let (kind, params) = a.job();
                QuoteJob {
                    kind: kind.into(),
                    params,
                }
            })
            .collect();
        let quote = Quote {
            observed_at: 5000,
            provider: bare.provider.clone(),
            account: bare.account_name.clone(),
            halted: None,
            work: work.clone(),
            scopes: vec![QuoteScope {
                key: "stash-list".into(),
                endpoints: vec!["stash-list".into(), "stash".into()],
                requests: 2,
                queued_ahead: 0,
                policy: None,
                rules: Vec::new(),
                observed_seconds_ago: None,
                eta_seconds: None,
                notes: vec!["policy not yet learned".into()],
            }],
            not_covered: vec!["a HEAD probe before the first request (N16)".into()],
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
        // An accountless quote on an account-bound plan refuses too: it
        // projects someone else's limiter state (`null` is not a wildcard).
        let mut accountless = quote.clone();
        accountless.account = None;
        assert!(bare.clone().with_quote(accountless).is_err());
        // A quote that projects fewer requests than the plan authorizes
        // cannot dress the plan up — at attach and at parse alike.
        let mut partial = quote.clone();
        partial.scopes[0].requests = 1;
        let err = bare.clone().with_quote(partial).unwrap_err();
        assert!(err.to_string().contains("authorizes 2"), "{err}");
        let mut shrunk = json.clone();
        shrunk["quote"]["scopes"][0]["requests"] = json!(1);
        let err = RefreshPlan::from_value(&shrunk).unwrap_err();
        assert!(err.to_string().contains("authorizes 2"), "{err}");
        // Equal counts are not coverage: a quote for two *other* jobs of
        // the same size — or the right jobs in a different order — is not
        // a quote for this plan.
        let mut foreign_work = quote.clone();
        foreign_work.work = vec![
            QuoteJob {
                kind: "fetch".into(),
                params: json!({}),
            },
            QuoteJob {
                kind: "fetch".into(),
                params: json!({}),
            },
        ];
        let err = bare.clone().with_quote(foreign_work).unwrap_err();
        assert!(err.to_string().contains("not the plan's action"), "{err}");
        let mut reordered = quote.clone();
        reordered.work.swap(0, 1);
        assert!(bare.clone().with_quote(reordered).is_err());
        let mut swapped = json.clone();
        swapped["quote"]["work"] = serde_json::to_value([&work[1], &work[0]]).unwrap();
        assert!(RefreshPlan::from_value(&swapped).is_err());
        // Scope totals that overflow are a structured refusal, never a
        // wrap past validation or a panic (the no-panic rule covers a
        // malformed envelope too).
        let mut overflowing = quote;
        let extra = QuoteScope {
            requests: u64::MAX,
            ..overflowing.scopes[0].clone()
        };
        overflowing.scopes.push(extra);
        let err = bare.with_quote(overflowing).unwrap_err();
        assert!(err.to_string().contains("overflows"), "{err}");
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
        newer["plan_schema"] = json!(REFRESH_PLAN_SCHEMA + 1);
        assert_eq!(
            RefreshPlan::from_value(&newer).unwrap_err(),
            PlanError::PlanSchemaUnsupported {
                found: REFRESH_PLAN_SCHEMA + 1,
                supported: REFRESH_PLAN_SCHEMA
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
        // …and an action for another realm: the envelope's realm binds
        // every action, so a pc plan cannot smuggle a console fetch.
        let mut stray = good.clone();
        stray["actions"][0]["realm"] = json!("xbox");
        let err = RefreshPlan::from_value(&stray).unwrap_err();
        assert!(err.to_string().contains("xbox"), "{err}");
        let mut unknown = good.clone();
        unknown["realm"] = json!("ps5");
        assert!(RefreshPlan::from_value(&unknown).is_err());
    }

    #[test]
    fn put_sync_policy_validates_before_it_stores_and_is_a_cas() {
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        // A typo'd value is refused by the strict parse and nothing lands:
        // intent is never half-honored, on any frontend's write surface.
        let typo = json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": "all", "max_age_secs": 60 } }
        });
        let err = put_sync_policy(&mut a, &typo, None).unwrap_err();
        assert!(matches!(err, PutPolicyError::Invalid(_)), "{err}");
        assert!(a.get("account", "", SYNC_POLICY_KIND).unwrap().is_none());
        // `None` creates; a second `None` over a live row is a conflict
        // naming the current revision — a caller (an agent especially)
        // never replaces intent it has not read.
        let value = all_policy_value(3600);
        let first = put_sync_policy(&mut a, &value, None).unwrap();
        assert_eq!(first.revision, 1);
        let err = put_sync_policy(&mut a, &value, None).unwrap_err();
        assert!(matches!(err, PutPolicyError::Store(_)), "{err}");
        assert!(err.to_string().contains("revision 1"), "{err}");
        // Naming the reviewed revision replaces it; naming a stale one
        // conflicts and the stored value is untouched.
        let second = put_sync_policy(&mut a, &value, Some(1)).unwrap();
        assert_eq!(second.revision, 2);
        let err = put_sync_policy(&mut a, &all_policy_value(60), Some(1)).unwrap_err();
        assert!(err.to_string().contains("revision 2"), "{err}");
        let held = a.get("account", "", SYNC_POLICY_KIND).unwrap().unwrap();
        assert_eq!(held.value, value);
    }

    #[test]
    fn check_spendable_is_the_shared_staleness_and_identity_gate() {
        // The plan derives from a rev-1 policy for account u-1 on mock.
        let plan = plan(&store(), &all_policy_value(3600), 2_000);
        assert_eq!(plan.basis.policy_revision, 1);
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        // Intent gone since the plan: refused, citing the derivation.
        let err = plan.check_spendable("mock", Some("u-1"), &a).unwrap_err();
        assert!(matches!(err, SpendError::PolicyGone { .. }), "{err}");
        // Intent standing at the plan's revision: spendable.
        put_sync_policy(&mut a, &all_policy_value(3600), None).unwrap();
        plan.check_spendable("mock", Some("u-1"), &a).unwrap();
        // Intent moved since the plan (the step-7 ruling): refused with
        // both revisions named, remedy = replan.
        put_sync_policy(&mut a, &all_policy_value(60), Some(1)).unwrap();
        let err = plan.check_spendable("mock", Some("u-1"), &a).unwrap_err();
        assert!(matches!(err, SpendError::PolicyMoved { .. }), "{err}");
        let msg = err.to_string();
        assert!(
            msg.contains("revision 1") && msg.contains("revision 2") && msg.contains("replan"),
            "{msg}"
        );
        // A plan for another identity is never spent: wrong uuid (or no
        // uuid at all), wrong provider.
        let err = plan
            .check_spendable("mock", Some("u-other"), &a)
            .unwrap_err();
        assert!(matches!(err, SpendError::WrongAccount { .. }), "{err}");
        let err = plan.check_spendable("mock", None, &a).unwrap_err();
        assert!(err.to_string().contains("unmapped"), "{err}");
        let err = plan.check_spendable("ggg", Some("u-1"), &a).unwrap_err();
        assert!(matches!(err, SpendError::WrongProvider { .. }), "{err}");
    }

    #[test]
    fn apply_params_renders_exactly_the_plans_actions() {
        let plan = plan(&store(), &all_policy_value(3600), 2_000);
        assert!(!plan.actions.is_empty());
        let params = plan.apply_params(None);
        let jobs = params["jobs"].as_array().unwrap();
        assert_eq!(jobs.len(), plan.actions.len());
        for (job, action) in jobs.iter().zip(&plan.actions) {
            let (kind, action_params) = action.job();
            assert_eq!(job["kind"], json!(kind));
            assert_eq!(job["params"], action_params);
        }
        assert!(params.get("max_requests").is_none());
        assert_eq!(plan.apply_params(Some(3))["max_requests"], json!(3));
    }

    /// Realm rides through: a plan for Standard on xbox lists and fetches
    /// under xbox and its tuples say so explicitly; the pc snapshot of
    /// the same league is not covered by an xbox-only policy.
    #[test]
    fn a_plan_is_for_one_realm_and_its_tuples_say_which() {
        let mut s = store();
        s.record(
            &Endpoint::Stashes {
                realm: "xbox".into(),
                league: "Standard".into(),
            },
            &json!({ "realm": "xbox", "league": "Standard" }),
            200,
            &json!({ "stashes": [ { "id": "x1", "name": "Console", "type": "PremiumStash", "index": 0 } ] }),
            1000,
        )
        .unwrap();
        let policy = json!({
            "version": 2,
            "realms": { "xbox": { "leagues": { "Standard": { "tabs": "all", "max_age_seconds": 60 } } } }
        });
        let mut a = Annotations::open_memory_for("u-1").unwrap();
        a.put("account", "", SYNC_POLICY_KIND, &policy, None)
            .unwrap();
        let snap = s.stash_snapshot("xbox", "Standard", &a).unwrap();
        let plan = plan_refresh("mock", &snap, 1500).unwrap();
        assert_eq!(plan.realm, Realm::Xbox);
        let jobs: Vec<(String, Value)> = plan
            .actions
            .iter()
            .map(|a| {
                let (k, p) = a.job();
                (k.into(), p)
            })
            .collect();
        assert_eq!(
            jobs,
            vec![
                (
                    "stashes".into(),
                    json!({ "realm": "xbox", "league": "Standard" })
                ),
                (
                    "stash".into(),
                    json!({ "realm": "xbox", "league": "Standard", "id": "x1", "deep": false })
                ),
            ]
        );
        assert_eq!(
            Endpoint::from_job(&jobs[1].0, &jobs[1].1),
            Some(Endpoint::Stash {
                realm: "xbox".into(),
                league: "Standard".into(),
                id: "x1".into(),
                sub: None
            })
        );
        let pc = s.stash_snapshot("pc", "Standard", &a).unwrap();
        assert!(matches!(
            plan_refresh("mock", &pc, 1500).unwrap_err(),
            PlanError::LeagueNotCovered {
                realm: Realm::Pc,
                ..
            }
        ));
        // The envelope round-trips with its realm.
        let json = serde_json::to_value(&plan).unwrap();
        assert_eq!(json["realm"], "xbox");
        assert_eq!(RefreshPlan::from_value(&json).unwrap(), plan);
    }

    /// A tab a listing dropped and a later listing revived has no live
    /// facts (they were retired with it) and no fetch on record, so the
    /// plan fetches it again — the loop, not a manual step, restores it.
    #[test]
    fn a_revived_tab_is_planned_as_never_fetched() {
        let mut s = store();
        let t1 = json!([{ "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 }]);
        list(&mut s, t1.clone(), 1000);
        fetch(
            &mut s,
            json!({ "id": "t1", "name": "One", "type": "PremiumStash", "items": [item("i1")] }),
            1100,
        );
        list(&mut s, json!([]), 1200);
        list(&mut s, t1, 1300);
        let plan = plan(&s, &all_policy_value(3600), 1400);
        assert!(
            matches!(
                plan.actions[..],
                [RefreshAction::FetchTab { ref id, reason: FetchReason::NeverFetched, .. }] if id == "t1"
            ),
            "{:?}",
            plan.actions
        );
    }

    /// A map tab a listing dropped and a later listing revived: its
    /// substashes kept their rows (orphaned, then re-parented) but lost
    /// their facts with the parent, so the plan fetches the parent *and*
    /// each substash again — nothing waits out a freshness window empty.
    #[test]
    fn a_revived_parent_replans_its_substashes() {
        let mut s = store();
        let m1 = json!([{ "id": "m1", "name": "Maps", "type": "MapStash", "index": 0 }]);
        list(&mut s, m1.clone(), 1000);
        fetch(
            &mut s,
            json!({ "id": "m1", "name": "Maps", "type": "MapStash", "items": [],
                "children": [ { "id": "s1", "name": "", "type": "MapStash", "parent": "m1", "metadata": { "items": 1 } } ] }),
            1100,
        );
        s.record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "m1".into(),
                sub: Some("s1".into()),
            },
            &json!({ "league": "Standard", "id": "m1", "sub": "s1" }),
            200,
            &json!({ "stash": { "id": "s1", "name": "", "type": "MapStash", "parent": "m1", "items": [item("map1")] } }),
            1200,
        )
        .unwrap();
        list(&mut s, json!([]), 1300);
        list(&mut s, m1, 1400);
        let plan = plan(&s, &all_policy_value(3600), 1500);
        let mut kinds: Vec<(String, String)> = plan
            .actions
            .iter()
            .map(|a| match a {
                RefreshAction::FetchTab { id, reason, .. } => {
                    (format!("tab {id}"), format!("{reason:?}"))
                }
                RefreshAction::FetchSubstash { id, reason, .. } => {
                    (format!("sub {id}"), format!("{reason:?}"))
                }
                RefreshAction::ListStashes { .. } => ("list".into(), String::new()),
            })
            .collect();
        kinds.sort();
        assert_eq!(
            kinds,
            vec![
                ("sub s1".to_string(), "NeverFetched".to_string()),
                ("tab m1".to_string(), "NeverFetched".to_string()),
            ],
            "{:?}",
            plan.actions
        );
    }
}
