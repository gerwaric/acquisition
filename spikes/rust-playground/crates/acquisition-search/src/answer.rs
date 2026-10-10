//! The request and the answer (C100, C96, C98): a query bound to a scope
//! and a view in, one answer shape out — the canonical query, the scope,
//! the basis, the terms block, the total, the rows, and how to go on.
//!
//! The shapes are `search/DESIGN.md`'s reference (*Two values*, *One
//! worked example*, *Explain*, *Invariants of the surface*), cited and not restated
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
//!   failed is `has:<field> -term` — `-term` alone for `class`, which
//!   never lacks (`class.rs`) — or `line(<selector>) -term` — with
//!   `or undecided(line(<selector>))` where the selector asks a flag, which
//!   an occurrence with unread flags leaves open; a `linked( … )` lacks
//!   with `-has:links`, an item with no link group, and fails with
//!   `has:links -term` (`sockets.rs`); a term
//!   that cannot lack fails by `-term`. The together count's is the failed
//!   route and the selector's sum against the bound.
//! - **A `:` or `~` selector is printed as authored with what it resolved
//!   to beside it** (invariant 2): the values its matched items carry,
//!   or — for a line's group, a `sum`'s too — the templates its selector
//!   picks over the scope, whatever its comparisons then make of them; the
//!   ten most carried listed and the rest counted. A quoted template
//!   resolves to itself and lists nothing, unless it found two spellings
//!   of one line, which any-case `=` can, and then both are listed — the
//!   spelling of the row, where it named a mod by a row (C90). The
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
//! - **The undecided block is by reason, never by item** (C93's contract
//!   detail, V8, 2026-09-26; taken here at step 9b): an answer shows its
//!   undecided items as one line per distinct reason — by what was
//!   unread, the kind a count tallies (C105) — each with one example
//!   item, the first met in the store's order, then the one route to
//!   them all; after the rows, before the routes; the first ten listed
//!   and the rest counted (invariant 5). No item list and no count per
//!   reason: no term selects a reason, so such a count would have no
//!   route. The JSON has the same shape. A per-reason count and its
//!   selector wait for someone missing the number. The seat's first shape
//!   — ten items, each with every term's reason — printed one reason
//!   eighteen times under eighteen `class:` terms (F9).
//! - **Rows** are the matching items in the store's stable order, or by
//!   the sort scalar with items that have none last either way; past the
//!   limit they are counted, and the way on is the answer's `next`. A
//!   counts view shows none: the matches are handed to `counts.rs`
//!   instead, with the one maker of a route (`Router`, private), so a
//!   bucket's route is made as a term's is.
//! - **A continuation is a request with its basis and its place** (C98,
//!   C100; the reference, *Two values*, *Outside the first surface*; the
//!   build plan, step 10): the answer's own request, scope resolved and
//!   query canonical, with the basis it was counted at and how many rows
//!   of the order came before. The place is what makes a request a
//!   continuation: with one, a basis that differs in any part from the
//!   corpus's is refused, both named part by part (`Basis::against`),
//!   and the offer is the request from its start; without one a basis
//!   is compared with nothing, since a route runs on any basis. No
//!   answer is held between asks: the order is a function of the basis,
//!   the store's stable order or a stable sort over it, so a place in
//!   it needs nothing kept — but where a facts file was deleted and
//!   refetched, or restored from a copy, to a revision the basis already
//!   names, which the basis cannot tell (`corpus.rs`; parked in
//!   `decisions/search.md`, its trigger fired by this). Rows alone
//!   continue; a table's values past
//!   its limit are a larger limit's. The token a terminal carries is
//!   that request and nothing else, encoded ([`Request::token`]).
//! - **A route read back** ([`Asked`]) is the request inside it; run
//!   where the basis has moved since it was counted, the answer carries
//!   what it said then — the count and its basis — beside what it says
//!   now (the reference, *One worked example*).
//! - **A zero total** prints, in place of rows: the selectors that
//!   resolved to nothing in this scope — a group's bound selector picked
//!   no occurrence, a field's term matched no item — each with the values
//!   sharing its words — a suggestion to type, never a match made for the author
//!   (S107: never fuzzy-matched); an `id:` that matched nothing, with the
//!   note that an id is whole (F7); the root's undecided route; and the
//!   offers (the reference, *When the total is zero*): explain of each
//!   part of the query — a child of the root, a root that is no and its
//!   one part — the part fewest items match first and by path where two
//!   are equal, ten listed and the rest counted (invariant 5), then the
//!   corpus context beside the view the request had, where a query was
//!   typed. An offer is a request, made by the one maker of a route's
//!   (`Router::request`), and a command only as every route is one
//!   ([`Request::command`]): the block computes no forced count.
//! - **Explain is a view** (the reference, *Explain*; the build plan,
//!   step 10): it travels in the request as the rows and the counts do,
//!   so it stands beside neither, and an adapter's flag is an encoding
//!   of it. The path names a node of the query as returned
//!   (`tree::nodes`, the one walk that numbers them); the substitution
//!   is that walk's too, the one node replaced by `true()` or `false()`
//!   and nothing around it touched (invariant 1). Each transformed query
//!   is then bound and answered as any query is — this module asks
//!   itself, as the request it is, and reads the total — so a forced
//!   count is a count the build gives a query a user could type, by the
//!   one evaluator, and its route is that query. A path the query lacks
//!   is an authoring error whose readings list the nodes with their
//!   paths; a substitution the binder refuses is the binder's error;
//!   both are said before any item is read. The answer keeps its terms
//!   block and its total, shows no rows, and no zero block: its three
//!   pairs of counts stand where the rows would.
//! - **The corpus context** (the reference, *The request at a
//!   terminal*): the answer is the empty query's under whatever view the
//!   request has, and the query typed is carried as set aside — read
//!   first, so that one that does not bind is the error it is in any
//!   context, and counted nowhere: no term, no route and no bucket holds
//!   it. Nothing typed, nothing set aside.
//! - **A row says what it sorts by** (F6, the first seat: a sort on a line
//!   the query did not name showed the number alone): the field by its
//!   name, the occurrence whose slot is the largest, a sum's
//!   contributors, a computed value's inputs — shown as a matched term's
//!   evidence is, bounded the same way (`eval::sorted_by`).
//! - **Membership is the corpus's** (the reference, *Membership*; C108;
//!   the build plan, step 10): `live`, or `all`, where the store's read
//!   hands over every item at the place it was last seen. A request
//!   naming the other than the corpus holds is refused, as a realm is.
//!   Under `all` a removed item is a row like any other, marked with
//!   when it left and counted by every term; the scope block counts the
//!   live and the removed apart; its price is lacked (`price.rs`); every
//!   route and every command carries the membership. No count says how
//!   many of its items are removed: nothing selects the removed alone,
//!   so such a count would have no route (invariant 4). Under `live` an
//!   `id:` term that matches nothing says whether the id is among what
//!   the read left out — an item removed, a tab or a character retired
//!   — with what it was, when it left and the route under `all`,
//!   whatever the total; under `all` the same is said of a retired tab
//!   or character that held no item, with no route, there being nothing
//!   more to find. Never of an id that is also a live location's.

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

/// What a zero answer says of an `id:` term that matched nothing (F7, the
/// first seat): the help's rule, where the reader is.
pub const ID_WHOLE: &str =
    "an id is matched whole, as an answer printed it: a part of one finds nothing";

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
    /// What the answer is of: the query's matches, or the corpus — the
    /// empty query over the same scope, the query typed set aside.
    #[serde(default, skip_serializing_if = "Context::is_matches")]
    pub context: Context,
    /// The basis a continuation was counted at (C98, C100): what its
    /// place in the rows is a place in. Beside no place it is carried
    /// and compared with nothing — a route runs on any basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<Basis>,
}

/// What an answer is of (the reference, *The request at a terminal*):
/// `matches`, what every answer is unless it says otherwise, or `corpus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Context {
    #[default]
    Matches,
    Corpus,
}

impl Context {
    fn is_matches(&self) -> bool {
        *self == Context::Matches
    }

    /// The word an adapter's flag carries.
    pub fn parse(word: &str) -> Result<Context, LanguageError> {
        match word {
            _ if word.eq_ignore_ascii_case("matches") => Ok(Context::Matches),
            _ if word.eq_ignore_ascii_case("corpus") => Ok(Context::Corpus),
            _ => Err(LanguageError::new(
                crate::error::ErrorKind::View,
                format!(
                    "`{word}` is no context: `matches`, what every answer is of, or `corpus`, the empty query over the same scope with the query typed set aside"
                ),
            )
            .with_readings(vec!["matches".to_string(), "corpus".to_string()])),
        }
    }
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
    /// Removed items too, each row marked with when it left.
    All,
}

impl From<acquisition_store::Membership> for Membership {
    fn from(membership: acquisition_store::Membership) -> Membership {
        match membership {
            acquisition_store::Membership::Live => Membership::Live,
            acquisition_store::Membership::All => Membership::All,
        }
    }
}

impl From<Membership> for acquisition_store::Membership {
    fn from(membership: Membership) -> acquisition_store::Membership {
        match membership {
            Membership::Live => acquisition_store::Membership::Live,
            Membership::All => acquisition_store::Membership::All,
        }
    }
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
/// table for each key, or one crossed table of two (C95) — or, of one
/// node of the query, the root's counts with it forced true and forced
/// false, and no rows (the reference, *Explain*). One of the four, as the
/// reference's synopsis has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    Rows(Rows),
    Counts(Counts),
    Cross(Counts),
    Explain(Explain),
}

/// The node explained, by its path in the query as an answer returns it
/// (`tree::nodes`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Explain {
    pub path: String,
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
    /// A continuation's place: how many rows of this order the answers
    /// before it returned. It is what makes a request a continuation,
    /// refused across a changed basis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<usize>,
}

impl QueryInput {
    /// The query, parsed or read, and bound.
    pub fn bind(&self) -> Result<Query, LanguageError> {
        match (&self.text, &self.tree) {
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

/// The empty query: every item in scope.
fn is_empty(root: &Node) -> bool {
    matches!(root, Node::All(children) if children.is_empty())
}

impl Request {
    /// The query of a request as typed, parsed or read, and bound.
    pub fn bind(&self) -> Result<Query, LanguageError> {
        self.query.bind()
    }

    /// The query the answer is of, and the one set aside: under `corpus`
    /// the empty query, the typed one beside it — still read, so that one
    /// that does not bind is the error it is in any context. Explain is
    /// of the query typed, so beside `corpus` it is the view's error.
    fn asked(&self) -> Result<(Query, Option<Query>), LanguageError> {
        if self.context == Context::Corpus && matches!(self.view, View::Explain(_)) {
            return Err(LanguageError::new(
                crate::error::ErrorKind::View,
                "`--explain` is of the query typed, and `--context corpus` sets it aside: ask one or the other",
            ));
        }
        let typed = self.bind()?;
        Ok(match self.context {
            Context::Matches => (typed, None),
            Context::Corpus => (
                bind::parse_query("")?,
                (!is_empty(&typed.root)).then_some(typed),
            ),
        })
    }

    /// A continuation's place, where the request is one: a place names
    /// the basis it is a place in.
    fn place(&self) -> Result<Option<(usize, &Basis)>, LanguageError> {
        match (&self.view, &self.basis) {
            (
                View::Rows(Rows {
                    after: Some(after), ..
                }),
                Some(basis),
            ) => Ok(Some((*after, basis))),
            (View::Rows(Rows { after: Some(_), .. }), None) => Err(LanguageError::new(
                crate::error::ErrorKind::View,
                "a place in the rows is a place at a basis: a continuation names the basis its answer was counted at, as an answer's `next` does",
            )),
            _ => Ok(None),
        }
    }

    /// The request as a token a terminal carries: `--next` takes it.
    pub fn token(&self) -> String {
        use base64::Engine as _;
        let json = serde_json::to_vec(self).unwrap_or_default();
        format!(
            "{TOKEN_PREFIX}{}",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(json)
        )
    }

    /// The continuation a token is: a request with its basis and its
    /// place, and nothing that is not one.
    pub fn from_token(token: &str) -> Result<Request, LanguageError> {
        use base64::Engine as _;
        let no_token = || {
            LanguageError::new(
                crate::error::ErrorKind::Token,
                "this is no token an answer printed: `--next` takes one whole, as the answer's `more` line gives it",
            )
        };
        let encoded = token
            .trim()
            .strip_prefix(TOKEN_PREFIX)
            .ok_or_else(no_token)?;
        let json = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| no_token())?;
        let request: Request = serde_json::from_slice(&json).map_err(|_| no_token())?;
        match request.place() {
            Ok(Some(_)) => Ok(request),
            _ => Err(no_token()),
        }
    }

    /// Every authoring error the request holds — the query's, the sort's,
    /// the view's, a path the query lacks — with no store read: what an
    /// adapter asks before it opens one.
    pub fn check(&self) -> Result<(), LanguageError> {
        self.place()?;
        let (query, _) = self.asked()?;
        self.view.bound()?.of(&query).map(|_| ())
    }
}

/// What a token starts with: the format, so that another is told apart.
const TOKEN_PREFIX: &str = "acq1.";

/// What `--request` reads: a request, or what an answer holds one in —
/// a count's route, the request with the count and the basis it was
/// counted at (the reference, *One worked example*); a `next`; an offer.
#[derive(Debug, Clone, PartialEq)]
pub struct Asked {
    pub request: Request,
    pub before: Option<Before>,
}

/// What a route said when it was printed: its count, and the basis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Before {
    pub count: usize,
    pub counted_at: Basis,
}

impl Asked {
    /// A request, or an object with the request inside it — and where
    /// that object says a count, the basis it was counted at beside it,
    /// which makes it a route; what is neither is the tree's error, as a
    /// request that does not read is.
    pub fn read(value: &Json) -> Result<Asked, LanguageError> {
        let unread = |e: serde_json::Error| {
            LanguageError::new(
                crate::error::ErrorKind::Tree,
                format!("this is no request and no route an answer printed: {e}"),
            )
        };
        if !value.is_object() {
            return Err(LanguageError::new(
                crate::error::ErrorKind::Tree,
                "this is no request and no route an answer printed: a request is an object",
            ));
        }
        match value.get("request") {
            Some(request) => Ok(Asked {
                request: serde_json::from_value(request.clone()).map_err(unread)?,
                before: match value.get("count") {
                    Some(count) => Some(
                        serde_json::from_value(serde_json::json!({
                            "count": count,
                            "counted_at": value.get("counted_at"),
                        }))
                        .map_err(unread)?,
                    ),
                    None => None,
                },
            }),
            None if value.get("count").is_some() || value.get("counted_at").is_some() => {
                Err(LanguageError::new(
                    crate::error::ErrorKind::Tree,
                    "this is no request and no route an answer printed: a route holds its request",
                ))
            }
            None => Ok(Asked {
                request: serde_json::from_value(value.clone()).map_err(unread)?,
                before: None,
            }),
        }
    }

    /// The answer, and beside it what the route said where the basis has
    /// moved since: both counts, each with its basis.
    pub fn answer(&self, corpus: &Corpus) -> Result<Answer, SearchError> {
        let mut answered = answer(corpus, &self.request)?;
        answered.before = self
            .before
            .clone()
            .filter(|before| before.counted_at != corpus.basis);
        Ok(answered)
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
    Explain(String),
}

/// One node of a query, and the query with it forced each way.
struct Explained {
    node: crate::tree::Pathed,
    forced: [Node; 2],
}

impl BoundView {
    /// The view of one query: for explain, the node its path names and
    /// the two transformed queries, each bound as any query is — a path
    /// the query lacks is an authoring error whose readings list the
    /// query's nodes with their paths, and a substitution the binder
    /// refuses is refused with the binder's own error.
    fn of(&self, query: &Query) -> Result<Option<Explained>, LanguageError> {
        let BoundView::Explain(path) = self else {
            return Ok(None);
        };
        let nodes = crate::tree::nodes(&query.root);
        let lacking = |nodes: Vec<crate::tree::Pathed>| {
            let message = if nodes.is_empty() {
                format!("the query has no node at `{path}`: the empty query has none")
            } else {
                format!(
                    "the query has no node at `{path}`: a path names a term, a group of terms, or a condition inside line( … ) or linked( … ), as this query numbers them"
                )
            };
            LanguageError::new(crate::error::ErrorKind::NoSuchPath, message).with_readings(
                nodes
                    .into_iter()
                    .map(|node| format!("{}  {}", node.path, node.text))
                    .collect(),
            )
        };
        let (Some(forced_true), Some(forced_false)) = (
            crate::tree::forced(&query.root, path, true),
            crate::tree::forced(&query.root, path, false),
        ) else {
            return Err(lacking(nodes));
        };
        let Some(node) = nodes.iter().find(|node| node.path == *path).cloned() else {
            return Err(lacking(nodes));
        };
        bind::bind(&forced_true)?;
        bind::bind(&forced_false)?;
        Ok(Some(Explained {
            node,
            forced: [forced_true, forced_false],
        }))
    }
}

impl View {
    /// The view an adapter's flat arguments name: rows unless a count, a
    /// crossed table or an explained path is asked for, and never two of
    /// them. A sort orders rows and a sum stands beside a count, so either
    /// with the other view is an authoring error and not a flag ignored;
    /// explain shows counts and no rows, so beside a count, a crossed
    /// table or a sort it is one too. A limit bounds what a view lists,
    /// and explain lists nothing: the one an adapter always sends is no
    /// conflict.
    pub fn of(
        count: Option<Vec<String>>,
        cross: Option<Vec<String>>,
        sum: Option<String>,
        explain: Option<String>,
        rows: Rows,
    ) -> Result<View, LanguageError> {
        let view_error =
            |message: &str| Err(LanguageError::new(crate::error::ErrorKind::View, message));
        if let Some(path) = explain {
            let beside = [
                ("--count", count.is_some()),
                ("--cross", cross.is_some()),
                ("--sum", sum.is_some()),
                ("--sort", rows.sort.is_some()),
            ];
            if let Some((flag, _)) = beside.iter().find(|(_, typed)| *typed) {
                return view_error(&format!(
                    "`--explain` shows the root's counts with one node forced true and forced false, and no rows: `{flag}` has nothing to stand beside — an answer has one view"
                ));
            }
            return Ok(View::Explain(Explain { path }));
        }
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
            View::Explain(explain) => BoundView::Explain(explain.path.clone()),
        })
    }
}

// ---- the answer ---------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    pub query: QueryOut,
    /// Under the corpus context, the query typed beside it: read, set
    /// aside, and counted nowhere.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_aside: Option<QueryOut>,
    pub scope: ScopeOut,
    pub basis: Basis,
    /// What a route said when it was printed, where it is run on a later
    /// basis (`Asked::answer`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Before>,
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
    /// Of an `id:` term that matches nothing live: that the id is among
    /// what left, when, and the route to it under membership `all`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<Removed>,
}

/// An id the scope held and holds no longer: a handle from an earlier
/// answer never reads as an item that did not exist.
#[derive(Debug, Clone, Serialize)]
pub struct Removed {
    /// `item`, `tab` or `character`.
    pub what: &'static str,
    pub removed_at: i64,
    /// Under `live`, the request that finds it under `all`. None under
    /// `all`, where a retired tab or character that matches nothing held
    /// no item.
    #[serde(flatten)]
    pub route: Option<Route>,
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
    /// Why items are undecided at the root: one entry per distinct reason
    /// — by what was unread, the kind a count tallies (C105) — with one
    /// example item, the first met; the first ten, the rest counted (the
    /// module doc, V8).
    pub undecided_reasons: Vec<UndecidedReason>,
    #[serde(skip_serializing_if = "is_zero")]
    pub reasons_left_out: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct UndecidedReason {
    /// The example's own: its problem names the item's part.
    #[serde(flatten)]
    pub reason: Reason,
    pub example: Example,
}

/// An item named where a whole row is too much.
#[derive(Debug, Clone, Serialize)]
pub struct Example {
    pub id: String,
    pub name: Option<String>,
}

/// The view as answered: the rows' bounds, the tables (`counts.rs`), or
/// the node explained.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewOut {
    Rows(RowsOut),
    Counts(counts::CountsOut),
    Cross(counts::CrossOut),
    Explain(Box<ExplainOut>),
}

/// One node explained (the reference, *Explain*): the root's matched and
/// undecided counts as given, with the node forced true and with it
/// forced false, each forced pair beside its transformed query. The
/// words are literal: forced, never removed, never a contribution.
#[derive(Debug, Clone, Serialize)]
pub struct ExplainOut {
    pub path: String,
    /// The node, as the canonical text prints it.
    pub node: String,
    pub given: Given,
    pub forced_true: Forced,
    pub forced_false: Forced,
}

/// The root's counts as the query was given: the answer's own total.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Given {
    pub matched: usize,
    pub undecided: usize,
}

/// The root's counts of the transformed query, each with its route: the
/// transformed query, asked as a request of its own.
#[derive(Debug, Clone, Serialize)]
pub struct Forced {
    pub query: QueryOut,
    pub matched: Count,
    pub undecided: Count,
}

/// A continuation (C100): the request that returns the rows after
/// these, naming the basis, the query, the view and the order it
/// continues, and refused across a change (C98).
#[derive(Debug, Clone, Serialize)]
pub struct Next {
    pub token: String,
    pub request: Request,
}

#[derive(Debug, Clone, Serialize)]
pub struct RowsOut {
    pub limit: usize,
    pub returned: usize,
    /// Where this answer's rows begin in the order: a continuation's
    /// place, absent on a first answer.
    #[serde(skip_serializing_if = "is_zero")]
    pub after: usize,
    /// Matching items past these rows.
    pub left_out: usize,
    /// How to go on, where rows are left out: the continuation as a
    /// request, and as the token a terminal carries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<Next>,
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
    /// When the item left, under membership `all`: a fetch of its place
    /// no longer had it, or the place was retired.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed_at: Option<i64>,
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

/// An item's sort scalar, or why it has none (C92), and what it sorts by
/// (F6, the first seat): the field with its name, the occurrence whose
/// slot is the largest, a sum's contributors, a computed value's inputs —
/// as a matched term's `shows` are, bounded the same way.
#[derive(Debug, Clone, Serialize)]
pub struct Sorted {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Json>,
    /// `no satisfying occurrence`, or `incomplete` beside what was readable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shows: Vec<Evidence>,
    #[serde(skip_serializing_if = "is_zero")]
    pub left_out: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Zero {
    /// Selectors nothing in this scope carries, with what shares their
    /// words.
    pub resolved_to_nothing: Vec<Nothing>,
    pub said: &'static str,
    pub undecided: Count,
    /// What the reference offers here (*When the total is zero*): each a
    /// request, never its output.
    pub offers: Offers,
}

/// What a zero answer offers: explain of each part of the query — a
/// child of the root — the part fewest items match first and by path
/// where two are equal, the first ten and the rest counted (invariant
/// 5); then the corpus context, where a query was typed.
#[derive(Debug, Clone, Serialize)]
pub struct Offers {
    pub explain: Vec<ExplainOffer>,
    #[serde(skip_serializing_if = "is_zero")]
    pub explain_left_out: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Offer>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExplainOffer {
    pub path: String,
    pub node: String,
    pub request: Request,
}

#[derive(Debug, Clone, Serialize)]
pub struct Offer {
    pub request: Request,
}

#[derive(Debug, Clone, Serialize)]
pub struct Nothing {
    pub path: String,
    pub term: String,
    pub of: String,
    pub suggestions: Vec<Suggestion>,
    /// Why nothing could carry it, where that is not S107's: an `id:`
    /// that is no whole id ([`ID_WHOLE`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'static str>,
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
    if request.scope.membership != corpus.membership.into() {
        return Err(SearchError::scope(
            "membership_mismatch",
            format!(
                "this corpus holds {} items and the request names membership `{}`",
                if corpus.membership == acquisition_store::Membership::All {
                    "all"
                } else {
                    "live"
                },
                if request.scope.membership == Membership::All {
                    "all"
                } else {
                    "live"
                },
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
    let (query, set_aside) = request.asked()?;
    let view = request.view.bound()?;
    let explained = view.of(&query)?;
    let typed = set_aside.as_ref().unwrap_or(&query).text();
    let resolved = Scope {
        account: Some(
            corpus
                .account_name
                .clone()
                .unwrap_or_else(|| corpus.basis.account.clone()),
        ),
        realm: Some(corpus.realm.clone()),
        membership: corpus.membership.into(),
    };
    // a continuation is a place in one basis's order: across a change it
    // is refused, both named, and the way on is the request from its start
    let after = match request.place()? {
        Some((_, basis)) if *basis != corpus.basis => {
            let mut start = request.clone();
            start.scope = resolved;
            start.query = QueryInput {
                text: Some(typed),
                tree: None,
            };
            start.basis = None;
            if let View::Rows(rows) = &mut start.view {
                rows.after = None;
            }
            return Err(SearchError::scope(
                "basis_changed",
                format!(
                    "this continues an answer counted at another basis, and its place is no place in this one — {}: ask again from the start",
                    basis.against(&corpus.basis)
                ),
            )
            .with_offers(vec![start.command()]));
        }
        Some((after, _)) => after,
        None => 0,
    };

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
            membership: corpus.membership.into(),
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
    let mut reasons: Vec<UndecidedReason> = Vec::new();
    let mut n_undecided = 0usize;
    let mut outcomes: Vec<Outcome> = Vec::with_capacity(n_terms);
    // the query's parts — the children of the root, a root that is no and
    // its one part — and how many items each holds on: what a zero answer
    // orders its offers by, counted while nothing has matched
    let parts = eval::parts(&query.bound);
    let mut part_matched = vec![0usize; parts.len()];
    for (at, held) in corpus.items.iter().enumerate() {
        eval::asked(&query.terms, held, &mut outcomes);
        if matches.is_empty() {
            for (n, part) in parts.iter().enumerate() {
                if eval::truth(part, &outcomes) == Truth::True {
                    part_matched[n] += 1;
                }
            }
        }
        for (i, term) in query.terms.iter().enumerate() {
            let outcome = outcomes[i];
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
                // one entry per kind of reason, the first item met its
                // example (V8): the whole list, so the rest can be counted
                let mut blamed = Vec::new();
                eval::blame(&query.bound, &outcomes, &mut blamed);
                for reason in eval::distinct_reasons(&query.terms, &blamed, held) {
                    if !reasons.iter().any(|r| r.reason.unread == reason.unread) {
                        reasons.push(UndecidedReason {
                            reason,
                            example: Example {
                                id: held.item.facts.id.clone(),
                                name: label(held),
                            },
                        });
                    }
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
                removed: match &term.atom {
                    Atom::Id(id) if tally[Outcome::Matched as usize] == 0 => {
                        corpus.gone.get(id).map(|left| Removed {
                            what: match left.kind.as_str() {
                                "item" => "item",
                                "stash" => "tab",
                                _ => "character",
                            },
                            removed_at: left.removed_at,
                            route: (corpus.membership == acquisition_store::Membership::Live).then(
                                || {
                                    let mut route = router.route(&term.node, None);
                                    route.request.scope.membership = Membership::All;
                                    route
                                },
                            ),
                        })
                    }
                    _ => None,
                },
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
                        // every pick of a category term is printed: the
                        // ids are a closed set, and no count lists the rest
                        let listed = match &term.atom {
                            Atom::Closed {
                                thing: Thing::Category,
                                ..
                            } => values.len(),
                            _ => LISTED,
                        };
                        let more = values.len().saturating_sub(listed);
                        values.truncate(listed);
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
    let root_undecided = (!is_empty(&query.root))
        .then(|| Node::Undecided(Probe::Term(Box::new(query.root.clone()))));
    let reasons_left_out = reasons.len().saturating_sub(LISTED);
    reasons.truncate(LISTED);
    let total = Total {
        matched: matches.len(),
        undecided: count(n_undecided, root_undecided.as_ref()),
        undecided_reasons: reasons,
        reasons_left_out,
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
        // a count shows no rows (AQ1), and explain none
        BoundView::Counts(_) | BoundView::Cross(_) | BoundView::Explain(_) => (None, false, 0),
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
        .skip(after)
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
                removed_at: held.item.facts.removed_at,
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
                sort: scalars.as_ref().zip(sort).map(|(s, (_, key))| {
                    let (shows, left_out) = eval::sorted_by(key, held);
                    let (value, status) = match s[m] {
                        Scalar::Value(n) => (Some(eval::number_json(n.as_f64())), None),
                        Scalar::None => (None, Some("no satisfying occurrence")),
                        Scalar::Incomplete(n) => {
                            (n.map(|n| eval::number_json(n.as_f64())), Some("incomplete"))
                        }
                    };
                    Sorted {
                        value,
                        status,
                        shows,
                        left_out,
                    }
                }),
            }
        })
        .collect();

    // explain shows counts in place of rows, a zero among them: the zero
    // block is the other views'
    let zero = (total.matched == 0 && explained.is_none()).then(|| Zero {
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
        offers: offers(&router, &query, &part_matched, request),
    });
    let left_out = total.matched.saturating_sub(after + rows.len());
    let view = match &view {
        BoundView::Rows { .. } => ViewOut::Rows(RowsOut {
            limit,
            returned: rows.len(),
            after,
            left_out,
            // a limit of nothing returns nothing from any place: the way
            // on is a larger limit, never a continuation that stands still
            next: (left_out > 0 && limit > 0).then(|| {
                let request = Request {
                    scope: resolved.clone(),
                    query: QueryInput {
                        text: Some(typed.clone()),
                        tree: None,
                    },
                    view: View::Rows(Rows {
                        limit: Some(limit),
                        sort: sort.map(|(text, _)| text.clone()),
                        desc,
                        after: Some(after + rows.len()),
                    }),
                    context: request.context,
                    basis: Some(corpus.basis.clone()),
                };
                Next {
                    token: request.token(),
                    request,
                }
            }),
            sort: sort.map(|(text, _)| text.clone()),
            desc,
        }),
        BoundView::Counts(bound) => ViewOut::Counts(counts::tables(bound, &counted)),
        BoundView::Cross(bound) => ViewOut::Cross(counts::crossed(bound, &counted)),
        BoundView::Explain(_) => {
            let Some(Explained {
                node,
                forced: [forced_true, forced_false],
            }) = explained
            else {
                // `BoundView::of` gives an explain view its node or its error
                return Err(SearchError::Language(LanguageError::new(
                    crate::error::ErrorKind::NoSuchPath,
                    "the explained node was not resolved",
                )));
            };
            ViewOut::Explain(Box::new(ExplainOut {
                path: node.path,
                node: node.text,
                given: Given {
                    matched: total.matched,
                    undecided: total.undecided.count,
                },
                forced_true: forced(corpus, &router, request, forced_true)?,
                forced_false: forced(corpus, &router, request, forced_false)?,
            }))
        }
    };
    Ok(Answer {
        before: None,
        query: QueryOut {
            text: query.text(),
            tree: json::to_json(&query.root),
        },
        set_aside: set_aside.map(|typed| QueryOut {
            text: typed.text(),
            tree: json::to_json(&typed.root),
        }),
        scope: ScopeOut {
            account: Account {
                uuid: corpus.basis.account.clone(),
                name: corpus.account_name.clone(),
            },
            realm: corpus.realm.clone(),
            membership: corpus.membership.into(),
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

/// The root's counts of a transformed query: the query bound and answered
/// as any query is, by the one evaluator — this function asks it as the
/// request it is, and reads its total (the reference, *Outside the first
/// surface*: "by the normal evaluator").
fn forced(
    corpus: &Corpus,
    router: &Router<'_>,
    request: &Request,
    transformed: Node,
) -> Result<Forced, SearchError> {
    let tree = json::to_json(&transformed);
    let asked = answer(
        corpus,
        &Request {
            scope: request.scope.clone(),
            query: QueryInput {
                text: None,
                tree: Some(tree.clone()),
            },
            view: View::Rows(Rows {
                limit: Some(0),
                ..Rows::default()
            }),
            context: Context::Matches,
            basis: None,
        },
    )?;
    let undecided = Node::Undecided(Probe::Term(Box::new(transformed.clone())));
    Ok(Forced {
        query: QueryOut {
            text: print::print(&transformed),
            tree,
        },
        matched: router.count(asked.total.matched, Some(&transformed)),
        undecided: router.count(asked.total.undecided.count, Some(&undecided)),
    })
}

/// What a zero answer offers (the module doc): explain of each part, in
/// the order of what each matched and then of its path, cut after the
/// order; then the corpus context beside the view the request had —
/// rows, where it had explain, which the empty query has no path for.
fn offers(router: &Router<'_>, query: &Query, part_matched: &[usize], request: &Request) -> Offers {
    let nodes: Vec<&Node> = match &query.root {
        Node::All(children) => children.iter().collect(),
        root => vec![root],
    };
    let mut parts: Vec<(usize, usize, &Node)> = nodes
        .into_iter()
        .enumerate()
        .map(|(n, node)| (part_matched.get(n).copied().unwrap_or(0), n, node))
        .collect();
    parts.sort_by_key(|(matched, n, _)| (*matched, *n));
    let explain_left_out = parts.len().saturating_sub(LISTED);
    parts.truncate(LISTED);
    Offers {
        explain: parts
            .into_iter()
            .map(|(_, n, node)| {
                let path = crate::tree::child_path("", n);
                ExplainOffer {
                    node: print::print(node),
                    request: router.request(
                        router.root,
                        None,
                        View::Explain(Explain { path: path.clone() }),
                        Context::Matches,
                    ),
                    path,
                }
            })
            .collect(),
        explain_left_out,
        context: (!is_empty(&query.root)).then(|| Offer {
            request: router.request(
                router.root,
                None,
                match &request.view {
                    View::Explain(_) => View::default(),
                    // a place is a place in the answer continued, never
                    // in the corpus's
                    View::Rows(rows) => View::Rows(Rows {
                        after: None,
                        ..rows.clone()
                    }),
                    view => view.clone(),
                },
                Context::Corpus,
            ),
        }),
    }
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
    /// A request over the answer's scope: what a route carries, and what
    /// a zero answer offers.
    fn request(&self, node: &Node, realm: Option<&str>, view: View, context: Context) -> Request {
        let mut scope = self.scope.clone();
        if let Some(realm) = realm {
            scope.realm = Some(Realm::One(realm.to_string()));
        }
        Request {
            scope,
            query: QueryInput {
                text: Some(print::print(node)),
                tree: None,
            },
            view,
            context,
            basis: None,
        }
    }

    fn route(&self, node: &Node, realm: Option<&str>) -> Route {
        Route {
            counted_at: self.corpus.basis.clone(),
            denominator: "scope",
            request: self.request(node, realm, View::default(), Context::Matches),
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
        (Atom::Has(_) | Atom::HasComputed(_), _) => (None, Some(not(node.clone())), None),
        // a category term failed on an item that has a category or whose
        // category is open (`category.rs`): what it lacked is `-has:` alone
        (
            Atom::Closed {
                thing: Thing::Category,
                ..
            },
            Node::Test { field, .. },
        ) => {
            let has = Node::Has(field.clone());
            let had_or_open = Node::Any(vec![
                has.clone(),
                Node::Undecided(Probe::Thing(ValueRef::Field(field.clone()))),
            ]);
            (
                Some(Node::All(vec![had_or_open, not(node.clone())])),
                Some(not(has)),
                None,
            )
        }
        // a class never lacks (`class.rs`), and `has:class` is open on an
        // item whose base is under several classes while `class:X` on it
        // may be false (F2): the failed route is the not alone
        (
            Atom::Closed {
                thing: Thing::Class,
                ..
            },
            _,
        ) => (Some(not(node.clone())), None, None),
        // a link group's member is one of the item's link groups: an item
        // with none lacks it
        (Atom::Links(_), _) => {
            let has = Node::Has("links".to_string());
            (
                Some(Node::All(vec![has.clone(), not(node.clone())])),
                Some(not(has)),
                None,
            )
        }
        // a computed value lacked is `-has:` of it: a derived field (T2),
        // a total of nothing (C94) — of the value, never of a number of
        // it: `has:` takes no slot word, and a ranged total asked by one
        // is lacked where the total is (J31)
        (Atom::Pseudo { name, slot, .. }, _) => {
            let whole = slot
                .as_deref()
                .and_then(|slot| name.strip_suffix(slot)?.strip_suffix('.'))
                .unwrap_or(name);
            let has = Node::Has(whole.to_string());
            (
                Some(Node::All(vec![has.clone(), not(node.clone())])),
                Some(not(has)),
                None,
            )
        }
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
        // no count lists every id a category term picked: a count by
        // category lists an item under its most specific id alone
        Atom::Closed {
            thing: Thing::Category,
            ..
        } => None,
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
/// to, and its value failed — by its whole text, or by the name a quoted
/// template found it by, a row's among them (`Group::resolved`). A
/// field's is the value that matched.
fn resolved_values(term: &Term, held: &Held, outcome: Outcome) -> Vec<String> {
    let mut values: Vec<String> = match &term.atom {
        Atom::Lines(group) | Atom::Sum { group, .. } => eval::selected(held, group)
            .flat_map(|l| group.resolved(l))
            .map(str::to_string)
            .collect(),
        Atom::Text { thing, test } if outcome == Outcome::Matched => eval::texts(held, *thing)
            .into_iter()
            .filter(|v| test.holds(v))
            .map(str::to_string)
            .collect(),
        // the picked ids that hold on the item (`category.rs`)
        Atom::Closed {
            thing: Thing::Category,
            ids,
            ..
        } if outcome == Outcome::Matched => crate::category::tables()
            .map(|tables| tables.holding(&held.category, *ids))
            .unwrap_or_default()
            .into_iter()
            .map(str::to_string)
            .collect(),
        // what the closed set's selector picked of the item's values — an
        // item that is each of two classes carries one the term did not
        // pick (the rule of 2026-09-26; `class.rs`)
        Atom::Closed { thing, values, .. } if outcome == Outcome::Matched => {
            eval::texts(held, *thing)
                .into_iter()
                .filter(|v| values.iter().any(|legal| legal.eq_ignore_ascii_case(v)))
                .map(str::to_string)
                .collect()
        }
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
            // an id is whole, and one that matched nothing says so (F7):
            // no value shares words with it
            (_, Atom::Id(_)) if matched == 0 => {
                return Some(Nothing {
                    path: term.path.clone(),
                    term: print::print(&term.node),
                    of: "id".to_string(),
                    suggestions: Vec::new(),
                    // an id that left is whole, and the term says where
                    note: match &term.atom {
                        Atom::Id(id) if corpus.gone.contains_key(id) => None,
                        _ => Some(ID_WHOLE),
                    },
                });
            }
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
            // a template's term names a mod by a row too (C90)
            let carries = |held: &Held| match thing {
                Some(_) => values_of(held).iter().any(|v| test.holds(v)),
                None => held
                    .item
                    .lines
                    .iter()
                    .any(|l| l.names().any(|name| test.holds(name))),
            };
            suggestions.push(Suggestion {
                term: print::print(&exactly(&of, &candidate.value)),
                value: candidate.value,
                items: corpus.items.iter().filter(|held| carries(held)).count(),
            });
            offered.push(test);
        }
    }
    Some(Nothing {
        path: term.path.clone(),
        term: print::print(&term.node),
        suggestions,
        of,
        note: None,
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
    command_with(verb, account, realm, last, "")
}

/// [`command`] with the flags a view and a context add: after the query,
/// as a reader types them — and before it where the query goes after
/// `--`, past which a terminal reads no flag.
fn command_with(
    verb: &str,
    account: Option<&str>,
    realm: Option<&str>,
    last: &str,
    flags: &str,
) -> String {
    let mut out = opening(verb, account, realm);
    if last.starts_with('-') {
        out.push_str(flags);
        out.push_str(" -- ");
        out.push_str(&shell_quoted(last));
    } else {
        out.push(' ');
        out.push_str(&shell_quoted(last));
        out.push_str(flags);
    }
    out
}

/// How every command the search prints opens: the verb, the account it
/// was answered for, the realm.
fn opening(verb: &str, account: Option<&str>, realm: Option<&str>) -> String {
    let mut out = format!("acq {verb}");
    if let Some(account) = account {
        out.push_str(&format!(" --account {}", shell_quoted(account)));
    }
    if let Some(realm) = realm {
        out.push_str(&format!(" --realm {realm}"));
    }
    out
}

impl Next {
    /// The continuation as a command a terminal takes: the token whole,
    /// and the account it was answered for (`command`).
    pub fn command(&self) -> String {
        // the realm is the token's to carry
        let mut out = opening("search", self.request.scope.account.as_deref(), None);
        out.push_str(&format!(" --next {}", shell_quoted(&self.token)));
        out
    }
}

impl Route {
    /// The route as a command a terminal takes ([`Request::command`]).
    pub fn command(&self) -> String {
        self.request.command()
    }
}

impl Request {
    /// The request as a command a terminal takes: the query, then the
    /// view where it is not the default rows, then the context where it
    /// is not the matches. A request that carries a tree and no text has
    /// none of its own: an answer returns the text.
    pub fn command(&self) -> String {
        let scope = &self.scope;
        let mut out = String::new();
        if scope.membership == Membership::All {
            out.push_str(" --membership all");
        }
        self.view_flags(&mut out);
        if self.context == Context::Corpus {
            out.push_str(" --context corpus");
        }
        command_with(
            "search",
            scope.account.as_deref(),
            scope.realm.as_ref().map(Realm::as_str),
            self.query.text.as_deref().unwrap_or_default(),
            &out,
        )
    }

    fn view_flags(&self, out: &mut String) {
        let (flag, counts) = match &self.view {
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
                return;
            }
            View::Explain(explain) => {
                out.push_str(&format!(" --explain {}", shell_quoted(&explain.path)));
                return;
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
