//! The sockets (`search/BUILD-PLAN.md`, step 8; C101, C93, C105), through
//! the crate's boundary: the counts over the whole item and within one
//! link group (S59), what an item lacks and what is left open at the
//! socket's grain, every route followed by id, the counts view's buckets,
//! the sort, `show`, and every authoring error a socket term has. Every
//! count is worked by hand from the fixture below.

mod common;

use acquisition_search::{Request, answer, show, sockets};
use acquisition_store::Store;
use common::*;
use serde_json::{Value, json};

fn socket(colour: &str, group: i64) -> Value {
    let attr = match colour {
        "R" => "S",
        "G" => "D",
        "B" => "I",
        "W" => "G",
        other => other,
    };
    json!({ "group": group, "attr": attr, "sColour": colour })
}

fn sockets_of(each: &[(&str, i64)]) -> Value {
    json!({ "sockets": each.iter().map(|(c, g)| socket(c, *g)).collect::<Vec<_>>() })
}

/// Ten pc items in Standard and one in poe2.
///
/// `Gear` (g1): `six`, one group `R-R-G-B-B-W`; `split`, two groups `R-R`
/// and `G-B`; `four`, one group `R-R-G-B` (the owner's example, S59);
/// `abyss`, a belt with one abyssal socket `A`; `empty`, GGG's `[]`;
/// `ring`, no sockets at all; `junk`, one red socket and an element that
/// is no socket; `blind`, one socket with an `attr` and no `sColour`;
/// `loose`, one red socket with no `group`; `notarray`, `sockets` a
/// string. `Vault` (v1) in poe2: `p2bow`, two poe2 sockets `{group,
/// type}` with no colour.
fn stash() -> Store {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("g1", "Gear")]), 10);
    list_tabs(&mut s, "poe2", "Standard", json!([tab("v1", "Vault")]), 11);
    let rare =
        |id: &str, base: &str, more: Value| item(id, &format!("Item {id}"), base, "Rare", more);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "g1",
        "Gear",
        vec![
            rare(
                "six",
                "Vaal Regalia",
                sockets_of(&[("R", 0), ("R", 0), ("G", 0), ("B", 0), ("B", 0), ("W", 0)]),
            ),
            rare(
                "split",
                "Astral Plate",
                sockets_of(&[("R", 0), ("R", 0), ("G", 1), ("B", 1)]),
            ),
            rare(
                "four",
                "Glorious Plate",
                sockets_of(&[("R", 0), ("R", 0), ("G", 0), ("B", 0)]),
            ),
            rare("abyss", "Stygian Vise", sockets_of(&[("A", 0)])),
            rare("empty", "Rusted Sword", json!({ "sockets": [] })),
            rare("ring", "Iron Ring", json!({})),
            rare(
                "junk",
                "Iron Hat",
                json!({ "sockets": [socket("R", 0), 5] }),
            ),
            rare(
                "blind",
                "Iron Hat",
                json!({ "sockets": [{ "group": 0, "attr": "S" }] }),
            ),
            rare(
                "loose",
                "Iron Hat",
                json!({ "sockets": [{ "sColour": "R" }] }),
            ),
            rare("notarray", "Iron Hat", json!({ "sockets": "six" })),
        ],
        20,
    );
    fetch_tab(
        &mut s,
        "poe2",
        "Standard",
        "v1",
        "Vault",
        vec![rare(
            "p2bow",
            "Crude Bow",
            json!({ "sockets": [{ "group": 0, "type": "gem" }, { "group": 1, "type": "gem" }] }),
        )],
        21,
    );
    s
}

const PC: [&str; 10] = [
    "abyss", "blind", "empty", "four", "junk", "loose", "notarray", "ring", "six", "split",
];

fn asked(s: &Store, realm: &str, text: &str) -> Value {
    as_json(&ask(&load(s, Some(realm)), text).unwrap())
}

fn view(s: &Store, text: &str, view: Value) -> Result<Value, Value> {
    let request: Request =
        serde_json::from_value(json!({ "query": { "text": text }, "view": view })).unwrap();
    answer(&load(s, Some("pc")), &request)
        .map(|a| as_json(&a))
        .map_err(|e| e.to_json())
}

fn counts(term: &Value) -> [u64; 4] {
    ["matched", "failed", "lacked", "undecided"].map(|k| term[k]["count"].as_u64().unwrap())
}

fn follow(s: &Store, count: &Value) -> Vec<String> {
    if count["count"] == 0 {
        assert!(count.get("request").is_none());
        return Vec::new();
    }
    let request = serde_json::from_value(count["request"].clone()).unwrap();
    let routed = as_json(&answer(&load(s, Some("pc")), &request).unwrap());
    assert_eq!(routed["total"]["matched"], count["count"]);
    ids(&routed)
}

/// Every route of a term returns exactly what it counted, and the four
/// partition the scope (invariant 4).
fn routes_partition(s: &Store, text: &str) -> Value {
    let a = asked(s, "pc", text);
    for term in a["terms"].as_array().unwrap() {
        let mut all: Vec<String> = Vec::new();
        for kind in ["matched", "failed", "lacked", "undecided"] {
            all.extend(follow(s, &term[kind]));
        }
        all.sort();
        assert_eq!(all, PC, "{} in `{text}`", term["term"]);
    }
    a
}

/// C101: `sockets` counts every socket; a comparison on what the item
/// lacks is false (C93) and what could not be read is open at the socket's
/// grain — an element that was no socket may be one, so `junk` is
/// established at neither 1 nor 2 and decided where the interval agrees.
#[test]
fn c101_sockets_counts_every_socket_and_a_count_is_decided_where_its_interval_agrees() {
    let s = stash();
    let a = routes_partition(&s, "sockets>=5");
    // six matched; split, four, abyss, empty, junk (at most 2), blind and
    // loose failed; ring lacked; notarray undecided
    assert_eq!(counts(&a["terms"][0]), [1, 7, 1, 1]);
    assert_eq!(ids(&a), ["six"]);
    let a = routes_partition(&s, "sockets>=1");
    // junk is at least 1: matched
    assert_eq!(counts(&a["terms"][0]), [7, 1, 1, 1]);
    assert_eq!(ids(&asked(&s, "pc", "sockets=0")), ["empty"]);
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(sockets=1)")),
        ["junk", "notarray"]
    );
    assert_eq!(ids(&asked(&s, "pc", "-has:sockets")), ["ring"]);
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(has:sockets)")),
        ["notarray"]
    );
    // the row shows the count and the layout
    let row = &asked(&s, "pc", "sockets>=5")["rows"][0]["matched"][0]["shows"];
    assert_eq!(
        row,
        &json!([
            { "value": { "name": "sockets", "value": 6 } },
            { "shown": { "part": "sockets", "text": "R-R-G-B-B-W" } }
        ])
    );
    // an interval that matched shows what was read
    let row = &asked(&s, "pc", "id:junk sockets>=1")["rows"][0]["matched"][0]["shows"];
    assert_eq!(
        row[0],
        json!({ "value": { "name": "sockets", "value": "1..2" } })
    );
    assert_eq!(row[1]["shown"]["text"], "R 1 element unread");
}

/// C101: a colour counted over the whole item, in the reference's words
/// for GGG's letters; an abyssal socket is a socket of no colour word; a
/// socket whose colour could not be read leaves only the colour counts
/// open, and one whose group could not be read leaves only `links`.
#[test]
fn c101_a_colour_over_the_whole_item_and_what_each_unread_socket_leaves_open() {
    let s = stash();
    let a = routes_partition(&s, "sockets.red>=2");
    // six, split, four matched; abyss, empty, loose and blind (0..1, so
    // never 2) failed; ring lacked; junk (1..2) and notarray undecided
    assert_eq!(counts(&a["terms"][0]), [3, 4, 1, 2]);
    assert_eq!(ids(&asked(&s, "pc", "sockets.red=0")), ["abyss", "empty"]);
    assert_eq!(ids(&asked(&s, "pc", "sockets.white>=1")), ["six"]);
    // blind: its count of sockets and its links are read, its colour is not
    assert_eq!(
        ids(&asked(&s, "pc", "id:blind sockets=1 links=1")),
        ["blind"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(sockets.red)")),
        ["blind", "junk", "notarray"]
    );
    // loose: its socket and its colour are read, its group is not
    assert_eq!(
        ids(&asked(&s, "pc", "id:loose sockets=1 sockets.red=1 links=1")),
        ["loose"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(links)")),
        ["junk", "notarray"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "undecided(sockets)")),
        ["junk", "notarray"]
    );
    // the reasons name the socket and what was wrong with it
    let why = &asked(&s, "pc", "id:blind sockets.red>=1")["total"]["undecided_items"][0]["why"];
    assert_eq!(why[0]["unread"], "a socket's colour");
    assert_eq!(
        why[0]["problem"],
        "`sockets[0]` has an `attr` and no `sColour`: the colour is not read from the attribute"
    );
    assert_eq!(
        asked(&s, "pc", "id:loose links=1")["total"]["undecided"]["count"],
        0
    );
    let why = &asked(&s, "pc", "id:junk sockets=1")["total"]["undecided_items"][0]["why"];
    assert_eq!(why[0]["unread"], "the sockets");
    assert_eq!(why[0]["problem"], "`sockets[1]` is a number, not a socket");
    // a poe2 socket has no colour, known
    let p2 = asked(&s, "poe2", "sockets=2 links=1 sockets.red=0");
    assert_eq!(ids(&p2), ["p2bow"]);
    assert_eq!(p2["total"]["undecided"]["count"], 0);
}

/// C101, S59: within a link group every count named holds together on one
/// group — `R-R-G-B` matches `red>=2 blue>=1` and `red>=1 green>=1`, not
/// `red>=3`; `R-R` beside `G-B` has the colours and no group with both.
/// An item with no link group lacks it, and `-has:links` is its route.
#[test]
fn c101_s59_a_link_group_holds_every_count_named_together() {
    let s = stash();
    let a = routes_partition(&s, "linked(red>=2 blue>=1)");
    // six and four matched; split, abyss, blind and loose (one socket,
    // never two reds) failed; empty and ring lacked; junk and notarray
    // undecided
    assert_eq!(counts(&a["terms"][0]), [2, 4, 2, 2]);
    assert_eq!(ids(&a), ["four", "six"]);
    assert_eq!(
        ids(&asked(&s, "pc", "id:four linked(red>=1 green>=1)")),
        ["four"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "id:four linked(red>=3)")),
        Vec::<String>::new()
    );
    // the two ways to ask a colour (C101)
    assert_eq!(
        ids(&asked(&s, "pc", "sockets.red>=2 sockets.green>=1")),
        ["four", "six", "split"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "linked(red>=2 green>=1)")),
        ["four", "six"]
    );
    assert_eq!(ids(&asked(&s, "pc", "linked(size>=5 blue>=2)")), ["six"]);
    assert_eq!(ids(&asked(&s, "pc", "linked(size=2)")), ["split"]);
    assert_eq!(
        ids(&asked(&s, "pc", "linked(red>=2 or blue>=2)")),
        ["four", "six", "split"]
    );
    assert_eq!(
        ids(&asked(&s, "pc", "linked(-red>=1 size>=1)")),
        ["abyss", "split"]
    );
    assert_eq!(ids(&asked(&s, "pc", "-has:links")), ["empty", "ring"]);
    assert_eq!(ids(&asked(&s, "pc", "links>=6")), ["six"]);
    // the row shows the group that held, as the game shows it
    assert_eq!(
        asked(&s, "pc", "id:split linked(size=2)")["rows"][0]["matched"][0]["shows"],
        json!([
            { "shown": { "part": "link group", "text": "R-R" } },
            { "shown": { "part": "link group", "text": "G-B" } }
        ])
    );
    // the failed route is the field's shape
    assert_eq!(
        a["terms"][0]["failed"]["request"]["query"]["text"],
        "has:links -linked(red>=2 blue>=1)"
    );
    assert_eq!(
        a["terms"][0]["lacked"]["request"]["query"]["text"],
        "-has:links"
    );
}

/// C105: `--count sockets` and `--count links` account for every match —
/// a value, `none` where the item lacks the count, `undecided` where the
/// count is not established — the buckets summing to the total, each
/// route returning its members; the sum and the sort read the same
/// status.
#[test]
fn c105_a_socket_count_is_a_key_whose_buckets_sum_to_the_total() {
    let s = stash();
    let shape = |table: &Value| -> Vec<(String, u64)> {
        table["buckets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| {
                let label = match b["bucket"].as_str().unwrap() {
                    "value" => b["value"].to_string(),
                    other => format!("({other})"),
                };
                (label, b["count"].as_u64().unwrap())
            })
            .collect()
    };
    let shaped = |pairs: &[(&str, u64)]| -> Vec<(String, u64)> {
        pairs.iter().map(|(l, n)| (l.to_string(), *n)).collect()
    };
    let a = view(
        &s,
        "",
        json!({ "counts": { "keys": ["sockets", "links"], "sum": "sockets" } }),
    )
    .unwrap();
    assert_eq!(a["total"]["matched"], 10);
    let sockets = &a["view"]["counts"]["tables"][0];
    // six 6; split, four 4; abyss, blind, loose 1; empty 0; ring none;
    // junk, notarray undecided
    assert_eq!(
        shape(sockets),
        shaped(&[
            ("1", 3),
            ("4", 2),
            ("0", 1),
            ("6", 1),
            ("(none)", 1),
            ("(undecided)", 2)
        ])
    );
    let links = &a["view"]["counts"]["tables"][1];
    // six 6; four 4; split 2; abyss, blind, loose 1; empty, ring none;
    // junk, notarray undecided
    assert_eq!(
        shape(links),
        shaped(&[
            ("1", 3),
            ("2", 1),
            ("4", 1),
            ("6", 1),
            ("(none)", 2),
            ("(undecided)", 2)
        ])
    );
    for table in [sockets, links] {
        let mut all: Vec<String> = Vec::new();
        for bucket in table["buckets"].as_array().unwrap() {
            all.extend(follow(&s, bucket));
        }
        all.sort();
        assert_eq!(all, PC);
    }
    // the sum: 6 + 4 + 4 + 1 + 0 + 1 + 1 = 17 over the established, ring
    // lacking, junk and notarray unread, so incomplete
    assert_eq!(
        a["view"]["counts"]["sum"],
        json!({ "name": "sockets", "value": 18, "lacking": 1, "incomplete": true, "unread": 2 })
    );
    // the sort: values first, largest down, the not established last
    let sorted: Request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "" },
        "view": { "rows": { "sort": "links", "desc": true } },
    }))
    .unwrap();
    let r = as_json(&answer(&load(&s, Some("pc")), &sorted).unwrap());
    let order: Vec<&str> = r["rows"]
        .as_array()
        .unwrap()
        .iter()
        .take(6)
        .map(|row| row["id"].as_str().unwrap())
        .collect();
    assert_eq!(order, ["six", "four", "split", "abyss", "blind", "loose"]);
    let last: Vec<(&str, &Value)> = r["rows"]
        .as_array()
        .unwrap()
        .iter()
        .skip(6)
        .map(|row| (row["id"].as_str().unwrap(), &row["sort"]))
        .collect();
    for (id, sort) in last {
        assert!(
            sort["value"].is_null() || sort["status"].is_string(),
            "{id}: {sort}"
        );
    }
    assert_eq!(r["rows"][8]["sort"]["status"], "incomplete");
}

/// `show` gives the sockets as read — group, colour, what was unread of
/// each — and the layout is made once, for `show` and a row alike.
#[test]
fn show_gives_every_socket_as_read_and_the_layout_is_one_makers() {
    let s = stash();
    let shown = |id: &str| serde_json::to_value(show(&s, id, false).unwrap()).unwrap();
    assert_eq!(
        shown("split")["item"]["sockets"],
        json!([
            { "group": 0, "colour": "R" }, { "group": 0, "colour": "R" },
            { "group": 1, "colour": "G" }, { "group": 1, "colour": "B" }
        ])
    );
    assert_eq!(
        shown("blind")["item"]["sockets"],
        json!([{ "group": 0, "colour": null, "colour_unread": true }])
    );
    assert_eq!(
        shown("loose")["item"]["sockets"],
        json!([{ "group": null, "colour": "R" }])
    );
    assert_eq!(shown("ring")["item"].get("sockets"), None);
    assert_eq!(shown("empty")["item"]["sockets"], json!([]));
    assert_eq!(shown("notarray")["item"].get("sockets"), None);
    assert_eq!(shown("notarray")["item"]["unread"][0]["part"], "sockets");
    assert_eq!(
        shown("blind")["item"]["unread"][0],
        json!({ "part": "socket_colour", "problem": "`sockets[0]` has an `attr` and no `sColour`: the colour is not read from the attribute", "socket": 0 })
    );
    let p2 = serde_json::to_value(show(&s, "p2bow", false).unwrap()).unwrap();
    assert_eq!(
        p2["item"]["sockets"][0],
        json!({ "group": 0, "colour": null, "type": "gem" })
    );
    let layout = |id: &str| {
        let item: acquisition_search::Item = show(&s, id, false).map(|s| s.item).unwrap();
        sockets::layout(&item)
    };
    assert_eq!(layout("split").as_deref(), Some("R-R G-B"));
    assert_eq!(layout("abyss").as_deref(), Some("A"));
    assert_eq!(layout("junk").as_deref(), Some("R 1 element unread"));
    assert_eq!(layout("blind").as_deref(), Some("?"));
    assert_eq!(layout("loose").as_deref(), Some("R (group unread)"));
    assert_eq!(layout("empty").as_deref(), Some(""));
    assert_eq!(layout("ring"), None);
    assert_eq!(layout("p2bow").as_deref(), Some("gem gem"));
}

/// What a socket term cannot be is an authoring error before anything is
/// evaluated, with what was meant offered: a colour word outside the
/// four, a count a link group has none of, a template or a flag inside
/// `linked( … )`, a text operator on a number.
#[test]
fn a_socket_term_the_language_cannot_say_is_an_authoring_error_with_readings() {
    let s = stash();
    let corpus = load(&s, Some("pc"));
    for (text, kind, reading) in [
        ("sockets.pink>=1", "unknown_value", Some("sockets.red>=1")),
        ("linked(arg1>=3)", "unknown_name", None),
        ("linked(reds>=3)", "unknown_name", Some("linked(red>=3)")),
        ("linked(template:life)", "unknown_name", None),
        ("linked(is:crafted)", "unknown_name", None),
        ("linked(red:3)", "operator_mismatch", None),
        ("sockets:3", "operator_mismatch", Some("sockets=3")),
        ("linked(\"# to maximum Life\")", "not_inside_group", None),
        ("has:linked", "unknown_name", None),
    ] {
        let e = ask(&corpus, text).unwrap_err();
        let json = e.to_json();
        assert_eq!(json["kind"], kind, "`{text}`: {e}");
        if let Some(reading) = reading {
            assert_eq!(json["readings"][0], reading, "`{text}`: {e}");
        }
    }
    let e = ask(&corpus, "sockets.pink>=1").unwrap_err().to_json();
    assert_eq!(
        e["readings"],
        json!([
            "sockets.red>=1",
            "sockets.green>=1",
            "sockets.blue>=1",
            "sockets.white>=1"
        ])
    );
    // the help names the four and the link group's counts
    let d = serde_json::to_value(acquisition_search::describe(&["sockets".to_string()]).unwrap())
        .unwrap();
    let names: Vec<&str> = d["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["sockets", "sockets.<colour>"]);
    assert_eq!(
        d["fields"][1]["values"],
        json!(["red", "green", "blue", "white"])
    );
    let d = serde_json::to_value(acquisition_search::describe(&["linked".to_string()]).unwrap())
        .unwrap();
    assert_eq!(d["linked"].as_array().unwrap().len(), 2);
}

/// A group is GGG's number, not a run: `R G R` numbered 0, 1, 0 is two
/// groups, `R-R` and `G` (the C++ app's run rule, S22, would say three of
/// one). A socket whose group could not be read may sit in any group or
/// make one of its own, so it widens `links` and every group's `size` by
/// one and leaves what it could turn open — and nothing else.
#[test]
fn a_group_is_ggg_s_number_and_an_unplaced_socket_widens_every_group() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "T")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "T",
        vec![
            item(
                "weave",
                "Weave",
                "Iron Hat",
                "Rare",
                sockets_of(&[("R", 0), ("G", 1), ("R", 0)]),
            ),
            item(
                "mixed",
                "Mixed",
                "Iron Hat",
                "Rare",
                json!({ "sockets": [socket("R", 0), socket("R", 0), { "sColour": "G" }] }),
            ),
        ],
        20,
    );
    let asked = |text: &str| as_json(&ask(&load(&s, Some("pc")), text).unwrap());
    assert_eq!(
        ids(&asked("id:weave links=2 sockets=3 sockets.red=2")),
        ["weave"]
    );
    let weave: acquisition_search::Item = show(&s, "weave", false).unwrap().item;
    assert_eq!(sockets::layout(&weave).as_deref(), Some("R-R G"));
    // mixed: 3 sockets, 2 red and 1 green, read; its largest group is 2 or 3
    assert_eq!(
        ids(&asked("id:mixed sockets=3 sockets.red=2 sockets.green=1")),
        ["mixed"]
    );
    assert_eq!(ids(&asked("id:mixed links>=2")), ["mixed"]);
    assert_eq!(ids(&asked("id:mixed links>=4")), Vec::<String>::new());
    // counted over the scope: weave's 2 and mixed's at most 3 both fail
    assert_eq!(asked("id:mixed links>=4")["terms"][1]["failed"]["count"], 2);
    assert_eq!(ids(&asked("undecided(links)")), ["mixed"]);
    assert_eq!(ids(&asked("id:mixed undecided(links=2)")), ["mixed"]);
    let why = &asked("id:mixed links=2")["total"]["undecided_items"][0]["why"];
    assert_eq!(why[0]["unread"], "a socket's group");
    assert_eq!(why[0]["problem"], "`sockets[2]` has no `group`");
    assert_eq!(ids(&asked("undecided(sockets)")), Vec::<String>::new());
    // within a group: the size it may reach is open, its floor is not
    assert_eq!(
        ids(&asked("id:mixed undecided(linked(red>=2 size=2))")),
        ["mixed"]
    );
    assert_eq!(ids(&asked("id:mixed linked(red>=2 size>=2)")), ["mixed"]);
    assert_eq!(
        ids(&asked("id:mixed linked(green>=1 size>=1)")),
        Vec::<String>::new()
    );
    assert_eq!(
        ids(&asked("id:mixed undecided(linked(green>=1 size>=1))")),
        ["mixed"]
    );
    let mixed: acquisition_search::Item = show(&s, "mixed", false).unwrap().item;
    assert_eq!(
        sockets::layout(&mixed).as_deref(),
        Some("R-R G (group unread)")
    );
}

/// An outside review of step 8 (2026-09-24), each finding reproduced
/// here before it was fixed: a `linked( … )` left open by a socket's
/// colour says why; a socket whose group is unread adds to its own
/// colour's count and to no other; alone in the collection it is a group
/// of one, exactly, so `links=1` and `has:links` hold and nothing sorts
/// as incomplete; and `--describe sockets.red` answers.
#[test]
fn review_a_socket_whose_group_is_unread_is_a_socket_still_and_an_open_group_says_why() {
    let mut s = store();
    list_tabs(&mut s, "pc", "Standard", json!([tab("t1", "T")]), 10);
    fetch_tab(
        &mut s,
        "pc",
        "Standard",
        "t1",
        "T",
        vec![
            item(
                "blind",
                "Blind",
                "Iron Hat",
                "Rare",
                json!({ "sockets": [{ "group": 0, "attr": "S" }] }),
            ),
            item(
                "green",
                "Green",
                "Iron Hat",
                "Rare",
                json!({ "sockets": [socket("R", 0), { "sColour": "G" }] }),
            ),
            item(
                "alone",
                "Alone",
                "Iron Hat",
                "Rare",
                json!({ "sockets": [{ "sColour": "R" }] }),
            ),
        ],
        20,
    );
    let asked = |text: &str| as_json(&ask(&load(&s, Some("pc")), text).unwrap());
    // 1: the reason names the colour
    let a = asked("id:blind linked(red>=1)");
    assert_eq!(a["total"]["undecided"]["count"], 1);
    let why = &a["total"]["undecided_items"][0]["why"];
    assert_eq!(why[0]["unread"], "a socket's colour");
    assert_eq!(asked("id:blind linked(size>=1)")["total"]["matched"], 1);
    // 2: an unplaced green socket cannot turn a blue count, and may turn green
    // blind's one socket may be blue: open, and rightly
    assert_eq!(ids(&asked("linked(blue=0)")), ["alone", "green"]);
    assert_eq!(ids(&asked("undecided(linked(blue=0))")), ["blind"]);
    assert_eq!(
        ids(&asked("id:green linked(green>=1)")),
        Vec::<String>::new()
    );
    assert_eq!(
        ids(&asked("id:green undecided(linked(green>=1))")),
        ["green"]
    );
    assert_eq!(ids(&asked("id:green linked(red>=1 green<=1)")), ["green"]);
    // 3: a single socket is a group of one
    assert_eq!(
        ids(&asked(
            "id:alone links=1 has:links linked(red>=1 size=1) sockets.red=1"
        )),
        ["alone"]
    );
    assert_eq!(ids(&asked("undecided(links)")), ["green"]);
    // round 2: the socket alone is counted once — its colour is exactly
    // one, and one socket never holds two of anything
    assert_eq!(ids(&asked("id:alone linked(red=1 size=1)")), ["alone"]);
    assert_eq!(ids(&asked("id:alone linked(red>=2)")), Vec::<String>::new());
    // over the scope: alone, blind and green each hold one socket per group
    assert_eq!(asked("linked(red>=2)")["terms"][0]["failed"]["count"], 3);
    assert_eq!(asked("linked(red>=2)")["terms"][0]["undecided"]["count"], 0);
    // each comparison is read over the interval on its own: the two
    // together cover it, and the answer is careful, never wrong
    assert_eq!(
        ids(&asked("id:blind undecided(linked(red=1 or red=0))")),
        ["blind"]
    );
    let sorted: Request = serde_json::from_value(json!({
        "scope": { "realm": "pc" },
        "query": { "text": "id:alone" },
        "view": { "rows": { "sort": "links" } },
    }))
    .unwrap();
    let r = as_json(&answer(&load(&s, Some("pc")), &sorted).unwrap());
    assert_eq!(r["rows"][0]["sort"], json!({ "value": 1 }));
    // 4: the help answers to each colour field
    let d =
        serde_json::to_value(acquisition_search::describe(&["sockets.red".to_string()]).unwrap())
            .unwrap();
    assert_eq!(d["fields"][0]["name"], "sockets.<colour>");
    // round 2: in any case, as every name is (B1)
    let d =
        serde_json::to_value(acquisition_search::describe(&["SOCKETS.RED".to_string()]).unwrap())
            .unwrap();
    assert_eq!(d["fields"][0]["name"], "sockets.<colour>");
}
