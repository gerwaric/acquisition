# Rust playground

This branch is the Rust implementation of Acquisition (`CONTEXT.md`,
"Orientation"), built slice by slice, tested against a mock provider,
and used against the real one — the default since C88; a real-mode
daemon is started only from a terminal. Its purpose is to
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

One entry per crate: purpose and implementation entry point. Module
inventories and mechanisms live in the linked source documentation.

- `crates/acquisition-protocol` — the daemon contract shared by frontends.
  [Implementation](crates/acquisition-protocol/src/lib.rs);
  rulings: `decisions/daemon.md`.
- `crates/acquisition-daemon` — `acqd`, the only GGG sender: daemon,
  limiter, gate and providers.
  [Implementation](crates/acquisition-daemon/src/lib.rs);
  rulings: `decisions/daemon.md`, `decisions/network.md`.
- `crates/acquisition-client` — shared IPC client, daemon discovery and
  connection policies. [Implementation](crates/acquisition-client/src/lib.rs);
  rulings: `decisions/daemon.md`.
- `crates/acquisition-store` — shared facts, intent and world storage,
  with neutral read snapshots. [Implementation](crates/acquisition-store/src/lib.rs)
  and [world](crates/acquisition-store/src/world.rs);
  rulings: `decisions/store.md`, `decisions/daemon.md`.
- `crates/acquisition-plan` — offline refresh planning and pricing.
  [Implementation](crates/acquisition-plan/src/lib.rs);
  rulings: `decisions/plans.md`, `decisions/pricing.md`.
- `crates/acquisition-search` — item search over the store's read API;
  so far the query language, its text and its tree, and the derivation
  of an item from its stored body.
  [Implementation](crates/acquisition-search/src/lib.rs);
  rulings: `decisions/search.md`.
- `crates/acquisition-cli` — `acq`, clap and rendering.
  [Entry point](crates/acquisition-cli/src/main.rs); generated help:
  `CLI-REFERENCE.md`; rulings: `decisions/frontends.md`.
- `crates/acquisition-mcp` — `acq-mcp`, the stdio MCP frontend sharing
  semantics with the CLI through `acquisition-plan`.
  [Implementation](crates/acquisition-mcp/src/main.rs); generated tools:
  `MCP-REFERENCE.md`; rulings: `decisions/frontends.md`.

The documents are indexed in `AGENTS.md` ("Read before changing
anything"); the rulings are `CONTEXT.md` and `decisions/`.

## The tour

One line per verb. A verb's semantics — flags, defaults, the rulings it
implements — is its `--help` (`CLI-REFERENCE.md` holds every one); the
line here shows the shape.

```sh
cargo build --workspace && alias acq=./target/debug/acq   # build acq and its sibling acqd; rebuild before a run (AGENTS.md, "Quality gate")
acq <verb> --help   # every verb's reference

# a session — scripted mock login: the mock-session skill
acq auth [--no-browser]   # OAuth login
acq auth status | check | logout   # local session; token check; logout
acq accounts   # logged-in accounts (no daemon)

# jobs against the API — daemon lazy-spawns; first route use probes (C20)
acq profile | leagues   # profile; account leagues
acq characters [--realm R]   # character list
acq character <name> [--realm R]   # equipment and inventory
acq stashes [--league L] [--realm R]   # tab list (PoE1 only)
acq stash <id> [--sub <id>] [--deep]   # one tab, optionally with substashes
acq refresh --tabs a,b,c | --all [--deep]   # ad-hoc refresh of selected tabs
acq submit <kind> [--params JSON] [--detach]   # submit a job by kind
acq demo   # mock burst with limiter ETAs
acq jobs [--watch] | status <id> | result <id>   # queue; one job; finished payload
acq cancel <id> | set-priority <id> <n>   # cancel a subtree; change priority
acq dash   # live TUI dashboard

# intent, plans, apply — binding plans (C38); compilation and reads are offline
acq policy [show]   # sync policy and revision
acq policy set '<json>'|-|@FILE [--if-revision N]   # validated v3: {"version":3,"realms":{"pc":{"leagues":{…}}}}
acq refresh --plan [--realm R] [--league L] [--expand] [--json]   # compile offline; a running daemon adds its quote
acq refresh --apply[=plan.json] [--max-requests N]   # execute the plan's actions

# pricing and the shop — offline, no daemon (C64)
acq reference currency [WORD] [--expand]   # versioned currency table
acq price status | list | show <target>   # listing state; status is the default
acq price set <target> <type> [<amount> <currency>] [--if-revision N]   # set a price, with undo
acq price clear <target> [--if-revision N]   # remove a price, with undo
acq shop render [--size N] [--template FILE] [--page N] [--expand]   # forum pages to paste, omissions counted

# the store — no daemon, no network
acq tabs [--league L] [--realm R]   # tab tree and live item counts
acq store characters [--realm R] [--league L]   # character locations, ages and live items
acq items search <text> [--removed] | show <id>   # substring search; one item verbatim
acq store status | events [--hours N] | refused [id]   # counts; ingest events; malformed bodies
acq store import <snapshot.json> | rebuild   # replay a retired-pull snapshot; re-extract columns

acq daemon status | stop | reset-tripwire   # debugging: inspect, stop, reset rails
acq version [--json]   # build identity and sibling acqd
```

Every command takes `--json`, and it is total: a failure is `{"error":…}`
on stdout with exit 1 (C11). `--account <username|name|uuid>` picks the
account when several are logged in (C51).

The real provider, under the existing registration (invariant 4), is
the default; `ACQ_PROVIDER=mock` on the command that spawns the daemon
selects the mock (C88); a job command
replaces a daemon in the other mode, of another contract revision or
running another `acqd` than the one beside it (C84), an
observing verb (`jobs`, `status`, `daemon status`) reports it and never does (C10),
and mock and real refresh tokens are separate keyring entries. The rule is
`LIVE-TESTING.md`; the procedure is the live-run skill; the record is
`RUN-LEDGER.md`; the refresh loop under the rails is `tools/tracer-rung.sh`.

An MCP host pointed at `target/debug/acq-mcp` (stdio) shares the daemon
and the store with the CLI; it spends through a *running* daemon in
either mode and never spawns or replaces one in real mode (C13, C14).

## Knobs

| Knob | Default | Effect | Read in |
| --- | --- | --- | --- |
| `ACQ_PROVIDER=mock` | `ggg` | select the mock (C88); real spawning requires a terminal; retired `ACQ_GGG` is refused | [provider.rs][provider] |
| `ACQ_ACCOUNT=<sel>` | the sole account | env form of `--account`, exact match (C51) | [main.rs][cli] |
| `ACQ_STORE_DIR=<dir>` | the platform data dir | store root and daemon world (C83, C31) | [world.rs][world] |
| `ACQ_LOG_DIR=<dir>` | the platform log dir | bounded daemon log and default journal, by world and provider | [world.rs][world] |
| `ACQ_NO_KEYRING=1` | off | sessions in memory only, never plaintext on disk | [auth.rs][auth] |
| `ACQ_KEYRING_ROUNDTRIP=1` | off | opt into the test against the OS keyring (CI uses gnome-keyring) | [auth.rs][auth] (tests) |
| `ACQ_CODESIGN_IDENTITY=<sha1\|name>` | unset | macOS shell-profile identity for signing the debug daemon after building | [sign-acqd.sh](tools/sign-acqd.sh) |
| `ACQ_NO_SPAWN=1` | off | never start or replace a daemon; for cron, ssh and headless shells | [client.rs][client]; keyring context: [auth.rs][auth] |
| `ACQ_IDLE_SHUTDOWN=<s>` | 60 | idle exit, held while limiter history is live (C3) | [daemon.rs][daemon] |
| `ACQ_JOB_RETENTION_DAYS`, `ACQ_FAILED_JOB_RETENTION_DAYS` | 7, 30 | finished-job retention for `acq result`; invalid values keep defaults (`JOBS CONFIG`) | [daemon.rs][daemon] |
| `ACQ_TRIPWIRE=1` | off | rail 1: halt on 429/401/403/503 until reset, persisted | [rails.rs][rails] |
| `ACQ_MAX_SENDS=<n>` | off | rail 5: halt after `n` real sends this lifetime, not persisted | [rails.rs][rails] |
| `ACQ_JOURNAL=<path>` | `<log dir>/sends.jsonl`, bounded; `0` disables | rail 4: one JSON line per send, never a token or body | [rails.rs][rails] |
| `ACQ_MOCK_DEGRADED_HEAD=1` | off | reproduce the Dec-2023 HEAD regression (N20) | [mockggg.rs][mock] |

[provider]: crates/acquisition-protocol/src/provider.rs
[cli]: crates/acquisition-cli/src/main.rs
[world]: crates/acquisition-store/src/world.rs
[auth]: crates/acquisition-daemon/src/auth.rs
[client]: crates/acquisition-client/src/client.rs
[daemon]: crates/acquisition-daemon/src/daemon.rs
[rails]: crates/acquisition-daemon/src/rails.rs
[mock]: crates/acquisition-daemon/src/mockggg.rs

Rail semantics: `LIVE-TESTING.md`, "Rails"; journal contract:
`TESTING-NOTES.md`. Rails are read at daemon start: set them on the
spawning command, or stop the daemon first. `acq daemon status` prints
the world, paths and rails state; read-site docs explain invalid values.

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
- **Unix only.** No Windows named pipes yet; the protocol doesn't care.
- Everything parked: `CONTEXT.md` for what crosses every area, each
  `decisions/<area>.md` ("Parked") for the rest.
