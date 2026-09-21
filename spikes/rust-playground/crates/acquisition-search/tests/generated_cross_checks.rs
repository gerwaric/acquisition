//! Two parts of an answer that say the same thing must agree, and a copy
//! of the evidence changes an answer only where the rule says it does: the
//! cross-checks of step 4b (the build plan; the outside agent's
//! transformations 10 and 13, which are neither equivalence nor
//! completion). Through the crate's boundary: a request in, an answer out,
//! as JSON, and `show`.
//!
//! - **A value's probe is its sort status** (C92, C93): `undecided(V)`
//!   matches exactly the items that sorting by `V` marks incomplete.
//! - **The zero block is the terms block** (C100): a selector listed as
//!   resolving to nothing lists no value it resolved to, and one that lists
//!   a value is never said to resolve to nothing.
//! - **A row's evidence is the item's** (C100, C103): every line a row
//!   shows is a line `show` derives of that item — source, flags and text.
//! - **Every item twice** doubles every count and changes no row's
//!   evidence or scalar.
//! - **Every occurrence twice** (C92): a term on a line binds one
//!   occurrence, so no term on a line's group moves; a largest stays; a sum
//!   doubles.

mod common;

use std::collections::{BTreeMap, HashMap};

use acquisition_search::{Corpus, show};
use acquisition_store::Store;
use common::generated::*;
use proptest::prelude::*;
use serde_json::{Value, json};

fn term_members(
    corpus: &Corpus,
    scope: &Ids,
    answer: &Value,
) -> Result<Vec<BTreeMap<&'static str, Ids>>, String> {
    let mut cache = HashMap::new();
    answer["terms"]
        .as_array()
        .ok_or("no terms block")?
        .iter()
        .map(|term| {
            ["matched", "failed", "lacked", "undecided"]
                .into_iter()
                .map(|kind| Ok((kind, members(corpus, &term[kind], scope, &mut cache)?)))
                .collect()
        })
        .collect()
}

/// `id → sort` of every item, by the empty query.
fn scalars(corpus: &Corpus, scope: &Ids, value: &str) -> Option<HashMap<String, Value>> {
    let answer = run(corpus, &request("", Some(value), false, scope.len().max(1))).ok()?;
    Some(
        answer["rows"]
            .as_array()?
            .iter()
            .filter_map(|row| Some((row["id"].as_str()?.to_string(), row["sort"].clone())))
            .collect(),
    )
}

fn probe_is_sort_status(corpus: &Corpus, scope: &Ids, value: &str) -> Result<(), String> {
    let probe = request(
        &format!("undecided({value})"),
        None,
        false,
        scope.len().max(1),
    );
    let (probed, sorted) = (run(corpus, &probe), scalars(corpus, scope, value));
    let (Ok(probed), Some(sorted)) = (&probed, &sorted) else {
        return if probed.is_err() && sorted.is_none() {
            Ok(())
        } else {
            Err(format!(
                "`{value}`: a probe and a sort, and one of them is refused"
            ))
        };
    };
    let open: Ids = probed["rows"]
        .as_array()
        .ok_or("no rows")?
        .iter()
        .filter_map(|row| row["id"].as_str().map(str::to_string))
        .collect();
    let incomplete: Ids = sorted
        .iter()
        .filter(|(_, sort)| sort["status"] == "incomplete")
        .map(|(id, _)| id.clone())
        .collect();
    if open != incomplete {
        return Err(format!(
            "`undecided({value})` matches {open:?} and `--sort {value}` marks {incomplete:?} incomplete"
        ));
    }
    Ok(())
}

fn zero_block_is_terms_block(answer: &Value) -> Result<(), String> {
    let Some(nothing) = answer["zero"]["resolved_to_nothing"].as_array() else {
        return Ok(());
    };
    for entry in nothing {
        let term = answer["terms"]
            .as_array()
            .and_then(|terms| terms.iter().find(|t| t["path"] == entry["path"]))
            .ok_or_else(|| format!("the zero block names a path no term has: {entry}"))?;
        if term["resolved"]["values"]
            .as_array()
            .is_some_and(|values| !values.is_empty())
        {
            return Err(format!(
                "{} resolved to nothing in the zero block and to {} in the terms block",
                entry["term"], term["resolved"]["values"]
            ));
        }
    }
    Ok(())
}

fn evidence_is_the_items(store: &Store, answer: &Value) -> Result<(), String> {
    for row in answer["rows"].as_array().ok_or("no rows")? {
        let id = row["id"].as_str().ok_or("a row with no id")?;
        let shown = serde_json::to_value(show(store, id, false).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        let lines = shown["lines"].as_array().ok_or("show has no lines")?;
        let flags = |line: &Value| line["flags"].clone();
        for touched in row["matched"].as_array().ok_or("a row with no matched")? {
            for evidence in touched["shows"].as_array().ok_or("no shows")? {
                let Some(line) = evidence.get("line") else {
                    continue;
                };
                let found = lines.iter().any(|l| {
                    l["source"] == line["source"]
                        && l["text"] == line["text"]
                        && flags(l) == flags(line)
                });
                if !found {
                    return Err(format!(
                        "row {id} shows {line} under {} and `show` derives no such line: {lines:?}",
                        touched["term"]
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Every count of an answer, by where it sits.
fn counts(value: &Value, at: &str, out: &mut BTreeMap<String, u64>) {
    match value {
        Value::Object(fields) => {
            if let Some(n) = fields.get("count").and_then(Value::as_u64) {
                out.insert(at.to_string(), n);
            }
            for (name, child) in fields {
                if name != "request" && name != "counted_at" {
                    counts(child, &format!("{at}.{name}"), out);
                }
            }
        }
        Value::Array(entries) => {
            for (i, entry) in entries.iter().enumerate() {
                counts(entry, &format!("{at}[{i}]"), out);
            }
        }
        _ => {}
    }
}

fn every_item_twice(bodies: &[Value], text: &str, sort: Option<&str>) -> Result<(), String> {
    let (once, scope) = fixture(bodies.to_vec());
    let (twice, scope_twice) = fixture(bodies.iter().chain(bodies).cloned().collect());
    let ask = |corpus: &Corpus, n: usize| run(corpus, &request(text, sort, false, n.max(1)));
    let (Ok(a), Ok(b)) = (ask(&once, scope.len()), ask(&twice, scope_twice.len())) else {
        return Ok(());
    };
    let (mut one, mut two) = (BTreeMap::new(), BTreeMap::new());
    counts(&a["terms"], "terms", &mut one);
    counts(&a["total"], "total", &mut one);
    counts(&b["terms"], "terms", &mut two);
    counts(&b["total"], "total", &mut two);
    for (at, n) in &one {
        if two.get(at) != Some(&(n * 2)) {
            return Err(format!(
                "`{text}`: {at} is {n} over the items and {:?} over every item twice",
                two.get(at)
            ));
        }
    }
    if b["total"]["matched"].as_u64() != a["total"]["matched"].as_u64().map(|n| n * 2) {
        return Err(format!(
            "`{text}`: the total does not double with the items"
        ));
    }
    // item n and item n + len are one body: one evidence, one scalar
    let rows = |answer: &Value| -> HashMap<String, Value> {
        answer["rows"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|row| {
                Some((
                    row["id"].as_str()?.to_string(),
                    json!([row["matched"], row["sort"]]),
                ))
            })
            .collect()
    };
    let (first, both) = (rows(&a), rows(&b));
    for (n, _) in bodies.iter().enumerate() {
        let (id, copy) = (format!("i{n}"), format!("i{}", n + bodies.len()));
        if first.get(&id) != both.get(&id) || both.get(&id) != both.get(&copy) {
            return Err(format!(
                "`{text}`: {id} and its copy {copy} are shown differently"
            ));
        }
    }
    Ok(())
}

fn every_occurrence_twice(
    bodies: &[Body],
    q: &Q,
    projection: &str,
    sum: &str,
) -> Result<(), String> {
    let doubled: Vec<Body> = bodies.iter().map(Body::every_line_twice).collect();
    let (once, scope) = fixture(bodies.iter().map(Body::json).collect());
    let (twice, _) = fixture(doubled.iter().map(Body::json).collect());
    let text = q_text(q, Spelling::Authored);
    let ask = |corpus: &Corpus| run(corpus, &request(&text, None, false, scope.len().max(1)));
    if let (Ok(a), Ok(b)) = (ask(&once), ask(&twice)) {
        let kinds = terms(q);
        let (a, b) = (
            term_members(&once, &scope, &a)?,
            term_members(&twice, &scope, &b)?,
        );
        for (i, (_, kind)) in kinds.iter().enumerate() {
            if *kind != TermKind::Sum && a.get(i) != b.get(i) {
                return Err(format!(
                    "`{text}`: term {i} moved when every occurrence was written twice: {:?} against {:?}",
                    a.get(i),
                    b.get(i)
                ));
            }
        }
    }
    if let (Some(a), Some(b)) = (
        scalars(&once, &scope, projection),
        scalars(&twice, &scope, projection),
    ) && a != b
    {
        return Err(format!(
            "`--sort {projection}`: a largest moved when every occurrence was written twice"
        ));
    }
    if let (Some(a), Some(b)) = (scalars(&once, &scope, sum), scalars(&twice, &scope, sum)) {
        for (id, was) in &a {
            let now = &b[id];
            let double = |v: &Value| v.as_f64().map(|n| n * 2.0);
            if was["status"] != now["status"] || double(&was["value"]) != now["value"].as_f64() {
                return Err(format!(
                    "`--sort {sum}`: {id} sums to {was} and, every occurrence written twice, to {now}"
                ));
            }
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(192), failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn two_parts_of_an_answer_agree_and_a_copy_moves_only_what_the_rule_says(
        q in query(true), projection in (group(), proptest::sample::select(vec!["arg1", "avg"])),
        sum in (group(), proptest::sample::select(vec!["arg1", "avg"])),
        bodies in proptest::collection::vec(body(true), 0..6),
    ) {
        let projection = sort_text(&Sort::Proj(projection.0, projection.1.to_string()), Spelling::Authored).unwrap();
        let sum = sort_text(&Sort::Sum(sum.0, sum.1.to_string()), Spelling::Authored).unwrap();
        let mut all = anchors();
        all.extend(bodies.iter().map(Body::json));
        let (store, corpus, scope) = fixture_with_store(all.clone());
        let text = q_text(&q, Spelling::Authored);
        for value in [&projection, &sum, &"ilvl".to_string()] {
            probe_is_sort_status(&corpus, &scope, value).map_err(TestCaseError::fail)?;
        }
        for sort in [None, Some(projection.as_str()), Some(sum.as_str())] {
            if let Ok(answer) = run(&corpus, &request(&text, sort, false, 50)) {
                zero_block_is_terms_block(&answer).map_err(TestCaseError::fail)?;
                evidence_is_the_items(&store, &answer).map_err(TestCaseError::fail)?;
            }
        }
        every_item_twice(&all, &text, Some(&sum)).map_err(TestCaseError::fail)?;
        every_occurrence_twice(&bodies, &q, &projection, &sum).map_err(TestCaseError::fail)?;
    }
}
