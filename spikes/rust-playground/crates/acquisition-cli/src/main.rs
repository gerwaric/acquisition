mod dash;
mod plan_cmd;
mod price_cmd;
#[cfg(test)]
mod real_scale_fixture;
mod reference_cmd;
mod shop_cmd;
mod store_cmd;

use std::io::{IsTerminal as _, Write as _};
use std::time::{Duration, Instant};

use acquisition_client::client::{
    Client, ConnectOptions, DaemonError, Observed, Signal, Subscription,
};
use acquisition_protocol::job::{JobInfo, JobState, Outcome};
use acquisition_protocol::protocol::{Request, Response};
use acquisition_protocol::provider::GGG;
use acquisition_protocol::realm::Realm;
use acquisition_store::world::World;
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

/// A use verb's connect (C10): lazy-spawn as asked, and replace a daemon
/// of another contract, artifact or provider (C84) — the caller is the
/// human expressing intent.
pub(crate) async fn connect(spawn: bool) -> Result<Client> {
    Ok(Client::connect(ConnectOptions::interactive(spawn)).await?)
}

/// The running daemon, for a verb that observes it or acts on it (C10):
/// never spawns or replaces. Absence and a mismatch are errors that say
/// which; `daemon status` renders the same observation as states.
pub(crate) async fn attach() -> Result<Client> {
    match Client::observe().await? {
        Observed::Compatible(client) => Ok(client),
        Observed::Absent => bail!("daemon is not running (it spawns on demand for job commands)"),
        Observed::Incompatible(found) => bail!("{found}; {}", mismatch_remedy(&found)),
    }
}

/// What a human does about a daemon this client will not use: a
/// mismatch of contract, artifact or provider is replaced by a job
/// command; a daemon on another world (C83) is never replaced — stop it,
/// or point this shell at its world; one on the legacy socket, from
/// before the derived rendezvous, is stopped once and never comes back.
pub(crate) fn mismatch_remedy(found: &acquisition_client::client::DaemonId) -> &'static str {
    if found.on_legacy_socket() {
        "`acq daemon stop` stops it — once: nothing binds the legacy socket any more, and this world's socket is the rendezvous from then on (C83)"
    } else if found.world_matches() {
        "`acq daemon stop` stops it, a job command (`acq profile`, `acq refresh --apply`) replaces it"
    } else {
        "a job command from this shell refuses it too (C83: another world is never replaced) — `acq daemon stop` stops it, or point ACQ_STORE_DIR at its world"
    }
}

/// The log the running daemon opened, as it reports it (`daemon_status`,
/// C83) — never recomputed from this shell's environment, which may
/// differ from the daemon's (review 2026-09-11).
pub(crate) async fn daemon_log_path(client: &mut Client) -> Option<String> {
    match client.request(&Request::DaemonStatus).await {
        Ok(Response::DaemonStatus { log, .. }) => log,
        _ => None,
    }
}

#[derive(Parser)]
#[command(
    name = "acq",
    // `<pkg version> (contract <revision>)`: the contract half of the
    // handshake identity (C10, C84); `acq version --json` is the
    // structured form, with the sibling `acqd` beside it.
    version = acquisition_protocol::VERSION_WITH_CONTRACT,
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
    /// Log in via OAuth (mock provider, or real GGG with ACQ_GGG=1). The
    /// login completes only once its own profile job lands the account
    /// uuid (C50); the mock's page accepts any username, so a second
    /// account is one login apart.
    Auth {
        #[command(subcommand)]
        cmd: Option<AuthCmd>,
        /// Print the login URL instead of opening a browser.
        #[arg(long)]
        no_browser: bool,
    },
    /// The account profile (account:profile).
    Profile,
    /// List characters on the logged-in account. A route's first use
    /// queues a visible `probe` job — one HEAD that learns the policy and
    /// the account's current counters before anything real is sent (C20).
    Characters {
        /// pc (default), xbox, sony, or poe2. pc is omitted on the wire
        /// (C58); another realm is its own route segment, with its own probe.
        #[arg(long, default_value = "pc", value_parser = parse_realm)]
        realm: Realm,
    },
    /// Fetch one character with its equipment and inventory.
    Character {
        name: String,
        /// pc (default), xbox, sony, or poe2; pc is omitted on the wire (C58).
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
        /// pc (default), xbox, or sony — stashes are PoE1 only (C59).
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
    /// List stash tabs for a league: a second rate-limit policy, paced in
    /// parallel with the character routes.
    Stashes {
        #[arg(long, default_value = "Standard")]
        league: String,
        /// pc (default), xbox, or sony — the stash endpoints are PoE1 only;
        /// poe2 is refused at admission (C59).
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
        /// pc (default), xbox, or sony — the stash endpoints are PoE1 only;
        /// poe2 is refused at admission (C59).
        #[arg(long, default_value = "pc", value_parser = parse_realm)]
        realm: Realm,
    },
    /// Refresh tabs: one stash-list request, then one `stash` child job per
    /// selected tab. Selection is explicit — there is no default. (The
    /// ad-hoc `--tabs`/`--all` kind and the plan path are two doors to one
    /// task; C76 rules the direction.)
    #[command(after_long_help = "\
Reading a plan as an agent: the text is a function of the envelope (C53), so count \
with `jq` over `acq refresh --plan --json` rather than parsing prose —
  jq '.logical_requests'
  jq '[.actions[] | .action] | group_by(.) | map({(.[0]): length}) | add'          # requests by kind
  jq '[.actions[] | select(.action == \"fetch_substash\") | .parent] | group_by(.) | map({(.[0]): length}) | add'   # substashes per parent
  jq '[.skipped_tabs[], .skipped_characters[] | .reason.kind] | group_by(.) | map({(.[0]): length}) | add'   # skips by reason
`acq refresh --apply --json` adds `store_changes` beside the outcome; `acq store events --summary --json` \
is the per-location summary.")]
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
    /// Submit any job kind by hand.
    Submit {
        /// A network kind — profile, characters, character, leagues,
        /// stashes, stash, refresh, apply — or a mock-only one: sleep,
        /// fetch, whoami (refused in real mode).
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
    /// Submit a burst of fetch jobs against the mock's 5-per-10 s policy
    /// and watch the rate limiter queue them (the ETAs are the limiter's
    /// prediction, corrected by headers).
    Demo {
        #[arg(long, default_value_t = 8)]
        count: u32,
    },
    /// Live dashboard (TUI): rate limiter state (enter expands a policy:
    /// bucket state, the observed X-Rate-Limit headers, per-endpoint
    /// sends), job queue, HTTP sends, recent errors, a rails halt in red.
    /// With --json, prints one snapshot and exits.
    Dash,
    /// What this build is (C84): the package version, the shared-contract
    /// revision the daemon handshake compares — a digest over the
    /// protocol and store crates' manifests and `src` trees, the root
    /// manifest and the lock, never a git commit — and the sibling
    /// `acqd` a job command would start, as
    /// found on disk (path, length, modification time; no hash — the run
    /// record hashes it). This reports the candidate; `acq daemon status`
    /// reports the daemon running. `--json`: {"version", "contract",
    /// "provider", "acqd": {path, len, mtime_ns} | null}, plus
    /// "acqd_absent" (why) when acqd is null; `--version` is the human
    /// form of the first two.
    Version,
    /// The live jobs: id, parent, kind, target (from params, C7), state
    /// (`↻n` counts 429 re-queues, C26), priority, account, submitter, ETA.
    Jobs {
        /// Subscribe, print the queue, then every job-state change as it
        /// happens; the queue is printed again after a missed-events signal
        /// (C85), and the watch ends when the daemon stops.
        #[arg(long)]
        watch: bool,
    },
    /// One job's state and ETA. A large ETA is the limiter holding, not a
    /// hang: holds can reach 300 s plus the timing bucket.
    Status { id: u64 },
    /// A finished job's payload or error, answered across daemon restarts
    /// (C27: a client that disappears leaves its jobs running). A failed
    /// fetch's refused body is in `acq store refused <id>`, not here.
    Result { id: u64 },
    /// Cancel a waiting or running job; cascades to every descendant
    /// still waiting (C23).
    Cancel { id: u64 },
    /// Change a waiting job's priority; higher runs sooner and the queue
    /// reorders live (C5).
    SetPriority { id: u64, priority: u8 },
    /// The listing state (no daemon): what every item, tab and character
    /// is priced as — by hand, in game, and how the two stand.
    Price {
        #[command(subcommand)]
        cmd: Option<PriceCmd>,
    },
    /// Reference data the binary ships (no store, no daemon): the currency
    /// table, by version.
    Reference {
        #[command(subcommand)]
        cmd: ReferenceCmd,
    },
    /// The forum shop (no daemon, sends nothing): render the page set a
    /// hand price would post, every omission counted with its reason.
    Shop {
        #[command(subcommand)]
        cmd: ShopCmd,
    },
    /// Debugging only — normal use never needs manual lifecycle.
    Daemon {
        #[command(subcommand)]
        cmd: DaemonCmd,
    },
}

#[derive(Subcommand)]
enum PriceCmd {
    /// One line on the league's listing state and the next action (the
    /// default); --expand adds the game side, the rows and the basis.
    Status {
        #[arg(long, default_value = "Standard")]
        league: String,
        #[arg(long, value_parser = parse_realm)]
        realm: Option<Realm>,
        #[arg(long)]
        expand: bool,
    },
    /// One target's listing: `item/<id>`, `character/<id>`,
    /// `tab/<realm>/<id>` or `substash/<realm>/<parent>/<id>` — both
    /// sides with their causes, the raw note beside the parse; a
    /// container's items summarized under it (--expand lists them all).
    Show {
        target: String,
        #[arg(long, default_value = "Standard")]
        league: String,
        /// The realm of an item or character target; a tab or substash
        /// address carries its own.
        #[arg(long, value_parser = parse_realm)]
        realm: Option<Realm>,
        #[arg(long)]
        expand: bool,
    },
    /// Listed items grouped by container, ten or fewer listed per group
    /// and more counted (--expand lists every one with its texts).
    /// With neither --relation nor --effective, items whose relation is
    /// `none` are left out — except unresolved ones (a row that cannot
    /// be read could decide), which are always listed.
    List {
        #[arg(long, default_value = "Standard")]
        league: String,
        /// The realm; a tab or substash address given to --in or
        /// --covered-by carries its own.
        #[arg(long, value_parser = parse_realm)]
        realm: Option<Realm>,
        /// none, manual_only, game_only, agree or conflict.
        #[arg(long)]
        relation: Option<String>,
        /// Who decides (C81): game, manual, none or unresolved.
        #[arg(long)]
        effective: Option<String>,
        /// Only items physically in this container (a tab, substash or
        /// character address).
        #[arg(long = "in")]
        location: Option<String>,
        /// Only items a row on this target would cover (C70): a folder's
        /// tabs' items, a map or unique tab's substashes' items too.
        #[arg(long)]
        covered_by: Option<String>,
        #[arg(long)]
        expand: bool,
    },
    /// Set one target's price by hand — a row on an item, tab, substash
    /// or character; what it covers inherits it (C70). Prints what it
    /// replaced, and how to put that back.
    Set {
        /// `item/<id>`, `character/<id>`, `tab/<realm>/<id>` or
        /// `substash/<realm>/<parent>/<id>`.
        target: String,
        /// exact, negotiable, no_price or skip (the game's `price`, `b/o`
        /// and `~skip` work too).
        #[arg(value_name = "TYPE")]
        kind: String,
        /// For exact and negotiable: a decimal of up to four places
        /// (`12.5`), or a `wanted/lot` ratio (`1/5`).
        amount: Option<String>,
        /// For exact and negotiable: a tag of `acq reference currency`.
        currency: Option<String>,
        /// Only write over exactly this revision (what `acq price show`
        /// printed when you reviewed it); refused naming the current one
        /// otherwise. Without it the write replaces whatever is stored —
        /// though a write racing in between still conflicts rather than
        /// being clobbered.
        #[arg(long)]
        if_revision: Option<i64>,
    },
    /// Remove one target's own price row; what it covered falls back to
    /// the next row up (C70). Prints what it removed and the command that
    /// puts it back.
    Clear {
        /// The address, as for `set`.
        target: String,
        /// Only clear exactly this revision; see `set`.
        #[arg(long)]
        if_revision: Option<i64>,
    },
}

#[derive(Subcommand)]
enum ShopCmd {
    /// Render the shop pages to stdout for pasting by hand (C74): one
    /// link code per hand-priced item under its price's spoiler, one
    /// page spoiler labelled n of N, in pages of at most --size
    /// characters; what the
    /// game already lists is omitted and counted, an unobserved case is
    /// blocked and counted; the sync policy's coverage and freshness are
    /// reported, never enforced (C72).
    Render {
        #[arg(long, default_value = "Standard")]
        league: String,
        #[arg(long, value_parser = parse_realm)]
        realm: Option<Realm>,
        /// Characters per page, the template included (the forum's hard
        /// limit, T23).
        #[arg(long, default_value_t = acquisition_plan::shop::DEFAULT_PAGE_SIZE)]
        size: usize,
        /// A file whose `[items]` token each page replaces; without it
        /// the page is the items alone.
        #[arg(long)]
        template: Option<std::path::PathBuf>,
        /// Print one page alone (its text; with --json, its record), for
        /// the clipboard.
        #[arg(long)]
        page: Option<usize>,
        /// The whole policy table with each row's rule and count, and
        /// every item left off the page with its cell.
        #[arg(long)]
        expand: bool,
    },
}

#[derive(Subcommand)]
enum ReferenceCmd {
    /// The currency table: every row (tag, display name, the words a
    /// parser accepts, retired marks), or one word resolved. Exact and
    /// case-sensitive; `--json` is the whole table with its evidence.
    Currency {
        /// A tag, the game's word, or a legacy alias to resolve.
        word: Option<String>,
        /// Show each row's evidence and the table's sources.
        #[arg(long)]
        expand: bool,
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
    /// The daemon running: pid, version, contract revision, the executable
    /// it runs from and its hash (C84), provider, the world it serves
    /// (C83: its canonical store root), uptime, connections, queue
    /// counts, policies learned, the socket, log and journal paths, the
    /// rails state, keyring health. Observes only (C10): never spawns or
    /// replaces; a daemon of another contract, artifact, provider or
    /// world is reported by its identity alone — no vitals — and left
    /// running. `--json`, in both cases: running, compatible, `socket` (the
    /// one this shell reached it through: derived from the world, C83;
    /// `legacy_socket` true for a daemon from before the derived
    /// rendezvous, never used or replaced), which of
    /// the four dimensions match (`contract_matches`, `artifact_matches`,
    /// `provider_matches`, `world_matches`), how the daemon's file relates to this
    /// client's sibling — `artifact_relation`: `same_file`; `same_bytes`,
    /// another copy; `different`; `unhashable`; `no_sibling`, no `acqd`
    /// beside this executable; `unreported`, a daemon from before the
    /// field — and under `wanted` this client's own version, contract,
    /// provider and world (`world_absent` says why when this shell's
    /// store root does not exist) with the sibling `acqd` a job command
    /// would start, all from the one look that judged the daemon.
    Status,
    /// Stop the daemon listening on this shell's world's socket, this
    /// build's — its contract and artifact — or another's; when that
    /// socket is silent, one from before the derived rendezvous on the
    /// legacy socket (C83; stopped once, it never comes back).
    /// Queued jobs stay on disk and resume under the next one (C6); a
    /// client's jobs are never cancelled by its leaving (C27). `--json`:
    /// `stopped`, `pid`, `version`, `provider`, `compatible`, `socket`,
    /// `legacy_socket`.
    Stop,
    /// Clear the live-test rails' tripwire/ceiling halt (see LIVE-TESTING.md).
    /// Observe the post-violation rule before using this. With no daemon
    /// running, clears the persisted state in this shell's world
    /// (`<store root>/<provider>/rails.json`, C83) so the next daemon
    /// starts clear.
    ResetTripwire,
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
                // A daemon refusal carries its closed kind beside the
                // message (C85; additive under C53).
                let mut report = json!({ "error": format!("{e:#}") });
                if let Some(refusal) = DaemonError::find(&e) {
                    report["kind"] = json!(refusal.kind);
                }
                println!("{report}");
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
                let mut client = attach().await?;
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
                    Response::Error { kind, message } => {
                        Err(anyhow::Error::from(DaemonError { kind, message })
                            .context("auth check failed"))
                    }
                    status => {
                        if !cli.json {
                            println!("session verified (live token round-trip succeeded)");
                        }
                        print_auth(&status, cli.json)
                    }
                }
            }
            Some(AuthCmd::Logout) => {
                let mut client = attach().await?;
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
        Cmd::Price { cmd } => match cmd.unwrap_or(PriceCmd::Status {
            league: "Standard".into(),
            realm: None,
            expand: false,
        }) {
            PriceCmd::Status {
                league,
                realm,
                expand,
            } => price_cmd::status(realm.unwrap_or(Realm::DEFAULT), &league, expand, cli.json),
            PriceCmd::Show {
                target,
                league,
                realm,
                expand,
            } => price_cmd::show(&target, realm, &league, expand, cli.json),
            PriceCmd::List {
                league,
                realm,
                relation,
                effective,
                location,
                covered_by,
                expand,
            } => price_cmd::list(
                realm,
                &league,
                &price_cmd::ListArgs {
                    relation,
                    effective,
                    location,
                    covered_by,
                },
                expand,
                cli.json,
            ),
            PriceCmd::Set {
                target,
                kind,
                amount,
                currency,
                if_revision,
            } => price_cmd::set(
                &target,
                &kind,
                amount.as_deref(),
                currency.as_deref(),
                if_revision,
                cli.json,
            ),
            PriceCmd::Clear {
                target,
                if_revision,
            } => price_cmd::clear(&target, if_revision, cli.json),
        },
        Cmd::Reference { cmd } => match cmd {
            ReferenceCmd::Currency { word, expand } => {
                reference_cmd::currency(word.as_deref(), expand, cli.json)
            }
        },
        Cmd::Shop { cmd } => match cmd {
            ShopCmd::Render {
                league,
                realm,
                size,
                template,
                page,
                expand,
            } => shop_cmd::render_cmd(
                realm.unwrap_or(Realm::DEFAULT),
                &league,
                &shop_cmd::RenderArgs {
                    size,
                    template: template.as_deref(),
                    page,
                    expand,
                    json: cli.json,
                },
            ),
        },
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
        Cmd::Version => {
            // The candidate, not the daemon running (C84): the sibling
            // `acqd` as found, with no hash — the run record hashes it.
            let sibling = acquisition_client::artifact::sibling();
            if cli.json {
                let mut report = json!({
                    "version": acquisition_protocol::VERSION,
                    "contract": acquisition_protocol::CONTRACT_REVISION,
                    "provider": acquisition_protocol::provider::wanted(),
                    "acqd": sibling.as_ref().ok().map(|s| json!({
                        "path": s.identity.path,
                        "len": s.identity.len,
                        "mtime_ns": s.identity.mtime_ns,
                    })),
                });
                if let Err(e) = &sibling {
                    report["acqd_absent"] = json!(e.to_string());
                }
                println!("{report}");
            } else {
                println!("acq {}", acquisition_protocol::VERSION_WITH_CONTRACT);
                match sibling {
                    Ok(s) => println!(
                        "acqd: {} ({} bytes) — what a job command would start; `acq daemon status` reports the daemon running",
                        s.identity.path, s.identity.len
                    ),
                    Err(e) => println!("acqd: none — {e}"),
                }
            }
            Ok(())
        }
        Cmd::Jobs { watch } => {
            if watch {
                return watch_jobs(cli.json).await;
            }
            let mut client = attach().await?;
            let jobs = list(&mut client).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&jobs)?);
            } else {
                print_table(&jobs);
            }
            Ok(())
        }
        Cmd::Status { id } => {
            let mut client = attach().await?;
            let job = client.status(id).await?;
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&job)?);
            } else {
                print_table(std::slice::from_ref(&job));
            }
            Ok(())
        }
        Cmd::Result { id } => {
            let mut client = attach().await?;
            print_result(&mut client, id, cli.json).await
        }
        Cmd::Cancel { id } => {
            let mut client = attach().await?;
            client.expect_ack(&Request::Cancel { id }).await?;
            if cli.json {
                println!("{}", json!({ "job_id": id, "cancel_requested": true }));
            } else {
                println!("job {id} cancel requested");
            }
            Ok(())
        }
        Cmd::SetPriority { id, priority } => {
            let mut client = attach().await?;
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
            DaemonCmd::Status => {
                // An observation (C10): absent, this build's, or another
                // daemon reported and left alone — never spawned or replaced.
                let mut client = match Client::observe().await? {
                    Observed::Compatible(c) => c,
                    Observed::Absent => {
                        if cli.json {
                            println!("{}", json!({ "running": false }));
                        } else {
                            println!("daemon is not running");
                        }
                        return Ok(());
                    }
                    Observed::Incompatible(found) => {
                        if cli.json {
                            // `socket` and `legacy_socket` are the report's:
                            // the endpoint the handshake ran over.
                            let mut report = found.report();
                            report["running"] = json!(true);
                            report["compatible"] = json!(false);
                            println!("{}", serde_json::to_string_pretty(&report)?);
                        } else {
                            println!("{found} — running, not this client's");
                            println!("socket: {}", found.socket());
                            println!("next:   {}", mismatch_remedy(&found));
                        }
                        return Ok(());
                    }
                };
                let found = client.daemon().clone();
                let status = client.request(&Request::DaemonStatus).await?;
                if cli.json {
                    let mut report = serde_json::to_value(&status)?;
                    report["running"] = json!(true);
                    report["compatible"] = json!(true);
                    // The identity the handshake carried (C84), beside the
                    // vitals: the same keys the incompatible report has —
                    // the daemon's contract and artifact, how the artifact
                    // relates to this client's sibling (`same_file` or
                    // `same_bytes`, a copy), and the sibling itself under
                    // `wanted`, so "where did it run from" and "is that my
                    // sibling" are both answered (review 2026-09-10).
                    let identity = found.report();
                    // `socket` is the one this shell reached the daemon
                    // through (C83: derived from the world); the log is
                    // the daemon's own report (`log`, on the wire since
                    // the world), never recomputed here.
                    for key in [
                        "contract",
                        "artifact",
                        "world",
                        "socket",
                        "legacy_socket",
                        "contract_matches",
                        "artifact_matches",
                        "artifact_relation",
                        "provider_matches",
                        "world_matches",
                        "wanted",
                    ] {
                        report[key] = identity[key].clone();
                    }
                    println!("{}", serde_json::to_string_pretty(&report)?);
                } else if let Response::DaemonStatus {
                    pid,
                    version,
                    contract,
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
                    log,
                } = status
                {
                    println!(
                        "daemon {version} (contract {contract}) pid {pid}, up {uptime_seconds}s, provider {provider}"
                    );
                    println!("world:  {}", found.world());
                    match (found.artifact(), &found.verdict().artifact) {
                        (
                            Some(a),
                            acquisition_client::artifact::ArtifactVerdict::SameBytes { sibling },
                        ) => {
                            println!(
                                "acqd:   {} (sha256 {}) — the same artifact as this client's sibling {}, another copy",
                                a.file.path,
                                a.short_hash(),
                                sibling.path
                            )
                        }
                        (Some(a), _) => println!(
                            "acqd:   {} (sha256 {}) — this client's sibling, the file a job command would start",
                            a.file.path,
                            a.short_hash()
                        ),
                        (None, _) => println!("acqd:   not reported"),
                    }
                    println!(
                        "connections: {connections}  waiting: {jobs_waiting}  running: {jobs_running}  in flight: {in_flight}/{max_in_flight}  policies learned: {policies_known}"
                    );
                    println!("socket: {}", found.socket());
                    println!("log:    {}", log.as_deref().unwrap_or("not reported"));
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
                match Client::observe().await? {
                    Observed::Compatible(mut client) => {
                        let resp = client.request(&Request::ResetTripwire).await?;
                        if cli.json {
                            println!("{}", serde_json::to_string_pretty(&resp)?);
                        } else {
                            println!("rails reset");
                        }
                    }
                    // Another daemon holds its rails in memory; clearing the
                    // file under it would not reset anything. Stop it first.
                    Observed::Incompatible(found) => {
                        bail!("{found}; `acq daemon stop` first")
                    }
                    Observed::Absent => {
                        // The trip lives on disk, in the world (C83) — and,
                        // until a daemon has moved it, beside the socket;
                        // clear both so the next spawned daemon is not
                        // still halted.
                        let provider = acquisition_protocol::provider::wanted();
                        let mut candidates =
                            vec![acquisition_store::world::legacy_rails_state_path(provider)];
                        if let Ok(world) = World::observe() {
                            candidates.insert(0, world.rails_state_path(provider));
                        }
                        // A file, or an empty directory in a file's place
                        // (the daemon refuses to start over either shape
                        // it cannot read, and names this verb as the
                        // remedy); anything else is named for the hand.
                        let mut cleared = Vec::new();
                        for state in &candidates {
                            let removed = match std::fs::symlink_metadata(state) {
                                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                                Err(e) => Err(e),
                                Ok(meta) if meta.is_dir() => std::fs::remove_dir(state).map_err(|e| {
                                    std::io::Error::new(
                                        e.kind(),
                                        format!("it is a directory that is not empty ({e}); remove it by hand"),
                                    )
                                }),
                                Ok(_) => std::fs::remove_file(state),
                            };
                            match removed {
                                Ok(()) => cleared.push(state.clone()),
                                Err(e) => bail!(
                                    "daemon is not running; could not clear {}: {e}",
                                    state.display()
                                ),
                            }
                        }
                        if cli.json {
                            println!(
                                "{}",
                                json!({ "cleared": !cleared.is_empty(), "state": cleared.first() })
                            );
                        } else if cleared.is_empty() {
                            println!("daemon is not running and no rails state is persisted");
                        } else {
                            for state in cleared {
                                println!(
                                    "daemon is not running; cleared persisted rails state {}",
                                    state.display()
                                );
                            }
                        }
                    }
                }
                Ok(())
            }
            DaemonCmd::Stop => {
                // Stops this build's daemon or any other (C10): stopping is
                // how a mismatch is resolved by hand.
                match Client::stop_any().await? {
                    Some(found) => {
                        if cli.json {
                            println!(
                                "{}",
                                json!({
                                    "stopped": true,
                                    "pid": found.pid(),
                                    "version": found.version(),
                                    "provider": found.provider(),
                                    "compatible": found.is_ours(),
                                    "socket": found.socket(),
                                    "legacy_socket": found.on_legacy_socket(),
                                })
                            );
                        } else if found.is_ours() {
                            println!("daemon stopped (pid {})", found.pid());
                        } else {
                            println!("stopped: {found}");
                        }
                    }
                    None => {
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
        Response::Error { kind, message } => return Err(DaemonError { kind, message }.into()),
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
    if provider == GGG {
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
        Response::Error { kind, message } => Err(DaemonError { kind, message }.into()),
        other => bail!("unexpected response: {other:?}"),
    }
}

async fn list(client: &mut Client) -> Result<Vec<JobInfo>> {
    match client.request(&Request::List).await? {
        Response::Jobs { jobs } => Ok(jobs),
        other => bail!("unexpected response: {other:?}"),
    }
}

/// `jobs --watch`: the subscriber's sequence under C85, as the reference
/// consumer. Subscribe first, then take the snapshot over a request
/// connection to the same daemon instance (a restart landing between the
/// two connections starts the sequence over); a job that changes between
/// subscribing and reading is an event, never a gap. Every event is an
/// invalidation, so the job is re-read over the request connection before
/// its line is printed — an event never stands in for the read, and a
/// read that fails restarts the sequence rather than trusting the hint.
/// On `resync_required` the lagged subscription is dropped — its queued
/// events predate any snapshot taken now — and the sequence starts over
/// with a fresh subscription and snapshot; when the daemon goes, observe
/// it again — an observer never spawns — and subscribe and snapshot
/// afresh if it is back. Pinned by `tests/watch_recovery.rs`.
async fn watch_jobs(json: bool) -> Result<()> {
    let print_snapshot = |jobs: &[JobInfo]| -> Result<()> {
        if json {
            println!("{}", serde_json::to_string_pretty(jobs)?);
        } else {
            print_table(jobs);
        }
        Ok(())
    };
    loop {
        let mut subscription = match Subscription::observe().await? {
            Observed::Compatible(subscription) => subscription,
            Observed::Absent => bail!("daemon stopped (it spawns on demand for job commands)"),
            Observed::Incompatible(found) => bail!("{found}"),
        };
        let mut client = attach().await?;
        if client.daemon().pid() != subscription.daemon().pid() {
            // A restart between the two connections: the subscription is
            // to a daemon that is gone. Start over.
            continue;
        }
        print_snapshot(&list(&mut client).await?)?;
        loop {
            let signal = match subscription.next().await {
                Ok(signal) => signal,
                // A transport failure mid-stream is a disconnect; anything
                // else is a protocol violation worth stopping on.
                Err(e) if e.downcast_ref::<std::io::Error>().is_some() => None,
                Err(e) => return Err(e),
            };
            match signal {
                Some(Signal::Event(hint)) => {
                    // The read, never the hint, is what gets printed; a
                    // read that fails restarts the sequence.
                    let job = match client.status(hint.id).await {
                        Ok(job) => job,
                        Err(e) => {
                            if !json {
                                println!(
                                    "re-reading job {} failed ({e:#}); subscribing and reading again",
                                    hint.id
                                );
                            }
                            break;
                        }
                    };
                    if json {
                        println!("{}", serde_json::to_string(&job)?);
                    } else {
                        println!("job {:>3}  {:<8} -> {}", job.id, job.kind, job.state);
                    }
                }
                Some(Signal::ResyncRequired { missed }) => {
                    if !json {
                        println!("missed {missed} event(s); subscribing and reading again");
                    }
                    break;
                }
                None => {
                    if !json {
                        println!("daemon went away; watching for it");
                    }
                    break;
                }
            }
        }
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
                Response::Error { kind, message } => Err(DaemonError { kind, message }.into()),
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
        Response::Error { kind, message } => return Err(DaemonError { kind, message }.into()),
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
    fn price_set_takes_a_type_then_an_optional_amount_and_currency() {
        for ok in [
            vec!["acq", "price", "set", "item/i1", "exact", "12.5", "chaos"],
            vec![
                "acq",
                "price",
                "set",
                "tab/pc/t1",
                "b/o",
                "1/5",
                "divine",
                "--if-revision",
                "2",
            ],
            vec!["acq", "price", "set", "item/i1", "skip"],
            vec!["acq", "price", "clear", "item/i1", "--if-revision", "3"],
        ] {
            assert!(Cli::try_parse_from(&ok).is_ok(), "{ok:?} must parse");
        }
        // The value's words are positional and at most three: a fourth is
        // a parse error, never a silently dropped word.
        assert!(
            Cli::try_parse_from([
                "acq", "price", "set", "item/i1", "exact", "1", "chaos", "extra"
            ])
            .is_err()
        );
        assert!(Cli::try_parse_from(["acq", "price", "set", "item/i1"]).is_err());
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
