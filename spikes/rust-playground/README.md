# Rust playground — daemon + CLI reference implementation

This branch is the **reference implementation** of the daemon and rate
limiter described in [CONTEXT.md](CONTEXT.md). Its purpose is to find out
what they need to be and to pin that as tests and recorded decisions; the
code is replaceable given a reason (a bug, performance, maintainability,
understandability) no matter how complete it gets, and a fully operational
CLI is still evidence, not a promotion. It may become the real
implementation, or a fresh build may replace it — judged by the same tests
and the same live ladder. The limiter's behavior is fully specified
(`ratelimit.rs` test tables, keyed to ground truth). Of the daemon's two
boundaries, the **GGG side is proven**: the live ladder closed on
2026-08-27 with every rung passed and zero 429s across ~1,450 real sends
(`LIVE-TESTING.md`), and the send journal is its contract surface. The
**frontend side is the frontier**: the shared store (`acquisition-store`)
is the first answer to what frontends need, built 2026-08-29 and proven
against real data by replay; the protocol is not yet pinned.
Tests pin behavior at those boundaries, never mechanisms.

**By default nothing
here talks to GGG**: job kinds are fakes (`sleep`, `fetch`, `whoami`),
OAuth runs against an in-process localhost provider (`mockggg.rs`), and the
mock's data endpoints sit behind truthfully simulated rate-limit policies
(real sliding windows, real restrictions, real 429s with `Retry-After`,
HEADs that report but don't count — except on `/account/leagues`, where
GGG counts them and so does the mock; `/profile` answers with no headers
and refuses HEAD, as GGG does). The rate limiter is header-driven: it
knows nothing except what responses told it (`X-Rate-Limit-*` per policy
name, plus when counted responses arrived) and pads every wait by GGG's
server-side timing bucket. Its spec is the test table at the bottom of
`ratelimit.rs`, each row citing `docs/design/network-ground-truth.md` by
claim number. Starting the daemon with `ACQ_GGG=1` opts into the real
provider (see below). All daemon-owned HTTP reaches one structural choke
point: the only `reqwest::Client` in the workspace lives inside
`ChokePoint`, and token exchange/refresh feed their responses back into the
limiter. Its common gate owns the actual-request, send-lifetime bound. A 429
re-queues the job behind the limiter's hold (shown as `↻n` in job tables),
bounded by `MAX_429_RETRIES`; a Cloudflare-shaped 403/503 is never retried.

## Layout

- [`LIVE-TESTING.md`](LIVE-TESTING.md) — control document for testing against
  the real API: the standing first-contact rule (rails on, ceiling 3, read
  the journal, record the policy), the safety rails, the closed ladder as
  history, and the run ledger. **The ladder is closed (2026-08-27)**: every
  rung passed, zero 429s across ~1,450 live sends; rung 11 (2026-08-30)
  added per-account counting. Its history (blast-radius review, rails
  build, review register) is in git at `9fa99459`.
- [`TESTING-NOTES.md`](TESTING-NOTES.md) — closed record of how this
  project checks its own work: what the rung-8 soak exposed, the journal
  as test oracle, the wire invariants, and what a rewrite keeps.
- [`NETWORK-CLEANUP.md`](NETWORK-CLEANUP.md) — closed record of the N0–N6
  network cleanup: accepted ranges, findings, and the quality-gate baseline
  every later change keeps green.
- `crates/acquisition-core` — protocol types, job model, header-driven rate
  limiter + choke point (`ratelimit.rs`, `gate.rs`), the mock provider
  (`mockggg.rs`), live-test rails (`rails.rs`), and the daemon itself
  (priority queue + dispatcher + Unix-socket server + idle watchdog). The
  gate and dispatcher properties are CONTEXT.md decisions, not restated here.
- `crates/acquisition-store` — the shared store (SQLite, one file per
  **account** under one directory per provider, named by the GGG username
  the token response reports): the daemon records every storable API
  response through one call, `Store::record(endpoint, params, status,
  body)`, and every frontend reads the file directly. `accounts.json` next
  to the files is the non-secret account index: written at login/logout,
  read by frontends to resolve `ACQ_ACCOUNT` without a daemon, and read by
  the daemon at start to know which keyring entries (one per account) to
  load. Every entry carries the account **uuid**, required at login: after
  token exchange the daemon submits a profile job (visible in `acq jobs`),
  and only when the uuid lands is the session registered, the keyring
  written, and the index updated — a login whose profile fetch fails
  **fails whole**. A rename (same uuid, new username) is a mapping update.
  `<uuid>.annotations.db` beside the fact files is the **intent layer**
  (`annotations.rs`): buyouts, notes, the sync policy — keyed on stable
  GGG ids, written only through the store crate under integer-revision
  compare-and-swap, never deleted by any fact-side event (an annotation
  whose item is gone is kept and surfaceable as orphaned; a frontend
  delete is a tombstone under the same compare-and-swap, so revisions
  never reset across delete/recreate), backed up via store-managed
  `VACUUM INTO` export. The only irreplaceable local state;
  the store crate's production code is held to no-panic by a clippy
  ratchet (`unwrap_used`/`expect_used` denied).
  `Store::stash_snapshot` (`snapshot.rs`) is the planner's read, taken
  in one read transaction and bound to the account uuid the facts file
  records: the annotations file carries its owner's uuid internally
  (`Annotations::open_for` stamps and verifies it), so a copied or
  renamed file keeps its owner and a mismatched or unbound handle is
  refused. The snapshot is one league's listing basis (the `responses`
  row a plan cites — tab membership is stamped with that id, so two
  listings in one second cannot disagree), tab identities with
  freshness and the listing's metadata verbatim (kept in its own
  column; a fetch never overwrites it), and the sync-policy annotation
  row at its revision — facts and intent named together, never a
  staleness verdict; compiling them into requests is
  `acquisition-plan`'s job (tracer step 4, built 2026-09-01).
  A 2xx body missing its array/object or carrying an identity-less
  entry (a tab or item without `id`, a character without `name`) is a
  typed `MalformedBody` refusal that writes nothing — and it fails the
  job: the daemon's `record` classifies the store's verdict, so a
  malformed response is `Outcome::Failure` while genuine persistence
  trouble stays logged-and-absorbed. `acq store import` keeps the
  legacy tolerance at its own boundary (id-less snapshot items are
  skipped and counted, never ingested silently). Both store files carry
  schema versions: a newer file is refused, and migrations run
  serialized so two openers cannot interleave them.
  Bodies are kept verbatim except at the item
  seams: each item array (tab `items`, character `inventory`/`equipment`/
  `jewels`/`rucksack`, and every `socketedItems`) is lifted into `items`,
  one row per GGG item id, so `items` is the only place to look for an
  item. Each ingest compares with what was known and writes
  `item_events` (added/moved/changed/removed; `veiledMods` ignored, N36).
  Its tests are the spec; `acq store import <snapshot>` replays a
  retired-`acq pull` snapshot through it with no GGG traffic (19,210 rows
  in ~2.3 s). `daemon.db` in the same directory is the **persisted job
  queue** (`jobs.rs`): the daemon mirrors every job there at each state
  change and takes the open ones back when it starts, so the queue
  survives an idle exit, `daemon stop`, a version respawn, or a crash.
  A job that was running is re-queued (idempotent GETs; the restart
  probe reads GGG's counters first) — except on no-probe routes, where
  it fails as interrupted, and a parent restarted mid-fan-out, which
  holds for the children it already has and then finishes as interrupted
  (the full child set is unknown, so success is never claimed; its own
  payload is lost) — probes are dropped, ids continue. A queue write
  failure at runtime is sticky: the daemon refuses new jobs and stops
  dispatching (running jobs finish) until a restart finds a working
  `daemon.db`; a queue it cannot read at start is fatal. Finished rows
  stay for `acq result <id>` across restarts, pruned by age at start.
- `crates/acquisition-plan` — the planner (tracer step 4): policy
  compilation and `RefreshPlan` construction, linked by frontends only,
  never the daemon — "the daemon never reads the store" is enforced by
  the dependency graph. `plan_refresh(provider, &snapshot, now)` parses
  the snapshot's sync-policy row (the planner owns that value's schema:
  version-stamped and strict-parsed — a typo'd field is a structured
  error, never intent half-honored; a newer version is refused as such)
  and compiles it, with the daemon down, into a serializable
  `RefreshPlan`: the explicit action set (re-list and/or per-tab
  fetches, each carrying its reason — never fetched, stale, or a
  count disagreement), covered-but-skipped tabs with reasons, policy
  ids the facts lack reported rather than invented into actions, the
  basis it cites (listing response id, policy revision, account uuid,
  snapshot time), exact `logical_requests`, and a coarse `wire_sends`
  range with named prerequisites (probe, OAuth refresh) — never a
  precise wire accounting. Plans always derive from the stored policy
  row and carry its revision (no ad-hoc path), and a serialized plan
  re-validates on parse — unknown fields at any depth, a newer schema
  stamp, a wrong operation, an action outside the envelope's league, or
  derived quantities that do not recompute (the wire projection,
  prerequisites included) are refused whole — so apply can trust what
  it reads back. Plans are binding and act only on facts on
  record: a never-listed league plans the listing alone (covered tabs
  are reported as awaiting the listing — without a basis the plan has
  no membership authority), substash
  fetches come only from stubs already in the store (no dynamic deep
  fan-out; one whose recorded parent has been retired is skipped with
  its reason, never fetched by a guessed path), and newly discovered
  tabs wait for the next plan. A listed
  `metadata.items` count forces a fetch when a listing newer than our
  fetch disagrees with what the store holds; it never skips one. Each
  action renders the daemon's own `(kind, params)` job tuple
  (`RefreshAction::job`), pinned by decoding through
  `Endpoint::from_job` — the store's production decoder of the job
  vocabulary — and by a plan→record→replan loop that proves applied
  actions satisfy the plan. A plan may carry the daemon's **quote** as
  optional enrichment (tracer step 5, built 2026-09-01): `quote` is its
  own protocol request — a read-only, non-reserving projection of the
  work's `(kind, params)` tuples over current limiter state, per
  scheduling scope with per-window headroom (stamped with its own
  observation age, read under one limiter lock with the ETA), the queue
  counted ahead, and a forward-simulated ETA that is an estimate, never
  a promise — seeded (as a count, safe against absurd headers) with
  server-reported hits the local history never saw, so it over-waits
  rather than floods; unlearned routes, probes, OAuth refresh, 429
  re-sends, and a rails halt are named rather than silently omitted.
  The quote echoes the job tuples it priced (`work`), and attaching one
  (`with_quote`) validates it speaks about the plan's own provider,
  exactly its account, and exactly its actions in order — never just a
  matching count; carrying it bumped the plan schema (v3; the `empty_stub`
  skip kind made v4 on 2026-09-01). Same
  no-panic clippy ratchet as the store crate.
- `crates/acquisition-cli` — the `acq` binary. Thin: clap parsing, output
  formatting, `store_cmd.rs` — the reads of the shared store (`tabs`,
  `items`, `store`) — and `plan_cmd.rs` — the intent surface (`acq policy`,
  writing the sync-policy annotation through the store crate) and
  `acq refresh --plan` (tracer step 6): compiles the stored policy via
  `acquisition-plan` into a `RefreshPlan`, offline, spending nothing; a
  *running* daemon enriches it with its quote (never spawned for this),
  and `--json` emits the plan envelope verbatim. `acq refresh --apply`
  (tracer step 7) executes a plan — compiled fresh, or a reviewed
  envelope from `--plan --json` re-validated by the planner's parse:
  the staleness gate (the stored policy revision must still be the
  plan's — CONTEXT.md's step-7 ruling) runs offline before any daemon
  contact, an empty plan applies as a no-op with no daemon at all, and
  the actions go out as one `apply` parent job — a pure fan-out the
  daemon admits or refuses whole at submit (single-request vocabulary
  only, plus the `--max-requests` logical budget), executing exactly
  the reviewed tuples and never expanding them; newly discovered tabs
  wait for the next plan, and the plan→apply→replan loop is pinned at
  process level (`tests/apply_loop.rs`) to close in a bootstrap
  listing plus two reconciliation cycles. The protocol client (connect, lazy spawn, version
  handshake) lives in `acquisition-core/src/client.rs` and is shared by
  every frontend; the frontends differ only in connect *policy*
  (`ConnectOptions`). `acq pull` (client-side snapshot + diff, 2026-08-24)
  is retired: the store's `item_events` answer the same question for every
  consumer. The daemon is reached via the hidden-ish `acq daemon run`
  subcommand, which is what lazy spawn execs.
- `crates/acquisition-mcp` — the `acq-mcp` binary: an MCP server over
  stdio (official `rmcp` SDK), the fourth thin client. Store-read tools
  (`accounts`, `characters`, `tabs`, `search_items`, `get_item`,
  `store_status`, `item_events` — no daemon, no network), job tools (`submit_job`,
  `list_jobs`, `job_status`, `job_result`, `cancel_job`, `daemon_status`),
  and the plan slice (tracer step 8): `sync_policy` / `set_sync_policy`
  (the intent annotation — local, sends nothing, allowed in either mode;
  replacing an existing policy must name the revision it replaces, so an
  agent never clobbers intent it has not read), `refresh_plan` (the
  offline compile, quote-enriched by a *running* daemon in either mode,
  never spawned for it),
  and `apply_plan` (the reviewed envelope as one `apply` parent, the
  staleness gate run before any daemon contact; returns the job id to
  poll). The slice shares its semantics with the CLI through
  `acquisition-plan` (validate-then-CAS policy writes, the validating
  parse, `check_spendable`, `apply_params`), and the whole loop is
  pinned at process level in `tests/plan_loop.rs` (offline claims proven
  with the daemon stopped) and `tests/ggg_refusal.rs` (real mode never
  spawns a daemon, proven with none present).
  It **never kills or replaces a daemon** (a mismatch may be a human's
  live GGG run — it reports and stops) and lazy-spawns only in mock mode:
  a real-GGG daemon is a human's act. Agent traffic through the daemon is
  allowed in either mode (owner ruling 2026-09-01, CONTEXT.md) — the
  daemon is the single gate, and every client is paced by the same code.
  Login is human, via `acq auth`.

## Try it

```sh
cargo build
alias acq=./target/debug/acq

acq auth                                     # OAuth login: opens a fake provider page in your browser
                                             # (the page takes any username, so two accounts are one login apart;
                                             #  scripted: curl the printed URL with /authorize?→/approve?&user=NAME;
                                             #  login completes only once its own profile job lands the account uuid)
acq auth status                              # session, token expiry, keyring health (local belief)
acq auth check                               # preflight: proves the session via a forced token round-trip
acq submit whoami                            # mock-only auth job; refreshes the access token silently
acq profile                                  # GET /profile (account:profile)
acq characters                               # auth-required; GET /character against the mock
                                             # (first use of a route queues a visible `probe` job:
                                             #  one HEAD that learns the policy + current counters)
acq stashes --league Standard                # GET /stash/{league}: a second policy, runs in parallel
acq character <name>                         # GET /character/{name}: equipment + inventory
acq leagues                                  # GET /account/leagues (account:leagues)
acq stash <id> [--sub <id>] [--deep]         # one tab; --deep follows a map/unique tab's substashes as child jobs
acq refresh --tabs a,b,c | --all [--deep]    # list, then one `stash` child per tab; parent finishes last
acq policy [show]                            # the per-account sync policy: declared coverage + freshness (an annotation)
acq policy set '<json>' [--if-revision N]    # validated through the planner's strict parse before anything lands;
                                             #  `-` reads stdin, `@file` reads a file; --if-revision writes only
                                             #  over exactly the revision you reviewed (without it, the currently
                                             #  stored revision is replaced; racing writes conflict, never clobber)
acq refresh --plan                           # compile policy + facts into the explicit action set — sends nothing;
                                             #  a running daemon adds its read-only quote (never spawned for this),
                                             #  and --json prints the serialized plan envelope itself
acq refresh --apply[=plan.json]              # execute the plan: exactly its actions, as one `apply` parent job
                                             #  (bare --apply compiles the stored policy now; =FILE applies a
                                             #  reviewed envelope, =- reads stdin). Refused before any daemon
                                             #  contact if the stored policy revision moved since the plan;
                                             #  --max-requests N makes the daemon refuse at admission if the
                                             #  plan authorizes more, before any child job exists
acq cancel <parent-id>                       # cascades to every descendant still waiting
acq accounts                                 # accounts this machine has logged into, from the store's index (no daemon)
acq tabs [--league L]                        # from the shared store: tab tree with live item counts (no daemon)
acq items search <text> [--removed]          # substring search over name/type/base; socketed gems are rows too
acq items show <id>                          # one item, verbatim
acq store status | events [--hours N]        # row counts; what recent ingests concluded
acq store import <snapshot.json> | rebuild   # replay a retired-pull snapshot (no GGG traffic); re-extract columns
acq auth logout                              # drops session + keyring entry
acq submit sleep --params '{"seconds": 5}'   # blocks with progress; daemon lazy-spawns
acq demo                                     # burst of 8 fetch jobs against the mock's 5-per-10s policy; watch ETAs
acq submit fetch --detach                    # job mode: returns id immediately
acq jobs                                     # job table with targets (from params) and ETAs
acq jobs --watch                             # subscribe to job-state-changed events
acq dash                                     # live TUI: rate limits, jobs, HTTP sends, errors
                                             # (enter expands a rate-limit policy: bucket state,
                                             #  observed X-Rate-Limit headers, per-endpoint sends)
acq set-priority <id> 5                      # higher runs sooner; queue reorders live
acq cancel <id>
acq result <id> --json                       # every command takes --json; answers across daemon restarts
                                             # (--json is total: errors are {"error":…} on stdout, and a
                                             #  failed job exits 1 in both output modes; `acq auth
                                             #  --no-browser --json` prints {"authorize_url":…} first)
acq daemon status                            # debugging only
```

To use the MCP server, point an MCP host at `target/debug/acq-mcp`
(stdio); it shares the daemon and store with the CLI. Mock-only for
job submission — see the layout note above.

## Real GGG mode (`ACQ_GGG=1`)

```sh
ACQ_GGG=1 acq auth          # real OAuth against pathofexile.com in your browser
                            # (also sends one GET /profile — the login's own
                            #  profile job; uuid-at-login, N38: not rate limited)
ACQ_GGG=1 acq characters    # GET api.pathofexile.com/character
ACQ_GGG=1 acq stashes       # GET api.pathofexile.com/stash/Standard
```

`ACQ_GGG=1` on any command selects the real provider; the CLI kills and
respawns a daemon running in the wrong mode (the handshake carries the
provider name), so mock and real never mix on one daemon. Real mode uses the
existing "acquisition" registration — same client id, callback path
(`/auth/path-of-exile` on a random loopback port), scopes, and user-agent as
the shipped C++ app. Refresh tokens live in a keyring entry separate from the
mock's, so a mock token can never be sent to GGG. Trap: the debug binary
is unsigned, so macOS Keychain treats every rebuild as a new program and
asks for the login-keychain password on first access — typically twice
per login (the stored session read, then the rotated token saved). A
known cost of debug builds, not a stop condition; a signed build would
ask once. The limiter paces `Account`-scoped policies per account (rung 11: GGG
counts them per account) and `Ip`-scoped ones — the token endpoint — as
one shared counter; `acq dash` shows the state keys
(`stash-request-limit@Alice#1234`, `token-request-limit`). The limiter is
the same code in both modes and starts empty: the first job on an endpoint queues a
`probe` (a HEAD, which GGG doesn't count on the probed routes) that
teaches the policy and the account's current counters — including hits
made by other tools — before anything real is sent. Two routes are not
probed, by declared knowledge (`declare_route_knowledge` in `daemon.rs`):
`/profile` (HEAD 403, no rate headers; paced by the send gate alone) and
`/account/leagues` (HEAD counted); their first GET teaches the limiter. A probe that fails or comes back without rule
definitions closes the endpoint for 60s (login reopens it). Nothing retries
on a Cloudflare-shaped 403/503. A 429 re-queues behind the policy hold and
retries at most `MAX_429_RETRIES` times; after that it fails with the
accumulated evidence. The limiter holds the policy until `Retry-After` plus
the timing bucket.

## Lifecycle & knobs

The daemon exits on its own after 60s with no connections and no live jobs —
unless the limiter still holds history inside a policy window (up to 300s),
in which case it stays up so a quick respawn doesn't have to assume the worst
about hits it can no longer see.
Its log is next to the socket (`acq daemon status` prints both paths).
A daemon that refuses to start (a broken `daemon.db`, a failed bind) writes
the refusal to that log, and a lazy spawn that dies surfaces it: the CLI
notices the exited daemon and prints the log's new lines instead of timing
out.
`ACQ_SOCKET=<path>` overrides the socket location for parallel testing — keep
it short (Unix socket paths cap out around 104 bytes). Parallel daemons are
for the mock: two daemons in real mode on one machine each hold their own
Cloudflare gate and tripwire, so the IP can have four sends in flight that
neither sees (P-B) — multiple accounts are sessions in one daemon, never
one daemon each. `ACQ_NO_KEYRING=1`
degrades sessions to in-memory only (never plaintext on disk). Mock access
tokens live 60 seconds, so silent refresh is exercised constantly.
`ACQ_IDLE_SHUTDOWN=<secs>` overrides the idle exit. `ACQ_STORE_DIR=<dir>`
relocates the shared store (`<dir>/<provider>/<account>.db` plus
`accounts.json`; default is the platform data directory via the `directories`
crate — `~/.local/share/acquisition-playground/store` on Linux,
`~/Library/Application Support/gerwaric.acquisition-playground/store` on
macOS, `%APPDATA%\gerwaric\acquisition-playground\data\store` on Windows).
`ACQ_JOB_RETENTION_DAYS` (default 7) and `ACQ_FAILED_JOB_RETENTION_DAYS`
(default 30) say how long finished job rows stay in `daemon.db`; a
misread value is logged as a `JOBS CONFIG` error and the default stays.
`--account <username|name|uuid>` (global; `ACQ_ACCOUNT` is the env form)
names the account a command acts as. The daemon holds **one session per
logged-in account** (log in again as someone else and both stay live;
`acq auth status` lists them) and never picks one for you: with one
session the selector is implicit, with several every job command and
store read refuses and lists the choices. The account is fixed on the
job at submit — it runs with that account's token and its response lands
in that account's file — and is checked again when the token is taken,
so a job whose account was logged out meanwhile fails without a send
("no session for A"). `acq auth logout [--account A]` drops one session;
a logged-out account stays listed (its store remains) as "not
persisted". The rate limiter paces `Account`-scoped policies per account
and the `Ip`-scoped token endpoint once for all (`acq dash`). `ACQ_NO_SPAWN=1` makes
the CLI refuse to start or replace a daemon — for cron and other
non-interactive callers, which on macOS would spawn a daemon with no
keychain access and therefore no session.

Live-test rails (`LIVE-TESTING.md`), read by the daemon at start — set them
on the command that spawns it, or `acq daemon stop` first:

- `ACQ_TRIPWIRE=1` — the first landed 429 (any route, HEAD and token
  included) or any 401/403/503 halts every later send until
  `acq daemon reset-tripwire`; persisted per provider across restarts. Off
  by default; never on in mock mode by accident. Queued jobs wait out a
  halt (they are on disk); a halted daemon with nothing running idles out
  and its successor holds the queue until the reset.
- Always on, not a knob: a refresh token the provider rejects (4xx other
  than 429) is never re-sent until `acq auth` or logout; the mark persists
  across restarts.
- `ACQ_MAX_SENDS=<n>` — halt after `n` real sends this daemon lifetime
  (not persisted).
- `ACQ_JOURNAL=<path>` — one JSON line per actual send (method, route,
  status, every `X-Rate-Limit-*` header; never a token or body), flushed
  per line; defaults to `<socket>.<provider>.sends.jsonl`, `0` disables.
  A 403/503 line also carries `shape`: `cloudflare` (N3/N28 page markers),
  `origin` (an openresty/nginx error page that passed through Cloudflare —
  rung 10's 503, N35), or `unclassified`. All three are equally never retried.
  Each daemon lifetime opens with `{"event":"open","pid","build","clock"}`
  — the git commit the binary was built from and whether time was the
  system's or a test's manual clock.
  `route` is the limiter's endpoint key — `stash@Alice#1234` for a send
  on an account, `oauth-token` for the account-blind token endpoint — so
  the journal names the account of every send.
  A journal that cannot be opened is reported in `daemon status`, not
  silently dropped.
  The path's directory is created on demand. A non-2xx response adds a
  `headers` object (the `X-Rate-Limit-*`, `Retry-After`, `cf-*`,
  `content-type`, `server`, `date` headers) — for a failed HEAD probe
  that is the whole of the evidence.
- Misunderstood values (`ACQ_TRIPWIRE=maybe`, `ACQ_MAX_SENDS=ten`) are
  logged at startup as `RAILS CONFIG` errors; the rail stays off. A
  persisted trip is honored only by a daemon started with the tripwire.
- The HTTP client has a 10 s connect and 60 s request timeout; a send lost
  in transport, or answered without rate headers (an origin 503), is paced
  as if the server counted it.

`acq daemon status` prints the rails state; `acq dash` shows a halt in red.

`ACQ_MOCK_DEGRADED_HEAD=1` makes the mock reproduce the Dec-2023 HEAD
regression (N20) so the degraded path can be exercised.

## Known gaps

- **Two endpoints carry declared route knowledge** (`/profile`
  policyless, `/account/leagues` no-probe). GGG answered Q12
  (2026-08-30): `/profile` is not rate limited at present — its
  declaration is confirmed and stays until headers ever appear (N38);
  the counted HEAD on `/account/leagues` is a defect GGG will correct
  in a future release — treat it as counted until the free HEAD is
  observed, then the declaration goes and the probe returns (N39).
- **The mock does not simulate timing-bucket quantization** (N11–N12); the
  limiter pads for it regardless.
- **The mock reports an active restriction on every window of the rule,**
  so the limiter picks the larger bucket after a 429. Whether real GGG
  flags only the violated window is unobserved.
- **Unix only.** No Windows named pipes yet; the protocol doesn't care.
- Everything in CONTEXT.md's "Explicitly deferred" list.
