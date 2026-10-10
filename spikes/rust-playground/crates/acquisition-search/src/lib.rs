//! `acquisition-search` — item search over the store's read API (C89).
//!
//! Rulings: `decisions/search.md`, C89–C107. The language as this build
//! knows it, an example per construct, is `--describe` ([`mod@describe`]);
//! the rules that decide an answer are the module docs below, each under
//! its decision's id; what the build met — the plan's rules, cited here as
//! "rule N of the plan", the shapes of fault its reviews taught, the
//! measurements' verdicts: `SEARCH-SLICE.md`, the closed record (the
//! plan itself is `search/BUILD-PLAN.md` at `453a4429`); its steps and the holes ruled:
//! `search/LEDGER.md`; the measurements' tables: `search/MEASUREMENTS.md`.
//!
//! # The words
//!
//! Three words are fixed here. The **query** is the conditions and nothing
//! else: C91's tree and its text. The word is the trade site's own — its
//! request is `{query, sort}` posted to a realm and a league, the conditions
//! inside `query` and everything else outside it — and *filter* stays the
//! word for GGG's item filters and for the site's single controls. A
//! **request** is a query bound to a scope and a view. **`pseudo`** is this
//! search's namespace for computed values; the trade site's 298 are called
//! the site's pseudo stats, and neither promises the other (owner,
//! 2026-09-19: "This doesn't need to 100% align with trade").
//!
//! The owner's words on the query (2026-09-19, verbatim but for spelling)
//! are quoted in the module doc each bears on — `parse.rs`, `template.rs`,
//! `bind.rs`, `derive.rs`, `group.rs`, `pseudo.rs`, `tree.rs`; whole, with
//! the reasoning, in `search/search-forms/11-owner-amendments.md`.
//!
//! # Invariants of the surface
//!
//! Cited by number across the crate and its tests.
//!
//! 1. Canonicalisation lowers shorthand and normalises spelling; it never
//!    reorders, flattens, merges, deduplicates or simplifies.
//! 2. A `:` or `~` selector is never replaced by what it resolved to; the
//!    answer shows the binding beside the authored selector, and a
//!    vocabulary row's selector is always the exact form.
//! 3. A selector's validity never depends on the corpus.
//! 4. A count shown has a route to its members, and the route keeps its
//!    denominator.
//! 5. Every block of an answer is bounded: summaries and a route to the
//!    whole, never an enumeration by default.
//! 6. Search is a pure read, and no item predicate supports a claim about
//!    an unfetched location.
//! 7. Parentheses group and do nothing else: a rewrite that changes no
//!    meaning changes no match, no count, no error and no route — only the
//!    canonical text and the paths (invariant 1).
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
//! - [`mod@parse`] — text to tree, over the whole language;
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
//!   `tests/answer.rs` (the worked example reduced to what is
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
//!   every construct of the language has a case.
//! - **C106 — what reference data admits, and the test.** Three kinds:
//!   *data*, what the game files state and a registered surface exports;
//!   *convention*, a concept the community has published a definition for
//!   (the owner, 2026-09-18, on "equipment": "This is something that's
//!   well-defined by the community, e.g. RePoE, the trade site categories,
//!   poedb.tw, and <https://www.poewiki.net/wiki/Equipment>"); *judgment*,
//!   never embedded. C79 never required a surface to be GGG's: it requires
//!   registration, no runtime fetch, and a human's commit (C68). A
//!   convention behaves as a named total does (C94): defined once, printing
//!   its definition and source on request, naming one authority with the
//!   rest as corroboration and any disagreement noted on the row — order of
//!   authority: GGG-authored surfaces (the API, the trade site's own
//!   categories, the item-filter documentation), then game-file exports
//!   (RePoE, Path of Building), then community prose (the wiki, poedb). A
//!   word that is the trade site's says so. **The admission test:** (a) a
//!   definition at a registered surface: published there, or, where the
//!   surface publishes the word and not its definition, evidenced by dated
//!   observations of what the surface answers, each row citing its capture
//!   (the trade site's pseudo stats; owner, 2026-09-26, V6); (b) reviewable —
//!   generated by a committed script from a pinned source and reviewed as a
//!   diff at each pull (a "small enough to read every row" test would have
//!   refused the 476-row base-defence extract); (c) the word prints its
//!   definition and its source version, which the basis cites (C98); (d)
//!   every question can still be asked without it. A table enters one at a
//!   time — "may back a field", never "the app carries game data" — and a
//!   surface's row changes only when the first definition needs it, its
//!   licence read then (the wiki is CC BY-NC 3.0; poedb states no terms and
//!   is unregistered). "A roll outside the base's current range" is data;
//!   calling it legacy is history the export does not carry — legacy is
//!   never a field (C107); a unique's variant, a label its source prints,
//!   is the parked candidate (`decisions/search.md`, "Parked"). The three
//!   class readings the first seat ruled (V1, V2) are `class.rs`'s doc.
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
//!   `tests/answer.rs` (the worked example whole) and
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
