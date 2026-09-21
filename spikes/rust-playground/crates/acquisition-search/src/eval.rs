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
//! - **A readable hit is a witness; absence needs everything readable
//!   that could have held the thing — and nothing else.** A term is
//!   undecided only when it found no witness *and* the evidence it needs
//!   is unread: for a field, its own key or the body; for an item's flag,
//!   its key, and `influences` for the flags that live there; for a
//!   phrase, what holds a displayed string; for a line's group, the body,
//!   a source of lines the group admits, `hybrid` when it admits that
//!   source, and — where the group asks a flag — an occurrence whose own
//!   flags are unread. Which sources a group admits, what it selects and
//!   what it says of one occurrence are the group's own (`group.rs`), read
//!   here and never worked out again.
//! - **A line's group is three-valued on each occurrence** (`group.rs`). A
//!   sum over occurrences that may or may
//!   not be selected is incomplete, and they establish no together count —
//!   where they name the slot: one that does not cannot contribute
//!   whichever way its flag falls, and leaves nothing open.
//! - **A value is open exactly when it would sort as incomplete**:
//!   `undecided(sum( … ))`, `undecided(line(P).<slot>)` and `--sort` ask
//!   one function.
//! - **Outcomes are computed for every term on every item, and nothing
//!   else is**: what a row shows ([`evidence`]) and why an item is
//!   undecided ([`reasons`]) are worked out only for the items an answer
//!   prints. An `undecided( … )` that matched shows the reasons of what it
//!   asked about, so an undecided count's route returns its members with
//!   why.
//! - **A sum** adds the named slot — as decimals, exactly and the same in
//!   any order (`exact.rs`) — over the occurrences that satisfy its
//!   group; an occurrence that names no such slot adds nothing; a sum of
//!   nothing is zero; with a possible contributor unread it is an
//!   incomplete subtotal, and a comparison on it is undecided whatever
//!   the subtotal already reaches.
//! - **The sort scalar** of `line(P).<slot>` is the largest value of the
//!   slot over the occurrences that satisfy `P`, in either direction; an
//!   item with none has no scalar and sorts last either way. So does one
//!   whose largest is not established — a source `P` admits is unread, or
//!   an occurrence `P` may select holds a larger number — with what was
//!   readable shown and its status beside it, as an incomplete sum is.

use serde::Serialize;

use crate::bind::{Atom, BProbe, Bound, NumTest, SortKey, Term, Thing};
use crate::corpus::Held;
use crate::derive::{Line, Part, Shown, Slot, Unread};
use crate::exact;
pub(crate) use crate::group::Truth;
use crate::group::{Asked, Group};
use crate::tree::Number;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    Matched,
    Failed,
    Lacked,
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
    /// Why the term an `undecided( … )` asked about is open on this item.
    Undecided {
        path: String,
        term: String,
        #[serde(flatten)]
        reason: Reason,
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
        // a phrase may sit in any displayed string: what holds one leaves
        // it open, and a flag or a stack size holds none
        (Thing::Text, _) => held
            .item
            .unread
            .iter()
            .filter(|u| match &u.part {
                Part::Body | Part::Properties(_) | Part::Lines(_) => true,
                Part::Field(key) => {
                    matches!(
                        key.as_str(),
                        "name" | "typeLine" | "baseType" | "ilvl" | "hybrid"
                    )
                }
                Part::Flags(_) | Part::Numbers(_) => false,
            })
            .collect(),
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
pub(crate) fn texts(held: &Held, thing: Thing) -> Vec<&str> {
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

/// What stops this group claiming an absence on this item: the body, a
/// source of lines the group admits, and `hybrid` — a vaal gem's base
/// skill, whose lines are the source `hybrid` — when it admits that.
fn unread_lines<'a>(held: &'a Held, group: &Group) -> Vec<&'a Unread> {
    held.item
        .unread
        .iter()
        .filter(|u| match &u.part {
            Part::Body => true,
            Part::Lines(source) => group.admits(source),
            Part::Field(key) => key == "hybrid" && group.admits("hybrid"),
            // a line's flags and numbers: the occurrence itself says so
            // (`Asked::of`)
            Part::Flags(_) | Part::Numbers(_) | Part::Properties(_) => false,
        })
        .collect()
}

fn satisfying<'a>(held: &'a Held, asked: &'a Asked) -> impl Iterator<Item = &'a Line> {
    held.item.lines.iter().filter(move |l| asked.holds(l))
}

/// Whether some occurrence leaves what is asked open: its flags are unread
/// and the group asks about one.
fn open_on_a_line(held: &Held, asked: &Asked) -> bool {
    held.item
        .lines
        .iter()
        .any(|l| asked.of(l) == Truth::Undecided)
}

/// Whether an occurrence leaves the slot's contribution open: it may or
/// may not satisfy what is asked and names the slot, or it satisfies it
/// and the slot's number could not be read. One that does not name the
/// slot cannot contribute whichever way its flag falls, so it leaves no
/// sum, no largest and no together count open (C93's known absence).
fn leaves_the_slot_open(asked: &Asked, slot: &str, line: &Line) -> bool {
    match (asked.of(line), line.slot(slot)) {
        (Truth::False, _) | (_, Slot::Absent) | (Truth::True, Slot::Is(_)) => false,
        (Truth::Undecided, _) | (Truth::True, Slot::Unread) => true,
    }
}

fn open_with_the_slot(held: &Held, asked: &Asked, slot: &str) -> bool {
    held.item
        .lines
        .iter()
        .any(|l| leaves_the_slot_open(asked, slot, l))
}

fn value(line: &Line, slot: &str) -> Option<f64> {
    match line.slot(slot) {
        Slot::Is(n) => Some(n),
        Slot::Absent | Slot::Unread => None,
    }
}

/// A sum and whether every possible contributor was readable.
pub(crate) fn sum(held: &Held, group: &Group, slot: &str) -> (f64, bool) {
    let total = exact::sum(satisfying(held, &group.whole).filter_map(|line| value(line, slot)));
    let complete =
        unread_lines(held, group).is_empty() && !open_with_the_slot(held, &group.whole, slot);
    (total, complete)
}

/// C92's together count, asked of an item the group failed on: no
/// occurrence meets the lower bound and the sum of the selected ones does.
/// An occurrence that may or may not be selected establishes nothing.
pub(crate) fn together(held: &Held, group: &Group) -> bool {
    let Some(lower) = &group.together else {
        return false;
    };
    if open_with_the_slot(held, &group.selector, &lower.slot) {
        return false;
    }
    // a sum of no occurrence is zero, which is at least any bound of zero
    // or less — and is nothing reaching it: only occurrences that count
    // reach a bound together
    let counted: Vec<f64> = satisfying(held, &group.selector)
        .filter_map(|line| value(line, &lower.slot))
        .collect();
    !counted.is_empty() && NumTest::Cmp(lower.op, lower.bound.as_f64()).holds(exact::sum(counted))
}

/// The occurrences a group's selector picks on this item: what the
/// selector resolved to here, whatever its comparisons then made of them.
pub(crate) fn selected<'a>(held: &'a Held, group: &'a Group) -> impl Iterator<Item = &'a Line> {
    satisfying(held, &group.selector)
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
            !unread_for(held, Thing::Text).is_empty(),
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
            // an occurrence that may be selected is no known absence
            !group.selects_only
                && held
                    .item
                    .lines
                    .iter()
                    .any(|l| group.selector.of(l) != Truth::False),
            !unread_lines(held, group).is_empty() || open_on_a_line(held, &group.whole),
        ),
        Atom::Sum { group, slot, test } => match sum(held, group, slot) {
            (total, true) => decided(test.holds(total), true, false),
            (_, false) => Outcome::Undecided,
        },
        Atom::Undecided(probe) => {
            let open = match probe {
                BProbe::Field(thing) => !unread_for(held, *thing).is_empty(),
                // one judgement with the sort's: a value is open exactly
                // when it would sort as incomplete
                BProbe::Value(key) => matches!(scalar(key, held), Scalar::Incomplete(_)),
                BProbe::Term(inner) => truth(inner, earlier) == Truth::Undecided,
            };
            decided(open, true, false)
        }
    }
}

/// What stops an item's flag being a no: its own key, and for the flags
/// that live in `influences`, that object or the flag's own key in it —
/// never a sibling's, and never `influences` for `corrupted`.
fn unread_flag<'a>(held: &'a Held, flag: &str) -> Vec<&'a Unread> {
    held.item
        .unread
        .iter()
        .filter(|u| match &u.part {
            Part::Body => true,
            Part::Field(key) => {
                key == flag
                    || (crate::derive::INFLUENCES.contains(&flag)
                        && (key == "influences" || key.strip_prefix("influences.") == Some(flag)))
            }
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
/// An `undecided( … )` that matched shows why — the reasons of what it
/// asked about — so the route of an undecided count returns its members
/// with their reasons. Bounded, with what was left out counted: the item
/// whole is `show <id>`. What is kept follows the item — its lines in its
/// order, its unread parts in theirs — and never the order the terms were
/// written in, which a rewrite that changes no meaning may change
/// (invariant 7).
pub(crate) fn evidence(
    terms: &[Term],
    term: &Term,
    held: &Held,
    outcomes: &[Outcome],
) -> (Vec<Evidence>, usize) {
    let blamed = match &term.atom {
        Atom::Undecided(BProbe::Term(inner)) => {
            let mut blamed = Vec::new();
            blame(inner, outcomes, &mut blamed);
            blamed
        }
        Atom::Undecided(_) => terms
            .iter()
            .position(|t| t.path == term.path)
            .into_iter()
            .collect(),
        _ => {
            let mut all = everything(term, held);
            let keep = SHOWN + usize::from(matches!(all.first(), Some(Evidence::Value { .. })));
            let left_out = all.len().saturating_sub(keep);
            all.truncate(keep);
            return (all, left_out);
        }
    };
    let (pairs, left_out) = why(terms, &blamed, held);
    let shows = pairs
        .into_iter()
        .map(|(asked, reason)| Evidence::Undecided {
            path: asked.path.clone(),
            term: crate::print::print(&asked.node),
            reason,
        })
        .collect();
    (shows, left_out)
}

/// How much of one term a row shows, a sum's value beside it, and how many
/// of an item's unread parts are given as why it is undecided.
const SHOWN: usize = 6;

/// Why these terms are open on an item: each term with each unread part
/// it rests on. The bound is on the item's parts, the first [`SHOWN`] in
/// the item's own order, every term's pair with each kept — how many terms
/// a query has is its author's — and the parts past the bound are counted.
pub(crate) fn why<'a>(
    terms: &'a [Term],
    blamed: &[usize],
    held: &Held,
) -> (Vec<(&'a Term, Reason)>, usize) {
    let at = |unread: &Unread| {
        held.item
            .unread
            .iter()
            .position(|u| std::ptr::eq(u, unread))
            .unwrap_or(usize::MAX)
    };
    let mut pairs: Vec<(usize, usize, &Unread)> = blamed
        .iter()
        .filter_map(|i| terms.get(*i).map(|term| (*i, term)))
        .flat_map(|(i, term)| {
            unread_of(&term.atom, held)
                .into_iter()
                .map(move |unread| (i, unread))
        })
        .map(|(i, unread)| (at(unread), i, unread))
        .collect();
    pairs.sort_by_key(|(part, term, _)| (*part, *term));
    let mut parts: Vec<usize> = pairs.iter().map(|(part, _, _)| *part).collect();
    parts.dedup();
    let left_out = parts.len().saturating_sub(SHOWN);
    let last = parts.get(SHOWN.saturating_sub(1)).copied();
    let shown = pairs
        .into_iter()
        .filter(|(part, _, _)| last.is_none_or(|last| *part <= last))
        .map(|(_, i, unread)| (&terms[i], reason(unread)))
        .collect();
    (shown, left_out)
}

fn everything(term: &Term, held: &Held) -> Vec<Evidence> {
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

/// What left a term open on an item.
fn unread_of<'a>(atom: &Atom, held: &'a Held) -> Vec<&'a Unread> {
    let of_group = |group: &Group, slot: Option<&str>| {
        let mut unread = unread_lines(held, group);
        // what an occurrence itself left open, explained by that occurrence
        // and by what the group asked of it: its flags where a flag is
        // asked, its numbers where a number is
        let open: Vec<usize> = held
            .item
            .lines
            .iter()
            .enumerate()
            .filter(|(_, l)| match slot {
                Some(slot) => leaves_the_slot_open(&group.whole, slot, l),
                None => group.whole.of(l) == Truth::Undecided,
            })
            .map(|(at, _)| at)
            .collect();
        unread.extend(held.item.unread.iter().filter(|u| {
            let asked = match &u.part {
                Part::Flags(_) => group.asks_a_flag,
                Part::Numbers(_) => slot.is_some() || !group.selects_only,
                _ => false,
            };
            asked && u.line.is_some_and(|at| open.contains(&at))
        }));
        unread
    };
    match atom {
        Atom::Text { thing, .. }
        | Atom::Closed { thing, .. }
        | Atom::Number { thing, .. }
        | Atom::Has(thing)
        | Atom::Undecided(BProbe::Field(thing)) => unread_for(held, *thing),
        Atom::Is(flag) => unread_flag(held, flag),
        Atom::Lines(group) => of_group(group, None),
        Atom::Sum { group, slot, .. } => of_group(group, Some(slot.as_str())),
        Atom::Undecided(BProbe::Value(key)) => match key.as_ref() {
            SortKey::Projection { group, slot } | SortKey::Sum { group, slot } => {
                of_group(group, Some(slot.as_str()))
            }
            SortKey::Number(thing) => unread_for(held, *thing),
        },
        Atom::Id(_) | Atom::Const(_) | Atom::Undecided(BProbe::Term(_)) => Vec::new(),
    }
}

fn reason(unread: &Unread) -> Reason {
    Reason {
        unread: match &unread.part {
            Part::Body => "the body".to_string(),
            Part::Field(key) => format!("`{key}`"),
            Part::Properties(array) => format!("`{array}`"),
            Part::Lines(source) => format!("{source} lines"),
            Part::Flags(source) => format!("the flags of {source} lines"),
            Part::Numbers(source) => format!("the numbers of {source} lines"),
        },
        problem: unread.problem.clone(),
        hint: UNREAD_HINT,
    }
}

/// What an item sorts by.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Scalar {
    Value(f64),
    /// No satisfying occurrence, or the item lacks the field.
    None,
    /// What was readable, which is not the answer: a field that could not
    /// be read; a sum with a possible
    /// contributor unread is a subtotal, never a total, and a largest
    /// occurrence is not the largest while an unread source the group
    /// admits, or an occurrence it may select, could hold a larger one. No
    /// place in the order.
    Incomplete(Option<f64>),
}

pub(crate) fn scalar(key: &SortKey, held: &Held) -> Scalar {
    match key {
        SortKey::Number(thing) => match number(held, *thing) {
            Some(n) => Scalar::Value(n),
            // unread is not absent (C93): `undecided(ilvl)` says the same
            None if !unread_for(held, *thing).is_empty() => Scalar::Incomplete(None),
            None => Scalar::None,
        },
        SortKey::Projection { group, slot } => {
            let largest = satisfying(held, &group.whole)
                .filter_map(|line| value(line, slot))
                .reduce(f64::max);
            // an unread number may be any number
            let could_be_larger = held.item.lines.iter().any(|line| {
                leaves_the_slot_open(&group.whole, slot, line)
                    && value(line, slot).is_none_or(|n| largest.is_none_or(|most| n > most))
            });
            if could_be_larger || !unread_lines(held, group).is_empty() {
                Scalar::Incomplete(largest)
            } else {
                largest.map_or(Scalar::None, Scalar::Value)
            }
        }
        SortKey::Sum { group, slot } => match sum(held, group, slot) {
            (total, true) => Scalar::Value(total),
            (subtotal, false) => Scalar::Incomplete(Some(subtotal)),
        },
    }
}
