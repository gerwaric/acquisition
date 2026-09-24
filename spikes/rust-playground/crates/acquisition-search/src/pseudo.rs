//! Computed values (the reference, *Values*; C94, C101; the build plan,
//! step 7): `pseudo.<name>`, one namespace for the named totals of the
//! table (`totals.rs`) and the derived fields — what a comparison, a sort,
//! a sum or `undecided( … )` consumes when it names one, and what a row
//! shows of it.
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
//!   as no number and no range, the field is undecided with that reason.
//!   The base defence percentile the same paragraph names is not built
//!   (`bind::NOT_BUILT`, its formula unpinned: `search/pseudo-stats/README.md`,
//!   open question 3).
//!
//! # As built
//!
//! - **A name is the table's or a derived field's**, matched in any case
//!   (B1); an unknown one is an authoring error with the near names
//!   offered, as a field's is — unless it carries a slot word, which asks
//!   for a ranged total, and this build ships none (`bind::NOT_BUILT`).
//!   A slot word on a total that is no range, or on a derived field, is an
//!   authoring error; a ranged total asked for with none is one that
//!   offers `low`, `high` and `avg`.
//! - **A value has the sum's three statuses** ([`Valued`]; the reference,
//!   *A sum's status*): complete, an incomplete subtotal, or — a derived
//!   field alone — lacked. Unavailable, a total with no definition for the
//!   item's realm, is incomplete with nothing readable, as a field that
//!   could not be read is; its reason is the table's, made once. A value
//!   is open exactly when it would sort as incomplete, so `undecided( … )`,
//!   the sort scalar and a count's sum ask one function (`eval::scalar`).
//! - **A total's arithmetic is the evaluator's sum** (`eval::sum`, and
//!   `eval::count` for a row without a slot), weighted in halves
//!   (`exact::Exact::halved`), so a total and a `sum( … )` over the same
//!   lines cannot disagree (rule 10 of the plan); a ranged total's `avg` is
//!   the mean of its two totals, exact.
//! - **What a row shows** of a computed value: its value, then the lines
//!   the total's rows named on the item, or the properties the derived
//!   field read, bounded as every term's evidence is (`eval::evidence`).
//!   Why it is open is the open contributors' own reasons (rule 8): a
//!   source a row admits unread, an occurrence whose number or flag is,
//!   the property that could not be read.

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::corpus::Held;
use crate::derive::{Part, Property, Unread};
use crate::describe::Named as Entry;
use crate::error::{ErrorKind, LanguageError};
use crate::eval::{self, Evidence};
use crate::exact::{self, Exact};
use crate::totals::{self, Total, TotalsTable};

/// A computed value, bound by name.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Named {
    Total {
        table: &'static TotalsTable,
        total: &'static Total,
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
    Ok(table.get(name).map(|total| Named::Total { table, total }))
}

fn table() -> Result<&'static TotalsTable, LanguageError> {
    totals::table().map_err(|e| {
        LanguageError::new(
            ErrorKind::Tree,
            format!("the totals table this build ships does not load: {e}"),
        )
    })
}

/// Every name, the totals in the table's order then the derived fields:
/// what a near-name suggestion is scored against. Leaked once from the
/// shipped table, as the class names are (`class::names`).
static NAMES: LazyLock<&'static [&'static str]> = LazyLock::new(|| {
    let mut names: Vec<&'static str> = totals::table()
        .map(|t| {
            t.totals()
                .iter()
                .map(|t| &*Box::leak(t.name.clone().into_boxed_str()))
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
    /// A derived field whose input the item lacks: known absence.
    Lacked,
}

pub(crate) fn value(named: Named, slot: Option<&str>, held: &Held) -> Valued {
    match named {
        Named::Total { table, total } => total_of(table, total, slot, held),
        Named::Derived(derived) => derived_of(derived, held),
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
            unread.into_iter().map(Cow::Borrowed).collect()
        }
        Named::Derived(derived) => match inputs(derived, held) {
            Ok(_) | Err(Inputs::Lacked) => Vec::new(),
            Err(Inputs::Unread(unread)) => unread,
        },
    }
}

/// What a row shows of a computed value that matched: the lines the
/// total's rows named on the item, or the properties the derived field
/// read.
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
    let value = if total.ranged {
        let low = Exact::sum(total.rows.iter().map(|row| row_sum(Some("low"), row)));
        let high = Exact::sum(total.rows.iter().map(|row| row_sum(Some("high"), row)));
        match slot {
            Some("low") => low,
            Some("high") => high,
            _ => low.mid(high),
        }
    } else {
        Exact::sum(
            total
                .rows
                .iter()
                .map(|row| row_sum(row.slot.as_deref(), row)),
        )
    };
    if complete {
        Valued::Value(value)
    } else {
        Valued::Incomplete(Some(value))
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

/// The one reading of a derived field's inputs: what was unread of the
/// properties leaves every derived field open — an element that could
/// not be read may be the one the field needs — and a property displayed
/// as no number and no range is unread under `properties`, said here.
fn inputs(derived: Derived, held: &Held) -> Result<Read<'_>, Inputs<'_>> {
    let item = &held.item;
    let properties = Part::Properties("properties".to_string());
    let unread: Vec<Cow<'_, Unread>> = item
        .unread
        .iter()
        .filter(|u| u.part == Part::Body || u.part == properties)
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
    if !problems.is_empty() {
        return Err(Inputs::Unread(problems));
    }
    let (Some(speed), Some(means)) = (speed, means.into_iter().collect::<Option<Vec<Exact>>>())
    else {
        return Err(Inputs::Unread(Vec::new()));
    };
    let mut shown: Vec<&Property> = std::iter::once(aps).chain(damage).collect();
    shown.sort_by_key(|p| item.properties.iter().position(|q| std::ptr::eq(q, *p)));
    Ok(Read {
        value: Exact::sum(means).times(speed),
        shown,
    })
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
                    leak(format!("undecided({name})")),
                ]
            } else {
                vec![
                    leak(format!("{name}>=60")),
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
    }
    for derived in Derived::ALL {
        let name = format!("pseudo.{}", derived.name());
        out.push(Entry {
            examples: vec![
                leak(format!("{name}>=300")),
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
/// its definition, then the derived fields.
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
            place: Place {
                realm: realm.to_string(),
                league: Some("Standard".to_string()),
                kind: "stash".to_string(),
                id: "t".to_string(),
                name: None,
                parent: None,
                container: None,
                socketed_in: None,
            },
            class: Classed::BaseUnread,
        }
    }

    fn table(totals: &str) -> &'static TotalsTable {
        let text = format!(
            "version = 1\nrealms = [\"pc\"]\nsource = \"s\"\ngenerated_by = \"t\"\n{totals}"
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
        // nothing sums to an honest zero; a realm the table does not cover
        // has no total, never zero
        let bare = held("pc", r#"{"explicitMods": ["+20 to maximum Mana"]}"#);
        assert_eq!(
            f64_of(value(named(t, "total_life"), None, &bare)),
            Some(0.0)
        );
        let poe2 = held("poe2", r#"{"explicitMods": ["+90 to maximum Life"]}"#);
        assert_eq!(
            value(named(t, "total_life"), None, &poe2),
            Valued::Incomplete(None)
        );
        assert_eq!(
            super::unread_of(named(t, "total_life"), &poe2)[0].problem,
            "no totals table for realm poe2 (totals v1 covers pc)"
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
