//! The held corpus (C98, C103): every item of a scope — the live ones,
//! under membership `all` the removed beside them, or under `removed`
//! the removed alone — derived once
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
//! - **The basis** names the store, the account, the build — a digest
//!   of the sources an answer is computed by (`build.rs`; B1,
//!   `search/LEDGER.md`), so that two builds never label different
//!   answers with one basis, as every evaluator fix did while the
//!   versions below were all that named one — the facts revision — the
//!   highest response id, when it was recorded and the facts schema
//!   beside them — the
//!   derivation's version ([`DERIVATION`]), the class table's
//!   (`class.rs`; C106), the category table's and the beast colour
//!   table's (`category.rs`; C106) and the totals table's (`totals.rs`; C94). The store is named by twelve
//!   hex digits of the SHA-256 of its file's canonical path, as C83 names
//!   a world (owner, 2026-09-20: "(a') now and park (c)"): no path in an
//!   answer, no migration, and two files of one account are two stores. A
//!   file moved is another store, which costs a reload; a file
//!   deleted and refetched at one path is told by when its highest
//!   response was recorded (the store's `corpus.rs`). Stores that are no
//!   file — a test's, in memory — share one name. The intent file's
//!   revision joins it with the price (`price.rs`; the build plan, step
//!   9), with the currency table's and the note parser's versions the
//!   listing state was resolved under. [`Corpus::is_current`] is C98's
//!   check before every answer: a consumer that holds a corpus across asks
//!   compares, and reloads whole when it differs; an answer already given
//!   stays what its basis says it was.
//! - **Each item's price is the listing state's** (`price.rs`, C81, C100):
//!   after the one read, one pricing snapshot and one `resolve` for each
//!   (realm, league) the scope's locations name, joined by item id. Those
//!   snapshots are reads of their own, so the facts revision and the intent
//!   revision are read again after them and the whole corpus is read again
//!   where either moved — three times, then an error — so no answer is
//!   labelled with a basis its prices were not read at.
//! - **Each item is classed as it is derived** (`class.rs`), and its
//!   category computed from that class beside it (`category.rs`): the
//!   tables are checked before the read, so a build whose table does not
//!   load answers nothing rather than an unclassed corpus; the totals
//!   table is checked with it, so a total is never asked of a corpus over a table that
//!   does not load.
//! - **Place is the store's** (C103): the league as the read joined it, the
//!   location by its full coordinate (C54), its name, its parent's and
//!   its tab's `type` (V10: `tab.type`, verbatim, a substash's own or its
//!   tab's) from the header. An item whose location the header does not
//!   list cannot occur — the read hands over live items at live
//!   locations, and under `all` the retired locations with the removed
//!   items — and would carry its ids and no names.
//! - **Membership is the read's** (C108; the reference, *Membership*):
//!   a corpus holds the live items, or all of them, a removed one at the
//!   place it was last seen with when it left (`Facts::removed_at`) and a
//!   price that is lacked (`price.rs`) — or the removed alone, which are
//!   the removed of the store's read of all: the live rows of that read
//!   are counted and passed over, never derived. Coverage and the listing
//!   states are the live locations' whichever it holds, and the scope's
//!   items are the live ones, the removed counted beside them. Under `live` the corpus
//!   keeps the ids the read left out, each with when it left, for an
//!   `id:` term that matches nothing (`answer.rs`).
//! - **Coverage is stated from the header**: a folder is a row no fetch
//!   fills and is counted apart; a location never fetched is one no item
//!   predicate supports a claim about (invariant 6 of the surface).

use std::collections::HashMap;

use acquisition_store::corpus::{CorpusHeader, CorpusItem, LocationRow, RealmScope, Revision};
use acquisition_store::{Annotations, Store};
use serde::{Deserialize, Serialize};

use crate::answer::Membership;
use crate::category::{self, CATEGORY_TABLE_VERSION, COLOUR_TABLE_VERSION, Category};
use crate::class::{self, CLASS_TABLE_VERSION, ClassTable, Classed};
use crate::defences::{self, DEFENCE_TABLE_VERSION};
use crate::derive::{Facts, Item, Part, Unread, derive};
use crate::error::SearchError;
use crate::price::{self, CURRENCY_TABLE_VERSION, NOTE_PARSER_VERSION, Priced};
use crate::totals::{self, TOTALS_TABLE_VERSION, TotalsTable};

/// The version of [`derive()`]'s reading of a body: it moves when the same
/// body would derive to another item.
pub const DERIVATION: u32 = 11;

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
    /// The build that answered: a digest of the sources an answer is
    /// computed by (`build.rs`).
    pub build: String,
    /// The facts revision: the highest response id, when it was recorded,
    /// and the facts schema.
    pub snapshot: Revision,
    pub derivation: u32,
    /// The class table's version (C106).
    pub classes: u32,
    /// The category table's version (C106; `category.rs`).
    pub categories: u32,
    /// The beast colour table's version (C106; `category.rs`).
    pub beast_colours: u32,
    /// The totals table's version (C94).
    pub totals: u32,
    /// The defence table's version (C101, C106; `defences.rs`).
    pub defences: u32,
    /// The intent file's revision the prices were read at (C98, C81).
    pub intent: i64,
    /// The currency table's version (C68).
    pub currency: u32,
    /// The note parser's version (C69).
    pub notes: u32,
}

/// An epoch as the time it is, to the second, in UTC: what a message
/// says where the JSON beside it carries the epoch (C53). The civil
/// date is counted from days since 1970 in the proleptic Gregorian
/// calendar, 400 years at a time.
pub(crate) fn utc(epoch: i64) -> String {
    let (days, second) = (epoch.div_euclid(86_400), epoch.rem_euclid(86_400));
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_from_march = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
    let month = if month_from_march < 10 {
        month_from_march + 3
    } else {
        month_from_march - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        second / 3_600,
        second % 3_600 / 60,
        second % 60
    )
}

impl Basis {
    /// What differs between this basis and another, each part named
    /// with both its values: what a refusal across a change says.
    pub fn against(&self, now: &Basis) -> String {
        let parts: [(&str, String, String); 14] = [
            ("store", self.store.clone(), now.store.clone()),
            ("account", self.account.clone(), now.account.clone()),
            ("build", self.build.clone(), now.build.clone()),
            (
                "snapshot",
                self.snapshot.response.to_string(),
                now.snapshot.response.to_string(),
            ),
            ("recorded", utc(self.snapshot.at), utc(now.snapshot.at)),
            (
                "facts v",
                self.snapshot.facts_version.to_string(),
                now.snapshot.facts_version.to_string(),
            ),
            ("intent", self.intent.to_string(), now.intent.to_string()),
            (
                "derivation",
                self.derivation.to_string(),
                now.derivation.to_string(),
            ),
            (
                "classes v",
                self.classes.to_string(),
                now.classes.to_string(),
            ),
            (
                "categories v",
                self.categories.to_string(),
                now.categories.to_string(),
            ),
            (
                "beast colours v",
                self.beast_colours.to_string(),
                now.beast_colours.to_string(),
            ),
            ("totals v", self.totals.to_string(), now.totals.to_string()),
            (
                "defences v",
                self.defences.to_string(),
                now.defences.to_string(),
            ),
            (
                "currency v",
                self.currency.to_string(),
                now.currency.to_string(),
            ),
        ];
        let mut differing: Vec<String> = parts
            .iter()
            .filter(|(_, then, now)| then != now)
            .map(|(name, then, now)| {
                let gap = if name.ends_with('v') { "" } else { " " };
                format!("{name}{gap}{then} then, {name}{gap}{now} now")
            })
            .collect();
        if self.notes != now.notes {
            differing.push(format!(
                "notes v{} then, notes v{} now",
                self.notes, now.notes
            ));
        }
        differing.join("; ")
    }
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
    /// GGG's `type` of the tab, verbatim, as the store's read hands it
    /// over (C108); a substash's own, or its tab's where it has none; a
    /// character has none (the field `tab.type`, V10).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tab_type: Option<String>,
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

/// One held item: what the body says, where it is, what the class table
/// says of it, what the category ids answer of it, and what the listing
/// state prices it at.
#[derive(Debug, Clone)]
pub struct Held {
    pub item: Item,
    pub place: Place,
    pub class: Classed,
    pub category: Category,
    pub price: Priced,
}

/// What the scope block says was searched (C96).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Coverage {
    /// The live items in scope.
    pub items: usize,
    /// Under membership `all` and `removed`, the removed items: each is
    /// a row marked with when it left.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<usize>,
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
    /// Which items it holds: the live ones, every item with the removed
    /// marked, or the removed alone (the reference, *Membership*).
    pub membership: Membership,
    /// The ids the scope held and no item of this corpus answers to: under
    /// `live`, an item removed and a tab or a character retired; under
    /// `all` and `removed`, where the removed items are rows, the retired
    /// locations alone. Each with what it was and when it left; never an id that is
    /// also a live location's, and the latest time where two rows share
    /// one. What an `id:` that matches nothing is looked up in.
    pub(crate) gone: HashMap<String, Left>,
    pub(crate) items: Vec<Held>,
}

/// What an id that left was, and when it left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Left {
    /// `item`, `stash` or `character`, as the store spells them.
    pub kind: String,
    pub removed_at: i64,
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
    let tab = location
        .filter(|l| l.kind == "stash")
        .and_then(|l| l.parent.as_deref())
        .and_then(|parent| at("stash", parent))
        // a tab's parent is its folder: only a substash's parent is
        // where an item sits
        .filter(|p| !is_folder(p));
    let place = Place {
        realm: row.realm.clone(),
        league: row.league.clone(),
        kind: row.location_kind.clone(),
        id: row.location_id.clone(),
        name: location.map(|l| l.name.clone()),
        tab_type: location
            .filter(|l| l.kind == "stash")
            .and_then(|l| l.tab_type.clone())
            .or_else(|| tab.and_then(|t| t.tab_type.clone())),
        parent: tab.map(|p| Named {
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
        removed_at: row.removed_at,
    };
    (facts, place)
}

impl Basis {
    /// The basis of what one read of `store` handed over, with the intent
    /// revision the prices were read at.
    pub(crate) fn of(store: &Store, header: &CorpusHeader, intent: i64) -> Basis {
        use sha2::{Digest as _, Sha256};
        let path = store.path();
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let digest = Sha256::digest(path.as_os_str().as_encoded_bytes());
        Basis {
            store: digest.iter().take(6).map(|b| format!("{b:02x}")).collect(),
            account: header.account_uuid.clone(),
            build: env!("ACQ_SEARCH_BUILD").to_string(),
            snapshot: header.revision,
            derivation: DERIVATION,
            classes: CLASS_TABLE_VERSION,
            categories: CATEGORY_TABLE_VERSION,
            beast_colours: COLOUR_TABLE_VERSION,
            totals: TOTALS_TABLE_VERSION,
            defences: DEFENCE_TABLE_VERSION,
            intent,
            currency: CURRENCY_TABLE_VERSION,
            notes: NOTE_PARSER_VERSION,
        }
    }
}

/// The class table, or the error a build whose table does not load
/// answers with (`class.rs`).
pub(crate) fn class_table() -> Result<&'static ClassTable, SearchError> {
    class::table().map_err(|e| SearchError::scope("class_table", e.to_string()))
}

/// The category tables, or the error a build whose tables do not load
/// answers with (`category.rs`).
pub(crate) fn category_tables() -> Result<&'static category::Tables, SearchError> {
    category::tables().map_err(|e| SearchError::scope("category_table", e.to_string()))
}

/// The totals table, or the error a build whose table does not load
/// answers with (`totals.rs`).
pub(crate) fn totals_table() -> Result<&'static TotalsTable, SearchError> {
    totals::table().map_err(|e| SearchError::scope("totals_table", e.to_string()))
}

/// The defence table, or the error a build whose table does not load
/// answers with (`defences.rs`).
pub(crate) fn defence_table() -> Result<&'static defences::DefenceTable, SearchError> {
    defences::table().map_err(|e| SearchError::scope("defence_table", e.to_string()))
}

/// How many times a corpus is read again when the facts or the intent
/// moved while its prices were being joined, before that is an error.
pub(crate) const REREADS: usize = 3;

impl Corpus {
    /// Read and derive every live item of `realm` under one snapshot, and
    /// join each item's price from the listing state (`price.rs`). A named
    /// realm the store does not hold is a valid scope of no items; no realm
    /// named is resolved against what the store holds (C96): the one it
    /// holds, all when it holds none, and an error listing them when it
    /// holds several.
    pub fn load(
        store: &Store,
        intent: &Annotations,
        realm: Option<&Realm>,
    ) -> Result<Corpus, SearchError> {
        Self::load_of(store, intent, realm, Membership::Live)
    }

    /// [`Corpus::load`] over a membership: under `all`, every item the
    /// file holds at the place it was last seen, the removed marked;
    /// under `removed`, those alone.
    pub fn load_of(
        store: &Store,
        intent: &Annotations,
        realm: Option<&Realm>,
        membership: Membership,
    ) -> Result<Corpus, SearchError> {
        for _ in 0..REREADS {
            if let Some(corpus) = Self::read(store, intent, realm, membership)? {
                return Ok(corpus);
            }
        }
        Err(moved())
    }

    /// One read: the corpus, or nothing where the facts or the intent moved
    /// before its prices were joined.
    fn read(
        store: &Store,
        intent: &Annotations,
        realm: Option<&Realm>,
        membership: Membership,
    ) -> Result<Option<Corpus>, SearchError> {
        let wanted = match realm {
            Some(Realm::One(realm)) => Some(realm.clone()),
            _ => None,
        };
        let scope = match &wanted {
            Some(realm) => RealmScope::One(realm),
            None => RealmScope::All,
        };
        let named = realm.cloned();
        let table = class_table()?;
        let categories = category_tables()?;
        totals_table()?;
        defence_table()?;
        currency_table()?;
        let intent_revision = price::intent_revision(intent)?;
        let (mut corpus, leagues) =
            store
                .read_corpus_of(scope, membership.read(), |header, rows| {
                    let realm = match named {
                        Some(realm) => realm,
                        None => match header.realms.as_slice() {
                            [] => Realm::All,
                            [one] => Realm::One(one.clone()),
                            // no row has been stepped to: the error costs no read
                            several => {
                                return Ok(Err(realm_needed(
                                    several,
                                    header
                                        .account_name
                                        .as_deref()
                                        .unwrap_or(&header.account_uuid),
                                )));
                            }
                        },
                    };
                    let index = locations(header);
                    let mut items = Vec::new();
                    // under `removed`, the live items the read hands over:
                    // counted for the scope block and never derived
                    let mut passed_over = 0usize;
                    for row in rows {
                        let mut row = row?;
                        // no realm named over a store holding one reads every
                        // realm, which is that one: the filter then drops nothing
                        if let Realm::One(only) = &realm
                            && row.realm != *only
                        {
                            continue;
                        }
                        if membership == Membership::Removed && row.removed_at.is_none() {
                            passed_over += 1;
                            continue;
                        }
                        let body = std::mem::take(&mut row.body);
                        let (facts, place) = placed(&index, row);
                        let mut item = derive(facts, &body);
                        item.unread.extend(tab_type_unread(&place));
                        let class = table.classify(&item);
                        items.push(Held {
                            category: categories.of(&item, &class),
                            class,
                            item,
                            place,
                            // joined below, once the read is done
                            price: Priced::open(price::PriceGap::Uncovered, "not joined yet"),
                        });
                    }
                    name_sockets(&mut items);
                    // coverage and the listing states are the live locations':
                    // a retired one is in the header only to name what left
                    let locations: Vec<LocationRow> = header
                        .locations
                        .iter()
                        .filter(|l| l.removed_at.is_none())
                        .filter(|l| match &realm {
                            Realm::One(only) => l.realm == *only,
                            Realm::All => true,
                        })
                        .cloned()
                        .collect();
                    let removed = items
                        .iter()
                        .filter(|h| h.item.facts.removed_at.is_some())
                        .count();
                    let in_realm = |of: &str| match &realm {
                        Realm::One(only) => of == only,
                        Realm::All => true,
                    };
                    let mut gone: HashMap<String, Left> = HashMap::new();
                    let left =
                        header
                            .gone
                            .iter()
                            .map(|g| (&g.realm, &g.kind, &g.id, g.removed_at))
                            .chain(header.locations.iter().filter_map(|l| {
                                l.removed_at.map(|at| (&l.realm, &l.kind, &l.id, at))
                            }));
                    for (of, kind, id, removed_at) in left {
                        let live = locations.iter().any(|l| l.id == *id);
                        let later = gone.get(id).is_some_and(|l| l.removed_at >= removed_at);
                        if in_realm(of) && !live && !later {
                            gone.insert(
                                id.clone(),
                                Left {
                                    kind: kind.clone(),
                                    removed_at,
                                },
                            );
                        }
                    }
                    // the leagues the scope's items sit in: under `all`, a
                    // league whose every location is retired among them
                    let mut sat_in: Vec<String> = header
                        .locations
                        .iter()
                        .filter(|l| in_realm(&l.realm))
                        .filter_map(|l| l.league.clone())
                        // a removed character item sits in the league it left in,
                        // which its character may have left since
                        .chain(
                            items
                                .iter()
                                .filter(|h| h.item.facts.removed_at.is_some())
                                .filter_map(|h| h.place.league.clone()),
                        )
                        .collect();
                    sat_in.sort();
                    sat_in.dedup();
                    let leagues = price::leagues(&locations);
                    Ok(Ok((
                        Corpus {
                            basis: Basis::of(store, header, intent_revision),
                            account_name: header.account_name.clone(),
                            coverage: Coverage {
                                removed: (membership != Membership::Live).then_some(removed),
                                leagues: sat_in,
                                ..coverage(
                                    header,
                                    &realm,
                                    &locations,
                                    items.len() - removed + passed_over,
                                )
                            },
                            realm,
                            membership,
                            gone,
                            items,
                        },
                        leagues,
                    )))
                })
                .map_err(SearchError::store)??;
        let mut prices = price::join(store, intent, &leagues)?;
        if !still_at(store, intent, &corpus.basis)? {
            return Ok(None);
        }
        for held in &mut corpus.items {
            held.price = match held.item.facts.removed_at {
                // an item not in the stash has no listing state
                Some(_) => Priced::removed(),
                None => price::of_item(&mut prices, &held.item.facts.id),
            };
        }
        Ok(Some(corpus))
    }

    /// C98's check before every answer: whether this is the store the
    /// corpus was read from, its facts still stand at that revision, and
    /// the intent file at its.
    pub fn is_current(&self, store: &Store, intent: &Annotations) -> Result<bool, SearchError> {
        let intent_revision = price::intent_revision(intent)?;
        let now = store
            .read_corpus(RealmScope::All, |header, _| {
                Ok(Basis::of(store, header, intent_revision))
            })
            .map_err(SearchError::store)?;
        Ok(now == self.basis)
    }

    pub fn items(&self) -> &[Held] {
        &self.items
    }
}

/// Whether the facts and the intent still stand at the basis: what the
/// prices were joined after, checked before the corpus is handed over.
pub(crate) fn still_at(
    store: &Store,
    intent: &Annotations,
    basis: &Basis,
) -> Result<bool, SearchError> {
    let facts = store.revision().map_err(SearchError::store)?;
    Ok(facts == basis.snapshot && price::intent_revision(intent)? == basis.intent)
}

/// The store or the intent file kept moving while a corpus was read.
pub(crate) fn moved() -> SearchError {
    SearchError::scope(
        "basis_moved",
        format!(
            "the facts or the intent file changed {REREADS} times while a corpus was being read: a refresh or a price write is under way — ask again"
        ),
    )
}

/// The currency table, or the error a build whose table does not load
/// answers with (`price.rs`).
pub(crate) fn currency_table() -> Result<(), SearchError> {
    price::currency_table().map_err(|e| SearchError::scope("currency_table", e))
}

fn realm_needed(held: &[String], account: &str) -> SearchError {
    SearchError::scope(
        "realm_needed",
        format!(
            "this store holds {} realms and a search names one (C96): --realm {}, or --realm all",
            held.len(),
            held.join(" | ")
        ),
    )
    .offering_realms(held, account)
}

/// What the store's read could not give of a stash tab's type, said once
/// where the place is read (rule 8 of the plan): GGG's `type` that was no
/// string is stored as none, and every stash tab has one, so none on a
/// stash tab is unread — never the known absence a character's item has
/// (the review of 9b, 2026-09-26).
pub(crate) fn tab_type_unread(place: &Place) -> Option<Unread> {
    (place.kind == "stash" && place.tab_type.is_none()).then(|| Unread {
        part: Part::TabType,
        problem: format!(
            "the store holds no type for tab {}: GGG's `type` was no string when it was listed",
            place.id
        ),
        line: None,
        name: None,
        socket: None,
    })
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
        removed: None,
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

#[cfg(test)]
mod tests {
    use super::utc;

    /// The calendar is counted by hand, so it is held to dates worked
    /// outside it (Python's `datetime`): the epoch and the second before
    /// it, a leap day and the days around one, the century that is no
    /// leap year, a year's last second, and the year 2400.
    #[test]
    fn an_epoch_is_said_as_its_time_in_utc() {
        for (epoch, said) in [
            (0, "1970-01-01 00:00:00 UTC"),
            (-1, "1969-12-31 23:59:59 UTC"),
            (20, "1970-01-01 00:00:20 UTC"),
            (951782400, "2000-02-29 00:00:00 UTC"),
            (951868799, "2000-02-29 23:59:59 UTC"),
            (1709251199, "2024-02-29 23:59:59 UTC"),
            (1709251200, "2024-03-01 00:00:00 UTC"),
            (1760000000, "2025-10-09 08:53:20 UTC"),
            (4107542399, "2100-02-28 23:59:59 UTC"),
            (4107542400, "2100-03-01 00:00:00 UTC"),
            (1798761599, "2026-12-31 23:59:59 UTC"),
            (1798761600, "2027-01-01 00:00:00 UTC"),
            (13569465600, "2400-01-01 00:00:00 UTC"),
        ] {
            assert_eq!(utc(epoch), said, "{epoch}");
        }
    }
}
