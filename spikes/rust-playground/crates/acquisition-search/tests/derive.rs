//! The derivation's boundary (C103, C90): a body and its ingest facts in,
//! the item out. Bodies are shaped as GGG gives them (`search/item-facts/`)
//! with invented values; every expectation is worked by hand from the
//! body above it. The differential against the census over a real corpus
//! is M2 (`search/item-facts/scripts/m2-differential.py`), never in the gate.

use acquisition_search::{Facts, Item, Line, Part, Shown, Slot, derive};
use proptest::prelude::*;
use serde_json::{Value, json};

fn facts() -> Facts {
    Facts {
        id: "a1".into(),
        realm: "pc".into(),
        league: Some("Standard".into()),
        location_kind: "stash".into(),
        location_id: "tab1".into(),
        container: Some("items".into()),
        socketed_in: None,
        first_seen: 100,
        last_seen: 200,
        removed_at: None,
    }
}

fn item(body: Value) -> Item {
    derive(facts(), &body.to_string())
}

fn line<'a>(item: &'a Item, template: &str) -> &'a Line {
    let mut found = item.lines.iter().filter(|l| l.template == template);
    let line = found
        .next()
        .unwrap_or_else(|| panic!("no line {template:?} in {:#?}", item.lines));
    assert!(found.next().is_none(), "{template:?} occurs twice");
    line
}

fn ring() -> Value {
    json!({
        "id": "a1", "name": "Rune Loop", "typeLine": "Two-Stone Ring", "baseType": "Two-Stone Ring",
        "rarity": "Rare", "frameType": 2, "frameTypeId": "Rare", "ilvl": 84,
        "identified": true, "verified": false, "fractured": true,
        "league": "Hardcore", "realm": "poe2",
        "influences": {"shaper": true, "hunter": true}, "shaper": true,
        "note": "~price 1 divine", "flavourText": ["never searched"], "descrText": "nor this",
        "requirements": [
            {"name": "Level", "values": [["67", 0]], "displayMode": 0, "type": 62},
            {"name": "Str", "values": [["159", 0]], "displayMode": 1, "type": 63}
        ],
        "properties": [{"name": "Quality", "values": [["+20%", 1]], "displayMode": 0, "type": 6}],
        "implicitMods": [{"description": "+16% to Fire and Cold Resistances"}],
        "explicitMods": [
            {"description": "+92 to maximum Life", "flags": {"fractured": true}},
            {"description": "-27% to Chaos Resistance"},
            {"description": "+20 to maximum Life", "flags": {"crafted": true, "mutated": false}},
            {"description": "Adds 15 to 45 Cold Damage"}
        ]
    })
}

#[test]
fn the_header_and_the_fields_are_what_ggg_gives() {
    let ring = item(ring());
    assert_eq!(ring.name.as_deref(), Some("Rune Loop"));
    assert_eq!(ring.typeline.as_deref(), Some("Two-Stone Ring"));
    assert_eq!(ring.base.as_deref(), Some("Two-Stone Ring"));
    assert_eq!(ring.rarity.as_deref(), Some("Rare"));
    assert_eq!(ring.frame.as_deref(), Some("Rare"));
    assert_eq!(ring.ilvl, Some(84));
    assert_eq!(ring.item_level.as_deref(), Some("Item Level: 84"));
    assert_eq!(ring.stack, None);
    assert_eq!(ring.note.as_deref(), Some("~price 1 divine"));
    // every yes the body says, `influences` folded in, sorted; never a no
    assert_eq!(ring.flags, ["fractured", "hunter", "identified", "shaper"]);
    assert!(ring.unread.is_empty());
}

/// C103: place comes from the store's read, never from a body.
#[test]
fn place_is_never_read_from_the_body() {
    let ring = item(ring());
    assert_eq!(ring.facts, facts());
    assert_eq!(ring.facts.league.as_deref(), Some("Standard"));
    assert_eq!(ring.facts.realm, "pc");
}

/// C90: the template, the sign carried in the number, both coordinates of
/// the kind. C92: two occurrences of one template stay two.
#[test]
fn a_line_is_kind_template_numbers() {
    let ring = item(ring());
    let sources: Vec<&str> = ring.lines.iter().map(|l| l.source.as_str()).collect();
    assert_eq!(
        sources,
        ["explicit", "explicit", "explicit", "explicit", "implicit"]
    );

    let life: Vec<&Line> = ring
        .lines
        .iter()
        .filter(|l| l.template == "# to maximum Life")
        .collect();
    assert_eq!(life.len(), 2);
    assert_eq!(
        (life[0].numbers.as_slice(), &life[0].flags[..]),
        (&[Some(92.0)][..], &["fractured".to_string()][..])
    );
    assert_eq!(
        (life[1].numbers.as_slice(), &life[1].flags[..]),
        (&[Some(20.0)][..], &["crafted".to_string()][..])
    );
    assert_eq!(life[0].text, "+92 to maximum Life");
    assert_eq!(life[0].slot("arg1"), Slot::Is(92.0));
    assert_eq!(life[0].slot("arg2"), Slot::Absent);
    assert_eq!(life[0].slot("low"), Slot::Absent);

    let chaos = line(&ring, "#% to Chaos Resistance");
    assert_eq!(chaos.numbers, [Some(-27.0)]);
    let implicit = line(&ring, "#% to Fire and Cold Resistances");
    assert_eq!(
        (implicit.source.as_str(), implicit.numbers.as_slice()),
        ("implicit", &[Some(16.0)][..])
    );
    assert!(implicit.flags.is_empty());
}

/// The reference, *Slots*: exactly one `# to #` is ranged; a further number
/// is positional; two pairs are not ranged; `(#-#)` never reads `# to #`.
#[test]
fn the_ranged_rule_names_low_high_and_avg() {
    let it = item(json!({"typeLine": "Bow", "explicitMods": [
        "Adds 15 to 45 Cold Damage",
        "3 to 5 Added Physical Damage per 15 Armour or Evasion Rating on Shield",
        "Adds 1 to 2 Fire Damage and 3 to 4 Cold Damage",
        "Bow: Adds (11-12) to (20-23) Cold Damage",
        "Base duration is 1.60 seconds"
    ]}));
    let cold = line(&it, "Adds # to # Cold Damage");
    assert_eq!(
        (cold.slot("low"), cold.slot("high"), cold.slot("avg")),
        (Slot::Is(15.0), Slot::Is(45.0), Slot::Is(30.0))
    );
    assert_eq!(
        (cold.slot("arg1"), cold.slot("arg2")),
        (Slot::Is(15.0), Slot::Is(45.0))
    );
    let words: Vec<String> = cold.slots().into_iter().map(|(w, _)| w).collect();
    assert_eq!(words, ["low", "high", "avg", "arg1", "arg2"]);

    // the owner: "average is of the first two numbers"
    let shield = line(
        &it,
        "# to # Added Physical Damage per # Armour or Evasion Rating on Shield",
    );
    assert_eq!(
        (shield.slot("avg"), shield.slot("arg3")),
        (Slot::Is(4.0), Slot::Is(15.0))
    );

    let two = line(&it, "Adds # to # Fire Damage and # to # Cold Damage");
    assert_eq!(
        (two.slot("low"), two.slot("avg"), two.slot("arg4")),
        (Slot::Absent, Slot::Absent, Slot::Is(4.0))
    );

    let rolls = line(&it, "Bow: Adds (#-#) to (#-#) Cold Damage");
    assert_eq!(
        rolls.numbers,
        [Some(11.0), Some(12.0), Some(20.0), Some(23.0)]
    );
    assert_eq!(
        (rolls.slot("low"), rolls.slot("arg3")),
        (Slot::Absent, Slot::Is(20.0))
    );

    assert_eq!(line(&it, "Base duration is # seconds").numbers, [Some(1.6)]);
}

/// The reference, *Strings*: a mod over several rows is one occurrence with
/// one template, its row break `\n` whatever the body wrote; a phrase
/// tests each row on its own.
#[test]
fn a_mod_over_several_rows_is_one_occurrence() {
    let it = item(json!({"typeLine": "Map", "explicitMods": [
        {"description": "Monsters have 30% more Life\r\nMonsters cannot be Stunned"}
    ]}));
    assert_eq!(it.lines.len(), 1);
    let l = &it.lines[0];
    assert_eq!(
        l.template,
        "Monsters have #% more Life\nMonsters cannot be Stunned"
    );
    assert_eq!(l.numbers, [Some(30.0)]);
    assert_eq!(
        l.shown_rows().collect::<Vec<_>>(),
        ["Monsters have 30% more Life", "Monsters cannot be Stunned"]
    );
    let rows: Vec<&str> = it.displayed().map(|(_, row)| row).collect();
    assert_eq!(
        rows,
        [
            "Map",
            "Monsters have 30% more Life",
            "Monsters cannot be Stunned"
        ]
    );
}

/// C90, the build plan's 9c4: a mod displayed over several rows keeps each
/// row as a name of it, read on its own — Thread of Hope's, as the trade
/// site's capture R1 gives it, its resistance the second of three rows.
#[test]
fn c90_a_mod_of_several_rows_keeps_each_row_as_a_name() {
    let it = item(json!({"typeLine": "Crimson Jewel", "explicitMods": [
        {"description": "Only affects Passives in Small Ring"},
        {"description": "Passive Skills in Radius can be Allocated without being connected to your tree\n-17% to all Elemental Resistances\nPassage"}
    ]}));
    assert_eq!(it.lines.len(), 2);
    let (one, thread) = (&it.lines[0], &it.lines[1]);
    // a mod of one row has one name, its template
    assert!(one.rows.is_empty());
    assert_eq!(
        one.names().collect::<Vec<_>>(),
        ["Only affects Passives in Small Ring"]
    );
    assert_eq!(
        thread.names().collect::<Vec<_>>(),
        [
            "Passive Skills in Radius can be Allocated without being connected to your tree\n#% to all Elemental Resistances\nPassage",
            "Passive Skills in Radius can be Allocated without being connected to your tree",
            "#% to all Elemental Resistances",
            "Passage",
        ]
    );
    assert_eq!(thread.numbers, [Some(-17.0)]);
    let numbers: Vec<&[Option<f64>]> = thread.rows.iter().map(|r| &r.numbers[..]).collect();
    assert_eq!(numbers, [&[][..], &[Some(-17.0)][..], &[][..]]);
    assert_eq!(thread.rows[1].read().slot("arg1"), Slot::Is(-17.0));
    assert_eq!(thread.rows[0].read().slot("arg1"), Slot::Absent);
    // the rows are on the item as `show` prints it, and on no mod of one
    let shown = serde_json::to_value(&it).unwrap();
    assert_eq!(shown["lines"][0].get("rows"), None);
    assert_eq!(
        shown["lines"][1]["rows"][1],
        json!({"template": "#% to all Elemental Resistances", "numbers": [-17]})
    );
}

/// Rule 8 of the plan, at the row's grain: a row's number is its own —
/// the second row's first number is `arg1` of that row and `arg2` of the
/// mod — and a number unread in one row is unread of that row alone. A
/// row that displays nothing is no name.
#[test]
fn a_row_is_read_on_its_own() {
    let it = item(json!({"explicitMods": [
        "Adds 3 to 9 Cold Damage\n+40 to maximum Life",
        "+1.12345 to maximum Life\r\n\r\n+5% to Cold Resistance"
    ]}));
    let two = &it.lines[0];
    assert_eq!(two.slot("arg3"), Slot::Is(40.0));
    assert_eq!(two.slot("low"), Slot::Is(3.0));
    let life = two.rows[1].read();
    assert_eq!(
        (life.slot("arg1"), life.slot("arg2"), life.slot("low")),
        (Slot::Is(40.0), Slot::Absent, Slot::Absent)
    );
    assert_eq!(
        two.rows[0].read().slots(),
        [
            ("low".to_string(), Some(3.0)),
            ("high".to_string(), Some(9.0)),
            ("avg".to_string(), Some(6.0)),
            ("arg1".to_string(), Some(3.0)),
            ("arg2".to_string(), Some(9.0)),
        ]
    );
    let long = &it.lines[1];
    assert_eq!(long.template, "# to maximum Life\n\n#% to Cold Resistance");
    assert_eq!(long.numbers, [None, Some(5.0)]);
    assert_eq!(
        long.names().collect::<Vec<_>>(),
        [
            "# to maximum Life\n\n#% to Cold Resistance",
            "# to maximum Life",
            "#% to Cold Resistance"
        ]
    );
    assert_eq!(long.rows[0].read().slot("arg1"), Slot::Unread);
    assert_eq!(long.rows[1].read().slot("arg1"), Slot::Is(5.0));
    // said once, of the occurrence
    let unread: Vec<(&Part, Option<usize>)> = it.unread.iter().map(|u| (&u.part, u.line)).collect();
    assert_eq!(unread, [(&Part::Numbers("explicit".into()), Some(1))]);
}

/// Two rows of one mod displaying one template, in any case as `=`
/// compares one: a number named by that row would be either's, so it is
/// unread, said where it is read; the mod's own numbers stand, and so
/// does a row no other shares.
#[test]
fn two_rows_of_one_template_leave_that_rows_numbers_unread() {
    let it = item(json!({"explicitMods": [
        "+3 to maximum Life\n+7 to Maximum life\n+5% to Cold Resistance\nPassage\nPassage"
    ]}));
    let l = &it.lines[0];
    assert_eq!(l.numbers, [Some(3.0), Some(7.0), Some(5.0)]);
    let numbers: Vec<&[Option<f64>]> = l.rows.iter().map(|r| &r.numbers[..]).collect();
    assert_eq!(
        numbers,
        [&[None][..], &[None][..], &[Some(5.0)][..], &[][..], &[][..]]
    );
    assert_eq!(l.rows[1].read().slot("arg1"), Slot::Unread);
    assert_eq!(it.unread.len(), 1, "{:#?}", it.unread);
    assert_eq!(it.unread[0].part, Part::Numbers("explicit".into()));
    assert_eq!(it.unread[0].line, Some(0));
    assert_eq!(
        it.unread[0].problem,
        "`explicitMods[0]`: two rows of this mod display one template, `# to maximum Life`: a number named by that row is either's"
    );
    // a veiled line carries no number, and its rows none
    let veiled = item(json!({"veiledMods": ["Prefix01\nSuffix02"]}));
    assert!(veiled.lines[0].rows.iter().all(|r| r.numbers.is_empty()));
    assert!(veiled.unread.is_empty());
}

/// C90: markup reduced to what the player sees — S4's brackets, and the
/// styled braces of a divination card's reward. A property's `{0}` is a
/// format slot, never markup.
#[test]
fn markup_is_reduced_to_its_display_half() {
    let it = item(json!({
        "typeLine": "The [Card|Nurse]",
        "properties": [
            {"name": "Requires {1} (Level {0})", "values": [["2", 0], ["[Brute|Brute Force]", 0]], "displayMode": 3},
            {"name": "[Spirit]", "values": [["30", 0]], "displayMode": 0}
        ],
        "explicitMods": [
            {"description": "Rare Monsters have [ElementalThorns|Elemental Thorns]"},
            {"description": "<size:31>{<uniqueitem>{Ngamahu's Flame}}\r\n<default>{Item Level:} <normal>{100}"},
            {"description": "<currencyitem>{1,500x Vivid Crystallised Lifeforce}"}
        ]
    }));
    assert_eq!(it.typeline.as_deref(), Some("The Nurse"));
    let texts: Vec<&str> = it.properties.iter().map(|p| p.text.as_str()).collect();
    assert_eq!(texts, ["Requires Brute Force (Level 2)", "Spirit: 30"]);
    assert_eq!(it.properties[0].values, ["2", "Brute Force"]);
    assert_eq!(it.lines[0].text, "Rare Monsters have Elemental Thorns");
    assert_eq!(it.lines[1].text, "Ngamahu's Flame\nItem Level: 100");
    assert_eq!(it.lines[1].template, "Ngamahu's Flame\nItem Level: #");
    // a thousands comma is part of its number
    assert_eq!(it.lines[2].template, "#x Vivid Crystallised Lifeforce");
    assert_eq!(it.lines[2].numbers, [Some(1500.0)]);
}

/// Properties render by `displayMode` as the game shows them. Each
/// requirement is kept on its own, its name without a colon, and they are
/// displayed as one row (owner, 2026-09-19).
#[test]
fn properties_and_requirements_are_displayed_strings() {
    let mut body = ring();
    body["properties"] = json!([
        {"name": "Quality", "values": [["+20%", 1]], "displayMode": 0, "type": 6},
        {"name": "Sceptre", "values": [], "displayMode": 0},
        {"name": "Elemental Damage", "values": [["38-71", 4], ["57-108", 5]], "displayMode": 0, "type": 10},
        {"name": "", "values": [], "displayMode": 4},
        {"name": "Any Heist member can equip this item.", "values": [["", 0]], "displayMode": 3}
    ]);
    body["additionalProperties"] = json!([{"name": "Experience", "values": [["1/70", 0]], "displayMode": 2, "progress": 0.01, "type": 20}]);
    let ring = item(body);
    let shown: Vec<(&str, &str)> = ring
        .properties
        .iter()
        .map(|p| (p.array.as_str(), p.text.as_str()))
        .collect();
    assert_eq!(
        shown,
        [
            ("properties", "Quality: +20%"),
            ("properties", "Sceptre"),
            ("properties", "Elemental Damage: 38-71, 57-108"),
            ("properties", "Any Heist member can equip this item."),
            ("additionalProperties", "Experience: 1/70"),
        ]
    );
    let each: Vec<(&str, &str, &str)> = ring
        .requirements
        .iter()
        .map(|r| (r.name.as_str(), r.values[0].as_str(), r.text.as_str()))
        .collect();
    assert_eq!(
        each,
        [("Level", "67", "Level 67"), ("Str", "159", "159 Str")]
    );
    assert_eq!(ring.requires.as_deref(), Some("Requires Level 67, 159 Str"));
    // an item that requires nothing displays no row
    let gem = item(json!({"typeLine": "Portal", "requirements": []}));
    assert_eq!((gem.requires, gem.requirements.len()), (None, 0));
    assert!(ring.unread.is_empty());
}

/// The reference, *Item-level*: what a phrase is tested against — and the
/// flavour text, the description and the note are not.
#[test]
fn displayed_is_the_header_the_properties_and_every_line() {
    let ring = item(ring());
    let rows: Vec<&str> = ring.displayed().map(|(_, row)| row).collect();
    assert_eq!(
        rows,
        [
            "Rune Loop",
            "Two-Stone Ring",
            "Two-Stone Ring",
            "Quality: +20%",
            "Item Level: 84",
            "Requires Level 67, 159 Str",
            "+92 to maximum Life",
            "-27% to Chaos Resistance",
            "+20 to maximum Life",
            "Adds 15 to 45 Cold Damage",
            "+16% to Fire and Cold Resistances",
        ]
    );
    let wheres: Vec<Shown> = ring.displayed().map(|(w, _)| w).collect();
    assert_eq!(wheres[..3], [Shown::Name, Shown::Typeline, Shown::Base]);
    assert!(matches!(wheres[3], Shown::Property(p) if p.name == "Quality"));
    assert_eq!(wheres[4..6], [Shown::ItemLevel, Shown::Requires]);
    assert!(matches!(wheres[6], Shown::Line(l) if l.numbers == [Some(92.0)]));
}

/// The reference, *Item-level*: an item with no name lacks it — known
/// absence. A gem carries no rarity; the deriver reads and does not
/// interpret, so the frame sits beside it.
#[test]
fn an_empty_string_is_an_item_without_one() {
    let gem = item(json!({
        "name": "", "typeLine": "Vaal Discipline", "baseType": "Vaal Discipline",
        "frameTypeId": "Gem", "ilvl": 0, "identified": true, "corrupted": true, "support": false,
        "stackSize": 1
    }));
    assert_eq!(gem.name, None);
    assert_eq!(gem.rarity, None);
    assert_eq!(gem.frame.as_deref(), Some("Gem"));
    // GGG's 0: the game shows a gem no item level (owner, 2026-09-19)
    assert_eq!(gem.ilvl, None);
    assert_eq!(gem.item_level, None);
    assert_eq!(gem.stack, Some(1));
    assert_eq!(gem.flags, ["corrupted", "identified"]);
    assert!(gem.unread.is_empty());
}

/// A vaal gem's base skill sits under `hybrid` and is displayed on the
/// item: its lines are the source `hybrid`, its properties displayed
/// strings (owner, 2026-09-19).
#[test]
fn a_vaal_gems_base_skill_is_the_source_hybrid() {
    let gem = item(json!({
        "typeLine": "Vaal Discipline", "frameTypeId": "Gem",
        "properties": [{"name": "Level", "values": [["20 (Max)", 0]], "displayMode": 0, "type": 5}],
        "explicitMods": [{"description": "Base duration is 3.00 seconds"}],
        "hybrid": {
            "isVaalGem": true, "baseTypeName": "Discipline",
            "properties": [{"name": "Cooldown Time", "values": [["1.20 sec", 0]], "displayMode": 0}],
            "explicitMods": ["You and nearby allies gain 303 additional Energy Shield", 7]
        }
    }));
    let kinds: Vec<(&str, &str)> = gem
        .lines
        .iter()
        .map(|l| (l.source.as_str(), l.template.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("explicit", "Base duration is # seconds"),
            (
                "hybrid",
                "You and nearby allies gain # additional Energy Shield"
            ),
        ]
    );
    let shown: Vec<(&str, &str)> = gem
        .properties
        .iter()
        .map(|p| (p.array.as_str(), p.text.as_str()))
        .collect();
    assert_eq!(
        shown,
        [
            ("properties", "Level: 20 (Max)"),
            ("hybrid.properties", "Cooldown Time: 1.20 sec")
        ]
    );
    assert_eq!(gem.unread.len(), 1);
    assert_eq!(gem.unread[0].part, Part::Lines("hybrid".into()));
    assert_eq!(
        gem.unread[0].problem,
        "`hybrid.explicitMods[1]` is a number, not a line"
    );
    // a flag inside `hybrid` is the base skill's, never the item's
    assert!(gem.flags.is_empty());
}

/// Every `*Mods` array is a source, string elements as object ones —
/// except `ultimatumMods`, ids of what `explicitMods` already displays.
/// S12: a veiled line keeps its template and carries no number, so a value
/// query never matches it. An empty line displays nothing.
#[test]
fn sources_are_the_mods_arrays() {
    let it = item(json!({
        "typeLine": "Inscribed Ultimatum",
        "ultimatumMods": [{"type": "FrostInfection", "tier": 3}],
        "explicitMods": [{"description": "Blistering Cold III"}, {"description": ""}],
        "enchantMods": ["Allocates Arcane Potency"],
        "utilityMods": ["+40% to Fire Resistance"],
        "veiledMods": ["Suffix02"],
        "someNewMods": ["12% more of something GGG adds tomorrow"]
    }));
    let kinds: Vec<(&str, &str)> = it
        .lines
        .iter()
        .map(|l| (l.source.as_str(), l.template.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            ("enchant", "Allocates Arcane Potency"),
            ("explicit", "Blistering Cold III"),
            ("someNew", "#% more of something GGG adds tomorrow"),
            ("utility", "#% to Fire Resistance"),
            ("veiled", "Suffix#"),
        ]
    );
    let veiled = line(&it, "Suffix#");
    assert_eq!(veiled.text, "Suffix02");
    assert!(veiled.numbers.is_empty());
    assert_eq!(veiled.slot("arg1"), Slot::Absent);
    assert!(veiled.slots().is_empty());
    assert!(it.unread.is_empty());
}

/// C93: what could not be read is named by collection, the readable rest
/// is still derived — a readable line is a witness — and an unread body
/// leaves every part unread. C47: never a panic.
#[test]
fn what_could_not_be_read_is_named_by_collection() {
    let it = item(json!({
        "typeLine": "Ring", "ilvl": "84", "influences": [],
        "implicitMods": [{"description": "+16 to maximum Life"}],
        "explicitMods": [{"description": "+95 to maximum Life"}, 7, {"text": "?"}, {"description": "+1 to Level", "flags": 3}],
        "enchantMods": {"0": "not an array"},
        "properties": [{"name": "Quality", "values": [["+20%", 1]], "displayMode": 9}, {"name": "Armour", "values": [["100", 0]], "displayMode": 0}],
        "requirements": "Level 67"
    }));
    assert_eq!(it.typeline.as_deref(), Some("Ring"));
    assert_eq!(it.ilvl, None);
    // the witnesses
    let implicit: Vec<f64> = it
        .lines
        .iter()
        .filter(|l| l.source == "implicit")
        .flat_map(|l| l.numbers.iter().flatten().copied().collect::<Vec<f64>>())
        .collect();
    assert_eq!(implicit, [16.0]);
    let explicit: Vec<&str> = it
        .lines
        .iter()
        .filter(|l| l.source == "explicit")
        .map(|l| l.text.as_str())
        .collect();
    assert_eq!(explicit, ["+95 to maximum Life", "+1 to Level"]);
    assert_eq!(it.properties.len(), 1);
    assert_eq!(it.properties[0].text, "Armour: 100");

    let parts: Vec<&Part> = it.unread.iter().map(|u| &u.part).collect();
    assert_eq!(
        parts,
        [
            &Part::Field("ilvl".into()),
            &Part::Field("influences".into()),
            &Part::Properties("properties".into()),
            &Part::Properties("requirements".into()),
            // the array being no array, the Level row may be among what
            // was lost: `reqlevel` says so under its own name (step 6)
            &Part::Field("reqlevel".into()),
            &Part::Lines("enchant".into()),
            &Part::Lines("explicit".into()),
            &Part::Lines("explicit".into()),
            // the line was read and is a witness to its text; its flags
            // were not, and it says so itself
            &Part::Flags("explicit".into()),
        ]
    );
    let level = it.lines.iter().find(|l| l.text == "+1 to Level").unwrap();
    assert!(level.flags_unread && level.flags.is_empty());
    assert!(it.lines.iter().filter(|l| l.flags_unread).count() == 1);
    assert_eq!(
        it.unread[6].problem,
        "`explicitMods[1]` is a number, not a line"
    );
    assert_eq!(
        it.unread[4].problem,
        "`requirements` could not be read and may be the `Level` requirement"
    );
    assert_eq!(it.unread_in(&Part::Lines("explicit".into())).count(), 2);
    assert_eq!(it.unread_in(&Part::Flags("explicit".into())).count(), 1);
    assert_eq!(it.unread_in(&Part::Lines("implicit".into())).count(), 0);

    for body in ["not json", "[1, 2]", "", "null"] {
        let it = derive(facts(), body);
        assert_eq!(it.facts, facts(), "place survives an unread body");
        assert_eq!(it.unread.len(), 1);
        assert_eq!(it.unread[0].part, Part::Body);
        assert_eq!(it.unread_in(&Part::Lines("implicit".into())).count(), 1);
        assert!(it.lines.is_empty() && it.typeline.is_none());
    }
}

/// C101 (the build plan, step 8): the sockets are a collection of their
/// own — group and colour as GGG gives them, `[]` a present collection
/// with nothing in it, no key no collection — and what could not be read
/// is unread at the socket's grain: a group that is no whole number
/// leaves that socket's group open, an `attr` with no `sColour` or a
/// `sColour` that is no string that socket's colour, an element that is
/// no object may be any socket, and the array not being one is the
/// collection.
#[test]
fn sockets_are_a_collection_read_at_the_sockets_grain() {
    let plate = item(json!({"typeLine": "Plate", "sockets": [
        {"group": 0, "attr": "S", "sColour": "R"},
        {"group": 0, "attr": "D", "sColour": "G"},
        {"group": 1, "attr": "I", "sColour": "B"},
        {"group": 1, "attr": "G", "sColour": "W"},
        {"group": 2, "attr": "A", "sColour": "A"},
        {"group": 3, "attr": "DV", "sColour": "DV"}
    ]}));
    let sockets = plate.sockets.as_deref().unwrap();
    assert_eq!(sockets.len(), 6);
    assert_eq!(
        sockets
            .iter()
            .map(|s| (s.group, s.colour.as_deref()))
            .collect::<Vec<_>>(),
        [
            (Some(0), Some("R")),
            (Some(0), Some("G")),
            (Some(1), Some("B")),
            (Some(1), Some("W")),
            (Some(2), Some("A")),
            (Some(3), Some("DV"))
        ]
    );
    assert!(sockets.iter().all(|s| !s.colour_unread && s.kind.is_none()));
    assert!(plate.unread.is_empty());

    // poe2: a socket with a type and no colour, known
    let bow = item(
        json!({"typeLine": "Bow", "sockets": [{"group": 0, "type": "gem"}, {"group": 1, "type": "rune"}]}),
    );
    let sockets = bow.sockets.as_deref().unwrap();
    assert_eq!(sockets[0].colour, None);
    assert!(!sockets[0].colour_unread);
    assert_eq!(sockets[1].kind.as_deref(), Some("rune"));
    assert!(bow.unread.is_empty());

    // GGG's `[]` (N44) is a collection with no socket; no key is none
    assert_eq!(item(json!({"sockets": []})).sockets, Some(Vec::new()));
    assert_eq!(item(json!({})).sockets, None);

    // each hole at its grain
    let holes = item(json!({"sockets": [
        {"group": "one", "attr": "S", "sColour": "R"},
        {"attr": "S", "sColour": "R"},
        {"group": 1, "attr": "S"},
        {"group": 1, "attr": "S", "sColour": 7},
        "socket",
        {"group": 2, "sColour": "B"}
    ]}));
    let sockets = holes.sockets.as_deref().unwrap();
    assert_eq!(sockets.len(), 5);
    assert_eq!(
        (sockets[0].group, sockets[0].colour.as_deref()),
        (None, Some("R"))
    );
    assert_eq!(
        (sockets[1].group, sockets[1].colour.as_deref()),
        (None, Some("R"))
    );
    assert_eq!(
        (sockets[2].group, sockets[2].colour_unread),
        (Some(1), true)
    );
    assert_eq!(
        (sockets[3].group, sockets[3].colour_unread),
        (Some(1), true)
    );
    assert_eq!(
        (sockets[4].group, sockets[4].colour.as_deref()),
        (Some(2), Some("B"))
    );
    let unread: Vec<(String, Option<usize>, &str)> = holes
        .unread
        .iter()
        .map(|u| {
            (
                serde_json::to_value(&u.part).unwrap()["part"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                u.socket,
                u.problem.as_str(),
            )
        })
        .collect();
    assert_eq!(
        unread,
        [
            (
                "socket_group".to_string(),
                Some(0),
                "`sockets[0]`: `group` is a string, not a whole number"
            ),
            (
                "socket_group".to_string(),
                Some(1),
                "`sockets[1]` has no `group`"
            ),
            (
                "socket_colour".to_string(),
                Some(2),
                "`sockets[2]` has an `attr` and no `sColour`: the colour is not read from the attribute"
            ),
            (
                "socket_colour".to_string(),
                Some(3),
                "`sockets[3]`: `sColour` is a number, not a colour"
            ),
            (
                "sockets".to_string(),
                None,
                "`sockets[4]` is a string, not a socket"
            ),
        ]
    );
    // the collection itself
    let none = item(json!({"sockets": "R-R-G"}));
    assert_eq!(none.sockets, None);
    assert_eq!(none.unread.len(), 1);
    assert_eq!(none.unread_in(&Part::Sockets).count(), 1);
    assert_eq!(
        none.unread[0].problem,
        "`sockets` is a string, not an array"
    );
}

/// The item's JSON form, which `acq show --json` will print (C53): a whole
/// number prints whole, and an unread entry names its part.
#[test]
fn the_json_form() {
    let it = item(json!({
        "typeLine": "Ring", "frameTypeId": "Magic", "ilvl": 3, "identified": true,
        "explicitMods": [{"description": "Adds 1.5 to 4 Cold Damage", "flags": {"crafted": true}}, 7]
    }));
    assert_eq!(
        serde_json::to_value(&it).unwrap(),
        json!({
            "facts": {
                "id": "a1", "realm": "pc", "league": "Standard", "location_kind": "stash", "location_id": "tab1",
                "container": "items", "socketed_in": null, "first_seen": 100, "last_seen": 200, "removed_at": null
            },
            "name": null, "typeline": "Ring", "base": null, "rarity": null, "frame": "Magic",
            "ilvl": 3, "item_level": "Item Level: 3", "stack": null, "reqlevel": null, "note": null,
            "flags": ["identified"],
            "properties": [], "requirements": [], "requires": null,
            "lines": [{
                "source": "explicit", "flags": ["crafted"], "template": "Adds # to # Cold Damage",
                "numbers": [1.5, 4], "text": "Adds 1.5 to 4 Cold Damage"
            }],
            "unread": [{"part": "lines", "of": "explicit", "problem": "`explicitMods[1]` is a number, not a line"}]
        })
    );
    let unread = derive(facts(), "{");
    assert_eq!(
        serde_json::to_value(&unread.unread[0]).unwrap()["part"],
        "body"
    );
}

fn any_json() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::from),
        any::<i64>().prop_map(Value::from),
        any::<f64>().prop_map(Value::from),
        ".{0,12}".prop_map(Value::from),
        "[<>{}\\[\\]|#+\\-0-9,. a-z\r\n]{0,24}".prop_map(Value::from),
    ];
    leaf.prop_recursive(4, 48, 6, |inner| {
        let key = prop_oneof![
            Just("explicitMods".to_string()),
            Just("properties".to_string()),
            Just("requirements".to_string()),
            Just("description".to_string()),
            Just("flags".to_string()),
            Just("name".to_string()),
            Just("values".to_string()),
            Just("displayMode".to_string()),
            Just("influences".to_string()),
            Just("ilvl".to_string()),
            "[a-zA-Z]{1,8}",
        ];
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..5).prop_map(Value::from),
            prop::collection::vec((key, inner), 0..6)
                .prop_map(|kv| Value::Object(kv.into_iter().collect())),
        ]
    })
}

proptest! {
    /// A mod's rows read apart are the mod read whole (C90): the rows'
    /// templates are the whole's between its row breaks, and their numbers
    /// the whole's in order — but a row whose template another shares,
    /// whose numbers are unread.
    #[test]
    fn the_rows_of_a_mod_are_the_mod_read_apart(
        text in "([+\\-]?[0-9]{1,3}([,.][0-9]{1,3})?|[#%a-c ]|\r?\n){0,14}"
    ) {
        let it = item(json!({"explicitMods": [text]}));
        for l in &it.lines {
            if !l.text.contains('\n') {
                prop_assert!(l.rows.is_empty());
                continue;
            }
            let apart: Vec<&str> = l.template.split('\n').filter(|r| !r.is_empty()).collect();
            let rows: Vec<&str> = l.rows.iter().map(|r| r.template.as_str()).collect();
            prop_assert_eq!(&rows, &apart);
            let mut rest = &l.numbers[..];
            for row in &l.rows {
                let (own, after) = rest.split_at(row.numbers.len());
                rest = after;
                let shared = l.rows.iter().filter(|r| r.template.eq_ignore_ascii_case(&row.template)).count() > 1;
                if shared {
                    prop_assert!(row.numbers.iter().all(Option::is_none));
                } else {
                    prop_assert_eq!(&row.numbers[..], own);
                }
            }
            prop_assert!(rest.is_empty());
        }
    }

    /// C47: the deriver is total — any text and any JSON is an item — and
    /// a line's numbers are its template's `#`s wherever it names a slot.
    #[test]
    fn no_body_panics_the_deriver(text in ".{0,64}", body in any_json()) {
        let _ = derive(facts(), &text);
        let it = derive(facts(), &body.to_string());
        for l in &it.lines {
            let _ = l.shown_rows().count();
            for (word, n) in l.slots() {
                prop_assert_eq!(l.slot(&word), n.map_or(Slot::Unread, Slot::Is));
            }
            for row in &l.rows {
                for (word, n) in row.read().slots() {
                    prop_assert_eq!(row.read().slot(&word), n.map_or(Slot::Unread, Slot::Is));
                }
            }
        }
        prop_assert!(serde_json::to_value(&it).is_ok());
        let _ = it.displayed().count();
    }
}
