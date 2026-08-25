//! `acq pull`: the first real consumer of the daemon.
//!
//! One end-to-end task a user wants — pull every stash tab in a league,
//! keep a snapshot on disk, and on the next run say what changed. It uses
//! only the existing protocol verbs (`submit`, `status`, `list`, `result`)
//! on purpose: every place the walk is awkward is a fact about what the
//! frontend boundary needs, recorded in CONTEXT.md rather than patched here.
//!
//! Snapshots live on the *client's* disk. CONTEXT.md defers caching API
//! results behind the daemon; a frontend remembering what it fetched is not
//! that, and it is what every frontend will do.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use acquisition_core::job::{JobInfo, JobState, Outcome};
use acquisition_core::protocol::{Request, Response};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::client::Client;

/// Everything one pull learned, keyed for diffing. Items are keyed by the
/// API's item `id` (stable across fetches; the C++ app relies on it too).
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub taken_at_unix: u64,
    pub provider: String,
    pub league: String,
    pub deep: bool,
    /// Root `refresh` job id and the number of jobs in its subtree, so a
    /// snapshot can be matched against the daemon log and send journal.
    pub root_job: u64,
    pub jobs: usize,
    pub tabs: BTreeMap<String, Tab>,
    /// Tabs whose fetch failed or was cancelled: `(tab id, reason)`. A
    /// snapshot with errors is still written — it is what we know — but the
    /// diff treats those tabs as unknown, not empty.
    pub errors: BTreeMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Tab {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: String,
    /// Parent tab for a map/unique substash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Item id → the item as the API sent it.
    pub items: BTreeMap<String, Value>,
}

/// What changed between two snapshots. Tabs that errored in either
/// snapshot are excluded from item comparison and listed in `unknown_tabs`.
#[derive(Debug, Default, Serialize, PartialEq)]
pub struct Diff {
    pub tabs_added: Vec<String>,
    pub tabs_removed: Vec<String>,
    pub unknown_tabs: Vec<String>,
    pub items_added: Vec<ItemRef>,
    pub items_removed: Vec<ItemRef>,
    /// Same item id, different tab.
    pub items_moved: Vec<Move>,
    /// Same id and tab, but the JSON differs (position, stack size, sockets…).
    pub items_changed: Vec<ItemRef>,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct ItemRef {
    pub tab: String,
    pub id: String,
    pub label: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Move {
    pub from: String,
    pub to: String,
    pub id: String,
    pub label: String,
}

impl Diff {
    pub fn is_empty(&self) -> bool {
        self.tabs_added.is_empty()
            && self.tabs_removed.is_empty()
            && self.items_added.is_empty()
            && self.items_removed.is_empty()
            && self.items_moved.is_empty()
            && self.items_changed.is_empty()
    }
}

/// Human label for an item: `name typeLine` with stack size if present.
fn item_label(item: &Value) -> String {
    let name = item.get("name").and_then(Value::as_str).unwrap_or("");
    let ty = item.get("typeLine").and_then(Value::as_str).unwrap_or("?");
    let mut s = if name.is_empty() {
        ty.to_string()
    } else {
        format!("{name} {ty}")
    };
    if let Some(n) = item.get("stackSize").and_then(Value::as_u64) {
        s.push_str(&format!(" ×{n}"));
    }
    s
}

/// Display name for a tab: its name, or for a nameless substash (the API
/// leaves map/unique substash names empty) `parent/id`.
pub fn tab_label(s: &Snapshot, id: &str) -> String {
    let Some(t) = s.tabs.get(id) else {
        return id.to_string();
    };
    let own = if t.name.is_empty() {
        id.to_string()
    } else {
        t.name.clone()
    };
    match &t.parent {
        Some(p) => format!(
            "{}/{own}",
            s.tabs.get(p).map(|x| x.name.as_str()).unwrap_or(p)
        ),
        None => own,
    }
}

/// Pure diff over two snapshots; order of output is deterministic.
pub fn diff<'a>(old: &'a Snapshot, new: &'a Snapshot) -> Diff {
    let mut d = Diff::default();
    // A shallow pull never looks inside map/unique tabs, so substashes seen
    // by only one side of a shallow/deep pair are unknown, not added/removed.
    let substash_only_in = |a: &'a Snapshot, b: &'a Snapshot| {
        a.tabs
            .iter()
            .filter(move |(id, t)| t.parent.is_some() && !b.tabs.contains_key(*id))
            .map(|(id, _)| id)
    };
    let mut unknown: BTreeSet<&String> = old.errors.keys().chain(new.errors.keys()).collect();
    if !new.deep {
        unknown.extend(substash_only_in(old, new));
    }
    if !old.deep {
        unknown.extend(substash_only_in(new, old));
    }
    d.unknown_tabs = unknown.iter().map(|s| s.to_string()).collect();
    d.tabs_added = new
        .tabs
        .keys()
        .filter(|t| !old.tabs.contains_key(*t) && !unknown.contains(t))
        .cloned()
        .collect();
    d.tabs_removed = old
        .tabs
        .keys()
        .filter(|t| !new.tabs.contains_key(*t) && !unknown.contains(t))
        .cloned()
        .collect();

    // item id → (tab, item) across all known tabs.
    let index = |s: &Snapshot| -> BTreeMap<String, (String, Value)> {
        s.tabs
            .iter()
            .filter(|(t, _)| !unknown.contains(t))
            .flat_map(|(t, tab)| {
                tab.items
                    .iter()
                    .map(move |(id, it)| (id.clone(), (t.clone(), it.clone())))
            })
            .collect()
    };
    let before = index(old);
    let after = index(new);
    for (id, (tab, item)) in &after {
        match before.get(id) {
            None => d.items_added.push(ItemRef {
                tab: tab.clone(),
                id: id.clone(),
                label: item_label(item),
            }),
            Some((old_tab, _)) if old_tab != tab => d.items_moved.push(Move {
                from: old_tab.clone(),
                to: tab.clone(),
                id: id.clone(),
                label: item_label(item),
            }),
            Some((_, old_item)) if old_item != item => d.items_changed.push(ItemRef {
                tab: tab.clone(),
                id: id.clone(),
                label: item_label(item),
            }),
            Some(_) => {}
        }
    }
    for (id, (tab, item)) in &before {
        if !after.contains_key(id) {
            d.items_removed.push(ItemRef {
                tab: tab.clone(),
                id: id.clone(),
                label: item_label(item),
            });
        }
    }
    d
}

/// `$ACQ_SNAPSHOTS`, else `~/.local/share/acquisition-playground/snapshots`.
pub fn default_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("ACQ_SNAPSHOTS") {
        return PathBuf::from(p);
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".local/share/acquisition-playground/snapshots")
}

fn league_dir(dir: &Path, provider: &str, league: &str) -> PathBuf {
    dir.join(provider).join(league)
}

/// Snapshots are `<unix secs, zero-padded>.json`; the league dir also holds
/// `inflight.json` and temp files, which are not snapshots.
fn is_snapshot_file(p: &Path) -> bool {
    p.extension().is_some_and(|e| e == "json")
        && p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
}

/// Newest snapshot file in the league dir (names sort by time).
fn newest(dir: &Path) -> Result<Option<PathBuf>> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Ok(None);
    };
    let mut names: Vec<PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| is_snapshot_file(p))
        .collect();
    names.sort();
    Ok(names.pop())
}

pub fn load(path: &Path) -> Result<Snapshot> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

fn save(dir: &Path, snap: &Snapshot) -> Result<PathBuf> {
    let d = league_dir(dir, &snap.provider, &snap.league);
    std::fs::create_dir_all(&d)?;
    let path = d.join(format!("{:010}.json", snap.taken_at_unix));
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(snap)?)?;
    std::fs::rename(&tmp, &path)?;
    Ok(path)
}

async fn result(client: &mut Client, id: u64) -> Result<Outcome> {
    match client.request(&Request::Result { id }).await? {
        Response::Result { outcome, .. } => Ok(outcome),
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    }
}

async fn list(client: &mut Client) -> Result<Vec<JobInfo>> {
    match client.request(&Request::List).await? {
        Response::Jobs { jobs } => Ok(jobs),
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    }
}

async fn daemon_pid(client: &mut Client) -> Result<u32> {
    match client.request(&Request::DaemonStatus).await? {
        Response::DaemonStatus { pid, .. } => Ok(pid),
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    }
}

/// A refresh this client submitted and may not have finished collecting:
/// written at submit, removed after the snapshot is saved. Job ids restart
/// per daemon lifetime, so the daemon pid is part of the identity.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Inflight {
    daemon_pid: u32,
    root: u64,
    params: Value,
}

fn inflight_path(dir: &Path, provider: &str, league: &str) -> PathBuf {
    league_dir(dir, provider, league).join("inflight.json")
}

/// The recorded in-flight refresh, if the daemon that ran it is still the
/// one we are talking to and the job is still what we submitted.
async fn reattachable(client: &mut Client, path: &Path, params: &Value) -> Result<Option<u64>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(None);
    };
    let Ok(rec) = serde_json::from_str::<Inflight>(&text) else {
        return Ok(None);
    };
    if rec.daemon_pid != daemon_pid(client).await? || &rec.params != params {
        return Ok(None);
    }
    match client.request(&Request::Status { id: rec.root }).await? {
        Response::Status { job } if job.kind == "refresh" && job.params == rec.params => {
            Ok(Some(rec.root))
        }
        _ => Ok(None),
    }
}

/// Ids in `root`'s subtree, from a job list.
fn subtree(jobs: &[JobInfo], root: u64) -> Vec<&JobInfo> {
    let mut members: BTreeSet<u64> = BTreeSet::from([root]);
    // Children always have larger ids than parents, so one ascending pass
    // closes the set.
    let mut sorted: Vec<&JobInfo> = jobs.iter().collect();
    sorted.sort_by_key(|j| j.id);
    sorted
        .into_iter()
        .filter(|j| {
            j.id == root
                || j.parent.is_some_and(|p| members.contains(&p)) && {
                    members.insert(j.id);
                    true
                }
        })
        .collect()
}

/// Submit the refresh, wait for the whole subtree, collect every result.
async fn collect(
    client: &mut Client,
    provider: &str,
    league: &str,
    deep: bool,
    dir: &Path,
    quiet: bool,
) -> Result<Snapshot> {
    let params = json!({ "league": league, "all": true, "deep": deep });
    let inflight = inflight_path(dir, provider, league);
    // A pull that died mid-wait (Ctrl-C, closed terminal) left its refresh
    // running in the daemon; the sends happen either way, so collect them
    // rather than submit a second refresh on top.
    let root = match reattachable(client, &inflight, &params).await? {
        Some(id) => {
            if !quiet {
                println!("reattaching to refresh job {id} still in the daemon");
            }
            id
        }
        None => {
            let submitted_by = format!("cli:{}", std::process::id());
            let id = match client
                .request(&Request::Submit {
                    kind: "refresh".into(),
                    params: params.clone(),
                    priority: 0,
                    submitted_by,
                })
                .await?
            {
                Response::Submitted { id } => id,
                Response::Error { message } => bail!("{message}"),
                other => bail!("unexpected response: {other:?}"),
            };
            let rec = Inflight {
                daemon_pid: daemon_pid(client).await?,
                root: id,
                params: params.clone(),
            };
            std::fs::create_dir_all(inflight.parent().unwrap())?;
            std::fs::write(&inflight, serde_json::to_vec(&rec)?)?;
            id
        }
    };

    // The parent finishes when its last descendant does (CONTEXT.md), so
    // waiting on it is waiting on the tree. Progress comes from `list`.
    let mut last = String::new();
    loop {
        let jobs = list(client).await?;
        let tree = subtree(&jobs, root);
        let Some(me) = tree.iter().find(|j| j.id == root) else {
            bail!("job {root} vanished from the daemon");
        };
        let done = tree.iter().filter(|j| j.state.is_terminal()).count();
        let eta = tree
            .iter()
            .filter_map(|j| j.eta_seconds)
            .max()
            .filter(|&e| e > 0)
            .map(|e| format!(", ~{e}s"))
            .unwrap_or_default();
        let held = tree.iter().filter(|j| j.retries > 0).count();
        let line = format!(
            "pull {league}: {done}/{} jobs done{eta}{}",
            tree.len(),
            if held > 0 {
                format!(", {held} re-queued after 429")
            } else {
                String::new()
            }
        );
        if !quiet && line != last {
            print!("\r\x1b[2K{line}");
            std::io::stdout().flush().ok();
            last = line;
        }
        if me.state.is_terminal() {
            if !quiet {
                println!();
            }
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let jobs = list(client).await?;
    let tree = subtree(&jobs, root);
    let mut snap = Snapshot {
        taken_at_unix: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
        league: league.into(),
        deep,
        root_job: root,
        jobs: tree.len(),
        ..Default::default()
    };
    let root_payload = match result(client, root).await? {
        Outcome::Success { payload } => payload,
        Outcome::Failure { error } => bail!("refresh failed: {error}"),
        Outcome::Cancelled => bail!("refresh was cancelled"),
    };
    snap.provider = root_payload
        .get("provider")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .into();
    // A child's tab is in its own params (`sub` for a substash, else `id`).
    let tab_of = |j: &JobInfo| {
        j.params
            .get("sub")
            .or_else(|| j.params.get("id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("job-{}", j.id))
    };

    let mut ordered: Vec<&JobInfo> = tree.iter().copied().filter(|j| j.id != root).collect();
    ordered.sort_by_key(|j| j.id);
    for job in ordered {
        let tab_id = tab_of(job);
        let payload = match result(client, job.id).await? {
            Outcome::Success { payload } if job.state == JobState::Done => payload,
            Outcome::Success { .. } => {
                snap.errors
                    .insert(tab_id, format!("job {}: {}", job.id, job.state));
                continue;
            }
            Outcome::Failure { error } => {
                snap.errors
                    .insert(tab_id, format!("job {}: {error}", job.id));
                continue;
            }
            Outcome::Cancelled => {
                snap.errors
                    .insert(tab_id, format!("job {}: cancelled", job.id));
                continue;
            }
        };
        let stash = payload.get("stash").cloned().unwrap_or(Value::Null);
        let id = stash
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or(tab_id);
        let items = stash
            .get("items")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|it| {
                it.get("id")
                    .and_then(Value::as_str)
                    .map(|iid| (iid.to_string(), it.clone()))
            })
            .collect();
        snap.tabs.insert(
            id,
            Tab {
                name: stash
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .into(),
                ty: stash
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .into(),
                parent: stash
                    .get("parent")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                items,
            },
        );
    }
    Ok(snap)
}

pub async fn run(
    client: &mut Client,
    league: &str,
    deep: bool,
    dir: &Path,
    json: bool,
) -> Result<()> {
    let provider = client.provider().to_string();
    let snap = collect(client, &provider, league, deep, dir, json).await?;
    let previous = newest(&league_dir(dir, &snap.provider, &snap.league))?;
    let path = save(dir, &snap)?;
    let _ = std::fs::remove_file(inflight_path(dir, &snap.provider, &snap.league));
    let diff = match &previous {
        Some(p) => Some(diff(&load(p)?, &snap)),
        None => None,
    };
    let items: usize = snap.tabs.values().map(|t| t.items.len()).sum();

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "snapshot": path,
                "previous": previous,
                "provider": snap.provider,
                "league": snap.league,
                "root_job": snap.root_job,
                "jobs": snap.jobs,
                "tabs": snap.tabs.len(),
                "items": items,
                "errors": snap.errors,
                "diff": diff,
            }))?
        );
        return Ok(());
    }

    println!(
        "pull {} ({}): {} stashes, {items} items → {}",
        snap.league,
        snap.provider,
        snap.tabs.len(),
        path.display()
    );
    for (tab, why) in &snap.errors {
        println!("  ! tab {tab}: {why}");
    }
    let Some(d) = diff else {
        println!("first snapshot for this league; run again to see changes");
        return Ok(());
    };
    let prev = previous.unwrap();
    if d.is_empty() {
        println!("no changes since {}", prev.display());
        return Ok(());
    }
    println!(
        "since {}: +{} -{} items, {} moved, {} changed; tabs +{} -{}",
        prev.display(),
        d.items_added.len(),
        d.items_removed.len(),
        d.items_moved.len(),
        d.items_changed.len(),
        d.tabs_added.len(),
        d.tabs_removed.len()
    );
    let name = |s: &Snapshot, t: &str| tab_label(s, t);
    for t in &d.tabs_added {
        println!("  + tab {} ({t})", name(&snap, t));
    }
    for t in &d.tabs_removed {
        println!("  - tab {t}");
    }
    const SHOW: usize = 40;
    let mut lines: Vec<String> = Vec::new();
    lines.extend(
        d.items_added
            .iter()
            .map(|i| format!("  + {} [{}]", i.label, name(&snap, &i.tab))),
    );
    lines.extend(
        d.items_removed
            .iter()
            .map(|i| format!("  - {} [{}]", i.label, name(&snap, &i.tab))),
    );
    lines.extend(d.items_moved.iter().map(|m| {
        format!(
            "  → {} [{} → {}]",
            m.label,
            name(&snap, &m.from),
            name(&snap, &m.to)
        )
    }));
    lines.extend(
        d.items_changed
            .iter()
            .map(|i| format!("  ~ {} [{}]", i.label, name(&snap, &i.tab))),
    );
    for l in lines.iter().take(SHOW) {
        println!("{l}");
    }
    if lines.len() > SHOW {
        println!("  … {} more (--json for all)", lines.len() - SHOW);
    }
    if !d.unknown_tabs.is_empty() {
        println!(
            "  ? not compared (errored in one snapshot): {}",
            d.unknown_tabs.join(", ")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, ty: &str, x: u64) -> (String, Value) {
        (
            id.into(),
            json!({ "id": id, "name": "", "typeLine": ty, "x": x, "y": 0 }),
        )
    }

    fn snap(tabs: &[(&str, &[(String, Value)])], errors: &[(&str, &str)]) -> Snapshot {
        Snapshot {
            tabs: tabs
                .iter()
                .map(|(t, items)| {
                    (
                        t.to_string(),
                        Tab {
                            name: t.to_uppercase(),
                            items: items.iter().cloned().collect(),
                            ..Default::default()
                        },
                    )
                })
                .collect(),
            errors: errors
                .iter()
                .map(|(t, e)| (t.to_string(), e.to_string()))
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn identical_snapshots_diff_empty() {
        let a = snap(&[("t1", &[item("i1", "Chaos Orb", 0)])], &[]);
        let b = snap(&[("t1", &[item("i1", "Chaos Orb", 0)])], &[]);
        assert!(diff(&a, &b).is_empty());
    }

    #[test]
    fn added_removed_moved_changed() {
        let a = snap(
            &[
                (
                    "t1",
                    &[
                        item("stay", "A", 0),
                        item("gone", "B", 1),
                        item("mv", "C", 2),
                        item("pos", "D", 3),
                    ],
                ),
                ("t2", &[]),
            ],
            &[],
        );
        let b = snap(
            &[
                (
                    "t1",
                    &[
                        item("stay", "A", 0),
                        item("new", "E", 1),
                        item("pos", "D", 9),
                    ],
                ),
                ("t2", &[item("mv", "C", 0)]),
            ],
            &[],
        );
        let d = diff(&a, &b);
        assert_eq!(
            d.items_added
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            ["new"]
        );
        assert_eq!(
            d.items_removed
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            ["gone"]
        );
        assert_eq!(d.items_moved.len(), 1);
        assert_eq!(
            (d.items_moved[0].from.as_str(), d.items_moved[0].to.as_str()),
            ("t1", "t2")
        );
        assert_eq!(
            d.items_changed
                .iter()
                .map(|i| i.id.as_str())
                .collect::<Vec<_>>(),
            ["pos"]
        );
        assert!(d.tabs_added.is_empty() && d.tabs_removed.is_empty());
    }

    #[test]
    fn tabs_added_and_removed() {
        let a = snap(&[("t1", &[])], &[]);
        let b = snap(&[("t2", &[])], &[]);
        let d = diff(&a, &b);
        assert_eq!(d.tabs_added, ["t2"]);
        assert_eq!(d.tabs_removed, ["t1"]);
    }

    #[test]
    fn errored_tab_is_unknown_not_empty() {
        // t1 failed on the second pull: its items are not "removed", and the
        // tab is not "removed" — we don't know.
        let a = snap(
            &[("t1", &[item("i1", "A", 0)]), ("t2", &[item("i2", "B", 0)])],
            &[],
        );
        let b = snap(&[("t2", &[item("i2", "B", 0)])], &[("t1", "job 9: 503")]);
        let d = diff(&a, &b);
        assert!(d.items_removed.is_empty(), "{d:?}");
        assert!(d.tabs_removed.is_empty(), "{d:?}");
        assert_eq!(d.unknown_tabs, ["t1"]);
        // And an item that moved *into* an errored tab is not "removed" either
        // — it simply cannot be seen; but one appearing in a known tab from
        // an errored one counts as added (we never saw it).
    }

    #[test]
    fn shallow_after_deep_leaves_substashes_unknown() {
        let mut deep = snap(&[("maps", &[]), ("m001", &[item("i1", "Map", 0)])], &[]);
        deep.deep = true;
        deep.tabs.get_mut("m001").unwrap().parent = Some("maps".into());
        let shallow = snap(&[("maps", &[])], &[]);
        let d = diff(&deep, &shallow);
        assert!(
            d.items_removed.is_empty() && d.tabs_removed.is_empty(),
            "{d:?}"
        );
        assert_eq!(d.unknown_tabs, ["m001"]);
        let d = diff(&shallow, &deep);
        assert!(d.items_added.is_empty() && d.tabs_added.is_empty(), "{d:?}");
        assert_eq!(d.unknown_tabs, ["m001"]);
    }

    #[test]
    fn subtree_follows_grandchildren_only() {
        let j = |id, parent| JobInfo {
            id,
            parent,
            kind: "stash".into(),
            state: JobState::Done,
            priority: 0,
            submitted_by: "test".into(),
            eta_seconds: None,
            retries: 0,
            params: Value::Null,
        };
        let jobs = vec![
            j(1, None),
            j(2, Some(1)),
            j(3, Some(2)),
            j(4, None),
            j(5, Some(4)),
            j(6, Some(3)),
        ];
        let ids: Vec<u64> = subtree(&jobs, 1).iter().map(|x| x.id).collect();
        assert_eq!(ids, [1, 2, 3, 6]);
    }

    #[test]
    fn snapshot_round_trips_through_disk() {
        let dir = std::env::temp_dir().join(format!("acq-pull-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut s = snap(&[("t1", &[item("i1", "A", 0)])], &[]);
        s.provider = "mock".into();
        s.league = "Standard".into();
        s.taken_at_unix = 5;
        let p = save(&dir, &s).unwrap();
        assert_eq!(load(&p).unwrap(), s);
        s.taken_at_unix = 7;
        let p2 = save(&dir, &s).unwrap();
        assert_eq!(
            newest(&league_dir(&dir, "mock", "Standard")).unwrap(),
            Some(p2.clone())
        );
        // The in-flight record and a stray temp file are not snapshots.
        let ld = league_dir(&dir, "mock", "Standard");
        std::fs::write(ld.join("inflight.json"), b"{}").unwrap();
        std::fs::write(ld.join("0000000009.json.tmp"), b"{}").unwrap();
        assert_eq!(newest(&ld).unwrap(), Some(p2));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
