//! How deep a query nests (C47; `tree::MAX_DEPTH`), through the crate's
//! boundary — a request in, an answer or a refusal out, as JSON. A query
//! nested past what the stack holds once aborted the process, a resident
//! one among them: malformed input that was no structured error. Each
//! `( )`, `not`, `holds( )` and `undecided( )` around a term is a level,
//! inside a line's group as outside it; a query at the limit answers under
//! every view and what the answer prints is accepted back, one level
//! deeper is refused by its kind, and so is one far past every depth that
//! aborted. A test thread's stack is the smallest a search runs on.

mod common;

use acquisition_search::{Corpus, Request, answer};
use common::*;
use serde_json::{Value, json};

/// The limit, written out: a test that read the crate's constant would
/// move with it.
const LIMIT: usize = 32;

/// Past every depth that aborted on the owner's copy, in either build.
const FAR: usize = 10_000;

/// A way of nesting: its name, and its text of exactly `n` levels.
type Shape = (&'static str, fn(usize) -> String);

/// Every way the language nests.
fn shapes() -> Vec<Shape> {
    fn wrapped(open: &str, leaf: &str, close: &str, n: usize) -> String {
        format!("{}{leaf}{}", open.repeat(n), close.repeat(n))
    }
    fn alternating(leaf: &str, n: usize) -> String {
        let opens: String = (0..n)
            .map(|i| format!("(is:corrupted {}", if i % 2 == 0 { "or " } else { "" }))
            .collect();
        format!("{opens}{leaf}{}", ")".repeat(n))
    }
    vec![
        ("parentheses", |n| wrapped("(", "rarity=rare", ")", n)),
        ("not", |n| wrapped("-", "rarity=rare", "", n)),
        ("not of a group", |n| {
            let odd = if n % 2 == 1 { "-" } else { "" };
            format!("{odd}{}", wrapped("-(", "rarity=rare", ")", n / 2))
        }),
        ("and inside or", |n| alternating("rarity=rare", n)),
        ("holds", |n| {
            wrapped("holds(", "rarity=rare", ", is:corrupted)>=1", n)
        }),
        ("undecided", |n| wrapped("undecided(", "ilvl>=3", ")", n)),
        ("parentheses in a line", |n| {
            format!("line({})", wrapped("(", "\"# to maximum Life\"", ")", n))
        }),
        ("not in a line", |n| {
            format!("line({})", wrapped("-", "template:life", "", n))
        }),
        ("and inside or in a line", |n| {
            let opens: String = (0..n)
                .map(|i| format!("(is:crafted {}", if i % 2 == 0 { "or " } else { "" }))
                .collect();
            format!("line({opens}template:life{})", ")".repeat(n))
        }),
    ]
}

fn asked(corpus: &Corpus, request: Value) -> Result<Value, Value> {
    let request: Request = serde_json::from_value(request).unwrap();
    answer(corpus, &request)
        .map(|a| as_json(&a))
        .map_err(|e| e.to_json())
}

fn too_deep(refused: &Value, what: &str) {
    assert_eq!(refused["kind"], "too_deep", "{what}: {refused}");
    let said = refused["error"].as_str().unwrap();
    assert!(
        said.contains(&format!("at most {LIMIT} deep")),
        "{what}: {said}"
    );
}

#[test]
fn c47_every_way_of_nesting_answers_at_the_limit_and_is_refused_one_deeper() {
    let store = worked();
    let corpus = load(&store, Some("pc"));
    for (name, shape) in shapes() {
        let at = shape(LIMIT);
        let rows = asked(&corpus, json!({ "query": { "text": at } }))
            .unwrap_or_else(|e| panic!("{name} at the limit: {e}"));
        for view in [
            json!({ "counts": { "keys": ["rarity", "line:life"] } }),
            json!({ "explain": { "path": "0" } }),
        ] {
            asked(&corpus, json!({ "query": { "text": at }, "view": view }))
                .unwrap_or_else(|e| panic!("{name} at the limit, {view}: {e}"));
        }
        // what the answer prints is a query of this build: its text and
        // its tree are each accepted back, and are the same query
        for query in [
            json!({ "text": rows["query"]["text"] }),
            json!({ "tree": rows["query"]["tree"] }),
        ] {
            let again = asked(&corpus, json!({ "query": query }))
                .unwrap_or_else(|e| panic!("{name} at the limit, sent back as {query}: {e}"));
            assert_eq!(again["query"], rows["query"], "{name}");
            assert_eq!(again["total"], rows["total"], "{name}");
        }
        let deeper =
            asked(&corpus, json!({ "query": { "text": shape(LIMIT + 1) } })).expect_err(name);
        too_deep(&deeper, name);
    }
}

#[test]
fn c47_a_query_nested_far_past_the_limit_is_refused_and_aborts_nothing() {
    let store = worked();
    let corpus = load(&store, Some("pc"));
    for (name, shape) in shapes() {
        let refused = asked(&corpus, json!({ "query": { "text": shape(FAR) } })).expect_err(name);
        too_deep(&refused, name);
    }
}

/// A tree that arrives as JSON is held to the same limit by the same
/// kind: nodes the text would show as levels, counted as the text counts
/// them.
#[test]
fn c47_a_tree_nested_past_the_limit_is_refused_as_the_text_is() {
    let store = worked();
    let corpus = load(&store, Some("pc"));
    let nots = |n: usize| {
        (0..n).fold(
            json!({ "field": "rarity", "op": "=", "value": "rare" }),
            |inner, _| json!({ "not": inner }),
        )
    };
    asked(&corpus, json!({ "query": { "tree": nots(LIMIT) } })).unwrap();
    for n in [LIMIT + 1, 120] {
        let refused = asked(&corpus, json!({ "query": { "tree": nots(n) } })).unwrap_err();
        too_deep(&refused, &format!("{n} nots"));
    }
}

/// A line's own group and a sum are terms, never levels: the commonest
/// term, a template compared, costs nothing, so the limit is the count of
/// what was typed around it.
#[test]
fn c47_a_line_or_a_sum_is_a_term_and_never_a_level() {
    let store = worked();
    let corpus = load(&store, Some("pc"));
    for leaf in [
        "\"# to maximum Life\">=90",
        "line(\"# to maximum Life\" arg1>=90 source=explicit)",
        "sum(\"# to maximum Life\")>=90",
        "pseudo.total_res>=60",
    ] {
        let at = format!("{}{leaf}", "-".repeat(LIMIT));
        asked(&corpus, json!({ "query": { "text": at } }))
            .unwrap_or_else(|e| panic!("{leaf} under {LIMIT} nots: {e}"));
        let deeper = format!("-{at}");
        too_deep(
            &asked(&corpus, json!({ "query": { "text": deeper } })).unwrap_err(),
            leaf,
        );
    }
}
