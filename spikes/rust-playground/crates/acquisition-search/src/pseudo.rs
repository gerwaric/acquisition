//! Computed values (the reference, *Values*; C94, C101; the build plan,
//! steps 7 and 9c3): `pseudo.<name>`, one namespace for the named totals
//! of the table (`totals.rs`), the readings of them and the derived fields
//! — what a comparison, a sort, a sum or `undecided( … )` consumes when it
//! names one, and what a row shows of it.
//!
//! The words are `search/DESIGN.md`'s reference (*Values*, *A sum's
//! status*); the rules that decide a computed value are this doc's and
//! `totals.rs`'s (C94, C101; taken from the contract detail 2026-09-23).
//!
//! # Decisions as recorded
//!
//! - **C101 — the derived fields.** A missing reach is a field to add,
//!   never a general door: a pseudo that is no sum over lines is a named
//!   pure function over the item's properties, listed with its definition
//!   under `pseudo.` by `--describe` (C97), never a total row. Built here:
//!   `pseudo.dps` and `pseudo.pdps`, each attacks per second times the
//!   mean of a damage range (S22), read from the properties as the game
//!   displays them — `Physical Damage: 59-88`, `Attacks per Second: 1.25`
//!   — quality applied as GGG has already applied it to what it displays.
//!   `pdps` is the physical range's mean times the attacks per second;
//!   `dps` is the sum of the physical, every elemental and the chaos
//!   range's means, times the attacks per second. An item with no
//!   `Attacks per Second` property, or none of the damage properties a
//!   field reads, *lacks* the field — known absence, false and never zero
//!   (C93) — where a property the field reads is unread, or is displayed
//!   as no number and no range, or its product needs more decimals than
//!   the units hold (`exact::Exact::times`), the field is undecided with
//!   that reason. An unread element of `properties` leaves a field open
//!   only when it may be one the field reads: its name unread, or one of
//!   the field's (`derive::Unread::name`; rule 8 at the element's grain,
//!   outside audit 2026-09-24).
//!   `has:` asks a computed value's presence, a derived field's or a
//!   total's: `-has:pseudo.dps` is the lacked count's route (owner,
//!   2026-09-24, T2 in `SEARCH-SLICE.md`: "A ring has no dps"), and
//!   `-has:pseudo.total_res` a total's, since a total of nothing is
//!   lacked (`totals.rs`, C94 as ruled 2026-09-26, which takes back
//!   T2's "every item has a total").
//!   The base defence percentile the same paragraph names is not built
//!   (`bind::NOT_BUILT`, its formula unpinned: `search/pseudo-stats/README.md`,
//!   open question 3).
//! - **C101 — the readings of other totals** (the build plan, step 9c3;
//!   owner, 2026-09-26: "They should be present at launch."). Four of the
//!   trade site's pseudos are no sum of lines and so no total: each reads
//!   totals, as the table names them (`totals::Reading`), and what it
//!   reads is each total as this search counts it — the rows the owner
//!   ruled counted where the site leaves them out among them. A *count*
//!   (`pseudo.count_res`, `pseudo.count_ele_res`) is how many of its
//!   totals the item shows: a total below nothing is shown and counts, a
//!   total of nothing is not, and a count of none is lacked, as the site
//!   shows no count there. A *least* (`pseudo.total_all_ele_res`,
//!   `pseudo.total_all_attributes`) is the smallest of its totals on an
//!   item that shows every one, with the line that names them all or
//!   without it, and lacked where one of them is nothing. What the
//!   captures hold of each is `search/pseudo-stats/README.md`.
//! - **C93, C94 — a reading where a total it reads is open** (rule 8 of
//!   the plan: unknown of no more than was lost). A count of totals is a
//!   count of three-valued things, which C93 rules for at-least-N-of: the
//!   totals established are its floor and each open one widens its
//!   ceiling by one, a comparison is decided where the whole interval
//!   agrees, and `pseudo.count_res>=2` answers as `holds( … )>=2` over
//!   `has:` of its totals does. The count is had where one total is
//!   established, so `has:` of it is decided there; its value is
//!   established when the interval is one number, which is when it sorts
//!   and sums as a value and `undecided( … )` of it is false — a socket
//!   count's rule (`sockets.rs`). With no total established and one open
//!   the count may be none, and every term on it is undecided — where
//!   `holds` is false of a bound it cannot reach, the count's term is
//!   failed of a count and lacked of none, which is open as a link
//!   group's is (`eval.rs`). A least is
//!   a number read from sums: lacked where one of its totals is known to
//!   be nothing, whatever is open beside it; otherwise open while one is,
//!   with the least of those established as what was readable, and a
//!   comparison on it undecided whatever that already reaches, as a
//!   sum's is. In a realm the table does not cover a reading is
//!   unavailable as a total is, with the table's one reason.
//!
//! # As built
//!
//! - **A name is the table's — a total's or a reading's — or a derived
//!   field's**, matched in any case
//!   (B1); an unknown one is an authoring error with the near names
//!   offered, as a field's is — unless it carries a slot word, which asks
//!   for a ranged total, and this build ships none (`bind::NOT_BUILT`).
//!   A slot word on a total that is no range, on a reading or on a
//!   derived field, is an authoring error; a ranged total asked for with
//!   none is one that offers `low`, `high` and `avg`.
//! - **A value has the sum's three statuses** ([`Valued`]; the reference,
//!   *A sum's status*): complete, an incomplete subtotal, or lacked — a
//!   derived field whose input the item lacks, a total whose lines sum
//!   to nothing. Unavailable, a total with no definition for the
//!   item's realm, is incomplete with nothing readable, as a field that
//!   could not be read is; its reason is the table's, made once. A value
//!   is open exactly when it would sort as incomplete, so `undecided( … )`,
//!   the sort scalar and a count's sum ask one function (`eval::scalar`).
//!   What a comparison and `has:` make of a value is said here once
//!   ([`compared`], [`present`]), a count left open among it.
//! - **A reading's totals are the table's own** ([`total_of`]): one maker
//!   of a total (rule 10 of the plan), so a reading and the totals it
//!   reads cannot disagree. It costs the rows of every total it reads —
//!   a least stops at the first that is nothing — which is the totals
//!   batch park's to lower (`decisions/search.md`).
//! - **A total's arithmetic is the evaluator's sum** (`eval::sum`, and
//!   `eval::count` for a row without a slot), weighted in halves
//!   (`exact::Exact::halved`), so a total and a `sum( … )` over the same
//!   lines cannot disagree (rule 10 of the plan); a ranged total's `avg` is
//!   the mean of its two totals, exact.
//! - **What a row shows** of a computed value: its value, then the lines
//!   the total's rows named on the item, the totals a reading read, each
//!   by its name with its value, or the properties the derived field
//!   read, bounded as every term's evidence is (`eval::evidence`). A count
//!   left open prints its interval, `2..3`, as a socket count does. A
//!   row sorted by a reading shows it first by its own name
//!   ([`sorted_by`]).
//! - **The vocabulary lists a computed value beside the templates** whose
//!   name or definition its narrowing matches, and none under `line`
//!   alone, where `--describe` names them (owner, 2026-09-24, T5 in
//!   `SEARCH-SLICE.md`; the reference, `--count line[:text]`;
//!   `counts.rs`), marked computed, with the count of the matches that
//!   carry it — as `has:` of it asks ([`present`]), which a lacked one and
//!   an incomplete one do not — routed by `has:pseudo.<name>`, exactly
//!   those.
//!   Why it is open is the open contributors' own reasons (rule 8): a
//!   source a row admits unread, an occurrence whose number or flag is,
//!   the property that could not be read; a reading's are those of the
//!   totals that leave it open.

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::bind::NumTest;
use crate::corpus::Held;
use crate::derive::{Part, Property, Unread};
use crate::describe::Named as Entry;
use crate::error::{ErrorKind, LanguageError};
use crate::eval::{self, Evidence, Outcome, Truth};
use crate::exact::{self, Exact};
use crate::sockets::Counted;
use crate::totals::{self, Reading, ReadingKind, Total, TotalsTable};
use crate::tree::{Node, Op, Value};

/// A computed value, bound by name.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Named {
    Total {
        table: &'static TotalsTable,
        total: &'static Total,
    },
    /// A reading of other totals (`totals::Reading`).
    Reading {
        table: &'static TotalsTable,
        reading: &'static Reading,
    },
    Derived(Derived),
}

/// The derived fields (C101): each a named pure function of the item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Derived {
    Dps,
    Pdps,
}

impl Derived {
    const ALL: [Derived; 2] = [Derived::Dps, Derived::Pdps];

    pub fn name(self) -> &'static str {
        match self {
            Derived::Dps => "dps",
            Derived::Pdps => "pdps",
        }
    }

    pub fn definition(self) -> &'static str {
        match self {
            Derived::Dps => {
                "damage per second: the attacks per second times the mean of every damage range the item displays — physical, each elemental, chaos — as `Physical Damage`, `Elemental Damage`, `Chaos Damage` and `Attacks per Second` show them, quality already applied by the game; an item with no attacks per second, or no damage range, lacks it"
            }
            Derived::Pdps => {
                "physical damage per second: the attacks per second times the mean of `Physical Damage`; an item lacking either property lacks it"
            }
        }
    }
}

impl Named {
    pub fn ranged(self) -> bool {
        matches!(self, Named::Total { total, .. } if total.ranged)
    }
}

/// A computed value by name, in any case; none for a name nothing defines.
/// A build whose table does not load says so rather than knowing no total.
pub(crate) fn lookup(name: &str) -> Result<Option<Named>, LanguageError> {
    if let Some(derived) = Derived::ALL
        .into_iter()
        .find(|d| d.name().eq_ignore_ascii_case(name))
    {
        return Ok(Some(Named::Derived(derived)));
    }
    let table = table()?;
    if let Some(total) = table.get(name) {
        return Ok(Some(Named::Total { table, total }));
    }
    Ok(table
        .reading(name)
        .map(|reading| Named::Reading { table, reading }))
}

fn table() -> Result<&'static TotalsTable, LanguageError> {
    totals::table().map_err(|e| {
        LanguageError::new(
            ErrorKind::Tree,
            format!("the totals table this build ships does not load: {e}"),
        )
    })
}

/// Every name, the totals in the table's order, its readings, then the
/// derived fields: what a near-name suggestion is scored against. Leaked
/// once from the shipped table, as the class names are (`class::names`).
static NAMES: LazyLock<&'static [&'static str]> = LazyLock::new(|| {
    let mut names: Vec<&'static str> = totals::table()
        .map(|t| {
            t.totals()
                .iter()
                .map(|t| &t.name)
                .chain(t.readings().iter().map(|r| &r.name))
                .map(|name| &*Box::leak(name.clone().into_boxed_str()))
                .collect()
        })
        .unwrap_or_default();
    names.extend(Derived::ALL.iter().map(|d| d.name()));
    Box::leak(names.into_boxed_slice())
});

pub(crate) fn names() -> &'static [&'static str] {
    &NAMES
}

/// The value of a computed value on one item (the module doc).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Valued {
    Value(Exact),
    /// A subtotal, never a total; none readable where the total is
    /// unavailable or a derived field's input could not be read.
    Incomplete(Option<Exact>),
    /// Known absence: a derived field whose input the item lacks, a
    /// total whose lines sum to nothing — no line a row names, or lines
    /// that cancel (C94) — a count of no total, a least one of whose
    /// totals is nothing.
    Lacked,
}

pub(crate) fn value(named: Named, slot: Option<&str>, held: &Held) -> Valued {
    read(named, slot, held).0
}

/// A computed value on one item, and the interval a count lies in where
/// it is had and not established: at least one total shown, and more
/// that may be.
fn read(named: Named, slot: Option<&str>, held: &Held) -> (Valued, Option<(usize, usize)>) {
    match named {
        Named::Total { table, total } => (total_of(table, total, slot, held), None),
        Named::Reading { table, reading } => match reading.kind {
            ReadingKind::Count => count_of(table, reading, held),
            ReadingKind::Least => (least_of(table, reading, held), None),
        },
        Named::Derived(derived) => (derived_of(derived, held), None),
    }
}

/// A comparison on a computed value (the module doc): decided on a value,
/// undecided on one left open — but a count, decided where the whole of
/// its interval agrees — and lacked where the item lacks it.
pub(crate) fn compared(named: Named, slot: Option<&str>, test: &NumTest, held: &Held) -> Outcome {
    match read(named, slot, held) {
        (_, Some((low, high))) => match (Counted::Range { low, high }).truth(test) {
            Truth::True => Outcome::Matched,
            Truth::False => Outcome::Failed,
            Truth::Undecided => Outcome::Undecided,
        },
        (Valued::Value(n), _) if test.holds(n.as_f64()) => Outcome::Matched,
        (Valued::Value(_), _) => Outcome::Failed,
        (Valued::Incomplete(_), _) => Outcome::Undecided,
        (Valued::Lacked, _) => Outcome::Lacked,
    }
}

/// Whether the item has the computed value, as `has:` of it asks: its
/// value established — a derived field's inputs read and displayed as
/// numbers (T2), a total's lines summing to something (C94) — or a count
/// one of whose totals is; what is unread leaves it open.
pub(crate) fn present(named: Named, held: &Held) -> Outcome {
    match read(named, named.ranged().then_some("avg"), held) {
        (Valued::Value(_), _) | (_, Some(_)) => Outcome::Matched,
        (Valued::Incomplete(_), _) => Outcome::Undecided,
        (Valued::Lacked, _) => Outcome::Lacked,
    }
}

/// A computed value as a row prints it beside its name: the number, what
/// was readable of one left open — a count's interval, `2..3` — or
/// nothing.
pub(crate) fn printed(named: Named, slot: Option<&str>, held: &Held) -> serde_json::Value {
    match read(named, slot, held) {
        (_, Some((low, high))) => serde_json::Value::from(format!("{low}..{high}")),
        (Valued::Value(n) | Valued::Incomplete(Some(n)), _) => eval::number_json(n.as_f64()),
        (Valued::Incomplete(None) | Valued::Lacked, _) => serde_json::Value::Null,
    }
}

/// What leaves a computed value open on this item, each contributor's own
/// reason (rule 8 of the plan).
pub(crate) fn unread_of(named: Named, held: &Held) -> Vec<Cow<'_, Unread>> {
    match named {
        Named::Total { table, total } => {
            if let Some(reason) = table.unavailable(&held.item.facts.realm) {
                return vec![Cow::Borrowed(reason)];
            }
            let mut unread: Vec<&Unread> = Vec::new();
            unread_of_total(total, held, &mut unread);
            unread.into_iter().map(Cow::Borrowed).collect()
        }
        // what leaves its totals open, and nothing where it is decided
        // all the same: a least one of whose totals is nothing
        Named::Reading { table, reading } => {
            if let Some(reason) = table.unavailable(&held.item.facts.realm) {
                return vec![Cow::Borrowed(reason)];
            }
            if !matches!(value(named, None, held), Valued::Incomplete(_)) {
                return Vec::new();
            }
            let mut unread: Vec<&Unread> = Vec::new();
            for total in table.read_by(reading) {
                unread_of_total(total, held, &mut unread);
            }
            unread.into_iter().map(Cow::Borrowed).collect()
        }
        Named::Derived(derived) => match inputs(derived, held) {
            Ok(_) | Err(Inputs::Lacked) => Vec::new(),
            Err(Inputs::Unread(unread)) => unread,
        },
    }
}

/// What leaves a total open on this item, each part once.
fn unread_of_total<'a>(total: &Total, held: &'a Held, unread: &mut Vec<&'a Unread>) {
    for row in &total.rows {
        let slots: &[Option<&str>] = if total.ranged {
            &[Some("low"), Some("high")]
        } else {
            &[row.slot.as_deref()]
        };
        for slot in slots {
            for u in eval::unread_of_sum(held, &row.group, *slot) {
                if !unread.iter().any(|seen| std::ptr::eq(*seen, u)) {
                    unread.push(u);
                }
            }
        }
    }
}

/// What a row shows of a computed value that matched: the lines the
/// total's rows named on the item, the totals a reading read that the
/// item shows, or the properties the derived field read.
pub(crate) fn evidence(named: Named, held: &Held) -> Vec<Evidence> {
    match named {
        Named::Total { total, .. } => {
            let mut lines: Vec<&crate::derive::Line> = Vec::new();
            for row in &total.rows {
                for line in eval::selected(held, &row.group) {
                    if !lines.iter().any(|seen| std::ptr::eq(*seen, line)) {
                        lines.push(line);
                    }
                }
            }
            lines.sort_by_key(|line| held.item.lines.iter().position(|l| std::ptr::eq(l, *line)));
            lines.into_iter().map(eval::line_evidence).collect()
        }
        Named::Reading { table, reading } => table
            .read_by(reading)
            .filter_map(|total| match total_of(table, total, None, held) {
                Valued::Value(n) => Some(Evidence::Value {
                    name: format!("pseudo.{}", total.name),
                    value: eval::number_json(n.as_f64()),
                }),
                Valued::Incomplete(_) | Valued::Lacked => None,
            })
            .collect(),
        Named::Derived(derived) => match inputs(derived, held) {
            Ok(read) => read
                .shown
                .into_iter()
                .map(|p| Evidence::Shown {
                    part: p.array.clone(),
                    text: p.text.clone(),
                })
                .collect(),
            Err(_) => Vec::new(),
        },
    }
}

/// What a row shows of a computed value it is sorted by (F6, the first
/// seat): what [`evidence`] gives — and a reading first by its own name
/// with its value, as a field is, since what it read is named values too
/// and the first of a sort's is taken for the sort's own. One that has
/// nothing to print shows nothing.
pub(crate) fn sorted_by(named: Named, slot: Option<&str>, held: &Held) -> Vec<Evidence> {
    let Named::Reading { reading, .. } = named else {
        return evidence(named, held);
    };
    match printed(named, slot, held) {
        serde_json::Value::Null => Vec::new(),
        value => std::iter::once(Evidence::Value {
            name: format!("pseudo.{}", reading.name),
            value,
        })
        .chain(evidence(named, held))
        .collect(),
    }
}

// ---- totals ---------------------------------------------------------------------------------------

fn total_of(table: &TotalsTable, total: &Total, slot: Option<&str>, held: &Held) -> Valued {
    if table.unavailable(&held.item.facts.realm).is_some() {
        return Valued::Incomplete(None);
    }
    let mut complete = true;
    let mut row_sum = |slot: Option<&str>, row: &totals::Row| -> Exact {
        let (value, whole) = match slot {
            Some(slot) => eval::sum(held, &row.group, slot),
            None => eval::count(held, &row.group),
        };
        complete &= whole;
        value.halved(row.halves)
    };
    let nothing = Exact::default();
    let (value, is_nothing) = if total.ranged {
        let low = Exact::sum(total.rows.iter().map(|row| row_sum(Some("low"), row)));
        let high = Exact::sum(total.rows.iter().map(|row| row_sum(Some("high"), row)));
        let value = match slot {
            Some("low") => low,
            Some("high") => high,
            _ => low.mid(high),
        };
        (value, low == nothing && high == nothing)
    } else {
        let value = Exact::sum(
            total
                .rows
                .iter()
                .map(|row| row_sum(row.slot.as_deref(), row)),
        );
        (value, value == nothing)
    };
    if !complete {
        // what could not be read may be what makes it something
        Valued::Incomplete(Some(value))
    } else if is_nothing {
        Valued::Lacked
    } else {
        Valued::Value(value)
    }
}

// ---- readings of other totals -------------------------------------------------------------------------

/// A count (the module doc): how many of its totals the item shows, and
/// the interval it lies in where one is shown and another is open.
fn count_of(
    table: &TotalsTable,
    reading: &Reading,
    held: &Held,
) -> (Valued, Option<(usize, usize)>) {
    if table.unavailable(&held.item.facts.realm).is_some() {
        return (Valued::Incomplete(None), None);
    }
    let (mut low, mut high) = (0, 0);
    for total in table.read_by(reading) {
        match total_of(table, total, None, held) {
            Valued::Value(_) => {
                low += 1;
                high += 1;
            }
            // what could not be read may be what makes it something
            Valued::Incomplete(_) => high += 1,
            Valued::Lacked => {}
        }
    }
    let shown = Exact::of(low as f64);
    match (low, high) {
        (_, 0) => (Valued::Lacked, None),
        (low, high) if low == high => (Valued::Value(shown), None),
        // none established: the count may be none
        (0, _) => (Valued::Incomplete(None), None),
        (low, high) => (Valued::Incomplete(Some(shown)), Some((low, high))),
    }
}

/// A least (the module doc): the smallest of its totals where the item
/// shows every one. A total of nothing decides it, so the reading stops
/// there.
fn least_of(table: &TotalsTable, reading: &Reading, held: &Held) -> Valued {
    if table.unavailable(&held.item.facts.realm).is_some() {
        return Valued::Incomplete(None);
    }
    let mut least: Option<Exact> = None;
    let mut complete = true;
    for total in table.read_by(reading) {
        match total_of(table, total, None, held) {
            Valued::Value(n) => least = Some(least.map_or(n, |least| least.min(n))),
            Valued::Incomplete(_) => complete = false,
            Valued::Lacked => return Valued::Lacked,
        }
    }
    match (complete, least) {
        (true, Some(least)) => Valued::Value(least),
        (true, None) => Valued::Lacked,
        (false, least) => Valued::Incomplete(least),
    }
}

// ---- derived fields -----------------------------------------------------------------------------------

/// The properties a derived field reads, by the name the game displays.
const ATTACKS_PER_SECOND: &str = "Attacks per Second";
const PHYSICAL: &str = "Physical Damage";
const ELEMENTAL: &str = "Elemental Damage";
const CHAOS: &str = "Chaos Damage";

/// What a derived field read on an item.
struct Read<'a> {
    value: Exact,
    /// The properties read, in the item's order.
    shown: Vec<&'a Property>,
}

/// Why a derived field has no value.
enum Inputs<'a> {
    Lacked,
    Unread(Vec<Cow<'a, Unread>>),
}

fn derived_of(derived: Derived, held: &Held) -> Valued {
    match inputs(derived, held) {
        Ok(read) => Valued::Value(read.value),
        Err(Inputs::Lacked) => Valued::Lacked,
        Err(Inputs::Unread(_)) => Valued::Incomplete(None),
    }
}

/// The properties a derived field reads.
fn reads(derived: Derived) -> &'static [&'static str] {
    match derived {
        Derived::Pdps => &[ATTACKS_PER_SECOND, PHYSICAL],
        Derived::Dps => &[ATTACKS_PER_SECOND, PHYSICAL, ELEMENTAL, CHAOS],
    }
}

/// The one reading of a derived field's inputs: what was unread of the
/// properties leaves the field open where it may be a property the field
/// reads — the array itself, an element with no readable name, or one
/// named as the field's — and a property displayed as no number and no
/// range, or whose product the units cannot hold, is unread under
/// `properties`, said here.
fn inputs(derived: Derived, held: &Held) -> Result<Read<'_>, Inputs<'_>> {
    let item = &held.item;
    let properties = Part::Properties("properties".to_string());
    let unread: Vec<Cow<'_, Unread>> = item
        .unread
        .iter()
        .filter(|u| {
            u.part == Part::Body
                || (u.part == properties
                    && u.name.as_deref().is_none_or(|name| {
                        reads(derived).iter().any(|r| r.eq_ignore_ascii_case(name))
                    }))
        })
        .map(Cow::Borrowed)
        .collect();
    if !unread.is_empty() {
        return Err(Inputs::Unread(unread));
    }
    let property = |name: &str| {
        item.properties
            .iter()
            .find(|p| p.array == "properties" && p.name.eq_ignore_ascii_case(name))
    };
    let mut problems: Vec<Cow<'_, Unread>> = Vec::new();
    let mut problem = |p: &Property, what: &str| {
        problems.push(Cow::Owned(Unread {
            part: properties.clone(),
            problem: format!("`{}` is `{}`: {what}", p.name, p.values.join(", ")),
            line: None,
            name: Some(p.name.clone()),
            socket: None,
        }));
    };
    // the mean of a damage property: the sum of its ranges' means
    let mean_of = |p: &Property, problem: &mut dyn FnMut(&Property, &str)| -> Option<Exact> {
        let mut means = Vec::with_capacity(p.values.len());
        for value in &p.values {
            match range(value) {
                Some((low, high)) => means.push(low.mid(high)),
                None => {
                    problem(p, "no number and no range of two the search reads");
                    return None;
                }
            }
        }
        if means.is_empty() {
            problem(p, "no value");
            return None;
        }
        Some(Exact::sum(means))
    };
    let aps = property(ATTACKS_PER_SECOND);
    let damage: Vec<&Property> = match derived {
        Derived::Pdps => property(PHYSICAL).into_iter().collect(),
        Derived::Dps => [PHYSICAL, ELEMENTAL, CHAOS]
            .into_iter()
            .filter_map(property)
            .collect(),
    };
    let (Some(aps), false) = (aps, damage.is_empty()) else {
        return Err(Inputs::Lacked);
    };
    let speed = match aps.values.as_slice() {
        [one] => match number(one) {
            Some(n) => Some(n),
            None => {
                problem(aps, "no number the search reads");
                None
            }
        },
        _ => {
            problem(aps, "not one value");
            None
        }
    };
    let means: Vec<Option<Exact>> = damage.iter().map(|p| mean_of(p, &mut problem)).collect();
    let product = match (speed, means.into_iter().collect::<Option<Vec<Exact>>>()) {
        (Some(speed), Some(means)) => {
            let mean = Exact::sum(means);
            let product = mean.times(speed);
            if product.is_none() {
                problem(
                    aps,
                    &format!(
                        "times the damage's mean {}, a product of more decimals than the search reads",
                        mean.as_f64()
                    ),
                );
            }
            product
        }
        _ => None,
    };
    if !problems.is_empty() {
        return Err(Inputs::Unread(problems));
    }
    let Some(value) = product else {
        return Err(Inputs::Unread(Vec::new()));
    };
    let mut shown: Vec<&Property> = std::iter::once(aps).chain(damage).collect();
    shown.sort_by_key(|p| item.properties.iter().position(|q| std::ptr::eq(q, *p)));
    Ok(Read { value, shown })
}

// ---- the vocabulary -----------------------------------------------------------------------------------

/// The computed values a vocabulary narrowing matches by name or
/// definition — none when it narrows by nothing (the module doc) — each
/// with the term that selects the items carrying it.
pub(crate) fn matching(
    narrow: Option<&(Op, String)>,
) -> Result<Vec<(String, Named, Node)>, LanguageError> {
    let Some((op, text)) = narrow else {
        return Ok(Vec::new());
    };
    let test = crate::bind::text_test("line", *op, &Value::Text(text.clone()))?;
    let mut out = Vec::new();
    for entry in DESCRIBED.iter() {
        if !(test.holds(&entry.name) || test.holds(&entry.what)) {
            continue;
        }
        let name = entry.name.trim_start_matches("pseudo.");
        let Some(named) = lookup(name)? else {
            continue;
        };
        out.push((entry.name.clone(), named, Node::Has(entry.name.clone())));
    }
    Ok(out)
}

/// Whether the item carries the computed value, as `has:` of it asks.
pub(crate) fn carried(named: Named, held: &Held) -> bool {
    present(named, held) == Outcome::Matched
}

/// A displayed number the search reads, or none.
fn number(text: &str) -> Option<Exact> {
    let text = text.trim();
    if !exact::reads(text) {
        return None;
    }
    text.parse::<f64>()
        .ok()
        .filter(|n| n.is_finite())
        .map(Exact::of)
}

/// A displayed range, `59-88`, or one number standing for both ends; the
/// dash is the range's, since a displayed damage is never negative.
fn range(text: &str) -> Option<(Exact, Exact)> {
    let text = text.trim();
    let (low, high) = text.split_once('-').unwrap_or((text, text));
    Some((number(low)?, number(high)?))
}

// ---- the help -----------------------------------------------------------------------------------------

static DESCRIBED: LazyLock<Vec<Entry>> = LazyLock::new(|| {
    let leak = |s: String| -> &'static str { Box::leak(s.into_boxed_str()) };
    let mut out: Vec<Entry> = Vec::new();
    if let Ok(table) = totals::table() {
        for total in table.totals() {
            let name = format!("pseudo.{}", total.name);
            let examples = if total.ranged {
                vec![
                    leak(format!("{name}.avg>=30")),
                    leak(format!("-has:{name}")),
                    leak(format!("undecided({name})")),
                ]
            } else {
                vec![
                    leak(format!("{name}>=60")),
                    leak(format!("-has:{name}")),
                    leak(format!("undecided({name})")),
                ]
            };
            out.push(Entry {
                name,
                kind: if total.ranged {
                    "ranged total"
                } else {
                    "total"
                },
                what: total.definition(),
                values: Vec::new(),
                examples,
            });
        }
        // a reading is a derived field (C101), defined by the table
        for reading in table.readings() {
            let name = format!("pseudo.{}", reading.name);
            let bound = match reading.kind {
                ReadingKind::Count => 3,
                ReadingKind::Least => 10,
            };
            out.push(Entry {
                examples: vec![
                    leak(format!("{name}>={bound}")),
                    leak(format!("-has:{name}")),
                    leak(format!("undecided({name})")),
                ],
                name,
                kind: "derived",
                what: reading.definition(),
                values: Vec::new(),
            });
        }
    }
    for derived in Derived::ALL {
        let name = format!("pseudo.{}", derived.name());
        out.push(Entry {
            examples: vec![
                leak(format!("{name}>=300")),
                leak(format!("-has:{name}")),
                leak(format!("undecided({name})")),
            ],
            name,
            kind: "derived",
            what: derived.definition().to_string(),
            values: Vec::new(),
        });
    }
    out
});

/// The computed values as `--describe` lists them (C97): every total with
/// its definition, the readings of them, then the derived fields.
pub(crate) fn described() -> Vec<Entry> {
    DESCRIBED.clone()
}

/// What `--describe` says of the totals table as a whole: its version and
/// source (C106's clause (c)), or why it does not load.
pub(crate) fn provenance() -> String {
    match totals::table() {
        Ok(table) => table.provenance(),
        Err(e) => format!("the totals table this build ships does not load: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Derived, Named, Valued, value};
    use crate::class::Classed;
    use crate::corpus::{Held, Place};
    use crate::derive::{Facts, derive};
    use crate::totals::TotalsTable;

    fn held(realm: &str, body: &str) -> Held {
        let facts = Facts {
            id: "i".to_string(),
            realm: realm.to_string(),
            league: Some("Standard".to_string()),
            location_kind: "stash".to_string(),
            location_id: "t".to_string(),
            container: None,
            socketed_in: None,
            first_seen: 0,
            last_seen: 0,
            removed_at: None,
        };
        Held {
            item: derive(facts, body),
            price: crate::price::Priced::None {
                kind: "none".to_string(),
                why: String::new(),
            },
            place: Place {
                realm: realm.to_string(),
                league: Some("Standard".to_string()),
                kind: "stash".to_string(),
                id: "t".to_string(),
                name: None,
                tab_type: None,
                parent: None,
                container: None,
                socketed_in: None,
            },
            class: Classed::BaseUnread,
        }
    }

    fn table(totals: &str) -> &'static TotalsTable {
        let text = format!(
            "version = 5\nrealms = [\"pc\"]\nsource = \"s\"\ngenerated_by = \"t\"\n{totals}"
        );
        Box::leak(Box::new(TotalsTable::parse(&text).unwrap()))
    }

    fn named(table: &'static TotalsTable, name: &str) -> Named {
        Named::Total {
            table,
            total: table.get(name).unwrap(),
        }
    }

    fn f64_of(valued: Valued) -> Option<f64> {
        match valued {
            Valued::Value(n) | Valued::Incomplete(Some(n)) => Some(n.as_f64()),
            _ => None,
        }
    }

    /// The site's own answer, `+94.5 total maximum Life` over `+90` life
    /// and `+9` Strength (the plan, step 7; the fetch capture q4): a half
    /// weight, and a total never rounded.
    #[test]
    fn a_half_weight_is_exact_and_a_total_is_never_rounded() {
        let t = table(
            "[[total]]\nname = \"total_life\"\nrows = [\n  { template = \"# to maximum Life\", slot = \"arg1\", weight = 1 },\n  { template = \"# to Strength\", slot = \"arg1\", weight = 0.5 },\n]\n",
        );
        let plate = held(
            "pc",
            r#"{"explicitMods": ["+9 to Strength", "+90 to maximum Life"]}"#,
        );
        assert_eq!(
            value(named(t, "total_life"), None, &plate),
            Valued::Value(crate::exact::Exact::of(94.5))
        );
        // a total of nothing is lacked, no line a row names or lines that
        // cancel; a realm the table does not cover has no total, never zero
        let bare = held("pc", r#"{"explicitMods": ["+20 to maximum Mana"]}"#);
        assert_eq!(value(named(t, "total_life"), None, &bare), Valued::Lacked);
        let cancels = held(
            "pc",
            r#"{"implicitMods": ["+30 to maximum Life"], "explicitMods": ["-30 to maximum Life"]}"#,
        );
        assert_eq!(
            value(named(t, "total_life"), None, &cancels),
            Valued::Lacked
        );
        let poe2 = held("poe2", r#"{"explicitMods": ["+90 to maximum Life"]}"#);
        assert_eq!(
            value(named(t, "total_life"), None, &poe2),
            Valued::Incomplete(None)
        );
        assert_eq!(
            super::unread_of(named(t, "total_life"), &poe2)[0].problem,
            "no totals table for realm poe2 (totals v5 covers pc)"
        );
        // a contributor unread: a subtotal, marked incomplete
        let open = held(
            "pc",
            r#"{"implicitMods": "no", "explicitMods": ["+90 to maximum Life"]}"#,
        );
        assert_eq!(
            value(named(t, "total_life"), None, &open),
            Valued::Incomplete(Some(crate::exact::Exact::of(90.0)))
        );
        assert_eq!(
            super::unread_of(named(t, "total_life"), &open)[0].problem,
            "`implicitMods` is a string, not an array"
        );
    }

    /// A reading's reasons are those of the totals that leave it open,
    /// each part once, and none where it is decided with a total open
    /// beside it — which no answer asks for, a reason being given of an
    /// undecided term alone.
    #[test]
    fn a_reading_gives_reasons_only_where_it_is_open() {
        let t = table(
            "[[total]]\nname = \"fire\"\nrows = [\n  { template = \"#% to Fire Resistance\", slot = \"arg1\", weight = 1 },\n]\n[[total]]\nname = \"cold\"\nrows = [\n  { template = \"#% to Cold Resistance\", slot = \"arg1\", weight = 1 },\n]\n[[reading]]\nname = \"both\"\nkind = \"least\"\nof = [\"fire\", \"cold\"]\n[[reading]]\nname = \"either\"\nkind = \"count\"\nof = [\"fire\", \"cold\"]\n",
        );
        let reading = |name: &str| Named::Reading {
            table: t,
            reading: t.reading(name).unwrap(),
        };
        // fire open, cold nothing: no least, and one resistance or none
        let open = held(
            "pc",
            r#"{"explicitMods": ["+10000000000% to Fire Resistance"]}"#,
        );
        assert_eq!(value(reading("both"), None, &open), Valued::Lacked);
        assert!(super::unread_of(reading("both"), &open).is_empty());
        assert_eq!(
            value(reading("either"), None, &open),
            Valued::Incomplete(None)
        );
        let why = super::unread_of(reading("either"), &open);
        assert_eq!(why.len(), 1);
        assert!(why[0].problem.contains("a number with more digits"));
        // both open by one unread array: one reason, not one a total
        let unread = held("pc", r#"{"explicitMods": "no"}"#);
        assert_eq!(
            value(reading("both"), None, &unread),
            Valued::Incomplete(None)
        );
        assert_eq!(super::unread_of(reading("both"), &unread).len(), 1);
        let poe2 = held("poe2", r#"{"explicitMods": ["+9% to Fire Resistance"]}"#);
        for name in ["both", "either"] {
            assert_eq!(value(reading(name), None, &poe2), Valued::Incomplete(None));
            assert_eq!(
                super::unread_of(reading(name), &poe2)[0].problem,
                "no totals table for realm poe2 (totals v5 covers pc)"
            );
        }
    }

    /// A ranged total sums low with low and high with high, and takes
    /// low, high and avg; a row without a slot counts occurrences.
    #[test]
    fn a_ranged_total_and_a_counting_row() {
        let t = table(
            "[[total]]\nname = \"cold\"\nranged = true\nrows = [\n  { template = \"Adds # to # Cold Damage\", weight = 1 },\n  { template = \"Adds # to # Cold Damage to Attacks\", weight = 2 },\n]\n[[total]]\nname = \"sockets\"\nrows = [\n  { template = \"Has # Abyssal Sockets\", weight = 1 },\n  { template = \"# to maximum Life\", weight = 3 },\n]\n",
        );
        let item = held(
            "pc",
            r#"{"implicitMods": ["Adds 1 to 4 Cold Damage"], "explicitMods": ["Adds 10 to 20 Cold Damage to Attacks", "Has 2 Abyssal Sockets", "+90 to maximum Life", "+5 to maximum Life"]}"#,
        );
        let cold = named(t, "cold");
        assert_eq!(f64_of(value(cold, Some("low"), &item)), Some(21.0));
        assert_eq!(f64_of(value(cold, Some("high"), &item)), Some(44.0));
        assert_eq!(f64_of(value(cold, Some("avg"), &item)), Some(32.5));
        assert_eq!(f64_of(value(named(t, "sockets"), None, &item)), Some(7.0));
    }

    /// C101: dps and pdps as the properties display them; an item lacking
    /// the property lacks the field, one whose property is no number is
    /// undecided with that reason.
    #[test]
    fn dps_and_pdps_read_the_displayed_properties() {
        let sword = held(
            "pc",
            r#"{"properties": [
                {"name": "One Handed Sword", "values": [], "displayMode": 0},
                {"name": "Physical Damage", "values": [["59-88", 1]], "displayMode": 0, "type": 9},
                {"name": "Elemental Damage", "values": [["38-71", 4], ["57-108", 5]], "displayMode": 0, "type": 10},
                {"name": "Chaos Damage", "values": [["10-20", 7]], "displayMode": 0, "type": 11},
                {"name": "Attacks per Second", "values": [["1.25", 0]], "displayMode": 0, "type": 13}]}"#,
        );
        // 73.5 × 1.25; (73.5 + 54.5 + 82.5 + 15) × 1.25
        assert_eq!(
            f64_of(value(Named::Derived(Derived::Pdps), None, &sword)),
            Some(91.875)
        );
        assert_eq!(
            f64_of(value(Named::Derived(Derived::Dps), None, &sword)),
            Some(281.875)
        );
        let shown: Vec<String> = super::evidence(Named::Derived(Derived::Dps), &sword)
            .into_iter()
            .map(|e| match e {
                crate::eval::Evidence::Shown { text, .. } => text,
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(
            shown,
            [
                "Physical Damage: 59-88",
                "Elemental Damage: 38-71, 57-108",
                "Chaos Damage: 10-20",
                "Attacks per Second: 1.25"
            ]
        );
        let ring = held("pc", r#"{"explicitMods": ["+90 to maximum Life"]}"#);
        assert_eq!(
            value(Named::Derived(Derived::Dps), None, &ring),
            Valued::Lacked
        );
        let wand = held(
            "pc",
            r#"{"properties": [
                {"name": "Attacks per Second", "values": [["1.4", 0]], "displayMode": 0},
                {"name": "Elemental Damage", "values": [["38-71", 4]], "displayMode": 0}]}"#,
        );
        assert_eq!(
            value(Named::Derived(Derived::Pdps), None, &wand),
            Valued::Lacked
        );
        assert_eq!(
            f64_of(value(Named::Derived(Derived::Dps), None, &wand)),
            Some(76.3)
        );
        // a number the game never displays, and a product the units cannot
        // hold, are unread to the field — never a panic, never a rounding
        // (outside audit, 2026-09-24: `1e34` overflowed, 0.000025 read 0.00003)
        for (aps, phys, problem) in [
            (
                "1e34",
                "1",
                "`Attacks per Second` is `1e34`: no number the search reads",
            ),
            (
                "0.25",
                "0.0001",
                "`Attacks per Second` is `0.25`: times the damage's mean 0.0001, a product of more decimals than the search reads",
            ),
        ] {
            let odd = held(
                "pc",
                &format!(
                    r#"{{"properties": [
                        {{"name": "Attacks per Second", "values": [["{aps}", 0]], "displayMode": 0}},
                        {{"name": "Physical Damage", "values": [["{phys}", 1]], "displayMode": 0}}]}}"#
                ),
            );
            assert_eq!(
                value(Named::Derived(Derived::Pdps), None, &odd),
                Valued::Incomplete(None)
            );
            assert_eq!(
                super::unread_of(Named::Derived(Derived::Pdps), &odd)[0].problem,
                problem
            );
        }
        // an unread element leaves a field open only when it may be one
        // the field reads (outside audit, 2026-09-24)
        let quality = held(
            "pc",
            r#"{"properties": [
                {"name": "Quality", "values": 7, "displayMode": 0},
                {"name": "Elemental Damage", "values": "no", "displayMode": 0},
                {"name": "Attacks per Second", "values": [["2", 0]], "displayMode": 0},
                {"name": "Physical Damage", "values": [["10-20", 1]], "displayMode": 0}]}"#,
        );
        assert_eq!(
            f64_of(value(Named::Derived(Derived::Pdps), None, &quality)),
            Some(30.0)
        );
        assert_eq!(
            value(Named::Derived(Derived::Dps), None, &quality),
            Valued::Incomplete(None)
        );
        let nameless = held(
            "pc",
            r#"{"properties": [7,
                {"name": "Attacks per Second", "values": [["2", 0]], "displayMode": 0},
                {"name": "Physical Damage", "values": [["10-20", 1]], "displayMode": 0}]}"#,
        );
        assert_eq!(
            value(Named::Derived(Derived::Pdps), None, &nameless),
            Valued::Incomplete(None)
        );
        let odd = held(
            "pc",
            r#"{"properties": [
                {"name": "Attacks per Second", "values": [["fast", 0]], "displayMode": 0},
                {"name": "Physical Damage", "values": [["59-88", 1]], "displayMode": 0}]}"#,
        );
        assert_eq!(
            value(Named::Derived(Derived::Pdps), None, &odd),
            Valued::Incomplete(None)
        );
        assert_eq!(
            super::unread_of(Named::Derived(Derived::Pdps), &odd)[0].problem,
            "`Attacks per Second` is `fast`: no number the search reads"
        );
        let unread = held("pc", r#"{"properties": 7}"#);
        assert_eq!(
            value(Named::Derived(Derived::Dps), None, &unread),
            Valued::Incomplete(None)
        );
        assert_eq!(
            super::unread_of(Named::Derived(Derived::Dps), &unread)[0].problem,
            "`properties` is a number, not an array"
        );
    }
}
