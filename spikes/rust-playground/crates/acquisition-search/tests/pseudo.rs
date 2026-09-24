//! The computed values (`search/BUILD-PLAN.md`, step 7; C94, C101, C93,
//! C95), through the crate's boundary: the shipped totals table, a total's
//! three statuses and its routes, `dps` and `pdps`, the sort and the sum
//! over a computed value, `--describe`, and every authoring error a
//! computed value has. Every count is worked by hand from the fixture
//! below.

mod common;

use acquisition_search::{Request, TOTALS_TABLE_VERSION, answer, describe, show};
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
/// with its route and its reason — and a total of nothing is an honest
/// zero, never lacked.
#[test]
fn c94_a_total_has_three_statuses_and_a_total_of_nothing_is_zero() {
    let s = stash();
    let a = asked(&s, "pc", "pseudo.total_res>=60");
    let term = &a["terms"][0];
    // ring_a 124; ring_c, sword, wand and odd 0; ring_b a subtotal of 30
    assert_eq!(counts(term), [1, 4, 0, 1]);
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
        follow(&s, "pc", &term["failed"]),
        ["odd", "ring_c", "sword", "wand"]
    );
    assert_eq!(follow(&s, "pc", &term["undecided"]), ["ring_b"]);
    let why = &a["total"]["undecided_items"][0];
    assert_eq!(why["id"], "ring_b");
    assert_eq!(
        (&why["why"][0]["term"], &why["why"][0]["unread"]),
        (&json!("pseudo.total_res>=60"), &json!("implicit lines"))
    );
    // `undecided( … )` of a total is decided itself, and finds the same item
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(pseudo.total_res)")),
        ["ring_b"]
    );
    assert_eq!(asked(&s, "pc", "pseudo.total_res=0")["total"]["matched"], 4);
    assert_eq!(
        counts(&asked(&s, "pc", "pseudo.total_fire_res=72")["terms"][0]),
        [1, 4, 0, 1]
    );

    // no definition for the realm: never zero, undecided with that reason
    let p2 = asked(&s, "poe2", "pseudo.total_res>=1");
    assert_eq!(counts(&p2["terms"][0]), [0, 0, 0, 1]);
    let why = &p2["total"]["undecided_items"][0];
    assert_eq!(why["id"], "p2ring");
    assert_eq!(
        why["why"][0]["unread"],
        "the total: no definition for the realm"
    );
    assert_eq!(
        why["why"][0]["problem"],
        "no totals table for realm poe2 (totals v1 covers pc, xbox, sony)"
    );
    assert_eq!(
        why["why"][0]["hint"],
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
        serde_json::to_value(show(&s, "ring_a", false).unwrap()).unwrap()["basis"]["totals"],
        TOTALS_TABLE_VERSION
    );
}

/// C101: `pdps` and `dps` read the properties as displayed; an item
/// lacking the property lacks the field — counted, with no route the
/// language can say yet — and one whose property is no number is undecided
/// with that reason; the routes there are partition the rest.
#[test]
fn c101_dps_and_pdps_read_the_displayed_properties() {
    let s = stash();
    let a = asked(&s, "pc", "pseudo.pdps>=90");
    let term = &a["terms"][0];
    // sword 91.875; the three rings and the wand lack it; odd cannot be read
    assert_eq!(counts(term), [1, 0, 4, 1]);
    assert!(
        term["lacked"].get("request").is_none(),
        "no `has:` on a computed value: a lacked count has no route (the plan, an unruled hole)"
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
    let why = &a["total"]["undecided_items"][0];
    assert_eq!(why["id"], "odd");
    assert_eq!(why["why"][0]["unread"], "`properties`");
    assert_eq!(
        why["why"][0]["problem"],
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
    let shown = serde_json::to_value(show(&s, "ele", false).unwrap()).unwrap();
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
    // the totals whose definition names a fire-resistance line: ring_a
    // carries each, ring_b's subtotal is incomplete, the rest are zero
    assert_eq!(
        computed,
        [
            ("pseudo.total_fire_res", 1),
            ("pseudo.total_ele_res", 1),
            ("pseudo.total_res", 1)
        ]
    );
    let row = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["value"] == "pseudo.total_res")
        .unwrap();
    assert_eq!(row["term"], "pseudo.total_res>0 or pseudo.total_res<0");
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
    let a = view(&s, "", json!({ "counts": { "keys": ["line"] } })).unwrap();
    assert!(
        a["view"]["counts"]["tables"][0]["buckets"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["bucket"] != "computed")
    );
}

/// Outside audit, 2026-09-24 (5): a reason made beyond the item's parts —
/// a derived field's malformed property — is a part of its own under the
/// six-part bound, and what the bound leaves out is counted: five unread
/// arrays and four malformed properties are nine parts, six shown.
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
            "Rusted Sword",
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
    let a = asked(&s, "pc", "pseudo.total_res>=1 pseudo.dps>=1");
    let item = &a["total"]["undecided_items"][0];
    assert_eq!(item["id"], "nine");
    let parts: Vec<String> = item["why"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["problem"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(parts.len(), 6, "{parts:?}");
    assert_eq!(item["why_left_out"], 3);
    // the item's own parts first, in its order, then the reasons made
    // beyond them in the order met
    assert!(parts[0].starts_with("`craftedMods`"));
    assert!(parts[4].starts_with("`implicitMods`"));
    assert_eq!(
        parts[5],
        "`Attacks per Second` is `fast`: no number the search reads"
    );
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
        .map(|r| (r["id"].as_str().unwrap().to_string(), r["sort"].clone()))
        .collect();
    assert_eq!(rows[0], ("ring_a".to_string(), json!({ "value": 124 })));
    assert_eq!(
        rows[5],
        (
            "ring_b".to_string(),
            json!({ "value": 30, "status": "incomplete" })
        )
    );
    assert!(
        rows[1..5]
            .iter()
            .all(|(_, sort)| *sort == json!({ "value": 0 }))
    );

    let a = view(
        &s,
        "rarity=rare",
        json!({ "rows": { "sort": "pseudo.dps", "desc": true } }),
    )
    .unwrap();
    let rows: Vec<(&str, &Value)> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), &r["sort"]))
        .collect();
    assert_eq!(rows[0], ("sword", &json!({ "value": 281.875 })));
    assert_eq!(rows[1], ("wand", &json!({ "value": 76.3 })));
    // the rest last, in the store's order: odd with what was readable
    // (nothing), the rings lacking the field
    assert_eq!(rows[2], ("odd", &json!({ "status": "incomplete" })));
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
    // 124 and ring_b's readable 30; the rest zero
    assert_eq!(
        a["view"]["counts"]["sum"],
        json!({ "name": "pseudo.total_res", "value": 154, "lacking": 0, "incomplete": true, "unread": 1 })
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
    assert_eq!(computed.len(), 37);
    assert_eq!(computed[0]["name"], "pseudo.total_cold_res");
    assert_eq!(computed[35]["name"], "pseudo.dps");
    assert_eq!(computed[36]["kind"], "derived");
    assert!(
        whole["totals"]
            .as_str()
            .unwrap()
            .starts_with("totals v1, 35 totals over pc, xbox, sony: the C++ app"),
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
        json!(["pseudo.total_res>=60", "undecided(pseudo.total_res)"])
    );
    let block = serde_json::to_value(describe(&["pseudo".to_string()]).unwrap()).unwrap();
    assert_eq!(block["computed"].as_array().unwrap().len(), 37);
    // what the reference names and no step builds is refused by that name
    let e = describe(&["defence_pct".to_string()]).unwrap_err();
    assert!(
        e.message
            .starts_with("not built: pseudo.defence_pct (no step of the plan)")
    );
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
