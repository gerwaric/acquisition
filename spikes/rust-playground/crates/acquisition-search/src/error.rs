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
