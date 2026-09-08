//! `shop render` (C74, C72): the forum shop page as a pure function of
//! the listing state ([`ListingReport`], C69/C81), the shipped currency
//! table (C68), a template and the policy table below. Built by the
//! pricing slice's plan step 6 (`PRICING-SLICE.md`, 2026-09-07). Nothing
//! here reads a store, contacts a daemon or sends anything: the CLI
//! (`acq shop render`, `shop_cmd.rs`) writes the pages to stdout and the
//! owner pastes them by hand (`SURFACES.md`, the forum row).
//!
//! # Decisions as recorded
//!
//! **C74 — `shop render` is in scope as a derivation, publishing isn't,
//! and every item the page omits is counted with a named reason.** The
//! page is a pure function of facts, intent, reference data, a template
//! and a ruled policy; it is written to stdout in pages labelled *n* of
//! *N*; it sends nothing; the template is an input, not stored. Nothing
//! is skipped silently: what the game already lists (C81) is omitted and
//! counted, an unruled cell is blocked, a prior post is never read (T6).
//! The policy's rows cite trade claims (T7, T13) and owner observations
//! (T11, T12), never its own output. *Why:* every human surface is a
//! derivation over a machine surface (pattern 3); posting stays parked
//! behind its own session. *Details:* `shop.rs`. *Pinned:* the `c74_`
//! tests. Amended 2026-09-06.
//!
//! **C72 — Pricing never edits the sync policy, and a price never locks a
//! tab into refresh.** The relationship between the two kinds of intent is
//! reported, not enforced: the consumer that needs freshness — `shop
//! render` first — names priced locations outside the policy's coverage,
//! priced facts older than its stated window, and items moved or reindexed
//! since the render's basis, each with the remedy (the policy edit, or the
//! `RefreshPlan` it would take, C41). *Why:* C++'s "priced tabs are always
//! refreshed" is one kind of intent silently rewriting another; a report
//! keeps the policy what its author wrote (07 §10, 08 boundary 7).
//! *Details:* `shop.rs`. *Pinned:* the `c72_` test. Ruled 2026-09-03;
//! amended 2026-09-04.
//!
//! # As built
//!
//! **One cell per item.** Every item listing of the report lands in
//! exactly one [`Cell`] of the policy table, read from the effective
//! outcome — `side` and `kind` together (the render handoff,
//! `PRICING-SLICE.md`) — and then from the item's address. A cell's
//! verdict is `post`, `omit`, `block` or `off_page`; the table's opening
//! rows are the two that post (a stash item at a listed tab, a character
//! item in a slot), everything unobserved is a blocked row citing the
//! open question it waits on (`docs/design/trade-ground-truth.md`, Q1–Q7),
//! and each row carries its count, so the page's omissions are the table
//! read down. The order of the reads is the order of [`cell`]: what the
//! effective price says (the game decides → omitted, since the site
//! already shows it and a page must not contradict it, C81; a hand `skip`
//! → omitted, as the word means; `no_price` → blocked on Q5; nothing →
//! off the page; unresolved → blocked, never treated as unpriced), then
//! what a hand price carries (a kind other than `exact` or `negotiable`
//! → blocked, since no row of the table says how it is written; a ratio
//! → Q6; a retired tag → C68), then whether the item can be addressed at
//! all (a league-less character's item is not evidence for this league;
//! the realm must be one the site lists — `pc`, `xbox`, `sony`, T4, and
//! any other word is blocked, not assumed; a socketed item has no
//! position, T13; a substash item's link is unobserved, Q3; a stash
//! item's tab must have been listed for its index, T13, and a recorded
//! index below zero is blocked as a corrupt fact, never ranked; a
//! character item needs its slot). Vocabularies are
//! matched explicitly, so a new kind or realm lands in a blocked cell
//! until a row is ruled for it (C74) — the table fails closed. A corrupt
//! timestamp in a fact saturates the age it yields; nothing here
//! panics on a store row (C47).
//!
//! **The link code and the spoiler title.** A stash item renders as
//! `[linkItem realm="<r>" location="Stash<n>" league="<L>" x="<x>"
//! y="<y>"]`, the attributes in the order the website's own link button
//! writes them (T7, T24), where `n` is the tab's one-based rank among
//! the tabs the website lists — the top-level tabs and folder children
//! in listing order, folders and substashes left out (T24: the site's
//! stash view numbers those 48 tabs 0–47 and its link says `Stash17`
//! for web index 16; the API's `index` counts the folder too, so the
//! C++ app's `index + 1` was one too high past a folder). A character
//! item renders as `[linkItem realm="<r>" location="<inventoryId>"
//! character="<name>" x="<x>" y="<y>"]` (T7, T8). The price
//! is the title of the spoiler the links sit in — `[spoiler=" ~price
//! <amount> <word>"]` or `[spoiler=" ~b/o <amount> <word>"]`, the C++
//! app's form (T15) with its leading space, the currency table's `emit`
//! word for the row's tag (C68) and the amount's canonical text (C67).
//! A `no_price` item posts as its link alone under an empty title
//! (`[spoiler=""]`, the C++ app's no-price row): the site lists it as
//! "No Price Set" (T21). The owner's post of 2026-09-07 is the evidence
//! the forum accepts this shape and that the site prices every item in
//! a spoiler by that spoiler's title (T22).
//!
//! **Grouping and pages.** Items with exactly the same price share one
//! spoiler, their links run together on its line: groups sort `~price`
//! before `~b/o` before no price, then by tag, then by amount, in a
//! stable sort over the report's order. Each page is one spoiler titled
//! `Shop Post <n> of <N> (<k> items)` holding the price spoilers one per
//! line; newlines stand only between spoiler tags (the owner's shape,
//! 2026-09-07). Pages are cut so that each page, the template around it
//! included, holds at most `size` characters (the C++ constant 50,000 is
//! the default, and the forum's hard limit, T23) — the page title is
//! reserved at its widest, so a page never grows past the size when its
//! numbers are filled in; a price group that runs across a cut is closed
//! and reopened on the next page; an entry that would not fit an empty
//! page is blocked (`page_size`). The template must hold `[items]`
//! exactly once; each page is the template with its spoiler in that
//! place.
//!
//! **The C72 report** is over the posted items only — an omitted or
//! blocked item asks nothing of a refresh. *Coverage:* each posted item's
//! container against the sync policy's selection for this (realm,
//! league) — a tab by its own id or its parent's, the planner's rule
//! (C37, [`Selection::covers_tab`]); a character by its id. *Staleness:*
//! a posted item in a covered container whose fact is older than the
//! policy's window (`max_age_seconds`), the same declaration the planner
//! refreshes by — so the refresh the line cites is the one that fetches
//! them; an uncovered item past the window is counted beside the
//! coverage line instead, since its remedy is the policy edit first.
//! *Positions:* a posted stash item whose fetch predates the stash
//! listing the page's `Stash<n>` comes from — the two halves of its link
//! were observed at different times, and a tab reindexed between them
//! would move the link (the "moved or reindexed" C72 names; without a
//! stored render basis, parked under "what did I last post", this is the
//! basis the render has). Each line names its remedy: the policy edit,
//! or the refresh — the caller passes the plan's request count, or why
//! it could not compile one ([`PolicySource::Set`]), so the text can say
//! what `acq refresh --plan` would send without this module compiling
//! one, and a plan that fails to compile is said, never swallowed.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use acquisition_core::realm::Realm;
use serde::{Deserialize, Serialize};

use crate::currency::{self, CurrencyTableError};
use crate::listing::{Listing, ListingReport, ReportHeader, Side};
use crate::price::{Amount, Price, PriceTarget};
use crate::{Selection, SyncPolicy};

/// The render's JSON shape; changes are additive (C53) until they are
/// not, and then this moves. The shape is pinned by
/// `reference/shop-render-schema-3.json`
/// (`the_render_json_matches_the_committed_fixture`). **3** since
/// 2026-09-07: the owner's page shape (spoilers) renamed `Posted.price`
/// to `title`; 2 measured staleness over the stale set only; neither 1
/// nor 2 reached a consumer.
pub const SHOP_SCHEMA: u32 = 3;

/// The forum's hard post limit in characters (T23); the C++ app's
/// constant (T15) had it right.
pub const DEFAULT_PAGE_SIZE: usize = 50_000;

/// The token a template holds exactly once; the default template is the
/// token alone.
pub const ITEMS_TOKEN: &str = "[items]";

/// The GGG tab type that groups tabs and holds no items.
const FOLDER: &str = "Folder";

/// What the table says to do with a cell's items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// On the page.
    Post,
    /// Left off by rule, counted.
    Omit,
    /// Left off because the cell is unobserved or the item cannot be
    /// addressed, counted; the row names what would unblock it.
    Block,
    /// Nothing applies to it: no row, no statement.
    OffPage,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Post => "post",
            Verdict::Omit => "omit",
            Verdict::Block => "block",
            Verdict::OffPage => "off_page",
        }
    }
}

/// The policy table's rows, in the order the reads are made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cell {
    GameLists,
    GameSkips,
    HandSkip,
    NothingApplies,
    Unresolved,
    LeagueUnknown,
    HandNoPrice,
    Ratio,
    RetiredCurrency,
    RealmUnlisted,
    Socketed,
    NoPosition,
    Substash,
    TabUnlisted,
    InvalidIndex,
    NoSlot,
    PageSize,
    UnruledKind,
    StashItem,
    CharacterItem,
}

impl Cell {
    pub const ALL: [Cell; 20] = [
        Cell::StashItem,
        Cell::CharacterItem,
        Cell::HandNoPrice,
        Cell::GameLists,
        Cell::GameSkips,
        Cell::HandSkip,
        Cell::NothingApplies,
        Cell::Unresolved,
        Cell::LeagueUnknown,
        Cell::UnruledKind,
        Cell::Ratio,
        Cell::RetiredCurrency,
        Cell::RealmUnlisted,
        Cell::Socketed,
        Cell::NoPosition,
        Cell::Substash,
        Cell::TabUnlisted,
        Cell::InvalidIndex,
        Cell::NoSlot,
        Cell::PageSize,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Cell::GameLists => "game_lists",
            Cell::GameSkips => "game_skips",
            Cell::HandSkip => "hand_skip",
            Cell::NothingApplies => "nothing_applies",
            Cell::Unresolved => "unresolved",
            Cell::LeagueUnknown => "league_unknown",
            Cell::HandNoPrice => "hand_no_price",
            Cell::Ratio => "ratio",
            Cell::RetiredCurrency => "retired_currency",
            Cell::RealmUnlisted => "realm_unlisted",
            Cell::Socketed => "socketed",
            Cell::NoPosition => "no_position",
            Cell::Substash => "substash",
            Cell::TabUnlisted => "tab_unlisted",
            Cell::InvalidIndex => "invalid_index",
            Cell::NoSlot => "no_slot",
            Cell::UnruledKind => "unruled_kind",
            Cell::PageSize => "page_size",
            Cell::StashItem => "stash_item",
            Cell::CharacterItem => "character_item",
        }
    }

    pub fn verdict(self) -> Verdict {
        match self {
            Cell::StashItem | Cell::CharacterItem | Cell::HandNoPrice => Verdict::Post,
            Cell::GameLists | Cell::GameSkips | Cell::HandSkip => Verdict::Omit,
            Cell::NothingApplies => Verdict::OffPage,
            Cell::Unresolved
            | Cell::LeagueUnknown
            | Cell::Ratio
            | Cell::RetiredCurrency
            | Cell::RealmUnlisted
            | Cell::Socketed
            | Cell::NoPosition
            | Cell::Substash
            | Cell::TabUnlisted
            | Cell::InvalidIndex
            | Cell::NoSlot
            | Cell::UnruledKind
            | Cell::PageSize => Verdict::Block,
        }
    }

    /// The rule and its citation, one sentence.
    pub fn why(self) -> &'static str {
        match self {
            Cell::StashItem => {
                "a hand-priced stash item at a listed tab: `[linkItem realm= location=\"Stash<n>\" league= x= y=]` under the price's spoiler, `n` the tab's rank among the tabs the website lists — folders and substashes left out (T7, T13, T24; a forum price over the tab's is T12)"
            }
            Cell::CharacterItem => {
                "a hand-priced character item in a slot: `[linkItem realm= location=\"<inventoryId>\" character= x= y=]` under the price's spoiler (T7, T8)"
            }
            Cell::GameLists => {
                "the game already lists it at its own price (T11, C81); a page must not contradict what the site shows (C74)"
            }
            Cell::GameSkips => {
                "the game marks it do-not-index (~skip, T10) and the game decides (C81); a forum price against a game skip is Q7"
            }
            Cell::HandSkip => "a hand skip means leave it out (C67)",
            Cell::NothingApplies => "no row applies and the game states nothing",
            Cell::Unresolved => {
                "a row that cannot be read could decide (C81): never treated as unpriced; read it with `acq price show`"
            }
            Cell::LeagueUnknown => {
                "the listing gave its character no league, so the report is not evidence it belongs to this one"
            }
            Cell::HandNoPrice => {
                "a hand `no_price` item at an address a posting row takes: its link alone under an empty spoiler title, which the site lists as \"No Price Set\" (T21)"
            }
            Cell::Ratio => {
                "a ratio on the forum is unobserved (Q6); in game a typed ratio unlists the item (T19)"
            }
            Cell::RetiredCurrency => {
                "the row cites a retired currency tag (C68): the word the site reads for it is unknown"
            }
            Cell::RealmUnlisted => "the trade site's realms are pc, xbox and sony (T4)",
            Cell::Socketed => "a socketed item has no position and cannot be linked (T13)",
            Cell::NoPosition => "the fetch gave the item no position, so no link can name it",
            Cell::Substash => {
                "the website's stash view offers no link for an item in a map or unique substash (T20), and no other code for one has been observed: blocked until one is"
            }
            Cell::TabUnlisted => {
                "the tab is not on the current stash listing, so it has no index for `Stash<n>` (T13)"
            }
            Cell::InvalidIndex => {
                "the tab's recorded index is below zero: a corrupt fact, blocked rather than ranked (C47)"
            }
            Cell::NoSlot => "a character item without an `inventoryId` cannot be addressed (T13)",
            Cell::UnruledKind => {
                "a hand price of a kind no row of this table writes (`exact` and `negotiable` are ruled; C74 blocks the rest)"
            }
            Cell::PageSize => "the entry alone, with the template around it, exceeds the page size",
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One row of the policy table as rendered: the cell, its verdict, the
/// rule, and how many items it took.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyRow {
    pub cell: Cell,
    pub verdict: Verdict,
    pub count: usize,
    pub why: String,
}

/// An item on the page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Posted {
    pub target: PriceTarget,
    pub label: String,
    pub location: PriceTarget,
    pub cell: Cell,
    /// 1-based.
    pub page: usize,
    /// The spoiler title the link sits under, as written: ` ~price 5
    /// chaos`, ` ~b/o 1/5 divine`, or empty for a `no_price` item.
    pub title: String,
    /// The link code, as written on the page.
    pub link: String,
}

/// An item not on the page, and the cell that decided it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeftOut {
    pub target: PriceTarget,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<PriceTarget>,
    pub cell: Cell,
    pub verdict: Verdict,
}

/// One page to paste.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    /// 1-based.
    pub number: usize,
    pub of: usize,
    /// Characters, the template included.
    pub chars: usize,
    pub items: usize,
    pub text: String,
}

/// What the render was told about the sync policy (C72): none is set,
/// the stored one cannot be read, or the policy at a revision — with the
/// request count of the `RefreshPlan` the caller compiled from it, if it
/// compiled one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicySource<'a> {
    NotSet,
    Unreadable {
        revision: i64,
        why: String,
    },
    Set {
        policy: &'a SyncPolicy,
        revision: i64,
        /// The request count of the `RefreshPlan` the caller compiled
        /// from this policy now, or why it could not.
        refresh: Result<u64, String>,
    },
}

/// The C72 report, over the posted items only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Freshness {
    /// `not_set`, `unreadable`, `league_not_covered` or `covers`.
    pub policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_revision: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_problem: Option<String>,
    /// The league's `max_age_seconds`, when the policy covers it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_seconds: Option<u32>,
    /// What `acq refresh --plan` would send now, when the caller
    /// compiled it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_requests: Option<u64>,
    /// Why the caller could not compile that plan, when it could not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_problem: Option<String>,
    /// Containers of posted items the policy does not cover.
    pub uncovered: Vec<PriceTarget>,
    /// Posted items in covered containers whose fact is older than the
    /// window — the ones the cited refresh fetches.
    pub stale: Vec<PriceTarget>,
    /// Posted items in uncovered containers past the window: the policy
    /// edit comes first for these.
    pub stale_uncovered: usize,
    /// The oldest age among `stale`, seconds — the covered items the
    /// cited refresh fetches, never an uncovered one (schema 2; schema 1
    /// measured every posted item under the name `oldest_seconds`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oldest_stale_seconds: Option<i64>,
    /// Posted stash items whose fetch predates the stash listing their
    /// `Stash<n>` comes from.
    pub position_before_listing: Vec<PriceTarget>,
}

/// The counts a first line needs.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ShopCounts {
    pub items: usize,
    pub posted: usize,
    pub omitted: usize,
    pub blocked: usize,
    pub off_page: usize,
    pub pages: usize,
    /// Items by cell; every cell present, zeros included.
    pub by_cell: BTreeMap<Cell, usize>,
}

/// The render: the contract `--json` prints and everything the text is a
/// function of (C53) — the listing header it was resolved from, the
/// inputs (size, template), the policy table with counts, every item's
/// disposition, the pages, and the C72 report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopRender {
    pub schema: u32,
    pub listing: ReportHeader,
    pub rendered_at: i64,
    pub size: usize,
    pub template: String,
    pub policy: Vec<PolicyRow>,
    pub counts: ShopCounts,
    pub posted: Vec<Posted>,
    pub left_out: Vec<LeftOut>,
    pub pages: Vec<Page>,
    pub freshness: Freshness,
}

/// What a render takes besides the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderOptions<'a> {
    /// Characters per page, template included.
    pub size: usize,
    /// The template; `[items]` exactly once.
    pub template: &'a str,
    pub now: i64,
    pub policy: PolicySource<'a>,
}

/// Why a page could not be rendered at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// The template does not hold `[items]` exactly once.
    Template {
        occurrences: usize,
    },
    /// The template alone does not fit the size.
    TemplateTooLarge {
        template_chars: usize,
        size: usize,
    },
    Currency(CurrencyTableError),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Template { occurrences } => write!(
                f,
                "the template must hold {ITEMS_TOKEN} exactly once ({occurrences} found)"
            ),
            RenderError::TemplateTooLarge {
                template_chars,
                size,
            } => write!(
                f,
                "the template alone is {template_chars} characters, over the page size {size}"
            ),
            RenderError::Currency(e) => write!(f, "currency table: {e}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// A candidate entry before pages are cut: the group it sorts into and
/// the text it puts on the page.
struct Entry {
    target: PriceTarget,
    label: String,
    location: PriceTarget,
    cell: Cell,
    /// Sort key: `~price` before `~b/o` before no price, then tag, then
    /// amount; also the group — equal keys share a spoiler.
    group: (u8, String, Option<Amount>),
    link: String,
    /// The spoiler title; empty for no price.
    title: String,
    /// Basis, for the C72 report.
    seen_response: Option<i64>,
    seen_at: Option<i64>,
    /// The tab's parent (a folder), for coverage.
    parent: Option<String>,
}

/// A tab's number in a forum link, or why it has none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StashNumber {
    /// One-based rank among the tabs the website lists.
    Rank(usize),
    /// A recorded index below zero.
    Invalid,
}

/// `Stash<n>` for every listed tab (T24): the website's stash view lists
/// the top-level tabs and folder children in listing order, folders and
/// substashes left out, numbered from 0, and its link writes that number
/// plus one. The API's `index` counts folders, so the rank is taken over
/// the listed tabs sorted by `index`, never `index + 1` itself.
fn stash_numbers(report: &ListingReport) -> HashMap<String, StashNumber> {
    let mut listed: Vec<(i64, &str, bool)> = report
        .listings
        .iter()
        .filter_map(|l| match (&l.subject.target, l.subject.index) {
            (PriceTarget::Tab { id, .. }, Some(index)) => Some((
                index,
                id.as_str(),
                l.subject.tab_type.as_deref() == Some(FOLDER),
            )),
            _ => None,
        })
        .collect();
    listed.sort_by_key(|(index, _, _)| *index);
    let mut out = HashMap::new();
    let mut rank = 0usize;
    for (index, id, folder) in listed {
        if index < 0 {
            out.insert(id.to_string(), StashNumber::Invalid);
        } else if !folder {
            rank += 1;
            out.insert(id.to_string(), StashNumber::Rank(rank));
        }
    }
    out
}

/// The cell an item falls in, and — when it posts — its entry.
fn cell(
    l: &Listing,
    report: &ListingReport,
    table: &currency::CurrencyTable,
    stash_numbers: &HashMap<String, StashNumber>,
) -> Result<Entry, Cell> {
    let s = &l.subject;
    let e = &l.effective;
    // The kinds a row of the table writes, matched by name: a kind this
    // build does not know how to write is blocked, never assumed exact.
    let (price, prefix, order): (Option<&Price>, &str, u8) = match (e.side, e.kind.as_str()) {
        (Some(Side::Game), "skip") => return Err(Cell::GameSkips),
        (Some(Side::Game), _) => return Err(Cell::GameLists),
        (Some(Side::Manual), "skip") => return Err(Cell::HandSkip),
        (Some(Side::Manual), "no_price") => (None, "", 2),
        (Some(Side::Manual), kind @ ("exact" | "negotiable")) => match &e.price {
            Some(p) => (
                Some(p),
                if kind == "exact" { " ~price" } else { " ~b/o" },
                u8::from(kind == "negotiable"),
            ),
            // A manual side with a priced kind carries its price; a
            // report that says otherwise is unreadable here.
            None => return Err(Cell::Unresolved),
        },
        (Some(Side::Manual), _) => return Err(Cell::UnruledKind),
        (None, "unresolved") => return Err(Cell::Unresolved),
        (None, _) => return Err(Cell::NothingApplies),
    };
    let (title, group) = match price {
        None => (String::new(), (order, String::new(), None)),
        Some(price) => {
            if matches!(price.amount, Amount::Ratio { .. }) {
                return Err(Cell::Ratio);
            }
            let word = match table.by_tag(&price.currency) {
                Some(row) if !row.is_retired() => row.emit.clone(),
                // A row cites a tag the table holds (C67's write rule); one
                // it does not is a newer table's, unknown to this build.
                _ => return Err(Cell::RetiredCurrency),
            };
            (
                format!("{prefix} {} {word}", price.amount),
                (order, price.currency.clone(), Some(price.amount)),
            )
        }
    };
    if s.league_unknown {
        return Err(Cell::LeagueUnknown);
    }
    // The realms the site lists (T4), by name: any other word is a
    // realm this table has no row for.
    let realm = report.header.realm;
    if !matches!(realm, Realm::Pc | Realm::Xbox | Realm::Sony) {
        return Err(Cell::RealmUnlisted);
    }
    if s.socketed_in.is_some() {
        return Err(Cell::Socketed);
    }
    let (Some(x), Some(y)) = (s.x, s.y) else {
        return Err(Cell::NoPosition);
    };
    let Some(location) = s.location.clone() else {
        return Err(Cell::NoPosition);
    };
    let (cell, link, parent) = match &location {
        PriceTarget::Substash { .. } => return Err(Cell::Substash),
        PriceTarget::Character { .. } => {
            let Some(character) = report.find(&location) else {
                return Err(Cell::NoSlot);
            };
            let Some(slot) = s.inventory_id.as_deref() else {
                return Err(Cell::NoSlot);
            };
            (
                Cell::CharacterItem,
                format!(
                    "[linkItem realm=\"{}\" location=\"{slot}\" character=\"{}\" x=\"{x}\" y=\"{y}\"]",
                    realm.as_str(),
                    character.subject.name
                ),
                None,
            )
        }
        PriceTarget::Tab { .. } | PriceTarget::Item { .. } => {
            let Some(tab) = report.find(&location) else {
                return Err(Cell::TabUnlisted);
            };
            // `Stash<n>` is the tab's rank among the tabs the website
            // lists (T24); a tab never listed has none, a negative index
            // is a corrupt fact.
            let PriceTarget::Tab { id, .. } = &location else {
                return Err(Cell::TabUnlisted);
            };
            let stash_n = match stash_numbers.get(id) {
                Some(StashNumber::Rank(n)) => *n,
                Some(StashNumber::Invalid) => return Err(Cell::InvalidIndex),
                None => return Err(Cell::TabUnlisted),
            };
            let parent = tab.chain.get(1).and_then(|t| match t {
                PriceTarget::Tab { id, .. } => Some(id.clone()),
                _ => None,
            });
            (
                Cell::StashItem,
                format!(
                    "[linkItem realm=\"{}\" location=\"Stash{stash_n}\" league=\"{}\" x=\"{x}\" y=\"{y}\"]",
                    realm.as_str(),
                    report.header.league
                ),
                parent,
            )
        }
    };
    // A no-price item's cell is the no-price row, once its address
    // passed the checks a posting row makes.
    let cell = if price.is_none() {
        Cell::HandNoPrice
    } else {
        cell
    };
    Ok(Entry {
        target: s.target.clone(),
        label: s.label(),
        location,
        cell,
        group,
        link,
        title,
        seen_response: l.basis.response,
        seen_at: l.basis.at,
        parent,
    })
}

/// Render the page set (module doc). Pure: the report, the shipped
/// currency table and the options in; the render out.
pub fn render(report: &ListingReport, opts: &RenderOptions<'_>) -> Result<ShopRender, RenderError> {
    let occurrences = opts.template.matches(ITEMS_TOKEN).count();
    if occurrences != 1 {
        return Err(RenderError::Template { occurrences });
    }
    let template_chars = opts.template.chars().count();
    let overhead = template_chars - ITEMS_TOKEN.chars().count();
    if overhead > opts.size {
        return Err(RenderError::TemplateTooLarge {
            template_chars,
            size: opts.size,
        });
    }
    let table = currency::table().map_err(RenderError::Currency)?;

    let mut counts = ShopCounts::default();
    for c in Cell::ALL {
        counts.by_cell.insert(c, 0);
    }
    let mut left_out = Vec::new();
    let mut take = |counts: &mut ShopCounts,
                    target: &PriceTarget,
                    label: String,
                    location: Option<PriceTarget>,
                    cell: Cell| {
        *counts.by_cell.entry(cell).or_default() += 1;
        match cell.verdict() {
            Verdict::Post => counts.posted += 1,
            Verdict::Omit => counts.omitted += 1,
            Verdict::Block => counts.blocked += 1,
            Verdict::OffPage => counts.off_page += 1,
        }
        if cell.verdict() != Verdict::Post {
            left_out.push(LeftOut {
                target: target.clone(),
                label,
                location,
                cell,
                verdict: cell.verdict(),
            });
        }
    };
    // Classify first; the page cutter's size check needs the candidate
    // count, since the page title is reserved at its widest.
    let numbers = stash_numbers(report);
    let mut candidates: Vec<Entry> = Vec::new();
    for l in report.listings.iter().filter(|l| l.subject.is_item()) {
        counts.items += 1;
        match cell(l, report, table, &numbers) {
            Ok(entry) => candidates.push(entry),
            Err(cell) => take(
                &mut counts,
                &l.subject.target,
                l.subject.label(),
                l.subject.location.clone(),
                cell,
            ),
        }
    }
    let cut = cut_pages(candidates, opts.template, opts.size);
    for e in &cut.too_large {
        take(
            &mut counts,
            &e.target,
            e.label.clone(),
            Some(e.location.clone()),
            Cell::PageSize,
        );
    }
    for e in &cut.entries {
        take(
            &mut counts,
            &e.target,
            e.label.clone(),
            Some(e.location.clone()),
            e.cell,
        );
    }
    counts.pages = cut.pages.len();

    let freshness = freshness(report, &cut.entries, opts);
    let policy = Cell::ALL
        .into_iter()
        .map(|c| PolicyRow {
            cell: c,
            verdict: c.verdict(),
            count: counts.by_cell.get(&c).copied().unwrap_or(0),
            why: c.why().to_string(),
        })
        .collect();
    Ok(ShopRender {
        schema: SHOP_SCHEMA,
        listing: report.header.clone(),
        rendered_at: opts.now,
        size: opts.size,
        template: opts.template.to_string(),
        policy,
        counts,
        posted: cut.posted,
        left_out,
        pages: cut.pages,
        freshness,
    })
}

/// What the page cutter returns ([`cut_pages`]).
struct Cut {
    /// The pages, numbered from 1, each within the size.
    pages: Vec<Page>,
    /// One record per entry on a page, in page order.
    posted: Vec<Posted>,
    /// The entries on the pages, in the same order — the set the C72
    /// report runs over.
    entries: Vec<Entry>,
    /// The candidates that would not fit an empty page (`page_size`),
    /// in their order.
    too_large: Vec<Entry>,
}

/// The page cutter (module doc, "Grouping and pages"): pure over the
/// candidates, the template and the size. An entry that would not fit
/// an empty page — the page spoiler reserved at its widest, the template
/// around it — is set aside; the rest sort into their groups (equal keys
/// share a spoiler) and are cut so that each page's text, the template
/// included, holds at most `size` characters, a group that runs across a
/// cut closed and reopened on the next page. The template holds `[items]`
/// once and its overhead is within `size` ([`render`] checks both before
/// calling). *Pinned:* `c74_any_entries_cut_into_pages_within_the_size…`
/// (the property test) and `c74_pages_are_cut_under_the_size…` (the boundaries).
fn cut_pages(candidates: Vec<Entry>, template: &str, size: usize) -> Cut {
    let overhead = template
        .chars()
        .count()
        .saturating_sub(ITEMS_TOKEN.chars().count());
    let fixed = page_fixed_chars(candidates.len(), overhead);
    let (mut entries, too_large): (Vec<Entry>, Vec<Entry>) =
        candidates.into_iter().partition(|entry| {
            group_open(&entry.title).chars().count()
                + GROUP_CLOSE.len()
                + entry.link.chars().count()
                + fixed
                <= size
        });
    entries.sort_by_cached_key(|e| e.group.clone());

    // Pages: a running character count with the page's fixed cost in
    // it; a group whose next link does not fit is closed here and
    // reopened on the next page.
    let mut pages: Vec<PageBuild> = Vec::new();
    let mut page = PageBuild::default();
    let mut posted = Vec::with_capacity(entries.len());
    for entry in &entries {
        let link_chars = entry.link.chars().count();
        let same_group = page.groups.last().is_some_and(|g| g.key == entry.group);
        let group_cost = if same_group {
            0
        } else {
            group_open(&entry.title).chars().count() + GROUP_CLOSE.len()
        };
        if page.items > 0 && page.chars + group_cost + link_chars + fixed > size {
            pages.push(std::mem::take(&mut page));
        }
        if page.groups.last().is_none_or(|g| g.key != entry.group) {
            page.chars += group_open(&entry.title).chars().count() + GROUP_CLOSE.len();
            page.groups.push(GroupBuild {
                key: entry.group.clone(),
                title: entry.title.clone(),
                links: Vec::new(),
            });
        }
        if let Some(g) = page.groups.last_mut() {
            g.links.push(entry.link.clone());
        }
        page.chars += link_chars;
        page.items += 1;
        posted.push(Posted {
            target: entry.target.clone(),
            label: entry.label.clone(),
            location: entry.location.clone(),
            cell: entry.cell,
            page: pages.len() + 1,
            title: entry.title.clone(),
            link: entry.link.clone(),
        });
    }
    if page.items > 0 {
        pages.push(page);
    }
    let of = pages.len();
    let pages: Vec<Page> = pages
        .into_iter()
        .enumerate()
        .map(|(i, page)| {
            let mut body = page_open(&(i + 1).to_string(), &of.to_string(), page.items);
            for g in &page.groups {
                body.push_str(&group_open(&g.title));
                for link in &g.links {
                    body.push_str(link);
                }
                body.push_str(GROUP_CLOSE);
            }
            body.push_str(PAGE_CLOSE);
            let text = template.replacen(ITEMS_TOKEN, &body, 1);
            Page {
                number: i + 1,
                of,
                chars: text.chars().count(),
                items: page.items,
                text,
            }
        })
        .collect();
    Cut {
        pages,
        posted,
        entries,
        too_large,
    }
}

/// A price spoiler's opening tag.
fn group_open(title: &str) -> String {
    format!("[spoiler=\"{title}\"]")
}

/// Closes a price spoiler; the newline stands between spoiler tags.
const GROUP_CLOSE: &str = "[/spoiler]\n";

/// Closes the page spoiler.
const PAGE_CLOSE: &str = "[/spoiler]\n";

/// The page spoiler's opening tag and its line.
fn page_open(n: &str, of: &str, items: usize) -> String {
    format!(
        "[spoiler=\"Shop Post {n} of {of} ({items} {})\"]\n",
        if items == 1 { "item" } else { "items" }
    )
}

/// What every page costs before its groups: the page spoiler at its
/// widest (every number as many digits as the candidate count has),
/// its close, and the template around it — so filling the numbers in
/// never grows a page past the size.
fn page_fixed_chars(candidates: usize, template_overhead: usize) -> usize {
    let widest = "9".repeat(candidates.max(1).to_string().len());
    let items_widest: usize = widest.parse().unwrap_or(9);
    page_open(&widest, &widest, items_widest).chars().count() + PAGE_CLOSE.len() + template_overhead
}

/// One price spoiler under construction.
struct GroupBuild {
    key: (u8, String, Option<Amount>),
    title: String,
    links: Vec<String>,
}

/// One page under construction: its groups, its character count (the
/// groups' tags and links; the page's fixed cost is added at the cut),
/// its items.
#[derive(Default)]
struct PageBuild {
    groups: Vec<GroupBuild>,
    chars: usize,
    items: usize,
}

/// The C72 report over the posted entries (module doc).
fn freshness(report: &ListingReport, entries: &[Entry], opts: &RenderOptions<'_>) -> Freshness {
    let mut out = Freshness {
        policy: "not_set".into(),
        policy_revision: None,
        policy_problem: None,
        window_seconds: None,
        refresh_requests: None,
        refresh_problem: None,
        uncovered: Vec::new(),
        stale: Vec::new(),
        stale_uncovered: 0,
        oldest_stale_seconds: None,
        position_before_listing: Vec::new(),
    };
    let listing_response = report.header.stash_listing.map(|b| b.response_id);
    for e in entries {
        if let (Some(seen), Some(listing)) = (e.seen_response, listing_response)
            && matches!(e.location, PriceTarget::Tab { .. })
            && seen < listing
        {
            out.position_before_listing.push(e.target.clone());
        }
    }
    let league = match &opts.policy {
        PolicySource::NotSet => return out,
        PolicySource::Unreadable { revision, why } => {
            out.policy = "unreadable".into();
            out.policy_revision = Some(*revision);
            out.policy_problem = Some(why.clone());
            return out;
        }
        PolicySource::Set {
            policy,
            revision,
            refresh,
        } => {
            out.policy_revision = Some(*revision);
            match refresh {
                Ok(n) => out.refresh_requests = Some(*n),
                Err(why) => out.refresh_problem = Some(why.clone()),
            }
            match policy.league(report.header.realm, &report.header.league) {
                Some(league) => league,
                None => {
                    out.policy = "league_not_covered".into();
                    return out;
                }
            }
        }
    };
    out.policy = "covers".into();
    out.window_seconds = Some(league.max_age_seconds);
    let window = i64::from(league.max_age_seconds);
    let mut uncovered: BTreeSet<PriceTarget> = BTreeSet::new();
    for e in entries {
        let covered = match &e.location {
            PriceTarget::Character { id } => league
                .characters
                .as_ref()
                .is_some_and(|sel: &Selection| sel.covers_id(id)),
            PriceTarget::Tab { id, .. } => league
                .tabs
                .as_ref()
                .is_some_and(|sel| sel.covers_tab(id, e.parent.as_deref())),
            _ => false,
        };
        // A corrupt timestamp saturates (C47), as the planner's does.
        let age = e.seen_at.map(|at| opts.now.saturating_sub(at).max(0));
        let past_window = age.is_some_and(|a| a > window);
        if !covered {
            uncovered.insert(e.location.clone());
            if past_window {
                out.stale_uncovered += 1;
            }
        } else if past_window {
            out.stale.push(e.target.clone());
            // The oldest of the stale set, so the CLI's "oldest" belongs
            // to the items its remedy fetches (review round 2, 2026-09-07).
            if let Some(a) = age
                && out.oldest_stale_seconds.is_none_or(|o| a > o)
            {
                out.oldest_stale_seconds = Some(a);
            }
        }
    }
    out.uncovered = uncovered.into_iter().collect();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::listing::resolve;
    use acquisition_store::{
        AnnotationRow, CharacterSnapshot, ItemSnapshot, ListingBasis, PricingSnapshot, TabSnapshot,
    };
    use serde_json::{Value, json};

    fn tab(
        id: &str,
        parent: Option<&str>,
        name: &str,
        r#type: &str,
        public: bool,
        idx: Option<i64>,
    ) -> TabSnapshot {
        TabSnapshot {
            id: id.into(),
            parent: parent.map(str::to_string),
            name: name.into(),
            r#type: r#type.into(),
            idx,
            listed_at: Some(1_000),
            listed_response: Some(2),
            fetched_at: Some(1_100),
            metadata: if public {
                json!({ "public": true })
            } else {
                Value::Null
            },
            item_count: 0,
        }
    }

    fn item(id: &str, kind: &str, location: &str, x: i64, y: i64) -> ItemSnapshot {
        ItemSnapshot {
            id: id.into(),
            location_kind: kind.into(),
            location_id: location.into(),
            container: Some(
                if kind == "stash" {
                    "items"
                } else {
                    "equipment"
                }
                .into(),
            ),
            socketed_in: None,
            name: String::new(),
            type_line: "Chaos Orb".into(),
            stack_size: None,
            x: Some(x),
            y: Some(y),
            note: None,
            inventory_id: Some(
                if kind == "stash" {
                    "Stash1"
                } else {
                    "BodyArmour"
                }
                .into(),
            ),
            seen_response: Some(7),
            last_seen: 1_200,
        }
    }

    fn row(scope: &str, key: &str, value: Value) -> AnnotationRow {
        AnnotationRow {
            scope: scope.into(),
            key: key.into(),
            kind: "buyout".into(),
            value,
            revision: 3,
            created_at: 900,
            updated_at: 950,
            written_via: "test".into(),
            actor: None,
        }
    }

    fn exact(amount: &str, currency: &str) -> Value {
        json!({ "version": 1, "type": "exact", "amount": amount, "currency": currency })
    }

    fn character(id: &str, name: &str, league: Option<&str>) -> CharacterSnapshot {
        CharacterSnapshot {
            id: id.into(),
            name: name.into(),
            league: league.map(str::to_string),
            listed_at: Some(1_001),
            listed_response: Some(3),
            fetched_at: Some(1_101),
            listed: Value::Null,
            fetched: Value::Null,
        }
    }

    /// Every cell of the table has an item: a folder holding a public
    /// priced tab (the game lists most of it; one item priced by hand
    /// beats the tab name, one skipped by hand, one noted so the game
    /// wins the tie, one socketed, one a ratio, one a retired tag, one
    /// no_price); a non-public priced tab whose row prices it by hand; a
    /// public unpriced tab with an item nothing applies to and one with
    /// an unreadable row; a map tab's substash; a tab never listed; a
    /// character with a worn item and a socketed gem; a league-less
    /// character with a priced item, an unpriced one and a skipped one.
    fn snapshot() -> PricingSnapshot {
        let mut items = vec![
            item("i-game", "stash", "c1", 0, 0),
            item("i-hand", "stash", "c1", 1, 0),
            item("i-hand2", "stash", "c1", 2, 0),
            item("i-bo", "stash", "c1", 3, 0),
            item("i-skip", "stash", "c1", 4, 0),
            item("i-noted", "stash", "c1", 5, 0),
            item("i-gem", "stash", "c1", 0, 0),
            item("i-ratio", "stash", "c1", 6, 0),
            item("i-retired", "stash", "c1", 7, 0),
            item("i-noprice", "stash", "c1", 8, 0),
            item("i-gameskip", "stash", "c1", 9, 0),
            item("i-tabrow", "stash", "t2", 0, 1),
            item("i-plain", "stash", "t3", 0, 0),
            item("i-unres", "stash", "t3", 1, 0),
            item("i-sub", "stash", "s1", 0, 0),
            item("i-unlisted", "stash", "u1", 0, 0),
            item("i-worn", "character", "ch1", 0, 0),
            item("i-worngem", "character", "ch1", 0, 0),
            item("i-nowhere", "character", "ch2", 0, 0),
            item("i-nowhere-plain", "character", "ch2", 1, 0),
            item("i-nowhere-skip", "character", "ch2", 2, 0),
        ];
        items[5].note = Some("~price 4 chaos".into());
        items[6].socketed_in = Some("i-game".into());
        items[6].x = None;
        items[6].y = None;
        items[6].inventory_id = None;
        items[10].note = Some("~skip".into());
        items[11].seen_response = Some(1);
        items[11].last_seen = 100;
        items[17].socketed_in = Some("i-worn".into());
        items[17].x = None;
        items[17].y = None;
        items[17].inventory_id = None;
        PricingSnapshot {
            account_uuid: "u-1".into(),
            account_name: Some("tom".into()),
            realm: "pc".into(),
            league: "Standard".into(),
            taken_at: 2_000,
            stash_listing: Some(ListingBasis {
                response_id: 2,
                fetched_at: 1_000,
            }),
            tabs: vec![
                tab("f1", None, "Sale", "Folder", false, Some(0)),
                tab(
                    "c1",
                    Some("f1"),
                    "~price 3 chaos",
                    "PremiumStash",
                    true,
                    Some(1),
                ),
                tab(
                    "t2",
                    None,
                    "~price 9 chaos (Remove-only)",
                    "PremiumStash",
                    false,
                    Some(2),
                ),
                tab("t3", None, "Plain", "PremiumStash", true, Some(3)),
                tab("m1", None, "Maps", "MapStash", false, Some(4)),
                tab("s1", Some("m1"), "1", "MapStash", false, Some(5)),
                tab("u1", None, "Fetched only", "PremiumStash", false, None),
            ],
            character_listing: Some(ListingBasis {
                response_id: 3,
                fetched_at: 1_001,
            }),
            characters: vec![
                character("ch1", "Exile", Some("Standard")),
                character("ch2", "Nowhere", None),
            ],
            items,
            buyouts: vec![
                row("item", "i-hand", exact("5", "chaos")),
                row("item", "i-hand2", exact("5", "chaos")),
                row(
                    "item",
                    "i-bo",
                    json!({ "version": 1, "type": "negotiable", "amount": "1.5", "currency": "divine" }),
                ),
                row("item", "i-skip", json!({ "version": 1, "type": "skip" })),
                row("item", "i-noted", exact("1", "divine")),
                row("item", "i-gem", exact("1", "chaos")),
                row("item", "i-ratio", exact("1/5", "divine")),
                row("item", "i-retired", exact("1", "silver")),
                row(
                    "item",
                    "i-noprice",
                    json!({ "version": 1, "type": "no_price" }),
                ),
                row("item", "i-gameskip", exact("2", "chaos")),
                row(
                    "tab",
                    "pc/t2",
                    json!({ "version": 1, "type": "negotiable", "amount": "2", "currency": "divine" }),
                ),
                row(
                    "item",
                    "i-unres",
                    json!({ "version": 9, "type": "auction" }),
                ),
                row("item", "i-sub", exact("1", "chaos")),
                row("item", "i-unlisted", exact("1", "chaos")),
                row("item", "i-worn", exact("10", "divine")),
                row("item", "i-worngem", exact("1", "chaos")),
                row("item", "i-nowhere", exact("1", "chaos")),
                row(
                    "item",
                    "i-nowhere-skip",
                    json!({ "version": 1, "type": "skip" }),
                ),
            ],
        }
    }

    fn report() -> ListingReport {
        resolve(&snapshot()).unwrap()
    }

    fn opts<'a>(policy: PolicySource<'a>) -> RenderOptions<'a> {
        RenderOptions {
            size: DEFAULT_PAGE_SIZE,
            template: ITEMS_TOKEN,
            now: 5_000,
            policy,
        }
    }

    fn by_cell(r: &ShopRender, cell: Cell) -> Vec<String> {
        let mut out: Vec<String> = r
            .posted
            .iter()
            .filter(|p| p.cell == cell)
            .map(|p| p.target.to_string())
            .chain(
                r.left_out
                    .iter()
                    .filter(|l| l.cell == cell)
                    .map(|l| l.target.to_string()),
            )
            .collect();
        out.sort();
        out
    }

    /// C74 — every item lands in exactly one cell of the table, each
    /// cell's count is its items, and the four verdicts sum to the item
    /// count: nothing is skipped silently.
    #[test]
    fn c74_every_item_is_posted_omitted_blocked_or_off_the_page_and_counted() {
        let r = render(&report(), &opts(PolicySource::NotSet)).unwrap();
        let c = &r.counts;
        assert_eq!(c.items, 21);
        assert_eq!(c.posted + c.omitted + c.blocked + c.off_page, c.items);
        assert_eq!(c.posted, 6, "{:?}", r.posted);
        assert_eq!(r.posted.len(), c.posted);
        assert_eq!(r.left_out.len(), c.items - c.posted);
        assert_eq!(c.by_cell.values().sum::<usize>(), c.items);
        assert_eq!(c.by_cell.len(), Cell::ALL.len());
        for row in &r.policy {
            assert_eq!(row.count, c.by_cell[&row.cell], "{}", row.cell);
            assert_eq!(row.verdict, row.cell.verdict());
            assert_eq!(row.why, row.cell.why());
        }
        let expect = |cell: Cell, targets: &[&str]| {
            let mut want: Vec<String> = targets.iter().map(|t| format!("item/{t}")).collect();
            want.sort();
            assert_eq!(by_cell(&r, cell), want, "{cell}");
        };
        expect(Cell::StashItem, &["i-hand", "i-hand2", "i-bo", "i-tabrow"]);
        expect(Cell::CharacterItem, &["i-worn"]);
        expect(Cell::GameLists, &["i-game", "i-noted"]);
        expect(Cell::GameSkips, &["i-gameskip"]);
        // A league-less character's items read their effective outcome
        // first: only a hand price reaches the league cell.
        expect(Cell::HandSkip, &["i-skip", "i-nowhere-skip"]);
        expect(Cell::NothingApplies, &["i-plain", "i-nowhere-plain"]);
        expect(Cell::Unresolved, &["i-unres"]);
        expect(Cell::LeagueUnknown, &["i-nowhere"]);
        // Q5 answered (T21): a no-price item posts as its link alone.
        expect(Cell::HandNoPrice, &["i-noprice"]);
        expect(Cell::Ratio, &["i-ratio"]);
        expect(Cell::RetiredCurrency, &["i-retired"]);
        expect(Cell::Socketed, &["i-gem", "i-worngem"]);
        expect(Cell::Substash, &["i-sub"]);
        expect(Cell::TabUnlisted, &["i-unlisted"]);
        expect(Cell::RealmUnlisted, &[]);
        expect(Cell::NoPosition, &[]);
        expect(Cell::NoSlot, &[]);
        expect(Cell::PageSize, &[]);
        expect(Cell::InvalidIndex, &[]);
        expect(Cell::UnruledKind, &[]);

        // A poe2 report: nothing the site lists (T4).
        let mut poe2 = snapshot();
        poe2.realm = "poe2".into();
        poe2.tabs.clear();
        poe2.items.retain(|i| i.location_kind == "character");
        let r = render(&resolve(&poe2).unwrap(), &opts(PolicySource::NotSet)).unwrap();
        assert_eq!(r.counts.posted, 0);
        // The worn item and its gem: the realm is read before the address.
        assert_eq!(r.counts.by_cell[&Cell::RealmUnlisted], 2);
        assert_eq!(r.counts.by_cell[&Cell::LeagueUnknown], 1);
        assert_eq!(r.counts.by_cell[&Cell::HandSkip], 1);
        assert_eq!(r.counts.by_cell[&Cell::NothingApplies], 1);
    }

    /// C47, C74 — a store row never panics the render and a vocabulary
    /// the table has no row for fails closed: a corrupt timestamp
    /// saturates the age, a negative index is `invalid_index`,
    /// a hand-price kind the table does not write is `unruled_kind`, and
    /// a realm the site does not list is `realm_unlisted`.
    #[test]
    fn c47_corrupt_facts_and_unruled_words_are_cells_never_panics() {
        let mut snap = snapshot();
        for i in &mut snap.items {
            if i.id == "i-hand" {
                i.last_seen = i64::MIN;
            }
        }
        for t in &mut snap.tabs {
            if t.id == "c1" {
                t.idx = Some(-1);
            }
            if t.id == "t2" {
                // Not a corrupt fact any more: a rank never adds to it.
                t.idx = Some(i64::MAX);
            }
        }
        let policy = SyncPolicy::from_value(&json!({
            "version": 3,
            "realms": { "pc": { "leagues": { "Standard": {
                "tabs": "all", "characters": "all", "max_age_seconds": 3600 } } } }
        }))
        .unwrap();
        let r = render(
            &resolve(&snap).unwrap(),
            &opts(PolicySource::Set {
                policy: &policy,
                revision: 1,
                refresh: Err("no such league".into()),
            }),
        )
        .unwrap();
        // c1's four hand-priced items are blocked; t2's item, at the
        // largest index, is simply the last-ranked tab: after t3 and m1
        // (the folder and the corrupt tab take no number) it is Stash3.
        assert_eq!(r.counts.by_cell[&Cell::InvalidIndex], 4);
        assert_eq!(r.counts.posted, 2);
        assert!(
            r.posted
                .iter()
                .any(|p| p.target.to_string() == "item/i-tabrow" && p.link.contains("Stash3")),
            "{:?}",
            r.posted
        );
        assert_eq!(
            r.freshness.refresh_problem.as_deref(),
            Some("no such league")
        );
        assert_eq!(r.freshness.refresh_requests, None);

        // The corrupt timestamp on a posted item: the age saturates.
        let mut snap = snapshot();
        for i in &mut snap.items {
            if i.id == "i-hand" {
                i.last_seen = i64::MIN;
            }
        }
        let r = render(
            &resolve(&snap).unwrap(),
            &opts(PolicySource::Set {
                policy: &policy,
                revision: 1,
                refresh: Ok(3),
            }),
        )
        .unwrap();
        assert_eq!(r.freshness.oldest_stale_seconds, Some(i64::MAX));
        assert!(
            r.freshness
                .stale
                .iter()
                .any(|t| t.to_string() == "item/i-hand")
        );

        // A kind the table does not write: the report is edited in place
        // (no value schema carries one yet), and the cell blocks it.
        let mut report = report();
        for l in &mut report.listings {
            if l.subject.target.to_string() == "item/i-hand" {
                l.effective.kind = "current_offer".into();
            }
        }
        let r = render(&report, &opts(PolicySource::NotSet)).unwrap();
        assert_eq!(by_cell(&r, Cell::UnruledKind), ["item/i-hand"]);
        assert_eq!(r.counts.posted, 5);
    }

    /// C74, T7, T13, T24 — the two link shapes in the website's attribute
    /// order: a stash item by its tab's rank among the tabs the website
    /// lists (the folder at index 0 takes no number, so c1 at index 1 is
    /// `Stash1` and t2 at index 2 is `Stash2`), a character item by its
    /// slot and the character's name; `~price` and `~b/o` as spoiler
    /// titles with the table's word; one spoiler per price.
    #[test]
    fn c74_the_link_codes_and_spoiler_titles_are_the_observed_shapes() {
        let r = render(&report(), &opts(PolicySource::NotSet)).unwrap();
        let find = |id: &str| {
            r.posted
                .iter()
                .find(|p| p.target.to_string() == format!("item/{id}"))
                .unwrap()
        };
        let hand = find("i-hand");
        assert_eq!(
            hand.link,
            "[linkItem realm=\"pc\" location=\"Stash1\" league=\"Standard\" x=\"1\" y=\"0\"]"
        );
        assert_eq!(hand.title, " ~price 5 chaos");
        let worn = find("i-worn");
        assert_eq!(
            worn.link,
            "[linkItem realm=\"pc\" location=\"BodyArmour\" character=\"Exile\" x=\"0\" y=\"0\"]"
        );
        assert_eq!(worn.title, " ~price 10 divine");
        assert_eq!(find("i-bo").title, " ~b/o 1.5 divine");
        assert_eq!(find("i-tabrow").title, " ~b/o 2 divine");
        assert_eq!(find("i-noprice").title, "");
        assert_eq!(r.pages.len(), 1);
        let page = &r.pages[0];
        // The owner's shape (2026-09-07): one spoiler per price with the
        // links run together, one page spoiler labelled n of N, newlines
        // only between spoiler tags; no price last, under an empty title.
        assert_eq!(
            page.text,
            "[spoiler=\"Shop Post 1 of 1 (6 items)\"]\n\
             [spoiler=\" ~price 5 chaos\"][linkItem realm=\"pc\" location=\"Stash1\" league=\"Standard\" x=\"1\" y=\"0\"][linkItem realm=\"pc\" location=\"Stash1\" league=\"Standard\" x=\"2\" y=\"0\"][/spoiler]\n\
             [spoiler=\" ~price 10 divine\"][linkItem realm=\"pc\" location=\"BodyArmour\" character=\"Exile\" x=\"0\" y=\"0\"][/spoiler]\n\
             [spoiler=\" ~b/o 1.5 divine\"][linkItem realm=\"pc\" location=\"Stash1\" league=\"Standard\" x=\"3\" y=\"0\"][/spoiler]\n\
             [spoiler=\" ~b/o 2 divine\"][linkItem realm=\"pc\" location=\"Stash2\" league=\"Standard\" x=\"0\" y=\"1\"][/spoiler]\n\
             [spoiler=\"\"][linkItem realm=\"pc\" location=\"Stash1\" league=\"Standard\" x=\"8\" y=\"0\"][/spoiler]\n\
             [/spoiler]\n"
        );
        assert_eq!(page.chars, page.text.chars().count());
        assert_eq!(page.items, 6);
        assert!(r.posted.iter().all(|p| p.page == 1));
    }

    /// C74 — pages are cut under the size with the template counted, a
    /// group runs on across the cut, every page is labelled n of N, the
    /// template wraps each page, an entry too large for an empty page is
    /// blocked, and a template without the token refuses.
    #[test]
    fn c74_pages_are_cut_under_the_size_and_carry_the_template() {
        let report = report();
        let template = "Shop\n[items]Thanks\n";
        let overhead = template.chars().count() - ITEMS_TOKEN.len();
        let full = render(&report, &opts(PolicySource::NotSet)).unwrap();
        let link = |id: &str| {
            full.posted
                .iter()
                .find(|p| p.target.to_string() == format!("item/{id}"))
                .unwrap()
                .link
                .chars()
                .count()
        };
        // Exactly the first group's two links fit the first page with
        // the page spoiler reserved at its widest; the next group does
        // not, so the cut falls between spoilers.
        let size = page_fixed_chars(6, overhead)
            + group_open(" ~price 5 chaos").chars().count()
            + GROUP_CLOSE.len()
            + link("i-hand")
            + link("i-hand2");
        let r = render(
            &report,
            &RenderOptions {
                size,
                template,
                now: 5_000,
                policy: PolicySource::NotSet,
            },
        )
        .unwrap();
        assert_eq!(r.counts.posted, 6);
        assert_eq!(r.counts.blocked, 8);
        assert_eq!(r.counts.omitted, 5);
        assert_eq!(r.counts.off_page, 2);
        assert!(r.pages.len() >= 3, "{:#?}", r.pages);
        assert_eq!(r.counts.pages, r.pages.len());
        for page in &r.pages {
            assert!(
                page.chars <= size,
                "page {} is {} > {size}",
                page.number,
                page.chars
            );
            assert_eq!(page.of, r.pages.len());
            assert!(
                page.text.starts_with(&format!(
                    "Shop\n[spoiler=\"Shop Post {} of {} ({} {})\"]\n[spoiler=\"",
                    page.number,
                    page.of,
                    page.items,
                    if page.items == 1 { "item" } else { "items" }
                )),
                "{}",
                page.text
            );
            assert!(
                page.text.ends_with("[/spoiler]\n[/spoiler]\nThanks\n"),
                "{}",
                page.text
            );
            assert_eq!(
                r.posted.iter().filter(|p| p.page == page.number).count(),
                page.items
            );
        }
        assert_eq!(r.pages[0].items, 2);
        assert_eq!(r.pages.iter().map(|p| p.items).sum::<usize>(), 6);
        // Page numbers follow the group order.
        assert!(r.posted.windows(2).all(|w| w[0].page <= w[1].page));

        // A group cut in two is closed on one page and reopened on the
        // next: one link of the first group per page.
        let r = render(
            &report,
            &RenderOptions {
                size: size - link("i-hand2"),
                template,
                now: 5_000,
                policy: PolicySource::NotSet,
            },
        )
        .unwrap();
        assert_eq!(r.pages[0].items, 1);
        assert_eq!(r.pages[1].items, 1);
        assert!(r.pages[0].text.contains("[spoiler=\" ~price 5 chaos\"]"));
        assert!(r.pages[1].text.contains("[spoiler=\" ~price 5 chaos\"]"));

        // Below one entry: every candidate is blocked as page_size and
        // there are no pages.
        let r = render(
            &report,
            &RenderOptions {
                size: overhead + 10,
                template,
                now: 5_000,
                policy: PolicySource::NotSet,
            },
        )
        .unwrap();
        assert_eq!(r.counts.posted, 0);
        assert_eq!(r.counts.by_cell[&Cell::PageSize], 6);
        assert!(r.pages.is_empty());
        assert_eq!(r.counts.blocked, 14);

        for (template, want) in [
            ("no token", RenderError::Template { occurrences: 0 }),
            ("[items] [items]", RenderError::Template { occurrences: 2 }),
        ] {
            let err = render(
                &report,
                &RenderOptions {
                    size: 100,
                    template,
                    now: 5_000,
                    policy: PolicySource::NotSet,
                },
            )
            .unwrap_err();
            assert_eq!(err, want);
        }
        let err = render(
            &report,
            &RenderOptions {
                size: 3,
                template,
                now: 5_000,
                policy: PolicySource::NotSet,
            },
        )
        .unwrap_err();
        assert!(matches!(err, RenderError::TemplateTooLarge { .. }), "{err}");
    }

    /// C72 — coverage, staleness and positions are reported over the
    /// posted items, never enforced: containers outside the policy's
    /// selection are named (a tab through its folder counts as covered,
    /// C37), facts older than the window are named with the refresh's
    /// request count, a stash item whose fetch predates the listing is
    /// named; an absent, unreadable or non-covering policy says which.
    #[test]
    fn c72_coverage_staleness_and_positions_are_reported_never_enforced() {
        let report = report();
        let policy = SyncPolicy::from_value(&json!({
            "version": 3,
            "realms": { "pc": { "leagues": { "Standard": {
                "tabs": ["f1"], "characters": ["ch1"], "max_age_seconds": 3600 } } } }
        }))
        .unwrap();
        let r = render(
            &report,
            &opts(PolicySource::Set {
                policy: &policy,
                revision: 4,
                refresh: Ok(7),
            }),
        )
        .unwrap();
        let f = &r.freshness;
        assert_eq!(f.policy, "covers");
        assert_eq!(f.policy_revision, Some(4));
        assert_eq!(f.window_seconds, Some(3600));
        assert_eq!(f.refresh_requests, Some(7));
        // c1 is covered through its folder f1; t2 is not named.
        assert_eq!(
            f.uncovered
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["tab/pc/t2"]
        );
        // now 5000: the c1 items and the worn item were seen at 1200
        // (3800 s ago), over the window, in covered containers — the
        // cited refresh fetches them; the t2 item (seen at 100, 4900 s
        // ago) is past the window too, but its container is uncovered,
        // so it is counted beside the coverage line, not as stale — and
        // the oldest age is the stale set's, never the uncovered item's.
        assert_eq!(f.stale.len(), 5);
        assert!(!f.stale.iter().any(|t| t.to_string() == "item/i-tabrow"));
        assert_eq!(f.stale_uncovered, 1);
        assert_eq!(f.oldest_stale_seconds, Some(3_800));
        // The t2 item's fetch (response 1) predates the listing (2).
        assert_eq!(
            f.position_before_listing
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["item/i-tabrow"]
        );

        // A wide window and full coverage: nothing to report.
        let policy = SyncPolicy::from_value(&json!({
            "version": 3,
            "realms": { "pc": { "leagues": { "Standard": {
                "tabs": "all", "characters": "all", "max_age_seconds": 10000 } } } }
        }))
        .unwrap();
        let r = render(
            &report,
            &opts(PolicySource::Set {
                policy: &policy,
                revision: 5,
                refresh: Err("nothing to plan".into()),
            }),
        )
        .unwrap();
        assert!(r.freshness.uncovered.is_empty());
        assert!(r.freshness.stale.is_empty());
        assert_eq!(r.freshness.stale_uncovered, 0);
        assert_eq!(r.freshness.refresh_requests, None);
        assert_eq!(
            r.freshness.refresh_problem.as_deref(),
            Some("nothing to plan")
        );

        // The policy covers another league only.
        let policy = SyncPolicy::from_value(&json!({
            "version": 3,
            "realms": { "pc": { "leagues": { "Hardcore": {
                "tabs": "all", "max_age_seconds": 10 } } } }
        }))
        .unwrap();
        let r = render(
            &report,
            &opts(PolicySource::Set {
                policy: &policy,
                revision: 6,
                refresh: Ok(0),
            }),
        )
        .unwrap();
        assert_eq!(r.freshness.policy, "league_not_covered");
        assert_eq!(r.freshness.window_seconds, None);
        assert!(r.freshness.uncovered.is_empty());

        let r = render(
            &report,
            &opts(PolicySource::Unreadable {
                revision: 9,
                why: "version 12".into(),
            }),
        )
        .unwrap();
        assert_eq!(r.freshness.policy, "unreadable");
        assert_eq!(r.freshness.policy_problem.as_deref(), Some("version 12"));
        let r = render(&report, &opts(PolicySource::NotSet)).unwrap();
        assert_eq!(r.freshness.policy, "not_set");
        // The position line needs no policy.
        assert_eq!(r.freshness.position_before_listing.len(), 1);
    }

    /// T24 — `Stash<n>` against the website's own numbering, at scale: the
    /// owner's two leagues as the site listed them on 2026-09-07 beside
    /// the API's listings (`reference/website-tabs-2026-09-07.json`).
    /// Standard has 16 folders interleaved among 402 tabs, 64 substashes
    /// and 274 remove-only tabs; every tab the site lists ranks exactly at
    /// its site number plus one, and no folder or substash has a number.
    #[test]
    fn t24_stash_numbers_match_the_websites_own_list() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/reference/website-tabs-2026-09-07.json"
        );
        let doc: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        for (league, lists) in doc["leagues"].as_object().unwrap() {
            let tabs: Vec<TabSnapshot> = lists["api"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| TabSnapshot {
                    id: t["id"].as_str().unwrap().into(),
                    parent: t["parent"].as_str().map(str::to_string),
                    name: String::new(),
                    r#type: t["type"].as_str().unwrap().into(),
                    idx: t["index"].as_i64(),
                    listed_at: Some(1_000),
                    listed_response: Some(2),
                    fetched_at: None,
                    metadata: Value::Null,
                    item_count: 0,
                })
                .collect();
            let folders: usize = tabs.iter().filter(|t| t.r#type == FOLDER).count();
            let substashes: usize = tabs
                .iter()
                .filter(|t| {
                    t.parent
                        .as_deref()
                        .is_some_and(|p| tabs.iter().any(|q| q.id == p && q.r#type != FOLDER))
                })
                .count();
            let mut snap = snapshot();
            snap.league = league.clone();
            snap.tabs = tabs;
            snap.items.clear();
            snap.buyouts.clear();
            let numbers = stash_numbers(&resolve(&snap).unwrap());
            let web = lists["web"].as_array().unwrap();
            assert_eq!(
                numbers.len(),
                web.len(),
                "{league}: {} numbered, {} on the site ({folders} folders, {substashes} substashes)",
                numbers.len(),
                web.len()
            );
            for t in web {
                let site_id = t["id"].as_str().unwrap();
                let api_id: String = site_id.chars().take(10).collect();
                let want = t["i"].as_u64().unwrap() as usize + 1;
                assert_eq!(
                    numbers.get(&api_id),
                    Some(&StashNumber::Rank(want)),
                    "{league}: {} ({}) at site number {}",
                    t["n"],
                    api_id,
                    t["i"]
                );
            }
            for t in &snap.tabs {
                if t.r#type == FOLDER
                    || t.parent
                        .as_deref()
                        .is_some_and(|p| snap.tabs.iter().any(|q| q.id == p && q.r#type != FOLDER))
                {
                    assert!(
                        !numbers.contains_key(&t.id),
                        "{league}: {} has a number",
                        t.id
                    );
                }
            }
        }
    }

    /// C53 — the JSON shape is the contract: the rendered fixture must
    /// serialize to exactly the committed document. A difference is
    /// either additive (regenerate with `ACQ_UPDATE_FIXTURES=1`) or a
    /// schema bump (`SHOP_SCHEMA`), decided by reading the diff.
    #[test]
    fn the_render_json_matches_the_committed_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/reference/shop-render-schema-3.json"
        );
        let policy = SyncPolicy::from_value(&json!({
            "version": 3,
            "realms": { "pc": { "leagues": { "Standard": {
                "tabs": ["f1"], "characters": ["ch1"], "max_age_seconds": 3600 } } } }
        }))
        .unwrap();
        let r = render(
            &report(),
            &opts(PolicySource::Set {
                policy: &policy,
                revision: 4,
                refresh: Ok(7),
            }),
        )
        .unwrap();
        let actual = serde_json::to_value(&r).unwrap();
        if std::env::var_os("ACQ_UPDATE_FIXTURES").is_some() {
            std::fs::write(path, serde_json::to_string_pretty(&actual).unwrap() + "\n").unwrap();
        }
        let expected: Value = serde_json::from_str(
            &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}")),
        )
        .unwrap();
        assert_eq!(expected["schema"], json!(SHOP_SCHEMA));
        assert_eq!(
            actual, expected,
            "the render's JSON changed; additive → ACQ_UPDATE_FIXTURES=1, else bump SHOP_SCHEMA"
        );
        let back: ShopRender = serde_json::from_value(expected).unwrap();
        assert_eq!(back, r);
    }

    /// The plan step 7 property tests (`PRICING-SLICE.md`): the page
    /// cutter over any entries, template and size.
    mod properties {
        use super::*;
        use proptest::prelude::*;

        /// An entry's parts: its group order (`~price`, `~b/o`, no price),
        /// tag, whole amount, and the text that makes its link.
        fn entry_parts() -> impl Strategy<Value = (u8, &'static str, u64, String)> {
            (
                0u8..3,
                prop_oneof![Just("chaos"), Just("divine"), Just("exalted")],
                1u64..20,
                "[a-z ]{0,30}",
            )
        }

        /// The entry as [`cell`] would build it: the title a function of
        /// the group, the link unique to the entry.
        fn entry(i: usize, (order, tag, amount, text): (u8, &'static str, u64, String)) -> Entry {
            let amount = Amount::Decimal {
                ten_thousandths: amount * 10_000,
            };
            let (group, title) = if order == 2 {
                ((2, String::new(), None), String::new())
            } else {
                let prefix = if order == 0 { " ~price" } else { " ~b/o" };
                (
                    (order, tag.to_string(), Some(amount)),
                    format!("{prefix} {amount} {tag}"),
                )
            };
            Entry {
                target: PriceTarget::Item {
                    id: format!("i{i}"),
                },
                label: format!("item {i}"),
                location: PriceTarget::Tab {
                    realm: Realm::Pc,
                    id: "t".into(),
                },
                cell: if order == 2 {
                    Cell::HandNoPrice
                } else {
                    Cell::StashItem
                },
                group,
                link: format!("[link #{i}# {text}]"),
                title,
                seen_response: None,
                seen_at: None,
                parent: None,
            }
        }

        proptest! {
            /// C74 — for any entries, any template and any size: every
            /// candidate is on a page or set aside, never both; every page
            /// is within the size, the template around it, numbered n of N
            /// and holding at least one item; every posted item's link is on
            /// its page exactly once and on no other; groups are sorted
            /// across the run and contiguous on a page — no group reopens
            /// on the page that closed it, and a page's spoilers are its
            /// groups plus its own.
            #[test]
            fn c74_any_entries_cut_into_pages_within_the_size_each_posted_once_in_contiguous_groups(
                parts in prop::collection::vec(entry_parts(), 0..40),
                before in "[^\\[]{0,30}",
                after in "[^\\[]{0,30}",
                size in 0usize..1200,
            ) {
                let template = format!("{before}{ITEMS_TOKEN}{after}");
                let candidates: Vec<Entry> = parts.into_iter().enumerate().map(|(i, p)| entry(i, p)).collect();
                let n = candidates.len();
                let cut = cut_pages(candidates, &template, size);

                prop_assert_eq!(cut.entries.len() + cut.too_large.len(), n);
                prop_assert_eq!(cut.posted.len(), cut.entries.len());
                let mut targets: Vec<&PriceTarget> = cut
                    .entries
                    .iter()
                    .chain(&cut.too_large)
                    .map(|e| &e.target)
                    .collect();
                targets.sort();
                targets.dedup();
                prop_assert_eq!(targets.len(), n, "an entry landed twice or not at all");

                let of = cut.pages.len();
                prop_assert_eq!(cut.pages.iter().map(|p| p.items).sum::<usize>(), cut.posted.len());
                for (i, page) in cut.pages.iter().enumerate() {
                    prop_assert_eq!(page.number, i + 1);
                    prop_assert_eq!(page.of, of);
                    prop_assert!(page.items > 0);
                    prop_assert_eq!(page.chars, page.text.chars().count());
                    prop_assert!(page.chars <= size, "page {} holds {} chars over {}:\n{}", page.number, page.chars, size, page.text);
                    prop_assert!(page.text.starts_with(&before) && page.text.ends_with(&after));
                    prop_assert_eq!(cut.posted.iter().filter(|p| p.page == page.number).count(), page.items);
                }

                for (e, p) in cut.entries.iter().zip(&cut.posted) {
                    prop_assert_eq!(&e.target, &p.target);
                    prop_assert_eq!(&e.link, &p.link);
                    prop_assert!((1..=of).contains(&p.page));
                    for page in &cut.pages {
                        prop_assert_eq!(
                            page.text.matches(&e.link).count(),
                            usize::from(page.number == p.page),
                            "{} on page {}", e.link, page.number
                        );
                    }
                }

                prop_assert!(cut.entries.windows(2).all(|w| w[0].group <= w[1].group), "groups out of order");
                prop_assert!(cut.posted.windows(2).all(|w| w[0].page <= w[1].page), "pages out of order");
                for page in &cut.pages {
                    let mut keys: Vec<&(u8, String, Option<Amount>)> = cut
                        .entries
                        .iter()
                        .zip(&cut.posted)
                        .filter(|(_, p)| p.page == page.number)
                        .map(|(e, _)| &e.group)
                        .collect();
                    keys.dedup();
                    let mut distinct = keys.clone();
                    distinct.sort();
                    distinct.dedup();
                    prop_assert_eq!(keys.len(), distinct.len(), "a group reopened on page {}", page.number);
                    prop_assert_eq!(page.text.matches("[spoiler=").count(), keys.len() + 1, "{}", page.text);
                    prop_assert_eq!(page.text.matches("[/spoiler]").count(), keys.len() + 1, "{}", page.text);
                }
            }
        }
    }

    /// The real-scale fixture (plan step 7, item 2, `PRICING-SLICE.md`):
    /// the owner's pc/Standard league, rendered whole — the page the
    /// owner pasted for validation reading 2, from the same facts.
    mod real_scale {
        use std::time::{Duration, Instant};

        use super::*;
        use crate::real_scale_fixture;
        use acquisition_store::{PricingSnapshot, TabSnapshot};

        fn fixture() -> (PricingSnapshot, ListingReport) {
            let s = real_scale_fixture::snapshot();
            let r = resolve(&s).unwrap();
            (s, r)
        }

        fn opts_at(now: i64, policy: PolicySource<'_>) -> RenderOptions<'_> {
            RenderOptions {
                size: DEFAULT_PAGE_SIZE,
                template: ITEMS_TOKEN,
                now,
                policy,
            }
        }

        fn tab_id(target: &PriceTarget) -> &str {
            let PriceTarget::Tab { id, .. } = target else {
                panic!("{target} is not a tab");
            };
            id
        }

        /// C74, C81 — every one of the 1,977 items lands in one cell and
        /// the cells are read off the listing state: the 48 game prices
        /// and the one `~skip` are omitted, the character's 3 hand skips
        /// too, the substash row's 5 items are blocked on Q3, 1,889 have
        /// nothing applying, and the 31 the hand decides are the page:
        /// one page, under the size, every link once, under three titles.
        /// The render is guarded: under 1 ms in a debug build on
        /// 2026-09-07 — the bound is a cliff, not a budget.
        #[test]
        fn c74_at_real_scale_every_item_lands_in_one_cell_and_the_page_is_the_owners_test_tab() {
            let (s, report) = fixture();
            let t = Instant::now();
            let r = render(&report, &opts_at(s.taken_at, PolicySource::NotSet)).unwrap();
            let took = t.elapsed();
            assert!(took < Duration::from_secs(2), "render took {took:?}");
            assert_eq!(r.schema, SHOP_SCHEMA);
            assert_eq!(r.listing, report.header);

            let c = &r.counts;
            assert_eq!(c.items, 1977);
            assert_eq!(
                (c.posted, c.omitted, c.blocked, c.off_page, c.pages),
                (31, 52, 5, 1889, 1)
            );
            assert_eq!(c.posted + c.omitted + c.blocked + c.off_page, c.items);
            assert_eq!(r.posted.len() + r.left_out.len(), c.items);
            assert_eq!(c.by_cell.values().sum::<usize>(), c.items);
            let expected: BTreeMap<Cell, usize> = Cell::ALL
                .iter()
                .map(|cell| {
                    let n = match cell {
                        Cell::GameLists => 48,
                        Cell::GameSkips => 1,
                        Cell::HandSkip => 3,
                        Cell::Substash => 5,
                        Cell::StashItem => 31,
                        Cell::NothingApplies => 1889,
                        _ => 0,
                    };
                    (*cell, n)
                })
                .collect();
            assert_eq!(c.by_cell, expected);
            for row in &r.policy {
                assert_eq!(row.count, c.by_cell[&row.cell], "{}", row.cell);
            }
            assert_eq!(c.by_cell[&Cell::GameLists], report.counts.game_priced);
            assert_eq!(
                c.by_cell[&Cell::GameSkips],
                report.counts.by_game_statement["skip"]
            );
            assert_eq!(
                c.by_cell[&Cell::NothingApplies],
                report.counts.by_relation["none"]
            );
            assert!(
                r.left_out
                    .iter()
                    .filter(|l| l.cell == Cell::HandSkip)
                    .all(|l| matches!(l.location, Some(PriceTarget::Character { .. })))
            );
            assert!(
                r.left_out
                    .iter()
                    .filter(|l| l.cell == Cell::Substash)
                    .all(|l| matches!(l.location, Some(PriceTarget::Substash { .. })))
            );

            let tab = real_scale_fixture::owners_test_tab(&s);
            assert!(
                r.posted
                    .iter()
                    .all(|p| p.location == tab && p.page == 1 && p.cell == Cell::StashItem)
            );
            let mut titles: BTreeMap<&str, usize> = BTreeMap::new();
            for p in &r.posted {
                *titles.entry(p.title.as_str()).or_default() += 1;
            }
            assert_eq!(
                titles,
                [
                    (" ~b/o 1.5 divine", 1),
                    (" ~price 2 chaos", 1),
                    (" ~price 5 chaos", 29)
                ]
                .into()
            );
            let [page] = &r.pages[..] else {
                panic!("one page");
            };
            assert_eq!((page.number, page.of, page.items), (1, 1, 31));
            assert!(page.chars <= DEFAULT_PAGE_SIZE);
            assert_eq!(page.chars, page.text.chars().count());
            assert_eq!(page.text.matches("[spoiler=").count(), 1 + titles.len());
            assert!(
                page.text
                    .starts_with("[spoiler=\"Shop Post 1 of 1 (31 items)\"]\n")
            );
            for p in &r.posted {
                assert_eq!(page.text.matches(&p.link).count(), 1, "{}", p.link);
                assert!(
                    page.text.contains(&format!("[spoiler=\"{}\"]", p.title)),
                    "{}",
                    p.title
                );
            }

            let text = serde_json::to_string(&r).unwrap();
            let back: ShopRender = serde_json::from_str(&text).unwrap();
            assert_eq!(back, r);
        }

        /// T24 — the site as the oracle for the numbering, over the real
        /// posted items: the tabs the site lists (top-level and folder
        /// children; folders and substashes out) in the snapshot's order
        /// are the site's own list of the same day by name and type
        /// (`reference/website-tabs-2026-09-07.json`), and every posted
        /// link's `Stash<n>` is its tab's site number plus one — the
        /// owner's test tab is the site's 56, so `Stash57`, the page the
        /// owner read correct on the trade site.
        #[test]
        fn t24_at_real_scale_every_posted_link_numbers_its_tab_as_the_website_does() {
            let (s, report) = fixture();
            let r = render(&report, &opts_at(s.taken_at, PolicySource::NotSet)).unwrap();
            let path = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/reference/website-tabs-2026-09-07.json"
            );
            let doc: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let web = doc["leagues"][&s.league]["web"].as_array().unwrap();

            let by_id: HashMap<&str, &TabSnapshot> =
                s.tabs.iter().map(|t| (t.id.as_str(), t)).collect();
            let listed: Vec<&TabSnapshot> = s
                .tabs
                .iter()
                .filter(|t| t.r#type != FOLDER)
                .filter(|t| {
                    t.parent
                        .as_deref()
                        .is_none_or(|p| by_id.get(p).is_some_and(|q| q.r#type == FOLDER))
                })
                .collect();
            assert_eq!(listed.len(), web.len());
            assert_eq!(listed.len(), 322);
            for (i, (t, w)) in listed.iter().zip(web).enumerate() {
                assert_eq!(w["i"], json!(i));
                assert_eq!(w["n"].as_str(), Some(t.name.as_str()), "site {i}");
                assert_eq!(w["type"].as_str(), Some(t.r#type.as_str()), "site {i}");
            }

            assert_eq!(r.posted.len(), 31);
            for p in &r.posted {
                let id = tab_id(&p.location);
                let rank = listed.iter().position(|t| t.id == id).unwrap();
                let site = web[rank]["i"].as_u64().unwrap() as usize;
                assert_eq!(site, rank);
                let location = format!(" location=\"Stash{}\" ", site + 1);
                assert!(p.link.contains(&location), "{}: {}", p.target, p.link);
                assert!(
                    p.link.contains(&format!(" league=\"{}\" ", s.league)),
                    "{}",
                    p.link
                );
            }
            let tab = real_scale_fixture::owners_test_tab(&s);
            let rank = listed.iter().position(|t| t.id == tab_id(&tab)).unwrap();
            assert_eq!(rank, 56);
            assert!(r.posted[0].link.contains(" location=\"Stash57\" "));
        }

        /// C72 — the freshness lines over the real page, at the snapshot's
        /// own clock: the test tab was fetched four days before the
        /// snapshot was taken, so under a policy that covers it with an
        /// hour's window every posted item is stale and the oldest age is
        /// that gap; under a policy that does not name it, it is the one
        /// container outside coverage and its items are counted beside
        /// that line; under a window wider than the gap, nothing. The
        /// tab's fetch came after the stash listing the links number by,
        /// so no position predates the listing.
        #[test]
        fn c72_at_real_scale_the_freshness_lines_name_the_owners_test_tab() {
            let (s, report) = fixture();
            let tab = real_scale_fixture::owners_test_tab(&s);
            let seen: Vec<i64> = s
                .items
                .iter()
                .filter(|i| i.location_id == tab_id(&tab))
                .map(|i| i.last_seen)
                .collect();
            assert_eq!(seen.len(), 80);
            let gap = s.taken_at - seen.iter().min().unwrap();
            assert!(gap > 4 * 86_400 && gap < 5 * 86_400, "{gap}");
            let listing = s.stash_listing.unwrap().response_id;
            assert!(
                s.items
                    .iter()
                    .filter(|i| i.location_id == tab_id(&tab))
                    .all(|i| i.seen_response.is_some_and(|seen| seen > listing))
            );

            let policy = |value: Value| SyncPolicy::from_value(&value).unwrap();
            let covering = policy(json!({
                "version": 3,
                "realms": { "pc": { "leagues": { "Standard": {
                    "tabs": [tab_id(&tab)], "max_age_seconds": 3600 } } } }
            }));
            let r = render(
                &report,
                &opts_at(
                    s.taken_at,
                    PolicySource::Set {
                        policy: &covering,
                        revision: 3,
                        refresh: Ok(2),
                    },
                ),
            )
            .unwrap();
            let f = &r.freshness;
            assert_eq!(f.policy, "covers");
            assert_eq!(f.window_seconds, Some(3600));
            assert_eq!(f.refresh_requests, Some(2));
            assert!(f.uncovered.is_empty());
            assert_eq!(f.stale.len(), 31);
            assert_eq!(
                f.stale,
                r.posted
                    .iter()
                    .map(|p| p.target.clone())
                    .collect::<Vec<_>>()
            );
            assert_eq!(f.stale_uncovered, 0);
            assert_eq!(f.oldest_stale_seconds, Some(gap));
            assert!(f.position_before_listing.is_empty());

            let elsewhere = policy(json!({
                "version": 3,
                "realms": { "pc": { "leagues": { "Standard": {
                    "characters": "all", "max_age_seconds": 3600 } } } }
            }));
            let r = render(
                &report,
                &opts_at(
                    s.taken_at,
                    PolicySource::Set {
                        policy: &elsewhere,
                        revision: 4,
                        refresh: Ok(0),
                    },
                ),
            )
            .unwrap();
            let f = &r.freshness;
            assert_eq!(f.uncovered, std::slice::from_ref(&tab));
            assert!(f.stale.is_empty());
            assert_eq!(f.stale_uncovered, 31);
            assert_eq!(f.oldest_stale_seconds, None);

            let wide = policy(json!({
                "version": 3,
                "realms": { "pc": { "leagues": { "Standard": {
                    "tabs": "all", "max_age_seconds": 5 * 86_400 } } } }
            }));
            let r = render(
                &report,
                &opts_at(
                    s.taken_at,
                    PolicySource::Set {
                        policy: &wide,
                        revision: 5,
                        refresh: Ok(400),
                    },
                ),
            )
            .unwrap();
            let f = &r.freshness;
            assert!(f.uncovered.is_empty());
            assert!(f.stale.is_empty());
            assert_eq!(f.stale_uncovered, 0);
            assert_eq!(f.oldest_stale_seconds, None);
        }
    }
}
