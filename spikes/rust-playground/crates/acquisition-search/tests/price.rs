//! The effective price through the crate's boundary (`search/BUILD-PLAN.md`,
//! step 9; C81, C100, C98): a request in, an answer out, as JSON. Every
//! count is worked by hand from the fixture below, every route an answer
//! prints is followed and compared by id, and the price an item shows is
//! the listing state's — never a reading of this crate's own.

mod common;

use std::collections::BTreeSet;

use acquisition_plan::price::{Buyout, Price, PriceTarget, set_buyout};
use acquisition_search::{Request, answer, describe};
use acquisition_store::{Annotations, IntentValue, Provenance, Store};
use common::*;
use serde_json::{Value, json};

fn price(amount: &str, currency: &str) -> Price {
    Price {
        amount: amount.parse().unwrap(),
        currency: currency.into(),
    }
}

/// A `buyout` row stamped newer than this build reads: what the listing
/// state calls a row that cannot be read (C81).
#[derive(serde::Serialize)]
#[serde(transparent)]
struct Newer(Value);

impl IntentValue for Newer {
    const KIND: &'static str = "buyout";
    const VERSION: i64 = 2;
    fn parse(value: &Value) -> Result<Self, String> {
        Ok(Newer(value.clone()))
    }
}

fn via() -> Provenance {
    Provenance::via("test")
}

fn set(intent: &mut Annotations, target: &str, value: Buyout) {
    let target: PriceTarget = target.parse().unwrap();
    set_buyout(intent, &target, &value, None, &via()).unwrap();
}

/// Thirteen pc items in Standard.
///
/// `Sale` (s1), public: `noted` (`~price 5 chaos`), `bo` (`~b/o 2/3
/// divine`), `bad` (`~price 5 nosuch`: no such currency, so the tab's name
/// applies, and `Sale` is no price), `rowed` (no note; the owner's row,
/// exact 12.5 chaos), `tied` (`~price 7 chaos` and the owner's row of 12
/// chaos: the game wins the tie, C81), `skipped` (the owner's `skip`),
/// `nop` (the owner's `no_price`), `stuck` (a row stamped v2, which this
/// build cannot read), `oddnote` (a note that is a number: unread at the
/// note), `plain` (nothing).
/// `~price 3 chaos` (d1), not public: `hidden` (`~price 9 chaos`: residue,
/// the index cannot see it) and `dumped`; the owner's row on the tab,
/// negotiable 1 exalted, covers both.
/// `Mover` wears `worn` (`~price 4 chaos`: a character has no public tab);
/// the owner's row on the character, negotiable 2 divine, covers it.
///
/// Priced: noted, bo, rowed, tied, hidden, dumped, worn (7). Not priced:
/// bad, skipped, nop, plain (4). Not established: stuck, oddnote (2).
fn stash() -> (Store, Annotations) {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([
            { "id": "s1", "name": "Sale", "type": "PremiumStash", "metadata": { "public": true } },
            { "id": "d1", "name": "~price 3 chaos", "type": "PremiumStash" },
        ]),
        10,
    );
    let rare = |id: &str, name: &str, more: Value| item(id, name, "Two-Stone Ring", "Rare", more);
    let noted = |id: &str, name: &str, note: &str| rare(id, name, json!({ "note": note }));
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "s1",
        "Sale",
        vec![
            noted("noted", "Noted Loop", "~price 5 chaos"),
            noted("bo", "Bulk Loop", "~b/o 2/3 divine"),
            noted("bad", "Odd Loop", "~price 5 nosuch"),
            rare("rowed", "Doom Plate", json!({})),
            noted("tied", "Tied Loop", "~price 7 chaos"),
            rare("skipped", "Kept Loop", json!({})),
            rare("nop", "Listed Loop", json!({})),
            rare("stuck", "Stuck Loop", json!({})),
            rare("oddnote", "Numbered Loop", json!({ "note": 5 })),
            rare("plain", "Plain Loop", json!({})),
        ],
        20,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "d1",
        "~price 3 chaos",
        vec![
            noted("hidden", "Hidden Loop", "~price 9 chaos"),
            rare("dumped", "Dumped Loop", json!({})),
        ],
        21,
    );
    list_characters(
        &mut s,
        "pc",
        json!([{ "id": "c1", "name": "Mover", "league": "Standard" }]),
        30,
    );
    fetch_character(
        &mut s,
        "pc",
        json!({ "id": "c1", "name": "Mover", "league": "Standard",
                "equipment": [ noted("worn", "Worn Loop", "~price 4 chaos") ], "inventory": [] }),
        31,
    );
    let mut intent = common::intent(&s);
    set(
        &mut intent,
        "item/rowed",
        Buyout::Exact(price("12.5", "chaos")),
    );
    set(
        &mut intent,
        "item/tied",
        Buyout::Exact(price("12", "chaos")),
    );
    set(&mut intent, "item/skipped", Buyout::Skip);
    set(&mut intent, "item/nop", Buyout::NoPrice);
    set(
        &mut intent,
        "tab/pc/d1",
        Buyout::Negotiable(price("1", "exalted")),
    );
    set(
        &mut intent,
        "character/c1",
        Buyout::Negotiable(price("2", "divine")),
    );
    intent
        .put::<Newer>(
            "item",
            "stuck",
            &json!({ "version": 2, "type": "exact" }),
            None,
            &via(),
        )
        .unwrap();
    (s, intent)
}

fn asked(s: &Store, intent: &Annotations, request: Value) -> Value {
    let request: Request = serde_json::from_value(request).unwrap();
    as_json(&answer(&load_with(s, intent, Some("pc")), &request).unwrap())
}

fn query(text: &str) -> Value {
    json!({ "query": { "text": text }, "view": { "rows": { "limit": 100 } } })
}

fn set_of(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

/// The ids a count's route returns, every row of it; none for a zero.
fn members(s: &Store, intent: &Annotations, counted: &Value) -> BTreeSet<String> {
    let n = counted["count"].as_u64().unwrap();
    let Some(route) = counted.get("request") else {
        assert_eq!(n, 0, "a count with no route: {counted}");
        return BTreeSet::new();
    };
    let mut route = route.clone();
    route["view"]["rows"]["limit"] = json!(100);
    let request: Request = serde_json::from_value(route.clone()).unwrap();
    let routed = as_json(&answer(&load_with(s, intent, Some("pc")), &request).unwrap());
    let ids: BTreeSet<String> = ids(&routed).into_iter().collect();
    assert_eq!(ids.len() as u64, n, "{route}");
    ids
}

/// The four counts of a term, each route followed and its members returned.
fn counts_of(s: &Store, intent: &Annotations, term: &Value) -> [(usize, BTreeSet<String>); 4] {
    ["matched", "failed", "lacked", "undecided"].map(|word| {
        (
            term[word]["count"].as_u64().unwrap() as usize,
            members(s, intent, &term[word]),
        )
    })
}

fn shows(row: &Value) -> Vec<(String, Value)> {
    row["matched"][0]["shows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["value"]["name"].as_str().unwrap_or_default().to_string(),
                e["value"]["value"].clone(),
            )
        })
        .collect()
}

/// C81 through the boundary: `has:priced` is a witness where the effective
/// statement carries a price — the game's or the owner's, the game's on a
/// tie — known absence where the listing resolved to none, no price or
/// skip, and undecided where a row or a note that could decide cannot be
/// read. A row shows the price as the listing state prints it, the side
/// and where it came from (C100).
#[test]
fn c81_has_priced_is_the_effective_price_three_valued_with_the_price_on_the_row() {
    let (s, intent) = stash();
    let a = asked(&s, &intent, query("has:priced"));
    let term = &a["terms"][0];
    let [matched, failed, lacked, undecided] = counts_of(&s, &intent, term);
    assert_eq!(
        (matched.0, failed.0, lacked.0, undecided.0),
        (7, 0, 4, 2),
        "{term}"
    );
    assert_eq!(
        matched.1,
        set_of(&["bo", "dumped", "hidden", "noted", "rowed", "tied", "worn"])
    );
    assert_eq!(lacked.1, set_of(&["bad", "nop", "plain", "skipped"]));
    assert_eq!(undecided.1, set_of(&["oddnote", "stuck"]));
    assert_eq!(a["total"]["matched"], 7);
    assert_eq!(a["total"]["undecided"]["count"], 2);

    let row = |id: &str| {
        a["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .unwrap()
            .clone()
    };
    // the owner's row: 12.5 chaos, by hand; the item's own row is the
    // row's, so nothing says where
    assert_eq!(
        shows(&row("rowed")),
        [
            ("price".to_string(), json!("12.5 chaos")),
            ("price.side".to_string(), json!("manual")),
        ]
    );
    // the game's note beats the owner's row of 12 on a tie (C81)
    assert_eq!(
        shows(&row("tied")),
        [
            ("price".to_string(), json!("7 chaos")),
            ("price.side".to_string(), json!("game")),
        ]
    );
    // a bulk ratio, negotiable, as the listing state prints it
    assert_eq!(shows(&row("bo"))[0].1, json!("2/3 divine b/o"));
    // a character item under the character's row: inherited, from the character
    assert_eq!(
        shows(&row("worn")),
        [
            ("price".to_string(), json!("2 divine b/o")),
            ("price.side".to_string(), json!("manual")),
            ("price.from".to_string(), json!("character/c1")),
        ]
    );
    // a non-public tab's name and note are residue: the owner's tab row applies
    assert_eq!(shows(&row("hidden"))[0].1, json!("1 exalted b/o"));
    assert_eq!(shows(&row("hidden"))[2].1, json!("tab/pc/d1"));

    // known absence: `-has:priced` is a route, and it holds the four
    let none = asked(&s, &intent, query("-has:priced"));
    assert_eq!(
        ids(&none),
        ["bad", "nop", "plain", "skipped"].map(str::to_string)
    );

    // the two that could not be established, each with its reason, said
    // where the listing state read it (rule 8)
    let open = asked(&s, &intent, query("undecided(priced)"));
    assert_eq!(ids(&open), ["oddnote", "stuck"].map(str::to_string));
    for row in open["rows"].as_array().unwrap() {
        let reason = &row["matched"][0]["shows"][0]["undecided"];
        // a row's reason names the intent file; a note's names the body,
        // which a refresh may bring readable
        let (unread, hint) = if row["id"] == "stuck" {
            (
                "the price: a row that could decide cannot be read",
                "a refresh will not help",
            )
        } else {
            ("the price: the note cannot be read", "a refresh may help")
        };
        assert_eq!(reason["unread"], unread, "{row}");
        assert!(
            reason["problem"]
                .as_str()
                .unwrap()
                .ends_with("would decide (C81)"),
            "{reason}"
        );
        assert!(
            reason["hint"].as_str().unwrap().starts_with(hint),
            "{reason}"
        );
    }
    let stuck = open["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "stuck")
        .unwrap();
    assert_eq!(
        stuck["matched"][0]["shows"][0]["undecided"]["problem"],
        "the row on the item cannot be read (buyout declares version 2, newer than this build's v1 — a newer build wrote it) and would decide (C81)"
    );
    let odd = open["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "oddnote")
        .unwrap();
    assert_eq!(
        odd["matched"][0]["shows"][0]["undecided"]["problem"],
        "the note cannot be read: it is no string, and a note would decide (C81)"
    );
}

/// The parts of a price: `price.amount` is the decimal or a ratio's
/// `wanted`, `price.lot` a ratio's lot which a decimal price lacks,
/// `price.currency` the table's tag as a closed set. Each is lacked with
/// the price and open with it.
#[test]
fn c100_the_prices_parts_are_numbers_and_a_closed_set_lacked_and_open_with_it() {
    let (s, intent) = stash();
    let a = asked(&s, &intent, query("price.amount>=5"));
    let [matched, failed, lacked, undecided] = counts_of(&s, &intent, &a["terms"][0]);
    assert_eq!(matched.1, set_of(&["noted", "rowed", "tied"]));
    assert_eq!(failed.1, set_of(&["bo", "dumped", "hidden", "worn"]));
    assert_eq!(lacked.1, set_of(&["bad", "nop", "plain", "skipped"]));
    assert_eq!(undecided.1, set_of(&["oddnote", "stuck"]));
    // the value shows beside the price
    let rowed = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "rowed")
        .unwrap();
    assert_eq!(shows(rowed)[0], ("price".to_string(), json!("12.5 chaos")));

    let chaos = asked(&s, &intent, query("price.currency=chaos"));
    let [matched, failed, lacked, undecided] = counts_of(&s, &intent, &chaos["terms"][0]);
    assert_eq!((matched.0, failed.0, lacked.0, undecided.0), (3, 4, 4, 2));
    assert_eq!(matched.1, set_of(&["noted", "rowed", "tied"]));
    // `:` picks among the legal tags: `div` is divine alone
    let div = asked(&s, &intent, query("price.currency:div"));
    assert_eq!(ids(&div), ["bo", "worn"].map(str::to_string));
    assert_eq!(div["terms"][0]["resolved"]["values"][0]["value"], "divine");

    // a bulk ratio's lot; a decimal price has none
    let lot = asked(&s, &intent, query("price.lot=3"));
    let [matched, failed, lacked, undecided] = counts_of(&s, &intent, &lot["terms"][0]);
    assert_eq!((matched.0, failed.0, lacked.0, undecided.0), (1, 0, 10, 2));
    assert_eq!(matched.1, set_of(&["bo"]));
    assert!(lacked.1.contains("rowed") && lacked.1.contains("plain"));
    let no_lot = asked(&s, &intent, query("-has:price.lot"));
    assert_eq!(no_lot["total"]["matched"], 10);
    // a decimal's amount is the decimal, a ratio's the wanted
    let ratio = asked(&s, &intent, query("price.amount=2 has:price.lot"));
    assert_eq!(ids(&ratio), ["bo"].map(str::to_string));
}

/// A price sorts and sums as a number, and a count by currency or amount
/// keeps the amount's decimals (C95, C105): every bucket's term selects
/// exactly its members.
#[test]
fn c95_a_price_sorts_sums_and_counts_with_its_decimals_kept() {
    let (s, intent) = stash();
    let sorted = asked(
        &s,
        &intent,
        json!({ "query": { "text": "has:priced" },
                "view": { "rows": { "limit": 100, "sort": "price.amount", "desc": true } } }),
    );
    let order: Vec<(&str, Value)> = sorted["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), r["sort"]["value"].clone()))
        .collect();
    assert_eq!(
        order,
        [
            ("rowed", json!(12.5)),
            ("tied", json!(7)),
            ("noted", json!(5)),
            // ties keep the store's order: characters before tabs, tabs by
            // id, items by id
            ("worn", json!(2)),
            ("bo", json!(2)),
            ("dumped", json!(1)),
            ("hidden", json!(1)),
        ]
    );

    let counted = asked(
        &s,
        &intent,
        json!({ "query": { "text": "has:priced" },
                "view": { "counts": { "keys": ["price.currency"], "sum": "price.amount" } } }),
    );
    let table = &counted["view"]["counts"]["tables"][0];
    let buckets: Vec<(Value, u64, Value)> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["value"].clone(),
                b["count"].as_u64().unwrap(),
                b["sum"]["value"].clone(),
            )
        })
        .collect();
    // 5 + 12.5 + 7 chaos; 2 + 2 divine; 1 + 1 exalted — ranked by count,
    // then by the value
    assert_eq!(
        buckets,
        [
            (json!("chaos"), 3, json!(24.5)),
            (json!("divine"), 2, json!(4)),
            (json!("exalted"), 2, json!(2)),
        ]
    );
    for bucket in table["buckets"].as_array().unwrap() {
        let got = members(&s, &intent, bucket);
        let want = match bucket["value"].as_str().unwrap() {
            "chaos" => set_of(&["noted", "rowed", "tied"]),
            "divine" => set_of(&["bo", "worn"]),
            _ => set_of(&["dumped", "hidden"]),
        };
        assert_eq!(got, want, "{}", bucket["term"]);
    }

    // a count by the amount: 12.5 is a bucket of its own, its term
    // `price.amount=12.5`, and it returns the one item
    let by_amount = asked(
        &s,
        &intent,
        json!({ "query": { "text": "has:priced" },
                "view": { "counts": { "keys": ["price.amount"] } } }),
    );
    let table = &by_amount["view"]["counts"]["tables"][0];
    let labels: Vec<(Value, u64, Value)> = table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["value"].clone(),
                b["count"].as_u64().unwrap(),
                b["term"].clone(),
            )
        })
        .collect();
    assert_eq!(
        labels,
        [
            (json!(1), 2, json!("price.amount=1")),
            (json!(2), 2, json!("price.amount=2")),
            (json!(5), 1, json!("price.amount=5")),
            (json!(7), 1, json!("price.amount=7")),
            (json!(12.5), 1, json!("price.amount=12.5")),
        ]
    );
    for bucket in table["buckets"].as_array().unwrap() {
        members(&s, &intent, bucket);
    }
}

/// C98: the basis names the intent revision the prices were read at and
/// the versions the listing state was resolved under; a price written
/// after the load is what the check sees, and the next load carries it.
#[test]
fn c98_the_basis_names_the_intent_revision_and_a_price_write_moves_it() {
    let (s, mut intent) = stash();
    let held = load_with(&s, &intent, Some("pc"));
    assert_eq!(held.basis.intent, intent.revision().unwrap());
    assert_eq!(held.basis.intent, 7);
    let a = as_json(&ask(&held, "has:priced").unwrap());
    assert_eq!(
        (
            &a["basis"]["intent"],
            &a["basis"]["currency"],
            &a["basis"]["notes"]
        ),
        (&json!(7), &json!(1), &json!(2))
    );
    assert!(held.is_current(&s, &intent).unwrap());

    set(&mut intent, "item/plain", Buyout::Exact(price("3", "alch")));
    assert!(!held.is_current(&s, &intent).unwrap());
    // the answer already given stays what its basis says it was
    assert_eq!(
        as_json(&ask(&held, "has:priced").unwrap())["total"]["matched"],
        7
    );
    let next = load_with(&s, &intent, Some("pc"));
    assert_eq!(next.basis.intent, 8);
    let a = as_json(&ask(&next, "has:priced").unwrap());
    assert_eq!(a["total"]["matched"], 8);
    assert!(ids(&a).contains(&"plain".to_string()));

    // `show` carries the same price and the same basis
    let shown =
        serde_json::to_value(acquisition_search::show(&s, &intent, "plain", false).unwrap())
            .unwrap();
    assert_eq!(shown["price"]["status"], "is");
    assert_eq!(shown["price"]["text"], "3 alch");
    assert_eq!(shown["price"]["side"], "manual");
    assert_eq!(shown["basis"]["intent"], 8);
    let stuck =
        serde_json::to_value(acquisition_search::show(&s, &intent, "stuck", false).unwrap())
            .unwrap();
    assert_eq!(stuck["price"]["status"], "open");
    assert_eq!(stuck["price"]["part"], "price");
    let plain_none =
        serde_json::to_value(acquisition_search::show(&s, &intent, "skipped", false).unwrap())
            .unwrap();
    assert_eq!(
        (&plain_none["price"]["status"], &plain_none["price"]["kind"]),
        (&json!("none"), &json!("skip"))
    );
}

/// What a price cannot be asked is an authoring error that offers what
/// can, said before anything is read; `--describe` prints the four names
/// and the currency table's tags.
#[test]
fn the_price_names_refuse_what_they_cannot_take_and_describe_prints_them() {
    let (s, intent) = stash();
    let corpus = load_with(&s, &intent, Some("pc"));
    for (text, kind, reading) in [
        (
            "priced=yes",
            "operator_mismatch",
            Some(json!(["has:priced", "-has:priced", "undecided(priced)"])),
        ),
        ("priced:yes", "operator_mismatch", None),
        ("price.currency=nosuch", "unknown_value", None),
        ("price.currency>=5", "operator_mismatch", None),
        (
            "price.amount:5",
            "operator_mismatch",
            Some(json!(["price.amount=5"])),
        ),
        ("price.nosuch=1", "unknown_name", None),
    ] {
        let e = ask(&corpus, text).unwrap_err().to_json();
        assert_eq!(e["kind"], kind, "`{text}`: {e}");
        if let Some(reading) = reading {
            assert_eq!(e["readings"], reading, "`{text}`");
        }
    }
    let near = ask(&corpus, "price.currency=chaso").unwrap_err().to_json();
    assert_eq!(near["readings"][0], "price.currency=chaos");
    for (view, kind) in [
        (json!({ "counts": { "keys": ["priced"] } }), "view"),
        (
            json!({ "rows": { "sort": "price.currency" } }),
            "operator_mismatch",
        ),
        (
            json!({ "counts": { "keys": ["tab"], "sum": "priced" } }),
            "operator_mismatch",
        ),
    ] {
        let request: Request =
            serde_json::from_value(json!({ "query": { "text": "" }, "view": view })).unwrap();
        let e = answer(&corpus, &request).unwrap_err().to_json();
        assert_eq!(e["kind"], kind, "{view}: {e}");
    }

    let d = serde_json::to_value(describe(&[]).unwrap()).unwrap();
    let field = |name: &str| {
        d["fields"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| f["name"] == name)
            .unwrap_or_else(|| panic!("no field {name}"))
            .clone()
    };
    assert_eq!(field("priced")["kind"], "flag");
    assert_eq!(field("price.amount")["kind"], "number");
    assert_eq!(field("price.lot")["kind"], "number");
    let currency = field("price.currency");
    assert_eq!(currency["kind"], "closed set");
    let tags: Vec<&str> = currency["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(tags.contains(&"chaos") && tags.contains(&"divine") && tags.len() > 30);
    // `has:` names `priced`; a count's keys do not
    let has = d["fields"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "has")
        .unwrap();
    assert!(has["values"].as_array().unwrap().contains(&json!("priced")));
    let keys = &d["counts"][0]["values"];
    assert!(keys.as_array().unwrap().contains(&json!("price.currency")));
    assert!(!keys.as_array().unwrap().contains(&json!("priced")));
    // nothing of the price is left on the not-built list
    for n in d["not_built"].as_array().unwrap() {
        assert!(!n["construct"].as_str().unwrap().contains("price"), "{n}");
    }
}

/// Invariant 3 of the surface: a price term is valid over a store holding
/// nothing and no intent file has to hold a row; and over a store with
/// items and no rows, every item is priced or not by the game alone.
#[test]
fn a_price_term_over_an_empty_intent_file_is_the_games_alone() {
    let (s, _) = stash();
    let empty = common::intent(&s);
    let corpus = load_with(&s, &empty, Some("pc"));
    assert_eq!(corpus.basis.intent, 0);
    let a = as_json(&ask(&corpus, "has:priced").unwrap());
    // the public tab's notes alone: noted, bo, tied; `oddnote` still unread
    assert_eq!(ids(&a), ["bo", "noted", "tied"].map(str::to_string));
    assert_eq!(a["total"]["undecided"]["count"], 1);
    let sides: BTreeSet<String> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| shows(r)[1].1.as_str().unwrap().to_string())
        .collect();
    assert_eq!(sides, set_of(&["game"]));
}

/// The step-9 outside review, each finding reproduced (the record,
/// "Findings"): a body's malformed field is unread at that field and never
/// fails the search; a note the index cannot see decides nothing whatever
/// it says; an object note is unread too; a league-less character in a
/// realm with no league on record is priced by its row; a price past the
/// search's rule for numbers is unread at its number, never a float that
/// drops digits into another price's bucket.
fn one_priced(public: bool, more: Value, amount: &str) -> (Store, Annotations) {
    let mut s = store();
    list_tabs(
        &mut s,
        "pc",
        "Standard",
        json!([{ "id": "s1", "name": "Sale", "type": "PremiumStash", "metadata": { "public": public } }]),
        10,
    );
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "s1",
        "Sale",
        vec![item("i1", "Ring", "Iron Ring", "Rare", more)],
        20,
    );
    let mut intent = common::intent(&s);
    set(
        &mut intent,
        "item/i1",
        Buyout::Exact(price(amount, "chaos")),
    );
    (s, intent)
}

#[test]
fn review_a_malformed_field_of_one_body_is_unread_there_and_never_fails_the_search() {
    // the inventoryId is no string: the snapshot carries it as unread and
    // the search, which never reads it, is not the poorer
    let (s, intent) = one_priced(true, json!({ "inventoryId": 5 }), "12");
    let a = asked(&s, &intent, query("has:priced"));
    assert_eq!(ids(&a), ["i1"]);
    // an object note, which SQLite's extract flattens to text, is unread
    // as a number is: undecided, with the deriver's hint, since a refresh
    // may bring a body that reads
    for note in [json!({ "x": 1 }), json!(["~price 5 chaos"]), json!(5)] {
        let (s, intent) = one_priced(true, json!({ "note": note }), "12");
        let a = asked(&s, &intent, query("has:priced"));
        assert_eq!(a["total"]["undecided"]["count"], 1, "note={note}: {a}");
        let why = &a["total"]["undecided_items"][0]["why"][0];
        assert_eq!(why["unread"], "the price: the note cannot be read");
        assert!(
            why["hint"]
                .as_str()
                .unwrap()
                .starts_with("a refresh may help")
        );
    }
}

#[test]
fn review_an_unread_note_the_index_cannot_see_decides_nothing() {
    // a private tab: the note is residue whatever it says (C81), and the
    // owner's row prices the item
    let (s, intent) = one_priced(false, json!({ "note": 5 }), "12");
    let a = asked(&s, &intent, query("has:priced"));
    assert_eq!(ids(&a), ["i1"]);
    assert_eq!(shows(&a["rows"][0])[0].1, json!("12 chaos"));
    // a character's item likewise
    let mut s = store();
    list_characters(
        &mut s,
        "pc",
        json!([{ "id": "c1", "name": "Mover", "league": "Standard" }]),
        10,
    );
    fetch_character(
        &mut s,
        "pc",
        json!({ "id": "c1", "name": "Mover", "league": "Standard",
                "equipment": [ item("worn", "Ring", "Iron Ring", "Rare", json!({ "note": 5 })) ], "inventory": [] }),
        20,
    );
    let mut intent = common::intent(&s);
    set(
        &mut intent,
        "character/c1",
        Buyout::Exact(price("3", "divine")),
    );
    let a = asked(&s, &intent, query("has:priced"));
    assert_eq!(ids(&a), ["worn"]);
    assert_eq!(shows(&a["rows"][0])[0].1, json!("3 divine"));
}

#[test]
fn review_a_league_less_character_in_a_realm_with_no_league_is_priced_by_its_row() {
    let mut s = store();
    list_characters(&mut s, "pc", json!([{ "id": "c1", "name": "Mover" }]), 10);
    fetch_character(
        &mut s,
        "pc",
        json!({ "id": "c1", "name": "Mover",
                "equipment": [ item("i1", "Ring", "Iron Ring", "Rare", json!({})) ], "inventory": [] }),
        20,
    );
    let mut intent = common::intent(&s);
    set(&mut intent, "item/i1", Buyout::Exact(price("12", "chaos")));
    let a = asked(&s, &intent, query("has:priced"));
    assert_eq!(ids(&a), ["i1"]);
    assert_eq!(a["total"]["undecided"]["count"], 0);
    let shown =
        serde_json::to_value(acquisition_search::show(&s, &intent, "i1", false).unwrap()).unwrap();
    assert_eq!(shown["price"]["text"], "12 chaos");
}

#[test]
fn review_a_price_past_the_search_s_number_rule_is_unread_at_its_number() {
    // two prices a float cannot tell apart, and one it can: the first two
    // are priced and their amount is unread — undecided, never one bucket
    let (mut s, mut intent) = one_priced(true, json!({}), "9007199254740993/1");
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "s1",
        "Sale",
        vec![
            item("i1", "Ring", "Iron Ring", "Rare", json!({})),
            item("i2", "Ring", "Iron Ring", "Rare", json!({})),
            item("i3", "Ring", "Iron Ring", "Rare", json!({})),
        ],
        30,
    );
    set(
        &mut intent,
        "item/i2",
        Buyout::Exact(price("9007199254740992/1", "chaos")),
    );
    set(
        &mut intent,
        "item/i3",
        Buyout::Exact(price("1/18446744073709551615", "chaos")),
    );
    let priced = asked(&s, &intent, query("has:priced"));
    assert_eq!(ids(&priced), ["i1", "i2", "i3"]);
    let amount = asked(&s, &intent, query("price.amount>=1"));
    let [matched, failed, lacked, undecided] = counts_of(&s, &intent, &amount["terms"][0]);
    assert_eq!(
        (matched.0, failed.0, lacked.0, undecided.0),
        (1, 0, 0, 2),
        "{amount}"
    );
    assert_eq!(matched.1, set_of(&["i3"]));
    assert_eq!(undecided.1, set_of(&["i1", "i2"]));
    let why = &asked(&s, &intent, query("undecided(price.amount)"))["rows"][0]["matched"][0]["shows"]
        [0]["undecided"];
    assert_eq!(why["unread"], "the price's number");
    assert!(
        why["problem"]
            .as_str()
            .unwrap()
            .ends_with("is written longer than the search reads a number"),
        "{why}"
    );
    // the lot alone past the rule: the amount reads, the lot does not
    let lot = asked(&s, &intent, query("price.lot>=1"));
    let [matched, _, lacked, undecided] = counts_of(&s, &intent, &lot["terms"][0]);
    assert_eq!(
        (matched.1, lacked.0, undecided.1),
        (set_of(&["i1", "i2"]), 0, set_of(&["i3"]))
    );
    // a count by the amount: no bucket holds two prices that differ
    let counted = asked(
        &s,
        &intent,
        json!({ "query": { "text": "has:priced" }, "view": { "counts": { "keys": ["price.amount"] } } }),
    );
    let buckets: Vec<(Value, u64)> = counted["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| (b["bucket"].clone(), b["count"].as_u64().unwrap()))
        .collect();
    assert_eq!(buckets, [(json!("value"), 1), (json!("undecided"), 2)]);
    for bucket in counted["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
    {
        members(&s, &intent, bucket);
    }
}
