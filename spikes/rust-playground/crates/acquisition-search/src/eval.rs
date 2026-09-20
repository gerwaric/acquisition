//! Evaluation (C92, C93): one bound term asked of one held item, and the
//! tree composed over the answers.
//!
//! The rules are `search/DESIGN.md`'s contract detail (C92 — slots, the
//! sort scalar, the together count; C93 — composition and witnesses; C94,
//! C95 — a sum's status), cited and not restated (the build plan, rule 6).
//!
//! # As built
//!
//! - **An atomic term has four outcomes**: matched, failed (the item has
//!   the thing and the comparison is false), lacked (known absence), and
//!   undecided. The tree sees three values: failed and lacked are both
//!   false.
//! - **A readable hit is a witness; absence needs everything readable.**
//!   A term is undecided only when it found no witness *and* something
//!   that could have held one is unread: for a field, its own key or the
//!   body; for a phrase, anything at all; for a line's group, the body or
//!   a source of lines the group's own `source=` does not rule out.
//! - **Outcomes are computed for every term on every item, and nothing
//!   else is**: what a row shows ([`evidence`]) and why an item is
//!   undecided ([`reasons`]) are worked out only for the items an answer
//!   prints.
//! - **A sum** adds the named slot over the occurrences that satisfy its
//!   group; an occurrence that names no such slot adds nothing; a sum of
//!   nothing is zero; with a possible contributor unread it is an
//!   incomplete subtotal, and a comparison on it is undecided whatever
//!   the subtotal already reaches.
//! - **The sort scalar** of `line(P).<slot>` is the largest value of the
//!   slot over the occurrences that satisfy `P`, in either direction; an
//!   item with none has no scalar and sorts last either way.

use serde::Serialize;

use crate::bind::{Atom, BMember, BProbe, Bound, Group, NumTest, SortKey, Term, Thing};
use crate::corpus::Held;
use crate::derive::{Line, Part, Shown, Unread};
use crate::tree::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    Matched,
    Failed,
    Lacked,
    Undecided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Truth {
    True,
    False,
    Undecided,
}

impl Outcome {
    pub fn truth(self) -> Truth {
        match self {
            Outcome::Matched => Truth::True,
            Outcome::Failed | Outcome::Lacked => Truth::False,
            Outcome::Undecided => Truth::Undecided,
        }
    }
}

/// Why something could not be established on an item (C93), with a hint
/// of what might resolve it — never a guarantee.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reason {
    /// What was unread: `the body`, `implicit lines`, `` `ilvl` ``.
    pub unread: String,
    pub problem: String,
    pub hint: &'static str,
}

const UNREAD_HINT: &str =
    "a refresh may help; a body GGG really gives this way needs a build that reads it";

/// What a row shows of a term that matched (C100: the lines the query
/// touched, the string a phrase hit, the value compared).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    Line {
        source: String,
        flags: Vec<String>,
        text: String,
    },
    Shown {
        part: String,
        text: String,
    },
    Value {
        name: String,
        value: serde_json::Value,
    },
}

pub(crate) fn number_json(n: f64) -> serde_json::Value {
    match Number::from_f64(n) {
        Number::Int(i) => serde_json::Value::from(i),
        Number::Float(f) => serde_json::Value::from(f),
    }
}

// ---- an item's things ----------------------------------------------------------------

/// The body's key a thing is read from; `None` for place, which is the
/// store's and is never unread.
fn body_key(thing: Thing) -> Option<&'static str> {
    Some(match thing {
        Thing::Name => "name",
        Thing::Typeline => "typeLine",
        Thing::Base => "baseType",
        Thing::Note => "note",
        Thing::Rarity => "rarity",
        Thing::Frame => "frameTypeId",
        Thing::Ilvl => "ilvl",
        Thing::Stack => "stackSize",
        Thing::Text
        | Thing::League
        | Thing::Tab
        | Thing::Character
        | Thing::Container
        | Thing::Id => {
            return None;
        }
    })
}

fn unread_for(held: &Held, thing: Thing) -> Vec<&Unread> {
    match (thing, body_key(thing)) {
        // a phrase may sit anywhere: anything unread leaves it open
        (Thing::Text, _) => held.item.unread.iter().collect(),
        (_, Some(key)) => held
            .item
            .unread
            .iter()
            .filter(|u| match &u.part {
                Part::Body => true,
                Part::Field(field) => field == key,
                _ => false,
            })
            .collect(),
        (_, None) => Vec::new(),
    }
}

/// The text values a thing has on this item: none when it lacks it, two
/// for an item in a substash (the substash's name and its tab's).
fn texts(held: &Held, thing: Thing) -> Vec<&str> {
    let item = &held.item;
    let place = &held.place;
    fn one(value: &Option<String>) -> Vec<&str> {
        value.as_deref().into_iter().collect()
    }
    match thing {
        Thing::Name => one(&item.name),
        Thing::Typeline => one(&item.typeline),
        Thing::Base => one(&item.base),
        Thing::Note => one(&item.note),
        Thing::Rarity => one(&item.rarity),
        Thing::Frame => one(&item.frame),
        Thing::League => one(&place.league),
        Thing::Container => one(&place.container),
        Thing::Tab if place.kind == "stash" => place
            .name
            .as_deref()
            .into_iter()
            .chain(place.parent.as_ref().and_then(|p| p.name.as_deref()))
            .collect(),
        Thing::Character if place.kind == "character" => one(&place.name),
        Thing::Tab | Thing::Character | Thing::Text | Thing::Ilvl | Thing::Stack | Thing::Id => {
            Vec::new()
        }
    }
}

fn number(held: &Held, thing: Thing) -> Option<f64> {
    match thing {
        Thing::Ilvl => held.item.ilvl.map(|n| n as f64),
        Thing::Stack => held.item.stack.map(|n| n as f64),
        _ => None,
    }
}

fn present(held: &Held, thing: Thing) -> bool {
    number(held, thing).is_some() || !texts(held, thing).is_empty()
}

fn shown_part(shown: Shown<'_>) -> String {
    match shown {
        Shown::Name => "name".to_string(),
        Shown::Typeline => "typeline".to_string(),
        Shown::Base => "base".to_string(),
        Shown::Property(p) => p.array.clone(),
        Shown::ItemLevel => "item level".to_string(),
        Shown::Requires => "requirements".to_string(),
        Shown::Line(line) => format!("{} line", line.source),
    }
}

// ---- a line's group ---------------------------------------------------------------------

pub(crate) fn member_holds(member: &BMember, line: &Line) -> bool {
    match member {
        BMember::All(children) => children.iter().all(|c| member_holds(c, line)),
        BMember::Any(children) => children.iter().any(|c| member_holds(c, line)),
        BMember::Not(inner) => !member_holds(inner, line),
        BMember::Const(value) => *value,
        BMember::Template(test) => test.holds(&line.template),
        BMember::Source(sources) => sources.contains(&line.source.as_str()),
        BMember::Slot { word, test } => line.slot(word).is_some_and(|n| test.holds(n)),
        BMember::Is(flag) => line.flags.iter().any(|f| f == flag),
    }
}

/// What stops this group claiming an absence on this item.
fn unread_lines<'a>(held: &'a Held, group: &Group) -> Vec<&'a Unread> {
    held.item
        .unread
        .iter()
        .filter(|u| match &u.part {
            Part::Body => true,
            Part::Lines(source) => group
                .sources
                .as_ref()
                .is_none_or(|sources| sources.contains(&source.as_str())),
            _ => false,
        })
        .collect()
}

fn satisfying<'a>(held: &'a Held, member: &'a BMember) -> impl Iterator<Item = &'a Line> {
    held.item
        .lines
        .iter()
        .filter(move |l| member_holds(member, l))
}

/// A sum and whether every possible contributor was readable.
pub(crate) fn sum(held: &Held, group: &Group, slot: &str) -> (f64, bool) {
    let total = satisfying(held, &group.whole)
        .filter_map(|line| line.slot(slot))
        .sum();
    (total, unread_lines(held, group).is_empty())
}

/// C92's together count, asked of an item the group failed on: no
/// occurrence meets the lower bound and the sum of the selected ones does.
pub(crate) fn together(held: &Held, group: &Group) -> bool {
    let Some(lower) = &group.together else {
        return false;
    };
    let total: f64 = satisfying(held, &group.selector)
        .filter_map(|line| line.slot(&lower.slot))
        .sum();
    NumTest::Cmp(lower.op, lower.bound.as_f64()).holds(total)
}

// ---- outcomes ---------------------------------------------------------------------------------

fn decided(hit: bool, has: bool, unread: bool) -> Outcome {
    match (hit, unread, has) {
        (true, _, _) => Outcome::Matched,
        (false, true, _) => Outcome::Undecided,
        (false, false, true) => Outcome::Failed,
        (false, false, false) => Outcome::Lacked,
    }
}

/// One term asked of one item. `earlier` holds the outcomes of the terms
/// bound before this one, which is every term an `undecided( … )` asks
/// about.
pub(crate) fn outcome(atom: &Atom, held: &Held, earlier: &[Outcome]) -> Outcome {
    match atom {
        Atom::Const(value) => decided(*value, true, false),
        Atom::Text {
            thing: Thing::Text,
            test,
        } => decided(
            held.item.displayed().any(|(_, row)| test.holds(row)),
            true,
            !held.item.unread.is_empty(),
        ),
        Atom::Text { thing, test } => {
            let values = texts(held, *thing);
            decided(
                values.iter().any(|v| test.holds(v)),
                !values.is_empty(),
                !unread_for(held, *thing).is_empty(),
            )
        }
        Atom::Closed { thing, values } => {
            let has = texts(held, *thing);
            decided(
                has.iter()
                    .any(|v| values.iter().any(|legal| legal.eq_ignore_ascii_case(v))),
                !has.is_empty(),
                !unread_for(held, *thing).is_empty(),
            )
        }
        Atom::Number { thing, test } => {
            let n = number(held, *thing);
            decided(
                n.is_some_and(|n| test.holds(n)),
                n.is_some(),
                !unread_for(held, *thing).is_empty(),
            )
        }
        Atom::Id(id) => {
            let place = &held.place;
            let hit = held.item.facts.id == *id
                || place.id == *id
                || place.parent.as_ref().is_some_and(|p| p.id == *id);
            decided(hit, true, false)
        }
        Atom::Has(thing) => decided(
            present(held, *thing),
            false,
            !unread_for(held, *thing).is_empty(),
        ),
        Atom::Is(flag) => decided(
            held.item.flags.iter().any(|f| f == flag),
            true,
            !unread_flag(held, flag).is_empty(),
        ),
        Atom::Lines(group) => decided(
            satisfying(held, &group.whole).next().is_some(),
            satisfying(held, &group.selector).next().is_some() && !group.selects_only,
            !unread_lines(held, group).is_empty(),
        ),
        Atom::Sum { group, slot, test } => match sum(held, group, slot) {
            (total, true) => decided(test.holds(total), true, false),
            (_, false) => Outcome::Undecided,
        },
        Atom::Undecided(probe) => {
            let open = match probe {
                BProbe::Field(thing) => !unread_for(held, *thing).is_empty(),
                BProbe::Lines(group) => !unread_lines(held, group).is_empty(),
                BProbe::Term(inner) => truth(inner, earlier) == Truth::Undecided,
            };
            decided(open, true, false)
        }
    }
}

fn unread_flag<'a>(held: &'a Held, flag: &str) -> Vec<&'a Unread> {
    held.item
        .unread
        .iter()
        .filter(|u| match &u.part {
            Part::Body => true,
            Part::Field(key) => key == flag || key == "influences",
            _ => false,
        })
        .collect()
}

/// The tree over its terms' outcomes (C93): three-valued and, or, not,
/// and at-least-N-of as a count interval.
pub(crate) fn truth(bound: &Bound, outcomes: &[Outcome]) -> Truth {
    match bound {
        Bound::Term(i) => outcomes.get(*i).map_or(Truth::Undecided, |o| o.truth()),
        Bound::Not(inner) => match truth(inner, outcomes) {
            Truth::True => Truth::False,
            Truth::False => Truth::True,
            Truth::Undecided => Truth::Undecided,
        },
        Bound::All(children) => {
            let each: Vec<Truth> = children.iter().map(|c| truth(c, outcomes)).collect();
            if each.contains(&Truth::False) {
                Truth::False
            } else if each.contains(&Truth::Undecided) {
                Truth::Undecided
            } else {
                Truth::True
            }
        }
        Bound::Any(children) => {
            let each: Vec<Truth> = children.iter().map(|c| truth(c, outcomes)).collect();
            if each.contains(&Truth::True) {
                Truth::True
            } else if each.contains(&Truth::Undecided) {
                Truth::Undecided
            } else {
                Truth::False
            }
        }
        Bound::Holds { of, min, max } => {
            let each: Vec<Truth> = of.iter().map(|c| truth(c, outcomes)).collect();
            let sure = each.iter().filter(|t| **t == Truth::True).count() as u64;
            let open = each.iter().filter(|t| **t == Truth::Undecided).count() as u64;
            let (min, max) = (min.map_or(0, u64::from), max.map_or(u64::MAX, u64::from));
            if sure >= min && sure + open <= max {
                Truth::True
            } else if sure + open < min || sure > max {
                Truth::False
            } else {
                Truth::Undecided
            }
        }
    }
}

/// The undecided terms an undecided node rests on.
pub(crate) fn blame(bound: &Bound, outcomes: &[Outcome], out: &mut Vec<usize>) {
    if truth(bound, outcomes) != Truth::Undecided {
        return;
    }
    match bound {
        Bound::Term(i) => out.push(*i),
        Bound::Not(inner) => blame(inner, outcomes, out),
        Bound::All(children) | Bound::Any(children) | Bound::Holds { of: children, .. } => {
            children.iter().for_each(|c| blame(c, outcomes, out));
        }
    }
}

/// The matched terms a matching item's row shows: those no `-` negates.
pub(crate) fn touched(bound: &Bound, outcomes: &[Outcome], positive: bool, out: &mut Vec<usize>) {
    match bound {
        Bound::Term(i) => {
            if positive && outcomes.get(*i) == Some(&Outcome::Matched) {
                out.push(*i);
            }
        }
        Bound::Not(inner) => touched(inner, outcomes, !positive, out),
        Bound::All(children) | Bound::Any(children) | Bound::Holds { of: children, .. } => {
            children
                .iter()
                .for_each(|c| touched(c, outcomes, positive, out));
        }
    }
}

// ---- what an answer prints of one item --------------------------------------------------

fn line_evidence(line: &Line) -> Evidence {
    Evidence::Line {
        source: line.source.clone(),
        flags: line.flags.clone(),
        text: line.text.clone(),
    }
}

/// What a row shows of a term that matched on its item. The header and
/// the place are on every row already, so a term on them shows nothing.
pub(crate) fn evidence(term: &Term, held: &Held) -> Vec<Evidence> {
    const MOST: usize = 3;
    match &term.atom {
        Atom::Text {
            thing: Thing::Text,
            test,
        } => held
            .item
            .displayed()
            .filter(|(_, row)| test.holds(row))
            .map(|(shown, row)| Evidence::Shown {
                part: shown_part(shown),
                text: row.to_string(),
            })
            .take(MOST)
            .collect(),
        Atom::Text {
            thing: Thing::Note,
            test,
        } => texts(held, Thing::Note)
            .into_iter()
            .filter(|v| test.holds(v))
            .map(|v| Evidence::Shown {
                part: "note".to_string(),
                text: v.to_string(),
            })
            .collect(),
        Atom::Number { thing, .. } => number(held, *thing)
            .map(|n| Evidence::Value {
                name: match &term.node {
                    crate::tree::Node::Test { field, .. } => field.clone(),
                    other => crate::print::print(other),
                },
                value: number_json(n),
            })
            .into_iter()
            .collect(),
        Atom::Lines(group) => satisfying(held, &group.whole).map(line_evidence).collect(),
        Atom::Sum { group, slot, .. } => {
            let (total, _) = sum(held, group, slot);
            let name = match &term.node {
                crate::tree::Node::Compare { value, .. } => crate::print::print_value(value),
                other => crate::print::print(other),
            };
            std::iter::once(Evidence::Value {
                name,
                value: number_json(total),
            })
            .chain(satisfying(held, &group.whole).map(line_evidence))
            .collect()
        }
        _ => Vec::new(),
    }
}

/// Why a term is undecided on an item.
pub(crate) fn reasons(atom: &Atom, held: &Held) -> Vec<Reason> {
    let unread = match atom {
        Atom::Text { thing, .. }
        | Atom::Closed { thing, .. }
        | Atom::Number { thing, .. }
        | Atom::Has(thing) => unread_for(held, *thing),
        Atom::Is(flag) => unread_flag(held, flag),
        Atom::Lines(group) | Atom::Sum { group, .. } => unread_lines(held, group),
        Atom::Id(_) | Atom::Const(_) | Atom::Undecided(_) => Vec::new(),
    };
    unread
        .into_iter()
        .map(|u| Reason {
            unread: match &u.part {
                Part::Body => "the body".to_string(),
                Part::Field(key) => format!("`{key}`"),
                Part::Properties(array) => format!("`{array}`"),
                Part::Lines(source) => format!("{source} lines"),
            },
            problem: u.problem.clone(),
            hint: UNREAD_HINT,
        })
        .collect()
}

/// What an item sorts by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Scalar {
    Value(f64),
    /// No satisfying occurrence, or the item lacks the field.
    None,
    /// A sum with a possible contributor unread: the subtotal, never a
    /// total, and no place in the order.
    Incomplete(f64),
}

pub(crate) fn scalar(key: &SortKey, held: &Held) -> Scalar {
    match key {
        SortKey::Number(thing) => number(held, *thing).map_or(Scalar::None, Scalar::Value),
        SortKey::Projection { group, slot } => satisfying(held, &group.whole)
            .filter_map(|line| line.slot(slot))
            .reduce(f64::max)
            .map_or(Scalar::None, Scalar::Value),
        SortKey::Sum { group, slot } => match sum(held, group, slot) {
            (total, true) => Scalar::Value(total),
            (subtotal, false) => Scalar::Incomplete(subtotal),
        },
    }
}
