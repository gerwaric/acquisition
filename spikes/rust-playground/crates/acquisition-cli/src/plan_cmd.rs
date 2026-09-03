//! The intent layer's CLI surface: `acq policy` reads and writes the
//! per-account sync-policy annotation (through the store crate, under
//! compare-and-swap), and `acq refresh --plan` compiles it into a
//! [`RefreshPlan`] — offline, spending nothing (tracer step 6). A running
//! daemon enriches the plan with its read-only quote; the command never
//! spawns one just to decorate output the planner computed with the
//! daemon down.
//!
//! Text output follows the legibility ruling (CONTEXT.md, "Legible output
//! for the refresh slice", 2026-09-02): one verdict line before detail,
//! actions grouped by kind and parent and counted by reason (`--expand`
//! lists every one), a failure that names the job, its target, the cause
//! and where the evidence is, and the same renderer for a fresh compile
//! and a reviewed file (`--plan=FILE`) — the text is a function of the
//! envelope, which itself never changes for presentation.

use std::collections::BTreeMap;
use std::path::PathBuf;

use acquisition_core::client::{Client, ConnectOptions, is_no_daemon};
use acquisition_core::job::Outcome;
use acquisition_core::protocol::{Quote, QuoteJob, QuoteScope, Request, Response};
use acquisition_core::realm::Realm;
use acquisition_plan::{
    CharacterSkipReason, FetchReason, ListingReason, PlanError, RefreshAction, RefreshPlan,
    SkipReason, plan_refresh, put_sync_policy,
};
use acquisition_store::{
    AccountEntry, Annotations, SYNC_POLICY_KEY, SYNC_POLICY_KIND, SYNC_POLICY_SCOPE, Store,
    account_path,
};
use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::store_cmd;

/// A shape hint for humans; the planner's strict parse is the authority.
const POLICY_EXAMPLE: &str = r#"{"version":3,"realms":{"pc":{"leagues":{"Standard":{"tabs":"all","characters":"all","max_age_seconds":3600}}}}}"#;

/// Groups of this many entities or fewer are listed one per line; larger
/// groups are counted (the ruling's threshold; `--expand` lists every one).
const LIST_UP_TO: usize = 10;

/// The selected account's provider directory, index entry, and annotations
/// file — the latter addressed by the uuid the index maps the account to.
/// An entry without a uuid predates uuid-at-login; intent cannot be bound
/// to it.
fn open_intent() -> Result<(PathBuf, AccountEntry, Annotations)> {
    let (dir, entry) = store_cmd::resolve()?;
    let Some(uuid) = entry.uuid.as_deref() else {
        bail!(
            "account {} has no recorded uuid (a login predating uuid-at-login); \
             log in once with `acq auth` to fix it",
            entry.username
        );
    };
    let annotations = Annotations::open_for(&dir, uuid)?;
    Ok((dir, entry, annotations))
}

pub fn policy_show(json: bool) -> Result<()> {
    let (_, _, annotations) = open_intent()?;
    let row = annotations.get(SYNC_POLICY_SCOPE, SYNC_POLICY_KEY, SYNC_POLICY_KIND)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&row)?);
        return Ok(());
    }
    match row {
        None => {
            println!("no sync policy is set — write one with `acq policy set '{POLICY_EXAMPLE}'`")
        }
        Some(row) => {
            println!("{}", serde_json::to_string_pretty(&row.value)?);
            println!(
                "revision {}, updated {}",
                row.revision,
                store_cmd::ago(acquisition_store::now(), Some(row.updated_at))
            );
        }
    }
    Ok(())
}

pub fn policy_set(value: &str, if_revision: Option<i64>, json: bool) -> Result<()> {
    let text = match value {
        "-" => std::io::read_to_string(std::io::stdin()).context("reading policy from stdin")?,
        v if v.starts_with('@') => {
            let path = &v[1..];
            std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?
        }
        v => v.to_string(),
    };
    let value: Value = serde_json::from_str(&text).context("the policy must be JSON")?;
    let (_, _, mut annotations) = open_intent()?;
    let row = write_policy(&mut annotations, &value, if_revision)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&row)?);
    } else {
        println!(
            "sync policy written (revision {}) — preview the work with `acq refresh --plan`",
            row.revision
        );
    }
    Ok(())
}

/// The CLI's policy write: validate-then-CAS through the planner's shared
/// [`put_sync_policy`] (built once, inherited by every frontend).
///
/// `if_revision` is the CAS at the human boundary: given, the write lands
/// only if the stored revision is exactly that — what the caller reviewed
/// (`acq policy show` prints it) is what they replace, and another
/// frontend's write in between is a structured conflict. Omitted, the
/// write blindly replaces the revision read here, just before the put —
/// the right default for one human at one keyboard, and deliberately this
/// frontend's default rather than the shared function's — but it is still
/// a CAS underneath: a write landing between the read and the put is a
/// structured conflict to retry, never a silent clobber in either
/// direction.
fn write_policy(
    annotations: &mut Annotations,
    value: &Value,
    if_revision: Option<i64>,
) -> Result<acquisition_store::AnnotationRow> {
    let expected = match if_revision {
        Some(revision) => Some(revision),
        None => annotations
            .get(SYNC_POLICY_SCOPE, SYNC_POLICY_KEY, SYNC_POLICY_KIND)?
            .map(|row| row.revision),
    };
    Ok(put_sync_policy(annotations, value, expected)?)
}

/// Read a plan envelope from a path or stdin (`-`) through the planner's
/// validating parse — the only way a serialized plan becomes a
/// [`RefreshPlan`] here.
fn read_plan(source: &str) -> Result<RefreshPlan> {
    let text = match source {
        "-" => std::io::read_to_string(std::io::stdin()).context("reading the plan from stdin")?,
        path => std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?,
    };
    let value: Value = serde_json::from_str(&text)
        .context("the plan must be JSON — the envelope `acq refresh --plan --json` prints")?;
    Ok(RefreshPlan::from_value(&value)?)
}

/// `acq refresh --plan`: compile the stored sync policy into the explicit
/// action set and print it, human or JSON. Nothing is submitted and
/// nothing is sent; the JSON form is the serialized plan envelope itself
/// (self-validating on parse), so it can be reviewed, stored, or handed
/// to `--apply`. `--plan=FILE` renders a reviewed envelope instead — the
/// same renderer, the quote it carries (none is asked for), no store read.
pub async fn refresh_plan(
    realm: Realm,
    league: &str,
    source: Option<&str>,
    json: bool,
    expand: bool,
) -> Result<()> {
    let now = acquisition_store::now();
    let (plan, quote_note) = match source {
        Some(source) => (read_plan(source)?, None),
        None => {
            let (dir, entry, annotations) = open_intent()?;
            let store = Store::open(&account_path(&dir, &entry.username))?;
            let snapshot = store.refresh_snapshot(realm.as_str(), league, &annotations)?;
            let provider = store_cmd::provider();
            let plan = match plan_refresh(provider, &snapshot, now) {
                Err(PlanError::NoSyncPolicy) => bail!(
                    "no sync policy is set for {} — declare one first, e.g. \
                     `acq policy set '{POLICY_EXAMPLE}'`",
                    entry.username
                ),
                other => other?,
            };
            try_quote(plan).await
        }
    };
    if json {
        // Stdout is the plan envelope, nothing else, so it can be piped;
        // why the quote is absent is diagnostics, not part of the plan.
        if let Some(note) = &quote_note {
            eprintln!("note: {note}");
        }
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }
    print!(
        "{}",
        render_plan(&plan, quote_note.as_deref(), now, expand, source)
    );
    Ok(())
}

/// `acq refresh --apply`: execute a plan — exactly its actions, as one
/// `apply` parent job the daemon admits or refuses whole (tracer step 7).
/// With no source, the stored policy is compiled right now (the normal
/// one-keyboard loop); a source is a reviewed plan envelope from
/// `refresh --plan --json`, re-validated by the planner's own parse. The
/// staleness gate runs before any daemon contact, and an empty plan never
/// contacts one — there is nothing to spend. After the parent finishes the
/// CLI reads the store back (a frontend read, no daemon) and says what
/// landed: "what changed" is the store's to answer, not the job's.
pub async fn refresh_apply(
    realm_flag: Option<Realm>,
    league_flag: Option<&str>,
    plan_source: Option<&str>,
    max_requests: Option<u64>,
    json: bool,
) -> Result<()> {
    let (dir, entry, annotations) = open_intent()?;
    let provider = store_cmd::provider();
    let store = Store::open(&account_path(&dir, &entry.username))?;
    let plan = match plan_source {
        Some(source) => read_plan(source)?,
        None => {
            let league = league_flag.unwrap_or("Standard");
            let realm = realm_flag.unwrap_or(Realm::DEFAULT);
            let snapshot = store.refresh_snapshot(realm.as_str(), league, &annotations)?;
            match plan_refresh(provider, &snapshot, acquisition_store::now()) {
                Err(PlanError::NoSyncPolicy) => bail!(
                    "no sync policy is set for {} — declare one first, e.g. \
                     `acq policy set '{POLICY_EXAMPLE}'`",
                    entry.username
                ),
                other => other?,
            }
        }
    };
    check_plan_applies(
        &plan,
        provider,
        &entry,
        realm_flag,
        league_flag,
        &annotations,
    )?;
    if plan.actions.is_empty() {
        // A strict subset of zero actions is satisfied by doing nothing;
        // no daemon is contacted for it. "Authorizes no requests" is the
        // honest claim — a plan can be empty for reasons besides freshness
        // (unknown ids, folders, orphaned substashes), and the plan's own
        // skipped/unknown reporting says which.
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "applied": false,
                    "requests": 0,
                    "note": "nothing to do: the plan authorizes no requests",
                    "skipped": {
                        "tabs": plan.skipped_tabs.len(),
                        "characters": plan.skipped_characters.len(),
                    },
                })
            );
        } else {
            print!("{}", render_nothing_to_do(&plan));
        }
        return Ok(());
    }
    let params = plan.apply_params(max_requests);
    let before = store.status()?;
    let started = acquisition_store::now();
    // Applying spends, so the interactive connect policy (lazy spawn
    // included) is the right one here — unlike the quote path, which
    // promises to spend nothing.
    let mut client = crate::connect(true).await?;
    let account = entry.username.clone();
    let id = match client
        .request(&Request::Submit {
            kind: "apply".into(),
            params,
            priority: 0,
            submitted_by: format!("cli:{}", std::process::id()),
            account: Some(account),
        })
        .await?
    {
        Response::Submitted { id } => id,
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    };
    if !json {
        println!("applying {} as job {id}", requests(plan.logical_requests));
    }
    let outcome = crate::wait_for_job(&mut client, id, json).await?;
    let changes = StoreChanges::since(&store, &before, started)?;
    if json {
        let mut value = serde_json::to_value(&outcome)?;
        value["store_changes"] = serde_json::to_value(&changes)?;
        println!("{}", serde_json::to_string_pretty(&value)?);
        return match outcome {
            Outcome::Failure { .. } => Err(crate::AlreadyReported.into()),
            _ => Ok(()),
        };
    }
    let report = crate::report_apply(&mut client, id, &outcome).await;
    print!("{report}");
    println!("{}", changes.line());
    if let Outcome::Failure { .. } = outcome {
        println!(
            "daemon log: {}",
            acquisition_core::daemon::log_path().display()
        );
        return Err(crate::AlreadyReported.into());
    }
    Ok(())
}

/// What the store recorded between an apply's submit and its end: the
/// answer to "what changed", read from the facts rather than inferred from
/// the job. Additive on `--apply --json` as `store_changes`.
#[derive(Debug, Default, serde::Serialize)]
pub(crate) struct StoreChanges {
    pub responses: i64,
    pub added: usize,
    pub changed: usize,
    pub moved: usize,
    pub removed: usize,
    pub locations: usize,
}

impl StoreChanges {
    fn since(store: &Store, before: &acquisition_store::Status, started: i64) -> Result<Self> {
        let after = store.status()?;
        let events = store.events_since(started, 1_000_000)?;
        let mut out = StoreChanges {
            responses: after.responses - before.responses,
            ..Default::default()
        };
        let mut locations = std::collections::BTreeSet::new();
        for e in &events {
            match e.kind.as_str() {
                "added" => out.added += 1,
                "changed" => out.changed += 1,
                "moved" => out.moved += 1,
                "removed" => out.removed += 1,
                _ => {}
            }
            for loc in [&e.from_location, &e.to_location].into_iter().flatten() {
                locations.insert(loc.clone());
            }
        }
        out.locations = locations.len();
        Ok(out)
    }

    /// `changed: +98 items added, 3 changed at 5 locations; 112 responses
    /// recorded (acq store events)` — nonzero kinds only.
    fn line(&self) -> String {
        let mut parts = Vec::new();
        if self.added > 0 {
            parts.push(format!("+{} added", self.added));
        }
        if self.changed > 0 {
            parts.push(format!("~{} changed", self.changed));
        }
        if self.moved > 0 {
            parts.push(format!(">{} moved", self.moved));
        }
        if self.removed > 0 {
            parts.push(format!("-{} removed", self.removed));
        }
        let responses = format!(
            "{} response{} recorded",
            self.responses,
            if self.responses == 1 { "" } else { "s" }
        );
        if parts.is_empty() {
            format!("changed: no items — {responses}, nothing added, changed, moved, or removed")
        } else {
            format!(
                "changed: {} at {} location{} since the apply started; {responses} (acq store events)",
                parts.join(", "),
                self.locations,
                if self.locations == 1 { "" } else { "s" }
            )
        }
    }
}

/// The step-7 staleness gate (CONTEXT.md, decided 2026-09-01), via the
/// planner's shared [`RefreshPlan::check_spendable`]: a plan is spent only
/// while the intent it derives from still stands, and only on the identity
/// it names. The daemon is intent-blind, so the comparison happens
/// frontend-side, against a fresh read of the policy row; the CLI adds
/// only its own flag-conflict check (`--league` naming a different league
/// than the reviewed envelope is caller confusion, refused before spend).
fn check_plan_applies(
    plan: &RefreshPlan,
    provider: &str,
    entry: &AccountEntry,
    realm_flag: Option<Realm>,
    league_flag: Option<&str>,
    annotations: &Annotations,
) -> Result<()> {
    if let Some(realm) = realm_flag
        && realm != plan.realm
    {
        bail!(
            "--realm {realm} conflicts with the plan's realm {}",
            plan.realm
        );
    }
    if let Some(league) = league_flag
        && league != plan.league
    {
        bail!(
            "--league {league} conflicts with the plan's league {:?}",
            plan.league
        );
    }
    plan.check_spendable(provider, entry.uuid.as_deref(), annotations)
        .map_err(|e| match e {
            // The shared message names the remedy; the CLI names its verb.
            acquisition_plan::SpendError::PolicyGone { .. }
            | acquisition_plan::SpendError::PolicyMoved { .. } => {
                anyhow::anyhow!("{e} with `acq refresh --plan`")
            }
            other => anyhow::anyhow!(other),
        })
}

/// The quote attempt is bounded: the plan in hand is the deliverable, and
/// a wedged daemon or socket peer must not keep an already-compiled
/// offline plan from printing.
const QUOTE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// The quote path's connection policy: never spawn, never replace. A
/// version- or provider-mismatched daemon may be a human's live GGG run,
/// and `--plan` promises to spend nothing — the interactive policy's
/// kill-and-respawn is a spend, because the successor resumes the
/// persisted queue. Same rationale as the MCP server's rule.
fn quote_connect_options() -> ConnectOptions {
    ConnectOptions::autonomous(false)
}

/// Best-effort enrichment: ask a *running* daemon to quote the plan's
/// actions (read-only, non-reserving) and attach it. The plan is
/// computable with the daemon down, so no daemon is spawned, replaced,
/// or waited on past [`QUOTE_TIMEOUT`] for this; the plan goes out
/// unquoted with the reason instead.
async fn try_quote(plan: RefreshPlan) -> (RefreshPlan, Option<String>) {
    try_quote_within(plan, QUOTE_TIMEOUT).await
}

/// The one line for the common case, without an OS error: `--plan` never
/// spawns a daemon, so whether spawning is allowed is beside the point.
const NO_DAEMON_NOTE: &str =
    "no quote: no daemon running (the plan needs none; a running daemon adds its ETA)";

async fn try_quote_within(
    plan: RefreshPlan,
    limit: std::time::Duration,
) -> (RefreshPlan, Option<String>) {
    let Some(account) = plan.account_name.clone() else {
        // `with_quote` requires the quote to name exactly the plan's
        // account; with no name on record there is nothing to quote as.
        return (
            plan,
            Some("no quote: the facts record no account name to quote as".into()),
        );
    };
    let jobs: Vec<QuoteJob> = plan
        .actions
        .iter()
        .map(|action| {
            let (kind, params) = action.job();
            QuoteJob {
                kind: kind.into(),
                params,
            }
        })
        .collect();
    let attempt = async {
        let mut client = Client::connect(quote_connect_options())
            .await
            .map_err(|e| {
                if is_no_daemon(&e) {
                    NO_DAEMON_NOTE.to_string()
                } else {
                    format!("no quote: {e:#} — plan compiled offline")
                }
            })?;
        client
            .quote(jobs, Some(account))
            .await
            .map_err(|e| format!("no quote: {e:#} — plan compiled offline"))
    };
    match tokio::time::timeout(limit, attempt).await {
        Err(_) => (
            plan,
            Some(format!(
                "no quote: the daemon did not answer within {limit:?} — plan compiled offline"
            )),
        ),
        Ok(Err(why)) => (plan, Some(why)),
        Ok(Ok(quote)) => match plan.clone().with_quote(quote) {
            Ok(enriched) => (enriched, None),
            // The daemon answered about something other than this plan —
            // a planner/daemon disagreement worth seeing, not hiding.
            Err(e) => (plan, Some(format!("daemon quote rejected: {e}"))),
        },
    }
}

// ---- rendering --------------------------------------------------------------

fn requests(n: u64) -> String {
    format!("{n} request{}", if n == 1 { "" } else { "s" })
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Two-column lines padded to the widest left column (capped), so reasons
/// line up under each other without truncating a name.
fn columns(rows: &[(usize, String, String)]) -> String {
    let width = rows
        .iter()
        .map(|(indent, left, _)| indent + left.chars().count())
        .max()
        .unwrap_or(0)
        .min(72);
    let mut out = String::new();
    for (indent, left, right) in rows {
        let pad = " ".repeat(*indent);
        if right.is_empty() {
            out.push_str(&format!("{pad}{left}\n"));
        } else {
            let used = indent + left.chars().count();
            let gap = " ".repeat(width.saturating_sub(used) + 2);
            out.push_str(&format!("{pad}{left}{gap}{right}\n"));
        }
    }
    out
}

/// The header both the grouped and the expanded forms share.
fn render_header(plan: &RefreshPlan, now: i64) -> String {
    let account = plan.account_name.as_deref().unwrap_or(&plan.account_uuid);
    let mut out = format!(
        "refresh plan: {}{} on {} as {} — policy revision {}\n",
        store_cmd::realm_prefix(plan.realm),
        plan.league,
        plan.provider,
        account,
        plan.basis.policy_revision,
    );
    // Both bases are the facts as of the read; which facets the policy
    // covers shows in the actions and skips, not here.
    let stash = match &plan.basis.stash_listing {
        Some(l) => format!(
            "stash listing {} old (response {})",
            dur(now - l.fetched_at),
            l.response_id
        ),
        None => "stashes never listed".into(),
    };
    let characters = match &plan.basis.character_listing {
        Some(l) => format!(
            "character listing {} old (response {})",
            dur(now - l.fetched_at),
            l.response_id
        ),
        None => "characters never listed".into(),
    };
    out.push_str(&format!("basis: {stash}, {characters}\n\n"));
    out
}

/// The verdict line over the actions: the counts, the wire range, and the
/// prerequisites the envelope names (verbatim — they are the plan's).
fn render_verdict(plan: &RefreshPlan) -> String {
    let prerequisites = match plan.wire_sends.prerequisites.as_slice() {
        [] => String::new(),
        [one] => format!(" (plus {one})"),
        [rest @ .., last] => format!(" (plus {}, and {last})", rest.join(", ")),
    };
    format!(
        "{}, {}..{} wire sends{prerequisites}:\n",
        requests(plan.logical_requests),
        plan.wire_sends.min,
        plan.wire_sends.max,
    )
}

/// The full text of `acq refresh --plan`: header, verdict, the grouped (or
/// expanded) actions, the skips, the quote or why there is none, and what
/// to type next.
pub(crate) fn render_plan(
    plan: &RefreshPlan,
    quote_note: Option<&str>,
    now: i64,
    expand: bool,
    source: Option<&str>,
) -> String {
    let mut out = render_header(plan, now);
    if plan.actions.is_empty() {
        // Same honesty as apply's no-op note: an empty plan is not
        // necessarily "all fresh" — the skip lines carry the reasons.
        out.push_str("nothing to do: the plan authorizes no requests\n");
    } else {
        out.push_str(&render_verdict(plan));
        if expand {
            for action in &plan.actions {
                out.push_str(&format!("  {}\n", describe_action(action)));
            }
        } else {
            out.push_str(&render_grouped_actions(&plan.actions));
        }
    }
    out.push_str(&render_skips(plan));
    // An empty plan has nothing to quote and nothing to apply: the
    // verdict and the skips are the whole of it.
    if plan.actions.is_empty() {
        return out;
    }
    out.push('\n');
    match (&plan.quote, quote_note) {
        (Some(quote), _) => out.push_str(&render_quote(quote, now, expand)),
        // Notes are self-describing ("no quote: …", "daemon quote
        // rejected: …"), so no prefix here.
        (None, Some(note)) => out.push_str(&format!("{note}\n")),
        (None, None) => {}
    }
    {
        let mut apply = String::from("acq refresh --apply");
        match source {
            Some(path) => apply.push_str(&format!("={path}")),
            None => {
                if plan.realm != Realm::DEFAULT {
                    apply.push_str(&format!(" --realm {}", plan.realm));
                }
                if plan.league != "Standard" {
                    apply.push_str(&format!(" --league {}", plan.league));
                }
            }
        }
        if expand {
            out.push_str(&format!("next: {apply}\n"));
        } else {
            out.push_str(&format!(
                "next: {apply}   (--expand lists every action; --json is the envelope)\n"
            ));
        }
    }
    out
}

/// The skip and unknown-id lines, one per facet — counts by reason; the
/// per-entity detail is in the envelope.
fn render_skips(plan: &RefreshPlan) -> String {
    let mut out = String::new();
    if !plan.skipped_tabs.is_empty() {
        out.push_str(&format!("{}\n", summarize_skipped_tabs(plan)));
    }
    if !plan.unknown_tabs.is_empty() {
        out.push_str(&format!(
            "policy names {} tab id(s) the facts lack (reported, never fetched): {}\n",
            plan.unknown_tabs.len(),
            plan.unknown_tabs.join(", ")
        ));
    }
    if !plan.skipped_characters.is_empty() {
        out.push_str(&format!("{}\n", summarize_skipped_characters(plan)));
    }
    if !plan.unknown_characters.is_empty() {
        out.push_str(&format!(
            "policy names {} character id(s) the facts lack (reported, never fetched): {}\n",
            plan.unknown_characters.len(),
            plan.unknown_characters.join(", ")
        ));
    }
    out
}

/// The no-op apply's text: what is fresh and for how long, then the claim.
fn render_nothing_to_do(plan: &RefreshPlan) -> String {
    let fresh_tabs = plan
        .skipped_tabs
        .iter()
        .filter(|t| matches!(t.reason, SkipReason::Fresh { .. }))
        .count();
    let fresh_characters = plan
        .skipped_characters
        .iter()
        .filter(|c| matches!(c.reason, CharacterSkipReason::Fresh { .. }))
        .count();
    let mut fresh = Vec::new();
    if fresh_tabs > 0 {
        fresh.push(format!("{} covered tab(s)", fresh_tabs));
    }
    if fresh_characters > 0 {
        fresh.push(format!("{} character(s)", fresh_characters));
    }
    let mut out = if fresh.is_empty() {
        "nothing to do: the plan authorizes no requests\n".to_string()
    } else {
        format!(
            "nothing to do: {} {} fresh (within {} s); the plan authorizes no requests\n",
            fresh.join(" and "),
            if fresh_tabs + fresh_characters == 1 {
                "is"
            } else {
                "are"
            },
            plan.max_age_seconds
        )
    };
    // Any skip that is not freshness still deserves its line.
    let other_tabs = plan.skipped_tabs.len() - fresh_tabs;
    let other_characters = plan.skipped_characters.len() - fresh_characters;
    if other_tabs > 0 || !plan.unknown_tabs.is_empty() {
        out.push_str(&format!("{}\n", summarize_skipped_tabs(plan)));
    }
    if other_characters > 0 || !plan.unknown_characters.is_empty() {
        out.push_str(&format!("{}\n", summarize_skipped_characters(plan)));
    }
    if !plan.unknown_tabs.is_empty() {
        out.push_str(&format!(
            "policy names {} tab id(s) the facts lack: {}\n",
            plan.unknown_tabs.len(),
            plan.unknown_tabs.join(", ")
        ));
    }
    if !plan.unknown_characters.is_empty() {
        out.push_str(&format!(
            "policy names {} character id(s) the facts lack: {}\n",
            plan.unknown_characters.len(),
            plan.unknown_characters.join(", ")
        ));
    }
    out
}

/// One fetch in a group, for the grouped renderer.
struct Entity<'a> {
    /// `id "name" (type)` — what the expanded line says minus the verb.
    label: String,
    reason: &'a FetchReason,
    /// A substash's parent tab id.
    parent: Option<&'a str>,
}

/// Actions grouped by kind and parent, counted by reason (the ruling's
/// rule 2): listings stay single lines; fetches of the same kind form a
/// group, listed one per line up to [`LIST_UP_TO`] and counted beyond,
/// substashes broken down by parent. The order is the planner's: the tab
/// facet's block, then the character facet's.
fn render_grouped_actions(actions: &[RefreshAction]) -> String {
    let mut rows: Vec<(usize, String, String)> = Vec::new();
    let mut tabs: Vec<Entity> = Vec::new();
    let mut substashes: Vec<Entity> = Vec::new();
    let mut characters: Vec<Entity> = Vec::new();
    // Substash counts per parent, for the parents' own lines.
    let mut under: BTreeMap<&str, usize> = BTreeMap::new();
    for action in actions {
        if let RefreshAction::FetchSubstash { parent, .. } = action {
            *under.entry(parent.as_str()).or_default() += 1;
        }
    }
    let flush_tabs = |rows: &mut Vec<(usize, String, String)>,
                      tabs: &mut Vec<Entity>,
                      substashes: &mut Vec<Entity>| {
        // The substash qualifier reads the tab group before it is emitted
        // (emitting clears it): "under 2 of those tabs" when every parent
        // is itself fetched in this plan, "under 2 tabs" otherwise.
        let parents: std::collections::BTreeSet<&str> =
            substashes.iter().filter_map(|e| e.parent).collect();
        let named_above = parents
            .iter()
            .filter(|p| tabs.iter().any(|t| t.label.starts_with(&format!("{p} "))))
            .count();
        let qualifier = if !parents.is_empty() && named_above == parents.len() {
            format!(" under {} of those tabs", parents.len())
        } else {
            format!(" under {}", plural(parents.len(), "tab", "tabs"))
        };
        if !tabs.is_empty() {
            emit_group(rows, tabs, "tab", "tabs", "", &under);
        }
        if !substashes.is_empty() {
            emit_group(
                rows,
                substashes,
                "substash",
                "substashes",
                &qualifier,
                &under,
            );
        }
        tabs.clear();
        substashes.clear();
    };
    for action in actions {
        match action {
            RefreshAction::ListStashes { league, reason, .. } => {
                flush_tabs(&mut rows, &mut tabs, &mut substashes);
                rows.push((2, format!("list stashes {league}"), listing_why(reason)));
            }
            RefreshAction::FetchTab {
                id,
                name,
                tab_type,
                reason,
                ..
            } => tabs.push(Entity {
                label: format!("{id} {} ({tab_type})", label(name)),
                reason,
                parent: None,
            }),
            RefreshAction::FetchSubstash {
                parent,
                id,
                name,
                tab_type,
                reason,
                ..
            } => substashes.push(Entity {
                label: format!("{parent}/{id} {} ({tab_type})", label(name)),
                reason,
                parent: Some(parent),
            }),
            RefreshAction::ListCharacters { realm, reason } => {
                flush_tabs(&mut rows, &mut tabs, &mut substashes);
                rows.push((
                    2,
                    format!("list characters ({realm}, realm-wide)"),
                    listing_why(reason),
                ));
            }
            RefreshAction::FetchCharacter {
                id,
                name,
                league,
                reason,
                ..
            } => {
                flush_tabs(&mut rows, &mut tabs, &mut substashes);
                characters.push(Entity {
                    label: format!("{id} {}", label(name)),
                    reason,
                    parent: None,
                });
                // The league qualifier rides on the group; every character
                // action in one plan is in the envelope's league.
                let _ = league;
            }
        }
    }
    flush_tabs(&mut rows, &mut tabs, &mut substashes);
    if !characters.is_empty() {
        let league = actions
            .iter()
            .find_map(|a| match a {
                RefreshAction::FetchCharacter { league, .. } => Some(league.as_str()),
                _ => None,
            })
            .unwrap_or("");
        emit_group(
            &mut rows,
            &mut characters,
            "character",
            "characters",
            &format!(" in {league}"),
            &under,
        );
    }
    columns(&rows)
}

/// One group: its count line with the breakdown by reason, then either
/// every entity (small groups) or the per-parent breakdown (substashes).
fn emit_group(
    rows: &mut Vec<(usize, String, String)>,
    entities: &mut Vec<Entity>,
    one: &str,
    many: &str,
    qualifier: &str,
    under: &BTreeMap<&str, usize>,
) {
    let n = entities.len();
    rows.push((
        2,
        format!("fetch {}{qualifier}", plural(n, one, many)),
        reason_breakdown(entities.iter().map(|e| e.reason)),
    ));
    if n <= LIST_UP_TO {
        for e in entities.iter() {
            let mut right = fetch_why(e.reason);
            if e.parent.is_none()
                && let Some(id) = e.label.split(' ').next()
                && let Some(k) = under.get(id)
            {
                right.push_str(&format!(
                    "  + {} below",
                    plural(*k, "substash", "substashes")
                ));
            }
            rows.push((4, e.label.clone(), right));
        }
    } else if entities.iter().any(|e| e.parent.is_some()) {
        // Counted substashes: which parents, how many each — the two
        // facts a reviewer wants before spending sixty-four requests.
        let mut per_parent: Vec<(&str, usize)> = Vec::new();
        for e in entities.iter() {
            let Some(p) = e.parent else { continue };
            match per_parent.iter_mut().find(|(id, _)| *id == p) {
                Some((_, k)) => *k += 1,
                None => per_parent.push((p, 1)),
            }
        }
        let parts: Vec<String> = per_parent
            .iter()
            .map(|(p, k)| format!("{k} under {p}"))
            .collect();
        if parts.len() <= 2 {
            rows.push((4, parts.join(", "), String::new()));
        } else {
            for part in parts {
                rows.push((4, part, String::new()));
            }
        }
    }
    entities.clear();
}

/// `5 stale (13h)`, `64 stale (13h–14h)`, `41 never fetched`, `3 stale
/// (2d), 2 listing disagrees` — kinds in order of first appearance.
fn reason_breakdown<'a>(reasons: impl Iterator<Item = &'a FetchReason>) -> String {
    // (label, count, min age, max age)
    let mut kinds: Vec<(&'static str, usize, i64, i64)> = Vec::new();
    for r in reasons {
        let (label, age) = match r {
            FetchReason::NeverFetched => ("never fetched", None),
            FetchReason::Stale { age_seconds } => ("stale", Some(*age_seconds)),
            FetchReason::ListedCountDisagrees { .. } => ("listing disagrees", None),
            FetchReason::ListedExperienceDisagrees { .. } => ("played since listed", None),
            FetchReason::ListedLeagueDisagrees { .. } => ("league changed", None),
        };
        match kinds.iter_mut().find(|(l, ..)| *l == label) {
            Some((_, n, lo, hi)) => {
                *n += 1;
                if let Some(a) = age {
                    *lo = (*lo).min(a);
                    *hi = (*hi).max(a);
                }
            }
            None => kinds.push((label, 1, age.unwrap_or(0), age.unwrap_or(0))),
        }
    }
    kinds
        .iter()
        .map(|(label, n, lo, hi)| {
            if *label == "stale" {
                let (lo, hi) = (dur(*lo), dur(*hi));
                if lo == hi {
                    format!("{n} stale ({lo})")
                } else {
                    format!("{n} stale ({lo}–{hi})")
                }
            } else {
                format!("{n} {label}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn listing_why(reason: &ListingReason) -> String {
    match reason {
        ListingReason::NeverListed => "never listed".into(),
        ListingReason::Stale { age_seconds } => stale(*age_seconds),
    }
}

/// The expanded form: one line per action, the verb first.
fn describe_action(action: &RefreshAction) -> String {
    match action {
        RefreshAction::ListStashes { league, reason, .. } => {
            format!("list stashes {league:<30} {}", listing_why(reason))
        }
        RefreshAction::FetchTab {
            id,
            name,
            tab_type,
            reason,
            ..
        } => format!(
            "fetch tab {:<34} {}",
            format!("{id} {} ({tab_type})", label(name)),
            fetch_why(reason)
        ),
        RefreshAction::FetchSubstash {
            parent,
            id,
            name,
            tab_type,
            reason,
            ..
        } => format!(
            "fetch substash {:<29} {}",
            format!("{parent}/{id} {} ({tab_type})", label(name)),
            fetch_why(reason)
        ),
        RefreshAction::ListCharacters { realm, reason } => format!(
            "list characters {:<27} {}",
            format!("({realm}, realm-wide)"),
            listing_why(reason)
        ),
        // The full id: matching is exact, and a prefix cannot be pasted
        // into a policy (CONTEXT.md, 2026-09-02).
        RefreshAction::FetchCharacter {
            id, name, reason, ..
        } => format!(
            "fetch character {:<28} {}",
            format!("{id} {}", label(name)),
            fetch_why(reason)
        ),
    }
}

fn label(name: &str) -> String {
    if name.is_empty() {
        "(unnamed)".into()
    } else {
        format!("{name:?}")
    }
}

/// A saturated age from the planner, human-shaped: "45s", "12m", "3h", "9d".
fn dur(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < 90 {
        format!("{s}s")
    } else if s < 5400 {
        format!("{}m", s / 60)
    } else if s < 172_800 {
        format!("{}h", s / 3600)
    } else {
        format!("{}d", s / 86400)
    }
}

fn stale(age_seconds: i64) -> String {
    format!("stale for {}", dur(age_seconds))
}

fn fetch_why(reason: &FetchReason) -> String {
    match reason {
        FetchReason::NeverFetched => "never fetched".into(),
        FetchReason::Stale { age_seconds } => stale(*age_seconds),
        FetchReason::ListedCountDisagrees { listed, held } => {
            format!("listing says {listed} item(s), store holds {held}")
        }
        FetchReason::ListedExperienceDisagrees { listed, held } => {
            format!("listing says experience {listed}, last fetch had {held} — played since")
        }
        FetchReason::ListedLeagueDisagrees { listed, held } => {
            format!("listing says league {listed}, last fetch said {held}")
        }
    }
}

/// One line of counts by reason — a real account skips dozens of fresh
/// characters, and the full per-character detail is in the JSON envelope.
fn summarize_skipped_characters(plan: &RefreshPlan) -> String {
    let (mut fresh, mut awaiting, mut no_league, mut deleted, mut expired) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for c in &plan.skipped_characters {
        match c.reason {
            CharacterSkipReason::Fresh { .. } => fresh += 1,
            CharacterSkipReason::AwaitingListing => awaiting += 1,
            CharacterSkipReason::NoLeague => no_league += 1,
            CharacterSkipReason::Deleted => deleted += 1,
            CharacterSkipReason::Expired => expired += 1,
        }
    }
    let mut parts = Vec::new();
    if fresh > 0 {
        parts.push(format!("{fresh} fresh"));
    }
    if awaiting > 0 {
        parts.push(format!("{awaiting} awaiting the listing"));
    }
    if no_league > 0 {
        parts.push(format!(
            "{no_league} with no league — outside every league's coverage"
        ));
    }
    if deleted > 0 {
        parts.push(format!("{deleted} listed as deleted — never fetched"));
    }
    if expired > 0 {
        parts.push(format!("{expired} listed as expired — never fetched"));
    }
    format!(
        "skipped {} covered character(s): {} (per-character reasons in --json)",
        plan.skipped_characters.len(),
        parts.join(", ")
    )
}

/// One line of counts by reason — a real league skips hundreds of fresh
/// tabs, and the full per-tab detail is in the JSON envelope.
fn summarize_skipped_tabs(plan: &RefreshPlan) -> String {
    let (mut fresh, mut folders, mut awaiting, mut orphaned, mut empty) =
        (0usize, 0usize, 0usize, 0usize, 0usize);
    for tab in &plan.skipped_tabs {
        match tab.reason {
            SkipReason::Fresh { .. } => fresh += 1,
            SkipReason::Folder => folders += 1,
            SkipReason::AwaitingListing => awaiting += 1,
            SkipReason::OrphanedParent { .. } => orphaned += 1,
            SkipReason::EmptyStub => empty += 1,
        }
    }
    let mut parts = Vec::new();
    if fresh > 0 {
        parts.push(format!("{fresh} fresh"));
    }
    if folders > 0 {
        parts.push(format!("{folders} folder(s) — never fetched"));
    }
    if awaiting > 0 {
        parts.push(format!("{awaiting} awaiting the listing"));
    }
    if orphaned > 0 {
        parts.push(format!("{orphaned} with an off-record parent"));
    }
    if empty > 0 {
        parts.push(format!("{empty} empty substash stub(s) — nothing to fetch"));
    }
    format!(
        "skipped {} covered tab(s): {} (per-tab reasons in --json)",
        plan.skipped_tabs.len(),
        parts.join(", ")
    )
}

/// The quote: one line per scope, the windows when the policy is known,
/// and what it does not cover on one line (the daemon's own phrases, cut at
/// their explanatory tail; `--expand` prints them whole, notes included).
fn render_quote(quote: &Quote, now: i64, expand: bool) -> String {
    let mut out = format!(
        "quote (the running daemon's estimate, observed {}):\n",
        store_cmd::ago(now, Some(quote.observed_at))
    );
    if let Some(cause) = &quote.halted {
        // Report the cause only: a tripwire halt has a post-violation wait
        // before `reset-tripwire` is appropriate, and a ceiling halt cannot
        // be reset mid-lifetime at all — LIVE-TESTING.md governs clearing.
        out.push_str(&format!(
            "  HALTED: {cause} — nothing sends while the halt stands, so every \
             estimate below is a floor (clearing it is governed by LIVE-TESTING.md)\n"
        ));
    }
    let width = quote
        .scopes
        .iter()
        .map(|s| s.key.chars().count())
        .max()
        .unwrap_or(0);
    for scope in &quote.scopes {
        out.push_str(&render_scope(scope, width, expand));
    }
    if !quote.not_covered.is_empty() {
        if expand {
            for line in &quote.not_covered {
                out.push_str(&format!("  not covered: {line}\n"));
            }
        } else {
            let short: Vec<&str> = quote
                .not_covered
                .iter()
                .map(|l| l.split(" — ").next().unwrap_or(l))
                .collect();
            out.push_str(&format!("  not covered: {}\n", short.join("; ")));
        }
    }
    out
}

fn render_scope(scope: &QuoteScope, width: usize, expand: bool) -> String {
    let unlearned = scope.notes.iter().any(|n| n.contains("not yet learned"));
    let eta = match scope.eta_seconds {
        Some(0) => "ready now".into(),
        Some(s) => format!("last could dispatch in ~{s}s"),
        None if unlearned => "no ETA until its policy is learned (probe first)".into(),
        None => "no ETA".into(),
    };
    let count = format!(
        "{} request{},",
        scope.requests,
        if scope.requests == 1 { "" } else { "s" }
    );
    let mut out = format!(
        "  {:<width$}  {count:<12} {} queued ahead — {eta}\n",
        scope.key, scope.queued_ahead,
    );
    for rule in &scope.rules {
        for window in &rule.windows {
            let observed = scope
                .observed_seconds_ago
                .map(|s| format!(", seen {s}s ago"))
                .unwrap_or_default();
            let restricted = if window.restricted_secs > 0 {
                format!(", RESTRICTED {}s", window.restricted_secs)
            } else {
                String::new()
            };
            out.push_str(&format!(
                "      {} {}/{} hits per {}s{observed}{restricted}\n",
                rule.name, window.hits, window.max_hits, window.period_secs
            ));
        }
    }
    if expand || !unlearned {
        for note in &scope.notes {
            out.push_str(&format!("      note: {note}\n"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use acquisition_store::{AnnotationRow, RefreshSnapshot};
    use serde_json::json;

    fn annotations() -> Annotations {
        Annotations::open_memory_for("u-cli").unwrap()
    }

    fn example() -> Value {
        serde_json::from_str(POLICY_EXAMPLE).unwrap()
    }

    #[test]
    fn a_policy_is_validated_before_it_is_stored() {
        let mut a = annotations();
        // A typo'd field is refused by the planner's strict parse, and
        // nothing lands on disk — intent is never half-honored, and the
        // write surface enforces it too, not only plan time.
        let typo = json!({
            "version": 1,
            "leagues": { "Standard": { "tabs": "all", "max_age_secs": 60 } }
        });
        assert!(write_policy(&mut a, &typo, None).is_err());
        assert!(
            a.get(SYNC_POLICY_SCOPE, SYNC_POLICY_KEY, SYNC_POLICY_KIND)
                .unwrap()
                .is_none()
        );
        // The example the CLI prints must itself be a valid policy.
        let first = write_policy(&mut a, &example(), None).unwrap();
        assert_eq!(first.revision, 1);
        // An unconditional rewrite continues the revision sequence.
        let second = write_policy(&mut a, &example(), None).unwrap();
        assert_eq!(second.revision, 2);
    }

    #[test]
    fn if_revision_is_the_cas_at_the_human_boundary() {
        let mut a = annotations();
        let first = write_policy(&mut a, &example(), None).unwrap();
        // A stale expectation (someone else wrote since the review) is a
        // structured conflict naming the current revision, and the stored
        // value is untouched.
        write_policy(&mut a, &example(), Some(first.revision)).unwrap();
        let err = write_policy(
            &mut a,
            &json!({"version": 1, "leagues": {}}),
            Some(first.revision),
        )
        .unwrap_err();
        assert!(err.to_string().contains("revision 2"), "{err}");
        let held = a
            .get(SYNC_POLICY_SCOPE, SYNC_POLICY_KEY, SYNC_POLICY_KIND)
            .unwrap()
            .unwrap();
        assert_eq!(held.value, example());
        // The reviewed revision, when it still stands, is replaceable.
        let third = write_policy(&mut a, &example(), Some(held.revision)).unwrap();
        assert_eq!(third.revision, 3);
    }

    #[test]
    fn the_quote_path_never_spawns_or_replaces_a_daemon() {
        // The P-scale property behind "spends nothing": the interactive
        // policy's kill-and-respawn is a spend (the successor resumes the
        // persisted queue), so the quote connection must be the
        // autonomous, no-spawn one.
        let opts = quote_connect_options();
        assert!(!opts.spawn, "the quote path must not spawn a daemon");
        assert!(!opts.replace, "the quote path must not replace a daemon");
    }

    /// A plan with no store behind it, for exercising the quote attempt.
    fn tiny_plan() -> RefreshPlan {
        let snapshot = RefreshSnapshot {
            account_uuid: "u-cli".into(),
            account_name: Some("Alice#1234".into()),
            realm: "pc".into(),
            league: "Standard".into(),
            taken_at: 1_000,
            stash_listing: None,
            tabs: Vec::new(),
            character_listing: None,
            characters: Vec::new(),
            policy: Some(AnnotationRow {
                scope: SYNC_POLICY_SCOPE.into(),
                key: SYNC_POLICY_KEY.into(),
                kind: SYNC_POLICY_KIND.into(),
                value: example(),
                revision: 1,
                created_at: 1_000,
                updated_at: 1_000,
            }),
        };
        plan_refresh("mock", &snapshot, 2_000).unwrap()
    }

    #[test]
    fn the_staleness_gate_refuses_moved_intent_and_wrong_identity() {
        let plan = tiny_plan();
        assert_eq!(plan.basis.policy_revision, 1);
        let entry = AccountEntry {
            username: "Alice#1234".into(),
            last_login: 0,
            persisted: true,
            uuid: Some("u-cli".into()),
        };
        let mut a = annotations();
        // Intent deleted since the plan: refused, citing the revision the
        // plan derived from.
        let err = check_plan_applies(&plan, "mock", &entry, None, None, &a).unwrap_err();
        assert!(err.to_string().contains("gone"), "{err}");
        // Intent standing at the plan's revision: applies — and explicit
        // --realm / --league flags that agree are fine; ones that do not
        // are caller confusion, refused before spend.
        write_policy(&mut a, &example(), None).unwrap();
        check_plan_applies(&plan, "mock", &entry, None, None, &a).unwrap();
        check_plan_applies(&plan, "mock", &entry, Some(Realm::Pc), Some("Standard"), &a).unwrap();
        let err =
            check_plan_applies(&plan, "mock", &entry, None, Some("Hardcore"), &a).unwrap_err();
        assert!(err.to_string().contains("Standard"), "{err}");
        let err =
            check_plan_applies(&plan, "mock", &entry, Some(Realm::Xbox), None, &a).unwrap_err();
        assert!(err.to_string().contains("--realm xbox"), "{err}");
        // Intent moved since the plan (the step-7 ruling): refused with
        // both revisions named, remedy = replan.
        write_policy(&mut a, &example(), None).unwrap();
        let err = check_plan_applies(&plan, "mock", &entry, None, None, &a).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("revision 1") && msg.contains("revision 2"),
            "{msg}"
        );
        assert!(msg.contains("replan"), "{msg}");
        // A plan for another identity is never spent here: wrong uuid,
        // wrong provider.
        let other = AccountEntry {
            uuid: Some("u-other".into()),
            ..entry.clone()
        };
        let err = check_plan_applies(&plan, "mock", &other, None, None, &a).unwrap_err();
        assert!(err.to_string().contains("u-cli"), "{err}");
        let err = check_plan_applies(&plan, "ggg", &entry, None, None, &a).unwrap_err();
        assert!(err.to_string().contains("mock"), "{err}");
    }

    #[tokio::test]
    async fn a_wedged_daemon_cannot_block_the_offline_plan() {
        // A socket that accepts connections and never answers the
        // handshake: the bounded quote attempt must hand back the offline
        // plan with the timeout named, not hang the command.
        let dir = std::env::temp_dir().join(format!("acq-p6-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sock = dir.join("q.sock");
        let _ = std::fs::remove_file(&sock);
        let _listener = tokio::net::UnixListener::bind(&sock).unwrap();
        // SAFETY: this is the only test in the binary that touches
        // ACQ_SOCKET, and nothing else in the test process reads it
        // concurrently.
        unsafe { std::env::set_var("ACQ_SOCKET", &sock) };
        let plan = tiny_plan();
        // The outer watchdog outlives the tested bound on purpose: if the
        // production timeout is ever removed, this test must fail cleanly
        // instead of hanging the quality gate.
        let (back, note) = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            try_quote_within(plan.clone(), std::time::Duration::from_millis(200)),
        )
        .await
        .expect(
            "the bounded quote attempt itself never returned — is the production timeout gone?",
        );
        assert_eq!(back, plan, "the compiled plan must come back untouched");
        let note = note.expect("a timed-out quote must say why it is absent");
        assert!(note.contains("did not answer"), "{note}");
    }

    // ---- the grouped renderer (the legibility ruling) ----

    fn tab(id: &str, name: &str, reason: FetchReason) -> RefreshAction {
        RefreshAction::FetchTab {
            realm: Realm::Pc,
            league: "Standard".into(),
            id: id.into(),
            name: name.into(),
            tab_type: "MapStash".into(),
            reason,
        }
    }

    fn substash(parent: &str, id: &str, reason: FetchReason) -> RefreshAction {
        RefreshAction::FetchSubstash {
            realm: Realm::Pc,
            league: "Standard".into(),
            parent: parent.into(),
            id: id.into(),
            name: "1 (Remove-only)".into(),
            tab_type: "MapStash".into(),
            reason,
        }
    }

    fn character(i: usize) -> RefreshAction {
        RefreshAction::FetchCharacter {
            realm: Realm::Pc,
            league: "Standard".into(),
            id: format!("{i:0>64x}"),
            name: format!("Exile{i}"),
            reason: FetchReason::NeverFetched,
        }
    }

    /// The 2026-09-02 shape in miniature: a listing, tabs with substashes
    /// interleaved after their parents, the character listing, and more
    /// characters than the threshold.
    fn wall() -> Vec<RefreshAction> {
        let stale = |h: i64| FetchReason::Stale {
            age_seconds: h * 3600,
        };
        let mut actions = vec![
            RefreshAction::ListStashes {
                realm: Realm::Pc,
                league: "Standard".into(),
                reason: ListingReason::Stale {
                    age_seconds: 13 * 3600,
                },
            },
            tab("dump", "Dump Tab", stale(13)),
            tab("maps", "Maps (Remove-only)", stale(13)),
        ];
        for i in 0..8 {
            actions.push(substash("maps", &format!("m{i}"), stale(14)));
        }
        actions.push(tab("uniq", "Uniques 1 (Remove-only)", stale(13)));
        for i in 0..4 {
            actions.push(substash("uniq", &format!("u{i}"), stale(13)));
        }
        actions.push(RefreshAction::ListCharacters {
            realm: Realm::Pc,
            reason: ListingReason::Stale {
                age_seconds: 13 * 3600,
            },
        });
        for i in 0..12 {
            actions.push(character(i));
        }
        actions
    }

    #[test]
    fn grouping_counts_large_groups_and_lists_small_ones() {
        let text = render_grouped_actions(&wall());
        // Listings stay single lines in the planner's order.
        assert!(text.starts_with("  list stashes Standard"), "{text}");
        // Three tabs: under the threshold, so each is listed with its
        // reason, and the parents say how many substashes follow.
        assert!(text.contains("fetch 3 tabs"), "{text}");
        assert!(text.contains("3 stale (13h)"), "{text}");
        assert!(
            text.contains("maps \"Maps (Remove-only)\" (MapStash)"),
            "{text}"
        );
        assert!(text.contains("+ 8 substashes below"), "{text}");
        assert!(text.contains("+ 4 substashes below"), "{text}");
        // Twelve substashes: over the threshold — counted, broken down by
        // parent and by reason with the age range, never listed.
        assert!(
            text.contains("fetch 12 substashes under 2 of those tabs"),
            "{text}"
        );
        assert!(text.contains("12 stale (13h–14h)"), "{text}");
        assert!(text.contains("8 under maps, 4 under uniq"), "{text}");
        assert!(!text.contains("maps/m0"), "{text}");
        // Twelve characters: counted, no ids in the default view.
        assert!(text.contains("list characters (pc, realm-wide)"), "{text}");
        assert!(text.contains("fetch 12 characters in Standard"), "{text}");
        assert!(text.contains("12 never fetched"), "{text}");
        assert!(!text.contains("Exile0"), "{text}");
        // Order: the tab facet's block, then the character facet's.
        let tabs_at = text.find("fetch 3 tabs").unwrap();
        let subs_at = text.find("fetch 12 substashes").unwrap();
        let chars_at = text.find("fetch 12 characters").unwrap();
        assert!(tabs_at < subs_at && subs_at < chars_at, "{text}");
    }

    #[test]
    fn small_groups_list_every_entity_with_its_reason() {
        let actions = vec![
            tab("a", "A", FetchReason::NeverFetched),
            tab(
                "b",
                "B",
                FetchReason::ListedCountDisagrees {
                    listed: 92,
                    held: 90,
                },
            ),
            character(1),
        ];
        let text = render_grouped_actions(&actions);
        assert!(text.contains("fetch 2 tabs"), "{text}");
        assert!(
            text.contains("1 never fetched, 1 listing disagrees"),
            "{text}"
        );
        assert!(text.contains("b \"B\" (MapStash)"), "{text}");
        assert!(
            text.contains("listing says 92 item(s), store holds 90"),
            "{text}"
        );
        assert!(text.contains("fetch 1 character in Standard"), "{text}");
        assert!(text.contains("\"Exile1\""), "{text}");
    }

    #[test]
    fn the_expanded_form_keeps_one_line_per_action_and_the_json_is_untouched() {
        let mut plan = tiny_plan();
        plan.actions = wall();
        plan.logical_requests = plan.actions.len() as u64;
        let grouped = render_plan(&plan, Some(NO_DAEMON_NOTE), 2_000, false, None);
        let expanded = render_plan(&plan, Some(NO_DAEMON_NOTE), 2_000, true, None);
        assert_eq!(
            expanded.matches("\n  fetch ").count(),
            27,
            "one line per fetch:\n{expanded}"
        );
        assert!(grouped.lines().count() < expanded.lines().count());
        // Both end with the next step and carry the no-quote line once.
        for text in [&grouped, &expanded] {
            assert_eq!(text.matches("no quote").count(), 1, "{text}");
            assert!(text.contains("next: acq refresh --apply"), "{text}");
            assert!(!text.contains("os error"), "{text}");
        }
        assert!(grouped.contains("--expand lists every action"), "{grouped}");
        // Rendered from a file, the next step names that file.
        let from_file = render_plan(&plan, None, 2_000, false, Some("/tmp/p.json"));
        assert!(
            from_file.contains("next: acq refresh --apply=/tmp/p.json"),
            "{from_file}"
        );
    }

    #[test]
    fn an_empty_plan_says_what_is_fresh() {
        let mut plan = tiny_plan();
        plan.skipped_tabs = (0..69)
            .map(|i| acquisition_plan::SkippedTab {
                id: format!("t{i}"),
                name: String::new(),
                reason: SkipReason::Fresh { age_seconds: 60 },
            })
            .collect();
        plan.skipped_characters = (0..41)
            .map(|i| acquisition_plan::SkippedCharacter {
                id: format!("c{i}"),
                name: String::new(),
                reason: CharacterSkipReason::Fresh { age_seconds: 60 },
            })
            .collect();
        plan.actions.clear();
        plan.logical_requests = 0;
        let text = render_nothing_to_do(&plan);
        assert!(
            text.starts_with(
                "nothing to do: 69 covered tab(s) and 41 character(s) are fresh (within 3600 s); the plan authorizes no requests"
            ),
            "{text}"
        );
        let rendered = render_plan(&plan, Some(NO_DAEMON_NOTE), 2_000, false, None);
        assert!(rendered.contains("nothing to do"), "{rendered}");
        assert!(!rendered.contains("next:"), "{rendered}");
        // Nothing to quote, so no quote line either way.
        assert!(!rendered.contains("quote"), "{rendered}");
    }

    #[test]
    fn store_changes_render_nonzero_kinds_only() {
        let none = StoreChanges {
            responses: 3,
            ..Default::default()
        };
        assert_eq!(
            none.line(),
            "changed: no items — 3 responses recorded, nothing added, changed, moved, or removed"
        );
        let some = StoreChanges {
            responses: 112,
            added: 98,
            changed: 0,
            moved: 2,
            removed: 0,
            locations: 5,
        };
        assert_eq!(
            some.line(),
            "changed: +98 added, >2 moved at 5 locations since the apply started; 112 responses recorded (acq store events)"
        );
    }
}
