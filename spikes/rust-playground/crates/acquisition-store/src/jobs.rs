//! The persisted job queue: `store/<provider>/daemon.db`, one row per job,
//! written by the daemon at every state change and read back when a
//! daemon starts. The daemon's memory is the source of truth while it
//! runs; this table is its mirror, and the queue survives a restart —
//! idle exit, `daemon stop`, a version respawn, a crash. What a
//! restarting daemon does with each row — resume, replay, hold, or fail
//! as interrupted — is its decision, not this crate's.
//!
//! This crate knows nothing about job semantics (what `running` means for
//! a restart, which kinds are per-lifetime); it stores rows and hands them
//! back. Those decisions are the daemon's (`CONTEXT.md`, "The job queue
//! persists"). Ids come from `AUTOINCREMENT`, so an id is never reused even
//! after its row is pruned: a stale `acq result <id>` names nothing rather
//! than a different job.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::Value;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS jobs (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    kind             TEXT    NOT NULL,
    state            TEXT    NOT NULL,   -- waiting|running|done|failed|cancelled
    priority         INTEGER NOT NULL,
    submitted_by     TEXT    NOT NULL,
    parent           INTEGER,
    retries          INTEGER NOT NULL,
    account          TEXT,
    params           TEXT    NOT NULL,   -- verbatim JSON
    outcome          TEXT,               -- JSON, set once terminal
    deferred         TEXT,               -- a parent's held-back outcome while its children run
    cancel_requested INTEGER NOT NULL DEFAULT 0,
    submitted_at     INTEGER NOT NULL,   -- unix seconds
    updated_at       INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS jobs_state ON jobs (state);
";

/// One job as the table holds it. Column-for-column with the daemon's
/// `JobInfo` plus the restart-relevant runtime fields.
#[derive(Clone, Debug, PartialEq)]
pub struct JobRow {
    pub id: u64,
    pub kind: String,
    pub state: String,
    pub priority: u8,
    pub submitted_by: String,
    pub parent: Option<u64>,
    pub retries: u32,
    pub account: Option<String>,
    pub params: Value,
    pub outcome: Option<Value>,
    pub deferred: Option<Value>,
    pub cancel_requested: bool,
    pub submitted_at: i64,
    pub updated_at: i64,
}

/// How long terminal rows are kept, in days, by final state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Retention {
    pub done_days: u32,
    pub failed_days: u32,
}

impl Default for Retention {
    fn default() -> Self {
        Retention {
            done_days: 7,
            failed_days: 30,
        }
    }
}

pub fn daemon_db_path(dir: &Path) -> PathBuf {
    dir.join("daemon.db")
}

pub struct JobDb {
    conn: Connection,
    path: PathBuf,
}

impl JobDb {
    pub fn open(path: &Path) -> Result<JobDb> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let conn = Connection::open(path).with_context(|| format!("opening {}", path.display()))?;
        Self::init(conn, path.to_path_buf())
    }

    pub fn open_memory() -> Result<JobDb> {
        Self::init(Connection::open_in_memory()?, PathBuf::from(":memory:"))
    }

    fn init(conn: Connection, path: PathBuf) -> Result<JobDb> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch(SCHEMA)?;
        Ok(JobDb { conn, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Make every later statement fail, so the daemon's queue-failure
    /// handling can be exercised. Compiled only for builds that opt into
    /// the `test-hooks` feature (acquisition-core's dev-dependency);
    /// production builds have no way to call this.
    #[cfg(feature = "test-hooks")]
    pub fn break_for_tests(&self) {
        self.conn.execute_batch("DROP TABLE jobs").unwrap();
    }

    /// Insert or replace the row for `row.id`. One small transaction; the
    /// daemon calls this under the lock that guards its in-memory job, so
    /// the table sees every change in the order memory did.
    pub fn upsert(&self, row: &JobRow) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO jobs
             (id, kind, state, priority, submitted_by, parent, retries, account, params,
              outcome, deferred, cancel_requested, submitted_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                row.id,
                row.kind,
                row.state,
                row.priority,
                row.submitted_by,
                row.parent,
                row.retries,
                row.account,
                row.params.to_string(),
                row.outcome.as_ref().map(Value::to_string),
                row.deferred.as_ref().map(Value::to_string),
                row.cancel_requested,
                row.submitted_at,
                row.updated_at,
            ],
        )?;
        Ok(())
    }

    /// One row by id, if it exists.
    pub fn get(&self, id: u64) -> Result<Option<JobRow>> {
        let mut rows = self.select("WHERE id = ?1", params![id])?;
        Ok(rows.pop())
    }

    pub fn delete(&self, id: u64) -> Result<()> {
        self.conn
            .execute("DELETE FROM jobs WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Every row, by id.
    pub fn load(&self) -> Result<Vec<JobRow>> {
        self.select("", [])
    }

    /// The rows a restarting daemon takes back: everything not yet
    /// terminal, plus the finished children of those (a parent's own
    /// result summarizes its children, so a parent held open needs them;
    /// one level suffices — a parent only finishes once every descendant
    /// has).
    pub fn load_open(&self) -> Result<Vec<JobRow>> {
        self.select(
            "WHERE state IN ('waiting', 'running')
                OR parent IN (SELECT id FROM jobs WHERE state IN ('waiting', 'running'))",
            [],
        )
    }

    fn select(&self, filter: &str, args: impl rusqlite::Params) -> Result<Vec<JobRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT id, kind, state, priority, submitted_by, parent, retries, account, params,
                    outcome, deferred, cancel_requested, submitted_at, updated_at
             FROM jobs {filter} ORDER BY id"
        ))?;
        let rows = stmt.query_map(args, |r| {
            Ok(JobRow {
                id: r.get(0)?,
                kind: r.get(1)?,
                state: r.get(2)?,
                priority: r.get(3)?,
                submitted_by: r.get(4)?,
                parent: r.get(5)?,
                retries: r.get(6)?,
                account: r.get(7)?,
                params: parse(r.get::<_, String>(8)?),
                outcome: r.get::<_, Option<String>>(9)?.map(parse),
                deferred: r.get::<_, Option<String>>(10)?.map(parse),
                cancel_requested: r.get(11)?,
                submitted_at: r.get(12)?,
                updated_at: r.get(13)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    /// The next id a new job gets: one past the largest ever issued, pruned
    /// rows included (`sqlite_sequence` remembers what the table forgot).
    pub fn next_id(&self) -> Result<u64> {
        let seq: Option<u64> = self
            .conn
            .query_row(
                "SELECT seq FROM sqlite_sequence WHERE name = 'jobs'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(seq.unwrap_or(0) + 1)
    }

    /// Delete terminal rows older than the retention for their state, as
    /// of `now` (unix seconds). A finished child of a still-open parent is
    /// live state, not history, and stays. Returns how many went.
    pub fn prune(&self, retention: Retention, now: i64) -> Result<usize> {
        let day = 86_400i64;
        let done_before = now - i64::from(retention.done_days) * day;
        let failed_before = now - i64::from(retention.failed_days) * day;
        let n = self.conn.execute(
            "DELETE FROM jobs
             WHERE ((state IN ('done', 'cancelled') AND updated_at < ?1)
                 OR (state = 'failed' AND updated_at < ?2))
               AND (parent IS NULL
                    OR parent NOT IN (SELECT id FROM jobs WHERE state IN ('waiting', 'running')))",
            params![done_before, failed_before],
        )?;
        Ok(n)
    }
}

fn parse(text: String) -> Value {
    serde_json::from_str(&text).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(id: u64, state: &str, at: i64) -> JobRow {
        JobRow {
            id,
            kind: "stash".into(),
            state: state.into(),
            priority: 3,
            submitted_by: "test".into(),
            parent: (id > 1).then_some(1),
            retries: 0,
            account: Some("Alice#1234".into()),
            params: json!({ "league": "Standard", "id": format!("t{id}") }),
            outcome: (state == "done").then(|| json!({ "outcome": "success", "payload": {} })),
            deferred: None,
            cancel_requested: false,
            submitted_at: at,
            updated_at: at,
        }
    }

    #[test]
    fn rows_round_trip_and_replace_in_place() {
        let db = JobDb::open_memory().unwrap();
        let mut r = row(1, "waiting", 100);
        db.upsert(&r).unwrap();
        r.state = "running".into();
        r.deferred = Some(json!({ "outcome": "success", "payload": { "tabs": 2 } }));
        db.upsert(&r).unwrap();
        db.upsert(&row(2, "waiting", 101)).unwrap();
        assert_eq!(db.load().unwrap(), vec![r, row(2, "waiting", 101)]);
    }

    #[test]
    fn next_id_never_reuses_a_pruned_id() {
        let dir = std::env::temp_dir().join(format!("acq-jobdb-{}", std::process::id()));
        let path = dir.join("daemon.db");
        let _ = std::fs::remove_file(&path);
        {
            let db = JobDb::open(&path).unwrap();
            assert_eq!(db.next_id().unwrap(), 1, "an empty table starts at 1");
            db.upsert(&row(7, "done", 0)).unwrap();
            assert_eq!(db.prune(Retention::default(), 10 * 86_400).unwrap(), 1);
            assert!(db.load().unwrap().is_empty());
        }
        let db = JobDb::open(&path).unwrap();
        assert_eq!(
            db.next_id().unwrap(),
            8,
            "the sequence survives prune and reopen"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_rows_are_the_queue_and_get_finds_history() {
        let db = JobDb::open_memory().unwrap();
        db.upsert(&row(1, "done", 0)).unwrap();
        db.upsert(&row(2, "running", 0)).unwrap();
        db.upsert(&row(3, "waiting", 0)).unwrap();
        db.upsert(&row(4, "failed", 0)).unwrap();
        let mut child = row(5, "done", 0);
        child.parent = Some(2);
        db.upsert(&child).unwrap();
        let open: Vec<u64> = db.load_open().unwrap().iter().map(|r| r.id).collect();
        assert_eq!(open, [2, 3, 5], "an open parent's finished child comes too");
        assert_eq!(db.get(1).unwrap(), Some(row(1, "done", 0)));
        assert_eq!(db.get(9).unwrap(), None);
    }

    #[test]
    fn prune_keeps_failures_longer_than_successes() {
        let db = JobDb::open_memory().unwrap();
        let day = 86_400;
        let now = 100 * day;
        db.upsert(&row(1, "done", now - 8 * day)).unwrap();
        db.upsert(&row(2, "cancelled", now - 8 * day)).unwrap();
        db.upsert(&row(3, "failed", now - 8 * day)).unwrap();
        db.upsert(&row(4, "failed", now - 31 * day)).unwrap();
        db.upsert(&row(5, "done", now - 6 * day)).unwrap();
        db.upsert(&row(6, "waiting", now - 40 * day)).unwrap();
        let mut child = row(7, "done", now - 40 * day);
        child.parent = Some(6);
        db.upsert(&child).unwrap();
        assert_eq!(db.prune(Retention::default(), now).unwrap(), 3);
        let left: Vec<u64> = db.load().unwrap().iter().map(|r| r.id).collect();
        assert_eq!(
            left,
            [3, 5, 6, 7],
            "a waiting row is never pruned by age, nor an open parent's child"
        );
    }
}
