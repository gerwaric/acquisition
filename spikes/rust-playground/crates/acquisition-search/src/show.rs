//! `show <id>` (C100, C103; the reference's `acq show`): one item as the
//! search derives it — its fields, its place, every line with its kind,
//! template and numbers, what was unread — and the stored body on request,
//! which is still there when deriving fails.
//!
//! # As built
//!
//! The item is found by the search's own read (C108), every realm, in one
//! pass that derives the one item it is looking for: so its place is
//! joined as every answer joins it (S131), its basis is the snapshot it was
//! read from, and its body is the text as stored — `Store::item` parses
//! the body, turns one that does not parse into a null, and carries the
//! league the item was stamped with. What the item is socketed in is given
//! by id, which `show` accepts in turn. That read hands over live items
//! only: an id the store holds and the read does not is said to be
//! removed, and an id that names a tab or a character is said to be one,
//! with the search that lists what is in it. The item's price is the
//! listing state's (`price.rs`, C81), joined after the read as a corpus
//! joins every item's, under the same check that the facts and the intent
//! still stand at the basis.

use acquisition_store::corpus::{LocationRow, RealmScope};
use acquisition_store::{Annotations, Store};
use serde::Serialize;

use crate::answer::command;
use crate::class::Classed;
use crate::corpus::{
    Basis, Place, REREADS, class_table, currency_table, locations, moved, placed, still_at,
};
use crate::derive::{Facts, Item, Line, derive};
use crate::describe::limit;
use crate::error::SearchError;
use crate::eval::number_json;
use crate::price::{self, Priced};

#[derive(Debug, Clone, Serialize)]
pub struct Shown {
    pub basis: Basis,
    pub place: Place,
    /// The item without its lines, which follow with their slots.
    pub item: Item,
    /// What the class table says of it (`class.rs`).
    pub class: Classed,
    /// What the listing state prices it at (`price.rs`, C81).
    pub price: Priced,
    pub lines: Vec<ShownLine>,
    /// The stored body as text, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShownLine {
    #[serde(flatten)]
    pub line: Line,
    /// The words that name this occurrence's numbers, each with its number.
    pub slots: Vec<(String, serde_json::Value)>,
    /// A limit this line meets, in the register's words (C102).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<&'static str>,
}

/// One live item by id, derived, with its body when `body` is set.
pub fn show(
    store: &Store,
    intent: &Annotations,
    id: &str,
    body: bool,
) -> Result<Shown, SearchError> {
    for _ in 0..REREADS {
        if let Some(shown) = read_one(store, intent, id, body)? {
            return Ok(shown);
        }
    }
    Err(moved())
}

/// One read of the item, or nothing where the facts or the intent moved
/// before its price was joined (`corpus.rs`).
fn read_one(
    store: &Store,
    intent: &Annotations,
    id: &str,
    body: bool,
) -> Result<Option<Shown>, SearchError> {
    let table = class_table()?;
    currency_table()?;
    let intent_revision = price::intent_revision(intent)?;
    let found = store
        .read_corpus(RealmScope::All, |header, rows| {
            let basis = Basis::of(store, header, intent_revision);
            let mut live = 0usize;
            for row in rows {
                let mut row = row?;
                live += 1;
                if row.id == id {
                    let stored = std::mem::take(&mut row.body);
                    let (facts, place) = placed(&locations(header), row);
                    // the listing state of the item's league, or of every
                    // league of its realm where it has none
                    let leagues = price::leagues(
                        &header
                            .locations
                            .iter()
                            .filter(|l| {
                                l.realm == place.realm
                                    && place
                                        .league
                                        .as_ref()
                                        .is_none_or(|league| l.league.as_ref() == Some(league))
                            })
                            .cloned()
                            .collect::<Vec<LocationRow>>(),
                    );
                    return Ok(Ok((basis, place, derive(facts, &stored), stored, leagues)));
                }
            }
            Ok(Err((
                header.locations.iter().find(|l| l.id == id).cloned(),
                live,
                basis.account,
            )))
        })
        .map_err(SearchError::store)?;
    let (basis, place, mut item, stored, leagues) = match found {
        Ok(found) => found,
        Err((location, live, account)) => {
            return Err(not_shown(store, location, live, &account, id));
        }
    };
    let mut prices = price::join(store, intent, &leagues)?;
    if !still_at(store, intent, &basis)? {
        return Ok(None);
    }
    let priced = price::of_item(&mut prices, id);
    let lines = std::mem::take(&mut item.lines)
        .into_iter()
        .map(|line| ShownLine {
            slots: line
                .slots()
                .into_iter()
                .map(|(w, n)| (w, n.map_or(serde_json::Value::Null, number_json)))
                .collect(),
            limit: (line.source == "veiled").then(|| limit("S12")),
            line,
        })
        .collect();
    Ok(Some(Shown {
        basis,
        place,
        class: table.classify(&item),
        price: priced,
        item,
        lines,
        body: body.then_some(stored),
    }))
}

fn not_shown(
    store: &Store,
    location: Option<LocationRow>,
    live: usize,
    account: &str,
    id: &str,
) -> SearchError {
    if let Some(location) = location {
        return SearchError::scope(
            "not_an_item",
            format!(
                "`{id}` is the {} `{}`, not an item: a search lists what is in it",
                if location.kind == "stash" {
                    "tab"
                } else {
                    "character"
                },
                location.name
            ),
        )
        .with_offers(vec![command(
            "search",
            Some(account),
            Some(&location.realm),
            &format!("id:{id}"),
        )]);
    }
    match store.item(id) {
        Ok(Some(_)) => SearchError::scope(
            "item_not_live",
            format!("item `{id}` is in the store and is not live — removed, or its tab or character was: the search reads live items only"),
        )
        .with_offers(vec![command("items show", Some(account), None, id)]),
        Ok(None) => SearchError::scope(
            "item_not_found",
            format!("no item `{id}` in this store: an id is given whole, as an answer printed it ({live} live items, every realm)"),
        ),
        Err(e) => SearchError::store(e),
    }
}

/// Derive one item outside a store: what a test or a tool holding a body
/// shows of it.
pub fn derived(facts: Facts, body: &str) -> Item {
    derive(facts, body)
}
