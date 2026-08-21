# Rust playground — daemon + CLI spike

Throwaway code for the daemon/CLI architecture in [CONTEXT.md](CONTEXT.md);
it exercises that doc's decisions with fake workloads. **By default nothing
here talks to GGG**: job kinds are fakes (`sleep`, `fetch`, `profile`),
OAuth runs against an in-process localhost provider (`mockggg.rs`), and the
rate limiter is a pair of simulated token buckets (API and OAuth token
endpoint), deliberately tight so queueing is visible within seconds of play.
Starting the daemon with `ACQ_GGG=1` opts into the real provider (see
below). All HTTP goes through the choke point structurally: the only
`reqwest::Client` in the workspace lives inside `ChokePoint`, so even token
exchange/refresh pays a limiter token, and sending without paying requires a
`Paid` receipt only the limiter can mint.

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
acq characters                               # auth-required; GET /character against the mock
acq auth logout                              # drops session + keyring entry
acq submit sleep --params '{"seconds": 5}'   # blocks with progress; daemon lazy-spawns
acq demo                                     # burst of 8 fetch jobs; watch ETAs count down
acq submit fetch --detach                    # job mode: returns id immediately
acq jobs                                     # job table with ETAs
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
ACQ_GGG=1 acq characters    # GET api.pathofexile.com/character — the one real API call
```

`ACQ_GGG=1` on any command selects the real provider; the CLI kills and
respawns a daemon running in the wrong mode (the handshake carries the
provider name), so mock and real never mix on one daemon. Real mode uses the
existing "acquisition" registration — same client id, callback path
(`/auth/path-of-exile` on a random loopback port), scopes, and user-agent as
the shipped C++ app. Refresh tokens live in a keyring entry separate from the
mock's, so a mock token can never be sent to GGG. Real-mode buckets are
hard-coded far under the observed character-list policy (1 API request/60s,
2 token requests/30s burst). `X-Rate-Limit-*` headers are logged and returned
in job payloads. Nothing retries on failure: a 429 or a Cloudflare-shaped
403/503 fails the job with the evidence and a do-not-retry note.

## Lifecycle & knobs

The daemon exits on its own after 60s with no connections and no live jobs.
Its log is next to the socket (`acq daemon status` prints both paths).
`ACQ_SOCKET=<path>` overrides the socket location for parallel testing — keep
it short (Unix socket paths cap out around 104 bytes). `ACQ_NO_KEYRING=1`
degrades sessions to in-memory only (never plaintext on disk). Mock access
tokens live 60 seconds, so silent refresh is exercised constantly.

## Known gaps

- **Rate-limit headers are recorded, not obeyed.** Invariant 2 (headers
  correct local state) is not built; real mode compensates with buckets far
  under the known limits. The header-driven policy layer is the next real
  piece of core.
- **Lazy spawn hides daemon startup errors.** The spawned daemon's stderr goes
  to null, so a failed bind looks like "could not reach daemon after 5s" —
  check the daemon log.
- **Unix only.** No Windows named pipes yet; the protocol doesn't care.
- Everything in CONTEXT.md's "Explicitly deferred" list.
