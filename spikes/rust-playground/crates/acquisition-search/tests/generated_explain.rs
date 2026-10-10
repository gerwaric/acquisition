//! Explain over generated queries and generated items (`answer.rs`,
//! `tree.rs`; the build plan, step 10), through the crate's boundary
//! alone. Nothing explain answers is new arithmetic: a forced node is the
//! query with that node replaced by `true()` or `false()`, which a user
//! could type. So, at every path the answer names:
//!
//! - each forced answer has the counts the build gives the transformed
//!   query it prints, asked as a request of its own — by its text and by
//!   its tree;
//! - the printed text parses to the answered tree with that one node
//!   replaced and nothing else changed — the replacement made here, on
//!   the answer's JSON, by this file's own reading of what a path names;
//! - a respelling that changes no meaning (the equivalence property's
//!   ways: parentheses and shorthand undone, another order) changes no
//!   explain count at the node that corresponds, by the members each
//!   count routes to.
//!
//! What this cannot see: a fault the evaluator has on a query a user
//! types, which explain inherits. That is every other property's.

mod common;

use std::collections::HashMap;

use acquisition_search::Corpus;
use common::generated::*;
use proptest::prelude::*;
use serde_json::{Value, json};

fn child(parent: &str, n: usize) -> String {
    if parent.is_empty() {
        n.to_string()
    } else {
        format!("{parent}.{n}")
    }
}

/// Every path of a returned tree, by this file's own reading of the
/// rule (`tree.rs`): the root's children are `0`, `1`, …, a root that is no and
/// is `0`; a child of an and, an or or a `holds` is `<parent>.<n>`; what a
/// not negates or `undecided( … )` asks about is `<parent>.0`; inside
/// `line( … )` or `linked( … )` the group's conjuncts are `<term>.<n>`, a
/// group of one condition `<term>.0`, and so on down.
fn paths(tree: &Value) -> Vec<String> {
    fn node(tree: &Value, path: &str, member: bool, out: &mut Vec<String>) {
        out.push(path.to_string());
        for key in ["all", "any", "holds"] {
            if let Some(children) = tree.get(key).and_then(Value::as_array) {
                for (n, c) in children.iter().enumerate() {
                    node(c, &child(path, n), member, out);
                }
            }
        }
        if let Some(inner) = tree.get("not") {
            node(inner, &child(path, 0), member, out);
        }
        if let Some(inner) = tree.get("undecided").and_then(|u| u.get("term")) {
            node(inner, &child(path, 0), member, out);
        }
        if let (false, Some(group)) = (member, tree.get("where")) {
            match group.get("all").and_then(Value::as_array) {
                Some(conjuncts) => {
                    for (n, c) in conjuncts.iter().enumerate() {
                        node(c, &child(path, n), true, out);
                    }
                }
                None => node(group, &child(path, 0), true, out),
            }
        }
    }
    let mut out = Vec::new();
    match tree.get("all").and_then(Value::as_array) {
        Some(children) => {
            for (n, c) in children.iter().enumerate() {
                node(c, &n.to_string(), false, &mut out);
            }
        }
        None => node(tree, "0", false, &mut out),
    }
    out
}

/// The tree with the node at `path` replaced by a constant, and nothing
/// else changed.
fn replaced(tree: &Value, path: &str, value: bool) -> Value {
    fn walk(tree: &mut Value, here: &str, member: bool, path: &str, value: bool) {
        if here == path {
            *tree = json!({ "const": value });
            return;
        }
        for key in ["all", "any", "holds"] {
            if let Some(children) = tree.get_mut(key).and_then(Value::as_array_mut) {
                for (n, c) in children.iter_mut().enumerate() {
                    walk(c, &child(here, n), member, path, value);
                }
            }
        }
        if let Some(inner) = tree.get_mut("not") {
            walk(inner, &child(here, 0), member, path, value);
        }
        if let Some(inner) = tree.get_mut("undecided").and_then(|u| u.get_mut("term")) {
            walk(inner, &child(here, 0), member, path, value);
        }
        if member {
            return;
        }
        if let Some(group) = tree.get_mut("where") {
            if let Some(conjuncts) = group.get_mut("all").and_then(Value::as_array_mut) {
                for (n, c) in conjuncts.iter_mut().enumerate() {
                    walk(c, &child(here, n), true, path, value);
                }
            } else {
                walk(group, &child(here, 0), true, path, value);
            }
        }
    }
    let mut out = tree.clone();
    if let Some(children) = out.get_mut("all").and_then(Value::as_array_mut) {
        for (n, c) in children.iter_mut().enumerate() {
            walk(c, &n.to_string(), false, path, value);
        }
    } else {
        walk(&mut out, "0", false, path, value);
    }
    out
}

fn explain(query: &str, path: &str) -> Value {
    json!({
        "scope": { "realm": "pc" },
        "query": { "text": query },
        "view": { "explain": { "path": path } },
    })
}

/// What explain says at one path, by members: the three pairs of counts,
/// each forced count followed by its route.
fn counts(
    corpus: &Corpus,
    scope: &Ids,
    explained: &Value,
    cache: &mut HashMap<String, Ids>,
) -> Result<Value, String> {
    let block = &explained["view"]["explain"];
    let mut out = json!({ "given": block["given"] });
    for which in ["forced_true", "forced_false"] {
        let forced = &block[which];
        out[which] = json!({
            "matched": members(corpus, &forced["matched"], scope, cache)?,
            "undecided": members(corpus, &forced["undecided"], scope, cache)?,
        });
    }
    Ok(out)
}

/// The two properties of one spelling, and what explain says at each of
/// its atomic terms, by the term's key.
fn check_one(corpus: &Corpus, scope: &Ids, q: &Q, spelling: Spelling) -> Result<Said, String> {
    let text = q_text(q, spelling);
    let answer = match run(corpus, &request(&text, None, false, 50)) {
        Ok(answer) => answer,
        // an authoring error: explain is refused as the query is
        Err(error) => {
            let refused = run(corpus, &explain(&text, "0"))
                .err()
                .ok_or_else(|| format!("`{text}` is refused, and explained"))?;
            if refused["kind"] != error["kind"] {
                return Err(format!(
                    "`{text}` is refused as {error} and its explain as {refused}"
                ));
            }
            return Ok(None);
        }
    };
    let tree = &answer["query"]["tree"];
    let canonical = answer["query"]["text"]
        .as_str()
        .ok_or("no canonical text")?;
    let all = paths(tree);
    // the nodes the build lists are the nodes this file reads
    let listed: Vec<String> = match run(corpus, &explain(&text, "no such path")) {
        Err(error) if error["kind"] == "no_such_path" => error["readings"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| r.as_str()?.split("  ").next().map(str::to_string))
            .collect(),
        other => {
            return Err(format!(
                "`{text}`: a path no query has is answered: {other:?}"
            ));
        }
    };
    if listed != all {
        return Err(format!(
            "`{canonical}`: the build lists the paths {listed:?} and the tree has {all:?}"
        ));
    }
    let mut cache = HashMap::new();
    let mut said: HashMap<String, Value> = HashMap::new();
    let term_paths: Vec<&str> = answer["terms"]
        .as_array()
        .ok_or("no terms block")?
        .iter()
        .filter_map(|t| t["path"].as_str())
        .collect();
    for path in &term_paths {
        if !all.iter().any(|p| p == path) {
            return Err(format!("`{canonical}`: the term at {path} has no node"));
        }
    }
    let keys: Vec<String> = terms(q).into_iter().map(|(key, _)| key).collect();
    if keys.len() != term_paths.len() {
        return Err(format!(
            "`{canonical}`: the generator's terms and the answer's differ in number"
        ));
    }
    for path in &all {
        let explained = match run(corpus, &explain(&text, path)) {
            Ok(explained) => explained,
            // a substitution the binder refuses is refused with the
            // binder's own error: the one the transformed tree gets,
            // asked as a request of its own
            Err(error) => {
                let own = |value: bool| {
                    run(
                        corpus,
                        &json!({ "scope": { "realm": "pc" }, "query": { "tree": replaced(tree, path, value) } }),
                    )
                    .err()
                    .ok_or_else(|| {
                        format!("`{canonical}` at {path}: refused, and its transformed tree is answered: {error}")
                    })
                };
                // the first substitution made is the one refused
                if own(true)? != error || own(false)?["kind"] != error["kind"] {
                    return Err(format!(
                        "`{canonical}` at {path}: refused as {error}, its transformed tree otherwise"
                    ));
                }
                continue;
            }
        };
        let block = &explained["view"]["explain"];
        if explained["query"]["tree"] != *tree || block["path"] != **path {
            return Err(format!(
                "`{canonical}` at {path}: another query or path is explained"
            ));
        }
        if block["given"]["matched"] != answer["total"]["matched"]
            || block["given"]["undecided"] != answer["total"]["undecided"]["count"]
        {
            return Err(format!(
                "`{canonical}` at {path}: as given is not the answer's total"
            ));
        }
        for (which, value) in [("forced_true", true), ("forced_false", false)] {
            let forced = &block[which];
            let printed = forced["query"]["text"]
                .as_str()
                .ok_or("no transformed text")?;
            // the tree with that one node replaced, and nothing else changed
            let expected = replaced(tree, path, value);
            if forced["query"]["tree"] != expected {
                return Err(format!(
                    "`{canonical}` at {path}, {which}: the transformed tree is {} and the node replaced gives {expected}",
                    forced["query"]["tree"]
                ));
            }
            // asked as a request of its own: by its text, which parses to
            // that tree, and by the tree
            let by_text = run(corpus, &request(printed, None, false, 50))
                .map_err(|e| format!("`{printed}` is printed, and refused: {e}"))?;
            if by_text["query"]["tree"] != expected {
                return Err(format!(
                    "`{printed}` parses to {} and the node replaced gives {expected}",
                    by_text["query"]["tree"]
                ));
            }
            let by_tree = run(
                corpus,
                &json!({ "scope": { "realm": "pc" }, "query": { "tree": expected } }),
            )
            .map_err(|e| format!("`{printed}`: its tree is refused: {e}"))?;
            for own in [&by_text, &by_tree] {
                if own["total"]["matched"] != forced["matched"]["count"]
                    || own["total"]["undecided"]["count"] != forced["undecided"]["count"]
                {
                    return Err(format!(
                        "`{canonical}` at {path}, {which}: explain says {} · {} and `{printed}` asked alone says {} · {}",
                        forced["matched"]["count"],
                        forced["undecided"]["count"],
                        own["total"]["matched"],
                        own["total"]["undecided"]["count"]
                    ));
                }
            }
        }
        if let Some(at) = term_paths.iter().position(|p| p == path) {
            said.insert(
                keys[at].clone(),
                counts(corpus, scope, &explained, &mut cache)
                    .map_err(|e| format!("`{canonical}` at {path}: {e}"))?,
            );
        }
    }
    // a key two terms share names no one node
    let mut seen: HashMap<&String, usize> = HashMap::new();
    for key in &keys {
        *seen.entry(key).or_default() += 1;
    }
    said.retain(|key, _| seen.get(key) == Some(&1));
    Ok(Some(said))
}

type Said = Option<HashMap<String, Value>>;

/// Whether two spellings are explained alike: both refused — of two
/// errors an order may change which, never whether — or the same terms,
/// each with the same counts by their members.
fn same(a: &Said, b: &Said) -> Result<(), String> {
    let (Some(a), Some(b)) = (a, b) else {
        return if a.is_some() == b.is_some() {
            Ok(())
        } else {
            Err("one is refused".to_string())
        };
    };
    for (key, said) in a {
        match b.get(key) {
            Some(other) if other == said => {}
            other => {
                return Err(format!(
                    "`{key}` is explained differently:\n{said}\n  against\n{other:?}"
                ));
            }
        }
    }
    if a.len() != b.len() {
        return Err("different terms are explained".to_string());
    }
    Ok(())
}

fn check(corpus: &Corpus, scope: &Ids, q: &Q, seed: u64) -> Result<(), String> {
    let written = check_one(corpus, scope, q, Spelling::Authored)?;
    let text = q_text(q, Spelling::Authored);
    let plain = normal(q);
    let flattened = check_one(corpus, scope, &plain, Spelling::Explicit)?;
    let moved = shuffled(q, seed);
    let reordered = check_one(corpus, scope, &moved, Spelling::Authored)?;
    for (other, other_text) in [
        (&flattened, q_text(&plain, Spelling::Explicit)),
        (&reordered, q_text(&moved, Spelling::Authored)),
    ] {
        same(&written, other).map_err(|e| format!("`{text}` and `{other_text}`: {e}"))?;
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(48), failure_persistence: None, ..ProptestConfig::default() })]
    #[test]
    fn explain_is_the_transformed_query_asked_at_every_path_and_in_every_spelling(
        q in query(true), seed in any::<u64>(),
        bodies in proptest::collection::vec(body(true), 0..7), anchored in any::<bool>(),
    ) {
        let mut all = if anchored { anchors() } else { Vec::new() };
        all.extend(bodies.iter().map(Body::json));
        let (corpus, scope) = fixture(all);
        check(&corpus, &scope, &q, seed).map_err(TestCaseError::fail)?;
    }
}

/// The generator's reach, a fixed case beside the generated ones: a not,
/// an or, a `holds`, a probe, a line's group with a not inside and a
/// link group, over the anchors — so that every kind of path is explained
/// on every run, and a property that passes has asked something.
#[test]
fn explain_a_fixed_case_reaches_every_kind_of_path() {
    let (corpus, scope) = fixture(anchors());
    let line = |g: G| Q::Line(g);
    let leaf = |text: &str| G::Leaf(text.to_string());
    let q = Q::And(vec![
        Q::Not(Box::new(Q::Or(vec![
            Q::Plain("is:corrupted".into()),
            line(G::And(vec![
                leaf("\"# to maximum Life\""),
                G::Not(Box::new(G::Or(vec![
                    leaf("arg1>=90"),
                    leaf("source=implicit"),
                ]))),
            ])),
        ]))),
        Q::Holds(
            vec![
                Q::Plain("has:sockets".into()),
                Q::Linked(G::Leaf("red>=1".into())),
                Q::Undecided(Box::new(line(leaf("template:life")))),
            ],
            ">=1".into(),
        ),
        Q::Plain("rarity=rare".into()),
    ]);
    let said = check_one(&corpus, &scope, &q, Spelling::Authored)
        .unwrap()
        .unwrap();
    // seven atomic terms, each key its own
    assert_eq!(said.len(), 7, "{said:?}");
    // and something moved: forcing `rarity=rare` false empties the answer
    let rare = said
        .iter()
        .find(|(key, _)| key.contains("rarity"))
        .map(|(_, said)| said)
        .unwrap();
    assert_eq!(rare["forced_false"]["matched"], json!([]));
    assert_ne!(rare["forced_true"]["matched"], json!([]));
    for seed in 1..4 {
        check(&corpus, &scope, &q, seed).unwrap();
    }
    // the comparison's own negative control: the same query over fewer
    // items is explained differently, and is said to be
    let said = Some(said);
    assert!(same(&said, &said).is_ok());
    let (fewer, fewer_scope) = fixture(anchors().into_iter().take(3).collect());
    let other = check_one(&fewer, &fewer_scope, &q, Spelling::Authored).unwrap();
    assert!(same(&said, &other).is_err());
    assert!(same(&said, &None).is_err());
}

/// The negative control of this file's own reading: a path numbered from
/// the wrong child replaces another node, and the two trees differ.
#[test]
fn the_tests_own_replacement_names_one_node() {
    let tree = json!({ "all": [
        { "field": "rarity", "op": "=", "value": "rare" },
        { "exists": "lines", "where": { "all": [
            { "attr": "template", "op": "=", "value": "# to maximum Life" },
            { "not": { "attr": "arg1", "op": ">=", "value": 90 } } ] } },
        { "not": { "any": [ { "is": "corrupted" }, { "has": "note" } ] } } ] });
    assert_eq!(
        paths(&tree),
        [
            "0", "1", "1.0", "1.1", "1.1.0", "2", "2.0", "2.0.0", "2.0.1"
        ]
    );
    let forced = replaced(&tree, "1.1.0", true);
    assert_eq!(
        forced["all"][1]["where"]["all"][1],
        json!({ "not": { "const": true } })
    );
    assert_eq!(forced["all"][0], tree["all"][0]);
    assert_eq!(forced["all"][2], tree["all"][2]);
    assert_eq!(
        replaced(&tree, "2.0.1", false)["all"][2],
        json!({ "not": { "any": [ { "is": "corrupted" }, { "const": false } ] } })
    );
    assert_eq!(replaced(&tree, "7", true), tree);
}
