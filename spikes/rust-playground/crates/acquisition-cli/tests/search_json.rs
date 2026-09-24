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
/// holds an apostrophe; `r1` with 95 life; `r2` with 20 and 75; `r3`
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
            item("kaom", "Kaom's Heart", "Glorious Plate", "Unique", json!({ "explicitMods": ["+500 to maximum Life"] })),
            item("r3", "Hex Band", "Two-Stone Ring", "Rare", json!({ "implicitMods": "unreadable", "explicitMods": ["+40 to maximum Life"] })),
            item("plain", "Dull Turn", "Iron Ring", "Rare", json!({ "explicitMods": ["+20 to maximum Mana"] })),
        ] } }),
        30,
    );
}

/// A second realm in the same store: one ring in a poe2 tab named as the
/// pc one is, and a pc item with more lines than a row shows.
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
            "items": [ring("two", vec!["+30 to Spirit".to_string()])] } }),
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
    assert_eq!(a["total"]["undecided_items"][0]["id"], "r3");

    // the text says what the JSON says: every block, every nonzero count, every id
    let shown = text(&acq(&base, &["search", "--realm", "pc", LIFE, "--routes"]));
    for needle in [
        "query   line(\"# to maximum Life\" arg1>=90)",
        "scope   account Alice#1234 · pc · live · 5 items · 2 locations fetched",
        "1 location never fetched",
        "--view locations, not built (step 10)",
        &format!(
            "basis   store {} · snapshot {} · facts v",
            a["basis"]["store"].as_str().unwrap(),
            a["basis"]["snapshot"]["response"]
        ),
        "2 matched · 1 failed · 1 lacked · 1 undecided · 1 reaches 90 only together",
        "total   2 matches · 1 undecided",
        "Kaom's Heart Glorious Plate · unique · Standard / Dump",
        "+500 to maximum Life (explicit)",
        "id kaom",
        "undecided Hex Band Two-Stone Ring · id r3",
        "implicit lines unread",
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

#[test]
fn c11_a_failure_is_structured_and_a_later_steps_flag_is_refused_by_name() {
    let base = base();
    seed(&base);
    for (args, kind) in [
        (vec!["search", "rare"], "bare_word"),
        (vec!["search", "sockets>=1"], "not_built"),
        (vec!["search", "class:staff"], "unknown_value"),
        (vec!["search", "--fields", "name"], "not_built"),
        (vec!["search", "--view", "locations"], "not_built"),
        (vec!["search", "--count", "rarty"], "unknown_name"),
        (vec!["search", "--count", "price.currency"], "not_built"),
        (vec!["search", "--sum", "stack"], "view"),
        (vec!["search", "--count", "tab", "--sort", "ilvl"], "view"),
        (
            vec!["search", "--count", "tab", "--cross", "tab,league"],
            "view",
        ),
        (vec!["search", "--realm", "ps4"], "realm_unknown"),
        (vec!["search", "--sort", "name"], "operator_mismatch"),
        (vec!["search", "--describe", "clas"], "unknown_name"),
        (vec!["show", "r1", "--against", "rarity=rare"], "not_built"),
        (vec!["show", "t1"], "not_an_item"),
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
            .starts_with("not built: --fields (step 10)")
    );
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
            "ilvl",
            "reqlevel",
            "stack",
            "league",
            "tab",
            "character",
            "container",
            "id",
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
}
