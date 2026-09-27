//! The computed values (`search/LEDGER.md`, step 7; C94, C101, C93,
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
        "no totals table for realm poe2 (totals v5 covers pc, xbox, sony)"
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
    assert_eq!(computed.len(), 67);
    assert_eq!(computed[0]["name"], "pseudo.total_cold_res");
    assert_eq!(computed[35]["name"], "pseudo.total_life");
    assert_eq!(computed[36]["name"], "pseudo.total_mana");
    assert_eq!(computed[60]["name"], "pseudo.increased_mana_regen");
    // the readings of other totals after the totals they read (step 9c3)
    assert_eq!(computed[60]["kind"], "total");
    assert_eq!(computed[61]["name"], "pseudo.count_res");
    assert_eq!(computed[61]["kind"], "derived");
    assert_eq!(computed[64]["name"], "pseudo.total_all_attributes");
    assert_eq!(computed[65]["name"], "pseudo.dps");
    assert_eq!(computed[66]["kind"], "derived");
    assert!(
        whole["totals"].as_str().unwrap().starts_with(
            "totals v5, 61 totals and 4 readings of them over pc, xbox, sony: the trade site’s pseudo stats"
        ),
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
    assert_eq!(block["computed"].as_array().unwrap().len(), 67);
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

/// Fourteen items whose totals the trade site answered at step 9c2
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
/// regeneration; `greaves`, an eldritch implicit's 9 movement speed;
/// `plate`, 19.2 life regenerated a second and an implicit 3.4 — 22.6.
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
            rare(
                "plate",
                "Astral Plate",
                json!({
                    "implicitMods": ["Regenerate 3.4 Life per second"],
                    "explicitMods": ["Regenerate 19.2 Life per second"],
                }),
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
    // life regenerated is what the lines' text says, and no more: the site
    // rounds what the text cuts (g037; owner, 2026-09-27: "yes, let's go
    // with what we can observe directly from the text we have.")
    assert_eq!(found("pseudo.life_regen=22.6"), ["plate"]);
    assert_eq!(found("has:pseudo.life_regen"), ["plate"]);
    assert_eq!(found("has:pseudo.life_regen_pct"), ["belt"]);
}

/// Sixteen items, eleven of them as the trade site's captures hold them,
/// for the four pseudos that are a reading of other totals (the build
/// plan, step 9c3; `tools/trade_rows.py`, `DERIVED`), and one in poe2.
///
/// `Read` (r1), each with its fire, cold, lightning and chaos totals:
/// `thread` (R1, Thread of Hope), -17 to all elemental, the second of its
/// mod's three rows as the capture gives it — -17, -17, -17;
/// `entropy` (R1), 24 lightning and 8 all elemental — 8, 8, 32; `twostone`
/// (g001), 13 fire and lightning — 13, none, 13; `grasp` (h040), 23 chaos;
/// `turn` (h040), 23 fire, 21 chaos and 12 Dexterity; `crest` (h040,
/// Geofri's Crest) — 17, 20, 17, 22; `spiral` (g003), 16 all elemental, 7
/// cold, 22 lightning — 16, 23, 38; `gamble` (h039, Ventor's Gamble) — 15,
/// 14, 36 and no all-elemental line; `clasp` (g004), 34 cold, 15 to all
/// Attributes and 9 Strength — Strength 24, Dexterity 15, Intelligence
/// 15; `bite` (h001, Hyrri's Bite) — 21, 45, 18 and no all-attributes
/// line; `knot` (h002), 12 fire and cold, 18 to all Attributes and -18
/// Dexterity — 18, nothing, 18; `mana`, mana alone; `long`, 20 fire, 20
/// cold and a chaos line whose number has eleven digits; `wide`, 20 fire,
/// 20 cold and such a lightning line; `lone`, such a chaos line alone;
/// `first`, such a fire line and 20 cold. `Vault` (v1) in poe2: `p2ring`.
fn reading_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("r1", "Read")]), 10);
    list_tabs(&mut s, "poe2", "Standard", json!([tab("v1", "Vault")]), 11);
    let rare =
        |id: &str, base: &str, more: Value| item(id, &format!("Item {id}"), base, "Rare", more);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "r1",
        "Read",
        vec![
            item(
                "thread",
                "Thread of Hope",
                "Crimson Jewel",
                "Unique",
                json!({ "explicitMods": [
                    "Only affects Passives in Small Ring",
                    "Passive Skills in Radius can be Allocated without being connected to your tree\n-17% to all Elemental Resistances\nPassage",
                ] }),
            ),
            rare(
                "entropy",
                "Topaz Ring",
                json!({
                    "implicitMods": ["+24% to Lightning Resistance"],
                    "explicitMods": ["+52 to maximum Life", "+8% to all Elemental Resistances"],
                }),
            ),
            rare(
                "twostone",
                "Two-Stone Ring",
                json!({ "implicitMods": ["+13% to Fire and Lightning Resistances"] }),
            ),
            rare(
                "grasp",
                "Spiked Gloves",
                json!({ "explicitMods": ["11% increased Attack Speed", "+23% to Chaos Resistance"] }),
            ),
            rare(
                "turn",
                "Amethyst Ring",
                json!({
                    "implicitMods": ["+21% to Chaos Resistance"],
                    "explicitMods": ["+12 to Dexterity", "+23% to Fire Resistance"],
                }),
            ),
            item(
                "crest",
                "Geofri's Crest",
                "Great Crown",
                "Unique",
                json!({ "explicitMods": [
                    "+17% to Fire Resistance",
                    "+20% to Cold Resistance",
                    "+17% to Lightning Resistance",
                    "+22% to Chaos Resistance",
                ] }),
            ),
            rare(
                "spiral",
                "Moonstone Ring",
                json!({ "explicitMods": [
                    "+16% to all Elemental Resistances",
                    "+7% to Cold Resistance",
                    "+22% to Lightning Resistance",
                ] }),
            ),
            item(
                "gamble",
                "Ventor's Gamble",
                "Gold Ring",
                "Unique",
                json!({ "explicitMods": [
                    "+15% to Fire Resistance",
                    "+14% to Cold Resistance",
                    "+36% to Lightning Resistance",
                ] }),
            ),
            rare(
                "clasp",
                "Onyx Amulet",
                json!({
                    "implicitMods": ["+15 to all Attributes"],
                    "explicitMods": ["+9 to Strength", "+34% to Cold Resistance"],
                }),
            ),
            item(
                "bite",
                "Hyrri's Bite",
                "Sharktooth Arrow Quiver",
                "Unique",
                json!({ "explicitMods": ["+21 to Strength", "+45 to Dexterity", "+18 to Intelligence"] }),
            ),
            rare(
                "knot",
                "Two-Stone Ring",
                json!({
                    "implicitMods": ["+12% to Fire and Cold Resistances"],
                    "explicitMods": ["+18 to all Attributes", "-18 to Dexterity"],
                }),
            ),
            rare(
                "mana",
                "Iron Ring",
                json!({ "explicitMods": ["+20 to maximum Mana"] }),
            ),
            rare(
                "long",
                "Iron Ring",
                json!({ "explicitMods": [
                    "+20% to Fire Resistance",
                    "+20% to Cold Resistance",
                    "+10000000000% to Chaos Resistance",
                ] }),
            ),
            rare(
                "wide",
                "Iron Ring",
                json!({ "explicitMods": [
                    "+20% to Fire Resistance",
                    "+20% to Cold Resistance",
                    "+10000000000% to Lightning Resistance",
                ] }),
            ),
            rare(
                "lone",
                "Iron Ring",
                json!({ "explicitMods": ["+10000000000% to Chaos Resistance"] }),
            ),
            rare(
                "first",
                "Iron Ring",
                json!({ "explicitMods": [
                    "+10000000000% to Fire Resistance",
                    "+20% to Cold Resistance",
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

/// The items a count routes to; a count of none carries no route.
fn members(s: &Store, count: &Value) -> Vec<String> {
    if count["count"] == 0 {
        return Vec::new();
    }
    follow(s, "pc", count)
}

/// What a row shows of a reading: its value, then each total it read.
fn read_by(a: &Value, id: &str) -> Vec<(String, Value)> {
    shows(a, id, "0")
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["value"]["name"].as_str().unwrap().to_string(),
                e["value"]["value"].clone(),
            )
        })
        .collect()
}

/// C101, step 9c3 (owner, 2026-09-26: "They should be present at
/// launch."): the trade site's two counts are how many of the resistance
/// totals an item shows, chaos among the four and not among the three —
/// the resistances, never the lines (R1, g001, g002, h040) — and a count
/// of nothing is no count, as the site shows none (h040, six items).
#[test]
fn c101_a_count_is_how_many_of_its_totals_the_item_shows() {
    let s = reading_stash();
    let found = |query: &str| ids(&asked(&s, "pc", query));
    // one line of all three, and below nothing, is three (R1)
    assert_eq!(
        found("pseudo.count_res=3 pseudo.count_ele_res=3"),
        ["entropy", "gamble", "spiral", "thread"]
    );
    // one line of two resistances is two (g001)
    assert_eq!(
        found("pseudo.count_res=2 pseudo.count_ele_res=2"),
        ["knot", "twostone"]
    );
    // chaos is counted by the one and not by the other (h040)
    assert_eq!(
        found("pseudo.count_res=4 pseudo.count_ele_res=3"),
        ["crest"]
    );
    assert_eq!(found("pseudo.count_res=2 pseudo.count_ele_res=1"), ["turn"]);
    assert_eq!(
        found("pseudo.count_res=1 -has:pseudo.count_ele_res"),
        ["grasp"]
    );
    assert_eq!(
        found("pseudo.count_res=1 pseudo.count_ele_res=1"),
        ["clasp"]
    );

    let a = asked(&s, "pc", "pseudo.count_res>=3");
    let term = &a["terms"][0];
    // crest 4; entropy, gamble, spiral and thread 3; clasp and grasp 1,
    // knot, turn and twostone 2, and first one or two; bite and mana show
    // no resistance; long and wide show two and may show a third, lone
    // none and may show one
    assert_eq!(counts(term), [5, 6, 2, 3]);
    assert_eq!(
        follow(&s, "pc", &term["failed"]),
        ["clasp", "first", "grasp", "knot", "turn", "twostone"]
    );
    assert_eq!(follow(&s, "pc", &term["lacked"]), ["bite", "mana"]);
    assert_eq!(
        follow(&s, "pc", &term["undecided"]),
        ["lone", "long", "wide"]
    );
    // a row shows the count, then the totals it counted
    assert_eq!(
        read_by(&a, "crest"),
        [
            ("pseudo.count_res".to_string(), json!(4)),
            ("pseudo.total_fire_res".to_string(), json!(17)),
            ("pseudo.total_cold_res".to_string(), json!(20)),
            ("pseudo.total_lightning_res".to_string(), json!(17)),
            ("pseudo.total_chaos_res".to_string(), json!(22)),
        ]
    );
    assert_eq!(
        read_by(&a, "thread"),
        [
            ("pseudo.count_res".to_string(), json!(3)),
            ("pseudo.total_fire_res".to_string(), json!(-17)),
            ("pseudo.total_cold_res".to_string(), json!(-17)),
            ("pseudo.total_lightning_res".to_string(), json!(-17)),
        ]
    );
    // no item has a count of 0: a comparison on a count it lacks is false,
    // and at most four holds of the thirteen that have one — long, wide
    // and first among them, whatever their open total is
    assert_eq!(asked(&s, "pc", "pseudo.count_res=0")["total"]["matched"], 0);
    assert_eq!(
        asked(&s, "pc", "pseudo.count_res<=4")["total"]["matched"],
        13
    );
    assert_eq!(found("-has:pseudo.count_res"), ["bite", "mana"]);
    assert_eq!(
        found("-has:pseudo.count_ele_res"),
        ["bite", "grasp", "lone", "mana"]
    );
}

/// C94, C101, step 9c4: a row inside a mod of several rows feeds its
/// totals. The 110 readings the trade site's captures hold of the four
/// that are no sum, the sixteen that show none among them, each answered
/// as the site shows it with the capture's mods entered unchanged — a
/// mod's rows in one description, as the site gives them. Thread of Hope
/// (R1) is the one whose total is read from a row, the second of three:
/// written apart, the fixture above it hid that no total read it (the
/// outside review of step 9c3, finding 1).
#[test]
fn c94_a_row_inside_a_mod_of_several_rows_feeds_its_totals() {
    let captures: Value = serde_json::from_str(include_str!(
        "../../../search/pseudo-stats/data/captures.json"
    ))
    .unwrap();
    let names = [
        ("pseudo_count_resistances", "count_res"),
        ("pseudo_count_elemental_resistances", "count_ele_res"),
        (
            "pseudo_total_all_elemental_resistances",
            "total_all_ele_res",
        ),
        ("pseudo_total_all_attributes", "total_all_attributes"),
    ];
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t", "Capture")]), 10);
    let mut bodies = Vec::new();
    let mut expected: Vec<(String, &str, Option<f64>)> = Vec::new();
    let mut of_several_rows = 0;
    for key in [
        "R1", "g001", "g002", "g003", "g004", "h001", "h002", "h039", "h040",
    ] {
        for (i, capture) in captures["searches"][key]["fetched"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let id = format!("{key}_{i}");
            let mut more = json!({});
            for (array, lines) in capture["lines"].as_object().unwrap() {
                if array != "pseudoMods" {
                    of_several_rows += lines
                        .as_array()
                        .unwrap()
                        .iter()
                        .filter(|l| l["description"].as_str().unwrap().contains('\n'))
                        .count();
                    more[array] = lines.clone();
                }
            }
            bodies.push(item(
                &id,
                capture["name"].as_str().unwrap_or(""),
                capture["baseType"].as_str().unwrap_or("Iron Ring"),
                capture["rarity"].as_str().unwrap_or("Rare"),
                more,
            ));
            for (site, name) in names {
                let hash = format!("stat.pseudo.{site}");
                let shown = capture["lines"]["pseudoMods"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|line| line["hash"] == hash);
                if let Some(line) = shown {
                    let text = line["description"].as_str().unwrap();
                    let number = text
                        .split_whitespace()
                        .next()
                        .unwrap()
                        .trim_end_matches('%')
                        .parse::<f64>()
                        .unwrap();
                    expected.push((id.clone(), name, Some(number)));
                // the two searches that asked for items showing none
                } else if (key == "h002" && name == "total_all_attributes")
                    || (key == "h040" && name == "count_ele_res")
                {
                    expected.push((id.clone(), name, None));
                }
            }
        }
    }
    assert_eq!(expected.len(), 110);
    assert_eq!(expected.iter().filter(|(_, _, n)| n.is_none()).count(), 16);
    // the input holds what the test is of: a mod of several rows
    assert!(of_several_rows > 0);
    fetch_tab(&mut s, "pc", "Standard", "t", "Capture", bodies, 20);
    let corpus = load(&s, Some("pc"));
    let matched = |query: &str| as_json(&ask(&corpus, query).unwrap())["total"]["matched"].clone();
    let mut misses = Vec::new();
    for (id, name, n) in &expected {
        let query = match n {
            Some(n) => format!("id:{id} pseudo.{name}={n}"),
            None => format!("id:{id} -has:pseudo.{name}"),
        };
        if matched(&query) != 1 {
            misses.push(query);
        }
    }
    assert!(misses.is_empty(), "{misses:#?}");
    // Thread of Hope's, by hand: -17 under each of the three, so both
    // counts are 3 and the least is -17
    assert_eq!(
        captures["searches"]["R1"]["fetched"][0]["name"],
        "Thread of Hope"
    );
    assert_eq!(
        matched(
            "id:R1_0 pseudo.count_res=3 pseudo.count_ele_res=3 pseudo.total_all_ele_res=-17 pseudo.total_fire_res=-17 pseudo.total_res=-51"
        ),
        1
    );
}

/// C101, step 9c3: the site's `total to all Elemental Resistances` and
/// `total to all Attributes` are the least of their three totals, shown
/// where the item shows all three — with the line that names them or
/// without it (g003, h039; g004, h001) — and an item one of whose totals
/// is nothing shows none (h002: 18 to all Attributes beside -18
/// Dexterity).
#[test]
fn c101_a_least_is_the_smallest_of_its_totals_where_the_item_shows_all() {
    let s = reading_stash();
    let found = |query: &str| ids(&asked(&s, "pc", query));
    assert_eq!(found("pseudo.total_all_ele_res=16"), ["spiral"]);
    assert_eq!(found("pseudo.total_all_ele_res=14"), ["gamble"]);
    assert_eq!(found("pseudo.total_all_ele_res=17"), ["crest"]);
    assert_eq!(found("pseudo.total_all_ele_res=8"), ["entropy"]);
    // a least below nothing is a least
    assert_eq!(found("pseudo.total_all_ele_res<0"), ["thread"]);
    assert_eq!(found("pseudo.total_all_attributes=15"), ["clasp"]);
    assert_eq!(found("pseudo.total_all_attributes=18"), ["bite"]);
    assert_eq!(found("has:pseudo.total_all_attributes"), ["bite", "clasp"]);

    let a = asked(&s, "pc", "pseudo.total_all_ele_res>=14");
    let term = &a["terms"][0];
    // crest 17, gamble 14, spiral 16; entropy 8, thread -17; wide's
    // lightning is open; the ten others lack a total of the three
    assert_eq!(counts(term), [3, 2, 10, 1]);
    assert_eq!(follow(&s, "pc", &term["failed"]), ["entropy", "thread"]);
    assert_eq!(
        follow(&s, "pc", &term["lacked"]),
        [
            "bite", "clasp", "first", "grasp", "knot", "lone", "long", "mana", "turn", "twostone"
        ]
    );
    assert_eq!(follow(&s, "pc", &term["undecided"]), ["wide"]);
    assert_eq!(
        read_by(&a, "spiral"),
        [
            ("pseudo.total_all_ele_res".to_string(), json!(16)),
            ("pseudo.total_fire_res".to_string(), json!(16)),
            ("pseudo.total_cold_res".to_string(), json!(23)),
            ("pseudo.total_lightning_res".to_string(), json!(38)),
        ]
    );
    // knot carries the line and one attribute's lines sum to nothing
    let a = asked(&s, "pc", "pseudo.total_all_attributes>=1");
    assert_eq!(counts(&a["terms"][0]), [2, 0, 14, 0]);
    assert!(
        follow(&s, "pc", &a["terms"][0]["lacked"]).contains(&"knot".to_string()),
        "a total of nothing among the three"
    );
}

/// C93, C94, rule 8 of the plan: what a reading says where a total it
/// reads is open. A count is an interval, as a count of three-valued
/// terms is (C93's at-least-N-of): a comparison is decided where the
/// whole interval agrees, the count is had where one total is, and its
/// value is open while the interval is more than one number. A least is
/// lacked where one total is known to be nothing, whatever is open
/// beside it, and open otherwise. In a realm the table does not cover
/// each is unavailable, with the table's reason.
#[test]
fn c93_a_reading_is_open_by_no_more_than_the_totals_it_reads() {
    let s = reading_stash();
    let found = |query: &str| ids(&asked(&s, "pc", query));
    // long shows fire and cold, and its chaos total is open: two or three
    let said_of = |id: &str, query: &str| -> &str {
        let a = asked(&s, "pc", query);
        ["matched", "failed", "lacked", "undecided"]
            .into_iter()
            .find(|outcome| members(&s, &a["terms"][0][*outcome]).contains(&id.to_string()))
            .unwrap()
    };
    assert_eq!(said_of("long", "pseudo.count_res>=2"), "matched");
    assert_eq!(said_of("long", "pseudo.count_res>=3"), "undecided");
    assert_eq!(said_of("long", "pseudo.count_res>=4"), "failed");
    assert_eq!(said_of("long", "pseudo.count_res<=3"), "matched");
    assert_eq!(said_of("long", "pseudo.count_res=2"), "undecided");
    assert_eq!(said_of("long", "has:pseudo.count_res"), "matched");
    // lone shows none, and its chaos total is open: none, or one — no
    // count is known to be there, so nothing on it is decided
    for query in [
        "has:pseudo.count_res",
        "pseudo.count_res>=1",
        "pseudo.count_res<=1",
        "pseudo.count_res>=2",
    ] {
        assert_eq!(said_of("lone", query), "undecided", "{query}");
    }
    // first shows cold, and its fire total is open, read before the
    // lightning total that is nothing: one or two, and no least
    assert_eq!(said_of("first", "pseudo.count_res>=1"), "matched");
    assert_eq!(said_of("first", "pseudo.count_ele_res>=2"), "undecided");
    assert_eq!(said_of("first", "pseudo.count_res>=3"), "failed");
    assert_eq!(said_of("first", "pseudo.total_all_ele_res>=1"), "lacked");
    assert_eq!(said_of("first", "has:pseudo.total_all_ele_res"), "lacked");
    // a row shows the interval, then the totals established
    let a = asked(&s, "pc", "pseudo.count_res>=2");
    assert_eq!(
        read_by(&a, "long"),
        [
            ("pseudo.count_res".to_string(), json!("2..3")),
            ("pseudo.total_fire_res".to_string(), json!(20)),
            ("pseudo.total_cold_res".to_string(), json!(20)),
        ]
    );
    assert_eq!(found("id:long pseudo.count_res=2..3"), ["long"]);
    assert_eq!(found("id:long has:pseudo.count_res"), ["long"]);
    assert_eq!(
        found("undecided(pseudo.count_res)"),
        ["first", "lone", "long", "wide"]
    );
    // its chaos line is none of the three: the other count is closed
    assert_eq!(found("id:long pseudo.count_ele_res=2"), ["long"]);
    assert_eq!(found("undecided(pseudo.count_ele_res)"), ["first", "wide"]);
    // and its lightning total is nothing, known: no least, whatever chaos is
    assert_eq!(found("id:long -has:pseudo.total_all_ele_res"), ["long"]);
    assert_eq!(found("undecided(pseudo.total_all_ele_res)"), ["wide"]);
    let a = asked(&s, "pc", "pseudo.total_all_ele_res>=1");
    let why = &a["total"]["undecided_reasons"][0];
    assert_eq!(why["example"]["id"], "wide");
    assert_eq!(why["unread"], "the numbers of explicit lines");

    // the count is the term its totals' `has:` make under `holds`, from
    // one up — but on lone, where the count may be none: `holds` is false
    // of a bound it cannot reach, and the count's term, which is failed of
    // a count and lacked of none, is open as a link group's is
    for n in 1..=4 {
        let of = "has:pseudo.total_fire_res, has:pseudo.total_cold_res, has:pseudo.total_lightning_res, has:pseudo.total_chaos_res";
        for (count, held) in [
            (
                format!("pseudo.count_res>={n}"),
                format!("holds({of})>={n}"),
            ),
            (format!("pseudo.count_res={n}"), format!("holds({of})={n}")),
            (
                format!("pseudo.count_res<={n}"),
                format!("holds({of})=1..{n}"),
            ),
        ] {
            let (count, held) = (asked(&s, "pc", &count), asked(&s, "pc", &held));
            assert_eq!(ids(&count), ids(&held), "{}", count["query"]["text"]);
            let open = |a: &Value| -> Vec<String> {
                members(&s, &a["total"]["undecided"])
                    .into_iter()
                    .filter(|id| id != "lone")
                    .collect()
            };
            assert_eq!(open(&count), open(&held), "{}", count["query"]["text"]);
            assert!(
                members(&s, &count["total"]["undecided"]).contains(&"lone".to_string()),
                "{}",
                count["query"]["text"]
            );
        }
    }

    // the sort: a value first, then what is no value — an open count with
    // what was established, an open least with the least established
    let sorted = |by: &str| -> Vec<(String, Value)> {
        view(&s, "", json!({ "rows": { "sort": by, "desc": true } })).unwrap()["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| (r["id"].as_str().unwrap().to_string(), scalar(&r["sort"])))
            .collect()
    };
    // what a row shows of its sort begins with the reading by its own
    // name, as a field's does, and then the totals it read (F6): the text
    // takes the first named value for the sort's own
    let a = view(
        &s,
        "id:crest",
        json!({ "rows": { "sort": "pseudo.total_all_ele_res" } }),
    )
    .unwrap();
    assert_eq!(
        a["rows"][0]["sort"]["shows"],
        json!([
            { "value": { "name": "pseudo.total_all_ele_res", "value": 17 } },
            { "value": { "name": "pseudo.total_fire_res", "value": 17 } },
            { "value": { "name": "pseudo.total_cold_res", "value": 20 } },
            { "value": { "name": "pseudo.total_lightning_res", "value": 17 } },
        ])
    );
    let a = view(
        &s,
        "id:long",
        json!({ "rows": { "sort": "pseudo.count_res" } }),
    )
    .unwrap();
    assert_eq!(
        a["rows"][0]["sort"]["shows"],
        json!([
            { "value": { "name": "pseudo.count_res", "value": "2..3" } },
            { "value": { "name": "pseudo.total_fire_res", "value": 20 } },
            { "value": { "name": "pseudo.total_cold_res", "value": 20 } },
        ])
    );
    let a = view(
        &s,
        "id:mana",
        json!({ "rows": { "sort": "pseudo.count_res" } }),
    )
    .unwrap();
    assert!(
        a["rows"][0]["sort"].get("shows").is_none(),
        "{}",
        a["rows"][0]
    );
    let rows = sorted("pseudo.count_res");
    assert_eq!(rows[0], ("crest".to_string(), json!({ "value": 4 })));
    let of =
        |rows: &[(String, Value)], id: &str| rows.iter().find(|(i, _)| i == id).unwrap().1.clone();
    assert_eq!(
        of(&rows, "long"),
        json!({ "value": 2, "status": "incomplete" })
    );
    assert_eq!(of(&rows, "mana")["status"], "no satisfying occurrence");
    let rows = sorted("pseudo.total_all_ele_res");
    assert_eq!(rows[0], ("crest".to_string(), json!({ "value": 17 })));
    assert_eq!(
        of(&rows, "wide"),
        json!({ "value": 20, "status": "incomplete" })
    );
    assert_eq!(of(&rows, "long")["status"], "no satisfying occurrence");

    // a sum beside a count: 4 + 3 × 4 + 2 × 3 + 1 × 2, and what was
    // established of the four left open — long's and wide's two each,
    // first's one, lone's none — marked
    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["rarity"], "sum": "pseudo.count_res" } }),
    )
    .unwrap();
    assert_eq!(
        a["view"]["counts"]["sum"],
        json!({ "name": "pseudo.count_res", "value": 29, "lacking": 2, "incomplete": true, "unread": 4 })
    );

    // no definition for the realm: never a count, never a least
    for reading in [
        "pseudo.count_res",
        "pseudo.count_ele_res",
        "pseudo.total_all_ele_res",
        "pseudo.total_all_attributes",
    ] {
        let p2 = asked(&s, "poe2", &format!("{reading}>=1"));
        assert_eq!(counts(&p2["terms"][0]), [0, 0, 0, 1], "{reading}");
        let why = &p2["total"]["undecided_reasons"];
        assert_eq!(why.as_array().unwrap().len(), 1, "{reading}");
        assert_eq!(
            why[0]["unread"], "the total: no definition for the realm",
            "{reading}"
        );
        assert_eq!(
            counts(&asked(&s, "poe2", &format!("has:{reading}"))["terms"][0]),
            [0, 0, 0, 1],
            "{reading}"
        );
    }
}

/// C97, C101: a reading is listed with its definition, the site's name
/// for it and the totals it reads; it takes no slot word, and the
/// vocabulary lists it where its name or definition is asked for.
#[test]
fn c97_a_reading_is_described_by_the_totals_it_reads() {
    let one = serde_json::to_value(describe(&["count_res".to_string()]).unwrap()).unwrap();
    assert_eq!(one["computed"].as_array().unwrap().len(), 1);
    assert_eq!(one["computed"][0]["name"], "pseudo.count_res");
    assert_eq!(one["computed"][0]["kind"], "derived");
    assert_eq!(
        one["computed"][0]["what"],
        "the trade site's pseudo_count_resistances (`# total Resistances`): how many of pseudo.total_fire_res, pseudo.total_cold_res, pseudo.total_lightning_res and pseudo.total_chaos_res the item has, each as its own definition counts it; an item with none of them lacks it"
    );
    assert_eq!(
        one["computed"][0]["examples"],
        json!([
            "pseudo.count_res>=3",
            "-has:pseudo.count_res",
            "undecided(pseudo.count_res)"
        ])
    );
    let one =
        serde_json::to_value(describe(&["total_all_attributes".to_string()]).unwrap()).unwrap();
    assert_eq!(
        one["computed"][0]["what"],
        "the trade site's pseudo_total_all_attributes (`+# total to all Attributes`): the least of pseudo.total_str, pseudo.total_dex and pseudo.total_int on an item that has every one of them, each as its own definition counts it; an item lacking any of them lacks it"
    );
    assert_eq!(
        one["computed"][0]["examples"],
        json!([
            "pseudo.total_all_attributes>=10",
            "-has:pseudo.total_all_attributes",
            "undecided(pseudo.total_all_attributes)"
        ])
    );
    let s = reading_stash();
    let corpus = load(&s, Some("pc"));
    for text in ["pseudo.count_res.avg>=1", "pseudo.total_all_ele_res.low>=1"] {
        let e = ask(&corpus, text).unwrap_err().to_json();
        assert_eq!(e["kind"], "slot_unknown", "`{text}`: {e}");
    }
    // the vocabulary: the two counts by their name, each with the matches
    // that have it and a route that returns exactly them
    let a = view(&s, "", json!({ "counts": { "keys": ["line:count_"] } })).unwrap();
    let computed: Vec<(&str, u64)> = a["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["bucket"] == "computed")
        .map(|b| (b["value"].as_str().unwrap(), b["count"].as_u64().unwrap()))
        .collect();
    // thirteen have the one — every item but bite and mana, which show no
    // resistance, and lone, which may show none — and twelve the other,
    // grasp's resistance being chaos: long, wide and first have theirs
    // whatever their open total is
    assert_eq!(
        computed,
        [("pseudo.count_res", 13), ("pseudo.count_ele_res", 12)]
    );
    for bucket in a["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
    {
        if bucket["value"] == "pseudo.count_ele_res" {
            assert_eq!(bucket["term"], "has:pseudo.count_ele_res");
            assert_eq!(
                follow(&s, "pc", bucket),
                [
                    "clasp", "crest", "entropy", "first", "gamble", "knot", "long", "spiral",
                    "thread", "turn", "twostone", "wide"
                ]
            );
        }
    }
}
