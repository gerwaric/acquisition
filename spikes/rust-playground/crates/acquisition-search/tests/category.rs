//! Category (C106; `search/LEDGER.md`, "Holes ruled", J1–J4 and J9–J22),
//! through the crate's boundary: the trade site's 82 category ids asked as
//! terms, counted, shown and described.
//!
//! The first test reads the track's expected answers where they lie
//! (`search/category/data/expected-answers.csv`, written before anything
//! was built, from the site's captures and the owner's rulings) and holds
//! the build to every row. The others are fixtures worked by hand before
//! running, from the rulings and C93: a beast found by its category, a
//! category under `not` and `or`, `show`, the help.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use acquisition_search::{Corpus, Request, answer, describe};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

/// The expected answers, one row an item, and the ids with the copy's
/// counts: the tests' side, never read by the build.
const EXPECTED: &str = include_str!("../../../search/category/data/expected-answers.csv");
const EXPECTED_COPY: &str = include_str!("../../../search/category/data/expected-copy.csv");

/// The line a captured beast's body carries (`search/category/README.md`,
/// F3: `descrText` on all 158 of the copy's).
const BESTIARY: &str = "Right-click to add this to your bestiary.";

// ---- reading the table ------------------------------------------------------------------------

/// A CSV text as rows of fields: a comment line dropped, a quoted field
/// kept whole with its commas, a doubled quote one quote.
fn csv(text: &str) -> Vec<Vec<String>> {
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
fn table(text: &str) -> Vec<BTreeMap<String, String>> {
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

/// The 82 ids, in the site's order: the id column of the copy's table
/// before its two buckets.
fn the_ids() -> Vec<String> {
    let ids: Vec<String> = table(EXPECTED_COPY)
        .into_iter()
        .map(|row| row["id"].clone())
        .take_while(|id| !id.starts_with('('))
        .collect();
    assert_eq!(ids.len(), 82);
    ids
}

const MONSTER: [&str; 3] = ["monster.beast", "monster.yellowbeast", "monster.redbeast"];

/// One row of the expected answers.
struct Case {
    /// The item's id in the fixture store: the row's place in the table.
    id: String,
    realm: String,
    base: String,
    frame: String,
    bestiary: bool,
    /// The class or classes (`A and B`), empty where not established.
    classes: BTreeSet<String>,
    class_not_established: String,
    yes: BTreeSet<String>,
    /// Each id left open, with the cause the table gives.
    undecided: BTreeMap<String, String>,
    /// The ids a count lists it under, or `(none)`, or `(undecided)`.
    listed: BTreeSet<String>,
}

fn cases() -> Vec<Case> {
    let ids = the_ids();
    let words =
        |text: &str| -> BTreeSet<String> { text.split_whitespace().map(str::to_string).collect() };
    table(EXPECTED)
        .into_iter()
        .enumerate()
        .map(|(n, row)| {
            let mut undecided = BTreeMap::new();
            for part in row["ids_undecided"].split("; ").filter(|p| !p.is_empty()) {
                let (which, cause) = part.split_once(": ").unwrap();
                let open: Vec<String> = match which {
                    "every id" => ids.clone(),
                    "every id but the three monster.* leaves" => ids
                        .iter()
                        .filter(|id| !MONSTER.contains(&id.as_str()))
                        .cloned()
                        .collect(),
                    listed => words(listed).into_iter().collect(),
                };
                for id in open {
                    assert!(ids.contains(&id), "{id}");
                    undecided.insert(id, cause.to_string());
                }
            }
            let listed = match row["listed_under"].as_str() {
                "none" => BTreeSet::from(["(none)".to_string()]),
                "undecided" => BTreeSet::from(["(undecided)".to_string()]),
                listed => words(listed),
            };
            assert!(row["ids_unruled"].is_empty(), "an unruled cell: row {n}");
            Case {
                id: format!("r{n:03}"),
                realm: row["realm"].clone(),
                base: row["base"].clone(),
                frame: row["frame"].clone(),
                bestiary: row["bestiary_line"] == "yes",
                classes: row["class"]
                    .split(" and ")
                    .filter(|c| !c.is_empty())
                    .map(str::to_string)
                    .collect(),
                class_not_established: row["class_not_established"].clone(),
                yes: words(&row["ids_yes"]),
                undecided,
                listed,
            }
        })
        .collect()
}

/// An item of a row's facts, as GGG would give it: the base as shown, the
/// frame, the bestiary line; nothing where the row has nothing.
fn body_of(case: &Case, at: usize) -> Value {
    let mut body = json!({ "id": case.id, "name": "", "identified": true, "ilvl": 0,
                           "x": at % 24, "y": at / 24 });
    if !case.base.is_empty() {
        body["typeLine"] = json!(case.base);
        body["baseType"] = json!(case.base);
    }
    if !case.frame.is_empty() {
        body["frameTypeId"] = json!(case.frame);
    }
    if case.bestiary {
        body["descrText"] = json!(BESTIARY);
    }
    body
}

fn rows_of(corpus: &Corpus, text: &str) -> Value {
    let request: Request = serde_json::from_value(json!({
        "query": { "text": text },
        "view": { "rows": { "limit": 5000 } },
    }))
    .unwrap();
    as_json(&answer(corpus, &request).unwrap())
}

fn id_set(a: &Value) -> BTreeSet<String> {
    ids(a).into_iter().collect()
}

fn counted_by(corpus: &Corpus, text: &str, keys: &[&str]) -> Value {
    let request: Request = serde_json::from_value(json!({
        "query": { "text": text },
        "view": { "counts": { "keys": keys, "limit": 500 } },
    }))
    .unwrap();
    as_json(&answer(corpus, &request).unwrap())
}

/// A bucket's name: its value, or `(none)`, `(undecided)`.
fn bucket_name(bucket: &Value) -> String {
    match bucket["bucket"].as_str().unwrap() {
        "value" => bucket["value"].as_str().unwrap().to_string(),
        other => format!("({other})"),
    }
}

/// The items a count's route returns, which are as many as it counted
/// (invariant 4 of the surface).
fn members(corpus: &Corpus, counted: &Value) -> BTreeSet<String> {
    let n = counted["count"].as_u64().unwrap();
    let Some(route) = counted.get("request") else {
        assert_eq!(n, 0, "a count with no route: {counted}");
        return BTreeSet::new();
    };
    let mut route = route.clone();
    route["view"]["rows"]["limit"] = json!(5000);
    let request: Request = serde_json::from_value(route.clone()).unwrap();
    let routed = id_set(&as_json(&answer(corpus, &request).unwrap()));
    assert_eq!(
        routed.len() as u64,
        n,
        "a route returns what was counted: {route}"
    );
    routed
}

/// Each item with the buckets a count by `key` lists it under, read by
/// following every bucket's route.
fn listing(corpus: &Corpus, key: &str) -> BTreeMap<String, BTreeSet<String>> {
    let a = counted_by(corpus, "", &[key]);
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(table["left_out"], 0);
    let mut under: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for bucket in table["buckets"].as_array().unwrap() {
        for item in members(corpus, bucket) {
            under.entry(item).or_default().insert(bucket_name(bucket));
        }
    }
    under
}

fn shown(s: &Store, id: &str) -> Value {
    serde_json::to_value(show(s, id, false).unwrap()).unwrap()
}

/// What `show` says left an id open on the item: the part, and the gap
/// under it where the part has one.
fn open_by(shown: &Value, id: &str) -> Vec<(String, String)> {
    shown["category"]["open"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|open| open["ids"].as_array().unwrap().contains(&json!(id)))
        .map(|open| {
            (
                open["part"].as_str().unwrap().to_string(),
                open["of"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect()
}

/// The gap `class.rs` names for a cause of the table: the table's two
/// causes for a base with no class — the table lacks it, the export
/// prints no name for its class — are one gap in this build.
fn gap_of(cause: &str) -> &'static str {
    match cause {
        "the class: base under several classes" => "several",
        "the class: no table for the realm" => "no_table_for_realm",
        "the class: base not in the table"
        | "the class: the export prints no name for the base's class" => "not_in_table",
        "the class: no base" => "no_base",
        other => panic!("a cause the test does not know: {other}"),
    }
}

// ---- the table ------------------------------------------------------------------------------------

/// C106 (category), J1–J4, J9–J22: every row of the expected answers —
/// 553 items of the facts the row gives, put into a store through its
/// ingest — is answered as the row says. Each of the 82 ids asked as a
/// term matches the rows listing it under `ids_yes`, is undecided on the
/// rows listing it under `ids_undecided` and fails every other; a count
/// by category lists each row under `listed_under`; the class is the
/// row's, or not established by the gap its cause names; an id is left
/// open by the class or by the colour, as the row's cause says.
#[test]
fn c106_every_row_of_the_expected_answers_is_answered_as_the_row_says() {
    let cases = cases();
    assert_eq!(cases.len(), 553);
    let ids = the_ids();
    let mut s = store();
    let mut faults: Vec<String> = Vec::new();
    for realm in ["pc", "poe2"] {
        let of_realm: Vec<&Case> = cases.iter().filter(|c| c.realm == realm).collect();
        assert!(!of_realm.is_empty(), "{realm}");
        list_tabs(&mut s, realm, "Standard", json!([tab("t", "Table")]), 10);
        fetch_tab(
            &mut s,
            realm,
            "Standard",
            "t",
            "Table",
            of_realm
                .iter()
                .enumerate()
                .map(|(at, case)| body_of(case, at))
                .collect(),
            20,
        );
    }
    assert_eq!(
        cases.iter().filter(|c| c.realm == "pc").count()
            + cases.iter().filter(|c| c.realm == "poe2").count(),
        553
    );
    for realm in ["pc", "poe2"] {
        let corpus = load(&s, Some(realm));
        let of_realm: Vec<&Case> = cases.iter().filter(|c| c.realm == realm).collect();
        let all: BTreeSet<String> = of_realm.iter().map(|c| c.id.clone()).collect();
        assert_eq!(id_set(&rows_of(&corpus, "")), all);

        // every id as a term: matched, undecided, and every other row failed
        for id in &ids {
            let yes: BTreeSet<String> = of_realm
                .iter()
                .filter(|c| c.yes.contains(id))
                .map(|c| c.id.clone())
                .collect();
            let open: BTreeSet<String> = of_realm
                .iter()
                .filter(|c| c.undecided.contains_key(id))
                .map(|c| c.id.clone())
                .collect();
            let asked = rows_of(&corpus, &format!("category=\"{id}\""));
            let matched = id_set(&asked);
            let undecided = id_set(&rows_of(&corpus, &format!("undecided(category=\"{id}\")")));
            for case in &of_realm {
                let says = (yes.contains(&case.id), open.contains(&case.id));
                let built = (matched.contains(&case.id), undecided.contains(&case.id));
                if says != built {
                    faults.push(format!(
                        "{} `{}` ({}, frame `{}`): category={id} — the table says (yes, undecided) {says:?}, the build {built:?}",
                        case.id, case.base, case.realm, case.frame
                    ));
                }
            }
            // the term's own counts say the same, and nothing else is open
            let term = &asked["terms"][0];
            let n = |k: &str| term[k]["count"].as_u64().unwrap() as usize;
            if (n("matched"), n("undecided")) != (yes.len(), open.len())
                || n("matched") + n("failed") + n("lacked") + n("undecided") != of_realm.len()
            {
                faults.push(format!(
                    "category={id} in {realm}: the term's counts {term}"
                ));
            }
        }

        // the count by category: each row under what the table lists
        let listed = listing(&corpus, "category");
        for case in &of_realm {
            let built = listed.get(&case.id).cloned().unwrap_or_default();
            if built != case.listed {
                faults.push(format!(
                    "{} `{}`: listed under {:?}, the table says {:?}",
                    case.id, case.base, built, case.listed
                ));
            }
        }

        // the class: the row's, or not established by the gap its cause names
        let classed = listing(&corpus, "class");
        for case in &of_realm {
            let built = classed.get(&case.id).cloned().unwrap_or_default();
            let says = if case.classes.is_empty() {
                BTreeSet::from(["(undecided)".to_string()])
            } else {
                case.classes.clone()
            };
            if built != says {
                faults.push(format!(
                    "{} `{}`: class {:?}, the table says {:?}",
                    case.id, case.base, built, says
                ));
            }
        }
    }
    // what could not be established, by `show`: the class's gap, and what
    // left each id open — the class by that gap, or the colour
    for case in cases
        .iter()
        .filter(|c| !c.class_not_established.is_empty() || !c.undecided.is_empty())
    {
        let shown = shown(&s, &case.id);
        if !case.class_not_established.is_empty() {
            let class = &shown["class"];
            let gap = class["open"]["of"]
                .as_str()
                .or_else(|| class["among"]["why"]["of"].as_str())
                .unwrap_or_default();
            if gap != gap_of(&case.class_not_established) {
                faults.push(format!(
                    "{} `{}`: the class is open by `{gap}` ({class}), the table says {}",
                    case.id, case.base, case.class_not_established
                ));
            }
        }
        for (id, cause) in &case.undecided {
            let says = if cause.starts_with("the colour: ") {
                ("colour".to_string(), String::new())
            } else {
                ("class".to_string(), gap_of(cause).to_string())
            };
            let built = open_by(&shown, id);
            if built != [says.clone()] {
                faults.push(format!(
                    "{} `{}`: {id} is left open by {built:?}, the table says {says:?}",
                    case.id, case.base
                ));
            }
        }
    }
    assert!(
        faults.is_empty(),
        "{} faults against the expected answers, the first 40:\n{}",
        faults.len(),
        faults
            .iter()
            .take(40)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ---- a beast found by its category ----------------------------------------------------------------

fn plain(id: &str, base: &str, frame: &str, more: Value) -> Value {
    let mut body = json!({ "id": id, "name": "", "typeLine": base, "baseType": base,
        "frameTypeId": frame, "identified": true, "ilvl": 0, "x": 0, "y": 0 });
    if base.is_empty() {
        body.as_object_mut().unwrap().remove("typeLine");
        body.as_object_mut().unwrap().remove("baseType");
    }
    if frame.is_empty() {
        body.as_object_mut().unwrap().remove("frameTypeId");
    }
    for (key, value) in more.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

fn beast(id: &str, base: &str) -> Value {
    plain(id, base, "Rare", json!({ "descrText": BESTIARY }))
}

/// Six items in `Menagerie` (m1), worked by hand from J1, J9, J10, J15
/// and the colour table (`search/category/data/beast-colours.csv`):
///
/// - `hellion`, a captured Dune Hellion: yellow by the table's row (k022);
/// - `ursa`, a captured Farric Ursa: red (l011);
/// - `crab`, a captured Bleached Crustacean, which the table marks unknown;
/// - `nobody`, a captured beast of a name the table lacks;
/// - `chaos`, a stack of Chaos Orbs: Stackable Currency, `currency`;
/// - `ghost`, an item named Dune Hellion with no bestiary line: no beast,
///   since a beast is known by its line and never by its name, and a base
///   the class table lacks.
fn menagerie() -> Store {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("m1", "Menagerie")]),
        10,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "m1",
        "Menagerie",
        vec![
            beast("hellion", "Dune Hellion"),
            beast("ursa", "Farric Ursa"),
            beast("crab", "Bleached Crustacean"),
            beast("nobody", "A Beast The Table Lacks"),
            plain("chaos", "Chaos Orb", "Currency", json!({ "stackSize": 7 })),
            plain("ghost", "Dune Hellion", "Rare", json!({})),
        ],
        20,
    );
    s
}

fn asked(corpus: &Corpus, text: &str) -> Value {
    as_json(&ask(corpus, text).unwrap())
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

/// The four counts of a term, each returning by its route as many items
/// as it counted (invariant 4): matched, failed, lacked, undecided.
fn routed(corpus: &Corpus, a: &Value, path: &str) -> [BTreeSet<String>; 4] {
    let t = a["terms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["path"] == path)
        .unwrap();
    ["matched", "failed", "lacked", "undecided"].map(|k| members(corpus, &t[k]))
}

fn set(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

/// J1, J9, J10, J15 (C106): a captured beast is found by its category —
/// `monster.beast` by its bestiary line, its colour leaf by the colour
/// table's row — and never by `currency`, though its class is the game's
/// word for it, Stackable Currency. A colour the table lacks or marks
/// unknown leaves the two colour leaves open and `monster.beast` true
/// (rule 8 of the plan), with the colour for its reason; a count lists
/// such a beast under `monster.beast`.
#[test]
fn j1_a_beast_is_found_by_its_category_and_its_colour_by_the_tables_row() {
    let s = menagerie();
    let corpus = load(&s, Some("pc"));
    let beasts = asked(&corpus, "category=\"monster.beast\"");
    assert_eq!(ids(&beasts), ["crab", "hellion", "nobody", "ursa"]);
    // chaos and ghost: no beast. Neither lacks a category — chaos has one,
    // and ghost's is open — so both failed
    assert_eq!(counts(&beasts, "0"), (4, 2, 0, 0));
    assert_eq!(
        routed(&corpus, &beasts, "0"),
        [
            set(&["crab", "hellion", "nobody", "ursa"]),
            set(&["chaos", "ghost"]),
            set(&[]),
            set(&[])
        ]
    );

    let yellow = asked(&corpus, "category=\"monster.yellowbeast\"");
    assert_eq!(ids(&yellow), ["hellion"]);
    assert_eq!(counts(&yellow, "0"), (1, 3, 0, 2));
    assert_eq!(
        routed(&corpus, &yellow, "0"),
        [
            set(&["hellion"]),
            set(&["chaos", "ghost", "ursa"]),
            set(&[]),
            set(&["crab", "nobody"])
        ]
    );
    let red = asked(&corpus, "category=\"monster.redbeast\"");
    assert_eq!(ids(&red), ["ursa"]);
    assert_eq!(counts(&red, "0"), (1, 3, 0, 2));

    // J10: a beast's class gives it no id — Stackable Currency, never
    // `currency`; ghost, whose class is open, may be anything a class gives
    let currency = asked(&corpus, "category=currency");
    assert_eq!(ids(&currency), ["chaos"]);
    assert_eq!(counts(&currency, "0"), (1, 4, 0, 1));
    assert_eq!(
        routed(&corpus, &currency, "0"),
        [
            set(&["chaos"]),
            set(&["crab", "hellion", "nobody", "ursa"]),
            set(&[]),
            set(&["ghost"])
        ]
    );
    // J9: the class is the game's word, read by the same line
    assert_eq!(
        ids(&asked(&corpus, "class=\"Stackable Currency\"")),
        ["chaos", "crab", "hellion", "nobody", "ursa"]
    );
    assert_eq!(ids(&asked(&corpus, "undecided(class)")), ["ghost"]);

    // the row shows the category a count lists the item under
    assert_eq!(
        yellow["rows"][0]["matched"][0]["shows"],
        json!([{ "value": { "name": "category", "value": "monster.yellowbeast" } }])
    );
    let shows = |a: &Value, id: &str| -> Value {
        a["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()["matched"][0]["shows"]
            .clone()
    };
    assert_eq!(
        shows(&beasts, "crab"),
        json!([{ "value": { "name": "category", "value": "monster.beast" } }])
    );

    // the one new reason, the colour: what it is of, and the hint that
    // the site may list the beast
    let open = asked(&corpus, "undecided(category=\"monster.yellowbeast\")");
    assert_eq!(ids(&open), ["crab", "nobody"]);
    for id in ["crab", "nobody"] {
        let why = &shows(&open, id)[0]["undecided"];
        assert!(
            why["unread"].as_str().unwrap().starts_with("the colour"),
            "{why}"
        );
        assert!(why["hint"].as_str().unwrap().contains("site"), "{why}");
    }
    let crab = shows(&open, "crab")[0]["undecided"]["problem"]
        .as_str()
        .unwrap()
        .to_string();
    let nobody = shows(&open, "nobody")[0]["undecided"]["problem"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(
        crab.contains("Bleached Crustacean") && crab.contains("unknown"),
        "{crab}"
    );
    assert!(nobody.contains("A Beast The Table Lacks"), "{nobody}");
    assert_ne!(crab, nobody);

    // counted (J11, J15, C105): a beast under its colour leaf, one of
    // undecided colour under `monster.beast`, ghost undecided — no id holds
    // on it and one is open — and the buckets sum to the total
    let a = counted_by(&corpus, "", &["category"]);
    let table = &a["view"]["counts"]["tables"][0];
    let by: BTreeMap<String, BTreeSet<String>> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| (bucket_name(b), members(&corpus, b)))
        .collect();
    assert_eq!(
        by,
        BTreeMap::from([
            ("currency".to_string(), set(&["chaos"])),
            ("monster.beast".to_string(), set(&["crab", "nobody"])),
            ("monster.redbeast".to_string(), set(&["ursa"])),
            ("monster.yellowbeast".to_string(), set(&["hellion"])),
            ("(undecided)".to_string(), set(&["ghost"])),
        ])
    );
    assert_eq!(a["total"]["matched"], 6);
    let undecided = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["bucket"] == "undecided")
        .unwrap();
    assert_eq!(undecided["term"], "undecided(category)");
    assert_eq!(
        undecided["tally"],
        json!([{ "unread": "the class: base not in the table", "items": 1 }])
    );
    // the same three questions as terms
    assert_eq!(ids(&asked(&corpus, "undecided(category)")), ["ghost"]);
    assert_eq!(
        ids(&asked(&corpus, "has:category")),
        ["chaos", "crab", "hellion", "nobody", "ursa"]
    );
    assert_eq!(asked(&corpus, "-has:category")["total"]["matched"], 0);

    // crossed with another key: each beast in one cell
    let request: Request = serde_json::from_value(json!({
        "query": { "text": "category=\"monster.beast\"" },
        "view": { "cross": { "keys": ["category", "frame"] } },
    }))
    .unwrap();
    let crossed = as_json(&answer(&corpus, &request).unwrap());
    let cells: BTreeMap<String, u64> = crossed["view"]["cross"]["cells"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            assert_eq!(c["of"][1]["value"], "rare");
            assert_eq!(
                members(&corpus, c).len() as u64,
                c["count"].as_u64().unwrap()
            );
            (bucket_name(&c["of"][0]), c["count"].as_u64().unwrap())
        })
        .collect();
    assert_eq!(
        cells,
        BTreeMap::from([
            ("monster.beast".to_string(), 2),
            ("monster.redbeast".to_string(), 1),
            ("monster.yellowbeast".to_string(), 1),
        ])
    );
}

// ---- under not and or -----------------------------------------------------------------------------

/// Five items in `Odds` (o1), worked by hand from the table's rows and
/// C93:
///
/// - `stone`, Tul's Breachstone with no frame: its class open among
///   Breachstones and Misc Map Items, so `map` holds (J17: every
///   candidate gives it), `map.breachstone` is open, and no candidate
///   gives any other id;
/// - `reson`, a Potent Chaotic Resonator with no frame: open among two
///   classes that both give `currency.resonator`, so that id and
///   `currency` hold and nothing is open;
/// - `crab`, a captured Bleached Crustacean, its colour unknown:
///   `monster.beast` holds, the two colour leaves are open;
/// - `ring`, an Iron Ring: `accessory.ring` and `accessory`;
/// - `quest`, a quest-frame Polaric Invitation: Quest Items, no id.
fn odds() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("o1", "Odds")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "o1",
        "Odds",
        vec![
            plain("stone", "Tul's Breachstone", "", json!({})),
            plain("reson", "Potent Chaotic Resonator", "", json!({})),
            beast("crab", "Bleached Crustacean"),
            plain("ring", "Iron Ring", "Rare", json!({})),
            plain("quest", "Polaric Invitation", "Quest", json!({})),
        ],
        20,
    );
    s
}

/// C93 over category: a not of an id that holds is false; a not of an id
/// left open is undecided and never a match; a not of an id no candidate
/// gives is a match; an or of an open id with one that holds is true.
/// Every term's four counts return by their routes what they counted.
#[test]
fn c93_a_category_under_not_and_or_is_undecided_only_where_an_id_is_open() {
    let s = odds();
    let corpus = load(&s, Some("pc"));
    let all = ["crab", "quest", "reson", "ring", "stone"];
    let without = |gone: &[&str]| -> Vec<&str> {
        all.iter()
            .copied()
            .filter(|id| !gone.contains(id))
            .collect()
    };
    let check = |text: &str, matched: &[&str], undecided: &[&str]| {
        let a = asked(&corpus, text);
        assert_eq!(ids(&a), matched, "{text}");
        assert_eq!(
            id_set(&asked(&corpus, &format!("undecided({text})"))),
            set(undecided),
            "{text}"
        );
        assert_eq!(a["total"]["undecided"]["count"], undecided.len(), "{text}");
        // every term's counts account for every item, each by its route
        for term in a["terms"].as_array().unwrap() {
            let path = term["path"].as_str().unwrap();
            let each = routed(&corpus, &a, path);
            assert_eq!(each.iter().map(BTreeSet::len).sum::<usize>(), 5, "{text}");
        }
    };

    // the breachstone whose class is open
    check("category=map", &["stone"], &[]);
    check("-category=map", &without(&["stone"]), &[]);
    check("category=\"map.breachstone\"", &[], &["stone"]);
    check(
        "-category=\"map.breachstone\"",
        &without(&["stone"]),
        &["stone"],
    );
    check("-category=currency", &without(&["reson"]), &[]);
    assert!(ids(&asked(&corpus, "-category=currency")).contains(&"stone".to_string()));
    check(
        "category=\"map.breachstone\" or category=map",
        &["stone"],
        &[],
    );
    check(
        "category=\"map.breachstone\" or category=currency",
        &["reson"],
        &["stone"],
    );
    check("category=\"map.breachstone\" category=map", &[], &["stone"]);

    // the resonator: open in class, decided in every id (J17)
    check("category=\"currency.resonator\"", &["reson"], &[]);
    check(
        "-category=\"currency.resonator\"",
        &without(&["reson"]),
        &[],
    );
    assert_eq!(ids(&asked(&corpus, "undecided(class)")), ["reson", "stone"]);

    // the beast of unknown colour
    check("-category=\"monster.beast\"", &without(&["crab"]), &[]);
    check(
        "-category=\"monster.yellowbeast\"",
        &without(&["crab"]),
        &["crab"],
    );
    check("-category=currency", &without(&["reson"]), &[]);
    assert!(ids(&asked(&corpus, "-category=currency")).contains(&"crab".to_string()));
    check(
        "category=\"monster.redbeast\" or category=\"monster.beast\"",
        &["crab"],
        &[],
    );
    check(
        "category=\"monster.redbeast\" or category=accessory",
        &["ring"],
        &["crab"],
    );

    // the four counts of one term (C93): a quest item lacks a category,
    // known absence; the others failed
    let a = asked(&corpus, "category=\"accessory.ring\"");
    assert_eq!(counts(&a, "0"), (1, 3, 1, 0));
    assert_eq!(
        routed(&corpus, &a, "0"),
        [
            set(&["ring"]),
            set(&["crab", "reson", "stone"]),
            set(&["quest"]),
            set(&[])
        ]
    );
    assert_eq!(ids(&asked(&corpus, "-has:category")), ["quest"]);
    assert_eq!(asked(&corpus, "undecided(category)")["total"]["matched"], 0);

    // counted: each under the most specific id that holds (J11, J21),
    // the quest item under none
    let by = listing(&corpus, "category");
    assert_eq!(
        by,
        BTreeMap::from([
            ("crab".to_string(), set(&["monster.beast"])),
            ("quest".to_string(), set(&["(none)"])),
            ("reson".to_string(), set(&["currency.resonator"])),
            ("ring".to_string(), set(&["accessory.ring"])),
            ("stone".to_string(), set(&["map"])),
        ])
    );
}

// ---- the selector ---------------------------------------------------------------------------------

/// The field is a closed set of the 82 ids: `=` names one, in any case;
/// `:` and `~` pick among them, every pick the matches carry printed; a
/// word that picks none is an authoring error, never a match made for
/// the author (S107).
#[test]
fn a_category_is_one_of_the_sites_ids_and_a_pick_among_them_is_printed() {
    let s = odds();
    let corpus = load(&s, Some("pc"));
    assert_eq!(
        ids(&asked(&corpus, "category=\"ACCESSORY.RING\"")),
        ["ring"]
    );
    // `:` picks every id that holds the word: `map` and the four under it
    let maps = asked(&corpus, "category:map");
    assert_eq!(ids(&maps), ["stone"]);
    assert_eq!(maps["terms"][0]["term"], "category:map");
    assert_eq!(
        maps["terms"][0]["resolved"]["values"],
        json!([{ "items": 1, "value": "map" }])
    );
    // the leaf is among the picks and open on the stone: matched all the
    // same, since `map` is picked too and holds
    assert_eq!(counts(&maps, "0"), (1, 3, 1, 0));
    let picked = asked(&corpus, "category~\"^(accessory|currency)\"");
    assert_eq!(ids(&picked), ["reson", "ring"]);
    assert_eq!(
        picked["terms"][0]["resolved"]["values"],
        json!([
            { "items": 1, "value": "accessory" },
            { "items": 1, "value": "accessory.ring" },
            { "items": 1, "value": "currency" },
            { "items": 1, "value": "currency.resonator" },
        ])
    );
    let e = ask(&corpus, "category=ring").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_value");
    assert_eq!(e["readings"][0], "category=\"accessory.ring\"");
    // an id with a dot is more than a word, and quoted as any such value
    // is: the grammar's own error offers the quoted form
    let e = ask(&corpus, "category=accessory.ring")
        .unwrap_err()
        .to_json();
    assert_eq!(e["kind"], "unexpected");
    assert_eq!(e["readings"], json!(["\"accessory.ring\""]));
    let e = ask(&corpus, "category:zzz").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_value");
    let e = ask(&corpus, "category>=3").unwrap_err().to_json();
    assert_eq!(e["kind"], "operator_mismatch");
}

// ---- show -----------------------------------------------------------------------------------------

/// `show` gives the item's category beside its class: the id or ids a
/// count lists it under, `none`, or undecided, with the reason wherever an
/// id is left open.
#[test]
fn show_gives_the_category_a_count_lists_the_item_under_and_why_an_id_is_open() {
    let mut s = odds();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("o1", "Odds"), tab("o2", "More")]),
        30,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "o2",
        "More",
        vec![
            // each of two sword classes, two leaves not nested (J16)
            plain("blade", "Energy Blade", "Rare", json!({})),
            beast("hellion", "Dune Hellion"),
            plain("lost", "A Base The Table Lacks", "Normal", json!({})),
        ],
        40,
    );
    let category = |id: &str| shown(&s, id)["category"].clone();

    // a beast: its colour leaf
    assert_eq!(
        category("hellion"),
        json!({ "bucket": "value", "listed": ["monster.yellowbeast"] })
    );
    assert_eq!(
        shown(&s, "hellion")["class"],
        json!({ "is": "Stackable Currency" })
    );
    // two leaves, in the site's order
    assert_eq!(
        category("blade"),
        json!({ "bucket": "value", "listed": ["weapon.basesword", "weapon.twosword"] })
    );
    // no id: known absence
    assert_eq!(category("quest"), json!({ "bucket": "none", "listed": [] }));
    // left open: no id holds, and every id a class gives is open by the
    // class's own reason
    let lost = category("lost");
    assert_eq!(lost["bucket"], "undecided");
    assert_eq!(lost["listed"], json!([]));
    assert_eq!(lost["open"].as_array().unwrap().len(), 1);
    assert_eq!(lost["open"][0]["part"], "class");
    assert_eq!(lost["open"][0]["of"], "not_in_table");
    assert_eq!(lost["open"][0]["ids"].as_array().unwrap().len(), 79);
    assert!(
        !lost["open"][0]["ids"]
            .as_array()
            .unwrap()
            .contains(&json!("monster.beast"))
    );
    // listed and open at once: the stone under `map`, its leaf open; the
    // beast under `monster.beast`, its colour open
    assert_eq!(
        (&category("stone")["bucket"], &category("stone")["listed"]),
        (&json!("value"), &json!(["map"]))
    );
    assert_eq!(
        category("stone")["open"][0]["ids"],
        json!(["map.breachstone"])
    );
    assert_eq!(category("stone")["open"][0]["of"], "several");
    let crab = category("crab");
    assert_eq!(crab["listed"], json!(["monster.beast"]));
    assert_eq!(
        crab["open"][0]["ids"],
        json!(["monster.yellowbeast", "monster.redbeast"])
    );
    assert_eq!(crab["open"][0]["part"], "colour");
    // the versions of the tables are on the basis (C98; C106's clause (c))
    let basis = &shown(&s, "crab")["basis"];
    assert_eq!(basis["categories"], 1);
    assert_eq!(basis["beast_colours"], 1);
    assert_eq!(asked(&load(&s, Some("pc")), "")["basis"], *basis);
}

/// A bestiary line that cannot be read leaves open what it decides and no
/// more (rule 8 of the plan): the item is a ring or a beast, so its class
/// is open among Rings and Stackable Currency, `accessory.ring` and
/// `monster.beast` are open, and an id neither reading gives is false.
/// Fixtures alone reach this: GGG gives the line as a string.
#[test]
fn a_bestiary_line_that_is_no_string_leaves_open_only_what_it_decides() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("u1", "Unread")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "u1",
        "Unread",
        vec![plain("odd", "Iron Ring", "Rare", json!({ "descrText": 7 }))],
        20,
    );
    let corpus = load(&s, Some("pc"));
    for open in [
        "class=Rings",
        "class=\"Stackable Currency\"",
        "category=\"accessory.ring\"",
        "category=accessory",
        "category=\"monster.beast\"",
        "category=\"monster.yellowbeast\"",
        "has:category",
    ] {
        let a = asked(&corpus, open);
        assert_eq!(a["total"]["matched"], 0, "{open}");
        assert_eq!(a["total"]["undecided"]["count"], 1, "{open}");
        assert_eq!(
            a["total"]["undecided_reasons"][0]["unread"], "`descrText`",
            "{open}"
        );
    }
    for no in ["class=Boots", "category=currency", "category=armour"] {
        let a = asked(&corpus, no);
        assert_eq!(
            (&a["total"]["matched"], &a["total"]["undecided"]["count"]),
            (&json!(0), &json!(0)),
            "{no}"
        );
    }
    assert_eq!(ids(&asked(&corpus, "undecided(category)")), ["odd"]);
    assert_eq!(ids(&asked(&corpus, "undecided(class)")), ["odd"]);
}

// ---- the help -------------------------------------------------------------------------------------

/// `--describe category`: the field among the fields, its 82 legal values
/// the site's ids in the site's order, every id with the site's text for
/// it and what makes it hold, and the tables' sources and versions.
#[test]
fn describe_category_lists_every_id_with_the_sites_text_and_what_makes_it_hold() {
    let whole = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let field = |d: &Value| -> Value {
        d["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == "category")
            .unwrap()
            .clone()
    };
    let expected: Vec<(String, String)> = table(EXPECTED_COPY)
        .into_iter()
        .take(82)
        .map(|row| (row["id"].clone(), row["text"].clone()))
        .collect();
    let ids: Vec<Value> = expected.iter().map(|(id, _)| json!(id)).collect();
    assert_eq!(field(&whole)["kind"], "closed set");
    assert_eq!(field(&whole)["values"], json!(ids));
    // every example the entry prints is a query this build binds
    assert!(!field(&whole)["examples"].as_array().unwrap().is_empty());
    for example in field(&whole)["examples"].as_array().unwrap() {
        acquisition_search::parse_query(example.as_str().unwrap()).unwrap();
    }
    let has = whole["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "has")
        .unwrap();
    assert!(
        has["values"]
            .as_array()
            .unwrap()
            .contains(&json!("category"))
    );
    assert!(
        whole["counts"][0]["values"]
            .as_array()
            .unwrap()
            .contains(&json!("category"))
    );

    let one = serde_json::to_value(describe(&["category".to_string()]).unwrap()).unwrap();
    assert_eq!(one["fields"].as_array().unwrap().len(), 1);
    let what = field(&one)["what"].as_str().unwrap().to_string();
    for needle in [
        "categories v1",
        "beast colours v1",
        "classes v1",
        "2026-09-12",
        "trade site",
    ] {
        assert!(what.contains(needle), "`{needle}` is not in: {what}");
    }
    let rows = one["categories"].as_array().unwrap();
    let described: Vec<(String, String)> = rows
        .iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap().to_string(),
                r["text"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(described, expected);
    let holds = |id: &str| -> String {
        rows.iter().find(|r| r["id"] == id).unwrap()["holds"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert!(
        holds("accessory.ring").contains("Rings"),
        "{}",
        holds("accessory.ring")
    );
    assert!(holds("weapon.onemace").contains("weapon.sceptre"));
    assert!(holds("jewel.cluster").contains("Cluster Jewel"));
    assert!(holds("monster.beast").contains("bestiary"));
    assert!(holds("monster.redbeast").contains("colour"));
    assert!(
        rows.iter()
            .all(|r| !r["holds"].as_str().unwrap().is_empty())
    );
    // asked for another entry, the ids are not printed
    let other = serde_json::to_value(describe(&["class".to_string()]).unwrap()).unwrap();
    assert!(other.get("categories").is_none_or(|c| c == &json!([])));
}

// ---- what a body GGG does not give leaves open, and no more ---------------------------------------

fn body(id: &str, more: Value) -> Value {
    let mut body = json!({ "id": id, "name": "", "identified": true, "ilvl": 0, "x": 0, "y": 0 });
    for (key, value) in more.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

/// Five items in `Unread` (p1), three of them bodies GGG does not give:
///
/// - `nameless`: a captured beast whose `baseType` is a number, so its
///   name is unread — its bestiary line is read;
/// - `anonymous`: a captured beast with no `baseType` at all, read whole;
/// - `hellion`: a captured Dune Hellion, read whole;
/// - `orb`: a Chaos Orb whose `descrText` is a number, so whether it is a
///   beast is unread — and a beast's class is Stackable Currency too;
/// - `tattoo`: an Honoured Tattoo of the Oak, a base the class table
///   lists under Stackable Currency.
fn unread_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("p1", "Unread")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "p1",
        "Unread",
        vec![
            body(
                "nameless",
                json!({ "typeLine": "Dune Hellion", "baseType": 7, "frameTypeId": "Rare",
                        "descrText": BESTIARY }),
            ),
            body(
                "anonymous",
                json!({ "frameTypeId": "Rare", "descrText": BESTIARY }),
            ),
            body(
                "hellion",
                json!({ "typeLine": "Dune Hellion", "baseType": "Dune Hellion",
                        "frameTypeId": "Rare", "descrText": BESTIARY }),
            ),
            body(
                "orb",
                json!({ "typeLine": "Chaos Orb", "baseType": "Chaos Orb",
                        "frameTypeId": "Currency", "descrText": 7 }),
            ),
            body(
                "tattoo",
                json!({ "typeLine": "Honoured Tattoo of the Oak",
                        "baseType": "Honoured Tattoo of the Oak", "frameTypeId": "Currency" }),
            ),
        ],
        20,
    );
    s
}

/// Two statuses for one thing (the record's shape; invariant 4 of the
/// surface, C105). A captured beast's class is read by its bestiary line
/// and by nothing else (`class.rs`), so `nameless`, whose base is unread,
/// is Stackable Currency: `class="Stackable Currency"` matches it and
/// `show` says `is`. It was `undecided(class)` too, the unread base
/// counted as leaving open a class it did not decide. One status: an
/// item whose class is established is undecided on class under no
/// spelling.
#[test]
fn two_statuses_a_beast_whose_name_is_unread_has_a_class_and_an_undecided_class() {
    let s = unread_stash();
    let corpus = load(&s, Some("pc"));
    assert_eq!(
        shown(&s, "nameless")["class"],
        json!({ "is": "Stackable Currency" })
    );
    let both = asked(&corpus, "class=\"Stackable Currency\" undecided(class)");
    assert_eq!(
        ids(&both),
        Vec::<String>::new(),
        "an item whose class matched is also undecided(class)"
    );
    assert_eq!(asked(&corpus, "undecided(class)")["total"]["matched"], 0);
    assert_eq!(asked(&corpus, "has:class")["total"]["matched"], 5);
    // and under a not: the class is established, so its not is false
    let not = asked(&corpus, "-class=\"Stackable Currency\"");
    assert_eq!(
        (
            &not["total"]["matched"],
            &not["total"]["undecided"]["count"]
        ),
        (&json!(0), &json!(0))
    );
}

/// The same fault by its second sign: a count by class put `nameless` in
/// the undecided bucket, so the bucket `Stackable Currency` counted fewer
/// than its own route, `class="Stackable Currency"`, returned. Every
/// bucket's route returns what the bucket counted, and the five items are
/// one bucket.
#[test]
fn two_statuses_a_class_buckets_route_returns_more_than_the_bucket_counted() {
    let s = unread_stash();
    let corpus = load(&s, Some("pc"));
    let counted = counted_by(&corpus, "", &["class"]);
    let by: BTreeMap<String, BTreeSet<String>> = counted["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|bucket| (bucket_name(bucket), members(&corpus, bucket)))
        .collect();
    assert_eq!(
        by,
        BTreeMap::from([(
            "Stackable Currency".to_string(),
            set(&["anonymous", "hellion", "nameless", "orb", "tattoo"])
        )])
    );
    // every term's four counts, each by its route
    let a = asked(&corpus, "class=\"Stackable Currency\"");
    assert_eq!(counts(&a, "0"), (5, 0, 0, 0));
    assert_eq!(routed(&corpus, &a, "0")[0].len(), 5);
}

/// A second maker of a reason, with the wrong hint (rules 8 and 10 of the
/// plan: unknown is said once, where the evidence is read). `nameless`
/// has no colour because its `baseType` could not be read — the deriver
/// says so, with the hint that a refresh may help. The colour leaf's
/// reason was made again here as the colour table's, which was never
/// asked, with the hint that a refresh will not help. The reason an open
/// colour leaf carries is the unread `baseType` — under a not, in `show`
/// and beneath a count's bucket alike. A beast with no base at all was
/// read whole: there the table could not be asked, and the reason is the
/// colour's own.
#[test]
fn a_second_maker_an_unread_name_is_blamed_on_the_colour_table() {
    let s = unread_stash();
    let corpus = load(&s, Some("pc"));
    let why_of = |text: &str, id: &str| -> Value {
        asked(&corpus, text)["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap_or_else(|| panic!("`{id}` is not open under {text}"))["matched"][0]["shows"][0]
            ["undecided"]
            .clone()
    };
    let open = "undecided(category=\"monster.yellowbeast\")";
    let why = why_of(open, "nameless");
    assert_eq!(why["unread"], "`baseType`", "{why}");
    assert!(
        why["hint"]
            .as_str()
            .unwrap()
            .starts_with("a refresh may help"),
        "{why}"
    );
    // the not of an open leaf is open by the same reason, never a match:
    // the yellow hellion and the tattoo are not red; the two beasts with
    // no colour and the orb, which may be a beast of none, are open
    let not = asked(&corpus, "-category=\"monster.redbeast\"");
    assert_eq!(ids(&not), ["hellion", "tattoo"]);
    assert_eq!(not["total"]["undecided"]["count"], 3);
    assert_eq!(
        why_of("undecided(-category=\"monster.redbeast\")", "nameless")["unread"],
        "`baseType`"
    );
    // it is a beast all the same, listed under `monster.beast` (J15)
    assert!(ids(&asked(&corpus, "category=\"monster.beast\"")).contains(&"nameless".to_string()));
    let shown_nameless = shown(&s, "nameless");
    assert_eq!(
        shown_nameless["category"]["listed"],
        json!(["monster.beast"])
    );
    assert_eq!(
        open_by(&shown_nameless, "monster.yellowbeast"),
        [("field".to_string(), "baseType".to_string())]
    );
    // no base at all: nothing was unread, and the colour is what is open
    let why = why_of(open, "anonymous");
    assert!(
        why["unread"].as_str().unwrap().starts_with("the colour"),
        "{why}"
    );
    assert_eq!(
        open_by(&shown(&s, "anonymous"), "monster.yellowbeast"),
        [("colour".to_string(), String::new())]
    );
    // the hellion's colour is the table's row, and nothing of it is open
    assert_eq!(
        id_set(&asked(&corpus, open)),
        set(&["anonymous", "nameless", "orb"])
    );
}

/// Unknown said at the wrong grain (rule 8 of the plan; C93; J17's rule,
/// said of the class itself — the owner, 2026-10-07). Whether `orb` is a
/// beast is unread, and it is Stackable Currency either way: its class
/// was left open among one candidate. It is that class, its not fails,
/// and what stays open is what the two readings differ on — `currency`
/// and the beast ids — by the unread `descrText`.
#[test]
fn wrong_grain_a_class_both_readings_give_is_left_open_by_the_bestiary_line() {
    let s = unread_stash();
    let corpus = load(&s, Some("pc"));
    // what stays open, rightly: the orb is `currency` or a beast
    let open = asked(&corpus, "undecided(category=currency)");
    assert_eq!(ids(&open), ["orb"]);
    assert_eq!(
        open["rows"][0]["matched"][0]["shows"][0]["undecided"]["unread"],
        "`descrText`"
    );
    assert_eq!(
        ids(&asked(&corpus, "undecided(category=\"monster.beast\")")),
        ["orb"]
    );
    assert_eq!(ids(&asked(&corpus, "undecided(category)")), ["orb"]);
    // an id neither reading gives is false, not open
    let armour = asked(&corpus, "category=armour");
    assert_eq!(counts(&armour, "0"), (0, 5, 0, 0));
    let a = asked(&corpus, "class=\"Stackable Currency\"");
    assert!(
        ids(&a).contains(&"orb".to_string()),
        "the orb is Stackable Currency under every reading: {}",
        a["terms"][0]
    );
    assert_eq!(
        shown(&s, "orb")["class"],
        json!({ "is": "Stackable Currency" })
    );
    assert!(!ids(&asked(&corpus, "undecided(class)")).contains(&"orb".to_string()));
}

/// A claim the code does not make (the record's shape; J10). The help
/// said of `currency` "an item whose class is Currency or Gold or
/// Stackable Currency", and of a class that a captured beast is Stackable
/// Currency; the build, rightly, never lists a beast under `currency`.
/// The row says a captured beast is out. Two wordings beside it: a
/// grouping's row said "one of its reviewed members, and no other", which
/// an item of two members contradicts; and `currency`'s read as if a
/// Fossil left it, whose leaf is under it by the dot.
#[test]
fn a_claim_the_help_says_currency_holds_every_stackable_currency_item() {
    let s = unread_stash();
    let corpus = load(&s, Some("pc"));
    // the build's answer, as ruled
    assert_eq!(
        ids(&asked(
            &corpus,
            "class=\"Stackable Currency\" -category=currency"
        )),
        ["anonymous", "hellion", "nameless"]
    );
    let d = serde_json::to_value(describe(&["category".to_string()]).unwrap()).unwrap();
    let holds = |id: &str| -> String {
        d["categories"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()["holds"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let currency = holds("currency");
    assert!(currency.contains("Stackable Currency"), "{currency}");
    assert!(
        currency.to_lowercase().contains("beast"),
        "the help gives a beast's class as what makes `currency` hold, and names no exception: {currency}"
    );
    // a Fossil is `currency` still: its leaf is named among those under it
    // and never as an exception to it
    assert!(!currency.contains("Fossil"), "{currency}");
    assert!(currency.contains("`currency.fossil`"), "{currency}");
    // where a rule does take a base out of an id, the row says so
    assert!(holds("jewel.base").contains("`jewel.cluster` instead"));
    assert!(holds("gem.supportgem").contains("`gem.supportgemplus` instead"));
    // a grouping holds an item of two of its members (J16's Energy Blade)
    for grouping in ["weapon.one", "weapon.onemelee", "weapon.onesword"] {
        assert!(!holds(grouping).contains("no other"), "{}", holds(grouping));
        assert!(holds(grouping).contains("any of its reviewed members"));
    }
}

/// J23 (C106; round m, m01 and m03): a Stackable Currency base whose name
/// holds the word Tattoo is `currency.tattoo` — the site lists an
/// Honoured Tattoo there — and not only a name beginning `Tattoo of `,
/// which left 40 of the class table's 100 tattoo bases under `currency`.
#[test]
fn j23_a_tattoo_is_known_by_the_word_wherever_it_stands_in_its_name() {
    let s = unread_stash();
    let corpus = load(&s, Some("pc"));
    assert_eq!(
        ids(&asked(&corpus, "category=\"currency.tattoo\"")),
        ["tattoo"]
    );
    assert_eq!(
        listing(&corpus, "category")["tattoo"],
        set(&["currency.tattoo"])
    );
    let d = serde_json::to_value(describe(&["category".to_string()]).unwrap()).unwrap();
    let row = d["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "currency.tattoo")
        .unwrap();
    assert_eq!(
        row["holds"],
        "an item whose class is Stackable Currency and whose base holds `Tattoo`"
    );
}
