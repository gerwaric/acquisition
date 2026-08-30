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
  load. Bodies are kept verbatim except at the item
  seams: each item array (tab `items`, character `inventory`/`equipment`/
  `jewels`/`rucksack`, and every `socketedItems`) is lifted into `items`,
  one row per GGG item id, so `items` is the only place to look for an
  item. Each ingest compares with what was known and writes
  `item_events` (added/moved/changed/removed; `veiledMods` ignored, N36).
  Its tests are the spec; `acq store import <snapshot>` replays a
  retired-`acq pull` snapshot through it with no GGG traffic (19,210 rows
  in ~2.3 s).
- `crates/acquisition-cli` — the `acq` binary. Thin: clap parsing, a small
  protocol client, output formatting, and `store_cmd.rs` — the reads of
  the shared store (`tabs`, `items`, `store`). `acq pull` (client-side
  snapshot + diff, 2026-08-24) is retired: the store's `item_events` answer
  the same question for every consumer. The daemon is reached via the
  hidden-ish `acq daemon run` subcommand, which is what lazy spawn execs.

## Try it

```sh
cargo build
alias acq=./target/debug/acq

acq auth                                     # OAuth login: opens a fake provider page in your browser
                                             # (the page takes any username, so two accounts are one login apart;
                                             #  scripted: curl the printed URL with /authorize?→/approve?&user=NAME)
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
acq result <id> --json                       # every command takes --json
acq daemon status                            # debugging only
```

## Real GGG mode (`ACQ_GGG=1`)

```sh
ACQ_GGG=1 acq auth          # real OAuth against pathofexile.com in your browser
ACQ_GGG=1 acq characters    # GET api.pathofexile.com/character
ACQ_GGG=1 acq stashes       # GET api.pathofexile.com/stash/Standard
```

`ACQ_GGG=1` on any command selects the real provider; the CLI kills and
respawns a daemon running in the wrong mode (the handshake carries the
provider name), so mock and real never mix on one daemon. Real mode uses the
existing "acquisition" registration — same client id, callback path
(`/auth/path-of-exile` on a random loopback port), scopes, and user-agent as
the shipped C++ app. Refresh tokens live in a keyring entry separate from the
mock's, so a mock token can never be sent to GGG. The limiter paces `Account`-scoped policies per account (rung 11: GGG
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
  by default; never on in mock mode by accident.
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

- **Refresh has no delta/selection smarts.** `--all` fetches every listed
  tab every time; the store's per-tab `fetched_at` plus the real API's
  `metadata.items` counts on substash stubs (free) are the obvious lever
  for skipping, not used yet. A refresh that loses tabs (a 503, a rails
  halt) leaves those tabs' rows at their previous state; nothing yet
  refetches only the failed set.
- **`/profile` advertises no rate-limit policy** (N38), so it is paced by
  the send gate only and called once per login; whether it is limited at
  all is asked of GGG (Q12). `/account/leagues` counts its HEAD (N39).
  Both are declared per route and the declarations go when GGG answers.
- **The mock does not simulate timing-bucket quantization** (N11–N12); the
  limiter pads for it regardless.
- **The mock reports an active restriction on every window of the rule,**
  so the limiter picks the larger bucket after a 429. Whether real GGG
  flags only the violated window is unobserved.
- **Lazy spawn hides daemon startup errors.** The spawned daemon's stderr goes
  to null, so a failed bind looks like "could not reach daemon after 5s" —
  check the daemon log.
- **Unix only.** No Windows named pipes yet; the protocol doesn't care.
- Everything in CONTEXT.md's "Explicitly deferred" list.
