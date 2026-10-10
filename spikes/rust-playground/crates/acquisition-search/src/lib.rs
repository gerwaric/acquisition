//! `acquisition-search` — item search over the store's read API (C89).
//!
//! Rulings: `decisions/search.md`, C89–C107. The language on one page, an
//! example per construct, and the contract detail that decides answers:
//! `search/DESIGN.md`, binding until a module doc here takes a paragraph;
//! the build's order: `search/BUILD-PLAN.md`; what each step met — the
//! shapes of fault its reviews taught, the measurements' verdicts:
//! `SEARCH-SLICE.md`; its steps and the holes ruled: `search/LEDGER.md`; the
//! measurements' tables: `search/MEASUREMENTS.md`.
//!
//! # As built
//!
//! The language (the build plan, step 1), the derivation (step 2), the
//! first surface (step 4), the counts view (step 5), the class table
//! (step 6), the computed values (step 7), the sockets (step 8), the
//! categories (step 9d-B), the base defence percentile (step 9d-A) and
//! the why-not and the continuation (step 10: `--explain`, `--context
//! corpus`, `show --against`; a `next`, a route read back): a request
//! in, an answer out.
//!
//! - [`tree`] — the query tree (C91): one typed value, and [`check`], what
//!   a tree must satisfy to be one the language can say.
//! - [`mod@parse`] — text to tree, over the whole language of the reference;
//!   shorthand lowers at once, a slotless comparison is decided by the
//!   template's own numbers before it lowers (C92), and every ambiguity the
//!   grammar defines is an error that shows its readings (C91).
//! - [`mod@print`] — tree to canonical text. Invariant 1 of the surface:
//!   canonicalisation lowers shorthand and normalises spelling, and never
//!   reorders, flattens, merges, deduplicates or simplifies.
//! - [`json`] — the tree's JSON form, read strictly (C104: the text is
//!   carried on every seat; the tree is accepted and always returned).
//! - [`error`] — authoring errors with stable kinds (C47, C11), and what a
//!   search can fail with beside them: a refusal's way on, a request
//!   beside its reading.
//! - [`mod@derive`] — one item's body and ingest facts in, the item out
//!   (C103, C90): fields, displayed strings, lines as kind, template and
//!   numbers with their slots, and what could not be read, by collection.
//! - [`mod@bind`] — the names: fields, closed sets, what a line has; near
//!   names; and the one closed list of what is not built, refused by name.
//! - `group` — what `line( … )` means, computed once when the query is
//!   bound: the one module that reads a group's tree for its meaning (the
//!   build plan, step 4b; `tools/docs-check.sh` §7).
//! - [`class`] — the class table (C106): reviewed reference data shipped
//!   in the binary, the item's class read from its base, and what the
//!   search says when the table cannot class it (C93).
//! - [`category`] — the trade site's categories (C106): the grouping
//!   above class, each of the site's ids answered of an item from its
//!   class and reviewed rules in one place; a captured beast by its
//!   bestiary line and the beast colour table.
//! - [`totals`] — the totals table (C94): a named total as reviewed
//!   reference data, one meaning on every surface; `pseudo` — the
//!   computed values, `pseudo.<name>`: the totals and the derived fields
//!   (C101), asked as a sum is. [`defences`] — the defence table (C101,
//!   C106): each base's defence ranges and each text a local defence
//!   stat displays, which the base defence percentile reads.
//! - [`sockets`] — the socket counts (C101): `sockets`, `links`,
//!   `sockets.<colour>` and a link group's counts as intervals of what
//!   was read; `linked( … )` is bound in `group` beside a line's group.
//! - [`corpus`] — every live item of a scope, derived and classed from one
//!   snapshot of the store's read (C108) and held with its basis (C98).
//! - `eval` — a term asked of an item: matched, failed, lacked or
//!   undecided, witnesses, sums, the sort scalar, the together count
//!   (C92, C93).
//! - [`mod@answer`] — the request and the answer (C100, C96): scope, basis,
//!   every term's counts with a route each, the total, the rows, the zero
//!   block and its offers, a node explained, the corpus context; and the
//!   one maker of a route.
//! - [`counts`] — the counts view (C95, C105) and the vocabulary read
//!   (C97): the matches by a key, one table each or one crossed table, a
//!   sum beside each count, every bucket a term with its route.
//! - [`mod@describe`] — the language as this build knows it (C97), and the
//!   limits it states (C102). [`mod@show`] — one item as the deriver sees it.
//!
//! The parser knows the grammar and no field, class or computed value by
//! name: a name is the binder's to know, so validity never depends on a
//! corpus (invariant 3). What the evaluator does not yet build is refused
//! by name (the build plan, "How a partial build stays honest").
//!
//! # Decisions as recorded
//!
//! - **C89.** The crate links the store, and the planner once a query
//!   reads the effective price; never the daemon, the client, an HTTP
//!   client or an async runtime, and the daemon and the planner never link
//!   it. `tools/docs-check.sh` §5 refuses each edge, and
//!   `tools/docs-check-breakers.sh` proves it refuses.
//! - **C103, C90.** The deriver is [`mod@derive`]'s module doc; pinned by
//!   `tests/derive.rs`, and set against the census over a real corpus by
//!   `search/item-facts/scripts/m2-differential.py` (never in the gate).
//!   The read that hands the deriver its facts is the store's, C108
//!   (`acquisition-store/src/corpus.rs`). A mod displayed over several
//!   rows, named by its whole text or by a row, is the deriver's doc and
//!   `group`'s; pinned by `tests/rows.rs` (a line's term, its slots, what
//!   is unread of one row, the vocabulary, what a quoted template resolved
//!   to), `tests/generated_rows.rs` (a row named answers as the row
//!   alone) and `tests/pseudo.rs` (a total's row, on the trade site's
//!   captures).
//! - **C91, C92, C93, C96, C98, C100, C102.** The binder, evaluation, the
//!   held corpus and the answer are their modules' docs; pinned at the
//!   crate's boundary — a request in, an answer out, as JSON — by
//!   `tests/answer.rs` (the reference's worked example reduced to what is
//!   built, the route property, invariants 2 and 6), `tests/acceptance.rs`
//!   (the acceptance set's rows green at step 4, the limits' wording) and
//!   `tests/refusal.rs` (the refusal walk, invariant 3). The why-not —
//!   what a path names, the literal substitution, the corpus context,
//!   the zero block's offers, `show --against` — is [`tree`]'s,
//!   [`mod@answer`]'s and [`mod@show`]'s docs; pinned by
//!   `tests/explain.rs` (every cell of the piece's expected answers, a
//!   node under a not, an or and a `holds`, the refusals, the offers
//!   followed as requests), `tests/against.rs` (the seven items of the
//!   worked example), `tests/generated_explain.rs` (each forced answer
//!   is the transformed query's, asked alone, at every path, and a
//!   term's is the same in every spelling) and `tests/acceptance.rs`
//!   (AQ3 whole). The continuation and a route read back are
//!   [`mod@answer`]'s doc; pinned by `tests/next.rs` (the pages in order
//!   are the answer at a limit past the total, fixed and generated; the
//!   refusal across a changed basis, another build's and a refetched
//!   file's among them; both counts of a route read back, with its
//!   basis sent or without) and `tests/build_name.rs` (which files the
//!   build's name is a digest of). What a refusal offers is [`error`]'s
//!   doc; pinned by `tests/offers.rs` (each offer the request its
//!   reading is, and one that runs). Membership is [`corpus`]'s doc and the store's
//!   (`acquisition-store/src/corpus.rs`, C108); pinned by
//!   `tests/membership.rs` (the removed beside the live, each marked at
//!   the place it was last seen; a removed item's price; every route
//!   under `all`; AQ4 across a refresh that removed the item; the
//!   removed alone, the rows `all` marks).
//! - **C95, C97, C105.** The counts view and the vocabulary are
//!   [`counts`]'s doc; pinned by `tests/counts.rs`: AQ1, C105's two
//!   invariants on a one-value key, C95's sum over the three kinds, the
//!   vocabulary's pasted term selecting its row, every bucket's route
//!   followed by id.
//! - **C104.** Pinned by `tests/language.rs`: the round trip over
//!   generated trees and over the corpus (`tests/language.toml`), where
//!   every construct of the reference has a case.
//! - **C106.** The class table is [`class`]'s doc and
//!   `reference/classes-v1.toml`'s header; pinned by `tests/class.rs`
//!   (the shipped file, every reason the table gives, `undecided(class)`,
//!   the count's tally, `show`) and `tests/acceptance.rs` (OQ1 and OQ4 as
//!   worded, OQ5 askable). Category is [`category`]'s doc — the paragraph
//!   of the contract detail and the rulings J1–J4, J9–J22, taken there at
//!   step 9d-B — and the headers of `reference/categories-v1.toml` and
//!   `reference/beast-colours-v1.toml`; pinned by `tests/category.rs`
//!   (every row of the track's expected answers, a beast found by its
//!   category, a category under not and or, `show`, the help) and
//!   `tests/acceptance.rs` (OQ5 by category).
//! - **C94, C101.** The totals table is [`totals`]'s doc and
//!   `reference/totals-v6.toml`'s header — the ranged family and what no
//!   total reads (J29) among them, taken from the contract detail at
//!   step 9d-C; the computed values, their three
//!   statuses, the readings of other totals and the derived fields are
//!   `pseudo`'s doc; pinned by
//!   `tests/pseudo.rs` (the shipped table through the boundary, a total's
//!   three statuses, a count and a least on the trade site's captures and
//!   where a total they read is open, `dps` and `pdps`, the sort and the
//!   sum over one), `tests/ranged.rs` (every row of the ranged family's
//!   expected answers, the plan's closing row, J29 on the totals shipped
//!   before it and on round n's captures, a row that reads the category
//!   under not and or, the help),
//!   `tests/answer.rs` (the reference's worked example whole) and
//!   `tests/acceptance.rs` (AQ2 as worded). The base defence percentile
//!   — the paragraph of the contract detail and the rulings J5, J6, J8
//!   and J33–J35, taken at step 9d-A — is `pseudo`'s doc, its reference
//!   data [`defences`]'s and `reference/defences-v1.toml`'s header;
//!   pinned by `tests/percentile.rs` (every row of the track's expected
//!   answers, the worked cases asked every way a computed value is, the
//!   field under not and or, what does not open it, the refusal of
//!   `has:` with a slot word, the help). The sockets are [`sockets`]'s
//!   doc, the link group `group`'s; pinned by `tests/sockets.rs` (the
//!   counts two ways, S59, what each unread socket leaves open, the
//!   buckets, the sort, `show`, every authoring error), `tests/derive.rs`
//!   (the collection at the socket's grain) and `tests/acceptance.rs`
//!   (OQ3's socket colours).

// The lint ratchet (C47): malformed input — a text, a tree — is a
// structured error, never a panic. Tests may unwrap.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

pub mod answer;
pub mod bind;
pub mod category;
pub mod class;
pub mod corpus;
pub mod counts;
pub mod defences;
pub mod derive;
pub mod describe;
pub mod error;
mod eval;
mod exact;
mod group;
pub mod json;
pub mod parse;
pub mod price;
pub mod print;
mod pseudo;
pub mod show;
pub mod sockets;
mod template;
pub mod totals;
pub mod tree;

pub use answer::{Answer, Request, answer};
pub use bind::{NOT_BUILT, NotBuilt, Query, bind, not_built, parse_query};
pub use category::{CATEGORY_TABLE_VERSION, COLOUR_TABLE_VERSION, Category, CategoryTableError};
pub use class::{CLASS_TABLE_VERSION, ClassGap, ClassTable, ClassTableError, Classed};
pub use corpus::{Basis, Corpus, Realm};
pub use defences::{DEFENCE_TABLE_VERSION, DefenceTable, DefenceTableError, PercentileGap};
pub use derive::{
    Facts, Item, Line, Numbers, Part, Place, Property, Row, Shown, Slot, Socket, Unread, derive,
};
pub use describe::{Describe, describe};
pub use error::{ErrorKind, LanguageError, SearchError};
pub use json::{from_json, to_json};
pub use parse::{parse, parse_value};
pub use price::{PriceGap, PriceOf, Priced};
pub use print::{print, print_value};
pub use show::show;
pub use totals::{TOTALS_TABLE_VERSION, TotalGap, TotalsTable, TotalsTableError};
pub use tree::{Collection, Member, Node, Number, Op, Probe, Value, ValueRef, check};
