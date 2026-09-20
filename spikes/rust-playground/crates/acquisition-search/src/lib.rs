//! `acquisition-search` — item search over the store's read API (C89).
//!
//! Rulings: `decisions/search.md`, C89–C107. The language on one page, an
//! example per construct, and the contract detail that decides answers:
//! `search/DESIGN.md`, binding until a module doc here takes a paragraph;
//! the build's order and evidence: `search/BUILD-PLAN.md`.
//!
//! # As built
//!
//! The language (the build plan, step 1) — a query's text and its tree,
//! each printing to the other — and the derivation (step 2), the item a
//! query will be asked of. Nothing evaluates yet.
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
//! - [`error`] — authoring errors with stable kinds (C47, C11).
//! - [`mod@derive`] — one item's body and ingest facts in, the item out
//!   (C103, C90): fields, displayed strings, lines as kind, template and
//!   numbers with their slots, and what could not be read, by collection.
//!
//! The parser knows the grammar and no field, class or computed value by
//! name: a name is the binder's to know, so validity never depends on a
//! corpus (invariant 3). What the evaluator does not yet build it will
//! refuse by name (the build plan, "How a partial build stays honest").
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
//! - **C104.** Pinned by `tests/language.rs`: the round trip over
//!   generated trees and over the corpus (`tests/language.toml`), where
//!   every construct of the reference has a case.

// The lint ratchet (C47): malformed input — a text, a tree — is a
// structured error, never a panic. Tests may unwrap.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

pub mod derive;
pub mod error;
pub mod json;
pub mod parse;
pub mod print;
mod template;
pub mod tree;

pub use derive::{Facts, Item, Line, Part, Property, Shown, Unread, derive};
pub use error::{ErrorKind, LanguageError};
pub use json::{from_json, to_json};
pub use parse::{parse, parse_value};
pub use print::{print, print_value};
pub use tree::{Collection, Member, Node, Number, Op, Probe, Value, ValueRef, check};
