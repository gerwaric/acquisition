//! The parser: a query's text to its tree (C91, C104), over the whole
//! language of the reference (`search/DESIGN.md`, "The language
//! reference") — composition, strings, item-level terms, member groups,
//! slots and values.
//!
//! What it knows and what it leaves. It knows the grammar, the words the
//! grammar keeps (`and or not`, the calls, `has: is: pseudo. realm:`, the
//! slot words) and what a quoted template says about its own numbers. It
//! knows no field, class, flag or computed value by name: `field op value`
//! is a shape here, and the binder decides whether the field exists — so
//! validity never depends on a corpus, and `mod(` is an ordinary unknown
//! call.
//!
//! Shorthand lowers at once and a tree never holds it: `"T">=90` is
//! `line("T" arg1>=90)`, `line(P).avg>=20` is `line(P avg>=20)`,
//! `sum("T")` is `sum(line("T").arg1)`. A comparison that names no slot is
//! decided by the template's own `#`s **before** it lowers (C92): one is
//! `arg1`, several is the error that lists them, none is an error.
//!
//! A quoted string with a `#` in it is a template, never a phrase: alone it
//! is `line("T")`, the item carries the line (and `-"T"`, it does not).
//! In a quoted template a `+` before a `#` is spelling, as the game and the
//! trade site write a line, and is dropped — the sign is the number's
//! (C90); a `-` there is an error that offers the comparison.
//!
//! There are no silent modes: a bare word, `a b or c` unparenthesised, a
//! template typed with its numbers, and slot words outside their group are
//! each an error that shows its readings, and every reading is a text this
//! parser accepts.

use crate::error::{ErrorKind, LanguageError};
use crate::print;
use crate::template;
use crate::tree::{
    self, Collection, Member, Node, Number, Op, Probe, Value, ValueRef, is_keyword, is_slot_word,
    is_word_char,
};

/// The calls of the language, as an unknown one is answered with.
const CALLS: [&str; 7] = [
    "line",
    "linked",
    "holds",
    "undecided",
    "sum",
    "true",
    "false",
];

/// Parse a query. The empty text is the empty query: every item in scope.
pub fn parse(text: &str) -> Result<Node, LanguageError> {
    let mut p = Parser { src: text, pos: 0 };
    let root = match p.sequence(Level::Root)? {
        Some(node) => node,
        None => Node::All(Vec::new()),
    };
    p.skip_ws();
    if !p.at_end() {
        return Err(p.unexpected("the query to end"));
    }
    finish(root)
}

/// Parse a value: what `--sort` and `--sum` take.
pub fn parse_value(text: &str) -> Result<ValueRef, LanguageError> {
    let mut p = Parser { src: text, pos: 0 };
    p.skip_ws();
    let value = p.value_ref(ValueContext::Standalone)?;
    p.skip_ws();
    if !p.at_end() {
        return Err(p.unexpected("the value to end"));
    }
    match tree::check(&Node::Undecided(Probe::Thing(value.clone()))) {
        Ok(()) => Ok(value),
        Err(e) => Err(e),
    }
}

/// The tree's own rules, run on the parser's output as on any tree, and a
/// template typed with its numbers answered with its two readings: any
/// value, or that value.
fn finish(root: Node) -> Result<Node, LanguageError> {
    match tree::check(&root) {
        Ok(()) => Ok(root),
        Err(e) if e.kind == ErrorKind::TemplateWithNumbers => {
            let any = print::print(&retyped_node(&root, false));
            let that = print::print(&retyped_node(&root, true));
            Err(e.with_readings(vec![any, that]))
        }
        Err(e) => Err(e),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Level {
    Root,
    /// Inside `( … )`, `undecided( … )`: ends at `)`.
    Paren,
    /// A `holds` argument: ends at `,` or `)`.
    Argument,
}

#[derive(Clone, Copy, PartialEq)]
enum ValueContext {
    /// `--sort`, `--sum`: a quoted template or a group with no slot named
    /// takes its one slot, by the template's own numbers.
    Standalone,
    /// Inside `undecided( … )`: a lone `"…"` or `line( … )` is a term.
    Thing,
}

#[derive(Clone, Copy, PartialEq)]
enum Conn {
    And,
    Or,
}

struct Parser<'a> {
    src: &'a str,
    pos: usize,
}

/// One term of an item-level sequence, with what the sequence needs to
/// know about how it was written.
struct Item {
    node: Node,
    start: usize,
    /// The text of a bare `"…"` — a phrase, or a template alone: what slot
    /// words may wrongly follow.
    quoted: Option<String>,
}

impl<'a> Parser<'a> {
    // ---- characters --------------------------------------------------

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn peek_at(&self, n: usize) -> Option<char> {
        self.rest().chars().nth(n)
    }

    fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += c.len_utf8();
            true
        } else {
            false
        }
    }

    fn skip_ws(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn error(&self, kind: ErrorKind, message: impl Into<String>, start: usize) -> LanguageError {
        LanguageError::new(kind, message).at(start, self.pos.max(start))
    }

    fn unexpected(&self, expected: &str) -> LanguageError {
        let found = match self.peek() {
            Some(c) => format!("`{c}`"),
            None => "the end of the text".to_string(),
        };
        LanguageError::new(
            ErrorKind::Unexpected,
            format!("expected {expected}, found {found}"),
        )
        .at(self.pos, self.pos + self.peek().map_or(0, char::len_utf8))
    }

    fn expect(&mut self, c: char, what: &str) -> Result<(), LanguageError> {
        if self.eat(c) {
            Ok(())
        } else {
            Err(self.unexpected(what))
        }
    }

    /// A run of word characters, or nothing.
    fn word(&mut self) -> &'a str {
        let start = self.pos;
        while self.peek().is_some_and(is_word_char) {
            self.bump();
        }
        &self.src[start..self.pos]
    }

    /// A keyword here, followed by something that is not part of a word.
    fn keyword(&mut self, word: &str) -> bool {
        let rest = self.rest();
        let matches = rest.len() >= word.len()
            && rest.is_char_boundary(word.len())
            && rest[..word.len()].eq_ignore_ascii_case(word)
            && !rest[word.len()..].chars().next().is_some_and(is_word_char);
        if matches {
            self.pos += word.len();
        }
        matches
    }

    // ---- strings, numbers, values ------------------------------------

    /// A quoted string; the opening quote is next.
    fn string(&mut self) -> Result<String, LanguageError> {
        let start = self.pos;
        self.bump();
        let mut out = String::new();
        loop {
            match self.bump() {
                None => {
                    return Err(self.error(
                        ErrorKind::UnterminatedString,
                        "a quoted string never closes",
                        start,
                    ));
                }
                Some('"') => return Ok(out),
                Some('\\') => match self.bump() {
                    Some('"') => out.push('"'),
                    Some('\\') => out.push('\\'),
                    Some('n') => out.push('\n'),
                    _ => {
                        return Err(self.error(
                            ErrorKind::BadEscape,
                            "inside quotes the only escapes are \\\" \\\\ and \\n",
                            self.pos.saturating_sub(2),
                        ));
                    }
                },
                Some(c) => out.push(c),
            }
        }
    }

    /// A number, if one is next: an optional sign, digits, an optional
    /// fraction — and nothing word-like after it, or it is a word (`6L`, an
    /// id that starts with a digit). Never consumes the `..` of a range.
    fn number(&mut self) -> Option<Number> {
        let start = self.pos;
        let rest = self.rest();
        let bytes = rest.as_bytes();
        let mut i = 0;
        if matches!(bytes.first(), Some(b'+' | b'-')) {
            i += 1;
        }
        let digits = i;
        while bytes.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == digits {
            return None;
        }
        let mut fraction = false;
        if bytes.get(i) == Some(&b'.') && bytes.get(i + 1).is_some_and(u8::is_ascii_digit) {
            fraction = true;
            i += 1;
            while bytes.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
        }
        if rest[i..].chars().next().is_some_and(is_word_char) {
            return None;
        }
        let literal = &rest[..i];
        let number = if fraction {
            Number::from_f64(literal.parse().ok()?)
        } else {
            match literal.parse::<i64>() {
                Ok(n) => Number::Int(n),
                Err(_) => Number::from_f64(literal.parse().ok()?),
            }
        };
        self.pos = start + i;
        Some(number)
    }

    /// The right-hand side of `op`: a quoted string, a number, a range, or
    /// one plain word.
    fn value(&mut self, op: Op) -> Result<Value, LanguageError> {
        let start = self.pos;
        let value = if self.peek() == Some('"') {
            Value::Text(self.string()?)
        } else if self.rest().starts_with("..") {
            self.pos += 2;
            match self.number() {
                Some(to) => Value::Range {
                    from: None,
                    to: Some(to),
                },
                None => return Err(self.unexpected("a number after `..`")),
            }
        } else if let Some(from) = self.number() {
            if self.rest().starts_with("..") {
                self.pos += 2;
                Value::Range {
                    from: Some(from),
                    to: self.number(),
                }
            } else {
                Value::Number(from)
            }
        } else {
            let word = self.word();
            if word.is_empty() {
                return Err(
                    self.unexpected("a value: a word, a number, a range or a quoted string")
                );
            }
            if is_keyword(word) {
                return Err(self.error(
                    ErrorKind::Unexpected,
                    format!(
                        "`{word}` is a word of the language; as a value it is quoted: \"{word}\""
                    ),
                    start,
                ));
            }
            Value::Text(word.to_string())
        };
        if !matches!(self.peek(), None | Some(')' | ','))
            && !self.peek().is_some_and(char::is_whitespace)
        {
            let end = self
                .rest()
                .find(|c: char| c.is_whitespace() || c == ')' || c == ',');
            let whole = &self.src[start..end.map_or(self.src.len(), |n| self.pos + n)];
            return Err(self
                .error(
                    ErrorKind::Unexpected,
                    "a value that is more than letters, digits and underscores is quoted",
                    start,
                )
                .with_readings(if whole.starts_with('"') {
                    Vec::new()
                } else {
                    vec![print::quoted_text(whole)]
                }));
        }
        match &value {
            Value::Range { .. } if op != Op::Eq => Err(self.error(
                ErrorKind::RangeNeedsEquals,
                "a range `a..b` takes `=`",
                start,
            )),
            Value::Text(_) if op.is_ordering() => Err(self.error(
                ErrorKind::ComparisonNeedsNumber,
                format!("`{}` compares with a number", op.as_str()),
                start,
            )),
            _ => Ok(value),
        }
    }

    fn op(&mut self) -> Option<Op> {
        for text in [">=", "<=", ">", "<", "=", ":", "~"] {
            if self.rest().starts_with(text) {
                self.pos += text.len();
                return Op::parse(text);
            }
        }
        None
    }

    fn at_op(&self) -> bool {
        matches!(self.peek(), Some('>' | '<' | '=' | ':' | '~'))
    }

    /// A name, possibly dotted; the first word is already read.
    fn dotted(&mut self, start: usize) -> Result<&'a str, LanguageError> {
        while self.peek() == Some('.') && self.peek_at(1).is_some_and(is_word_char) {
            self.bump();
            self.word();
        }
        let name = &self.src[start..self.pos];
        if tree::is_name(name) {
            Ok(name)
        } else {
            Err(self.error(
                ErrorKind::Unexpected,
                format!("`{name}` is not a name: letters, digits and underscores, never starting with a digit"),
                start,
            ))
        }
    }

    // ---- the item level ------------------------------------------------

    fn at_sequence_end(&self, level: Level) -> bool {
        match self.peek() {
            None => true,
            Some(')') => level != Level::Root,
            Some(',') => level == Level::Argument,
            _ => false,
        }
    }

    /// Terms joined by and (whitespace, `and`) or by `or`. `None`: nothing
    /// was there.
    fn sequence(&mut self, level: Level) -> Result<Option<Node>, LanguageError> {
        let start = self.pos;
        let mut items: Vec<Item> = Vec::new();
        let mut conns: Vec<Conn> = Vec::new();
        loop {
            self.skip_ws();
            if self.at_sequence_end(level) {
                break;
            }
            if !items.is_empty() {
                if self.keyword("or") {
                    conns.push(Conn::Or);
                } else {
                    self.keyword("and");
                    conns.push(Conn::And);
                }
                self.skip_ws();
                if self.at_sequence_end(level) {
                    return Err(self.unexpected("a term after the connective"));
                }
            } else if self.keyword("or") || self.keyword("and") {
                return Err(self.error(
                    ErrorKind::Unexpected,
                    "a connective joins two terms: there is none before it",
                    start,
                ));
            }
            items.push(self.item()?);
        }
        self.slot_words_outside(&items, &conns)?;
        let nodes: Vec<Node> = items.into_iter().map(|i| i.node).collect();
        combine(nodes, &conns, Node::All, Node::Any).map_err(|(and_first, or_first)| {
            self.error(
                ErrorKind::MixedAndOr,
                "and beside or needs parentheses: which did you mean?",
                start,
            )
            .with_readings(vec![print::print(&and_first), print::print(&or_first)])
        })
    }

    /// `"T" low>=15 high<=45`: whitespace never binds a condition to an
    /// occurrence, so slot words after a phrase are an error that offers
    /// the group and the projections.
    fn slot_words_outside(&self, items: &[Item], conns: &[Conn]) -> Result<(), LanguageError> {
        let is_slot_test =
            |item: &Item| matches!(&item.node, Node::Test { field, .. } if is_slot_word(field));
        let Some(first) = items.iter().position(is_slot_test) else {
            return Ok(());
        };
        let word = match &items[first].node {
            Node::Test { field, .. } => field.as_str(),
            _ => "",
        };
        let err =
            tree::slot_outside_group(word).at(items[first].start, items[first].start + word.len());
        let after_quoted = first > 0 && conns.get(first - 1) == Some(&Conn::And);
        let Some(template) = after_quoted
            .then(|| items[first - 1].quoted.as_deref())
            .flatten()
        else {
            return Err(err);
        };
        let mut tests = Vec::new();
        for (i, item) in items.iter().enumerate().skip(first) {
            match &item.node {
                Node::Test { field, op, value }
                    if is_slot_word(field) && conns.get(i - 1) == Some(&Conn::And) =>
                {
                    tests.push((field.clone(), *op, value.clone()));
                }
                _ => break,
            }
        }
        let mut members = vec![template_test(template)];
        members.extend(tests.iter().map(|(attr, op, value)| Member::Test {
            attr: attr.clone(),
            op: *op,
            value: value.clone(),
        }));
        let grouped = print::print(&Node::Members {
            of: Collection::Lines,
            where_: Box::new(Member::All(members)),
        });
        let projected: Vec<String> = tests
            .iter()
            .map(|(slot, op, value)| print::shorthand(template, slot, *op, value))
            .collect();
        Err(err.with_readings(vec![grouped, projected.join(" ")]))
    }

    fn item(&mut self) -> Result<Item, LanguageError> {
        let start = self.pos;
        let opens_quoted = self.peek() == Some('"');
        let node = self.term()?;
        let quoted = match &node {
            Node::Test {
                field,
                op: Op::Contains,
                value: Value::Text(phrase),
            } if opens_quoted && field == "text" => Some(phrase.clone()),
            Node::Members {
                of: Collection::Lines,
                where_,
            } if opens_quoted => match where_.as_ref() {
                Member::Test {
                    attr,
                    op: Op::Eq,
                    value: Value::Text(template),
                } if attr == "template" => Some(template.clone()),
                _ => None,
            },
            _ => None,
        };
        Ok(Item {
            node,
            start,
            quoted,
        })
    }

    fn term(&mut self) -> Result<Node, LanguageError> {
        let start = self.pos;
        if self.eat('-') || self.keyword("not") {
            self.skip_ws();
            if self.at_end() {
                return Err(self.unexpected("a term to negate"));
            }
            return Ok(Node::Not(Box::new(self.term()?)));
        }
        if self.eat('(') {
            let inner = self.sequence(Level::Paren)?;
            self.expect(')', "`)`")?;
            return inner
                .ok_or_else(|| self.error(ErrorKind::EmptyGroup, "`()` holds nothing", start));
        }
        if self.peek() == Some('"') {
            return self.phrase_or_shorthand();
        }
        if self.peek().is_some_and(|c| c.is_ascii_digit()) && self.number().is_some() {
            return Err(self.error(
                ErrorKind::Unexpected,
                "a number is not a term: compare something with it (ilvl>=84)",
                start,
            ));
        }
        let word = self.word();
        if word.is_empty() {
            return Err(self.unexpected("a term"));
        }
        if self.peek() == Some('(') {
            return self.call(word, start);
        }
        let name = self.dotted(start)?;
        let computed = name == "pseudo" || name.starts_with("pseudo.");
        if !self.at_op() && !computed {
            return Err(self
                .spaced_operator(name, start)
                .unwrap_or_else(|| self.bare_word(name, start, Collection::Lines, false)));
        }
        self.named_test(name, start)
    }

    /// `ilvl >= 84`: whitespace separates terms, so an operator sits against
    /// its name and its value.
    fn spaced_operator(&self, name: &str, start: usize) -> Option<LanguageError> {
        let after = self.rest().trim_start();
        let op = [">=", "<=", ">", "<", "=", ":", "~"]
            .into_iter()
            .find(|op| after.starts_with(op))?;
        Some(self.error(
            ErrorKind::Unexpected,
            format!("whitespace separates terms, so an operator sits against its name and its value: {name}{op}…"),
            start,
        ))
    }

    fn bare_word(&self, word: &str, start: usize, of: Collection, inside: bool) -> LanguageError {
        let template = Member::Test {
            attr: "template".to_string(),
            op: Op::Contains,
            value: Value::Text(word.to_string()),
        };
        let mut readings = Vec::new();
        if !inside {
            readings.push(print::print(&phrase(word)));
            readings.push(print::print(&Node::Members {
                of: Collection::Lines,
                where_: Box::new(template),
            }));
        } else if of == Collection::Lines {
            readings.push(print::print_group(of, &template));
        }
        self.error(
            ErrorKind::BareWord,
            format!("`{word}` alone is never guessed: say what it is"),
            start,
        )
        .with_readings(readings)
    }

    /// `name op value`, with the names the grammar keeps.
    fn named_test(&mut self, name: &str, start: usize) -> Result<Node, LanguageError> {
        if name == "realm" {
            return Err(tree::realm_is_scope().at(start, self.pos));
        }
        if name == "pseudo" || name.starts_with("pseudo.") {
            let value = pseudo(name).map_err(|e| e.at(start, self.pos))?;
            return self.comparison(value, start);
        }
        let op = self.op().ok_or_else(|| self.unexpected("an operator"))?;
        if name == "has" || name == "is" {
            if op != Op::Contains {
                return Err(self.error(
                    ErrorKind::Unexpected,
                    format!("`{name}` takes `:` and a name: {name}:corrupted"),
                    start,
                ));
            }
            let at = self.pos;
            let first = self.word();
            if first.is_empty() {
                return Err(self.unexpected("a name"));
            }
            let what = self.dotted(at)?;
            if name == "is" {
                return Ok(Node::Is(what.to_string()));
            }
            return Ok(Node::Has(what.to_string()));
        }
        let value = self.value(op)?;
        Ok(Node::Test {
            field: name.to_string(),
            op,
            value,
        })
    }

    /// A computed value and its comparison: `= > >= < <=` and a number or
    /// a range.
    fn comparison(&mut self, value: ValueRef, start: usize) -> Result<Node, LanguageError> {
        let shown = print::print_value(&value);
        let Some(op) = self.op() else {
            return Err(self.error(
                ErrorKind::Unexpected,
                format!("`{shown}` is a value: compare it ({shown}>=…) or ask undecided({shown})"),
                start,
            ));
        };
        if matches!(op, Op::Contains | Op::Match) {
            return Err(self.error(
                ErrorKind::ComparisonNeedsNumber,
                format!("`{shown}` is a number: it takes = > >= < <="),
                start,
            ));
        }
        let at = self.pos;
        let rhs = self.value(op)?;
        if matches!(rhs, Value::Text(_)) {
            return Err(self.error(
                ErrorKind::ComparisonNeedsNumber,
                format!("`{shown}` compares with a number"),
                at,
            ));
        }
        Ok(Node::Compare { value, op, rhs })
    }

    /// `"a phrase"`, or the shorthand `"T">=90` / `"T".avg>=20`, which
    /// lowers at once into the line's group.
    fn phrase_or_shorthand(&mut self) -> Result<Node, LanguageError> {
        let start = self.pos;
        let text = self.string()?;
        let named_slot = if self.peek() == Some('.') {
            self.bump();
            Some(self.slot_word()?)
        } else {
            None
        };
        if named_slot.is_none() && !self.at_op() {
            if text.contains('#') {
                // A quoted string with a # in it is a template, never a
                // phrase: alone it means the item carries the line.
                let template = template::unsigned(&text).map_err(|e| e.at(start, self.pos))?;
                return Ok(Node::Members {
                    of: Collection::Lines,
                    where_: Box::new(template_test(&template)),
                });
            }
            if text.is_empty() {
                return Err(self.error(
                    ErrorKind::Unexpected,
                    "an empty phrase finds nothing",
                    start,
                ));
            }
            return Ok(phrase(&text));
        }
        let text = template::unsigned(&text).map_err(|e| e.at(start, self.pos))?;
        let slot = match named_slot {
            Some(slot) => slot,
            None => match slotless(&text) {
                Ok(slot) => slot.to_string(),
                // Typed with its numbers and more than one of them: the
                // numbers are the error, and the comparison is left to the
                // author once the template is right.
                Err(_) if text.bytes().any(|b| b.is_ascii_digit()) => {
                    let group = Node::Members {
                        of: Collection::Lines,
                        where_: Box::new(template_test(&text)),
                    };
                    let readings = vec![
                        print::print(&retyped_node(&group, false)),
                        print::print(&retyped_node(&group, true)),
                    ];
                    return Err(template::check_typed(&text)
                        .err()
                        .unwrap_or_else(|| {
                            LanguageError::new(
                                ErrorKind::TemplateWithNumbers,
                                "a template writes each number as #",
                            )
                        })
                        .with_readings(readings)
                        .at(start, self.pos));
                }
                Err(e) => return Err(e.at(start, self.pos)),
            },
        };
        let Some(op) = self.op() else {
            return Err(self.error(
                ErrorKind::Unexpected,
                format!("`\"…\".{slot}` is a value: compare it (\"…\".{slot}>=…)"),
                start,
            ));
        };
        if matches!(op, Op::Contains | Op::Match) {
            return Err(self.error(
                ErrorKind::ComparisonNeedsNumber,
                "a line's number takes = > >= < <=",
                start,
            ));
        }
        let at = self.pos;
        let value = self.value(op)?;
        if matches!(value, Value::Text(_)) {
            return Err(self.error(
                ErrorKind::ComparisonNeedsNumber,
                "a line's number compares with a number",
                at,
            ));
        }
        Ok(Node::Members {
            of: Collection::Lines,
            where_: Box::new(Member::All(vec![
                template_test(&text),
                Member::Test {
                    attr: slot,
                    op,
                    value,
                },
            ])),
        })
    }

    fn slot_word(&mut self) -> Result<String, LanguageError> {
        let start = self.pos;
        let word = self.word();
        if is_slot_word(word) {
            Ok(word.to_string())
        } else {
            Err(self.error(
                ErrorKind::SlotUnknown,
                format!("`{word}` is not a slot word: low, high, avg, arg1, arg2 …"),
                start,
            ))
        }
    }

    fn call(&mut self, name: &str, start: usize) -> Result<Node, LanguageError> {
        match name {
            "true" | "false" => {
                self.bump();
                self.expect(')', "`)`: the constants are true() and false()")?;
                Ok(Node::Const(name == "true"))
            }
            "line" | "linked" => {
                let of = if name == "line" {
                    Collection::Lines
                } else {
                    Collection::Links
                };
                let where_ = self.group(of)?;
                if of == Collection::Lines && self.peek() == Some('.') {
                    self.bump();
                    let slot = self.slot_word()?;
                    return self.projected_comparison(where_, slot, start);
                }
                Ok(Node::Members {
                    of,
                    where_: Box::new(where_),
                })
            }
            "holds" => self.holds(start),
            "undecided" => {
                self.bump();
                let probe = self.probe()?;
                self.skip_ws();
                self.expect(')', "`)`")?;
                Ok(Node::Undecided(probe))
            }
            "sum" => {
                let value = self.sum()?;
                self.comparison(value, start)
            }
            other => Err(self.unknown_call(other, start)),
        }
    }

    fn unknown_call(&self, name: &str, start: usize) -> LanguageError {
        let near: Vec<&str> = CALLS
            .iter()
            .copied()
            .filter(|c| distance(c, name) <= 2)
            .collect();
        let offered = if near.is_empty() {
            CALLS.to_vec()
        } else {
            near
        };
        LanguageError::new(
            ErrorKind::UnknownCall,
            format!("`{name}(` is no call of the language"),
        )
        .with_readings(offered.iter().map(|c| format!("{c}(")).collect())
        .at(start, self.pos)
    }

    /// `line(P).slot op value` lowers to `line(P slot op value)`: a
    /// comparison on a projection means what it means inside the group.
    fn projected_comparison(
        &mut self,
        where_: Member,
        slot: String,
        start: usize,
    ) -> Result<Node, LanguageError> {
        let Some(op) = self.op() else {
            return Err(self.error(
                ErrorKind::Unexpected,
                format!("`line(…).{slot}` is a value: compare it (line(…).{slot}>=…)"),
                start,
            ));
        };
        if matches!(op, Op::Contains | Op::Match) {
            return Err(self.error(
                ErrorKind::ComparisonNeedsNumber,
                "a line's number takes = > >= < <=",
                start,
            ));
        }
        let at = self.pos;
        let value = self.value(op)?;
        if matches!(value, Value::Text(_)) {
            return Err(self.error(
                ErrorKind::ComparisonNeedsNumber,
                "a line's number compares with a number",
                at,
            ));
        }
        let test = Member::Test {
            attr: slot,
            op,
            value,
        };
        let where_ = match where_ {
            Member::All(mut members) => {
                members.push(test);
                Member::All(members)
            }
            single => Member::All(vec![single, test]),
        };
        Ok(Node::Members {
            of: Collection::Lines,
            where_: Box::new(where_),
        })
    }

    fn holds(&mut self, start: usize) -> Result<Node, LanguageError> {
        self.bump();
        let mut of = Vec::new();
        loop {
            let at = self.pos;
            match self.sequence(Level::Argument)? {
                Some(node) => of.push(node),
                None => {
                    return Err(self.error(
                        ErrorKind::EmptyGroup,
                        "`holds` counts queries: one is missing here",
                        at,
                    ));
                }
            }
            if !self.eat(',') {
                break;
            }
        }
        self.expect(')', "`,` or `)`")?;
        let missing = || {
            LanguageError::new(
                ErrorKind::HoldsNeedsBound,
                "`holds( … )` needs how many: holds(…)>=2, holds(…)=1, holds(…)=2..3",
            )
        };
        let Some(op) = self.op() else {
            return Err(missing().at(start, self.pos));
        };
        let at = self.pos;
        let bound = self.value(op)?;
        let whole = |n: Number| match n {
            Number::Int(i) => u32::try_from(i).ok(),
            Number::Float(_) => None,
        };
        let not_a_count = || {
            LanguageError::new(
                ErrorKind::HoldsNeedsBound,
                "`holds` counts queries: its bound is a whole number, zero or more",
            )
            .at(at, self.pos)
        };
        let (min, max) = match (op, bound) {
            (Op::Eq, Value::Range { from, to }) => (
                from.map(|n| whole(n).ok_or_else(not_a_count)).transpose()?,
                to.map(|n| whole(n).ok_or_else(not_a_count)).transpose()?,
            ),
            (op, Value::Number(n)) => {
                let n = whole(n).ok_or_else(not_a_count)?;
                match op {
                    Op::Eq => (Some(n), Some(n)),
                    Op::Ge => (Some(n), None),
                    Op::Le => (None, Some(n)),
                    Op::Gt => (Some(n.checked_add(1).ok_or_else(not_a_count)?), None),
                    Op::Lt => (None, Some(n.checked_sub(1).ok_or_else(not_a_count)?)),
                    Op::Contains | Op::Match => return Err(missing().at(start, self.pos)),
                }
            }
            _ => return Err(missing().at(start, self.pos)),
        };
        Ok(Node::Holds { of, min, max })
    }

    /// Inside `undecided( … )`: a thing, when what is there is a value and
    /// nothing more; otherwise a term.
    fn probe(&mut self) -> Result<Probe, LanguageError> {
        self.skip_ws();
        let start = self.pos;
        if let Ok(value) = self.value_ref(ValueContext::Thing) {
            self.skip_ws();
            if self.peek() == Some(')') {
                return Ok(Probe::Thing(value));
            }
        }
        self.pos = start;
        match self.sequence(Level::Paren)? {
            Some(term) => Ok(Probe::Term(Box::new(term))),
            None => Err(self.error(
                ErrorKind::EmptyGroup,
                "`undecided()` asks about nothing: a thing (undecided(class)) or a term",
                start,
            )),
        }
    }

    // ---- values ----------------------------------------------------------

    fn value_ref(&mut self, context: ValueContext) -> Result<ValueRef, LanguageError> {
        let start = self.pos;
        if self.peek() == Some('"') {
            let text = self.string()?;
            let text = template::unsigned(&text).map_err(|e| e.at(start, self.pos))?;
            let lines = Box::new(template_test(&text));
            return self.projection(lines, Some(&text), context, start);
        }
        let word = self.word();
        if word.is_empty() {
            return Err(self.unexpected("a value"));
        }
        if self.peek() == Some('(') {
            return match word {
                "sum" => self.sum(),
                "line" => {
                    let where_ = self.group(Collection::Lines)?;
                    let quoted = template::selected(&where_).map(str::to_string);
                    self.projection(Box::new(where_), quoted.as_deref(), context, start)
                }
                other => Err(self.error(
                    ErrorKind::Unexpected,
                    format!("`{other}( … )` has no value: a value is a field, pseudo.<name>, line( … ).<slot> or sum( … )"),
                    start,
                )),
            };
        }
        let name = self.dotted(start)?;
        if name == "realm" {
            return Err(tree::realm_is_scope().at(start, self.pos));
        }
        if name == "pseudo" || name.starts_with("pseudo.") {
            return pseudo(name).map_err(|e| e.at(start, self.pos));
        }
        Ok(ValueRef::Field(name.to_string()))
    }

    /// `… .slot` after a quoted template or a line's group. With no slot
    /// named, a standalone value takes the template's one slot; under
    /// `undecided( … )` it is no value at all, and the caller reads a term.
    fn projection(
        &mut self,
        lines: Box<Member>,
        quoted: Option<&str>,
        context: ValueContext,
        start: usize,
    ) -> Result<ValueRef, LanguageError> {
        if self.peek() == Some('.') {
            self.bump();
            let slot = self.slot_word()?;
            return Ok(ValueRef::Projection { lines, slot });
        }
        if context == ValueContext::Thing {
            return Err(self.error(ErrorKind::Unexpected, "not a value", start));
        }
        let slot = slot_of(quoted).map_err(|e| e.at(start, self.pos))?;
        Ok(ValueRef::Projection { lines, slot })
    }

    /// `sum("T")` or `sum(line(P).<slot>)`; `sum` is already read.
    fn sum(&mut self) -> Result<ValueRef, LanguageError> {
        let start = self.pos;
        self.bump();
        self.skip_ws();
        let (lines, quoted) = if self.peek() == Some('"') {
            let text = self.string()?;
            let text = template::unsigned(&text).map_err(|e| e.at(start, self.pos))?;
            (template_test(&text), Some(text))
        } else {
            let at = self.pos;
            let word = self.word();
            if word != "line" || self.peek() != Some('(') {
                return Err(self.error(
                    ErrorKind::Unexpected,
                    "`sum` adds up a line: sum(\"T\") or sum(line( … ).<slot>)",
                    at,
                ));
            }
            let where_ = self.group(Collection::Lines)?;
            let quoted = template::selected(&where_).map(str::to_string);
            (where_, quoted)
        };
        let slot = if self.peek() == Some('.') {
            self.bump();
            self.slot_word()?
        } else {
            slot_of(quoted.as_deref()).map_err(|e| e.at(start, self.pos))?
        };
        self.skip_ws();
        self.expect(')', "`)`")?;
        Ok(ValueRef::Sum {
            lines: Box::new(lines),
            slot,
        })
    }

    // ---- member groups ---------------------------------------------------

    /// `( … )` of a `line` or a `linked`; the `(` is next.
    fn group(&mut self, of: Collection) -> Result<Member, LanguageError> {
        let start = self.pos;
        self.bump();
        let inner = self.member_sequence(of)?;
        self.expect(')', "`)`")?;
        inner.ok_or_else(|| {
            self.error(
                ErrorKind::EmptyGroup,
                format!("`{}()` says nothing about its member", of.call()),
                start,
            )
        })
    }

    fn member_sequence(&mut self, of: Collection) -> Result<Option<Member>, LanguageError> {
        let start = self.pos;
        let mut members = Vec::new();
        let mut conns = Vec::new();
        loop {
            self.skip_ws();
            if matches!(self.peek(), None | Some(')')) {
                break;
            }
            if !members.is_empty() {
                if self.keyword("or") {
                    conns.push(Conn::Or);
                } else {
                    self.keyword("and");
                    conns.push(Conn::And);
                }
                self.skip_ws();
                if matches!(self.peek(), None | Some(')')) {
                    return Err(self.unexpected("a condition after the connective"));
                }
            } else if self.keyword("or") || self.keyword("and") {
                return Err(self.error(
                    ErrorKind::Unexpected,
                    "a connective joins two conditions: there is none before it",
                    start,
                ));
            }
            members.push(self.member_term(of)?);
        }
        combine(members, &conns, Member::All, Member::Any).map_err(|(and_first, or_first)| {
            self.error(
                ErrorKind::MixedAndOr,
                "and beside or needs parentheses: which did you mean?",
                start,
            )
            .with_readings(vec![
                print::print_group(of, &and_first),
                print::print_group(of, &or_first),
            ])
        })
    }

    fn member_term(&mut self, of: Collection) -> Result<Member, LanguageError> {
        let start = self.pos;
        if self.eat('-') || self.keyword("not") {
            self.skip_ws();
            return Ok(Member::Not(Box::new(self.member_term(of)?)));
        }
        if self.eat('(') {
            let inner = self.member_sequence(of)?;
            self.expect(')', "`)`")?;
            return inner
                .ok_or_else(|| self.error(ErrorKind::EmptyGroup, "`()` holds nothing", start));
        }
        if self.peek() == Some('"') {
            let text = self.string()?;
            if of != Collection::Lines {
                return Err(self.error(
                    ErrorKind::NotInsideGroup,
                    "a quoted template selects a line: a link group has none",
                    start,
                ));
            }
            let text = template::unsigned(&text).map_err(|e| e.at(start, self.pos))?;
            if self.at_op() || self.peek() == Some('.') {
                let slot = slotless(&text).unwrap_or("arg1");
                return Err(self
                    .error(
                        ErrorKind::Unexpected,
                        "inside the group a number is named after the template",
                        start,
                    )
                    .with_readings(vec![format!("{} {slot}>=…", print::quoted_text(&text))]));
            }
            return Ok(template_test(&text));
        }
        let word = self.word();
        if word.is_empty() {
            return Err(self.unexpected("a condition"));
        }
        if self.peek() == Some('(') {
            return match word {
                "true" | "false" => {
                    self.bump();
                    self.expect(')', "`)`: the constants are true() and false()")?;
                    Ok(Member::Const(word == "true"))
                }
                call if CALLS.contains(&call) => Err(self.error(
                    ErrorKind::NotInsideGroup,
                    format!("`{call}( … )` is asked of the item, never inside a member's group"),
                    start,
                )),
                other => Err(self.unknown_call(other, start)),
            };
        }
        let name = self.dotted(start)?;
        if !self.at_op() {
            return Err(self
                .spaced_operator(name, start)
                .unwrap_or_else(|| self.bare_word(name, start, of, true)));
        }
        if name == "has" || name == "realm" || name == "pseudo" || name.starts_with("pseudo.") {
            return Err(self.error(
                ErrorKind::NotInsideGroup,
                format!("`{name}` is asked of the item, never inside a member's group"),
                start,
            ));
        }
        let op = self.op().ok_or_else(|| self.unexpected("an operator"))?;
        if name == "is" {
            if op != Op::Contains {
                return Err(self.error(
                    ErrorKind::Unexpected,
                    "`is` takes `:` and a name: is:fractured",
                    start,
                ));
            }
            let at = self.pos;
            let first = self.word();
            if first.is_empty() {
                return Err(self.unexpected("a name"));
            }
            return Ok(Member::Is(self.dotted(at)?.to_string()));
        }
        let value = match self.value(op)? {
            // `template="T"` is the bare "T" spelled out: the same spelling rule.
            Value::Text(template) if name == "template" && op == Op::Eq => {
                Value::Text(template::unsigned(&template).map_err(|e| e.at(start, self.pos))?)
            }
            other => other,
        };
        Ok(Member::Test {
            attr: name.to_string(),
            op,
            value,
        })
    }
}

// ---- helpers -------------------------------------------------------------

fn phrase(text: &str) -> Node {
    Node::Test {
        field: "text".to_string(),
        op: Op::Contains,
        value: Value::Text(text.to_string()),
    }
}

fn template_test(template: &str) -> Member {
    Member::Test {
        attr: "template".to_string(),
        op: Op::Eq,
        value: Value::Text(template.to_string()),
    }
}

/// The slot a slotless comparison means, by the template's own numbers. A
/// template typed with its numbers is judged by the template it stands
/// for, so that the error the author sees is the one about the numbers.
fn slotless(template: &str) -> Result<&'static str, LanguageError> {
    if template.bytes().any(|b| b.is_ascii_digit()) {
        template::default_slot(&template::typed(template).0)
    } else {
        template::default_slot(template)
    }
}

fn slot_of(quoted: Option<&str>) -> Result<String, LanguageError> {
    match quoted {
        Some(template) => slotless(template).map(str::to_string),
        None => Err(LanguageError::new(
            ErrorKind::SlotMissing,
            "name the number: line( … ).arg1, or .low .high .avg on a ranged line",
        )),
    }
}

/// `pseudo.<name>` or `pseudo.<name>.<slot>`.
fn pseudo(dotted: &str) -> Result<ValueRef, LanguageError> {
    let mut parts = dotted.split('.').skip(1);
    let bad = || {
        LanguageError::new(
            ErrorKind::Unexpected,
            format!(
                "`{dotted}` is not a computed value: pseudo.<name>, and a ranged one takes a slot word last (pseudo.<name>.avg)"
            ),
        )
    };
    let name = parts.next().ok_or_else(bad)?;
    let slot = parts.next();
    if parts.next().is_some() || slot.is_some_and(|s| !is_slot_word(s)) {
        return Err(bad());
    }
    Ok(ValueRef::Pseudo {
        name: name.to_string(),
        slot: slot.map(str::to_string),
    })
}

/// Terms and their connectives into one node; a mix is the error, carried
/// as its two readings — and binding tighter, then or binding tighter.
fn combine<T: Clone>(
    terms: Vec<T>,
    conns: &[Conn],
    all: fn(Vec<T>) -> T,
    any: fn(Vec<T>) -> T,
) -> Result<Option<T>, (T, T)> {
    let group = |mut run: Vec<T>, make: fn(Vec<T>) -> T| match run.len() {
        1 => run.remove(0),
        _ => make(run),
    };
    if terms.is_empty() {
        return Ok(None);
    }
    if conns.iter().all(|c| *c == Conn::And) {
        return Ok(Some(group(terms, all)));
    }
    if conns.iter().all(|c| *c == Conn::Or) {
        return Ok(Some(group(terms, any)));
    }
    let split = |at: Conn, inner: fn(Vec<T>) -> T, outer: fn(Vec<T>) -> T| {
        let mut runs: Vec<Vec<T>> = vec![Vec::new()];
        for (i, term) in terms.iter().enumerate() {
            if i > 0 && conns[i - 1] == at {
                runs.push(Vec::new());
            }
            if let Some(run) = runs.last_mut() {
                run.push(term.clone());
            }
        }
        group(
            runs.into_iter().map(|run| group(run, inner)).collect(),
            outer,
        )
    };
    Err((split(Conn::Or, all, any), split(Conn::And, any, all)))
}

/// Edit distance, for offering a near name.
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { previous } else { previous + 1 };
            previous = row[j + 1];
            row[j + 1] = cost.min(row[j] + 1).min(previous + 1);
        }
    }
    row[b.len()]
}

// ---- the two readings of a template typed with its numbers ---------------

/// The tree with every typed template replaced by the template it stands
/// for; with `that_value`, each number it carried becomes an `argN=` beside
/// it.
fn retyped_node(node: &Node, that_value: bool) -> Node {
    let again = |n: &Node| retyped_node(n, that_value);
    match node {
        Node::All(c) => Node::All(c.iter().map(again).collect()),
        Node::Any(c) => Node::Any(c.iter().map(again).collect()),
        Node::Not(inner) => Node::Not(Box::new(again(inner))),
        Node::Holds { of, min, max } => Node::Holds {
            of: of.iter().map(again).collect(),
            min: *min,
            max: *max,
        },
        Node::Undecided(Probe::Term(term)) => Node::Undecided(Probe::Term(Box::new(again(term)))),
        Node::Undecided(Probe::Thing(value)) => {
            Node::Undecided(Probe::Thing(retyped_value(value, that_value)))
        }
        Node::Members {
            of: Collection::Lines,
            where_,
        } => Node::Members {
            of: Collection::Lines,
            where_: Box::new(retyped_group(where_, that_value)),
        },
        Node::Compare { value, op, rhs } => Node::Compare {
            value: retyped_value(value, that_value),
            op: *op,
            rhs: rhs.clone(),
        },
        other => other.clone(),
    }
}

fn retyped_value(value: &ValueRef, that_value: bool) -> ValueRef {
    match value {
        ValueRef::Sum { lines, slot } => ValueRef::Sum {
            lines: Box::new(retyped_group(lines, that_value)),
            slot: slot.clone(),
        },
        ValueRef::Projection { lines, slot } => ValueRef::Projection {
            lines: Box::new(retyped_group(lines, that_value)),
            slot: slot.clone(),
        },
        other => other.clone(),
    }
}

fn retyped_group(where_: &Member, that_value: bool) -> Member {
    let Some(quoted) = template::selected(where_) else {
        return where_.clone();
    };
    if !quoted.bytes().any(|b| b.is_ascii_digit()) {
        return where_.clone();
    }
    let (hashed, numbers) = template::typed(quoted);
    let mut members = match where_ {
        Member::All(members) => members.clone(),
        single => vec![single.clone()],
    };
    let Some(at) = members
        .iter()
        .position(|m| matches!(m, Member::Test { attr, op: Op::Eq, .. } if attr == "template"))
    else {
        return where_.clone();
    };
    members[at] = template_test(&hashed);
    if that_value {
        let pins = numbers.iter().enumerate().map(|(i, n)| Member::Test {
            attr: format!("arg{}", i + 1),
            op: Op::Eq,
            value: Value::Number(Number::from_f64(*n)),
        });
        members.splice(at + 1..at + 1, pins);
    }
    match members.len() {
        1 => members.remove(0),
        _ => Member::All(members),
    }
}
