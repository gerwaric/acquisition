//! Generators for step 4b's properties (the build plan, "Step 4b"):
//! queries as a tree of this file's own, so that a rewrite is made on the
//! tree and only text crosses the crate's boundary, and item bodies with
//! holes — something the deriver cannot read — that a completion fills.
//!
//! Since step 7's third look the generators reach the computed values
//! (`SEARCH-SLICE.md`, "Findings"): `pseudo.total_res` over resistance
//! lines of three templates and two weights, `pseudo.dps` and
//! `pseudo.pdps` over a `properties` array — as comparisons, `has:`,
//! probes and sorts in the one composition — and the array's own holes:
//! no array, an element that is no property, a property whose values
//! cannot be read, and a value the fields cannot read as a number.
//!
//! Since step 8 they reach the sockets (C101): `sockets`, `links` and
//! `sockets.<colour>` as comparisons, `has:` and sorts, `linked( … )` as
//! a group of colour and size counts in the one composition — and the
//! collection's own holes: no array, an element that is no socket, a
//! socket whose colour could not be read (an `attr` and no `sColour`),
//! and one whose group could not.
//!
//! Since step 9 they reach the price (C81, C100): the fixture's tab is
//! public and carries the owner's row, a note is a game price, a bulk
//! ratio, a `~skip`, a word that is no price, or a hole — a note that is no
//! string, which the store carries as unread and the listing state leaves
//! unresolved — and `has:priced`, `price.amount`, `price.currency`,
//! `price.lot` join the composition as comparisons, `has:`, probes and
//! sorts.
//!
//! Nothing here reads the crate's tree, binder or evaluator: a query is
//! rendered to text and asked, and a body enters through `Store::record`;
//! a row enters through the pricing area's one write.

use std::collections::{BTreeSet, HashMap};

use acquisition_plan::price::{Buyout, Price, PriceTarget, set_buyout};
use acquisition_search::{Corpus, Request, answer};
use acquisition_store::{Annotations, Endpoint, Provenance, Store};
use proptest::prelude::*;
use serde_json::{Map, Value, json};

pub type Ids = BTreeSet<String>;

// ---- the templates every generator shares -------------------------------------------------

pub const LIFE: &str = "# to maximum Life";
pub const COLD: &str = "#% to Cold Resistance";
pub const ADDS: &str = "Adds # to # Cold Damage";
pub const FROZEN: &str = "Cannot be Frozen";
pub const SPIRIT: &str = "# to Spirit";
/// A line whose numbers are decimals: tenths, from the same small range.
pub const LEECH: &str = "#% of Damage Leeched as Life";
/// Two more rows of `pseudo.total_res` beside `COLD`'s: one at weight 1,
/// one at weight 3, so the total sums several templates by weight.
pub const FIRE: &str = "#% to Fire Resistance";
pub const ALL_RES: &str = "#% to all Elemental Resistances";

/// Values and bounds share one small range, so that a bound lands
/// immediately below, at and above a value, zero and negatives included
/// (the outside agent's transformation 9).
const LOW: i32 = -3;
const HIGH: i32 = 12;

// ---- a line's group -------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum G {
    Leaf(String),
    And(Vec<G>),
    Or(Vec<G>),
    Not(Box<G>),
}

fn quoted(template: &str) -> String {
    format!("\"{template}\"")
}

fn template_leaf() -> BoxedStrategy<String> {
    prop_oneof![
        2 => proptest::sample::select(vec![LIFE, COLD, ADDS, FROZEN, SPIRIT, FIRE, ALL_RES])
            .prop_map(quoted),
        3 => proptest::sample::select(vec![
            "template:life",
            "template:resistance",
            "template:cold",
            "template:fire",
            "template:Frozen",
            "template:spirit",
            "template:nothing",
            "template:leeched",
            // a word no line has whole and some have part of: the zero
            // block's suggestions
            "template:lifes",
            "template~\"life|resistance\"",
            "template~\"^adds\"",
            "template~\"nothing\"",
        ])
        .prop_map(str::to_string),
    ]
    .boxed()
}

/// A bound as typed: a whole number of the range, or — one time in four —
/// tenths, which is what a sum of decimal lines is compared with.
fn bound() -> BoxedStrategy<String> {
    prop_oneof![
        3 => (LOW..=HIGH).prop_map(|n| n.to_string()),
        1 => (LOW * 3..=HIGH * 3).prop_map(tenths),
    ]
    .boxed()
}

fn tenths(n: i32) -> String {
    let sign = if n < 0 { "-" } else { "" };
    format!("{sign}{}.{}", n.abs() / 10, n.abs() % 10)
}

fn restriction_leaf() -> BoxedStrategy<String> {
    proptest::sample::select(vec![
        "source=explicit",
        "source=implicit",
        "source=hybrid",
        "source:plicit",
        "is:crafted",
        "is:fractured",
    ])
    .prop_map(str::to_string)
    .boxed()
}

fn comparison_leaf() -> BoxedStrategy<String> {
    (
        prop_oneof![
            16 => Just("arg1"),
            1 => Just("arg2"),
            1 => Just("low"),
            1 => Just("high"),
            1 => Just("avg"),
            1 => Just("arg3"),
        ],
        proptest::sample::select(vec![">=", ">", "<=", "<", "="]),
        LOW..=HIGH,
    )
        .prop_map(|(slot, op, n)| format!("{slot}{op}{n}"))
        .boxed()
}

fn g_leaf() -> BoxedStrategy<G> {
    prop_oneof![
        4 => template_leaf(),
        3 => restriction_leaf(),
        4 => comparison_leaf(),
        1 => proptest::sample::select(vec!["true()", "false()"]).prop_map(str::to_string),
    ]
    .prop_map(G::Leaf)
    .boxed()
}

/// A template and a comparison on a slot it has — nine times in ten: the
/// tenth names a slot it may lack, which is an authoring error however the
/// group is parenthesised.
fn template_and_comparison() -> BoxedStrategy<(String, String)> {
    let fitting = proptest::sample::select(vec![
        (LIFE, "arg1"),
        (COLD, "arg1"),
        (SPIRIT, "arg1"),
        (ADDS, "arg1"),
        (ADDS, "arg2"),
        (ADDS, "low"),
        (ADDS, "high"),
        (ADDS, "avg"),
        (LEECH, "arg1"),
        (FIRE, "arg1"),
        (ALL_RES, "arg1"),
    ])
    .prop_map(|(template, slot)| (quoted(template), slot));
    let worded = (
        proptest::sample::select(vec![
            "template:life",
            "template:resistance",
            "template:cold",
            "template:nothing",
            "template~\"life|resistance\"",
        ]),
        Just("arg1"),
    )
        .prop_map(|(template, slot)| (template.to_string(), slot));
    let fitted = (
        prop_oneof![fitting, worded],
        proptest::sample::select(vec![">=", ">", "<=", "<", "="]),
        bound(),
    )
        .prop_map(|((template, slot), op, n)| (template, format!("{slot}{op}{n}")));
    prop_oneof![9 => fitted, 1 => (template_leaf(), comparison_leaf())].boxed()
}

/// A comparison nested beside a template, a source and a flag restriction,
/// in the shapes the audits broke (transformation 2).
fn shaped() -> BoxedStrategy<G> {
    (template_and_comparison(), restriction_leaf(), 0u8..9)
        .prop_map(|((t, c), r, shape)| {
            let (t, r, c) = (G::Leaf(t), G::Leaf(r), G::Leaf(c));
            let not = |g: G| G::Not(Box::new(g));
            match shape {
                0 => G::And(vec![t, c]),
                1 => G::And(vec![t, r, c]),
                2 => G::And(vec![t, G::And(vec![r, c])]),
                3 => G::And(vec![G::And(vec![t, r]), c]),
                4 => G::And(vec![t, not(not(c))]),
                5 => G::And(vec![G::Or(vec![t, G::Leaf("false()".into())]), c]),
                6 => G::And(vec![t, G::Or(vec![r, c])]),
                7 => G::And(vec![t, not(G::And(vec![r, c]))]),
                _ => G::And(vec![not(not(G::And(vec![t, r]))), c]),
            }
        })
        .boxed()
}

pub fn group() -> BoxedStrategy<G> {
    let free = g_leaf().prop_recursive(3, 12, 3, |inner| {
        prop_oneof![
            3 => proptest::collection::vec(inner.clone(), 2..4).prop_map(G::And),
            2 => proptest::collection::vec(inner.clone(), 2..4).prop_map(G::Or),
            1 => inner.clone().prop_map(|g| G::Not(Box::new(g))),
            1 => inner.prop_map(|g| G::Not(Box::new(G::Not(Box::new(g))))),
        ]
    });
    prop_oneof![3 => shaped(), 2 => free].boxed()
}

fn g_text(g: &G, top: bool, and: &str) -> String {
    let wrapped = |text: String| {
        if top { text } else { format!("({text})") }
    };
    match g {
        G::Leaf(text) => text.clone(),
        G::And(children) => wrapped(
            children
                .iter()
                .map(|c| g_text(c, false, and))
                .collect::<Vec<_>>()
                .join(and),
        ),
        G::Or(children) => wrapped(
            children
                .iter()
                .map(|c| g_text(c, false, and))
                .collect::<Vec<_>>()
                .join(" or "),
        ),
        G::Not(inner) => format!("-({})", g_text(inner, true, and)),
    }
}

/// Nested ands and ors flattened, a doubled not cancelled: the rewrites
/// invariant 7 says change no meaning, undone.
fn g_normal(g: &G) -> G {
    match g {
        G::Leaf(_) => g.clone(),
        G::And(children) => {
            let mut flat = Vec::new();
            for child in children.iter().map(g_normal) {
                match child {
                    G::And(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            G::And(flat)
        }
        G::Or(children) => {
            let mut flat = Vec::new();
            for child in children.iter().map(g_normal) {
                match child {
                    G::Or(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            G::Or(flat)
        }
        G::Not(inner) => match g_normal(inner) {
            G::Not(twice) => *twice,
            other => G::Not(Box::new(other)),
        },
    }
}

fn g_sorted(g: &G) -> G {
    let sorted = |children: &[G]| {
        let mut children: Vec<G> = children.iter().map(g_sorted).collect();
        children.sort_by_key(|c| g_text(c, false, " "));
        children
    };
    match g {
        G::Leaf(_) => g.clone(),
        G::And(children) => G::And(sorted(children)),
        G::Or(children) => G::Or(sorted(children)),
        G::Not(inner) => G::Not(Box::new(g_sorted(inner))),
    }
}

fn g_shuffled(g: &G, rng: &mut Lcg) -> G {
    let shuffled = |children: &[G], rng: &mut Lcg| {
        let mut children: Vec<G> = children.iter().map(|c| g_shuffled(c, rng)).collect();
        rng.shuffle(&mut children);
        children
    };
    match g {
        G::Leaf(_) => g.clone(),
        G::And(children) => G::And(shuffled(children, rng)),
        G::Or(children) => G::Or(shuffled(children, rng)),
        G::Not(inner) => G::Not(Box::new(g_shuffled(inner, rng))),
    }
}

// ---- the item's tree --------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Q {
    /// A term that is no line's: `rarity=rare`.
    Plain(String),
    /// A spelling and the explicit form it lowers to or means
    /// (transformation 11): `"life"` and `text:life`.
    Alt(String, String, TermKind),
    Line(G),
    /// `linked(G)`: counts of colour and size that hold together on one
    /// link group (C101); the leaves are `link_leaf`'s.
    Linked(G),
    /// `sum(line(G).slot)` and its comparison.
    Sum(G, String, String),
    /// `line(G).slot` and its comparison: lowers into the group.
    Proj(G, String, String),
    /// `undecided(line(G).slot)`, or of the sum when `true`.
    Open(G, String, bool),
    /// `undecided(pseudo.<name>)`: a computed value's probe.
    Probe(String),
    /// A named total's comparison, `pseudo.total_res>=6`: a sum over the
    /// item's lines (C94), so it moves as a sum does.
    Total(String),
    And(Vec<Q>),
    Or(Vec<Q>),
    Not(Box<Q>),
    Holds(Vec<Q>, String),
    Undecided(Box<Q>),
}

/// What kind of atomic term a leaf is, for a property that treats a line's
/// group apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermKind {
    Plain,
    Group,
    Sum,
    Probe,
}

fn slot_word() -> BoxedStrategy<String> {
    prop_oneof![
        24 => Just("arg1"),
        1 => Just("arg2"),
        1 => Just("low"),
        1 => Just("avg"),
    ]
    .prop_map(str::to_string)
    .boxed()
}

fn comparison() -> BoxedStrategy<String> {
    (
        proptest::sample::select(vec![">=", ">", "<=", "<", "="]),
        prop_oneof![
            3 => (LOW..=HIGH * 2).prop_map(|n| n.to_string()),
            1 => (LOW * 3..=HIGH * 6).prop_map(tenths),
        ],
    )
        .prop_map(|(op, n)| format!("{op}{n}"))
        .boxed()
}

fn plain() -> BoxedStrategy<Q> {
    prop_oneof![
        20 => proptest::sample::select(vec![
            "rarity=rare",
            "rarity:ma",
            "base:ring",
            "name:doom",
            "name:knott",
            "name~\"nothing\"",
            "name=\"Doom Knot\"",
            "ilvl>=80",
            "ilvl=1..79",
            "has:ilvl",
            "reqlevel<=30",
            "reqlevel=20..45",
            "has:reqlevel",
            "has:note",
            "has:name",
            "note:price",
            "is:corrupted",
            "is:shaper",
            "is:hunter",
            "is:split",
            "is:duplicated",
            "is:replica",
            "is:mutated",
            "\"life\"",
            "\"Level: 8\"",
            "text~\"^adds\"",
            "tab:routes",
            "true()",
            "false()",
            "has:priced",
            "-has:priced",
            "price.amount>=2",
            "price.amount=1..5",
            "price.amount<2",
            "price.currency=chaos",
            "price.currency:div",
            "price.lot>=2",
            "has:price.lot",
            "sockets>=2",
            "sockets=1..3",
            "has:sockets",
            "-has:links",
            "links>=2",
            "links=1",
            "sockets.red>=2",
            "sockets.blue=0",
            "sockets.white>=1",
            "sockets.green<=1",
        ]),
        // an authoring error: never evaluated, and the same error however
        // the rest is written
        1 => proptest::sample::select(vec!["nosuch=1", "class:nosuch", "is:crafted", "ilvl:84"]),
    ]
    .prop_map(|text| Q::Plain(text.to_string()))
    .boxed()
}

/// The price's names (`price.rs`), as `undecided( … )` asks them.
pub const PRICE: [&str; 3] = ["priced", "price.amount", "price.currency"];

/// The computed values (`pseudo.rs`): a total over the resistance lines
/// and the two derived fields over the properties, compared within the
/// bounds of `comparison()`; `has:` of a derived field; and — one time in
/// nine — `has:` of a total, the authoring error T2 rules.
pub const PSEUDO: [&str; 3] = ["pseudo.total_res", "pseudo.dps", "pseudo.pdps"];

fn pseudo_leaf() -> BoxedStrategy<Q> {
    prop_oneof![
        2 => comparison().prop_map(|cmp| Q::Total(format!("pseudo.total_res{cmp}"))),
        4 => (proptest::sample::select(vec!["pseudo.dps", "pseudo.pdps"]), comparison())
            .prop_map(|(value, cmp)| Q::Plain(format!("{value}{cmp}"))),
        2 => proptest::sample::select(vec!["has:pseudo.dps", "has:pseudo.pdps"])
            .prop_map(|text| Q::Plain(text.to_string())),
        1 => Just(Q::Plain("has:pseudo.total_res".to_string())),
    ]
    .boxed()
}

/// A link group's leaves: a colour's count or the group's size, compared
/// within the range a generated collection reaches, and the constants.
fn link_leaf() -> BoxedStrategy<G> {
    prop_oneof![
        6 => (
            proptest::sample::select(vec!["red", "green", "blue", "white", "size"]),
            proptest::sample::select(vec![">=", ">", "<=", "<", "="]),
            0i32..=4,
        )
            .prop_map(|(attr, op, n)| format!("{attr}{op}{n}")),
        1 => proptest::sample::select(vec!["true()", "false()"]).prop_map(str::to_string),
    ]
    .prop_map(G::Leaf)
    .boxed()
}

fn linked() -> BoxedStrategy<G> {
    link_leaf()
        .prop_recursive(2, 8, 3, |inner| {
            prop_oneof![
                3 => proptest::collection::vec(inner.clone(), 2..4).prop_map(G::And),
                2 => proptest::collection::vec(inner.clone(), 2..4).prop_map(G::Or),
                1 => inner.clone().prop_map(|g| G::Not(Box::new(g))),
                1 => inner.prop_map(|g| G::Not(Box::new(G::Not(Box::new(g))))),
            ]
        })
        .boxed()
}

fn alt() -> BoxedStrategy<Q> {
    let pairs: Vec<(String, String, TermKind)> = vec![
        ("\"life\"".into(), "text:life".into(), TermKind::Plain),
        ("RARITY=Rare".into(), "rarity=rare".into(), TermKind::Plain),
        (
            quoted(LIFE),
            format!("line({})", quoted(LIFE)),
            TermKind::Group,
        ),
        (
            "\"+# to maximum Life\">=5".into(),
            format!("line({} arg1>=5)", quoted(LIFE)),
            TermKind::Group,
        ),
        (
            format!("{}.avg>4", quoted(ADDS)),
            format!("line({} avg>4)", quoted(ADDS)),
            TermKind::Group,
        ),
        (
            format!("sum({})>=9", quoted(LIFE)),
            format!("sum(line({}).arg1)>=9", quoted(LIFE)),
            TermKind::Sum,
        ),
        (
            format!("-{}", quoted(COLD)),
            format!("-line({})", quoted(COLD)),
            TermKind::Group,
        ),
        // a computed value's name in any case (B1)
        (
            "pseudo.TOTAL_RES>=9".into(),
            "pseudo.total_res>=9".into(),
            TermKind::Sum,
        ),
        (
            "pseudo.PDPS>=9".into(),
            "pseudo.pdps>=9".into(),
            TermKind::Plain,
        ),
        // a field's name in any case (B1)
        (
            "SOCKETS.Red>=1".into(),
            "sockets.red>=1".into(),
            TermKind::Plain,
        ),
        ("has:PRICED".into(), "has:priced".into(), TermKind::Plain),
        (
            "PRICE.Currency=Chaos".into(),
            "price.currency=chaos".into(),
            TermKind::Plain,
        ),
    ];
    proptest::sample::select(pairs)
        .prop_map(|(spelled, explicit, kind)| Q::Alt(spelled, explicit, kind))
        .boxed()
}

fn q_leaf(probes: bool) -> BoxedStrategy<Q> {
    let open = if probes { 2 } else { 0 };
    prop_oneof![
        6 => group().prop_map(Q::Line),
        3 => (group(), slot_word(), comparison()).prop_map(|(g, s, c)| Q::Sum(g, s, c)),
        2 => (group(), slot_word(), comparison()).prop_map(|(g, s, c)| Q::Proj(g, s, c)),
        open => (group(), slot_word(), any::<bool>()).prop_map(|(g, s, sum)| Q::Open(g, s, sum)),
        open => proptest::sample::select(PSEUDO.to_vec())
            .prop_map(|value| Q::Probe(format!("undecided({value})"))),
        open => proptest::sample::select(PRICE.to_vec())
            .prop_map(|value| Q::Probe(format!("undecided({value})"))),
        4 => plain(),
        3 => pseudo_leaf(),
        2 => alt(),
        2 => linked().prop_map(Q::Linked),
    ]
    .boxed()
}

/// A query. With `probes` off it holds no `undecided( … )`, which says
/// something of the evidence and not of the item, so that no completion is
/// owed the same answer.
pub fn query(probes: bool) -> BoxedStrategy<Q> {
    let undecided = if probes { 1 } else { 0 };
    q_leaf(probes)
        .prop_recursive(3, 10, 3, move |inner| {
            prop_oneof![
                3 => proptest::collection::vec(inner.clone(), 2..4).prop_map(Q::And),
                2 => proptest::collection::vec(inner.clone(), 2..4).prop_map(Q::Or),
                1 => inner.clone().prop_map(|q| Q::Not(Box::new(q))),
                1 => inner.clone().prop_map(|q| Q::Not(Box::new(Q::Not(Box::new(q))))),
                1 => (
                    proptest::collection::vec(inner.clone(), 2..4),
                    proptest::sample::select(vec![">=1", "=1", "<=1", "=1..2", ">=2"]),
                )
                    .prop_map(|(of, bound)| Q::Holds(of, bound.to_string())),
                undecided => inner.prop_map(|q| Q::Undecided(Box::new(q))),
            ]
        })
        .boxed()
}

/// What `--sort` takes, or nothing.
#[derive(Debug, Clone, PartialEq)]
pub enum Sort {
    None,
    Field(&'static str),
    Proj(G, String),
    Sum(G, String),
}

pub fn sort() -> BoxedStrategy<Sort> {
    prop_oneof![
        3 => Just(Sort::None),
        1 => proptest::sample::select(vec!["ilvl", "stack", "reqlevel", "sockets", "links", "sockets.red", "price.amount", "price.lot"]).prop_map(Sort::Field),
        1 => proptest::sample::select(PSEUDO.to_vec()).prop_map(Sort::Field),
        2 => (group(), slot_word()).prop_map(|(g, s)| Sort::Proj(g, s)),
        2 => (group(), slot_word()).prop_map(|(g, s)| Sort::Sum(g, s)),
    ]
    .boxed()
}

/// How a tree is spelled: the first writes every and as the word at the
/// item's level and as whitespace in a group, the second the other way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spelling {
    Authored,
    Explicit,
}

impl Spelling {
    fn item_and(self) -> &'static str {
        match self {
            Spelling::Authored => " and ",
            Spelling::Explicit => " ",
        }
    }

    fn group_and(self) -> &'static str {
        match self {
            Spelling::Authored => " ",
            Spelling::Explicit => " AND ",
        }
    }
}

fn line_text(g: &G, spelling: Spelling) -> String {
    format!("line({})", g_text(g, true, spelling.group_and()))
}

pub fn sort_text(sort: &Sort, spelling: Spelling) -> Option<String> {
    match sort {
        Sort::None => None,
        Sort::Field(name) => Some(name.to_string()),
        Sort::Proj(g, slot) => Some(format!("{}.{slot}", line_text(g, spelling))),
        Sort::Sum(g, slot) => Some(format!("sum({}.{slot})", line_text(g, spelling))),
    }
}

pub fn q_text(q: &Q, spelling: Spelling) -> String {
    text(q, true, spelling)
}

fn text(q: &Q, top: bool, spelling: Spelling) -> String {
    let wrapped = |text: String| {
        if top { text } else { format!("({text})") }
    };
    let each = |children: &[Q], joiner: &str| {
        children
            .iter()
            .map(|c| text(c, false, spelling))
            .collect::<Vec<_>>()
            .join(joiner)
    };
    match q {
        Q::Plain(term) | Q::Probe(term) | Q::Total(term) => term.clone(),
        Q::Alt(spelled, explicit, _) => match spelling {
            Spelling::Authored => spelled.clone(),
            Spelling::Explicit => explicit.clone(),
        },
        Q::Line(g) => line_text(g, spelling),
        Q::Linked(g) => format!("linked({})", g_text(g, true, spelling.group_and())),
        Q::Sum(g, slot, cmp) => format!("sum({}.{slot}){cmp}", line_text(g, spelling)),
        Q::Proj(g, slot, cmp) => match spelling {
            Spelling::Authored => format!("{}.{slot}{cmp}", line_text(g, spelling)),
            // what the projection lowers to: the comparison inside the group
            Spelling::Explicit => line_text(
                &G::And(vec![g.clone(), G::Leaf(format!("{slot}{cmp}"))]),
                spelling,
            ),
        },
        Q::Open(g, slot, false) => format!("undecided({}.{slot})", line_text(g, spelling)),
        Q::Open(g, slot, true) => format!("undecided(sum({}.{slot}))", line_text(g, spelling)),
        Q::And(children) => wrapped(each(children, spelling.item_and())),
        Q::Or(children) => wrapped(each(children, " or ")),
        Q::Not(inner) => format!("-({})", text(inner, true, spelling)),
        Q::Holds(of, bound) => format!("holds({}){bound}", each(of, ", ")),
        Q::Undecided(inner) => format!("undecided({})", text(inner, true, spelling)),
    }
}

/// The tree with every rewrite that changes no meaning undone, at both
/// levels: nested ands and ors flattened, a doubled not cancelled.
pub fn normal(q: &Q) -> Q {
    match q {
        Q::Plain(_) | Q::Probe(_) | Q::Total(_) | Q::Alt(..) => q.clone(),
        Q::Line(g) => Q::Line(g_normal(g)),
        Q::Linked(g) => Q::Linked(g_normal(g)),
        Q::Sum(g, slot, cmp) => Q::Sum(g_normal(g), slot.clone(), cmp.clone()),
        Q::Proj(g, slot, cmp) => Q::Proj(g_normal(g), slot.clone(), cmp.clone()),
        Q::Open(g, slot, sum) => Q::Open(g_normal(g), slot.clone(), *sum),
        Q::And(children) => {
            let mut flat = Vec::new();
            for child in children.iter().map(normal) {
                match child {
                    Q::And(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            Q::And(flat)
        }
        Q::Or(children) => {
            let mut flat = Vec::new();
            for child in children.iter().map(normal) {
                match child {
                    Q::Or(inner) => flat.extend(inner),
                    other => flat.push(other),
                }
            }
            Q::Or(flat)
        }
        Q::Not(inner) => match normal(inner) {
            Q::Not(twice) => *twice,
            other => Q::Not(Box::new(other)),
        },
        Q::Holds(of, bound) => Q::Holds(of.iter().map(normal).collect(), bound.clone()),
        Q::Undecided(inner) => Q::Undecided(Box::new(normal(inner))),
    }
}

pub fn sort_normal(sort: &Sort) -> Sort {
    match sort {
        Sort::Proj(g, slot) => Sort::Proj(g_normal(g), slot.clone()),
        Sort::Sum(g, slot) => Sort::Sum(g_normal(g), slot.clone()),
        other => other.clone(),
    }
}

/// The members of every and, or and `holds` in another order, at both
/// levels (transformation 4).
pub fn shuffled(q: &Q, seed: u64) -> Q {
    fn walk(q: &Q, rng: &mut Lcg) -> Q {
        let each = |children: &[Q], rng: &mut Lcg| {
            let mut children: Vec<Q> = children.iter().map(|c| walk(c, rng)).collect();
            rng.shuffle(&mut children);
            children
        };
        match q {
            Q::Plain(_) | Q::Probe(_) | Q::Total(_) | Q::Alt(..) => q.clone(),
            Q::Line(g) => Q::Line(g_shuffled(g, rng)),
            Q::Linked(g) => Q::Linked(g_shuffled(g, rng)),
            Q::Sum(g, slot, cmp) => Q::Sum(g_shuffled(g, rng), slot.clone(), cmp.clone()),
            Q::Proj(g, slot, cmp) => Q::Proj(g_shuffled(g, rng), slot.clone(), cmp.clone()),
            Q::Open(g, slot, sum) => Q::Open(g_shuffled(g, rng), slot.clone(), *sum),
            Q::And(children) => Q::And(each(children, rng)),
            Q::Or(children) => Q::Or(each(children, rng)),
            Q::Not(inner) => Q::Not(Box::new(walk(inner, rng))),
            Q::Holds(of, bound) => Q::Holds(each(of, rng), bound.clone()),
            Q::Undecided(inner) => Q::Undecided(Box::new(walk(inner, rng))),
        }
    }
    walk(q, &mut Lcg(seed | 1))
}

pub fn sort_shuffled(sort: &Sort, seed: u64) -> Sort {
    let mut rng = Lcg(seed.rotate_left(17) | 1);
    match sort {
        Sort::Proj(g, slot) => Sort::Proj(g_shuffled(g, &mut rng), slot.clone()),
        Sort::Sum(g, slot) => Sort::Sum(g_shuffled(g, &mut rng), slot.clone()),
        other => other.clone(),
    }
}

/// The tree in one spelling whatever rewrite or order it was written in:
/// what two trees that mean the same have in common.
fn sorted(q: &Q) -> Q {
    let each = |children: &[Q]| {
        let mut children: Vec<Q> = children.iter().map(sorted).collect();
        children.sort_by_key(|c| text(c, false, Spelling::Explicit));
        children
    };
    match q {
        Q::Plain(_) | Q::Probe(_) | Q::Total(_) | Q::Alt(..) => q.clone(),
        Q::Line(g) => Q::Line(g_sorted(g)),
        Q::Linked(g) => Q::Linked(g_sorted(g)),
        Q::Sum(g, slot, cmp) => Q::Sum(g_sorted(g), slot.clone(), cmp.clone()),
        // a projection's comparison is one more conjunct of its group
        Q::Proj(g, slot, cmp) => Q::Line(g_sorted(&g_normal(&G::And(vec![
            g.clone(),
            G::Leaf(format!("{slot}{cmp}")),
        ])))),
        Q::Open(g, slot, sum) => Q::Open(g_sorted(g), slot.clone(), *sum),
        Q::And(children) => Q::And(each(children)),
        Q::Or(children) => Q::Or(each(children)),
        Q::Not(inner) => Q::Not(Box::new(sorted(inner))),
        Q::Holds(of, bound) => Q::Holds(each(of), bound.clone()),
        Q::Undecided(inner) => Q::Undecided(Box::new(sorted(inner))),
    }
}

fn key(q: &Q) -> String {
    text(&sorted(&normal(q)), true, Spelling::Explicit)
}

/// The atomic terms of a tree in the order an answer's terms block lists
/// them — what `undecided( … )` asks about before the probe itself — each
/// with a key that no rewrite and no reordering changes.
pub fn terms(q: &Q) -> Vec<(String, TermKind)> {
    fn walk(q: &Q, out: &mut Vec<(String, TermKind)>) {
        match q {
            Q::Plain(_) => out.push((key(q), TermKind::Plain)),
            Q::Alt(_, _, kind) => out.push((key(q), *kind)),
            Q::Line(_) | Q::Linked(_) | Q::Proj(..) => out.push((key(q), TermKind::Group)),
            Q::Sum(..) | Q::Total(_) => out.push((key(q), TermKind::Sum)),
            Q::Open(..) | Q::Probe(_) => out.push((key(q), TermKind::Probe)),
            Q::And(children) | Q::Or(children) | Q::Holds(children, _) => {
                children.iter().for_each(|c| walk(c, out));
            }
            Q::Not(inner) => walk(inner, out),
            Q::Undecided(inner) => {
                walk(inner, out);
                out.push((key(q), TermKind::Probe));
            }
        }
    }
    let mut out = Vec::new();
    walk(q, &mut out);
    out
}

/// How many cases a property runs: its gate number, or `PROPTEST_CASES`
/// for a longer run by hand.
pub fn cases(gate: u32) -> u32 {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|n| n.parse().ok())
        .unwrap_or(gate)
}

/// A generator's own small source of choices, seeded by proptest: a
/// shuffle needs no more, and a seed shrinks with its case.
pub struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = (self.next() % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }
}

// ---- bodies with holes ------------------------------------------------------------------------

/// How a completion fills the unread flags of a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coins {
    No,
    Yes,
    Tossed(u64),
}

/// Yes, no, or something the deriver cannot read as either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tri {
    Yes,
    No,
    Hole,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Flags {
    /// `flags` is no object: every flag of the line is unknown.
    Hole,
    Each {
        crafted: Tri,
        fractured: Tri,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineM {
    pub kind: u8,
    pub n: i32,
    /// The number is written with more decimals than the search reads: an
    /// unread slot, which a completion fills with a number. Never on the
    /// ranged line, whose second number is read and must stay as it is.
    pub unread_number: bool,
    pub flags: Flags,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Elem {
    Line(LineM),
    /// An element that is no line: the array may hold one more, or not.
    Junk,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lines {
    Absent,
    /// The array is no array.
    Hole,
    Of(Vec<Elem>),
}

/// The properties a derived field reads (`pseudo.rs`, C101), and one it
/// does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropName {
    Aps,
    Physical,
    Elemental,
    Chaos,
    Quality,
}

impl PropName {
    const ALL: [PropName; 5] = [
        PropName::Aps,
        PropName::Physical,
        PropName::Elemental,
        PropName::Chaos,
        PropName::Quality,
    ];

    pub fn shown(self) -> &'static str {
        match self {
            PropName::Aps => "Attacks per Second",
            PropName::Physical => "Physical Damage",
            PropName::Elemental => "Elemental Damage",
            PropName::Chaos => "Chaos Damage",
            PropName::Quality => "Quality",
        }
    }

    /// The values a completion may give the property: the ends of a
    /// small range and what the dice pick between them.
    fn readable(self) -> &'static [&'static str] {
        match self {
            PropName::Aps => &["0.5", "1", "1.25", "1.4", "2"],
            PropName::Physical => &["0-0", "1", "5-9", "10-17", "12-19"],
            PropName::Elemental => &["0-0", "2-4", "5-9", "10-17", "12-19"],
            PropName::Chaos => &["0-0", "1-3", "3", "8-11", "12-19"],
            PropName::Quality => &["+0%", "+20%"],
        }
    }
}

/// One element of the `properties` array.
#[derive(Debug, Clone, PartialEq)]
pub enum Prop {
    /// A property as displayed: its name and its values, one or several.
    Read(PropName, Vec<String>),
    /// Its values are no array: unread, the name kept (`Unread::name`).
    Malformed(PropName),
    /// A value the fields cannot read as a number: `fast`, `1e34`,
    /// nothing. The element reads; the field is undecided by it.
    Unreadable(PropName, &'static str),
    /// An element that is no property: may be any property.
    Nameless,
}

impl Prop {
    fn name(&self) -> Option<PropName> {
        match self {
            Prop::Read(name, _) | Prop::Malformed(name) | Prop::Unreadable(name, _) => Some(*name),
            Prop::Nameless => None,
        }
    }

    fn has_hole(&self) -> bool {
        !matches!(self, Prop::Read(..))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Props {
    Absent,
    /// The array is no array.
    Hole,
    Of(Vec<Prop>),
}

/// One element of `sockets` (C101).
#[derive(Debug, Clone, PartialEq)]
pub enum Sock {
    /// A socket as GGG gives it: its colour letter and its group.
    Of(&'static str, u8),
    /// A socket whose colour could not be read: an `attr` and no `sColour`.
    Blind(u8),
    /// A socket whose group could not be read.
    Loose(&'static str),
    /// An element that is no socket: may be one, or nothing.
    Junk,
}

impl Sock {
    fn has_hole(&self) -> bool {
        !matches!(self, Sock::Of(..))
    }
}

/// The colours a generated socket takes: the four words' letters and an
/// abyssal socket's, which no word names.
const LETTERS: [&str; 5] = ["R", "G", "B", "W", "A"];

#[derive(Debug, Clone, PartialEq)]
pub enum Socks {
    Absent,
    /// The array is no array.
    Hole,
    Of(Vec<Sock>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Known<T> {
    Absent,
    Is(T),
    Hole,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub explicit: Lines,
    pub implicit: Lines,
    pub hybrid: Lines,
    pub corrupted: Tri,
    /// `influences` is no object: both of its flags are unknown.
    pub influences_hole: bool,
    pub shaper: Tri,
    pub hunter: Tri,
    pub ilvl: Known<i64>,
    /// The `Level` requirement; its hole is a `Level` row with no value
    /// (the step-6 review, 1).
    pub reqlevel: Known<i64>,
    pub name: Known<&'static str>,
    pub note: Known<&'static str>,
    /// The `properties` array the derived fields read.
    pub props: Props,
    /// The socket collection.
    pub socks: Socks,
}

fn tri(holes: bool) -> BoxedStrategy<Tri> {
    let hole = if holes { 2 } else { 0 };
    prop_oneof![3 => Just(Tri::No), 2 => Just(Tri::Yes), hole => Just(Tri::Hole)].boxed()
}

pub fn line(holes: bool) -> BoxedStrategy<LineM> {
    let hole = if holes { 1 } else { 0 };
    let flags = prop_oneof![
        5 => (tri(holes), tri(holes)).prop_map(|(crafted, fractured)| Flags::Each { crafted, fractured }),
        hole => Just(Flags::Hole),
    ];
    (
        0u8..9,
        LOW..=HIGH,
        proptest::bool::weighted(if holes { 0.15 } else { 0.0 }),
        flags,
    )
        .prop_map(|(kind, n, unread, flags)| LineM {
            kind,
            n,
            unread_number: unread && kind != 2 && kind != 3,
            flags,
        })
        .boxed()
}

/// A readable property of the name: one value, or two for the elemental
/// damage, which the game may display as several ranges.
fn prop_read(name: PropName) -> BoxedStrategy<Prop> {
    let values = proptest::sample::select(name.readable().to_vec()).prop_map(str::to_string);
    let several = if name == PropName::Elemental { 1 } else { 0 };
    prop_oneof![
        3 => values.clone().prop_map(|v| vec![v]),
        several => (values.clone(), values).prop_map(|(a, b)| vec![a, b]),
    ]
    .prop_map(move |values| Prop::Read(name, values))
    .boxed()
}

fn prop(name: PropName, holes: bool) -> BoxedStrategy<Prop> {
    let hole = if holes { 1 } else { 0 };
    prop_oneof![
        6 => prop_read(name),
        hole => Just(Prop::Malformed(name)),
        hole => proptest::sample::select(vec!["fast", "1e34", ""])
            .prop_map(move |v| Prop::Unreadable(name, v)),
    ]
    .boxed()
}

fn sock(holes: bool) -> BoxedStrategy<Sock> {
    let hole = if holes { 1 } else { 0 };
    let letter = proptest::sample::select(LETTERS.to_vec());
    prop_oneof![
        8 => (letter.clone(), 0u8..3).prop_map(|(c, g)| Sock::Of(c, g)),
        hole => (0u8..3).prop_map(Sock::Blind),
        hole => letter.prop_map(Sock::Loose),
        hole => Just(Sock::Junk),
    ]
    .boxed()
}

/// The socket collection: none, GGG's `[]`, up to six sockets in up to
/// three groups; with `holes`, a socket whose colour or group cannot be
/// read, an element that is no socket, or no array at all.
fn socks(holes: bool) -> BoxedStrategy<Socks> {
    let hole = if holes { 1 } else { 0 };
    prop_oneof![
        3 => Just(Socks::Absent),
        hole => Just(Socks::Hole),
        5 => proptest::collection::vec(sock(holes), 0..7).prop_map(Socks::Of),
    ]
    .boxed()
}

/// The `properties` array: some of the five names, each once, in any
/// order; with `holes`, an element that is no property among them, or
/// no array at all.
fn props(holes: bool) -> BoxedStrategy<Props> {
    let hole = if holes { 1 } else { 0 };
    let elems = proptest::sample::subsequence(PropName::ALL.to_vec(), 0..=5)
        .prop_flat_map(move |names| {
            names
                .into_iter()
                .map(|n| prop(n, holes))
                .collect::<Vec<_>>()
        })
        .prop_shuffle();
    let nameless = if holes {
        proptest::option::weighted(0.15, 0usize..6).boxed()
    } else {
        Just(None).boxed()
    };
    prop_oneof![
        3 => Just(Props::Absent),
        hole => Just(Props::Hole),
        5 => (elems, nameless).prop_map(|(mut elems, nameless)| {
            if let Some(at) = nameless {
                elems.insert(at.min(elems.len()), Prop::Nameless);
            }
            Props::Of(elems)
        }),
    ]
    .boxed()
}

fn lines(holes: bool) -> BoxedStrategy<Lines> {
    let hole = if holes { 1 } else { 0 };
    let elem = prop_oneof![
        8 => line(holes).prop_map(Elem::Line),
        hole => Just(Elem::Junk),
    ];
    prop_oneof![
        1 => Just(Lines::Absent),
        hole => Just(Lines::Hole),
        5 => proptest::collection::vec(elem, 0..4).prop_map(Lines::Of),
    ]
    .boxed()
}

fn known<T: Clone + std::fmt::Debug + 'static>(
    values: Vec<T>,
    holes: bool,
) -> BoxedStrategy<Known<T>> {
    let hole = if holes { 1 } else { 0 };
    prop_oneof![
        2 => Just(Known::Absent),
        4 => proptest::sample::select(values).prop_map(Known::Is),
        hole => Just(Known::Hole),
    ]
    .boxed()
}

/// An item's body. With `holes` it may carry what cannot be read; without,
/// every part of it is readable, which is what a completion is made from.
pub fn body(holes: bool) -> BoxedStrategy<Body> {
    if holes {
        return prop_oneof![1 => some_body(false), 2 => some_body(true)].boxed();
    }
    some_body(false)
}

fn some_body(holes: bool) -> BoxedStrategy<Body> {
    let hybrid = prop_oneof![4 => Just(Lines::Absent), 1 => lines(holes)];
    (
        (lines(holes), lines(holes), hybrid),
        (
            tri(holes),
            proptest::bool::weighted(if holes { 0.1 } else { 0.0 }),
            tri(holes),
            tri(holes),
        ),
        known(vec![0, 1, 79, 80, 84], holes),
        known(vec![1, 20, 30, 45], holes),
        known(vec!["Doom Knot", "Life Ring"], holes),
        known(
            vec![
                "~price 1 chaos",
                "~price 5 chaos",
                "~b/o 2/3 divine",
                "~skip",
                "keep",
            ],
            holes,
        ),
        props(holes),
        socks(holes),
    )
        .prop_map(
            |(
                (explicit, implicit, hybrid),
                (corrupted, influences_hole, shaper, hunter),
                ilvl,
                reqlevel,
                name,
                note,
                props,
                socks,
            )| Body {
                explicit,
                implicit,
                hybrid,
                corrupted,
                influences_hole,
                shaper,
                hunter,
                ilvl,
                reqlevel,
                name,
                note,
                props,
                socks,
            },
        )
        .boxed()
}

/// A body with something unread in it, whatever the dice gave.
pub fn holed() -> BoxedStrategy<Body> {
    (some_body(true), 0u8..7)
        .prop_map(|(mut body, which)| {
            if !body.has_holes() {
                match which {
                    0 => body.implicit = Lines::Hole,
                    1 => body.explicit = Lines::Hole,
                    2 => body.corrupted = Tri::Hole,
                    3 => body.reqlevel = Known::Hole,
                    4 => body.props = Props::Hole,
                    5 => body.socks = Socks::Of(vec![Sock::Of("R", 0), Sock::Blind(0), Sock::Junk]),
                    _ => body.ilvl = Known::Hole,
                }
            }
            body
        })
        .boxed()
}

impl Body {
    /// A body that says nothing, every yes or no of it answered `flag`:
    /// what the two plainest completions are filled from.
    pub fn blank(flag: Tri) -> Body {
        Body {
            explicit: Lines::Absent,
            implicit: Lines::Absent,
            hybrid: Lines::Absent,
            corrupted: flag,
            influences_hole: false,
            shaper: flag,
            hunter: flag,
            ilvl: Known::Absent,
            reqlevel: Known::Absent,
            name: Known::Absent,
            note: Known::Absent,
            props: Props::Absent,
            socks: Socks::Absent,
        }
    }

    /// A readable body whose arrays hold `lines`, and whose numbers, names
    /// and yes-or-noes are the `n`th of a fixed round — so that a handful
    /// of completions tries an unread number as several, a name as two and
    /// as none, and three flags every way, whatever the dice gave — and
    /// whose properties are none, a whole weapon's, an attack speed with
    /// no damage, or a damage with no attack speed.
    pub fn nth(n: usize, lines: Vec<LineM>) -> Body {
        let read = |name: PropName, values: &[&str]| {
            Prop::Read(name, values.iter().map(|v| v.to_string()).collect())
        };
        let flag = |bit: usize| if n >> bit & 1 == 1 { Tri::Yes } else { Tri::No };
        let of = |lines: &[LineM]| Lines::Of(lines.iter().cloned().map(Elem::Line).collect());
        let half = lines.len().div_ceil(2);
        Body {
            explicit: of(&lines[..half]),
            implicit: of(&lines[half..]),
            hybrid: if n.is_multiple_of(3) {
                of(&lines)
            } else {
                Lines::Absent
            },
            corrupted: flag(0),
            influences_hole: false,
            shaper: flag(1),
            hunter: flag(2),
            ilvl: [
                Known::Absent,
                Known::Is(0),
                Known::Is(1),
                Known::Is(79),
                Known::Is(80),
                Known::Is(84),
            ][n % 6]
                .clone(),
            reqlevel: [
                Known::Absent,
                Known::Is(1),
                Known::Is(20),
                Known::Is(30),
                Known::Is(45),
            ][n % 5]
                .clone(),
            name: [
                Known::Absent,
                Known::Is("Doom Knot"),
                Known::Is("Life Ring"),
            ][n % 3]
                .clone(),
            note: [
                Known::Is("keep"),
                Known::Absent,
                Known::Is("~price 1 chaos"),
            ][n / 3 % 3]
                .clone(),
            props: [
                Props::Absent,
                Props::Of(vec![
                    read(PropName::Aps, &["1.25"]),
                    read(PropName::Physical, &["59-88"]),
                    read(PropName::Elemental, &["38-71", "57-108"]),
                    read(PropName::Chaos, &["10-20"]),
                ]),
                Props::Of(vec![
                    read(PropName::Quality, &["+20%"]),
                    read(PropName::Aps, &["2"]),
                ]),
                Props::Of(vec![read(PropName::Physical, &["10-20"])]),
            ][n % 4]
                .clone(),
            // none, one group of three, two groups, GGG's `[]`, six white
            socks: [
                Socks::Absent,
                Socks::Of(vec![Sock::Of("R", 0), Sock::Of("R", 0), Sock::Of("G", 0)]),
                Socks::Of(vec![Sock::Of("R", 0), Sock::Of("B", 1)]),
                Socks::Of(Vec::new()),
                Socks::Of(vec![Sock::Of("W", 0); 6]),
            ][n % 5]
                .clone(),
        }
    }

    /// One line of every kind at each of four values, its flags as given.
    pub fn every_line(flag: Tri) -> Vec<LineM> {
        (0u8..9)
            .flat_map(|kind| {
                [LOW, 0, 1, HIGH].map(|n| LineM {
                    kind,
                    n,
                    unread_number: false,
                    flags: Flags::Each {
                        crafted: flag,
                        fractured: flag,
                    },
                })
            })
            .collect()
    }

    /// The body with the elements of every array in the opposite order
    /// (transformation 4), the properties too.
    pub fn every_line_reversed(&self) -> Body {
        let reversed = |lines: &Lines| match lines {
            Lines::Of(elems) => Lines::Of(elems.iter().rev().cloned().collect()),
            other => other.clone(),
        };
        Body {
            explicit: reversed(&self.explicit),
            implicit: reversed(&self.implicit),
            hybrid: reversed(&self.hybrid),
            props: match &self.props {
                Props::Of(elems) => Props::Of(elems.iter().rev().cloned().collect()),
                other => other.clone(),
            },
            socks: match &self.socks {
                Socks::Of(elems) => Socks::Of(elems.iter().rev().cloned().collect()),
                other => other.clone(),
            },
            ..self.clone()
        }
    }

    /// The body with every element of every line array written twice
    /// (transformation 10); the properties, which no line is, as they are.
    pub fn every_line_twice(&self) -> Body {
        let twice = |lines: &Lines| match lines {
            Lines::Of(elems) => {
                Lines::Of(elems.iter().flat_map(|e| [e.clone(), e.clone()]).collect())
            }
            other => other.clone(),
        };
        Body {
            explicit: twice(&self.explicit),
            implicit: twice(&self.implicit),
            hybrid: twice(&self.hybrid),
            ..self.clone()
        }
    }

    /// Whether anything of it cannot be read.
    pub fn has_holes(&self) -> bool {
        let tri = |t: Tri| t == Tri::Hole;
        let of = |lines: &Lines| match lines {
            Lines::Absent => false,
            Lines::Hole => true,
            Lines::Of(elems) => elems.iter().any(|e| match e {
                Elem::Junk => true,
                Elem::Line(l) => {
                    l.unread_number
                        || match &l.flags {
                            Flags::Hole => true,
                            Flags::Each { crafted, fractured } => tri(*crafted) || tri(*fractured),
                        }
                }
            }),
        };
        of(&self.explicit)
            || of(&self.implicit)
            || of(&self.hybrid)
            || tri(self.corrupted)
            || self.influences_hole
            || tri(self.shaper)
            || tri(self.hunter)
            || self.ilvl == Known::Hole
            || self.reqlevel == Known::Hole
            || self.name == Known::Hole
            || self.note == Known::Hole
            || match &self.props {
                Props::Absent => false,
                Props::Hole => true,
                Props::Of(elems) => elems.iter().any(Prop::has_hole),
            }
            || match &self.socks {
                Socks::Absent => false,
                Socks::Hole => true,
                Socks::Of(elems) => elems.iter().any(Sock::has_hole),
            }
    }

    /// The body with every hole filled from `fill`, which has none: an
    /// unread array is the fill's array, an unread flag of a line a coin,
    /// an element that was no line a line or nothing. What could be read
    /// is kept.
    pub fn completed(&self, fill: &Body, coins: Coins) -> Body {
        let rng = Lcg(match coins {
            Coins::Tossed(seed) => seed | 1,
            _ => 1,
        });
        let rng = std::cell::RefCell::new(rng);
        let coin = || match coins {
            Coins::No => false,
            Coins::Yes => true,
            Coins::Tossed(_) => rng.borrow_mut().next().is_multiple_of(2),
        };
        let pick = |of: usize| (rng.borrow_mut().next() as usize) % of.max(1);
        let filled = |t: Tri, from: Tri| match (t, from) {
            (Tri::Hole, Tri::Hole) => Tri::No,
            (Tri::Hole, from) => from,
            (known, _) => known,
        };
        let mut spare: Vec<LineM> = [&fill.explicit, &fill.implicit, &fill.hybrid]
            .into_iter()
            .flat_map(|lines| match lines {
                Lines::Of(elems) => elems.clone(),
                _ => Vec::new(),
            })
            .filter_map(|e| match e {
                Elem::Line(l) => Some(l),
                Elem::Junk => None,
            })
            .collect();
        let mut of = |lines: &Lines, from: &Lines| match lines {
            Lines::Absent => Lines::Absent,
            Lines::Hole => from.clone(),
            Lines::Of(elems) => {
                let mut out = Vec::new();
                for elem in elems {
                    match elem {
                        Elem::Junk => {
                            let at = pick(spare.len());
                            if coin() && !spare.is_empty() {
                                out.push(Elem::Line(spare.remove(at)));
                            }
                        }
                        Elem::Line(l) => {
                            let toss = |t: Tri| match t {
                                Tri::Hole if coin() => Tri::Yes,
                                Tri::Hole => Tri::No,
                                known => known,
                            };
                            let (crafted, fractured) = match &l.flags {
                                Flags::Hole => (toss(Tri::Hole), toss(Tri::Hole)),
                                Flags::Each { crafted, fractured } => {
                                    (toss(*crafted), toss(*fractured))
                                }
                            };
                            // an unread number may be any number: the
                            // ends of the range, or one the dice pick
                            let n = match (l.unread_number, coins) {
                                (false, _) => l.n,
                                (true, Coins::No) => LOW,
                                (true, Coins::Yes) => HIGH,
                                (true, Coins::Tossed(_)) => {
                                    LOW + pick((HIGH - LOW + 1) as usize) as i32
                                }
                            };
                            out.push(Elem::Line(LineM {
                                n,
                                unread_number: false,
                                flags: Flags::Each { crafted, fractured },
                                ..l.clone()
                            }));
                        }
                    }
                }
                Lines::Of(out)
            }
        };
        let explicit = of(&self.explicit, &fill.explicit);
        let implicit = of(&self.implicit, &fill.implicit);
        let hybrid = of(&self.hybrid, &fill.hybrid);
        let known = |k: &Known<&'static str>, from: &Known<&'static str>| match (k, from) {
            (Known::Hole, Known::Hole) => Known::Absent,
            (Known::Hole, from) => from.clone(),
            (other, _) => other.clone(),
        };
        let influence =
            |t: Tri, from: Tri| filled(if self.influences_hole { Tri::Hole } else { t }, from);
        // a property whose values could not be read, or read as no
        // number, may be any value of its name: the ends of the range, or
        // one the dice pick; an element that is no property may be a
        // property of a name the array lacks, or nothing
        let readable = |name: PropName| {
            let of = name.readable();
            let value = match coins {
                Coins::No => of[0],
                Coins::Yes => of[of.len() - 1],
                Coins::Tossed(_) => of[pick(of.len())],
            };
            Prop::Read(name, vec![value.to_string()])
        };
        let props = match &self.props {
            Props::Absent => Props::Absent,
            Props::Hole => fill.props.clone(),
            Props::Of(elems) => {
                let mut spare: Vec<Prop> = match &fill.props {
                    Props::Of(from) => from
                        .iter()
                        .filter(|p| !elems.iter().any(|e| e.name() == p.name()))
                        .cloned()
                        .collect(),
                    _ => Vec::new(),
                };
                let mut out = Vec::new();
                for elem in elems {
                    match elem {
                        Prop::Read(..) => out.push(elem.clone()),
                        Prop::Malformed(name) | Prop::Unreadable(name, _) => {
                            out.push(readable(*name));
                        }
                        Prop::Nameless => {
                            let at = pick(spare.len());
                            if coin() && !spare.is_empty() {
                                out.push(spare.remove(at));
                            }
                        }
                    }
                }
                Props::Of(out)
            }
        };
        // a socket whose colour could not be read may be any colour, one
        // whose group could not be read may sit in any group, and an
        // element that is no socket may be one of any colour in any
        // group, or nothing
        let letter = || match coins {
            Coins::No => LETTERS[0],
            Coins::Yes => LETTERS[LETTERS.len() - 1],
            Coins::Tossed(_) => LETTERS[pick(LETTERS.len())],
        };
        let group = || match coins {
            Coins::No => 0u8,
            Coins::Yes => 3,
            Coins::Tossed(_) => pick(4) as u8,
        };
        let socks = match &self.socks {
            Socks::Absent => Socks::Absent,
            Socks::Hole => fill.socks.clone(),
            Socks::Of(elems) => Socks::Of(
                elems
                    .iter()
                    .filter_map(|sock| match sock {
                        Sock::Of(..) => Some(sock.clone()),
                        Sock::Blind(g) => Some(Sock::Of(letter(), *g)),
                        Sock::Loose(c) => Some(Sock::Of(c, group())),
                        Sock::Junk => coin().then(|| Sock::Of(letter(), group())),
                    })
                    .collect(),
            ),
        };
        Body {
            explicit,
            implicit,
            hybrid,
            props,
            socks,
            corrupted: filled(self.corrupted, fill.corrupted),
            influences_hole: false,
            shaper: influence(self.shaper, fill.shaper),
            hunter: influence(self.hunter, fill.hunter),
            ilvl: match (&self.ilvl, &fill.ilvl) {
                (Known::Hole, Known::Hole) => Known::Absent,
                (Known::Hole, from) => from.clone(),
                (other, _) => other.clone(),
            },
            reqlevel: match (&self.reqlevel, &fill.reqlevel) {
                (Known::Hole, Known::Hole) => Known::Absent,
                (Known::Hole, from) => from.clone(),
                (other, _) => other.clone(),
            },
            name: known(&self.name, &fill.name),
            note: known(&self.note, &fill.note),
        }
    }

    /// The body as GGG would give it, a hole being a value of a type the
    /// deriver does not read there.
    pub fn json(&self) -> Value {
        const UNREAD: &str = "unread";
        let tri = |t: Tri| match t {
            Tri::Yes => json!(true),
            Tri::No => json!(false),
            Tri::Hole => json!(UNREAD),
        };
        let line = |l: &LineM| {
            // five decimals: one more than the search reads
            let description = match l.kind {
                0 | 1 | 4 | 6 | 7 | 8 if l.unread_number => {
                    let of = [
                        "to maximum Life",
                        "% to Cold Resistance",
                        "",
                        "",
                        "to Spirit",
                        "",
                        "to maximum life",
                        "% to Fire Resistance",
                        "% to all Elemental Resistances",
                    ][usize::from(l.kind)];
                    let gap = if of.starts_with('%') { "" } else { " " };
                    format!("{:+}.12345{gap}{of}", l.n)
                }
                5 if l.unread_number => format!("{}2345% of Damage Leeched as Life", tenths(l.n)),
                0 => format!("{:+} to maximum Life", l.n),
                1 => format!("{:+}% to Cold Resistance", l.n),
                2 => format!("Adds {} to {} Cold Damage", l.n.abs(), l.n.abs() + 7),
                3 => FROZEN.to_string(),
                4 => format!("{:+} to Spirit", l.n),
                5 => format!("{}% of Damage Leeched as Life", tenths(l.n)),
                // GGG has spelled some lines two ways
                6 => format!("{:+} to maximum life", l.n),
                7 => format!("{:+}% to Fire Resistance", l.n),
                _ => format!("{:+}% to all Elemental Resistances", l.n),
            };
            let flags = match &l.flags {
                Flags::Hole => json!(UNREAD),
                Flags::Each { crafted, fractured } => {
                    json!({ "crafted": tri(*crafted), "fractured": tri(*fractured) })
                }
            };
            json!({ "description": description, "flags": flags })
        };
        let of = |lines: &Lines| match lines {
            Lines::Absent => None,
            Lines::Hole => Some(json!(UNREAD)),
            Lines::Of(elems) => Some(Value::Array(
                elems
                    .iter()
                    .map(|e| match e {
                        Elem::Line(l) => line(l),
                        Elem::Junk => json!(5),
                    })
                    .collect(),
            )),
        };
        let mut body = Map::new();
        if let Some(lines) = of(&self.explicit) {
            body.insert("explicitMods".into(), lines);
        }
        if let Some(lines) = of(&self.implicit) {
            body.insert("implicitMods".into(), lines);
        }
        match of(&self.hybrid) {
            None => {}
            Some(Value::Array(lines)) => {
                body.insert("hybrid".into(), json!({ "explicitMods": lines }));
            }
            Some(hole) => {
                body.insert("hybrid".into(), hole);
            }
        }
        body.insert("corrupted".into(), tri(self.corrupted));
        body.insert(
            "influences".into(),
            if self.influences_hole {
                json!(UNREAD)
            } else {
                json!({ "shaper": tri(self.shaper), "hunter": tri(self.hunter) })
            },
        );
        body.insert(
            "ilvl".into(),
            match &self.ilvl {
                Known::Absent => Value::Null,
                Known::Is(n) => json!(n),
                Known::Hole => json!(UNREAD),
            },
        );
        // a Level requirement, or one whose value could not be read
        let level =
            |values: Value| json!([{ "name": "Level", "values": values, "displayMode": 0 }]);
        match &self.reqlevel {
            Known::Absent => {}
            Known::Is(n) => {
                body.insert("requirements".into(), level(json!([[n.to_string(), 0]])));
            }
            Known::Hole => {
                body.insert("requirements".into(), level(json!([])));
            }
        }
        body.insert(
            "name".into(),
            match &self.name {
                Known::Absent => json!(""),
                Known::Is(name) => json!(name),
                Known::Hole => json!(5),
            },
        );
        match &self.note {
            Known::Absent => {}
            Known::Is(note) => {
                body.insert("note".into(), json!(note));
            }
            Known::Hole => {
                body.insert("note".into(), json!(5));
            }
        }
        // the properties as GGG displays them: `[text, style]` pairs
        let prop = |p: &Prop| match p {
            Prop::Read(name, values) => json!({
                "name": name.shown(),
                "values": values.iter().map(|v| json!([v, 0])).collect::<Vec<_>>(),
                "displayMode": 0,
            }),
            Prop::Malformed(name) => json!({ "name": name.shown(), "values": 7, "displayMode": 0 }),
            Prop::Unreadable(name, value) => {
                json!({ "name": name.shown(), "values": [[value, 0]], "displayMode": 0 })
            }
            Prop::Nameless => json!(7),
        };
        match &self.props {
            Props::Absent => {}
            Props::Hole => {
                body.insert("properties".into(), json!(UNREAD));
            }
            Props::Of(elems) => {
                body.insert(
                    "properties".into(),
                    Value::Array(elems.iter().map(prop).collect()),
                );
            }
        }
        // the sockets as GGG gives them: `attr` beside `sColour`
        fn attr(c: &str) -> &str {
            match c {
                "R" => "S",
                "G" => "D",
                "B" => "I",
                "W" => "G",
                other => other,
            }
        }
        let sock = |s: &Sock| match s {
            Sock::Of(c, g) => json!({ "group": g, "attr": attr(c), "sColour": c }),
            Sock::Blind(g) => json!({ "group": g, "attr": "S" }),
            Sock::Loose(c) => json!({ "sColour": c }),
            Sock::Junk => json!(7),
        };
        match &self.socks {
            Socks::Absent => {}
            Socks::Hole => {
                body.insert("sockets".into(), json!(UNREAD));
            }
            Socks::Of(elems) => {
                body.insert(
                    "sockets".into(),
                    Value::Array(elems.iter().map(sock).collect()),
                );
            }
        }
        Value::Object(body)
    }
}

/// Four lines of one template whose flags cannot be read.
fn flagless(template: &str, from: i32) -> Vec<Value> {
    (from..from + 4)
        .map(|n| json!({ "description": format!("+{n}{}{template}", if template.starts_with('%') { "" } else { " " }), "flags": "unread" }))
        .collect()
}

/// A property whose value no field reads as a number.
fn fast(name: &str) -> Value {
    json!({ "name": name, "values": [["fast", 0]], "displayMode": 0 })
}

/// The six items the audits' reproductions were made of, a seventh that
/// is past every bound, and an eighth past the bound on the reasons a
/// computed value makes beyond the item's own parts.
pub fn anchors() -> Vec<Value> {
    vec![
        json!({"explicitMods": ["+95 to maximum Life"]}),
        json!({"explicitMods": ["+20 to maximum Life", "+75 to maximum Life"]}),
        json!({}),
        json!({"implicitMods": "unread"}),
        json!({"explicitMods": ["+20 to maximum Life",
            {"description": "+95 to maximum Life", "flags": "unread"}]}),
        json!({"explicitMods": [{"description": "Cannot be Frozen", "flags": "unread"}],
            "hybrid": "unread"}),
        // past every bound of a row: more unread parts than are shown, in
        // two arrays and on the item, so that a block is cut and the cut
        // is asked of every spelling and every order
        json!({
            "explicitMods": flagless("to maximum Life", 1),
            "implicitMods": flagless("% to Cold Resistance", 5),
            "corrupted": "unread", "split": "unread", "duplicated": "unread",
            "replica": "unread", "mutated": "unread",
            "influences": {"shaper": "unread", "hunter": "unread"}}),
        // past the bound on reasons: three line arrays unread, which a
        // total rests on, and four properties the derived fields read
        // that are no number — seven parts, four of them made beyond the
        // item's own (step 7's outside audit, 5)
        json!({
            "implicitMods": "unread", "explicitMods": "unread", "craftedMods": "unread",
            "properties": [
                fast("Attacks per Second"), fast("Physical Damage"),
                fast("Elemental Damage"), fast("Chaos Damage")]}),
        // the sockets with every hole the deriver names (step 8): a red
        // socket read whole, one whose colour is unread, one whose group
        // is, and an element that is no socket
        json!({"explicitMods": ["+50 to maximum Life"], "sockets": [
            {"group": 0, "attr": "S", "sColour": "R"}, {"group": 0, "attr": "D"},
            {"sColour": "B"}, 7]}),
    ]
}

// ---- the boundary -----------------------------------------------------------------------------------

/// One public tab of rare rings `i0`, `i1`, … in the realm `pc`, the
/// owner's row on the tab (negotiable 1 divine) beneath every note, and an
/// item in another realm that no answer over `pc` may return.
pub fn fixture(bodies: Vec<Value>) -> (Corpus, Ids) {
    let (_, corpus, ids) = fixture_with_store(bodies);
    (corpus, ids)
}

/// The intent file the fixture is loaded beside: the owner's one row.
pub fn fixture_intent() -> Annotations {
    let mut intent = Annotations::open_memory_for("generated").unwrap();
    set_buyout(
        &mut intent,
        &PriceTarget::from_address("tab", "pc/t").unwrap(),
        &Buyout::Negotiable(Price {
            amount: "1".parse().unwrap(),
            currency: "divine".into(),
        }),
        None,
        &Provenance::via("test"),
    )
    .unwrap();
    intent
}

/// The same, with the store it was loaded from: what `show` reads.
pub fn fixture_with_store(bodies: Vec<Value>) -> (Store, Corpus, Ids) {
    let mut store = Store::open_memory().unwrap();
    store
        .record(
            &Endpoint::Profile,
            &json!({}),
            200,
            &json!({"uuid": "generated", "name": "Generated"}),
            1,
        )
        .unwrap();
    super::list_tabs(
        &mut store,
        "pc",
        "Standard",
        json!([{ "id": "t", "name": "Routes", "type": "PremiumStash", "metadata": { "public": true } }]),
        2,
    );
    let ids: Ids = (0..bodies.len()).map(|n| format!("i{n}")).collect();
    let items = bodies
        .into_iter()
        .enumerate()
        .map(|(n, body)| super::item(&format!("i{n}"), "", "Iron Ring", "Rare", body))
        .collect();
    super::fetch_tab(&mut store, "pc", "Standard", "t", "Routes", items, 3);
    super::list_tabs(
        &mut store,
        "poe2",
        "Standard",
        json!([super::tab("other", "Other")]),
        4,
    );
    super::fetch_tab(
        &mut store,
        "poe2",
        "Standard",
        "other",
        "Other",
        vec![super::item("outside", "", "Iron Ring", "Rare", json!({}))],
        5,
    );
    let corpus = super::load_with(&store, &fixture_intent(), Some("pc"));
    (store, corpus, ids)
}

/// A request in, an answer out, as JSON; an error as the JSON `--json`
/// prints, without the span, which is a place in one spelling's text.
pub fn run(corpus: &Corpus, request: &Value) -> Result<Value, Value> {
    let request: Request = serde_json::from_value(request.clone())
        .map_err(|e| json!({ "kind": "request", "error": e.to_string() }))?;
    match answer(corpus, &request) {
        Ok(answer) => Ok(serde_json::to_value(answer).unwrap()),
        Err(e) => {
            let mut error = e.to_json();
            if let Some(fields) = error.as_object_mut() {
                fields.remove("span");
            }
            Err(error)
        }
    }
}

pub fn request(query: &str, sort: Option<&str>, desc: bool, limit: usize) -> Value {
    let mut rows = json!({ "limit": limit, "desc": desc });
    if let Some(sort) = sort {
        rows["sort"] = json!(sort);
    }
    json!({ "scope": { "realm": "pc" }, "query": { "text": query }, "view": { "rows": rows } })
}

/// The ids a count's route returns, every row of it; none for a zero.
pub fn members(
    corpus: &Corpus,
    counted: &Value,
    scope: &Ids,
    cache: &mut HashMap<String, Ids>,
) -> Result<Ids, String> {
    let Some(route) = counted.get("request") else {
        return if counted["count"] == 0 {
            Ok(Ids::new())
        } else {
            Err(format!("a count with no route: {counted}"))
        };
    };
    let mut route = route.clone();
    route["view"]["rows"]["limit"] = json!(scope.len().max(1));
    let key = route.to_string();
    if let Some(ids) = cache.get(&key) {
        return Ok(ids.clone());
    }
    let routed = run(corpus, &route).map_err(|e| format!("route refused: {route}: {e}"))?;
    let ids: Ids = routed["rows"]
        .as_array()
        .ok_or("a route's answer has no rows")?
        .iter()
        .filter_map(|row| row["id"].as_str().map(str::to_string))
        .collect();
    if ids.len() as u64 != counted["count"].as_u64().unwrap_or(u64::MAX) {
        return Err(format!(
            "a route returned {} of the {} it counted: {route}",
            ids.len(),
            counted["count"]
        ));
    }
    cache.insert(key, ids.clone());
    Ok(ids)
}
