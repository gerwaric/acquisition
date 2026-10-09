//! Computed values (the reference, *Values*; C94, C101; the build plan,
//! steps 7, 9c3 and 9d-A): `pseudo.<name>`, one namespace for the named totals
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
//!   pure function over the item's properties, its lines and reference
//!   data, listed with its definition
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
//!   2026-09-24, T2 in `search/LEDGER.md`: "A ring has no dps"), and
//!   `-has:pseudo.total_res` a total's, since a total of nothing is
//!   lacked (`totals.rs`, C94 as ruled 2026-09-26, which takes back
//!   T2's "every item has a total").
//! - **C101 — the base defence percentile** (`pseudo.defence_pct`; V5,
//!   the first seat: the trade site's Base Percentile; the build plan,
//!   step 9d-A; taken from the contract detail there). The item's defence
//!   against its base's range from reference data (C68, C106;
//!   `defences.rs`). It is the site's own stated rule — each base defence
//!   roll's percentile, averaged, rounded half up, a roll being any
//!   integer whose display is the item's — with one departure the
//!   definition prints: a display at an exact half admits both integers
//!   as rolls (J5, 2026-10-01: the stated rule reproduced 541 of 543
//!   captured items and this reading all 543, each one value). A
//!   defence's display is (roll + the local flat additions) × (1 + the
//!   sum of the local increases) × (1 + the quality displayed), flat
//!   before increase and quality multiplying, as both community tools
//!   state; the `reduced` spelling of an increase is its stat below
//!   nothing. A roll's percentile is 100 × (roll − minimum) ÷ (maximum −
//!   minimum). The value is a whole number and takes no slot word.
//!   - *The shapes round i pinned (J6).* Three defence types on one base
//!     are the average of three rolls; under the enchant that zeroes
//!     quality, `Quality does not increase Defences`, the quality term is
//!     dropped; ward is a roll as the others are; a defence the item
//!     displays and its base ranges none of is no base defence roll and
//!     is not read.
//!   - *A twin text, which a local stat and a global one both display,
//!     is told by the item's mark (J8, ruled 2026-10-05).* An implicit
//!     twin on an eldritch item, `searing` or `tangled`, is the global
//!     stat and left out, 17 of 17 captured; an explicit twin, and an
//!     implicit twin on a synthesised item, are the item's own, 442 of
//!     442 and 10 of 10 (`search/pseudo-stats/scripts/defence-twins.py`;
//!     the 421 the contract detail said was of an earlier day); an
//!     implicit twin on an item of neither mark, seen nowhere, cannot be
//!     established and the field is undecided there with that reason
//!     (C93). A text only a local stat displays is the item's own
//!     whatever its array, a crucible node's apart (J33, below). The
//!     definition prints the counts each part rests on.
//!   - *A value only where the item fixes one (J33, 2026-10-08).* The
//!     field is a value only where what the item shows gives one value
//!     from lines whose side is established; anywhere else it is
//!     undecided, the reason saying which part failed: a defence line in
//!     `crucibleMods`, the site not asked; a twin text in another array
//!     J8 does not name; an implicit twin on an item eldritch and
//!     synthesised; a base ranging a defence the item does not display; a
//!     base whose minimum is its maximum; a display several rolls give at
//!     different values. Several rolls that give one value are that
//!     value.
//!   - *A roll out of the base's range still reads (J34, 2026-10-08).*
//!     An item above its base's current maximum reads over 100 % and one
//!     under its minimum below 0 — the owner: "keep reads over 100". It
//!     is the stated rule's and no capture's: the site lists no armour
//!     over 100. So a line misread shows a number out of range, and no
//!     reason.
//!   - *A realm with no table (J35, 2026-10-08)* is undecided with that
//!     reason, never zero, on every item of it, one displaying no defence
//!     among them — the owner: "undecided". The realms are the class
//!     table's.
//!   - *Lacked* is an item, in a realm the table covers, that displays
//!     none of the four defences — known absence (C93), claimed only
//!     where everything that could display one was read.
//!   - A defence bound on the site is inexact (C102), and a
//!     quality-normalised defence is parked (`decisions/search.md`).
//! - **C94 — what no total reads (J29)** is `totals.rs`'s, said there
//!   once (the owner, 2026-10-08: "Leave both out."): no total reads a
//!   line of `crucibleMods` or `utilityMods`, and neither array unread
//!   leaves a total open. A reading follows the totals it reads and has
//!   no rule of its own; a damage field reads properties, never lines.
//!   J29 is not the percentile's rule: it reads every array of lines, is
//!   undecided on a defence line of `crucibleMods` (J33), and an array it
//!   reads that is no array leaves it open.
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
//!   offered, as a field's is, with a slot word or without one.
//!   A slot word on a total that is no range, on a reading or on a
//!   derived field, is an authoring error, and so is a slot word a range
//!   does not take — a total has no occurrence, so no `arg1` — which
//!   offers `avg`, `low` and `high`. `has:` asks presence and takes no
//!   slot word: one behind a ranged total there is refused as a slot word
//!   where none is taken (`ErrorKind::SlotUnknown`), offering `has:` of
//!   the total, never as an unknown name (`bind.rs`).
//! - **A ranged total with no slot word is its `avg`** (C94, the ranged
//!   family, `totals.rs`; owner, 2026-09-26: "yes, bare means average"),
//!   wherever a computed value is asked: a comparison, `has:`, `--sort`,
//!   a count's sum. `low` and `high` stay. The bare name is bound and
//!   printed as written and read as `avg` ([`total_of`]). A ranged total
//!   is nothing, and so lacked, only where its low and its high both are
//!   (J31), so `has:` of it, which asks its `avg`, holds on a range whose
//!   average alone is 0.
//! - **A row that reads what the item is** (`totals.rs`, J7, J30) is
//!   asked here, where a total is summed ([`total_of`]) and nowhere
//!   else: `totals::Row::counts_on` is the category's answer, and where
//!   that is not established a line the row names is unread to the total
//!   — an incomplete subtotal of the other rows, undecided under a
//!   comparison and `has:`, open under `undecided( … )` — by the
//!   category's own reasons for the id asked (`category::unread`, the one
//!   maker a category term reads; [`unread_of`]). What a row shows of a
//!   total is the lines of the rows that count on the item.
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
//!   read — and, of the percentile, the lines it took as the item's own
//!   — bounded as every term's evidence is (`eval::evidence`). A count
//!   left open prints its interval, `2..3`, as a socket count does. A
//!   row sorted by a reading shows it first by its own name
//!   ([`sorted_by`]).
//! - **The vocabulary lists a computed value beside the templates** whose
//!   name or definition its narrowing matches — a total's own
//!   definition, its rows and its limit, never the rule the table states
//!   of every total (J29), by a word of which every total would be
//!   listed, and the percentile's what it reads and answers, never the
//!   captured counts and the table's provenance its definition prints
//!   beside that — and none under `line`
//!   alone, where `--describe` names them (owner, 2026-09-24, T5 in
//!   `search/LEDGER.md`; the reference, `--count line[:text]`;
//!   `counts.rs`), marked computed, with the count of the matches that
//!   have it, as `has:` of it asks ([`present`]): one whose value is
//!   established, and a count one of whose totals is — never one lacked,
//!   nor one left open with nothing established — routed by
//!   `has:pseudo.<name>`, exactly those.
//!   Why it is open is the open contributors' own reasons (rule 8): a
//!   source a row admits unread, an occurrence whose number or flag is,
//!   the property that could not be read; a reading's are those of the
//!   totals that leave it open.
//! - **The percentile is one reading, in stages** ([`percentile`]): the
//!   realm's table, the properties, the base, the arrays, the side of
//!   each line, the rolls — each answered before the next is read, so a
//!   reason is of the first thing that could not be established, and its
//!   value, its reasons and what a row shows of it cannot disagree (rule
//!   10 of the plan). What the deriver said unread is the deriver's
//!   reason, borrowed: `properties`, an array of lines, a line's number,
//!   the base, a mark under its own key. What only this field can say is
//!   one kind each (`defences::PercentileGap`): the realm has no table;
//!   the item has no base; the base is not in the table, is held more
//!   than once with nothing telling which, or no row of it has every
//!   defence it ranges displayed; an implicit twin on an item of neither
//!   mark, or of both; a twin text in an array no ruling names; a
//!   defence line of a crucible node; no roll gives the display; the
//!   base's minimum is its maximum; several rolls give different values.
//!   A reason of a line whose side is open says what each reading would
//!   give, where both can be worked.
//! - **Unknown of no more than was lost** (rule 8 of the plan). A line of
//!   a type no roll of the base reads opens nothing, whatever stat it is,
//!   whatever its array and whether or not its number was read. A mark
//!   is asked only of an implicit twin text a roll reads, so a mark that
//!   is no yes or no opens the field there and nowhere else. A mark is
//!   three-valued — a yes where the deriver holds the flag, unknown where
//!   its own key is no yes or no, a no otherwise — and an item eldritch
//!   by one of its two marks is eldritch whatever the other says, so only
//!   a mark still needed and unknown opens it. What `influences` holds is
//!   not asked: on the owner's copy (2026-10-09) the three marks are
//!   their own keys on 92, 96 and 261 items and under `influences` on
//!   none. The enchant drops the quality wherever it sits, being no
//!   defence line; lines that take a defence to nothing or below leave
//!   the display fixing no roll, which is several values. A flag is
//!   never asked (J32): a line counts by its text. The quality is asked
//!   only of an item displaying a defence. A property the deriver read
//!   with no name is a property of no name and none of the field's; one
//!   whose name it could not read may be any.
//! - **A property is a whole number, once.** A defence is displayed as
//!   digits alone and the quality as `+20%`, its sign and its `%` each
//!   read where written; either displayed otherwise,
//!   with other than one value, or more than once, is unread under
//!   `properties`, said here, as a damage field's is. A base name the
//!   table holds on several rows is the one row all of whose defences the
//!   item displays.
//! - **The arithmetic is of whole numbers** (`exact::Ratio`; the record's
//!   shape of fault, arithmetic on floats): a displayed number, an
//!   increase and a quality are fractions, never floats — the run that
//!   wrote the expected answers found three items of the owner's copy a
//!   float read wrongly, a product at an exact half landing under it. The
//!   rolls of a display are an interval, both ends in (J5 is the upper
//!   end), and a percentile rises with its roll, so the least rolls and
//!   the most give the least value and the most: the display fixes the
//!   value exactly when the two are one, and no combination is walked.
//!   Numbers past what the whole numbers hold — no displayed item's —
//!   are unread to the field, never rounded.

use std::borrow::Cow;
use std::sync::LazyLock;

use crate::bind::NumTest;
use crate::category;
use crate::corpus::Held;
use crate::defences::{self, Base, Defence, Does, PercentileGap};
use crate::derive::{Item, Line, Part, Property, Unread};
use crate::describe::Named as Entry;
use crate::error::{ErrorKind, LanguageError};
use crate::eval::{self, Evidence, Outcome, Truth};
use crate::exact::{self, Exact, Ratio};
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
    /// The base defence percentile ([`percentile`]).
    DefencePct,
}

impl Derived {
    const ALL: [Derived; 3] = [Derived::Dps, Derived::Pdps, Derived::DefencePct];

    pub fn name(self) -> &'static str {
        match self {
            Derived::Dps => "dps",
            Derived::Pdps => "pdps",
            Derived::DefencePct => "defence_pct",
        }
    }

    pub fn definition(self) -> String {
        match self {
            Derived::Dps => {
                "damage per second: the attacks per second times the mean of every damage range the item displays — physical, each elemental, chaos — as `Physical Damage`, `Elemental Damage`, `Chaos Damage` and `Attacks per Second` show them, quality already applied by the game; an item with no attacks per second, or no damage range, lacks it".to_string()
            }
            Derived::Pdps => {
                "physical damage per second: the attacks per second times the mean of `Physical Damage`; an item lacking either property lacks it".to_string()
            }
            Derived::DefencePct => format!(
                "{} Of the captured lines whose side the site's id names, each part holds on all: 442 of 442 explicit, 17 of 17 implicit on an eldritch item, 10 of 10 implicit on a synthesised item — counted by search/pseudo-stats/scripts/defence-twins.py. Ruled: the exact half J5, the twin J8, undecided J33, out of range J34, a realm with no table J35; the site's bound C102. {}",
                self.own_definition(),
                match defences::table() {
                    Ok(table) => table.provenance(),
                    Err(e) => format!("the defence table this build ships does not load: {e}"),
                }
            ),
        }
    }

    /// What a vocabulary narrowing is matched against ([`OWN`]): what the
    /// field reads and answers, without the bookkeeping its definition
    /// prints beside that — the captured lines a part rests on, the
    /// table's version, sources and counts — which names no line and no
    /// value, and by a word of which the field would be listed under a
    /// text it does not read (T5: a narrowing lists those it matches).
    /// One text, which [`Derived::definition`] prints whole and adds to.
    pub fn own_definition(self) -> String {
        match self {
            Derived::Dps | Derived::Pdps => self.definition(),
            Derived::DefencePct => format!(
                "the trade site's Base Percentile, by the site's own rule — \"{SITE_TIP}\": each of `Armour`, `Evasion Rating`, `Energy Shield` and `Ward` that the item's base ranges is worked back from the number displayed to the roll behind it, a roll being any whole number whose display is the item's — (the roll + what the item's own lines add) × (1 + the item's own increases) × (1 + the quality displayed), rounded half up — and the value is each roll's place in the base's range as a percentage, averaged over the defences and rounded half up to a whole number. One departure from the site's words: a display at an exact half admits both whole numbers as rolls. Under `Quality does not increase Defences` the quality is dropped. A line whose text a global stat displays too, a twin text, is told by where it sits: an explicit is the item's own; an implicit on an eldritch item, `searing` or `tangled`, is the global stat and left out; an implicit on a synthesised item is the item's own. A roll out of the base's range still reads, over 100 above its maximum and below 0 under its minimum, which no capture has shown: a line misread can show a number out of range. An item displaying none of the four defences lacks it. It is undecided, with the reason, wherever what the item shows does not give one value from lines whose side is established: in a realm with no table, on every item; a base the table lacks, holds more than once with nothing telling which, or no row of which fits what is displayed; a property, an array or a line unread; a twin implicit on an item of neither mark, of both, or whose mark is unread; a twin text in any other array; a line of a crucible node that the field reads, of which the site has not yet been asked; no roll giving the display; a base whose minimum is its maximum; several rolls giving different values. A defence bound on the trade site is inexact."
            ),
        }
    }

    /// The bound a printed example compares the field with.
    fn example(self) -> u32 {
        match self {
            Derived::Dps | Derived::Pdps => 300,
            Derived::DefencePct => 80,
        }
    }
}

/// The site's own words for its Base Percentile: the tip of its filter
/// (`search/trade-query/data/grammar.json`, the armour filters).
const SITE_TIP: &str = "The percentile of all base defence rolls, averaged (0-100%)";

impl Named {
    pub fn ranged(self) -> bool {
        matches!(self, Named::Total { total, .. } if total.ranged)
    }

    /// The name after `pseudo.`, as the table or the field spells it.
    pub fn name(self) -> &'static str {
        match self {
            Named::Total { total, .. } => &total.name,
            Named::Reading { reading, .. } => &reading.name,
            Named::Derived(derived) => derived.name(),
        }
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
        Named::Derived(Derived::DefencePct) => match percentile(held) {
            Ok(_) | Err(Inputs::Lacked) => Vec::new(),
            Err(Inputs::Unread(unread)) => unread,
        },
        Named::Derived(derived) => match inputs(derived, held) {
            Ok(_) | Err(Inputs::Lacked) => Vec::new(),
            Err(Inputs::Unread(unread)) => unread,
        },
    }
}

/// What leaves a total open on this item, each part once: what each row
/// that counts on the item could not read, and — under a row that reads
/// the item's category, where that is not established and the row names
/// a line of the item — the category's own reasons, made where a
/// category term's are (`category::unread`; rule 10 of the plan), and
/// those alone: such a row is not summed, so a number or an array it
/// would have read is no second reason for the one contribution lost
/// (rule 8). A row that does not count on the item reads nothing of it.
fn unread_of_total<'a>(total: &Total, held: &'a Held, unread: &mut Vec<&'a Unread>) {
    let mut push = |u: &'a Unread| {
        if !unread.iter().any(|seen| std::ptr::eq(*seen, u)) {
            unread.push(u);
        }
    };
    for row in &total.rows {
        match row.counts_on(&held.category) {
            Truth::False => continue,
            Truth::Undecided if names_a_line(row, held) => {
                category::unread(&held.item, &held.class, &held.category, row.asks())
                    .into_iter()
                    .for_each(&mut push);
                // said once (rule 8): the row was not summed
                // ([`total_of`]), so nothing else of it was asked
                continue;
            }
            Truth::Undecided | Truth::True => {}
        }
        let slots: &[Option<&str>] = if total.ranged {
            &[Some("low"), Some("high")]
        } else {
            &[row.slot.as_deref()]
        };
        for slot in slots {
            eval::unread_of_sum(held, &row.group, *slot)
                .into_iter()
                .for_each(&mut push);
        }
    }
}

/// Whether the row's selector picks a line of the item: what a row that
/// reads the item's category leaves unread where the category cannot say
/// whether the row counts it.
fn names_a_line(row: &totals::Row, held: &Held) -> bool {
    eval::selected(held, &row.group).next().is_some()
}

/// What a row shows of a computed value that matched: the lines the
/// total's rows named on the item, the totals a reading read that the
/// item shows, or the properties the derived field read.
pub(crate) fn evidence(named: Named, held: &Held) -> Vec<Evidence> {
    match named {
        Named::Total { total, .. } => {
            let mut lines: Vec<&crate::derive::Line> = Vec::new();
            // the lines of the rows that count on the item: a weapon's
            // own line is no line of a row that means the other twin
            for row in total
                .rows
                .iter()
                .filter(|row| row.counts_on(&held.category) == Truth::True)
            {
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
        // the properties read, then the lines taken as the item's own
        Named::Derived(Derived::DefencePct) => match percentile(held) {
            Ok(read) => read
                .shown
                .into_iter()
                .map(shown_evidence)
                .chain(read.lines.into_iter().map(eval::line_evidence))
                .collect(),
            Err(_) => Vec::new(),
        },
        Named::Derived(derived) => match inputs(derived, held) {
            Ok(read) => read.shown.into_iter().map(shown_evidence).collect(),
            Err(_) => Vec::new(),
        },
    }
}

fn shown_evidence(property: &Property) -> Evidence {
    Evidence::Shown {
        part: property.array.clone(),
        text: property.text.clone(),
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
    let nothing = Exact::default();
    let mut row_sum = |slot: Option<&str>, row: &totals::Row| -> Exact {
        // a row that reads what the item is (`totals.rs`): where its id
        // holds the row counts nothing and reads nothing of the item;
        // where the category cannot say, a line the row names is a
        // contribution unread — and with no such line there is none
        match row.counts_on(&held.category) {
            Truth::False => return nothing,
            Truth::Undecided if names_a_line(row, held) => {
                complete = false;
                return nothing;
            }
            Truth::Undecided | Truth::True => {}
        }
        let (value, whole) = match slot {
            Some(slot) => eval::sum(held, &row.group, slot),
            None => eval::count(held, &row.group),
        };
        complete &= whole;
        value.halved(row.halves)
    };
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
    let read = match derived {
        Derived::DefencePct => percentile(held).map(|read| read.value),
        Derived::Dps | Derived::Pdps => inputs(derived, held).map(|read| read.value),
    };
    match read {
        Ok(value) => Valued::Value(value),
        Err(Inputs::Lacked) => Valued::Lacked,
        Err(Inputs::Unread(_)) => Valued::Incomplete(None),
    }
}

/// The properties a damage field reads; the percentile's are its own
/// ([`percentile`]).
fn reads(derived: Derived) -> &'static [&'static str] {
    match derived {
        Derived::Pdps => &[ATTACKS_PER_SECOND, PHYSICAL],
        Derived::Dps => &[ATTACKS_PER_SECOND, PHYSICAL, ELEMENTAL, CHAOS],
        Derived::DefencePct => &[],
    }
}

/// The one reading of a damage field's inputs: what was unread of the
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
        Derived::DefencePct => Vec::new(),
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

// ---- the base defence percentile -------------------------------------------------------------------

/// The property the quality is displayed under.
const QUALITY: &str = "Quality";

/// The item's marks J8 reads: an eldritch item's, and a synthesised one's.
const ELDRITCH: [&str; 2] = ["searing", "tangled"];
const SYNTHESISED: &str = "synthesised";

/// What the percentile read on an item.
struct Percentile<'a> {
    value: Exact,
    /// The properties read, in the item's order.
    shown: Vec<&'a Property>,
    /// The lines taken as the item's own, the enchant among them, in the
    /// item's order.
    lines: Vec<&'a Line>,
}

/// What J8 reads of the item: which kind of item it is.
enum Mark<'a> {
    Eldritch,
    Synthesised,
    Both,
    Neither,
    /// A mark is no yes or no: the deriver's own unread, under its key.
    Unread(Vec<&'a Unread>),
}

fn mark_of(item: &Item) -> Mark<'_> {
    // a mark is three-valued: a yes where the deriver holds the flag,
    // unknown where its own key is no yes or no, a no otherwise; and an
    // item is eldritch by either of its two marks. Only a mark still
    // needed and unknown leaves the item's kind open (rule 8).
    let has = |mark: &str| item.flags.iter().any(|flag| flag == mark);
    let (eldritch, synthesised) = (ELDRITCH.into_iter().any(has), has(SYNTHESISED));
    let unread: Vec<&Unread> = item
        .unread
        .iter()
        .filter(|u| {
            matches!(&u.part, Part::Field(key)
                if (!eldritch && ELDRITCH.contains(&key.as_str()))
                    || (!synthesised && key == SYNTHESISED))
        })
        .collect();
    if !unread.is_empty() {
        return Mark::Unread(unread);
    }
    match (eldritch, synthesised) {
        (true, true) => Mark::Both,
        (true, false) => Mark::Eldritch,
        (false, true) => Mark::Synthesised,
        (false, false) => Mark::Neither,
    }
}

/// A line the item's own stat made: what it does, to which of the
/// defences the base ranges, and its number.
#[derive(Clone)]
struct Local {
    does: Does,
    to: Vec<Defence>,
    number: Ratio,
}

/// What the rolls behind a display give.
enum Rolled {
    Value(i128),
    /// What the display does not fix, each with its words.
    Open(Vec<(PercentileGap, String)>),
    /// Whole numbers past what the search computes with.
    TooLarge,
}

/// The field left open, by these reasons.
fn open<'a, T>(reasons: Vec<Cow<'a, Unread>>) -> Result<T, Inputs<'a>> {
    Err(Inputs::Unread(reasons))
}

fn gap(gap: PercentileGap, problem: String) -> Cow<'static, Unread> {
    Cow::Owned(Unread {
        part: Part::Percentile(gap),
        problem,
        line: None,
        name: None,
        socket: None,
    })
}

/// A whole number as a property displays it: digits alone, within the
/// crate's rule for a number (`exact::reads`).
fn whole(text: &str) -> Option<i128> {
    (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) && exact::reads(text))
        .then(|| text.parse().ok())
        .flatten()
}

/// The base defence percentile of an item (the module doc, C101): the
/// one reading of it, which its value, its reasons and what a row shows
/// of it all ask. Each stage answers before the next is read, so a
/// reason is of the first thing that could not be established: the
/// realm's table, the properties, the base, the arrays, the side of each
/// line, the rolls.
fn percentile(held: &Held) -> Result<Percentile<'_>, Inputs<'_>> {
    let item = &held.item;
    let table = match defences::table() {
        Ok(table) => table,
        // no corpus is loaded over a table that does not load (`corpus.rs`)
        Err(e) => return open(vec![gap(PercentileGap::NoTableForRealm, e.to_string())]),
    };
    // J35: a realm with no table, on every item of it
    if let Some(reason) = table.unavailable(&item.facts.realm) {
        return open(vec![Cow::Borrowed(reason)]);
    }

    // the properties: the four defences, and the quality
    let properties = Part::Properties("properties".to_string());
    let named = |name: &str| {
        item.properties
            .iter()
            .filter(|p| p.array == "properties" && p.name.eq_ignore_ascii_case(name))
            .collect::<Vec<&Property>>()
    };
    let unread_named = |reads: &dyn Fn(&str) -> bool, nameless: bool| -> Vec<Cow<'_, Unread>> {
        item.unread
            .iter()
            .filter(|u| {
                (nameless && u.part == Part::Body)
                    || (u.part == properties
                        && match u.name.as_deref() {
                            Some(name) => reads(name),
                            None => nameless,
                        })
            })
            .map(Cow::Borrowed)
            .collect()
    };
    let problem = |p: &Property, what: &str| -> Cow<'static, Unread> {
        Cow::Owned(Unread {
            part: properties.clone(),
            problem: format!("`{}` is `{}`: {what}", p.name, p.values.join(", ")),
            line: None,
            name: Some(p.name.clone()),
            socket: None,
        })
    };
    let is_defence = |name: &str| {
        Defence::ALL
            .iter()
            .any(|d| d.property().eq_ignore_ascii_case(name))
    };
    let mut reasons = unread_named(&is_defence, true);
    let mut displayed: Vec<(Defence, i128, &Property)> = Vec::new();
    for defence in Defence::ALL {
        match named(defence.property()).as_slice() {
            [] => {}
            [property] => match property.values.as_slice() {
                [value] => match whole(value) {
                    Some(number) => displayed.push((defence, number, property)),
                    None => reasons.push(problem(property, "no whole number the search reads")),
                },
                _ => reasons.push(problem(property, "not one value")),
            },
            [property, ..] => reasons.push(problem(property, "displayed more than once")),
        }
    }
    if !reasons.is_empty() {
        return open(reasons);
    }
    // known absence (C93): none of the four is displayed, and everything
    // that could hold one was read
    if displayed.is_empty() {
        return Err(Inputs::Lacked);
    }
    // the quality is asked only of an item displaying a defence (rule 8)
    let mut reasons = unread_named(&|name| name.eq_ignore_ascii_case(QUALITY), false);
    let mut quality: Option<(i128, &Property)> = None;
    match named(QUALITY).as_slice() {
        [] => {}
        [property] => match property.values.as_slice() {
            [value] => {
                let digits = value.strip_prefix('+').unwrap_or(value);
                match whole(digits.strip_suffix('%').unwrap_or(digits)) {
                    Some(number) => quality = Some((number, property)),
                    None => reasons.push(problem(property, "no whole number the search reads")),
                }
            }
            _ => reasons.push(problem(property, "not one value")),
        },
        [property, ..] => reasons.push(problem(property, "displayed more than once")),
    }
    if !reasons.is_empty() {
        return open(reasons);
    }

    // the base: the one row of the table all of whose defences the item
    // displays
    let Some(name) = item.base.as_deref() else {
        let unread: Vec<Cow<'_, Unread>> = item
            .unread
            .iter()
            .filter(|u| matches!(&u.part, Part::Field(key) if key == "baseType"))
            .map(Cow::Borrowed)
            .collect();
        return open(if unread.is_empty() {
            vec![gap(
                PercentileGap::NoBase,
                format!(
                    "the item has no base to look up in the defence table (defences v{})",
                    table.version()
                ),
            )]
        } else {
            unread
        });
    };
    let rows = table.bases(name);
    if rows.is_empty() {
        return open(vec![gap(
            PercentileGap::BaseNotInTable,
            format!(
                "`{name}` is not in the defence table (defences v{})",
                table.version()
            ),
        )]);
    }
    let shows = |defence: Defence| displayed.iter().any(|(d, _, _)| *d == defence);
    let fits: Vec<&Base> = rows
        .iter()
        .filter(|base| !base.is_empty() && base.ranges().all(|(defence, _)| shows(defence)))
        .collect();
    let base = match fits.as_slice() {
        [base] => *base,
        [] => {
            return open(vec![gap(
                PercentileGap::BaseDefenceNotDisplayed,
                if rows.iter().all(|base| base.is_empty()) {
                    format!(
                        "`{name}` ranges no defence in the defence table (defences v{})",
                        table.version()
                    )
                } else {
                    format!(
                        "no row of `{name}` in the defence table has every defence it ranges displayed (defences v{})",
                        table.version()
                    )
                },
            )]);
        }
        fits => {
            return open(vec![gap(
                PercentileGap::BaseHeldMoreThanOnce,
                format!(
                    "the defence table holds `{name}` {} times, and the defences displayed are those of {} of its rows (defences v{})",
                    rows.len(),
                    fits.len(),
                    table.version()
                ),
            )]);
        }
    };

    // the arrays: one that is no array, or an element that is no line,
    // may hold a line the field reads (rule 8) — a crucible node's too,
    // J29 being the totals' rule and not this field's
    let reasons: Vec<Cow<'_, Unread>> = item
        .unread
        .iter()
        .filter(|u| match &u.part {
            Part::Lines(_) => true,
            Part::Field(key) => key == "hybrid",
            _ => false,
        })
        .map(Cow::Borrowed)
        .collect();
    if !reasons.is_empty() {
        return open(reasons);
    }

    // the lines: which are the item's own (J8, J33)
    let mark = mark_of(item);
    let mut own: Vec<Local> = Vec::new();
    // a line whose side is not established: its reason, and the line as
    // it would count were it the item's own
    let mut sides: Vec<(Option<PercentileGap>, &Line, Option<Local>)> = Vec::new();
    let mut reasons: Vec<Cow<'_, Unread>> = Vec::new();
    let mut taken: Vec<&Line> = Vec::new();
    let mut quality_dropped = false;
    for (at, line) in item.lines.iter().enumerate() {
        // a mod displayed over several rows is read row by row
        let names: Vec<(&str, &[Option<f64>])> = if line.rows.is_empty() {
            vec![(line.template.as_str(), line.numbers.as_slice())]
        } else {
            line.rows
                .iter()
                .map(|row| (row.template.as_str(), row.numbers.as_slice()))
                .collect()
        };
        for (template, numbers) in names {
            let Some(row) = table.line(template) else {
                continue;
            };
            if row.does == Does::DropQuality {
                // J6: under the enchant the quality term is dropped —
                // wherever it sits: it is no defence line, and is not
                // asked whose it is
                quality_dropped = true;
                taken.push(line);
                continue;
            }
            // a line of a type no roll reads changes no roll, whatever
            // stat it is (rule 8): it opens nothing
            let to: Vec<Defence> = row
                .to
                .iter()
                .copied()
                .filter(|defence| base.ranged(*defence))
                .collect();
            if to.is_empty() {
                continue;
            }
            // whose the line is; `None` where it is the global stat's
            let side: Option<Result<(), Option<PercentileGap>>> = match line.source.as_str() {
                "crucible" => Some(Err(Some(PercentileGap::CrucibleLine))),
                _ if !row.twin => Some(Ok(())),
                "explicit" => Some(Ok(())),
                "implicit" => match &mark {
                    Mark::Eldritch => None,
                    Mark::Synthesised => Some(Ok(())),
                    Mark::Neither => Some(Err(Some(PercentileGap::TwinOfNeitherMark))),
                    Mark::Both => Some(Err(Some(PercentileGap::TwinOfBothMarks))),
                    Mark::Unread(_) => Some(Err(None)),
                },
                _ => Some(Err(Some(PercentileGap::TwinInAnotherArray))),
            };
            let Some(side) = side else {
                continue;
            };
            // the `reduced` spelling is its stat below nothing
            let local = numbers.first().copied().flatten().map(|number| {
                let number = Ratio::of(Exact::of(match row.does {
                    Does::Reduce => -number.abs(),
                    Does::Add | Does::Increase | Does::DropQuality => number,
                }));
                Local {
                    does: row.does,
                    to: to.clone(),
                    number,
                }
            });
            match (side, local) {
                (Ok(()), Some(local)) => {
                    own.push(local);
                    taken.push(line);
                }
                // the item's own line, its number unread: the deriver's
                // own reason, of this occurrence
                (Ok(()), None) => {
                    let before = reasons.len();
                    reasons.extend(
                        item.unread
                            .iter()
                            .filter(|u| u.line == Some(at) && matches!(u.part, Part::Numbers(_)))
                            .map(Cow::Borrowed),
                    );
                    if reasons.len() == before {
                        reasons.push(Cow::Owned(Unread {
                            part: Part::Numbers(line.source.clone()),
                            problem: format!("`{}`: a number the search does not read", line.text),
                            line: Some(at),
                            name: None,
                            socket: None,
                        }));
                    }
                }
                (Err(why), local) => sides.push((why, line, local)),
            }
        }
    }
    let quality_term = match quality {
        Some((number, _)) if !quality_dropped => number,
        _ => 0,
    };
    let shown_of = |defence: Defence| {
        displayed
            .iter()
            .find(|(d, _, _)| *d == defence)
            .map(|(_, number, _)| *number)
    };
    if !sides.is_empty() {
        // what each reading would give, where both can be worked
        let said = |rolled: Rolled| match rolled {
            Rolled::Value(value) => value.to_string(),
            Rolled::Open(_) | Rolled::TooLarge => "no value".to_string(),
        };
        let readings = if reasons.is_empty() && sides.iter().all(|(_, _, local)| local.is_some()) {
            let without = said(rolled(base, &shown_of, quality_term, &own));
            let with: Vec<Local> = own
                .iter()
                .chain(sides.iter().filter_map(|(_, _, local)| local.as_ref()))
                .cloned()
                .collect();
            format!(
                ": {} were it the item's own, {without} were it left out",
                said(rolled(base, &shown_of, quality_term, &with))
            )
        } else {
            String::new()
        };
        for (why, line, _) in &sides {
            let text = line.text.replace('\n', " / ");
            match why {
                Some(PercentileGap::CrucibleLine) => reasons.push(gap(
                    PercentileGap::CrucibleLine,
                    format!(
                        "`{text}` is a defence line of a crucible node, and the trade site has not been asked whether it reads one{readings}"
                    ),
                )),
                Some(PercentileGap::TwinInAnotherArray) => reasons.push(gap(
                    PercentileGap::TwinInAnotherArray,
                    format!(
                        "`{text}` is a text a local stat and a global one both display, among the item's {} lines, where no ruling says which it is{readings}",
                        line.source
                    ),
                )),
                Some(PercentileGap::TwinOfBothMarks) => reasons.push(gap(
                    PercentileGap::TwinOfBothMarks,
                    format!(
                        "the implicit `{text}` is a text a local stat and a global one both display, on an item both eldritch and synthesised{readings}"
                    ),
                )),
                Some(why) => reasons.push(gap(
                    *why,
                    format!(
                        "the implicit `{text}` is a text a local stat and a global one both display, on an item neither eldritch nor synthesised{readings}"
                    ),
                )),
                // the mark is the deriver's to say unread, once
                None => {
                    if let Mark::Unread(unread) = &mark {
                        for u in unread {
                            if !reasons.iter().any(|seen| std::ptr::eq(seen.as_ref(), *u)) {
                                reasons.push(Cow::Borrowed(*u));
                            }
                        }
                    }
                }
            }
        }
    }
    if !reasons.is_empty() {
        return open(reasons);
    }

    // the rolls
    let value = match rolled(base, &shown_of, quality_term, &own) {
        Rolled::Value(value) => value,
        Rolled::Open(gaps) => {
            return open(
                gaps.into_iter()
                    .map(|(why, problem)| gap(why, problem))
                    .collect(),
            );
        }
        Rolled::TooLarge => return open(vec![too_large(&properties)]),
    };
    let Some(value) = Exact::whole(value) else {
        return open(vec![too_large(&properties)]);
    };
    let mut shown: Vec<&Property> = displayed
        .iter()
        .filter(|(defence, _, _)| base.ranged(*defence))
        .map(|(_, _, property)| *property)
        .chain(quality.map(|(_, property)| property))
        .collect();
    shown.sort_by_key(|p| item.properties.iter().position(|q| std::ptr::eq(q, *p)));
    let mut lines: Vec<&Line> = Vec::new();
    for line in taken {
        if !lines.iter().any(|seen| std::ptr::eq(*seen, line)) {
            lines.push(line);
        }
    }
    Ok(Percentile {
        value,
        shown,
        lines,
    })
}

/// The reason of whole numbers past what the search computes with: no
/// displayed item's, said as a product the units cannot hold is said
/// (`exact.rs`) — unread to what asked it, never rounded.
fn too_large(properties: &Part) -> Cow<'static, Unread> {
    Cow::Owned(Unread {
        part: properties.clone(),
        problem: "the defences displayed and the lines beside them give a roll of more digits than the search computes with".to_string(),
        line: None,
        name: None,
        socket: None,
    })
}

/// The value the rolls behind a display give (the module doc, C101, J5,
/// J33, J34), in whole numbers alone. A defence displays (roll + flat) ×
/// (1 + increase) × (1 + quality), rounded half up, so the rolls of a
/// display `d` are the whole numbers from (d − ½) ÷ mult − flat up to
/// (d + ½) ÷ mult − flat, both ends in: the upper end is the display at
/// an exact half, which J5 admits. Each roll's percentile rises with the
/// roll, so the least rolls give the least value and the most the most,
/// and the display fixes the value exactly when the two are one.
fn rolled(
    base: &Base,
    shown: &dyn Fn(Defence) -> Option<i128>,
    quality: i128,
    own: &[Local],
) -> Rolled {
    let work = || -> Option<Rolled> {
        let hundred = Ratio::whole(100);
        let one = Ratio::whole(1);
        let half = Ratio::new(1, 2)?;
        let quality = one.plus(Ratio::whole(quality).over(hundred)?)?;
        let mut open: Vec<(PercentileGap, String)> = Vec::new();
        let (mut least, mut most) = (Ratio::whole(0), Ratio::whole(0));
        let mut defences: i128 = 0;
        for (defence, (low, high)) in base.ranges() {
            // every defence the base ranges is displayed: the base was
            // chosen so
            let display = Ratio::whole(shown(defence)?);
            let (mut flat, mut increase) = (Ratio::whole(0), Ratio::whole(0));
            for local in own.iter().filter(|local| local.to.contains(&defence)) {
                match local.does {
                    Does::Add => flat = flat.plus(local.number)?,
                    Does::Increase | Does::Reduce => increase = increase.plus(local.number)?,
                    Does::DropQuality => {}
                }
            }
            let mult = one.plus(increase.over(hundred)?)?.times(quality)?;
            if !mult.positive() {
                // the lines take the defence to nothing or below, and
                // every roll displays the same: the display does not fix
                // the value
                open.push((
                    PercentileGap::SeveralValues,
                    format!(
                        "the item's lines take `{}` to nothing, whatever its roll",
                        defence.property()
                    ),
                ));
                continue;
            }
            let from = display.minus(half)?.over(mult)?.minus(flat)?.ceil();
            let to = display.plus(half)?.over(mult)?.minus(flat)?.floor();
            let rolls = (from <= to).then_some((from, to));
            let Some((from, to)) = rolls else {
                open.push((
                    PercentileGap::NoRoll,
                    format!(
                        "no whole number is a roll that displays `{}` {}",
                        defence.property(),
                        display.floor()
                    ),
                ));
                continue;
            };
            if low == high {
                open.push((
                    PercentileGap::MinimumIsMaximum,
                    format!(
                        "the base's minimum `{}` is its maximum, {low}: a roll has no place in such a range",
                        defence.property()
                    ),
                ));
                continue;
            }
            let (low, width) = (i128::from(low), i128::from(high) - i128::from(low));
            let place = |roll: i128| -> Option<Ratio> {
                Ratio::new(roll.checked_sub(low)?.checked_mul(100)?, width)
            };
            least = least.plus(place(from)?)?;
            most = most.plus(place(to)?)?;
            defences += 1;
        }
        if !open.is_empty() {
            return Some(Rolled::Open(open));
        }
        let count = Ratio::whole(defences);
        let (least, most) = (least.over(count)?.half_up()?, most.over(count)?.half_up()?);
        Some(if least == most {
            Rolled::Value(least)
        } else {
            Rolled::Open(vec![(
                PercentileGap::SeveralValues,
                format!(
                    "more than one roll gives what is displayed, and they give values from {least} to {most}"
                ),
            )])
        })
    };
    work().unwrap_or(Rolled::TooLarge)
}

// ---- the vocabulary -----------------------------------------------------------------------------------

/// The computed values a vocabulary narrowing matches by name or by
/// their own definition ([`OWN`]), never by the rule every total's
/// definition prints — none when it narrows by nothing (the module doc)
/// — each with the term that selects the items carrying it.
pub(crate) fn matching(
    narrow: Option<&(Op, String)>,
) -> Result<Vec<(String, Named, Node)>, LanguageError> {
    let Some((op, text)) = narrow else {
        return Ok(Vec::new());
    };
    let test = crate::bind::text_test("line", *op, &Value::Text(text.clone()))?;
    let mut out = Vec::new();
    for (entry, own) in DESCRIBED.iter().zip(OWN.iter()) {
        if !(test.holds(&entry.name) || test.holds(own)) {
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
                    leak(format!("{name}>=30")),
                    leak(format!("{name}.low>=10 {name}.high<=60")),
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
                leak(format!("{name}>={}", derived.example())),
                leak(format!("-has:{name}")),
                leak(format!("undecided({name})")),
            ],
            name,
            kind: "derived",
            what: derived.definition(),
            values: Vec::new(),
        });
    }
    out
});

/// What a narrowing is matched against of each computed value, in
/// [`DESCRIBED`]'s order: its definition as printed — but a total's own
/// (`Total::own_definition`), without the rule the table states of every
/// total (J29), which names no total and would list every one under a
/// word of it (T5: a narrowing lists those it matches).
/// A derived field's is its own too (`Derived::own_definition`): the
/// percentile's without the bookkeeping its definition prints.
static OWN: LazyLock<Vec<String>> = LazyLock::new(|| {
    let totals = totals::table().ok();
    DESCRIBED
        .iter()
        .map(|entry| {
            let name = entry.name.trim_start_matches("pseudo.");
            if let Some(derived) = Derived::ALL.into_iter().find(|d| d.name() == name) {
                return derived.own_definition();
            }
            totals
                .and_then(|table| table.get(name))
                .map_or_else(|| entry.what.clone(), Total::own_definition)
        })
        .collect()
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
            category: crate::category::Category::none(),
        }
    }

    fn table(totals: &str) -> &'static TotalsTable {
        let text = format!(
            "version = 6\nrealms = [\"pc\"]\nsource = \"s\"\ngenerated_by = \"t\"\nleaves_out = []\nleaves_out_rule = \"\"\n{totals}"
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
            "no totals table for realm poe2 (totals v6 covers pc)"
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
                "no totals table for realm poe2 (totals v6 covers pc)"
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
