//! The totals table (C94; the build plan, step 7): a named total is a sum
//! that answers, defined once as reviewed reference data (C68, C106), so
//! that it has one meaning on every surface.
//!
//! The words are `search/DESIGN.md`'s reference (*Values*, *A sum's
//! status*); the rule that decides a total is this doc's (C94, taken from
//! the contract detail 2026-09-23). How a computed value is asked for,
//! evaluated and shown is `pseudo.rs`; what reference data admits, and the
//! admission test the table passed, is the contract detail (C106), cited
//! and not restated.
//!
//! # Decisions as recorded
//!
//! - **C94 — the totals table.** A total's row names the contributing
//!   line — its template, exact; its source and flag where they matter,
//!   any otherwise — the slot of it or none, and a weight. For each
//!   occurrence the row names, a row with a slot contributes that slot's
//!   value times its weight: an all-elemental resistance line with `arg1`
//!   20 and weight 3 contributes 60 to `pseudo.total_res`; a row without a
//!   slot contributes its weight per occurrence, so weight 1 counts them.
//!   A readable line absent from every row is not an unknown contributor:
//!   the total answers its declared definition. A total of nothing is
//!   lacked — no line a row names on the item, or lines that cancel, `+21
//!   to Strength` beside `-21` — as the trade site shows no pseudo for
//!   either: known absence, false under a comparison and never zero
//!   (C93), asked by `has:` (owner, 2026-09-26: "Yes, let's make it absent
//!   to match the site"; of both cases alike: "A". It takes back "a total
//!   of nothing is an honest zero", as this entry read until then). A
//!   `sum( … )` of the same lines is the user's arithmetic and stays a
//!   number, zero of nothing (C95). A total whose rows are
//!   ranged lines sums low with low and high with high and is a ranged
//!   value taking `low`, `high` and `avg`. A contributor the search could
//!   not read — a source a row admits unread, an occurrence a row names
//!   whose number is unread, or whose flag is where the row asks one —
//!   leaves a subtotal marked incomplete, never a total: its comparison
//!   is undecided, `undecided( … )` of it is true (`pseudo.rs`). A flag
//!   that is no yes or no opens nothing a row does not ask, and no row of
//!   the shipped table asks one: the line counts by its text (J32, owner,
//!   2026-10-08: "yes, i agree"; the plan, rule 8). No definition for the item's realm is never a
//!   zero: the value is unavailable with that reason, undecided as an
//!   incomplete one is, and the reason names the realm and the versions
//!   (rule 8 of the plan: said once, where the table is read). The answer
//!   prints a definition on request (`--describe pseudo.<name>`, C97),
//!   and the table is checked, loudly, when it is built: a table that
//!   fails is a build defect surfaced as [`TotalsTableError`] to every
//!   caller, never a partial table.
//! - **C94 — the ranged family** (the build plan, 9d-C; taken from the
//!   contract detail 2026-10-08; `search/LEDGER.md`, "Holes ruled", the
//!   9c and 9d rows). The unsuffixed `Adds # to # <type> Damage` displays
//!   one of two stats: a weapon's own line feeds the attacks' total
//!   alone, the other twin the plain, the attacks' and the spells'
//!   (`search/pseudo-stats/README.md`, rounds two to four). Which it is
//!   the item's category decides: on a weapon the line is the weapon's
//!   own (J7, ruled 2026-10-01, the owner's assent to the builder's
//!   recommendation). A ranged total asked with no slot word is its
//!   `avg`, the one number the site shows (owner, 2026-09-26: "yes, bare
//!   means average"). Every ranged total prints the limit — a line under
//!   the other stat on a weapon is read as the weapon's own — with what
//!   the site showed: no listed weapon was found to carry one (round i,
//!   i06–i08). Where both twins count toward a total at one weight its
//!   row asks nothing of the item, so only the plain and the spells' rows
//!   read its category, and an item whose category is not established is
//!   undecided there alone: a total none of whose rows reads the item's
//!   category is never left open by it (J30). A ranged total is nothing,
//!   and so lacked, only where its low and its high both are — a range
//!   that cancels in its average alone, `-5` to `5`, is a value whose
//!   `avg` is 0 (J31). J30 and J31 are the builder's recommendations, the
//!   owner's assent to each 2026-10-08: "yes and yes".
//! - **C94 — what no total reads (J29).** A total leaves out a crucible
//!   node's lines and a flask's own, the arrays `crucibleMods` and
//!   `utilityMods` and no other: the trade site files them under no stat
//!   and counts neither (round n; the owner, 2026-10-08, of both: "Leave
//!   both out."). It is a rule of every total, the ones shipped before
//!   the ranged family among them and the readings by the totals they
//!   read; an array a total does not read cannot leave that total
//!   incomplete (rule 8 of the plan). A line's term, the item's own
//!   `sum( … )` of a line and the vocabulary read both arrays as they
//!   read any: the rule is a total's.
//!
//! # As built
//!
//! - **The table is `reference/totals-v6.toml`** beside this crate,
//!   compiled in by `include_str!` and parsed once, on first use
//!   ([`table`]); generated by `tools/totals-table.py` from the C++ app's
//!   pseudomod tables (`src/pseudomods.cpp` at `master@946a4f51`, the
//!   extract `search/cpp-search/data/pseudomods.toml`) and from what the
//!   trade site itself answered of each pseudo
//!   (`search/pseudo-stats/data/table-changes.csv`: a change a row, each
//!   with the captures that ask for it), reviewed as a diff at each
//!   change, pinned by this module's tests (the shipped file loads at
//!   version 6; the loader refuses what a reviewer would) and at the
//!   crate's boundary by `tests/pseudo.rs` and `tests/ranged.rs`, the
//!   second holding the build to every row of the ranged family's
//!   expected answers
//!   (`search/pseudo-stats/data/ranged-expected.csv`). It is *convention* under
//!   C106: every total counts what the trade site's pseudo of that name
//!   (`site`) counts (owner, 2026-09-26, V6 in `search/LEDGER.md`), never a
//!   judgment — but where the owner ruled the site wrong, and the change
//!   says `not mimicked`. The file's header and the script's docstring
//!   carry the rules a reviewer reads. A total's
//!   `limit` is what its text cannot match of the site's, printed with
//!   its definition: life regenerated is read from a line's text, which
//!   cuts what the site rounds (owner, 2026-09-27: "yes, let's go with
//!   what we can observe directly from the text we have."). The coverage
//!   trial the plan owed is `search/pseudo-stats/scripts/coverage-trial.py`
//!   (M6, `search/MEASUREMENTS.md`).
//! - **A row is a line's group** (`group::row`): bound once when the table
//!   is parsed, so what a row admits, selects and says of one occurrence
//!   is read as every other group's is (the build plan, step 4b), and a
//!   row's sum is the evaluator's own sum (`eval::sum`): one maker of a
//!   sum (rule 10 of the plan). So a row names a mod displayed over
//!   several rows by a row of it, as a line's term does (C90; the build
//!   plan, 9c4; owner, 2026-09-19: "the two-line mods should contribute to
//!   relevant pseudo-lines"), and reads that row's number: Thread of
//!   Hope's resistance, the second of its mod's three rows, is under every
//!   elemental total (`tests/pseudo.rs`, on the trade site's capture R1).
//!   The trade site's unit is the stat, and a stat whose rows another's
//!   sit between — Thread of Hope's mod is two — is named here by either
//!   of its rows or by the mod, never by the stat's own text: the
//!   translation's to say (C99). The table's realms are the file's: PoE1's
//!   three share one game and one recipe, and `poe2` has none.
//! - **A weight is a whole number or a half, above nothing or below**: a
//!   row's `reduced` spelling is the row at -1, as the trade site shows
//!   `-10% total Attack Speed` over `10% reduced Attack Speed` (step 9c2;
//!   owner, 2026-09-27: "yes, we need to be able to find reduced lines and
//!   totals"). It is carried in halves and
//!   applied exactly (`exact::Exact::halved`): the site's own
//!   `+94.5 total maximum Life` over `+90` life and `+9` Strength is the
//!   half (`search/pseudo-stats/README.md`). Units hold the half of a
//!   displayed number, of a sum of them and of a count, and not the half
//!   of a mean, so the loader refuses a half weight on a row whose slot is
//!   `avg` and on a ranged total, whose `avg` is a mean. A total is never
//!   rounded: `94.5` prints as `94.5`, and `>=94.5` is the comparison the
//!   reference's arithmetic rule gives it (*Slots*: exact in decimals).
//! - **What no total reads is said once, in the table's header** (J29):
//!   `leaves_out`, the arrays by GGG's key, and `leaves_out_rule`, the
//!   rule as the table's source states it — on no row, since it is no
//!   row's. The loader makes each array the source of lines it is and
//!   hands them to the one place a row becomes a group (`group::row`),
//!   which binds the row as its line in none of those sources: so a line
//!   there is no occurrence of any row, the group does not admit the
//!   source, and one of them unread leaves no total open — read from the
//!   group as every other fact of it is, with no case in the evaluator. A
//!   row's term, which a definition prints, stays the row as the table
//!   writes it, and every total's definition ([`Total::definition`]) and
//!   the table's own line ([`TotalsTable::provenance`]) print the rule
//!   once.
//! - **A row that reads what the item is** says so by `not_on`, the
//!   category id its line is not counted on (`Row::counts_on`): the 26
//!   rows of the unsuffixed line under the plain and the spells' totals
//!   name `weapon`, and no row of an attacks' total names anything (J30).
//!   What a weapon is, is category's answer for that id and nothing
//!   else's (`category::Category::asked`, as a term asks it; rule 10 of
//!   the plan): where the id holds the row counts nothing and reads
//!   nothing of the item; where it does not, the row is a row; where the
//!   category cannot say, a line the row names is a contribution unread —
//!   the total an incomplete subtotal, as under any contributor a row
//!   names that could not be read — and with no such line nothing is left
//!   open (rule 8: said once, at the row, of no more than was lost). The
//!   reasons are the category's own, made where a category term's are
//!   (`pseudo.rs`). A definition prints such a row with `where
//!   -category=<id>`, and says once what that means.
//! - **The loader checks what a reviewer would**: the version this build
//!   expects, every realm one GGG has, every array left out one the
//!   search names as a source of lines, once, and named by the rule
//!   beside it; every total named once by a name
//!   the language takes, every total with a row, every row's template one
//!   the tree admits — no sign before a `#`, no number typed into it — its
//!   source and flag legal words and its source none the table leaves
//!   out, its slot one the template has (or, on a
//!   ranged total, none, every template ranged), its weight finite,
//!   nonzero, a whole number or a half, its `not_on` an id of the
//!   category table, and no row named twice.
//! - **A reading is written apart from the totals** ([`Reading`]; C101,
//!   the build plan, step 9c3): a pseudo of the site's that is no sum of
//!   lines — how many of the resistance totals an item shows, the least of
//!   its three attribute totals — is no total and no row of one, and what
//!   it is on an item is `pseudo.rs`'s. What it reads is reference data
//!   as a total's rows are, evidenced by the same captures, generated by
//!   the same script and cited by the same version, so its definition
//!   lives here: its name, the site's id and text, its `kind` (`count` or
//!   `least`) and the totals it reads, `of`. The loader checks it as it
//!   checks a total: a name the language takes and nothing else has, a
//!   kind it knows, two totals or more, each a total of this file that is
//!   one number — never a reading, never a range — and none twice.
//! - **The version is on the basis** (C98; C106's clause (c)): `totals`
//!   beside `classes`, so two builds shipping different tables never label
//!   one answer alike.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, LazyLock};

use serde::{Deserialize, Serialize};

use crate::bind::SOURCES;
use crate::category::{self, Category, Holds, Ids};
use crate::corpus::REALMS;
use crate::derive::{Part, Unread};
use crate::error::LanguageError;
use crate::group::{self, Group, Truth};
use crate::tree::{self, Node};
use crate::{print, template};

/// The table this build ships, verbatim (the reviewed file).
pub const TOTALS_TABLE_TOML: &str = include_str!("../reference/totals-v6.toml");

/// The table version this build expects and every basis cites; a file
/// stamped otherwise is a build defect, not a runtime condition.
pub const TOTALS_TABLE_VERSION: u32 = 6;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    version: u32,
    realms: Vec<String>,
    source: String,
    #[allow(dead_code)]
    generated_by: String,
    /// The arrays no total reads (J29), by GGG's key.
    leaves_out: Vec<String>,
    /// The rule that names them, as the table's source states it.
    leaves_out_rule: String,
    #[serde(rename = "total", default)]
    totals: Vec<TotalIn>,
    #[serde(rename = "reading", default)]
    readings: Vec<ReadingIn>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TotalIn {
    name: String,
    /// The trade site's pseudo stat this total corresponds to, where one
    /// does: a citation the translation reads (C99), never a binding.
    site: Option<String>,
    /// The text the source prints the total as.
    text: Option<String>,
    #[serde(default)]
    ranged: bool,
    /// What a total read from an item's text cannot match of the site's.
    limit: Option<String>,
    rows: Vec<RowIn>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadingIn {
    name: String,
    site: Option<String>,
    text: Option<String>,
    kind: String,
    of: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowIn {
    template: String,
    source: Option<String>,
    flag: Option<String>,
    slot: Option<String>,
    weight: f64,
    /// The category id the row's line is not counted on.
    not_on: Option<String>,
}

/// The parsed, checked table. Read-only; one per build.
#[derive(Debug)]
pub struct TotalsTable {
    version: u32,
    realms: Vec<String>,
    source: String,
    /// The arrays no total reads (J29), and the rule that names them.
    leaves_out: Vec<String>,
    leaves_out_rule: Arc<str>,
    totals: Vec<Total>,
    by_name: HashMap<String, usize>,
    readings: Vec<Reading>,
    /// Why a total is unavailable in a realm the table does not cover, one
    /// reason per realm GGG has, made once so that a reason is borrowed as
    /// the deriver's are (`eval::why`).
    unavailable: BTreeMap<String, Unread>,
    /// The reason for a realm outside GGG's list.
    elsewhere: Unread,
}

/// One named total.
#[derive(Debug)]
pub struct Total {
    pub name: String,
    pub site: Option<String>,
    pub text: Option<String>,
    /// A ranged value, taking `low`, `high` and `avg`.
    pub ranged: bool,
    /// What a total read from an item's text cannot match of the site's,
    /// printed with the definition.
    pub limit: Option<String>,
    pub(crate) rows: Vec<Row>,
    /// The table's rule of what no total reads (J29), which every
    /// definition prints; empty where the table leaves nothing out.
    leaves_out_rule: Arc<str>,
}

/// What a reading makes of the totals it names (`pseudo.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadingKind {
    /// How many of them the item shows.
    Count,
    /// The smallest of them, where the item shows every one.
    Least,
}

/// One reading of other totals: a computed value that is no sum of lines
/// (C101).
#[derive(Debug)]
pub struct Reading {
    pub name: String,
    pub site: Option<String>,
    pub text: Option<String>,
    pub kind: ReadingKind,
    /// The totals read, by name, in the file's order.
    pub of: Vec<String>,
    /// Where each is among the table's totals.
    pub(crate) at: Vec<usize>,
}

/// What names a row once: its template, source, flag and slot.
type RowKey = (String, Option<String>, Option<String>, Option<String>);

/// The category id a row's line is not counted on (the module doc, "a
/// row that reads what the item is").
#[derive(Debug)]
pub(crate) struct NotOn {
    /// The id as the table writes it.
    pub id: String,
    /// The id as the category asks it.
    ids: Ids,
}

/// One contributing line.
#[derive(Debug)]
pub(crate) struct Row {
    /// The line as a term: what the definition prints.
    pub term: Node,
    pub group: Group,
    /// None counts occurrences.
    pub slot: Option<String>,
    /// The weight, in halves.
    pub halves: i64,
    /// Where the row reads what the item is: the category id its line is
    /// not counted on.
    pub not_on: Option<NotOn>,
}

impl Row {
    /// Whether the row counts its line on an item of this category (the
    /// module doc, "a row that reads what the item is"): yes for a row
    /// that asks nothing of the item, and for one that does where its id
    /// does not hold; no where it holds; undecided where the category
    /// cannot say. The question is the category's own, asked as a term
    /// asks it (rule 10 of the plan).
    pub(crate) fn counts_on(&self, category: &Category) -> Truth {
        match &self.not_on {
            None => Truth::True,
            Some(not_on) => match category.asked(not_on.ids) {
                Holds::Yes => Truth::False,
                Holds::No => Truth::True,
                Holds::Open => Truth::Undecided,
            },
        }
    }

    /// The ids the row asks the item's category by, where it asks.
    pub(crate) fn asks(&self) -> Option<Ids> {
        self.not_on.as_ref().map(|not_on| not_on.ids)
    }
}

/// Why a table text is not a table. Every arm names the offending row so a
/// reviewer can fix the file, not the loader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TotalsTableError {
    Parse(String),
    Version {
        found: u32,
        expected: u32,
    },
    UnknownRealm(String),
    NoRealm,
    NoTotal,
    BadName(String),
    DuplicateTotal(String),
    NoRows {
        total: String,
    },
    /// A row the language or the table's rules refuse, with the message.
    Row {
        total: String,
        template: String,
        problem: String,
    },
    DuplicateRow {
        total: String,
        template: String,
    },
    /// A reading the table's rules refuse, with the message.
    Reading {
        reading: String,
        problem: String,
    },
    /// What the table says no total reads, refused, with the message.
    LeavesOut(String),
}

impl std::fmt::Display for TotalsTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TotalsTableError::Parse(e) => write!(f, "the totals table does not parse: {e}"),
            TotalsTableError::Version { found, expected } => write!(
                f,
                "the totals table is version {found}; this build reads version {expected}"
            ),
            TotalsTableError::UnknownRealm(realm) => {
                write!(f, "the totals table names a realm GGG has not: {realm:?}")
            }
            TotalsTableError::NoRealm => write!(f, "the totals table names no realm"),
            TotalsTableError::NoTotal => write!(f, "the totals table has no total"),
            TotalsTableError::BadName(name) => write!(
                f,
                "the totals table names a total {name:?}, which is no name the language takes: one word of letters, digits and underscores"
            ),
            TotalsTableError::DuplicateTotal(name) => {
                write!(f, "the totals table names the total {name:?} twice")
            }
            TotalsTableError::NoRows { total } => {
                write!(f, "the totals table's {total:?} has no row")
            }
            TotalsTableError::Row {
                total,
                template,
                problem,
            } => write!(
                f,
                "the totals table's {total:?}, row {template:?}: {problem}"
            ),
            TotalsTableError::DuplicateRow { total, template } => write!(
                f,
                "the totals table's {total:?} names the row {template:?} twice"
            ),
            TotalsTableError::Reading { reading, problem } => {
                write!(f, "the totals table's reading {reading:?}: {problem}")
            }
            TotalsTableError::LeavesOut(problem) => {
                write!(f, "the totals table's `leaves_out`: {problem}")
            }
        }
    }
}

impl std::error::Error for TotalsTableError {}

static TABLE: LazyLock<Result<TotalsTable, TotalsTableError>> =
    LazyLock::new(|| TotalsTable::parse(TOTALS_TABLE_TOML));

/// The shipped table, parsed and checked once.
pub fn table() -> Result<&'static TotalsTable, &'static TotalsTableError> {
    TABLE.as_ref()
}

/// The hint on every total reason: the table, not the store, is what
/// could change (C93: a hint, never a guarantee).
pub const TOTAL_HINT: &str = crate::class::CLASS_HINT;

/// Why a total could not be established on an item, this table's part of
/// C93's closed list of reasons: the kind a count tallies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TotalGap {
    /// The item's realm has no definition in the table.
    NoDefinitionForRealm,
}

impl TotalGap {
    /// What was unread, as a reason and a tally name it (`eval::reason`).
    pub fn unread(self) -> &'static str {
        match self {
            TotalGap::NoDefinitionForRealm => "the total: no definition for the realm",
        }
    }
}

impl TotalsTable {
    /// Parse and check a table text. The shipped file goes through
    /// [`table`]; this is the same path over any text, for the tests
    /// that pin what the loader refuses.
    pub fn parse(text: &str) -> Result<TotalsTable, TotalsTableError> {
        let file: File =
            toml::from_str(text).map_err(|e| TotalsTableError::Parse(e.to_string()))?;
        if file.version != TOTALS_TABLE_VERSION {
            return Err(TotalsTableError::Version {
                found: file.version,
                expected: TOTALS_TABLE_VERSION,
            });
        }
        if file.realms.is_empty() {
            return Err(TotalsTableError::NoRealm);
        }
        if let Some(realm) = file.realms.iter().find(|r| !REALMS.contains(&r.as_str())) {
            return Err(TotalsTableError::UnknownRealm(realm.clone()));
        }
        if file.totals.is_empty() {
            return Err(TotalsTableError::NoTotal);
        }
        // J29: the arrays no total reads, each a source of lines the
        // language names, each named by the rule the definitions print
        let mut left_out: Vec<&'static str> = Vec::with_capacity(file.leaves_out.len());
        for array in &file.leaves_out {
            let source = array
                .strip_suffix("Mods")
                .and_then(|word| SOURCES.iter().find(|source| **source == word));
            let Some(source) = source else {
                return Err(TotalsTableError::LeavesOut(format!(
                    "{array:?} is no array of lines the search names: {}",
                    SOURCES
                        .iter()
                        .map(|source| format!("{source}Mods"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )));
            };
            if left_out.contains(source) {
                return Err(TotalsTableError::LeavesOut(format!(
                    "{array:?} is named twice"
                )));
            }
            if !file.leaves_out_rule.contains(&format!("`{array}`")) {
                return Err(TotalsTableError::LeavesOut(format!(
                    "the rule does not name `{array}`: {:?}",
                    file.leaves_out_rule
                )));
            }
            left_out.push(source);
        }
        if left_out.is_empty() != file.leaves_out_rule.trim().is_empty() {
            return Err(TotalsTableError::LeavesOut(
                "a rule with no array, or an array with no rule".to_string(),
            ));
        }
        let leaves_out_rule: Arc<str> = Arc::from(file.leaves_out_rule.trim());
        let mut totals: Vec<Total> = Vec::with_capacity(file.totals.len());
        let mut by_name: HashMap<String, usize> = HashMap::new();
        for total in file.totals {
            if !tree::is_segment(&total.name) {
                return Err(TotalsTableError::BadName(total.name));
            }
            if by_name.contains_key(&total.name) {
                return Err(TotalsTableError::DuplicateTotal(total.name));
            }
            if total.rows.is_empty() {
                return Err(TotalsTableError::NoRows { total: total.name });
            }
            let mut rows: Vec<Row> = Vec::with_capacity(total.rows.len());
            let mut named: Vec<RowKey> = Vec::new();
            for row in total.rows {
                let refuse = |problem: String| TotalsTableError::Row {
                    total: total.name.clone(),
                    template: row.template.clone(),
                    problem,
                };
                let key = (
                    row.template.clone(),
                    row.source.clone(),
                    row.flag.clone(),
                    row.slot.clone(),
                );
                if named.contains(&key) {
                    return Err(TotalsTableError::DuplicateRow {
                        total: total.name,
                        template: row.template,
                    });
                }
                named.push(key);
                if let Some(source) = row.source.as_deref()
                    && left_out.iter().any(|out| out.eq_ignore_ascii_case(source))
                {
                    return Err(refuse(format!(
                        "it names the source {source:?}, which the table leaves out of every total"
                    )));
                }
                let (term, group) = group::row(
                    &row.template,
                    row.source.as_deref(),
                    row.flag.as_deref(),
                    &left_out,
                )
                .map_err(|e: LanguageError| refuse(e.message))?;
                let not_on = match row.not_on.as_deref() {
                    None => None,
                    Some(id) => {
                        let tables = category::tables().map_err(|e| refuse(e.to_string()))?;
                        let Some(ids) = tables.one(id) else {
                            return Err(refuse(format!(
                                "`not_on` names {id:?}, which is no id of the category table"
                            )));
                        };
                        Some(NotOn {
                            id: id.to_string(),
                            ids,
                        })
                    }
                };
                let ranged_template = template::slots(&row.template).ranged.is_some();
                match (&row.slot, total.ranged) {
                    (Some(slot), false) => {
                        template::check_slot(&row.template, slot).map_err(|e| refuse(e.message))?;
                    }
                    (None, false) => {}
                    (Some(_), true) => {
                        return Err(refuse(
                            "a ranged total's row names no slot: low and high are its pair"
                                .to_string(),
                        ));
                    }
                    (None, true) if !ranged_template => {
                        return Err(refuse(
                            "a ranged total's rows are ranged lines: a template with exactly one `# to #`"
                                .to_string(),
                        ));
                    }
                    (None, true) => {}
                }
                let halves = row.weight * 2.0;
                if !halves.is_finite() || halves == 0.0 || halves.fract() != 0.0 {
                    return Err(refuse(format!(
                        "a weight is a whole number or a half, never {}",
                        row.weight
                    )));
                }
                let halves = halves as i64;
                if halves % 2 != 0 && (total.ranged || row.slot.as_deref() == Some("avg")) {
                    return Err(refuse(
                        "a half weight is on a displayed number, never on a mean: not on `avg`, and not on a ranged total"
                            .to_string(),
                    ));
                }
                rows.push(Row {
                    term,
                    group,
                    slot: row.slot,
                    halves,
                    not_on,
                });
            }
            by_name.insert(total.name.clone(), totals.len());
            totals.push(Total {
                name: total.name,
                site: total.site,
                text: total.text,
                ranged: total.ranged,
                limit: total.limit,
                rows,
                leaves_out_rule: leaves_out_rule.clone(),
            });
        }
        let mut readings: Vec<Reading> = Vec::with_capacity(file.readings.len());
        for reading in file.readings {
            let refuse = |problem: String| TotalsTableError::Reading {
                reading: reading.name.clone(),
                problem,
            };
            if !tree::is_segment(&reading.name) {
                return Err(TotalsTableError::BadName(reading.name));
            }
            if by_name.contains_key(&reading.name)
                || readings.iter().any(|r| r.name == reading.name)
            {
                return Err(TotalsTableError::DuplicateTotal(reading.name));
            }
            let kind = match reading.kind.as_str() {
                "count" => ReadingKind::Count,
                "least" => ReadingKind::Least,
                other => {
                    return Err(refuse(format!(
                        "a reading is a `count` or a `least`, never {other:?}"
                    )));
                }
            };
            if reading.of.len() < 2 {
                return Err(refuse(
                    "a reading reads two totals or more: one total is itself".to_string(),
                ));
            }
            let mut at: Vec<usize> = Vec::with_capacity(reading.of.len());
            for name in &reading.of {
                let Some(found) = by_name.get(name) else {
                    return Err(refuse(format!(
                        "it reads {name:?}, which is no total of this table"
                    )));
                };
                if totals[*found].ranged {
                    return Err(refuse(format!(
                        "it reads {name:?}, a range: a reading reads a total that is one number"
                    )));
                }
                if at.contains(found) {
                    return Err(refuse(format!("it reads {name:?} twice")));
                }
                at.push(*found);
            }
            readings.push(Reading {
                name: reading.name,
                site: reading.site,
                text: reading.text,
                kind,
                of: reading.of,
                at,
            });
        }
        let covered = file.realms.join(", ");
        let reason = |realm: &str| Unread {
            part: Part::Total(TotalGap::NoDefinitionForRealm),
            problem: format!(
                "no totals table for realm {realm} (totals v{} covers {covered})",
                file.version
            ),
            line: None,
            name: None,
            socket: None,
        };
        Ok(TotalsTable {
            unavailable: REALMS
                .iter()
                .filter(|r| !file.realms.iter().any(|c| c == *r))
                .map(|r| ((*r).to_string(), reason(r)))
                .collect(),
            elsewhere: reason("outside GGG's"),
            version: file.version,
            realms: file.realms,
            source: file.source,
            leaves_out: file.leaves_out,
            leaves_out_rule,
            totals,
            by_name,
            readings,
        })
    }

    /// The version every basis cites.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The realms the table covers.
    pub fn realms(&self) -> &[String] {
        &self.realms
    }

    /// The table-level source, verbatim.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The totals, in the file's order.
    pub fn totals(&self) -> &[Total] {
        &self.totals
    }

    /// A total by name, in any case (B1: every name is matched so).
    pub fn get(&self, name: &str) -> Option<&Total> {
        self.by_name
            .get(name)
            .or_else(|| {
                self.by_name
                    .iter()
                    .find(|(n, _)| n.eq_ignore_ascii_case(name))
                    .map(|(_, at)| at)
            })
            .map(|at| &self.totals[*at])
    }

    /// The readings, in the file's order.
    pub fn readings(&self) -> &[Reading] {
        &self.readings
    }

    /// A reading by name, in any case, as a total is found.
    pub fn reading(&self, name: &str) -> Option<&Reading> {
        self.readings.iter().find(|r| r.name == name).or_else(|| {
            self.readings
                .iter()
                .find(|r| r.name.eq_ignore_ascii_case(name))
        })
    }

    /// The totals a reading reads, in its order.
    pub(crate) fn read_by<'a>(&'a self, reading: &'a Reading) -> impl Iterator<Item = &'a Total> {
        reading.at.iter().map(|at| &self.totals[*at])
    }

    /// Why a total has no value in `realm`, when the table does not cover
    /// it: the one reason, made once.
    pub fn unavailable(&self, realm: &str) -> Option<&Unread> {
        if self.realms.iter().any(|r| r == realm) {
            return None;
        }
        Some(self.unavailable.get(realm).unwrap_or(&self.elsewhere))
    }

    /// The arrays no total reads (J29), by GGG's key.
    pub fn leaves_out(&self) -> &[String] {
        &self.leaves_out
    }

    /// What `--describe` says of the table as a whole: where it is from,
    /// with its version, and what no total reads (J29).
    pub fn provenance(&self) -> String {
        let rule = match &*self.leaves_out_rule {
            "" => String::new(),
            rule => format!("; {rule}"),
        };
        format!(
            "totals v{}, {} totals and {} readings of them over {}: {}{rule}",
            self.version,
            self.totals.len(),
            self.readings.len(),
            self.realms.join(", "),
            self.source
        )
    }
}

impl Total {
    /// Whether a row of the total reads what the item is.
    pub fn reads_category(&self) -> bool {
        self.rows.iter().any(|row| row.not_on.is_some())
    }

    /// The definition, as `--describe` prints it (C94: printed on
    /// request): the total's own, then what no total reads (J29), the
    /// table's rule, said of every total.
    pub fn definition(&self) -> String {
        match &*self.leaves_out_rule {
            "" => self.own_definition(),
            rule => format!("{}; {rule}", self.own_definition()),
        }
    }

    /// What this total is defined as, and no other: each row's weight,
    /// slot and line, and where it reads what the item is; then what the
    /// rows that do so mean, and the total's limit. What a vocabulary
    /// narrowing is matched against (`pseudo::matching`): the table's
    /// rule of every total names none.
    pub fn own_definition(&self) -> String {
        let rows: Vec<String> = self
            .rows
            .iter()
            .map(|row| {
                let weight = if row.halves % 2 == 0 {
                    (row.halves / 2).to_string()
                } else {
                    format!("{}", row.halves as f64 / 2.0)
                };
                let line = match &row.slot {
                    Some(slot) => format!("{weight} × {slot} of {}", print::print(&row.term)),
                    None if self.ranged => {
                        format!("{weight} × low and high of {}", print::print(&row.term))
                    }
                    None => format!("{weight} per {}", print::print(&row.term)),
                };
                match &row.not_on {
                    Some(not_on) => format!("{line} where {}", not_category(&not_on.id)),
                    None => line,
                }
            })
            .collect();
        let reads = if self.reads_category() {
            "; a row with `where` counts its line only on an item that term holds on, and its contribution is unread — the total undecided — where the item has such a line and its category is not established (C93)"
        } else {
            ""
        };

        let what = if self.ranged {
            "the sum over the item's lines, low with low and high with high — a range taking low, high and avg, the bare name its avg"
        } else {
            "the sum over the item's lines"
        };
        let site = match (&self.site, &self.text) {
            (Some(site), Some(text)) => format!("the trade site's {site} (`{text}`): "),
            (Some(site), None) => format!("the trade site's {site}: "),
            (None, Some(text)) => format!("`{text}`: "),
            (None, None) => String::new(),
        };
        let limit = match &self.limit {
            Some(limit) => format!("; {limit}"),
            None => String::new(),
        };
        format!("{site}{what} of {}{reads}{limit}", rows.join(" + "))
    }
}

/// `-category=<id>`, as the language writes it: what a row that reads
/// the item's category asks, printed by the printer.
fn not_category(id: &str) -> String {
    print::print(&Node::Not(Box::new(Node::Test {
        field: category::FIELD.to_string(),
        op: tree::Op::Eq,
        value: tree::Value::Text(id.to_string()),
    })))
}

/// `a, b and c`, as a definition lists what it reads.
fn listed(names: &[String]) -> String {
    let names: Vec<String> = names.iter().map(|n| format!("pseudo.{n}")).collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => names.join(""),
    }
}

impl Reading {
    /// The definition, as `--describe` prints it (C101: listed with its
    /// definition).
    pub fn definition(&self) -> String {
        let site = match (&self.site, &self.text) {
            (Some(site), Some(text)) => format!("the trade site's {site} (`{text}`): "),
            (Some(site), None) => format!("the trade site's {site}: "),
            (None, Some(text)) => format!("`{text}`: "),
            (None, None) => String::new(),
        };
        let of = listed(&self.of);
        match self.kind {
            ReadingKind::Count => format!(
                "{site}how many of {of} the item has, each as its own definition counts it; an item with none of them lacks it"
            ),
            ReadingKind::Least => format!(
                "{site}the least of {of} on an item that has every one of them, each as its own definition counts it; an item lacking any of them lacks it"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ReadingKind, TOTALS_TABLE_VERSION, TotalsTable, TotalsTableError, table};

    fn text(totals: &str) -> String {
        format!(
            "version = 6\nrealms = [\"pc\"]\nsource = \"s\"\ngenerated_by = \"t\"\nleaves_out = []\nleaves_out_rule = \"\"\n{totals}"
        )
    }

    const ONE: &str = "[[total]]\nname = \"total_res\"\nrows = [\n  { template = \"#% to Fire Resistance\", slot = \"arg1\", weight = 1 },\n]\n";

    /// The shipped file parses under the loader's checks, is the version
    /// this build cites, and every total prints a definition.
    #[test]
    fn the_shipped_table_loads_and_is_version_6() {
        let t = table().unwrap();
        assert_eq!(t.version(), TOTALS_TABLE_VERSION);
        assert_eq!(t.realms(), ["pc", "xbox", "sony"]);
        assert_eq!(t.totals().len(), 82);
        let res = t.get("total_res").unwrap();
        assert_eq!(res.site.as_deref(), Some("pseudo_total_resistance"));
        assert!(!res.ranged);
        assert!(
            res.definition()
                .contains("3 × arg1 of line(\"#% to all Elemental Resistances\")"),
            "{}",
            res.definition()
        );
        // a row's `reduced` spelling is the row, below nothing (step 9c2)
        let speed = t.get("total_attack_speed").unwrap().definition();
        assert!(
            speed.contains("-1 × arg1 of line(\"#% reduced Attack Speed\")"),
            "{speed}"
        );
        // a limit is printed with the definition (step 9c2), and after it
        // what no total reads (J29), as the table's rule states it
        let regen = t.get("life_regen").unwrap().definition();
        assert!(
            regen.ends_with(
                "of 1 × arg1 of line(\"Regenerate # Life per second\"); read from the item's text, which cuts what the trade site rounds: under the site's by 0.1 a line at most; a total leaves out a crucible node's lines and a flask's own lines: the arrays `crucibleMods` and `utilityMods`, and no other array"
            ),
            "{regen}"
        );
        assert_eq!(t.leaves_out(), ["crucibleMods", "utilityMods"]);
        assert!(t.provenance().ends_with(
            "; a total leaves out a crucible node's lines and a flask's own lines: the arrays `crucibleMods` and `utilityMods`, and no other array"
        ));
        // the ranged family (step 9d-C): 21 totals, 135 rows, 26 of which
        // read what the item is — the plain and the spells' totals' rows
        // of the unsuffixed line, and no row of an attacks' total (J30)
        let ranged: Vec<&super::Total> = t.totals().iter().filter(|t| t.ranged).collect();
        assert_eq!(ranged.len(), 21);
        assert!(ranged.iter().all(|t| t.name.starts_with("adds_")));
        assert_eq!(ranged.iter().map(|t| t.rows.len()).sum::<usize>(), 135);
        let reads = |t: &super::Total| t.rows.iter().filter(|r| r.not_on.is_some()).count();
        assert_eq!(ranged.iter().map(|t| reads(t)).sum::<usize>(), 26);
        assert!(
            ranged
                .iter()
                .all(|t| t.name.ends_with("_to_attacks") != t.reads_category())
        );
        assert!(t.totals().iter().all(|t| t.ranged || !t.reads_category()));
        assert_eq!(
            t.get("adds_fire").unwrap().definition(),
            "the trade site's pseudo_adds_fire_damage (`Adds # to # Fire Damage`): the sum over the item's lines, low with low and high with high — a range taking low, high and avg, the bare name its avg of 1 × low and high of line(\"Adds # to # Fire Damage\") where -category=weapon; a row with `where` counts its line only on an item that term holds on, and its contribution is unread — the total undecided — where the item has such a line and its category is not established (C93); a line under the other stat on a weapon is read as the weapon's own: no weapon the trade site listed was found to carry one (round i, i06–i08); a total leaves out a crucible node's lines and a flask's own lines: the arrays `crucibleMods` and `utilityMods`, and no other array"
        );
        assert!(t.get("TOTAL_RES").is_some());
        assert!(t.get("total_nothing").is_none());
        assert!(t.unavailable("pc").is_none());
        assert!(
            t.unavailable("poe2")
                .unwrap()
                .problem
                .contains("no totals table for realm poe2 (totals v6 covers pc, xbox, sony)")
        );
        assert!(t.unavailable("mars").is_some());
        // the four that are no sum, each a reading of totals the file writes
        // (step 9c3)
        let read: Vec<(&str, ReadingKind, Vec<&str>)> = t
            .readings()
            .iter()
            .map(|r| {
                (
                    r.name.as_str(),
                    r.kind,
                    t.read_by(r).map(|total| total.name.as_str()).collect(),
                )
            })
            .collect();
        let elemental = vec!["total_fire_res", "total_cold_res", "total_lightning_res"];
        let mut every = elemental.clone();
        every.push("total_chaos_res");
        assert_eq!(
            read,
            [
                ("count_res", ReadingKind::Count, every),
                ("count_ele_res", ReadingKind::Count, elemental.clone()),
                ("total_all_ele_res", ReadingKind::Least, elemental),
                (
                    "total_all_attributes",
                    ReadingKind::Least,
                    vec!["total_str", "total_dex", "total_int"]
                ),
            ]
        );
        assert!(t.reading("COUNT_RES").is_some());
        assert!(t.reading("total_res").is_none());
        assert!(t.get("count_res").is_none());
    }

    /// What the loader refuses, each by the row it names.
    #[test]
    fn the_loader_refuses_what_a_reviewer_would() {
        assert!(TotalsTable::parse(&text(ONE)).is_ok());
        for (bad, expected) in [
            (
                text(ONE).replace("version = 6", "version = 1"),
                TotalsTableError::Version {
                    found: 1,
                    expected: TOTALS_TABLE_VERSION,
                },
            ),
            (
                text(ONE).replace("[\"pc\"]", "[\"ps4\"]"),
                TotalsTableError::UnknownRealm("ps4".to_string()),
            ),
            (
                text(ONE).replace("[\"pc\"]", "[]"),
                TotalsTableError::NoRealm,
            ),
            (text(""), TotalsTableError::NoTotal),
            (
                text(ONE).replace("total_res", "total-res"),
                TotalsTableError::BadName("total-res".to_string()),
            ),
            (
                text(&format!("{ONE}{ONE}")),
                TotalsTableError::DuplicateTotal("total_res".to_string()),
            ),
            (
                text("[[total]]\nname = \"total_res\"\nrows = []\n"),
                TotalsTableError::NoRows {
                    total: "total_res".to_string(),
                },
            ),
            (
                text(
                    "[[total]]\nname = \"total_res\"\nrows = [\n  { template = \"#% to Fire Resistance\", slot = \"arg1\", weight = 1 },\n  { template = \"#% to Fire Resistance\", slot = \"arg1\", weight = 2 },\n]\n",
                ),
                TotalsTableError::DuplicateRow {
                    total: "total_res".to_string(),
                    template: "#% to Fire Resistance".to_string(),
                },
            ),
        ] {
            assert_eq!(TotalsTable::parse(&bad).unwrap_err(), expected, "{bad}");
        }
        // a row the language or the table's rules refuse, with the message
        for (edit, problem) in [
            ("weight = 1", "weight = 0"),
            ("weight = 1", "weight = 0.25"),
            ("slot = \"arg1\"", "slot = \"arg2\""),
            ("slot = \"arg1\"", "slot = \"avg\""),
            (
                "slot = \"arg1\", weight = 1",
                "slot = \"avg\", weight = 0.5",
            ),
            ("\"#% to Fire Resistance\"", "\"+#% to Fire Resistance\""),
            ("\"#% to Fire Resistance\"", "\"+48% to Fire Resistance\""),
            ("slot = \"arg1\"", "source = \"explicits\", slot = \"arg1\""),
            ("slot = \"arg1\"", "flag = \"corrupted\", slot = \"arg1\""),
        ] {
            let bad = text(&ONE.replace(edit, problem));
            match TotalsTable::parse(&bad) {
                Err(TotalsTableError::Row { total, .. }) => assert_eq!(total, "total_res"),
                other => panic!("{problem}: {other:?}"),
            }
        }
        // a half weight on a displayed number is admitted; on a mean, not
        let half = text(&ONE.replace("weight = 1", "weight = 0.5"));
        assert!(TotalsTable::parse(&half).is_ok());
        // a weight below nothing is admitted, whole or a half
        for below in ["weight = -1", "weight = -0.5"] {
            assert!(
                TotalsTable::parse(&text(&ONE.replace("weight = 1", below))).is_ok(),
                "{below}"
            );
        }
        let ranged = "[[total]]\nname = \"cold\"\nranged = true\nrows = [\n  { template = \"Adds # to # Cold Damage\", weight = 1 },\n]\n";
        assert!(TotalsTable::parse(&text(ranged)).is_ok());
        for (edit, problem) in [
            ("weight = 1", "weight = 0.5"),
            ("weight = 1", "slot = \"low\", weight = 1"),
            ("Adds # to # Cold Damage", "#% to Cold Resistance"),
        ] {
            assert!(
                TotalsTable::parse(&text(&ranged.replace(edit, problem)))
                    .unwrap_err()
                    .to_string()
                    .contains("\"cold\", row"),
                "{problem}"
            );
        }
        assert!(matches!(
            TotalsTable::parse("version = 6\nnope = 2"),
            Err(TotalsTableError::Parse(_))
        ));
        // a row that reads what the item is names an id of the category
        // table, and a definition says so on the row
        let reads = ranged.replace("weight = 1 }", "weight = 1, not_on = \"weapon\" }");
        let t = TotalsTable::parse(&text(&reads)).unwrap();
        assert!(t.get("cold").unwrap().reads_category());
        assert!(
            t.get("cold")
                .unwrap()
                .definition()
                .contains("line(\"Adds # to # Cold Damage\") where -category=weapon; a row with")
        );
        assert!(
            TotalsTable::parse(&text(&reads.replace("\"weapon\"", "\"weapons\"")))
                .unwrap_err()
                .to_string()
                .contains("`not_on` names \"weapons\", which is no id of the category table")
        );
        // what no total reads (J29): arrays of lines the search names,
        // each once, each named by the rule beside them — and by no row
        let leaves = |arrays: &str, rule: &str| {
            text(ONE).replace(
                "leaves_out = []\nleaves_out_rule = \"\"",
                &format!("leaves_out = [{arrays}]\nleaves_out_rule = \"{rule}\""),
            )
        };
        let t = TotalsTable::parse(&leaves("\"crucibleMods\"", "not `crucibleMods`")).unwrap();
        assert_eq!(t.leaves_out(), ["crucibleMods"]);
        assert!(
            t.get("total_res")
                .unwrap()
                .definition()
                .ends_with("; not `crucibleMods`")
        );
        for (arrays, rule, says) in [
            ("\"crucible\"", "not `crucible`", "is no array of lines"),
            ("\"stashMods\"", "not `stashMods`", "is no array of lines"),
            (
                "\"crucibleMods\", \"crucibleMods\"",
                "not `crucibleMods`",
                "named twice",
            ),
            ("\"crucibleMods\"", "not `utilityMods`", "does not name"),
            ("\"crucibleMods\"", "", "does not name"),
            ("", "not `crucibleMods`", "a rule with no array"),
        ] {
            match TotalsTable::parse(&leaves(arrays, rule)) {
                Err(e @ TotalsTableError::LeavesOut(_)) => {
                    assert!(e.to_string().contains(says), "{arrays} {rule}: {e}")
                }
                other => panic!("{arrays} {rule}: {other:?}"),
            }
        }
        let named = leaves("\"crucibleMods\"", "not `crucibleMods`")
            .replace("slot = \"arg1\"", "source = \"crucible\", slot = \"arg1\"");
        assert!(
            TotalsTable::parse(&named)
                .unwrap_err()
                .to_string()
                .contains("which the table leaves out of every total")
        );
        // a definition prints a half as written and a count as `per`
        let t = TotalsTable::parse(&text(
            "[[total]]\nname = \"life\"\nrows = [\n  { template = \"# to maximum Life\", slot = \"arg1\", weight = 1 },\n  { template = \"# to Strength\", slot = \"arg1\", weight = 0.5 },\n  { template = \"Has # Abyssal Sockets\", weight = 1 },\n]\n",
        ))
        .unwrap();
        assert_eq!(
            t.get("life").unwrap().definition(),
            "the sum over the item's lines of 1 × arg1 of line(\"# to maximum Life\") + 0.5 × arg1 of line(\"# to Strength\") + 1 per line(\"Has # Abyssal Sockets\")"
        );
    }

    /// What the loader refuses of a reading, each by the reading it names.
    #[test]
    fn the_loader_refuses_a_reading_a_reviewer_would() {
        const TWO: &str = "[[total]]\nname = \"fire\"\nrows = [\n  { template = \"#% to Fire Resistance\", slot = \"arg1\", weight = 1 },\n]\n[[total]]\nname = \"cold\"\nrows = [\n  { template = \"#% to Cold Resistance\", slot = \"arg1\", weight = 1 },\n]\n[[total]]\nname = \"adds\"\nranged = true\nrows = [\n  { template = \"Adds # to # Cold Damage\", weight = 1 },\n]\n";
        const READ: &str =
            "[[reading]]\nname = \"both\"\nkind = \"count\"\nof = [\"fire\", \"cold\"]\n";
        let t = TotalsTable::parse(&text(&format!("{TWO}{READ}"))).unwrap();
        let both = t.reading("both").unwrap();
        assert_eq!(both.kind, ReadingKind::Count);
        assert_eq!(
            both.definition(),
            "how many of pseudo.fire and pseudo.cold the item has, each as its own definition counts it; an item with none of them lacks it"
        );
        for (edit, problem, says) in [
            (
                "kind = \"count\"",
                "kind = \"most\"",
                "a `count` or a `least`",
            ),
            ("[\"fire\", \"cold\"]", "[\"fire\"]", "two totals or more"),
            ("[\"fire\", \"cold\"]", "[]", "two totals or more"),
            (
                "[\"fire\", \"cold\"]",
                "[\"fire\", \"lightning\"]",
                "\"lightning\", which is no total of this table",
            ),
            (
                "[\"fire\", \"cold\"]",
                "[\"fire\", \"both\"]",
                "\"both\", which is no total of this table",
            ),
            ("[\"fire\", \"cold\"]", "[\"fire\", \"adds\"]", "a range"),
            ("[\"fire\", \"cold\"]", "[\"fire\", \"fire\"]", "twice"),
        ] {
            let bad = text(&format!("{TWO}{}", READ.replace(edit, problem)));
            match TotalsTable::parse(&bad) {
                Err(e @ TotalsTableError::Reading { .. }) => {
                    assert!(e.to_string().contains("reading \"both\""), "{e}");
                    assert!(e.to_string().contains(says), "{problem}: {e}");
                }
                other => panic!("{problem}: {other:?}"),
            }
        }
        // one namespace: a reading named as a total is, or as a reading is
        for (bad, expected) in [
            (
                READ.replace("\"both\"", "\"fire\""),
                TotalsTableError::DuplicateTotal("fire".to_string()),
            ),
            (
                format!("{READ}{READ}"),
                TotalsTableError::DuplicateTotal("both".to_string()),
            ),
            (
                READ.replace("\"both\"", "\"bo-th\""),
                TotalsTableError::BadName("bo-th".to_string()),
            ),
        ] {
            assert_eq!(
                TotalsTable::parse(&text(&format!("{TWO}{bad}"))).unwrap_err(),
                expected,
                "{bad}"
            );
        }
        // a field the loader does not know is refused, as a total's is
        let unknown = READ.replace("kind =", "weight = 1\nkind =");
        assert!(matches!(
            TotalsTable::parse(&text(&format!("{TWO}{unknown}"))),
            Err(TotalsTableError::Parse(_))
        ));
    }
}
