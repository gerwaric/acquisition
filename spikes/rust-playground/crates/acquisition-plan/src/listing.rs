//! The listing state (C69, C70, C80): what every item, tab, substash and
//! character in one (realm, league) is listed as — the manual side, the
//! game side, and their relation — resolved by one pure function,
//! [`resolve`], over a [`PricingSnapshot`]. Built by the pricing slice's
//! plan step 4 (`PRICING-SLICE.md`, 2026-09-06). Nothing here reads a
//! store or writes anything; the CLI (`acq price status | show | list`,
//! `price_cmd.rs`) renders the report under C53.
//!
//! # Decisions as recorded
//!
//! **C69 — A listing is 2 independent resolutions, their relation, and the
//! effective price (C81).** The manual side resolves by specificity (C70);
//! the game side reads note, then tab name, as a price (exact or
//! negotiable; a ratio too, T2) or `skip` (T10); any other text — free
//! text, an empty amount, an unknown word, a ratio in a tab name — has no
//! effect (T11, T16, T18, T19), is shown verbatim, and the tab applies.
//! The relation is manual-only, game-only, agree, conflict or none. Each
//! result carries both sides with causes, revisions, basis, versions, the
//! raw texts. *Why:* four statements C++ fused and needed locks; the
//! site's display states are not modelled. *Details:* `game_side.rs`,
//! `listing.rs`. *Pinned:* the `c69_` tests. Amended 2026-09-06.
//!
//! **C70 — A priced tab covers that tab and its children, the way a policy
//! id does (C37);** a substash row overrides its parent's for that
//! substash; an item row overrides both; a character row covers its items.
//! Coverage is the manual side's inheritance only — it says nothing about
//! eligibility (C74). *Why:* the C++ store already lands substash items'
//! location prices on the parent, and the house rule for tab-scoped intent
//! should not have two shapes. Ruled 2026-09-03.
//!
//! **C81 — The effective price is the most specific statement, the game's
//! on a tie; only a public tab's game side is a statement.** Levels are
//! C70's; the game speaks
//! at item level (a note) and tab level (a name, C80). A note or name in a
//! non-public tab is invisible to the index (T1, T11) — residue, shown,
//! never a side. A prior forum post is never a side (T6). A row that
//! cannot be read, at a level that could decide, leaves the price
//! unresolved — never the game's by default. *Why:* acquisition's prices
//! reach the world only
//! through the forum, and a page must not contradict what the site
//! already shows; owner, 2026-09-06: "In-game prices should take priority
//! over acquisition prices." *Details:* `listing.rs`. *Pinned:* the `c81_`
//! tests. Ruled 2026-09-06.
//!
//! **C80 — The game side's tab for a nested item is the tab a user can name
//! and publish: a substash reads its parent, a folder child reads itself, a
//! folder's name is never a price.** A map or unique substash has the
//! game's own name and no `public`, so an item in one reads its note, then
//! the parent's name and `public`, the substash's name reported verbatim
//! beside it. A folder is a grouping, not a tab: its child's own name and
//! `public` are read. Provisional until the hand experiment
//! (`PRICING-SLICE.md`, observations) is run on the site. *Why:* the
//! owner's facts hold 64 substashes with no settable name or `public` and
//! 77 folder children as ordinary tabs; owner, 2026-09-06: "Folders never
//! carry price, but substash parents can." *Pinned:* the `c80_` tests.
//! Ruled 2026-09-06.
//!
//! # As built
//!
//! **One subject per row of the facts.** Every live tab (folders and
//! substash stubs included), every live character of the league, and
//! every live item at them gets a [`Listing`]; containers come first in
//! the snapshot's order, then items. A listing is its [`Subject`] (the
//! [`PriceTarget`] it would be priced at, plus what a line needs: name,
//! type line, stack, tab type, the container an item sits in), the two
//! sides, the relation with a sentence naming both sides, and the
//! [`Basis`] the facts came from (the fetch that last saw an item; the
//! listing that last named a tab or character). The report stamps the
//! parser and reference versions every reading was made under
//! ([`NOTE_PARSER_VERSION`], [`CURRENCY_TABLE_VERSION`]) and the snapshot's
//! bases, so a later reading of the same text can be told from a later
//! fact.
//!
//! **The manual side walks the specificity chain (C70)** and stops at the
//! first row it meets: an item's own row, then its substash's, then the
//! tab's, then the folder the tab sits in (a folder is a grouping, but a
//! row on it covers the tabs it groups the way a policy id does, C37); a
//! character item's own row, then its character's. A row on a target is
//! any `buyout` value, `skip` and `no_price` included — coverage is the
//! row's existence, never its type. The side names the row it took
//! (`from`, with `inherited` set when that is not the subject's own
//! target), its revision, when it was written and through what (C65), so
//! a `set` can cite the revision it read. A row that cannot be read (a
//! newer value schema, a malformed address) never yields to a less
//! specific one: the walk stops, `manual` is empty and `manual_problem`
//! says why — the report's `rows.unreadable` lists every such row once.
//! Rows whose realm is not the snapshot's are counted `other_realm`; valid
//! rows naming nothing in these facts (another league's item, a removed
//! tab) are listed `unmatched` — every nothing says which nothing (C53).
//!
//! **The game side is a statement only where the trade index can see it
//! (C81, T1).** For a subject in a public tab, [`game_side::read`] reads
//! the note, then the tab name C80 chooses (a top-level tab or a folder
//! child its own; a substash its parent's, its own name carried in
//! `substash_name`; a folder never), and the statement is the first that
//! reads as a price or `skip`: a note that reads as anything else — free
//! text, the dialog's empty-amount residue, an unknown word, a ratio the
//! site would refuse — has no effect and the tab applies (T18; the C++
//! app substituted the tab for a broken note, and so does the site).
//! Where there is no public tab — a non-public tab, a character's items,
//! a folder, a substash whose parent is not on record — the statement is
//! `none` whatever the texts say; a text that would have read as a price
//! or `skip` is marked `residue`, so a remove-only tab's old price is
//! shown but never a side. Both texts ride verbatim with their parser
//! readings beside the statement. `public` is the listing's
//! `metadata.public` being exactly `true` (absent means not public,
//! census 2c); it is `None` where there is no stash to publish.
//!
//! **The relation is over the two sides' presence and content.** `none`:
//! no row applies and the game states nothing. `manual_only` and
//! `game_only`: one side speaks. `agree`: both do and say the same thing —
//! the same price under the same prefix, or `skip` beside `~skip`.
//! `conflict`: both speak and differ. A row the walk could not read is
//! named in the sentence where "no row applies" would otherwise stand.
//! The relation describes; the effective price decides — and `list`'s
//! default selection is every relation but `none` plus every unresolved
//! item, so an unreadable row is never hidden by the relation word.
//!
//! **The effective price (C81)** is the more specific of the two
//! statements, the game's on a tie. A statement's level is its position
//! on the subject's chain: the manual side's is the row's target (an
//! item's own row is level 0, its substash's 1, its tab's 2, its folder's
//! 3; a character item's character is 1); the game's is 0 for a note and
//! the C80 tab's position for a tab name. So an item row beats a game tab
//! price, a substash row beats its parent's game name, a game note beats
//! an item row, a game tab price beats a tab row or a folder row.
//! [`Effective`] names the winner, its side, the target it came from and
//! why. A row the walk stopped at because it cannot be read is a
//! statement of unknown content: when it sits at a level that could beat
//! the game's (or there is no game statement) the effective price is
//! `unresolved`, naming the row — never the game's by default, since that
//! is the fall-through C++ had; when the game's statement is at least as
//! specific, the game decides as it would against any row there. What a
//! page does with the winner — omit what the game already lists — is the
//! render's (C74).
//!
//! **League unknown.** The store carries a character the listing gave no
//! league under every league of its realm (the planner's rule, so every
//! plan can report it as outside coverage); the pricing snapshot inherits
//! that read. Here such a character and its items are flagged
//! `league_unknown` and counted, in every league's report: the report is
//! not evidence they belong to its league, and a render blocks them by
//! name rather than placing them on a page. None of the owner's
//! characters is one (census 2c); the flag exists for the store's rule,
//! not for a case in hand.
//!
//! **Two item sets per container, and the views.** Every listing carries
//! its manual chain, so a container has two sets: the items physically in
//! it ([`ListingReport::items_in`]) and the items a row on it would cover
//! ([`ListingReport::items_covered_by`]) — the same for an ordinary tab,
//! different for a folder (nothing here, its tabs' items covered) and for
//! a map or unique tab (its own items here, its substashes' covered too).
//! [`ListView`] and [`ShowView`] are `list`'s and `show`'s JSON contracts
//! and everything their text is a function of (C53): the header, the
//! selection, the containers the selected items sit in, both item sets.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use acquisition_core::realm::Realm;
use acquisition_store::{
    AnnotationRow, CharacterSnapshot, ItemSnapshot, ListingBasis, PricingSnapshot, TabSnapshot,
    ValueError, check_value,
};
use serde::{Deserialize, Serialize};

use crate::currency::{self, CURRENCY_TABLE_VERSION, CurrencyTable, CurrencyTableError};
use crate::game_side::{self, GamePrice, NOTE_PARSER_VERSION, Source};
use crate::price::{Buyout, Price, PriceTarget};

/// The report's JSON shape, stamped on every [`ListingReport`] and view;
/// changes are additive (C53) until they are not, and then this moves.
/// **2** since 2026-09-06: schema 1 never left development, and the
/// second step-4 review's count fields (`by_game_statement`,
/// `game_priced`) replaced rather than joined `by_game_reading`,
/// `game_priced_public` and `game_priced_not_public` — a rename, so a
/// bump, not a compatibility shim nobody would read. The shape is
/// pinned by `reference/listing-report-schema-2.json`
/// (`the_report_json_matches_the_committed_fixture`); a change that
/// fails it is either additive (regenerate the fixture) or a bump.
pub const LISTING_SCHEMA: u32 = 2;

/// The GGG tab type that groups tabs and holds no items.
const FOLDER: &str = "Folder";

/// What a listing is about, and what a line about it needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    /// The address a row on this subject itself would have.
    pub target: PriceTarget,
    /// The item's name (empty for most non-unique items), the tab's, or
    /// the character's.
    pub name: String,
    /// Items: the type line. Empty for a container.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub type_line: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_size: Option<i64>,
    /// Tabs and substashes: the GGG type (`PremiumStash`, `MapStash`,
    /// `Folder`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_type: Option<String>,
    /// Items: the container they sit in — a tab, substash or character.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<PriceTarget>,
    /// Items: the array they came from (`items`, `equipment`, …).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    /// Items: the item this one is socketed in; such an item has no
    /// position of its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socketed_in: Option<String>,
    /// A character the listing gave no league, and its items: the store
    /// carries them under every league of the realm, so this report is
    /// not evidence they belong to its league (module doc, "League
    /// unknown").
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub league_unknown: bool,
}

impl Subject {
    /// An item, as opposed to a container.
    pub fn is_item(&self) -> bool {
        matches!(self.target, PriceTarget::Item { .. })
    }

    /// The name a line shows: the name, else the type line, else the
    /// address.
    pub fn label(&self) -> String {
        if !self.name.is_empty() {
            self.name.clone()
        } else if !self.type_line.is_empty() {
            self.type_line.clone()
        } else {
            self.target.to_string()
        }
    }
}

/// The manual side: the row that applies, by specificity (C70).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualSide {
    pub value: Buyout,
    /// The row's own target — the subject's, or the container's it
    /// inherits from.
    pub from: PriceTarget,
    pub inherited: bool,
    pub revision: i64,
    pub updated_at: i64,
    pub written_via: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
}

/// One text the game side read, verbatim, and what it read as.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reading {
    pub text: String,
    pub reading: GamePrice,
}

/// The game side: what the note, then the tab name, says (C69), with
/// both texts verbatim, the tab C80 chose, and whether it is public.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSide {
    /// The reading that applies: the note's unless it is `none`, else
    /// the tab name's, else `none`.
    pub reading: GamePrice,
    /// Where `reading` came from; absent when it is `none`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The item's note, verbatim, with its own reading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<Reading>,
    /// The tab whose name and `public` were read (C80): the subject's own
    /// tab, or a substash's parent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab: Option<String>,
    /// That tab's name, verbatim, with its own reading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tab_name: Option<Reading>,
    /// A substash's own name, verbatim, beside the parent's (C80).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substash_name: Option<String>,
    /// The listing's `metadata.public`, exactly `true`; `None` where
    /// there is no stash to publish.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
    /// A note or tab name that reads as a price or `skip` where the
    /// index cannot see it (no public tab): shown, never a statement
    /// (C81).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub residue: bool,
}

/// How the two sides stand to each other (C69).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    None,
    ManualOnly,
    GameOnly,
    Agree,
    Conflict,
}

impl Relation {
    pub const ALL: [Relation; 5] = [
        Relation::None,
        Relation::ManualOnly,
        Relation::GameOnly,
        Relation::Agree,
        Relation::Conflict,
    ];

    /// The word, as in JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Relation::None => "none",
            Relation::ManualOnly => "manual_only",
            Relation::GameOnly => "game_only",
            Relation::Agree => "agree",
            Relation::Conflict => "conflict",
        }
    }

    pub fn parse(s: &str) -> Option<Relation> {
        Relation::ALL.into_iter().find(|r| r.as_str() == s)
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whose statement the effective price is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Game,
    Manual,
}

impl Side {
    pub fn as_str(self) -> &'static str {
        match self {
            Side::Game => "game",
            Side::Manual => "manual",
        }
    }
}

/// What applies (C81): the more specific statement, the game's on a tie.
/// A consumer reads `side` and `kind` together — `none` and `unresolved`
/// both carry no side, and a manual `skip` is not a price to post
/// (`PRICING-SLICE.md`, the render handoff).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Effective {
    /// `exact`, `negotiable`, `no_price`, `skip`, `none`, or `unresolved`
    /// (a row that cannot be read could decide; `from` names it).
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub side: Option<Side>,
    /// Where the winning statement sits: the row's target, or the item
    /// whose note or the tab whose name the game read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<PriceTarget>,
    /// One sentence naming the winner and the rule that chose it.
    pub why: String,
}

impl Effective {
    /// The side word `Counts::by_effective` and `ListFilter::effective`
    /// use: `game`, `manual`, `none` or `unresolved`.
    pub fn side_word(&self) -> &'static str {
        match (self.side, self.kind.as_str()) {
            (Some(side), _) => side.as_str(),
            (None, "unresolved") => "unresolved",
            (None, _) => "none",
        }
    }
}

impl fmt::Display for Effective {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.price, self.kind.as_str()) {
            (Some(p), "negotiable") => write!(f, "{p} b/o"),
            (Some(p), _) => write!(f, "{p}"),
            (None, "no_price") => f.write_str("no price"),
            (None, kind) => f.write_str(kind),
        }
    }
}

/// The fact the subject's texts came from: the fetch that last saw an
/// item, or the listing that last named a tab or character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Basis {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<i64>,
}

/// One subject's listing state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listing {
    pub subject: Subject,
    /// The manual side's walk (C70), most specific first: the subject's
    /// own target, then each container whose row would cover it. A row
    /// on any of these is what `manual` reports; a target's coverage is
    /// every listing whose chain holds it ([`ListingReport::items_covered_by`]).
    pub chain: Vec<PriceTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual: Option<ManualSide>,
    /// A row that would apply could not be read; the walk stopped there.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_problem: Option<String>,
    pub game: GameSide,
    pub relation: Relation,
    /// One sentence naming both sides.
    pub why: String,
    /// What applies (C81).
    pub effective: Effective,
    pub basis: Basis,
}

/// A `buyout` row the state could not use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RowProblem {
    pub scope: String,
    pub key: String,
    pub revision: i64,
    pub why: String,
}

/// Where every `buyout` row of the account went.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RowAccounting {
    pub total: usize,
    /// Rows whose target is a subject of these facts.
    pub applied: usize,
    /// Tab and substash rows of another realm.
    pub other_realm: usize,
    /// Readable rows naming nothing in these facts.
    pub unmatched: Vec<PriceTarget>,
    pub unreadable: Vec<RowProblem>,
}

/// Counts over the items (containers are not items), so `status` needs
/// no listing to say its line.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Counts {
    pub items: usize,
    pub containers: usize,
    /// Items by relation word.
    pub by_relation: BTreeMap<String, usize>,
    /// Items by the game side's statement (`exact`, `negotiable`, `skip`,
    /// `none`) — a statement exists only in a public tab (C81); a note's
    /// raw reading is `invalid_notes` and `residue`.
    pub by_game_statement: BTreeMap<String, usize>,
    /// Items whose note reads `invalid` (no effect, shown).
    pub invalid_notes: usize,
    /// Items with a game-side price statement (exact or negotiable);
    /// public by construction.
    pub game_priced: usize,
    /// Tabs (folders and substashes excluded) whose own name reads as a
    /// price, public or not; `priced_tabs_public` are the statements.
    pub priced_tabs: usize,
    pub priced_tabs_public: usize,
    /// Items whose manual side is inherited.
    pub inherited: usize,
    /// Items of a character the listing gave no league.
    pub league_unknown: usize,
    /// Items by the effective price's side: `game`, `manual`, `none`,
    /// `unresolved` (a row that cannot be read could decide).
    pub by_effective: BTreeMap<String, usize>,
    /// Items whose price text the index cannot see (C81).
    pub residue: usize,
}

/// What every view of the state says first: whose facts, which
/// (realm, league), when, on which bases, under which versions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportHeader {
    pub schema: u32,
    pub account_uuid: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_name: Option<String>,
    pub realm: Realm,
    pub league: String,
    pub taken_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stash_listing: Option<ListingBasis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_listing: Option<ListingBasis>,
    pub note_parser_version: u32,
    pub currency_table_version: u32,
}

/// The listing state of one (realm, league): every subject, the row
/// accounting, the counts, and the bases and versions it was resolved
/// under.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListingReport {
    #[serde(flatten)]
    pub header: ReportHeader,
    pub counts: Counts,
    pub rows: RowAccounting,
    /// Containers first (tabs in listing order, then characters), then
    /// items in the snapshot's order.
    pub listings: Vec<Listing>,
}

/// Which items `list` selects: by relation, by the effective price's
/// side (when neither is given: every relation but `none`, plus every
/// unresolved item whatever its relation), physically in one container,
/// or covered by a row on one target (C70) — the two are different sets
/// for a folder or a parent tab.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ListFilter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<Relation>,
    /// The effective price's side: `game`, `manual`, `none` or
    /// `unresolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#in: Option<PriceTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub covered_by: Option<PriceTarget>,
}

/// `list`'s contract: everything its text is a function of (C53) — the
/// header, the filter, the containers on the selected items' chains (the
/// one each sits in and those above it, in the report's order, each with
/// its own listing), and the items.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListView {
    #[serde(flatten)]
    pub header: ReportHeader,
    pub filter: ListFilter,
    /// How many items the report holds, so an empty selection can say
    /// which nothing it is.
    pub items_on_record: usize,
    pub containers: Vec<Listing>,
    pub items: Vec<Listing>,
}

/// `show`'s contract: the listing, the container an item sits in, and
/// for a container both item sets — physically here, and covered by a
/// row on it (C70).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShowView {
    #[serde(flatten)]
    pub header: ReportHeader,
    pub listing: Listing,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<Listing>,
    /// A container's items; empty for an item.
    pub items_here: Vec<Listing>,
    /// Items whose chain holds this target but that do not sit in it: a
    /// folder's, through its tabs; a parent tab's, through its substashes.
    /// Empty for an item.
    pub items_covered_below: Vec<Listing>,
}

impl ListingReport {
    /// The listing of one target, if it is a subject of these facts.
    pub fn find(&self, target: &PriceTarget) -> Option<&Listing> {
        self.listings.iter().find(|l| &l.subject.target == target)
    }

    /// The item listings sitting in one container.
    pub fn items_in<'a>(&'a self, location: &'a PriceTarget) -> impl Iterator<Item = &'a Listing> {
        self.listings
            .iter()
            .filter(move |l| l.subject.location.as_ref() == Some(location))
    }

    /// The item listings a row on `target` would cover (C70): every item
    /// whose chain holds it, its own items and its children's.
    pub fn items_covered_by<'a>(
        &'a self,
        target: &'a PriceTarget,
    ) -> impl Iterator<Item = &'a Listing> {
        self.listings
            .iter()
            .filter(move |l| l.subject.is_item() && l.chain.contains(target))
    }

    /// The report without its listings: what `status` is.
    pub fn summary(&self) -> ListingReport {
        ListingReport {
            listings: Vec::new(),
            ..self.clone()
        }
    }

    /// `list`'s view under a filter. `None` when the filter names a
    /// container that is not a subject of these facts — a refusal, never
    /// an empty selection.
    pub fn list_view(&self, filter: ListFilter) -> Option<ListView> {
        for named in [&filter.r#in, &filter.covered_by].into_iter().flatten() {
            self.find(named)?;
        }
        let items: Vec<Listing> = self
            .listings
            .iter()
            .filter(|l| l.subject.is_item())
            // The default suppression of `none` applies only when neither
            // selector was given: `--effective none` must be able to select
            // exactly the items nothing applies to.
            .filter(|l| match (filter.relation, &filter.effective) {
                (Some(rel), _) => l.relation == rel,
                (None, Some(_)) => true,
                (None, None) => l.relation != Relation::None || l.effective.kind == "unresolved",
            })
            .filter(|l| {
                filter
                    .effective
                    .as_deref()
                    .is_none_or(|side| l.effective.side_word() == side)
            })
            .filter(|l| {
                filter
                    .r#in
                    .as_ref()
                    .is_none_or(|loc| l.subject.location.as_ref() == Some(loc))
            })
            .filter(|l| {
                filter
                    .covered_by
                    .as_ref()
                    .is_none_or(|t| l.chain.contains(t))
            })
            .cloned()
            .collect();
        // Every container on a selected item's chain — the one it sits
        // in and the ones above it — so a substash's parent, whose name
        // is the one read (C80), is in the document the text renders.
        let containers: Vec<Listing> = self
            .listings
            .iter()
            .filter(|c| !c.subject.is_item())
            .filter(|c| items.iter().any(|l| l.chain.contains(&c.subject.target)))
            .cloned()
            .collect();
        Some(ListView {
            header: self.header.clone(),
            filter,
            items_on_record: self.counts.items,
            containers,
            items,
        })
    }

    /// `show`'s view of one target, if it is a subject of these facts.
    pub fn show_view(&self, target: &PriceTarget) -> Option<ShowView> {
        let listing = self.find(target)?.clone();
        let container = listing
            .subject
            .location
            .as_ref()
            .and_then(|loc| self.find(loc))
            .cloned();
        // The two sets belong to a container; an item has neither.
        let (items_here, items_covered_below) = if listing.subject.is_item() {
            (Vec::new(), Vec::new())
        } else {
            (
                self.items_in(target).cloned().collect(),
                self.items_covered_by(target)
                    .filter(|l| l.subject.location.as_ref() != Some(target))
                    .cloned()
                    .collect(),
            )
        };
        Some(ShowView {
            header: self.header.clone(),
            listing,
            container,
            items_here,
            items_covered_below,
        })
    }
}

/// Why a snapshot could not be resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListingError {
    /// The shipped currency table is not usable.
    Currency(CurrencyTableError),
    /// The snapshot's realm is not a word [`Realm`] knows.
    UnknownRealm { realm: String },
}

impl fmt::Display for ListingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ListingError::Currency(e) => write!(f, "currency table: {e}"),
            ListingError::UnknownRealm { realm } => write!(f, "{realm:?} is not a realm"),
        }
    }
}

impl std::error::Error for ListingError {}

/// A stored row, read through the typed door (C66).
struct ManualRow {
    value: Buyout,
    revision: i64,
    updated_at: i64,
    written_via: String,
    actor: Option<String>,
}

/// The rows, by target, with what could not be indexed.
struct Rows {
    by_target: BTreeMap<PriceTarget, ManualRow>,
    /// Targets whose row exists but could not be read, with why.
    unreadable: BTreeMap<PriceTarget, String>,
    accounting: RowAccounting,
}

fn index_rows(rows: &[AnnotationRow], realm: Realm) -> Rows {
    let mut out = Rows {
        by_target: BTreeMap::new(),
        unreadable: BTreeMap::new(),
        accounting: RowAccounting {
            total: rows.len(),
            ..RowAccounting::default()
        },
    };
    for row in rows {
        let problem = |why: String| RowProblem {
            scope: row.scope.clone(),
            key: row.key.clone(),
            revision: row.revision,
            why,
        };
        let target = match PriceTarget::from_address(&row.scope, &row.key) {
            Ok(t) => t,
            Err(e) => {
                out.accounting.unreadable.push(problem(e.to_string()));
                continue;
            }
        };
        match &target {
            PriceTarget::Tab { realm: r, .. } | PriceTarget::Substash { realm: r, .. }
                if *r != realm =>
            {
                out.accounting.other_realm += 1;
                continue;
            }
            _ => {}
        }
        match check_value::<Buyout>(&row.value) {
            Ok(value) => {
                out.by_target.insert(
                    target,
                    ManualRow {
                        value,
                        revision: row.revision,
                        updated_at: row.updated_at,
                        written_via: row.written_via.clone(),
                        actor: row.actor.clone(),
                    },
                );
            }
            Err(e) => {
                let why = match &e {
                    ValueError::VersionUnsupported { .. } => {
                        format!("{e} — a newer build wrote it")
                    }
                    _ => e.to_string(),
                };
                out.accounting.unreadable.push(problem(why.clone()));
                out.unreadable.insert(target, why);
            }
        }
    }
    out
}

/// How a tab sits in the stash (the store's rule: a parent that is a
/// folder makes a folder child; any other parent makes a substash).
#[derive(Clone, Copy, PartialEq, Eq)]
enum TabKind {
    Folder,
    Top,
    FolderChild,
    Substash,
}

struct TabInfo<'a> {
    tab: &'a TabSnapshot,
    kind: TabKind,
    target: PriceTarget,
    /// The manual side's walk from this tab up (C70): itself, then its
    /// parent tab (a substash), then the folder.
    chain: Vec<PriceTarget>,
    /// The tab whose name and `public` the game side reads (C80).
    game_tab: Option<&'a TabSnapshot>,
}

fn tab_infos<'a>(tabs: &'a [TabSnapshot], realm: Realm) -> Vec<TabInfo<'a>> {
    let by_id: HashMap<&str, &TabSnapshot> = tabs.iter().map(|t| (t.id.as_str(), t)).collect();
    let tab_target = |id: &str| PriceTarget::Tab {
        realm,
        id: id.into(),
    };
    tabs.iter()
        .map(|tab| {
            let parent = tab.parent.as_deref().map(|p| (p, by_id.get(p).copied()));
            let kind = if tab.r#type == FOLDER {
                TabKind::Folder
            } else {
                match parent {
                    None => TabKind::Top,
                    Some((_, Some(p))) if p.r#type == FOLDER => TabKind::FolderChild,
                    Some(_) => TabKind::Substash,
                }
            };
            let (target, chain, game_tab) = match kind {
                TabKind::Substash => {
                    let (parent_id, parent_tab) = parent.unwrap_or(("", None));
                    let target = PriceTarget::Substash {
                        realm,
                        parent: parent_id.into(),
                        id: tab.id.clone(),
                    };
                    let mut chain = vec![target.clone(), tab_target(parent_id)];
                    if let Some(folder) = parent_tab.and_then(|p| p.parent.as_deref()) {
                        chain.push(tab_target(folder));
                    }
                    (target, chain, parent_tab)
                }
                TabKind::FolderChild => {
                    let target = tab_target(&tab.id);
                    let mut chain = vec![target.clone()];
                    if let Some((folder, _)) = parent {
                        chain.push(tab_target(folder));
                    }
                    (target, chain, Some(tab))
                }
                TabKind::Top => (tab_target(&tab.id), vec![tab_target(&tab.id)], Some(tab)),
                TabKind::Folder => (tab_target(&tab.id), vec![tab_target(&tab.id)], None),
            };
            TabInfo {
                tab,
                kind,
                target,
                chain,
                game_tab,
            }
        })
        .collect()
}

fn is_public(tab: &TabSnapshot) -> bool {
    tab.metadata
        .get("public")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
}

/// Whether a text's reading is a statement the site would act on: a
/// price or `skip`.
fn speaks(reading: &GamePrice) -> bool {
    matches!(
        reading,
        GamePrice::Exact(_) | GamePrice::Negotiable(_) | GamePrice::Skip
    )
}

/// The tab half of a game side: the C80 tab's name read as a tab name,
/// and its `public`. The name is a statement only when the tab is
/// public (C81); otherwise it is residue at most.
fn tab_game_side(info: &TabInfo<'_>, table: &CurrencyTable) -> GameSide {
    let substash_name = (info.kind == TabKind::Substash).then(|| info.tab.name.clone());
    let mut side = GameSide {
        reading: GamePrice::None,
        source: None,
        note: None,
        tab: None,
        tab_name: None,
        substash_name,
        public: None,
        residue: false,
    };
    if let Some(t) = info.game_tab {
        let reading = game_side::read(Source::TabName, &t.name, table);
        let public = is_public(t);
        if speaks(&reading) {
            if public {
                side.reading = reading.clone();
                side.source = Some(Source::TabName);
            } else {
                side.residue = true;
            }
        }
        side.tab = Some(t.id.clone());
        side.tab_name = Some(Reading {
            text: t.name.clone(),
            reading,
        });
        side.public = Some(public);
    }
    side
}

/// The note in front of the tab half (C69): a note that reads as a price
/// or `skip` is the statement where the tab is public; any other note
/// has no effect and the tab applies (T18). Where there is no public
/// tab the note is residue at most.
fn with_note(mut side: GameSide, note: Option<&str>, table: &CurrencyTable) -> GameSide {
    if let Some(text) = note {
        let reading = game_side::read(Source::Note, text, table);
        if speaks(&reading) {
            if side.public == Some(true) {
                side.reading = reading.clone();
                side.source = Some(Source::Note);
            } else {
                side.residue = true;
            }
        }
        side.note = Some(Reading {
            text: text.into(),
            reading,
        });
    }
    side
}

/// A row the walk stopped at because it cannot be read: where it sits
/// and why.
struct Unreadable {
    target: PriceTarget,
    why: String,
}

/// The manual side's walk (C70): the first row on the chain, or the
/// row that stopped it.
fn manual_side(chain: &[PriceTarget], rows: &Rows) -> (Option<ManualSide>, Option<Unreadable>) {
    for (i, target) in chain.iter().enumerate() {
        if let Some(row) = rows.by_target.get(target) {
            return (
                Some(ManualSide {
                    value: row.value.clone(),
                    from: target.clone(),
                    inherited: i > 0,
                    revision: row.revision,
                    updated_at: row.updated_at,
                    written_via: row.written_via.clone(),
                    actor: row.actor.clone(),
                }),
                None,
            );
        }
        if let Some(why) = rows.unreadable.get(target) {
            return (
                None,
                Some(Unreadable {
                    target: target.clone(),
                    why: why.clone(),
                }),
            );
        }
    }
    (None, None)
}

/// The relation and its sentence (module doc, "The relation"). A row the
/// walk could not read is named in the sentence: "no row applies" would
/// be false, and the effective price says what the row could do.
fn relate(
    manual: Option<&Buyout>,
    unreadable: Option<&Unreadable>,
    game: &GamePrice,
) -> (Relation, String) {
    let unread = |tail: &str| match unreadable {
        Some(u) => format!("the row on {} cannot be read{tail}", u.target),
        None => format!("no row applies{tail}"),
    };
    match (manual, game) {
        (None, GamePrice::None) => (Relation::None, unread(" and the game states nothing")),
        (Some(m), GamePrice::None) => (
            Relation::ManualOnly,
            format!("by hand: {m}; the game states nothing"),
        ),
        (None, g) => (Relation::GameOnly, format!("in game: {g}; {}", unread(""))),
        (Some(m), g) => {
            let agree = match (m, g) {
                (Buyout::Exact(a), GamePrice::Exact(b)) => a == b,
                (Buyout::Negotiable(a), GamePrice::Negotiable(b)) => a == b,
                (Buyout::Skip, GamePrice::Skip) => true,
                _ => false,
            };
            if agree {
                (Relation::Agree, format!("by hand and in game: {g}"))
            } else {
                (Relation::Conflict, format!("by hand: {m}; in game: {g}"))
            }
        }
    }
}

/// The effective price (C81): the more specific statement, the game's on
/// a tie, each statement's level being its target's position on the
/// chain.
fn effective(
    chain: &[PriceTarget],
    manual: Option<&ManualSide>,
    unreadable: Option<&Unreadable>,
    game: &GameSide,
    subject: &PriceTarget,
) -> Effective {
    let level_of = |t: &PriceTarget| chain.iter().position(|c| c == t).unwrap_or(chain.len());
    let game_from: Option<PriceTarget> = match game.source {
        None => None,
        Some(Source::Note) => Some(subject.clone()),
        Some(Source::TabName) => {
            let id = game.tab.clone().unwrap_or_default();
            chain
                .iter()
                .find(|t| matches!(t, PriceTarget::Tab { id: i, .. } if *i == id))
                .cloned()
        }
    };
    let game_level = game_from.as_ref().map(&level_of);
    let from_manual = |m: &ManualSide, why: String| Effective {
        kind: m.value.kind().into(),
        price: m.value.price().cloned(),
        side: Some(Side::Manual),
        from: Some(m.from.clone()),
        why,
    };
    let from_game = |why: String| Effective {
        kind: game.reading.kind().into(),
        price: game.reading.price().cloned(),
        side: Some(Side::Game),
        from: game_from.clone(),
        why,
    };
    let level_word = |t: &PriceTarget| match t {
        PriceTarget::Item { .. } => "item",
        PriceTarget::Substash { .. } => "substash",
        PriceTarget::Tab { .. } => "tab",
        PriceTarget::Character { .. } => "character",
    };
    // A row that cannot be read is a statement of unknown content: if it
    // could beat the game's statement, nothing is decided here.
    if let Some(u) = unreadable {
        let ul = level_of(&u.target);
        let could_decide = match game_level {
            None => true,
            Some(gl) => ul < gl,
        };
        if could_decide {
            return Effective {
                kind: "unresolved".into(),
                price: None,
                side: None,
                from: Some(u.target.clone()),
                why: format!(
                    "unresolved: the row on {} cannot be read ({}) and would decide{} (C81)",
                    u.target,
                    u.why,
                    match game_level {
                        None => "",
                        Some(_) => " over the game's less specific statement",
                    }
                ),
            };
        }
    }
    match (manual, game_level) {
        (None, None) => Effective {
            kind: "none".into(),
            price: None,
            side: None,
            from: None,
            why: if game.residue {
                "nothing applies: the price text in game is not where the index can see it (C81)"
                    .into()
            } else {
                "nothing applies".into()
            },
        },
        (Some(m), None) => from_manual(
            m,
            if game.residue {
                format!(
                    "{} by hand (row on {}); the price text in game is not where the index can see it (C81)",
                    m.value, m.from
                )
            } else {
                format!(
                    "{} by hand (row on {}); the game states nothing",
                    m.value, m.from
                )
            },
        ),
        (None, Some(_)) => from_game(match unreadable {
            Some(u) => format!(
                "{} in game: the game wins against the row on {}, readable or not, at its level (C81)",
                game.reading, u.target
            ),
            None => format!("{} in game; no row applies", game.reading),
        }),
        (Some(m), Some(gl)) => {
            let ml = level_of(&m.from);
            let gf = game_from.clone().unwrap_or_else(|| subject.clone());
            if ml < gl {
                from_manual(
                    m,
                    format!(
                        "{} by hand: the row on the {} is more specific than the game's {} name (C81)",
                        m.value,
                        level_word(&m.from),
                        level_word(&gf)
                    ),
                )
            } else if gl < ml {
                from_game(format!(
                    "{} in game: the {} is more specific than the row on the {} (C81)",
                    game.reading,
                    match game.source {
                        Some(Source::Note) => "note".to_string(),
                        _ => format!("{} name", level_word(&gf)),
                    },
                    level_word(&m.from)
                ))
            } else {
                from_game(format!(
                    "{} in game: the game wins a tie with the row on the {} (C81)",
                    game.reading,
                    level_word(&m.from)
                ))
            }
        }
    }
}

fn listing(
    subject: Subject,
    chain: &[PriceTarget],
    rows: &Rows,
    game: GameSide,
    basis: Basis,
) -> Listing {
    let (manual, unreadable) = manual_side(chain, rows);
    let (relation, why) = relate(
        manual.as_ref().map(|m| &m.value),
        unreadable.as_ref(),
        &game.reading,
    );
    let effective = effective(
        chain,
        manual.as_ref(),
        unreadable.as_ref(),
        &game,
        &subject.target,
    );
    let manual_problem =
        unreadable.map(|u| format!("the row on {} cannot be read: {}", u.target, u.why));
    Listing {
        subject,
        chain: chain.to_vec(),
        manual,
        manual_problem,
        game,
        relation,
        why,
        effective,
        basis,
    }
}

fn character_subject(c: &CharacterSnapshot) -> Subject {
    Subject {
        target: PriceTarget::Character { id: c.id.clone() },
        name: c.name.clone(),
        type_line: String::new(),
        stack_size: None,
        tab_type: None,
        location: None,
        container: None,
        socketed_in: None,
        league_unknown: c.league.is_none(),
    }
}

fn item_subject(item: &ItemSnapshot, location: PriceTarget, league_unknown: bool) -> Subject {
    Subject {
        target: PriceTarget::Item {
            id: item.id.clone(),
        },
        name: item.name.clone(),
        type_line: item.type_line.clone(),
        stack_size: item.stack_size,
        tab_type: None,
        location: Some(location),
        container: item.container.clone(),
        socketed_in: item.socketed_in.clone(),
        league_unknown,
    }
}

/// Resolve the listing state of one snapshot: every subject's two sides
/// and relation, the row accounting, the counts. Pure — the snapshot and
/// the shipped currency table in, the report out; the only failures are
/// a table this build cannot load and a realm word it does not know.
pub fn resolve(snapshot: &PricingSnapshot) -> Result<ListingReport, ListingError> {
    let table = currency::table().map_err(ListingError::Currency)?;
    let realm = Realm::parse(&snapshot.realm).ok_or_else(|| ListingError::UnknownRealm {
        realm: snapshot.realm.clone(),
    })?;
    let rows = index_rows(&snapshot.buyouts, realm);
    let tabs = tab_infos(&snapshot.tabs, realm);
    let tab_by_id: HashMap<&str, &TabInfo<'_>> =
        tabs.iter().map(|i| (i.tab.id.as_str(), i)).collect();
    let character_by_id: HashMap<&str, &CharacterSnapshot> = snapshot
        .characters
        .iter()
        .map(|c| (c.id.as_str(), c))
        .collect();

    let mut listings =
        Vec::with_capacity(tabs.len() + snapshot.characters.len() + snapshot.items.len());
    let mut counts = Counts::default();
    for info in &tabs {
        let subject = Subject {
            target: info.target.clone(),
            name: info.tab.name.clone(),
            type_line: String::new(),
            stack_size: None,
            tab_type: Some(info.tab.r#type.clone()),
            location: None,
            container: None,
            socketed_in: None,
            league_unknown: false,
        };
        let game = tab_game_side(info, table);
        // A name that reads as a price, public or not: the owner's
        // remove-only tabs are the case (census 2c), and the public
        // count beside it says how many the index can see.
        if info.kind != TabKind::Folder
            && info.kind != TabKind::Substash
            && game
                .tab_name
                .as_ref()
                .is_some_and(|t| t.reading.price().is_some())
        {
            counts.priced_tabs += 1;
            if game.public == Some(true) {
                counts.priced_tabs_public += 1;
            }
        }
        let basis = Basis {
            response: info.tab.listed_response,
            at: info.tab.listed_at,
        };
        listings.push(listing(subject, &info.chain, &rows, game, basis));
    }
    for c in &snapshot.characters {
        let subject = character_subject(c);
        let chain = [subject.target.clone()];
        let game = GameSide {
            reading: GamePrice::None,
            source: None,
            note: None,
            tab: None,
            tab_name: None,
            substash_name: None,
            public: None,
            residue: false,
        };
        let basis = Basis {
            response: c.listed_response,
            at: c.listed_at,
        };
        listings.push(listing(subject, &chain, &rows, game, basis));
    }
    counts.containers = listings.len();
    for item in &snapshot.items {
        let own = PriceTarget::Item {
            id: item.id.clone(),
        };
        let (location, chain, game) = if item.location_kind == "character" {
            let location = PriceTarget::Character {
                id: item.location_id.clone(),
            };
            let chain = vec![own, location.clone()];
            let game = with_note(
                GameSide {
                    reading: GamePrice::None,
                    source: None,
                    note: None,
                    tab: None,
                    tab_name: None,
                    substash_name: None,
                    public: None,
                    residue: false,
                },
                item.note.as_deref(),
                table,
            );
            (location, chain, game)
        } else {
            match tab_by_id.get(item.location_id.as_str()) {
                Some(info) => {
                    let mut chain = Vec::with_capacity(info.chain.len() + 1);
                    chain.push(own);
                    chain.extend(info.chain.iter().cloned());
                    let game = with_note(tab_game_side(info, table), item.note.as_deref(), table);
                    (info.target.clone(), chain, game)
                }
                None => {
                    // The snapshot's items are at its tabs by construction;
                    // an item that is not is reported, never dropped.
                    let location = PriceTarget::Tab {
                        realm,
                        id: item.location_id.clone(),
                    };
                    let chain = vec![own, location.clone()];
                    let game = with_note(
                        GameSide {
                            reading: GamePrice::None,
                            source: None,
                            note: None,
                            tab: None,
                            tab_name: None,
                            substash_name: None,
                            public: None,
                            residue: false,
                        },
                        item.note.as_deref(),
                        table,
                    );
                    (location, chain, game)
                }
            }
        };
        let league_unknown = item.location_kind == "character"
            && character_by_id
                .get(item.location_id.as_str())
                .is_some_and(|c| c.league.is_none());
        let subject = item_subject(item, location, league_unknown);
        let basis = Basis {
            response: item.seen_response,
            at: Some(item.last_seen),
        };
        let l = listing(subject, &chain, &rows, game, basis);
        counts.items += 1;
        if league_unknown {
            counts.league_unknown += 1;
        }
        *counts
            .by_relation
            .entry(l.relation.to_string())
            .or_default() += 1;
        *counts
            .by_game_statement
            .entry(l.game.reading.kind().to_string())
            .or_default() += 1;
        if l.game
            .note
            .as_ref()
            .is_some_and(|n| matches!(n.reading, GamePrice::Invalid { .. }))
        {
            counts.invalid_notes += 1;
        }
        if l.game.reading.price().is_some() {
            counts.game_priced += 1;
        }
        if l.manual.as_ref().is_some_and(|m| m.inherited) {
            counts.inherited += 1;
        }
        *counts
            .by_effective
            .entry(l.effective.side_word().to_string())
            .or_default() += 1;
        if l.game.residue {
            counts.residue += 1;
        }
        listings.push(l);
    }
    for relation in Relation::ALL {
        counts.by_relation.entry(relation.to_string()).or_default();
    }
    for side in ["game", "manual", "none", "unresolved"] {
        counts.by_effective.entry(side.into()).or_default();
    }

    let mut accounting = rows.accounting;
    let subjects: std::collections::BTreeSet<&PriceTarget> =
        listings.iter().map(|l| &l.subject.target).collect();
    for target in rows.by_target.keys() {
        if subjects.contains(target) {
            accounting.applied += 1;
        } else {
            accounting.unmatched.push(target.clone());
        }
    }

    Ok(ListingReport {
        header: ReportHeader {
            schema: LISTING_SCHEMA,
            account_uuid: snapshot.account_uuid.clone(),
            account_name: snapshot.account_name.clone(),
            realm,
            league: snapshot.league.clone(),
            taken_at: snapshot.taken_at,
            stash_listing: snapshot.stash_listing,
            character_listing: snapshot.character_listing,
            note_parser_version: NOTE_PARSER_VERSION,
            currency_table_version: CURRENCY_TABLE_VERSION,
        },
        counts,
        rows: accounting,
        listings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn tab(
        id: &str,
        parent: Option<&str>,
        name: &str,
        r#type: &str,
        metadata: Value,
    ) -> TabSnapshot {
        TabSnapshot {
            id: id.into(),
            parent: parent.map(str::to_string),
            name: name.into(),
            r#type: r#type.into(),
            idx: None,
            listed_at: Some(100),
            listed_response: Some(2),
            fetched_at: Some(110),
            metadata,
            item_count: 0,
        }
    }

    fn item(id: &str, kind: &str, location: &str, note: Option<&str>) -> ItemSnapshot {
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
            type_line: "Imperial Bow".into(),
            stack_size: None,
            x: Some(0),
            y: Some(0),
            note: note.map(str::to_string),
            seen_response: Some(7),
            last_seen: 120,
        }
    }

    fn row(scope: &str, key: &str, value: Value, revision: i64) -> AnnotationRow {
        AnnotationRow {
            scope: scope.into(),
            key: key.into(),
            kind: "buyout".into(),
            value,
            revision,
            created_at: 50,
            updated_at: 60,
            written_via: "test".into(),
            actor: None,
        }
    }

    fn exact(amount: &str, currency: &str) -> Value {
        json!({ "version": 1, "type": "exact", "amount": amount, "currency": currency })
    }

    /// The stash in miniature: a folder named like a price holding a
    /// public priced tab, a plain tab, a public priced map tab with a
    /// substash, an unpriced unique tab with a substash, one character.
    fn snapshot() -> PricingSnapshot {
        let public = json!({ "public": true });
        PricingSnapshot {
            account_uuid: "u-1".into(),
            account_name: Some("tom".into()),
            realm: "pc".into(),
            league: "Standard".into(),
            taken_at: 200,
            stash_listing: Some(ListingBasis {
                response_id: 2,
                fetched_at: 100,
            }),
            tabs: vec![
                tab(
                    "f1",
                    None,
                    "~price 9 chaos",
                    "Folder",
                    json!({ "folder": true }),
                ),
                tab(
                    "c1",
                    Some("f1"),
                    "~price 3 chaos (A)",
                    "PremiumStash",
                    public.clone(),
                ),
                tab(
                    "t1",
                    None,
                    "Dump",
                    "PremiumStash",
                    json!({ "colour": "ff0000" }),
                ),
                tab(
                    "m1",
                    None,
                    "~price 1 divine (Remove-only)",
                    "MapStash",
                    public.clone(),
                ),
                tab(
                    "s1",
                    Some("m1"),
                    "1 (Remove-only)",
                    "MapStash",
                    json!({ "items": 5, "map": {} }),
                ),
                tab("u1", None, "Uniques", "UniqueStash", Value::Null),
                tab(
                    "s2",
                    Some("u1"),
                    "Belts",
                    "UniqueStash",
                    json!({ "items": 2 }),
                ),
            ],
            character_listing: Some(ListingBasis {
                response_id: 3,
                fetched_at: 101,
            }),
            characters: vec![CharacterSnapshot {
                id: "ch1".into(),
                name: "Exile".into(),
                league: Some("Standard".into()),
                listed_at: Some(101),
                listed_response: Some(3),
                fetched_at: Some(111),
                listed: json!({ "id": "ch1", "name": "Exile", "league": "Standard" }),
                fetched: json!({ "id": "ch1", "name": "Exile" }),
            }],
            items: vec![
                item("i-worn", "character", "ch1", Some("~b/o 2 divine")),
                item("i-exact", "stash", "c1", Some("~price 5 chaos")),
                item("i-plain", "stash", "c1", None),
                item("i-invalid", "stash", "c1", Some("~price  chaos")),
                item("i-fifty", "stash", "c1", Some("50")),
                item("i-skip", "stash", "c1", Some("~skip ")),
                item("i-dump", "stash", "t1", None),
                item("i-sub", "stash", "s1", None),
                item("i-sub2", "stash", "s2", None),
            ],
            buyouts: vec![
                row("item", "i-plain", exact("3", "chaos"), 1),
                row(
                    "item",
                    "i-exact",
                    json!({ "version": 1, "type": "skip" }),
                    2,
                ),
                row("tab", "pc/f1", json!({ "version": 1, "type": "skip" }), 3),
                row("tab", "pc/m1", exact("1", "divine"), 4),
                row(
                    "substash",
                    "pc/u1/s2",
                    json!({ "version": 1, "type": "negotiable", "amount": "7", "currency": "chaos" }),
                    5,
                ),
                row("character", "ch1", exact("2222", "jewellers"), 6),
                row("tab", "xbox/c1", exact("1", "chaos"), 7),
                row("item", "i-elsewhere", exact("1", "chaos"), 8),
                row("tab", "pc", exact("1", "chaos"), 9),
                row(
                    "item",
                    "i-dump",
                    json!({ "version": 2, "type": "exact" }),
                    10,
                ),
            ],
        }
    }

    fn item_target(id: &str) -> PriceTarget {
        PriceTarget::Item { id: id.into() }
    }

    fn tab_target(id: &str) -> PriceTarget {
        PriceTarget::Tab {
            realm: Realm::Pc,
            id: id.into(),
        }
    }

    fn get<'a>(report: &'a ListingReport, target: &PriceTarget) -> &'a Listing {
        report
            .find(target)
            .unwrap_or_else(|| panic!("no listing for {target}"))
    }

    /// C69 — the game side reads the note, then the tab name, as a price
    /// or `skip`; an invalid note has no effect and the tab applies (T18);
    /// the raw texts ride beside every reading; a note where the index
    /// cannot see it is residue; the relation names both sides; the
    /// versions are stamped.
    #[test]
    fn c69_two_sides_resolve_independently_and_the_relation_names_both() {
        let report = resolve(&snapshot()).unwrap();
        assert_eq!(report.header.note_parser_version, NOTE_PARSER_VERSION);
        assert_eq!(report.header.currency_table_version, CURRENCY_TABLE_VERSION);
        assert_eq!(report.header.stash_listing.map(|b| b.response_id), Some(2));

        // A note in front of a priced, public tab: the note applies, and
        // beats the item's own row on a tie (C81).
        let l = get(&report, &item_target("i-exact"));
        assert_eq!(l.game.source, Some(Source::Note));
        assert_eq!(l.game.reading.to_string(), "5 chaos");
        assert_eq!(l.game.note.as_ref().unwrap().text, "~price 5 chaos");
        assert_eq!(l.game.tab_name.as_ref().unwrap().text, "~price 3 chaos (A)");
        assert_eq!(
            l.game.tab_name.as_ref().unwrap().reading.to_string(),
            "3 chaos"
        );
        assert_eq!(l.game.public, Some(true));
        assert_eq!(l.manual.as_ref().unwrap().value, Buyout::Skip);
        assert_eq!(l.relation, Relation::Conflict);
        assert_eq!(l.why, "by hand: skip; in game: 5 chaos");
        assert_eq!(
            l.basis,
            Basis {
                response: Some(7),
                at: Some(120)
            }
        );

        // No note: the tab name applies, and it agrees with the own row.
        let l = get(&report, &item_target("i-plain"));
        assert_eq!(l.game.source, Some(Source::TabName));
        assert_eq!(l.game.reading.to_string(), "3 chaos");
        assert_eq!(l.game.note, None);
        assert_eq!(l.relation, Relation::Agree);
        assert_eq!(l.why, "by hand and in game: 3 chaos");
        assert_eq!(l.manual.as_ref().unwrap().revision, 1);

        // The dialog's residue reads `invalid`, has no effect, and the tab
        // applies — as the site does (T18); the note is still shown.
        let l = get(&report, &item_target("i-invalid"));
        assert!(
            matches!(
                l.game.note.as_ref().unwrap().reading,
                GamePrice::Invalid { .. }
            ),
            "{:?}",
            l.game.note
        );
        assert_eq!(l.game.note.as_ref().unwrap().text, "~price  chaos");
        assert_eq!(l.game.source, Some(Source::TabName));
        assert_eq!(l.game.reading.to_string(), "3 chaos");
        assert_eq!(l.relation, Relation::Conflict); // beside the folder's skip
        assert_eq!(l.why, "by hand: skip; in game: 3 chaos");

        // A note that is not a price note: the tab name applies.
        let l = get(&report, &item_target("i-fifty"));
        assert_eq!(l.game.note.as_ref().unwrap().reading, GamePrice::None);
        assert_eq!(l.game.source, Some(Source::TabName));
        assert_eq!(l.game.reading.to_string(), "3 chaos");

        // `~skip` beside an inherited `skip`: both leave it out.
        let l = get(&report, &item_target("i-skip"));
        assert_eq!(l.game.reading, GamePrice::Skip);
        assert_eq!(l.relation, Relation::Agree);

        // A character item: the note is read and shown, but a character
        // is not a public tab, so it is residue, not a statement (C81).
        let l = get(&report, &item_target("i-worn"));
        assert_eq!(
            l.game.note.as_ref().unwrap().reading.to_string(),
            "2 divine b/o"
        );
        assert_eq!(l.game.reading, GamePrice::None);
        assert!(l.game.residue);
        assert_eq!(l.game.tab, None);
        assert_eq!(l.game.public, None);
        assert_eq!(l.relation, Relation::ManualOnly);
        assert_eq!(l.why, "by hand: 2222 jewellers; the game states nothing");

        // An unreadable own row and nothing in game: the relation is
        // `none`, and the sentence names the row, not "no row applies".
        let l = get(&report, &item_target("i-dump"));
        assert_eq!(l.relation, Relation::None);
        assert_eq!(
            l.why,
            "the row on item/i-dump cannot be read and the game states nothing"
        );
        assert_eq!(l.game.public, Some(false));
        assert!(!l.game.residue);

        // The counts are the items' — containers are not items.
        assert_eq!(report.counts.items, 9);
        assert_eq!(report.counts.containers, 8);
        assert_eq!(report.counts.by_relation["conflict"], 3);
        assert_eq!(report.counts.by_relation["agree"], 3);
        assert_eq!(report.counts.by_relation["manual_only"], 2);
        assert_eq!(report.counts.by_relation["game_only"], 0);
        assert_eq!(report.counts.by_relation["none"], 1);
        assert_eq!(report.counts.invalid_notes, 1);
        assert_eq!(report.counts.by_game_statement["exact"], 5);
        assert_eq!(report.counts.by_game_statement["skip"], 1);
        assert_eq!(report.counts.by_game_statement["none"], 3);
        assert_eq!(report.counts.game_priced, 5);
        assert_eq!(report.counts.residue, 1);
    }

    /// C81 — the effective price is the more specific statement, the
    /// game's on a tie; only a public tab's game side is a statement.
    #[test]
    fn c81_the_effective_price_is_the_most_specific_statement_and_the_games_on_a_tie() {
        let report = resolve(&snapshot()).unwrap();
        let eff = |t: PriceTarget| get(&report, &t).effective.clone();
        // A game note beats the item's own row (a tie at item level).
        let e = eff(item_target("i-exact"));
        assert_eq!(
            (e.side, e.to_string()),
            (Some(Side::Game), "5 chaos".into())
        );
        assert_eq!(e.from, Some(item_target("i-exact")));
        assert!(
            e.why
                .ends_with("the game wins a tie with the row on the item (C81)"),
            "{}",
            e.why
        );
        // An item row beats a game tab price.
        let e = eff(item_target("i-plain"));
        assert_eq!((e.side, e.kind.as_str()), (Some(Side::Manual), "exact"));
        assert!(
            e.why
                .contains("the row on the item is more specific than the game's tab name"),
            "{}",
            e.why
        );
        // A game tab price beats a folder row (an invalid note has no effect).
        let e = eff(item_target("i-invalid"));
        assert_eq!(
            (e.side, e.to_string()),
            (Some(Side::Game), "3 chaos".into())
        );
        assert_eq!(e.from, Some(tab_target("c1")));
        assert!(
            e.why
                .contains("the tab name is more specific than the row on the tab"),
            "{}",
            e.why
        );
        // Game `skip` beside the folder's `skip`: the game's, by level.
        let e = eff(item_target("i-skip"));
        assert_eq!((e.side, e.kind.as_str()), (Some(Side::Game), "skip"));
        // A substash item under a public priced parent: the parent's name
        // and the parent's row tie at tab level; the game wins.
        let e = eff(item_target("i-sub"));
        assert_eq!(
            (e.side, e.to_string()),
            (Some(Side::Game), "1 divine".into())
        );
        assert_eq!(e.from, Some(tab_target("m1")));
        // A substash row is more specific than the parent's name: the
        // substash's own listing takes its row, its item inherits it.
        let s2 = PriceTarget::Substash {
            realm: Realm::Pc,
            parent: "u1".into(),
            id: "s2".into(),
        };
        let e = eff(item_target("i-sub2"));
        assert_eq!(
            (e.side, e.to_string()),
            (Some(Side::Manual), "7 chaos b/o".into())
        );
        assert_eq!(e.from, Some(s2));
        // A character's items: the note is residue, the row applies, and
        // the sentence says why the game's text does not.
        let e = eff(item_target("i-worn"));
        assert_eq!(
            (e.side, e.to_string()),
            (Some(Side::Manual), "2222 jewellers".into())
        );
        assert!(
            e.why.contains("not where the index can see it (C81)"),
            "{}",
            e.why
        );
        // An unreadable own row and no game statement: unresolved, naming
        // the row — never "nothing applies".
        let e = eff(item_target("i-dump"));
        assert_eq!((e.side, e.kind.as_str()), (None, "unresolved"));
        assert_eq!(e.from, Some(item_target("i-dump")));
        assert!(
            e.why
                .starts_with("unresolved: the row on item/i-dump cannot be read"),
            "{}",
            e.why
        );
        // Containers resolve the same way: the tab's own name beats the
        // folder's row; the folder itself has only its row.
        let e = eff(tab_target("c1"));
        assert_eq!(
            (e.side, e.to_string()),
            (Some(Side::Game), "3 chaos".into())
        );
        let e = eff(tab_target("f1"));
        assert_eq!((e.side, e.kind.as_str()), (Some(Side::Manual), "skip"));
        assert_eq!(report.counts.by_effective["game"], 5);
        assert_eq!(report.counts.by_effective["manual"], 3);
        assert_eq!(report.counts.by_effective["none"], 0);
        assert_eq!(report.counts.by_effective["unresolved"], 1);
        // A public tab's price text is a statement; the same text on a
        // non-public tab is residue: the manual row applies alone.
        let mut snap = snapshot();
        snap.tabs[1].metadata = Value::Null; // c1 no longer public
        let report = resolve(&snap).unwrap();
        let l = get(&report, &item_target("i-exact"));
        assert_eq!(l.game.reading, GamePrice::None);
        assert!(l.game.residue);
        assert_eq!(l.game.note.as_ref().unwrap().reading.to_string(), "5 chaos");
        assert_eq!(l.relation, Relation::ManualOnly);
        assert_eq!(l.effective.side, Some(Side::Manual));
        assert_eq!(l.effective.kind, "skip");
        // An unreadable row that could beat the game's statement leaves
        // the price unresolved; one the game beats by level does not.
        let unreadable =
            |scope: &str, key: &str| row(scope, key, json!({ "version": 9, "type": "exact" }), 20);
        let mut snap = snapshot();
        snap.buyouts.retain(|r| r.key != "i-plain");
        snap.buyouts.push(unreadable("item", "i-plain")); // above the public tab price
        snap.buyouts.push(unreadable("substash", "pc/m1/s1")); // above the parent's price
        let report = resolve(&snap).unwrap();
        for id in ["i-plain", "i-sub"] {
            let l = get(&report, &item_target(id));
            assert!(l.game.reading.price().is_some(), "{id}");
            assert_eq!(l.effective.kind, "unresolved", "{id}: {}", l.effective.why);
            assert_eq!(l.effective.side, None);
            assert!(l.manual_problem.is_some(), "{id}");
        }
        assert_eq!(
            get(&report, &item_target("i-plain")).effective.from,
            Some(item_target("i-plain"))
        );
        let mut snap = snapshot();
        snap.buyouts.retain(|r| r.key != "pc/f1");
        snap.buyouts.push(unreadable("tab", "pc/f1")); // below the tab's price
        let report = resolve(&snap).unwrap();
        let l = get(&report, &item_target("i-fifty"));
        assert_eq!(l.effective.side, Some(Side::Game));
        assert!(
            l.effective.why.contains("readable or not"),
            "{}",
            l.effective.why
        );
        assert!(l.manual_problem.is_some());
        // An item's show view carries no item sets.
        let view = report.show_view(&item_target("i-fifty")).unwrap();
        assert!(view.items_here.is_empty() && view.items_covered_below.is_empty());
        // The relation sentence names the unreadable row beside a game
        // statement too, and the default list never hides an unresolved
        // item behind its relation word; `effective` selects by side.
        assert_eq!(
            l.why,
            "in game: 3 chaos; the row on tab/pc/f1 cannot be read"
        );
        let report = resolve(&snapshot()).unwrap();
        let default = report.list_view(ListFilter::default()).unwrap();
        assert!(
            default
                .items
                .iter()
                .any(|l| l.subject.target == item_target("i-dump"))
        );
        let unresolved = report
            .list_view(ListFilter {
                effective: Some("unresolved".into()),
                ..ListFilter::default()
            })
            .unwrap();
        assert_eq!(
            unresolved
                .items
                .iter()
                .map(|l| l.subject.target.to_string())
                .collect::<Vec<_>>(),
            ["item/i-dump"]
        );
        // Every effective side selects on its own, `none` included: the
        // default suppression of relation `none` does not run under a
        // selector, or `--effective none` could never find anything.
        let mut snap = snapshot();
        snap.buyouts.retain(|r| r.key != "i-dump"); // i-dump: nothing applies
        let report = resolve(&snap).unwrap();
        let by_side = |side: &str| -> Vec<String> {
            report
                .list_view(ListFilter {
                    effective: Some(side.into()),
                    ..ListFilter::default()
                })
                .unwrap()
                .items
                .iter()
                .map(|l| l.subject.target.to_string())
                .collect()
        };
        assert_eq!(by_side("game").len(), 5);
        assert_eq!(by_side("manual").len(), 3);
        assert_eq!(by_side("none"), ["item/i-dump"]);
        assert_eq!(by_side("unresolved"), Vec::<String>::new());
        assert_eq!(
            by_side("game").len() + by_side("manual").len() + by_side("none").len(),
            report.counts.items
        );
    }

    /// C53 — the JSON shape is the contract: the resolved fixture must
    /// serialize to exactly the committed document. A difference is
    /// either additive (regenerate with `ACQ_UPDATE_FIXTURES=1`) or a
    /// schema bump (`LISTING_SCHEMA`), decided by reading the diff.
    #[test]
    fn the_report_json_matches_the_committed_fixture() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/reference/listing-report-schema-2.json"
        );
        let report = resolve(&snapshot()).unwrap();
        let actual = serde_json::to_value(&report).unwrap();
        if std::env::var_os("ACQ_UPDATE_FIXTURES").is_some() {
            std::fs::write(path, serde_json::to_string_pretty(&actual).unwrap() + "\n").unwrap();
        }
        let expected: Value = serde_json::from_str(
            &std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}")),
        )
        .unwrap();
        assert_eq!(expected["schema"], json!(LISTING_SCHEMA));
        assert_eq!(
            actual, expected,
            "the report's JSON changed; additive → ACQ_UPDATE_FIXTURES=1, else bump LISTING_SCHEMA"
        );
        let back: ListingReport = serde_json::from_value(expected).unwrap();
        assert_eq!(back, report);
    }

    /// C70 — the manual side by specificity: the own row, else the
    /// substash's, else the tab's, else the folder's; a character row
    /// covers its items; coverage is a row's existence, never its type.
    #[test]
    fn c70_the_manual_side_resolves_by_specificity() {
        let report = resolve(&snapshot()).unwrap();
        let from = |id: &str| {
            let l = get(&report, &item_target(id));
            let m = l
                .manual
                .as_ref()
                .unwrap_or_else(|| panic!("{id} has no manual side"));
            (m.from.clone(), m.inherited, m.value.to_string())
        };
        assert_eq!(
            from("i-plain"),
            (item_target("i-plain"), false, "3 chaos".into())
        );
        assert_eq!(from("i-fifty"), (tab_target("f1"), true, "skip".into()));
        assert_eq!(from("i-sub"), (tab_target("m1"), true, "1 divine".into()));
        assert_eq!(
            from("i-sub2"),
            (
                PriceTarget::Substash {
                    realm: Realm::Pc,
                    parent: "u1".into(),
                    id: "s2".into()
                },
                true,
                "7 chaos b/o".into()
            )
        );
        assert_eq!(
            from("i-worn"),
            (
                PriceTarget::Character { id: "ch1".into() },
                true,
                "2222 jewellers".into()
            )
        );
        assert_eq!(
            get(&report, &item_target("i-sub")).relation,
            Relation::Agree
        );
        assert_eq!(
            get(&report, &item_target("i-sub2")).relation,
            Relation::ManualOnly
        );
        // A container's own listing walks the same chain: the substash
        // under the priced map tab inherits it; the folder child inherits
        // the folder's row.
        let s1 = PriceTarget::Substash {
            realm: Realm::Pc,
            parent: "m1".into(),
            id: "s1".into(),
        };
        assert_eq!(
            get(&report, &s1).manual.as_ref().unwrap().from,
            tab_target("m1")
        );
        assert_eq!(
            get(&report, &tab_target("c1"))
                .manual
                .as_ref()
                .unwrap()
                .from,
            tab_target("f1")
        );
        assert_eq!(report.counts.inherited, 6);
        // An unreadable row that would apply stops the walk: the item
        // under it has no manual side and says why, and does not fall
        // through to the tab.
        let l = get(&report, &item_target("i-dump"));
        assert_eq!(l.manual, None);
        assert!(
            l.manual_problem.as_deref().unwrap().contains("item/i-dump"),
            "{:?}",
            l.manual_problem
        );
    }

    /// C80 — a substash reads its parent's name and `public`, its own
    /// name beside; a folder child reads itself; a folder's name is never
    /// a price, for the folder or through it.
    #[test]
    fn c80_a_substash_reads_its_parent_a_folder_child_itself_and_a_folder_never() {
        let report = resolve(&snapshot()).unwrap();
        let l = get(&report, &item_target("i-sub"));
        assert_eq!(l.game.tab.as_deref(), Some("m1"));
        assert_eq!(
            l.game.tab_name.as_ref().unwrap().text,
            "~price 1 divine (Remove-only)"
        );
        assert_eq!(l.game.reading.to_string(), "1 divine");
        assert_eq!(l.game.substash_name.as_deref(), Some("1 (Remove-only)"));
        assert_eq!(l.game.public, Some(true));
        let l = get(&report, &item_target("i-sub2"));
        assert_eq!(l.game.tab.as_deref(), Some("u1"));
        assert_eq!(l.game.reading, GamePrice::None);
        assert_eq!(l.game.substash_name.as_deref(), Some("Belts"));
        assert_eq!(l.game.public, Some(false));
        // The folder child reads its own name, not the folder's.
        let l = get(&report, &item_target("i-plain"));
        assert_eq!(l.game.tab.as_deref(), Some("c1"));
        assert_eq!(l.game.reading.to_string(), "3 chaos");
        // The folder itself: named like a price, read as nothing.
        let l = get(&report, &tab_target("f1"));
        assert_eq!(l.subject.tab_type.as_deref(), Some("Folder"));
        assert_eq!(l.game.reading, GamePrice::None);
        assert_eq!(l.game.tab_name, None);
        assert_eq!(l.game.public, None);
        assert_eq!(l.relation, Relation::ManualOnly);
        // Priced tabs count the tabs a user can name: not folders, not substashes.
        assert_eq!(report.counts.priced_tabs, 2);
        assert_eq!(report.counts.priced_tabs_public, 2);
        // A substash whose parent is not on record reads nothing and says so.
        let mut snap = snapshot();
        snap.tabs
            .push(tab("s9", Some("gone"), "9", "MapStash", Value::Null));
        snap.items.push(item("i-orphan", "stash", "s9", None));
        let report = resolve(&snap).unwrap();
        let l = get(&report, &item_target("i-orphan"));
        assert_eq!(l.game.tab, None);
        assert_eq!(l.game.substash_name.as_deref(), Some("9"));
        assert_eq!(l.game.public, None);
    }

    /// C53 — every nothing says which nothing: rows of another realm are
    /// counted, readable rows naming nothing in these facts are listed,
    /// unreadable rows are listed with why; and the report is its own
    /// contract — it re-reads exactly.
    #[test]
    fn rows_are_accounted_for_and_the_report_round_trips() {
        let report = resolve(&snapshot()).unwrap();
        assert_eq!(report.rows.total, 10);
        assert_eq!(report.rows.applied, 6);
        assert_eq!(report.rows.other_realm, 1);
        assert_eq!(report.rows.unmatched, [item_target("i-elsewhere")]);
        let unreadable: Vec<(&str, &str)> = report
            .rows
            .unreadable
            .iter()
            .map(|p| (p.key.as_str(), p.why.as_str()))
            .collect();
        assert_eq!(unreadable.len(), 2, "{unreadable:?}");
        assert_eq!(unreadable[0].0, "pc");
        assert!(
            unreadable[0].1.contains("is not a tab key"),
            "{}",
            unreadable[0].1
        );
        assert_eq!(unreadable[1].0, "i-dump");
        assert!(
            unreadable[1].1.contains("a newer build wrote it"),
            "{}",
            unreadable[1].1
        );

        let text = serde_json::to_string(&report).unwrap();
        let back: ListingReport = serde_json::from_str(&text).unwrap();
        assert_eq!(back, report);
        assert_eq!(report.summary().listings.len(), 0);
        assert_eq!(report.summary().counts, report.counts);
        assert_eq!(report.items_in(&tab_target("c1")).count(), 5);
        // A realm word the snapshot carries that Realm does not know is
        // an error, never a report with every tab row set aside.
        let mut snap = snapshot();
        snap.realm = "ps5".into();
        assert!(matches!(
            resolve(&snap),
            Err(ListingError::UnknownRealm { .. })
        ));
    }

    /// C70 — every listing carries its chain, so a target's coverage is
    /// a query: a folder covers its tabs' items without holding any, a
    /// parent tab covers its substashes' items beside its own, an
    /// ordinary tab's two sets are one.
    #[test]
    fn c70_a_targets_coverage_is_every_listing_whose_chain_holds_it() {
        let report = resolve(&snapshot()).unwrap();
        let ids = |it: Box<dyn Iterator<Item = &Listing> + '_>| -> Vec<String> {
            it.map(|l| l.subject.target.to_string()).collect()
        };
        let f1 = tab_target("f1");
        assert_eq!(ids(Box::new(report.items_in(&f1))), Vec::<String>::new());
        assert_eq!(
            ids(Box::new(report.items_covered_by(&f1))),
            [
                "item/i-exact",
                "item/i-plain",
                "item/i-invalid",
                "item/i-fifty",
                "item/i-skip"
            ]
        );
        let m1 = tab_target("m1");
        assert_eq!(ids(Box::new(report.items_in(&m1))), Vec::<String>::new());
        assert_eq!(ids(Box::new(report.items_covered_by(&m1))), ["item/i-sub"]);
        let c1 = tab_target("c1");
        assert_eq!(
            ids(Box::new(report.items_in(&c1))),
            ids(Box::new(report.items_covered_by(&c1)))
        );
        let l = get(&report, &item_target("i-sub"));
        assert_eq!(
            l.chain.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["item/i-sub", "substash/pc/m1/s1", "tab/pc/m1"]
        );
        let l = get(&report, &item_target("i-worn"));
        assert_eq!(
            l.chain.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["item/i-worn", "character/ch1"]
        );
        // The views carry both sets and refuse a container not on record.
        let view = report.show_view(&f1).unwrap();
        assert!(view.items_here.is_empty());
        assert_eq!(view.items_covered_below.len(), 5);
        assert_eq!(view.container, None);
        let view = report.show_view(&item_target("i-sub")).unwrap();
        assert_eq!(
            view.container.as_ref().map(|c| c.subject.name.as_str()),
            Some("1 (Remove-only)")
        );
        let view = report
            .list_view(ListFilter {
                covered_by: Some(m1.clone()),
                ..ListFilter::default()
            })
            .unwrap();
        assert_eq!(ids(Box::new(view.items.iter())), ["item/i-sub"]);
        assert_eq!(
            ids(Box::new(view.containers.iter())),
            ["tab/pc/m1", "substash/pc/m1/s1"]
        );
        assert_eq!(view.items_on_record, 9);
        assert_eq!(
            report.list_view(ListFilter {
                r#in: Some(tab_target("nope")),
                ..ListFilter::default()
            }),
            None
        );
        // A view is its own contract: it re-reads exactly.
        let text = serde_json::to_string(&view).unwrap();
        assert_eq!(serde_json::from_str::<ListView>(&text).unwrap(), view);
        let show = report.show_view(&m1).unwrap();
        let text = serde_json::to_string(&show).unwrap();
        assert_eq!(serde_json::from_str::<ShowView>(&text).unwrap(), show);
        assert_eq!(
            text.matches(&format!("\"schema\":{LISTING_SCHEMA}"))
                .count(),
            1,
            "{text}"
        );
    }

    /// A character the listing gave no league is carried by every
    /// league's snapshot (the planner's rule); its items are flagged and
    /// counted here, never silently attributed.
    #[test]
    fn a_league_less_characters_items_are_flagged_in_every_leagues_report() {
        let mut snap = snapshot();
        snap.characters.push(CharacterSnapshot {
            id: "ch-none".into(),
            name: "Drifter".into(),
            league: None,
            listed_at: Some(101),
            listed_response: Some(3),
            fetched_at: Some(111),
            listed: json!({ "id": "ch-none", "name": "Drifter" }),
            fetched: Value::Null,
        });
        snap.items.push(item(
            "i-drift",
            "character",
            "ch-none",
            Some("~price 1 chaos"),
        ));
        for league in ["Standard", "Hardcore"] {
            snap.league = league.into();
            let report = resolve(&snap).unwrap();
            let l = get(&report, &item_target("i-drift"));
            assert!(l.subject.league_unknown, "{league}");
            assert!(
                get(
                    &report,
                    &PriceTarget::Character {
                        id: "ch-none".into()
                    }
                )
                .subject
                .league_unknown
            );
            assert!(!get(&report, &item_target("i-worn")).subject.league_unknown);
            assert_eq!(report.counts.league_unknown, 1);
            assert_eq!(report.counts.items, 10);
        }
        // The flag is absent from JSON when false, present when true.
        let report = resolve(&snap).unwrap();
        let text = serde_json::to_string(&report).unwrap();
        assert_eq!(text.matches("\"league_unknown\":true").count(), 2, "{text}");
        assert!(!text.contains("\"league_unknown\":false"));
    }
}
