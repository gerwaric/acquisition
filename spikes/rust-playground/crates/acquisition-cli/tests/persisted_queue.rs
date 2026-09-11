//! C45 through the real binary: an observer with no daemon to ask reads
//! the queue from the ledger on disk — `jobs` lists the non-terminal
//! rows as the last daemon left them, `daemon status` counts them — and
//! neither spawns anything. The ledger is seeded through the store crate
//! (the daemon's own writer), so what the verbs read is exactly what a
//! daemon would have left: two waiting rows (one a child), one recorded
//! running, one done that must not appear.

use acquisition_store::jobs::{JobDb, JobRow, daemon_db_path};
use serde_json::{Value, json};

mod harness;

use harness::{acq, scratch_tmp, sockets_under, sole_json, text};

fn row(id: u64, state: &str, parent: Option<u64>, updated_at: i64) -> JobRow {
    JobRow {
        id,
        kind: "stash".into(),
        state: state.into(),
        priority: 5,
        submitted_by: "cli".into(),
        parent,
        retries: 0,
        account: Some("Exile#1234".into()),
        params: json!({ "league": "Standard", "id": format!("tab{id}") }),
        outcome: None,
        deferred: None,
        cancel_requested: false,
        submitted_at: 1_700_000_000,
        updated_at,
    }
}

#[test]
fn c45_an_observer_with_no_daemon_reads_the_queue_on_disk() {
    let scratch = harness::scratch("c45");
    let base = scratch.0.clone();

    // The ledger as a mock-mode daemon leaves it: under the world's
    // provider directory (C83).
    let dir = base.join("store").join("mock");
    let db = JobDb::open(&daemon_db_path(&dir)).unwrap();
    db.upsert(&row(1, "waiting", None, 1_700_000_010)).unwrap();
    db.upsert(&row(2, "running", None, 1_700_000_030)).unwrap();
    db.upsert(&row(3, "waiting", Some(2), 1_700_000_020))
        .unwrap();
    db.upsert(&row(4, "done", None, 1_700_000_005)).unwrap();
    db.checkpoint().unwrap();
    drop(db);

    // `jobs --json`: running false, the three open rows by id, the
    // newest write as the picture's age.
    let out = acq(&base, &["jobs", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let report = sole_json(&out);
    assert_eq!(report["running"], json!(false));
    assert_eq!(report["persisted"]["written_at"], json!(1_700_000_030));
    let jobs = report["persisted"]["jobs"].as_array().unwrap();
    let ids: Vec<u64> = jobs.iter().map(|j| j["id"].as_u64().unwrap()).collect();
    assert_eq!(ids, vec![1, 2, 3], "{report}");
    assert_eq!(jobs[1]["state"], json!("running"));
    assert_eq!(jobs[2]["parent"], json!(2));
    assert_eq!(jobs[0]["params"]["id"], json!("tab1"));
    assert!(
        jobs[0].get("deferred").is_none(),
        "bulk columns are not read: {report}"
    );

    // The text form: the counts and the shared table, no ETA column
    // filled.
    let out = acq(&base, &["jobs"]);
    assert!(out.status.success(), "{out:?}");
    let shown = text(&out);
    assert!(shown.contains("2 waiting, 1 recorded running"), "{shown}");
    assert!(shown.contains("Standard/tab3"), "{shown}");
    assert!(!shown.contains("tab4"), "a done row was listed: {shown}");

    // `daemon status --json`: the counts beside `running: false`.
    let out = acq(&base, &["daemon", "status", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        sole_json(&out),
        json!({
            "running": false,
            "persisted": { "waiting": 2, "recorded_running": 1, "written_at": 1_700_000_030 }
        })
    );

    // Nothing spawned: no socket derived into the scratch runtime
    // directory, and the ledger is untouched.
    assert!(
        sockets_under(&scratch_tmp(&base)).is_empty(),
        "an observer spawned"
    );
    let db = JobDb::open(&daemon_db_path(&dir)).unwrap();
    assert_eq!(db.load().unwrap().len(), 4);

    // A world with no ledger yet reads as `persisted: null`, not as an
    // empty queue: the root exists (a store, no daemon ever), the file
    // does not.
    let fresh = harness::scratch("c45-fresh");
    std::fs::create_dir_all(fresh.0.join("store")).unwrap();
    let out = acq(&fresh.0, &["jobs", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(sole_json(&out)["persisted"], Value::Null);
}
