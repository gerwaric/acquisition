//! C45 through the MCP binary: `list_jobs` and `daemon_status` with no
//! daemon to ask answer from the ledger on disk, and spawn nothing. The
//! ledger is seeded through the store crate, the daemon's own writer.

mod harness;

use std::path::PathBuf;

use acquisition_store::jobs::{JobDb, JobRow, daemon_db_path};
use harness::Mcp;
use serde_json::json;

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn row(id: u64, state: &str, updated_at: i64) -> JobRow {
    JobRow {
        id,
        kind: "stash".into(),
        state: state.into(),
        priority: 5,
        submitted_by: "mcp".into(),
        parent: None,
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
fn c45_the_tools_read_the_queue_on_disk_when_no_daemon_answers() {
    let base = std::env::temp_dir().join(format!("acq-mcp-c45-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let _scratch = Scratch(base.clone());

    let dir = base.join("store").join("mock");
    let db = JobDb::open(&daemon_db_path(&dir)).unwrap();
    db.upsert(&row(1, "waiting", 1_700_000_010)).unwrap();
    db.upsert(&row(2, "running", 1_700_000_030)).unwrap();
    db.upsert(&row(3, "cancelled", 1_700_000_040)).unwrap();
    db.checkpoint().unwrap();
    drop(db);

    let mut mcp = Mcp::start(&base, &[]);
    let report = mcp.expect_ok("list_jobs", json!({}));
    assert_eq!(report["running"], json!(false), "{report}");
    let ids: Vec<u64> = report["persisted"]["jobs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|j| j["id"].as_u64().unwrap())
        .collect();
    assert_eq!(ids, vec![1, 2], "{report}");
    // The picture's age is the newest write of any row, the cancelled
    // one included: it says how stale the ledger is, not the open rows.
    assert_eq!(report["persisted"]["written_at"], json!(1_700_000_040));

    assert_eq!(
        mcp.expect_ok("daemon_status", json!({})),
        json!({
            "running": false,
            "persisted": { "waiting": 1, "recorded_running": 1, "written_at": 1_700_000_040 }
        })
    );
    assert!(
        harness::no_daemon_appeared(&base),
        "a tool spawned a daemon"
    );
}
