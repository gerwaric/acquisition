//! The acceptance set as tests (`search/BUILD-PLAN.md`, "The acceptance
//! set as tests"; `search/DIGEST.md`, "The acceptance set"): the rows
//! marked green at step 4, one test per question over a fixture store,
//! through the crate's boundary. The limits the build meets (C102) are
//! pinned beside them, by the wording printed. Every count is worked by
//! hand from the fixture below.

mod common;

use acquisition_search::{Request, describe, show};
use acquisition_store::{Endpoint, Store};
use common::*;
use serde_json::{Value, json};

/// Twelve pc items in Standard.
///
/// `Gear` (g1): `ash` the unique amulet; five rare rings — `ring_res`
/// (12 + 40 + 30 resistance, 95 life), `ring_str` (strength, a damage
/// implicit), `ring_frac` (a fractured chaos resistance of 35, mana),
/// `ring_none` (mana alone), `veil` (50 life and a veiled suffix) — and
/// `ring_magic`, a magic ring with 30 fire resistance.
/// `Crucible leftovers` (o1): `staff`, and `chaos`, a stack of 40.
/// The character `Mover` wears `belt` (32 implicit and 110 explicit life)
/// and `amu` (92 implicit life). `Maps` (m1) holds the substash `Tier 1`
/// (s1) with `map1`. `Unopened` (n1) was never fetched.
fn stash() -> Store {
    let mut s = store();
    let maps = json!({ "id": "m1", "name": "Maps", "type": "MapStash" });
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([
            tab("g1", "Gear"),
            tab("o1", "Crucible leftovers"),
            maps,
            tab("n1", "Unopened")
        ]),
        10,
    );
    let rare = |id: &str, name: &str, base: &str, more: Value| item(id, name, base, "Rare", more);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "g1",
        "Gear",
        vec![
            item(
                "ash",
                "Ashes of the Stars",
                "Onyx Amulet",
                "Unique",
                json!({
                "implicitMods": ["+16 to all Attributes"],
                "explicitMods": ["20% increased Reservation Efficiency of Skills"] }),
            ),
            rare(
                "ring_res",
                "Doom Loop",
                "Two-Stone Ring",
                json!({
                "implicitMods": ["+12% to Fire and Cold Resistances"],
                "explicitMods": ["+40% to Fire Resistance", "+30% to Cold Resistance", "+95 to maximum Life"] }),
            ),
            rare(
                "ring_str",
                "Rune Coil",
                "Iron Ring",
                json!({
                "implicitMods": ["Adds 1 to 4 Physical Damage to Attacks"],
                "explicitMods": ["+45 to Strength"] }),
            ),
            rare(
                "ring_frac",
                "Hex Band",
                "Amethyst Ring",
                json!({
                "explicitMods": [{ "description": "+35% to Chaos Resistance", "flags": { "fractured": true } },
                                 "+20 to maximum Mana"] }),
            ),
            rare(
                "ring_none",
                "Dull Turn",
                "Iron Ring",
                json!({ "explicitMods": ["+20 to maximum Mana"] }),
            ),
            item(
                "ring_magic",
                "",
                "Ruby Ring",
                "Magic",
                json!({
                "typeLine": "Seething Ruby Ring", "explicitMods": ["+30% to Fire Resistance"] }),
            ),
            rare(
                "veil",
                "Veil Knot",
                "Coral Ring",
                json!({
                "explicitMods": ["+50 to maximum Life"], "veiledMods": ["Suffix01"], "veiled": true }),
            ),
        ],
        20,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "o1",
        "Crucible leftovers",
        vec![
            rare(
                "staff",
                "Dread Spire",
                "Judgement Staff",
                json!({
                "explicitMods": ["+2 to Level of all Spell Skill Gems"],
                "crucibleMods": ["15% increased Spell Damage"] }),
            ),
            json!({ "id": "chaos", "name": "", "typeLine": "Chaos Orb", "baseType": "Chaos Orb",
                    "frameTypeId": "Currency", "identified": true, "ilvl": 0, "stackSize": 40, "x": 1, "y": 0 }),
        ],
        21,
    );
    let sub = |sub: Option<&str>| Endpoint::Stash {
        realm: "pc".into(),
        league: "Standard".into(),
        id: "m1".into(),
        sub: sub.map(str::to_string),
    };
    s.record(
        &sub(None),
        &json!({ "id": "m1" }),
        200,
        &json!({ "stash": { "id": "m1", "name": "Maps", "type": "MapStash", "items": [],
                            "children": [{ "id": "s1", "name": "Tier 1", "type": "MapStash" }] } }),
        22,
    )
    .unwrap();
    s.record(
        &sub(Some("s1")),
        &json!({ "id": "m1", "sub": "s1" }),
        200,
        &json!({ "stash": { "id": "s1", "name": "Tier 1", "type": "MapStash",
                            "items": [item("map1", "", "Beach Map", "Normal", json!({}))] } }),
        23,
    )
    .unwrap();
    list_characters(
        &mut s,
        "pc",
        json!([{ "id": "c1", "name": "Mover", "league": "Standard" }]),
        30,
    );
    fetch_character(
        &mut s,
        "pc",
        json!({ "id": "c1", "name": "Mover", "league": "Standard", "equipment": [
            rare("belt", "Grim Clasp", "Leather Belt", json!({
                "implicitMods": ["+32 to maximum Life"], "explicitMods": ["+110 to maximum Life"] })),
            rare("amu", "Pain Locket", "Jade Amulet", json!({ "implicitMods": ["+92 to maximum Life"] })),
        ] }),
        31,
    );
    s
}

fn asked(s: &Store, text: &str) -> Value {
    as_json(&ask(&load(s, Some("pc")), text).unwrap())
}

/// What a row shows of the lines the query touched, as displayed.
fn shown_lines(row: &Value) -> Vec<String> {
    row["matched"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|m| m["shows"].as_array().unwrap())
        .filter_map(|e| e["line"]["text"].as_str().map(str::to_string))
        .collect()
}

const OQ1: &str = "class:ring rarity=rare (line(template:resistance) or line(template:strength))";

/// OQ1 as worded (step 6: `class:ring` over the class table): the query
/// names the lines it wants to see, so the rows show them; a ring with
/// neither line must not appear; refined by a fractured line.
#[test]
fn oq1_a_rare_ring_for_resistances_or_attributes_with_its_lines_on_the_row() {
    let s = stash();
    let a = asked(&s, OQ1);
    assert_eq!(ids(&a), ["ring_frac", "ring_res", "ring_str"]);
    assert_eq!(a["scope"]["items"], 12);
    // step 4's spelling, by the base, finds the same
    assert_eq!(
        ids(&asked(
            &s,
            "base:ring rarity=rare (line(template:resistance) or line(template:strength))"
        )),
        ids(&a)
    );
    // `class:ring` resolved to the one class whose name holds the word,
    // shown beside the selector as authored (invariant 2); every ring of
    // the fixture is of it, whatever its rarity
    assert_eq!(a["terms"][0]["term"], "class:ring");
    assert_eq!(
        a["terms"][0]["resolved"]["values"],
        json!([{ "value": "Rings", "items": 6 }])
    );
    assert_eq!(a["terms"][0]["matched"]["count"], 6);
    let row = |id: &str| {
        a["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
            .clone()
    };
    assert_eq!(
        shown_lines(&row("ring_res")),
        // by source word, then as the body orders them
        [
            "+40% to Fire Resistance",
            "+30% to Cold Resistance",
            "+12% to Fire and Cold Resistances"
        ]
    );
    assert_eq!(shown_lines(&row("ring_str")), ["+45 to Strength"]);
    // what the two selectors resolved to, beside them as authored
    // over the whole scope, the magic ring's line among them (C93): the
    // most carried first, then by name
    assert_eq!(
        (&a["terms"][2]["path"], &a["terms"][2]["term"]),
        (&json!("2.0"), &json!("line(template:resistance)"))
    );
    assert_eq!(
        a["terms"][2]["resolved"]["values"],
        json!([
            { "value": "#% to Fire Resistance", "items": 2 },
            { "value": "#% to Chaos Resistance", "items": 1 },
            { "value": "#% to Cold Resistance", "items": 1 },
            { "value": "#% to Fire and Cold Resistances", "items": 1 },
        ])
    );
    let refined = asked(
        &s,
        "base:ring rarity=rare line(template:resistance is:fractured)",
    );
    assert_eq!(ids(&refined), ["ring_frac"]);
    assert_eq!(
        refined["rows"][0]["matched"][0]["shows"][0]["line"]["flags"],
        json!(["fractured"])
    );
}

/// OQ2: one item and its place, or a zero answer naming the scope; then
/// `show` for every line with its values. Nothing says legacy (C107).
#[test]
fn oq2_a_unique_by_name_its_place_and_every_line_through_show() {
    let s = stash();
    let a = asked(&s, r##"name="Ashes of the Stars""##);
    assert_eq!(ids(&a), ["ash"]);
    let place = &a["rows"][0]["place"];
    assert_eq!(
        (
            &place["kind"],
            &place["name"],
            &place["id"],
            &place["league"]
        ),
        (
            &json!("stash"),
            &json!("Gear"),
            &json!("g1"),
            &json!("Standard")
        )
    );

    let none = asked(&s, r##"name="Headhunter""##);
    assert_eq!(none["total"]["matched"], 0);
    assert_eq!(none["rows"], json!([]));
    let scope = &none["scope"];
    assert_eq!(
        (&scope["realm"], &scope["items"], &scope["never_fetched"]),
        (&json!("pc"), &json!(12), &json!(1))
    );
    assert!(none["zero"].is_object());

    let shown = serde_json::to_value(show(&s, "ash", false).unwrap()).unwrap();
    assert_eq!(shown["item"]["name"], "Ashes of the Stars");
    assert_eq!(shown["place"]["name"], "Gear");
    let lines: Vec<(&str, &str, &Value)> = shown["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["source"].as_str().unwrap(),
                l["template"].as_str().unwrap(),
                &l["slots"],
            )
        })
        .collect();
    assert_eq!(
        lines[0],
        (
            "explicit",
            "#% increased Reservation Efficiency of Skills",
            &json!([["arg1", 20]])
        )
    );
    assert_eq!(
        lines[1],
        ("implicit", "# to all Attributes", &json!([["arg1", 16]]))
    );
    let printed = format!(
        "{a} {none} {shown} {}",
        serde_json::to_value(describe(&[]).unwrap()).unwrap()
    );
    assert!(
        !printed.to_lowercase().contains("legacy is")
            || printed.contains("legacy is the user's knowledge")
    );
    assert!(!shown.to_string().to_lowercase().contains("legacy"));
}

/// OQ3: a mod, a base, a unique — each its own query. Socket colours are step 8.
#[test]
fn oq3_a_mod_a_base_a_unique_each_its_own_query() {
    let s = stash();
    assert_eq!(
        ids(&asked(&s, r##""#% increased Spell Damage""##)),
        ["staff"]
    );
    assert_eq!(
        ids(&asked(&s, r##"base="Iron Ring""##)),
        ["ring_none", "ring_str"]
    );
    assert_eq!(ids(&asked(&s, "rarity=unique name:ashes")), ["ash"]);
    assert_eq!(
        ask(&load(&s, Some("pc")), "sockets.red>=2")
            .unwrap_err()
            .to_json()["kind"],
        "not_built"
    );
}

/// OQ4 as worded (step 6): a staff by what is remembered of it — its
/// class, a phrase — item and tab; `--describe league` says place, never
/// origin (S177). The game's words are `Staves` and `Warstaves`, which
/// `class:staves` picks both of (a Judgement Staff is a warstaff), and a
/// word no class name holds is an authoring error offering the near one,
/// never a match made for the author (S107).
#[test]
fn oq4_a_staff_by_what_is_remembered_and_league_is_place_never_origin() {
    let s = stash();
    let a = asked(&s, r##"class:staves "spell skill""##);
    assert_eq!(ids(&a), ["staff"]);
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"][0],
        json!({ "value": { "name": "class", "value": "Warstaves" } })
    );
    let e = ask(&load(&s, Some("pc")), r##"class:staff "spell skill""##)
        .unwrap_err()
        .to_json();
    assert_eq!(e["kind"], "unknown_value");
    // `staff` is three edits from `Staves`, past what near names reach,
    // so the offer is the whole list (an observation for the seat)
    let readings = e["readings"].as_array().unwrap();
    assert!(readings.len() > 50 && readings.contains(&json!("class=Staves")));
    assert_eq!(ids(&asked(&s, r##"base:staff "spell skill""##)), ["staff"]);
    assert_eq!(a["rows"][0]["place"]["name"], "Crucible leftovers");
    assert_eq!(
        a["rows"][0]["matched"][1]["shows"][0],
        json!({ "shown": { "part": "explicit line", "text": "+2 to Level of all Spell Skill Gems" } })
    );
    assert_eq!(ids(&asked(&s, "tab:crucible")), ["chaos", "staff"]);
    let league = serde_json::to_value(describe(&["league".to_string()]).unwrap()).unwrap();
    let what = league["fields"][0]["what"].as_str().unwrap();
    assert!(
        what.contains("its place, never where it came from"),
        "{what}"
    );
    assert!(
        what.contains("league of origin is not a fact the search holds"),
        "{what}"
    );
}

/// OQ7: one line across every tab and character, item and place on each row.
#[test]
fn oq7_one_line_across_every_tab_and_character() {
    let s = stash();
    let a = asked(&s, r##""+# to maximum Life""##);
    assert_eq!(ids(&a), ["amu", "belt", "ring_res", "veil"]);
    let places: Vec<(&str, &str)> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["place"]["kind"].as_str().unwrap(),
                r["place"]["name"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        places,
        [
            ("character", "Mover"),
            ("character", "Mover"),
            ("stash", "Gear"),
            ("stash", "Gear")
        ]
    );
    assert_eq!(a["rows"][0]["place"]["container"], "equipment");
    // place is terms like any other (C96)
    assert_eq!(
        ids(&asked(&s, r##""+# to maximum Life" character:mover"##)),
        ["amu", "belt"]
    );
    assert_eq!(
        ids(&asked(&s, r##""+# to maximum Life" -has:character"##)),
        ["ring_res", "veil"]
    );
    // a substash's item is found by its own name and by its tab's
    assert_eq!(ids(&asked(&s, "tab:maps")), ["map1"]);
    assert_eq!(ids(&asked(&s, r##"tab="Tier 1""##)), ["map1"]);
    assert_eq!(
        asked(&s, "tab:maps")["rows"][0]["place"]["parent"],
        json!({ "id": "m1", "name": "Maps" })
    );
}

/// AQ2: the old query parenthesised and one new term. The fixture holds
/// OQ1 matches under 60 — `ring_frac` at 35, `ring_str` at an honest zero —
/// and the test fails unless they leave.
#[test]
fn aq2_a_refinement_is_the_old_query_parenthesised_and_a_new_term() {
    let s = stash();
    let a = asked(
        &s,
        &format!("({OQ1}) sum(line(template:resistance).arg1)>=60"),
    );
    assert_eq!(ids(&a), ["ring_res"]);
    // 12 + 40 + 30
    let sum = a["rows"][0]["matched"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["path"] == "1")
        .unwrap();
    assert_eq!(sum["shows"][0]["value"]["value"], 82);
    // a sum of nothing is zero: matched or failed, never lacked (C94, C95)
    assert_eq!(
        a["terms"].as_array().unwrap().last().unwrap()["lacked"]["count"],
        0
    );
}

/// AQ3: each row names what matched; a zero answer names the scope
/// searched and what its selectors resolved to. The why-not is step 10.
#[test]
fn aq3_a_row_names_what_matched_and_a_zero_answer_names_the_scope() {
    let s = stash();
    let a = asked(&s, r##"(name:doom or "reservation") ilvl>=80"##);
    let matched = |id: &str| {
        let row = a["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        row["matched"].clone()
    };
    // the header is on the row already; a phrase shows the string it hit
    assert_eq!(matched("ash")[0]["term"], "\"reservation\"");
    assert_eq!(
        matched("ash")[0]["shows"][0]["shown"]["text"],
        "20% increased Reservation Efficiency of Skills"
    );
    assert_eq!(
        matched("ring_res"),
        json!([{ "path": "1", "term": "ilvl>=80", "shows": [{ "value": { "name": "ilvl", "value": 84 } }] }])
    );

    // S107 (C102): a template nothing carries is never fuzzy-matched; the
    // templates sharing its words are offered, each with its exact term
    let zero = asked(&s, r##""+#% to Fire Resistances">=30"##);
    assert_eq!(zero["total"]["matched"], 0);
    let nothing = &zero["zero"]["resolved_to_nothing"][0];
    assert_eq!(
        (&nothing["path"], &nothing["of"]),
        (&json!("0"), &json!("template"))
    );
    assert_eq!(
        nothing["suggestions"][0],
        json!({ "value": "#% to Fire Resistance", "items": 2, "term": "line(\"#% to Fire Resistance\")" })
    );
    assert_eq!(
        zero["zero"]["said"],
        "a line the search cannot name is shown as unknown, with what it knows of it, and never fuzzy-matched"
    );
    // rule 5: what the reference offers here and this build refuses is named, never printed as a command
    assert_eq!(zero["zero"]["not_built"], json!(["--explain", "--context"]));
    // a selector something carries is not listed, though the query matched nothing
    let other = asked(&s, "base:ring rarity=unique");
    assert_eq!(other["zero"]["resolved_to_nothing"], json!([]));
    let named = asked(&s, r##"base="Iron Rings""##);
    assert_eq!(
        named["zero"]["resolved_to_nothing"][0]["suggestions"][0]["term"],
        "base=\"Iron Ring\""
    );
}

/// AQ4: every id an answer printed is accepted back (S198) — by `id:`, the
/// item's and the location's, and an item's by `show`.
#[test]
fn aq4_every_printed_id_is_accepted_back() {
    let s = stash();
    let corpus = load(&s, Some("pc"));
    let a = as_json(&ask(&corpus, "").unwrap());
    assert_eq!(a["total"]["matched"], 12);
    for row in a["rows"].as_array().unwrap() {
        let id = row["id"].as_str().unwrap();
        assert_eq!(ids(&asked(&s, &format!("id:{id}"))), [id]);
        assert_eq!(
            serde_json::to_value(show(&s, id, false).unwrap()).unwrap()["item"]["facts"]["id"],
            id
        );
    }
    assert_eq!(asked(&s, "id:g1")["total"]["matched"], 7);
    assert_eq!(ids(&asked(&s, "id:c1")), ["amu", "belt"]);
    assert_eq!(ids(&asked(&s, "id:m1")), ["map1"]);
    // an id that is no item says what it is, and one the store lacks says so
    assert_eq!(
        show(&s, "g1", false).unwrap_err().to_json()["kind"],
        "not_an_item"
    );
    assert_eq!(
        show(&s, "nope", false).unwrap_err().to_json()["kind"],
        "item_not_found"
    );
}

/// AQ5: one whole-corpus mod query with a value, a count and a sorted
/// list, across every mod array.
#[test]
fn aq5_life_at_ninety_or_more_counted_and_sorted_across_every_mod_array() {
    let s = stash();
    let request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "\"+# to maximum Life\">=90" },
        "view": { "rows": { "sort": "line(\"+# to maximum Life\").arg1", "desc": true, "limit": 10 } },
    }))
    .unwrap();
    let a = as_json(&acquisition_search::answer(&load(&s, Some("pc")), &request).unwrap());
    assert_eq!(a["total"]["matched"], 3);
    let rows: Vec<(&str, &Value, &str)> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap(),
                &r["sort"]["value"],
                r["matched"][0]["shows"][0]["line"]["source"]
                    .as_str()
                    .unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("belt", &json!(110), "explicit"),
            ("ring_res", &json!(95), "explicit"),
            ("amu", &json!(92), "implicit")
        ]
    );
    // veil's 50 failed; the eight others carry no such line
    let term = &a["terms"][0];
    assert_eq!(
        (
            &term["matched"]["count"],
            &term["failed"]["count"],
            &term["lacked"]["count"]
        ),
        (&json!(3), &json!(1), &json!(8))
    );
    assert_eq!(term["together"]["count"], 0);
}

/// S12 (C102): a veiled line is shown as the placeholder it is, and a
/// value query never matches it.
#[test]
fn c102_s12_a_veiled_line_is_shown_as_its_placeholder_and_no_value_matches_it() {
    let s = stash();
    assert_eq!(ids(&asked(&s, "line(source=veiled)")), ["veil"]);
    assert_eq!(ids(&asked(&s, "is:veiled")), ["veil"]);
    for value in [
        "line(source=veiled arg1>=0)",
        "line(source=veiled arg1<=100)",
        "sum(line(source=veiled).arg1)>=1",
    ] {
        assert_eq!(asked(&s, value)["total"]["matched"], 0, "{value}");
    }
    let shown = serde_json::to_value(show(&s, "veil", false).unwrap()).unwrap();
    let veiled = shown["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["source"] == "veiled")
        .unwrap();
    assert_eq!(
        (&veiled["text"], &veiled["template"], &veiled["slots"]),
        (&json!("Suffix01"), &json!("Suffix#"), &json!([]))
    );
    assert_eq!(
        veiled["limit"],
        "a veiled line is shown as the placeholder it is; a value query never matches it"
    );
}

/// S52 (C102): the line's description says what a line is matched as.
#[test]
fn c102_s52_the_description_says_which_mods_made_a_line_is_never_guessed() {
    let line = serde_json::to_value(describe(&["template".to_string()]).unwrap()).unwrap();
    assert!(line["line"][0]["what"].as_str().unwrap().contains(
        "a line is matched as displayed text and value; which mods made it is not known and never guessed"
    ));
}

/// The reference's `acq show`: the item as the deriver sees it, what was
/// unread, and the stored body on request — which is still there when part
/// of the item could not be derived.
#[test]
fn c100_show_is_the_derived_item_and_the_stored_body_on_request() {
    let mut s = stash();
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "n1",
        "Unopened",
        vec![item(
            "odd",
            "Odd One",
            "Iron Ring",
            "Rare",
            json!({ "explicitMods": "not an array", "ilvl": "high" }),
        )],
        40,
    );
    let shown = serde_json::to_value(show(&s, "odd", true).unwrap()).unwrap();
    assert_eq!(shown["item"]["name"], "Odd One");
    assert_eq!(shown["lines"], json!([]));
    let unread: Vec<&str> = shown["item"]["unread"]
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["part"].as_str().unwrap())
        .collect();
    assert_eq!(unread, ["field", "lines"]);
    let body: Value = serde_json::from_str(shown["body"].as_str().unwrap()).unwrap();
    assert_eq!(body["explicitMods"], "not an array");
    assert!(
        serde_json::to_value(show(&s, "ash", false).unwrap())
            .unwrap()
            .get("body")
            .is_none()
    );
    // the basis is the answer's: the same snapshot names both
    assert_eq!(shown["basis"], asked(&s, "")["basis"]);
    // removed from its tab: in the store, and not live
    fetch_tab(&mut s, "pc", "Standard", "n1", "Unopened", vec![], 50);
    assert_eq!(
        show(&s, "odd", false).unwrap_err().to_json()["kind"],
        "item_not_live"
    );
}

/// OQ5, askable as the plan worded it (step 6): a level bracket across
/// tabs by class, with `reqlevel` and its absence. The owner's own
/// definition — wearable at a low level, with resistance, life or damage
/// lines — is with him (`SEARCH-SLICE.md`, "Holes ruled", step 6, G6),
/// and this test moves to his words when he rules; until then it pins
/// the constructs, never the coverage. The fixture holds the
/// distractors the bracket alone admits — a low-level gem, a flask, a
/// currency stack with no level requirement at all — and none appears;
/// a pair of boots whose requirements could not be read is undecided,
/// since an unread requirements array satisfies neither side. Whether
/// OQ5 is *covered* — a grouping above class — is the owner's (the plan,
/// "Parks whose triggers the build fires").
#[test]
fn oq5_leveling_gear_by_class_and_level_bracket_askable_as_worded() {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([tab("l1", "Leveling"), tab("l2", "Odds")]),
        10,
    );
    let requires = |level: &str| json!({ "requirements": [{ "name": "Level", "values": [[level, 0]], "displayMode": 0 }] });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "l1",
        "Leveling",
        vec![
            item(
                "boots",
                "Dusk Stride",
                "Iron Greaves",
                "Rare",
                requires("20"),
            ),
            item(
                "gloves",
                "Grim Grip",
                "Iron Gauntlets",
                "Rare",
                requires("45"),
            ),
            item("helm", "", "Iron Hat", "Normal", json!({})),
            json!({ "id": "gem", "name": "", "typeLine": "Fireball", "baseType": "Fireball",
                    "frameTypeId": "Gem", "identified": true, "ilvl": 0, "x": 0, "y": 0,
                    "requirements": [{ "name": "Level", "values": [["1", 0]], "displayMode": 0 }] }),
        ],
        20,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "l2",
        "Odds",
        vec![
            item("flask", "", "Small Life Flask", "Normal", requires("3")),
            json!({ "id": "chaos", "name": "", "typeLine": "Chaos Orb", "baseType": "Chaos Orb",
                    "frameTypeId": "Currency", "identified": true, "ilvl": 0, "stackSize": 7, "x": 1, "y": 0 }),
            item(
                "odd",
                "Odd Tread",
                "Iron Greaves",
                "Rare",
                json!({ "requirements": "Level 12" }),
            ),
        ],
        21,
    );
    let a = asked(
        &s,
        "(class:boots or class:gloves or class:helmet) (reqlevel=..30 or -has:reqlevel)",
    );
    assert_eq!(ids(&a), ["boots", "helm"]);
    assert_eq!(a["total"]["undecided"]["count"], 1);
    // the level required is a number; its absence is known only where
    // the requirements were readable
    let term = |path: &str| {
        a["terms"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["path"] == path)
            .unwrap()
            .clone()
    };
    assert_eq!(term("1.0")["term"], "reqlevel=..30");
    assert_eq!(
        (
            &term("1.0")["matched"]["count"],
            &term("1.0")["failed"]["count"],
            &term("1.0")["lacked"]["count"],
            &term("1.0")["undecided"]["count"]
        ),
        (&json!(3), &json!(1), &json!(2), &json!(1))
    );
    // the not's child is the atomic term; its absence is known on the two
    // items whose requirements were read and hold none
    assert_eq!(term("1.1.0")["term"], "has:reqlevel");
    assert_eq!(
        (
            &term("1.1.0")["matched"]["count"],
            &term("1.1.0")["lacked"]["count"],
            &term("1.1.0")["undecided"]["count"]
        ),
        (&json!(4), &json!(2), &json!(1))
    );
    // the same bracket counted by tab: a set of items across tabs
    let counted: Request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "(class:boots or class:gloves or class:helmet) (reqlevel=..30 or -has:reqlevel)" },
        "view": { "counts": { "keys": ["tab", "class"] } },
    }))
    .unwrap();
    let c = as_json(&acquisition_search::answer(&load(&s, Some("pc")), &counted).unwrap());
    let table = &c["view"]["counts"]["tables"][1];
    assert_eq!(table["key"], "class");
    assert_eq!(
        (&table["buckets"][0]["value"], &table["buckets"][0]["count"]),
        (&json!("Boots"), &json!(1))
    );
    assert_eq!(
        (&table["buckets"][1]["value"], &table["buckets"][1]["count"]),
        (&json!("Helmets"), &json!(1))
    );
    assert_eq!(c["view"]["counts"]["tables"][0]["key"], "tab");
    // `--sort reqlevel` orders the bracket, an item with none last
    let sorted: Request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "class:boots or class:gloves or class:helmet" },
        "view": { "rows": { "sort": "reqlevel", "desc": true } },
    }))
    .unwrap();
    let sorted = as_json(&acquisition_search::answer(&load(&s, Some("pc")), &sorted).unwrap());
    let order: Vec<&str> = sorted["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(order, ["gloves", "boots", "helm", "odd"]);
}
