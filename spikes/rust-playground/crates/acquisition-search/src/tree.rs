//! The query tree (C91): one typed value, immutable and serializable, that
//! the text and the JSON form each print to and parse from (C104).
//!
//! A tree is always *lowered*: the shorthand of the text (`"T">=90`,
//! `line(P).avg>=20`, `sum("T")`) never appears in it. [`check`] states
//! what a tree must satisfy to be one the printer can print and the
//! parser would give back; the parser's output satisfies it by
//! construction, and a tree arriving as JSON is checked before use.

use crate::error::{ErrorKind, LanguageError};
use crate::template;

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
        Node::Undecided(Probe::Thing(value)) => check_value_ref(value),
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
        Node::Members { of, where_ } => check_members(*of, where_, None),
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
            check_value_ref(value)?;
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

/// `has:` on a total (T2, `SEARCH-SLICE.md`, "Holes ruled"): every item
/// has one, so the binder refuses it with the readings that ask what was
/// meant. A derived field takes `has:` (`bind.rs`).
pub(crate) fn has_on_computed(name: &str) -> LanguageError {
    invalid(
        ErrorKind::HasOnComputed,
        format!("`has:` does not apply to a total: `{name}` always has a status, never a presence"),
    )
    .with_readings(vec![format!("{name}>0"), format!("undecided({name})")])
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
    if field == "pseudo" || field.starts_with("pseudo.") {
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

fn check_value_ref(value: &ValueRef) -> Result<(), LanguageError> {
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
        ValueRef::Sum { lines, slot } | ValueRef::Projection { lines, slot } => {
            check_members(Collection::Lines, lines, Some(slot))
        }
    }
}

/// A member group, and — when it selects one quoted template — every slot
/// it names against that template's own numbers.
fn check_members(
    of: Collection,
    where_: &Member,
    projected: Option<&str>,
) -> Result<(), LanguageError> {
    check_member(where_)?;
    if let Some(slot) = projected
        && !is_slot_word(slot)
    {
        return Err(invalid(
            ErrorKind::Tree,
            format!("`{slot}` is not a slot word: low, high, avg, arg1, arg2 …"),
        ));
    }
    if of != Collection::Lines {
        return Ok(());
    }
    let Some(quoted) = template::selected(where_) else {
        return Ok(());
    };
    template::check_typed(quoted)?;
    let mut slots = Vec::new();
    collect_slots(where_, &mut slots);
    slots.extend(projected);
    slots
        .into_iter()
        .try_for_each(|slot| template::check_slot(quoted, slot))
}

fn collect_slots<'a>(member: &'a Member, out: &mut Vec<&'a str>) {
    match member {
        Member::All(children) | Member::Any(children) => {
            children.iter().for_each(|c| collect_slots(c, out));
        }
        Member::Not(inner) => collect_slots(inner, out),
        Member::Test { attr, .. } if is_slot_word(attr) => out.push(attr),
        Member::Test { .. } | Member::Const(_) | Member::Is(_) => {}
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
