# Rust playground — daemon + CLI spike

Throwaway code for the daemon/CLI architecture in [CONTEXT.md](CONTEXT.md);
it exercises that doc's decisions with fake workloads. **Nothing here talks
to GGG** — no code path in this workspace reaches a non-loopback host. Job
kinds are fakes (`sleep`, `fetch`, `profile`), OAuth runs against an
in-process localhost provider (`mockggg.rs`), and the rate limiter is a pair
of simulated token buckets (API and OAuth token endpoint), deliberately tight
so queueing is visible within seconds of play. All HTTP goes through the
choke point structurally: the only `reqwest::Client` in the workspace lives
inside `ChokePoint`, so even token exchange/refresh pays a limiter token.

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
`ACQ_SOCKET=<path>` overrides the socket location for parallel testing — keep
it short (Unix socket paths cap out around 104 bytes). `ACQ_NO_KEYRING=1`
degrades sessions to in-memory only (never plaintext on disk). Mock access
tokens live 60 seconds, so silent refresh is exercised constantly.

## Known gaps

- **Lazy spawn hides daemon startup errors.** The spawned daemon's stderr goes
  to null, so a failed bind looks like "could not reach daemon after 5s" —
  check the daemon log.
- **Unix only.** No Windows named pipes yet; the protocol doesn't care.
- Everything in CONTEXT.md's "Explicitly deferred" list.
