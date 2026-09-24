//! The crate's boundary (`search/BUILD-PLAN.md`, rule 3): a request in, an
//! answer out, as JSON. The reference's worked example reduced to what is
//! built, every count worked by hand; the route property; invariants 2 and
//! 6 of the surface; C93's composition; C92's sort scalar; C96's scope;
//! C98's basis at the answer's boundary.

mod common;

use acquisition_search::{Corpus, Realm, answer};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

const LIFE: &str = "+# to maximum Life";

fn ring(id: &str, rarity: &str, more: Value) -> Value {
    item(id, &format!("Ring {id}"), "Two-Stone Ring", rarity, more)
}

fn lines(life: &[i64], res: i64) -> Vec<String> {
    life.iter()
        .map(|n| format!("+{n} to maximum Life"))
        .chain([format!("+{res}% to Fire Resistance")])
        .collect()
}

/// The reference's seven pc items (`search/DESIGN.md`, "One worked
/// example"), two fetched locations and one never fetched. `class` and
/// `pseudo.total_res` are not built, so the base stands for the class,
/// one resistance line summed for the total, and r5 — whose base the class
/// table lacks — is an ordinary rare ring.
fn worked() -> Store {
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
            ring("r5", "Rare", json!({ "explicitMods": lines(&[95], 65) })),
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

const WORKED: &str = r##"league=Standard base:ring rarity=rare "+# to maximum Life">=90 sum("+#% to Fire Resistance")>=60"##;

fn counts(term: &Value) -> [u64; 4] {
    ["matched", "failed", "lacked", "undecided"].map(|k| term[k]["count"].as_u64().unwrap())
}

/// Run a count's route as the request it is.
fn follow(corpus: &Corpus, count: &Value) -> Vec<String> {
    let request = serde_json::from_value(count["request"].clone()).unwrap();
    let routed = as_json(&answer(corpus, &request).unwrap());
    assert_eq!(
        routed["total"]["matched"], count["count"],
        "{}",
        count["request"]
    );
    assert_eq!(count["denominator"], "scope");
    ids(&routed)
}

#[test]
fn c100_the_worked_example_reduced_to_what_is_built_counts_as_worked_by_hand() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let a = as_json(&ask(&corpus, WORKED).unwrap());

    // the canonical query, as text and as tree (C104)
    assert_eq!(
        a["query"]["text"],
        r##"league=Standard base:ring rarity=rare line("# to maximum Life" arg1>=90) sum(line("#% to Fire Resistance").arg1)>=60"##
    );
    assert!(a["query"]["tree"]["all"].is_array());

    // the scope says what was searched (C96), the basis what it was read at (C98)
    let scope = &a["scope"];
    assert_eq!(scope["account"], json!({ "uuid": "u-1", "name": "A" }));
    assert_eq!(
        (&scope["realm"], &scope["membership"]),
        (&json!("pc"), &json!("live"))
    );
    assert_eq!(
        (&scope["items"], &scope["fetched"], &scope["never_fetched"]),
        (&json!(7), &json!(2), &json!(1))
    );
    assert_eq!(
        (
            &scope["oldest_fetch"],
            &scope["newest_fetch"],
            &scope["list_seen"]
        ),
        (&json!(20), &json!(30), &json!(10))
    );
    assert_eq!(scope["leagues"], json!(["Standard"]));
    assert_eq!(a["basis"]["account"], "u-1");
    assert_eq!(
        a["basis"]["snapshot"]["response"],
        s.revision().unwrap().response
    );

    // each term over the seven, independently (C93)
    let terms = a["terms"].as_array().unwrap();
    let paths: Vec<&str> = terms.iter().map(|t| t["path"].as_str().unwrap()).collect();
    assert_eq!(paths, ["0", "1", "2", "3", "4"]);
    assert_eq!(counts(&terms[0]), [7, 0, 0, 0]);
    assert_eq!(counts(&terms[1]), [7, 0, 0, 0]);
    assert_eq!(counts(&terms[2]), [6, 1, 0, 0]); // r4 is magic
    // r2 holds 20 and 75: failed, and the one that reaches 90 only
    // together; r7 has no life line; r6's explicit 95 is a witness
    assert_eq!(counts(&terms[3]), [5, 1, 1, 0]);
    assert_eq!(terms[3]["together"]["count"], 1);
    assert_eq!(
        (
            &terms[3]["together"]["slot"],
            &terms[3]["together"]["bound"]
        ),
        (&json!("arg1"), &json!(90))
    );
    // r3 sums to 55; r6's sum is an incomplete subtotal
    assert_eq!(counts(&terms[4]), [5, 1, 0, 1]);
    assert!(
        terms[0].get("together").is_none(),
        "not applicable is absent, never zero"
    );

    // r1 and r5; r6 is undecided at the root and is no match (C93)
    assert_eq!(a["total"]["matched"], 2);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    assert_eq!(ids(&a), ["r1", "r5"]);
    let why = &a["total"]["undecided_items"][0];
    assert_eq!(why["id"], "r6");
    assert_eq!(why["why"][0]["path"], "4");
    assert_eq!(why["why"][0]["unread"], "implicit lines");
    assert!(why["why"][0]["hint"].as_str().unwrap().contains("may help"));

    // a row: the header, the place by name and id, the lines the query touched
    let r1 = &a["rows"][0];
    assert_eq!(
        (&r1["id"], &r1["base"], &r1["rarity"]),
        (&json!("r1"), &json!("Two-Stone Ring"), &json!("Rare"))
    );
    assert_eq!(r1["place"]["league"], "Standard");
    assert_eq!(
        (&r1["place"]["name"], &r1["place"]["id"]),
        (&json!("Rings"), &json!("t1"))
    );
    let shows: Vec<&Value> = r1["matched"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["shows"].as_array().unwrap())
        .collect();
    assert!(shows.contains(
        &&json!({ "line": { "source": "explicit", "flags": [], "text": "+95 to maximum Life" } })
    ));
    assert!(shows.contains(
        &&json!({ "value": { "name": "sum(line(\"#% to Fire Resistance\").arg1)", "value": 65 } })
    ));
    assert!(a.get("zero").is_none());

    // the routes the example names, by their members
    assert_eq!(follow(&corpus, &terms[3]["failed"]), ["r2"]);
    assert_eq!(
        terms[3]["failed"]["request"]["query"]["text"],
        r##"line("# to maximum Life") -line("# to maximum Life" arg1>=90)"##
    );
    assert_eq!(follow(&corpus, &terms[3]["together"]), ["r2"]);
    assert_eq!(follow(&corpus, &terms[3]["lacked"]), ["r7"]);
    assert_eq!(follow(&corpus, &terms[4]["undecided"]), ["r6"]);
    assert_eq!(follow(&corpus, &a["total"]["undecided"]), ["r6"]);
    assert_eq!(terms[3]["failed"]["counted_at"], a["basis"]);
}

/// Invariant 4, and rule 5 of the plan: every count an answer shows has a
/// route, the route is a request this build runs, and it returns exactly
/// the members counted — per term the four routes are disjoint and cover
/// the scope.
#[test]
fn the_route_property_every_count_routes_to_exactly_its_members() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    for text in [
        WORKED,
        "",
        r##"rarity=rare or "+# to maximum Life">=100"##,
        r##"-rarity=magic -line("# to maximum Life" arg1>=90)"##,
        r##"holds(rarity=rare, "+# to maximum Life">=95, sum("+#% to Fire Resistance")>=65)>=2"##,
        r##"undecided(sum("+#% to Fire Resistance")>=60) or tab:dump"##,
        r##"line(template:life source=implicit) name:r2 has:note -has:ilvl is:identified "two-stone""##,
        r##"line("# to maximum Life" (arg1>=100 or arg1<=20))"##,
        r##"character:mover league:hard ilvl=80..86 id:t2 true() false()"##,
        // comparisons nested where a selector by syntax would lose them
        r##"line(template:life (source=explicit arg1>=90))"##,
        r##"line((template:life arg1>=96) or (template:resistance arg1>=65))"##,
        r##"line(template:life -(arg1>=90 source=implicit))"##,
    ] {
        let a = as_json(&ask(&corpus, text).unwrap());
        for term in a["terms"].as_array().unwrap() {
            let mut all: Vec<String> = Vec::new();
            for kind in ["matched", "failed", "lacked", "undecided"] {
                let count = &term[kind];
                if count["count"] == 0 {
                    assert!(count.get("request").is_none());
                    continue;
                }
                all.extend(follow(&corpus, count));
            }
            all.sort();
            let whole: Vec<String> = ["r1", "r2", "r3", "r4", "r5", "r6", "r7"]
                .map(String::from)
                .to_vec();
            assert_eq!(
                all, whole,
                "`{}` in `{text}`: its four routes partition the scope",
                term["term"]
            );
            if term["together"]["count"].as_u64().is_some_and(|n| n > 0) {
                follow(&corpus, &term["together"]);
            }
        }
        if a["total"]["undecided"]["count"] != 0 {
            follow(&corpus, &a["total"]["undecided"]);
        }
        // every command an answer would print is one the parser takes
        let text = a["query"]["text"].as_str().unwrap();
        assert_eq!(as_json(&ask(&corpus, text).unwrap())["query"]["text"], text);
    }
}

/// Invariant 2: a `:` or `~` selector prints back as authored, with what
/// it resolved to beside it.
#[test]
fn invariant_2_a_selector_is_printed_as_authored_with_its_binding_beside_it() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let a = as_json(
        &ask(
            &corpus,
            r##"base:ring line(template~"resist" arg1>=60) rarity:ra"##,
        )
        .unwrap(),
    );
    assert_eq!(
        a["query"]["text"],
        r##"base:ring line(template~"resist" arg1>=60) rarity:ra"##
    );
    let terms = a["terms"].as_array().unwrap();
    assert_eq!(
        terms[0]["resolved"],
        json!({ "of": "base", "values": [{ "value": "Two-Stone Ring", "items": 7 }], "more": 0 })
    );
    assert_eq!(
        terms[1]["resolved"],
        json!({ "of": "template", "values": [{ "value": "#% to Fire Resistance", "items": 7 }], "more": 0 })
    );
    // `ra` picks `rare` among the legal values and no other
    assert_eq!(
        terms[2]["resolved"]["values"],
        json!([{ "value": "Rare", "items": 6 }])
    );
    // an exact selector resolves to nothing but itself
    let a = as_json(&ask(&corpus, "base=\"Two-Stone Ring\"").unwrap());
    assert!(a["terms"][0].get("resolved").is_none());
}

/// Invariant 6: search is a pure read — every table of the facts file is
/// the same before and after.
#[test]
fn invariant_6_a_search_writes_nothing() {
    fn dump(path: &std::path::Path) -> Vec<String> {
        let conn = rusqlite::Connection::open(path).unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        let mut out = Vec::new();
        for table in tables {
            let mut stmt = conn.prepare(&format!("SELECT * FROM \"{table}\"")).unwrap();
            let n = stmt.column_count();
            let mut rows = stmt.query([]).unwrap();
            while let Some(row) = rows.next().unwrap() {
                let cells: Vec<String> = (0..n)
                    .map(|i| format!("{:?}", row.get_ref(i).unwrap()))
                    .collect();
                out.push(format!("{table}: {}", cells.join(" | ")));
            }
        }
        out
    }
    let s = worked();
    let path = s.path().to_path_buf();
    let before = dump(&path);
    assert!(before.len() > 10);
    let corpus = load(&s, None);
    for text in [WORKED, "", r##"name="nothing here""##] {
        ask(&corpus, text).unwrap();
    }
    acquisition_search::show(&s, "r1", true).unwrap();
    assert_eq!(dump(&path), before);
}

/// C93 — composition over an undecided term, asked of r6, whose sum is an
/// incomplete subtotal.
#[test]
fn c93_undecided_composes_and_is_never_a_match() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let open = r##"sum("+#% to Fire Resistance")>=60"##;
    let of_r6 = |text: &str| {
        let a = as_json(&ask(&corpus, &format!("id:r6 ({text})")).unwrap());
        (
            a["total"]["matched"].as_u64().unwrap(),
            a["total"]["undecided"]["count"].as_u64().unwrap(),
        )
    };
    assert_eq!(of_r6(&format!("{open} rarity=rare")), (0, 1));
    assert_eq!(
        of_r6(&format!("{open} or rarity=rare")),
        (1, 0),
        "true or undecided is true"
    );
    assert_eq!(
        of_r6(&format!("{open} rarity=magic")),
        (0, 0),
        "false and undecided is false"
    );
    assert_eq!(
        of_r6(&format!("-{open} rarity=rare")),
        (0, 1),
        "not undecided is undecided"
    );
    // one sure, one open: the count is in [1, 2]
    assert_eq!(of_r6(&format!("holds(rarity=rare, {open})>=1")), (1, 0));
    assert_eq!(of_r6(&format!("holds(rarity=rare, {open})>=2")), (0, 1));
    assert_eq!(of_r6(&format!("holds(rarity=rare, {open})<=1")), (0, 1));
    assert_eq!(of_r6(&format!("holds(rarity=magic, {open})>=2")), (0, 0));
    // asked for by name it is decided, and known absence does not satisfy it
    assert_eq!(of_r6(&format!("undecided({open}) rarity=rare")), (1, 0));
    assert_eq!(of_r6("undecided(name) or undecided(league)"), (0, 0));
    // a readable line is a witness; a readable miss beside an unread array is not
    assert_eq!(of_r6(r##""+# to maximum Life">=90 rarity=rare"##), (1, 0));
    assert_eq!(of_r6(r##""+# to maximum Life">=96 rarity=rare"##), (0, 1));
    // the group's own source rules the unread array out
    assert_eq!(
        of_r6(r##"line("# to maximum Life" source=explicit arg1>=96) or rarity=magic"##),
        (0, 0)
    );
    // a phrase: a hit in what is readable matches; no hit is undecided
    assert_eq!(of_r6(r##""maximum life" rarity=rare"##), (1, 0));
    assert_eq!(of_r6(r##""chaos" rarity=rare"##), (0, 1));
}

/// C92 — the sort scalar is the largest satisfying occurrence; an item
/// with none sorts last in either direction, its status shown.
#[test]
fn c92_the_sort_scalar_and_an_item_without_one_last_either_way() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let sorted = |desc: bool| {
        let request = serde_json::from_value(json!({
            "scope": { "realm": "pc" },
            "query": { "text": "rarity=rare" },
            "view": { "rows": { "sort": format!("line(\"{LIFE}\").arg1"), "desc": desc } },
        }))
        .unwrap();
        let a = as_json(&answer(&corpus, &request).unwrap());
        assert_eq!(
            a["view"]["rows"]["sort"],
            "line(\"# to maximum Life\").arg1"
        );
        let rows: Vec<(String, Value)> = a["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["id"].as_str().unwrap().to_string(), r["sort"].clone()))
            .collect();
        rows
    };
    let down = sorted(true);
    let order: Vec<&str> = down.iter().map(|(id, _)| id.as_str()).collect();
    // r3 100; r1 and r5 95 in the store's order; r2's largest is 75; r6's
    // 95 is no largest while its implicit array is unread; r7 has none
    assert_eq!(order, ["r3", "r1", "r5", "r2", "r6", "r7"]);
    assert_eq!(down[3].1, json!({ "value": 75 }));
    assert_eq!(down[4].1, json!({ "value": 95, "status": "incomplete" }));
    assert_eq!(down[5].1, json!({ "status": "no satisfying occurrence" }));
    let up = sorted(false);
    let order: Vec<&str> = up.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(order, ["r2", "r1", "r5", "r3", "r6", "r7"]);

    // the limit counts what it leaves out, and names no command this build refuses
    let request = serde_json::from_value(json!({ "view": { "rows": { "limit": 2 } } })).unwrap();
    let a = as_json(&answer(&corpus, &request).unwrap());
    assert_eq!(
        a["view"]["rows"],
        json!({ "limit": 2, "returned": 2, "left_out": 5, "left_out_needs": "--next" })
    );
    assert_eq!(a["total"]["matched"], 7);
}

/// C96 — over a store holding more than one realm a search names one; all
/// is a scope of its own; a realm the store lacks is a valid scope of
/// nothing, and says so.
#[test]
fn c96_the_realm_is_the_scope_and_is_always_resolved() {
    let mut s = worked();
    // one realm held: none named resolves to it, and the answer prints it
    assert_eq!(load(&s, None).realm, Realm::One("pc".to_string()));
    list_tabs(&mut s, "xbox", "Standard", json!([tab("x1", "Box")]), 40);
    fetch_tab(
        &mut s,
        "xbox",
        "Standard",
        "x1",
        "Box",
        vec![ring("rx", "Rare", json!({}))],
        41,
    );
    let e = Corpus::load(&s, None).unwrap_err();
    assert_eq!(e.to_json()["kind"], "realm_needed");
    assert_eq!(
        e.to_json()["readings"],
        json!(["--realm pc", "--realm xbox"])
    );
    let all = load(&s, Some("all"));
    let a = as_json(&ask(&all, "").unwrap());
    assert_eq!(
        (&a["scope"]["realm"], &a["total"]["matched"]),
        (&json!("all"), &json!(8))
    );
    assert_eq!(
        a["terms"],
        json!([]),
        "the empty query: every item in scope"
    );
    let pc = as_json(&ask(&load(&s, Some("pc")), "").unwrap());
    assert_eq!(
        (&pc["scope"]["items"], &pc["scope"]["fetched"]),
        (&json!(7), &json!(2))
    );
    // a route carries its realm resolved
    let xbox = as_json(&ask(&load(&s, Some("xbox")), "rarity=rare").unwrap());
    assert_eq!(
        xbox["terms"][0]["matched"]["request"]["scope"],
        json!({ "account": "A", "realm": "xbox", "membership": "live" })
    );
    let sony = as_json(&ask(&load(&s, Some("sony")), "").unwrap());
    assert_eq!(
        (&sony["scope"]["items"], &sony["scope"]["realm_not_held"]),
        (&json!(0), &json!(true))
    );
    assert_eq!(
        Realm::parse("ps4").unwrap_err().to_json()["kind"],
        "realm_unknown"
    );
    // a request bound to another account, or another realm than the corpus holds, is refused
    let other = serde_json::from_value(json!({ "scope": { "account": "B" } })).unwrap();
    assert_eq!(
        answer(&all, &other).unwrap_err().to_json()["kind"],
        "account_mismatch"
    );
    let other = serde_json::from_value(json!({ "scope": { "realm": "pc" } })).unwrap();
    assert_eq!(
        answer(&all, &other).unwrap_err().to_json()["kind"],
        "realm_mismatch"
    );
}

/// C98, at the answer's boundary: an answer from a held corpus describes
/// the state its basis names, whatever landed since; the check before
/// every answer sees the change, and the next load carries it.
#[test]
fn c98_a_held_corpus_answers_at_its_basis_and_the_check_sees_the_change() {
    let s = worked();
    let path = s.path().to_path_buf();
    let held = load(&s, Some("pc"));
    assert!(held.is_current(&s).unwrap());
    let mut other = Store::open(&path).unwrap();
    fetch_tab(
        &mut other,
        "pc",
        "Standard",
        "t3",
        "Never",
        vec![ring("late", "Rare", json!({}))],
        50,
    );

    let a = as_json(&ask(&held, "").unwrap());
    assert_eq!(
        (&a["total"]["matched"], &a["scope"]["never_fetched"]),
        (&json!(7), &json!(1))
    );
    assert_eq!(a["basis"], serde_json::to_value(&held.basis).unwrap());
    assert!(!held.is_current(&s).unwrap());

    let next = load(&s, Some("pc"));
    assert!(next.basis.snapshot.response > held.basis.snapshot.response);
    let a = as_json(&ask(&next, "").unwrap());
    assert_eq!(
        (&a["total"]["matched"], &a["scope"]["never_fetched"]),
        (&json!(8), &json!(0))
    );
}

/// C98 — the basis names the store (outside audit, 2026-09-20;
/// owner, 2026-09-20: "(a') now and park (c)"): two facts files of one
/// account at one revision are two stores, their bases differ, and a
/// corpus read from one is not current against the other.
#[test]
fn c98_the_basis_names_the_store_and_the_check_tells_two_files_apart() {
    let (a, b) = (worked(), worked());
    assert_eq!(a.revision().unwrap(), b.revision().unwrap());
    let (of_a, of_b) = (load(&a, Some("pc")), load(&b, Some("pc")));
    assert_eq!(of_a.basis.account, of_b.basis.account);
    assert_ne!(of_a.basis.store, of_b.basis.store);
    assert!(of_a.is_current(&a).unwrap() && !of_a.is_current(&b).unwrap());
    // twelve hex digits, no path: the same file under another handle is the same store
    let again = load(&Store::open(a.path()).unwrap(), Some("pc"));
    assert_eq!(again.basis, of_a.basis);
    assert!(
        of_a.basis.store.len() == 12 && of_a.basis.store.chars().all(|c| c.is_ascii_hexdigit())
    );
    let answered = as_json(&ask(&of_a, "").unwrap());
    assert_eq!(answered["basis"]["store"], of_a.basis.store.as_str());
    assert!(
        !answered
            .to_string()
            .contains(a.path().parent().unwrap().to_str().unwrap())
    );
}

/// C104 — a request sends the text or the tree, and the answer returns both.
#[test]
fn c104_a_tree_is_accepted_and_the_answer_returns_both() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let by_text = as_json(&ask(&corpus, WORKED).unwrap());
    let request =
        serde_json::from_value(json!({ "query": { "tree": by_text["query"]["tree"] } })).unwrap();
    let by_tree = as_json(&answer(&corpus, &request).unwrap());
    assert_eq!(by_tree, by_text);
    let both = serde_json::from_value(
        json!({ "query": { "text": "", "tree": by_text["query"]["tree"] } }),
    )
    .unwrap();
    assert_eq!(
        answer(&corpus, &both).unwrap_err().to_json()["kind"],
        "tree"
    );
    assert!(serde_json::from_value::<acquisition_search::Request>(json!({ "views": {} })).is_err());
}
