//! The held corpus (C98, C103): every live item of a scope, derived once
//! from one snapshot of the store's read (C108), with the basis that
//! snapshot is labelled by and the places the items sit at.
//!
//! # As built
//!
//! - **One read, one snapshot.** [`Corpus::load`] is one
//!   `Store::read_corpus`: the revision, the locations and every body
//!   describe one state, and each body is derived as it streams past, so
//!   no body outlives its item. Nothing is persisted and nothing is cached
//!   between processes (C98, C48).
//! - **The basis** names the store, the account, the facts revision — the
//!   highest response id and the facts schema beside it — and the
//!   derivation's version ([`DERIVATION`]). The store is named by twelve
//!   hex digits of the SHA-256 of its file's canonical path, as C83 names
//!   a world (owner, 2026-09-20: "(a') now and park (c)"): no path in an
//!   answer, no migration, and two files of one account are two stores. A
//!   file moved is another store, which costs a reload; an id the file
//!   itself carries is parked (`decisions/search.md`). Stores that are no
//!   file — a test's, in memory — share one name. The intent revision joins it with price (the
//!   build plan, step 9). [`Corpus::is_current`] is C98's check before
//!   every answer: a consumer that holds a corpus across asks compares, and
//!   reloads whole when it differs; an answer already given stays what its
//!   basis says it was.
//! - **Place is the store's** (C103): the league as the read joined it, the
//!   location by its full coordinate (C54), its name and its parent's from
//!   the header. An item whose location the header does not list cannot
//!   occur — the read hands over live items at live locations — and would
//!   carry its ids and no names.
//! - **Coverage is stated from the header**: a folder is a row no fetch
//!   fills and is counted apart; a location never fetched is one no item
//!   predicate supports a claim about (invariant 6 of the surface).

use std::collections::HashMap;

use acquisition_store::Store;
use acquisition_store::corpus::{CorpusHeader, CorpusItem, LocationRow, RealmScope, Revision};
use serde::{Deserialize, Serialize};

use crate::derive::{Facts, Item, derive};
use crate::error::SearchError;

/// The version of [`derive()`]'s reading of a body: it moves when the same
/// body would derive to another item.
pub const DERIVATION: u32 = 3;

/// The realms GGG has (owner, 2026-09-20: "the full list").
pub const REALMS: [&str; 4] = ["pc", "xbox", "sony", "poe2"];

/// Which realms a request covers (C96): the scope, never a term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Realm {
    All,
    One(String),
}

impl Serialize for Realm {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Realm {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Realm, D::Error> {
        let word = String::deserialize(deserializer)?;
        Realm::parse(&word).map_err(serde::de::Error::custom)
    }
}

impl Realm {
    /// A realm word as a request or a terminal spells it.
    pub fn parse(word: &str) -> Result<Realm, SearchError> {
        if word.eq_ignore_ascii_case("all") {
            return Ok(Realm::All);
        }
        match REALMS.iter().find(|r| r.eq_ignore_ascii_case(word)) {
            Some(realm) => Ok(Realm::One(realm.to_string())),
            None => Err(SearchError::scope(
                "realm_unknown",
                format!("`{word}` is no realm: {}, or all", REALMS.join(", ")),
            )),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Realm::All => "all",
            Realm::One(realm) => realm,
        }
    }
}

/// What an answer is labelled with (C98).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Basis {
    /// The facts file, by a short hash of its canonical path.
    pub store: String,
    pub account: String,
    /// The facts revision: the highest response id, and the facts schema.
    pub snapshot: Revision,
    pub derivation: u32,
}

/// Where an item sits, by name and id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Place {
    pub realm: String,
    pub league: Option<String>,
    /// `stash` or `character`.
    pub kind: String,
    pub id: String,
    pub name: Option<String>,
    /// A substash's tab.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Named>,
    pub container: Option<String>,
    /// The item this one is socketed in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socketed_in: Option<Named>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Named {
    pub id: String,
    pub name: Option<String>,
}

/// One held item: what the body says, and where it is.
#[derive(Debug, Clone)]
pub struct Held {
    pub item: Item,
    pub place: Place,
}

/// What the scope block says was searched (C96).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Coverage {
    pub items: usize,
    /// Every realm the store holds, whatever the scope.
    pub realms_held: Vec<String>,
    pub leagues: Vec<String>,
    /// Locations that hold items and have been fetched.
    pub fetched: usize,
    /// Locations a listing names and no fetch has filled, folders apart.
    pub never_fetched: usize,
    pub folders: usize,
    pub oldest_fetch: Option<i64>,
    pub newest_fetch: Option<i64>,
    /// When the oldest of the scope's location lists was last seen.
    pub list_seen: Option<i64>,
}

#[derive(Debug)]
pub struct Corpus {
    pub basis: Basis,
    pub account_name: Option<String>,
    pub realm: Realm,
    pub coverage: Coverage,
    pub(crate) items: Vec<Held>,
}

type Coordinate = (String, String, String, Option<String>);

fn coordinate(realm: &str, kind: &str, id: &str, league: Option<&str>) -> Coordinate {
    // a tab's coordinate carries its league; a character's is its realm's
    let league = (kind == "stash").then(|| league.unwrap_or_default().to_string());
    (realm.to_string(), kind.to_string(), id.to_string(), league)
}

pub(crate) type Locations<'a> = HashMap<Coordinate, &'a LocationRow>;

pub(crate) fn locations(header: &CorpusHeader) -> Locations<'_> {
    header
        .locations
        .iter()
        .map(|l| (coordinate(&l.realm, &l.kind, &l.id, l.league.as_deref()), l))
        .collect()
}

/// One row of the store's read as the deriver's facts and the item's
/// place, by name and id. What an item is socketed in is named by
/// [`Corpus::load`], which holds every item.
pub(crate) fn placed(index: &Locations<'_>, row: CorpusItem) -> (Facts, Place) {
    let at = |kind: &str, id: &str| {
        index
            .get(&coordinate(&row.realm, kind, id, row.league.as_deref()))
            .copied()
    };
    let location = at(&row.location_kind, &row.location_id);
    let place = Place {
        realm: row.realm.clone(),
        league: row.league.clone(),
        kind: row.location_kind.clone(),
        id: row.location_id.clone(),
        name: location.map(|l| l.name.clone()),
        parent: location
            .filter(|l| l.kind == "stash")
            .and_then(|l| l.parent.as_deref())
            .and_then(|parent| at("stash", parent))
            // a tab's parent is its folder: only a substash's parent is
            // where an item sits
            .filter(|p| !is_folder(p))
            .map(|p| Named {
                id: p.id.clone(),
                name: Some(p.name.clone()),
            }),
        container: row.container.clone(),
        socketed_in: row.socketed_in.clone().map(|id| Named { id, name: None }),
    };
    let facts = Facts {
        id: row.id,
        realm: row.realm,
        league: row.league,
        location_kind: row.location_kind,
        location_id: row.location_id,
        container: row.container,
        socketed_in: row.socketed_in,
        first_seen: row.first_seen,
        last_seen: row.last_seen,
        removed_at: None,
    };
    (facts, place)
}

impl Basis {
    /// The basis of what one read of `store` handed over.
    pub(crate) fn of(store: &Store, header: &CorpusHeader) -> Basis {
        use sha2::{Digest as _, Sha256};
        let path = store.path();
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let digest = Sha256::digest(path.as_os_str().as_encoded_bytes());
        Basis {
            store: digest.iter().take(6).map(|b| format!("{b:02x}")).collect(),
            account: header.account_uuid.clone(),
            snapshot: header.revision,
            derivation: DERIVATION,
        }
    }
}

impl Corpus {
    /// Read and derive every live item of `realm` under one snapshot. A
    /// named realm the store does not hold is a valid scope of no items; no
    /// realm named is resolved against what the store holds (C96): the one
    /// it holds, all when it holds none, and an error listing them when it
    /// holds several.
    pub fn load(store: &Store, realm: Option<&Realm>) -> Result<Corpus, SearchError> {
        let wanted = match realm {
            Some(Realm::One(realm)) => Some(realm.clone()),
            _ => None,
        };
        let scope = match &wanted {
            Some(realm) => RealmScope::One(realm),
            None => RealmScope::All,
        };
        let named = realm.cloned();
        store
            .read_corpus(scope, |header, rows| {
                let realm = match named {
                    Some(realm) => realm,
                    None => match header.realms.as_slice() {
                        [] => Realm::All,
                        [one] => Realm::One(one.clone()),
                        // no row has been stepped to: the error costs no read
                        several => return Ok(Err(realm_needed(several))),
                    },
                };
                let index = locations(header);
                let mut items = Vec::new();
                for row in rows {
                    let mut row = row?;
                    // no realm named over a store holding one reads every
                    // realm, which is that one: the filter then drops nothing
                    if let Realm::One(only) = &realm
                        && row.realm != *only
                    {
                        continue;
                    }
                    let body = std::mem::take(&mut row.body);
                    let (facts, place) = placed(&index, row);
                    items.push(Held {
                        item: derive(facts, &body),
                        place,
                    });
                }
                name_sockets(&mut items);
                let locations: Vec<LocationRow> = header
                    .locations
                    .iter()
                    .filter(|l| match &realm {
                        Realm::One(only) => l.realm == *only,
                        Realm::All => true,
                    })
                    .cloned()
                    .collect();
                Ok(Ok(Corpus {
                    basis: Basis::of(store, header),
                    account_name: header.account_name.clone(),
                    coverage: coverage(header, &realm, &locations, items.len()),
                    realm,
                    items,
                }))
            })
            .map_err(SearchError::store)?
    }

    /// C98's check before every answer: whether this is the store the
    /// corpus was read from, and its facts still stand at that revision.
    pub fn is_current(&self, store: &Store) -> Result<bool, SearchError> {
        let now = store
            .read_corpus(RealmScope::All, |header, _| Ok(Basis::of(store, header)))
            .map_err(SearchError::store)?;
        Ok(now == self.basis)
    }

    pub fn items(&self) -> &[Held] {
        &self.items
    }
}

fn realm_needed(held: &[String]) -> SearchError {
    SearchError::scope(
        "realm_needed",
        format!(
            "this store holds {} realms and a search names one (C96): --realm {}, or --realm all",
            held.len(),
            held.join(" | ")
        ),
    )
    .with_offers(held.iter().map(|r| format!("--realm {r}")).collect())
}

pub(crate) fn is_folder(location: &LocationRow) -> bool {
    location.tab_type.as_deref() == Some("Folder")
}

/// What an item is socketed in, by the name it is found under.
fn name_sockets(items: &mut [Held]) {
    let names: HashMap<String, Option<String>> = items
        .iter()
        .filter(|h| h.place.socketed_in.is_none())
        .map(|h| {
            let item = &h.item;
            let name = match (&item.name, &item.typeline) {
                (Some(name), Some(typeline)) => Some(format!("{name} {typeline}")),
                (name, typeline) => name.clone().or_else(|| typeline.clone()),
            };
            (item.facts.id.clone(), name)
        })
        .collect();
    for held in items {
        if let Some(parent) = &mut held.place.socketed_in {
            parent.name = names.get(&parent.id).cloned().flatten();
        }
    }
}

fn coverage(
    header: &CorpusHeader,
    realm: &Realm,
    locations: &[LocationRow],
    items: usize,
) -> Coverage {
    let holds_items: Vec<&LocationRow> = locations.iter().filter(|l| !is_folder(l)).collect();
    let fetches = || holds_items.iter().filter_map(|l| l.fetched_at);
    let mut leagues: Vec<String> = locations.iter().filter_map(|l| l.league.clone()).collect();
    leagues.sort();
    leagues.dedup();
    Coverage {
        items,
        realms_held: header.realms.clone(),
        leagues,
        fetched: fetches().count(),
        never_fetched: holds_items.len() - fetches().count(),
        folders: locations.len() - holds_items.len(),
        oldest_fetch: fetches().min(),
        newest_fetch: fetches().max(),
        list_seen: header
            .listings
            .iter()
            .filter(|l| match realm {
                Realm::One(only) => l.realm == *only,
                Realm::All => true,
            })
            .map(|l| l.fetched_at)
            .min(),
    }
}
