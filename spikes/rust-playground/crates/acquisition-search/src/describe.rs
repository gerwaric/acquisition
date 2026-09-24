//! `--describe` (C97: the closed lists are printed in the help): the
//! language as this build knows it — fields, what a line has, what has a
//! value, how terms compose, operators, closed value sets, slots, computed
//! values, what a count takes, what is not built, and the limits it states
//! (C102). An entry is a name, a line and an example: the reference is
//! `search/DESIGN.md`, and this is what a terminal can ask of it. Every
//! example printed is a query this build binds, or a key a count takes
//! (`tests/fifth_audit.rs`, `tests/counts.rs`), and every word an entry is
//! named by can be asked for alone. Printed from the binder's own tables,
//! so the help cannot say a name the binder refuses.

use serde::Serialize;

use crate::bind::{
    self, FIELDS, ITEM_FLAGS, Kind, LINE_FLAGS, NOT_BUILT, NotBuilt, SOURCES, Thing,
};
use crate::error::{ErrorKind, LanguageError};

#[derive(Debug, Clone, Serialize)]
pub struct Describe {
    pub fields: Vec<Named>,
    /// What `line( … )` takes inside.
    pub line: Vec<Named>,
    /// What a comparison, a sum or `--sort` consumes.
    pub values: Vec<Named>,
    /// How terms are put together.
    pub composition: Vec<Named>,
    pub operators: Vec<Named>,
    pub slots: Vec<Named>,
    /// `pseudo.<name>`: none is built yet.
    pub computed: Vec<Named>,
    /// What `--count`, `--cross` and `--sum` take (C95, C97).
    pub counts: Vec<Named>,
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
/// attribute, `is`, `has`, an operator, a slot word (`arg7` is one), a word
/// of composition, a block by its name. An unknown name is an authoring
/// error with the near ones offered.
pub fn describe(names: &[String]) -> Result<Describe, LanguageError> {
    let mut fields: Vec<Named> = FIELDS
        .iter()
        .map(|f| {
            let (kind, values) = match f.kind {
                Kind::Text => ("text", Vec::new()),
                Kind::Number => ("number", Vec::new()),
                Kind::Handle => ("id", Vec::new()),
                Kind::Closed(list) => ("closed set", list().to_vec()),
            };
            // the class prints its definition and its source version
            // (C106's clause (c)); a table that does not load says so
            let (what, examples) = match f.thing {
                Thing::Class => (
                    match crate::class::table() {
                        Ok(table) => table.definition(),
                        Err(e) => {
                            format!("{} — the table this build ships does not load: {e}", f.what)
                        }
                    },
                    vec![
                        "class:ring",
                        "class=Rings",
                        "class:sword",
                        "undecided(class)",
                    ],
                ),
                Thing::ReqLevel => (f.what.to_string(), vec!["reqlevel=..30", "-has:reqlevel"]),
                _ => (f.what.to_string(), Vec::new()),
            };
            Named {
                values,
                examples,
                ..named(f.name, kind, &what)
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
    let values = vec![
        Named {
            examples: vec![
                "line(\"Adds # to # Cold Damage\").avg>=20",
                "\"# to maximum Life\">=90",
            ],
            ..named(
                "line(P).<slot>",
                "value",
                "one occurrence's number: a comparison on it means what it means inside the group, and `\"T\">=90` is its shorthand; `--sort` takes it, the largest that satisfies P",
            )
        },
        Named {
            examples: vec![
                "sum(\"# to maximum Life\")>=90",
                "sum(line(template:resistance).arg1)>=60",
            ],
            ..named(
                "sum",
                "value",
                "the item's sum of a slot over the occurrences P selects, exact in decimals; nothing sums to zero; with a possible contributor unread it is incomplete and its comparison undecided",
            )
        },
    ];
    let composition = vec![
        Named {
            examples: vec!["rarity=rare base:ring", "rarity=rare and base:ring"],
            ..named(
                "and",
                "composition",
                "whitespace or the word, at the level you are in; `a b or c` unparenthesised is an error showing both readings",
            )
        },
        Named {
            examples: vec!["base:ring or base:amulet"],
            ..named("or", "composition", "either")
        },
        Named {
            examples: vec!["-is:corrupted", "not has:note"],
            ..named(
                "not -",
                "composition",
                "the term is false; at a terminal a query that starts with - goes after --",
            )
        },
        Named {
            examples: vec!["(base:ring or base:amulet) rarity=rare"],
            ..named("( )", "composition", "grouping, and nothing else")
        },
        Named {
            examples: vec!["holds(rarity=rare, is:corrupted, has:note)>=2"],
            ..named(
                "holds",
                "composition",
                "how many of its queries hold: >=n, <=n, =n, =a..b",
            )
        },
        Named {
            examples: vec!["undecided(ilvl)", "undecided(\"# to maximum Life\">=90)"],
            ..named(
                "undecided",
                "composition",
                "true when a thing's value or a term's truth cannot be established on the item — something it needs was unread; always decided itself",
            )
        },
        named("true() false()", "composition", "always true, always false"),
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
    let keys: Vec<&'static str> = FIELDS
        .iter()
        .filter(|f| !matches!(f.name, "text" | "id"))
        .map(|f| f.name)
        .chain(["line"])
        .collect();
    let counts = vec![
        Named {
            values: keys,
            examples: vec!["tab,league,rarity", "base", "line", "line:resist,life"],
            ..named(
                "--count",
                "view",
                "the matches counted by each key, one table each, no rows: a field an item has or lacks — under `none` with none, `undecided` where it could not be read — or `line`, the vocabulary (every count has its route)",
            )
        },
        Named {
            examples: vec!["line:resist,life", "line~^adds", "line:life,~^adds"],
            ..named(
                "line:",
                "view",
                "the vocabulary narrowed: the templates the matching items carry whose text holds the words, ranked, with the term that selects each, its numbers' range, and its sources and flags; `line~` a pattern; the rest of the list is texts, one table each",
            )
        },
        Named {
            examples: vec!["league,tab", "rarity,base"],
            ..named(
                "--cross",
                "view",
                "one table of two fields: the cells that hold an item, each routed under both",
            )
        },
        Named {
            examples: vec!["stack", "ilvl", "sum(\"# to maximum Life\")"],
            ..named(
                "--sum",
                "value",
                "beside each count, one number of each item added: an item lacking it adds nothing and is counted as lacking; one unread leaves a subtotal marked incomplete, never a total",
            )
        },
    ];
    let mut out = Describe {
        fields,
        line,
        values,
        composition,
        operators,
        slots,
        computed: Vec::new(),
        counts,
        not_built: NOT_BUILT.to_vec(),
        limits: LIMITS.to_vec(),
    };
    if names.is_empty() {
        return Ok(out);
    }
    // narrow to what was named: an entry answers to its name and to each
    // word of it, the positional slots to any `arg<N>`, a block to its own
    let answers = |entry: &str, asked: &str| {
        entry.eq_ignore_ascii_case(asked)
            // a flag answers to its word: `count` is `--count`
            || entry.trim_start_matches('-').eq_ignore_ascii_case(asked)
            || entry
                .split_whitespace()
                .any(|word| word != "…" && word.eq_ignore_ascii_case(asked))
            || (entry.starts_with("arg1") && crate::tree::arg_index(asked).is_some())
    };
    const BLOCKS: [&str; 7] = [
        "fields",
        "line",
        "values",
        "composition",
        "operators",
        "slots",
        "counts",
    ];
    let entries = |d: &Describe| -> Vec<String> {
        [
            &d.fields,
            &d.line,
            &d.values,
            &d.composition,
            &d.operators,
            &d.slots,
            &d.counts,
        ]
        .into_iter()
        .flatten()
        .map(|n| n.name.clone())
        .collect()
    };
    let all = entries(&out);
    for name in names {
        let known = BLOCKS.iter().any(|b| b.eq_ignore_ascii_case(name))
            || all.iter().any(|entry| answers(entry, name));
        if !known {
            if let Some(unbuilt) = NOT_BUILT.iter().find(|n| {
                n.construct
                    .trim_end_matches([':', '*', '.'])
                    .eq_ignore_ascii_case(name)
            }) {
                return Err(bind::not_built(unbuilt.construct));
            }
            let words: Vec<&str> = all
                .iter()
                .flat_map(|entry| entry.split_whitespace())
                .filter(|word| *word != "…")
                .chain(BLOCKS)
                .collect();
            let near = bind::near(name, &words);
            return Err(LanguageError::new(
                ErrorKind::UnknownName,
                format!(
                    "`{name}` is nothing `--describe` knows: {}",
                    words.join(", ")
                ),
            )
            .with_readings(near.into_iter().map(str::to_string).collect()));
        }
    }
    let wants = |block: &str, entry: &str| {
        names
            .iter()
            .any(|asked| asked.eq_ignore_ascii_case(block) || answers(entry, asked))
    };
    out.fields.retain(|n| wants("fields", &n.name));
    out.line.retain(|n| wants("line", &n.name));
    out.values.retain(|n| wants("values", &n.name));
    out.composition.retain(|n| wants("composition", &n.name));
    out.operators.retain(|n| wants("operators", &n.name));
    // a line's numbers are named by its slots
    out.slots
        .retain(|n| wants("slots", &n.name) || wants("line", &n.name));
    out.counts.retain(|n| wants("counts", &n.name));
    out.not_built.clear();
    out.limits.clear();
    Ok(out)
}
