//! `show <id> --against '<query>'` (C100: the why-not; the build plan,
//! step 10): the item as `show` reads it, and beside it the query's
//! canonical text, the root's outcome on this item and each node's, with
//! what a row shows of a matched term and what an undecided item says of
//! its reason. The seven items of the reference's worked example against
//! its query, each worked by hand.

mod common;

use acquisition_search::answer::QueryInput;
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

fn against(s: &Store, id: &str, query: &str) -> Result<Value, Value> {
    let query = QueryInput {
        text: Some(query.to_string()),
        tree: None,
    };
    acquisition_search::show::against(s, &intent(s), id, false, &query)
        .map(|shown| serde_json::to_value(shown).unwrap())
        .map_err(|e| e.to_json())
}

fn outcomes(against: &Value) -> Vec<(String, String)> {
    against["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| {
            (
                n["path"].as_str().unwrap().to_string(),
                n["outcome"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn pairs(of: &[(&str, &str)]) -> Vec<(String, String)> {
    of.iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

fn texts(node: &Value) -> Vec<String> {
    let mut texts: Vec<String> = node["shows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["line"]["text"].as_str().map(str::to_string))
        .collect();
    texts.sort();
    texts
}

/// The brief's second table: the seven against the worked example's
/// query.
#[test]
fn against_the_worked_example_each_of_the_seven() {
    let s = worked();
    let of = |id: &str| against(&s, id, WORKED).unwrap();

    // r1: a match, all five matched — and the row's evidence beside each
    let r1 = of("r1");
    assert_eq!(r1["item"]["facts"]["id"], "r1");
    let a = &r1["against"];
    assert_eq!(
        a["query"]["text"],
        r##"league=Standard class=Rings rarity=rare line("# to maximum Life" arg1>=90) pseudo.total_res>=60"##
    );
    assert!(a["query"]["tree"]["all"].is_array());
    assert_eq!(a["outcome"], "match");
    assert_eq!(
        outcomes(a),
        pairs(&[
            ("0", "matched"),
            ("1", "matched"),
            ("2", "matched"),
            ("3", "matched"),
            ("4", "matched")
        ])
    );
    assert_eq!(
        a["nodes"][3]["node"],
        r##"line("# to maximum Life" arg1>=90)"##
    );
    assert_eq!(texts(&a["nodes"][3]), ["+95 to maximum Life"]);
    assert_eq!(
        a["nodes"][4]["shows"][0],
        json!({ "value": { "name": "pseudo.total_res", "value": 65 } })
    );
    // the basis the item was read at is named, and none is compared
    assert_eq!(r1["basis"]["account"], "u-1");

    // r2: no match — 3 failed: 20 and 75, neither reaching 90
    let r2 = of("r2");
    let a = &r2["against"];
    assert_eq!(a["outcome"], "no_match");
    assert_eq!(
        outcomes(a),
        pairs(&[
            ("0", "matched"),
            ("1", "matched"),
            ("2", "matched"),
            ("3", "failed"),
            ("4", "matched")
        ])
    );
    assert_eq!(
        texts(&a["nodes"][3]),
        ["+20 to maximum Life", "+75 to maximum Life"]
    );

    // r3: no match — 4 failed: 55
    let r3 = of("r3");
    let a = &r3["against"];
    assert_eq!(a["outcome"], "no_match");
    assert_eq!(outcomes(a)[4], ("4".to_string(), "failed".to_string()));
    assert_eq!(outcomes(a)[3].1, "matched");
    assert_eq!(
        a["nodes"][4]["shows"][0],
        json!({ "value": { "name": "pseudo.total_res", "value": 55 } })
    );

    // r4: no match — 2 failed: magic, which the item beside it says
    let r4 = of("r4");
    let a = &r4["against"];
    assert_eq!(a["outcome"], "no_match");
    assert_eq!(
        outcomes(a),
        pairs(&[
            ("0", "matched"),
            ("1", "matched"),
            ("2", "failed"),
            ("3", "matched"),
            ("4", "matched")
        ])
    );
    assert_eq!(r4["item"]["rarity"], "Magic");

    // r5: undecided — 1 undecided: its base is not in the class table
    let r5 = of("r5");
    let a = &r5["against"];
    assert_eq!(a["outcome"], "undecided");
    assert_eq!(
        outcomes(a),
        pairs(&[
            ("0", "matched"),
            ("1", "undecided"),
            ("2", "matched"),
            ("3", "matched"),
            ("4", "matched")
        ])
    );
    let why = &a["nodes"][1]["shows"][0]["undecided"];
    assert_eq!(why["unread"], "the class: base not in the table");
    assert_eq!(why["path"], "1");
    assert!(
        why["hint"]
            .as_str()
            .unwrap()
            .contains("a refresh will not help; a reference update may")
    );

    // r6: undecided — 4 undecided: its implicit lines are unread
    let r6 = of("r6");
    let a = &r6["against"];
    assert_eq!(a["outcome"], "undecided");
    assert_eq!(
        outcomes(a),
        pairs(&[
            ("0", "matched"),
            ("1", "matched"),
            ("2", "matched"),
            ("3", "matched"),
            ("4", "undecided")
        ])
    );
    assert_eq!(
        a["nodes"][4]["shows"][0]["undecided"]["unread"],
        "implicit lines"
    );

    // r7: no match — 3 lacked: no life line
    let r7 = of("r7");
    let a = &r7["against"];
    assert_eq!(a["outcome"], "no_match");
    assert_eq!(outcomes(a)[3], ("3".to_string(), "lacked".to_string()));
    assert!(a["nodes"][3].get("shows").is_none(), "{}", a["nodes"][3]);
}

/// Each node's outcome: a group of terms is a node with an outcome of
/// its own, in the root's words, and a term under a not is said as the
/// term it is. r4, magic with 95 life and a total of 70, against a not,
/// an or and a `holds`, worked by hand.
#[test]
fn against_says_each_nodes_outcome_under_not_or_and_holds() {
    let s = worked();
    let shown = against(
        &s,
        "r4",
        r##"-rarity=magic (rarity=rare or pseudo.total_res>=70) holds(is:corrupted, "+# to maximum Life">=90, ilvl>=90)>=2"##,
    )
    .unwrap();
    let a = &shown["against"];
    // the not fails, the or holds by its total, and of the three r4 holds
    // its life alone
    assert_eq!(a["outcome"], "no_match");
    assert_eq!(
        outcomes(a),
        pairs(&[
            ("0", "no_match"),
            ("0.0", "matched"),
            ("1", "match"),
            ("1.0", "failed"),
            ("1.1", "matched"),
            ("2", "no_match"),
            ("2.0", "failed"),
            ("2.1", "matched"),
            ("2.2", "failed"),
        ])
    );
    assert_eq!(a["nodes"][0]["node"], "-rarity=magic");
    assert_eq!(a["nodes"][2]["node"], "rarity=rare or pseudo.total_res>=70");
    // a number that failed is shown
    assert_eq!(
        a["nodes"][8]["shows"],
        json!([{ "value": { "name": "ilvl", "value": 84 } }])
    );
    // the empty query matches every item, and has no node
    let shown = against(&s, "r4", "").unwrap();
    assert_eq!(
        shown["against"],
        json!({ "query": { "text": "", "tree": { "all": [] } }, "outcome": "match", "nodes": [] })
    );
    // without a query `show` is what it was
    assert!(
        serde_json::to_value(common::show(&s, "r4", false).unwrap())
            .unwrap()
            .get("against")
            .is_none()
    );
}

/// A query the binder refuses is that error; an id that is no item's is
/// `show`'s own.
#[test]
fn against_a_query_the_binder_refuses_is_that_error() {
    let s = worked();
    let e = against(&s, "r1", "rare").unwrap_err();
    assert_eq!(e["kind"], "bare_word");
    assert_eq!(e["readings"][0], "rarity=rare");
    assert_eq!(
        against(&s, "nothing", "rarity=rare").unwrap_err()["kind"],
        "item_not_found"
    );
    assert_eq!(
        against(&s, "t1", "rarity=rare").unwrap_err()["kind"],
        "not_an_item"
    );
}

/// Bounded as a row is: six of what a term touched, the rest counted.
#[test]
fn against_shows_six_of_what_a_term_touched_and_counts_the_rest() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "Rings")]), 10);
    let spirit: Vec<String> = (1..=8).map(|n| format!("+{n} to Spirit")).collect();
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "Rings",
        vec![ring("many", "Rare", json!({ "explicitMods": spirit }))],
        20,
    );
    for (query, outcome) in [
        ("line(template:spirit)", "matched"),
        ("line(template:spirit arg1>=9)", "failed"),
    ] {
        let shown = against(&s, "many", query).unwrap();
        let node = &shown["against"]["nodes"][0];
        assert_eq!(node["outcome"], outcome, "{query}");
        assert_eq!(node["shows"].as_array().unwrap().len(), 6, "{query}");
        assert_eq!(node["left_out"], 2, "{query}");
    }
}

/// An undecided term says its own reason and no other's (rule 8): an item
/// whose base no class table holds and whose implicit array is unread,
/// against a class and a total — the class's node names the base, the
/// total's the array, neither both.
#[test]
fn against_an_undecided_term_says_its_own_reason_and_no_others() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "Rings")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "Rings",
        vec![item(
            "both",
            "Ring both",
            "Mystery Ring",
            "Rare",
            json!({ "implicitMods": "unreadable", "explicitMods": ["+65% to Fire Resistance"] }),
        )],
        20,
    );
    let shown = against(&s, "both", "class=Rings pseudo.total_res>=60").unwrap();
    let a = &shown["against"];
    assert_eq!(a["outcome"], "undecided");
    assert_eq!(
        outcomes(a),
        pairs(&[("0", "undecided"), ("1", "undecided")])
    );
    for node in a["nodes"].as_array().unwrap() {
        let shows = node["shows"].as_array().unwrap();
        assert_eq!(shows.len(), 1, "{node}");
        assert_eq!(shows[0]["undecided"]["path"], node["path"], "{node}");
    }
    assert_ne!(
        a["nodes"][0]["shows"][0]["undecided"]["unread"],
        a["nodes"][1]["shows"][0]["undecided"]["unread"]
    );
}
