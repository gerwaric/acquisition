//! A mod displayed over several rows, generated (the build plan, step 9c4;
//! C90, C92, C94, C97), through the crate's boundary: a request in, an
//! answer out, as JSON.
//!
//! - **A row named answers as the row alone.** The items are asked as
//!   GGG gives them, a mod's rows in one description, and again with
//!   every such mod written as its rows, each a line of its own with the
//!   mod's source and flags (`Body::rows_apart`) — the reading the trade
//!   site's evidence was taken by (`tools/trade-evidence.py`), which is
//!   the oracle a row's name has. A term that names one exact template, a
//!   sum of one, a largest of one, a total and a reading of totals return
//!   the same items, count the same four ways and sort by the same
//!   number. What is asked names one template in a group, and never
//!   under a not there: two in one group hold on the mod and on no row
//!   apart, a not of one holds on a row apart of the mod it names, and
//!   `template:` tests the whole text, which the rows apart have none of
//!   — each by the rule that a mod is one occurrence.
//! - **A vocabulary row counts what its term returns** (invariant 4):
//!   every bucket of `--count line`, narrowed or not, and every source and
//!   flag beneath it, routes to as many items as it counted — a template
//!   that names a mod by a row, and one GGG spelled two ways, among them.
//!
//! What this cannot see: a fault the mod and its rows apart share. That
//! is the hand-counted tests' (`tests/rows.rs`, `tests/pseudo.rs`).

mod common;

use std::collections::{BTreeMap, HashMap};

use acquisition_search::Corpus;
use common::generated::*;
use proptest::prelude::*;
use serde_json::{Value, json};

/// A template a row displays, with a slot it has.
fn named() -> BoxedStrategy<(&'static str, &'static str)> {
    proptest::sample::select(vec![
        (LIFE, "arg1"),
        (COLD, "arg1"),
        (FIRE, "arg1"),
        (ALL_RES, "arg1"),
        (SPIRIT, "arg1"),
        (LEECH, "arg1"),
        (ADDS, "low"),
        (ADDS, "avg"),
        (ADDS, "arg2"),
    ])
    .boxed()
}

fn compared() -> BoxedStrategy<String> {
    (
        proptest::sample::select(vec![">=", ">", "<=", "<", "="]),
        -3i32..=14,
    )
        .prop_map(|(op, n)| format!("{op}{n}"))
        .boxed()
}

fn restriction() -> BoxedStrategy<String> {
    proptest::sample::select(vec![
        "",
        " source=explicit",
        " source=implicit",
        " is:crafted",
        " -is:crafted",
        " -is:fractured",
        " (is:crafted or source=hybrid)",
    ])
    .prop_map(str::to_string)
    .boxed()
}

/// One term that names one exact template, or a computed value.
fn term() -> BoxedStrategy<String> {
    let line = (named(), restriction(), compared(), 0u8..5).prop_map(
        |((template, slot), restriction, compared, shape)| match shape {
            0 => format!("line(\"{template}\"{restriction})"),
            1 => format!("line(\"{template}\"{restriction} {slot}{compared})"),
            2 => format!("line(\"{template}\"{restriction} -{slot}{compared})"),
            3 => format!("sum(line(\"{template}\"{restriction}).{slot}){compared}"),
            _ => format!("line(\"{template}\"{restriction}).{slot}{compared}"),
        },
    );
    let computed = (
        proptest::sample::select(vec![
            "pseudo.total_res",
            "pseudo.total_cold_res",
            "pseudo.total_fire_res",
            "pseudo.total_life",
            "pseudo.count_res",
            "pseudo.count_ele_res",
            "pseudo.total_all_ele_res",
        ]),
        compared(),
        0u8..3,
    )
        .prop_map(|(name, compared, shape)| match shape {
            0 => format!("{name}{compared}"),
            1 => format!("has:{name}"),
            _ => format!("undecided({name})"),
        });
    prop_oneof![
        6 => line,
        1 => Just(format!("line(template=\"{PASSAGE}\")")),
        3 => computed,
    ]
    .boxed()
}

fn query() -> BoxedStrategy<String> {
    prop_oneof![
        3 => term(),
        1 => term().prop_map(|t| format!("-{t}")),
        1 => term().prop_map(|t| format!("undecided({t})")),
        1 => (term(), term()).prop_map(|(a, b)| format!("{a} {b}")),
        1 => (term(), term()).prop_map(|(a, b)| format!("({a} or {b})")),
    ]
    .boxed()
}

fn sort() -> BoxedStrategy<String> {
    prop_oneof![
        named().prop_map(|(template, slot)| format!("line(\"{template}\").{slot}")),
        named().prop_map(|(template, slot)| format!("sum(line(\"{template}\").{slot})")),
        proptest::sample::select(vec!["pseudo.total_res", "pseudo.total_all_ele_res"])
            .prop_map(str::to_string),
    ]
    .boxed()
}

/// What an answer says that the rows apart must say too: who matched, how
/// every term counted and whom each count routes to, the together count,
/// and what every item sorts by. Never what a row shows: a mod is shown
/// whole, and its row apart is the row.
fn said(corpus: &Corpus, scope: &Ids, request: &Value) -> Result<Value, String> {
    let a = run(corpus, request).map_err(|e| format!("refused: {request}: {e}"))?;
    let mut cache = HashMap::new();
    let mut terms = Vec::new();
    for term in a["terms"].as_array().ok_or("no terms block")? {
        let mut counted: BTreeMap<&str, Ids> = BTreeMap::new();
        for kind in ["matched", "failed", "lacked", "undecided"] {
            counted.insert(kind, members(corpus, &term[kind], scope, &mut cache)?);
        }
        if term.get("together").is_some_and(|t| !t.is_null()) {
            counted.insert(
                "together",
                members(corpus, &term["together"], scope, &mut cache)?,
            );
        }
        terms.push(json!(counted));
    }
    let rows: Vec<Value> = a["rows"]
        .as_array()
        .ok_or("no rows")?
        .iter()
        .map(|row| {
            json!({
                "id": row["id"],
                "sort": { "value": row["sort"]["value"], "status": row["sort"]["status"] },
            })
        })
        .collect();
    Ok(json!({
        "matched": a["total"]["matched"],
        "undecided": a["total"]["undecided"]["count"],
        "terms": terms,
        "rows": rows,
    }))
}

fn a_row_named_answers_as_the_row_alone(
    bodies: &[Body],
    query: &str,
    sort: Option<&str>,
    desc: bool,
) -> Result<(), String> {
    let whole: Vec<Value> = bodies.iter().map(Body::json).collect();
    let apart: Vec<Value> = bodies.iter().map(|b| b.rows_apart().json()).collect();
    let (whole, scope) = fixture(whole);
    let (apart, _) = fixture(apart);
    let request = request(query, sort, desc, scope.len().max(1));
    let (one, two) = (
        said(&whole, &scope, &request)?,
        said(&apart, &scope, &request)?,
    );
    if one != two {
        return Err(format!(
            "`{query}` sorted by {sort:?}: the mods as given say {one:#}, their rows apart {two:#}"
        ));
    }
    Ok(())
}

/// Every bucket a counts view holds, its kinds beneath it among them.
fn buckets(answer: &Value) -> Vec<&Value> {
    let mut out = Vec::new();
    for table in answer["view"]["counts"]["tables"]
        .as_array()
        .into_iter()
        .flatten()
    {
        for bucket in table["buckets"].as_array().into_iter().flatten() {
            out.push(bucket);
            for kinds in ["sources", "flags"] {
                out.extend(bucket[kinds].as_array().into_iter().flatten());
            }
        }
    }
    out
}

fn a_vocabulary_row_counts_what_its_term_returns(
    corpus: &Corpus,
    scope: &Ids,
    query: &str,
) -> Result<usize, String> {
    let request = json!({
        "scope": { "realm": "pc" },
        "query": { "text": query },
        "view": { "counts": {
            "keys": ["line", "line:life", "line:resist", "line~^#% to", "line:passage"],
            "limit": 50,
        } },
    });
    let a = run(corpus, &request).map_err(|e| format!("refused: {request}: {e}"))?;
    let mut cache = HashMap::new();
    let mut by_a_row = 0;
    for bucket in buckets(&a) {
        // `members` refuses a route that returns another number than was
        // counted
        let ids = members(corpus, bucket, scope, &mut cache)
            .map_err(|e| format!("under `{query}`: {bucket}: {e}"))?;
        if bucket["value"] == ALL_RES || bucket["value"] == COLD {
            by_a_row += ids.len();
        }
    }
    Ok(by_a_row)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: cases(192), failure_persistence: None, ..ProptestConfig::default()
    })]

    #[test]
    fn c90_a_row_named_answers_as_the_row_alone(
        bodies in proptest::collection::vec(body(true), 1..6),
        queries in proptest::collection::vec(query(), 1..4),
        sort in proptest::option::of(sort()),
        desc in any::<bool>(),
    ) {
        for query in &queries {
            a_row_named_answers_as_the_row_alone(&bodies, query, sort.as_deref(), desc)
                .map_err(TestCaseError::fail)?;
        }
    }

    #[test]
    fn c97_a_vocabulary_row_counts_what_its_term_returns(
        bodies in proptest::collection::vec(body(true), 1..6),
        query in prop_oneof![2 => Just(String::new()), 1 => query()],
    ) {
        let (corpus, scope) = fixture(bodies.iter().map(Body::json).collect());
        a_vocabulary_row_counts_what_its_term_returns(&corpus, &scope, &query)
            .map_err(TestCaseError::fail)?;
    }
}

/// One line of every kind at four values, a mod of several rows among
/// them, as given and apart: the fixed case beside what the dice find,
/// which a property that never met such a mod would pass as well.
#[test]
fn the_generators_reach_a_mod_of_several_rows() {
    let lines = Body::every_line(Tri::No);
    let of_several = lines.iter().filter(|l| matches!(l.kind, 9 | 10)).count();
    assert_eq!(of_several, 8);
    let bodies: Vec<Body> = lines
        .chunks(4)
        .enumerate()
        .map(|(n, chunk)| Body::nth(n, chunk.to_vec()))
        .collect();
    let given: Vec<Value> = bodies.iter().map(Body::json).collect();
    let several = given
        .iter()
        .flat_map(|b| {
            ["explicitMods", "implicitMods"]
                .into_iter()
                .flat_map(|array| b[array].as_array().into_iter().flatten())
        })
        .filter(|l| l["description"].as_str().is_some_and(|d| d.contains('\n')))
        .count();
    assert_eq!(several, 8);
    for query in [
        format!("\"{ALL_RES}\">=1"),
        format!("\"{COLD}\">=3"),
        format!("\"{LIFE}\"<=0"),
        format!("sum(\"{COLD}\")>=2"),
        format!("line(template=\"{PASSAGE}\")"),
        "pseudo.total_cold_res>=3".to_string(),
        "pseudo.count_ele_res=3".to_string(),
    ] {
        a_row_named_answers_as_the_row_alone(
            &bodies,
            &query,
            Some(&format!("line(\"{COLD}\").arg1")),
            true,
        )
        .unwrap();
    }
    let (corpus, scope) = fixture(given);
    // the rows the two templates name are counted: more than none
    let by_a_row = a_vocabulary_row_counts_what_its_term_returns(&corpus, &scope, "").unwrap();
    assert!(by_a_row > 0);
}
