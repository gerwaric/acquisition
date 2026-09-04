//! The currency reference table — the first reference-data input (C68;
//! `decisions/pricing.md`), built by the pricing slice's plan step 1
//! (`PRICING-SLICE.md`, 2026-09-04).
//!
//! # Decisions as recorded
//!
//! **C68 — Reference data is a fifth input, versioned by the build: a
//! reviewed, committed table whose every row cites its evidence, shipped
//! inside the binary, read-only, never in a store file, enumerable
//! through every surface, cited by version wherever used.** A tool may
//! propose rows from a governed source (C79); a human commits. The
//! currency table is first: the immutable `tag` intent cites; `emit`, the
//! word GGG's client writes into a note (the 2026-09-04 run); `aliases`,
//! that word plus the legacy C++ tag, nothing more — the indexer's loose
//! matching is not modelled. A tag is never removed or reused; a dropped
//! currency keeps its row, marked; additions are reported. *Why:* not a
//! fact, not intent, not a derivation (pattern 8); the C++ list rotted
//! without evidence. Amended 2026-09-04.
//!
//! # As built
//!
//! The table is `reference/currency-v1.toml` beside this crate, compiled
//! into the binary by `include_str!` and parsed once, on first use
//! ([`table`]); the file's header states the row rules and the evidence
//! grammar, and is the human-reviewed artifact — this module only reads
//! it. Parsing is strict (`deny_unknown_fields`) and the loader checks
//! what a reviewer would: the version stamp is the one this build
//! expects, every `tag`, `emit` and alias is a single word, every word
//! resolves to exactly one row across the whole table, and every row
//! cites at least one `game:` evidence entry. A table that fails is a
//! build defect surfaced as [`CurrencyTableError`] to every caller (the
//! crate's no-panic ratchet applies; the `c68_` tests pin the shipped
//! file), never a partial table.
//!
//! [`CurrencyTable::resolve`] is the one lookup: exact, case-sensitive,
//! over tag, emit and aliases. A retired row still resolves — intent may
//! cite it forever — and says so through [`Currency::retired`]; what a
//! consumer does with a retired currency (refuse a new price, render an
//! old one) is that consumer's rule. Enumeration is [`CurrencyTable::rows`]
//! in the file's order (the in-game dialog's, then the retired rows),
//! and every surface that shows a currency cites [`CurrencyTable::version`].
//!
//! v1 (2026-09-04): 39 active rows — the words the in-game price dialog
//! writes, with the display names the owner read in it — and three
//! retired (`chisel`, `coin`, `silver`: the C++ table had them, the dialog
//! no longer offers them). The four C++ tags that differed from the game's
//! word (`exa`, `chrom`, `jew`, `fuse`) are aliases of `exalted`, `chrome`,
//! `jewellers`, `fusing`.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use acquisition_core::realm::Realm;

/// The table this build ships, verbatim (the reviewed file).
pub const CURRENCY_TABLE_TOML: &str = include_str!("../reference/currency-v1.toml");

/// The table version this build expects; a file stamped otherwise is a
/// build defect, not a runtime condition.
pub const CURRENCY_TABLE_VERSION: u32 = 1;

/// One currency: the identity intent cites, the word the game writes, the
/// name a human reads, and the evidence for each.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Currency {
    /// Immutable identity (C67 cites it). Never removed, never reused.
    pub tag: String,
    /// The name the in-game price dialog shows.
    pub display: String,
    /// The word GGG's client writes into a note after the amount — and so
    /// the word a render writes.
    pub emit: String,
    /// Extra words that resolve to this row beyond `tag` and `emit`: the
    /// legacy C++ tag where it differed, nothing more.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Realm applicability.
    pub realms: Vec<Realm>,
    /// Set when the game no longer offers the currency: when and on what
    /// evidence. The row stays; intent may still cite it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retired: Option<String>,
    /// One entry per source, in the file header's grammar; at least one
    /// `game:` entry per row.
    pub evidence: Vec<String>,
}

impl Currency {
    /// Every word that resolves to this row: tag, emit, then aliases.
    pub fn words(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.tag.as_str())
            .chain(std::iter::once(self.emit.as_str()))
            .chain(self.aliases.iter().map(String::as_str))
    }

    pub fn is_retired(&self) -> bool {
        self.retired.is_some()
    }
}

/// The parsed, checked table. Read-only; one per build.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrencyTable {
    version: u32,
    status: String,
    sources: Vec<String>,
    #[serde(rename = "currency", default)]
    rows: Vec<Currency>,
}

/// Why a table text is not a table. Every arm names the offending row or
/// word so a reviewer can fix the file, not the loader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurrencyTableError {
    /// The text is not the TOML shape the loader reads.
    Parse(String),
    /// The file's `version` is not [`CURRENCY_TABLE_VERSION`].
    Version { found: u32, expected: u32 },
    /// A tag, emit or alias is empty or holds whitespace.
    MalformedWord { tag: String, word: String },
    /// One word would resolve to two rows.
    AmbiguousWord { word: String, tags: [String; 2] },
    /// A row cites no `game:` evidence (or none at all).
    Uncited { tag: String },
    /// A row names no realm.
    NoRealm { tag: String },
    /// The table has no rows.
    Empty,
}

impl std::fmt::Display for CurrencyTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurrencyTableError::Parse(e) => write!(f, "currency table does not parse: {e}"),
            CurrencyTableError::Version { found, expected } => write!(
                f,
                "currency table is version {found}; this build reads version {expected}"
            ),
            CurrencyTableError::MalformedWord { tag, word } => {
                write!(f, "currency {tag:?}: {word:?} is not a single word")
            }
            CurrencyTableError::AmbiguousWord { word, tags } => write!(
                f,
                "currency word {word:?} resolves to both {:?} and {:?}",
                tags[0], tags[1]
            ),
            CurrencyTableError::Uncited { tag } => {
                write!(f, "currency {tag:?} cites no `game:` evidence")
            }
            CurrencyTableError::NoRealm { tag } => write!(f, "currency {tag:?} names no realm"),
            CurrencyTableError::Empty => write!(f, "currency table has no rows"),
        }
    }
}

impl std::error::Error for CurrencyTableError {}

impl CurrencyTable {
    /// Parse and check a table text. The shipped file goes through
    /// [`table`]; this is the same path over any text, for the tests
    /// that pin what the loader refuses.
    pub fn parse(text: &str) -> Result<CurrencyTable, CurrencyTableError> {
        let table: CurrencyTable =
            toml::from_str(text).map_err(|e| CurrencyTableError::Parse(e.to_string()))?;
        table.check()?;
        Ok(table)
    }

    fn check(&self) -> Result<(), CurrencyTableError> {
        if self.version != CURRENCY_TABLE_VERSION {
            return Err(CurrencyTableError::Version {
                found: self.version,
                expected: CURRENCY_TABLE_VERSION,
            });
        }
        if self.rows.is_empty() {
            return Err(CurrencyTableError::Empty);
        }
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for row in &self.rows {
            for word in row.words() {
                if word.is_empty() || word.chars().any(char::is_whitespace) {
                    return Err(CurrencyTableError::MalformedWord {
                        tag: row.tag.clone(),
                        word: word.to_string(),
                    });
                }
                // A row's own tag and emit are one word today; the same
                // row naming a word twice is not an ambiguity.
                if let Some(other) = seen.insert(word, &row.tag)
                    && other != row.tag
                {
                    return Err(CurrencyTableError::AmbiguousWord {
                        word: word.to_string(),
                        tags: [other.to_string(), row.tag.clone()],
                    });
                }
            }
            if !row.evidence.iter().any(|e| e.starts_with("game:")) {
                return Err(CurrencyTableError::Uncited {
                    tag: row.tag.clone(),
                });
            }
            if row.realms.is_empty() {
                return Err(CurrencyTableError::NoRealm {
                    tag: row.tag.clone(),
                });
            }
        }
        Ok(())
    }

    /// The version every surface cites beside a currency.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The file's review status, verbatim ("draft" until the owner has
    /// read it). Exposed, gated on by nothing.
    pub fn status(&self) -> &str {
        &self.status
    }

    /// The table-level sources, verbatim.
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Every row in the file's order: the in-game dialog's, then retired.
    pub fn rows(&self) -> &[Currency] {
        &self.rows
    }

    /// The rows the game still offers.
    pub fn active(&self) -> impl Iterator<Item = &Currency> {
        self.rows.iter().filter(|r| !r.is_retired())
    }

    /// The one lookup: exact and case-sensitive over tag, emit and aliases.
    /// A retired row resolves and says so.
    pub fn resolve(&self, word: &str) -> Option<&Currency> {
        self.rows.iter().find(|r| r.words().any(|w| w == word))
    }

    /// A row by its immutable tag only (what intent cites).
    pub fn by_tag(&self, tag: &str) -> Option<&Currency> {
        self.rows.iter().find(|r| r.tag == tag)
    }
}

static TABLE: LazyLock<Result<CurrencyTable, CurrencyTableError>> =
    LazyLock::new(|| CurrencyTable::parse(CURRENCY_TABLE_TOML));

/// The table this build ships, parsed and checked once. An error here is
/// a build defect (the file failed the loader's review checks); callers
/// surface it, they never guess a row.
pub fn table() -> Result<&'static CurrencyTable, CurrencyTableError> {
    TABLE.as_ref().map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture the table cites, read the way the loader's evidence
    /// grammar describes it: the notes' currency words (first section;
    /// the tab-name section between is skipped) and the dialog's
    /// word→display list (third section).
    fn fixture() -> (Vec<String>, Vec<(String, String)>) {
        let text = include_str!("../reference/price-notes-2026-09-04.txt");
        let mut note_words = Vec::new();
        let mut dialog = Vec::new();
        let mut section = 0;
        for line in text.lines() {
            if line.starts_with('#') {
                if line.contains("Tab names that parse as prices") {
                    section = 1;
                } else if line.contains("currency list as the owner read it") {
                    section = 2;
                }
                continue;
            }
            if line.trim().is_empty() {
                continue;
            }
            match section {
                0 if line.starts_with('~') => {
                    let note = line.split('\t').next().unwrap();
                    let mut parts = note.splitn(3, ' ');
                    let (_prefix, _amount, word) =
                        (parts.next(), parts.next(), parts.next().unwrap_or(""));
                    if !word.is_empty() {
                        note_words.push(word.to_string());
                    }
                }
                2 => {
                    let (word, display) = line.split_once('\t').unwrap();
                    dialog.push((word.to_string(), display.to_string()));
                }
                _ => {}
            }
        }
        (note_words, dialog)
    }

    /// C68: the shipped file parses under the loader's review checks and
    /// carries the version this build cites.
    #[test]
    fn c68_the_shipped_table_parses_and_is_version_1() {
        let t = table().unwrap();
        assert_eq!(t.version(), CURRENCY_TABLE_VERSION);
        assert_eq!(t.rows().len(), 42);
        assert_eq!(t.active().count(), 39);
        assert!(!t.sources().is_empty());
        assert!(t.rows().iter().all(|r| !r.evidence.is_empty()));
    }

    /// C68: `emit` is the word the game writes — every word the in-game
    /// dialog wrote into a note resolves to a row whose `emit` is that
    /// word, and the dialog's list (word → display name, as the owner
    /// read it) is exactly the active rows, in order.
    #[test]
    fn c68_the_active_rows_are_the_dialog_words_and_display_names() {
        let t = table().unwrap();
        let (note_words, dialog) = fixture();
        assert!(!note_words.is_empty());
        for word in &note_words {
            let row = t
                .resolve(word)
                .unwrap_or_else(|| panic!("{word} unresolved"));
            assert_eq!(&row.emit, word, "{word} is a note word, so it is an emit");
            assert!(
                !row.is_retired(),
                "{word}: the game wrote it, so it is not retired"
            );
        }
        let active: Vec<(String, String)> = t
            .active()
            .map(|r| (r.emit.clone(), r.display.clone()))
            .collect();
        assert_eq!(active, dialog);
        let noted: std::collections::BTreeSet<&str> =
            note_words.iter().map(String::as_str).collect();
        for (word, _) in &dialog {
            assert!(
                noted.contains(word.as_str()),
                "{word}: in the dialog, never in a note"
            );
        }
    }

    /// C68: the legacy C++ tag is an alias where it differed from the
    /// game's word; where it did not, the row carries no alias.
    #[test]
    fn c68_the_legacy_tags_resolve_as_aliases() {
        let t = table().unwrap();
        for (legacy, tag) in [
            ("exa", "exalted"),
            ("chrom", "chrome"),
            ("jew", "jewellers"),
            ("fuse", "fusing"),
        ] {
            let row = t.resolve(legacy).unwrap();
            assert_eq!(row.tag, tag);
            assert_eq!(row.aliases, vec![legacy.to_string()]);
            assert!(row.evidence.iter().any(|e| e.starts_with("cpp:")));
        }
        let with_alias = t.rows().iter().filter(|r| !r.aliases.is_empty()).count();
        assert_eq!(
            with_alias, 4,
            "aliases are the legacy tags that differed, nothing more"
        );
    }

    /// C68: a dropped currency keeps its row, marked, and still resolves
    /// by tag — intent that cites it never dangles.
    #[test]
    fn c68_a_retired_tag_keeps_its_row_and_is_marked() {
        let t = table().unwrap();
        for tag in ["chisel", "coin", "silver"] {
            let row = t.resolve(tag).unwrap();
            assert_eq!(row.tag, tag);
            assert!(row.is_retired(), "{tag} is retired");
            assert!(row.retired.as_deref().unwrap().starts_with("2026-09-04"));
        }
        let retired: Vec<&str> = t
            .rows()
            .iter()
            .filter(|r| r.is_retired())
            .map(|r| r.tag.as_str())
            .collect();
        assert_eq!(retired, ["chisel", "coin", "silver"]);
    }

    /// C68: resolution is exact — the indexer's loose matching is not
    /// modelled — and a word nobody wrote resolves to nothing.
    #[test]
    fn c68_resolution_is_exact_and_case_sensitive() {
        let t = table().unwrap();
        assert!(t.resolve("chaos").is_some());
        assert!(t.resolve("Chaos").is_none());
        assert!(t.resolve("chaos ").is_none());
        assert!(t.resolve("exalt").is_none());
        assert!(t.resolve("").is_none());
        assert!(t.by_tag("exa").is_none(), "by_tag ignores aliases");
        assert_eq!(t.by_tag("exalted").unwrap().tag, "exalted");
    }

    /// C68: the loader is the reviewer's checklist — a duplicate word, an
    /// uncited row, a foreign version stamp, an unknown field: refused
    /// whole, with the offending row named.
    #[test]
    fn c68_the_loader_refuses_what_a_reviewer_would() {
        let row = |tag: &str, emit: &str, aliases: &str, evidence: &str| {
            format!(
                "[[currency]]\ntag = \"{tag}\"\ndisplay = \"X\"\nemit = \"{emit}\"\naliases = [{aliases}]\nrealms = [\"pc\"]\nevidence = [{evidence}]\n"
            )
        };
        let head = "version = 1\nstatus = \"draft\"\nsources = []\n";
        let ok = format!("{head}{}", row("chaos", "chaos", "", "\"game:x\""));
        assert!(CurrencyTable::parse(&ok).is_ok());

        let dup = format!(
            "{head}{}{}",
            row("exalted", "exalted", "\"exa\"", "\"game:x\""),
            row("exa2", "exa", "", "\"game:x\"")
        );
        assert_eq!(
            CurrencyTable::parse(&dup),
            Err(CurrencyTableError::AmbiguousWord {
                word: "exa".into(),
                tags: ["exalted".into(), "exa2".into()]
            })
        );

        let uncited = format!("{head}{}", row("chaos", "chaos", "", "\"cpp:x\""));
        assert_eq!(
            CurrencyTable::parse(&uncited),
            Err(CurrencyTableError::Uncited {
                tag: "chaos".into()
            })
        );

        let spaced = format!("{head}{}", row("chaos", "chaos orb", "", "\"game:x\""));
        assert!(matches!(
            CurrencyTable::parse(&spaced),
            Err(CurrencyTableError::MalformedWord { .. })
        ));

        let v2 = ok.replacen("version = 1", "version = 2", 1);
        assert_eq!(
            CurrencyTable::parse(&v2),
            Err(CurrencyTableError::Version {
                found: 2,
                expected: 1
            })
        );

        let unknown = format!("{ok}colour = \"gold\"\n");
        assert!(matches!(
            CurrencyTable::parse(&unknown),
            Err(CurrencyTableError::Parse(_))
        ));

        assert_eq!(CurrencyTable::parse(head), Err(CurrencyTableError::Empty));
    }

    /// C68: read-only and enumerable — the JSON a surface emits is the
    /// table itself, every row with its evidence, and round-trips.
    #[test]
    fn c68_the_table_serializes_whole_and_round_trips() {
        let t = table().unwrap();
        let json = serde_json::to_value(t).unwrap();
        assert_eq!(json["version"], 1);
        assert_eq!(json["currency"].as_array().unwrap().len(), 42);
        assert!(json["currency"][0]["evidence"].as_array().unwrap().len() >= 2);
        let back: CurrencyTable = serde_json::from_value(json).unwrap();
        assert_eq!(&back, t);
    }
}
