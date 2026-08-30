mod client;
mod dash;
mod store_cmd;

use std::io::Write as _;
use std::time::Duration;

use acquisition_core::daemon;
use acquisition_core::job::{JobInfo, JobState, Outcome};
use acquisition_core::protocol::{Request, Response};
use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use serde_json::json;

use client::Client;

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
    Characters,
    /// Fetch one character with its equipment and inventory.
    Character { name: String },
    /// List the account's leagues.
    Leagues,
    /// Accounts this machine has logged into (the store's index; no daemon).
    Accounts,
    /// Tabs of a league, from the shared store (no daemon round-trip).
    Tabs {
        #[arg(long, default_value = "Standard")]
        league: String,
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
        #[arg(long, default_value = "Standard")]
        league: String,
    },
    /// Submit a job (kinds: sleep, fetch, whoami, profile, characters, character, leagues, stashes, stash, refresh).
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
    /// Item events (added/moved/changed/removed) from recent ingests.
    Events {
        #[arg(long, default_value_t = 24.0)]
        hours: f64,
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },
    /// Re-extract derived columns from each item's own JSON.
    Rebuild,
    /// Replay a snapshot file from the retired `acq pull` into the store (no GGG traffic).
    Import { path: std::path::PathBuf },
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    store_cmd::set_selector(cli.account.clone());
    let _ = ACCOUNT.set(cli.account.clone());
    match cli.cmd {
        Cmd::Auth { cmd, no_browser } => match cmd {
            None => login(no_browser, cli.json).await,
            Some(AuthCmd::Status) => {
                let mut client = Client::connect(false).await?;
                let status = client.request(&Request::AuthStatus).await?;
                print_auth(&status, cli.json)?;
                if !cli.json {
                    store_cmd::print_other_accounts(match &status {
                        Response::Auth { username, .. } => username.as_deref(),
                        _ => None,
                    })?;
                }
                Ok(())
            }
            Some(AuthCmd::Check) => {
                let mut client = Client::connect(true).await?;
                match client.request(&Request::AuthCheck).await? {
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
                let mut client = Client::connect(false).await?;
                let account = ACCOUNT.get().cloned().flatten();
                client.expect_ack(&Request::AuthLogout { account }).await?;
                println!("logged out (session dropped, keyring cleared)");
                Ok(())
            }
        },
        Cmd::Profile => {
            let mut client = Client::connect(true).await?;
            let id = submit(&mut client, "profile".into(), json!({}), 0).await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Characters => {
            let mut client = Client::connect(true).await?;
            let id = submit(&mut client, "characters".into(), json!({}), 0).await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Character { name } => {
            let mut client = Client::connect(true).await?;
            let id = submit(&mut client, "character".into(), json!({ "name": name }), 0).await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Leagues => {
            let mut client = Client::connect(true).await?;
            let id = submit(&mut client, "leagues".into(), json!({}), 0).await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Accounts => store_cmd::accounts(cli.json),
        Cmd::Tabs { league } => store_cmd::tabs(&league, cli.json),
        Cmd::Items { cmd } => match cmd {
            ItemsCmd::Search {
                text,
                league,
                removed,
                limit,
            } => store_cmd::search(&text, league.as_deref(), removed, limit, cli.json),
            ItemsCmd::Show { id } => store_cmd::show(&id, cli.json),
        },
        Cmd::Store { cmd } => match cmd {
            StoreCmd::Status => store_cmd::status(cli.json),
            StoreCmd::Events { hours, limit } => store_cmd::events(hours, limit, cli.json),
            StoreCmd::Rebuild => store_cmd::rebuild(cli.json),
            StoreCmd::Import { path } => store_cmd::import(&path, cli.json),
        },
        Cmd::Stashes { league } => {
            let mut client = Client::connect(true).await?;
            let id = submit(
                &mut client,
                "stashes".into(),
                json!({ "league": league }),
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
        } => {
            let mut client = Client::connect(true).await?;
            let id = submit(
                &mut client,
                "stash".into(),
                json!({ "league": league, "id": id, "sub": sub, "deep": deep }),
                0,
            )
            .await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Refresh {
            all,
            tabs,
            deep,
            league,
        } => {
            if !all && tabs.is_empty() {
                bail!("refresh needs --all or --tabs <id,...>");
            }
            let mut client = Client::connect(true).await?;
            let id = submit(
                &mut client,
                "refresh".into(),
                json!({ "league": league, "all": all, "tabs": tabs, "deep": deep }),
                0,
            )
            .await?;
            block_on_job(&mut client, id, cli.json).await
        }
        Cmd::Submit {
            kind,
            params,
            priority,
            detach,
        } => {
            let params: serde_json::Value = serde_json::from_str(&params)?;
            let mut client = Client::connect(true).await?;
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
            let mut client = Client::connect(true).await?;
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
            let mut client = Client::connect(false).await?;
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
            let mut client = Client::connect(false).await?;
            let job = client.status(id).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&job)?);
            } else {
                print_table(std::slice::from_ref(&job));
            }
            Ok(())
        }
        Cmd::Result { id } => {
            let mut client = Client::connect(false).await?;
            print_result(&mut client, id, cli.json).await
        }
        Cmd::Cancel { id } => {
            let mut client = Client::connect(false).await?;
            client.expect_ack(&Request::Cancel { id }).await?;
            println!("job {id} cancel requested");
            Ok(())
        }
        Cmd::SetPriority { id, priority } => {
            let mut client = Client::connect(false).await?;
            client
                .expect_ack(&Request::SetPriority { id, priority })
                .await?;
            println!("job {id} priority -> {priority}");
            Ok(())
        }
        Cmd::Daemon { cmd } => match cmd {
            DaemonCmd::Run => daemon::run().await,
            DaemonCmd::Status => {
                let mut client = match Client::connect(false).await {
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
                match Client::connect(false).await {
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
                            Ok(()) => println!(
                                "daemon is not running; cleared persisted rails state {}",
                                state.display()
                            ),
                            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                                println!("daemon is not running and no rails state is persisted")
                            }
                            Err(e) => println!(
                                "daemon is not running; could not clear {}: {e}",
                                state.display()
                            ),
                        }
                    }
                }
                Ok(())
            }
            DaemonCmd::Stop => {
                match Client::connect(false).await {
                    Ok(mut client) => {
                        let _ = client.request(&Request::DaemonStop).await;
                        println!("daemon stopped");
                    }
                    Err(_) => println!("daemon is not running"),
                }
                Ok(())
            }
        },
    }
}

/// The interactive login flow: ask the daemon to start OAuth, hand the URL to
/// the browser, then poll auth status until the flow resolves.
async fn login(no_browser: bool, json: bool) -> Result<()> {
    let mut client = Client::connect(true).await?;
    let url = match client.request(&Request::AuthStart).await? {
        Response::AuthUrl { authorize_url } => authorize_url,
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    };
    if !json {
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
            logged_in,
            pending,
            ref username,
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
        if !logged_in {
            bail!("login did not complete (see daemon log)");
        }
        if json {
            return print_auth(&status, true);
        }
        println!(
            "logged in as {}",
            username.as_deref().unwrap_or("<unknown>")
        );
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
        username,
        access_expires_in_seconds,
        keyring,
        provider,
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
            println!(
                "logged in as {}",
                username.as_deref().unwrap_or("<unknown>")
            );
            match access_expires_in_seconds {
                Some(s) if *s > 0 => println!("access token valid for ~{s}s"),
                _ => println!("access token expired (will refresh on next use)"),
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
async fn block_on_job(client: &mut Client, id: u64, json: bool) -> Result<()> {
    let mut last = String::new();
    loop {
        let job = client.status(id).await?;
        if !json {
            let line = match (job.state, job.eta_seconds) {
                (JobState::Waiting, Some(eta)) if eta > 0 && job.retries > 0 => {
                    format!("job {id}: got a 429, retry {} in ~{eta}s...", job.retries)
                }
                (JobState::Waiting, Some(eta)) if eta > 0 => {
                    format!("job {id}: rate limited, starting in ~{eta}s...")
                }
                (state, _) => format!("job {id}: {state}"),
            };
            if line != last {
                print!("\r\x1b[2K{line}");
                std::io::stdout().flush().ok();
                last = line;
            }
        }
        if job.state.is_terminal() {
            if !json {
                println!();
            }
            return print_result(client, id, json).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

async fn print_result(client: &mut Client, id: u64, json: bool) -> Result<()> {
    match client.request(&Request::Result { id }).await? {
        Response::Result { outcome, .. } => {
            if json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
                return Ok(());
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
        Response::Error { message } => bail!("{message}"),
        other => bail!("unexpected response: {other:?}"),
    }
}

fn print_table(jobs: &[JobInfo]) {
    if jobs.is_empty() {
        println!("no jobs");
        return;
    }
    println!(
        "{:>4}  {:>6}  {:<10}  {:<22}  {:<10}  {:>4}  {:<16}  {:<12}  eta",
        "id", "parent", "kind", "target", "state", "prio", "account", "by"
    );
    for job in jobs {
        let eta = job
            .eta_seconds
            .map(|s| format!("~{s}s"))
            .unwrap_or_default();
        println!(
            "{:>4}  {:>6}  {:<10}  {:<22}  {:<10}  {:>4}  {:<16}  {:<12}  {}",
            job.id,
            job.parent.map(|p| p.to_string()).unwrap_or_default(),
            job.kind,
            job.target(),
            if job.retries > 0 {
                format!("{} ↻{}", job.state, job.retries)
            } else {
                job.state.to_string()
            },
            job.priority,
            job.account.as_deref().unwrap_or("-"),
            job.submitted_by,
            eta
        );
    }
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
