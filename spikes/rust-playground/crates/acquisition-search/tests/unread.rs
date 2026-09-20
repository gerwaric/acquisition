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
    // a readable object names the flag it could not read; an unreadable
    // one leaves every flag open
    assert_eq!(shown["lines"][0]["flags_unknown"], json!(["crafted"]));
    assert!(shown["lines"][0].get("flags_unread").is_none());
    let whole = serde_json::to_value(show(&s, "whole", false).unwrap()).unwrap();
    assert_eq!(whole["lines"][0]["flags_unread"], true);
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

/// A flag a readable object says no to is a no, whatever sits beside it:
/// uncertainty is a flag's own, on a line and in `influences`.
#[test]
fn c93_an_unread_flag_never_erases_the_known_no_beside_it() {
    let s = one_tab(vec![ring(
        "f",
        json!({
            "influences": { "shaper": false, "hunter": "unread" },
            "explicitMods": [{ "description": "+50 to maximum Life", "flags": { "crafted": false, "fractured": "unread" } }],
        }),
    )]);
    let corpus = load(&s, Some("pc"));
    assert_eq!(of(&corpus, "f", "line(template:life -is:crafted)"), (1, 0));
    assert_eq!(of(&corpus, "f", "line(template:life is:crafted)"), (0, 0));
    assert_eq!(
        of(&corpus, "f", "line(template:life -is:fractured)"),
        (0, 1)
    );
    assert_eq!(of(&corpus, "f", "-is:shaper"), (1, 0));
    assert_eq!(of(&corpus, "f", "-is:hunter"), (0, 1));
    assert_eq!(of(&corpus, "f", "-is:corrupted"), (1, 0));
    let shown = serde_json::to_value(show(&s, "f", false).unwrap()).unwrap();
    assert_eq!(shown["lines"][0]["flags_unknown"], json!(["fractured"]));
    assert!(shown["lines"][0].get("flags_unread").is_none());
    assert_eq!(
        shown["item"]["unread"][0],
        json!({ "part": "field", "of": "influences.hunter", "problem": "`influences.hunter` is a string, not yes or no" })
    );
}

/// A group's selector is what it means, never where its comparisons sit
/// (the second audit of step 4, finding 1): a nested comparison takes no
/// restriction with it, so the lacked route returns what it counted and
/// the resolved list is the unnested query's.
#[test]
fn c92_a_selector_is_the_groups_meaning_wherever_its_comparisons_sit() {
    let s = one_tab(vec![
        ring("open", json!({ "implicitMods": "unread" })),
        ring(
            "both",
            json!({ "explicitMods": ["+20 to maximum Life", "+80% to Cold Resistance"] }),
        ),
        ring(
            "high",
            json!({ "implicitMods": ["+95 to maximum Life"], "explicitMods": ["+10 to maximum Life"] }),
        ),
        ring("only", json!({ "explicitMods": ["+20 to maximum Life"] })),
    ]);
    let corpus = load(&s, Some("pc"));
    let scope = ["both", "high", "only", "open"];
    for query in [
        "line(template:life (source=explicit arg1>=90))",
        "line((template:life arg1>=60) source=explicit)",
        "line(template:life -(arg1>=90 source=implicit))",
        "line((template:life arg1>=60) or (template:resistance arg1>=90))",
        "line(-(template:life arg1<=50))",
    ] {
        routes_partition(&corpus, query, &scope);
    }
    // `open` has no explicit line and an unread implicit array, which the
    // group rules out: it lacked the line, and the route says so of it
    let a = as_json(&ask(&corpus, "line(template:life (source=explicit arg1>=90))").unwrap());
    assert_eq!(
        (
            &a["terms"][0]["failed"]["count"],
            &a["terms"][0]["lacked"]["count"]
        ),
        (&json!(3), &json!(1))
    );
    assert_eq!(
        a["terms"][0]["lacked"]["request"]["query"]["text"],
        "-line(template:life source=explicit)"
    );
    let nested = as_json(&ask(&corpus, "line((template:life arg1>=60) source=explicit)").unwrap());
    let flat = as_json(&ask(&corpus, "line(template:life arg1>=60 source=explicit)").unwrap());
    assert_eq!(nested["terms"][0]["resolved"], flat["terms"][0]["resolved"]);
    assert_eq!(
        flat["terms"][0]["resolved"]["values"],
        json!([{ "value": "# to maximum Life", "items": 3 }])
    );
    // under a not a comparison is favoured as false: `only`'s one line is a
    // life line of 20, which other numbers would have let through, so the
    // item failed the group and did not lack it. The routes cannot tell —
    // they are made from the selector, so a wrong one stays consistent with
    // itself — and the split is stated by hand.
    let negated = as_json(&ask(&corpus, "line(-(template:life arg1<=50))").unwrap());
    let term = &negated["terms"][0];
    assert_eq!(
        (&term["failed"]["count"], &term["lacked"]["count"]),
        (&json!(1), &json!(0))
    );
    assert_eq!(
        term["failed"]["request"]["query"]["text"],
        "line(true()) -line(-(template:life arg1<=50))"
    );
    let request = serde_json::from_value(term["failed"]["request"].clone()).unwrap();
    assert_eq!(ids(&as_json(&answer(&corpus, &request).unwrap())), ["only"]);
    // the ordinary group's selector is as short as the reference's
    let plain = as_json(&ask(&corpus, "\"+# to maximum Life\">=90").unwrap());
    assert_eq!(
        plain["terms"][0]["failed"]["request"]["query"]["text"],
        "line(\"# to maximum Life\") -line(\"# to maximum Life\" arg1>=90)"
    );
}

/// C92: a largest occurrence is the largest only when nothing unread could
/// hold a larger one — an occurrence the group may select, or a source it
/// admits. What was readable is shown, its status beside it, and the item
/// has no place in the order.
#[test]
fn c92_a_sort_scalar_that_is_not_established_says_so_and_sorts_last() {
    let open = json!({ "description": "+95 to maximum Life", "flags": "unread" });
    let s = one_tab(vec![
        ring(
            "may",
            json!({ "explicitMods": ["+20 to maximum Life", open] }),
        ),
        ring(
            "low",
            json!({ "explicitMods": ["+30 to maximum Life", { "description": "+10 to maximum Life", "flags": "unread" }] }),
        ),
        ring("sure", json!({ "explicitMods": ["+25 to maximum Life"] })),
        ring(
            "src",
            json!({ "implicitMods": "unread", "explicitMods": ["+99 to maximum Life"] }),
        ),
    ]);
    let corpus = load(&s, Some("pc"));
    let sorted = |sort: &str| {
        let request =
            serde_json::from_value(json!({ "view": { "rows": { "sort": sort, "desc": true } } }))
                .unwrap();
        let a = as_json(&answer(&corpus, &request).unwrap());
        let rows: Vec<(String, Value)> = a["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["id"].as_str().unwrap().to_string(), r["sort"].clone()))
            .collect();
        rows
    };
    // `may`'s 95 may be uncrafted; `low`'s open 10 cannot pass its 30
    assert_eq!(
        sorted("line(template:life -is:crafted).arg1"),
        [
            ("low".to_string(), json!({ "value": 30 })),
            ("sure".to_string(), json!({ "value": 25 })),
            (
                "may".to_string(),
                json!({ "value": 20, "status": "incomplete" })
            ),
            (
                "src".to_string(),
                json!({ "value": 99, "status": "incomplete" })
            ),
        ]
    );
    // a group that rules the unread source out has its largest established
    let explicit = sorted("line(template:life source=explicit).arg1");
    assert_eq!(explicit[0], ("src".to_string(), json!({ "value": 99 })));
}

/// An occurrence that names no such number cannot contribute, selected or
/// not (the third audit of step 4, finding 1): whether `crafted` is yes
/// or no, the sum is a complete zero and the projection has no occurrence,
/// so neither is left open by the flag (C93's known absence; C94's sum of
/// nothing). One that does name the number leaves both open.
#[test]
fn c93_an_open_occurrence_without_the_slot_leaves_no_value_open() {
    let open = |text: &str| json!({ "description": text, "flags": { "crafted": "unread" } });
    let s = one_tab(vec![
        ring(
            "frozen",
            json!({ "explicitMods": [open("Cannot be Frozen")] }),
        ),
        ring(
            "life",
            json!({ "explicitMods": [open("+50 to maximum Life")] }),
        ),
    ]);
    let corpus = load(&s, Some("pc"));
    let frozen = "line(template:Frozen -is:crafted).arg1";
    assert_eq!(of(&corpus, "frozen", &format!("sum({frozen})=0")), (1, 0));
    assert_eq!(
        of(&corpus, "frozen", &format!("undecided(sum({frozen}))")),
        (0, 0)
    );
    assert_eq!(
        of(&corpus, "frozen", &format!("undecided({frozen})")),
        (0, 0)
    );
    // the line itself is open still: the flag decides whether it is selected
    assert_eq!(
        of(&corpus, "frozen", "line(template:Frozen -is:crafted)"),
        (0, 1)
    );
    let life = "line(template:life -is:crafted).arg1";
    assert_eq!(of(&corpus, "life", &format!("sum({life})>=0")), (0, 1));
    assert_eq!(
        of(&corpus, "life", &format!("undecided(sum({life}))")),
        (1, 0)
    );
    assert_eq!(of(&corpus, "life", &format!("undecided({life})")), (1, 0));
    // `undecided( … )` of a value and the sort's status are one judgement
    let sorted = |id: &str, sort: &str| {
        let request = serde_json::from_value(json!({ "query": { "text": format!("id:{id}") }, "view": { "rows": { "sort": sort } } })).unwrap();
        as_json(&answer(&corpus, &request).unwrap())["rows"][0]["sort"].clone()
    };
    assert_eq!(
        sorted("frozen", frozen),
        json!({ "status": "no satisfying occurrence" })
    );
    assert_eq!(
        sorted("frozen", &format!("sum({frozen})")),
        json!({ "value": 0 })
    );
    assert_eq!(sorted("life", life), json!({ "status": "incomplete" }));
    assert_eq!(
        sorted("life", &format!("sum({life})")),
        json!({ "value": 0, "status": "incomplete" })
    );
    // the reasons follow: nothing of `frozen` is said to be unread for the sum
    let a = as_json(&ask(&corpus, &format!("sum({life})>=0")).unwrap());
    assert_eq!(
        a["total"]["undecided_items"][0]["why"][0]["unread"],
        "the flags of explicit lines"
    );
}

/// C92's together count is the group's, however its and is parenthesised
/// (the third audit, finding 2): the diagnostic and its route, for both
/// spellings — the four counts alone do not show it.
#[test]
fn c92_parentheses_never_change_whether_the_together_count_applies() {
    let s = one_tab(vec![
        ring(
            "pair",
            json!({ "explicitMods": ["+20 to maximum Life", "+75 to maximum Life"] }),
        ),
        ring(
            "short",
            json!({ "explicitMods": ["+20 to maximum Life", "+30 to maximum Life"] }),
        ),
    ]);
    let corpus = load(&s, Some("pc"));
    let mut seen = Vec::new();
    for query in [
        "line(template:life source=explicit arg1>=90)",
        "line(template:life (source=explicit arg1>=90))",
        "line((template:life (source=explicit (arg1>=90))))",
        "line(template:life source=explicit --arg1>=90)",
    ] {
        let a = as_json(&ask(&corpus, query).unwrap());
        let together = &a["terms"][0]["together"];
        assert_eq!(
            (&together["count"], &together["slot"], &together["bound"]),
            (&json!(1), &json!("arg1"), &json!(90)),
            "{query}"
        );
        let request = serde_json::from_value(together["request"].clone()).unwrap();
        assert_eq!(
            ids(&as_json(&answer(&corpus, &request).unwrap())),
            ["pair"],
            "{query}"
        );
        let term = a["terms"][0]["term"].as_str().unwrap();
        seen.push(
            together["request"]["query"]["text"]
                .as_str()
                .unwrap()
                .replace(term, "<term>"),
        );
    }
    // one selector, one sum, whatever the spelling
    assert_eq!(
        seen[0],
        "line(template:life source=explicit) -<term> sum(line(template:life source=explicit).arg1)>=90"
    );
    assert!(seen.iter().all(|route| *route == seen[0]), "{seen:?}");
    // what is not one lower bound beside a selector stays not applicable, never zero
    for query in [
        "line(template:life (arg1>=90 or arg1<=5))",
        "line(template:life arg1>=90 arg1<=200)",
        "line(template:life -(arg1>=90))",
        "line(template:life arg1=90)",
    ] {
        assert!(
            as_json(&ask(&corpus, query).unwrap())["terms"][0]
                .get("together")
                .is_none(),
            "{query}"
        );
    }
}

/// The build plan's rule 5 against step 1's slot check (the fourth audit of
/// step 4, finding 1): a slot the quoted template does not have is an
/// authoring error wherever the template sits among the group's conjuncts,
/// so no answer can print a route the same build refuses — and a generated
/// route is checked before it is offered, whatever folding made of it.
#[test]
fn rule_5_no_group_is_accepted_whose_route_would_be_refused() {
    let s = one_tab(vec![ring(
        "r",
        json!({ "explicitMods": ["+20 to maximum Life"] }),
    )]);
    let corpus = load(&s, Some("pc"));
    for query in [
        r##"line(("# to maximum Life" source=explicit) arg3>=0)"##,
        r##"line("# to maximum Life" source=explicit arg3>=0)"##,
        r##"line((("# to maximum Life") (source=explicit)) low>=0)"##,
        r##"sum(line(("# to maximum Life" source=explicit)).arg3)>=0"##,
    ] {
        assert_eq!(
            ask(&corpus, query).unwrap_err().to_json()["kind"],
            "slot_unknown",
            "{query}"
        );
    }
    // a template under an or states no numbers: the evaluator's, as step 1
    // says — and folding one into the selector's and must offer no route
    // the checker refuses
    for query in [
        r##"line(("# to maximum Life" or "# to Strength") arg3>=0)"##,
        r##"line(("# to maximum Life" or false()) arg3>=0)"##,
    ] {
        let a = as_json(&ask(&corpus, query).unwrap());
        routes_partition(&corpus, query, &["r"]);
        if let Some(request) = a["terms"][0]["together"].get("request") {
            answer(&corpus, &serde_json::from_value(request.clone()).unwrap()).unwrap();
        }
    }
}

/// The zero block reads the group as the rest of the answer does (the
/// fourth audit, finding 2): a selector that picked occurrences resolved
/// to something, whatever a template test inside it would find alone.
#[test]
fn a_selector_that_resolved_to_something_is_never_said_to_resolve_to_nothing() {
    let s = one_tab(vec![ring(
        "r",
        json!({ "explicitMods": ["+20 to maximum Life"] }),
    )]);
    let corpus = load(&s, Some("pc"));
    for query in [
        "line(template:absent or (template:life arg1>=90))",
        "line(-template:absent arg1>=90)",
        "sum(line(template:life).arg1)>=90",
    ] {
        let a = as_json(&ask(&corpus, query).unwrap());
        assert_eq!(a["total"]["matched"], 0, "{query}");
        assert_eq!(a["zero"]["resolved_to_nothing"], json!([]), "{query}");
    }
    // one that picked nothing is said so, inside a sum too, with what
    // shares its words
    for query in [
        "line(template:lifes arg1>=90)",
        "sum(line(template:lifes).arg1)>=90",
    ] {
        let a = as_json(&ask(&corpus, query).unwrap());
        let nothing = &a["zero"]["resolved_to_nothing"];
        assert_eq!(nothing[0]["of"], "template", "{query}");
        assert_eq!(
            nothing[0]["suggestions"][0]["term"], "line(\"# to maximum Life\")",
            "{query}"
        );
    }
}

/// A number that could not be read is no known absence in the order either
/// (the fourth audit, finding 3): the row says incomplete, as
/// `undecided(ilvl)` says of the same item.
#[test]
fn c93_sorting_by_an_unread_number_says_unread_not_absent() {
    let s = one_tab(vec![
        ring("odd", json!({ "ilvl": "unread" })),
        item("none", "", "Chaos Orb", "Currency", json!({ "ilvl": 0 })),
        ring("has", json!({})),
    ]);
    let corpus = load(&s, Some("pc"));
    let request =
        serde_json::from_value(json!({ "view": { "rows": { "sort": "ilvl" } } })).unwrap();
    let a = as_json(&answer(&corpus, &request).unwrap());
    let sorts: Vec<(&str, &Value)> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), &r["sort"]))
        .collect();
    assert_eq!(sorts[0], ("has", &json!({ "value": 84 })));
    assert!(sorts.contains(&("odd", &json!({ "status": "incomplete" }))));
    assert!(sorts.contains(&("none", &json!({ "status": "no satisfying occurrence" }))));
    assert_eq!(of(&corpus, "odd", "undecided(ilvl)"), (1, 0));
    assert_eq!(of(&corpus, "none", "undecided(ilvl)"), (0, 0));
}
