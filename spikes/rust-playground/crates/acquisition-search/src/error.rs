//! Authoring errors: what the language says when a text or a tree is not a
//! query (C91 — "an ambiguity the grammar defines is an error that shows
//! the readings"). Structured, with stable kinds (C47, C11): a frontend
//! prints `message` and `readings`, a test or an agent reads `kind`.

use serde::Serialize;

/// Stable kinds. Additive: a new kind is a new variant, never a rename.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// A word with no operator: never guessed, its readings shown.
    BareWord,
    /// `a b or c`, unparenthesised: both readings shown.
    MixedAndOr,
    /// A template typed with its numbers: any value, or that value.
    TemplateWithNumbers,
    /// `-#` in a quoted template: the sign is the number's (C90), so a
    /// negative value is asked for by a comparison. (`+#` is spelling, and
    /// is dropped.)
    SignedTemplate,
    /// A comparison, sort or sum names no slot on a template with several
    /// numbers, or with none (C92): the slots are listed.
    SlotMissing,
    /// A slot the quoted template does not have: `low` on a line that is
    /// not ranged, `arg3` on a template with two numbers.
    SlotUnknown,
    /// A slot word outside a line's group: `"T" low>=15 high<=45`.
    SlotOutsideGroup,
    /// `has:` on a computed value.
    HasOnComputed,
    /// `realm:` — the realm is the scope, never a term (C96).
    RealmIsScope,
    /// `word(` that is no call of the language: `mod(`, `stat(`.
    UnknownCall,
    /// `holds( … )` with no bound.
    HoldsNeedsBound,
    /// `>`, `>=`, `<`, `<=` against something that is not a number.
    ComparisonNeedsNumber,
    /// A range under an operator other than `=`.
    RangeNeedsEquals,
    /// A quoted string that never closes.
    UnterminatedString,
    /// A backslash before anything but `"`, `\` or `n`.
    BadEscape,
    /// A group with nothing in it: `()`, `line()`.
    EmptyGroup,
    /// A member group inside a member group, or a term that belongs to the
    /// item inside one.
    NotInsideGroup,
    /// Anything else the grammar does not allow here.
    Unexpected,
    /// A tree that arrived as JSON and is not one the language can say.
    Tree,
    /// A field, a flag, a line attribute or a `--describe` name the
    /// language does not know: near names offered, never a guess.
    UnknownName,
    /// A value outside a closed set: the legal values offered.
    UnknownValue,
    /// An operator or a value the named thing does not take: `ilvl:84`,
    /// `rarity>=rare`.
    OperatorMismatch,
    /// A `~` pattern that does not compile.
    BadPattern,
    /// A view the request cannot have: a key named twice, `--sum` with no
    /// count beside it, a crossed table of other than two keys.
    View,
    /// A construct of the reference this build does not evaluate yet: an
    /// error of its own, never the unknown-name error, never undecided,
    /// never an empty answer (the build plan, rule 1).
    NotBuilt,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LanguageError {
    pub kind: ErrorKind,
    pub message: String,
    /// Byte offsets into the text, when the error came from a text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<(usize, usize)>,
    /// What the author may have meant, each a text the parser accepts.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub readings: Vec<String>,
}

impl LanguageError {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> LanguageError {
        LanguageError {
            kind,
            message: message.into(),
            span: None,
            readings: Vec::new(),
        }
    }

    pub(crate) fn with_readings(mut self, readings: Vec<String>) -> LanguageError {
        self.readings = readings;
        self
    }

    pub(crate) fn at(mut self, start: usize, end: usize) -> LanguageError {
        self.span.get_or_insert((start, end));
        self
    }
}

impl std::fmt::Display for LanguageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        for (i, reading) in self.readings.iter().enumerate() {
            write!(f, "{}{reading}", if i == 0 { " — " } else { " · " })?;
        }
        Ok(())
    }
}

impl std::error::Error for LanguageError {}

/// What a search or a show can fail with (C47, C11): an authoring error,
/// a scope the request cannot be run over, or the store's own failure.
#[derive(Debug)]
pub enum SearchError {
    Language(LanguageError),
    /// The scope or the item named, not the query: a stable kind, a
    /// message, and what may be typed instead.
    Scope {
        kind: &'static str,
        message: String,
        offers: Vec<String>,
    },
    Store(anyhow::Error),
}

impl SearchError {
    pub(crate) fn scope(kind: &'static str, message: impl Into<String>) -> SearchError {
        SearchError::Scope {
            kind,
            message: message.into(),
            offers: Vec::new(),
        }
    }

    pub(crate) fn with_offers(mut self, offered: Vec<String>) -> SearchError {
        if let SearchError::Scope { offers, .. } = &mut self {
            *offers = offered;
        }
        self
    }

    pub(crate) fn store(e: anyhow::Error) -> SearchError {
        SearchError::Store(e)
    }

    /// The stable kind: an authoring error's own, a scope error's, or
    /// `store`.
    pub fn kind(&self) -> String {
        match self {
            SearchError::Language(e) => serde_json::to_value(e.kind)
                .ok()
                .and_then(|k| k.as_str().map(str::to_string))
                .unwrap_or_default(),
            SearchError::Scope { kind, .. } => kind.to_string(),
            SearchError::Store(_) => "store".to_string(),
        }
    }

    /// The failure as `--json` prints it (C11): `error`, `kind`, and what
    /// the author may have meant.
    pub fn to_json(&self) -> serde_json::Value {
        let mut out = serde_json::json!({ "error": self.to_string(), "kind": self.kind() });
        match self {
            SearchError::Language(e) => {
                out["error"] = serde_json::json!(e.message);
                if !e.readings.is_empty() {
                    out["readings"] = serde_json::json!(e.readings);
                }
                if let Some((start, end)) = e.span {
                    out["span"] = serde_json::json!([start, end]);
                }
            }
            SearchError::Scope {
                message, offers, ..
            } => {
                out["error"] = serde_json::json!(message);
                if !offers.is_empty() {
                    out["readings"] = serde_json::json!(offers);
                }
            }
            SearchError::Store(_) => {}
        }
        out
    }
}

impl From<LanguageError> for SearchError {
    fn from(e: LanguageError) -> SearchError {
        SearchError::Language(e)
    }
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::Language(e) => write!(f, "{e}"),
            SearchError::Scope {
                message, offers, ..
            } => {
                write!(f, "{message}")?;
                for (i, offer) in offers.iter().enumerate() {
                    write!(f, "{}{offer}", if i == 0 { " — " } else { " · " })?;
                }
                Ok(())
            }
            SearchError::Store(e) => write!(f, "{e:#}"),
        }
    }
}

impl std::error::Error for SearchError {}
