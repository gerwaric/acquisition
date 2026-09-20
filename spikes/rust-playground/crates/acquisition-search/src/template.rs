//! What a quoted template says about its own numbers (C90, C92; the
//! reference, *Slots*). A template is the displayed English with each
//! number replaced by `#`, so its slots are a property of the string and
//! every check here needs no corpus: validity never depends on one.

use crate::error::{ErrorKind, LanguageError};
use crate::tree::{Member, Op, Value, arg_index};

/// The quoted template a line group selects, when it selects exactly one
/// at its own level: the group is `"T"` alone, or an and whose members
/// include one `"T"`. A template inside an or, or under a not, selects no
/// single line and is left to the evaluator.
pub(crate) fn selected(where_: &Member) -> Option<&str> {
    fn quoted(member: &Member) -> Option<&str> {
        match member {
            Member::Test {
                attr,
                op: Op::Eq,
                value: Value::Text(t),
            } if attr == "template" => Some(t),
            _ => None,
        }
    }
    match where_ {
        Member::All(children) => {
            let mut found = children.iter().filter_map(quoted);
            match (found.next(), found.next()) {
                (Some(t), None) => Some(t),
                _ => None,
            }
        }
        other => quoted(other),
    }
}

/// How many numbers the template has, and which two are its ranged pair:
/// a template with exactly one `# to #` is ranged (owner, 2026-09-19).
pub(crate) struct Slots {
    pub count: usize,
    /// 1-based positions of `low` and `high`.
    pub ranged: Option<(usize, usize)>,
}

pub(crate) fn slots(template: &str) -> Slots {
    let count = template.matches('#').count();
    let mut pairs = Vec::new();
    let mut from = 0;
    while let Some(at) = template[from..].find("# to #") {
        let start = from + at;
        pairs.push(template[..start].matches('#').count() + 1);
        // `# to # to #` is two pairs sharing a number: step past one `#`.
        from = start + "# to ".len();
    }
    let ranged = match pairs.as_slice() {
        [low] => Some((*low, *low + 1)),
        _ => None,
    };
    Slots { count, ranged }
}

/// The slot words this template takes, as an error lists them.
pub(crate) fn slot_words(template: &str) -> Vec<String> {
    let s = slots(template);
    let mut words: Vec<String> = Vec::new();
    if s.ranged.is_some() {
        words.extend(["low", "high", "avg"].map(String::from));
    }
    words.extend((1..=s.count).map(|n| format!("arg{n}")));
    words
}

/// The slot a comparison, a sort or a sum means when it names none (C92):
/// one number is `arg1`; several is the error that lists them; none is an
/// error. Decided before the shorthand lowers.
pub(crate) fn default_slot(template: &str) -> Result<&'static str, LanguageError> {
    match slots(template).count {
        1 => Ok("arg1"),
        0 => Err(LanguageError::new(
            ErrorKind::SlotMissing,
            format!("\"{template}\" has no number to compare: a template writes each number as #"),
        )),
        n => Err(LanguageError::new(
            ErrorKind::SlotMissing,
            format!(
                "\"{template}\" has {n} numbers, so say which: {}",
                slot_words(template).join(", ")
            ),
        )),
    }
}

/// A named slot against the template's own numbers.
pub(crate) fn check_slot(template: &str, slot: &str) -> Result<(), LanguageError> {
    let s = slots(template);
    let known = match slot {
        "low" | "high" | "avg" => s.ranged.is_some(),
        other => arg_index(other).is_some_and(|n| n <= s.count),
    };
    if known {
        return Ok(());
    }
    let words = slot_words(template);
    let message = if words.is_empty() {
        format!("\"{template}\" has no number, so `{slot}` names nothing")
    } else if matches!(slot, "low" | "high" | "avg") {
        format!(
            "`{slot}` is for a ranged line, a template with exactly one `# to #`; \"{template}\" takes: {}",
            words.join(", ")
        )
    } else {
        format!("\"{template}\" takes: {}", words.join(", "))
    };
    Err(LanguageError::new(ErrorKind::SlotUnknown, message))
}

/// Where a sign sits directly before a `#`: `+# to maximum Life`, as the
/// game and the trade site write it. A `+` or `-` straight after a `#`, a
/// digit or a `)` is a range dash — `(#-#)` — never a sign.
fn signs(template: &str) -> Vec<(usize, char)> {
    let chars: Vec<(usize, char)> = template.char_indices().collect();
    let mut found = Vec::new();
    for (i, &(at, c)) in chars.iter().enumerate() {
        let before_hash = chars.get(i + 1).is_some_and(|&(_, next)| next == '#');
        let dash = i > 0 && matches!(chars[i - 1].1, '#' | ')' | '0'..='9');
        if (c == '+' || c == '-') && before_hash && !dash {
            found.push((at, c));
        }
    }
    found
}

/// A quoted template as the tree holds it (C90: the sign is carried in the
/// number, so a line's identity has none). A `+` before a `#` is spelling
/// and is dropped — the trade site's `+# to maximum Life` names the line
/// `# to maximum Life`. A `-` there is not spelling: it says the value is
/// negative, which a comparison says, so it is the error that offers one.
pub(crate) fn unsigned(template: &str) -> Result<String, LanguageError> {
    let signs = signs(template);
    let bare: String = template
        .char_indices()
        .filter(|(at, _)| !signs.iter().any(|(s, _)| s == at))
        .map(|(_, c)| c)
        .collect();
    let negative: Vec<String> = signs
        .iter()
        .filter(|(_, c)| *c == '-')
        .map(|(at, _)| format!("arg{}<0", template[..*at].matches('#').count() + 1))
        .collect();
    if negative.is_empty() {
        return Ok(bare);
    }
    let quoted = crate::print::quoted_text(&bare);
    Err(LanguageError::new(
        ErrorKind::SignedTemplate,
        format!("the sign is the number's, never the template's: \"{template}\" is the line {quoted} with a negative value"),
    )
    .with_readings(vec![format!("line({quoted} {})", negative.join(" "))]))
}

/// What a tree may hold: a template with no sign before a `#`.
pub(crate) fn check_unsigned(template: &str) -> Result<(), LanguageError> {
    if signs(template).is_empty() {
        return Ok(());
    }
    Err(LanguageError::new(
        ErrorKind::Tree,
        format!("\"{template}\": a tree holds a template without the sign before its #"),
    ))
}

/// A template typed with its numbers (`"+92 to maximum Life"`): no real
/// template carries a digit, so this is the author showing a line as the
/// game displayed it. The error's two readings are the caller's to print.
pub(crate) fn check_typed(template: &str) -> Result<(), LanguageError> {
    if template.bytes().any(|b| b.is_ascii_digit()) {
        let (hashed, _) = typed(template);
        return Err(LanguageError::new(
            ErrorKind::TemplateWithNumbers,
            format!("\"{template}\" is typed with its numbers; its template is \"{hashed}\""),
        ));
    }
    Ok(())
}

/// The template of a displayed string and the numbers taken out of it, in
/// order, the sign carried in the number (C90). A sign straight after a
/// digit or a `#` is a range dash, never a sign: `59-88` is `#-#`.
pub(crate) fn typed(text: &str) -> (String, Vec<f64>) {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut numbers = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let after_number = out.ends_with('#') || (i > 0 && chars[i - 1].is_ascii_digit());
        let signed = (c == '+' || c == '-')
            && !after_number
            && chars.get(i + 1).is_some_and(char::is_ascii_digit);
        if c.is_ascii_digit() || signed {
            let start = i;
            i += 1;
            while chars.get(i).is_some_and(char::is_ascii_digit) {
                i += 1;
            }
            if chars.get(i) == Some(&'.') && chars.get(i + 1).is_some_and(char::is_ascii_digit) {
                i += 1;
                while chars.get(i).is_some_and(char::is_ascii_digit) {
                    i += 1;
                }
            }
            let literal: String = chars[start..i].iter().collect();
            numbers.push(literal.parse().unwrap_or(0.0));
            out.push('#');
        } else {
            out.push(c);
            i += 1;
        }
    }
    (out, numbers)
}
