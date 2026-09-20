//! `--describe` (C97: the closed lists are printed in the help): the
//! language as this build knows it — fields, what a line has, operators,
//! closed value sets, slots, computed values, what is not built, and the
//! limits it states (C102). Printed from the binder's own tables, so the
//! help cannot say a name the binder refuses.

use serde::Serialize;

use crate::bind::{self, FIELDS, ITEM_FLAGS, Kind, LINE_FLAGS, NOT_BUILT, NotBuilt, SOURCES};
use crate::error::{ErrorKind, LanguageError};

#[derive(Debug, Clone, Serialize)]
pub struct Describe {
    pub fields: Vec<Named>,
    /// What `line( … )` takes inside.
    pub line: Vec<Named>,
    pub operators: Vec<Named>,
    pub slots: Vec<Named>,
    /// `pseudo.<name>`: none is built yet.
    pub computed: Vec<Named>,
    pub not_built: Vec<NotBuilt>,
    pub limits: Vec<Limit>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Named {
    pub name: String,
    /// `text`, `number`, `closed set`, `id`, `flag`.
    pub kind: &'static str,
    pub what: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<&'static str>,
}

/// One limit of the digest's register, in its wording (C102).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Limit {
    pub id: &'static str,
    pub said: &'static str,
}

/// The limits this build can meet (`search/DIGEST.md`, "Limits
/// register"), in the register's words.
pub const LIMITS: &[Limit] = &[
    Limit {
        id: "S12",
        said: "a veiled line is shown as the placeholder it is; a value query never matches it",
    },
    Limit {
        id: "S52",
        said: "a line is matched as displayed text and value; which mods made it is not known and never guessed",
    },
    Limit {
        id: "S53",
        said: "an item's class is not a field; a class the search names is a derivation it owns, and an item it cannot class is shown unclassed",
    },
    Limit {
        id: "S107",
        said: crate::answer::S107,
    },
    Limit {
        id: "S177",
        said: "league of origin is not a fact the search holds; a derivation from league-specific mods or first-seen is offered as a derivation, never as the answer",
    },
    Limit {
        id: "S178",
        said: "legacy is the user's knowledge; the search finds what the user names and never says whether the game can still produce it",
    },
];

/// A limit's wording, by id.
pub fn limit(id: &str) -> &'static str {
    LIMITS.iter().find(|l| l.id == id).map_or("", |l| l.said)
}

fn named(name: &str, kind: &'static str, what: &str) -> Named {
    Named {
        name: name.to_string(),
        kind,
        what: what.to_string(),
        values: Vec::new(),
        examples: Vec::new(),
    }
}

/// The whole description, or the entries named — a field, `line`, a line's
/// attribute, `is`, `has`, an operator, a slot word. An unknown name is an
/// authoring error with the near ones offered.
pub fn describe(names: &[String]) -> Result<Describe, LanguageError> {
    let mut fields: Vec<Named> = FIELDS
        .iter()
        .map(|f| {
            let (kind, values) = match f.kind {
                Kind::Text => ("text", Vec::new()),
                Kind::Number => ("number", Vec::new()),
                Kind::Handle => ("id", Vec::new()),
                Kind::Closed(list) => ("closed set", list.to_vec()),
            };
            Named {
                values,
                ..named(f.name, kind, f.what)
            }
        })
        .collect();
    fields.push(Named {
        values: ITEM_FLAGS.to_vec(),
        examples: vec!["is:corrupted", "-is:identified"],
        ..named(
            "is",
            "flag",
            "every yes the item's body says, as GGG spells it; `-is:` is no, silence is any",
        )
    });
    fields.push(Named {
        values: FIELDS
            .iter()
            .map(|f| f.name)
            .filter(|n| !matches!(*n, "text" | "id"))
            .collect(),
        examples: vec!["has:note", "-has:name"],
        ..named(
            "has",
            "flag",
            "presence; absence only when all that could hold the thing was readable",
        )
    });
    let line = vec![
        Named {
            examples: vec![
                "line(\"# to maximum Life\" arg1>=90)",
                "line(template:resistance is:fractured)",
                "line(template~\"^Adds\")",
            ],
            ..named(
                "template",
                "text",
                &format!(
                    "the displayed line with each number a #, the sign in the number; a bare \"T\" means template=\"T\". {}",
                    limit("S52")
                ),
            )
        },
        Named {
            values: SOURCES.to_vec(),
            examples: vec!["line(\"# to maximum Life\" source=explicit)"],
            ..named(
                "source",
                "closed set",
                "the array the line came in, without its `Mods`; hybrid is a vaal gem's base skill",
            )
        },
        Named {
            values: LINE_FLAGS.to_vec(),
            examples: vec!["line(template:life -is:crafted)"],
            ..named("is", "flag", "the flags the body sets on the line")
        },
    ];
    let operators = vec![
        named(
            ":",
            "operator",
            "contains, in any case; on a closed set, picks among its legal values",
        ),
        named(
            "=",
            "operator",
            "the whole value, in any case; with a..b, a number inside the range, a side may be blank",
        ),
        named(
            "~",
            "operator",
            "a pattern: Rust regex syntax, unanchored, any case unless it says (?-i); ^ and $ are the ends of one displayed string. Patterns are for words, comparisons for numbers",
        ),
        named("> >= < <=", "operator", "a number against a number"),
    ];
    let slots = vec![
        named(
            "arg1 arg2 …",
            "slot",
            "every number of a template by position, counting from 1",
        ),
        named(
            "low high avg",
            "slot",
            "on a ranged line — a template with exactly one `# to #`: that pair, and their mean",
        ),
    ];
    let mut out = Describe {
        fields,
        line,
        operators,
        slots,
        computed: Vec::new(),
        not_built: NOT_BUILT.to_vec(),
        limits: LIMITS.to_vec(),
    };
    if names.is_empty() {
        return Ok(out);
    }
    // narrow to what was named
    let known: Vec<String> = out
        .fields
        .iter()
        .chain(&out.line)
        .map(|n| n.name.clone())
        .chain([
            "line".to_string(),
            "slots".to_string(),
            "operators".to_string(),
        ])
        .collect();
    for name in names {
        if !known.iter().any(|k| k.eq_ignore_ascii_case(name)) {
            if let Some(unbuilt) = NOT_BUILT.iter().find(|n| {
                n.construct
                    .trim_end_matches([':', '*', '.'])
                    .eq_ignore_ascii_case(name)
            }) {
                return Err(bind::not_built(unbuilt.construct));
            }
            let known: Vec<&str> = known.iter().map(String::as_str).collect();
            let near = bind::near(name, &known);
            return Err(LanguageError::new(
                ErrorKind::UnknownName,
                format!(
                    "`{name}` is nothing `--describe` knows: {}",
                    known.join(", ")
                ),
            )
            .with_readings(near.into_iter().map(str::to_string).collect()));
        }
    }
    let wants = |n: &str| names.iter().any(|w| w.eq_ignore_ascii_case(n));
    let whole_line = wants("line");
    out.fields.retain(|f| wants(&f.name));
    out.line.retain(|l| whole_line || wants(&l.name));
    if !wants("operators") {
        out.operators.clear();
    }
    if !wants("slots") && !whole_line {
        out.slots.clear();
    }
    out.not_built.clear();
    out.limits.clear();
    Ok(out)
}
