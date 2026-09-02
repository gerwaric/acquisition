//! Frontend-side reads of the shared store: no daemon round-trip. The CLI
//! opens the same SQLite file the daemon writes; WAL makes that safe.

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
    println!(
        "{:<12} {:<28} {:<14} {:>6}  {:<12} parent",
        "id", "name", "type", "items", "fetched"
    );
    for t in &tabs {
        let name = if t.name.is_empty() {
            "(unnamed)".to_string()
        } else {
            t.name.clone()
        };
        let indent = if t.parent.is_some() { "  " } else { "" };
        println!(
            "{:<12} {:<28} {:<14} {:>6}  {:<12} {}",
            t.id,
            format!("{indent}{name}")
                .chars()
                .take(28)
                .collect::<String>(),
            t.r#type,
            t.item_count,
            ago(now, t.fetched_at),
            t.parent.as_deref().unwrap_or("")
        );
    }
    println!(
        "{} tabs, {} items",
        tabs.len(),
        tabs.iter().map(|t| t.item_count).sum::<i64>()
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
    }
    Ok(())
}

pub fn events(since_hours: f64, limit: usize, json: bool) -> Result<()> {
    let store = open()?;
    let since = acquisition_store::now() - (since_hours * 3600.0) as i64;
    let ev = store.events_since(since, limit)?;
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
            "{} {:<8} {:<40} {loc}",
            e.at,
            e.kind,
            label.chars().take(40).collect::<String>()
        );
    }
    println!("{} event(s)", ev.len());
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
