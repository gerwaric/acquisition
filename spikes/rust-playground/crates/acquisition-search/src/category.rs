//! Category (C106; the build plan, step 9d-B): the trade site's grouping
//! above class — its 82 category ids, as spelled — computed from the
//! item's class and reviewed rules in this one place (rule 10 of the
//! plan), never by a second classifier. One question is asked of an item
//! here — does this id hold: yes, no, or undecided with its reason — and
//! the term, the count, `show` and the help all ask it.
//!
//! What reference data admits, and the admission test both tables passed,
//! is the crate's doc (C106 — what reference data admits, and the
//! test), cited and not restated. The class is `class.rs`'s
//! and the buckets of a count are `counts.rs`'s (C105).
//!
//! # Decisions as recorded
//!
//! **C106 — category.** The rulings are `search/LEDGER.md`, "Holes
//! ruled", the 9d rows (J1–J4, J9–J22), each the owner's; his words are
//! quoted where they state the rule.
//!
//! - **A term names the trade site's category id as spelled** — the ids
//!   of the site's filter as the owner captured it 2026-09-12, in the
//!   site's order, each with the site's text for it. The word is the trade
//!   site's and the help says so (the owner at the first seat, V1: "I'd
//!   like to mimic the trade site as much as practically feasible, because
//!   that's how most players will think.").
//! - **A leaf is a class's, or a class's with a rule on the base name.**
//!   An item of the class has the leaf; where a name rule of that class
//!   holds the item's base, it has the rule's leaf in place of the
//!   class's own (an Awakened support gem is `gem.supportgemplus` and not
//!   `gem.supportgem`). A name rule is one clause on a name within one
//!   class; a case one clause cannot say is an exception row citing its
//!   capture, never a second clause (the owner's ruling, 2026-10-07) —
//!   none is needed today, and the generator stops on a base that holds a
//!   leaf's word and misses its rule unless the miss is named with its
//!   reason (one: `Fossilised Delirium Orb`). The Maven's Invitation leaf
//!   holds a base named for the Maven, and every other invitation takes
//!   `map`, the second half not asked of the site (J3). A Stackable
//!   Currency base whose name holds the word `Tattoo` is
//!   `currency.tattoo`, the site listing Honoured and Journey tattoos
//!   there and the answer extended to the word for the families not asked
//!   (J23; round m). A class no id is keyed by gives none: a
//!   quest-frame invitation, whose class is Quest Items, has no id, its
//!   class deciding (J19).
//! - **A parent matches the classes the site puts at it and every id
//!   under it by the dot** (J2): `map` holds Inscribed Ultimatums of its
//!   own and its children's scarabs and breachstones alike. The dot is in
//!   the ids themselves; `map` and `currency` are a parent and a class's
//!   own id at once.
//! - **A grouping** — `weapon.one` and the six others, whose members no
//!   dot names — **is a term over reviewed members**, each row citing what
//!   answered it, Fishing Rods outside two-handed melee (J4). It holds its
//!   reviewed members alone, a leaf nothing places being out (J12).
//! - **A captured beast, known by its bestiary line, is `monster.beast`,
//!   and its colour leaf is the row of the colour table** (J1): reference
//!   data under clause (a) of the admission test, generated from the
//!   sittings' captures, 250 yellow, 79 red, 32 unknown, each row citing
//!   its capture. A name the table lacks or marks unknown is undecided
//!   with that reason and the hint that the site may list it (the owner,
//!   2026-10-05: "Let's leave the undecided beasts undecided."). The count
//!   of bestiary mods is the track's check and reads nothing at run time.
//! - **A beast's class gives it no id** (J10): the class is the game's
//!   word, Stackable Currency (J9, `class.rs`), and the beast is under
//!   `monster.beast` and its colour leaf, never `currency` (the owner,
//!   2026-10-06: "no, let's not count captured beasts as currency. They
//!   are "Captured Beast" and either "Red Beast" or "Yellow Beast" or
//!   neither.").
//! - **A count by category lists an item under its most specific id**
//!   (J11: "under the most specific. consumers can roll up these counts as
//!   needed."): the ids that hold with no id beneath them holding — a
//!   breachstone, `map` twice over, under `map.breachstone`. An item of
//!   two leaves not nested is listed under each, as a count by class lists
//!   it (J16); a beast of undecided colour under `monster.beast` (J15);
//!   an item whose class is open under the most specific id that holds
//!   (J21).
//! - **What is undecided, and what is known absence** (C93, C105). An id
//!   every candidate class gives holds while the class is open (J17): a
//!   resonator with no frame is `currency.resonator`, both its classes
//!   giving it. A base the class table cannot class — the grafts among
//!   them, whose class the export prints no name for (J13) — leaves every
//!   id a class could give undecided, and the three beast ids false, the
//!   item carrying no bestiary line. In a realm with no table every id is
//!   undecided on every item, the beast ids among them (J18, J22). An item
//!   whose class gives no id lacks a category: `none`.
//! - **Growing Wombgift is `wombgift`** by its class, which `class.rs`
//!   reads as Wombgifts (J14, J20: "I want growing wombgift to behave like
//!   other wombgift items, however they are handled.").
//!
//! # As built
//!
//! - **Two tables, shipped in the binary**, each generated by
//!   `tools/category-table.py` from committed sources and checked by its
//!   `--check`, each with its version on every basis (C98; clause (c)):
//!   `reference/categories-v1.toml`, the ids with what makes each hold,
//!   and `reference/beast-colours-v1.toml`, a beast's name to the colour
//!   the site lists it under. The files' headers and the script's
//!   docstring carry the rules a reviewer reads. The loader
//!   ([`Tables::parse`]) checks what a reviewer would — the versions, the
//!   realms being the class table's, every class named being a class of
//!   the class table and under one id alone, every member a leaf, one
//!   beast id, two colour leaves — and a table that fails is a build
//!   defect surfaced as [`CategoryTableError`] to every caller, never a
//!   partial table.
//! - **An id brings others** ([`Tables::brings`]): an item that has an id
//!   has every id over it by the dot, every grouping it is a member of,
//!   and the id its row names `under`. What an item has of its own is its
//!   class's leaf, or `monster.beast` and its colour leaf; what holds is
//!   what those bring. One id is *beneath* another where it brings it,
//!   which is all "most specific" means.
//! - **The question, once** ([`Tables::of`], [`Category`]): each way the
//!   item may be read — one for an established class, one for each
//!   candidate where the class is open, the beast's beside them where the
//!   bestiary line could not be read — gives the ids that hold under it;
//!   an id holds where every reading gives it, fails where none does, and
//!   is undecided otherwise. Unknown is said at that grain (rule 8 of the
//!   plan): an unknown colour leaves the two colour leaves open and
//!   `monster.beast` true.
//! - **The reasons are the class's reading's or the colour's** (`unread`,
//!   below): an id left open by how the item is read is explained by what
//!   that reading rests on and could not read (`class::rests_on`: the
//!   table's gap, or the deriver's unread base, frame or bestiary line),
//!   never by a second reason made here — a beast whose name is unread
//!   has its colour open by that unread name, with the deriver's hint,
//!   and a stack of orbs whose bestiary line is unread has `currency`
//!   open by that line while its class, which both readings give, is
//!   established. The one reason made here is the colour, [`Part::Colour`],
//!   with its own hint: the colour table asked, and giving none.
//! - **`has:category`, `-has:category`, `undecided(category)`** are the
//!   three buckets of a count (C105; `eval.rs`): an id holds; no id holds
//!   and none is open; no id holds and one is open. An item listed under
//!   an id with another id open — the frameless breachstone under `map` —
//!   has a category, and `show` says which ids stay open and why.
//! - **A bucket's term selects what the bucket lists** (`Tables::listed_under`;
//!   invariant 4 of the surface): a leaf's is `category=<id>`; an id with
//!   ids beneath it is listed only where none of those is known to hold,
//!   which its term says — `category=map (-category~"…" or
//!   undecided(category~"…"))`, the pattern picking exactly the ids
//!   beneath.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::class::{self, Captured, Classed};
use crate::corpus::REALMS;
use crate::derive::{Item, Part, Unread};
use crate::tree::{Node, Op, Probe, Value};

/// The category table this build ships, verbatim (the reviewed file).
pub const CATEGORY_TABLE_TOML: &str = include_str!("../reference/categories-v1.toml");

/// The category table version this build expects and every basis cites.
pub const CATEGORY_TABLE_VERSION: u32 = 1;

/// The beast colour table this build ships, verbatim.
pub const COLOUR_TABLE_TOML: &str = include_str!("../reference/beast-colours-v1.toml");

/// The beast colour table version this build expects and every basis
/// cites.
pub const COLOUR_TABLE_VERSION: u32 = 1;

/// The hint on the colour's reason (J1): the table, not the store, is what
/// could change, and the site may list the beast (C93: a hint, never a
/// guarantee).
pub const COLOUR_HINT: &str = "a refresh will not help; the trade site may list the beast under a colour, and a reference update would then say so";

/// What the colour's reason is called, where a reason and a tally name it
/// (`eval::reason`).
pub const COLOUR_UNREAD: &str = "the colour: the colour table gives the beast none";

/// The field's name, and the name a row's evidence carries.
pub(crate) const FIELD: &str = "category";

// ---- the files ----------------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    version: u32,
    realms: Vec<String>,
    source: String,
    filter: String,
    classes: u32,
    #[allow(dead_code)]
    generated_by: String,
    #[serde(rename = "id", default)]
    ids: Vec<Row>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Row {
    id: String,
    text: String,
    kind: String,
    #[serde(default)]
    classes: Vec<String>,
    rule: Option<RuleRow>,
    #[serde(default)]
    members: Vec<MemberRow>,
    colour: Option<String>,
    under: Option<String>,
    cites: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleRow {
    class: String,
    starts: Option<String>,
    ends: Option<String>,
    contains: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberRow {
    leaf: String,
    #[allow(dead_code)]
    said_by: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColourFile {
    version: u32,
    realms: Vec<String>,
    source: String,
    #[allow(dead_code)]
    generated_by: String,
    #[serde(rename = "beast", default)]
    beasts: Vec<BeastRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BeastRow {
    name: String,
    colour: String,
    evidence: String,
}

// ---- the tables ---------------------------------------------------------------------------------

/// A set of the table's ids, by their place in the site's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ids(u128);

impl Ids {
    const NONE: Ids = Ids(0);

    fn one(at: usize) -> Ids {
        Ids(1 << at)
    }

    fn has(self, at: usize) -> bool {
        self.0 & (1 << at) != 0
    }

    fn with(self, other: Ids) -> Ids {
        Ids(self.0 | other.0)
    }

    fn and(self, other: Ids) -> Ids {
        Ids(self.0 & other.0)
    }

    fn without(self, other: Ids) -> Ids {
        Ids(self.0 & !other.0)
    }

    fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn meets(self, other: Ids) -> bool {
        self.0 & other.0 != 0
    }
}

/// What kind of id a row is (the module doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Parent,
    Grouping,
    Leaf,
    Beast,
    Colour(Colour),
}

/// A colour the site has a leaf for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Colour {
    Yellow,
    Red,
}

impl Colour {
    fn word(self) -> &'static str {
        match self {
            Colour::Yellow => "yellow",
            Colour::Red => "red",
        }
    }
}

/// How a name rule tests the base.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Test {
    Starts(String),
    Ends(String),
    Contains(String),
}

impl Test {
    fn holds(&self, base: &str) -> bool {
        match self {
            Test::Starts(text) => base.starts_with(text.as_str()),
            Test::Ends(text) => base.ends_with(text.as_str()),
            Test::Contains(text) => base.contains(text.as_str()),
        }
    }

    fn said(&self) -> String {
        match self {
            Test::Starts(text) => format!("starts with `{text}`"),
            Test::Ends(text) => format!("ends with `{text}`"),
            Test::Contains(text) => format!("holds `{text}`"),
        }
    }
}

#[derive(Debug)]
struct Id {
    id: &'static str,
    text: String,
    kind: Kind,
    classes: Vec<String>,
    rule: Option<(String, Test)>,
    members: Vec<usize>,
    cites: String,
}

/// The two tables, parsed and checked. Read-only; one per build.
#[derive(Debug)]
pub struct Tables {
    version: u32,
    realms: Vec<String>,
    source: String,
    /// The ids in the site's order.
    ids: Vec<Id>,
    /// The ids alone, in that order: the field's legal values.
    names: &'static [&'static str],
    /// An id to its place in the site's order.
    at_of: HashMap<String, usize>,
    /// A class to the id an item of it has.
    of_class: HashMap<String, usize>,
    /// The name rules, in the table's order: the class, the test, the id.
    rules: Vec<(String, Test, usize)>,
    /// Each id with every id an item that has it has too, itself among
    /// them.
    brings: Vec<Ids>,
    /// Each id's ids beneath it: those that bring it.
    beneath: Vec<Ids>,
    beast: usize,
    yellow: usize,
    red: usize,
    colours_version: u32,
    colours_source: String,
    /// A beast's name to the colour the site lists it under; `None` where
    /// the table marks it unknown.
    colour_of: HashMap<String, Option<Colour>>,
}

/// Why a table text is not a table. Every arm names the offending row so
/// a reviewer can fix the file, not the loader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CategoryTableError {
    Parse(String),
    Version {
        table: &'static str,
        found: u32,
        expected: u32,
    },
    /// The class table this table is keyed by does not load.
    ClassTable(String),
    /// A row the loader refuses, and why.
    Row(String),
}

impl std::fmt::Display for CategoryTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CategoryTableError::Parse(e) => write!(f, "the category tables do not parse: {e}"),
            CategoryTableError::Version {
                table,
                found,
                expected,
            } => write!(
                f,
                "the {table} table is version {found}; this build reads version {expected}"
            ),
            CategoryTableError::ClassTable(e) => write!(
                f,
                "the category table is keyed by the class table, which does not load: {e}"
            ),
            CategoryTableError::Row(e) => write!(f, "the category tables: {e}"),
        }
    }
}

impl std::error::Error for CategoryTableError {}

static TABLES: LazyLock<Result<Tables, CategoryTableError>> =
    LazyLock::new(|| Tables::parse(CATEGORY_TABLE_TOML, COLOUR_TABLE_TOML));

/// The shipped tables, parsed and checked once.
pub fn tables() -> Result<&'static Tables, &'static CategoryTableError> {
    TABLES.as_ref()
}

/// The legal values of `category`, in the site's order; empty when the
/// tables do not load.
pub fn ids() -> &'static [&'static str] {
    match tables() {
        Ok(tables) => tables.names,
        Err(_) => &[],
    }
}

fn row_error<T>(message: String) -> Result<T, CategoryTableError> {
    Err(CategoryTableError::Row(message))
}

impl Tables {
    /// Parse and check the two table texts against the shipped class
    /// table. The shipped files go through [`tables`]; this is the same
    /// path over any texts, for the tests that pin what the loader
    /// refuses.
    pub fn parse(categories: &str, colours: &str) -> Result<Tables, CategoryTableError> {
        let file: File =
            toml::from_str(categories).map_err(|e| CategoryTableError::Parse(e.to_string()))?;
        let colour_file: ColourFile =
            toml::from_str(colours).map_err(|e| CategoryTableError::Parse(e.to_string()))?;
        if file.version != CATEGORY_TABLE_VERSION {
            return Err(CategoryTableError::Version {
                table: "category",
                found: file.version,
                expected: CATEGORY_TABLE_VERSION,
            });
        }
        if colour_file.version != COLOUR_TABLE_VERSION {
            return Err(CategoryTableError::Version {
                table: "beast colour",
                found: colour_file.version,
                expected: COLOUR_TABLE_VERSION,
            });
        }
        let classes = class::table().map_err(|e| CategoryTableError::ClassTable(e.to_string()))?;
        if file.classes != classes.version() {
            return row_error(format!(
                "generated against classes v{}, and this build ships classes v{}",
                file.classes,
                classes.version()
            ));
        }
        // a category is computed from a class: one set of realms
        for (table, realms) in [
            ("category", &file.realms),
            ("beast colour", &colour_file.realms),
        ] {
            if let Some(realm) = realms.iter().find(|r| !REALMS.contains(&r.as_str())) {
                return row_error(format!(
                    "the {table} table names a realm GGG has not: {realm:?}"
                ));
            }
            if realms.as_slice() != classes.realms() {
                return row_error(format!(
                    "the {table} table covers {} and the class table {}",
                    realms.join(", "),
                    classes.realms().join(", ")
                ));
            }
        }
        if file.ids.is_empty() {
            return row_error("the category table has no id".to_string());
        }
        if file.ids.len() > 128 {
            return row_error(format!(
                "{} ids, and a set of them holds 128",
                file.ids.len()
            ));
        }
        let mut at_of: HashMap<String, usize> = HashMap::new();
        for (at, row) in file.ids.iter().enumerate() {
            if row.id.trim().is_empty() || row.text.trim().is_empty() || row.cites.trim().is_empty()
            {
                return row_error(format!(
                    "an id, a text or a citation is empty at {:?}",
                    row.id
                ));
            }
            if at_of.insert(row.id.clone(), at).is_some() {
                return row_error(format!("the id {:?} is named twice", row.id));
            }
        }
        let at = |id: &str, of: &str| -> Result<usize, CategoryTableError> {
            match at_of.get(id) {
                Some(at) => Ok(*at),
                None => row_error(format!("{of} names {id:?}, which is no id")),
            }
        };
        let mut ids: Vec<Id> = Vec::with_capacity(file.ids.len());
        let mut of_class: HashMap<String, usize> = HashMap::new();
        let mut rules: Vec<(String, Test, usize)> = Vec::new();
        let mut under: Vec<Option<usize>> = Vec::with_capacity(file.ids.len());
        for (here, row) in file.ids.iter().enumerate() {
            let kind = match (row.kind.as_str(), row.colour.as_deref()) {
                ("parent", None) => Kind::Parent,
                ("grouping", None) => Kind::Grouping,
                ("leaf", None) => Kind::Leaf,
                ("beast", None) => Kind::Beast,
                ("colour", Some("yellow")) => Kind::Colour(Colour::Yellow),
                ("colour", Some("red")) => Kind::Colour(Colour::Red),
                _ => {
                    return row_error(format!(
                        "{:?} is of no kind this build reads: {:?}, colour {:?}",
                        row.id, row.kind, row.colour
                    ));
                }
            };
            let keyed = matches!(kind, Kind::Parent | Kind::Leaf);
            if !keyed && (!row.classes.is_empty() || row.rule.is_some()) {
                return row_error(format!("{:?} is no class's id and names a class", row.id));
            }
            if (kind == Kind::Grouping) == row.members.is_empty() {
                return row_error(format!(
                    "{:?}: a grouping has members, and nothing else has",
                    row.id
                ));
            }
            for name in &row.classes {
                if !classes.names().contains(name) {
                    return row_error(format!(
                        "{:?} names {name:?}, which is no class of the class table",
                        row.id
                    ));
                }
                // one class, one id: two ids keyed by one class would make
                // the category a second classifier
                if of_class.insert(name.clone(), here).is_some() {
                    return row_error(format!("the class {name:?} is under two ids"));
                }
            }
            let rule = match &row.rule {
                None => None,
                Some(rule) => {
                    let test = match (&rule.starts, &rule.ends, &rule.contains) {
                        (Some(text), None, None) if !text.is_empty() => Test::Starts(text.clone()),
                        (None, Some(text), None) if !text.is_empty() => Test::Ends(text.clone()),
                        (None, None, Some(text)) if !text.is_empty() => {
                            Test::Contains(text.clone())
                        }
                        _ => {
                            return row_error(format!(
                                "{:?}: a name rule is one test of the base",
                                row.id
                            ));
                        }
                    };
                    if !classes.names().contains(&rule.class) {
                        return row_error(format!(
                            "{:?}: its rule is on {:?}, which is no class of the class table",
                            row.id, rule.class
                        ));
                    }
                    rules.push((rule.class.clone(), test.clone(), here));
                    Some((rule.class.clone(), test))
                }
            };
            let members = row
                .members
                .iter()
                .map(|member| at(&member.leaf, &row.id))
                .collect::<Result<Vec<usize>, _>>()?;
            under.push(match &row.under {
                Some(over) => Some(at(over, &row.id)?),
                None => None,
            });
            ids.push(Id {
                id: Box::leak(row.id.clone().into_boxed_str()),
                text: row.text.clone(),
                kind,
                classes: row.classes.clone(),
                rule,
                members,
                cites: row.cites.clone(),
            });
        }
        for (class, _, leaf) in &rules {
            if !of_class.contains_key(class) {
                return row_error(format!(
                    "the rule of {:?} is on {class:?}, a class with no id of its own",
                    ids[*leaf].id
                ));
            }
        }
        for id in &ids {
            if let Some(member) = id.members.iter().find(|m| ids[**m].kind != Kind::Leaf) {
                return row_error(format!(
                    "{:?} has the member {:?}, which is no leaf",
                    id.id, ids[*member].id
                ));
            }
        }
        let sole = |kind: Kind, what: &str| -> Result<usize, CategoryTableError> {
            let mut found = ids.iter().enumerate().filter(|(_, id)| id.kind == kind);
            match (found.next(), found.next()) {
                (Some((at, _)), None) => Ok(at),
                _ => row_error(format!("the category table has not exactly one {what}")),
            }
        };
        let beast = sole(Kind::Beast, "beast id")?;
        let yellow = sole(Kind::Colour(Colour::Yellow), "yellow leaf")?;
        let red = sole(Kind::Colour(Colour::Red), "red leaf")?;
        if under[yellow] != Some(beast) || under[red] != Some(beast) {
            return row_error("a colour leaf is under the beast id".to_string());
        }

        // what an id brings: itself, every id over it by the dot, the id
        // its row names, and every grouping it is a member of — and what
        // each of those brings
        let mut brings: Vec<Ids> = (0..ids.len()).map(Ids::one).collect();
        for (here, id) in ids.iter().enumerate() {
            let mut over = id.id;
            while let Some((head, _)) = over.rsplit_once('.') {
                if let Some(parent) = at_of.get(head) {
                    brings[here] = brings[here].with(Ids::one(*parent));
                }
                over = head;
            }
            if let Some(over) = under[here] {
                brings[here] = brings[here].with(Ids::one(over));
            }
        }
        for (grouping, id) in ids.iter().enumerate() {
            for member in &id.members {
                brings[*member] = brings[*member].with(Ids::one(grouping));
            }
        }
        // closed under itself: a grouping's parent by the dot, a parent's
        loop {
            let before = brings.clone();
            for here in &mut brings {
                *here = (0..before.len())
                    .filter(|other| here.has(*other))
                    .fold(*here, |all, other| all.with(before[other]));
            }
            if brings == before {
                break;
            }
        }
        let beneath: Vec<Ids> = (0..ids.len())
            .map(|here| {
                (0..ids.len())
                    .filter(|other| *other != here && brings[*other].has(here))
                    .fold(Ids::NONE, |all, other| all.with(Ids::one(other)))
            })
            .collect();

        let mut colour_of: HashMap<String, Option<Colour>> = HashMap::new();
        for row in &colour_file.beasts {
            let colour = match row.colour.as_str() {
                "yellow" => Some(Colour::Yellow),
                "red" => Some(Colour::Red),
                "unknown" => None,
                other => {
                    return row_error(format!(
                        "the beast {:?} has the colour {other:?}, which the site has no leaf for",
                        row.name
                    ));
                }
            };
            if row.name.trim().is_empty() || row.evidence.trim().is_empty() {
                return row_error(format!(
                    "a beast with no name or no search that evidences it: {:?}",
                    row.name
                ));
            }
            if colour_of.insert(row.name.clone(), colour).is_some() {
                return row_error(format!("the beast {:?} is named twice", row.name));
            }
        }
        if colour_of.is_empty() {
            return row_error("the beast colour table has no beast".to_string());
        }
        let names: Vec<&'static str> = ids.iter().map(|id| id.id).collect();
        Ok(Tables {
            version: file.version,
            realms: file.realms,
            source: format!("{} — the filter of {}", file.source, file.filter),
            names: Box::leak(names.into_boxed_slice()),
            ids,
            at_of,
            of_class,
            rules,
            brings,
            beneath,
            beast,
            yellow,
            red,
            colours_version: colour_file.version,
            colours_source: colour_file.source,
            colour_of,
        })
    }

    /// The category table's version, which every basis cites.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The beast colour table's version, which every basis cites.
    pub fn colours_version(&self) -> u32 {
        self.colours_version
    }

    /// The realms the tables cover: the class table's.
    pub fn realms(&self) -> &[String] {
        &self.realms
    }

    /// How many beasts the colour table names.
    pub fn beasts(&self) -> usize {
        self.colour_of.len()
    }

    /// The ids, in the site's order.
    pub fn ids(&self) -> &'static [&'static str] {
        self.names
    }

    fn at(&self, id: &str) -> Option<usize> {
        self.at_of.get(id).copied()
    }

    fn names_of(&self, ids: Ids) -> Vec<&'static str> {
        (0..self.names.len())
            .filter(|at| ids.has(*at))
            .map(|at| self.names[at])
            .collect()
    }

    fn all(&self) -> Ids {
        (0..self.names.len()).fold(Ids::NONE, |all, at| all.with(Ids::one(at)))
    }

    /// The three beast ids: what only a bestiary line gives.
    fn beast_ids(&self) -> Ids {
        Ids::one(self.beast)
            .with(Ids::one(self.yellow))
            .with(Ids::one(self.red))
    }

    /// The legal values a term picked, as a set.
    pub(crate) fn picked(&self, values: &[&'static str]) -> Ids {
        values
            .iter()
            .filter_map(|value| self.at(value))
            .fold(Ids::NONE, |all, at| all.with(Ids::one(at)))
    }

    /// One id as a set, none where the table has no such id: what a
    /// total's row asks an item's category by (`totals.rs`, C94 — a row
    /// that reads what the item is), the question being [`Category::asked`]
    /// as a term's is.
    pub(crate) fn one(&self, id: &str) -> Option<Ids> {
        self.at(id).map(Ids::one)
    }

    /// Of the values a term picked, those that hold on the item: what its
    /// selector resolved to there.
    pub(crate) fn holding(&self, category: &Category, picked: Ids) -> Vec<&'static str> {
        self.names_of(category.yes.and(picked))
    }

    /// The ids an item that has `id` has too, `id` among them: what
    /// [`Tables::of`] closes an item's own ids under (the module doc).
    pub fn brings(&self, id: &str) -> Vec<&'static str> {
        self.at(id)
            .map(|at| self.names_of(self.brings[at]))
            .unwrap_or_default()
    }

    /// The id an item of this class and base has of its own: a name
    /// rule's where one holds the base, the class's otherwise, none where
    /// the class gives none.
    fn leaf_of(&self, class: &str, base: Option<&str>) -> Option<usize> {
        self.rules
            .iter()
            .find(|(of, test, _)| of == class && base.is_some_and(|base| test.holds(base)))
            .map(|(_, _, leaf)| *leaf)
            .or_else(|| self.of_class.get(class).copied())
    }

    /// What holds on an item that is each of these classes.
    fn of_classes<'a>(
        &self,
        classes: impl IntoIterator<Item = &'a str>,
        base: Option<&str>,
    ) -> Ids {
        classes
            .into_iter()
            .filter_map(|class| self.leaf_of(class, base))
            .fold(Ids::NONE, |all, leaf| all.with(self.brings[leaf]))
    }

    /// What holds on a captured beast of this name, what its colour
    /// leaves open, and the colour's reason where it has none (J1).
    fn of_beast(&self, item: &Item) -> (Ids, Ids, ColourWhy) {
        let colours = Ids::one(self.yellow).with(Ids::one(self.red));
        let open = |problem: String| {
            (
                self.brings[self.beast],
                colours,
                ColourWhy::Table(Unread {
                    part: Part::Colour,
                    problem,
                    line: None,
                    name: None,
                    socket: None,
                }),
            )
        };
        let Some(base) = item.base.as_deref() else {
            // the name could not be read: the deriver's unread on it is
            // the reason, said once, where it was read (rules 8 and 10 of
            // the plan) — never the colour table's, which was not asked
            if item
                .unread_in(&Part::Field("baseType".to_string()))
                .next()
                .is_some()
            {
                return (self.brings[self.beast], colours, ColourWhy::Name);
            }
            return open(format!(
                "the beast has no name to look up in the beast colour table (beast colours v{})",
                self.colours_version
            ));
        };
        match self.colour_of.get(base) {
            Some(Some(Colour::Yellow)) => (self.brings[self.yellow], Ids::NONE, ColourWhy::Known),
            Some(Some(Colour::Red)) => (self.brings[self.red], Ids::NONE, ColourWhy::Known),
            Some(None) => open(format!(
                "the beast colour table marks `{base}` unknown: the site listed it under neither colour (beast colours v{})",
                self.colours_version
            )),
            None => open(format!(
                "`{base}` is not in the beast colour table (beast colours v{})",
                self.colours_version
            )),
        }
    }

    /// The category of one derived item, given what the class table says
    /// of it (the module doc: the question, once).
    pub fn of(&self, item: &Item, class: &Classed) -> Category {
        // J22: in a realm with no table every id is undecided
        if !self.realms.contains(&item.facts.realm) {
            return self.category(Ids::NONE, self.all(), Ids::NONE, None);
        }
        let base = item.base.as_deref();
        let captured = class::captured(item);
        // each way the item may be read, as the ids that hold under it;
        // none where the class says nothing of what it could be
        let as_classed = |class: &Classed| -> Option<Vec<Ids>> {
            match class {
                Classed::Is(name) => Some(vec![self.of_classes([name.as_str()], base)]),
                Classed::All(names) => Some(vec![
                    self.of_classes(names.iter().map(String::as_str), base),
                ]),
                Classed::Among { candidates, .. } | Classed::FrameUnread { candidates } => Some(
                    candidates
                        .iter()
                        .map(|name| self.of_classes([name.as_str()], base))
                        .collect(),
                ),
                Classed::Open(_) | Classed::BaseUnread | Classed::BestiaryUnread { .. } => None,
            }
        };
        let (readings, colour_open, colour) = match (captured, class) {
            // J10: a beast's class gives it no id
            (Captured::Yes, _) => {
                let (yes, open, why) = self.of_beast(item);
                (Some(vec![yes]), open, why)
            }
            (Captured::No, class) => (as_classed(class), Ids::NONE, ColourWhy::Known),
            // a beast, or what the base says otherwise — which is its
            // class outright where both readings give one class
            (Captured::Unread, class) => {
                let otherwise = match class {
                    Classed::BestiaryUnread { otherwise, .. } => otherwise.as_ref(),
                    other => other,
                };
                let (yes, open, why) = self.of_beast(item);
                (
                    as_classed(otherwise).map(|mut readings| {
                        readings.push(yes);
                        readings
                    }),
                    open,
                    why,
                )
            }
        };
        let Some(readings) = readings else {
            // the class says nothing of what the item could be: every id a
            // class gives is open, and the beast ids with them only where
            // the item may be a beast
            let open = match captured {
                Captured::No => self.all().without(self.beast_ids()),
                Captured::Yes | Captured::Unread => self.all(),
            };
            return self.category(Ids::NONE, open, Ids::NONE, None);
        };
        // an id holds where every reading gives it (J17), fails where none
        // does, and is open by the class where the readings disagree
        let every = readings
            .iter()
            .fold(self.all(), |every, reading| every.and(*reading));
        let any = readings
            .iter()
            .fold(Ids::NONE, |any, reading| any.with(*reading));
        // a colour leaf is open by the colour where the item is a beast
        // under some reading — and by the class too where it is one under
        // one reading and not under another
        let by_name = matches!(colour, ColourWhy::Name);
        let (by_table, colour) = match colour {
            ColourWhy::Table(why) => (colour_open, Some(why)),
            ColourWhy::Known | ColourWhy::Name => (Ids::NONE, None),
        };
        let by_colour = by_table.without(every);
        // the readings disagree on a colour leaf where the item is a
        // beast under one and not under another; and a beast whose name
        // is unread has its colour open by that unread, which is among
        // what the class's reading rests on (`class::rests_on`)
        let beside = if readings.len() > 1 || by_name {
            colour_open
        } else {
            Ids::NONE
        };
        let by_class = any.with(beside).without(every);
        self.category(every, by_class, by_colour, colour)
    }

    fn category(
        &self,
        yes: Ids,
        open_by_class: Ids,
        open_by_colour: Ids,
        colour: Option<Unread>,
    ) -> Category {
        // the most specific ids that hold: none beneath them holds
        let listed = (0..self.names.len())
            .filter(|at| yes.has(*at) && !self.beneath[*at].meets(yes))
            .map(|at| self.names[at])
            .collect();
        Category {
            yes,
            open_by_class,
            open_by_colour,
            colour: colour.filter(|_| !open_by_colour.is_empty()),
            listed,
        }
    }

    /// The terms that select what a count lists under `id` (the module
    /// doc): the id, and — where ids sit beneath it — none of those known
    /// to hold.
    pub(crate) fn listed_under(&self, id: &str) -> Vec<Node> {
        let test = |op: Op, value: String| Node::Test {
            field: FIELD.to_string(),
            op,
            value: Value::Text(value),
        };
        let holds = test(Op::Eq, id.to_string());
        let beneath = self
            .at(id)
            .map(|at| self.names_of(self.beneath[at]))
            .unwrap_or_default();
        if beneath.is_empty() {
            return vec![holds];
        }
        let pattern = format!(
            "^({})$",
            beneath
                .iter()
                .map(|id| id.replace('.', "[.]"))
                .collect::<Vec<String>>()
                .join("|")
        );
        let any_beneath = test(Op::Match, pattern);
        vec![
            holds,
            Node::Any(vec![
                Node::Not(Box::new(any_beneath.clone())),
                Node::Undecided(Probe::Term(Box::new(any_beneath))),
            ]),
        ]
    }

    /// What `--describe category` says of the field: what it is, and the
    /// tables' sources and versions (C106's clause (c)).
    pub fn definition(&self) -> String {
        format!(
            "the trade site's category of the item, by the site's own id — a convention (C106), computed from the item's class and reviewed rules: categories v{}, {} ids over {}: {}. A captured beast is known by its bestiary line and its colour is the beast colours v{} table's, {} names: {}. An id with a dot is quoted, category=\"armour.boots\"; a parent holds everything under it by the dot. An item whose class gives no id lacks a category; one whose class or colour could not be established is undecided on the ids that needed it, with that reason (C93). A count lists an item under the most specific id that holds",
            self.version,
            self.names.len(),
            self.realms.join(", "),
            self.source,
            self.colours_version,
            self.colour_of.len(),
            self.colours_source
        )
    }

    /// Every id with the site's text for it and what makes it hold, in the
    /// site's order.
    pub fn described(&self) -> Vec<Described> {
        let quoted = |ids: Vec<&str>| -> String {
            ids.iter()
                .map(|id| format!("`{id}`"))
                .collect::<Vec<String>>()
                .join(", ")
        };
        self.ids
            .iter()
            .enumerate()
            .map(|(here, id)| {
                // what the classes give, and beside it what they do not
                let classes = || -> (String, String) {
                    let given = format!("an item whose class is {}", id.classes.join(" or "));
                    let mut said = String::new();
                    // what a name rule takes out of the class's own id —
                    // said only where it leaves this id: a Fossil's leaf
                    // is under `currency`, which holds it still (J2)
                    for (class, test, leaf) in &self.rules {
                        if id.classes.contains(class) && !self.brings[*leaf].has(here) {
                            said.push_str(&format!(
                                " — but a {class} base that {} is `{}` instead",
                                test.said(),
                                self.ids[*leaf].id
                            ));
                        }
                    }
                    // J10: a beast's class gives it no id
                    if id.classes.iter().any(|c| c == class::BEAST_CLASS) {
                        said.push_str(&format!(
                            " — but never a captured beast, whose class is {} too: it is `{}` (J10)",
                            class::BEAST_CLASS,
                            self.ids[self.beast].id
                        ));
                    }
                    (given, said)
                };
                let holds = match id.kind {
                    Kind::Parent => {
                        let under = quoted(self.names_of(self.beneath[here]));
                        if id.classes.is_empty() {
                            format!("an item that has an id under it by the dot: {under}")
                        } else {
                            let (given, but) = classes();
                            format!("{given}, or that has an id under it by the dot: {under}{but}")
                        }
                    }
                    Kind::Grouping => format!(
                        "an item that has any of its reviewed members, the only leaves it holds: {}",
                        quoted(id.members.iter().map(|m| self.ids[*m].id).collect())
                    ),
                    Kind::Leaf => match (&id.rule, id.classes.is_empty()) {
                        (Some((class, test)), _) => {
                            format!("an item whose class is {class} and whose base {}", test.said())
                        }
                        (None, false) => {
                            let (given, but) = classes();
                            format!("{given}{but}")
                        }
                        (None, true) => format!("no item this build can class — {}", id.cites),
                    },
                    Kind::Beast => format!(
                        "a captured beast, known by its bestiary line (`{}`), whatever its colour; never by its class",
                        class::BESTIARY_LINE
                    ),
                    Kind::Colour(colour) => format!(
                        "a captured beast whose name the beast colour table lists as {}; a beast the table lacks or marks unknown is undecided, by the colour, and one whose name could not be read, by that",
                        colour.word()
                    ),
                };
                Described {
                    id: id.id,
                    text: id.text.clone(),
                    holds,
                }
            })
            .collect()
    }
}

/// Why a beast's colour leaves are open, where they are.
enum ColourWhy {
    /// The colour table gives the name a colour: nothing is open.
    Known,
    /// The table was asked and gave none: its own reason (J1).
    Table(Unread),
    /// The name could not be read: the deriver's unread on it.
    Name,
}

/// One id as `--describe category` prints it.
#[derive(Debug, Clone, Serialize)]
pub struct Described {
    pub id: &'static str,
    /// The site's text for the id.
    pub text: String,
    /// What makes it hold.
    pub holds: String,
}

// ---- an item's category -------------------------------------------------------------------------

/// What the ids answer of one item (the module doc).
#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    yes: Ids,
    /// The ids the class — or the bestiary line — leaves open.
    open_by_class: Ids,
    /// The colour leaves a beast's colour leaves open.
    open_by_colour: Ids,
    /// The colour's reason, where a colour leaf is open by it.
    colour: Option<Unread>,
    /// The ids a count lists the item under: the most specific that hold,
    /// in the site's order.
    listed: Vec<&'static str>,
}

/// What a set of ids answers of an item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Holds {
    /// One of them holds.
    Yes,
    /// None holds, and one is open.
    Open,
    /// None holds and none is open.
    No,
}

impl Category {
    /// A category that says nothing: what a test builds an item with
    /// where no table is asked.
    #[cfg(test)]
    pub(crate) fn none() -> Category {
        Category {
            yes: Ids::NONE,
            open_by_class: Ids::NONE,
            open_by_colour: Ids::NONE,
            colour: None,
            listed: Vec::new(),
        }
    }

    fn open(&self) -> Ids {
        self.open_by_class.with(self.open_by_colour)
    }

    /// The one question: do these ids hold on the item.
    pub(crate) fn asked(&self, picked: Ids) -> Holds {
        if self.yes.meets(picked) {
            Holds::Yes
        } else if self.open().meets(picked) {
            Holds::Open
        } else {
            Holds::No
        }
    }

    /// The ids a count lists the item under; empty where none holds.
    pub fn listed(&self) -> &[&'static str] {
        &self.listed
    }

    /// Whether no id holds and one is open: the category is not
    /// established, `undecided(category)`.
    pub fn undecided(&self) -> bool {
        self.yes.is_empty() && !self.open().is_empty()
    }

    /// The bucket a count puts the item in: `value`, `none` or
    /// `undecided` (C105).
    pub fn bucket(&self) -> &'static str {
        if !self.yes.is_empty() {
            "value"
        } else if self.open().is_empty() {
            "none"
        } else {
            "undecided"
        }
    }
}

/// What left these ids open on the item — every id, where none is named:
/// what the class's reading rests on that could not be read, where that
/// leaves one of them open (`class::rests_on`: the table's gap, the
/// deriver's unread base, frame or bestiary line — a beast's unread name
/// among them, which leaves its colour open), and the colour table's own
/// reason where the table was asked and gave none. One maker
/// (rule 10 of the plan): a term's reasons, a count's tally and `show`
/// read this.
pub(crate) fn unread<'a>(
    item: &'a Item,
    class: &'a Classed,
    category: &'a Category,
    picked: Option<Ids>,
) -> Vec<&'a Unread> {
    let asked = |open: Ids| picked.map_or(!open.is_empty(), |picked| open.meets(picked));
    let mut reasons: Vec<&Unread> = Vec::new();
    if asked(category.open_by_class) {
        reasons.extend(class::rests_on(item, class));
    }
    if asked(category.open_by_colour) {
        reasons.extend(category.colour.as_ref());
    }
    reasons
}

/// An item's category as `show` gives it: the id or ids a count lists it
/// under, and the reason wherever an id is left open.
#[derive(Debug, Clone, Serialize)]
pub struct Shown {
    /// `value`, `none` or `undecided`: the bucket a count puts the item in.
    pub bucket: &'static str,
    pub listed: Vec<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub open: Vec<Open>,
}

/// The ids one reason leaves open.
#[derive(Debug, Clone, Serialize)]
pub struct Open {
    pub ids: Vec<&'static str>,
    #[serde(flatten)]
    pub why: Unread,
}

impl Tables {
    /// What `show` says of an item's category.
    pub fn shown(&self, item: &Item, class: &Classed, category: &Category) -> Shown {
        let mut open: Vec<Open> = Vec::new();
        if !category.open_by_class.is_empty() {
            open.extend(class::rests_on(item, class).map(|why| Open {
                ids: self.names_of(category.open_by_class),
                why: why.clone(),
            }));
        }
        if let Some(why) = &category.colour {
            open.push(Open {
                ids: self.names_of(category.open_by_colour),
                why: why.clone(),
            });
        }
        Shown {
            bucket: category.bucket(),
            listed: category.listed.clone(),
            open,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CATEGORY_TABLE_TOML, CATEGORY_TABLE_VERSION, COLOUR_TABLE_TOML, COLOUR_TABLE_VERSION,
        CategoryTableError, Tables, tables,
    };

    /// The shipped files parse under the loader's checks and are the
    /// versions this build cites: 82 ids, 361 beasts.
    #[test]
    fn the_shipped_tables_load_and_are_the_versions_cited() {
        let t = tables().unwrap();
        assert_eq!(t.version(), CATEGORY_TABLE_VERSION);
        assert_eq!(t.colours_version(), COLOUR_TABLE_VERSION);
        assert_eq!(t.realms(), ["pc", "xbox", "sony"]);
        assert_eq!(t.ids().len(), 82);
        assert_eq!(t.beasts(), 361);
        assert_eq!(t.ids()[0], "weapon");
    }

    /// What an id brings (J2, J4, J12, J15): the ids over it by the dot,
    /// the groupings it is a reviewed member of, the id its row names.
    #[test]
    fn an_id_brings_its_parents_its_groupings_and_what_its_row_names() {
        let t = tables().unwrap();
        assert_eq!(
            t.brings("weapon.sceptre"),
            [
                "weapon",
                "weapon.one",
                "weapon.onemelee",
                "weapon.onemace",
                "weapon.sceptre"
            ]
        );
        // J4: Fishing Rods outside two-handed melee; J12: nothing places a bow
        assert_eq!(t.brings("weapon.rod"), ["weapon", "weapon.rod"]);
        assert_eq!(t.brings("weapon.bow"), ["weapon", "weapon.bow"]);
        assert_eq!(t.brings("map.scarab"), ["map", "map.scarab"]);
        assert_eq!(
            t.brings("monster.redbeast"),
            ["monster.beast", "monster.redbeast"]
        );
        // dotted, the top token no id
        assert_eq!(t.brings("sanctum.relic"), ["sanctum.relic"]);
        assert_eq!(t.brings("flask"), ["flask"]);
    }

    /// What the loader refuses, each by what it names.
    #[test]
    fn the_loader_refuses_what_a_reviewer_would() {
        let refused = |categories: &str, colours: &str| match Tables::parse(categories, colours) {
            Err(CategoryTableError::Row(said)) => said,
            other => panic!("not refused by a row: {other:?}"),
        };
        let ok = CATEGORY_TABLE_TOML;
        assert!(matches!(
            Tables::parse(
                &ok.replacen("version = 1", "version = 2", 1),
                COLOUR_TABLE_TOML
            ),
            Err(CategoryTableError::Version {
                table: "category",
                found: 2,
                expected: 1
            })
        ));
        assert!(matches!(
            Tables::parse(
                ok,
                &COLOUR_TABLE_TOML.replacen("version = 1", "version = 2", 1)
            ),
            Err(CategoryTableError::Version {
                table: "beast colour",
                ..
            })
        ));
        assert!(matches!(
            Tables::parse("version = 1\nnope = 2", COLOUR_TABLE_TOML),
            Err(CategoryTableError::Parse(_))
        ));
        // one class, one id
        let twice = ok.replacen(
            "classes = [\"Claws\"]",
            "classes = [\"Claws\", \"Bows\"]",
            1,
        );
        assert!(refused(&twice, COLOUR_TABLE_TOML).contains("\"Bows\" is under two ids"));
        let no_class = ok.replacen("classes = [\"Claws\"]", "classes = [\"Clawz\"]", 1);
        assert!(refused(&no_class, COLOUR_TABLE_TOML).contains("no class of the class table"));
        let realms = ok.replacen(
            "realms = [\"pc\", \"xbox\", \"sony\"]",
            "realms = [\"pc\"]",
            1,
        );
        assert!(refused(&realms, COLOUR_TABLE_TOML).contains("covers pc"));
        let member = ok.replacen("{ leaf = \"weapon.claw\"", "{ leaf = \"weapon\"", 1);
        assert!(refused(&member, COLOUR_TABLE_TOML).contains("which is no leaf"));
        let colour = COLOUR_TABLE_TOML.replacen("colour = \"yellow\"", "colour = \"green\"", 1);
        assert!(refused(ok, &colour).contains("no leaf for"));
        let named_twice =
            COLOUR_TABLE_TOML.replacen("name = \"Barrow Ape\"", "name = \"Alpine Devil\"", 1);
        assert!(refused(ok, &named_twice).contains("named twice"));
    }
}
