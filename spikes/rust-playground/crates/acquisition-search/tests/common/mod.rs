//! Fixture stores for the search's boundary tests, built through the
//! store's own ingest (`Store::record`), never by writing rows (the build
//! plan, rule 7).
#![allow(dead_code)]

pub mod generated;

use std::path::PathBuf;

use acquisition_search::show::Shown;
use acquisition_search::{Answer, Corpus, Realm, Request, SearchError, answer};
use acquisition_store::corpus::RealmScope;
use acquisition_store::{Annotations, Endpoint, Store};
use serde_json::{Value, json};

/// A facts file of this test's own. The process id alone is not: ids come
/// round again, and a directory an earlier run left behind would be opened
/// as this one's store — an empty store seen holding twelve items, once.
pub fn tmp() -> PathBuf {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "acq-search-{}-{}",
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("facts.db")
}

/// A store that knows its account and nothing else.
pub fn store() -> Store {
    let mut s = Store::open(&tmp()).unwrap();
    s.record(
        &Endpoint::Profile,
        &json!({}),
        200,
        &json!({ "uuid": "u-1", "name": "A" }),
        1,
    )
    .unwrap();
    s
}

pub fn tab(id: &str, name: &str) -> Value {
    json!({ "id": id, "name": name, "type": "PremiumStash" })
}

pub fn list_tabs(s: &mut Store, realm: &str, league: &str, tabs: Value, at: i64) {
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

pub fn fetch_tab(
    s: &mut Store,
    realm: &str,
    league: &str,
    id: &str,
    name: &str,
    items: Vec<Value>,
    at: i64,
) {
    s.record(
        &Endpoint::Stash {
            realm: realm.into(),
            league: league.into(),
            id: id.into(),
            sub: None,
        },
        &json!({ "realm": realm, "league": league, "id": id }),
        200,
        &json!({ "stash": { "id": id, "name": name, "type": "PremiumStash", "items": items } }),
        at,
    )
    .unwrap();
}

pub fn list_characters(s: &mut Store, realm: &str, entries: Value, at: i64) {
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

pub fn fetch_character(s: &mut Store, realm: &str, character: Value, at: i64) {
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

/// An item body: a header, and whatever else the test adds.
pub fn item(id: &str, name: &str, base: &str, rarity: &str, more: Value) -> Value {
    let mut body = json!({
        "id": id, "name": name, "typeLine": base, "baseType": base,
        "rarity": rarity, "frameTypeId": rarity, "identified": true, "ilvl": 84, "x": 0, "y": 0,
    });
    for (key, value) in more.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

pub fn ask(corpus: &Corpus, text: &str) -> Result<Answer, SearchError> {
    answer(corpus, &request(text))
}

pub fn request(text: &str) -> Request {
    serde_json::from_value(json!({ "query": { "text": text } })).unwrap()
}

/// An intent file of this test's own, in memory, bound to the store's
/// account: what a corpus is loaded beside where no price is written.
pub fn intent(s: &Store) -> Annotations {
    let uuid = s
        .read_corpus(RealmScope::All, |header, _| Ok(header.account_uuid.clone()))
        .unwrap();
    Annotations::open_memory_for(&uuid).unwrap()
}

pub fn load(s: &Store, realm: Option<&str>) -> Corpus {
    load_with(s, &intent(s), realm)
}

pub fn load_with(s: &Store, intent: &Annotations, realm: Option<&str>) -> Corpus {
    let realm = realm.map(|r| Realm::parse(r).unwrap());
    Corpus::load(s, intent, realm.as_ref()).unwrap()
}

/// `show` beside an empty intent file.
pub fn show(s: &Store, id: &str, body: bool) -> Result<Shown, SearchError> {
    acquisition_search::show(s, &intent(s), id, body)
}

pub fn as_json(answer: &Answer) -> Value {
    serde_json::to_value(answer).unwrap()
}

/// The ids of an answer's rows, sorted.
pub fn ids(answer: &Value) -> Vec<String> {
    let mut ids: Vec<String> = answer["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect();
    ids.sort();
    ids
}
