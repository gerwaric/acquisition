# Rust playground

This branch is the Rust implementation of Acquisition (`CONTEXT.md`,
"Orientation"), built slice by slice against a mock provider — nothing
here talks to GGG unless a human sets `ACQ_GGG=1`. Its purpose is to
find out what the system needs to be and to pin that as tests at its
boundaries and as recorded rulings, so the code stays replaceable given
a reason. The work is judged by evidence — the offline suite, live runs
under the standing rule, and the owner's use of each slice and the
verdict it returns — and never by ceremony: a rung, a rule, a
generalization or a procedure is added when evidence asks for it and
retired when the evidence is in. The rate limiter and the single gate
are the proven foundation; their payoff is that every later caller,
human or agent, is regulated by construction and is never a fresh risk.
Whether this code ships is the owner's decision (ADR 0003) and nothing
here anticipates it.

## What exists

- `crates/acquisition-core` — the protocol and job model, the header-driven
  rate limiter and its choke point (`ratelimit.rs`: the spec is its test
  tables; `gate.rs`), the mock provider (`mockggg.rs`), the live-test
  rails (`rails.rs`), the daemon (`daemon.rs`: queue, dispatcher,
  Unix-socket server, idle watchdog) and the protocol client every
  frontend shares (`client.rs`: connect, lazy spawn, the runtime-revision
  handshake). Rulings: `decisions/daemon.md`, `decisions/network.md`.
- `crates/acquisition-store` — the shared store: SQLite, one facts file per
  account under one directory per provider, the uuid-named annotations
  file (intent — buyouts, the sync policy — the only irreplaceable local
  state) and `daemon.db` (the persisted queue). The daemon writes facts
  through `Store::record` and never reads; every frontend reads the files
  directly, through neutral snapshots. As built: `src/lib.rs`,
  `annotations.rs`, `snapshot.rs`. Rulings: `decisions/store.md`.
- `crates/acquisition-plan` — the planner (the stored policy plus a
  snapshot compiled into a `RefreshPlan`, offline, linked by frontends
  only, never the daemon) and pricing (`currency.rs`, `price.rs`,
  `game_side.rs`, `listing.rs`, `shop.rs`). As built: `src/lib.rs`.
  Rulings: `decisions/plans.md`, `decisions/pricing.md`.
- `crates/acquisition-cli` — the `acq` binary: clap and rendering, one
  module per surface (`store_cmd.rs`, `plan_cmd.rs`, `price_cmd.rs`,
  `shop_cmd.rs`, `reference_cmd.rs`, `dash.rs`). Every verb's help is
  `CLI-REFERENCE.md`, generated. Rulings: `decisions/frontends.md`.
- `crates/acquisition-mcp` — the `acq-mcp` binary: an MCP server over
  stdio (`rmcp`), the fourth thin client, sharing semantics with the CLI
  through `acquisition-plan`. Its tools are `MCP-REFERENCE.md`, generated.

The documents are indexed in `AGENTS.md` ("Read before changing
anything"); the rulings are `CONTEXT.md` and `decisions/`.

## The tour

One line per verb. A verb's semantics — flags, defaults, the rulings it
implements — is its `--help` (`CLI-REFERENCE.md` holds every one); the
line here shows the shape.

```sh
cargo build && alias acq=./target/debug/acq   # cargo test/clippy do not rebuild the binary: check acq --version
acq <verb> --help                             # the reference for every verb

# a session — the mock provider's login page accepts any username (scripted login: the mock-session skill)
acq auth [--no-browser]                       # OAuth login; completes once its own profile job lands the account uuid (C50)
acq auth status | check | logout              # local belief; a forced token round-trip; drop the session
acq accounts                                  # accounts this machine has logged into, from the store's index (no daemon)

# jobs against the API — the daemon lazy-spawns; a route's first use queues a visible `probe` (C20)
acq profile | leagues                         # GET /profile; GET /account/leagues
acq characters [--realm R]                    # the character list; pc is omitted on the wire (C58)
acq character <name> [--realm R]              # one character: equipment + inventory
acq stashes [--league L] [--realm R]          # the tab list; the stash endpoints are PoE1 only (C59)
acq stash <id> [--sub <id>] [--deep]          # one tab; --deep follows a map/unique tab's substashes as child jobs
acq refresh --tabs a,b,c | --all [--deep]     # the ad-hoc kind: list, then one `stash` child per tab (C76: one door, as direction)
acq submit <kind> [--params JSON] [--detach]  # any kind by hand; sleep, fetch and whoami are mock-only
acq demo                                      # a burst of fetch jobs against the mock's 5-per-10 s policy: watch the ETAs
acq jobs [--watch] | status <id> | result <id>   # the live queue; one job; a finished job's payload, across restarts (C27)
acq cancel <id> | set-priority <id> <n>       # cascades to waiting descendants; higher runs sooner
acq dash                                      # live TUI: limiter state, jobs, sends, errors; a rails halt in red

# intent, plans, apply — a plan is a binding envelope (C38); compiling and reading are offline
acq policy [show]                             # the per-account sync policy and its revision
acq policy set '<json>'|-|@FILE [--if-revision N]   # strict parse first; v3: {"version":3,"realms":{"pc":{"leagues":{…}}}}
acq refresh --plan [--realm R] [--league L] [--expand] [--json]   # policy + facts → the action set; sends nothing; a running daemon adds its quote
acq refresh --apply[=plan.json] [--max-requests N]   # exactly the plan's actions, as one `apply` parent (C43, C44)

# pricing and the shop — intent edited offline (C64); nothing here contacts the daemon
acq reference currency [WORD] [--expand]      # the currency table this build ships, by version (C68)
acq price status | list | show <target>       # the listing state (C69) under C53's views; status is the default
acq price set <target> <type> [<amount> <currency>] [--if-revision N]   # one row by hand; prints what it replaced and the undo
acq price clear <target> [--if-revision N]    # remove the row; prints the `set` that puts it back
acq shop render [--size N] [--template FILE] [--page N] [--expand]   # the forum pages to paste (C74); every omission counted (C72)

# the store — no daemon, no network
acq tabs [--league L] [--realm R]             # the tab tree with live item counts
acq store characters [--realm R] [--league L] # characters by id: address, league, ages, live items
acq items search <text> [--removed] | show <id>   # substring search over name/type/base; one item verbatim
acq store status | events [--hours N] | refused [id]   # row counts; what recent ingests concluded; bodies refused as malformed
acq store import <snapshot.json> | rebuild    # replay a retired-pull snapshot (no GGG traffic); re-extract derived columns

acq daemon status | stop | reset-tripwire     # debugging only (C3): paths, policies learned, the rails state; a halt's reset
acq version [--json]                          # the package version and the runtime revision the handshake compares (C10)
```

Every command takes `--json`, and it is total: a failure is `{"error":…}`
on stdout with exit 1 (C11). `--account <username|name|uuid>` picks the
account when several are logged in (C51).

`ACQ_GGG=1` on the command that spawns the daemon selects the real
provider, under the existing registration (invariant 4); a job command
replaces a daemon in the other mode or of another runtime revision, an
observing verb (`jobs`, `status`, `daemon status`) reports it and never does (C10),
and mock and real refresh tokens are separate keyring entries. The rule is
`LIVE-TESTING.md`; the procedure is the live-run skill; the refresh loop
under the rails is `tools/tracer-rung.sh`.

An MCP host pointed at `target/debug/acq-mcp` (stdio) shares the daemon
and the store with the CLI; it spends through a *running* daemon in
either mode and never spawns or replaces one in real mode (C13, C14).

## Knobs

| Knob | Default | Effect | Read in |
| --- | --- | --- | --- |
| `ACQ_GGG=1` | off | the real provider (above) | `provider.rs` |
| `ACQ_ACCOUNT=<sel>` | the sole account | env form of `--account`; exact match, never a prefix (C51) | `main.rs` |
| `ACQ_SOCKET=<path>` | `acquisition-playground.sock` in the temp dir | the socket (log and journal beside it), for parallel *mock* daemons; keep it short (Unix socket paths cap near 104 bytes); two daemons in real mode are forbidden (C31) | `daemon.rs` |
| `ACQ_STORE_DIR=<dir>` | the platform data dir | the store root: `<dir>/<provider>/<account>.db`, `accounts.json`, `daemon.db`; one daemon per store directory (C6) | `index.rs` |
| `ACQ_NO_KEYRING=1` | off | sessions in memory only, never plaintext on disk | `auth.rs` |
| `ACQ_NO_SPAWN=1` | off | the CLI never starts or replaces a daemon — for cron, which on macOS spawns without a keychain and so without a session | `client.rs` |
| `ACQ_IDLE_SHUTDOWN=<s>` | 60 | idle exit with no connections and no live jobs; a daemon holding limiter history inside a window stays up to 300 s (C3) | `daemon.rs` |
| `ACQ_JOB_RETENTION_DAYS`, `ACQ_FAILED_JOB_RETENTION_DAYS` | 7, 30 | how long finished job rows stay in `daemon.db` for `acq result`; a misread value logs `JOBS CONFIG` and keeps the default | `daemon.rs` |
| `ACQ_TRIPWIRE=1` | off | rail 1: the first landed 429, or any 401/403/503, halts every later send until `acq daemon reset-tripwire`; persisted per provider (`LIVE-TESTING.md`, "Rails") | `rails.rs` |
| `ACQ_MAX_SENDS=<n>` | off | rail 5: halt after `n` real sends this daemon lifetime; not persisted | `rails.rs` |
| `ACQ_JOURNAL=<path>` | `<socket>.<provider>.sends.jsonl`; `0` disables | rail 4: one JSON line per actual send, never a token or body — the contract surface (`TESTING-NOTES.md`; the line format is `rails.rs`) | `rails.rs` |
| `ACQ_MOCK_DEGRADED_HEAD=1` | off | the mock reproduces the Dec-2023 HEAD regression (N20) | `mockggg.rs` |

The rails are read at daemon start: set them on the command that spawns
it, or `acq daemon stop` first. A misread value (`ACQ_TRIPWIRE=maybe`) is
logged as a `RAILS CONFIG` error and the rail stays off. `acq daemon
status` prints the socket, log and journal paths and the rails state.

## Known gaps

- **Two endpoints carry declared route knowledge** (`/profile`
  policyless, N38; `/account/leagues` no-probe, N39). GGG answered Q12
  (2026-08-30): `/profile` is not rate limited at present, and the
  counted HEAD on `/account/leagues` is a defect GGG will correct —
  each declaration stays until the headers, or the free HEAD, appear.
- **The mock does not simulate timing-bucket quantization** (N11–N12); the
  limiter pads for it regardless.
- **The mock reports an active restriction on every window of the rule,**
  so the limiter picks the larger bucket after a 429. Whether real GGG
  flags only the violated window is unobserved.
- **`acq policy set` replaces the whole policy.** A poe2 run's policy
  erases the pc one (seen 2026-09-02: revision 4 carried poe2 alone),
  so the pc tabs-and-characters policy must be set again before pc
  work. A per-realm merge is unbuilt; trigger: a second realm in daily
  use (`decisions/plans.md`, "Parked").
- **A failed fetch child's result carries only the error string.** The
  body it refused is in `acq store refused <id>` (the error names the
  id), not in `acq result`; the parent's report (`acq refresh --apply`,
  `acq result <parent>`) expands each failed child's line inline.
- **Unix only.** No Windows named pipes yet; the protocol doesn't care.
- Everything parked: `CONTEXT.md` for what crosses every area, each
  `decisions/<area>.md` ("Parked") for the rest.
