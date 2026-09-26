//! The counts view and the vocabulary (`search/BUILD-PLAN.md`, step 5;
//! C95, C97, C105), through the crate's boundary: a request in, an answer
//! out, as JSON. Every count is worked by hand from the fixture below, and
//! every route an answer prints is followed and compared, by id, with the
//! members worked by hand.

mod common;

use std::collections::BTreeSet;

use acquisition_search::{Request, answer, describe};
use acquisition_store::{Endpoint, Store};
use common::*;
use serde_json::{Value, json};

/// Fourteen pc items, and one in poe2.
///
/// Standard, `Dump` (d1): `r1` a rare Two-Stone Ring (12 fire-and-cold
/// implicit, 40 fire, 95 life); `r2` a rare Iron Ring (30 fire crafted, 20
/// life, 25 life implicit); `r3` an Iron Ring whose rarity is a number, so
/// unread, with 10 fire; `u1` Kaom's Heart (500 life); `gem`, no rarity.
/// `Maps` (m1) holds a substash *named `Dump`* (s1) with `map1`.
/// `Currency` (c1): `chaos` 40, `chaos2` 15, `wis` whose stack is unread,
/// `tome` with no stack at all.
/// Hardcore has a `Dump` of its own (d2): `r4`, a rare Two-Stone Ring whose
/// fire line GGG spelled another way — `+44% to fire Resistance`.
/// `Mover` wears `belt` (32 implicit and 110 explicit life), `amu` (92
/// implicit life, its explicit lines unread) and `blank` (its explicit
/// lines unread, nothing else).
/// `r2` is named `Rune Coil` and `r3` `rune coil`.
fn stash() -> Store {
    let mut s = store();
    let maps = json!({ "id": "m1", "name": "Maps", "type": "MapStash" });
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("d1", "Dump"), maps, tab("c1", "Currency")]),
        10,
    );
    list_tabs(&mut s, "pc", "Hardcore", json!([tab("d2", "Dump")]), 11);
    let rare = |id: &str, name: &str, base: &str, more: Value| item(id, name, base, "Rare", more);
    let plain = |id: &str, base: &str, frame: &str, more: Value| {
        let mut body = json!({ "id": id, "name": "", "typeLine": base, "baseType": base,
            "frameTypeId": frame, "identified": true, "ilvl": 0, "x": 0, "y": 0 });
        for (key, value) in more.as_object().unwrap() {
            body[key] = value.clone();
        }
        body
    };
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "d1",
        "Dump",
        vec![
            rare(
                "r1",
                "Doom Loop",
                "Two-Stone Ring",
                json!({ "implicitMods": ["+12% to Fire and Cold Resistances"],
                        "explicitMods": ["+40% to Fire Resistance", "+95 to maximum Life"] }),
            ),
            rare(
                "r2",
                "Rune Coil",
                "Iron Ring",
                json!({ "implicitMods": ["+25 to maximum Life"],
                        "explicitMods": [{ "description": "+30% to Fire Resistance", "flags": { "crafted": true } },
                                         "+20 to maximum Life"] }),
            ),
            rare(
                "r3",
                "rune coil",
                "Iron Ring",
                json!({ "rarity": 7, "explicitMods": ["+10% to Fire Resistance"] }),
            ),
            item(
                "u1",
                "Kaom's Heart",
                "Glorious Plate",
                "Unique",
                json!({ "explicitMods": ["+500 to maximum Life"] }),
            ),
            plain("gem", "Fireball", "Gem", json!({})),
        ],
        20,
    );
    let sub = |sub: Option<&str>| Endpoint::Stash {
        realm: "pc".into(),
        league: "Standard".into(),
        id: "m1".into(),
        sub: sub.map(str::to_string),
    };
    s.record(
        &sub(None),
        &json!({ "id": "m1" }),
        200,
        &json!({ "stash": { "id": "m1", "name": "Maps", "type": "MapStash", "items": [],
                            "children": [{ "id": "s1", "name": "Dump", "type": "MapStash" }] } }),
        21,
    )
    .unwrap();
    s.record(
        &sub(Some("s1")),
        &json!({ "id": "m1", "sub": "s1" }),
        200,
        &json!({ "stash": { "id": "s1", "name": "Dump", "type": "MapStash",
                            "items": [item("map1", "", "Beach Map", "Normal", json!({}))] } }),
        22,
    )
    .unwrap();
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "c1",
        "Currency",
        vec![
            plain("chaos", "Chaos Orb", "Currency", json!({ "stackSize": 40 })),
            plain(
                "chaos2",
                "Chaos Orb",
                "Currency",
                json!({ "stackSize": 15 }),
            ),
            plain(
                "wis",
                "Scroll of Wisdom",
                "Currency",
                json!({ "stackSize": "many" }),
            ),
            plain("tome", "Forbidden Tome", "Currency", json!({})),
        ],
        23,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Hardcore",
        "d2",
        "Dump",
        vec![rare(
            "r4",
            "Hex Band",
            "Two-Stone Ring",
            json!({ "explicitMods": ["+44% to fire Resistance"] }),
        )],
        24,
    );
    list_characters(
        &mut s,
        "pc",
        json!([{ "id": "c9", "name": "Mover", "league": "Standard" }]),
        30,
    );
    fetch_character(
        &mut s,
        "pc",
        json!({ "id": "c9", "name": "Mover", "league": "Standard", "equipment": [
            rare("belt", "Grim Clasp", "Leather Belt", json!({
                "implicitMods": ["+32 to maximum Life"], "explicitMods": ["+110 to maximum Life"] })),
            rare("amu", "Pain Locket", "Jade Amulet", json!({
                "implicitMods": ["+92 to maximum Life"], "explicitMods": "unread" })),
            rare("blank", "Dull Turn", "Coral Ring", json!({ "explicitMods": "unread" })),
        ] }),
        31,
    );
    list_tabs(&mut s, "poe2", "Standard", json!([tab("p1", "Two")]), 40);
    fetch_tab(
        &mut s,
        "poe2",
        "Standard",
        "p1",
        "Two",
        vec![rare(
            "second",
            "Other Loop",
            "Iron Ring",
            json!({ "explicitMods": ["+50 to maximum Life"] }),
        )],
        41,
    );
    s
}

fn view(s: &Store, realm: &str, text: &str, view: Value) -> Result<Value, Value> {
    let request: Request =
        serde_json::from_value(json!({ "query": { "text": text }, "view": view })).unwrap();
    answer(&load(s, Some(realm)), &request)
        .map(|a| as_json(&a))
        .map_err(|e| e.to_json())
}

fn counted(s: &Store, text: &str, keys: &[&str]) -> Value {
    view(s, "pc", text, json!({ "counts": { "keys": keys } })).unwrap()
}

fn set(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

/// The ids a count's route returns — run as the request it is, over the
/// realm it names — which must be as many as it counted.
fn members(s: &Store, counted: &Value) -> BTreeSet<String> {
    let n = counted["count"].as_u64().unwrap();
    let Some(route) = counted.get("request") else {
        assert_eq!(n, 0, "a count with no route: {counted}");
        return BTreeSet::new();
    };
    assert_eq!(counted["denominator"], "scope");
    let mut route = route.clone();
    route["view"]["rows"]["limit"] = json!(100);
    let realm = route["scope"]["realm"].as_str().unwrap().to_string();
    let request: Request = serde_json::from_value(route.clone()).unwrap();
    let routed = as_json(&answer(&load(s, Some(&realm)), &request).unwrap());
    let ids: BTreeSet<String> = ids(&routed).into_iter().collect();
    assert_eq!(ids.len() as u64, n, "{route}");
    ids
}

fn bucket<'a>(table: &'a Value, value: &str) -> &'a Value {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["bucket"] == "value" && b["value"] == value)
        .unwrap_or_else(|| panic!("no bucket `{value}` in {table}"))
}

fn apart<'a>(table: &'a Value, which: &str) -> &'a Value {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["bucket"] == which)
        .unwrap_or_else(|| panic!("no `{which}` bucket in {table}"))
}

/// A table's buckets as (label, count), in the order printed — the
/// computed values a narrowing matches apart, which `tests/pseudo.rs`
/// pins.
fn shape(table: &Value) -> Vec<(String, u64)> {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["bucket"] != "computed")
        .map(|b| {
            let label = match b["bucket"].as_str().unwrap() {
                "value" => match &b["value"] {
                    Value::String(text) => text.clone(),
                    other => other.to_string(),
                },
                other => format!("({other})"),
            };
            (label, b["count"].as_u64().unwrap())
        })
        .collect()
}

fn shaped(pairs: &[(&str, u64)]) -> Vec<(String, u64)> {
    pairs.iter().map(|(l, n)| (l.to_string(), *n)).collect()
}

/// AQ1: `--count tab,league,rarity` — three tables, no rows.
#[test]
fn aq1_three_keys_are_three_tables_and_no_rows() {
    let s = stash();
    let a = counted(&s, "", &["tab", "league", "rarity"]);
    assert_eq!(a["total"]["matched"], 14);
    assert_eq!(a["rows"], json!([]));
    assert!(a["view"].get("rows").is_none());
    let tables = a["view"]["counts"]["tables"].as_array().unwrap();
    let keys: Vec<&str> = tables.iter().map(|t| t["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["tab", "league", "rarity"]);
    // most carried first, then the name, then the id: the two tabs named
    // Dump are two tabs
    assert_eq!(
        shape(&tables[0]),
        shaped(&[
            ("Dump", 5),
            ("Currency", 4),
            ("Dump", 1),
            ("Maps", 1),
            ("(none)", 3)
        ])
    );
    assert_eq!(
        shape(&tables[1]),
        shaped(&[("Standard", 13), ("Hardcore", 1)])
    );
    assert_eq!(
        shape(&tables[2]),
        shaped(&[
            ("rare", 6),
            ("normal", 1),
            ("unique", 1),
            ("(none)", 5),
            ("(undecided)", 1)
        ])
    );
}

/// C105's two invariants, on a key with one value for each item: every
/// matching item is in a bucket, in `none` or in `undecided`, and the
/// buckets sum exactly to the total — fourteen items by rarity: rare 6,
/// unique 1, normal 1, none 5, undecided 1. (The reference's own example
/// counts by class, which step 6 builds.)
#[test]
fn c105_a_count_accounts_for_every_matching_item_and_sums_to_the_total() {
    let s = stash();
    let a = counted(&s, "", &["rarity"]);
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(table["values"], 3);
    assert_eq!(
        members(&s, bucket(table, "rare")),
        set(&["r1", "r2", "r4", "belt", "amu", "blank"])
    );
    assert_eq!(members(&s, bucket(table, "unique")), set(&["u1"]));
    assert_eq!(members(&s, bucket(table, "normal")), set(&["map1"]));
    // a gem, a currency stack: known absence, never undecided and never
    // merged with a value
    let none = apart(table, "none");
    assert_eq!(none["term"], "-has:rarity");
    assert_eq!(
        members(&s, none),
        set(&["gem", "chaos", "chaos2", "wis", "tome"])
    );
    // a rarity that could not be read: undecided, its reason on the item
    // and its kind beneath the bucket
    let undecided = apart(table, "undecided");
    assert_eq!(undecided["term"], "undecided(rarity)");
    assert_eq!(members(&s, undecided), set(&["r3"]));
    assert_eq!(
        undecided["tally"],
        json!([{ "unread": "`rarity`", "items": 1 }])
    );
    let sum: u64 = shape(table).iter().map(|(_, n)| n).sum();
    assert_eq!(sum, 14);
    assert_eq!(a["total"]["matched"], 14);
}

/// A bucket routes under the query: the old query, parenthesised, and the
/// bucket's term — never a fragment, and an item the query does not match
/// is in no bucket.
#[test]
fn a_bucket_routes_under_the_query() {
    let s = stash();
    let a = counted(&s, "base:ring or base:plate", &["rarity", "ilvl"]);
    // five rings — r1, r2, r3, r4, blank — and Kaom's Heart
    assert_eq!(a["total"]["matched"], 6);
    // r3 is a ring whose rarity is unread: a match, in `undecided`
    let rarity = &a["view"]["counts"]["tables"][0];
    assert_eq!(
        shape(rarity),
        shaped(&[("rare", 4), ("unique", 1), ("(undecided)", 1)])
    );
    assert_eq!(
        bucket(rarity, "rare")["request"]["query"]["text"],
        "(base:ring or base:plate) rarity=rare"
    );
    assert_eq!(
        members(&s, bucket(rarity, "rare")),
        set(&["r1", "r2", "r4", "blank"])
    );
    assert_eq!(members(&s, apart(rarity, "undecided")), set(&["r3"]));
    // a number is a key like another
    let ilvl = &a["view"]["counts"]["tables"][1];
    assert_eq!(shape(ilvl), shaped(&[("84", 6)]));
    assert_eq!(ilvl["buckets"][0]["term"], "ilvl=84");
}

/// OQ7's count, and B3's edge: an item in a substash is counted under its
/// tab, by the tab's id — a substash here is *named* as another tab is, and
/// a third tab of that name sits in another league, so a route by name
/// would return seven where five were counted.
#[test]
fn oq7_a_tab_is_counted_by_the_tab_and_never_by_its_name() {
    let s = stash();
    let a = counted(&s, "", &["tab"]);
    let table = &a["view"]["counts"]["tables"][0];
    let buckets = table["buckets"].as_array().unwrap();
    let dump = &buckets[0];
    assert_eq!(
        (&dump["value"], &dump["id"], &dump["league"], &dump["term"]),
        (
            &json!("Dump"),
            &json!("d1"),
            &json!("Standard"),
            &json!("id:d1 league=Standard")
        )
    );
    assert_eq!(members(&s, dump), set(&["r1", "r2", "r3", "u1", "gem"]));
    assert_eq!(
        (&buckets[2]["id"], &buckets[2]["league"]),
        (&json!("d2"), &json!("Hardcore"))
    );
    assert_eq!(members(&s, &buckets[2]), set(&["r4"]));
    // the map in the substash named Dump is in Maps
    assert_eq!(buckets[3]["id"], "m1");
    assert_eq!(members(&s, &buckets[3]), set(&["map1"]));
    // what a character wears is in no tab: known absence
    assert_eq!(
        members(&s, apart(table, "none")),
        set(&["belt", "amu", "blank"])
    );
    // the name's own term finds all three places, which is why it is not
    // the route
    let by_name = as_json(&ask(&load(&s, Some("pc")), "tab=Dump").unwrap());
    assert_eq!(by_name["total"]["matched"], 7);
    // one line everywhere, counted by tab
    let life = counted(&s, "\"# to maximum Life\"", &["tab"]);
    assert_eq!(
        shape(&life["view"]["counts"]["tables"][0]),
        shaped(&[("Dump", 3), ("(none)", 2)])
    );
}

/// `=` is any-case, so a value spelled two ways is two buckets, each with
/// a term that selects it alone.
#[test]
fn two_spellings_of_a_value_are_two_buckets_with_two_terms() {
    let s = stash();
    let a = counted(&s, "", &["name"]);
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(table["values"], 8);
    let (upper, lower) = (bucket(table, "Rune Coil"), bucket(table, "rune coil"));
    assert_eq!(upper["term"], "name~\"(?-i)^Rune Coil$\"");
    assert_eq!(members(&s, upper), set(&["r2"]));
    assert_eq!(members(&s, lower), set(&["r3"]));
    // one spelling alone keeps the plain term, an apostrophe and all
    assert_eq!(
        bucket(table, "Kaom's Heart")["term"],
        "name=\"Kaom's Heart\""
    );
    assert_eq!(members(&s, bucket(table, "Kaom's Heart")), set(&["u1"]));
    assert_eq!(apart(table, "none")["count"], 6);
}

/// C95's sum, over the three kinds: an item with the value, one lacking it
/// — it adds nothing and is counted as lacking — and one whose value is
/// unread: the subtotal marked incomplete, never a total.
#[test]
fn c95_a_sum_beside_a_count_over_the_three_kinds() {
    let s = stash();
    let a = view(
        &s,
        "pc",
        "frame=currency",
        json!({ "counts": { "keys": ["base"], "sum": "stack" } }),
    )
    .unwrap();
    assert_eq!(a["total"]["matched"], 4);
    let counts = &a["view"]["counts"];
    assert_eq!(
        counts["sum"],
        json!({ "name": "stack", "value": 55, "lacking": 1, "incomplete": true, "unread": 1 })
    );
    let table = &counts["tables"][0];
    assert_eq!(
        shape(table),
        shaped(&[
            ("Chaos Orb", 2),
            ("Forbidden Tome", 1),
            ("Scroll of Wisdom", 1)
        ])
    );
    assert_eq!(
        bucket(table, "Chaos Orb")["sum"],
        json!({ "value": 55, "lacking": 0 })
    );
    assert_eq!(
        bucket(table, "Forbidden Tome")["sum"],
        json!({ "value": 0, "lacking": 1 })
    );
    assert_eq!(
        bucket(table, "Scroll of Wisdom")["sum"],
        json!({ "value": 0, "lacking": 0, "incomplete": true, "unread": 1 })
    );
    // an item's own sum: nothing sums to zero and is lacking nothing; an
    // unread array leaves the item's subtotal, and the bucket's, incomplete
    let life = view(
        &s,
        "pc",
        "rarity=rare",
        json!({ "counts": { "keys": ["league"], "sum": "sum(\"# to maximum Life\")" } }),
    )
    .unwrap();
    let table = &life["view"]["counts"]["tables"][0];
    // r1 95, r2 20 + 25, belt 32 + 110, amu 92 and unread, blank unread
    assert_eq!(
        bucket(table, "Standard")["sum"],
        json!({ "value": 374, "lacking": 0, "incomplete": true, "unread": 2 })
    );
    assert_eq!(
        bucket(table, "Hardcore")["sum"],
        json!({ "value": 0, "lacking": 0 })
    );
}

/// C97: the vocabulary is the same call as search — the templates the
/// matching items carry, ranked, the range of each number, the kinds
/// beneath — and the term a row carries, pasted under the query, selects
/// exactly that row. Several texts, one call.
#[test]
fn c97_the_vocabulary_and_the_pasted_term_selects_its_row() {
    let s = stash();
    let a = counted(&s, "rarity=rare", &["line:resist", "line:life"]);
    assert_eq!(a["total"]["matched"], 6);
    assert_eq!(a["total"]["undecided"]["count"], 1); // r3, in no table
    let tables = a["view"]["counts"]["tables"].as_array().unwrap();

    let resist = &tables[0];
    assert_eq!(resist["key"], "line:resist");
    assert_eq!(
        shape(resist),
        shaped(&[
            ("#% to Fire Resistance", 2),
            ("#% to Fire and Cold Resistances", 1),
            ("#% to fire Resistance", 1),
            ("(none)", 1),
            ("(undecided)", 2)
        ])
    );
    // GGG spelled the fire line two ways: two rows, and each term turns
    // case back on to select its own (owner, 2026-09-20)
    let fire = bucket(resist, "#% to Fire Resistance");
    assert_eq!(
        fire["term"],
        "line(template~\"(?-i)^#% to Fire Resistance$\")"
    );
    assert_eq!(members(&s, fire), set(&["r1", "r2"]));
    assert_eq!(
        members(&s, bucket(resist, "#% to fire Resistance")),
        set(&["r4"])
    );
    assert_eq!(
        fire["slots"],
        json!([{ "slot": "arg1", "min": 30, "max": 40 }])
    );
    // source and flag both selectable, each with its own term
    assert_eq!(fire["sources"][0]["kind"], "explicit");
    assert_eq!(members(&s, &fire["sources"][0]), set(&["r1", "r2"]));
    assert_eq!(fire["flags"][0]["kind"], "crafted");
    assert_eq!(members(&s, &fire["flags"][0]), set(&["r2"]));
    assert_eq!(members(&s, apart(resist, "none")), set(&["belt"]));
    assert_eq!(apart(resist, "none")["term"], "-line(template:resist)");
    let undecided = apart(resist, "undecided");
    assert_eq!(undecided["term"], "undecided(line(template:resist))");
    assert_eq!(members(&s, undecided), set(&["amu", "blank"]));
    assert_eq!(
        undecided["tally"],
        json!([{ "unread": "explicit lines", "items": 2 }])
    );

    let life = &tables[1];
    assert_eq!(
        shape(life),
        shaped(&[("# to maximum Life", 4), ("(none)", 1), ("(undecided)", 1)])
    );
    let row = bucket(life, "# to maximum Life");
    assert_eq!(row["term"], "line(\"# to maximum Life\")");
    // an item is counted once, whatever it carries twice; a line read is a
    // witness, so `amu` is in its row though its explicit lines are unread
    assert_eq!(members(&s, row), set(&["r1", "r2", "belt", "amu"]));
    assert_eq!(
        row["slots"],
        json!([{ "slot": "arg1", "min": 20, "max": 110 }])
    );
    let kinds: Vec<(&str, u64)> = row["sources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| (k["kind"].as_str().unwrap(), k["count"].as_u64().unwrap()))
        .collect();
    assert_eq!(kinds, [("explicit", 3), ("implicit", 3)]);
    assert_eq!(members(&s, &row["sources"][1]), set(&["r2", "belt", "amu"]));
    assert_eq!(members(&s, apart(life, "none")), set(&["r4"]));
    assert_eq!(members(&s, apart(life, "undecided")), set(&["blank"]));
}

/// The vocabulary whole, a pattern, and a ranged line's numbers by name.
#[test]
fn the_vocabulary_unnarrowed_by_a_pattern_and_of_a_ranged_line() {
    let mut s = stash();
    fetch_tab(
        &mut s,
        "pc",
        "Hardcore",
        "d2",
        "Dump",
        vec![item(
            "bow",
            "Storm Horn",
            "Short Bow",
            "Rare",
            json!({ "explicitMods": ["Adds 3 to 9 Cold Damage", "Adds 5 to 20 Cold Damage"] }),
        )],
        50,
    );
    let a = counted(&s, "league=Hardcore", &["line", "line~^adds"]);
    let tables = a["view"]["counts"]["tables"].as_array().unwrap();
    assert_eq!(shape(&tables[0]), shaped(&[("Adds # to # Cold Damage", 1)]));
    assert_eq!(tables[1]["key"], "line~^adds");
    assert_eq!(
        bucket(&tables[1], "Adds # to # Cold Damage")["slots"],
        json!([{ "slot": "low", "min": 3, "max": 5 }, { "slot": "high", "min": 9, "max": 20 }])
    );
    // nothing carries a line: `none`, by the not of every line
    let none = counted(&s, "frame=currency", &["line"]);
    let table = &none["view"]["counts"]["tables"][0];
    assert_eq!(shape(table), shaped(&[("(none)", 4)]));
    assert_eq!(apart(table, "none")["term"], "-line(true())");
    assert_eq!(members(&s, apart(table, "none")).len(), 4);
}

/// C96, C97: under an all-realms scope the vocabulary lists a template once
/// for each realm, and the row's term with the scope narrowed to that realm
/// selects exactly that row.
#[test]
fn c97_under_all_realms_a_row_is_one_realms() {
    let s = stash();
    let a = view(
        &s,
        "all",
        "rarity=rare",
        json!({ "counts": { "keys": ["line:life", "league"] } }),
    )
    .unwrap();
    let life = &a["view"]["counts"]["tables"][0];
    let rows: Vec<(&str, &str, u64)> = life["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["bucket"] == "value")
        .map(|b| {
            (
                b["realm"].as_str().unwrap(),
                b["value"].as_str().unwrap(),
                b["count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("pc", "# to maximum Life", 4),
            ("poe2", "# to maximum Life", 1)
        ]
    );
    assert_eq!(life["buckets"][1]["request"]["scope"]["realm"], "poe2");
    assert_eq!(members(&s, &life["buckets"][1]), set(&["second"]));
    assert_eq!(members(&s, &life["buckets"][0]).len(), 4);
    // a field's value is one bucket whatever realm holds it, routed over
    // the scope asked
    let league = &a["view"]["counts"]["tables"][1];
    assert_eq!(bucket(league, "Standard")["count"], 6);
    assert_eq!(
        bucket(league, "Standard")["request"]["scope"]["realm"],
        "all"
    );
    assert_eq!(members(&s, bucket(league, "Standard")).len(), 6);
}

/// `--cross`: one table of the cells that hold an item, each cell's route
/// carrying both keys; each key's `none` and `undecided` in the margins,
/// the tally once.
#[test]
fn c95_a_crossed_table_and_a_cells_route_carries_both_keys() {
    let s = stash();
    let a = view(
        &s,
        "pc",
        "",
        json!({ "cross": { "keys": ["league", "rarity"] } }),
    )
    .unwrap();
    assert_eq!(a["rows"], json!([]));
    let cross = &a["view"]["cross"];
    assert_eq!(cross["keys"], json!(["league", "rarity"]));
    let cells: Vec<(String, String, u64)> = cross["cells"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let label = |l: &Value| match l["bucket"].as_str().unwrap() {
                "value" => l["value"].as_str().unwrap().to_string(),
                other => format!("({other})"),
            };
            (
                label(&c["of"][0]),
                label(&c["of"][1]),
                c["count"].as_u64().unwrap(),
            )
        })
        .collect();
    let expected = [
        ("Standard", "rare", 5),
        ("Standard", "(none)", 5),
        ("Hardcore", "rare", 1),
        ("Standard", "normal", 1),
        ("Standard", "unique", 1),
        ("Standard", "(undecided)", 1),
    ]
    .map(|(a, b, n)| (a.to_string(), b.to_string(), n));
    assert_eq!(cells, expected);
    assert_eq!(cells.iter().map(|c| c.2).sum::<u64>(), 14);
    assert_eq!(cross["cells_in_all"], 6);
    assert_eq!(
        members(&s, &cross["cells"][0]),
        set(&["r1", "r2", "belt", "amu", "blank"])
    );
    let open = &cross["cells"][5];
    assert_eq!(
        open["request"]["query"]["text"],
        "league=Standard undecided(rarity)"
    );
    assert_eq!(members(&s, open), set(&["r3"]));
    let rarity = &cross["margins"][1];
    assert_eq!(
        (&rarity["none"]["count"], &rarity["undecided"]["count"]),
        (&json!(5), &json!(1))
    );
    assert_eq!(members(&s, &rarity["none"]).len(), 5);
    assert_eq!(cross["margins"][0]["none"]["count"], 0);
    assert_eq!(
        cross["tally"],
        json!([{ "unread": "`rarity`", "items": 1 }])
    );
}

/// Invariant 5, and the build plan's rule 9: a table is cut by the data —
/// most carried, then the value — and says how many values it left out.
#[test]
fn a_table_is_bounded_and_counts_what_it_left_out() {
    let s = stash();
    let a = view(
        &s,
        "pc",
        "",
        json!({ "counts": { "keys": ["base"], "limit": 2 } }),
    )
    .unwrap();
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(shape(table), shaped(&[("Chaos Orb", 2), ("Iron Ring", 2)]));
    assert_eq!(
        (&table["values"], &table["left_out"]),
        (&json!(11), &json!(9))
    );
    assert_eq!(table["left_out_needs"], "--limit");
    // `none` and `undecided` are never cut
    let cut = view(
        &s,
        "pc",
        "",
        json!({ "counts": { "keys": ["rarity"], "limit": 0 } }),
    )
    .unwrap();
    assert_eq!(
        shape(&cut["view"]["counts"]["tables"][0]),
        shaped(&[("(none)", 5), ("(undecided)", 1)])
    );
}

/// What a view cannot be is an authoring error, said before anything is
/// counted; what a later step builds is refused by that step's name.
#[test]
fn a_view_that_cannot_be_is_an_authoring_error() {
    let s = stash();
    for (view_json, kind, says) in [
        (
            json!({ "counts": { "keys": ["rarty"] } }),
            "unknown_name",
            "rarity",
        ),
        (
            json!({ "counts": { "keys": ["priced"] } }),
            "view",
            "yes or no",
        ),
        (
            json!({ "counts": { "keys": ["text"] } }),
            "view",
            "is no key",
        ),
        (json!({ "counts": { "keys": [] } }), "view", "takes a key"),
        (
            json!({ "counts": { "keys": ["tab", "Tab"] } }),
            "view",
            "named twice",
        ),
        (
            json!({ "counts": { "keys": ["line:"] } }),
            "view",
            "takes a text",
        ),
        (
            json!({ "counts": { "keys": ["tab:dump"] } }),
            "view",
            "only `line`",
        ),
        (
            json!({ "counts": { "keys": ["line~("] } }),
            "bad_pattern",
            "pattern",
        ),
        (json!({ "cross": { "keys": ["tab"] } }), "view", "two keys"),
        (
            json!({ "cross": { "keys": ["tab", "line"] } }),
            "view",
            "two fields",
        ),
        (
            json!({ "counts": { "keys": ["tab"], "sum": "line(\"# to maximum Life\").arg1" } }),
            "view",
            "sum(line(\"# to maximum Life\").arg1)",
        ),
        (
            json!({ "counts": { "keys": ["tab"], "sum": "name" } }),
            "operator_mismatch",
            "`--sum` takes a number",
        ),
        (
            json!({ "counts": { "keys": ["tab"], "sum": "pseudo.defence_pct" } }),
            "not_built",
            "pseudo.defence_pct",
        ),
        (
            json!({ "counts": { "keys": ["tab"], "sum": "pseudo.total_res.avg" } }),
            "slot_unknown",
            "takes no slot word",
        ),
    ] {
        let e = view(&s, "pc", "", view_json.clone()).unwrap_err();
        assert_eq!(e["kind"], kind, "{view_json}: {e}");
        let said: Vec<&str> = std::iter::once(&e["error"])
            .chain(e["readings"].as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .collect();
        assert!(said.join(" · ").contains(says), "{view_json}: {e}");
    }
}

/// C97: `--describe counts` says what a count takes, every example it
/// prints is a key or a sum this build binds, and each word names its
/// entry alone.
#[test]
fn c97_describe_counts_and_every_example_binds() {
    let s = stash();
    let whole = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let entries = whole["counts"].as_array().unwrap();
    let names: Vec<&str> = entries
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["--count", "line:", "--cross", "--sum"]);
    for entry in entries {
        for example in entry["examples"].as_array().unwrap() {
            let example = example.as_str().unwrap();
            let asked = match entry["name"].as_str().unwrap() {
                "--sum" => json!({ "counts": { "keys": ["tab"], "sum": example } }),
                "--cross" => json!({ "cross": { "keys": keys(example) } }),
                _ => json!({ "counts": { "keys": keys(example) } }),
            };
            view(&s, "pc", "", asked).unwrap_or_else(|e| panic!("{example}: {e}"));
        }
    }
    // every key the entry lists is one a count takes
    for key in whole["counts"][0]["values"].as_array().unwrap() {
        counted(&s, "", &[key.as_str().unwrap()]);
    }
    for (word, name) in [
        ("count", "--count"),
        ("--sum", "--sum"),
        ("cross", "--cross"),
        ("line:", "line:"),
    ] {
        let one = serde_json::to_value(describe(&[word.to_string()]).unwrap()).unwrap();
        let counts = one["counts"].as_array().unwrap();
        assert_eq!(counts.len(), 1, "`{word}`: {one}");
        assert_eq!(counts[0]["name"], name);
    }
}

/// The keys of an example, as a terminal spells them: `line:` takes the
/// rest of the list.
fn keys(example: &str) -> Vec<String> {
    let mut narrowing = false;
    example
        .split(',')
        .map(|part| {
            if part.starts_with("line:") || part.starts_with("line~") {
                narrowing = true;
                part.to_string()
            } else if narrowing {
                format!("line:{part}")
            } else {
                part.to_string()
            }
        })
        .collect()
}

// ---- faults an outside audit found (2026-09-22), each reproduced first ----------------------------------------

/// Outside audit, 2026-09-22: a tab is its full coordinate (C54). One id under two
/// leagues and two realms is three tabs, three buckets, each labelled and
/// routed by realm, league and id — in a table and in a crossed one.
#[test]
fn a_tab_bucket_is_the_tabs_full_coordinate() {
    let mut s = store();
    for (realm, league, id, name) in [
        ("pc", "Standard", "one", "First"),
        ("pc", "Hardcore", "two", "Second"),
        ("xbox", "Standard", "three", "Third"),
    ] {
        list_tabs(&mut s, realm, league, json!([tab("same", name)]), 10);
        fetch_tab(
            &mut s,
            realm,
            league,
            "same",
            name,
            vec![item(id, "", "Ring", "Rare", json!({}))],
            20,
        );
    }
    let a = view(&s, "all", "", json!({ "counts": { "keys": ["tab"] } })).unwrap();
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(table["values"], 3);
    let at: Vec<(&str, &str, &str, &str, &str)> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["value"].as_str().unwrap(),
                b["realm"].as_str().unwrap(),
                b["league"].as_str().unwrap(),
                b["id"].as_str().unwrap(),
                b["term"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        at,
        [
            ("First", "pc", "Standard", "same", "id:same league=Standard"),
            (
                "Second",
                "pc",
                "Hardcore",
                "same",
                "id:same league=Hardcore"
            ),
            (
                "Third",
                "xbox",
                "Standard",
                "same",
                "id:same league=Standard"
            ),
        ]
    );
    for (bucket, id, realm) in [(0, "one", "pc"), (1, "two", "pc"), (2, "three", "xbox")] {
        let b = &table["buckets"][bucket];
        assert_eq!(b["request"]["scope"]["realm"], realm);
        assert_eq!(members(&s, b), set(&[id]));
    }
    let crossed = view(
        &s,
        "all",
        "",
        json!({ "cross": { "keys": ["tab", "rarity"] } }),
    )
    .unwrap();
    let cells = crossed["view"]["cross"]["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 3);
    let ids: BTreeSet<String> = cells.iter().flat_map(|c| members(&s, c)).collect();
    assert_eq!(ids, set(&["one", "two", "three"]));
}

/// Outside audit, 2026-09-22: a bucket's sum of item totals is exact where a float
/// times 100,000 is not — one item, its own sum and the bucket's the same
/// decimal; two items, the decimal sum; an incomplete subtotal added too.
#[test]
fn c95_a_sum_of_item_sums_is_exact() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t", "T")]), 10);
    let lines = |n: usize| json!({ "explicitMods": vec!["9999999999.9997 to maximum Life"; n] });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t",
        "T",
        vec![
            item("one", "", "Ring", "Rare", lines(23)),
            item("two", "", "Amulet", "Rare", lines(23)),
            item(
                "three",
                "",
                "Belt",
                "Rare",
                json!({ "explicitMods": ["+0.0001 to maximum Life"], "implicitMods": "unread" }),
            ),
        ],
        20,
    );
    let total = 229999999999.9931;
    let by_item = as_json(
        &ask(
            &load(&s, Some("pc")),
            &format!("sum(\"# to maximum Life\")={total}"),
        )
        .unwrap(),
    );
    assert_eq!(by_item["total"]["matched"], 2);
    let a = view(
        &s,
        "pc",
        "",
        json!({ "counts": { "keys": ["base"], "sum": "sum(\"# to maximum Life\")" } }),
    )
    .unwrap();
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(bucket(table, "Ring")["sum"]["value"], json!(total));
    assert_eq!(
        bucket(table, "Belt")["sum"],
        json!({ "value": 0.0001, "lacking": 0, "incomplete": true, "unread": 1 })
    );
    assert_eq!(
        a["view"]["counts"]["sum"],
        json!({ "name": "sum(line(\"# to maximum Life\").arg1)", "value": 459999999999.9863, "lacking": 0, "incomplete": true, "unread": 1 })
    );
}

/// Outside audit, 2026-09-22: a flag is counted by its legal spelling and matched in
/// any case (B2), so a kind's route returns what it counted; a spelling
/// outside the list is counted and has no route, and says so.
#[test]
fn c97_a_kind_is_counted_by_its_legal_spelling_and_routed_by_it() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t", "T")]), 10);
    let flagged = |id: &str, flags: Value| {
        item(
            id,
            "",
            "Ring",
            "Rare",
            json!({ "explicitMods": [{ "description": "10 to maximum Life", "flags": flags }] }),
        )
    };
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t",
        "T",
        vec![
            flagged("upper", json!({ "Crafted": true })),
            flagged("lower", json!({ "crafted": true })),
            flagged("odd", json!({ "Weird": true })),
        ],
        20,
    );
    let a = counted(&s, "", &["line"]);
    let row = &a["view"]["counts"]["tables"][0]["buckets"][0];
    let flags = row["flags"].as_array().unwrap();
    assert_eq!(flags.len(), 2, "{flags:?}");
    assert_eq!(flags[0]["kind"], "crafted");
    assert_eq!(members(&s, &flags[0]), set(&["upper", "lower"]));
    assert_eq!(flags[1]["kind"], "Weird");
    assert_eq!(flags[1]["count"], 1);
    assert!(flags[1].get("request").is_none());
    assert!(
        flags[1]["needs"]
            .as_str()
            .unwrap()
            .contains("outside the closed list")
    );
    // the evaluator itself, on an item's flag spelled as GGG might
    let corpus = load(&s, Some("pc"));
    assert_eq!(
        as_json(&ask(&corpus, "line(is:CRAFTED)").unwrap())["total"]["matched"],
        2
    );
}

/// Outside audit, 2026-09-22: what a selector resolved to past the ten listed is
/// reached by a count that lists them all — the term alone over the
/// scope, since the query's other terms would drop some — and `tab` has
/// none, since a tab is counted by the tab and resolved to names (E1).
#[test]
fn c100_the_rest_of_what_a_selector_resolved_to_is_a_count_that_lists_it() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t", "T")]), 10);
    let mut items = Vec::new();
    for n in 0..12 {
        // twelve ring bases, one of them unique; twelve life-ish templates
        items.push(item(
            &format!("r{n}"),
            "",
            &format!("Ring {n:02}"),
            if n == 0 { "Unique" } else { "Rare" },
            // a kind in letters: a digit would be one more `#` of one template
            json!({ "explicitMods": [format!("+{n} to maximum Life of kind {}", "abcdefghijkl".chars().nth(n).unwrap())] }),
        ));
    }
    fetch_tab(&mut s, "pc", "Standard", "t", "T", items, 20);
    let corpus = load(&s, Some("pc"));
    let a = as_json(&ask(&corpus, "base:ring rarity=unique").unwrap());
    let resolved = &a["terms"][0]["resolved"];
    assert_eq!(
        (
            &resolved["values"].as_array().unwrap().len(),
            &resolved["more"]
        ),
        (&10, &json!(2))
    );
    let rest = &resolved["rest"];
    assert_eq!(rest["request"]["query"]["text"], "base:ring");
    assert_eq!(
        rest["request"]["view"],
        json!({ "counts": { "keys": ["base"] } })
    );
    let request: Request = serde_json::from_value(rest["request"].clone()).unwrap();
    let listed = as_json(&answer(&corpus, &request).unwrap());
    assert_eq!(listed["view"]["counts"]["tables"][0]["values"], 12);
    // a group's: the vocabulary narrowed by its one template test
    let a = as_json(&ask(&corpus, "line(template:life arg1>=100)").unwrap());
    let rest = &a["terms"][0]["resolved"]["rest"];
    assert_eq!(rest["request"]["query"]["text"], "line(template:life)");
    assert_eq!(
        rest["request"]["view"]["counts"]["keys"],
        json!(["line:life"])
    );
    let request: Request = serde_json::from_value(rest["request"].clone()).unwrap();
    let listed = as_json(&answer(&corpus, &request).unwrap());
    assert_eq!(listed["view"]["counts"]["tables"][0]["values"], 12);
    // a selector of more than one test, and a tab's: the count is said and
    // no continuation is promised
    let a = as_json(&ask(&corpus, "line(template:life source=explicit)").unwrap());
    assert!(a["terms"][0]["resolved"].get("rest").is_none());
    let mut children = Vec::new();
    for n in 0..12 {
        children
            .push(json!({ "id": format!("s{n}"), "name": format!("Map {n}"), "type": "MapStash" }));
    }
    let sub = |sub: Option<String>| Endpoint::Stash {
        realm: "pc".into(),
        league: "Standard".into(),
        id: "m".into(),
        sub,
    };
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("t", "T"), { "id": "m", "name": "Maps", "type": "MapStash" }]),
        30,
    );
    s.record(&sub(None), &json!({}), 200, &json!({ "stash": { "id": "m", "name": "Maps", "type": "MapStash", "items": [], "children": children } }), 31).unwrap();
    for n in 0..12 {
        s.record(&sub(Some(format!("s{n}"))), &json!({}), 200, &json!({ "stash": { "id": format!("s{n}"), "name": format!("Map {n}"), "type": "MapStash",
            "items": [item(&format!("map{n}"), "", "Beach Map", "Normal", json!({}))] } }), 40 + n).unwrap();
    }
    let a = as_json(&ask(&load(&s, Some("pc")), "tab:map").unwrap());
    let resolved = &a["terms"][0]["resolved"];
    // twelve substash names and their tab's, `Maps`: thirteen, ten listed
    assert_eq!(resolved["more"], 3);
    assert!(resolved.get("rest").is_none());
}

/// Outside review, 2026-09-22: units are carried through both levels of
/// adding, never read back off a float. Nine means of large pairs on one
/// item, their negatives and a ten-thousandth on another: the bucket's
/// sum is what one item carrying all of them sums to.
#[test]
fn c95_a_sum_of_item_sums_is_exact_between_items() {
    let up = "Adds 9999999999.9997 to 9999999999.9998 Cold Damage";
    let down = "Adds -9999999999.9997 to -9999999999.9998 Cold Damage";
    let bit = "Adds 0.0001 to 0.0001 Cold Damage";
    let mut split: Vec<&str> = vec![up; 9];
    let mut rest: Vec<&str> = vec![down; 9];
    rest.push(bit);
    let mut whole = split.clone();
    whole.extend(&rest);
    let counted = |items: Vec<Value>| {
        let mut s = store();
        list_tabs(&mut s, "pc", "Standard", json!([tab("t", "T")]), 10);
        fetch_tab(&mut s, "pc", "Standard", "t", "T", items, 20);
        let a = view(
            &s,
            "pc",
            "",
            json!({ "counts": { "keys": ["base"], "sum": "sum(line(template:cold).avg)" } }),
        )
        .unwrap();
        a["view"]["counts"]["sum"]["value"].clone()
    };
    let one = counted(vec![item(
        "one",
        "",
        "Ring",
        "Rare",
        json!({ "explicitMods": whole }),
    )]);
    split.truncate(9);
    let two = counted(vec![
        item("a", "", "Ring", "Rare", json!({ "explicitMods": split })),
        item("b", "", "Ring", "Rare", json!({ "explicitMods": rest })),
    ]);
    assert_eq!((one, two), (json!(0.0001), json!(0.0001)));
}

/// C105's first test as worded in the contract detail, with the class
/// names the table gives (step 6): ten rare items by class — Rings 6,
/// Wands 3, undecided 1 (an itemised beast, whose name is no base of the
/// table), total 10; no `none`, since every item has a class.
#[test]
fn c105_ten_rare_items_by_class_sum_to_the_total() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "Rares")]), 10);
    let rare = |id: &str, base: &str| item(id, "A Rare", base, "Rare", json!({}));
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "Rares",
        vec![
            rare("ring1", "Iron Ring"),
            rare("ring2", "Coral Ring"),
            rare("ring3", "Ruby Ring"),
            rare("ring4", "Amethyst Ring"),
            rare("ring5", "Two-Stone Ring"),
            rare("ring6", "Iron Ring"),
            rare("wand1", "Driftwood Wand"),
            rare("wand2", "Driftwood Wand"),
            rare("wand3", "Driftwood Wand"),
            rare("beast", "Dune Hellion"),
        ],
        20,
    );
    let a = counted(&s, "rarity=rare", &["class"]);
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(table["values"], 2);
    assert_eq!(
        shape(table),
        shaped(&[("Rings", 6), ("Wands", 3), ("(undecided)", 1)])
    );
    assert_eq!(bucket(table, "Rings")["term"], "class=Rings");
    assert_eq!(
        members(&s, bucket(table, "Rings")),
        set(&["ring1", "ring2", "ring3", "ring4", "ring5", "ring6"])
    );
    assert_eq!(
        members(&s, bucket(table, "Wands")),
        set(&["wand1", "wand2", "wand3"])
    );
    let undecided = apart(table, "undecided");
    assert_eq!(undecided["term"], "undecided(class)");
    assert_eq!(members(&s, undecided), set(&["beast"]));
    assert_eq!(
        undecided["tally"],
        json!([{ "unread": "the class: base not in the table", "items": 1 }])
    );
    assert!(
        table["buckets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["bucket"] != "none")
    );
    let sum: u64 = shape(table).iter().map(|(_, n)| n).sum();
    assert_eq!(sum, 10);
    assert_eq!(a["total"]["matched"], 10);
}
