//! The effective price, joined read-only (C81, C100; the build plan, step
//! 9): what an item is priced as, as the pricing area's listing state
//! resolves it, made once there and read here — never a second reading of
//! a note, a tab name or a row (rule 10 of the plan).
//!
//! # Decisions as recorded
//!
//! - **C100 — `priced` is the effective price the pricing area defines
//!   (C81), joined read-only.** The search asks `acquisition-plan`'s
//!   `listing::resolve` over the store's pricing snapshot of each (realm,
//!   league) the corpus covers, and keeps of every item listing only its
//!   effective price ([`acquisition_plan::listing::Effective`]): the
//!   search writes no intent, reads no note for
//!   itself, and has no rule of its own for which statement wins. What the
//!   join needs of the store is the snapshot the pricing CLI already reads
//!   and the intent file's revision (`Annotations::revision`).
//! - **C81, read here.** A statement that carries a price — `exact` or
//!   `negotiable`, the game's or the owner's — is the item's price:
//!   `has:priced` holds, and `price.amount`, `price.currency` and
//!   `price.lot` are its parts. A listing resolved to `none`, `no_price`
//!   or `skip` is known absence (C93): `-has:priced` finds it, and every
//!   `price.*` term is lacked. `unresolved` — a row, or a note where the
//!   index could see one, that cannot be read sits where it could decide
//!   — is undecided with that reason, said once here at the item's grain
//!   (rule 8). A price's number is read under the crate's one rule for
//!   numbers (`exact::reads`: ten whole digits and four decimals): an
//!   amount or a lot the intent file holds beyond it is unread at that
//!   part — `has:priced` holds, `price.amount` is undecided — never a
//!   float that drops its last digits (the step-9 review: two prices
//!   past 2^53 in one bucket).
//!
//! # As built
//!
//! - **The join is over every (realm, league) the corpus's locations
//!   name**, one snapshot and one `resolve` each; an item's price is its
//!   own item listing's, by id. A league-less character is carried by the
//!   snapshot of every league of its realm; where its realm names no
//!   league at all, one snapshot under the empty league name — which no
//!   tab has — carries it, so its rows decide as they would (the step-9
//!   review). The snapshots are reads of their own,
//!   after the corpus's one transaction, so the facts revision and the
//!   intent revision are read again after them (`corpus.rs`): a corpus
//!   whose facts or intent moved between is read again, never labelled
//!   with a basis its prices do not belong to (C98).
//! - **`price.amount` is the number the price states**: a decimal price's
//!   decimal, a bulk ratio's `wanted` (C67); `price.lot` is the ratio's
//!   lot, and a decimal price lacks it — what the owner or the game wrote,
//!   never a lot of one invented (the plan's foot, P2, the builder's
//!   recommendation). `price.currency` is the tag of the currency table
//!   (C68), a closed set whose legal values the table's rows are.
//! - **What a row shows** of a price term: the price as the listing state
//!   prints it, the side that decided it, and the target it came from
//!   where that is above the item — its own note or row is the row's.
//! - **The basis** carries the intent revision and the versions the
//!   listing state was resolved under — the currency table's and the note
//!   parser's — beside the class and totals tables' (C98).

use std::collections::{BTreeSet, HashMap};

use acquisition_plan::listing::{Listing, ListingError, resolve};
use acquisition_store::corpus::LocationRow;
use acquisition_store::{Annotations, Store};
use serde::Serialize;

pub use acquisition_plan::currency::CURRENCY_TABLE_VERSION;
pub use acquisition_plan::game_side::NOTE_PARSER_VERSION;

use crate::derive::{Part, Unread};
use crate::error::SearchError;

/// C93's closed list of reasons: the kind a count tallies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceGap {
    /// A row that cannot be read sits where it could decide (C81).
    Unresolved,
    /// The item's note is no string, where the index could see one.
    NoteUnread,
    /// No listing state covered the item's place.
    Uncovered,
    /// The price's amount or lot is written longer than the search reads.
    NumberUnread,
}

impl PriceGap {
    /// What was unread, as a reason and a tally name it (`eval::reason`).
    pub fn unread(self) -> &'static str {
        match self {
            PriceGap::Unresolved => "the price: a row that could decide cannot be read",
            PriceGap::NoteUnread => "the price: the note cannot be read",
            PriceGap::Uncovered => "the price: no listing state covers the item's league",
            PriceGap::NumberUnread => "the price's number",
        }
    }

    /// What might resolve it — never a guarantee (C93): the intent file
    /// for a row, the body for a note, and for a number the search's own
    /// rule, which no refresh and no rewrite of the row within the rule
    /// changes.
    pub fn hint(self) -> &'static str {
        match self {
            PriceGap::Unresolved => {
                "a refresh will not help; `acq price show <target>` names the row, and a build that reads it or a rewrite of it resolves the price"
            }
            PriceGap::NoteUnread => crate::eval::UNREAD_HINT,
            PriceGap::Uncovered => {
                "a refresh of the realm's stash list would give its leagues a listing state; `acq price show <target>` resolves the item meanwhile"
            }
            PriceGap::NumberUnread => {
                "the search reads at most ten whole digits and four decimals; a price within them is read"
            }
        }
    }
}

/// What an item is priced as (C81), as the listing state answered.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Priced {
    /// The effective statement carries a price.
    Is(Box<PriceOf>),
    /// Resolved, and no price: `none`, `no_price` or `skip`.
    None { kind: String, why: String },
    /// Not established: why, in the deriver's shape of a reason.
    Open(Unread),
}

/// A price an item carries.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PriceOf {
    /// `exact` or `negotiable`.
    pub kind: String,
    /// The number the price states — a decimal, or a ratio's `wanted` —
    /// where it is within the search's rule for numbers; none where it is
    /// not, and `amount_unread` says so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount_unread: Option<Unread>,
    /// A ratio's lot, within the rule; a decimal price has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lot: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lot_unread: Option<Unread>,
    /// The currency table's tag (C68).
    pub currency: String,
    /// The price as the listing state prints it: `5 chaos`, `2/3 divine b/o`.
    pub text: String,
    /// `game` or `manual`.
    pub side: &'static str,
    /// The target the statement came from: `item/<id>`, `tab/pc/<id>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// The listing state's sentence naming the winner and the rule.
    pub why: String,
}

fn unread(gap: PriceGap, problem: impl Into<String>) -> Unread {
    Unread {
        part: Part::Price(gap),
        problem: problem.into(),
        line: None,
        name: None,
        socket: None,
    }
}

/// A price's number under the crate's rule (`exact::reads`): the float
/// the parser reads a typed bound as, or why it is not one.
fn number(text: &str, what: &str) -> (Option<f64>, Option<Unread>) {
    if crate::exact::reads(text) {
        (text.parse().ok(), None)
    } else {
        (
            None,
            Some(unread(
                PriceGap::NumberUnread,
                format!("the {what} `{text}` is written longer than the search reads a number"),
            )),
        )
    }
}

impl Priced {
    pub(crate) fn open(gap: PriceGap, problem: impl Into<String>) -> Priced {
        Priced::Open(unread(gap, problem))
    }

    fn uncovered() -> Priced {
        Priced::open(
            PriceGap::Uncovered,
            "no listing state covers this item's league, so its price was not resolved",
        )
    }

    /// The listing state's effective price read as the search's answer.
    /// The reason an item is unresolved is worded here from the listing's
    /// parts, never copied from its sentence, which names the item's own
    /// address: a row shows the same reason on an item and on its copy
    /// (the cross-checks, transformation 10).
    fn of(listing: &Listing) -> Priced {
        let effective = &listing.effective;
        if effective.kind == "unresolved" {
            if listing.game.note_unread {
                return Priced::open(
                    PriceGap::NoteUnread,
                    "the note cannot be read: it is no string, and a note would decide (C81)",
                );
            }
            let own = &listing.subject.target;
            let place = match &effective.from {
                Some(from) if from != own => from.to_string(),
                _ => "the item".to_string(),
            };
            // the listing says `the row on <target> cannot be read: <why>`;
            // the why is what a reader needs beside the place
            let detail = listing
                .manual_problem
                .as_deref()
                .and_then(|p| p.split_once("cannot be read: "))
                .map(|(_, why)| format!(" ({why})"))
                .unwrap_or_default();
            return Priced::open(
                PriceGap::Unresolved,
                format!("the row on {place} cannot be read{detail} and would decide (C81)"),
            );
        }
        let Some(price) = &effective.price else {
            return Priced::None {
                kind: effective.kind.clone(),
                why: effective.why.clone(),
            };
        };
        use acquisition_plan::price::Amount;
        // the canonical text, read as the parser reads a typed bound: one
        // float for one decimal (`exact.rs`), and none past the rule
        let ((amount, amount_unread), (lot, lot_unread)) = match price.amount {
            Amount::Decimal { .. } => (number(&price.amount.to_string(), "amount"), (None, None)),
            Amount::Ratio { wanted, lot } => (
                number(&wanted.to_string(), "amount"),
                match number(&lot.to_string(), "lot") {
                    (Some(_), None) => (Some(lot), None),
                    (_, unread) => (None, unread),
                },
            ),
        };
        Priced::Is(Box::new(PriceOf {
            kind: effective.kind.clone(),
            amount,
            amount_unread,
            lot,
            lot_unread,
            currency: price.currency.clone(),
            text: effective.to_string(),
            side: effective.side_word(),
            from: effective.from.as_ref().map(ToString::to_string),
            why: effective.why.clone(),
        }))
    }

    pub fn price(&self) -> Option<&PriceOf> {
        match self {
            Priced::Is(price) => Some(price.as_ref()),
            Priced::None { .. } | Priced::Open(_) => None,
        }
    }

    /// Why the price is not established, when it is not.
    pub fn unread(&self) -> Option<&Unread> {
        match self {
            Priced::Open(unread) => Some(unread),
            Priced::Is(_) | Priced::None { .. } => None,
        }
    }
}

/// The (realm, league) pairs a set of locations names: every stash tab's,
/// and every character's that has one. A league-less character is carried
/// by the snapshot of every league of its realm, so it needs no pair of
/// its own — unless its realm names none, and then one snapshot under the
/// empty league name, which no tab has, carries the realm's league-less
/// characters and nothing else (the module doc).
pub(crate) type Leagues = BTreeSet<(String, String)>;

pub(crate) fn leagues(locations: &[LocationRow]) -> Leagues {
    let mut leagues: Leagues = locations
        .iter()
        .filter_map(|l| l.league.clone().map(|league| (l.realm.clone(), league)))
        .collect();
    for l in locations
        .iter()
        .filter(|l| l.kind == "character" && l.league.is_none())
    {
        if !leagues.iter().any(|(realm, _)| *realm == l.realm) {
            leagues.insert((l.realm.clone(), String::new()));
        }
    }
    leagues
}

/// The prices of every item the listing state of these leagues covers, by
/// item id: one snapshot and one `resolve` per pair, read as the pricing
/// CLI reads them. An item two snapshots list — a league-less character's
/// — keeps the first answer, which is the same answer.
pub(crate) fn join(
    store: &Store,
    intent: &Annotations,
    leagues: &Leagues,
) -> Result<HashMap<String, Priced>, SearchError> {
    let mut prices = HashMap::new();
    for (realm, league) in leagues {
        let snapshot = store
            .pricing_snapshot(realm, league, intent)
            .map_err(SearchError::store)?;
        let report = resolve(&snapshot).map_err(listing_error)?;
        for listing in report.listings.iter().filter(|l| l.subject.is_item()) {
            let acquisition_plan::price::PriceTarget::Item { id } = &listing.subject.target else {
                continue;
            };
            prices
                .entry(id.clone())
                .or_insert_with(|| Priced::of(listing));
        }
    }
    Ok(prices)
}

/// An item's price from the join, or the reason it has none.
pub(crate) fn of_item(prices: &mut HashMap<String, Priced>, id: &str) -> Priced {
    prices.remove(id).unwrap_or_else(Priced::uncovered)
}

/// The intent file's revision, as the basis names it (C98).
pub(crate) fn intent_revision(intent: &Annotations) -> Result<i64, SearchError> {
    intent
        .revision()
        .map_err(|e| SearchError::store(anyhow::Error::new(e).context("reading the intent file")))
}

fn listing_error(e: ListingError) -> SearchError {
    match e {
        ListingError::Currency(e) => SearchError::scope(
            "currency_table",
            format!("the currency table this build ships does not load: {e}"),
        ),
        ListingError::UnknownRealm { realm } => SearchError::scope(
            "realm_unknown",
            format!("`{realm}` is no realm the pricing area knows"),
        ),
    }
}

/// The legal values of `price.currency`: the currency table's tags, in the
/// table's order; empty where the table does not load, which `bind.rs`
/// says instead of offering nothing.
pub(crate) fn currency_tags() -> &'static [&'static str] {
    static TAGS: std::sync::LazyLock<Vec<&'static str>> = std::sync::LazyLock::new(|| {
        acquisition_plan::currency::table()
            .map(|table| table.rows().iter().map(|c| c.tag.as_str()).collect())
            .unwrap_or_default()
    });
    &TAGS
}

/// Whether the currency table loads, said as `bind.rs` says it of a closed
/// set whose list is a table's.
pub(crate) fn currency_table() -> Result<(), String> {
    acquisition_plan::currency::table()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// What a row shows of a price term (C100): the price as printed, the side
/// that decided it and — where the statement sits above the item, on its
/// tab, substash or character — where; the item's own note or row is the
/// row's, which its id already names.
pub(crate) fn evidence(held: &crate::corpus::Held) -> Vec<crate::eval::Evidence> {
    use crate::eval::Evidence;
    let Some(price) = held.price.price() else {
        return Vec::new();
    };
    let own = format!("item/{}", held.item.facts.id);
    let value = |name: &str, value: serde_json::Value| Evidence::Value {
        name: name.to_string(),
        value,
    };
    let mut shows = vec![
        value("price", serde_json::Value::from(price.text.as_str())),
        value("price.side", serde_json::Value::from(price.side)),
    ];
    if let Some(from) = price.from.as_ref().filter(|from| **from != own) {
        shows.push(value("price.from", serde_json::Value::from(from.as_str())));
    }
    shows
}
