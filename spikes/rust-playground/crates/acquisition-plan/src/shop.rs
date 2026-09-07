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
//! tab into refresh.** The relationship between the two kinds of intent
//! is reported, not enforced: the consumer that needs freshness — `shop
//! render` first — names priced locations outside the policy's coverage,
//! priced facts older than its stated window, and items moved or
//! reindexed since the render's basis, each with the remedy (the policy
//! edit, or the `RefreshPlan` it would take, C41). *Why:* C++'s "priced
//! tabs are always refreshed" is one kind of intent silently rewriting
//! another; a report keeps the policy what its author wrote. Ruled
//! 2026-09-03; amended 2026-09-04.
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
//! what a hand price carries (a ratio → Q6; a retired tag → C68), then
//! whether the item can be addressed at all (the realm is one the site
//! lists, T4; a socketed item has no position, T13; a substash item's
//! link is unobserved, Q3; a stash item's tab must have been listed for
//! its index, T13; a character item needs its slot).
//!
//! **The link code and the price line.** A stash item renders as
//! `[linkItem location="Stash<index+1>" league="<L>" x="<x>" y="<y>"
//! realm="<r>"]` (T7, T13, T15 — `index + 1` is the C++ app's derivation;
//! which tab it names when folders occupy indices is Q1, and the forum's
//! preview shows the item picture before anything is posted); a
//! character item as `[linkItem location="<inventoryId>"
//! character="<name>" x="<x>" y="<y>" realm="<r>"]` (T7, T8). The price
//! is the line after the link — `~price <amount> <word>` or `~b/o
//! <amount> <word>` — with the currency table's `emit` word for the row's
//! tag (C68) and the amount's canonical text (C67). Both shapes were
//! accepted by the forum when the C++ app and Procurement wrote them
//! (T15); whether the site reads the price from the line after the code
//! is the wiki's statement (T14) and reading 2's to confirm.
//!
//! **Grouping and pages.** Entries are grouped by price — `~price`
//! before `~b/o`, then by tag, then by amount — in a stable sort over the
//! report's order, one blank line between groups. Pages are cut so that
//! each page, the template around it included, holds at most `size`
//! characters (the C++ constant 50,000 is the default; the forum's real
//! limit is Q4); a group runs on across a cut, and an entry that would
//! not fit an empty page is blocked (`page_size`). The template must hold
//! `[items]` exactly once; each page is the template with its body in
//! that place.
//!
//! **The C72 report** is over the posted items only — an omitted or
//! blocked item asks nothing of a refresh. *Coverage:* each posted item's
//! container against the sync policy's selection for this (realm,
//! league) — a tab by its own id or its parent's, the planner's rule
//! (C37, [`Selection::covers_tab`]); a character by its id. *Staleness:*
//! a posted item whose fact is older than the policy's window
//! (`max_age_seconds`), the same declaration the planner refreshes by.
//! *Positions:* a posted stash item whose fetch predates the stash
//! listing the page's `Stash<n>` comes from — the two halves of its link
//! were observed at different times, and a tab reindexed between them
//! would move the link (the "moved or reindexed" C72 names; without a
//! stored render basis, parked under "what did I last post", this is the
//! basis the render has). Each line names its remedy: the policy edit,
//! or the refresh — the caller passes the plan's request count
//! ([`PolicySource::Set`]), so the text can say what `acq refresh --plan`
//! would send without this module compiling one.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use acquisition_core::realm::Realm;
use serde::{Deserialize, Serialize};

use crate::currency::{self, CurrencyTableError};
use crate::listing::{Listing, ListingReport, ReportHeader, Side};
use crate::price::{Amount, Price, PriceTarget};
use crate::{Selection, SyncPolicy};

/// The render's JSON shape; changes are additive (C53) until they are
/// not, and then this moves. The shape is pinned by
/// `reference/shop-render-schema-1.json`
/// (`the_render_json_matches_the_committed_fixture`).
pub const SHOP_SCHEMA: u32 = 1;

/// The C++ app's post limit (T15) — a constant, not a measured limit (Q4).
pub const DEFAULT_PAGE_SIZE: usize = 50_000;

/// The token a template holds exactly once; the default template is the
/// token alone.
pub const ITEMS_TOKEN: &str = "[items]";

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
    NoSlot,
    PageSize,
    StashItem,
    CharacterItem,
}

impl Cell {
    pub const ALL: [Cell; 18] = [
        Cell::StashItem,
        Cell::CharacterItem,
        Cell::GameLists,
        Cell::GameSkips,
        Cell::HandSkip,
        Cell::NothingApplies,
        Cell::Unresolved,
        Cell::LeagueUnknown,
        Cell::HandNoPrice,
        Cell::Ratio,
        Cell::RetiredCurrency,
        Cell::RealmUnlisted,
        Cell::Socketed,
        Cell::NoPosition,
        Cell::Substash,
        Cell::TabUnlisted,
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
            Cell::NoSlot => "no_slot",
            Cell::PageSize => "page_size",
            Cell::StashItem => "stash_item",
            Cell::CharacterItem => "character_item",
        }
    }

    pub fn verdict(self) -> Verdict {
        match self {
            Cell::StashItem | Cell::CharacterItem => Verdict::Post,
            Cell::GameLists | Cell::GameSkips | Cell::HandSkip => Verdict::Omit,
            Cell::NothingApplies => Verdict::OffPage,
            Cell::Unresolved
            | Cell::LeagueUnknown
            | Cell::HandNoPrice
            | Cell::Ratio
            | Cell::RetiredCurrency
            | Cell::RealmUnlisted
            | Cell::Socketed
            | Cell::NoPosition
            | Cell::Substash
            | Cell::TabUnlisted
            | Cell::NoSlot
            | Cell::PageSize => Verdict::Block,
        }
    }

    /// The rule and its citation, one sentence.
    pub fn why(self) -> &'static str {
        match self {
            Cell::StashItem => {
                "a hand-priced stash item at a listed tab: `[linkItem location=\"Stash<index+1>\" league= x= y= realm=]`, the price on the next line (T7, T13, T15; a forum price over the tab's is T12; which tab `Stash<n>` names under folders is Q1 — the forum's preview shows the picture before posting)"
            }
            Cell::CharacterItem => {
                "a hand-priced character item in a slot: `[linkItem location=\"<inventoryId>\" character= x= y= realm=]`, the price on the next line (T7, T8)"
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
                "whether an unpriced forum link is indexed is unobserved (Q5): one hand experiment"
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
                "the link code for an item in a map or unique substash is unobserved (Q3): the owner's hand experiment at reading 2"
            }
            Cell::TabUnlisted => {
                "the tab is not on the current stash listing, so it has no index for `Stash<n>` (T13)"
            }
            Cell::NoSlot => "a character item without an `inventoryId` cannot be addressed (T13)",
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
    /// The price line, as written on the page.
    pub price: String,
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
        refresh_requests: Option<u64>,
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
    /// Containers of posted items the policy does not cover.
    pub uncovered: Vec<PriceTarget>,
    /// Posted items whose fact is older than the window.
    pub stale: Vec<PriceTarget>,
    /// The oldest posted fact's age, seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oldest_seconds: Option<i64>,
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
    /// Sort key: `~price` before `~b/o`, then tag, then amount.
    group: (u8, String, Amount),
    link: String,
    price: String,
    /// Basis, for the C72 report.
    seen_response: Option<i64>,
    seen_at: Option<i64>,
    /// The tab's parent (a folder), for coverage.
    parent: Option<String>,
}

/// The cell an item falls in, and — when it posts — its entry.
fn cell(
    l: &Listing,
    report: &ListingReport,
    table: &currency::CurrencyTable,
) -> Result<Entry, Cell> {
    let s = &l.subject;
    if s.league_unknown {
        return Err(Cell::LeagueUnknown);
    }
    let e = &l.effective;
    let price: &Price = match (e.side, e.kind.as_str()) {
        (Some(Side::Game), "skip") => return Err(Cell::GameSkips),
        (Some(Side::Game), _) => return Err(Cell::GameLists),
        (Some(Side::Manual), "skip") => return Err(Cell::HandSkip),
        (Some(Side::Manual), "no_price") => return Err(Cell::HandNoPrice),
        (Some(Side::Manual), _) => match &e.price {
            Some(p) => p,
            // A manual side with a priced kind carries its price; a
            // report that says otherwise is unreadable here.
            None => return Err(Cell::Unresolved),
        },
        (None, "unresolved") => return Err(Cell::Unresolved),
        (None, _) => return Err(Cell::NothingApplies),
    };
    if matches!(price.amount, Amount::Ratio { .. }) {
        return Err(Cell::Ratio);
    }
    let word = match table.by_tag(&price.currency) {
        Some(row) if !row.is_retired() => row.emit.clone(),
        // A row cites a tag the table holds (C67's write rule); one it
        // does not is a newer table's, unknown to this build.
        _ => return Err(Cell::RetiredCurrency),
    };
    let realm = report.header.realm;
    if realm == Realm::Poe2 {
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
    let prefix = if e.kind == "negotiable" {
        "~b/o"
    } else {
        "~price"
    };
    let price_line = format!("{prefix} {} {word}", price.amount);
    let group = (
        u8::from(e.kind == "negotiable"),
        price.currency.clone(),
        price.amount,
    );
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
                    "[linkItem location=\"{slot}\" character=\"{}\" x=\"{x}\" y=\"{y}\" realm=\"{}\"]",
                    character.subject.name,
                    realm.as_str()
                ),
                None,
            )
        }
        PriceTarget::Tab { .. } | PriceTarget::Item { .. } => {
            let Some(tab) = report.find(&location) else {
                return Err(Cell::TabUnlisted);
            };
            let Some(index) = tab.subject.index else {
                return Err(Cell::TabUnlisted);
            };
            let parent = tab.chain.get(1).and_then(|t| match t {
                PriceTarget::Tab { id, .. } => Some(id.clone()),
                _ => None,
            });
            (
                Cell::StashItem,
                format!(
                    "[linkItem location=\"Stash{}\" league=\"{}\" x=\"{x}\" y=\"{y}\" realm=\"{}\"]",
                    index + 1,
                    report.header.league,
                    realm.as_str()
                ),
                parent,
            )
        }
    };
    Ok(Entry {
        target: s.target.clone(),
        label: s.label(),
        location,
        cell,
        group,
        link,
        price: price_line,
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
    let mut entries: Vec<Entry> = Vec::new();
    let mut take = |counts: &mut ShopCounts, l: &Listing, cell: Cell| {
        *counts.by_cell.entry(cell).or_default() += 1;
        match cell.verdict() {
            Verdict::Post => counts.posted += 1,
            Verdict::Omit => counts.omitted += 1,
            Verdict::Block => counts.blocked += 1,
            Verdict::OffPage => counts.off_page += 1,
        }
        if cell.verdict() != Verdict::Post {
            left_out.push(LeftOut {
                target: l.subject.target.clone(),
                label: l.subject.label(),
                location: l.subject.location.clone(),
                cell,
                verdict: cell.verdict(),
            });
        }
    };
    for l in report.listings.iter().filter(|l| l.subject.is_item()) {
        counts.items += 1;
        match cell(l, report, table) {
            Ok(entry) => {
                let text_chars = entry.link.chars().count() + entry.price.chars().count() + 2;
                if text_chars + overhead > opts.size {
                    take(&mut counts, l, Cell::PageSize);
                } else {
                    take(&mut counts, l, entry.cell);
                    entries.push(entry);
                }
            }
            Err(cell) => take(&mut counts, l, cell),
        }
    }
    entries.sort_by_cached_key(|e| e.group.clone());

    // Pages: a running character count, the template's overhead in it;
    // a group runs on across a cut.
    let mut pages: Vec<(String, usize)> = Vec::new();
    let mut body = String::new();
    let mut body_chars = 0usize;
    let mut body_items = 0usize;
    let mut last_group: Option<&(u8, String, Amount)> = None;
    let mut posted = Vec::with_capacity(entries.len());
    for entry in &entries {
        let text = format!("{}\n{}\n", entry.link, entry.price);
        let text_chars = text.chars().count();
        let separator = usize::from(body_items > 0 && last_group != Some(&entry.group));
        if body_items > 0 && body_chars + separator + text_chars + overhead > opts.size {
            pages.push((std::mem::take(&mut body), body_items));
            body_chars = 0;
            body_items = 0;
        } else if separator == 1 {
            body.push('\n');
            body_chars += 1;
        }
        body.push_str(&text);
        body_chars += text_chars;
        body_items += 1;
        last_group = Some(&entry.group);
        posted.push(Posted {
            target: entry.target.clone(),
            label: entry.label.clone(),
            location: entry.location.clone(),
            cell: entry.cell,
            page: pages.len() + 1,
            price: entry.price.clone(),
            link: entry.link.clone(),
        });
    }
    if body_items > 0 {
        pages.push((body, body_items));
    }
    let of = pages.len();
    let pages: Vec<Page> = pages
        .into_iter()
        .enumerate()
        .map(|(i, (body, items))| {
            let text = opts.template.replacen(ITEMS_TOKEN, &body, 1);
            Page {
                number: i + 1,
                of,
                chars: text.chars().count(),
                items,
                text,
            }
        })
        .collect();
    counts.pages = of;

    let freshness = freshness(report, &entries, opts);
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
        posted,
        left_out,
        pages,
        freshness,
    })
}

/// The C72 report over the posted entries (module doc).
fn freshness(report: &ListingReport, entries: &[Entry], opts: &RenderOptions<'_>) -> Freshness {
    let mut out = Freshness {
        policy: "not_set".into(),
        policy_revision: None,
        policy_problem: None,
        window_seconds: None,
        refresh_requests: None,
        uncovered: Vec::new(),
        stale: Vec::new(),
        oldest_seconds: None,
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
        if let Some(at) = e.seen_at {
            let age = opts.now - at;
            if out.oldest_seconds.is_none_or(|o| age > o) {
                out.oldest_seconds = Some(age);
            }
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
            refresh_requests,
        } => {
            out.policy_revision = Some(*revision);
            out.refresh_requests = *refresh_requests;
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
        if !covered {
            uncovered.insert(e.location.clone());
        }
        if e.seen_at.is_some_and(|at| opts.now - at > window) {
            out.stale.push(e.target.clone());
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
    /// character.
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
        assert_eq!(c.items, 19);
        assert_eq!(c.posted + c.omitted + c.blocked + c.off_page, c.items);
        assert_eq!(c.posted, 5, "{:?}", r.posted);
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
        expect(Cell::HandSkip, &["i-skip"]);
        expect(Cell::NothingApplies, &["i-plain"]);
        expect(Cell::Unresolved, &["i-unres"]);
        expect(Cell::LeagueUnknown, &["i-nowhere"]);
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
    }

    /// C74, T7, T13, T15 — the two link shapes and the price line: a
    /// stash item by its tab's `index + 1`, a character item by its slot
    /// and the character's name; `~price` and `~b/o` with the table's
    /// word; grouped by price with a blank line between groups.
    #[test]
    fn c74_the_link_codes_and_price_lines_are_the_observed_shapes() {
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
            "[linkItem location=\"Stash2\" league=\"Standard\" x=\"1\" y=\"0\" realm=\"pc\"]"
        );
        assert_eq!(hand.price, "~price 5 chaos");
        let worn = find("i-worn");
        assert_eq!(
            worn.link,
            "[linkItem location=\"BodyArmour\" character=\"Exile\" x=\"0\" y=\"0\" realm=\"pc\"]"
        );
        assert_eq!(worn.price, "~price 10 divine");
        assert_eq!(find("i-bo").price, "~b/o 1.5 divine");
        assert_eq!(find("i-tabrow").price, "~b/o 2 divine");
        assert_eq!(r.pages.len(), 1);
        let page = &r.pages[0];
        assert_eq!(
            page.text,
            "[linkItem location=\"Stash2\" league=\"Standard\" x=\"1\" y=\"0\" realm=\"pc\"]\n\
             ~price 5 chaos\n\
             [linkItem location=\"Stash2\" league=\"Standard\" x=\"2\" y=\"0\" realm=\"pc\"]\n\
             ~price 5 chaos\n\
             \n\
             [linkItem location=\"BodyArmour\" character=\"Exile\" x=\"0\" y=\"0\" realm=\"pc\"]\n\
             ~price 10 divine\n\
             \n\
             [linkItem location=\"Stash2\" league=\"Standard\" x=\"3\" y=\"0\" realm=\"pc\"]\n\
             ~b/o 1.5 divine\n\
             \n\
             [linkItem location=\"Stash3\" league=\"Standard\" x=\"0\" y=\"1\" realm=\"pc\"]\n\
             ~b/o 2 divine\n"
        );
        assert_eq!(page.chars, page.text.chars().count());
        assert_eq!(page.items, 5);
        assert!(r.posted.iter().all(|p| p.page == 1));
    }

    /// C74 — pages are cut under the size with the template counted, a
    /// group runs on across the cut, every page is labelled n of N, the
    /// template wraps each page, an entry too large for an empty page is
    /// blocked, and a template without the token refuses.
    #[test]
    fn c74_pages_are_cut_under_the_size_and_carry_the_template() {
        let report = report();
        let template = "[spoiler]\n[items][/spoiler]\n";
        let overhead = template.chars().count() - ITEMS_TOKEN.len();
        let entry = |id: &str| {
            let full = render(&report, &opts(PolicySource::NotSet)).unwrap();
            let p = full
                .posted
                .iter()
                .find(|p| p.target.to_string() == format!("item/{id}"))
                .unwrap()
                .clone();
            p.link.chars().count() + p.price.chars().count() + 2
        };
        // Two entries of the first group fit; the third entry starts a
        // new page.
        let size = entry("i-hand") + entry("i-hand2") + overhead + 1;
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
        assert_eq!(r.counts.posted, 5);
        assert_eq!(r.counts.blocked, 9);
        assert_eq!(r.pages.len(), 3, "{:#?}", r.pages);
        assert_eq!(r.counts.pages, 3);
        for page in &r.pages {
            assert!(
                page.chars <= size,
                "page {} is {} > {size}",
                page.number,
                page.chars
            );
            assert_eq!(page.of, 3);
            assert!(page.text.starts_with("[spoiler]\n"));
            assert!(page.text.ends_with("[/spoiler]\n"));
            assert_eq!(
                r.posted.iter().filter(|p| p.page == page.number).count(),
                page.items
            );
        }
        assert_eq!(r.pages[0].items, 2);
        assert_eq!(r.pages.iter().map(|p| p.items).sum::<usize>(), 5);
        // Page numbers follow the group order.
        assert!(r.posted.windows(2).all(|w| w[0].page <= w[1].page));

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
        assert_eq!(r.counts.by_cell[&Cell::PageSize], 5);
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
                refresh_requests: Some(7),
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
        // now 5000: the c1 items were seen at 1200 (3800 s ago), over
        // the window; the t2 item at 100.
        assert_eq!(f.stale.len(), 5);
        assert_eq!(f.oldest_seconds, Some(4_900));
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
                refresh_requests: None,
            }),
        )
        .unwrap();
        assert!(r.freshness.uncovered.is_empty());
        assert!(r.freshness.stale.is_empty());
        assert_eq!(r.freshness.refresh_requests, None);

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
                refresh_requests: None,
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

    /// C53 — the JSON shape is the contract: the rendered fixture must
    /// serialize to exactly the committed document. A difference is
    /// either additive (regenerate with `ACQ_UPDATE_FIXTURES=1`) or a
    /// schema bump (`SHOP_SCHEMA`), decided by reading the diff.
    #[test]
    fn the_render_json_matches_the_committed_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/reference/shop-render-schema-1.json"
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
                refresh_requests: Some(7),
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
}
