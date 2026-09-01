//! The intent layer's CLI surface: `acq policy` reads and writes the
//! per-account sync-policy annotation (through the store crate, under
//! compare-and-swap), and `acq refresh --plan` compiles it into a
//! [`RefreshPlan`] — offline, spending nothing (tracer step 6). A running
//! daemon enriches the plan with its read-only quote; the command never
//! spawns one just to decorate output the planner computed with the
//! daemon down.

use std::path::PathBuf;

use acquisition_core::client::{Client, ConnectOptions};
use acquisition_core::protocol::{Quote, QuoteJob, QuoteScope, Request, Response};
use acquisition_plan::{
    FetchReason, ListingReason, PlanError, RefreshAction, RefreshPlan, SkipReason, SyncPolicy,
    plan_refresh,
};
use acquisition_store::{AccountEntry, Annotations, SYNC_POLICY_KIND, Store, account_path};
use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::store_cmd;

/// The sync policy's annotation address: per account, so scope `"account"`
/// with an empty key (`snapshot.rs` documents the convention).
const SCOPE: &str = "account";
const KEY: &str = "";

/// A shape hint for humans; the planner's strict parse is the authority.
const POLICY_EXAMPLE: &str =
    r#"{"version":1,"leagues":{"Standard":{"tabs":"all","max_age_seconds":3600}}}"#;

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
    let row = annotations.get(SCOPE, KEY, SYNC_POLICY_KIND)?;
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

/// Validate, then write under compare-and-swap. Validation first: the
/// policy is intent, and the planner refuses a typo'd or newer-versioned
/// value on every parse — storing one anyway would just move that error to
/// plan time, with the typo now on disk.
///
/// `if_revision` is the CAS at the human boundary: given, the write lands
/// only if the stored revision is exactly that — what the caller reviewed
/// (`acq policy show` prints it) is what they replace, and another
/// frontend's write in between is a structured conflict. Omitted, the
/// write blindly replaces the revision read here, just before the put —
/// the right default for one human at one keyboard — but it is still a
/// CAS underneath: a write landing between the read and the put is a
/// structured conflict to retry, never a silent clobber in either
/// direction.
fn write_policy(
    annotations: &mut Annotations,
    value: &Value,
    if_revision: Option<i64>,
) -> Result<acquisition_store::AnnotationRow> {
    SyncPolicy::from_value(value)?;
    let expected = match if_revision {
        Some(revision) => Some(revision),
        None => annotations
            .get(SCOPE, KEY, SYNC_POLICY_KIND)?
            .map(|row| row.revision),
    };
    Ok(annotations.put(SCOPE, KEY, SYNC_POLICY_KIND, value, expected)?)
}

/// `acq refresh --plan`: compile the stored sync policy into the explicit
/// action set and print it, human or JSON. Nothing is submitted and
/// nothing is sent; the JSON form is the serialized plan envelope itself
/// (self-validating on parse), so it can be reviewed, stored, or handed
/// to `--apply`.
pub async fn refresh_plan(league: &str, json: bool) -> Result<()> {
    let (dir, entry, annotations) = open_intent()?;
    let store = Store::open(&account_path(&dir, &entry.username))?;
    let snapshot = store.stash_snapshot(league, &annotations)?;
    let provider = store_cmd::provider();
    let now = acquisition_store::now();
    let plan = match plan_refresh(provider, &snapshot, now) {
        Err(PlanError::NoSyncPolicy) => bail!(
            "no sync policy is set for {} — declare one first, e.g. \
             `acq policy set '{POLICY_EXAMPLE}'`",
            entry.username
        ),
        other => other?,
    };
    let (plan, quote_note) = try_quote(plan).await;
    if json {
        // Stdout is the plan envelope, nothing else, so it can be piped;
        // why the quote is absent is diagnostics, not part of the plan.
        if let Some(note) = &quote_note {
            eprintln!("note: {note}");
        }
        println!("{}", serde_json::to_string_pretty(&plan)?);
        return Ok(());
    }
    print_plan(&plan, quote_note.as_deref(), now);
    Ok(())
}

/// `acq refresh --apply`: execute a plan — exactly its actions, as one
/// `apply` parent job the daemon admits or refuses whole (tracer step 7).
/// With no source, the stored policy is compiled right now (the normal
/// one-keyboard loop); a source is a reviewed plan envelope from
/// `refresh --plan --json`, re-validated by the planner's own parse. The
/// staleness gate runs before any daemon contact, and an empty plan never
/// contacts one — there is nothing to spend.
pub async fn refresh_apply(
    league_flag: Option<&str>,
    plan_source: Option<&str>,
    max_requests: Option<u64>,
    json: bool,
) -> Result<()> {
    let (dir, entry, annotations) = open_intent()?;
    let provider = store_cmd::provider();
    let plan = match plan_source {
        Some(source) => {
            let text = match source {
                "-" => std::io::read_to_string(std::io::stdin())
                    .context("reading the plan from stdin")?,
                path => std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?,
            };
            let value: Value = serde_json::from_str(&text).context(
                "the plan must be JSON — the envelope `acq refresh --plan --json` prints",
            )?;
            RefreshPlan::from_value(&value)?
        }
        None => {
            let league = league_flag.unwrap_or("Standard");
            let store = Store::open(&account_path(&dir, &entry.username))?;
            let snapshot = store.stash_snapshot(league, &annotations)?;
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
    check_plan_applies(&plan, provider, &entry, league_flag, &annotations)?;
    if plan.actions.is_empty() {
        // A strict subset of zero actions is satisfied by doing nothing;
        // no daemon is contacted for it.
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "applied": false,
                    "requests": 0,
                    "note": "nothing to do: everything the policy covers is fresh",
                })
            );
        } else {
            println!("nothing to do: everything the policy covers is fresh");
        }
        return Ok(());
    }
    let jobs: Vec<Value> = plan
        .actions
        .iter()
        .map(|action| {
            let (kind, params) = action.job();
            serde_json::json!({ "kind": kind, "params": params })
        })
        .collect();
    let mut params = serde_json::json!({ "jobs": jobs });
    if let Some(max) = max_requests {
        params["max_requests"] = serde_json::json!(max);
    }
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
        println!(
            "applying plan: {} request(s) as job {id}",
            plan.logical_requests
        );
    }
    crate::block_on_job(&mut client, id, json).await
}

/// The step-7 staleness gate (CONTEXT.md, decided 2026-09-01): a plan is
/// spent only while the intent it derives from still stands, and only on
/// the identity it names. The daemon is intent-blind, so this comparison
/// can only happen here, against a fresh read of the policy row. A
/// concurrent policy write between this read and the submit is the
/// accepted human-boundary race (the same register as `policy set`
/// without `--if-revision`); fact drift deliberately does not refuse —
/// the authorization is the bounded action set, and the next plan
/// reconciles.
fn check_plan_applies(
    plan: &RefreshPlan,
    provider: &str,
    entry: &AccountEntry,
    league_flag: Option<&str>,
    annotations: &Annotations,
) -> Result<()> {
    if plan.provider != provider {
        bail!(
            "the plan is for provider {:?}, but this command runs against {provider:?}",
            plan.provider
        );
    }
    if entry.uuid.as_deref() != Some(plan.account_uuid.as_str()) {
        bail!(
            "the plan is for account uuid {}, but {} is {} — replan as the right account",
            plan.account_uuid,
            entry.username,
            entry.uuid.as_deref().unwrap_or("unmapped")
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
    match annotations.get(SCOPE, KEY, SYNC_POLICY_KIND)? {
        None => bail!(
            "the sync policy is gone (the plan cites revision {}); declare one and replan \
             with `acq refresh --plan`",
            plan.basis.policy_revision
        ),
        Some(row) if row.revision != plan.basis.policy_revision => bail!(
            "the sync policy moved: the plan cites revision {}, but revision {} is stored — \
             review and replan with `acq refresh --plan`",
            plan.basis.policy_revision,
            row.revision
        ),
        Some(_) => Ok(()),
    }
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
            .map_err(|e| format!("{e:#}"))?;
        let request = Request::Quote {
            jobs,
            account: Some(account),
        };
        match client.request(&request).await {
            Ok(Response::Quote { quote }) => Ok(quote),
            Ok(Response::Error { message }) => Err(message),
            Ok(other) => Err(format!("unexpected response {other:?}")),
            Err(e) => Err(format!("{e:#}")),
        }
    };
    match tokio::time::timeout(limit, attempt).await {
        Err(_) => (
            plan,
            Some(format!(
                "no quote: the daemon did not answer within {limit:?} — plan compiled offline"
            )),
        ),
        Ok(Err(why)) => (
            plan,
            Some(format!("no quote: {why} — plan compiled offline")),
        ),
        Ok(Ok(quote)) => match plan.clone().with_quote(quote) {
            Ok(enriched) => (enriched, None),
            // The daemon answered about something other than this plan —
            // a planner/daemon disagreement worth seeing, not hiding.
            Err(e) => (plan, Some(format!("daemon quote rejected: {e}"))),
        },
    }
}

fn print_plan(plan: &RefreshPlan, quote_note: Option<&str>, now: i64) {
    let account = plan.account_name.as_deref().unwrap_or(&plan.account_uuid);
    println!(
        "refresh plan: {} on {} as {} (policy revision {}, facts as of {})",
        plan.league,
        plan.provider,
        account,
        plan.basis.policy_revision,
        store_cmd::ago(now, Some(plan.basis.snapshot_taken_at)),
    );
    match &plan.basis.listing {
        Some(listing) => println!(
            "basis: listing response {} (fetched {})",
            listing.response_id,
            store_cmd::ago(now, Some(listing.fetched_at))
        ),
        None => println!("basis: league never listed — the listing itself is the plan"),
    }
    println!();
    if plan.actions.is_empty() {
        println!("nothing to do: everything the policy covers is fresh");
    } else {
        println!(
            "actions ({} request{}, {}..{} wire sends):",
            plan.logical_requests,
            if plan.logical_requests == 1 { "" } else { "s" },
            plan.wire_sends.min,
            plan.wire_sends.max,
        );
        for action in &plan.actions {
            println!("  {}", describe_action(action));
        }
    }
    for prerequisite in &plan.wire_sends.prerequisites {
        println!("  + {prerequisite}");
    }
    if !plan.skipped.is_empty() {
        println!("{}", summarize_skipped(plan));
    }
    if !plan.unknown_tabs.is_empty() {
        println!(
            "policy names {} id(s) the facts lack (reported, never fetched): {}",
            plan.unknown_tabs.len(),
            plan.unknown_tabs.join(", ")
        );
    }
    println!();
    match (&plan.quote, quote_note) {
        (Some(quote), _) => print_quote(quote, now),
        // Notes are self-describing ("no quote: …", "daemon quote
        // rejected: …"), so no prefix here.
        (None, Some(note)) => println!("{note}"),
        (None, None) => {}
    }
}

fn describe_action(action: &RefreshAction) -> String {
    match action {
        RefreshAction::ListStashes { league, reason } => {
            let why = match reason {
                ListingReason::NeverListed => "never listed".into(),
                ListingReason::Stale { age_seconds } => stale(*age_seconds),
            };
            format!("list stashes {league:<30} {why}")
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
    }
}

/// One line of counts by reason — a real league skips hundreds of fresh
/// tabs, and the full per-tab detail is in the JSON envelope.
fn summarize_skipped(plan: &RefreshPlan) -> String {
    let (mut fresh, mut folders, mut awaiting, mut orphaned) = (0usize, 0usize, 0usize, 0usize);
    for tab in &plan.skipped {
        match tab.reason {
            SkipReason::Fresh { .. } => fresh += 1,
            SkipReason::Folder => folders += 1,
            SkipReason::AwaitingListing => awaiting += 1,
            SkipReason::OrphanedParent { .. } => orphaned += 1,
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
    format!(
        "skipped {} covered tab(s): {} (per-tab reasons in --json)",
        plan.skipped.len(),
        parts.join(", ")
    )
}

fn print_quote(quote: &Quote, now: i64) {
    println!(
        "quote (daemon's read-only projection, observed {}; an estimate, not a promise):",
        store_cmd::ago(now, Some(quote.observed_at))
    );
    if let Some(cause) = &quote.halted {
        // Report the cause only: a tripwire halt has a post-violation wait
        // before `reset-tripwire` is appropriate, and a ceiling halt cannot
        // be reset mid-lifetime at all — LIVE-TESTING.md governs clearing.
        println!(
            "  HALTED: {cause} — nothing sends while the halt stands, so every \
             estimate below is a floor (clearing it is governed by LIVE-TESTING.md)"
        );
    }
    for scope in &quote.scopes {
        print_scope(scope);
    }
    for line in &quote.not_covered {
        println!("  not covered: {line}");
    }
}

fn print_scope(scope: &QuoteScope) {
    let eta = match scope.eta_seconds {
        Some(0) => "ready now".into(),
        Some(s) => format!("last could dispatch in ~{s}s"),
        None => "no ETA until the policy is learned".into(),
    };
    println!(
        "  {}: {} request(s), {} queued ahead — {eta}",
        scope.key, scope.requests, scope.queued_ahead
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
            println!(
                "      {} {}/{} hits per {}s{observed}{restricted}",
                rule.name, window.hits, window.max_hits, window.period_secs
            );
        }
    }
    for note in &scope.notes {
        println!("      note: {note}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acquisition_store::{AnnotationRow, StashSnapshot};
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
        assert!(a.get(SCOPE, KEY, SYNC_POLICY_KIND).unwrap().is_none());
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
        let held = a.get(SCOPE, KEY, SYNC_POLICY_KIND).unwrap().unwrap();
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
        let snapshot = StashSnapshot {
            account_uuid: "u-cli".into(),
            account_name: Some("Alice#1234".into()),
            league: "Standard".into(),
            taken_at: 1_000,
            listing: None,
            tabs: Vec::new(),
            policy: Some(AnnotationRow {
                scope: SCOPE.into(),
                key: KEY.into(),
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
        let err = check_plan_applies(&plan, "mock", &entry, None, &a).unwrap_err();
        assert!(err.to_string().contains("gone"), "{err}");
        // Intent standing at the plan's revision: applies — and an
        // explicit --league that agrees is fine.
        write_policy(&mut a, &example(), None).unwrap();
        check_plan_applies(&plan, "mock", &entry, None, &a).unwrap();
        check_plan_applies(&plan, "mock", &entry, Some("Standard"), &a).unwrap();
        let err = check_plan_applies(&plan, "mock", &entry, Some("Hardcore"), &a).unwrap_err();
        assert!(err.to_string().contains("Standard"), "{err}");
        // Intent moved since the plan (the step-7 ruling): refused with
        // both revisions named, remedy = replan.
        write_policy(&mut a, &example(), None).unwrap();
        let err = check_plan_applies(&plan, "mock", &entry, None, &a).unwrap_err();
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
        let err = check_plan_applies(&plan, "mock", &other, None, &a).unwrap_err();
        assert!(err.to_string().contains("u-cli"), "{err}");
        let err = check_plan_applies(&plan, "ggg", &entry, None, &a).unwrap_err();
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
}
