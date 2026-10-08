//! The ranged family (C94; `search/LEDGER.md`, "Holes ruled", J7 and
//! J29–J31; the build plan, 9d-C), through the crate's boundary: the 21
//! totals named `adds_…`, whose rows mean one of two stats displaying one
//! text, and the rule of every total that the piece's sitting brought —
//! no total reads a crucible node's lines or a flask's own (J29).
//!
//! The first test reads the track's expected answers where they lie
//! (`search/pseudo-stats/data/ranged-expected.csv`, written before
//! anything was built, from the site's captures and the owner's rulings)
//! and holds the build to every row. The others are fixtures worked by
//! hand before running: the plan's closing row, J29 on the totals shipped
//! before the family, and a row that reads the item's category under
//! `not` and `or` (C93).

mod common;

use std::collections::{BTreeMap, BTreeSet};

use acquisition_search::{Corpus, Request, answer, describe};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

/// The expected answers, one row a case and a total: the tests' side,
/// never read by the build.
const EXPECTED: &str = include_str!("../../../search/pseudo-stats/data/ranged-expected.csv");

/// The trade site's captures, as captured.
const CAPTURES: &str = include_str!("../../../search/pseudo-stats/data/captures.json");

// ---- reading the table ------------------------------------------------------------------------

/// What a total answers of a case.
#[derive(Debug, Clone, PartialEq)]
enum Cell {
    Lacked,
    Value {
        low: f64,
        high: f64,
        avg: f64,
    },
    /// With the cause the table gives: what could not be established.
    Undecided(String),
}

/// One case of the expected answers: an item, and what each total the
/// table lists answers of it. A total a case does not list is lacked.
struct Case {
    /// The item's id in the fixture store: the case's place in the table.
    id: String,
    /// The table's name for the case.
    name: String,
    realm: String,
    base: String,
    rarity: String,
    unique: String,
    /// The `lines` column, as the row gives it.
    lines: String,
    cells: BTreeMap<String, Cell>,
}

fn number(text: &str) -> f64 {
    text.parse::<f64>()
        .unwrap_or_else(|_| panic!("the table holds a number the test does not read: {text:?}"))
}

fn cases() -> Vec<Case> {
    let mut cases: Vec<Case> = Vec::new();
    for row in table(EXPECTED) {
        if cases.last().is_none_or(|case| case.name != row["case"]) {
            assert!(
                cases.iter().all(|case| case.name != row["case"]),
                "the rows of {} are apart",
                row["case"]
            );
            cases.push(Case {
                id: format!("c{:03}", cases.len()),
                name: row["case"].clone(),
                realm: row["realm"].clone(),
                base: row["base"].clone(),
                rarity: row["rarity"].clone(),
                unique: row["unique"].clone(),
                lines: row["lines"].clone(),
                cells: BTreeMap::new(),
            });
        }
        let case = cases.last_mut().unwrap();
        // one item a case: every row of it gives the same facts
        assert_eq!(
            (&case.realm, &case.base, &case.rarity, &case.lines),
            (&row["realm"], &row["base"], &row["rarity"], &row["lines"]),
            "{}",
            case.name
        );
        let cell = match row["status"].as_str() {
            "lacked" => Cell::Lacked,
            "value" => Cell::Value {
                low: number(&row["low"]),
                high: number(&row["high"]),
                avg: number(&row["avg"]),
            },
            "undecided" => Cell::Undecided(row["cause_or_readings"].clone()),
            other => panic!("{}: a status the test does not read: {other}", case.name),
        };
        assert!(
            case.cells.insert(row["total"].clone(), cell).is_none(),
            "{} lists {} twice",
            case.name,
            row["total"]
        );
    }
    cases
}

/// A case's lines as the arrays of its body: `array[flags]: text | …`,
/// each line entered as the row gives it — its array, its flags, a mod of
/// several rows (` ⏎ `) as one entry with its row breaks. An array the
/// row gives as no array is entered as what the row gives. The lines the
/// table left out, `(+N lines no row names)`, are no line of the family
/// and are not entered; the count of them is returned.
fn lines_of(case: &Case) -> (BTreeMap<String, Value>, usize) {
    let mut arrays: BTreeMap<String, Value> = BTreeMap::new();
    let mut left_out = 0;
    for piece in case.lines.split(" | ").filter(|p| !p.is_empty()) {
        if let Some(count) = piece
            .strip_prefix("(+")
            .and_then(|p| p.strip_suffix(" lines no row names)"))
        {
            left_out += count.parse::<usize>().unwrap();
            continue;
        }
        let (head, text) = piece
            .split_once(": ")
            .unwrap_or_else(|| panic!("{}: no line: {piece:?}", case.name));
        let (array, marks) = match head.split_once('[') {
            Some((array, marks)) => (array, marks.strip_suffix(']').unwrap()),
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
            assert!(marks.is_empty() && !arrays.contains_key(array));
            arrays.insert(array.to_string(), serde_json::from_str(given).unwrap());
            continue;
        }
        assert!(!text.starts_with("(no "), "{}: {piece:?}", case.name);
        let text = text.replace(" ⏎ ", "\n");
        let element = if marks.is_empty() {
            json!(text)
        } else {
            let mut flags = serde_json::Map::new();
            for mark in marks.split(';') {
                match mark.split_once('=') {
                    None => flags.insert(mark.to_string(), json!(true)),
                    // a flag that is no yes or no, as the script's `repr` wrote it
                    Some((flag, value)) => {
                        let value = value
                            .strip_suffix(" (unread)")
                            .and_then(|v| v.strip_prefix('\''))
                            .and_then(|v| v.strip_suffix('\''))
                            .unwrap_or_else(|| panic!("{}: {piece:?}", case.name));
                        flags.insert(flag.to_string(), json!(value))
                    }
                };
            }
            json!({ "description": text, "flags": flags })
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

/// An item of a case's facts, as GGG would give it: the base as shown,
/// the rarity — and the frame, which GGG words as the rarity on an item
/// that has one — the unique's name, and the lines; nothing where the
/// row has nothing.
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

/// Each item's sort scalar under `by`: its value, or its status.
fn sorted(corpus: &Corpus, by: &str) -> BTreeMap<String, Value> {
    viewed(
        corpus,
        "",
        json!({ "rows": { "sort": by, "limit": EVERY } }),
    )["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap().to_string(), scalar(&r["sort"])))
        .collect()
}

/// What an undecided cause names, as the kind a reason and a count's
/// tally carry (`eval::reason`): what could not be established, never
/// the table's wording of it.
fn kind_of(cause: &str) -> &'static str {
    match cause {
        c if c.starts_with("realm: ") => "the total: no definition for the realm",
        "unread: explicitMods is no array" => "explicit lines",
        "unread: a flag of a line a row names is no yes or no" => "the flags of explicit lines",
        "side: the base is not in the class table" => "the class: base not in the table",
        "side: the item has no base" => "the class: no base",
        other => panic!("a cause the test does not read: {other}"),
    }
}

/// C94, J7, J29–J31: every row of the expected answers — 598 cases of
/// the facts the row gives, put into a store through its ingest, each
/// line in the array and under the flags the row gives — is answered as
/// the row says. Each of the 21 totals is asked of each case: lacked, by
/// `has:` and its not; or its `low`, `high` and `avg`, each by its slot
/// word, and the bare name beside `avg` (the owner, 2026-09-26: "yes,
/// bare means average"); or undecided, by what could not be established.
/// A total a case does not list is lacked.
#[test]
fn c94_every_row_of_the_ranged_expected_answers_is_answered_as_the_row_says() {
    let cases = cases();
    assert_eq!(cases.len(), 598);
    let totals: BTreeSet<String> = cases
        .iter()
        .flat_map(|case| case.cells.keys().cloned())
        .collect();
    assert_eq!(totals.len(), 21);
    assert!(totals.iter().all(|total| total.starts_with("adds_")));
    // the input holds what the test is of: a mod of several rows, a line
    // under a flag, a crucible node's line, a flask's own, an array and a
    // flag that could not be read
    for held in [
        " ⏎ ",
        "[crafted]: ",
        "[fractured]: ",
        "crucibleMods: ",
        "utilityMods: ",
        "(no array: ",
        "(unread)]: ",
    ] {
        assert!(cases.iter().any(|c| c.lines.contains(held)), "{held}");
    }
    assert_eq!(
        cases.iter().map(|c| lines_of(c).1).sum::<usize>(),
        2687,
        "the lines the table left out as no row's"
    );

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
    let mut faults: Vec<String> = Vec::new();
    let mut asked = (0, 0, 0);
    for realm in ["pc", "poe2"] {
        let corpus = load(&s, Some(realm));
        let of_realm: Vec<&Case> = cases.iter().filter(|c| c.realm == realm).collect();
        let all: BTreeSet<String> = of_realm.iter().map(|c| c.id.clone()).collect();
        assert_eq!(id_set(&rows_of(&corpus, "")), all);
        for total in &totals {
            let has = id_set(&rows_of(&corpus, &format!("has:pseudo.{total}")));
            let lacks = id_set(&rows_of(&corpus, &format!("-has:pseudo.{total}")));
            let open = id_set(&rows_of(&corpus, &format!("undecided(pseudo.{total})")));
            // one status each: the three partition the realm's items
            assert_eq!(has.len() + lacks.len() + open.len(), all.len(), "{total}");
            assert_eq!(
                has.iter()
                    .chain(&lacks)
                    .chain(&open)
                    .cloned()
                    .collect::<BTreeSet<String>>(),
                all,
                "{total}"
            );
            let by = |slot: &str| sorted(&corpus, &format!("pseudo.{total}{slot}"));
            let (low, high, avg, bare) = (by(".low"), by(".high"), by(".avg"), by(""));
            for case in &of_realm {
                let expected = case.cells.get(total).unwrap_or(&Cell::Lacked);
                let status = match (
                    has.contains(&case.id),
                    lacks.contains(&case.id),
                    open.contains(&case.id),
                ) {
                    (true, false, false) => "value",
                    (false, true, false) => "lacked",
                    (false, false, true) => "undecided",
                    other => panic!("{} {total}: {other:?}", case.name),
                };
                let said = |answers: String| {
                    format!(
                        "{} ({}) pseudo.{total}: the table says {expected:?}, the build {answers}",
                        case.name, case.id
                    )
                };
                let scalars = [&low, &high, &avg, &bare].map(|by| by[&case.id].clone());
                match expected {
                    Cell::Lacked => {
                        asked.0 += 1;
                        if status != "lacked"
                            || scalars
                                .iter()
                                .any(|s| s["status"] != "no satisfying occurrence")
                        {
                            faults.push(said(format!("{status} {}", json!(scalars))));
                        }
                    }
                    Cell::Value { low, high, avg } => {
                        asked.1 += 1;
                        let read: Vec<Option<f64>> =
                            scalars.iter().map(|s| s["value"].as_f64()).collect();
                        if status != "value"
                            || read != [Some(*low), Some(*high), Some(*avg), Some(*avg)]
                            || scalars.iter().any(|s| !s["status"].is_null())
                        {
                            faults.push(said(format!("{status} {}", json!(scalars))));
                        }
                    }
                    Cell::Undecided(cause) => {
                        asked.2 += 1;
                        let a = as_json(
                            &ask(&corpus, &format!("id:{} has:pseudo.{total}", case.id)).unwrap(),
                        );
                        let kinds: BTreeSet<&str> = a["total"]["undecided_reasons"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|r| r["unread"].as_str().unwrap())
                            .collect();
                        if status != "undecided"
                            || scalars.iter().any(|s| s["status"] != "incomplete")
                            || kinds != BTreeSet::from([kind_of(cause)])
                        {
                            faults.push(said(format!("{status} {}, by {kinds:?}", json!(scalars))));
                        }
                    }
                }
            }
        }
    }
    // every cell asked: 598 cases of 21 totals, the table's 615 lacked
    // rows and each total a case does not list
    assert_eq!(asked.0 + asked.1 + asked.2, 598 * 21);
    // the table's own tally (`ranged-expected.py`, rows by kind and
    // status): 2387 values and 52 undecided since J32, three cells of
    // `r-unread-flag` a value where the table read a flag no row asks
    assert_eq!((asked.1, asked.2), (2387, 52));
    assert!(
        faults.is_empty(),
        "{} cells of {} are not answered as the table says:\n{}",
        faults.len(),
        598 * 21,
        faults.join("\n")
    );
}

// ---- the plan's closing row -------------------------------------------------------------------

fn rare(id: &str, base: &str, more: Value) -> Value {
    item(id, "", base, "Rare", more)
}

fn asked(s: &Store, text: &str) -> Value {
    as_json(&ask(&load(s, Some("pc")), text).unwrap())
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

/// The table's two worked cases of the fire line (`w-fire-on-a-weapon`,
/// `w-fire-on-no-weapon`): `wand`, a Driftwood Wand, and `ring`, a Ruby
/// Ring, each with `Adds 10 to 21 Fire Damage` and nothing else.
fn fire() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("f1", "Fire")]), 10);
    let line = json!({ "explicitMods": ["Adds 10 to 21 Fire Damage"] });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "f1",
        "Fire",
        vec![
            rare("wand", "Driftwood Wand", line.clone()),
            rare("ring", "Ruby Ring", line),
        ],
        20,
    );
    s
}

/// C94, J7 — the plan's closing row: `Adds # to # Fire Damage` on a
/// weapon is the weapon's own line and counts toward the attacks' total
/// alone; off a weapon it is the other twin and counts toward the plain,
/// the attacks' and the spells'. Worked by hand: one line, low 10, high
/// 21, avg (10 + 21) / 2 = 15.5. Asked by comparison, by `has:`, sorted,
/// and summed in a count; the bare name is the `avg`.
#[test]
fn c94_adds_fire_damage_on_a_weapon_counts_toward_the_attacks_total_alone() {
    let s = fire();
    let found = |text: &str| ids(&asked(&s, text));

    // by comparison: the wand lacks the plain total and the spells'
    let a = asked(&s, "pseudo.adds_fire>=15.5");
    assert_eq!(ids(&a), ["ring"]);
    assert_eq!(counts(&a["terms"][0]), [1, 0, 1, 0]);
    assert_eq!(follow(&s, &a["terms"][0]["lacked"]), ["wand"]);
    for attacks in [
        "pseudo.adds_fire_to_attacks>=15.5",
        "pseudo.adds_fire_to_attacks=15.5",
        "pseudo.adds_fire_to_attacks.avg=15.5",
        "pseudo.adds_fire_to_attacks.low=10 pseudo.adds_fire_to_attacks.high=21",
        "pseudo.adds_elemental_to_attacks=15.5 pseudo.adds_damage_to_attacks=15.5",
    ] {
        assert_eq!(found(attacks), ["ring", "wand"], "{attacks}");
    }
    for other in [
        "pseudo.adds_fire=15.5",
        "pseudo.adds_fire.avg=15.5",
        "pseudo.adds_fire.low=10 pseudo.adds_fire.high=21",
        "pseudo.adds_fire_to_spells=15.5",
        "pseudo.adds_elemental=15.5 pseudo.adds_damage=15.5",
        "pseudo.adds_elemental_to_spells.low=10 pseudo.adds_damage_to_spells.high=21",
    ] {
        assert_eq!(found(other), ["ring"], "{other}");
    }
    // a comparison on a total the item lacks is false, never against zero
    let a = asked(&s, "pseudo.adds_fire<15.5");
    assert_eq!(counts(&a["terms"][0]), [0, 1, 1, 0]);
    assert_eq!(a["total"]["matched"], 0);
    // the bare name is its `avg`, never its low or its high
    assert_eq!(asked(&s, "pseudo.adds_fire=10")["total"]["matched"], 0);
    assert_eq!(asked(&s, "pseudo.adds_fire=21")["total"]["matched"], 0);
    // a row shows the value under the name as asked, then the line counted
    let a = asked(&s, "pseudo.adds_fire>=15.5");
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"],
        json!([
            { "value": { "name": "pseudo.adds_fire", "value": 15.5 } },
            { "line": { "source": "explicit", "flags": [], "text": "Adds 10 to 21 Fire Damage" } },
        ])
    );

    // by `has:`
    assert_eq!(found("has:pseudo.adds_fire"), ["ring"]);
    assert_eq!(found("-has:pseudo.adds_fire"), ["wand"]);
    assert_eq!(found("has:pseudo.adds_fire_to_attacks"), ["ring", "wand"]);
    assert_eq!(found("-has:pseudo.adds_fire_to_spells"), ["wand"]);
    assert_eq!(
        asked(&s, "undecided(pseudo.adds_fire)")["total"]["matched"],
        0
    );

    // sorted: the wand has no plain total to sort by
    let corpus = load(&s, Some("pc"));
    for (by, ring, wand) in [
        ("pseudo.adds_fire", json!({ "value": 15.5 }), None),
        ("pseudo.adds_fire.avg", json!({ "value": 15.5 }), None),
        ("pseudo.adds_fire.low", json!({ "value": 10 }), None),
        ("pseudo.adds_fire.high", json!({ "value": 21 }), None),
        (
            "pseudo.adds_fire_to_attacks",
            json!({ "value": 15.5 }),
            Some(json!({ "value": 15.5 })),
        ),
        (
            "pseudo.adds_fire_to_attacks.high",
            json!({ "value": 21 }),
            Some(json!({ "value": 21 })),
        ),
    ] {
        let rows = sorted(&corpus, by);
        assert_eq!(rows["ring"], ring, "--sort {by}");
        assert_eq!(
            rows["wand"],
            wand.unwrap_or(json!({ "status": "no satisfying occurrence" })),
            "--sort {by}"
        );
    }

    // summed in a count: 15.5 and one lacking; 15.5 + 15.5; low 10 + 10
    let sum = |of: &str| {
        viewed(
            &corpus,
            "",
            json!({ "counts": { "keys": ["base"], "sum": of } }),
        )["view"]["counts"]["sum"]
            .clone()
    };
    assert_eq!(
        sum("pseudo.adds_fire"),
        json!({ "name": "pseudo.adds_fire", "value": 15.5, "lacking": 1 })
    );
    assert_eq!(
        sum("pseudo.adds_fire_to_attacks"),
        json!({ "name": "pseudo.adds_fire_to_attacks", "value": 31, "lacking": 0 })
    );
    assert_eq!(
        sum("pseudo.adds_fire_to_attacks.low"),
        json!({ "name": "pseudo.adds_fire_to_attacks.low", "value": 20, "lacking": 0 })
    );
}

// ---- J29 ----------------------------------------------------------------------------------------

/// Seven items in `Arrays` (a1), worked by hand from J29 — a total reads
/// no line of `crucibleMods` or `utilityMods`, and neither array can
/// leave a total open:
///
/// - `flask`, a Quicksilver Flask whose own line (`utilityMods`) is `40%
///   increased Movement Speed`, a row's template: lacks the total;
/// - `boots`, the same line as an explicit: 40;
/// - `amethyst`, an Amethyst Flask whose own line is `+35% to Chaos
///   Resistance`: lacks the chaos total, every resistance total and the
///   counts that read them;
/// - `staff`, a crucible node's `15% increased Spell Damage` and an
///   explicit `30% increased Spell Damage`: 30, the explicit's alone;
/// - `sceptre`, a crucible array that is no array and an explicit `20%
///   increased Spell Damage`: 20, a total and no subtotal;
/// - `axe`, a crucible array that is no array and nothing else: lacks it;
/// - `ring`, a `utilityMods` that is no array and an explicit `+30% to
///   Fire Resistance`: 30.
fn arrays() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("a1", "Arrays")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "a1",
        "Arrays",
        vec![
            item(
                "flask",
                "",
                "Quicksilver Flask",
                "Magic",
                json!({ "utilityMods": ["40% increased Movement Speed"],
                        "explicitMods": ["39% increased Charge Recovery"] }),
            ),
            rare(
                "boots",
                "Iron Greaves",
                json!({ "explicitMods": ["40% increased Movement Speed"] }),
            ),
            item(
                "amethyst",
                "",
                "Amethyst Flask",
                "Normal",
                json!({ "utilityMods": ["+35% to Chaos Resistance"] }),
            ),
            rare(
                "staff",
                "Serpentine Staff",
                json!({ "crucibleMods": ["15% increased Spell Damage"],
                        "explicitMods": ["30% increased Spell Damage"] }),
            ),
            rare(
                "sceptre",
                "Bronze Sceptre",
                json!({ "crucibleMods": "unreadable",
                        "explicitMods": ["20% increased Spell Damage"] }),
            ),
            item(
                "axe",
                "",
                "Despot Axe",
                "Normal",
                json!({ "crucibleMods": "unreadable" }),
            ),
            rare(
                "ring",
                "Ruby Ring",
                json!({ "utilityMods": "unreadable",
                        "explicitMods": ["+30% to Fire Resistance"] }),
            ),
        ],
        20,
    );
    s
}

/// J29 (the owner, 2026-10-08, of both arrays: "Leave both out."), on
/// the totals shipped before the ranged family: no total reads a line of
/// `crucibleMods` or `utilityMods`, and an array a total does not read
/// cannot leave it incomplete. A line's term and the item's own `sum( … )`
/// read both arrays as they did: the rule is a total's.
#[test]
fn j29_no_total_reads_a_crucible_nodes_lines_or_a_flasks_own() {
    let s = arrays();
    let found = |text: &str| ids(&asked(&s, text));

    // a flask's own line is counted by no total; the explicit is
    assert_eq!(found("has:pseudo.increased_movement_speed"), ["boots"]);
    assert_eq!(found("pseudo.increased_movement_speed=40"), ["boots"]);
    let a = asked(&s, "pseudo.increased_movement_speed>=1");
    assert_eq!(counts(&a["terms"][0]), [1, 0, 6, 0]);
    assert_eq!(
        follow(&s, &a["terms"][0]["lacked"]),
        ["amethyst", "axe", "flask", "ring", "sceptre", "staff"]
    );
    // nor by a reading of totals: the amethyst flask shows no resistance
    for lacked in [
        "-has:pseudo.total_chaos_res",
        "-has:pseudo.total_res",
        "-has:pseudo.count_res",
    ] {
        assert!(found(lacked).contains(&"amethyst".to_string()), "{lacked}");
    }
    assert_eq!(found("has:pseudo.total_chaos_res"), [] as [&str; 0]);

    // a crucible node's line is counted by no total: the staff's 30 is
    // its explicit's alone, never 45
    assert_eq!(found("pseudo.increased_spell_damage=30"), ["staff"]);
    assert_eq!(
        asked(&s, "pseudo.increased_spell_damage=45")["total"]["matched"],
        0
    );
    let a = asked(&s, "pseudo.increased_spell_damage=30");
    let shown: Vec<&str> = a["rows"][0]["matched"][0]["shows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| e["line"]["source"].as_str())
        .collect();
    assert_eq!(shown, ["explicit"], "the lines the total counted");

    // an array a total does not read cannot leave it open
    let a = asked(&s, "pseudo.increased_spell_damage>=1");
    assert_eq!(ids(&a), ["sceptre", "staff"]);
    assert_eq!(counts(&a["terms"][0]), [2, 0, 5, 0]);
    assert_eq!(found("pseudo.increased_spell_damage=20"), ["sceptre"]);
    assert_eq!(found("pseudo.total_fire_res=30"), ["ring"]);
    for total in [
        "increased_spell_damage",
        "increased_movement_speed",
        "total_fire_res",
        "count_res",
    ] {
        assert_eq!(
            asked(&s, &format!("undecided(pseudo.{total})"))["total"]["matched"],
            0,
            "{total}"
        );
    }
    let corpus = load(&s, Some("pc"));
    let by = sorted(&corpus, "pseudo.increased_spell_damage");
    assert_eq!(by["sceptre"], json!({ "value": 20 }));
    assert_eq!(by["axe"], json!({ "status": "no satisfying occurrence" }));

    // the rule is a total's: a line's term, and the item's own sum of a
    // line, read both arrays and are left open by both
    assert_eq!(
        found("\"#% increased Movement Speed\"=40"),
        ["boots", "flask"]
    );
    assert_eq!(found("sum(\"#% increased Spell Damage\")=45"), ["staff"]);
    assert_eq!(
        found("line(\"#% to Chaos Resistance\" source=utility)"),
        ["amethyst"]
    );
    assert_eq!(
        found("undecided(line(\"#% increased Cast Speed\"))"),
        ["axe", "ring", "sceptre"]
    );
    assert_eq!(
        found("undecided(sum(\"#% increased Spell Damage\"))"),
        ["axe", "ring", "sceptre"]
    );
}

/// J29 on round n's captures, as captured (`search/pseudo-stats/README.md`,
/// "Round n, read"): ten items each, every one lacking the pseudo its
/// search named — n02, a crucible `#% increased Spell Damage` node; n07
/// and n09, a flask's own movement speed and chaos resistance; n05, a
/// weapon's crucible `Adds # to # Fire Damage`, under the attacks' pseudo.
/// On n05 the attacks' total is the evidence and the plain total is none:
/// every item of it is a weapon, asked here, on which the plain total
/// counts no unsuffixed line wherever it lies (J7), so its lack shows
/// nothing of J29 and is not asked.
#[test]
fn j29_round_ns_captured_items_lack_the_pseudo_their_search_named() {
    let captures: Value = serde_json::from_str(CAPTURES).unwrap();
    let named: [(&str, &[&str], &str); 4] = [
        ("n02", &["increased_spell_damage"], "crucibleMods"),
        ("n07", &["increased_movement_speed"], "utilityMods"),
        ("n09", &["total_chaos_res"], "utilityMods"),
        ("n05", &["adds_fire_to_attacks"], "crucibleMods"),
    ];
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("n", "Round n")]), 10);
    let mut bodies = Vec::new();
    for (key, _, array) in named {
        let fetched = captures["searches"][key]["fetched"].as_array().unwrap();
        assert_eq!(fetched.len(), 10, "{key}");
        for (i, capture) in fetched.iter().enumerate() {
            // the site showed no pseudo on any of them
            assert!(
                capture["lines"]["pseudoMods"]
                    .as_array()
                    .is_none_or(Vec::is_empty),
                "{key}[{i}]"
            );
            // the input holds what the test is of
            assert!(
                capture["lines"][array]
                    .as_array()
                    .is_some_and(|l| !l.is_empty()),
                "{key}[{i}] holds no line of {array}"
            );
            let mut more = json!({});
            for (array, lines) in capture["lines"].as_object().unwrap() {
                if array != "pseudoMods" {
                    more[array] = lines.clone();
                }
            }
            bodies.push(item(
                &format!("{key}_{i}"),
                capture["name"].as_str().unwrap_or(""),
                capture["baseType"].as_str().unwrap(),
                capture["rarity"].as_str().unwrap(),
                more,
            ));
        }
    }
    fetch_tab(&mut s, "pc", "Standard", "n", "Round n", bodies, 20);
    let corpus = load(&s, Some("pc"));
    let mut misses = Vec::new();
    for (key, totals, _) in named {
        for i in 0..10 {
            for total in totals {
                let query = format!("id:{key}_{i} -has:pseudo.{total}");
                match ask(&corpus, &query) {
                    Ok(a) if as_json(&a)["total"]["matched"] == 1 => {}
                    Ok(_) => misses.push(query),
                    Err(e) => misses.push(format!("{query}: {e}")),
                }
            }
        }
    }
    assert!(misses.is_empty(), "{misses:#?}");
    // n05's items are weapons, every one: why the plain total is no
    // evidence there
    for i in 0..10 {
        let weapon = ask(&corpus, &format!("id:n05_{i} category=weapon")).unwrap();
        assert_eq!(as_json(&weapon)["total"]["matched"], 1, "n05_{i}");
    }
}

/// The site's pseudo id as the total's name here: `pseudo_adds_<type>_damage…`
/// is `adds_<type>…`, and the three of every type, `pseudo_adds_damage…`,
/// keep the word.
fn total_named(site: &str) -> Option<String> {
    let rest = site.strip_prefix("stat.pseudo.pseudo_adds_")?;
    Some(if rest.starts_with("damage") {
        format!("adds_{rest}")
    } else {
        format!("adds_{}", rest.replacen("_damage", "", 1))
    })
}

/// C94, against the site and not the expected table: every captured item
/// on which the trade site showed a pseudo of the ranged family
/// (`search/pseudo-stats/data/captures.json`), entered whole — every line
/// the capture holds in the array it came in, never the table's `lines`
/// column — answers each such pseudo's number as the total's `avg`, a
/// value and no subtotal: 459 cells, of 459 items. It holds the totals to
/// what the site counted on real items, the lines no row names among
/// them; it asks nothing of a total the site did not show.
#[test]
fn c94_every_captured_item_entered_whole_answers_the_sites_number() {
    let captures: Value = serde_json::from_str(CAPTURES).unwrap();
    let mut bodies = Vec::new();
    // an item, a total, and the number the site showed of it
    let mut shown: Vec<(String, String, f64)> = Vec::new();
    for (key, search) in captures["searches"].as_object().unwrap() {
        for (i, capture) in search["fetched"].as_array().unwrap().iter().enumerate() {
            let Some(lines) = capture["lines"].as_object() else {
                continue;
            };
            let id = format!("{key}_{i}");
            let before = shown.len();
            for pseudo in lines
                .get("pseudoMods")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                let Some(total) = pseudo["hash"].as_str().and_then(total_named) else {
                    continue;
                };
                // `Adds 15.5 Fire Damage`: the one number the site shows
                let text = pseudo["description"].as_str().unwrap();
                shown.push((id.clone(), total, number(text.split(' ').nth(1).unwrap())));
            }
            if shown.len() == before {
                continue;
            }
            let mut more = json!({});
            for (array, lines) in lines {
                if array != "pseudoMods" {
                    more[array] = lines.clone();
                }
            }
            bodies.push(item(
                &id,
                capture["name"].as_str().unwrap_or(""),
                capture["baseType"].as_str().unwrap(),
                capture["rarity"].as_str().unwrap(),
                more,
            ));
        }
    }
    assert_eq!((bodies.len(), shown.len()), (459, 459));
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("w", "Whole")]), 10);
    fetch_tab(&mut s, "pc", "Standard", "w", "Whole", bodies, 20);
    let corpus = load(&s, Some("pc"));
    let mut by: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    let mut faults = Vec::new();
    for (id, total, site) in &shown {
        let answered = &by
            .entry(total.clone())
            .or_insert_with(|| sorted(&corpus, &format!("pseudo.{total}")))[id];
        if answered["value"].as_f64() != Some(*site) || !answered["status"].is_null() {
            faults.push(format!(
                "{id} pseudo.{total}: the site {site}, the build {answered}"
            ));
        }
    }
    assert!(
        faults.is_empty(),
        "{} cells of {} are not the site's:\n{}",
        faults.len(),
        shown.len(),
        faults.join("\n")
    );
}

// ---- a row that reads the category --------------------------------------------------------------

/// Five items in `Sides` (s1), worked by hand from the table's side-open
/// cases (`r-side-open-unsuffixed`, `r-side-open-scoped`), J30 and C93:
///
/// - `ring`, a Ruby Ring with `Adds 5 to 10 Fire Damage`: no weapon, so
///   the plain total, the attacks' and the spells' are each 5 to 10;
/// - `wand`, a Driftwood Wand with the same line: the weapon's own, so
///   the attacks' total alone;
/// - `open`, the same line on a base the class table lacks: whether it is
///   a weapon is not established, so the plain total and the spells' are
///   undecided, and the attacks', whose row asks nothing of the item, is
///   5 to 10 (J30);
/// - `scoped`, `Adds 5 to 10 Fire Damage to Attacks` on such a base: no
///   row that reads the category names a line of it, so nothing is left
///   open — the attacks' total answers and the plain is lacked;
/// - `bare`, such a base with a life line alone: every total lacked.
fn sides() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("s1", "Sides")]), 10);
    let fire = json!({ "explicitMods": ["Adds 5 to 10 Fire Damage"] });
    let lacks = "A Base The Class Table Lacks";
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "s1",
        "Sides",
        vec![
            rare("ring", "Ruby Ring", fire.clone()),
            rare("wand", "Driftwood Wand", fire.clone()),
            rare("open", lacks, fire),
            rare(
                "scoped",
                lacks,
                json!({ "explicitMods": ["Adds 5 to 10 Fire Damage to Attacks"] }),
            ),
            rare(
                "bare",
                lacks,
                json!({ "explicitMods": ["+30 to maximum Life"] }),
            ),
        ],
        20,
    );
    s
}

/// C93, J30, over a row that reads the item's category: an item whose
/// category is not established is undecided under a total one of whose
/// rows reads it and names a line of the item — by the category's own
/// reason — and under nothing else. A not of such a comparison is
/// undecided and never a match; an or of it with a term that holds is
/// true. A total none of whose rows reads the category is never left open
/// by it. Every term's four counts return by their routes what they
/// counted.
#[test]
fn c93_a_row_that_reads_the_category_under_not_and_or() {
    let s = sides();
    let found = |text: &str| ids(&asked(&s, text));

    let a = asked(&s, "pseudo.adds_fire>=5");
    assert_eq!(ids(&a), ["ring"]);
    // ring 7.5; wand, scoped and bare lack it; open is undecided
    assert_eq!(counts(&a["terms"][0]), [1, 0, 3, 1]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    assert_eq!(follow(&s, &a["terms"][0]["undecided"]), ["open"]);
    assert_eq!(
        follow(&s, &a["terms"][0]["lacked"]),
        ["bare", "scoped", "wand"]
    );
    // by category's own reason, and no other
    let why = a["total"]["undecided_reasons"].as_array().unwrap();
    assert_eq!(why.len(), 1);
    assert_eq!(why[0]["example"]["id"], "open");
    assert_eq!(why[0]["unread"], "the class: base not in the table");
    assert_eq!(
        why[0]["hint"],
        "a refresh will not help; a reference update may"
    );
    // which the category term on the same item gives
    let weapon = asked(&s, "id:open category=weapon");
    assert_eq!(
        weapon["total"]["undecided_reasons"][0]["unread"],
        why[0]["unread"]
    );
    assert_eq!(
        weapon["total"]["undecided_reasons"][0]["problem"],
        why[0]["problem"]
    );

    // a not of it: undecided, never a match — the three that lack the
    // total are the matches, the ring fails
    let a = asked(&s, "-pseudo.adds_fire>=5");
    assert_eq!(ids(&a), ["bare", "scoped", "wand"]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    let a = asked(&s, "-pseudo.adds_fire>=100");
    assert_eq!(ids(&a), ["bare", "ring", "scoped", "wand"]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    assert_eq!(found("-has:pseudo.adds_fire"), ["bare", "scoped", "wand"]);
    assert_eq!(found("undecided(pseudo.adds_fire)"), ["open"]);
    assert_eq!(
        found("undecided(pseudo.adds_fire_to_spells.high)"),
        ["open"]
    );
    assert_eq!(found("undecided(pseudo.adds_damage)"), ["open"]);

    // an or of it with a term that holds is true: the attacks' total
    // answers on the item whose side is open (J30)
    let a = asked(&s, "pseudo.adds_fire>=5 or pseudo.adds_fire_to_attacks>=5");
    assert_eq!(ids(&a), ["open", "ring", "scoped", "wand"]);
    assert_eq!(a["total"]["undecided"]["count"], 0);
    assert_eq!(counts(&a["terms"][1]), [4, 0, 1, 0]);
    // an and of it with one is undecided still, and with one that fails, false
    let a = asked(&s, "pseudo.adds_fire>=5 pseudo.adds_fire_to_attacks=7.5");
    assert_eq!(ids(&a), ["ring"]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    let a = asked(&s, "pseudo.adds_fire>=5 pseudo.adds_fire_to_attacks>=100");
    assert_eq!(a["total"]["matched"], 0);
    assert_eq!(a["total"]["undecided"]["count"], 0);

    // a total none of whose rows reads the category is never left open by
    // it; nor is one whose row that reads it names no line of the item
    assert_eq!(
        found("undecided(pseudo.adds_fire_to_attacks)"),
        [] as [&str; 0]
    );
    assert_eq!(
        found("pseudo.adds_fire_to_attacks.low=5 pseudo.adds_damage_to_attacks.high=10"),
        ["open", "ring", "scoped", "wand"]
    );
    assert_eq!(found("id:scoped -has:pseudo.adds_fire"), ["scoped"]);
    assert_eq!(found("id:bare -has:pseudo.adds_damage_to_spells"), ["bare"]);

    // sorted and summed, the open one is what was readable of it: nothing
    let corpus = load(&s, Some("pc"));
    let by = sorted(&corpus, "pseudo.adds_fire");
    assert_eq!(by["ring"], json!({ "value": 7.5 }));
    assert_eq!(by["open"], json!({ "value": 0, "status": "incomplete" }));
    assert_eq!(by["wand"], json!({ "status": "no satisfying occurrence" }));
    assert_eq!(
        viewed(
            &corpus,
            "",
            json!({ "counts": { "keys": ["rarity"], "sum": "pseudo.adds_fire" } }),
        )["view"]["counts"]["sum"],
        json!({ "name": "pseudo.adds_fire", "value": 7.5, "lacking": 3, "incomplete": true, "unread": 1 })
    );
}

// ---- the help -----------------------------------------------------------------------------------

/// C97, C94: `--describe` of a ranged total prints its rows, which of
/// them read what the item is, the site's id and text, the limit and
/// J29's rule with the two arrays; the table's own line carries its
/// version and the rule; every total's definition prints the rule, the
/// totals shipped before the family among them.
#[test]
fn c97_describe_of_a_ranged_total_says_its_rows_the_limit_and_what_no_total_reads() {
    let all = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let computed = all["computed"].as_array().unwrap();
    let ranged: Vec<&Value> = computed
        .iter()
        .filter(|c| c["kind"] == "ranged total")
        .collect();
    assert_eq!(ranged.len(), 21);
    for total in &ranged {
        let name = total["name"].as_str().unwrap();
        let what = total["what"].as_str().unwrap();
        assert!(name.starts_with("pseudo.adds_"), "{name}");
        // the limit, with what the site showed
        assert!(what.contains("i06–i08"), "{name}: {what}");
    }
    // J29's rule and the two arrays, in every total's definition
    for total in computed
        .iter()
        .filter(|c| c["kind"] == "total" || c["kind"] == "ranged total")
    {
        let what = total["what"].as_str().unwrap();
        assert!(
            what.contains("`crucibleMods`") && what.contains("`utilityMods`"),
            "{}: {what}",
            total["name"]
        );
    }
    let table = all["totals"].as_str().unwrap();
    assert!(
        table.starts_with("totals v6, 82 totals and 4 readings"),
        "{table}"
    );
    assert!(
        table.contains("`crucibleMods`") && table.contains("`utilityMods`"),
        "{table}"
    );

    let one =
        serde_json::to_value(describe(&["adds_fire_to_spells".to_string()]).unwrap()).unwrap();
    let spells = &one["computed"][0];
    assert_eq!(spells["name"], "pseudo.adds_fire_to_spells");
    let what = spells["what"].as_str().unwrap();
    assert!(
        what.starts_with(
            "the trade site's pseudo_adds_fire_damage_to_spells (`Adds # to # Fire Damage to Spells`): "
        ),
        "{what}"
    );
    // its five rows, and the one of them that reads what the item is
    for row in [
        "line(\"Adds # to # Fire Damage to Spells\")",
        "line(\"Adds # to # Fire Damage to Spells and Attacks\")",
        "line(\"While a Unique Enemy is in your Presence, Adds # to # Fire Damage to Spells\")",
        "line(\"While a Pinnacle Atlas Boss is in your Presence, Adds # to # Fire Damage to Spells\")",
        "line(\"Adds # to # Fire Damage\")",
    ] {
        assert!(
            what.contains(&format!("1 × low and high of {row}")),
            "{row}: {what}"
        );
    }
    assert_eq!(what.matches("category=weapon").count(), 1, "{what}");
    // every example it prints is a query this build runs (rule 5)
    let s = fire();
    for example in spells["examples"].as_array().unwrap() {
        ask(&load(&s, Some("pc")), example.as_str().unwrap())
            .unwrap_or_else(|e| panic!("{example}: {e}"));
    }
    // the attacks' total reads the category by no row (J30)
    let attacks =
        serde_json::to_value(describe(&["adds_fire_to_attacks".to_string()]).unwrap()).unwrap();
    assert!(
        !attacks["computed"][0]["what"]
            .as_str()
            .unwrap()
            .contains("category=weapon")
    );
}

/// C94: a ranged total takes `low`, `high` and `avg`, and the bare name
/// is its `avg`; any other slot word — a line's `arg1` — is an authoring
/// error that offers the three, never read as one of them. A name
/// nothing defines is an unknown name with or without a slot word: the
/// refusal of a ranged total left with the build of them.
#[test]
fn c94_a_ranged_total_takes_low_high_and_avg_and_no_other_slot_word() {
    let s = fire();
    let corpus = load(&s, Some("pc"));
    for text in [
        "pseudo.adds_fire.arg1>=1",
        "pseudo.adds_fire.arg2>=1",
        "undecided(pseudo.adds_fire.arg1)",
    ] {
        let e = ask(&corpus, text).unwrap_err().to_json();
        assert_eq!(e["kind"], "slot_unknown", "`{text}`: {e}");
        assert_eq!(
            e["readings"],
            json!([
                "pseudo.adds_fire.avg",
                "pseudo.adds_fire.low",
                "pseudo.adds_fire.high"
            ]),
            "`{text}`"
        );
    }
    let request: Request = serde_json::from_value(
        json!({ "query": { "text": "" }, "view": { "rows": { "sort": "pseudo.adds_fire.arg1" } } }),
    )
    .unwrap();
    assert_eq!(
        answer(&corpus, &request).unwrap_err().to_json()["kind"],
        "slot_unknown"
    );
    for text in [
        "pseudo.adds_fyre>=1",
        "pseudo.adds_fyre.avg>=1",
        "has:pseudo.adds_fyre",
    ] {
        let e = ask(&corpus, text).unwrap_err().to_json();
        assert_eq!(e["kind"], "unknown_name", "`{text}`: {e}");
        assert!(
            e["readings"][0]
                .as_str()
                .unwrap()
                .ends_with("pseudo.adds_fire"),
            "`{text}`: {e}"
        );
    }
    // a name is known in any case (B1); a slot word is spelled as a
    // line's is
    assert_eq!(
        ids(&as_json(
            &ask(&corpus, "PSEUDO.Adds_Fire.avg=15.5").unwrap()
        )),
        ["ring"]
    );
}

/// J31, worked by hand from the table's `r-nothing-zero` and
/// `r-nothing-cancels`: a ranged total is nothing, and so lacked, only
/// where its low and its high both are. `zero`, a ring with `Adds 0 to 0
/// Fire Damage`, lacks the total; `cancels`, a ring with `Adds -5 to 5
/// Fire Damage`, has it — low -5, high 5 and an `avg` of 0, which is a
/// value and never nothing.
#[test]
fn j31_a_ranged_total_is_nothing_only_where_its_low_and_its_high_both_are() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("z1", "Zero")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "z1",
        "Zero",
        vec![
            rare(
                "zero",
                "Ruby Ring",
                json!({ "explicitMods": ["Adds 0 to 0 Fire Damage"] }),
            ),
            rare(
                "cancels",
                "Ruby Ring",
                json!({ "explicitMods": ["Adds -5 to 5 Fire Damage"] }),
            ),
        ],
        20,
    );
    let found = |text: &str| ids(&asked(&s, text));
    assert_eq!(found("has:pseudo.adds_fire"), ["cancels"]);
    assert_eq!(found("-has:pseudo.adds_fire"), ["zero"]);
    assert_eq!(found("pseudo.adds_fire=0"), ["cancels"]);
    assert_eq!(
        found("pseudo.adds_fire.low=-5 pseudo.adds_fire.high=5 pseudo.adds_fire.avg=0"),
        ["cancels"]
    );
    // a comparison on a total the item lacks is false, never against zero
    let a = asked(&s, "pseudo.adds_fire<=0");
    assert_eq!(ids(&a), ["cancels"]);
    assert_eq!(counts(&a["terms"][0]), [1, 0, 1, 0]);
    let corpus = load(&s, Some("pc"));
    let by = sorted(&corpus, "pseudo.adds_fire");
    assert_eq!(by["cancels"], json!({ "value": 0 }));
    assert_eq!(by["zero"], json!({ "status": "no satisfying occurrence" }));
}

/// C93 and the plan's rule 8, at a row that reads the category: a row
/// that does not count on the item reads nothing of it, so what could not
/// be read of the item leaves that row's total open by nothing. `blind`
/// is a Driftwood Wand whose `explicitMods` is no array: its plain fire
/// total has one row, which counts no line on a weapon whatever the array
/// holds, and is lacked; its attacks' total, whose rows count on every
/// item, is undecided by the array. `dim`, a Ruby Ring with the same
/// array, is undecided under both.
#[test]
fn c93_a_row_that_does_not_count_on_the_item_reads_nothing_of_it() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("b1", "Blind")]), 10);
    let unread = json!({ "explicitMods": "unreadable" });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "b1",
        "Blind",
        vec![
            rare("blind", "Driftwood Wand", unread.clone()),
            rare("dim", "Ruby Ring", unread),
        ],
        20,
    );
    let found = |text: &str| ids(&asked(&s, text));
    assert_eq!(found("-has:pseudo.adds_fire"), ["blind"]);
    assert_eq!(found("undecided(pseudo.adds_fire)"), ["dim"]);
    assert_eq!(
        found("undecided(pseudo.adds_fire_to_attacks)"),
        ["blind", "dim"]
    );
    // the spells' total holds rows that count on a weapon too
    assert_eq!(
        found("undecided(pseudo.adds_fire_to_spells)"),
        ["blind", "dim"]
    );
    let a = asked(&s, "pseudo.adds_fire>=1");
    assert_eq!(counts(&a["terms"][0]), [0, 0, 1, 1]);
    let why = a["total"]["undecided_reasons"].as_array().unwrap();
    assert_eq!(why.len(), 1);
    assert_eq!(
        (&why[0]["example"]["id"], &why[0]["unread"]),
        (&json!("dim"), &json!("explicit lines"))
    );
}

/// C97, T5, J29: the vocabulary lists a computed value beside the
/// templates where a narrowing matches its name or what the total itself
/// is defined as — its rows, its limit — and never by the rule the table
/// states of every total, which every definition prints: narrowed by a
/// word of that rule alone, `crucible` or `utility`, it lists no computed
/// value, where it would list every one (the fault T5 was ruled on, come
/// back by a sentence).
#[test]
fn c97_a_narrowing_matches_a_totals_own_definition_and_never_the_tables_rule() {
    let s = arrays();
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
    for word in [
        "crucible",
        "utility",
        "crucibleMods",
        "flask's own",
        "leaves out",
    ] {
        assert_eq!(computed(word), [] as [&str; 0], "line:{word}");
    }
    // a total's own definition still finds it: its rows, and a ranged
    // total's limit
    assert_eq!(
        computed("movement speed"),
        ["pseudo.increased_movement_speed"]
    );
    let fire = computed("Adds # to # Fire Damage to Spells");
    assert!(
        fire.contains(&"pseudo.adds_fire_to_spells".to_string()),
        "{fire:?}"
    );
    assert!(!fire.contains(&"pseudo.adds_fire".to_string()), "{fire:?}");
    assert_eq!(computed("i06").len(), 21);
}

/// C94, J7: what a row shows of a total is the lines of the rows that
/// count on the item, and no line of a row that does not. Two items with
/// the same two lines, `Adds 10 to 21 Fire Damage` then `Adds 1 to 3 Fire
/// Damage to Spells`, worked by hand under the spells' total: on `wand`, a
/// Driftwood Wand, the unsuffixed line is the weapon's own and the row
/// that names it does not count, so the total is the spells' line alone —
/// low 1, high 3, avg (1 + 3) / 2 = 2 — and that line alone is shown; on
/// `ring`, a Ruby Ring, both rows count — low 10 + 1 = 11, high 21 + 3 =
/// 24, avg 17.5 — and both lines are shown, in the item's order.
#[test]
fn c94_a_total_shows_only_the_lines_of_the_rows_that_count_on_the_item() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("e1", "Shown")]), 10);
    let lines = json!({ "explicitMods":
        ["Adds 10 to 21 Fire Damage", "Adds 1 to 3 Fire Damage to Spells"] });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "e1",
        "Shown",
        vec![
            rare("wand", "Driftwood Wand", lines.clone()),
            rare("ring", "Ruby Ring", lines),
        ],
        20,
    );
    let line = |text: &str| json!({ "line": { "source": "explicit", "flags": [], "text": text } });
    let value = |n: Value| json!({ "value": { "name": "pseudo.adds_fire_to_spells", "value": n } });
    let a = asked(&s, "pseudo.adds_fire_to_spells=2");
    assert_eq!(ids(&a), ["wand"]);
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"],
        json!([value(json!(2)), line("Adds 1 to 3 Fire Damage to Spells")])
    );
    let a = asked(&s, "pseudo.adds_fire_to_spells=17.5");
    assert_eq!(ids(&a), ["ring"]);
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"],
        json!([
            value(json!(17.5)),
            line("Adds 10 to 21 Fire Damage"),
            line("Adds 1 to 3 Fire Damage to Spells")
        ])
    );
}

/// C93 and the plan's rule 8, where a row that reads the category meets a
/// number it could not read: unknown is said once, at the row, and of no
/// more than was lost. Three items with one line, `Adds 5 to
/// 100000000000000000000 Fire Damage`, whose high has more digits than
/// the search reads; the reasons worked from the rules before running.
///
/// - `open`, a base the class table lacks. The plain total has one row,
///   which reads the category: whether it counts the line is not
///   established, so the row's contribution is unread and the total is
///   undecided — by the category's reason alone. The row was never summed,
///   so no number of the line was asked: on a weapon the row reads nothing
///   of the item (`ruler` below), and the number is lost to this total
///   only if the class is one that counts it. One thing was lost, the
///   row's contribution, and it is said once. The attacks' total, whose
///   row asks nothing of the item, sums the line and is undecided by the
///   number alone.
/// - `ring`, a Ruby Ring: the row counts, and both totals are undecided by
///   the number alone.
/// - `ruler`, a Driftwood Wand: the row does not count and reads nothing,
///   so the plain total is lacked; the attacks' is undecided by the number.
#[test]
fn c93_a_row_left_open_by_the_category_says_no_reason_of_a_number_it_never_read() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("o1", "Once")]), 10);
    let long = json!({ "explicitMods": ["Adds 5 to 100000000000000000000 Fire Damage"] });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "o1",
        "Once",
        vec![
            rare("open", "A Base The Class Table Lacks", long.clone()),
            rare("ring", "Ruby Ring", long.clone()),
            rare("ruler", "Driftwood Wand", long),
        ],
        20,
    );
    // why an item is undecided under `has:` of a total: every reason given
    let why = |id: &str, total: &str| -> Vec<String> {
        let a = asked(&s, &format!("id:{id} has:pseudo.{total}"));
        assert_eq!(a["total"]["undecided"]["count"], 1, "{id} {total}");
        a["total"]["undecided_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["unread"].as_str().unwrap().to_string())
            .collect()
    };
    const CLASS: &str = "the class: base not in the table";
    const NUMBER: &str = "the numbers of explicit lines";
    assert_eq!(why("open", "adds_fire"), [CLASS]);
    assert_eq!(why("open", "adds_fire_to_spells"), [CLASS]);
    assert_eq!(why("open", "adds_fire_to_attacks"), [NUMBER]);
    assert_eq!(why("ring", "adds_fire"), [NUMBER]);
    assert_eq!(why("ring", "adds_fire_to_attacks"), [NUMBER]);
    assert_eq!(why("ruler", "adds_fire_to_attacks"), [NUMBER]);
    // what each total answers is as it was: open under both on the two,
    // lacked on the weapon
    let found = |text: &str| ids(&asked(&s, text));
    assert_eq!(found("undecided(pseudo.adds_fire)"), ["open", "ring"]);
    assert_eq!(found("-has:pseudo.adds_fire"), ["ruler"]);
    assert_eq!(
        found("undecided(pseudo.adds_fire_to_attacks)"),
        ["open", "ring", "ruler"]
    );
}
