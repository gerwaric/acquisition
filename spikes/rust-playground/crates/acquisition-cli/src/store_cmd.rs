//! Frontend-side reads of the shared store: no daemon round-trip. The CLI
//! opens the same SQLite file the daemon writes; WAL makes that safe.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::path::PathBuf;

use acquisition_core::provider::ggg_mode;
use acquisition_core::realm::Realm;
use acquisition_store::{AccountEntry, Endpoint, Index, Store, account_path, store_dir};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

static SELECTOR: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

/// `--account` (clap already folded in `ACQ_ACCOUNT`), set once by main.
pub fn set_selector(selector: Option<String>) {
    let _ = SELECTOR.set(selector);
}

pub(crate) fn provider() -> &'static str {
    if ggg_mode() { "ggg" } else { "mock" }
}

/// Which account's store: `ACQ_ACCOUNT` (exact username, name without
/// discriminator, or uuid), else the sole known account. Resolved against
/// the index file, so no daemon is involved.
pub(crate) fn resolve() -> Result<(PathBuf, AccountEntry)> {
    let dir = store_dir(provider());
    let index = Index::load(&dir)?;
    let selector = SELECTOR.get().cloned().flatten();
    let entry = index
        .resolve(selector.as_deref())
        .map_err(anyhow::Error::from)?
        .clone();
    Ok((dir, entry))
}

pub fn open() -> Result<Store> {
    let (dir, entry) = resolve()?;
    Store::open(&account_path(&dir, &entry.username))
}

/// After `auth status`: the other accounts the index knows.
pub fn print_other_accounts(live: Vec<String>) -> Result<()> {
    let dir = store_dir(provider());
    let index = Index::load(&dir)?;
    let others: Vec<&AccountEntry> = index
        .entries()
        .iter()
        .filter(|e| !live.contains(&e.username))
        .collect();
    if others.is_empty() {
        return Ok(());
    }
    println!(
        "other accounts: {}",
        others
            .iter()
            .map(|e| format!(
                "{}{}",
                e.username,
                if e.persisted { "" } else { " (not persisted)" }
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

/// Every account the index knows, with its store file.
pub fn accounts(json: bool) -> Result<()> {
    let dir = store_dir(provider());
    let index = Index::load(&dir)?;
    if json {
        let rows: Vec<Value> = index
            .entries()
            .iter()
            .map(|e| json!({ "username": e.username, "last_login": e.last_login, "persisted": e.persisted,
                             "uuid": e.uuid, "store": account_path(&dir, &e.username) }))
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    if index.entries().is_empty() {
        println!("no accounts known for {} (run `acq auth`)", provider());
        return Ok(());
    }
    let now = acquisition_store::now();
    for e in index.entries() {
        let size = std::fs::metadata(account_path(&dir, &e.username))
            .map(|m| m.len())
            .unwrap_or(0);
        println!(
            "{:<24} last login {:<10} {:<14} store {:.1} MB{}",
            e.username,
            ago(now, Some(e.last_login)),
            if e.persisted {
                "in keyring"
            } else {
                "not persisted"
            },
            size as f64 / 1e6,
            e.uuid
                .as_deref()
                .map(|u| format!("  uuid {u}"))
                .unwrap_or_default()
        );
    }
    Ok(())
}

pub(crate) fn ago(now: i64, t: Option<i64>) -> String {
    match t {
        None => "never".into(),
        Some(t) => {
            let d = (now - t).max(0);
            if d < 90 {
                format!("{d}s ago")
            } else if d < 5400 {
                format!("{}m ago", d / 60)
            } else if d < 172_800 {
                format!("{}h ago", d / 3600)
            } else {
                format!("{}d ago", d / 86400)
            }
        }
    }
}

/// `xbox/` before a league in a message; nothing for pc, as on the wire.
pub(crate) fn realm_prefix(realm: Realm) -> String {
    match realm.segment() {
        Some(seg) => format!("{seg}/"),
        None => String::new(),
    }
}

/// Cut a label to `max` characters with a marker, never silently.
fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}

/// The items column: `-` when never fetched (nothing is known), the live
/// count otherwise — a fetched-empty body reads `0`, never the same as
/// unfetched (legibility ruling, 2026-09-02).
fn items_cell(fetched_at: Option<i64>, item_count: i64) -> String {
    match fetched_at {
        None => "-".into(),
        Some(_) => item_count.to_string(),
    }
}

pub fn tabs(realm: Realm, league: &str, json: bool) -> Result<()> {
    let store = open()?;
    let tabs = store.tabs(realm.as_str(), league)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&tabs)?);
        return Ok(());
    }
    if tabs.is_empty() {
        println!(
            "no tabs known for {}{league} in {} (run `acq stashes` or `acq refresh --all`)",
            realm_prefix(realm),
            store.path().display()
        );
        return Ok(());
    }
    let now = acquisition_store::now();
    let names: Vec<String> = tabs
        .iter()
        .map(|t| {
            let name = if t.name.is_empty() {
                "(unnamed)"
            } else {
                t.name.as_str()
            };
            let indent = if t.parent.is_some() { "  " } else { "" };
            clip(&format!("{indent}{name}"), 40)
        })
        .collect();
    let name_w = names
        .iter()
        .map(|n| n.chars().count())
        .max()
        .unwrap_or(4)
        .max(4);
    let type_w = tabs
        .iter()
        .map(|t| t.r#type.chars().count())
        .max()
        .unwrap_or(4)
        .max(4);
    println!(
        "{:<12} {:<name_w$} {:<type_w$} {:>6}  {:<12} parent",
        "id", "name", "type", "items", "fetched"
    );
    for (t, name) in tabs.iter().zip(&names) {
        println!(
            "{:<12} {:<name_w$} {:<type_w$} {:>6}  {:<12} {}",
            t.id,
            name,
            t.r#type,
            items_cell(t.fetched_at, t.item_count),
            ago(now, t.fetched_at),
            t.parent.as_deref().unwrap_or("")
        );
    }
    let fetched = tabs.iter().filter(|t| t.fetched_at.is_some()).count();
    let folders = tabs.iter().filter(|t| t.r#type == "Folder").count();
    let never = tabs.len() - fetched - folders;
    let mut parts = vec![format!("{fetched} fetched")];
    if never > 0 {
        parts.push(format!("{never} never fetched"));
    }
    if folders > 0 {
        parts.push(format!(
            "{folders} folder{} (never fetched)",
            if folders == 1 { "" } else { "s" }
        ));
    }
    println!(
        "{} tabs: {}; {} items",
        tabs.len(),
        parts.join(", "),
        tabs.iter().map(|t| t.item_count).sum::<i64>()
    );
    Ok(())
}

/// `acq store characters`: the characters on record, from the shared
/// store (no daemon, no network) — the CLI's twin of the MCP `characters`
/// tool. The readable columns first; the full id last (policy ids match
/// exactly, CONTEXT.md 2026-09-02, so it is never cut).
pub fn characters(realm: Option<Realm>, league: Option<&str>, json: bool) -> Result<()> {
    let store = open()?;
    let characters = store.characters(realm.map(Realm::as_str), league)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&characters)?);
        return Ok(());
    }
    if characters.is_empty() {
        println!(
            "no characters known in {} (run `acq characters` or cover them in the sync policy)",
            store.path().display()
        );
        return Ok(());
    }
    let now = acquisition_store::now();
    let name_w = characters
        .iter()
        .map(|c| c.name.chars().count())
        .max()
        .unwrap_or(4)
        .clamp(4, 32);
    let league_w = characters
        .iter()
        .map(|c| c.league.as_deref().unwrap_or("-").chars().count())
        .max()
        .unwrap_or(6)
        .max(6);
    println!(
        "{:<name_w$} {:<5} {:<league_w$} {:>5} {:>6}  {:<9} {:<9} id",
        "name", "realm", "league", "level", "items", "fetched", "listed"
    );
    for c in &characters {
        println!(
            "{:<name_w$} {:<5} {:<league_w$} {:>5} {:>6}  {:<9} {:<9} {}",
            clip(&c.name, 32),
            c.realm,
            c.league.as_deref().unwrap_or("-"),
            c.level.map(|l| l.to_string()).unwrap_or_default(),
            items_cell(c.fetched_at, c.item_count),
            ago(now, c.fetched_at),
            ago(now, c.listed_at),
            c.id,
        );
    }
    let fetched = characters.iter().filter(|c| c.fetched_at.is_some()).count();
    let empty = characters
        .iter()
        .filter(|c| c.fetched_at.is_some() && c.fetched_items == Some(0))
        .count();
    let never = characters.len() - fetched;
    let mut summary = format!("{fetched} fetched");
    if empty > 0 {
        summary.push_str(&format!(" ({empty} with empty bodies)"));
    }
    if never > 0 {
        summary.push_str(&format!(", {never} never fetched"));
    }
    println!(
        "{} characters: {summary}; {} items",
        characters.len(),
        characters.iter().map(|c| c.item_count).sum::<i64>()
    );
    Ok(())
}

pub fn search(
    text: &str,
    realm: Option<Realm>,
    league: Option<&str>,
    removed: bool,
    limit: usize,
    json: bool,
) -> Result<()> {
    let store = open()?;
    let items = store.search(text, realm.map(Realm::as_str), league, removed, limit)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }
    for i in &items {
        let label = if i.name.is_empty() {
            i.type_line.clone()
        } else {
            format!("{} {}", i.name, i.type_line)
        };
        let stack = i.stack_size.map(|n| format!(" x{n}")).unwrap_or_default();
        let gone = if i.removed_at.is_some() {
            "  [removed]"
        } else {
            ""
        };
        let socket = i
            .socketed_in
            .as_deref()
            .map(|p| format!(" (in {})", &p[..p.len().min(8)]))
            .unwrap_or_default();
        println!(
            "{:<10} {:<8} {:<12} {label}{stack}{socket}{gone}",
            &i.id[..i.id.len().min(10)],
            i.location_kind,
            i.location_id
        );
    }
    println!(
        "{} item(s){}",
        items.len(),
        if items.len() == limit {
            " (limit reached)"
        } else {
            ""
        }
    );
    Ok(())
}

pub fn show(id: &str, json: bool) -> Result<()> {
    let store = open()?;
    let Some(item) = store.item(id)? else {
        bail!("no item {id}")
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&item)?);
    } else {
        println!(
            "{}/{}{}  first seen {}  last seen {}{}",
            item.location_kind,
            item.location_id,
            item.socketed_in
                .as_deref()
                .map(|p| format!(" socketed in {p}"))
                .unwrap_or_default(),
            item.first_seen,
            item.last_seen,
            item.removed_at
                .map(|t| format!("  removed {t}"))
                .unwrap_or_default()
        );
        println!("{}", serde_json::to_string_pretty(&item.json)?);
    }
    Ok(())
}

pub fn status(json: bool) -> Result<()> {
    let store = open()?;
    let st = store.status()?;
    if json {
        println!("{}", serde_json::to_string_pretty(&st)?);
    } else {
        println!("{}  ({:.1} MB)", st.path, st.bytes as f64 / 1e6);
        println!(
            "responses {}  leagues {}  characters {}  tabs {}  items {} (+{} removed)  events {}",
            st.responses, st.leagues, st.characters, st.tabs, st.items, st.items_removed, st.events
        );
        if st.withheld_responses > 0 {
            println!(
                "withheld: {} response(s) fetched for locations a listing had retired, {} item fact(s) kept on their response rows and landed nowhere",
                st.withheld_responses, st.withheld_items
            );
        }
        if st.unlifted_items > 0 {
            // The drift tripwire: an item array the store does not lift.
            println!(
                "drift: {} item-shaped object(s) in character arrays this build does not lift (see `_unlifted` in the responses)",
                st.unlifted_items
            );
        }
        if st.granted_skills > 0 {
            println!(
                "granted: {} item-granted skill(s) left in place on live characters (PoE2; `_granted` in the character json, never rows)",
                st.granted_skills
            );
        }
        if st.refused_bodies > 0 {
            println!(
                "refused: {} body(ies) the store would not ingest, kept verbatim as evidence (`acq store refused`)",
                st.refused_bodies
            );
        }
    }
    Ok(())
}

/// Bodies the store refused as malformed: the list, or one body in full.
pub fn refused(id: Option<i64>, limit: usize, json: bool) -> Result<()> {
    let store = open()?;
    if let Some(id) = id {
        let Some(row) = store.refused(id)? else {
            anyhow::bail!("no refused body with id {id} (see `acq store refused`)");
        };
        if json {
            println!("{}", serde_json::to_string_pretty(&row)?);
        } else {
            println!(
                "refused {}  {} {}  fetched {}  status {}",
                row.id,
                row.endpoint,
                row.params,
                ago(acquisition_store::now(), Some(row.fetched_at)),
                row.status
            );
            println!("reason: {}", row.reason);
            println!("{}", serde_json::to_string_pretty(&row.body)?);
        }
        return Ok(());
    }
    let rows = store.refused_list(limit)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&rows)?);
        return Ok(());
    }
    let now = acquisition_store::now();
    for r in &rows {
        println!(
            "{:>5}  {:<10} {:<40} {:<9}  {}",
            r.id,
            r.endpoint,
            clip(&r.params.to_string(), 40),
            ago(now, Some(r.fetched_at)),
            r.reason
        );
    }
    println!(
        "{} refused body(ies){}",
        rows.len(),
        if rows.is_empty() {
            ""
        } else {
            " — `acq store refused <id>` prints one in full"
        }
    );
    Ok(())
}

/// One location's share of the events since a time: the counts a person
/// asks for ("what changed where"), with the location's name resolved
/// from the store's tab and character rows.
#[derive(Debug, Default, Clone, serde::Serialize)]
pub(crate) struct LocationChanges {
    pub location: String,
    pub name: Option<String>,
    pub added: usize,
    pub changed: usize,
    pub moved_in: usize,
    pub moved_out: usize,
    pub removed: usize,
    pub first: i64,
    pub last: i64,
}

impl LocationChanges {
    fn counts(&self) -> String {
        let mut parts = Vec::new();
        if self.added > 0 {
            parts.push(format!("+{} added", self.added));
        }
        if self.changed > 0 {
            parts.push(format!("~{} changed", self.changed));
        }
        if self.moved_in > 0 {
            parts.push(format!(">{} moved in", self.moved_in));
        }
        if self.moved_out > 0 {
            parts.push(format!("<{} moved out", self.moved_out));
        }
        if self.removed > 0 {
            parts.push(format!("-{} removed", self.removed));
        }
        parts.join(", ")
    }
}

/// Group events by location, newest activity first. `names` resolves a
/// location string to what a person calls it.
pub(crate) fn summarize_events(
    events: &[acquisition_store::EventRow],
    names: &mut dyn FnMut(&str) -> Option<String>,
) -> Vec<LocationChanges> {
    let mut by: BTreeMap<String, LocationChanges> = BTreeMap::new();
    let touch = |by: &mut BTreeMap<String, LocationChanges>, loc: &str, at: i64| {
        let e = by
            .entry(loc.to_string())
            .or_insert_with(|| LocationChanges {
                location: loc.to_string(),
                first: at,
                last: at,
                ..Default::default()
            });
        e.first = e.first.min(at);
        e.last = e.last.max(at);
    };
    for ev in events {
        match ev.kind.as_str() {
            "moved" => {
                if let Some(from) = &ev.from_location {
                    touch(&mut by, from, ev.at);
                    if let Some(e) = by.get_mut(from) {
                        e.moved_out += 1;
                    }
                }
                if let Some(to) = &ev.to_location {
                    touch(&mut by, to, ev.at);
                    if let Some(e) = by.get_mut(to) {
                        e.moved_in += 1;
                    }
                }
            }
            "removed" => {
                if let Some(from) = &ev.from_location {
                    touch(&mut by, from, ev.at);
                    if let Some(e) = by.get_mut(from) {
                        e.removed += 1;
                    }
                }
            }
            kind => {
                if let Some(to) = &ev.to_location {
                    touch(&mut by, to, ev.at);
                    if let Some(e) = by.get_mut(to) {
                        if kind == "added" {
                            e.added += 1;
                        } else {
                            e.changed += 1;
                        }
                    }
                }
            }
        }
    }
    let mut out: Vec<LocationChanges> = by.into_values().collect();
    for e in &mut out {
        e.name = names(&e.location);
    }
    out.sort_by(|a, b| {
        b.last
            .cmp(&a.last)
            .then_with(|| a.location.cmp(&b.location))
    });
    out
}

/// `stash/<realm>/<league>/<id>` → the tab's name; `character/<realm>/<id>`
/// → the character's — through the store's own row reads, cached per
/// listing so a thousand events cost a handful of queries.
fn location_namer(store: &Store) -> impl FnMut(&str) -> Option<String> + '_ {
    let mut tabs: HashMap<(String, String), HashMap<String, String>> = HashMap::new();
    let mut characters: HashMap<String, HashMap<String, String>> = HashMap::new();
    move |location: &str| {
        let parts: Vec<&str> = location.split('/').collect();
        match parts.as_slice() {
            ["stash", realm, league, id] => {
                let key = (realm.to_string(), league.to_string());
                let map = tabs.entry(key).or_insert_with(|| {
                    store
                        .tabs(realm, league)
                        .map(|rows| rows.into_iter().map(|t| (t.id, t.name)).collect())
                        .unwrap_or_default()
                });
                map.get(*id).cloned()
            }
            ["character", realm, id] => {
                let map = characters.entry(realm.to_string()).or_insert_with(|| {
                    store
                        .characters(Some(realm), None)
                        .map(|rows| rows.into_iter().map(|c| (c.id, c.name)).collect())
                        .unwrap_or_default()
                });
                map.get(*id).cloned()
            }
            _ => None,
        }
    }
}

/// `acq store events`: the text default is one line per location with
/// counts; the JSON default is the event list (what the driver reads).
/// `--expand` / `--summary` select either form in either mode (the
/// legibility ruling's one stated divergence).
pub fn events(
    since_hours: f64,
    limit: usize,
    expand: bool,
    summary: bool,
    json: bool,
) -> Result<()> {
    let store = open()?;
    let now = acquisition_store::now();
    let since = now - (since_hours * 3600.0) as i64;
    let ev = store.events_since(since, limit)?;
    let summarize = summary || (!expand && !json);
    if summarize {
        let mut namer = location_namer(&store);
        let rows = summarize_events(&ev, &mut namer);
        if json {
            println!("{}", serde_json::to_string_pretty(&rows)?);
            return Ok(());
        }
        let total = LocationChanges {
            added: rows.iter().map(|r| r.added).sum(),
            changed: rows.iter().map(|r| r.changed).sum(),
            moved_in: rows.iter().map(|r| r.moved_in).sum(),
            moved_out: 0,
            removed: rows.iter().map(|r| r.removed).sum(),
            ..Default::default()
        };
        let hours = if since_hours == since_hours.trunc() {
            format!("{}", since_hours as i64)
        } else {
            format!("{since_hours:.1}")
        };
        if ev.is_empty() {
            println!("0 events in the last {hours} h");
            return Ok(());
        }
        let counts = total.counts().replace(" moved in", " moved");
        println!(
            "{} event{} at {} location{} in the last {hours} h: {counts}{}",
            ev.len(),
            if ev.len() == 1 { "" } else { "s" },
            rows.len(),
            if rows.len() == 1 { "" } else { "s" },
            if ev.len() == limit {
                " (limit reached)"
            } else {
                ""
            }
        );
        let loc_w = rows
            .iter()
            .map(|r| r.location.chars().count())
            .max()
            .unwrap_or(0);
        let name_w = rows
            .iter()
            .map(|r| {
                r.name
                    .as_deref()
                    .map_or(0, |n| n.chars().count().max(7) + 2)
            })
            .max()
            .unwrap_or(0);
        for r in &rows {
            let name = match r.name.as_deref() {
                Some("") => "(unnamed)".to_string(),
                Some(n) => format!("{n:?}"),
                None => String::new(),
            };
            println!(
                "  {:<loc_w$}  {:<name_w$}  {:<28}  {}",
                r.location,
                name,
                r.counts(),
                ago(now, Some(r.last))
            );
        }
        println!("(--expand lists every event)");
        return Ok(());
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&ev)?);
        return Ok(());
    }
    for e in &ev {
        let label = match (&e.name, &e.type_line) {
            (Some(n), Some(t)) if !n.is_empty() => format!("{n} {t}"),
            (_, Some(t)) => t.clone(),
            _ => "?".into(),
        };
        let loc = match e.kind.as_str() {
            "moved" => format!(
                "{} -> {}",
                e.from_location.as_deref().unwrap_or("?"),
                e.to_location.as_deref().unwrap_or("?")
            ),
            "removed" => format!("from {}", e.from_location.as_deref().unwrap_or("?")),
            _ => format!("at {}", e.to_location.as_deref().unwrap_or("?")),
        };
        println!(
            "{:<9} {:<8} {:<40} {loc}",
            ago(now, Some(e.at)),
            e.kind,
            clip(&label, 40)
        );
    }
    println!(
        "{} event(s){}",
        ev.len(),
        if ev.len() == limit {
            " (limit reached)"
        } else {
            ""
        }
    );
    Ok(())
}

pub fn rebuild(json: bool) -> Result<()> {
    let mut store = open()?;
    let n = store.rebuild()?;
    if json {
        println!("{}", json!({ "reextracted": n }));
    } else {
        println!("re-extracted columns for {n} items");
    }
    Ok(())
}

/// Replay a snapshot from the retired `acq pull` (its format: `{league,
/// taken_at_unix, tabs: {id: {name, type, items: {id: item}}}}`) into the
/// store as if each tab had just been fetched: real-shape data, zero GGG
/// traffic. Those snapshots lost tab metadata and parents, so tabs land
/// flat. Kept as the real-data fixture path.
pub fn import(path: &Path, json: bool) -> Result<()> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let snap: Value = serde_json::from_str(&text)?;
    let league = snap
        .get("league")
        .and_then(Value::as_str)
        .unwrap_or("Standard")
        .to_string();
    let at = snap
        .get("taken_at_unix")
        .and_then(Value::as_i64)
        .unwrap_or_else(acquisition_store::now);
    let Some(tabs) = snap.get("tabs").and_then(Value::as_object) else {
        bail!("not a pull snapshot: no `tabs`")
    };
    let mut store = open()?;
    let started = std::time::Instant::now();
    let mut total = acquisition_store::Ingest::default();
    let mut skipped_no_id = 0usize;
    for (id, tab) in tabs {
        let mut items: Vec<Value> = match tab.get("items") {
            Some(Value::Object(m)) => m.values().cloned().collect(),
            Some(Value::Array(a)) => a.clone(),
            _ => Vec::new(),
        };
        // Legacy tolerance lives here, at the import boundary, and is
        // reported: the store's network ingest refuses an id-less item as
        // malformed (it cannot be tracked, and dropped silently it would
        // poison removal), but an old pull snapshot is what it is.
        strip_idless_items(&mut items, &mut skipped_no_id);
        let body = json!({ "stash": { "id": id, "name": tab.get("name"), "type": tab.get("type"), "items": items } });
        // A retired-pull snapshot predates realms: pc, the only realm it
        // could have been taken on.
        let ep = Endpoint::Stash {
            realm: "pc".into(),
            league: league.clone(),
            id: id.clone(),
            sub: None,
        };
        let ing = store.record(
            &ep,
            &json!({ "realm": "pc", "league": league, "id": id }),
            200,
            &body,
            at,
        )?;
        total.items += ing.items;
        total.added += ing.added;
        total.moved += ing.moved;
        total.changed += ing.changed;
        total.removed += ing.removed;
    }
    let elapsed = started.elapsed();
    if json {
        println!(
            "{}",
            json!({ "tabs": tabs.len(), "ingest": total, "seconds": elapsed.as_secs_f64(),
                    "skipped_no_id": skipped_no_id })
        );
    } else {
        println!(
            "imported {} tabs, {} items (+{} added, ~{} changed, >{} moved, -{} removed) in {:.2}s",
            tabs.len(),
            total.items,
            total.added,
            total.changed,
            total.moved,
            total.removed,
            elapsed.as_secs_f64()
        );
        if skipped_no_id > 0 {
            println!("skipped {skipped_no_id} item(s) without ids (legacy snapshot tolerance)");
        }
    }
    Ok(())
}

/// Drop items (and socketed gems) that carry no `id`, counting them, so a
/// legacy snapshot imports with a report instead of failing — the store's
/// own ingest refuses id-less items as malformed.
fn strip_idless_items(items: &mut Vec<Value>, skipped: &mut usize) {
    items.retain(|item| {
        let ok = item.get("id").and_then(Value::as_str).is_some();
        if !ok {
            *skipped += 1;
        }
        ok
    });
    for item in items {
        if let Some(gems) = item.get_mut("socketedItems").and_then(Value::as_array_mut) {
            let before = gems.len();
            gems.retain(|gem| gem.get("id").and_then(Value::as_str).is_some());
            *skipped += before - gems.len();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acquisition_store::EventRow;

    fn ev(at: i64, kind: &str, from: Option<&str>, to: Option<&str>) -> EventRow {
        EventRow {
            at,
            item_id: "i".into(),
            kind: kind.into(),
            from_location: from.map(str::to_string),
            to_location: to.map(str::to_string),
            name: None,
            type_line: None,
        }
    }

    #[test]
    fn events_group_by_location_newest_first_with_names() {
        let events = vec![
            ev(10, "added", None, Some("character/pc/c1")),
            ev(11, "added", None, Some("character/pc/c1")),
            ev(
                12,
                "moved",
                Some("stash/pc/Standard/t1"),
                Some("character/pc/c1"),
            ),
            ev(13, "removed", Some("stash/pc/Standard/t1"), None),
            ev(20, "changed", None, Some("stash/pc/Standard/t2")),
        ];
        let mut namer = |loc: &str| match loc {
            "character/pc/c1" => Some("Exile".to_string()),
            "stash/pc/Standard/t1" => Some("Dump".to_string()),
            _ => None,
        };
        let rows = summarize_events(&events, &mut namer);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].location, "stash/pc/Standard/t2");
        assert_eq!(rows[0].changed, 1);
        assert!(rows[0].name.is_none());
        let t1 = rows.iter().find(|r| r.location.ends_with("t1")).unwrap();
        assert_eq!((t1.moved_out, t1.removed), (1, 1));
        assert_eq!(t1.name.as_deref(), Some("Dump"));
        let c1 = rows.iter().find(|r| r.location.ends_with("c1")).unwrap();
        assert_eq!((c1.added, c1.moved_in, c1.first, c1.last), (2, 1, 10, 12));
        assert_eq!(c1.counts(), "+2 added, >1 moved in");
    }

    #[test]
    fn the_items_cell_tells_unfetched_from_empty() {
        assert_eq!(items_cell(None, 0), "-");
        assert_eq!(items_cell(Some(1), 0), "0");
        assert_eq!(items_cell(Some(1), 7), "7");
        assert_eq!(clip("abcdef", 4), "abc…");
        assert_eq!(clip("abc", 4), "abc");
    }
}
