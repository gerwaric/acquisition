//! The item search over MCP (`search/BUILD-PLAN.md`, step 11b; the
//! rulings H1–H5 of `search/LEDGER.md`, "11 · the opening"): `search`,
//! `describe` and `show` are adapters of `acquisition-search` as `acq
//! search` and `acq show` are, so what a tool returns is the document the
//! CLI's `--json` prints — the answer whole on success, `{"error",
//! "kind", "readings", "offers"}` in a result marked `isError` on a
//! refusal (the owner, 2026-10-10) — and every request an answer or a
//! refusal holds is sent back as it was printed. The acceptance set's
//! agent questions, AQ1–AQ5, are asked through the tool over a fixture
//! whose every count is worked by hand below. Facts are seeded through
//! the store crate; no daemon runs.

mod harness;

use std::path::{Path, PathBuf};

use acquisition_store::{Endpoint, Index, Store, account_path};
use harness::{Mcp, acq, no_daemon_appeared};
use serde_json::{Value, json};

const USER: &str = "Alice#1234";

/// A scratch world, a server over it, and the `acq` beside the server.
struct Seat {
    base: PathBuf,
    mcp: Mcp,
    files: usize,
}

impl Drop for Seat {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.base);
    }
}

fn scratch() -> PathBuf {
    static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let base = std::env::temp_dir().join(format!(
        "acq-mcp-search-{}-{}",
        std::process::id(),
        N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    // a process id comes round again: never open what an earlier run left
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}

/// Where the server and the `acq` beside it look for the mock's stores.
fn stores(base: &Path) -> PathBuf {
    base.join("store").join("mock")
}

fn item(id: &str, name: &str, base: &str, rarity: &str, more: Value) -> Value {
    let mut v = json!({ "id": id, "name": name, "typeLine": base, "baseType": base, "rarity": rarity,
                        "frameTypeId": rarity, "identified": true, "ilvl": 84, "x": 0, "y": 0 });
    for (k, value) in more.as_object().unwrap() {
        v[k] = value.clone();
    }
    v
}

fn record(store: &mut Store, ep: Endpoint, realm: &str, body: Value, at: i64) {
    store
        .record(
            &ep,
            &json!({ "realm": realm, "league": "Standard" }),
            200,
            &body,
            at,
        )
        .unwrap();
}

fn stash(realm: &str, id: &str) -> Endpoint {
    Endpoint::Stash {
        realm: realm.into(),
        league: "Standard".into(),
        id: id.into(),
        sub: None,
    }
}

/// Five pc items in two tabs of Standard, every count below worked from
/// this list:
///
/// | id | tab | rarity | base | lines |
/// | --- | --- | --- | --- | --- |
/// | `r1` | Rings | rare | Two-Stone Ring | life 95, fire res 65 |
/// | `r2` | Rings | rare | Two-Stone Ring | life 20 (implicit) and 75, fire res 40 |
/// | `kaom` | Dump | unique | Glorious Plate | life 500 |
/// | `plain` | Dump | rare | Iron Ring | mana 20 |
/// | `str` | Dump | rare | Iron Ring | strength 25 |
fn seed(base: &Path) {
    let dir = stores(base);
    std::fs::create_dir_all(&dir).unwrap();
    let mut index = Index::load(&dir).unwrap();
    index.record_login(USER, "u-search", false, 1).unwrap();
    let mut store = Store::open(&account_path(&dir, USER)).unwrap();
    record(
        &mut store,
        Endpoint::Profile,
        "pc",
        json!({ "uuid": "u-search", "name": USER }),
        1,
    );
    record(
        &mut store,
        Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        },
        "pc",
        json!({ "stashes": [
            { "id": "t1", "name": "Rings", "type": "PremiumStash" },
            { "id": "t2", "name": "Dump", "type": "PremiumStash" } ] }),
        10,
    );
    record(
        &mut store,
        stash("pc", "t1"),
        "pc",
        json!({ "stash": { "id": "t1", "name": "Rings", "type": "PremiumStash", "items": [
            item("r1", "Doom Loop", "Two-Stone Ring", "Rare", json!({ "explicitMods":
                ["+95 to maximum Life", "+65% to Fire Resistance"] })),
            item("r2", "Rune Coil", "Two-Stone Ring", "Rare", json!({ "implicitMods": ["+20 to maximum Life"],
                "explicitMods": ["+75 to maximum Life", "+40% to Fire Resistance"] })),
        ] } }),
        20,
    );
    record(
        &mut store,
        stash("pc", "t2"),
        "pc",
        json!({ "stash": { "id": "t2", "name": "Dump", "type": "PremiumStash", "items": [
            item("kaom", "Kaom's Heart", "Glorious Plate", "Unique", json!({ "explicitMods": ["+500 to maximum Life"] })),
            item("plain", "Dull Turn", "Iron Ring", "Rare", json!({ "explicitMods": ["+20 to maximum Mana"] })),
            item("str", "Grim Knot", "Iron Ring", "Rare", json!({ "explicitMods": ["+25 to Strength"] })),
        ] } }),
        30,
    );
}

/// A second realm in the same store: one poe2 ring.
fn seed_poe2(base: &Path) {
    let mut store = Store::open(&account_path(&stores(base), USER)).unwrap();
    record(
        &mut store,
        Endpoint::Stashes {
            realm: "poe2".into(),
            league: "Standard".into(),
        },
        "poe2",
        json!({ "stashes": [{ "id": "p1", "name": "Rings", "type": "PremiumStash" }] }),
        40,
    );
    record(
        &mut store,
        stash("poe2", "p1"),
        "poe2",
        json!({ "stash": { "id": "p1", "name": "Rings", "type": "PremiumStash", "items": [
            item("two", "Bleak Eye", "Iron Ring", "Rare", json!({ "explicitMods": ["+30 to Spirit"] })),
        ] } }),
        50,
    );
}

/// A second account, with one item of its own.
fn seed_bob(base: &Path) {
    let dir = stores(base);
    let mut index = Index::load(&dir).unwrap();
    index.record_login("Bob#9", "u-bob", false, 2).unwrap();
    let mut store = Store::open(&account_path(&dir, "Bob#9")).unwrap();
    record(
        &mut store,
        Endpoint::Profile,
        "pc",
        json!({ "uuid": "u-bob", "name": "Bob#9" }),
        1,
    );
    record(
        &mut store,
        Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        },
        "pc",
        json!({ "stashes": [{ "id": "b1", "name": "Bobs", "type": "PremiumStash" }] }),
        10,
    );
    record(
        &mut store,
        stash("pc", "b1"),
        "pc",
        json!({ "stash": { "id": "b1", "name": "Bobs", "type": "PremiumStash", "items": [
            item("bob1", "Lone Band", "Iron Ring", "Rare", json!({})),
        ] } }),
        20,
    );
}

/// What the CLI's `--json` printed: the document, and whether it exited 0.
fn cli(base: &Path, args: &[&str]) -> (Value, bool) {
    let out = acq(base, &[&["--json"], args].concat());
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    let doc = serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!(
            "acq {args:?} printed no one JSON document ({e}):\n{stdout}\n{}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (doc, out.status.success())
}

impl Seat {
    /// The seeded fixture and a server over it.
    fn new() -> Seat {
        let base = scratch();
        seed(&base);
        Seat::over(base)
    }

    fn over(base: PathBuf) -> Seat {
        let mcp = Mcp::start(&base, &[]);
        Seat {
            base,
            mcp,
            files: 0,
        }
    }

    /// One tool call: the document, and whether the result was marked an
    /// error. H4: the JSON whole and no text — the result's one content
    /// block is that same document, for a host that reads no structured
    /// content, and nothing else stands beside it.
    fn call(&mut self, tool: &str, args: &Value) -> (Value, bool) {
        let result = self.mcp.result(tool, args.clone());
        let doc = result["structuredContent"].clone();
        assert!(doc.is_object(), "{tool} {args}: {result}");
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1, "{tool} {args}: {result}");
        assert_eq!(
            serde_json::from_str::<Value>(content[0]["text"].as_str().unwrap()).unwrap(),
            doc,
            "{tool} {args}"
        );
        (doc, result["isError"] == json!(true))
    }

    /// A request through `search`, held equal to the same request through
    /// `acq search --request`: one document, and a refusal on both or on
    /// neither.
    fn search(&mut self, request: &Value) -> (Value, bool) {
        self.files += 1;
        let file = self.base.join(format!("request-{}.json", self.files));
        std::fs::write(&file, request.to_string()).unwrap();
        let (printed, ok) = cli(&self.base, &["search", "--request", file.to_str().unwrap()]);
        let (doc, refused) = self.call("search", request);
        assert_eq!(doc, printed, "search {request}");
        assert_eq!(refused, !ok, "search {request}: {doc}");
        (doc, refused)
    }

    fn answer(&mut self, request: &Value) -> Value {
        let (doc, refused) = self.search(request);
        assert!(!refused, "search {request} was refused: {doc}");
        doc
    }

    fn refusal(&mut self, request: &Value) -> Value {
        let (doc, refused) = self.search(request);
        assert!(refused, "search {request} answered: {doc}");
        doc
    }
}

fn ids(answer: &Value) -> Vec<&str> {
    answer["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap())
        .collect()
}

/// A table's buckets as (value, count).
fn buckets(table: &Value) -> Vec<(String, u64)> {
    table["buckets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["value"].as_str().unwrap_or("?").to_string(),
                b["count"].as_u64().unwrap(),
            )
        })
        .collect()
}

const OQ1: &str = "class:ring rarity=rare (line(template:resistance) or line(template:strength))";

/// AQ1: three tables and no rows, the empty request's scope — and `{}`
/// is a request: every item, the realm resolved to the one held (H3).
#[test]
fn aq1_counts_by_tab_league_and_rarity_and_no_rows() {
    let mut seat = Seat::new();
    let every = seat.answer(&json!({}));
    assert_eq!(every["total"]["matched"], 5);
    assert_eq!(every["scope"]["realm"], "pc");
    assert_eq!(every["scope"]["account"]["name"], USER);

    let a = seat.answer(&json!({ "view": { "counts": { "keys": ["tab", "league", "rarity"] } } }));
    assert!(a["rows"].as_array().unwrap().is_empty(), "{}", a["rows"]);
    let tables = a["view"]["counts"]["tables"].as_array().unwrap();
    let keys: Vec<&str> = tables.iter().map(|t| t["key"].as_str().unwrap()).collect();
    assert_eq!(keys, ["tab", "league", "rarity"]);
    assert_eq!(
        buckets(&tables[0]),
        [("Dump".to_string(), 3), ("Rings".to_string(), 2)]
    );
    assert_eq!(buckets(&tables[1]), [("Standard".to_string(), 5)]);
    assert_eq!(
        buckets(&tables[2]),
        [("rare".to_string(), 4), ("unique".to_string(), 1)]
    );
    // the flags a terminal types give the same tables
    let (typed, _) = cli(&seat.base, &["search", "--count", "tab,league,rarity"]);
    assert_eq!(
        typed["view"]["counts"]["tables"],
        a["view"]["counts"]["tables"]
    );
    // a bucket is a count with its route, sent back as printed
    let dump = tables[0]["buckets"][0].clone();
    let members = seat.answer(&dump);
    assert_eq!(ids(&members), ["kaom", "plain", "str"]);
}

/// AQ2: the old query parenthesised and one new term. `r2` matches OQ1
/// with 40 of resistance and `str` with none, and both must leave.
#[test]
fn aq2_a_refinement_is_the_old_query_parenthesised_and_a_new_term() {
    let mut seat = Seat::new();
    let old = seat.answer(&json!({ "query": { "text": OQ1 } }));
    assert_eq!(ids(&old), ["r1", "r2", "str"]);
    let refined = format!(
        "({}) pseudo.total_res>=60",
        old["query"]["text"].as_str().unwrap()
    );
    let new = seat.answer(&json!({ "query": { "text": refined } }));
    assert_eq!(ids(&new), ["r1"]);
    // the tree an answer returned is accepted in place of its text (C104)
    let by_tree = seat.answer(&json!({ "query": { "tree": new["query"]["tree"] } }));
    assert_eq!(by_tree["query"], new["query"]);
    assert_eq!(by_tree["rows"], new["rows"]);
    let (typed, _) = cli(&seat.base, &["search", &refined]);
    assert_eq!(typed["rows"], new["rows"]);
}

/// AQ3: a row names what matched, a zero answer names the scope searched
/// and offers requests that run, and `show` with `against` says why not.
#[test]
fn aq3_a_row_names_what_matched_a_zero_answer_its_scope_and_show_says_why_not() {
    let mut seat = Seat::new();
    let a = seat.answer(&json!({ "query": { "text": "\"+# to maximum Life\">=90" } }));
    assert_eq!(ids(&a), ["r1", "kaom"]);
    assert_eq!(
        a["rows"][0]["matched"][0]["shows"][0]["line"]["text"],
        "+95 to maximum Life"
    );

    let zero = seat.answer(&json!({ "query": { "text": "rarity=rare name:kaom" } }));
    assert_eq!(zero["total"]["matched"], 0);
    assert_eq!(zero["scope"]["realm"], "pc");
    assert_eq!(zero["scope"]["items"], 5);
    // what it offers is sent back as printed: explain of each part, and
    // the corpus beside the same view
    let explains = zero["zero"]["offers"]["explain"].as_array().unwrap();
    assert_eq!(explains.len(), 2, "{}", zero["zero"]);
    for offer in explains {
        let explained = seat.answer(offer);
        assert_eq!(explained["view"]["explain"]["path"], offer["path"]);
    }
    let corpus = seat.answer(&zero["zero"]["offers"]["context"]);
    assert_eq!(corpus["total"]["matched"], 5);
    assert_eq!(corpus["set_aside"]["text"], zero["query"]["text"]);

    // r2's two life lines are 20 and 75: no one occurrence reaches 90
    let args = json!({ "id": "r2", "against": { "text": "\"+# to maximum Life\">=90" } });
    let (shown, refused) = seat.call("show", &args);
    assert!(!refused, "{shown}");
    assert_eq!(shown["against"]["outcome"], "no_match");
    let (printed, _) = cli(
        &seat.base,
        &["show", "r2", "--against", "\"+# to maximum Life\">=90"],
    );
    assert_eq!(shown, printed);
}

/// AQ4: every id an answer printed is accepted back, by `id:` and by
/// `show`, whose document is `acq show`'s — the stored body on request.
#[test]
fn aq4_every_printed_id_is_accepted_back() {
    let mut seat = Seat::new();
    let every = seat.answer(&json!({}));
    let printed = ids(&every)
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert_eq!(printed, ["r1", "r2", "kaom", "plain", "str"]);
    for id in &printed {
        let found = seat.answer(&json!({ "query": { "text": format!("id:{id}") } }));
        assert_eq!(ids(&found), [id.as_str()]);
        let (shown, refused) = seat.call("show", &json!({ "id": id }));
        assert!(!refused, "{shown}");
        assert_eq!(shown["item"]["facts"]["id"], id.as_str());
        assert!(shown.get("body").is_none(), "{shown}");
        assert_eq!(shown, cli(&seat.base, &["show", id]).0);
    }
    // a place's id is its items
    let tab = seat.answer(&json!({ "query": { "text": "id:t1" } }));
    assert_eq!(ids(&tab), ["r1", "r2"]);
    let (with_body, _) = seat.call("show", &json!({ "id": "kaom", "body": true }));
    assert!(
        with_body["body"].as_str().unwrap().contains("Kaom's Heart"),
        "{with_body}"
    );
    assert_eq!(with_body, cli(&seat.base, &["show", "kaom", "--body"]).0);
}

/// AQ5: a mod with a value across every mod array, counted and sorted:
/// 500 and 95 match; `r2`'s 75 and 20 fail, each alone; two lack the line.
#[test]
fn aq5_life_at_ninety_or_more_counted_and_sorted() {
    let mut seat = Seat::new();
    let request = json!({
        "query": { "text": "\"+# to maximum Life\">=90" },
        "view": { "rows": { "sort": "line(\"+# to maximum Life\").arg1", "desc": true } },
    });
    let a = seat.answer(&request);
    assert_eq!(a["total"]["matched"], 2);
    assert_eq!(ids(&a), ["kaom", "r1"]);
    let sorted: Vec<&Value> = a["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| &row["sort"]["value"])
        .collect();
    assert_eq!(sorted, [&json!(500), &json!(95)]);
    let term = &a["terms"][0];
    assert_eq!(
        (
            &term["matched"]["count"],
            &term["failed"]["count"],
            &term["lacked"]["count"]
        ),
        (&json!(2), &json!(1), &json!(2))
    );
    let (typed, _) = cli(
        &seat.base,
        &[
            "search",
            "\"+# to maximum Life\">=90",
            "--sort",
            "line(\"+# to maximum Life\").arg1",
            "--desc",
        ],
    );
    assert_eq!(typed["rows"], a["rows"]);
    assert_eq!(typed["terms"], a["terms"]);
    // the failed count's route, sent back as printed, is r2
    assert_eq!(ids(&seat.answer(&term["failed"])), ["r2"]);
}

/// H1: a `next` is sent back as the answer holds it, and the pages in
/// order are the answer; across a fetch it is refused, the refusal a
/// result marked an error, and what it offers is a request that runs.
#[test]
fn a_next_is_sent_back_whole_and_refused_across_a_changed_basis() {
    let mut seat = Seat::new();
    let first = seat.answer(&json!({ "view": { "rows": { "limit": 2 } } }));
    assert_eq!(ids(&first), ["r1", "r2"]);
    let next = first["view"]["rows"]["next"].clone();
    assert!(
        next["token"].is_string() && next["request"].is_object(),
        "{next}"
    );
    let second = seat.answer(&next);
    assert_eq!(ids(&second), ["kaom", "plain"]);
    // a token alone is no request: the tool takes none
    let (refused, is_error) = seat.call("search", &json!({ "token": next["token"] }));
    assert!(is_error, "{refused}");
    assert_eq!(refused["kind"], "tree");

    {
        let mut store = Store::open(&account_path(&stores(&seat.base), USER)).unwrap();
        record(
            &mut store,
            stash("pc", "t2"),
            "pc",
            json!({ "stash": { "id": "t2", "name": "Dump", "type": "PremiumStash", "items": [] } }),
            60,
        );
    }
    let e = seat.refusal(&next);
    assert_eq!(e["kind"], "basis_changed", "{e}");
    let offers = e["offers"].as_array().unwrap_or_else(|| panic!("{e}"));
    assert_eq!(offers.len(), 1, "{e}");
    let again = seat.answer(&offers[0]);
    assert_eq!(ids(&again), ["r1", "r2"]);
    assert_eq!(again["total"]["matched"], 2);
}

/// H3 and H5: over two realms a request naming none is refused as it is
/// at a terminal, and each realm is offered as the request itself.
#[test]
fn a_realm_needed_is_offered_as_the_request_in_each_realm() {
    let base = scratch();
    seed(&base);
    seed_poe2(&base);
    let mut seat = Seat::over(base);
    let request =
        json!({ "query": { "text": "rarity=rare" }, "view": { "counts": { "keys": ["tab"] } } });
    let e = seat.refusal(&request);
    assert_eq!(e["kind"], "realm_needed", "{e}");
    let offers = e["offers"]
        .as_array()
        .unwrap_or_else(|| panic!("{e}"))
        .clone();
    let matched: Vec<(String, u64)> = offers
        .iter()
        .map(|offer| {
            let a = seat.answer(offer);
            (
                a["scope"]["realm"].as_str().unwrap().to_string(),
                a["total"]["matched"].as_u64().unwrap(),
            )
        })
        .collect();
    // pc: r1, r2, plain, str; poe2: two
    assert_eq!(matched, [("pc".to_string(), 4), ("poe2".to_string(), 1)]);
    let all =
        seat.answer(&json!({ "scope": { "realm": "all" }, "query": { "text": "rarity=rare" } }));
    assert_eq!(all["total"]["matched"], 5);
}

/// An authoring error is the CLI's document in a result marked an error,
/// never a protocol error: its readings are what an agent reads next.
#[test]
fn an_authoring_error_is_a_result_with_its_kind_and_readings() {
    let mut seat = Seat::new();
    let e = seat.refusal(&json!({ "query": { "text": "ring" } }));
    assert_eq!(e["kind"], "bare_word", "{e}");
    assert!(!e["readings"].as_array().unwrap().is_empty(), "{e}");
    let e = seat.refusal(
        &json!({ "query": { "text": "rarity=rare" }, "view": { "explain": { "path": "9" } } }),
    );
    assert_eq!(e["kind"], "no_such_path", "{e}");
    // a request that is none: a key the request does not have
    let e = seat.refusal(&json!({ "text": "rarity=rare" }));
    assert_eq!(e["kind"], "tree", "{e}");
    // `show` of a tab's id says what it is and offers the search of it
    let (e, is_error) = seat.call("show", &json!({ "id": "t1" }));
    assert!(is_error, "{e}");
    assert_eq!(e["kind"], "not_an_item");
    assert_eq!(e, cli(&seat.base, &["show", "t1"]).0);
    let listed = seat.answer(&e["offers"][0]);
    assert_eq!(ids(&listed), ["r1", "r2"]);
    let (e, is_error) = seat.call("show", &json!({ "id": "nope" }));
    assert!(is_error, "{e}");
    assert_eq!(e["kind"], "item_not_found");
    // `against` is a query as a request holds one, its error the binder's
    let (e, is_error) = seat.call(
        "show",
        &json!({ "id": "r1", "against": { "text": "ring" } }),
    );
    assert!(is_error, "{e}");
    assert_eq!(e["kind"], "bare_word");
}

/// `describe` is `acq search --describe`: the language whole, or the
/// entries named, with no store read — it answers over an empty world.
#[test]
fn describe_is_the_clis_and_reads_no_store() {
    let mut seat = Seat::over(scratch());
    let (whole, refused) = seat.call("describe", &json!({}));
    assert!(!refused, "{whole}");
    assert_eq!(whole, cli(&seat.base, &["search", "--describe"]).0);
    let (some, _) = seat.call("describe", &json!({ "names": ["league", "total_res"] }));
    assert_eq!(
        some,
        cli(&seat.base, &["search", "--describe", "league,total_res"]).0
    );
    assert_ne!(some, whole);
    let (e, is_error) = seat.call("describe", &json!({ "names": ["leauge"] }));
    assert!(is_error, "{e}");
    assert_eq!(e["kind"], "unknown_name");
    assert_eq!(e, cli(&seat.base, &["search", "--describe", "leauge"]).0);
}

/// F10 of the agent's seat (R18): where no account's store is found the
/// refusal names the directory it looked in, on `search` and on `show`.
#[test]
fn a_missing_store_names_the_directory_it_looked_in() {
    let mut seat = Seat::over(scratch());
    let looked = stores(&seat.base);
    for (tool, args) in [("search", json!({})), ("show", json!({ "id": "r1" }))] {
        let (e, is_error) = seat.call(tool, &args);
        assert!(is_error, "{tool}: {e}");
        assert_eq!(e["kind"], "no_accounts", "{tool}: {e}");
        assert!(
            e["error"]
                .as_str()
                .unwrap()
                .contains(looked.to_str().unwrap()),
            "{tool}: {e}"
        );
    }
    // the older store tools say the same of it
    let message = seat.mcp.expect_err("store_status", json!({}));
    assert!(message.contains(looked.to_str().unwrap()), "{message}");
}

/// H2: `scope.account` selects the store. With two accounts known a
/// request naming none is refused and lists them; one named is answered
/// from its own store, and a continuation carries it.
#[test]
fn the_requests_account_selects_the_store() {
    let base = scratch();
    seed(&base);
    seed_bob(&base);
    let mut seat = Seat::over(base);
    let (e, is_error) = seat.call("search", &json!({}));
    assert!(is_error, "{e}");
    assert_eq!(e["kind"], "account_needed", "{e}");
    assert_eq!(e["readings"], json!([USER, "Bob#9"]));
    let (e, is_error) = seat.call("search", &json!({ "scope": { "account": "carol" } }));
    assert!(is_error, "{e}");
    assert_eq!(e["kind"], "account_unknown", "{e}");
    assert_eq!(e["readings"], json!([USER, "Bob#9"]));

    let (bob, refused) = seat.call("search", &json!({ "scope": { "account": "bob" } }));
    assert!(!refused, "{bob}");
    assert_eq!(ids(&bob), ["bob1"]);
    let (alice, _) = seat.call(
        "search",
        &json!({ "scope": { "account": USER }, "view": { "rows": { "limit": 2 } } }),
    );
    assert_eq!(ids(&alice), ["r1", "r2"]);
    // every request an answer holds names the account it was answered for
    let (second, refused) = seat.call("search", &alice["view"]["rows"]["next"]);
    assert!(!refused, "{second}");
    assert_eq!(ids(&second), ["kaom", "plain"]);
    let (shown, refused) = seat.call("show", &json!({ "id": "bob1", "account": "u-bob" }));
    assert!(!refused, "{shown}");
    assert_eq!(shown["item"]["facts"]["id"], "bob1");
}

/// The three tools read the store and nothing else: no daemon was
/// reached for or started by any of them.
#[test]
fn no_tool_of_the_search_reaches_for_a_daemon() {
    let mut seat = Seat::new();
    seat.answer(&json!({ "query": { "text": "rarity=rare" } }));
    seat.call("describe", &json!({}));
    seat.call("show", &json!({ "id": "r1" }));
    assert!(no_daemon_appeared(&seat.base));
}
