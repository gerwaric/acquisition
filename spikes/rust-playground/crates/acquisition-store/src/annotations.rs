//! Intent (annotations): the only irreplaceable local state (CONTEXT.md,
//! decided 2026-08-31). Facts are refetchable at the cost of requests;
//! intent has no server to refetch from, so it lives in its own per-account
//! file — named by the account **uuid** (stable across renames), never the
//! username — and is written only through this API, with integer-revision
//! compare-and-swap so two frontends cannot silently clobber each other.
//!
//! No fact-side event ever deletes intent: an annotation whose item is
//! removed is kept and surfaceable as orphaned
//! ([`crate::Store::orphaned_item_annotations`]). Deleting an annotation is
//! itself an act of intent, so [`Annotations::delete`] exists — for
//! frontends, under the same compare-and-swap — and it *tombstones* rather
//! than removes: the revision sequence survives a delete/recreate, so a
//! writer holding a pre-delete revision can never silently overwrite the
//! recreated value (the ABA hole a hard delete would open).
//!
//! Rows are addressed `(scope, key, kind)`:
//! - `scope` is what kind of thing the key names: `"item"` (key = GGG item
//!   id), `"tab"` (key = tab id; substash identity is the caller's
//!   `parent/id` convention), `"account"` (key = `""` for per-account
//!   singletons like the sync policy).
//! - `kind` is what the annotation says: `"buyout"`, `"note"`,
//!   `"sync-policy"`, …
//! - `value` is JSON; its shape is the kind's business.
//!
//! Backup is store-managed: [`Annotations::export`] writes a consistent
//! snapshot via `VACUUM INTO` — a raw file copy under WAL is not a backup.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::index::filename_safe;

/// The schema this build reads and writes. A file stamped newer is refused
/// (never auto-migrated: this is the one file that must not be damaged);
/// additions like the deferred event log bump this and migrate forward.
/// v2 added `meta`, which carries the account uuid *inside* the file so a
/// copied or renamed database cannot silently pair with another account's
/// facts — the filename convention alone was bypassable.
const SCHEMA_VERSION: i64 = 2;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS annotations (
    scope       TEXT    NOT NULL,
    key         TEXT    NOT NULL,
    kind        TEXT    NOT NULL,
    value       TEXT    NOT NULL,   -- JSON
    revision    INTEGER NOT NULL,   -- 1 on create, +1 per write, delete included
    created_at  INTEGER NOT NULL,   -- unix seconds
    updated_at  INTEGER NOT NULL,
    deleted_at  INTEGER,            -- tombstone; the revision keeps counting
    PRIMARY KEY (scope, key, kind)
);
CREATE TABLE IF NOT EXISTS meta (
    key    TEXT PRIMARY KEY,        -- 'account_uuid'
    value  TEXT NOT NULL
);
";

/// The `meta` key holding the owning account's uuid.
const META_UUID: &str = "account_uuid";

/// `<dir>/<uuid>.annotations.db` — beside the username-named fact files,
/// but keyed by the identity that survives a rename.
pub fn annotations_path(dir: &Path, uuid: &str) -> PathBuf {
    dir.join(format!("{}.annotations.db", filename_safe(uuid)))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnnotationRow {
    pub scope: String,
    pub key: String,
    pub kind: String,
    pub value: Value,
    pub revision: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug)]
pub enum AnnotationError {
    /// The compare-and-swap failed: someone else wrote (or removed) this
    /// annotation since the caller read it. `current` is the row as it
    /// exists now (`None`: no such annotation) — enough to re-read, merge,
    /// and retry without another round trip. Boxed so the error stays small.
    Conflict {
        current: Option<Box<AnnotationRow>>,
    },
    /// The file was written by a newer build. Refused rather than guessed
    /// at: annotations are the state that must never be damaged.
    SchemaTooNew {
        found: i64,
        supported: i64,
    },
    /// The file carries another account's uuid: a copy or rename cannot
    /// silently pair one account's intent with another account's facts.
    WrongAccount {
        stored: String,
        requested: String,
    },
    Db(rusqlite::Error),
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnnotationError::Conflict { current: Some(row) } => write!(
                f,
                "annotation {}/{}/{} is at revision {} (re-read and retry)",
                row.scope, row.key, row.kind, row.revision
            ),
            AnnotationError::Conflict { current: None } => {
                write!(f, "annotation no longer exists (re-read and retry)")
            }
            AnnotationError::SchemaTooNew { found, supported } => write!(
                f,
                "annotation file uses schema v{found}, newer than this build's v{supported}"
            ),
            AnnotationError::WrongAccount { stored, requested } => write!(
                f,
                "annotation file belongs to account uuid {stored}, not {requested}"
            ),
            AnnotationError::Db(e) => write!(f, "annotation store: {e}"),
            AnnotationError::Io(e) => write!(f, "annotation store: {e}"),
            AnnotationError::Json(e) => write!(f, "annotation store: {e}"),
        }
    }
}

impl std::error::Error for AnnotationError {}

impl From<rusqlite::Error> for AnnotationError {
    fn from(e: rusqlite::Error) -> Self {
        AnnotationError::Db(e)
    }
}

impl From<std::io::Error> for AnnotationError {
    fn from(e: std::io::Error) -> Self {
        AnnotationError::Io(e)
    }
}

impl From<serde_json::Error> for AnnotationError {
    fn from(e: serde_json::Error) -> Self {
        AnnotationError::Json(e)
    }
}

pub struct Annotations {
    conn: Connection,
    path: PathBuf,
    /// The owning account's uuid as stored in the file's `meta` table;
    /// `None` for a pre-v2 file never opened via [`Annotations::open_for`].
    uuid: Option<String>,
}

impl Annotations {
    /// Open (or create) the account's annotations file under `dir`, bound
    /// to `uuid`: the uuid is stored inside the file, and a file already
    /// carrying a different account's uuid is refused
    /// ([`AnnotationError::WrongAccount`]) — a copy or rename cannot
    /// silently pair another account's intent. A pre-v2 file has no stored
    /// uuid; the uuid it is addressed by is stamped on this first open
    /// (the filename convention was its only binding, upgraded here).
    /// This is the way to open annotations for real use; [`Annotations::open`]
    /// on a raw path yields a handle without a verified identity, which
    /// [`crate::Store::stash_snapshot`] refuses.
    pub fn open_for(dir: &Path, uuid: &str) -> Result<Annotations, AnnotationError> {
        let mut a = Self::open(&annotations_path(dir, uuid))?;
        a.bind(uuid)?;
        Ok(a)
    }

    /// An in-memory file bound to `uuid`, for tests and ephemeral use.
    pub fn open_memory_for(uuid: &str) -> Result<Annotations, AnnotationError> {
        let mut a = Self::open_memory()?;
        a.bind(uuid)?;
        Ok(a)
    }

    pub fn open(path: &Path) -> Result<Annotations, AnnotationError> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Self::init(Connection::open(path)?, path.to_path_buf())
    }

    pub fn open_memory() -> Result<Annotations, AnnotationError> {
        Self::init(Connection::open_in_memory()?, PathBuf::from(":memory:"))
    }

    fn init(mut conn: Connection, path: PathBuf) -> Result<Annotations, AnnotationError> {
        // WAL like the fact store: one writer at a time, any number of readers.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        // Creation and migration serialize under one immediate transaction
        // so two processes opening the same file cannot interleave them.
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let found: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        match found {
            // 0 is a fresh file; 1 gains the `meta` table (an addition —
            // the annotation rows are not touched).
            0 | 1 => {
                tx.execute_batch(SCHEMA)?;
                tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            }
            v if v == SCHEMA_VERSION => {}
            v => {
                return Err(AnnotationError::SchemaTooNew {
                    found: v,
                    supported: SCHEMA_VERSION,
                });
            }
        }
        tx.commit()?;
        let uuid: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key = ?1", [META_UUID], |r| {
                r.get(0)
            })
            .optional()?;
        Ok(Annotations { conn, path, uuid })
    }

    /// Bind this handle to `uuid`: stamp it into a file that has none, or
    /// verify it against the stored one.
    fn bind(&mut self, uuid: &str) -> Result<(), AnnotationError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let stored: Option<String> = tx
            .query_row("SELECT value FROM meta WHERE key = ?1", [META_UUID], |r| {
                r.get(0)
            })
            .optional()?;
        match stored {
            None => {
                tx.execute(
                    "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                    params![META_UUID, uuid],
                )?;
            }
            Some(ref u) if u == uuid => {}
            Some(u) => {
                return Err(AnnotationError::WrongAccount {
                    stored: u,
                    requested: uuid.into(),
                });
            }
        }
        tx.commit()?;
        self.uuid = Some(uuid.into());
        Ok(())
    }

    /// The owning account's uuid, when the file (or this handle) carries
    /// one. `None` means the pairing is uncheckable — a pre-v2 file opened
    /// from a raw path — and consumers that pair intent with facts refuse
    /// such handles.
    pub fn uuid(&self) -> Option<&str> {
        self.uuid.as_deref()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write one annotation under compare-and-swap. `expected_revision` is
    /// what the caller last read: `None` creates (fails if the annotation
    /// currently exists), `Some(r)` updates (fails unless the current
    /// revision is exactly `r`). Returns the row as written; a mismatch is
    /// [`AnnotationError::Conflict`] carrying the current row. Creating
    /// over a tombstone continues its revision sequence — revisions are
    /// monotonic for the life of the file, never reset by delete/recreate,
    /// so a stale writer always conflicts.
    pub fn put(
        &mut self,
        scope: &str,
        key: &str,
        kind: &str,
        value: &Value,
        expected_revision: Option<i64>,
    ) -> Result<AnnotationRow, AnnotationError> {
        let now = crate::now();
        // BEGIN IMMEDIATE: the write lock is taken up front, so two
        // connections racing the same row serialize here (bounded by the
        // busy timeout) and the loser reads the winner's committed state —
        // a real Conflict, not a snapshot-upgrade error at commit.
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = row_of(&tx, scope, key, kind)?;
        let row = match (expected_revision, current) {
            (None, None) => AnnotationRow {
                scope: scope.into(),
                key: key.into(),
                kind: kind.into(),
                value: value.clone(),
                revision: 1,
                created_at: now,
                updated_at: now,
            },
            // Recreation after a delete: the tombstone's revision carries on.
            (None, Some((tombstone, true))) => AnnotationRow {
                value: value.clone(),
                revision: tombstone.revision + 1,
                created_at: now,
                updated_at: now,
                ..tombstone
            },
            (Some(expected), Some((current, false))) if current.revision == expected => {
                AnnotationRow {
                    value: value.clone(),
                    revision: expected + 1,
                    updated_at: now,
                    ..current
                }
            }
            (_, current) => return Err(conflict(current)),
        };
        tx.execute(
            "INSERT OR REPLACE INTO annotations (scope, key, kind, value, revision, created_at, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)",
            params![row.scope, row.key, row.kind, row.value.to_string(), row.revision, row.created_at, row.updated_at],
        )?;
        tx.commit()?;
        Ok(row)
    }

    /// Remove one annotation, under the same compare-and-swap as `put`.
    /// This is a frontend expressing intent — the fact side has no delete
    /// path at all. The row becomes a tombstone (invisible to `get`/`list`)
    /// whose revision keeps counting, so no later writer can slip a stale
    /// value past the delete.
    pub fn delete(
        &mut self,
        scope: &str,
        key: &str,
        kind: &str,
        expected_revision: i64,
    ) -> Result<(), AnnotationError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        match row_of(&tx, scope, key, kind)? {
            Some((current, false)) if current.revision == expected_revision => {
                tx.execute(
                    "UPDATE annotations SET deleted_at = ?4, updated_at = ?4, revision = revision + 1
                      WHERE scope = ?1 AND key = ?2 AND kind = ?3",
                    params![scope, key, kind, crate::now()],
                )?;
            }
            current => return Err(conflict(current)),
        }
        tx.commit()?;
        Ok(())
    }

    pub fn get(
        &self,
        scope: &str,
        key: &str,
        kind: &str,
    ) -> Result<Option<AnnotationRow>, AnnotationError> {
        Ok(row_of(&self.conn, scope, key, kind)?
            .and_then(|(row, deleted)| (!deleted).then_some(row)))
    }

    /// Every live annotation (tombstones excluded), optionally restricted
    /// to one scope, ordered by (scope, key, kind).
    pub fn list(&self, scope: Option<&str>) -> Result<Vec<AnnotationRow>, AnnotationError> {
        let mut stmt = self.conn.prepare(
            "SELECT scope, key, kind, value, revision, created_at, updated_at FROM annotations
              WHERE deleted_at IS NULL AND (?1 IS NULL OR scope = ?1) ORDER BY scope, key, kind",
        )?;
        let rows: Vec<(String, String, String, String, i64, i64, i64)> = stmt
            .query_map([scope], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            })?
            .collect::<Result<_, _>>()?;
        rows.into_iter()
            .map(
                |(scope, key, kind, value, revision, created_at, updated_at)| {
                    Ok(AnnotationRow {
                        scope,
                        key,
                        kind,
                        value: serde_json::from_str(&value)?,
                        revision,
                        created_at,
                        updated_at,
                    })
                },
            )
            .collect()
    }

    /// Store-managed backup: a consistent snapshot of this file at `dest`,
    /// via SQLite's `VACUUM INTO`. Fails if `dest` already exists — a
    /// backup never overwrites another backup silently.
    pub fn export(&self, dest: &Path) -> Result<(), AnnotationError> {
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir)?;
        }
        self.conn
            .execute("VACUUM INTO ?1", [dest.to_string_lossy()])?;
        Ok(())
    }
}

/// A conflict, exposing only a live row: a tombstone reads as "no such
/// annotation", but its revision still gates the next write.
fn conflict(current: Option<(AnnotationRow, bool)>) -> AnnotationError {
    AnnotationError::Conflict {
        current: current.and_then(|(row, deleted)| (!deleted).then(|| Box::new(row))),
    }
}

/// The stored row, tombstones included; the bool is "deleted".
fn row_of(
    conn: &Connection,
    scope: &str,
    key: &str,
    kind: &str,
) -> Result<Option<(AnnotationRow, bool)>, AnnotationError> {
    let found: Option<(String, i64, i64, i64, Option<i64>)> = conn
        .query_row(
            "SELECT value, revision, created_at, updated_at, deleted_at FROM annotations
              WHERE scope = ?1 AND key = ?2 AND kind = ?3",
            params![scope, key, kind],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .optional()?;
    match found {
        None => Ok(None),
        Some((value, revision, created_at, updated_at, deleted_at)) => Ok(Some((
            AnnotationRow {
                scope: scope.into(),
                key: key.into(),
                kind: kind.into(),
                value: serde_json::from_str(&value)?,
                revision,
                created_at,
                updated_at,
            },
            deleted_at.is_some(),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn conflict_revision(e: AnnotationError) -> Option<i64> {
        match e {
            AnnotationError::Conflict { current } => current.map(|r| r.revision),
            other => panic!("expected a conflict, got {other}"),
        }
    }

    #[test]
    fn compare_and_swap_creates_updates_and_conflicts() {
        let mut a = Annotations::open_memory().unwrap();
        // Create requires "does not exist yet".
        let row = a
            .put("item", "i1", "buyout", &json!({"price": "1 divine"}), None)
            .unwrap();
        assert_eq!(row.revision, 1);
        let err = a
            .put("item", "i1", "buyout", &json!({"price": "2 divine"}), None)
            .unwrap_err();
        assert_eq!(conflict_revision(err), Some(1));
        // Update requires the exact current revision.
        let row = a
            .put(
                "item",
                "i1",
                "buyout",
                &json!({"price": "2 divine"}),
                Some(1),
            )
            .unwrap();
        assert_eq!(row.revision, 2);
        assert!(row.created_at <= row.updated_at);
        let stale = a
            .put("item", "i1", "buyout", &json!({"price": "3c"}), Some(1))
            .unwrap_err();
        assert_eq!(conflict_revision(stale), Some(2));
        // The stored value is the last accepted write.
        let got = a.get("item", "i1", "buyout").unwrap().unwrap();
        assert_eq!(got.value, json!({"price": "2 divine"}));
        // The same key under another kind is its own row.
        a.put("item", "i1", "note", &json!("keep"), None).unwrap();
        assert_eq!(a.list(Some("item")).unwrap().len(), 2);
        // Delete is CAS too; a stale delete conflicts and changes nothing.
        let err = a.delete("item", "i1", "buyout", 1).unwrap_err();
        assert_eq!(conflict_revision(err), Some(2));
        a.delete("item", "i1", "buyout", 2).unwrap();
        assert!(a.get("item", "i1", "buyout").unwrap().is_none());
        // Updating what is gone reports "gone", not "wrong revision".
        let err = a
            .put("item", "i1", "buyout", &json!({}), Some(2))
            .unwrap_err();
        assert_eq!(conflict_revision(err), None);
    }

    #[test]
    fn delete_and_recreate_never_reset_the_revision() {
        let mut a = Annotations::open_memory().unwrap();
        // A stale reader remembers revision 1...
        let stale = a
            .put("item", "i1", "buyout", &json!({"price": "1c"}), None)
            .unwrap()
            .revision;
        // ...while someone else deletes and recreates the annotation.
        a.delete("item", "i1", "buyout", 1).unwrap();
        let recreated = a
            .put("item", "i1", "buyout", &json!({"price": "5 divine"}), None)
            .unwrap();
        assert!(
            recreated.revision > stale,
            "recreation continues the sequence ({} > {stale})",
            recreated.revision
        );
        // The stale writer's update conflicts instead of silently landing
        // its pre-delete value over the recreated one (the ABA hole).
        let err = a
            .put("item", "i1", "buyout", &json!({"price": "1c"}), Some(stale))
            .unwrap_err();
        assert_eq!(conflict_revision(err), Some(recreated.revision));
        assert_eq!(
            a.get("item", "i1", "buyout").unwrap().unwrap().value,
            json!({"price": "5 divine"})
        );
        // A stale delete cannot remove the recreated value either.
        assert!(a.delete("item", "i1", "buyout", stale).is_err());
    }

    #[test]
    fn two_connections_conflict_deterministically_never_clobber() {
        let dir = std::env::temp_dir().join(format!(
            "acq-ann-2c-{}-{}",
            std::process::id(),
            crate::now()
        ));
        let path = annotations_path(&dir, "u-race");
        let mut a = Annotations::open(&path).unwrap();
        let mut b = Annotations::open(&path).unwrap();
        // Both connections race the same create. BEGIN IMMEDIATE
        // serializes them, so exactly one wins and the loser gets the
        // documented Conflict carrying the winner's row — never a raw
        // SQLite busy/snapshot error, never two revision-1 writes.
        let results = std::thread::scope(|scope| {
            let ta = scope.spawn(|| a.put("item", "i1", "buyout", &json!({"from": "a"}), None));
            let tb = scope.spawn(|| b.put("item", "i1", "buyout", &json!({"from": "b"}), None));
            [ta.join().unwrap(), tb.join().unwrap()]
        });
        let (ok, err): (Vec<_>, Vec<_>) = results.into_iter().partition(Result::is_ok);
        assert_eq!((ok.len(), err.len()), (1, 1), "one winner, one conflict");
        let winner = ok.into_iter().next().unwrap().unwrap();
        assert_eq!(winner.revision, 1);
        match err.into_iter().next().unwrap().unwrap_err() {
            AnnotationError::Conflict { current: Some(row) } => assert_eq!(*row, winner),
            other => panic!("the loser must see a Conflict, got {other}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn account_singletons_use_the_empty_key() {
        let mut a = Annotations::open_memory().unwrap();
        let policy = json!({ "leagues": ["Standard"], "deep": false });
        a.put("account", "", "sync-policy", &policy, None).unwrap();
        let row = a.get("account", "", "sync-policy").unwrap().unwrap();
        assert_eq!(row.value, policy);
    }

    #[test]
    fn export_is_a_consistent_snapshot_and_never_overwrites() {
        let dir =
            std::env::temp_dir().join(format!("acq-ann-{}-{}", std::process::id(), crate::now()));
        let uuid = "00000000-0000-4000-8000-000000000001";
        let mut a = Annotations::open(&annotations_path(&dir, uuid)).unwrap();
        a.put("item", "i1", "buyout", &json!({"price": "1c"}), None)
            .unwrap();
        let backup = dir.join("backup.db");
        a.export(&backup).unwrap();
        // The snapshot is a complete, standalone annotation file.
        let restored = Annotations::open(&backup).unwrap();
        assert_eq!(restored.list(None).unwrap().len(), 1);
        // A second export to the same path is refused, not an overwrite.
        assert!(a.export(&backup).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_for_stamps_the_uuid_and_refuses_a_foreign_file() {
        let dir = std::env::temp_dir().join(format!(
            "acq-ann-uuid-{}-{}",
            std::process::id(),
            crate::now()
        ));
        // First open stamps; reopening from the raw path still knows the
        // owner, because the uuid lives inside the file.
        {
            let a = Annotations::open_for(&dir, "u-1").unwrap();
            assert_eq!(a.uuid(), Some("u-1"));
        }
        let raw = Annotations::open(&annotations_path(&dir, "u-1")).unwrap();
        assert_eq!(raw.uuid(), Some("u-1"));
        // u-1's file copied over u-2's path: open_for(u-2) refuses to
        // rebind it rather than adopting the copy.
        std::fs::copy(annotations_path(&dir, "u-1"), annotations_path(&dir, "u-2")).unwrap();
        match Annotations::open_for(&dir, "u-2").err() {
            Some(AnnotationError::WrongAccount { stored, requested }) => {
                assert_eq!((stored.as_str(), requested.as_str()), ("u-1", "u-2"));
            }
            other => panic!("expected WrongAccount, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_v1_file_is_migrated_forward_with_its_rows_intact() {
        let dir = std::env::temp_dir().join(format!(
            "acq-ann-mig-{}-{}",
            std::process::id(),
            crate::now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = annotations_path(&dir, "u-1");
        {
            // A v1 file: the annotations table alone, no meta.
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE annotations (
                    scope TEXT NOT NULL, key TEXT NOT NULL, kind TEXT NOT NULL,
                    value TEXT NOT NULL, revision INTEGER NOT NULL,
                    created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
                    deleted_at INTEGER, PRIMARY KEY (scope, key, kind));
                 INSERT INTO annotations VALUES ('item', 'i1', 'buyout', '{}', 3, 1, 2, NULL);",
            )
            .unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
        }
        // open_for migrates (meta table added, rows untouched) and stamps
        // the uuid the file is addressed by — the v1 filename convention
        // was its only binding, upgraded on this first open.
        let a = Annotations::open_for(&dir, "u-1").unwrap();
        assert_eq!(a.uuid(), Some("u-1"));
        let row = a.get("item", "i1", "buyout").unwrap().unwrap();
        assert_eq!(row.revision, 3);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_newer_schema_is_refused_not_guessed_at() {
        let dir =
            std::env::temp_dir().join(format!("acq-ann-v-{}-{}", std::process::id(), crate::now()));
        let path = annotations_path(&dir, "u-1");
        {
            let a = Annotations::open(&path).unwrap();
            a.conn.pragma_update(None, "user_version", 99).unwrap();
        }
        match Annotations::open(&path).err() {
            Some(AnnotationError::SchemaTooNew { found: 99, .. }) => {}
            other => panic!("expected SchemaTooNew, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_file_is_named_by_uuid() {
        let p = annotations_path(Path::new("/store/mock"), "0000-4000#odd");
        assert_eq!(p.file_name().unwrap(), "0000-4000_odd.annotations.db");
    }
}
