//! The game side of a listing (C69): what an item's `note` or a tab's
//! name says, read as one pure function. Built by the pricing slice's
//! plan step 3 (`PRICING-SLICE.md`, 2026-09-06). Nothing here reads a
//! store; the listing state (plan step 4) calls [`read`] once per note
//! and once per tab name and keeps the raw text beside the result.
//!
//! # Decisions as recorded
//!
//! **C69 — A listing is 2 independent resolutions and their relation.** The
//! manual side resolves by specificity (C70); the game side reads note,
//! then tab name, as a price (exact or negotiable), `skip` (do not index),
//! `invalid` (an empty amount, or a ratio in a tab name) or none (T10,
//! T11), and whether its stash is public. The relation is manual-only,
//! game-only, agree, conflict or none; `ignore` is a manual disposition and
//! never denies an in-game price. Each result carries both sides with
//! causes, revisions, basis, parser and reference versions, the raw note
//! verbatim. What a relation *means* is each consumer's rule (C74), not a
//! frontend's. *Why:* four statements C++ fused and needed locks.
//! *Details:* `game_side.rs`. *Pinned:* the parse, `c69_`; the rest at step
//! 4. Amended 2026-09-04.
//!
//! # As built
//!
//! **The grammar is the game's, exactly** (T10: "the prefix, one space,
//! the amount, one space, the word — that is the whole grammar the game
//! wrote"). `~price <amount> <word>` is [`GamePrice::Exact`], `~b/o
//! <amount> <word>` is [`GamePrice::Negotiable`], `~skip` is
//! [`GamePrice::Skip`]; the amount is [`Amount`]'s grammar (a decimal of
//! at most four places, or `wanted/lot`, T2) and the word is any word
//! the shipped currency table resolves — tag, emit or a cited alias
//! (C68) — the price carrying the row's **tag**, so a hand-typed `~price
//! 5 exa` and the game's `~price 5 exalted` are one price. A retired
//! word still reads as a price: the table says it is retired, the
//! consumer decides what that means.
//!
//! **A leading `~` opens a price note.** Text that does not start with
//! `~` is [`GamePrice::None`]: not a price, whatever it says (the 0.18
//! corpus's one such note is `50`). Text that starts with `~` and is not
//! the grammar is [`GamePrice::Invalid`] naming why: the game's own
//! residue `~price  <word>` (an empty amount, two spaces — what the
//! dialog leaves after an invalid entry, T10), a ratio in a tab name
//! (T11: it unlists the whole tab), an amount [`Amount`] refuses (a fifth
//! decimal, a zero, a sign), a word the table does not know (spelled
//! the table's way, case-sensitive — the indexer's loose matching is not
//! modelled, C68), a prefix the game does not write (`~c/o`, `~gb/o`:
//! the C++ app's, never observed in a note or on the site), and, in a
//! note, anything after the word. A misread is one more fixture line
//! (`reference/price-notes-2026-09-04.txt`), never a special case.
//!
//! **The two sources differ in two rules** ([`Source`]). A tab name
//! tolerates trailing text after the word — the game itself appends
//! `(Remove-only)`, and the owner's letters `(A)`…`(G)` ride along (T11)
//! — where a note holds the grammar and nothing more. And a ratio is a
//! price in a note (T2) but invalid in a tab name (T11). Trailing
//! whitespace is trimmed from both before reading (`~skip ` is how the
//! game writes skip); leading whitespace and text are not tolerated —
//! the marker comes first or the text is not a price note.
//!
//! **Nothing is decided here.** `~skip` in a tab name, `~b/o` with a
//! ratio, a retired currency, a price on a non-public tab, which tab a
//! nested item's game side reads (C80: a substash its parent, a folder
//! child itself): the parser reports what one text says under
//! [`NOTE_PARSER_VERSION`], and every consumer (the listing state, the
//! render's policy table) cites that version beside its own rule (C74).

use std::fmt;

use crate::currency::CurrencyTable;
use crate::price::{Amount, Price};

/// The version of this grammar, cited by every result that carries a
/// game-side reading (C69). Bumped when a reading of the same text
/// changes.
pub const NOTE_PARSER_VERSION: u32 = 1;

/// Where a text came from; the two sources read under two rules (module
/// doc, "The two sources differ").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    /// An item's `note`, as the API returns it.
    Note,
    /// A stash tab's `name`.
    TabName,
}

impl Source {
    /// The word a result is labelled with.
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Note => "note",
            Source::TabName => "tab_name",
        }
    }
}

/// What a note or a tab name says about a price: C69's four outcomes,
/// the price split by its prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePrice {
    /// `~price <amount> <word>`.
    Exact(Price),
    /// `~b/o <amount> <word>`.
    Negotiable(Price),
    /// `~skip`: the dialog's "Do not index" (T10).
    Skip,
    /// A `~` text that is not the grammar; `why` names the reason.
    Invalid { why: String },
    /// Not a price note: no leading `~`.
    None,
}

impl GamePrice {
    /// The price, for the two outcomes that carry one.
    pub fn price(&self) -> Option<&Price> {
        match self {
            GamePrice::Exact(p) | GamePrice::Negotiable(p) => Some(p),
            GamePrice::Skip | GamePrice::Invalid { .. } | GamePrice::None => None,
        }
    }

    /// The outcome word: `exact`, `negotiable`, `skip`, `invalid`, `none`.
    pub fn kind(&self) -> &'static str {
        match self {
            GamePrice::Exact(_) => "exact",
            GamePrice::Negotiable(_) => "negotiable",
            GamePrice::Skip => "skip",
            GamePrice::Invalid { .. } => "invalid",
            GamePrice::None => "none",
        }
    }
}

impl fmt::Display for GamePrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GamePrice::Exact(p) => write!(f, "{p}"),
            GamePrice::Negotiable(p) => write!(f, "{p} b/o"),
            GamePrice::Skip => write!(f, "skip"),
            GamePrice::Invalid { why } => write!(f, "invalid: {why}"),
            GamePrice::None => write!(f, "none"),
        }
    }
}

/// The prefixes the game writes (T10), with the outcome each opens.
const EXACT: &str = "~price";
const NEGOTIABLE: &str = "~b/o";
const SKIP: &str = "~skip";

/// Read one note or tab name. Pure: the text, its source and the shipped
/// currency table in; one [`GamePrice`] out. Never fails — a text this
/// cannot read is an outcome, not an error.
pub fn read(source: Source, text: &str, table: &CurrencyTable) -> GamePrice {
    let text = text.trim_end();
    if !text.starts_with('~') {
        return GamePrice::None;
    }
    let invalid = |why: String| GamePrice::Invalid { why };

    // The prefix is the text up to the first space (or all of it).
    let (prefix, rest) = match text.split_once(' ') {
        Some((p, r)) => (p, Some(r)),
        None => (text, None),
    };
    if prefix == SKIP {
        return match rest {
            None => GamePrice::Skip,
            Some(more) => match source {
                Source::TabName => GamePrice::Skip,
                Source::Note => invalid(format!("`{SKIP}` takes nothing; found {more:?} after it")),
            },
        };
    }
    let negotiable = match prefix {
        EXACT => false,
        NEGOTIABLE => true,
        other => {
            return invalid(format!(
                "{other:?} is not a prefix the game writes ({EXACT}, {NEGOTIABLE}, {SKIP}; T10)"
            ));
        }
    };

    // `<amount> <word>`, then nothing (a note) or anything (a tab name).
    let Some(rest) = rest else {
        return invalid(format!(
            "`{prefix}` takes `<amount> <word>`; found nothing after it"
        ));
    };
    let Some((amount_text, after_amount)) = rest.split_once(' ') else {
        return invalid(format!(
            "`{prefix}` takes `<amount> <word>`; found only {rest:?} after it"
        ));
    };
    if amount_text.is_empty() {
        return invalid(
            "the amount is empty — what the dialog leaves after an invalid entry (T10)".into(),
        );
    }
    let amount = match amount_text.parse::<Amount>() {
        Ok(a) => a,
        Err(e) => return invalid(e.to_string()),
    };
    if source == Source::TabName && matches!(amount, Amount::Ratio { .. }) {
        return invalid(format!(
            "a ratio ({amount_text}) in a tab name is invalid and lists nothing (T11)"
        ));
    }
    // A tab name's suffix is tolerated, not read; a note's word must be
    // its last.
    let word = match source {
        Source::Note => after_amount,
        Source::TabName => after_amount
            .split_once(char::is_whitespace)
            .map_or(after_amount, |(word, _suffix)| word),
    };
    if word.is_empty() {
        return invalid(format!("`{prefix} {amount_text}` names no currency word"));
    }
    if word.chars().any(char::is_whitespace) {
        return invalid(format!(
            "a note is `{prefix} <amount> <word>` and nothing more; found {after_amount:?} after the amount"
        ));
    }
    let Some(row) = table.resolve(word) else {
        return invalid(format!(
            "{word:?} is not a word of currency table v{} (tag, emit or alias)",
            table.version()
        ));
    };
    let price = Price {
        amount,
        currency: row.tag.clone(),
    };
    if negotiable {
        GamePrice::Negotiable(price)
    } else {
        GamePrice::Exact(price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::table;
    use crate::price_notes_fixture::{Note, price_notes};

    fn note(text: &str) -> GamePrice {
        read(Source::Note, text, table().unwrap())
    }

    fn tab(text: &str) -> GamePrice {
        read(Source::TabName, text, table().unwrap())
    }

    fn price(amount: &str, tag: &str) -> Price {
        Price {
            amount: amount.parse().unwrap(),
            currency: tag.into(),
        }
    }

    /// C69, T10 — every note the game wrote reads as the game meant it:
    /// `~skip ` is skip, the empty amount is invalid (never "no price"),
    /// `~b/o` is negotiable, everything else is exact; the amount's
    /// canonical text is the note's own and the currency is the tag of
    /// the row whose `emit` is the note's word.
    #[test]
    fn c69_every_fixture_note_reads_as_the_game_meant_it() {
        let fixture = price_notes();
        assert!(fixture.notes.len() >= 50);
        let t = table().unwrap();
        for Note { note, .. } in &fixture.notes {
            let got = read(Source::Note, note, t);
            let mut parts = note.trim_end().splitn(3, ' ');
            let (prefix, amount, word) = (
                parts.next().unwrap(),
                parts.next().unwrap_or(""),
                parts.next().unwrap_or(""),
            );
            match (prefix, amount) {
                ("~skip", "") => assert_eq!(got, GamePrice::Skip, "{note:?}"),
                (_, "") => assert!(
                    matches!(&got, GamePrice::Invalid { why } if why.contains("empty")),
                    "{note:?} → {got}"
                ),
                (_, _) => {
                    let p = got
                        .price()
                        .unwrap_or_else(|| panic!("{note:?} → {got}, not a price"));
                    assert_eq!(p.amount.to_string(), amount, "{note:?}");
                    assert_eq!(p.currency, t.resolve(word).unwrap().tag, "{note:?}");
                    assert_eq!(&t.resolve(word).unwrap().emit, word);
                    assert_eq!(
                        got.kind(),
                        if prefix == "~b/o" {
                            "negotiable"
                        } else {
                            "exact"
                        }
                    );
                }
            }
        }
        let kinds = |k: &str| {
            fixture
                .notes
                .iter()
                .filter(|n| note(&n.note).kind() == k)
                .count()
        };
        assert_eq!(kinds("skip"), 1);
        assert_eq!(kinds("invalid"), 1);
        assert_eq!(kinds("negotiable"), 2);
        assert_eq!(kinds("none"), 0);
    }

    /// C69, T11 — every priced tab name in the owner's listing reads as
    /// an exact price despite the game's `(Remove-only)` and the owner's
    /// `(A)`…`(G)` after the word; the same text as a *note* is invalid,
    /// because a note holds the grammar and nothing more.
    #[test]
    fn c69_a_tab_name_tolerates_trailing_text_and_a_note_does_not() {
        let fixture = price_notes();
        assert!(fixture.tab_names.len() >= 13);
        for name in &fixture.tab_names {
            let got = tab(name);
            let p = got.price().unwrap_or_else(|| panic!("{name:?} → {got}"));
            assert_eq!(got.kind(), "exact", "{name:?}");
            assert!(matches!(p.amount, Amount::Decimal { .. }), "{name:?}");
            assert!(
                ["chaos", "divine"].contains(&p.currency.as_str()),
                "{name:?}"
            );
            if name.contains('(') {
                assert!(
                    matches!(note(name), GamePrice::Invalid { .. }),
                    "{name:?} as a note"
                );
            }
        }
        assert_eq!(
            tab("~price 30 chaos (C)"),
            GamePrice::Exact(price("30", "chaos"))
        );
        assert_eq!(
            tab("~price 20 chaos (A) (Remove-only)"),
            GamePrice::Exact(price("20", "chaos"))
        );
        // A tab name's suffix is tolerated, not read: the word must still
        // stand alone.
        assert!(matches!(
            tab("~price 30 chaos(C)"),
            GamePrice::Invalid { .. }
        ));
    }

    /// C69, T2, T11 — a lot ratio is a price in a note (the owner's own
    /// `~price 22/10 chaos`, kept unreduced) and invalid in a tab name,
    /// where it unlists the whole tab.
    #[test]
    fn c69_a_ratio_is_a_price_in_a_note_and_invalid_in_a_tab_name() {
        for (text, wanted, lot) in [
            ("~price 22/10 chaos", 22, 10),
            ("~price 55/600 chaos", 55, 600),
            ("~price 10/80 chaos", 10, 80),
            ("~price 3/1 alch", 3, 1),
        ] {
            let got = note(text);
            assert_eq!(
                got.price().unwrap().amount,
                Amount::Ratio { wanted, lot },
                "{text:?}"
            );
            assert!(
                matches!(&tab(text), GamePrice::Invalid { why } if why.contains("T11")),
                "{text:?} as a tab name"
            );
        }
        // The grammar does not know that the dialog refuses a ratio for
        // `~b/o` (T12); what it means is the consumer's rule.
        assert_eq!(
            note("~b/o 2/35 divine"),
            GamePrice::Negotiable(price("2/35", "divine"))
        );
    }

    /// C69, T10 — the game's residue after an invalid entry is invalid,
    /// never "no price"; `~skip` with or without the game's trailing
    /// space is skip; `~skip` followed by anything in a note is invalid.
    #[test]
    fn c69_the_games_residue_is_invalid_and_skip_is_skip() {
        assert!(
            matches!(&note("~price  chaos"), GamePrice::Invalid { why } if why.contains("empty"))
        );
        assert!(
            matches!(&note("~b/o  divine"), GamePrice::Invalid { why } if why.contains("empty"))
        );
        assert_eq!(note("~skip "), GamePrice::Skip);
        assert_eq!(note("~skip"), GamePrice::Skip);
        assert!(matches!(note("~skip this one"), GamePrice::Invalid { .. }));
        assert_eq!(tab("~skip (Remove-only)"), GamePrice::Skip);
        assert!(matches!(note("~price"), GamePrice::Invalid { .. }));
        assert!(matches!(note("~price chaos"), GamePrice::Invalid { .. }));
        assert!(matches!(note("~price 5 "), GamePrice::Invalid { .. }));
    }

    /// C69 — a leading `~` opens a price note: text without it is none,
    /// whatever it says; a `~` prefix the game does not write is invalid
    /// (the C++ app's `~c/o` and `~gb/o` included).
    #[test]
    fn c69_no_marker_is_none_and_an_unknown_marker_is_invalid() {
        for text in [
            "",
            "50",
            "Sale ~price 5 chaos",
            "price 5 chaos",
            " ~price 5 chaos",
        ] {
            assert_eq!(note(text), GamePrice::None, "{text:?}");
            assert_eq!(tab(text), GamePrice::None, "{text:?}");
        }
        for text in ["~c/o 5 chaos", "~gb/o 5 chaos", "~PRICE 5 chaos", "~"] {
            assert!(
                matches!(&note(text), GamePrice::Invalid { why } if why.contains("prefix")),
                "{text:?}"
            );
        }
    }

    /// C69, C68 — the word is resolved through the shipped table, exact
    /// and case-sensitive: a cited alias reads as the row's tag, a
    /// retired word still reads as a price, and an unknown or miscased
    /// word is invalid naming the table version.
    #[test]
    fn c69_the_currency_word_resolves_to_a_tag_or_is_invalid() {
        assert_eq!(
            note("~price 5 exa"),
            GamePrice::Exact(price("5", "exalted"))
        );
        assert_eq!(
            note("~price 5 exalted"),
            GamePrice::Exact(price("5", "exalted"))
        );
        assert_eq!(
            note("~b/o 1.5 divine"),
            GamePrice::Negotiable(price("1.5", "divine"))
        );
        let retired = note("~price 5 chisel");
        assert_eq!(retired, GamePrice::Exact(price("5", "chisel")));
        assert!(table().unwrap().by_tag("chisel").unwrap().is_retired());
        for text in ["~price 5 Chaos", "~price 5 chaoss", "~price 5 chaos orb"] {
            assert!(
                matches!(&note(text), GamePrice::Invalid { why } if why.contains("v1") || why.contains("nothing more")),
                "{text:?} → {}",
                note(text)
            );
        }
    }

    /// C69, C67 — the amount is `Amount`'s grammar: a fifth decimal, a
    /// zero, a leading zero or a sign is invalid, naming the amount rule;
    /// the game's own four places and five digits read back verbatim.
    #[test]
    fn c69_the_amount_is_the_typed_grammar() {
        assert_eq!(
            note("~price 999.1234 chaos"),
            GamePrice::Exact(price("999.1234", "chaos"))
        );
        assert_eq!(
            note("~price 12345 chaos"),
            GamePrice::Exact(price("12345", "chaos"))
        );
        for text in [
            "~price 999.12345 chaos",
            "~price 0 chaos",
            "~price 05 chaos",
            "~price -5 chaos",
            "~price 5. chaos",
            "~price 10/0 chaos",
        ] {
            assert!(
                matches!(&note(text), GamePrice::Invalid { why } if why.starts_with("amount")),
                "{text:?} → {}",
                note(text)
            );
        }
    }
}
