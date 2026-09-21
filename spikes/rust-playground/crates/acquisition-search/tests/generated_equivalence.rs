//! Invariant 7 of the surface (`search/DESIGN.md`): parentheses group and
//! do nothing else — a rewrite that changes no meaning changes no match, no
//! count, no error and no route, only the canonical text and the paths.
//! Step 4b's equivalence property (the build plan), over generated queries
//! and generated items, through the crate's boundary alone: a request in,
//! an answer out, as JSON.
//!
//! A query is generated as a tree of the test's own (`common::generated`)
//! and asked three ways: as written; with every nested and and or
//! flattened, every doubled not cancelled and every shorthand spelled out;
//! and with the members of every and, or and `holds` in another order. The
//! three answers must be one answer once what may differ is set aside: the
//! canonical text, the paths, and the text of a route — a route is followed
//! and compared by the ids it returns.
//!
//! What this cannot see: a fault every spelling shares. That is the
//! hand-counted tests' and the completion property's.

mod common;

use std::collections::HashMap;

use acquisition_search::Corpus;
use common::generated::*;
use proptest::prelude::*;
use serde_json::{Value, json};

/// An answer with what a rewrite may change set aside: each path replaced
/// by its term's key, the authored texts dropped, each route replaced by
/// the ids it returns, and the blocks a reordering reorders sorted.
fn normalized(
    corpus: &Corpus,
    scope: &Ids,
    answer: &Value,
    keys: &[String],
) -> Result<Value, String> {
    let paths: Vec<&str> = answer["terms"]
        .as_array()
        .ok_or("an answer with no terms block")?
        .iter()
        .filter_map(|t| t["path"].as_str())
        .collect();
    if paths.len() != keys.len() {
        return Err(format!(
            "the generator's terms and the answer's differ in number: {keys:?} against {paths:?}"
        ));
    }
    let by_path: HashMap<&str, &str> = paths
        .iter()
        .copied()
        .zip(keys.iter().map(String::as_str))
        .collect();
    let mut cache = HashMap::new();
    let mut out = answer.clone();
    let fields = out.as_object_mut().ok_or("an answer that is no object")?;
    fields.remove("query");
    if let Some(rows) = fields
        .get_mut("view")
        .and_then(|v| v.get_mut("rows"))
        .and_then(Value::as_object_mut)
    {
        rows.remove("sort");
    }
    walk(&mut out, corpus, scope, &by_path, &mut cache)?;
    Ok(out)
}

fn walk(
    value: &mut Value,
    corpus: &Corpus,
    scope: &Ids,
    by_path: &HashMap<&str, &str>,
    cache: &mut HashMap<String, Ids>,
) -> Result<(), String> {
    match value {
        Value::Object(fields) => {
            if let Some(path) = fields.get("path").and_then(Value::as_str) {
                let key = by_path
                    .get(path)
                    .ok_or_else(|| format!("a path no term has: {path}"))?;
                fields.insert("path".into(), json!(key));
                fields.remove("term");
            }
            if fields.contains_key("count") {
                let ids = members(corpus, &Value::Object(fields.clone()), scope, cache)?;
                fields.remove("request");
                fields.insert("ids".into(), json!(ids));
            }
            // a sum's evidence is named by its authored text
            if fields.contains_key("value")
                && fields
                    .get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|name| name.starts_with("sum("))
            {
                fields.insert("name".into(), json!("sum"));
            }
            for (name, child) in fields.iter_mut() {
                walk(child, corpus, scope, by_path, cache)?;
                // what follows the order the terms were written in
                let Value::Array(entries) = child else {
                    continue;
                };
                // the reasons an `undecided( … )` shows follow it too; the
                // lines a term shows follow the item, and stay as they are
                let reasons =
                    name == "shows" && entries.iter().all(|e| e.get("undecided").is_some());
                if reasons
                    || matches!(
                        name.as_str(),
                        "terms" | "matched" | "why" | "resolved_to_nothing"
                    )
                {
                    entries.sort_by_key(Value::to_string);
                }
            }
        }
        Value::Array(entries) => {
            for entry in entries {
                walk(entry, corpus, scope, by_path, cache)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// One way of writing the query, asked: the answer normalized, or the
/// error.
fn asked(
    corpus: &Corpus,
    scope: &Ids,
    q: &Q,
    sort: &Sort,
    desc: bool,
    spelling: Spelling,
) -> Result<Result<Value, Value>, String> {
    let text = q_text(q, spelling);
    let sort = sort_text(sort, spelling);
    let request = request(&text, sort.as_deref(), desc, 50);
    match run(corpus, &request) {
        Err(error) => Ok(Err(error)),
        Ok(answer) => {
            // transformation 11: the returned tree, sent back, is the same
            // request — the whole answer, text and paths too
            let mut by_tree = request.clone();
            by_tree["query"] = json!({ "tree": answer["query"]["tree"] });
            let again = run(corpus, &by_tree)
                .map_err(|e| format!("`{text}`: its returned tree is refused: {e}"))?;
            if again != answer {
                return Err(format!(
                    "`{text}`: its returned tree, sent back, is answered differently"
                ));
            }
            let keys: Vec<String> = terms(q).into_iter().map(|(key, _)| key).collect();
            normalized(corpus, scope, &answer, &keys)
                .map(Ok)
                .map_err(|e| format!("`{text}`: {e}"))
        }
    }
}

fn differs(a: &Value, b: &Value) -> String {
    let (Some(a), Some(b)) = (a.as_object(), b.as_object()) else {
        return format!("{a}\n  against\n{b}");
    };
    a.iter()
        .filter(|(name, value)| b.get(*name) != Some(value))
        .map(|(name, value)| format!("`{name}`: {value}\n  against\n{}", b[name]))
        .collect::<Vec<_>>()
        .join("\n")
}

fn check(
    corpus: &Corpus,
    scope: &Ids,
    q: &Q,
    sort: &Sort,
    desc: bool,
    seed: u64,
) -> Result<(), String> {
    let written = asked(corpus, scope, q, sort, desc, Spelling::Authored)?;
    let text = q_text(q, Spelling::Authored);

    // parentheses, a doubled not, a shorthand: the same answer or the same
    // error, word for word
    let plain = normal(q);
    let plain_text = q_text(&plain, Spelling::Explicit);
    let flattened = asked(
        corpus,
        scope,
        &plain,
        &sort_normal(sort),
        desc,
        Spelling::Explicit,
    )?;
    match (&written, &flattened) {
        (Ok(a), Ok(b)) if a == b => {}
        (Err(a), Err(b)) if a == b => {}
        (Ok(a), Ok(b)) => {
            return Err(format!(
                "`{text}` and `{plain_text}` are answered differently:\n{}",
                differs(a, b)
            ));
        }
        (a, b) => {
            return Err(format!(
                "`{text}` and `{plain_text}`: {} against {}",
                a.as_ref()
                    .map_or_else(Value::to_string, |_| "an answer".into()),
                b.as_ref()
                    .map_or_else(Value::to_string, |_| "an answer".into()),
            ));
        }
    }

    // another order: the same answer; of two errors the first met is
    // reported, so an order may change which, never whether
    let moved = shuffled(q, seed);
    let moved_text = q_text(&moved, Spelling::Authored);
    let reordered = asked(
        corpus,
        scope,
        &moved,
        &sort_shuffled(sort, seed),
        desc,
        Spelling::Authored,
    )?;
    match (&written, &reordered) {
        (Ok(a), Ok(b)) if a == b => Ok(()),
        (Err(_), Err(_)) => Ok(()),
        (Ok(a), Ok(b)) => Err(format!(
            "`{text}` and `{moved_text}` are answered differently:\n{}",
            differs(a, b)
        )),
        (a, b) => Err(format!(
            "`{text}` and `{moved_text}`: {} against {}",
            a.as_ref()
                .map_or_else(Value::to_string, |_| "an answer".into()),
            b.as_ref()
                .map_or_else(Value::to_string, |_| "an answer".into()),
        )),
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(256), failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn invariant_7_a_rewrite_that_changes_no_meaning_changes_no_answer(
        q in query(true), sort in sort(), desc in any::<bool>(), seed in any::<u64>(),
        bodies in proptest::collection::vec(body(true), 0..7), anchored in any::<bool>(),
    ) {
        let mut all = if anchored { anchors() } else { Vec::new() };
        all.extend(bodies.iter().map(Body::json));
        let (corpus, scope) = fixture(all);
        check(&corpus, &scope, &q, &sort, desc, seed).map_err(TestCaseError::fail)?;
    }
}
