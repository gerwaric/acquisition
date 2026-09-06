//! Process-level pin of the `--json` stdout contract for the pricing
//! surface (C53: text is a function of JSON — the documents `acq price
//! list|show --json` print are the [`ListView`] and [`ShowView`] the
//! text renderers read, whole; `status --json` is the report without
//! its listings). Facts and intent are seeded through the store crate,
//! as `plan_json.rs` does, so no daemon runs: `ACQ_NO_SPAWN=1` makes any
//! contact an error.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use acquisition_plan::listing::{ListView, ListingReport, ShowView};
use acquisition_plan::price::Buyout;
use acquisition_store::{Annotations, Endpoint, Index, Provenance, Store, account_path};
use serde_json::{Value, json};

const USER: &str = "Alice#1234";
const UUID: &str = "u-price";

fn acq(base: &Path, args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_STORE_DIR", base)
        .env("ACQ_SOCKET", base.join("no.sock"))
        .env("ACQ_NO_KEYRING", "1")
        .env("ACQ_NO_SPAWN", "1");
    for var in [
        "ACQ_GGG",
        "ACQ_ACCOUNT",
        "ACQ_TRIPWIRE",
        "ACQ_MAX_SENDS",
        "ACQ_JOURNAL",
        "ACQ_IDLE_SHUTDOWN",
    ] {
        cmd.env_remove(var);
    }
    cmd.output().expect("spawning acq")
}

fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A folder holding a public priced tab with two items (one noted, one
/// with a row), a map tab with a substash holding one item, a character
/// wearing one; a row on the folder and one naming nothing here.
fn seed(base: &Path) -> PathBuf {
    let mock = base.join("mock");
    std::fs::create_dir_all(&mock).unwrap();
    let at = acquisition_store::now();
    let mut index = Index::load(&mock).unwrap();
    index.record_login(USER, UUID, false, at).unwrap();
    let mut store = Store::open(&account_path(&mock, USER)).unwrap();
    let record = |store: &mut Store, ep: Endpoint, params: Value, body: Value| {
        store.record(&ep, &params, 200, &body, at).unwrap();
    };
    record(
        &mut store,
        Endpoint::Profile,
        json!({}),
        json!({ "uuid": UUID, "name": USER }),
    );
    let stashes = |realm: &str, league: &str| Endpoint::Stashes {
        realm: realm.into(),
        league: league.into(),
    };
    record(
        &mut store,
        stashes("pc", "Standard"),
        json!({ "league": "Standard" }),
        json!({ "stashes": [
            { "id": "f1", "name": "Sale", "type": "Folder", "index": 0,
              "children": [ { "id": "c1", "name": "~price 3 chaos", "type": "PremiumStash", "index": 1, "metadata": { "public": true } } ] },
            { "id": "m1", "name": "Maps", "type": "MapStash", "index": 2 } ] }),
    );
    let item = |id: &str, note: Option<&str>| {
        let mut v = json!({ "id": id, "name": "", "typeLine": "Chaos Orb", "baseType": "Chaos Orb", "x": 0, "y": 0, "stackSize": 5 });
        if let Some(n) = note {
            v["note"] = json!(n);
        }
        v
    };
    let stash = |id: &str, sub: Option<&str>| Endpoint::Stash {
        realm: "pc".into(),
        league: "Standard".into(),
        id: id.into(),
        sub: sub.map(str::to_string),
    };
    record(
        &mut store,
        stash("c1", None),
        json!({}),
        json!({ "stash": { "id": "c1", "name": "~price 3 chaos", "type": "PremiumStash",
                            "items": [ item("i-noted", Some("~price 5 chaos")), item("i-plain", None) ] } }),
    );
    record(
        &mut store,
        stash("m1", None),
        json!({}),
        json!({ "stash": { "id": "m1", "name": "Maps", "type": "MapStash", "items": [],
                            "children": [ { "id": "s1", "name": "1", "type": "MapStash", "metadata": { "items": 1 } } ] } }),
    );
    record(
        &mut store,
        stash("m1", Some("s1")),
        json!({}),
        json!({ "stash": { "id": "s1", "name": "1", "type": "MapStash", "items": [ item("i-map", None) ] } }),
    );
    record(
        &mut store,
        Endpoint::Characters { realm: "pc".into() },
        json!({ "realm": "pc" }),
        json!({ "characters": [ { "id": "ch1", "name": "Exile", "league": "Standard" } ] }),
    );
    record(
        &mut store,
        Endpoint::Character {
            realm: "pc".into(),
            name: "Exile".into(),
        },
        json!({ "realm": "pc", "name": "Exile" }),
        json!({ "character": { "id": "ch1", "name": "Exile", "league": "Standard",
                                "equipment": [ item("i-worn", Some("~b/o 2 divine")) ], "inventory": [] } }),
    );
    let mut a = Annotations::open_for(&mock, UUID).unwrap();
    let via = Provenance::via("test");
    a.put::<Buyout>(
        "item",
        "i-plain",
        &json!({ "version": 1, "type": "exact", "amount": "3", "currency": "chaos" }),
        None,
        &via,
    )
    .unwrap();
    a.put::<Buyout>(
        "tab",
        "pc/f1",
        &json!({ "version": 1, "type": "skip" }),
        None,
        &via,
    )
    .unwrap();
    a.put::<Buyout>(
        "item",
        "i-elsewhere",
        &json!({ "version": 1, "type": "skip" }),
        None,
        &via,
    )
    .unwrap();
    mock
}

#[test]
fn price_json_documents_are_the_views_the_text_reads() {
    let base = std::env::temp_dir().join(format!(
        "acq-price-json-{}-{}",
        std::process::id(),
        acquisition_store::now()
    ));
    seed(&base);

    // status: the report without listings, re-read exactly.
    let out = acq(&base, &["price", "status", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let report: ListingReport = serde_json::from_value(sole_json(&out)).unwrap();
    assert!(report.listings.is_empty());
    assert_eq!(report.counts.items, 4);
    assert_eq!(report.counts.containers, 5);
    assert_eq!(report.rows.total, 3);
    assert_eq!(report.rows.applied, 2);
    assert_eq!(report.rows.unmatched.len(), 1);

    // list: the view carries the containers its text names.
    let out = acq(&base, &["price", "list", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let view: ListView = serde_json::from_value(sole_json(&out)).unwrap();
    // The unpriced map item and the character's noted item are relation
    // `none` (a character's items are not indexed, C81): left out by default.
    assert_eq!(view.items.len(), 2);
    // The containers on the selected items' chains, in the report's
    // order: the folder (on the chain), the tab, the character.
    let containers: Vec<&str> = view
        .containers
        .iter()
        .map(|c| c.subject.name.as_str())
        .collect();
    assert_eq!(containers, ["Sale", "~price 3 chaos"]);
    assert_eq!(view.items_on_record, 4);
    // The same selection in text names the same containers, in order.
    let out = acq(&base, &["price", "list"]);
    let text = String::from_utf8_lossy(&out.stdout);
    let pos = |s: &str| {
        text.find(s)
            .unwrap_or_else(|| panic!("{s:?} not in\n{text}"))
    };
    assert!(
        pos("~price 3 chaos (3 chaos, public)") < pos("next:"),
        "{text}"
    );
    assert!(!text.contains("Sale  "), "{text}");
    // The residue items under `--relation none`: a substash labelled under
    // its parent's name, which the view carries; the character last.
    let out = acq(&base, &["price", "list", "--relation", "none"]);
    let text = String::from_utf8_lossy(&out.stdout);
    let pos = |s: &str| {
        text.find(s)
            .unwrap_or_else(|| panic!("{s:?} not in\n{text}"))
    };
    assert!(
        pos("Maps / 1 (not public)  1 item: 1 unlisted  substash/pc/m1/s1")
            < pos("Exile  1 item: 1 unlisted"),
        "{text}"
    );

    // --covered-by a folder: its tab's items, through the chain (C70);
    // --in the folder: nothing physically there.
    let out = acq(
        &base,
        &["price", "list", "--covered-by", "tab/pc/f1", "--json"],
    );
    assert!(out.status.success(), "{}", stderr(&out));
    let view: ListView = serde_json::from_value(sole_json(&out)).unwrap();
    let ids: Vec<String> = view
        .items
        .iter()
        .map(|l| l.subject.target.to_string())
        .collect();
    assert_eq!(ids, ["item/i-noted", "item/i-plain"]);
    let out = acq(
        &base,
        &["price", "list", "--in", "tab/pc/f1", "--relation", "none"],
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with(
            "4 items on record, none match for Standard with relation none in tab/pc/f1\n"
        ),
        "{text}"
    );

    // show: an item's view carries its container; a folder's carries the
    // covered set; both re-read exactly.
    let out = acq(&base, &["price", "show", "item/i-noted", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let view: ShowView = serde_json::from_value(sole_json(&out)).unwrap();
    assert_eq!(
        view.container.as_ref().map(|c| c.subject.name.as_str()),
        Some("~price 3 chaos")
    );
    assert_eq!(
        view.listing.game.note.as_ref().map(|n| n.text.as_str()),
        Some("~price 5 chaos")
    );
    assert_eq!(
        view.listing.manual.as_ref().map(|m| m.from.to_string()),
        Some("tab/pc/f1".into())
    );
    // C81: the note beats the folder's row; the site's view of it is the price.
    assert_eq!(
        view.listing.effective.side,
        Some(acquisition_plan::listing::Side::Game)
    );
    assert_eq!(view.listing.effective.to_string(), "5 chaos");
    let out = acq(&base, &["price", "show", "tab/pc/f1", "--json"]);
    let view: ShowView = serde_json::from_value(sole_json(&out)).unwrap();
    assert!(view.items_here.is_empty());
    assert_eq!(view.items_covered_below.len(), 2);
    let out = acq(&base, &["price", "show", "tab/pc/f1"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.contains("items here: none on record\nitems covered through children (C70): 2 items"),
        "{text}"
    );

    // A container not on record, or another realm's address, is a
    // refusal naming the facts — never an empty selection.
    let out = acq(&base, &["price", "list", "--in", "tab/pc/nope"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("tab/pc/nope is not in the facts for Standard"),
        "{}",
        stderr(&out)
    );
    let out = acq(
        &base,
        &["price", "list", "--in", "tab/xbox/c1", "--realm", "pc"],
    );
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("tab/xbox/c1 is a xbox address; pc was given"),
        "{}",
        stderr(&out)
    );
    let out = acq(&base, &["price", "list", "--in", "tab/xbox/c1"]);
    assert!(!out.status.success());
    assert!(
        stderr(&out).contains("not in the facts for xbox/Standard"),
        "{}",
        stderr(&out)
    );

    // A league with no facts still shows the intent rows (C35).
    let out = acq(&base, &["price", "status", "--league", "Hardcore"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        text.starts_with(
            "no items on record for Hardcore: 0 tabs and characters listed, none fetched with items\n\
             rows: 0 of 3 apply here; 3 name nothing in these facts, 0 unreadable, 0 for other realms\n"
        ),
        "{text}"
    );

    let _ = std::fs::remove_dir_all(&base);
}
