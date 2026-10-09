//! The base defence percentile (C101; `search/LEDGER.md`, "Holes ruled",
//! V5, J5, J6, J8 and J33–J35; the build plan, 9d-A), through the crate's
//! boundary: `pseudo.defence_pct`, the trade site's Base Percentile.
//!
//! The first test reads the track's expected answers where they lie
//! (`search/pseudo-stats/data/percentile-expected.csv`, written before
//! anything was built, from the site's captures and the owner's rulings)
//! and holds the build to every row. The others are fixtures worked by
//! hand before running: the table's four worked cases asked every way a
//! computed value is asked, an undecided field under `not` and `or`
//! (C93), what does not open the field (rule 8 of the plan), and the
//! refusal of `has:` with a slot word.

mod common;

use std::collections::{BTreeMap, BTreeSet};

use acquisition_search::{Corpus, Request, answer, describe};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

/// The expected answers, one row a case: the tests' side, never read by
/// the build.
const EXPECTED: &str = include_str!("../../../search/pseudo-stats/data/percentile-expected.csv");

const FIELD: &str = "pseudo.defence_pct";

// ---- reading the table ------------------------------------------------------------------------

/// What the field answers of a case.
#[derive(Debug, Clone, PartialEq)]
enum Cell {
    Lacked,
    Value(i64),
    /// With the cause the table gives: what could not be established.
    Undecided(String),
}

/// One case of the expected answers: an item as the row gives it.
struct Case {
    /// The item's id in the fixture store: the case's place in the table.
    id: String,
    /// The table's name for the case.
    name: String,
    /// `captured`, `ruled` or `worked`.
    kind: String,
    realm: String,
    base: String,
    rarity: String,
    unique: String,
    marks: String,
    properties: String,
    lines: String,
    cell: Cell,
}

fn cases() -> Vec<Case> {
    let mut seen = BTreeSet::new();
    table(EXPECTED)
        .into_iter()
        .enumerate()
        .map(|(at, row)| {
            assert!(seen.insert(row["case"].clone()), "{} twice", row["case"]);
            let cell =
                match row["status"].as_str() {
                    "lacked" => Cell::Lacked,
                    "value" => Cell::Value(row["value"].parse().unwrap_or_else(|_| {
                        panic!("{}: a value the test does not read", row["case"])
                    })),
                    "undecided" => Cell::Undecided(row["cause_or_readings"].clone()),
                    other => panic!("{}: a status the test does not read: {other}", row["case"]),
                };
            Case {
                id: format!("c{at:04}"),
                name: row["case"].clone(),
                kind: row["kind"].clone(),
                realm: row["realm"].clone(),
                base: row["base"].clone(),
                rarity: row["rarity"].clone(),
                unique: row["unique"].clone(),
                marks: row["marks"].clone(),
                properties: row["properties"].clone(),
                lines: row["lines"].clone(),
                cell,
            }
        })
        .collect()
}

/// A case's properties as the body's array, each as the row displays it:
/// `Name value; …`, the name as GGG wrote it, markup and all. `(none)` is
/// a body with no `properties`; `(none the field reads)` one whose array
/// holds none the field reads; `(no array: …)` what the row gives in the
/// array's place; `(name unread: …)` an element whose name is what the row
/// gives; `Name (unread: …)` a value that is no string.
fn properties_of(case: &Case) -> Option<Value> {
    let text = case.properties.as_str();
    if text == "(none)" {
        return None;
    }
    if text == "(none the field reads)" {
        return Some(json!([]));
    }
    if let Some(given) = text
        .strip_prefix("(no array: ")
        .and_then(|t| t.strip_suffix(')'))
    {
        return Some(serde_json::from_str(given).unwrap());
    }
    let property = |name: Value, value: Value| json!({ "name": name, "values": [[value, 0]], "displayMode": 0 });
    let mut out = Vec::new();
    for piece in text.split("; ") {
        if let Some(given) = piece
            .strip_prefix("(name unread: ")
            .and_then(|t| t.strip_suffix(')'))
        {
            out.push(property(serde_json::from_str(given).unwrap(), json!("")));
            continue;
        }
        assert!(!piece.starts_with('('), "{}: {piece:?}", case.name);
        if let Some((name, given)) = piece.split_once(" (unread: ") {
            let given = given.strip_suffix(')').unwrap();
            out.push(property(json!(name), serde_json::from_str(given).unwrap()));
            continue;
        }
        let (name, value) = piece
            .rsplit_once(' ')
            .unwrap_or_else(|| panic!("{}: no property: {piece:?}", case.name));
        out.push(property(json!(name), json!(value)));
    }
    Some(json!(out))
}

/// A case's marks as the body's keys: `-` is none; a name is a yes;
/// `name=<json>` a mark that is no yes or no, as the row gives it. `(not
/// kept)` is a captured item whose marks nothing kept: each of the three
/// is entered as no yes or no, which is what is known of it — the field
/// asks a mark of an implicit twin alone, and the row's answer must hold
/// whatever the marks were.
fn marks_of(case: &Case) -> Vec<(String, Value)> {
    match case.marks.as_str() {
        "-" => Vec::new(),
        "(not kept)" => ["searing", "tangled", "synthesised"]
            .map(|mark| (mark.to_string(), json!("(not kept)")))
            .to_vec(),
        marks => marks
            .split(';')
            .map(|mark| match mark.split_once('=') {
                None => (mark.to_string(), json!(true)),
                Some((mark, given)) => (mark.to_string(), serde_json::from_str(given).unwrap()),
            })
            .collect(),
    }
}

/// A case's lines as the arrays of its body: `array[flags]: text | …`,
/// each line entered as the row gives it — its array, its flags, a mod of
/// several rows (` // `) as one entry with its row breaks, never taken
/// apart. An array the row gives as no array, and a line it gives as no
/// text, are entered as what the row gives. The lines the table left out,
/// `(+N lines of no defence stat)`, are not the row's to give and are not
/// entered; the count of them is returned.
fn lines_of(case: &Case) -> (BTreeMap<String, Value>, usize) {
    let mut arrays: BTreeMap<String, Value> = BTreeMap::new();
    let mut left_out = 0;
    for piece in case.lines.split(" | ").filter(|p| !p.is_empty()) {
        if let Some(count) = piece
            .strip_prefix("(+")
            .and_then(|p| p.strip_suffix(" lines of no defence stat)"))
        {
            left_out += count.parse::<usize>().unwrap();
            continue;
        }
        let (head, text) = piece
            .split_once(": ")
            .unwrap_or_else(|| panic!("{}: no line: {piece:?}", case.name));
        let (array, flags) = match head.split_once('[') {
            Some((array, flags)) => (array, flags.strip_suffix(']').unwrap()),
            None => (head, ""),
        };
        assert!(
            array.ends_with("Mods") && array.chars().all(|c| c.is_ascii_alphabetic()),
            "{}: no array: {piece:?}",
            case.name
        );
        if let Some(given) = text
            .strip_prefix("(no array: ")
            .and_then(|t| t.strip_suffix(')'))
        {
            assert!(flags.is_empty() && !arrays.contains_key(array));
            arrays.insert(array.to_string(), serde_json::from_str(given).unwrap());
            continue;
        }
        let element = if let Some(given) = text
            .strip_prefix("(no text: ")
            .and_then(|t| t.strip_suffix(')'))
        {
            assert!(flags.is_empty());
            serde_json::from_str(given).unwrap()
        } else {
            assert!(!text.starts_with("(no "), "{}: {piece:?}", case.name);
            let text = text.replace(" // ", "\n");
            if flags.is_empty() {
                json!(text)
            } else {
                let mut object = serde_json::Map::new();
                for flag in flags.split(';') {
                    match flag.split_once('=') {
                        None => object.insert(flag.to_string(), json!(true)),
                        // a flag that is no yes or no, as the script's `repr` wrote it
                        Some((flag, value)) => {
                            let value = value
                                .strip_suffix(" (unread)")
                                .and_then(|v| v.strip_prefix('\''))
                                .and_then(|v| v.strip_suffix('\''))
                                .unwrap_or_else(|| panic!("{}: {piece:?}", case.name));
                            object.insert(flag.to_string(), json!(value))
                        }
                    };
                }
                json!({ "description": text, "flags": object })
            }
        };
        arrays
            .entry(array.to_string())
            .or_insert_with(|| json!([]))
            .as_array_mut()
            .unwrap_or_else(|| panic!("{}: {array} is no array and holds a line", case.name))
            .push(element);
    }
    (arrays, left_out)
}

/// An item of a case's facts, as GGG would give it: the base, the rarity
/// — and the frame, which GGG words as the rarity on an item that has one
/// — the unique's name, the marks, the properties and the lines.
fn body_of(case: &Case, at: usize) -> Value {
    let mut body = json!({ "id": case.id, "name": case.unique, "identified": true, "ilvl": 0,
                           "x": at % 24, "y": at / 24 });
    if !case.base.is_empty() {
        body["typeLine"] = json!(case.base);
        body["baseType"] = json!(case.base);
    }
    if !case.rarity.is_empty() {
        body["rarity"] = json!(case.rarity);
        body["frameTypeId"] = json!(case.rarity);
    }
    for (mark, value) in marks_of(case) {
        body[mark] = value;
    }
    if let Some(properties) = properties_of(case) {
        body["properties"] = properties;
    }
    for (array, lines) in lines_of(case).0 {
        body[array] = lines;
    }
    body
}

// ---- asking -----------------------------------------------------------------------------------

/// More rows than any fixture here holds.
const EVERY: usize = 10_000;

fn rows_of(corpus: &Corpus, text: &str) -> Value {
    viewed(corpus, text, json!({ "rows": { "limit": EVERY } }))
}

fn viewed(corpus: &Corpus, text: &str, view: Value) -> Value {
    let request: Request =
        serde_json::from_value(json!({ "query": { "text": text }, "view": view })).unwrap();
    match answer(corpus, &request) {
        Ok(a) => as_json(&a),
        Err(e) => panic!("`{text}` {view}: {e}"),
    }
}

fn id_set(a: &Value) -> BTreeSet<String> {
    ids(a).into_iter().collect()
}

/// Each item's sort scalar under the field: its value, or its status.
fn sorted(corpus: &Corpus) -> BTreeMap<String, Value> {
    viewed(
        corpus,
        "",
        json!({ "rows": { "sort": FIELD, "limit": EVERY } }),
    )["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap().to_string(), scalar(&r["sort"])))
        .collect()
}

/// What an undecided cause names, as the kind a reason and a count's
/// tally carry (`eval::reason`): what could not be established, never
/// the table's wording of it nor the build's.
fn kind_of(cause: &str) -> &'static str {
    let (head, _) = cause.split_once(": ").unwrap_or((cause, ""));
    match head {
        "realm" => "the percentile: no table for the realm",
        "property unread" => "`properties`",
        "base" if cause.contains("the range table lacks") => {
            "the percentile: base not in the table"
        }
        "base" if cause.contains("the range table holds") => {
            "the percentile: base held more than once"
        }
        "base" if cause.contains("has every defence it ranges displayed") => {
            "the percentile: no row of the base fits what is displayed"
        }
        "an implicit twin on an item of neither mark" => {
            "the percentile: an implicit twin on an item of neither mark"
        }
        "an implicit twin on an item both eldritch and synthesised" => {
            "the percentile: an implicit twin on an item of both marks"
        }
        // the mark is the deriver's to say unread, under its own key
        "an implicit twin on an item whose mark is unread" => "`searing`",
        "a twin text in `scourgeMods`" => "the percentile: a twin text in an array no ruling names",
        "a defence line in `crucibleMods`" => "the percentile: a defence line of a crucible node",
        "several values" => "the percentile: several rolls of different values",
        "range" => "the percentile: the base's minimum is its maximum",
        "no roll" => "the percentile: no roll gives the display",
        "`explicitMods` is no array" | "a line of `explicitMods` is no text" => "explicit lines",
        other => panic!("a cause the test does not read: {other}"),
    }
}

/// The fixture store of the table: every case an item, through the
/// store's own ingest.
fn table_store(cases: &[Case]) -> Store {
    let mut s = store();
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
    s
}

/// C101, V5, J5, J6, J8, J33–J35: every row of the expected answers — a
/// case of the facts the row gives, put into a store through its ingest —
/// is answered as the row says: 4849 captured, 45 ruled, 4 worked. The
/// field is asked of each case: lacked, by `has:` and its not; or its
/// value, by a comparison, by the sort and by what the row shows; or
/// undecided, by what could not be established. The plan's closing row is
/// among them: the 896 captured cells where the site showed a value. The
/// table's `site` and `agrees` columns are evidence and assert nothing.
#[test]
fn c101_every_row_of_the_percentile_expected_answers_is_answered_as_the_row_says() {
    let cases = cases();
    let of_kind = |kind: &str| cases.iter().filter(|c| c.kind == kind).count();
    assert_eq!(
        (of_kind("captured"), of_kind("ruled"), of_kind("worked")),
        (4849, 45, 4)
    );
    assert_eq!(cases.len(), 4898);
    assert_eq!(
        cases
            .iter()
            .filter(|c| c.kind == "captured" && matches!(c.cell, Cell::Value(_)))
            .count(),
        896,
        "the captured cells where the site showed a value"
    );
    // the input holds what the test is of: a mod of several rows, a line
    // under a flag, each array a ruling names, an array, a line, a flag, a
    // mark and a property that could not be read, and marks not kept
    for held in [
        " // ",
        "[crafted]: ",
        "[fractured]: ",
        "implicitMods: ",
        "enchantMods: ",
        "crucibleMods: ",
        "scourgeMods: ",
        "(no array: ",
        "(no text: ",
        "(unread)]: ",
    ] {
        assert!(cases.iter().any(|c| c.lines.contains(held)), "{held}");
    }
    for held in ["searing=\"yes\"", "(not kept)", "synthesised;tangled"] {
        assert!(cases.iter().any(|c| c.marks == held), "{held}");
    }
    for held in [
        "(no array: ",
        "(name unread: ",
        "Armour many",
        "Quality some",
    ] {
        assert!(cases.iter().any(|c| c.properties.contains(held)), "{held}");
    }
    assert_eq!(
        cases.iter().map(|c| lines_of(c).1).sum::<usize>(),
        18133,
        "the lines the table left out as no defence stat's"
    );

    let s = table_store(&cases);
    let mut faults: Vec<String> = Vec::new();
    let mut asked = (0, 0, 0);
    for realm in ["pc", "poe2"] {
        let corpus = load(&s, Some(realm));
        let of_realm: Vec<&Case> = cases.iter().filter(|c| c.realm == realm).collect();
        let all: BTreeSet<String> = of_realm.iter().map(|c| c.id.clone()).collect();
        assert_eq!(id_set(&rows_of(&corpus, "")), all);
        let has = id_set(&rows_of(&corpus, &format!("has:{FIELD}")));
        let lacks = id_set(&rows_of(&corpus, &format!("-has:{FIELD}")));
        let open = id_set(&rows_of(&corpus, &format!("undecided({FIELD})")));
        // one status each: the three partition the realm's items
        assert_eq!(has.len() + lacks.len() + open.len(), all.len(), "{realm}");
        assert_eq!(
            has.iter()
                .chain(&lacks)
                .chain(&open)
                .cloned()
                .collect::<BTreeSet<String>>(),
            all,
            "{realm}"
        );
        // by comparison: each value the table holds, asked by `=`, finds
        // the cases of that value and no other; and by the row: what a
        // row shows first of a comparison on the field is its value
        let mut shown: BTreeMap<String, Value> = BTreeMap::new();
        let mut by_value: BTreeMap<i64, BTreeSet<String>> = BTreeMap::new();
        for case in &of_realm {
            if let Cell::Value(value) = case.cell {
                by_value.entry(value).or_default().insert(case.id.clone());
            }
        }
        for (value, expected) in &by_value {
            let rows = rows_of(&corpus, &format!("{FIELD}={value}"));
            for row in rows["rows"].as_array().unwrap() {
                shown.insert(
                    row["id"].as_str().unwrap().to_string(),
                    row["matched"][0]["shows"][0].clone(),
                );
            }
            let found = id_set(&rows);
            if &found != expected {
                faults.push(format!(
                    "{FIELD}={value} finds {} cases where the table holds {}: the build alone {:?}, the table alone {:?}",
                    found.len(),
                    expected.len(),
                    found.difference(expected).take(5).collect::<Vec<_>>(),
                    expected.difference(&found).take(5).collect::<Vec<_>>(),
                ));
            }
        }
        let by = sorted(&corpus);
        for case in &of_realm {
            let status = match (
                has.contains(&case.id),
                lacks.contains(&case.id),
                open.contains(&case.id),
            ) {
                (true, false, false) => "value",
                (false, true, false) => "lacked",
                (false, false, true) => "undecided",
                other => panic!("{}: {other:?}", case.name),
            };
            let scalar = &by[&case.id];
            let said = |answers: String| {
                format!(
                    "{} ({}): the table says {:?}, the build {answers}",
                    case.name, case.id, case.cell
                )
            };
            match &case.cell {
                Cell::Lacked => {
                    asked.0 += 1;
                    if status != "lacked" || scalar["status"] != "no satisfying occurrence" {
                        faults.push(said(format!("{status} {scalar}")));
                    }
                }
                Cell::Value(value) => {
                    asked.1 += 1;
                    if status != "value"
                        || *scalar != json!({ "value": value })
                        || shown.get(&case.id)
                            != Some(&json!({ "value": { "name": FIELD, "value": value } }))
                    {
                        faults.push(said(format!(
                            "{status} {scalar}, the row showing {:?}",
                            shown.get(&case.id)
                        )));
                    }
                }
                Cell::Undecided(cause) => {
                    asked.2 += 1;
                    let a = as_json(&ask(&corpus, &format!("id:{} has:{FIELD}", case.id)).unwrap());
                    let kinds: BTreeSet<&str> = a["total"]["undecided_reasons"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(|r| r["unread"].as_str().unwrap())
                        .collect();
                    if status != "undecided"
                        || *scalar != json!({ "status": "incomplete" })
                        || kinds != BTreeSet::from([kind_of(cause)])
                    {
                        faults.push(said(format!("{status} {scalar}, by {kinds:?}")));
                    }
                }
            }
        }
    }
    // every case asked, by the table's own tally of statuses
    assert_eq!(asked, (3956, 923, 19));
    assert!(
        faults.is_empty(),
        "{} of {} cases are not answered as the table says:\n{}",
        faults.len(),
        cases.len(),
        faults.join("\n")
    );
}

// ---- the worked cases -------------------------------------------------------------------------

fn asked(s: &Store, text: &str) -> Value {
    as_json(&ask(&load(s, Some("pc")), text).unwrap())
}

fn refused(s: &Store, text: &str) -> Value {
    ask(&load(s, Some("pc")), text).unwrap_err().to_json()
}

fn counts(term: &Value) -> [u64; 4] {
    ["matched", "failed", "lacked", "undecided"].map(|k| term[k]["count"].as_u64().unwrap())
}

/// The items a count's route returns, which are as many as it counted
/// (invariant 4 of the surface).
fn follow(s: &Store, count: &Value) -> Vec<String> {
    let request = serde_json::from_value(count["request"].clone()).unwrap();
    let routed = as_json(&answer(&load(s, Some("pc")), &request).unwrap());
    assert_eq!(routed["total"]["matched"], count["count"]);
    ids(&routed)
}

fn property(name: &str, value: &str) -> Value {
    json!({ "name": name, "values": [[value, 0]], "displayMode": 0 })
}

/// An item of a base, a rarity, the properties it displays and whatever
/// else the test adds.
fn armour(id: &str, base: &str, rarity: &str, properties: &[(&str, &str)], more: Value) -> Value {
    let mut body = item(id, "", base, rarity, more);
    body["properties"] = properties
        .iter()
        .map(|(name, value)| property(name, value))
        .collect();
    body
}

fn one_tab(items: Vec<Value>) -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("p1", "Armour")]), 10);
    fetch_tab(&mut s, "pc", "Standard", "p1", "Armour", items, 20);
    s
}

/// The table's four worked cases, their arithmetic the table's `by_hand`
/// column, and a ring:
///
/// - `one` (`w-one-type`): a Cone Helmet, Armour 44. The base ranges
///   Armour 38–50; no line, no quality, so the roll is the display.
///   100 × (44 − 38) / 12 = 50.
/// - `two` (`w-two-types`): a Scale Doublet, Armour 100 and Evasion
///   Rating 103, each ranged 97–121. 100 × 3 / 24 = 12.5 and 100 × 6 / 24
///   = 25; their average 18.75, rounded half up, 19.
/// - `flat` (`w-flat-increase-quality`): a Copper Plate, ranged 176–221,
///   quality 20, `+50 to Armour`, `40% increased Armour`, Armour 420.
///   Display = (roll + 50) × 1.40 × 1.20; roll 200 gives 420 exactly, 199
///   gives 418.32 and 201 gives 421.68. 100 × 24 / 45 = 53.33, 53.
/// - `half` (`w-exact-half`): a Cone Helmet, quality 10, Armour 49. Roll
///   44 displays 48.4, roll 45 displays 49.5 and roll 46 displays 50.6:
///   no roll shows 49 by the stated rule, and J5 admits 45.
///   100 × 7 / 12 = 58.33, 58.
/// - `ring`: a Ruby Ring, which displays no defence and lacks the field.
fn worked() -> Store {
    one_tab(vec![
        armour(
            "one",
            "Cone Helmet",
            "Normal",
            &[("Armour", "44")],
            json!({}),
        ),
        armour(
            "two",
            "Scale Doublet",
            "Normal",
            &[("Armour", "100"), ("Evasion Rating", "103")],
            json!({}),
        ),
        armour(
            "flat",
            "Copper Plate",
            "Rare",
            &[("Quality", "+20%"), ("Armour", "420")],
            json!({ "explicitMods": ["+50 to Armour", "40% increased Armour"] }),
        ),
        armour(
            "half",
            "Cone Helmet",
            "Normal",
            &[("Quality", "+10%"), ("Armour", "49")],
            json!({}),
        ),
        item("ring", "", "Ruby Ring", "Rare", json!({})),
    ])
}

/// C101: the four worked cases, asked by comparison, by `has:`, sorted,
/// and summed in a count. 50 + 19 + 53 + 58 = 180.
#[test]
fn c101_the_worked_cases_by_comparison_by_has_sorted_and_summed() {
    let s = worked();
    let found = |text: &str| ids(&asked(&s, text));

    // by comparison: the value is a whole number
    assert_eq!(found("pseudo.defence_pct=50"), ["one"]);
    assert_eq!(found("pseudo.defence_pct=19"), ["two"]);
    assert_eq!(found("pseudo.defence_pct=53"), ["flat"]);
    assert_eq!(found("pseudo.defence_pct=58"), ["half"]);
    assert_eq!(found("pseudo.defence_pct=18.75"), [] as [&str; 0]);
    let a = asked(&s, "pseudo.defence_pct>=53");
    assert_eq!(ids(&a), ["flat", "half"]);
    // two matched, two failed, the ring lacks it, none undecided
    assert_eq!(counts(&a["terms"][0]), [2, 2, 1, 0]);
    assert_eq!(follow(&s, &a["terms"][0]["failed"]), ["one", "two"]);
    assert_eq!(follow(&s, &a["terms"][0]["lacked"]), ["ring"]);
    assert_eq!(a["query"]["text"], "pseudo.defence_pct>=53");
    assert_eq!(found("pseudo.defence_pct=19..50"), ["one", "two"]);

    // by `has:`
    let a = asked(&s, "has:pseudo.defence_pct");
    assert_eq!(ids(&a), ["flat", "half", "one", "two"]);
    assert_eq!(counts(&a["terms"][0]), [4, 0, 1, 0]);
    assert_eq!(found("-has:pseudo.defence_pct"), ["ring"]);
    assert_eq!(found("undecided(pseudo.defence_pct)"), [] as [&str; 0]);

    // sorted, the ring last either way
    let order = |desc: bool| -> Vec<(String, Value)> {
        viewed(
            &load(&s, Some("pc")),
            "",
            json!({ "rows": { "sort": FIELD, "desc": desc } }),
        )["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["id"].as_str().unwrap().to_string(), scalar(&r["sort"])))
            .collect()
    };
    let value = |id: &str, value: i64| (id.to_string(), json!({ "value": value }));
    let last = (
        "ring".to_string(),
        json!({ "status": "no satisfying occurrence" }),
    );
    assert_eq!(
        order(false),
        [
            value("two", 19),
            value("one", 50),
            value("flat", 53),
            value("half", 58),
            last.clone()
        ]
    );
    assert_eq!(
        order(true),
        [
            value("half", 58),
            value("flat", 53),
            value("one", 50),
            value("two", 19),
            last
        ]
    );

    // summed in a count
    assert_eq!(
        viewed(
            &load(&s, Some("pc")),
            "",
            json!({ "counts": { "keys": ["tab"], "sum": FIELD } }),
        )["view"]["counts"]["sum"],
        json!({ "name": FIELD, "value": 180, "lacking": 1 })
    );

    // what a row shows of it: the value, then what was read
    let a = asked(&s, "id:flat pseudo.defence_pct>=0");
    let shows = a["rows"][0]["matched"][0]["shows"].as_array().unwrap();
    assert_eq!(shows[0], json!({ "value": { "name": FIELD, "value": 53 } }));
    let read: Vec<&str> = shows[1..]
        .iter()
        .map(|e| {
            e["shown"]["text"]
                .as_str()
                .or(e["line"]["text"].as_str())
                .unwrap_or_default()
        })
        .collect();
    for text in [
        "Quality: +20%",
        "Armour: 420",
        "+50 to Armour",
        "40% increased Armour",
    ] {
        assert!(read.contains(&text), "{text}: {shows:?}");
    }
}

// ---- undecided, composed ----------------------------------------------------------------------

/// C93, J33, over one of the table's cases
/// (`r-several-rolls-different-values`): `hat`, a magic Iron Hat, Armour
/// 5 under `50% reduced Armour`, whose display rolls of different values
/// give, so the field is undecided on it. Beside it `helm`, the worked
/// Cone Helmet at 50, and `ring`, which lacks the field. Worked by hand:
/// a not of a comparison on the field is undecided on `hat` and never a
/// match; an or of it with a term that holds is true; an or with one that
/// fails stays undecided; an and with one that fails is false.
#[test]
fn c93_an_undecided_percentile_under_not_and_or() {
    let s = one_tab(vec![
        armour(
            "hat",
            "Iron Hat",
            "Magic",
            &[("Armour", "5")],
            json!({ "explicitMods": ["50% reduced Armour"] }),
        ),
        armour(
            "helm",
            "Cone Helmet",
            "Normal",
            &[("Armour", "44")],
            json!({}),
        ),
        item("ring", "", "Ruby Ring", "Rare", json!({})),
    ]);
    let found = |text: &str| ids(&asked(&s, text));

    let a = asked(&s, "pseudo.defence_pct>=60");
    assert_eq!(ids(&a), [] as [&str; 0]);
    // helm fails at 50, the ring lacks it, the hat is undecided
    assert_eq!(counts(&a["terms"][0]), [0, 1, 1, 1]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    assert_eq!(follow(&s, &a["terms"][0]["undecided"]), ["hat"]);
    let why = a["total"]["undecided_reasons"].as_array().unwrap();
    assert_eq!(why.len(), 1);
    assert_eq!(why[0]["example"]["id"], "hat");
    assert_eq!(
        why[0]["unread"],
        "the percentile: several rolls of different values"
    );
    assert_eq!(found("undecided(pseudo.defence_pct)"), ["hat"]);

    // the not of it: undecided, never a match
    let a = asked(&s, "-pseudo.defence_pct>=60");
    assert_eq!(ids(&a), ["helm", "ring"]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    let a = asked(&s, "-pseudo.defence_pct>=0");
    assert_eq!(ids(&a), ["ring"]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    // `has:` and its not leave it open too
    assert_eq!(found("has:pseudo.defence_pct"), ["helm"]);
    assert_eq!(found("-has:pseudo.defence_pct"), ["ring"]);

    // an or of it with a term that holds is true
    let a = asked(&s, "pseudo.defence_pct>=60 or rarity=magic");
    assert_eq!(ids(&a), ["hat"]);
    assert_eq!(a["total"]["undecided"]["count"], 0);
    // with one that fails on it, undecided still
    let a = asked(&s, "pseudo.defence_pct>=60 or rarity=rare");
    assert_eq!(ids(&a), ["ring"]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    // an and with one that fails is false; with one that holds, undecided
    let a = asked(&s, "pseudo.defence_pct>=60 rarity=rare");
    assert_eq!(a["total"]["matched"], 0);
    assert_eq!(a["total"]["undecided"]["count"], 0);
    let a = asked(&s, "pseudo.defence_pct>=60 rarity=magic");
    assert_eq!(a["total"]["matched"], 0);
    assert_eq!(a["total"]["undecided"]["count"], 1);

    // sorted and summed, the open one is no number
    let corpus = load(&s, Some("pc"));
    let by = sorted(&corpus);
    assert_eq!(by["helm"], json!({ "value": 50 }));
    assert_eq!(by["hat"], json!({ "status": "incomplete" }));
    assert_eq!(by["ring"], json!({ "status": "no satisfying occurrence" }));
    assert_eq!(
        viewed(
            &corpus,
            "",
            json!({ "counts": { "keys": ["tab"], "sum": FIELD } }),
        )["view"]["counts"]["sum"],
        json!({ "name": FIELD, "value": 50, "lacking": 1, "incomplete": true, "unread": 1 })
    );
}

// ---- what does not open it --------------------------------------------------------------------

/// Rule 8 of the plan, unknown of no more than was lost: what the field
/// does not ask leaves it a value. Each is a Plate Vest, whose Armour the
/// base ranges 19–27, so Armour 23 is the roll 23 and 100 × 4 / 8 = 50;
/// under `+17 to Armour`, Armour 40 is the same roll.
///
/// - `no_roll`: an implicit `+20 to maximum Energy Shield` on an item of
///   neither mark — a twin text, of a type no roll of the base reads.
/// - `crucible`: the same text in `crucibleMods`, where a defence line a
///   roll reads would leave the field open (J33).
/// - `long`: an explicit `+10000000000 to maximum Energy Shield`, whose
///   number the search does not read: a line of a type no roll reads.
/// - `flag`: `+17 to Armour` under a flag that is no yes or no (J32).
/// - `flags`: the same line whose `flags` is no object.
/// - `mark`: an item whose `searing` is no yes or no, with no implicit
///   twin: an explicit twin, `+17 to Armour`, and an implicit only a
///   local stat displays, `10% increased Armour and Evasion`. Armour 44 is
///   (23 + 17) × 1.10.
/// - `other`: an array that holds no defence line, read: a flask's own.
///
/// And beside them what does open it, so that the test is of the grain:
/// `mark_twin`, the unread mark with an implicit twin a roll reads;
/// `long_own`, the unread number on a line a roll reads; `nameless`, a
/// ring with a property whose name is a number, which the deriver cannot
/// read and which may be a defence — where `unnamed`, a ring with a
/// property whose name is null, which the deriver reads as a property of
/// no name (`derive.rs`), displays no defence and lacks the field.
#[test]
fn rule_8_what_the_percentile_does_not_ask_does_not_open_it() {
    let vest = |id: &str, armour_shown: &str, more: Value| {
        armour(id, "Plate Vest", "Rare", &[("Armour", armour_shown)], more)
    };
    let s = one_tab(vec![
        vest(
            "no_roll",
            "23",
            json!({ "implicitMods": ["+20 to maximum Energy Shield"] }),
        ),
        vest(
            "crucible",
            "23",
            json!({ "crucibleMods": ["+20 to maximum Energy Shield"] }),
        ),
        vest(
            "long",
            "23",
            json!({ "explicitMods": ["+10000000000 to maximum Energy Shield"] }),
        ),
        vest(
            "flag",
            "40",
            json!({ "explicitMods": [{ "description": "+17 to Armour", "flags": { "crafted": "yes" } }] }),
        ),
        vest(
            "flags",
            "40",
            json!({ "explicitMods": [{ "description": "+17 to Armour", "flags": 7 }] }),
        ),
        vest(
            "mark",
            "44",
            json!({ "searing": "yes", "explicitMods": ["+17 to Armour"],
                    "implicitMods": ["10% increased Armour and Evasion", "+25 to maximum Life"] }),
        ),
        vest(
            "other",
            "23",
            json!({ "utilityMods": ["40% increased Movement Speed"] }),
        ),
        vest(
            "mark_twin",
            "23",
            json!({ "searing": "yes", "implicitMods": ["19% increased Armour"] }),
        ),
        vest(
            "long_own",
            "40",
            json!({ "explicitMods": ["+10000000000 to Armour"] }),
        ),
        item(
            "nameless",
            "",
            "Ruby Ring",
            "Rare",
            json!({ "properties": [{ "name": 7, "values": [["23", 0]], "displayMode": 0 }] }),
        ),
        item(
            "unnamed",
            "",
            "Ruby Ring",
            "Rare",
            json!({ "properties": [{ "name": null, "values": [["23", 0]], "displayMode": 0 }] }),
        ),
    ]);
    let found = |text: &str| ids(&asked(&s, text));
    assert_eq!(
        found("pseudo.defence_pct=50"),
        [
            "crucible", "flag", "flags", "long", "mark", "no_roll", "other"
        ]
    );
    assert_eq!(
        found("undecided(pseudo.defence_pct)"),
        ["long_own", "mark_twin", "nameless"]
    );
    assert_eq!(found("-has:pseudo.defence_pct"), ["unnamed"]);
    let kinds = |id: &str| -> Vec<String> {
        asked(&s, &format!("id:{id} has:pseudo.defence_pct"))["total"]["undecided_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["unread"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(kinds("mark_twin"), ["`searing`"]);
    assert_eq!(kinds("long_own"), ["the numbers of explicit lines"]);
    assert_eq!(kinds("nameless"), ["`properties`"]);
}

// ---- the twin ---------------------------------------------------------------------------------

/// What the field asks opens it, and of no more than was lost (rule 8 of
/// the plan; C93), each a rule no row of the table reaches: an array of
/// lines that is no array, a crucible node's among them (J29 is the
/// totals' rule), and `hybrid` unread, may hold a line it reads; a
/// defence displayed twice, an unread element named for a defence, and
/// one named `Quality` beside a defence, are properties it reads — the
/// quality asked only of an item displaying a defence; a text only a
/// local stat displays is the item's own on an eldritch item too; a twin
/// text in an array J8 does not name, the enchant's, is of no side
/// (J33); each of the three marks J8 reads, unread, opens an implicit
/// twin under its own key; and a base that could not be read is unread
/// under `baseType`, never an item with no base.
#[test]
fn rule_8_what_the_percentile_asks_opens_it() {
    let vest = |id: &str, armour_shown: &str, more: Value| {
        armour(id, "Plate Vest", "Rare", &[("Armour", armour_shown)], more)
    };
    let unread = |name: &str| json!({ "name": name, "values": 7, "displayMode": 0 });
    let mut twice = vest("twice", "23", json!({}));
    twice["properties"] = json!([property("Armour", "23"), property("Armour", "23")]);
    let mut named = vest("named", "23", json!({}));
    named["properties"] = json!([unread("Armour")]);
    let mut quality = vest("quality", "23", json!({}));
    quality["properties"] = json!([property("Armour", "23"), unread("Quality")]);
    let s = one_tab(vec![
        vest(
            "crucible_array",
            "23",
            json!({ "crucibleMods": "no array" }),
        ),
        vest("enchant_array", "23", json!({ "enchantMods": 7 })),
        vest("hybrid", "23", json!({ "hybrid": 7 })),
        twice,
        named,
        quality,
        item(
            "quality_alone",
            "",
            "Ruby Ring",
            "Rare",
            json!({ "properties": [unread("Quality")] }),
        ),
        // (roll + 0) × 1.10: roll 23 displays 25.3, 25; left out, the roll
        // would be 25 and the value 75
        vest(
            "local_only",
            "25",
            json!({ "searing": true, "implicitMods": ["10% increased Armour and Evasion"] }),
        ),
        vest(
            "enchant_twin",
            "40",
            json!({ "enchantMods": ["+17 to Armour"] }),
        ),
        vest(
            "tangled",
            "23",
            json!({ "tangled": "yes", "implicitMods": ["19% increased Armour"] }),
        ),
        vest(
            "synthesised",
            "23",
            json!({ "synthesised": "yes", "implicitMods": ["19% increased Armour"] }),
        ),
        vest("base", "23", json!({ "baseType": 7 })),
    ]);
    let found = |text: &str| ids(&asked(&s, text));
    assert_eq!(found("pseudo.defence_pct=50"), ["local_only"]);
    assert_eq!(found("-has:pseudo.defence_pct"), ["quality_alone"]);
    assert_eq!(
        found("undecided(pseudo.defence_pct)"),
        [
            "base",
            "crucible_array",
            "enchant_array",
            "enchant_twin",
            "hybrid",
            "named",
            "quality",
            "synthesised",
            "tangled",
            "twice"
        ]
    );
    let kinds = |id: &str| -> Vec<String> {
        asked(&s, &format!("id:{id} has:pseudo.defence_pct"))["total"]["undecided_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["unread"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(kinds("tangled"), ["`tangled`"]);
    assert_eq!(kinds("synthesised"), ["`synthesised`"]);
    assert_eq!(kinds("base"), ["`baseType`"]);
    assert_eq!(kinds("crucible_array"), ["crucible lines"]);
    assert_eq!(kinds("hybrid"), ["`hybrid`"]);
    assert_eq!(kinds("twice"), ["`properties`"]);
    assert_eq!(kinds("named"), ["`properties`"]);
    assert_eq!(kinds("quality"), ["`properties`"]);
    assert_eq!(
        kinds("enchant_twin"),
        ["the percentile: a twin text in an array no ruling names"]
    );
}

/// Four edges the look at the build found (2026-10-08), each a rule
/// already stated, misapplied at its edge:
///
/// - *A mark is unread wherever the deriver keeps it.* A mark is a yes
///   under its own key or under `influences`, so one that is no yes or no
///   under `influences` is unread too: a synthesised item whose
///   `influences.searing` is unreadable may be of both marks, and its
///   implicit twin is open — never the synthesised item's own.
/// - *Unknown of no more than was lost* (rule 8). An item `tangled` is
///   eldritch whatever `searing` says, so an unreadable `searing` beside
///   it opens nothing: the implicit twin is the global stat, left out.
/// - *A crucible node's line the field reads is undecided* (J33), the
///   enchant that drops the quality term among them: it is no reason to
///   drop it.
/// - *A reason says what failed.* Under `100% reduced Armour` every roll
///   displays 0: the display does not fix the value, which is several
///   rolls of different values and never "no roll gives the display".
#[test]
fn the_edges_of_a_mark_a_crucible_enchant_and_a_display_every_roll_gives() {
    let vest = |id: &str, properties: &[(&str, &str)], more: Value| {
        armour(id, "Plate Vest", "Rare", properties, more)
    };
    let twin = json!(["19% increased Armour"]);
    let s = one_tab(vec![
        vest(
            "influence",
            &[("Armour", "23")],
            json!({ "synthesised": true, "influences": { "searing": "x" }, "implicitMods": twin }),
        ),
        vest(
            "influences",
            &[("Armour", "23")],
            json!({ "synthesised": true, "influences": "no object", "implicitMods": twin }),
        ),
        // left out: the roll is the display, 23 of 19–27
        vest(
            "eldritch",
            &[("Armour", "23")],
            json!({ "tangled": true, "searing": "yes", "implicitMods": twin }),
        ),
        // an eldritch item that may be synthesised too is of one mark or both
        vest(
            "maybe_both",
            &[("Armour", "23")],
            json!({ "tangled": true, "synthesised": "yes", "implicitMods": twin }),
        ),
        vest(
            "enchant",
            &[("Quality", "+20%"), ("Armour", "23")],
            json!({ "crucibleMods": ["Quality does not increase Defences"] }),
        ),
        vest(
            "nothing",
            &[("Armour", "0")],
            json!({ "explicitMods": ["100% reduced Armour"] }),
        ),
    ]);
    let found = |text: &str| ids(&asked(&s, text));
    assert_eq!(found("pseudo.defence_pct=50"), ["eldritch"]);
    assert_eq!(
        found("undecided(pseudo.defence_pct)"),
        [
            "enchant",
            "influence",
            "influences",
            "maybe_both",
            "nothing"
        ]
    );
    let kinds = |id: &str| -> Vec<String> {
        asked(&s, &format!("id:{id} has:pseudo.defence_pct"))["total"]["undecided_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["unread"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(kinds("influence"), ["`influences.searing`"]);
    assert_eq!(kinds("influences"), ["`influences`"]);
    assert_eq!(kinds("maybe_both"), ["`synthesised`"]);
    assert_eq!(
        kinds("enchant"),
        ["the percentile: a defence line of a crucible node"]
    );
    assert_eq!(
        kinds("nothing"),
        ["the percentile: several rolls of different values"]
    );
}

/// J8, J33, over the table's composed cases (`r-j8-…`, `r-twin-…`,
/// `r-crucible-line`): a Plate Vest, Armour ranged 19–27, displaying
/// Armour 23 beside `19% increased Armour`, a text the local stat and the
/// global one both display. Worked by hand: left out, the roll is 23 and
/// 100 × 4 / 8 = 50; the item's own, the roll is 19 — 19 × 1.19 = 22.61,
/// shown 23, where 20 × 1.19 = 23.8 is shown 24 — and 100 × 0 / 8 = 0.
///
/// - `eldritch`: the line an implicit of a `tangled` item — the global
///   stat, left out: 50. `searing` the same.
/// - `synthesised`: an implicit of a synthesised item — its own: 0.
/// - `explicit`: an explicit of an item of no mark — its own: 0.
/// - `neither`: an implicit of an item of neither mark — undecided.
/// - `both`: an implicit of an item tangled and synthesised — undecided.
/// - `scourge`: the line in an array J8 does not name — undecided.
/// - `crucible`: the line in `crucibleMods` — undecided, as is `node`,
///   a crucible line only the local stat displays, `19% increased Armour
///   and Evasion`.
/// - `local`: that text as an implicit of an item of neither mark, which
///   no global stat displays — its own: 0.
#[test]
fn j8_the_items_mark_tells_whose_an_implicit_twin_is() {
    let vest = |id: &str, more: Value| armour(id, "Plate Vest", "Rare", &[("Armour", "23")], more);
    let twin = "19% increased Armour";
    let s = one_tab(vec![
        vest(
            "eldritch",
            json!({ "tangled": true, "implicitMods": [twin] }),
        ),
        vest(
            "searing",
            json!({ "searing": true, "implicitMods": [twin] }),
        ),
        vest(
            "synthesised",
            json!({ "synthesised": true, "implicitMods": [twin] }),
        ),
        vest("explicit", json!({ "explicitMods": [twin] })),
        vest("neither", json!({ "implicitMods": [twin] })),
        vest(
            "both",
            json!({ "tangled": true, "synthesised": true, "implicitMods": [twin] }),
        ),
        vest("scourge", json!({ "scourgeMods": [twin] })),
        vest("crucible", json!({ "crucibleMods": [twin] })),
        vest(
            "node",
            json!({ "crucibleMods": ["19% increased Armour and Evasion"] }),
        ),
        vest(
            "local",
            json!({ "implicitMods": ["19% increased Armour and Evasion"] }),
        ),
    ]);
    let found = |text: &str| ids(&asked(&s, text));
    assert_eq!(found("pseudo.defence_pct=50"), ["eldritch", "searing"]);
    assert_eq!(
        found("pseudo.defence_pct=0"),
        ["explicit", "local", "synthesised"]
    );
    assert_eq!(
        found("undecided(pseudo.defence_pct)"),
        ["both", "crucible", "neither", "node", "scourge"]
    );
    let kind = |id: &str| -> Value {
        let a = asked(&s, &format!("id:{id} has:pseudo.defence_pct"));
        let why = a["total"]["undecided_reasons"].as_array().unwrap();
        assert_eq!(why.len(), 1, "{id}: {why:?}");
        why[0]["unread"].clone()
    };
    assert_eq!(
        kind("neither"),
        "the percentile: an implicit twin on an item of neither mark"
    );
    assert_eq!(
        kind("both"),
        "the percentile: an implicit twin on an item of both marks"
    );
    assert_eq!(
        kind("scourge"),
        "the percentile: a twin text in an array no ruling names"
    );
    for id in ["crucible", "node"] {
        assert_eq!(
            kind(id),
            "the percentile: a defence line of a crucible node"
        );
    }
}

// ---- the refusal ------------------------------------------------------------------------------

/// The record's observation from 9d-C: `has:` asks a computed value's
/// presence and takes no slot word. On a ranged total the refusal is of
/// its own kind — a slot word where none is taken — never the unknown
/// name it was, and offers the reading that is right.
#[test]
fn has_with_a_slot_word_on_a_computed_value_is_refused_under_its_own_kind() {
    let s = worked();
    for slot in ["avg", "low", "high", "arg1"] {
        for (text, reading) in [
            (
                format!("has:pseudo.adds_fire.{slot}"),
                "has:pseudo.adds_fire",
            ),
            (
                format!("-has:pseudo.adds_fire.{slot}"),
                "has:pseudo.adds_fire",
            ),
            (
                format!("has:Pseudo.Adds_Cold_To_Attacks.{slot}"),
                "has:pseudo.adds_cold_to_attacks",
            ),
        ] {
            let e = refused(&s, &text);
            assert_eq!(e["kind"], "slot_unknown", "`{text}`: {e}");
            assert_eq!(e["readings"], json!([reading]), "`{text}`: {e}");
            assert!(
                e["error"].as_str().unwrap().contains("takes no slot word"),
                "`{text}`: {e}"
            );
        }
    }
    // a computed value that is one number takes none either: the field
    // this piece built, a total that is no range, a derived field
    for (text, reading) in [
        ("has:pseudo.defence_pct.avg", "has:pseudo.defence_pct"),
        ("has:pseudo.total_res.avg", "has:pseudo.total_res"),
        ("-has:pseudo.dps.low", "has:pseudo.dps"),
    ] {
        let e = refused(&s, text);
        assert_eq!(e["kind"], "slot_unknown", "`{text}`: {e}");
        assert_eq!(e["readings"], json!([reading]), "`{text}`: {e}");
        assert!(
            e["error"].as_str().unwrap().contains("takes no slot word"),
            "`{text}`: {e}"
        );
    }
    // the reading it offers binds; a name nothing defines is unknown
    // still, with a slot word as without one
    assert_eq!(ids(&asked(&s, "has:pseudo.adds_fire")), [] as [&str; 0]);
    assert_eq!(
        refused(&s, "has:pseudo.adds_fir.avg")["kind"],
        "unknown_name"
    );
}

// ---- the help ---------------------------------------------------------------------------------

/// C97, C101: `--describe` of the field prints its definition as every
/// computed value's is printed — the site's rule in the site's words, the
/// one departure (J5), how a twin text is told with the captured lines
/// each part rests on, that a roll out of range still reads (J34), where
/// it is undecided (J33), the site's inexact bound (C102), and the
/// reference data's source and version — and the field is no longer on
/// the list of what is not built.
#[test]
fn c97_describe_of_the_percentile_prints_its_definition() {
    let one = serde_json::to_value(describe(&["defence_pct".to_string()]).unwrap()).unwrap();
    assert_eq!(one["computed"].as_array().unwrap().len(), 1);
    let entry = &one["computed"][0];
    assert_eq!(entry["name"], FIELD);
    assert_eq!(entry["kind"], "derived");
    let what = entry["what"].as_str().unwrap();
    for says in [
        "Base Percentile",
        "The percentile of all base defence rolls, averaged",
        "exact half",
        "17",
        "442",
        "10 ",
        "over 100",
        "below 0",
        "crucible",
        "inexact",
        "defences v1",
        "RePoE",
        "undecided",
    ] {
        assert!(what.contains(says), "{says}: {what}");
    }
    let examples: Vec<&str> = entry["examples"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_str().unwrap())
        .collect();
    assert!(
        examples.contains(&"-has:pseudo.defence_pct"),
        "{examples:?}"
    );
    let whole = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    assert!(
        whole["not_built"]
            .as_array()
            .unwrap()
            .iter()
            .all(|n| n["construct"] != FIELD)
    );
    // every example printed is a query this build binds
    let s = worked();
    for example in examples {
        asked(&s, example);
    }
}

/// A vocabulary narrowing lists the percentile by its name or by what it
/// reads and answers — its own definition — and never by the bookkeeping
/// its definition prints beside that: the captured lines a part rests on
/// and the script that counted them, the table's version, its sources
/// and its counts. By a word of those it would be listed under a text
/// that names no line and no value (T5: a narrowing lists those it
/// matches; the look at the build, 2026-10-08).
#[test]
fn c97_a_narrowing_matches_what_the_percentile_reads_and_never_its_bookkeeping() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let computed = |narrow: &str| -> Vec<String> {
        viewed(
            &corpus,
            "",
            json!({ "counts": { "keys": [format!("line:{narrow}")], "limit": 1000 } }),
        )["view"]["counts"]["tables"][0]["buckets"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| b["bucket"] == "computed")
            .map(|b| b["value"].as_str().unwrap().to_string())
            .collect()
    };
    // every one of these is in the definition `--describe` prints
    let one = serde_json::to_value(describe(&["defence_pct".to_string()]).unwrap()).unwrap();
    let what = one["computed"][0]["what"].as_str().unwrap().to_string();
    for word in [
        "442",
        "17 of 17",
        "defence-twins",
        "scripts",
        "defences v1",
        "RePoE",
        "476",
    ] {
        assert!(what.contains(word), "{word}: {what}");
        assert_eq!(computed(word), [] as [&str; 0], "line:{word}");
    }
    for word in ["percentile", "defence_pct", "exact half", "crucible node"] {
        assert_eq!(computed(word), [FIELD], "line:{word}");
    }
}
