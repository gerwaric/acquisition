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
//!   number — and so do alternatives inside one group, each naming its
//!   own row, a sum and a largest over them among these: a mod named by
//!   two alternatives has the two numbers its rows apart have. What is
//!   asked names one template in each alternative, and never under a not
//!   alone: two conjoined hold on the mod and on no row apart, a not of
//!   one holds on a row apart of the mod it names, and `template:` tests
//!   the whole text, which the rows apart have none of — each by the
//!   rule that a mod is one occurrence.
//! - **A vocabulary row counts what its term returns** (invariant 4):
//!   every bucket of `--count line`, narrowed or not, and every source and
//!   flag beneath it, routes to as many items as it counted — a template
//!   that names a mod by a row, and one GGG spelled two ways, among them.
//!
//! - **On one occurrence a group or its not holds, never both and never
//!   neither**, and the two are open together (C93): any generated group,
//!   `template:` and patterns among its tests, over items of one line —
//!   a mod of several rows or not, its flags and its number read or not.
//!   A not that parted a comparison from the template saying what it
//!   reads held beside the group it was the not of (the outside review
//!   of step 9c4, its second look).
//! - **A group's or is the item's or of the same alternatives**: `line(A
//!   or B)` and `line(A) or line(B)` return the same items of one line,
//!   and leave the same open.
//!
//! What this cannot see: a fault the mod and its rows apart share, or a
//! group and its not. That is the hand-counted tests' (`tests/rows.rs`,
//! `tests/pseudo.rs`).

mod common;

use std::collections::HashMap;

use acquisition_search::Corpus;
use common::generated::*;
use proptest::prelude::*;
use serde_json::{Value, json};

/// A template a row displays, with a slot it has: the two rows of the
/// generated mod of two foremost, the second above all, whose number is
/// not its mod's first.
fn named() -> BoxedStrategy<(&'static str, &'static str)> {
    proptest::sample::select(vec![
        (LIFE, "arg1"),
        (LIFE, "arg1"),
        (LIFE, "arg1"),
        (COLD, "arg1"),
        (COLD, "arg1"),
        (COLD, "arg1"),
        (COLD, "arg1"),
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
    let line = (named(), restriction(), compared(), 0u8..7).prop_map(
        |((template, slot), restriction, compared, shape)| match shape {
            0 => format!("line(\"{template}\"{restriction})"),
            1 => format!("line(\"{template}\"{restriction} {slot}{compared})"),
            2 => format!("line(\"{template}\"{restriction} -{slot}{compared})"),
            3 => format!("sum(line(\"{template}\"{restriction}).{slot}){compared}"),
            4 => format!("line(\"{template}\"{restriction}).{slot}{compared}"),
            // a not of several: either's not, and both's
            5 => format!("line(\"{template}\" -({slot}{compared}{restriction}))"),
            _ => format!("line(\"{template}\" -({slot}{compared} or source=implicit))"),
        },
    );
    // a row named and words that hold of the same row, the mod's first
    // with a number: one number, read by the row and by the mod in order
    // (the second look's finding 2)
    let twice = (
        proptest::sample::select(vec![(LIFE, "life"), (ALL_RES, "elemental")]),
        compared(),
        restriction(),
        0u8..3,
    )
        .prop_map(|((template, words), compared, restriction, shape)| {
            let either = format!("(\"{template}\" or template:{words}){restriction}");
            match shape {
                0 => format!("sum(line({either}).arg1){compared}"),
                1 => format!("line({either} arg1{compared})"),
                _ => format!("line({either}).arg1{compared}"),
            }
        });
    // alternatives inside one group, each naming its own row: what the
    // same alternatives say at the item's level, a mod written apart or
    // not (the outside review of step 9c4, finding 1)
    let either = (
        (named(), compared()),
        (named(), compared()),
        restriction(),
        0u8..6,
    )
        .prop_map(
            |(((a, slot_a), cmp_a), ((b, slot_b), cmp_b), restriction, shape)| {
                let (a, b) = (format!("\"{a}\""), format!("\"{b}\""));
                match shape {
                    0 => format!("line(({a} {slot_a}{cmp_a}) or ({b} {slot_b}{cmp_b}))"),
                    1 => format!(
                        "line(({a}{restriction} {slot_a}{cmp_a}) or ({b} -{slot_b}{cmp_b}))"
                    ),
                    2 => format!("line(({a} or {b}){restriction} arg1{cmp_a})"),
                    3 => format!("line(({a} {slot_a}{cmp_a}) or ({b} -{b}))"),
                    4 => format!("sum(line(({a} or {b}){restriction}).arg1){cmp_a}"),
                    _ => format!("line({a} or ({b}{restriction})).arg1{cmp_b}"),
                }
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
        4 => either,
        4 => twice,
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
        (named(), named()).prop_map(|((a, _), (b, _))| format!("line(\"{a}\" or \"{b}\").arg1")),
        named().prop_map(|(template, slot)| format!("line(\"{template}\").{slot}")),
        named().prop_map(|(template, slot)| format!("sum(line(\"{template}\").{slot})")),
        proptest::sample::select(vec!["pseudo.total_res", "pseudo.total_all_ele_res"])
            .prop_map(str::to_string),
    ]
    .boxed()
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

/// An item of one mod displayed over several rows, or an item as the
/// dice give it: an item of several lines matches by any of them, which
/// hides what one mod answered.
fn item() -> BoxedStrategy<Body> {
    let one_mod = line(true).prop_map(|mut mod_| {
        mod_.kind = 9 + mod_.kind % 2;
        Body {
            explicit: Lines::Of(vec![Elem::Line(mod_)]),
            ..Body::blank(Tri::No)
        }
    });
    prop_oneof![2 => one_mod, 1 => body(true)].boxed()
}

/// An item of one line, whatever the line: a mod of several rows two
/// times in five.
fn of_one_line() -> BoxedStrategy<Body> {
    (line(true), any::<bool>(), 0u8..5)
        .prop_map(|(mut line, implicit, several)| {
            if several < 2 {
                line.kind = 9 + several;
            }
            let lines = Lines::Of(vec![Elem::Line(line)]);
            let mut body = Body::blank(Tri::No);
            if implicit {
                body.implicit = lines;
            } else {
                body.explicit = lines;
            }
            body
        })
        .boxed()
}

/// Who matched a query and whom it left open; none where it is refused.
fn matched_and_open(corpus: &Corpus, scope: &Ids, q: &Q) -> Result<Option<(Ids, Ids)>, String> {
    let text = q_text(q, Spelling::Authored);
    let Ok(a) = run(corpus, &request(&text, None, false, scope.len().max(1))) else {
        return Ok(None);
    };
    let matched: Ids = a["rows"]
        .as_array()
        .ok_or("no rows")?
        .iter()
        .filter_map(|row| row["id"].as_str().map(str::to_string))
        .collect();
    let mut cache = HashMap::new();
    let open = members(corpus, &a["total"]["undecided"], scope, &mut cache)
        .map_err(|e| format!("`{text}`: {e}"))?;
    Ok(Some((matched, open)))
}

fn a_group_or_its_not_holds(corpus: &Corpus, scope: &Ids, g: &G) -> Result<(), String> {
    let is = Q::Line(g.clone());
    let not = Q::Line(G::Not(Box::new(g.clone())));
    let (Some((is_in, is_open)), Some((not_in, not_open))) = (
        matched_and_open(corpus, scope, &is)?,
        matched_and_open(corpus, scope, &not)?,
    ) else {
        return Ok(());
    };
    let text = q_text(&is, Spelling::Authored);
    if let Some(both) = is_in.intersection(&not_in).next() {
        return Err(format!("`{text}` and its not both hold on {both}"));
    }
    if is_open != not_open {
        return Err(format!(
            "`{text}` leaves {is_open:?} open and its not {not_open:?}"
        ));
    }
    let said: Ids = is_in.union(&not_in).chain(&is_open).cloned().collect();
    if said != *scope {
        let neither: Vec<&String> = scope.difference(&said).collect();
        return Err(format!("neither `{text}` nor its not holds on {neither:?}"));
    }
    Ok(())
}

fn a_groups_or_is_the_items(corpus: &Corpus, scope: &Ids, a: &G, b: &G) -> Result<(), String> {
    let inside = Q::Line(G::Or(vec![a.clone(), b.clone()]));
    let outside = Q::Or(vec![Q::Line(a.clone()), Q::Line(b.clone())]);
    let (Some(one), Some(two)) = (
        matched_and_open(corpus, scope, &inside)?,
        matched_and_open(corpus, scope, &outside)?,
    ) else {
        return Ok(());
    };
    if one != two {
        return Err(format!(
            "`{}` says {one:?} and `{}` {two:?}",
            q_text(&inside, Spelling::Authored),
            q_text(&outside, Spelling::Authored)
        ));
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: cases(192), failure_persistence: None, ..ProptestConfig::default()
    })]

    #[test]
    fn c93_on_one_occurrence_a_group_or_its_not_holds(
        bodies in proptest::collection::vec(of_one_line(), 1..6),
        groups in proptest::collection::vec(group(), 1..4),
    ) {
        let (corpus, scope) = fixture(bodies.iter().map(Body::json).collect());
        for g in &groups {
            a_group_or_its_not_holds(&corpus, &scope, g).map_err(TestCaseError::fail)?;
        }
    }

    #[test]
    fn c91_a_groups_or_is_the_items_or_of_the_same_alternatives(
        bodies in proptest::collection::vec(of_one_line(), 1..6),
        a in group(), b in group(),
    ) {
        let (corpus, scope) = fixture(bodies.iter().map(Body::json).collect());
        a_groups_or_is_the_items(&corpus, &scope, &a, &b).map_err(TestCaseError::fail)?;
    }

    #[test]
    fn c90_a_row_named_answers_as_the_row_alone(
        bodies in proptest::collection::vec(item(), 1..6),
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
    // an item for each line: an item of several matches by any of them
    let bodies: Vec<Body> = lines
        .iter()
        .map(|line| Body {
            explicit: Lines::Of(vec![Elem::Line(line.clone())]),
            ..Body::blank(Tri::No)
        })
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
        format!("line((\"{LIFE}\" arg1>=1) or arg1=999)"),
        format!("line((\"{COLD}\" arg1>=10) or (\"{LIFE}\" source=scourge))"),
        format!("line((\"{LIFE}\" or \"{COLD}\") arg1>=10)"),
        format!("sum(line(\"{LIFE}\" or \"{COLD}\").arg1)=9"),
        format!("sum(line(\"{LIFE}\" or template:life).arg1)>=2"),
        format!("line((\"{ALL_RES}\" or template:elemental) arg1>=13)"),
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
