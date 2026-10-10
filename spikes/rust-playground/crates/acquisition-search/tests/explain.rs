//! Why not (`search/DESIGN.md`, *Explain*, *When the total is zero*, *The
//! request at a terminal*; the build plan, step 10): `--explain`,
//! `--context corpus` and the zero block's offer of them, through the
//! crate's boundary — a request in, an answer out, as JSON. Every count
//! is worked by hand on the reference's seven items (`common::worked`)
//! before it was run: a forced node is the query with that node replaced
//! by `true()` or `false()`, so each cell is what the build answers the
//! transformed query, typed.

mod common;

use acquisition_search::{Corpus, Request, answer};
use common::*;
use serde_json::{Value, json};

const CANONICAL: &str = r##"league=Standard class=Rings rarity=rare line("# to maximum Life" arg1>=90) pseudo.total_res>=60"##;

fn explained(corpus: &Corpus, query: &str, path: &str) -> Result<Value, Value> {
    let request: Request = serde_json::from_value(json!({
        "query": { "text": query },
        "view": { "explain": { "path": path } },
    }))
    .unwrap();
    answer(corpus, &request)
        .map(|a| as_json(&a))
        .map_err(|e| e.to_json())
}

/// A forced answer as the table writes a cell: matched · undecided, and
/// the items that match — each count followed by its route, which is the
/// transformed query asked as a request of its own.
fn cell(corpus: &Corpus, forced: &Value) -> (u64, u64, Vec<String>) {
    let follow = |count: &Value| -> Vec<String> {
        let Some(request) = count.get("request") else {
            assert_eq!(count["count"], 0, "a count with no route: {count}");
            return Vec::new();
        };
        let routed =
            as_json(&answer(corpus, &serde_json::from_value(request.clone()).unwrap()).unwrap());
        assert_eq!(routed["total"]["matched"], count["count"], "{request}");
        ids(&routed)
    };
    let matched = follow(&forced["matched"]);
    follow(&forced["undecided"]);
    // the transformed query, typed: the same two counts
    let typed = as_json(&ask(corpus, forced["query"]["text"].as_str().unwrap()).unwrap());
    assert_eq!(typed["total"]["matched"], forced["matched"]["count"]);
    assert_eq!(
        typed["total"]["undecided"]["count"],
        forced["undecided"]["count"]
    );
    assert_eq!(typed["query"]["tree"], forced["query"]["tree"]);
    assert_eq!(ids(&typed), matched);
    (
        forced["matched"]["count"].as_u64().unwrap(),
        forced["undecided"]["count"].as_u64().unwrap(),
        matched,
    )
}

fn expect(
    corpus: &Corpus,
    query: &str,
    path: &str,
    node: &str,
    forced_true: (u64, u64, &[&str]),
    forced_false: (u64, u64, &[&str]),
) -> Value {
    let a = explained(corpus, query, path).unwrap_or_else(|e| panic!("{query} at {path}: {e}"));
    let explain = &a["view"]["explain"];
    assert_eq!(explain["path"], path, "{query}");
    assert_eq!(explain["node"], node, "{query} at {path}");
    for (which, (matched, undecided, who)) in
        [("forced_true", forced_true), ("forced_false", forced_false)]
    {
        let got = cell(corpus, &explain[which]);
        assert_eq!(
            (got.0, got.1, got.2.iter().map(String::as_str).collect()),
            (matched, undecided, who.to_vec()),
            "{query} at {path}, {which}: {}",
            explain[which]["query"]["text"]
        );
    }
    // explain shows counts and no rows
    assert_eq!(a["rows"], json!([]));
    a
}

/// The brief's first table, cell by cell: the worked example's query,
/// 1 match (r1) and 2 undecided (r5, r6) as given.
#[test]
fn explain_the_worked_example_cell_by_cell() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let none: (u64, u64, &[&str]) = (0, 0, &[]);
    for (path, node, forced_true) in [
        ("0", "league=Standard", (1, 2, &["r1"][..])),
        ("1", "class=Rings", (2, 1, &["r1", "r5"][..])),
        ("2", "rarity=rare", (2, 2, &["r1", "r4"][..])),
        (
            "3",
            r##"line("# to maximum Life" arg1>=90)"##,
            (3, 2, &["r1", "r2", "r7"][..]),
        ),
        ("3.0", r##""# to maximum Life""##, (1, 2, &["r1"][..])),
        ("3.1", "arg1>=90", (2, 2, &["r1", "r2"][..])),
        ("4", "pseudo.total_res>=60", (3, 1, &["r1", "r3", "r6"][..])),
    ] {
        let a = expect(&corpus, WORKED, path, node, forced_true, none);
        // as given, three times beside the two forced: the answer's own total
        assert_eq!(a["query"]["text"], CANONICAL);
        assert_eq!(
            a["view"]["explain"]["given"],
            json!({ "matched": 1, "undecided": 2 })
        );
        assert_eq!(a["total"]["matched"], 1);
        assert_eq!(a["total"]["undecided"]["count"], 2);
    }
}

/// The substitution is literal (invariant 1): the one node replaced,
/// nothing reordered, flattened, merged or simplified around it — the
/// transformed query's text and tree are the query's with that node a
/// `true()` or a `false()`.
#[test]
fn explain_replaces_one_node_and_changes_nothing_around_it() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let a = explained(&corpus, WORKED, "3.0").unwrap();
    let explain = &a["view"]["explain"];
    assert_eq!(
        explain["forced_true"]["query"]["text"],
        r##"league=Standard class=Rings rarity=rare line(true() arg1>=90) pseudo.total_res>=60"##
    );
    assert_eq!(
        explain["forced_false"]["query"]["text"],
        r##"league=Standard class=Rings rarity=rare line(false() arg1>=90) pseudo.total_res>=60"##
    );
    let mut tree = a["query"]["tree"].clone();
    tree["all"][3]["where"]["all"][0] = json!({ "const": true });
    assert_eq!(explain["forced_true"]["query"]["tree"], tree);

    // a group whole, and the parentheses around a node kept as written
    let a = explained(&corpus, WORKED, "3").unwrap();
    assert_eq!(
        a["view"]["explain"]["forced_true"]["query"]["text"],
        r##"league=Standard class=Rings rarity=rare true() pseudo.total_res>=60"##
    );
    let nested = "(rarity=rare (class=Rings league=Standard)) rarity=rare";
    let a = explained(&corpus, nested, "0.1.0").unwrap();
    assert_eq!(a["view"]["explain"]["node"], "class=Rings");
    assert_eq!(
        a["view"]["explain"]["forced_true"]["query"]["text"],
        "(rarity=rare (true() league=Standard)) rarity=rare"
    );
    assert_eq!(
        a["view"]["explain"]["forced_false"]["query"]["text"],
        "(rarity=rare (false() league=Standard)) rarity=rare"
    );
    // a group of terms is a node too, and forcing it leaves its siblings
    let a = explained(&corpus, nested, "0.1").unwrap();
    assert_eq!(a["view"]["explain"]["node"], "class=Rings league=Standard");
    assert_eq!(
        a["view"]["explain"]["forced_false"]["query"]["text"],
        "(rarity=rare false()) rarity=rare"
    );
}

/// A node under a not, worked by hand: `-rarity=magic` and a life of 90.
/// As given r1, r3, r5 and r6 (its explicit 95 is a witness) match; r2
/// holds 20 and 75, r4 is magic, r7 has no life line.
#[test]
fn explain_a_node_under_a_not() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let query = r##"-rarity=magic "+# to maximum Life">=90"##;
    let a = expect(
        &corpus,
        query,
        "0.0",
        "rarity=magic",
        // not true is false on every item
        (0, 0, &[]),
        // not false is true: the magic r4 comes in
        (5, 0, &["r1", "r3", "r4", "r5", "r6"]),
    );
    assert_eq!(
        a["view"]["explain"]["given"],
        json!({ "matched": 4, "undecided": 0 })
    );
    assert_eq!(
        a["view"]["explain"]["forced_true"]["query"]["text"],
        r##"-true() line("# to maximum Life" arg1>=90)"##
    );
    // the not itself, forced: the other way round
    expect(
        &corpus,
        query,
        "0",
        "-rarity=magic",
        (5, 0, &["r1", "r3", "r4", "r5", "r6"]),
        (0, 0, &[]),
    );
}

/// A node under an or, worked by hand: `rarity=magic or
/// pseudo.total_res<60`. As given r4 (magic) and r3 (55) match and r6,
/// whose total is an incomplete subtotal, is undecided. An or alone is
/// the query's one term, path 0, and its alternatives 0.0 and 0.1.
#[test]
fn explain_a_node_under_an_or() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let query = "rarity=magic or pseudo.total_res<60";
    let all: &[&str] = &["r1", "r2", "r3", "r4", "r5", "r6", "r7"];
    let a = expect(
        &corpus,
        query,
        "0.0",
        "rarity=magic",
        (7, 0, all),
        // r4 totals 70 and leaves; r6 stays open
        (1, 1, &["r3"]),
    );
    assert_eq!(
        a["view"]["explain"]["given"],
        json!({ "matched": 2, "undecided": 1 })
    );
    expect(
        &corpus,
        query,
        "0.1",
        "pseudo.total_res<60",
        (7, 0, all),
        (1, 0, &["r4"]),
    );
    let a = expect(
        &corpus,
        query,
        "0",
        "rarity=magic or pseudo.total_res<60",
        (7, 0, all),
        (0, 0, &[]),
    );
    assert_eq!(
        a["view"]["explain"]["forced_true"]["query"]["text"],
        "true()"
    );
}

/// A node under `holds`, worked by hand: two at least of magic, a total
/// of 65 and a life of 100. As given r4 holds two; r6 holds none for
/// sure and two may hold — its total is incomplete, its life of 95 sits
/// beside an unread array — so it is undecided; every other item holds
/// one or none.
#[test]
fn explain_a_node_under_holds_and_a_condition_inside_its_group() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let query = r##"holds(rarity=magic, pseudo.total_res>=65, "+# to maximum Life">=100)>=2"##;
    let a = expect(
        &corpus,
        query,
        "0.0",
        "rarity=magic",
        // one for sure on every item: r1, r5 and r7 total 65, r3 has 100
        // life, r4 its total; r2 has neither; r6 holds one, may hold three
        (5, 1, &["r1", "r3", "r4", "r5", "r7"]),
        // r4 holds its total alone; r6 may hold two
        (0, 1, &[]),
    );
    assert_eq!(
        a["view"]["explain"]["given"],
        json!({ "matched": 1, "undecided": 1 })
    );
    // the bound inside the line's group, under the holds: forced true the
    // group asks the life line alone, which r7 lacks — it never creates a
    // member — and r6's explicit line is a witness, its total still open
    let a = expect(
        &corpus,
        query,
        "0.2.1",
        "arg1>=100",
        (3, 1, &["r1", "r4", "r5"]),
        // no occurrence holds false(): r6's unread array leaves nothing open
        (1, 0, &["r4"]),
    );
    assert_eq!(
        a["view"]["explain"]["forced_true"]["query"]["text"],
        r##"holds(rarity=magic, pseudo.total_res>=65, line("# to maximum Life" true()))>=2"##
    );
}

/// Explain shows counts and no rows: beside a count, a crossed table or
/// a sort it is an authoring error, said where the view's other
/// conflicts are said.
#[test]
fn explain_beside_a_count_a_cross_or_a_sort_is_an_authoring_error() {
    use acquisition_search::answer::{Rows, View};
    let keys = |text: &str| Some(text.split(',').map(str::to_string).collect::<Vec<_>>());
    let path = || Some("0".to_string());
    for (count, cross, sort) in [
        (keys("rarity"), None, None),
        (None, keys("rarity,league"), None),
        (None, None, Some("ilvl".to_string())),
    ] {
        let e = View::of(
            count.clone(),
            cross.clone(),
            None,
            path(),
            Rows {
                limit: Some(20),
                sort: sort.clone(),
                desc: false,
                after: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            serde_json::to_value(e.kind).unwrap(),
            "view",
            "{count:?} {cross:?} {sort:?}: {e}"
        );
        assert!(e.message.contains("--explain"), "{e}");
    }
    // alone it is the view, whatever limit an adapter always sends
    let view = View::of(None, None, None, path(), Rows::default()).unwrap();
    assert_eq!(
        serde_json::to_value(&view).unwrap(),
        json!({ "explain": { "path": "0" } })
    );
    // and in a request it is one view of the four: a sort has nowhere to stand
    assert!(
        serde_json::from_value::<Request>(
            json!({ "view": { "explain": { "path": "0", "sort": "ilvl" } } })
        )
        .is_err()
    );
}

/// A path the query lacks is an authoring error whose readings list the
/// query's nodes with their paths — a term, a group of terms, a
/// condition inside `line( … )` — and the conditions inside a `sum( … )`
/// take none.
#[test]
fn explain_a_path_the_query_lacks_lists_the_nodes_with_their_paths() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    for path in ["5", "3.2", "", "x", "0.0", "3.1.0", "03", "3."] {
        let e = explained(&corpus, WORKED, path).unwrap_err();
        assert_eq!(e["kind"], "no_such_path", "`{path}`: {e}");
        assert_eq!(
            e["readings"],
            json!([
                "0  league=Standard",
                "1  class=Rings",
                "2  rarity=rare",
                "3  line(\"# to maximum Life\" arg1>=90)",
                "3.0  \"# to maximum Life\"",
                "3.1  arg1>=90",
                "4  pseudo.total_res>=60",
            ]),
            "`{path}`"
        );
    }
    // a not, an or inside a group, a probe and a link group are numbered
    // as the tree's children are; a sum's conditions take no path
    let e = explained(
        &corpus,
        r##"-(rarity=rare or is:corrupted) line(-("# to maximum Life" or source=implicit)) undecided(ilvl>=80) linked(red>=1) sum("+#% to Fire Resistance")>=60 undecided(ilvl)"##,
        "9",
    )
    .unwrap_err();
    assert_eq!(
        e["readings"],
        json!([
            "0  -(rarity=rare or is:corrupted)",
            "0.0  rarity=rare or is:corrupted",
            "0.0.0  rarity=rare",
            "0.0.1  is:corrupted",
            "1  line(-(\"# to maximum Life\" or source=implicit))",
            "1.0  -(\"# to maximum Life\" or source=implicit)",
            "1.0.0  \"# to maximum Life\" or source=implicit",
            "1.0.0.0  \"# to maximum Life\"",
            "1.0.0.1  source=implicit",
            "2  undecided(ilvl>=80)",
            "2.0  ilvl>=80",
            "3  linked(red>=1)",
            "3.0  red>=1",
            "4  sum(line(\"#% to Fire Resistance\").arg1)>=60",
            "5  undecided(ilvl)",
        ])
    );
    // every path listed is one explain takes, and every term's path is listed
    let query = r##"-(rarity=rare or is:corrupted) line(-("# to maximum Life" or source=implicit)) undecided(ilvl>=80) linked(red>=1) sum("+#% to Fire Resistance")>=60 undecided(ilvl)"##;
    let listed: Vec<String> = e["readings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap().split("  ").next().unwrap().to_string())
        .collect();
    for path in &listed {
        explained(&corpus, query, path).unwrap_or_else(|e| panic!("{path}: {e}"));
    }
    let asked = as_json(&ask(&corpus, query).unwrap());
    for term in asked["terms"].as_array().unwrap() {
        assert!(listed.contains(&term["path"].as_str().unwrap().to_string()));
    }
    // the empty query has no node
    let e = explained(&corpus, "", "0").unwrap_err();
    assert_eq!(e["kind"], "no_such_path");
    assert!(e.get("readings").is_none(), "{e}");
}

/// A substitution the binder refuses is refused with the binder's own
/// error; on the table's query none is, and a slot whose template was
/// the forced node is the evaluator's, as under an or.
#[test]
fn explain_a_group_whose_template_is_forced_still_binds() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    assert!(ask(&corpus, r##"line(true() arg1>=90)"##).is_ok());
    // the slot check held `low` against the quoted template; forced, the
    // template states no numbers and the slot is read of each occurrence
    let a = explained(
        &corpus,
        r##"line("Adds # to # Cold Damage" low>=1)"##,
        "0.0",
    )
    .unwrap();
    assert_eq!(
        a["view"]["explain"]["forced_true"]["query"]["text"],
        "line(true() low>=1)"
    );
}

fn with_context(corpus: &Corpus, query: &str, view: Value) -> Value {
    let request: Request = serde_json::from_value(json!({
        "query": { "text": query },
        "context": "corpus",
        "view": view,
    }))
    .unwrap();
    as_json(&answer(corpus, &request).unwrap())
}

fn buckets(table: &Value) -> Vec<(String, u64)> {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                match b["bucket"].as_str().unwrap() {
                    "value" => b["value"].as_str().unwrap().to_string(),
                    other => format!("({other})"),
                },
                b["count"].as_u64().unwrap(),
            )
        })
        .collect()
}

/// `--context corpus` takes any view: the answer's query is the empty
/// one, and the query typed beside it is carried as set aside and
/// counted nowhere. Beside the worked example's query: 7 matches, 0
/// undecided; by rarity, rare 6 and magic 1; by class, Rings 6 and
/// undecided 1 (r5's base is in no class table).
#[test]
fn context_corpus_answers_the_empty_query_and_sets_the_typed_one_aside() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let rows = with_context(&corpus, WORKED, json!({ "rows": {} }));
    assert_eq!(rows["query"]["text"], "");
    assert_eq!(rows["query"]["tree"], json!({ "all": [] }));
    assert_eq!(rows["set_aside"]["text"], CANONICAL);
    assert!(rows["set_aside"]["tree"]["all"].is_array());
    assert_eq!(rows["terms"], json!([]));
    assert_eq!(rows["total"]["matched"], 7);
    assert_eq!(rows["total"]["undecided"]["count"], 0);
    assert_eq!(ids(&rows), ["r1", "r2", "r3", "r4", "r5", "r6", "r7"]);
    // the same answer as the empty query's but for what was set aside
    let mut empty = as_json(&ask(&corpus, "").unwrap());
    empty["set_aside"] = rows["set_aside"].clone();
    assert_eq!(rows, empty);

    let counted = with_context(
        &corpus,
        WORKED,
        json!({ "counts": { "keys": ["rarity", "class"] } }),
    );
    assert_eq!(counted["query"]["text"], "");
    assert_eq!(counted["set_aside"]["text"], CANONICAL);
    assert_eq!(counted["total"]["matched"], 7);
    let tables = counted["view"]["counts"]["tables"].as_array().unwrap();
    assert_eq!(
        buckets(&tables[0]),
        [("rare".to_string(), 6), ("magic".to_string(), 1)]
    );
    assert_eq!(
        buckets(&tables[1]),
        [("Rings".to_string(), 6), ("(undecided)".to_string(), 1)]
    );
    // a bucket routes under the answer's query, the empty one: the query
    // set aside is in no route
    let magic: Request =
        serde_json::from_value(tables[0]["buckets"][1]["request"].clone()).unwrap();
    assert!(!magic.query.text.as_deref().unwrap().contains("Standard"));
    assert_eq!(ids(&as_json(&answer(&corpus, &magic).unwrap())), ["r4"]);

    let crossed = with_context(
        &corpus,
        WORKED,
        json!({ "cross": { "keys": ["rarity", "class"] } }),
    );
    assert_eq!(crossed["set_aside"]["text"], CANONICAL);
    assert_eq!(crossed["total"]["matched"], 7);
    assert_eq!(crossed["view"]["cross"]["keys"], json!(["rarity", "class"]));

    // `matches` is what every answer is, and changes nothing
    let matches: Request =
        serde_json::from_value(json!({ "query": { "text": WORKED }, "context": "matches" }))
            .unwrap();
    assert_eq!(
        as_json(&answer(&corpus, &matches).unwrap()),
        as_json(&ask(&corpus, WORKED).unwrap())
    );
    // nothing typed, nothing set aside
    let nothing = with_context(&corpus, "", json!({ "rows": {} }));
    assert!(nothing.get("set_aside").is_none());
    // a query set aside is still read: one that does not bind is the
    // error it always was
    let bad: Request =
        serde_json::from_value(json!({ "query": { "text": "rare" }, "context": "corpus" }))
            .unwrap();
    assert_eq!(
        answer(&corpus, &bad).unwrap_err().to_json()["kind"],
        "bare_word"
    );
    assert!(bad.check().is_err());
}

fn offered(answer: &Value) -> Vec<String> {
    answer["zero"]["offers"]["explain"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["path"].as_str().unwrap().to_string())
        .collect()
}

/// The brief's zero answer, `rarity=magic pseudo.total_res<60`: nothing
/// matches, each term matches one. It offers explain of path 0, then of
/// path 1, then the corpus context — each a request, never its output —
/// and followed, each answers what the second table says.
#[test]
fn a_zero_answer_offers_explain_of_each_part_and_then_the_corpus() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let query = "rarity=magic pseudo.total_res<60";
    let a = as_json(&ask(&corpus, query).unwrap());
    assert_eq!(a["total"]["matched"], 0);
    assert_eq!(a["total"]["undecided"]["count"], 0);
    assert_eq!(a["terms"][0]["matched"]["count"], 1);
    assert_eq!(a["terms"][1]["matched"]["count"], 1);
    let zero = &a["zero"];
    assert!(zero.get("not_built").is_none(), "{zero}");
    assert_eq!(offered(&a), ["0", "1"]);
    let offers = &zero["offers"];
    assert!(offers.get("explain_left_out").is_none());
    assert_eq!(offers["explain"][0]["node"], "rarity=magic");
    assert_eq!(offers["explain"][1]["node"], "pseudo.total_res<60");
    // an offer computes no forced count
    assert!(!offers.to_string().contains("forced"), "{offers}");

    let follow = |request: &Value| -> Value {
        as_json(&answer(&corpus, &serde_json::from_value(request.clone()).unwrap()).unwrap())
    };
    let none: (u64, u64, Vec<String>) = (0, 0, Vec::new());
    for (at, forced_true) in [(0, (1, 1, vec!["r3"])), (1, (1, 0, vec!["r4"]))] {
        let request = &offers["explain"][at]["request"];
        assert_eq!(request["query"]["text"], query);
        assert_eq!(request["scope"]["realm"], "pc");
        let followed = follow(request);
        let explain = &followed["view"]["explain"];
        assert_eq!(explain["path"], offers["explain"][at]["path"]);
        assert_eq!(explain["given"], json!({ "matched": 0, "undecided": 0 }));
        let (matched, undecided, who) = cell(&corpus, &explain["forced_true"]);
        assert_eq!(
            (matched, undecided, who),
            (
                forced_true.0,
                forced_true.1,
                forced_true.2.iter().map(|s| s.to_string()).collect()
            )
        );
        assert_eq!(cell(&corpus, &explain["forced_false"]), none);
    }
    let request = &offers["context"]["request"];
    assert_eq!(request["context"], "corpus");
    assert_eq!(request["query"]["text"], query);
    let followed = follow(request);
    assert_eq!(followed["total"]["matched"], 7);
    assert_eq!(followed["query"]["text"], "");
    assert_eq!(followed["set_aside"]["text"], query);
}

/// The offers follow the counts, never the query's order (rule 9): the
/// part fewest items match first, and by path where two are equal — a
/// part being a child of the root, a group of terms among them.
#[test]
fn the_zero_blocks_offers_are_ordered_by_what_each_part_matches() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    // rare 6, magic 1
    let a = as_json(&ask(&corpus, "rarity=rare rarity=magic").unwrap());
    assert_eq!(offered(&a), ["1", "0"]);
    // the or matches all seven, the league none, the unique none: the two
    // equal ones by path
    let a = as_json(
        &ask(
            &corpus,
            "(rarity=rare or rarity=magic) league=Hardcore rarity=unique",
        )
        .unwrap(),
    );
    assert_eq!(offered(&a), ["1", "2", "0"]);
    assert_eq!(
        a["zero"]["offers"]["explain"][2]["node"],
        "rarity=rare or rarity=magic"
    );
    // one term alone is the query's one part
    let a = as_json(&ask(&corpus, "rarity=unique").unwrap());
    assert_eq!(offered(&a), ["0"]);
    // an or alone too: its alternatives are not the root's children
    let a = as_json(&ask(&corpus, "rarity=unique or league=Hardcore").unwrap());
    assert_eq!(offered(&a), ["0"]);
    // the empty query over a scope of nothing has no part to explain, and
    // nothing to set aside
    let empty = store();
    let a = as_json(&ask(&load(&empty, None), "").unwrap());
    assert_eq!(a["total"]["matched"], 0);
    assert_eq!(a["zero"]["offers"], json!({ "explain": [] }));
    // an answer that is not zero offers nothing
    assert!(
        as_json(&ask(&corpus, WORKED).unwrap())
            .get("zero")
            .is_none()
    );
}

/// The block is bounded (invariant 5): ten offers and the rest counted,
/// the cut made after the order.
#[test]
fn the_zero_blocks_offers_are_cut_at_ten_and_the_rest_counted() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    // rare first, matching six, then eleven magics matching one each
    let query = format!("rarity=rare {}", ["rarity=magic"; 11].join(" "));
    let a = as_json(&ask(&corpus, &query).unwrap());
    assert_eq!(a["total"]["matched"], 0);
    let expected: Vec<String> = (1..=10).map(|n| n.to_string()).collect();
    assert_eq!(offered(&a), expected);
    assert_eq!(a["zero"]["offers"]["explain_left_out"], 2);
    assert!(a["zero"]["offers"]["context"].is_object());
}

/// A view kept: the corpus context is offered beside the view the
/// request had, and explain in place of it.
#[test]
fn a_zero_answer_under_a_count_offers_the_corpus_under_the_same_count() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let request: Request = serde_json::from_value(json!({
        "query": { "text": "rarity=unique" },
        "view": { "counts": { "keys": ["tab"] } },
    }))
    .unwrap();
    let a = as_json(&answer(&corpus, &request).unwrap());
    let offers = &a["zero"]["offers"];
    assert_eq!(
        offers["context"]["request"]["view"],
        json!({ "counts": { "keys": ["tab"] } })
    );
    assert_eq!(
        offers["explain"][0]["request"]["view"],
        json!({ "explain": { "path": "0" } })
    );
    let followed: Request = serde_json::from_value(offers["context"]["request"].clone()).unwrap();
    let followed = as_json(&answer(&corpus, &followed).unwrap());
    assert_eq!(followed["total"]["matched"], 7);
    assert_eq!(followed["view"]["counts"]["tables"][0]["key"], "tab");
}

/// `--describe` says what each construct does, each can be asked for by
/// its word, and none of the three is on the list of what is not built.
#[test]
fn describe_says_what_explain_context_and_against_do() {
    use acquisition_search::{NOT_BUILT, describe};
    let whole = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let names: Vec<&str> = whole["explain"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["--explain", "--context", "show --against"]);
    assert_eq!(whole["explain"][1]["values"], json!(["matches", "corpus"]));
    for (word, name) in [
        ("explain", "--explain"),
        ("--explain", "--explain"),
        ("context", "--context"),
        ("against", "show --against"),
    ] {
        let one = serde_json::to_value(describe(&[word.to_string()]).unwrap()).unwrap();
        assert!(
            one["explain"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["name"] == name),
            "`{word}`: {one}"
        );
        assert!(one["explain"][0]["what"].as_str().unwrap().len() > 40);
    }
    let one = serde_json::to_value(describe(&["context".to_string()]).unwrap()).unwrap();
    assert_eq!(one["explain"].as_array().unwrap().len(), 1);
    for construct in ["--explain", "--context", "show --against"] {
        assert!(
            NOT_BUILT.iter().all(|n| n.construct != construct),
            "{construct}"
        );
    }
    // the four that wait keep their names and their step
    let waiting: Vec<&str> = NOT_BUILT.iter().map(|n| n.construct).collect();
    assert_eq!(
        waiting,
        ["--fields", "--view locations", "--rebind", "show --basis"]
    );
    assert!(NOT_BUILT.iter().all(|n| n.step == Some(10)));
}

/// `--sum` stands beside a count, and explain shows none: the pair is the
/// view's error as a count, a cross and a sort beside explain are.
#[test]
fn explain_beside_a_sum_is_an_authoring_error() {
    use acquisition_search::answer::{Rows, View};
    let e = View::of(
        None,
        None,
        Some("ilvl".to_string()),
        Some("0".to_string()),
        Rows::default(),
    )
    .unwrap_err();
    assert_eq!(serde_json::to_value(e.kind).unwrap(), "view");
    assert!(e.message.contains("--explain"), "{e}");
}

/// An explained answer has no zero block: explain is asked of a zero
/// answer more than of any other, and what it answers is the three
/// counts, never an offer of itself.
#[test]
fn an_explained_zero_answer_has_no_zero_block() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let a = explained(&corpus, "rarity=magic pseudo.total_res<60", "0").unwrap();
    assert_eq!(a["total"]["matched"], 0);
    assert!(a.get("zero").is_none(), "{}", a["zero"]);
}

/// A part's count is of the items it holds on, never of those it leaves
/// open: the total under 60 holds on r3 and leaves r6 open, magic holds
/// on r4, so the two are equal and the offers are by path — where
/// counting the open one would put magic first.
#[test]
fn the_zero_blocks_offers_count_what_a_part_holds_on_and_not_what_it_leaves_open() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let a = as_json(&ask(&corpus, "pseudo.total_res<60 rarity=magic").unwrap());
    assert_eq!(a["total"]["matched"], 0);
    assert_eq!(a["terms"][0]["undecided"]["count"], 1);
    assert_eq!(offered(&a), ["0", "1"]);
}

/// Explain is of the query typed and the corpus context sets it aside:
/// the pair is an authoring error that names both, never "the empty
/// query has no node".
#[test]
fn explain_beside_the_corpus_context_is_an_authoring_error_naming_both() {
    let s = worked();
    let corpus = load(&s, Some("pc"));
    let request: Request = serde_json::from_value(json!({
        "query": { "text": "rarity=rare" },
        "context": "corpus",
        "view": { "explain": { "path": "0" } },
    }))
    .unwrap();
    let e = answer(&corpus, &request).unwrap_err().to_json();
    assert_eq!(e["kind"], "view", "{e}");
    let message = e["error"].as_str().unwrap();
    assert!(
        message.contains("--explain") && message.contains("--context corpus"),
        "{e}"
    );
    assert!(request.check().is_err());
}
