//! Neutral snapshots: the read the planner (`acquisition-plan`) compiles
//! plans from. A snapshot names facts and intent together — the listing
//! basis a plan cites, tab identities with their freshness and listed
//! metadata, the sync-policy annotation row at its revision, and the
//! account uuid the pairing is bound to — and carries nothing derived:
//! no staleness verdicts, no request lists. Policy compilation lives in
//! `acquisition-plan`, never here — the store exposes neutral snapshots,
//! "never half a planner" (CONTEXT.md, decided 2026-08-31).

use anyhow::{Context, Result, bail};
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
    /// `responses.id` of the listing (or, for a substash stub, the parent
    /// fetch) that last listed this tab. After a listing, every top-level
    /// tab and folder child carries the basis `response_id` — which is
    /// what lets a plan check its tab set against the basis it cites.
    /// `None`: fetched directly, never listed.
    pub listed_response: Option<i64>,
    pub fetched_at: Option<i64>,
    /// The tab's `metadata` from the listing entry / substash stub,
    /// verbatim (colour, map name, the heuristic `items` count — evidence
    /// that can prove a tab changed, never that it didn't). A fetch never
    /// overwrites it; `Null` when never listed or the entry had none.
    pub metadata: Value,
    /// Live items this store holds at the tab — what fetches produced, as
    /// opposed to what the listing promised.
    pub item_count: i64,
}

/// A named snapshot of one league's stash facts plus the account's sync
/// policy, taken with no daemon involved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StashSnapshot {
    /// The account uuid the facts file records (`/profile` lands at every
    /// login) — the identity annotation files carry internally, so a plan
    /// can cite it. Facts and intent are paired under this one uuid;
    /// [`Store::stash_snapshot`] refuses an annotations handle whose
    /// stored uuid differs or is absent.
    /// The provider is not here: the store cannot verify it — the caller
    /// binds it by the provider directory it opened.
    pub account_uuid: String,
    /// Display name beside the uuid, when the profile carried one.
    pub account_name: Option<String>,
    pub league: String,
    pub taken_at: i64,
    /// `None`: this league was never listed (tabs may still exist from
    /// direct fetches). A plan that needs a listing says so; the snapshot
    /// does not invent one.
    pub listing: Option<ListingBasis>,
    /// Live tabs in listing order (same order as [`Store::tabs`]),
    /// consistent with `listing` — both are read in one transaction.
    pub tabs: Vec<TabSnapshot>,
    /// The sync-policy annotation at its revision — the annotation basis
    /// a plan cites. `None` means exactly "no sync policy": the
    /// annotations handle is required, so absent intent is never
    /// conflated with not having looked.
    pub policy: Option<AnnotationRow>,
}

impl Store {
    /// Snapshot one league's stash facts and the account's sync-policy
    /// row, so a plan's fact basis and annotation revision come from one
    /// read, bound to one account.
    pub fn stash_snapshot(&self, league: &str, annotations: &Annotations) -> Result<StashSnapshot> {
        // All fact reads share one read transaction: under WAL the daemon
        // commits while frontends read, and a listing landing between the
        // basis query and the tab query would pair the old response id
        // with the new tab set. (The annotations file is a separate
        // database and cannot join this transaction; its basis is the
        // policy row's revision, which the CAS write path re-checks.)
        let tx = self.conn.unchecked_transaction()?;
        let accounts: Vec<(String, Option<String>)> = {
            let mut stmt = tx.prepare("SELECT uuid, name FROM account")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        let (account_uuid, account_name) = match accounts.as_slice() {
            [one] => one.clone(),
            [] => bail!(
                "facts file {} records no account identity (no profile response has landed); \
                 a snapshot cannot bind intent to it — one login fixes this",
                self.path.display()
            ),
            many => bail!(
                "facts file {} records {} account identities; refusing to pair intent with it",
                self.path.display(),
                many.len()
            ),
        };
        // Pairing is by the uuid the annotations file carries internally
        // (v2 `meta`), not by filename — a copied or renamed file keeps its
        // owner. A handle with no identity (a pre-v2 file opened from a
        // raw path) is refused, never trusted.
        match annotations.uuid() {
            Some(u) if u == account_uuid => {}
            Some(u) => bail!(
                "annotations file {} belongs to account uuid {u}, not {account_uuid}",
                annotations.path().display()
            ),
            None => bail!(
                "annotations handle {} carries no account identity; open it with \
                 Annotations::open_for so the pairing is checkable",
                annotations.path().display()
            ),
        }
        // The league of a listing lives in its params; an omitted league
        // defaulted to "Standard" at record time (`Endpoint::from_job`),
        // so the match here defaults the same way.
        let listing = tx
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
        type RawTab = (
            String,
            Option<String>,
            String,
            String,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
            i64,
        );
        let rows: Vec<RawTab> = {
            let mut stmt = tx.prepare(&format!(
                "SELECT t.id, t.parent, COALESCE(t.name, ''), COALESCE(t.type, ''), t.idx, t.listed_at, t.listed_response, t.fetched_at, t.listed_json,
                        (SELECT count(*) FROM items i WHERE i.location_kind = 'stash' AND i.location_id = t.id AND i.removed_at IS NULL)
                   FROM tabs t WHERE t.league = ?1 AND t.removed_at IS NULL {TAB_ORDER_SQL}"
            ))?;
            let rows = stmt.query_map([league], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                ))
            })?;
            rows.collect::<Result<_, _>>()?
        };
        tx.finish()?;
        let tabs = rows
            .into_iter()
            .map(
                |(
                    id,
                    parent,
                    name,
                    r#type,
                    idx,
                    listed_at,
                    listed_response,
                    fetched_at,
                    listed_json,
                    item_count,
                )| {
                    // A row this store wrote that no longer parses is a
                    // damaged file, reported with its address — never
                    // silently read as "no metadata".
                    let metadata = match &listed_json {
                        None => Value::Null,
                        Some(raw) => serde_json::from_str::<Value>(raw)
                            .with_context(|| {
                                format!("tab {league}/{id}: malformed listing entry in store")
                            })?
                            .get("metadata")
                            .cloned()
                            .unwrap_or(Value::Null),
                    };
                    Ok(TabSnapshot {
                        id,
                        parent,
                        name,
                        r#type,
                        idx,
                        listed_at,
                        listed_response,
                        fetched_at,
                        metadata,
                        item_count,
                    })
                },
            )
            .collect::<Result<Vec<_>>>()?;
        let policy = annotations.get("account", "", SYNC_POLICY_KIND)?;
        Ok(StashSnapshot {
            account_uuid,
            account_name,
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
    use crate::{Endpoint, annotations_path};
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

    /// A store whose account identity is on record, as every real store's
    /// is after one login (`/profile` lands at login since 2026-08-31).
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

    #[test]
    fn the_snapshot_names_the_latest_listing_and_the_policy_revision() {
        let mut s = store();
        let mut a = Annotations::open_memory_for("u-1").unwrap();
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
        let first = s.stash_snapshot("Standard", &a).unwrap();
        let basis = first.listing.unwrap();
        assert_eq!(basis.fetched_at, 100);
        assert!(first.policy.is_none());
        assert_eq!(first.account_uuid, "u-1");
        assert_eq!(first.account_name.as_deref(), Some("tom"));
        // The refresh parent records the same listing with normalized
        // params; a later listing replaces the basis, and the tab set is
        // stamped with the basis it belongs to.
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
        let snap = s.stash_snapshot("Standard", &a).unwrap();
        let later = snap.listing.unwrap();
        assert!(later.response_id > basis.response_id);
        assert_eq!(later.fetched_at, 200);
        assert_eq!(snap.tabs[0].listed_response, Some(later.response_id));
        let row = snap.policy.unwrap();
        assert_eq!((row.revision, &row.value), (1, &policy.value));
        // A tombstoned policy is no policy — but its revision still gates
        // the next write, which is the annotation layer's business.
        a.delete("account", "", SYNC_POLICY_KIND, 1).unwrap();
        assert!(s.stash_snapshot("Standard", &a).unwrap().policy.is_none());
    }

    #[test]
    fn the_basis_is_per_league_and_absent_when_never_listed() {
        let mut s = store();
        let a = Annotations::open_memory_for("u-1").unwrap();
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
        let std = s.stash_snapshot("Standard", &a).unwrap();
        assert!(std.listing.is_none());
        assert_eq!(std.tabs.len(), 1);
        let x1 = &std.tabs[0];
        assert_eq!(
            (x1.id.as_str(), x1.listed_at, x1.fetched_at),
            ("x1", None, Some(60))
        );
        assert_eq!((x1.listed_response, x1.item_count), (None, 1));
        assert_eq!(x1.metadata, Value::Null);
        let hc = s.stash_snapshot("Hardcore", &a).unwrap();
        assert_eq!(hc.listing.unwrap().fetched_at, 50);
        assert_eq!(hc.tabs.len(), 1);
        assert_eq!(hc.tabs[0].id, "h1");
    }

    #[test]
    fn metadata_rides_verbatim_and_removed_tabs_leave_the_snapshot() {
        let mut s = store();
        let a = Annotations::open_memory_for("u-1").unwrap();
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
        let snap = s.stash_snapshot("Standard", &a).unwrap();
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
            .stash_snapshot("Standard", &a)
            .unwrap()
            .tabs
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["t1", "m1", "s1"]);
    }

    #[test]
    fn a_fetch_never_clobbers_the_listed_metadata() {
        let mut s = store();
        let a = Annotations::open_memory_for("u-1").unwrap();
        s.record(
            &listing_ep(),
            &json!({}),
            200,
            &json!({ "stashes": [
                { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0, "metadata": { "colour": "7c5436" } },
            ]}),
            100,
        )
        .unwrap();
        // The fetched body lands in `json` — with different metadata, as
        // GGG is free to send — and the listing's copy must survive it.
        s.record(
            &stash_ep("t1", None),
            &json!({ "league": "Standard", "id": "t1" }),
            200,
            &json!({ "stash": { "id": "t1", "name": "One", "type": "PremiumStash",
                "metadata": { "colour": "000000" }, "items": [ item("i1") ] } }),
            110,
        )
        .unwrap();
        let snap = s.stash_snapshot("Standard", &a).unwrap();
        let t1 = &snap.tabs[0];
        assert_eq!(t1.metadata, json!({ "colour": "7c5436" }));
        assert_eq!((t1.fetched_at, t1.item_count), (Some(110), 1));
        // The next listing refreshes the listed copy.
        s.record(
            &listing_ep(),
            &json!({}),
            200,
            &json!({ "stashes": [
                { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0, "metadata": { "colour": "ffffff" } },
            ]}),
            120,
        )
        .unwrap();
        let snap = s.stash_snapshot("Standard", &a).unwrap();
        assert_eq!(snap.tabs[0].metadata, json!({ "colour": "ffffff" }));
    }

    #[test]
    fn two_listings_in_one_second_still_retire_dropped_tabs() {
        let mut s = store();
        let a = Annotations::open_memory_for("u-1").unwrap();
        let two = json!({ "stashes": [
            { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 },
            { "id": "t2", "name": "Two", "type": "PremiumStash", "index": 1 },
        ]});
        let one = json!({ "stashes": [
            { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 },
        ]});
        // Same unix second: membership must be per response, not per
        // clock tick, or t2 stays live while the basis says otherwise.
        s.record(&listing_ep(), &json!({}), 200, &two, 100).unwrap();
        s.record(&listing_ep(), &json!({}), 200, &one, 100).unwrap();
        let snap = s.stash_snapshot("Standard", &a).unwrap();
        let basis = snap.listing.unwrap();
        assert_eq!(
            snap.tabs.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
            vec!["t1"]
        );
        assert_eq!(snap.tabs[0].listed_response, Some(basis.response_id));
    }

    #[test]
    fn a_listing_without_a_stashes_array_is_refused_whole() {
        let mut s = store();
        let a = Annotations::open_memory_for("u-1").unwrap();
        s.record(
            &listing_ep(),
            &json!({}),
            200,
            &json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] }),
            100,
        )
        .unwrap();
        // A 2xx with no `stashes` array (a maintenance page, a shape
        // change) is an error of the stable kind: no tab removed, no
        // basis minted.
        let err = s
            .record(
                &listing_ep(),
                &json!({}),
                200,
                &json!({ "error": "?" }),
                200,
            )
            .unwrap_err();
        assert!(
            err.downcast_ref::<crate::MalformedBody>().is_some(),
            "{err:#}"
        );
        let snap = s.stash_snapshot("Standard", &a).unwrap();
        assert_eq!(snap.listing.unwrap().fetched_at, 100);
        assert_eq!(snap.tabs.len(), 1);
        // An array whose entries lack their identity is just as malformed:
        // ingesting it would retire every real tab. The error rolls the
        // whole transaction back — the id-less entry is not half-applied.
        let err = s
            .record(
                &listing_ep(),
                &json!({}),
                200,
                &json!({ "stashes": [ { "name": "NoId", "type": "PremiumStash" } ] }),
                300,
            )
            .unwrap_err();
        assert!(
            err.downcast_ref::<crate::MalformedBody>().is_some(),
            "{err:#}"
        );
        let snap = s.stash_snapshot("Standard", &a).unwrap();
        assert_eq!(snap.listing.unwrap().fetched_at, 100);
        assert_eq!(snap.tabs.len(), 1);
        // Same rule for the character list: malformed is never "empty",
        // and a name-less entry poisons nothing.
        s.record(
            &Endpoint::Characters,
            &json!({}),
            200,
            &json!({ "characters": [ { "name": "Hero", "league": "Standard" } ] }),
            100,
        )
        .unwrap();
        assert!(
            s.record(&Endpoint::Characters, &json!({}), 200, &json!({}), 200)
                .is_err()
        );
        assert!(
            s.record(
                &Endpoint::Characters,
                &json!({}),
                200,
                &json!({ "characters": [ { "league": "Standard" } ] }),
                200
            )
            .is_err()
        );
        assert_eq!(s.characters(None).unwrap().len(), 1);
    }

    #[test]
    fn malformed_stored_listing_json_is_an_error_with_the_tab_address() {
        let mut s = store();
        let a = Annotations::open_memory_for("u-1").unwrap();
        s.record(
            &listing_ep(),
            &json!({}),
            200,
            &json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] }),
            100,
        )
        .unwrap();
        s.conn
            .execute("UPDATE tabs SET listed_json = 'not json'", [])
            .unwrap();
        let err = s.stash_snapshot("Standard", &a).unwrap_err();
        assert!(err.to_string().contains("Standard/t1"), "{err:#}");
    }

    #[test]
    fn the_snapshot_binds_facts_and_intent_to_one_account() {
        let a = Annotations::open_memory_for("u-1").unwrap();
        // A facts file with no recorded account cannot bind intent.
        let s = Store::open_memory().unwrap();
        let err = s.stash_snapshot("Standard", &a).unwrap_err();
        assert!(err.to_string().contains("no account identity"), "{err:#}");
        // A handle bound to another account's uuid is refused; a handle
        // never bound at all (raw open, no stored identity) is refused
        // too — never trusted to the caller.
        let s = store(); // records uuid u-1
        let other = Annotations::open_memory_for("u-2").unwrap();
        let err = s.stash_snapshot("Standard", &other).unwrap_err();
        assert!(err.to_string().contains("u-2"), "{err:#}");
        let unbound = Annotations::open_memory().unwrap();
        let err = s.stash_snapshot("Standard", &unbound).unwrap_err();
        assert!(err.to_string().contains("no account identity"), "{err:#}");
        // The account's own file is accepted — including when reopened
        // from its raw path, because the uuid lives inside the file.
        let dir = std::env::temp_dir().join(format!(
            "acq-snap-bind-{}-{}",
            std::process::id(),
            crate::now()
        ));
        drop(Annotations::open_for(&dir, "u-1").unwrap());
        let reopened = Annotations::open(&annotations_path(&dir, "u-1")).unwrap();
        assert_eq!(reopened.uuid(), Some("u-1"));
        let snap = s.stash_snapshot("Standard", &reopened).unwrap();
        assert_eq!(snap.account_uuid, "u-1");
        // A copied/renamed file keeps its owner: u-2's database placed at
        // u-1's path still says u-2 and is refused.
        drop(Annotations::open_for(&dir, "u-2").unwrap());
        let dir2 = dir.join("elsewhere");
        std::fs::create_dir_all(&dir2).unwrap();
        std::fs::copy(
            annotations_path(&dir, "u-2"),
            annotations_path(&dir2, "u-1"),
        )
        .unwrap();
        let copied = Annotations::open(&annotations_path(&dir2, "u-1")).unwrap();
        assert_eq!(copied.uuid(), Some("u-2"));
        let err = s.stash_snapshot("Standard", &copied).unwrap_err();
        assert!(err.to_string().contains("u-2"), "{err:#}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
