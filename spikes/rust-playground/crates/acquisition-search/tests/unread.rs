//! C93 — what could not be read is never a no: the uncertainty follows
//! the evidence a predicate needs, no further and no less. Each case was
//! reproduced by an outside audit of step 4 (2026-09-20) against the build
//! at `28606ed4`, which answered every one of them definitely.

mod common;

use acquisition_search::{Corpus, answer, show};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

fn one_tab(items: Vec<Value>) -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "T")]), 10);
    fetch_tab(&mut s, "pc", "Standard", "t1", "T", items, 20);
    s
}

fn ring(id: &str, more: Value) -> Value {
    item(id, id, "Iron Ring", "Rare", more)
}

/// (matches, undecided at the root) of a query asked of one item.
fn of(corpus: &Corpus, id: &str, query: &str) -> (u64, u64) {
    let a = as_json(&ask(corpus, &format!("id:{id} ({query})")).unwrap());
    (
        a["total"]["matched"].as_u64().unwrap(),
        a["total"]["undecided"]["count"].as_u64().unwrap(),
    )
}

/// Every count's route returns as many as it counted, and a term's four
/// routes partition the scope.
fn routes_partition(corpus: &Corpus, query: &str, scope: &[&str]) {
    let a = as_json(&ask(corpus, query).unwrap());
    for term in a["terms"].as_array().unwrap() {
        let mut all: Vec<String> = Vec::new();
        for kind in ["matched", "failed", "lacked", "undecided"] {
            let count = &term[kind];
            if count["count"] == 0 {
                continue;
            }
            let request = serde_json::from_value(count["request"].clone()).unwrap();
            let routed = as_json(&answer(corpus, &request).unwrap());
            assert_eq!(
                routed["total"]["matched"], count["count"],
                "{kind} of {} in `{query}`",
                term["term"]
            );
            all.extend(ids(&routed));
        }
        all.sort();
        assert_eq!(all, scope, "{} in `{query}`", term["term"]);
    }
}

/// A line whose flags could not be read is a witness to its text and its
/// numbers, never to a flag it does not name: `-is:crafted` is undecided
/// on it, and the item says why.
#[test]
fn c93_a_line_with_unread_flags_is_no_witness_that_it_is_not_crafted() {
    let life = |flags: Value| json!({ "explicitMods": [{ "description": "+50 to maximum Life", "flags": flags }] });
    let s = one_tab(vec![
        ring("whole", life(json!("not an object"))),
        ring("one", life(json!({ "crafted": "not a boolean" }))),
        ring(
            "yes",
            life(json!({ "crafted": true, "fractured": "not a boolean" })),
        ),
        ring("plain", life(json!({}))),
    ]);
    let corpus = load(&s, Some("pc"));
    for id in ["whole", "one"] {
        assert_eq!(
            of(&corpus, id, "line(template:life -is:crafted)"),
            (0, 1),
            "{id}"
        );
        assert_eq!(
            of(&corpus, id, "line(template:life is:crafted)"),
            (0, 1),
            "{id}"
        );
        // the text and the number were read: they are witnesses still
        assert_eq!(
            of(&corpus, id, "line(template:life arg1>=50)"),
            (1, 0),
            "{id}"
        );
        assert_eq!(of(&corpus, id, "\"maximum life\""), (1, 0), "{id}");
        // and nothing about another line is left open by it
        assert_eq!(of(&corpus, id, "-line(template:strength)"), (1, 0), "{id}");
        // a sum over lines that may or may not be selected is incomplete
        assert_eq!(
            of(&corpus, id, "sum(line(template:life -is:crafted).arg1)>=0"),
            (0, 1),
            "{id}"
        );
    }
    // a flag the line does name is a yes, whatever else was unread
    assert_eq!(of(&corpus, "yes", "line(template:life is:crafted)"), (1, 0));
    assert_eq!(
        of(&corpus, "yes", "line(template:life -is:crafted)"),
        (0, 0)
    );
    assert_eq!(
        of(&corpus, "yes", "line(template:life is:fractured)"),
        (0, 1)
    );
    assert_eq!(
        of(&corpus, "plain", "line(template:life -is:crafted)"),
        (1, 0)
    );

    let a = as_json(&ask(&corpus, "id:one line(template:life -is:crafted)").unwrap());
    let why = &a["total"]["undecided_items"][0]["why"][0];
    assert_eq!(why["unread"], "the flags of explicit lines");
    assert!(
        why["problem"]
            .as_str()
            .unwrap()
            .contains("`flags.crafted` is a string, not yes or no")
    );
    let shown = serde_json::to_value(show(&s, "one", false).unwrap()).unwrap();
    assert_eq!(shown["lines"][0]["flags_unread"], true);
    assert_eq!(shown["item"]["unread"][0]["part"], "flags");
    assert!(
        serde_json::to_value(show(&s, "plain", false).unwrap()).unwrap()["lines"][0]
            .get("flags_unread")
            .is_none()
    );

    // a selector that asks a flag, a comparison that fails: the item did
    // not lack the line, and its route finds it
    let scope = ["one", "plain", "whole", "yes"];
    for query in [
        "line(template:life -is:crafted arg1>=90)",
        "line(template:life -is:crafted)",
        "line(template:life is:crafted arg1>=10)",
    ] {
        routes_partition(&corpus, query, &scope);
    }
    let a = as_json(&ask(&corpus, "line(template:life -is:crafted arg1>=90)").unwrap());
    // whole, one and plain failed; yes is crafted, so it lacked such a line
    assert_eq!(
        (
            &a["terms"][0]["failed"]["count"],
            &a["terms"][0]["lacked"]["count"]
        ),
        (&json!(3), &json!(1))
    );
    assert_eq!(
        a["terms"][0]["failed"]["request"]["query"]["text"],
        "(line(template:life -is:crafted) or undecided(line(template:life -is:crafted))) -line(template:life -is:crafted arg1>=90)"
    );
}

/// An item's flag that is neither yes nor no is unread under its own key;
/// `influences` leaves open the flags that live there, and no other.
#[test]
fn c93_an_items_flag_follows_its_own_key() {
    let s = one_tab(vec![
        ring("odd", json!({ "corrupted": "unread" })),
        ring(
            "no",
            json!({ "corrupted": false, "influences": "not an object" }),
        ),
        ring(
            "yes",
            json!({ "corrupted": true, "influences": { "shaper": "unread" } }),
        ),
    ]);
    let corpus = load(&s, Some("pc"));
    assert_eq!(of(&corpus, "odd", "-is:corrupted"), (0, 1));
    assert_eq!(of(&corpus, "odd", "is:corrupted"), (0, 1));
    assert_eq!(of(&corpus, "odd", "-is:mutated"), (1, 0));
    // corrupted never lives in `influences`
    assert_eq!(of(&corpus, "no", "-is:corrupted"), (1, 0));
    assert_eq!(of(&corpus, "no", "-is:shaper"), (0, 1));
    assert_eq!(of(&corpus, "no", "-is:hunter"), (0, 1));
    assert_eq!(of(&corpus, "yes", "is:corrupted"), (1, 0));
    assert_eq!(of(&corpus, "yes", "-is:shaper"), (0, 1));
    // a flag holds no displayed string: a phrase is not left open by one
    assert_eq!(of(&corpus, "odd", "\"chaos\""), (0, 0));
    routes_partition(&corpus, "-is:corrupted", &["no", "odd", "yes"]);
}

/// A vaal gem's base skill that could not be read is no absence of its
/// lines: `source=hybrid` stays open, under a not and in a sum, and a
/// group that rules the source out is decided all the same.
#[test]
fn c93_an_unread_hybrid_is_no_absence_of_hybrid_lines() {
    let s = one_tab(vec![item(
        "gem",
        "",
        "Vaal Arc",
        "Gem",
        json!({ "hybrid": "unread" }),
    )]);
    let corpus = load(&s, Some("pc"));
    for query in [
        "line(source=hybrid)",
        "-line(source=hybrid)",
        "sum(line(source=hybrid).arg1)>=0",
        "line(template:chains)",
    ] {
        assert_eq!(of(&corpus, "gem", query), (0, 1), "{query}");
    }
    assert_eq!(
        of(&corpus, "gem", "-line(source=explicit template:chains)"),
        (1, 0)
    );
    assert_eq!(of(&corpus, "gem", "undecided(line(source=hybrid))"), (1, 0));
}

/// Which sources a group admits is what it means, never where its
/// parentheses sit.
#[test]
fn c93_parentheses_never_change_which_sources_a_group_admits() {
    let s = one_tab(vec![ring(
        "p",
        json!({ "implicitMods": "unread", "explicitMods": ["+20 to maximum Life"] }),
    )]);
    let corpus = load(&s, Some("pc"));
    for query in [
        "line(source=explicit template:life arg1>=90)",
        "line((source=explicit template:life) arg1>=90)",
        "line(template:life (arg1>=90 source=explicit))",
        "line(-(-source=explicit) template:life arg1>=90)",
        "line(-source=implicit template:life arg1>=90)",
    ] {
        assert_eq!(of(&corpus, "p", query), (0, 0), "{query}");
        assert_eq!(of(&corpus, "p", &format!("-{query}")), (1, 0), "-{query}");
    }
    for query in [
        "line(template:life arg1>=90)",
        "line((source=explicit or source=implicit) template:life arg1>=90)",
        "line(-source=explicit template:life arg1>=90)",
    ] {
        assert_eq!(of(&corpus, "p", query), (0, 1), "{query}");
    }
}

/// Invariant 2: what a selector resolved to is its selector's doing, not
/// its comparisons' — and a `sum`'s selector resolves as a line's does.
#[test]
fn invariant_2_a_selector_resolves_apart_from_its_comparisons() {
    let s = one_tab(vec![
        ring(
            "both",
            json!({ "explicitMods": ["+20% to Fire Resistance", "+80% to Cold Resistance"] }),
        ),
        ring(
            "low",
            json!({ "explicitMods": ["+15% to Chaos Resistance"] }),
        ),
    ]);
    let corpus = load(&s, Some("pc"));
    let every = json!([
        { "value": "#% to Chaos Resistance", "items": 1 },
        { "value": "#% to Cold Resistance", "items": 1 },
        { "value": "#% to Fire Resistance", "items": 1 },
    ]);
    for query in [
        "line(template:resistance arg1>=60)",
        "sum(line(template:resistance).arg1)>=60",
        "line(template:resistance arg1>=1000)",
    ] {
        let a = as_json(&ask(&corpus, query).unwrap());
        assert_eq!(a["terms"][0]["resolved"]["values"], every, "{query}");
    }
    // the row still shows the occurrence that satisfied the whole
    let a = as_json(&ask(&corpus, "line(template:resistance arg1>=60)").unwrap());
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"],
        json!([{ "line": { "source": "explicit", "flags": [], "text": "+80% to Cold Resistance" } }])
    );
}

/// Following an undecided route returns its members with their reasons,
/// past the ten the first answer listed.
#[test]
fn c93_an_undecided_route_carries_the_reasons_to_its_rows() {
    let items: Vec<Value> = (0..12)
        .map(|n| {
            ring(
                &format!("u{n:02}"),
                json!({ "implicitMods": "unread", "explicitMods": ["+20 to maximum Life"] }),
            )
        })
        .collect();
    let s = one_tab(items);
    let corpus = load(&s, Some("pc"));
    let a = as_json(&ask(&corpus, "line(template:life arg1>=90)").unwrap());
    assert_eq!(a["total"]["undecided"]["count"], 12);
    assert_eq!(a["total"]["undecided_items"].as_array().unwrap().len(), 10);
    let request = serde_json::from_value(a["total"]["undecided"]["request"].clone()).unwrap();
    let routed = as_json(&answer(&corpus, &request).unwrap());
    assert_eq!(routed["total"]["matched"], 12);
    for row in routed["rows"].as_array().unwrap() {
        let why = &row["matched"][0]["shows"][0]["undecided"];
        assert_eq!(
            (&why["path"], &why["term"], &why["unread"]),
            (
                &json!("0.0"),
                &json!("line(template:life arg1>=90)"),
                &json!("implicit lines")
            ),
            "{row}"
        );
    }
    // asked of a thing, it says what was unread of the thing
    let by_thing = as_json(&ask(&corpus, "undecided(sum(line(template:life).arg1))").unwrap());
    assert_eq!(
        by_thing["rows"][0]["matched"][0]["shows"][0]["undecided"]["unread"],
        "implicit lines"
    );
}
