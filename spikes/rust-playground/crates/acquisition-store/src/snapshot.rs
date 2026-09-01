//! Neutral snapshots: the read the planner (`acquisition-plan`) compiles
//! plans from. A snapshot names facts and intent together — the listing
//! basis a plan cites, tab identities with their freshness and listed
//! metadata, and the sync-policy annotation row at its revision — and
//! carries nothing derived: no staleness verdicts, no request lists.
//! Policy compilation lives in `acquisition-plan`, never here — the store
//! exposes neutral snapshots, "never half a planner" (CONTEXT.md, decided
//! 2026-08-31).

use anyhow::Result;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::annotations::{AnnotationRow, Annotations};
use crate::{Store, TAB_ORDER_SQL};

/// The per-account sync policy's annotation address (scope `"account"`,
/// key `""`): the declaration of desired coverage and freshness. Its
/// value's shape is the planner's business; the store only carries it.
pub const SYNC_POLICY_KIND: &str = "sync-policy";

/// The stash listing this snapshot's tab set derives from — the fact
/// basis a plan cites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListingBasis {
    /// Row id in `responses`, stable for the life of the file.
    pub response_id: i64,
    pub fetched_at: i64,
}

/// One tab as the planner sees it: identity and freshness, plus what the
/// server said about it. Substash identity is `(parent, id)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabSnapshot {
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub r#type: String,
    pub idx: Option<i64>,
    pub listed_at: Option<i64>,
    pub fetched_at: Option<i64>,
    /// The tab's `metadata` as GGG sent it (colour, map name, the `items`
    /// count on substash stubs — heuristic evidence: it can prove a tab
    /// changed, never that it didn't); `Null` when absent.
    pub metadata: Value,
    /// Live items this store holds at the tab — what fetches produced, as
    /// opposed to what the listing promised.
    pub item_count: i64,
}

/// A named snapshot of one league's stash facts plus the account's sync
/// policy, taken with no daemon involved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashSnapshot {
    pub league: String,
    pub taken_at: i64,
    /// `None`: this league was never listed (tabs may still exist from
    /// direct fetches). A plan that needs a listing says so; the snapshot
    /// does not invent one.
    pub listing: Option<ListingBasis>,
    /// Live tabs in listing order (same order as [`Store::tabs`]).
    pub tabs: Vec<TabSnapshot>,
    /// The sync-policy annotation at its revision — the annotation basis
    /// a plan cites. `None`: no annotations handle given, or no policy.
    pub policy: Option<AnnotationRow>,
}

impl Store {
    /// Snapshot one league's stash facts, bundling the sync-policy row
    /// when an annotations handle is given, so a plan's fact basis and
    /// annotation revision come from one read.
    pub fn stash_snapshot(
        &self,
        league: &str,
        annotations: Option<&Annotations>,
    ) -> Result<StashSnapshot> {
        // The league of a listing lives in its params; an omitted league
        // defaulted to "Standard" at record time (`Endpoint::from_job`),
        // so the match here defaults the same way.
        let listing = self
            .conn
            .query_row(
                "SELECT id, fetched_at FROM responses
                  WHERE endpoint = 'stashes' AND status BETWEEN 200 AND 299
                    AND COALESCE(json_extract(params, '$.league'), 'Standard') = ?1
                  ORDER BY id DESC LIMIT 1",
                [league],
                |r| {
                    Ok(ListingBasis {
                        response_id: r.get(0)?,
                        fetched_at: r.get(1)?,
                    })
                },
            )
            .optional()?;
        let mut stmt = self.conn.prepare(&format!(
            "SELECT t.id, t.parent, COALESCE(t.name, ''), COALESCE(t.type, ''), t.idx, t.listed_at, t.fetched_at, t.json,
                    (SELECT count(*) FROM items i WHERE i.location_kind = 'stash' AND i.location_id = t.id AND i.removed_at IS NULL)
               FROM tabs t WHERE t.league = ?1 AND t.removed_at IS NULL {TAB_ORDER_SQL}"
        ))?;
        let tabs = stmt
            .query_map([league], |r| {
                let json: String = r.get(7)?;
                Ok(TabSnapshot {
                    id: r.get(0)?,
                    parent: r.get(1)?,
                    name: r.get(2)?,
                    r#type: r.get(3)?,
                    idx: r.get(4)?,
                    listed_at: r.get(5)?,
                    fetched_at: r.get(6)?,
                    metadata: serde_json::from_str::<Value>(&json)
                        .ok()
                        .and_then(|v| v.get("metadata").cloned())
                        .unwrap_or(Value::Null),
                    item_count: r.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let policy = match annotations {
            Some(a) => a.get("account", "", SYNC_POLICY_KIND)?,
            None => None,
        };
        Ok(StashSnapshot {
            league: league.into(),
            taken_at: crate::now(),
            listing,
            tabs,
            policy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Endpoint;
    use serde_json::json;

    fn listing_ep() -> Endpoint {
        Endpoint::Stashes {
            league: "Standard".into(),
        }
    }

    fn stash_ep(id: &str, sub: Option<&str>) -> Endpoint {
        Endpoint::Stash {
            league: "Standard".into(),
            id: id.into(),
            sub: sub.map(str::to_string),
        }
    }

    fn item(id: &str) -> Value {
        json!({ "id": id, "name": "Foo", "typeLine": "Imperial Bow", "baseType": "Imperial Bow", "x": 0, "y": 0 })
    }

    #[test]
    fn the_snapshot_names_the_latest_listing_and_the_policy_revision() {
        let mut s = Store::open_memory().unwrap();
        let mut a = Annotations::open_memory().unwrap();
        // A direct `stashes` job may record with no league in its params;
        // that listing defaulted to Standard and must be found as such.
        s.record(
            &listing_ep(),
            &json!({}),
            200,
            &json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] }),
            100,
        )
        .unwrap();
        let first = s.stash_snapshot("Standard", Some(&a)).unwrap();
        let basis = first.listing.unwrap();
        assert_eq!(basis.fetched_at, 100);
        assert!(first.policy.is_none());
        // The refresh parent records the same listing with normalized
        // params; a later listing replaces the basis.
        s.record(
            &listing_ep(),
            &json!({ "league": "Standard" }),
            200,
            &json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] }),
            200,
        )
        .unwrap();
        let policy = a
            .put(
                "account",
                "",
                SYNC_POLICY_KIND,
                &json!({ "leagues": ["Standard"], "deep": false }),
                None,
            )
            .unwrap();
        let snap = s.stash_snapshot("Standard", Some(&a)).unwrap();
        let later = snap.listing.unwrap();
        assert!(later.response_id > basis.response_id);
        assert_eq!(later.fetched_at, 200);
        let row = snap.policy.unwrap();
        assert_eq!((row.revision, &row.value), (1, &policy.value));
        // A tombstoned policy is no policy — but its revision still gates
        // the next write, which is the annotation layer's business.
        a.delete("account", "", SYNC_POLICY_KIND, 1).unwrap();
        assert!(
            s.stash_snapshot("Standard", Some(&a))
                .unwrap()
                .policy
                .is_none()
        );
    }

    #[test]
    fn the_basis_is_per_league_and_absent_when_never_listed() {
        let mut s = Store::open_memory().unwrap();
        s.record(
            &Endpoint::Stashes {
                league: "Hardcore".into(),
            },
            &json!({ "league": "Hardcore" }),
            200,
            &json!({ "stashes": [ { "id": "h1", "name": "HC", "type": "PremiumStash", "index": 0 } ] }),
            50,
        )
        .unwrap();
        // A tab fetched directly, with no listing for its league: it is in
        // the snapshot (fetched, never listed) and the basis stays None.
        s.record(
            &stash_ep("x1", None),
            &json!({ "league": "Standard", "id": "x1" }),
            200,
            &json!({ "stash": { "id": "x1", "name": "Fetched", "type": "PremiumStash", "items": [ item("i1") ] } }),
            60,
        )
        .unwrap();
        let std = s.stash_snapshot("Standard", None).unwrap();
        assert!(std.listing.is_none());
        assert_eq!(std.tabs.len(), 1);
        let x1 = &std.tabs[0];
        assert_eq!(
            (x1.id.as_str(), x1.listed_at, x1.fetched_at),
            ("x1", None, Some(60))
        );
        assert_eq!(x1.item_count, 1);
        let hc = s.stash_snapshot("Hardcore", None).unwrap();
        assert_eq!(hc.listing.unwrap().fetched_at, 50);
        assert_eq!(hc.tabs.len(), 1);
        assert_eq!(hc.tabs[0].id, "h1");
    }

    #[test]
    fn metadata_rides_verbatim_and_removed_tabs_leave_the_snapshot() {
        let mut s = Store::open_memory().unwrap();
        s.record(
            &listing_ep(),
            &json!({}),
            200,
            &json!({ "stashes": [
                { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0, "metadata": { "colour": "7c5436" } },
                { "id": "m1", "name": "Maps", "type": "MapStash", "index": 1 },
                { "id": "gone", "name": "Old", "type": "PremiumStash", "index": 2 },
            ]}),
            100,
        )
        .unwrap();
        // Fetching the map tab lists a substash stub whose metadata carries
        // the heuristic `items` count and the map name.
        s.record(
            &stash_ep("m1", None),
            &json!({ "league": "Standard", "id": "m1" }),
            200,
            &json!({ "stash": { "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
                { "id": "s1", "name": "", "type": "MapStash", "parent": "m1",
                  "metadata": { "items": 1, "map": { "name": "Tier 16" } } } ] } }),
            110,
        )
        .unwrap();
        let snap = s.stash_snapshot("Standard", None).unwrap();
        assert_eq!(
            snap.tabs.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
            vec!["t1", "m1", "s1", "gone"]
        );
        let t1 = &snap.tabs[0];
        assert_eq!(t1.metadata, json!({ "colour": "7c5436" }));
        assert_eq!((t1.listed_at, t1.fetched_at), (Some(100), None));
        let s1 = snap.tabs.iter().find(|t| t.id == "s1").unwrap();
        assert_eq!(s1.parent.as_deref(), Some("m1"));
        assert_eq!(s1.metadata["items"], 1);
        assert_eq!(s1.metadata["map"]["name"], "Tier 16");
        // The next listing drops "gone": no longer coverage, so not in the
        // snapshot — while the never-listed substash survives.
        s.record(
            &listing_ep(),
            &json!({}),
            200,
            &json!({ "stashes": [
                { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0, "metadata": { "colour": "7c5436" } },
                { "id": "m1", "name": "Maps", "type": "MapStash", "index": 1 },
            ]}),
            200,
        )
        .unwrap();
        let ids: Vec<String> = s
            .stash_snapshot("Standard", None)
            .unwrap()
            .tabs
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["t1", "m1", "s1"]);
    }
}
