//! The counts view (C95, C105) and the vocabulary read (C97): the matching
//! items of an answer grouped by a key, one table for each key or one
//! crossed table of two, with one summed value beside each count.
//!
//! The shapes are `search/DESIGN.md`'s reference (*The request at a
//! terminal*, the worked example); the rules that decide a count are this
//! doc's (C95, C105; taken from the contract detail 2026-09-23). The
//! totals table a sum may name is `totals.rs` (C94).
//!
//! # As built
//!
//! - **A bucket is a term.** Its count is how many of the answer's matches
//!   that term matches, and its route is the query and that term — a
//!   request under the query, over the scope (invariant 4). A value's is
//!   `<key>=<value>`, `none` is `-has:<key>` and `undecided` is
//!   `undecided(<key>)` (C105). Which of the three an item falls in is the
//!   evaluator's own outcome of `has:<key>` on it, and what the value is
//!   comes from the accessor its term reads, so a bucket and its route have
//!   one maker (the build plan, rule 10) and cannot disagree. Counted are
//!   the matches: an item undecided at the root is no match, and is in no
//!   table.
//! - **A key is a field `has:` can be asked of, or `line`.** Every field
//!   has at most one value on an item, so its buckets sum exactly to the
//!   total (C105's second invariant); `line` puts an item in a bucket for
//!   each template it carries, and only the first invariant holds.
//! - **`tab` is counted by the tab, never by its name.** An item in a
//!   substash is in its tab's bucket. A name is no identity — two leagues
//!   each have a `Dump` — and `tab=` tests a substash's name beside its
//!   tab's, so a name's term would return what other buckets counted. Nor
//!   is an id alone: a tab is its full coordinate, realm, league and id
//!   (C54, C58), and the store has held one id under two realms. The
//!   bucket is that coordinate, its route `id:<id> league=<league>` over
//!   its realm, and its label carries all three (outside audit, 2026-09-22).
//! - **`=` is any-case, so a value spelled two ways is two buckets with two
//!   terms.** Where the corpus holds another spelling of a value the term
//!   is a pattern that turns case back on, `name~"(?-i)^…$"`, which selects
//!   that spelling alone (owner, 2026-09-20, on templates; one rule for
//!   every text). Spellings are told apart by the matcher's own fold
//!   (`bind::folded`), over the whole held corpus and not only the matches,
//!   so a term pasted into another query still selects its row. A closed
//!   set is counted by its legal value, which its `=` matches in any case.
//! - **A value outside a closed list has a count and no route**, and says
//!   what it needs: a rarity, a source or a flag GGG adds is derived and
//!   counted the day it appears and cannot be asked for until the list
//!   gains it (the build plan, rule 5; the reference, *Item-level*).
//! - **A table is bounded** (invariant 5): its values ranked by how many
//!   items carry each, then by the value, the first `limit` listed and the
//!   rest counted — in values, the unit cut. `none` and `undecided` are
//!   never cut. The order follows the data and never the query (the build
//!   plan, rule 9); a larger limit returns the rest.
//! - **The vocabulary** is the key `line`: a row for each template the
//!   matching items carry — by realm under an all-realms scope, where the
//!   row's route is scoped to its realm (C96, C97) — with the term that
//!   selects it, the range of each of its numbers over the occurrences
//!   counted, and beneath it how many items carry it from each source and
//!   under each flag, each with its own term. `line:text` and
//!   `line~pattern` narrow it as `template:` and `template~` would. A
//!   row's count is its term's matched count: an item counted under a
//!   template it could be read to carry. Its `undecided` bucket is
//!   `undecided(line( … ))` of the narrowing — no line read that it selects
//!   and a part unread that could hold one — so an item with a line read
//!   *and* an array unread is in its rows and not in `undecided`: the
//!   language has no term for "its lines could not all be read" (C105;
//!   E3 accepted, `SEARCH-SLICE.md`). A source or flag beneath a row is counted by
//!   its legal spelling, which the evaluator matches in any case (B2), so
//!   the kind's route returns what it counted; a spelling outside the list
//!   is counted and has no route.
//! - **The sum** (C95) adds one number of each item of a bucket, exactly
//!   (`exact.rs`): an item lacking the thing adds nothing and is counted as
//!   lacking; one whose number could not be read adds what was readable of
//!   it and marks the subtotal incomplete, never a total; nothing sums to
//!   zero.
//! - **Beneath `undecided`, a tally by what was unread** — a diagnostic,
//!   never a partition: an item unread twice counts twice (C105). A
//!   crossed table gains at most one `none` and one `undecided` row or
//!   column per key, and the tally once per table; a part that left both
//!   keys open is one part.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::Serialize;
use serde_json::Value as Json;

use crate::answer::{Count, Router};
use crate::bind::{self, Atom, FieldDef, Key, LINE_FLAGS, SOURCES, SortKey, Thing};
use crate::corpus::{Corpus, Held, Realm};
use crate::error::{ErrorKind, LanguageError};
use crate::eval::{self, Outcome, Scalar};
use crate::exact::Exact;
use crate::group::{self, Group, LineKind};
use crate::tree::{Node, Number, Op, Probe, Value, ValueRef};
use crate::{parse, print};

// ---- the view, bound ----------------------------------------------------------------------------

pub(crate) struct BoundCounts {
    keys: Vec<BoundKey>,
    sum: Option<(String, SortKey)>,
    limit: usize,
}

struct BoundKey {
    text: String,
    kind: BoundKind,
}

enum BoundKind {
    Field(&'static FieldDef),
    /// The narrowing as a term, and that term bound.
    Line(Box<(Node, Group)>),
}

/// Bind a counts view: its keys, and what it sums. `crossed` is one table
/// of exactly two fields.
pub(crate) fn bind(
    keys: &[String],
    crossed: bool,
    sum: Option<&str>,
    limit: Option<usize>,
) -> Result<BoundCounts, LanguageError> {
    let view_error = |message: String| LanguageError::new(ErrorKind::View, message);
    if keys.is_empty() {
        return Err(view_error(
            "a count takes a key: a field, or `line` for the vocabulary".to_string(),
        ));
    }
    let mut bound: Vec<BoundKey> = Vec::new();
    for key in keys {
        let (text, kind) = match bind::bind_key(key)? {
            Key::Field(def) => (def.name.to_string(), BoundKind::Field(def)),
            Key::Line(narrow) => {
                let text = match &narrow {
                    None => "line".to_string(),
                    Some((Op::Match, pattern)) => format!("line~{pattern}"),
                    Some((_, words)) => format!("line:{words}"),
                };
                let (term, group) = group::narrowing(narrow.as_ref())?;
                (text, BoundKind::Line(Box::new((term, group))))
            }
        };
        if bound.iter().any(|b| b.text == text) {
            return Err(view_error(format!("`{text}` is named twice")));
        }
        bound.push(BoundKey { text, kind });
    }
    if crossed {
        if bound.len() != 2 {
            return Err(view_error(format!(
                "a crossed table is one key against another: two keys, and this names {}",
                bound.len()
            )));
        }
        if let Some(line) = bound.iter().find(|b| matches!(b.kind, BoundKind::Line(_))) {
            return Err(view_error(format!(
                "`{}` is counted in a table of its own: a crossed table takes two fields",
                line.text
            )));
        }
    }
    let sum = match sum {
        Some(text) => {
            let value = parse::parse_value(text)?;
            Some((print::print_value(&value), bind::bind_sum(&value)?))
        }
        None => None,
    };
    Ok(BoundCounts {
        keys: bound,
        sum,
        limit: limit.unwrap_or(crate::answer::DEFAULT_LIMIT),
    })
}

// ---- what an answer carries ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct CountsOut {
    pub limit: usize,
    /// What is summed, and its sum over every match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<SumOf>,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CrossOut {
    pub keys: Vec<String>,
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<SumOf>,
    /// How many cells hold an item.
    pub cells_in_all: usize,
    /// The cells that hold one, ranked; a cell is a bucket of each key.
    pub cells: Vec<Cell>,
    pub left_out: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_out_needs: Option<&'static str>,
    /// Each key's `none` and `undecided`, which the cut never hides.
    pub margins: Vec<Margin>,
    /// Beneath `undecided`, by what was unread: once for the table.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tally: Vec<Tallied>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SumOf {
    pub name: String,
    #[serde(flatten)]
    pub total: Summed,
}

/// One summed value over a bucket's items (C95).
#[derive(Debug, Clone, Serialize)]
pub struct Summed {
    pub value: Json,
    /// Items that lack the thing: they add nothing.
    pub lacking: usize,
    /// A subtotal, never a total: an item's number could not be read.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub incomplete: bool,
    #[serde(skip_serializing_if = "is_zero")]
    pub unread: usize,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Debug, Clone, Serialize)]
pub struct Table {
    pub key: String,
    /// How many values the matching items carry.
    pub values: usize,
    /// The values, ranked and cut; then `none`, then `undecided`, each only
    /// when it holds an item.
    pub buckets: Vec<Bucket>,
    /// Values past the limit.
    pub left_out: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_out_needs: Option<&'static str>,
}

/// What a bucket is of: a value, `none` or `undecided` — a word of its own,
/// since a tab may be named `none`.
#[derive(Debug, Clone, Serialize)]
pub struct Label {
    /// `value`, `none` or `undecided`.
    pub bucket: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Json>,
    /// A tab's id and league: its name is no identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub league: Option<String>,
    /// A vocabulary row's realm, under an all-realms scope (C97).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<String>,
    /// The term that selects it, ready to paste; none where the language
    /// cannot say it yet, and `needs` says why.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Bucket {
    #[serde(flatten)]
    pub label: Label,
    #[serde(flatten)]
    pub count: Count,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<Summed>,
    /// A vocabulary row's numbers, over the occurrences counted.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<SlotRange>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<OfKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<OfKind>,
    /// Beneath `undecided`, by what was unread.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tally: Vec<Tallied>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SlotRange {
    pub slot: String,
    pub min: Option<Json>,
    pub max: Option<Json>,
    /// An occurrence's number could not be read: the range is of the rest.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub incomplete: bool,
}

/// How many items carry a template from one source, or under one flag.
#[derive(Debug, Clone, Serialize)]
pub struct OfKind {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
    #[serde(flatten)]
    pub count: Count,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tallied {
    pub unread: String,
    pub items: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Cell {
    pub of: Vec<Label>,
    #[serde(flatten)]
    pub count: Count,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum: Option<Summed>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Margin {
    pub key: String,
    pub none: Count,
    pub undecided: Count,
}

// ---- piles ------------------------------------------------------------------------------------------

/// A bucket's items, and what they add to the sum.
#[derive(Debug, Clone, Default)]
struct Pile {
    items: usize,
    /// In units, added again exactly (`exact.rs`).
    total: Exact,
    lacking: usize,
    unread: usize,
}

impl Pile {
    fn add(&mut self, scalar: Option<Scalar>) {
        self.items += 1;
        match scalar {
            None => {}
            Some(Scalar::Value(n)) => self.total = Exact::sum([self.total, n]),
            Some(Scalar::None) => self.lacking += 1,
            Some(Scalar::Incomplete(readable)) => {
                self.unread += 1;
                if let Some(n) = readable {
                    self.total = Exact::sum([self.total, n]);
                }
            }
        }
    }

    fn summed(&self) -> Summed {
        Summed {
            value: eval::number_json(self.total.as_f64()),
            lacking: self.lacking,
            incomplete: self.unread > 0,
            unread: self.unread,
        }
    }
}

fn tallied(tally: BTreeMap<String, usize>) -> Vec<Tallied> {
    tally
        .into_iter()
        .map(|(unread, items)| Tallied { unread, items })
        .collect()
}

/// The folds more than one spelling shares, of the values `of` reads over
/// the whole held corpus (the module doc).
fn twins<'a>(corpus: &'a Corpus, of: impl Fn(&'a Held) -> Vec<&'a str>) -> HashSet<String> {
    let spellings: HashSet<&str> = corpus.items.iter().flat_map(of).collect();
    let mut folds: HashMap<String, usize> = HashMap::new();
    for spelling in spellings {
        *folds.entry(bind::folded(spelling)).or_default() += 1;
    }
    folds
        .into_iter()
        .filter(|(_, spellings)| *spellings > 1)
        .map(|(fold, _)| fold)
        .collect()
}

// ---- a field's buckets ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Of {
    Text(String),
    /// A value outside its closed list.
    Unlisted(String),
    Number(i64),
    /// A tab, by its full coordinate (C54).
    Tab(TabAt),
    None,
    Undecided,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TabAt {
    realm: String,
    league: Option<String>,
    id: String,
}

/// Where a tab's bucket gets its name: the first item met in it.
type Tabs = HashMap<TabAt, Option<String>>;

/// What selects a bucket: its terms, under the query, over one realm where
/// the bucket is one realm's.
struct Selects {
    terms: Vec<Node>,
    realm: Option<String>,
}

fn bucket_of(def: &'static FieldDef, held: &Held, tabs: &mut Tabs) -> Of {
    match eval::outcome(&Atom::Has(def.thing), held, &[]) {
        Outcome::Undecided => return Of::Undecided,
        Outcome::Failed | Outcome::Lacked => return Of::None,
        Outcome::Matched => {}
    }
    if def.thing == Thing::Tab {
        let place = &held.place;
        let (id, name) = match &place.parent {
            Some(tab) => (&tab.id, &tab.name),
            None => (&place.id, &place.name),
        };
        let at = TabAt {
            realm: place.realm.clone(),
            league: place.league.clone(),
            id: id.clone(),
        };
        tabs.entry(at.clone()).or_insert_with(|| name.clone());
        return Of::Tab(at);
    }
    if let Some(n) = eval::number(held, def.thing) {
        return Of::Number(n as i64);
    }
    match (eval::texts(held, def.thing).first(), def.kind) {
        (None, _) => Of::None,
        (Some(value), bind::Kind::Closed(list)) => match bind::legal(list(), value) {
            Some(legal) => Of::Text(legal.to_string()),
            None => Of::Unlisted(value.to_string()),
        },
        (Some(value), _) => Of::Text(value.to_string()),
    }
}

/// A bucket's label and what selects it — none where the language cannot
/// say it, and the label says why.
fn label_of(
    def: &'static FieldDef,
    of: &Of,
    twins: &HashSet<String>,
    tabs: &Tabs,
) -> (Label, Option<Selects>) {
    let test = |field: &str, op: Op, value: Value| Node::Test {
        field: field.to_string(),
        op,
        value,
    };
    let field = ValueRef::Field(def.name.to_string());
    let (bucket, value, term): (&'static str, Option<Json>, Option<Node>) = match of {
        Of::None => (
            "none",
            None,
            Some(Node::Not(Box::new(Node::Has(def.name.to_string())))),
        ),
        Of::Undecided => (
            "undecided",
            None,
            Some(Node::Undecided(Probe::Thing(field))),
        ),
        Of::Number(n) => (
            "value",
            Some(Json::from(*n)),
            Some(test(def.name, Op::Eq, Value::Number(Number::Int(*n)))),
        ),
        Of::Tab(at) => (
            "value",
            tabs.get(at).cloned().flatten().map(Json::from),
            Some(test("id", Op::Contains, Value::Text(at.id.clone()))),
        ),
        Of::Unlisted(value) => ("value", Some(Json::from(value.as_str())), None),
        Of::Text(value) => {
            let term = if twins.contains(&bind::folded(value)) {
                test(def.name, Op::Match, Value::Text(bind::exact_pattern(value)))
            } else {
                test(def.name, Op::Eq, Value::Text(value.clone()))
            };
            ("value", Some(Json::from(value.as_str())), Some(term))
        }
    };
    // a tab is its coordinate: the id, and the league the tab is listed
    // under, over its realm
    let (tab, selects) = match (of, term) {
        (Of::Tab(at), Some(term)) => {
            let mut terms = vec![term];
            if let Some(league) = &at.league {
                terms.push(test("league", Op::Eq, Value::Text(league.clone())));
            }
            (
                Some(at.clone()),
                Some(Selects {
                    terms,
                    realm: Some(at.realm.clone()),
                }),
            )
        }
        (_, term) => (
            None,
            term.map(|term| Selects {
                terms: vec![term],
                realm: None,
            }),
        ),
    };
    let label = Label {
        bucket,
        value,
        id: tab.as_ref().map(|at| at.id.clone()),
        league: tab.as_ref().and_then(|at| at.league.clone()),
        realm: tab.map(|at| at.realm),
        term: selects.as_ref().map(|s| match s.terms.as_slice() {
            [one] => print::print(one),
            many => print::print(&Node::All(many.to_vec())),
        }),
        needs: selects.is_none().then(|| {
            format!(
                "a value outside the closed list of `{}`: it is counted, and cannot be asked for until the list gains it",
                def.name
            )
        }),
    };
    (label, selects)
}

/// How a bucket is ordered among those that hold as many items: a value
/// before `none` before `undecided`; a number by its size, a text by its
/// spelling, a tab by its name, then its league, realm and id. The order
/// follows the data, never the query (the build plan, rule 9).
fn spelled(of: &Of, tabs: &Tabs) -> (u8, i64, String, String) {
    match of {
        Of::Number(n) => (0, *n, String::new(), String::new()),
        Of::Text(v) | Of::Unlisted(v) => (0, 0, v.clone(), String::new()),
        Of::Tab(at) => (
            0,
            0,
            tabs.get(at).cloned().flatten().unwrap_or_default(),
            format!(
                "{} {} {}",
                at.league.as_deref().unwrap_or_default(),
                at.realm,
                at.id
            ),
        ),
        Of::None => (1, 0, String::new(), String::new()),
        Of::Undecided => (2, 0, String::new(), String::new()),
    }
}

fn field_twins(corpus: &Corpus, def: &'static FieldDef) -> HashSet<String> {
    match def.kind {
        bind::Kind::Text if def.thing != Thing::Tab => {
            twins(corpus, |held| eval::texts(held, def.thing))
        }
        _ => HashSet::new(),
    }
}

// ---- answering ----------------------------------------------------------------------------------------

/// A count under the query, by what selects it; with nothing the language
/// can say, a count and no route.
fn routed(matches: &Matches<'_>, n: usize, selects: Option<Selects>) -> Count {
    match selects {
        Some(s) => matches.router.under(n, s.terms, s.realm.as_deref()),
        None => Count {
            count: n,
            route: None,
        },
    }
}

/// The matches an answer counted, and what each adds to the sum.
pub(crate) struct Matches<'a> {
    pub corpus: &'a Corpus,
    /// The matching items, by their place in the corpus.
    pub at: &'a [usize],
    pub router: &'a Router<'a>,
}

impl Matches<'_> {
    fn held(&self) -> impl Iterator<Item = &Held> {
        self.at.iter().map(|at| &self.corpus.items[*at])
    }

    fn scalars(&self, sum: &Option<(String, SortKey)>) -> Option<Vec<Scalar>> {
        sum.as_ref()
            .map(|(_, key)| self.held().map(|held| eval::scalar(key, held)).collect())
    }

    fn sum_of(
        &self,
        sum: &Option<(String, SortKey)>,
        scalars: &Option<Vec<Scalar>>,
    ) -> Option<SumOf> {
        let ((name, _), scalars) = sum.as_ref().zip(scalars.as_ref())?;
        let mut all = Pile::default();
        scalars.iter().for_each(|s| all.add(Some(*s)));
        Some(SumOf {
            name: name.clone(),
            total: all.summed(),
        })
    }
}

pub(crate) fn tables(bound: &BoundCounts, matches: &Matches<'_>) -> CountsOut {
    let scalars = matches.scalars(&bound.sum);
    CountsOut {
        limit: bound.limit,
        sum: matches.sum_of(&bound.sum, &scalars),
        tables: bound
            .keys
            .iter()
            .map(|key| match &key.kind {
                BoundKind::Field(def) => {
                    field_table(&key.text, def, bound.limit, matches, &scalars)
                }
                BoundKind::Line(line) => {
                    let (term, group) = line.as_ref();
                    vocabulary(&key.text, term, group, bound.limit, matches, &scalars)
                }
            })
            .collect(),
    }
}

fn field_table(
    key: &str,
    def: &'static FieldDef,
    limit: usize,
    matches: &Matches<'_>,
    scalars: &Option<Vec<Scalar>>,
) -> Table {
    let mut piles: HashMap<Of, Pile> = HashMap::new();
    let mut tabs = Tabs::new();
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for (m, held) in matches.held().enumerate() {
        let of = bucket_of(def, held, &mut tabs);
        if of == Of::Undecided {
            for kind in eval::unread_kinds(&Atom::Has(def.thing), held) {
                *tally.entry(kind).or_default() += 1;
            }
        }
        piles
            .entry(of)
            .or_default()
            .add(scalars.as_ref().map(|s| s[m]));
    }
    let twins = field_twins(matches.corpus, def);
    let bucket = |of: &Of, pile: &Pile, tally: Vec<Tallied>| {
        let (label, selects) = label_of(def, of, &twins, &tabs);
        Bucket {
            label,
            count: routed(matches, pile.items, selects),
            sum: scalars.as_ref().map(|_| pile.summed()),
            slots: Vec::new(),
            sources: Vec::new(),
            flags: Vec::new(),
            tally,
        }
    };
    let none = piles.remove(&Of::None);
    let undecided = piles.remove(&Of::Undecided);
    let mut values: Vec<(Of, Pile)> = piles.into_iter().collect();
    values.sort_by(|(a, x), (b, y)| {
        y.items
            .cmp(&x.items)
            .then_with(|| spelled(a, &tabs).cmp(&spelled(b, &tabs)))
    });
    let in_all = values.len();
    let mut buckets: Vec<Bucket> = values
        .iter()
        .take(limit)
        .map(|(of, pile)| bucket(of, pile, Vec::new()))
        .collect();
    buckets.extend(none.map(|pile| bucket(&Of::None, &pile, Vec::new())));
    buckets.extend(undecided.map(|pile| bucket(&Of::Undecided, &pile, tallied(tally))));
    let left_out = in_all.saturating_sub(limit);
    Table {
        key: key.to_string(),
        values: in_all,
        buckets,
        left_out,
        left_out_needs: (left_out > 0).then_some("--limit"),
    }
}

pub(crate) fn crossed(bound: &BoundCounts, matches: &Matches<'_>) -> CrossOut {
    let defs: Vec<&'static FieldDef> = bound
        .keys
        .iter()
        .filter_map(|key| match &key.kind {
            BoundKind::Field(def) => Some(*def),
            BoundKind::Line(_) => None,
        })
        .collect();
    let scalars = matches.scalars(&bound.sum);
    let mut cells: HashMap<Vec<Of>, Pile> = HashMap::new();
    let mut tabs = Tabs::new();
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    let mut margins: Vec<[usize; 2]> = vec![[0, 0]; defs.len()];
    for (m, held) in matches.held().enumerate() {
        let of: Vec<Of> = defs
            .iter()
            .map(|def| bucket_of(def, held, &mut tabs))
            .collect();
        let mut kinds: Vec<String> = Vec::new();
        for (k, (def, of)) in defs.iter().zip(&of).enumerate() {
            match of {
                Of::None => margins[k][0] += 1,
                Of::Undecided => {
                    margins[k][1] += 1;
                    kinds.extend(eval::unread_kinds(&Atom::Has(def.thing), held));
                }
                _ => {}
            }
        }
        // once for the table: a part that left both keys open is one part
        kinds.sort();
        kinds.dedup();
        for kind in kinds {
            *tally.entry(kind).or_default() += 1;
        }
        cells
            .entry(of)
            .or_default()
            .add(scalars.as_ref().map(|s| s[m]));
    }
    let twins: Vec<HashSet<String>> = defs
        .iter()
        .map(|def| field_twins(matches.corpus, def))
        .collect();
    let mut ranked_cells: Vec<(Vec<Of>, Pile)> = cells.into_iter().collect();
    // most carried first, then each key's value in turn
    ranked_cells.sort_by(|(a, x), (b, y)| {
        let spelled = |of: &Vec<Of>| of.iter().map(|of| spelled(of, &tabs)).collect::<Vec<_>>();
        y.items
            .cmp(&x.items)
            .then_with(|| spelled(a).cmp(&spelled(b)))
    });
    let in_all = ranked_cells.len();
    let left_out = in_all.saturating_sub(bound.limit);
    let cells = ranked_cells
        .iter()
        .take(bound.limit)
        .map(|(of, pile)| {
            let (labels, selects): (Vec<Label>, Vec<Option<Selects>>) = of
                .iter()
                .enumerate()
                .map(|(k, of)| label_of(defs[k], of, &twins[k], &tabs))
                .unzip();
            // a cell's route carries both keys, and is none where one cannot
            // be said; a tab's names its realm, and no cross holds two tabs
            let selects = selects
                .into_iter()
                .collect::<Option<Vec<Selects>>>()
                .map(|each| Selects {
                    realm: each.iter().find_map(|s| s.realm.clone()),
                    terms: each.into_iter().flat_map(|s| s.terms).collect(),
                });
            Cell {
                of: labels,
                count: routed(matches, pile.items, selects),
                sum: scalars.as_ref().map(|_| pile.summed()),
            }
        })
        .collect();
    let empty = HashSet::new();
    CrossOut {
        keys: bound.keys.iter().map(|key| key.text.clone()).collect(),
        limit: bound.limit,
        sum: matches.sum_of(&bound.sum, &scalars),
        cells_in_all: in_all,
        cells,
        left_out,
        left_out_needs: (left_out > 0).then_some("--limit"),
        margins: defs
            .iter()
            .zip(&margins)
            .map(|(def, [none, undecided])| {
                let route = |n: usize, of: Of| {
                    let (_, selects) = label_of(def, &of, &empty, &tabs);
                    routed(matches, n, selects)
                };
                Margin {
                    key: def.name.to_string(),
                    none: route(*none, Of::None),
                    undecided: route(*undecided, Of::Undecided),
                }
            })
            .collect(),
        tally: tallied(tally),
    }
}

// ---- the vocabulary -------------------------------------------------------------------------------

#[derive(Default)]
struct Row {
    pile: Pile,
    sources: BTreeMap<String, usize>,
    flags: BTreeMap<String, usize>,
    /// Each number's smallest and largest, and whether one went unread.
    slots: Vec<(Option<f64>, Option<f64>, bool)>,
}

/// The words a template's numbers go by, in order: `low` and `high` for a
/// ranged pair, `arg<N>` for the rest (the reference, *Slots*).
fn slot_names(template: &str) -> Vec<String> {
    let slots = crate::template::slots(template);
    (1..=slots.count)
        .map(|n| match slots.ranged {
            Some((low, _)) if n == low => "low".to_string(),
            Some((_, high)) if n == high => "high".to_string(),
            _ => format!("arg{n}"),
        })
        .collect()
}

fn vocabulary(
    key: &str,
    narrowing: &Node,
    group: &Group,
    limit: usize,
    matches: &Matches<'_>,
    scalars: &Option<Vec<Scalar>>,
) -> Table {
    let atom = &Atom::Lines(group.clone());
    let mut rows: HashMap<(String, String), Row> = HashMap::new();
    let (mut none, mut undecided) = (Pile::default(), Pile::default());
    let mut tally: BTreeMap<String, usize> = BTreeMap::new();
    for (m, held) in matches.held().enumerate() {
        let scalar = scalars.as_ref().map(|s| s[m]);
        match eval::outcome(atom, held, &[]) {
            Outcome::Undecided => {
                for kind in eval::unread_kinds(atom, held) {
                    *tally.entry(kind).or_default() += 1;
                }
                undecided.add(scalar);
            }
            Outcome::Failed | Outcome::Lacked => none.add(scalar),
            Outcome::Matched => {
                // an item is counted once under a template however many
                // occurrences of it it carries
                let mut seen: HashSet<(&str, Option<(bool, String)>)> = HashSet::new();
                for line in eval::selected(held, group) {
                    let row = rows
                        .entry((held.place.realm.clone(), line.template.clone()))
                        .or_default();
                    if seen.insert((&line.template, None)) {
                        row.pile.add(scalar);
                    }
                    // by the legal spelling, which `=` and `is:` match in any
                    // case; a spelling outside the list stands as it is
                    let spelled = |list: &'static [&'static str], word: &str| {
                        bind::legal(list, word).map_or_else(|| word.to_string(), str::to_string)
                    };
                    let source = spelled(SOURCES, &line.source);
                    if seen.insert((&line.template, Some((true, source.clone())))) {
                        *row.sources.entry(source).or_default() += 1;
                    }
                    for flag in &line.flags {
                        let flag = spelled(LINE_FLAGS, flag);
                        if seen.insert((&line.template, Some((false, flag.clone())))) {
                            *row.flags.entry(flag).or_default() += 1;
                        }
                    }
                    // a line names its numbers only while they are its
                    // template's `#`s (`Line::slot`)
                    if line.numbers.len() == crate::template::slots(&line.template).count {
                        row.slots.resize(line.numbers.len(), (None, None, false));
                        for (range, number) in row.slots.iter_mut().zip(&line.numbers) {
                            match number {
                                Some(n) => {
                                    range.0 = Some(range.0.map_or(*n, |min| min.min(*n)));
                                    range.1 = Some(range.1.map_or(*n, |max| max.max(*n)));
                                }
                                None => range.2 = true,
                            }
                        }
                    }
                }
            }
        }
    }
    let twins = twins(matches.corpus, |held| {
        held.item
            .lines
            .iter()
            .map(|l| l.template.as_str())
            .collect()
    });
    // by realm under an all-realms scope: a template is another line in
    // another game (C90), and its row's route is scoped to its realm (C97)
    let by_realm = matches.corpus.realm == Realm::All;
    let mut ranked: Vec<((String, String), Row)> = rows.into_iter().collect();
    ranked.sort_by(|((ra, ta), a), ((rb, tb), b)| {
        b.pile
            .items
            .cmp(&a.pile.items)
            .then_with(|| ta.cmp(tb))
            .then_with(|| ra.cmp(rb))
    });
    let in_all = ranked.len();
    let mut buckets: Vec<Bucket> = ranked
        .iter()
        .take(limit)
        .map(|((realm, template), row)| {
            let twin = twins.contains(&bind::folded(template));
            let realm = by_realm.then_some(realm.as_str());
            let kinds = |counted: &BTreeMap<String, usize>, source: bool| -> Vec<OfKind> {
                let mut kinds: Vec<OfKind> = counted
                    .iter()
                    .map(|(kind, items)| {
                        let listed = bind::legal(if source { SOURCES } else { LINE_FLAGS }, kind);
                        let term = listed.map(|kind| {
                            group::line_exactly(
                                template,
                                twin,
                                if source {
                                    LineKind::Source(kind)
                                } else {
                                    LineKind::Flag(kind)
                                },
                            )
                        });
                        OfKind {
                            kind: kind.clone(),
                            term: term.as_ref().map(print::print),
                            needs: term.is_none().then(|| {
                                format!(
                                    "outside the closed list of a line's {}: it is counted, and cannot be asked for until the list gains it",
                                    if source { "sources" } else { "flags" }
                                )
                            }),
                            count: routed(
                                matches,
                                *items,
                                term.map(|term| Selects {
                                    terms: vec![term],
                                    realm: realm.map(str::to_string),
                                }),
                            ),
                        }
                    })
                    .collect();
                kinds.sort_by(|a, b| {
                    b.count
                        .count
                        .cmp(&a.count.count)
                        .then_with(|| a.kind.cmp(&b.kind))
                });
                kinds
            };
            let term = group::line_exactly(template, twin, LineKind::Any);
            Bucket {
                label: Label {
                    bucket: "value",
                    value: Some(Json::from(template.as_str())),
                    id: None,
                    league: None,
                    realm: realm.map(str::to_string),
                    term: Some(print::print(&term)),
                    needs: None,
                },
                count: matches.router.under(row.pile.items, vec![term], realm),
                sum: scalars.as_ref().map(|_| row.pile.summed()),
                slots: slot_names(template)
                    .into_iter()
                    .zip(&row.slots)
                    .map(|(slot, (min, max, incomplete))| SlotRange {
                        slot,
                        min: min.map(eval::number_json),
                        max: max.map(eval::number_json),
                        incomplete: *incomplete,
                    })
                    .collect(),
                sources: kinds(&row.sources, true),
                flags: kinds(&row.flags, false),
                tally: Vec::new(),
            }
        })
        .collect();
    let apart = |bucket: &'static str, term: Node, pile: &Pile, tally: Vec<Tallied>| Bucket {
        label: Label {
            bucket,
            value: None,
            id: None,
            league: None,
            realm: None,
            term: Some(print::print(&term)),
            needs: None,
        },
        count: matches.router.under(pile.items, vec![term], None),
        sum: scalars.as_ref().map(|_| pile.summed()),
        slots: Vec::new(),
        sources: Vec::new(),
        flags: Vec::new(),
        tally,
    };
    if none.items > 0 {
        let term = Node::Not(Box::new(narrowing.clone()));
        buckets.push(apart("none", term, &none, Vec::new()));
    }
    if undecided.items > 0 {
        let term = Node::Undecided(Probe::Term(Box::new(narrowing.clone())));
        buckets.push(apart("undecided", term, &undecided, tallied(tally)));
    }
    let left_out = in_all.saturating_sub(limit);
    Table {
        key: key.to_string(),
        values: in_all,
        buckets,
        left_out,
        left_out_needs: (left_out > 0).then_some("--limit"),
    }
}
