//! Fixture stores for the search's boundary tests, built through the
//! store's own ingest (`Store::record`), never by writing rows (the build
//! plan, rule 7).
#![allow(dead_code)]

pub mod generated;

use std::collections::BTreeMap;
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

/// A track's committed table, read where it lies (the build plan, "The
/// steps left": the expected answers first): a CSV text as rows of
/// fields, a comment line dropped, a quoted field kept whole with its
/// commas, a doubled quote one quote.
pub fn csv(text: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for line in text.lines().filter(|l| !l.starts_with('#')) {
        let mut fields = vec![String::new()];
        let mut quoted = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match (c, quoted) {
                ('"', true) if chars.peek() == Some(&'"') => {
                    chars.next();
                    fields.last_mut().unwrap().push('"');
                }
                ('"', _) => quoted = !quoted,
                (',', false) => fields.push(String::new()),
                (c, _) => fields.last_mut().unwrap().push(c),
            }
        }
        assert!(!quoted, "a quoted field runs past its line: {line}");
        rows.push(fields);
    }
    rows
}

/// The rows as maps by the header's names.
pub fn table(text: &str) -> Vec<BTreeMap<String, String>> {
    let rows = csv(text);
    let header = &rows[0];
    rows[1..]
        .iter()
        .map(|row| {
            assert_eq!(row.len(), header.len(), "{row:?}");
            header.iter().cloned().zip(row.iter().cloned()).collect()
        })
        .collect()
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

/// A row's sort scalar alone — its value and status — without what the
/// row shows of it (F6, the first seat), which follows the item's order.
pub fn scalar(sort: &Value) -> Value {
    let mut scalar = sort.clone();
    if let Some(fields) = scalar.as_object_mut() {
        fields.remove("shows");
        fields.remove("left_out");
    }
    scalar
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

// ---- the worked example (`answer.rs`) ------------------------------------------------------

pub fn ring(id: &str, rarity: &str, more: Value) -> Value {
    item(id, &format!("Ring {id}"), "Two-Stone Ring", rarity, more)
}

pub fn lines(life: &[i64], res: i64) -> Vec<String> {
    life.iter()
        .map(|n| format!("+{n} to maximum Life"))
        .chain([format!("+{res}% to Fire Resistance")])
        .collect()
}

/// The worked example's seven pc items (`answer.rs`, "The worked
/// example"), two fetched locations and one never fetched: rare rings r1
/// (life 95, res 65), r2 (20 implicit + 75 explicit, 60), r3 (100, 55),
/// r6 (readable 95, implicit array unread, res subtotal 65 incomplete),
/// r7 (no life line, 65); r4 a magic ring (95, 70); r5 rare, life 95, res
/// 65, its base missing from the class table.
pub fn worked() -> Store {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("t1", "Rings"), tab("t2", "Dump"), tab("t3", "Never")]),
        10,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "Rings",
        vec![
            ring("r1", "Rare", json!({ "explicitMods": lines(&[95], 65) })),
            ring(
                "r2",
                "Rare",
                json!({ "implicitMods": ["+20 to maximum Life"], "explicitMods": lines(&[75], 60) }),
            ),
            ring("r3", "Rare", json!({ "explicitMods": lines(&[100], 55) })),
            ring("r4", "Magic", json!({ "explicitMods": lines(&[95], 70) })),
        ],
        20,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t2",
        "Dump",
        vec![
            item(
                "r5",
                "Ring r5",
                "Mystery Ring",
                "Rare",
                json!({ "explicitMods": lines(&[95], 65) }),
            ),
            // its implicit array is not one: unread
            ring(
                "r6",
                "Rare",
                json!({ "implicitMods": "unreadable", "explicitMods": lines(&[95], 65) }),
            ),
            ring("r7", "Rare", json!({ "explicitMods": lines(&[], 65) })),
        ],
        30,
    );
    s
}

pub const WORKED: &str =
    r##"league=Standard class=Rings rarity=rare "+# to maximum Life">=90 pseudo.total_res>=60"##;
