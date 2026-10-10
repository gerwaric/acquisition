//! Membership (`search/DESIGN.md`, *Membership*; C108, C100; the build
//! plan, step 10 and gap 3): `live` by default, `all` on request — the
//! removed beside the live, each marked with when it left, at the place
//! it was last seen — through the crate's boundary, a request in, an
//! answer out, as JSON. Every count is worked by hand on a store built
//! through the store's own ingest: a fetch that no longer has an item,
//! and a listing that no longer names a tab.

mod common;

use acquisition_search::answer::Request;
use acquisition_search::{Corpus, answer};
use acquisition_store::{Membership, Store};
use common::*;
use serde_json::{Value, json};

fn ring(id: &str, rarity: &str, life: i64) -> Value {
    item(
        id,
        &format!("Ring {id}"),
        "Two-Stone Ring",
        rarity,
        json!({ "explicitMods": [format!("+{life} to maximum Life")] }),
    )
}

/// Two tabs, `Rings` holding `a` and `b` and `Gone` holding `c`; then a
/// fetch of `Rings` at 50 that has `a` alone, so `b` left at 50; then a
/// listing at 60 that names `Rings` alone, so `Gone` is retired and `c`
/// left with it at 60. One live item, two removed.
fn left() -> Store {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("t1", "Rings"), tab("t2", "Gone")]),
        10,
    );
    let both = vec![ring("a", "Rare", 95), ring("b", "Rare", 80)];
    fetch_tab(&mut s, "pc", "Standard", "t1", "Rings", both, 20);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t2",
        "Gone",
        vec![ring("c", "Magic", 70)],
        30,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "Rings",
        vec![ring("a", "Rare", 95)],
        50,
    );
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "Rings")]), 60);
    s
}

fn all(s: &Store) -> Corpus {
    Corpus::load_of(
        s,
        &intent(s),
        Some(&acquisition_search::Realm::parse("pc").unwrap()),
        Membership::All,
    )
    .unwrap()
}

fn asked(corpus: &Corpus, request: Value) -> Result<Value, Value> {
    let request: Request = serde_json::from_value(request).unwrap();
    answer(corpus, &request)
        .map(|a| as_json(&a))
        .map_err(|e| e.to_json())
}

fn under_all(corpus: &Corpus, text: &str) -> Value {
    asked(
        corpus,
        json!({ "scope": { "membership": "all" }, "query": { "text": text } }),
    )
    .unwrap_or_else(|e| panic!("{text}: {e}"))
}

fn in_order(answer: &Value) -> Vec<String> {
    answer["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect()
}

/// Live is what it was: the one item in the stash, no row marked, and
/// the scope saying so.
#[test]
fn live_is_the_default_and_holds_what_is_in_the_stash() {
    let s = left();
    let corpus = load(&s, Some("pc"));
    let a = as_json(&ask(&corpus, "").unwrap());
    assert_eq!(in_order(&a), ["a"]);
    assert_eq!(a["scope"]["membership"], "live");
    assert_eq!(a["scope"]["items"], 1);
    assert!(a["scope"].get("removed").is_none());
    assert!(a["rows"][0].get("removed_at").is_none());
    // the retired tab is no location of the scope
    assert_eq!(
        (&a["scope"]["fetched"], &a["scope"]["never_fetched"]),
        (&json!(1), &json!(0))
    );
}

/// All adds the removed, each row marked with when it left and sitting
/// where it was last seen — `c` in a tab a listing retired, by that
/// tab's name; the scope block counts the two apart, and coverage is
/// still the live locations'.
#[test]
fn all_adds_the_removed_each_marked_at_the_place_it_was_last_seen() {
    let s = left();
    let corpus = all(&s);
    let a = under_all(&corpus, "");
    assert_eq!(in_order(&a), ["a", "b", "c"]);
    assert_eq!(a["total"]["matched"], 3);
    let scope = &a["scope"];
    assert_eq!(scope["membership"], "all");
    assert_eq!((&scope["items"], &scope["removed"]), (&json!(1), &json!(2)));
    assert_eq!(
        (&scope["fetched"], &scope["never_fetched"]),
        (&json!(1), &json!(0))
    );
    let rows = a["rows"].as_array().unwrap();
    assert!(rows[0].get("removed_at").is_none());
    assert_eq!(rows[1]["removed_at"], 50);
    assert_eq!(rows[1]["place"]["name"], "Rings");
    assert_eq!(rows[2]["removed_at"], 60);
    assert_eq!(rows[2]["place"]["name"], "Gone");
    assert_eq!(rows[2]["place"]["league"], "Standard");

    // a removed item is found by its place, and by what its body says
    assert_eq!(in_order(&under_all(&corpus, "tab:Gone")), ["c"]);
    assert_eq!(in_order(&under_all(&corpus, "tab:Rings")), ["a", "b"]);
    assert_eq!(in_order(&under_all(&corpus, "rarity=rare")), ["a", "b"]);
    assert_eq!(
        in_order(&under_all(&corpus, r##""+# to maximum Life">=75"##)),
        ["a", "b"]
    );
    assert_eq!(in_order(&under_all(&corpus, "id:t2")), ["c"]);
    assert_eq!(in_order(&under_all(&corpus, "id:b")), ["b"]);
    // every term counts over the three
    let a = under_all(&corpus, "rarity=rare");
    assert_eq!(
        (
            &a["terms"][0]["matched"]["count"],
            &a["terms"][0]["failed"]["count"]
        ),
        (&json!(2), &json!(1))
    );
}

/// A removed item's price is lacked, never undecided: an item not in the
/// stash has no listing state.
#[test]
fn a_removed_items_price_is_lacked() {
    let s = left();
    let corpus = all(&s);
    let live = as_json(&ask(&load(&s, Some("pc")), "-has:priced").unwrap());
    assert_eq!(
        (
            &live["total"]["matched"],
            &live["total"]["undecided"]["count"]
        ),
        (&json!(1), &json!(0))
    );
    let a = under_all(&corpus, "-has:priced");
    assert_eq!(in_order(&a), ["a", "b", "c"]);
    assert_eq!(a["total"]["undecided"]["count"], 0);
    assert_eq!(under_all(&corpus, "has:priced")["total"]["matched"], 0);
    assert_eq!(
        under_all(&corpus, "undecided(has:priced)")["total"]["matched"],
        0
    );
    let shown = serde_json::to_value(show(&s, "b", false).unwrap()).unwrap();
    assert_eq!(shown["price"]["status"], "none");
    assert_eq!(shown["price"]["kind"], "removed");
}

/// Every route of an answer under `all` is a request under `all`, and
/// the command it prints says so; a corpus holds one membership, and a
/// request naming the other is refused as another realm is.
#[test]
fn a_route_under_all_is_under_all_and_a_corpus_holds_one_membership() {
    let s = left();
    let corpus = all(&s);
    let a = under_all(&corpus, "rarity=rare");
    let route = &a["terms"][0]["failed"];
    assert_eq!(route["request"]["scope"]["membership"], "all");
    let request: Request = serde_json::from_value(route["request"].clone()).unwrap();
    assert_eq!(
        in_order(&as_json(&answer(&corpus, &request).unwrap())),
        ["c"]
    );
    assert!(
        request.command().contains(" --membership all"),
        "{}",
        request.command()
    );
    // a continuation too
    let cut = asked(
        &corpus,
        json!({ "scope": { "membership": "all" }, "view": { "rows": { "limit": 1 } } }),
    )
    .unwrap();
    let next = cut["view"]["rows"]["next"]["request"].clone();
    assert_eq!(next["scope"]["membership"], "all");
    assert_eq!(in_order(&asked(&corpus, next).unwrap()), ["b"]);

    let e = asked(&corpus, json!({ "query": { "text": "" } })).unwrap_err();
    assert_eq!(e["kind"], "membership_mismatch", "{e}");
    let e = asked(
        &load(&s, Some("pc")),
        json!({ "scope": { "membership": "all" } }),
    )
    .unwrap_err();
    assert_eq!(e["kind"], "membership_mismatch", "{e}");
    // a live answer's command names no membership
    let live = as_json(&ask(&load(&s, Some("pc")), "rarity=rare").unwrap());
    let request: Request =
        serde_json::from_value(live["terms"][0]["matched"]["request"].clone()).unwrap();
    assert!(!request.command().contains("--membership"));
}

/// AQ4 across a refresh that removed the item: an id an earlier answer
/// printed and the stash no longer holds matches nothing live, and the
/// term says the id left, when, and the route under `all` — whatever
/// the total — so that it never reads as an item that did not exist. A
/// retired tab's id is said the same way.
#[test]
fn aq4_an_id_that_left_says_so_with_when_and_the_route_under_all() {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("t1", "Rings"), tab("t2", "Gone")]),
        10,
    );
    let both = vec![ring("a", "Rare", 95), ring("b", "Rare", 80)];
    fetch_tab(&mut s, "pc", "Standard", "t1", "Rings", both, 20);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t2",
        "Gone",
        vec![ring("c", "Magic", 70)],
        30,
    );
    // every id printed is accepted back while the item is there
    let before = load(&s, Some("pc"));
    let printed = in_order(&as_json(&ask(&before, "").unwrap()));
    assert_eq!(printed, ["a", "b", "c"]);
    for id in &printed {
        let a = as_json(&ask(&before, &format!("id:{id}")).unwrap());
        assert_eq!(in_order(&a), [id.as_str()]);
        assert!(a["terms"][0].get("removed").is_none());
    }
    // the refresh
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "Rings",
        vec![ring("a", "Rare", 95)],
        50,
    );
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "Rings")]), 60);
    let live = load(&s, Some("pc"));
    let everything = all(&s);

    for (id, at, found) in [("b", 50, "b"), ("c", 60, "c"), ("t2", 60, "c")] {
        let a = as_json(&ask(&live, &format!("id:{id}")).unwrap());
        assert_eq!(a["total"]["matched"], 0, "{id}");
        let removed = &a["terms"][0]["removed"];
        assert_eq!(removed["removed_at"], at, "{id}: {}", a["terms"][0]);
        assert_eq!(removed["request"]["scope"]["membership"], "all");
        assert_eq!(removed["request"]["query"]["text"], format!("id:{id}"));
        let request: Request = serde_json::from_value(removed["request"].clone()).unwrap();
        assert_eq!(
            in_order(&as_json(&answer(&everything, &request).unwrap())),
            [found]
        );
        // the zero block no longer says the id is a part of one
        let nothing = &a["zero"]["resolved_to_nothing"][0];
        assert_eq!(nothing["of"], "id");
        assert!(nothing.get("note").is_none(), "{nothing}");
    }
    // whatever the total: beside an id that is there
    let a = as_json(&ask(&live, "id:a or id:b").unwrap());
    assert_eq!(in_order(&a), ["a"]);
    let by_term: Vec<bool> = a["terms"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t.get("removed").is_some())
        .collect();
    assert_eq!(by_term, [false, true]);
    // an id the store never held is still a part of one, and no more
    let a = as_json(&ask(&live, "id:zzz").unwrap());
    assert!(a["terms"][0].get("removed").is_none());
    assert!(a["zero"]["resolved_to_nothing"][0]["note"].is_string());
    // under all the id is there, and nothing is said of its leaving
    let a = under_all(&everything, "id:b");
    assert!(a["terms"][0].get("removed").is_none());

    // `show` shows a removed item, marked, at the place it left
    for (id, at, place) in [("b", 50, "Rings"), ("c", 60, "Gone")] {
        let shown = serde_json::to_value(show(&s, id, false).unwrap()).unwrap();
        assert_eq!(shown["item"]["facts"]["removed_at"], at);
        assert_eq!(shown["place"]["name"], place);
    }
    let shown = serde_json::to_value(show(&s, "a", false).unwrap()).unwrap();
    assert!(shown["item"]["facts"]["removed_at"].is_null());
}

/// What came back is live again: a fetch that has the item once more
/// takes its mark away, and a listing that names the tab again brings
/// the tab back with nothing in it until it is fetched.
#[test]
fn what_comes_back_is_live_again() {
    let mut s = left();
    let both = vec![ring("a", "Rare", 95), ring("b", "Rare", 80)];
    fetch_tab(&mut s, "pc", "Standard", "t1", "Rings", both, 70);
    assert_eq!(
        in_order(&as_json(&ask(&load(&s, Some("pc")), "").unwrap())),
        ["a", "b"]
    );
    let a = under_all(&all(&s), "");
    assert_eq!(in_order(&a), ["a", "b", "c"]);
    assert!(a["rows"][1].get("removed_at").is_none());
    assert_eq!(
        (&a["scope"]["items"], &a["scope"]["removed"]),
        (&json!(2), &json!(1))
    );
    let live = as_json(&ask(&load(&s, Some("pc")), "id:b").unwrap());
    assert!(live["terms"][0].get("removed").is_none());
}

/// A tab a listing retired that held no item: the id left, and the term
/// says a tab did — under `live` with the route, which finds nothing,
/// and under `all`, where there is nothing more to find, with none; the
/// two answers never disagree on whether the id was ever there. `show`
/// of a retired tab's id says it is one and prints the search under
/// `all`, made by the one maker of a command.
#[test]
fn a_retired_tab_that_held_nothing_is_said_to_have_left_either_way() {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("t1", "Rings"), tab("t2", "Gone"), tab("t4", "Empty")]),
        10,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "Rings",
        vec![ring("a", "Rare", 95)],
        20,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t2",
        "Gone",
        vec![ring("c", "Magic", 70)],
        30,
    );
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "Rings")]), 60);

    let live = as_json(&ask(&load(&s, Some("pc")), "id:t4").unwrap());
    let removed = &live["terms"][0]["removed"];
    assert_eq!(
        (&removed["what"], &removed["removed_at"]),
        (&json!("tab"), &json!(60))
    );
    let request: Request = serde_json::from_value(removed["request"].clone()).unwrap();
    let everything = all(&s);
    let followed = as_json(&answer(&everything, &request).unwrap());
    assert_eq!(followed["total"]["matched"], 0);
    let said = &followed["terms"][0]["removed"];
    assert_eq!(
        (&said["what"], &said["removed_at"]),
        (&json!("tab"), &json!(60))
    );
    assert!(said.get("request").is_none(), "{said}");
    assert!(
        followed["zero"]["resolved_to_nothing"][0]
            .get("note")
            .is_none(),
        "{}",
        followed["zero"]
    );
    // what an item's id and a tab's that held one say they were
    let a = as_json(&ask(&load(&s, Some("pc")), "id:c or id:t2").unwrap());
    let what: Vec<&str> = a["terms"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["removed"]["what"].as_str().unwrap())
        .collect();
    assert_eq!(what, ["item", "tab"]);

    for (id, name) in [("t2", "Gone"), ("t4", "Empty")] {
        let e = show(&s, id, false).unwrap_err().to_json();
        assert_eq!(e["kind"], "not_an_item");
        assert!(e["error"].as_str().unwrap().contains(name), "{e}");
        assert_eq!(
            e["readings"],
            json!([format!(
                "acq search --account u-1 --realm pc id:{id} --membership all"
            )])
        );
    }
    // a live tab's is the live search, as it was
    let e = show(&s, "t1", false).unwrap_err().to_json();
    assert_eq!(
        e["readings"],
        json!(["acq search --account u-1 --realm pc id:t1"])
    );
}

/// An id is said to have left only where nothing live of the scope has
/// it: a tab id two leagues share, retired in one and live in the other,
/// left nothing. And under `all` the scope names every league its items
/// sit in, one whose every tab is retired among them.
#[test]
fn an_id_live_elsewhere_in_the_scope_has_not_left_and_all_names_every_league() {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Old",
        json!([tab("o1", "First"), tab("x1", "Shared")]),
        10,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Old",
        "o1",
        "First",
        vec![ring("old1", "Rare", 60)],
        20,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Old",
        "x1",
        "Shared",
        vec![ring("old2", "Rare", 61)],
        21,
    );
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("x1", "Shared"), tab("t1", "Rings")]),
        22,
    );
    fetch_tab(&mut s, "pc", "Standard", "x1", "Shared", vec![], 23);
    list_tabs(&mut s, "pc", "Old", json!([]), 30);

    let live = load(&s, Some("pc"));
    let a = as_json(&ask(&live, "id:x1").unwrap());
    assert_eq!(a["total"]["matched"], 0);
    assert!(a["terms"][0].get("removed").is_none(), "{}", a["terms"][0]);
    assert_eq!(a["scope"]["leagues"], json!(["Standard"]));
    // the tab that is gone everywhere did leave
    let a = as_json(&ask(&live, "id:o1").unwrap());
    assert_eq!(a["terms"][0]["removed"]["what"], "tab");

    let everything = all(&s);
    let a = under_all(&everything, "");
    assert_eq!(in_order(&a), ["old1", "old2"]);
    assert_eq!(a["scope"]["leagues"], json!(["Old", "Standard"]));
    assert_eq!(
        in_order(&under_all(&everything, "league=Old")),
        ["old1", "old2"]
    );
    assert_eq!(a["rows"][1]["place"]["league"], "Old");
}

/// A removed character item is in the league it was last seen in, as a
/// removed stash item is in its tab's: `gone` left a character in
/// Settlers, the character was then listed and fetched in Standard, and
/// under `all` the one character has an item in each — its gear in the
/// league it is in today, what left it in the league it left in. The
/// scope names both leagues. Live, the character's items are in its
/// league today, as they were (S131).
#[test]
fn a_removed_character_item_is_in_the_league_it_was_last_seen_in() {
    let mut s = store();
    let in_league = |league: &str, equipment: Vec<Value>| json!({ "id": "c1", "name": "Mover", "league": league, "equipment": equipment });
    let listed = |league: &str| json!([{ "id": "c1", "name": "Mover", "league": league }]);
    list_characters(&mut s, "pc", listed("Settlers"), 10);
    let both = vec![ring("kept", "Rare", 90), ring("gone", "Rare", 80)];
    fetch_character(&mut s, "pc", in_league("Settlers", both), 20);
    fetch_character(
        &mut s,
        "pc",
        in_league("Settlers", vec![ring("kept", "Rare", 90)]),
        30,
    );
    // the league ends: the character is in Standard, and is fetched there
    list_characters(&mut s, "pc", listed("Standard"), 40);
    fetch_character(
        &mut s,
        "pc",
        in_league("Standard", vec![ring("kept", "Rare", 90)]),
        50,
    );

    let live = as_json(&ask(&load(&s, Some("pc")), "").unwrap());
    assert_eq!(in_order(&live), ["kept"]);
    assert_eq!(live["rows"][0]["place"]["league"], "Standard");
    assert_eq!(live["scope"]["leagues"], json!(["Standard"]));

    let everything = all(&s);
    let a = under_all(&everything, "");
    let leagued: Vec<(String, Value, Value)> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap().to_string(),
                r["place"]["league"].clone(),
                r["place"]["name"].clone(),
            )
        })
        .collect();
    assert_eq!(
        leagued,
        [
            ("gone".to_string(), json!("Settlers"), json!("Mover")),
            ("kept".to_string(), json!("Standard"), json!("Mover")),
        ]
    );
    assert_eq!(a["rows"][0]["removed_at"], 30);
    assert_eq!(
        in_order(&under_all(&everything, "league=Settlers")),
        ["gone"]
    );
    assert_eq!(
        in_order(&under_all(&everything, "league=Standard")),
        ["kept"]
    );
    assert_eq!(a["scope"]["leagues"], json!(["Settlers", "Standard"]));
    // and `show` says the same of it
    let shown = serde_json::to_value(show(&s, "gone", false).unwrap()).unwrap();
    assert_eq!(shown["place"]["league"], "Settlers");
}
