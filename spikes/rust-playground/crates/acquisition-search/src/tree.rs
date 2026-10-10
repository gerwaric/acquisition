//! The query tree (C91): one typed value, immutable and serializable, that
//! the text and the JSON form each print to and parse from (C104).
//!
//! A tree is always *lowered*: the shorthand of the text (`"T">=90`,
//! `line(P).avg>=20`, `sum("T")`) never appears in it. [`check`] states
//! what a tree must satisfy to be one the printer can print and the
//! parser would give back; the parser's output satisfies it by
//! construction, and a tree arriving as JSON is checked before use.
//!
//! # As built
//!
//! **A path names a node** (`search/DESIGN.md`'s reference, *Explain*;
//! the build plan, step 10). One walk numbers a query's nodes
//! ([`nodes`]) and replaces one of them by a constant ([`forced`]), so
//! what a path names and what a substitution replaces cannot differ: the
//! children of the root are `0`, `1`, …, and a root that is no and is
//! `0`; a child of an and, an or or a `holds` is `<parent>.<n>`; what a
//! not negates or `undecided( … )` asks about is `<parent>.0`. Inside
//! `line( … )` and `linked( … )` the same rule holds with the term in
//! the root's place: the group's conjuncts are `<term>.<n>`, a group
//! that is one condition — an or and a not among them — holds it at
//! `<term>.0`, and below that a child of an and or an or is
//! `<parent>.<n>` and what a not negates `<parent>.0`. The root has no
//! path, and the conditions inside a `sum( … )` or a projection take
//! none. The substitution rebuilds the tree around the node as it was:
//! nothing reordered, flattened, merged or simplified (invariant 1), so
//! `line(true() arg1>=90)` and `(a false()) b` are what it prints. The
//! binder numbers its terms by the same two functions (`bind.rs`).

use crate::error::{ErrorKind, LanguageError};
use crate::group::Slots;
use crate::{print, template};

/// A number of the language. A whole value is always [`Number::Int`] —
/// `90.0` is `90` — so a number has one spelling and one tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    /// The one constructor that keeps a whole value whole: every whole
    /// value an i64 holds (from -2^63 up to, not including, 2^63) is an
    /// `Int`, which is exactly what the parser reads such digits as.
    pub fn from_f64(f: f64) -> Number {
        const TWO_63: f64 = 9_223_372_036_854_775_808.0;
        if f.fract() == 0.0 && (-TWO_63..TWO_63).contains(&f) {
            Number::Int(f as i64)
        } else {
            Number::Float(f)
        }
    }

    pub fn as_f64(self) -> f64 {
        match self {
            Number::Int(i) => i as f64,
            Number::Float(f) => f,
        }
    }
}

/// The right-hand side of a test.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Text(String),
    Number(Number),
    /// `a..b`, inclusive; a side may be blank, never both.
    Range {
        from: Option<Number>,
        to: Option<Number>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `:` — contains, or picks among a closed set's values.
    Contains,
    /// `=` — the whole value; the only operator a range takes.
    Eq,
    /// `~` — a pattern.
    Match,
    Gt,
    Ge,
    Lt,
    Le,
}

impl Op {
    pub const ALL: [Op; 7] = [
        Op::Contains,
        Op::Eq,
        Op::Match,
        Op::Gt,
        Op::Ge,
        Op::Lt,
        Op::Le,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Op::Contains => ":",
            Op::Eq => "=",
            Op::Match => "~",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::Lt => "<",
            Op::Le => "<=",
        }
    }

    pub fn parse(s: &str) -> Option<Op> {
        Op::ALL.into_iter().find(|op| op.as_str() == s)
    }

    pub fn is_ordering(self) -> bool {
        matches!(self, Op::Gt | Op::Ge | Op::Lt | Op::Le)
    }
}

/// The collection a member group ranges over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collection {
    /// `line( … )` — one displayed occurrence.
    Lines,
    /// `linked( … )` — one link group.
    Links,
}

impl Collection {
    pub fn call(self) -> &'static str {
        match self {
            Collection::Lines => "line",
            Collection::Links => "linked",
        }
    }

    pub fn json(self) -> &'static str {
        match self {
            Collection::Lines => "lines",
            Collection::Links => "links",
        }
    }
}

/// A node of the item-level tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// And. Two or more children; none only at the root, the empty query.
    All(Vec<Node>),
    /// Or. Two or more children.
    Any(Vec<Node>),
    Not(Box<Node>),
    /// At-least-N-of, with a lower and an upper bound (C91); one at least.
    Holds {
        of: Vec<Node>,
        min: Option<u32>,
        max: Option<u32>,
    },
    Undecided(Probe),
    /// `true()` / `false()` — what `--explain` prints for a forced node.
    Const(bool),
    /// `field op value`; a quoted phrase is `text:` with its phrase.
    Test {
        field: String,
        op: Op,
        value: Value,
    },
    Has(String),
    Is(String),
    /// Conditions that hold together on one member (C92).
    Members {
        of: Collection,
        where_: Box<Member>,
    },
    /// A comparison on a computed value: a `pseudo.` name or a `sum`.
    Compare {
        value: ValueRef,
        op: Op,
        rhs: Value,
    },
}

/// What `undecided( … )` asks about: a thing's value, or a term's truth.
#[derive(Debug, Clone, PartialEq)]
pub enum Probe {
    Thing(ValueRef),
    Term(Box<Node>),
}

/// A node inside a member group.
#[derive(Debug, Clone, PartialEq)]
pub enum Member {
    All(Vec<Member>),
    Any(Vec<Member>),
    Not(Box<Member>),
    Const(bool),
    Test { attr: String, op: Op, value: Value },
    Is(String),
}

/// Something with a value: what a comparison, a sum or a sort consumes.
#[derive(Debug, Clone, PartialEq)]
pub enum ValueRef {
    Field(String),
    Pseudo {
        name: String,
        slot: Option<String>,
    },
    Sum {
        lines: Box<Member>,
        slot: String,
    },
    /// `line(P).<slot>`. A comparison on one lowers into the group, so in
    /// a query tree it appears only under `undecided( … )`; `--sort` and
    /// `--sum` take one directly.
    Projection {
        lines: Box<Member>,
        slot: String,
    },
}

// ---- names ---------------------------------------------------------------

/// Words the grammar keeps: a name is never one of them.
pub(crate) const KEYWORDS: [&str; 3] = ["and", "or", "not"];

pub(crate) fn is_keyword(word: &str) -> bool {
    KEYWORDS.iter().any(|k| k.eq_ignore_ascii_case(word))
}

pub(crate) fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// One segment of a name: letters, digits and underscores, never starting
/// with a digit; a hyphen is never part of a name.
pub(crate) fn is_segment(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(is_word_char)
        && !is_keyword(s)
}

/// A name, possibly dotted (`sockets.red`, `price.amount`).
pub(crate) fn is_name(s: &str) -> bool {
    !s.is_empty() && s.split('.').all(is_segment)
}

/// The words that name a number (the reference, *Slots*).
/// Whether a name is the computed namespace's — `pseudo`, or `pseudo.…`
/// — in any case, as every name is read (F10, the first seat: `has:Pseudo.DPS`
/// offered no reading).
pub(crate) fn is_computed(name: &str) -> bool {
    name.get(..6)
        .is_some_and(|head| head.eq_ignore_ascii_case("pseudo"))
        && name
            .get(6..)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with('.'))
}

pub fn is_slot_word(s: &str) -> bool {
    matches!(s, "low" | "high" | "avg") || arg_index(s).is_some()
}

/// `arg<N>`, counting from 1.
pub(crate) fn arg_index(s: &str) -> Option<usize> {
    let digits = s.strip_prefix("arg")?;
    if digits.is_empty() || digits.starts_with('0') || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

// ---- paths -----------------------------------------------------------------

/// The path of the `n`th child of the node at `parent`: of an and, an or
/// or a `holds`, and of a member group's conjuncts under their term. The
/// children of the root carry no prefix.
pub(crate) fn child_path(parent: &str, n: usize) -> String {
    if parent.is_empty() {
        n.to_string()
    } else {
        format!("{parent}.{n}")
    }
}

/// The path of the one node under the node at `parent`: what a not
/// negates, what `undecided( … )` asks about, a member group's one
/// condition — and, under no parent, a root that is no and.
pub(crate) fn sole_path(parent: &str) -> String {
    child_path(parent, 0)
}

/// A node of a query a path names (the reference, *Explain*).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pathed {
    pub path: String,
    /// The node, as the canonical text prints it.
    pub text: String,
    /// Whether it is a condition inside `line( … )` or `linked( … )`,
    /// which holds of one member and has no outcome on an item.
    pub inside: bool,
}

/// What [`walk`] shows its visitor: a node of the item's tree, or a
/// condition inside a member group.
enum At<'a> {
    Node(&'a Node),
    Inside(Collection, &'a Member),
}

/// The one walk that numbers a query's nodes (the module doc of
/// `bind.rs`, "Atomic terms are numbered by path", whose rule this is):
/// every node with a path is shown to `visit` before its children, and
/// one `visit` answers with a constant is replaced by it — that node and
/// no other, the tree around it rebuilt as it was (invariant 1 of the
/// surface). The root has no path; the conditions inside a `sum( … )`
/// and a projection take none.
fn walk(root: &Node, visit: &mut dyn FnMut(&str, At<'_>) -> Option<bool>) -> Node {
    match root {
        Node::All(children) => Node::All(
            children
                .iter()
                .enumerate()
                .map(|(n, child)| walk_node(child, &child_path("", n), visit))
                .collect(),
        ),
        other => walk_node(other, &sole_path(""), visit),
    }
}

fn walk_node(node: &Node, path: &str, visit: &mut dyn FnMut(&str, At<'_>) -> Option<bool>) -> Node {
    if let Some(value) = visit(path, At::Node(node)) {
        return Node::Const(value);
    }
    let mut each = |children: &[Node]| -> Vec<Node> {
        children
            .iter()
            .enumerate()
            .map(|(n, child)| walk_node(child, &child_path(path, n), visit))
            .collect()
    };
    match node {
        Node::All(children) => Node::All(each(children)),
        Node::Any(children) => Node::Any(each(children)),
        Node::Holds { of, min, max } => Node::Holds {
            of: each(of),
            min: *min,
            max: *max,
        },
        Node::Not(inner) => Node::Not(Box::new(walk_node(inner, &sole_path(path), visit))),
        Node::Undecided(Probe::Term(inner)) => Node::Undecided(Probe::Term(Box::new(walk_node(
            inner,
            &sole_path(path),
            visit,
        )))),
        Node::Members { of, where_ } => Node::Members {
            of: *of,
            // the group's conjuncts are numbered under their term, as the
            // root's children are under no prefix; a group of one
            // condition holds that one
            where_: Box::new(match where_.as_ref() {
                Member::All(children) => Member::All(
                    children
                        .iter()
                        .enumerate()
                        .map(|(n, child)| walk_member(*of, child, &child_path(path, n), visit))
                        .collect(),
                ),
                other => walk_member(*of, other, &sole_path(path), visit),
            }),
        },
        Node::Undecided(Probe::Thing(_))
        | Node::Const(_)
        | Node::Test { .. }
        | Node::Has(_)
        | Node::Is(_)
        | Node::Compare { .. } => node.clone(),
    }
}

fn walk_member(
    of: Collection,
    member: &Member,
    path: &str,
    visit: &mut dyn FnMut(&str, At<'_>) -> Option<bool>,
) -> Member {
    if let Some(value) = visit(path, At::Inside(of, member)) {
        return Member::Const(value);
    }
    let mut each = |children: &[Member]| -> Vec<Member> {
        children
            .iter()
            .enumerate()
            .map(|(n, child)| walk_member(of, child, &child_path(path, n), visit))
            .collect()
    };
    match member {
        Member::All(children) => Member::All(each(children)),
        Member::Any(children) => Member::Any(each(children)),
        Member::Not(inner) => {
            Member::Not(Box::new(walk_member(of, inner, &sole_path(path), visit)))
        }
        Member::Const(_) | Member::Test { .. } | Member::Is(_) => member.clone(),
    }
}

/// Every node of the query that has a path, a parent before its
/// children: what `--explain` takes, and what its refusal lists.
pub fn nodes(root: &Node) -> Vec<Pathed> {
    let mut out = Vec::new();
    walk(root, &mut |path, at| {
        out.push(match at {
            At::Node(node) => Pathed {
                path: path.to_string(),
                text: print::print(node),
                inside: false,
            },
            At::Inside(of, member) => Pathed {
                path: path.to_string(),
                text: print::print_group(of, member),
                inside: true,
            },
        });
        None
    });
    out
}

/// The query with the node at `path` replaced by `true()` or `false()`
/// and nothing else changed, or none where the query has no such node.
pub fn forced(root: &Node, path: &str, value: bool) -> Option<Node> {
    let mut found = false;
    let out = walk(root, &mut |at, _| {
        (at == path).then(|| {
            found = true;
            value
        })
    });
    found.then_some(out)
}

// ---- validity --------------------------------------------------------------

fn invalid(kind: ErrorKind, message: impl Into<String>) -> LanguageError {
    LanguageError::new(kind, message)
}

/// Whether this tree is one the language can say: what the printer prints
/// and the parser gives back. The parser's trees pass by construction; a
/// tree that arrived as JSON is checked here before anything reads it.
pub fn check(root: &Node) -> Result<(), LanguageError> {
    match root {
        Node::All(children) if children.is_empty() => Ok(()),
        node => check_node(node),
    }
}

fn check_group_size(n: usize, what: &str) -> Result<(), LanguageError> {
    if n < 2 {
        return Err(invalid(
            ErrorKind::Tree,
            format!(
                "`{what}` holds {n} of its two or more members: a group of one is its member, and only the root may be empty"
            ),
        ));
    }
    Ok(())
}

fn check_node(node: &Node) -> Result<(), LanguageError> {
    match node {
        Node::All(children) => {
            check_group_size(children.len(), "all")?;
            children.iter().try_for_each(check_node)
        }
        Node::Any(children) => {
            check_group_size(children.len(), "any")?;
            children.iter().try_for_each(check_node)
        }
        Node::Not(inner) => check_node(inner),
        Node::Holds { of, min, max } => {
            if of.is_empty() {
                return Err(invalid(ErrorKind::Tree, "`holds` counts no query"));
            }
            match (min, max) {
                (None, None) => {
                    return Err(invalid(
                        ErrorKind::HoldsNeedsBound,
                        "`holds` needs how many: a lower bound, an upper bound or both",
                    ));
                }
                (Some(a), Some(b)) if a > b => {
                    return Err(invalid(
                        ErrorKind::Tree,
                        format!("`holds` from {a} to {b} is an empty range"),
                    ));
                }
                _ => {}
            }
            of.iter().try_for_each(check_node)
        }
        Node::Undecided(Probe::Thing(value)) => check_value_ref(value, &|asked| {
            print::print(&Node::Undecided(Probe::Thing(asked.clone())))
        }),
        Node::Undecided(Probe::Term(term)) => check_node(term),
        Node::Const(_) => Ok(()),
        Node::Test { field, op, value } => {
            check_field(field)?;
            if field == "text" && *op == Op::Contains && *value == Value::Text(String::new()) {
                return Err(invalid(ErrorKind::Tree, "an empty phrase finds nothing"));
            }
            check_test(*op, value)
        }
        Node::Has(name) => check_plain_name(name),
        Node::Is(name) => check_plain_name(name),
        Node::Members { of, where_ } => check_members(*of, where_, None, &|asked| {
            print::print(&Node::Members {
                of: *of,
                where_: Box::new(asked.clone()),
            })
        }),
        Node::Compare { value, op, rhs } => {
            match value {
                ValueRef::Pseudo { .. } | ValueRef::Sum { .. } => {}
                ValueRef::Field(_) => {
                    return Err(invalid(
                        ErrorKind::Tree,
                        "a comparison on a field is a `field` test, never a `value` one",
                    ));
                }
                ValueRef::Projection { .. } => {
                    return Err(invalid(
                        ErrorKind::Tree,
                        "a comparison on `line(P).<slot>` lowers into the group: `line(P <slot> …)`",
                    ));
                }
            }
            check_value_ref(value, &|asked| {
                print::print(&Node::Compare {
                    value: asked.clone(),
                    op: *op,
                    rhs: rhs.clone(),
                })
            })?;
            match rhs {
                Value::Text(_) => Err(invalid(
                    ErrorKind::ComparisonNeedsNumber,
                    "a computed value compares with a number",
                )),
                _ if *op == Op::Contains || *op == Op::Match => Err(invalid(
                    ErrorKind::ComparisonNeedsNumber,
                    "a computed value takes = > >= < <=",
                )),
                _ => check_test(*op, rhs),
            }
        }
    }
}

fn check_plain_name(name: &str) -> Result<(), LanguageError> {
    if is_name(name) {
        Ok(())
    } else {
        Err(invalid(
            ErrorKind::Tree,
            format!("`{name}` is not a name: letters, digits and underscores, dotted"),
        ))
    }
}

fn check_field(field: &str) -> Result<(), LanguageError> {
    check_plain_name(field)?;
    if field == "realm" {
        return Err(realm_is_scope());
    }
    if field == "has" || field == "is" {
        return Err(invalid(
            ErrorKind::Tree,
            format!("`{field}:` is its own node, never a field"),
        ));
    }
    if is_computed(field) {
        return Err(invalid(
            ErrorKind::Tree,
            "a `pseudo.` name is a computed value: a `value` comparison, never a `field` test",
        ));
    }
    if is_slot_word(field) {
        return Err(slot_outside_group(field));
    }
    Ok(())
}

pub(crate) fn realm_is_scope() -> LanguageError {
    invalid(
        ErrorKind::RealmIsScope,
        "the realm is the search's scope, never a term (C96): name it outside the query — pc, xbox, sony, poe2, or all",
    )
}

pub(crate) fn slot_outside_group(word: &str) -> LanguageError {
    invalid(
        ErrorKind::SlotOutsideGroup,
        format!(
            "`{word}` names a number of one line, so it lives inside that line's group: line(\"…\" {word}>=…)"
        ),
    )
}

fn check_test(op: Op, value: &Value) -> Result<(), LanguageError> {
    match value {
        Value::Range { from, to } => {
            if op != Op::Eq {
                return Err(invalid(
                    ErrorKind::RangeNeedsEquals,
                    "a range `a..b` takes `=`",
                ));
            }
            match (from, to) {
                (None, None) => Err(invalid(ErrorKind::Tree, "a range needs one side at least")),
                (Some(a), Some(b)) if a.as_f64() > b.as_f64() => {
                    Err(invalid(ErrorKind::Tree, "a range runs from low to high"))
                }
                _ => check_numbers(from.iter().chain(to.iter())),
            }
        }
        Value::Number(n) => check_numbers(std::iter::once(n)),
        Value::Text(_) if op.is_ordering() => Err(invalid(
            ErrorKind::ComparisonNeedsNumber,
            "> >= < <= compare with a number",
        )),
        Value::Text(_) => Ok(()),
    }
}

fn check_numbers<'a>(mut numbers: impl Iterator<Item = &'a Number>) -> Result<(), LanguageError> {
    let canonical = |n: &Number| match n {
        Number::Int(_) => true,
        Number::Float(f) => f.is_finite() && Number::from_f64(*f) == *n,
    };
    if numbers.all(canonical) {
        Ok(())
    } else {
        Err(invalid(
            ErrorKind::Tree,
            "a number is finite, and a whole one is written whole",
        ))
    }
}

/// A value alone: what `--sort` and `--sum` take, a reading printed as
/// the value it is.
pub(crate) fn check_value(value: &ValueRef) -> Result<(), LanguageError> {
    check_value_ref(value, &print::print_value)
}

fn check_value_ref(
    value: &ValueRef,
    shown: &dyn Fn(&ValueRef) -> String,
) -> Result<(), LanguageError> {
    match value {
        ValueRef::Field(name) => check_field(name),
        ValueRef::Pseudo { name, slot } => {
            if !is_segment(name) {
                return Err(invalid(
                    ErrorKind::Tree,
                    format!(
                        "`{name}` is not a computed value's name: one word of letters, digits and underscores"
                    ),
                ));
            }
            match slot {
                Some(slot) if !is_slot_word(slot) => Err(invalid(
                    ErrorKind::Tree,
                    format!("`{slot}` is not a slot word: low, high, avg, arg1, arg2 …"),
                )),
                _ => Ok(()),
            }
        }
        ValueRef::Sum { lines, slot } => {
            check_members(Collection::Lines, lines, Some(slot), &|asked| {
                shown(&ValueRef::Sum {
                    lines: Box::new(asked.clone()),
                    slot: slot.clone(),
                })
            })
        }
        ValueRef::Projection { lines, slot } => {
            check_members(Collection::Lines, lines, Some(slot), &|asked| {
                shown(&ValueRef::Projection {
                    lines: Box::new(asked.clone()),
                    slot: slot.clone(),
                })
            })
        }
    }
}

/// A member group, and what its slot words read (the reference, *Slots*;
/// `group::Slots`): a slot beside two rows of a mod named together is
/// refused, each reading that the check passes offered as `shown`
/// prints it; and where the group selects one quoted template, every
/// slot that template alone says the numbers of is checked against the
/// template's own.
fn check_members(
    of: Collection,
    where_: &Member,
    projected: Option<&str>,
    shown: &dyn Fn(&Member) -> String,
) -> Result<(), LanguageError> {
    let (slot, one, other) = match members_checked(of, where_, projected) {
        Ok(()) => return Ok(()),
        Err(Refused::Error(e)) => return Err(e),
        Err(Refused::TwoRows { slot, one, other }) => (slot, one, other),
    };
    let readings = [other, one]
        .into_iter()
        .map(|asked| asked_of_the_text(where_, asked))
        .filter(|reading| members_checked(of, reading, projected).is_ok())
        .map(|reading| shown(&reading))
        .collect();
    Err(invalid(
        ErrorKind::SlotOfTwoRows,
        format!(
            "`{slot}` names a number of one row, and {} and {} name two rows together: quote the row whose number is meant, and ask the other of the mod's text with template:",
            print::quoted_text(one),
            print::quoted_text(other)
        ),
    )
    .with_readings(readings))
}

/// Why a member group is refused: a slot beside two rows named together
/// apart, its readings being made once and by the group that was asked.
enum Refused<'a> {
    Error(LanguageError),
    TwoRows {
        slot: &'a str,
        one: &'a str,
        other: &'a str,
    },
}

impl From<LanguageError> for Refused<'_> {
    fn from(e: LanguageError) -> Self {
        Refused::Error(e)
    }
}

fn members_checked<'a>(
    of: Collection,
    where_: &'a Member,
    projected: Option<&'a str>,
) -> Result<(), Refused<'a>> {
    check_member(where_)?;
    if let Some(slot) = projected
        && !is_slot_word(slot)
    {
        return Err(invalid(
            ErrorKind::Tree,
            format!("`{slot}` is not a slot word: low, high, avg, arg1, arg2 …"),
        )
        .into());
    }
    if of != Collection::Lines {
        return Ok(());
    }
    let quoted = template::selected(where_);
    if let Some(quoted) = quoted {
        template::check_typed(quoted)?;
    }
    let slots = Slots::of(where_, projected);
    if let Some((slot, one, other)) = slots.of_two_rows() {
        return Err(Refused::TwoRows { slot, one, other });
    }
    let Some(quoted) = quoted else {
        return Ok(());
    };
    for slot in slots.of_alone(quoted) {
        template::check_slot(quoted, slot)?;
    }
    Ok(())
}

/// The group with one quoted template asked of the mod's text instead,
/// wherever it sits: `template:` tests the whole text and names no row.
fn asked_of_the_text(member: &Member, quoted: &str) -> Member {
    let each = |children: &[Member]| {
        children
            .iter()
            .map(|c| asked_of_the_text(c, quoted))
            .collect()
    };
    match member {
        Member::All(children) => Member::All(each(children)),
        Member::Any(children) => Member::Any(each(children)),
        Member::Not(inner) => Member::Not(Box::new(asked_of_the_text(inner, quoted))),
        Member::Test {
            attr,
            op: Op::Eq,
            value: Value::Text(template),
        } if attr == "template" && template.to_lowercase() == quoted.to_lowercase() => {
            Member::Test {
                attr: attr.clone(),
                op: Op::Contains,
                value: Value::Text(template.clone()),
            }
        }
        other => other.clone(),
    }
}

fn check_member(member: &Member) -> Result<(), LanguageError> {
    match member {
        Member::All(children) => {
            check_group_size(children.len(), "all")?;
            children.iter().try_for_each(check_member)
        }
        Member::Any(children) => {
            check_group_size(children.len(), "any")?;
            children.iter().try_for_each(check_member)
        }
        Member::Not(inner) => check_member(inner),
        Member::Const(_) => Ok(()),
        Member::Is(name) => check_plain_name(name),
        Member::Test { attr, op, value } => {
            check_plain_name(attr)?;
            if let (true, Op::Eq, Value::Text(template)) = (attr == "template", op, value) {
                template::check_unsigned(template)?;
            }
            if attr == "has" || attr == "is" {
                return Err(invalid(
                    ErrorKind::Tree,
                    format!("`{attr}` is not a member's attribute"),
                ));
            }
            check_test(*op, value)
        }
    }
}
