//! The binder (C91, C92, C93; the build plan, step 4): a tree in, a query
//! the evaluator can run out — or an authoring error. The parser knows the
//! grammar and no name; every name is known here, from closed lists this
//! file holds, so a query's validity never depends on a corpus (invariant
//! 3 of the surface).
//!
//! What the words mean is `search/DESIGN.md`'s reference (*Item-level*,
//! *Members*, *Values*), cited by section and not restated (the build
//! plan, rule 6).
//!
//! # As built
//!
//! - **One closed list of what is not built** ([`NOT_BUILT`]): a construct
//!   of the reference and the step that builds it. A query, a sort or a
//!   flag that uses one is refused before anything is evaluated, by an
//!   error of its own kind ([`ErrorKind::NotBuilt`]) that names the
//!   construct — never the unknown-name error, never undecided, never an
//!   empty answer. `--describe` prints this list, and
//!   `tests/refusal.rs` walks it.
//! - **Every text comparison is one matcher, in any case.** `:` contains,
//!   `=` the whole value, `~` a pattern (Rust regex syntax, unanchored);
//!   each compiles to a regex, so `rarity=rare` finds GGG's `Rare` and
//!   `name="kaom's heart"` finds `Kaom's Heart` by the same rule. A pattern
//!   may turn case back on for itself: `(?-i)`.
//! - **A closed set is a list here**: `rarity`, `frame`, the `is:` words,
//!   a line's `source` and its flags — GGG's own spellings, as the census
//!   of 2026-09-13 met them (22,721 items), matched in any case. `=` names
//!   one value, `:` and `~` pick among the legal ones, and a word that
//!   picks none is an authoring error with the near ones offered. A flag
//!   or a source GGG adds is derived and shown the day it appears and
//!   cannot be asked for until its list gains it.
//! - **Atomic terms are numbered by path** — `0`, `1`, `3.1` — as the
//!   answer's terms block prints them (C93): a child of an and, an or or a
//!   `holds` is `<parent>.<n>`, what a not negates or `undecided( … )`
//!   asks about is `<parent>.0`, and the children of the root carry no
//!   prefix.
//! - **A line group's selector** is the group with its slot comparisons
//!   taken as favourably as they can be and folded away: what the group
//!   picks before any number is compared, by meaning and never by where a
//!   comparison sits. It is what tells *lacked* (no occurrence is
//!   selected) from *failed* (one is, and none satisfies the whole), what
//!   the answer says a `:` or `~` template selector resolved to, and what
//!   the together count sums (C92).
//! - **A bare word's closed-set readings** arrive here: the parser offers
//!   `"rare"` and `line(template:rare)`, and [`parse_query`] puts
//!   `rarity=rare` before them.

use regex::{Regex, RegexBuilder};

use crate::error::{ErrorKind, LanguageError};
use crate::print;
use crate::tree::{self, Collection, Member, Node, Number, Op, Probe, Value, ValueRef};

// ---- what is not built -------------------------------------------------------

/// A construct of the reference this build refuses, and the step of
/// `search/BUILD-PLAN.md` that builds it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct NotBuilt {
    pub construct: &'static str,
    pub step: u8,
    pub what: &'static str,
}

/// The one list (the build plan, rule 1). An entry leaves in the commit
/// that builds its construct.
pub const NOT_BUILT: &[NotBuilt] = &[
    NotBuilt {
        construct: "--count",
        step: 5,
        what: "counts by one or more keys, one table each",
    },
    NotBuilt {
        construct: "--cross",
        step: 5,
        what: "one crossed table of two keys",
    },
    NotBuilt {
        construct: "--sum",
        step: 5,
        what: "one summed value beside a count",
    },
    NotBuilt {
        construct: "class:",
        step: 6,
        what: "an item's class is not a field; a class the search names is a derivation it owns, and an item it cannot class is shown unclassed (S53)",
    },
    NotBuilt {
        construct: "reqlevel",
        step: 6,
        what: "the level an item requires, as a number",
    },
    NotBuilt {
        construct: "pseudo.*",
        step: 7,
        what: "computed values: named totals and derived fields",
    },
    NotBuilt {
        construct: "sockets",
        step: 8,
        what: "how many sockets an item has",
    },
    NotBuilt {
        construct: "links",
        step: 8,
        what: "the size of an item's largest link group",
    },
    NotBuilt {
        construct: "sockets.<colour>",
        step: 8,
        what: "how many sockets of one colour",
    },
    NotBuilt {
        construct: "linked(…)",
        step: 8,
        what: "conditions that hold together on one link group",
    },
    NotBuilt {
        construct: "has:priced",
        step: 9,
        what: "whether the item carries an effective price",
    },
    NotBuilt {
        construct: "price.*",
        step: 9,
        what: "the effective price: amount, currency, lot",
    },
    NotBuilt {
        construct: "--fields",
        step: 10,
        what: "the caller names a row's fields",
    },
    NotBuilt {
        construct: "--next",
        step: 10,
        what: "continue an answer past its limit, refused across a changed basis",
    },
    NotBuilt {
        construct: "--explain",
        step: 10,
        what: "one node forced true and forced false",
    },
    NotBuilt {
        construct: "--context",
        step: 10,
        what: "the empty query over the bound scope",
    },
    NotBuilt {
        construct: "--view locations",
        step: 10,
        what: "the full coverage list the scope block summarises",
    },
    NotBuilt {
        construct: "--print-request",
        step: 10,
        what: "print the request as JSON without running it",
    },
    NotBuilt {
        construct: "--request",
        step: 10,
        what: "run a request read from a file",
    },
    NotBuilt {
        construct: "--rebind",
        step: 10,
        what: "run another account's request deliberately",
    },
    NotBuilt {
        construct: "show --against",
        step: 10,
        what: "why one item does or does not match a query",
    },
    NotBuilt {
        construct: "show --basis",
        step: 10,
        what: "one item as a named basis held it",
    },
];

/// The refusal of one construct of [`NOT_BUILT`], by its name there.
pub fn not_built(construct: &str) -> LanguageError {
    match NOT_BUILT.iter().find(|n| n.construct == construct) {
        Some(n) => LanguageError::new(
            ErrorKind::NotBuilt,
            format!("not built: {} (step {}) — {}", n.construct, n.step, n.what),
        ),
        // a name outside the list is a bug in the caller, said as loudly
        None => LanguageError::new(
            ErrorKind::NotBuilt,
            format!("not built: {construct} (no step names it)"),
        ),
    }
}

/// The construct of [`NOT_BUILT`] a field's name belongs to.
fn unbuilt_field(name: &str) -> Option<&'static str> {
    match name {
        "class" => Some("class:"),
        "reqlevel" => Some("reqlevel"),
        "sockets" => Some("sockets"),
        "links" => Some("links"),
        "priced" => Some("has:priced"),
        "price" => Some("price.*"),
        _ if name.starts_with("sockets.") => Some("sockets.<colour>"),
        _ if name.starts_with("price.") => Some("price.*"),
        _ => None,
    }
}

// ---- the vocabulary ------------------------------------------------------------

pub(crate) const RARITIES: &[&str] = &["normal", "magic", "rare", "unique"];

/// GGG's `frameTypeId`, the nine the census met.
pub(crate) const FRAMES: &[&str] = &[
    "normal",
    "magic",
    "rare",
    "unique",
    "gem",
    "currency",
    "divinationcard",
    "quest",
    "supporterfoil",
];

pub(crate) use crate::derive::ITEM_FLAGS;

/// A line's source: its array's key without `Mods`, and `hybrid`.
pub(crate) const SOURCES: &[&str] = &[
    "bonded", "crucible", "enchant", "explicit", "hybrid", "implicit", "rune", "scourge",
    "utility", "veiled",
];

/// The flags a line carries (S17).
pub(crate) const LINE_FLAGS: &[&str] = &["crafted", "fractured", "mutated"];

/// Something of an item a term names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Thing {
    /// Every displayed string, each on its own.
    Text,
    Name,
    Typeline,
    Base,
    Note,
    Rarity,
    Frame,
    Ilvl,
    Stack,
    League,
    Tab,
    Character,
    Container,
    Id,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Text,
    Closed(&'static [&'static str]),
    Number,
    Handle,
}

pub(crate) struct FieldDef {
    pub name: &'static str,
    pub thing: Thing,
    pub kind: Kind,
    pub what: &'static str,
}

pub(crate) const FIELDS: &[FieldDef] = &[
    FieldDef {
        name: "text",
        thing: Thing::Text,
        kind: Kind::Text,
        what: "every displayed string, each tested on its own: the name, the type line, the base, each property, the item level, the requirements' row, each row of each line; what a quoted phrase searches",
    },
    FieldDef {
        name: "name",
        thing: Thing::Name,
        kind: Kind::Text,
        what: "GGG's name; a magic or normal item, a gem, a currency stack has none",
    },
    FieldDef {
        name: "typeline",
        thing: Thing::Typeline,
        kind: Kind::Text,
        what: "GGG's type line: the base with what it carries, a magic item's affix names among it",
    },
    FieldDef {
        name: "base",
        thing: Thing::Base,
        kind: Kind::Text,
        what: "GGG's base type",
    },
    FieldDef {
        name: "note",
        thing: Thing::Note,
        kind: Kind::Text,
        what: "the item's note as GGG gives it; never part of `text`",
    },
    FieldDef {
        name: "rarity",
        thing: Thing::Rarity,
        kind: Kind::Closed(RARITIES),
        what: "GGG's rarity; a gem, a currency stack, a card has none",
    },
    FieldDef {
        name: "frame",
        thing: Thing::Frame,
        kind: Kind::Closed(FRAMES),
        what: "GGG's frameTypeId, on every item",
    },
    FieldDef {
        name: "ilvl",
        thing: Thing::Ilvl,
        kind: Kind::Number,
        what: "the item level; GGG's 0 is an item with none",
    },
    FieldDef {
        name: "stack",
        thing: Thing::Stack,
        kind: Kind::Number,
        what: "GGG's stackSize",
    },
    FieldDef {
        name: "league",
        thing: Thing::League,
        kind: Kind::Text,
        what: "the league the item is in now — its place, never where it came from: league of origin is not a fact the search holds (S177)",
    },
    FieldDef {
        name: "tab",
        thing: Thing::Tab,
        kind: Kind::Text,
        what: "the name of the stash tab the item is in; in a substash, that name and its tab's",
    },
    FieldDef {
        name: "character",
        thing: Thing::Character,
        kind: Kind::Text,
        what: "the name of the character the item is on",
    },
    FieldDef {
        name: "container",
        thing: Thing::Container,
        kind: Kind::Text,
        what: "the array the item came in: items, equipment, inventory, jewels, skills, rucksack, guardian",
    },
    FieldDef {
        name: "id",
        thing: Thing::Id,
        kind: Kind::Handle,
        what: "any id an answer printed, whole: the item's own, or its tab's, its substash's or its character's",
    },
];

fn field(name: &str) -> Option<&'static FieldDef> {
    FIELDS.iter().find(|f| f.name.eq_ignore_ascii_case(name))
}

/// A closed value by any-case name, in the list's own spelling.
fn legal(list: &'static [&'static str], word: &str) -> Option<&'static str> {
    list.iter().copied().find(|v| v.eq_ignore_ascii_case(word))
}

// ---- near names ------------------------------------------------------------------

fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut last = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { last } else { last + 1 };
            last = row[j + 1];
            row[j + 1] = cost.min(last + 1).min(row[j] + 1);
        }
    }
    row[b.len()]
}

/// The known names a typed one may have meant, nearest first: an edit or
/// two away, or one the start of the other. A suggestion, never a binding.
pub(crate) fn near<'a>(word: &str, known: &[&'a str]) -> Vec<&'a str> {
    let word = word.to_ascii_lowercase();
    let mut scored: Vec<(usize, &'a str)> = known
        .iter()
        .filter_map(|k| {
            let lower = k.to_ascii_lowercase();
            let d = distance(&word, &lower);
            let budget = if word.chars().count() <= 3 { 1 } else { 2 };
            if d <= budget {
                Some((d, *k))
            } else if lower.starts_with(&word) || word.starts_with(&lower) {
                Some((budget + 1, *k))
            } else {
                None
            }
        })
        .collect();
    scored.sort();
    scored.into_iter().map(|(_, k)| k).take(3).collect()
}

// ---- the bound query -----------------------------------------------------------------

/// One text comparison, compiled (the module doc: one matcher, any case).
#[derive(Debug, Clone)]
pub(crate) struct TextTest(Regex);

impl TextTest {
    pub fn holds(&self, text: &str) -> bool {
        self.0.is_match(text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NumTest {
    Cmp(Op, f64),
    Range(Option<f64>, Option<f64>),
}

impl NumTest {
    pub fn holds(&self, x: f64) -> bool {
        match *self {
            NumTest::Cmp(Op::Gt, n) => x > n,
            NumTest::Cmp(Op::Ge, n) => x >= n,
            NumTest::Cmp(Op::Lt, n) => x < n,
            NumTest::Cmp(Op::Le, n) => x <= n,
            NumTest::Cmp(_, n) => x == n,
            NumTest::Range(from, to) => from.is_none_or(|a| x >= a) && to.is_none_or(|b| x <= b),
        }
    }
}

/// A node inside a line's group, bound.
#[derive(Debug, Clone)]
pub(crate) enum BMember {
    All(Vec<BMember>),
    Any(Vec<BMember>),
    Not(Box<BMember>),
    Const(bool),
    Template(TextTest),
    Source(Vec<&'static str>),
    Slot { word: String, test: NumTest },
    Is(&'static str),
}

/// A lower bound on one slot of a line selector: what the together count
/// is defined for (C92).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LowerBound {
    pub slot: String,
    pub op: Op,
    pub bound: Number,
}

/// `line( … )`, bound, with its selector (the module doc).
#[derive(Debug, Clone)]
pub(crate) struct Group {
    pub whole: BMember,
    pub selector: BMember,
    /// The selector as a tree, for the routes that name it.
    pub selector_tree: Member,
    /// True when the group compares no slot: the selector is the whole.
    pub selects_only: bool,
    pub together: Option<LowerBound>,
}

#[derive(Debug, Clone)]
pub(crate) enum Atom {
    Text {
        thing: Thing,
        test: TextTest,
    },
    Closed {
        thing: Thing,
        values: Vec<&'static str>,
    },
    Number {
        thing: Thing,
        test: NumTest,
    },
    Id(String),
    Has(Thing),
    Is(&'static str),
    Lines(Group),
    Sum {
        group: Group,
        slot: String,
        test: NumTest,
    },
    Const(bool),
    Undecided(BProbe),
}

#[derive(Debug, Clone)]
pub(crate) enum BProbe {
    Field(Thing),
    /// A sum or a projection: undecided when a possible contributor is unread.
    Lines(Box<Group>),
    Term(Box<Bound>),
}

#[derive(Debug, Clone)]
pub(crate) enum Bound {
    All(Vec<Bound>),
    Any(Vec<Bound>),
    Not(Box<Bound>),
    Holds {
        of: Vec<Bound>,
        min: Option<u32>,
        max: Option<u32>,
    },
    /// An atomic term, by its index in [`Query::terms`].
    Term(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct Term {
    pub path: String,
    pub node: Node,
    pub atom: Atom,
}

/// A query the evaluator can run: the tree as authored, and what its
/// names were bound to.
#[derive(Debug, Clone)]
pub struct Query {
    pub(crate) root: Node,
    pub(crate) bound: Bound,
    pub(crate) terms: Vec<Term>,
}

impl Query {
    pub fn tree(&self) -> &Node {
        &self.root
    }

    pub fn text(&self) -> String {
        print::print(&self.root)
    }
}

/// What `--sort` orders by (C92, the sort scalar).
#[derive(Debug, Clone)]
pub(crate) enum SortKey {
    Number(Thing),
    Projection { group: Group, slot: String },
    Sum { group: Group, slot: String },
}

/// Parse and bind a query's text.
pub fn parse_query(text: &str) -> Result<Query, LanguageError> {
    let root = crate::parse::parse(text).map_err(|e| closed_set_readings(e, text))?;
    bind(&root)
}

/// Bind a tree: checked first, since one that arrived as JSON may not be a
/// tree the language can say.
pub fn bind(root: &Node) -> Result<Query, LanguageError> {
    tree::check(root)?;
    let mut binder = Binder { terms: Vec::new() };
    let bound = match root {
        Node::All(children) => Bound::All(binder.children(children, "")?),
        other => binder.node(other, "0".to_string())?,
    };
    Ok(Query {
        root: root.clone(),
        bound,
        terms: binder.terms,
    })
}

/// Bind what `--sort` takes.
pub(crate) fn bind_sort(value: &ValueRef) -> Result<SortKey, LanguageError> {
    match value {
        ValueRef::Pseudo { .. } => Err(not_built("pseudo.*")),
        ValueRef::Field(name) => {
            let def = known_field(name, |near| near.to_string())?;
            match def.kind {
                Kind::Number => Ok(SortKey::Number(def.thing)),
                _ => Err(LanguageError::new(
                    ErrorKind::OperatorMismatch,
                    format!(
                        "`--sort` takes a number: `{}` is not one — ilvl, stack, a line's slot or a sum",
                        def.name
                    ),
                )),
            }
        }
        ValueRef::Projection { lines, slot } => Ok(SortKey::Projection {
            group: group(lines)?,
            slot: slot.clone(),
        }),
        ValueRef::Sum { lines, slot } => Ok(SortKey::Sum {
            group: group(lines)?,
            slot: slot.clone(),
        }),
    }
}

struct Binder {
    terms: Vec<Term>,
}

impl Binder {
    fn children(&mut self, children: &[Node], prefix: &str) -> Result<Vec<Bound>, LanguageError> {
        children
            .iter()
            .enumerate()
            .map(|(i, child)| {
                let path = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}.{i}")
                };
                self.node(child, path)
            })
            .collect()
    }

    fn node(&mut self, node: &Node, path: String) -> Result<Bound, LanguageError> {
        Ok(match node {
            Node::All(children) => Bound::All(self.children(children, &path)?),
            Node::Any(children) => Bound::Any(self.children(children, &path)?),
            Node::Not(inner) => Bound::Not(Box::new(self.node(inner, format!("{path}.0"))?)),
            Node::Holds { of, min, max } => Bound::Holds {
                of: self.children(of, &path)?,
                min: *min,
                max: *max,
            },
            leaf => {
                let atom = self.atom(leaf, &path)?;
                self.terms.push(Term {
                    path,
                    node: leaf.clone(),
                    atom,
                });
                Bound::Term(self.terms.len() - 1)
            }
        })
    }

    fn atom(&mut self, node: &Node, path: &str) -> Result<Atom, LanguageError> {
        match node {
            Node::Const(value) => Ok(Atom::Const(*value)),
            Node::Test { field, op, value } => {
                let def = known_field(field, |near| {
                    print::print(&Node::Test {
                        field: near.to_string(),
                        op: *op,
                        value: value.clone(),
                    })
                })?;
                test(def, *op, value)
            }
            Node::Has(name) => {
                let def = known_field(name, |near| format!("has:{near}"))?;
                match def.thing {
                    Thing::Text | Thing::Id => Err(LanguageError::new(
                        ErrorKind::OperatorMismatch,
                        format!("every item has `{}`: `has:` asks nothing of it", def.name),
                    )),
                    thing => Ok(Atom::Has(thing)),
                }
            }
            Node::Is(name) => match legal(ITEM_FLAGS, name) {
                Some(flag) => Ok(Atom::Is(flag)),
                None if legal(LINE_FLAGS, name).is_some() => Err(LanguageError::new(
                    ErrorKind::UnknownName,
                    format!("`is:{name}` is said of a line, inside its group"),
                )
                .with_readings(vec![format!("line(is:{name})")])),
                None => Err(unknown("is:", name, ITEM_FLAGS, |near| {
                    format!("is:{near}")
                })),
            },
            Node::Members {
                of: Collection::Links,
                ..
            } => Err(not_built("linked(…)")),
            Node::Members {
                of: Collection::Lines,
                where_,
            } => Ok(Atom::Lines(group(where_)?)),
            Node::Compare { value, op, rhs } => match value {
                ValueRef::Sum { lines, slot } => Ok(Atom::Sum {
                    group: group(lines)?,
                    slot: slot.clone(),
                    test: num_test("sum( … )", *op, rhs)?,
                }),
                ValueRef::Pseudo { .. } => Err(not_built("pseudo.*")),
                // `tree::check` has refused both
                ValueRef::Field(_) | ValueRef::Projection { .. } => Err(LanguageError::new(
                    ErrorKind::Tree,
                    "a comparison is on a sum or a computed value",
                )),
            },
            Node::Undecided(probe) => Ok(Atom::Undecided(match probe {
                Probe::Thing(ValueRef::Pseudo { .. }) => return Err(not_built("pseudo.*")),
                Probe::Thing(ValueRef::Field(name)) => {
                    BProbe::Field(known_field(name, |near| format!("undecided({near})"))?.thing)
                }
                Probe::Thing(ValueRef::Sum { lines, .. } | ValueRef::Projection { lines, .. }) => {
                    BProbe::Lines(Box::new(group(lines)?))
                }
                Probe::Term(inner) => {
                    BProbe::Term(Box::new(self.node(inner, format!("{path}.0"))?))
                }
            })),
            // composition is `node`'s
            Node::All(_) | Node::Any(_) | Node::Not(_) | Node::Holds { .. } => Err(
                LanguageError::new(ErrorKind::Tree, "a composition is not an atomic term"),
            ),
        }
    }
}

fn known_field(
    name: &str,
    reading: impl Fn(&str) -> String,
) -> Result<&'static FieldDef, LanguageError> {
    if let Some(construct) = unbuilt_field(name) {
        return Err(not_built(construct));
    }
    field(name).ok_or_else(|| {
        let names: Vec<&'static str> = FIELDS.iter().map(|f| f.name).collect();
        unknown("field", name, &names, reading)
    })
}

fn unknown(
    what: &str,
    name: &str,
    known: &[&'static str],
    reading: impl Fn(&str) -> String,
) -> LanguageError {
    let near = near(name, known);
    let message = if near.is_empty() {
        format!(
            "`{name}` is no {what} the language knows: {}",
            known.join(", ")
        )
    } else {
        format!("`{name}` is no {what} the language knows; near it:")
    };
    LanguageError::new(ErrorKind::UnknownName, message)
        .with_readings(near.into_iter().map(reading).collect())
}

fn test(def: &'static FieldDef, op: Op, value: &Value) -> Result<Atom, LanguageError> {
    match def.kind {
        Kind::Text => Ok(Atom::Text {
            thing: def.thing,
            test: text_test(def.name, op, value)?,
        }),
        Kind::Closed(list) => Ok(Atom::Closed {
            thing: def.thing,
            values: closed(def.name, list, op, value)?,
        }),
        Kind::Number => match op {
            Op::Contains | Op::Match => Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!("`{}` is a number: it takes = > >= < <= and a..b", def.name),
            )
            .with_readings(match value {
                Value::Number(_) => vec![print::print(&Node::Test {
                    field: def.name.to_string(),
                    op: Op::Eq,
                    value: value.clone(),
                })],
                _ => Vec::new(),
            })),
            _ => Ok(Atom::Number {
                thing: def.thing,
                test: num_test(def.name, op, value)?,
            }),
        },
        Kind::Handle => match (op, value) {
            (Op::Contains | Op::Eq, Value::Text(id)) => Ok(Atom::Id(id.clone())),
            (Op::Contains | Op::Eq, Value::Number(n)) => Ok(Atom::Id(number_text(*n))),
            _ => Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                "`id` takes `:` or `=` and a whole id, as an answer printed it",
            )),
        },
    }
}

fn number_text(n: Number) -> String {
    match n {
        Number::Int(i) => i.to_string(),
        Number::Float(f) => f.to_string(),
    }
}

fn text_test(name: &str, op: Op, value: &Value) -> Result<TextTest, LanguageError> {
    let text = match value {
        Value::Text(text) => text.clone(),
        Value::Number(n) => number_text(*n),
        Value::Range { .. } => {
            return Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!("`{name}` is text: a range is for a number"),
            ));
        }
    };
    let pattern = match op {
        Op::Contains => regex::escape(&text),
        Op::Eq => format!("^(?:{})$", regex::escape(&text)),
        Op::Match => text.clone(),
        _ => {
            return Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!(
                    "`{name}` is text: it takes `:` contains, `=` the whole value, `~` a pattern"
                ),
            ));
        }
    };
    RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
        .map(TextTest)
        .map_err(|e| {
            LanguageError::new(
                ErrorKind::BadPattern,
                format!("`{name}~` takes a pattern in Rust regex syntax, and this is not one: {e}"),
            )
        })
}

fn num_test(name: &str, op: Op, value: &Value) -> Result<NumTest, LanguageError> {
    match (op, value) {
        (Op::Eq, Value::Range { from, to }) => Ok(NumTest::Range(
            from.map(Number::as_f64),
            to.map(Number::as_f64),
        )),
        (Op::Eq | Op::Gt | Op::Ge | Op::Lt | Op::Le, Value::Number(n)) => {
            Ok(NumTest::Cmp(op, n.as_f64()))
        }
        _ => Err(LanguageError::new(
            ErrorKind::OperatorMismatch,
            format!("`{name}` is a number: it takes = > >= < <= with a number, and = with a..b"),
        )),
    }
}

/// The legal values a closed-set test names or picks.
fn closed(
    name: &str,
    list: &'static [&'static str],
    op: Op,
    value: &Value,
) -> Result<Vec<&'static str>, LanguageError> {
    let Value::Text(word) = value else {
        return Err(LanguageError::new(
            ErrorKind::OperatorMismatch,
            format!("`{name}` is one of: {}", list.join(", ")),
        ));
    };
    let picked: Vec<&'static str> = match op {
        Op::Eq => legal(list, word).into_iter().collect(),
        Op::Contains => {
            let word = word.to_ascii_lowercase();
            list.iter()
                .copied()
                .filter(|v| v.to_ascii_lowercase().contains(&word))
                .collect()
        }
        Op::Match => {
            let test = text_test(name, op, value)?;
            list.iter().copied().filter(|v| test.holds(v)).collect()
        }
        _ => {
            return Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!(
                    "`{name}` is a closed set: `=` names one value, `:` and `~` pick among them"
                ),
            ));
        }
    };
    if !picked.is_empty() {
        return Ok(picked);
    }
    let offered = match near(word, list) {
        near if near.is_empty() => list.to_vec(),
        near => near,
    };
    Err(LanguageError::new(
        ErrorKind::UnknownValue,
        format!("`{word}` is no `{name}`: {}", list.join(", ")),
    )
    .with_readings(offered.iter().map(|v| format!("{name}={v}")).collect()))
}

// ---- a line's group ---------------------------------------------------------------------

fn has_slot(member: &Member) -> bool {
    match member {
        Member::All(children) | Member::Any(children) => children.iter().any(has_slot),
        Member::Not(inner) => has_slot(inner),
        Member::Test { attr, .. } => tree::is_slot_word(attr),
        Member::Const(_) | Member::Is(_) => false,
    }
}

/// The group's selector (the module doc): the group with every slot
/// comparison taken as favourably as it can be — true where it helps the
/// group hold, false under a not — and the constants folded away. By
/// meaning, never by where a comparison sits: an occurrence the selector
/// refuses is one no numbers could make the group hold on, so
/// `"T" (source=explicit arg1>=90)` selects as `"T" source=explicit` does,
/// and `"T" arg1>=90` selects as `"T"`.
pub(crate) fn selector(whole: &Member) -> Member {
    fn favoured(member: &Member, positive: bool) -> Member {
        match member {
            Member::All(children) => {
                Member::All(children.iter().map(|c| favoured(c, positive)).collect())
            }
            Member::Any(children) => {
                Member::Any(children.iter().map(|c| favoured(c, positive)).collect())
            }
            Member::Not(inner) => Member::Not(Box::new(favoured(inner, !positive))),
            Member::Test { attr, .. } if tree::is_slot_word(attr) => Member::Const(positive),
            other => other.clone(),
        }
    }
    /// Fold the constants out: a generated tree, so nothing the author
    /// wrote is simplified (invariant 1 is the canonical text's).
    fn folded(member: Member) -> Member {
        let group = |children: Vec<Member>, absorbs: bool| {
            let mut kept = Vec::new();
            for child in children.into_iter().map(folded) {
                match child {
                    Member::Const(value) if value == absorbs => return Member::Const(absorbs),
                    Member::Const(_) => {}
                    other => kept.push(other),
                }
            }
            match kept.len() {
                0 => Member::Const(!absorbs),
                1 => kept.remove(0),
                _ if absorbs => Member::Any(kept),
                _ => Member::All(kept),
            }
        };
        match member {
            Member::All(children) => group(children, false),
            Member::Any(children) => group(children, true),
            Member::Not(inner) => match folded(*inner) {
                Member::Const(value) => Member::Const(!value),
                other => Member::Not(Box::new(other)),
            },
            other => other,
        }
    }
    folded(favoured(whole, true))
}

pub(crate) fn group(whole: &Member) -> Result<Group, LanguageError> {
    let selector_tree = selector(whole);
    let bound = member(whole)?;
    let top: &[Member] = match whole {
        Member::All(children) => children,
        other => std::slice::from_ref(other),
    };
    let slotted: Vec<&Member> = top.iter().filter(|c| has_slot(c)).collect();
    let together = match slotted.as_slice() {
        [
            Member::Test {
                attr,
                op: op @ (Op::Gt | Op::Ge),
                value: Value::Number(bound),
            },
        ] => Some(LowerBound {
            slot: attr.clone(),
            op: *op,
            bound: *bound,
        }),
        _ => None,
    };
    Ok(Group {
        selector: member(&selector_tree)?,
        selects_only: !has_slot(whole),
        whole: bound,
        selector_tree,
        together,
    })
}

fn member(m: &Member) -> Result<BMember, LanguageError> {
    Ok(match m {
        Member::All(children) => {
            BMember::All(children.iter().map(member).collect::<Result<_, _>>()?)
        }
        Member::Any(children) => {
            BMember::Any(children.iter().map(member).collect::<Result<_, _>>()?)
        }
        Member::Not(inner) => BMember::Not(Box::new(member(inner)?)),
        Member::Const(value) => BMember::Const(*value),
        Member::Is(name) => match legal(LINE_FLAGS, name) {
            Some(flag) => BMember::Is(flag),
            None if legal(ITEM_FLAGS, name).is_some() => {
                return Err(LanguageError::new(
                    ErrorKind::UnknownName,
                    format!(
                        "`is:{name}` is said of the item, outside the line's group; a line is: {}",
                        LINE_FLAGS.join(", ")
                    ),
                ));
            }
            None => {
                return Err(unknown("flag of a line", name, LINE_FLAGS, |near| {
                    format!("is:{near}")
                }));
            }
        },
        Member::Test { attr, op, value } => match attr.as_str() {
            "template" => BMember::Template(text_test("template", *op, value)?),
            "source" => BMember::Source(closed("source", SOURCES, *op, value)?),
            slot if tree::is_slot_word(slot) => BMember::Slot {
                word: slot.to_string(),
                test: num_test(slot, *op, value)?,
            },
            other => {
                return Err(LanguageError::new(
                    ErrorKind::UnknownName,
                    format!(
                        "`{other}` is nothing a line has: template, source, is:<flag>, and its numbers — low, high, avg on a ranged line, arg1, arg2 … by position"
                    ),
                ));
            }
        },
    })
}

// ---- a bare word's closed-set readings -------------------------------------------------

/// The parser's bare-word error, with the readings only the vocabulary
/// knows put first: `rare` → `rarity=rare`, `corrupted` → `is:corrupted`.
fn closed_set_readings(mut e: LanguageError, text: &str) -> LanguageError {
    if e.kind != ErrorKind::BareWord {
        return e;
    }
    let Some(word) = e.span.and_then(|(start, end)| text.get(start..end)) else {
        return e;
    };
    // the parser's readings say which level the word stood at
    let inside = !e.readings.first().is_some_and(|r| r.starts_with('"'));
    let mut first = Vec::new();
    if inside {
        first.extend(legal(SOURCES, word).map(|v| format!("source={v}")));
        first.extend(legal(LINE_FLAGS, word).map(|v| format!("is:{v}")));
    } else {
        first.extend(legal(RARITIES, word).map(|v| format!("rarity={v}")));
        if legal(RARITIES, word).is_none() {
            first.extend(legal(FRAMES, word).map(|v| format!("frame={v}")));
        }
        first.extend(legal(ITEM_FLAGS, word).map(|v| format!("is:{v}")));
    }
    first.append(&mut e.readings);
    e.readings = first;
    e
}
