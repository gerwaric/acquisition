# Rust playground — daemon + CLI spike

Throwaway code for the daemon/CLI architecture in [CONTEXT.md](CONTEXT.md);
it exercises that doc's decisions with fake workloads. **By default nothing
here talks to GGG**: job kinds are fakes (`sleep`, `fetch`, `profile`),
OAuth runs against an in-process localhost provider (`mockggg.rs`), and the
mock's data endpoints sit behind truthfully simulated rate-limit policies
(real sliding windows, real restrictions, real 429s with `Retry-After`,
HEADs that report but don't count). The rate limiter is header-driven: it
knows nothing except what responses told it (`X-Rate-Limit-*` per policy
name, plus when counted responses arrived) and pads every wait by GGG's
server-side timing bucket. Its spec is the test table at the bottom of
`ratelimit.rs`, each row citing `docs/design/network-ground-truth.md` by
claim number. Starting the daemon with `ACQ_GGG=1` opts into the real
provider (see below). All HTTP goes through the choke point structurally:
the only `reqwest::Client` in the workspace lives inside `ChokePoint`, so
even token exchange/refresh consults the limiter and feeds its response
back, and sending without asking requires a `Paid` receipt only the limiter
can mint.

## Layout

- `crates/acquisition-core` — protocol types, job model, header-driven rate
  limiter + choke point, the mock provider, and the daemon itself (priority
  queue + dispatcher + Unix-socket server + idle watchdog). The dispatcher
  is where the burst bound lives (`MAX_IN_FLIGHT`, ground truth P-B): at
  most 2 requests in flight overall, at most one per policy, at most one
  probe ever (N18).
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
                                             # (first use of a route queues a visible `probe` job:
                                             #  one HEAD that learns the policy + current counters)
acq stashes --league Standard                # GET /stash/{league}: a second policy, runs in parallel
acq auth logout                              # drops session + keyring entry
acq submit sleep --params '{"seconds": 5}'   # blocks with progress; daemon lazy-spawns
acq demo                                     # burst of 8 fetch jobs against the mock's 5-per-10s policy; watch ETAs
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
ACQ_GGG=1 acq characters    # GET api.pathofexile.com/character
ACQ_GGG=1 acq stashes       # GET api.pathofexile.com/stash/Standard
```

`ACQ_GGG=1` on any command selects the real provider; the CLI kills and
respawns a daemon running in the wrong mode (the handshake carries the
provider name), so mock and real never mix on one daemon. Real mode uses the
existing "acquisition" registration — same client id, callback path
(`/auth/path-of-exile` on a random loopback port), scopes, and user-agent as
the shipped C++ app. Refresh tokens live in a keyring entry separate from the
mock's, so a mock token can never be sent to GGG. The limiter is the same
code in both modes and starts empty: the first job on an endpoint queues a
`probe` (a HEAD, which GGG doesn't count) that teaches the policy and the
account's current counters — including hits made by other tools — before
anything real is sent. A probe that fails or comes back without rule
definitions closes the endpoint for 60s (login reopens it). Nothing retries on
failure: a 429 or a Cloudflare-shaped 403/503 fails the job with the
evidence, and the limiter holds that policy until `Retry-After` plus the
timing bucket.

## Lifecycle & knobs

The daemon exits on its own after 60s with no connections and no live jobs —
unless the limiter still holds history inside a policy window (up to 300s),
in which case it stays up so a quick respawn doesn't have to assume the worst
about hits it can no longer see.
Its log is next to the socket (`acq daemon status` prints both paths).
`ACQ_SOCKET=<path>` overrides the socket location for parallel testing — keep
it short (Unix socket paths cap out around 104 bytes). `ACQ_NO_KEYRING=1`
degrades sessions to in-memory only (never plaintext on disk). Mock access
tokens live 60 seconds, so silent refresh is exercised constantly.

## Known gaps

- **`ACQ_MOCK_DEGRADED_HEAD=1`** makes the mock reproduce the Dec-2023
  regression (N20) so the degraded path can be exercised.
- **429s are not recovered from.** The job fails with the evidence and the
  policy is held for `Retry-After` + bucket; nothing reschedules (P-A).
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
