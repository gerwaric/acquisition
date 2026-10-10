//! Authoring errors: what the language says when a text or a tree is not a
//! query (C91 — "an ambiguity the grammar defines is an error that shows
//! the readings"). Structured, with stable kinds (C47, C11): a frontend
//! prints `message` and `readings`, a test or an agent reads `kind`.

use serde::Serialize;

use crate::answer::Request;
use crate::corpus::Realm;

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
    /// not ranged, `arg3` on a template with two numbers. And a slot word
    /// where none is taken: on a computed value that is one number, and
    /// behind `has:`, which asks a ranged total's presence.
    SlotUnknown,
    /// A slot word outside a line's group: `"T" low>=15 high<=45`.
    SlotOutsideGroup,
    /// A slot word beside two rows of a mod named together: which row's
    /// number is never guessed, and each reading quotes one.
    SlotOfTwoRows,
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
    /// count beside it, a crossed table of other than two keys, `--explain`
    /// beside a count or a sort; and a context that is no context.
    View,
    /// A path `--explain` was given that the query has no node at: the
    /// query's nodes are listed with their paths.
    NoSuchPath,
    /// What `--next` was given is no token an answer printed.
    Token,
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
    /// message, and what may be asked instead.
    Scope {
        kind: &'static str,
        message: String,
        offers: Vec<Offered>,
    },
    Store(anyhow::Error),
}

/// One way on from a refusal (`search/LEDGER.md`, H5): what a terminal
/// types, and — where that is a search — the request it is, for a caller
/// that sends requests and types nothing. The reading of a request is
/// the request's own command ([`Request::command`]), made here and
/// nowhere else, so the two never disagree.
#[derive(Debug, Clone, PartialEq)]
pub struct Offered {
    pub reading: String,
    pub request: Option<Request>,
    /// A realm offered where no request was at hand to put it in: the
    /// store's read knows the realms held and not what was asked
    /// ([`SearchError::asked_by`]).
    realm: Option<Realm>,
}

impl Offered {
    fn words(reading: String) -> Offered {
        Offered {
            reading,
            request: None,
            realm: None,
        }
    }

    fn request(request: Request) -> Offered {
        Offered {
            reading: request.command(),
            request: Some(request),
            realm: None,
        }
    }
}

impl SearchError {
    pub(crate) fn scope(kind: &'static str, message: impl Into<String>) -> SearchError {
        SearchError::Scope {
            kind,
            message: message.into(),
            offers: Vec::new(),
        }
    }

    /// Ways on that are words alone: no search runs them.
    pub(crate) fn with_offers(mut self, offered: Vec<String>) -> SearchError {
        if let SearchError::Scope { offers, .. } = &mut self {
            *offers = offered.into_iter().map(Offered::words).collect();
        }
        self
    }

    /// Ways on that are searches: each a request, read as its command.
    pub(crate) fn offering(mut self, requests: Vec<Request>) -> SearchError {
        if let SearchError::Scope { offers, .. } = &mut self {
            *offers = requests.into_iter().map(Offered::request).collect();
        }
        self
    }

    /// The realms a search may name, each the flag a terminal adds.
    pub(crate) fn offering_realms(mut self, realms: &[String]) -> SearchError {
        if let SearchError::Scope { offers, .. } = &mut self {
            *offers = realms
                .iter()
                .map(|realm| Offered {
                    reading: format!("--realm {realm}"),
                    request: None,
                    realm: Some(Realm::One(realm.clone())),
                })
                .collect();
        }
        self
    }

    /// The refusal as the answer to `request`: a realm offered becomes
    /// the request in that realm, nothing else of it changed.
    pub(crate) fn asked_by(mut self, request: &Request) -> SearchError {
        if let SearchError::Scope { offers, .. } = &mut self {
            for offer in offers.iter_mut() {
                if let Some(realm) = &offer.realm {
                    let mut asked = request.clone();
                    asked.scope.realm = Some(realm.clone());
                    offer.request = Some(asked);
                }
            }
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

    /// The failure as `--json` prints it (C11): `error`, `kind`, what
    /// the author may have meant as `readings`, and under `offers` each
    /// way on that is a request, as an answer's zero block holds one.
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
                    out["readings"] =
                        serde_json::json!(offers.iter().map(|o| &o.reading).collect::<Vec<_>>());
                }
                let requests: Vec<_> = offers
                    .iter()
                    .filter_map(|o| o.request.as_ref())
                    .map(|request| serde_json::json!({ "request": request }))
                    .collect();
                if !requests.is_empty() {
                    out["offers"] = serde_json::json!(requests);
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
                    write!(f, "{}{}", if i == 0 { " — " } else { " · " }, offer.reading)?;
                }
                Ok(())
            }
            SearchError::Store(e) => write!(f, "{e:#}"),
        }
    }
}

impl std::error::Error for SearchError {}
