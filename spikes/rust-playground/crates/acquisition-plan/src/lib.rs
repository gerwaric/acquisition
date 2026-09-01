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
        }
    }
}

impl std::error::Error for PlanError {}

/// The per-account sync policy: an inspectable declaration of desired
/// coverage and freshness — not a scheduler. Stored as the
/// `("account", "", "sync-policy")` annotation; written by frontends,
/// compiled here into minimal requests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyncPolicy {
    /// [`SYNC_POLICY_VERSION`]; a newer stamp is refused at parse.
    pub version: i64,
    /// Coverage per league. A league not named here compiles to
    /// [`PlanError::LeagueNotCovered`], never to implicit work.
    pub leagues: BTreeMap<String, LeaguePolicy>,
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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ListingReason {
    NeverListed,
    Stale { age_seconds: i64 },
}

/// Why the plan fetches a tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SkipReason {
    /// Fetched within the policy's window and nothing proved a change.
    Fresh { age_seconds: i64 },
    /// Folders hold tabs, never items, and are never fetched — their
    /// children are covered individually.
    Folder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedTab {
    pub id: String,
    pub name: String,
    pub reason: SkipReason,
}

/// One authorized request. Self-contained on purpose: an action can be
/// rendered, reviewed, or turned into a daemon job without the envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
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
pub struct PlanBasis {
    /// When the snapshot was taken (facts as of this read).
    pub snapshot_taken_at: i64,
    /// The listing the tab set derives from; `None` when the league was
    /// never listed (which is itself the first action).
    pub listing: Option<ListingBasis>,
    /// The sync-policy annotation revision compiled from; `None` when the
    /// policy was supplied ad hoc rather than read from the store.
    pub policy_revision: Option<i64>,
}

/// The coarse wire projection. The range covers the authorized requests
/// and their bounded 429 retries; `prerequisites` names sends the range
/// deliberately does not count — a precise accounting is the deferred
/// wire-budget feature, and pretending to one here would be false.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireEstimate {
    pub min: u64,
    pub max: u64,
    pub prerequisites: Vec<String>,
}

/// A refresh authorization: the bounded work the user reviewed, derived
/// from one snapshot of facts + intent. Immutable by convention — new
/// facts produce a new plan, never an edit — and serializable so it can
/// cross a socket, land in a journal, or be read back by apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
}

/// Compile the snapshot's own sync-policy row into a plan. This is the
/// normal path: the annotation basis is the row's revision.
pub fn plan_refresh(
    provider: &str,
    snapshot: &StashSnapshot,
    now: i64,
) -> Result<RefreshPlan, PlanError> {
    let row = snapshot.policy.as_ref().ok_or(PlanError::NoSyncPolicy)?;
    let policy = SyncPolicy::from_value(&row.value)?;
    compile_refresh(provider, snapshot, &policy, Some(row.revision), now)
}

/// Compile an explicit policy against a snapshot — the ad-hoc path for a
/// frontend that builds its selection in hand (`policy_revision: None`
/// records that no stored intent was cited).
pub fn compile_refresh(
    provider: &str,
    snapshot: &StashSnapshot,
    policy: &SyncPolicy,
    policy_revision: Option<i64>,
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
    match &snapshot.listing {
        None => actions.push(RefreshAction::ListStashes {
            league: snapshot.league.clone(),
            reason: ListingReason::NeverListed,
        }),
        Some(basis) => {
            let age = (now - basis.fetched_at).max(0);
            if age > max_age {
                actions.push(RefreshAction::ListStashes {
                    league: snapshot.league.clone(),
                    reason: ListingReason::Stale { age_seconds: age },
                });
            }
        }
    }
    let mut skipped = Vec::new();
    for tab in &snapshot.tabs {
        if !league_policy.tabs.selects(&tab.id) {
            continue;
        }
        if tab.r#type == "Folder" {
            skipped.push(SkippedTab {
                id: tab.id.clone(),
                name: tab.name.clone(),
                reason: SkipReason::Folder,
            });
            continue;
        }
        match fetch_verdict(tab, max_age, now) {
            Ok(reason) => actions.push(fetch_action(snapshot, tab, reason)),
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
    let wire_sends = WireEstimate {
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
    };
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
    let age = (now - fetched_at).max(0);
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
/// map/unique tab on record (a substash), by its own id otherwise (a
/// top-level tab or a folder child — folders group, they don't contain).
fn fetch_action(snapshot: &StashSnapshot, tab: &TabSnapshot, reason: FetchReason) -> RefreshAction {
    let substash_parent = tab.parent.as_deref().filter(|p| {
        snapshot
            .tabs
            .iter()
            .any(|t| &t.id == p && matches!(t.r#type.as_str(), "MapStash" | "UniqueStash"))
    });
    match substash_parent {
        Some(parent) => RefreshAction::FetchSubstash {
            league: snapshot.league.clone(),
            parent: parent.into(),
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
    }
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

    fn all_policy(max_age_seconds: u32) -> SyncPolicy {
        policy(json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": "all", "max_age_seconds": max_age_seconds } }
        }))
    }

    fn snapshot(s: &Store) -> StashSnapshot {
        let a = Annotations::open_memory_for("u-1").unwrap();
        s.stash_snapshot("Standard", &a).unwrap()
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
        let p = policy(json!({
            "version": 1,
            "leagues": { "Hardcore": { "tabs": "all", "max_age_seconds": 60 } }
        }));
        let err = compile_refresh("mock", &snapshot(&s), &p, None, 1000).unwrap_err();
        assert_eq!(
            err,
            PlanError::LeagueNotCovered {
                league: "Standard".into()
            }
        );
    }

    #[test]
    fn a_never_listed_league_plans_the_listing_and_only_known_facts() {
        let s = store();
        let plan = compile_refresh("mock", &snapshot(&s), &all_policy(3600), None, 1000).unwrap();
        // No invention: the one action is the listing; tabs wait for the
        // facts it will land (D5a's eventual reconciliation).
        assert_eq!(
            plan.actions,
            vec![RefreshAction::ListStashes {
                league: "Standard".into(),
                reason: ListingReason::NeverListed,
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
        assert_eq!(plan.basis.policy_revision, Some(row.revision));
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
        let plan = compile_refresh("mock", &snapshot(&s), &all_policy(3600), None, 5000).unwrap();
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
        let plan = compile_refresh("mock", &snapshot(&s), &all_policy(3600), None, 1100).unwrap();
        assert!(plan.actions.is_empty(), "{:?}", plan.actions);
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
        let plan = compile_refresh("mock", &snapshot(&s), &all_policy(3600), None, 1100).unwrap();
        assert_eq!(
            plan.actions,
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
        let plan = compile_refresh("mock", &snapshot(&s), &all_policy(3600), None, 9000).unwrap();
        assert!(plan.actions.iter().any(|a| matches!(
            a,
            RefreshAction::FetchSubstash { id, reason: FetchReason::Stale { .. }, .. } if id == "s1"
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
        let plan = compile_refresh("mock", &snapshot(&s), &all_policy(3600), None, 1100).unwrap();
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
        let p = policy(json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": ["t1", "ghost"], "max_age_seconds": 3600 } }
        }));
        let plan = compile_refresh("mock", &snapshot(&s), &p, None, 1100).unwrap();
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
    fn actions_speak_the_daemons_job_vocabulary() {
        // These tuples are exactly what the daemon's own code submits —
        // pinned here so the plan and the dispatcher cannot drift apart.
        let listing = RefreshAction::ListStashes {
            league: "Standard".into(),
            reason: ListingReason::NeverListed,
        };
        assert_eq!(listing.job(), ("stashes", json!({ "league": "Standard" })));
        let tab = RefreshAction::FetchTab {
            league: "Standard".into(),
            id: "t1".into(),
            name: "One".into(),
            tab_type: "PremiumStash".into(),
            reason: FetchReason::NeverFetched,
        };
        assert_eq!(
            tab.job(),
            (
                "stash",
                json!({ "league": "Standard", "id": "t1", "deep": false })
            )
        );
        let substash = RefreshAction::FetchSubstash {
            league: "Standard".into(),
            parent: "m1".into(),
            id: "s1".into(),
            name: "".into(),
            tab_type: "MapStash".into(),
            reason: FetchReason::NeverFetched,
        };
        assert_eq!(
            substash.job(),
            (
                "stash",
                json!({ "league": "Standard", "id": "m1", "sub": "s1", "deep": false })
            )
        );
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
        let p = policy(json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": ["t1", "f1", "c1", "ghost"], "max_age_seconds": 60 } }
        }));
        let plan = compile_refresh("mock", &snapshot(&s), &p, Some(7), 5000).unwrap();
        // A plan crosses sockets and lands in journals; serialization is
        // part of its contract, not a debug convenience.
        let json = serde_json::to_value(&plan).unwrap();
        let back: RefreshPlan = serde_json::from_value(json).unwrap();
        assert_eq!(back, plan);
    }
}
