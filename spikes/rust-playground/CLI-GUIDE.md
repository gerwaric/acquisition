# The `acq` CLI — a self-contained guide

**Snapshot, written 2026-08-31.** This document is a deliberate point-in-time
description of the CLI and the concepts it sits on (the daemon, the job
model, the shared store, accounts, providers), written to be readable
without the repository — for design conversations that bring in outside
material. It intentionally duplicates what code and other docs say; when it
disagrees with them, **they win** (`CONTEXT.md` for boundaries and
decisions, the code for implemented shape, `LIVE-TESTING.md` for anything
touching the real GGG API). It is not maintained as code changes.

---

## 1. What the system is

Acquisition is a Path of Exile inventory-management tool being rewritten
from C++/Qt to Rust. The architecture is a **local daemon that owns all
GGG API traffic**, with thin frontends. Four exist or are planned:

1. **`acq`** — the CLI, and currently the primary human interface.
2. **`acq-mcp`** — an MCP server over stdio, for agents.
3. A GUI (Tauri) — planned, not started.
4. (Internal) the daemon itself is reached via `acq daemon run`, which is
   what lazy spawn executes.

A frontend consumes **exactly two surfaces**, by decision:

- **The daemon protocol** — JSON lines over a Unix socket — for anything
  that acts: submitting jobs, auth, watching the queue.
- **The store crate's read API** — direct SQLite reads of the shared
  store — for anything that only looks at data already fetched. No daemon
  round-trip, no network; these work with no daemon running at all.

There is deliberately no third channel. The GGG relationship (the OAuth
registration, rate-limit standing) is the project's most valuable asset;
the design's one structural guarantee is that every daemon-originated
request to GGG passes through a single rate-limit choke point and a single
send-lifetime gate.

**Status in one line:** the daemon's GGG-facing side is *proven* (a live
test ladder closed 2026-08-27: every rung passed, zero 429s across ~1,450
real sends); the frontend-facing side (protocol verbs, store read API) is
the *frontier* — the boundary's location is pinned, its content is still
being validated by consumers and is revisable.

## 2. Providers: mock by default, real GGG by opt-in

By default **nothing talks to GGG**. The daemon runs against an in-process
localhost mock (`mockggg.rs`) that truthfully simulates OAuth and GGG's
rate-limit behavior: real sliding windows, real 429s with `Retry-After`,
HEADs that report but don't count (except where GGG really counts them),
a `/profile` that answers without rate headers and refuses HEAD — the
known quirks of the real API are reproduced. Mock access tokens live 60
seconds, so silent refresh is exercised constantly.

`ACQ_GGG=1` on any command selects the real provider:
`api.pathofexile.com`, real OAuth in the browser, the existing
"acquisition" registration (same client id, callback path, scopes,
user-agent as the shipped C++ app). The daemon handshake carries the
provider name; the CLI kills and respawns a daemon running in the wrong
mode, so mock and real never mix on one daemon. Mock and real refresh
tokens live in separate keyring entries, so a mock token can never be
sent to GGG.

**Live-testing is human-controlled.** `LIVE-TESTING.md` is the control
document; the standing rule is: never set `ACQ_GGG=1` without it (rails
on, ceiling 3, read the journal, record the policy). Agent sessions work
against the mock.

## 3. The daemon

- **Lazy spawn, idle exit** (the gpg-agent model). Any command that needs
  a daemon spawns one if none answers; the daemon exits on its own after
  60 s with no connections and no live jobs — unless the rate limiter
  still holds history inside a policy window (up to 300 s), in which case
  it stays up so a quick respawn doesn't have to assume the worst about
  hits it can no longer see. Normal use never manages the lifecycle;
  `acq daemon status|stop` exist for debugging.
- **Single version, kill-and-respawn migration.** The protocol has a
  version handshake and no compat story: an interactive CLI that meets a
  version- or provider-mismatched daemon kills and replaces it (the human
  at the terminal is expressing intent). An autonomous client (MCP)
  **never** kills or replaces a daemon — the mismatch it sees might be a
  human's live GGG run — it reports and stops.
- **Transport**: JSON lines over a Unix socket, one JSON object per line
  each way. Request/response, plus an event channel: a connection that
  sends `subscribe` receives unsolicited job-state-changed events
  interleaved with responses. The live verb list is
  `crates/acquisition-core/src/protocol.rs`; the verbs (as of this
  snapshot): `hello`, `submit`, `status`, `result`, `cancel`,
  `set_priority`, `list`, `subscribe`, `auth_start`, `auth_status`,
  `auth_check`, `auth_logout`, `daemon_status`, `daemon_stop`,
  `reset_tripwire`, `dashboard`.
- **Unix only** so far (macOS/Linux); Windows named pipes are a known gap
  the protocol doesn't preclude.
- The daemon's log lives next to the socket; `acq daemon status` prints
  both paths. A daemon that refuses to start (broken `daemon.db`, failed
  socket bind) writes the refusal to the log, and a lazy spawn that dies
  surfaces it: the CLI notices and prints the log's new lines instead of
  timing out.

## 4. Jobs: the core abstraction

**API requests are jobs, not calls.** Rate-limit waits can reach five
minutes; a blocking call can't represent that across frontends. Every
network touch — API GETs, and even endpoint discovery — is a job with:

- **`id`** — a `u64`, monotonically increasing, never reused (SQLite
  `AUTOINCREMENT`), so a stale `acq result <id>` can never name a
  different job.
- **`kind`** — `profile`, `characters`, `character`, `leagues`,
  `stashes`, `stash`, `refresh`; mock-only `sleep`, `fetch`, `whoami`;
  and the daemon-queued `probe` (see rate limiting).
- **`state`** — `waiting → running → done | failed | cancelled`. A 429
  moves a job back to `waiting` with a retry counter (rendered `↻n` in
  job tables), bounded by `MAX_429_RETRIES` (2).
- **`priority`** — `u8`, higher runs sooner, ties break FIFO; changeable
  while waiting (`acq set-priority`).
- **`eta_seconds`** — predicted start time for waiting jobs, computed
  from limiter state plus same-route queue depth. The daemon can predict
  because it sees everything.
- **`parent`** — fan-out work (a `refresh`, a `--deep` stash) is a
  parent job that submits child jobs; the parent finishes when its last
  descendant does, and cancelling it cascades to every descendant still
  waiting. The children only exist once the parent runs, so any progress
  view must expect the denominator to grow.
- **`account`** — which GGG account the job runs as (below); fixed at
  submit.
- **`params`** — the submitted params, verbatim and **public** (every
  connected client sees them; a job's params must never carry a secret —
  tokens are obtained inside the daemon, never passed in). This is what
  makes a queued job identifiable to a person; rendering is the client's
  business (`acq jobs`' `target` column derives labels like
  `Standard/cur1` from params).

**The queue persists.** Every job is mirrored to a `jobs` table in
`daemon.db` (SQLite, in the provider's store directory) at each state
change, and open jobs are taken back at daemon start — so the queue
survives idle exit, `daemon stop`, a version respawn, or a crash. A job
that was mid-run is re-queued (safe: every network kind is an idempotent
GET, and a restart probe reads GGG's counters before sending), with two
carve-outs: on a no-probe route it fails as `interrupted`, and a parent
restarted mid-fan-out holds for the children it already has and then
finishes as `interrupted` (it can't know how many children it never
submitted, so it must not claim success). Finished rows are kept for
`acq result <id>` across restarts, pruned by age at start (7 days
default; 30 for failed). A queue write failure at runtime is sticky: the
daemon refuses new jobs and stops dispatching until a restart finds a
working `daemon.db`.

**Clients are disposable.** Ctrl-C, a closed terminal, a crash — none of
it cancels anything. Sends are committed once dispatched; a client that
wants the outcome reattaches with `acq result <id>`.

## 5. Accounts and auth

- **Login is OAuth via the browser**: `acq auth` asks the daemon to
  start the flow (PKCE + loopback redirect listener), hands the URL to
  the browser, polls until done. The mock's login page accepts any
  username, so multi-account testing is one login apart. Refresh tokens
  live in the OS keyring (never plaintext on disk); one entry per
  account. `ACQ_NO_KEYRING=1` degrades sessions to memory-only.
- **Identity is the token response's `username`** (`name#discriminator`).
- **Multi-account is one daemon holding many sessions** — never one
  daemon per account (two daemons on one IP would each hold their own
  Cloudflare burst gate and see half the picture). Log in again as
  someone else and both sessions stay live.
- **No default account; stateless selection.** With one session the
  selector is implicit. With several, every job command and store read
  refuses and lists the choices. `--account <username|name|uuid>` is a
  global flag (env form `ACQ_ACCOUNT`); matching is exact, never by
  prefix. The CLI resolves it client-side against a non-secret index
  file (`accounts.json` in the store directory), so store reads never
  spawn a daemon.
- The account is **fixed on the job at submit** — it selects the token
  the job sends with and the store file its response lands in — and is
  checked again when the token is taken, so a job whose account was
  logged out meanwhile fails without a send.
- `acq auth logout [--account A]` drops one session; a logged-out
  account stays listed (its store file remains) as "not persisted".
- The rate limiter paces `Account`-scoped policies **per account** (GGG
  counts them per account — established live, rung 11) and `Ip`-scoped
  ones (the OAuth token endpoint) once for all.

## 6. The shared store

`acquisition-store`: SQLite, **one file per account** under one directory
per provider (`<store>/<provider>/<account>.db`), plus `accounts.json`
(the non-secret account index) and `daemon.db` (the persisted job queue)
in the same directory.

The division of labor is strict:

- **The daemon only writes**: one call, `record(endpoint, params,
  status, body)`, after each storable API success. It never reads the
  store and never looks inside a body.
- **Every frontend reads the file directly** (WAL makes concurrent reads
  safe) through the store crate's functions — the CLI, the MCP server,
  and a future GUI all call the same code and see the same data.

Bodies are stored verbatim **except at the item seams**: every item array
(tab `items`, character `inventory`/`equipment`/`jewels`/`rucksack`, each
`socketedItems`) is lifted into an `items` table, one row per GGG item id
(stable across moves), keyed by location. `items` is the only place to
look for an item. Each ingest diffs against the previous state and writes
`item_events` (added/moved/changed/removed; volatile fields like
`veiledMods` are ignored to avoid false changes). Derived columns come
from the row's own JSON, so `acq store rebuild` re-extracts without a
refetch, and `acq store import <snapshot>` replays an old client-side
snapshot through the same ingest with zero network (the real-data
fixture path: 322 tabs / 19,210 rows in ~2.3 s).

Store reads currently surfaced in the CLI: `accounts`, `tabs`,
`items search`, `items show`, `store status`, `store events`, plus the
maintenance verbs `store rebuild` and `store import`.

## 7. Rate limiting, as a CLI user experiences it

The limiter is **header-driven**: it knows nothing except what responses
told it (`X-Rate-Limit-*` per policy name, plus when counted responses
arrived) and pads every wait by GGG's server-side timing bucket. Local
state is a prediction; headers correct it. Its full spec is the test
table in `ratelimit.rs`, keyed to `docs/design/network-ground-truth.md`
claim numbers. What that means at the terminal:

- The limiter **starts empty each daemon lifetime**. The first job on an
  endpoint queues a visible **`probe` job** — one HEAD that teaches the
  policy and the account's *current* counters (including hits made by
  other tools) before anything real is sent. Two routes are declared
  no-probe from route knowledge (`/profile`: no rate policy at all,
  confirmed by GGG; `/account/leagues`: GGG currently counts the HEAD)
  and are taught by their first GET.
- A **429 re-queues the job** behind the limiter's hold (`↻n` in tables,
  "got a 429, retry 1 in ~Ns..." in blocking mode), at most twice, then
  fails with the evidence. A **Cloudflare-shaped 403/503 is never
  retried** — that's an invariant, not a policy.
- Blocking commands show the wait: `job 12: rate limited, starting in
  ~4m37s...`. Long holds (up to 300 s + 60 s) are the limiter working,
  not a hang.
- `acq dash` shows the live limiter state: per-policy buckets, the
  observed headers, per-endpoint sends, the state keys
  (`stash-request-limit@Alice#1234` for account-scoped,
  `token-request-limit` for the shared IP-scoped token endpoint).
- All daemon HTTP — API GETs, HEAD probes, OAuth code exchange and
  refresh — goes through the one choke point; even token traffic is
  rate-limit-paced (GGG's token endpoint has its own policy). A global
  send gate bounds concurrent live sends (Cloudflare watches bursts
  across policies).

## 8. Output contract

- **`--json` on every command, total.** The CLI is itself an API.
  Failures are `{"error": …}` on **stdout** with exit 1; a failed job's
  outcome exits 1 in both output modes. `acq auth --no-browser --json`
  prints `{"authorize_url": …}` as its first line so a scripted login
  can read the URL (final auth status is the second line).
- **Default mode is blocking-with-progress**; `--detach` on `submit` is
  the async form (returns `{"job_id": n}` immediately; reattach with
  `acq status`/`acq result`).
- `acq --version` prints `<pkg version> (<git commit>)` — the thing to
  check before a live run is the binary, not the checkout; `-dirty`
  means uncommitted changes were built in.

## 9. Command reference

Global flags: `--json` (everywhere), `--account <sel>` / `ACQ_ACCOUNT`.

### Auth

| Command | Does |
|---|---|
| `acq auth [--no-browser]` | OAuth login. Opens the provider's page (or prints the URL); polls until the flow resolves (5-min deadline). |
| `acq auth status` | Local belief: sessions, token expiry, keyring health, per account. Also lists other known (not live) accounts. |
| `acq auth check` | Preflight: proves the session via a forced token round-trip through the provider. Exit 1 on failure. |
| `acq auth logout [--account A]` | Drops the session and clears its keyring entry; with another account named, clears only that entry. |

### Data-fetching (jobs; daemon + network)

Each of these submits a job and blocks with progress; all take `--json`.

| Command | Does |
|---|---|
| `acq profile` | GET `/profile` (called at most once per login; policyless route). |
| `acq characters` | List characters. |
| `acq character <name>` | One character with equipment + inventory. |
| `acq leagues` | GET `/account/leagues`. |
| `acq stashes [--league L]` | List stash tabs (default league `Standard`). |
| `acq stash <id> [--sub <id>] [--deep] [--league L]` | One tab, or one substash. `--deep` follows a map/unique tab's substashes as child jobs (opt-in per tab — one map tab can hold hundreds). |
| `acq refresh --tabs a,b,c \| --all [--deep] [--league L]` | One stash-list request, then one `stash` child per selected tab; the parent finishes last. Selection is explicit — no default. |

### Store reads (no daemon, no network)

| Command | Does |
|---|---|
| `acq accounts` | Accounts this machine has logged into, from the index: username, last login, keyring persistence, store size, uuid. |
| `acq tabs [--league L]` | Tab tree with live item counts and per-tab fetch age. |
| `acq store characters [--realm R] [--league L]` | Characters on record by GGG id: address (name), realm, league, level, live item count, listed/fetched age. |
| `acq items search <text> [--league L] [--removed] [--limit N]` | Substring search over name/type/base; socketed gems are rows too. |
| `acq items show <id>` | One item, verbatim JSON plus location/first-seen/last-seen. |
| `acq store status` | Store path, size, row counts. |
| `acq store events [--hours N] [--limit N]` | What recent ingests concluded (added/moved/changed/removed). |
| `acq store rebuild` | Re-extract derived columns from each item's own JSON. |
| `acq store import <snapshot.json>` | Replay a retired-`acq pull` snapshot into the store. |

### Reference data (no store, no daemon)

| Command | What it does |
| --- | --- |
| `acq reference currency [WORD] [--expand]` | The currency table the binary ships, cited by version: every row (tag, display name, the words a parser accepts, retired marks) or one word resolved — exact, case-sensitive. `--expand` adds each row's evidence; `--json` is the whole table. |

### Job management

| Command | Does |
|---|---|
| `acq submit <kind> [--params '<json>'] [--priority N] [--detach]` | Generic submit. Kinds: the network kinds above plus mock-only `sleep`, `fetch`, `whoami`. |
| `acq jobs [--watch]` | Job table (live jobs only): id, parent, kind, target, state (with `↻n` retries), priority, account, submitter, ETA. `--watch` subscribes and streams state-change events. |
| `acq status <id>` | One job's state and ETA. |
| `acq result <id>` | A finished job's payload or error — answers across daemon restarts (persisted queue). |
| `acq cancel <id>` | Cancel; cascades to every descendant still waiting. |
| `acq set-priority <id> <n>` | Reorder a waiting job live. |
| `acq demo [--count N]` | Burst of fetch jobs against the mock's 5-per-10s policy; watch the ETAs. |
| `acq dash` | Live TUI: rate-limit policies (enter expands one: bucket state, observed headers, per-endpoint sends), jobs, HTTP sends, errors, rails halts in red. `--json` prints one snapshot. |

### Daemon (debugging only)

| Command | Does |
|---|---|
| `acq daemon status` | pid, version, provider, uptime, connections, queue counts, in-flight, policies learned, socket/log paths, rails state, keyring health. |
| `acq daemon stop` | Stop the daemon. |
| `acq daemon reset-tripwire` | Clear a rails halt (works on the persisted state file even with no daemon running). Observe the post-violation rule first. |
| `acq daemon run` | Run the daemon in the foreground — what lazy spawn execs. |

## 10. Environment knobs

| Variable | Effect |
|---|---|
| `ACQ_GGG=1` | Real GGG provider. **Never set without `LIVE-TESTING.md`'s standing rule.** |
| `ACQ_SOCKET=<path>` | Socket location override, for parallel *mock* testing (keep short; Unix socket paths cap ~104 bytes). Parallel daemons in real mode are forbidden — the per-IP burst gate is per-process. |
| `ACQ_STORE_DIR=<dir>` | Relocate the store (`<dir>/<provider>/<account>.db` + `accounts.json` + `daemon.db`). Default is the platform data dir (`~/Library/Application Support/gerwaric.acquisition-playground/store` on macOS). One daemon per store directory is an invariant: two daemons on one `daemon.db` would run the same queue. |
| `ACQ_ACCOUNT=<sel>` | Account selection (env form of `--account`). |
| `ACQ_NO_KEYRING=1` | Sessions become memory-only (never plaintext on disk). |
| `ACQ_NO_SPAWN=1` | CLI refuses to start/replace a daemon — for cron and other non-interactive callers (a cron-spawned daemon on macOS has no keychain, hence no session). |
| `ACQ_IDLE_SHUTDOWN=<secs>` | Idle-exit override (default 60). |
| `ACQ_JOB_RETENTION_DAYS` / `ACQ_FAILED_JOB_RETENTION_DAYS` | How long finished job rows stay in `daemon.db` (defaults 7 / 30). |
| `ACQ_TRIPWIRE=1` | Live-test rail: the first landed 429 (any route) or any 401/403/503 halts every later send until `reset-tripwire`; persisted per provider. Queued jobs wait out a halt, they don't fail. |
| `ACQ_MAX_SENDS=<n>` | Live-test rail: halt after n real sends this daemon lifetime (not persisted). |
| `ACQ_JOURNAL=<path>` | Send journal: one JSON line per actual send (method, route with account, status, all `X-Rate-Limit-*` headers; never a token or body). Defaults on, next to the socket; `0` disables. The journal is the GGG-side contract surface and the test oracle for live runs. |
| `ACQ_MOCK_DEGRADED_HEAD=1` | Mock reproduces the Dec-2023 HEAD regression, to exercise the degraded-probe path. |

Rails knobs are read by the **daemon at start** — set them on the command
that spawns it, or `acq daemon stop` first. Misread values are logged as
`CONFIG` errors and the rail stays off/default. One rail is always on:
a refresh token the provider rejects (4xx other than 429) is never
re-sent until `acq auth` or logout.

## 11. The MCP server (`acq-mcp`)

The fourth thin client: stdio MCP server (official `rmcp` SDK), sharing
the daemon and store with the CLI. Store-read tools (`accounts`,
`characters`, `tabs`, `search_items`, `get_item`, `store_status`,
`item_events` — no daemon, no network) plus job tools (`submit_job`,
`list_jobs`, `job_status`, `job_result`, `cancel_job`, `daemon_status`).
Structural rules: it never kills or replaces a daemon (a mismatch may be
a human's live GGG run — it reports and stops), lazy-spawns only in mock
mode, and **refuses `submit_job` in real-GGG mode** until GGG's stance on
agent traffic is verified. Login stays human, via `acq auth`. The MCP
tracer is the consumer currently validating the protocol shape; once it
has, the protocol gets pinned and the GUI arrives to a pinned boundary.

## 12. Current state: proven, frontier, open

Useful orientation for a design discussion.

**Proven:**
- The rate limiter's behavior — fully specified as test tables in
  `ratelimit.rs`, each row citing ground-truth claim numbers.
- The daemon's GGG boundary — live ladder closed 2026-08-27, ~1,450 real
  sends, zero 429s; per-account counting confirmed 2026-08-30 (rung 11).
  The send journal is the contract surface.
- The store's ingest — proven by replaying real snapshot data (zero
  false changes across pulls 8 h apart).
- Job persistence — proven live 2026-08-30 (persist check pass).
- Multi-account — built through step 6 of its design (session map
  included); one live sample (`/character/{name}`) still pending.

**Frontier (deliberately not pinned):**
- The daemon protocol's verb set and the store's read API. The boundary's
  *location* is a decision (two surfaces, no third door); its *content*
  waits for the MCP tracer to validate it. Known shape questions from the
  first consumer: collecting a fan-out's results is N+1 round trips
  (wants results-on-the-event-channel or a subtree `results` verb — which
  one waits for a second consumer); progress denominators grow as
  parents fan out.

**Open topics / known gaps:**
- Refresh has no delta/selection smarts — `--all` refetches every listed
  tab every time; per-tab `fetched_at` plus the API's free
  `metadata.items` counts are the obvious unused lever, and a partial
  refresh failure has no "retry just the failed set".
- User state on items (buyouts, notes) — the store has the key, no table
  yet; needs the first frontend that writes.
- Priority levels: how many, named or numeric; the intuition is
  interactive > background *regardless of frontend* (an agent in a live
  conversation is interactive; the caller states its urgency).
- Windows transport (named pipes).
- Agent/MCP traffic against real GGG — deferred until GGG's policy
  stance is verified; structurally refused in `acq-mcp` meanwhile.
- ADR 0003 (rewrite vs. evolve the C++ app) — owner's call, explicitly
  out of scope for agents; this whole workspace is a *reference
  implementation* whose code is replaceable given a reason, judged by the
  same tests and the same live ladder.

## 13. Pointers (authoritative sources)

| Topic | Where |
|---|---|
| Invariants, decisions, boundaries, open topics | `CONTEXT.md` |
| What exists, how to run it, knobs | `README.md` |
| Anything touching real GGG | `LIVE-TESTING.md` |
| How the project checks its own work; the journal as oracle | `TESTING-NOTES.md` |
| Facts about GGG's API (cited by claim number) | `../../docs/design/network-ground-truth.md` |
| Protocol verbs (live definition) | `crates/acquisition-core/src/protocol.rs` |
| Job model | `crates/acquisition-core/src/job.rs` |
| Limiter spec (test tables) | `crates/acquisition-core/src/ratelimit.rs` |
| Store schema and read API | `crates/acquisition-store/src/` |
| CLI verbs (live definition) | `acq --help`, `crates/acquisition-cli/src/main.rs` |
