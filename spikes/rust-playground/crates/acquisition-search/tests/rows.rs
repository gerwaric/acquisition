//! A row of a mod displayed over several rows (the build plan, step 9c4;
//! C90, C92, C93, C97), through the crate's boundary: an exact template
//! names a mod by its whole text or by a row of it, the mod stays one
//! occurrence, a slot word names the named row's numbers, and what is
//! unread of one row is unread of that row. A total's row names a row the
//! same way, which `tests/pseudo.rs` holds on the trade site's captures.
//! Every count is worked by hand from the fixture below, and every route
//! followed.

mod common;

use std::collections::BTreeSet;

use acquisition_search::{
    Collection, ErrorKind, Member, Node, Number, Op, Request, Value as TreeValue, answer, check,
};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

const THREAD: &str = "Passive Skills in Radius can be Allocated without being connected to your tree\n#% to all Elemental Resistances\nPassage";
/// As a query writes it: a row break is `\n` inside the quotes.
const THREAD_TYPED: &str = r#""Passive Skills in Radius can be Allocated without being connected to your tree\n#% to all Elemental Resistances\nPassage""#;
const ALL_RES: &str = "#% to all Elemental Resistances";
const LIFE: &str = "# to maximum Life";

/// Seven items in pc Standard, `Rows` (w1), every line explicit.
///
/// `thread`, Thread of Hope as the trade site's capture gives it: one row
/// alone, then a mod of three rows whose second is -17 to all elemental;
/// `ring`, 12 to all elemental and 30 life; `taken`, a sentence wrapped
/// over two rows, 36 in the second; `pair`, a crafted mod of two rows,
/// cold damage 3 to 9 and 40 life, and 25 life alone; `long`, a mod of two
/// rows, life with five decimals and 5 cold resistance; `twice`, a mod
/// whose two rows are life, 3 and 7, GGG's two spellings of it; `plain`,
/// 95 life.
fn stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("w1", "Rows")]), 10);
    let rare = |id: &str, mods: Value| {
        item(
            id,
            &format!("Item {id}"),
            "Iron Ring",
            "Rare",
            json!({ "explicitMods": mods }),
        )
    };
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "w1",
        "Rows",
        vec![
            item(
                "thread",
                "Thread of Hope",
                "Crimson Jewel",
                "Unique",
                json!({ "explicitMods": [
                    "Only affects Passives in Small Ring",
                    "Passive Skills in Radius can be Allocated without being connected to your tree\n-17% to all Elemental Resistances\nPassage",
                ] }),
            ),
            rare(
                "ring",
                json!(["+12% to all Elemental Resistances", "+30 to maximum Life"]),
            ),
            rare(
                "taken",
                json!([
                    "Enemies you Kill that are affected by Elemental Ailments\ngrant 36% increased Flask Charges"
                ]),
            ),
            rare(
                "pair",
                json!([
                    { "description": "Adds 3 to 9 Cold Damage\n+40 to maximum Life", "flags": { "crafted": true } },
                    "+25 to maximum Life",
                ]),
            ),
            rare(
                "long",
                json!(["+1.12345 to maximum Life\n+5% to Cold Resistance"]),
            ),
            rare("twice", json!(["+3 to maximum Life\n+7 to Maximum life"])),
            rare("plain", json!(["+95 to maximum Life"])),
        ],
        20,
    );
    s
}

fn asked(s: &Store, text: &str) -> Value {
    as_json(&ask(&load(s, Some("pc")), text).unwrap())
}

fn found(s: &Store, text: &str) -> Vec<String> {
    ids(&asked(s, text))
}

fn view(s: &Store, text: &str, view: Value) -> Value {
    let request: Request =
        serde_json::from_value(json!({ "query": { "text": text }, "view": view })).unwrap();
    as_json(&answer(&load(s, Some("pc")), &request).unwrap())
}

fn counts(term: &Value) -> [u64; 4] {
    ["matched", "failed", "lacked", "undecided"].map(|k| term[k]["count"].as_u64().unwrap())
}

/// The ids a count's route returns, which must be as many as it counted.
fn follow(s: &Store, count: &Value) -> Vec<String> {
    let mut route = count["request"].clone();
    route["view"]["rows"]["limit"] = json!(100);
    let request = serde_json::from_value(route).unwrap();
    let routed = as_json(&answer(&load(s, Some("pc")), &request).unwrap());
    assert_eq!(routed["total"]["matched"], count["count"], "{count}");
    ids(&routed)
}

fn set(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|id| id.to_string()).collect()
}

fn bucket<'a>(table: &'a Value, value: &str) -> &'a Value {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["bucket"] == "value" && b["value"] == value)
        .unwrap_or_else(|| panic!("no bucket `{value}` in {table}"))
}

/// A table's buckets as (label, count), in the order printed — the
/// computed values a narrowing matches apart, which `tests/pseudo.rs`
/// pins.
fn shape(table: &Value) -> Vec<(String, u64)> {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["bucket"] != "computed")
        .map(|b| {
            let label = match b["bucket"].as_str().unwrap() {
                "value" => b["value"].as_str().unwrap().to_string(),
                other => format!("({other})"),
            };
            (label, b["count"].as_u64().unwrap())
        })
        .collect()
}

fn shaped(pairs: &[(&str, u64)]) -> Vec<(String, u64)> {
    pairs.iter().map(|(l, n)| (l.to_string(), *n)).collect()
}

/// C90 (owner, 2026-09-27, of a line's term as of a total: "From my side
/// the answer is yes."): an exact template names a mod by a row of it, in
/// any case, and its whole text still names it; `template:` and
/// `template~` test the whole text, across its rows. Naming is the
/// occurrence's, so a not of a row's template is a no on the mod.
#[test]
fn c90_an_exact_template_names_a_mod_by_its_whole_text_or_by_a_row() {
    let s = stash();
    let found = |text: &str| found(&s, text);
    assert_eq!(found(&format!("line(\"{ALL_RES}\")")), ["ring", "thread"]);
    assert_eq!(found(&format!("\"{ALL_RES}\"<0")), ["thread"]);
    assert_eq!(found(&format!("\"{ALL_RES}\">=10")), ["ring"]);
    assert_eq!(found(&format!("line({THREAD_TYPED})")), ["thread"]);
    assert_eq!(found(&format!("line({THREAD_TYPED} arg1=-17)")), ["thread"]);
    // a row without a number is a name too, and a wrapped sentence's row
    assert_eq!(found("line(template=\"PASSAGE\")"), ["thread"]);
    assert_eq!(found("\"grant #% increased Flask Charges\">=36"), ["taken"]);
    assert_eq!(
        found("line(\"Enemies you Kill that are affected by Elemental Ailments\")"),
        ["taken"]
    );
    // two rows are no name: a name is the whole text or one row
    assert_eq!(
        found(
            "line(\"Passive Skills in Radius can be Allocated without being connected to your tree\\n#% to all Elemental Resistances\")"
        ),
        Vec::<String>::new()
    );
    // the item has no such line, by a row as by the whole text
    assert_eq!(
        found(&format!("-\"{ALL_RES}\"")),
        ["long", "pair", "plain", "taken", "twice"]
    );
    // a line of resistance that is not that one: the mod it names is out
    assert_eq!(
        found(&format!("line(template:resistance -\"{ALL_RES}\")")),
        ["long"]
    );
    // `:` and `~` reach across the rows of the whole text, and a pattern's
    // ends are the whole text's
    assert_eq!(found("line(template:\"Life\\n#% to Cold\")"), ["long"]);
    assert_eq!(found("line(template~\"^#% to all\")"), ["ring"]);

    // the terms block, every count routed: ring, pair, long, twice and
    // plain carry a life line; pair's row is 40 and plain's line 95; ring's
    // is 30; long's and twice's numbers are unread
    let a = asked(&s, &format!("\"{LIFE}\">=40"));
    let term = &a["terms"][0];
    assert_eq!(counts(term), [2, 1, 2, 2]);
    assert_eq!(follow(&s, &term["matched"]), ["pair", "plain"]);
    assert_eq!(follow(&s, &term["failed"]), ["ring"]);
    assert_eq!(follow(&s, &term["lacked"]), ["taken", "thread"]);
    assert_eq!(follow(&s, &term["undecided"]), ["long", "twice"]);
}

/// The reference, *Strings* (owner, 2026-09-19: "it makes sense they are
/// a single occurrence"): a mod named by a row is one occurrence — its
/// source and its flags are its rows', a row shows it whole, and it is
/// shown once however many of its rows a term names.
#[test]
fn a_mod_named_by_a_row_is_one_occurrence() {
    let s = stash();
    let found = |text: &str| found(&s, text);
    assert_eq!(found(&format!("line(\"{LIFE}\" is:crafted)")), ["pair"]);
    assert_eq!(
        found("line(\"Adds # to # Cold Damage\" is:crafted source=explicit)"),
        ["pair"]
    );
    assert_eq!(
        found(&format!("line(\"{LIFE}\" source=implicit)")),
        Vec::<String>::new()
    );
    // two templates, two rows of the one mod
    assert_eq!(
        found(&format!("line(\"Adds # to # Cold Damage\" \"{LIFE}\")")),
        ["pair"]
    );
    let a = asked(&s, &format!("line(\"{LIFE}\")"));
    let row = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "pair")
        .unwrap();
    assert_eq!(
        row["matched"][0]["shows"],
        json!([
            { "line": { "source": "explicit", "flags": ["crafted"],
                "text": "Adds 3 to 9 Cold Damage\n+40 to maximum Life" } },
            { "line": { "source": "explicit", "flags": [], "text": "+25 to maximum Life" } },
        ])
    );
    let twice = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "twice")
        .unwrap();
    assert_eq!(twice["matched"][0]["shows"].as_array().unwrap().len(), 1);
}

/// C92, the reference's *Slots*: named by a row, a slot word names that
/// row's numbers — `arg1` is the row's first number, whatever the rows
/// before it display — and named by its whole text, the mod's in order.
/// A comparison, a sum, a largest and the together count read the same
/// number.
#[test]
fn c92_a_slot_names_the_numbers_of_the_row_named() {
    let s = stash();
    let found = |text: &str| found(&s, text);
    // pair's life is the second row's first number, the mod's third
    assert_eq!(found(&format!("\"{LIFE}\"=40")), ["pair"]);
    assert_eq!(found(&format!("\"{LIFE}\"=3")), Vec::<String>::new());
    assert_eq!(
        found("line(\"Adds # to # Cold Damage\" low>=3 high<=9)"),
        ["pair"]
    );
    assert_eq!(found("\"Adds # to # Cold Damage\".avg=6"), ["pair"]);
    let whole = "\"Adds # to # Cold Damage\\n# to maximum Life\"";
    assert_eq!(found(&format!("line({whole} arg3=40 low=3)")), ["pair"]);
    // a slot is checked against the template typed: a row of one number
    // has no third
    let e = ask(&load(&s, Some("pc")), &format!("line(\"{LIFE}\" arg3=40)"))
        .unwrap_err()
        .to_json();
    assert_eq!(e["kind"], "slot_unknown", "{e}");
    // two rows named together name no one row, and a number beside them
    // is refused (`c92_a_number_beside_two_rows_…`, below)
    let both = format!("\"Adds # to # Cold Damage\" \"{LIFE}\"");
    let e = ask(&load(&s, Some("pc")), &format!("line({both} arg3=40)"))
        .unwrap_err()
        .to_json();
    assert_eq!(e["kind"], "slot_of_two_rows", "{e}");
    // under an or the row a template names on the occurrence: pair's life
    // row, plain's line; long's two rows are both named, and its first
    // number is unread
    let either = format!("line((\"{LIFE}\" or \"#% to Cold Resistance\") arg1>=40)");
    let a = asked(&s, &either);
    assert_eq!(ids(&a), ["pair", "plain"]);
    assert_eq!(follow(&s, &a["terms"][0]["undecided"]), ["long", "twice"]);

    // the item's sum: 40 and 25
    assert_eq!(found(&format!("sum(\"{LIFE}\")=65")), ["pair"]);
    // together: neither of pair's reaches 60 and the two do; ring's 30
    // does not
    let a = asked(&s, &format!("\"{LIFE}\">=60"));
    let term = &a["terms"][0];
    assert_eq!(counts(term), [1, 2, 2, 2]);
    assert_eq!(follow(&s, &term["failed"]), ["pair", "ring"]);
    assert_eq!(term["together"]["count"], 1);
    assert_eq!(follow(&s, &term["together"]), ["pair"]);

    // the largest: plain 95, pair's row 40, ring 30; long's and twice's
    // are not established, and the two others have none
    let sorted = view(
        &s,
        "",
        json!({ "rows": { "sort": format!("line(\"{LIFE}\").arg1"), "desc": true } }),
    );
    let rows: Vec<(&str, &Value, &Value)> = sorted["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap(),
                &r["sort"]["value"],
                &r["sort"]["status"],
            )
        })
        .collect();
    assert_eq!(
        rows[..3],
        [
            ("plain", &json!(95), &Value::Null),
            ("pair", &json!(40), &Value::Null),
            ("ring", &json!(30), &Value::Null),
        ]
    );
    let open: BTreeSet<String> = rows
        .iter()
        .filter(|(_, _, status)| **status == "incomplete")
        .map(|(id, _, _)| id.to_string())
        .collect();
    assert_eq!(open, set(&["long", "twice"]));
    assert_eq!(
        sorted["rows"][1]["sort"]["shows"][0]["line"]["text"],
        "Adds 3 to 9 Cold Damage\n+40 to maximum Life"
    );
}

/// C91, C92 (the outside review of step 9c4, finding 1): a comparison
/// reads the numbers of the part the quoted templates conjoined with it
/// name, so an alternative beside it — one that holds nowhere, one that
/// names another row, one with no template — changes no number it reads.
/// The group is its alternatives spread out, each read on its own, as the
/// same alternatives are at the item's level.
#[test]
fn c92_an_alternative_changes_no_number_a_comparison_reads() {
    let s = stash();
    let found = |text: &str| found(&s, text);
    let life = format!("\"{LIFE}\" arg1=40");
    assert_eq!(found(&format!("line({life})")), ["pair"]);
    for beside in [
        "arg1=999",
        "\"Adds # to # Cold Damage\" source=implicit",
        "\"Adds # to # Cold Damage\" -\"Adds # to # Cold Damage\"",
        "\"#% to Cold Resistance\" arg1=999",
        "false()",
    ] {
        assert_eq!(
            found(&format!("line(({life}) or ({beside}))")),
            ["pair"],
            "{beside}"
        );
        assert_eq!(
            found(&format!("line(({beside}) or ({life}))")),
            ["pair"],
            "{beside}"
        );
        // as the two are at the item's level
        assert_eq!(
            found(&format!("line({life}) or line({beside})")),
            ["pair"],
            "{beside}"
        );
    }
    // each alternative reads its own part: the mod's first number is 3,
    // its cold damage row's 3 to 9, its life row's 40 — and twice's mod,
    // read in order, begins with 3 too
    assert_eq!(
        found(&format!("line((\"{LIFE}\" arg1=41) or arg1=3)")),
        ["pair", "twice"]
    );
    assert_eq!(
        found(&format!(
            "line((\"{LIFE}\" arg1=41) or (\"Adds # to # Cold Damage\" high=9))"
        )),
        ["pair"]
    );
    assert_eq!(
        found(&format!("line((\"{LIFE}\" arg1=3) or arg1=40)")),
        Vec::<String>::new()
    );
    // a not of two together is either's not, and of either is both's:
    // thread's resistance is -17 and explicit, ring's 12
    assert_eq!(
        found(&format!("line(\"{ALL_RES}\" -(arg1=-17 source=implicit))")),
        ["ring", "thread"]
    );
    assert_eq!(
        found(&format!(
            "line(\"{ALL_RES}\" -(arg1=-17 or source=implicit))"
        )),
        ["ring"]
    );
    // a template beside the alternatives is conjoined with each
    assert_eq!(
        found(&format!(
            "line(\"{LIFE}\" (arg1=40 or arg1=999) is:crafted)"
        )),
        ["pair"]
    );
    // a sum and a largest read each part an alternative names: pair's mod
    // is 3 and 40 under the two, and its own life line 25
    assert_eq!(
        found(&format!(
            "sum(line(\"{LIFE}\" or \"Adds # to # Cold Damage\").arg1)=68"
        )),
        ["pair"]
    );
    let sorted = view(
        &s,
        "id:pair",
        json!({ "rows": { "sort": format!("line(\"Adds # to # Cold Damage\" or \"{LIFE}\").arg1") } }),
    );
    assert_eq!(sorted["rows"][0]["sort"]["value"], 40);
}

const COLD: &str = "Adds # to # Cold Damage";
const FIRE: &str = "Adds # to # Fire Damage";
/// As a query writes it, `one`'s mod by its whole text.
const COLD_LIFE_TYPED: &str = r#""Adds # to # Cold Damage\n# to maximum Life""#;

/// Three items of one mod each, in pc Standard, `Both` (b1): `one`, cold
/// damage 3 to 9 then 40 life; `other`, 40 life then cold damage 3 to 9;
/// `ranged`, cold damage 3 to 9 then fire damage 5 to 7.
fn both_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("b1", "Both")]), 10);
    let one = |id: &str, line: &str| {
        item(
            id,
            &format!("Item {id}"),
            "Iron Ring",
            "Rare",
            json!({ "explicitMods": [line] }),
        )
    };
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "b1",
        "Both",
        vec![
            one("one", "Adds 3 to 9 Cold Damage\n+40 to maximum Life"),
            one("other", "+40 to maximum Life\nAdds 3 to 9 Cold Damage"),
            one("ranged", "Adds 3 to 9 Cold Damage\nAdds 5 to 7 Fire Damage"),
        ],
        20,
    );
    s
}

/// C91, C92 (owner, 2026-09-28: "refuse"; the outside review of step
/// 9c4, its fourth look): a number word beside two rows of a mod named
/// together has no one row to read, and which is never guessed — the
/// term is refused however it is spelled, as a text and as a tree, and
/// each reading offered is one this build answers. Read of the mod's
/// numbers in order, as it was, a row named as a condition moved what
/// the number beside it read: `one` left `"# to maximum Life" arg1>=40`
/// when the cold damage it shows was named, and a not of cold damage's
/// `high` held on `ranged`, whose `high` is 9.
#[test]
fn c92_a_number_beside_two_rows_named_together_is_refused() {
    let s = both_stash();
    let c = load(&s, Some("pc"));
    let found = |text: &str| found(&s, text);
    let none = Vec::<String>::new();
    let refused = |text: &str| -> Value {
        let e = match ask(&c, text) {
            Ok(_) => panic!("`{text}` is answered"),
            Err(e) => e.to_json(),
        };
        assert_eq!(e["kind"], "slot_of_two_rows", "`{text}`: {e}");
        // a reading is a text this build answers
        for reading in e["readings"].as_array().into_iter().flatten() {
            let reading = reading.as_str().unwrap();
            if let Err(e) = ask(&c, reading) {
                panic!("`{text}` offers `{reading}`, which is refused: {e}");
            }
        }
        e
    };
    let (life, cold, fire) = (
        format!("\"{LIFE}\""),
        format!("\"{COLD}\""),
        format!("\"{FIRE}\""),
    );
    // in any order, under any parentheses, beside an alternative that
    // holds nowhere or one that names the same row again
    for text in [
        format!("line({life} {cold} arg1>=40)"),
        format!("line({cold} arg1>=40 {life})"),
        format!("line(({life} arg1>=40) {cold})"),
        format!("line({life} ({cold} or false()) arg1>=40)"),
        format!("line({life} ({life} or {cold}) arg1=3)"),
        format!("line(({life} {cold} arg1=3) or {fire})"),
        // a doubled not is none, and names what it holds
        format!("line(--{life} {cold} arg1>=40)"),
        // a not is read with what is conjoined with it
        format!("line({fire} -({cold} high=9))"),
        format!("line({fire} -(is:crafted or -({cold} high=9)))"),
        // a value, compared or asked
        format!("sum(line({life} {cold}).arg1)>=1"),
        format!("line({life} {cold}).arg1>=40"),
        format!("undecided(line({life} {cold}).arg1)"),
        format!("-line({life} {cold} arg1>=40) rarity=rare"),
    ] {
        let e = refused(&text);
        assert_eq!(e["readings"].as_array().map(Vec::len), Some(2), "{text}");
    }
    // what is offered: the row whose number is meant stays quoted, and
    // the other is asked of the mod's text
    let e = refused(&format!("line({life} {cold} arg1>=40)"));
    assert_eq!(
        e["readings"],
        json!([
            format!("line({life} template:{cold} arg1>=40)"),
            format!("line(template:{life} {cold} arg1>=40)"),
        ])
    );
    assert!(
        e["error"].as_str().unwrap().contains(&life)
            && e["error"].as_str().unwrap().contains(&cold),
        "{e}"
    );
    assert_eq!(
        found(&format!("line({life} template:{cold} arg1>=40)")),
        ["one", "other"]
    );
    assert_eq!(
        found(&format!("line(template:{life} {cold} arg1>=40)")),
        none
    );
    // and under the not: fire's high is 7, cold's is 9
    let e = refused(&format!("line({fire} -({cold} high=9))"));
    assert_eq!(
        e["readings"],
        json!([
            format!("line({fire} -(template:{cold} high=9))"),
            format!("line(template:{fire} -({cold} high=9))"),
        ])
    );
    assert_eq!(
        found(&format!("line({fire} -(template:{cold} high=9))")),
        ["ranged"]
    );
    assert_eq!(
        found(&format!("line(template:{fire} -({cold} high=9))")),
        none
    );
    // a reading the build would refuse is not offered: with life asked
    // of the text, fire and cold are two rows still
    let e = refused(&format!("line(({life} or {fire}) {cold} arg1>=1)"));
    assert_eq!(
        e["readings"],
        json!([format!("line(({life} or {fire}) template:{cold} arg1>=1)")])
    );
    // what a row is sorted by
    let sort = format!("line({life} {cold}).arg1");
    let request: Request = serde_json::from_value(
        json!({ "query": { "text": "" }, "view": { "rows": { "sort": sort } } }),
    )
    .unwrap();
    let e = match answer(&c, &request) {
        Ok(_) => panic!("`{sort}` sorts"),
        Err(e) => e.to_json(),
    };
    assert_eq!(e["kind"], "slot_of_two_rows", "{e}");
    assert_eq!(
        e["readings"],
        json!([
            format!("line({life} template:{cold}).arg1"),
            format!("line(template:{life} {cold}).arg1"),
        ])
    );
    // a tree is refused as its text is
    let test = |attr: &str, op: Op, value: TreeValue| Member::Test {
        attr: attr.to_string(),
        op,
        value,
    };
    let tree = Node::Members {
        of: Collection::Lines,
        where_: Box::new(Member::All(vec![
            test("template", Op::Eq, TreeValue::Text(LIFE.to_string())),
            test("template", Op::Eq, TreeValue::Text(COLD.to_string())),
            test("arg1", Op::Ge, TreeValue::Number(Number::Int(40))),
        ])),
    };
    let e = check(&tree).unwrap_err();
    assert_eq!(e.kind, ErrorKind::SlotOfTwoRows);
    assert_eq!(
        e.readings,
        [
            format!("line({life} template:{cold} arg1>=40)"),
            format!("line(template:{life} {cold} arg1>=40)"),
        ]
    );
}

/// C92, the reference's *Slots* (the same look, finding 1): what is not
/// refused is read as it was — two rows named with no number beside
/// them, alternatives each naming its own row, a row asked of the mod's
/// text, a not of a name, a mod named by its whole text beside a row of
/// it — and a slot is checked against the one quoted template among the
/// group's conjuncts where that template alone says what the slot reads:
/// beside the mod's whole text the numbers are the mod's, and the check
/// that took `arg3` for the row's refused a term the evaluator reads.
#[test]
fn c92_a_slot_is_checked_against_the_template_that_says_what_it_reads() {
    let s = both_stash();
    let found = |text: &str| found(&s, text);
    let none = Vec::<String>::new();
    let (life, cold, fire) = (
        format!("\"{LIFE}\""),
        format!("\"{COLD}\""),
        format!("\"{FIRE}\""),
    );
    assert_eq!(found(&format!("line({life} {cold})")), ["one", "other"]);
    assert_eq!(
        found(&format!("line(({life} or {cold}) arg1>=40)")),
        ["one", "other"]
    );
    assert_eq!(
        found(&format!("line(({life} {cold}) or ({fire} high=7))")),
        ["one", "other", "ranged"]
    );
    // GGG's two spellings of one row are one name
    assert_eq!(
        found("line(\"# to maximum Life\" \"# to Maximum life\" arg1>=40)"),
        ["one", "other"]
    );
    // a row asked of the mod's text names nothing, and moves no number
    assert_eq!(
        found(&format!("line({life} template:\"cold damage\" arg1>=40)")),
        ["one", "other"]
    );
    assert_eq!(
        found(&format!("line({life} template:\"cold damage\" arg1=3)")),
        none
    );
    // a not of a name is of the occurrence, and names nothing
    assert_eq!(
        found(&format!("line({life} -{fire} arg1=40)")),
        ["one", "other"]
    );
    assert_eq!(found(&format!("line({life} -{cold} arg1=40)")), none);
    // named by its whole text, the mod's numbers in order, a row of it
    // named beside or not
    let whole = COLD_LIFE_TYPED;
    assert_eq!(found(&format!("line({whole} arg3=40)")), ["one"]);
    assert_eq!(found(&format!("line({whole} {life} arg3=40)")), ["one"]);
    assert_eq!(
        found(&format!("line({life} ({whole} or false()) arg3=40)")),
        ["one"]
    );
    assert_eq!(
        found(&format!("line({life} -({whole} arg3=40))")),
        ["other"]
    );
    // the row alone says what the slot reads, wherever the slot sits
    for text in [
        format!("line({life} arg3=40)"),
        format!("line({life} -(high=9))"),
        format!("line({life} (high=9 or true()))"),
        format!("line({life} -{cold} high=9)"),
        format!("line({life} template:\"cold damage\" high=9)"),
    ] {
        let e = ask(&load(&s, Some("pc")), &text).unwrap_err().to_json();
        assert_eq!(e["kind"], "slot_unknown", "`{text}`: {e}");
    }
}

/// What a slot is read beside is weighed from the query, and a reading
/// is made by the group that was asked and by no reading of it: a
/// hundred rows named in five alternations, twenty rows named together,
/// and a slot thirty groups deep are each weighed in no time. None of
/// the first two's readings is offered, two rows being named in each
/// still.
#[test]
fn what_a_slot_is_read_beside_costs_the_groups_size() {
    let s = both_stash();
    let c = load(&s, Some("pc"));
    let name = |group: u8, n: u8| {
        format!(
            "\"# to {}{}\"",
            char::from(b'A' + group),
            char::from(b'a' + n)
        )
    };
    let alternation = |group: u8| {
        let names: Vec<String> = (0..20).map(|n| name(group, n)).collect();
        format!("({})", names.join(" or "))
    };
    let wide: Vec<String> = (0..5).map(alternation).collect();
    let together: Vec<String> = (0..20).map(|n| name(0, n)).collect();
    let mut deep = format!("\"{LIFE}\" arg1>=40");
    for _ in 0..30 {
        deep = format!("(({deep}) or false()) -is:fractured");
    }
    let started = std::time::Instant::now();
    for text in [
        format!("line({} arg1>=1)", wide.join(" ")),
        format!("line({} arg1>=1)", together.join(" ")),
    ] {
        let e = ask(&c, &text).unwrap_err().to_json();
        assert_eq!(e["kind"], "slot_of_two_rows", "{e}");
        assert_eq!(e.get("readings"), None, "{e}");
    }
    assert_eq!(found(&s, &format!("line({deep})")), ["one", "other"]);
    assert!(
        started.elapsed() < std::time::Duration::from_secs(20),
        "{:?}",
        started.elapsed()
    );
    println!("weighed in {:?}", started.elapsed());
}

/// Four items of one line each, in pc Standard, `Parts` (p1): `two`,
/// cold damage 3 to 9 and 40 life in one mod; `first`, 40 life and 5 cold
/// resistance in one mod; `equal`, 40 life and 40 cold resistance in one
/// mod; `alone`, 40 life.
fn parts_stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("p1", "Parts")]), 10);
    let one = |id: &str, line: &str| {
        item(
            id,
            &format!("Item {id}"),
            "Iron Ring",
            "Rare",
            json!({ "explicitMods": [line] }),
        )
    };
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "p1",
        "Parts",
        vec![
            one("two", "Adds 3 to 9 Cold Damage\n+40 to maximum Life"),
            one("first", "+40 to maximum Life\n+5% to Cold Resistance"),
            one("equal", "+40 to maximum Life\n+40% to Cold Resistance"),
            one("alone", "+40 to maximum Life"),
        ],
        20,
    );
    s
}

/// C91, C93 (the outside review of step 9c4, its second look, finding
/// 1): a not is of what it holds, read as it is read there — the
/// templates inside it say what its comparisons read, as they do outside
/// one, and those conjoined with the not say it where it has none. On an
/// occurrence that is read, a group or its not holds, never both and
/// never neither.
#[test]
fn c93_a_not_is_of_what_it_holds_read_as_it_is_read_there() {
    let s = parts_stash();
    let found = |text: &str| found(&s, text);
    let all = ["alone", "equal", "first", "two"];
    let none = Vec::<String>::new();
    let life_is = |n: i32| format!("\"{LIFE}\" arg1={n}");
    assert_eq!(found(&format!("line({})", life_is(40))), all);
    assert_eq!(found(&format!("line(-({}))", life_is(40))), none);
    // two's first number is 3, and its life is not
    assert_eq!(found(&format!("line({})", life_is(3))), none);
    assert_eq!(found(&format!("line(-({}))", life_is(3))), all);
    for n in [3, 40] {
        let (is, not) = (life_is(n), format!("-({})", life_is(n)));
        assert_eq!(found(&format!("line(({is}) or {not})")), all, "{n}");
        assert_eq!(found(&format!("line({not} or ({is}))")), all, "{n}");
        assert_eq!(found(&format!("line(({is}) {not})")), none, "{n}");
        assert_eq!(found(&format!("line({is}) or line({not})")), all, "{n}");
        // a doubled not is none, and a template under one still says
        // what the comparison beside it reads
        assert_eq!(
            found(&format!("line(-{not})")),
            found(&format!("line({is})")),
            "{n}"
        );
        assert_eq!(
            found(&format!("line(--\"{LIFE}\" arg1={n})")),
            found(&format!("line({is})")),
            "{n}"
        );
    }
    // the template conjoined with the not says what it reads
    assert_eq!(found(&format!("line(\"{LIFE}\" -(arg1=40))")), none);
    assert_eq!(found(&format!("line(\"{LIFE}\" -(arg1=3))")), all);
    assert_eq!(
        found("line(\"#% to Cold Resistance\" -(arg1=5 or arg1=3))"),
        ["equal"]
    );
    // a not inside a not: first's cold resistance is 5, equal's 40, and
    // the two others have none
    assert_eq!(
        found("line(-(\"#% to Cold Resistance\" -(arg1=5)))"),
        ["alone", "first", "two"]
    );
    // every count of the not, routed
    let a = asked(&s, &format!("line(-({}))", life_is(40)));
    let term = &a["terms"][0];
    assert_eq!(counts(term)[0], 0);
    assert_eq!(counts(term)[3], 0);
    for kind in ["failed", "lacked"] {
        if term[kind]["count"] != 0 {
            follow(&s, &term[kind]);
        }
    }
}

/// C92, C95 (the same look, finding 2): a number of a mod is added
/// once, however many alternatives read it — the mod's first number and
/// its first row's first are one number — and two numbers of two rows are
/// two, equal or not.
#[test]
fn c95_a_number_two_alternatives_read_is_added_once() {
    let s = parts_stash();
    let found = |text: &str| found(&s, text);
    // by words alone, the mod's numbers in order: two's first is 3
    assert_eq!(
        found("sum(line(template:life).arg1)=40"),
        ["alone", "equal", "first"]
    );
    assert_eq!(found("sum(line(template:life).arg1)=3"), ["two"]);
    // the row named and the mod's first number are one number, but on
    // two, whose life is its third
    let either = format!("\"{LIFE}\" or template:life");
    assert_eq!(
        found(&format!("sum(line({either}).arg1)=40")),
        ["alone", "equal", "first"]
    );
    assert_eq!(found(&format!("sum(line({either}).arg1)=43")), ["two"]);
    assert_eq!(
        found(&format!("sum(line({either}).arg1)=80")),
        Vec::<String>::new()
    );
    // two rows are two numbers, equal or not
    let rows = format!("\"{LIFE}\" or \"#% to Cold Resistance\"");
    assert_eq!(found(&format!("sum(line({rows}).arg1)=80")), ["equal"]);
    assert_eq!(found(&format!("sum(line({rows}).arg1)=45")), ["first"]);
    // together: one number of 40 reaches no 60, and two's 40 and 3 reach 42
    let a = asked(&s, &format!("line(({either}) arg1>=60)"));
    assert_eq!(counts(&a["terms"][0]), [0, 4, 0, 0]);
    assert_eq!(a["terms"][0]["together"]["count"], 0);
    let a = asked(&s, &format!("line(({either}) arg1>=42)"));
    assert_eq!(a["terms"][0]["together"]["count"], 1);
    assert_eq!(follow(&s, &a["terms"][0]["together"]), ["two"]);
    // what a row sorts by is the number, once
    let sorted = view(
        &s,
        "id:first",
        json!({ "rows": { "sort": format!("sum(line({either}).arg1)") } }),
    );
    assert_eq!(sorted["rows"][0]["sort"]["value"], 40);

    // a number added is added, whatever an alternative that may hold
    // would read: 40 life beside a cold resistance the search does not
    // read, which leaves the second alternative open and the sum whole
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("o1", "Open")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "o1",
        "Open",
        vec![item(
            "half",
            "Item half",
            "Iron Ring",
            "Rare",
            json!({ "explicitMods": ["+40 to maximum Life\n+5.12345% to Cold Resistance"] }),
        )],
        20,
    );
    let may = format!("\"{LIFE}\" or (template:life arg2>=1)");
    assert_eq!(
        self::found(
            &s,
            &format!("line({may}) undecided(line(template:life arg2>=1))")
        ),
        ["half"]
    );
    assert_eq!(
        self::found(&s, &format!("sum(line({may}).arg1)=40")),
        ["half"]
    );
    // and one no alternative that holds reads is open
    assert_eq!(
        self::found(&s, "undecided(sum(line(template:life arg2>=1).arg1))"),
        ["half"]
    );
}

/// C93, rule 8 of the plan (the outside review of step 9c4, its third
/// look, finding 2): a row whose template another row of its mod
/// displays has no one place among the mod's numbers — its number is
/// either's — so nothing read elsewhere is known to be it, and what it
/// leaves open stays open beside a number that is added. twice's two
/// rows are life, 3 and 7.
#[test]
fn c93_a_number_that_is_either_rows_is_no_number_added_already() {
    let s = stash();
    let found = |text: &str| found(&s, &format!("id:twice {text}"));
    let none = Vec::<String>::new();
    // by words alone, the mod's numbers in order
    assert_eq!(found("sum(line(template:life).arg1)=3"), ["twice"]);
    // by the row named, either's
    assert_eq!(found(&format!("undecided(sum(\"{LIFE}\"))")), ["twice"]);
    // by both: 3 is added, and the row named may be the 7
    let either = format!("sum(line(\"{LIFE}\" or template:life).arg1)");
    assert_eq!(found(&format!("undecided({either})")), ["twice"]);
    for n in [3, 6, 7, 10] {
        assert_eq!(found(&format!("{either}={n}")), none, "{n}");
    }
    let sorted = view(&s, "id:twice", json!({ "rows": { "sort": either } }));
    assert_eq!(
        scalar(&sorted["rows"][0]["sort"]),
        json!({ "value": 3, "status": "incomplete" })
    );
    // and the comparison is open on it: its 3 is under 5, and the row
    // named may be the 7. Over the seven items ring, pair and plain
    // match, thread and taken have no such line, and long's life is
    // unread
    let a = asked(&s, &format!("line((\"{LIFE}\" or template:life) arg1>=5)"));
    let term = &a["terms"][0];
    assert_eq!(counts(term), [3, 0, 2, 2]);
    assert_eq!(follow(&s, &term["undecided"]), ["long", "twice"]);
    assert_eq!(term["together"]["count"], 0);
}

/// A group costs its size (the same look, finding 1): a not inside a
/// not inside a not is weighed once for each, where each had weighed
/// all it held again for every part a template may name — ten deep took
/// 2.7 s of a release build on one item. Thirty deep is no time, and an
/// even number of nots is none.
#[test]
fn a_not_inside_a_not_costs_its_size() {
    let s = parts_stash();
    let mut inner = format!("\"{LIFE}\" arg1=40");
    let mut answers = Vec::new();
    let started = std::time::Instant::now();
    for depth in 0..=30 {
        if depth % 10 == 0 || depth == 29 {
            answers.push((depth, found(&s, &format!("line({inner})"))));
        }
        inner = format!("-(true() ({inner}))");
    }
    let all = ["alone", "equal", "first", "two"]
        .map(String::from)
        .to_vec();
    assert_eq!(
        answers,
        [
            (0, all.clone()),
            (10, all.clone()),
            (20, all.clone()),
            (29, Vec::new()),
            (30, all),
        ]
    );
    assert!(
        started.elapsed() < std::time::Duration::from_secs(20),
        "{:?}",
        started.elapsed()
    );
}

/// Rule 8 of the plan, C93: what is unread of one row is unread of that
/// row. A number the search does not read in the first row leaves the
/// second row's comparison decided and the first's open, with its reason;
/// the text is a witness of the line all the same. Two rows displaying
/// one template leave that row's number open, and say so.
#[test]
fn c93_a_row_is_open_by_its_own_number_and_no_other() {
    let s = stash();
    let found = |text: &str| found(&s, text);
    assert_eq!(found("\"#% to Cold Resistance\"=5"), ["long"]);
    assert_eq!(
        found(&format!("line(\"{LIFE}\")")),
        ["long", "pair", "plain", "ring", "twice"]
    );
    let open = asked(&s, &format!("undecided(\"{LIFE}\">=1)"));
    assert_eq!(ids(&open), ["long", "twice"]);
    let why = |id: &str| -> Vec<(String, String)> {
        let row = open["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap();
        row["matched"][0]["shows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| {
                (
                    e["undecided"]["unread"].as_str().unwrap().to_string(),
                    e["undecided"]["problem"].as_str().unwrap().to_string(),
                )
            })
            .collect()
    };
    assert_eq!(
        why("long"),
        [(
            "the numbers of explicit lines".to_string(),
            "`explicitMods[0]`: a number with more digits than the search reads: ten whole, four decimals".to_string()
        )]
    );
    assert_eq!(
        why("twice"),
        [(
            "the numbers of explicit lines".to_string(),
            "`explicitMods[0]`: two rows of this mod display one template, `# to maximum Life`: a number named by that row is either's".to_string()
        )]
    );
    // named by its whole text, the mod's numbers in order: the second is
    // read, the first is not
    let whole = "\"# to maximum Life\\n#% to Cold Resistance\"";
    assert_eq!(found(&format!("line({whole} arg2=5)")), ["long"]);
    let a = asked(&s, &format!("line({whole} arg1>=1)"));
    assert_eq!(counts(&a["terms"][0]), [0, 0, 6, 1]);
    // twice's own numbers stand where the mod is named whole
    assert_eq!(
        found("line(\"# to maximum Life\\n# to maximum Life\" arg1=3 arg2=7)"),
        ["twice"]
    );
    // a total reads the row: long's cold resistance is 5, decided, and
    // its life is open
    assert_eq!(found("pseudo.total_cold_res=5"), ["long"]);
    assert_eq!(found("undecided(pseudo.total_life)"), ["long", "twice"]);
}

/// C97, C105: the vocabulary lists a mod by its whole text, and a row's
/// count is its term's matched count — a template's row counts the mods
/// it names by a row too, with their numbers, sources and flags. A row of
/// a mod that no mod displays alone — a wrapped sentence's — is no row of
/// the vocabulary.
#[test]
fn c97_the_vocabulary_counts_a_template_as_its_term_matches() {
    let s = stash();
    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["line"], "limit": 20 } }),
    );
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(
        shape(table),
        shaped(&[
            (LIFE, 5),
            (ALL_RES, 2),
            ("# to maximum Life\n# to Maximum life", 1),
            ("# to maximum Life\n#% to Cold Resistance", 1),
            ("Adds # to # Cold Damage\n# to maximum Life", 1),
            (
                "Enemies you Kill that are affected by Elemental Ailments\ngrant #% increased Flask Charges",
                1
            ),
            ("Only affects Passives in Small Ring", 1),
            (THREAD, 1),
        ])
    );
    // every row's route returns what it counted, its kinds' too
    for b in table["buckets"].as_array().unwrap() {
        follow(&s, b);
        for kind in b["sources"].as_array().into_iter().flatten() {
            follow(&s, kind);
        }
        for kind in b["flags"].as_array().into_iter().flatten() {
            follow(&s, kind);
        }
    }
    let all = bucket(table, ALL_RES);
    assert_eq!(all["term"], format!("line(\"{ALL_RES}\")"));
    assert_eq!(follow(&s, all), ["ring", "thread"]);
    assert_eq!(
        all["slots"],
        json!([{ "slot": "arg1", "min": -17, "max": 12 }])
    );
    let life = bucket(table, LIFE);
    assert_eq!(follow(&s, life), ["long", "pair", "plain", "ring", "twice"]);
    // pair's 25 and plain's 95, a row's 40 between; long's and twice's
    // were not read
    assert_eq!(
        life["slots"],
        json!([{ "slot": "arg1", "min": 25, "max": 95, "incomplete": true }])
    );
    assert_eq!(life["flags"][0]["kind"], "crafted");
    assert_eq!(follow(&s, &life["flags"][0]), ["pair"]);
    assert_eq!(life["sources"][0]["count"], 5);
    // a mod's own row reads the mod's numbers in order
    assert_eq!(
        bucket(table, "Adds # to # Cold Damage\n# to maximum Life")["slots"],
        json!([
            { "slot": "low", "min": 3, "max": 3 },
            { "slot": "high", "min": 9, "max": 9 },
            { "slot": "arg3", "min": 40, "max": 40 },
        ])
    );

    // narrowed by words: the whole texts that hold them
    let a = view(&s, "", json!({ "counts": { "keys": ["line:resist"] } }));
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(
        shape(table),
        shaped(&[
            (ALL_RES, 2),
            ("# to maximum Life\n#% to Cold Resistance", 1),
            (THREAD, 1),
            ("(none)", 4),
        ])
    );
    for b in table["buckets"].as_array().unwrap() {
        if b["bucket"] != "computed" {
            follow(&s, b);
        }
    }
    // narrowed by a pattern whose ends are the whole text's: the template
    // is listed by ring's line, and counted as its term matches — thread
    // by its row, which the narrowing's own term does not select
    let a = view(&s, "", json!({ "counts": { "keys": ["line~^#% to all"] } }));
    let table = &a["view"]["counts"]["tables"][0];
    assert_eq!(shape(table), shaped(&[(ALL_RES, 2), ("(none)", 6)]));
    assert_eq!(follow(&s, bucket(table, ALL_RES)), ["ring", "thread"]);
}

/// C100, invariant 2: a quoted template resolves to itself and lists
/// nothing where it named a row — and lists the spellings where it found
/// two, a row's among them; a `:` selector resolves to whole texts. A
/// suggestion's count is its term's (invariant 4), the mods it names by
/// a row counted.
#[test]
fn c100_a_quoted_template_resolves_to_the_names_it_found() {
    let s = stash();
    let a = asked(&s, &format!("line(\"{ALL_RES}\")"));
    assert!(a["terms"][0].get("resolved").is_none(), "{}", a["terms"][0]);
    // GGG's two spellings, one of them a row of twice alone
    let a = asked(&s, &format!("line(\"{LIFE}\")"));
    assert_eq!(
        a["terms"][0]["resolved"],
        json!({ "of": "template", "values": [
            { "value": "# to maximum Life", "items": 5 },
            { "value": "# to Maximum life", "items": 1 },
        ], "more": 0 })
    );
    let a = asked(&s, "line(template:resistances)");
    assert_eq!(
        a["terms"][0]["resolved"],
        json!({ "of": "template", "values": [
            { "value": ALL_RES, "items": 1 },
            { "value": THREAD, "items": 1 },
        ], "more": 0 })
    );
    let a = asked(&s, "line(\"#% to all Elemental Resistance\")");
    assert_eq!(a["total"]["matched"], 0);
    let suggestions = a["zero"]["resolved_to_nothing"][0]["suggestions"]
        .as_array()
        .unwrap();
    assert!(!suggestions.is_empty());
    for suggested in suggestions {
        let term = suggested["term"].as_str().unwrap();
        assert_eq!(
            asked(&s, term)["total"]["matched"],
            suggested["items"],
            "{suggested}"
        );
    }
    let of = |value: &str| {
        suggestions
            .iter()
            .find(|s| s["value"] == value)
            .unwrap_or_else(|| panic!("no suggestion `{value}` in {suggestions:?}"))
    };
    assert_eq!(of(ALL_RES)["items"], 2);
    assert_eq!(of(THREAD)["items"], 1);
}

/// `show` prints a mod of several rows with each row as the search reads
/// it: its template and its numbers.
#[test]
fn show_gives_a_mod_its_rows() {
    let s = stash();
    let shown = serde_json::to_value(show(&s, "pair", false).unwrap()).unwrap();
    assert_eq!(
        shown["lines"][0]["rows"],
        json!([
            { "template": "Adds # to # Cold Damage", "numbers": [3, 9] },
            { "template": "# to maximum Life", "numbers": [40] },
        ])
    );
    assert_eq!(
        shown["lines"][0]["template"],
        "Adds # to # Cold Damage\n# to maximum Life"
    );
    assert_eq!(shown["lines"][1].get("rows"), None);
}
