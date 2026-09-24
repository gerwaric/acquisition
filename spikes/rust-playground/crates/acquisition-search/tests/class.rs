//! The class table (`search/BUILD-PLAN.md`, step 6; C106, C93, C105),
//! through the crate's boundary: the shipped file, every reason the table
//! gives, `undecided(class)` and its route, the count's tally, `show`, and
//! `reqlevel` beside it. Every count is worked by hand from the fixture
//! below.

mod common;

use acquisition_search::{CLASS_TABLE_VERSION, ClassTable, Request, answer, class, show};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

/// Eight pc items in Standard and one in poe2.
///
/// `Gear` (g1): `ring` an Iron Ring; `nova`, a transfigured gem whose
/// base is its own name (`Ice Nova of Frostbolts`); `blade`, an Energy
/// Blade, which the table lists under three classes; `invite`, a Polaric
/// Invitation, under two; `beast`, an itemised beast no table lists;
/// `blank`, an item with no base at all; `odd`, whose `baseType` is a
/// number, so unread; `late`, a helmet whose `Level` requirement reads
/// `soon`. `Vault` (v1) in poe2: `p2ring`, an Iron Ring the table has no
/// realm for.
fn stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("g1", "Gear")]), 10);
    list_tabs(&mut s, "poe2", "Standard", json!([tab("v1", "Vault")]), 11);
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
        "g1",
        "Gear",
        vec![
            item("ring", "Doom Loop", "Iron Ring", "Rare", json!({})),
            plain("nova", "Ice Nova of Frostbolts", "Gem", json!({})),
            item("blade", "Storm Edge", "Energy Blade", "Rare", json!({})),
            plain("invite", "Polaric Invitation", "Normal", json!({})),
            item("beast", "Dune Hellion", "Dune Hellion", "Rare", json!({})),
            plain("blank", "", "Normal", json!({})),
            plain("odd", "Iron Ring", "Rare", json!({ "baseType": 7 })),
            item(
                "late",
                "Slow Crown",
                "Iron Hat",
                "Rare",
                json!({ "requirements": [{ "name": "Level", "values": [["soon", 0]], "displayMode": 0 }] }),
            ),
        ],
        20,
    );
    fetch_tab(
        &mut s,
        "poe2",
        "Standard",
        "v1",
        "Vault",
        vec![item("p2ring", "Far Loop", "Iron Ring", "Rare", json!({}))],
        21,
    );
    s
}

fn asked(s: &Store, realm: &str, text: &str) -> Value {
    as_json(&ask(&load(s, Some(realm)), text).unwrap())
}

fn term<'a>(a: &'a Value, path: &str) -> &'a Value {
    a["terms"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["path"] == path)
        .unwrap()
}

fn counts(a: &Value, path: &str) -> (u64, u64, u64, u64) {
    let t = term(a, path);
    let n = |k: &str| t[k]["count"].as_u64().unwrap();
    (n("matched"), n("failed"), n("lacked"), n("undecided"))
}

/// C106, C68: the shipped table loads, is the version every basis cites,
/// and holds what the generator measured — the classes, the rows, the
/// names under more than one class, the game's spellings.
#[test]
fn c106_the_shipped_table_is_what_the_generator_measured() {
    let t = class::table().unwrap();
    assert_eq!(t.version(), CLASS_TABLE_VERSION);
    assert_eq!(t.realms(), ["pc", "xbox", "sony"]);
    assert_eq!(t.names().len(), 82);
    assert_eq!(t.rows(), 4547);
    assert_eq!(t.shared().len(), 32);
    assert!(t.source().contains("e2bd511a"));
    assert_eq!(t.game(), "3.29.3.3");
    // the game's own names, plural, as the clipboard prints them
    for name in [
        "Rings",
        "Body Armours",
        "Thrusting One Hand Swords",
        "Staves",
        "Skill Gems",
    ] {
        assert!(class::names().contains(&name), "{name}");
    }
    assert_eq!(t.of("Iron Ring"), ["Rings"]);
    assert_eq!(t.of("Map (Tier 1)"), ["Maps"]);
    assert_eq!(t.of("Ice Nova of Frostbolts"), ["Skill Gems"]);
    assert_eq!(
        t.of("Energy Blade"),
        ["One Hand Swords", "Skill Gems", "Two Hand Swords"]
    );
    assert_eq!(t.of("Blighted Map (Tier 1)"), [] as [&str; 0]);
    // a base the export marks unreleased is one the game no longer drops,
    // and the owner holds one: it is in the table
    assert_eq!(t.of("Blade Trap"), ["Skill Gems"]);
    // the file on disk is the one the loader reads
    assert_eq!(
        ClassTable::parse(class::CLASS_TABLE_TOML).unwrap().rows(),
        t.rows()
    );
}

/// C93: every reason the table gives, each its own undecided with its
/// kind, and a class that is given is the item's one value of `class`.
#[test]
fn c93_what_the_table_cannot_class_is_undecided_with_its_reason() {
    let s = stash();
    let a = asked(&s, "pc", "class:ring");
    assert_eq!(ids(&a), ["ring"]);
    // ring matched; nova, late failed (a class, not this one); blade,
    // invite, beast, blank, odd undecided; nothing lacked, ever
    assert_eq!(counts(&a, "0"), (1, 2, 0, 5));
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"][0],
        json!({ "value": { "name": "class", "value": "Rings" } })
    );

    let open = asked(&s, "pc", "undecided(class)");
    assert_eq!(ids(&open), ["beast", "blade", "blank", "invite", "odd"]);
    let why = |id: &str| -> Value {
        open["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()["matched"][0]["shows"][0]
            .clone()
    };
    assert_eq!(
        why("beast")["undecided"]["unread"],
        "the class: base not in the table"
    );
    assert_eq!(
        why("beast")["undecided"]["problem"],
        "`Dune Hellion` is not in the class table (classes v1)"
    );
    assert_eq!(
        why("beast")["undecided"]["hint"],
        "a refresh will not help; a reference update may"
    );
    assert_eq!(
        why("blade")["undecided"]["unread"],
        "the class: base under several classes"
    );
    assert_eq!(
        why("blade")["undecided"]["problem"],
        "`Energy Blade` is in the class table under One Hand Swords and Skill Gems and Two Hand Swords (classes v1): the table never chooses"
    );
    assert_eq!(
        why("invite")["undecided"]["unread"],
        "the class: base under several classes"
    );
    assert_eq!(why("blank")["undecided"]["unread"], "the class: no base");
    // the base unread is the deriver's reason, said once
    assert_eq!(why("odd")["undecided"]["unread"], "`baseType`");
    assert_eq!(
        why("odd")["undecided"]["hint"],
        "a refresh may help; a body GGG really gives this way needs a build that reads it"
    );

    // a realm the table does not cover: undecided with that reason
    let p2 = asked(&s, "poe2", "class:ring");
    assert_eq!(p2["total"]["matched"], 0);
    assert_eq!(counts(&p2, "0"), (0, 0, 0, 1));
    let p2open = asked(&s, "poe2", "undecided(class)");
    assert_eq!(ids(&p2open), ["p2ring"]);
    assert_eq!(
        p2open["rows"][0]["matched"][0]["shows"][0]["undecided"]["problem"],
        "no class table for realm poe2 (classes v1 covers pc, xbox, sony)"
    );

    // known absence never occurs: every item has a class
    assert_eq!(asked(&s, "all", "-has:class")["total"]["matched"], 0);
    assert_eq!(asked(&s, "all", "has:class")["total"]["matched"], 3);
    // composition carries it (C93): true and undecided is undecided —
    // invite and blank, the two normal-frame items the table cannot
    // class — and false and undecided is false, so the rare ones are not
    let a = asked(&s, "pc", "class:ring frame=normal");
    assert_eq!(a["total"]["undecided"]["count"], 2);
    assert_eq!(a["total"]["matched"], 0);
}

/// The selector: `:` picks among the names, `=` names one in any case,
/// and a word no name holds is an authoring error with the near names —
/// never a match made for the author (S107).
#[test]
fn a_class_is_picked_among_the_games_names_and_never_guessed() {
    let s = stash();
    let corpus = load(&s, Some("pc"));
    assert_eq!(ids(&asked(&s, "pc", "class=rings")), ["ring"]);
    assert_eq!(ids(&asked(&s, "pc", "class:GEM")), ["nova"]);
    let swords = asked(&s, "pc", "class:sword");
    assert_eq!(swords["terms"][0]["resolved"]["values"], json!([]));
    assert_eq!(swords["total"]["matched"], 0);
    // the resolved list is of the matches, so a selector that picked
    // classes no item carries is listed in the zero block with nothing
    // to suggest (the fifth audit, 7)
    assert_eq!(
        swords["zero"]["resolved_to_nothing"],
        json!([{ "of": "class", "path": "0", "suggestions": [], "term": "class:sword" }])
    );
    // a word two names hold picks both; a misspelling offers the near one
    let staves = asked(&s, "pc", "class:stave");
    assert_eq!(staves["total"]["matched"], 0);
    assert_eq!(counts(&staves, "0"), (0, 3, 0, 5));
    let e = ask(&corpus, "class:stavs").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_value");
    assert_eq!(e["readings"], json!(["class=Staves"]));
    let e = ask(&corpus, "class=ring").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_value");
    assert_eq!(e["readings"][0], "class=Rings");
    // a bare word some class name holds offers the class (the reference)
    let e = ask(&corpus, "ring").unwrap_err().to_json();
    assert_eq!(e["kind"], "bare_word");
    assert_eq!(e["readings"][0], "class:ring");
    let e = ask(&corpus, "boots life").unwrap_err().to_json();
    assert_eq!(e["readings"][0], "class:boots");
}

/// C105: counted by class over every realm, each reason tallied beneath
/// `undecided`, the buckets summing to the total; and the class beside
/// another key in a crossed table.
#[test]
fn c105_a_count_by_class_tallies_every_reason_and_sums_to_the_total() {
    let s = stash();
    let request: Request = serde_json::from_value(json!({
        "scope": { "realm": "all" },
        "query": { "text": "" },
        "view": { "counts": { "keys": ["class"] } },
    }))
    .unwrap();
    let a = as_json(&answer(&load(&s, Some("all")), &request).unwrap());
    let table = &a["view"]["counts"]["tables"][0];
    let buckets: Vec<(String, u64)> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["value"]
                    .as_str()
                    .unwrap_or(b["bucket"].as_str().unwrap())
                    .to_string(),
                b["count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        buckets,
        [
            ("Helmets".to_string(), 1),
            ("Rings".to_string(), 1),
            ("Skill Gems".to_string(), 1),
            ("undecided".to_string(), 6),
        ]
    );
    let undecided = table["buckets"].as_array().unwrap().last().unwrap();
    assert_eq!(undecided["term"], "undecided(class)");
    assert_eq!(
        undecided["tally"],
        json!([
            { "unread": "`baseType`", "items": 1 },
            { "unread": "the class: base not in the table", "items": 1 },
            { "unread": "the class: base under several classes", "items": 2 },
            { "unread": "the class: no base", "items": 1 },
            { "unread": "the class: no table for the realm", "items": 1 },
        ])
    );
    assert_eq!(a["total"]["matched"], 9);
    // the route returns the six, over every realm
    assert_eq!(undecided["denominator"], "scope");
    let route: Request = serde_json::from_value(undecided["request"].clone()).unwrap();
    let followed = as_json(&answer(&load(&s, Some("all")), &route).unwrap());
    assert_eq!(followed["total"]["matched"], 6);
    // crossed with the realm's league: a margin per key
    let crossed: Request = serde_json::from_value(json!({
        "scope": { "realm": "all" },
        "query": { "text": "" },
        "view": { "cross": { "keys": ["class", "league"] } },
    }))
    .unwrap();
    let c = as_json(&answer(&load(&s, Some("all")), &crossed).unwrap());
    let margins = &c["view"]["cross"]["margins"];
    assert_eq!(margins[0]["key"], "class");
    assert_eq!(margins[0]["undecided"]["count"], 6);
    assert_eq!(margins[0]["none"]["count"], 0);
}

/// `show` prints what the table says of the item, with the basis naming
/// the table's version (C98, C106).
#[test]
fn c106_show_says_the_class_or_why_there_is_none_and_the_basis_cites_the_table() {
    let s = stash();
    let shown = |id: &str| serde_json::to_value(show(&s, id, false).unwrap()).unwrap();
    assert_eq!(shown("ring")["class"], json!({ "is": "Rings" }));
    assert_eq!(shown("nova")["class"], json!({ "is": "Skill Gems" }));
    let beast = shown("beast");
    assert_eq!(beast["class"]["open"]["part"], "class");
    assert_eq!(beast["class"]["open"]["of"], "not_in_table");
    assert_eq!(shown("odd")["class"], json!("base_unread"));
    assert_eq!(shown("p2ring")["class"]["open"]["of"], "no_table_for_realm");
    assert_eq!(shown("ring")["basis"]["classes"], CLASS_TABLE_VERSION);
    assert_eq!(shown("ring")["basis"], asked(&s, "pc", "")["basis"]);
    // the deriver's unread list stays the deriver's: the table's reason
    // is not in it
    assert_eq!(beast["item"]["unread"], json!([]));
}

/// `reqlevel` (step 6): the `Level` requirement as a number; absent where
/// the requirements were readable and hold none; unread, under its own
/// name only, where the value is no whole number.
#[test]
fn reqlevel_is_the_level_requirement_as_a_number() {
    let s = stash();
    let a = asked(&s, "pc", "reqlevel=..30");
    assert_eq!(a["total"]["matched"], 0);
    // seven lack it; `late` is undecided, its row still a displayed string
    assert_eq!(counts(&a, "0"), (0, 0, 7, 1));
    let open = asked(&s, "pc", "undecided(reqlevel)");
    assert_eq!(ids(&open), ["late"]);
    assert_eq!(
        open["rows"][0]["matched"][0]["shows"][0]["undecided"]["problem"],
        "`requirements`: `Level` is `soon`, not a whole number"
    );
    assert_eq!(ids(&asked(&s, "pc", r#""Requires Level soon""#)), ["late"]);
    assert_eq!(asked(&s, "pc", "-has:reqlevel")["total"]["matched"], 7);
    let shown = serde_json::to_value(show(&s, "late", false).unwrap()).unwrap();
    assert_eq!(shown["item"]["reqlevel"], Value::Null);
    assert_eq!(shown["item"]["unread"][0]["of"], "reqlevel");
}

/// The step-6 review's findings 1 and 2, reproduced first: a `Level` row
/// with no value, several values or a twin is no absence and no number —
/// unread under `reqlevel`; and a `Level` that was read is a witness to
/// its number whatever else in `requirements` could not be read, so a
/// comparison, a sort and a sum agree.
#[test]
fn review_a_malformed_level_row_is_unread_and_a_read_one_is_a_witness() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("r1", "Req")]), 10);
    let req = |rows: Value| json!({ "requirements": rows });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "r1",
        "Req",
        vec![
            item(
                "empty",
                "",
                "Iron Ring",
                "Rare",
                req(json!([{ "name": "Level", "values": [], "displayMode": 0 }])),
            ),
            item(
                "two",
                "",
                "Iron Ring",
                "Rare",
                req(
                    json!([{ "name": "Level", "values": [["10", 0], ["20", 0]], "displayMode": 0 }]),
                ),
            ),
            item(
                "twin",
                "",
                "Iron Ring",
                "Rare",
                req(
                    json!([{ "name": "Level", "values": [["10", 0]], "displayMode": 0 },
                                                             { "name": "Level", "values": [["40", 0]], "displayMode": 0 }]),
                ),
            ),
            item(
                "beside",
                "",
                "Iron Ring",
                "Rare",
                req(
                    json!([{ "name": "Level", "values": [["10", 0]], "displayMode": 0 },
                                                               { "name": "Str", "values": "no" }]),
                ),
            ),
            item(
                "plain",
                "",
                "Iron Ring",
                "Rare",
                req(json!([{ "name": "Level", "values": [["10", 0]], "displayMode": 0 }])),
            ),
        ],
        20,
    );
    // finding 1: none of the three malformed rows is an absence
    assert_eq!(asked(&s, "pc", "-has:reqlevel")["total"]["matched"], 0);
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(reqlevel)")),
        ["empty", "twin", "two"]
    );
    assert_eq!(ids(&asked(&s, "pc", "reqlevel=10")), ["beside", "plain"]);
    // finding 2: a read Level is a witness beside an unread sibling
    assert_eq!(ids(&asked(&s, "pc", "reqlevel>30")), [] as [&str; 0]);
    let over = asked(&s, "pc", "reqlevel>30");
    assert_eq!(counts(&over, "0"), (0, 2, 0, 3));
    assert_eq!(asked(&s, "pc", "has:reqlevel")["total"]["matched"], 2);
    let summed: Request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "reqlevel=10" },
        "view": { "counts": { "keys": ["tab"], "sum": "reqlevel" } },
    }))
    .unwrap();
    let c = as_json(&answer(&load(&s, Some("pc")), &summed).unwrap());
    assert_eq!(
        c["view"]["counts"]["sum"],
        json!({ "name": "reqlevel", "value": 20, "lacking": 0 })
    );
    // the phrase over the row is still a hit: the row was displayed —
    // `Requires Level 10, 20`, `Requires Level 10, Level 40` among them
    assert_eq!(
        asked(&s, "pc", r#""Requires Level 10""#)["total"]["matched"],
        4
    );
}

/// The review's finding 3: every reading a class error offers is a
/// query that binds — a name with a space is quoted by the printer.
#[test]
fn review_every_reading_a_class_error_offers_binds() {
    let s = stash();
    let corpus = load(&s, Some("pc"));
    for text in ["class:bodyarmour", "class=body", "class:nosuchclass"] {
        let e = ask(&corpus, text).unwrap_err().to_json();
        assert_eq!(e["kind"], "unknown_value", "{text}");
        for reading in e["readings"].as_array().unwrap() {
            let reading = reading.as_str().unwrap();
            assert!(
                ask(&corpus, reading).is_ok(),
                "`{text}` offers `{reading}`, which does not bind"
            );
        }
    }
    let e = ask(&corpus, "class:bodyarmour").unwrap_err().to_json();
    assert_eq!(e["readings"][0], "class=\"Body Armours\"");
}

/// The second review's findings 1 and 2: one status for `reqlevel` — a
/// read `Level` beside an element that may be a second `Level` row is
/// not established, for a comparison, a sort and a sum alike — and a
/// `[Level]` row, which the reader shows as `Level`, is told as one
/// when it could not be read.
#[test]
fn review_two_reqlevel_has_one_status_and_markup_hides_no_level_row() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("r2", "Req")]), 10);
    let req = |rows: Value| json!({ "requirements": rows });
    let level = json!({ "name": "Level", "values": [["10", 0]], "displayMode": 0 });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "r2",
        "Req",
        vec![
            item("unnamed", "", "Iron Ring", "Rare", req(json!([level, 42]))),
            item(
                "twinbad",
                "",
                "Iron Ring",
                "Rare",
                req(json!([level, { "name": "Level", "values": "no" }])),
            ),
            item(
                "marked",
                "",
                "Iron Ring",
                "Rare",
                req(json!([{ "name": "[Level]", "values": false, "displayMode": 0 }])),
            ),
            item(
                "named",
                "",
                "Iron Ring",
                "Rare",
                req(json!([level, { "name": "Str", "values": "no" }])),
            ),
        ],
        20,
    );
    // finding 1: neither a match nor a failure nor a complete sum
    assert_eq!(asked(&s, "pc", "reqlevel=10")["total"]["matched"], 1);
    assert_eq!(ids(&asked(&s, "pc", "reqlevel=10")), ["named"]);
    assert_eq!(counts(&asked(&s, "pc", "reqlevel>30"), "0"), (0, 1, 0, 3));
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(reqlevel)")),
        ["marked", "twinbad", "unnamed"]
    );
    let summed: Request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "" },
        "view": { "counts": { "keys": ["tab"], "sum": "reqlevel" } },
    }))
    .unwrap();
    let c = as_json(&answer(&load(&s, Some("pc")), &summed).unwrap());
    assert_eq!(
        c["view"]["counts"]["sum"],
        json!({ "name": "reqlevel", "value": 10, "lacking": 0, "incomplete": true, "unread": 3 })
    );
    let sorted: Request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "" },
        "view": { "rows": { "sort": "reqlevel" } },
    }))
    .unwrap();
    let r = as_json(&answer(&load(&s, Some("pc")), &sorted).unwrap());
    assert_eq!(r["rows"][0]["id"], "named");
    assert!(r["rows"][1]["sort"]["status"].is_string());
    // finding 2: `[Level]` is a Level row, so its loss is said
    assert_eq!(asked(&s, "pc", "-has:reqlevel")["total"]["matched"], 0);
    let shown = serde_json::to_value(show(&s, "marked", false).unwrap()).unwrap();
    assert_eq!(shown["item"]["reqlevel"], Value::Null);
    assert!(
        shown["item"]["unread"]
            .as_array()
            .unwrap()
            .iter()
            .any(|u| u["of"] == "reqlevel")
    );
    // the review's third finding: the derivation moved
    assert_eq!(shown["basis"]["derivation"], 7);
}
