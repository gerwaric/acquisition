//! Plan step 7, item 3 (`PRICING-SLICE.md`): the pricing surface as one
//! story through the spawned binary — set, show, render, clear — and the
//! races the story invites, each pinned as a property that holds under
//! every interleaving: two blind writers on one row never clobber and
//! their receipts chain (C35, C78); a stale `--if-revision` conflicts
//! naming the current revision and changes nothing (C35); a clear landing
//! under a render never yields a torn page — the render is a function of
//! one read (C74), and the writer is never blocked by the reader (C64:
//! no daemon, no lock a refresh could fight). Facts are seeded through
//! the store crate as `price_json.rs` does; `ACQ_NO_SPAWN=1` makes any
//! daemon contact an error.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

use acquisition_plan::listing::ShowView;
use acquisition_plan::price::PriceWrite;
use acquisition_plan::shop::{Cell, ShopRender};
use acquisition_store::{AnnotationRow, Endpoint, Index, Store, account_path};
use serde_json::{Value, json};

const USER: &str = "Alice#1234";
const UUID: &str = "u-story";

fn command(base: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_acq"));
    cmd.args(args)
        .env("ACQ_STORE_DIR", base)
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
    cmd.env("ACQ_PROVIDER", "mock");
    cmd
}

fn acq(base: &Path, args: &[&str]) -> Output {
    command(base, args).output().expect("spawning acq")
}

/// Start `acq` without waiting, so two can overlap.
fn spawn(base: &Path, args: &[&str]) -> Child {
    command(base, args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawning acq")
}

fn sole_json(out: &Output) -> Value {
    let stdout = String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not exactly one JSON document ({e}):\n{stdout}"))
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A `--json` run's outcome: the receipt, or the error the total `--json`
/// contract prints on stdout with exit 1 (C11).
fn write_outcome(out: &Output) -> Result<PriceWrite, String> {
    let doc = sole_json(out);
    if out.status.success() {
        Ok(serde_json::from_value(doc).unwrap())
    } else {
        Err(doc["error"]
            .as_str()
            .unwrap_or_else(|| panic!("a failed --json run prints {{\"error\":…}}: {doc}"))
            .to_string())
    }
}

fn render(base: &Path) -> ShopRender {
    let out = acq(base, &["shop", "render", "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    serde_json::from_value(sole_json(&out)).unwrap()
}

/// The manual side `show` reads for a target: the value's text and its
/// revision, or nothing.
fn manual(base: &Path, target: &str) -> Option<(String, i64)> {
    let out = acq(base, &["price", "show", target, "--json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let view: ShowView = serde_json::from_value(sole_json(&out)).unwrap();
    view.listing
        .manual
        .map(|m| (m.value.to_string(), m.revision))
}

/// A page's internal consistency: what the counts say is what the pages
/// and the posted list hold (C74: the page is one function of one read).
fn assert_consistent(r: &ShopRender) {
    assert_eq!(r.counts.posted, r.posted.len(), "{r:?}");
    assert_eq!(
        r.counts.posted,
        r.pages.iter().map(|p| p.items).sum::<usize>(),
        "{r:?}"
    );
    assert_eq!(r.counts.pages, r.pages.len());
    assert_eq!(
        r.counts.posted + r.counts.omitted + r.counts.blocked + r.counts.off_page,
        r.counts.items
    );
    for p in &r.posted {
        let page = &r.pages[p.page - 1];
        assert_eq!(page.text.matches(&p.link).count(), 1, "{r:?}");
    }
}

/// One public premium tab, listed at index 0, holding two plain items.
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
    record(
        &mut store,
        Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        },
        json!({ "league": "Standard" }),
        json!({ "stashes": [
            { "id": "p1", "name": "Plain", "type": "PremiumStash", "index": 0, "metadata": { "public": true } } ] }),
    );
    let item = |id: &str, x: i64| json!({ "id": id, "name": "", "typeLine": "Chaos Orb", "baseType": "Chaos Orb", "x": x, "y": 0, "stackSize": 5, "inventoryId": "Stash1" });
    record(
        &mut store,
        Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "p1".into(),
            sub: None,
        },
        json!({}),
        json!({ "stash": { "id": "p1", "name": "Plain", "type": "PremiumStash",
                            "items": [ item("i-a", 0), item("i-b", 1) ] } }),
    );
    mock
}

fn base_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "acq-story-{name}-{}-{}",
        std::process::id(),
        acquisition_store::now()
    ))
}

/// C64, C74, C78 — the story: nothing posts until a hand price is set;
/// `show` reads the row back; the render posts the item under its price
/// with the link the site resolves; `clear` returns the row and the
/// render posts nothing again. No daemon is contacted anywhere in it.
#[test]
fn c74_set_show_render_clear_is_one_story() {
    let base = base_dir("story");
    seed(&base);

    let r = render(&base);
    assert_consistent(&r);
    assert_eq!(
        (r.counts.items, r.counts.posted, r.counts.off_page),
        (2, 0, 2)
    );
    assert!(r.pages.is_empty());

    let w = write_outcome(&acq(
        &base,
        &["price", "set", "item/i-a", "exact", "5", "chaos", "--json"],
    ))
    .unwrap();
    assert!(w.prior.is_none());
    assert_eq!(w.written.as_ref().map(|r| r.revision), Some(1));
    assert_eq!(manual(&base, "item/i-a"), Some(("5 chaos".into(), 1)));

    let r = render(&base);
    assert_consistent(&r);
    assert_eq!(
        (r.counts.posted, r.counts.off_page, r.pages.len()),
        (1, 1, 1)
    );
    assert_eq!(r.posted[0].target.to_string(), "item/i-a");
    assert_eq!(r.posted[0].cell, Cell::StashItem);
    assert_eq!(r.posted[0].title, " ~price 5 chaos");
    assert_eq!(
        r.pages[0].text,
        "[spoiler=\"Shop Post 1 of 1 (1 item)\"]\n\
         [spoiler=\" ~price 5 chaos\"][linkItem realm=\"pc\" location=\"Stash1\" league=\"Standard\" x=\"0\" y=\"0\"][/spoiler]\n\
         [/spoiler]\n"
    );

    let w = write_outcome(&acq(&base, &["price", "clear", "item/i-a", "--json"])).unwrap();
    assert!(w.written.is_none());
    assert_eq!(w.prior.as_ref().map(|r| r.revision), Some(1));
    assert_eq!(manual(&base, "item/i-a"), None);
    let r = render(&base);
    assert_consistent(&r);
    assert_eq!((r.counts.posted, r.counts.off_page), (0, 2));

    let _ = std::fs::remove_dir_all(&base);
}

/// C35, C78 — two blind writers on one row, started together, round
/// after round: whatever the interleaving, at least one lands, no write
/// clobbers another — every receipt's `prior` is exactly the row the
/// write replaced, so the successes chain from the row before the round
/// to the row `show` reads after it — and a loser conflicts naming the
/// revision it lost to, landing nothing. The seed leaves no intent file,
/// so the first round also races its creation: two first-ever writers
/// met `database is locked` there until the store's WAL switch learned
/// to retry (`acquisition-store`, `ensure_wal`).
#[test]
fn c35_two_blind_writers_on_one_row_never_clobber_and_the_receipts_chain() {
    let base = base_dir("writers");
    seed(&base);
    let mut before: Option<AnnotationRow> = None;
    let mut landed = 0usize;
    let mut conflicts = 0usize;
    for round in 1..=5 {
        let a = spawn(
            &base,
            &["price", "set", "item/i-b", "exact", "1", "chaos", "--json"],
        );
        let b = spawn(
            &base,
            &["price", "set", "item/i-b", "b/o", "2", "divine", "--json"],
        );
        let outs = [a.wait_with_output().unwrap(), b.wait_with_output().unwrap()];
        let outcomes: Vec<Result<PriceWrite, String>> = outs.iter().map(write_outcome).collect();
        let mut successes: Vec<&PriceWrite> =
            outcomes.iter().filter_map(|o| o.as_ref().ok()).collect();
        assert!(
            !successes.is_empty(),
            "round {round}: nobody landed: {outcomes:?}"
        );
        successes.sort_by_key(|w| w.written.as_ref().unwrap().revision);
        // The chain: the first success replaced the row before the round,
        // each later one replaced the one before it.
        let mut expect_prior = before.clone();
        for w in &successes {
            assert_eq!(
                w.prior, expect_prior,
                "round {round}: a write replaced a row it did not read"
            );
            expect_prior = w.written.clone();
        }
        landed += successes.len();
        let last = successes.last().unwrap().written.clone().unwrap();
        assert_eq!(
            last.revision, landed as i64,
            "round {round}: the revision sequence carries on"
        );
        for err in outcomes.iter().filter_map(|o| o.as_ref().err()) {
            assert!(
                err.contains(&format!(
                    "item/i-b/buyout is at revision {} (re-read and retry)",
                    last.revision
                )),
                "round {round}: {err}"
            );
            conflicts += 1;
        }
        // What show reads is the last write, at its revision.
        let value: acquisition_plan::price::Buyout =
            serde_json::from_value(last.value.clone()).unwrap();
        assert_eq!(
            manual(&base, "item/i-b"),
            Some((value.to_string(), last.revision))
        );
        before = Some(last);
    }
    assert!(landed >= 5);
    // Whether the rounds overlapped is the machine's to decide; the
    // properties above hold either way. Said, so a run can be read.
    eprintln!("two writers: {landed} landed, {conflicts} conflicted over 5 rounds");
    let _ = std::fs::remove_dir_all(&base);
}

/// C35, C11 — a stale `--if-revision` on `set` and on `clear` conflicts
/// naming the current revision, prints `{"error":…}` under `--json` with
/// exit 1, and changes nothing: the row, its revision and the page are
/// what they were.
#[test]
fn c35_a_stale_revision_conflicts_and_changes_nothing() {
    let base = base_dir("stale");
    seed(&base);
    for args in [
        ["price", "set", "item/i-a", "exact", "5", "chaos", "--json"].as_slice(),
        ["price", "set", "item/i-a", "exact", "6", "chaos", "--json"].as_slice(),
    ] {
        write_outcome(&acq(&base, args)).unwrap();
    }
    assert_eq!(manual(&base, "item/i-a"), Some(("6 chaos".into(), 2)));
    let page = render(&base);
    for stale in [
        vec![
            "price",
            "set",
            "item/i-a",
            "skip",
            "--if-revision",
            "1",
            "--json",
        ],
        vec!["price", "clear", "item/i-a", "--if-revision", "1", "--json"],
        vec![
            "price",
            "set",
            "item/i-a",
            "skip",
            "--if-revision",
            "3",
            "--json",
        ],
    ] {
        let out = acq(&base, &stale);
        assert_eq!(out.status.code(), Some(1), "{stale:?}");
        let err = write_outcome(&out).unwrap_err();
        assert!(
            err.contains("item/i-a/buyout is at revision 2 (re-read and retry)"),
            "{stale:?}: {err}"
        );
        assert_eq!(
            manual(&base, "item/i-a"),
            Some(("6 chaos".into(), 2)),
            "{stale:?}"
        );
    }
    let again = render(&base);
    assert_eq!(again.posted, page.posted);
    assert_eq!(again.pages, page.pages);
    let _ = std::fs::remove_dir_all(&base);
}

/// C64, C74 — a clear landing under a render, round after round: the
/// clear always lands (a reader never holds a writer), and the render is
/// one function of one read — the item is on the page with its link or
/// off it, its counts, its pages and its posted list agreeing either way,
/// never a page that counts an item it does not hold.
#[test]
fn c74_a_clear_under_a_render_is_never_a_torn_page() {
    let base = base_dir("clear-under-render");
    seed(&base);
    let mut seen = [0usize; 2];
    for round in 1..=6 {
        let w = write_outcome(&acq(
            &base,
            &["price", "set", "item/i-a", "exact", "5", "chaos", "--json"],
        ))
        .unwrap();
        let written = w.written.unwrap();
        let render = spawn(&base, &["shop", "render", "--json"]);
        let clear = spawn(&base, &["price", "clear", "item/i-a", "--json"]);
        let render = render.wait_with_output().unwrap();
        let clear = clear.wait_with_output().unwrap();
        let cleared = write_outcome(&clear)
            .unwrap_or_else(|e| panic!("round {round}: the clear did not land: {e}"));
        assert_eq!(
            cleared.prior,
            Some(written),
            "round {round}: the clear removed what the set wrote"
        );
        assert!(
            render.status.success(),
            "round {round}: {}",
            stderr(&render)
        );
        let r: ShopRender = serde_json::from_value(sole_json(&render)).unwrap();
        assert_consistent(&r);
        match r.counts.posted {
            1 => {
                assert_eq!(r.posted[0].target.to_string(), "item/i-a");
                assert!(
                    r.pages[0]
                        .text
                        .contains("[spoiler=\" ~price 5 chaos\"][linkItem")
                );
                seen[0] += 1;
            }
            0 => {
                assert!(r.pages.is_empty());
                assert_eq!(r.counts.off_page, 2);
                seen[1] += 1;
            }
            n => panic!("round {round}: {n} posted from two items, one priced"),
        }
    }
    assert_eq!(seen[0] + seen[1], 6);
    eprintln!(
        "clear under render: {} renders saw the item posted, {} saw it cleared",
        seen[0], seen[1]
    );
    assert_eq!(manual(&base, "item/i-a"), None);
    let r = render(&base);
    assert_consistent(&r);
    assert_eq!(r.counts.posted, 0);
    let _ = std::fs::remove_dir_all(&base);
}
