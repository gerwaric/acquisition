//! The derivation (C103, C90): one item's stored body and its ingest
//! facts in, the item as the search sees it out. A pure function — no
//! store type, no file, no clock — so it can move under the store
//! unchanged if persistence ever fires (C98).
//!
//! What the words mean is `search/DESIGN.md`'s reference (*Strings*,
//! *Item-level*, *Slots*), cited here by section and not restated (the
//! build plan, rule 6).
//!
//! # As built
//!
//! **The deriver reads; it does not interpret.** A field is what GGG
//! gives under its own key, and a rule that joins two of them is never
//! made here: `rarity` and `frame` are two fields (owner, 2026-09-19), a
//! gem lacking the first and carrying the second. What is normalised is
//! what every displayed string gets (C90; `shown`, below), and the two
//! spellings GGG has for nothing: an empty string, and an `ilvl` of 0,
//! which the game shows no item level for (owner, 2026-09-19).
//!
//! - **Place is never read from a body (C103).** [`Facts`] is handed in
//!   and handed back; a body's own `league` or `realm` is ignored.
//! - **Displayed strings** (`shown`): a row break is `\n` whatever the
//!   body wrote (`\r\n` on 406 lines of the census's store copy), and
//!   markup is reduced to what the player sees — `[Tag|Display]` and
//!   `[Display]` (S4), and `<style>{Display}`, nested, which a divination
//!   card's reward text carries (821 tags on that copy, each followed by
//!   its brace).
//! - **Lines (C90).** Every top-level array whose key ends in `Mods` is a
//!   source of lines, its word the key without the suffix (`explicit`,
//!   `enchant`, `crucible`, …), so an array GGG adds is read the day it
//!   appears; a vaal gem's base skill, `hybrid.explicitMods`, is the source
//!   `hybrid`, and `hybrid.properties` are displayed strings (owner,
//!   2026-09-19); an element is a string or an object with a `description`,
//!   and an object's true `flags` are the line's. The template and the
//!   numbers are `template::typed`'s — the sign in the number, a range
//!   dash never a sign, `1,500` one number. Two exceptions, each a fact
//!   about GGG's body and not a choice of meaning: `ultimatumMods` holds
//!   ids (`FrostInfection`, tier 3) of what `explicitMods` already
//!   displays (`Blistering Cold III`) and is no source; a `veiledMods`
//!   line is a placeholder re-rolled per response (S12), so it keeps its
//!   template (`Suffix#`) and carries no number — a value query never
//!   matches it.
//! - **A number the search does not read** — more whole digits or decimals
//!   than any game displays (`exact.rs`, the rule and its measurement) —
//!   is an unread slot of its line, said here, once, so that no arithmetic
//!   downstream has a case for it. As with a flag: unknown, never absent
//!   and never a no; the line's text, template, source and its other
//!   numbers are read, and only what asks that slot is left open
//!   ([`Slot`], [`Part::Numbers`]).
//! - **Slots** ([`Line::slot`]): `arg<N>` by position, and `low`, `high`,
//!   `avg` on a template with exactly one `# to #` (C92). A line whose
//!   numbers are not its template's `#`s — a veiled line, a displayed
//!   literal `#` — names no slot.
//! - **Properties** are displayed strings with their name and values kept
//!   apart, rendered by `displayMode` as the game shows them: `Quality:
//!   +20%`, `Weapon Range: 1.1 metres`. Each is a string of its own; a
//!   phrase never matches across two.
//! - **Requirements** are kept one by one, the name without a colon
//!   (`Level 67`, `159 Str`), and displayed as the one row the game, the
//!   trade site and the C++ app show: `Requires Level 67, 159 Str` (owner,
//!   2026-09-19). A phrase is tested against the row.
//! - **The item level** is a field, `ilvl`, and a displayed string, `Item
//!   Level: 84`, where the item has one (owner, 2026-09-19).
//! - **The level required** is a field, `reqlevel`: the one `Level` row
//!   of `requirements`, its one value as a whole number (the build plan,
//!   step 6). An item with no such row lacks it — known absence, so
//!   `-has:reqlevel` finds it — and every other shape is unread under
//!   `reqlevel`, said here and nowhere else (rule 8 of the plan): a
//!   `Level` row with no value, several, or a value that is no whole
//!   number; two `Level` rows; an element of `requirements` that could not
//!   be read and might be the `Level` row — its name unreadable or
//!   `Level` — and the array itself not being one. One status, which a
//!   comparison, a sort and a sum all read: the number is established
//!   exactly when nothing is unread under `reqlevel`, so a `Level` that
//!   was read stands beside an unread `Str` and does not beside an
//!   unread element that may be a second `Level` (the step-6 reviews, 1
//!   and 2). The row itself stays a displayed string.
//! - **The class is not the deriver's** (C103, C106): `class.rs` reads it
//!   from the base in the class table, and says under [`Part::Class`] why
//!   it could not.
//! - **A yes or no that is neither** is unread, never a no (C93): a key
//!   of [`ITEM_FLAGS`] or of `influences` whose value is no boolean is
//!   unread under that key — `influences.hunter`, so that a no beside it
//!   is a no still — and a line says so itself: [`Line::flags_unread`]
//!   when its `flags` is no object, and the flags by name
//!   ([`Line::flags_unknown`]) when a value in it is no boolean. Its text
//!   is a witness all the same, and so is every flag it could read.
//! - **Unread, by collection (C93).** What the deriver met and could not
//!   read is an [`Unread`] naming the [`Part`] — never a panic (C47), never
//!   a silent gap. The readable rest of the part is still derived, since a
//!   readable line is a witness; absence is claimed only when
//!   [`Item::unread_in`] finds nothing.
//!
//! Not read yet: sockets (the build plan, step 8), and the other
//! property-shaped arrays — `nextLevelRequirements`,
//! `weaponRequirements`, `supportGemRequirements`, and poe2's `gemTabs`
//! and `grantedSkills` (the plan's holes table, D4).

use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Map, Value};

pub use crate::class::ClassGap;
use crate::template;
use crate::tree::Number;

/// One item's ingest facts, as the store's read hands them over (C103): a
/// plain struct this crate owns, so the deriver names no store type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facts {
    pub id: String,
    pub realm: String,
    /// None: a league-less character's item (S131).
    pub league: Option<String>,
    /// `stash` or `character`.
    pub location_kind: String,
    pub location_id: String,
    /// The array the item came from; None on a row from before facts v4.
    pub container: Option<String>,
    /// The item this one is socketed in.
    pub socketed_in: Option<String>,
    pub first_seen: i64,
    pub last_seen: i64,
    pub removed_at: Option<i64>,
}

/// The item as the search sees it. Every `Option` is present or *known*
/// absent only while [`Item::unread_in`] says its part was readable.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Item {
    pub facts: Facts,
    /// The header (the reference, *Item-level*): GGG's `name`, `typeLine`
    /// and `baseType`; an empty string is an item without one.
    pub name: Option<String>,
    pub typeline: Option<String>,
    pub base: Option<String>,
    /// GGG's `rarity`, as given: absent on a gem, a currency stack, a card.
    pub rarity: Option<String>,
    /// GGG's `frameTypeId`, as given: on every item.
    pub frame: Option<String>,
    /// GGG's `ilvl`; its 0 is absent.
    pub ilvl: Option<i64>,
    /// The row it is displayed as: `Item Level: 84`.
    pub item_level: Option<String>,
    /// GGG's `stackSize`.
    pub stack: Option<i64>,
    /// The `Level` requirement, as a number; absent on an item with none.
    pub reqlevel: Option<i64>,
    pub note: Option<String>,
    /// Every yes the body says: a top-level boolean that is true, and the
    /// true keys of `influences`, as GGG spells them, sorted.
    pub flags: Vec<String>,
    pub properties: Vec<Property>,
    /// Each requirement on its own: `Level 67`, `159 Str`.
    pub requirements: Vec<Property>,
    /// The row they are displayed as: `Requires Level 67, 159 Str`.
    pub requires: Option<String>,
    pub lines: Vec<Line>,
    pub unread: Vec<Unread>,
}

/// One element of a property-shaped array, as displayed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Property {
    /// The body's key: `properties`, `additionalProperties`,
    /// `requirements`, `hybrid.properties`.
    pub array: String,
    pub name: String,
    pub values: Vec<String>,
    /// The string the player sees.
    pub text: String,
}

/// One displayed occurrence of a line (C90, C92).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Line {
    /// The source array, without its `Mods`: `explicit`, `implicit`, …;
    /// `hybrid` for a vaal gem's base skill.
    pub source: String,
    /// The flags the body sets on the line: `crafted`, `fractured`, …
    pub flags: Vec<String>,
    /// The line's `flags` is no object: one named in `flags` is a yes, and
    /// every other is unknown, never a no.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub flags_unread: bool,
    /// The flags whose value was no boolean: unknown, where every other
    /// flag of a readable object is the yes or no it says.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags_unknown: Vec<String>,
    pub template: String,
    /// The template's numbers, in order; `None` where the search does not
    /// read the number as written: unknown, never absent.
    #[serde(serialize_with = "whole_numbers")]
    pub numbers: Vec<Option<f64>>,
    /// As shown, with its numbers; a mod over several rows holds `\n`.
    pub text: String,
}

/// Something the deriver met and could not read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unread {
    #[serde(flatten)]
    pub part: Part,
    pub problem: String,
    /// The occurrence it is of, by its place in [`Item::lines`], when it is
    /// one line's flags or numbers: what is open on one occurrence is
    /// explained by that occurrence, never by another of its array.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
}

/// The collection a reading belongs to: what C93's "everything that could
/// hold the thing was readable" is asked of.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "part", content = "of", rename_all = "snake_case")]
pub enum Part {
    /// The body is not a JSON object: nothing it displays was read.
    Body,
    /// A field or flag, by the body's key; `reqlevel`, the `Level`
    /// requirement's number, by its own name.
    Field(String),
    /// A property-shaped array, by the body's key.
    Properties(String),
    /// A source of lines, by its word.
    Lines(String),
    /// The flags of a line of that source: its text was read, so it
    /// hides no line and no displayed string — only what `is:` asks of it.
    Flags(String),
    /// A number of a line of that source: only what asks that slot.
    Numbers(String),
    /// The class, which the class table could not give the base
    /// (`class.rs`): only what asks the class.
    Class(ClassGap),
}

/// What a slot word names on one occurrence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Slot {
    /// The template has no such number: known absence.
    Absent,
    /// It has, and the number could not be read.
    Unread,
    Is(f64),
}

/// Where a displayed string sits on its item.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shown<'a> {
    Name,
    Typeline,
    Base,
    Property(&'a Property),
    /// The one row of [`Item::item_level`].
    ItemLevel,
    /// The one row of [`Item::requires`].
    Requires,
    Line(&'a Line),
}

/// Every yes an item's body is known to say: the top-level booleans and
/// the keys of `influences`, as GGG spells them, as the census of
/// 2026-09-13 met them (22,721 items). The language's `is:` words
/// (`bind`), and the keys whose value must be a boolean to be read.
pub const ITEM_FLAGS: &[&str] = &[
    "abyssJewel",
    "corrupted",
    "crusader",
    "delve",
    "duplicated",
    "elder",
    "fractured",
    "hunter",
    "identified",
    "isRelic",
    "memoryItem",
    "mutated",
    "redeemer",
    "replica",
    "searing",
    "shaper",
    "split",
    "support",
    "synthesised",
    "tangled",
    "unmodifiable",
    "unmodifiableExceptChaos",
    "veiled",
    "vestigial",
    "warlord",
];

/// The flags `influences` holds; `shaper` and `elder` are top-level keys
/// too.
pub const INFLUENCES: &[&str] = &[
    "crusader", "elder", "hunter", "redeemer", "shaper", "warlord",
];

/// The arrays read as properties; `hybrid.properties` joins them.
const PROPERTY_ARRAYS: [&str; 2] = ["properties", "additionalProperties"];

/// `*Mods` arrays that are no source of lines (the module doc).
const NOT_LINES: [&str; 1] = ["ultimatumMods"];

/// Derive one item. Total: any text is an item, at worst one whose body is
/// unread (C47).
pub fn derive(facts: Facts, body: &str) -> Item {
    let mut item = Item {
        facts,
        name: None,
        typeline: None,
        base: None,
        rarity: None,
        frame: None,
        ilvl: None,
        item_level: None,
        stack: None,
        reqlevel: None,
        note: None,
        flags: Vec::new(),
        properties: Vec::new(),
        requirements: Vec::new(),
        requires: None,
        lines: Vec::new(),
        unread: Vec::new(),
    };
    let body = match serde_json::from_str::<Value>(body) {
        Ok(Value::Object(body)) => body,
        Ok(_) => {
            item.unread(Part::Body, "the body is not a JSON object");
            return item;
        }
        Err(e) => {
            item.unread(Part::Body, format!("the body is not JSON: {e}"));
            return item;
        }
    };
    item.name = item.text(&body, "name");
    item.typeline = item.text(&body, "typeLine");
    item.base = item.text(&body, "baseType");
    item.rarity = item.text(&body, "rarity");
    item.frame = item.text(&body, "frameTypeId");
    item.note = item.text(&body, "note");
    // GGG's 0 is an item the game shows no level for: known absence
    item.ilvl = item.int(&body, "ilvl").filter(|ilvl| *ilvl != 0);
    item.item_level = item.ilvl.map(|ilvl| format!("Item Level: {ilvl}"));
    item.stack = item.int(&body, "stackSize");
    item.read_flags(&body);
    for array in PROPERTY_ARRAYS {
        item.read_properties(&body, array, array);
    }
    item.read_properties(&body, "requirements", "requirements");
    if !item.requirements.is_empty() {
        let each: Vec<&str> = item.requirements.iter().map(|r| r.text.as_str()).collect();
        item.requires = Some(format!("Requires {}", each.join(", ")));
    }
    // one status: the number stands only while nothing under `reqlevel`
    // is unread — an unread element may be a second `Level` row
    item.reqlevel = item.read_reqlevel().filter(|_| {
        !item
            .unread
            .iter()
            .any(|u| u.part == Part::Field("reqlevel".to_string()))
    });
    let mut sources: Vec<(&str, String, &Value)> = body
        .iter()
        .filter(|(key, _)| !NOT_LINES.contains(&key.as_str()))
        .filter_map(|(key, value)| Some((key.strip_suffix("Mods")?, key.clone(), value)))
        .filter(|(source, _, _)| !source.is_empty())
        .collect();
    // a vaal gem's base skill: its lines are a source of their own
    match body.get("hybrid") {
        None | Some(Value::Null) => {}
        Some(Value::Object(hybrid)) => {
            item.read_properties(hybrid, "properties", "hybrid.properties");
            if let Some(lines) = hybrid.get("explicitMods") {
                sources.push(("hybrid", "hybrid.explicitMods".to_string(), lines));
            }
        }
        Some(other) => {
            let problem = format!("`hybrid` is {}, not an object", json_kind(other));
            item.unread(Part::Field("hybrid".to_string()), problem);
        }
    }
    // by source word, so the order never depends on how the body was held
    sources.sort_by_key(|(source, _, _)| *source);
    for (source, key, value) in sources {
        item.read_lines(source, &key, value);
    }
    item
}

impl Item {
    /// What stops an absence in `part` being claimed (C93): the part's own
    /// unread entries, and an unread body, which leaves every part unread.
    pub fn unread_in<'a>(&'a self, part: &'a Part) -> impl Iterator<Item = &'a Unread> {
        self.unread
            .iter()
            .filter(move |u| u.part == Part::Body || u.part == *part)
    }

    /// Every displayed string a phrase is tested against, row by row (the
    /// reference, *Item-level*): the header, each property, the item
    /// level, the requirements' row, each row of each line. Never the flavour text,
    /// the description or the note.
    pub fn displayed(&self) -> impl Iterator<Item = (Shown<'_>, &str)> {
        let header = [
            (Shown::Name, self.name.as_deref()),
            (Shown::Typeline, self.typeline.as_deref()),
            (Shown::Base, self.base.as_deref()),
        ]
        .into_iter()
        .filter_map(|(shown, text)| text.map(|text| (shown, text)));
        let properties = self
            .properties
            .iter()
            .flat_map(|p| p.text.split('\n').map(move |row| (Shown::Property(p), row)));
        let item_level = self
            .item_level
            .as_deref()
            .map(|row| (Shown::ItemLevel, row));
        let requires = self.requires.as_deref().map(|row| (Shown::Requires, row));
        let lines = self
            .lines
            .iter()
            .flat_map(|l| l.rows().map(move |row| (Shown::Line(l), row)));
        header
            .chain(properties)
            .chain(item_level)
            .chain(requires)
            .chain(lines)
    }

    fn unread(&mut self, part: Part, problem: impl Into<String>) {
        self.unread.push(Unread {
            part,
            problem: problem.into(),
            line: None,
        });
    }

    /// Something unread of the occurrence about to be kept.
    fn unread_of_this_line(&mut self, part: Part, problem: String) {
        let line = Some(self.lines.len());
        self.unread.push(Unread {
            part,
            problem,
            line,
        });
    }

    /// A text field: absent when the key is missing or the string empty.
    fn text(&mut self, body: &Map<String, Value>, key: &str) -> Option<String> {
        match body.get(key) {
            None | Some(Value::Null) => None,
            Some(Value::String(s)) if s.is_empty() => None,
            Some(Value::String(s)) => Some(shown(s)),
            Some(other) => {
                self.unread(
                    Part::Field(key.to_string()),
                    format!("`{key}` is {}, not a string", json_kind(other)),
                );
                None
            }
        }
    }

    fn int(&mut self, body: &Map<String, Value>, key: &str) -> Option<i64> {
        match body.get(key) {
            None | Some(Value::Null) => None,
            Some(value) => match value.as_i64() {
                Some(n) => Some(n),
                None => {
                    self.unread(
                        Part::Field(key.to_string()),
                        format!("`{key}` is {}, not a whole number", json_kind(value)),
                    );
                    None
                }
            },
        }
    }

    /// The `Level` requirement's number (the module doc): one row, one
    /// value, a whole number; any other shape is unread under `reqlevel`.
    fn read_reqlevel(&mut self) -> Option<i64> {
        let levels: Vec<&Property> = self
            .requirements
            .iter()
            .filter(|r| r.name.eq_ignore_ascii_case("Level"))
            .collect();
        let problem = match levels.as_slice() {
            [] => return None,
            [one] => match one.values.as_slice() {
                [value] => match value.trim().parse::<i64>() {
                    Ok(n) => return Some(n),
                    Err(_) => format!("`requirements`: `Level` is `{value}`, not a whole number"),
                },
                values => format!(
                    "`requirements`: `Level` has {} values, not one",
                    values.len()
                ),
            },
            several => format!(
                "`requirements`: `Level` appears {} times, not once",
                several.len()
            ),
        };
        self.unread(Part::Field("reqlevel".to_string()), problem);
        None
    }

    /// An element of `requirements` that could not be read, or the array
    /// itself: the `Level` row may be among what was lost.
    fn level_may_be_lost(&mut self, at: &str, element: Option<&Value>) {
        // the name as the reader normalises it: `[Level]` is `Level`
        let could_be_level = match element.and_then(|e| e.get("name")) {
            Some(Value::String(name)) => shown(name).eq_ignore_ascii_case("Level"),
            _ => true,
        };
        if could_be_level {
            self.unread(
                Part::Field("reqlevel".to_string()),
                format!("{at} could not be read and may be the `Level` requirement"),
            );
        }
    }

    fn read_flags(&mut self, body: &Map<String, Value>) {
        let mut flags: Vec<String> = body
            .iter()
            .filter(|(_, value)| **value == Value::Bool(true))
            .map(|(key, _)| key.clone())
            .collect();
        for key in ITEM_FLAGS {
            match body.get(*key) {
                None | Some(Value::Null | Value::Bool(_)) => {}
                Some(other) => self.unread(
                    Part::Field(key.to_string()),
                    format!("`{key}` is {}, not yes or no", json_kind(other)),
                ),
            }
        }
        match body.get("influences") {
            None | Some(Value::Null) => {}
            Some(Value::Object(influences)) => {
                for (key, value) in influences {
                    match value {
                        Value::Bool(true) => flags.push(key.clone()),
                        Value::Bool(false) => {}
                        // under its own key: a no beside it is a no still
                        other => self.unread(
                            Part::Field(format!("influences.{key}")),
                            format!("`influences.{key}` is {}, not yes or no", json_kind(other)),
                        ),
                    }
                }
            }
            Some(other) => self.unread(
                Part::Field("influences".to_string()),
                format!("`influences` is {}, not an object", json_kind(other)),
            ),
        }
        flags.sort();
        flags.dedup();
        self.flags = flags;
    }

    /// `array` is the name the part and its properties carry; `key` is
    /// where it sits in `holder`.
    fn read_properties(&mut self, holder: &Map<String, Value>, key: &str, array: &str) {
        let part = || Part::Properties(array.to_string());
        let elements = match holder.get(key) {
            None | Some(Value::Null) => return,
            Some(Value::Array(elements)) => elements,
            Some(other) => {
                let problem = format!("`{array}` is {}, not an array", json_kind(other));
                self.unread(part(), problem);
                if array == "requirements" {
                    self.level_may_be_lost(&format!("`{array}`"), None);
                }
                return;
            }
        };
        for (i, element) in elements.iter().enumerate() {
            match property(array, element) {
                Ok(Some(property)) if array == "requirements" => self.requirements.push(property),
                Ok(Some(property)) => self.properties.push(property),
                Ok(None) => {}
                Err(problem) => {
                    self.unread(part(), format!("`{array}[{i}]`: {problem}"));
                    if array == "requirements" {
                        self.level_may_be_lost(&format!("`{array}[{i}]`"), Some(element));
                    }
                }
            }
        }
    }

    fn read_lines(&mut self, source: &str, key: &str, value: &Value) {
        let part = || Part::Lines(source.to_string());
        let elements = match value {
            Value::Null => return,
            Value::Array(elements) => elements,
            other => {
                let problem = format!("`{key}` is {}, not an array", json_kind(other));
                return self.unread(part(), problem);
            }
        };
        for (i, element) in elements.iter().enumerate() {
            let at = format!("`{key}[{i}]`");
            let (text, flags) = match element {
                Value::String(text) => (text, None),
                Value::Object(line) => match line.get("description") {
                    Some(Value::String(text)) => (text, line.get("flags")),
                    _ => {
                        self.unread(part(), format!("{at} has no `description` string"));
                        continue;
                    }
                },
                other => {
                    let problem = format!("{at} is {}, not a line", json_kind(other));
                    self.unread(part(), problem);
                    continue;
                }
            };
            // an essence's description spaces its rows with empty lines:
            // no occurrence, so nothing of it is kept or unread
            let text = shown(text);
            if text.is_empty() {
                continue;
            }
            let mut flags_unread = false;
            let mut flags_unknown = Vec::new();
            let flags = match flags {
                None | Some(Value::Null) => Vec::new(),
                Some(Value::Object(flags)) => {
                    for (flag, value) in flags {
                        if !value.is_boolean() && !value.is_null() {
                            flags_unknown.push(flag.clone());
                            let problem = format!(
                                "{at}: `flags.{flag}` is {}, not yes or no",
                                json_kind(value)
                            );
                            self.unread_of_this_line(Part::Flags(source.to_string()), problem);
                        }
                    }
                    let mut set: Vec<String> = flags
                        .iter()
                        .filter(|(_, v)| **v == Value::Bool(true))
                        .map(|(k, _)| k.clone())
                        .collect();
                    set.sort();
                    set
                }
                // the text is a witness all the same; its flags are not
                Some(other) => {
                    flags_unread = true;
                    let problem = format!("{at}: `flags` is {}, not an object", json_kind(other));
                    self.unread_of_this_line(Part::Flags(source.to_string()), problem);
                    Vec::new()
                }
            };
            let (template, read) = template::read(&text);
            let mut numbers: Vec<Option<f64>> = read
                .into_iter()
                .map(|(n, reads)| reads.then_some(n))
                .collect();
            if source == "veiled" {
                numbers.clear();
            }
            if numbers.contains(&None) {
                let problem = format!(
                    "{at}: a number with more digits than the search reads: ten whole, four decimals"
                );
                self.unread_of_this_line(Part::Numbers(source.to_string()), problem);
            }
            self.lines.push(Line {
                source: source.to_string(),
                flags,
                flags_unread,
                flags_unknown,
                template,
                numbers,
                text,
            });
        }
    }
}

impl Line {
    /// The displayed rows: a phrase tests each on its own.
    pub fn rows(&self) -> impl Iterator<Item = &str> {
        self.text.split('\n')
    }

    /// The number a slot word names on this occurrence (the reference,
    /// *Slots*); None when the template has no such slot, or when the
    /// numbers are not the template's `#`s.
    pub fn slot(&self, word: &str) -> Slot {
        let slots = template::slots(&self.template);
        if slots.count != self.numbers.len() {
            return Slot::Absent;
        }
        let at = |n: usize| match n.checked_sub(1).and_then(|i| self.numbers.get(i)) {
            None => Slot::Absent,
            Some(None) => Slot::Unread,
            Some(Some(n)) => Slot::Is(*n),
        };
        match (word, slots.ranged) {
            ("low", Some((low, _))) => at(low),
            ("high", Some((_, high))) => at(high),
            ("avg", Some((low, high))) => match (at(low), at(high)) {
                (Slot::Is(low), Slot::Is(high)) => Slot::Is(crate::exact::mean(low, high)),
                (Slot::Absent, _) | (_, Slot::Absent) => Slot::Absent,
                _ => Slot::Unread,
            },
            ("low" | "high" | "avg", None) => Slot::Absent,
            (other, _) => crate::tree::arg_index(other).map_or(Slot::Absent, at),
        }
    }

    /// Every slot this occurrence names, with its number — `None` where it
    /// could not be read: `low`, `high` and `avg` first on a ranged line,
    /// then `arg1`, `arg2`, …
    pub fn slots(&self) -> Vec<(String, Option<f64>)> {
        template::slot_words(&self.template)
            .into_iter()
            .filter_map(|word| match self.slot(&word) {
                Slot::Absent => None,
                Slot::Unread => Some((word, None)),
                Slot::Is(n) => Some((word, Some(n))),
            })
            .collect()
    }
}

/// One property as displayed; `Ok(None)` for one that shows nothing (a
/// separator, `displayMode` 4).
fn property(array: &str, element: &Value) -> Result<Option<Property>, String> {
    let Value::Object(p) = element else {
        return Err(format!("{}, not a property", json_kind(element)));
    };
    let name = match p.get("name") {
        Some(Value::String(name)) => shown(name),
        None | Some(Value::Null) => String::new(),
        Some(other) => return Err(format!("`name` is {}, not a string", json_kind(other))),
    };
    let mut values = Vec::new();
    match p.get("values") {
        None | Some(Value::Null) => {}
        Some(Value::Array(pairs)) => {
            for pair in pairs {
                // `[text, style]`: the style is a colour class (item-facts F4)
                match pair.as_array().and_then(|pair| pair.first()) {
                    Some(Value::String(value)) => values.push(shown(value)),
                    _ => return Err("a value is not `[text, style]`".to_string()),
                }
            }
        }
        Some(other) => return Err(format!("`values` is {}, not an array", json_kind(other))),
    }
    let joined = values.join(", ");
    let text = match p.get("displayMode").and_then(Value::as_i64) {
        // name: values — a requirement takes no colon
        Some(0 | 2) if name.is_empty() || values.is_empty() => format!("{name}{joined}"),
        Some(0 | 2) if array == "requirements" => format!("{name} {joined}"),
        Some(0 | 2) => format!("{name}: {joined}"),
        Some(1) if name.is_empty() => joined,
        Some(1) => format!("{joined} {name}"),
        Some(3) => {
            let mut text = name.clone();
            for (i, value) in values.iter().enumerate() {
                let slot = format!("{{{i}}}");
                // a heist trinket's `Any Heist member can equip this item.`
                // carries one empty value and no slot: nothing is lost
                if !text.contains(&slot) && !value.is_empty() {
                    return Err(format!("`{name}` has no {slot} for its value"));
                }
                text = text.replace(&slot, value);
            }
            text
        }
        Some(4) => String::new(),
        _ => return Err("a `displayMode` this deriver does not render".to_string()),
    };
    if text.is_empty() {
        return Ok(None);
    }
    Ok(Some(Property {
        array: array.to_string(),
        name,
        values,
        text,
    }))
}

/// A displayed string as the player sees it (C90; the reference,
/// *Item-level*: "markup reduced to what the player sees").
pub(crate) fn shown(raw: &str) -> String {
    let text = raw.replace("\r\n", "\n").replace('\r', "\n");
    unbracketed(&unstyled(&text))
}

/// `<style>{Display}` → `Display`, nested. A brace no style opened stays:
/// a property's `{0}` is a format slot, not markup.
fn unstyled(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut open = 0usize;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if c == '<'
            && let Some(end) = rest.find('>')
            && rest[end + 1..].starts_with('{')
        {
            open += 1;
            rest = &rest[end + 2..];
            continue;
        }
        if c == '}' && open > 0 {
            open -= 1;
        } else {
            out.push(c);
        }
        rest = &rest[c.len_utf8()..];
    }
    out
}

/// `[Tag|Display]` → `Display`; `[Display]` → `Display` (S4).
fn unbracketed(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('[') {
        let Some(len) = rest[start..].find(']') else {
            break;
        };
        let inside = &rest[start + 1..start + len];
        out.push_str(&rest[..start]);
        out.push_str(inside.rsplit('|').next().unwrap_or(inside));
        rest = &rest[start + len + 1..];
    }
    out.push_str(rest);
    out
}

fn json_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

/// A whole number prints whole: `92`, never `92.0`.
fn whole_numbers<S: Serializer>(numbers: &[Option<f64>], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(numbers.iter().map(|n| match n.map(Number::from_f64) {
        None => Value::Null,
        Some(Number::Int(i)) => Value::from(i),
        Some(Number::Float(f)) => Value::from(f),
    }))
}
