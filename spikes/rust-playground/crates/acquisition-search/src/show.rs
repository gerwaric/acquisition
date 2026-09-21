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
//! with the search that lists what is in it.

use acquisition_store::Store;
use acquisition_store::corpus::{LocationRow, RealmScope};
use serde::Serialize;

use crate::corpus::{Basis, Place, locations, placed};
use crate::derive::{Facts, Item, Line, derive};
use crate::describe::limit;
use crate::error::SearchError;
use crate::eval::number_json;

#[derive(Debug, Clone, Serialize)]
pub struct Shown {
    pub basis: Basis,
    pub place: Place,
    /// The item without its lines, which follow with their slots.
    pub item: Item,
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
pub fn show(store: &Store, id: &str, body: bool) -> Result<Shown, SearchError> {
    let found = store
        .read_corpus(RealmScope::All, |header, rows| {
            let basis = Basis::of(store, header);
            let mut live = 0usize;
            for row in rows {
                let mut row = row?;
                live += 1;
                if row.id == id {
                    let stored = std::mem::take(&mut row.body);
                    let (facts, place) = placed(&locations(header), row);
                    return Ok(Ok((basis, place, derive(facts, &stored), stored)));
                }
            }
            Ok(Err((
                header.locations.iter().find(|l| l.id == id).cloned(),
                live,
            )))
        })
        .map_err(SearchError::store)?;
    let (basis, place, mut item, stored) = match found {
        Ok(found) => found,
        Err((location, live)) => return Err(not_shown(store, location, live, id)),
    };
    let lines = std::mem::take(&mut item.lines)
        .into_iter()
        .map(|line| ShownLine {
            slots: line
                .slots()
                .into_iter()
                .map(|(w, n)| (w, number_json(n)))
                .collect(),
            limit: (line.source == "veiled").then(|| limit("S12")),
            line,
        })
        .collect();
    Ok(Shown {
        basis,
        place,
        item,
        lines,
        body: body.then_some(stored),
    })
}

fn not_shown(store: &Store, location: Option<LocationRow>, live: usize, id: &str) -> SearchError {
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
        .with_offers(vec![format!(
            "acq search --realm {} id:{id}",
            location.realm
        )]);
    }
    match store.item(id) {
        Ok(Some(_)) => SearchError::scope(
            "item_not_live",
            format!("item `{id}` is in the store and is not live — removed, or its tab or character was: the search reads live items only"),
        )
        .with_offers(vec![format!("acq items show {id}")]),
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
