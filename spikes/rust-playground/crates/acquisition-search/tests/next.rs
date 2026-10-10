//! Continuing and exchanging (`search/DESIGN.md`, *Two values*, *The
//! request at a terminal*, *Outside the first surface*; C98, C100; the
//! build plan, step 10): an answer cut at its limit says how to go on, a
//! continuation names its basis and is refused across a change, and a
//! route resubmitted on a later basis reports both counts — through the
//! crate's boundary, a request in, an answer out, as JSON.

mod common;

use acquisition_search::answer::{Asked, Request};
use acquisition_search::{Corpus, answer};
use common::generated::*;
use common::*;
use proptest::prelude::*;
use serde_json::{Value, json};

fn rows_of(answer: &Value) -> Vec<Value> {
    answer["rows"].as_array().unwrap().clone()
}

fn in_order(answer: &Value) -> Vec<String> {
    rows_of(answer)
        .iter()
        .map(|r| r["id"].as_str().unwrap().to_string())
        .collect()
}

fn asked(corpus: &Corpus, asked: &Value) -> Result<Value, Value> {
    let asked: Request = serde_json::from_value(asked.clone()).unwrap();
    answer(corpus, &asked)
        .map(|a| as_json(&a))
        .map_err(|e| e.to_json())
}

/// Every page of a request, each the continuation the one before named,
/// and the rows of all of them in order.
fn pages(corpus: &Corpus, first: &Value) -> (Vec<Value>, usize) {
    let mut rows = Vec::new();
    let mut request = first.clone();
    let mut asks = 0;
    loop {
        let a = asked(corpus, &request).unwrap_or_else(|e| panic!("{request}: {e}"));
        asks += 1;
        rows.extend(rows_of(&a));
        let view = &a["view"]["rows"];
        match view.get("next") {
            Some(next) => {
                assert!(view["left_out"].as_u64().unwrap() > 0, "{view}");
                // the token and the request are one value
                let token = next["token"].as_str().unwrap();
                assert_eq!(
                    serde_json::to_value(Request::from_token(token).unwrap()).unwrap(),
                    next["request"]
                );
                request = next["request"].clone();
            }
            None => {
                assert_eq!(view["left_out"], 0, "{view}");
                return (rows, asks);
            }
        }
        assert!(asks < 1000, "a continuation that never ends");
    }
}

/// The seven of the worked example three at a time: three asks, and the
/// rows of the three in order are the rows of one ask at a limit past the
/// total — unsorted, in the store's order, and sorted either way.
#[test]
fn the_pages_of_an_answer_are_the_answer_at_a_limit_past_the_total() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    for sort in [
        json!({}),
        json!({ "sort": "pseudo.total_res" }),
        json!({ "sort": "pseudo.total_res", "desc": true }),
    ] {
        let view = |limit: u64| {
            let mut rows = sort.clone();
            rows["limit"] = json!(limit);
            json!({ "query": { "text": "" }, "view": { "rows": rows } })
        };
        let whole = asked(&corpus, &view(100)).unwrap();
        assert_eq!(whole["total"]["matched"], 7);
        assert!(whole["view"]["rows"].get("next").is_none());
        let (rows, asks) = pages(&corpus, &view(3));
        assert_eq!(asks, 3, "{sort}");
        assert_eq!(rows, rows_of(&whole), "{sort}");
    }
    // the first page, as it says how to go on
    let first = asked(
        &corpus,
        &json!({ "query": { "text": "rarity=rare" }, "view": { "rows": { "limit": 2 } } }),
    )
    .unwrap();
    assert_eq!(in_order(&first), ["r1", "r2"]);
    let view = &first["view"]["rows"];
    assert_eq!(
        (&view["returned"], &view["left_out"]),
        (&json!(2), &json!(4))
    );
    let next = &view["next"]["request"];
    // a continuation names its basis, and the query, view and order it
    // continues (C100), its scope resolved
    assert_eq!(next["basis"], first["basis"]);
    assert_eq!(next["query"]["text"], "rarity=rare");
    assert_eq!(next["view"], json!({ "rows": { "limit": 2, "after": 2 } }));
    assert_eq!(next["scope"]["realm"], "pc");
    assert_eq!(next["scope"]["account"], "A");
    let second = asked(&corpus, next).unwrap();
    assert_eq!(in_order(&second), ["r3", "r5"]);
    assert_eq!(second["view"]["rows"]["after"], 2);
    // the counts are the answer's, on every page
    assert_eq!(second["total"], first["total"]);
    assert_eq!(second["terms"], first["terms"]);
}

/// A continuation is refused across a changed basis, and the refusal
/// names both (C98, C100): a fetch between two pages.
#[test]
fn a_continuation_is_refused_across_a_changed_basis_and_names_both() {
    let mut s = worked();
    let corpus = load(&s, Some("pc"));
    let first = asked(
        &corpus,
        &json!({ "query": { "text": "" }, "view": { "rows": { "limit": 3 } } }),
    )
    .unwrap();
    let next = first["view"]["rows"]["next"]["request"].clone();
    let before = first["basis"]["snapshot"]["response"].as_i64().unwrap();
    // held, the corpus still answers at its basis
    assert!(asked(&corpus, &next).is_ok());

    fetch_tab(&mut s, "pc", "Standard", "t3", "Never", vec![], 40);
    let later = load(&s, Some("pc"));
    let now = later.basis.snapshot.response;
    assert!(now > before);
    let e = asked(&later, &next).unwrap_err();
    assert_eq!(e["kind"], "basis_changed", "{e}");
    let message = e["error"].as_str().unwrap();
    assert!(
        message.contains(&before.to_string()) && message.contains(&now.to_string()),
        "{e}"
    );
    // the way on is the request from its start, a command that runs
    assert_eq!(
        e["readings"],
        json!(["acq search --account A --realm pc '' --limit 3"])
    );
    // the same request without its place is no continuation: it answers
    let mut again = next.clone();
    again["view"]["rows"]
        .as_object_mut()
        .unwrap()
        .remove("after");
    assert_eq!(asked(&later, &again).unwrap()["total"]["matched"], 7);
}

/// What makes a request a continuation is its place, and a place needs
/// the basis it was counted at.
#[test]
fn a_place_without_a_basis_is_an_authoring_error() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let e = asked(
        &corpus,
        &json!({ "query": { "text": "" }, "view": { "rows": { "after": 3 } } }),
    )
    .unwrap_err();
    assert_eq!(e["kind"], "view", "{e}");
    let placed: Request =
        serde_json::from_value(json!({ "view": { "rows": { "after": 3 } } })).unwrap();
    assert!(placed.check().is_err());
    // a place past the total is an empty page, never an error
    let first = asked(
        &corpus,
        &json!({ "query": { "text": "" }, "view": { "rows": { "limit": 3 } } }),
    )
    .unwrap();
    let mut far = first["view"]["rows"]["next"]["request"].clone();
    far["view"]["rows"]["after"] = json!(70);
    let a = asked(&corpus, &far).unwrap();
    assert_eq!(rows_of(&a), Vec::<Value>::new());
    assert_eq!(a["view"]["rows"]["left_out"], 0);
    assert!(a["view"]["rows"].get("next").is_none());
}

/// A limit of nothing returns no row, and a continuation from it would
/// return none again for ever: it says what it left out and names no way
/// on but a larger limit.
#[test]
fn a_limit_of_nothing_has_no_continuation() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let a = asked(
        &corpus,
        &json!({ "query": { "text": "" }, "view": { "rows": { "limit": 0 } } }),
    )
    .unwrap();
    let view = &a["view"]["rows"];
    assert_eq!(
        (&view["returned"], &view["left_out"]),
        (&json!(0), &json!(7))
    );
    assert!(view.get("next").is_none(), "{view}");
    // and from a place: a token's limit set to nothing
    let first = asked(
        &corpus,
        &json!({ "query": { "text": "" }, "view": { "rows": { "limit": 3 } } }),
    )
    .unwrap();
    let mut none = first["view"]["rows"]["next"]["request"].clone();
    none["view"]["rows"]["limit"] = json!(0);
    let a = asked(&corpus, &none).unwrap();
    assert_eq!(a["view"]["rows"]["left_out"], 4);
    assert!(a["view"]["rows"].get("next").is_none());
}

/// What an answer holds beside a request is read as the answer holds it:
/// a `next` with its token, an offer with its path, a route to the rest
/// with no count — each is the request inside, and none says what a
/// count said before.
#[test]
fn a_next_and_an_offer_are_read_whole_as_an_answer_holds_them() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let first = asked(
        &corpus,
        &json!({ "query": { "text": "" }, "view": { "rows": { "limit": 3 } } }),
    )
    .unwrap();
    let next = &first["view"]["rows"]["next"];
    let read = Asked::read(next).unwrap();
    assert!(read.before.is_none());
    assert_eq!(
        in_order(&as_json(&read.answer(&corpus).unwrap())),
        ["r4", "r5", "r6"]
    );
    let zero = as_json(&ask(&corpus, "rarity=magic pseudo.total_res<60").unwrap());
    let offers = &zero["zero"]["offers"];
    for offer in [&offers["explain"][0], &offers["context"]] {
        let read = Asked::read(offer).unwrap_or_else(|e| panic!("{offer}: {e}"));
        assert!(read.before.is_none());
        read.answer(&corpus).unwrap();
    }
    // a route with a basis and no count: the request, nothing before
    let mut rest = zero["terms"][0]["matched"].clone();
    rest.as_object_mut().unwrap().remove("count");
    assert!(Asked::read(&rest).unwrap().before.is_none());
}

/// A token is the continuation's request and nothing else; what is no
/// token is an error of its own kind.
#[test]
fn a_token_that_is_none_is_refused() {
    for token in ["", "abc", "acq1.", "acq1.!!!", "acq1.e30", "acq2.e30"] {
        let e = Request::from_token(token).unwrap_err();
        assert_eq!(
            serde_json::to_value(e.kind).unwrap(),
            "token",
            "`{token}`: {e}"
        );
    }
}

/// A route is a request that runs on any basis: resubmitted where it was
/// counted it says nothing more, and on a later basis it reports both
/// counts (the reference, *One worked example*).
#[test]
fn a_route_resubmitted_on_a_later_basis_reports_both_counts() {
    let mut s = worked();
    let corpus = load(&s, Some("pc"));
    let a = as_json(&ask(&corpus, "rarity=rare").unwrap());
    let route = a["terms"][0]["matched"].clone();
    assert_eq!(route["count"], 6);

    let read = |corpus: &Corpus, value: &Value| -> Value {
        let asked = Asked::read(value).unwrap();
        as_json(&asked.answer(corpus).unwrap())
    };
    let same = read(&corpus, &route);
    assert_eq!(same["total"]["matched"], 6);
    assert!(same.get("before").is_none(), "{}", same["before"]);

    // a rare ring more, in the tab never fetched
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t3",
        "Never",
        vec![item("r8", "Ring r8", "Two-Stone Ring", "Rare", json!({}))],
        40,
    );
    let later = load(&s, Some("pc"));
    let moved = read(&later, &route);
    assert_eq!(moved["total"]["matched"], 7);
    assert_eq!(
        moved["before"],
        json!({ "count": 6, "counted_at": route["counted_at"] })
    );
    assert_ne!(moved["basis"], route["counted_at"]);

    // a bare request is read as one, and what is neither is refused
    let bare = read(&later, &route["request"]);
    assert_eq!(bare["total"]["matched"], 7);
    assert!(bare.get("before").is_none());
    for bad in [
        json!([]),
        json!({ "count": 6 }),
        json!({ "query": 3 }),
        json!({ "count": 6, "request": {} }),
    ] {
        assert!(Asked::read(&bad).is_err(), "{bad}");
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: cases(48), failure_persistence: None, ..ProptestConfig::default() })]
    /// For any query, any sort and any limit, the pages in order are the
    /// answer at a limit past the total.
    #[test]
    fn the_pages_are_the_whole_for_any_query_sort_and_limit(
        q in query(true), sort in proptest::option::of(sort()), desc in any::<bool>(),
        limit in 1usize..4,
        bodies in proptest::collection::vec(body(true), 0..7),
    ) {
        let mut all = anchors();
        all.extend(bodies.iter().map(Body::json));
        let (corpus, _) = fixture(all);
        let text = q_text(&q, Spelling::Authored);
        let sort = sort.and_then(|s| sort_text(&s, Spelling::Authored));
        let view = |limit: usize| {
            let mut rows = json!({ "limit": limit });
            if let Some(sort) = &sort {
                rows["sort"] = json!(sort);
                rows["desc"] = json!(desc);
            }
            json!({ "query": { "text": text }, "view": { "rows": rows } })
        };
        let whole = match asked(&corpus, &view(10_000)) {
            Ok(whole) => whole,
            // a query or a sort the binder refuses pages nowhere
            Err(_) => return Ok(()),
        };
        let (rows, asks) = pages(&corpus, &view(limit));
        prop_assert_eq!(&rows, &rows_of(&whole), "`{}` sorted by {:?}", text, sort);
        let total = whole["total"]["matched"].as_u64().unwrap() as usize;
        prop_assert_eq!(asks, total.div_ceil(limit).max(1));
    }
}
