//! C104 — the text and the tree are one value, each printing to the other:
//! the round trip, over generated trees and over the corpus
//! (`tests/language.toml`), where every construct of the reference
//! (`search/DESIGN.md`, "The language reference") has a case. The boundary
//! pinned is the crate's: `parse`, `print`, `to_json`, `from_json`.

use acquisition_search::{
    Collection, LanguageError, Member, Node, Number, Op, Probe, Value, ValueRef, check, from_json,
    parse, parse_value, print, print_value, to_json,
};
use proptest::prelude::*;
use serde::Deserialize;

// ---- the corpus ------------------------------------------------------------

#[derive(Deserialize)]
struct Corpus {
    case: Vec<Case>,
    value: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    construct: String,
    text: String,
    canonical: Option<String>,
    error: Option<String>,
    readings: Option<Vec<String>>,
}

/// The constructs of the reference, section by section, and the errors the
/// grammar defines. A construct with no case in the corpus fails the test,
/// and so does a case naming a construct that is not here.
const CONSTRUCTS: &[&str] = &[
    // Composition
    "composition.empty",
    "composition.and",
    "composition.or",
    "composition.not",
    "composition.parens",
    "composition.refinement",
    "composition.mixed_and_or",
    "composition.holds",
    "composition.holds_needs_bound",
    "composition.undecided_thing",
    "composition.undecided_term",
    "composition.constants",
    // Strings
    "strings.escapes",
    "strings.bad_escape",
    "strings.unterminated",
    "strings.multirow_template",
    "strings.template_sign",
    "strings.template_negative",
    // Item-level
    "item.phrase",
    "item.template_alone",
    "item.literal_hash",
    "item.bare_word",
    "item.contains",
    "item.whole",
    "item.pattern",
    "item.header",
    "item.text_field",
    "item.closed_set",
    "item.number",
    "item.range",
    "item.range_needs_equals",
    "item.comparison_needs_number",
    "item.spaced_operator",
    "item.unquoted_value",
    "item.has",
    "item.is",
    "item.place",
    "item.id",
    "item.realm_is_scope",
    "item.unknown_call",
    // Members
    "members.line",
    "members.template_forms",
    "members.linked",
    "members.inside",
    "members.bare_word",
    "members.empty",
    "members.not_inside_group",
    "members.template_with_numbers",
    // Slots
    "slots.args",
    "slots.ranged",
    "slots.not_ranged",
    "slots.two_pairs",
    "slots.paren_range",
    "slots.beyond",
    "slots.unchecked_selector",
    // Values
    "values.shorthand",
    "values.projection_lowers",
    "values.slotless_several",
    "values.slotless_none",
    "values.slots_outside_group",
    "values.group_shorthand",
    "values.sum",
    "values.sum_slotless",
    "values.pseudo",
    "values.pseudo_unbuilt",
    "values.pseudo_slot",
    "values.pseudo_needs_comparison",
    "values.names",
    "values.has_on_computed",
    "values.sockets",
    "values.price",
    // What --sort and --sum take
    "sort.projection",
    "sort.slotless",
    "sort.field",
    "sort.computed",
    // The reference's worked example, and the acceptance set
    "worked_example",
    "acceptance.OQ1",
    "acceptance.OQ2",
    "acceptance.OQ3",
    "acceptance.OQ4",
    "acceptance.OQ5",
    "acceptance.OQ6",
    "acceptance.OQ7",
    "acceptance.AQ2",
    "acceptance.AQ3",
    "acceptance.AQ4",
    "acceptance.AQ5",
];

fn corpus() -> Corpus {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/language.toml");
    toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn kind_of(e: &LanguageError) -> String {
    serde_json::to_value(e.kind)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

/// A text that parses prints as its canonical form, and the canonical form
/// is a fixed point: it parses to the same tree and prints as itself. The
/// tree survives its JSON form, as a value and as a string.
fn assert_round_trip(text: &str, canonical: &str) {
    let tree = parse(text).unwrap_or_else(|e| panic!("`{text}` did not parse: {e}"));
    assert_eq!(print(&tree), canonical, "the canonical form of `{text}`");
    assert_eq!(
        parse(canonical).unwrap(),
        tree,
        "`{canonical}` parses to the tree of `{text}`"
    );
    assert_eq!(
        from_json(&to_json(&tree)).unwrap(),
        tree,
        "the JSON form of `{text}`"
    );
    let wire = serde_json::to_string(&tree).unwrap();
    assert_eq!(
        serde_json::from_str::<Node>(&wire).unwrap(),
        tree,
        "the wire form of `{text}`"
    );
}

fn assert_error(case: &Case, e: &LanguageError) {
    let expected = case.error.as_deref().unwrap();
    assert_eq!(kind_of(e), expected, "`{}`: {e}", case.text);
    assert!(
        e.span.is_none_or(|(a, b)| a <= b && b <= case.text.len()),
        "`{}`: the span",
        case.text
    );
    if let Some(readings) = &case.readings {
        assert_eq!(&e.readings, readings, "the readings of `{}`", case.text);
    }
}

#[test]
fn c104_every_case_of_the_corpus_round_trips_or_refuses_by_kind() {
    let corpus = corpus();
    for case in &corpus.case {
        match (&case.canonical, &case.error) {
            (Some(canonical), None) => assert_round_trip(&case.text, canonical),
            (None, Some(_)) => match parse(&case.text) {
                Ok(tree) => panic!("`{}` parsed as `{}`", case.text, print(&tree)),
                Err(e) => assert_error(case, &e),
            },
            _ => panic!(
                "`{}`: a case gives `canonical` or `error`, one of them",
                case.text
            ),
        }
    }
    for case in &corpus.value {
        match (&case.canonical, &case.error) {
            (Some(canonical), None) => {
                let value =
                    parse_value(&case.text).unwrap_or_else(|e| panic!("`{}`: {e}", case.text));
                assert_eq!(
                    &print_value(&value),
                    canonical,
                    "the canonical form of `{}`",
                    case.text
                );
                assert_eq!(parse_value(canonical).unwrap(), value);
            }
            (None, Some(_)) => assert_error(case, &parse_value(&case.text).unwrap_err()),
            _ => panic!(
                "`{}`: a case gives `canonical` or `error`, one of them",
                case.text
            ),
        }
    }
}

/// C91 — an error shows its readings, and a reading is never notation the
/// parser itself refuses: an item-level reading parses as a query, a
/// member-level one inside its group. (An unknown call's readings are call
/// names, and a reading with `…` in it leaves a number for the author.)
#[test]
fn c91_every_reading_an_error_shows_is_a_text_the_parser_accepts() {
    for case in corpus().case.iter().filter(|c| c.error.is_some()) {
        let e = parse(&case.text).unwrap_err();
        if kind_of(&e) == "unknown_call" {
            continue;
        }
        for reading in e.readings.iter().filter(|r| !r.contains('…')) {
            let accepted = parse(reading).is_ok() || parse(&format!("line({reading})")).is_ok();
            assert!(
                accepted,
                "`{}` offers `{reading}`, which does not parse",
                case.text
            );
        }
    }
}

#[test]
fn every_construct_of_the_reference_has_a_case_and_every_case_a_construct() {
    let corpus = corpus();
    let cases: Vec<&Case> = corpus.case.iter().chain(&corpus.value).collect();
    for case in &cases {
        assert!(
            CONSTRUCTS.contains(&case.construct.as_str()),
            "`{}` is no construct of the list",
            case.construct
        );
    }
    println!("{:<34} cases  first case", "construct");
    for construct in CONSTRUCTS {
        let mine: Vec<&&Case> = cases.iter().filter(|c| c.construct == *construct).collect();
        assert!(!mine.is_empty(), "the corpus has no case for `{construct}`");
        println!(
            "{construct:<34} {:>5}  {}",
            mine.len(),
            mine[0].text.replace('\n', "\\n")
        );
    }
}

// ---- invariant 1 of the surface ----------------------------------------------

/// Canonicalisation lowers shorthand and normalises spelling; it never
/// reorders, flattens, merges, deduplicates or simplifies.
#[test]
fn invariant_1_the_printer_never_reorders_flattens_merges_deduplicates_or_simplifies() {
    let kept = [
        // nesting is kept: two trees, two texts
        "(rarity=rare is:corrupted) ilvl>=84",
        "rarity=rare is:corrupted ilvl>=84",
        "rarity=rare (is:corrupted ilvl>=84)",
        // order is kept
        "ilvl>=84 rarity=rare",
        // a repeated term stays repeated
        "rarity=rare rarity=rare",
        "line(\"# to maximum Life\" arg1>=90 arg1>=90)",
        // nothing is simplified: a double negation, a constant, a bound that implies another
        "--is:corrupted",
        "true() rarity=rare",
        "rarity=rare or false()",
        "ilvl>=84 ilvl>=80",
        // two groups over one template are never merged into one occurrence
        "line(\"Adds # to # Cold Damage\" low>=15) line(\"Adds # to # Cold Damage\" high<=45)",
    ];
    for text in kept {
        assert_round_trip(text, text);
    }
    let trees: Vec<Node> = kept.iter().map(|t| parse(t).unwrap()).collect();
    for (i, a) in trees.iter().enumerate() {
        for b in &trees[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

// ---- the JSON form ---------------------------------------------------------------

/// The reference's worked example, as written there: the request's tree
/// prints as the answer's `query` line.
#[test]
fn c104_the_worked_example_tree_prints_as_its_query_line() {
    let tree = serde_json::json!({ "all": [
        { "field": "league", "op": "=", "value": "Standard" },
        { "field": "class",  "op": "=", "value": "ring" },
        { "field": "rarity", "op": "=", "value": "rare" },
        { "exists": "lines", "where": { "all": [
            { "attr": "template", "op": "=",  "value": "# to maximum Life" },
            { "attr": "arg1",     "op": ">=", "value": 90 } ] } },
        { "value": { "pseudo": "total_res" }, "op": ">=", "number": 60 } ] });
    let node = from_json(&tree).unwrap();
    assert_eq!(
        print(&node),
        r##"league=Standard class=ring rarity=rare line("# to maximum Life" arg1>=90) pseudo.total_res>=60"##
    );
    assert_eq!(
        to_json(&node),
        tree,
        "a tree that is read re-serializes to exactly what was read"
    );
}

/// A tree that arrives as JSON is read strictly and checked before use.
#[test]
fn c104_a_tree_the_language_cannot_say_is_refused() {
    let refused = [
        // an unknown key beside a known one; a missing one
        serde_json::json!({ "field": "ilvl", "op": ">=", "value": 84, "note": "x" }),
        serde_json::json!({ "field": "ilvl", "op": ">=" }),
        serde_json::json!({ "all": [], "any": [] }),
        serde_json::json!({ "nothing": 1 }),
        // a group of one is its member
        serde_json::json!({ "all": [ { "is": "corrupted" } ] }),
        serde_json::json!({ "not": { "any": [] } }),
        // the grammar's own errors hold for a tree as for a text
        serde_json::json!({ "field": "realm", "op": ":", "value": "pc" }),
        serde_json::json!({ "has": "pseudo.total_res" }),
        serde_json::json!({ "holds": [ { "is": "corrupted" } ] }),
        serde_json::json!({ "field": "ilvl", "op": ">=", "value": { "from": 1, "to": 2 } }),
        serde_json::json!({ "field": "ilvl", "op": ">=", "value": "high" }),
        serde_json::json!({ "field": "arg1", "op": ">=", "value": 90 }),
        serde_json::json!({ "field": "text", "op": ":", "value": "" }),
        // a slot the quoted template does not have; a template typed with its numbers
        serde_json::json!({ "exists": "lines", "where": { "all": [
            { "attr": "template", "op": "=", "value": "# to maximum Life" },
            { "attr": "low", "op": ">=", "value": 90 } ] } }),
        serde_json::json!({ "exists": "lines", "where": { "attr": "template", "op": "=", "value": "+92 to maximum Life" } }),
        // a sign before a # is spelling, which a tree never holds
        serde_json::json!({ "exists": "lines", "where": { "any": [
            { "attr": "template", "op": "=", "value": "+# to maximum Life" },
            { "is": "crafted" } ] } }),
        // a comparison on a projection lowers into the group: a tree never holds it
        serde_json::json!({ "value": { "lines": { "attr": "template", "op": "=", "value": "# to maximum Life" }, "slot": "arg1" }, "op": ">=", "number": 90 }),
    ];
    for json in &refused {
        assert!(from_json(json).is_err(), "{json} was accepted");
    }
    // …and two that are fine, so the list above is not refusing everything.
    assert!(
        from_json(&serde_json::json!({ "field": "ilvl", "op": ">=", "value": f64::MAX })).is_ok()
    );
    assert!(
        from_json(&serde_json::json!({ "all": [] })).is_ok(),
        "the empty query"
    );
    // 90.0 is 90: a number has one tree.
    assert_eq!(
        from_json(&serde_json::json!({ "field": "ilvl", "op": ">=", "value": 90.0 })).unwrap(),
        parse("ilvl>=90").unwrap()
    );
}

/// A whole number is an `Int` exactly when an i64 holds it, which is what
/// the parser reads such digits as: the edge a generated f64 rarely lands on.
#[test]
fn c104_a_whole_number_at_the_edge_of_an_i64_has_one_tree() {
    let two_63 = 9_223_372_036_854_775_808.0_f64;
    for f in [
        9.089555452199103e18,
        two_63,
        -two_63,
        two_63 - 1024.0,
        9.0e15,
        -0.0,
        1.0e300,
    ] {
        let tree = Node::Test {
            field: "ilvl".into(),
            op: Op::Ge,
            value: Value::Number(Number::from_f64(f)),
        };
        assert!(check(&tree).is_ok());
        assert_eq!(parse(&print(&tree)), Ok(tree), "{f:e}");
    }
    assert_eq!(
        Number::from_f64(two_63 - 1024.0),
        Number::Int(9_223_372_036_854_774_784)
    );
    assert_eq!(Number::from_f64(two_63), Number::Float(two_63));
    assert_eq!(Number::from_f64(-two_63), Number::Int(i64::MIN));
}

// ---- generated trees -------------------------------------------------------------

const FIELDS: &[&str] = &[
    "name",
    "typeline",
    "base",
    "text",
    "rarity",
    "class",
    "ilvl",
    "league",
    "tab",
    "id",
    "sockets",
    "sockets.red",
    "price.amount",
    "reqlevel",
    "line",
    "holds",
    "true",
];
const NAMES: &[&str] = &[
    "corrupted",
    "fractured",
    "note",
    "priced",
    "price.lot",
    "reqlevel",
];
const ATTRS: &[&str] = &["source", "template", "red", "green", "size", "kind"];
/// Quoted templates with the slot words each takes (the reference, *Slots*).
const TEMPLATES: &[(&str, &[&str])] = &[
    ("# to maximum Life", &["arg1"]),
    (
        "Adds # to # Cold Damage",
        &["low", "high", "avg", "arg1", "arg2"],
    ),
    (
        "# to # Added Physical Damage per # Armour or Evasion Rating on Shield",
        &["low", "high", "avg", "arg1", "arg2", "arg3"],
    ),
    (
        "Adds # to # Fire Damage and # to # Cold Damage",
        &["arg1", "arg2", "arg3", "arg4"],
    ),
    ("Bow: Adds (#-#) to (#-#) Cold Damage", &["arg1", "arg4"]),
    (
        "Monsters' Action Speed cannot be modified\nbelow \"Base\" \\ Value by #%",
        &["arg1"],
    ),
];
const SLOTS: &[&str] = &["low", "high", "avg", "arg1", "arg2", "arg12"];

fn pick(of: &'static [&'static str]) -> impl Strategy<Value = String> {
    proptest::sample::select(of).prop_map(str::to_string)
}

fn text() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-zA-Z_][a-zA-Z0-9_]{0,8}",
        "[0-9]{1,4}",
        Just("and".to_string()),
        Just("OR".to_string()),
        // quotes, backslashes, row breaks, `#`, operators, other scripts
        "[ -~\\n]{0,12}",
        "\\PC{0,8}",
    ]
}

/// Numbers as an author writes them: whole, or a few decimal places. Every
/// finite f64 has its own test below, through the text alone — the wire
/// form reads an extreme float to within one unit in the last place
/// (serde_json without `float_roundtrip`), which no typed bound reaches.
fn number() -> impl Strategy<Value = Number> {
    prop_oneof![
        any::<i64>().prop_map(Number::Int),
        (-100_000_000i64..100_000_000).prop_map(|n| Number::from_f64(n as f64 / 10_000.0)),
    ]
}

fn range() -> impl Strategy<Value = Value> {
    (number(), number(), 0..3u8).prop_map(|(a, b, shape)| {
        let (low, high) = if a.as_f64() <= b.as_f64() {
            (a, b)
        } else {
            (b, a)
        };
        match shape {
            0 => Value::Range {
                from: Some(low),
                to: None,
            },
            1 => Value::Range {
                from: None,
                to: Some(high),
            },
            _ => Value::Range {
                from: Some(low),
                to: Some(high),
            },
        }
    })
}

/// An operator with a right-hand side it takes.
fn op_value() -> impl Strategy<Value = (Op, Value)> {
    prop_oneof![
        text().prop_map(|t| (Op::Contains, Value::Text(t))),
        number().prop_map(|n| (Op::Contains, Value::Number(n))),
        text().prop_map(|t| (Op::Eq, Value::Text(t))),
        text().prop_map(|t| (Op::Match, Value::Text(t))),
        range().prop_map(|r| (Op::Eq, r)),
        (
            proptest::sample::select(&[Op::Eq, Op::Gt, Op::Ge, Op::Lt, Op::Le][..]),
            number()
        )
            .prop_map(|(op, n)| (op, Value::Number(n))),
    ]
}

fn numeric() -> impl Strategy<Value = (Op, Value)> {
    prop_oneof![
        range().prop_map(|r| (Op::Eq, r)),
        (
            proptest::sample::select(&[Op::Eq, Op::Gt, Op::Ge, Op::Lt, Op::Le][..]),
            number()
        )
            .prop_map(|(op, n)| (op, Value::Number(n))),
    ]
}

/// A member's condition. `slots`: the slot words it may name; a quoted
/// template is only ever added by `lines`, which knows the words it takes.
fn member(slots: &'static [&'static str]) -> impl Strategy<Value = Member> {
    let leaf = prop_oneof![
        (pick(ATTRS), op_value())
            .prop_filter(
                "a quoted template is `lines`' to add",
                |(attr, (op, value))| {
                    !(attr == "template" && *op == Op::Eq && matches!(value, Value::Text(_)))
                }
            )
            .prop_map(|(attr, (op, value))| Member::Test { attr, op, value }),
        (pick(slots), numeric()).prop_map(|(attr, (op, value))| Member::Test { attr, op, value }),
        pick(NAMES).prop_map(Member::Is),
        any::<bool>().prop_map(Member::Const),
    ];
    leaf.prop_recursive(3, 12, 3, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 2..4).prop_map(Member::All),
            proptest::collection::vec(inner.clone(), 2..4).prop_map(Member::Any),
            inner.prop_map(|m| Member::Not(Box::new(m))),
        ]
    })
}

/// A line's group and a slot word it takes: over one quoted template, whose
/// own numbers say which slots exist, or over none, where any slot may be
/// named.
fn lines() -> impl Strategy<Value = (Member, String)> {
    let quoted = proptest::sample::select(TEMPLATES).prop_flat_map(|(template, slots)| {
        let template_test = Member::Test {
            attr: "template".to_string(),
            op: Op::Eq,
            value: Value::Text(template.to_string()),
        };
        (
            proptest::collection::vec(member(slots), 0..3),
            0..3usize,
            pick(slots),
        )
            .prop_map(move |(mut rest, at, slot)| {
                rest.insert(at.min(rest.len()), template_test.clone());
                let group = if rest.len() == 1 {
                    rest.remove(0)
                } else {
                    Member::All(rest)
                };
                (group, slot)
            })
    });
    prop_oneof![quoted, (member(SLOTS), pick(SLOTS))]
}

fn value_ref() -> impl Strategy<Value = ValueRef> {
    prop_oneof![
        ("[a-z][a-z0-9_]{0,8}", proptest::option::of(pick(SLOTS)))
            .prop_filter("a name is never a word of the grammar", |(name, _)| {
                !["and", "or", "not"].contains(&name.as_str())
            })
            .prop_map(|(name, slot)| ValueRef::Pseudo { name, slot }),
        lines().prop_map(|(lines, slot)| ValueRef::Sum {
            lines: Box::new(lines),
            slot
        }),
    ]
}

fn node() -> impl Strategy<Value = Node> {
    let leaf = prop_oneof![
        (pick(FIELDS), op_value())
            .prop_filter("an empty phrase finds nothing", |(field, (op, value))| {
                !(field == "text" && *op == Op::Contains && *value == Value::Text(String::new()))
            })
            .prop_map(|(field, (op, value))| Node::Test { field, op, value }),
        pick(NAMES).prop_map(Node::Has),
        pick(NAMES).prop_map(Node::Is),
        any::<bool>().prop_map(Node::Const),
        lines().prop_map(|(group, _)| Node::Members {
            of: Collection::Lines,
            where_: Box::new(group)
        }),
        member(&["red", "blue", "size"]).prop_map(|group| Node::Members {
            of: Collection::Links,
            where_: Box::new(group)
        }),
        (value_ref(), numeric()).prop_map(|(value, (op, rhs))| Node::Compare { value, op, rhs }),
        value_ref().prop_map(|v| Node::Undecided(Probe::Thing(v))),
        pick(FIELDS).prop_map(|f| Node::Undecided(Probe::Thing(ValueRef::Field(f)))),
        lines().prop_map(|(lines, slot)| {
            Node::Undecided(Probe::Thing(ValueRef::Projection {
                lines: Box::new(lines),
                slot,
            }))
        }),
    ];
    leaf.prop_recursive(4, 24, 4, |inner| {
        prop_oneof![
            proptest::collection::vec(inner.clone(), 2..5).prop_map(Node::All),
            proptest::collection::vec(inner.clone(), 2..5).prop_map(Node::Any),
            inner.clone().prop_map(|n| Node::Not(Box::new(n))),
            inner
                .clone()
                .prop_map(|n| Node::Undecided(Probe::Term(Box::new(n)))),
            (
                proptest::collection::vec(inner, 1..4),
                0..5u32,
                0..5u32,
                0..3u8
            )
                .prop_map(|(of, a, b, shape)| {
                    let (low, high) = (a.min(b), a.max(b));
                    let (min, max) = match shape {
                        0 => (Some(low), None),
                        1 => (None, Some(high)),
                        _ => (Some(low), Some(high)),
                    };
                    Node::Holds { of, min, max }
                }),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// C104 — every tree the language can say prints to a text that parses
    /// back to it, and that text is canonical: printing the parsed text
    /// gives it again. The JSON form makes the same trip.
    #[test]
    fn c104_a_generated_tree_survives_its_text_and_its_json(tree in node()) {
        prop_assert!(check(&tree).is_ok(), "the generator made an invalid tree: {:?}", check(&tree));
        let text = print(&tree);
        let parsed = parse(&text);
        prop_assert_eq!(parsed.as_ref(), Ok(&tree), "the text was `{}`", text);
        prop_assert_eq!(print(&parsed.unwrap()), text);
        prop_assert_eq!(from_json(&to_json(&tree)), Ok(tree.clone()));
        let wire = serde_json::to_string(&tree).unwrap();
        prop_assert_eq!(serde_json::from_str::<Node>(&wire).ok(), Some(tree));
    }

    /// A number has one spelling, and it reads back as the same number.
    #[test]
    fn c104_any_finite_number_survives_its_text(f in any::<f64>().prop_filter("finite", |f| f.is_finite())) {
        let tree = Node::Test { field: "ilvl".into(), op: Op::Ge, value: Value::Number(Number::from_f64(f)) };
        prop_assert_eq!(parse(&print(&tree)), Ok(tree));
    }

    /// What `--sort` and `--sum` take makes the same trip.
    #[test]
    fn c104_a_generated_value_survives_its_text(value in prop_oneof![
        value_ref(),
        pick(FIELDS).prop_map(ValueRef::Field),
        lines().prop_map(|(lines, slot)| ValueRef::Projection { lines: Box::new(lines), slot }),
    ]) {
        let text = print_value(&value);
        prop_assert_eq!(parse_value(&text), Ok(value), "the text was `{}`", text);
    }

    /// C47 — no text makes the parser panic: it is a query or a structured
    /// error whose span lies inside the text.
    #[test]
    fn c47_any_text_is_a_query_or_a_structured_error(text in "[ -~\\n]{0,40}|\\PC{0,20}") {
        if let Err(e) = parse(&text) {
            prop_assert!(e.span.is_none_or(|(a, b)| a <= b && b <= text.len()), "{:?}", e);
        }
    }
}
