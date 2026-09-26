//! The first seat's faults (`runs/seat-2026-09-26/`, 2026-09-26; the
//! record's findings and "Holes ruled", the seat row), each reproduced
//! here as a failing test before it was fixed at step 9b: the class
//! reading (V1, V2, F2), the undecided block's shape (V8, F9), a tab's
//! type as a field (V10), the place keys under an all-realms scope (F3),
//! and the answer's words — the help's limit (F1), a comparison after a
//! group (F4), a closed set's refusal (F5), a computed value's case
//! (F10), a partial id (F7), what a row sorts by (F6). The text these
//! render to is `acquisition-cli/tests/search_json.rs`.

mod common;

use std::collections::BTreeSet;

use acquisition_search::{Request, answer, describe, parse_query};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

fn plain(id: &str, base: &str, frame: &str, more: Value) -> Value {
    let mut body = json!({ "id": id, "name": "", "typeLine": base, "baseType": base,
        "frameTypeId": frame, "identified": true, "ilvl": 0, "x": 0, "y": 0 });
    for (key, value) in more.as_object().unwrap() {
        body[key] = value.clone();
    }
    body
}

fn asked(s: &Store, realm: &str, text: &str) -> Value {
    as_json(&ask(&load(s, Some(realm)), text).unwrap())
}

fn view(s: &Store, realm: &str, text: &str, view: Value) -> Value {
    let request: Request =
        serde_json::from_value(json!({ "query": { "text": text }, "view": view })).unwrap();
    as_json(&answer(&load(s, Some(realm)), &request).unwrap())
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

/// The ids a count's route returns, over the realm it names: as many as
/// it counted (invariant 4).
fn members(s: &Store, counted: &Value) -> BTreeSet<String> {
    let n = counted["count"].as_u64().unwrap();
    let Some(route) = counted.get("request") else {
        assert_eq!(n, 0, "a count with no route: {counted}");
        return BTreeSet::new();
    };
    let mut route = route.clone();
    route["view"]["rows"]["limit"] = json!(100);
    let realm = route["scope"]["realm"].as_str().unwrap().to_string();
    let request: Request = serde_json::from_value(route.clone()).unwrap();
    let routed = as_json(&answer(&load(s, Some(&realm)), &request).unwrap());
    let ids: BTreeSet<String> = ids(&routed).into_iter().collect();
    assert_eq!(ids.len() as u64, n, "{route}");
    ids
}

fn bucket<'a>(table: &'a Value, value: &str) -> Vec<&'a Value> {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["bucket"] == "value" && b["value"] == value)
        .collect()
}

/// One tab, `Gear`, of what the seat met: three blighted maps, two
/// invitations, an Energy Blade, two itemised beasts, a plain map and a
/// ring.
fn seat_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("g1", "Gear")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "g1",
        "Gear",
        vec![
            plain("b13", "Blighted Map (Tier 13)", "Normal", json!({})),
            plain("b16", "Blight-ravaged Map (Tier 16)", "Rare", json!({})),
            plain("m1", "Map (Tier 1)", "Normal", json!({})),
            plain("inv", "Polaric Invitation", "Normal", json!({})),
            plain("quest", "Incandescent Invitation", "Quest", json!({})),
            item("blade", "Storm Edge", "Energy Blade", "Rare", json!({})),
            item("beast", "Dune Hellion", "Dune Hellion", "Rare", json!({})),
            item("wolf", "Snow Wolf", "Snow Wolf", "Rare", json!({})),
            item("ring", "Doom Loop", "Iron Ring", "Rare", json!({})),
        ],
        20,
    );
    s
}

/// V1 (owner, 2026-09-26: "I approve your suggestion for blighted maps"):
/// a blighted or blight-ravaged map's class is read from its base past
/// the prefix the API adds, so it is Maps like the plain map — never
/// undecided as a base the table lacks.
#[test]
fn v1_a_blighted_maps_class_is_read_past_the_prefix_the_api_adds() {
    let s = seat_stash();
    assert_eq!(ids(&asked(&s, "pc", "class=Maps")), ["b13", "b16", "m1"]);
    // `class:map` picks Misc Map Items too, the normal invitation's (V2)
    assert_eq!(
        ids(&asked(&s, "pc", "class:map")),
        ["b13", "b16", "inv", "m1"]
    );
    let open = asked(&s, "pc", "undecided(class)");
    assert!(
        !ids(&open).iter().any(|id| id == "b13" || id == "b16"),
        "{:?}",
        ids(&open)
    );
    let shown = serde_json::to_value(common::show(&s, "b16", false).unwrap()).unwrap();
    assert_eq!(shown["class"], json!({ "is": "Maps" }));
    // what stays undecided under `class=Maps` is the two beasts, whose
    // bases the table lacks — never a blighted map
    assert_eq!(
        asked(&s, "pc", "class=Maps")["total"]["undecided"]["count"],
        2
    );
}

/// V2 (owner: "let's use frame to determine an invitation's class"): a
/// base the table lists under Misc Map Items and Quest Items is read by
/// its frame — the quest frame Quest Items, any other Misc Map Items.
#[test]
fn v2_an_invitations_class_is_its_frames() {
    let s = seat_stash();
    assert_eq!(ids(&asked(&s, "pc", "class=\"Misc Map Items\"")), ["inv"]);
    assert_eq!(ids(&asked(&s, "pc", "class=\"Quest Items\"")), ["quest"]);
    let open = asked(&s, "pc", "undecided(class)");
    assert!(
        !ids(&open).iter().any(|id| id == "inv" || id == "quest"),
        "{:?}",
        ids(&open)
    );
    let shown = serde_json::to_value(common::show(&s, "inv", false).unwrap()).unwrap();
    assert_eq!(shown["class"], json!({ "is": "Misc Map Items" }));
    let shown = serde_json::to_value(common::show(&s, "quest", false).unwrap()).unwrap();
    assert_eq!(shown["class"], json!({ "is": "Quest Items" }));
}

/// F2: `class:X` on an item whose base the table lists under several
/// classes is undecided only where a candidate satisfies `X` — the table
/// never chooses, and a term false under every candidate needs no choice
/// (C93: undecided is where the truth cannot be established).
#[test]
fn f2_a_class_term_false_under_every_candidate_class_is_false() {
    let s = seat_stash();
    // ring matched; blade failed, its three classes none of them Rings;
    // beast and wolf undecided, the table lacking their bases
    let a = asked(&s, "pc", "class:ring");
    assert_eq!(counts(&a, "0"), (1, 6, 0, 2), "{}", a["terms"][0]);
    assert_eq!(
        ids(&asked(&s, "pc", "-class:ring")),
        ["b13", "b16", "blade", "inv", "m1", "quest"]
    );
    // a candidate the term names keeps it open
    let swords = asked(&s, "pc", "class:sword");
    assert_eq!(counts(&swords, "0"), (0, 6, 0, 3), "{}", swords["terms"][0]);
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(class:sword)")),
        ["beast", "blade", "wolf"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(class:ring)")),
        ["beast", "wolf"]
    );
    // the class itself is still not established: `undecided(class)` lists
    // it, `has:class` is open, and a count by class keeps it undecided
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(class)")),
        ["beast", "blade", "wolf"]
    );
    assert_eq!(
        asked(&s, "pc", "has:class")["terms"][0]["undecided"]["count"],
        3
    );
    let table = &view(&s, "pc", "", json!({ "counts": { "keys": ["class"] } }))["view"]["counts"]["tables"]
        [0];
    let undecided = table["buckets"].as_array().unwrap().last().unwrap();
    assert_eq!(undecided["bucket"], "undecided");
    assert_eq!(undecided["count"], 3);
    // every route returns exactly what it counted (invariant 4): the
    // failed route admits the blade, whose class no `has:` establishes
    let a = asked(&s, "pc", "class:ring");
    assert_eq!(members(&s, &a["terms"][0]["failed"]).len(), 6);
    assert!(members(&s, &a["terms"][0]["failed"]).contains("blade"));
    assert_eq!(members(&s, &a["terms"][0]["undecided"]).len(), 2);
    assert_eq!(a["terms"][0]["lacked"]["count"], 0);
}

/// V8 (owner: "i like your proposal better. let's go with it as you've
/// suggested"): the answer shows its undecided items as one line per
/// distinct reason — by what was unread, the kind a count tallies (C105)
/// — each with one example item, then the one route; no item list, no
/// count per reason. Bounded, the rest counted (invariant 5). F9 was the
/// old shape: one reason repeated once per term.
#[test]
fn v8_the_undecided_block_is_one_line_per_distinct_reason_with_an_example() {
    let s = seat_stash();
    // three terms the blade is open under, and the beasts under every one:
    // two reasons, however many terms met them
    let a = asked(&s, "pc", "class:sword or class:gem or class:staves");
    assert_eq!(a["total"]["undecided"]["count"], 3);
    assert!(
        a["total"].get("undecided_items").is_none(),
        "{}",
        a["total"]
    );
    let reasons = a["total"]["undecided_reasons"].as_array().unwrap();
    // in the store's order: the beast is met before the blade, so the
    // beasts' reason is first and the beast its example (rule 9)
    assert_eq!(reasons.len(), 2, "{reasons:?}");
    assert_eq!(reasons[0]["unread"], "the class: base not in the table");
    assert_eq!(reasons[0]["example"]["id"], "beast");
    assert_eq!(reasons[0]["example"]["name"], "Dune Hellion Dune Hellion");
    assert!(
        reasons[0]["problem"]
            .as_str()
            .unwrap()
            .contains("`Dune Hellion` is not in the class table")
    );
    assert_eq!(
        reasons[0]["hint"],
        "a refresh will not help; a reference update may"
    );
    assert_eq!(
        reasons[1]["unread"],
        "the class: base under several classes"
    );
    assert_eq!(reasons[1]["example"]["id"], "blade");
    assert_eq!(
        reasons[1]["problem"],
        "`Energy Blade` is in the class table under One Hand Swords and Skill Gems and Two Hand Swords (classes v1): the table never chooses"
    );
    assert!(reasons[0].get("path").is_none() && reasons[0].get("term").is_none());
    assert!(a["total"].get("reasons_left_out").is_none());
    // the same two whichever term is written first (rule 9 of the plan)
    let b = asked(&s, "pc", "class:staves or class:gem or class:sword");
    assert_eq!(
        b["total"]["undecided_reasons"],
        a["total"]["undecided_reasons"]
    );
    // the one route returns every undecided item with its reasons
    let route: Request =
        serde_json::from_value(a["total"]["undecided"]["request"].clone()).unwrap();
    let routed = as_json(&answer(&load(&s, Some("pc")), &route).unwrap());
    assert_eq!(ids(&routed), ["beast", "blade", "wolf"]);
    // bounded: eleven flags unread are eleven reasons, ten listed and one
    // counted, in the item's order
    let flags = [
        "corrupted",
        "identified",
        "split",
        "duplicated",
        "replica",
        "fractured",
        "mutated",
        "synthesised",
        "veiled",
        "searing",
        "tangled",
    ];
    let mut body = json!({});
    for flag in flags {
        body[flag] = json!("unread");
    }
    let (corpus, _) = generated::fixture(vec![body]);
    let terms: Vec<String> = flags.iter().map(|f| format!("is:{f}")).collect();
    let a = generated::run(
        &corpus,
        &generated::request(&terms.join(" "), None, false, 10),
    )
    .unwrap();
    let reasons = a["total"]["undecided_reasons"].as_array().unwrap();
    assert_eq!(reasons.len(), 10, "{reasons:?}");
    assert_eq!(a["total"]["reasons_left_out"], 1);
    for reason in reasons {
        let unread = reason["unread"].as_str().unwrap();
        assert!(flags.iter().any(|f| unread == format!("`{f}`")), "{unread}");
        assert_eq!(reason["example"]["id"], "i0");
    }
}

/// V10 (owner: "let's add tab type in the next build session"): GGG's
/// `type` of the tab an item is in is the field `tab.type`, verbatim from
/// the store's read (C108), so everything in a map tab, whatever it is
/// called, can be asked for; a substash's is its tab's; an item on a
/// character lacks it.
#[test]
fn v10_a_tabs_type_is_a_field() {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([
            tab("d1", "Dump"),
            { "id": "m1", "name": "Atlas", "type": "MapStash" },
            { "id": "q1", "name": "Bulk", "type": "QuadStash" }
        ]),
        10,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "d1",
        "Dump",
        vec![item("r1", "Doom Loop", "Iron Ring", "Rare", json!({}))],
        20,
    );
    let sub = |sub: Option<&str>| acquisition_store::Endpoint::Stash {
        realm: "pc".into(),
        league: "Standard".into(),
        id: "m1".into(),
        sub: sub.map(str::to_string),
    };
    s.record(
        &sub(None),
        &json!({ "id": "m1" }),
        200,
        &json!({ "stash": { "id": "m1", "name": "Atlas", "type": "MapStash", "items": [],
                            "children": [{ "id": "s1", "name": "1", "type": "MapStash" }] } }),
        21,
    )
    .unwrap();
    s.record(
        &sub(Some("s1")),
        &json!({ "id": "m1", "sub": "s1" }),
        200,
        &json!({ "stash": { "id": "s1", "name": "1", "type": "MapStash",
                            "items": [item("map", "", "Beach Map", "Normal", json!({}))] } }),
        22,
    )
    .unwrap();
    s.record(
        &acquisition_store::Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "q1".into(),
            sub: None,
        },
        &json!({ "id": "q1" }),
        200,
        &json!({ "stash": { "id": "q1", "name": "Bulk", "type": "QuadStash",
                            "items": [plain("chaos", "Chaos Orb", "Currency", json!({ "stackSize": 3 }))] } }),
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
            item("worn", "Grim Clasp", "Leather Belt", "Rare", json!({})) ] }),
        31,
    );
    // the map is found by what its tab is, not what it is called
    assert_eq!(ids(&asked(&s, "pc", "tab.type=MapStash")), ["map"]);
    assert_eq!(ids(&asked(&s, "pc", "tab.type:map")), ["map"]);
    assert_eq!(ids(&asked(&s, "pc", "tab.type=quadstash")), ["chaos"]);
    assert_eq!(
        ids(&asked(&s, "pc", "tab.type:stash")),
        ["chaos", "map", "r1"]
    );
    // `tab:` is still the name, never the type
    assert_eq!(asked(&s, "pc", "tab:mapstash")["total"]["matched"], 0);
    assert_eq!(ids(&asked(&s, "pc", "tab:atlas")), ["map"]);
    // known absence on a character (C93), never undecided
    assert_eq!(ids(&asked(&s, "pc", "-has:tab.type")), ["worn"]);
    let a = asked(&s, "pc", "tab.type=MapStash");
    assert_eq!(counts(&a, "0"), (1, 2, 1, 0), "{}", a["terms"][0]);
    assert_eq!(a["rows"][0]["place"]["tab_type"], "MapStash");
    // a `:` says what it picked (invariant 2)
    let picked = asked(&s, "pc", "tab.type:stash");
    assert_eq!(
        picked["terms"][0]["resolved"]["values"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    // counted, `none` for the character's item, every bucket routed
    let table = &view(&s, "pc", "", json!({ "counts": { "keys": ["tab.type"] } }))["view"]["counts"]
        ["tables"][0];
    let shape: Vec<(String, u64)> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["value"].as_str().map_or_else(
                    || format!("({})", b["bucket"].as_str().unwrap()),
                    str::to_string,
                ),
                b["count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        shape,
        [
            ("MapStash".to_string(), 1),
            ("PremiumStash".to_string(), 1),
            ("QuadStash".to_string(), 1),
            ("(none)".to_string(), 1)
        ]
    );
    for b in table["buckets"].as_array().unwrap() {
        members(&s, b);
    }
    // the help lists it, under `tab` too, and `show` carries it
    let d = serde_json::to_value(describe(&["tab".to_string()]).unwrap()).unwrap();
    let names: Vec<&str> = d["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["tab", "tab.type"]);
    let shown = serde_json::to_value(common::show(&s, "map", false).unwrap()).unwrap();
    assert_eq!(shown["place"]["tab_type"], "MapStash");
    let shown = serde_json::to_value(common::show(&s, "worn", false).unwrap()).unwrap();
    assert!(shown["place"].get("tab_type").is_none());
    for example in [
        "tab.type=MapStash",
        "tab.type:map",
        "undecided(tab.type)",
        "has:tab.type",
    ] {
        parse_query(example).unwrap_or_else(|e| panic!("{example}: {e}"));
    }
}

/// F3: under `--realm all` a place value — a league, a character, a
/// container, a tab's type — is keyed by realm, as a tab and a vocabulary
/// row are (C96, C97): poe2's Standard is another league, and its bucket's
/// route is scoped to its realm. Under one realm nothing changes.
#[test]
fn f3_under_all_realms_a_place_value_is_keyed_by_realm() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("d1", "Dump")]), 10);
    list_tabs(&mut s, "pc", "Hardcore", json!([tab("d2", "Dump")]), 11);
    list_tabs(&mut s, "poe2", "Standard", json!([tab("p1", "Dump")]), 12);
    let ring = |id: &str| item(id, "Doom Loop", "Iron Ring", "Rare", json!({}));
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "d1",
        "Dump",
        vec![ring("a"), ring("b")],
        20,
    );
    fetch_tab(&mut s, "pc", "Hardcore", "d2", "Dump", vec![ring("c")], 21);
    fetch_tab(
        &mut s,
        "poe2",
        "Standard",
        "p1",
        "Dump",
        vec![ring("d")],
        22,
    );
    let a = view(
        &s,
        "all",
        "",
        json!({ "counts": { "keys": ["league", "tab.type"] } }),
    );
    let league = &a["view"]["counts"]["tables"][0];
    let rows: Vec<(String, String, u64)> = league["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["realm"].as_str().unwrap().to_string(),
                b["value"].as_str().unwrap().to_string(),
                b["count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("pc".to_string(), "Standard".to_string(), 2),
            ("pc".to_string(), "Hardcore".to_string(), 1),
            ("poe2".to_string(), "Standard".to_string(), 1)
        ]
    );
    let standard = bucket(league, "Standard");
    assert_eq!(standard[0]["request"]["scope"]["realm"], "pc");
    assert_eq!(standard[1]["request"]["scope"]["realm"], "poe2");
    assert_eq!(standard[0]["term"], "league=Standard");
    for b in league["buckets"].as_array().unwrap() {
        members(&s, b);
    }
    // the tab's type too: one PremiumStash bucket for each realm
    let types = &a["view"]["counts"]["tables"][1];
    assert_eq!(bucket(types, "PremiumStash").len(), 2);
    // a crossed table's cell carries the realm of its place key
    let c = view(
        &s,
        "all",
        "",
        json!({ "cross": { "keys": ["league", "rarity"] } }),
    );
    let cells = c["view"]["cross"]["cells"].as_array().unwrap();
    assert_eq!(cells.len(), 3, "{cells:?}");
    for cell in cells {
        assert!(cell["of"][0]["realm"].is_string(), "{cell}");
        members(&s, cell);
    }
    // under one realm the key is the value alone, routed over the scope
    let one = view(&s, "pc", "", json!({ "counts": { "keys": ["league"] } }));
    let league = &one["view"]["counts"]["tables"][0];
    assert!(league["buckets"][0].get("realm").is_none());
    assert_eq!(
        bucket(league, "Standard")[0]["request"]["scope"]["realm"],
        "pc"
    );
}

/// F1: the help's limit S53 contradicted its own field list — "an item's
/// class is not a field" beside `class`, a closed-set field. It says what
/// step 6 made true: the class is a derivation the search owns, read from
/// the base by the class table, and an item the table cannot class is
/// undecided with its reason.
#[test]
fn f1_the_helps_limit_on_class_says_what_the_class_table_made_true() {
    let d = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let s53 = d["limits"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["id"] == "S53")
        .unwrap()["said"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(!s53.contains("is not a field"), "{s53}");
    assert!(s53.contains("class table"), "{s53}");
    assert!(s53.contains("undecided"), "{s53}");
    assert!(
        d["fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"] == "class" && f["kind"] == "closed set")
    );
}

/// F4: a comparison written after `line( … )` — `line("T")>=15` — got a
/// parser message that said nothing of the language. It names the two
/// spellings that exist, `line("T").<slot>>=15` and `line("T" <slot>>=15)`,
/// one pair per slot the template has, each a query that binds.
#[test]
fn f4_a_comparison_after_a_line_group_names_the_two_spellings() {
    let ranged = "line(\"Adds # to # Cold Damage\")>=15";
    let e = parse_query(ranged).unwrap_err();
    let e = serde_json::to_value(&e).unwrap();
    assert_eq!(e["kind"], "slot_missing", "{e}");
    let readings: Vec<&str> = e["readings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert_eq!(
        readings,
        [
            "line(\"Adds # to # Cold Damage\").low>=15",
            "line(\"Adds # to # Cold Damage\" low>=15)",
            "line(\"Adds # to # Cold Damage\").high>=15",
            "line(\"Adds # to # Cold Damage\" high>=15)",
            "line(\"Adds # to # Cold Damage\").avg>=15",
            "line(\"Adds # to # Cold Damage\" avg>=15)",
        ]
    );
    for reading in readings {
        parse_query(reading).unwrap_or_else(|e| panic!("{reading}: {e}"));
    }
    // one number: the one slot; no quoted template: arg1
    for (text, first, second) in [
        (
            "line(\"# to maximum Life\")>=90",
            "line(\"# to maximum Life\").arg1>=90",
            "line(\"# to maximum Life\" arg1>=90)",
        ),
        (
            "line(template:cold is:fractured)=15..45",
            "line(template:cold is:fractured).arg1=15..45",
            "line(template:cold is:fractured arg1=15..45)",
        ),
    ] {
        let e = serde_json::to_value(parse_query(text).unwrap_err()).unwrap();
        assert_eq!(e["kind"], "slot_missing", "`{text}`: {e}");
        assert_eq!(e["readings"], json!([first, second]), "`{text}`");
        parse_query(first).unwrap();
        parse_query(second).unwrap();
    }
    // the message says where a slot goes, and the span is the group's
    let e = parse_query(ranged).unwrap_err();
    assert!(e.message.contains("slot"), "{}", e.message);
    assert_eq!(e.span, Some((0, ranged.len())));
}

/// F5: `class:staff` answered with all 82 names, twice, and the one meant
/// not singled out. A closed set's refusal offers the near names first —
/// a plural the game spells otherwise among them — and leaves the full
/// list to `--describe <field>`; a short list is still printed whole.
#[test]
fn f5_a_closed_sets_refusal_offers_the_near_names_and_not_the_list() {
    let (corpus, _) = generated::fixture(vec![json!({})]);
    let e = ask(&corpus, "class:staff").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_value");
    assert_eq!(e["readings"], json!(["class=Staves", "class=Warstaves"]));
    let message = e["error"].as_str().unwrap();
    assert!(!message.contains("Abyss Jewels"), "{message}");
    // the seat's ask 48: `=` on the singular
    let e = ask(&corpus, "class=ring").unwrap_err().to_json();
    assert_eq!(e["readings"][0], "class=Rings");
    assert!(!e["error"].as_str().unwrap().contains("Abyss Jewels"));
    // nothing near: the list is behind --describe, never inline
    let e = ask(&corpus, "class:zzzz").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_value");
    assert!(e.get("readings").is_none(), "{e}");
    let message = e["error"].as_str().unwrap();
    assert!(message.contains("--describe class"), "{message}");
    assert!(!message.contains("Abyss Jewels"), "{message}");
    // a short list is printed whole, as before
    let e = ask(&corpus, "rarity=zz").unwrap_err().to_json();
    assert!(
        e["error"]
            .as_str()
            .unwrap()
            .contains("normal, magic, rare, unique")
    );
    assert_eq!(
        e["readings"],
        json!([
            "rarity=normal",
            "rarity=magic",
            "rarity=rare",
            "rarity=unique"
        ])
    );
    let e = ask(&corpus, "rarity=rar").unwrap_err().to_json();
    assert_eq!(e["readings"], json!(["rarity=rare"]));
    // the plural forms a name may take: y to ies, f to ves, a bare s
    for (typed, first) in [
        ("class=glove", "class=Gloves"),
        ("class=boot", "class=Boots"),
        ("class=\"body armour\"", "class=\"Body Armours\""),
        ("class=quiver", "class=Quivers"),
        ("class:stavs", "class=Staves"),
    ] {
        let e = ask(&corpus, typed).unwrap_err().to_json();
        assert_eq!(e["readings"][0], first, "`{typed}`: {e}");
    }
}

/// F10: `has:Pseudo.DPS` offered no near reading. A computed value's name
/// is any-case, as every name is, and a misspelling offers the near one
/// — never `has:` on a total, which the same build refuses (T2).
#[test]
fn f10_a_computed_value_is_named_in_any_case_and_a_near_one_is_offered() {
    let (corpus, _) = generated::fixture(vec![json!({
        "properties": [
            { "name": "Physical Damage", "values": [["10-20", 0]], "displayMode": 0 },
            { "name": "Attacks per Second", "values": [["1.5", 0]], "displayMode": 0 } ]
    })]);
    for (typed, as_written) in [
        ("has:Pseudo.DPS", "has:pseudo.dps"),
        ("Pseudo.DPS>=1", "pseudo.dps>=1"),
        ("PSEUDO.total_res>=0", "pseudo.total_res>=0"),
        ("undecided(Pseudo.Total_Res)", "undecided(pseudo.total_res)"),
    ] {
        let a = as_json(&ask(&corpus, typed).unwrap_or_else(|e| panic!("`{typed}`: {e}")));
        let b = as_json(&ask(&corpus, as_written).unwrap());
        assert_eq!(a["total"], b["total"], "`{typed}`");
    }
    let e = ask(&corpus, "has:psuedo.dps").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_name");
    assert_eq!(e["readings"], json!(["has:pseudo.dps"]));
    let e = ask(&corpus, "psuedo.dps>=1").unwrap_err().to_json();
    assert_eq!(e["readings"], json!(["pseudo.dps>=1"]));
    // a total is never offered behind `has:` (T2)
    let e = ask(&corpus, "has:psuedo.total_res").unwrap_err().to_json();
    assert_eq!(e["kind"], "unknown_name");
    for reading in e["readings"].as_array().into_iter().flatten() {
        let reading = reading.as_str().unwrap();
        ask(&corpus, reading).unwrap_or_else(|e| panic!("`{reading}` offered, refused: {e}"));
    }
    assert_eq!(
        ask(&corpus, "has:pseudo.total_res").unwrap_err().to_json()["kind"],
        "has_on_computed"
    );
}

/// F7: a partial id got the never-fetched sentence as its zero
/// explanation. `id:` takes an id whole, as an answer printed it, and a
/// zero answer says so: the term is listed among what resolved to nothing,
/// with that note and no suggestion.
#[test]
fn f7_a_partial_id_is_said_to_find_nothing_because_an_id_is_whole() {
    let (corpus, _) = generated::fixture(vec![json!({}), json!({})]);
    let whole = as_json(&ask(&corpus, "id:i0").unwrap());
    assert_eq!(whole["total"]["matched"], 1);
    let part = as_json(&ask(&corpus, "id:i").unwrap());
    assert_eq!(part["total"]["matched"], 0);
    let nothing = part["zero"]["resolved_to_nothing"].as_array().unwrap();
    assert_eq!(nothing.len(), 1, "{nothing:?}");
    assert_eq!(nothing[0]["of"], "id");
    assert_eq!(nothing[0]["term"], "id:i");
    assert_eq!(nothing[0]["suggestions"], json!([]));
    assert!(
        nothing[0]["note"].as_str().unwrap().contains("whole"),
        "{}",
        nothing[0]
    );
    // an id that matched is not in the block
    assert_eq!(
        as_json(&ask(&corpus, "id:i0 rarity=unique").unwrap())["zero"]["resolved_to_nothing"],
        json!([])
    );
}

/// F6: a sort on a line the query did not name showed the number and not
/// the line, and a field's sort the number and not the field. A row says
/// what it sorts by: the field, the occurrence whose slot sorted it, a
/// sum's contributors, a computed value's inputs — as the query's own
/// terms are shown (C100).
#[test]
fn f6_a_row_says_what_it_sorts_by() {
    let (corpus, _) = generated::fixture(vec![
        json!({ "ilvl": 78, "explicitMods": ["Adds 1 to 4 Cold Damage", "+30 to maximum Life"] }),
        json!({ "ilvl": 73, "explicitMods": ["Adds 3 to 9 Cold Damage", "+20 to maximum Life", "+25 to maximum Life"] }),
    ]);
    let sorted = |sort: &str| -> Value {
        let a = generated::run(
            &corpus,
            &generated::request("rarity=rare", Some(sort), true, 10),
        )
        .unwrap();
        a["rows"][0].clone()
    };
    // a field: its name beside the number
    let row = sorted("ilvl");
    assert_eq!(row["id"], "i0");
    assert_eq!(
        row["sort"],
        json!({ "value": 78, "shows": [{ "value": { "name": "ilvl", "value": 78 } }] })
    );
    // a line: the occurrence whose slot sorted the row
    let row = sorted("line(\"Adds # to # Cold Damage\").high");
    assert_eq!(row["id"], "i1");
    assert_eq!(row["sort"]["value"], 9);
    assert_eq!(
        row["sort"]["shows"],
        json!([{ "line": { "source": "explicit", "flags": [], "text": "Adds 3 to 9 Cold Damage" } }])
    );
    // a sum: what it added
    let row = sorted("sum(\"# to maximum Life\")");
    assert_eq!(row["id"], "i1");
    assert_eq!(row["sort"]["value"], 45);
    let texts: Vec<&str> = row["sort"]["shows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["line"]["text"].as_str().unwrap())
        .collect();
    assert_eq!(texts, ["+20 to maximum Life", "+25 to maximum Life"]);
    // nothing to sort by: nothing shown, the status as before
    let a = generated::run(
        &corpus,
        &generated::request("rarity=rare", Some("line(template:spirit).arg1"), true, 10),
    )
    .unwrap();
    assert_eq!(
        a["rows"][0]["sort"],
        json!({ "status": "no satisfying occurrence" })
    );
}
