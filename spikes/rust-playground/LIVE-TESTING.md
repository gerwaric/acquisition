# Live testing control

Control document for testing the Rust daemon against the real GGG API. It
holds the blast-radius review that gates the first live send, the safety
rails package, the staged ladder, and the run ledger that becomes the
medium-term baseline. `CONTEXT.md` invariants and the authority order in
`NETWORK-CLEANUP.md` apply. Process is lighter than the network cleanup by
decision (2026-08-22): one independent read-only review of the rails diff,
then spike-style work recorded by outcome.

Goal: confidence that the daemon **halts rather than floods** — no HEAD
streams, no repeated rate-limit violations — so a baseline can be collected
before further development or refactoring. Not a correctness proof.

## Blast-radius review (2026-08-22, read-only, at `c1be5c39`)

Every daemon HTTP path is one of four `ChokePoint` methods (`get_bearer`,
`get`, `head`, `post_form`), each holding a gate permit from before dispatch
through body completion. Worst-case send counts per daemon lifetime, and
what bounds them:

| Path | Trigger | Bound | Verdict |
| --- | --- | --- | --- |
| HEAD probe | first job on an `Unknown` route | one per route per lifetime; a failed probe degrades the route for `PROBE_COOLDOWN` (60 s), after which the next job triggers one more | bounded: ≤ 1 HEAD / route / 60 s while jobs keep arriving |
| API GET | one per job attempt | limiter pacing from learned headers; 429 → requeue ≤ `MAX_429_RETRIES` (2) → 3 attempts max per job; 403/503 never retried; `Degraded` routes fail jobs without sending | bounded per job |
| OAuth refresh POST | any auth-required job with an expired/absent access token | singleflight while a flight is open; paced by `token-request-limit` once learned (N33: 60/30 s) | **see R1** |
| OAuth code exchange POST | one per `acq auth` | user-driven | bounded |
| Daemon restart | CLI respawns at most once per invocation on version/provider mismatch; idle exit after 60 s unless a policy window is live | probe state is in-memory, so every restart re-probes used routes | bounded: ≤ 1 HEAD / route / restart; a scripted loop of CLI calls after idle exit is the only HEAD-stream shape |
| Fan-out (`refresh --all`, `--deep`) | one `stash` child per tab / substash | data-bounded (tabs, substashes); each child is a normal job | bounded, but multiplies R1 and R2 |

Risks found (none is a bug against the accepted N0–N6 contract; all are
gaps between "correct" and "safe to run unattended against GGG"):

- **R1 — dead refresh token is retried per job.** A non-429 refresh failure
  (e.g. `400 invalid_grant` after revocation) ends the flight but leaves the
  refresh token in place. Every later auth-required job opens a new flight
  and POSTs the same dead token. Paced at ≤ 60 per 30 s by N33 once the
  policy is learned, so not a violation, but `refresh --all` over 50 tabs
  with a revoked token is 50 pointless token POSTs. (`daemon.rs`
  `finish_refresh`, `auth.rs` `token_request`.)
- **R2 — no global violation budget.** `Limiter::violations` is counted and
  never consulted. With a mis-modeled policy, a 50-child fan-out can burn
  up to 150 violations (3 attempts each), each behind a hold but each a
  counted violation against N10's revocation threshold. One bad assumption
  should cost one violation, not one per job.
- **R3 — no HTTP timeout.** The `reqwest::Client` has neither connect nor
  request timeout. A hung send holds its gate permit and scheduling key
  indefinitely: the daemon stalls (safe direction) and nothing surfaces why.
- **R4 — no durable send record.** `ChokePoint::sends` is a 100-entry
  in-memory ring; the daemon log is prose and is lost on the idle exit /
  respawn cycle that happens every couple of minutes. The baseline needs
  every real send with its headers, across restarts.
- **R5 — no hard ceiling.** Nothing caps total real sends per lifetime. Not
  needed for correctness; wanted as the backstop for the bug nobody
  imagined, during the ladder only.

Sound and unchanged: probe-before-send on every unknown route; 429 bound
and hold; Cloudflare-shape non-retry; gate permit lifetime; singleflight
refresh; mock `fetch`/`demo` refusing to map to a route in real mode; CLI
respawn bounded to once per invocation.

## L0 — live-test rails (package)

Status: `planned`. One build commit, one independent read-only review of
the exact diff with the single question *can any change add a send?*, then
use. No semantic change to gate, limiter, OAuth, classification, retry, or
dispatcher behavior; the rails only refuse or record.

1. **Tripwire (R2).** A daemon-wide halt that trips on the first counted
   violation (any landed 429, on any route, including HEAD and token) and on
   any 403/503. While tripped, every `ChokePoint` transport method fails
   fast before acquiring a permit; jobs fail with the tripwire reason;
   `acq daemon status` and `acq dash` show it. Reset is explicit:
   `acq daemon reset-tripwire`. Persisted next to the socket so a respawned
   daemon stays tripped.
2. **Dead-token stop (R1).** A non-429, non-2xx token-endpoint response
   marks the session `refresh-failed`; later refreshes fail fast without
   sending until `acq auth` or logout. Log the response evidence once.
3. **HTTP timeouts (R3).** `connect_timeout` 10 s, request timeout 60 s on
   the one client. A timeout classifies as transport failure (existing
   `Network` path); no retry.
4. **Send journal (R4).** Append-only JSONL next to the daemon log, one
   line per actual send: wall-clock time, daemon pid, method, route, URL
   path, status or transport error, every `X-Rate-Limit-*` header and
   `Retry-After`, the limiter's predicted wait before the send, and whether
   the send was counted. Written from `record_completed`, which every path
   already passes through.
5. **Send ceiling (R5).** `ACQ_MAX_SENDS=<n>`: after `n` real sends in a
   lifetime the daemon trips the tripwire with reason `ceiling`. Unset means
   no ceiling. Ladder runs set it.

Required tests: each rail has a deterministic test against the mock and the
existing fake-clock/localhost harness; the N6 integration stress passes with
all rails installed and untripped; a tripped daemon sends nothing on a
queued job. Quality gates from `NETWORK-CLEANUP.md` stay green.

## Ladder

Each rung has a stop condition; stopping means reading the journal before
the next rung, not retrying. `ACQ_GGG=1` throughout; journal on; ceiling
set per rung. Record every rung in the run ledger.

| Rung | Command(s) | Expect | Ceiling | Stop if |
| --- | --- | --- | --- | --- |
| 1 | `acq auth`, `acq auth status` | one code-exchange POST; `token-request-limit` learned with N33's shape | 2 | token response headers differ from N33; any non-2xx |
| 2 | `acq auth check` | one refresh POST; rotated refresh token persisted | 2 | refresh token not rotated; headers differ from N33 |
| 3 | `acq characters` | one HEAD on `/character` then one GET; policy shape matches ground truth | 3 | probe degrades; parser rejects headers; any 429 |
| 4 | `acq stashes --league Standard` | HEAD + GET under `stash-list-request-limit` | 3 | as rung 3 |
| 5 | `acq stash <id>` on one small tab | HEAD + GET under `stash-request-limit` | 3 | as rung 3 |
| 6 | `acq refresh --tabs a,b,c` (3 tabs) | list + 3 children; observed hits match limiter prediction | 8 | predicted vs. observed state drift > 1 hit |
| 7 | `acq refresh --tabs …` (≈10 tabs) over several minutes | pacing engages; zero 429 | 20 | any 429 |
| 8 | soak: `acq characters` every 10 min for days via cron | stable headers; idle exit / respawn / re-probe cycle is clean | 10 per daemon lifetime | any tripwire; HEAD count > 1 per restart |

Rung 8 is the baseline. Anything the rungs teach about GGG goes into
`docs/design/network-ground-truth.md` as numbered claims and is promoted
to master promptly; this file records only runs.

## Run ledger

One row per rung execution. Keep the journal files under
`spikes/rust-playground/runs/<date>-<rung>/` (gitignored) and cite them.

| Date | Rung | Daemon tip | Result | Sends (HEAD/GET/POST) | Violations | Notes |
| --- | --- | --- | --- | --- | --- | --- |

## Next action

Build L0 on a commit from `c1be5c39`; record its hash here and request the
single read-only review.
