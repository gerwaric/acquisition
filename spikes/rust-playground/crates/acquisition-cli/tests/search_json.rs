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
        "1 never fetched",
        "--view locations, not built (step 10)",
        &format!(
            "basis   snapshot {} · facts v",
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
        (vec!["search", "class:ring"], "not_built"),
        (vec!["search", "--count", "tab"], "not_built"),
        (vec!["search", "--view", "locations"], "not_built"),
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
    let refused = sole_json(&acq(&base, &["--json", "search", "--count", "tab"]));
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .starts_with("not built: --count (step 5)")
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
            "ilvl",
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
