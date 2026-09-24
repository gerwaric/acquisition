//! The request and the answer (C100, C96, C98): a query bound to a scope
//! and a view in, one answer shape out — the canonical query, the scope,
//! the basis, the terms block, the total, the rows, and how to go on.
//!
//! The shapes are `search/DESIGN.md`'s reference (*Two values*, *One
//! worked example*, *Invariants of the surface*), cited and not restated
//! (the build plan, rule 6). This is the crate's boundary: tests pin a
//! request in and an answer out, as JSON, and a frontend's text is a
//! function of that JSON (C53).
//!
//! # As built
//!
//! - **Every term is counted over the whole scope** — matched, failed,
//!   lacked, undecided — before composition and before the limit, with no
//!   short circuit (C93). A count that is not zero carries its **route**:
//!   the request that returns exactly its members, with the basis it was
//!   counted at and its denominator, the scope (invariant 4). A route is
//!   a request and never a fragment: a term's failed members are asked of
//!   the scope, not appended to the query that counted them.
//! - **The routes of a term**: matched is the term; undecided is
//!   `undecided(term)`; lacked is `-has:<field>`, or `-line(<selector>)`;
//!   failed is `has:<field> -term`, or `line(<selector>) -term` — with
//!   `or undecided(line(<selector>))` where the selector asks a flag, which
//!   an occurrence with unread flags leaves open; a term
//!   that cannot lack fails by `-term`. The together count's is the failed
//!   route and the selector's sum against the bound.
//! - **A `:` or `~` selector is printed as authored with what it resolved
//!   to beside it** (invariant 2): the values its matched items carry,
//!   or — for a line's group, a `sum`'s too — the templates its selector
//!   picks over the scope, whatever its comparisons then make of them; the
//!   ten most carried listed and the rest counted. A quoted template
//!   resolves to itself and lists nothing, unless it found two spellings
//!   of one line, which any-case `=` can, and then both are listed. The
//!   route to the rest is a count: the term alone over the scope — never
//!   the query, whose other terms would drop values the selector resolved
//!   to — counted by the field, or the vocabulary narrowed by the group's
//!   one template test (`counts.rs`); a `tab` selector has none, since a
//!   tab is counted by the tab and it resolved to names (outside audit,
//!   2026-09-22).
//! - **What a row shows of one term is bounded** and says how many it left
//!   out; the item whole is `show <id>` (invariant 5). Of lines and
//!   strings, six, a sum's value beside them. Of why an item is undecided,
//!   six of the item's unread parts in the item's order, each with every
//!   term that rests on it — so a block may hold more entries than six,
//!   and what it leaves out is counted in parts, never in entries
//!   (`eval::why`).
//! - **Rows** are the matching items in the store's stable order, or by
//!   the sort scalar with items that have none last either way; past the
//!   limit they are counted, and the way on is a larger limit until
//!   `--next` is built. A counts view shows none: the matches are handed
//!   to `counts.rs` instead, with the one maker of a route (`Router`,
//!   private), so a bucket's route is made as a term's is.
//! - **A zero total** prints, in place of rows: the selectors that
//!   resolved to nothing in this scope — a group's bound selector picked
//!   no occurrence, a field's term matched no item — each with the values
//!   sharing its words — a suggestion to type, never a match made for the author
//!   (S107: never fuzzy-matched); and the root's undecided route.
//! - **Membership is `live`**: the store's read hands over no removed item
//!   (the build plan, gap 3).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::bind::{self, Atom, Query, Term, Thing};
use crate::corpus::{Basis, Corpus, Coverage, Held, Place, Realm};
use crate::counts;
use crate::error::{LanguageError, SearchError};
use crate::eval::{self, Outcome, Scalar, Truth};
pub use crate::eval::{Evidence, Reason};
use crate::group;
use crate::tree::{Collection, Node, Op, Probe, Value, ValueRef};
use crate::{json, parse, print};

/// The register's wording for a line the search cannot name (C102, S107).
pub const S107: &str = "a line the search cannot name is shown as unknown, with what it knows of it, and never fuzzy-matched";

/// How many rows an answer returns when the request names no limit.
pub const DEFAULT_LIMIT: usize = 20;
/// How many of a bounded block's entries are listed before the rest are
/// counted (C53).
const LISTED: usize = 10;
const SUGGESTED: usize = 5;

// ---- the request ------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub query: QueryInput,
    #[serde(default)]
    pub view: View,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    /// The account's uuid or name; a receiver holding another refuses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// Resolved in every answer and every route; omitted only by an
    /// author over a store holding one realm (C96).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realm: Option<Realm>,
    #[serde(default)]
    pub membership: Membership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Membership {
    #[default]
    Live,
}

/// The query as text or as a tree; an answer returns both (C104).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryInput {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<Json>,
}

/// What an answer shows of its matches: rows, or counts of them — one
/// table for each key, or one crossed table of two (C95). One of the
/// three, as the reference's synopsis has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    Rows(Rows),
    Counts(Counts),
    Cross(Counts),
}

impl Default for View {
    fn default() -> View {
        View::Rows(Rows::default())
    }
}

/// A counts view: the keys, as `counts.rs` binds them — a field, `line`,
/// `line:<text>`, `line~<pattern>` — and the one value summed beside each
/// count, a value's text as `--sort` takes one.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub keys: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sum: Option<String>,
    /// How many values a table lists, or cells a crossed one; the rest are
    /// counted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rows {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// A value's text: a number field, `line(P).<slot>`, `sum( … )`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub desc: bool,
}

impl Request {
    /// The query of a request, parsed or read, and bound.
    pub fn bind(&self) -> Result<Query, LanguageError> {
        match (&self.query.text, &self.query.tree) {
            (Some(_), Some(_)) => Err(LanguageError::new(
                crate::error::ErrorKind::Tree,
                "a request sends `text` or `tree`, never both: an answer returns both",
            )),
            (Some(text), None) => bind::parse_query(text),
            (None, Some(tree)) => bind::bind(&json::from_json(tree)?),
            (None, None) => bind::parse_query(""),
        }
    }

    /// Every authoring error the request holds — the query's, the sort's,
    /// the view's — with no store read: what an adapter asks before it
    /// opens one.
    pub fn check(&self) -> Result<(), LanguageError> {
        self.bind()?;
        self.view.bound().map(|_| ())
    }
}

/// A view with its names bound.
enum BoundView {
    Rows {
        sort: Option<(String, bind::SortKey)>,
        desc: bool,
        limit: usize,
    },
    Counts(counts::BoundCounts),
    Cross(counts::BoundCounts),
}

impl View {
    /// The view an adapter's flat arguments name: rows unless a count or a
    /// crossed table is asked for, and never two of them. A sort orders
    /// rows and a sum stands beside a count, so either with the other view
    /// is an authoring error and not a flag ignored.
    pub fn of(
        count: Option<Vec<String>>,
        cross: Option<Vec<String>>,
        sum: Option<String>,
        rows: Rows,
    ) -> Result<View, LanguageError> {
        let view_error =
            |message: &str| Err(LanguageError::new(crate::error::ErrorKind::View, message));
        let limit = rows.limit;
        let keys = match (count, cross) {
            (Some(_), Some(_)) => {
                return view_error(
                    "`--count` is a table for each key and `--cross` one table of two: an answer has one view",
                );
            }
            (None, None) if sum.is_some() => {
                return view_error(
                    "`--sum` stands beside a count: name what to count by, `--count <key>` or `--cross <key>,<key>`",
                );
            }
            (None, None) => return Ok(View::Rows(rows)),
            (Some(keys), None) => (keys, false),
            (None, Some(keys)) => (keys, true),
        };
        if rows.sort.is_some() {
            return view_error(
                "`--sort` orders rows, and a count shows none: a table is ranked by its counts",
            );
        }
        let counts = Counts {
            keys: keys.0,
            sum,
            limit,
        };
        Ok(if keys.1 {
            View::Cross(counts)
        } else {
            View::Counts(counts)
        })
    }

    fn bound(&self) -> Result<BoundView, LanguageError> {
        Ok(match self {
            View::Rows(rows) => BoundView::Rows {
                sort: match &rows.sort {
                    Some(text) => {
                        let value = parse::parse_value(text)?;
                        Some((print::print_value(&value), bind::bind_sort(&value)?))
                    }
                    None => None,
                },
                desc: rows.desc,
                limit: rows.limit.unwrap_or(DEFAULT_LIMIT),
            },
            View::Counts(c) => {
                BoundView::Counts(counts::bind(&c.keys, false, c.sum.as_deref(), c.limit)?)
            }
            View::Cross(c) => {
                BoundView::Cross(counts::bind(&c.keys, true, c.sum.as_deref(), c.limit)?)
            }
        })
    }
}

// ---- the answer ---------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    pub query: QueryOut,
    pub scope: ScopeOut,
    pub basis: Basis,
    pub terms: Vec<TermOut>,
    pub total: Total,
    pub view: ViewOut,
    pub rows: Vec<Row>,
    /// In place of rows, when the total is zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero: Option<Zero>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryOut {
    pub text: String,
    pub tree: Json,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScopeOut {
    pub account: Account,
    pub realm: Realm,
    pub membership: Membership,
    #[serde(flatten)]
    pub coverage: Coverage,
    /// A named realm this store has nothing under.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub realm_not_held: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Account {
    pub uuid: String,
    pub name: Option<String>,
}

/// A count, and — when it is not zero — the request that returns exactly
/// its members (invariant 4).
#[derive(Debug, Clone, Serialize)]
pub struct Count {
    pub count: usize,
    #[serde(flatten)]
    pub route: Option<Route>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Route {
    pub counted_at: Basis,
    /// What the count is a part of: always the scope.
    pub denominator: &'static str,
    pub request: Request,
}

#[derive(Debug, Clone, Serialize)]
pub struct TermOut {
    pub path: String,
    pub term: String,
    pub matched: Count,
    pub failed: Count,
    pub lacked: Count,
    pub undecided: Count,
    /// C92: items no occurrence of which meets the lower bound, and whose
    /// occurrences do together. Absent where it does not apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub together: Option<Together>,
    /// What a `:` or `~` selector resolved to (invariant 2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<Resolved>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Together {
    #[serde(flatten)]
    pub count: Count,
    pub slot: String,
    pub bound: Json,
}

#[derive(Debug, Clone, Serialize)]
pub struct Resolved {
    /// `template`, or the field's name.
    pub of: String,
    pub values: Vec<Carried>,
    /// Values past the listed ones.
    pub more: usize,
    /// The count that lists every value, the listed ones too: none where
    /// no count does (the module doc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest: Option<Route>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Carried {
    pub value: String,
    pub items: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Total {
    pub matched: usize,
    pub undecided: Count,
    /// The undecided items, the first ten, each with why.
    pub undecided_items: Vec<UndecidedItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UndecidedItem {
    pub id: String,
    pub name: Option<String>,
    pub why: Vec<Why>,
    /// Unread parts of the item past the ones `why` gives (invariant 5).
    #[serde(skip_serializing_if = "is_zero")]
    pub why_left_out: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Why {
    pub path: String,
    pub term: String,
    #[serde(flatten)]
    pub reason: Reason,
}

/// The view as answered: the rows' bounds, or the tables (`counts.rs`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewOut {
    Rows(RowsOut),
    Counts(counts::CountsOut),
    Cross(counts::CrossOut),
}

#[derive(Debug, Clone, Serialize)]
pub struct RowsOut {
    pub limit: usize,
    pub returned: usize,
    /// Matching items past the limit: a larger limit returns them.
    pub left_out: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_out_needs: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub desc: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub id: String,
    pub name: Option<String>,
    pub typeline: Option<String>,
    pub base: Option<String>,
    pub rarity: Option<String>,
    pub place: Place,
    pub matched: Vec<Touched>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sorted>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Touched {
    pub path: String,
    pub term: String,
    pub shows: Vec<Evidence>,
    /// What the term touched past what is shown — lines or strings, or,
    /// of reasons, unread parts of the item: `show <id>` has the item whole
    /// (invariant 5).
    #[serde(skip_serializing_if = "is_zero")]
    pub left_out: usize,
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// An item's sort scalar, or why it has none (C92).
#[derive(Debug, Clone, Serialize)]
pub struct Sorted {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Json>,
    /// `no satisfying occurrence`, or `incomplete` beside what was readable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Zero {
    /// Selectors nothing in this scope carries, with what shares their
    /// words.
    pub resolved_to_nothing: Vec<Nothing>,
    pub said: &'static str,
    pub undecided: Count,
    /// What the reference offers here and this build refuses (rule 5).
    pub not_built: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Nothing {
    pub path: String,
    pub term: String,
    pub of: String,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub value: String,
    pub items: usize,
    /// The exact term that selects it.
    pub term: String,
}

// ---- answering -----------------------------------------------------------------------------------

/// Answer a request from a held corpus. The corpus is the scope: a
/// request naming another realm or account than it holds is refused.
pub fn answer(corpus: &Corpus, request: &Request) -> Result<Answer, SearchError> {
    if let Some(account) = &request.scope.account
        && !account_is(corpus, account)
    {
        return Err(SearchError::scope(
            "account_mismatch",
            format!(
                "this request is bound to account `{account}` and the store selected is {}'s: rebinding is deliberate (--rebind, not built: step 10)",
                corpus
                    .account_name
                    .as_deref()
                    .unwrap_or(&corpus.basis.account)
            ),
        ));
    }
    if let Some(realm) = &request.scope.realm
        && *realm != corpus.realm
    {
        return Err(SearchError::scope(
            "realm_mismatch",
            format!(
                "this corpus holds `{}` and the request names `{}`",
                corpus.realm.as_str(),
                realm.as_str()
            ),
        ));
    }
    let query = request.bind()?;
    let view = request.view.bound()?;

    let router = Router {
        corpus,
        scope: Scope {
            account: Some(
                corpus
                    .account_name
                    .clone()
                    .unwrap_or_else(|| corpus.basis.account.clone()),
            ),
            realm: Some(corpus.realm.clone()),
            membership: request.scope.membership,
        },
        root: &query.root,
    };
    let count = |n: usize, node: Option<&Node>| router.count(n, node);

    // every term over every item, then the tree
    let n_terms = query.terms.len();
    let mut tallies = vec![[0usize; 4]; n_terms];
    let mut together = vec![0usize; n_terms];
    // whether a group's selector picked any occurrence in the scope: what
    // the zero block means by a selector that resolved to nothing
    let mut picked = vec![false; n_terms];
    let mut carried: Vec<Option<HashMap<String, usize>>> = query
        .terms
        .iter()
        .map(|t| resolves(t).map(|_| HashMap::new()))
        .collect();
    let mut matches: Vec<(usize, Vec<Outcome>)> = Vec::new();
    let mut undecided: Vec<(usize, Vec<Outcome>)> = Vec::new();
    let mut n_undecided = 0usize;
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(n_terms);
    for (at, held) in corpus.items.iter().enumerate() {
        outcomes.clear();
        for (i, term) in query.terms.iter().enumerate() {
            let outcome = eval::outcome(&term.atom, held, &outcomes);
            outcomes.push(outcome);
            tallies[i][outcome as usize] += 1;
            if let (Atom::Lines(group), Outcome::Failed) = (&term.atom, outcome)
                && eval::together(held, group)
            {
                together[i] += 1;
            }
            if !picked[i]
                && let Atom::Lines(group) | Atom::Sum { group, .. } = &term.atom
            {
                picked[i] = eval::selected(held, group).next().is_some();
            }
            if let Some(carried) = &mut carried[i] {
                for value in resolved_values(term, held, outcome) {
                    *carried.entry(value).or_default() += 1;
                }
            }
        }
        match eval::truth(&query.bound, &outcomes) {
            Truth::True => matches.push((at, outcomes.clone())),
            Truth::Undecided => {
                n_undecided += 1;
                if undecided.len() < LISTED {
                    undecided.push((at, outcomes.clone()));
                }
            }
            Truth::False => {}
        }
    }

    let terms: Vec<TermOut> = query
        .terms
        .iter()
        .enumerate()
        .map(|(i, term)| {
            let routes = routes(term);
            let tally = tallies[i];
            TermOut {
                path: term.path.clone(),
                term: print::print(&term.node),
                matched: count(tally[Outcome::Matched as usize], Some(&term.node)),
                failed: count(tally[Outcome::Failed as usize], routes.failed.as_ref()),
                lacked: count(tally[Outcome::Lacked as usize], routes.lacked.as_ref()),
                undecided: count(tally[Outcome::Undecided as usize], Some(&routes.undecided)),
                together: routes.together.as_ref().map(|(node, lower)| Together {
                    count: count(together[i], Some(node)),
                    slot: lower.slot.clone(),
                    bound: eval::number_json(lower.bound.as_f64()),
                }),
                resolved: carried[i]
                    .take()
                    .filter(|carried| !exact_only(term) || carried.len() > 1)
                    .zip(resolves(term))
                    .map(|(carried, of)| {
                        let mut values: Vec<Carried> = carried
                            .into_iter()
                            .map(|(value, items)| Carried { value, items })
                            .collect();
                        values.sort_by(|a, b| {
                            b.items.cmp(&a.items).then_with(|| a.value.cmp(&b.value))
                        });
                        let more = values.len().saturating_sub(LISTED);
                        values.truncate(LISTED);
                        let rest = (more > 0)
                            .then(|| rest_of(term))
                            .flatten()
                            .map(|(node, key)| router.continuation(&node, key));
                        Resolved {
                            of,
                            values,
                            more,
                            rest,
                        }
                    }),
            }
        })
        .collect();

    // the empty query has no term for `undecided( … )` to ask about, and
    // nothing on an item can leave it open
    let root_undecided = match &query.root {
        Node::All(children) if children.is_empty() => None,
        root => Some(Node::Undecided(Probe::Term(Box::new(root.clone())))),
    };
    let total = Total {
        matched: matches.len(),
        undecided: count(n_undecided, root_undecided.as_ref()),
        undecided_items: undecided
            .iter()
            .map(|(at, outcomes)| {
                let held = &corpus.items[*at];
                let mut blamed = Vec::new();
                eval::blame(&query.bound, outcomes, &mut blamed);
                let (why, why_left_out) = eval::why(&query.terms, &blamed, held);
                UndecidedItem {
                    id: held.item.facts.id.clone(),
                    name: label(held),
                    why: why
                        .into_iter()
                        .map(|(term, reason)| Why {
                            path: term.path.clone(),
                            term: print::print(&term.node),
                            reason,
                        })
                        .collect(),
                    why_left_out,
                }
            })
            .collect(),
    };

    // the view: counts of the matches, or the rows — the order, then the
    // limit
    let at: Vec<usize> = matches.iter().map(|(at, _)| *at).collect();
    let counted = counts::Matches {
        corpus,
        at: &at,
        router: &router,
    };
    let (sort, desc, limit) = match &view {
        BoundView::Rows { sort, desc, limit } => (sort.as_ref(), *desc, *limit),
        // a count shows no rows (AQ1)
        BoundView::Counts(_) | BoundView::Cross(_) => (None, false, 0),
    };
    let scalars: Option<Vec<Scalar>> = sort.map(|(_, key)| {
        matches
            .iter()
            .map(|(at, _)| eval::scalar(key, &corpus.items[*at]))
            .collect()
    });
    let mut order: Vec<usize> = (0..matches.len()).collect();
    if let Some(scalars) = &scalars {
        order.sort_by(|a, b| match (scalars[*a], scalars[*b]) {
            (Scalar::Value(x), Scalar::Value(y)) => {
                let by = x.cmp(&y);
                if desc { by.reverse() } else { by }
            }
            (Scalar::Value(_), _) => std::cmp::Ordering::Less,
            (_, Scalar::Value(_)) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        });
    }
    let rows: Vec<Row> = order
        .iter()
        .take(limit)
        .map(|&m| {
            let (at, outcomes) = &matches[m];
            let held = &corpus.items[*at];
            let mut touched = Vec::new();
            eval::touched(&query.bound, outcomes, true, &mut touched);
            Row {
                id: held.item.facts.id.clone(),
                name: held.item.name.clone(),
                typeline: held.item.typeline.clone(),
                base: held.item.base.clone(),
                rarity: held.item.rarity.clone(),
                place: held.place.clone(),
                matched: touched
                    .into_iter()
                    .map(|i| {
                        let term = &query.terms[i];
                        let (shows, left_out) = eval::evidence(&query.terms, term, held, outcomes);
                        Touched {
                            path: term.path.clone(),
                            term: print::print(&term.node),
                            shows,
                            left_out,
                        }
                    })
                    .filter(|t| !t.shows.is_empty())
                    .collect(),
                sort: scalars.as_ref().map(|s| match s[m] {
                    Scalar::Value(n) => Sorted {
                        value: Some(eval::number_json(n.as_f64())),
                        status: None,
                    },
                    Scalar::None => Sorted {
                        value: None,
                        status: Some("no satisfying occurrence"),
                    },
                    Scalar::Incomplete(n) => Sorted {
                        value: n.map(|n| eval::number_json(n.as_f64())),
                        status: Some("incomplete"),
                    },
                }),
            }
        })
        .collect();

    let zero = (total.matched == 0).then(|| Zero {
        resolved_to_nothing: query
            .terms
            .iter()
            .enumerate()
            .filter_map(|(i, term)| {
                nothing(
                    corpus,
                    term,
                    picked[i],
                    tallies[i][Outcome::Matched as usize],
                )
            })
            .collect(),
        said: S107,
        undecided: count(n_undecided, root_undecided.as_ref()),
        not_built: vec!["--explain", "--context"],
    });
    let left_out = total.matched - rows.len();
    let view = match &view {
        BoundView::Rows { .. } => ViewOut::Rows(RowsOut {
            limit,
            returned: rows.len(),
            left_out,
            left_out_needs: (left_out > 0).then_some("--next"),
            sort: sort.map(|(text, _)| text.clone()),
            desc,
        }),
        BoundView::Counts(bound) => ViewOut::Counts(counts::tables(bound, &counted)),
        BoundView::Cross(bound) => ViewOut::Cross(counts::crossed(bound, &counted)),
    };
    Ok(Answer {
        query: QueryOut {
            text: query.text(),
            tree: json::to_json(&query.root),
        },
        scope: ScopeOut {
            account: Account {
                uuid: corpus.basis.account.clone(),
                name: corpus.account_name.clone(),
            },
            realm: corpus.realm.clone(),
            membership: request.scope.membership,
            realm_not_held: match &corpus.realm {
                Realm::One(realm) => !corpus.coverage.realms_held.contains(realm),
                Realm::All => false,
            },
            coverage: corpus.coverage.clone(),
        },
        basis: corpus.basis.clone(),
        terms,
        total,
        view,
        rows,
        zero,
    })
}

fn account_is(corpus: &Corpus, account: &str) -> bool {
    let name = corpus.account_name.as_deref().unwrap_or_default();
    account.eq_ignore_ascii_case(&corpus.basis.account)
        || account.eq_ignore_ascii_case(name)
        // a username may carry its discriminator: `name#1234`
        || name.split('#').next().is_some_and(|n| account.eq_ignore_ascii_case(n))
        || account.split('#').next().is_some_and(|a| a.eq_ignore_ascii_case(name))
}

/// What an item is called where a whole row is too much.
fn label(held: &Held) -> Option<String> {
    match (&held.item.name, &held.item.typeline) {
        (Some(name), Some(typeline)) => Some(format!("{name} {typeline}")),
        (name, typeline) => name.clone().or_else(|| typeline.clone()),
    }
}

// ---- routes ------------------------------------------------------------------------------------------

/// The one maker of a route (invariant 4): a request over the answer's
/// scope, labelled with the basis it was counted at.
pub(crate) struct Router<'a> {
    corpus: &'a Corpus,
    scope: Scope,
    /// The query answered: what a bucket's route is a request under.
    root: &'a Node,
}

impl Router<'_> {
    fn route(&self, node: &Node, realm: Option<&str>) -> Route {
        let mut scope = self.scope.clone();
        if let Some(realm) = realm {
            scope.realm = Some(Realm::One(realm.to_string()));
        }
        Route {
            counted_at: self.corpus.basis.clone(),
            denominator: "scope",
            request: Request {
                scope,
                query: QueryInput {
                    text: Some(print::print(node)),
                    tree: None,
                },
                view: View::default(),
            },
        }
    }

    /// The count of `node`'s matches by `key`, over the scope.
    fn continuation(&self, node: &Node, key: String) -> Route {
        let mut route = self.route(node, None);
        route.request.view = View::Counts(Counts {
            keys: vec![key],
            sum: None,
            limit: None,
        });
        route
    }

    /// A count over the scope, with its route when it is not zero.
    pub(crate) fn count(&self, n: usize, node: Option<&Node>) -> Count {
        Count {
            count: n,
            route: node.filter(|_| n > 0).map(|node| self.route(node, None)),
        }
    }

    /// A count of the answer's matches: its route is the query and the
    /// terms that select it — the old query, parenthesised, and new terms
    /// (C91) — never a fragment, and under one realm where a vocabulary row
    /// is one realm's (C97).
    pub(crate) fn under(&self, n: usize, terms: Vec<Node>, realm: Option<&str>) -> Count {
        let mut all = match self.root {
            Node::All(children) if children.is_empty() => Vec::new(),
            root => vec![root.clone()],
        };
        all.extend(terms);
        let node = if all.len() == 1 {
            all.remove(0)
        } else {
            Node::All(all)
        };
        Count {
            count: n,
            route: (n > 0).then(|| self.route(&node, realm)),
        }
    }
}

struct Routes {
    failed: Option<Node>,
    lacked: Option<Node>,
    undecided: Node,
    together: Option<(Node, group::LowerBound)>,
}

fn not(node: Node) -> Node {
    Node::Not(Box::new(node))
}

fn routes(term: &Term) -> Routes {
    let node = term.node.clone();
    let undecided = Node::Undecided(Probe::Term(Box::new(node.clone())));
    let (failed, lacked, together) = match (&term.atom, &term.node) {
        (Atom::Lines(group), _) if group.selects_only => (None, Some(not(node.clone())), None),
        (Atom::Lines(group), _) => {
            let selected = Node::Members {
                of: Collection::Lines,
                where_: Box::new(group.selector_tree.clone()),
            };
            // a selector that asks a flag may be open on an occurrence whose
            // flags are unread: such an item did not lack the line, so the
            // failed route admits it, and stays as short as the reference's
            // wherever no flag is asked
            let picked = if group.selector_asks_a_flag {
                Node::Any(vec![
                    selected.clone(),
                    Node::Undecided(Probe::Term(Box::new(selected.clone()))),
                ])
            } else {
                selected.clone()
            };
            let failed = Node::All(vec![picked, not(node.clone())]);
            let together = group.together.clone().map(|lower| {
                let sum = Node::Compare {
                    value: ValueRef::Sum {
                        lines: Box::new(group.selector_tree.clone()),
                        slot: lower.slot.clone(),
                    },
                    op: lower.op,
                    rhs: Value::Number(lower.bound),
                };
                (
                    Node::All(vec![selected.clone(), not(node.clone()), sum]),
                    lower,
                )
            });
            (Some(failed), Some(not(selected)), together)
        }
        (Atom::Has(_), _) => (None, Some(not(node.clone())), None),
        (
            Atom::Text { thing, .. } | Atom::Closed { thing, .. } | Atom::Number { thing, .. },
            Node::Test { field, .. },
        ) if *thing != Thing::Text => {
            let has = Node::Has(field.clone());
            (
                Some(Node::All(vec![has.clone(), not(node.clone())])),
                Some(not(has)),
                None,
            )
        }
        _ => (Some(not(node.clone())), None, None),
    };
    Routes {
        failed,
        lacked,
        undecided,
        together,
    }
}

// ---- what a selector resolved to ------------------------------------------------------------

/// What a term's `:` or `~` selector ranges over, when it has one: a
/// field, or the templates of a line's group — a `sum`'s group too.
/// Whether every template test of the term's group is a quoted `"T"`. Such
/// a term resolves to itself and lists nothing — unless it found more than
/// one spelling, which any-case `=` can (owner, 2026-09-20: six pairs of
/// the census's templates differ only by capitals), and then it says so.
fn exact_only(term: &Term) -> bool {
    match &term.atom {
        Atom::Lines(group) | Atom::Sum { group, .. } => group.quoted_only(),
        _ => false,
    }
}

fn resolves(term: &Term) -> Option<String> {
    // a quoted template resolves too: `=` is any-case, and GGG has
    // spelled some lines two ways (`exact_only`, above)
    if let Atom::Lines(group) | Atom::Sum { group, .. } = &term.atom {
        return group.names_a_template().then(|| "template".to_string());
    }
    match &term.node {
        Node::Test {
            field,
            op: Op::Contains | Op::Match,
            ..
        } if !matches!(
            &term.atom,
            Atom::Text {
                thing: Thing::Text,
                ..
            } | Atom::Id(_)
        ) =>
        {
            Some(field.clone())
        }
        _ => None,
    }
}

/// The count that lists everything a term's selector resolved to: the
/// term alone — a group's selector alone — counted by its field, or the
/// vocabulary narrowed by the group's one template test. None for `tab`
/// (E1: counted by the tab, resolved to names) and for a group whose
/// selector is more than one template test.
fn rest_of(term: &Term) -> Option<(Node, String)> {
    match &term.atom {
        Atom::Lines(group) | Atom::Sum { group, .. } => {
            let (op, text) = group.sole_template_test()?;
            let key = match op {
                Op::Match => format!("line~{text}"),
                _ => format!("line:{text}"),
            };
            Some((
                Node::Members {
                    of: Collection::Lines,
                    where_: Box::new(group.selector_tree.clone()),
                },
                key,
            ))
        }
        Atom::Text { thing, .. } | Atom::Closed { thing, .. } if *thing != Thing::Tab => {
            match &term.node {
                Node::Test { field, .. } => Some((term.node.clone(), field.clone())),
                _ => None,
            }
        }
        _ => None,
    }
}

/// What the term's selector resolved to on this item. A group's is what
/// its selector picks, whatever its comparisons then make of it: `20%
/// to Fire Resistance` is a line `template:resistance arg1>=60` resolved
/// to, and its value failed. A field's is the value that matched.
fn resolved_values(term: &Term, held: &Held, outcome: Outcome) -> Vec<String> {
    let mut values: Vec<String> = match &term.atom {
        Atom::Lines(group) | Atom::Sum { group, .. } => eval::selected(held, group)
            .map(|l| l.template.clone())
            .collect(),
        Atom::Text { thing, test } if outcome == Outcome::Matched => eval::texts(held, *thing)
            .into_iter()
            .filter(|v| test.holds(v))
            .map(str::to_string)
            .collect(),
        Atom::Closed { thing, .. } if outcome == Outcome::Matched => eval::texts(held, *thing)
            .into_iter()
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    values.sort();
    values.dedup();
    values
}

// ---- a selector nothing carries ------------------------------------------------------------------

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3)
        .map(str::to_lowercase)
        .collect()
}

/// A term whose selector nothing in scope carries, with the values
/// sharing its words. Exact execution, tolerant suggestion: nothing here
/// is matched for the author (S107). A group is read as the rest of the
/// answer reads it — by its bound selector, a `sum`'s too — so one that
/// picked any occurrence resolved to something, whatever a template test
/// inside it would find alone; a field's term carried nothing when no item
/// matched it. A pattern that resolved to nothing is listed like any other
/// and has no words to suggest by.
fn nothing(corpus: &Corpus, term: &Term, picked: bool, matched: usize) -> Option<Nothing> {
    // what the selector ranges over; the words a suggestion is scored
    // against, which a pattern has none of and is listed all the same; and
    // the field its values are read from, a group's being its templates
    let (of, wanted, thing): (String, Option<String>, Option<Thing>) =
        match (&term.node, &term.atom) {
            (_, Atom::Lines(group) | Atom::Sum { group, .. })
                if !picked && group.names_a_template() =>
            {
                ("template".to_string(), group.wanted(), None)
            }
            (
                Node::Test {
                    field, op, value, ..
                },
                Atom::Text { thing, .. },
            ) if *thing != Thing::Text && matched == 0 => {
                let wanted = match (op, value) {
                    (Op::Contains | Op::Eq, Value::Text(wanted)) => Some(wanted.clone()),
                    _ => None,
                };
                (field.clone(), wanted, Some(*thing))
            }
            // a closed set's `:` or `~` picks among legal values, and the
            // answer says what it resolved to: nothing, here
            (
                Node::Test {
                    field,
                    op: Op::Contains | Op::Match,
                    ..
                },
                Atom::Closed { thing, .. },
            ) if matched == 0 => (field.clone(), None, Some(*thing)),
            _ => return None,
        };
    let values_of = |held: &'_ Held| -> Vec<String> {
        let mut values: Vec<String> = match thing {
            Some(thing) => eval::texts(held, thing)
                .into_iter()
                .map(str::to_string)
                .collect(),
            None => held.item.lines.iter().map(|l| l.template.clone()).collect(),
        };
        values.sort_unstable();
        values.dedup();
        values
    };
    let mut suggestions: Vec<Suggestion> = Vec::new();
    if let Some(wanted) = wanted {
        let mut carried: HashMap<String, usize> = HashMap::new();
        for held in &corpus.items {
            for value in values_of(held) {
                *carried.entry(value).or_default() += 1;
            }
        }
        let wanted_words = words(&wanted);
        let mut scored: Vec<(usize, Carried)> = carried
            .into_iter()
            .filter_map(|(value, items)| {
                let has = words(&value);
                // a plural typed for a singular shares the word too
                let shared = wanted_words
                    .iter()
                    .filter(|w| has.iter().any(|h| h.contains(*w) || w.contains(h)))
                    .count();
                (shared > 0).then_some((shared, Carried { value, items }))
            })
            .collect();
        scored.sort_by(|(sa, a), (sb, b)| {
            sb.cmp(sa)
                .then_with(|| b.items.cmp(&a.items))
                .then_with(|| a.value.cmp(&b.value))
        });
        // a suggestion's count is its term's (invariant 4): the term is an
        // any-case `=`, so it is asked of the scope as it will be when
        // typed, and a second spelling of a value already offered is the
        // same term and is not offered again
        let mut offered: Vec<bind::TextTest> = Vec::new();
        for (_, candidate) in scored {
            if suggestions.len() == SUGGESTED {
                break;
            }
            if offered.iter().any(|test| test.holds(&candidate.value)) {
                continue;
            }
            let exact = Value::Text(candidate.value.clone());
            let Ok(test) = bind::text_test(&of, Op::Eq, &exact) else {
                continue;
            };
            suggestions.push(Suggestion {
                term: print::print(&exactly(&of, &candidate.value)),
                value: candidate.value,
                items: corpus
                    .items
                    .iter()
                    .filter(|held| values_of(held).iter().any(|v| test.holds(v)))
                    .count(),
            });
            offered.push(test);
        }
    }
    Some(Nothing {
        path: term.path.clone(),
        term: print::print(&term.node),
        suggestions,
        of,
    })
}

/// The exact term that selects one value: `line("T")`, `base="…"`.
fn exactly(of: &str, value: &str) -> Node {
    if of == "template" {
        return group::line_of(value);
    }
    Node::Test {
        field: of.to_string(),
        op: Op::Eq,
        value: Value::Text(value.to_string()),
    }
}

/// A command a terminal takes: the one way the search prints one (the
/// build plan's rule 5 — a command printed is a command that runs). It
/// names the account it was answered for, since a second account makes a
/// command without one an error, and a last word that starts with `-` —
/// the language's not, a terminal's flag — goes after `--`, or `-has:note`
/// is read as `-h`.
pub fn command(verb: &str, account: Option<&str>, realm: Option<&str>, last: &str) -> String {
    let mut out = format!("acq {verb}");
    if let Some(account) = account {
        out.push_str(&format!(" --account {}", shell_quoted(account)));
    }
    if let Some(realm) = realm {
        out.push_str(&format!(" --realm {realm}"));
    }
    out.push_str(if last.starts_with('-') { " -- " } else { " " });
    out.push_str(&shell_quoted(last));
    out
}

impl Route {
    /// The route as a command a terminal takes: the query, then the view
    /// where it is not the default rows.
    pub fn command(&self) -> String {
        let scope = &self.request.scope;
        let mut out = command(
            "search",
            scope.account.as_deref(),
            scope.realm.as_ref().map(Realm::as_str),
            self.request.query.text.as_deref().unwrap_or_default(),
        );
        let (flag, counts) = match &self.request.view {
            View::Rows(rows) => {
                if let Some(sort) = &rows.sort {
                    out.push_str(&format!(" --sort {}", shell_quoted(sort)));
                    if rows.desc {
                        out.push_str(" --desc");
                    }
                }
                if let Some(limit) = rows.limit {
                    out.push_str(&format!(" --limit {limit}"));
                }
                return out;
            }
            View::Counts(c) => ("--count", c),
            View::Cross(c) => ("--cross", c),
        };
        // a key is spelled at a terminal as a word of the list: after
        // `line:` the rest are texts, so a text key is its text alone — and
        // a text is quoted in the list's own grammar where the list would
        // read it otherwise (`search_cmd::keys`): a comma, a quote, a
        // backslash, a row break, a leading `~`, whitespace at an end
        let mut keys = Vec::new();
        let mut narrowing = false;
        for key in &counts.keys {
            let spelled = match (narrowing, key.split_once(['~', ':'])) {
                (true, Some((_, text))) if key.starts_with("line~") => {
                    format!("~{}", listed_text(text))
                }
                (true, Some((_, text))) if key.starts_with("line:") => listed_text(text),
                (false, Some((prefix, text)))
                    if key.starts_with("line~") || key.starts_with("line:") =>
                {
                    format!("{prefix}{}{}", &key[4..5], listed_text(text))
                }
                _ => key.clone(),
            };
            narrowing |= key.starts_with("line:") || key.starts_with("line~");
            keys.push(spelled);
        }
        out.push_str(&format!(" {flag} {}", shell_quoted(&keys.join(","))));
        if let Some(sum) = &counts.sum {
            out.push_str(&format!(" --sum {}", shell_quoted(sum)));
        }
        if let Some(limit) = counts.limit {
            out.push_str(&format!(" --limit {limit}"));
        }
        out
    }
}

impl Answer {
    /// The command that shows one of this answer's items whole: where a
    /// bounded block of a row sends its reader (invariant 5).
    pub fn show_command(&self, id: &str) -> String {
        let account = &self.scope.account;
        command(
            "show",
            Some(account.name.as_deref().unwrap_or(&account.uuid)),
            None,
            id,
        )
    }
}

/// A text of the count list, quoted in the list's grammar when the list
/// would read part of it as syntax, with the language's three escapes.
fn listed_text(text: &str) -> String {
    let plain = !text.is_empty()
        && !text.contains([',', '"', '\\', '\n'])
        && !text.starts_with('~')
        && text.trim() == text;
    if plain {
        return text.to_string();
    }
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

/// One shell word, in single quotes when it needs them; an apostrophe
/// inside is `'\''` (the build plan, gap 2).
pub fn shell_quoted(text: &str) -> String {
    let plain = !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./@:=".contains(c));
    if plain {
        text.to_string()
    } else {
        format!("'{}'", text.replace('\'', "'\\''"))
    }
}
