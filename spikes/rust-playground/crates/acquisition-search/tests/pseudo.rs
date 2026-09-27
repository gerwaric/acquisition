//! The computed values (`SEARCH-SLICE.md`, step 7; C94, C101, C93,
//! C95), through the crate's boundary: the shipped totals table, a total's
//! three statuses and its routes, `dps` and `pdps`, the sort and the sum
//! over a computed value, `--describe`, and every authoring error a
//! computed value has. Every count is worked by hand from the fixture
//! below.

mod common;

use acquisition_search::{Request, TOTALS_TABLE_VERSION, answer, describe};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

/// Six rare items in pc Standard and one in poe2.
///
/// `Gear` (g1): `ring_a`, 40 fire, 20 all-elemental and 12 fire-and-cold
/// — `total_res` 40 + 60 + 24 = 124, `total_fire_res` 72; `ring_b`, 30
/// cold and an implicit array that is no array; `ring_c`, mana alone;
/// `sword`, physical 59-88, elemental 38-71 and 57-108, chaos 10-20, at
/// 1.25 attacks per second — `pdps` 91.875, `dps` 281.875; `wand`, no
/// physical damage, elemental 38-71 at 1.4 — `dps` 76.3; `odd`, whose
/// attacks per second reads `fast`. `Vault` (v1) in poe2: `p2ring`, 40
/// fire, in a realm the totals table does not cover.
fn stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("g1", "Gear")]), 10);
    list_tabs(&mut s, "poe2", "Standard", json!([tab("v1", "Vault")]), 11);
    let rare =
        |id: &str, base: &str, more: Value| item(id, &format!("Item {id}"), base, "Rare", more);
    let property = |name: &str, values: &[&str]| json!({ "name": name, "values": values.iter().map(|v| json!([v, 0])).collect::<Vec<_>>(), "displayMode": 0 });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "g1",
        "Gear",
        vec![
            rare(
                "ring_a",
                "Two-Stone Ring",
                json!({ "explicitMods": ["+40% to Fire Resistance", "+20% to all Elemental Resistances", "+12% to Fire and Cold Resistances"] }),
            ),
            rare(
                "ring_b",
                "Two-Stone Ring",
                json!({ "implicitMods": "unreadable", "explicitMods": ["+30% to Cold Resistance"] }),
            ),
            rare(
                "ring_c",
                "Iron Ring",
                json!({ "explicitMods": ["+20 to maximum Mana"] }),
            ),
            rare(
                "sword",
                "Rusted Sword",
                json!({ "properties": [
                    property("Physical Damage", &["59-88"]),
                    property("Elemental Damage", &["38-71", "57-108"]),
                    property("Chaos Damage", &["10-20"]),
                    property("Attacks per Second", &["1.25"]),
                ] }),
            ),
            rare(
                "wand",
                "Driftwood Wand",
                json!({ "properties": [
                    property("Elemental Damage", &["38-71"]),
                    property("Attacks per Second", &["1.4"]),
                ] }),
            ),
            rare(
                "odd",
                "Rusted Sword",
                json!({ "properties": [
                    property("Physical Damage", &["59-88"]),
                    property("Attacks per Second", &["fast"]),
                ] }),
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
        vec![rare(
            "p2ring",
            "Iron Ring",
            json!({ "explicitMods": ["+40% to Fire Resistance"] }),
        )],
        21,
    );
    s
}

fn asked(s: &Store, realm: &str, text: &str) -> Value {
    as_json(&ask(&load(s, Some(realm)), text).unwrap())
}

fn view(s: &Store, text: &str, view: Value) -> Result<Value, Value> {
    let request: Request =
        serde_json::from_value(json!({ "query": { "text": text }, "view": view })).unwrap();
    answer(&load(s, Some("pc")), &request)
        .map(|a| as_json(&a))
        .map_err(|e| e.to_json())
}

fn counts(term: &Value) -> [u64; 4] {
    ["matched", "failed", "lacked", "undecided"].map(|k| term[k]["count"].as_u64().unwrap())
}

fn follow(s: &Store, realm: &str, count: &Value) -> Vec<String> {
    let request = serde_json::from_value(count["request"].clone()).unwrap();
    let routed = as_json(&answer(&load(s, Some(realm)), &request).unwrap());
    assert_eq!(routed["total"]["matched"], count["count"]);
    ids(&routed)
}

fn shows<'a>(a: &'a Value, id: &str, path: &str) -> &'a Value {
    let row = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap();
    &row["matched"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["path"] == path)
        .unwrap()["shows"]
}

/// C94: a total is its declared definition, exact; complete, an incomplete
/// subtotal, or unavailable in a realm the table does not cover — each
/// with its route and its reason — and a total of nothing is lacked, as
/// the trade site shows no pseudo where an item's lines sum to nothing
/// (owner, 2026-09-26: "Yes, let's make it absent to match the site"; of
/// an item with no such line at all as of one whose lines cancel: "A").
/// `has:` asks a total's presence as it asks a derived field's.
#[test]
fn c94_a_total_has_three_statuses_and_a_total_of_nothing_is_lacked() {
    let s = stash();
    let a = asked(&s, "pc", "pseudo.total_res>=60");
    let term = &a["terms"][0];
    // ring_a 124; ring_c, sword, wand and odd carry no resistance line and
    // lack it; ring_b a subtotal of 30
    assert_eq!(counts(term), [1, 0, 4, 1]);
    assert_eq!(ids(&a), ["ring_a"]);
    assert_eq!(
        shows(&a, "ring_a", "0")[0],
        json!({ "value": { "name": "pseudo.total_res", "value": 124 } })
    );
    // the lines the total counted, in the item's order
    let lines: Vec<&str> = shows(&a, "ring_a", "0")
        .as_array()
        .unwrap()
        .iter()
        .skip(1)
        .map(|e| e["line"]["text"].as_str().unwrap())
        .collect();
    assert_eq!(
        lines,
        [
            "+40% to Fire Resistance",
            "+20% to all Elemental Resistances",
            "+12% to Fire and Cold Resistances"
        ]
    );
    assert_eq!(
        follow(&s, "pc", &term["lacked"]),
        ["odd", "ring_c", "sword", "wand"],
        "a total lacked is routed by `-has:` of it"
    );
    assert_eq!(follow(&s, "pc", &term["undecided"]), ["ring_b"]);
    assert_eq!(
        ids(&asked(&s, "pc", "-has:pseudo.total_res")),
        ["odd", "ring_c", "sword", "wand"]
    );
    let has = asked(&s, "pc", "has:pseudo.total_res");
    assert_eq!(counts(&has["terms"][0]), [1, 0, 4, 1]);
    assert_eq!(ids(&has), ["ring_a"]);
    let why = &a["total"]["undecided_reasons"][0];
    assert_eq!(why["example"]["id"], "ring_b");
    assert_eq!(why["unread"], "implicit lines");
    // `undecided( … )` of a total is decided itself, and finds the same item
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(pseudo.total_res)")),
        ["ring_b"]
    );
    // no item has a total of 0: a comparison on a total it lacks is false
    assert_eq!(asked(&s, "pc", "pseudo.total_res=0")["total"]["matched"], 0);
    assert_eq!(
        asked(&s, "pc", "pseudo.total_res<60")["total"]["matched"],
        0
    );
    assert_eq!(
        counts(&asked(&s, "pc", "pseudo.total_fire_res=72")["terms"][0]),
        [1, 0, 4, 1]
    );

    // no definition for the realm: never zero, undecided with that reason
    let p2 = asked(&s, "poe2", "pseudo.total_res>=1");
    assert_eq!(counts(&p2["terms"][0]), [0, 0, 0, 1]);
    let why = &p2["total"]["undecided_reasons"][0];
    assert_eq!(why["example"]["id"], "p2ring");
    assert_eq!(why["unread"], "the total: no definition for the realm");
    assert_eq!(
        why["problem"],
        "no totals table for realm poe2 (totals v3 covers pc, xbox, sony)"
    );
    assert_eq!(
        why["hint"],
        "a refresh will not help; a reference update may"
    );
    assert_eq!(
        asked(&s, "poe2", "pseudo.total_res=0")["total"]["undecided"]["count"],
        1
    );
    assert_eq!(
        ids(&asked(&s, "poe2", "undecided(pseudo.total_res)")),
        ["p2ring"]
    );

    // the basis cites the table's version (C98)
    assert_eq!(a["basis"]["totals"], TOTALS_TABLE_VERSION);
    assert_eq!(
        serde_json::to_value(common::show(&s, "ring_a", false).unwrap()).unwrap()["basis"]["totals"],
        TOTALS_TABLE_VERSION
    );
}

/// C101: `pdps` and `dps` read the properties as displayed; an item
/// lacking the property lacks the field — counted, its route `-has:` of
/// the field (T2) — and one whose property is no number is undecided
/// with that reason; the routes partition the matches.
#[test]
fn c101_dps_and_pdps_read_the_displayed_properties() {
    let s = stash();
    let a = asked(&s, "pc", "pseudo.pdps>=90");
    let term = &a["terms"][0];
    // sword 91.875; the three rings and the wand lack it; odd cannot be read
    assert_eq!(counts(term), [1, 0, 4, 1]);
    assert_eq!(
        follow(&s, "pc", &term["lacked"]),
        ["ring_a", "ring_b", "ring_c", "wand"],
        "T2: a derived field lacked is routed by `-has:` of it"
    );
    assert_eq!(follow(&s, "pc", &term["matched"]), ["sword"]);
    assert_eq!(follow(&s, "pc", &term["undecided"]), ["odd"]);
    let evidence = shows(&a, "sword", "0");
    assert_eq!(
        evidence[0],
        json!({ "value": { "name": "pseudo.pdps", "value": 91.875 } })
    );
    assert_eq!(
        evidence[1],
        json!({ "shown": { "part": "properties", "text": "Physical Damage: 59-88" } })
    );
    let why = &a["total"]["undecided_reasons"][0];
    assert_eq!(why["example"]["id"], "odd");
    assert_eq!(why["unread"], "`properties`");
    assert_eq!(
        why["problem"],
        "`Attacks per Second` is `fast`: no number the search reads"
    );

    let a = asked(&s, "pc", "pseudo.dps>=1");
    assert_eq!(counts(&a["terms"][0]), [2, 0, 3, 1]);
    assert_eq!(ids(&a), ["sword", "wand"]);
    assert_eq!(
        shows(&a, "sword", "0")[0],
        json!({ "value": { "name": "pseudo.dps", "value": 281.875 } })
    );
    assert_eq!(
        shows(&a, "wand", "0")[0],
        json!({ "value": { "name": "pseudo.dps", "value": 76.3 } })
    );
    assert_eq!(shows(&a, "sword", "0").as_array().unwrap().len(), 5);
    assert_eq!(
        ids(&asked(&s, "pc", "pseudo.dps=76.3 pseudo.dps<100")),
        ["wand"]
    );
    assert_eq!(ids(&asked(&s, "pc", "undecided(pseudo.dps)")), ["odd"]);
}

/// Outside audit, 2026-09-24 (3): an unread element of `properties`
/// leaves a derived field open only when it may be a property the field
/// reads — a malformed `Quality` hides no physical dps, a malformed
/// `Elemental Damage` hides the dps and not the pdps, and an element with
/// no readable name hides both.
#[test]
fn audit_an_unrelated_malformed_property_hides_no_derived_field() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("g1", "Gear")]), 10);
    let property =
        |name: &str, values: Value| json!({ "name": name, "values": values, "displayMode": 0 });
    let weapon = |id: &str, more: Vec<Value>| {
        let mut properties = vec![
            property("Physical Damage", json!([["10-20", 1]])),
            property("Attacks per Second", json!([["2", 0]])),
        ];
        properties.extend(more);
        item(
            id,
            &format!("Item {id}"),
            "Rusted Sword",
            "Rare",
            json!({ "properties": properties }),
        )
    };
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "g1",
        "Gear",
        vec![
            weapon("quality", vec![property("Quality", json!(7))]),
            weapon("ele", vec![property("Elemental Damage", json!("no"))]),
            weapon("nameless", vec![json!(7)]),
            weapon("clean", vec![]),
        ],
        20,
    );
    let a = asked(&s, "pc", "pseudo.pdps=30");
    assert_eq!(counts(&a["terms"][0]), [3, 0, 0, 1]);
    assert_eq!(ids(&a), ["clean", "ele", "quality"]);
    let a = asked(&s, "pc", "pseudo.dps=30");
    assert_eq!(counts(&a["terms"][0]), [2, 0, 0, 2]);
    assert_eq!(ids(&a), ["clean", "quality"]);
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(pseudo.dps)")),
        ["ele", "nameless"]
    );
    // the reason is the element's own, named (`derive::Unread::name`)
    let shown = serde_json::to_value(common::show(&s, "ele", false).unwrap()).unwrap();
    assert_eq!(
        (
            &shown["item"]["unread"][0]["name"],
            &shown["item"]["unread"][0]["of"]
        ),
        (&json!("Elemental Damage"), &json!("properties"))
    );
}

/// Outside audit, 2026-09-24 (4): the vocabulary lists beside its
/// templates the computed values whose name or definition the narrowing
/// matches, marked computed, each counting the matches that carry it —
/// established and not zero — with a route that returns exactly them;
/// `line` alone lists none.
#[test]
fn audit_the_vocabulary_lists_the_computed_values_a_narrowing_matches() {
    let s = stash();
    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["line:fire resist"] } }),
    )
    .unwrap();
    let table = &a["view"]["counts"]["tables"][0];
    let computed: Vec<(&str, u64)> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["bucket"] == "computed")
        .map(|b| (b["value"].as_str().unwrap(), b["count"].as_u64().unwrap()))
        .collect();
    // the totals whose definition names a fire-resistance line — ring_a
    // carries each, ring_b's subtotal is incomplete, the rest lack it —
    // ranked by count, then by name
    assert_eq!(
        computed,
        [
            ("pseudo.total_ele_res", 1),
            ("pseudo.total_fire_res", 1),
            ("pseudo.total_res", 1)
        ]
    );
    let row = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["value"] == "pseudo.total_res")
        .unwrap();
    assert_eq!(row["term"], "has:pseudo.total_res");
    // the count is flattened into the row, as every bucket's is
    assert_eq!(follow(&s, "pc", row), ["ring_a"]);
    // a template row is still a template row, and the values counted are
    // theirs alone: the one template holding both words
    assert_eq!(table["values"], 1);
    // by name: the derived fields, counting the weapons that have one
    let a = view(&s, "", json!({ "counts": { "keys": ["line:dps"] } })).unwrap();
    let table = &a["view"]["counts"]["tables"][0];
    let computed: Vec<(&str, u64)> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["bucket"] == "computed")
        .map(|b| (b["value"].as_str().unwrap(), b["count"].as_u64().unwrap()))
        .collect();
    assert_eq!(computed, [("pseudo.dps", 2), ("pseudo.pdps", 1)]);
    // `line` alone lists none (T5, the owner's ruling 2026-09-24, after
    // the audit's second round had it list every one): the templates are
    // cut by the limit as before, and the computed count is zero
    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["line"], "limit": 2 } }),
    )
    .unwrap();
    let table = &a["view"]["counts"]["tables"][0];
    assert!(
        table["buckets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["bucket"] != "computed"),
        "{table}"
    );
    assert_eq!(
        (
            &table["computed"],
            &table["computed_left_out"],
            &table["left_out_needs"]
        ),
        (&json!(null), &json!(null), &json!("--limit"))
    );
    assert_eq!(table["values"], 5);
    // a requested sum is summed over a computed row's members too
    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["line:fire resist"], "sum": "ilvl" } }),
    )
    .unwrap();
    let row = a["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["value"] == "pseudo.total_res")
        .unwrap();
    assert_eq!(row["sum"], json!({ "value": 84, "lacking": 0 }));
}

/// Outside audit, 2026-09-24 (5): a reason made beyond the item's parts —
/// a derived field's malformed property — is a part of its own under the
/// six-part bound on an `undecided( … )` row, and what the bound leaves
/// out is counted: five unread arrays and four malformed properties are
/// nine parts, six shown. The answer's own block is by reason since the
/// first seat (V8): six kinds, the item's own parts' first, then the
/// reasons made beyond them — in every spelling the same.
#[test]
fn audit_reasons_beyond_the_items_parts_are_bounded_and_counted() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("g1", "Gear")]), 10);
    let property = |name: &str| json!({ "name": name, "values": [["fast", 0]], "displayMode": 0 });
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "g1",
        "Gear",
        vec![item(
            "nine",
            "Nine Ways",
            "Mystery Blade",
            "Rare",
            json!({
                "implicitMods": 1, "explicitMods": 2, "craftedMods": 3, "enchantMods": 4, "fracturedMods": 5,
                "properties": [
                    property("Attacks per Second"), property("Physical Damage"),
                    property("Elemental Damage"), property("Chaos Damage")]
            }),
        )],
        20,
    );
    let why = |text: &str| -> (Vec<String>, u64) {
        let a = asked(&s, "pc", &format!("undecided({text})"));
        let row = &a["rows"][0]["matched"][0];
        assert_eq!(a["rows"][0]["id"], "nine");
        let mut parts: Vec<String> = row["shows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|w| w["undecided"]["problem"].as_str().unwrap().to_string())
            .collect();
        parts.dedup();
        (parts, row["left_out"].as_u64().unwrap())
    };
    let (parts, left_out) = why("pseudo.total_res>=1 pseudo.dps>=1");
    assert_eq!(parts.len(), 6, "{parts:?}");
    assert_eq!(left_out, 3);
    // the item's own parts first, in its order, then the reasons made
    // beyond them by what they say — never by the term that met them, which
    // a rewrite may reorder (rule 9; the audit's second round)
    assert!(parts[0].starts_with("`craftedMods`"));
    assert!(parts[4].starts_with("`implicitMods`"));
    assert_eq!(
        parts[5],
        "`Attacks per Second` is `fast`: no number the search reads"
    );
    assert_eq!(why("pseudo.dps>=1 pseudo.total_res>=1"), (parts, left_out));
    // the answer's block, by reason: the five arrays' kinds, then the
    // property's — and with the class's reason among them a spelling never
    // changes the list (V8; rule 9)
    let reasons = |a: &Value| -> Vec<String> {
        a["total"]["undecided_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["unread"].as_str().unwrap().to_string())
            .collect()
    };
    let a = asked(&s, "pc", "pseudo.total_res>=1 pseudo.dps>=1");
    assert_eq!(
        reasons(&a),
        [
            "crafted lines",
            "enchant lines",
            "explicit lines",
            "fractured lines",
            "implicit lines",
            "`properties`"
        ]
    );
    assert!(a["total"].get("reasons_left_out").is_none());
    let a = asked(&s, "pc", "class=Rings pseudo.dps>=1 pseudo.total_res>=1");
    let b = asked(&s, "pc", "pseudo.total_res>=1 pseudo.dps>=1 class=Rings");
    assert_eq!(reasons(&a), reasons(&b));
    assert_eq!(reasons(&a).len(), 7);
    assert!(
        reasons(&a)
            .iter()
            .any(|p| p == "the class: base not in the table")
    );
    assert_eq!(a["total"]["undecided_reasons"][6]["example"]["id"], "nine");
}

/// C92, C95: a computed value sorts the rows and sums beside a count, with
/// the statuses a sum has — an item lacking it last and counted as
/// lacking, an incomplete one last with what was readable, a subtotal
/// marked incomplete.
#[test]
fn c92_c95_a_computed_value_sorts_and_sums() {
    let s = stash();
    let a = view(
        &s,
        "rarity=rare",
        json!({ "rows": { "sort": "pseudo.total_res", "desc": true } }),
    )
    .unwrap();
    let rows: Vec<(String, Value)> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap().to_string(), scalar(&r["sort"])))
        .collect();
    assert_eq!(rows[0], ("ring_a".to_string(), json!({ "value": 124 })));
    // then what is not a value, as a derived field's: ring_b with what
    // was readable, and the four that lack the total
    assert_eq!(rows.len(), 6);
    for (id, sort) in &rows[1..] {
        if id == "ring_b" {
            assert_eq!(*sort, json!({ "value": 30, "status": "incomplete" }));
        } else {
            assert_eq!(sort["status"], "no satisfying occurrence", "{id}");
        }
    }

    let a = view(
        &s,
        "rarity=rare",
        json!({ "rows": { "sort": "pseudo.dps", "desc": true } }),
    )
    .unwrap();
    let rows: Vec<(&str, Value)> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), scalar(&r["sort"])))
        .collect();
    assert_eq!(rows[0], ("sword", json!({ "value": 281.875 })));
    assert_eq!(rows[1], ("wand", json!({ "value": 76.3 })));
    // the rest last, in the store's order: odd with what was readable
    // (nothing), the rings lacking the field
    assert_eq!(rows[2], ("odd", json!({ "status": "incomplete" })));
    assert!(
        rows[3..]
            .iter()
            .all(|(_, sort)| sort["status"] == "no satisfying occurrence")
    );

    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["rarity"], "sum": "pseudo.total_res" } }),
    )
    .unwrap();
    // 124 and ring_b's readable 30; the four others lack it
    assert_eq!(
        a["view"]["counts"]["sum"],
        json!({ "name": "pseudo.total_res", "value": 154, "lacking": 4, "incomplete": true, "unread": 1 })
    );
    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["base"], "sum": "pseudo.dps" } }),
    )
    .unwrap();
    assert_eq!(
        a["view"]["counts"]["sum"],
        json!({ "name": "pseudo.dps", "value": 358.175, "lacking": 3, "incomplete": true, "unread": 1 })
    );
    let table = &a["view"]["counts"]["tables"][0];
    let bucket = |value: &str| -> Value {
        table["buckets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["value"] == value)
            .unwrap()["sum"]
            .clone()
    };
    assert_eq!(
        bucket("Rusted Sword"),
        json!({ "value": 281.875, "lacking": 0, "incomplete": true, "unread": 1 })
    );
    assert_eq!(
        bucket("Driftwood Wand"),
        json!({ "value": 76.3, "lacking": 0 })
    );
    assert_eq!(
        bucket("Two-Stone Ring"),
        json!({ "value": 0, "lacking": 2 })
    );
}

/// C97: `--describe` lists every computed value with its definition and
/// the table's provenance; a name answers to itself without `pseudo.`.
#[test]
fn c97_describe_lists_every_computed_value_with_its_definition() {
    let whole = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let computed = whole["computed"].as_array().unwrap();
    assert_eq!(computed.len(), 62);
    assert_eq!(computed[0]["name"], "pseudo.total_cold_res");
    assert_eq!(computed[35]["name"], "pseudo.total_life");
    assert_eq!(computed[36]["name"], "pseudo.total_mana");
    assert_eq!(computed[59]["name"], "pseudo.increased_mana_regen");
    assert_eq!(computed[60]["name"], "pseudo.dps");
    assert_eq!(computed[61]["kind"], "derived");
    assert!(
        whole["totals"]
            .as_str()
            .unwrap()
            .starts_with("totals v3, 60 totals over pc, xbox, sony: the trade site’s pseudo stats"),
        "{}",
        whole["totals"]
    );
    let one = serde_json::to_value(describe(&["total_res".to_string()]).unwrap()).unwrap();
    assert_eq!(one["computed"].as_array().unwrap().len(), 1);
    assert!(one["fields"].as_array().unwrap().is_empty());
    let what = one["computed"][0]["what"].as_str().unwrap();
    assert!(
        what.starts_with("the trade site's pseudo_total_resistance (`+#% total Resistance`): the sum over the item's lines of 1 × arg1 of line(\"#% to Fire Resistance\")"),
        "{what}"
    );
    assert!(what.contains("3 × arg1 of line(\"#% to all Elemental Resistances\")"));
    assert_eq!(
        one["computed"][0]["examples"],
        json!([
            "pseudo.total_res>=60",
            "-has:pseudo.total_res",
            "undecided(pseudo.total_res)"
        ])
    );
    let block = serde_json::to_value(describe(&["pseudo".to_string()]).unwrap()).unwrap();
    assert_eq!(block["computed"].as_array().unwrap().len(), 62);
    // what the reference names and no step builds is refused by that name
    let e = describe(&["defence_pct".to_string()]).unwrap_err();
    assert!(
        e.message
            .starts_with("not built: pseudo.defence_pct (no step of the plan)")
    );
}

/// T2 (owner, 2026-09-24: "A ring has no dps"): `has:pseudo.dps` matches
/// the items whose properties establish it, lacks on those with none, is
/// undecided where one could not be read, and every count is routed;
/// `-has:` returns the lackers. A total takes `has:` the same way since
/// it can be lacked (C94 as ruled 2026-09-26; the test above).
#[test]
fn t2_has_on_a_derived_field_is_a_property_s_presence() {
    let s = stash();
    let a = asked(&s, "pc", "has:pseudo.dps");
    let term = &a["terms"][0];
    // sword and wand display attacks per second and a damage range; the
    // three rings display neither; odd's attacks per second is `fast`
    assert_eq!(counts(term), [2, 0, 3, 1]);
    assert_eq!(follow(&s, "pc", &term["matched"]), ["sword", "wand"]);
    assert_eq!(
        follow(&s, "pc", &term["lacked"]),
        ["ring_a", "ring_b", "ring_c"]
    );
    assert_eq!(follow(&s, "pc", &term["undecided"]), ["odd"]);
    assert_eq!(a["query"]["text"], "has:pseudo.dps");
    let a = asked(&s, "pc", "-has:pseudo.dps");
    assert_eq!(ids(&a), ["ring_a", "ring_b", "ring_c"]);
    assert_eq!(a["query"]["text"], "-has:pseudo.dps");
}

/// Every authoring error a computed value has, each with its kind and
/// what it offers — near names, never a guess.
#[test]
fn a_computed_value_that_cannot_be_is_an_authoring_error() {
    let s = stash();
    let corpus = load(&s, Some("pc"));
    for (text, kind, says) in [
        ("pseudo.total_rse>=60", "unknown_name", "pseudo.total_res"),
        (
            "has:pseudo.total_rse",
            "unknown_name",
            "has:pseudo.total_res",
        ),
        (
            "has:pseudo.nothing_here",
            "unknown_name",
            "no computed value",
        ),
        (
            "pseudo.nothing_here>=1",
            "unknown_name",
            "no computed value",
        ),
        (
            "pseudo.total_res.avg>=1",
            "slot_unknown",
            "takes no slot word",
        ),
        ("pseudo.dps.low>=1", "slot_unknown", "takes no slot word"),
        (
            "pseudo.cold_damage.avg>=30",
            "not_built",
            "pseudo.<name>.<slot>",
        ),
        ("pseudo.defence_pct>=90", "not_built", "pseudo.defence_pct"),
        (
            "undecided(pseudo.defence_pct)",
            "not_built",
            "pseudo.defence_pct",
        ),
        (
            "undecided(pseudo.total_rse)",
            "unknown_name",
            "pseudo.total_res",
        ),
    ] {
        let e = ask(&corpus, text).unwrap_err().to_json();
        assert_eq!(e["kind"], kind, "`{text}`: {e}");
        let said: Vec<&str> = std::iter::once(&e["error"])
            .chain(e["readings"].as_array().into_iter().flatten())
            .filter_map(Value::as_str)
            .collect();
        assert!(said.join(" · ").contains(says), "`{text}`: {e}");
    }
    for (sort, kind) in [
        ("pseudo.total_res", "ok"),
        ("pseudo.dps", "ok"),
        ("pseudo.total_res.high", "slot_unknown"),
        ("pseudo.nothing_here.avg", "not_built"),
        ("pseudo.defence_pct", "not_built"),
    ] {
        let result = view(&s, "", json!({ "rows": { "sort": sort } }));
        match (result, kind) {
            (Ok(_), "ok") => {}
            (Err(e), kind) => assert_eq!(e["kind"], kind, "--sort {sort}: {e}"),
            (Ok(_), kind) => panic!("--sort {sort} bound, and {kind} was expected"),
        }
    }
}

/// Nine items whose totals the trade site's own answers decide (the
/// build plan, step 9c; `search/pseudo-stats/data/table-changes.csv`).
///
/// `Site` (s1): `boots`, 30 fire and an eldritch implicit's 13 — fire 43;
/// `helm`, the other eldritch form's 22 chaos; `veil`, 13 to All
/// Resistances and 24 cold — fire 13, cold 37, elemental 3 × 13 + 24 = 63,
/// every resistance 4 × 13 + 24 = 76; `visor`, 40 life, a crafted 10 and
/// 11 Strength and Intelligence — life 50 + 5.5; `jewel`, 6 to all
/// Attributes and 10 Strength and Intelligence — life 8, Strength 16;
/// `gloves`, an eldritch implicit's 13 attack speed; `taken`, the unique
/// whose Strength and Intelligence the site leaves out — life 18.5;
/// `blade`, 1 to the level of socketed skill gems; `cancel`, 21 Strength
/// and -21.
fn site_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("s1", "Site")]), 10);
    let rare =
        |id: &str, base: &str, more: Value| item(id, &format!("Item {id}"), base, "Rare", more);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "s1",
        "Site",
        vec![
            rare(
                "boots",
                "Crusader Boots",
                json!({
                    "implicitMods": ["While a Unique Enemy is in your Presence, +13% to Fire Resistance"],
                    "explicitMods": ["+30% to Fire Resistance"],
                }),
            ),
            rare(
                "helm",
                "Lion Pelt",
                json!({ "implicitMods": ["While a Pinnacle Atlas Boss is in your Presence, +22% to Chaos Resistance"] }),
            ),
            rare(
                "veil",
                "Zodiac Leather",
                json!({ "explicitMods": ["+24% to Cold Resistance", "+13% to All Resistances"] }),
            ),
            rare(
                "visor",
                "Vaal Mask",
                json!({
                    "explicitMods": ["+40 to maximum Life", "+11 to Strength and Intelligence"],
                    "craftedMods": ["+10 to maximum Life"],
                }),
            ),
            rare(
                "jewel",
                "Crimson Jewel",
                json!({ "explicitMods": ["+6 to all Attributes", "+10 to Strength and Intelligence"] }),
            ),
            rare(
                "gloves",
                "Shagreen Gloves",
                json!({ "implicitMods": ["While a Unique Enemy is in your Presence, 13% increased Attack Speed"] }),
            ),
            item(
                "taken",
                "That Which Was Taken",
                "Crimson Jewel",
                "Unique",
                json!({ "explicitMods": ["+37 to Strength and Intelligence"] }),
            ),
            rare(
                "blade",
                "Etched Greatsword",
                json!({ "explicitMods": ["+1 to Level of Socketed Skill Gems"] }),
            ),
            rare(
                "cancel",
                "Amber Amulet",
                json!({
                    "implicitMods": ["+21 to Strength"],
                    "explicitMods": ["-21 to Strength"],
                }),
            ),
        ],
        20,
    );
    s
}

/// C94, as the owner ruled it 2026-09-26: a total whose lines cancel is
/// lacked as one with no line is — the trade site shows no pseudo for
/// either (c3, d01: `+21 to Strength` beside `-21 to Strength`).
#[test]
fn c94_a_total_whose_lines_cancel_is_lacked() {
    let s = site_stash();
    // `cancel` carries two Strength lines, 21 and -21: every Strength
    // total's rows name them, and each sums to nothing
    for total in ["pseudo.total_str", "pseudo.total_life"] {
        let a = asked(&s, "pc", &format!("{total}<=0"));
        assert_eq!(a["total"]["matched"], 0, "{total}");
        let lackers = follow(&s, "pc", &a["terms"][0]["lacked"]);
        assert!(
            lackers.contains(&"cancel".to_string()),
            "{total}: {lackers:?}"
        );
        assert!(
            !ids(&asked(&s, "pc", &format!("has:{total}"))).contains(&"cancel".to_string()),
            "{total}"
        );
    }
    // visor's 11, jewel's 16 and taken's 37 are the Strength totals there are
    assert_eq!(
        ids(&asked(&s, "pc", "has:pseudo.total_str")),
        ["jewel", "taken", "visor"]
    );
    // the lines are on the item all the same, and a sum of them is the
    // user's arithmetic: zero
    assert_eq!(
        ids(&asked(
            &s,
            "pc",
            "sum(\"# to Strength\")=0 \"# to Strength\""
        )),
        ["cancel"]
    );
}

/// C94, V6: a total counts what the trade site's pseudo of that name
/// counts, each row on the capture that asked for it — a row's two
/// eldritch forms as the row, `All Resistances` under every resistance
/// total, total life with every Strength line at a half — and where the
/// owner ruled the site wrong, the line is counted: the twin on That
/// Which Was Taken, and the skill gems' own text.
#[test]
fn v6_a_total_counts_what_the_sites_pseudo_counts() {
    let s = site_stash();
    let found = |query: &str| ids(&asked(&s, "pc", query));
    // a row's eldritch forms are the row (c6, d03, round three)
    assert_eq!(found("pseudo.total_fire_res=43"), ["boots"]);
    assert_eq!(found("pseudo.total_chaos_res=22"), ["helm"]);
    assert_eq!(found("pseudo.total_attack_speed=13"), ["gloves"]);
    // All Resistances, under each total it names a part of (e042, e044, e048)
    assert_eq!(found("pseudo.total_fire_res=13"), ["veil"]);
    assert_eq!(found("pseudo.total_cold_res=37"), ["veil"]);
    assert_eq!(found("pseudo.total_chaos_res=13"), ["veil"]);
    assert_eq!(found("pseudo.total_ele_res=63"), ["veil"]);
    assert_eq!(found("pseudo.total_res=76"), ["veil"]);
    // total life: life at 1, a crafted line as any other, Strength at a half (q4, p1, c1)
    assert_eq!(found("pseudo.total_life=55.5"), ["visor"]);
    assert_eq!(found("pseudo.total_life=8"), ["jewel"]);
    assert_eq!(found("pseudo.total_str=16"), ["jewel"]);
    // what the site leaves out and the search counts, by the owner's rulings
    assert_eq!(found("pseudo.total_life=18.5"), ["taken"]);
    assert_eq!(found("pseudo.total_str=37"), ["taken"]);
    assert_eq!(found("pseudo.total_skill_gem_levels=1"), ["blade"]);
    // the lines a total counted are shown: these three, whatever their order
    let a = asked(&s, "pc", "pseudo.total_life>=50");
    let mut lines: Vec<&str> = shows(&a, "visor", "0")
        .as_array()
        .unwrap()
        .iter()
        .skip(1)
        .map(|e| e["line"]["text"].as_str().unwrap())
        .collect();
    lines.sort_unstable();
    assert_eq!(
        lines,
        [
            "+10 to maximum Life",
            "+11 to Strength and Intelligence",
            "+40 to maximum Life"
        ]
    );
}

/// Thirteen items whose totals the trade site answered at step 9c2
/// (`search/pseudo-stats/data/table-changes.csv`, rounds five and six).
///
/// `Other` (o1): `anvil`, 10 reduced attack, cast and movement speed —
/// each total -10; `devoto`, 10 reduced global physical damage and 20
/// movement speed; `gyre`, 48 reduced rarity and a fractured 48 increased
/// — nothing; `heart`, 25 reduced maximum energy shield and 28 to all
/// Attributes — mana 14; `pace`, 25 Intelligence and 10 Dexterity — mana
/// 12.5; `taken`, the unique whose 37 Strength and Intelligence the site
/// leaves out — mana 18.5; `quiver`, 24 mana and 9 Dexterity and
/// Intelligence — mana 28.5; `robe`, 25 and an implicit 11 energy shield;
/// `wand`, 12 lightning, 10 elemental and 17 spell damage, and 10
/// elemental damage with attack skills; `torch`, 20 burning and 15 fire
/// damage; `belt`, 0.4 of each leech, 1.5% of life regenerated, 30 global
/// critical strike chance and 25 multiplier; `crown`, 20 reduced mana
/// regeneration; `greaves`, an eldritch implicit's 9 movement speed.
fn other_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("o1", "Other")]), 10);
    let rare =
        |id: &str, base: &str, more: Value| item(id, &format!("Item {id}"), base, "Rare", more);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "o1",
        "Other",
        vec![
            rare(
                "anvil",
                "Amber Amulet",
                json!({ "explicitMods": ["10% reduced Attack Speed", "10% reduced Cast Speed", "10% reduced Movement Speed"] }),
            ),
            rare(
                "devoto",
                "Nightmare Bascinet",
                json!({ "explicitMods": ["10% reduced Global Physical Damage", "20% increased Movement Speed"] }),
            ),
            rare(
                "gyre",
                "Amethyst Ring",
                json!({
                    "explicitMods": ["48% reduced Rarity of Items found"],
                    "fracturedMods": ["48% increased Rarity of Items found"],
                }),
            ),
            rare(
                "heart",
                "Onyx Amulet",
                json!({ "explicitMods": ["25% reduced maximum Energy Shield", "+28 to all Attributes"] }),
            ),
            rare(
                "pace",
                "Scholar Boots",
                json!({ "explicitMods": ["+25 to Intelligence", "+10 to Dexterity"] }),
            ),
            item(
                "taken",
                "That Which Was Taken",
                "Crimson Jewel",
                "Unique",
                json!({ "explicitMods": ["+37 to Strength and Intelligence"] }),
            ),
            rare(
                "quiver",
                "Fire Arrow Quiver",
                json!({ "explicitMods": ["+24 to maximum Mana", "+9 to Dexterity and Intelligence"] }),
            ),
            rare(
                "robe",
                "Sage's Robe",
                json!({
                    "implicitMods": ["+11 to maximum Energy Shield"],
                    "explicitMods": ["+25 to maximum Energy Shield"],
                }),
            ),
            rare(
                "wand",
                "Tornado Wand",
                json!({ "explicitMods": [
                    "12% increased Lightning Damage",
                    "10% increased Elemental Damage",
                    "17% increased Spell Damage",
                    "10% increased Elemental Damage with Attack Skills",
                ] }),
            ),
            rare(
                "torch",
                "Ashscale Talisman",
                json!({ "explicitMods": ["20% increased Burning Damage", "15% increased Fire Damage"] }),
            ),
            rare(
                "belt",
                "Leather Belt",
                json!({ "explicitMods": [
                    "0.4% of Physical Attack Damage Leeched as Life",
                    "0.4% of Physical Attack Damage Leeched as Mana",
                    "Regenerate 1.5% of Life per second",
                    "30% increased Global Critical Strike Chance",
                    "+25% to Global Critical Strike Multiplier",
                ] }),
            ),
            rare(
                "crown",
                "Prophet Crown",
                json!({ "explicitMods": ["20% reduced Mana Regeneration Rate"] }),
            ),
            rare(
                "greaves",
                "Crusader Boots",
                json!({ "implicitMods": ["While a Unique Enemy is in your Presence, 9% increased Movement Speed"] }),
            ),
        ],
        20,
    );
    s
}

/// C94, step 9c2 (owner, 2026-09-27: "yes, we need to be able to find
/// reduced lines and totals"): a row's `reduced` spelling is the row,
/// below nothing, as the trade site shows `-10% total Attack Speed` over
/// `10% reduced Attack Speed` (h042–h052); a total below nothing is a
/// total, and one whose lines cancel is lacked (g034).
#[test]
fn c94_a_reduced_line_counts_below_nothing() {
    let s = other_stash();
    let found = |query: &str| ids(&asked(&s, "pc", query));
    // three totals the build shipped before the rule, and one line each
    assert_eq!(found("pseudo.total_attack_speed=-10"), ["anvil"]);
    assert_eq!(found("pseudo.total_cast_speed=-10"), ["anvil"]);
    assert_eq!(found("pseudo.total_increased_phys=-10"), ["devoto"]);
    // and the totals of this step
    assert_eq!(found("pseudo.total_increased_energy_shield=-25"), ["heart"]);
    assert_eq!(found("pseudo.increased_mana_regen=-20"), ["crown"]);
    assert_eq!(found("pseudo.increased_movement_speed<0"), ["anvil"]);
    // below nothing is a total: present, and under a bound above it
    assert_eq!(
        found("has:pseudo.increased_movement_speed"),
        ["anvil", "devoto", "greaves"]
    );
    assert_eq!(
        found("pseudo.increased_movement_speed<=9"),
        ["anvil", "greaves"]
    );
    // 48 reduced beside 48 increased is a total of nothing: lacked
    let a = asked(&s, "pc", "pseudo.increased_rarity<=0");
    assert_eq!(a["total"]["matched"], 0);
    assert!(follow(&s, "pc", &a["terms"][0]["lacked"]).contains(&"gyre".to_string()));
    assert!(found("has:pseudo.increased_rarity").is_empty());
    // the line that made the total is shown
    let a = asked(&s, "pc", "pseudo.total_attack_speed<0");
    let lines: Vec<&str> = shows(&a, "anvil", "0")
        .as_array()
        .unwrap()
        .iter()
        .skip(1)
        .map(|e| e["line"]["text"].as_str().unwrap())
        .collect();
    assert_eq!(lines, ["10% reduced Attack Speed"]);
}

/// C94, V6, step 9c2: the site's other totals count what the site's
/// pseudo of that name counts, each row on the capture that asked for it
/// (rounds five and six) — and the twin on That Which Was Taken is counted
/// toward mana as toward life (owner, 2026-09-27: "yes, count it for
/// mana.").
#[test]
fn v6_the_other_totals_count_what_the_sites_pseudo_counts() {
    let s = other_stash();
    let found = |query: &str| ids(&asked(&s, "pc", query));
    // total mana: mana at 1, every Intelligence line at a half (g005)
    assert_eq!(found("pseudo.total_mana=14"), ["heart"]);
    assert_eq!(found("pseudo.total_mana=12.5"), ["pace"]);
    assert_eq!(found("pseudo.total_mana=28.5"), ["quiver"]);
    assert_eq!(found("pseudo.total_mana=18.5"), ["taken"]);
    assert_eq!(
        found("has:pseudo.total_mana"),
        ["heart", "pace", "quiver", "taken"]
    );
    // energy shield: the line, an implicit as any other (g006)
    assert_eq!(found("pseudo.total_energy_shield=36"), ["robe"]);
    // a type's damage counts elemental damage; its spell damage, spell
    // damage too; its damage with attack skills, elemental damage with
    // attack skills too (g014–g031)
    assert_eq!(found("pseudo.increased_lightning_damage=22"), ["wand"]);
    assert_eq!(
        found("pseudo.increased_lightning_spell_damage=39"),
        ["wand"]
    );
    assert_eq!(
        found("pseudo.increased_lightning_attack_damage=32"),
        ["wand"]
    );
    assert_eq!(found("pseudo.increased_cold_damage=10"), ["wand"]);
    assert_eq!(found("pseudo.increased_cold_spell_damage=27"), ["wand"]);
    assert_eq!(found("pseudo.increased_ele_attack_damage=20"), ["wand"]);
    assert_eq!(found("pseudo.increased_ele_damage=10"), ["wand"]);
    assert_eq!(found("pseudo.increased_spell_damage=17"), ["wand"]);
    // burning damage counts fire damage, and fire damage no burning (g035)
    assert_eq!(found("pseudo.increased_burning_damage=35"), ["torch"]);
    assert_eq!(found("pseudo.increased_fire_damage=15"), ["torch"]);
    // what is displayed to a hundredth is read to a hundredth (g039–g042)
    assert_eq!(found("pseudo.phys_attack_life_leech=0.4"), ["belt"]);
    assert_eq!(found("pseudo.phys_attack_mana_leech=0.4"), ["belt"]);
    assert_eq!(found("pseudo.life_regen_pct=1.5"), ["belt"]);
    assert_eq!(found("pseudo.global_crit_chance=30"), ["belt"]);
    assert_eq!(found("pseudo.global_crit_multi=25"), ["belt"]);
    // a row's eldritch forms are the row, of these totals as of 9c's
    assert_eq!(found("pseudo.increased_movement_speed=9"), ["greaves"]);
}
