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

use acquisition_search::{Request, answer};
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
    // two rows named name no one row: the mod's numbers in order
    let both = format!("\"Adds # to # Cold Damage\" \"{LIFE}\"");
    assert_eq!(found(&format!("line({both} arg3=40)")), ["pair"]);
    assert_eq!(
        found(&format!("line({both} arg1=40)")),
        Vec::<String>::new()
    );
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
