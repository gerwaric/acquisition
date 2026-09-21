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
//!   Both ways: a selector with nothing beside it in the terms block is in
//!   the zero block, so an empty zero block is no way to pass. And a
//!   suggestion's count is what its term returns (invariant 4).
//! - **A row's evidence is the item's** (C100, C103): every line a row
//!   shows is a line `show` derives of that item — source, flags and text —
//!   and what a row shows of one term is bounded (invariant 5): six lines
//!   or strings, or the reasons of six unread parts.
//! - **Every occurrence in another order** (C92): no term moves, and no
//!   sum and no largest — decimals among the numbers.
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

fn zero_block_is_terms_block(corpus: &Corpus, answer: &Value) -> Result<(), String> {
    let Some(nothing) = answer["zero"]["resolved_to_nothing"].as_array() else {
        return Ok(());
    };
    let terms = answer["terms"].as_array().ok_or("no terms block")?;
    for entry in nothing {
        let term = terms
            .iter()
            .find(|t| t["path"] == entry["path"])
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
        for suggestion in entry["suggestions"].as_array().ok_or("no suggestions")? {
            let text = suggestion["term"]
                .as_str()
                .ok_or("a suggestion with no term")?;
            let asked = run(corpus, &request(text, None, false, 1))
                .map_err(|e| format!("the suggestion `{text}` is refused: {e}"))?;
            if asked["total"]["matched"] != suggestion["items"] {
                return Err(format!(
                    "the suggestion `{text}` says {} and returns {}",
                    suggestion["items"], asked["total"]["matched"]
                ));
            }
        }
    }
    // and the other way: a selector the terms block shows resolving to no
    // value is said to in the zero block
    for term in terms {
        let empty = term["resolved"]["values"]
            .as_array()
            .is_some_and(Vec::is_empty);
        if empty && !nothing.iter().any(|entry| entry["path"] == term["path"]) {
            return Err(format!(
                "{} resolved to no value and the zero block does not say so",
                term["term"]
            ));
        }
    }
    Ok(())
}

/// A reason is of something its term asked: a line's unread flags explain
/// a term that asks a flag, its unread numbers one that asks a number.
/// (Which occurrence a reason is of is pinned by hand, `fifth_audit.rs`.)
fn a_reason_is_of_what_its_term_asked(answer: &Value) -> Result<(), String> {
    fn walk(value: &Value, out: &mut Vec<Value>) {
        match value {
            Value::Object(fields) => {
                if fields.contains_key("unread") && fields.contains_key("term") {
                    out.push(value.clone());
                }
                fields.values().for_each(|child| walk(child, out));
            }
            Value::Array(entries) => entries.iter().for_each(|child| walk(child, out)),
            _ => {}
        }
    }
    let mut reasons = Vec::new();
    walk(answer, &mut reasons);
    for reason in reasons {
        let (unread, term) = (
            reason["unread"].as_str().unwrap_or_default(),
            reason["term"].as_str().unwrap_or_default(),
        );
        let asks_a_number = ["arg", "low", "high", "avg"]
            .iter()
            .any(|w| term.contains(w));
        if (unread.starts_with("the flags of") && !term.contains("is:"))
            || (unread.starts_with("the numbers of") && !asks_a_number)
        {
            return Err(format!(
                "{term} is explained by {unread}, which it never asked"
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
            // six lines or strings, and a sum's value beside them; of
            // reasons, six unread parts — every term's pair with each, so
            // the parts are counted and never the entries
            let shows = touched["shows"].as_array().ok_or("no shows")?;
            let parts: std::collections::BTreeSet<String> = shows
                .iter()
                .filter_map(|e| e.get("undecided"))
                .map(|r| format!("{} {}", r["unread"], r["problem"]))
                .collect();
            let others = shows
                .iter()
                .filter(|e| e.get("undecided").is_none())
                .count();
            if others > 7 || parts.len() > 6 {
                return Err(format!("row {id} shows more than its bound: {touched}"));
            }
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

fn every_occurrence_in_another_order(
    bodies: &[Body],
    q: &Q,
    projection: &str,
    sum: &str,
) -> Result<(), String> {
    let reversed: Vec<Body> = bodies.iter().map(Body::every_line_reversed).collect();
    let (as_given, scope) = fixture(bodies.iter().map(Body::json).collect());
    let (other, _) = fixture(reversed.iter().map(Body::json).collect());
    let text = q_text(q, Spelling::Authored);
    let ask = |corpus: &Corpus| run(corpus, &request(&text, None, false, scope.len().max(1)));
    if let (Ok(a), Ok(b)) = (ask(&as_given), ask(&other)) {
        let (a, b) = (
            term_members(&as_given, &scope, &a)?,
            term_members(&other, &scope, &b)?,
        );
        if a != b {
            return Err(format!(
                "`{text}`: a term moved when the occurrences were written in another order: {a:?} against {b:?}"
            ));
        }
    }
    for value in [projection, sum] {
        let (a, b) = (
            scalars(&as_given, &scope, value),
            scalars(&other, &scope, value),
        );
        if a != b {
            return Err(format!(
                "`--sort {value}`: {a:?} and, the occurrences in another order, {b:?}"
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
                zero_block_is_terms_block(&corpus, &answer).map_err(TestCaseError::fail)?;
                evidence_is_the_items(&store, &answer).map_err(TestCaseError::fail)?;
                a_reason_is_of_what_its_term_asked(&answer).map_err(TestCaseError::fail)?;
            }
        }
        // a generated query seldom ends in a zero block with suggestions in
        // it, so two misspellings are asked of every corpus: the life line,
        // which the generators spell two ways, and a name
        for misspelt in ["line(template:lifes)", "sum(line(template:lifes).arg1)>0", "name:knott"] {
            let answer = run(&corpus, &request(misspelt, None, false, 50)).map_err(|e| TestCaseError::fail(e.to_string()))?;
            zero_block_is_terms_block(&corpus, &answer).map_err(TestCaseError::fail)?;
        }
        every_item_twice(&all, &text, Some(&sum)).map_err(TestCaseError::fail)?;
        every_occurrence_twice(&bodies, &q, &projection, &sum).map_err(TestCaseError::fail)?;
        every_occurrence_in_another_order(&bodies, &q, &projection, &sum)
            .map_err(TestCaseError::fail)?;
    }
}

/// The checker's own negative controls, on a real answer: an empty zero
/// block, a suggestion miscounted and an oversized row must each be
/// refused — the fifth audit found the first of these passing.
#[test]
fn the_checks_refuse_an_empty_zero_block_a_miscounted_suggestion_and_an_unbounded_row() {
    let many: Vec<String> = (1..=9).map(|n| format!("+{n} to maximum Life")).collect();
    let (store, corpus, _) = fixture_with_store(vec![json!({ "explicitMods": many })]);
    let answer = run(&corpus, &request("line(template:lifes)", None, false, 10)).unwrap();
    zero_block_is_terms_block(&corpus, &answer).unwrap();
    assert_eq!(
        answer["zero"]["resolved_to_nothing"][0]["suggestions"][0]["items"],
        1
    );
    let mut bad = answer.clone();
    bad["zero"]["resolved_to_nothing"] = json!([]);
    assert!(
        zero_block_is_terms_block(&corpus, &bad)
            .unwrap_err()
            .contains("does not say so")
    );
    let mut bad = answer.clone();
    bad["zero"]["resolved_to_nothing"][0]["suggestions"][0]["items"] = json!(2);
    assert!(
        zero_block_is_terms_block(&corpus, &bad)
            .unwrap_err()
            .contains("says 2 and returns 1")
    );
    // many terms resting on one unread part are one part, however many
    // pairs: the bound counts parts, and this is within it
    let (one_part, probed, _) = fixture_with_store(vec![json!({ "corrupted": "unread" })]);
    let eight = ["is:corrupted"; 8].join(" ");
    let probe = run(
        &probed,
        &request(&format!("undecided({eight})"), None, false, 10),
    )
    .unwrap();
    assert_eq!(
        probe["rows"][0]["matched"][0]["shows"]
            .as_array()
            .unwrap()
            .len(),
        8
    );
    evidence_is_the_items(&one_part, &probe).unwrap();
    let answer = run(&corpus, &request("line(template:life)", None, false, 10)).unwrap();
    evidence_is_the_items(&store, &answer).unwrap();
    let mut bad = answer.clone();
    let shows = bad["rows"][0]["matched"][0]["shows"].clone();
    let twice: Vec<Value> = shows
        .as_array()
        .unwrap()
        .iter()
        .chain(shows.as_array().unwrap())
        .cloned()
        .collect();
    bad["rows"][0]["matched"][0]["shows"] = json!(twice);
    assert!(
        evidence_is_the_items(&store, &bad)
            .unwrap_err()
            .contains("more than its bound")
    );
}
