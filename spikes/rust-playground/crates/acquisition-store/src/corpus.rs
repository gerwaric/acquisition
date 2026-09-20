//! The search read (C108, C98): every live item's stored body with its
//! ingest facts, streamed under one snapshot with the locations it sits
//! at, and the revision that snapshot is labelled with. What a body says
//! is derived in `acquisition-search`, never here; this crate names no
//! search type, and the search crate copies a [`CorpusItem`]'s columns
//! into a struct of its own. Rulings: `decisions/store.md`, C108;
//! `decisions/search.md`, C103 (the deriver) and C98 (the basis).
//!
//! # As built
//!
//! - **One snapshot.** [`Store::read_corpus`] runs inside one read
//!   transaction, as [`Store::refresh_snapshot`] does. Its first statement
//!   reads the revision, which is what fixes the WAL snapshot: the header
//!   and every body the caller is then handed describe that state, however
//!   long the caller takes and whatever lands meanwhile.
//! - **Streamed.** The caller's closure is handed the rows one at a time
//!   ([`CorpusItems`]); a returned stream would have to own the statement
//!   and the transaction it borrows. Ordered by location kind and id, then
//!   realm and item id: the `items_location` index yields that order with
//!   a sort per location, where realm first sorted the whole corpus,
//!   bodies included (43 MB against 10 MB resident over 22,721 items,
//!   2026-09-20). Every key is an ingest fact or the id, so
//!   [`Store::rebuild`] cannot reorder a read.
//! - **League is joined as `read_items` joins it** (S131): a stash item's
//!   league is its tab's; a character item's is the character's current
//!   listing-owned league — never the stamp the item took at ingest — and
//!   a league-less character's items carry none.
//! - **The header is what coverage is stated from**: every live location,
//!   and the latest listing of each realm's characters and of each (realm,
//!   league)'s tabs — the bases the snapshots cite (`stash_basis`,
//!   `character_basis`), an empty listing included, since a location's own
//!   `listed_at` cannot say when a list that names nothing was seen, and a
//!   substash's is its parent's fetch. A location is live with its parent
//!   (C54): a substash whose tab a listing retired keeps its row for the
//!   planner's orphan report, has no live item, cannot be fetched, and is
//!   not in the header. A folder is a row no fetch ever fills, so a tab
//!   carries its `type` as GGG gave it, as the snapshots do, and the
//!   consumer counts never-fetched locations without the folders.
//! - **Live** is the item and its location both, the membership
//!   `read_items` reports: a location a listing retired takes its items
//!   with it (C54). Removed items are not read: no consumer asks for them
//!   yet, and a read that handed them over would change under a prune
//!   (below).
//! - **The body is text, unparsed**, exactly as `items.json` holds it (the
//!   item minus its lifted `socketedItems`). One that does not parse is
//!   the search crate's to report on that item; [`Store::search`] turns it
//!   into a silent null.
//! - **The realms** are those the file has a tab or a character under,
//!   live or retired, whatever the scope: what C96's "a search names its
//!   realm" is asked of. An item lands only beside its location's row, so
//!   `items` adds no realm, and reading it for one cost more than the rest
//!   of the header (8 ms of 13 over 22,721 items, a release build).
//!
//! # Decisions as recorded
//!
//! ## C108, C98 — the revision is the highest `responses.id`
//!
//! Ruled 2026-09-19 (owner: "I agree with (a)"): no schema change, facts
//! stay v7, and a stamped counter stays the fallback. It advances in the
//! transaction that changes bodies, locations or membership, as read line
//! by line on 2026-09-20:
//!
//! - [`Store::record`] is the one door that writes `items`, `tabs` or
//!   `characters` outside a migration, and every path through it that
//!   commits has inserted exactly one `responses` row in that transaction;
//!   every earlier exit is an error, which rolls back whole. A refused
//!   body's row goes to `refused` alone, in a transaction of its own, and
//!   the id it did not keep is the next record's — no reader ever saw it.
//! - `acq store import` goes through `record`. [`Store::rebuild`] rewrites
//!   the derived columns alone, none of which this read hands over or
//!   orders by. No code deletes a response. No other crate opens a facts
//!   file to write it.
//! - A listing, a profile or a withheld fetch advances the revision though
//!   no item changed: a held corpus is reloaded once more than it needed.
//! - A schema migration moves rows with no response, so the facts version
//!   is part of the [`Revision`] and is printed beside it.
//! - `responses.id` is a plain `INTEGER PRIMARY KEY`, so SQLite hands out
//!   the highest id plus one, and an id comes back only if the highest row
//!   is deleted. **Whatever prunes this file keeps two rules: it never
//!   deletes the highest `responses` row, and a prune that changes what a
//!   basis read returns advances the revision.** A prune of removed items,
//!   of events or of old responses changes nothing this read returns while
//!   it keeps the latest listing of each realm and league, which the
//!   header and the snapshots read.
//! - Not covered, by this number or by a stamped counter: a facts file
//!   deleted and refetched, or restored from a copy, counts from an earlier
//!   number again (`decisions/search.md`, "Parked").

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{Store, TAB_ORDER_KEYS};

/// What a read of the facts is labelled with (C98's basis, the facts
/// half): equal revisions of one file are one state of its items, their
/// locations and their membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    /// The highest `responses.id`; 0 before any response has landed.
    pub response: i64,
    /// The facts schema the file is stamped with: a migration moves rows
    /// and writes no response.
    pub facts_version: i64,
}

/// Which realms a read covers (C58, C96): one, or every realm the file
/// holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmScope<'a> {
    One(&'a str),
    All,
}

/// What [`Store::read_corpus`] says before the first item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusHeader {
    pub revision: Revision,
    /// The account the facts file records, as the snapshots name it.
    pub account_uuid: String,
    pub account_name: Option<String>,
    /// Every realm the file has a tab or a character under, sorted,
    /// whatever the scope.
    pub realms: Vec<String>,
    /// The latest listing in scope of each realm's characters and of each
    /// (realm, league)'s tabs, by realm; one that named nothing included.
    pub listings: Vec<ListingSeen>,
    /// Every live location in scope, items or none: tabs in listing order
    /// by realm and league, then characters as `acq store characters`
    /// orders them.
    pub locations: Vec<LocationRow>,
}

/// When a list of locations was last seen: the `responses` row the
/// snapshots cite as a plan's basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListingSeen {
    pub realm: String,
    /// A tab listing's league; `None` on a character listing, which is
    /// its realm's.
    pub league: Option<String>,
    /// `stash` or `character`: what the list names, as
    /// [`LocationRow::kind`] spells it.
    pub kind: String,
    pub response_id: i64,
    pub fetched_at: i64,
}

/// One live location: a folder, a tab, a substash or a character, by its
/// full coordinate (C54), with what coverage needs to say about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocationRow {
    pub realm: String,
    /// A tab's league; a character's current listing-owned league, `None`
    /// when the listing gave none (C61).
    pub league: Option<String>,
    /// `stash` or `character`, as [`CorpusItem::location_kind`] spells it.
    pub kind: String,
    pub id: String,
    /// A tab's folder, or a substash's tab.
    pub parent: Option<String>,
    pub name: String,
    /// A tab's `type`, verbatim; `None` on a character. `Folder` is the
    /// one that holds no items and is never fetched.
    pub tab_type: Option<String>,
    pub listed_at: Option<i64>,
    /// `None`: never fetched — or retired and revived since (C54).
    pub fetched_at: Option<i64>,
}

/// One live item: its ingest facts (S128) and its body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CorpusItem {
    pub id: String,
    pub realm: String,
    /// Joined, never the item's own stamp; `None` on a league-less
    /// character's item.
    pub league: Option<String>,
    pub location_kind: String,
    pub location_id: String,
    /// The array the item came from; `None` on a row from before facts v4.
    pub container: Option<String>,
    /// The item this one is socketed in.
    pub socketed_in: Option<String>,
    pub first_seen: i64,
    pub last_seen: i64,
    /// `items.json` as stored, unparsed.
    pub body: String,
}

/// The items of one [`Store::read_corpus`], one row at a time.
pub struct CorpusItems<'a> {
    rows: rusqlite::Rows<'a>,
}

impl Iterator for CorpusItems<'_> {
    type Item = Result<CorpusItem>;

    fn next(&mut self) -> Option<Self::Item> {
        let row = match self.rows.next() {
            Ok(Some(row)) => row,
            Ok(None) => return None,
            Err(e) => return Some(Err(e.into())),
        };
        Some(corpus_item(row).map_err(Into::into))
    }
}

fn corpus_item(r: &rusqlite::Row<'_>) -> rusqlite::Result<CorpusItem> {
    Ok(CorpusItem {
        id: r.get(0)?,
        realm: r.get(1)?,
        league: r.get(2)?,
        location_kind: r.get(3)?,
        location_id: r.get(4)?,
        container: r.get(5)?,
        socketed_in: r.get(6)?,
        first_seen: r.get(7)?,
        last_seen: r.get(8)?,
        body: r.get(9)?,
    })
}

const ITEMS_SQL: &str = "SELECT i.id, i.realm,
        CASE WHEN i.location_kind = 'character' THEN c.league ELSE i.league END,
        i.location_kind, i.location_id, i.container, i.socketed_in, i.first_seen, i.last_seen, i.json
   FROM items i
   LEFT JOIN tabs t ON i.location_kind = 'stash' AND t.realm = i.realm AND t.league = i.league
                   AND t.id = i.location_id AND t.removed_at IS NULL
   LEFT JOIN characters c ON i.location_kind = 'character' AND c.realm = i.realm
                   AND c.id = i.location_id AND c.removed_at IS NULL
  WHERE i.removed_at IS NULL AND (?1 IS NULL OR i.realm = ?1)
    AND (t.id IS NOT NULL OR c.id IS NOT NULL)
  ORDER BY i.location_kind, i.location_id, i.realm, i.id";

fn read_revision(tx: &rusqlite::Transaction) -> Result<Revision> {
    Ok(Revision {
        response: tx.query_row("SELECT COALESCE(MAX(id), 0) FROM responses", [], |r| {
            r.get(0)
        })?,
        facts_version: tx.query_row("PRAGMA user_version", [], |r| r.get(0))?,
    })
}

fn read_realms(tx: &rusqlite::Transaction) -> Result<Vec<String>> {
    let mut stmt =
        tx.prepare("SELECT realm FROM tabs UNION SELECT realm FROM characters ORDER BY 1")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// The realm and league of a listing live in its params and default as
/// `stash_basis` and `character_basis` default them; the bare `fetched_at`
/// is the row `MAX(id)` chose, which SQLite guarantees.
fn read_listings(tx: &rusqlite::Transaction, realm: Option<&str>) -> Result<Vec<ListingSeen>> {
    let mut stmt = tx.prepare(
        "SELECT COALESCE(json_extract(params, '$.realm'), 'pc'),
                CASE WHEN endpoint = 'stashes' THEN COALESCE(json_extract(params, '$.league'), 'Standard') END,
                CASE WHEN endpoint = 'stashes' THEN 'stash' ELSE 'character' END,
                MAX(id), fetched_at
           FROM responses
          WHERE endpoint IN ('stashes', 'characters') AND status BETWEEN 200 AND 299
            AND (?1 IS NULL OR COALESCE(json_extract(params, '$.realm'), 'pc') = ?1)
          GROUP BY 1, 2, 3 ORDER BY 1, 3 DESC, 2",
    )?;
    let rows = stmt.query_map([realm], |r| {
        Ok(ListingSeen {
            realm: r.get(0)?,
            league: r.get(1)?,
            kind: r.get(2)?,
            response_id: r.get(3)?,
            fetched_at: r.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

fn read_locations(tx: &rusqlite::Transaction, realm: Option<&str>) -> Result<Vec<LocationRow>> {
    let mut locations = Vec::new();
    let mut tabs = tx.prepare(&format!(
        "SELECT t.realm, t.league, 'stash', t.id, t.parent, COALESCE(t.name, ''), t.listed_at, t.fetched_at, t.type
           FROM tabs t WHERE t.removed_at IS NULL AND (?1 IS NULL OR t.realm = ?1)
            AND NOT EXISTS (SELECT 1 FROM tabs p WHERE p.realm = t.realm AND p.league = t.league
                                                  AND p.id = t.parent AND p.removed_at IS NOT NULL)
          ORDER BY t.realm, t.league, {TAB_ORDER_KEYS}"
    ))?;
    let mut characters = tx.prepare(
        "SELECT c.realm, c.league, 'character', c.id, NULL, c.name, c.listed_at, c.fetched_at, NULL
           FROM characters c WHERE c.removed_at IS NULL AND (?1 IS NULL OR c.realm = ?1)
          ORDER BY c.realm, c.league, c.level DESC, c.name",
    )?;
    for stmt in [&mut tabs, &mut characters] {
        let rows = stmt.query_map([realm], |r| {
            Ok(LocationRow {
                realm: r.get(0)?,
                league: r.get(1)?,
                kind: r.get(2)?,
                id: r.get(3)?,
                parent: r.get(4)?,
                name: r.get(5)?,
                listed_at: r.get(6)?,
                fetched_at: r.get(7)?,
                tab_type: r.get(8)?,
            })
        })?;
        for row in rows {
            locations.push(row?);
        }
    }
    Ok(locations)
}

impl Store {
    /// The revision of the facts as they stand: what a consumer holding a
    /// corpus compares before every answer (C98).
    pub fn revision(&self) -> Result<Revision> {
        let tx = self.conn.unchecked_transaction()?;
        let revision = read_revision(&tx)?;
        tx.finish()?;
        Ok(revision)
    }

    /// Read every live item of the scope under one snapshot: `read` is
    /// handed the header and then the items, and what it returns is
    /// returned. A facts file that records no account identity is refused,
    /// as the snapshots refuse it.
    pub fn read_corpus<T>(
        &self,
        realm: RealmScope<'_>,
        read: impl FnOnce(&CorpusHeader, &mut CorpusItems<'_>) -> Result<T>,
    ) -> Result<T> {
        let realm = match realm {
            RealmScope::One(realm) => Some(realm),
            RealmScope::All => None,
        };
        let tx = self.conn.unchecked_transaction()?;
        // First, so that the number names the snapshot everything below is
        // read from.
        let revision = read_revision(&tx)?;
        let (account_uuid, account_name) = self.account_identity(&tx)?;
        let header = CorpusHeader {
            revision,
            account_uuid,
            account_name,
            realms: read_realms(&tx)?,
            listings: read_listings(&tx, realm)?,
            locations: read_locations(&tx, realm)?,
        };
        let out = {
            let mut stmt = tx.prepare(ITEMS_SQL)?;
            let mut items = CorpusItems {
                rows: stmt.query([realm])?,
            };
            read(&header, &mut items)?
        };
        tx.finish()?;
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Endpoint;
    use serde_json::{Value, json};
    use std::path::PathBuf;

    fn tmp() -> PathBuf {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "acq-corpus-{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("facts.db")
    }

    fn profile(s: &mut Store) {
        s.record(
            &Endpoint::Profile,
            &json!({}),
            200,
            &json!({ "uuid": "u-1", "name": "tom" }),
            1,
        )
        .unwrap();
    }

    fn item(id: &str, name: &str) -> Value {
        json!({ "id": id, "name": name, "typeLine": "Imperial Bow", "baseType": "Imperial Bow", "x": 0, "y": 0 })
    }

    fn list_tabs(s: &mut Store, realm: &str, league: &str, tabs: Value, at: i64) {
        s.record(
            &Endpoint::Stashes {
                realm: realm.into(),
                league: league.into(),
            },
            &json!({ "realm": realm, "league": league }),
            200,
            &json!({ "stashes": tabs }),
            at,
        )
        .unwrap();
    }

    fn fetch_tab(s: &mut Store, realm: &str, league: &str, id: &str, items: Vec<Value>, at: i64) {
        s.record(
            &Endpoint::Stash {
                realm: realm.into(),
                league: league.into(),
                id: id.into(),
                sub: None,
            },
            &json!({ "realm": realm, "league": league, "id": id }),
            200,
            &json!({ "stash": { "id": id, "name": id, "type": "PremiumStash", "items": items } }),
            at,
        )
        .unwrap();
    }

    fn list_characters(s: &mut Store, realm: &str, entries: Value, at: i64) {
        s.record(
            &Endpoint::Characters {
                realm: realm.into(),
            },
            &json!({ "realm": realm }),
            200,
            &json!({ "characters": entries }),
            at,
        )
        .unwrap();
    }

    fn fetch_character(s: &mut Store, realm: &str, character: Value, at: i64) {
        let name = character["name"].as_str().unwrap().to_string();
        s.record(
            &Endpoint::Character {
                realm: realm.into(),
                name: name.clone(),
            },
            &json!({ "realm": realm, "name": name }),
            200,
            &json!({ "character": character }),
            at,
        )
        .unwrap();
    }

    fn tab(id: &str) -> Value {
        json!({ "id": id, "name": id, "type": "PremiumStash" })
    }

    fn read_all(s: &Store, realm: RealmScope<'_>) -> (CorpusHeader, Vec<CorpusItem>) {
        s.read_corpus(realm, |header, items| {
            Ok((header.clone(), items.collect::<Result<Vec<_>>>()?))
        })
        .unwrap()
    }

    fn ids(items: &[CorpusItem]) -> Vec<&str> {
        items.iter().map(|i| i.id.as_str()).collect()
    }

    /// C108, C98 — the read is one snapshot: a fetch that lands part-way
    /// through the items changes nothing the read then says, and the next
    /// read carries a higher revision and the change.
    #[test]
    fn a_fetch_landing_mid_read_is_the_next_reads_and_never_this_ones() {
        let path = tmp();
        let mut a = Store::open(&path).unwrap();
        profile(&mut a);
        list_tabs(&mut a, "pc", "Standard", json!([tab("t1"), tab("t2")]), 10);
        fetch_tab(
            &mut a,
            "pc",
            "Standard",
            "t1",
            vec![item("a", "Old"), item("b", "Gone Soon")],
            20,
        );
        let before = a.revision().unwrap();
        let mut b = Store::open(&path).unwrap();

        let (header, items) = a
            .read_corpus(RealmScope::All, |header, items| {
                // The second handle commits twice while this read is open.
                // Before the first item is stepped to — where a read with
                // no transaction would begin a newer snapshot than its
                // header's — `a` is rewritten, `b` leaves and `z` arrives;
                // part-way through the items, t2 is fetched for the first
                // time.
                fetch_tab(
                    &mut b,
                    "pc",
                    "Standard",
                    "t1",
                    vec![item("a", "New"), item("z", "Arrived")],
                    30,
                );
                let first = items.next().unwrap()?;
                fetch_tab(&mut b, "pc", "Standard", "t2", vec![item("c", "Late")], 31);
                assert!(b.revision().unwrap().response > before.response);
                let mut all = vec![first];
                for next in items {
                    all.push(next?);
                }
                Ok((header.clone(), all))
            })
            .unwrap();
        assert_eq!(header.revision, before);
        assert_eq!(ids(&items), ["a", "b"]);
        assert!(items[0].body.contains("\"Old\""));
        let t = |h: &CorpusHeader, id: &str| {
            h.locations.iter().find(|l| l.id == id).unwrap().fetched_at
        };
        assert_eq!((t(&header, "t1"), t(&header, "t2")), (Some(20), None));

        let (header, items) = read_all(&a, RealmScope::All);
        assert_eq!(header.revision.response, before.response + 2);
        assert_eq!(ids(&items), ["a", "z", "c"]);
        assert!(items[0].body.contains("\"New\""));
        assert_eq!((t(&header, "t1"), t(&header, "t2")), (Some(30), Some(31)));
    }

    /// C108 (S131) — league is joined as `read_items` joins it: a character
    /// item takes the character's current listing league, never its own
    /// stamp, and a league-less character's items carry none.
    #[test]
    fn a_character_items_league_is_the_listings_and_none_when_the_listing_gave_none() {
        let mut s = Store::open_memory().unwrap();
        profile(&mut s);
        list_characters(
            &mut s,
            "pc",
            json!([{ "id": "c1", "name": "Mover", "league": "Allflame" },
                   { "id": "c2", "name": "Nowhere" }]),
            10,
        );
        fetch_character(
            &mut s,
            "pc",
            json!({ "id": "c1", "name": "Mover", "league": "Allflame", "equipment": [item("helm", "Helm")] }),
            20,
        );
        fetch_character(
            &mut s,
            "pc",
            json!({ "id": "c2", "name": "Nowhere", "inventory": [item("scrap", "Scrap")] }),
            21,
        );
        // The league ends: the listing moves the character, and no fetch
        // has restamped its items.
        list_characters(
            &mut s,
            "pc",
            json!([{ "id": "c1", "name": "Mover", "league": "Standard" },
                   { "id": "c2", "name": "Nowhere" }]),
            30,
        );
        let stamp: Option<String> = s
            .conn
            .query_row("SELECT league FROM items WHERE id = 'helm'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(stamp.as_deref(), Some("Allflame"));

        let (header, items) = read_all(&s, RealmScope::All);
        let league = |id: &str| items.iter().find(|i| i.id == id).unwrap().league.clone();
        assert_eq!(league("helm").as_deref(), Some("Standard"));
        assert_eq!(league("scrap"), None);
        let helm = items.iter().find(|i| i.id == "helm").unwrap();
        assert_eq!(
            (helm.location_kind.as_str(), helm.location_id.as_str()),
            ("character", "c1")
        );
        assert_eq!(helm.container.as_deref(), Some("equipment"));
        let of = |id: &str| header.locations.iter().find(|l| l.id == id).unwrap();
        assert_eq!(of("c1").league.as_deref(), Some("Standard"));
        assert_eq!(of("c2").league, None);
        assert_eq!(of("c1").name, "Mover");
    }

    /// C108, C54 — live is the item and its location both, a location is
    /// its full coordinate, and the scope is one realm or all while the
    /// realms are every realm the file holds.
    #[test]
    fn only_live_items_at_live_locations_by_full_coordinate_in_the_scope_asked() {
        let mut s = Store::open_memory().unwrap();
        profile(&mut s);
        list_tabs(&mut s, "pc", "Standard", json!([tab("t1"), tab("t2")]), 10);
        list_tabs(&mut s, "xbox", "Standard", json!([tab("t1")]), 11);
        let mut host = item("bow", "Bow");
        host["socketedItems"] = json!([item("gem", "Gem")]);
        fetch_tab(
            &mut s,
            "pc",
            "Standard",
            "t1",
            vec![host, item("sold", "Sold")],
            20,
        );
        fetch_tab(
            &mut s,
            "pc",
            "Standard",
            "t2",
            vec![item("doomed", "Doomed")],
            21,
        );
        fetch_tab(
            &mut s,
            "xbox",
            "Standard",
            "t1",
            vec![item("pad", "Pad")],
            22,
        );
        list_characters(
            &mut s,
            "poe2",
            json!([{ "id": "c9", "name": "Two", "league": "Standard" }]),
            23,
        );
        // `sold` leaves its tab; a listing retires t2 and its item with it.
        let mut host = item("bow", "Bow");
        host["socketedItems"] = json!([item("gem", "Gem")]);
        fetch_tab(&mut s, "pc", "Standard", "t1", vec![host], 30);
        list_tabs(&mut s, "pc", "Standard", json!([tab("t1")]), 31);

        let (header, items) = read_all(&s, RealmScope::All);
        assert_eq!(header.realms, ["pc", "poe2", "xbox"]);
        assert_eq!(
            (header.account_uuid.as_str(), header.account_name.as_deref()),
            ("u-1", Some("tom"))
        );
        // Location kind and id, then realm, then item id.
        assert_eq!(ids(&items), ["bow", "gem", "pad"]);
        assert_eq!(items[1].socketed_in.as_deref(), Some("bow"));
        assert!(!items[0].body.contains("socketedItems"));
        assert_eq!((items[0].first_seen, items[0].last_seen), (20, 30));
        let places: Vec<(&str, &str, &str)> = header
            .locations
            .iter()
            .map(|l| (l.realm.as_str(), l.kind.as_str(), l.id.as_str()))
            .collect();
        assert_eq!(
            places,
            [
                ("pc", "stash", "t1"),
                ("xbox", "stash", "t1"),
                ("poe2", "character", "c9")
            ]
        );

        let (header, items) = read_all(&s, RealmScope::One("xbox"));
        assert_eq!(header.realms, ["pc", "poe2", "xbox"]);
        assert_eq!(ids(&items), ["pad"]);
        assert_eq!(items[0].realm, "xbox");
        assert_eq!(header.locations.len(), 1);
    }

    /// C108, C54 — the header is what coverage is stated from: a folder is
    /// told apart from a tab no fetch has reached, a substash whose tab a
    /// listing retired is no live location, and the latest listing of each
    /// list is carried, one that named nothing included.
    #[test]
    fn the_header_tells_folders_orphans_and_empty_listings_as_they_are() {
        let mut s = Store::open_memory().unwrap();
        profile(&mut s);
        let folder = json!({ "id": "f1", "name": "Sale", "type": "Folder",
                             "children": [{ "id": "c1", "name": "Bows", "type": "PremiumStash" }] });
        let maps = json!({ "id": "m1", "name": "Maps", "type": "MapStash" });
        list_tabs(
            &mut s,
            "pc",
            "Standard",
            json!([folder.clone(), maps, tab("t9")]),
            10,
        );
        fetch_tab(&mut s, "pc", "Standard", "c1", vec![item("bow", "Bow")], 20);
        let stash = |sub: Option<&str>| Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "m1".into(),
            sub: sub.map(str::to_string),
        };
        s.record(
            &stash(None),
            &json!({ "id": "m1" }),
            200,
            &json!({ "stash": { "id": "m1", "name": "Maps", "type": "MapStash", "items": [],
                                "children": [{ "id": "s1", "name": "Tier 1", "type": "MapStash" }] } }),
            21,
        )
        .unwrap();
        s.record(
            &stash(Some("s1")),
            &json!({ "id": "m1", "sub": "s1" }),
            200,
            &json!({ "stash": { "id": "s1", "name": "Tier 1", "type": "MapStash", "items": [item("map", "Map")] } }),
            22,
        )
        .unwrap();
        list_characters(&mut s, "pc", json!([]), 40);

        let (header, items) = read_all(&s, RealmScope::All);
        assert_eq!(ids(&items), ["bow", "map"]);
        let row = |h: &CorpusHeader, id: &str| h.locations.iter().find(|l| l.id == id).cloned();
        let f1 = row(&header, "f1").unwrap();
        assert_eq!(
            (f1.tab_type.as_deref(), f1.fetched_at),
            (Some("Folder"), None)
        );
        let c1 = row(&header, "c1").unwrap();
        assert_eq!(
            (c1.tab_type.as_deref(), c1.parent.as_deref(), c1.fetched_at),
            (Some("PremiumStash"), Some("f1"), Some(20))
        );
        assert_eq!(row(&header, "t9").unwrap().fetched_at, None);
        let s1 = row(&header, "s1").unwrap();
        assert_eq!(
            (s1.parent.as_deref(), s1.fetched_at),
            (Some("m1"), Some(22))
        );
        let seen: Vec<(&str, Option<&str>, &str, i64)> = header
            .listings
            .iter()
            .map(|l| {
                (
                    l.realm.as_str(),
                    l.league.as_deref(),
                    l.kind.as_str(),
                    l.fetched_at,
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                ("pc", Some("Standard"), "stash", 10),
                ("pc", None, "character", 40)
            ]
        );
        assert_eq!(header.listings[0].response_id, 2);

        // A listing drops the map tab: its substash keeps its row for the
        // planner's orphan report and is no live location here.
        list_tabs(&mut s, "pc", "Standard", json!([folder, tab("t9")]), 50);
        let kept: i64 = s
            .conn
            .query_row(
                "SELECT count(*) FROM tabs WHERE id = 's1' AND removed_at IS NULL",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(kept, 1);
        let (header, items) = read_all(&s, RealmScope::All);
        assert_eq!(ids(&items), ["bow"]);
        assert!(row(&header, "m1").is_none() && row(&header, "s1").is_none());
        assert_eq!(header.listings[0].fetched_at, 50);

        let (header, _) = read_all(&s, RealmScope::One("xbox"));
        assert!(header.listings.is_empty() && header.locations.is_empty());
    }

    /// C108, C98 — the revision never goes back and moves with every
    /// committed record; a refusal, a rebuild and a reopen leave it. A new
    /// write door on the facts file joins this test (the module doc's two
    /// rules for a prune).
    #[test]
    fn the_revision_moves_with_every_record_and_with_nothing_else() {
        let path = tmp();
        let mut s = Store::open(&path).unwrap();
        let empty = s.revision().unwrap();
        assert_eq!((empty.response, empty.facts_version), (0, 7));
        let mut last = empty.response;
        fn moved(s: &Store, last: &mut i64, what: &str) {
            let now = s.revision().unwrap().response;
            assert!(now > *last, "{what}: {last} -> {now}");
            *last = now;
        }
        profile(&mut s);
        moved(&s, &mut last, "profile");
        list_tabs(&mut s, "pc", "Standard", json!([tab("t1"), tab("t2")]), 10);
        moved(&s, &mut last, "a listing");
        fetch_tab(&mut s, "pc", "Standard", "t2", vec![item("a", "A")], 20);
        moved(&s, &mut last, "a fetch");
        list_tabs(&mut s, "pc", "Standard", json!([tab("t1")]), 30);
        moved(&s, &mut last, "a listing that retires a tab and its item");
        fetch_tab(&mut s, "pc", "Standard", "t2", vec![item("a", "A")], 40);
        moved(&s, &mut last, "a withheld fetch");

        let held = read_all(&s, RealmScope::All);
        let refused = s.record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t1".into(),
                sub: None,
            },
            &json!({ "id": "t1" }),
            200,
            &json!({ "stash": { "id": "t1", "items": [{ "name": "no id" }] } }),
            50,
        );
        assert!(refused.is_err());
        s.rebuild().unwrap();
        drop(s);
        let mut s = Store::open(&path).unwrap();
        assert_eq!(s.revision().unwrap().response, last);
        assert_eq!(read_all(&s, RealmScope::All), held);

        fetch_tab(&mut s, "pc", "Standard", "t1", vec![item("b", "B")], 60);
        moved(&s, &mut last, "the record after a refusal");
    }

    /// C108 — the read hands over ingest facts and the body as stored: no
    /// derived column reaches it or orders it, and a body that does not
    /// parse arrives as it is.
    #[test]
    fn derived_columns_never_reach_the_read_and_a_bad_body_arrives_as_text() {
        let mut s = Store::open_memory().unwrap();
        profile(&mut s);
        list_tabs(&mut s, "pc", "Standard", json!([tab("t1")]), 10);
        let at = |id: &str, x: i64| {
            let mut i = item(id, id);
            i["x"] = json!(x);
            i
        };
        fetch_tab(
            &mut s,
            "pc",
            "Standard",
            "t1",
            vec![at("m", 5), at("k", 9), at("q", 1)],
            20,
        );
        let before = read_all(&s, RealmScope::All);
        assert_eq!(ids(&before.1), ["k", "m", "q"]);
        s.conn
            .execute("UPDATE items SET name = 'wrong', x = 100 - x, y = x", [])
            .unwrap();
        assert_eq!(read_all(&s, RealmScope::All), before);

        s.conn
            .execute("UPDATE items SET json = 'not json' WHERE id = 'm'", [])
            .unwrap();
        let (_, items) = read_all(&s, RealmScope::All);
        assert_eq!(items[1].body, "not json");
    }

    /// C108 — a facts file that records no account is refused, as the
    /// snapshots refuse it: the basis names the account (C98).
    #[test]
    fn a_store_with_no_account_on_record_is_refused() {
        let s = Store::open_memory().unwrap();
        let err = s
            .read_corpus(RealmScope::All, |_, _| Ok(()))
            .unwrap_err()
            .to_string();
        assert!(err.contains("records no account identity"), "{err}");
    }
}
