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
- `crates/acquisition-store` — the shared store: SQLite, one facts file
  per **account** under one directory per provider, plus
  `<uuid>.annotations.db` (the intent layer — buyouts, notes, the sync
  policy; the only irreplaceable local state) and `daemon.db` (the
  persisted job queue, `jobs.rs`). The daemon writes facts through one
  call, `Store::record(endpoint, params, status, body)`, and never
  reads; every frontend reads the files directly. How ingest works as
  built — item lifting at the seams, listing-owned membership and
  liveness, withheld and refused bodies, `item_events`, schema versions
  and migrations, the no-panic lint ratchet — is the crate's module doc
  (`src/lib.rs`, "As built") and its tests; the boundary properties are
  `CONTEXT.md` decisions ("Bodies are stored verbatim…", "A refused body
  is evidence…", "Annotations are the only irreplaceable local state").
- `crates/acquisition-plan` — the planner: compiles the stored sync
  policy plus a store snapshot into a `RefreshPlan`, offline, linked by
  frontends only, never the daemon ("the daemon never reads the store"
  is enforced by the dependency graph). The policy shape (v3), what a
  plan contains, the strict re-serializing parse, quote enrichment and
  the schema history are the crate's module doc (`src/lib.rs`, "As
  built"); the boundary properties are the Plan, quote and apply
  decisions in `CONTEXT.md`. Same no-panic lint ratchet as the store.
- `crates/acquisition-cli` — the `acq` binary. Thin: clap parsing, output
  rendering, `store_cmd.rs` (reads of the shared store, no daemon) and
  `plan_cmd.rs` (the intent surface `acq policy`, and `acq refresh
  --plan|--apply` through `acquisition-plan`). The protocol client
  (connect, lazy spawn, version handshake) is
  `acquisition-core/src/client.rs`, shared by every frontend; frontends
  differ only in connect *policy* (`ConnectOptions`). The daemon is
  reached via `acq daemon run`, which is what lazy spawn execs. The
  loop is pinned at process level in `tests/apply_loop.rs`,
  `tests/plan_json.rs`, `tests/characters_wire.rs`, `tests/realm_wire.rs`.
- `crates/acquisition-mcp` — the `acq-mcp` binary: an MCP server over
  stdio (official `rmcp` SDK), the fourth thin client. Store-read tools
  (no daemon, no network), job tools, and the plan slice (`sync_policy`,
  `set_sync_policy`, `refresh_plan`, `apply_plan`), sharing semantics
  with the CLI through `acquisition-plan`. Its rules are `CONTEXT.md`
  decisions: it never kills or replaces a daemon, lazy-spawns only in
  mock mode, and spends through a *running* daemon in either mode;
  login is human, via `acq auth`. Pinned in `tests/plan_loop.rs` and
  `tests/ggg_refusal.rs`.

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
acq characters [--realm poe2]                # auth-required; GET /character[/{realm}] against the mock
                                             # (first use of a route queues a visible `probe` job:
                                             #  one HEAD that learns the policy + current counters;
                                             #  --realm: pc by default and omitted on the wire, else
                                             #  xbox|sony|poe2 as a segment — and its own probe)
acq stashes --league Standard [--realm xbox] # GET /stash[/{realm}]/{league}: a second policy, runs in parallel
                                             # (stashes are PoE1 only: --realm poe2 is refused at admission)
acq character <name> [--realm R]             # GET /character[/{realm}]/{name}: equipment + inventory
acq leagues                                  # GET /account/leagues (account:leagues)
acq stash <id> [--sub <id>] [--deep]         # one tab; --deep follows a map/unique tab's substashes as child jobs
acq policy [show]                            # the per-account sync policy: declared coverage + freshness (an annotation)
acq policy set '<json>' [--if-revision N]    # validated through the planner's strict parse before anything lands;
                                             #  v3 shape: {"version":3,"realms":{"pc":{"leagues":{"Standard":
                                             #  {"tabs":"all","characters":"all","max_age_seconds":3600}}}}} —
                                             #  per league, `tabs` and/or `characters` ("all" or ids; absent =
                                             #  no coverage of that facet; neither = refused); `tabs` is refused
                                             #  under poe2, `characters` taken everywhere (a v1 `leagues` value
                                             #  still parses, as realm pc; v1/v2 as tab coverage only);
                                             #  `-` reads stdin, `@file` reads a file; --if-revision writes only
                                             #  over exactly the revision you reviewed (without it, the currently
                                             #  stored revision is replaced; racing writes conflict, never clobber)
acq refresh --plan [--realm R] [--league L]  # compile policy + facts into the explicit action set — sends nothing;
                                             #  a running daemon adds its read-only quote (never spawned for this),
                                             #  and --json prints the serialized plan envelope itself
                                             #  (--realm poe2 plans a character-only entry). The text groups the
                                             #  actions by kind and parent and counts by reason (a group of more
                                             #  than ten is counted, never listed); --expand is one line per
                                             #  action; --plan=FILE renders a reviewed envelope the same way
acq refresh --apply[=plan.json]              # execute the plan: exactly its actions, as one `apply` parent job
                                             #  (bare --apply compiles the stored policy now; =FILE applies a
                                             #  reviewed envelope, =- reads stdin). Refused before any daemon
                                             #  contact if the stored policy revision moved since the plan;
                                             #  --max-requests N makes the daemon refuse at admission if the
                                             #  plan authorizes more, before any child job exists
acq refresh --tabs a,b,c | --all [--deep]    # the ad-hoc kind: list, then one `stash` child per tab, no plan
                                             #  (an open topic in CONTEXT.md: two doors to one task)
acq cancel <parent-id>                       # cascades to every descendant still waiting
acq accounts                                 # accounts this machine has logged into, from the store's index (no daemon)
acq tabs [--league L] [--realm R]            # from the shared store: tab tree with live item counts (no daemon)
acq store characters [--realm R] [--league L]  # from the shared store: characters by id — address, league,
                                             #  listed/fetched age, live item count (no daemon)
acq items search <text> [--removed] [--realm R]  # substring search over name/type/base; socketed gems are rows too
acq items show <id>                          # one item, verbatim
acq store status | events [--hours N]        # row counts; what recent ingests concluded — events as one line per
                                             #  location with counts (text) or the event list (--json);
                                             #  --expand / --summary pick either form in either mode
acq store refused [id]                       # bodies the store refused as malformed, kept verbatim: the list, or one in full
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

Reading the plan as an agent: the text is a function of the envelope, so
count with `jq` rather than parsing prose —
`jq '.logical_requests'`, `jq '[.actions[] | .action] | group_by(.) | map({(.[0]): length}) | add'`
(requests by kind), `jq '[.actions[] | select(.action == "fetch_substash") | .parent] | group_by(.) | map({(.[0]): length}) | add'`
(substashes per parent), `jq '[.skipped_tabs[], .skipped_characters[] | .reason.kind] | group_by(.) | map({(.[0]): length}) | add'`
(skips by reason). `acq refresh --apply --json` adds `store_changes`
beside the outcome; `acq store events --summary --json` is the
per-location summary.

To use the MCP server, point an MCP host at `target/debug/acq-mcp`
(stdio); it shares the daemon and store with the CLI. It submits, applies
and quotes in either mode against a *running* daemon (agent-traffic
ruling, 2026-09-01) and never spawns or replaces one in real mode —
see the layout note above.

## Real GGG mode (`ACQ_GGG=1`)

```sh
ACQ_GGG=1 acq auth          # real OAuth against pathofexile.com in your browser
                            # (also sends one GET /profile — the login's own
                            #  profile job; uuid-at-login, N38: not rate limited)
ACQ_GGG=1 acq characters    # GET api.pathofexile.com/character
ACQ_GGG=1 acq stashes       # GET api.pathofexile.com/stash/Standard
ACQ_GGG=1 acq characters --realm poe2   # GET …/character/poe2 (first contact 2026-09-02: N41–N44)
tools/tracer-rung.sh --account A --characters all <tab ids>   # the refresh loop under the rails,
                                        # journal verified — see the live-run procedure
```

`ACQ_GGG=1` on any command selects the real provider; the CLI kills and
respawns a daemon running in the wrong mode or from another build (the
handshake carries the provider name and the build stamp `acq --version`
prints), so mock and real never mix on one daemon and a stale daemon
never serves a newer client's jobs. Real mode uses the
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
  the journal names the account of every send. A realm other than pc
  suffixes the route (`stash-list/xbox@Alice#1234`): each realm's URL
  shape gets its own free HEAD probe, and whether it shares the pc
  policy is learned from its headers; pc routes are unchanged.
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
- **`acq policy set` replaces the whole policy.** A poe2 run's policy
  erases the pc one (seen 2026-09-02: revision 4 carried poe2 alone),
  so the pc tabs-and-characters policy must be set again before pc
  work. A per-realm merge is unbuilt; trigger: a second realm in daily
  use (CONTEXT.md parking lot).
- **A failed fetch child's result carries only the error string.** The
  body it refused is in `acq store refused <id>` (the error names the
  id), not in `acq result`; the parent's report (`acq refresh --apply`,
  `acq result <parent>`) expands each failed child's line inline.
- **Unix only.** No Windows named pipes yet; the protocol doesn't care.
- Everything in CONTEXT.md's "Explicitly deferred" list.
