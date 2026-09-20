//! C92, C93, C100; surface invariant 4 and build-plan rule 5.
//! Generated count/route checks from the outside audits, for step 4b.
//! Only Request -> answer -> JSON is inspected; fixture bodies enter by
//! Store::record. No binder, evaluator, selector or route builder is an oracle.
//!
//! This checks consistency, not semantic equivalence or completion: an
//! evaluator and its routes can agree on a wrong classification. Step 4b's
//! other properties and the hand-counted regressions remain necessary.

mod common;

use std::collections::{BTreeSet, HashMap};

use acquisition_search::{Corpus, Request, answer};
use acquisition_store::{Endpoint, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

type Ids = BTreeSet<String>;

fn group() -> BoxedStrategy<String> {
    let leaf = prop_oneof![
        proptest::sample::select(vec![
            "template:life",
            "template:resistance",
            "template:Frozen",
            "template:absent",
            "template~\"life|resistance\"",
            "source=explicit",
            "source=implicit",
            "source=hybrid",
            "is:crafted",
            "is:fractured",
            "-is:crafted",
            "true()",
            "false()",
        ])
        .prop_map(str::to_string),
        (
            proptest::sample::select(vec![">=", ">", "<=", "<", "="]),
            -5i32..101
        )
            .prop_map(|(op, n)| format!("arg1{op}{n}")),
    ];
    leaf.prop_recursive(3, 18, 3, |inner| {
        prop_oneof![
            inner.clone().prop_map(|a| format!("-({a})")),
            inner.clone().prop_map(|a| format!("--({a})")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a} and {b})")),
            (inner.clone(), inner).prop_map(|(a, b)| format!("({a} or {b})")),
        ]
    })
    .boxed()
}

fn query() -> BoxedStrategy<String> {
    let leaf = prop_oneof![
        group().prop_map(|g| format!("line({g})")),
        (group(), -5i32..101).prop_map(|(g, n)| format!("sum(line({g}).arg1)>={n}")),
        group().prop_map(|g| format!("undecided(line({g}).arg1)")),
        group().prop_map(|g| format!("undecided(sum(line({g}).arg1))")),
        proptest::sample::select(vec![
            "rarity=rare",
            "base:ring",
            "ilvl>=80",
            "has:ilvl",
            "has:note",
            "is:corrupted",
            "is:shaper",
            "undecided(ilvl)",
            "\"life\"",
            "true()",
            "false()",
        ])
        .prop_map(str::to_string),
    ];
    leaf.prop_recursive(2, 8, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(|a| format!("-({a})")),
            inner.clone().prop_map(|a| format!("undecided({a})")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a}) and ({b})")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a}) or ({b})")),
            (inner.clone(), inner).prop_map(|(a, b)| format!("holds({a}, {b})=1")),
        ]
    })
    .boxed()
}

fn flag() -> impl Strategy<Value = Value> {
    proptest::sample::select(vec![json!(false), json!(true), json!("unread")])
}

fn lines() -> BoxedStrategy<Value> {
    let line = (0u8..4, -5i32..101, flag(), flag()).prop_map(|(kind, n, crafted, fractured)| {
        let description = match kind {
            0 => format!("{n} to maximum Life"),
            1 => format!("{n}% to Cold Resistance"),
            2 => format!("Adds {n} to {} Cold Damage", n + 10),
            _ => "Cannot be Frozen".to_string(),
        };
        json!({"description": description, "flags": {"crafted": crafted, "fractured": fractured}})
    });
    prop_oneof![
        1 => Just(json!("unread")),
        5 => proptest::collection::vec(line, 0..4).prop_map(Value::Array),
    ]
    .boxed()
}

fn body() -> impl Strategy<Value = Value> {
    (lines(), lines(), flag(), flag(), 0u8..3).prop_map(
        |(explicit, implicit, corrupted, shaper, n)| {
            json!({"explicitMods": explicit, "implicitMods": implicit,
            "corrupted": corrupted, "influences": {"shaper": shaper, "hunter": "unread"},
            "ilvl": ([json!(84), json!(null), json!("unread")][usize::from(n)])})
        },
    )
}

fn anchors() -> Vec<Value> {
    vec![
        json!({"explicitMods": ["+95 to maximum Life"]}),
        json!({"explicitMods": ["+20 to maximum Life", "+75 to maximum Life"]}),
        json!({}),
        json!({"implicitMods": "unread"}),
        json!({"explicitMods": ["+20 to maximum Life",
            {"description": "+95 to maximum Life", "flags": "unread"}]}),
        json!({"explicitMods": [{"description": "Cannot be Frozen", "flags": "unread"}],
            "hybrid": "unread"}),
    ]
}

fn fixture(bodies: Vec<Value>) -> (Corpus, Ids) {
    let mut store = Store::open_memory().unwrap();
    store
        .record(
            &Endpoint::Profile,
            &json!({}),
            200,
            &json!({"uuid": "route-account", "name": "Route test"}),
            1,
        )
        .unwrap();
    common::list_tabs(
        &mut store,
        "pc",
        "Standard",
        json!([common::tab("t", "Routes")]),
        2,
    );
    let ids: Ids = (0..bodies.len()).map(|n| format!("i{n}")).collect();
    let items = bodies
        .into_iter()
        .enumerate()
        .map(|(n, body)| common::item(&format!("i{n}"), "", "Ring", "Rare", body))
        .collect();
    common::fetch_tab(&mut store, "pc", "Standard", "t", "Routes", items, 3);
    // The scope oracle excludes this item; it comes from fixture IDs, never
    // from the evaluator's classification or the corpus's internal items.
    common::list_tabs(
        &mut store,
        "poe2",
        "Standard",
        json!([common::tab("other", "Other")]),
        4,
    );
    common::fetch_tab(
        &mut store,
        "poe2",
        "Standard",
        "other",
        "Other",
        vec![common::item("outside", "", "Ring", "Rare", json!({}))],
        5,
    );
    (common::load(&store, Some("pc")), ids)
}

fn run(corpus: &Corpus, request: &Value) -> Result<Value, String> {
    let request: Request = serde_json::from_value(request.clone()).map_err(|e| e.to_string())?;
    let answer = answer(corpus, &request).map_err(|e| e.to_string())?;
    serde_json::to_value(answer).map_err(|e| e.to_string())
}

fn count(value: &Value) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("not a count: {value}"))
}

/// Follow every count, including together and root/zero undecided routes.
/// Change only the row limit to observe the full membership, not just the
/// default page of twenty. The original query and scope are kept verbatim.
fn route_ids(
    counted: &Value,
    scope: &Ids,
    follow: &mut impl FnMut(&Value) -> Result<Value, String>,
    cache: &mut HashMap<String, Ids>,
) -> Result<Ids, String> {
    let expected = count(&counted["count"])?;
    let Some(request) = counted.get("request") else {
        return if expected == 0 {
            Ok(Ids::new())
        } else {
            Err(format!("nonzero count has no route: {counted}"))
        };
    };
    let mut request = request.clone();
    request["view"]["rows"]["limit"] = json!(scope.len().max(1));
    let key = request.to_string();
    let ids = if let Some(ids) = cache.get(&key) {
        ids.clone()
    } else {
        let response = follow(&request).map_err(|e| format!("route refused: {request}: {e}"))?;
        let rows = response["rows"]
            .as_array()
            .ok_or("route has no rows array")?;
        let ids: Ids = rows
            .iter()
            .map(|row| {
                row["id"]
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("row has no id: {row}"))
            })
            .collect::<Result<_, _>>()?;
        if ids.len() != rows.len() || ids.len() != count(&response["total"]["matched"])? {
            return Err(format!(
                "route rows are duplicated or incomplete: {request}: {response}"
            ));
        }
        if !ids.is_subset(scope) {
            return Err(format!("route leaves scope: {request}: {ids:?}"));
        }
        cache.insert(key, ids.clone());
        ids
    };
    if ids.len() != expected {
        return Err(format!(
            "route count mismatch: expected {expected}, got {}: {counted}",
            ids.len()
        ));
    }
    Ok(ids)
}

fn check_routes(
    document: &Value,
    scope: &Ids,
    follow: &mut impl FnMut(&Value) -> Result<Value, String>,
) -> Result<(), String> {
    fn walk(
        value: &Value,
        scope: &Ids,
        follow: &mut impl FnMut(&Value) -> Result<Value, String>,
        cache: &mut HashMap<String, Ids>,
    ) -> Result<(), String> {
        match value {
            Value::Object(fields) => {
                if fields.contains_key("count") {
                    route_ids(value, scope, follow, cache)?;
                }
                for child in fields.values() {
                    walk(child, scope, follow, cache)?;
                }
            }
            Value::Array(children) => {
                for child in children {
                    walk(child, scope, follow, cache)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut cache = HashMap::new();
    walk(document, scope, follow, &mut cache)?;
    let terms = document["terms"]
        .as_array()
        .ok_or("answer has no terms array")?;
    // All queries this generator makes are nonempty, even on an empty
    // scope. Losing the terms block must not make the property vacuous.
    if terms.is_empty() {
        return Err("nonempty query has no terms".to_string());
    }
    for term in terms {
        let mut partition = Ids::new();
        for kind in ["matched", "failed", "lacked", "undecided"] {
            let ids = route_ids(&term[kind], scope, follow, &mut cache)?;
            if !partition.is_disjoint(&ids) {
                return Err(format!("term routes overlap at {kind}: {term}"));
            }
            partition.extend(ids);
        }
        if &partition != scope {
            return Err(format!(
                "term routes do not cover scope: {term}: {partition:?} != {scope:?}"
            ));
        }
    }
    Ok(())
}

proptest! {
    // Fixed gate cost; failed cases shrink and print here rather than writing
    // an unrequested regression file. Retain a reported seed/case on failure.
    #![proptest_config(ProptestConfig { cases: 512, failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn c93_generated_routes_partition_the_scope(
        a in group(), b in group(), c in group(), outer in query(),
        extra in proptest::collection::vec(body(), 0..7), empty in 0u8..12,
    ) {
        let mut bodies = if empty == 0 { Vec::new() } else { anchors() };
        if empty != 0 { bodies.extend(extra); }
        let (corpus, scope) = fixture(bodies);
        // The two original audit shapes, plus item-level composition, sums
        // and probes. Every generated query is intended to be valid: no
        // rejected input is silently skipped with prop_assume or filtering.
        for query in [format!("line(({a} and {b}) or {c})"),
            format!("line({a} and -({b} or {c}))"), outer] {
            let request = json!({"scope": {"realm": "pc"}, "query": {"text": query}});
            let document = run(&corpus, &request).map_err(TestCaseError::fail)?;
            check_routes(&document, &scope, &mut |r| run(&corpus, r))
                .map_err(|e| TestCaseError::fail(format!("query {query}: {e}")))?;
        }
    }
}

/// Negative controls on real boundary output: the checker must reject each
/// fault, including equal-sized overlapping routes (counts alone miss it).
#[test]
fn the_route_checker_rejects_refusal_miscounts_missing_routes_and_overlap() {
    let (corpus, scope) = fixture(vec![
        json!({"explicitMods": ["+95 to maximum Life"]}),
        json!({"explicitMods": ["+20 to maximum Life"]}),
        json!({}),
        json!({"implicitMods": "unread"}),
    ]);
    let document = run(
        &corpus,
        &json!({"query": {"text": "line(template:life arg1>=90)"}}),
    )
    .unwrap();
    let check = |v: &Value| check_routes(v, &scope, &mut |r| run(&corpus, r));
    check(&document).unwrap();
    for kind in ["matched", "failed", "lacked", "undecided"] {
        assert_eq!(document["terms"][0][kind]["count"], 1);
    }
    let mut bad = document.clone();
    bad["terms"][0]["matched"]["request"]["query"] = json!({"text": "not_a_field=1"});
    assert!(check(&bad).unwrap_err().contains("route refused"));
    let mut bad = document.clone();
    bad["terms"][0]["matched"]["count"] = json!(2);
    assert!(check(&bad).unwrap_err().contains("count mismatch"));
    let mut bad = document.clone();
    bad["terms"][0]["matched"]
        .as_object_mut()
        .unwrap()
        .remove("request");
    assert!(check(&bad).unwrap_err().contains("no route"));
    let mut bad = document.clone();
    bad["terms"][0]["failed"]["request"] = document["terms"][0]["matched"]["request"].clone();
    assert!(check(&bad).unwrap_err().contains("overlap"));
    let mut bad = document.clone();
    bad["terms"][0]["failed"] = json!({"count": 0});
    assert!(check(&bad).unwrap_err().contains("do not cover"));
    let mut bad = document;
    bad["terms"] = json!([]);
    assert!(check(&bad).unwrap_err().contains("no terms"));
}

#[test]
fn route_membership_is_checked_beyond_the_default_page() {
    let (corpus, scope) = fixture(vec![json!({}); 24]);
    let document = run(&corpus, &json!({"query": {"text": "true()"}})).unwrap();
    assert_eq!(document["rows"].as_array().unwrap().len(), 20);
    check_routes(&document, &scope, &mut |r| run(&corpus, r)).unwrap();
}
