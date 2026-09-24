//! The refusal walk (`search/BUILD-PLAN.md`, "How a partial build stays
//! honest", rule 1) and invariant 3 of the surface: every construct of the
//! reference either evaluates or is refused by its name on the one list
//! `--describe` prints — never the unknown-name error, never undecided,
//! never an empty answer — and a query's validity never depends on the
//! corpus, so the whole language corpus is asked of a store holding
//! nothing.

mod common;

use acquisition_search::{ErrorKind, NOT_BUILT, SearchError, answer, describe};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
struct Language {
    case: Vec<Case>,
    value: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    construct: String,
    text: String,
    canonical: Option<String>,
}

fn language() -> Language {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/language.toml");
    toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The construct a refusal names, which must be one of the list's.
fn refused(e: &SearchError) -> Option<&'static str> {
    let SearchError::Language(e) = e else {
        return None;
    };
    if e.kind != ErrorKind::NotBuilt {
        return None;
    }
    NOT_BUILT
        .iter()
        .map(|n| n.construct)
        .find(|c| e.message.starts_with(&format!("not built: {c} (step ")))
}

#[test]
fn every_construct_of_the_reference_evaluates_or_is_refused_by_its_name_on_the_list() {
    let store = common::store();
    let corpus = common::load(&store, None);
    let language = language();
    let (mut evaluated, mut refusals) = (0, std::collections::BTreeSet::new());
    let mut stray: Vec<String> = Vec::new();
    let queries = language
        .case
        .iter()
        .map(|c| (c, json!({ "query": { "text": c.text } })));
    let sorts = language
        .value
        .iter()
        .map(|c| (c, json!({ "view": { "rows": { "sort": c.text } } })));
    for (case, request) in queries.chain(sorts) {
        if case.canonical.is_none() {
            continue; // an authoring error of the grammar: step 1's
        }
        match answer(&corpus, &serde_json::from_value(request).unwrap()) {
            // invariant 3: valid over a store holding nothing, and the
            // nothing is said — a total, never an error
            Ok(answer) => {
                assert_eq!(
                    answer.total.matched, 0,
                    "{}: `{}`",
                    case.construct, case.text
                );
                evaluated += 1;
            }
            Err(e) => match refused(&e) {
                Some(construct) => {
                    refusals.insert(construct);
                }
                None => stray.push(format!("{}: `{}`: {e}", case.construct, case.text)),
            },
        }
    }
    assert!(
        stray.is_empty(),
        "neither evaluated nor refused by a listed name:\n{}",
        stray.join("\n")
    );
    assert!(evaluated > 0 && !refusals.is_empty());
    // what the corpus can reach of the list is refused by that name; the
    // flags are the CLI's to refuse, by the same function
    for construct in ["pseudo.*", "sockets", "linked(…)", "has:priced", "price.*"] {
        assert!(
            refusals.contains(construct),
            "no case of the corpus met `{construct}`: {refusals:?}"
        );
    }
}

#[test]
fn the_list_the_walk_checks_is_the_list_describe_prints() {
    let printed = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let printed: Vec<&str> = printed["not_built"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["construct"].as_str().unwrap())
        .collect();
    let list: Vec<&str> = NOT_BUILT.iter().map(|n| n.construct).collect();
    assert_eq!(printed, list);
}

/// S53 (C102): an item's class is not a field of the body — it is a
/// derivation the search owns, from the class table (C106) — and the
/// limit stays stated in the register's words; `class:ring` is a valid
/// selector over a store holding nothing (invariant 3), and the table's
/// definition is what `--describe class` prints.
#[test]
fn c102_s53_a_class_is_a_derivation_the_search_owns_and_says_so() {
    let store = common::store();
    let corpus = common::load(&store, None);
    let a = common::ask(&corpus, "class:ring").unwrap();
    assert_eq!(a.total.matched, 0);
    let printed = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let limits: Vec<&str> = printed["limits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["said"].as_str().unwrap())
        .collect();
    assert!(limits.contains(
        &"an item's class is not a field; a class the search names is a derivation it owns, and an item it cannot class is shown unclassed"
    ));
    let class = serde_json::to_value(describe(&["class".to_string()]).unwrap()).unwrap();
    let what = class["fields"][0]["what"].as_str().unwrap();
    assert!(
        what.contains("classes v1") && what.contains("RePoE"),
        "{what}"
    );
    assert!(
        class["fields"][0]["values"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "Rings")
    );
}

#[test]
fn a_name_the_language_does_not_know_is_an_authoring_error_with_near_names() {
    let store = common::store();
    let corpus = common::load(&store, None);
    for (text, kind, reading) in [
        ("rarty=rare", "unknown_name", Some("rarity=rare")),
        ("rarity=rar", "unknown_value", Some("rarity=rare")),
        ("is:corupted", "unknown_name", Some("is:corrupted")),
        ("ilvl:84", "operator_mismatch", Some("ilvl=84")),
        ("name~\"(\"", "bad_pattern", None),
        ("line(is:corrupted)", "unknown_name", None),
        ("is:crafted", "unknown_name", Some("line(is:crafted)")),
        ("rare", "bare_word", Some("rarity=rare")),
        ("corrupted", "bare_word", Some("is:corrupted")),
        ("line(fractured)", "bare_word", Some("is:fractured")),
    ] {
        let e = common::ask(&corpus, text).unwrap_err();
        let json = e.to_json();
        assert_eq!(json["kind"], kind, "`{text}`: {e}");
        if let Some(reading) = reading {
            assert_eq!(json["readings"][0], reading, "`{text}`: {e}");
        }
    }
}
