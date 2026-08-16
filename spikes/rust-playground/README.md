# Rust playground — daemon + CLI spike

Throwaway code for the daemon/CLI architecture in [CONTEXT.md](CONTEXT.md).
**Nothing here talks to GGG.** Job kinds are fakes (`sleep`, `fetch`) and the
rate limiter is a simulated token bucket (burst of 5, one token per 3s), so we
can exercise queueing, ETAs, cancellation, and long-wait ergonomics with zero
risk to the OAuth registration.

## Layout

- `crates/acquisition-core` — protocol types, job model, token bucket, and the
  daemon itself (queue + single worker + Unix-socket server + idle watchdog).
- `crates/acquisition-cli` — the `acq` binary. Thin: clap parsing, a small
  protocol client, output formatting. The daemon is reached via the hidden-ish
  `acq daemon run` subcommand, which is what lazy spawn execs.

## Try it

```sh
cargo build
alias acq=./target/debug/acq

acq auth                                     # OAuth login: opens a fake provider page in your browser
acq auth status                              # session, token expiry, keyring health (local belief)
acq auth check                               # preflight: proves the session via a forced token round-trip
acq submit profile                           # auth-required job; refreshes the access token silently
acq auth logout                              # drops session + keyring entry
acq submit sleep --params '{"seconds": 5}'   # blocks with progress; daemon lazy-spawns
acq demo                                     # burst of 8 fetch jobs; watch ETAs count down
acq submit fetch --detach                    # job mode: returns id immediately
acq jobs                                     # job table with ETAs
acq jobs --watch                             # subscribe to job-state-changed events
acq set-priority <id> 5                      # higher runs sooner; queue reorders live
acq cancel <id>
acq result <id> --json                       # every command takes --json
acq daemon status                            # debugging only
```

The daemon exits on its own after 60s with no connections and no live jobs.
Its log is next to the socket (`acq daemon status` prints both paths).
`ACQ_SOCKET=<path>` overrides the socket location (useful for parallel testing).

## What this spike exercises (from CONTEXT.md decisions)

- Cargo workspace, library-centric core; CLI is a thin frontend.
- Jobs, not calls: submit/status/result/cancel/set_priority/list/subscribe.
- Priority queue (one `u8` field; higher = sooner, FIFO within a level).
- JSON lines over a Unix socket; subscribe/event channel on the same socket.
- Version handshake: client kills + respawns a version-mismatched daemon
  (both binaries share `acquisition_core::VERSION`).
- Lazy spawn + idle auto-shutdown (gpg-agent model).
- ETA prediction from token-bucket state + queue depth ahead of the job.
- Blocking-with-progress default (`rate limited, starting in ~Ns...`),
  `--detach` for job mode.
- OAuth with real mechanics against a fake endpoint: authorization code +
  PKCE (S256), loopback redirect listener, token exchange/refresh via reqwest,
  refresh-token rotation, and refresh tokens in the OS keyring (invariant 5).
  The daemon owns the entire flow and all tokens; clients never see them.
  The provider is an in-process localhost stub (`mockggg.rs`) — pointing at
  real GGG endpoints later is a config change, not a redesign. Sessions
  survive daemon restarts via the keyring (access token re-derived by
  refresh); access tokens live 60s so refresh is exercised constantly.
  `ACQ_NO_KEYRING=1` degrades to in-memory sessions (never plaintext on disk).

## Not built (deliberately)

- Anything that touches GGG. OAuth runs against the localhost mock provider;
  the token bucket is a stand-in for the header-driven policy layer described
  in CONTEXT.md. No code path in this workspace reaches a non-loopback host.
- Windows named pipes (`UnixListener` only; the protocol doesn't care).
- Job persistence, result caching, MCP — all deferred per CONTEXT.md.

## Lessons learned (feed these back into CONTEXT.md when they firm up)

- **Unix socket paths have a ~104-byte limit** (`SUN_LEN`). Deep per-user
  runtime dirs can exceed it; the daemon needs a short, stable socket path and
  a clear error when an override is too long.
- **Lazy spawn eats daemon startup errors.** The spawned daemon's stderr goes
  to null, so a daemon that fails to bind looks like "could not reach daemon
  after 5s". The client should surface the daemon log tail on connect timeout.
- **Head-of-line blocking is a real design point.** The single worker waits on
  the rate limiter *while holding the queue head*, so it must re-check the
  queue head between wait slices or a reprioritized job can't jump a
  token-starved one. Slice-based waiting (1s) made cancel + reorder easy;
  a smarter design might want the wait to be interruptible instead.
- **One connection interleaves responses and events once subscribed.** Client
  code must dispatch on response type, not assume request/reply lockstep.
  Fine at this scale; worth remembering when the GUI client grows.
- **Convenience logging + coarse lock = startup deadlock.** `Daemon::log()`
  reads uptime under the shared mutex; a caller that logged while holding
  that (non-reentrant) mutex froze the daemon before it served a byte, and
  lazy spawn made it look like a connect timeout. Helpers that take locks
  must never be called from under those locks — or the log path should be
  lock-free by construction.
- **macOS Keychain worked first try from an unsigned dev binary** — no prompt,
  create/read/delete all silent from a sandboxed shell. Thing to watch: ad-hoc
  signatures change on rebuild, and reading an item created by a previous
  build may prompt; if that bites, the fix is signing the binary consistently.
- **Login is a protocol verb, not a job.** Auth is interactive
  (browser-latency, no rate-limit ETA), so `auth_start`/`auth_status` fit
  better than the job queue. Token *refresh*, though, happens inside job
  execution — which means a refresh HTTP call is API traffic that currently
  bypasses the token bucket. Fine against localhost, but not against GGG:
  probes of the real endpoints (2026-08-16: one deliberate `invalid_grant`
  request, then a full real login + one refresh) answered the open question.
  The whole auth cycle is governed by **one** policy: `token-request-limit`,
  IP-scoped, `60:30:30` (60 requests / 30s, 30s restriction), behind
  Cloudflare. Authorization-code exchange and refresh share the same counter
  (observed state `1:30:0` → `2:30:0` across the two grants), and error
  responses carry the headers too. So token requests must flow through the
  daemon's rate limiter as their own policy bucket like everything else; the
  limits are generous, but the headers are there to be obeyed. (The C++ app
  currently bypasses its limiter for oauth/token — tolerable only because
  its token traffic is rare.)
- **More real-endpoint facts from the login probe (2026-08-16):** access
  tokens live 36000s (10h); refresh tokens ROTATE on every refresh, so the
  new one must be persisted after every grant or the session is lost; the
  granted scope string can come back in a different order than requested;
  and a headless GET of `/oauth/authorize` (correct UA, single request) gets
  a Cloudflare 403 with no rate-limit headers — the authorize page is
  browser-only, which is fine (only the browser ever loads it) but means its
  rate-limit regime, if any, is unobservable to us and irrelevant to the
  daemon.
