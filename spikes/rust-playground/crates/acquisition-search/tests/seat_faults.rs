//! The first seat's faults (`runs/seat-2026-09-26/`, 2026-09-26; the
//! record's findings and "Holes ruled", the seat row), each reproduced
//! here as a failing test before it was fixed at step 9b: the class
//! reading (V1, V2, F2), the undecided block's shape (V8, F9), a tab's
//! type as a field (V10), the place keys under an all-realms scope (F3),
//! and the answer's words — the help's limit (F1), a comparison after a
//! group (F4), a closed set's refusal (F5), a computed value's case
//! (F10), a partial id (F7), what a row sorts by (F6). The text these
//! render to is `acquisition-cli/tests/search_json.rs`.

mod common;

use std::collections::BTreeSet;

use acquisition_search::{Request, answer};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

fn plain(id: &str, base: &str, frame: &str, more: Value) -> Value {
    let mut body = json!({ "id": id, "name": "", "typeLine": base, "baseType": base,
        "frameTypeId": frame, "identified": true, "ilvl": 0, "x": 0, "y": 0 });
    for (key, value) in more.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

fn asked(s: &Store, realm: &str, text: &str) -> Value {
    as_json(&ask(&load(s, Some(realm)), text).unwrap())
}

fn view(s: &Store, realm: &str, text: &str, view: Value) -> Value {
    let request: Request =
        serde_json::from_value(json!({ "query": { "text": text }, "view": view })).unwrap();
    as_json(&answer(&load(s, Some(realm)), &request).unwrap())
}

fn counts(a: &Value, path: &str) -> (u64, u64, u64, u64) {
    let t = a["terms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["path"] == path)
        .unwrap();
    let n = |k: &str| t[k]["count"].as_u64().unwrap();
    (n("matched"), n("failed"), n("lacked"), n("undecided"))
}

/// The ids a count's route returns, over the realm it names: as many as
/// it counted (invariant 4).
fn members(s: &Store, counted: &Value) -> BTreeSet<String> {
    let n = counted["count"].as_u64().unwrap();
    let Some(route) = counted.get("request") else {
        assert_eq!(n, 0, "a count with no route: {counted}");
        return BTreeSet::new();
    };
    let mut route = route.clone();
    route["view"]["rows"]["limit"] = json!(100);
    let realm = route["scope"]["realm"].as_str().unwrap().to_string();
    let request: Request = serde_json::from_value(route.clone()).unwrap();
    let routed = as_json(&answer(&load(s, Some(&realm)), &request).unwrap());
    let ids: BTreeSet<String> = ids(&routed).into_iter().collect();
    assert_eq!(ids.len() as u64, n, "{route}");
    ids
}

/// One tab, `Gear`, of what the seat met: three blighted maps, two
/// invitations, an Energy Blade, two itemised beasts, a plain map and a
/// ring.
fn seat_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("g1", "Gear")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "g1",
        "Gear",
        vec![
            plain("b13", "Blighted Map (Tier 13)", "Normal", json!({})),
            plain("b16", "Blight-ravaged Map (Tier 16)", "Rare", json!({})),
            plain("m1", "Map (Tier 1)", "Normal", json!({})),
            plain("inv", "Polaric Invitation", "Normal", json!({})),
            plain("quest", "Incandescent Invitation", "Quest", json!({})),
            item("blade", "Storm Edge", "Energy Blade", "Rare", json!({})),
            item("beast", "Dune Hellion", "Dune Hellion", "Rare", json!({})),
            item("wolf", "Snow Wolf", "Snow Wolf", "Rare", json!({})),
            item("ring", "Doom Loop", "Iron Ring", "Rare", json!({})),
        ],
        20,
    );
    s
}

/// V1 (owner, 2026-09-26: "I approve your suggestion for blighted maps"):
/// a blighted or blight-ravaged map's class is read from its base past
/// the prefix the API adds, so it is Maps like the plain map — never
/// undecided as a base the table lacks.
#[test]
fn v1_a_blighted_maps_class_is_read_past_the_prefix_the_api_adds() {
    let s = seat_stash();
    assert_eq!(ids(&asked(&s, "pc", "class=Maps")), ["b13", "b16", "m1"]);
    // `class:map` picks Misc Map Items too, the normal invitation's (V2)
    assert_eq!(
        ids(&asked(&s, "pc", "class:map")),
        ["b13", "b16", "inv", "m1"]
    );
    let open = asked(&s, "pc", "undecided(class)");
    assert!(
        !ids(&open).iter().any(|id| id == "b13" || id == "b16"),
        "{:?}",
        ids(&open)
    );
    let shown = serde_json::to_value(common::show(&s, "b16", false).unwrap()).unwrap();
    assert_eq!(shown["class"], json!({ "is": "Maps" }));
    // what stays undecided under `class=Maps` is the two beasts, whose
    // bases the table lacks — never a blighted map
    assert_eq!(
        asked(&s, "pc", "class=Maps")["total"]["undecided"]["count"],
        2
    );
}

/// V2 (owner: "let's use frame to determine an invitation's class"): a
/// base the table lists under Misc Map Items and Quest Items is read by
/// its frame — the quest frame Quest Items, any other Misc Map Items.
#[test]
fn v2_an_invitations_class_is_its_frames() {
    let s = seat_stash();
    assert_eq!(ids(&asked(&s, "pc", "class=\"Misc Map Items\"")), ["inv"]);
    assert_eq!(ids(&asked(&s, "pc", "class=\"Quest Items\"")), ["quest"]);
    let open = asked(&s, "pc", "undecided(class)");
    assert!(
        !ids(&open).iter().any(|id| id == "inv" || id == "quest"),
        "{:?}",
        ids(&open)
    );
    let shown = serde_json::to_value(common::show(&s, "inv", false).unwrap()).unwrap();
    assert_eq!(shown["class"], json!({ "is": "Misc Map Items" }));
    let shown = serde_json::to_value(common::show(&s, "quest", false).unwrap()).unwrap();
    assert_eq!(shown["class"], json!({ "is": "Quest Items" }));
}

/// F2: `class:X` on an item whose base the table lists under several
/// classes is undecided only where a candidate satisfies `X` — the table
/// never chooses, and a term false under every candidate needs no choice
/// (C93: undecided is where the truth cannot be established).
#[test]
fn f2_a_class_term_false_under_every_candidate_class_is_false() {
    let s = seat_stash();
    // ring matched; blade failed, its three classes none of them Rings;
    // beast and wolf undecided, the table lacking their bases
    let a = asked(&s, "pc", "class:ring");
    assert_eq!(counts(&a, "0"), (1, 6, 0, 2), "{}", a["terms"][0]);
    assert_eq!(
        ids(&asked(&s, "pc", "-class:ring")),
        ["b13", "b16", "blade", "inv", "m1", "quest"]
    );
    // a candidate the term names keeps it open
    let swords = asked(&s, "pc", "class:sword");
    assert_eq!(counts(&swords, "0"), (0, 6, 0, 3), "{}", swords["terms"][0]);
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(class:sword)")),
        ["beast", "blade", "wolf"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(class:ring)")),
        ["beast", "wolf"]
    );
    // the class itself is still not established: `undecided(class)` lists
    // it, `has:class` is open, and a count by class keeps it undecided
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(class)")),
        ["beast", "blade", "wolf"]
    );
    assert_eq!(
        asked(&s, "pc", "has:class")["terms"][0]["undecided"]["count"],
        3
    );
    let table = &view(&s, "pc", "", json!({ "counts": { "keys": ["class"] } }))["view"]["counts"]["tables"]
        [0];
    let undecided = table["buckets"].as_array().unwrap().last().unwrap();
    assert_eq!(undecided["bucket"], "undecided");
    assert_eq!(undecided["count"], 3);
    // every route returns exactly what it counted (invariant 4): the
    // failed route admits the blade, whose class no `has:` establishes
    let a = asked(&s, "pc", "class:ring");
    assert_eq!(members(&s, &a["terms"][0]["failed"]).len(), 6);
    assert!(members(&s, &a["terms"][0]["failed"]).contains("blade"));
    assert_eq!(members(&s, &a["terms"][0]["undecided"]).len(), 2);
    assert_eq!(a["terms"][0]["lacked"]["count"], 0);
}
