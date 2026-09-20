//! The canonical printer (C104, C91): every valid tree has one text, and
//! parsing that text gives the tree back.
//!
//! Invariant 1 of the surface: canonicalisation lowers shorthand and
//! normalises spelling; it never reorders, flattens, merges, deduplicates
//! or simplifies. The printer has no shorthand to lower — a tree holds
//! none — and adds parentheses only where the nesting needs them, so
//! `(a b) c` and `a b c` stay two texts for two trees.
//!
//! Spelling, as printed: and is whitespace, not is `-`; a phrase is
//! `"…"`, unless it holds a `#`, when it is `text:"…#…"` — a bare quoted
//! string with a `#` is a template; inside `line( … )` the template selector is the bare `"T"`; a
//! text value is bare when it is one plain word and quoted otherwise; a
//! pattern is always quoted; a `holds` bound is `>=n`, `<=n`, `=n` or
//! `=a..b`.

use crate::tree::{
    Collection, Member, Node, Number, Op, Probe, Value, ValueRef, is_keyword, is_word_char,
};

/// The canonical text of a query. The empty query prints as nothing.
pub fn print(root: &Node) -> String {
    let mut out = String::new();
    sequence(root, &mut out);
    out
}

/// The canonical text of a value: what `--sort` and `--sum` take.
pub fn print_value(value: &ValueRef) -> String {
    let mut out = String::new();
    value_ref(value, &mut out);
    out
}

/// A member group's inside, as an error's reading shows it.
pub(crate) fn print_group(of: Collection, where_: &Member) -> String {
    let mut out = String::new();
    member_sequence(of, where_, &mut out);
    out
}

/// `"T".slot op value` — the shorthand, as an error's reading offers it.
pub(crate) fn shorthand(template: &str, slot: &str, op: Op, rhs: &Value) -> String {
    let mut out = String::new();
    quoted(template, &mut out);
    out.push('.');
    out.push_str(slot);
    out.push_str(op.as_str());
    value(rhs, op, &mut out);
    out
}

pub(crate) fn quoted_text(text: &str) -> String {
    let mut out = String::new();
    quoted(text, &mut out);
    out
}

/// A node where a whole sequence is allowed — the root, a `holds`
/// argument, inside `undecided( … )` — so its own and / or needs no
/// parentheses.
fn sequence(node: &Node, out: &mut String) {
    match node {
        Node::All(children) => joined(children, " ", out, term),
        Node::Any(children) => joined(children, " or ", out, term),
        other => term(other, out),
    }
}

fn joined<T>(items: &[T], sep: &str, out: &mut String, each: fn(&T, &mut String)) {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        each(item, out);
    }
}

/// A node as one term: a nested and / or is parenthesised.
fn term(node: &Node, out: &mut String) {
    match node {
        Node::All(_) | Node::Any(_) => {
            out.push('(');
            sequence(node, out);
            out.push(')');
        }
        Node::Not(inner) => {
            out.push('-');
            term(inner, out);
        }
        Node::Holds { of, min, max } => {
            out.push_str("holds(");
            joined(of, ", ", out, sequence);
            out.push(')');
            match (min, max) {
                (Some(a), Some(b)) if a == b => out.push_str(&format!("={a}")),
                (Some(a), Some(b)) => out.push_str(&format!("={a}..{b}")),
                (Some(a), None) => out.push_str(&format!(">={a}")),
                (None, Some(b)) => out.push_str(&format!("<={b}")),
                (None, None) => {}
            }
        }
        Node::Undecided(probe) => {
            out.push_str("undecided(");
            match probe {
                Probe::Thing(value) => value_ref(value, out),
                Probe::Term(inner) => sequence(inner, out),
            }
            out.push(')');
        }
        Node::Const(true) => out.push_str("true()"),
        Node::Const(false) => out.push_str("false()"),
        Node::Test {
            field,
            op: Op::Contains,
            value: Value::Text(phrase),
        } if field == "text" && !phrase.contains('#') => quoted(phrase, out),
        Node::Test { field, op, value } => test(field, *op, value, out),
        Node::Has(name) => {
            out.push_str("has:");
            out.push_str(name);
        }
        Node::Is(name) => {
            out.push_str("is:");
            out.push_str(name);
        }
        Node::Members { of, where_ } => members(*of, where_, out),
        Node::Compare { value, op, rhs } => {
            value_ref(value, out);
            out.push_str(op.as_str());
            self::value(rhs, *op, out);
        }
    }
}

fn members(of: Collection, where_: &Member, out: &mut String) {
    out.push_str(of.call());
    out.push('(');
    member_sequence(of, where_, out);
    out.push(')');
}

fn member_sequence(of: Collection, member: &Member, out: &mut String) {
    match member {
        Member::All(children) => {
            for (i, child) in children.iter().enumerate() {
                if i > 0 {
                    out.push(' ');
                }
                member_term(of, child, out);
            }
        }
        Member::Any(children) => {
            for (i, child) in children.iter().enumerate() {
                if i > 0 {
                    out.push_str(" or ");
                }
                member_term(of, child, out);
            }
        }
        other => member_term(of, other, out),
    }
}

fn member_term(of: Collection, member: &Member, out: &mut String) {
    match member {
        Member::All(_) | Member::Any(_) => {
            out.push('(');
            member_sequence(of, member, out);
            out.push(')');
        }
        Member::Not(inner) => {
            out.push('-');
            member_term(of, inner, out);
        }
        Member::Const(true) => out.push_str("true()"),
        Member::Const(false) => out.push_str("false()"),
        Member::Is(name) => {
            out.push_str("is:");
            out.push_str(name);
        }
        // Inside a line's group the template selector is the bare "T".
        Member::Test {
            attr,
            op: Op::Eq,
            value: Value::Text(template),
        } if of == Collection::Lines && attr == "template" => quoted(template, out),
        Member::Test { attr, op, value } => test(attr, *op, value, out),
    }
}

fn test(name: &str, op: Op, rhs: &Value, out: &mut String) {
    out.push_str(name);
    out.push_str(op.as_str());
    value(rhs, op, out);
}

fn value(rhs: &Value, op: Op, out: &mut String) {
    match rhs {
        Value::Text(text) if op != Op::Match && is_plain_word(text) => out.push_str(text),
        Value::Text(text) => quoted(text, out),
        Value::Number(n) => number(*n, out),
        Value::Range { from, to } => {
            if let Some(n) = from {
                number(*n, out);
            }
            out.push_str("..");
            if let Some(n) = to {
                number(*n, out);
            }
        }
    }
}

fn number(n: Number, out: &mut String) {
    match n {
        Number::Int(i) => out.push_str(&i.to_string()),
        // Rust prints the shortest text that reads back as the same f64,
        // never an exponent.
        Number::Float(f) => out.push_str(&f.to_string()),
    }
}

/// A text value that reads back as itself unquoted: word characters only,
/// not a keyword, and nothing the lexer would take for a number.
fn is_plain_word(text: &str) -> bool {
    !text.is_empty()
        && text.chars().all(is_word_char)
        && !is_keyword(text)
        && !text.bytes().all(|b| b.is_ascii_digit())
}

/// The only escapes are `\"`, `\\` and `\n` (the reference, *Strings*).
fn quoted(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out.push('"');
}

fn value_ref(value: &ValueRef, out: &mut String) {
    match value {
        ValueRef::Field(name) => out.push_str(name),
        ValueRef::Pseudo { name, slot } => {
            out.push_str("pseudo.");
            out.push_str(name);
            if let Some(slot) = slot {
                out.push('.');
                out.push_str(slot);
            }
        }
        ValueRef::Sum { lines, slot } => {
            out.push_str("sum(");
            members(Collection::Lines, lines, out);
            out.push('.');
            out.push_str(slot);
            out.push(')');
        }
        ValueRef::Projection { lines, slot } => {
            members(Collection::Lines, lines, out);
            out.push('.');
            out.push_str(slot);
        }
    }
}
