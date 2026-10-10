//! Process-level pin of `acq search` and `acq show` (`search/BUILD-PLAN.md`,
//! rule 3: the crate's boundary, and the CLI's `--json`): the document
//! `--json` prints is the search crate's answer whole, the text is a
//! function of it (C53), a failure is structured on both paths (C11), a
//! flag of a later step is refused by its name, and every command an answer
//! prints is one this build runs (rule 5). Facts are seeded through the
//! store crate; no daemon runs — `ACQ_NO_SPAWN=1` makes any contact an
//! error.

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use acquisition_store::{Endpoint, Index, Store, account_path};
use serde_json::{Value, json};

const USER: &str = "Alice#1234";

fn command(base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_STORE_DIR", base)
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_NO_SPAWN", "1")
        .env("ACQ_PROVIDER", "mock");
    for var in ["ACQ_GGG", "ACQ_ACCOUNT"] {
        cmd.env_remove(var);
    }
    cmd
}

fn acq(base: &Path, args: &[&str]) -> Output {
    command(base, args).output().expect("spawning acq")
}

fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

fn text(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn base() -> std::path::PathBuf {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "acq-search-cli-{}-{}",
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    // a process id comes round again: never open what an earlier run left
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Two fetched tabs and one never fetched: `kaom`, a unique whose name
/// holds an apostrophe, sockets `R-R-G B`; `r1` with 95 life; `r2` with 20 and 75; `r3`
/// whose implicit array is unread; `plain`, with no life line, so that a
/// route starting with the language's `-` is printed.
fn seed(base: &Path) {
    let mock = base.join("mock");
    std::fs::create_dir_all(&mock).unwrap();
    let mut index = Index::load(&mock).unwrap();
    index.record_login(USER, "u-search", false, 1).unwrap();
    let mut store = Store::open(&account_path(&mock, USER)).unwrap();
    let mut record = |ep: Endpoint, body: Value, at: i64| {
        store
            .record(
                &ep,
                &json!({ "realm": "pc", "league": "Standard" }),
                200,
                &body,
                at,
            )
            .unwrap();
    };
    record(
        Endpoint::Profile,
        json!({ "uuid": "u-search", "name": USER }),
        1,
    );
    record(
        Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        },
        json!({ "stashes": [
            { "id": "t1", "name": "Rings", "type": "PremiumStash" },
            { "id": "t2", "name": "Dump", "type": "PremiumStash" },
            { "id": "t3", "name": "Never", "type": "PremiumStash" } ] }),
        10,
    );
    let item = |id: &str, name: &str, base: &str, rarity: &str, more: Value| {
        let mut v = json!({ "id": id, "name": name, "typeLine": base, "baseType": base, "rarity": rarity,
                            "frameTypeId": rarity, "identified": true, "ilvl": 84, "x": 0, "y": 0 });
        for (k, value) in more.as_object().unwrap() {
            v[k] = value.clone();
        }
        v
    };
    let stash = |id: &str| Endpoint::Stash {
        realm: "pc".into(),
        league: "Standard".into(),
        id: id.into(),
        sub: None,
    };
    record(
        stash("t1"),
        json!({ "stash": { "id": "t1", "name": "Rings", "type": "PremiumStash", "items": [
            item("r1", "Doom Loop", "Two-Stone Ring", "Rare", json!({ "explicitMods": ["+95 to maximum Life"] })),
            item("r2", "Rune Coil", "Two-Stone Ring", "Rare", json!({ "implicitMods": ["+20 to maximum Life"], "explicitMods": ["+75 to maximum Life"] })),
        ] } }),
        20,
    );
    record(
        stash("t2"),
        json!({ "stash": { "id": "t2", "name": "Dump", "type": "PremiumStash", "items": [
            item("kaom", "Kaom's Heart", "Glorious Plate", "Unique", json!({ "explicitMods": ["+500 to maximum Life"],
                "sockets": [{ "group": 0, "attr": "S", "sColour": "R" }, { "group": 0, "attr": "S", "sColour": "R" },
                            { "group": 0, "attr": "D", "sColour": "G" }, { "group": 1, "attr": "I", "sColour": "B" }] })),
            item("r3", "Hex Band", "Two-Stone Ring", "Rare", json!({ "implicitMods": "unreadable", "explicitMods": ["+40 to maximum Life"] })),
            item("plain", "Dull Turn", "Iron Ring", "Rare", json!({ "explicitMods": ["+20 to maximum Mana"] })),
        ] } }),
        30,
    );
}

/// A second realm in the same store: one ring in a poe2 tab named as the
/// pc one is, a ring there with two identical life lines, and a pc item
/// with more lines than a row shows.
fn seed_more(base: &Path) {
    let mut store = Store::open(&account_path(&base.join("mock"), USER)).unwrap();
    let mut record = |ep: Endpoint, realm: &str, body: Value, at: i64| {
        store
            .record(
                &ep,
                &json!({ "realm": realm, "league": "Standard" }),
                200,
                &body,
                at,
            )
            .unwrap();
    };
    let ring = |id: &str, mods: Vec<String>| {
        json!({ "id": id, "name": "", "typeLine": "Iron Ring", "baseType": "Iron Ring", "rarity": "Rare",
                "frameTypeId": "Rare", "identified": true, "ilvl": 84, "x": 0, "y": 0, "explicitMods": mods })
    };
    record(
        Endpoint::Stashes {
            realm: "poe2".into(),
            league: "Standard".into(),
        },
        "poe2",
        json!({ "stashes": [{ "id": "p1", "name": "Rings", "type": "PremiumStash" }] }),
        40,
    );
    record(
        Endpoint::Stash {
            realm: "poe2".into(),
            league: "Standard".into(),
            id: "p1".into(),
            sub: None,
        },
        "poe2",
        json!({ "stash": { "id": "p1", "name": "Rings", "type": "PremiumStash",
            "items": [ring("two", vec!["+30 to Spirit".to_string()]),
                      ring("twin", vec!["+10 to maximum Life".to_string(), "+20 to maximum Life".to_string(), "+10 to maximum Life".to_string()])] } }),
        50,
    );
    record(
        Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "t3".into(),
            sub: None,
        },
        "pc",
        json!({ "stash": { "id": "t3", "name": "Never", "type": "PremiumStash",
            "items": [ring("many", (1..=8).map(|n| format!("+{n} to Spirit")).collect())] } }),
        60,
    );
}

/// Outside audit, 2026-09-21: under an all-realms scope a row says
/// which realm its item is in, and a row that shows part of what a term
/// touched says how much it left out and where the whole is.
#[test]
fn c96_c100_a_row_names_its_realm_under_all_and_says_what_it_left_out() {
    let base = base();
    seed(&base);
    seed_more(&base);
    let shown = text(&acq(
        &base,
        &["search", "--realm", "all", "line(template:spirit)"],
    ));
    for needle in [
        "poe2 · Standard / Rings",
        "pc · Standard / Never",
        "2 more of term 0: acq show --account 'Alice#1234' many",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }
    // one realm in scope: the scope line has said it
    let shown = text(&acq(
        &base,
        &["search", "--realm", "pc", "line(template:spirit)"],
    ));
    assert!(shown.contains(" · Standard / Never"), "{shown}");
    assert!(!shown.contains("pc · Standard / Never"), "{shown}");
}

/// Every `acq …` a text prints, to the end of its line — but a hint that
/// names a flag this build refuses, which says so itself.
fn printed_commands(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| !line.contains("not built"))
        .filter_map(|line| line.find("acq ").map(|at| line[at..].trim().to_string()))
        .collect()
}

fn run_printed(base: &Path, command: &str) -> Output {
    Command::new("sh")
        .arg("-c")
        .arg(command.replacen("acq ", "\"$ACQ\" --json ", 1))
        .env("ACQ", env!("CARGO_BIN_EXE_acq"))
        .env("ACQ_STORE_DIR", base)
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_NO_SPAWN", "1")
        .env("ACQ_PROVIDER", "mock")
        .env_remove("ACQ_ACCOUNT")
        .output()
        .unwrap()
}

/// Rule 5 of the build plan, over everything and not only the routes: a
/// command the search prints — a route, where a cut block sends its
/// reader, what an error offers — runs as printed. With two accounts
/// known, which a command that drops its account does not survive (outside
/// review, 2026-09-21).
#[test]
fn rule_5_every_command_printed_runs_with_a_second_account_known() {
    let base = base();
    seed(&base);
    seed_more(&base);
    let mock = base.join("mock");
    let mut index = Index::load(&mock).unwrap();
    index.record_login("Bob#5678", "u-other", false, 2).unwrap();
    let mut other = Store::open(&account_path(&mock, "Bob#5678")).unwrap();
    other
        .record(
            &Endpoint::Profile,
            &json!({}),
            200,
            &json!({ "uuid": "u-other", "name": "Bob#5678" }),
            2,
        )
        .unwrap();
    drop(other);
    // the premise: with no account named, nothing runs
    assert!(!acq(&base, &["search", "--realm", "pc"]).status.success());

    let mut ran = 0;
    for query in [
        "line(template:spirit)",
        "line(template:life arg1>=90)",
        "is:corrupted",
        "linked(red>=2) sockets.blue>=1",
    ] {
        let shown = text(&acq(
            &base,
            &[
                "--account",
                USER,
                "search",
                "--realm",
                "all",
                query,
                "--routes",
            ],
        ));
        let commands = printed_commands(&shown);
        assert!(!commands.is_empty(), "{shown}");
        for command in commands {
            let out = run_printed(&base, &command);
            assert!(
                out.status.success(),
                "`{command}` was printed and does not run: {}",
                String::from_utf8_lossy(&out.stdout)
            );
            ran += 1;
        }
    }
    // an answer cut at its limit: the continuation it prints, and what a
    // continuation refused across a changed basis offers
    let shown = text(&acq(
        &base,
        &[
            "--account",
            USER,
            "search",
            "--realm",
            "all",
            "--limit",
            "2",
        ],
    ));
    let more: Vec<String> = printed_commands(&shown)
        .into_iter()
        .filter(|command| command.contains("--next"))
        .collect();
    assert_eq!(more.len(), 1, "{shown}");
    assert!(run_printed(&base, &more[0]).status.success(), "{}", more[0]);
    ran += 1;

    // what a zero answer offers, where the query starts with the
    // language's not and goes after `--`, the flags before it; and the
    // routes of an explained node's forced counts
    for args in [
        vec!["--routes", "--", "-is:identified"],
        vec!["--count", "tab", "--", "-is:identified -has:name"],
        vec!["line(template:life arg1>=90)", "--explain", "0.1"],
    ] {
        let mut all = vec!["--account", USER, "search", "--realm", "pc"];
        all.extend(args);
        let shown = text(&acq(&base, &all));
        let commands = printed_commands(&shown);
        assert!(commands.len() >= 2, "{shown}");
        for command in commands {
            let out = run_printed(&base, &command);
            assert!(
                out.status.success(),
                "`{command}` was printed and does not run: {}",
                String::from_utf8_lossy(&out.stdout)
            );
            ran += 1;
        }
    }
    // what `show` offers when the id is a tab's
    let refused = sole_json(&acq(&base, &["--account", USER, "--json", "show", "t1"]));
    for offered in refused["readings"].as_array().unwrap() {
        let out = run_printed(&base, offered.as_str().unwrap());
        assert!(out.status.success(), "{offered}");
        ran += 1;
    }
    assert!(ran > 8, "{ran} commands ran");
}

/// The counts view at a terminal (step 5): `--count` spelled as the
/// reference's synopsis spells it, `line:` taking the rest of the list; the
/// JSON is the crate's view whole; and every route the text prints — a
/// pattern that turns case back on, a name with an apostrophe, a not after
/// `--` — runs through a shell and returns exactly the members counted.
#[test]
fn step_5_a_count_at_a_terminal_and_every_route_it_prints_returns_what_it_counted() {
    let base = base();
    seed(&base);
    // the tab never fetched, fetched: a fire line GGG spelled two ways
    let mut store = Store::open(&account_path(&base.join("mock"), USER)).unwrap();
    let ring = |id: &str, line: &str| {
        json!({ "id": id, "name": "", "typeLine": "Ruby Ring", "baseType": "Ruby Ring", "rarity": "Magic",
                "frameTypeId": "Magic", "identified": true, "ilvl": 70, "x": 0, "y": 0, "explicitMods": [line] })
    };
    store
        .record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t3".into(),
                sub: None,
            },
            &json!({ "realm": "pc", "league": "Standard" }),
            200,
            &json!({ "stash": { "id": "t3", "name": "Never", "type": "PremiumStash", "items": [
                ring("f1", "+10% to Fire Resistance"), ring("f2", "+12% to fire Resistance") ] } }),
            40,
        )
        .unwrap();
    drop(store);

    let count = "tab,name,line:life,~\"resist|^adds\"";
    let answer = sole_json(&acq(
        &base,
        &["--json", "search", "--count", count, "--sum", "ilvl"],
    ));
    let counts = &answer["view"]["counts"];
    let keys: Vec<&str> = counts["tables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["key"].as_str().unwrap())
        .collect();
    assert_eq!(keys, ["tab", "name", "line:life", "line~resist|^adds"]);
    assert_eq!(answer["rows"], json!([]));
    // seven items at ilvl 84 and 70: 5 × 84 + 2 × 70
    assert_eq!(counts["sum"]["value"], 560);

    let shown = text(&acq(
        &base,
        &["search", "--count", count, "--sum", "ilvl", "--routes"],
    ));
    assert!(shown.contains("count   tab · 3 values"), "{shown}");
    assert!(
        shown.contains("sum     ilvl over every match: 560"),
        "{shown}"
    );
    // each route's heading says what it counted; the line after it is the
    // command, which must return that many
    let lines: Vec<&str> = shown.lines().collect();
    let mut ran = 0;
    for pair in lines.windows(2) {
        let Some(counted) = pair[0]
            .strip_suffix("), over the scope:")
            .and_then(|head| head.rsplit_once('('))
            .and_then(|(_, n)| n.parse::<u64>().ok())
        else {
            continue;
        };
        let command = pair[1].trim();
        let out = run_printed(&base, command);
        assert!(out.status.success(), "`{command}` does not run");
        assert_eq!(
            sole_json(&out)["total"]["matched"],
            counted,
            "`{command}` under `{}`",
            pair[0]
        );
        ran += 1;
    }
    assert!(ran >= 12, "{ran} routes ran:\n{shown}");
    for printed in ["'name=\"Kaom'\\''s Heart\"'", "(?-i)^", "-- -has:name"] {
        assert!(shown.contains(printed), "`{printed}` in:\n{shown}");
    }

    let crossed = text(&acq(&base, &["search", "--cross", "tab,rarity"]));
    assert!(
        crossed.contains("cross   tab × rarity · 4 cells"),
        "{crossed}"
    );
}

/// Outside audit, 2026-09-22, at the terminal: a quoted key is the
/// text as written and a quote that never closes is an error; the count
/// that lists what a selector resolved to past the listed is printed as a
/// command and lists them; a count with no route says why, in a crossed
/// cell and beneath a vocabulary row.
#[test]
fn keys_are_parsed_strictly_and_text_says_why_a_count_has_no_route() {
    let base = base();
    seed(&base);
    let mut store = Store::open(&account_path(&base.join("mock"), USER)).unwrap();
    let stash = |id: &str| Endpoint::Stash {
        realm: "pc".into(),
        league: "Standard".into(),
        id: id.into(),
        sub: None,
    };
    // twelve ring bases, a rarity outside the list, a flag outside the list
    let mut items: Vec<Value> = (0..12)
        .map(|n| {
            json!({ "id": format!("b{n}"), "name": "", "typeLine": format!("Ring {}", "abcdefghijkl".chars().nth(n).unwrap()),
                "baseType": format!("Ring {}", "abcdefghijkl".chars().nth(n).unwrap()), "rarity": "Rare", "frameTypeId": "Rare",
                "identified": true, "ilvl": 84, "x": 0, "y": 0 })
        })
        .collect();
    items.push(json!({ "id": "relic", "name": "New", "typeLine": "Ring", "baseType": "Ring", "rarity": "Relic",
        "frameTypeId": "Relic", "identified": true, "ilvl": 84, "x": 0, "y": 0,
        "explicitMods": [{ "description": "+7 to maximum Life", "flags": { "Weird": true } }] }));
    store
        .record(
            &stash("t3"),
            &json!({}),
            200,
            &json!({ "stash": { "id": "t3", "name": "Never", "type": "PremiumStash", "items": items } }),
            50,
        )
        .unwrap();
    drop(store);

    // 2: a quote that never closes, an escape the language lacks
    for bad in ["line:\"Life", "line:\"Life\\q\"", "line:\"Life\\"] {
        let out = acq(&base, &["--json", "search", "--count", bad]);
        assert_eq!(out.status.code(), Some(1), "{bad:?}");
        assert_eq!(sole_json(&out)["kind"], "view", "{bad:?}");
    }
    // 2: quoted, a `~` is text and no pattern; unquoted, it is one
    let literal = sole_json(&acq(
        &base,
        &["--json", "search", "--count", "line:\"~Life\""],
    ));
    let table = &literal["view"]["counts"]["tables"][0];
    assert_eq!(
        (&table["key"], &table["values"]),
        (&json!("line:~Life"), &json!(0))
    );
    let pattern = sole_json(&acq(&base, &["--json", "search", "--count", "line:~Life"]));
    assert_eq!(pattern["view"]["counts"]["tables"][0]["values"], 1);

    // 3: the continuation printed is a command, and it lists every base
    let shown = text(&acq(&base, &["search", "base:ring rarity=unique"]));
    let continuation = shown
        .lines()
        .find(|l| l.contains(" more: acq "))
        .unwrap_or_else(|| panic!("no continuation in:\n{shown}"));
    let command = &continuation[continuation.find("acq ").unwrap()..];
    let listed = sole_json(&run_printed(&base, command));
    assert_eq!(
        listed["view"]["counts"]["tables"][0]["values"], 15,
        "{command}"
    );
    assert!(shown.contains("--count base"), "{shown}");

    // 6: a cell with no route says why, and so does a kind
    let crossed = text(&acq(
        &base,
        &["search", "--cross", "rarity,base", "--routes"],
    ));
    assert!(crossed.contains("Relic × Ring"), "{crossed}");
    assert!(
        crossed.contains("Relic: a value outside the closed list of `rarity`"),
        "{crossed}"
    );
    let vocabulary = text(&acq(&base, &["search", "id:relic", "--count", "line"]));
    assert!(
        vocabulary.contains("Weird: outside the closed list of a line's flags"),
        "{vocabulary}"
    );
}

/// Outside review, 2026-09-22: a continuation whose key holds a comma,
/// a quote or a backslash is printed in the count list's own grammar and
/// runs; a quoted text keeps its trailing space through the binder.
#[test]
fn step_5_review_a_continuations_key_is_encoded_and_a_quoted_text_is_kept_whole() {
    let base = base();
    seed(&base);
    let mut store = Store::open(&account_path(&base.join("mock"), USER)).unwrap();
    // twelve templates a pattern with a comma selects, each with a comma
    // and a backslash of its own
    let items: Vec<Value> = (0..12)
        .map(|n| {
            let k = "abcdefghijkl".chars().nth(n).unwrap();
            json!({ "id": format!("m{n}"), "name": "", "typeLine": "Ring", "baseType": "Ring", "rarity": "Rare",
                "frameTypeId": "Rare", "identified": true, "ilvl": 84, "x": 0, "y": 0,
                "explicitMods": [format!("+1 to Marker {k}{k}, half \\ {k}")] })
        })
        .collect();
    store
        .record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t3".into(),
                sub: None,
            },
            &json!({}),
            200,
            &json!({ "stash": { "id": "t3", "name": "Never", "type": "PremiumStash", "items": items } }),
            50,
        )
        .unwrap();
    drop(store);
    let mut ran = 0;
    for query in [
        r#"line(template~"Marker [a-z]{1,2}")"#,
        r#"line(template:", half")"#,
        r#"line(template:"\\")"#,
    ] {
        let shown = text(&acq(&base, &["search", query]));
        let continuation = shown
            .lines()
            .find(|l| l.contains(" more: acq "))
            .unwrap_or_else(|| panic!("no continuation for `{query}` in:\n{shown}"));
        let command = &continuation[continuation.find("acq ").unwrap()..];
        let out = run_printed(&base, command);
        assert!(
            out.status.success(),
            "`{command}`: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        let listed = sole_json(&out);
        assert_eq!(
            listed["view"]["counts"]["tables"][0]["values"], 12,
            "`{command}`"
        );
        ran += 1;
    }
    assert_eq!(ran, 3);
    // a quoted text's trailing space is the text's: nothing ends in `Life `
    let kept = sole_json(&acq(
        &base,
        &["--json", "search", "--count", "line:\"Life \""],
    ));
    let table = &kept["view"]["counts"]["tables"][0];
    assert_eq!(
        (&table["key"], &table["values"]),
        (&json!("line:Life "), &json!(0))
    );
}

const LIFE: &str = r##""+# to maximum Life">=90"##;

#[test]
fn c53_search_json_is_the_answer_whole_and_the_text_is_a_function_of_it() {
    let base = base();
    seed(&base);
    let out = acq(
        &base,
        &[
            "--json",
            "search",
            "--realm",
            "pc",
            LIFE,
            "--sort",
            "line(\"+# to maximum Life\").arg1",
            "--desc",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let a = sole_json(&out);
    assert_eq!(a["query"]["text"], "line(\"# to maximum Life\" arg1>=90)");
    assert!(a["query"]["tree"].is_object());
    assert_eq!(
        a["scope"]["account"],
        json!({ "uuid": "u-search", "name": USER })
    );
    assert_eq!(
        (&a["scope"]["items"], &a["scope"]["never_fetched"]),
        (&json!(5), &json!(1))
    );
    let term = &a["terms"][0];
    // kaom and r1; r2 holds 20 and 75; r3's 40 sits beside an unread array
    assert_eq!(
        (
            &term["matched"]["count"],
            &term["failed"]["count"],
            &term["lacked"]["count"],
            &term["undecided"]["count"],
            &term["together"]["count"]
        ),
        (&json!(2), &json!(1), &json!(1), &json!(1), &json!(1))
    );
    let ids: Vec<&str> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["kaom", "r1"]);
    assert_eq!(a["total"]["undecided_reasons"][0]["example"]["id"], "r3");

    // the text says what the JSON says: every block, every nonzero count, every id
    let shown = text(&acq(&base, &["search", "--realm", "pc", LIFE, "--routes"]));
    for needle in [
        "query   line(\"# to maximum Life\" arg1>=90)",
        "scope   account Alice#1234 · pc · live · 5 items · 2 locations fetched",
        "1 location never fetched",
        "--view locations, not built",
        &format!(
            "basis   store {} · build {} · snapshot {} · facts v",
            a["basis"]["store"].as_str().unwrap(),
            a["basis"]["build"].as_str().unwrap(),
            a["basis"]["snapshot"]["response"]
        ),
        "2 matched · 1 failed · 1 lacked · 1 undecided · 1 reaches 90 only together",
        "total   2 matches · 1 undecided",
        "Kaom's Heart Glorious Plate · unique · Standard / Dump",
        "+500 to maximum Life (explicit)",
        "id kaom",
        "undecided implicit lines unread — e.g. Hex Band Two-Stone Ring · id r3",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }

    // rule 5, the route property at a terminal: every command the answer
    // prints runs, and returns as many as it counted
    let mut routed = 0;
    let lines: Vec<&str> = shown.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let Some(command) = line.trim().strip_prefix("acq ") else {
            continue;
        };
        let counted: u64 = lines[i - 1]
            .split(['(', ')'])
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
        let out = Command::new("sh")
            .arg("-c")
            .arg(format!("\"$ACQ\" --json {command}"))
            .env("ACQ", env!("CARGO_BIN_EXE_acq"))
            .env("ACQ_STORE_DIR", &base)
            .env("ACQ_NO_KEYRING", "1")
            .env("ACQ_NO_SPAWN", "1")
            .env("ACQ_PROVIDER", "mock")
            .env_remove("ACQ_ACCOUNT")
            .output()
            .unwrap();
        assert_eq!(sole_json(&out)["total"]["matched"], counted, "{command}");
        routed += 1;
    }
    assert_eq!(
        routed, 6,
        "matched, failed, lacked, undecided, together, and the root's undecided:\n{shown}"
    );
    // the lacked route starts with the language's not, which a terminal
    // reads as a flag: it is printed after `--`, and it ran above
    assert!(
        shown.contains(r##"--realm pc -- '-line("# to maximum Life")'"##),
        "{shown}"
    );
}

/// The first seat's words (2026-09-26; `tests/seat_faults.rs` holds the
/// JSON): a row says what it sorts by (F6), a partial id is said to find
/// nothing because an id is whole (F7), the terms header under every
/// realm says so (F8), and a closed set's refusal is near names and not
/// the list (F5).
#[test]
fn first_seat_the_text_says_what_sorted_what_an_id_is_and_of_every_realm() {
    let base = base();
    seed(&base);
    seed_more(&base);
    let shown = text(&acq(&base, &["search", "--realm", "all", "is:corrupted"]));
    assert!(
        shown.contains("over live items of every realm in all leagues"),
        "{shown}"
    );
    let shown = text(&acq(
        &base,
        &[
            "search", "--realm", "pc", "--sort", "ilvl", "--desc", "--limit", "1",
        ],
    ));
    assert!(shown.contains("sorts by ilvl 84"), "{shown}");
    let shown = text(&acq(
        &base,
        &[
            "search",
            "--realm",
            "pc",
            "rarity=rare",
            "--sort",
            "line(\"# to maximum Life\").arg1",
            "--desc",
            "--limit",
            "1",
        ],
    ));
    assert!(
        shown.contains("sorts by 95 · +95 to maximum Life (explicit)"),
        "{shown}"
    );
    // a line the sort and a term both show is printed once, with another
    // term's evidence between them (the seat's ask 56 replayed)
    let shown = text(&acq(
        &base,
        &[
            "search",
            "--realm",
            "pc",
            "class:ring \"# to maximum Life\">=90",
            "--sort",
            "line(\"# to maximum Life\").arg1",
            "--desc",
            "--limit",
            "1",
        ],
    ));
    assert!(shown.contains("class Rings"), "{shown}");
    assert_eq!(
        shown.matches("+95 to maximum Life (explicit)").count(),
        1,
        "{shown}"
    );
    // the review of 9b, 1: two identical lines are two, whatever an
    // earlier part of the row showed — a sum of 40 shows 10, 20 and 10
    let shown = text(&acq(
        &base,
        &[
            "search",
            "--realm",
            "poe2",
            "sum(\"# to maximum Life\")>=40",
            "--sort",
            "sum(\"# to maximum Life\")",
        ],
    ));
    assert!(shown.contains("sorts by 40"), "{shown}");
    assert_eq!(
        shown.matches("+10 to maximum Life (explicit)").count(),
        2,
        "{shown}"
    );
    assert_eq!(
        shown.matches("+20 to maximum Life (explicit)").count(),
        1,
        "{shown}"
    );
    // the validation of the review: a sort that shows one of the two
    // identical lines hides neither from a later total, and the terms'
    // order changes no count (rule 9)
    let counts = |query: &str, sort: &str| -> (usize, usize) {
        let shown = text(&acq(
            &base,
            &["search", "--realm", "poe2", query, "--sort", sort],
        ));
        assert!(shown.contains("id twin"), "{shown}");
        (
            shown.matches("+10 to maximum Life (explicit)").count(),
            shown.matches("+20 to maximum Life (explicit)").count(),
        )
    };
    let one_of_the_twins = "line(\"# to maximum Life\" arg1<=10).arg1";
    assert_eq!(
        counts("sum(\"# to maximum Life\")>=40", one_of_the_twins),
        (2, 1)
    );
    assert_eq!(
        counts(
            "sum(\"# to maximum Life\")>=40 \"# to maximum Life\"",
            one_of_the_twins
        ),
        (2, 1)
    );
    assert_eq!(
        counts(
            "\"# to maximum Life\" sum(\"# to maximum Life\")>=40",
            one_of_the_twins
        ),
        (2, 1)
    );
    // the review of 9b, a presentation gap: a row that sorts last names
    // what it was sorted by — r3's implicit array is unread, so its 40 is
    // no largest, and `plain` has no life line at all
    let shown = text(&acq(
        &base,
        &[
            "search",
            "--realm",
            "pc",
            "rarity=rare",
            "--sort",
            "line(\"# to maximum Life\").arg1",
        ],
    ));
    assert!(
        shown.contains("sorts last: incomplete at 40 (line(\"# to maximum Life\").arg1)"),
        "{shown}"
    );
    assert!(
        shown.contains("sorts last: no satisfying occurrence (line(\"# to maximum Life\").arg1)"),
        "{shown}"
    );
    let shown = text(&acq(&base, &["search", "--realm", "pc", "id:r"]));
    assert!(
        shown.contains("nothing in scope carries the id of term 0: id:r — an id is matched whole"),
        "{shown}"
    );
    let out = acq(&base, &["search", "--realm", "pc", "class:staff"]);
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("near it"), "{err}");
    assert!(err.contains("class=Staves"), "{err}");
    assert!(!err.contains("Abyss Jewels"), "{err}");
}

#[test]
fn c11_a_failure_is_structured_and_a_later_steps_flag_is_refused_by_name() {
    let base = base();
    seed(&base);
    for (args, kind) in [
        (vec!["search", "rare"], "bare_word"),
        (vec!["search", "class:staff"], "unknown_value"),
        (vec!["search", "--fields", "name"], "not_built"),
        (vec!["search", "--view", "locations"], "not_built"),
        (vec!["search", "--count", "rarty"], "unknown_name"),
        (vec!["search", "--sum", "stack"], "view"),
        (vec!["search", "--count", "tab", "--sort", "ilvl"], "view"),
        (
            vec!["search", "--count", "tab", "--cross", "tab,league"],
            "view",
        ),
        (vec!["search", "--realm", "ps4"], "realm_unknown"),
        (vec!["search", "--sort", "name"], "operator_mismatch"),
        (vec!["search", "--describe", "clas"], "unknown_name"),
        // built at step 10: a query the binder refuses is that error
        (vec!["show", "r1", "--against", "rare"], "bare_word"),
        (vec!["show", "r1", "--basis", "x"], "not_built"),
        (vec!["show", "t1"], "not_an_item"),
        (
            vec!["search", "rarity=rare", "--explain", "0", "--count", "tab"],
            "view",
        ),
        (
            vec!["search", "rarity=rare", "--explain", "0", "--sort", "ilvl"],
            "view",
        ),
        (
            vec!["search", "rarity=rare", "--explain", "1"],
            "no_such_path",
        ),
        // built at step 10: a continuation is whole, and a token is one
        (vec!["search", "--explain", "0", "--next", "x"], "view"),
        (vec!["search", "rarity=rare", "--next", "x"], "view"),
        (vec!["search", "--next", "x"], "token"),
        (vec!["search", "--request", "-", "--limit", "3"], "view"),
        (vec!["search", "--request", "-", "--realm", "pc"], "view"),
        (vec!["search", "--request", "-", "--describe"], "view"),
        (vec!["search", "--next", "x", "--describe"], "view"),
        (vec!["search", "--print-request", "--describe"], "view"),
        (vec!["search", "--context", "nowhere"], "view"),
        (vec!["search", "rare", "--context", "corpus"], "bare_word"),
    ] {
        let mut with_json = vec!["--json"];
        with_json.extend(&args);
        let out = acq(&base, &with_json);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        let failure = sole_json(&out);
        assert_eq!(failure["kind"], kind, "{args:?}: {failure}");
        assert!(failure["error"].is_string());
        let out = acq(&base, &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(
            out.stdout.is_empty() && String::from_utf8_lossy(&out.stderr).starts_with("Error: "),
            "{args:?}"
        );
    }
    let bare = sole_json(&acq(&base, &["--json", "search", "rare"]));
    assert_eq!(
        bare["readings"],
        json!(["rarity=rare", "\"rare\"", "line(template:rare)"])
    );
    let refused = sole_json(&acq(&base, &["--json", "search", "--fields", "name"]));
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .starts_with("not built: --fields (waits for a consumer that asks)")
    );
}

/// Step 10's first piece at a terminal, one `--json` smoke for each of
/// the three flags, and the text a function of it (C53): `--explain`,
/// `--context corpus`, `show --against`, and a zero answer's offers
/// printed as the commands they are.
#[test]
fn step_10_explain_context_and_against_at_a_terminal() {
    let base = base();
    seed(&base);

    // kaom and r1 reach 90; r2 holds 20 and 75; r3's 40 sits beside an
    // unread array; `plain` has no life line. The bound forced true asks
    // the line alone: kaom, r1, r2 and r3, whose 40 is then a witness
    let out = acq(
        &base,
        &[
            "--json",
            "search",
            "--realm",
            "pc",
            LIFE,
            "--explain",
            "0.1",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let a = sole_json(&out);
    let explain = &a["view"]["explain"];
    assert_eq!(
        (&explain["path"], &explain["node"]),
        (&json!("0.1"), &json!("arg1>=90"))
    );
    assert_eq!(explain["given"], json!({ "matched": 2, "undecided": 1 }));
    assert_eq!(
        explain["forced_true"]["query"]["text"],
        "line(\"# to maximum Life\" true())"
    );
    assert_eq!(
        (
            &explain["forced_true"]["matched"]["count"],
            &explain["forced_true"]["undecided"]["count"]
        ),
        (&json!(4), &json!(0))
    );
    assert_eq!(
        (
            &explain["forced_false"]["matched"]["count"],
            &explain["forced_false"]["undecided"]["count"]
        ),
        (&json!(0), &json!(0))
    );
    assert_eq!(a["rows"], json!([]));
    let shown = text(&acq(
        &base,
        &["search", "--realm", "pc", LIFE, "--explain", "0.1"],
    ));
    for needle in [
        "explain 0.1  arg1>=90",
        "as given      2 matches · 1 undecided",
        "forced true   4 matches",
        "line(\"# to maximum Life\" true())",
        "forced false  0 matches",
        "line(\"# to maximum Life\" false())",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }
    assert!(!shown.contains("rows    "), "{shown}");

    // the corpus context: the empty query, and the typed one set aside
    let out = acq(
        &base,
        &[
            "--json",
            "search",
            "--realm",
            "pc",
            LIFE,
            "--context",
            "corpus",
            "--count",
            "rarity",
        ],
    );
    assert!(out.status.success());
    let a = sole_json(&out);
    assert_eq!(a["query"]["text"], "");
    assert_eq!(
        a["set_aside"]["text"],
        "line(\"# to maximum Life\" arg1>=90)"
    );
    assert_eq!(a["total"]["matched"], 5);
    assert_eq!(a["view"]["counts"]["tables"][0]["key"], "rarity");
    let shown = text(&acq(
        &base,
        &["search", "--realm", "pc", LIFE, "--context", "corpus"],
    ));
    for needle in [
        "query   (every item in scope)",
        "aside   line(\"# to maximum Life\" arg1>=90) — set aside under --context corpus, counted nowhere",
        "total   5 matches",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }
    // `matches` is what every answer is
    let plain = sole_json(&acq(&base, &["--json", "search", "--realm", "pc", LIFE]));
    let matches = sole_json(&acq(
        &base,
        &[
            "--json",
            "search",
            "--realm",
            "pc",
            LIFE,
            "--context",
            "matches",
        ],
    ));
    assert_eq!(plain, matches);

    // the why-not: r2 holds 20 and 75, neither reaching 90
    let out = acq(&base, &["--json", "show", "r2", "--against", LIFE]);
    assert!(out.status.success());
    let shown = sole_json(&out);
    assert_eq!(shown["item"]["facts"]["id"], "r2");
    let against = &shown["against"];
    assert_eq!(
        against["query"]["text"],
        "line(\"# to maximum Life\" arg1>=90)"
    );
    assert_eq!(against["outcome"], "no_match");
    assert_eq!(against["nodes"][0]["outcome"], "failed");
    assert_eq!(against["nodes"][0]["shows"].as_array().unwrap().len(), 2);
    let shown = text(&acq(&base, &["show", "r2", "--against", LIFE]));
    for needle in [
        "against line(\"# to maximum Life\" arg1>=90)",
        "no match",
        "0    line(\"# to maximum Life\" arg1>=90)  failed",
        "+20 to maximum Life (implicit)",
        "+75 to maximum Life (explicit)",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }
    // an undecided item says why
    let shown = text(&acq(&base, &["show", "r3", "--against", LIFE]));
    assert!(shown.contains("undecided"), "{shown}");
    assert!(shown.contains("implicit lines unread"), "{shown}");

    // a zero answer prints its offers as commands: explain of each part,
    // fewest matched first, then the corpus
    let shown = text(&acq(
        &base,
        &["search", "--realm", "pc", "rarity=rare rarity=unique"],
    ));
    let commands = printed_commands(&shown);
    let offered: Vec<&String> = commands
        .iter()
        .filter(|c| c.contains("--explain") || c.contains("--context"))
        .collect();
    assert_eq!(
        offered,
        [
            "acq search --account 'Alice#1234' --realm pc 'rarity=rare rarity=unique' --explain 1",
            "acq search --account 'Alice#1234' --realm pc 'rarity=rare rarity=unique' --explain 0",
            "acq search --account 'Alice#1234' --realm pc 'rarity=rare rarity=unique' --limit 20 --context corpus",
        ],
        "{shown}"
    );
    assert!(!shown.contains("not built: --explain"), "{shown}");
    for command in offered {
        let out = run_printed(&base, command);
        assert!(out.status.success(), "{command}");
    }
}

/// The plan's gap 2: a name with an apostrophe, from stdin, and the route
/// that carries it printed so that a shell reads it back.
#[test]
fn a_query_with_an_apostrophe_is_read_from_stdin_and_routed_shell_quoted() {
    let base = base();
    seed(&base);
    let mut child = command(&base, &["--json", "search", "--query-file", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"name=\"Kaom's Heart\"\n")
        .unwrap();
    let a = sole_json(&child.wait_with_output().unwrap());
    assert_eq!(a["rows"][0]["id"], "kaom");
    let shown = text(&acq(
        &base,
        &["search", "--query-file", "/dev/stdin", "--routes"],
    ));
    assert!(shown.contains("total   5 matches"), "{shown}");
    let shown = {
        let mut child = command(&base, &["search", "--query-file", "-", "--routes"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(b"name=\"Kaom's Heart\"")
            .unwrap();
        text(&child.wait_with_output().unwrap())
    };
    assert!(
        shown.contains(r#"acq search --account 'Alice#1234' --realm pc 'name="Kaom'\''s Heart"'"#),
        "{shown}"
    );
}

/// Step 9 at a terminal: a price set through `acq price set` is what
/// `acq search 'has:priced'` finds and `acq show` prints, the basis names
/// the intent revision, and the text is a function of the JSON (C53).
#[test]
fn oq6_a_price_set_by_hand_is_found_shown_and_named_in_the_basis() {
    let base = base();
    seed(&base);
    let before = sole_json(&acq(
        &base,
        &["--json", "search", "--realm", "pc", "has:priced"],
    ));
    assert_eq!(before["total"]["matched"], 0);
    assert_eq!(before["basis"]["intent"], 0);
    let set = acq(
        &base,
        &["price", "set", "item/r1", "exact", "12.5", "chaos"],
    );
    assert!(
        set.status.success(),
        "{}",
        String::from_utf8_lossy(&set.stderr)
    );
    let a = sole_json(&acq(
        &base,
        &["--json", "search", "--realm", "pc", "has:priced"],
    ));
    assert_eq!(a["total"]["matched"], 1);
    assert_eq!(a["rows"][0]["id"], "r1");
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"][0]["value"]["value"],
        "12.5 chaos"
    );
    assert_eq!(a["basis"]["intent"], 1);
    let shown = text(&acq(&base, &["search", "--realm", "pc", "has:priced"]));
    for needle in [
        "price 12.5 chaos · price.side manual",
        "· intent 1 ·",
        "currency v1 · notes v2",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }
    let one = sole_json(&acq(&base, &["--json", "show", "r1"]));
    assert_eq!(one["price"]["status"], "is");
    assert_eq!(one["price"]["text"], "12.5 chaos");
    let shown = text(&acq(&base, &["show", "r1"]));
    assert!(shown.contains("price   12.5 chaos · manual\n"), "{shown}");
    let none = text(&acq(&base, &["show", "r2"]));
    assert!(none.contains("price   none (none)"), "{none}");
}

#[test]
fn describe_and_show_print_json_whole_and_text_from_it() {
    let base = base();
    seed(&base);
    let d = sole_json(&acq(&base, &["--json", "search", "--describe"]));
    let fields: Vec<&str> = d["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        fields,
        [
            "text",
            "name",
            "typeline",
            "base",
            "note",
            "rarity",
            "frame",
            "class",
            "category",
            "ilvl",
            "reqlevel",
            "stack",
            "sockets",
            "links",
            "priced",
            "price.amount",
            "price.currency",
            "price.lot",
            "league",
            "tab",
            "tab.type",
            "character",
            "container",
            "id",
            "sockets.<colour>",
            "is",
            "has"
        ]
    );
    let shown = text(&acq(&base, &["search", "--describe"]));
    for n in d["not_built"].as_array().unwrap() {
        assert!(shown.contains(n["construct"].as_str().unwrap()), "{shown}");
    }
    let one = sole_json(&acq(
        &base,
        &["--json", "search", "--describe", "league,line"],
    ));
    assert_eq!(
        (
            one["fields"].as_array().unwrap().len(),
            one["line"].as_array().unwrap().len()
        ),
        (1, 3)
    );

    let s = sole_json(&acq(&base, &["--json", "show", "r3", "--body"]));
    assert_eq!(s["item"]["unread"][0]["part"], "lines");
    assert!(s["body"].as_str().unwrap().contains("unreadable"));
    let shown = text(&acq(&base, &["show", "r2"]));
    for needle in [
        "Rune Coil Two-Stone Ring · rare",
        "id      r2",
        "place   pc · Standard / Rings",
        "implicit",
        "\"# to maximum Life\"   arg1 20",
        "unread  nothing",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }
    assert!(
        !shown.contains("sockets"),
        "a ring has no sockets line:\n{shown}"
    );
    // the sockets as the game shows them, and the largest link group (step 8)
    let shown = text(&acq(&base, &["show", "kaom"]));
    assert!(shown.contains("sockets R-R-G B · links 3"), "{shown}");
    let rows = text(&acq(&base, &["search", "--realm", "pc", "linked(red>=2)"]));
    assert!(rows.contains("R-R-G (link group)"), "{rows}");
}

/// Step 9c3, met at the first smoke of the build: a row sorted by a
/// reading of other totals says so by the reading's own name — it printed
/// `sorts by pseudo.total_fire_res 17`, the first total it had read, the
/// text taking the first named value a sort shows as the sort's own.
#[test]
fn step_9c3_a_row_sorted_by_a_reading_says_so_by_its_own_name() {
    let base = base();
    let mock = base.join("mock");
    std::fs::create_dir_all(&mock).unwrap();
    let mut index = Index::load(&mock).unwrap();
    index.record_login(USER, "u-search", false, 1).unwrap();
    let mut store = Store::open(&account_path(&mock, USER)).unwrap();
    let mut record = |ep: Endpoint, body: Value, at: i64| {
        store
            .record(
                &ep,
                &json!({ "realm": "pc", "league": "Standard" }),
                200,
                &body,
                at,
            )
            .unwrap();
    };
    record(
        Endpoint::Profile,
        json!({ "uuid": "u-search", "name": USER }),
        1,
    );
    record(
        Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        },
        json!({ "stashes": [{ "id": "t1", "name": "Read", "type": "PremiumStash" }] }),
        10,
    );
    let item = |id: &str, name: &str, base: &str, mods: Value| {
        json!({ "id": id, "name": name, "typeLine": base, "baseType": base, "rarity": "Unique",
                "frameTypeId": "Unique", "identified": true, "ilvl": 84, "x": 0, "y": 0,
                "explicitMods": mods })
    };
    record(
        Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "t1".into(),
            sub: None,
        },
        json!({ "stash": { "id": "t1", "name": "Read", "type": "PremiumStash", "items": [
            item("crest", "Geofri's Crest", "Great Crown", json!([
                "+17% to Fire Resistance", "+20% to Cold Resistance",
                "+17% to Lightning Resistance", "+22% to Chaos Resistance"])),
            item("long", "Long", "Iron Ring", json!([
                "+20% to Fire Resistance", "+20% to Cold Resistance",
                "+10000000000% to Chaos Resistance"])),
            item("plain", "Plain", "Iron Ring", json!(["+20 to maximum Mana"])),
        ] } }),
        20,
    );
    drop(store);
    let sorted = |by: &str| {
        text(&acq(
            &base,
            &["search", "--realm", "pc", "--sort", by, "--desc"],
        ))
    };
    let shown = sorted("pseudo.total_all_ele_res");
    assert!(
        shown.contains(
            "sorts by pseudo.total_all_ele_res 17 · pseudo.total_fire_res 17 · pseudo.total_cold_res 20 · pseudo.total_lightning_res 17"
        ),
        "{shown}"
    );
    let shown = sorted("pseudo.count_res");
    assert!(
        shown.contains(
            "sorts by pseudo.count_res 4 · pseudo.total_fire_res 17 · pseudo.total_cold_res 20 · pseudo.total_lightning_res 17 · pseudo.total_chaos_res 22"
        ),
        "{shown}"
    );
    // a count left open sorts last, with what was established and the
    // totals that are
    assert!(
        shown.contains(
            "sorts last: incomplete at 2 (pseudo.count_res) · pseudo.total_fire_res 20 · pseudo.total_cold_res 20"
        ),
        "{shown}"
    );
    assert!(
        shown.contains("sorts last: no satisfying occurrence (pseudo.count_res)"),
        "{shown}"
    );
}

/// Step 9c4 (C90): a mod displayed over several rows is found by a row of
/// it at a terminal, shown whole, and `show` prints each row as a name
/// with its own numbers — Thread of Hope's resistance, the second of
/// three rows, and its counts as the trade site shows them.
#[test]
fn step_9c4_a_mod_is_found_by_a_row_and_show_prints_its_rows() {
    let base = base();
    let mock = base.join("mock");
    std::fs::create_dir_all(&mock).unwrap();
    let mut index = Index::load(&mock).unwrap();
    index.record_login(USER, "u-search", false, 1).unwrap();
    let mut store = Store::open(&account_path(&mock, USER)).unwrap();
    let mut record = |ep: Endpoint, body: Value, at: i64| {
        store
            .record(
                &ep,
                &json!({ "realm": "pc", "league": "Standard" }),
                200,
                &body,
                at,
            )
            .unwrap();
    };
    record(
        Endpoint::Profile,
        json!({ "uuid": "u-search", "name": USER }),
        1,
    );
    record(
        Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        },
        json!({ "stashes": [{ "id": "t1", "name": "Rows", "type": "PremiumStash" }] }),
        10,
    );
    record(
        Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "t1".into(),
            sub: None,
        },
        json!({ "stash": { "id": "t1", "name": "Rows", "type": "PremiumStash", "items": [
            { "id": "thread", "name": "Thread of Hope", "typeLine": "Crimson Jewel",
              "baseType": "Crimson Jewel", "rarity": "Unique", "frameTypeId": "Unique",
              "identified": true, "ilvl": 84, "x": 0, "y": 0,
              "explicitMods": [
                "Only affects Passives in Small Ring",
                "Passive Skills in Radius can be Allocated without being connected to your tree\n-17% to all Elemental Resistances\nPassage" ] },
            { "id": "ring", "name": "Ring", "typeLine": "Iron Ring", "baseType": "Iron Ring",
              "rarity": "Rare", "frameTypeId": "Rare", "identified": true, "ilvl": 84,
              "x": 1, "y": 0, "explicitMods": ["+12% to all Elemental Resistances"] },
        ] } }),
        20,
    );
    drop(store);
    let found = sole_json(&acq(
        &base,
        &[
            "--json",
            "search",
            "--realm",
            "pc",
            r##""#% to all Elemental Resistances"<0 pseudo.count_res=3 pseudo.count_ele_res=3"##,
        ],
    ));
    assert_eq!(found["total"]["matched"], 1);
    assert_eq!(found["rows"][0]["id"], "thread");
    let said = text(&acq(
        &base,
        &[
            "search",
            "--realm",
            "pc",
            r##""#% to all Elemental Resistances"<0"##,
        ],
    ));
    // the row shows the mod whole, its rows on one line
    assert!(
        said.contains(
            "Passive Skills in Radius can be Allocated without being connected to your tree / -17% to all Elemental Resistances / Passage"
        ),
        "{said}"
    );
    let shown = text(&acq(&base, &["show", "thread"]));
    for line in [
        r##""Passive Skills in Radius can be Allocated without being connected to your tree\n#% to all Elemental Resistances\nPassage"   arg1 -17"##,
        r##"row "Passive Skills in Radius can be Allocated without being connected to your tree""##,
        r##"row "#% to all Elemental Resistances"   arg1 -17"##,
        r##"row "Passage""##,
    ] {
        assert!(shown.contains(line), "{line}\n{shown}");
    }
    // a mod of one row prints no row
    assert_eq!(shown.matches("row \"").count(), 3, "{shown}");
    let described = text(&acq(&base, &["search", "--describe", "template"]));
    assert!(
        described.contains("or by any one of its rows"),
        "{described}"
    );
}

/// The seed, and in tab `t3` what step 9d-B's tests ask by category: a
/// pair of boots, a gem with a damage line, a captured Dune Hellion
/// (yellow by the colour table), a captured Bleached Crustacean (unknown
/// there) and a breachstone with no frame.
fn seed_categories(base: &Path) {
    seed(base);
    let mut store = Store::open(&account_path(&base.join("mock"), USER)).unwrap();
    let plain = |id: &str, base: &str, more: Value| {
        let mut v = json!({ "id": id, "name": "", "typeLine": base, "baseType": base,
                            "identified": true, "ilvl": 0, "x": 0, "y": 0 });
        for (k, value) in more.as_object().unwrap() {
            v[k] = value.clone();
        }
        v
    };
    let bestiary = "Right-click to add this to your bestiary.";
    store
        .record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t3".into(),
                sub: None,
            },
            &json!({ "realm": "pc", "league": "Standard" }),
            200,
            &json!({ "stash": { "id": "t3", "name": "Never", "type": "PremiumStash", "items": [
                plain("boots", "Iron Greaves", json!({ "frameTypeId": "Rare", "rarity": "Rare",
                    "requirements": [{ "name": "Level", "values": [["20", 0]], "displayMode": 0 }],
                    "explicitMods": ["+30 to maximum Life"] })),
                plain("fire", "Fireball", json!({ "frameTypeId": "Gem",
                    "explicitMods": ["Deals 9 to 14 Fire Damage"] })),
                plain("hellion", "Dune Hellion", json!({ "frameTypeId": "Rare", "descrText": bestiary })),
                plain("crab", "Bleached Crustacean", json!({ "frameTypeId": "Rare", "descrText": bestiary })),
                plain("stone", "Tul's Breachstone", json!({})),
            ] } }),
            40,
        )
        .unwrap();
}

/// Step 9d-B at a terminal (C106, category): OQ5 spelled by category, a
/// beast found by its category and shown, the count by category with
/// every route it prints returning what it counted, and `--describe
/// category`. Tab `t3` gains a pair of boots, a gem with a damage line, a
/// captured Dune Hellion (yellow by the colour table), a captured
/// Bleached Crustacean (unknown there) and a breachstone with no frame.
#[test]
fn step_9d_category_at_a_terminal_oq5_a_beast_the_count_and_the_help() {
    let base = base();
    seed_categories(&base);

    // OQ5 by category: the seed's plate and three rings with a life line,
    // and the boots; the gem's damage line and the plain ring's mana stay out
    let oq5 = "(category=armour or category=accessory) (reqlevel=..30 or -has:reqlevel) \
               (line(template:resistance) or line(template:life) or line(template:damage))";
    let a = sole_json(&acq(&base, &["--json", "search", oq5]));
    let mut found: Vec<&str> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    found.sort_unstable();
    assert_eq!(found, ["boots", "kaom", "r1", "r2", "r3"]);
    assert_eq!(a["basis"]["categories"], 1);
    assert_eq!(a["basis"]["beast_colours"], 1);
    let shown = text(&acq(&base, &["search", oq5]));
    assert!(
        shown.contains("categories v1 · beast colours v1"),
        "{shown}"
    );
    assert!(shown.contains("category armour.boots"), "{shown}");

    // the beast, by its category and by its colour
    let a = sole_json(&acq(
        &base,
        &["--json", "search", "category=\"monster.yellowbeast\""],
    ));
    assert_eq!(a["total"]["matched"], 1);
    assert_eq!(a["rows"][0]["id"], "hellion");
    assert_eq!(a["terms"][0]["undecided"]["count"], 1);
    let a = sole_json(&acq(&base, &["--json", "search", "category:beast"]));
    assert_eq!(a["total"]["matched"], 2);
    let s = sole_json(&acq(&base, &["--json", "show", "crab"]));
    assert_eq!(s["class"], json!({ "is": "Stackable Currency" }));
    assert_eq!(s["category"]["listed"], json!(["monster.beast"]));
    assert_eq!(s["category"]["open"][0]["part"], "colour");
    let shown = text(&acq(&base, &["show", "crab"]));
    for needle in [
        "class   Stackable Currency",
        "category monster.beast",
        "open monster.yellowbeast, monster.redbeast:",
    ] {
        assert!(
            shown.contains(needle),
            "the text lacks `{needle}`:\n{shown}"
        );
    }
    let shown = text(&acq(&base, &["show", "hellion"]));
    assert!(shown.contains("category monster.yellowbeast"), "{shown}");
    let shown = text(&acq(&base, &["show", "fire"]));
    assert!(shown.contains("category gem.activegem"), "{shown}");

    // counted: every route printed returns what its heading counted, the
    // breachstone's among them — under `map`, its leaf open (J21)
    let counted = sole_json(&acq(&base, &["--json", "search", "--count", "category"]));
    let by: Vec<(String, u64)> = counted["view"]["counts"]["tables"][0]["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["value"].as_str().unwrap_or("").to_string(),
                b["count"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        by,
        [
            ("accessory.ring", 4),
            ("armour.boots", 1),
            ("armour.chest", 1),
            ("gem.activegem", 1),
            ("map", 1),
            ("monster.beast", 1),
            ("monster.yellowbeast", 1),
        ]
        .map(|(id, n)| (id.to_string(), n))
    );
    let shown = text(&acq(&base, &["search", "--count", "category", "--routes"]));
    assert!(shown.contains("count   category · 7 values"), "{shown}");
    let lines: Vec<&str> = shown.lines().collect();
    let mut ran = 0;
    for pair in lines.windows(2) {
        let Some(n) = pair[0]
            .strip_suffix("), over the scope:")
            .and_then(|head| head.rsplit_once('('))
            .and_then(|(_, n)| n.parse::<u64>().ok())
        else {
            continue;
        };
        let command = pair[1].trim();
        let out = run_printed(&base, command);
        assert!(out.status.success(), "`{command}` does not run");
        assert_eq!(
            sole_json(&out)["total"]["matched"],
            n,
            "`{command}` under `{}`",
            pair[0]
        );
        ran += 1;
    }
    assert!(ran >= 7, "{ran} routes ran:\n{shown}");
    let crossed = text(&acq(&base, &["search", "--cross", "category,tab"]));
    assert!(crossed.contains("cross   category × tab"), "{crossed}");

    // the help: the field, and every id with the site's text for it
    let d = sole_json(&acq(&base, &["--json", "search", "--describe", "category"]));
    assert_eq!(d["fields"][0]["name"], "category");
    assert_eq!(d["fields"][0]["values"].as_array().unwrap().len(), 82);
    assert_eq!(d["categories"].as_array().unwrap().len(), 82);
    assert_eq!(
        d["categories"][0],
        json!({ "id": "weapon", "text": "Any Weapon", "holds": d["categories"][0]["holds"] })
    );
    let shown = text(&acq(&base, &["search", "--describe", "category"]));
    for needle in [
        "monster.yellowbeast",
        "Yellow Beast",
        "Maven's Invitation",
        "categories v1",
    ] {
        assert!(
            shown.contains(needle),
            "the help lacks `{needle}`:\n{shown}"
        );
    }
    // a word that is no id is refused, structured (C11)
    let out = acq(&base, &["--json", "search", "category=beasts"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(sole_json(&out)["kind"], "unknown_value");
}

/// A block cut with no route to the rest (invariant 5 of the surface; the
/// brief of 9d-B: every pick of a category term is printed). The text cut
/// what a selector resolved to at five, and a category term has no count
/// that lists the rest: `category:one` on the owner's copy hid
/// `map.breachstone` behind "· 1 more". Here the pattern picks every id
/// and the store's items carry ten of them: each is printed, with its
/// count, and nothing is left behind a "more".
#[test]
fn every_pick_of_a_category_term_is_printed_at_a_terminal() {
    let base = base();
    seed_categories(&base);
    let a = sole_json(&acq(&base, &["--json", "search", "category~\".\""]));
    let picks: Vec<(String, u64)> = a["terms"][0]["resolved"]["values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["value"].as_str().unwrap().to_string(),
                v["items"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(picks.len(), 10, "{picks:?}");
    assert_eq!(a["terms"][0]["resolved"]["more"], 0);
    let shown = text(&acq(&base, &["search", "category~\".\""]));
    let resolved = shown
        .lines()
        .find(|line| line.trim_start().starts_with('→'))
        .unwrap_or_else(|| panic!("no resolved line:\n{shown}"));
    for (id, items) in &picks {
        assert!(
            resolved.contains(&format!("{id} ({items})")),
            "`{id}` is not printed: {resolved}"
        );
    }
    assert!(!resolved.contains("more"), "{resolved}");
    // another field's selector is cut as it was, with its route
    let bases = text(&acq(&base, &["search", "base:e"]));
    assert!(bases.contains("more"), "{bases}");
}

/// Step 9d-C at a terminal (C94, J7, J29): the plan's closing row — `Adds
/// # to # Fire Damage` on a weapon counted toward the attacks' total
/// alone — a flask's own line counted by no total, and `--describe` of a
/// ranged total. Tab `t3` gains `wand`, a Driftwood Wand, and `ruby`, a
/// Ruby Ring, each with `Adds 10 to 21 Fire Damage` (15.5 by hand);
/// `flask`, a Quicksilver Flask whose own line is `40% increased Movement
/// Speed`; and `greaves`, with that line as an explicit.
#[test]
fn step_9d_c_the_ranged_family_and_a_flask_at_a_terminal() {
    let base = base();
    seed(&base);
    let mut store = Store::open(&account_path(&base.join("mock"), USER)).unwrap();
    let item = |id: &str, base: &str, rarity: &str, more: Value| {
        let mut v = json!({ "id": id, "name": "", "typeLine": base, "baseType": base,
                            "rarity": rarity, "frameTypeId": rarity,
                            "identified": true, "ilvl": 0, "x": 0, "y": 0 });
        for (k, value) in more.as_object().unwrap() {
            v[k] = value.clone();
        }
        v
    };
    let fire = json!({ "explicitMods": ["Adds 10 to 21 Fire Damage"] });
    store
        .record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t3".into(),
                sub: None,
            },
            &json!({ "realm": "pc", "league": "Standard" }),
            200,
            &json!({ "stash": { "id": "t3", "name": "Never", "type": "PremiumStash", "items": [
                item("wand", "Driftwood Wand", "Rare", fire.clone()),
                item("ruby", "Ruby Ring", "Rare", fire),
                item("flask", "Quicksilver Flask", "Magic", json!({
                    "utilityMods": ["40% increased Movement Speed"] })),
                item("greaves", "Iron Greaves", "Rare", json!({
                    "explicitMods": ["40% increased Movement Speed"] })),
            ] } }),
            40,
        )
        .unwrap();
    drop(store);
    let found = |query: &str| -> (Vec<String>, Value) {
        let a = sole_json(&acq(&base, &["--json", "search", query]));
        let mut ids: Vec<String> = a["rows"]
            .as_array()
            .unwrap_or_else(|| panic!("`{query}`: {a}"))
            .iter()
            .map(|r| r["id"].as_str().unwrap().to_string())
            .collect();
        ids.sort_unstable();
        (ids, a)
    };

    // the closing row: the bare name is the `avg`
    let (ids, a) = found("pseudo.adds_fire_to_attacks=15.5");
    assert_eq!(ids, ["ruby", "wand"]);
    assert_eq!(a["basis"]["totals"], 6);
    assert_eq!(a["query"]["text"], "pseudo.adds_fire_to_attacks=15.5");
    assert_eq!(found("pseudo.adds_fire=15.5").0, ["ruby"]);
    assert_eq!(
        found("pseudo.adds_fire.low=10 pseudo.adds_fire.high=21").0,
        ["ruby"]
    );
    assert_eq!(found("has:pseudo.adds_fire_to_spells").0, ["ruby"]);
    let sorted = text(&acq(
        &base,
        &[
            "search",
            "--realm",
            "pc",
            "has:pseudo.adds_fire_to_attacks",
            "--sort",
            "pseudo.adds_fire",
        ],
    ));
    assert!(
        sorted.contains("sorts by 15.5 · Adds 10 to 21 Fire Damage (explicit)"),
        "{sorted}"
    );
    assert!(
        sorted.contains("sorts last: no satisfying occurrence (pseudo.adds_fire)"),
        "{sorted}"
    );
    let summed = sole_json(&acq(
        &base,
        &[
            "--json",
            "search",
            "has:pseudo.adds_fire_to_attacks",
            "--count",
            "base",
            "--sum",
            "pseudo.adds_fire",
        ],
    ));
    assert_eq!(
        summed["view"]["counts"]["sum"],
        json!({ "name": "pseudo.adds_fire", "value": 15.5, "lacking": 1 })
    );

    // a flask's own line is counted by no total (J29)
    assert_eq!(found("has:pseudo.increased_movement_speed").0, ["greaves"]);
    assert_eq!(
        found("id:flask -has:pseudo.increased_movement_speed").0,
        ["flask"]
    );
    // and found by its line still
    assert_eq!(
        found("\"#% increased Movement Speed\"=40").0,
        ["flask", "greaves"]
    );

    // `--describe` of one ranged total
    let d = sole_json(&acq(
        &base,
        &["--json", "search", "--describe", "adds_fire"],
    ));
    assert_eq!(d["computed"].as_array().unwrap().len(), 1);
    assert_eq!(d["computed"][0]["name"], "pseudo.adds_fire");
    assert_eq!(d["computed"][0]["kind"], "ranged total");
    let what = d["computed"][0]["what"].as_str().unwrap();
    for needle in [
        "pseudo_adds_fire_damage",
        "line(\"Adds # to # Fire Damage\")",
        "category=weapon",
        "`crucibleMods`",
        "`utilityMods`",
        "i06–i08",
    ] {
        assert!(
            what.contains(needle),
            "the definition lacks `{needle}`: {what}"
        );
    }
    assert!(
        d["totals"].as_str().unwrap().starts_with("totals v6, "),
        "{d}"
    );
    let shown = text(&acq(&base, &["search", "--describe", "adds_fire"]));
    assert!(
        shown.contains("pseudo.adds_fire") && shown.contains("`utilityMods`"),
        "{shown}"
    );
    // the refusal of a ranged total is gone from the list of what is not built
    let all = sole_json(&acq(&base, &["--json", "search", "--describe"]));
    assert!(
        all["not_built"]
            .as_array()
            .unwrap()
            .iter()
            .all(|n| n["construct"] != "pseudo.<name>.<slot>")
    );
}

/// Step 9d-A at a terminal (C101, V5): the base defence percentile of a
/// captured hybrid — the table's `e002[3]`, Ambush Mitts as the trade
/// site listed them, which the site showed at 90 — of an item that lacks
/// the field, and `--describe` of it. Tab `t3` gains `mitts`: quality 6,
/// Evasion Rating 212 and Energy Shield 45 under `125% increased Evasion
/// and Energy Shield`, on a base ranging evasion 81–91 and energy shield
/// 17–19. By hand: the display is the roll × 2.25 × 1.06 = roll × 2.385;
/// evasion 89 gives 212.265, shown 212, and 88 and 90 give 209.88 and
/// 214.65; energy shield 19 gives 45.315, shown 45, and 18 gives 42.93.
/// 100 × 8 / 10 = 80 and 100 × 2 / 2 = 100: their average 90. And
/// `ruby`, a Ruby Ring, which displays no defence.
#[test]
fn step_9d_a_the_percentile_of_a_captured_hybrid_at_a_terminal() {
    let base = base();
    seed(&base);
    let mut store = Store::open(&account_path(&base.join("mock"), USER)).unwrap();
    let property =
        |name: &str, value: &str| json!({ "name": name, "values": [[value, 0]], "displayMode": 0 });
    store
        .record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t3".into(),
                sub: None,
            },
            &json!({ "realm": "pc", "league": "Standard" }),
            200,
            &json!({ "stash": { "id": "t3", "name": "Never", "type": "PremiumStash", "items": [
                { "id": "mitts", "name": "", "typeLine": "Ambush Mitts", "baseType": "Ambush Mitts",
                  "rarity": "Rare", "frameTypeId": "Rare", "identified": true, "ilvl": 0,
                  "x": 0, "y": 0, "searing": true,
                  "properties": [property("Quality", "+6%"), property("Evasion Rating", "212"),
                                 property("Energy Shield", "45")],
                  "explicitMods": ["125% increased Evasion and Energy Shield"] },
                { "id": "ruby", "name": "", "typeLine": "Ruby Ring", "baseType": "Ruby Ring",
                  "rarity": "Rare", "frameTypeId": "Rare", "identified": true, "ilvl": 0,
                  "x": 1, "y": 0 },
            ] } }),
            40,
        )
        .unwrap();
    drop(store);
    let found = |query: &str| -> (Vec<String>, Value) {
        let a = sole_json(&acq(&base, &["--json", "search", query]));
        let mut ids: Vec<String> = a["rows"]
            .as_array()
            .unwrap_or_else(|| panic!("`{query}`: {a}"))
            .iter()
            .map(|r| r["id"].as_str().unwrap().to_string())
            .collect();
        ids.sort_unstable();
        (ids, a)
    };

    // the captured hybrid, as the site showed it
    let (ids, a) = found("pseudo.defence_pct=90");
    assert_eq!(ids, ["mitts"]);
    assert_eq!(a["basis"]["defences"], 1);
    assert_eq!(a["query"]["text"], "pseudo.defence_pct=90");
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"][0],
        json!({ "value": { "name": "pseudo.defence_pct", "value": 90 } })
    );
    let shown = text(&acq(
        &base,
        &["search", "--realm", "pc", "pseudo.defence_pct>=80"],
    ));
    assert!(shown.contains("pseudo.defence_pct 90"), "{shown}");
    assert!(shown.contains("defences v1"), "{shown}");

    // an item that lacks the field: lacked, never zero, and never a match.
    // A term's counts are over the scope: the seed's five items display no
    // defence and `ruby` is the sixth, `mitts` the one that has it
    let (ids, a) = found("id:ruby pseudo.defence_pct>=0");
    assert_eq!(ids, [] as [&str; 0]);
    assert_eq!(a["terms"][1]["matched"]["count"], 1);
    assert_eq!(a["terms"][1]["lacked"]["count"], 6);
    assert_eq!(a["terms"][1]["failed"]["count"], 0);
    assert_eq!(a["total"]["undecided"]["count"], 0);
    assert_eq!(found("id:ruby -has:pseudo.defence_pct").0, ["ruby"]);

    // `--describe` of it
    let d = sole_json(&acq(
        &base,
        &["--json", "search", "--describe", "defence_pct"],
    ));
    assert_eq!(d["computed"].as_array().unwrap().len(), 1);
    assert_eq!(d["computed"][0]["name"], "pseudo.defence_pct");
    let what = d["computed"][0]["what"].as_str().unwrap();
    for needle in [
        "The percentile of all base defence rolls, averaged",
        "exact half",
        "defences v1",
    ] {
        assert!(
            what.contains(needle),
            "the definition lacks `{needle}`: {what}"
        );
    }
    let shown = text(&acq(&base, &["search", "--describe", "defence_pct"]));
    assert!(
        shown.contains("pseudo.defence_pct") && shown.contains("exact half"),
        "{shown}"
    );
    let all = sole_json(&acq(&base, &["--json", "search", "--describe"]));
    assert!(
        all["not_built"]
            .as_array()
            .unwrap()
            .iter()
            .all(|n| n["construct"] != "pseudo.defence_pct")
    );
}

/// A query that starts with the language's not is taken after
/// `--against` as it is typed: a zero answer's query, printed after `--`,
/// is what a reader carries to `show --against`.
#[test]
fn show_against_takes_a_query_that_starts_with_a_not() {
    let base = base();
    seed(&base);
    let out = acq(
        &base,
        &["--json", "show", "r1", "--against", "-rarity=magic"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(sole_json(&out)["against"]["query"]["text"], "-rarity=magic");
}

/// Step 10's second piece at a terminal: an answer cut at its limit
/// prints the command that goes on, and the pages are the whole;
/// `--print-request` prints what `--request` reads, the one answering as
/// the search typed does; a count's route read back from an answer's
/// `--json` runs.
#[test]
fn step_10_next_request_and_print_request_at_a_terminal() {
    let base = base();
    seed(&base);
    let rows = |a: &Value| a["rows"].as_array().unwrap().clone();

    // the pages, each by the command the one before printed
    let whole = sole_json(&acq(
        &base,
        &["--json", "search", "--realm", "pc", "--limit", "100"],
    ));
    let total = whole["total"]["matched"].as_u64().unwrap();
    assert!(total >= 5, "{total}");
    let shown = text(&acq(&base, &["search", "--realm", "pc", "--limit", "2"]));
    let more = shown
        .lines()
        .find(|line| line.starts_with("more    "))
        .unwrap_or_else(|| panic!("no more line:\n{shown}"));
    assert!(
        more.starts_with(&format!(
            "more    rows 1–2 of {total} shown: acq search --account 'Alice#1234' --next acq1."
        )),
        "{more}"
    );
    let mut command = printed_commands(more).remove(0);
    // the command printed is the token behind the account, so the later
    // pages below, spelled so from each answer's token, are as printed
    assert_eq!(
        command,
        format!(
            "acq search --account '{USER}' --next {}",
            sole_json(&acq(
                &base,
                &["--json", "search", "--realm", "pc", "--limit", "2"]
            ))["view"]["rows"]["next"]["token"]
                .as_str()
                .unwrap()
        )
    );
    let first = sole_json(&acq(
        &base,
        &["--json", "search", "--realm", "pc", "--limit", "2"],
    ));
    let mut paged = rows(&first);
    loop {
        let out = run_printed(&base, &command);
        assert!(out.status.success(), "{command}");
        let page = sole_json(&out);
        paged.extend(rows(&page));
        match page["view"]["rows"]["next"]["token"].as_str() {
            Some(token) => command = format!("acq search --account '{USER}' --next {token}"),
            None => break,
        }
    }
    assert_eq!(paged, rows(&whole));
    // `--limit` alone stands beside a token, and the place is kept
    let token = first["view"]["rows"]["next"]["token"].as_str().unwrap();
    let wide = sole_json(&acq(
        &base,
        &["--json", "search", "--next", token, "--limit", "100"],
    ));
    assert_eq!(rows(&wide), rows(&whole)[2..].to_vec());
    assert!(wide["view"]["rows"].get("next").is_none());

    // `--print-request` prints what `--request` reads: the answer is the
    // one the same words typed give
    let dir = base.join("requests");
    std::fs::create_dir_all(&dir).unwrap();
    for (n, args) in [
        vec!["search", "--realm", "pc", LIFE],
        vec![
            "search",
            "rarity=rare",
            "--sort",
            "ilvl",
            "--desc",
            "--limit",
            "3",
        ],
        vec!["search", "--realm", "pc", "--count", "rarity,tab"],
        vec![
            "search",
            "rarity=rare",
            "--cross",
            "rarity,tab",
            "--sum",
            "ilvl",
        ],
        vec!["search", LIFE, "--explain", "0.1"],
        vec!["search", LIFE, "--context", "corpus", "--count", "rarity"],
        vec!["search", "--", "-rarity=rare"],
    ]
    .into_iter()
    .enumerate()
    {
        let mut printing = vec!["search", "--print-request"];
        printing.extend(&args[1..]);
        let out = acq(&base, &printing);
        assert!(out.status.success(), "{args:?}");
        let request = sole_json(&out);
        // resolved, though the words typed named neither
        assert_eq!(request["scope"]["realm"], "pc", "{args:?}");
        assert_eq!(request["scope"]["account"], USER, "{args:?}");
        assert!(request.get("basis").is_none());
        let file = dir.join(format!("{n}.json"));
        std::fs::write(&file, &out.stdout).unwrap();
        let mut direct = vec!["--json"];
        direct.extend(&args);
        assert_eq!(
            sole_json(&acq(
                &base,
                &["--json", "search", "--request", file.to_str().unwrap()]
            )),
            sole_json(&acq(&base, &direct)),
            "{args:?}"
        );
    }

    // a count's route, as an answer's `--json` holds it, is a request
    // file: on the basis it was counted at it says nothing more
    let a = sole_json(&acq(&base, &["--json", "search", "rarity=rare"]));
    let route = &a["terms"][0]["matched"];
    let file = dir.join("route.json");
    std::fs::write(&file, route.to_string()).unwrap();
    let routed = sole_json(&acq(
        &base,
        &["--json", "search", "--request", file.to_str().unwrap()],
    ));
    assert_eq!(routed["total"]["matched"], route["count"]);
    assert!(routed.get("before").is_none());
    // and a continuation's request is one too
    let file = dir.join("next.json");
    std::fs::write(&file, first["view"]["rows"]["next"]["request"].to_string()).unwrap();
    let next = sole_json(&acq(
        &base,
        &["--json", "search", "--request", file.to_str().unwrap()],
    ));
    assert_eq!(rows(&next), rows(&whole)[2..4].to_vec());
}

/// Step 10's third piece at a terminal: `--membership all` adds the
/// removed, each row marked, the scope block counting the two apart; an
/// id that left says so and prints the command that finds it, which
/// runs; `acq show` shows a removed item, marked.
#[test]
fn step_10_membership_all_at_a_terminal() {
    let base = base();
    seed(&base);
    // a fetch of Rings that no longer has r2
    let mock = base.join("mock");
    let mut store = Store::open(&account_path(&mock, USER)).unwrap();
    store
        .record(
            &Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "t1".into(),
                sub: None,
            },
            &json!({ "realm": "pc", "league": "Standard" }),
            200,
            &json!({ "stash": { "id": "t1", "name": "Rings", "type": "PremiumStash", "items": [
                { "id": "r1", "name": "Doom Loop", "typeLine": "Two-Stone Ring", "baseType": "Two-Stone Ring",
                  "rarity": "Rare", "frameTypeId": "Rare", "identified": true, "ilvl": 84, "x": 0, "y": 0,
                  "explicitMods": ["+95 to maximum Life"] } ] } }),
            40,
        )
        .unwrap();
    drop(store);
    let ids = |a: &Value| -> Vec<String> {
        a["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap().to_string())
            .collect()
    };

    let live = sole_json(&acq(&base, &["--json", "search", "--realm", "pc"]));
    assert!(!ids(&live).contains(&"r2".to_string()));
    assert_eq!(live["scope"]["membership"], "live");
    let all = sole_json(&acq(
        &base,
        &["--json", "search", "--realm", "pc", "--membership", "all"],
    ));
    assert_eq!(all["scope"]["membership"], "all");
    assert_eq!(all["scope"]["items"], live["scope"]["items"]);
    assert_eq!(all["scope"]["removed"], 1);
    assert_eq!(ids(&all).len(), ids(&live).len() + 1);
    let r2 = all["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == "r2")
        .unwrap();
    assert_eq!(r2["removed_at"], 40);
    assert_eq!(r2["place"]["name"], "Rings");
    let shown = text(&acq(
        &base,
        &[
            "search",
            "--realm",
            "pc",
            "rarity=rare",
            "--membership",
            "all",
            "--routes",
        ],
    ));
    assert!(
        shown.contains(" · all · 4 items live · 1 removed · "),
        "{shown}"
    );
    // the terms are counted over both, and the block says so
    assert!(
        shown.contains("each term evaluated independently over live and removed pc items"),
        "{shown}"
    );
    assert!(
        shown
            .lines()
            .any(|l| l.contains("Rune Coil") && l.contains(" · removed ")),
        "{shown}"
    );
    // every command printed under all runs, and stays under all
    let commands = printed_commands(&shown);
    assert!(!commands.is_empty());
    for command in &commands {
        if command.starts_with("acq search") {
            assert!(command.contains(" --membership all"), "{command}");
        }
        assert!(run_printed(&base, command).status.success(), "{command}");
    }

    // an id that left: said on the term, with the command that finds it
    let gone = text(&acq(&base, &["search", "--realm", "pc", "id:r2"]));
    let said = gone
        .lines()
        .find(|l| l.contains("this id is an item that left the scope"))
        .unwrap_or_else(|| panic!("{gone}"));
    let command = printed_commands(said).remove(0);
    assert_eq!(
        command,
        format!("acq search --account '{USER}' --realm pc id:r2 --membership all")
    );
    assert_eq!(ids(&sole_json(&run_printed(&base, &command))), ["r2"]);
    assert!(!gone.contains("a part of one finds nothing"), "{gone}");
    assert!(!gone.contains("never fuzzy-matched"), "{gone}");
    assert!(
        gone.contains("id:r2 — it left the scope, as its term says"),
        "{gone}"
    );

    // `acq show` shows it, marked
    let out = acq(&base, &["--json", "show", "r2"]);
    assert!(out.status.success());
    assert_eq!(sole_json(&out)["item"]["facts"]["removed_at"], 40);
    let shown = text(&acq(&base, &["show", "r2"]));
    assert!(shown.lines().any(|l| l.starts_with("removed ")), "{shown}");
    assert!(!text(&acq(&base, &["show", "r1"])).contains("\nremoved "));

    // a word that is no membership, and a membership beside a token
    for (args, kind) in [
        (
            vec!["--json", "search", "--membership", "gone"],
            "membership_unknown",
        ),
        (
            vec!["--json", "search", "--next", "x", "--membership", "all"],
            "view",
        ),
    ] {
        assert_eq!(sole_json(&acq(&base, &args))["kind"], kind, "{args:?}");
    }
}

/// A command printed for a count view is the view it was printed for
/// (rule 5, and more than that it runs): a request may name a field
/// after a narrowed vocabulary, which one `--count` list cannot say —
/// after `line:` the rest of a list are texts — so the printer opens a
/// second list. Outside audit, 2026-10-10: `["line:life", "rarity"]`
/// was printed as `--count 'line:life,rarity'`, which reads as the
/// vocabulary narrowed twice, and both commands ran.
#[test]
fn a_printed_count_command_names_the_keys_it_was_printed_for() {
    let base = base();
    seed(&base);
    let dir = base.join("requests");
    std::fs::create_dir_all(&dir).unwrap();
    for (n, keys) in [
        json!(["line:life", "rarity"]),
        json!(["line:life", "line", "tab", "line~^adds", "league", "rarity"]),
        json!(["tab", "line:life", "league"]),
        json!(["line:life, or more", "rarity", "line:~odd"]),
        // the audit's second round: a prefix in another case, and a text
        // that reads as a key of its own
        json!(["LINE:life", "rarity"]),
        json!(["line:life", "line:line:foo"]),
        json!(["Line~^adds", "TAB", "line:LINE~x", "line"]),
    ]
    .into_iter()
    .enumerate()
    {
        // nothing is unique and magic at once: a zero answer, whose corpus
        // offer keeps the view
        let file = dir.join(format!("keys-{n}.json"));
        std::fs::write(
            &file,
            json!({ "scope": { "realm": "pc" },
                    "query": { "text": "rarity=unique rarity=magic" },
                    "view": { "counts": { "keys": keys } } })
            .to_string(),
        )
        .unwrap();
        let args = ["search", "--request", file.to_str().unwrap()];
        let offered = sole_json(&acq(&base, &[&["--json"], &args[..]].concat()))["zero"]["offers"]
            ["context"]["request"]
            .clone();
        let shown = text(&acq(&base, &args));
        let command = printed_commands(&shown)
            .into_iter()
            .find(|c| c.contains("--context corpus"))
            .unwrap_or_else(|| panic!("{shown}"));
        let ran = run_printed(&base, &command);
        assert!(ran.status.success(), "{command}");
        let by_command = sole_json(&ran);
        // the same view, table for table, as the offer's own request gives
        let offer = dir.join(format!("offer-{n}.json"));
        std::fs::write(&offer, offered.to_string()).unwrap();
        let by_request = sole_json(&acq(
            &base,
            &["--json", "search", "--request", offer.to_str().unwrap()],
        ));
        let tables: Vec<Value> = by_command["view"]["counts"]["tables"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["key"].clone())
            .collect();
        // each table's key as the binder names it: a name in its own
        // case, the text after it as written
        let named: Vec<Value> = by_request["view"]["counts"]["tables"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["key"].clone())
            .collect();
        assert_eq!(named.len(), keys.as_array().unwrap().len(), "{keys}");
        assert_eq!(tables, named, "{keys}: {command}");
        assert!(
            by_command["view"] == by_request["view"],
            "{keys}: the views differ: {command}"
        );
    }
}
