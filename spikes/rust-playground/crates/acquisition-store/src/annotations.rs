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
//!   id), `"character"` (key = GGG character id), `"tab"` (key =
//!   `<realm>/<id>`), `"substash"` (key = `<realm>/<parent>/<id>`),
//!   `"account"` (key = `""` for per-account singletons like the sync
//!   policy). The realm-bearing keys are rendered and parsed by one type,
//!   `acquisition_plan::price::PriceTarget` (C67), defined before the
//!   first tab-scoped row landed; the store carries them as text.
//! - `kind` is what the annotation says: `"buyout"`, `"note"`,
//!   `"sync-policy"`, …
//! - `value` is JSON; its shape is the kind's business, declared through
//!   [`IntentValue`] and enforced at the write door ([`Annotations::put`]).
//!
//! Backup is store-managed: [`Annotations::export`] writes a consistent
//! snapshot via `VACUUM INTO` — a raw file copy under WAL is not a backup.
//!
//! # Decisions as recorded
//!
//! The rulings are the decision registry — `decisions/plans.md` and
//! `decisions/pricing.md` for this area, `CONTEXT.md` for the cross-cutting
//! ones (`C<n>`); what follows is each entry's full text as recorded there,
//! moved here because the mechanism it describes is this module's. The
//! registry is current; this is the mechanism as decided, kept beside the
//! code that implements it.
//!
//! ## C35 — Annotations are the only irreplaceable local state.
//!
//! **Annotations are the only irreplaceable local state.** A separate per-account file named by the account uuid (identity decision in "Multi-account design"), keyed on stable GGG ids, written only through the store crate with integer-revision compare-and-swap; no fact-side event ever deletes intent — an annotation whose item is removed is kept and surfaceable as orphaned; export/backup is a store-managed consistent snapshot (`VACUUM INTO` / SQLite backup API — a raw file copy under WAL is not a backup). Rationale: facts are refetchable at the cost of requests; intent has no server to refetch from — the C++ legacy-buyout saga is the full price of getting this wrong. Decided 2026-08-31.
//!
//! ## C65 — Every intent write carries structured provenance
//!
//! **Every intent write carries structured provenance: the channel it came
//! through (`written_via`) and an optional claimed `actor`; the hash of the
//! plan that landed it (`applied_plan`, C71) joins when receipts exist.**
//! Stored on the row (annotations v3), returned on every read;
//! `written_via` required by the write API; `actor` is untrusted audit
//! metadata, never identity or authorization; origin detail lives on the
//! receipt (C78). v3 is the floor: a file below it is refused, never
//! migrated — every row on record has a writer. Ruled 2026-09-03;
//! `applied_plan` deferred 2026-09-04; the floor 2026-09-05.
//!
//! As built: [`Provenance`] is a required argument of [`Annotations::put`]
//! and [`Annotations::delete`] (a tombstone records who cleared the row);
//! `written_via` must be a non-empty single word. Files below v3 were
//! written only by development builds (the owner's held one policy row),
//! so rather than carry a migration that would have to invent a writer
//! for rows that had none, the open refuses them
//! ([`AnnotationError::SchemaTooOld`]) naming the file and the fix; a
//! future column is added by a stepwise `ALTER TABLE` in the same
//! IMMEDIATE transaction, the way the facts store migrates.
//!
//! ## C66 — Intent values are typed at the write API
//!
//! **Intent values are typed at the write API: a kind declares its schema
//! version and a strict parser, a value that does not parse under its
//! stamp never lands, and a current-schema value re-serializes to exactly
//! what was read.** The generic — version gate, unknown fields refused at
//! every depth, exact round-trip, then compare-and-swap — is factored out
//! of the sync policy's parser into the store crate over a per-kind trait;
//! each kind's shape stays its owner's; an older stored value upgrades in
//! memory, its raw JSON untouched. Ruled 2026-09-03.
//!
//! As built: [`IntentValue`] is the per-kind trait (the kind's name, the
//! version this build writes, its strict parse); [`check_value`] is the
//! generic, in this order: an integer `version` stamp, refused above the
//! kind's version *before* the shape is read (a newer value is reported as
//! such, never as a typo); the kind's own parse (each kind's wire structs
//! carry `deny_unknown_fields` at every depth); then, for a value stamped
//! with the current version only, the parsed value must re-serialize to
//! exactly what was read — the first differing path is named — which
//! closes the holes serde leaves open (`null` for an absent field, an
//! empty list for "none", an extra field beside a unit variant's tag). A
//! value stamped older is upgraded in memory by the kind's parse and
//! stored as written. [`Annotations::put`] runs the generic before the
//! compare-and-swap, so there is no untyped write door: the store crate
//! itself cannot land a value its kind refuses.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, ErrorCode, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::index::filename_safe;

/// The schema this build reads and writes. A file stamped newer is refused
/// (never auto-migrated: this is the one file that must not be damaged);
/// additions like the deferred event log bump this and migrate forward
/// from [`SCHEMA_FLOOR`]. v2 added `meta`, which carries the account uuid
/// *inside* the file so a copied or renamed database cannot silently pair
/// with another account's facts — the filename convention alone was
/// bypassable. v3 (2026-09-05, C65) added `written_via` and `actor` to
/// every row.
const SCHEMA_VERSION: i64 = 3;

/// The oldest schema this build opens. Files below it were written only
/// by development builds and are refused rather than migrated: a row
/// without provenance would need a writer invented for it (C65).
const SCHEMA_FLOOR: i64 = 3;

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
    written_via TEXT    NOT NULL,   -- the channel of the last write (C65)
    actor       TEXT,               -- the writer's claim, untrusted
    PRIMARY KEY (scope, key, kind)
);
CREATE TABLE IF NOT EXISTS meta (
    key    TEXT PRIMARY KEY,        -- 'account_uuid'
    value  TEXT NOT NULL
);
";

/// The `meta` key holding the owning account's uuid.
const META_UUID: &str = "account_uuid";

/// How long a writer waits for another connection's write lock before
/// giving up as [`AnnotationError::Busy`].
const BUSY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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
    /// The channel the last write came through (C65): `cli`, `mcp`, ….
    pub written_via: String,
    /// What the writer claimed about who was acting. Untrusted audit
    /// metadata: never identity, never authorization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
}

/// Who is writing, as the write API requires it (C65). Built once per
/// frontend surface, passed by reference to every write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provenance {
    /// The channel: a non-empty single word (`cli`, `mcp`, a test's name).
    pub written_via: String,
    /// An optional claim about the actor, recorded verbatim.
    pub actor: Option<String>,
}

impl Provenance {
    /// A write through `channel` with no actor claim.
    pub fn via(channel: &str) -> Provenance {
        Provenance {
            written_via: channel.into(),
            actor: None,
        }
    }

    /// The same channel, with an actor claim attached.
    pub fn as_actor(mut self, actor: &str) -> Provenance {
        self.actor = Some(actor.into());
        self
    }

    fn check(&self) -> Result<(), AnnotationError> {
        let via = &self.written_via;
        let detail = if via.is_empty() {
            "`written_via` is empty"
        } else if via.chars().any(char::is_whitespace) {
            "`written_via` must be a single word"
        } else {
            return Ok(());
        };
        Err(AnnotationError::Provenance {
            detail: detail.into(),
        })
    }
}

/// One kind of intent value (C66): the row's `kind`, the schema version
/// this build writes, and the strict parse that turns a stored JSON value
/// into the kind's own type. The generic checks — the version gate, the
/// exact round-trip, the compare-and-swap — are [`check_value`] and
/// [`Annotations::put`]; a kind supplies only its shape.
///
/// `parse` receives a value whose integer `version` is at most
/// [`IntentValue::VERSION`] and returns the in-memory form: for the
/// current version, exactly what was read; for an older one, the upgrade.
/// The wire structs behind it carry `deny_unknown_fields` at every depth,
/// so a typo is a structured error, never intent half-honored. `Serialize`
/// must produce the current wire shape, since a current-schema value is
/// held to re-serializing to exactly what was read.
pub trait IntentValue: Sized + Serialize {
    /// The `kind` column.
    const KIND: &'static str;
    /// The `version` stamp this build writes.
    const VERSION: i64;
    /// The kind's own strict parse; the detail names what was wrong.
    fn parse(value: &Value) -> Result<Self, String>;
}

/// Why a value is not a `K` (C66). Stable kinds so a frontend can render
/// "newer than this build" differently from "a typo".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueError {
    /// No integer `version` stamp.
    MissingVersion { kind: &'static str },
    /// Stamped newer than this build's parser for the kind.
    VersionUnsupported {
        kind: &'static str,
        found: i64,
        supported: i64,
    },
    /// The kind's parse refused it.
    Malformed { kind: &'static str, detail: String },
    /// Stamped with the current version, but the parsed value
    /// re-serializes to something else: `path` is the first difference
    /// (`/realms/pc/leagues/Standard/tabs`), `read` what the value said
    /// there, `canonical` what the kind writes there — `None` on either
    /// side is "absent" (`null` and absent are not the same value).
    NotCanonical {
        kind: &'static str,
        path: String,
        read: Option<Value>,
        canonical: Option<Value>,
    },
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueError::MissingVersion { kind } => {
                write!(f, "{kind}: missing integer `version`")
            }
            ValueError::VersionUnsupported {
                kind,
                found,
                supported,
            } => write!(
                f,
                "{kind} declares version {found}, newer than this build's v{supported}"
            ),
            ValueError::Malformed { kind, detail } => write!(f, "{kind}: {detail}"),
            ValueError::NotCanonical {
                kind,
                path,
                read,
                canonical,
            } => {
                let side = |v: &Option<Value>| match v {
                    Some(v) => v.to_string(),
                    None => "absent".into(),
                };
                write!(
                    f,
                    "{kind}: not in canonical form at {path}: read {}, the canonical form has {}",
                    side(read),
                    side(canonical)
                )
            }
        }
    }
}

impl std::error::Error for ValueError {}

/// The generic half of every typed write and read (C66): the version
/// gate, the kind's strict parse, and — for a current-schema value — the
/// exact round-trip. Pure; the store calls it before the compare-and-swap.
pub fn check_value<K: IntentValue>(value: &Value) -> Result<K, ValueError> {
    let found = value
        .get("version")
        .and_then(Value::as_i64)
        .ok_or(ValueError::MissingVersion { kind: K::KIND })?;
    if found > K::VERSION {
        return Err(ValueError::VersionUnsupported {
            kind: K::KIND,
            found,
            supported: K::VERSION,
        });
    }
    let parsed = K::parse(value).map_err(|detail| ValueError::Malformed {
        kind: K::KIND,
        detail,
    })?;
    if found == K::VERSION {
        let canonical = serde_json::to_value(&parsed).map_err(|e| ValueError::Malformed {
            kind: K::KIND,
            detail: format!("does not serialize: {e}"),
        })?;
        if let Some((path, read, canonical)) = first_difference(value, &canonical, "") {
            return Err(ValueError::NotCanonical {
                kind: K::KIND,
                path,
                read,
                canonical,
            });
        }
    }
    Ok(parsed)
}

/// The first JSON path at which `read` and `canonical` differ, with both
/// sides there (`None`: absent); `None` overall when equal. Object keys
/// are compared as sets (an object's key order is not a difference),
/// arrays element by element.
type Difference = (String, Option<Value>, Option<Value>);

fn first_difference(read: &Value, canonical: &Value, path: &str) -> Option<Difference> {
    match (read, canonical) {
        (Value::Object(a), Value::Object(b)) => {
            let mut keys: Vec<&String> = a.keys().chain(b.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let sub = format!("{path}/{k}");
                match (a.get(k), b.get(k)) {
                    (Some(x), Some(y)) => {
                        if let Some(d) = first_difference(x, y, &sub) {
                            return Some(d);
                        }
                    }
                    (Some(x), None) => return Some((sub, Some(x.clone()), None)),
                    (None, Some(y)) => return Some((sub, None, Some(y.clone()))),
                    (None, None) => {}
                }
            }
            None
        }
        (Value::Array(a), Value::Array(b)) => {
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                if let Some(d) = first_difference(x, y, &format!("{path}/{i}")) {
                    return Some(d);
                }
            }
            (a.len() != b.len()).then(|| {
                let i = a.len().min(b.len());
                (format!("{path}/{i}"), a.get(i).cloned(), b.get(i).cloned())
            })
        }
        (a, b) => (a != b).then(|| {
            (
                if path.is_empty() {
                    "/".into()
                } else {
                    path.into()
                },
                Some(a.clone()),
                Some(b.clone()),
            )
        }),
    }
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
    /// The file predates the schema floor (a development build wrote it).
    /// Refused rather than migrated: its rows have no writer to record.
    SchemaTooOld {
        found: i64,
        floor: i64,
        path: PathBuf,
    },
    /// The file carries another account's uuid: a copy or rename cannot
    /// silently pair one account's intent with another account's facts.
    WrongAccount {
        stored: String,
        requested: String,
    },
    /// The value is not one its kind accepts (C66); nothing landed.
    Invalid(ValueError),
    /// The write named no acceptable channel (C65); nothing landed.
    Provenance {
        detail: String,
    },
    /// Another connection held the write lock past the busy timeout. The
    /// row is untouched and unread: retry later — distinct from
    /// [`AnnotationError::Conflict`], whose remedy is re-read and retry.
    Busy,
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
            AnnotationError::SchemaTooOld { found, floor, path } => write!(
                f,
                "annotation file {} uses schema v{found}, below this build's floor v{floor} \
                 (a development build wrote it); delete it and set the policy again",
                path.display()
            ),
            AnnotationError::WrongAccount { stored, requested } => write!(
                f,
                "annotation file belongs to account uuid {stored}, not {requested}"
            ),
            AnnotationError::Invalid(e) => write!(f, "{e}"),
            AnnotationError::Provenance { detail } => write!(f, "annotation write: {detail}"),
            AnnotationError::Busy => write!(
                f,
                "annotation file is busy: another writer held it past {} s (retry later)",
                BUSY_TIMEOUT.as_secs()
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
        match &e {
            rusqlite::Error::SqliteFailure(f, _)
                if matches!(f.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) =>
            {
                AnnotationError::Busy
            }
            _ => AnnotationError::Db(e),
        }
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

impl From<ValueError> for AnnotationError {
    fn from(e: ValueError) -> Self {
        AnnotationError::Invalid(e)
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
    /// [`crate::Store::refresh_snapshot`] refuses.
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
        // The version gate comes before any pragma: switching the journal
        // mode rewrites the file header, and a file this build refuses is
        // left exactly as it was found.
        let found: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if found > SCHEMA_VERSION {
            return Err(AnnotationError::SchemaTooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        if found != 0 && found < SCHEMA_FLOOR {
            return Err(AnnotationError::SchemaTooOld {
                found,
                floor: SCHEMA_FLOOR,
                path,
            });
        }
        // WAL like the fact store: one writer at a time, any number of readers.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // FULL, not the fact store's NORMAL: under WAL, NORMAL keeps the
        // file consistent but lets the last commits before a power loss
        // roll back, and this is the one file with no server to refetch
        // from (C35). Writes here are human-paced and a batch is one
        // commit, so the fsync per commit costs nothing that matters.
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.busy_timeout(BUSY_TIMEOUT)?;
        // Creation (and, from the floor up, migration) serializes under
        // one immediate transaction so two processes opening the same file
        // cannot interleave them. A later version adds its columns here by
        // a stepwise `ALTER TABLE` per version, never by rewriting a row.
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Re-read under the lock: two processes creating one file serialize
        // here, and the loser sees the winner's stamp.
        let found: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        match found {
            0 => {
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

    /// Write one annotation of kind `K` under compare-and-swap (C35),
    /// typed at the door (C66) and stamped with who wrote it (C65). The
    /// value is checked whole before the transaction opens — the version
    /// gate, `K`'s strict parse, the exact round-trip for a current-schema
    /// value — and a value that fails never lands
    /// ([`AnnotationError::Invalid`]); what lands is `value` as given,
    /// never a re-serialization, so an older stamp stays stored as written.
    ///
    /// `expected_revision` is what the caller last read: `None` creates
    /// (fails if the annotation currently exists), `Some(r)` updates (fails
    /// unless the current revision is exactly `r`). Returns the row as
    /// written; a mismatch is [`AnnotationError::Conflict`] carrying the
    /// current row. Creating over a tombstone continues its revision
    /// sequence — revisions are monotonic for the life of the file, never
    /// reset by delete/recreate, so a stale writer always conflicts, and a
    /// `clear` then `set` on one target works without the caller knowing
    /// the tombstone is there.
    pub fn put<K: IntentValue>(
        &mut self,
        scope: &str,
        key: &str,
        value: &Value,
        expected_revision: Option<i64>,
        provenance: &Provenance,
    ) -> Result<AnnotationRow, AnnotationError> {
        provenance.check()?;
        check_value::<K>(value)?;
        let kind = K::KIND;
        let now = crate::now();
        // BEGIN IMMEDIATE: the write lock is taken up front, so two
        // connections racing the same row serialize here (bounded by the
        // busy timeout) and the loser reads the winner's committed state —
        // a real Conflict, not a snapshot-upgrade error at commit.
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let current = row_of(&tx, scope, key, kind)?;
        let stamped = |row: AnnotationRow| AnnotationRow {
            value: value.clone(),
            updated_at: now,
            written_via: provenance.written_via.clone(),
            actor: provenance.actor.clone(),
            ..row
        };
        let row = match (expected_revision, current) {
            (None, None) => stamped(AnnotationRow {
                scope: scope.into(),
                key: key.into(),
                kind: kind.into(),
                value: Value::Null,
                revision: 1,
                created_at: now,
                updated_at: now,
                written_via: String::new(),
                actor: None,
            }),
            // Recreation after a delete: the tombstone's revision carries on.
            (None, Some((tombstone, true))) => stamped(AnnotationRow {
                revision: tombstone.revision + 1,
                created_at: now,
                ..tombstone
            }),
            (Some(expected), Some((current, false))) if current.revision == expected => {
                stamped(AnnotationRow {
                    revision: expected + 1,
                    ..current
                })
            }
            (_, current) => return Err(conflict(current)),
        };
        tx.execute(
            "INSERT OR REPLACE INTO annotations
                (scope, key, kind, value, revision, created_at, updated_at, deleted_at, written_via, actor)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, ?8, ?9)",
            params![
                row.scope,
                row.key,
                row.kind,
                row.value.to_string(),
                row.revision,
                row.created_at,
                row.updated_at,
                row.written_via,
                row.actor
            ],
        )?;
        tx.commit()?;
        Ok(row)
    }

    /// Remove one annotation, under the same compare-and-swap as `put`.
    /// This is a frontend expressing intent — the fact side has no delete
    /// path at all. The row becomes a tombstone (invisible to `get`/`list`)
    /// whose revision keeps counting, so no later writer can slip a stale
    /// value past the delete; the tombstone records who cleared it (C65).
    pub fn delete(
        &mut self,
        scope: &str,
        key: &str,
        kind: &str,
        expected_revision: i64,
        provenance: &Provenance,
    ) -> Result<(), AnnotationError> {
        provenance.check()?;
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        match row_of(&tx, scope, key, kind)? {
            Some((current, false)) if current.revision == expected_revision => {
                tx.execute(
                    "UPDATE annotations
                        SET deleted_at = ?4, updated_at = ?4, revision = revision + 1,
                            written_via = ?5, actor = ?6
                      WHERE scope = ?1 AND key = ?2 AND kind = ?3",
                    params![
                        scope,
                        key,
                        kind,
                        crate::now(),
                        provenance.written_via,
                        provenance.actor
                    ],
                )?;
            }
            current => return Err(conflict(current)),
        }
        tx.commit()?;
        Ok(())
    }

    /// The stored row, raw: the value as written, with its provenance.
    pub fn get(
        &self,
        scope: &str,
        key: &str,
        kind: &str,
    ) -> Result<Option<AnnotationRow>, AnnotationError> {
        Ok(row_of(&self.conn, scope, key, kind)?
            .and_then(|(row, deleted)| (!deleted).then_some(row)))
    }

    /// The stored row of kind `K` and its typed value (C66): the same
    /// generic as the write door, so a row a newer build wrote is reported
    /// as such and an older stamp upgrades in memory. A stored row that
    /// fails is an error, never "no such annotation".
    pub fn get_as<K: IntentValue>(
        &self,
        scope: &str,
        key: &str,
    ) -> Result<Option<(AnnotationRow, K)>, AnnotationError> {
        self.get(scope, key, K::KIND)?
            .map(|row| {
                check_value::<K>(&row.value)
                    .map(|v| (row, v))
                    .map_err(Into::into)
            })
            .transpose()
    }

    /// Every live annotation (tombstones excluded), optionally restricted
    /// to one scope and/or one kind, ordered by (scope, key, kind). A
    /// pricing read wants one kind across four scopes; at 10k rows the
    /// scan is 35 ms, so a filter is needed and an index is not.
    pub fn list(
        &self,
        scope: Option<&str>,
        kind: Option<&str>,
    ) -> Result<Vec<AnnotationRow>, AnnotationError> {
        let mut stmt = self.conn.prepare(
            "SELECT scope, key, kind, value, revision, created_at, updated_at, written_via, actor
               FROM annotations
              WHERE deleted_at IS NULL AND (?1 IS NULL OR scope = ?1) AND (?2 IS NULL OR kind = ?2)
              ORDER BY scope, key, kind",
        )?;
        let rows: Vec<RawRow> = stmt
            .query_map(params![scope, kind], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                ))
            })?
            .collect::<Result<_, _>>()?;
        rows.into_iter().map(row_from_raw).collect()
    }

    /// Store-managed backup (C35): a consistent snapshot of this file at
    /// `dest`, via SQLite's `VACUUM INTO`, published atomically. `VACUUM
    /// INTO` writes `dest` directly and never fsyncs it, so an interrupted
    /// export would leave a partial file that both looks like a backup and
    /// blocks every retry. The copy is therefore written to
    /// `<dest>.partial` (a stale partial from an earlier interruption is
    /// replaced), checked with `quick_check`, fsynced, then linked into
    /// place; the partial is removed either way. Fails if `dest` already
    /// exists, before anything is written — a backup never overwrites
    /// another backup silently.
    pub fn export(&self, dest: &Path) -> Result<(), AnnotationError> {
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir)?;
        }
        if dest.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "{} exists; a backup never overwrites another backup",
                    dest.display()
                ),
            )
            .into());
        }
        let mut partial = dest.as_os_str().to_owned();
        partial.push(".partial");
        let partial = PathBuf::from(partial);
        let _ = std::fs::remove_file(&partial);
        let published = self.export_to(&partial).and_then(|()| {
            std::fs::hard_link(&partial, dest)
                .or_else(|_| std::fs::rename(&partial, dest))
                .map_err(AnnotationError::Io)
        });
        let _ = std::fs::remove_file(&partial);
        published
    }

    /// The copy itself: vacuumed into `partial`, integrity-checked through
    /// a fresh connection, and fsynced.
    fn export_to(&self, partial: &Path) -> Result<(), AnnotationError> {
        self.conn
            .execute("VACUUM INTO ?1", [partial.to_string_lossy()])?;
        let verdict: String =
            Connection::open(partial)?.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
        if verdict != "ok" {
            return Err(std::io::Error::other(format!(
                "backup {} failed its integrity check: {verdict}",
                partial.display()
            ))
            .into());
        }
        std::fs::File::open(partial)?.sync_all()?;
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

/// The columns of one row as SQLite hands them back, before the JSON is
/// parsed: scope, key, kind, value text, revision, created, updated,
/// written_via, actor.
type RawRow = (
    String,
    String,
    String,
    String,
    i64,
    i64,
    i64,
    String,
    Option<String>,
);

fn row_from_raw(raw: RawRow) -> Result<AnnotationRow, AnnotationError> {
    let (scope, key, kind, value, revision, created_at, updated_at, written_via, actor) = raw;
    Ok(AnnotationRow {
        scope,
        key,
        kind,
        value: serde_json::from_str(&value)?,
        revision,
        created_at,
        updated_at,
        written_via,
        actor,
    })
}

/// The stored row, tombstones included; the bool is "deleted".
fn row_of(
    conn: &Connection,
    scope: &str,
    key: &str,
    kind: &str,
) -> Result<Option<(AnnotationRow, bool)>, AnnotationError> {
    // value, revision, created, updated, deleted, written_via, actor
    type Stored = (String, i64, i64, i64, Option<i64>, String, Option<String>);
    let found: Option<Stored> = conn
        .query_row(
            "SELECT value, revision, created_at, updated_at, deleted_at, written_via, actor
               FROM annotations
              WHERE scope = ?1 AND key = ?2 AND kind = ?3",
            params![scope, key, kind],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            },
        )
        .optional()?;
    match found {
        None => Ok(None),
        Some((value, revision, created_at, updated_at, deleted_at, written_via, actor)) => {
            Ok(Some((
                AnnotationRow {
                    scope: scope.into(),
                    key: key.into(),
                    kind: kind.into(),
                    value: serde_json::from_str(&value)?,
                    revision,
                    created_at,
                    updated_at,
                    written_via,
                    actor,
                },
                deleted_at.is_some(),
            )))
        }
    }
}

/// Kinds for this crate's tests: a value of any shape under a named kind,
/// stamped `version: 1`. The store crate owns no real kind (each kind's
/// shape is its owner's, C66), so its tests declare these.
#[cfg(test)]
pub(crate) mod test_kinds {
    use super::IntentValue;
    use serde::Serialize;
    use serde_json::Value;

    macro_rules! loose_kind {
        ($name:ident, $kind:literal) => {
            #[derive(Debug, Clone, PartialEq, Serialize)]
            #[serde(transparent)]
            pub(crate) struct $name(pub Value);

            impl IntentValue for $name {
                const KIND: &'static str = $kind;
                const VERSION: i64 = 1;
                fn parse(value: &Value) -> Result<Self, String> {
                    Ok($name(value.clone()))
                }
            }
        };
    }

    loose_kind!(Buyout, "buyout");
    loose_kind!(Note, "note");
    loose_kind!(Policy, "sync-policy");

    /// The provenance every test write carries.
    pub(crate) fn via_test() -> super::Provenance {
        super::Provenance::via("test")
    }
}

#[cfg(test)]
mod tests {
    use super::test_kinds::{Buyout, Note, Policy, via_test};
    use super::*;
    use serde_json::json;

    fn conflict_revision(e: AnnotationError) -> Option<i64> {
        match e {
            AnnotationError::Conflict { current } => current.map(|r| r.revision),
            other => panic!("expected a conflict, got {other}"),
        }
    }

    fn price(p: &str) -> Value {
        json!({ "version": 1, "price": p })
    }

    /// C35 — integer-revision compare-and-swap: create requires absence, update the exact revision, delete the same.
    #[test]
    fn compare_and_swap_creates_updates_and_conflicts() {
        let mut a = Annotations::open_memory().unwrap();
        let via = via_test();
        // Create requires "does not exist yet".
        let row = a
            .put::<Buyout>("item", "i1", &price("1 divine"), None, &via)
            .unwrap();
        assert_eq!(row.revision, 1);
        let err = a
            .put::<Buyout>("item", "i1", &price("2 divine"), None, &via)
            .unwrap_err();
        assert_eq!(conflict_revision(err), Some(1));
        // Update requires the exact current revision.
        let row = a
            .put::<Buyout>("item", "i1", &price("2 divine"), Some(1), &via)
            .unwrap();
        assert_eq!(row.revision, 2);
        assert!(row.created_at <= row.updated_at);
        let stale = a
            .put::<Buyout>("item", "i1", &price("3c"), Some(1), &via)
            .unwrap_err();
        assert_eq!(conflict_revision(stale), Some(2));
        // The stored value is the last accepted write.
        let got = a.get("item", "i1", "buyout").unwrap().unwrap();
        assert_eq!(got.value, price("2 divine"));
        // The same key under another kind is its own row.
        a.put::<Note>(
            "item",
            "i1",
            &json!({ "version": 1, "text": "keep" }),
            None,
            &via,
        )
        .unwrap();
        assert_eq!(a.list(Some("item"), None).unwrap().len(), 2);
        // Delete is CAS too; a stale delete conflicts and changes nothing.
        let err = a.delete("item", "i1", "buyout", 1, &via).unwrap_err();
        assert_eq!(conflict_revision(err), Some(2));
        a.delete("item", "i1", "buyout", 2, &via).unwrap();
        assert!(a.get("item", "i1", "buyout").unwrap().is_none());
        // Updating what is gone reports "gone", not "wrong revision".
        let err = a
            .put::<Buyout>("item", "i1", &price("x"), Some(2), &via)
            .unwrap_err();
        assert_eq!(conflict_revision(err), None);
    }

    /// C35 — delete tombstones; the revision sequence survives delete/recreate, so a stale writer always conflicts.
    #[test]
    fn delete_and_recreate_never_reset_the_revision() {
        let mut a = Annotations::open_memory().unwrap();
        let via = via_test();
        // A stale reader remembers revision 1...
        let stale = a
            .put::<Buyout>("item", "i1", &price("1c"), None, &via)
            .unwrap()
            .revision;
        // ...while someone else deletes and recreates the annotation.
        a.delete("item", "i1", "buyout", 1, &via).unwrap();
        let recreated = a
            .put::<Buyout>("item", "i1", &price("5 divine"), None, &via)
            .unwrap();
        assert!(
            recreated.revision > stale,
            "recreation continues the sequence ({} > {stale})",
            recreated.revision
        );
        // The stale writer's update conflicts instead of silently landing
        // its pre-delete value over the recreated one (the ABA hole).
        let err = a
            .put::<Buyout>("item", "i1", &price("1c"), Some(stale), &via)
            .unwrap_err();
        assert_eq!(conflict_revision(err), Some(recreated.revision));
        assert_eq!(
            a.get("item", "i1", "buyout").unwrap().unwrap().value,
            price("5 divine")
        );
        // A stale delete cannot remove the recreated value either.
        assert!(a.delete("item", "i1", "buyout", stale, &via).is_err());
    }

    /// C35 — two writers on one file serialize under BEGIN IMMEDIATE: one wins, the other gets a Conflict, never a clobber.
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
        let via = via_test();
        // Both connections race the same create. BEGIN IMMEDIATE
        // serializes them, so exactly one wins and the loser gets the
        // documented Conflict carrying the winner's row — never a raw
        // SQLite busy/snapshot error, never two revision-1 writes.
        let results = std::thread::scope(|scope| {
            let ta = scope.spawn(|| {
                a.put::<Buyout>(
                    "item",
                    "i1",
                    &json!({"version": 1, "from": "a"}),
                    None,
                    &via,
                )
            });
            let tb = scope.spawn(|| {
                b.put::<Buyout>(
                    "item",
                    "i1",
                    &json!({"version": 1, "from": "b"}),
                    None,
                    &via,
                )
            });
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

    /// A writer that cannot get the lock inside the busy timeout is told
    /// to retry later — its own kind, not a bare SQLite error — because
    /// the remedy differs from a Conflict's "re-read and retry".
    #[test]
    fn a_writer_held_past_the_busy_timeout_is_busy_not_db() {
        let dir = std::env::temp_dir().join(format!(
            "acq-ann-busy-{}-{}",
            std::process::id(),
            crate::now()
        ));
        let path = annotations_path(&dir, "u-busy");
        let mut a = Annotations::open(&path).unwrap();
        let b = Annotations::open(&path).unwrap();
        a.conn
            .busy_timeout(std::time::Duration::from_millis(50))
            .unwrap();
        // `b` holds the write lock in an open transaction the whole time.
        b.conn.execute_batch("BEGIN IMMEDIATE").unwrap();
        let err = a
            .put::<Buyout>("item", "i1", &price("1c"), None, &via_test())
            .unwrap_err();
        assert!(matches!(err, AnnotationError::Busy), "{err}");
        assert!(err.to_string().contains("retry later"), "{err}");
        b.conn.execute_batch("ROLLBACK").unwrap();
        // The row is untouched: the same create succeeds once the lock is free.
        assert_eq!(
            a.put::<Buyout>("item", "i1", &price("1c"), None, &via_test())
                .unwrap()
                .revision,
            1
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn account_singletons_use_the_empty_key() {
        let mut a = Annotations::open_memory().unwrap();
        let policy = json!({ "version": 1, "leagues": ["Standard"], "deep": false });
        a.put::<Policy>("account", "", &policy, None, &via_test())
            .unwrap();
        let row = a.get("account", "", "sync-policy").unwrap().unwrap();
        assert_eq!(row.value, policy);
    }

    /// `list` filters by scope, by kind, or both: a pricing read wants one
    /// kind across every scope.
    #[test]
    fn list_filters_by_scope_and_by_kind() {
        let mut a = Annotations::open_memory().unwrap();
        let via = via_test();
        a.put::<Buyout>("item", "i1", &price("1c"), None, &via)
            .unwrap();
        a.put::<Buyout>("tab", "pc/t1", &price("2c"), None, &via)
            .unwrap();
        a.put::<Note>(
            "item",
            "i1",
            &json!({ "version": 1, "text": "n" }),
            None,
            &via,
        )
        .unwrap();
        let keys = |rows: Vec<AnnotationRow>| -> Vec<String> {
            rows.into_iter()
                .map(|r| format!("{}/{}/{}", r.scope, r.key, r.kind))
                .collect()
        };
        assert_eq!(
            keys(a.list(None, None).unwrap()),
            ["item/i1/buyout", "item/i1/note", "tab/pc/t1/buyout"]
        );
        assert_eq!(
            keys(a.list(None, Some("buyout")).unwrap()),
            ["item/i1/buyout", "tab/pc/t1/buyout"]
        );
        assert_eq!(
            keys(a.list(Some("item"), Some("buyout")).unwrap()),
            ["item/i1/buyout"]
        );
        assert!(a.list(Some("account"), Some("buyout")).unwrap().is_empty());
    }

    /// C65 — every write carries who wrote it; a delete stamps the
    /// tombstone; an empty or whitespace channel is refused before
    /// anything lands.
    #[test]
    fn c65_every_write_carries_its_provenance() {
        let mut a = Annotations::open_memory().unwrap();
        let cli = Provenance::via("cli").as_actor("tom");
        let row = a
            .put::<Buyout>("item", "i1", &price("1c"), None, &cli)
            .unwrap();
        assert_eq!(
            (row.written_via.as_str(), row.actor.as_deref()),
            ("cli", Some("tom"))
        );
        let read = a.get("item", "i1", "buyout").unwrap().unwrap();
        assert_eq!(read, row);
        // The next writer's stamp replaces it, actor included.
        let mcp = Provenance::via("mcp");
        let row = a
            .put::<Buyout>("item", "i1", &price("2c"), Some(1), &mcp)
            .unwrap();
        assert_eq!((row.written_via.as_str(), row.actor), ("mcp", None));
        // The tombstone records who cleared the row.
        a.delete("item", "i1", "buyout", 2, &cli).unwrap();
        let (tomb, deleted) = row_of(&a.conn, "item", "i1", "buyout").unwrap().unwrap();
        assert!(deleted);
        assert_eq!(tomb.written_via, "cli");
        assert_eq!(tomb.actor.as_deref(), Some("tom"));
        // Refused channels: nothing lands, nothing is deleted.
        for bad in ["", "two words"] {
            let err = a
                .put::<Buyout>("item", "i2", &price("1c"), None, &Provenance::via(bad))
                .unwrap_err();
            assert!(
                matches!(err, AnnotationError::Provenance { .. }),
                "{bad:?}: {err}"
            );
            assert!(a.get("item", "i2", "buyout").unwrap().is_none());
        }
        a.put::<Buyout>("item", "i2", &price("1c"), None, &mcp)
            .unwrap();
        let err = a
            .delete("item", "i2", "buyout", 1, &Provenance::via(""))
            .unwrap_err();
        assert!(matches!(err, AnnotationError::Provenance { .. }), "{err}");
        assert!(a.get("item", "i2", "buyout").unwrap().is_some());
        // The row's JSON carries the stamp; a missing actor is omitted, not null.
        let json = serde_json::to_value(a.get("item", "i2", "buyout").unwrap().unwrap()).unwrap();
        assert_eq!(json["written_via"], "mcp");
        assert!(json.get("actor").is_none());
    }

    /// C66 — the generic at the write door: no version, a newer version,
    /// the kind's own refusal, and a current-schema value that does not
    /// re-serialize to what was read all fail before anything lands.
    #[test]
    fn c66_the_write_door_is_typed_and_a_value_that_fails_never_lands() {
        /// A kind with one field, `n`, that must be an integer; its
        /// canonical form has nothing else.
        #[derive(Debug, Serialize)]
        struct Strict {
            version: i64,
            n: i64,
        }
        impl IntentValue for Strict {
            const KIND: &'static str = "strict";
            const VERSION: i64 = 2;
            fn parse(value: &Value) -> Result<Self, String> {
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Wire {
                    version: i64,
                    n: i64,
                    // v2 only; v1 read `m` and upgrades.
                    #[serde(default)]
                    m: Option<i64>,
                }
                let w: Wire = serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
                match (w.version, w.m) {
                    (1, Some(m)) => Ok(Strict { version: 2, n: m }),
                    (1, None) => Err("v1 needs `m`".into()),
                    (2, None) => Ok(Strict { version: 2, n: w.n }),
                    (2, Some(_)) => Err("v2 has no `m`".into()),
                    (v, _) => Err(format!("version {v}")),
                }
            }
        }
        let mut a = Annotations::open_memory().unwrap();
        let via = via_test();
        let refused = |a: &mut Annotations, v: Value| {
            let err = a.put::<Strict>("item", "i1", &v, None, &via).unwrap_err();
            assert!(
                a.get("item", "i1", "strict").unwrap().is_none(),
                "{v} landed"
            );
            match err {
                AnnotationError::Invalid(e) => e,
                other => panic!("{v}: expected Invalid, got {other}"),
            }
        };
        assert_eq!(
            refused(&mut a, json!({ "n": 1 })),
            ValueError::MissingVersion { kind: "strict" }
        );
        // Newer is reported as newer, before the shape is read.
        assert_eq!(
            refused(&mut a, json!({ "version": 3, "n": 1, "future": true })),
            ValueError::VersionUnsupported {
                kind: "strict",
                found: 3,
                supported: 2
            }
        );
        // The kind's own strictness: an unknown field.
        assert!(matches!(
            refused(&mut a, json!({ "version": 2, "n": 1, "typo": 1 })),
            ValueError::Malformed { detail, .. } if detail.contains("typo")
        ));
        // Current schema, parses, but the canonical form differs
        // (`m: null` is read as absent and written as nothing): refused
        // naming the path and both sides — null is not absent.
        let err = refused(&mut a, json!({ "version": 2, "n": 1, "m": null }));
        assert_eq!(
            err,
            ValueError::NotCanonical {
                kind: "strict",
                path: "/m".into(),
                read: Some(Value::Null),
                canonical: None,
            }
        );
        assert_eq!(
            err.to_string(),
            "strict: not in canonical form at /m: read null, the canonical form has absent"
        );
        // An older stamp upgrades in memory and lands as written.
        let v1 = json!({ "version": 1, "n": 0, "m": 7 });
        a.put::<Strict>("item", "i1", &v1, None, &via).unwrap();
        let (row, typed) = a.get_as::<Strict>("item", "i1").unwrap().unwrap();
        assert_eq!(row.value, v1, "stored as written");
        assert_eq!((typed.version, typed.n), (2, 7), "upgraded in memory");
        // A current-schema value lands and reads back typed.
        let v2 = json!({ "version": 2, "n": 9 });
        a.put::<Strict>("item", "i1", &v2, Some(1), &via).unwrap();
        let (_, typed) = a.get_as::<Strict>("item", "i1").unwrap().unwrap();
        assert_eq!(typed.n, 9);
        // A stored row a newer build wrote is an error on read, never "none".
        a.conn
            .execute(
                "UPDATE annotations SET value = ?1 WHERE kind = 'strict'",
                [json!({ "version": 3, "n": 1 }).to_string()],
            )
            .unwrap();
        assert!(matches!(
            a.get_as::<Strict>("item", "i1").unwrap_err(),
            AnnotationError::Invalid(ValueError::VersionUnsupported { found: 3, .. })
        ));
        assert!(a.get_as::<Note>("item", "nope").unwrap().is_none());
    }

    /// The difference finder names the first path that differs, both
    /// sides, and treats key order as no difference.
    #[test]
    fn the_first_difference_names_the_path_and_both_sides() {
        let same = json!({ "b": [1, { "c": 2 }], "a": 1 });
        let reordered = json!({ "a": 1, "b": [1, { "c": 2 }] });
        assert_eq!(first_difference(&same, &reordered, ""), None);
        assert_eq!(
            first_difference(
                &json!({ "a": { "b": [1, 2] } }),
                &json!({ "a": { "b": [1, 3] } }),
                ""
            ),
            Some(("/a/b/1".into(), Some(json!(2)), Some(json!(3))))
        );
        assert_eq!(
            first_difference(&json!({ "a": [1] }), &json!({ "a": [1, 2] }), ""),
            Some(("/a/1".into(), None, Some(json!(2))))
        );
        assert_eq!(
            first_difference(&json!({ "a": 1, "x": null }), &json!({ "a": 1 }), ""),
            Some(("/x".into(), Some(Value::Null), None))
        );
        assert_eq!(
            first_difference(&json!(1), &json!("1"), ""),
            Some(("/".into(), Some(json!(1)), Some(json!("1"))))
        );
    }

    /// C35 — backup is a store-managed consistent snapshot, never a raw file copy; it never overwrites.
    #[test]
    fn export_is_a_consistent_snapshot_and_never_overwrites() {
        let dir =
            std::env::temp_dir().join(format!("acq-ann-{}-{}", std::process::id(), crate::now()));
        let uuid = "00000000-0000-4000-8000-000000000001";
        let mut a = Annotations::open(&annotations_path(&dir, uuid)).unwrap();
        a.put::<Buyout>("item", "i1", &price("1c"), None, &via_test())
            .unwrap();
        let backup = dir.join("backup.db");
        // A partial left by an interrupted earlier export is replaced,
        // never published: after the export only the backup exists.
        let partial = dir.join("backup.db.partial");
        std::fs::write(&partial, b"garbage from an interrupted export").unwrap();
        a.export(&backup).unwrap();
        assert!(!partial.exists(), "the partial must not outlive the export");
        // The snapshot is a complete, standalone annotation file.
        let restored = Annotations::open(&backup).unwrap();
        assert_eq!(restored.list(None, None).unwrap().len(), 1);
        // A second export to the same path is refused, not an overwrite —
        // and refused before anything is written.
        assert!(a.export(&backup).is_err());
        assert!(!partial.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C35 — the intent file is fully synchronous: a commit that returned
    /// survives a power loss, not only a process crash (the fact store's
    /// NORMAL is fine for refetchable facts, not here).
    #[test]
    fn c35_the_intent_file_is_fully_synchronous() {
        let dir = std::env::temp_dir().join(format!(
            "acq-ann-sync-{}-{}",
            std::process::id(),
            crate::now()
        ));
        let a = Annotations::open_for(&dir, "u-1").unwrap();
        let sync: i64 = a
            .conn
            .query_row("PRAGMA synchronous", [], |r| r.get(0))
            .unwrap();
        assert_eq!(sync, 2, "PRAGMA synchronous must be FULL (2)");
        let mode: String = a
            .conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode, "wal");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C35 — the file is bound to the account uuid inside it; a copy under another account's name is refused.
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

    /// C65 — v3 is the floor: a file a development build wrote below it
    /// is refused naming the file and the fix, never migrated with a
    /// writer invented for its rows, and never touched.
    #[test]
    fn c65_a_file_below_the_floor_is_refused_never_migrated() {
        let dir = std::env::temp_dir().join(format!(
            "acq-ann-floor-{}-{}",
            std::process::id(),
            crate::now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = annotations_path(&dir, "u-1");
        let v2_table = "CREATE TABLE annotations (
                scope TEXT NOT NULL, key TEXT NOT NULL, kind TEXT NOT NULL,
                value TEXT NOT NULL, revision INTEGER NOT NULL,
                created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL,
                deleted_at INTEGER, PRIMARY KEY (scope, key, kind));
            CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta VALUES ('account_uuid', 'u-1');
            INSERT INTO annotations VALUES ('account', '', 'sync-policy', '{\"version\":1}', 9, 1, 2, NULL);";
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(v2_table).unwrap();
            conn.pragma_update(None, "user_version", 2).unwrap();
        }
        let before = std::fs::read(&path).unwrap();
        match Annotations::open_for(&dir, "u-1").err() {
            Some(AnnotationError::SchemaTooOld {
                found: 2,
                floor: 3,
                path: p,
            }) => assert_eq!(p, path),
            other => panic!("expected SchemaTooOld, got {other:?}"),
        }
        let err = match Annotations::open(&path) {
            Err(e) => e.to_string(),
            Ok(_) => panic!("a v2 file opened"),
        };
        assert!(
            err.contains("v2") && err.contains("floor v3") && err.contains("delete it"),
            "{err}"
        );
        assert_eq!(
            std::fs::read(&path).unwrap(),
            before,
            "refused, not touched"
        );
        // A fresh file is created at the current version outright.
        let a = Annotations::open_for(&dir, "u-2").unwrap();
        let v: i64 = a
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, SCHEMA_VERSION);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// C35 — a file a newer build wrote is refused, never guessed at.
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

    /// C35 — named by the account uuid, never the username.
    #[test]
    fn the_file_is_named_by_uuid() {
        let p = annotations_path(Path::new("/store/mock"), "0000-4000#odd");
        assert_eq!(p.file_name().unwrap(), "0000-4000_odd.annotations.db");
    }
}
