//! The shared store: one SQLite file per account (under one directory per
//! provider), written by the daemon as API responses land and read directly
//! by every frontend (CLI, GUI, MCP). `index` is the non-secret list of
//! accounts that names those files.
//!
//! The daemon's whole contract is [`Store::record`]: endpoint, params, status,
//! body. It never looks inside a body. Inside this crate, a body is kept
//! verbatim except at the item seams — every array of items (a tab's
//! `items`, a character's `inventory`/`equipment`/`jewels`/`rucksack`, and
//! each item's `socketedItems`) is lifted out into the `items` table, one
//! row per GGG item id. `responses` + `items` is the response, exactly,
//! split at those seams; `items` is the only place to look for an item.
//!
//! Every derived column (`name`, `type_line`, …) comes from the row's own
//! `json`, so a wrong extraction is repaired by re-extracting, never by
//! refetching.
//!
//! Facts are one of four layers (CONTEXT.md, 2026-08-31); `annotations` is
//! the intent layer, the only irreplaceable local state.

// The lint ratchet (CONTEXT.md, "Panics are for broken internal invariants
// only"): the store crate's production code panics on nothing external — a
// malformed body, row, or file is a structured error. Tests may unwrap.
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const SCHEMA: &str = include_str!("schema.sql");

/// The fact-file schema this build reads and writes; a file stamped newer
/// is refused (CONTEXT.md: schema versions and compatibility errors, never
/// guessing). Version 2 added `tabs.listed_json` / `tabs.listed_response`;
/// version 3 added `realm` (the coordinate above league, 2026-09-02) to
/// `tabs` (rekeyed `(realm, league, id)` — the table is rebuilt, every
/// existing row as pc), `characters`, and `items`; version 4 (same day)
/// rekeyed `characters` by the GGG `id` (rebuilt through each row's json;
/// item locations move from the name to the id), added the listing
/// columns to characters, `items.container`, and `items.seen_response`;
/// version 5 (same day) added `responses.withheld` as a count, and
/// version 6 (same day) made it nullable — NULL is "not withheld", so an
/// empty withheld fetch is still marked. 0 is both "fresh file" and
/// "pre-versioning file" — the DDL and column checks are idempotent, so
/// one migration path serves both.
const FACT_SCHEMA_VERSION: i64 = 6;

/// A facts file written by a newer build than this one. Facts are
/// refetchable, but guessing at an unknown schema is how a file gets
/// silently misread — refuse instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaTooNew {
    pub found: i64,
    pub supported: i64,
}

impl std::fmt::Display for SchemaTooNew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "facts file uses schema v{}, newer than this build's v{}",
            self.found, self.supported
        )
    }
}

impl std::error::Error for SchemaTooNew {}

/// Malformed external input at the record boundary: a 2xx body that lacks
/// the identity-bearing shape ingest depends on. A stable kind (downcast
/// target) per CONTEXT.md's structured-error rule; nothing is written when
/// `record` returns it — the transaction rolls back whole, so a malformed
/// body can never retire tabs, characters, or items, and never mints a
/// response row a snapshot could cite as a basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MalformedBody {
    /// Endpoint whose body was malformed (`"stashes"`, `"characters"`, …).
    pub endpoint: &'static str,
    /// What was missing.
    pub missing: &'static str,
}

impl std::fmt::Display for MalformedBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "malformed {} response: missing {}",
            self.endpoint, self.missing
        )
    }
}

impl std::error::Error for MalformedBody {}

/// Item fields the server re-randomizes per fetch (ground-truth N36), so
/// they never count as a change.
pub const VOLATILE_ITEM_FIELDS: &[&str] = &["veiledMods"];

/// Where the character response keeps its items (documented arrays;
/// `guardian` is the animate guardian's gear on PoE1, `skills` PoE2's).
/// An array of item-shaped objects that is *not* named here is the drift
/// tripwire's business: counted, surfaced, never lifted.
const CHARACTER_ITEM_ARRAYS: &[&str] = &[
    "inventory",
    "equipment",
    "jewels",
    "rucksack",
    "guardian",
    "skills",
];

/// Which API response a body is. The daemon maps a job kind onto this; the
/// store maps it onto tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Leagues,
    Profile,
    /// `realm` on the four data endpoints is the request's realm — what
    /// the URL was rendered with — and is what the facts are stamped
    /// with; a body's own `realm` field stays verbatim in its json.
    Characters {
        realm: String,
    },
    Character {
        realm: String,
        name: String,
    },
    Stashes {
        realm: String,
        league: String,
    },
    /// One tab or one substash; `sub` is the substash id under tab `id`.
    Stash {
        realm: String,
        league: String,
        id: String,
        sub: Option<String>,
    },
}

impl Endpoint {
    /// The daemon's job vocabulary → endpoint. `None` for kinds that carry
    /// no storable response (probe, sleep, fetch, refresh…). This is the
    /// decode boundary where an omitted realm means pc and an omitted
    /// league means Standard, so persisted pre-realm jobs still decode;
    /// the daemon sends both explicitly.
    pub fn from_job(kind: &str, params: &Value) -> Option<Endpoint> {
        let s = |k: &str| params.get(k).and_then(Value::as_str).map(str::to_string);
        let realm = || s("realm").unwrap_or_else(|| "pc".into());
        let league = || s("league").unwrap_or_else(|| "Standard".into());
        Some(match kind {
            "leagues" => Endpoint::Leagues,
            "profile" => Endpoint::Profile,
            "characters" => Endpoint::Characters { realm: realm() },
            "character" => Endpoint::Character {
                realm: realm(),
                name: s("name")?,
            },
            "stashes" => Endpoint::Stashes {
                realm: realm(),
                league: league(),
            },
            "stash" => Endpoint::Stash {
                realm: realm(),
                league: league(),
                id: s("id")?,
                sub: s("sub"),
            },
            _ => return None,
        })
    }

    fn name(&self) -> &'static str {
        match self {
            Endpoint::Leagues => "leagues",
            Endpoint::Profile => "profile",
            Endpoint::Characters { .. } => "characters",
            Endpoint::Character { .. } => "character",
            Endpoint::Stashes { .. } => "stashes",
            Endpoint::Stash { .. } => "stash",
        }
    }
}

/// What one `record` did, for logs and tests.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ingest {
    pub response_id: i64,
    pub items: usize,
    pub added: usize,
    pub moved: usize,
    pub changed: usize,
    pub removed: usize,
    /// Item-shaped objects left in a character envelope in an array this
    /// build does not lift (the drift tripwire): surfaced, never a failure.
    pub unlifted: usize,
    /// `Some(n)` when the fetch was of a location a listing has retired:
    /// the whole body stays verbatim in the response row and nothing
    /// else lands — membership belongs to the listing, and only a listing
    /// revives a location (clearing its `fetched_at`, so the next plan
    /// fetches it again). `n` counts every item fact the body carried,
    /// socketed gems included; `Some(0)` is a withheld fetch of an empty
    /// location. `None`: an ordinary response.
    pub withheld: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabRow {
    pub realm: String,
    pub league: String,
    pub id: String,
    pub parent: Option<String>,
    pub name: String,
    pub r#type: String,
    pub idx: Option<i64>,
    pub listed_at: Option<i64>,
    pub fetched_at: Option<i64>,
    pub removed_at: Option<i64>,
    /// Live (not removed) items whose location is this tab.
    pub item_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRow {
    /// The GGG character id — identity, stable across renames; the full
    /// 64-hex string, since policy ids match exactly.
    pub id: String,
    pub name: String,
    pub realm: String,
    pub league: Option<String>,
    pub class: Option<String>,
    pub level: Option<i64>,
    pub listed_at: Option<i64>,
    /// Set once the full character (equipment + inventory) has been fetched;
    /// a listed-only character has items only if a fetch ever ran.
    pub fetched_at: Option<i64>,
    /// Live (not removed) items whose location is this character.
    pub item_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemRow {
    pub id: String,
    pub realm: String,
    pub league: Option<String>,
    pub location_kind: String,
    pub location_id: String,
    /// The array the item came from (`items` for a stash; a character's
    /// `inventory`, `equipment`, `jewels`, `rucksack`, `guardian`, or
    /// `skills`). `None`: recorded before facts v4.
    pub container: Option<String>,
    pub socketed_in: Option<String>,
    pub name: String,
    pub type_line: String,
    pub base_type: String,
    pub rarity: Option<String>,
    pub stack_size: Option<i64>,
    pub first_seen: i64,
    pub last_seen: i64,
    pub removed_at: Option<i64>,
    pub json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRow {
    pub at: i64,
    pub item_id: String,
    pub kind: String,
    pub from_location: Option<String>,
    pub to_location: Option<String>,
    pub name: Option<String>,
    pub type_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub path: String,
    pub bytes: u64,
    pub responses: i64,
    pub leagues: i64,
    pub characters: i64,
    pub tabs: i64,
    pub items: i64,
    pub items_removed: i64,
    pub events: i64,
    /// The drift tripwire's count: item-shaped objects in arrays this
    /// build does not lift, as of each live character's latest fetch (so
    /// it clears once a build that lifts them has fetched). Non-zero
    /// means GGG added an item array the store does not know — a code
    /// change, not a fault.
    pub unlifted_items: i64,
    /// Responses withheld — fetches of locations a listing had retired,
    /// their bodies kept on the response row and landed nowhere — and the
    /// item facts they carried in total (`responses.withheld`).
    pub withheld_responses: i64,
    pub withheld_items: i64,
}

pub mod annotations;
pub mod index;
pub mod jobs;
pub mod snapshot;
pub use annotations::{AnnotationError, AnnotationRow, Annotations, annotations_path};
pub use index::{
    AccountEntry, Index, Resolve, account_matches, account_path, index_path, store_dir,
};
pub use snapshot::{
    CharacterSnapshot, ListingBasis, RefreshSnapshot, SYNC_POLICY_KEY, SYNC_POLICY_KIND,
    SYNC_POLICY_SCOPE, TabSnapshot,
};

/// Listing order shared by [`Store::tabs`] and [`Store::refresh_snapshot`]:
/// folder children after their folder, substashes after their tab.
pub(crate) const TAB_ORDER_SQL: &str = "ORDER BY COALESCE((SELECT p.idx FROM tabs p WHERE p.realm = t.realm AND p.league = t.league AND p.id = t.parent), t.idx, 1000000),
                       t.parent IS NOT NULL, COALESCE(t.idx, 0), t.name";

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub struct Store {
    conn: Connection,
    path: PathBuf,
}

impl Store {
    pub fn open(path: &Path) -> Result<Store> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        let conn = Connection::open(path).with_context(|| format!("opening {}", path.display()))?;
        Self::init(conn, path.to_path_buf())
    }

    pub fn open_memory() -> Result<Store> {
        Self::init(Connection::open_in_memory()?, PathBuf::from(":memory:"))
    }

    fn init(mut conn: Connection, path: PathBuf) -> Result<Store> {
        // WAL: the daemon writes while any number of frontends read.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        // Discovery and migration serialize under one immediate
        // transaction: two processes opening the same legacy file must not
        // both run the ALTERs (the loser would fail on a duplicate
        // column); the loser waits here and then reads the stamped
        // version. Facts are refetchable, but a live store is not
        // discarded over a new column: pre-version files get them added in
        // place.
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let found: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        match found {
            v if v > FACT_SCHEMA_VERSION => {
                return Err(SchemaTooNew {
                    found: v,
                    supported: FACT_SCHEMA_VERSION,
                }
                .into());
            }
            FACT_SCHEMA_VERSION => {}
            _ => {
                tx.execute_batch(SCHEMA)?;
                for (table, column, ddl) in [
                    (
                        "tabs",
                        "listed_json",
                        "ALTER TABLE tabs ADD COLUMN listed_json TEXT",
                    ),
                    (
                        "tabs",
                        "listed_response",
                        "ALTER TABLE tabs ADD COLUMN listed_response INTEGER",
                    ),
                    // v3: realm, the coordinate above league. Existing
                    // rows are pc — the only realm anything ever fetched.
                    (
                        "items",
                        "realm",
                        "ALTER TABLE items ADD COLUMN realm TEXT NOT NULL DEFAULT 'pc'",
                    ),
                    (
                        "characters",
                        "realm",
                        "ALTER TABLE characters ADD COLUMN realm TEXT NOT NULL DEFAULT 'pc'",
                    ),
                    // v4: the array an item came from — not in its json,
                    // so pre-v4 character items stay NULL; stash items
                    // are backfilled below (theirs is always `items`).
                    (
                        "items",
                        "container",
                        "ALTER TABLE items ADD COLUMN container TEXT",
                    ),
                    // v4: item membership per response, like listings —
                    // a pre-v4 row (NULL) counts as unseen by any later
                    // fetch of its location, which is exactly right.
                    (
                        "items",
                        "seen_response",
                        "ALTER TABLE items ADD COLUMN seen_response INTEGER",
                    ),
                    // v6: a withheld fetch's count on its response row,
                    // NULL for an ordinary response.
                    (
                        "responses",
                        "withheld",
                        "ALTER TABLE responses ADD COLUMN withheld INTEGER",
                    ),
                ] {
                    if !has_column(&tx, table, column)? {
                        tx.execute(ddl, [])?;
                    }
                }
                // A v5 file has the column NOT NULL DEFAULT 0, where 0 meant
                // both "not withheld" and "withheld, empty": rebuild it as
                // nullable, reading a positive count as withheld and 0 as
                // ordinary (v5 could not tell an empty withheld fetch
                // apart, so that information is gone; it never lived past
                // this branch's own day).
                let v5_not_null: bool = tx.query_row(
                    "SELECT \"notnull\" FROM pragma_table_info('responses') WHERE name = 'withheld'",
                    [],
                    |r| r.get::<_, i64>(0).map(|n| n != 0),
                )?;
                if v5_not_null {
                    tx.execute_batch(
                        "ALTER TABLE responses RENAME COLUMN withheld TO withheld_v5;
                         ALTER TABLE responses ADD COLUMN withheld INTEGER;
                         UPDATE responses SET withheld = CASE WHEN withheld_v5 > 0 THEN withheld_v5 END;
                         ALTER TABLE responses DROP COLUMN withheld_v5;",
                    )?;
                }
                // v3 rekeys `tabs` as (realm, league, id). A primary key
                // cannot be altered in place, so a pre-v3 table is rebuilt
                // row for row (the listing columns above exist by now, so
                // v0 and v2 files take the same path); items and events
                // are untouched.
                if !has_column(&tx, "tabs", "realm")? {
                    tx.execute("ALTER TABLE tabs RENAME TO tabs_pre_realm", [])?;
                    tx.execute_batch(SCHEMA)?;
                    tx.execute(
                        "INSERT INTO tabs (realm, league, id, parent, name, type, idx, json, listed_json, listed_at, listed_response, fetched_at, removed_at)
                         SELECT 'pc', league, id, parent, name, type, idx, json, listed_json, listed_at, listed_response, fetched_at, removed_at FROM tabs_pre_realm",
                        [],
                    )?;
                    tx.execute("DROP TABLE tabs_pre_realm", [])?;
                }
                // v4 rekeys `characters` by the GGG id, taken from each
                // row's json (list entries and fetched bodies both carry
                // it); a row whose json lacks one is dropped and its items
                // retired (facts are refetchable). Item locations move
                // from the name to the id through the same json, so the
                // first post-migration fetch produces no false moves;
                // event history keeps its old location strings.
                if !has_column(&tx, "characters", "id")? {
                    tx.execute("ALTER TABLE characters RENAME TO characters_pre_id", [])?;
                    tx.execute_batch(SCHEMA)?;
                    tx.execute(
                        "INSERT OR IGNORE INTO characters (id, realm, name, league, class, level, json, listed_json, listed_at, fetched_at, removed_at)
                         SELECT json_extract(json, '$.id'), realm, name, league, class, level, json,
                                CASE WHEN fetched_at IS NULL THEN json END, listed_at, fetched_at, removed_at
                           FROM characters_pre_id
                          WHERE json_extract(json, '$.id') IS NOT NULL
                          ORDER BY COALESCE(fetched_at, 0) DESC, COALESCE(listed_at, 0) DESC",
                        [],
                    )?;
                    tx.execute(
                        "UPDATE items SET location_id = (SELECT json_extract(p.json, '$.id') FROM characters_pre_id p WHERE p.name = items.location_id)
                          WHERE location_kind = 'character'
                            AND EXISTS (SELECT 1 FROM characters_pre_id p WHERE p.name = items.location_id AND json_extract(p.json, '$.id') IS NOT NULL)",
                        [],
                    )?;
                    tx.execute(
                        "UPDATE items SET removed_at = ?1 WHERE location_kind = 'character' AND removed_at IS NULL
                            AND location_id IN (SELECT name FROM characters_pre_id WHERE json_extract(json, '$.id') IS NULL)",
                        [now()],
                    )?;
                    tx.execute("DROP TABLE characters_pre_id", [])?;
                    // The listing basis a planner would cite must have
                    // rows stamped to it: re-stamp membership from the
                    // latest character listing on record per realm. The
                    // envelope is the verbatim list (character lists carry
                    // no items), so each named row gets its entry and the
                    // response id back; rows it does not name stay
                    // unstamped and the next listing retires them.
                    restamp_character_listings(&tx)?;
                }
                tx.execute(
                    "UPDATE items SET container = 'items' WHERE location_kind = 'stash' AND container IS NULL",
                    [],
                )?;
                tx.pragma_update(None, "user_version", FACT_SCHEMA_VERSION)?;
            }
        }
        tx.commit()?;
        Ok(Store { conn, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    // ---- the write side (daemon) ----------------------------------------

    /// Record one API response at time `at` (unix seconds). One transaction
    /// per response: the envelope, every lifted item, and the events the
    /// comparison with what was already known produced.
    pub fn record(
        &mut self,
        endpoint: &Endpoint,
        params: &Value,
        status: u16,
        body: &Value,
        at: i64,
    ) -> Result<Ingest> {
        let tx = self.conn.transaction()?;
        let mut ingest = Ingest::default();
        let mut envelope = body.clone();
        let mut seams: Vec<Seam> = Vec::new();
        // (realm, league, id) per tab this response listed; stamped with
        // the response id once the responses row exists, so listing
        // membership is linked to the response a snapshot cites, never to
        // the clock.
        let mut listed_tabs: Vec<(String, String, String)> = Vec::new();
        // Character ids this response listed, stamped the same way.
        let mut listed_characters: Vec<String> = Vec::new();
        // A fetched tab whose substash stubs this response is the listing
        // of: (realm, league, parent id).
        let mut relist_children_of: Option<(String, String, String)> = None;
        // `Some(n)` once this fetch turns out to be of a location a listing
        // has retired: nothing lands but the response row.
        let mut withheld: Option<usize> = None;

        match endpoint {
            Endpoint::Leagues => {
                for l in body
                    .get("leagues")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if let Some(id) = l.get("id").and_then(Value::as_str) {
                        tx.execute(
                            "INSERT INTO leagues (id, json, seen_at) VALUES (?1, ?2, ?3)
                             ON CONFLICT(id) DO UPDATE SET json = excluded.json, seen_at = excluded.seen_at",
                            params![id, l.to_string(), at],
                        )?;
                    }
                }
            }
            Endpoint::Profile => {
                if let Some(uuid) = body.get("uuid").and_then(Value::as_str) {
                    tx.execute(
                        "INSERT INTO account (uuid, name, json, seen_at) VALUES (?1, ?2, ?3, ?4)
                         ON CONFLICT(uuid) DO UPDATE SET name = excluded.name, json = excluded.json, seen_at = excluded.seen_at",
                        params![uuid, body.get("name").and_then(Value::as_str), body.to_string(), at],
                    )?;
                }
            }
            Endpoint::Characters { realm } => {
                // A 2xx body without a `characters` array is malformed
                // input, not an empty account: treating it as empty would
                // remove every character (CONTEXT.md: malformed external
                // input is a structured error). An empty array is fine.
                let Some(list) = body.get("characters").and_then(Value::as_array) else {
                    return Err(MalformedBody {
                        endpoint: "characters",
                        missing: "a `characters` array",
                    }
                    .into());
                };
                for c in list {
                    // Identity-bearing entries error rather than skip: a
                    // list of id-less entries must not read as an
                    // authoritative empty and retire everyone (the error
                    // rolls the whole transaction back). `id` is the
                    // identity, `name` the address a plan renders — both
                    // documented required.
                    let Some(id) = c.get("id").and_then(Value::as_str) else {
                        return Err(MalformedBody {
                            endpoint: "characters",
                            missing: "an `id` on a character entry",
                        }
                        .into());
                    };
                    let Some(name) = c.get("name").and_then(Value::as_str) else {
                        return Err(MalformedBody {
                            endpoint: "characters",
                            missing: "a `name` on a character entry",
                        }
                        .into());
                    };
                    // The listing owns `league` (the coverage coordinate)
                    // and `listed_json`; `json` is the entry until a fetch
                    // replaces it, exactly as a tab's is.
                    tx.execute(
                        "INSERT INTO characters (id, realm, name, league, class, level, json, listed_json, listed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8)
                         ON CONFLICT(id) DO UPDATE SET realm = excluded.realm, name = excluded.name, league = excluded.league,
                           class = excluded.class, level = excluded.level, listed_json = excluded.listed_json,
                           listed_at = excluded.listed_at,
                           fetched_at = CASE WHEN characters.removed_at IS NOT NULL THEN NULL ELSE characters.fetched_at END,
                           removed_at = NULL",
                        params![id, realm, name, c.get("league").and_then(Value::as_str), c.get("class").and_then(Value::as_str),
                                c.get("level").and_then(Value::as_i64), c.to_string(), at],
                    )?;
                    listed_characters.push(id.to_string());
                }
                // Removal happens below, keyed to this response's id and
                // scoped to this realm.
            }
            Endpoint::Character { realm, name } => {
                let Some(character) = envelope.get("character").and_then(Value::as_object) else {
                    return Err(MalformedBody {
                        endpoint: "character",
                        missing: "a `character` object",
                    }
                    .into());
                };
                // The body's id keys the row — a 200 under a stale name is
                // a true fact about whoever holds that name now (CONTEXT.md:
                // no expected-id check); without an id there is no row to
                // file it under.
                let Some(id) = character
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                else {
                    return Err(MalformedBody {
                        endpoint: "character",
                        missing: "an `id` on the character",
                    }
                    .into());
                };
                // The malformed-body contract holds whatever happens next:
                // an id-less item is refused before the store decides
                // whether the body lands or is withheld.
                for key in CHARACTER_ITEM_ARRAYS {
                    check_item_ids(character.get(*key), "character")?;
                }
                // Membership is the listing's: at a location a listing has
                // retired, the whole body stays verbatim in the response
                // row (arrays included — nothing is split off and lost)
                // and nothing else lands: not the row, not its item facts.
                // Only a listing revives the location, and it clears the
                // row's `fetched_at` doing so, so the next plan fetches.
                let retired: Option<Option<i64>> = tx
                    .query_row(
                        "SELECT removed_at FROM characters WHERE id = ?1",
                        [&id],
                        |r| r.get(0),
                    )
                    .optional()?;
                if matches!(retired, Some(Some(_))) {
                    withheld = Some(
                        CHARACTER_ITEM_ARRAYS
                            .iter()
                            .map(|k| count_item_facts(character.get(*k)))
                            .sum(),
                    );
                } else if let Some(character) =
                    envelope.get_mut("character").and_then(Value::as_object_mut)
                {
                    let body_name = character
                        .get("name")
                        .and_then(Value::as_str)
                        .map_or_else(|| name.clone(), str::to_string);
                    let body_league = character
                        .get("league")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    let mut split = serde_json::Map::new();
                    let mut arrays: Vec<(&'static str, Vec<Value>)> = Vec::new();
                    for key in CHARACTER_ITEM_ARRAYS {
                        if let Some(Value::Array(items)) = character.remove(*key) {
                            split.insert((*key).into(), json!(items.len()));
                            arrays.push((key, items));
                        }
                    }
                    character.insert("_split".into(), Value::Object(split));
                    // The drift tripwire: GGG adds fields most leagues, and
                    // a new item array would otherwise go un-lifted
                    // silently. Whatever item-shaped array is left is
                    // counted into the envelope and the ingest — surfaced,
                    // never a failure.
                    let unlifted: serde_json::Map<String, Value> = character
                        .iter()
                        .filter_map(|(key, value)| {
                            let n = value
                                .as_array()
                                .filter(|a| !a.is_empty() && a.iter().all(is_item_shaped))
                                .map(Vec::len)?;
                            Some((key.clone(), json!(n)))
                        })
                        .collect();
                    if !unlifted.is_empty() {
                        ingest.unlifted =
                            unlifted.values().filter_map(Value::as_u64).sum::<u64>() as usize;
                        character.insert("_unlifted".into(), Value::Object(unlifted));
                    }
                    let c = Value::Object(character.clone());
                    // Once a listing has named this row, the listing owns
                    // the address and display fields (`name`, `league`,
                    // `class`, `level`): a fetch authorized under an old
                    // address can land after a newer listing (separate
                    // routes, concurrent sends) and must not roll the
                    // address back. The body's own say stays verbatim in
                    // `json`. A never-listed row takes the body's values
                    // on insert.
                    tx.execute(
                        "INSERT INTO characters (id, realm, name, league, class, level, json, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                         ON CONFLICT(id) DO UPDATE SET realm = excluded.realm,
                           name = CASE WHEN characters.listed_response IS NULL THEN excluded.name ELSE characters.name END,
                           class = CASE WHEN characters.listed_response IS NULL THEN excluded.class ELSE characters.class END,
                           level = CASE WHEN characters.listed_response IS NULL THEN excluded.level ELSE characters.level END,
                           json = excluded.json, fetched_at = excluded.fetched_at",
                        params![id, realm, body_name, body_league, c.get("class").and_then(Value::as_str), c.get("level").and_then(Value::as_i64), c.to_string(), at],
                    )?;
                    // The items take the row's coordinate: the
                    // listing-owned league, not the body's.
                    let row_league: Option<String> =
                        tx.query_row("SELECT league FROM characters WHERE id = ?1", [&id], |r| {
                            r.get(0)
                        })?;
                    for (key, items) in arrays {
                        seams.push(Seam {
                            realm: realm.clone(),
                            league: row_league.clone(),
                            kind: "character",
                            location_id: id.clone(),
                            container: key,
                            items,
                        });
                    }
                }
            }
            Endpoint::Stashes { realm, league } => {
                // A 2xx body without a `stashes` array is malformed input,
                // not an empty account: treating it as empty would remove
                // every listed tab and mint a false listing basis for later
                // snapshots (CONTEXT.md: malformed external input is a
                // structured error). An empty array is fine.
                let Some(list) = body.get("stashes").and_then(Value::as_array) else {
                    return Err(MalformedBody {
                        endpoint: "stashes",
                        missing: "a `stashes` array",
                    }
                    .into());
                };
                let mut idx = 0;
                for tab in list {
                    upsert_listed_tab(
                        &tx,
                        "stashes",
                        realm,
                        league,
                        tab,
                        None,
                        &mut idx,
                        at,
                        &mut listed_tabs,
                    )?;
                    for child in tab
                        .get("children")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                    {
                        let folder = tab.get("id").and_then(Value::as_str).map(str::to_string);
                        upsert_listed_tab(
                            &tx,
                            "stashes",
                            realm,
                            league,
                            child,
                            folder,
                            &mut idx,
                            at,
                            &mut listed_tabs,
                        )?;
                    }
                }
                // Removal happens below, keyed to this response's id.
            }
            Endpoint::Stash {
                realm,
                league,
                id,
                sub,
            } => {
                let Some(stash) = envelope.get("stash").and_then(Value::as_object) else {
                    return Err(MalformedBody {
                        endpoint: "stash",
                        missing: "a `stash` object",
                    }
                    .into());
                };
                let location = sub.clone().unwrap_or_else(|| id.clone());
                // The malformed-body contract holds whatever happens next:
                // an id-less item or substash stub is refused before the
                // store decides whether the body lands or is withheld.
                check_item_ids(stash.get("items"), "stash")?;
                for child in stash
                    .get("children")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if child.get("id").and_then(Value::as_str).is_none() {
                        return Err(MalformedBody {
                            endpoint: "stash",
                            missing: "an `id` on a listed tab entry",
                        }
                        .into());
                    }
                }
                // A tab a listing retired — or a substash whose parent is —
                // is not a live location: membership is the listing's (the
                // parent's fetch, for a substash). The body stays verbatim
                // in the response row and nothing else lands: not the row,
                // not its item facts, and for a retired parent not its
                // substash stubs either, which would rewrite the orphan
                // report the retirement left for the planner.
                let retired_row = |tab: &str| -> Result<bool> {
                    Ok(tx
                        .query_row(
                            "SELECT removed_at IS NOT NULL FROM tabs WHERE realm = ?1 AND league = ?2 AND id = ?3",
                            params![realm, league, tab],
                            |r| r.get::<_, bool>(0),
                        )
                        .optional()?
                        .unwrap_or(false))
                };
                let retired = retired_row(&location)? || (sub.is_some() && retired_row(id)?);
                if retired {
                    withheld = Some(count_item_facts(stash.get("items")));
                } else if let Some(stash) = envelope.get_mut("stash").and_then(Value::as_object_mut)
                {
                    let items = match stash.remove("items") {
                        Some(Value::Array(items)) => items,
                        _ => Vec::new(),
                    };
                    stash.insert("_split".into(), json!({ "items": items.len() }));
                    // Substash stubs of a map/unique tab: each becomes a tab
                    // row whose parent is this tab. The fetched tab's own
                    // row too.
                    let children = match stash.remove("children") {
                        Some(Value::Array(c)) => c,
                        _ => Vec::new(),
                    };
                    let mut idx = 0;
                    for child in &children {
                        upsert_listed_tab(
                            &tx,
                            "stash",
                            realm,
                            league,
                            child,
                            Some(id.clone()),
                            &mut idx,
                            at,
                            &mut listed_tabs,
                        )?;
                    }
                    let fetched = Value::Object(stash.clone());
                    // A fetch never revives a tab (only a listing — or, for
                    // a substash, its parent's fetch — owns membership).
                    tx.execute(
                        "INSERT INTO tabs (realm, league, id, parent, name, type, json, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                         ON CONFLICT(realm, league, id) DO UPDATE SET name = excluded.name, type = excluded.type,
                           json = excluded.json, fetched_at = excluded.fetched_at,
                           parent = COALESCE(excluded.parent, tabs.parent)",
                        params![realm, league, location, sub.as_ref().map(|_| id.clone()),
                                fetched.get("name").and_then(Value::as_str), fetched.get("type").and_then(Value::as_str),
                                fetched.to_string(), at],
                    )?;
                    seams.push(Seam {
                        realm: realm.clone(),
                        league: Some(league.clone()),
                        kind: "stash",
                        location_id: location,
                        container: "items",
                        items,
                    });
                    // A tab's own fetch is the listing of its substashes:
                    // the stubs it carries are stamped above, and the ones
                    // it no longer carries are retired below, once this
                    // response has its id.
                    if sub.is_none() {
                        relist_children_of = Some((realm.clone(), league.clone(), id.clone()));
                    }
                }
            }
        }

        let item_count: usize = seams.iter().map(|s| s.items.len()).sum();
        tx.execute(
            "INSERT INTO responses (endpoint, params, fetched_at, status, envelope, item_count, withheld) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![endpoint.name(), params.to_string(), at, status, envelope.to_string(), item_count as i64, withheld.map(|n| n as i64)],
        )?;
        let response_id = tx.last_insert_rowid();
        ingest.withheld = withheld;
        ingest.response_id = response_id;

        // Listing membership is per response, never per second: stamp the
        // tabs this response listed, then retire the rest. Two listings
        // recorded within one clock second still retire dropped tabs,
        // and a snapshot's tab set can be checked against the basis it
        // cites (`TabSnapshot::listed_response`).
        for (tab_realm, tab_league, tab_id) in &listed_tabs {
            tx.execute(
                "UPDATE tabs SET listed_response = ?4 WHERE realm = ?1 AND league = ?2 AND id = ?3",
                params![tab_realm, tab_league, tab_id, response_id],
            )?;
        }
        for id in &listed_characters {
            tx.execute(
                "UPDATE characters SET listed_response = ?2 WHERE id = ?1",
                params![id, response_id],
            )?;
        }
        // Locations this response retired (a character or tab the listing
        // no longer names): their items go with them — the location
        // vanishing is the strongest "no longer had it" — through the
        // same per-location removal below.
        let mut retired_locations: Vec<Location> = Vec::new();
        if let Endpoint::Characters { realm } = endpoint {
            // A character this listing did not stamp is gone (deleted).
            // Realm-scoped: whether a list spans realms is undocumented,
            // so a realm-R listing retires only realm-R rows — under-
            // retires if lists span realms, never over-retires
            // (CONTEXT.md, 2026-09-02).
            let mut stmt = tx.prepare(
                "UPDATE characters SET removed_at = ?2 WHERE realm = ?1 AND removed_at IS NULL
                   AND (listed_response IS NULL OR listed_response <> ?3)
                 RETURNING id",
            )?;
            let ids = stmt.query_map(params![realm, at, response_id], |r| r.get::<_, String>(0))?;
            for id in ids {
                retired_locations.push(Location::character(realm, id?));
            }
        }
        if let Endpoint::Stashes { realm, league } = endpoint {
            // Not stamped by this listing → removed (top-level and folder
            // children; substashes are only known from fetches and keep
            // their own row). A listing is one (realm, league)'s.
            let mut stmt = tx.prepare(
                "UPDATE tabs SET removed_at = ?3 WHERE realm = ?1 AND league = ?2 AND removed_at IS NULL
                   AND (listed_response IS NULL OR listed_response <> ?4)
                   AND (parent IS NULL OR parent IN (SELECT id FROM tabs t2 WHERE t2.realm = ?1 AND t2.league = ?2 AND t2.type = 'Folder'))
                 RETURNING id",
            )?;
            let ids = stmt.query_map(params![realm, league, at, response_id], |r| {
                r.get::<_, String>(0)
            })?;
            let retired: Vec<String> = ids.collect::<Result<_, _>>()?;
            // A retired parent's substashes keep their rows (the planner
            // reports them as orphaned rather than guessing a path) but
            // their items are not live inventory any more.
            let mut stmt = tx.prepare(
                "SELECT id FROM tabs WHERE realm = ?1 AND league = ?2 AND removed_at IS NULL AND parent = ?3",
            )?;
            let mut orphaned = Vec::new();
            for parent in &retired {
                let ids =
                    stmt.query_map(params![realm, league, parent], |r| r.get::<_, String>(0))?;
                for id in ids {
                    orphaned.push(id?);
                }
                // Their facts go with the parent's, so their freshness
                // does too: when the parent returns, its refetch re-lists
                // them and the plan fetches each again (a "fresh" empty
                // substash would otherwise wait out the window).
                tx.execute(
                    "UPDATE tabs SET fetched_at = NULL WHERE realm = ?1 AND league = ?2 AND removed_at IS NULL AND parent = ?3",
                    params![realm, league, parent],
                )?;
            }
            for id in retired.into_iter().chain(orphaned) {
                retired_locations.push(Location::stash(realm, league, id));
            }
        }
        if let Some((realm, league, parent)) = &relist_children_of {
            // The parent's fetch is its substashes' listing: a stub it no
            // longer carries is retired, with its items.
            let mut stmt = tx.prepare(
                "UPDATE tabs SET removed_at = ?3 WHERE realm = ?1 AND league = ?2 AND parent = ?4 AND removed_at IS NULL
                   AND (listed_response IS NULL OR listed_response <> ?5)
                 RETURNING id",
            )?;
            let ids = stmt.query_map(params![realm, league, at, parent, response_id], |r| {
                r.get::<_, String>(0)
            })?;
            for id in ids {
                retired_locations.push(Location::stash(realm, league, id?));
            }
        }

        // Every seam of one response is one location; a character's
        // arrays share `character/<id>`, so removal runs once per location.
        // Membership is per response: an item this response did not stamp
        // at a location it fetched — or at a location it retired — is
        // removed, whatever the clock says (two fetches in one second are
        // two responses). A location is its full coordinate: realm and,
        // for a stash, league — the same tab id under two realms is two
        // locations.
        let mut locations: Vec<Location> = retired_locations;
        for Seam {
            realm,
            league,
            kind,
            location_id,
            container,
            items,
        } in seams
        {
            for item in items {
                ingest_item(
                    &tx,
                    &mut ingest,
                    response_id,
                    at,
                    &realm,
                    league.as_deref(),
                    container,
                    kind,
                    &location_id,
                    None,
                    item,
                )?;
            }
            let location = Location {
                realm,
                league: (kind == "stash").then(|| league.clone()).flatten(),
                kind,
                id: location_id,
            };
            if !locations.contains(&location) {
                locations.push(location);
            }
        }
        for location in locations {
            // The ids come from the update itself, never from a timestamp
            // match: another response in the same second retiring other
            // items at this location must not be attributed here.
            let mut stmt = tx.prepare(
                "UPDATE items SET removed_at = ?5 WHERE realm = ?1 AND location_kind = ?2 AND location_id = ?3
                   AND (?4 IS NULL OR league = ?4)
                   AND removed_at IS NULL AND (seen_response IS NULL OR seen_response <> ?6)
                 RETURNING id",
            )?;
            let ids = stmt.query_map(
                params![
                    location.realm,
                    location.kind,
                    location.id,
                    location.league,
                    at,
                    response_id
                ],
                |r| r.get::<_, String>(0),
            )?;
            let removed: Vec<String> = ids.collect::<Result<_, _>>()?;
            let from = location.address();
            for id in &removed {
                tx.execute(
                    "INSERT INTO item_events (response_id, at, item_id, kind, from_location) VALUES (?1, ?2, ?3, 'removed', ?4)",
                    params![response_id, at, id, from],
                )?;
            }
            ingest.removed += removed.len();
        }
        tx.commit()?;
        Ok(ingest)
    }

    // ---- the read side (frontends) ---------------------------------------

    pub fn status(&self) -> Result<Status> {
        let count = |sql: &str| -> Result<i64> { Ok(self.conn.query_row(sql, [], |r| r.get(0))?) };
        Ok(Status {
            path: self.path.display().to_string(),
            bytes: std::fs::metadata(&self.path).map(|m| m.len()).unwrap_or(0),
            responses: count("SELECT count(*) FROM responses")?,
            leagues: count("SELECT count(*) FROM leagues")?,
            characters: count("SELECT count(*) FROM characters WHERE removed_at IS NULL")?,
            tabs: count("SELECT count(*) FROM tabs WHERE removed_at IS NULL")?,
            items: count("SELECT count(*) FROM items WHERE removed_at IS NULL")?,
            items_removed: count("SELECT count(*) FROM items WHERE removed_at IS NOT NULL")?,
            events: count("SELECT count(*) FROM item_events")?,
            withheld_responses: count("SELECT count(*) FROM responses WHERE withheld IS NOT NULL")?,
            withheld_items: count("SELECT COALESCE(SUM(withheld), 0) FROM responses")?,
            unlifted_items: count(
                "SELECT COALESCE(SUM(value), 0) FROM characters c, json_each(json_extract(c.json, '$._unlifted'))
                  WHERE c.removed_at IS NULL",
            )?,
        })
    }

    /// Tabs of a league in listing order (folder children after their
    /// folder, substashes after their tab), removed ones excluded.
    /// Characters known to the store (deleted ones excluded), with live item
    /// counts. `league` restricts to one league; the list endpoint spans all.
    pub fn characters(
        &self,
        realm: Option<&str>,
        league: Option<&str>,
    ) -> Result<Vec<CharacterRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.name, c.realm, c.league, c.class, c.level, c.listed_at, c.fetched_at,
                    (SELECT count(*) FROM items i WHERE i.realm = c.realm AND i.location_kind = 'character' AND i.location_id = c.id AND i.removed_at IS NULL)
               FROM characters c
              WHERE c.removed_at IS NULL AND (?1 IS NULL OR c.realm = ?1) AND (?2 IS NULL OR c.league = ?2)
              ORDER BY c.realm, c.league, c.level DESC, c.name",
        )?;
        let rows = stmt.query_map([realm, league], |r| {
            Ok(CharacterRow {
                id: r.get(0)?,
                name: r.get(1)?,
                realm: r.get(2)?,
                league: r.get(3)?,
                class: r.get(4)?,
                level: r.get(5)?,
                listed_at: r.get(6)?,
                fetched_at: r.get(7)?,
                item_count: r.get(8)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    pub fn tabs(&self, realm: &str, league: &str) -> Result<Vec<TabRow>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT t.realm, t.league, t.id, t.parent, COALESCE(t.name, ''), COALESCE(t.type, ''), t.idx, t.listed_at, t.fetched_at, t.removed_at,
                    (SELECT count(*) FROM items i WHERE i.realm = t.realm AND i.league = t.league AND i.location_kind = 'stash' AND i.location_id = t.id AND i.removed_at IS NULL)
               FROM tabs t WHERE t.realm = ?1 AND t.league = ?2 AND t.removed_at IS NULL {TAB_ORDER_SQL}"
        ))?;
        let rows = stmt.query_map([realm, league], |r| {
            Ok(TabRow {
                realm: r.get(0)?,
                league: r.get(1)?,
                id: r.get(2)?,
                parent: r.get(3)?,
                name: r.get(4)?,
                r#type: r.get(5)?,
                idx: r.get(6)?,
                listed_at: r.get(7)?,
                fetched_at: r.get(8)?,
                removed_at: r.get(9)?,
                item_count: r.get(10)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Case-insensitive substring search over name, type line, and base
    /// type, live items only unless `include_removed`.
    pub fn search(
        &self,
        text: &str,
        realm: Option<&str>,
        league: Option<&str>,
        include_removed: bool,
        limit: usize,
    ) -> Result<Vec<ItemRow>> {
        let pattern = format!("%{}%", text.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = self.conn.prepare(
            "SELECT id, league, location_kind, location_id, socketed_in, COALESCE(name, ''), COALESCE(type_line, ''),
                    COALESCE(base_type, ''), rarity, stack_size, first_seen, last_seen, removed_at, json, realm, container
               FROM items
              WHERE (name LIKE ?1 ESCAPE '\\' OR type_line LIKE ?1 ESCAPE '\\' OR base_type LIKE ?1 ESCAPE '\\')
                AND (?5 IS NULL OR realm = ?5)
                AND (?2 IS NULL OR league = ?2)
                AND (?3 OR removed_at IS NULL)
              ORDER BY location_kind, location_id, y, x
              LIMIT ?4",
        )?;
        let rows = stmt.query_map(
            params![pattern, league, include_removed, limit as i64, realm],
            |r| {
                let json: String = r.get(13)?;
                Ok(ItemRow {
                    id: r.get(0)?,
                    realm: r.get(14)?,
                    league: r.get(1)?,
                    location_kind: r.get(2)?,
                    location_id: r.get(3)?,
                    container: r.get(15)?,
                    socketed_in: r.get(4)?,
                    name: r.get(5)?,
                    type_line: r.get(6)?,
                    base_type: r.get(7)?,
                    rarity: r.get(8)?,
                    stack_size: r.get(9)?,
                    first_seen: r.get(10)?,
                    last_seen: r.get(11)?,
                    removed_at: r.get(12)?,
                    json: serde_json::from_str(&json).unwrap_or(Value::Null),
                })
            },
        )?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// One item by id, removed or not.
    pub fn item(&self, id: &str) -> Result<Option<ItemRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, league, location_kind, location_id, socketed_in, COALESCE(name, ''), COALESCE(type_line, ''),
                    COALESCE(base_type, ''), rarity, stack_size, first_seen, last_seen, removed_at, json, realm, container FROM items WHERE id = ?1",
        )?;
        Ok(stmt
            .query_row([id], |r| {
                let json: String = r.get(13)?;
                Ok(ItemRow {
                    id: r.get(0)?,
                    realm: r.get(14)?,
                    league: r.get(1)?,
                    location_kind: r.get(2)?,
                    location_id: r.get(3)?,
                    container: r.get(15)?,
                    socketed_in: r.get(4)?,
                    name: r.get(5)?,
                    type_line: r.get(6)?,
                    base_type: r.get(7)?,
                    rarity: r.get(8)?,
                    stack_size: r.get(9)?,
                    first_seen: r.get(10)?,
                    last_seen: r.get(11)?,
                    removed_at: r.get(12)?,
                    json: serde_json::from_str(&json).unwrap_or(Value::Null),
                })
            })
            .optional()?)
    }

    /// Item events since `since` (unix seconds), oldest first.
    pub fn events_since(&self, since: i64, limit: usize) -> Result<Vec<EventRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.at, e.item_id, e.kind, e.from_location, e.to_location, i.name, i.type_line
               FROM item_events e LEFT JOIN items i ON i.id = e.item_id
              WHERE e.at >= ?1 ORDER BY e.at, e.id LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![since, limit as i64], |r| {
            Ok(EventRow {
                at: r.get(0)?,
                item_id: r.get(1)?,
                kind: r.get(2)?,
                from_location: r.get(3)?,
                to_location: r.get(4)?,
                name: r.get(5)?,
                type_line: r.get(6)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Re-extract every derived column from each row's own `json`. The
    /// repair for a wrong extraction; never a refetch.
    pub fn rebuild(&mut self) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let rows: Vec<(String, String)> = {
            let mut stmt = tx.prepare("SELECT id, json FROM items")?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.collect::<Result<_, _>>()?
        };
        let n = rows.len();
        for (id, json) in rows {
            let v: Value = serde_json::from_str(&json)?;
            let c = Columns::of(&v);
            tx.execute(
                "UPDATE items SET name = ?2, type_line = ?3, base_type = ?4, rarity = ?5, stack_size = ?6, x = ?7, y = ?8, w = ?9, h = ?10 WHERE id = ?1",
                params![id, c.name, c.type_line, c.base_type, c.rarity, c.stack_size, c.x, c.y, c.w, c.h],
            )?;
        }
        tx.commit()?;
        Ok(n)
    }

    /// Item annotations whose item this fact store no longer has live
    /// (removed, or never seen). No fact-side event ever deletes intent
    /// (CONTEXT.md): this is how kept-but-detached intent stays visible so
    /// a frontend can surface it instead of it silently rotting.
    pub fn orphaned_item_annotations(
        &self,
        annotations: &Annotations,
    ) -> Result<Vec<AnnotationRow>> {
        let mut orphaned = Vec::new();
        for row in annotations.list(Some("item"))? {
            let live: Option<Option<i64>> = self
                .conn
                .query_row(
                    "SELECT removed_at FROM items WHERE id = ?1",
                    [&row.key],
                    |r| r.get(0),
                )
                .optional()?;
            match live {
                Some(None) => {} // the item is live; the annotation is attached
                _ => orphaned.push(row),
            }
        }
        Ok(orphaned)
    }
}

/// One item array of a response and where its items live: the realm and
/// league the request was made under, the location (`kind`, `id`) the
/// items are filed at, and the array they came from. A character's arrays
/// are several seams at one location.
struct Seam {
    realm: String,
    league: Option<String>,
    kind: &'static str,
    location_id: String,
    container: &'static str,
    items: Vec<Value>,
}

/// Where items are filed, as a full coordinate: realm, the league for a
/// stash (a tab is `(realm, league, id)`; a character's league is a
/// property the listing owns, never part of where it is), the kind, and
/// the id.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Location {
    realm: String,
    league: Option<String>,
    kind: &'static str,
    id: String,
}

impl Location {
    /// The address events carry: `stash/<realm>/<league>/<id>` or
    /// `character/<realm>/<id>` — the full coordinate, so a move across
    /// realms is recoverable from the event alone.
    fn address(&self) -> String {
        address(self.kind, &self.realm, self.league.as_deref(), &self.id)
    }

    fn character(realm: &str, id: String) -> Location {
        Location {
            realm: realm.into(),
            league: None,
            kind: "character",
            id,
        }
    }

    fn stash(realm: &str, league: &str, id: String) -> Location {
        Location {
            realm: realm.into(),
            league: Some(league.into()),
            kind: "stash",
            id,
        }
    }
}

fn address(kind: &str, realm: &str, league: Option<&str>, id: &str) -> String {
    match (kind, league) {
        ("stash", Some(league)) => format!("{kind}/{realm}/{league}/{id}"),
        _ => format!("{kind}/{realm}/{id}"),
    }
}

/// What the store already knew about an item, for the comparison.
struct Previous {
    realm: String,
    league: Option<String>,
    kind: String,
    location_id: String,
    json: String,
    removed_at: Option<i64>,
    container: Option<String>,
}

/// The identity check ingest applies to every item, run up front so it
/// holds for a withheld body too: an id-less item anywhere in `items`
/// (socketed gems included) is malformed, and nothing is written.
fn check_item_ids(items: Option<&Value>, endpoint: &'static str) -> Result<()> {
    for item in items.and_then(Value::as_array).into_iter().flatten() {
        if item.get("id").and_then(Value::as_str).is_none() {
            return Err(MalformedBody {
                endpoint,
                missing: "an `id` on an item",
            }
            .into());
        }
        check_item_ids(item.get("socketedItems"), endpoint)?;
    }
    Ok(())
}

/// How many item facts an array carries, the way ingest would count
/// them: every item plus every socketed gem, recursively.
fn count_item_facts(items: Option<&Value>) -> usize {
    items
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|item| 1 + count_item_facts(item.get("socketedItems")))
        .sum()
}

/// Whether a value looks like a GGG item: an object with a `typeLine`
/// (documented required on `Item`). The drift tripwire's test for an
/// un-lifted array in a character envelope.
fn is_item_shaped(v: &Value) -> bool {
    v.as_object().is_some_and(|o| o.contains_key("typeLine"))
}

/// The v4 migration's membership repair: for each realm, the latest 2xx
/// character listing on record re-stamps the rows it names (`listed_json`,
/// `listed_at`, `listed_response`), so a snapshot taken after the
/// migration cites a basis its rows are actually stamped to.
fn restamp_character_listings(tx: &Connection) -> Result<()> {
    let latest: Vec<(String, i64)> = {
        let mut stmt = tx.prepare(
            "SELECT COALESCE(json_extract(params, '$.realm'), 'pc'), MAX(id) FROM responses
              WHERE endpoint = 'characters' AND status BETWEEN 200 AND 299
              GROUP BY 1",
        )?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect::<Result<_, _>>()?
    };
    for (realm, response_id) in latest {
        let (fetched_at, envelope): (i64, String) = tx.query_row(
            "SELECT fetched_at, envelope FROM responses WHERE id = ?1",
            [response_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        // A row this store wrote that no longer parses is a damaged file,
        // reported — never silently skipped.
        let body: Value = serde_json::from_str(&envelope)
            .with_context(|| format!("response {response_id}: malformed envelope in store"))?;
        for entry in body
            .get("characters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(id) = entry.get("id").and_then(Value::as_str) {
                tx.execute(
                    "UPDATE characters SET listed_json = ?2, listed_at = ?3, listed_response = ?4
                      WHERE id = ?1 AND realm = ?5",
                    params![id, entry.to_string(), fetched_at, response_id, realm],
                )?;
            }
        }
    }
    Ok(())
}

/// Whether `table` has `column` — the migration's idempotence check.
fn has_column(tx: &Connection, table: &str, column: &str) -> Result<bool> {
    let present: i64 = tx.query_row(
        "SELECT count(*) FROM pragma_table_info(?1) WHERE name = ?2",
        [table, column],
        |r| r.get(0),
    )?;
    Ok(present > 0)
}

#[allow(clippy::too_many_arguments)]
fn upsert_listed_tab(
    tx: &Connection,
    via: &'static str,
    realm: &str,
    league: &str,
    tab: &Value,
    parent: Option<String>,
    idx: &mut i64,
    at: i64,
    listed: &mut Vec<(String, String, String)>,
) -> Result<()> {
    // Identity-bearing entries error rather than skip: an id-less entry
    // silently dropped would let a malformed list read as an authoritative
    // (near-)empty one and retire real tabs — the error rolls the whole
    // transaction back instead. A listing that revives a retired row
    // clears its `fetched_at`: the facts at that location were retired
    // with it, so the next plan must fetch again.
    let Some(id) = tab.get("id").and_then(Value::as_str) else {
        return Err(MalformedBody {
            endpoint: via,
            missing: "an `id` on a listed tab entry",
        }
        .into());
    };
    let mut entry = tab.clone();
    if let Some(o) = entry.as_object_mut() {
        o.remove("children");
    }
    let position = tab.get("index").and_then(Value::as_i64).unwrap_or(*idx);
    *idx += 1;
    // The list entry lives in `listed_json`, which a fetch never touches —
    // the listing's metadata (the heuristic `items` count on stubs) must
    // survive the fetched body landing in `json`. On insert the entry
    // doubles as `json` until a fetch replaces it.
    tx.execute(
        "INSERT INTO tabs (realm, league, id, parent, name, type, idx, json, listed_json, listed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9)
         ON CONFLICT(realm, league, id) DO UPDATE SET parent = excluded.parent, name = excluded.name, type = excluded.type,
           idx = excluded.idx, listed_json = excluded.listed_json, listed_at = excluded.listed_at,
           fetched_at = CASE WHEN tabs.removed_at IS NOT NULL THEN NULL ELSE tabs.fetched_at END,
           removed_at = NULL",
        params![realm, league, id, parent, tab.get("name").and_then(Value::as_str), tab.get("type").and_then(Value::as_str),
                position, entry.to_string(), at],
    )?;
    listed.push((realm.to_string(), league.to_string(), id.to_string()));
    Ok(())
}

struct Columns<'a> {
    name: &'a str,
    type_line: &'a str,
    base_type: &'a str,
    rarity: Option<&'a str>,
    stack_size: Option<i64>,
    x: Option<i64>,
    y: Option<i64>,
    w: Option<i64>,
    h: Option<i64>,
}

impl<'a> Columns<'a> {
    fn of(v: &'a Value) -> Columns<'a> {
        let s = |k: &str| v.get(k).and_then(Value::as_str);
        let i = |k: &str| v.get(k).and_then(Value::as_i64);
        Columns {
            name: s("name").unwrap_or(""),
            type_line: s("typeLine").unwrap_or(""),
            base_type: s("baseType").unwrap_or(""),
            rarity: s("rarity"),
            stack_size: i("stackSize"),
            x: i("x"),
            y: i("y"),
            w: i("w"),
            h: i("h"),
        }
    }
}

/// Two item bodies are the same item state once volatile fields are ignored.
pub fn same_item(a: &Value, b: &Value) -> bool {
    if a == b {
        return true;
    }
    let strip = |v: &Value| {
        let mut v = v.clone();
        if let Some(o) = v.as_object_mut() {
            for f in VOLATILE_ITEM_FIELDS {
                o.remove(*f);
            }
        }
        v
    };
    strip(a) == strip(b)
}

#[allow(clippy::too_many_arguments)]
fn ingest_item(
    tx: &Connection,
    ingest: &mut Ingest,
    response_id: i64,
    at: i64,
    realm: &str,
    league: Option<&str>,
    container: &'static str,
    kind: &'static str,
    location_id: &str,
    socketed_in: Option<&str>,
    mut item: Value,
) -> Result<()> {
    // Identity-bearing entries error rather than skip (same rule as tabs
    // and characters): an id is what makes an item trackable, and a fetch
    // full of id-less entries silently dropped would remove every real
    // item at the location. The error rolls the whole ingest back. Legacy
    // pull snapshots that need tolerance get it at the import boundary
    // (`acq store import` strips and reports), never here.
    let Some(id) = item.get("id").and_then(Value::as_str).map(str::to_string) else {
        return Err(MalformedBody {
            endpoint: kind,
            missing: "an `id` on an item",
        }
        .into());
    };
    // Socketed gems are items: lift them out, same location, parented.
    let gems = match item.as_object_mut().and_then(|o| o.remove("socketedItems")) {
        Some(Value::Array(g)) => g,
        _ => Vec::new(),
    };
    let to = address(
        kind,
        realm,
        (kind == "stash").then_some(league).flatten(),
        location_id,
    );
    let previous: Option<Previous> = tx
        .query_row(
            "SELECT location_kind, location_id, json, removed_at, container, realm, league FROM items WHERE id = ?1",
            [&id],
            |r| {
                Ok(Previous {
                    kind: r.get(0)?,
                    location_id: r.get(1)?,
                    json: r.get(2)?,
                    removed_at: r.get(3)?,
                    container: r.get(4)?,
                    realm: r.get(5)?,
                    league: r.get(6)?,
                })
            },
        )
        .optional()?;
    let c = Columns::of(&item);
    let json = item.to_string();
    match &previous {
        None => {
            tx.execute(
                "INSERT INTO items (id, league, location_kind, location_id, socketed_in, name, type_line, base_type, rarity, stack_size, x, y, w, h, json, first_seen, last_seen, realm, container, seen_response)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16, ?17, ?18, ?19)",
                params![id, league, kind, location_id, socketed_in, c.name, c.type_line, c.base_type, c.rarity, c.stack_size, c.x, c.y, c.w, c.h, json, at, realm, container, response_id],
            )?;
            tx.execute(
                "INSERT INTO item_events (response_id, at, item_id, kind, to_location) VALUES (?1, ?2, ?3, 'added', ?4)",
                params![response_id, at, id, to],
            )?;
            ingest.added += 1;
        }
        Some(Previous {
            realm: old_realm,
            league: old_league,
            kind: old_kind,
            location_id: old_loc,
            json: old_json,
            removed_at,
            container: old_container,
        }) => {
            let from = address(
                old_kind,
                old_realm,
                (old_kind == "stash")
                    .then_some(old_league.as_deref())
                    .flatten(),
                old_loc,
            );
            // A move is a change of the full coordinate (realm; league too
            // for a stash location), or a reappearance after removal.
            let moved = old_realm != realm
                || old_kind != kind
                || old_loc != location_id
                || (kind == "stash" && old_league.as_deref() != league)
                || removed_at.is_some();
            let old: Value = serde_json::from_str(old_json).unwrap_or(Value::Null);
            // The container is compared explicitly: a helm moving from the
            // character's own equipment to its guardian has identical json
            // (`inventoryId` Helm, x/y 0). A row recorded before the
            // column existed (NULL) is unknown, not different.
            let changed = !same_item(&old, &item)
                || old_container
                    .as_deref()
                    .is_some_and(|previous| previous != container);
            tx.execute(
                "UPDATE items SET league = ?2, location_kind = ?3, location_id = ?4, socketed_in = ?5, name = ?6, type_line = ?7, base_type = ?8,
                        rarity = ?9, stack_size = ?10, x = ?11, y = ?12, w = ?13, h = ?14, json = ?15, last_seen = ?16, removed_at = NULL, realm = ?17,
                        container = ?18, seen_response = ?19
                  WHERE id = ?1",
                params![id, league, kind, location_id, socketed_in, c.name, c.type_line, c.base_type, c.rarity, c.stack_size, c.x, c.y, c.w, c.h, json, at, realm, container, response_id],
            )?;
            if moved {
                tx.execute(
                    "INSERT INTO item_events (response_id, at, item_id, kind, from_location, to_location) VALUES (?1, ?2, ?3, 'moved', ?4, ?5)",
                    params![response_id, at, id, from, to],
                )?;
                ingest.moved += 1;
            }
            if changed {
                tx.execute(
                    "INSERT INTO item_events (response_id, at, item_id, kind, to_location) VALUES (?1, ?2, ?3, 'changed', ?4)",
                    params![response_id, at, id, to],
                )?;
                ingest.changed += 1;
            }
        }
    }
    ingest.items += 1;
    for gem in gems {
        ingest_item(
            tx,
            ingest,
            response_id,
            at,
            realm,
            league,
            container,
            kind,
            location_id,
            Some(&id),
            gem,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, name: &str, x: i64) -> Value {
        json!({ "id": id, "name": name, "typeLine": "Imperial Bow", "baseType": "Imperial Bow", "x": x, "y": 0, "w": 2, "h": 4, "inventoryId": "Stash1", "league": "Standard" })
    }

    fn stash(id: &str, items: Vec<Value>) -> Value {
        json!({ "stash": { "id": id, "name": format!("tab {id}"), "type": "PremiumStash", "index": 0, "metadata": { "colour": "7c5436" }, "items": items } })
    }

    fn stash_ep(id: &str) -> Endpoint {
        Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: id.into(),
            sub: None,
        }
    }

    #[test]
    fn stash_items_are_lifted_and_the_envelope_keeps_the_count() {
        let mut s = Store::open_memory().unwrap();
        let ing = s
            .record(
                &stash_ep("a"),
                &json!({"league":"Standard","id":"a"}),
                200,
                &stash("a", vec![item("i1", "Foo", 0), item("i2", "Bar", 2)]),
                100,
            )
            .unwrap();
        assert_eq!((ing.items, ing.added), (2, 2));
        let env: String = s
            .conn
            .query_row(
                "SELECT envelope FROM responses WHERE id = ?1",
                [ing.response_id],
                |r| r.get(0),
            )
            .unwrap();
        let env: Value = serde_json::from_str(&env).unwrap();
        assert!(env["stash"].get("items").is_none());
        assert_eq!(env["stash"]["_split"]["items"], 2);
        assert_eq!(env["stash"]["metadata"]["colour"], "7c5436");
        let tabs = s.tabs("pc", "Standard").unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!((tabs[0].id.as_str(), tabs[0].item_count), ("a", 2));
        assert_eq!(s.search("foo", None, None, false, 10).unwrap().len(), 1);
    }

    #[test]
    fn move_change_remove_become_events() {
        let mut s = Store::open_memory().unwrap();
        let p = json!({});
        s.record(
            &stash_ep("a"),
            &p,
            200,
            &stash("a", vec![item("i1", "Foo", 0), item("i2", "Bar", 2)]),
            100,
        )
        .unwrap();
        s.record(&stash_ep("b"), &p, 200, &stash("b", vec![]), 100)
            .unwrap();
        // i2 moves a→b (b fetched first: appears in b as moved), i1 changes.
        let ing = s
            .record(
                &stash_ep("b"),
                &p,
                200,
                &stash("b", vec![item("i2", "Bar", 5)]),
                200,
            )
            .unwrap();
        assert_eq!((ing.moved, ing.changed, ing.added), (1, 1, 0));
        let mut i1 = item("i1", "Foo", 0);
        i1["note"] = json!("~b/o 1 chaos");
        let ing = s
            .record(&stash_ep("a"), &p, 200, &stash("a", vec![i1]), 201)
            .unwrap();
        assert_eq!((ing.changed, ing.removed, ing.moved), (1, 0, 0));
        let ev = s.events_since(200, 100).unwrap();
        let kinds: Vec<_> = ev
            .iter()
            .map(|e| (e.item_id.as_str(), e.kind.as_str()))
            .collect();
        assert_eq!(
            kinds,
            vec![("i2", "moved"), ("i2", "changed"), ("i1", "changed")]
        );
        assert_eq!(ev[0].from_location.as_deref(), Some("stash/pc/Standard/a"));
        // Now i1 disappears from a entirely.
        let ing = s
            .record(&stash_ep("a"), &p, 200, &stash("a", vec![]), 300)
            .unwrap();
        assert_eq!(ing.removed, 1);
        assert_eq!(s.item("i1").unwrap().unwrap().removed_at, Some(300));
        assert!(s.search("foo", None, None, false, 10).unwrap().is_empty());
        assert_eq!(s.search("foo", None, None, true, 10).unwrap().len(), 1);
        // And comes back: a move event from its removed state.
        let ing = s
            .record(
                &stash_ep("b"),
                &p,
                200,
                &stash("b", vec![item("i2", "Bar", 5), item("i1", "Foo", 0)]),
                400,
            )
            .unwrap();
        assert_eq!((ing.moved, ing.changed), (1, 1));
        assert_eq!(s.status().unwrap().items, 2);
    }

    #[test]
    fn an_item_without_an_id_is_malformed_and_poisons_nothing() {
        let mut s = Store::open_memory().unwrap();
        let p = json!({});
        s.record(
            &stash_ep("a"),
            &p,
            200,
            &stash("a", vec![item("i1", "Foo", 0)]),
            100,
        )
        .unwrap();
        // A fetch whose items lack ids is refused whole: the held item is
        // neither removed nor half-replaced, and no response row lands.
        let err = s
            .record(
                &stash_ep("a"),
                &p,
                200,
                &stash("a", vec![json!({ "name": "NoId", "typeLine": "?" })]),
                200,
            )
            .unwrap_err();
        assert!(err.downcast_ref::<MalformedBody>().is_some(), "{err:#}");
        assert!(s.item("i1").unwrap().unwrap().removed_at.is_none());
        assert_eq!(s.status().unwrap().responses, 1);
        // Same for a socketed gem without an id.
        let mut bow = item("bow", "Bow", 0);
        bow["socketedItems"] = json!([{ "typeLine": "Nameless" }]);
        let err = s
            .record(&stash_ep("a"), &p, 200, &stash("a", vec![bow]), 300)
            .unwrap_err();
        assert!(err.downcast_ref::<MalformedBody>().is_some(), "{err:#}");
        assert!(s.item("i1").unwrap().unwrap().removed_at.is_none());
    }

    #[test]
    fn volatile_fields_are_not_changes() {
        let mut s = Store::open_memory().unwrap();
        let p = json!({});
        let mut a = item("i1", "Veiled", 0);
        a["veiledMods"] = json!(["Prefix04"]);
        s.record(&stash_ep("a"), &p, 200, &stash("a", vec![a.clone()]), 1)
            .unwrap();
        a["veiledMods"] = json!(["Prefix01"]);
        let ing = s
            .record(&stash_ep("a"), &p, 200, &stash("a", vec![a]), 2)
            .unwrap();
        assert_eq!(ing.changed, 0);
    }

    #[test]
    fn socketed_gems_are_rows_of_their_own() {
        let mut s = Store::open_memory().unwrap();
        let mut bow = item("bow", "Bow", 0);
        bow["socketedItems"] = json!([{ "id": "gem1", "typeLine": "Determination", "baseType": "Determination", "socket": 0 }]);
        let ing = s
            .record(&stash_ep("a"), &json!({}), 200, &stash("a", vec![bow]), 1)
            .unwrap();
        assert_eq!(ing.items, 2);
        let gem = s.item("gem1").unwrap().unwrap();
        assert_eq!(gem.socketed_in.as_deref(), Some("bow"));
        assert_eq!(gem.location_id, "a");
        assert!(
            s.item("bow")
                .unwrap()
                .unwrap()
                .json
                .get("socketedItems")
                .is_none()
        );
        assert_eq!(
            s.search("determination", None, None, false, 10)
                .unwrap()
                .len(),
            1
        );
        // Re-fetch with the gem unsocketed and gone: the gem is removed, the bow unchanged.
        let ing = s
            .record(
                &stash_ep("a"),
                &json!({}),
                200,
                &stash("a", vec![item("bow", "Bow", 0)]),
                2,
            )
            .unwrap();
        assert_eq!((ing.removed, ing.changed), (1, 0));
    }

    #[test]
    fn stash_list_and_substashes_fill_the_tabs_table() {
        let mut s = Store::open_memory().unwrap();
        let list = json!({ "stashes": [
            { "id": "f1", "name": "Folder", "type": "Folder", "index": 0, "children": [ { "id": "c1", "name": "In folder", "type": "PremiumStash", "index": 1 } ] },
            { "id": "m1", "name": "Maps", "type": "MapStash", "index": 2 },
            { "id": "gone", "name": "Old", "type": "PremiumStash", "index": 3 },
        ]});
        let ep = Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        };
        s.record(&ep, &json!({}), 200, &list, 10).unwrap();
        let tabs = s.tabs("pc", "Standard").unwrap();
        assert_eq!(
            tabs.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
            vec!["f1", "c1", "m1", "gone"]
        );
        assert_eq!(tabs[1].parent.as_deref(), Some("f1"));
        // Fetching the map tab lists substash stubs; fetching one substash lands items under it.
        let map = json!({ "stash": { "id": "m1", "name": "Maps", "type": "MapStash", "items": [], "children": [
            { "id": "s1", "name": "", "type": "MapStash", "parent": "m1", "metadata": { "items": 1, "map": { "name": "Tier 16" } } } ] } });
        s.record(&stash_ep("m1"), &json!({}), 200, &map, 11)
            .unwrap();
        let sub = Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "m1".into(),
            sub: Some("s1".into()),
        };
        let ing = s.record(&sub, &json!({}), 200, &json!({ "stash": { "id": "s1", "name": "", "type": "MapStash", "parent": "m1", "items": [item("map1", "", 0)] } }), 12).unwrap();
        assert_eq!(ing.added, 1);
        let tabs = s.tabs("pc", "Standard").unwrap();
        let s1 = tabs.iter().find(|t| t.id == "s1").unwrap();
        assert_eq!((s1.parent.as_deref(), s1.item_count), (Some("m1"), 1));
        // The next list drops "gone": removed, but the substash (never listed) survives.
        let list2 = json!({ "stashes": [ { "id": "f1", "name": "Folder", "type": "Folder", "index": 0, "children": [ { "id": "c1", "name": "In folder", "type": "PremiumStash", "index": 1 } ] }, { "id": "m1", "name": "Maps", "type": "MapStash", "index": 2 } ] });
        s.record(&ep, &json!({}), 200, &list2, 20).unwrap();
        let ids: Vec<_> = s
            .tabs("pc", "Standard")
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["f1", "c1", "m1", "s1"]);
    }

    #[test]
    fn character_arrays_share_one_location() {
        let mut s = Store::open_memory().unwrap();
        let body = json!({ "character": { "id": "c-hero", "name": "Hero", "league": "Standard", "class": "Witch", "level": 90,
            "inventory": [ item("inv1", "Bag", 0) ], "equipment": [ item("eq1", "Helm", 0) ], "jewels": [] } });
        let ep = Endpoint::Character {
            realm: "pc".into(),
            name: "Hero".into(),
        };
        let ing = s
            .record(&ep, &json!({"name":"Hero"}), 200, &body, 1)
            .unwrap();
        assert_eq!(ing.added, 2);
        let inv = s.item("inv1").unwrap().unwrap();
        assert_eq!(
            (inv.location_kind.as_str(), inv.location_id.as_str()),
            ("character", "c-hero")
        );
        assert_eq!(inv.container.as_deref(), Some("inventory"));
        let ing = s.record(&ep, &json!({"name":"Hero"}), 200, &json!({ "character": { "id": "c-hero", "name": "Hero", "league": "Standard", "inventory": [ item("inv1", "Bag", 0) ], "equipment": [] } }), 2).unwrap();
        assert_eq!(ing.removed, 1);
        let json: String = s
            .conn
            .query_row("SELECT json FROM characters WHERE id = 'c-hero'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(json.contains("\"_split\"") && !json.contains("\"inv1\""));
    }

    #[test]
    fn leagues_profile_characters_list() {
        let mut s = Store::open_memory().unwrap();
        s.record(
            &Endpoint::Leagues,
            &json!({}),
            200,
            &json!({ "leagues": [ { "id": "Standard" }, { "id": "Hardcore" } ] }),
            1,
        )
        .unwrap();
        s.record(
            &Endpoint::Profile,
            &json!({}),
            200,
            &json!({ "uuid": "u-1", "name": "tom" }),
            1,
        )
        .unwrap();
        s.record(&Endpoint::Characters { realm: "pc".into() }, &json!({}), 200, &json!({ "characters": [ { "id": "c-a", "name": "A", "class": "Witch", "level": 3, "league": "Standard" } ] }), 1).unwrap();
        let st = s.status().unwrap();
        assert_eq!((st.leagues, st.characters, st.responses), (2, 1, 3));
        s.record(
            &Endpoint::Characters { realm: "pc".into() },
            &json!({}),
            200,
            &json!({ "characters": [] }),
            2,
        )
        .unwrap();
        assert_eq!(s.status().unwrap().characters, 0);
    }

    #[test]
    fn characters_query_lists_live_rows_with_item_counts() {
        let mut s = Store::open_memory().unwrap();
        s.record(
            &Endpoint::Characters { realm: "pc".into() },
            &json!({}),
            200,
            &json!({ "characters": [
            { "id": "c-hero", "name": "Hero", "class": "Witch", "level": 90, "league": "Standard" },
            { "id": "c-mule", "name": "Mule", "class": "Scion", "level": 3, "league": "Hardcore" },
        ] }),
            1,
        )
        .unwrap();
        // Fetch fills fetched_at and lifts items into the character location.
        s.record(
            &Endpoint::Character { realm: "pc".into(),
                name: "Hero".into(),
            },
            &json!({"name":"Hero"}),
            200,
            &json!({ "character": { "id": "c-hero", "name": "Hero", "league": "Standard", "class": "Witch", "level": 90,
                "equipment": [ item("eq1", "Bow", 0) ], "inventory": [ item("inv1", "Bag", 1) ] } }),
            2,
        )
        .unwrap();
        let all = s.characters(None, None).unwrap();
        assert_eq!(
            all.iter()
                .map(|c| (c.name.as_str(), c.item_count, c.fetched_at.is_some()))
                .collect::<Vec<_>>(),
            vec![("Mule", 0, false), ("Hero", 2, true)]
        );
        let std_only = s.characters(None, Some("Standard")).unwrap();
        assert_eq!(std_only.len(), 1);
        assert_eq!(std_only[0].name, "Hero");
        // A character no longer listed disappears from the query.
        s.record(
            &Endpoint::Characters { realm: "pc".into() },
            &json!({}),
            200,
            &json!({ "characters": [
            { "id": "c-hero", "name": "Hero", "class": "Witch", "level": 90, "league": "Standard" },
        ] }),
            3,
        )
        .unwrap();
        assert_eq!(s.characters(None, None).unwrap().len(), 1);
    }

    #[test]
    fn rebuild_reextracts_columns() {
        let mut s = Store::open_memory().unwrap();
        s.record(
            &stash_ep("a"),
            &json!({}),
            200,
            &stash("a", vec![item("i1", "Foo", 0)]),
            1,
        )
        .unwrap();
        s.conn
            .execute("UPDATE items SET name = 'wrong'", [])
            .unwrap();
        assert_eq!(s.rebuild().unwrap(), 1);
        assert_eq!(s.item("i1").unwrap().unwrap().name, "Foo");
    }

    #[test]
    fn intent_survives_fact_removal_and_surfaces_as_orphaned() {
        let mut s = Store::open_memory().unwrap();
        let mut a = Annotations::open_memory().unwrap();
        s.record(
            &stash_ep("a"),
            &json!({}),
            200,
            &stash("a", vec![item("i1", "Foo", 0)]),
            100,
        )
        .unwrap();
        a.put("item", "i1", "buyout", &json!({"price": "1 divine"}), None)
            .unwrap();
        assert!(s.orphaned_item_annotations(&a).unwrap().is_empty());
        // The item disappears from its tab: the fact side records a removal
        // and touches no intent — the annotation stays, now orphaned.
        s.record(&stash_ep("a"), &json!({}), 200, &stash("a", vec![]), 200)
            .unwrap();
        let orphaned = s.orphaned_item_annotations(&a).unwrap();
        assert_eq!(orphaned.len(), 1);
        assert_eq!(orphaned[0].key, "i1");
        assert_eq!(a.get("item", "i1", "buyout").unwrap().unwrap().revision, 1);
        // The item comes back: the same annotation is attached again.
        s.record(
            &stash_ep("a"),
            &json!({}),
            200,
            &stash("a", vec![item("i1", "Foo", 0)]),
            300,
        )
        .unwrap();
        assert!(s.orphaned_item_annotations(&a).unwrap().is_empty());
        // An annotation on an item this store never saw is orphaned too.
        a.put("item", "ghost", "note", &json!("?"), None).unwrap();
        assert_eq!(s.orphaned_item_annotations(&a).unwrap().len(), 1);
    }

    #[test]
    fn files_from_before_the_listing_columns_are_migrated_in_place() {
        let dir =
            std::env::temp_dir().join(format!("acq-store-mig-{}-{}", std::process::id(), now()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("old.db");
        {
            // A tabs table as the schema had it before listed_json /
            // listed_response, with one live row.
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE tabs (
                    league TEXT NOT NULL, id TEXT NOT NULL, parent TEXT, name TEXT, type TEXT,
                    idx INTEGER, json TEXT NOT NULL, listed_at INTEGER, fetched_at INTEGER,
                    removed_at INTEGER, PRIMARY KEY (league, id));
                 INSERT INTO tabs (league, id, name, type, idx, json, listed_at)
                 VALUES ('Standard', 'old1', 'Old', 'PremiumStash', 0, '{}', 5);
                 CREATE TABLE items (
                    id TEXT PRIMARY KEY, league TEXT, location_kind TEXT NOT NULL, location_id TEXT NOT NULL,
                    socketed_in TEXT, name TEXT, type_line TEXT, base_type TEXT, rarity TEXT, stack_size INTEGER,
                    x INTEGER, y INTEGER, w INTEGER, h INTEGER, json TEXT NOT NULL,
                    first_seen INTEGER NOT NULL, last_seen INTEGER NOT NULL, removed_at INTEGER);
                 INSERT INTO items (id, league, location_kind, location_id, name, type_line, base_type, json, first_seen, last_seen)
                 VALUES ('old-item', 'Standard', 'stash', 'old1', 'Foo', 'Imperial Bow', 'Imperial Bow', '{\"id\":\"old-item\"}', 5, 5);
                 CREATE TABLE characters (
                    name TEXT PRIMARY KEY, league TEXT, class TEXT, level INTEGER, json TEXT NOT NULL,
                    listed_at INTEGER, fetched_at INTEGER, removed_at INTEGER);
                 INSERT INTO characters (name, league, json, listed_at, fetched_at)
                 VALUES ('Hero', 'Standard', '{\"id\":\"c-hero\",\"name\":\"Hero\",\"_split\":{}}', 5, 6);
                 INSERT INTO characters (name, league, json, listed_at) VALUES ('Ghost', 'Standard', '{\"name\":\"Ghost\"}', 5);
                 INSERT INTO items (id, league, location_kind, location_id, name, type_line, base_type, json, first_seen, last_seen)
                 VALUES ('hero-item', 'Standard', 'character', 'Hero', 'Bar', 'Bow', 'Bow', '{\"id\":\"hero-item\"}', 6, 6);
                 INSERT INTO items (id, league, location_kind, location_id, name, type_line, base_type, json, first_seen, last_seen)
                 VALUES ('ghost-item', 'Standard', 'character', 'Ghost', 'Baz', 'Bow', 'Bow', '{\"id\":\"ghost-item\"}', 6, 6);
                 CREATE TABLE responses (id INTEGER PRIMARY KEY, endpoint TEXT NOT NULL, params TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL, status INTEGER NOT NULL, envelope TEXT NOT NULL, item_count INTEGER NOT NULL);
                 INSERT INTO responses (id, endpoint, params, fetched_at, status, envelope, item_count)
                 VALUES (7, 'characters', '{}', 5, 200, '{\"characters\":[{\"id\":\"c-hero\",\"name\":\"Hero\",\"league\":\"Standard\"}]}', 0);",
            )
            .unwrap();
        }
        let mut s = Store::open(&path).unwrap();
        let tabs = s.tabs("pc", "Standard").unwrap();
        assert_eq!(
            (tabs[0].id.as_str(), tabs[0].realm.as_str()),
            ("old1", "pc")
        );
        // Pre-realm items are pc too — not null: a realm-filtered search
        // and the item's own row both say so (review finding 2026-09-02).
        assert_eq!(tabs[0].item_count, 1);
        assert_eq!(s.item("old-item").unwrap().unwrap().realm, "pc");
        assert_eq!(
            s.search("foo", Some("pc"), None, false, 10).unwrap().len(),
            1
        );
        assert_eq!(
            s.item("old-item").unwrap().unwrap().container.as_deref(),
            Some("items")
        );
        // v4: characters rekeyed by the id in their json — the row without
        // one is dropped and its items retired; the other's items move to
        // its id (container unknown: not in the json, so left NULL).
        let chars = s.characters(None, None).unwrap();
        assert_eq!(chars.len(), 1);
        let hero = &chars[0];
        assert_eq!(
            (
                hero.id.as_str(),
                hero.name.as_str(),
                hero.realm.as_str(),
                hero.fetched_at,
                hero.item_count
            ),
            ("c-hero", "Hero", "pc", Some(6), 1)
        );
        let hero_item = s.item("hero-item").unwrap().unwrap();
        assert_eq!(
            (
                hero_item.location_id.as_str(),
                hero_item.container.as_deref()
            ),
            ("c-hero", None)
        );
        assert!(s.item("ghost-item").unwrap().unwrap().removed_at.is_some());
        // Membership is re-stamped from the latest listing on record, so
        // a basis a planner cites has its rows stamped to it.
        let (listed_response, listed_json): (Option<i64>, Option<String>) = s
            .conn
            .query_row(
                "SELECT listed_response, listed_json FROM characters WHERE id = 'c-hero'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(listed_response, Some(7));
        assert!(listed_json.unwrap().contains("\"Hero\""));
        // The migrated file is stamped with the current schema version.
        let v: i64 = s
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, FACT_SCHEMA_VERSION);
        // The next listing stamps membership per response id and retires
        // the pre-migration row it dropped.
        s.record(
            &Endpoint::Stashes { realm: "pc".into(),
                league: "Standard".into(),
            },
            &json!({}),
            200,
            &json!({ "stashes": [ { "id": "new1", "name": "New", "type": "PremiumStash", "index": 0 } ] }),
            10,
        )
        .unwrap();
        let ids: Vec<String> = s
            .tabs("pc", "Standard")
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["new1"]);
        // A file stamped newer than this build is refused, not guessed at.
        s.conn.pragma_update(None, "user_version", 99).unwrap();
        drop(s);
        let err = Store::open(&path).err().unwrap();
        assert_eq!(
            err.downcast_ref::<SchemaTooNew>(),
            Some(&SchemaTooNew {
                found: 99,
                supported: FACT_SCHEMA_VERSION
            })
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn endpoint_from_job_kinds() {
        assert_eq!(
            Endpoint::from_job("stash", &json!({"league":"Standard","id":"a","sub":"b"})),
            Some(Endpoint::Stash {
                realm: "pc".into(),
                league: "Standard".into(),
                id: "a".into(),
                sub: Some("b".into())
            })
        );
        assert_eq!(Endpoint::from_job("stash", &json!({})), None);
        assert_eq!(Endpoint::from_job("probe", &json!({})), None);
        assert_eq!(
            Endpoint::from_job("stashes", &json!({})),
            Some(Endpoint::Stashes {
                realm: "pc".into(),
                league: "Standard".into()
            })
        );
    }

    #[test]
    fn a_large_tab_ingests_in_one_transaction_quickly() {
        let mut s = Store::open_memory().unwrap();
        let items: Vec<Value> = (0..10_000)
            .map(|i| item(&format!("i{i}"), "Chaos Orb", i % 24))
            .collect();
        let t = std::time::Instant::now();
        let ing = s
            .record(
                &stash_ep("quad"),
                &json!({}),
                200,
                &stash("quad", items.clone()),
                1,
            )
            .unwrap();
        let first = t.elapsed();
        assert_eq!(ing.added, 10_000);
        let t = std::time::Instant::now();
        let ing = s
            .record(&stash_ep("quad"), &json!({}), 200, &stash("quad", items), 2)
            .unwrap();
        let second = t.elapsed();
        assert_eq!((ing.added, ing.changed, ing.removed), (0, 0, 0));
        eprintln!("10k items: first ingest {first:?}, unchanged re-ingest {second:?}");
        assert!(
            second < std::time::Duration::from_secs(5),
            "re-ingest took {second:?}"
        );
    }

    /// Realm is the coordinate above league (CONTEXT.md, 2026-09-02): the
    /// same league and tab id under two realms are two rows, a listing
    /// retires only its own realm's tabs, items carry the request's
    /// realm, and a character listing never retires another realm's.
    #[test]
    fn realms_keep_the_same_league_and_id_apart() {
        let mut s = Store::open_memory().unwrap();
        let list = |realm: &str| Endpoint::Stashes {
            realm: realm.into(),
            league: "Standard".into(),
        };
        let body = json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] });
        s.record(&list("pc"), &json!({}), 200, &body, 10).unwrap();
        s.record(
            &list("xbox"),
            &json!({ "realm": "xbox", "league": "Standard" }),
            200,
            &body,
            11,
        )
        .unwrap();
        assert_eq!(s.tabs("pc", "Standard").unwrap().len(), 1);
        assert_eq!(s.tabs("xbox", "Standard").unwrap()[0].realm, "xbox");
        // An empty xbox listing retires the xbox row only.
        s.record(
            &list("xbox"),
            &json!({ "realm": "xbox" }),
            200,
            &json!({ "stashes": [] }),
            12,
        )
        .unwrap();
        assert!(s.tabs("xbox", "Standard").unwrap().is_empty());
        assert_eq!(s.tabs("pc", "Standard").unwrap()[0].id, "t1");
        // Items are stamped with the request's realm; search filters on it.
        let ing = s
            .record(
                &Endpoint::Stash {
                    realm: "sony".into(),
                    league: "Standard".into(),
                    id: "s9".into(),
                    sub: None,
                },
                &json!({ "realm": "sony", "league": "Standard", "id": "s9" }),
                200,
                &json!({ "stash": { "id": "s9", "name": "Sony", "type": "PremiumStash", "items": [ item("i1", "Foo", 0) ] } }),
                13,
            )
            .unwrap();
        assert_eq!(ing.added, 1);
        assert_eq!(s.item("i1").unwrap().unwrap().realm, "sony");
        assert_eq!(
            s.search("foo", Some("sony"), None, false, 10)
                .unwrap()
                .len(),
            1
        );
        assert!(
            s.search("foo", Some("pc"), None, false, 10)
                .unwrap()
                .is_empty()
        );
        // A poe2 character listing leaves pc's characters alone.
        s.record(
            &Endpoint::Characters { realm: "pc".into() },
            &json!({}),
            200,
            &json!({ "characters": [ { "id": "c-a", "name": "A", "league": "Standard" } ] }),
            20,
        )
        .unwrap();
        s.record(
            &Endpoint::Characters {
                realm: "poe2".into(),
            },
            &json!({ "realm": "poe2" }),
            200,
            &json!({ "characters": [ { "id": "c-b", "name": "B", "league": "Standard" } ] }),
            21,
        )
        .unwrap();
        let names: Vec<(String, String)> = s
            .characters(None, None)
            .unwrap()
            .into_iter()
            .map(|c| (c.realm, c.name))
            .collect();
        assert_eq!(
            names,
            vec![("pc".into(), "A".into()), ("poe2".into(), "B".into())]
        );
        assert_eq!(s.characters(Some("poe2"), None).unwrap().len(), 1);
    }

    /// Identity is the id; the name is the address (CONTEXT.md,
    /// 2026-09-02): a rename keeps the row and its items (no false moves),
    /// a deleted-and-recreated name is a new row that has never been
    /// fetched with the old one retired, the listing owns `league`, and a
    /// fetched body without an id has no row to land on.
    #[test]
    fn the_character_key_is_the_id_and_the_name_is_the_address() {
        let mut s = Store::open_memory().unwrap();
        let list = Endpoint::Characters { realm: "pc".into() };
        let fetch = |name: &str| Endpoint::Character {
            realm: "pc".into(),
            name: name.into(),
        };
        s.record(&list, &json!({}), 200, &json!({ "characters": [ { "id": "c1", "name": "Hero", "league": "Standard", "level": 90 } ] }), 10).unwrap();
        let ing = s
            .record(
                &fetch("Hero"),
                &json!({ "name": "Hero" }),
                200,
                &json!({ "character": { "id": "c1", "name": "Hero", "league": "Standard", "equipment": [ item("eq1", "Helm", 0) ] } }),
                11,
            )
            .unwrap();
        assert_eq!(ing.added, 1);
        // Renamed: the listing names the same id under a new name.
        s.record(&list, &json!({}), 200, &json!({ "characters": [ { "id": "c1", "name": "Champion", "league": "Standard", "level": 91 } ] }), 20).unwrap();
        let rows = s.characters(None, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (
                rows[0].id.as_str(),
                rows[0].name.as_str(),
                rows[0].item_count,
                rows[0].fetched_at
            ),
            ("c1", "Champion", 1, Some(11))
        );
        // Fetching by the new name lands on the same row; nothing moved.
        let ing = s
            .record(
                &fetch("Champion"),
                &json!({ "name": "Champion" }),
                200,
                &json!({ "character": { "id": "c1", "name": "Champion", "league": "Standard", "equipment": [ item("eq1", "Helm", 0) ] } }),
                21,
            )
            .unwrap();
        assert_eq!(
            (ing.added, ing.moved, ing.changed, ing.removed),
            (0, 0, 0, 0)
        );
        assert_eq!(s.item("eq1").unwrap().unwrap().location_id, "c1");
        // A fetch authorized under the old address that lands after the
        // rename (separate routes, concurrent sends) must not roll the
        // address back: the listing owns name/class/level once it has
        // named the row; the body's own say stays in json.
        s.record(
            &fetch("Hero"),
            &json!({ "name": "Hero" }),
            200,
            &json!({ "character": { "id": "c1", "name": "Hero", "league": "Standard", "level": 90, "equipment": [ item("eq1", "Helm", 0) ] } }),
            22,
        )
        .unwrap();
        let row = &s.characters(None, None).unwrap()[0];
        assert_eq!((row.name.as_str(), row.level), ("Champion", Some(91)));
        let json: String = s
            .conn
            .query_row("SELECT json FROM characters WHERE id = 'c1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(json.contains("\"Hero\""));
        // The listing owns league: a later fetch saying otherwise does not
        // move the coverage coordinate (its body keeps its own say).
        s.record(
            &list,
            &json!({}),
            200,
            &json!({ "characters": [ { "id": "c1", "name": "Champion", "league": "Hardcore" } ] }),
            30,
        )
        .unwrap();
        s.record(
            &fetch("Champion"),
            &json!({ "name": "Champion" }),
            200,
            &json!({ "character": { "id": "c1", "name": "Champion", "league": "Standard", "equipment": [ item("eq1", "Helm", 0) ] } }),
            31,
        )
        .unwrap();
        assert_eq!(
            s.characters(None, None).unwrap()[0].league.as_deref(),
            Some("Hardcore")
        );
        // …and its items take the row's coordinate, not the body's.
        assert_eq!(
            s.item("eq1").unwrap().unwrap().league.as_deref(),
            Some("Hardcore")
        );
        // Deleted and recreated under the old name: a new id, never
        // fetched, and the old row retired — a name-keyed store would have
        // inherited c1's freshness and never fetched the new one.
        s.record(
            &fetch("Champion"),
            &json!({ "name": "Champion" }),
            200,
            &json!({ "character": { "id": "c1", "name": "Champion", "league": "Standard", "equipment": [ item("eq1", "Helm", 0) ] } }),
            35,
        )
        .unwrap();
        let ing = s
            .record(
                &list,
                &json!({}),
                200,
                &json!({ "characters": [ { "id": "c2", "name": "Hero", "league": "Standard" } ] }),
                40,
            )
            .unwrap();
        let rows = s.characters(None, None).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            (rows[0].id.as_str(), rows[0].fetched_at, rows[0].item_count),
            ("c2", None, 0)
        );
        // The retired character's items went with it: retired, with a
        // removed event, and no longer live in search.
        assert_eq!(ing.removed, 1);
        assert!(s.item("eq1").unwrap().unwrap().removed_at.is_some());
        assert!(s.search("helm", None, None, false, 10).unwrap().is_empty());
        assert_eq!(s.search("helm", None, None, true, 10).unwrap().len(), 1);
        let removed_events: i64 = s
            .conn
            .query_row(
                "SELECT count(*) FROM item_events WHERE item_id = 'eq1' AND kind = 'removed' AND from_location = 'character/pc/c1' AND at = 40",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(removed_events, 1);
        let err = s
            .record(
                &fetch("Hero"),
                &json!({ "name": "Hero" }),
                200,
                &json!({ "character": { "name": "Hero" } }),
                41,
            )
            .unwrap_err();
        assert!(err.downcast_ref::<MalformedBody>().is_some(), "{err:#}");
    }

    /// Every item row records the array it came from: a guardian's helm
    /// and the character's own are distinguishable only by that, and a
    /// move between arrays is a `changed` event even with identical json.
    #[test]
    fn container_is_an_ingest_fact_and_moving_between_arrays_is_a_change() {
        let mut s = Store::open_memory().unwrap();
        let ep = Endpoint::Character {
            realm: "pc".into(),
            name: "Hero".into(),
        };
        let helm = |id: &str| json!({ "id": id, "name": "", "typeLine": "Iron Hat", "baseType": "Iron Hat", "inventoryId": "Helm", "x": 0, "y": 0 });
        s.record(
            &ep,
            &json!({ "name": "Hero" }),
            200,
            &json!({ "character": { "id": "c1", "name": "Hero",
                "equipment": [ helm("own") ], "guardian": [ helm("pet") ], "skills": [ item("sk", "Skill", 0) ] } }),
            1,
        )
        .unwrap();
        let own = s.item("own").unwrap().unwrap();
        let pet = s.item("pet").unwrap().unwrap();
        assert_eq!(own.container.as_deref(), Some("equipment"));
        assert_eq!(pet.container.as_deref(), Some("guardian"));
        assert_eq!(
            s.item("sk").unwrap().unwrap().container.as_deref(),
            Some("skills")
        );
        assert_eq!(own.location_id, pet.location_id);
        // Swap the helms between the arrays: json identical, containers
        // differ — changed, not moved (one location).
        let ing = s
            .record(
                &ep,
                &json!({ "name": "Hero" }),
                200,
                &json!({ "character": { "id": "c1", "name": "Hero",
                    "equipment": [ helm("pet") ], "guardian": [ helm("own") ], "skills": [ item("sk", "Skill", 0) ] } }),
                2,
            )
            .unwrap();
        assert_eq!((ing.moved, ing.changed, ing.removed), (0, 2, 0));
        assert_eq!(
            s.item("own").unwrap().unwrap().container.as_deref(),
            Some("guardian")
        );
        // A stash item's container is `items`.
        s.record(
            &stash_ep("t1"),
            &json!({}),
            200,
            &stash("t1", vec![item("i1", "Foo", 0)]),
            3,
        )
        .unwrap();
        assert_eq!(
            s.item("i1").unwrap().unwrap().container.as_deref(),
            Some("items")
        );
    }

    /// The drift tripwire: an item-shaped array the store does not lift
    /// is counted into the envelope and the status — never a failure, and
    /// never an item row. Arrays of non-items are left alone.
    #[test]
    fn an_unknown_item_array_trips_the_drift_counter_without_failing() {
        let mut s = Store::open_memory().unwrap();
        let ep = Endpoint::Character {
            realm: "pc".into(),
            name: "Hero".into(),
        };
        let ing = s
            .record(
                &ep,
                &json!({ "name": "Hero" }),
                200,
                &json!({ "character": { "id": "c1", "name": "Hero",
                    "equipment": [ item("eq1", "Helm", 0) ],
                    "pets": [ item("p1", "Cat", 0), item("p2", "Dog", 0) ],
                    "passives": [1, 2, 3], "notes": [] } }),
                1,
            )
            .unwrap();
        assert_eq!((ing.items, ing.unlifted), (1, 2));
        assert!(s.item("p1").unwrap().is_none());
        assert_eq!(s.status().unwrap().unlifted_items, 2);
        let json: String = s
            .conn
            .query_row("SELECT json FROM characters WHERE id = 'c1'", [], |r| {
                r.get(0)
            })
            .unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["_unlifted"], json!({ "pets": 2 }));
        assert_eq!(v["_split"], json!({ "equipment": 1 }));
        assert_eq!(v["passives"], json!([1, 2, 3]));
        // The count is the latest fetch's, per live character — a repeat
        // does not double it, and a fetch without the array clears it.
        s.record(
            &ep,
            &json!({ "name": "Hero" }),
            200,
            &json!({ "character": { "id": "c1", "name": "Hero",
                "equipment": [ item("eq1", "Helm", 0) ], "pets": [ item("p1", "Cat", 0), item("p2", "Dog", 0) ] } }),
            2,
        )
        .unwrap();
        assert_eq!(s.status().unwrap().unlifted_items, 2);
        s.record(
            &ep,
            &json!({ "name": "Hero" }),
            200,
            &json!({ "character": { "id": "c1", "name": "Hero", "equipment": [ item("eq1", "Helm", 0) ] } }),
            3,
        )
        .unwrap();
        assert_eq!(s.status().unwrap().unlifted_items, 0);
    }

    /// Membership belongs to the listing: a fetch that lands after a newer
    /// listing retired its location records the body but revives nothing
    /// — the row stays retired and its item facts are withheld — until a
    /// listing names it again; then the next fetch lands them.
    #[test]
    fn a_late_fetch_never_revives_a_location_a_listing_retired() {
        let mut s = Store::open_memory().unwrap();
        let list = Endpoint::Characters { realm: "pc".into() };
        let fetch = Endpoint::Character {
            realm: "pc".into(),
            name: "Hero".into(),
        };
        let body = json!({ "character": { "id": "c1", "name": "Hero", "league": "Standard", "equipment": [ item("eq1", "Helm", 0) ] } });
        let named =
            json!({ "characters": [ { "id": "c1", "name": "Hero", "league": "Standard" } ] });
        s.record(&list, &json!({}), 200, &named, 10).unwrap();
        s.record(&fetch, &json!({ "name": "Hero" }), 200, &body, 11)
            .unwrap();
        s.record(&list, &json!({}), 200, &json!({ "characters": [] }), 20)
            .unwrap();
        let ing = s
            .record(&fetch, &json!({ "name": "Hero" }), 200, &body, 21)
            .unwrap();
        assert_eq!(
            (ing.items, ing.added, ing.moved, ing.withheld),
            (0, 0, 0, Some(1))
        );
        assert!(s.characters(None, None).unwrap().is_empty());
        assert!(s.item("eq1").unwrap().unwrap().removed_at.is_some());
        assert!(s.search("helm", None, None, false, 10).unwrap().is_empty());
        // The row was not touched (its fetched_at is still the live
        // fetch's), and the whole body — arrays included — sits verbatim
        // on the response row, marked withheld.
        let fetched_at: i64 = s
            .conn
            .query_row(
                "SELECT fetched_at FROM characters WHERE id = 'c1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fetched_at, 11);
        let (held, envelope): (i64, String) = s
            .conn
            .query_row(
                "SELECT withheld, envelope FROM responses WHERE endpoint = 'character' AND fetched_at = 21",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(held, 1);
        let envelope: Value = serde_json::from_str(&envelope).unwrap();
        assert_eq!(envelope["character"]["equipment"][0]["id"], "eq1");
        assert!(envelope["character"].get("_split").is_none());
        let st = s.status().unwrap();
        assert_eq!((st.withheld_responses, st.withheld_items), (1, 1));
        // A listing naming it again revives the row — with no fetch on
        // record, since its facts were retired with it — and the next
        // fetch lands the items (a reappearance is a move).
        s.record(&list, &json!({}), 200, &named, 30).unwrap();
        let revived = &s.characters(None, None).unwrap()[0];
        assert_eq!((revived.fetched_at, revived.item_count), (None, 0));
        let ing = s
            .record(&fetch, &json!({ "name": "Hero" }), 200, &body, 31)
            .unwrap();
        assert_eq!((ing.items, ing.moved, ing.withheld), (1, 1, None));
        let row = &s.characters(None, None).unwrap()[0];
        assert_eq!((row.fetched_at, row.item_count), (Some(31), 1));
        // The same rule for a tab.
        let listing = Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        };
        let one = json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] });
        s.record(&listing, &json!({}), 200, &one, 40).unwrap();
        s.record(
            &stash_ep("t1"),
            &json!({}),
            200,
            &stash("t1", vec![item("i1", "Foo", 0)]),
            41,
        )
        .unwrap();
        s.record(&listing, &json!({}), 200, &json!({ "stashes": [] }), 50)
            .unwrap();
        let ing = s
            .record(
                &stash_ep("t1"),
                &json!({}),
                200,
                &stash("t1", vec![item("i1", "Foo", 0)]),
                51,
            )
            .unwrap();
        assert_eq!((ing.items, ing.withheld), (0, Some(1)));
        assert!(s.tabs("pc", "Standard").unwrap().is_empty());
        assert!(s.item("i1").unwrap().unwrap().removed_at.is_some());
        // Revived by a listing: no fetch on record, so the next plan
        // fetches it.
        s.record(&listing, &json!({}), 200, &one, 60).unwrap();
        let revived = &s.tabs("pc", "Standard").unwrap()[0];
        assert_eq!((revived.fetched_at, revived.item_count), (None, 0));
    }

    /// A location is its full coordinate: the same tab id under two realms
    /// is two locations, so a fetch of one never retires the other's items
    /// and counts stay apart; an item that turns up under the other realm's
    /// tab has moved.
    #[test]
    fn the_same_tab_id_under_two_realms_is_two_locations() {
        let mut s = Store::open_memory().unwrap();
        let ep = |realm: &str| Endpoint::Stash {
            realm: realm.into(),
            league: "Standard".into(),
            id: "t1".into(),
            sub: None,
        };
        s.record(
            &ep("pc"),
            &json!({}),
            200,
            &stash("t1", vec![item("i1", "Foo", 0)]),
            1,
        )
        .unwrap();
        let ing = s
            .record(
                &ep("xbox"),
                &json!({}),
                200,
                &stash("t1", vec![item("x1", "Bar", 0)]),
                2,
            )
            .unwrap();
        assert_eq!((ing.added, ing.removed), (1, 0));
        assert!(s.item("i1").unwrap().unwrap().removed_at.is_none());
        assert_eq!(s.tabs("pc", "Standard").unwrap()[0].item_count, 1);
        assert_eq!(s.tabs("xbox", "Standard").unwrap()[0].item_count, 1);
        let ing = s
            .record(
                &ep("xbox"),
                &json!({}),
                200,
                &stash("t1", vec![item("x1", "Bar", 0), item("i1", "Foo", 0)]),
                3,
            )
            .unwrap();
        assert_eq!(ing.moved, 1);
        assert_eq!(s.tabs("pc", "Standard").unwrap()[0].item_count, 0);
        assert_eq!(s.tabs("xbox", "Standard").unwrap()[0].item_count, 2);
        // The event names both coordinates in full.
        let (from, to): (String, String) = s
            .conn
            .query_row(
                "SELECT from_location, to_location FROM item_events WHERE item_id = 'i1' AND kind = 'moved'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(
            (from.as_str(), to.as_str()),
            ("stash/pc/Standard/t1", "stash/xbox/Standard/t1")
        );
    }

    /// A parent tab's fetch is the listing of its substashes: a stub it no
    /// longer carries is retired with its items; and when a listing
    /// retires the parent itself, the substash rows stay (orphaned, for
    /// the planner's report) but their items are not live inventory.
    #[test]
    fn substash_membership_follows_the_parents_fetch_and_the_parents_fate() {
        let mut s = Store::open_memory().unwrap();
        let parent = |children: &[&str]| {
            json!({ "stash": { "id": "m1", "name": "Maps", "type": "MapStash", "items": [],
                "children": children.iter().map(|c| json!({ "id": c, "name": "", "type": "MapStash", "parent": "m1", "metadata": { "items": 1 } })).collect::<Vec<_>>() } })
        };
        let sub = |id: &str| Endpoint::Stash {
            realm: "pc".into(),
            league: "Standard".into(),
            id: "m1".into(),
            sub: Some(id.into()),
        };
        let sub_body = |id: &str, item_id: &str| json!({ "stash": { "id": id, "name": "", "type": "MapStash", "parent": "m1", "items": [ item(item_id, "Map", 0) ] } });
        let listing = Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        };
        s.record(
            &listing,
            &json!({}),
            200,
            &json!({ "stashes": [ { "id": "m1", "name": "Maps", "type": "MapStash", "index": 0 } ] }),
            1,
        )
        .unwrap();
        s.record(&stash_ep("m1"), &json!({}), 200, &parent(&["s1", "s2"]), 2)
            .unwrap();
        s.record(&sub("s1"), &json!({}), 200, &sub_body("s1", "map1"), 3)
            .unwrap();
        // The parent no longer lists s1: retired, with its item.
        let ing = s
            .record(&stash_ep("m1"), &json!({}), 200, &parent(&["s2"]), 4)
            .unwrap();
        assert_eq!(ing.removed, 1);
        let ids: Vec<String> = s
            .tabs("pc", "Standard")
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["m1", "s2"]);
        assert!(s.item("map1").unwrap().unwrap().removed_at.is_some());
        // s2 gets an item; then the listing drops the parent: s2's row
        // stays (orphaned) and its item is no longer live.
        s.record(&sub("s2"), &json!({}), 200, &sub_body("s2", "map2"), 5)
            .unwrap();
        let ing = s
            .record(&listing, &json!({}), 200, &json!({ "stashes": [] }), 6)
            .unwrap();
        assert_eq!(ing.removed, 1);
        let ids: Vec<String> = s
            .tabs("pc", "Standard")
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["s2"]);
        assert!(s.item("map2").unwrap().unwrap().removed_at.is_some());
        let s2 = &s.tabs("pc", "Standard").unwrap()[0];
        // …and its freshness went with its facts: no fetch on record.
        assert_eq!((s2.item_count, s2.fetched_at), (0, None));
        // With the parent retired, a late fetch of the substash is
        // withheld (its parent is not live), and a late fetch of the
        // parent neither revives it nor rewrites its children: the orphan
        // report stays exactly as the listing left it.
        let ing = s
            .record(&sub("s2"), &json!({}), 200, &sub_body("s2", "map2"), 7)
            .unwrap();
        assert_eq!((ing.items, ing.withheld), (0, Some(1)));
        assert!(s.item("map2").unwrap().unwrap().removed_at.is_some());
        let ing = s
            .record(&stash_ep("m1"), &json!({}), 200, &parent(&["s9"]), 8)
            .unwrap();
        assert_eq!((ing.items, ing.withheld), (0, Some(0)));
        let ids: Vec<String> = s
            .tabs("pc", "Standard")
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["s2"]);
    }

    /// Removal events name exactly the items this response retired: two
    /// responses in one second retiring different items at one location
    /// each get their own event, never a duplicate attributed to the
    /// later one.
    #[test]
    fn removal_events_come_from_the_update_not_the_clock() {
        let mut s = Store::open_memory().unwrap();
        let at = 10;
        for items in [
            vec![item("i1", "A", 0), item("i2", "B", 1), item("i3", "C", 2)],
            vec![item("i1", "A", 0), item("i2", "B", 1)],
            vec![item("i1", "A", 0)],
        ] {
            s.record(&stash_ep("t1"), &json!({}), 200, &stash("t1", items), at)
                .unwrap();
        }
        let removed: Vec<(String, i64)> = {
            let mut stmt = s
                .conn
                .prepare("SELECT item_id, response_id FROM item_events WHERE kind = 'removed' ORDER BY id")
                .unwrap();
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
            rows.collect::<Result<_, _>>().unwrap()
        };
        assert_eq!(removed, vec![("i3".to_string(), 2), ("i2".to_string(), 3)]);
    }

    /// `withheld` is a marker as well as a count: a withheld fetch of an
    /// empty location is still marked (`Some(0)`), the count is every item
    /// fact the body carried (socketed gems included), and the
    /// malformed-body contract holds for a withheld body too — an id-less
    /// item is refused and nothing is written, not even the response row.
    #[test]
    fn a_withheld_fetch_is_marked_counted_exactly_and_still_validated() {
        let mut s = Store::open_memory().unwrap();
        let listing = Endpoint::Stashes {
            realm: "pc".into(),
            league: "Standard".into(),
        };
        s.record(
            &listing,
            &json!({}),
            200,
            &json!({ "stashes": [ { "id": "t1", "name": "One", "type": "PremiumStash", "index": 0 } ] }),
            1,
        )
        .unwrap();
        s.record(&listing, &json!({}), 200, &json!({ "stashes": [] }), 2)
            .unwrap();
        // Empty, withheld: marked, count 0.
        let ing = s
            .record(&stash_ep("t1"), &json!({}), 200, &stash("t1", vec![]), 3)
            .unwrap();
        assert_eq!(ing.withheld, Some(0));
        // A bow with a socketed gem: two item facts withheld.
        let mut bow = item("bow", "Bow", 0);
        bow["socketedItems"] = json!([ { "id": "gem", "typeLine": "Gem", "baseType": "Gem" } ]);
        let ing = s
            .record(&stash_ep("t1"), &json!({}), 200, &stash("t1", vec![bow]), 4)
            .unwrap();
        assert_eq!(ing.withheld, Some(2));
        let st = s.status().unwrap();
        assert_eq!((st.withheld_responses, st.withheld_items), (2, 2));
        // An ordinary response is not marked.
        let ing = s
            .record(&listing, &json!({}), 200, &json!({ "stashes": [] }), 5)
            .unwrap();
        assert_eq!(ing.withheld, None);
        // Malformed stays malformed at a retired location.
        let responses_before = s.status().unwrap().responses;
        let err = s
            .record(
                &stash_ep("t1"),
                &json!({}),
                200,
                &json!({ "stash": { "id": "t1", "items": [ { "typeLine": "NoId" } ] } }),
                6,
            )
            .unwrap_err();
        assert!(err.downcast_ref::<MalformedBody>().is_some(), "{err:#}");
        let err = s
            .record(
                &stash_ep("t1"),
                &json!({}),
                200,
                &json!({ "stash": { "id": "t1", "items": [], "children": [ { "name": "NoId" } ] } }),
                7,
            )
            .unwrap_err();
        assert!(err.downcast_ref::<MalformedBody>().is_some(), "{err:#}");
        assert_eq!(s.status().unwrap().responses, responses_before);
    }

    /// A v5 file's `withheld` (NOT NULL DEFAULT 0) is rebuilt nullable:
    /// a positive count stays a withheld mark, 0 becomes an ordinary
    /// response (v5 could not tell an empty withheld fetch apart).
    #[test]
    fn a_v5_withheld_column_is_rebuilt_nullable() {
        let dir =
            std::env::temp_dir().join(format!("acq-store-v5-{}-{}", std::process::id(), now()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("v5.db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE responses (id INTEGER PRIMARY KEY, endpoint TEXT NOT NULL, params TEXT NOT NULL,
                    fetched_at INTEGER NOT NULL, status INTEGER NOT NULL, envelope TEXT NOT NULL, item_count INTEGER NOT NULL,
                    withheld INTEGER NOT NULL DEFAULT 0);
                 INSERT INTO responses (endpoint, params, fetched_at, status, envelope, item_count, withheld)
                 VALUES ('stash', '{}', 1, 200, '{}', 0, 2), ('stash', '{}', 2, 200, '{}', 3, 0);
                 PRAGMA user_version = 5;",
            )
            .unwrap();
        }
        let s = Store::open(&path).unwrap();
        let rows: Vec<Option<i64>> = {
            let mut stmt = s
                .conn
                .prepare("SELECT withheld FROM responses ORDER BY id")
                .unwrap();
            let rows = stmt.query_map([], |r| r.get(0)).unwrap();
            rows.collect::<Result<_, _>>().unwrap()
        };
        assert_eq!(rows, vec![Some(2), None]);
        let st = s.status().unwrap();
        assert_eq!((st.withheld_responses, st.withheld_items), (1, 2));
        drop(s);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
