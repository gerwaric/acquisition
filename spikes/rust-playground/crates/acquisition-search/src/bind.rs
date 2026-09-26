//! The binder (C91, C92, C93; the build plan, step 4): a tree in, a query
//! the evaluator can run out — or an authoring error. The parser knows the
//! grammar and no name; every name is known here, from closed lists this
//! file holds, so a query's validity never depends on a corpus (invariant
//! 3 of the surface).
//!
//! What the words mean is `search/DESIGN.md`'s reference (*Item-level*,
//! *Members*, *Values*), cited by section and not restated (the build
//! plan, rule 6).
//!
//! # As built
//!
//! - **One closed list of what is not built** ([`NOT_BUILT`]): a construct
//!   of the reference and the step that builds it. A query, a sort or a
//!   flag that uses one is refused before anything is evaluated, by an
//!   error of its own kind ([`ErrorKind::NotBuilt`]) that names the
//!   construct — never the unknown-name error, never undecided, never an
//!   empty answer. `--describe` prints this list, and
//!   `tests/refusal.rs` walks it.
//! - **Every text comparison is one matcher, in any case.** `:` contains,
//!   `=` the whole value, `~` a pattern (Rust regex syntax, unanchored);
//!   each compiles to a regex, so `rarity=rare` finds GGG's `Rare` and
//!   `name="kaom's heart"` finds `Kaom's Heart` by the same rule. A pattern
//!   may turn case back on for itself: `(?-i)`.
//! - **A closed set is a list here**: `rarity`, `frame`, the `is:` words,
//!   a line's `source` and its flags — GGG's own spellings, as the census
//!   of 2026-09-13 met them (22,721 items), matched in any case — and
//!   `class`, whose list is the class table's names (`class.rs`, C106).
//!   `=` names one value, `:` and `~` pick among the legal ones, and a
//!   word that picks none is an authoring error with the near ones
//!   offered. A flag or a source GGG adds is derived and shown the day it
//!   appears and cannot be asked for until its list gains it.
//! - **`reqlevel` is a number** the deriver reads from the `Level`
//!   requirement (the build plan, step 6): `reqlevel=..30`, `-has:reqlevel`.
//! - **Atomic terms are numbered by path** — `0`, `1`, `3.1` — as the
//!   answer's terms block prints them (C93): a child of an and, an or or a
//!   `holds` is `<parent>.<n>`, what a not negates or `undecided( … )`
//!   asks about is `<parent>.0`, and the children of the root carry no
//!   prefix.
//! - **A line's group is bound in `group.rs`**, the one module that reads
//!   a group's tree for what it means: its selector, the sources it admits,
//!   its together bound, its template tests.
//! - **A bare word's closed-set readings** arrive here: the parser offers
//!   `"rare"` and `line(template:rare)`, and [`parse_query`] puts
//!   `rarity=rare` before them.
//! - **A computed value is bound by name** (`pseudo.rs`; the build plan,
//!   step 7): a total of the table or a derived field, in any case, with
//!   its slot word checked against what it is — a ranged total takes one,
//!   nothing else does. A name nothing defines is an unknown name with the
//!   near ones offered, or, with a slot word, the refusal of a ranged
//!   total, which this build ships none of.
//! - **The sockets are numbers** (`sockets.rs`, C101; the build plan, step
//!   8): `sockets`, `links` and `sockets.<colour>` for the four colour
//!   words are fields of `Kind::Number`, so a comparison, `has:`, a
//!   sort, a sum and a count's key take them as they take `ilvl`; a colour
//!   word outside the four is an authoring error offering them. A link
//!   group, `linked( … )`, is bound in `group.rs` beside a line's.
//! - **The price is four names** (`price.rs`, C81, C100; the build plan,
//!   step 9): `priced` is presence alone — `has:priced`, `-has:priced`,
//!   `undecided(priced)` — and a comparison on it or a count by it is an
//!   authoring error that offers those, or `price.currency`; `price.amount`
//!   and `price.lot` are numbers, and `price.currency` a closed set whose
//!   legal values are the currency table's tags (C68), so a tag outside
//!   the table is an authoring error with the near ones offered.

use regex::{Regex, RegexBuilder};

use crate::error::{ErrorKind, LanguageError};
use crate::group::Group;
use crate::print;
use crate::tree::{self, Collection, Node, Number, Op, Probe, Value, ValueRef};

// ---- what is not built -------------------------------------------------------

/// A construct of the reference this build refuses, and the step of
/// `search/BUILD-PLAN.md` that builds it — none, where no step of the plan
/// names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct NotBuilt {
    pub construct: &'static str,
    pub step: Option<u8>,
    pub what: &'static str,
}

/// The one list (the build plan, rule 1). An entry leaves in the commit
/// that builds its construct.
pub const NOT_BUILT: &[NotBuilt] = &[
    NotBuilt {
        construct: "pseudo.defence_pct",
        step: None,
        what: "the base defence percentile (C101): its formula is not pinned against the site's (search/pseudo-stats/README.md, open question 3)",
    },
    NotBuilt {
        construct: "pseudo.<name>.<slot>",
        step: None,
        what: "a ranged total: the totals table admits one, and this build ships none — which lines a site's ranged pseudo sums is unread (search/pseudo-stats/README.md, open question 2)",
    },
    NotBuilt {
        construct: "--fields",
        step: Some(10),
        what: "the caller names a row's fields",
    },
    NotBuilt {
        construct: "--next",
        step: Some(10),
        what: "continue an answer past its limit, refused across a changed basis",
    },
    NotBuilt {
        construct: "--explain",
        step: Some(10),
        what: "one node forced true and forced false",
    },
    NotBuilt {
        construct: "--context",
        step: Some(10),
        what: "the empty query over the bound scope",
    },
    NotBuilt {
        construct: "--view locations",
        step: Some(10),
        what: "the full coverage list the scope block summarises",
    },
    NotBuilt {
        construct: "--print-request",
        step: Some(10),
        what: "print the request as JSON without running it",
    },
    NotBuilt {
        construct: "--request",
        step: Some(10),
        what: "run a request read from a file",
    },
    NotBuilt {
        construct: "--rebind",
        step: Some(10),
        what: "run another account's request deliberately",
    },
    NotBuilt {
        construct: "show --against",
        step: Some(10),
        what: "why one item does or does not match a query",
    },
    NotBuilt {
        construct: "show --basis",
        step: Some(10),
        what: "one item as a named basis held it",
    },
];

impl NotBuilt {
    /// The step as the refusal and the help print it.
    pub fn step_text(&self) -> String {
        match self.step {
            Some(step) => format!("step {step}"),
            None => "no step of the plan".to_string(),
        }
    }
}

/// The refusal of one construct of [`NOT_BUILT`], by its name there.
pub fn not_built(construct: &str) -> LanguageError {
    match NOT_BUILT.iter().find(|n| n.construct == construct) {
        Some(n) => LanguageError::new(
            ErrorKind::NotBuilt,
            format!(
                "not built: {} ({}) — {}",
                n.construct,
                n.step_text(),
                n.what
            ),
        ),
        // a name outside the list is a bug in the caller, said as loudly
        None => LanguageError::new(
            ErrorKind::NotBuilt,
            format!("not built: {construct} (no step names it)"),
        ),
    }
}

// ---- the vocabulary ------------------------------------------------------------

pub(crate) const RARITIES: &[&str] = &["normal", "magic", "rare", "unique"];

fn rarities() -> &'static [&'static str] {
    RARITIES
}

fn frames() -> &'static [&'static str] {
    FRAMES
}

/// GGG's `frameTypeId`, the nine the census met.
pub(crate) const FRAMES: &[&str] = &[
    "normal",
    "magic",
    "rare",
    "unique",
    "gem",
    "currency",
    "divinationcard",
    "quest",
    "supporterfoil",
];

pub(crate) use crate::derive::ITEM_FLAGS;

/// A line's source: its array's key without `Mods`, and `hybrid`.
pub(crate) const SOURCES: &[&str] = &[
    "bonded", "crucible", "enchant", "explicit", "hybrid", "implicit", "rune", "scourge",
    "utility", "veiled",
];

/// The flags a line carries (S17).
pub(crate) const LINE_FLAGS: &[&str] = &["crafted", "fractured", "mutated"];

/// Something of an item a term names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Thing {
    /// Every displayed string, each on its own.
    Text,
    Name,
    Typeline,
    Base,
    Note,
    Rarity,
    Frame,
    /// The class table's word for the item's base (`class.rs`).
    Class,
    Ilvl,
    /// The `Level` requirement, as a number.
    ReqLevel,
    Stack,
    /// How many sockets the item has (`sockets.rs`).
    Sockets,
    /// The size of its largest link group.
    Links,
    /// How many sockets of one colour, by GGG's letter.
    SocketColour(&'static str),
    /// Whether the item carries an effective price (`price.rs`).
    Price,
    /// The number the price states.
    PriceAmount,
    /// The currency table's tag.
    PriceCurrency,
    /// A bulk ratio's lot.
    PriceLot,
    League,
    Tab,
    Character,
    Container,
    Id,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Kind {
    Text,
    /// The legal values, read when asked: a list this file holds, or the
    /// class table's names.
    Closed(fn() -> &'static [&'static str]),
    Number,
    Handle,
    /// Asked with `has:` alone: yes, no, or not established.
    Presence,
}

pub(crate) struct FieldDef {
    pub name: &'static str,
    pub thing: Thing,
    pub kind: Kind,
    pub what: &'static str,
}

pub(crate) const FIELDS: &[FieldDef] = &[
    FieldDef {
        name: "text",
        thing: Thing::Text,
        kind: Kind::Text,
        what: "every displayed string, each tested on its own: the name, the type line, the base, each property, the item level, the requirements' row, each row of each line; what a quoted phrase searches",
    },
    FieldDef {
        name: "name",
        thing: Thing::Name,
        kind: Kind::Text,
        what: "GGG's name; a magic or normal item, a gem, a currency stack has none",
    },
    FieldDef {
        name: "typeline",
        thing: Thing::Typeline,
        kind: Kind::Text,
        what: "GGG's type line: the base with what it carries, a magic item's affix names among it",
    },
    FieldDef {
        name: "base",
        thing: Thing::Base,
        kind: Kind::Text,
        what: "GGG's base type",
    },
    FieldDef {
        name: "note",
        thing: Thing::Note,
        kind: Kind::Text,
        what: "the item's note as GGG gives it; never part of `text`",
    },
    FieldDef {
        name: "rarity",
        thing: Thing::Rarity,
        kind: Kind::Closed(rarities),
        what: "GGG's rarity; a gem, a currency stack, a card has none",
    },
    FieldDef {
        name: "frame",
        thing: Thing::Frame,
        kind: Kind::Closed(frames),
        what: "GGG's frameTypeId, on every item",
    },
    FieldDef {
        name: "class",
        thing: Thing::Class,
        kind: Kind::Closed(crate::class::names),
        what: "the item's class as the game names it, read from its base in the class table",
    },
    FieldDef {
        name: "ilvl",
        thing: Thing::Ilvl,
        kind: Kind::Number,
        what: "the item level; GGG's 0 is an item with none",
    },
    FieldDef {
        name: "reqlevel",
        thing: Thing::ReqLevel,
        kind: Kind::Number,
        what: "the level the item requires: its `Level` requirement's number; an item with no such requirement lacks it",
    },
    FieldDef {
        name: "stack",
        thing: Thing::Stack,
        kind: Kind::Number,
        what: "GGG's stackSize",
    },
    FieldDef {
        name: "sockets",
        thing: Thing::Sockets,
        kind: Kind::Number,
        what: "how many sockets the item has, of every colour; an item whose body lists none lacks it, and one that lists an empty collection has 0",
    },
    FieldDef {
        name: "links",
        thing: Thing::Links,
        kind: Kind::Number,
        what: "the size of the item's largest link group; an item with no link group lacks it",
    },
    FieldDef {
        name: "sockets.red",
        thing: Thing::SocketColour("R"),
        kind: Kind::Number,
        what: "how many red sockets the item has, over every link group",
    },
    FieldDef {
        name: "sockets.green",
        thing: Thing::SocketColour("G"),
        kind: Kind::Number,
        what: "how many green sockets the item has, over every link group",
    },
    FieldDef {
        name: "sockets.blue",
        thing: Thing::SocketColour("B"),
        kind: Kind::Number,
        what: "how many blue sockets the item has, over every link group",
    },
    FieldDef {
        name: "sockets.white",
        thing: Thing::SocketColour("W"),
        kind: Kind::Number,
        what: "how many white sockets the item has, over every link group",
    },
    FieldDef {
        name: "priced",
        thing: Thing::Price,
        kind: Kind::Presence,
        what: "whether the item carries an effective price (C81): the game's note or public tab name, or the owner's row, whichever is the more specific, the game's on a tie; a listing resolved to none, no price or skip lacks it; one a row that cannot be read could decide is undecided",
    },
    FieldDef {
        name: "price.amount",
        thing: Thing::PriceAmount,
        kind: Kind::Number,
        what: "the number the effective price states: a decimal price's decimal, or a bulk ratio's wanted",
    },
    FieldDef {
        name: "price.currency",
        thing: Thing::PriceCurrency,
        kind: Kind::Closed(crate::price::currency_tags),
        what: "the effective price's currency, by the tag of the currency table (C68)",
    },
    FieldDef {
        name: "price.lot",
        thing: Thing::PriceLot,
        kind: Kind::Number,
        what: "a bulk ratio's lot — `wanted/lot` — which a decimal price lacks",
    },
    FieldDef {
        name: "league",
        thing: Thing::League,
        kind: Kind::Text,
        what: "the league the item is in now — its place, never where it came from: league of origin is not a fact the search holds (S177)",
    },
    FieldDef {
        name: "tab",
        thing: Thing::Tab,
        kind: Kind::Text,
        what: "the name of the stash tab the item is in; in a substash, that name and its tab's",
    },
    FieldDef {
        name: "character",
        thing: Thing::Character,
        kind: Kind::Text,
        what: "the name of the character the item is on",
    },
    FieldDef {
        name: "container",
        thing: Thing::Container,
        kind: Kind::Text,
        what: "the array the item came in: items, equipment, inventory, jewels, skills, rucksack, guardian",
    },
    FieldDef {
        name: "id",
        thing: Thing::Id,
        kind: Kind::Handle,
        what: "any id an answer printed, whole: the item's own, or its tab's, its substash's or its character's",
    },
];

fn field(name: &str) -> Option<&'static FieldDef> {
    FIELDS.iter().find(|f| f.name.eq_ignore_ascii_case(name))
}

/// A closed value by any-case name, in the list's own spelling.
pub(crate) fn legal(list: &'static [&'static str], word: &str) -> Option<&'static str> {
    list.iter().copied().find(|v| v.eq_ignore_ascii_case(word))
}

// ---- near names ------------------------------------------------------------------

fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut last = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { last } else { last + 1 };
            last = row[j + 1];
            row[j + 1] = cost.min(last + 1).min(row[j] + 1);
        }
    }
    row[b.len()]
}

/// The known names a typed one may have meant, nearest first: an edit or
/// two away, or one the start of the other. A suggestion, never a binding.
pub(crate) fn near<'a>(word: &str, known: &[&'a str]) -> Vec<&'a str> {
    let word = word.to_ascii_lowercase();
    let mut scored: Vec<(usize, &'a str)> = known
        .iter()
        .filter_map(|k| {
            let lower = k.to_ascii_lowercase();
            let d = distance(&word, &lower);
            let budget = if word.chars().count() <= 3 { 1 } else { 2 };
            if d <= budget {
                Some((d, *k))
            } else if lower.starts_with(&word) || word.starts_with(&lower) {
                Some((budget + 1, *k))
            } else {
                None
            }
        })
        .collect();
    scored.sort();
    scored.into_iter().map(|(_, k)| k).take(3).collect()
}

// ---- the bound query -----------------------------------------------------------------

/// One text comparison, compiled (the module doc: one matcher, any case).
#[derive(Debug, Clone)]
pub(crate) struct TextTest(Regex);

impl TextTest {
    pub fn holds(&self, text: &str) -> bool {
        self.0.is_match(text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NumTest {
    Cmp(Op, f64),
    Range(Option<f64>, Option<f64>),
}

impl NumTest {
    pub fn holds(&self, x: f64) -> bool {
        match *self {
            NumTest::Cmp(Op::Gt, n) => x > n,
            NumTest::Cmp(Op::Ge, n) => x >= n,
            NumTest::Cmp(Op::Lt, n) => x < n,
            NumTest::Cmp(Op::Le, n) => x <= n,
            NumTest::Cmp(_, n) => x == n,
            NumTest::Range(from, to) => from.is_none_or(|a| x >= a) && to.is_none_or(|b| x <= b),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Atom {
    Text {
        thing: Thing,
        test: TextTest,
    },
    Closed {
        thing: Thing,
        values: Vec<&'static str>,
    },
    Number {
        thing: Thing,
        test: NumTest,
    },
    Id(String),
    Has(Thing),
    /// `has:` on a derived field (T2); the term's node carries the name.
    HasComputed(crate::pseudo::Named),
    Is(&'static str),
    Lines(Group),
    /// `linked( … )`: conditions that hold together on one link group.
    Links(crate::group::Linked),
    Sum {
        group: Group,
        slot: String,
        test: NumTest,
    },
    /// A computed value compared (`pseudo.rs`): `name` as printed,
    /// `pseudo.total_res`.
    Pseudo {
        named: crate::pseudo::Named,
        name: String,
        slot: Option<String>,
        test: NumTest,
    },
    Const(bool),
    Undecided(BProbe),
}

#[derive(Debug, Clone)]
pub(crate) enum BProbe {
    Field(Thing),
    /// A sum or a projection, as `--sort` takes one: open exactly when it
    /// would sort as incomplete.
    Value(Box<SortKey>),
    Term(Box<Bound>),
}

#[derive(Debug, Clone)]
pub(crate) enum Bound {
    All(Vec<Bound>),
    Any(Vec<Bound>),
    Not(Box<Bound>),
    Holds {
        of: Vec<Bound>,
        min: Option<u32>,
        max: Option<u32>,
    },
    /// An atomic term, by its index in [`Query::terms`].
    Term(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct Term {
    pub path: String,
    pub node: Node,
    pub atom: Atom,
}

/// A query the evaluator can run: the tree as authored, and what its
/// names were bound to.
#[derive(Debug, Clone)]
pub struct Query {
    pub(crate) root: Node,
    pub(crate) bound: Bound,
    pub(crate) terms: Vec<Term>,
}

impl Query {
    pub fn tree(&self) -> &Node {
        &self.root
    }

    pub fn text(&self) -> String {
        print::print(&self.root)
    }
}

/// What `--sort` orders by (C92, the sort scalar).
#[derive(Debug, Clone)]
pub(crate) enum SortKey {
    Number(Thing),
    Projection {
        group: Group,
        slot: String,
    },
    Sum {
        group: Group,
        slot: String,
    },
    /// A computed value; the printed name is the caller's (`--sort` and
    /// `--sum` print the value they were given).
    Pseudo {
        named: crate::pseudo::Named,
        slot: Option<String>,
    },
}

/// Parse and bind a query's text.
pub fn parse_query(text: &str) -> Result<Query, LanguageError> {
    let root = crate::parse::parse(text).map_err(|e| closed_set_readings(e, text))?;
    bind(&root)
}

/// Bind a tree: checked first, since one that arrived as JSON may not be a
/// tree the language can say.
pub fn bind(root: &Node) -> Result<Query, LanguageError> {
    tree::check(root)?;
    let mut binder = Binder { terms: Vec::new() };
    let bound = match root {
        Node::All(children) => Bound::All(binder.children(children, "")?),
        other => binder.node(other, "0".to_string())?,
    };
    Ok(Query {
        root: root.clone(),
        bound,
        terms: binder.terms,
    })
}

// ---- what a count groups by ---------------------------------------------------------------

/// A key of the counts view (C95, C105): a field an item has or lacks, or
/// `line`, the vocabulary read (C97), narrowed by a text or a pattern.
pub(crate) enum Key {
    Field(&'static FieldDef),
    Line(Option<(Op, String)>),
}

/// Bind one key, as a request names it: a field's name, `line`,
/// `line:<text>` or `line~<pattern>` — the rest of the string is the text,
/// as it stands, whitespace and all; a name is trimmed, a text never (outside
/// review, 2026-09-22). A field is a key when `has:` can be asked of it, since
/// `-has:<key>` is where its `none` bucket routes: every field but `text`
/// and `id`. What a later step builds is refused by that step's name.
pub(crate) fn bind_key(text: &str) -> Result<Key, LanguageError> {
    let narrowed = text
        .char_indices()
        .find(|(_, c)| matches!(c, ':' | '~'))
        .map(|(at, c)| (&text[..at], c, &text[at + 1..]));
    if let Some((name, op, rest)) = narrowed {
        let name = name.trim();
        if !name.eq_ignore_ascii_case("line") {
            return Err(LanguageError::new(
                ErrorKind::View,
                format!("`{name}` is counted whole: only `line` takes a text to narrow by"),
            )
            .with_readings(vec![name.to_string()]));
        }
        if rest.is_empty() {
            return Err(LanguageError::new(
                ErrorKind::View,
                "`line:` takes a text to narrow by; `line` alone is every template",
            ));
        }
        let op = if op == ':' { Op::Contains } else { Op::Match };
        return Ok(Key::Line(Some((op, rest.to_string()))));
    }
    let text = text.trim();
    if text.eq_ignore_ascii_case("line") {
        return Ok(Key::Line(None));
    }
    let known = || -> Vec<&'static str> {
        FIELDS
            .iter()
            .filter(|f| !matches!(f.thing, Thing::Text | Thing::Id | Thing::Price))
            .map(|f| f.name)
            .chain(["line"])
            .collect()
    };
    match field(text) {
        Some(def) if def.thing == Thing::Price => Err(LanguageError::new(
            ErrorKind::View,
            format!(
                "`{}` is yes or no, not a value to count by: ask has:priced, or count by price.currency",
                def.name
            ),
        )
        .with_readings(vec!["price.currency".to_string()])),
        Some(def) if !matches!(def.thing, Thing::Text | Thing::Id) => Ok(Key::Field(def)),
        Some(def) => Err(LanguageError::new(
            ErrorKind::View,
            format!(
                "`{}` is no key: every item has it, and no two share a value worth a table — {}",
                def.name,
                known().join(", ")
            ),
        )),
        None => Err(unknown("key", text, &known(), |near| near.to_string())),
    }
}

/// What two texts share exactly when any-case `=` holds between them: each
/// character replaced by the least of those the matcher takes for it. Read
/// from the matcher's own tables (`regex-syntax`, which `regex` compiles
/// with), never from `to_lowercase`, which is another relation — it keeps
/// `ſ` from `s` where the matcher does not. A count tells two spellings of
/// one value apart by it, so that a bucket's term selects that bucket and
/// no other (invariant 4 of the surface).
pub(crate) fn folded(text: &str) -> String {
    use regex_syntax::hir::{ClassUnicode, ClassUnicodeRange};
    text.chars()
        .map(|c| {
            if c.is_ascii() {
                // an ASCII letter's capital is the least of its class
                return c.to_ascii_uppercase();
            }
            let mut class = ClassUnicode::new([ClassUnicodeRange::new(c, c)]);
            match class.try_case_fold_simple() {
                Ok(()) => class.ranges().first().map_or(c, |r| r.start()),
                Err(_) => c,
            }
        })
        .collect()
}

/// The pattern that selects one spelling of a text and no other: `~` with
/// case turned back on, anchored at both ends (the reference, *Members*:
/// `template~"(?-i)^Gain # Life per enemy killed$"`). Escaped are the
/// characters a pattern reads as syntax where a text stands, and no more —
/// a template's `#` and `%` are themselves there — so that the term reads
/// as the reference writes it.
pub(crate) fn exact_pattern(text: &str) -> String {
    let mut out = String::from("(?-i)^");
    for c in text.chars() {
        if r"\.+*?()|[]{}^$".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('$');
    out
}

/// Bind what `--sort` takes.
pub(crate) fn bind_sort(value: &ValueRef) -> Result<SortKey, LanguageError> {
    bind_value("--sort", value)
}

/// Bind what `--sum` takes (C95): one number of an item, added over a
/// bucket's items. A field, or the item's own `sum( … )`; never
/// `line(P).<slot>`, which is one occurrence's and says nothing of which —
/// the item's sum of it is offered instead.
pub(crate) fn bind_sum(value: &ValueRef) -> Result<SortKey, LanguageError> {
    if let ValueRef::Projection { lines, slot } = value {
        return Err(LanguageError::new(
            ErrorKind::View,
            "`--sum` adds one number for each item, and a line's slot is one occurrence's: the item's sum of it is",
        )
        .with_readings(vec![print::print_value(&ValueRef::Sum {
            lines: lines.clone(),
            slot: slot.clone(),
        })]));
    }
    bind_value("--sum", value)
}

fn bind_value(flag: &str, value: &ValueRef) -> Result<SortKey, LanguageError> {
    match value {
        ValueRef::Pseudo { name, slot } => {
            let (named, slot) = bind_pseudo(name, slot.as_deref())?;
            Ok(SortKey::Pseudo { named, slot })
        }
        ValueRef::Field(name) => {
            let def = known_field(name, |near| near.to_string())?;
            match def.kind {
                Kind::Number => Ok(SortKey::Number(def.thing)),
                _ => Err(LanguageError::new(
                    ErrorKind::OperatorMismatch,
                    format!(
                        "`{flag}` takes a number: `{}` is not one — ilvl, stack, {}",
                        def.name,
                        if flag == "--sum" {
                            "or the item's sum( … )"
                        } else {
                            "a line's slot or a sum"
                        }
                    ),
                )),
            }
        }
        ValueRef::Projection { lines, slot } => Ok(SortKey::Projection {
            group: Group::bind(lines)?,
            slot: slot.clone(),
        }),
        ValueRef::Sum { lines, slot } => Ok(SortKey::Sum {
            group: Group::bind(lines)?,
            slot: slot.clone(),
        }),
    }
}

/// A computed value by name (the module doc): the total or the derived
/// field, and its slot word where it takes one.
fn bind_pseudo(
    name: &str,
    slot: Option<&str>,
) -> Result<(crate::pseudo::Named, Option<String>), LanguageError> {
    let printed = |slot: Option<&str>| {
        print::print_value(&ValueRef::Pseudo {
            name: name.to_string(),
            slot: slot.map(str::to_string),
        })
    };
    if name.eq_ignore_ascii_case("defence_pct") {
        return Err(not_built("pseudo.defence_pct"));
    }
    let known = crate::pseudo::names();
    let Some(named) = crate::pseudo::lookup(name)? else {
        let near: Vec<String> = near(name, known)
            .into_iter()
            .map(|n| {
                print::print_value(&ValueRef::Pseudo {
                    name: n.to_string(),
                    slot: None,
                })
            })
            .collect();
        return Err(if slot.is_some() {
            not_built("pseudo.<name>.<slot>").with_readings(near)
        } else {
            unknown("computed value", name, known, |near| {
                print::print_value(&ValueRef::Pseudo {
                    name: near.to_string(),
                    slot: None,
                })
            })
        });
    };
    match (named.ranged(), slot) {
        (true, Some(slot)) => Ok((named, Some(slot.to_string()))),
        (true, None) => Err(LanguageError::new(
            ErrorKind::SlotMissing,
            format!("`{}` is a range: name low, high or avg", printed(None)),
        )
        .with_readings(
            ["avg", "low", "high"]
                .into_iter()
                .map(|s| printed(Some(s)))
                .collect(),
        )),
        (false, None) => Ok((named, None)),
        (false, Some(_)) => Err(LanguageError::new(
            ErrorKind::SlotUnknown,
            format!(
                "`{}` is one number, not a range: it takes no slot word",
                printed(None)
            ),
        )
        .with_readings(vec![printed(None)])),
    }
}

struct Binder {
    terms: Vec<Term>,
}

impl Binder {
    fn children(&mut self, children: &[Node], prefix: &str) -> Result<Vec<Bound>, LanguageError> {
        children
            .iter()
            .enumerate()
            .map(|(i, child)| {
                let path = if prefix.is_empty() {
                    i.to_string()
                } else {
                    format!("{prefix}.{i}")
                };
                self.node(child, path)
            })
            .collect()
    }

    fn node(&mut self, node: &Node, path: String) -> Result<Bound, LanguageError> {
        Ok(match node {
            Node::All(children) => Bound::All(self.children(children, &path)?),
            Node::Any(children) => Bound::Any(self.children(children, &path)?),
            Node::Not(inner) => Bound::Not(Box::new(self.node(inner, format!("{path}.0"))?)),
            Node::Holds { of, min, max } => Bound::Holds {
                of: self.children(of, &path)?,
                min: *min,
                max: *max,
            },
            leaf => {
                let atom = self.atom(leaf, &path)?;
                self.terms.push(Term {
                    path,
                    node: leaf.clone(),
                    atom,
                });
                Bound::Term(self.terms.len() - 1)
            }
        })
    }

    fn atom(&mut self, node: &Node, path: &str) -> Result<Atom, LanguageError> {
        match node {
            Node::Const(value) => Ok(Atom::Const(*value)),
            Node::Test { field, op, value } => {
                let def = known_field(field, |near| {
                    print::print(&Node::Test {
                        field: near.to_string(),
                        op: *op,
                        value: value.clone(),
                    })
                })?;
                test(def, *op, value)
            }
            Node::Has(name) if name.starts_with("pseudo.") => {
                // a derived field's absence is a property's, which `has:`
                // asks; a total's is never absence (owner, 2026-09-24, T2:
                // "has: applies to a derived field, never to a total. A
                // ring has no dps; every item has a total.")
                let (named, _) = bind_pseudo(&name["pseudo.".len()..], None)?;
                match named {
                    crate::pseudo::Named::Derived(_) => Ok(Atom::HasComputed(named)),
                    crate::pseudo::Named::Total { .. } => Err(tree::has_on_computed(name)),
                }
            }
            Node::Has(name) => {
                let def = known_field(name, |near| format!("has:{near}"))?;
                match def.thing {
                    Thing::Text | Thing::Id => Err(LanguageError::new(
                        ErrorKind::OperatorMismatch,
                        format!("every item has `{}`: `has:` asks nothing of it", def.name),
                    )),
                    thing => Ok(Atom::Has(thing)),
                }
            }
            Node::Is(name) => match legal(ITEM_FLAGS, name) {
                Some(flag) => Ok(Atom::Is(flag)),
                None if legal(LINE_FLAGS, name).is_some() => Err(LanguageError::new(
                    ErrorKind::UnknownName,
                    format!("`is:{name}` is said of a line, inside its group"),
                )
                .with_readings(vec![format!("line(is:{name})")])),
                None => Err(unknown("is:", name, ITEM_FLAGS, |near| {
                    format!("is:{near}")
                })),
            },
            Node::Members {
                of: Collection::Links,
                where_,
            } => Ok(Atom::Links(crate::group::Linked::bind(where_)?)),
            Node::Members {
                of: Collection::Lines,
                where_,
            } => Ok(Atom::Lines(Group::bind(where_)?)),
            Node::Compare { value, op, rhs } => match value {
                ValueRef::Sum { lines, slot } => Ok(Atom::Sum {
                    group: Group::bind(lines)?,
                    slot: slot.clone(),
                    test: num_test("sum( … )", *op, rhs)?,
                }),
                ValueRef::Pseudo { name, slot } => {
                    let (named, slot) = bind_pseudo(name, slot.as_deref())?;
                    let printed = print::print_value(value);
                    Ok(Atom::Pseudo {
                        named,
                        test: num_test(&printed, *op, rhs)?,
                        name: printed,
                        slot,
                    })
                }
                // `tree::check` has refused both
                ValueRef::Field(_) | ValueRef::Projection { .. } => Err(LanguageError::new(
                    ErrorKind::Tree,
                    "a comparison is on a sum or a computed value",
                )),
            },
            Node::Undecided(probe) => Ok(Atom::Undecided(match probe {
                Probe::Thing(ValueRef::Field(name)) => {
                    BProbe::Field(known_field(name, |near| format!("undecided({near})"))?.thing)
                }
                Probe::Thing(
                    value @ (ValueRef::Sum { .. }
                    | ValueRef::Projection { .. }
                    | ValueRef::Pseudo { .. }),
                ) => BProbe::Value(Box::new(bind_sort(value)?)),
                Probe::Term(inner) => {
                    BProbe::Term(Box::new(self.node(inner, format!("{path}.0"))?))
                }
            })),
            // composition is `node`'s
            Node::All(_) | Node::Any(_) | Node::Not(_) | Node::Holds { .. } => Err(
                LanguageError::new(ErrorKind::Tree, "a composition is not an atomic term"),
            ),
        }
    }
}

fn known_field(
    name: &str,
    reading: impl Fn(&str) -> String,
) -> Result<&'static FieldDef, LanguageError> {
    if let Some(field) = field(name) {
        return Ok(field);
    }
    // `sockets.<colour>`: the four colour words, offered whole
    if let Some(colour) = name.strip_prefix("sockets.") {
        let words = crate::sockets::colour_words();
        return Err(LanguageError::new(
            ErrorKind::UnknownValue,
            format!("`{colour}` is no socket colour: {}", words.join(", ")),
        )
        .with_readings(
            words
                .iter()
                .map(|word| reading(&format!("sockets.{word}")))
                .collect(),
        ));
    }
    let names: Vec<&'static str> = FIELDS.iter().map(|f| f.name).collect();
    Err(unknown("field", name, &names, reading))
}

pub(crate) fn unknown(
    what: &str,
    name: &str,
    known: &[&'static str],
    reading: impl Fn(&str) -> String,
) -> LanguageError {
    let near = near(name, known);
    let message = if near.is_empty() {
        format!(
            "`{name}` is no {what} the language knows: {}",
            known.join(", ")
        )
    } else {
        format!("`{name}` is no {what} the language knows; near it:")
    };
    LanguageError::new(ErrorKind::UnknownName, message)
        .with_readings(near.into_iter().map(reading).collect())
}

/// A closed set's legal values; the class's are the table's, and a build
/// whose table does not load says so rather than offering an empty list.
pub(crate) fn values_of(def: &FieldDef) -> Result<&'static [&'static str], LanguageError> {
    let Kind::Closed(list) = def.kind else {
        return Ok(&[]);
    };
    if def.thing == Thing::Class {
        crate::class::table().map_err(|e| {
            LanguageError::new(
                ErrorKind::Tree,
                format!("the class table this build ships does not load: {e}"),
            )
        })?;
    }
    if def.thing == Thing::PriceCurrency {
        crate::price::currency_table().map_err(|e| {
            LanguageError::new(
                ErrorKind::Tree,
                format!("the currency table this build ships does not load: {e}"),
            )
        })?;
    }
    Ok(list())
}

fn test(def: &'static FieldDef, op: Op, value: &Value) -> Result<Atom, LanguageError> {
    match def.kind {
        Kind::Text => Ok(Atom::Text {
            thing: def.thing,
            test: text_test(def.name, op, value)?,
        }),
        Kind::Closed(_) => Ok(Atom::Closed {
            thing: def.thing,
            values: closed(def.name, values_of(def)?, op, value)?,
        }),
        Kind::Number => match op {
            Op::Contains | Op::Match => Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!("`{}` is a number: it takes = > >= < <= and a..b", def.name),
            )
            .with_readings(match value {
                Value::Number(_) => vec![print::print(&Node::Test {
                    field: def.name.to_string(),
                    op: Op::Eq,
                    value: value.clone(),
                })],
                _ => Vec::new(),
            })),
            _ => Ok(Atom::Number {
                thing: def.thing,
                test: num_test(def.name, op, value)?,
            }),
        },
        Kind::Presence => Err(LanguageError::new(
            ErrorKind::OperatorMismatch,
            format!(
                "`{}` is yes or no: it is asked with has: — has:{0}, -has:{0}, undecided({0})",
                def.name
            ),
        )
        .with_readings(vec![
            format!("has:{}", def.name),
            format!("-has:{}", def.name),
            format!("undecided({})", def.name),
        ])),
        Kind::Handle => match (op, value) {
            (Op::Contains | Op::Eq, Value::Text(id)) => Ok(Atom::Id(id.clone())),
            (Op::Contains | Op::Eq, Value::Number(n)) => Ok(Atom::Id(number_text(*n))),
            _ => Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                "`id` takes `:` or `=` and a whole id, as an answer printed it",
            )),
        },
    }
}

fn number_text(n: Number) -> String {
    match n {
        Number::Int(i) => i.to_string(),
        Number::Float(f) => f.to_string(),
    }
}

pub(crate) fn text_test(name: &str, op: Op, value: &Value) -> Result<TextTest, LanguageError> {
    let text = match value {
        Value::Text(text) => text.clone(),
        Value::Number(n) => number_text(*n),
        Value::Range { .. } => {
            return Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!("`{name}` is text: a range is for a number"),
            ));
        }
    };
    let pattern = match op {
        Op::Contains => regex::escape(&text),
        Op::Eq => format!("^(?:{})$", regex::escape(&text)),
        Op::Match => text.clone(),
        _ => {
            return Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!(
                    "`{name}` is text: it takes `:` contains, `=` the whole value, `~` a pattern"
                ),
            ));
        }
    };
    RegexBuilder::new(&pattern)
        .case_insensitive(true)
        .build()
        .map(TextTest)
        .map_err(|e| {
            LanguageError::new(
                ErrorKind::BadPattern,
                format!("`{name}~` takes a pattern in Rust regex syntax, and this is not one: {e}"),
            )
        })
}

pub(crate) fn num_test(name: &str, op: Op, value: &Value) -> Result<NumTest, LanguageError> {
    match (op, value) {
        (Op::Eq, Value::Range { from, to }) => Ok(NumTest::Range(
            from.map(Number::as_f64),
            to.map(Number::as_f64),
        )),
        (Op::Eq | Op::Gt | Op::Ge | Op::Lt | Op::Le, Value::Number(n)) => {
            Ok(NumTest::Cmp(op, n.as_f64()))
        }
        _ => Err(LanguageError::new(
            ErrorKind::OperatorMismatch,
            format!("`{name}` is a number: it takes = > >= < <= with a number, and = with a..b"),
        )),
    }
}

/// The legal values a closed-set test names or picks.
pub(crate) fn closed(
    name: &str,
    list: &'static [&'static str],
    op: Op,
    value: &Value,
) -> Result<Vec<&'static str>, LanguageError> {
    let Value::Text(word) = value else {
        return Err(LanguageError::new(
            ErrorKind::OperatorMismatch,
            format!("`{name}` is one of: {}", list.join(", ")),
        ));
    };
    let picked: Vec<&'static str> = match op {
        Op::Eq => legal(list, word).into_iter().collect(),
        Op::Contains => {
            let word = word.to_ascii_lowercase();
            list.iter()
                .copied()
                .filter(|v| v.to_ascii_lowercase().contains(&word))
                .collect()
        }
        Op::Match => {
            let test = text_test(name, op, value)?;
            list.iter().copied().filter(|v| test.holds(v)).collect()
        }
        _ => {
            return Err(LanguageError::new(
                ErrorKind::OperatorMismatch,
                format!(
                    "`{name}` is a closed set: `=` names one value, `:` and `~` pick among them"
                ),
            ));
        }
    };
    if !picked.is_empty() {
        return Ok(picked);
    }
    let offered = match near(word, list) {
        near if near.is_empty() => list.to_vec(),
        near => near,
    };
    Err(LanguageError::new(
        ErrorKind::UnknownValue,
        format!("`{word}` is no `{name}`: {}", list.join(", ")),
    )
    // through the printer, so a value with a space is quoted (the step-6
    // review, 3)
    .with_readings(
        offered
            .iter()
            .map(|v| {
                print::print(&Node::Test {
                    field: name.to_string(),
                    op: Op::Eq,
                    value: Value::Text((*v).to_string()),
                })
            })
            .collect(),
    ))
}

// ---- a bare word's closed-set readings -------------------------------------------------

/// The parser's bare-word error, with the readings only the vocabulary
/// knows put first: `rare` → `rarity=rare`, `corrupted` → `is:corrupted`.
fn closed_set_readings(mut e: LanguageError, text: &str) -> LanguageError {
    if e.kind != ErrorKind::BareWord {
        return e;
    }
    let Some(word) = e.span.and_then(|(start, end)| text.get(start..end)) else {
        return e;
    };
    // the parser's readings say which level the word stood at
    let inside = !e.readings.first().is_some_and(|r| r.starts_with('"'));
    let mut first = Vec::new();
    if inside {
        first.extend(legal(SOURCES, word).map(|v| format!("source={v}")));
        first.extend(legal(LINE_FLAGS, word).map(|v| format!("is:{v}")));
    } else {
        first.extend(legal(RARITIES, word).map(|v| format!("rarity={v}")));
        if legal(RARITIES, word).is_none() {
            first.extend(legal(FRAMES, word).map(|v| format!("frame={v}")));
        }
        // `ring` → `class:ring` (the reference, *Item-level*): a word some
        // class name holds picks it as `:` would
        let lower = word.to_ascii_lowercase();
        if crate::class::names()
            .iter()
            .any(|name| name.to_ascii_lowercase().contains(&lower))
        {
            first.push(format!("class:{word}"));
        }
        first.extend(legal(ITEM_FLAGS, word).map(|v| format!("is:{v}")));
    }
    first.append(&mut e.readings);
    e.readings = first;
    e
}

#[cfg(test)]
mod tests {
    use super::{exact_pattern, folded, text_test};
    use crate::tree::{Op, Value};
    use proptest::prelude::*;

    /// Characters by the family a case-blind reader might put them in —
    /// the Kelvin sign with `k`, the long s with `s`, the final sigma, the
    /// dotted and dotless i, a title-case digraph: where the matcher and
    /// `to_lowercase` disagree — and everything a pattern reads as syntax.
    const FAMILIES: &[&[char]] = &[
        &['a', 'A'],
        &['k', 'K', '\u{212A}'],
        &['s', 'S', 'ſ'],
        &['σ', 'ς', 'Σ'],
        &['ß', 'ẞ'],
        &['İ', 'i', 'I', 'ı'],
        &['é', 'É'],
        &['ǆ', 'ǅ', 'Ǆ'],
        &['#', '%', ' ', '\n', '"', '-', '&', '~'],
        &[
            '.', '(', ')', '[', ']', '\\', '^', '$', '|', '+', '*', '?', '{', '}',
        ],
    ];

    /// Two texts of one length whose characters are of one family at each
    /// place, so that a pair sharing a fold — and a pair a wrong fold would
    /// wrongly join or part — is met often, never once in thousands. (The
    /// first generator drew both from one pool at random, and the fold
    /// swapped for `to_lowercase` survived 4,000 cases of it.)
    fn pair() -> impl Strategy<Value = (String, String)> {
        proptest::collection::vec((0..FAMILIES.len(), any::<usize>(), any::<usize>()), 0..4)
            .prop_map(|places| {
                let pick = |at: usize, n: usize| FAMILIES[at][n % FAMILIES[at].len()];
                (
                    places.iter().map(|(at, a, _)| pick(*at, *a)).collect(),
                    places.iter().map(|(at, _, b)| pick(*at, *b)).collect(),
                )
            })
    }

    fn holds(op: Op, pattern: String, text: &str) -> bool {
        text_test("name", op, &Value::Text(pattern))
            .unwrap()
            .holds(text)
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 4000, failure_persistence: None, ..ProptestConfig::default() })]

        /// A count tells two spellings apart by `folded`, and routes a
        /// bucket by any-case `=`: the two must be one relation, or a
        /// bucket's term returns what another bucket counted.
        #[test]
        fn two_texts_share_a_fold_exactly_when_any_case_equals_holds((a, b) in pair()) {
            prop_assert_eq!(folded(&a) == folded(&b), holds(Op::Eq, a.clone(), &b), "{:?} {:?}", a, b);
        }

        /// The pattern a spelling's term carries selects that spelling and
        /// no other, whatever a pattern would make of its characters.
        #[test]
        fn an_exact_pattern_selects_its_text_alone((a, b) in pair()) {
            prop_assert_eq!(holds(Op::Match, exact_pattern(&a), &b), a == b, "{:?} {:?}", a, b);
        }
    }
}
