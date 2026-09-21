//! Generators for step 4b's properties (the build plan, "Step 4b"):
//! queries as a tree of this file's own, so that a rewrite is made on the
//! tree and only text crosses the crate's boundary, and item bodies with
//! holes — something the deriver cannot read — that a completion fills.
//!
//! Nothing here reads the crate's tree, binder or evaluator: a query is
//! rendered to text and asked, and a body enters through `Store::record`.

use std::collections::{BTreeSet, HashMap};

use acquisition_search::{Corpus, Request, answer};
use acquisition_store::{Endpoint, Store};
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
        2 => proptest::sample::select(vec![LIFE, COLD, ADDS, FROZEN, SPIRIT]).prop_map(quoted),
        3 => proptest::sample::select(vec![
            "template:life",
            "template:resistance",
            "template:cold",
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
    /// `sum(line(G).slot)` and its comparison.
    Sum(G, String, String),
    /// `line(G).slot` and its comparison: lowers into the group.
    Proj(G, String, String),
    /// `undecided(line(G).slot)`, or of the sum when `true`.
    Open(G, String, bool),
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
            "has:note",
            "has:name",
            "note:price",
            "is:corrupted",
            "is:shaper",
            "is:hunter",
            "\"life\"",
            "\"Level: 8\"",
            "text~\"^adds\"",
            "tab:routes",
            "true()",
            "false()",
        ]),
        // an authoring error: never evaluated, and the same error however
        // the rest is written
        1 => proptest::sample::select(vec!["nosuch=1", "class:ring", "is:crafted", "ilvl:84"]),
    ]
    .prop_map(|text| Q::Plain(text.to_string()))
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
        4 => plain(),
        2 => alt(),
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
        1 => proptest::sample::select(vec!["ilvl", "stack"]).prop_map(Sort::Field),
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
        Q::Plain(term) => term.clone(),
        Q::Alt(spelled, explicit, _) => match spelling {
            Spelling::Authored => spelled.clone(),
            Spelling::Explicit => explicit.clone(),
        },
        Q::Line(g) => line_text(g, spelling),
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
        Q::Plain(_) | Q::Alt(..) => q.clone(),
        Q::Line(g) => Q::Line(g_normal(g)),
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
            Q::Plain(_) | Q::Alt(..) => q.clone(),
            Q::Line(g) => Q::Line(g_shuffled(g, rng)),
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
        Q::Plain(_) | Q::Alt(..) => q.clone(),
        Q::Line(g) => Q::Line(g_sorted(g)),
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
            Q::Line(_) | Q::Proj(..) => out.push((key(q), TermKind::Group)),
            Q::Sum(..) => out.push((key(q), TermKind::Sum)),
            Q::Open(..) => out.push((key(q), TermKind::Probe)),
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
    pub name: Known<&'static str>,
    pub note: Known<&'static str>,
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
    (0u8..7, LOW..=HIGH, flags)
        .prop_map(|(kind, n, flags)| LineM { kind, n, flags })
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
        known(vec!["Doom Knot", "Life Ring"], holes),
        known(vec!["~price 1 chaos", "keep"], holes),
    )
        .prop_map(
            |(
                (explicit, implicit, hybrid),
                (corrupted, influences_hole, shaper, hunter),
                ilvl,
                name,
                note,
            )| Body {
                explicit,
                implicit,
                hybrid,
                corrupted,
                influences_hole,
                shaper,
                hunter,
                ilvl,
                name,
                note,
            },
        )
        .boxed()
}

/// A body with something unread in it, whatever the dice gave.
pub fn holed() -> BoxedStrategy<Body> {
    (some_body(true), 0u8..4)
        .prop_map(|(mut body, which)| {
            if !body.has_holes() {
                match which {
                    0 => body.implicit = Lines::Hole,
                    1 => body.explicit = Lines::Hole,
                    2 => body.corrupted = Tri::Hole,
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
            name: Known::Absent,
            note: Known::Absent,
        }
    }

    /// A readable body whose arrays hold `lines`, and whose numbers, names
    /// and yes-or-noes are the `n`th of a fixed round — so that a handful
    /// of completions tries an unread number as several, a name as two and
    /// as none, and three flags every way, whatever the dice gave.
    pub fn nth(n: usize, lines: Vec<LineM>) -> Body {
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
        }
    }

    /// One line of every kind at each of four values, its flags as given.
    pub fn every_line(flag: Tri) -> Vec<LineM> {
        (0u8..7)
            .flat_map(|kind| {
                [LOW, 0, 1, HIGH].map(|n| LineM {
                    kind,
                    n,
                    flags: Flags::Each {
                        crafted: flag,
                        fractured: flag,
                    },
                })
            })
            .collect()
    }

    /// The body with the elements of every array in the opposite order
    /// (transformation 4).
    pub fn every_line_reversed(&self) -> Body {
        let reversed = |lines: &Lines| match lines {
            Lines::Of(elems) => Lines::Of(elems.iter().rev().cloned().collect()),
            other => other.clone(),
        };
        Body {
            explicit: reversed(&self.explicit),
            implicit: reversed(&self.implicit),
            hybrid: reversed(&self.hybrid),
            ..self.clone()
        }
    }

    /// The body with every element of every array written twice
    /// (transformation 10).
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
                Elem::Line(l) => match &l.flags {
                    Flags::Hole => true,
                    Flags::Each { crafted, fractured } => tri(*crafted) || tri(*fractured),
                },
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
            || self.name == Known::Hole
            || self.note == Known::Hole
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
                            out.push(Elem::Line(LineM {
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
        Body {
            explicit,
            implicit,
            hybrid,
            corrupted: filled(self.corrupted, fill.corrupted),
            influences_hole: false,
            shaper: influence(self.shaper, fill.shaper),
            hunter: influence(self.hunter, fill.hunter),
            ilvl: match (&self.ilvl, &fill.ilvl) {
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
            let description = match l.kind {
                0 => format!("{:+} to maximum Life", l.n),
                1 => format!("{:+}% to Cold Resistance", l.n),
                2 => format!("Adds {} to {} Cold Damage", l.n.abs(), l.n.abs() + 7),
                3 => FROZEN.to_string(),
                4 => format!("{:+} to Spirit", l.n),
                5 => format!("{}% of Damage Leeched as Life", tenths(l.n)),
                // GGG has spelled some lines two ways
                _ => format!("{:+} to maximum life", l.n),
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
        Value::Object(body)
    }
}

/// The six items the audits' reproductions were made of.
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
    ]
}

// ---- the boundary -----------------------------------------------------------------------------------

/// One tab of rare rings `i0`, `i1`, … in the realm `pc`, and an item in
/// another realm that no answer over `pc` may return.
pub fn fixture(bodies: Vec<Value>) -> (Corpus, Ids) {
    let (_, corpus, ids) = fixture_with_store(bodies);
    (corpus, ids)
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
        json!([super::tab("t", "Routes")]),
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
    let corpus = super::load(&store, Some("pc"));
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
