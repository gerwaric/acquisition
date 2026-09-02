mod dash;
mod plan_cmd;
mod store_cmd;

use std::io::{IsTerminal as _, Write as _};
use std::time::{Duration, Instant};

use acquisition_core::client::{Client, ConnectOptions};
use acquisition_core::daemon;
use acquisition_core::job::{JobInfo, JobState, Outcome};
use acquisition_core::protocol::{Request, Response};
use acquisition_core::realm::Realm;
use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use serde_json::json;

/// `--realm`: the coordinate above league (CONTEXT.md, 2026-09-02). pc by
/// default, as on the wire; a typo is refused by clap before any daemon
/// or store is touched.
fn parse_realm(s: &str) -> Result<Realm, String> {
    Realm::parse(s).ok_or_else(|| {
        format!(
            "unknown realm {s:?} (one of {})",
            Realm::ALL.map(Realm::as_str).join(", ")
        )
    })
}

/// The CLI's connect policy: lazy-spawn as asked, and replace a version- or
/// provider-mismatched daemon — the caller is the human expressing intent.
pub(crate) async fn connect(spawn: bool) -> Result<Client> {
    Client::connect(ConnectOptions::interactive(spawn)).await
}

#[derive(Parser)]
#[command(
    name = "acq",
    // `<pkg version> (<git commit>)`: the thing to check before a live run
    // is the binary, not the checkout. A `-dirty` suffix means uncommitted
    // changes were built in.
    version = acquisition_core::VERSION_WITH_BUILD,
    about = "Acquisition playground CLI (mock provider by default; ACQ_GGG=1 talks to real GGG)"
)]
struct Cli {
    /// Emit structured JSON instead of human-readable output.
    #[arg(long, global = true)]
    json: bool,
    /// Which account to act as: a username (with or without `#…`) or uuid.
    /// Defaults to `ACQ_ACCOUNT`, else the sole known/logged-in account.
    #[arg(long, global = true, env = "ACQ_ACCOUNT")]
    account: Option<String>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Log in via OAuth (mock provider, or real GGG with ACQ_GGG=1).
    Auth {
        #[command(subcommand)]
        cmd: Option<AuthCmd>,
        /// Print the login URL instead of opening a browser.
        #[arg(long)]
        no_browser: bool,
    },
    /// The account profile (account:profile).
    Profile,
    /// List characters on the logged-in account.
    Characters {
        /// pc (default), xbox, sony, or poe2.
        #[arg(long, default_value = "pc", value_parser = parse_realm)]
        realm: Realm,
    },
    /// Fetch one character with its equipment and inventory.
    Character {
        name: String,
        /// pc (default), xbox, sony, or poe2.
        #[arg(long, default_value = "pc", value_parser = parse_realm)]
        realm: Realm,
    },
    /// List the account's leagues.
    Leagues,
    /// Accounts this machine has logged into (the store's index; no daemon).
    Accounts,
    /// Tabs of a league, from the shared store (no daemon round-trip).
    Tabs {
        #[arg(long, default_value = "Standard")]
        league: String,
        /// pc (default), xbox, or sony.
        #[arg(long, default_value = "pc", value_parser = parse_realm)]
        realm: Realm,
    },
    /// Items in the shared store.
    Items {
        #[command(subcommand)]
        cmd: ItemsCmd,
    },
    /// The shared store itself (what the daemon writes; every frontend reads).
    Store {
        #[command(subcommand)]
        cmd: StoreCmd,
    },
    /// List stash tabs for a league.
    Stashes {
        #[arg(long, default_value = "Standard")]
        league: String,
        /// pc (default), xbox, or sony — the stash endpoints are PoE1 only.
        #[arg(long, default_value = "pc", value_parser = parse_realm)]
        realm: Realm,
    },
    /// Fetch one stash tab (or one substash of a map/unique tab).
    Stash {
        id: String,
        #[arg(long)]
        sub: Option<String>,
        /// Follow a map/unique tab's substashes as child jobs. Opt-in per
        /// tab: one map tab can hold hundreds.
        #[arg(long)]
        deep: bool,
        #[arg(long, default_value = "Standard")]
        league: String,
        /// pc (default), xbox, or sony — the stash endpoints are PoE1 only.
        #[arg(long, default_value = "pc", value_parser = parse_realm)]
        realm: Realm,
    },
    /// Refresh tabs: one stash-list request, then one `stash` child job per
    /// selected tab. Selection is explicit — there is no default.
    Refresh {
        /// Every tab in the league (folder children included, folders not).
        #[arg(long, conflicts_with = "tabs")]
        all: bool,
        /// Comma-separated tab ids.
        #[arg(long, value_delimiter = ',')]
        tabs: Vec<String>,
        /// Also follow map/unique substashes (per tab, as child jobs).
        #[arg(long)]
        deep: bool,
        /// Compile the stored sync policy (`acq policy`) into the explicit
        /// action set and print it — nothing is submitted or sent. A
        /// running daemon adds its read-only quote. With --json, prints
        /// the serializable plan envelope. `--plan=FILE` (or `=-` for
        /// stdin) renders a reviewed envelope instead, through the same
        /// renderer, with the quote it carries.
        #[arg(long, value_name = "FILE", num_args = 0..=1, require_equals = true,
              default_missing_value = "", conflicts_with_all = ["all", "tabs", "deep"])]
        plan: Option<String>,
        /// With --plan: one line per action and every quote note, instead
        /// of the grouped view (groups over ten entities are counted).
        #[arg(long, requires = "plan")]
        expand: bool,
        /// Execute the plan: exactly its actions, as one `apply` parent
        /// job that never expands the set. Bare `--apply` compiles the
        /// stored policy now; `--apply=FILE` (or `--apply=-` for stdin)
        /// reads a reviewed plan envelope from `refresh --plan --json`.
        /// Refused if the stored policy revision no longer matches the
        /// plan's.
        #[arg(long, value_name = "FILE", num_args = 0..=1, require_equals = true,
              default_missing_value = "", conflicts_with_all = ["all", "tabs", "deep", "plan"])]
        apply: Option<String>,
        /// Refuse the apply before anything runs if the plan authorizes
        /// more than this many requests (checked by the daemon at
        /// admission, before any child job exists).
        #[arg(long, requires = "apply")]
        max_requests: Option<u64>,
        /// Defaults to Standard. With `--apply=FILE`, the plan's own
        /// league governs; giving --league too asserts they agree.
        #[arg(long)]
        league: Option<String>,
        /// pc (default), xbox, sony, or poe2 (a poe2 policy entry can
        /// cover characters only). With `--apply=FILE`, the plan's own
        /// realm governs; giving --realm too asserts they agree.
        #[arg(long, value_parser = parse_realm)]
        realm: Option<Realm>,
    },
    /// The per-account sync policy: the declared coverage and freshness
    /// that `acq refresh --plan` compiles into requests.
    Policy {
        #[command(subcommand)]
        cmd: Option<PolicyCmd>,
    },
    /// Submit a job (kinds: sleep, fetch, whoami, profile, characters, character, leagues, stashes, stash, refresh, apply).
    Submit {
        kind: String,
        /// JSON params, e.g. '{"seconds": 5}'.
        #[arg(long, default_value = "{}")]
        params: String,
        /// Higher runs sooner. Default 0.
        #[arg(long, default_value_t = 0)]
        priority: u8,
        /// Return the job id immediately instead of blocking with progress.
        #[arg(long)]
        detach: bool,
    },
    /// Submit a burst of fetch jobs to watch the rate limiter queue them.
    Demo {
        #[arg(long, default_value_t = 8)]
        count: u32,
    },
    /// Live dashboard (TUI): rate limiter state, job queue, HTTP sends,
    /// recent errors. With --json, prints one snapshot and exits.
    Dash,
    /// List jobs the daemon knows about.
    Jobs {
        /// Stay subscribed and print job-state-changed events as they happen.
        #[arg(long)]
        watch: bool,
    },
    /// Show one job's state and ETA.
    Status { id: u64 },
    /// Fetch a finished job's payload or error.
    Result { id: u64 },
    /// Cancel a waiting or running job.
    Cancel { id: u64 },
    /// Change a waiting job's priority.
    SetPriority { id: u64, priority: u8 },
    /// Debugging only — normal use never needs manual lifecycle.
    Daemon {
        #[command(subcommand)]
        cmd: DaemonCmd,
    },
}

#[derive(Subcommand)]
enum ItemsCmd {
    /// Substring search over name, type line, and base type.
    Search {
        text: String,
        #[arg(long)]
        league: Option<String>,
        /// Restrict to one realm (pc, xbox, sony, poe2).
        #[arg(long, value_parser = parse_realm)]
        realm: Option<Realm>,
        /// Include items no longer seen at their last location.
        #[arg(long)]
        removed: bool,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// One item by id, verbatim.
    Show { id: String },
}

#[derive(Subcommand)]
enum StoreCmd {
    /// Path, size, and row counts.
    Status,
    /// Characters on record (no daemon): id, address, league, freshness,
    /// live item count. `acq characters` (the job) fetches the list anew.
    Characters {
        /// Restrict to one realm (pc, xbox, sony, poe2); every realm otherwise.
        #[arg(long, value_parser = parse_realm)]
        realm: Option<Realm>,
        /// Restrict to one league; every league otherwise.
        #[arg(long)]
        league: Option<String>,
    },
    /// Item events (added/moved/changed/removed) from recent ingests: by
    /// default one line per location with counts (text) or the event list
    /// (--json); --expand / --summary pick either form in both modes.
    Events {
        #[arg(long, default_value_t = 24.0)]
        hours: f64,
        #[arg(long, default_value_t = 200)]
        limit: usize,
        /// Every event, one per line (the JSON default).
        #[arg(long, conflicts_with = "summary")]
        expand: bool,
        /// One line per location with counts (the text default).
        #[arg(long)]
        summary: bool,
    },
    /// Bodies the store refused as malformed, kept verbatim as evidence:
    /// the list (newest first), or one body in full by its row id.
    Refused {
        /// Row id from the list (or from the failed job's error).
        id: Option<i64>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
    },
    /// Re-extract derived columns from each item's own JSON.
    Rebuild,
    /// Replay a snapshot file from the retired `acq pull` into the store (no GGG traffic).
    Import { path: std::path::PathBuf },
}

#[derive(Subcommand)]
enum PolicyCmd {
    /// Print the stored policy value and its revision (the default).
    Show,
    /// Write the policy: inline JSON, `-` to read stdin, or `@<path>` to
    /// read a file. Validated before it is stored — a typo'd field or an
    /// unknown version is refused, never half-honored.
    Set {
        value: String,
        /// Only write if the stored policy is at exactly this revision
        /// (what `acq policy show` printed when you reviewed it); refused
        /// with the current revision otherwise. Without it the write
        /// replaces whatever revision is currently stored — though a write
        /// racing in between still conflicts rather than being clobbered.
        #[arg(long)]
        if_revision: Option<i64>,
    },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Show session state (login, token expiry, keyring health).
    Status,
    /// Verify the session actually works: forces a token round-trip through
    /// the provider instead of trusting local state. Exit code 1 on failure.
    Check,
    /// Drop the session and clear the keyring entry. With --account naming
    /// another known account, clear only that account's keyring entry.
    Logout,
}

#[derive(Subcommand)]
enum DaemonCmd {
    Status,
    Stop,
    /// Clear the live-test rails' tripwire/ceiling halt (see LIVE-TESTING.md).
    /// Observe the post-violation rule before using this.
    ResetTripwire,
    /// Run the daemon in the foreground (what lazy-spawn execs).
    Run,
}

/// `--account`/`ACQ_ACCOUNT`, for every submit this process makes.
static ACCOUNT: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

/// The failure was already printed in the command's own output format (a
/// failed job's `--json` outcome); main only sets the exit code.
#[derive(Debug)]
struct AlreadyReported;

impl std::fmt::Display for AlreadyReported {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("already reported")
    }
}

impl std::error::Error for AlreadyReported {}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let json = cli.json;
    store_cmd::set_selector(cli.account.clone());
    let _ = ACCOUNT.set(cli.account.clone());
    if let Err(e) = run(cli).await {
        if e.downcast_ref::<AlreadyReported>().is_none() {
            if json {
                println!("{}", json!({ "error": format!("{e:#}") }));
            } else {
                eprintln!("Error: {e:#}");
            }
        }
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::Auth { cmd, no_browser } => match cmd {
            None => login(no_browser, cli.json).await,
            Some(AuthCmd::Status) => {
                let mut client = connect(false).await?;
                let status = client.request(&Request::AuthStatus).await?;
                print_auth(&status, cli.json)?;
                if !cli.json {
                    store_cmd::print_other_accounts(match &status {
                        Response::Auth { accounts, .. } => {
                            accounts.iter().map(|a| a.username.clone()).collect()
                        }
                        _ => Vec::new(),
                    })?;
                }
                Ok(())
            }
            Some(AuthCmd::Check) => {
                let mut client = connect(true).await?;
                let account = ACCOUNT.get().cloned().flatten();
                match client.request(&Request::AuthCheck { account }).await? {
                    Response::Error { message } => bail!("auth check failed: {message}"),
                    status => {
                        if !cli.json {
                            println!("session verified (live token round-trip succeeded)");
                        }
                        print_auth(&status, cli.json)
                    }
                }
            }
            Some(AuthCmd::Logout) => {
                let mut client = connect(false).await?;
                let account = ACCOUNT.get().cloned().flatten();
                client
                    .expect_ack(&Request::AuthLogout {
                        account: account.clone(),
                    })
                    .await?;
                if cli.json {
                    println!("{}", json!({ "logged_out": true, "account": account }));
                } else {
                    println!("logged out (session dropped, keyring cleared)");
                }
                Ok(())
            }
        },
        Cmd::Profile => {
            let mut client = connect(true).await?;
            let id = submit(&mut client, "profile".into(), json!({}), 0).await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Characters { realm } => {
            let mut client = connect(true).await?;
            let id = submit(
                &mut client,
                "characters".into(),
                json!({ "realm": realm }),
                0,
            )
            .await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Character { name, realm } => {
            let mut client = connect(true).await?;
            let id = submit(
                &mut client,
                "character".into(),
                json!({ "realm": realm, "name": name }),
                0,
            )
            .await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Leagues => {
            let mut client = connect(true).await?;
            let id = submit(&mut client, "leagues".into(), json!({}), 0).await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Accounts => store_cmd::accounts(cli.json),
        Cmd::Tabs { league, realm } => store_cmd::tabs(realm, &league, cli.json),
        Cmd::Items { cmd } => match cmd {
            ItemsCmd::Search {
                text,
                league,
                realm,
                removed,
                limit,
            } => store_cmd::search(&text, realm, league.as_deref(), removed, limit, cli.json),
            ItemsCmd::Show { id } => store_cmd::show(&id, cli.json),
        },
        Cmd::Store { cmd } => match cmd {
            StoreCmd::Status => store_cmd::status(cli.json),
            StoreCmd::Characters { realm, league } => {
                store_cmd::characters(realm, league.as_deref(), cli.json)
            }
            StoreCmd::Events {
                hours,
                limit,
                expand,
                summary,
            } => store_cmd::events(hours, limit, expand, summary, cli.json),
            StoreCmd::Refused { id, limit } => store_cmd::refused(id, limit, cli.json),
            StoreCmd::Rebuild => store_cmd::rebuild(cli.json),
            StoreCmd::Import { path } => store_cmd::import(&path, cli.json),
        },
        Cmd::Stashes { league, realm } => {
            let mut client = connect(true).await?;
            let id = submit(
                &mut client,
                "stashes".into(),
                json!({ "realm": realm, "league": league }),
                0,
            )
            .await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Stash {
            id,
            sub,
            deep,
            league,
            realm,
        } => {
            let mut client = connect(true).await?;
            let id = submit(
                &mut client,
                "stash".into(),
                json!({ "realm": realm, "league": league, "id": id, "sub": sub, "deep": deep }),
                0,
            )
            .await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Refresh {
            all,
            tabs,
            deep,
            plan,
            expand,
            apply,
            max_requests,
            league,
            realm,
        } => {
            if let Some(source) = plan {
                // Bare `--plan` compiles the stored policy; `=FILE` renders
                // a reviewed envelope.
                let source = (!source.is_empty()).then_some(source);
                return plan_cmd::refresh_plan(
                    realm.unwrap_or(Realm::DEFAULT),
                    league.as_deref().unwrap_or("Standard"),
                    source.as_deref(),
                    cli.json,
                    expand,
                )
                .await;
            }
            if let Some(source) = apply {
                // Bare `--apply` (clap's empty default_missing_value)
                // means "compile the stored policy now and run that".
                let source = (!source.is_empty()).then_some(source);
                return plan_cmd::refresh_apply(
                    realm,
                    league.as_deref(),
                    source.as_deref(),
                    max_requests,
                    cli.json,
                )
                .await;
            }
            if !all && tabs.is_empty() {
                bail!("refresh needs --all, --tabs <id,...>, --plan, or --apply");
            }
            let league = league.as_deref().unwrap_or("Standard");
            let realm = realm.unwrap_or(Realm::DEFAULT);
            let mut client = connect(true).await?;
            let id = submit(
                &mut client,
                "refresh".into(),
                json!({ "realm": realm, "league": league, "all": all, "tabs": tabs, "deep": deep }),
                0,
            )
            .await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Policy { cmd } => match cmd {
            None | Some(PolicyCmd::Show) => plan_cmd::policy_show(cli.json),
            Some(PolicyCmd::Set { value, if_revision }) => {
                plan_cmd::policy_set(&value, if_revision, cli.json)
            }
        },
        Cmd::Submit {
            kind,
            params,
            priority,
            detach,
        } => {
            let params: serde_json::Value = serde_json::from_str(&params)?;
            let mut client = connect(true).await?;
            let id = submit(&mut client, kind, params, priority).await?;
            if detach {
                if cli.json {
                    println!("{}", json!({ "job_id": id }));
                } else {
                    println!("job {id} submitted (acq status {id} / acq result {id})");
                }
                Ok(())
            } else {
                block_on_job(&mut client, id, cli.json).await
            }
        }
        Cmd::Demo { count } => {
            let mut client = connect(true).await?;
            let mut ids = Vec::new();
            for i in 0..count {
                let id = submit(
                    &mut client,
                    "fetch".into(),
                    json!({ "what": format!("demo stash tab {i}") }),
                    0,
                )
                .await?;
                ids.push(id);
            }
            println!("submitted {count} fetch jobs: {ids:?}");
            println!("(mock policy: 5 per 10s, then 30 per 300s — watch the ETAs)\n");
            watch_table_until_done(&mut client, &ids).await
        }
        Cmd::Dash => dash::run(cli.json).await,
        Cmd::Jobs { watch } => {
            let mut client = connect(false).await?;
            let jobs = list(&mut client).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&jobs)?);
            } else {
                print_table(&jobs);
            }
            if watch {
                match client.request(&Request::Subscribe).await? {
                    Response::Subscribed => {}
                    other => bail!("unexpected response: {other:?}"),
                }
                loop {
                    if let Response::Event { job } = client.recv().await? {
                        if cli.json {
                            println!("{}", serde_json::to_string(&job)?);
                        } else {
                            println!("job {:>3}  {:<8} -> {}", job.id, job.kind, job.state);
                        }
                    }
                }
            }
            Ok(())
        }
        Cmd::Status { id } => {
            let mut client = connect(false).await?;
            let job = client.status(id).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&job)?);
            } else {
                print_table(std::slice::from_ref(&job));
            }
            Ok(())
        }
        Cmd::Result { id } => {
            let mut client = connect(false).await?;
            print_result(&mut client, id, cli.json).await
        }
        Cmd::Cancel { id } => {
            let mut client = connect(false).await?;
            client.expect_ack(&Request::Cancel { id }).await?;
            if cli.json {
                println!("{}", json!({ "job_id": id, "cancel_requested": true }));
            } else {
                println!("job {id} cancel requested");
            }
            Ok(())
        }
        Cmd::SetPriority { id, priority } => {
            let mut client = connect(false).await?;
            client
                .expect_ack(&Request::SetPriority { id, priority })
                .await?;
            if cli.json {
                println!("{}", json!({ "job_id": id, "priority": priority }));
            } else {
                println!("job {id} priority -> {priority}");
            }
            Ok(())
        }
        Cmd::Daemon { cmd } => match cmd {
            DaemonCmd::Run => daemon::run().await,
            DaemonCmd::Status => {
                let mut client = match connect(false).await {
                    Ok(c) => c,
                    Err(_) => {
                        if cli.json {
                            println!("{}", json!({ "running": false }));
                        } else {
                            println!("daemon is not running");
                        }
                        return Ok(());
                    }
                };
                let status = client.request(&Request::DaemonStatus).await?;
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else if let Response::DaemonStatus {
                    pid,
                    version,
                    provider,
                    uptime_seconds,
                    connections,
                    jobs_waiting,
                    jobs_running,
                    policies_known,
                    in_flight,
                    max_in_flight,
                    rails,
                    keyring,
                } = status
                {
                    println!(
                        "daemon {version} pid {pid}, up {uptime_seconds}s, provider {provider}"
                    );
                    println!(
                        "connections: {connections}  waiting: {jobs_waiting}  running: {jobs_running}  in flight: {in_flight}/{max_in_flight}  policies learned: {policies_known}"
                    );
                    println!("socket: {}", daemon::socket_path().display());
                    println!("log:    {}", daemon::log_path().display());
                    println!(
                        "rails:  tripwire {} · sends {}{} · journal {}",
                        if rails.tripwire_enabled { "ON" } else { "off" },
                        rails.sends,
                        rails.max_sends.map_or(String::new(), |m| format!("/{m}")),
                        rails.journal.as_deref().unwrap_or("off"),
                    );
                    if let Some(cause) = &rails.halted {
                        println!("HALTED: {cause}");
                        println!(
                            "        clear with `acq daemon reset-tripwire` after the post-violation wait"
                        );
                    }
                    if keyring != "ok" {
                        println!("KEYRING: {keyring} — a rotated refresh token may be memory-only");
                    }
                    if let Some(cause) = &rails.refresh_failed {
                        println!("REFRESH DISABLED: {cause}");
                        println!("        re-login with `acq auth`");
                    }
                }
                Ok(())
            }
            DaemonCmd::ResetTripwire => {
                match connect(false).await {
                    Ok(mut client) => {
                        let resp = client.request(&Request::ResetTripwire).await?;
                        if cli.json {
                            println!("{}", serde_json::to_string_pretty(&resp)?);
                        } else {
                            println!("rails reset");
                        }
                    }
                    Err(_) => {
                        // The trip lives on disk; clear it there so the next
                        // spawned daemon is not still halted.
                        let provider = if acquisition_core::provider::ggg_mode() {
                            "ggg"
                        } else {
                            "mock"
                        };
                        let state =
                            daemon::socket_path().with_extension(format!("{provider}.rails.json"));
                        match std::fs::remove_file(&state) {
                            Ok(()) => {
                                if cli.json {
                                    println!("{}", json!({ "cleared": true, "state": state }));
                                } else {
                                    println!(
                                        "daemon is not running; cleared persisted rails state {}",
                                        state.display()
                                    );
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                                if cli.json {
                                    println!("{}", json!({ "cleared": false, "state": null }));
                                } else {
                                    println!(
                                        "daemon is not running and no rails state is persisted"
                                    );
                                }
                            }
                            Err(e) => bail!(
                                "daemon is not running; could not clear {}: {e}",
                                state.display()
                            ),
                        }
                    }
                }
                Ok(())
            }
            DaemonCmd::Stop => {
                match connect(false).await {
                    Ok(mut client) => {
                        let _ = client.request(&Request::DaemonStop).await;
                        if cli.json {
                            println!("{}", json!({ "stopped": true }));
                        } else {
                            println!("daemon stopped");
                        }
                    }
                    Err(_) => {
                        if cli.json {
                            println!("{}", json!({ "stopped": false, "running": false }));
                        } else {
                            println!("daemon is not running");
                        }
                    }
                }
                Ok(())
            }
        },
    }
}

/// The interactive login flow: ask the daemon to start OAuth, hand the URL to
/// the browser, then poll auth status until the flow resolves.
async fn login(no_browser: bool, json: bool) -> Result<()> {
    let mut client = connect(true).await?;
    let url = match client.request(&Request::AuthStart).await? {
        Response::AuthUrl { authorize_url } => authorize_url,
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    };
    if json {
        // First of two JSON lines (the final auth status is the second), so
        // a scripted login can read the URL it must visit.
        println!("{}", json!({ "authorize_url": url }));
        std::io::stdout().flush().ok();
    } else {
        println!("To log in, open:\n\n  {url}\n");
    }
    if !no_browser && open_browser(&url) && !json {
        println!("(opened in your browser)");
    }
    if !json {
        println!("waiting for login to complete...");
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(300);
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let status = client.request(&Request::AuthStatus).await?;
        let Response::Auth {
            pending,
            ref login_ok,
            ref login_error,
            ref keyring,
            ..
        } = status
        else {
            bail!("unexpected response: {status:?}");
        };
        if pending {
            if std::time::Instant::now() > deadline {
                bail!("login did not complete within 5 minutes");
            }
            continue;
        }
        // Only the flow's own terminal result counts: `logged_in` is
        // aggregate state, and another account's live session must not be
        // mistaken for this login succeeding.
        if let Some(error) = login_error {
            bail!("login failed: {error}");
        }
        let Some(user) = login_ok else {
            bail!("login did not complete (see daemon log)");
        };
        if json {
            return print_auth(&status, true);
        }
        println!("logged in as {user}");
        if keyring != "ok" {
            println!("warning: keyring {keyring}; session will not survive a daemon restart");
        }
        return Ok(());
    }
}

fn print_auth(status: &Response, json: bool) -> Result<()> {
    let Response::Auth {
        logged_in,
        pending,
        keyring,
        provider,
        accounts,
        ..
    } = status
    else {
        bail!("unexpected response: {status:?}");
    };
    if json {
        println!("{}", serde_json::to_string_pretty(status)?);
        return Ok(());
    }
    if provider == "ggg" {
        println!("provider: real GGG");
    }
    match (logged_in, pending) {
        (_, true) => println!("login in progress (waiting on the browser)"),
        (false, _) => println!("not logged in — run `acq auth`"),
        (true, _) => {
            for a in accounts {
                let token = match a.access_expires_in_seconds {
                    Some(s) if s > 0 => format!("access token valid for ~{s}s"),
                    _ => "access token expired (will refresh on next use)".into(),
                };
                let kr = if a.keyring == "ok" {
                    String::new()
                } else {
                    format!("; keyring {}", a.keyring)
                };
                println!("logged in as {}: {token}{kr}", a.username);
            }
        }
    }
    println!("keyring: {keyring}");
    Ok(())
}

fn open_browser(url: &str) -> bool {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(opener)
        .arg(url)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .is_ok()
}

async fn submit(
    client: &mut Client,
    kind: String,
    params: serde_json::Value,
    priority: u8,
) -> Result<u64> {
    let submitted_by = format!("cli:{}", std::process::id());
    let account = ACCOUNT.get().cloned().flatten();
    match client
        .request(&Request::Submit {
            kind,
            params,
            priority,
            submitted_by,
            account,
        })
        .await?
    {
        Response::Submitted { id } => Ok(id),
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    }
}

async fn list(client: &mut Client) -> Result<Vec<JobInfo>> {
    match client.request(&Request::List).await? {
        Response::Jobs { jobs } => Ok(jobs),
        other => bail!("unexpected response: {other:?}"),
    }
}

/// Default CLI mode: block with progress until the job finishes, then print
/// its result. This is where "rate limited, retrying in 4m37s..." UX lives.
pub(crate) async fn block_on_job(client: &mut Client, id: u64, json: bool) -> Result<()> {
    wait_for_job(client, id, json).await?;
    print_result(client, id, json).await
}

/// Progress is redrawn in place only on a tty; captured output (the
/// driver's `.out` files, an agent's shell) gets a plain line per change,
/// at most every this often — a 13-minute apply is not 1,500 lines.
const PROGRESS_EVERY: Duration = Duration::from_secs(10);

/// Block until the job is terminal and return its outcome. `quiet` skips
/// the progress line (JSON mode: stdout is the outcome, nothing else).
pub(crate) async fn wait_for_job(client: &mut Client, id: u64, quiet: bool) -> Result<Outcome> {
    let tty = std::io::stdout().is_terminal();
    let mut last = String::new();
    let mut last_print: Option<Instant> = None;
    loop {
        let job = client.status(id).await?;
        let terminal = job.state.is_terminal();
        if !quiet {
            let line = progress_line(client, &job).await?;
            let due = last_print.is_none_or(|t| t.elapsed() >= PROGRESS_EVERY);
            if line != last && (tty || terminal || due) {
                if tty {
                    print!("\r\x1b[2K{line}");
                    std::io::stdout().flush().ok();
                } else {
                    println!("{line}");
                }
                last = line;
                last_print = Some(Instant::now());
            }
        }
        if terminal {
            if !quiet && tty {
                println!();
            }
            return match client.request(&Request::Result { id }).await? {
                Response::Result { outcome, .. } => Ok(outcome),
                Response::Error { message } => bail!("{message}"),
                other => bail!("unexpected response: {other:?}"),
            };
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// What the children of a parent job are doing right now.
#[derive(Default)]
struct ChildTally {
    total: usize,
    done: usize,
    failed: usize,
    cancelled: usize,
    waiting: usize,
    running: usize,
    /// The soonest predicted start among waiting children, with its kind.
    next: Option<(u64, String)>,
}

async fn tally_children(client: &mut Client, parent: u64) -> Result<ChildTally> {
    let mut tally = ChildTally::default();
    for job in list(client).await? {
        if job.parent != Some(parent) {
            continue;
        }
        tally.total += 1;
        match job.state {
            JobState::Done => tally.done += 1,
            JobState::Failed => tally.failed += 1,
            JobState::Cancelled => tally.cancelled += 1,
            JobState::Running => tally.running += 1,
            JobState::Waiting => {
                tally.waiting += 1;
                if let Some(eta) = job.eta_seconds
                    && eta > 0
                    && tally.next.as_ref().is_none_or(|(e, _)| eta < *e)
                {
                    tally.next = Some((eta, job.kind.clone()));
                }
            }
        }
    }
    Ok(tally)
}

/// One line of progress: a parent reports its children (`30/112 done, 82
/// waiting, next in ~343s (limiter hold on stash)`), any other job its
/// own state and ETA.
async fn progress_line(client: &mut Client, job: &JobInfo) -> Result<String> {
    let id = job.id;
    if matches!(job.kind.as_str(), "apply" | "refresh") && job.state != JobState::Waiting {
        let t = tally_children(client, id).await?;
        if t.total > 0 {
            let mut line = format!("job {id}: {}/{} done", t.done, t.total);
            if t.failed > 0 {
                line.push_str(&format!(", {} failed", t.failed));
            }
            if t.cancelled > 0 {
                line.push_str(&format!(", {} cancelled", t.cancelled));
            }
            if t.running > 0 {
                line.push_str(&format!(", {} running", t.running));
            }
            if t.waiting > 0 {
                line.push_str(&format!(", {} waiting", t.waiting));
            }
            if let Some((eta, kind)) = t.next {
                line.push_str(&format!(", next in ~{eta}s (limiter hold on {kind})"));
            }
            return Ok(line);
        }
    }
    Ok(match (job.state, job.eta_seconds) {
        (JobState::Waiting, Some(eta)) if eta > 0 && job.retries > 0 => {
            format!("job {id}: got a 429, retry {} in ~{eta}s...", job.retries)
        }
        (JobState::Waiting, Some(eta)) if eta > 0 => {
            format!("job {id}: rate limited, starting in ~{eta}s...")
        }
        (state, _) => format!("job {id}: {state}"),
    })
}

/// The daemon's parent failure line, `k of n child jobs failed: [ids]
/// (acq result <id> for each)`, read back into its parts. The shape is the
/// daemon's (`maybe_finish_parent`); a change there must change this too,
/// and the unit test below pins the coupling.
fn parse_children_failure(error: &str) -> Option<(usize, usize, Vec<u64>)> {
    let (head, rest) = error.split_once(" child jobs failed: [")?;
    let (k, n) = head.split_once(" of ")?;
    let ids_text = rest.split_once(']')?.0;
    let ids = ids_text
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u64>().ok())
        .collect::<Option<Vec<u64>>>()?;
    Some((k.trim().parse().ok()?, n.trim().parse().ok()?, ids))
}

/// At most this many failed children are expanded inline; the rest are
/// counted and `acq jobs` named.
const FAILED_CHILDREN_SHOWN: usize = 10;

/// The text report for an `apply` parent (rule 4 of the legibility
/// ruling): success as one line with the child range; failure as the
/// count, then one line per failed child — its id, kind, target, and
/// the error that names its evidence.
pub(crate) async fn report_apply(client: &mut Client, id: u64, outcome: &Outcome) -> String {
    match outcome {
        Outcome::Success { payload } => {
            let requests = payload["requests"].as_u64().unwrap_or(0);
            let done = payload["children"]["done"].as_u64().unwrap_or(0);
            let ids: Vec<u64> = payload["child_jobs"]
                .as_array()
                .map(|a| a.iter().filter_map(serde_json::Value::as_u64).collect())
                .unwrap_or_default();
            let range = match (ids.iter().min(), ids.iter().max()) {
                (Some(a), Some(b)) if a == b => format!(" (job {a})"),
                (Some(a), Some(b)) => format!(" (jobs {a}–{b})"),
                _ => String::new(),
            };
            format!(
                "job {id} done: {requests} request{}, {done} done{range}\n",
                if requests == 1 { "" } else { "s" }
            )
        }
        Outcome::Failure { error } => {
            let Some((k, n, ids)) = parse_children_failure(error) else {
                return format!("job {id} failed: {error}\n");
            };
            let mut out = format!(
                "job {id} failed: {k} of {n} request{} failed, {} done\n",
                if n == 1 { "" } else { "s" },
                n.saturating_sub(k)
            );
            // The children as the daemon lists them this lifetime; after
            // a restart only their ids (from the message) remain.
            let listed: Vec<JobInfo> = list(client)
                .await
                .unwrap_or_default()
                .into_iter()
                .filter(|j| j.parent == Some(id) && j.state == JobState::Failed)
                .collect();
            let mut rows: Vec<(String, String, String)> = Vec::new();
            for cid in ids.iter().take(FAILED_CHILDREN_SHOWN) {
                let label = match listed.iter().find(|j| j.id == *cid) {
                    Some(j) => format!("{} {}", j.kind, j.target()),
                    None => String::new(),
                };
                let cause = match client.request(&Request::Result { id: *cid }).await {
                    Ok(Response::Result {
                        outcome: Outcome::Failure { error },
                        ..
                    }) => error,
                    _ => format!("(acq result {cid})"),
                };
                rows.push((format!("job {cid}"), label, cause));
            }
            let w1 = rows.iter().map(|r| r.0.len()).max().unwrap_or(0);
            let w2 = rows.iter().map(|r| r.1.chars().count()).max().unwrap_or(0);
            for (a, b, c) in &rows {
                out.push_str(&format!("  {a:<w1$}  {b:<w2$}  {c}\n"));
            }
            if ids.len() > FAILED_CHILDREN_SHOWN {
                out.push_str(&format!(
                    "  and {} more failed: acq jobs\n",
                    ids.len() - FAILED_CHILDREN_SHOWN
                ));
            }
            out
        }
        Outcome::Cancelled => format!("job {id} was cancelled\n"),
    }
}

async fn print_result(client: &mut Client, id: u64, json: bool) -> Result<()> {
    let outcome = match client.request(&Request::Result { id }).await? {
        Response::Result { outcome, .. } => outcome,
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    };
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
        // A failed job exits 1 in both output modes; the outcome above is
        // the report, so main adds no second message.
        return match outcome {
            Outcome::Failure { .. } => Err(AlreadyReported.into()),
            _ => Ok(()),
        };
    }
    // An apply parent's payload is a child-id list and a tally; a person
    // wants the report, not the array. (Status may be gone after a daemon
    // restart; then the generic rendering serves.)
    let kind = client.status(id).await.map(|j| j.kind).unwrap_or_default();
    if kind == "apply" {
        print!("{}", report_apply(client, id, &outcome).await);
        return match outcome {
            Outcome::Failure { .. } => Err(AlreadyReported.into()),
            _ => Ok(()),
        };
    }
    match outcome {
        Outcome::Success { payload } => {
            println!("{}", serde_json::to_string_pretty(&payload)?)
        }
        Outcome::Failure { error } => bail!("job {id} failed: {error}"),
        Outcome::Cancelled => println!("job {id} was cancelled"),
    }
    Ok(())
}

fn print_table(jobs: &[JobInfo]) {
    if jobs.is_empty() {
        println!("no jobs");
        return;
    }
    // Widths from the data: a route label or an account name must never
    // push the columns out of line or be cut.
    let rows: Vec<[String; 9]> = jobs
        .iter()
        .map(|job| {
            [
                job.id.to_string(),
                job.parent.map(|p| p.to_string()).unwrap_or_default(),
                job.kind.clone(),
                job.target(),
                if job.retries > 0 {
                    format!("{} ↻{}", job.state, job.retries)
                } else {
                    job.state.to_string()
                },
                job.priority.to_string(),
                job.account.clone().unwrap_or_else(|| "-".into()),
                job.submitted_by.clone(),
                job.eta_seconds
                    .map(|s| format!("~{s}s"))
                    .unwrap_or_default(),
            ]
        })
        .collect();
    let heads = [
        "id", "parent", "kind", "target", "state", "prio", "account", "by", "eta",
    ];
    let mut w = [0usize; 9];
    for (i, h) in heads.iter().enumerate() {
        w[i] = h.len();
    }
    for r in &rows {
        for (i, cell) in r.iter().enumerate() {
            w[i] = w[i].max(cell.chars().count());
        }
    }
    let line = |cells: [&str; 9]| {
        format!(
            "{:>w0$}  {:>w1$}  {:<w2$}  {:<w3$}  {:<w4$}  {:>w5$}  {:<w6$}  {:<w7$}  {}",
            cells[0],
            cells[1],
            cells[2],
            cells[3],
            cells[4],
            cells[5],
            cells[6],
            cells[7],
            cells[8],
            w0 = w[0],
            w1 = w[1],
            w2 = w[2],
            w3 = w[3],
            w4 = w[4],
            w5 = w[5],
            w6 = w[6],
            w7 = w[7],
        )
    };
    println!("{}", line(heads).trim_end());
    for r in &rows {
        println!(
            "{}",
            line([
                &r[0], &r[1], &r[2], &r[3], &r[4], &r[5], &r[6], &r[7], &r[8]
            ])
            .trim_end()
        );
    }
    let count = |s: JobState| jobs.iter().filter(|j| j.state == s).count();
    let mut parts = vec![format!("{} running", count(JobState::Running))];
    let waiting = count(JobState::Waiting);
    let next = jobs
        .iter()
        .filter(|j| j.state == JobState::Waiting)
        .filter_map(|j| j.eta_seconds)
        .filter(|e| *e > 0)
        .min();
    parts.push(match next {
        Some(eta) => format!("{waiting} waiting (next in ~{eta}s)"),
        None => format!("{waiting} waiting"),
    });
    parts.push(format!("{} done", count(JobState::Done)));
    let failed = count(JobState::Failed);
    if failed > 0 {
        parts.push(format!("{failed} failed"));
    }
    let cancelled = count(JobState::Cancelled);
    if cancelled > 0 {
        parts.push(format!("{cancelled} cancelled"));
    }
    println!("{} jobs: {}", jobs.len(), parts.join(", "));
}

/// Re-render the job table once a second until every listed job is terminal.
async fn watch_table_until_done(client: &mut Client, ids: &[u64]) -> Result<()> {
    let mut printed_lines = 0usize;
    loop {
        let jobs = list(client).await?;
        // Move the cursor up over the previous render and redraw in place.
        if printed_lines > 0 {
            print!("\x1b[{printed_lines}A");
        }
        let mine: Vec<JobInfo> = jobs.into_iter().filter(|j| ids.contains(&j.id)).collect();
        for _ in 0..printed_lines {
            println!("\x1b[2K");
        }
        if printed_lines > 0 {
            print!("\x1b[{printed_lines}A");
        }
        print_table(&mine);
        printed_lines = mine.len() + 1;
        if mine.iter().all(|j| j.state.is_terminal()) {
            let done = mine.iter().filter(|j| j.state == JobState::Done).count();
            println!("\nall jobs finished ({done}/{} done)", mine.len());
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_plan_conflicts_with_the_execute_selectors() {
        // `--plan` compiles the stored policy; --all/--tabs/--deep are the
        // ad-hoc execution path. Mixing them must be a parse error, not a
        // silently ignored flag.
        for bad in [
            vec!["acq", "refresh", "--plan", "--all"],
            vec!["acq", "refresh", "--plan", "--tabs", "a,b"],
            vec!["acq", "refresh", "--plan", "--deep"],
        ] {
            assert!(Cli::try_parse_from(&bad).is_err(), "{bad:?} must not parse");
        }
        assert!(Cli::try_parse_from(["acq", "refresh", "--plan"]).is_ok());
        assert!(Cli::try_parse_from(["acq", "refresh", "--plan", "--league", "Hardcore"]).is_ok());
        assert!(Cli::try_parse_from(["acq", "policy", "set", "{}", "--if-revision", "4"]).is_ok());
    }

    #[test]
    fn refresh_apply_conflicts_with_every_other_mode_and_owns_max_requests() {
        // `--apply` executes a plan; the ad-hoc selectors and `--plan`
        // are different modes, and `--max-requests` is meaningless
        // without an apply to budget.
        for bad in [
            vec!["acq", "refresh", "--apply", "--all"],
            vec!["acq", "refresh", "--apply", "--tabs", "a,b"],
            vec!["acq", "refresh", "--apply", "--deep"],
            vec!["acq", "refresh", "--apply", "--plan"],
            vec!["acq", "refresh", "--max-requests", "5"],
        ] {
            assert!(Cli::try_parse_from(&bad).is_err(), "{bad:?} must not parse");
        }
        assert!(Cli::try_parse_from(["acq", "refresh", "--apply"]).is_ok());
        assert!(Cli::try_parse_from(["acq", "refresh", "--apply=plan.json"]).is_ok());
        assert!(Cli::try_parse_from(["acq", "refresh", "--apply=-"]).is_ok());
        assert!(Cli::try_parse_from(["acq", "refresh", "--apply", "--max-requests", "5"]).is_ok());
        // The optional value takes `=` only, so a following flag can never
        // be swallowed as the FILE.
        let cli = Cli::try_parse_from(["acq", "refresh", "--apply", "--json"]).unwrap();
        match cli.cmd {
            Cmd::Refresh { apply, .. } => assert_eq!(apply.as_deref(), Some("")),
            other => panic!("parsed into the wrong command: {}", other_name(&other)),
        }
    }

    fn other_name(_: &Cmd) -> &'static str {
        "not refresh"
    }

    #[test]
    fn plan_takes_an_optional_file_and_expand_needs_it() {
        assert!(Cli::try_parse_from(["acq", "refresh", "--plan=plan.json"]).is_ok());
        assert!(Cli::try_parse_from(["acq", "refresh", "--plan", "--expand"]).is_ok());
        assert!(Cli::try_parse_from(["acq", "refresh", "--expand"]).is_err());
        let cli = Cli::try_parse_from(["acq", "refresh", "--plan", "--json"]).unwrap();
        match cli.cmd {
            Cmd::Refresh { plan, .. } => assert_eq!(plan.as_deref(), Some("")),
            other => panic!("parsed into the wrong command: {}", other_name(&other)),
        }
    }

    #[test]
    fn the_daemons_parent_failure_line_is_read_back_whole() {
        // The exact shape `maybe_finish_parent` emits (daemon.rs), ids
        // sorted at the source.
        let (k, n, ids) = parse_children_failure(
            "4 of 5 child jobs failed: [222, 223, 224, 226] (acq result <id> for each)",
        )
        .unwrap();
        assert_eq!((k, n), (4, 5));
        assert_eq!(ids, vec![222, 223, 224, 226]);
        assert!(parse_children_failure("2 of 3 child jobs were cancelled").is_none());
        assert!(parse_children_failure("GET /stash/x returned 404").is_none());
    }
}
