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
//!   ten most carried listed and the rest counted. The route to the rest is the vocabulary read, which is not
//!   built (rule 5 of the plan: the count, and the construct's name).
//! - **Rows** are the matching items in the store's stable order, or by
//!   the sort scalar with items that have none last either way; past the
//!   limit they are counted, and the way on is a larger limit until
//!   `--next` is built.
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
use crate::error::{LanguageError, SearchError};
use crate::eval::{self, Outcome, Scalar, Truth};
pub use crate::eval::{Evidence, Reason};
use crate::tree::{Collection, Member, Node, Op, Probe, Value, ValueRef};
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

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    #[serde(default)]
    pub rows: Rows,
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
    /// Values past the listed ones; their route is the vocabulary read.
    pub more: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_needs: Option<&'static str>,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct Why {
    pub path: String,
    pub term: String,
    #[serde(flatten)]
    pub reason: Reason,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViewOut {
    pub rows: RowsOut,
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
    let sort = match &request.view.rows.sort {
        Some(text) => {
            let value = parse::parse_value(text)?;
            Some((print::print_value(&value), bind::bind_sort(&value)?))
        }
        None => None,
    };
    let limit = request.view.rows.limit.unwrap_or(DEFAULT_LIMIT);

    let scope = Scope {
        account: Some(
            corpus
                .account_name
                .clone()
                .unwrap_or_else(|| corpus.basis.account.clone()),
        ),
        realm: Some(corpus.realm.clone()),
        membership: request.scope.membership,
    };
    let route = |node: &Node| Route {
        counted_at: corpus.basis.clone(),
        denominator: "scope",
        request: Request {
            scope: scope.clone(),
            query: QueryInput {
                text: Some(print::print(node)),
                tree: None,
            },
            view: View::default(),
        },
    };
    let count = |n: usize, node: Option<&Node>| Count {
        count: n,
        route: node.filter(|_| n > 0).map(&route),
    };

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
                resolved: carried[i].take().zip(resolves(term)).map(|(carried, of)| {
                    let mut values: Vec<Carried> = carried
                        .into_iter()
                        .map(|(value, items)| Carried { value, items })
                        .collect();
                    values
                        .sort_by(|a, b| b.items.cmp(&a.items).then_with(|| a.value.cmp(&b.value)));
                    let more = values.len().saturating_sub(LISTED);
                    values.truncate(LISTED);
                    Resolved {
                        of,
                        values,
                        more,
                        more_needs: (more > 0).then_some("--count"),
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
                UndecidedItem {
                    id: held.item.facts.id.clone(),
                    name: label(held),
                    why: blamed
                        .into_iter()
                        .flat_map(|i| {
                            let term = &query.terms[i];
                            eval::reasons(&term.atom, held)
                                .into_iter()
                                .map(|reason| Why {
                                    path: term.path.clone(),
                                    term: print::print(&term.node),
                                    reason,
                                })
                        })
                        .collect(),
                }
            })
            .collect(),
    };

    // the order, then the limit
    let scalars: Option<Vec<Scalar>> = sort.as_ref().map(|(_, key)| {
        matches
            .iter()
            .map(|(at, _)| eval::scalar(key, &corpus.items[*at]))
            .collect()
    });
    let mut order: Vec<usize> = (0..matches.len()).collect();
    if let Some(scalars) = &scalars {
        let desc = request.view.rows.desc;
        order.sort_by(|a, b| match (scalars[*a], scalars[*b]) {
            (Scalar::Value(x), Scalar::Value(y)) => {
                let by = x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal);
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
                        Touched {
                            path: term.path.clone(),
                            term: print::print(&term.node),
                            shows: eval::evidence(&query.terms, term, held, outcomes),
                        }
                    })
                    .filter(|t| !t.shows.is_empty())
                    .collect(),
                sort: scalars.as_ref().map(|s| match s[m] {
                    Scalar::Value(n) => Sorted {
                        value: Some(eval::number_json(n)),
                        status: None,
                    },
                    Scalar::None => Sorted {
                        value: None,
                        status: Some("no satisfying occurrence"),
                    },
                    Scalar::Incomplete(n) => Sorted {
                        value: n.map(eval::number_json),
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
        view: ViewOut {
            rows: RowsOut {
                limit,
                returned: rows.len(),
                left_out,
                left_out_needs: (left_out > 0).then_some("--next"),
                sort: sort.map(|(text, _)| text),
                desc: request.view.rows.desc,
            },
        },
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

struct Routes {
    failed: Option<Node>,
    lacked: Option<Node>,
    undecided: Node,
    together: Option<(Node, bind::LowerBound)>,
}

fn not(node: Node) -> Node {
    Node::Not(Box::new(node))
}

fn asks_a_flag(member: &Member) -> bool {
    match member {
        Member::All(children) | Member::Any(children) => children.iter().any(asks_a_flag),
        Member::Not(inner) => asks_a_flag(inner),
        Member::Is(_) => true,
        Member::Test { .. } | Member::Const(_) => false,
    }
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
            let picked = if asks_a_flag(&group.selector_tree) {
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

/// The template tests a group makes, anywhere in it.
fn template_tests(member: &Member, out: &mut Vec<(Op, String)>) {
    match member {
        Member::All(children) | Member::Any(children) => {
            children.iter().for_each(|c| template_tests(c, out));
        }
        Member::Not(inner) => template_tests(inner, out),
        Member::Test {
            attr,
            op,
            value: Value::Text(text),
        } if attr == "template" => out.push((*op, text.clone())),
        _ => {}
    }
}

/// What a term's `:` or `~` selector ranges over, when it has one: a
/// field, or the templates of a line's group — a `sum`'s group too.
fn resolves(term: &Term) -> Option<String> {
    let of_group = |where_: &Member| {
        let mut tests = Vec::new();
        template_tests(where_, &mut tests);
        tests
            .iter()
            .any(|(op, _)| matches!(op, Op::Contains | Op::Match))
            .then(|| "template".to_string())
    };
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
        Node::Members {
            of: Collection::Lines,
            where_,
        } => of_group(where_),
        Node::Compare {
            value: ValueRef::Sum { lines, .. },
            ..
        } => of_group(lines),
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
/// matched it.
fn nothing(corpus: &Corpus, term: &Term, picked: bool, matched: usize) -> Option<Nothing> {
    let of_group = |where_: &Member| {
        if picked {
            return None;
        }
        let mut tests = Vec::new();
        template_tests(where_, &mut tests);
        let wanted: Vec<String> = tests
            .into_iter()
            .filter(|(op, _)| *op != Op::Match)
            .map(|(_, text)| text)
            .collect();
        if wanted.is_empty() {
            return None;
        }
        let mut templates: HashMap<String, usize> = HashMap::new();
        for held in &corpus.items {
            let mut seen: Vec<&str> = held
                .item
                .lines
                .iter()
                .map(|l| l.template.as_str())
                .collect();
            seen.sort_unstable();
            seen.dedup();
            for template in seen {
                *templates.entry(template.to_string()).or_default() += 1;
            }
        }
        Some(("template".to_string(), wanted.join(" "), templates))
    };
    let (of, wanted, values): (String, String, HashMap<String, usize>) =
        match (&term.node, &term.atom) {
            (
                Node::Members {
                    of: Collection::Lines,
                    where_,
                },
                Atom::Lines(_),
            ) => of_group(where_)?,
            (
                Node::Compare {
                    value: ValueRef::Sum { lines, .. },
                    ..
                },
                Atom::Sum { .. },
            ) => of_group(lines)?,
            (
                Node::Test {
                    field,
                    op: Op::Contains | Op::Eq,
                    value: Value::Text(wanted),
                },
                Atom::Text { thing, .. },
            ) if *thing != Thing::Text && matched == 0 => {
                let mut values: HashMap<String, usize> = HashMap::new();
                for held in &corpus.items {
                    for value in eval::texts(held, *thing) {
                        *values.entry(value.to_string()).or_default() += 1;
                    }
                }
                (field.clone(), wanted.clone(), values)
            }
            _ => return None,
        };
    let wanted_words = words(&wanted);
    let mut scored: Vec<(usize, Carried)> = values
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
    Some(Nothing {
        path: term.path.clone(),
        term: print::print(&term.node),
        suggestions: scored
            .into_iter()
            .take(SUGGESTED)
            .map(|(_, carried)| Suggestion {
                term: print::print(&exactly(&of, &carried.value)),
                value: carried.value,
                items: carried.items,
            })
            .collect(),
        of,
    })
}

/// The exact term that selects one value: `line("T")`, `base="…"`.
fn exactly(of: &str, value: &str) -> Node {
    if of == "template" {
        return Node::Members {
            of: Collection::Lines,
            where_: Box::new(Member::Test {
                attr: "template".to_string(),
                op: Op::Eq,
                value: Value::Text(value.to_string()),
            }),
        };
    }
    Node::Test {
        field: of.to_string(),
        op: Op::Eq,
        value: Value::Text(value.to_string()),
    }
}

impl Route {
    /// The route as a command a terminal takes.
    pub fn command(&self) -> String {
        let scope = &self.request.scope;
        let mut out = String::from("acq search");
        if let Some(account) = &scope.account {
            out.push_str(&format!(" --account {}", shell_quoted(account)));
        }
        if let Some(realm) = &scope.realm {
            out.push_str(&format!(" --realm {}", realm.as_str()));
        }
        let query = self.request.query.text.as_deref().unwrap_or_default();
        // `-` is the language's not and a terminal's flag: a query that
        // starts with one goes after `--`, or `-has:note` is read as `-h`
        out.push_str(if query.starts_with('-') { " -- " } else { " " });
        out.push_str(&shell_quoted(query));
        out
    }
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
