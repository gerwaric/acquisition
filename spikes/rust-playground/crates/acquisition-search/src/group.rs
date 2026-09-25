//! A line's group, bound (C92, C93): what `line( … )` means, computed once
//! when the query is bound and read from here by every consumer — the
//! evaluator, the answer's routes, what a selector resolved to, the zero
//! block, the sort.
//!
//! The words are `search/DESIGN.md`'s reference (*Members*, *Slots*); the
//! rules that decide what a group means are this doc's and `eval.rs`'s
//! (C92, C93; taken from the contract detail 2026-09-23).
//!
//! # Decisions as recorded
//!
//! **A group's meaning has one reader (the build plan, step 4b).** Four
//! outside audits of the first surface found one fault five times: what a
//! group means was read again, off its syntax, by whichever function
//! needed it next — which sources it admits, its selector, its together
//! bound, the slot check, the zero block — and parentheses changed the
//! answer (invariant 7 of the surface). So outside the modules that build,
//! print and validate the tree (`parse`, `print`, `json`, `tree`, with
//! `template`'s checks) and this one, nothing names a group's tree, bound
//! or not — names, so that an alias is no way round: `tools/docs-check.sh`
//! refuses it, and `tools/docs-check-breakers.sh` proves it refuses. A
//! consumer that needs
//! a new fact of a group adds it here, beside the others, where
//! `tests/generated_equivalence.rs` asks it of every spelling.
//!
//! # As built
//!
//! - **The whole** is the group as authored, asked of one occurrence, and
//!   three-valued as the item's tree is on an item: a flag the line could
//!   not read is unknown, never a no (C93), so `-is:crafted` is undecided
//!   on a line whose flags are unread.
//! - **The selector** is the group with its slot comparisons taken as
//!   favourably as they can be and folded away: what the group picks before
//!   any number is compared, by meaning and never by where a comparison
//!   sits. It is what tells *lacked* (no occurrence is selected) from
//!   *failed* (one is, and none satisfies the whole), what the answer says
//!   a `:` or `~` template selector resolved to, and what the together
//!   count sums (C92). Whether it asks a flag decides the failed route's
//!   shape: an occurrence with unread flags leaves such a selector open.
//! - **Which sources it admits** is the whole asked with its source tests
//!   answered and everything else unknown: `(source=explicit "T") arg1>=90`
//!   rules the implicit array out exactly as `source=explicit "T" arg1>=90`
//!   does. Answered at binding for every source the language names, and
//!   once for any other — a source GGG adds is one no `source=` test names.
//! - **The together bound** is the one slot comparison among the group's
//!   conjuncts (`template::conjuncts`, the slot check's own reading), when
//!   it is a lower bound and the selector's sum of that slot is a value
//!   this build runs; anything else is not applicable.
//! - **Its template tests**, wherever they sit: whether it makes any, so
//!   that it resolves to templates; whether each is a quoted `"T"`, which
//!   resolves to itself; and the words a suggestion is scored against.
//! - **A link group's** (`linked( … )`, C101; the build plan, step 8) is
//!   [`Linked`]: the same one reader, over the attributes a link group
//!   has — `red green blue white`, the colour counts, and `size` — each a
//!   comparison, composed with and, or and not as a line's group is, and
//!   three-valued on one group of the item as `sockets.rs` counts it: a
//!   count the group's unread sockets leave open leaves the comparison
//!   undecided, never a no. Every comparison is favoured by more sockets
//!   or fewer, so a group holds a selector of nothing — every group is a
//!   candidate — and the group's members are the item's link groups, which
//!   is why an item with none *lacks* a `linked( … )` and `-has:links`
//!   routes it (`answer.rs`).

use crate::bind::{self, LINE_FLAGS, NumTest, SOURCES, TextTest};
use crate::derive::{ITEM_FLAGS, Line, Slot};
use crate::error::{ErrorKind, LanguageError};
use crate::sockets::{self, GroupCounts};
use crate::tree::{self, Collection, Member, Node, Number, Op, Probe, Value, ValueRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Truth {
    True,
    False,
    Undecided,
}

pub(crate) fn all_of(each: impl Iterator<Item = Truth>) -> Truth {
    let each: Vec<Truth> = each.collect();
    if each.contains(&Truth::False) {
        Truth::False
    } else if each.contains(&Truth::Undecided) {
        Truth::Undecided
    } else {
        Truth::True
    }
}

pub(crate) fn any_of(each: impl Iterator<Item = Truth>) -> Truth {
    let each: Vec<Truth> = each.collect();
    if each.contains(&Truth::True) {
        Truth::True
    } else if each.contains(&Truth::Undecided) {
        Truth::Undecided
    } else {
        Truth::False
    }
}

pub(crate) fn negated(truth: Truth) -> Truth {
    match truth {
        Truth::True => Truth::False,
        Truth::False => Truth::True,
        Truth::Undecided => Truth::Undecided,
    }
}

fn sure(value: bool) -> Truth {
    if value { Truth::True } else { Truth::False }
}

/// A node inside a line's group, bound.
#[derive(Debug, Clone)]
enum BMember {
    All(Vec<BMember>),
    Any(Vec<BMember>),
    Not(Box<BMember>),
    Const(bool),
    Template(TextTest),
    Source(Vec<&'static str>),
    Slot { word: String, test: NumTest },
    Is(&'static str),
}

/// Something a group asks of one occurrence: the whole, or the selector.
#[derive(Debug, Clone)]
pub(crate) struct Asked(BMember);

impl Asked {
    /// Three-valued (the module doc, "The whole").
    pub fn of(&self, line: &Line) -> Truth {
        truth(&self.0, line)
    }

    pub fn holds(&self, line: &Line) -> bool {
        self.of(line) == Truth::True
    }
}

fn truth(member: &BMember, line: &Line) -> Truth {
    match member {
        BMember::All(children) => all_of(children.iter().map(|c| truth(c, line))),
        BMember::Any(children) => any_of(children.iter().map(|c| truth(c, line))),
        BMember::Not(inner) => negated(truth(inner, line)),
        BMember::Const(value) => sure(*value),
        BMember::Template(test) => sure(test.holds(&line.template)),
        BMember::Source(sources) => sure(sources.contains(&line.source.as_str())),
        // a number that could not be read is unknown, as a flag is: never
        // a no, which a not would turn into a witness
        BMember::Slot { word, test } => match line.slot(word) {
            Slot::Is(n) => sure(test.holds(n)),
            Slot::Absent => Truth::False,
            Slot::Unread => Truth::Undecided,
        },
        // GGG's spelling in any case, as the word was bound (B2): the
        // vocabulary counts a flag by its legal spelling and routes by it,
        // and the two must be one compare (outside audit, 2026-09-22)
        BMember::Is(flag) if line.flags.iter().any(|f| f.eq_ignore_ascii_case(flag)) => Truth::True,
        BMember::Is(flag) => {
            if line.flags_unread
                || line
                    .flags_unknown
                    .iter()
                    .any(|f| f.eq_ignore_ascii_case(flag))
            {
                Truth::Undecided
            } else {
                Truth::False
            }
        }
    }
}

/// A lower bound on one slot of a line selector: what the together count
/// is defined for (C92). Anything else is not applicable.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LowerBound {
    pub slot: String,
    pub op: Op,
    pub bound: Number,
}

/// `line( … )`, bound (the module doc).
#[derive(Debug, Clone)]
pub(crate) struct Group {
    pub whole: Asked,
    pub selector: Asked,
    /// The selector as a tree, for the routes that name it.
    pub selector_tree: Member,
    /// True when the group compares no slot: the selector is the whole.
    pub selects_only: bool,
    /// Whether the selector asks a flag of the occurrence.
    pub selector_asks_a_flag: bool,
    /// Whether the whole does: what an occurrence's unread flags can leave
    /// open, as `selects_only` says of its unread numbers.
    pub asks_a_flag: bool,
    pub together: Option<LowerBound>,
    /// The sources of [`SOURCES`] an occurrence satisfying the whole may
    /// come from, and whether one from a source outside that list may.
    admitted: Vec<&'static str>,
    admits_another: bool,
    /// The group's template tests, wherever they sit.
    templates: Vec<(Op, String)>,
}

impl Group {
    pub fn bind(whole: &Member) -> Result<Group, LanguageError> {
        let selector_tree = selector(whole);
        let bound = member(whole)?;
        let mut all = Vec::new();
        crate::template::conjuncts(whole, &mut all);
        let slotted: Vec<&Member> = all.into_iter().filter(|c| has_slot(c)).collect();
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
        // a route is a query this build runs (the plan's rule 5): the together
        // route sums the selector's slot, and a selector folding has reshaped
        // may name a template that slot is not one of. Where the checker would
        // refuse that sum the count is not applicable, never a refused route.
        let together = together.filter(|lower| {
            tree::check(&Node::Undecided(Probe::Thing(ValueRef::Sum {
                lines: Box::new(selector_tree.clone()),
                slot: lower.slot.clone(),
            })))
            .is_ok()
        });
        let mut templates = Vec::new();
        template_tests(whole, &mut templates);
        Ok(Group {
            selector: Asked(member(&selector_tree)?),
            selects_only: !has_slot(whole),
            selector_asks_a_flag: asks_a_flag(&selector_tree),
            asks_a_flag: asks_a_flag(whole),
            admitted: SOURCES
                .iter()
                .copied()
                .filter(|source| admits(&bound, source))
                .collect(),
            admits_another: admits(&bound, ""),
            whole: Asked(bound),
            selector_tree,
            together,
            templates,
        })
    }

    /// Whether an occurrence from `source` could satisfy the whole (the
    /// module doc): what stops the group claiming an absence while that
    /// source is unread.
    pub fn admits(&self, source: &str) -> bool {
        if SOURCES.contains(&source) {
            self.admitted.contains(&source)
        } else {
            self.admits_another
        }
    }

    /// Whether the group tests a template at all: what it resolved to is
    /// then worth saying (invariant 2).
    pub fn names_a_template(&self) -> bool {
        !self.templates.is_empty()
    }

    /// Whether every template test is a quoted `"T"`, which resolves to
    /// itself — unless it found two spellings, which any-case `=` can.
    pub fn quoted_only(&self) -> bool {
        self.templates.iter().all(|(op, _)| *op == Op::Eq)
    }

    /// The one template test the selector is, when it is nothing else:
    /// then what the group resolved to is exactly what the vocabulary
    /// narrowed by that test lists, and the answer can route to the rest
    /// of it (outside audit, 2026-09-22).
    pub fn sole_template_test(&self) -> Option<(Op, String)> {
        match &self.selector_tree {
            Member::Test {
                attr,
                op,
                value: Value::Text(text),
            } if attr == "template" => Some((*op, text.clone())),
            _ => None,
        }
    }

    /// The words of its `:` and `=` template tests: what a suggestion is
    /// scored against when the selector picked nothing. A pattern has none.
    pub fn wanted(&self) -> Option<String> {
        let wanted: Vec<&str> = self
            .templates
            .iter()
            .filter(|(op, _)| *op != Op::Match)
            .map(|(_, text)| text.as_str())
            .collect();
        (!wanted.is_empty()).then(|| wanted.join(" "))
    }
}

/// The exact term that selects one template: `line("T")`.
pub(crate) fn line_of(template: &str) -> Node {
    line_exactly(template, false, LineKind::Any)
}

/// A line's kind, as far as one test names it (C90: both coordinates the
/// body gives, neither chosen for the author).
#[derive(Debug, Clone, Copy)]
pub(crate) enum LineKind<'a> {
    Any,
    Source(&'a str),
    Flag(&'a str),
}

/// The exact term that selects one template's occurrences of one kind, as
/// a vocabulary row carries it (C97). `=` is any-case, so where the corpus
/// holds another spelling of the template — `twin` — the term is a pattern
/// that turns case back on, which selects this spelling alone (the
/// reference, *Members*).
pub(crate) fn line_exactly(template: &str, twin: bool, kind: LineKind<'_>) -> Node {
    let test = |attr: &str, op: Op, text: String| Member::Test {
        attr: attr.to_string(),
        op,
        value: Value::Text(text),
    };
    let named = if twin {
        test("template", Op::Match, bind::exact_pattern(template))
    } else {
        test("template", Op::Eq, template.to_string())
    };
    let where_ = match kind {
        LineKind::Any => named,
        LineKind::Source(source) => {
            Member::All(vec![named, test("source", Op::Eq, source.to_string())])
        }
        LineKind::Flag(flag) => Member::All(vec![named, Member::Is(flag.to_string())]),
    };
    Node::Members {
        of: Collection::Lines,
        where_: Box::new(where_),
    }
}

/// A total's row as the group it is (`totals.rs`, C94): the template,
/// exact, and the source and the flag where the row names them — bound
/// here, so that what a row admits, selects and says of one occurrence is
/// read as every other group's is, and never worked out again.
pub(crate) fn row(
    template: &str,
    source: Option<&str>,
    flag: Option<&str>,
) -> Result<(Node, Group), LanguageError> {
    let test = |attr: &str, text: &str| Member::Test {
        attr: attr.to_string(),
        op: Op::Eq,
        value: Value::Text(text.to_string()),
    };
    let mut members = vec![test("template", template)];
    members.extend(source.map(|source| test("source", source)));
    members.extend(flag.map(|flag| Member::Is(flag.to_string())));
    let where_ = if members.len() == 1 {
        members.remove(0)
    } else {
        Member::All(members)
    };
    let node = Node::Members {
        of: Collection::Lines,
        where_: Box::new(where_.clone()),
    };
    // the tree's own checks on the template: no sign before a #, no number typed into it
    tree::check(&node)?;
    let group = Group::bind(&where_)?;
    Ok((node, group))
}

/// What a vocabulary read is narrowed by, as the term it is — `line(true())`
/// for every line, `line(template:words)`, `line(template~"pattern")` — and
/// that term bound: the lines a row is made of are the ones it selects, its
/// `none` bucket is its not and its `undecided` bucket is `undecided( … )`
/// of it (C105).
pub(crate) fn narrowing(narrow: Option<&(Op, String)>) -> Result<(Node, Group), LanguageError> {
    let where_ = match narrow {
        None => Member::Const(true),
        Some((op, text)) => Member::Test {
            attr: "template".to_string(),
            op: *op,
            value: Value::Text(text.clone()),
        },
    };
    let group = Group::bind(&where_)?;
    Ok((
        Node::Members {
            of: Collection::Lines,
            where_: Box::new(where_),
        },
        group,
    ))
}

fn admits(member: &BMember, source: &str) -> bool {
    fn asked(member: &BMember, source: &str) -> Truth {
        match member {
            BMember::All(children) => all_of(children.iter().map(|c| asked(c, source))),
            BMember::Any(children) => any_of(children.iter().map(|c| asked(c, source))),
            BMember::Not(inner) => negated(asked(inner, source)),
            BMember::Const(value) => sure(*value),
            BMember::Source(sources) => sure(sources.contains(&source)),
            BMember::Template(_) | BMember::Slot { .. } | BMember::Is(_) => Truth::Undecided,
        }
    }
    asked(member, source) != Truth::False
}

fn has_slot(member: &Member) -> bool {
    match member {
        Member::All(children) | Member::Any(children) => children.iter().any(has_slot),
        Member::Not(inner) => has_slot(inner),
        Member::Test { attr, .. } => tree::is_slot_word(attr),
        Member::Const(_) | Member::Is(_) => false,
    }
}

fn asks_a_flag(member: &Member) -> bool {
    match member {
        Member::All(children) | Member::Any(children) => children.iter().any(asks_a_flag),
        Member::Not(inner) => asks_a_flag(inner),
        Member::Is(_) => true,
        Member::Test { .. } | Member::Const(_) => false,
    }
}

fn template_tests(member: &Member, out: &mut Vec<(Op, String)>) {
    match member {
        Member::All(children) | Member::Any(children) => {
            children.iter().for_each(|c| template_tests(c, out));
        }
        Member::Not(inner) => template_tests(inner, out),
        Member::Test {
            attr,
            op,
            value: Value::Text(text),
        } if attr == "template" => out.push((*op, text.clone())),
        _ => {}
    }
}

/// The group's selector (the module doc): the group with every slot
/// comparison taken as favourably as it can be — true where it helps the
/// group hold, false under a not — and the constants folded away. An
/// occurrence the selector refuses is one no numbers could make the group
/// hold on, so `"T" (source=explicit arg1>=90)` selects as
/// `"T" source=explicit` does, and `"T" arg1>=90` selects as `"T"`.
fn selector(whole: &Member) -> Member {
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
        Member::Is(name) => match bind::legal(LINE_FLAGS, name) {
            Some(flag) => BMember::Is(flag),
            None if bind::legal(ITEM_FLAGS, name).is_some() => {
                return Err(LanguageError::new(
                    ErrorKind::UnknownName,
                    format!(
                        "`is:{name}` is said of the item, outside the line's group; a line is: {}",
                        LINE_FLAGS.join(", ")
                    ),
                ));
            }
            None => {
                return Err(bind::unknown("flag of a line", name, LINE_FLAGS, |near| {
                    format!("is:{near}")
                }));
            }
        },
        Member::Test { attr, op, value } => match attr.as_str() {
            "template" => BMember::Template(bind::text_test("template", *op, value)?),
            "source" => BMember::Source(bind::closed("source", SOURCES, *op, value)?),
            slot if tree::is_slot_word(slot) => BMember::Slot {
                word: slot.to_string(),
                test: bind::num_test(slot, *op, value)?,
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

// ---- a link group --------------------------------------------------------------------------

/// A node inside a link group, bound (the module doc, "A link group's").
#[derive(Debug, Clone)]
enum BLink {
    All(Vec<BLink>),
    Any(Vec<BLink>),
    Not(Box<BLink>),
    Const(bool),
    /// A colour's count in the group, by GGG's letter.
    Colour {
        letter: &'static str,
        test: NumTest,
    },
    Size(NumTest),
}

/// `linked( … )`, bound: what is asked of one link group.
#[derive(Debug, Clone)]
pub(crate) struct Linked(BLink);

/// The attributes a link group has, as the help and an error list them.
pub(crate) fn link_attributes() -> Vec<&'static str> {
    let mut out = sockets::colour_words();
    out.push("size");
    out
}

impl Linked {
    pub fn bind(whole: &Member) -> Result<Linked, LanguageError> {
        Ok(Linked(link(whole)?))
    }

    /// Whether the group asks a colour's count: then a socket whose colour
    /// could not be read is a reason it is open (an outside review,
    /// 2026-09-24, 1).
    pub fn asks_colour(&self) -> bool {
        fn asks(member: &BLink) -> bool {
            match member {
                BLink::All(children) | BLink::Any(children) => children.iter().any(asks),
                BLink::Not(inner) => asks(inner),
                BLink::Colour { .. } => true,
                BLink::Const(_) | BLink::Size(_) => false,
            }
        }
        asks(&self.0)
    }

    /// Three-valued on one link group, as far as its counts were read.
    pub fn of(&self, group: &GroupCounts) -> Truth {
        fn truth(member: &BLink, group: &GroupCounts) -> Truth {
            match member {
                BLink::All(children) => all_of(children.iter().map(|c| truth(c, group))),
                BLink::Any(children) => any_of(children.iter().map(|c| truth(c, group))),
                BLink::Not(inner) => negated(truth(inner, group)),
                BLink::Const(value) => sure(*value),
                BLink::Colour { letter, test } => group.colour(letter).truth(test),
                BLink::Size(test) => group.size.truth(test),
            }
        }
        truth(&self.0, group)
    }
}

fn link(m: &Member) -> Result<BLink, LanguageError> {
    Ok(match m {
        Member::All(children) => BLink::All(children.iter().map(link).collect::<Result<_, _>>()?),
        Member::Any(children) => BLink::Any(children.iter().map(link).collect::<Result<_, _>>()?),
        Member::Not(inner) => BLink::Not(Box::new(link(inner)?)),
        Member::Const(value) => BLink::Const(*value),
        Member::Is(name) => {
            return Err(LanguageError::new(
                ErrorKind::UnknownName,
                format!(
                    "`is:{name}` is nothing a link group has: its counts are {}",
                    link_attributes().join(", ")
                ),
            ));
        }
        Member::Test { attr, op, value } => {
            if attr.eq_ignore_ascii_case("size") {
                BLink::Size(bind::num_test("size", *op, value)?)
            } else if let Some(letter) = sockets::letter(attr) {
                BLink::Colour {
                    letter,
                    test: bind::num_test(attr, *op, value)?,
                }
            } else {
                return Err(bind::unknown(
                    "count of a link group",
                    attr,
                    &link_attributes(),
                    |near| {
                        crate::print::print(&Node::Members {
                            of: Collection::Links,
                            where_: Box::new(Member::Test {
                                attr: near.to_string(),
                                op: *op,
                                value: value.clone(),
                            }),
                        })
                    },
                ));
            }
        }
    })
}
