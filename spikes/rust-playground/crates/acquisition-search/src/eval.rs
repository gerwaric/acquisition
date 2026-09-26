//! Evaluation (C92, C93): one bound term asked of one held item, and the
//! tree composed over the answers.
//!
//! The words are `search/DESIGN.md`'s reference (*Slots*, *Composition*,
//! *A sum's status*); the rules that decide an answer are this doc's,
//! taken from the contract detail 2026-09-23 (C92 — slots, the sort
//! scalar, the together count; C93 — composition and witnesses; C95 — a
//! sum's status).
//!
//! # Decisions as recorded
//!
//! - **C93 — composition.** `true or undecided` is true; `false and
//!   undecided` is false; `not undecided` is undecided; at-least-N-of with
//!   `t` true and `u` undecided children has the count interval `[t, t+u]`:
//!   true when wholly inside the bound, false when disjoint, otherwise
//!   undecided, an omitted upper bound unbounded (`truth`, below). The four
//!   counts are taken per atomic term before outer composition, over the
//!   fixed item scope, before any limit, with no short-circuit omission;
//!   root matches and root undecided are separate answer counts
//!   (`answer.rs`).
//! - **C92 — the together count** is a diagnostic beside C93's counts,
//!   never a fifth bucket: for a lower bound on a one-slot line selector,
//!   an item counts once when no occurrence meets the bound and the
//!   complete sum of its occurrences does — 20 + 75 counts for ≥ 90, 95 + 5
//!   does not; an unreadable possible contributor cannot establish it, nor
//!   does an item none of whose occurrences counts; upper bounds, equality
//!   and compound terms are not applicable, never zero (`group.rs`, the
//!   together bound).
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
//! - **The class is the table's** (`class.rs`): what it gave is the item's
//!   one value of `class`, and what stopped it is one reason beside the
//!   deriver's unread parts — shown, tallied and hinted as they are, its
//!   hint the table's. `reqlevel` is the deriver's number.
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
//! - **A socket count** (`sockets.rs`; C101) is an interval of what was
//!   read: a comparison on it is decided where the whole interval agrees
//!   and undecided otherwise, so a socket whose colour could not be read
//!   leaves `sockets.red>=2` open only where it could turn it; the value
//!   is established when the interval is one number, which is when it
//!   sorts as a value and its probe is false. `linked( … )` is asked of
//!   each link group the item may have, three-valued (`group::Linked`):
//!   a group that holds is a witness; none known and none possible is
//!   lacked; a group known and none holding is failed.
//! - **A computed value** (`pseudo.rs`; C94, C101) is asked as a sum is:
//!   complete, its comparison is decided; an incomplete subtotal, or a
//!   total with no definition for the realm, is undecided; a derived
//!   field whose input the item lacks is lacked. Its sort scalar and its
//!   reasons are the same functions', and a total's arithmetic is this
//!   module's own sum over each row's group.
//! - **The price** (`price.rs`; C81, C100) is read as the listing state
//!   answered it: `has:priced` is a witness where the effective statement
//!   carries a price, known absence where it resolved to none, no price
//!   or skip, and undecided — with the listing state's own reason — where
//!   a row that could decide cannot be read; `price.amount`,
//!   `price.currency` and `price.lot` are that price's parts, lacked with
//!   it, and open with it. A price term shows the price as printed, the
//!   side that decided and the target it came from.

use std::borrow::Cow;

use serde::Serialize;

use crate::bind::{Atom, BProbe, Bound, NumTest, SortKey, Term, Thing};
use crate::corpus::Held;
use crate::derive::{Line, Part, Shown, Slot, Unread};
use crate::exact::{self, Exact};
pub(crate) use crate::group::Truth;
use crate::group::{Asked, Group};
use crate::pseudo::{self, Valued};
use crate::sockets::{self, Counted};
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
        | Thing::Class
        | Thing::ReqLevel
        | Thing::Sockets
        | Thing::Links
        | Thing::SocketColour(_)
        | Thing::Price
        | Thing::PriceAmount
        | Thing::PriceCurrency
        | Thing::PriceLot
        | Thing::League
        | Thing::Tab
        | Thing::Character
        | Thing::Container
        | Thing::Id => {
            return None;
        }
    })
}

/// Whether a thing is the price or one of its parts (`price.rs`).
fn is_price_thing(thing: Thing) -> bool {
    matches!(
        thing,
        Thing::Price | Thing::PriceAmount | Thing::PriceCurrency | Thing::PriceLot
    )
}

/// Whether a thing is a count over the sockets (`sockets.rs`).
fn is_socket_thing(thing: Thing) -> bool {
    matches!(
        thing,
        Thing::Sockets | Thing::Links | Thing::SocketColour(_)
    )
}

/// A socket count as far as it was read: absent where the item has no
/// such count.
fn counted(held: &Held, thing: Thing) -> Counted {
    let Some(groups) = sockets::groups(&held.item) else {
        return Counted::Absent;
    };
    match thing {
        Thing::Sockets => groups.count(),
        Thing::Links => groups.links(),
        Thing::SocketColour(letter) => groups.colour(letter),
        _ => Counted::Absent,
    }
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
                Part::Flags(_)
                | Part::Numbers(_)
                | Part::Sockets
                | Part::SocketGroup
                | Part::SocketColour
                | Part::Class(_)
                | Part::Total(_)
                | Part::Price(_) => false,
            })
            .collect(),
        // the base, or the body, unread leaves the class open; the table's
        // own reason is the class's (`class.rs`)
        (Thing::Class, _) => held
            .item
            .unread
            .iter()
            .filter(|u| match &u.part {
                Part::Body => true,
                Part::Field(key) => key == "baseType",
                _ => false,
            })
            .chain(held.class.open())
            .collect(),
        // the collection, or an element that may be any socket, leaves
        // every count open; a socket's colour only the colour counts, its
        // group only what places sockets in groups (rule 8 of the plan)
        (Thing::Sockets | Thing::Links | Thing::SocketColour(_), _) => held
            .item
            .unread
            .iter()
            .filter(|u| match &u.part {
                Part::Body | Part::Sockets => true,
                Part::SocketColour => matches!(thing, Thing::SocketColour(_)),
                Part::SocketGroup => thing == Thing::Links,
                _ => false,
            })
            .collect(),
        // the price is the listing state's, never the body's: what left it
        // open is its own reason, made once there (`price.rs`)
        (Thing::Price | Thing::PriceAmount | Thing::PriceCurrency | Thing::PriceLot, _) => {
            held.price.unread().into_iter().collect()
        }
        // the deriver says under `reqlevel` when the Level row may have
        // been lost, so a sibling that could not be read leaves it closed
        (Thing::ReqLevel, _) => held
            .item
            .unread
            .iter()
            .filter(|u| match &u.part {
                Part::Body => true,
                Part::Field(key) => key == "reqlevel",
                _ => false,
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
        Thing::Class => held.class.name().into_iter().collect(),
        Thing::League => one(&place.league),
        Thing::Container => one(&place.container),
        Thing::Tab if place.kind == "stash" => place
            .name
            .as_deref()
            .into_iter()
            .chain(place.parent.as_ref().and_then(|p| p.name.as_deref()))
            .collect(),
        Thing::Character if place.kind == "character" => one(&place.name),
        Thing::PriceCurrency => held
            .price
            .price()
            .map(|p| p.currency.as_str())
            .into_iter()
            .collect(),
        Thing::Tab
        | Thing::Character
        | Thing::Text
        | Thing::Ilvl
        | Thing::ReqLevel
        | Thing::Stack
        | Thing::Sockets
        | Thing::Links
        | Thing::SocketColour(_)
        | Thing::Price
        | Thing::PriceAmount
        | Thing::PriceLot
        | Thing::Id => Vec::new(),
    }
}

/// A thing's number, where it is established: a socket count's is its
/// interval's one value (`sockets.rs`).
pub(crate) fn number(held: &Held, thing: Thing) -> Option<f64> {
    match thing {
        Thing::Ilvl => held.item.ilvl.map(|n| n as f64),
        Thing::ReqLevel => held.item.reqlevel.map(|n| n as f64),
        Thing::Stack => held.item.stack.map(|n| n as f64),
        Thing::Sockets | Thing::Links | Thing::SocketColour(_) => {
            counted(held, thing).value().map(|n| n as f64)
        }
        Thing::PriceAmount => held.price.price().map(|p| p.amount),
        Thing::PriceLot => held.price.price().and_then(|p| p.lot).map(|n| n as f64),
        _ => None,
    }
}

/// Whether the item has the thing (C93): a socket count is had while the
/// collection is — a count of zero is a count — and `links` while a link
/// group is known.
fn present(held: &Held, thing: Thing) -> bool {
    match thing {
        Thing::Sockets | Thing::SocketColour(_) => held.item.sockets.is_some(),
        Thing::Links => sockets::groups(&held.item).is_some_and(|g| g.has_group()),
        Thing::Price => held.price.price().is_some(),
        _ => number(held, thing).is_some() || !texts(held, thing).is_empty(),
    }
}

/// Whether the thing's value is not established on the item, as
/// `undecided( … )` asks and the sort marks incomplete: evidence it needs
/// unread — for a socket count, exactly when its interval is more than
/// one number or its collection could not be read (`sockets.rs`).
fn open(held: &Held, thing: Thing) -> bool {
    if is_socket_thing(thing) {
        // a socket whose group is unread need not leave `links` open: alone
        // it is a group of one (`sockets.rs`)
        return match counted(held, thing) {
            Counted::Range { low, high } => low != high,
            Counted::Absent => !unread_for(held, thing).is_empty(),
        };
    }
    !unread_for(held, thing).is_empty()
}

/// What leaves a `linked( … )` open: the collection, a socket's group,
/// and a socket's colour where the group asks a colour.
fn unread_links<'a>(held: &'a Held, linked: &crate::group::Linked) -> Vec<&'a Unread> {
    held.item
        .unread
        .iter()
        .filter(|u| match &u.part {
            Part::Body | Part::Sockets | Part::SocketGroup => true,
            Part::SocketColour => linked.asks_colour(),
            _ => false,
        })
        .collect()
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
            Part::Flags(_)
            | Part::Numbers(_)
            | Part::Properties(_)
            | Part::Sockets
            | Part::SocketGroup
            | Part::SocketColour
            | Part::Class(_)
            | Part::Total(_)
            | Part::Price(_) => false,
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

/// A sum, in units, and whether every possible contributor was readable.
pub(crate) fn sum(held: &Held, group: &Group, slot: &str) -> (Exact, bool) {
    let total = Exact::sum(
        satisfying(held, &group.whole)
            .filter_map(|line| value(line, slot))
            .map(Exact::of),
    );
    let complete =
        unread_lines(held, group).is_empty() && !open_with_the_slot(held, &group.whole, slot);
    (total, complete)
}

/// How many occurrences satisfy the group, in units — a total's row with
/// no slot (`totals.rs`, C94) — and whether every possible one was
/// readable: a source the group admits unread, or an occurrence its flag
/// leaves open, leaves the count open.
pub(crate) fn count(held: &Held, group: &Group) -> (Exact, bool) {
    let n = satisfying(held, &group.whole).count();
    let complete = unread_lines(held, group).is_empty() && !open_on_a_line(held, &group.whole);
    (Exact::of(n as f64), complete)
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
        // a count over the sockets is an interval: decided where the whole
        // interval agrees (`sockets.rs`)
        Atom::Number { thing, test } if is_socket_thing(*thing) => {
            match counted(held, *thing).truth(test) {
                Truth::True => Outcome::Matched,
                Truth::Undecided => Outcome::Undecided,
                Truth::False if present(held, *thing) => Outcome::Failed,
                // absent, or the collection unread
                Truth::False => decided(false, false, open(held, *thing)),
            }
        }
        Atom::Number { thing, test } => {
            let n = number(held, *thing);
            decided(
                n.is_some_and(|n| test.holds(n)),
                n.is_some(),
                !unread_for(held, *thing).is_empty(),
            )
        }
        Atom::Links(linked) => {
            let Some(groups) = sockets::groups(&held.item) else {
                return decided(false, false, !unread_links(held, linked).is_empty());
            };
            // a group GGG numbered is a witness where it holds whatever its
            // unread sockets turn out to be; the group that unplaced sockets
            // may make of their own (`GroupCounts::members` empty) may not
            // exist, so it is never a witness — it leaves the term open
            // where it could hold (the completion property, step 8)
            let each = groups.each();
            let known = |truth: Truth| {
                each.iter()
                    .any(|g| !g.members.is_empty() && linked.of(g) == truth)
            };
            let possible = each
                .iter()
                .any(|g| g.members.is_empty() && linked.of(g) != Truth::False);
            if known(Truth::True) {
                Outcome::Matched
            } else if known(Truth::Undecided) || possible {
                Outcome::Undecided
            } else if groups.has_group() {
                Outcome::Failed
            } else if groups.may_have_group() {
                // no group can hold, and whether there is one is not
                // established: failed or lacked, which is open
                Outcome::Undecided
            } else {
                Outcome::Lacked
            }
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
        // present exactly when the field is established (T2): its inputs
        // read and displayed as numbers; unread inputs leave it open
        Atom::HasComputed(named) => match pseudo::value(*named, None, held) {
            Valued::Value(_) => decided(true, false, false),
            Valued::Incomplete(_) => Outcome::Undecided,
            Valued::Lacked => Outcome::Lacked,
        },
        Atom::Is(flag) => decided(
            // GGG's spelling in any case, as the word was bound (B2)
            held.item.flags.iter().any(|f| f.eq_ignore_ascii_case(flag)),
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
            (total, true) => decided(test.holds(total.as_f64()), true, false),
            (_, false) => Outcome::Undecided,
        },
        Atom::Pseudo {
            named, slot, test, ..
        } => match pseudo::value(*named, slot.as_deref(), held) {
            Valued::Value(n) => decided(test.holds(n.as_f64()), true, false),
            Valued::Incomplete(_) => Outcome::Undecided,
            Valued::Lacked => Outcome::Lacked,
        },
        Atom::Undecided(probe) => {
            let open = match probe {
                BProbe::Field(thing) => open(held, *thing),
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
                key.eq_ignore_ascii_case(flag)
                    || (crate::derive::INFLUENCES.contains(&flag)
                        && (key == "influences"
                            || key
                                .strip_prefix("influences.")
                                .is_some_and(|k| k.eq_ignore_ascii_case(flag))))
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

pub(crate) fn line_evidence(line: &Line) -> Evidence {
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
/// it rests on. The bound is on the parts, the first [`SHOWN`] in the
/// item's own order — a reason made beyond the item's parts (the class
/// table's, a total's, a derived field's) is a part of its own, told from
/// another by what it says, after the item's and ordered by what it says
/// — its part, then its problem — never by the term that met it first,
/// which a rewrite that changes no meaning may reorder (rule 9 of the
/// plan; outside audit, 2026-09-24, twice) — every term's pair with each
/// kept, how many terms a query has being its author's, and the parts
/// past the bound are counted.
pub(crate) fn why<'a>(
    terms: &'a [Term],
    blamed: &[usize],
    held: &Held,
) -> (Vec<(&'a Term, Reason)>, usize) {
    let met: Vec<(usize, Cow<'_, Unread>)> = blamed
        .iter()
        .filter_map(|i| terms.get(*i).map(|term| (*i, term)))
        .flat_map(|(i, term)| {
            unread_of(&term.atom, held)
                .into_iter()
                .map(move |unread| (i, unread))
        })
        .collect();
    let own = |unread: &Unread| {
        held.item
            .unread
            .iter()
            .position(|u| std::ptr::eq(u, unread))
    };
    let key = |unread: &Unread| {
        (
            serde_json::to_string(&unread.part).unwrap_or_default(),
            unread.problem.clone(),
        )
    };
    let mut beyond: Vec<(String, String)> = met
        .iter()
        .filter(|(_, u)| own(u).is_none())
        .map(|(_, u)| key(u))
        .collect();
    beyond.sort();
    beyond.dedup();
    let at = |unread: &Unread| match own(unread) {
        Some(i) => i,
        None => held.item.unread.len() + beyond.binary_search(&key(unread)).unwrap_or(beyond.len()),
    };
    let mut pairs: Vec<(usize, usize, Cow<'_, Unread>)> = met
        .into_iter()
        .map(|(i, unread)| (at(&unread), i, unread))
        .collect();
    pairs.sort_by_key(|(part, term, _)| (*part, *term));
    let mut parts: Vec<usize> = pairs.iter().map(|(part, _, _)| *part).collect();
    parts.dedup();
    let left_out = parts.len().saturating_sub(SHOWN);
    let last = parts.get(SHOWN.saturating_sub(1)).copied();
    let shown = pairs
        .into_iter()
        .filter(|(part, _, _)| last.is_none_or(|last| *part <= last))
        .map(|(_, i, unread)| (&terms[i], reason(&unread)))
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
        // a price term shows the price as the listing state prints it
        Atom::Number { thing, .. } | Atom::Closed { thing, .. } | Atom::Has(thing)
            if is_price_thing(*thing) =>
        {
            crate::price::evidence(held)
        }
        // a socket count shows its layout beside the number
        Atom::Number { thing, .. } if is_socket_thing(*thing) => {
            let name = match &term.node {
                crate::tree::Node::Test { field, .. } => field.clone(),
                other => crate::print::print(other),
            };
            let value = match counted(held, *thing) {
                Counted::Absent => None,
                Counted::Range { low, high } if low == high => Some(number_json(low as f64)),
                Counted::Range { low, high } => {
                    Some(serde_json::Value::from(format!("{low}..{high}")))
                }
            };
            value
                .map(|value| Evidence::Value { name, value })
                .into_iter()
                .chain(sockets::layout(&held.item).map(|text| Evidence::Shown {
                    part: "sockets".to_string(),
                    text,
                }))
                .collect()
        }
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
        // the link groups that hold, as the game shows each
        Atom::Links(linked) => match sockets::groups(&held.item) {
            Some(groups) => groups
                .each()
                .iter()
                .filter(|g| linked.of(g) == Truth::True && !g.members.is_empty())
                .map(|g| Evidence::Shown {
                    part: "link group".to_string(),
                    text: groups.group_text(&g.members),
                })
                .collect(),
            None => Vec::new(),
        },
        // the class is not on the row's header: a term on it shows it
        Atom::Closed {
            thing: Thing::Class,
            ..
        } => texts(held, Thing::Class)
            .into_iter()
            .map(|class| Evidence::Value {
                name: "class".to_string(),
                value: serde_json::Value::from(class),
            })
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
                value: number_json(total.as_f64()),
            })
            .chain(satisfying(held, &group.whole).map(line_evidence))
            .collect()
        }
        Atom::Pseudo {
            named, name, slot, ..
        } => {
            let value = match pseudo::value(*named, slot.as_deref(), held) {
                Valued::Value(n) | Valued::Incomplete(Some(n)) => number_json(n.as_f64()),
                Valued::Incomplete(None) | Valued::Lacked => serde_json::Value::Null,
            };
            std::iter::once(Evidence::Value {
                name: name.clone(),
                value,
            })
            .chain(pseudo::evidence(*named, held))
            .collect()
        }
        // what the field read, as its comparison shows it
        Atom::HasComputed(named) => pseudo::evidence(*named, held),
        _ => Vec::new(),
    }
}

/// What a sum over a group's occurrences rests on that was unread: a
/// source the group admits, and what an occurrence itself left open,
/// explained by that occurrence and by what was asked of it — its flags
/// where a flag is asked, its numbers where a number is (`slot`), or
/// where the group is more than a selector.
pub(crate) fn unread_of_sum<'a>(
    held: &'a Held,
    group: &Group,
    slot: Option<&str>,
) -> Vec<&'a Unread> {
    let mut unread = unread_lines(held, group);
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
}

/// What left a term open on an item: the item's own unread parts, borrowed,
/// and a reason made where the evidence is read — a total with no
/// definition for the realm, a property displayed as no number
/// (`pseudo.rs`).
fn unread_of<'a>(atom: &Atom, held: &'a Held) -> Vec<Cow<'a, Unread>> {
    let borrowed = |unread: Vec<&'a Unread>| unread.into_iter().map(Cow::Borrowed).collect();
    match atom {
        Atom::Text { thing, .. }
        | Atom::Closed { thing, .. }
        | Atom::Number { thing, .. }
        | Atom::Has(thing)
        | Atom::Undecided(BProbe::Field(thing)) => borrowed(unread_for(held, *thing)),
        Atom::Is(flag) => borrowed(unread_flag(held, flag)),
        Atom::Links(linked) => borrowed(unread_links(held, linked)),
        Atom::Lines(group) => borrowed(unread_of_sum(held, group, None)),
        Atom::Sum { group, slot, .. } => borrowed(unread_of_sum(held, group, Some(slot.as_str()))),
        Atom::Pseudo { named, .. } | Atom::HasComputed(named) => pseudo::unread_of(*named, held),
        Atom::Undecided(BProbe::Value(key)) => match key.as_ref() {
            SortKey::Projection { group, slot } | SortKey::Sum { group, slot } => {
                borrowed(unread_of_sum(held, group, Some(slot.as_str())))
            }
            SortKey::Number(thing) => borrowed(unread_for(held, *thing)),
            SortKey::Pseudo { named, .. } => pseudo::unread_of(*named, held),
        },
        Atom::Id(_) | Atom::Const(_) | Atom::Undecided(BProbe::Term(_)) => Vec::new(),
    }
}

/// What left a term open on an item, by what was unread: the kinds the
/// counts view tallies beneath an `undecided` bucket (C105). The reasons
/// themselves stay on the item.
pub(crate) fn unread_kinds(atom: &Atom, held: &Held) -> Vec<String> {
    let mut kinds: Vec<String> = unread_of(atom, held)
        .into_iter()
        .map(|unread| reason(&unread).unread)
        .collect();
    kinds.sort();
    kinds.dedup();
    kinds
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
            Part::Sockets => "the sockets".to_string(),
            Part::SocketGroup => "a socket's group".to_string(),
            Part::SocketColour => "a socket's colour".to_string(),
            Part::Class(gap) => gap.unread().to_string(),
            Part::Total(gap) => gap.unread().to_string(),
            Part::Price(gap) => gap.unread().to_string(),
        },
        problem: unread.problem.clone(),
        hint: match &unread.part {
            Part::Class(_) => crate::class::CLASS_HINT,
            Part::Total(_) => crate::totals::TOTAL_HINT,
            Part::Price(_) => crate::price::PRICE_HINT,
            _ => UNREAD_HINT,
        },
    }
}

/// What an item sorts by, and what a count sums (`counts.rs`): in units,
/// so that a total is added again exactly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Scalar {
    Value(Exact),
    /// No satisfying occurrence, or the item lacks the field.
    None,
    /// What was readable, which is not the answer: a field that could not
    /// be read; a sum with a possible
    /// contributor unread is a subtotal, never a total, and a largest
    /// occurrence is not the largest while an unread source the group
    /// admits, or an occurrence it may select, could hold a larger one. No
    /// place in the order.
    Incomplete(Option<Exact>),
}

pub(crate) fn scalar(key: &SortKey, held: &Held) -> Scalar {
    match key {
        SortKey::Number(thing) if is_socket_thing(*thing) => match counted(held, *thing) {
            Counted::Absent if open(held, *thing) => Scalar::Incomplete(None),
            Counted::Absent => Scalar::None,
            Counted::Range { low, high } if low == high => Scalar::Value(Exact::of(low as f64)),
            // what was read, which is not the answer
            Counted::Range { low, .. } => Scalar::Incomplete(Some(Exact::of(low as f64))),
        },
        SortKey::Number(thing) => match number(held, *thing) {
            Some(n) => Scalar::Value(Exact::of(n)),
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
            let largest = largest.map(Exact::of);
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
        SortKey::Pseudo { named, slot, .. } => match pseudo::value(*named, slot.as_deref(), held) {
            Valued::Value(n) => Scalar::Value(n),
            Valued::Incomplete(n) => Scalar::Incomplete(n),
            Valued::Lacked => Scalar::None,
        },
    }
}
