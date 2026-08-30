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

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const SCHEMA: &str = include_str!("schema.sql");

/// Item fields the server re-randomizes per fetch (ground-truth N36), so
/// they never count as a change.
pub const VOLATILE_ITEM_FIELDS: &[&str] = &["veiledMods"];

/// Where the character response keeps its items.
const CHARACTER_ITEM_ARRAYS: &[&str] = &["inventory", "equipment", "jewels", "rucksack"];

/// Which API response a body is. The daemon maps a job kind onto this; the
/// store maps it onto tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endpoint {
    Leagues,
    Profile,
    Characters,
    Character {
        name: String,
    },
    Stashes {
        league: String,
    },
    /// One tab or one substash; `sub` is the substash id under tab `id`.
    Stash {
        league: String,
        id: String,
        sub: Option<String>,
    },
}

impl Endpoint {
    /// The daemon's job vocabulary → endpoint. `None` for kinds that carry
    /// no storable response (probe, sleep, fetch, refresh…).
    pub fn from_job(kind: &str, params: &Value) -> Option<Endpoint> {
        let s = |k: &str| params.get(k).and_then(Value::as_str).map(str::to_string);
        let league = || s("league").unwrap_or_else(|| "Standard".into());
        Some(match kind {
            "leagues" => Endpoint::Leagues,
            "profile" => Endpoint::Profile,
            "characters" => Endpoint::Characters,
            "character" => Endpoint::Character { name: s("name")? },
            "stashes" => Endpoint::Stashes { league: league() },
            "stash" => Endpoint::Stash {
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
            Endpoint::Characters => "characters",
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabRow {
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
pub struct ItemRow {
    pub id: String,
    pub league: Option<String>,
    pub location_kind: String,
    pub location_id: String,
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
}

pub mod index;
pub use index::{AccountEntry, Index, Resolve, account_path, index_path, store_dir};

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

    fn init(conn: Connection, path: PathBuf) -> Result<Store> {
        // WAL: the daemon writes while any number of frontends read.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch(SCHEMA)?;
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
        // (league, location_kind, location_id, items) per seam.
        let mut seams: Vec<(Option<String>, &str, String, Vec<Value>)> = Vec::new();

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
            Endpoint::Characters => {
                let list = body
                    .get("characters")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                for c in &list {
                    if let Some(name) = c.get("name").and_then(Value::as_str) {
                        tx.execute(
                            "INSERT INTO characters (name, league, class, level, json, listed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                             ON CONFLICT(name) DO UPDATE SET league = excluded.league, class = excluded.class,
                               level = excluded.level, json = excluded.json, listed_at = excluded.listed_at, removed_at = NULL",
                            params![name, c.get("league").and_then(Value::as_str), c.get("class").and_then(Value::as_str),
                                    c.get("level").and_then(Value::as_i64), c.to_string(), at],
                        )?;
                    }
                }
                // A character no longer listed is gone (deleted), with its items.
                tx.execute(
                    "UPDATE characters SET removed_at = ?1 WHERE removed_at IS NULL AND (listed_at IS NULL OR listed_at < ?1)",
                    params![at],
                )?;
            }
            Endpoint::Character { name } => {
                let Some(character) = envelope.get_mut("character").and_then(Value::as_object_mut)
                else {
                    return Err(anyhow!("character response without a `character` object"));
                };
                let league = character
                    .get("league")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let mut split = serde_json::Map::new();
                for key in CHARACTER_ITEM_ARRAYS {
                    if let Some(Value::Array(items)) = character.remove(*key) {
                        split.insert((*key).into(), json!(items.len()));
                        seams.push((league.clone(), "character", name.clone(), items));
                    }
                }
                character.insert("_split".into(), Value::Object(split));
                let c = Value::Object(character.clone());
                tx.execute(
                    "INSERT INTO characters (name, league, class, level, json, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                     ON CONFLICT(name) DO UPDATE SET league = excluded.league, class = excluded.class,
                       level = excluded.level, json = excluded.json, fetched_at = excluded.fetched_at, removed_at = NULL",
                    params![name, league, c.get("class").and_then(Value::as_str), c.get("level").and_then(Value::as_i64), c.to_string(), at],
                )?;
            }
            Endpoint::Stashes { league } => {
                let list = body
                    .get("stashes")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default();
                let mut idx = 0;
                for tab in &list {
                    upsert_listed_tab(&tx, league, tab, None, &mut idx, at)?;
                    for child in tab
                        .get("children")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                    {
                        let folder = tab.get("id").and_then(Value::as_str).map(str::to_string);
                        upsert_listed_tab(&tx, league, child, folder, &mut idx, at)?;
                    }
                }
                // Not in this list → removed (top-level and folder children;
                // substashes are only known from fetches and keep their own row).
                tx.execute(
                    "UPDATE tabs SET removed_at = ?2 WHERE league = ?1 AND removed_at IS NULL
                       AND (listed_at IS NULL OR listed_at < ?2)
                       AND (parent IS NULL OR parent IN (SELECT id FROM tabs t2 WHERE t2.league = ?1 AND t2.type = 'Folder'))",
                    params![league, at],
                )?;
            }
            Endpoint::Stash { league, id, sub } => {
                let Some(stash) = envelope.get_mut("stash").and_then(Value::as_object_mut) else {
                    return Err(anyhow!("stash response without a `stash` object"));
                };
                let location = sub.clone().unwrap_or_else(|| id.clone());
                let items = match stash.remove("items") {
                    Some(Value::Array(items)) => items,
                    _ => Vec::new(),
                };
                stash.insert("_split".into(), json!({ "items": items.len() }));
                // Substash stubs of a map/unique tab: each becomes a tab row
                // whose parent is this tab. The fetched tab's own row too.
                let children = match stash.remove("children") {
                    Some(Value::Array(c)) => c,
                    _ => Vec::new(),
                };
                let mut idx = 0;
                for child in &children {
                    upsert_listed_tab(&tx, league, child, Some(id.clone()), &mut idx, at)?;
                }
                let fetched = Value::Object(stash.clone());
                tx.execute(
                    "INSERT INTO tabs (league, id, parent, name, type, json, fetched_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT(league, id) DO UPDATE SET name = excluded.name, type = excluded.type,
                       json = excluded.json, fetched_at = excluded.fetched_at, removed_at = NULL,
                       parent = COALESCE(excluded.parent, tabs.parent)",
                    params![league, location, sub.as_ref().map(|_| id.clone()),
                            fetched.get("name").and_then(Value::as_str), fetched.get("type").and_then(Value::as_str),
                            fetched.to_string(), at],
                )?;
                seams.push((Some(league.clone()), "stash", location, items));
            }
        }

        let item_count: usize = seams.iter().map(|s| s.3.len()).sum();
        tx.execute(
            "INSERT INTO responses (endpoint, params, fetched_at, status, envelope, item_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![endpoint.name(), params.to_string(), at, status, envelope.to_string(), item_count as i64],
        )?;
        let response_id = tx.last_insert_rowid();
        ingest.response_id = response_id;

        // Every seam of one response is one location; a character's four
        // arrays share `character/<name>`, so removal runs once per location.
        let mut locations: Vec<(String, String)> = Vec::new();
        for (league, kind, location_id, items) in seams {
            for item in items {
                ingest_item(
                    &tx,
                    &mut ingest,
                    response_id,
                    at,
                    league.as_deref(),
                    kind,
                    &location_id,
                    None,
                    item,
                )?;
            }
            if !locations
                .iter()
                .any(|(k, l)| k == kind && *l == location_id)
            {
                locations.push((kind.to_string(), location_id));
            }
        }
        for (kind, location_id) in locations {
            let removed = tx.execute(
                "UPDATE items SET removed_at = ?3 WHERE location_kind = ?1 AND location_id = ?2
                   AND removed_at IS NULL AND last_seen < ?3",
                params![kind, location_id, at],
            )?;
            if removed > 0 {
                let from = format!("{kind}/{location_id}");
                tx.execute(
                    "INSERT INTO item_events (response_id, at, item_id, kind, from_location)
                     SELECT ?1, ?2, id, 'removed', ?3 FROM items
                      WHERE location_kind = ?4 AND location_id = ?5 AND removed_at = ?2",
                    params![response_id, at, from, kind, location_id],
                )?;
                ingest.removed += removed;
            }
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
        })
    }

    /// Tabs of a league in listing order (folder children after their
    /// folder, substashes after their tab), removed ones excluded.
    pub fn tabs(&self, league: &str) -> Result<Vec<TabRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.league, t.id, t.parent, COALESCE(t.name, ''), COALESCE(t.type, ''), t.idx, t.listed_at, t.fetched_at, t.removed_at,
                    (SELECT count(*) FROM items i WHERE i.location_kind = 'stash' AND i.location_id = t.id AND i.removed_at IS NULL)
               FROM tabs t WHERE t.league = ?1 AND t.removed_at IS NULL
              ORDER BY COALESCE((SELECT p.idx FROM tabs p WHERE p.league = t.league AND p.id = t.parent), t.idx, 1000000),
                       t.parent IS NOT NULL, COALESCE(t.idx, 0), t.name",
        )?;
        let rows = stmt.query_map([league], |r| {
            Ok(TabRow {
                league: r.get(0)?,
                id: r.get(1)?,
                parent: r.get(2)?,
                name: r.get(3)?,
                r#type: r.get(4)?,
                idx: r.get(5)?,
                listed_at: r.get(6)?,
                fetched_at: r.get(7)?,
                removed_at: r.get(8)?,
                item_count: r.get(9)?,
            })
        })?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    /// Case-insensitive substring search over name, type line, and base
    /// type, live items only unless `include_removed`.
    pub fn search(
        &self,
        text: &str,
        league: Option<&str>,
        include_removed: bool,
        limit: usize,
    ) -> Result<Vec<ItemRow>> {
        let pattern = format!("%{}%", text.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = self.conn.prepare(
            "SELECT id, league, location_kind, location_id, socketed_in, COALESCE(name, ''), COALESCE(type_line, ''),
                    COALESCE(base_type, ''), rarity, stack_size, first_seen, last_seen, removed_at, json
               FROM items
              WHERE (name LIKE ?1 ESCAPE '\\' OR type_line LIKE ?1 ESCAPE '\\' OR base_type LIKE ?1 ESCAPE '\\')
                AND (?2 IS NULL OR league = ?2)
                AND (?3 OR removed_at IS NULL)
              ORDER BY location_kind, location_id, y, x
              LIMIT ?4",
        )?;
        let rows = stmt.query_map(
            params![pattern, league, include_removed, limit as i64],
            |r| {
                let json: String = r.get(13)?;
                Ok(ItemRow {
                    id: r.get(0)?,
                    league: r.get(1)?,
                    location_kind: r.get(2)?,
                    location_id: r.get(3)?,
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
                    COALESCE(base_type, ''), rarity, stack_size, first_seen, last_seen, removed_at, json FROM items WHERE id = ?1",
        )?;
        Ok(stmt
            .query_row([id], |r| {
                let json: String = r.get(13)?;
                Ok(ItemRow {
                    id: r.get(0)?,
                    league: r.get(1)?,
                    location_kind: r.get(2)?,
                    location_id: r.get(3)?,
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
}

fn upsert_listed_tab(
    tx: &Connection,
    league: &str,
    tab: &Value,
    parent: Option<String>,
    idx: &mut i64,
    at: i64,
) -> Result<()> {
    let Some(id) = tab.get("id").and_then(Value::as_str) else {
        return Ok(());
    };
    let mut entry = tab.clone();
    if let Some(o) = entry.as_object_mut() {
        o.remove("children");
    }
    let position = tab.get("index").and_then(Value::as_i64).unwrap_or(*idx);
    *idx += 1;
    tx.execute(
        "INSERT INTO tabs (league, id, parent, name, type, idx, json, listed_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(league, id) DO UPDATE SET parent = excluded.parent, name = excluded.name, type = excluded.type,
           idx = excluded.idx, json = excluded.json, listed_at = excluded.listed_at, removed_at = NULL",
        params![league, id, parent, tab.get("name").and_then(Value::as_str), tab.get("type").and_then(Value::as_str),
                position, entry.to_string(), at],
    )?;
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
    league: Option<&str>,
    kind: &str,
    location_id: &str,
    socketed_in: Option<&str>,
    mut item: Value,
) -> Result<()> {
    let Some(id) = item.get("id").and_then(Value::as_str).map(str::to_string) else {
        // An item without an id cannot be tracked; it stays in nobody's
        // table. Rare enough to just count.
        return Ok(());
    };
    // Socketed gems are items: lift them out, same location, parented.
    let gems = match item.as_object_mut().and_then(|o| o.remove("socketedItems")) {
        Some(Value::Array(g)) => g,
        _ => Vec::new(),
    };
    let to = format!("{kind}/{location_id}");
    let previous: Option<(String, String, String, Option<i64>)> = tx
        .query_row(
            "SELECT location_kind, location_id, json, removed_at FROM items WHERE id = ?1",
            [&id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;
    let c = Columns::of(&item);
    let json = item.to_string();
    match &previous {
        None => {
            tx.execute(
                "INSERT INTO items (id, league, location_kind, location_id, socketed_in, name, type_line, base_type, rarity, stack_size, x, y, w, h, json, first_seen, last_seen)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?16)",
                params![id, league, kind, location_id, socketed_in, c.name, c.type_line, c.base_type, c.rarity, c.stack_size, c.x, c.y, c.w, c.h, json, at],
            )?;
            tx.execute(
                "INSERT INTO item_events (response_id, at, item_id, kind, to_location) VALUES (?1, ?2, ?3, 'added', ?4)",
                params![response_id, at, id, to],
            )?;
            ingest.added += 1;
        }
        Some((old_kind, old_loc, old_json, removed_at)) => {
            let from = format!("{old_kind}/{old_loc}");
            let moved = from != to || removed_at.is_some();
            let old: Value = serde_json::from_str(old_json).unwrap_or(Value::Null);
            let changed = !same_item(&old, &item);
            tx.execute(
                "UPDATE items SET league = ?2, location_kind = ?3, location_id = ?4, socketed_in = ?5, name = ?6, type_line = ?7, base_type = ?8,
                        rarity = ?9, stack_size = ?10, x = ?11, y = ?12, w = ?13, h = ?14, json = ?15, last_seen = ?16, removed_at = NULL
                  WHERE id = ?1",
                params![id, league, kind, location_id, socketed_in, c.name, c.type_line, c.base_type, c.rarity, c.stack_size, c.x, c.y, c.w, c.h, json, at],
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
            league,
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
        let tabs = s.tabs("Standard").unwrap();
        assert_eq!(tabs.len(), 1);
        assert_eq!((tabs[0].id.as_str(), tabs[0].item_count), ("a", 2));
        assert_eq!(s.search("foo", None, false, 10).unwrap().len(), 1);
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
        assert_eq!(ev[0].from_location.as_deref(), Some("stash/a"));
        // Now i1 disappears from a entirely.
        let ing = s
            .record(&stash_ep("a"), &p, 200, &stash("a", vec![]), 300)
            .unwrap();
        assert_eq!(ing.removed, 1);
        assert_eq!(s.item("i1").unwrap().unwrap().removed_at, Some(300));
        assert!(s.search("foo", None, false, 10).unwrap().is_empty());
        assert_eq!(s.search("foo", None, true, 10).unwrap().len(), 1);
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
        assert_eq!(s.search("determination", None, false, 10).unwrap().len(), 1);
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
            league: "Standard".into(),
        };
        s.record(&ep, &json!({}), 200, &list, 10).unwrap();
        let tabs = s.tabs("Standard").unwrap();
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
            league: "Standard".into(),
            id: "m1".into(),
            sub: Some("s1".into()),
        };
        let ing = s.record(&sub, &json!({}), 200, &json!({ "stash": { "id": "s1", "name": "", "type": "MapStash", "parent": "m1", "items": [item("map1", "", 0)] } }), 12).unwrap();
        assert_eq!(ing.added, 1);
        let tabs = s.tabs("Standard").unwrap();
        let s1 = tabs.iter().find(|t| t.id == "s1").unwrap();
        assert_eq!((s1.parent.as_deref(), s1.item_count), (Some("m1"), 1));
        // The next list drops "gone": removed, but the substash (never listed) survives.
        let list2 = json!({ "stashes": [ { "id": "f1", "name": "Folder", "type": "Folder", "index": 0, "children": [ { "id": "c1", "name": "In folder", "type": "PremiumStash", "index": 1 } ] }, { "id": "m1", "name": "Maps", "type": "MapStash", "index": 2 } ] });
        s.record(&ep, &json!({}), 200, &list2, 20).unwrap();
        let ids: Vec<_> = s
            .tabs("Standard")
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(ids, vec!["f1", "c1", "m1", "s1"]);
    }

    #[test]
    fn character_arrays_share_one_location() {
        let mut s = Store::open_memory().unwrap();
        let body = json!({ "character": { "name": "Hero", "league": "Standard", "class": "Witch", "level": 90,
            "inventory": [ item("inv1", "Bag", 0) ], "equipment": [ item("eq1", "Helm", 0) ], "jewels": [] } });
        let ep = Endpoint::Character {
            name: "Hero".into(),
        };
        let ing = s
            .record(&ep, &json!({"name":"Hero"}), 200, &body, 1)
            .unwrap();
        assert_eq!(ing.added, 2);
        let inv = s.item("inv1").unwrap().unwrap();
        assert_eq!(
            (inv.location_kind.as_str(), inv.location_id.as_str()),
            ("character", "Hero")
        );
        let ing = s.record(&ep, &json!({"name":"Hero"}), 200, &json!({ "character": { "name": "Hero", "league": "Standard", "inventory": [ item("inv1", "Bag", 0) ], "equipment": [] } }), 2).unwrap();
        assert_eq!(ing.removed, 1);
        let json: String = s
            .conn
            .query_row("SELECT json FROM characters WHERE name = 'Hero'", [], |r| {
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
        s.record(&Endpoint::Characters, &json!({}), 200, &json!({ "characters": [ { "name": "A", "class": "Witch", "level": 3, "league": "Standard" } ] }), 1).unwrap();
        let st = s.status().unwrap();
        assert_eq!((st.leagues, st.characters, st.responses), (2, 1, 3));
        s.record(
            &Endpoint::Characters,
            &json!({}),
            200,
            &json!({ "characters": [] }),
            2,
        )
        .unwrap();
        assert_eq!(s.status().unwrap().characters, 0);
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
    fn endpoint_from_job_kinds() {
        assert_eq!(
            Endpoint::from_job("stash", &json!({"league":"Standard","id":"a","sub":"b"})),
            Some(Endpoint::Stash {
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
}
